//! Sensor state machines with debouncing.
//!
//! Raw readings are noisy (CPU spikes for one tick, RAM blips during a process exit).
//! Each sensor maps a raw [`Reading`] to a [`SensorState`] via per-sensor thresholds,
//! then a debouncer holds the previous state until the new one has been seen for N
//! consecutive ticks. The debounced state carries `entered_at` so messages can
//! render "for X minutes" durations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::sensors::{Reading, SensorId};

/// State name as used in the messages.json schema. Free-form string so each sensor
/// can use the vocabulary that fits its domain (battery: charging/low; thermal: cool/hot/critical).
pub type SensorState = &'static str;

/// Stable, debounced state of a sensor.
#[derive(Clone, Debug)]
pub struct StableState {
    pub sensor: SensorId,
    pub state: SensorState,
    /// When the current state was first entered (for "for X minutes" messages).
    pub entered_at: Instant,
    /// Last raw reading value, for templates that want to quote it.
    pub last_value: f64,
    /// Last raw context, for sensors that carry extra info (battery status, ram totals, etc.).
    pub context: Option<serde_json::Value>,
}

impl StableState {
    pub fn duration_in_state(&self) -> Duration {
        self.entered_at.elapsed()
    }
}

/// Tracks one sensor's state over time, applying a debounce.
struct Tracker {
    current: SensorState,
    candidate: Option<(SensorState, u32)>,
    entered_at: Instant,
    last_value: f64,
    context: Option<serde_json::Value>,
}

const DEBOUNCE_TICKS: u32 = 2;

pub struct StateMachine {
    trackers: HashMap<SensorId, Tracker>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, readings: &[Reading]) {
        for r in readings {
            let raw_state = classify(r);
            let now = Instant::now();
            let tr = self.trackers.entry(r.sensor).or_insert_with(|| Tracker {
                current: raw_state,
                candidate: None,
                entered_at: now,
                last_value: r.value,
                context: r.context.clone(),
            });
            tr.last_value = r.value;
            tr.context = r.context.clone();
            if raw_state == tr.current {
                tr.candidate = None;
            } else {
                let next = match tr.candidate {
                    Some((s, n)) if s == raw_state => (s, n + 1),
                    _ => (raw_state, 1),
                };
                if next.1 >= DEBOUNCE_TICKS {
                    tr.current = next.0;
                    tr.entered_at = now;
                    tr.candidate = None;
                } else {
                    tr.candidate = Some(next);
                }
            }
        }
    }

    pub fn snapshot(&self) -> Vec<StableState> {
        self.trackers
            .iter()
            .map(|(id, tr)| StableState {
                sensor: *id,
                state: tr.current,
                entered_at: tr.entered_at,
                last_value: tr.last_value,
                context: tr.context.clone(),
            })
            .collect()
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify a raw reading into a state name. Thresholds tuned for "interesting on a laptop".
/// State names match keys in `data/messages.json`.
fn classify(r: &Reading) -> SensorState {
    match r.sensor {
        SensorId::Cpu => match r.value {
            v if v < 5.0 => "idle",
            v if v < 40.0 => "normal",
            v if v < 75.0 => "busy",
            v if v < 95.0 => "high",
            _ => "critical",
        },
        SensorId::Ram => match r.value {
            v if v < 30.0 => "idle",
            v if v < 65.0 => "normal",
            v if v < 90.0 => "high",
            _ => "critical",
        },
        SensorId::Swap => match r.value {
            v if v < 1.0 => "idle",
            v if v < 25.0 => "normal",
            v if v < 75.0 => "high",
            _ => "critical",
        },
        SensorId::Thermal => match r.value {
            v if v < 45.0 => "idle",
            v if v < 60.0 => "normal",
            v if v < 75.0 => "warm",
            v if v < 90.0 => "hot",
            _ => "critical",
        },
        SensorId::Battery => {
            // Battery state combines capacity and charging status from context.
            let charging = r
                .context
                .as_ref()
                .and_then(|c| c.get("status"))
                .and_then(|s| s.as_str())
                .map(|s| s.eq_ignore_ascii_case("Charging") || s.eq_ignore_ascii_case("Full"))
                .unwrap_or(false);
            match (r.value, charging) {
                (v, _) if v >= 99.0 => "full",
                (_, true) => "charging",
                (v, false) if v < 10.0 => "critical",
                (v, false) if v < 25.0 => "low",
                _ => "discharging",
            }
        }
        SensorId::Disk => match r.value {
            v if v < 50.0 => "idle",
            v if v < 80.0 => "normal",
            v if v < 90.0 => "filling",
            v if v < 97.0 => "high",
            _ => "critical",
        },
        SensorId::Load => match r.value {
            v if v < 1.0 => "idle",
            v if v < 4.0 => "normal",
            v if v < 8.0 => "high",
            _ => "critical",
        },
        SensorId::Uptime => match r.value {
            v if v < 3600.0 => "fresh",
            v if v < 7.0 * 86400.0 => "long",
            _ => "ancient",
        },
    }
}
