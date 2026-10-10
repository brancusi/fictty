# Laura (laura-tui)

- **Maker:** Thomas Eleff (GitHub `thomaseleff`)
- **URL:** https://github.com/thomaseleff/laura-tui
- **Status (2026-10-10):** 0 stars on GitHub, MIT, Rust. Created 29 August 2026; v0.14.1 on
  7 October 2026; about 90 commits, releasing several times a week. Installs with `uv tool install
  laura-tui` or `cargo install laura-tui`. Very new and almost unknown, but the closest match to
  fictty's loop that we found in the terminal.

## What it is

"A TUI workspace that gives your agent an API over the TUI, so your agent can tile panes while you
pair-program." Laura runs your agent CLI (any of them) in a shell pane and gives the agent a
`laura` CLI, taught by a Claude Code plugin skill, to open files, plans, diffs and logs in panes
beside the chat, highlight lines, and annotate.

## The problem it's solving

Pair-programming with an agent means constant switching between the terminal, the editor and the
browser to look at what the agent is talking about. Laura lets the agent show you the thing in
place, and lets you answer on the thing (line comments, an inline review) rather than in prose.
The README also pitches it for learning: "your agent shows and explains concepts live, step by
step."

## Its path / bet

The agent composes the workspace, not the content: panes show real files, diffs and logs, and the
agent arranges and annotates them. Feedback flows back as a structured review in the chat.

## How it works

- One socket per workspace (`LAURA_TAB` in the shell); NDJSON, one request and one response per
  connection. Messages: `open` (split a pane and render a file, with `split`, `dir`, `ratio`,
  `side`, `highlight`, `diff`, `panel` to replace in place, `edit` for Neovim, `dry_run`),
  `close`, `focus`, `highlight`, `diffview`, `comment`, `layout`, `ready`, `update`.
- Panes have stable integer ids, never reused; requests address panes by id.
- `laura layout` and `--dry-run` return a JSON report of every pane's rect, overflow rows and
  clipping, so the agent can check whether the person can actually see what it opened.
- The person comments on lines and submits an inline review with Shift+S; it arrives in the chat
  as a `[laura review · <path>]` block of threads. While the person is typing, Laura holds agent
  requests until they finish.
- File panes reload on change, render Markdown, syntax-highlight, and mark lines changed since
  `HEAD`; `laura tail` follows logs.

## Strengths

- The full loop, in the terminal, today: agent writes to a socket by id, person answers in place,
  answer returns structured.
- Thoughtful agent ergonomics: dry runs, overflow reports, warnings instead of errors, holding
  writes while the person types, a journal of sessions.
- Works with every agent CLI, because the agent runs inside it.

## Weaknesses / limits

- Content is fixed to files, diffs, logs and an editor. No tables, forms, charts or custom screens;
  the agent can't build a UI, only arrange documents.
- It owns the whole terminal (it is the workspace and the multiplexer), so it competes with herdr
  and TUIOS rather than sitting inside them.
- No read-back of the rendered frame as text, as far as the docs say; the agent learns geometry,
  not content.
- No adoption yet; one maintainer.

## Relation to fictty

Compete and inspire. Laura and fictty make the same bet (an agent drives a terminal screen over a
socket by id, and the person answers in place) from opposite ends: Laura gives the agent
documents to arrange, fictty gives it components to compose. A fictty screen showing a file with
highlighted lines and comments is a subset of what Laura does well; a Laura pane showing a live
table is impossible.

## Could fictty adopt it instead of building?

No. Laura doesn't let the agent describe a screen, which is fictty's core. But it is the project
whose agent ergonomics fictty should study hardest.

## What fictty should take from it

- **Overflow reporting.** After a push, tell the agent which nodes are clipped or overflow and by
  how much. That is read-back aimed at the real failure (the person can't see it), and fictty
  doesn't have it.
- **Dry runs** for push and patch.
- **Hold writes while the person is typing** in a fictty input, and say so, rather than redrawing
  under their cursor.
- **Feedback as a structured block in the chat** (Laura types the review into the agent's shell).
  fictty's `watch` returns to the agent that asked; Laura's push into the transcript works with
  agents that can't long-poll.
- **Learning as a use case:** "shows and explains concepts live, step by step" is exactly the
  teaching screen fictty's tours aim at.
- Stable, never-reused ids.

## Sources

- https://github.com/thomaseleff/laura-tui (README, `docs/protocol.md`,
  `docs/agent-reference.md`, git log, read from a clone on 2026-10-10)
- https://api.github.com/repos/thomaseleff/laura-tui
