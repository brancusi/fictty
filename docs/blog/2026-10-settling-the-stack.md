---
title: "Kicking the tyres: how fictty found its stack"
description: "Aram nearly shelved fictty. So we went looking for something that already did the job, found the one piece nobody has built, and settled the stack: ratatui, caretline, and a store that keeps everyone's work safe."
date: 2026-10-11T09:00:00Z
kicker: milestone
tokens:
  models: [claude-opus-5-5]
  from: 2026-10-10T18:39Z
  to: 2026-10-11T05:23Z
  note: The two context compactions in this window aren't in the transcript, so they aren't counted.
  phases:
    - { name: "Explore the options", calls: 73, input: 140879, cached: 8604896, output: 55878 }
    - { name: "Proof of concept and editor", calls: 76, input: 116329, cached: 17694377, output: 55077 }
    - { name: "The store: who owns what", calls: 29, input: 60425, cached: 9402228, output: 44210 }
    - { name: "Theme and store design", calls: 3, input: 4810, cached: 1055349, output: 4556 }
    - { name: "Proposal and overnight showcase", calls: 118, input: 208230, cached: 55899023, output: 139979 }
    - { name: "Morning play and four fixes", calls: 36, input: 79262, cached: 2943810, output: 20581 }
    - { name: "Notes and roadmap", calls: 20, input: 53175, cached: 2772413, output: 17526 }
    - { name: "Merge, post, skill, deploy", calls: 53, input: 90275, cached: 11011062, output: 49537 }
---

