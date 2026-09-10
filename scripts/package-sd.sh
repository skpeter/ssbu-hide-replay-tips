#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MOD="$ROOT/dist/hide-replay-tips"
ZIP="$ROOT/hide-replay-tips-sd.zip"
if [[ ! -f "$MOD/info.toml" ]]; then
  echo "missing $MOD; run python scripts/patch_layouts.py first" >&2
  exit 1
fi
STAGE="$ROOT/dist/sd"
rm -rf "$STAGE"
mkdir -p "$STAGE/ultimate/mods"
cp -a "$MOD" "$STAGE/ultimate/mods/hide-replay-tips"
rm -f "$ZIP"
(cd "$STAGE" && zip -r "$ZIP" ultimate)
echo "wrote $ZIP"
