# Only the building blocks: the demo audit

Written on 17 October 2026, after step 3 and step 4's first half had landed. Aram's rule for the
demos, in his words: "The goal here with these demos is to only show what's possible within the
building blocks without having to do any special workarounds. The vocabulary, the units, the
styling, the JSON, all of that should be very straightforward and simple and very solid
abstractions so that agents can control it intuitively."

So every demo was read the way an agent new to fictty would read it: for every node, is this how
you'd say it, or is it there to get around something? Where a workaround was forced, the
building blocks were missing something and got it; where it wasn't, the demo was fixed. This is
what was found, what changed, what's left, and what comes next.

## How to spot a workaround

The signs, from this audit; check a screen against them before calling it done.

- **An empty text.** `{"type": "text", "text": ""}` pushing things apart or down is a spacer:
  the container should say it (`justify`, `gap`, `margin`, `self`).
- **Two nodes for one thing,** shown and hidden in turn (a Play button and a Pause button, a lit
  ♥ and an unlit one): one node with a state (`on`) and a template for what changes.
- **A value kept twice.** A count stored beside the list it counts, a flag copied into a second
  entry so a template can see it: the template should read the one source (`{$rows}`,
  `{$store.key}`).
- **A node bound somewhere only to reach a value** it doesn't show: the same.
- **A variant per state** (`lit`, `dim`) that the screen swaps by hand: a state in the theme
  (`on`, `selected`), styled by the variant's own `parts`.
- **Moving the focus to make something happen** (focus a list, press ↓, focus back): an action
  that names its target (`{"select": "next", "in": "tracks"}`).
- **A magic number tied to another node's look** (a margin as tall as a neighbour's title): a
  smell, sometimes acceptable; say why in a comment.
- **A button that does nothing,** or a key that does what its button doesn't: one action, used by
  both.
- **A demo's comment saying "trick".** The comments at the top of each screen in
  `examples/window/screens.py` say what's still faked; keep them true.

## What Nocturne needed (commit `0ce945a`)

Nocturne (`examples/nocturne/`) had the most: it's the screen that goes furthest.

| workaround | what it was | the building block now |
|---|---|---|
| Play and Pause | two buttons, one shown at a time by `when` | `on`, a state like `hover` or `selected` that holds while its condition does (`"on": "playing"`), styled by the theme's `button.on` or a variant's; the label a template on the store (`{$store.playing|true=❚❚ Pause;*=▶︎ Play}`) |
| ♥ and ⇄ lit | two buttons each, a `lit` variant on one | one button each, `"on": "loved/{id}"` and `"on": "shuffle"`, lit by the `ghost` variant's `parts.button.on` |
| NOW PLAYING / PAUSED | two texts by `when` | one text reading `{$store.playing}` beside the album it's bound to: `{$store.key}` in any text, label or title |
| headings per place | a text per place, by `when` | one text bound to the chosen place (`@nav`), its heading in the place's own data |
| n, p, ⏮︎, ⏭︎ | moved the focus to Up Next, pressed a key, moved it back | `{"select": "next", "in": "tracks"}` (and `"prev"`): a list moved a row, the focus where it was |
| the shelf's cards | padding and a radius set by hand on the template, and a glow clipped square inside | a variant's `parts` (`"shelf": {"parts": {"row": {…}}}`): a variant styles the parts inside its node, by state, so the shelf's rows are cards and Up Next's stay rows |
| spacers | empty texts between groups | `justify: "between"` |

## What the other demos needed (commit `4bc3576`)

