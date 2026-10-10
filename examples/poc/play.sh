#!/usr/bin/env bash
# Drives a running fictty screen from outside, the way an agent would: push a UI, send data,
# move the selection, patch one node, read it all back. Start the screen first in another pane:
#   fictty run            (an empty screen)
#   examples/poc/play.sh  (this script)
# Both use $FICTTY_SOCKET if set. Needs fictty on PATH and python3.
set -euo pipefail
cd "$(dirname "$0")"
say() { printf '\n\033[36m▸ %s\033[0m\n' "$1"; sleep "${2:-1.5}"; }

until fictty get --stats >/dev/null 2>&1; do sleep 0.3; done

say "1. Push the whole UI: three components and their data, as one JSON value"
fictty push ui.json | head -3

say "2. Write data only: 20 new latency readings into the store, one every 0.3 s (the UI isn't touched)"
python3 - <<'EOF' | while read -r line; do echo "$line" | fictty put latency - >/dev/null; sleep 0.3; done
import json, math
vals = [180, 176, 190, 188, 201, 230, 260, 245, 210, 195, 188, 182, 179, 185, 192, 220, 310, 420, 390, 300, 240, 210, 198, 190, 186, 183, 181, 184, 180, 182]
for i in range(20):
    vals = vals[1:] + [round(170 + 60 * abs(math.sin(i / 3)) + (i % 4) * 9)]
    print(json.dumps(vals))
EOF

say "3. Move the selection, like a person would: j j"
fictty key j j >/dev/null

say "4. Write new rows for the table: search has recovered (selection and detail stay put)"
fictty put services - >/dev/null <<'EOF'
[
  {"name": "api-gateway", "region": "us-east", "status": "ready", "version": "2.14", "p95": 178, "note": "Rolled out at 09:12."},
  {"name": "billing", "region": "eu-west", "status": "ready", "version": "1.8", "p95": 236, "note": "Nightly invoices run at 02:00."},
  {"name": "search", "region": "us-east", "status": "ready", "version": "3.3", "p95": 205, "note": "Index rebuilt at 10:41. Back to normal."},
  {"name": "notifications", "region": "ap-south", "status": "ready", "version": "0.9", "p95": 131, "note": "Queue depth normal."}
]
EOF

say "5. Patch one node by id: replace the title"
fictty patch title - >/dev/null <<'EOF'
{"type": "text", "id": "title", "size": 1, "style": {"bold": true, "fg": "green"},
 "text": "fictty · all services ready   (j/k move · q quit)"}
EOF

say "6. Read it back: the frame as text" 0.5
fictty screen | head -14

say "7. And the state as JSON: which row is selected, and what that row holds" 0.5
fictty get --state | python3 -c '
import json, sys
st = json.load(sys.stdin)
row = st["local"]["services"]["selected"]
print("focus:", st["focus"], "· selected row:", row)
print("that row:", json.dumps(st["store"]["services"]["value"][row]))'

say "Done. The screen is still live: press j/k in it, or q to quit." 0
