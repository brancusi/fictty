# Gemini generative UI (dynamic view, visual layout, AI Mode)

- **Maker:** Google (Google Research: Yaniv Leviathan, Dani Valevski, Vishnu Natchu, Yossi Matias)
- **URL:** https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/
- **Status (10 October 2026):** shipping in stages, closed. Announced 18 November 2025 with
  Gemini 3: two experiments in the Gemini app (dynamic view, visual layout) and generative layouts
  in AI Mode in Search for AI Pro and Ultra subscribers in the US. Canvas came to AI Mode for
  everyone in the US in March 2026. At I/O (May 2026) Google said Search would build custom
  layouts with interactive visuals, tables, graphs and simulations using Gemini 3.5 Flash and
  Antigravity, free to everyone "this summer", and that persistent trackers, dashboards and "mini
  apps" would follow "in the coming months", starting with US subscribers. We couldn't confirm
  whether the mini apps have shipped. The research paper's evaluation dataset (PAGEN) was
  promised for release; we didn't find it.

## What it is

The model answers a prompt with a custom interactive page instead of text: HTML, CSS and
JavaScript generated per prompt, with tools for images and search. A question about fractals gets
an explorable fractal; a trip question gets a planner. Gemini's Canvas is the separate, explicit
"build me an app/page" workspace.

## The problem it's solving

Text answers are a poor fit for many questions, especially ones about how something works.
Google's framing is that the model should design the *experience*, not just the content, per
question.

## Its path / bet

Fully generated, per-prompt, disposable HTML at consumer scale, inside the search box. Of everyone
in this cluster, Google is the furthest along the "UI is as cheap as a paragraph" line. The I/O 2026
move to persistent dashboards and mini apps is the same lab discovering that some generated UIs
should be kept.

## How it works

Per the research post: Gemini 3 Pro with (1) tool access (image generation, web search), whose
results can go to the model or straight to the browser; (2) long system instructions covering
planning, examples, technical specs and common errors; (3) post-processors that fix common
mistakes in the output. The output is HTML/CSS/JS. Product surfaces can impose a visual style. In
their evaluation, human raters strongly preferred these pages over the top search result, raw text
and Markdown, and only expert-made websites scored higher, with generation speed explicitly *not*
counted. The post concedes generation "can sometimes take a minute or more" and outputs "occasionally
contain inaccuracies".

## Strengths

- The most direct evidence anyone has published that people prefer generated interactive pages to
  text answers (with the speed caveat).
- Explicitly about understanding: explainers, simulations, interactive visuals. This is the
  "revolution in teaching" thread in its most mainstream form.
- Tool results can go straight to the browser, bypassing the model: the same principle as fictty's
  data sources.
- Distribution: Search and the Gemini app.

## Weaknesses / limits

- Latency: up to a minute per page, by Google's own account; preference numbers ignore it.
- No agent loop: the page is an answer, not a screen an agent keeps working on, patches or reads
  back.
- Consumer surfaces only; no developer API for "render this generated UI in my product" that we
  found (A2UI is Google's separate, data-first answer for that).
- Accuracy and consistency depend on the model; post-processors patch the most common failures.

## Relation to fictty

**Inspire.** Different surface, different user. Two things matter for fictty. First, it is the
strongest evidence for the learning thesis: generated interactive UI helps people understand.
Second, Google itself runs two bets: free-form HTML for consumer answers (this) and declarative
data for developers (A2UI). That split suggests the HTML/data question is decided by who owns the
renderer, not by which is better in the abstract.

## Could fictty adopt it instead of building?

No. It isn't available as a component or API.

## What fictty should take from it

- **The post-processor idea.** Google fixes common model mistakes after generation. fictty can do
  the same, more reliably, because its input is data: validate and repair a pushed UI value (unknown
  fields, missing ids, bad bindings) with a precise error, and record what it fixed.
- **Tool results straight to the view.** Same principle as fictty's sources; cite it.
- **Explainers as a first-class use case.** Build an "explain this system" recipe (concept
  explainer, interactive diagram, stepper) and measure it, rather than leaving learning as rhetoric.
- **Measure time-to-first-useful-frame,** the number Google left out. A fictty screen should be
  useful in a second or two because the agent writes a small value, not a page.

## Sources

- Google Research, [Generative UI: a rich, custom, visual interactive user experience for any prompt](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/) (18 November 2025)
- Google, [Search's I/O 2026 updates](https://blog.google/products-and-platforms/products/search/search-io-2026/) and [100 things we announced at I/O 2026](https://blog.google/innovation-and-ai/technology/ai/google-io-2026-all-our-announcements/)
- Google, [Canvas in AI Mode](https://blog.google/products-and-platforms/products/search/ai-mode-canvas-writing-coding/)
- Google Workspace Updates, [More ways to create with Canvas in Gemini](https://workspaceupdates.googleblog.com/2025/05/more-ways-to-create-with-canvas-gemini.html) (May 2025)
- [Generative UI glossary](https://inblog.ai/glossary/generative-ui) (secondary; used only for naming confusion)

Not verified: current status of dynamic view and visual layout in the Gemini app (still
experiments or generally available), and whether mini apps have shipped.
