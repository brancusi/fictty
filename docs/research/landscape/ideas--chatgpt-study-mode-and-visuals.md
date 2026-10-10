# ChatGPT study mode and interactive math and science visuals (OpenAI)

- **Maker:** OpenAI
- **URL:** https://openai.com/index/new-ways-to-learn-math-and-science-in-chatgpt and
  https://help.openai.com/en/articles/11780217-using-study-mode-in-chatgpt
- **Status (10 October 2026):** study mode launched 29 July 2025 for Free, Plus, Pro and Team users.
  Interactive visuals launched 10 March 2026 for all users, covering more than 70 maths and science
  concepts, with more planned. Closed source. We added this entity; it wasn't in the brief, but it
  is the counterexample to per-prompt generated UI.

## What it is

- **Study mode:** Socratic tutoring, switched on from "Study and learn" in the tools menu. ChatGPT
  asks questions and gives hints rather than answers.
- **Interactive visuals:** for set concepts (the Pythagorean theorem, Ohm's law, Coulomb's law, lens
  equations, compound interest), ChatGPT shows a visual where you change variables or the equation
  and see graphs and results update in real time.

## The problem it's solving

The same as Claude's learning mode and Gemini's Guided Learning: answer-giving undermines learning;
explanations of how quantities relate land better when you can manipulate them.

## Its path / bet

The visuals look like a **catalogue**, not generation: a fixed set of 70-odd concepts, each an
interactive module the model invokes and parameterises. OpenAI's page was not readable to us (403)
and Engadget doesn't say whether the modules are prebuilt, so this is our inference from the
fixed, enumerated concept list. If right, it is the data-first path (the model chooses and fills a
known component) inside a lab product, next to Google and Anthropic choosing per-prompt code.

## How it works (concretely)

Unverified beyond the above. Study mode is a system-prompt behaviour.

## Strengths

- A catalogue of tested modules is reliable: no layout errors, no wrong formulas drawn.
- Free on all plans, so it reaches the most students.

## Weaknesses / limits

- 70 concepts is a small fraction of what students ask about.
- No published evidence of learning effect that we found.
- Closed; chat only.

## Relation to fictty

**Inspire.** If our reading is right, OpenAI made the same trade-off fictty makes: reliability from a
fixed set of components the model fills in, rather than a fresh program every time. The cost is
coverage, which fictty addresses by making its components general (tables, charts, layers) rather
than one per concept.

## Could fictty adopt it instead of building?

No. Closed product feature.

## What fictty should take from it

- **The catalogue path is respectable.** It's not a lesser version of generation; a lab chose it for
  learning, where correctness matters most. Use that in the "data, never code" argument.
- **"Recipes" are concept modules.** A fictty recipe for "a formula with sliders and a plot" bound to
  a computed source would be the terminal equivalent of one of OpenAI's 70, written once and filled
  by any agent.

## Sources

- OpenAI, [New ways to learn math and science in ChatGPT](https://openai.com/index/new-ways-to-learn-math-and-science-in-chatgpt) (March 2026; not readable to us, 403)
- Engadget, [ChatGPT will now generate interactive visuals](https://engadget.com/ai/chatgpt-will-now-generate-interactive-visuals-to-help-you-with-math-and-science-concepts-170000520.html) (10 March 2026)
- OpenAI Help, [Using study mode in ChatGPT](https://help.openai.com/en/articles/11780217-using-study-mode-in-chatgpt)
- DataNorth, [OpenAI launches Study Mode](https://datanorth.ai/news/openai-launches-study-mode) (July 2025)
