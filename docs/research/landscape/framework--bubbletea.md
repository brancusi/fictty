# Bubble Tea (v2)

- **Maker:** Charm (charmbracelet), a venture-funded company in New York.
- **URL:** https://github.com/charmbracelet/bubbletea, https://charm.land
- **Status, 10 October 2026:** about 45,400 stars, MIT. v2.0.0 shipped 24 February 2026 after
  alphas from September 2024; v2.1.0 on 8 October 2026 added OSC 7501 (program status). Import path
  is now `charm.land/bubbletea/v2`. Very active.

## What it is

A Go framework for terminal apps built on the Elm architecture: a `Model` with `Init`, `Update(msg)`
returning a new model and a `Cmd`, and `View()`. Around it sit Lip Gloss (styling and layout),
Bubbles (components: list, table, textinput, viewport, spinner), Wish (serve apps over SSH) and, in
v2, Ultraviolet (the low-level cell renderer and input layer).

## The problem it's solving

Making terminal apps pleasant and correct to write in Go, at a quality bar high enough to ship to
millions (Charm's own Crush agent, gh-dash, Glow, TUIOS and many more).

## Its path / bet

The Elm loop as the programming model, with Charm owning the whole stack beneath it. v2's bets:

- **The Cursed Renderer**, built on Ultraviolet: a cell-based diffing renderer "based on the ncurses
  rendering algorithm" with no terminfo database. Charm says it cut Wish bandwidth "by orders of
  magnitude".
- **Declarative views.** `View()` returns a `tea.View` struct whose fields declare alt-screen, mouse
  mode, keyboard enhancements, window title, cursor and so on, replacing imperative commands.
- **Modern input:** kitty progressive keyboard enhancements, key release events, native clipboard.
- **Agents as a first-class use:** v2.1.0 adds OSC 7501 so programs report idle, working, blocked
  and done to the terminal.

## How it works

`Update` is a pure-ish function of model and message returning a command (a function the runtime
runs asynchronously, whose result comes back as a message). `View` returns a string (v1) or a
`tea.View` holding a string or layers of cells (v2). The runtime diffs the cell result and writes the
minimum. Testing uses `teatest` (in `charmbracelet/x/exp`), which drives a program with messages and
compares final output against golden files.

## Strengths

- The Elm loop is exactly fictty's architecture, proven at scale in another language.
- A mature, coherent ecosystem and the largest community of any TUI framework.
- v2's renderer and input handling are state of the art; Wish makes SSH serving trivial.
- Charm moves fast on agent-era terminal features (OSC 7501 within days of the spec's first draft).

## Weaknesses / limits

- The UI is Go code. An agent has to write, compile and run a program to put up a screen. That's
  the opposite of fictty's "data, never code".
- `View()` returns styled strings, so the runtime knows nothing about what a cell means: no node
  ids, no semantic read-back beyond the rendered text. Read-back is test-time (`teatest`), not a
  live socket.
- Model state is any Go struct; nothing makes it serializable, so replay and time-travel are up to
  each app.
- Like every cell-canvas framework, poor for screen readers (Casey Reeves's critique names it).

## Relation to fictty

**Inspire.** Same architecture, different layer: Bubble Tea is how you write a TUI app; fictty is a
running screen agents write as data.

## Could fictty adopt it instead of building?

Only by rewriting fictty in Go. A fictty-in-Go on Bubble Tea v2 would get the Cursed Renderer, Wish
(SSH for free) and modern input. It would lose caretline (a Rust crate), resvg, and the existing
3,900 lines of Rust, and fictty would still have to build everything that makes it fictty (the UI
value, sources, the socket, read-back, focus by id). Nothing on the drawing side is better enough
than ratatui to justify that. No.

## What fictty should take from it

- **Emit OSC 7501.** Bubble Tea shipped it on 8 October 2026. fictty should report `blocked` when a
  screen waits for the person and `working` while a source runs. Note: the protocol is Superlogical's
  (for its Rex terminal; draft 0.1 dated 28 September 2026), not TUIOS's as our earlier post said.
- **Declarative terminal modes.** v2 moved alt-screen, mouse and keyboard modes into the view value.
  fictty's UI value should carry these the same way rather than as runtime flags.
- **Wish's model** for SSH: a screen served to a remote person over SSH is a natural fictty mode.
- **Golden-file tests** in the style of `teatest` for `fictty render`.

## Sources

- https://github.com/charmbracelet/bubbletea (stars, licence; GitHub API, 10 Oct 2026)
- https://github.com/charmbracelet/bubbletea/releases/tag/v2.0.0 (24 Feb 2026 notes: Cursed Renderer, declarative views, keyboard enhancements)
- https://github.com/charmbracelet/bubbletea/releases/tag/v2.1.0 (8 Oct 2026, OSC 7501)
- https://github.com/charmbracelet/ultraviolet
- https://www.superlogical.com/rex/docs/build/program-status (OSC 7501 spec, revisions 28 Sep to 7 Oct 2026)
- https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility
- Not verified: Charm's funding status today; exact teatest API stability (it lives in `x/exp`).
