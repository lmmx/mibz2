# Hybrid systems: reading and decoder design

**Coverage: partial, 25 November 2026.** Chapter 1 has been read from the authors' public sample. The requested full reading of Chapters 2–9 is awaiting access to the private book supplied in the earlier conversation. Those chapters are indexed below, not represented as read. See [source notes](source-notes.md).

The engineering purpose is to find changes to the organisation of decoding: which work must happen, what information crosses each boundary, what permits progress, and which resource constrains it. A useful mathematical model must help choose or reject one of those changes.

## Two separate collections

- [Chapter précis](precis/01-introduction.md) explain the source. They do not certify a theorem about mibz2.
- [Applications](applications/00-goal-and-design-space.md) are our own interpretations and proposals, checked against the current code and measurements. They have their own assumptions and falsifiers. A connection to an unread chapter is a reading question, not a sourced result.

The application notes are arranged by engineering decision, rather than forcing one proposal per chapter:

| note | decision it supports |
|---|---|
| [Goal and design space](applications/00-goal-and-design-space.md) | What is being optimised, and what kinds of reorganisation are candidates? |
| [Modeling the decoder](applications/01-modeling-the-decoder.md) | What belongs in the state, and what do flows and jumps represent? |
| [Completion and model validity](applications/02-completion-and-model-validity.md) | What would an argument about progress or robustness actually establish? |
| [Structural levers](applications/03-structural-levers.md) | Which boundaries, representations, credits, ownership transfers or modes could change? |
| [Next experiment](applications/04-next-experiment.md) | Which existing measurements and smallest additional contrast settle the next decision? |

## Reading map

The questions in the last column are proposed uses to investigate. They do not assert that a chapter proves the desired decoder property. The machine-readable coverage index is [chapters.tsv](chapters.tsv).

| chapter | coverage | application question for the full reading |
|---|---|---|
| 1. Introduction | [Read; concise précis](precis/01-introduction.md) | Can an explicit state/guard/reset model expose a useful change of organisation? |
| 2. The Solution Concept | Pending source | Which executions are admitted, including simultaneous events, termination and repeated zero-time transitions? |
| 3. Uniform Asymptotic Stability | Pending source | Which progress quantities could support a finite-job argument, and which asymptotic conclusions would be irrelevant? |
| 4. Perturbations and Generalized Solutions | Pending source | What happens when delays, measurements or thresholds move a trajectory across a guard? |
| 5. Set-Valued Analysis | Pending source | How should uncertain service, nondeterministic schedules and limits of event traces be represented? |
| 6. Well-Posed Hybrid Systems | Pending source | What assumptions keep nearby executions meaningful, and what extra assumptions would a completion-time claim need? |
| 7. Asymptotic Stability in Depth | Pending source | Which robustness claims could survive parameter or timing variation, without assuming performance is invariant? |
| 8. Invariance Principles | Pending source | How do memory safety, persistent progress and invariant operating regimes differ? |
| 9. Conical Approximation | Pending source | Can a local approximation around an operating regime predict a useful intervention, including its switching boundary? |

Each completed chapter note should eventually contain its argument, important definitions, theorem hypotheses, instructive examples, limitations and section references. Its decoder interpretation belongs in `applications/`, with a cross-reference. No placeholder chapter essays stand in for the missing reading.

## Claim labels

- **Proved:** follows from an explicit argument under stated assumptions; the scope of the model is part of the claim.
- **Measured:** an observation in the archived diagnostic, with instrumentation and workload limits retained. Measurement alone is not a mechanism proof.
- **Argued:** supported by code, accounting or evidence, with an unresolved inference stated.
- **Conjectured:** a proposed mechanism, gain or model reduction that still needs a discriminating check.

## Fit with the existing programme

The current [experiment programme](../../briefs/experiment-programme.md) already contains the scanner fix, stage-2 mechanism work, public-API recalibration and joint frontier. Its [dynamical layer](../../leads/structure-and-function.md) already calls for pulse interventions and the smallest predictive state. These notes give those activities explicit semantic obligations and candidate structural changes; they do not create another run queue.

Use the current [stage-1 report](../../math-targets/data/character-stage1/report.txt), rather than treating the older scanner-only profile as the latest evidence. Stage 1 measures the reader and causal event ordering. It also reports instrumentation perturbation; its classifications do not by themselves adopt a production default.

**Recommended starting point:** read the goal, choose one structural lever, and write the exact change and cheapest falsifier. Read the relevant theory to strengthen that decision. A larger catalogue is useful only when it changes which implementation is worth trying.
