# The frame layer: window managers and agent multiplexers

Research as of 10 October 2026. Dossiers: [TUIOS](frame--tuios.md), [herdr](frame--herdr.md),
[zellij](frame--zellij.md), [tmux](frame--tmux.md), [Wave](frame--wave.md),
[the Codex app](frame--codex-app.md), [Warp](frame--warp.md), and four we added:
[cmux](frame--cmux.md), [Superset](frame--superset.md), [Claude Squad](frame--claude-squad.md),
[Vibe Kanban](frame--vibe-kanban.md). Every entity on the original list exists; none was dropped.

## What is really being solved

Not UI. **Attention.** A person now runs five to fifty long-lived agents, on several machines, in
worktrees, for hours. The frame layer answers four questions for them:

1. **Which one needs me?** Per-pane status (working, blocked, done, errored) and one place to see
   every blocked agent: TUIOS's Inbox, herdr's pane marks, cmux's blue rings, Superset's chimes,
   Warp's notifications.
2. **Will it still be there?** Sessions that survive a dropped SSH connection or a reboot (herdr,
   TUIOS, tmux, zellij, Wave's durable sessions).
3. **Can agents coordinate without me?** Socket APIs and CLIs so agents spawn panes, prompt each
   other and wait on each other (herdr, TUIOS, cmux, zellij 0.44, tmux).
4. **Where do I review?** Diffs, previews and PRs beside the agent, with the person's feedback
   routed back (Superset, Vibe Kanban, the Codex app, cmux's browser).

## The paths people are taking

**Path A: the terminal multiplexer, rebuilt for agents.** herdr (Rust, 43k stars in six months),
TUIOS (Go, 5k), with tmux and zellij as the incumbents and Claude Squad as the first-wave
ancestor. They run anywhere a terminal does, cross SSH, and own the PTYs. Their content is
whatever program runs in a pane. They are converging fast on the same surface: a JSON socket API,
`pane.read`-style text read-back of any pane, event subscriptions, status reporting, popups and
overlays for plugins.

**Path B: the desktop agent workspace.** The Codex app (now inside the ChatGPT desktop app),
Warp, cmux, Superset, Wave, Vibe Kanban. They are windows, not terminals, and they are steadily
absorbing the *content* layer: in-app browsers with comments, Sites, MCP Apps, inline
visualizations (Codex); Pages with comments and a watching agent (Superset); browser panes with
accessibility-tree scripting (cmux); AI-written Go widgets (Wave's Tsunami, planned). In every
case the content is **HTML or code**, checked by screenshot or DOM.

**The emerging standard: status reporting.** Three mechanisms compete: OSC 7501 (the Program
Status Protocol from Superlogical's Rex terminal, revision 0.3 of 7 October 2026, implemented by
TUIOS and parsed by libghostty-vt), herdr's `pane report-agent` CLI with sequence numbers, and the
older notification sequences OSC 9/99/777 (cmux, Warp). Any program in a pane can now say "I'm
waiting for you" and appear in the frame's inbox.

## The open debate

- **Terminal frame or desktop frame?** herdr's growth says the terminal frame is thriving; the
  Codex-into-ChatGPT merger says the labs are betting on windows. Both are growing. Remote,
  long-running agents favour the terminal; review and rich previews favour the window.
- **Does the frame eat the content?** In GUI frames, yes, already, via HTML. In terminal frames,
  not yet: herdr's plugin v1 is "terminal UI only" and says native plugin UI isn't part of v1;
  TUIOS's `ask-human` (a question with fixed answers) and dock components (commands bound to
  cells) are the first small steps. Zellij is the only one with host-drawn components (table,
  ribbon, nested list, text) described by plugins, but they are written from compiled WASM code.
- **Is the frame a business?** Vibe Kanban's company shut down in April 2026 despite 28k stars;
  Warp open-sourced its client and leans on cloud orchestration; cmux and Superset sell cloud and
  mobile. The frame is crowded and hard to charge for.

## What this means for fictty

- **The scope line holds.** No frame lets an agent push a UI value, patch it by id, bind live data
  to it, and read back its *state*. fictty is still content-for-a-pane, and every terminal frame
  is a host for it.
- **But one claim is weaker than we wrote.** Exact *text* read-back is no longer unique: herdr
  `pane.read`, zellij `subscribe`/`dump-screen`, tmux `capture-pane` and `tuios screenshot --text`
  all give agents the text of any pane. fictty's distinct claims are **semantic state read-back**
  (`get --state`: which row is selected, what was typed, by node id), **data that skips the
  model**, and **behaviour at frame rate**. The blog post should say this.
- **Integrate, don't compete.** Concretely: emit OSC 7501 and OSC 9/777 when a screen waits; call
  `herdr pane report-agent` when `HERDR_ENV=1`; ship a herdr plugin manifest and recipes for
  `tuios popup`, tmux 3.8 modal panes (`new-pane -O -W`) and zellij floating panes. Build no frame
  features (sessions, detach, remote, multi-agent status).
- **The window of opportunity is real and finite.** If herdr adds a declarative plugin UI, or TUIOS
  grows `ask-human` into forms and tables, the content layer gets claimed from above by tools with
  the distribution. fictty's best defence is to be the thing they'd embed: small, data-first,
  embeddable, spoken over a socket.
- **Teaching.** Frames are about supervising agents, not understanding things; their content is
  still mostly text the agent printed. The exceptions point at teaching: zellij's read-only
  sharing tokens ("for teaching, demonstrations"), Codex's inline visualizations, Superset's
  commentable Pages. None of them gives a learner a live, explorable screen built for the moment
  in the place the agent already runs. That remains open ground.

## Also seen, not dossiered

Conductor (closed macOS app for parallel Claude Code/Codex), Sculptor (Imbue, MIT, 238 stars),
Crystal (now Nimbalyst, MIT, 3.1k stars, last release February 2026), Coder's agentapi (an HTTP
API over agent TUIs, MIT, 1.5k stars), Dagger's container-use (Apache-2.0, 4k stars; isolation,
not a frame), the Rex terminal (Superlogical; author of OSC 7501; we couldn't confirm licence or
platforms), and the terminals themselves (Ghostty 62k, kitty 35k, WezTerm 29k), whose features
(libghostty-vt, kitty graphics, kitty sessions) increasingly decide what frames can do.

## Sources

See each dossier. Star counts and release dates were read from the GitHub API on 10 October 2026.
