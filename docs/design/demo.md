# The public demo

Decided with Aram on 13 October 2026: fictty's public demo is **one app that an agent builds
live in the native window**, not a tour of features. It replaces the terminal showcase
(`examples/showcase/`, what `try.sh` runs today) once the window ships in phase 5. Until then
it grows step by step with the plan, and it is part of each step's test of done beside the
wringer.

## Why this shape

fictty's pitch is that a UI is a value an agent can push, patch, fill, read back and restyle,
while a person uses it. A feature tour shows the parts; an agent building a believable app in
front of you shows the point. Every step it takes is a real command, logged on screen with the
command itself, so the demo doubles as the shortest possible guide.

## What it is today

`examples/demo/` (`open.sh`, `agent.py`, `start.json`): **Kestrel Cycles**, a made-up bike
workshop. The agent pushes the shell, fills an inbox, reads the screen back, selects a message
and types a reply, patches in a Jobs board and moves a bike between columns, adds a stock table
and filters it, adds live floor numbers (a `merge` a tick), cycles themes and reads back a
resolved style, adds Notes, then hands over and keeps the floor live. It uses only what the
window draws today: text with markup, lists, a table, tabs, inputs, editors, buttons, toasts.

Building it found one engine bug, fixed: `fictty set <list> {"selected": n}` was undone at once,
because the list still remembered the old row's key and its selection followed that row back.

## What each step adds

Each plan step lands with the demo showing it. A beat is added or improved, and the step isn't
done until the demo shows it in a real window.

| step (native.md) | what the demo gains |
|---|---|
| 2b–2c, parts as whole styles, the window draws them | the theme beat stops being colour swaps: radius, borders, depth and pad change too, so each theme looks like a different app |
| 2d, type | the header in a `display` role; the Floor's numbers large, tabular; message previews that cut with `…` |
| 2e, cards, the status bar | done: the Floor's stats wear the theme's `card` variant, and a new beat has the agent patch a card with a typo, read the problem back and fix it, while the status bar shows it |
| 2f, the styling study | done: the header and the Floor's numbers name type roles (`heading`, `stat`), and the six rewritten themes change type, edges and depth as the agent cycles them |
| 3, the layout vocabulary | the Floor's cards are a grid of as many as fit, up to four, so they wrap (done 17 October 2026); Notes and Stock scroll themselves (an editor, a table); a settings dialog laid out without tricks is still to write |
| 4, rows as nodes | the Jobs board as real cards; the inbox rows with avatars and badges; a chat with the customer, in bubbles |
| 5, editors | links and tasks in Notes, clicked |
| charts and icons | takings as a sparkline and bars on the Floor; icons on the tabs |
| toasts and layers | the agent points with callouts and rings ("this one needs you"), the way the terminal tour did |
| phase 2, drag and drop, screenshots | the agent moves a card by drag and so can you; it takes a screenshot of its work and says so |
| phase 5, ship | `try.sh` downloads the app and opens this demo; the landing page and a post show it |

## Rules for the demo

- **Every beat is a real command**, shown with the command. No hidden setup, nothing the
  person couldn't do from another terminal.
- **The person can take over at any moment.** The agent writes to the store and moves view
  state; it never fights the person for the focus (it doesn't send keys).
- **No tricks.** If a beat needs a workaround, that's a gap in fictty, and it goes in the plan.
- **Made-up data.** The shop, the people, the parts and the numbers are invented.
