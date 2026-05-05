#!/usr/bin/env bash
# Build a meowtrics .deb without depending on the full debian build toolchain.
# Produces dist/meowtrics_<version>_<arch>.deb, suitable for direct install
# or upload to the apt repository at ~/github-ra-yavuz/apt/.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(sed -nE '1 s/^[^(]*\(([^)]+)-[^-]+\).*/\1/p' "$ROOT/debian/changelog")
[ -n "$VERSION" ] || { echo "could not parse version from debian/changelog" >&2; exit 1; }
ARCH=$(dpkg --print-architecture 2>/dev/null || echo amd64)

PKG_DIR="$ROOT/dist/meowtrics_${VERSION}_${ARCH}"
DEB_OUT="$ROOT/dist/meowtrics_${VERSION}_${ARCH}.deb"

# Build the release binary
( cd "$ROOT" && cargo build --release --locked )

rm -rf "$PKG_DIR" "$DEB_OUT"
mkdir -p "$PKG_DIR/DEBIAN" \
         "$PKG_DIR/usr/bin" \
         "$PKG_DIR/usr/share/meowtrics" \
         "$PKG_DIR/usr/share/plasma/plasmoids/com.ra-yavuz.meowtrics" \
         "$PKG_DIR/usr/share/doc/meowtrics" \
         "$PKG_DIR/usr/share/icons/hicolor/512x512/apps" \
         "$PKG_DIR/usr/lib/systemd/user"

install -m 0755 "$ROOT/target/release/meowtrics"             "$PKG_DIR/usr/bin/meowtrics"
install -m 0644 "$ROOT/data/messages.json"                   "$PKG_DIR/usr/share/meowtrics/messages.json"
install -m 0644 "$ROOT/systemd/meowtrics.service"            "$PKG_DIR/usr/lib/systemd/user/meowtrics.service"
install -m 0644 "$ROOT/README.md"                            "$PKG_DIR/usr/share/doc/meowtrics/README.md"
install -m 0644 "$ROOT/LICENSE"                              "$PKG_DIR/usr/share/doc/meowtrics/copyright"
install -m 0644 "$ROOT/LICENSING.md"                         "$PKG_DIR/usr/share/doc/meowtrics/LICENSING.md"
install -m 0644 "$ROOT/plasmoid/contents/icons/meowtrics.png" "$PKG_DIR/usr/share/icons/hicolor/512x512/apps/meowtrics.png"

# Bundle the Oneko sprite frames so the daemon can use them for the SNI
# tray pixmap. Public-domain sprites by Tatsuya Kato (1990); see LICENSING.md.
mkdir -p "$PKG_DIR/usr/share/meowtrics/icons/neko"
install -m 0644 "$ROOT/plasmoid/icons/neko/"*.png "$PKG_DIR/usr/share/meowtrics/icons/neko/"
install -m 0755 "$ROOT/debian/postinst"                      "$PKG_DIR/DEBIAN/postinst"
install -m 0755 "$ROOT/debian/postrm"                        "$PKG_DIR/DEBIAN/postrm"
cp -r "$ROOT/plasmoid/." "$PKG_DIR/usr/share/plasma/plasmoids/com.ra-yavuz.meowtrics/"

cat > "$PKG_DIR/DEBIAN/control" <<EOF
Package: meowtrics
Version: ${VERSION}-1
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libc6, libssl3, libdbus-1-3
Recommends: fonts-noto-color-emoji
Maintainer: Ramazan Yavuz <yavuzramazan1994@gmail.com>
Homepage: https://github.com/ra-yavuz/meowtrics
Description: animated emoji tray pet that reacts to your machine's vital signs
 meowtrics shows a small animated emoji in your system tray that morphs
 based on live sensor readings (CPU, RAM, thermals, battery, disk, network,
 and more). Hover reveals a one-line take on what's going on. KDE Plasma 6
 gets a richer popup; other desktops get the tray icon via the
 StatusNotifierItem standard.
 .
 DISCLAIMER: provided AS IS, no warranty. The author is not liable for any
 damage to hardware, data, or system. By installing you accept full
 responsibility. Personal open-source project; no commercial support.
 See /usr/share/doc/meowtrics/README.md for the full disclaimer.
EOF

dpkg-deb --build --root-owner-group "$PKG_DIR" "$DEB_OUT"
echo
echo "Built: $DEB_OUT"
ls -la "$DEB_OUT"
