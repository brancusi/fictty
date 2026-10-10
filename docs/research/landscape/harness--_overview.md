# Terminal agent harnesses and the renderers under them

An overview of the harness cluster as of 10 October 2026. Dossiers:
[Claude Code](harness--claude-code.md), [Codex CLI](harness--codex-cli.md),
[opencode and OpenTUI](harness--opencode-opentui.md), [Gemini CLI](harness--gemini-cli.md),
[Crush](harness--crush.md), [pi](harness--pi.md) (added; not in the first pass).

## What changed since the first landscape pass

Three facts that the [first post](../../blog/2026-10-why-the-terminal.md) didn't have:

1. **Claude Code can now be drawn into by code.** Since v2.1.287 (1 October 2026) "mods" are on
   by default: plugins whose JavaScript draws panes beside the transcript, a band above the
   prompt, and replaces the harness's own rows, with buttons, inputs, a cell `Raster`, images,
   live data from processes and timers, and tools the model can call. Claude ships a skill for
   writing them and hot-reloads them. This is the most important finding for fictty in this
   cluster.
2. **Google's flagship terminal agent is no longer Gemini CLI.** On 19 May 2026 Google moved
   consumers to Antigravity CLI, a Go binary sharing a harness with a desktop app and not
   described as open source. The first post's line "all four labs ship their flagship coding
   agent as a terminal program" needs a correction: they all still ship one, but Google's is now
   closed and tied to a desktop app, and OpenAI's rich visuals go to its desktop app.
3. **pi belongs in the table.** About 114,000 stars, MIT, and an extension API (`ctx.ui`) that
   can draw overlays, widgets and per-tool renderers.

## The paths

Every harness has to draw a fast-changing chat in a terminal, and each has chosen a renderer and
a stance on who else may draw.

| harness | renderer | screen mode | who else can draw inside it |
|---|---|---|---|
| Claude Code | React on Anthropic's own renderer (left Ink) | fullscreen (alternate screen, cell diffs) by default for new users; classic available | **mods**: JS element trees in panes, a band, replaced rows; cross-surface |
| Codex CLI | ratatui + crossterm (Rust) | inline: live viewport, history into native scrollback | nobody; visuals are HTML opened in a browser (or the desktop app); Mermaid drawn natively |
| opencode | OpenTUI (Zig core, Solid) | fullscreen | **TUI plugins**: Solid routes, dialogs, slots |
| Gemini CLI | React on a fork of Ink | alternate screen, history written back on exit | nobody |
| Crush | Bubble Tea v2 + Ultraviolet (Go) | fullscreen | nobody |
| pi | pi-tui (TypeScript, line diffs, CSI 2026) | main or alternate screen | **extensions**: overlays, widgets, tool renderers |

Three paths stand out.

- **Own the renderer.** All of them have either written or forked their renderer in the last
  eighteen months. Flicker, scrollback and memory in long sessions are solved problems now, each
  team solved them separately, and none of the solutions is reusable by the others except
  OpenTUI and pi-tui (both TypeScript libraries).
- **Open the screen to code.** Claude Code, opencode and pi let third-party code draw inside the
  harness. All three chose code (JS/TS functions returning element trees or lines), loaded as
  plugins, with full or near-full machine access. None lets the agent push a UI as data, and none
  gives the model an exact read-back of what a plugin drew.
- **Send visuals elsewhere.** Codex, and Google with Antigravity, put rich UI in a desktop app or
  a browser and keep the terminal for text, diffs and (Codex) Mermaid. Their bet is that the
  terminal is the agent's control surface, not its display.

## The open debate

- **Inline or fullscreen?** Codex keeps native scrollback and selection; Claude Code, Gemini
  CLI, opencode and Crush take the alternate screen and rebuild scroll, search, selection and
  copy themselves. Both sides have support among users; Claude Code offers both renderers. fictty
  is fullscreen today; an inline mode deserves thought.
- **Is in-harness UI code or data?** Every harness that opened its screen chose code. The data
  camp (A2UI, json-render) lives outside harnesses. Nobody has yet put a data-driven, agent-
  writable UI *inside* a harness, though Claude Code mods make it a weekend project.
- **Is the terminal the display, or only the control surface?** OpenAI and Google are moving the
  display to desktop apps; Anthropic is investing in the terminal (fullscreen, mouse, mods) and
  in HTML artifacts at the same time; opencode, pi and Crush are terminal-first.

## What's really being solved

Underneath the rendering work, every harness is solving the same two things: **the person has
to watch and steer a long-running agent without the screen fighting them**, and **the transcript
isn't enough** for diffs, plans, approvals, usage and status. The first produced the renderer
rewrites. The second produced side panels (`/diff` as a Claude Code mod), slots and overlays
(opencode, pi), Mermaid (Codex), and desktop apps (Codex, Antigravity).

What none of them solves is the job fictty is aimed at: **a screen the agent writes for this
task, as data, edits by id, that the person drives at frame rate, with live data that never
passes through the model, and that the agent reads back exactly.** Plugins are written for many
sessions by a developer; fictty screens are written for one task by the agent.

## Adopt or build: the hard look

- **No harness offers a UI runtime fictty could adopt instead of building.** The two that come
  closest are harness-locked (Claude Code mods: proprietary, 30 redraws a second, no model
  read-back) or code-first libraries in another language (OpenTUI, pi-tui).
- **The real threat is Claude Code mods,** not as a library but as a substitute: for a Claude
  Code user, a mod (or a first-party "draw this JSON" mod Anthropic could ship any day) covers
  much of fictty's use without installing anything. fictty's defensible ground is everything
  outside one harness (Codex, opencode, pi, Crush, Gemini CLI users; SSH boxes; herdr and TUIOS
  panes), the exact read-back, the frame rate, and replay.
- **So build fictty, and bridge into the harnesses that allow it.** In order: a Claude Code mod
  that renders a fictty value in a pane with a `screen` tool for read-back; later, opencode and
  pi equivalents. Each bridge makes fictty the data format and the harness the surface, which is
  the position the A2UI camp is aiming for, applied to terminals.

## Things to take, across the cluster

- Report state to the window manager the way Crush reports to herdr (idle, working, blocked).
- An awaitable screen (`fictty ask`), as pi's `ctx.ui.custom()` and Claude Code's `AskUserQuestion`
  site do in-process.
- Mermaid as a component, rendered with Codex's Apache-licensed `codex-mermaid`.
- Named regions (slots, render sites) a pushed UI can target.
- Polite focus: never grab keys while the person is typing; wait for width.
- Synchronized output around every frame; a stated redraw ceiling.
- Leave the last frame in scrollback on exit; consider an inline mode.

## Scores at a glance

See the structured entries returned with this research. Calibrated against fictty's job (an
agent-written, data-driven, read-back screen), not against how good each harness is as a coding
agent.
