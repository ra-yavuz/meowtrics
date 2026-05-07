# meowtrics

**A small animated emoji that lives in your system tray and gossips about your machine. Cat-shaped by default, sweats when the CPU is hot, naps when you're idle. Hover reveals a one-line take on what's going on.**

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
> Personal open-source project, separate from the author's professional work. Provided free for personal use, with no support guarantee or commercial relationship implied. Full legal license: see [`LICENSE`](LICENSE) (MIT).

## Quick install (Debian / Ubuntu)

One command. Detects your distro, prints the disclaimer, asks for consent, then sets up the apt repository and installs the package.

```bash
curl -fsSL https://ra-yavuz.github.io/meowtrics/get.sh | sudo bash
```

After install:

```bash
systemctl --user enable --now meowtrics
```

For unattended installs, set `MEOWTRICS_YES=1` to skip the consent prompt. Future upgrades: `sudo apt upgrade`. Removal: `sudo apt purge meowtrics`. Other install paths (single `.deb`, source build, manual apt setup) appear further down.

## What it is

A tiny daemon reads your system's vital signs from the kernel (`/proc`, `/sys`, no root needed), runs each through a debounced state machine, and renders a small animated emoji in your system tray that reflects the most interesting state at any given moment. Hover and a one-line message tells you what it thinks is going on.

The KDE Plasma 6 frontend gets a richer popup with sensor breakdowns and weighted prose messages. Other desktops (GNOME with AppIndicator, XFCE, Cinnamon, Budgie, MATE, LXQt, Sway+waybar, etc.) get the tray icon with hover tooltip via the StatusNotifierItem standard. Tiling-WM users can consume the daemon's JSON output as a waybar/polybar/i3blocks custom module.

## Supported environments

Tested on **Ubuntu (Linux only)**. **macOS is not supported**: meowtrics relies on Linux `/proc` and `/sys` interfaces for sensors, and on D-Bus StatusNotifierItem (or KDE Plasma) for the tray icon, none of which exist on Darwin. **WSL2** is partially viable for the JSON-output mode (`meowtrics json` for waybar / polybar consumers running inside the WSL distro), but the tray icon path needs a Linux desktop session, which WSL2 does not provide by default.

| Desktop / WM | How meowtrics shows up | Rich popup? |
|---|---|---|
| KDE Plasma 6 (X11/Wayland) | Plasmoid widget in panel | yes, native QML popup |
| GNOME (with AppIndicator extension) | SNI tray icon | tooltip only |
| XFCE 4.16+, Cinnamon, Budgie, MATE, LXQt | SNI tray icon | tooltip + menu |
| Sway / Hyprland / river (waybar) | waybar custom module | text + tooltip |
| i3 / bspwm (polybar / i3blocks) | bar text module | text + tooltip |

## Sensors (v0.1)

`cpu`, `ram`, `swap`, `thermal`, `battery`, `disk`, `load average`, `uptime`. Each has 2-5 states (`idle` / `normal` / `busy` / `high` / `critical`, plus sensor-specific extras like `charging` / `low` / `full` for battery, `cool` / `warm` / `hot` for thermal).

Planned for v0.2: per-core CPU, fan speed, GPU load and temp, brightness, network throughput, webcam / mic in use, idle time, audio mute / volume, time-of-day, and composite states like "thermal throttling", "ghost load", "just woke up".

## Messages

A weighted JSON database of emoji sets and message templates per sensor state. Templates use `{value}`, `{duration}`, and other sensor-specific variables. Three layers, lowest-precedence first:

- bundled with the package at `/usr/share/meowtrics/messages.json`
- daily auto-refresh from the project Pages site (`https://ra-yavuz.github.io/meowtrics/messages.json`)
- user override at `~/.config/meowtrics/messages.json`

Contributions to the database are welcome via pull request to [`data/messages.json`](data/messages.json). v0.1 ships 18 sensor categories, 67 states, 220 emoji entries, and 105 message templates.

## CLI

```bash
meowtrics                # run the tray daemon (default)
meowtrics status         # current sensor states + active emoji
meowtrics status --short # one-line output for status bars
meowtrics json           # JSON for waybar/polybar/i3blocks
meowtrics sensors        # list known sensors
meowtrics refresh        # fetch the latest messages.json
meowtrics --help
```

## Install via apt repository (recommended for Debian / Ubuntu)

One line. Sets up the signed apt repo if not already added, refreshes the package index, and installs meowtrics. Idempotent, safe to re-run:

```bash
sudo bash -c 'set -e; install -m 0755 -d /etc/apt/keyrings && curl -fsSL https://ra-yavuz.github.io/apt/pubkey.gpg -o /etc/apt/keyrings/ra-yavuz.gpg && echo "deb [signed-by=/etc/apt/keyrings/ra-yavuz.gpg] https://ra-yavuz.github.io/apt stable main" > /etc/apt/sources.list.d/ra-yavuz.list && apt update && apt install -y meowtrics'
systemctl --user enable --now meowtrics
```

If you already added the `ra-yavuz` apt repo earlier, all you need is:

```bash
sudo apt update && sudo apt install meowtrics
```

The `sudo apt update` step is required: without it apt will not see new packages or new versions.

<details><summary>Step by step (manual repo setup)</summary>

```bash
# 1. Trust the signing key
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://ra-yavuz.github.io/apt/pubkey.gpg \
  | sudo tee /etc/apt/keyrings/ra-yavuz.gpg > /dev/null

# 2. Add the apt source
echo "deb [signed-by=/etc/apt/keyrings/ra-yavuz.gpg] https://ra-yavuz.github.io/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/ra-yavuz.list

# 3. Refresh the package index, then install
sudo apt update
sudo apt install meowtrics
systemctl --user enable --now meowtrics
```

</details>

## Install single `.deb` from GitHub Releases

No automatic updates with this path.

```bash
wget https://github.com/ra-yavuz/meowtrics/releases/latest/download/meowtrics_0.1.0_amd64.deb
sudo apt install ./meowtrics_0.1.0_amd64.deb
systemctl --user enable --now meowtrics
```

## Install from source

Any distro with Rust 1.75+, systemd, and a tray host:

```bash
git clone https://github.com/ra-yavuz/meowtrics.git
cd meowtrics
make
sudo make install
systemctl --user enable --now meowtrics
```

## KDE Plasma 6 widget

The package ships the plasmoid alongside the daemon. After install, right-click your panel, _Add Widgets_, search for _meowtrics_. The widget shows the active emoji at panel size and opens a popup on click with a per-sensor breakdown.

## tiling-WM bars (waybar / polybar / i3blocks)

Example snippets in [`frontends/examples/`](frontends/examples). For waybar:

```jsonc
"custom/meowtrics": {
  "exec": "meowtrics json",
  "interval": 5,
  "return-type": "json",
  "tooltip": true
}
```

## Configuration

Optional. User config goes at `~/.config/meowtrics/config.toml`. All keys optional, sensible defaults baked in. (v0.1 reads no config keys yet; v0.2 wires this up.)

## Licensing of bundled emoji packs

The default install ships **no bundled glyph assets**: the tray icon is rendered from Unicode codepoints using your system's emoji font. Optional companion packs (Volpeon's Blobcat, OpenMoji, Fluentui Emoji, Twemoji) ship as separate packages, each with their own license file and attribution. Full policy in [`LICENSING.md`](LICENSING.md).

## License

MIT. See [`LICENSE`](LICENSE).

## Contributing

Issues and PRs welcome. The most easily contributable surface is the message database in [`data/messages.json`](data/messages.json): more emoji, funnier lines, more nuanced states.
