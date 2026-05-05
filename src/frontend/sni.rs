//! StatusNotifierItem tray frontend.
//!
//! Architecture:
//!   - daemon.rs ticks sensors every TICK_SECS, computes a snapshot, and
//!     writes the picked-animation name + the multi-line tooltip into a
//!     shared TrayState.
//!   - A separate, faster animation timer (FRAME_SECS = 0.25s) advances
//!     the current animation's frame index and pushes the corresponding
//!     ARGB pixmap into the SNI Tray via ksni's Handle::update.
//!
//! The shared state lock is std::sync::RwLock (not tokio): ksni's Tray
//! trait methods are sync but run inside ksni's tokio runtime, and a
//! tokio lock would panic on blocking_read there.

use std::sync::{Arc, RwLock};

use anyhow::Result;
use ksni::{Icon, MenuItem, Tray, TrayMethods};

use crate::sprites::{Animation, SpriteLibrary};
use crate::state::StableState;

/// Shared state that the daemon writes and the SNI methods read.
#[derive(Clone, Default)]
pub struct TrayState {
    /// Title shown in the tooltip header (a short label, normally "meowtrics").
    pub title: String,
    /// Multi-line description shown under the tooltip header. Includes the
    /// catchy random message + a per-sensor table.
    pub tooltip: String,
    /// Name of the active animation in `SpriteLibrary`, or empty string for
    /// the fallback themed icon path.
    pub animation_name: String,
    /// Themed icon name to use when the sprite library is empty / animation
    /// is unknown (StatusNotifierWatcher's IconName fallback).
    pub fallback_icon_name: String,
}

pub struct MeowtricsTray {
    state: Arc<RwLock<TrayState>>,
    sprites: Arc<SpriteLibrary>,
    /// Frame index inside the current animation. Lives on the tray itself
    /// rather than the shared state so the daemon doesn't need to know.
    pub frame_index: usize,
    /// How many ticks the current frame has been displayed for.
    pub ticks_on_frame: u32,
    /// Cached animation name we last advanced (so we can reset frame_index
    /// when the daemon switches us to a new animation).
    last_animation: String,
}

impl MeowtricsTray {
    fn current_animation(&self) -> Option<&Animation> {
        let name = self.state.read().ok()?.animation_name.clone();
        if name.is_empty() {
            None
        } else {
            self.sprites.get(&name)
        }
    }

    fn current_frame_pixmap(&self) -> Option<Icon> {
        let anim = self.current_animation()?;
        let f = anim.frames.get(self.frame_index)?;
        Some(Icon {
            width: f.width,
            height: f.height,
            data: f.data.clone(),
        })
    }
}

impl Tray for MeowtricsTray {
    fn id(&self) -> String {
        "com.ra-yavuz.meowtrics".to_string()
    }

    fn title(&self) -> String {
        self.state
            .read()
            .ok()
            .map(|g| {
                if g.title.is_empty() {
                    "meowtrics".to_string()
                } else {
                    g.title.clone()
                }
            })
            .unwrap_or_else(|| "meowtrics".to_string())
    }

