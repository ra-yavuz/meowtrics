#!/usr/bin/env bash
# meowtrics installer: detects your distro and installs the best available package.
#
# Usage:
#   curl -fsSL https://ra-yavuz.github.io/meowtrics/get.sh | sudo bash
#
# Environment:
#   MEOWTRICS_YES=1   skip interactive disclaimer prompt (for unattended install)
#
# DISCLAIMER: meowtrics is provided AS IS, WITHOUT WARRANTY OF ANY KIND.
# The author is not liable for any damage to hardware, data, or system.
# By running this script you accept full responsibility.
# This is a personal open-source project; no commercial support implied.
# Full text: https://github.com/ra-yavuz/meowtrics/blob/main/README.md

set -euo pipefail

REPO_OWNER="ra-yavuz"
REPO_NAME="meowtrics"
APT_REPO_URL="https://ra-yavuz.github.io/apt"
APT_KEY_URL="${APT_REPO_URL}/pubkey.gpg"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

require_root() {
    if [ "${EUID:-$(id -u)}" -ne 0 ]; then
        red "This script needs root to install packages and trust the apt key."
        echo "Re-run as: curl -fsSL https://ra-yavuz.github.io/meowtrics/get.sh | sudo bash"
        exit 1
    fi
}

show_disclaimer() {
    cat <<'EOF'

================================================================
 meowtrics installer
================================================================

 DISCLAIMER

 This software is provided AS IS, WITHOUT WARRANTY OF ANY KIND,
 express or implied. The author and contributors are NOT LIABLE
 for any damage to your hardware, data, or system, however caused.

 By installing meowtrics you accept full responsibility. Sensor
 interpretations are heuristic and may be wrong on your hardware.
 Messages displayed by the widget are jokes; do not interpret them
 as authoritative system advice.

 This is a personal open-source project, separate from the author's
 professional work. Provided free for personal use, with no support
 guarantee or commercial relationship implied.

 Full disclaimer:
   https://github.com/ra-yavuz/meowtrics/blob/main/README.md

================================================================

EOF
}

prompt_consent() {
    if [ "${MEOWTRICS_YES:-}" = "1" ]; then
        green "MEOWTRICS_YES=1 set, skipping interactive prompt."
        return
    fi
    if [ ! -t 0 ]; then
        # Stdin is the curl pipe. Re-open /dev/tty for the prompt.
        if [ -e /dev/tty ]; then exec </dev/tty; fi
    fi
    printf "Do you accept these terms and want to continue? [y/N] "
    read -r reply || reply=""
    case "$reply" in
        [yY]|[yY][eE][sS]) ;;
        *) red "Cancelled."; exit 1 ;;
    esac
}

detect_distro() {
    if [ -r /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        echo "${ID:-unknown}:${ID_LIKE:-}"
    else
        echo "unknown:"
    fi
}

install_via_apt() {
    bold "Installing via apt repository (recommended; you'll get auto-updates with apt upgrade)"
    apt-get update
    apt-get install -y --no-install-recommends ca-certificates curl gnupg

    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL "$APT_KEY_URL" -o /etc/apt/keyrings/ra-yavuz.gpg
    chmod 0644 /etc/apt/keyrings/ra-yavuz.gpg

    cat > /etc/apt/sources.list.d/ra-yavuz.list <<EOF
deb [signed-by=/etc/apt/keyrings/ra-yavuz.gpg] ${APT_REPO_URL} stable main
EOF

    apt-get update
    apt-get install -y meowtrics
}

install_single_deb() {
    bold "Installing single .deb from GitHub Releases (no auto-updates)"
    apt-get update
    apt-get install -y --no-install-recommends ca-certificates curl
    arch=$(dpkg --print-architecture)
    deb_path=/tmp/meowtrics_latest_${arch}.deb
    # Find the latest release deb matching arch
    api="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    url=$(curl -fsSL "$api" | grep -oE "https://[^\"']*meowtrics_[0-9.]+-[0-9]+_${arch}\.deb" | head -1)
    if [ -z "$url" ]; then
        red "Could not find a .deb for architecture ${arch} in the latest release."
        echo "Visit https://github.com/${REPO_OWNER}/${REPO_NAME}/releases for manual download."
        exit 1
    fi
    curl -fsSL "$url" -o "$deb_path"
    apt-get install -y "$deb_path"
    rm -f "$deb_path"
}

install_from_source() {
    bold "Building from source (your distro is not in the apt repo)"
    if ! command -v cargo >/dev/null 2>&1; then
        red "cargo not found. Please install Rust 1.75+ from https://rustup.rs and re-run."
        exit 1
    fi
    if ! command -v git >/dev/null 2>&1; then
        red "git not found."
        exit 1
    fi
    workdir=$(mktemp -d)
    git clone --depth 1 "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" "$workdir/${REPO_NAME}"
    cd "$workdir/${REPO_NAME}"
    make
    make install
    cd /
    rm -rf "$workdir"
}

enable_user_service() {
    bold "Enabling user service (will start automatically on next graphical login)"
    user="${SUDO_USER:-}"
    if [ -z "$user" ]; then
        echo "Could not detect non-root user; skip enabling. Run manually:"
        echo "  systemctl --user enable --now meowtrics"
        return
    fi
    if command -v systemctl >/dev/null 2>&1; then
        sudo -u "$user" XDG_RUNTIME_DIR="/run/user/$(id -u "$user")" \
            systemctl --user daemon-reload 2>/dev/null || true
        sudo -u "$user" XDG_RUNTIME_DIR="/run/user/$(id -u "$user")" \
            systemctl --user enable --now meowtrics 2>/dev/null \
            || echo "(could not auto-start; run 'systemctl --user enable --now meowtrics' yourself)"
    fi
}

print_next_steps() {
    cat <<EOF

$(green "meowtrics installed successfully.")

Tray icon:
  The daemon will start at your next graphical login. To start it now:
    systemctl --user enable --now meowtrics

KDE Plasma 6 widget:
  Right-click your panel, Add Widgets, search for "meowtrics".

CLI:
  meowtrics status     # current sensor states + active emoji
  meowtrics json       # JSON output for waybar/polybar/i3blocks
  meowtrics --help     # all commands

Configuration:
  ~/.config/meowtrics/config.toml      (optional; see README for keys)
  ~/.config/meowtrics/messages.json    (optional override of message database)

Docs:
  https://ra-yavuz.github.io/meowtrics/

Uninstall:
  sudo apt remove meowtrics    (if installed via apt)
  or run: sudo $REPO_OWNER-${REPO_NAME}-uninstall

EOF
}

main() {
    require_root
    show_disclaimer
    prompt_consent

    distro=$(detect_distro)
    bold "Detected: $distro"

    case "$distro" in
        debian:*|ubuntu:*|*:*debian*|*:*ubuntu*|linuxmint:*|pop:*|*:*linuxmint*)
            install_via_apt
            ;;
        *)
            yellow_warn() { printf '\033[33m%s\033[0m\n' "$*"; }
            yellow_warn "Your distro isn't in the apt repository yet."
            yellow_warn "Falling back to source build (requires Rust 1.75+ and git)."
            install_from_source
            ;;
    esac

    enable_user_service
    print_next_steps
}

main "$@"
