# Superset

- **Maker:** Superset (superset-sh)
- **URL:** https://superset.sh, https://github.com/superset-sh/superset
- **Status (10 October 2026):** desktop v1.37.0, released 8 October 2026. Repository created
  21 October 2025; 15,045 stars. Licence: Elastic License 2.0 (source-available, not OSI open
  source). macOS primary, Linux AppImage experimental, no Windows. A Pro tier gates mobile. Not
  in our first pass; added as the most feature-complete GUI agent workspace that isn't from a lab.

## What it is

"An agentic IDE to orchestrate 100+ coding agents in parallel": a desktop app where each task
gets a git worktree, terminals, a CLI agent of your choice (Claude Code, Codex, others, on your
own subscription), a diff viewer, a browser pane with a Design mode, PR review, automations,
remote access, an iPhone app, and **Pages**.

## The problem it's solving

Planning and reviewing many agents' work is now most of the job. Superset puts the review tools
(diffs, previews, PR comments) and the supervision tools (status, chimes, badges) next to the
agents, and sends feedback back to an agent with context attached.

## Its path / bet

Bring-your-own-agent desktop workspace: don't build the agent, build everything around it, and
make every review surface a way to talk back to an agent.

## How it works

- Worktree-per-workspace with its own terminals and detected ports; persistent terminal sessions.
- **Design mode:** select an element in the browser pane and send a change request with element
  context to an agent.
- **PR feedback:** select diff lines and send them to a running or new agent session.
- **Pages:** "Ask an agent to publish a Page. Teammates pin comments, and a watching agent can
  revise it at the same link." The host and session must keep running for feedback to arrive.
- CLI, SDK and MCP server so agents and scripts can manage workspaces.

## Strengths

- Every surface closes the loop back to an agent (element, diff line, page comment).
- Pages are an agent-published, commentable, revisable artifact inside the workspace.

## Weaknesses / limits

- Desktop, mostly macOS; source-available licence.
- Pages are HTML artifacts; no terminal story beyond hosting terminals.
- We read the README only; we didn't verify how Pages are hosted or what format they take.

## Relation to fictty

**Inspire.** Superset shows where the GUI frames are heading: the frame becomes the place the
person *responds*, and the agent watches for the response. Pages are Claude Code Artifacts plus
Lavish inside an IDE.

## Could fictty adopt it instead of building?

No. Different medium, source-available, desktop-only.

## What fictty should take from it

- **Feedback with context.** "Send this element / this diff line to the agent" is the gesture
  fictty should support on nodes: select a row, attach a note, and `watch` delivers it to the
  agent with the node id and value.
- **A watching agent revises at the same address.** fictty already patches by id; make "the
  person commented, the agent patched" a first-class, demoed loop.

## Sources

- https://github.com/superset-sh/superset (README, LICENSE, releases), read 10 October 2026
- https://docs.superset.sh/pages
- https://superset.sh/blog/review-agent-work-with-pages
