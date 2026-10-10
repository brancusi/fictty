# Raxol

- **Maker:** DROO (GitHub `DROOdotFOO`), part of axol.io
- **URL:** https://github.com/DROOdotFOO/raxol, https://raxol.io, https://hex.pm/packages/raxol
- **Status (2026-10-10):** about 80 stars, MIT, Elixir. Repo created March 2025, pushed today.
  Hex 2.7.0 (10 September 2026), about 2,900 total downloads. All feature claims below come from
  the project's own README and docs; we found no independent review. The framework cluster has
  its own dossier ([framework--raxol.md](framework--raxol.md)); this one looks only at Raxol as an
  agent-facing terminal screen.

## What it is

"Write one app. Render it to a terminal, a browser, an SSH session, or an agent." An app is one
Elm-architecture module (`init`, `update`, `view`) running as an OTP GenServer, rendered to a
terminal (termbox2 NIF), Phoenix LiveView, Erlang SSH, and MCP tools. It has since grown a coding
agent (`mix raxol.code`), an agent harness with a durable event journal, payments, chat gateways
and an orchestrator.

## The problem it's solving

Agents scrape screens; Raxol wants them to work against the component tree. Every interactive
component automatically exposes MCP tools (Button gives `click`; TextInput gives `type_into`,
`clear`, `get_value`), and a "focus lens" narrows the tool list to about 15 per interaction.

## Its path / bet

Code-first and BEAM-first: the developer writes the app in Elixir; the runtime projects it to
every surface, the agent being one of them. Crash isolation, hot reload and clustering come from
OTP.

## How it works

- TEA module in a GenServer; components carry their own agent tools.
- `mix mcp.server` serves those tools over stdio to Claude Code; `Raxol.MCP.Test` drives an app
  headlessly (`start_session |> type_into |> click |> assert_component`).
- Claims a full frame in 5 ms on an M1; a render-determinism golden suite.

## Strengths

- The most serious attempt at "the agent reads the UI as structure, not pixels" in a terminal
  framework; same Elm architecture as fictty.
- Replay and journals in the harness; multiple surfaces from one source.

## Weaknesses / limits

- An agent can't write a Raxol screen on the fly without writing and compiling Elixir. It's a
  framework for apps developers build, not a canvas for agents.
- Scope has sprawled (coding agent, on-chain payments, chat gateways); the UI runtime is one part
  of many.
- Small adoption; claims unverified.

## Relation to fictty

Inspire, partial compete. Raxol and fictty share the architecture and the read-back goal; they
differ on who writes the UI. Raxol: a developer, in code. fictty: any agent, as data.

## Could fictty adopt it instead of building?

No. Elixir/BEAM runtime, code-first, and the agent can only operate apps, not create them.

## What fictty should take from it

- **Tools derived from the tree.** fictty's planned MCP adapter should generate semantic tools from
  the current UI value (press this button, pick this row, fill this field), with raw keys as the
  fallback, as the plan already says. Raxol's "focus lens" (only the tools relevant to what's
  focused) is a good answer to tool-list bloat.
- A headless test API in the same verbs as the agent's (`type_into`, `click`, `assert`), which
  fictty's `render --keys` half-covers.

## Sources

- https://github.com/DROOdotFOO/raxol (README)
- https://hex.pm/api/packages/raxol
- https://readme.hex.pm/raxol/2.6.1
