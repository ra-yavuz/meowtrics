//! Message database: emoji sets and weighted prose templates per sensor state.
//!
//! Resolution order, lowest precedence to highest:
//!   1. Bundled defaults at /usr/share/meowtrics/messages.json (or ./data/messages.json in dev)
//!   2. Cached refresh at $XDG_CACHE_HOME/meowtrics/messages.json (updated daily by `meowtrics refresh`)
//!   3. User overrides at $XDG_CONFIG_HOME/meowtrics/messages.json
//!
//! The database is a small JSON document, comfortably edited by hand. The remote source is
//! https://ra-yavuz.github.io/meowtrics/messages.json, served from this project's docs/ directory.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Database {
    pub version: u32,
    pub updated: String,
    pub sensors: HashMap<String, StateBundle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StateBundle {
    /// Map from sensor state ("idle"/"normal"/"high"/...) to that state's pack.
    #[serde(flatten)]
    pub states: HashMap<String, StatePack>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatePack {
    /// Emoji or kaomoji shown in the tray for this state. Daemon picks one per tick.
    pub emoji: Vec<String>,
    /// Hover/popup messages, with optional weight (higher = more frequent).
    pub messages: Vec<MessageTemplate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageTemplate {
    pub text: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

/// Trigger an out-of-band refresh of the cached messages.json, fetching the latest
/// from the project Pages site if the remote `version` is greater than the cached one.
///
/// Implemented as a stub for v0.1 scaffolding; wire up reqwest fetch and atomic-replace
/// before tagging.
pub async fn refresh_now() -> Result<()> {
    tracing::info!("refresh: not yet implemented in v0.1 scaffold");
    Ok(())
}
