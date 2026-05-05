//! System uptime in seconds from /proc/uptime.

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let s = fs::read_to_string("/proc/uptime").await.context("reading /proc/uptime")?;
    let secs: f64 = s
        .split_whitespace()
        .next()
        .context("empty /proc/uptime")?
        .parse()
        .context("parsing uptime")?;
    Ok(Reading {
        sensor: SensorId::Uptime,
        value: secs,
        context: None,
    })
}
