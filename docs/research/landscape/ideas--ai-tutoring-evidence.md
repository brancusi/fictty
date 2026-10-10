# The evidence on AI tutors (Bastani, Kestin, Oreopoulos and others)

- **Maker:** independent researchers. The three studies that matter most here: Bastani et al.
  (PNAS, 2025), Kestin et al. (Scientific Reports, June 2025), Oreopoulos and Low (NBER, August
  2026).
- **URL:** https://doi.org/10.1038/s41598-025-97652-6 (Kestin) and https://www.nber.org/papers/w35620 (Oreopoulos)
- **Status (10 October 2026):** a split evidence base. We added this entity to anchor the learning
  claims; the brief asked for research on visual and text learning, and the AI-tutor results are
  the other half of that question.

## What it is

Controlled studies of whether LLM tutors help people learn.

- **Bastani et al., "Generative AI without guardrails can harm learning"** (PNAS 122(26), 2025).
  Nearly a thousand Turkish high-school maths students. Access to GPT-4 during practice raised
  practice scores, but when access was removed, students who had used a plain ChatGPT-like
  interface ("GPT Base") did 17% worse on exams than students who never had it. A version with
  learning safeguards ("GPT Tutor") largely removed the harm.
- **Kestin et al., "AI tutoring outperforms in-class active learning"** (Scientific Reports, 3 June
  2025). 194 Harvard physics students, randomised crossover over two weeks. A GPT-4 tutor built to
  follow pedagogical best practice and not give answers: students learned significantly more in
  less time and reported more engagement than in an active-learning class. The 2024 preprint said
  "more than twice as much"; the published version dropped that wording. Critics note the short
  intervention and no test of retention or transfer.
- **Oreopoulos and Low, Khanmigo two-year trial** (NBER w35620, August 2026, not peer reviewed). 0.04
  SD pooled, similar to Khan Academy practice without AI; engagement was the limit (see the
  Khanmigo dossier).

## The problem it's solving

Whether the cheapest tutor ever made actually teaches.

## Its path / bet

Randomised trials, short (Kestin) to long (Oreopoulos).

## How it works (concretely)

See above. The common thread: tutors designed to make the student do the thinking help; tools that
do the thinking for the student hurt; and real-world use is limited by whether students engage at
all.

## Strengths

- Real randomised designs, published or circulated by credible venues.
- Consistent on the mechanism: the guardrails matter more than the model.

## Weaknesses / limits

- All three tutors are text chat. None tests generated visuals or interactive interfaces.
- Short interventions (Kestin), single countries, specific subjects.
- The positive and negative results come from different settings, so they don't cancel neatly.

## Relation to fictty

**Inspire.** Two points for us. First, the evidence on AI learning is about *behaviour design*
(make the learner act) more than about medium. Second, nobody has tested whether a generated,
interactive screen beats a text tutor; that's an open question, not a settled win for visuals.

## Could fictty adopt it instead of building?

Not applicable.

## What fictty should take from it

- **Make the person act.** A teaching screen should ask for a prediction, a choice, a step, and
  read it back. A screen the person only looks at is GPT Base with pictures.
- **Don't build answer-giving screens for learning.** A screen that shows the finished solution is
  the harmful pattern.
- **There's room for a small study.** "Text tutor against text tutor plus an interactive screen" on
  a developer task (understand an unfamiliar module, then answer questions a day later) would be
  new evidence. fictty's exact read-back makes it cheap to log.

## Sources

- Bastani et al., [Generative AI Without Guardrails Can Harm Learning](https://papers.ssrn.com/abstract=4895486) (SSRN; published in PNAS 122(26), 2025); Penn CHIBE, [summary](https://chibe.upenn.edu/news/without-guardrails-generative-ai-can-harm-education/)
- Kestin, Miller et al., [AI tutoring outperforms in-class active learning](https://pmc.ncbi.nlm.nih.gov/articles/PMC12179260/) (Scientific Reports, June 2025); ETC Journal, [critique](https://etcjournal.com/2025/11/10/review-of-kestin-et-al-s-june-2025-harvard-study-on-ai-tutoring/)
- Oreopoulos and Low, [One Click Away](https://www.nber.org/papers/w35620) (NBER w35620, August 2026)
