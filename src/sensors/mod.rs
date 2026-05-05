//! Sensor readings.
//!
//! Each sensor module exposes `pub async fn read() -> anyhow::Result<Reading>` and reads
//! from `/proc` or `/sys` only, no root, no extra deps. The state machine in [`crate::state`]
//! consumes a stream of readings and derives a stable `SensorState` for each, which the
//! messages layer and the tray frontend then render.

pub mod cpu;
pub mod ram;
pub mod swap;
pub mod thermal;
pub mod battery;
pub mod disk;
pub mod load;
pub mod uptime;

use serde::{Deserialize, Serialize};

/// Stable identifier for a sensor. Used as the key into the message database.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorId {
    Cpu,
    Ram,
    Swap,
    Thermal,
    Battery,
    Disk,
    Load,
    Uptime,
}

impl SensorId {
    pub fn as_str(self) -> &'static str {
        match self {
            SensorId::Cpu => "cpu",
            SensorId::Ram => "ram",
            SensorId::Swap => "swap",
            SensorId::Thermal => "thermal",
            SensorId::Battery => "battery",
            SensorId::Disk => "disk",
            SensorId::Load => "load",
            SensorId::Uptime => "uptime",
        }
    }

    pub fn all() -> &'static [SensorId] {
        &[
            SensorId::Cpu,
            SensorId::Ram,
            SensorId::Swap,
            SensorId::Thermal,
            SensorId::Battery,
            SensorId::Disk,
            SensorId::Load,
            SensorId::Uptime,
        ]
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

/// Read every sensor once. Sensor failures are logged but don't abort the cycle.
/// A sensor that is unavailable on this hardware (e.g. battery on a desktop) is silently skipped.
pub async fn read_all() -> Vec<Reading> {
    let mut out = Vec::new();
    macro_rules! try_read {
        ($name:expr, $f:expr) => {
            match $f.await {
                Ok(r) => out.push(r),
                Err(e) => tracing::debug!("{} sensor unavailable: {e:#}", $name),
            }
        };
    }
    try_read!("cpu", cpu::read());
    try_read!("ram", ram::read());
    try_read!("swap", swap::read());
    try_read!("thermal", thermal::read());
    try_read!("battery", battery::read());
    try_read!("disk", disk::read());
    try_read!("load", load::read());
    try_read!("uptime", uptime::read());
    out
}
