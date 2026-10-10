# tui-test (Microsoft)

- **Maker:** Microsoft
- **URL:** https://github.com/microsoft/tui-test
- **Status (2026-10-10):** 280 stars, MIT. Repo dates from January 2024 as a TypeScript,
  Playwright-style terminal test framework (`@microsoft/tui-test`, about 154,000 npm downloads in
  the last month, most of them presumably the older versions). On 3 October 2026 it shipped 0.1.0
  of a Rust rewrite with a CLI, Homebrew tap, and Rust, Python, JavaScript and Go bindings. The
  repo description now reads "makes the terminal fully accessible to AI".

## What it is

A controller for real shell sessions and full-screen terminal apps on Windows, Linux and macOS,
for tests and for agents: run a program, locate text, click it, press keys, assert, read cells and
styles, record, and take screenshots.

## The problem it's solving

Testing terminal apps the way Playwright tests web pages, and giving agents the same structured
access.

## Its path / bet

Playwright's model for terminals: locators (`get_by_text`, regex, anchors, style filters, OSC 8
link targets), auto-retrying `expect`, `click text`, sessions that persist across CLI calls. One
Rust engine, four language bindings, an agent skill (`tui-test skill --add`) and a JSON command
schema (`agent-context`).

## How it works

- Sessions via a daemon; `run`, `open`, `restart`, `close`.
- Read state: `text`, `cells X Y W H` with styles, cursor, title, clipboard, bells, last command,
  exit code, cwd.
- Waits on idle, title, clipboard, command, exit, prompt, bell.
- Full keyboard (down, repeat, up) and mouse (click, drag, scroll) input; failure artifacts as
  text, HTML and casts; screenshots as SVG, recordings as APNG, GIF or MP4.

## Strengths

- Microsoft backing, cross-platform including Windows, mature test ergonomics.
- The most complete read-state surface of any driver (cells with styles, not just text).
- Ships agent-facing docs and a machine-readable command schema.

## Weaknesses / limits

- Still a driver: it knows text and styles, not meaning. Locators are text-based.
- The Rust rewrite is days old; API churn likely.

## Relation to fictty

Complement. It's the best tool we found for testing fictty in real terminal conditions, and the
likely default that agents will reach for to "use" any TUI, fictty included.

## Could fictty adopt it instead of building?

For the screen, no. For fictty's own end-to-end tests, yes: `tui-test-rs` could run `fictty run`
in a real PTY and check that what `fictty screen` reports matches what the emulator shows, which
is the one thing `fictty render` can't prove (escape sequences, kitty graphics, resize).

## What fictty should take from it

- Adopt it (dev-dependency or CI script) for an emulator-level conformance test: `screen` equals
  the emulator's text, cell for cell.
- Ship `fictty agent-context`: the whole command surface as JSON, as tui-test does.
- Locator-style addressing in fictty's CLI (`fictty click --text "Continue"` alongside ids) costs
  little and matches what agents learn from Playwright.

## Sources

- https://github.com/microsoft/tui-test (README)
- https://api.github.com/repos/microsoft/tui-test/releases/latest
- https://registry.npmjs.org/@microsoft/tui-test
- https://docs.rs/tui-test-rs/latest/tui_test/
