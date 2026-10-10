# "HTML is the new markdown" (The unreasonable effectiveness of HTML)

- **Maker:** Thariq Shihipar (Claude Code team, Anthropic)
- **URL:** https://twitter.com/trq212/status/2052809885763747935 ; examples at https://thariqs.github.io/html-effectiveness/
- **Status (10 October 2026):** an essay and an example gallery, published around 7 to 8 May 2026
  (Simon Willison linked it on 8 May 2026). Widely shared; secondary sources report millions of views
  within a day, which we couldn't verify. Discussed further on Lenny's Newsletter's "How I AI"
  podcast. The phrase became Lavish's tagline.

## What it is

An argument, from inside the Claude Code team, that for most non-trivial output agents should
write a single self-contained HTML file instead of Markdown, with twenty examples in nine groups:
exploration and planning (three code approaches, design directions, an implementation plan), code
review (annotated PR, PR write-up, module map), design (living design system, component variants),
prototyping (animation sandbox, clickable flow), diagrams (SVG figure sheet, annotated flowchart),
an arrow-key slide deck, research and learning (how a feature works, a concept explainer), reports
(weekly status, incident timeline) and custom editors (ticket triage board, feature-flag editor,
prompt tuner).

## The problem it's solving

Long Markdown doesn't get read ("trades a document you'd skim for one you'd actually read"), and
Markdown flattens spatial information: "diffs and call-graphs are spatial information; markdown
flattens them." Token limits that once justified Markdown no longer bind.

## Its path / bet

Make HTML the default output format for anything spatial, interactive or long. It's a prompting
habit, not a product, though it is clearly the thinking behind Claude Code Artifacts a month later.

## How it works

Ask the agent for "an HTML artifact" (for example, "Help me review this PR by creating an HTML
artifact that describes it"), open the file in a browser, interact, and in the custom-editor
examples copy the result back into the session.

## Strengths

- Names a real shift clearly: producing a UI used to be expensive, so we wrote text; now a
  purpose-built page per question costs one model turn.
- The examples are concrete and teach well. The "custom editing interfaces" group (a triage board
  whose output you paste back) is the seed of the agent-and-person loop.
- Strong on learning: explainers, module maps and annotated flows are exactly where text fails.

## Weaknesses / limits

- The gallery lists no downsides. Critics raise token cost, security of running generated scripts,
  models drifting on tag hygiene and nesting, and accessibility.
- The page is a broadcast: the round trip back to the agent is copy and paste.
- Data is frozen into the page when it's written.
- It doesn't address where the page runs (a browser) or who can see it, which is why Artifacts and
  Lavish had to exist.

## Relation to fictty

**Inspire, and the strongest counter-argument.** It's the best statement of why the terminal is not
enough for output, and if it's simply right, the case for terminal screens shrinks to remote
sessions, live data and read-back. Read carefully, it also supports fictty's idea: most of the
twenty examples are a small number of shapes (side-by-side comparison, annotated code, a board, a
timeline, a stepper, an editor with an export), which is an argument for a catalogue of components
the agent fills, not a fresh page every time.

## Could fictty adopt it instead of building?

It's an idea, not a tool. The practical question is whether fictty should *output* HTML too. Yes,
as an export, not as its primary medium.

## What fictty should take from it

- **Port the gallery.** Rebuild the twenty examples as fictty screens. Each one that works is a
  recipe and a demo; each one that doesn't marks a real limit of the terminal. It's the most honest
  test of the scope we have.
- **Make "paste the result back" unnecessary.** fictty's read-back and `watch` turn the editor
  examples into a closed loop with no clipboard.
- **Teach with it.** The "concept explainer" and "how a feature works" examples are the learning
  thesis in practice; they should be among the first fictty recipes.

## Sources

- Thariq Shihipar, [post on X](https://twitter.com/trq212/status/2052809885763747935) and [HTML effectiveness gallery](https://thariqs.github.io/html-effectiveness/)
- Simon Willison, [link post, 8 May 2026](https://simonwillison.net/2026/May/8/)
- Lenny's Newsletter, [How I AI: HTML is the new Markdown](https://lennysnewsletter.com/p/how-i-ai-html-is-the-new-markdown)
- [HTML vs Markdown in Claude Code](https://pasqualepillitteri.it/en/news/2243/html-vs-markdown-claude-code-thariq-anthropic) and [Anthropic engineer advocates HTML over Markdown](https://letsdatascience.com/news/anthropic-engineer-advocates-html-over-markdown-for-agents-38c8211b) (secondary)

Not verified: the full text of the original post (we read the gallery and secondary summaries, not
the post itself), and the view counts.
