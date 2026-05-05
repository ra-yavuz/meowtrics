//! StatusNotifierItem tray frontend.
//!
//! Approach: register a tray item via `ksni` whose IconName maps from the most
//! "interesting" sensor's state to a freedesktop standard icon name (face-smile,
//! weather-clear, dialog-warning, ...). The emoji and current message are shown
//! in the tooltip, which every modern tray host (KDE, GNOME+ext, XFCE 4.16+,
//! Cinnamon, Budgie, MATE, LXQt, waybar) renders on hover.
//!
//! v0.1 deliberately avoids rendering emoji to a pixmap, because pure-Rust
//! emoji rendering with the system color emoji font is non-trivial (no
//! Rust crate handles COLR/CPAL or CBDT/CBLC tables out-of-the-box).
//! v0.2 will switch to IconPixmap rendering once we ship a bundled emoji
//! pack (PNG sprites at panel sizes).

use anyhow::Result;
use ksni::{Icon, MenuItem, Tray, TrayMethods};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::StableState;

#[derive(Clone, Default)]
pub struct TrayState {
    pub icon_name: String,
    pub title: String,
    pub tooltip_subtitle: String,
}

pub struct MeowtricsTray {
    state: Arc<RwLock<TrayState>>,
}

impl Tray for MeowtricsTray {
    fn id(&self) -> String {
        "com.ra-yavuz.meowtrics".to_string()
    }

    fn title(&self) -> String {
        let g = self.state.blocking_read();
        if g.title.is_empty() {
            "meowtrics".to_string()
        } else {
            g.title.clone()
        }
    }

    fn icon_name(&self) -> String {
        let g = self.state.blocking_read();
        if g.icon_name.is_empty() {
            "face-smile".to_string()
        } else {
            g.icon_name.clone()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        Vec::new()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let g = self.state.blocking_read();
        ksni::ToolTip {
            icon_name: g.icon_name.clone(),
            icon_pixmap: Vec::new(),
            title: format!(
                "{} {}",
                if g.title.is_empty() { "🐈" } else { &g.title },
                "meowtrics"
            ),
            description: g.tooltip_subtitle.clone(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Refresh messages".into(),
                activate: Box::new(|_: &mut Self| {
                    tokio::spawn(async {
                        if let Err(e) = crate::messages::refresh_now().await {
                            tracing::warn!("refresh failed: {e:#}");
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "About meowtrics".into(),
                activate: Box::new(|_: &mut Self| {
                    let _ = open::that("https://ra-yavuz.github.io/meowtrics/");
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_: &mut Self| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Map a snapshot of sensor states to a tray title/tooltip + a freedesktop icon name.
pub fn render_state(snap: &[StableState], db: &crate::messages::Database) -> TrayState {
    // Pick the "most interesting" sensor: highest severity wins.
    let priority = |state: &str| -> u8 {
        match state {
            "critical" => 5,
            "high" | "hot" | "low" => 4,
            "filling" | "warm" | "busy" => 3,
            "charging" | "discharging" | "normal" => 2,
            "idle" | "fresh" | "long" | "full" => 1,
            _ => 1,
        }
    };
    let active = snap
        .iter()
        .max_by_key(|s| priority(s.state))
        .cloned();

    let (emoji, message, icon_name) = if let Some(a) = active {
        let emoji = db
            .pick_emoji(a.sensor.as_str(), a.state)
            .unwrap_or_else(|| "🐈".to_string());
        let msg = db
            .render_message(&a)
            .unwrap_or_else(|| format!("{} {}", a.sensor.as_str(), a.state));
        let icon = freedesktop_icon_for(a.state);
        (emoji, msg, icon)
    } else {
        ("🐈".to_string(), "starting up".to_string(), "face-smile".to_string())
    };

    TrayState {
        icon_name,
        title: emoji,
        tooltip_subtitle: message,
    }
}

fn freedesktop_icon_for(state: &str) -> String {
    match state {
        "critical" => "dialog-warning",
        "high" | "hot" => "weather-storm",
        "low" => "battery-caution",
        "warm" | "busy" | "filling" | "charging" => "weather-clear",
        "idle" | "fresh" | "full" | "normal" | "long" => "face-smile",
        _ => "face-smile",
    }
    .to_string()
}

pub async fn spawn(state: Arc<RwLock<TrayState>>) -> Result<()> {
    let tray = MeowtricsTray { state };
    let _handle = tray.spawn().await?;
    Ok(())
}
