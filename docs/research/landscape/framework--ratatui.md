# ratatui

- **Maker:** the ratatui organisation (community; Orhun Parmaksız and Josh McKinney are the most visible maintainers). Forked from tui-rs in 2023.
- **URL:** https://ratatui.rs, https://github.com/ratatui/ratatui
- **Status, 10 October 2026:** about 22,900 stars, MIT. Latest release `ratatui` 0.30.2 (19 June 2026); 0.30.0 (26 December 2025) was the largest release so far. The main branch was pushed to on 10 October 2026. Still pre-1.0.
- **fictty today:** fictty is built on ratatui 0.30 already (`Cargo.toml`). It uses `Buffer`, `Cell`, `Layout`, `Terminal`, `TestBackend`, the stock widgets (Paragraph, Chart, Gauge, Sparkline, Canvas, Block) and crossterm through ratatui's re-export.

## What it is

A Rust library for drawing terminal interfaces. You call `terminal.draw(|frame| ...)` with a closure
that renders widgets into a cell buffer; ratatui diffs that buffer against the previous one and
writes only the changed cells. It is *immediate mode*: there is no retained widget tree, no event
loop, no state management and no focus model. Those are left to the application.

## The problem it's solving

Getting a fast, flicker-free, correctly laid-out grid of styled cells onto many terminals from Rust,
with a widget set good enough that most apps don't write their own drawing code.

## Its path / bet

Stay a library, not a framework. Keep the core small and modular (0.30 split the workspace into
`ratatui-core`, `ratatui-widgets`, per-backend crates for crossterm, termion, termwiz and termina,
and `ratatui-macros`, and added `no_std` support so it runs on microcontrollers through mousefood).
Let an ecosystem build frameworks, effects (tachyonfx), web backends (Ratzilla) and component
systems (tui-realm) on top.

## How it works

- A `Buffer` is a rectangle of `Cell`s (symbol, fg, bg, modifiers). Widgets implement `Widget` or
  `StatefulWidget` and write into a `Rect` of the buffer.
- `Layout` splits rectangles by constraints (length, min, max, percentage, ratio, fill) with a
  Cassowary solver.
- `Terminal` keeps two buffers, diffs them after each draw and sends the minimum escape sequences
  through a `Backend`. `TestBackend` keeps the buffer in memory, which is how fictty's `render`
  command and its view tests work without a terminal.
- Input is not ratatui's job; apps read crossterm (or termion/termwiz) events themselves.

## Strengths

- Fast and predictable. fictty's own measurements (a 50,000-row list replaced 20 times a second
  drawn in 0.18 ms; frames under 1 ms at 1,000 patches a second) sit on ratatui's diff.
- The most-used Rust TUI library. OpenAI's Codex CLI draws with it (`codex-rs` pins ratatui 0.30.2
  with a forked crossterm, checked 10 October 2026).
- Immediate mode fits the Elm architecture exactly: `view(&State)` draws from scratch each frame,
  so there is no second copy of the UI to keep in sync. That is fictty's "one source of truth" rule
  for free.
- `TestBackend` gives an exact in-memory frame, the foundation of `fictty screen` and `render`.
- Ecosystem pieces fictty may want: Ratzilla (browser), tachyonfx (effects), mousefood (embedded).

## Weaknesses / limits

- No retained tree, focus, event routing, data binding or theming: every app rebuilds these. That
  is exactly the gap fictty fills, but it means fictty owns that code.
- No pixel graphics in core. fictty does kitty graphics itself (`gfx.rs`); ratatui-image exists as
  a third-party crate but fictty doesn't use it.
- Layout is constraint-based, not flexbox. Agents and web-trained models think in flexbox; fictty's
  row/column layout has to translate.
- Pre-1.0 with breaking changes each minor release (0.30's breaking-changes list is long).
- Accessibility: none built in; a cell buffer carries no semantics.

## Relation to fictty

**Build on** (already). ratatui is fictty's drawing layer and should stay so.

## Could fictty adopt it instead of building?

It already has, for the part ratatui does. ratatui cannot replace the rest: it has no loop, no
socket, no serializable UI value, no data sources, no focus model and no read-back beyond a test
buffer. Nothing in ratatui's roadmap points that way. The honest risk runs the other direction:
fictty should not drift into rewriting what ratatui gives (layout, widget drawing, the diff).

## What fictty should take from it

- Keep the line where it is: ratatui draws, fictty decides. Use stock widgets wherever they fit,
  as AGENTS.md already says.
- Track 0.31+ for the modular crates; depending on `ratatui-core` plus selected widgets could trim
  build time.
- Ratzilla means a fictty `view` could draw into a browser with no new rendering code (see
  `framework--ratzilla.md`).
- Look at ratatui-image and tachyonfx before writing more pixel or animation code by hand.

## Sources

- https://github.com/ratatui/ratatui (stars, licence, workspace layout; GitHub API, 10 Oct 2026)
- https://github.com/ratatui/ratatui/releases (0.30.0, 26 Dec 2025; 0.30.2, 19 Jun 2026)
- https://ratatui.rs/highlights/v030/
- https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md
- https://github.com/openai/codex/blob/main/codex-rs/Cargo.toml (ratatui 0.30.2, crossterm fork)
- fictty `Cargo.toml`, `src/view.rs`, `src/runtime.rs`, `docs/design/architecture.md`
