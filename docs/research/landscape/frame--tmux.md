# tmux (and control mode)

- **Maker:** Nicholas Marriott and contributors
- **URL:** https://github.com/tmux/tmux, https://github.com/tmux/tmux/wiki/Control-Mode
- **Status (10 October 2026):** 3.8, released 9 September 2026. C, ISC licence, 49,876 stars on
  the GitHub mirror (the project dates from 2007). Mature and still moving: 3.8 is a large release.

## What it is

The default terminal multiplexer: sessions, windows, panes, detach and reattach. **Control mode**
(`tmux -C` / `-CC`) is a text protocol that lets another program be tmux's client, designed by
George Nachman so iTerm2 could show tmux panes as native windows.

## The problem it's solving

Originally: keep terminal sessions alive and split them. For agents, it has become the lowest
common denominator for "run something in a pane, send it keys, read what it printed." Claude
Code's agent teams use tmux (or iTerm2) for split-pane teammates; claude-squad runs each agent in
a tmux session; countless skills tell agents to "use tmux to test."

## Its path / bet

Stability and ubiquity. Everything is a command; everything has a format string. Grow by careful
additions, not by redesign.

## How it works

- **Driving panes.** `send-keys`, `capture-pane -p` (text of the visible pane or history),
  `list-panes -F` with formats, `wait-for`. This is the substrate of nearly every "screen driver."
- **Control mode.** Commands go in as text; each reply is wrapped in `%begin`/`%end` (or
  `%error`). Pane output streams as `%output %<pane> <octal-escaped data>`, with notifications such
  as `%window-add` and `%session-changed`. `refresh-client -B name:type:format` subscribes to a
  format, re-expanded at most once a second, reported with `%subscription-changed`.
- **3.8 (September 2026).** Floating panes throughout; **modal panes** (`new-pane -O`) that block
  the rest of the window, close on Escape (`-D`) or outside click (`-C`) and can take every key
  (`-K`); `new-pane -W` waits for the pane's command to exit; layout strings in a JSON subset;
  hooks rebuilt on events with key/value payloads, and **monitors** (`set-hook -B`) that fire when
  a format changes or becomes true; built-in light and dark themes and the terminal's theme
  reported to panes.

## Strengths

- Everywhere, stable, scriptable, understood by every model.
- 3.8's modal floating pane plus `-W` is a near-perfect primitive for "show the person a screen,
  wait until they're done, carry on."
- Control mode is a proven way to put a native UI around terminal sessions.

## Weaknesses / limits

- No notion of agents or status (monitors on formats get close).
- Read-back is text and escape codes, with no semantics.
- Graphics need passthrough configuration and Unicode placeholders for the kitty protocol.
- Control mode is line-oriented and old-fashioned to parse; no JSON envelope.

## Relation to fictty

**Complement; the floor.** tmux is where many people and agents already are. fictty needs to run
well inside tmux (including pixels via passthrough or a clean fallback), and an agent should be
able to pop a fictty screen up as a tmux 3.8 modal pane and block until it closes.

## Could fictty adopt it instead of building?

No. tmux is a frame with no content model. An agent could build a crude screen as a shell script
in a pane and read it with `capture-pane`, which is exactly the "screen driver" path fictty exists
to improve on.

## What fictty should take from it

- **Recipe: `tmux new-pane -O -W -D 'fictty run review.json'`** (modal, wait, Escape closes) as the
  documented way to ask a person something from inside tmux. Check the flag semantics against 3.8
  before publishing; we read them from CHANGES, not from a test.
- **Format-subscription monitors** are a cheap, proven model for `watch`: subscribe to an
  expression over the state, fire when it changes or becomes true.
- Respect the terminal theme tmux now reports to panes.

## Sources

- https://github.com/tmux/tmux (release 3.8), read 10 October 2026
- https://raw.githubusercontent.com/tmux/tmux/master/CHANGES
- https://github.com/tmux/tmux/wiki/Control-Mode
- https://code.claude.com/docs/en/agent-teams (teammate split panes use tmux or iTerm2)
