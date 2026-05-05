//! Daemon main loop: tick sensors, advance state machines, render frontends.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

use crate::sensors::read_all;
use crate::state::StateMachine;

pub async fn run() -> Result<()> {
    tracing::info!("meowtrics v{} starting", env!("CARGO_PKG_VERSION"));
    println!("meowtrics: provided AS IS, without warranty of any kind. By running this software you accept full responsibility. See README.md for full disclaimer.");

    let mut sm = StateMachine::new();
    loop {
        let readings = read_all().await;
        sm.ingest(&readings);
        for s in sm.snapshot() {
            tracing::debug!(
                "{}: {} (last={:.1}, in_state_for={:.1}s)",
                s.sensor.as_str(),
                s.state.as_str(),
                s.last_value,
                s.duration_in_state().as_secs_f64()
            );
        }
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn print_status(short: bool) -> Result<()> {
    let readings = read_all().await;
    let mut sm = StateMachine::new();
    sm.ingest(&readings);
    let snap = sm.snapshot();
    if short {
        // Compact: just the most-active sensor and its value.
        if let Some(s) = snap.iter().max_by(|a, b| a.last_value.partial_cmp(&b.last_value).unwrap_or(std::cmp::Ordering::Equal)) {
            println!("🐈 {} {:.0}%", s.sensor.as_str(), s.last_value);
        } else {
            println!("🐈");
        }
    } else {
        for s in snap {
            println!(
                "{:<12} {:<10} value={:.1}",
                s.sensor.as_str(),
                s.state.as_str(),
                s.last_value
            );
        }
    }
    Ok(())
}

pub async fn print_json() -> Result<()> {
    let readings = read_all().await;
    println!("{}", serde_json::to_string(&readings)?);
    Ok(())
}

pub async fn print_sensors() -> Result<()> {
    println!("cpu       (active in v0.1)");
    println!("ram       (planned)");
    println!("thermal   (planned)");
    println!("battery   (planned)");
    println!("disk      (planned)");
    println!("network   (planned)");
    println!("uptime    (planned)");
    Ok(())
}
