#!/bin/sh
# Nocturne in a window: draws the covers, the rooms, the theme and the UI into a folder outside
# the repo, opens it, and starts the player beside it. From the repo root:
#   examples/nocturne/open.sh
# ←/→ and ↑/↓ (or a click) choose an album; space plays and pauses; n and p skip.
set -e
B=${FICTTY:-target/release/fictty}
D=$(dirname "$0")
S=${FICTTY_SOCKET:-/tmp/fnoct.sock}
[ -x "$B" ] || { echo "no fictty at $B: build it (cargo build --release) or set FICTTY" >&2; exit 1; }
OUT=$(python3 "$D/nocturne.py")
rm -f "$S"
"$B" run --window --socket "$S" --themes "$OUT/themes.json" "$OUT/nocturne.json" &
APP=$!
i=0; while [ ! -S "$S" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
python3 "$D/player.py" "$S" &
PLAYER=$!
trap 'kill $PLAYER 2>/dev/null' EXIT
wait $APP
