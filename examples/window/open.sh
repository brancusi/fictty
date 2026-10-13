#!/bin/sh
# Open one of the wringer's screens in the native window, in a theme. From the repo root:
#   examples/window/open.sh mail                 # the first theme (night-city)
#   examples/window/open.sh kanban brutal        # another theme from themes.json
#   examples/window/open.sh all                  # every screen, one window each
# Each screen gets its own socket (/tmp/fw-<screen>.sock), so an agent can drive it:
#   fictty --socket /tmp/fw-mail.sock theme paper
#   fictty --socket /tmp/fw-mail.sock screen --nodes
# ops starts feed.py, which streams its numbers and log until the window closes.
set -e
B=${FICTTY:-target/release/fictty}
D=$(dirname "$0")
SCREENS="mail ops settings chat kanban player landing calendar sheet"
[ -n "$1" ] || { echo "usage: $0 <$(echo $SCREENS | tr ' ' '|')|all> [theme]"; exit 2; }
if [ "$1" = all ]; then
  for s in $SCREENS; do "$0" "$s" "$2" & sleep 0.3; done
  wait
  exit
fi
S=/tmp/fw-$1.sock
$B run --window --socket "$S" --themes "$D/themes.json" "$D/$1.json" &
APP=$!
# The socket shows up once the screen is running.
i=0; while [ ! -S "$S" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
[ -n "$2" ] && $B --socket "$S" theme "$2" >/dev/null
if [ "$1" = ops ]; then
  python3 "$D/feed.py" "$B" "$S" & FEED=$!
  trap 'kill $FEED 2>/dev/null' EXIT
fi
wait $APP
