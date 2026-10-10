# TUI frameworks fictty could build on or adopt: overview

As of 10 October 2026. One dossier per entity in this directory (`framework--<slug>.md`).

## The question

fictty is a runtime: a UI is one JSON value in a serializable state, agents push and patch it by id,
drive it, and read it back exactly, while data sources feed it without the model. It draws with
ratatui. The question for this cluster is whether any TUI framework should replace that foundation,
or whether some other one should replace fictty altogether.

## Who's in it

| entity | language | stars | latest | what it is | fictty relation |
|---|---|---|---|---|---|
| Bubble Tea v2 | Go | 45.4k | v2.1.0, 8 Oct 2026 | Elm-architecture framework, Cursed Renderer on Ultraviolet | inspire |
| Ink | TS/React | 40.1k | v8.0.0, 3 Oct 2026 | React renderer for CLIs, Yoga flexbox | inspire / complement |
| Textual | Python | 37.4k | 8.2.8, 30 Jun 2026 | DOM + CSS app framework, browser via textual-serve | inspire |
| ratatui | Rust | 22.9k | 0.30.2, 19 Jun 2026 | immediate-mode drawing library | build on (already) |
| OpenTUI | Zig + TS | 13.5k | v0.5.17, 8 Oct 2026 | native core, React/Solid, images, SSH, test renderer | watch; the only serious alternative substrate |
| Notcurses | C | 4.7k | v3.0.17, 28 Oct 2025 | planes, pixels, blitters | watch |
| iocraft | Rust | 1.6k | v0.9.1, 4 Sep 2026 | React-style Rust TUI on Taffy flexbox | watch |
| Ratzilla | Rust/WASM | 1.5k | v0.3.1, 8 Jun 2026 | ratatui backends for the browser | complement / adopt for a web surface |
| tui-realm | Rust | 1.0k | v4.1.0, 2 May 2026 | Elm/React components on ratatui | inspire |
| Raxol | Elixir | 80 | 2.7.0, 10 Sep 2026 | TEA module to terminal, LiveView, SSH and MCP | compete (weakly) / inspire |
| TUIKit | Markdown specs | 23 | none | specs an LLM compiles into Bubble Tea, Ink, OpenTUI, ratatui | inspire |

All the requested entities exist. Added in this pass: Ratzilla, tui-realm and iocraft (dossiers),
and, in this summary only, Charm's Ultraviolet (covered under Bubble Tea), Cursive (Rust, 4.9k),
FTXUI (C++, 10.8k), tview (Go, 14.1k) and newer declarative projects (go-tui, glyph, SvelTUI,
Symfony's Tui component, Melker). None of the latter changes the picture for fictty.

## The paths people are going down

1. **The Elm loop** (Bubble Tea, tui-realm, Raxol, and fictty). State, messages, a pure update, a
   view. Bubble Tea proved it at scale. Only Raxol and fictty go on to make the loop's state
   something an agent can address.
2. **React in the terminal** (Ink, OpenTUI's React and Solid bindings, iocraft, Claude Code's own
   renderer). Bet on the developer and model population that already writes JSX, plus flexbox.
3. **The web's architecture in cells** (Textual): a DOM, CSS, devtools, and then the browser as a
   second surface. Technically the most complete; commercially it didn't work, and it is now one
   maintainer.
4. **A native core under a scripting API** (OpenTUI's Zig, Bubble Tea v2's Ultraviolet, ratatui's
   modular core). The 2025–26 rewrites were all about the renderer: cell diffing, no flicker,
   synchronized output, kitty keyboard, pixels. This is where most effort went this year, because
   the coding agents needed it.
5. **Specs over code** (TUIKit): treat components as contracts and let an LLM regenerate code per
   framework. A small experiment, but the direction (behaviour and accessibility as the durable
   asset, code as disposable) is notable.
6. **One app, many surfaces** (Textual to browser, Raxol to LiveView/SSH/MCP, OpenTUI and Wish to
   SSH, Ratzilla to WASM).

## What's really being solved

Underneath the variety, these frameworks solve **how a developer writes a TUI app that ships**. The
year's work was making that output fast and correct enough for streaming coding agents (opencode,
Codex, Claude Code, Crush, Gemini CLI). Agents appear in two ways only: as *users* of the frameworks
(models write Bubble Tea and Ink well; OpenTUI ships an agent skill), and, newly, as *status
reporters* (Bubble Tea v2.1's OSC 7501).

None of them makes the running UI an addressable value. In all of them the UI is code, the state
lives in the language's heap (Go structs, React hooks, Python widgets, Zig buffers), and read-back
is a test helper (`teatest`, `lastFrame`, `captureCharFrame`, Pilot), not a live interface. Raxol is
the exception that proves the gap: it derives MCP tools from its component tree, but an agent still
can't put up a new screen without writing and compiling Elixir.

## The open debate

- **Code or data?** The frameworks' answer this year is "code, because models write code well" (and
  TUIKit pushes it furthest). fictty's answer is data, for screens that live minutes. Both can be
  right; they serve different lifetimes.
- **Flexbox or constraints?** Ink, OpenTUI, iocraft and Textual all use flexbox or CSS layout;
  ratatui uses constraints. Agents trained on the web expect flexbox.
- **Is the renderer the moat?** opencode and Claude Code each built their own renderer. Renderer
  quality matters, but it's commoditising fast: Bubble Tea v2, OpenTUI and ratatui are all good.
- **Accessibility.** Every cell-canvas framework is bad for screen readers. Ink has a basic linear
  mode; TUIKit puts roles in the spec; nobody has solved it.

## Verdict for fictty

- **Stay on ratatui.** It draws, it's fast, it's immediate mode (which matches fictty's pure `view`
  and "one source of truth" rule exactly), and its ecosystem gives a browser surface (Ratzilla) and
  effects (tachyonfx) on the same code.
- **The only alternative worth a second look is OpenTUI.** It would bring images, SSH, embedded
  terminals, code/diff/markdown primitives and a TypeScript home near A2UI and json-render, at the
  cost of a rewrite, a Bun/Node dependency and a pre-1.0 API driven by one product. Revisit only if
  fictty starts rebuilding what OpenTUI already has.
- **No framework replaces fictty.** Adopting any of them still leaves fictty's actual work undone:
  the UI as one serializable value, patch by id, live data sources, the socket and exact read-back.
  The threat is not from this cluster; it's from data-format renderers (json-render's Ink target, the
  a2ui crate) growing a live loop, and from HTML artifacts making the terminal unnecessary.

## What to take, in priority order

1. Emit **OSC 7501** (Bubble Tea v2.1 did; the spec is Superlogical's, for its Rex terminal, not
   TUIOS's as our first post said).
2. Generate **MCP tools from the UI value** (Raxol's idea, done with data).
3. A **library test API** shaped like OpenTUI's (`renderOnce`, `captureCharFrame`, mocked input) and
   **golden-file tests** like `teatest` and Textual's snapshots.
4. **Accessibility fields** on primitives from TUIKit's schema, and a linear screen-reader rendering
   (Ink has a crude one).
5. **Terminal modes declared in the UI value** (Bubble Tea v2's `tea.View`), and an **inline mode**
   (Ink) for short pickers.
6. **Keep `state` and `view` WASM-clean** so Ratzilla can give a browser surface later; consider
   **Taffy** if layout moves toward flexbox.
7. Ship an **agent skill** with the docs (OpenTUI does).

## Sources

See each dossier. Star counts and release dates are from the GitHub API on 10 October 2026.
