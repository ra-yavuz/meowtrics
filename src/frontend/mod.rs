//! Tray and IPC frontends.
//!
//! - `sni`: StatusNotifierItem (D-Bus tray icon). Works everywhere a modern tray works.
//! - `dbus`: org.rayavuz.Meowtrics service exposing current state for the KDE plasmoid
//!   (and any other rich frontend) to subscribe to.
//! - `stdout_json`: line-delimited JSON on stdout for waybar/polybar/i3blocks custom modules.
//!
//! All three frontends consume the same snapshot from the daemon, so the tray icon, the
//! plasmoid popup, and the waybar text never disagree.

pub mod sni;
