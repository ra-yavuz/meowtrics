//! 1-minute load average from /proc/loadavg. Useful for "sustained pressure"
//! states that an instantaneous CPU% can miss.

use anyhow::{Context, Result};
use tokio::fs;

use super::{Reading, SensorId};

pub async fn read() -> Result<Reading> {
    let s = fs::read_to_string("/proc/loadavg").await.context("reading /proc/loadavg")?;
    let one_min: f64 = s
        .split_whitespace()
        .next()
        .context("empty /proc/loadavg")?
        .parse()
        .context("parsing 1-min load")?;
    Ok(Reading {
        sensor: SensorId::Load,
        value: one_min,
        context: None,
    })
}
