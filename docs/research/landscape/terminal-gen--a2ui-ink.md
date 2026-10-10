# a2ui-ink (community Ink renderer for A2UI)

- **Maker:** GitHub `kokoro-ele` (npm `@evanyu/a2ui-ink`; the README says `@kokoro-ele/a2ui-ink`,
  which isn't on npm, so the published name is unclear)
- **URL:** https://github.com/kokoro-ele/a2ui-ink
- **Status (2026-10-10):** 4 stars, Apache-2.0, created 17 July 2026, last push 18 September 2026.
  `@evanyu/a2ui-ink` 0.1.0 published 17 July 2026, 17 downloads in the last month. Listed on
  a2ui.org as the only terminal renderer among community renderers. It follows A2UI issue #1998
  (July 2026), which said "there's no way to render an A2UI surface in a terminal."

## What it is

An Ink (React for the terminal) renderer for A2UI v0.9 and v0.9.1, built on the official
`@a2ui/web_core` message processor. All 18 basic-catalogue components, media as placeholders.

## The problem it's solving

Agents built on A2UI could not show their surfaces to someone in a terminal or over SSH.

## Its path / bet

Reuse the official web core for state and binding, swap only the drawing layer. Minimal surface.

## How it works

`new MessageProcessor([basicCatalog], onAction)`, `processor.processMessages(messages)`, then
`render(<A2uiSurface surface={...} />)`. Tab moves focus, Enter or Space activates, arrows drive
tabs and sliders. Ships the official v0.9.1 examples as fixtures and a live demo against Google's
restaurant-finder sample agent over A2A.

## Strengths

- Small, correct, and leans on the official core, so it tracks the spec cheaply.
- Recognised by the A2UI project's own ecosystem page.

## Weaknesses / limits

- Almost no adoption. A library, not a process: no socket, no read-back, no data sources.
- Ink performance ceiling.

## Relation to fictty

Watch. Together with the Rust a2ui crate and a2tea, it confirms that A2UI in the terminal exists in
three languages already. None of the three is a runtime an outside agent drives.

## Could fictty adopt it instead of building?

No: wrong language and no loop.

## What fictty should take from it

Use the same official v0.9.1 examples as conformance fixtures for fictty's A2UI adapter, and list
fictty on a2ui.org's renderer page once the adapter works; that page is where A2UI users look.

## Sources

- https://github.com/kokoro-ele/a2ui-ink (README)
- https://a2ui.org/ecosystem/renderers/
- https://github.com/a2ui-project/a2ui/issues/1998
- https://registry.npmjs.org/@evanyu/a2ui-ink
