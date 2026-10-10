# Ratzilla (added in this pass)

- **Maker:** the ratatui organisation (started by Orhun Parmaksız).
- **URL:** https://github.com/ratatui/ratzilla
- **Status, 10 October 2026:** about 1,470 stars, Apache-2.0 (verify per crate; the repo reports
  Apache-2.0). Latest release v0.3.1 (8 June 2026); last push 4 July 2026. Young but maintained by
  the same organisation as ratatui.

## What it is

A set of ratatui backends that run in the browser through WebAssembly: `DomBackend` (cells as DOM
elements), `CanvasBackend`, and a WebGL2 backend. You keep writing ratatui widgets and draw them into
a web page; key and mouse events come from the browser.

## The problem it's solving

Terminal-styled web apps written in Rust, and getting ratatui apps onto the web without a terminal
emulator in between.

## Its path / bet

Treat the browser as just another ratatui backend. No xterm.js, no pseudo-terminal: the widgets draw
straight into DOM or canvas.

## How it works

Build with `trunk` to WASM; create a `Terminal` over a web backend; register key handlers; call
`draw_web` with the same kind of closure you pass to `terminal.draw`.

## Strengths

- Same drawing code as the terminal, so a ratatui-based `view` ports with little change.
- No server-side terminal stream: unlike textual-serve, the app itself runs in the page.
- The DOM backend leaves real text in the page, which is better for copy, search and accessibility
  than a canvas of pixels.

## Weaknesses / limits

- WASM build tooling (trunk); no kitty graphics (fictty's pictures would need a separate path, though
  in a browser they could simply be `<img>` or SVG).
- Pre-1.0, slower release cadence than ratatui itself.
- Event handling is callback-based and different from crossterm's, so fictty's runtime would need a
  second, browser runtime.

## Relation to fictty

**Complement / adopt.** Because fictty's `update` and `view` are pure and its drawing is ratatui, a
browser surface could reuse both, with only `runtime.rs` replaced.

## Could fictty adopt it instead of building?

For a browser surface, yes, it is the cheapest route found: compile `model`, `state` and `view` to
WASM and draw with `DomBackend`. Unverified: whether `resvg` and caretline build for `wasm32` without
changes. It does not replace anything fictty has today; it adds a surface.

## What fictty should take from it

- Keep `state.rs` and `view.rs` free of anything that wouldn't compile to WASM (threads, sockets,
  `std::process`). The Elm rules in AGENTS.md already push this way; Ratzilla is a reason to enforce
  them.
- A shared, read-only fictty screen in a browser (for someone not in the terminal) becomes a small
  project rather than a new renderer.

## Sources

- https://github.com/ratatui/ratzilla (README, backends; GitHub API 10 Oct 2026)
- https://docs.rs/ratzilla
- Talk: "Bringing Terminal Aesthetics to the Web With Rust (and Vice Versa)", linked from the README
