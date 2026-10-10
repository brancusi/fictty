# Textual (and textual-serve, Textual Web)

- **Maker:** Will McGugan (author of Rich). Textualize Ltd, the company, wound down in mid-2025;
  Textual continues as open source maintained by McGugan.
- **URL:** https://github.com/Textualize/textual, https://textual.textualize.io
- **Status, 10 October 2026:** about 37,400 stars, MIT. Latest release 8.2.8 (30 June 2026). Commits
  since then are mostly typo fixes and, on 9 October 2026, a maintainer note in the bug template:
  "it takes a lot of my time to review issues and this has only increased in the AI age. I can't
  always allocate the time." About 360 open issues and PRs.
  - **textual-serve:** 407 stars, v1.1.3 on PyPI; last push November 2025. Works, rarely changes.
  - **Textual Web:** 1,470 stars; last push August 2024. Effectively dormant.
  - McGugan's newer project is **Toad** (batrachianai/toad, about 3,500 stars, AGPL-3.0), "a unified
    interface for AI in your terminal", built on Textual.

## What it is

A Python application framework for the terminal, modelled on the web: a retained DOM of widgets,
styled with a CSS dialect (TCSS), with reactive attributes, message passing, async workers and a
large widget library (DataTable, Tree, TextArea, Markdown with streaming, tabs, inputs). The same app
runs in a browser through `textual serve`.

## The problem it's solving

Making rich, app-quality terminal UIs as easy for Python developers as web UIs, and closing the gap
between terminal and web by running the same app in both.

## Its path / bet

The web's architecture in the terminal: a DOM, CSS, events that bubble, a devtools console. Then
"every Textual app is a web app": serve the same process to a browser over a websocket, rendering the
cell stream with xterm.js. The commercial bet (Textual Cloud, Textual Web as a hosting tunnel) didn't
find a business; the framework survived.

## How it works

Widgets form a DOM; TCSS rules apply styles and layout (vertical, horizontal, grid, docking); a
compositor renders widgets into strips of Rich segments and updates only changed regions. Testing is
strong: `App.run_test()` returns a `Pilot` that presses keys, clicks and waits headlessly, and
`pytest-textual-snapshot` compares SVG screenshots of the screen. `textual serve` runs the app as a
subprocess per browser session and streams its terminal output to xterm.js in the page.

## Strengths

- The richest widget set and the most complete app model of any TUI framework.
- CSS for terminal layout and theming: the closest thing to what web-trained agents already write.
- A real browser path today, with no extra code.
- Headless testing with Pilot plus snapshot tests is the best test story in the cluster.
- `Markdown` streaming and `TextArea` are well suited to LLM output, which is why Toad uses it.

## Weaknesses / limits

- Python: startup time and per-frame cost are higher than Rust, Go or Zig; Python must be installed.
- Maintenance is one person with limited time, by his own account. Bus factor is a real concern for
  anything built on it.
- The UI is Python classes; state lives in widget objects, so it isn't one serializable value.
- No kitty graphics in core (third-party `textual-image` exists).
- textual-serve ships the terminal byte stream to xterm.js; the browser gets pixels of a terminal,
  not semantic HTML.

## Relation to fictty

**Inspire.** Textual is the most complete argument that terminal UI can feel like an app and can
cross into the browser. It is not trying to be a screen agents write as data.

## Could fictty adopt it instead of building?

A fictty in Python on Textual would get a DOM, CSS, a widget library and a browser surface. It would
cost speed (fictty's stress numbers would not survive), the Rust core and caretline, and a single
binary that works over SSH with nothing installed. The maintenance outlook makes it a poor foundation
for a new project in October 2026. No.

## What fictty should take from it

- **Theme as CSS-like tokens.** TCSS shows how far a stylesheet over a widget tree can go in cells.
  fictty's theme layer should be at least as expressive, but as data.
- **Pilot and snapshot tests** are the model for fictty's `render --keys` plus golden files.
- **The browser path.** Serving the same screen to a browser is valuable; but fictty can do better
  than streaming terminal bytes, since its UI value could render to semantic HTML or to Ratzilla.
- **Streaming Markdown** as a primitive: agents emit Markdown constantly.

## Sources

- https://github.com/Textualize/textual (stars, commits, bug-template note; GitHub API 10 Oct 2026)
- https://pypi.org/project/textual/ (8.2.8, 30 Jun 2026)
- https://textual.textualize.io/blog/2025/05/07/the-future-of-textualize/ (company wind-down, 7 May 2025)
- https://github.com/Textualize/textual-serve, https://github.com/Textualize/textual-web
- https://github.com/batrachianai/toad
- https://textual.textualize.io/guide/testing/ (Pilot, snapshot testing; from prior knowledge, not re-fetched)
