# Gemini Guided Learning (Google)

- **Maker:** Google (Gemini app, LearnLM models)
- **URL:** https://blog.google/products-and-platforms/products/gemini/new-gemini-tools-students-august-2025/
- **Status (10 October 2026):** launched 6 August 2025, a week after ChatGPT's study mode. Shipping
  in the Gemini app. Students in the US, Japan, Indonesia, South Korea and Brazil were offered a
  free year of Google AI Pro at launch. Closed source.

## What it is

A Gemini mode, switched on from the prompt bar, that walks through problems step by step, asks
questions and adapts explanations, aiming at the "how" and "why." Launched with it: Gemini adds
images, diagrams and YouTube videos to ordinary answers on its own when a topic is complex (the
examples were photosynthesis and the parts of a cell), plus interactive quizzes, flashcards and
study guides.

## The problem it's solving

Answer-giving chatbots let students skip the thinking; text-only answers are weak for visual,
structural topics.

## Its path / bet

Socratic behaviour plus retrieved visuals (existing images, diagrams and videos), and the separate
Generative UI work (dynamic view) for generated interfaces. The visuals here are mostly found, not
made.

## How it works (concretely)

LearnLM-tuned Gemini models; image and video retrieval inserted into answers; quiz and flashcard
generation from conversation or uploaded materials.

## Strengths

- Combines the two learning ingredients with the best evidence: retrieval practice (quizzes) and
  text with relevant pictures (the multimedia principle).
- Found visuals (real diagrams, real videos) avoid generated-diagram errors.

## Weaknesses / limits

- We found no published evaluation of learning outcomes for Guided Learning itself.
- Retrieved images are generic; they aren't about your problem or your data.

## Relation to fictty

**Watch.** A lab-scale bet on visual, interactive tutoring in a chat. Not a terminal or developer
product. It's evidence of where AI learning is heading (visual, quizzed), not a competitor.

## Could fictty adopt it instead of building?

No.

## What fictty should take from it

- **Quizzes are cheap and they're the part with evidence.** Same lesson as Learn Your Way.
- **Found before made.** Where a real picture exists (a diagram in the repo, a chart a command
  already produces), show it rather than generate one. fictty's image and SVG support should make
  "show the file" trivial.

## Sources

- Google, [New Gemini app tools to help students learn](https://blog.google/products-and-platforms/products/gemini/new-gemini-tools-students-august-2025/) (6 August 2025)
- TechCrunch, [Google takes on ChatGPT's study mode with Guided Learning](https://techcrunch.com/2025/08/06/google-takes-on-chatgpts-study-mode-with-new-guided-learning-tool-in-gemini/)
- 9to5Google, [Gemini Guided Learning](https://9to5google.com/2025/08/06/gemini-guided-learning/)
