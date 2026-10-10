# Claude Artifacts (claude.ai)

- **Maker:** Anthropic
- **URL:** https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them
- **Status (10 October 2026):** shipping on all plans, closed source. Introduced in June 2024 with
  Claude 3.5 Sonnet. AI-powered artifacts (pages that call Claude, billed to the viewer) arrived in
  June 2025; MCP connectors and persistent storage in October 2025. On 16 September 2026 chat
  artifacts moved to a new system: artifacts created before then are "legacy" (still work, can't be
  created). Docs, Slides and Design templates are on by default, Dashboards (beta) on paid plans,
  Motion (beta, exports MP4) on Team and Enterprise. Enterprise gets the templates on by default
  from 15 October 2026.

## What it is

The chat-side ancestor of every "agent writes a page" product. When Claude produces something
substantial and self-contained (a document, a single-page site, a diagram, a dashboard, a small
tool) it opens beside the conversation as an artifact the person can view, run, edit, return to and
share.

## The problem it's solving

A chat transcript is a bad home for an object. Code, documents and small tools need to sit still,
be iterated on and be handed to someone else. Artifacts separate "the thing we're making" from
"the conversation about it".

## Its path / bet

Model writes code (HTML, React, SVG, Mermaid, Markdown) and the host runs it in a sandbox. Over two
years the bet widened from "show the output" to "the output is an app": pages can call Claude on
the viewer's account, call the viewer's connectors, and keep personal or shared storage (text only,
20 MB per artifact). The September 2026 move to typed templates (Docs, Slides, Design, Dashboards,
Motion) is a notable turn: for common shapes, the model fills a template rather than writing a
page from nothing.

## How it works

- Claude decides (or is asked) to create an artifact; content over roughly 15 lines that stands on
  its own is the stated heuristic.
- It renders in a sandboxed panel beside the chat. Requires "Cloud code execution and file
  creation" to be on.
- AI-powered artifacts call Claude on the viewer's own account, so the author pays nothing.
- Connected apps: each viewer connects their own; connector tools that need per-action approval
  aren't available to artifacts.
- Storage: personal (per viewer) or shared (everyone sees the same data), text only, 20 MB.
- Templates: Slides export to PowerPoint/PDF; Design can use a design system; Docs are collaborative
  documents.

## Strengths

- The largest installed base of any "model writes UI" surface, and the habit-former: millions of
  people have learned to expect an interactive thing rather than a paragraph.
- Viewer-billed AI and viewer-owned connectors solve the "who pays, whose data" problem neatly.
- Templates show the lab learning that free-form HTML isn't always the right output: for decks and
  documents, a structured type with its own editor wins.

## Weaknesses / limits

- Lives in claude.ai (web, desktop, mobile). Not a surface any other agent can write to.
- The model writes code, so every structural change is a regeneration; consistency across
  revisions depends on the model.
- Storage is text only and capped; no backend.
- Read-back is the source the model wrote plus storage, not the rendered page.

## Relation to fictty

**Inspire.** Different surface (chat host vs terminal), different author (Claude only vs any agent).
Its significance for fictty is cultural and directional: it trained users to expect generated,
interactive output, and its recent move to typed templates is evidence for fictty's "data, not
code" position for common shapes.

## Could fictty adopt it instead of building?

No. It is not available outside claude.ai and isn't a component anyone can embed.

## What fictty should take from it

- **Templates are the lab's own admission** that structured types beat raw HTML for recurring
  shapes. fictty's catalogue (table, list, tabs, chart, form) plays the same role, and recipes
  should be named and discoverable the way Slides/Docs/Dashboards are.
- **Personal versus shared state** is a distinction fictty will need once more than one person or
  agent watches a screen.
- **Viewer-scoped data access** (each viewer's own connectors) is the right default for anything
  shared; for fictty, data sources run as the person running the screen, and that should be said
  plainly.

## Sources

- Claude help centre, [What are artifacts and how do I use them?](https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them) (fetched 10 October 2026)
- Claude blog, [Build artifacts](https://claude.com/blog/build-artifacts)
- Simon Willison, [Build AI-powered apps with Claude artifacts](https://simonwillison.net/2025/Jun/25/ai-powered-apps-with-claude/) (June 2025)

Not verified: the exact list of renderable content types in the new (post-September 2026) system,
and whether the `window.claude` API Simon described in 2025 is unchanged.
