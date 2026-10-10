# claude-canvas

- **Maker:** David Siegel (GitHub `dvdsgl`)
- **URL:** https://github.com/dvdsgl/claude-canvas
- **Status (2026-10-10):** about 1,500 stars, 138 forks, MIT. Created 6 January 2026, last push
  8 January 2026, 19 commits, no releases. The README says: "This is a proof of concept and is
  unsupported." Several near-identical copies are listed on skill marketplaces (mcpmarket); we
  couldn't tell which, if any, are maintained.

## What it is

A Claude Code plugin ("Give Claude Code an external monitor") that lets the agent open one of
three prebuilt terminal screens in a tmux split: a calendar (display or meeting picker), a
document viewer/editor (display, edit, email preview) and a flight comparison with a seat map.
The canvases are React components rendered with Ink and run with Bun.

## The problem it's solving

A coding agent's only output is the chat transcript. Some tasks (pick a meeting slot, choose a
seat, select a passage of a document) are far easier to answer by pointing at a picture than by
typing. claude-canvas gives the agent a second screen beside the chat, and gives the person a way
to answer with a click that comes back to the agent as structured data.

## Its path / bet

Prebuilt, hand-designed canvases per task type, driven by a JSON config the agent writes, inside
the terminal the agent already runs in. The bet is that a small set of polished, purpose-built
screens beats a general renderer for the cases that matter.

## How it works

- The skill teaches the agent to run `bun run src/cli.ts spawn calendar --scenario
  meeting-picker --config '{...}'`. The CLI writes the config to a temp file, opens a tmux split
  (it refuses to run outside tmux) and launches the canvas there.
- Each canvas opens a Unix socket (`/tmp/canvas-<id>.sock`). Controller to canvas messages:
  `update` (replace the config), `close`, `ping`, `getSelection`, `getContent`. Canvas to
  controller: `ready`, `selected` (the person's choice), `cancelled`, `error`, `selection`
  (selected text with offsets) and `content` (document text and cursor).
- A TypeScript API wraps the round trip (`pickMeetingTime`, `editDocument`, `bookFlight`): spawn,
  wait for `selected`, return the result.

## Strengths

- Proved demand fast: 1,500 stars for a week of work, mostly from the demo of Claude opening a
  calendar beside itself.
- The loop is right: agent writes data, person answers in place, the answer comes back as JSON.
- The screens look good because people designed them.

## Weaknesses / limits

- Three canvases. Anything else needs a developer to write a new Ink component, a scenario and a
  skill. The agent cannot compose a new screen.
- No general read-back of what is drawn; only the document canvas returns its content.
- No data binding: any change of data goes through the agent sending a new `update` config.
- tmux required; Bun required; unmaintained since January.

## Relation to fictty

Inspire. It is the clearest evidence that people want an agent to put a screen beside itself in the
terminal and get an answer back. fictty generalises it: the canvases become UI values any agent
can write, and the selection round trip becomes `watch` plus `get --state`.

## Could fictty adopt it instead of building?

No. It is an unsupported proof of concept with three fixed screens and no extension path that
doesn't involve writing React. There is nothing to build on beyond the idea and the IPC message
shapes.

## What fictty should take from it

- The demo that sold it: a calendar meeting picker and a seat map. fictty's examples should
  include one "answer by pointing" screen of that kind, written as a UI value, to show the same
  thing without code.
- `selected` / `cancelled` as first-class results. fictty's `watch` should return exactly that
  shape: what the person chose, or that they backed out.
- Opening in a split beside the agent is the expected default; fictty should have a one-command
  "open beside me" path for tmux, herdr and TUIOS.

## Sources

- https://github.com/dvdsgl/claude-canvas (README, `canvas/README.md`, `canvas/CLAUDE.md`,
  `canvas/skills/*/SKILL.md`, `canvas/src/ipc/types.ts`, `canvas/src/terminal.ts`, read from a
  clone on 2026-10-10)
- https://api.github.com/repos/dvdsgl/claude-canvas (dates, stars)
- https://mcpmarket.com/tools/skills/canvas-tui-toolkit (marketplace copies)
