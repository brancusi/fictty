# fictty

**A terminal UI that is one value.** An agent writes it as JSON, rewrites it live a piece at a
time, and reads back exactly what it drew. Data streams from your commands straight to the screen,
never through the model.

![The fictty wordmark: letters built from overlapping panes of tinted glass](docs/design/identity/glass-wordmark.jpg)

> **Status: prototype.** It works and it's fast, but the format will change. The plan is in
> [docs/PLAN.md](docs/PLAN.md). Milestones are on [the fictty blog](https://fictty.com/blog/); the
> start of the story is on the Thought Control dev blog:
> [the experiment](https://thoughtcontrol.app/blog/a-ui-is-one-value/),
> [the landscape](https://thoughtcontrol.app/blog/mapping-the-landscape/) and
> [why it's its own project](https://thoughtcontrol.app/blog/fictty/).

## What it's for

Agents are good at deciding what a screen should show and bad at being the screen. When an agent
has to answer every keystroke itself, the UI is as slow as the model. fictty splits the work:

- **The agent writes the screen once,** as a small JSON value: components, the data they bind to,
  keys and theme.
- **The runtime does everything else** at frame rate: drawing, scrolling, selection, focus,
  mouse, live data.
- **The agent stays in the loop where it matters.** It can patch one node, place a callout, send
  keys or clicks, and read the screen back as text or state at any time.

It sits between [caretline](https://caretline.app), the text-editing engine, and apps like
[Thought Control](https://thoughtcontrol.app): apps describe screens, fictty runs them.

## Try it

One line fetches fictty into `~/.fictty` and opens a tour of it: nine short chapters, each with
something to play with and a quest, from a screen and its JSON side by side to an agent driving
the screen while you watch and a firehose of updates. It builds `main` if you have Rust, and
otherwise runs the latest release's binary (macOS and Linux):

```sh
curl -fsSL https://fictty.com/try.sh | sh
```

To drive it from a second terminal, open the Agents chapter (or the Agent app) and click a command (or select it and
press Enter): it's copied ready to paste, and it sets that terminal up so `fictty` reaches the
showcase. `tour.sh` there has an agent drive the whole thing. `… | sh -s -- <tag or commit>` runs a
given version, each in its own folder and store.

From a checkout:

```sh
examples/showcase/play.sh       # the tour, then the apps: notes, a ledger, a live market, a gallery
examples/showcase/tour.sh       # in another terminal: watch an agent drive it
```

Or by hand:

```sh
cargo build --release
export PATH="$PWD/target/release:$PATH"

fictty run examples/stress/ticker.json      # one terminal: a ticker streaming 500 updates a second

fictty push examples/stress/plasma.json     # another terminal: the whole UI swaps in one frame
fictty patch header node.json               # replace one node by id
fictty key j j                              # type, like a person
fictty screen                               # read back the frame as text
fictty get --state                          # read back everything, as JSON
fictty render examples/tasks.json --size 100x30   # no terminal at all
```

The demos in [examples/](examples/) cover stress tests, a live finance dashboard driven by a
scripted "analyst", and a pixel tour. [docs/guide.md](docs/guide.md) walks through each one.

## The model

A UI is one JSON value:

```json
{
  "root": {"type": "row", "children": [
    {"type": "table", "id": "tasks", "title": "Open tasks", "bind": "tasks",
     "columns": [{"title": "task", "value": "{text}"}, {"title": "due", "value": "{due}", "size": 11}],
     "keys": {"x": {"run": ["thc", "done", "{id}"]}}},
    {"type": "text", "bind": "@tasks", "text": "{text}\n\ndue {due}"}
  ]},
  "data": {
    "tasks": {"cmd": ["thc", "q", "status:open", "--json"], "path": "items", "every": 3}
  }
}
```

- **Components** come from a small set with behaviour built in: layout (rows, columns, stacks
  with floating and modal children), text, lists, trees, tables, editors, inputs, buttons, tabs,
  charts, toasts, layers (callouts, spotlights, arrows) and pixels (cards, plots, SVG, images).
  `when` shows a node while a condition holds, so dialogs and responsive panes are data too.
- **Data sources** are commands, JSON-line streams, literal values, or an embedding app's own
  (fictty is a Rust library too: `fictty::App::new(ui).host(my_app).run()`). `@tasks` binds the row
  selected in `tasks`.
- **View state is kept by node id,** like React's keys, so a push keeps your selection and focus.
- **The app's data lives in a store, apart from the UI.** A push never touches it. Typing, keys
  and agents write it, each write with a revision, so an agent can't overwrite what you just
  typed (`fictty put key file --if-rev 12`). Editors are [caretline](https://caretline.app).
- **The look is a theme, not the UI's business.** Nodes ask for a `variant` and a `tone`; themes
  (loaded with `--themes`, switched with a key) decide the colours.
- **It's the Elm architecture:** a serializable state, a pure `update` that returns effects, a
  pure `view`. That's what makes reading back, replay and (soon) history possible.

## How fast

Release build, a 170×52 terminal, one machine: about 6,000 whole-UI pushes a second from one
client while drawing about 108 fps; 1,000 patches a second with every frame under 1 ms; a
50,000-row list replaced 20 times a second draws in 0.18 ms. The full table is in the
[guide](docs/guide.md#stress-demos).

## Pixels

On terminals that speak the kitty graphics protocol (Ghostty, kitty, WezTerm), fictty draws
cards under the text, anti-aliased plots, SVG filled from data and images, and falls back to
cells everywhere else. Guidance for agents: components first, SVG for anything custom, PNG only
for real bitmaps, and animate the data, not the pictures.

## Docs

- [docs/guide.md](docs/guide.md): every feature, demo and measurement
- [docs/PLAN.md](docs/PLAN.md): the roadmap
- [docs/design/architecture.md](docs/design/architecture.md): the layers and the rules behind them
- [docs/design/time-travel.md](docs/design/time-travel.md): history an agent can use (proposal)
- [docs/design/identity/](docs/design/identity/): the brand explorations
- [AGENTS.md](AGENTS.md): for agents (and people) working on this repo

## License

MIT. See [LICENSE](LICENSE).
