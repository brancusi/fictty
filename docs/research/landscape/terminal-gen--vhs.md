# VHS and `.tape` scripts

- **Maker:** Charm (charmbracelet)
- **URL:** https://github.com/charmbracelet/vhs
- **Status (2026-10-10):** about 21,100 stars, MIT, Go. v0.12.1 on 24 September 2026. Mature and
  maintained. TUIOS has its own `.tape` language with recording and replay, borrowed in spirit.

## What it is

"Your CLI home video recorder." A `.tape` file is a script (`Output demo.gif`, `Type "ls"`,
`Enter`, `Sleep 500ms`, `Screenshot`, `Set FontSize`) that VHS plays in a headless terminal
(ttyd plus a browser) to produce GIFs, MP4s, PNGs, or text output for golden-file tests.

## The problem it's solving

Making reproducible terminal demos and screenshots, and integration tests that compare text
output.

## Its path / bet

A tiny declarative script language for terminal sessions, run headless and deterministic enough
for CI.

## How it works

Parse the tape, start a shell in a headless terminal, replay keystrokes with timing, capture
frames, encode. `.txt` or `.ascii` output gives a text transcript usable as a golden file.

## Strengths

- The standard for terminal demo GIFs; agents already know the tape syntax.
- Deterministic replay of input; text output for tests.

## Weaknesses / limits

- One-way: record and replay, no reading the screen mid-run to decide the next key.
- Heavy runtime (ttyd, headless Chromium, ffmpeg).
- Not built for agents; useful to them only for demos and golden tests.

## Relation to fictty

Complement. It is how fictty's README GIFs and the blog's terminal captures should be made, and a
reference for what a replay script looks like to a person.

## Could fictty adopt it instead of building?

For demos, yes, adopt it. For fictty's history and replay, no: VHS replays keystrokes into a
program, fictty will replay messages into a state, which is exact and needs no emulator.

## What fictty should take from it

- Use VHS (or `fictty render` frames) for the site's demos.
- A tape-like text format for fictty history export: one readable line per input (`key j`,
  `push ui.json`, `patch header`), so a person can read and edit a session as easily as a tape.

## Sources

- https://github.com/charmbracelet/vhs (README, release v0.12.1)
- https://github.com/Gaurav-Gosain/tuios (tape scripting)
