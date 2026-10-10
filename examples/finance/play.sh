#!/bin/sh
# Show the dashboard on the running screen and run the analyst's tour. From the repo root:
#   cargo run --release -- run        # one terminal
#   examples/finance/play.sh          # another
set -e
B=${FICTTY:-target/release/fictty}
$B push examples/finance/dashboard.json >/dev/null
sleep 2
python3 examples/finance/tour.py "${STEP:-6}"
