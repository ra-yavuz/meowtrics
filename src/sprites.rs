//! Oneko sprite loader and animations.
//!
//! Oneko is a 1990 public-domain pet animation by Tatsuya Kato. Sprites are
//! 32x32 1-bit bitmaps; we ship them as PNGs under
//! /usr/share/meowtrics/icons/neko/ (or `plasmoid/icons/neko/` in dev) and
//! load them at startup, recoloring to opaque white on transparent so the
//! tray host's panel background determines the actual rendered colour.
//!
//! Each sensor state maps to a named animation that cycles through one or
//! more of the loaded frames. The daemon advances frames on its own timer
//! (separate from the slower sensor tick) so we get smooth animation even
//! when nothing in the world changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A single animation frame: ARGB bitmap at 32x32. Stored in the `ksni::Icon`
/// width/height/data layout (ARGB32 big-endian, top-down rows, no padding).
#[derive(Clone)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
    /// ARGB32 pixels in big-endian byte order, length = width * height * 4.
    pub data: Vec<u8>,
}

/// Named animation: a list of frames played in order, cycling forever, with
/// a per-frame display duration in ticks (the daemon's frame timer).
#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
    /// How many timer ticks each frame lasts. `len(durations) == len(frames)`.
    pub durations: Vec<u32>,
}

/// The full sprite library indexed by animation name.
pub struct SpriteLibrary {
    pub animations: HashMap<&'static str, Animation>,
}

impl SpriteLibrary {
    /// Load every named animation we know about. Returns an empty library
    /// (no animations) if the sprite directory is missing, so the daemon
    /// falls back to themed icon names.
    pub fn load() -> Self {
        let dir = match find_sprite_dir() {
            Some(d) => d,
            None => {
                tracing::info!("no Oneko sprite directory found; falling back to themed icons");
                return Self {
                    animations: HashMap::new(),
                };
            }
        };
        tracing::info!("loading Oneko sprites from {}", dir.display());

        let load = |name: &str| -> Option<Frame> {
            let p = dir.join(format!("{name}.png"));
            match load_png_white_on_transparent(&p) {
                Ok(f) => Some(f),
                Err(e) => {
                    tracing::debug!("could not load sprite {}: {e:#}", p.display());
                    None
                }
            }
        };

        let mut animations: HashMap<&'static str, Animation> = HashMap::new();

        // sleep: slow snore cycle (boring states)
        if let (Some(s1), Some(s2)) = (load("sleep1"), load("sleep2")) {
            animations.insert(
                "sleep",
                Animation {
                    frames: vec![s1, s2],
                    durations: vec![16, 16], // very slow
                },
            );
        }

        // sit_calm: mostly mati2 with occasional mati3 look-up
        match (load("mati2"), load("mati3")) {
            (Some(m2), Some(m3)) => {
                animations.insert(
                    "sit_calm",
                    Animation {
                        frames: vec![m2.clone(), m2.clone(), m2, m3],
                        durations: vec![20, 20, 20, 6],
                    },
                );
            }
            (Some(m2), None) => {
                animations.insert(
                    "sit_calm",
                    Animation {
                        frames: vec![m2],
                        durations: vec![20],
                    },
                );
            }
            _ => {}
        }

        // sit_alert: ears up, looks like the cat noticed something
        if let Some(a) = load("awake") {
            animations.insert(
                "sit_alert",
                Animation {
                    frames: vec![a],
                    durations: vec![10],
                },
            );
        }

        // wash_face: groom cycle, jare2 + kaki1 + kaki2 (looks frantic at
        // higher temps because the cat is "sweating")
        if let (Some(j2), Some(k1), Some(k2)) = (load("jare2"), load("kaki1"), load("kaki2")) {
            animations.insert(
                "wash_face",
                Animation {
                    frames: vec![j2, k1, k2],
                    durations: vec![6, 4, 4],
                },
            );
        }

        // run_panic: alternating walk frames, fast (for critical states)
        if let (Some(l1), Some(l2)) = (load("dwleft1"), load("dwleft2")) {
            animations.insert(
                "run_panic",
                Animation {
                    frames: vec![l1, l2],
                    durations: vec![2, 2],
                },
            );
        }

        // dig_scratch: rapid scratching for high CPU/disk activity
        if let (Some(t1), Some(t2)) = (load("ltogi1"), load("ltogi2")) {
            animations.insert(
                "scratch",
                Animation {
                    frames: vec![t1, t2],
                    durations: vec![3, 3],
                },
            );
        }

        Self { animations }
    }

    pub fn get(&self, name: &str) -> Option<&Animation> {
        self.animations.get(name)
    }

    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

/// Pick the right animation name for a given (sensor, state) pair. Returns
/// None if the sprite library has nothing for it (caller falls back).
pub fn animation_for(sensor: &str, state: &str) -> &'static str {
    match (sensor, state) {
        // Severity overrides sensor.
        (_, "critical") => "run_panic",
        (_, "high") | (_, "hot") | (_, "low") => "wash_face",
        (_, "filling") | (_, "warm") | (_, "busy") => "sit_alert",

        ("cpu", "idle") => "sleep",
        ("ram", "idle") | ("swap", "idle") => "sleep",
        ("thermal", "cool") | ("thermal", "idle") => "sleep",

        ("battery", "full") | ("battery", "charging") => "sit_calm",
        ("battery", "discharging") => "sit_alert",

        ("uptime", "ancient") | ("uptime", "long") => "sleep",
        ("uptime", "fresh") => "sit_alert",

        // Default: calm sit.
        _ => "sit_calm",
    }
}

fn find_sprite_dir() -> Option<PathBuf> {
    // System install (the .deb drops the icons here):
    let system = PathBuf::from("/usr/share/meowtrics/icons/neko");
    if system.is_dir() {
        return Some(system);
    }
    // Dev tree (cargo run from the repo root):
    let dev = PathBuf::from("plasmoid/icons/neko");
    if dev.is_dir() {
        return Some(dev);
    }
    None
}

/// Decode a 1-bit-grayscale (or any) PNG into ARGB32 bytes, treating any
/// non-transparent dark pixel as opaque white. This makes the sprite "ink"
/// blend with whatever panel background the user has.
///
/// Output buffer layout: ARGB big-endian, top-down rows, length = w*h*4.
fn load_png_white_on_transparent(path: &Path) -> anyhow::Result<Frame> {
    use image::GenericImageView;
    let img = image::open(path)?;
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for px in rgba.pixels() {
        // Treat any pixel with non-zero alpha or non-white luminance as "ink".
        let lum = (px[0] as u32 + px[1] as u32 + px[2] as u32) / 3;
        let is_ink = px[3] > 0 && lum < 200;
        if is_ink {
            // ARGB big-endian: A, R, G, B = FF, FF, FF, FF (opaque white)
            out.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        } else {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }
    }
    Ok(Frame {
        width: w as i32,
        height: h as i32,
        data: out,
    })
}
