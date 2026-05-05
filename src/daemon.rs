//! Daemon main loop: tick sensors, advance state machines, push to tray frontend.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::frontend::sni::{render_state, spawn as spawn_sni, TrayState};
use crate::messages::Database;
use crate::sensors::read_all;
use crate::state::StateMachine;

const TICK_SECS: u64 = 5;

pub async fn run() -> Result<()> {
    tracing::info!("meowtrics v{} starting", env!("CARGO_PKG_VERSION"));
    println!(
        "meowtrics v{}: provided AS IS, without warranty of any kind. By running this software you accept full responsibility. See README for full disclaimer.",
        env!("CARGO_PKG_VERSION")
    );

    let db = match Database::load() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            tracing::warn!("messages database not loaded: {e:#}; running with placeholder text");
            Arc::new(Database::default())
        }
    };

    let tray_state = Arc::new(RwLock::new(TrayState::default()));
    if let Err(e) = spawn_sni(tray_state.clone()).await {
        tracing::warn!("could not register SNI tray icon: {e:#}; continuing in headless mode");
    }

    let mut sm = StateMachine::new();
    loop {
        let readings = read_all().await;
        sm.ingest(&readings);
        let snap = sm.snapshot();
        for s in &snap {
            tracing::debug!(
                "{}: {} (last={:.1}, in_state_for={:.1}s)",
                s.sensor.as_str(),
                s.state,
                s.last_value,
                s.duration_in_state().as_secs_f64()
            );
        }
        let new_tray = render_state(&snap, &db);
        *tray_state.write().await = new_tray;
        sleep(Duration::from_secs(TICK_SECS)).await;
    }
}

pub async fn print_status(short: bool) -> Result<()> {
    let db = Database::load().ok();
    let readings = read_all().await;
    let mut sm = StateMachine::new();
    sm.ingest(&readings);
    let snap = sm.snapshot();
    if short {
        if let Some(s) = snap.iter().max_by(|a, b| {
            a.last_value
                .partial_cmp(&b.last_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            let emoji = db
                .as_ref()
                .and_then(|d| d.pick_emoji(s.sensor.as_str(), s.state))
                .unwrap_or_else(|| "🐈".to_string());
            println!("{} {} {:.0}", emoji, s.sensor.as_str(), s.last_value);
        } else {
            println!("🐈");
        }
    } else {
        for s in snap {
            let emoji = db
                .as_ref()
                .and_then(|d| d.pick_emoji(s.sensor.as_str(), s.state))
                .unwrap_or_else(|| "  ".to_string());
            println!(
                "{} {:<10} {:<10} {:>7.1}",
                emoji,
                s.sensor.as_str(),
                s.state,
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
    use crate::sensors::SensorId;
    for id in SensorId::all() {
        println!("{}", id.as_str());
    }
    Ok(())
}
