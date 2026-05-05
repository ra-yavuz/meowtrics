//! Hottest thermal-zone temperature from /sys/class/thermal/thermal_zone*/temp.
//! Returns degrees Celsius. Skips zones without a temp file.

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let mut entries = fs::read_dir("/sys/class/thermal").await.context("reading /sys/class/thermal")?;
    let mut hottest: Option<f64> = None;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let temp_path = entry.path().join("temp");
        if let Ok(s) = fs::read_to_string(&temp_path).await {
            if let Ok(milli_c) = s.trim().parse::<i64>() {
                let c = milli_c as f64 / 1000.0;
                hottest = Some(hottest.map_or(c, |h| h.max(c)));
            }
        }
    }
    let temp = hottest.context("no thermal zones found")?;
    Ok(Reading {
        sensor: SensorId::Thermal,
        value: temp,
        context: None,
    })
}
