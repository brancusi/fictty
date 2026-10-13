# The demo: an agent builds an app

The window opens almost empty. An agent then builds **Kestrel Cycles**, a bike workshop's app,
in it, one command at a time, and logs each command on the right as it goes. Then it hands the
app over and keeps its live numbers running. It's a script (`agent.py`), not a model, so it
plays the same every time. Every step is a `fictty` command anyone could type in another
terminal while it runs. The shop, the people and the numbers are made up.

```sh
mise exec -- cargo build --release
examples/demo/open.sh               # the window and the agent, in the first theme
examples/demo/open.sh paper         # in another theme (examples/window/themes.json)
examples/demo/open.sh - --fast      # six times faster, for checking it
fictty --socket /tmp/fdemo.sock screen --nodes --style    # read it back while it runs
```

macOS stops drawing a window that's covered, so keep it in front.

## The beats

| # | what the agent does | the command | what it shows |
|---|---|---|---|
| 0 | reads the empty screen | `screen --nodes` | agents read the screen as nodes, not pixels |
| 1 | pushes the app: a header, an inbox, its log | `push` | a whole UI is one JSON value |
| 2 | six messages arrive | `put inbox` | data lives in the store; the UI only names it |
| 3 | reads the screen back | `screen --nodes` | an agent checks its own work |
| 4 | selects Dev's message, types a reply | `set`, `put reply` | view state and text are addressable; writes are undoable edits |
| 5 | adds a Jobs board, moves a bike onto a stand | `patch main`, `put` | one node replaced, everything else kept |
| 6 | adds Stock, filters it by typing | `patch`, `put part` | tables, filters, fields an agent can type into |
| 7 | adds the Floor, live numbers and a log | `merge floor` | a live feed is a stream of merges |
| 8 | cycles themes, reads back a style | `theme next`, `screen --nodes --style` | the UI names roles; themes decide the look; agents can see it |
| 9 | patches a card with a misspelled variant, reads the problem, fixes it | `patch`, `get --state` | wrong names are problems, on the status bar and in the state, never silence |
| 10 | adds Notes, then hands over | `patch`, `put notes` | the app is the person's; the agent keeps feeding it |

## It grows with the window

Since step 2c the window draws whole styles (pad, radius, borders, shadows, gradients, rings), so
the theme beat changes more than colours as soon as the themes use them (step 2f rewrites them).

The demo is part of the plan's test of done, next to the wringer (`examples/window/`): each
step of [docs/design/native.md](../../docs/design/native.md) lands with the demo showing it
off. What each step adds is in [docs/design/demo.md](../../docs/design/demo.md).
