//! CPU load sensor.
//!
//! Reads /proc/stat twice with a short delay and returns the busy fraction as a percent.
//! Caller is expected to invoke this on the daemon's tick interval, so we keep the
//! between-samples delay short (200 ms) to keep the daemon's wakeup cheap.

use anyhow::{Context, Result};
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

use super::{Reading, SensorId};

#[derive(Clone, Copy, Debug, Default)]
struct CpuTimes {
    total: u64,
    idle: u64,
}

async fn read_cpu_times() -> Result<CpuTimes> {
    let stat = fs::read_to_string("/proc/stat")
        .await
        .context("reading /proc/stat")?;
    let line = stat.lines().next().context("empty /proc/stat")?;
    // Format: "cpu  user nice system idle iowait irq softirq steal guest guest_nice"
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    if fields.len() < 4 {
        anyhow::bail!("unexpected /proc/stat line: {line}");
    }
    let total: u64 = fields.iter().sum();
    // idle = idle + iowait. iowait is "idle waiting on disk", count it as idle for this widget.
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0);
    Ok(CpuTimes { total, idle })
}

pub async fn read() -> Result<Reading> {
    let t1 = read_cpu_times().await?;
    sleep(Duration::from_millis(200)).await;
    let t2 = read_cpu_times().await?;

    let total_delta = t2.total.saturating_sub(t1.total);
    let idle_delta = t2.idle.saturating_sub(t1.idle);
    let busy_pct = if total_delta == 0 {
        0.0
    } else {
        100.0 * (1.0 - (idle_delta as f64 / total_delta as f64))
    };

    Ok(Reading {
        sensor: SensorId::Cpu,
        value: busy_pct.clamp(0.0, 100.0),
        context: None,
    })
}
