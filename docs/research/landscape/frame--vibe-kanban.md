# Vibe Kanban

- **Maker:** BloopAI (bloop)
- **URL:** https://www.vibekanban.com, https://github.com/BloopAI/vibe-kanban
- **Status (10 October 2026):** **sunsetting.** bloop announced it was shutting down as a company
  on 10 April 2026; the project continues as open source, community maintained. Rust + TypeScript,
  Apache-2.0, 28,315 stars, repository created 14 June 2025, last push 19 September 2026. Not in
  our first pass; added because its shutdown is evidence about the business of agent frames.

## What it is

A local web app (`npx vibe-kanban`) where you plan work as kanban issues, then start a coding agent
(Claude Code, Codex, Gemini CLI, Copilot, Amp, Cursor, OpenCode and others) in a workspace with a
branch, terminal and dev server; review diffs with inline comments sent back to the agent; preview
the app in a built-in browser; open and merge PRs.

## The problem it's solving

"Software engineers spend most of their time planning and reviewing coding agents," so make
planning and review faster.

## Its path / bet

Put the task board at the centre and treat agents as workers you assign issues to. A browser UI
served locally, so no desktop install.

## How it works

Issues on a board; each started issue becomes a workspace (worktree, terminal, dev server) running
an agent; review happens in the same UI, with comments fed back to the agent.

## Strengths

- Very fast adoption (28,000 stars in under a year) and daily use by thousands, per the makers.
- The board-first framing makes the person a planner and reviewer, which is the real job.

## Weaknesses / limits

- The company "couldn't find a business model that we could get excited about"; most users were
  free. Future maintenance depends on the community.

## Relation to fictty

**Inspire (and caution).** Two lessons. The frame layer is crowded and hard to monetise: if a
28,000-star frame couldn't sustain a company, a content runtime won't by being a product either.
And the board is a screen an agent could put up and keep live from data; it doesn't need to be an
app.

## Could fictty adopt it instead of building?

No; it's a web app frame, now community-maintained.

## What fictty should take from it

- Keep fictty a small open runtime with no service to sustain; integrate with frames rather than
  becoming one.
- A kanban view bound to `gh issue list --json` or a task CLI is a good fictty demo of "an app's
  main screen as one value".

## Sources

- https://github.com/BloopAI/vibe-kanban (README), read 10 October 2026
- https://www.vibekanban.com/blog/shutdown
