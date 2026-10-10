# Cursor Canvas (added in this pass)

- **Maker:** Anysphere (Cursor)
- **URL:** https://cursor.com/docs/agent/tools/canvas
- **Status (10 October 2026):** shipping, closed source. Introduced in Cursor 3.1 on 15 April 2026
  (third-party sources); shared canvases (publish to a read-only live link) on paid team plans; a
  June 2026 update reportedly added a design mode and buttons that run prompts. We didn't find it in
  the first landscape pass; it belongs in this cluster.

## What it is

An agent tool in Cursor that answers with a persistent, interactive canvas beside the chat instead
of text: sections, stat tiles, tables, charts and diagrams. Canvases are saved, reopened, rerun with
fresh data and published as read-only snapshots for teammates.

## The problem it's solving

The same as Artifacts, inside the IDE: analysis and status output that should be looked at, not
read, and that should stay around to be rerun.

## Its path / bet

A middle path between free-form HTML and pure data. Third-party guides describe canvases as
`.canvas.tsx` files written against a first-party React component library (tables, charts,
diagrams). So the agent writes code, but code constrained to a known catalogue, which keeps output
consistent and lets Cursor style it. Cursor's own docs page confirms the layout vocabulary and that
the source is editable, but doesn't name the format.

## How it works

- The agent produces a canvas; a card at the end of the response opens it next to the chat, or
  from the command palette or the Agents Window.
- Reopen and rerun to refresh data, or ask the agent to rerun the underlying query; skills can
  regenerate a canvas from a short prompt.
- **Publish** creates or refreshes a share link: a read-only live snapshot. Requires a paid plan,
  team membership and a privacy mode that allows storage.
- Agent read-back of a canvas's live state isn't documented.

## Strengths

- A constrained component library gives consistency and speed that free-form HTML lacks.
- Rerunnable: the canvas keeps the query, not just the numbers.
- Lives where the developer already is (the IDE), with team sharing.

## Weaknesses / limits

- Cursor-only. No terminal surface, no external agent can write one.
- Still code (TSX) rather than data, so the catalogue is a convention, not an enforced schema.
- Read-back and live streaming aren't documented.

## Relation to fictty

**Inspire.** It's the strongest commercial evidence that the right shape is "agent fills a known
catalogue" rather than "agent writes any page". It also confirms that rerunnable data (keep the
query) matters more than a pretty snapshot. Threat is modest: it's locked to Cursor's IDE.

## Could fictty adopt it instead of building?

No; it isn't available outside Cursor.

## What fictty should take from it

- **Keep the query, not the result.** fictty already binds commands as sources; make "rerun" and
  "refresh" explicit verbs and show when data was fetched.
- **Recipes as skills.** Cursor packages repeatable canvases as skills; fictty should ship its
  recipes the same way.
- **Stat tiles and sections** are the vocabulary that keeps recurring across Cursor, Artifacts and
  Claude Dashboards; check fictty's catalogue covers them without a custom layout each time.

## Sources

- Cursor docs, [Canvas](https://cursor.com/docs/agent/tools/canvas)
- [Cursor Canvas: live charts and tables beside the chat](https://mer.vin/2026/09/cursor-canvas-live-charts-and-tables-beside-the-chat/) (third-party)
- [Cursor shared canvases](https://aicatchup.com/news/cursor-shared-canvases-team-url) (third-party)
- [Agent canvas tool](https://www.learncursor.dev/learn/cursor-agents/agent-canvas-tool) (third-party; source of the `.canvas.tsx` detail)

Not verified: the `.canvas.tsx` file format and the June 2026 design-mode update, both from
third-party guides only.
