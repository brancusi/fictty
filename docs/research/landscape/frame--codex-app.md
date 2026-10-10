# The OpenAI Codex app

- **Maker:** OpenAI
- **URL:** https://openai.com/index/introducing-the-codex-app/ (announcement; returned 403 to our
  fetcher), https://learn.chatgpt.com/docs/changelog (changelog)
- **Status (10 October 2026):** shipping, closed source, part of ChatGPT plans. Launched as a
  standalone macOS app in February 2026 (date from our earlier pass; we couldn't re-read the
  announcement). Windows in March 2026. On **9 July 2026 (26.707) "Codex joins the ChatGPT desktop
  app"**: on macOS and Windows it is now a mode of the ChatGPT desktop app rather than its own
  download. The CLI (open source, Apache-2.0, 128,500 stars, rust-v0.162.1 on 9 October 2026) and
  the app share the Codex app server.

## What it is

A desktop "command centre" for running several Codex agents at once: threads per task, each able
to work in its own git worktree, with review of diffs, terminals, an in-app browser and computer
use, and a growing set of ways to show results richer than chat text.

## The problem it's solving

Supervising parallel agents is awkward in a terminal: the scrollback is the only view, there's no
good place to review diffs or preview what was built, and the person has to juggle windows. The
app puts threads, review and previews in one native window, and with the ChatGPT merger puts
coding next to everything else a ChatGPT user does.

## Its path / bet

Leave the terminal for the work surface, keep the terminal as the engine. Agents run through the
same harness as the CLI; the app is the frame around them, and increasingly also the content:
browser, Sites, inline visualizations, MCP apps, canvas views.

## How it works (from the changelog)

- **16 April 2026 (26.415):** "becoming a broader workspace"; an early in-app browser where you
  can "comment directly on the rendered page"; computer use for macOS apps.
- **23 April:** browser use. **21 May:** plugin bundles (skills, MCP servers, lifecycle hooks) for
  teams. **29 May:** thread coordination across local projects and worktrees; computer use on
  Windows.
- **2 June:** a **Sites plugin** to "create, save, deploy, and inspect websites, dashboards,
  internal tools". **11 June:** fixes to "MCP app sizing after right-pane transitions" (so MCP Apps
  render in a side pane).
- **9 July:** merger into the ChatGPT desktop app; inline editing of Markdown and code with
  annotations and "ask Codex to revise selected content"; custom domains for Sites.
- **iOS, July:** inline visualizations in tasks (13 July), Mermaid diagrams inline (20 July).
- **24 August / 5 September:** `codex mcp-server` deprecated, then removed, in favour of the
  app server (a JSON-RPC protocol other tools, TUIOS among them, now speak).

## Strengths

- OpenAI's distribution: every ChatGPT desktop user is one click from it.
- It is collecting every content surface at once: browser with comments, Sites, MCP Apps, inline
  visualizations, editable documents.
- The app-server protocol makes the agent drivable from other frames.

## Weaknesses / limits

- A desktop window: no SSH, no running where a remote agent lives, other than the agent itself
  being remote.
- Its rich content is HTML (Sites, MCP Apps, the browser) or chat-embedded visualizations; we
  found no evidence of an agent-writable UI value it can read back as state.
- Closed, tied to OpenAI models and plans.
- Our view of features is the changelog; we didn't use the app.

## Relation to fictty

**Watch; indirect competitor.** It is the strongest evidence for Arnór's prediction that agent
work moves into desktop apps, and the HTML camp's best-distributed product. It doesn't compete
for a terminal pane; it competes for the person's attention. If most people supervise Codex in
the ChatGPT app, a terminal screen matters less to them.

## Could fictty adopt it instead of building?

No. It's a closed product, not a platform for third-party UI runtimes beyond MCP Apps (HTML).

## What fictty should take from it

- **Comment on the rendered thing.** "Comment directly on the rendered page" is the same loop as
  Lavish: the person answers *on* the UI, and the agent gets the feedback. fictty's `watch` plus a
  way to attach a note to a node is the terminal version.
- **The frame is eating the content layer** in GUI land. In the terminal, nobody has done it yet;
  that's fictty's opening, and it may not last.
- **Speak the app server** (or at least don't block on it): if Codex agents report through it,
  fictty screens launched by Codex should be reachable the same way.

## Sources

- https://learn.chatgpt.com/docs/changelog (redirect from developers.openai.com/codex/changelog),
  read 10 October 2026
- https://github.com/openai/codex (stars, releases)
- https://www.i-programmer.info/news/105-artificial-intelligence/18719-openai-releases-codex-for-windows.html
- https://www.gradually.ai/en/changelogs/codex-app/ (secondary changelog mirror)
- Our earlier pass: docs/blog/2026-10-why-the-terminal.md
