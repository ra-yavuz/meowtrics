//! Sensor readings.
//!
//! Each sensor is a small module that reads from `/proc` or `/sys` (no root, no extra deps)
//! and returns a [`Reading`]. The state machine in [`crate::state`] consumes a stream of
//! readings and derives a stable `SensorState` for each sensor, which the messages layer
//! and the tray frontend then render.
//!
//! Add a new sensor by:
//!   1. Creating a module here that exposes `pub async fn read() -> anyhow::Result<Reading>`
//!   2. Adding it to [`SensorId`] and to the `read_all` dispatcher
//!   3. Defining its state-machine thresholds in `crate::state::thresholds`
//!   4. Adding emoji + message templates under its key in `data/messages.json`

pub mod cpu;

use serde::{Deserialize, Serialize};

/// Stable identifier for a sensor. Used as the key into the message database.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorId {
    Cpu,
    // Ram, Thermal, Battery, Disk, Network, Uptime, Fan, Gpu,
    // Brightness, Webcam, Mic, Idle, Audio, Swap, Load, TimeOfDay, ...
}

impl SensorId {
    pub fn as_str(self) -> &'static str {
        match self {
            SensorId::Cpu => "cpu",
        }
    }
}

/// A single reading from a sensor. The "value" is sensor-specific (percent, count, bytes/sec, ...);
/// the state-machine layer interprets it via per-sensor thresholds.
#[derive(Clone, Debug, Serialize)]
pub struct Reading {
    pub sensor: SensorId,
    pub value: f64,
    /// Optional context the message templates can reference (e.g. top process name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// Read every sensor once. Failures on individual sensors are logged but do not abort the cycle.
pub async fn read_all() -> Vec<Reading> {
    let mut out = Vec::new();
    match cpu::read().await {
        Ok(r) => out.push(r),
        Err(e) => tracing::warn!("cpu sensor: {e:#}"),
    }
    out
}
