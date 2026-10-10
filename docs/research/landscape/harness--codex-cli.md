# Codex CLI

- **Maker:** OpenAI
- **URL:** https://github.com/openai/codex (the Rust workspace is `codex-rs`)
- **Status (2026-10-10):** about 128,500 stars, Apache-2.0. Latest stable `rust-v0.162.1`
  (9 October 2026), alphas of 0.163 the next day. Created April 2025; rewritten from TypeScript to
  Rust during 2025. Plugins since v0.117 (March 2026, per a third-party guide), hooks general
  availability in May 2026 (same caveat). OpenAI also ships the Codex desktop app (February 2026)
  and an IDE extension; all of them talk to the same `app-server` (JSON-RPC).

## What it is

OpenAI's coding agent for the terminal. The TUI (`codex-rs/tui`) is a ratatui + crossterm
program, now a client of Codex's own app-server, the same protocol the desktop app and IDE use.

## The problem it's solving

A fast, dependency-free coding agent (one static binary, no Node) with a rich chat surface, that
can also run headless, in CI and behind other front ends.

## Its path / bet

- **Native Rust on ratatui.** The most direct sibling of fictty's stack: ratatui widgets, a
  custom terminal backend, cell diffing, `vt100` for parsing child output.
- **Inline, not fullscreen.** The TUI keeps a live viewport at the bottom and inserts finished
  history into the terminal's real scrollback (`insert_history.rs`), so native scroll, search and
  selection keep working. That's the opposite choice from Claude Code's and Gemini CLI's
  alternate-screen modes.
- **Front ends over a protocol.** Since 2026 the TUI is one client of `app-server`; the desktop
  app is another. The interesting UI goes to the desktop app.
- **Visuals as HTML, shown elsewhere.** Codex has an "inline visualization" directive
  (`::codex-inline-vis{…}`) the model writes into a reply, pointing at an HTML fragment. The TUI
  code calls itself the "terminal fallback": it materialises a sandboxed HTML viewer (strict CSP,
  `iframe sandbox="allow-scripts"`) and shows a link, "Open … visualization in the browser". We
  infer another surface, most likely the desktop app, renders these inline; we couldn't confirm
  it from a primary doc. OpenAI's "Visualizations" help page (for ChatGPT) says the Codex CLI and
  IDE extension don't render them.
- **One native diagram language.** A standalone crate, `codex-mermaid`, renders a bounded subset
  of Mermaid (flowchart, sequence, state, class, ER) as Unicode text inside the TUI's Markdown,
  with no Mermaid runtime. It refuses unsupported syntax rather than drawing a partial diagram.
  Actively developed in late September 2026.

## How it works

- `codex-rs/tui/src` holds the chat widget, bottom pane, diff renderer, pager overlay, Markdown
  streaming (incremental since July 2026), Mermaid, a screen-reader mode, keymaps, themes (32
  bundled per a third-party guide), status lines and an animated empty state.
- Extension points are plugins (skills, MCP servers, hooks, browser extensions per OpenAI's
  plugin docs) and hooks (lifecycle scripts). OpenAI's plugin page says MCP tools "can optionally
  include custom UI", but nothing we found says the **TUI** draws a plugin's UI. The CLI has a
  `/plugins` browser of its own.
- "Dynamic tools" exist in the protocol: a TUI connected to an external app server can host
  tools (thread management) for the agent. They're tools, not UI.

## Strengths

- Same language and drawing library as fictty; its source is a large, Apache-licensed body of
  production ratatui practice (inline viewport, history insertion, Markdown streaming, snapshot
  tests with `insta`).
- Respects the terminal: native scrollback and selection survive.
- `codex-mermaid` is exactly the kind of bounded, data-in, text-out renderer fictty wants for
  diagrams, under a licence fictty can vendor.
- A protocol-first architecture (app-server) means other front ends are first-class.

## Weaknesses / limits

- No way for an agent or a plugin to put a custom interactive UI inside the TUI. The visual
  story is "write HTML, open a browser", or the desktop app.
- The model can't read back anything visual beyond its own text.
- Inline mode limits what the TUI itself can do (no side panes, no overlays over history).

## Relation to fictty

**Complement.** A Codex user who wants a live screen has nowhere inside Codex to put it, so a
fictty screen in a neighbouring pane is a real fit, driven by Codex through the `fictty` CLI.
Codex is the clearest evidence that one major lab has put its rich-UI bet in a desktop app and
HTML, leaving the terminal to text and Mermaid.

## Could fictty adopt it instead of building?

No: Codex has no UI runtime to adopt. Two pieces are worth borrowing: `codex-mermaid` (Apache-2.0,
vendorable with attribution; we didn't find it published on crates.io) and patterns from the TUI
(inline viewport plus scrollback insertion, streaming Markdown).

## What fictty should take from it

- **Mermaid as a fictty component.** Agents already write Mermaid fluently. A `diagram` node that
  takes Mermaid text and renders it with `codex-mermaid` (cells everywhere, SVG pixels where the
  terminal has them) is cheap and fits "data, never code".
- **Refuse, don't half-draw.** codex-mermaid returns an error rather than a partial diagram;
  fictty's validated batches (plan phase 3) should take the same line.
- **An inline mode.** Codex shows that keeping native scrollback matters to people. A fictty
  screen that can run inline (a fixed-height region, history above it) would fit inside agents'
  own flows better than always taking the alternate screen.
- **The fallback link.** When a node can't render here (pixels over SSH without kitty), offer
  what Codex offers: a file or URL to open, not silence.

## Sources

- Repository and releases: https://github.com/openai/codex
- TUI source: https://github.com/openai/codex/tree/main/codex-rs/tui
- Inline visualization fallback: https://github.com/openai/codex/blob/main/codex-rs/tui/src/inline_visualization.rs
- codex-mermaid: https://github.com/openai/codex/tree/main/codex-rs/mermaid
- app-server: https://github.com/openai/codex/tree/main/codex-rs/app-server
- Plugins doc (redirects from developers.openai.com/codex/plugins): https://learn.chatgpt.com/docs/plugins
- Visualizations doc: https://learn.chatgpt.com/docs/visualizations
- Introducing the Codex app: https://openai.com/index/introducing-the-codex-app/
- Third-party guides (dates and counts, unverified): https://codex.danielvaughan.com/2026/05/04/codex-cli-plugin-ecosystem-building-distributing-marketplace-plugins/ and https://codex.danielvaughan.com/2026/05/05/codex-cli-tui-customisation-keymaps-themes-status-lines/

Couldn't verify: which surface renders `codex-inline-vis` inline, whether the Codex desktop app
renders MCP Apps, and the plugin and hook version dates (third-party only).
