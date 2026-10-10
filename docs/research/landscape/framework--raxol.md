# Raxol

- **Maker:** DROOdotFOO (axol.io), essentially a solo developer.
- **URL:** https://github.com/DROOdotFOO/raxol, https://raxol.io
- **Status, 10 October 2026:** about 80 stars, MIT. Hex package `raxol` 2.7.0 (10 September 2026),
  about 2,900 all-time downloads. Pushed daily. Created March 2025.
- **Correction to our first pass:** the landscape post called Raxol "the project that took the same
  bet as us most seriously". The architecture is close; the adoption is tiny.

## What it is

An Elixir/OTP framework where an app is one Elm-architecture module (`init`, `update`, `view`) run as
a GenServer, rendered to four surfaces: a terminal (termbox2 NIF), a browser (Phoenix LiveView), SSH
(Erlang `:ssh`) and agents (MCP tools derived from the component tree). Around it the repo has grown
a coding agent, an agent harness with a durable event journal, payments, chat gateways and an
OpenAI Symphony port.

## The problem it's solving

"Write one app, render it everywhere, including to an agent": a UI whose structure an LLM can operate
directly instead of scraping pixels, with BEAM properties (crash isolation, hot reload, clustering).

## Its path / bet

Code-first, runtime-heavy. The developer writes Elixir; the runtime projects the component tree into
both a picture and a tool surface. Every interactive component exposes MCP tools (`click`,
`type_into`, `get_value`), with a "focus lens" that narrows to about 15 relevant tools.

## How it works

TEA module in a GenServer; `view` returns a component tree; a renderer diffs and paints per surface.
The MCP server (`mix mcp.server`) exposes tools generated from the live tree, and a test DSL drives
sessions (`type_into`, `click`, `assert_component`). It reports a full frame in 5.0 ms on an M1.
Sessions journal events, so the agent product supports replay and rewind.

## Strengths

- The idea of deriving the agent's tool surface from the UI tree is strong, and nobody else in this
  cluster does it.
- Multi-surface from one source (terminal, LiveView, SSH) is real.
- Event-journal replay and rewind exist in its harness, which fictty only plans.

## Weaknesses / limits

- Tiny adoption and a single maintainer, with scope sprawling into payments, crypto settlement and
  chat platforms. Hard to depend on.
- Elixir/BEAM is a niche runtime for terminal tools; the termbox2 NIF needs a C toolchain.
- The UI is Elixir code; an agent can operate a Raxol app but cannot put up a new screen without
  writing and compiling a module.
- 5 ms frames are fine but slower than ratatui-based fictty frames.

## Relation to fictty

**Compete (weakly) / inspire.** Same shape of loop and the same claim (agents read structure, not
pixels), from the code-first side.

## Could fictty adopt it instead of building?

No. Different language and runtime, code-first, tiny community, and its maintainer's attention is
spread across a dozen products.

## What fictty should take from it

- **MCP tools derived from the UI value.** fictty knows every node's kind and id, so it can expose
  `click(id)`, `select(id, row)`, `type(id, text)` and `read(id)` automatically, filtered to the
  focused region. That is the cleanest form of the planned MCP server.
- **A test DSL in agent verbs** (`type_into`, `click`, `assert_component`) over `render`.
- **Journal-based replay** as the implementation of time travel.

## Sources

- https://github.com/DROOdotFOO/raxol (README, GitHub API 10 Oct 2026)
- https://hex.pm/packages/raxol (2.7.0, download counts)
- https://raxol.io
- Not verified: the MCP focus-lens behaviour in practice; any production users.
