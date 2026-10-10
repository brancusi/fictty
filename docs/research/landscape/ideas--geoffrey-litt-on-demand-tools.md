# On-demand tools, AI HUDs and "understanding is the new bottleneck" (Geoffrey Litt)

- **Maker:** Geoffrey Litt (formerly senior researcher at Ink & Switch, now at Notion per his site)
- **URL:** https://www.geoffreylitt.com/
- **Status (10 October 2026):** a series of essays, not a product: "Malleable software in the age of
  LLMs" (March 2023), "AI-generated tools can make programming more fun" (December 2024), "Is chat
  a good UI for AI?" (June 2025), "Enough AI copilots! We need AI HUDs" (27 July 2025), "Code like
  a surgeon" (October 2025), "Understanding is the new bottleneck" (2 July 2026).

## What it is

The most concrete body of writing on what we'd call throwaway UIs for understanding. Litt's
pattern: instead of asking an agent to fix a problem, ask it to build an instrument that lets *you*
see the problem, use the instrument, and discard or reshape it.

## The problem it's solving

When agents write most of the code, the human risks "cognitive debt": "you can get away with not
understanding what's going on in the short term, but it'll bite you eventually." Understanding
isn't only for verifying the agent's work; it's how you take part ("we can understand to
participate") and come up with the next idea. "The point was always to *augment*, not just
automate."

## Its path / bet

- **HUDs over copilots.** After Mark Weiser's 1992 critique: a HUD doesn't talk to you, it extends
  what you can perceive (spellcheck is the everyday example). Copilots suit routine work;
  extraordinary work calls for instruments.
- **Build the instrument on demand.** A custom tool takes a minute to generate, so the threshold
  for making one drops to "any time I'm spending more than a minute staring at a JSON blob."
- **Three techniques** in the 2026 essay: explanations (an agent-written "explain-diff" with
  background, intuition and interactive figures, followed by a quiz he must pass before sharing
  the code), micro-worlds (after Papert's Mathland: environments you learn in by doing), and shared
  spaces where a team builds a common mental model.

## How it works (concretely)

- **A Prolog debugger** (December 2024): Claude built a hacker-themed web UI showing the
  interpreter's stack and a timeline he can scrub. It took about a minute; each refinement took
  seconds. "I still haven't looked at the UI code." "I caught a couple bugs immediately."
- **A migration "command center"** (2026): for a site port he didn't know well, Claude built a UI
  where he steps through the port with the old and new sites side by side and the file tree
  changing.
- **Interactive explainer figures,** such as dragging rocks to see coordinates change under an
  isometric projection.

All of these are browser pages, written as code by the agent.

## Strengths

- Real examples from a working programmer, with honest notes on when it works ("AI development
  works well when your requirements are flexible").
- Names the actual goal: understanding, not output. This is the strongest argument in the whole
  cluster for why disposable UIs matter.
- The quiz-before-sharing step is a concrete, testable learning loop.

## Weaknesses / limits

- Anecdotal; no measurement of whether the instruments improve understanding beyond his own report.
- Each instrument is fresh code. Nothing carries over between them except the prompt, and nobody
  checks the instrument itself is right.
- Assumes a browser beside the editor.

## Relation to fictty

**Inspire**, strongly. Litt's debugger and command center are exactly the screens fictty is for: made
in a minute, used for an hour, bound to live program state, discarded. The difference is the
medium (his are HTML in a browser) and the representation (his are code; fictty's are data with
data sources). His "explain-diff plus quiz" is a teaching screen with an answer the agent needs to
read back.

## Could fictty adopt it instead of building?

Nothing to adopt; it's a practice. The practice already works today with HTML, which is the real
competition: fictty has to be better than "ask Claude for a web page" for these instruments, and it
will only be better where the instrument needs live data, runs on a remote box, or the agent needs
to read back what the person did.

## What fictty should take from it

- **Make "instrument" the headline use case,** not "dashboard." A stack viewer, a step-through, a
  side-by-side diff with a scrubber.
- **Quizzes and checkpoints need read-back.** A screen that asks a question and waits (`watch`) for
  the answer is the HUD-plus-learning loop in one primitive.
- **Scrubbing time is a primitive.** Both of his examples step through time. fictty's planned
  history (`state_at`, `seek`) should be usable as a UI component, not only as an agent API.

## Sources

- Geoffrey Litt, [Understanding is the new bottleneck](https://www.geoffreylitt.com/2026/07/02/understanding-is-the-new-bottleneck) (July 2026)
- Geoffrey Litt, [Enough AI copilots! We need AI HUDs](https://www.geoffreylitt.com/2025/07/27/enough-ai-copilots-we-need-ai-huds) (July 2025)
- Geoffrey Litt, [AI-generated tools can make programming more fun](https://www.geoffreylitt.com/2024/12/22/making-programming-more-fun-with-an-ai-generated-debugger) (December 2024)
- Geoffrey Litt, [Malleable software in the age of LLMs](https://www.geoffreylitt.com/2023/03/25/llm-end-user-programming) (March 2023)
- Simon Willison, [link post on AI HUDs](https://www.simonwillison.net/2025/Jul/27/enough-ai-copilots-we-need-ai-huds)
