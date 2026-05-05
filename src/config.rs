//! User configuration loaded from $XDG_CONFIG_HOME/meowtrics/config.toml
//!
//! Everything optional; sensible defaults baked in. The config file is a hand-editable
//! TOML, kept small and flat. Anything that needs structured data (the message database)
//! lives in messages.json instead.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// How often the daemon reads sensors and considers swapping the tray emoji. In seconds.
    #[serde(default = "default_tick_secs")]
    pub tick_secs: u64,

    /// How often the active emoji rotates within a state's emoji set, in seconds. Set to 0 to disable.
    #[serde(default = "default_blink_secs")]
    pub blink_secs: u64,

    /// Whether to fetch updated messages.json from the project Pages site daily.
    #[serde(default = "default_true")]
    pub auto_update_messages: bool,

    /// Override the messages.json source URL. Leave default unless you're hosting your own pack.
    #[serde(default = "default_messages_url")]
    pub messages_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_secs: default_tick_secs(),
            blink_secs: default_blink_secs(),
            auto_update_messages: true,
            messages_url: default_messages_url(),
        }
    }
}

fn default_tick_secs() -> u64 {
    5
}
fn default_blink_secs() -> u64 {
    2
}
fn default_true() -> bool {
    true
}
fn default_messages_url() -> String {
    "https://ra-yavuz.github.io/meowtrics/messages.json".to_string()
}
