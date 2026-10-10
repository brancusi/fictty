# TUIOS

- **Maker:** Gaurav Gosain (individual, open source)
- **URL:** https://github.com/Gaurav-Gosain/tuios, https://tuios.dev
- **Status (10 October 2026):** v0.9.2, released 9 October 2026. Go, MIT, 5,092 stars. Repository
  created 6 September 2025; v0.8.0 (27 September 2026) turned it toward agents; v0.9.0 landed
  8 October 2026. Pushed to daily. One main author; a bus-factor-of-one project moving very fast.

## What it is

A terminal window manager that runs inside your existing terminal: tiling and floating panes,
nine workspaces, a vim-style modal interface, a daemon with detach and reattach, sessions on
other machines over SSH, and (since v0.8) a layer that knows which pane holds a coding agent and
what that agent is doing. Its tagline is "a terminal window manager that knows what your agents
are doing."

## The problem it's solving

Running five or fifteen agents at once in terminals means losing track of which one is stuck on a
permission prompt, which finished, and which errored. TUIOS's answer is one **Inbox** across every
session and machine, answerable without switching panes, plus the plumbing to start, address and
coordinate fleets of agents.

## Its path / bet

Stay a terminal program (Bubble Tea v2, Lipgloss v2), own every PTY, and add agent awareness on top
of a full-featured multiplexer. Be the integration point by being generous with protocols: a JSON
control protocol, an MCP server, a tmux shim, a herdr compatibility layer, and OSC 7501 for any
program to report status. Win on breadth and polish rather than on one idea.

## How it works

- **Window manager.** BSP, master-stack and niri-style scrolling layouts; copy mode over 10,000
  lines; kitty graphics passthrough (including video), sixel, kitty keyboard protocol, mode 2026.
  An optional build uses libghostty-vt as the emulator.
- **Daemon and protocol.** The daemon speaks line-delimited JSON verbs on the same unix socket the
  client uses (`{"id":1,"verb":"list-windows","params":{...}}`). Errors carry a structured `hint`
  (`did_you_mean`, `available`, the exact CLI `command` to run): an AXI-style design for agent
  callers. `subscribe` streams events; `wait-for` blocks on agent state.
- **Agent state.** `tuios integration install` wires 19 harnesses to report state; it also detects
  24 agent CLIs by process and screen. States: working, needs_input, done, errored, idle.
- **OSC 7501 (Program Status Protocol).** Any program writes
  `ESC ] 7501 ; state=blocked:kind=permission:app=terraform:msg=<base64> ESC \` and shows up in
  the rail and Inbox like an agent. The spec is Superlogical's (the Rex terminal), revision 0.3 of
  7 October 2026; libghostty-vt parses it since ghostty-org/ghostty#14560 (6 October 2026), though
  the Ghostty app doesn't display it yet. `tuios status working --app build --progress 40` writes
  it from a script.
- **Inbox and asking.** `tuios ask-human` puts a question with fixed answers in the Inbox and
  returns the answer; agents mail each other (`send-agent-message`, `ask-agent`) with replies from
  the person marked `verified_human`. Approvals for Claude Code, opencode and others can be
  answered with one key.
- **Fleets.** `tuios fan` starts one prompt in several agents, each in a git worktree;
  `start-agent` runs a helper in its TUI or headless over ACP or the Codex app-server. Pane grants
  (`read`, `write`, `fan`, `respond`, `admin`) limit what an agent may do through tuios.
- **Read-back.** `tuios screenshot` renders a pane to PNG, SVG, ANSI, HTML or text.
- **Dock components.** A dock cell can be any command whose first stdout line becomes the cell,
  refreshed on events or by polling.
- **Automation.** A `.tape` DSL records and replays sessions; `tuios-web` serves the UI in a browser;
  tuios.dev/learn runs the real app compiled to WebAssembly.

## Strengths

- The most complete agent-supervision feature set we found in a terminal tool: Inbox, ask-human,
  mail, fleets, grants, resume, remote hosts.
- Excellent agent ergonomics: structured hints in errors, `--skill` guides, MCP server.
- Takes protocols seriously (OSC 7501, kitty graphics and keyboard, mode 2026) and is adopting
  libghostty-vt.
- Event-driven rendering, zero idle CPU.

## Weaknesses / limits

- One maintainer and a release every few days: the surface is very large for its age and the
  agent features are about two weeks old. Stability is unproven.
- 5,100 stars against herdr's 43,000; it ships a herdr compatibility layer, which suggests herdr
  set the shape.
- What it can put in front of a person is limited to panes of terminal programs plus fixed-answer
  questions in the Inbox. No forms, tables or bound data beyond a dock cell.
- We did not run it; everything here is from the README, the docs folder and the release list.

## Relation to fictty

**Complement.** TUIOS is the frame; fictty is content in a pane. The overlap to watch is
`ask-human` (a one-question form living in the frame) and dock components (commands bound to
cells, the same "data skips the model" idea at a very small scale). If TUIOS grows richer
Inbox items (a table to pick from, a multi-field form) it starts doing fictty's job inside its own
chrome.

## Could fictty adopt it instead of building?

No. TUIOS manages terminals; it has no UI model an agent can write, patch by id or read back as
state. `tuios screenshot --text` gives text read-back of *any* pane, which is useful and means
"exact text read-back" alone is not unique to fictty: what is unique is the semantic state
(`get --state`) and the data binding. Adopting TUIOS would mean building fictty as a TUIOS plugin
in Go against a fast-moving single-author API. Not worth it.

## What fictty should take from it

- **Emit OSC 7501** when a screen waits for the person (`state=blocked:kind=question`), and `done`
  when it closes. One escape sequence puts fictty in the Inbox of TUIOS, Rex and (soon)
  libghostty-vt-based terminals.
- **Copy the error `hint` shape** (`did_you_mean`, `available`, `command`) for the socket protocol.
- **A `--skill` style self-description** an agent can read in one call.
- **Popups:** `tuios popup -- fictty run ui.json` is the natural way to put a fictty screen in front
  of someone without disturbing their layout; test that path.
- Note the competition: `ask-human` is the minimal form. fictty's form should be clearly better
  (typed fields, validation, live data), or it should just use `ask-human`.

## Sources

- https://github.com/Gaurav-Gosain/tuios (README, docs/PROGRAM_STATUS.md, docs/protocol.md,
  docs/AGENT_STATE.md, examples/dock/README.md, releases), read 10 October 2026
- https://tuios.dev
- https://www.superlogical.com/rex/docs/build/program-status (OSC 7501 spec, rev 0.3)
- https://github.com/ghostty-org/ghostty/pull/14560
