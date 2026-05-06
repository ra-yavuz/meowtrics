use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod daemon;
mod frontend;
mod messages;
mod sensors;
mod sprites;
mod state;

#[derive(Parser)]
#[command(
    name = "meowtrics",
    version,
    about = "Animated cat in your KDE Plasma panel that reacts to your machine's vital signs",
    long_about = "meowtrics is a daemon that reads your system's vital signs (CPU, RAM, thermal, \
                  battery, disk, network, and more) and an animated Oneko cat panel widget that \
                  reacts to them. The daemon emits classified state as JSON via `meowtrics tray-state`; \
                  the KDE plasmoid widget consumes it and renders the cat.\n\n\
                  By default the daemon runs headless (no system tray icon). Pass --tray to ALSO \
                  register a StatusNotifierItem so the cat shows up in the system tray on desktops \
                  that don't have a plasmoid surface (GNOME, XFCE, etc).\n\n\
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
    /// Run the daemon in the foreground (default if no subcommand given).
    /// Headless by default; pass --tray to also register a system tray icon.
    Daemon {
        /// Also register a StatusNotifierItem (system tray) icon.
        /// Off by default: the KDE plasmoid panel widget is the recommended UI.
        /// Use --tray on desktops without a plasmoid surface (GNOME with
        /// AppIndicator, XFCE, Cinnamon, Sway+waybar, etc).
        #[arg(long)]
        tray: bool,
    },
    /// Print current sensor states and the active emoji to stdout.
    Status {
        /// Compact one-line output suitable for status bars (polybar, i3blocks).
        #[arg(long, short)]
        short: bool,
    },
    /// Print sensor readings as JSON. For waybar custom modules.
    Json,
    /// Print pre-classified tray state as JSON: `{animation, headline, sensors[]}`.
    /// Consumed by the KDE plasmoid; plasmoid is then a dumb renderer.
    TrayState,
    /// List all known sensors and their current state.
    Sensors,
    /// Trigger an immediate refresh of the message database from the project Pages site.
    Refresh,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // CRITICAL: log to stderr, not stdout. The CLI's stdout is consumed
    // by the plasmoid (`meowtrics tray-state`) and by status bars
    // (`meowtrics json`); polluting it with INFO log lines breaks JSON
    // parsing on the consumer side.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Cmd::Daemon { tray: false }) {
        Cmd::Daemon { tray } => daemon::run(tray).await,
        Cmd::Status { short } => daemon::print_status(short).await,
        Cmd::Json => daemon::print_json().await,
        Cmd::TrayState => daemon::print_tray_state().await,
        Cmd::Sensors => daemon::print_sensors().await,
        Cmd::Refresh => messages::refresh_now().await,
    }
}
