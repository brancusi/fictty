# Ideas: disposable UI, malleable software and learning by seeing

The overview for the "ideas" cluster of the landscape (10 October 2026). It extends the first pass
in [the landscape post](../../blog/2026-10-why-the-terminal.md). Each entity has its own dossier
beside this file.

## The shift in one sentence

For forty years an interface was a durable, costly thing a team built once and many people used;
in 2025 and 2026 it became something a model writes in a minute for one person and one question.
Everyone in this cluster is working out what follows from that, and they disagree.

## The paths people are going down

**1. Make software adaptable, not disposable** ([malleable software](ideas--malleable-software.md),
[Robin Sloan](ideas--robin-sloan-home-cooked-app.md), [Maggie Appleton](ideas--maggie-appleton-barefoot-developers.md)).
The Ink & Switch camp says regenerating an app is not the goal; people need to change the tools
they have, compose them over shared data, and make small edits without a conversation. "AI code
generation alone does not address all the barriers to malleability." Home-cooked software is about
audience (a few people you know), not lifespan; Sloan's family app has lasted six years.

**2. Generate the interface per question** ([Google Generative UI](ideas--google-generative-ui.md),
[Claude inline visuals](ideas--claude-inline-visuals.md)). The labs' default: the model writes HTML
for each answer and throws it away. Google's study found raters preferred it to markdown 83 to 91%
of the time, with the minute-long wait hidden. Anthropic made the distinction product policy:
Artifacts are permanent, visuals are "temporary," built "to aid users' understanding."

**3. Fill a catalogue** ([ChatGPT visuals](ideas--chatgpt-study-mode-and-visuals.md),
[NotebookLM](ideas--notebooklm.md), [Gemini Guided Learning](ideas--gemini-guided-learning.md)).
Where correctness matters, the labs quietly pick fixed formats the model fills: about 70 interactive
concept modules in ChatGPT (our reading), mind maps and slide decks in NotebookLM, retrieved diagrams
and quizzes in Gemini. Reliable, narrower.

**4. Build instruments for understanding** ([Geoffrey Litt](ideas--geoffrey-litt-on-demand-tools.md)).
The sharpest version of the thesis: when agents write the code, the human's bottleneck is
understanding, and the fix is HUDs, not copilots: a one-minute custom debugger, a migration
"command center," an explainer with a quiz to pass before sharing. All throwaway, all bound to live
program state.

**5. Keep the old craft of explanation, now cheaper** ([Bret Victor](ideas--bret-victor-explorable-explanations.md),
[Nicky Case](ideas--nicky-case-explorables.md), [Distill](ideas--distill.md),
[manim](ideas--manim-3blue1brown.md)). Explorable explanations were right and too expensive: Distill
stopped in 2021 after editors spent 50-plus hours on single articles. Agents now write manim scenes
and interactive figures, with layout errors common enough that research pipelines keep a human.

**6. Durable reactive documents, now driven by agents** ([Observable](ideas--observable.md),
[marimo](ideas--marimo.md)). Notebooks are where data explanation already lives. marimo pair (2026)
lets Claude Code or Codex act inside a live notebook kernel and read values back: the nearest thing
in this cluster to fictty's loop.

**7. Tutors, mostly in text** ([Khanmigo](ideas--khanmigo.md), [Claude learning mode](ideas--claude-learning-mode.md),
[AI-tutor evidence](ideas--ai-tutoring-evidence.md), [learning science](ideas--dual-coding-multimedia-learning.md)).
The evidence says behaviour design matters more than the model: unguarded help harms learning
(Bastani, PNAS 2025), a well-designed tutor can beat a class (Kestin, 2025), and the largest
deployment shows 0.04 SD because students don't engage (Khanmigo, NBER 2026). The learning science
supports relevant pictures plus words (g = 0.37 across Mayer's studies) and simulations for transfer,
warns that decorative visuals hurt, and rejects "visual learners." No study we found tests a
generated interactive screen against a text tutor.

**8. The medium fight** ([Ptacek](ideas--ptacek-stop-making-tuis.md), [the replies](ideas--tui-replies.md)).
Ptacek: cheap UI means build native, not TUI. The replies: remote and keyboard use, Jane Street's
finding that text snapshots make agents better at UI, and joshka's call for a terminal protocol of
semantics rather than cells. Ptacek himself names "the New Terminal" as the strongest rebuttal.

## What's really being solved

Under all eight paths is one problem: **closing the gap between what the computer knows and what the
person understands, at the moment they need it.** Cheap UI matters because it lets the interface be
specific to that gap. The debate is about who owns the interface's lifecycle (the vendor, the user,
or the model for a minute), how it's checked (by a human designer, a schema, or nothing), and whether
it changes understanding or only preference.

Three findings cut across the cluster:

- **Preference is not understanding.** The headline numbers (Google's 83%) measure liking. The
  learning evidence is modest and says the interface must make the person act (predict, choose,
  answer) and must avoid decoration.
- **Data and code split along reliability.** Where correctness matters most (teaching maths, citing
  sources), labs choose catalogues and grounding over free generation.
- **Live state and read-back are the new requirement.** Litt's instruments bind to running programs;
  marimo moved agents into the live kernel; Jane Street's agents check screens as text. The loop
  where agent and person share one live surface, and the agent reads it exactly, is where the field
  is heading.

## Where fictty stands in this cluster

Honestly: in teaching and explanation, the browser wins on richness, and the labs (Claude visuals,
Gemini, ChatGPT) own the general case. fictty shouldn't pitch itself as an education product.

Its defensible ground here is narrower and real: **instruments for understanding what an agent is
doing, in the terminal where the agent works**. Litt's debugger and command center, built as data
bound to live sources, readable back exactly, and with checkpoints (`watch`) where the person
predicts or answers. That combines path 4's purpose, path 3's reliability, path 6's live loop and
the learning evidence's demand that the person act.

What to do because of this cluster:

1. Make "instrument" (step-through, scrubber, side-by-side) the headline use case and demo.
2. Ship `watch` and a question or checkpoint component; a teaching screen must read back what the
   person did.
3. Add named parameters that sources depend on (Victor's reactive document, Observable's dataflow).
4. Return health after a patch (marimo): which nodes rendered, which sources failed.
5. Ship a Claude Code skill that pairs with the Explanatory and Learning styles.
6. Keep the model semantic and independent of cells, so it can move to a "New Terminal" or a local
   GUI renderer (Ptacek's own remote pattern).
7. In the blog post, claim the multimedia principle and the cost collapse (Distill's 50 hours), not
   "people are visual learners"; and measure time-to-useful-screen against generate-a-page.

## Entities dropped or added

All entities in the brief exist and are covered. Added: Google's Learn Your Way (a controlled test of
generated multi-representation material), ChatGPT study mode and visuals (the catalogue
counterexample), Claude inline visuals (the clearest lab statement of temporary UI), the AI-tutor
evidence (Bastani, Kestin, Oreopoulos), and the TUI replies (Jane Street's Bonsai_term, joshka's
protocol). Split: Claude learning mode and Claude inline visuals are separate dossiers; Sloan,
Appleton and Litt are separate dossiers.

## What we couldn't verify

- Whether ChatGPT's interactive visuals are prebuilt modules (inferred from the fixed concept list).
- How Claude's inline visuals are built beyond "HTML."
- The exact arXiv date of Google's Generative UI paper (listing and ID disagree).
- NotebookLM's "Cinematic Video Overviews" (one secondary source).
- Whether Jane Street's Bonsai_term is open source.
- Any new Nicky Case explorable or Dynamicland release since 2024.
