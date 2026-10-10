# Observable (notebooks, Framework, Notebooks 2.0)

- **Maker:** Observable, Inc. (Mike Bostock, creator of D3, and team)
- **URL:** https://observablehq.com
- **Status (10 October 2026):** hosted notebooks since 2018. Observable Framework (static data
  apps, ISC, 3.7k stars, last release v1.13.4 on 2 March 2026, last push May 2026) looks quieter.
  Notebooks 2.0 was previewed in late July 2025: an open HTML-based notebook file format, the open
  source Notebook Kit (ISC, 379 stars, pushed 9 October 2026) and a macOS desktop app (now
  "Notebook Desktop"). Observable Canvases is the hosted, collaborative, AI-heavy product. Observable
  Plot (ISC, 5.4k stars) is the charting library.

## What it is

A reactive JavaScript notebook: cells are re-run automatically when what they depend on changes,
and outputs are live visualisations, usually D3 or Plot. It is the closest thing the web has to a
medium for explorable data explanations, and much of the explorable-explanation community wrote in
it.

## The problem it's solving

Exploring and explaining data needs code, live output and prose in one place, and the result should
stay reactive so readers can change inputs.

## Its path / bet

Reactivity plus the web's full graphics, in a document. Over time the company has split the bet:
open files and local tools for developers (Framework, Notebook Kit, Desktop), and a hosted
collaborative canvas with AI for business (Canvases).

## How it works (concretely)

Cells form a dependency graph; changing an input reruns dependents. Notebooks 2.0 stores a notebook
as an HTML dialect and uses vanilla JavaScript (so standard tools like ESLint work), replacing the
Observable JS dialect's custom syntax such as `viewof`. Desktop appears to be a Tauri app with
sandboxed iframes and a CodeMirror editor. Tom MacWright (first hire, holds equity) describes the
notebook AI as a light touch and Canvases as AI-heavy. We couldn't read Observable's own pages in
detail (rate-limited), so the AI specifics are unverified.

## Strengths

- Reactive dataflow is the right model for explorable data, and it's proven at scale.
- Best-in-class web charts (D3, Plot).
- Moving to open files ("file over app") makes notebooks something an agent can write.

## Weaknesses / limits

- The author still writes code; the reader's interaction is what the author built.
- Product direction has shifted several times (notebooks, Framework, Canvases, Notebooks 2.0),
  which makes it hard to build on.
- Browser only.

## Relation to fictty

**Watch / inspire.** Observable's reactivity is the design fictty's data sources gesture at: a
dependency from a source to every node bound to it. Notebooks are durable documents; fictty screens
are short-lived. They would only compete if agents started generating Notebook Kit files as their
scratch UIs, which is plausible for data work in a browser.

## Could fictty adopt it instead of building?

No. Notebook Kit is a browser notebook runtime in JavaScript; fictty is a terminal runtime. One
could argue that for data exploration an agent should just write a Notebook Kit file and open it;
for a person in a browser that's often right.

## What fictty should take from it

- **Dependency-driven updates.** A parameter (input) that sources depend on, rerunning only what
  depends on it, is Observable's core idea and the missing piece for reactive teaching screens.
- **An HTML-ish file format was chosen over JSON** for human editing. fictty's JSON is written by
  agents; if people are to edit saved screens, consider whether a friendlier authoring surface
  (YAML or the template markup) is needed, and keep the canonical form JSON.

## Sources

- Observable, [Notebooks 2.0 technology preview](https://observablehq.com/notebooks/2/) and [blog post](https://observablehq.com/blog/previewing-notebooks-2) (July 2025)
- Tom MacWright, [Observable Notebooks 2.0](https://macwright.com/2025/07/31/observable-notebooks-2) (31 July 2025)
- Simon Willison, [link post](https://simonwillison.net/2025/Aug/6/observable-notebooks-20/)
- Observable, [2025 year in review](https://observablehq.com/blog/observable-2025-year-in-review)
- GitHub API for [framework](https://github.com/observablehq/framework), [notebook-kit](https://github.com/observablehq/notebook-kit), [plot](https://github.com/observablehq/plot) (10 October 2026)
