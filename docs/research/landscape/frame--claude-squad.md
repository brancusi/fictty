# Claude Squad

- **Maker:** smtg-ai
- **URL:** https://github.com/smtg-ai/claude-squad, https://smtg-ai.github.io/claude-squad/
- **Status (10 October 2026):** v1.0.20, released 20 August 2026; last push the same day. Go,
  AGPL-3.0, 8,587 stars. Repository created 9 March 2025. Quieter for seven weeks; mature for its
  scope. Not in our first pass; added as the archetype of the first wave of agent managers.

## What it is

A terminal app (`cs`) that manages several local agents (Claude Code, Codex, Gemini, Aider and
others), each in its own tmux session and git worktree, from one list: start, watch, review the
diff, check out, push.

## The problem it's solving

Running several agents on one repo without them stepping on each other, and without juggling
terminal tabs.

## Its path / bet

A thin TUI over tmux and git worktrees. Don't build a multiplexer; use tmux and add the agent
list and the diff review.

## How it works

Each instance is a tmux session in a worktree; the TUI shows a list with a preview of the selected
session and its diff; keys attach, pause, resume, push. Auto-accept ("yolo") mode lets tasks
finish in the background.

## Strengths

- Small, understandable, works anywhere tmux works.
- Defined the pattern (worktree per agent, list plus preview plus diff) that herdr, TUIOS,
  Superset, the Codex app and others later absorbed.

## Weaknesses / limits

- No status protocol, no inbox, no remote machines, no API for agents to drive it.
- Its niche has been overtaken by herdr and TUIOS in the terminal and by desktop apps.

## Relation to fictty

**Watch.** A useful data point: the first-wave managers were single-purpose TUIs written as code.
Their screens (a list, a preview, a diff) are exactly the kind of screen fictty can express as
data, which suggests a demo: "claude-squad's main screen as one fictty value bound to `git
worktree list` and `tmux list-sessions`."

## Could fictty adopt it instead of building?

No; unrelated layers.

## What fictty should take from it

- The list-preview-diff layout is the canonical supervision screen; ship it as a fictty example.
- A lesson about durability: a narrow tool gets absorbed by broader frames within a year. fictty's
  scope must stay a layer others build on, not a product people choose instead of their frame.

## Sources

- https://github.com/smtg-ai/claude-squad (README, releases), read 10 October 2026
