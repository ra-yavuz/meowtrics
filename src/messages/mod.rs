//! Message database: emoji sets and weighted prose templates per sensor state.
//!
//! Resolution order, lowest precedence to highest:
//!   1. Bundled defaults at /usr/share/meowtrics/messages.json (or ./data/messages.json in dev)
//!   2. Cached refresh at $XDG_CACHE_HOME/meowtrics/messages.json (daily auto-refresh)
//!   3. User overrides at $XDG_CONFIG_HOME/meowtrics/messages.json
//!
//! Remote source: https://ra-yavuz.github.io/meowtrics/messages.json

use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::state::StableState;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Database {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub updated: String,
    pub sensors: HashMap<String, HashMap<String, StatePack>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatePack {
    /// Emoji shown in the tray for this state. Daemon picks one per tick.
    pub emoji: Vec<String>,
    /// Hover messages, with optional weight (higher = more frequent).
    pub messages: Vec<MessageTemplate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageTemplate {
    pub text: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

impl Database {
    /// Load from disk, layering bundled < cached < user overrides.
    pub fn load() -> Result<Self> {
        let mut db = Self::default();
        for path in candidate_paths() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<Database>(&text) {
                    Ok(layer) => {
                        tracing::info!("loaded messages from {}", path.display());
                        db.merge(layer);
                    }
                    Err(e) => tracing::warn!("could not parse {}: {e}", path.display()),
                }
            }
        }
        if db.sensors.is_empty() {
            anyhow::bail!(
                "no messages.json found in any of: bundled, ~/.cache/meowtrics, ~/.config/meowtrics"
            );
        }
        Ok(db)
    }

    fn merge(&mut self, other: Database) {
        if other.version > 0 {
            self.version = other.version;
        }
        if !other.updated.is_empty() {
            self.updated = other.updated;
        }
        for (sensor, states) in other.sensors {
            self.sensors.entry(sensor).or_default().extend(states);
        }
    }

    /// Pick an emoji for the active state.
    pub fn pick_emoji(&self, sensor: &str, state: &str) -> Option<String> {
        let pack = self.sensors.get(sensor)?.get(state)?;
        pack.emoji.choose(&mut rand::thread_rng()).cloned()
    }

    /// Render a weighted-random message for the active state, with template substitution.
    pub fn render_message(&self, s: &StableState) -> Option<String> {
        let pack = self.sensors.get(s.sensor.as_str())?.get(s.state)?;
        if pack.messages.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let total: u32 = pack.messages.iter().map(|m| m.weight.max(1)).sum();
        let pick = rand::Rng::gen_range(&mut rng, 0..total);
        let mut acc = 0u32;
        let template = pack
            .messages
            .iter()
            .find(|m| {
                acc += m.weight.max(1);
                acc > pick
            })
            .unwrap_or(&pack.messages[0]);
        Some(fill_template(&template.text, s))
    }
}

/// Substitute `{value}`, `{duration}`, etc. in a template string.
fn fill_template(template: &str, s: &StableState) -> String {
    let mut out = template.to_string();
    out = out.replace("{value}", &format_value(s.last_value));
    out = out.replace("{duration}", &humanize_duration(s.duration_in_state()));
    if let Some(ctx) = s.context.as_ref().and_then(|c| c.as_object()) {
        for (k, v) in ctx {
            let placeholder = format!("{{{}}}", k);
            let val_str = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => v.to_string(),
            };
            out = out.replace(&placeholder, &val_str);
        }
    }
    out
}

fn format_value(v: f64) -> String {
    if v.fract().abs() < 0.05 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

fn humanize_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h{}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}d", s / 86400)
    }
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    // Bundled (system install)
    out.push(PathBuf::from("/usr/share/meowtrics/messages.json"));
    // Local checkout for `cargo run` from the repo root
    out.push(PathBuf::from("data/messages.json"));
    // Cache (auto-refresh) and config (user overrides)
    if let Some(dirs) = directories::ProjectDirs::from("io", "ra-yavuz", "meowtrics") {
        out.push(dirs.cache_dir().join("messages.json"));
        out.push(dirs.config_dir().join("messages.json"));
    }
    out.into_iter().filter(|p| Path::new(p).exists()).collect()
}

/// Trigger an out-of-band refresh of the cached messages.json.
pub async fn refresh_now() -> Result<()> {
    let url = "https://ra-yavuz.github.io/meowtrics/messages.json";
    let resp = reqwest::get(url).await.context("fetching messages.json")?;
    if !resp.status().is_success() {
        anyhow::bail!("fetch failed: HTTP {}", resp.status());
    }
    let body = resp.text().await?;
    // Validate before writing
    let _: Database = serde_json::from_str(&body).context("parsing remote messages.json")?;
    let dirs = directories::ProjectDirs::from("io", "ra-yavuz", "meowtrics")
        .context("could not determine cache dir")?;
    std::fs::create_dir_all(dirs.cache_dir())?;
    let dest = dirs.cache_dir().join("messages.json");
    let tmp = dest.with_extension("json.tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, &dest)?;
    println!("refreshed: {}", dest.display());
    Ok(())
}
