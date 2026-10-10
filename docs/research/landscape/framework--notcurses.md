# Notcurses

- **Maker:** Nick Black (dankamongmen).
- **URL:** https://github.com/dankamongmen/notcurses, https://notcurses.com
- **Status, 10 October 2026:** about 4,700 stars, Apache-2.0. Latest release v3.0.17 (28 October
  2025, "addresses build problems on Windows and Mac OSX"). Last commit 18 May 2026 (build fixes).
  About 330 open issues. The Rust bindings (`libnotcurses-sys`) were last pushed December 2025. In
  maintenance mode, not abandoned.

## What it is

A C library for terminal graphics and TUIs, "definitely not curses": z-ordered planes with alpha
blending, true colour, Unicode, multimedia (images and video through FFmpeg or OpenImageIO), and
bitmap graphics through sixel, the kitty protocol, and the Linux framebuffer, with fallbacks to
quadrant, sextant and braille blitters.

## The problem it's solving

Pushing the terminal to its graphical limits from C, with a correct model of what each terminal can
do, at high frame rates and with multithreading.

## Its path / bet

The terminal as a compositor: planes stacked in z-order, each a cell surface that can carry pixels,
composited into a frame. Capability detection is thorough and done once.

## How it works

Applications create `ncplane`s, draw into them, and call `notcurses_render`, which composites the
pile, diffs against the last frame and writes. Visuals (images, video frames) blit into planes using
the best available method. Widgets (menus, readers, plots, reels, trees) are provided.

## Strengths

- The most complete treatment of terminal pixels and their fallbacks anywhere.
- Plane compositing with z-order is the right model for layers (callouts, overlays) on top of cells.
- Deep, documented knowledge of terminal quirks (its man pages and the author's book).

## Weaknesses / limits

- C, with a large native dependency tree (FFmpeg optional); hard to ship as a single static binary on
  macOS.
- Slowing activity and one maintainer.
- No data model, no agent story, no read-back beyond inspecting planes.

## Relation to fictty

**Watch / inspire.** It's the reference for how pixels and planes should behave in a terminal.

## Could fictty adopt it instead of building?

No. Replacing ratatui with Notcurses would mean C FFI through dated bindings, a heavier build and a
library whose maintenance is slowing, to gain pixel features fictty already covers for the terminals
that matter (kitty protocol in Ghostty, kitty, WezTerm).

## What fictty should take from it

- **The blitter fallback ladder** (pixels, then sextants or octants, then quadrants, then half
  blocks) for fictty's pixel primitives on terminals without the kitty protocol. ratatui 0.30's
  canvas added quadrant, sextant and octant markers, so much of this is now available in fictty's
  own stack.
- **Capability detection done once and kept in state,** so `get --state` can say what the terminal
  supports.

## Sources

- https://github.com/dankamongmen/notcurses (GitHub API and commits, 10 Oct 2026)
- https://github.com/dankamongmen/notcurses/releases/tag/v3.0.17
- https://github.com/dankamongmen/libnotcurses-sys
- https://notcurses.com
