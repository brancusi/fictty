# Claude's inline visuals ("Claude builds visuals")

- **Maker:** Anthropic
- **URL:** https://claude.com/blog/claude-builds-visuals
- **Status (10 October 2026):** beta, launched 12 March 2026 on all plans, on by default; extended
  to Cowork on paid plans on 22 April 2026. Web and desktop only; the help centre says they don't
  render on iOS or Android. Closed source. We added this entity; it is the most direct lab statement
  of "UI as temporary."

## What it is

Claude decides mid-conversation that a visual would help and builds one inline: a compound interest
curve you can play with, a clickable periodic table, a diagram of a mechanism. The person can ask
for changes. It grew out of the "Imagine with Claude" research preview (September 2025).

## The problem it's solving

Explanations in text are weaker than explanations you can see and manipulate, and Artifacts (a side
panel, meant to be kept) are too heavy for a quick "let me show you."

## Its path / bet

Anthropic says it plainly: Artifacts are "permanent tools and documents ... designed to be shared or
downloaded"; visuals "serve a different purpose" and "they're temporary." They appear inline,
"shifting and evolving as the discussion progresses," and disappear when no longer relevant. Built
"to aid users' understanding." This is the disposable-UI thesis adopted as product policy by a lab.

## How it works (concretely)

The help centre says Claude builds them with HTML, so they're interactive and specific to the
question. Secondary sources disagree on details (SVG, or chart libraries such as Chart.js loaded
from a CDN); we couldn't confirm the implementation.

## Strengths

- Zero friction: no request needed, no panel, no link.
- Clear product distinction between durable (Artifacts) and disposable (visuals), which the rest
  of the industry hasn't articulated as well.
- Aimed squarely at understanding.

## Weaknesses / limits

- Code generated per visual, so correctness depends on the model each time.
- Not bound to live data; the numbers are whatever the model put in.
- Read-back is limited to the conversation; we found no indication that Claude sees what the person
  did inside the visual.
- Not in Claude Code's terminal, as far as we could find.

## Relation to fictty

**Compete**, at the level of the idea, and the clearest threat in this cluster. If inline visuals
come to Claude Code (its desktop and web surfaces already render HTML), "the agent shows you
something while it explains" is covered for most developers without fictty. What would remain is
what visuals don't do: live data from commands, frame-rate interaction, exact read-back of what the
person did, the remote terminal.

## Could fictty adopt it instead of building?

No, it's a closed feature of Claude's apps. But for a fictty user in Claude's desktop app, an inline
visual is often the better tool for a quick concept explanation, and we should say so.

## What fictty should take from it

- **Adopt the vocabulary.** Durable (artifact, app) against temporary (visual, screen). fictty is the
  temporary kind for the terminal, with live data.
- **Unprompted is the default.** The agent should put up a screen when it helps, without being asked.
  That's a skill-writing problem: tell the agent when a screen beats text.
- **Compete on what HTML visuals can't do:** bind to the real data, run while the agent works, and
  report back what the person chose.

## Sources

- Anthropic, [Claude now creates interactive charts, diagrams and visualizations](https://claude.com/blog/claude-builds-visuals) (12 March 2026)
- Claude Help Center, [Custom visuals in chat and Cowork](https://support.claude.com/en/articles/13979539-custom-visuals-in-chat-and-cowork) (April 2026)
- AlternativeTo, [coverage](https://alternativeto.net/news/2026/3/anthropic-s-claude-can-now-respond-with-interactive-visuals-such-as-charts-and-diagrams); The New Stack, [coverage](https://thenewstack.io/anthropics-claude-interactive-visualizations/)
