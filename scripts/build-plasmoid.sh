#!/usr/bin/env bash
# Bundle the KDE plasmoid into a .plasmoid (zip) installable via:
#   kpackagetool6 -t Plasma/Applet -i dist/meowtrics.plasmoid
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$ROOT/dist"
OUT="$ROOT/dist/meowtrics.plasmoid"
rm -f "$OUT"
( cd "$ROOT/plasmoid" && zip -r "$OUT" . -x '*.DS_Store' )
echo "Built: $OUT"
ls -la "$OUT"
