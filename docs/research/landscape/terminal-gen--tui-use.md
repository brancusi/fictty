# tui-use (and the PTY-driver crowd)

- **Maker:** GitHub `onesuper`
- **URL:** https://github.com/onesuper/tui-use
- **Status (2026-10-10):** 266 stars, MIT, TypeScript. Created 7 April 2026, last push 11 April
  2026; npm 0.1.20 (11 April), about 230 downloads last month. Ships Claude Code and Codex
  plugins. Looks finished or paused.

## What it is

"Like BrowserUse, but for the terminal." A CLI and daemon that run any program in a PTY behind a
headless xterm emulator so an agent can `start` it, `type`, `press` keys, take a `snapshot` of the
screen as plain text (or JSON), `find` text, scroll, and `wait` until the screen settles or a
pattern appears.

## The problem it's solving

Agents are stuck the moment a program wants a human at the keyboard: REPLs, debuggers holding
hours of in-memory state, full-screen TUIs. `tmux send-keys` has no signal for "done responding",
so agents sleep and poll.

## Its path / bet

Treat every terminal program as a black box and give the agent a snapshot loop with smart waiting.
Works with every program ever written; knows nothing about any of them.

## How it works

PTY output goes through a headless xterm emulator; each render event resets a debounce timer, so
`wait` resolves after about 100 ms of quiet, or as soon as `--text` matches. Snapshots include a
`highlights` list of inverse-video spans, the conventional way TUIs mark a selection, which is a
cheap semantic hint.

## The others in this group

The same design appears many times in 2026, which says the need is real and the solution is a
commodity:

- **pilotty** (msmps, Rust, 157 stars, no licence file, January 2026): "the terminal equivalent of
  agent-browser", built to let agents drive OpenTUI and opencode.
- **agent-tui** (pproenca, Rust, MIT, 118 stars, pushed today): daemon, JSON output, a live
  preview over WebSocket JSON-RPC.
- **PTY-Agent** (ming-14, Python), **agentic-tui** (codeboltai), **tui_test** (pmarreck, fork of
  agent-tty): small variants.
- The semantic and test-grade versions have their own dossiers: ConductorOne's agent-tui
  (`terminal-gen--agent-tui-c1.md`), Microsoft's tui-test (`terminal-gen--tui-test.md`), Coder's
  agent-tty (`terminal-gen--agent-tty.md`); MCP servers and headless terminals are in
  `terminal-gen--terminal-mcp.md`.

## Strengths

- Universal: any program, no cooperation needed.
- `wait` with debounce or pattern fixes the worst part of tmux-based driving.

## Weaknesses / limits

- The agent sees characters, not meaning: which row is selected, what a field is called, what
  happens on Enter are all guesses from the picture.
- Every agent has to relearn each program's keys.
- Low adoption for each individual tool; the space is fragmented.

## Relation to fictty

Complement. Drivers are for programs someone else wrote; fictty is for a screen the agent writes
itself and therefore understands. They meet in two places: an agent can drive fictty with a
driver (it works, but throws away fictty's state read-back), and fictty can use a driver to test
itself inside a real terminal emulator.

## Could fictty adopt it instead of building?

Not for the screen. fictty's read-back is exact because it owns the state; a driver can only
reconstruct it. For end-to-end tests of fictty under a real emulator, yes, a driver is the right
tool (see tui-test).

## What fictty should take from it

- `wait --text` and wait-for-stable as verbs. fictty's `watch` should accept "until this text is
  on screen" as well as events, so agents that only know screen-driving can still use it.
- Expose selection explicitly. tui-use infers selection from inverse video; fictty knows it, and
  `screen` should mark it the same way (and `get --state` already has it).
- The flood of near-identical drivers is a warning: generic plumbing is easy to clone. fictty's
  value has to be in what only a runtime that owns the state can do.

## Sources

- https://github.com/onesuper/tui-use (README)
- https://github.com/msmps/pilotty, https://github.com/pproenca/agent-tui,
  https://github.com/ming-14/PTY-Agent, https://github.com/codeboltai/agentic-tui,
  https://github.com/pmarreck/tui_test
- GitHub API and npm registry data, fetched 2026-10-10
