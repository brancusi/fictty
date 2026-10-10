# pi (and pi-tui)

- **Maker:** Mario Zechner (creator of libGDX); since April 2026 part of Earendil (Armin
  Ronacher, Colin Sidoti), per several secondary reports
- **URL:** https://github.com/earendil-works/pi (formerly `badlogic/pi-mono`)
- **Status (2026-10-10):** about 114,000 stars, MIT. v1.1.0 on 7 October 2026. Repository
  created August 2025. A monorepo: `ai` (unified LLM API), `agent` (loop), `tui` (pi-tui),
  `coding-agent` (the CLI), plus `server`, `protocol`, `client`, `mcp`, `codemode` and others.
  Not in the first landscape pass; added here because it's the fourth most-starred harness and
  the one whose extensions can draw.

## What it is

A deliberately minimal terminal coding agent (four tools: read, write, edit, bash) whose pitch is
that everything else is an extension, a skill, a script or a package. Third-party reports count
thousands of extensions in its package catalogue (2,143 in May 2026, unverified). OpenClaw uses
pi as its agent core.

## The problem it's solving

Harness lock-in and bloat: a small, hackable agent where the person owns the loop. Its TUI
library, pi-tui, solves flicker-free differential rendering for that agent and is published on
its own.

## Its path / bet

- **Extensions can draw.** The `ctx.ui` API in an extension offers dialogs (`select`, `confirm`,
  `input`, `editor`), notifications and status, persistent widgets near the editor
  (`setWidget`), replaceable header, footer and editor, custom renderers for tool calls and
  session entries, and `ctx.ui.custom()`: a temporary interactive screen or overlay that owns
  input until it resolves with a value. Examples include a Q&A dialog, a modal editor and a Doom
  overlay.
- **Lines, not boxes.** A pi-tui component renders "an array of terminal lines for an available
  width"; the TUI diffs lines (or viewport rows) and wraps updates in synchronized output (CSI
  2026). Main-screen and alternate-screen renderers share one interface.
- **Small library, real widgets.** `Text`, `Markdown`, `Image` (kitty and iTerm2 protocols),
  `Box`, `VStack`, `HStack`, `ScrollView`, `Input`, `Editor`, `SelectList`, `SettingsList`,
  `MouseRegion`, `Loader`.

## How it works

- An extension is TypeScript loaded by the agent. A custom component implements `render(width)`
  returning lines, optional key and mouse handlers, and `invalidate()`; it calls
  `tui.requestRender()` after a state change and the TUI coalesces renders.
- `ctx.ui.custom(factory, { overlay: true, … })` mounts the component with anchors, offsets and
  responsive visibility, and resolves a promise when the component calls its completion callback:
  the agent's tool call can await a person's answer.
- Debugging: `PI_TUI_WRITE_LOG` captures the raw ANSI stream.

## Strengths

- The most open and composable harness: MIT throughout, the TUI library usable on its own, and an
  extension API that can replace almost any part of the screen.
- `ctx.ui.custom()` returning a promise is a clean model for "ask the person through a screen
  and get structured data back".
- Custom renderers per tool call: a tool's result can be a component, not just text.

## Weaknesses / limits

- **Code, not data.** As with Claude Code mods and opencode plugins, someone writes TypeScript.
- Line-based rendering makes side-by-side layout and layered callouts harder than a cell grid.
- No exact read-back for the model of what an extension drew (we found none).
- Ownership changed in 2026; reports say future commercial additions may not be MIT.

## Relation to fictty

**Inspire.** pi shows what an agent harness looks like when the screen is extensible from the
inside, and its `custom()` promise is the in-process version of the loop fictty runs over a
socket. A pi extension that hosts a fictty UI value in an overlay is easy to imagine; so is a pi
user simply writing the component.

## Could fictty adopt it instead of building?

No. pi-tui is a TypeScript line renderer; fictty is a Rust cell renderer whose value is the
data model, the socket loop and read-back, none of which pi-tui offers. A bridge extension is
possible, behind the Claude Code one.

## What fictty should take from it

- **An awaitable screen.** `fictty ask ui.json` that pushes a screen and blocks until the person
  submits, printing the result as JSON, is pi's `custom()` in fictty's terms (and close to the
  planned `watch`). It makes the common case one command.
- **Tool-call renderers.** A harness could hand fictty a tool result and get a component back:
  "render this JSON as a table" without writing a UI.
- **Overlay anchoring vocabulary** (anchor, offset, margin, responsive visibility) is a good
  model for fictty's layers.
- **Synchronized output** (DEC mode 2026) around every frame. fictty doesn't emit it today (no
  mention in `src/`); pi-tui uses it, and Claude Code probes for it at startup.

## Sources

- Repository and releases: https://github.com/earendil-works/pi
- Extension TUI guide: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/tui.md
- pi-tui README: https://github.com/earendil-works/pi/tree/main/packages/tui
- Ownership and catalogue size (secondary): https://rywalker.com/research/pi,
  https://www.implicator.ai/pi-is-not-a-claude-code-rival-it-is-a-harness-rebellion/

Couldn't verify: the Earendil acquisition from a primary source, extension counts, and whether
pi's `server`/`protocol` packages let an outside process drive the UI.
