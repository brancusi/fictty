# "Stop Making TUIs" (Thomas Ptacek)

- **Maker:** Thomas Ptacek (sockpuppet.org, "A Final Ward")
- **URL:** https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/
- **Status (10 October 2026):** essay of 20 August 2026. The Hacker News thread stood at 437 points
  and 569 comments when we read it (the earlier landscape post counted 539). Linked by Simon
  Willison on 21 August. Covered in the first landscape pass; this dossier goes deeper on the
  argument and the thread.

## What it is

An argument that the terminal UI is obsolete for new work because coding agents made native GUIs
nearly free. "Building a CLI is almost always a good idea. Building a TUI almost never is." "TUIs
exist for just two reasons: modems, and because Unix nerds didn't want to learn Motif." "We build
terminal interfaces because we have to, not because we should."

## The problem it's solving

Developers keep choosing TUIs out of habit, accepting a medium that only approximates scrolling,
selection, drag and drop, windowing and images, when the cost reason for doing so is gone.

## Its path / bet

- Let agents write native apps. He builds SwiftUI macOS apps with template projects, skills and
  Makefiles, rarely touching the UI code, and gives agents `computer-use` so they can see and drive
  the app.
- For remote machines: "you probably don't need a user interface on prod," but a CLI there that a
  UI on your laptop can drive (he cites Emacs TRAMP).
- On accessibility: screen readers struggle with TUI redraws; native frameworks were built with
  accessibility in mind. He says he isn't an accessibility user and that some TUI frameworks try
  hard.

## What he concedes

TUIs are economical, fast, information-dense and keyboard-driven; they're cross-platform, which is
"not nothing" (he can't easily test native apps on Windows or Linux); his dislike is partly taste;
his apps are personal ("I'm building them for me"), vibe-coded rather than vibe-shipped; and he
still uses tmux to have agents test TUIs. In the thread he calls ratatui "such great work" and says
"The New Terminal, the complete break from VT100, is I think the most powerful rebuttal" to his
case, adding "I hope those people eventually set their sights higher."

## How it works (concretely)

An essay; the practice is native apps written by agents, verified by computer use.

## Strengths

- Right about apps people keep. A daily tool for a year should be native or web.
- Right that cheap UI changes the calculation; that's the same premise as this whole cluster.
- Honest about his own limits.

## Weaknesses / limits

- Argues from one person's macOS workflow for personal apps. His apps are durable; he doesn't
  consider the screen that lives ten minutes next to an agent.
- Verifying the GUI needs computer use (screenshots and a vision model); he doesn't count that
  cost, which is the cost exact read-back removes.
- Remote: "a CLI driven by a local GUI" means building two programs and a protocol, every time.

## Relation to fictty

**Compete** (as an argument). It's the most credible case against fictty's medium, from someone
respected, and fictty's answer has to concede most of it. Where it doesn't reach: disposable screens,
remote agents, exact read-back, and his own escape hatch ("the New Terminal," a break from VT100
with rich out-of-band UI), which is close to what fictty's data model could be rendered into.

## Could fictty adopt it instead of building?

The essay's alternative is "have the agent write a native app." For a fictty-type screen (minutes,
beside the agent, maybe remote), that is slower, needs a GUI session, and can't be read back
without screenshots. For anything durable, take his advice.

## What fictty should take from it

- **Concede the durable app.** Our scope already says it; keep saying it.
- **His remote pattern is our architecture.** "A CLI on prod that a local UI drives" is what a
  fictty socket is. The UI value can be rendered locally even when the data comes from the remote
  box. A local renderer of a remote fictty screen (terminal, or later native or web) answers him
  directly.
- **Be the New Terminal's content model.** If the break from VT100 happens, a UI that is semantic
  data (rows, nodes, actions) moves to it; a UI that is cells and escape codes doesn't. That's a
  reason to keep the model independent of the cell grid.

## Sources

- Thomas Ptacek, [Stop Making TUIs](https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/) (20 August 2026)
- [Hacker News thread](https://news.ycombinator.com/item?id=49384210)
- Simon Willison, [link post](https://simonwillison.net/2026/Aug/21/stop-making-tuis/) (21 August 2026)
