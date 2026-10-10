# agent-tty (Coder)

- **Maker:** Coder
- **URL:** https://github.com/coder/agent-tty
- **Status (2026-10-10):** 30 stars, Apache-2.0, TypeScript, Node 24 or later. Created 19 March
  2026, last push 24 September 2026; npm 0.5.0 (23 June 2026), about 185 downloads last month.

## What it is

"Give your AI agent a real terminal it can drive, and get back reviewable snapshots, screenshots,
and recordings of everything it did." A PTY driver whose distinguishing feature is proof: every
session is recorded and can be exported as a text snapshot, PNG, asciicast or WebM.

## The problem it's solving

You can't take an agent's word that it tested the TUI it just changed. agent-tty makes the agent
attach evidence a person can check at a glance, and gives a live dashboard to watch sessions.

## Its path / bet

Playwright's loop for terminals (create, run, wait, snapshot, screenshot, destroy), with
reviewability as the product. Screenshots and video render through Playwright's Chromium.

## How it works

JSON output on every command; `wait --text` and `--screen-stable-ms`; `send-keys` with chords;
`record export --format webm`; a "doctor" check; dogfood transcripts of Codex and Claude using it.

## Strengths

- Clear thesis (evidence, not claims) and good agent-facing CLI design.
- Backed by a company that sells agent infrastructure.

## Weaknesses / limits

- Heavy dependencies (Node 24, Chromium for pictures).
- Same semantic blindness as every driver.
- Little adoption so far.

## Relation to fictty

Complement and inspire. fictty's planned history (record every input, export, replay) is the same
idea from inside the runtime: an agent session on a fictty screen should produce a replayable
record without a separate recorder.

## Could fictty adopt it instead of building?

No for the screen. For demo and PR evidence, using agent-tty on a fictty screen works today, but
fictty's own history will be more exact (states, not pixels).

## What fictty should take from it

- Make "attach proof to the PR" a first-class export of fictty history: a `.cast`, a text
  transcript of frames, and a picture.
- A `doctor` command that checks the terminal (kitty graphics, mouse, size) and says what will
  degrade.

## Sources

- https://github.com/coder/agent-tty (README)
- https://registry.npmjs.org/agent-tty
- https://api.github.com/repos/coder/agent-tty
