# OpenTUI

- **Maker:** Anomaly (formerly SST; the team behind opencode). Repo moved from `sst/opentui` to
  `anomalyco/opentui`.
- **URL:** https://opentui.com, https://github.com/anomalyco/opentui
- **Status, 10 October 2026:** about 13,500 stars, MIT. v0.5.17 (8 October 2026); releases weekly or
  faster. Created July 2025. Pre-1.0. Runs on Bun (>= 1.3.14) and Node (>= 26.4). Development needs
  Zig 0.16.

## What it is

A terminal UI library with a native core in Zig and a TypeScript API, plus React and Solid
renderers. Flexbox layout through Yoga. Packages for keymaps, QR codes, a Three.js WebGPU renderer,
and SSH serving. It also has an embedded terminal component, image support and audio.

## The problem it's solving

opencode needed a renderer fast enough for a streaming coding agent with diffs, syntax highlighting
and markdown, in TypeScript, without Ink's flicker. The opencode team left Go and Bubble Tea and built
their own.

## Its path / bet

Native speed under a web developer's API: a Zig core does buffers, diffing, text and Unicode width,
image placement and terminal I/O; TypeScript drives it over FFI; React or Solid on top for those who
want components. It aims at feature parity with a browser-ish surface: mouse pointer styles, OSC 8
links, kitty graphics, 3D, embedded terminals, tree-sitter highlighting, a markdown renderer. It also
ships an "AI agent skill" with the docs so coding agents can write OpenTUI apps.

## How it works

`@opentui/core` exposes renderables (boxes, text, inputs, selects, scroll boxes, code, diff,
markdown) in a retained tree laid out by Yoga. The renderer draws into native buffers and emits the
diff. React and Solid reconcilers map JSX to renderables. Testing:
`createTestRenderer({width, height})` returns `mockInput`, `mockMouse`, `renderOnce`,
`captureCharFrame` (the frame as text), `resize`, plus capability mocks and a test recorder.

## Strengths

- Proven at very large scale by opencode (about 212,000 stars).
- Fast native core with a modern feature set: images, kitty z-ordering, mouse pointers, hyperlinks,
  embedded terminals, SSH.
- First-class headless testing with text frame capture and mocked input, close to fictty's
  `render --keys`.
- Shipping velocity is very high, and an agent skill is part of the docs.

## Weaknesses / limits

- Pre-1.0 and changing weekly; the API you build on will move.
- Requires Bun or a very recent Node plus a native library per platform. Not a single static binary
  in the way a Rust program is (Bun can compile one, but the native lib must be bundled).
- The UI is TypeScript/JSX code; state is in the JS heap or in React/Solid signals, not one
  serializable value.
- Its roadmap is set by opencode's needs.

## Relation to fictty

**Watch / inspire.** It is the strongest alternative substrate in this cluster, and it is a framework
for apps, not a runtime agents write data into.

## Could fictty adopt it instead of building?

This is the one worth taking seriously. A fictty on OpenTUI would get images, SSH, an embedded
terminal, mouse pointers, markdown and code views, and a test renderer, mostly for free; and json-
render, A2UI and MCP Apps all live in TypeScript, so adapters would be shorter. It would cost a rewrite
of the runtime in TypeScript, a Bun/Node dependency, the loss of caretline as a direct crate (it would
need FFI or a port), and dependence on a pre-1.0 API steered by one product. fictty's measured
performance on ratatui is already well above what it needs. Verdict: not now. Revisit if fictty's
primitives start re-implementing what OpenTUI already has (markdown, diff, code, embedded terminal)
or if the TypeScript adapters dominate the work.

## What fictty should take from it

- **The test renderer API** (`renderOnce`, `captureCharFrame`, `mockInput`) is a good shape for a
  fictty library API alongside the CLI.
- **Code, diff and markdown as primitives** with behaviour: these are what coding agents show most.
- **An agent skill shipped with the docs.** fictty should ship one too: "how to write a fictty
  screen", versioned with releases.
- **Embedded terminal** as a component is worth considering: an agent's screen that includes the
  live output of a command.

## Sources

- https://github.com/anomalyco/opentui (README, packages, engines in `packages/core/package.json`; GitHub API 10 Oct 2026)
- https://github.com/anomalyco/opentui/releases (v0.5.8 to v0.5.17, Aug to Oct 2026)
- https://github.com/anomalyco/opentui/blob/main/packages/core/src/testing/README.md
- https://opentui.com, https://opentui.com/docs (Yoga layout, testing section, embedded terminal)
- Not verified: performance numbers against ratatui; how complete Node support is in practice.
