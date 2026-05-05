# meowtrics

**A small animated emoji that lives in your system tray and gossips about your machine. Cat-shaped by default, sweats when the CPU is hot, naps when you're idle, hovers reveal a one-line take on what's going on.**

> ## Disclaimer / no warranty
>
> This software reads from your system's `/proc` and `/sys` interfaces and renders a tray icon. It does not modify hardware state, but it is provided **as is, without warranty of any kind**, express or implied, including but not limited to merchantability, fitness for a particular purpose, and noninfringement.
>
> By installing or running this software you accept that:
>
> - You alone are responsible for any damage to your hardware, data, or system.
> - The author(s) and contributors are **not liable** for any harm, data loss, hardware failure, or other damages, however caused.
> - Sensor interpretations are heuristic and may be wrong on your hardware. Do not rely on this widget for any decision that matters.
> - Messages displayed by this widget are jokes. Do not interpret them as authoritative system advice.
>
> If you do not accept these terms, do not install or run this software.
>
> Full legal license: see [`LICENSE`](LICENSE) (MIT).

## What it is

A daemon that reads your CPU, RAM, thermals, battery, disk, network, GPU, fan speed, brightness, webcam, mic, idle time, and a few more things from standard kernel interfaces, and renders a tiny animated emoji in your system tray that reflects the most interesting state at any given moment. Hover the tray icon and a one-line message tells you what it thinks is going on.

The KDE Plasma 6 frontend gets a richer popup with sensor breakdowns and weighted prose messages. Other desktops (GNOME with AppIndicator, XFCE, Cinnamon, Budgie, MATE, LXQt, Sway+waybar, etc.) get the tray icon with hover tooltip via the StatusNotifierItem standard.

## Status

Pre-alpha (v0.1 in progress). The daemon scaffolding, CPU sensor, state machine, and message database format are in place. Tray rendering, the rest of the sensors, the KDE plasmoid frontend, and the daily message refresh are next.

## Supported environments

| Desktop / WM | How meowtrics shows up | Rich popup? |
|---|---|---|
| KDE Plasma 6 (X11/Wayland) | Plasmoid widget in panel | yes, native QML popup |
| GNOME (with AppIndicator extension) | SNI tray icon | tooltip only |
| XFCE 4.16+, Cinnamon, Budgie, MATE, LXQt | SNI tray icon | tooltip + menu |
| Sway / Hyprland / river (waybar) | waybar custom module | text + tooltip |
| i3 / bspwm (polybar / i3blocks) | bar text module | text + tooltip |

## Sensors (planned)

CPU load (per-core in a later version), RAM, swap, thermal zones, fan speed, battery (level + charging + cycle count), disk usage and IO, network throughput, GPU load and temp, brightness, webcam-in-use, mic-in-use, idle time, audio mute/volume, load average, uptime, time-of-day, plus composite states like "thermal throttling", "ghost load", and "just woke up".

## Messages

A weighted JSON database of emoji sets and message templates per sensor state. Templates use `{value}`, `{duration}`, and other sensor-specific variables. The database can be:

- bundled (ships with the package at `/usr/share/meowtrics/messages.json`)
- refreshed daily from this project's GitHub Pages site (`https://ra-yavuz.github.io/meowtrics/messages.json`)
- overridden by the user at `~/.config/meowtrics/messages.json`

Contributions to the message database are welcome via pull request to `data/messages.json`.

## Install

Pre-alpha, no packages yet. To try the daemon from source:

```
git clone https://github.com/ra-yavuz/meowtrics.git
cd meowtrics
cargo build --release
./target/release/meowtrics status
```

## Configuration

User config (optional): `~/.config/meowtrics/config.toml`

```toml
tick_secs = 5             # how often sensors are polled
blink_secs = 2            # how often the active emoji rotates within a state's set
auto_update_messages = true
messages_url = "https://ra-yavuz.github.io/meowtrics/messages.json"
```

## License

MIT. See [`LICENSE`](LICENSE).

## Contributing

Issues and PRs welcome. The most easily contributable surface area is the message database in [`data/messages.json`](data/messages.json): more emoji, funnier lines, more nuanced states.
