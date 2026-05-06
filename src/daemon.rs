//! Daemon main loop.
//!
//! Two timers:
//!   - **sensor tick** (TICK_SECS = 5s): read all sensors, run the state
//!     machine, decide the active animation, write new TrayState.
//!   - **frame tick** (FRAME_SECS = 0.25s): advance the current animation
//!     by one frame. Pure visual; doesn't touch sensors.
//!
//! Both run on the same single-threaded tokio runtime; the frame tick stays
//! cheap so it never starves sensor reads.

use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

use crate::frontend::sni::{advance_frame, render_state, spawn as spawn_sni, TrayState};
use crate::messages::Database;
use crate::sensors::read_all;
use crate::sprites::SpriteLibrary;
use crate::state::StateMachine;

const TICK_SECS: u64 = 5;
const FRAME_MS: u64 = 250;

/// How long we keep showing the same headline for the same (sensor, state)
/// before re-rolling. The plasmoid calls `meowtrics tray-state` every 5s;
/// without this the headline flickers between weighted-random picks every
/// tick, which is jarring on a busy machine. 5 minutes feels right: long
/// enough to read and absorb, short enough that you do see variety.
const HEADLINE_HOLD_SECS: u64 = 300;

pub async fn run(enable_tray: bool) -> Result<()> {
    tracing::info!("meowtrics v{} starting", env!("CARGO_PKG_VERSION"));
    println!(
        "meowtrics v{}: provided AS IS, without warranty of any kind. By running this software you accept full responsibility. See README for full disclaimer.",
        env!("CARGO_PKG_VERSION")
    );

    let db = match Database::load() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            tracing::warn!("messages database not loaded: {e:#}; running with placeholder text");
            Arc::new(Database::default())
        }
    };

    let sprites = Arc::new(SpriteLibrary::load());
    if sprites.is_empty() {
        tracing::info!("running without sprite animation; SNI fallback will use themed icon names");
    } else {
        tracing::info!(
            "sprite library loaded: {} animations",
            sprites.animations.len()
        );
    }

    let tray_state = Arc::new(RwLock::new(TrayState::default()));

    // SNI tray icon is opt-in via --tray. The default UX is the KDE plasmoid
    // panel widget, which talks to the daemon via `meowtrics tray-state`.
    // Users on desktops without a plasmoid surface can pass --tray to ALSO
    // register a StatusNotifierItem.
    if enable_tray {
        let tray_handle = match spawn_sni(tray_state.clone(), sprites.clone()).await {
            Ok(h) => {
                tracing::info!("SNI tray icon registered (--tray)");
                Some(h)
            }
            Err(e) => {
                tracing::warn!("could not register SNI tray icon: {e:#}");
                None
            }
        };
        if let Some(handle) = tray_handle.clone() {
            let sprites_for_anim = sprites.clone();
            let state_for_anim = tray_state.clone();
            tokio::spawn(async move {
                loop {
                    advance_frame(&handle, &sprites_for_anim, &state_for_anim).await;
                    sleep(Duration::from_millis(FRAME_MS)).await;
                }
            });
        }
    } else {
        tracing::info!(
            "headless mode (no system tray icon). The KDE plasmoid panel widget reads `meowtrics tray-state` directly. Pass --tray to also register an SNI icon."
        );
    }

    let mut sm = StateMachine::new();
    loop {
        let readings = read_all().await;
        sm.ingest(&readings);
        let snap = sm.snapshot();
        for s in &snap {
            tracing::debug!(
                "{}: {} (last={:.1}, in_state_for={:.1}s)",
                s.sensor.as_str(),
                s.state,
                s.last_value,
                s.duration_in_state().as_secs_f64()
            );
        }
        let new_tray = render_state(&snap, &db);
        *tray_state.write().expect("tray state lock poisoned") = new_tray;
        sleep(Duration::from_secs(TICK_SECS)).await;
    }
}

