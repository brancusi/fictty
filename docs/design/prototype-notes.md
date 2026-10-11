# Notes from the store-and-theme prototype

10 and 11 October 2026. What we looked at before building, what we built, what we decided and
why, and what we learned along the way. The rules themselves are in
[the state and theme proposal](state-and-theme.md); the order of work that follows from this is in
[the plan](../PLAN.md).

## Why we built this, and not on something else

We stopped to ask whether fictty should exist at all, and tried the closest tools first.

### json-render and its Ink renderer

The nearest thing to fictty: an agent writes a JSON spec, a terminal draws it, changes stream in
as patches (RFC 6902). We ran version 0.21.0 with only its built-in components.

- **Good:** dashboards (metrics, sparklines, bar charts, tables, badges) look right with no design
  work; specs stream in piece by piece; values written to its state update the screen without
  resending the spec; Markdown and callouts render well.
- **Broken as published:** Tab doesn't move between form fields, so every key lands in the first
  one. One line of their code causes it; changing it makes the whole form work. Worth a pull
  request. There's also no visible focus marker. While a spec is still streaming, a real terminal
  shows `[Missing: …]` placeholders.
- **Missing, because of Ink:** no full screen (alternate screen), no scrolling (`overflow` is only
  visible or hidden), no mouse, no layers or popups beyond one confirm dialog, display-only tables,
  redraws capped at 30 a second, no pictures. Actions can only set, append or remove state, exit
  and log: a key can't run a command or reach an agent.
- **Missing, because it's a library:** nothing lets an outside agent push or patch a running
  screen, send keys, read back what's shown or selected, or point a widget at a command.

**Verdict:** the format is good and maps closely onto ours; the long-running program around it is
what fictty is. Building that host on Ink would be fictty again, in another language, on a weaker
engine. Take the format as an input; send the focus fix upstream.

### OpenTUI

A Zig core with a TypeScript and React interface, built by the opencode team. We ran 0.5.17 on
Node 26 (Bun is its main home): a scrolling box scrolled with the wheel, a select moved with the
arrows, and the frame came back as text headless in about 20 ms.

- **It has what Ink lacks:** full screen and a split-footer mode, mouse, scroll boxes, images,
  Markdown, code and diff views, text areas, an embedded terminal, serving over SSH, a settable
  frame rate, and a buffer we could read back for `screen`.
- **It doesn't have the loop either.** There's no JSON format (screens are code), no host process,
  no data sources, and its state lives in component objects, not one value we could save or replay.
- **Moving to it** means rewriting fictty in TypeScript, giving up the single binary, caretline
  and resvg, and tracking an API that changes weekly and follows one product's needs.

### ratatui, compared (numbers from 10 October 2026)

| | ratatui | OpenTUI |
|---|---|---|
| Age | tui-rs from 2016, forked in 2023 | July 2025 |
| Use | 58.7M downloads; 6,310 crates depend on it | 4.2M npm downloads a month, mostly opencode |
| Apps | Codex CLI, yazi, atuin, gitui, bottom, trippy, television, rainfrog, csvlens… | opencode, then a steep drop |
| Kit | Low level in core; the rest is separate crates | More finished components in the box |
| Packaging | One static binary | Bun or Node 26.4+, a native library per platform |

