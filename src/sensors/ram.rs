//! RAM usage from /proc/meminfo. Returns percent of MemTotal that is "used"
//! (MemTotal - MemAvailable, which is the kernel's best estimate of memory
//! actually unavailable to applications).

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let s = fs::read_to_string("/proc/meminfo")
        .await
        .context("reading /proc/meminfo")?;
    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;
    for line in s.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next();
        let val = parts.next().and_then(|v| v.parse::<u64>().ok());
        match (key, val) {
            (Some("MemTotal:"), Some(v)) => total = Some(v),
            (Some("MemAvailable:"), Some(v)) => available = Some(v),
            _ => {}
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }
    let total = total.context("MemTotal not found in /proc/meminfo")?;
    let available = available.context("MemAvailable not found in /proc/meminfo")?;
    let used_pct = if total == 0 {
        0.0
    } else {
        100.0 * (1.0 - (available as f64 / total as f64))
    };
    Ok(Reading {
        sensor: SensorId::Ram,
        value: used_pct.clamp(0.0, 100.0),
        context: Some(serde_json::json!({ "total_kb": total, "available_kb": available })),
    })
}
