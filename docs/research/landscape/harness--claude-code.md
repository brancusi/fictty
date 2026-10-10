# Claude Code (its renderer, and mods)

- **Maker:** Anthropic
- **URL:** https://github.com/anthropics/claude-code (issues, changelog, plugin and mod sources;
  the CLI itself is closed), docs at https://code.claude.com/docs
- **Status (2026-10-10):** about 150,000 stars on the public repo. Proprietary ("© Anthropic PBC.
  All rights reserved", commercial terms). Latest release v2.1.296 on 9 October 2026; releases
  land almost daily. Mods (code plugins that can draw) are on by default since v2.1.287
  (1 October 2026) in the terminal, v2.1.286 in the Desktop app's Code tab; the API is still
  marked "may change between releases without notice". Fullscreen rendering is a "research
  preview" (since v2.1.89, 1 April 2026) but is the default for most people who first used Claude
  Code on or after 6 May 2026.

## What it is

Anthropic's coding agent, run as a terminal program (and inside the Desktop app, VS Code,
JetBrains, the web and mobile through Remote Control). This dossier is about two things the first
landscape pass only touched: **how it draws**, and the October 2026 news that **third-party code
can now draw inside it** through "mods".

## The problem it's solving

Two problems. First, a chat UI that streams long, fast-changing output into a terminal flickers,
jumps scrollback and grows memory without bound; Claude Code spent 2025 and 2026 fixing that.
Second, the transcript is the only output channel, and some things (a diff, a queue of approvals,
a context-usage chart, a picker) are better as a standing panel than as more lines of text. Mods
are Anthropic's answer to the second: let anyone add panes, a band above the prompt, buttons and
fields, and restyle the harness's own rows.

## Its path / bet

- **Rendering:** keep React as the component model, but replace Ink's renderer with Anthropic's
  own (Steinberger, December 2025, quoting Thariq: Ink "lacked the fine-grained incremental
  updates"). Then add a second, alternate-screen "fullscreen" renderer that sends only changed
  cells, renders only visible messages, captures the mouse and owns scrolling, like vim or htop.
- **Extensibility:** code, not data. A mod is JavaScript or TypeScript running *inside* Claude
  Code's process in a sandboxed module environment (no DOM, no Node), hooking every engine event
  as middleware (`on(event, matcher, ($, e, next) => …)`), with a capability API (`$`) the engine
  can audit before load (`claude plugin validate` lists every event hooked and every call made).
- **One tree, many surfaces:** a mod returns an element tree (`Box`, `Text`, `Button`, `Input`,
  `Select`, `Link`, `Code`, `Markdown`, plus `Raster` and `Image` in the terminal, `Svg` on desktop),
  and each surface (terminal, desktop, mobile, VS Code) draws it with its own widgets. That's the
  A2UI idea, applied inside one vendor's harness, with the tree produced by code.

## How it works

- **Render sites.** `ui.render` fires before each draw of a site: `Pane` (a sidebar beside the
  transcript in a wide fullscreen terminal, a framed region above the prompt otherwise),
  `AbovePrompt` (a shared band), and the harness's own rows (`UserMessage`, `AssistantMessage`,
  `ToolUse`, `ToolResult`, `Spinner`, `AskUserQuestion`, `PromptHint`, and more). A hook returns a
  tree, or `next(e)` to keep the engine's drawing, or the engine's drawing wrapped in its own. The
  permission prompt is deliberately not a render site.
- **Panes.** `$.ui.open({ id, title, focus, closeOnEscape, rows, columns })`. A pane the mod opens
  on its own (a timer, a turn hook) appears only at 144 columns or more (110 once the person has
  opened it), so a mod can't take over a small screen. Keyboard focus is granted only when the
  prompt is empty; Tab and arrows are reserved for the engine.
- **The loop.** Elm-ish but with mutable module state: a callback changes a variable (or
  `$.state`, which is reactive and survives module reloads), calls `$.ui.invalidate('ui.render')`,
  and the engine re-runs the render hook. `$.store` persists JSON (4 MiB) across sessions.
- **Speed limits.** Redraws are throttled to 10 a second, or 30 in the terminal for the visible
  pane, the band and the hint line; faster calls coalesce. `$.ui.blit` repaints one `Raster`
  (up to 512×256 cells, packed as base64 code point/fg/bg triplets) without re-running the hook.
  `Client` elements run a second module on the drawing thread for animation and pointer input.
- **Data without the model.** `$.process.run/spawn`, `$.http.fetch`, `$.fs`, `$.clock.every`
  let a mod feed a pane from commands with no model turn. `$.tool.register` gives the model a new
  tool (`mcp__<plugin>__<name>`); `$.prompt.submit` starts a turn from a background job;
  `$.model.complete` calls a model on the user's plan.
- **Writing one.** "Describe what you want in a Claude Code session, and Claude writes the mod":
  a built-in `plugin-authoring` skill ships with Claude Code, and `--plugin-dir` hot-reloads the
  module on save. So an agent can, in principle, write and load its own UI mid-session.
