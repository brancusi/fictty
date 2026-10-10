# iocraft (added in this pass)

- **Maker:** Chris Brown (ccbrown).
- **URL:** https://github.com/ccbrown/iocraft
- **Status, 10 October 2026:** about 1,560 stars, Apache-2.0. v0.9.1 (4 September 2026); last push
  9 October 2026.

## What it is

A Rust crate for declarative CLIs and TUIs in the style of React: an `element!` macro for JSX-like
markup, components with props and hooks (state, futures, terminal events), and flexbox layout through
Taffy. It renders either to a full-screen terminal or as plain styled output to logs.

## The problem it's solving

Bringing React's model and flexbox to Rust terminal UIs, which ratatui doesn't offer.

## Its path / bet

Ink's model, in Rust, with its own renderer rather than ratatui.

## How it works

Components return element trees; iocraft lays them out with Taffy and renders a canvas; hooks
trigger re-renders; `render_loop` or `fullscreen` runs an interactive app.

## Strengths

- Flexbox (Taffy) in Rust: the layout model agents know, with a fast pure-Rust engine.
- Output both to an interactive terminal and to non-TTY text.

## Weaknesses / limits

- Smaller ecosystem than ratatui; no charts or pixel graphics comparable to fictty's needs.
- The UI is Rust code with hook state, not serializable.

## Relation to fictty

**Watch.** The interesting part for fictty is Taffy.

## Could fictty adopt it instead of building?

No. Switching from ratatui would cost the widget set and gain only a component model fictty doesn't
want (fictty's UI is data, not components with hooks).

## What fictty should take from it

- **Taffy for layout** if fictty's row/column sizing grows towards flexbox or grid. Taffy can compute
  layout on fictty's node tree and hand ratatui the `Rect`s, keeping ratatui for drawing.

## Sources

- https://github.com/ccbrown/iocraft (README; GitHub API 10 Oct 2026)
- https://docs.rs/taffy
