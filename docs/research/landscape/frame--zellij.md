# zellij (and its plugin system)

- **Maker:** Aram Drevekenin (`imsnif`) and the zellij-org community
- **URL:** https://zellij.dev, https://github.com/zellij-org/zellij
- **Status (10 October 2026):** v0.45.1 (28 August 2026); v0.45.0 on 20 August, v0.44.0 on
  23 March 2026. Rust, MIT, 35,690 stars, repository since September 2020. Mature, actively
  maintained, still pre-1.0 (the `zellij-tile` plugin crate has had 32 breaking releases).

## What it is

"A terminal workspace with batteries included": a multiplexer with layouts, floating and stacked
panes, sessions, a built-in web client, multiplayer, and a WebAssembly plugin system in which
zellij's own status bar, tab bar, session manager and layout manager are written.

## The problem it's solving

tmux's power without tmux's learning curve, plus a way to extend the workspace with UI (plugins)
rather than only scripts. It wasn't built for agents; it has become agent-usable through its CLI.

## Its path / bet

Make the workspace programmable at two levels: a CLI that can do everything (`zellij action`,
`zellij run`, `zellij subscribe`), and WASM plugins that are "first-class workspace citizens"
with their own UI. Reach beyond the terminal through the browser (web client, PWA, read-only
sharing tokens).

## How it works

- **CLI automation (0.44).** `zellij run --blocking`, `zellij action list-panes --json`,
  `zellij action send-keys "Enter" "Ctrl c"`, `zellij action dump-screen --pane-id`, and
  `zellij subscribe` to stream a pane's viewport and scrollback as JSON (optionally with ANSI).
  Commands return pane ids. An agent can drive and read any pane.
- **Plugins.** WASM/WASI; Rust is the only officially supported language (`zellij-tile`). A plugin
  receives events, issues commands (with permissions), and renders by printing UTF-8 ANSI to
  stdout in a `render` call; the previous frame is cleared each time.
- **UI components.** Zellij has four built-in components (Table, Ribbon, Nested List, Text) that
  follow the user's theme. They're serialized as a private DCS sequence: `ESC P z`, a clear-text
  component name, semicolon-separated items with optional coordinates, selection and colour
  indices, then `ESC \`. The SDK wrappers write it for you.
- **Web.** Sessions can be shared over HTTPS, attached from another terminal
  (`zellij attach https://...`), and issued read-only tokens "for teaching, demonstrations,
  screencasting". 0.45 added a touch interface for mobile browsers and PWA install.

## Strengths

- The only multiplexer with a real UI extension model, and the only one whose host draws theme-
  aware components on behalf of plugins.
- `subscribe` with JSON gives agents a live, structured stream of any pane.
- Read-only web sharing is a ready-made "show someone a terminal screen" path.
- Mature, multi-platform (native Windows since 0.44), well-known.

## Weaknesses / limits

- Plugins are code: compile Rust to WASM, request permissions, load. Not something an agent writes
  in the middle of a task.
- The component protocol is private, zellij-only, and four components deep.
- Graphics: sixel only, no kitty graphics protocol.
- Not agent-focused; no agent status model of its own (third-party plugins exist; we didn't
  survey them).

## Relation to fictty

**Complement, with an idea worth studying.** Zellij's DCS component protocol is the nearest thing
in a multiplexer to "the host draws components from a description": a plugin prints *what* (a
table with these cells, this one selected) and zellij draws *how*. That is a small cousin of
fictty's model, but as an output format inside code rather than a value an agent pushes and
reads back. A fictty screen in a zellij floating pane works today; pixels won't.

## Could fictty adopt it instead of building?

Partly tempting, ultimately no. One could write a single zellij plugin that reads a fictty-style
JSON value and draws it with zellij's components, inheriting sessions, web sharing and theming.
But it would be zellij-only, capped at four components, without pixels, bound to a pre-1.0 plugin
API with frequent breaking changes, and the read-back would be zellij's text dump rather than
state. fictty's value is that it runs in any terminal and any frame.

## What fictty should take from it

- **Read-only sharing for teaching.** Zellij's read-only web tokens are an explicit teaching and
  demo feature. A fictty screen shown through a zellij web session is a zero-cost way to show a
  live screen to a group; worth a guide entry.
- **`subscribe` as JSON with pane ids** is a good model for `fictty watch`.
- **Host-drawn, theme-aware components** validate fictty's split: the program says what, the
  runtime draws how.
- Test fictty under zellij (no kitty graphics: make sure the cell fallback is clean).

## Sources

- https://github.com/zellij-org/zellij (releases v0.44.0, v0.45.0, v0.45.1), read 10 October 2026
- https://zellij.dev/documentation/plugins.html
- https://zellij.dev/documentation/plugin-ui-rendering.html
- https://newreleases.io/project/github/zellij-org/zellij/release/v0.45.0
- https://dev.to/imsnif/zellij-043-brings-the-terminal-to-your-browser-57lg
