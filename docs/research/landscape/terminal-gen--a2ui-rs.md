# a2ui (Rust crate, ratatui renderer for A2UI)

- **Maker:** Liangdi (GitHub `Liangdi`)
- **URL:** https://github.com/Liangdi/a2ui, https://docs.rs/crate/a2ui
- **Status (2026-10-10):** 15 stars, MIT. Repo created 31 December 2025; crate 0.3.1 published
  18 June 2026 (seven releases); last push 5 July 2026. Bilingual README (Chinese first). We
  couldn't find a download count or any public user.

## What it is

A Rust implementation of the A2UI v1.0 protocol with a framework-agnostic core (`a2ui-base`:
protocol types, data model, catalogues, message processor, validation) and seven backends. The
default is `a2ui-tui` on ratatui; Slint, egui, Bevy, Iced, Dioxus and GPUI are optional.

## The problem it's solving

A2UI shipped with web and mobile renderers only. This crate lets a Rust program, including a
terminal one, render the surfaces an A2UI agent streams.

## Its path / bet

Be the complete Rust A2UI stack: every component, every backend, the same core logic for focus
and events everywhere. The terminal is the reference backend.

## How it works

- All 18 basic-catalogue components render in the terminal, nine of them interactive. Images go
  through `ratatui-image` (kitty, iTerm2, sixel, half-blocks); video is a placeholder.
- JSON Pointer data binding, 14 client-side functions (validation, formatting, pluralise),
  capability negotiation, inline catalogues, a fault-tolerant `parse_and_fix` for malformed model
  output.
- 235 tests, 19 examples (JSONL stream stepping, action responses, a sci-fi HUD fed by
  `updateDataModel`).
- It is a library. The examples hard-code JSONL messages and own the terminal loop themselves;
  there is no socket, CLI or agent transport in the crate. Getting messages from an agent to the
  renderer, and actions back, is the host's job.

## Strengths

- Thorough protocol coverage and tests; the core is cleanly separated from rendering.
- Same Rust and ratatui stack as fictty.
- The validation and repair code for model-written payloads is useful on its own.

## Weaknesses / limits

- One maintainer, little visible adoption, development burst in June 2026 then quiet.
- Breadth over depth: seven backends, a known build conflict between two of them.
- No runtime: no attachable process, no read-back, no data sources other than the agent sending
  `updateDataModel`.
- Tracks A2UI, which is still moving (v0.9 stable, 1.0 release candidate).

## Relation to fictty

Build on, narrowly. It answers "can A2UI render in a ratatui terminal" with yes, which means an
A2UI adapter is table stakes, not a moat. Its `a2ui-base` crate is the part fictty could use.

## Could fictty adopt it instead of building?

Not as the runtime: it has none of the loop. As a dependency for the planned A2UI input adapter,
maybe: `a2ui-base` parses, validates and maintains A2UI surfaces and data models. The risk is a
single-maintainer crate tracking a moving spec. A hard-nosed reading: write fictty's adapter as a
translation onto fictty's own value (which it has to be anyway, since fictty doesn't render A2UI
surfaces directly), and borrow `a2ui-base`'s types or validation only if they save real work.

## What fictty should take from it

- Test the A2UI adapter against the official spec examples the way this crate does (they are
  Apache-2.0 fixtures).
- `parse_and_fix`: repair common model JSON errors (smart quotes, trailing commas) before
  rejecting a push, and say what was fixed.
- The HUD demo is a good proof that "data flows, the layout stays" reads well in a screenshot.

## Sources

- https://github.com/Liangdi/a2ui (README_EN.md, CHANGELOG.md, examples, read from a clone on
  2026-10-10)
- https://docs.rs/crate/a2ui/latest
- https://api.github.com/repos/Liangdi/a2ui