pub async fn print_status(short: bool) -> Result<()> {
    let db = Database::load().ok();
    let readings = read_all().await;
    let mut sm = StateMachine::new();
    sm.ingest(&readings);
    let snap = sm.snapshot();
    if short {
        if let Some(s) = snap.iter().max_by(|a, b| {
            a.last_value
                .partial_cmp(&b.last_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            let emoji = db
                .as_ref()
                .and_then(|d| d.pick_emoji(s.sensor.as_str(), s.state))
                .unwrap_or_else(|| "🐈".to_string());
            println!("{} {} {:.0}", emoji, s.sensor.as_str(), s.last_value);
        } else {
            println!("🐈");
        }
    } else {
        for s in snap {
            let emoji = db
                .as_ref()
                .and_then(|d| d.pick_emoji(s.sensor.as_str(), s.state))
                .unwrap_or_else(|| "  ".to_string());
            println!(
                "{} {:<10} {:<10} {:>7.1}",
                emoji,
                s.sensor.as_str(),
                s.state,
                s.last_value
            );
        }
    }
    Ok(())
}

pub async fn print_json() -> Result<()> {
    let readings = read_all().await;
    println!("{}", serde_json::to_string(&readings)?);
    Ok(())
}

/// Emit the pre-classified tray state as JSON for the plasmoid:
///   { "animation": "wash_face",
///     "headline": "cpu has been at 92% for 3m. send help.",
///     "sensors": [ { "sensor": "cpu", "state": "high",
///                    "value": 92.4, "emoji": "🥵" }, ... ] }
///
/// The plasmoid is a dumb renderer: it picks frames from `animation`,
/// shows `headline` in the popup, and tabulates `sensors` in the body.
///
/// The plasmoid calls this every 5s. To avoid the headline rotating on
/// every call (jarring on a busy machine), we cache the picked headline
/// per (sensor, state) for HEADLINE_HOLD_SECS at ~/.cache/meowtrics/
/// tray-state.cache. New picks happen on state change OR every N minutes.
pub async fn print_tray_state() -> Result<()> {
    use serde_json::json;
    let db = Database::load().ok();
    let readings = read_all().await;
    let mut sm = StateMachine::new();
    sm.ingest(&readings);
    let snap = sm.snapshot();

    // Mirror the SNI priority logic so the plasmoid icon matches the tray icon.
    let priority = |state: &str| -> i8 {
        match state {
            "critical" => 5,
            "high" | "hot" | "low" => 4,
            "filling" | "warm" | "busy" => 3,
            "normal" | "charging" | "discharging" => 1,
            "idle" | "fresh" | "full" | "long" | "ancient" | "off" | "cool" => -1,
            _ => 0,
        }
    };
    let active = snap
        .iter()
        .filter(|s| priority(s.state) >= 0)
        .max_by_key(|s| priority(s.state))
        .or_else(|| snap.iter().max_by_key(|s| priority(s.state)));

    let (animation, headline) = match active {
        Some(a) => {
            let cache_key = format!("{}:{}", a.sensor.as_str(), a.state);
            let cached = read_headline_cache().and_then(|c| c.get(&cache_key).cloned());
            let h = match cached {
                Some(entry) if entry.is_fresh() => entry.headline,
                _ => {
                    let new_h = db
                        .as_ref()
                        .and_then(|d| d.render_message(a))
                        .unwrap_or_else(|| format!("{} {}", a.sensor.as_str(), a.state));
                    write_headline_cache_entry(&cache_key, &new_h);
                    new_h
                }
            };
            (
                crate::sprites::animation_for(a.sensor.as_str(), a.state).to_string(),
                h,
            )
        }
        None => ("sit_calm".to_string(), "starting up".to_string()),
    };

    let sensors: Vec<_> = snap
        .iter()
        .map(|s| {
            let emoji = db
                .as_ref()
                .and_then(|d| d.pick_emoji(s.sensor.as_str(), s.state))
                .unwrap_or_default();
            json!({
                "sensor": s.sensor.as_str(),
                "state": s.state,
                "value": s.last_value,
                "emoji": emoji,
            })
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string(&json!({
            "animation": animation,
            "headline": headline,
            "sensors": sensors,
        }))?
    );
    Ok(())
}

pub async fn print_sensors() -> Result<()> {
    use crate::sensors::SensorId;
    for id in SensorId::all() {
        println!("{}", id.as_str());
    }
    Ok(())
}

// ---------------------------------------------------------------------
// Headline cache: persists the last-picked random headline per
// (sensor, state) at ~/.cache/meowtrics/tray-state.cache so that the
// plasmoid's 5-second tray-state polling doesn't reroll the headline
// every tick.
// ---------------------------------------------------------------------

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct HeadlineEntry {
    headline: String,
    /// Unix timestamp when this entry was written.
    picked_at: u64,
}

impl HeadlineEntry {
    fn is_fresh(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.picked_at) < HEADLINE_HOLD_SECS
    }
}

fn cache_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("io", "ra-yavuz", "meowtrics").map(|dirs| {
        let p = dirs.cache_dir().to_path_buf();
        p.join("tray-state.cache")
    })
}

fn read_headline_cache() -> Option<std::collections::HashMap<String, HeadlineEntry>> {
    let path = cache_path()?;
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_headline_cache_entry(key: &str, headline: &str) {
    let Some(path) = cache_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut map = read_headline_cache().unwrap_or_default();
    map.insert(
        key.to_string(),
        HeadlineEntry {
            headline: headline.to_string(),
            picked_at: now,
        },
    );
    // Keep the cache small: prune entries older than 6h on every write.
    map.retain(|_, e| now.saturating_sub(e.picked_at) < 6 * 3600);
    if let Ok(text) = serde_json::to_string(&map) {
        let _ = std::fs::write(&path, text);
    }
}
