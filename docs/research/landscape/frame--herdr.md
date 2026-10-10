# herdr

- **Maker:** herdr (GitHub org `herdrdev`); the main committer by far is Can Celik
  (`ogulcancelik`, oddbit.ai), with about 1,300 of the commits. Enterprise contact at herdr.dev.
- **URL:** https://herdr.dev, https://github.com/herdrdev/herdr
- **Status (10 October 2026):** v0.9.3, released 29 September 2026. Rust, Apache-2.0, 43,188
  stars. Repository created 27 March 2026, v0.1.0 the same day. Roughly 43,000 stars in six and a
  half months makes it the breakout terminal tool of the year. Homebrew, mise and Windows
  installers; a plugin marketplace.

## What it is

"The runtime your coding agents live on." A terminal multiplexer in one Rust binary that keeps
terminals running in a background server, shows local and SSH machines in one window, marks every
pane as working, blocked or idle, and lets agents drive it through a CLI and socket API. It
doesn't wrap the agents; it owns their terminals.

## The problem it's solving

Long-running agents on several machines: losing them when SSH drops, losing track of which one is
blocked, and having no clean way for one agent to start, prompt or wait on another. herdr is
tmux rebuilt around those three needs.

## Its path / bet

Be the substrate, not the agent and not the IDE. Stay a terminal program ("one rust binary, no
electron") so it runs in whatever terminal and over SSH. Make the CLI the API ("the entire Herdr
CLI is the plugin API") so agents, scripts and plugins all drive it the same way. Grow an
ecosystem through agent integrations and a plugin marketplace.

## How it works

- **Server and clients.** Terminals live in a server; `ctrl+b q` detaches, `herdr` reattaches.
  After a machine restart herdr restores the layout and can resume supported agent sessions; the
  original processes don't survive.
- **Socket API.** Newline-delimited JSON over a unix socket (named pipe on Windows):
  `{"id":"req_1","method":"ping","params":{}}`. Methods include `pane.split`, `pane.send_keys`,
  `pane.send_text`, `pane.read` (sources `visible`, `recent`, `recent-unwrapped`, `detection`),
  `pane.wait_for_output`, `agent.wait`, `session.snapshot`, and `events.subscribe` (e.g.
  `pane.agent_status_changed`). Event history isn't durable; a slow subscriber gets `events_lost`
  and should reconcile from a snapshot.
- **Agent status.** Detection by screen for supported agents, or explicit reports: an agent
  inside a pane sees `HERDR_ENV`, `HERDR_PANE_ID`, `HERDR_BIN_PATH`, `HERDR_SOCKET_PATH` and calls
  `herdr pane report-agent $HERDR_PANE_ID --source my-agent --state blocked --message ... --seq N`.
  Reports with a stale `--seq` are ignored. No escape sequence is documented.
- **Plugins (v1).** A `herdr-plugin.toml` manifest declares actions, startup and event hooks,
  keybindings, link handlers and terminal panes. Plugins are any argv command (Bash, Node, Lua,
  Rust). They draw terminal UI only, placed as `overlay` (temporary zoomed pane), `popup` (modal,
  takes all input), `split`, `tab` or `zoomed`. "Native non-terminal plugin UI are not part of
  plugin v1." No sandbox.

## Strengths

- Momentum and adoption: by far the largest agent-specific frame in the terminal.
- A clean, small agent contract (four env vars, one CLI call, a sequence number).
- `pane.read` gives agents text read-back of any pane; `events.subscribe` lets them wait cheaply.
- Plugin popups and overlays are exactly the slot a fictty screen would fill.

## Weaknesses / limits

- What a pane shows is whatever program runs there. herdr has no model of content.
- Plugin UI is terminal-only by design for now; the docs hint that richer plugin UI may come later.
- Processes don't survive a server restart, only the layout and resumable agent sessions.
- Plugins run unsandboxed with full CLI access.
- We couldn't find a public statement about who funds herdr or its business model beyond a
  sponsors file and an enterprise email.

## Relation to fictty

**Complement, strongly.** herdr is the most likely place a person supervising agents will see a
fictty screen. A herdr plugin whose `popup` runs `fictty run`, plus `pane report-agent ... --state
blocked` while it waits for the person, would make fictty feel native there. The risk is the
roadmap line about non-terminal plugin UI: if herdr defines its own UI description for plugins,
it becomes a competitor at the content layer with a 43,000-star distribution.

## Could fictty adopt it instead of building?

No. herdr would give fictty panes, persistence, remote and status for free, but it does nothing
fictty does: no UI value, no patch by id, no state read-back, no data binding. The honest
consequence is the other way round: fictty should not build any frame features (sessions,
detach, remote attach, multi-agent status) because herdr and TUIOS already do them well.

## What fictty should take from it

- **Ship a herdr plugin** (a manifest that opens `fictty run <ui>` as a popup or split) and
  report `blocked`/`idle` through `HERDR_BIN_PATH` when inside herdr.
- **Sequence numbers on reports** to defeat reordering: the same idea fits fictty's patches.
- **The CLI is the API.** herdr's stance matches fictty's (one subcommand per socket operation);
  keep it that way.
- **`pane.read` sources** (`visible` vs `recent-unwrapped`) are a good vocabulary for `fictty screen`
  options.
- **Snapshot plus subscribe with explicit `events_lost`**: the right shape for fictty's `watch`.

## Sources

- https://github.com/herdrdev/herdr (README, releases, contributors), read 10 October 2026
- https://herdr.dev/docs/socket-api/
- https://herdr.dev/docs/plugins/
- https://herdr.dev/docs/add-herdr-support/
