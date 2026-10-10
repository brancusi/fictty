# opencode and OpenTUI

- **Maker:** Anomaly (formerly SST; the repos moved from `sst/` to `anomalyco/`)
- **URL:** https://github.com/anomalyco/opencode, https://github.com/anomalyco/opentui,
  https://opentui.com
- **Status (2026-10-10):** opencode about 212,500 stars, MIT, v1.18.35 (6 October 2026), the
  most-starred coding agent on GitHub. OpenTUI about 13,500 stars, MIT, v0.5.17 (8 October 2026),
  pre-1.0, created July 2025. TUI plugins in opencode since 27 March 2026 (`tui plugins (#19347)`).

## What it is

opencode is an open-source, model-agnostic coding agent with a client/server split: a server
holds sessions and runs the agent; the TUI, a web UI and a desktop app are clients. OpenTUI is the
terminal UI library the same team built for it after leaving Go and Bubble Tea: a native core
written in Zig, with TypeScript bindings and React or Solid renderers (opencode uses Solid).

## The problem it's solving

Building a rich, fast, mouse-capable terminal app in TypeScript without Ink's limits: flexbox
layout (Yoga), scroll boxes, inputs and selects with keyboard and mouse, images, even 3D, at a
frame rate a React-in-Node renderer struggles to reach. And for opencode, letting third parties
extend that UI.

## Its path / bet

- **Native core, JS components.** The expensive part (buffers, diffing, layout via a Yoga
  bridge, terminal I/O) is Zig; the authoring model is JSX. "OpenCode uses OpenTUI in production
  for millions of users."
- **Breadth of output.** Packages for React, Solid, QR codes, Three.js via WebGPU, and an SSH
  server integration (`@opentui/ssh`) so an OpenTUI app can be served over SSH.
- **Host-defined slots for plugins.** OpenTUI has a slot system (append, replace,
  single-winner); opencode exposes named slots and routes to TUI plugins.
- **Code plugins.** opencode TUI plugins are Solid TSX modules listed in `tui.json`.

## How it works

- **opencode TUI plugin API** (`@opencode-ai/plugin/tui`): `api.route.register` (whole screens
  a plugin owns), `api.ui.Dialog*` and `api.ui.dialog` (modal stack), `api.slots.register`
  (host slots: `home_logo`, `home_prompt`, `session_prompt`, `sidebar_title`, `sidebar_content`,
  `sidebar_footer`, `app`, `app_bottom` and others), `api.keymap` (commands, bindings, modes),
  `api.attention.notify` (desktop notification and sound), `api.kv`, `api.theme`, `api.event.on`,
  `api.client` (the opencode SDK) and `api.renderer` (raw OpenTUI renderer).
- Plugins are code with full process access; there's no auto-discovery, only explicit config.
- **OpenTUI testing:** `createTestRenderer` and `captureCharFrame` render a component headless
  and return the frame as text. This is the read-back fictty claims, but as a test utility for
  app authors, not an agent-facing loop.
- **Graphics:** an `image` component and terminal capability detection that includes the kitty
  graphics protocol (release 0.5.13).
- **Agent skill:** OpenTUI publishes its docs as a skill (`npx skills add https://opentui.com
  --skill opentui`), so agents can write OpenTUI apps.

## Strengths

- The fastest-moving, most-used open harness, and its renderer is open and reusable.
- Real layout (Yoga flexbox), real widgets, mouse, images, SSH serving.
- A clean plugin vocabulary: routes, dialogs, slots, keymap layers, attention.
- Client/server architecture: other front ends can drive the same sessions.

## Weaknesses / limits

- **Code, not data.** An agent can't push a screen; someone writes and installs a Solid plugin.
- **Pre-1.0 renderer,** Bun-first (development needs Bun 1.4 and Zig 0.16), and a JS runtime in
  the loop.
- The agent can't read back what a plugin drew.
- Slot context exposes only the theme today; plugins integrate through the SDK, not through a
  shared data model.

## Relation to fictty

**Inspire,** with an edge of compete. OpenTUI is the strongest "build a TUI people install"
library in the TypeScript world, which fictty's scope already says it isn't. opencode's plugin
slots are where an opencode-specific dashboard would go, ahead of a fictty pane. But neither
gives an agent a screen it can write as data and read back.

## Could fictty adopt it instead of building?

As the renderer: no. fictty is a Rust binary on ratatui, and moving to a Zig core with a
TypeScript layer means a JS runtime, a different language and a pre-1.0 dependency, to gain
flexbox and widgets fictty can get from `taffy` and ratatui (plan phase 2). As a host: an
opencode TUI plugin that renders fictty UI values in a route or `sidebar_content` slot is
plausible, the same bridge as the Claude Code mod. Worth doing only after the Claude Code one.

## What fictty should take from it

- **Named slots.** fictty layers and callouts could use host-defined slots ("sidebar",
  "footer", "above prompt") so a pushed UI can target a region without knowing the layout.
- **Routes as screens.** A UI value with several named screens and `navigate` as an action is
  a simple, data-only way to do multi-screen flows.
- **Attention as a primitive.** `api.attention.notify` (title, message, sound, desktop
  notification) is the right shape for fictty's "waiting for you" signal, alongside TUIOS's OSC
  7501 and herdr's socket.
- **Ship the docs as a skill.** fictty's skill (plan phase 3) should be installable the same way.
- **SSH serving** is a reminder that "runs where the agent is" includes serving a screen to a
  person who connects, not only drawing in the agent's own terminal.

## Sources

- opencode: https://github.com/anomalyco/opencode
- TUI plugin spec: https://github.com/anomalyco/opencode/blob/dev/packages/opencode/specs/tui-plugins.md
  (read through the GitHub API on 2026-10-10)
- OpenTUI: https://github.com/anomalyco/opentui and https://opentui.com
- OpenTUI plugin slots: https://opentui.com/docs/plugins/slots/ and https://opentui.com/docs/plugins/solid/
- OpenTUI test renderer: `packages/core/src/testing/README.md` in the OpenTUI repo
- Third-party: https://azukiazusa.dev/en/blog/build-tui-with-opentui

Couldn't verify: OpenTUI's performance numbers (we found no published benchmark comparable to
fictty's), and how many opencode users run TUI plugins.
