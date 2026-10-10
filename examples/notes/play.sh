#!/usr/bin/env bash
# An agent and a person sharing one note, without either losing the other's work. Start a
# screen in one pane (`fictty run --store notes-store.json` keeps the note across runs), then
# run this in another. The note is a store entry: pushes never touch it, and the agent's writes
# say which revision they're based on, so they're refused if the person typed in between.
# The screen's "Agent activity" panel lists what the agent sends as it arrives.
# Both use $FICTTY_SOCKET if set. Needs fictty on PATH and python3.
set -euo pipefail
cd "$(dirname "$0")"
say() { printf '\n\033[36m▸ %s\033[0m\n' "$1"; sleep "${2:-1.5}"; }
rev() { fictty read standup | python3 -c 'import json, sys; print(json.load(sys.stdin)["rev"])'; }
type_text() {
  local s=$1 i c
  for ((i = 0; i < ${#s}; i++)); do
    c=${s:i:1}
    [[ $c == " " ]] && c=space
    fictty key "$c" >/dev/null
    sleep 0.04
  done
}

until fictty get --stats >/dev/null 2>&1; do sleep 0.3; done

say "1. Push the notes UI. The editor shows the store entry \"standup\"; its text only seeds it"
fictty push ui.json | head -3

say "2. Type two todos into the note, one key at a time, like a person"
fictty key down down down down down down end >/dev/null
type_text "Ship the notes editor"
fictty key enter - space >/dev/null
type_text "Write the guide page"

say "3. Read the entry: its revision, and who wrote it last"
fictty read standup | python3 -c 'import json, sys; e = json.load(sys.stdin); print("rev", e["rev"], "· by", e["by"])'
old=$(rev)

say "4. The person types again (one more todo), so the agent's copy is now out of date"
fictty key enter - space >/dev/null
type_text "Book the retro"

say "5. The agent writes based on what it read: refused, and told what's there now"
echo '"# Standup\n\n(rewritten from an old copy)\n"' | { fictty put standup - --if-rev "$old" || true; } | python3 -c '
import json, sys
r = json.load(sys.stdin)
print("ok:", r["ok"], "·", r["status"])'

say "6. It reads again and adds a section at the end, as an edit: the person's work stays"
len=$(fictty read standup | python3 -c 'import json, sys; print(len(json.load(sys.stdin)["text"]))')
printf '[[%s, %s, "\\nAsk\\n- Who owns the search alerts?\\n"]]' "$len" "$len" | fictty edit standup - --if-rev "$(rev)" | head -4

say "7. Push a completely different UI, with no editor in it"
fictty push focus.json >/dev/null
sleep 2.5

say "8. Push the notes UI back: the note is all still there (the seed text doesn't win)"
fictty push ui.json >/dev/null
fictty patch title - >/dev/null <<'EOF'
{"type": "text", "id": "title", "size": 1, "style": {"bold": true, "fg": "green"}, "text": "fictty · notes · nothing lost"}
EOF

say "9. Read the note back"
fictty read standup | python3 -c 'import json, sys; e = json.load(sys.stdin); print(e["text"]); print("rev", e["rev"], "· by", e["by"])'

say "Done. Click the top pane: Ctrl-Z undoes the agent's section, Tab leaves the editor, then q quits." 0
