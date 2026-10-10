# Khanmigo (Khan Academy)

- **Maker:** Khan Academy
- **URL:** https://www.khanmigo.ai
- **Status (10 October 2026):** in production since 2023, sold to districts and free for teachers.
  Khan Academy reports sign-ups up 731% year on year (2024-25 against 2023-24). The best
  independent evidence is an NBER working paper from August 2026 (not peer reviewed).

## What it is

A GPT-4-class tutor and teaching assistant built into Khan Academy's exercises. It coaches rather
than gives answers (Socratic prompting), and helps teachers plan lessons.

## The problem it's solving

One-to-one tutoring works (Bloom's "two sigma" is the usual citation) but doesn't scale. A model
that tutors every student inside their practice could.

## Its path / bet

Text-first conversation attached to existing exercises, with guardrails so it doesn't hand out
answers. Visuals come from Khan Academy's existing content, not from the tutor generating them;
we found no evidence of Khanmigo generating interactive visuals.

## How it works (concretely)

A chat pane next to the exercise; the model sees the problem and the student's work, and is
prompted to ask guiding questions.

**The evidence.** Oreopoulos and Low, "One Click Away: AI Tutoring with Khanmigo in a Two-Year
School Experiment" (NBER w35620, August 2026): a cluster-randomised trial in 18 middle schools in
Hamilton County, Tennessee, over 2024-25 and 2025-26, with Khanmigo set to coach during a daily
25 to 40 minute remedial maths block. The pooled effect was 1.26 national percentile ranks per term
(0.040 SD; year one 0.020 SD, not distinguishable from zero; year two 0.084 SD). The authors say
the gain "resembles those from Khan Academy practice without AI assistance" and blame engagement,
not the model: the median student messaged on 33% of practice days, and 39.4% of student messages
were bare answers. Cost is about $15 per student per year. Khan Academy's own claim of being "8-14x
more effective" is about its full district package and is self-reported.

## Strengths

- Deployed at scale in real schools, with a serious independent trial.
- Guardrails against answer-giving, which other evidence (Bastani et al.) says matter.

## Weaknesses / limits

- Small measured effect so far, about the same as practice without AI.
- Students mostly don't use the conversation in a learning way: they type answers.
- Text chat is the whole interface; the tutor can't show you anything it didn't already have.

## Relation to fictty

**Inspire**, mostly as a warning. The most-deployed AI tutor is text, and its effect is small because
students don't engage with the conversation. That is an argument that the interface, not the model,
is the bottleneck, and that a tutor which only talks is not enough.

## Could fictty adopt it instead of building?

No. Different market, closed product.

## What fictty should take from it

- **Engagement is the problem to solve.** A screen that makes the next action obvious (one
  question, one control) may matter more than a better explanation.
- **Read back what the person did, not just what they said.** The Khanmigo data on "bare answers"
  came from logging messages. A fictty teaching screen can log every key and choice exactly; that's
  an evaluation tool as much as a UI.
- Don't claim learning gains we haven't measured. The best-funded tutor in the world shows 0.04 SD.

## Sources

- Oreopoulos and Low, [One Click Away: AI Tutoring with Khanmigo in a Two-Year School Experiment](https://www.nber.org/papers/w35620) (NBER w35620, August 2026); summary at [WINS Solutions](https://www.winssolutions.org/khanmigo-ai-tutoring-two-year-trial/)
- Khan Academy, [annual report](https://www.annualreport.khanacademy.org/)
- FutureEd, [Students tried Khanmigo's AI tutor; most didn't want its help](https://www.future-ed.org/students-tried-khanmigos-ai-tutor-most-didnt-want-its-help/)
- MSU EDLI, [report on Khanmigo usage](https://edli.commons.msu.edu/2025/07/11/report-on-khanmigo-usage-available/)