- **Testing and read-back.** `claude plugin test` runs a mod headless; `$.ui.mount` draws a
  site on a named surface and `find`/`press`/`input`/`key` act on elements by key. There is no
  built-in way for the *model* in a live session to read what a pane shows; a mod would have to
  expose that through a tool it registers. `$.ui.selection()` returns what the person selected.
- **Built-in mods.** `/diff` is now a mod (source public under `mods/diff`): a docked pane with a
  pinned file list over scrolling hunks, refreshed as Claude edits.

## Strengths

- It's where a large share of fictty's intended users already are, and it's first-party.
- Draws inside the harness itself: no second pane, no tmux, no window manager required.
- Cross-surface by design: the same tree draws in the terminal, the Desktop app and (partly)
  mobile and VS Code.
- A serious capability model: every side effect goes through `$`, auditable before install,
  overridable by organisation policy mods.
- Live data without the model, a tool registry, and hot reload: every ingredient of fictty's loop
  is available to a mod author.
- Thoroughly documented (TypeScript declarations of 13,000+ lines, guides, a test kit).

## Weaknesses / limits

- **Code, not data.** An agent that wants a screen has to write a JavaScript module, get it
  validated and loaded, and debug it. That's the HTML path with a smaller element set. Nothing
  stops a mod from accepting a JSON value and drawing it, but Claude Code ships no such mod.
- **No exact read-back for the agent.** The test kit can find elements; the live model can't see
  a pane unless someone builds that.
- **Frame-rate ceiling.** 30 redraws a second for the visible pane, coalesced; fine for
  dashboards, not for a 500-update-a-second stream drawn at 100+ fps.
- **One harness.** A mod only runs in Claude Code (terminal and Desktop; not `claude -p`, cloud
  sessions or the VS Code chat panel). Codex, opencode and the rest can't use it.
- **Early-access API** that "may change between releases without notice", and code that runs
  with the user's full permissions, not sandboxed from the filesystem.
- **Proprietary.** No fork, no embedding outside Anthropic's product.

## Relation to fictty

**Compete,** for the Claude Code user specifically. The scope doc said fictty is "what runs
inside" a herdr or TUIOS pane; Claude Code now has its own panes, and they're closer to the
transcript than any pane a multiplexer offers. For "show the person a live table beside the
chat", a 200-line mod is now a credible alternative to installing fictty. It's also a possible
**host**: a fictty mod could register a `show` tool that takes a fictty UI value and draws it with
`Box`/`Text`/`Raster`, with data from `$.process.spawn`.

## Could fictty adopt it instead of building?

Not as a replacement. It's harness-locked, proprietary, code-first, capped at 30 redraws a second
and gives the model no read-back. fictty's claims (data not code, exact read-back, frame rate,
runs anywhere a terminal does, replay) are things a mod doesn't give you out of the box. But the
honest version: for someone who only uses Claude Code and wants a dashboard beside the chat,
mods plus a well-written skill get most of the way there today, and Anthropic could ship a
first-party "draw this JSON" mod at any time. The defensible ground for fictty is everything
outside one harness (Codex, opencode, pi, SSH boxes, herdr/TUIOS panes) and the exactness of
the loop.

## What fictty should take from it

- **Build the bridge before someone else does.** A small Claude Code mod that renders a fictty UI
  value in a pane (degrading layers and pixels to `Box`/`Text`/`Raster`), with a `screen` tool for
  read-back, turns the biggest threat into a distribution channel. Worth a spike.
- **Render sites as a concept.** "Places a UI can draw" (pane, band, status line, toast, a
  replaced row) is a cleaner vocabulary than fictty's single root plus layers.
- **Polite focus rules.** A pane the agent opens unasked never steals keys and waits for width;
  fictty's `push` should have an equivalent ("don't grab focus while the person is typing").
- **Capability listing before run.** `claude plugin validate` prints every effect a module can
  have. fictty's command policy (plan phase 3) should print the same for a pushed UI: every
  command it binds, before it runs.
- **Render throttling as a stated limit,** not an accident: say what fictty's ceiling is.
- **Surface-neutral trees.** One tree drawn by several surfaces is where the industry is going
  (A2UI, json-render, now this); fictty's value should stay renderable elsewhere.

## Sources

- Mods overview: https://code.claude.com/docs/en/plugins/mods/overview
- Draw in the interface: https://code.claude.com/docs/en/plugins/mods/interface
- Mods API: https://code.claude.com/docs/en/plugins/mods/api
- Mods reference (limits, render sites, elements): https://code.claude.com/docs/en/plugins/mods/reference
- Type declarations: https://github.com/anthropics/claude-code/blob/main/mods/types/claude-code.d.ts
- Built-in mod sources: https://github.com/anthropics/claude-code/tree/main/mods
- Changelog: https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md
- Fullscreen rendering: https://code.claude.com/docs/en/fullscreen
- Peter Steinberger, Claude Code's signature flicker: https://steipete.me/posts/2025/signature-flicker
- Release dates from the GitHub releases API (v2.1.89, v2.1.110, v2.1.277, v2.1.287, v2.1.296)

Couldn't verify: the internal design of the custom renderer beyond what the docs say (cell diffs,
visible-only rendering), and the earlier secondary claim that Anthropic patched tmux and VS Code
for synchronized output (the fullscreen doc says only that tmux through 3.6 lacks it).
