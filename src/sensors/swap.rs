//! Swap usage from /proc/meminfo. Returns SwapUsed/SwapTotal as a percentage.
//! If the system has no swap configured, returns 0.

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let s = fs::read_to_string("/proc/meminfo").await.context("reading /proc/meminfo")?;
    let mut total: Option<u64> = None;
    let mut free: Option<u64> = None;
    for line in s.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next();
        let val = parts.next().and_then(|v| v.parse::<u64>().ok());
        match (key, val) {
            (Some("SwapTotal:"), Some(v)) => total = Some(v),
            (Some("SwapFree:"), Some(v)) => free = Some(v),
            _ => {}
        }
    }
    let total = total.unwrap_or(0);
    let free = free.unwrap_or(total);
    let used_pct = if total == 0 {
        0.0
    } else {
        100.0 * ((total - free) as f64 / total as f64)
    };
    Ok(Reading {
        sensor: SensorId::Swap,
        value: used_pct.clamp(0.0, 100.0),
        context: Some(serde_json::json!({ "total_kb": total })),
    })
}
