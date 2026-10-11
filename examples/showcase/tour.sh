#!/usr/bin/env bash
# An agent's tour of a running showcase. Start the showcase in one terminal
# (examples/showcase/play.sh), then run this in another. Everything it does goes over the
# socket, the way any agent would: push screens, write the store (with revisions), move the
# selection, type, change the theme, and read it all back. The Agent chapter lists each step.
set -euo pipefail
cd "$(dirname "$0")"
root="$(cd ../.. && pwd)"
f="$root/target/release/fictty"
pause="${TOUR_PAUSE:-1.6}"
say() { printf '\n\033[36m▸ %s\033[0m\n' "$1"; sleep "${2:-$pause}"; }
rev() { "$f" read "$1" | python3 -c 'import json, sys; print(json.load(sys.stdin)["rev"])'; }
type_text() {
  local s=$1 i c
  for ((i = 0; i < ${#s}; i++)); do
    c=${s:i:1}
    [[ $c == " " ]] && c=space
    "$f" key "$c" >/dev/null
    sleep 0.035
  done
}

until "$f" get --stats >/dev/null 2>&1; do
  echo "Waiting for the showcase (examples/showcase/play.sh)…"
  sleep 1
done

say "Notes: the agent opens the notes screen (a push replaces the UI, never the store)"
"$f" push notes.json >/dev/null

say "It adds a note to the list, based on the revision it read (--if-rev)"
r=$(rev notes)
"$f" read notes | python3 -c '
import json, sys
notes = json.load(sys.stdin)["value"]
notes = [n for n in notes if n.get("id") != "tour"] + [{"id": "tour", "title": "From the agent", "tag": "agent"}]
print(json.dumps(notes))' | "$f" put notes - --if-rev "$r" --by tour >/dev/null
printf '%s' '"# From the agent\n\nThis note was written over the socket, with a revision.\nThe list it sits in was a store write too.\n\nNext, the agent types here key by key, like a person.\n"' |
  "$f" put note/tour - --by tour >/dev/null
n=$("$f" read notes | python3 -c 'import json, sys; print(len(json.load(sys.stdin)["value"]) - 1)')
echo "{\"selected\": $n}" | "$f" set notes - >/dev/null

say "It moves to the editor and types" 0.6
"$f" key tab tab >/dev/null
"$f" key down down down down down down end enter >/dev/null
type_text "Typed live: each key went through caretline, and undo would take it back."

say "Ledger: it opens the books; a helper process (books.py) keeps the summary"
"$f" push ledger.json >/dev/null
sleep 1

say "It books an entry by writing the store, based on the revision it read"
r=$(rev ledger)
"$f" read ledger | python3 -c '
import json, sys
rows = [x for x in json.load(sys.stdin)["value"] if x.get("id") != "tour"]
rows.append({"id": "tour", "date": "2026-10-10", "payee": "Typeface licence", "category": "Software", "amount": "-120.00"})
print(json.dumps(rows))' | "$f" put ledger - --if-rev "$r" --by tour >/dev/null

say "And one through the form, the way a person would" 0.6
"$f" key tab tab >/dev/null
type_text "Corner Bakery"
"$f" key tab >/dev/null
"$f" key c-a >/dev/null
"$f" key s-end >/dev/null
type_text "Eating out"
"$f" key tab >/dev/null
type_text "-8.40"
"$f" key enter >/dev/null

say "Themes: the look is the person's; screens only ask for meanings"
"$f" theme paper >/dev/null
sleep "$pause"
"$f" theme phosphor >/dev/null
sleep "$pause"
"$f" theme glass >/dev/null

say "Markets: a live stream from ticks.py; the agent moves through the watchlist"
"$f" push markets.json >/dev/null
for _ in 1 2 3 4; do
  sleep 0.8
  "$f" key j >/dev/null
done

say "Gallery: every component; it flips through the tabs"
"$f" push gallery.json >/dev/null
sleep "$pause"
echo '{"tab": 1}' | "$f" set tabs - >/dev/null
sleep "$pause"
echo '{"tab": 2}' | "$f" set tabs - >/dev/null

say "Agent: everything above, as the screen saw it, and the store by revision"
"$f" push agent.json >/dev/null
sleep 2

say "Read back: the note it wrote, exactly"
"$f" read note/tour | python3 -c 'import json, sys; e = json.load(sys.stdin); print(e["text"]); print("rev", e["rev"], "· by", e["by"])'

say "Done. The showcase is yours: esc for the menu, t for the theme." 0
