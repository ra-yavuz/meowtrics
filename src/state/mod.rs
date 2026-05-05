//! Sensor state machines with debouncing.
//!
//! Raw readings are noisy (CPU spikes for one tick, RAM blips during a process exit).
//! Messages like "RAM has been thirsty for 12 minutes" depend on a *stable* state
//! that only changes when the underlying signal is sustained for a debounce window.
//!
//! Each sensor maps a raw [`Reading`] to a [`SensorState`] (Idle/Normal/High/Critical/...)
//! via per-sensor thresholds, then a debouncer holds the previous state until the new
//! one has been seen for N consecutive ticks. The debounced state carries `entered_at`
//! so messages can render durations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::sensors::{Reading, SensorId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorState {
    Idle,
    Normal,
    Busy,
    High,
    Critical,
}

impl SensorState {
    pub fn as_str(self) -> &'static str {
        match self {
            SensorState::Idle => "idle",
            SensorState::Normal => "normal",
            SensorState::Busy => "busy",
            SensorState::High => "high",
            SensorState::Critical => "critical",
        }
    }
}

/// Stable, debounced state of a sensor.
#[derive(Clone, Debug, Serialize)]
pub struct StableState {
    pub sensor: SensorId,
    pub state: SensorState,
    /// When the current state was first entered (for "for X minutes" messages).
    #[serde(skip_serializing)]
    pub entered_at: Instant,
    /// Last raw reading value, for templates that want to quote it.
    pub last_value: f64,
}

impl StableState {
    pub fn duration_in_state(&self) -> Duration {
        self.entered_at.elapsed()
    }
}

/// Tracks one sensor's state over time, applying a debounce.
struct Tracker {
    current: SensorState,
    candidate: Option<(SensorState, u32)>, // (state being voted on, consecutive ticks seen)
    entered_at: Instant,
    last_value: f64,
}

const DEBOUNCE_TICKS: u32 = 3;

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
            let tr = self.trackers.entry(r.sensor).or_insert(Tracker {
                current: raw_state,
                candidate: None,
                entered_at: now,
                last_value: r.value,
            });
            tr.last_value = r.value;
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
            })
            .collect()
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

fn classify(r: &Reading) -> SensorState {
    match r.sensor {
        SensorId::Cpu => match r.value {
            v if v < 5.0 => SensorState::Idle,
            v if v < 40.0 => SensorState::Normal,
            v if v < 75.0 => SensorState::Busy,
            v if v < 95.0 => SensorState::High,
            _ => SensorState::Critical,
        },
    }
}
