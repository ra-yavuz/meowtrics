//! Battery percentage from /sys/class/power_supply/BAT*/capacity.
//! Returns the first BAT* device's charge level. On systems without a battery
//! (desktops) this errors and the sensor is silently skipped.

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let mut entries = fs::read_dir("/sys/class/power_supply").await.context("reading /sys/class/power_supply")?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("BAT") {
            continue;
        }
        let cap_path = entry.path().join("capacity");
        let status_path = entry.path().join("status");
        let cap_s = fs::read_to_string(&cap_path).await.ok();
        let status = fs::read_to_string(&status_path).await.ok().map(|s| s.trim().to_string());
        if let Some(cap) = cap_s.and_then(|s| s.trim().parse::<f64>().ok()) {
            return Ok(Reading {
                sensor: SensorId::Battery,
                value: cap.clamp(0.0, 100.0),
                context: Some(serde_json::json!({ "status": status, "device": name.to_string() })),
            });
        }
    }
    anyhow::bail!("no battery found")
}
