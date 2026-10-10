# cmux

- **Maker:** Manaflow, Inc. (manaflow-ai)
- **URL:** https://cmux.com, https://github.com/manaflow-ai/cmux
- **Status (10 October 2026):** v0.65.0, released 5 October 2026. Repository created 28 January
  2026; 28,082 stars. Licence: the macOS app, iOS app, CLI and `cmux-tui` are GPL-3.0-or-later;
  server directories (web, relays, workers) are BUSL-1.1 and need a commercial licence for
  production use. Not in our first pass; added here because it's the second-largest
  agent-specific frame after herdr.

## What it is

A native macOS terminal (Swift and AppKit, rendering with libghostty, reading your Ghostty config)
built for running many coding agents: vertical tabs with git branch, PR status, ports and the
latest notification per workspace; a blue ring on panes whose agent needs you; a notification
panel; an in-app browser pane agents can script; SSH workspaces; cloud machines; and one-command
Claude Code agent teams as native splits. The repository also contains `cmux-tui`, a Rust TUI
multiplexer (on libghostty-vt) with a resource protocol (`cmux.protocol/2`) covering machines,
sessions, workspaces, panes, terminals and browsers.

## The problem it's solving

The author's own: many Claude Code and Codex sessions in Ghostty splits, and notifications that
say only "Claude is waiting for your input" with no context. Existing orchestrators were
Electron or Tauri and locked you into their workflow.

## Its path / bet

"cmux is a primitive, not a solution." A fast native terminal plus a browser plus notifications
plus a CLI and socket API, with no opinion about how to use agents: "Give a million developers
composable primitives and they'll collectively find the most efficient workflows."

## How it works

- **Notifications** from standard terminal sequences (OSC 9, 99, 777) and a `cmux notify` CLI to
  wire into agent hooks.
- **Browser pane** with a scriptable API ported from Vercel's agent-browser: agents snapshot the
  accessibility tree, get element refs, click, fill and evaluate JS against your dev server.
- **CLI and socket API** to create workspaces and tabs, split panes, send keystrokes, open URLs.
- **`cmux claude-teams`** runs Claude Code's teammate mode with teammates as native splits, no tmux.
- **`cmux.json`** custom commands in the palette; SSH workspaces route the browser through the
  remote network so `localhost` works.

## Strengths

- Native speed and Ghostty fidelity; it piggybacks on Ghostty's configuration and rendering.
- Uses standard notification sequences, so any program can participate without a cmux SDK.
- The browser pane gives agents a structured read of web content (accessibility tree), which is
  the web's version of fictty's read-back.
- Very fast growth for a nine-month-old project.

## Weaknesses / limits

- macOS-only app (the TUI is the cross-platform path; we didn't evaluate its maturity).
- Split licensing (GPL client, BUSL server) and a company behind it with cloud machines to sell.
- Content beyond terminals is a browser: the HTML path, not a terminal UI model.

## Relation to fictty

**Complement.** A fictty screen runs in a cmux pane and should raise a cmux notification when it
waits. cmux's browser pane is the alternative a cmux user reaches for when they want a richer
screen; fictty wins there only on live data, frame-rate behaviour and state read-back.

## Could fictty adopt it instead of building?

No. It's a frame. `cmux-tui`'s resource protocol is worth reading as a design reference for a
multiplexer protocol, not as something fictty should sit on.

## What fictty should take from it

- **Emit OSC 9/777 notifications** alongside OSC 7501: cmux, Warp and others already read them.
- **"Primitive, not a solution"** is a good line for fictty too: a runtime, not an app.
- **Accessibility-tree snapshots with element refs** are how agents read web UIs; fictty's
  `get --state` should be at least as convenient (stable refs, compact output).

## Sources

- https://github.com/manaflow-ai/cmux (README, LICENSE, cmux-tui/README.md, releases), read
  10 October 2026
- https://cmux.com/blog/zen-of-cmux
- https://cmux.com/de/docs/agent-integrations/claude-code-teams
