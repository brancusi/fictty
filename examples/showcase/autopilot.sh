#!/bin/sh
# The tour's autopilot: an agent driving the screen it runs in, over the socket, with the same
# commands anyone can type in another terminal. The Agents chapter starts it with a button
# (`"run": ["sh", "autopilot.sh"]`); the screen tells it where it is ($FICTTY, $FICTTY_SOCKET).
set -eu
f="${FICTTY:-fictty}"
q() { "$f" "$@" >/dev/null; }
say() { q toast "$1" --tone "${2:-accent}"; }
tasks() {
  printf '[{"id":"read","text":"Read the screen","done":%s},{"id":"write","text":"Write a note","done":%s},{"id":"point","text":"Point at something","done":%s},{"id":"theme","text":"Try a theme","done":%s}]' "$@" |
    "$f" put agent_tasks - --by autopilot >/dev/null
}

say "Hi. I'm the autopilot, another writer on this screen."
sleep 1.4

# 1. Read: the frame, as text, the way an agent checks its work.
"$f" screen >/dev/null
q set agent_tasks - <<'EOF'
{"selected": 0}
EOF
tasks true false false false
say "I read the screen: every cell, as text."
sleep 1.2

# 2. Write: the note, a few words at a time, each one an edit you could undo.
q set agent_tasks - <<'EOF'
{"selected": 1}
EOF
note=""
for w in "#" "From" "the" "autopilot\\n\\n" "I" "typed" "this" "over" "the" "socket," "a" "few" "words" "at" "a" "time.\\n\\n" "-" "[x]" "read" "the" "screen\\n" "-" "[x]" "write" "this" "note\\n" "-" "[" "]" "your" "turn"; do
  case $w in "[") note="$note[ " ;; *) note="$note$w " ;; esac
  printf '"%s"' "$note" | "$f" put agentpad - --by autopilot >/dev/null
  sleep 0.12
done
tasks true true false false
sleep 0.6

# 3. Point: a callout on the list, the way an agent shows a person where to look.
q set agent_tasks - <<'EOF'
{"selected": 2}
EOF
q layers - <<'EOF'
[{"on": "agent_tasks", "row": 2, "title": "Look here", "text": "Agents point with callouts and rings.", "arrow": true, "ring": true, "place": ["left", "below"], "width": 30, "by": "autopilot"}]
EOF
tasks true true true false
sleep 2.4
q layers - <<'EOF'
[]
EOF

# 4. Theme: one key restyles everything; an agent never picks a colour.
q set agent_tasks - <<'EOF'
{"selected": 3}
EOF
q theme next
sleep 1.1
q theme next
sleep 1.1
q theme next
tasks true true true true
sleep 0.4

echo true | "$f" put agent_done - --by autopilot >/dev/null
say "Done. Everything I did is in \$log. Your turn." pos
