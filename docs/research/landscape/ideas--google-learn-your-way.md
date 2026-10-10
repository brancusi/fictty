# Learn Your Way (Google Research)

- **Maker:** Google Research / Google LearnLM team, with the research firm Bold Insight
- **URL:** https://blog.google/products-and-platforms/products/education/learn-your-way/
- **Status (10 October 2026):** a Google Labs research experiment announced September 2025. The
  evaluation paper, "An Experimental Evaluation of an AI-Powered Interactive Learning Platform"
  (arXiv 2509.18664), was published in Frontiers in Artificial Intelligence. We added this entity;
  it wasn't in the brief, but it is the clearest controlled test of generated multi-representation
  learning material we found.

## What it is

A system that turns a textbook chapter into personalised, multi-format material: the text
re-levelled and re-themed to the student's grade and interests, plus generated slides with
narration, audio lessons, mind maps, and quizzes with feedback. The student moves between
representations of the same content.

## The problem it's solving

A textbook is one fixed representation for every reader. Learning science says multiple
representations and retrieval practice help, but authoring them by hand per student is impossible.

## Its path / bet

Generate several representations per student from a trusted source text, and let the student
choose and switch between them, with quizzes woven in. The source stays the ground truth; the
model re-presents it.

## How it works (concretely)

LearnLM models personalise the text, then a pipeline generates the other formats. We couldn't
verify the internal representation (whether the slides and maps are data or code).

**The study:** a randomised controlled trial with 60 Chicago high-school students aged 15 to 18,
up to 40 minutes on one chapter (adolescent brain development), Learn Your Way against a standard
PDF reader. Google's blog reports an 11 percentage point advantage on a recall test three to five
days later; secondary reports give 78% against 67% (another says 77% against 68%), and about 9
points immediately after. Students preferred it.

## Strengths

- A real controlled comparison with a delayed test, which is what learning research asks for.
- The design follows the evidence: multiple representations plus retrieval practice, rather than
  decoration.
- Generated from a source text, which limits invention.

## Weaknesses / limits

- Small (60 students), one chapter, a few days' delay.
- Funded and co-designed by Google; the paper says Google was involved in design, analysis and the
  decision to publish.
- Doesn't separate which ingredient helped: the quizzes alone might explain much of the gain.

## Relation to fictty

**Inspire.** It's evidence that generated, interactive, multi-representation material can beat a
static text for retention, in a narrow setting. It is a browser product for students, nowhere near
fictty's market.

## Could fictty adopt it instead of building?

No. A Google Labs experiment, not a component.

## What fictty should take from it

- **Retrieval practice is the part with the best evidence.** A teaching screen should ask
  questions and record answers, which fictty can read back exactly. Quizzes, not decoration.
- **Switching representations of the same data** (table, chart, text summary) is cheap when the UI
  is data bound to one source. That is a natural fictty demo: one data source, three views, tabs.
- **Generate from a trusted source.** The screen's data should come from the real artefact (the
  code, the log, the table), not from the model's recollection of it, which is fictty's data
  sources principle stated for learning.

## Sources

- Google, [Learn Your Way](https://blog.google/products-and-platforms/products/education/learn-your-way/) (September 2025)
- [An Experimental Evaluation of an AI-Powered Interactive Learning Platform](https://arxiv.org/pdf/2509.18664) (arXiv 2509.18664; Frontiers in AI)
- Maginative, [coverage](https://www.maginative.com/article/learn-your-way-googles-new-ai-transforms-static-textbooks-into-interactive-personalized-lessons/)
- Bold Insight, [blog post on the study](https://boldinsight.com/insights/blog/your-ai-platform-is-live-but-is-it-really-serving-your-users-needs)
