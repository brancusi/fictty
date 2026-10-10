# Ink

- **Maker:** Vadim Demedes (with Sindre Sorhus among maintainers), open source.
- **URL:** https://github.com/vadimdemedes/ink
- **Status, 10 October 2026:** about 40,100 stars, MIT. v8.0.0 released 3 October 2026 (requires
  React 19.3+, Node 22+); v7.0.0 was 8 April 2026, v6.0.0 29 May 2025. Active.
- **Notable users (per its README):** Claude Code, Gemini CLI, GitHub Copilot CLI, Wrangler, Shopify
  CLI, Prisma, Gatsby. Claude Code keeps React but replaced Ink's renderer with its own (see the
  Steinberger post); the README still lists it.

## What it is

A React renderer for the terminal. You write components with `<Box>` and `<Text>`, lay them out with
flexbox (Yoga), handle input with hooks (`useInput`, `useFocus`), and Ink reconciles the tree and
writes it to stdout.

## The problem it's solving

Letting the very large population of JavaScript and React developers build interactive CLIs with the
model they already know.

## Its path / bet

React everywhere. Components, hooks, Suspense and concurrent rendering, applied to a terminal. Ink
stays a renderer and leaves the ecosystem (ink-text-input, ink-select-input, Pastel) to others. v8
leans on output quality: incremental rendering that skips unchanged line prefixes, many scrollback,
`<Static>` and wide-character fixes, and scroll offsets on `<Box>`.

## How it works

React's reconciler builds a host tree of Ink nodes; Yoga computes a flexbox layout; Ink paints the
tree into an output buffer of lines and writes it. Historically it rewrote the whole frame (the
source of the flicker Claude Code fought); `incrementalRendering` now writes only changed parts of
lines. `<Static>` holds content that scrolls into terminal history and is never redrawn. Testing is
`ink-testing-library`: `render(<App/>)` returns `lastFrame()`, the frame as a string. There is basic
screen reader support (`isScreenReaderEnabled`, a subset of ARIA roles and labels) that renders a
linear text form instead of the visual layout.

## Strengths

- The biggest talent pool, and models write React fluently, so agents write Ink well.
- Flexbox layout through Yoga is what web-trained models expect.
- Inline (non-alt-screen) apps that coexist with scrollback are a first-class use, which is why chat
  CLIs chose it.
- It has the only built-in screen-reader mode among the big frameworks, small as it is.
- It is a renderer target for json-render (Vercel Labs), so "data in, Ink screen out" already exists.

## Weaknesses / limits

- Line-based painting, not a cell grid with a proper diff; performance under heavy streaming has
  been its weak point, and its most famous user replaced the renderer.
- No pixel graphics in core.
- The UI is JSX code plus a Node process; an agent has to write and run a program. The state lives in
  React hooks, so it isn't serializable or replayable.
- Read-back is a test helper (`lastFrame`), not a live interface to a running app.

## Relation to fictty

**Inspire / complement.** Ink is how JS developers write CLIs. Through json-render's Ink renderer it
also becomes part of the "agent writes data, terminal renders it" cell that fictty sits in, which is
the closer competition (that belongs in the format cluster's dossier on json-render).

## Could fictty adopt it instead of building?

Building fictty as a React app on Ink would give flexbox, React's ecosystem and a lot of developers.
It would cost the Rust core (caretline, resvg, the kitty graphics code), take on a Node runtime and
Ink's line-based renderer, and put the UI value inside React state, where fictty's exact read-back
and replay would have to be rebuilt against the reconciler. Ink's renderer is the weakest of the
candidates for a 500-updates-a-second ticker. No.

## What fictty should take from it

- **A screen-reader rendering of the same value.** Ink's `isScreenReaderEnabled` mode is a crude
  version of what our landscape post proposed: a linear text form of the UI for screen readers and
  small-context agents. fictty can do it better because it knows every node's role.
- **Inline mode.** Ink apps render below the prompt and leave history in scrollback (`<Static>`).
  fictty is alt-screen only; an inline mode would suit short pickers and confirmations an agent
  shows mid-session.
- **Flexbox vocabulary** for row/column sizing, since that's what agents will write.

## Sources

- https://github.com/vadimdemedes/ink (README, users, Testing and Screen Reader sections; stars via GitHub API, 10 Oct 2026)
- https://github.com/vadimdemedes/ink/releases/tag/v8.0.0 (3 Oct 2026)
- https://github.com/vadimdemedes/ink-testing-library
- https://steipete.me/posts/2025/signature-flicker (Claude Code's renderer)
- https://github.com/vercel-labs/json-render (Ink renderer)
- Not verified: whether Gemini CLI still runs stock Ink or a fork today.
