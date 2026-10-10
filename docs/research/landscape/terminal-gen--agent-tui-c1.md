# agent-tui (ConductorOne): "Terminal apps need a DOM"

- **Maker:** ConductorOne (C1); essay by Paul Querna, co-founder and CTO
- **URL:** https://github.com/ConductorOne/agent-tui, essay at
  https://www.c1.ai/labs/research/agent-tui-structured-terminal-access-for-ai-agents
- **Status (2026-10-10):** 13 stars, Apache-2.0 (README badge; GitHub shows no detected licence),
  Rust. Created 25 May 2026, last push 29 July 2026; binaries for macOS, Linux and Windows, a GHCR
  image. Essay published 30 June 2026. Not to be confused with `pproenca/agent-tui`, an unrelated
  PTY driver of the same name.

## What it is

A PTY driver like tui-use, with one important addition: it exposes the screen not only as text
but as an **outline of regions with roles and stable refs** (for example `@vim.mode`), queried
with selectors like `[role=buffer][focused]`, and lets the agent wait on those refs.

## The problem it's solving

From the essay: "A terminal program is already a state machine. The problem is that the state is
presented as a screen." Agents need to know what happened after they pressed a key, and text
snapshots don't say.

## Its path / bet

Retrofit a DOM onto programs that never had one, using adapters. "It does not pretend a cell grid
is a clean API."

## How it works

- A daemon keeps PTY sessions alive; `spawn`, `snapshot --mode text|outline`, `press`, `wait`.
- Built-in adapters (vim, shell) and TOML manifests name screen regions; a generic adapter groups
  any screen into coarse regions.
- `wait` on refs appearing, disappearing or taking a value; `wait --idle` as the fallback.
- `run` for one-shot non-interactive children; asciicast replay; PNG snapshots.

## Strengths

- States the problem fictty exists to solve more clearly than anyone else in this cluster.
- Refs and roles give agents durable handles instead of coordinates.

## Weaknesses / limits

- Adapters per program: without one, structure is coarse; htop-style apps fall back to idle
  waits. The essay says so.
- Adapters read rendered cells, so they are guesses, however good.
- Low adoption.

## Relation to fictty

Inspire. It's the strongest argument for fictty's design from someone who isn't us: a terminal
program that publishes its own structure doesn't need an adapter. Every fictty screen is "a
terminal app with a DOM" by construction.

## Could fictty adopt it instead of building?

No, it solves the reverse problem. But fictty could **publish into it**: an agent-tui adapter (or
manifest) for fictty that reads `get --state` would let agents already using agent-tui see fictty
screens with exact roles and refs instead of inferred ones.

## What fictty should take from it

- Use its vocabulary. "Refs" and "roles" are the words agents and their authors are learning;
  `fictty screen --outline` could print the tree as roles and ids in the same style.
- `wait` on a ref's value ("until `@status` reads done") is a better `watch` predicate than
  polling state.
- Quote the essay in the blog post: it names the problem in one sentence.

## Sources

- https://github.com/ConductorOne/agent-tui (README)
- https://www.c1.ai/labs/research/agent-tui-structured-terminal-access-for-ai-agents
- https://api.github.com/repos/ConductorOne/agent-tui
