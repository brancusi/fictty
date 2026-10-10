# marimo

- **Maker:** marimo Inc. (Akshay Agrawal, Myles Scolnick), acquired by CoreWeave (announced 30
  October 2025)
- **URL:** https://marimo.io and https://github.com/marimo-team/marimo
- **Status (10 October 2026):** Apache-2.0, 23.1k stars, release 0.25.1 on 1 October 2026, pushed
  daily. CoreWeave said it stays open source. `marimo pair`, an agent skill for driving a live
  notebook, launched in 2026 (repo created February 2026, Apache-2.0, 426 stars, last push
  September 2026); its underlying `marimo._code_mode` API is marked private and unstable.

## What it is

A reactive Python notebook stored as a plain `.py` file. Cells form a dependency graph, so running
or editing one reruns its dependents and deletes stale state; notebooks can also run as scripts or
be served as apps with UI elements (sliders, tables, forms).

## The problem it's solving

Jupyter's hidden state and out-of-order execution make notebooks irreproducible and hard to share.
More recently: coding agents' file and shell tools are bad at stateful data work, where the
interesting values live in memory between steps.

## Its path / bet

A notebook that is both a reproducible program and an interactive document. The 2026 bet is that
agents should "use notebooks the way people do": run code in the live kernel, inspect results,
edit cells, and keep the useful work, while the person watches and edits the same notebook.

## How it works (concretely)

- Static analysis of each cell's definitions and references gives the dependency graph.
- `marimo edit nb.py --mcp` exposes read-only MCP tools (inspect cells, data, errors).
- `marimo pair` (install with `npx skills add marimo-team/marimo-pair`) gives Claude Code, Codex or
  any skills-capable harness a CLI that executes Python in the live kernel (with notebook variables
  in scope; "the call returns the resulting values and errors directly") and a small API to add,
  edit and delete cells. After edits, marimo reports each cell as clean, errored or broken
  upstream. The team moved here after finding read-only MCP plus file edits too slow and too
  fixed.

## Strengths

- The nearest thing in the landscape to fictty's loop: an agent and a person share one live,
  stateful surface; the agent writes to it and reads exact values back.
- Reactive, reproducible, plain-file format; agents can diff and review it.
- Strong momentum and now a well-funded owner.

## Weaknesses / limits

- It's a notebook: the UI is code cells and their outputs, in a browser. Fine for data work, not a
  general screen.
- The agent API is private and unstable by their own description.
- Ownership by a GPU cloud may pull priorities towards CoreWeave's platform.

## Relation to fictty

**Watch; partial competitor for one use case.** For "the agent explores data with me and shows me
charts," marimo pair is a strong answer today, and the person gets real Python and real plots. It
doesn't run in the terminal, isn't a screen for non-data tasks, and the agent reads back values,
not what the person sees. Its lesson matters more than its overlap.

## Could fictty adopt it instead of building?

Not as a substitute: a browser notebook can't be the screen inside a herdr pane over SSH. For data
exploration specifically, a fictty user could reasonably be pointed at marimo. A fictty data source
could read from a marimo app's outputs, but that's speculative.

## What fictty should take from it

- **Live state beats files for agent work.** marimo found that read-only tools plus file edits were
  too slow and moved to letting the agent act in the live process. fictty already lets agents act
  on the live screen; keep that the primary path and treat files as save and load.
- **Report health after a patch.** marimo tells the agent which cells are clean, errored or broken
  upstream. A fictty patch should return the same: which nodes rendered, which data sources failed,
  which bindings are now empty.
- **A skill is the distribution.** `npx skills add` is how marimo reaches Claude Code and Codex.
  fictty should ship a skill the same way.

## Sources

- [marimo-team/marimo](https://github.com/marimo-team/marimo) and [marimo-pair](https://github.com/marimo-team/marimo-pair) (GitHub API, 10 October 2026)
- marimo, [marimo pair](https://marimo.io/blog/marimo-pair) and [Notebooks as a tool for agents](https://marimo.io/blog/notebooks-as-a-tool-for-agents) (20 July 2026)
- marimo docs, [MCP](https://docs.marimo.io/guides/editor_features/mcp/)
- CoreWeave, [CoreWeave acquires Marimo](https://coreweave.com/news/coreweave-acquires-marimo-to-unify-the-generative-ai-developer-workflow) (30 October 2025); Simon Willison, [link post](https://simonwillison.net/2025/Oct/31/coreweave-acquires-marimo)
- Talk Python, [episode 555 on marimo pair](https://talkpython.fm/episodes/show/555/marimo-pair-a-canvas-for-agent-developers-collaboration) (July 2026)