| demo | workaround | the building block now |
|---|---|---|
| wringer `settings` | the radio group was tabs with three empty pages | tabs count their choices by their titles; a tab without a page shows nothing, so tabs alone are a segmented control or radio group (`Node::page`); more pages than tabs is a problem |
| wringer `kanban` | each column's count stored beside its cards (`n`) | `{$rows}` in a list's, table's or tree's title: how many rows it shows after its filter. Its own name because inside an `each`, `{$count}` is the `each`'s |
| Kestrel | tabs bound to `inbox_count` only so a title could show the unread count | tab titles (and the bar's `lead` and `trail`) take `{$store.key}`: `"Inbox {$store.unread}"` |

And the demos' own slips, fixed without new building blocks: empty spacer texts in the wringer's
`player` (now `justify: "center"`) and in Kestrel's and `mail`'s button rows; chat's Send button
did nothing (it now runs the composer's Enter action, one action for both).

## Looked at and kept

- **Chat's bubble** is a column held to 44 cells (`max`) holding the name line and the bubble,
  each aligned to its side with `self`. It reads like a workaround for the measuring gap below,
  but it isn't one: the column is there for the name line anyway, and this is how you'd write it.
- **The calendar's hour column** has a one-row top `margin` so its times line up with the day
  columns, which start with a bordered title. A number tied to another node's look; acceptable,
  and commented.
- **Daybook's zen mode** pushes the page down with an empty text six rows tall shown only in
  zen (`examples/journal/journal.json`). This is a spacer, but the clean answer is layout that
  changes with state, which nothing has yet; see "Later" below.
- **The settings form's remaining tricks** (checkboxes as list rows that toggle, the select as a
  pop-up list, the slider as a gauge) are recipes and `slider`, planned for phase 2, and the
  screen says so.

## Known gaps in the foundations

- ~~**The window draws trees flat.**~~ Done in 4c: indent, fold glyphs, a press on the glyph
  folds.
- ~~**Tables in the window share every column equally.**~~ Done in 4c: widths, `align`, `heat`,
  totals, a column's `cell` template; the `sheet` rebuilt.
- **A row template is filled by a JSON round trip** for every row built, every frame: node rows
  cost twice item text (10.9 ms a frame against 5.4 for a 50,000-row list, headless, release).
  Step 4's "done when" asks for `fit` rows at a uniform list's frame time.
- **A `fit` node held by `max` in a `fit` row is measured one line tall** (Taffy measures a leaf
  under max-content without its `max`; GPUI's text cache hid it). Written up in
  [rows-step.md](rows-step.md); nothing needs it now that chat's bubble is written as above.
- **A row template's inner nodes aren't in the readback,** only the row's text: an agent can't
  read a card's ♡ rect except through `Headless::at_part`.

## Next steps, in order

1. ~~**4c, trees and tables in the window**~~ (done 17 October 2026; [rows-step.md](rows-step.md) has the detail): a
   tree's indent and fold glyph drawn by the runtime around its row (item text or `row`
   template), a press on the glyph folds; a table's column widths, alignment, heat and totals,
   its header over the same columns, and a column's `cell` as a node template. The wringer's
   `sheet` rebuilt without placeholders; Kestrel's Stock as it is, checked.
2. **The rest of 4d:** Kestrel's inbox and Jobs board as row templates (an unread dot, a
   two-line card), checked against the list above.
3. **Row templates without the JSON round trip:** fill a template's strings in place, so node
   rows cost what item text does. Measured before and after (rows-step.md's way).
4. **Step 3's look, with Aram:** the wringer's `settings`, `landing`, `calendar`, `chat` and
   `kanban` in three themes, in a real window, as screenshots; Kestrel's settings dialog; the
   frame time against the baseline at one refresh rate.
5. **Later, decided with Aram first:** layout that changes with state (Daybook's zen spacer:
   perhaps `on` styling `margin` and `pad` through a variant's parts, as it styles colour now);
   the `fit`-under-`max` measurement, in Taffy or by measuring `fit` nodes ourselves.

Each lands as the steps before did: tests in `tests/window.rs` (and `wringer.rs`, `demo.rs`,
`nocturne.rs` for the screens), the release fuzz clean at 400 runs with nothing else building,
`docs/guide.md` and the fictty-ui skill kept true, and every demo it touches read again against
the signs above.
