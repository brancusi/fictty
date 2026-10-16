#!/bin/sh
# Nocturne in a window: draws the covers, the rooms, the theme and the UI into a folder outside
# the repo, then opens it. From the repo root:
#   examples/nocturne/open.sh
# ←/→ and ↑/↓ (or a click) choose an album; everything else follows it.
set -e
B=${FICTTY:-target/release/fictty}
D=$(dirname "$0")
S=${FICTTY_SOCKET:-/tmp/fnoct.sock}
[ -x "$B" ] || { echo "no fictty at $B: build it (cargo build --release) or set FICTTY" >&2; exit 1; }
OUT=$(python3 "$D/nocturne.py")
rm -f "$S"
exec "$B" run --window --socket "$S" --themes "$OUT/themes.json" "$OUT/nocturne.json"
