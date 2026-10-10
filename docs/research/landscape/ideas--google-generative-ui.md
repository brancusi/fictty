# Generative UI (Google Research) and its user study

- **Maker:** Google Research (Yaniv Leviathan, Dani Valevski, Yossi Matias and others; 12 authors on
  the paper)
- **URL:** https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/
- **Status (10 October 2026):** blog post 18 November 2025; paper "Generative UI: LLMs are Effective
  UI Generators" on arXiv (2604.09577; the listing we fetched shows v1 dated 24 February 2026,
  which conflicts with the ID's April prefix, so treat the exact date as unverified). Shipped as
  experiments: "dynamic view" and "visual layout" in the Gemini app, and generated tools in AI Mode
  in Search for Pro and Ultra subscribers in the US. Closed source.

## What it is

A system that answers any prompt with a custom interactive web page instead of text: Gemini 3 Pro,
given tools (image generation, web search), long system instructions and post-processors, writes
HTML, CSS and JavaScript that the browser runs.

## The problem it's solving

A wall of markdown is a poor answer to many questions: how something works, how options compare,
how a quantity changes. Pre-built UIs can't cover the space of prompts. If the model writes the UI
per prompt, every answer can have the interface it needs.

## Its path / bet

The model writes code, per prompt, and throws it away. The bet is that model quality is the limit
("newer models perform substantially better"), so this gets better for free.

## How it works (concretely)

- Tool server for images and search; results go to the model or straight to the browser.
- System instructions covering planning, examples, technical specs, formatting and known pitfalls.
- Post-processors fix common output errors.
- A consistent product style can be imposed, or the model picks one.

**The evaluation.** Google built PAGEN, a dataset of expert-made websites for prompts. Raters
compared pairs of answers on a three-point scale, two raters per pair, with results pre-cached so
generation time was hidden. Expert-made sites came first. Generative UI came close behind and was
preferred over standard markdown 82.8% of the time on 100 LMArena prompts and 90.5% on an
information-seeking set, with an Elo of 1736 against every format but human experts. The abstract
says generated UIs are "at least comparable" to expert results in 50% of cases.

## Strengths

- The first large, published preference study for per-prompt generated UI, from a lab that ships it.
- Clear about its limits: generation "can take a minute or more," with "occasional inaccuracies."
- Shipped to real users, not only a demo.

## Weaknesses / limits

- **Preference is not learning or task success.** The study measures which answer raters like,
  with speed removed. It says nothing about whether people understood more or did the task faster.
- Speed removed from the comparison is a large thing to remove: a minute of waiting changes the
  experience.
- Each page is fresh code; correctness isn't checked beyond post-processing, and the model can't
  read back what the person did with it.
- Browser and chat only.

## Relation to fictty

**Watch / compete at the idea level.** It is the strongest evidence that people prefer a generated
interface to text when the wait is hidden. It also shows the labs' default: code, per prompt, in a
browser. fictty's bet is the opposite on every axis (data, long-lived, terminal, read-back), so
this is the baseline we'll be compared with.

## Could fictty adopt it instead of building?

No. It's a closed product feature in Gemini and Search. Its method (prompting a model to write
HTML) is open to anyone and is what Claude visuals and Artifacts do too.

## What fictty should take from it

- **Speed matters and they hid it.** A fictty screen is a small JSON value and draws in
  milliseconds; data comes without the model. Measure time-to-first-useful-screen against
  generate-a-page and publish it.
- **Their study design is reusable.** Pairwise preference with pre-cached results is cheap to run.
  If we claim screens help, run a small version of it, and add a comprehension question so it
  measures understanding, not just liking.
- **Post-processors are a tell.** Generated code needs repair passes; a validated data format
  needs a schema check. Keep the format small enough that "valid" means "renders."

## Sources

- Google Research, [Generative UI blog post](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/) (18 November 2025)
- Leviathan et al., [Generative UI: LLMs are Effective UI Generators](https://arxiv.org/abs/2604.09577) ([PDF](https://arxiv.org/pdf/2604.09577))
