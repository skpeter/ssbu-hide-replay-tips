#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NRO="$ROOT/target/aarch64-skyline-switch/release/libhide_replay_tips.nro"
ZIP="$ROOT/hide-replay-tips-sd.zip"
if [[ ! -f "$NRO" ]]; then
  echo "missing $NRO; cargo skyline build --release first" >&2
  exit 1
fi
STAGE="$ROOT/dist/sd"
rm -rf "$STAGE"
DEST="$STAGE/atmosphere/contents/01006A800016E000/romfs/skyline/plugins"
mkdir -p "$DEST"
cp "$NRO" "$DEST/libhide_replay_tips.nro"
rm -f "$ZIP"
(cd "$STAGE" && zip -r "$ZIP" atmosphere)
echo "wrote $ZIP"
