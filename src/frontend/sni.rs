//! StatusNotifierItem tray frontend.
//!
//! For v0.1 scaffold this is a stub. The real implementation in the next pass will:
//!   - Build a `ksni::Tray` whose `icon_pixmap` is rendered from the active emoji at panel size
//!   - Update the icon on each daemon tick
//!   - Set tooltip to a one-line summary (active state + top message)
//!   - Wire context menu items: Pause animation, Refresh messages, Open settings
//!
//! Rendering an emoji to a pixmap from Rust without a Qt/Cairo dependency is the only
//! non-obvious bit: we'll use `cosmic-text` or `tiny-skia` + `fontdue` with the system
//! emoji font, both pure-Rust.
