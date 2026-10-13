# Expressive screens: the layout and style stack

A proposal, 13 October 2026. Status: **decided, 13 October 2026** (below). It extends phase 1 of
[native.md](native.md) and reorders its steps from 2 on; native.md's "Next, step by step" is the
working list. The screens and themes it's measured against are in
[examples/window/](../../examples/window/README.md).

**The goal.** People and agents build rich, expressive screens in fictty, and a theme can change
how those screens look so completely that two apps built from the same components look nothing
alike: different type, shape, depth, colour and motion, not just different colours. A screen is
still data, an agent can still read back every part of it, and the core is still an Elm loop.

## How we got here

| when | what happened | what it taught us |
|---|---|---|
| to 10 Oct | fictty grew as a terminal runtime on ratatui: a UI as JSON, pushed and patched over a socket, read back by agents | The agent loop (push, patch, read back, answer in place) is the product. Nothing we tried had it ([adopt-or-build.md](adopt-or-build.md)). |
| 11 Oct | The store and themes landed ([state-and-theme.md](state-and-theme.md)): data apart from the UI, and a theme of tokens, parts and variants | The look can be its own layer. The UI says what things mean; the theme decides how they look. |
| 11 Oct | fictty left the terminal ([native.md](native.md)). The GPUI spike held 120 fps on every stress scenario | The terminal was what held the look back: one font and one size, whole cells, no real images, no motion. |
| 12 Oct | Phase 1, step 1 (layout) on `window-layout`, [PR #1](https://github.com/brancusi/fictty/pull/1): `px` sizes, `min`/`max`, `pad`, `align`, floats, modals with a scrim | The window can lay out everything the terminal could, and tests prove it under GPUI's test platform. |
| 12–13 Oct | Demos in a real window: a mail client, a live ops dashboard fed by a stream, and a `night-city` theme (hard edges, acid yellow on near-black) | Screens built from list-like parts already look finished. Speed is not the limit: 66 fps through 60 theme swaps a second, and 16,000 patches a second sent flat out. |
| 13 Oct | "Through the wringer": nine screens a designer might hand over (settings, chat, kanban, player, landing, calendar, spreadsheet, plus mail and ops), four of them in six themes | It showed where the line is (below). |
| 13 Oct | Aram asked what the stack for layout and styling is ("do we have something like HTML and CSS?"), and whether a framework could fill the gaps | This document. |

### What the wringer found

**Easy, no tricks:** panes, lists, tables, editors, forms, dialogs and live dashboards. Themes did
best of all: the same screens, unchanged, read correctly as cyberpunk, brutalist, soft pastel,
paper, phosphor and glass.

**Took tricks:**
- a checkbox was a list row that toggles;
- a radio group was tabs with empty pages;
- a dropdown was a pop-up list;
- chat bubbles were a background colour behind the text;
- the calendar needed a script to work out the gaps between events;
- the kanban needed its data reshaped into one store entry per column.

**Couldn't be done:**

1. **Nothing sizes to its content.** Every size is a share, a fixed size or a percentage, so
   pricing cards stretch to fill the screen.
2. **Only lists, tables and editors scroll.** A long form or page is cut off at the window's edge.
3. **A row is a line of text, not a box.** Kanban cards, real chat bubbles, and rows with avatars or
   buttons can't be built. This is the biggest single limit.
4. **There is one text size.** There are no headlines, hero numbers or captions.
5. **Nothing wraps.** Tags and card grids can't reflow as the window narrows.
6. **There's no margin, and no placement across a container.** Children always stretch.
7. **Nothing is placed by data.** A float sits at one of nine anchor points, never at a position
   taken from the store.
8. **There's no drag and drop.** Hover is thin, and there's no motion.
9. **A key can't change a number,** so sliders and steppers need the embedding app.
10. **A wrong name fails silently.** A misspelt `row_variant`, for example, is ignored without a
    word.

Charts, images, cards and heat colours exist in the model but still draw as placeholders in the
window. Phase 1 already plans them; they aren't gaps in the model.

## What stays fixed

These rules decide what we build and what we refuse:

- **A UI is data, never code.** There are no expressions, selectors, `calc()` or scripts. A theme is
  data too.
- **The UI arranges and the theme styles.** A node says what it is and where it goes (`size`,
  `gap`, `float`), plus what it means (`variant`, `tone`). The theme says how everything looks.
  This split is what lets one screen wear six themes.
- **Behaviour belongs in the runtime.** Scrolling, dragging, hover and motion are state in
  `State`, changed by `Msg`s, with time taken from `State::now_ms`. If it isn't in the state,
  agents can't read it, it can't be replayed and it can't be tested.
- **Agents see everything.** Every feature appears in `fictty screen --nodes` or
  `fictty get --state`, including the style each node ended up with.
- **The window is the target.** The terminal goes at the end of phase 1 (native.md, step 8).
  Everything below is designed for the window. The terminal ignores what it can't draw until then,
  and we spend nothing on terminal-only work.

## Adopt or build

**We are not adopting a whole UI framework.** We looked at a webview (HTML and CSS),
Dioxus/Blitz (HTML and CSS rendered natively in Rust), Iced, and Textual-style CSS. Each would
either:

- replace the structural readback agents rely on with a DOM that agents would have to scrape;
- bring in selectors, a cascade and scripting, which break "data, never code";
- or swap GPUI for something that has proven no faster (Iced is the fallback if GPUI ever costs
  more than it gives).

What we actually want from those frameworks is their layout and their painting, and GPUI already
gives us both.

**We adopt what GPUI already has.** It is in the build, pinned at `gpui = "=0.2.2"`:

| need | GPUI 0.2.2 |
|---|---|
| Layout | Taffy 0.9: flexbox and CSS grid. Sizes can be `auto`; margins, padding, `align-items`/`align-self`/`justify-content`, `flex-wrap`, grid columns and spans, absolute positions with insets, overflow and scrolling, aspect ratio |
| Text | a font system that loads font files at runtime (`add_fonts`), any size, weight and style, line height, alignment, wrapping, line clamping, underline and strikethrough |
| Paint | per-corner radii, several box shadows (blur and spread), solid and dashed borders, two-stop linear gradients, a slash pattern, opacity, cursors |
| Pictures | `img()` for raster images, `svg()` for vector images, and paths for drawing charts |
| The window | opaque, transparent or blurred backgrounds, for glass themes |

**What GPUI doesn't have**, which bounds how far a theme can go: radial gradients, a blur behind an
element inside the window, custom shaders and filters. A tiled image (phase 3) stands in for grain
or noise texture.

**One correction to what I said in chat on 13 October.** I proposed moving layout into
`fictty-core` on Taffy so that the terminal and the window would share one layout engine. Once the
terminal goes, that duplication goes with it: GPUI's Taffy is then the only layout. So layout stays
in the window, and it hands its rectangles back to the engine, as it does now. The core keeps what
it should own: reading sizes (`Size::measure`), and resolving each node's style from the theme
(below), so that the window and the readback agree on what was asked for.

**What we build** is the vocabulary, and the behaviour that has to live in `State`. That means:
- mapping the new layout fields onto GPUI's style;
- themes as whole styles, and style resolution in the core;
- rows that are trees of nodes;
- scrolling any container;
- drag and drop;
- declared motion;
- a few store actions and primitives.

## The stack

```
  UI value (the agent's)        Theme (the person's or project's)       Store (shared)
  structure: nodes, size,       tokens, scales (space, radius,          the app's data,
  gap, float, scroll, wrap,     type, depth, motion), parts as          versioned
  variant, tone                 whole styles with states, variants,
        │                       fonts, textures
        │                              │                                     │
        └──────────────┬───────────────┴─────────────────────────────────────┘
                       ▼
     fictty-core  ·  pure, tested without a window
     templates, when, each   →  the shown tree
     style resolution        →  each node's resolved style
                                (part, then variant, then the node's own; states from State)
     State: focus, selection, hover, scroll, drag, the motion clock
                       │
                       ▼
     fictty window  ·  GPUI
     resolved style → GPUI Style → Taffy layout → text and paint → Metal
     rectangles, hits and content sizes  →  back to the engine as `Drawn`
                       │
                       ▼
     readback  ·  `fictty screen --nodes` (kind, id, text, rectangle, resolved style)
                  `fictty get --state` (scroll, drag, focus, motion in progress)
```

Compared with the web:

| web | fictty |
|---|---|
| HTML | the UI value: a tree from a fixed set of components |
| layout CSS (flex, grid, margin, overflow) | fields on the node, mapped onto GPUI's Taffy |
| CSS custom properties | theme tokens and scales |
| component CSS, `:hover`, `:focus` | theme parts as whole styles, with states (`button.hover`) |
| classes | `variant` |
| inline `style` | the node's own `style` |
| the cascade and selectors | none. A node's style is its part, then its variant, then its own `style`. Nothing else applies |
| JavaScript | none. Behaviour is the runtime's, and the data says which behaviour to use |
| DevTools' computed style | `fictty screen --nodes --style` |

## The vocabulary

Field names here are proposals. Each one maps onto something GPUI already does, unless it's
marked as new behaviour.

### Layout, on nodes

| field | meaning | maps to |
|---|---|---|
| `size: "fit"` | as big as its content, along the parent's direction | `auto` with text measured |
| `min`, `max` | any unit, not only cells: `"240px"`, `"40%"`, `12` | min and max size |
| `margin` | `1`, `[x, y]` or `[top, right, bottom, left]`, in cells or `"8px"` | margin |
| `pad` | the same four-sided forms and units (today it's cells only) | padding |
| `self` | `start`, `center`, `end` or `stretch` across the parent | `align-self` |
| `aspect` | `"16/9"`, `1` | aspect ratio |

### On containers

| field | meaning | maps to |
|---|---|---|
| `gap` | `1` or `[x, y]`, in any unit | gap |
| `justify` | `start`, `center`, `end`, `between`, `around` | `justify-content` |
| `items` | `start`, `center`, `end`, `stretch` (the default), `baseline` | `align-items` |
| `wrap: true` | children flow onto the next line when they don't fit | `flex-wrap` |
| `scroll: true` | the container scrolls along its direction (**new behaviour**: below) | overflow, plus our own offset |
| `{"type": "grid", "cols": 3}` or `"cols": ["200px", "*", "2*"]` | a grid, with `span` on children | grid columns and spans |

### Floats

A float keeps its nine anchor points and `margin`, and gains `x` and `y` (any unit, and templates,
so `"x": "{$item.left}"` places it from data). A calendar's events, pins on a map and markers on a
timeline then need no gap-computing script.

### Content

- **Rows are nodes (new behaviour).** A `list`, `table` or `tree` can take `row` as a node
  template, the way `each` does, filled from `$item`. A row can then hold an avatar, a stack of
  text, a badge and a button.
  - **Selection, focus, hits and keys** work as today, with clicks on the row's own buttons
    routed to them.
  - **Rows stay virtualised:** only the rows in view are built. A row's height is the template's
    `size`, or `"fit"`, measured as each row first shows.
  - The kanban card and the chat bubble become a list with a row template.
- **`each` gains `$index`** and `$count`, so data no longer has to be shaped for the screen.
- **Images and icons.** `image` is already planned (phase 3). `icon` is a new primitive: a name
  from a bundled open icon set (Lucide, ISC licence), coloured and sized by the theme, so a UI
  never ships SVG paths.

### Type

- **A type scale in the theme.** It names roles, each with a font, size, weight, line height and
  letter spacing:

  ```json
  "type": {
    "caption": {"size": 11, "weight": 500, "tracking": 0.4},
    "body":    {"size": 14},
    "title":   {"size": 20, "weight": 600},
    "display": {"font": "display", "size": 56, "weight": 800, "tracking": -1.5}
  }
  ```

  A node asks for a role (`"type": "display"`), and markup asks with `<display>…</>`. Like
  `variant`, the UI names a role and the theme decides what it looks like. The existing `scale`
  becomes a shorthand for a role.
- **Fonts in the theme:** `"fonts": {"ui": "Inter", "mono": "JetBrains Mono", "display": "fonts/Anton.ttf"}`.
  A path loads that file at runtime. Distinct type is the strongest single thing that makes a theme
  look unique.
- **Text fields:** `wrap: true`, `lines: 3` (clamped, ending in `…`), and our own ellipsis (step 2
  of native.md). Editors stay monospace until phase 4.

### Looks: themes as whole styles

Today a part is a short ink (`"button.focused": "bg+on-accent+b"`). That form stays. A part can
also be an object, a whole style:

```json
"parts": {
  "button": {
    "fg": "fg", "bg": "panel", "type": "body", "pad": ["14px", "8px"],
    "radius": "r2", "border": {"width": 1, "color": "border"}, "shadow": "e1",
    "motion": "fast"
  },
  "button.hover":   {"bg": "hover", "shadow": "e2"},
  "button.focused": {"bg": "accent", "fg": "bg", "ring": {"width": 2, "offset": 2, "color": "accent"}},
  "tabs":           {"shape": "underline"},
  "card":           {"bg": {"from": "panel", "to": "bg", "angle": 160}, "radius": "r3", "shadow": "e3"}
}
```

- **Tokens gain scales:** `space` (`s1`, `s2`, …), `radius` (`r1`, …), `depth` (`e1`, … as
  shadow lists, including a glow), and `motion` (`fast: 120ms ease-out`). Parts refer to them by
  name, so a theme changes its whole feel in a few lines.
- **States are parts with suffixes:** `.hover`, `.focused`, `.active`, `.selected`, `.disabled`,
  `.blur`. This follows the naming parts already use (`row.selected`, `button.focused`). The state
  comes from `State`, so agents see it too.
- **Shapes a component can take** are a part property, so the theme picks them:
  - tabs: `underline`, `pill`, `segment` or `plain`;
  - a scrollbar: `thin`, `overlay` or `none`;
  - a check: its glyphs;
  - a focus ring: width, offset, colour and glow;
  - an edge: `rounded`, `square`, `double` or `thick`. The uncommitted `edge` token from the
    night-city work lands here.
- **Variants become whole styles too.** `"variant": "card"` gets the variant's style, padding,
  radius, depth and type. A variant styles; it never arranges. It may set `pad`, but never `size`,
  `gap` or position.
- **Textures and the window:** a part's `bg` can be a tiled image once images land, and the theme
  can ask for a transparent or blurred window.
- **Resolution is in the core.** For each shown node, the core works out its style:
  - from its part, plus the state suffixes that hold;
  - then its variant;
  - then its own `style`;
  - with token and scale names filled in.

  The window maps that one value onto GPUI, the readback prints it, and tests assert on it without
  a window. Resolution is cached per theme revision, so it costs nothing per frame for nodes that
  didn't change.
- **Wrong names are problems.** An unknown part, token, scale entry, role, variant or row variant
  is reported in `problems` and in the reply to the push or theme, in words. Nothing fails
  silently.

### Interaction (new behaviour)

- **Scrolling any container.** `scroll: true` keeps an offset in `State` for the node's id, as
  lists do:
  - the wheel and keys (page up and down) move it;
  - focus moving into a child scrolls it into view;
  - the readback shows the offset and the content's size.

  GPUI's own scroll handles are not used, because their state would be hidden from `State`.
- **Drag and drop.**
  - **Dragging:** a node says what it carries (`"drag": "{$item.id}"`). While it moves, `State`
    holds `$drag`: what is carried, where it came from, and the node under the mouse.
  - **Dropping:** a node says what it takes (`"drop": {"accept": "card", "on": <action>}`).
  - **Moving data:** a new store action, `move`, takes an item from one array and puts it in
    another at an index.
  - **Looks:** theme parts `drag.ghost` and `drop.hover` style both ends.
  - **Files:** phase 2's file drop zones are the same mechanism, with files as what's carried.
- **Numbers and controls.**
  - A store `add` action takes a step, a minimum and a maximum.
  - A `slider` primitive (dragging along a track sets a number) is a new primitive, justified by
    behaviour.
  - Checkbox, radio and select become project recipes on `toggle`, `put` and a float, rather than
    tricks, and are written down as such.
- **Hover everywhere:** every component's part has `.hover`, and a part can set the cursor.

### Motion (new behaviour)

Motion is declared, never scripted:

- **Transitions:** a part names which properties ease (`"motion": "fast"`, or
  `{"props": ["bg", "shadow"], "ms": 160, "ease": "out"}`). When a node's resolved style changes
  (hover, focus, selection, a theme swap), the view moves between the old and new values.
- **Enter and leave:** a float, layer or toast can `enter` and `leave` with `fade`, `slide`
  (with an edge) or `scale`.
- **Values:** a `big` number or a gauge can ease to its new value.
- **Time comes from the state, not from GPUI's animation API.**
  - `State` notes when each node's state last changed.
  - The view interpolates from that time and `State::now_ms`.
  - The engine keeps waking the window while motion is running (`Drawn`'s wake time already
    exists).

  So motion replays exactly, tests can step through it, and `get --state` shows it running.
  GPUI's animation API keeps its own clock outside our state, so we don't use it.

What makes a screen look unique, then, is all in the theme:
- type (fonts and the scale);
- shape (radii, edges, component shapes);
- depth (shadows and glows);
- colour (tokens and gradients);
- motion (durations and easing);
- texture.

Variants carry the same layers to single nodes. None of it needs the UI to change.

## Out of scope, still

- Free-form canvases, and positioning by dragging things around in pixels.
- Selectors, a cascade, `calc()`, expressions, scripts or custom shaders.
- Marketing pages built as long scrolling documents. These become possible with `scroll`, `fit`,
  type roles and images, but they're not what we design for.
- Proportional editors (phase 4, which needs a caretline release first).

## Order of work

This reorders native.md from step 2 on. Each step ends the way native.md's steps do:
- tests in the window;
- grid twins deleted;
- clean fuzz runs;
- the screens that use it looked at in a real window.

New fields are window-only, and the terminal ignores them until it goes.

1. **Step 2 becomes text and themes as styles (M–L).** Everything native.md already lists for step
   2 (ellipsis, `keep`, `variant`, `tone`, `edge`, `backdrop`, the status bar), plus:
   - style resolution in the core and `--style` in the readback;
   - parts as whole styles, with states;
   - scales;
   - type roles and theme fonts;
   - `wrap` and `lines` on text;
   - `problems` for wrong names.

   It is done when the wringer's six themes are rewritten as whole styles and look clearly more
   different from each other than they do today, in screenshots.
2. **New step, the layout vocabulary (M).**
   - `fit`, `margin`, four-sided `pad`, `min`/`max` in any unit, `self`, `aspect`;
   - `justify`, `items`, `wrap`, `grid` and `span`;
   - float `x`/`y` from data;
   - `$index` and `$count`;
   - `scroll: true`, with its offset in `State`.

   It is done when the pricing page, the settings form and the calendar are rebuilt without tricks.
3. **Step 3 grows: rows are nodes (L).** The lists, trees and tables step as native.md has it, plus
   row templates as nodes, virtualised with `fit` heights. It is done when the kanban and chat are
   rebuilt as real cards and bubbles.
4. **Steps 4 to 8 as native.md has them** (editors, charts, toasts and layers, embedding, the
   terminal goes). Two additions:
   - `icon` joins step 5;
   - `enter`/`leave` motion joins step 6 (toasts and layers).
5. **Phase 2 adds drag and drop beside file drops:**
   - `drag`, `drop` and `$drag`;
   - the `move`, `add` and `toggle` store actions;
   - `slider`.

   It is done when the kanban's cards move between columns by mouse and by key.
6. **Phase 3 keeps media and takes transitions:** part transitions and easing values, on the
   state's clock.
7. **Phase 4 keeps proportional editors.** Its "theme kit" (type scale, spacing, radii, elevation)
   has moved into step 2.

**The whole plan is done when** the nine wringer screens run with no tricks. Each must be shown in
three themes that a person would take for three different apps, with screenshots, and an agent
must be able to read back every one of them node by node, with its resolved style.

## Risks

| risk | what we do |
|---|---|
| Scope creep towards a browser | The rules above are the line: no selectors, no cascade beyond part, variant and node, no expressions. Every new field must map onto something GPUI already does or onto behaviour in `State`. |
| `fit` text and ellipsis under Taffy's measuring (GPUI's `text_ellipsis` already broke this way) | We measure with GPUI's text system and cut ourselves (step 2), and test it in `in_window` at many widths. |
| Rich rows with `fit` heights and virtualisation | Each row is measured once, and the result cached by the row's data revision. Speed is checked with `examples/stress/play.sh`, before and after. |
| Style resolution every frame | It is cached per theme revision and node. The 66 fps theme-swap numbers are re-measured before and after. |
| GPUI changes underneath us | Only `crates/fictty` touches GPUI. The vocabulary is ours and maps onto it, so moving to Iced would mean a new mapping, not a new model. |
| Themes get harder to write | The short ink form stays. Scales let a theme change its feel in a few lines. `problems` catches typos. Each wringer theme also serves as a worked example. |

## Decided with Aram (13 October 2026)

1. **The direction stands:** themes as whole styles, a layout vocabulary on what GPUI already has
   (its Taffy, text and paint), rows as nodes, drag and motion declared in data. No other UI
   framework.
2. **Drop the terminal.** No more terminal work: new fields are window-only, the grid view
   (`view.rs`) isn't extended, and a grid test whose feature has landed in the window is moved
   and deleted, not kept in step. The terminal's code goes in native.md's last step of phase 1,
   once the grid tests that still guard features the window lacks have moved.
3. **The theme kit comes forward** from phase 4 into step 2.
4. **The wringer is in the repo** (`examples/window/`, with `open.sh`), and it's the test of
   done. Pushing its themes is part of the work: how far a theme can move a screen's character,
   and which subtle things (type, spacing rhythm, edges, depth, states) move it most. Its README
   lists what to try.