    fn icon_name(&self) -> String {
        // Used when icon_pixmap is empty (no sprite library loaded). Otherwise
        // the pixmap wins on most tray hosts.
        self.state
            .read()
            .ok()
            .map(|g| {
                if g.fallback_icon_name.is_empty() {
                    "meowtrics".to_string()
                } else {
                    g.fallback_icon_name.clone()
                }
            })
            .unwrap_or_else(|| "meowtrics".to_string())
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // Sprite path: ship the current frame as the tray pixmap.
        self.current_frame_pixmap()
            .map(|i| vec![i])
            .unwrap_or_default()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let g = self.state.read();
        let (title, description, icon_name) = match g {
            Ok(g) => (
                if g.title.is_empty() {
                    "meowtrics".to_string()
                } else {
                    g.title.clone()
                },
                g.tooltip.clone(),
                g.fallback_icon_name.clone(),
            ),
            Err(_) => ("meowtrics".to_string(), String::new(), String::new()),
        };
        let icon_pixmap = self
            .current_frame_pixmap()
            .map(|i| vec![i])
            .unwrap_or_default();
        ksni::ToolTip {
            icon_name,
            icon_pixmap,
            title,
            description,
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

/// Compute the TrayState for a sensor snapshot: pick the active sensor,
/// derive the animation name, render the multi-line tooltip body.
pub fn render_state(snap: &[StableState], db: &crate::messages::Database) -> TrayState {
    // Priority for picking the "most interesting" sensor for the icon.
    // Boring states (idle / fresh / full / long / ancient / off / cool) are
    // excluded so e.g. uptime never wins after a week.
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
        .cloned()
        .or_else(|| snap.iter().max_by_key(|s| priority(s.state)).cloned());

    let (animation_name, fallback_icon_name, headline) = if let Some(a) = active.as_ref() {
        (
            crate::sprites::animation_for(a.sensor.as_str(), a.state).to_string(),
            freedesktop_icon_for(a.sensor.as_str(), a.state),
            db.render_message(a)
                .unwrap_or_else(|| format!("{} {}", a.sensor.as_str(), a.state)),
        )
    } else {
        (
            "sit_calm".to_string(),
            "meowtrics".to_string(),
            "starting up".to_string(),
        )
    };

    // Multi-line tooltip: catchy headline, then a per-sensor table.
    let mut tooltip = String::with_capacity(256);
    tooltip.push_str(&headline);
    for s in snap {
        let e = db
            .pick_emoji(s.sensor.as_str(), s.state)
            .unwrap_or_else(|| "  ".to_string());
        tooltip.push('\n');
        tooltip.push_str(&format!(
            "{}  {:<8} {:<10} {:>6.1}",
            e,
            s.sensor.as_str(),
            s.state,
            s.last_value
        ));
    }

    TrayState {
        title: "meowtrics".to_string(),
        tooltip,
        animation_name,
        fallback_icon_name,
    }
}

/// Map (sensor, state) to a freedesktop standard icon name. Only used when
/// the sprite library is empty (no Oneko PNGs found).
fn freedesktop_icon_for(sensor: &str, state: &str) -> String {
    let s = match (sensor, state) {
        (_, "critical") => "dialog-error",
        (_, "high") | (_, "hot") => "dialog-warning",
        ("cpu", "busy") => "system-run",
        ("cpu", "idle") | ("cpu", "normal") => "computer",
        ("ram", _) => "memory",
        ("swap", _) => "drive-harddisk",
        ("thermal", "warm") => "weather-clear",
        ("thermal", "cool") | ("thermal", "idle") => "weather-clear-night",
        ("thermal", _) => "computer",
        ("battery", "full") => "battery-full",
        ("battery", "charging") => "battery-good-charging",
        ("battery", "low") => "battery-caution",
        ("battery", "discharging") => "battery-good",
        ("battery", _) => "battery",
        ("disk", _) => "drive-harddisk",
        ("load", "busy") => "system-run",
        ("load", _) => "computer",
        ("uptime", "ancient") => "appointment-soon",
        ("uptime", _) => "computer",
        _ => "meowtrics",
    };
    s.to_string()
}

/// Spawn the tray and return a handle the daemon can use to advance frames.
pub async fn spawn(
    state: Arc<RwLock<TrayState>>,
    sprites: Arc<SpriteLibrary>,
) -> Result<ksni::Handle<MeowtricsTray>> {
    let tray = MeowtricsTray {
        state,
        sprites,
        frame_index: 0,
        ticks_on_frame: 0,
        last_animation: String::new(),
    };
    let handle = tray.spawn().await?;
    Ok(handle)
}

/// Advance the animation by one frame tick. Should be called from the daemon
/// on the frame timer (FRAME_SECS). Inspects the shared state to see what
/// animation should be playing; resets frame_index when the daemon switched
/// us to a new animation; otherwise honours the current frame's duration.
pub async fn advance_frame(
    handle: &ksni::Handle<MeowtricsTray>,
    sprites: &SpriteLibrary,
    state: &Arc<RwLock<TrayState>>,
) {
    let target = state
        .read()
        .ok()
        .map(|g| g.animation_name.clone())
        .unwrap_or_default();
    let anim = sprites.get(&target);
    handle
        .update(|tray| {
            // Switch animations cleanly when the daemon points us elsewhere.
            if tray.last_animation != target {
                tray.last_animation = target.clone();
                tray.frame_index = 0;
                tray.ticks_on_frame = 0;
            }
            let Some(anim) = anim else { return };
            tray.ticks_on_frame += 1;
            let dur = anim.durations.get(tray.frame_index).copied().unwrap_or(8);
            if tray.ticks_on_frame >= dur {
                tray.ticks_on_frame = 0;
                tray.frame_index = (tray.frame_index + 1) % anim.frames.len();
            }
        })
        .await;
}
