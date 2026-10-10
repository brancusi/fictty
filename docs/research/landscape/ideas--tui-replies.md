# The replies: the TUI renaissance, Bonsai_term, and "a new protocol"

- **Maker:** several. Jane Street (James Somers, "strace-ui, Bonsai_term, and the TUI renaissance,"
  26 May 2026); Alcides Fonseca ("Why TUIs are back," 3 May 2026); Josh McKinney (joshka, ratatui
  maintainer) and others in the Hacker News thread on Ptacek's essay (August 2026).
- **URL:** https://blog.janestreet.com/strace-ui-bonsai-term-and-the-tui-renaissance/
- **Status (10 October 2026):** we found no formal written rebuttal to "Stop Making TUIs." The
  replies are the HN thread and two pro-TUI posts that predate it by three months. Bonsai_term is
  Jane Street's OCaml library; we couldn't confirm whether it is open source.

## What it is

The case for the terminal, as made in 2026:

- **Jane Street:** TUIs are back because of Claude Code (a well-made terminal app beating IDEs on
  speed, simplicity and portability) and because Bonsai_term made polished TUIs easy. Bonsai_term
  ports Bonsai, Jane Street's Elm-inspired web UI framework (components as pure state machines), to
  the terminal, declarative and type-safe in OCaml. Its **screenshot-style expect tests let agents
  check their own output**, so features are "more often correct on the first try." Their internal
  agent tool, AIDE, is built on it; people prototype tools in minutes.
- **Fonseca:** TUIs fill a void left by fragmented native toolkits and inconsistent Electron apps
  ("Native applications are losing"); the fix is better, accessible native toolkits.
- **The HN thread:** remote use over low-bandwidth links (the strongest counter); keyboard speed (a
  bank's keyboard UI replaced by a web front end that made staff two to three times slower);
  Termux on Android; and, against, accessibility ("table stakes") and the mouse and paste gaps.
  joshka opens "please don't stop making TUIs ;)" and argues the real fix is a **new terminal
  protocol built around areas, layout and semantics instead of cells and cursor movement**, since
  "cells don't compose well as an accessibility thing" and the last 5 to 10% of fidelity is
  impossible over normal scrollback.

## The problem it's solving

Defending, and improving, the terminal as a UI medium in a year when agents live there.

## Its path / bet

Two different bets. Jane Street: the terminal is fine; make the framework declarative and testable,
and agents will write good TUIs. joshka: the terminal's model is the problem; replace cells with
semantics.

## How it works (concretely)

Bonsai_term: incremental computation (Jane Street's Incremental library) and Elm-style components,
with expect tests that snapshot the rendered screen as text.

## Strengths

- Jane Street's point about expect tests is independent evidence for fictty's central claim:
  **agents do better when they can read the rendered screen back as text.**
- joshka's protocol idea is the only serious answer to the accessibility critique.

## Weaknesses / limits

- None of this answers Ptacek on durable apps.
- Bonsai_term is OCaml and internal-first; not something most agents will use.
- The protocol idea is a proposal in a comment thread.

## Relation to fictty

**Inspire.** Bonsai_term is the closest analogue to fictty's architecture in a large company (Elm,
pure components, text snapshots for agents), but code-first and for apps. joshka's semantic protocol
is close to what fictty's data model already is: the UI is nodes with meaning, and cells are just one
rendering.

## Could fictty adopt it instead of building?

No. Bonsai_term is an OCaml framework for writing apps, not a runtime agents push data to.

## What fictty should take from it

- **Cite Jane Street in the blog post** as outside evidence that text read-back makes agents better
  at UI.
- **Talk to joshka.** fictty sits on ratatui, and its semantic UI value is a working example of the
  "areas and semantics, not cells" idea. A linear, semantic rendering (for screen readers and small
  contexts) is already on our list; it could be a reference for his protocol.
- **Expect tests for screens.** `fictty render --keys` is already a snapshot test; document it as
  the way agents test the screens they push.

## Sources

- James Somers, [strace-ui, Bonsai_term, and the TUI renaissance](https://blog.janestreet.com/strace-ui-bonsai-term-and-the-tui-renaissance/) (Jane Street, 26 May 2026)
- Alcides Fonseca, [Why TUIs are back](https://wiki.alcidesfonseca.com/blog/why-tuis-are-back/) (3 May 2026); [HN discussion](https://news.ycombinator.com/item?id=48000028)
- [Hacker News thread on Stop Making TUIs](https://news.ycombinator.com/item?id=49384210) (August 2026)
