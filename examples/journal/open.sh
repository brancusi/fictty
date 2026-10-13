#!/bin/sh
# Daybook in the native window. From the repo root:
#   examples/journal/open.sh          # in paper
#   examples/journal/open.sh glass    # another theme from examples/window/themes.json
#   DAYBOOK_STORE=~/daybook.json examples/journal/open.sh   # keep what you write between runs
# Drive it meanwhile from another terminal:
#   fictty --socket /tmp/fjournal.sock screen --nodes
set -e
B=${FICTTY:-target/release/fictty}
D=$(dirname "$0")
S=${FICTTY_SOCKET:-/tmp/fjournal.sock}
rm -f "$S"
$B run --window --socket "$S" --themes "$D/../window/themes.json" ${DAYBOOK_STORE:+--store "$DAYBOOK_STORE"} "$D/journal.json" &
APP=$!
i=0; while [ ! -S "$S" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
$B --socket "$S" theme "${1:-paper}" >/dev/null
wait $APP