The ecosystem is large and fragmented: 1,116 crates tagged `ratatui`, 53 widget libraries on
awesome-ratatui, no shadcn-style kit, and text editing split across three forks of tui-textarea.
The useful crates when we need them: ratatui-image, tui-markdown, tui-term, tachyonfx (the motion
OpenTUI's demo has), tui-popup, tui-scrollbar. ratatui's own `demo2` shows how polished it can
look; the polish is the author's work.

**Verdict:** stay on ratatui. Redrawing everything from state each frame matches the pure `view`.
fictty fills the missing role: one consistent kit and theme over the best crates, so an agent
never meets the patchwork. OpenTUI's component list is our target.

### caretline over tui-textarea

tui-textarea keeps its text and cursor inside the widget, and its serde support covers only
inputs, so its content can't be saved, read back or replayed. That breaks fictty's first rule.
caretline follows the same Elm architecture, separates the document (text and undo history) from
the view (carets and scroll), has Helix's model (ropes, many carets, an undo tree), and already
has an MCP server for editing beside a person. The cost is youth: a few weeks on crates.io, one
maintainer.

### The gap, in one line

Every library we tried is for whoever writes the app. None lets an outside agent push a screen
into a running terminal, patch part of it, read back what the person sees and did, and answer in
place. That loop is fictty; the question was only which engine sits under it.

## What we built

On the `store-theme-prototype` branch, merged into `main` on 11 October: about 4,200 lines across
29 files, 37 tests.

- **The store:** named, versioned entries, either a JSON value or a caretline document. Writes from
  typing, store actions and agents (`put`, `merge`, `edit`, all with `--if-rev` and `--by`);
  `read`; seeds; atomic persistence with `--store`.
- **Editors and inputs on caretline**, their text in the store. `doc` templates make an editor
  follow a list's selection, two editors on one entry share it, Markdown styling, click to place
  the caret, `accept: "number"` inputs.
- **Themes as their own layer:** tokens, parts, variants, `variant` and `tone` on nodes, switched by
  key or `fictty theme`, and a `$theme` source.
- **Store actions on keys:** `append`, `put`, `remove`, `do`, `theme`.
- **Problems in words:** wrong shapes at binds, unknown variants, tones and `accept` values.
- **Built-in sources:** `$log` (agent activity), `$store` (every entry by revision), `$theme`.
- **More drawing:** table totals, bars across, padding, a chart legend only for several series.
- **The showcase** (`examples/showcase/`): a menu, notes, a ledger with a helper process doing the
  books, a live market, a gallery of every component and variant, and an agent's log. There are
  three themes, and `tour.sh` drives all of it over the socket.

## Design decisions

- **State is split by owner.** Agents own the UI and replace it freely; the store is shared and
  every write can name the revision it was based on; people own their view (selection, carets,
  scroll, focus) and the theme. A push never touches the store or the theme. This came straight
  from the first demo: a push could delete a note, and `set` could overwrite what a person had
  just typed.
- **The look is a layer, not part of the UI.** A2UI v1.0 dropped theming so the host owns the
  look; json-render leaves it to each catalog's components. We took the middle: screens ask for
  meanings (`variant: "card"`, `tone: "warn"`), and the person's theme answers. Precedence is
  node style, then variant, then parts, then tokens. Nothing else cascades.
- **Errors are sentences.** A bound value of the wrong shape used to draw nothing. Now it says
  `table t: rows should be a list, but it's an object`, in the reply, the state and the status
  line. It costs about 13% of flat-out push speed (5,800 to 5,000 a second); see the guide.
- **Behaviour goes in components, never in the JSON.** Submitting a form is a `do` sequence of
  writes, not a script. Rejecting a bad amount is a property of the input (`accept: "number"`),
  not a condition in the action. When a screen seems to need logic, the answer is a component
  behaviour or a helper process, never an expression language.
- **Helpers are processes.** The ledger's balance, categories and running chart come from
  `books.py`, which reads the store over the socket (`$FICTTY`) and streams a summary back. The
  runtime stays generic; the app's arithmetic lives in the app.
- **Esc always leaves an editor,** so every screen can use it for "back". Inside an editor Ctrl-C
  copies; quitting is from outside, so a note can't be lost by a stray key.
- **Agent text edits are edits.** A `put` of a string, or an `edit` of ranges, goes through a view
  of its own, so the person's caret moves with the text and one undo takes it back.
- **Time comes from the runtime as data.** The log's clock time travels inside the message, so
  `update` stays pure and a replay shows the same times.

## Data decisions

- **Denormalised documents.** Each entry is whatever shape the agent or command produced, whole.
  A command's JSON drops straight in, and reading an entry gives an agent all of it. This matches
  A2UI's data model.
- **Shapes belong to components, not to the store.** There's no schema; each component says what
  it reads at its bind, and the problem checks hold it to that.
- **Derived values are never stored.** Totals, the selected row and sorting are computed from the
  one copy, or come from a helper that writes them under a key of its own.
- **Values keep their type.** A placeholder alone in a string keeps the value's type; text is never
  guessed into a number. Guessing turned `$id` values and a payee called "2024" into numbers.
  Components that add up read numbers out of strings instead.
- **New ids come from the store.** `{$id}` is built from the sum of every revision, which only goes
  up and survives a restart with the store.
- **One revision per entry, for now.** Two agents editing different rows of one list collide.
  Collections keyed by id, writes by path and a revision per record fix that; they wait for the
  next round (see the plan).

## What we learned

- **A revision per keystroke makes `--if-rev` strict for text.** One typed sentence took a note
  from revision 1 to 47. An agent that reads, thinks and then writes to a note someone is typing
  in will be refused nearly every time. Text needs edits that are rebased onto the person's
  typing (caretline can map edits through, as it does for live editors), with refusal kept for
  values.
- **The agent can't tell when the person has done something.** In the live session the person had
  to say "I wrote something" before the agent read it. `watch` is the most important missing
  piece of the loop.
- **Feedback has to show where the person is looking.** The showcase hides the status line, so a
  refused Enter said nothing. Errors about a field belong on the field.
- **Log what happened, not what was asked.** The activity log was written before a write was
  tried, so a refused write looked like one that landed. It now logs after, with the outcome.
- **The logical clock is the wrong clock for people.** `now_ms` since start read as `397:27`
  after a few hours. People need clock time; replays need it carried in the message.
- **Forms need validation, and data-only JSON can't express it.** A typed `3-` was booked and
  silently skipped by the totals. The fix was a component behaviour (`accept`). Dates and choices
  from a list will want the same.
- **Every view needs the store's editors created after any change.** An editor that follows a
  selection had no entry until the next push; editors are now set up at the end of every update.
- **Helpers must work without a screen.** `fictty render` ran `books.py`, which waited for a
  socket that wasn't there and hung the render. Helpers now print an empty answer when they can't
  read.
- **Template markup needs escaping.** `<<<` inside markup left a stray `</>` on screen. There is
  no escape for `<` in text yet.
- **Wide screens and small ones.** The showcase was built at 160×46. The home screen degrades at
  80×24; the denser screens need layout rules (flexbox) rather than fixed sizes.
- **Demo windows are part of the work.** WezTerm opened at 80×19 by default, and zsh doesn't
  word-split `$VARS` in `env`. Launch with an explicit size and set the socket inline.

## After the merge: putting it in people's hands

Shipping the showcase as a one-liner taught a few things the build didn't:

- **`curl | sh` hides the terminal.** The script's input is the pipe, and the generic `/dev/tty`
  isn't enough on macOS: kqueue can't wait on it, so the screen hung blank on its first read.
  The script hands over the terminal's own device (`ps -o tty=`), and fictty refuses a
  `/dev/tty` input on macOS with a sentence instead of spinning.
- **Whatever a script prints before a full-screen app is gone.** A "paste this in a second
  terminal" line was wiped at once. Anything a person needs while the screen runs has to be on
  the screen: the Agent screen's commands are a list you click to copy.
- **A screen can't be selected from while it has the mouse.** Hence a `copy` action through the
  terminal (OSC 52), and `$self`, so the copied line reaches this screen from any terminal.
- **A demo link has to mean one thing forever.** Each post's demo is a frozen script pinned to
  a commit, with its own folder, store and socket, so later work can't change it and two demos
  never collide.
- **Test it the way a reader runs it.** Every one of these bugs passed a direct run and failed
  through a pipe in a fresh terminal; `scripts/try-test.py` does the latter.

## Open questions

- **Do people want this in the terminal?** No tool trial answers it. Put the showcase beside a
  browser artifact for the same task, in front of a few people who supervise agents.
- **How much of json-render's format do we accept?** Agents already know it. An adapter onto our
  nodes is cheap; full compatibility is not.
- **Where does validation stop?** `accept` covers numbers. Dates, choices and required fields fit
  the same shape. Anything conditional across fields belongs to a helper process.
- **Who owns a theme:** the person, the project, or both, merged? Today it's a file passed to `run`.
