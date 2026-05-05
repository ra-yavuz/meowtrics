use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod daemon;
mod frontend;
mod messages;
mod sensors;
mod state;

#[derive(Parser)]
#[command(
    name = "meowtrics",
    version,
    about = "Animated emoji system tray pet that reacts to your machine's vital signs",
    long_about = "meowtrics shows a small animated emoji in your system tray (StatusNotifierItem) \
                  that morphs based on live sensor readings (CPU, RAM, thermal, battery, disk, network, \
                  and more). On hover it gossips about your machine using a weighted message database.\n\n\
                  DISCLAIMER: provided AS IS, without warranty of any kind. The author is not liable for \
                  any damage to hardware, data, or system. By installing and running this software you \
                  accept full responsibility. See /usr/share/doc/meowtrics/README.md for full text."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the tray daemon in the foreground (default if no subcommand given).
    Daemon,
    /// Print current sensor states and the active emoji to stdout.
    Status {
        /// Compact one-line output suitable for status bars (polybar, i3blocks).
        #[arg(long, short)]
        short: bool,
    },
    /// Print sensor readings as JSON. For waybar custom modules.
    Json,
    /// List all known sensors and their current state.
    Sensors,
    /// Trigger an immediate refresh of the message database from the project Pages site.
    Refresh,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Cmd::Daemon) {
        Cmd::Daemon => daemon::run().await,
        Cmd::Status { short } => daemon::print_status(short).await,
        Cmd::Json => daemon::print_json().await,
        Cmd::Sensors => daemon::print_sensors().await,
        Cmd::Refresh => messages::refresh_now().await,
    }
}