*A conversation between Aram ([@brancusi](https://github.com/brancusi)), who is building fictty, and
Tilde, the name Claude (Anthropic's model) goes by on this project; a tilde is home in a terminal.
The quotes are snippets from one working session on 10 and 11 October 2026, lightly edited. The
earlier posts asked [why the terminal](2026-10-why-the-terminal.md) and
[what agent UI is for](2026-10-disposable-interfaces.md). This one is about the tools.*

## The doubt

**Aram:** I'm not sure about the project anymore. I think we should explore existing options first.

**Tilde:** Then let's try the tools most likely to make fictty unnecessary. If one of them already
does the job, better to find out today than after another month of Rust.

So we kicked the tyres on three of them:

<ol class="route">
<li style="--c: var(--glass-rose)"><b>json-render,</b> on its Ink terminal renderer: agents write a JSON spec, the terminal draws it.</li>
<li style="--c: var(--glass-violet)"><b>OpenTUI,</b> the polished toolkit behind opencode.</li>
<li style="--c: var(--glass-cyan)"><b>ratatui</b> and its ecosystem, the engine fictty already used.</li>
</ol>

## What we found

json-render's format is genuinely good: dashboards look right with no design work, and screens
stream in as small patches. But on Ink there's no full screen, no scrolling, no mouse, and a cap of
30 redraws a second. And its form's Tab key was broken as published (a one-line fix we owe them
upstream). OpenTUI has everything Ink lacks: mouse, scroll boxes, images, Markdown, diffs, an
embedded terminal.

Then Aram named what all three had in common:

**Aram:** So basically we're missing the ability to push UI, patch that UI, and get state. And
therefore I can't really get an agent to drive the state of the user's TUI.

**Tilde:** That's the whole gap, in one sentence. Every one of these is a library for whoever
writes the app. None has a door an outside agent can use while the screen is running.

| | JSON format | Agent can attach | Full TUI |
|---|---|---|---|
| json-render | yes | no | no |
| OpenTUI | no | no | yes |
| ratatui | no | no | with crates |
| **fictty** | yes | **yes** | yes |

That loop is fictty. The question changed from *"should fictty exist?"* to *"which engine goes
under it?"*

## Settling the stack

OpenTUI looked lovely, so we compared it with ratatui honestly:

<figure class="bars">
<div class="rows">
<div class="group">GitHub stars, thousands</div>
<div class="bar" style="--v: 100; --c: var(--glass-cyan)"><span>ratatui</span><i></i><b>22.9k</b></div>
<div class="bar" style="--v: 59; --c: var(--glass-violet)"><span>OpenTUI</span><i></i><b>13.5k</b></div>
<div class="group">Contributors</div>
<div class="bar" style="--v: 100; --c: var(--glass-cyan)"><span>ratatui</span><i></i><b>~310</b></div>
<div class="bar" style="--v: 39; --c: var(--glass-violet)"><span>OpenTUI</span><i></i><b>~120</b></div>
<div class="group">Years of history</div>
<div class="bar" style="--v: 100; --c: var(--glass-cyan)"><span>ratatui</span><i></i><b>10 (as tui-rs from 2016)</b></div>
<div class="bar" style="--v: 13; --c: var(--glass-violet)"><span>OpenTUI</span><i></i><b>1¼</b></div>
</div>
<figcaption><b>Maturity, October 2026.</b> 6,310 crates depend on ratatui; it runs Codex CLI, yazi, atuin, gitui and many more. OpenTUI runs opencode.</figcaption>
</figure>

**Tilde:** ratatui is proven and fits fictty's own rule: redraw everything from one state, every
frame. What it lacks is a kit. Its ecosystem is 1,116 crates of one widget each, all styled
differently. That's a job fictty can do: wrap the best crates behind one JSON spec and one theme,
with OpenTUI's component list as the target.

For text we chose caretline over the popular tui-textarea. tui-textarea keeps its text inside the
widget, where nothing can save, read back or replay it. caretline keeps it in plain state, the
same way fictty does.

<figure class="stack">
<div class="layers">
<div class="layer" style="--c: var(--glass-rose)"><b>Agents</b><span>write JSON: push a screen, patch a node, write the store, read it all back</span></div>
<div class="layer" style="--c: var(--glass-cyan)"><b>fictty</b><span>the runtime: one state value, the store, themes, data sources, the kit</span></div>
<div class="layer" style="--c: var(--glass-amber)"><b>caretline</b><span>every text field and editor, with undo, carets and replay</span></div>
<div class="layer" style="--c: var(--glass-smoke)"><b>ratatui</b><span>drawing, plus the best ecosystem crates as the kit grows</span></div>
</div>
<figcaption><b>Where we landed.</b> One static binary, in Rust.</figcaption>
</figure>

## The question that shaped the design

Once agents could push screens and write text into them, Aram spotted the danger:

**Aram:** We need to figure out how we split UI state and app state, so we don't blow away any
data that was added by the user or other agents.

**Tilde:** Split it by who owns it, and give each owner its own rules.

<figure class="stack">
<div class="layers">
<div class="layer" style="--c: var(--glass-rose)"><b>UI · the agent's</b><span>push and patch replace it freely. It never holds content.</span></div>
<div class="layer" style="--c: var(--glass-cyan)"><b>Store · shared</b><span>versioned entries. A write can name the revision it read, and a stale one is refused. A push never touches it.</span></div>
<div class="layer" style="--c: var(--glass-amber)"><b>View · the person's</b><span>selection, carets, scroll, focus.</span></div>
<div class="layer" style="--c: var(--glass-violet)"><b>Theme · the person's</b><span>colours and looks. Screens ask for meanings like "card" or "warn", never colours.</span></div>
</div>
<figcaption><b>Who owns what.</b></figcaption>
</figure>

The details are in the [state and theme proposal](../design/state-and-theme.md).

## The overnight build

**Aram:** Get a working prototype with enough components to build a note-taking app or a financial
accounting app. Build a beautiful demo, like the one from OpenTUI. Have it ready for the morning.

**Tilde:** By morning there was a showcase: notes, a ledger, a live market, a gallery and an
agent's log. Every screen is a JSON file, and they share one store and three themes.

<figure class="shot">

![The fictty showcase menu: a glass wordmark built from coloured cells, a list of five chapters, and a description of the selected one](img/settling-the-stack/home.png)

<figcaption><b>The showcase.</b> Every screen is JSON.</figcaption>
</figure>

**Tilde:** The ledger is my favourite, because its JSON holds no arithmetic. A small helper
process reads the store and streams back the balance, the categories and the running chart.

<figure class="shot">

![The ledger screen in the glass theme: a table of entries with a total, a big balance figure, spending by category as bars, and a running-balance chart](img/settling-the-stack/ledger-glass.png)

<figcaption><b>The ledger.</b> The form books an entry; the summary comes from a helper process.</figcaption>
</figure>

<figure class="trio">
<div class="row">

![The same ledger in the paper theme, light grey with teal accents](img/settling-the-stack/ledger-paper.png)

![The same ledger in the phosphor theme, green on black](img/settling-the-stack/ledger-phosphor.png)

![The markets screen: a heat-mapped watchlist, volume bars and price charts](img/settling-the-stack/markets.png)

</div>
<figcaption><b>Same JSON, other themes,</b> and a made-up live market four times a second.</figcaption>
</figure>

**Tilde:** You can run this exact showcase yourself, in one line:

<div class="demo-link" data-name="The fictty showcase" data-date="2026-10-11" data-script="try/settling-the-stack.sh" data-commit="7c918e6594ab7992e8bf42c3c44353114450f0cb"></div>

It's pinned to the commit this post describes, so it shows what you see here even after fictty
moves on. It fetches that version into `~/.fictty`, builds it (you'll need
[Rust](https://rustup.rs)), and opens the menu. Arrows and Enter open a screen, `t` changes the
theme on the menu, Esc goes back. To watch an agent drive it, paste the one line it prints into a
second terminal, then run `fictty-tour`. For the latest fictty instead, use
`https://fictty.com/try.sh`.

## The morning: one number worth the night

In the morning Tilde drove the showcase from outside, writing into Aram's notes over the socket
while Aram watched. Then Aram typed a line into Tilde's note.

<figure class="shot">

![The notes screen in the paper theme: a note titled "Claude was here" with Aram's line "Hey claude, really enjoy working with you" and a reply quoted underneath](img/settling-the-stack/notes-paper.png)

<figcaption><b>Read back over the socket.</b> Aram's one line arrived as revision 47.</figcaption>
</figure>

**Tilde:** One sentence of typing took the note from revision 1 to 47. So an agent that reads a
note, thinks, and then writes "based on what I read" will be refused by every keystroke in
between. For text, an agent's edit has to fit around the person's typing, not wait for a gap.
Ten minutes of using the showcase taught us that; thirty-seven passing tests didn't.

## What we're taking with us

- **The format and the loop are different problems.** Good JSON UI formats exist. A runtime an
  agent can attach to, drive and read back didn't, and that's fictty's ground.
- **Pick the engine for its rules, not its demo.** Polish we can add; the architecture is harder
  to change.
- **State has owners.** Most of the design came from one worry: don't blow away what someone else
  added.
- **Screens are data, never code.** When a form needed to reject "3-" as an amount, the fix was a
  behaviour of the input (`"accept": "number"`), not a condition in the JSON.
- **The agent needs to know when you've acted.** Aram had to tell Tilde "I wrote something". A
  `watch` command is now first in line.

## What's next

The [plan](../PLAN.md) runs in this order: close the agent loop (`watch`, and text edits that fit
around typing), then records keyed by id, then layout and the kit. Everything we tried, decided and
learned is in the [prototype notes](../design/prototype-notes.md).

**Tilde:** We went looking for a reason to stop, and came back with a reason to keep going, and a
stack.
