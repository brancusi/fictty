#!/bin/sh
# The demo: a window, and an agent that builds Kestrel Cycles in it, one command at a time.
# From the repo root:
#   examples/demo/open.sh            # the first theme
#   examples/demo/open.sh paper      # another theme from examples/window/themes.json
#   examples/demo/open.sh - --fast   # six times faster (for checking it)
# Drive it yourself meanwhile from another terminal:
#   fictty --socket /tmp/fdemo.sock screen --nodes --style
set -e
B=${FICTTY:-target/release/fictty}
D=$(dirname "$0")
S=${FICTTY_SOCKET:-/tmp/fdemo.sock}
rm -f "$S"
$B run --window --socket "$S" --themes "$D/../window/themes.json" "$D/start.json" &
APP=$!
# The socket shows up once the screen is running.
i=0; while [ ! -S "$S" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
[ -n "$1" ] && [ "$1" != - ] && $B --socket "$S" theme "$1" >/dev/null
shift 2>/dev/null || true
python3 "$D/agent.py" "$B" "$S" "$@" & AGENT=$!
trap 'kill $AGENT 2>/dev/null' EXIT
wait $APP
