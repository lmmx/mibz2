# The programme: from a toy problem to where new mathematics fits

*Louis, 24 November 2026: "my recollection was that we were doing this to define a good task to identify a toy problem (simulation setup) so as to basically do something like this https://www.cs.wustl.edu/~jain/iucee/index.html so as to then identify the 'systems biology' style model of the program so as to establish where exactly new mathematical devices would fit". This page holds that chain and its status. It is the first place to look.*

The reference is Raj Jain's course, *Computer Systems Performance Analysis: Techniques for Experimental Design, Measurement, Simulation, and Modeling*, based on *The Art of Computer Systems Performance Analysis* (Wiley, 1991). Its modules are:
- metrics and workload characterisation;
- summarising and comparing measured data, and regression;
- experimental design: 2^k, 2^k·r and fractional factorial;
- queueing networks, operational laws and mean-value analysis;
- simulation and its analysis.

## The chain

| step | what it is | Jain's module | our artefact | status |
|---|---|---|---|---|
| 1 | **The task and the referee:** what counts as success, in which currency | metrics and selection of techniques | `essence.md`, `council/comparison.md`, journal §§33–34, §42 | done |
| 2 | **The toy problem:** a fixed machine, workload and referee; payoff ceilings by oracle | workload characterisation; comparing systems | `math-targets/` (T1–T6), the four-arm oracle (§44), `reports/2026-11-24-from-search-to-formulated-problem.md` | done (T2, T3 and T6 ceilings still unmeasured) |
| 3 | **The system's structure:** activities, buffers, shared resources and the feedback loop | queueing networks | `briefs/decomposition.md`, three answers, `briefs/character.md` (§51) | designed |
| 4 | **Measurement across 1–20 workers** | measurement; summarising data | `bench-scan-profile` (#838) | **done** (journal §57): knee predicted at 7.49 from 1–4 workers, measured 7.67; best wall at 8 workers; efficiency ≥ 0.945 everywhere |
| 5 | **Experimental design:** baseline at every count, then interventions at the anchors | experimental design (OAT, 2^k·r, fractional factorial) | `briefs/character.md` stages 1–2 | with the Pathfinder, being built |
| 6 | **The model:** operational laws, a queueing or simulation model fitted and held out, a roofline per activity | operational laws; MVA; simulation | `briefs/character.md` §7 | after step 5's data |
| 7 | **Where new mathematics fits:** the activities with real control at the operating points that matter, and what a new device must change there (demand, dependency or representation) to move the whole decoder | (the purpose of the model) | `math-targets/targets.tsv`, to be re-ranked by step 6 | after step 6 |
| 8 | **The mathematics itself,** aimed at step 7's targets | — | MONODROMY and Neon records; the open threads in the report §4 | continuing; the closure problem answered negatively within record 119's class |

## The ideal-parts matrix: the current task (25 November)

**The current task is [`ideal-parts/`](ideal-parts/README.md):** measure the decoder with each part made ideal, alone and in combination (Jain's factorial design over oracles), to find each part's room. It fills steps 2 and 5's missing ceilings (T2, T3 and T6), and it is characterisation only.

## Hybrid systems: a reading companion for structural redesign

*25 November 2026.* The [hybrid-systems notes](methodology/hybrid-systems/index.md)
separate source précis from independent decoder interpretations. Current coverage is
explicitly partial: Chapter 1 is read; Chapters 2–9 await access to the supplied book.
The [structural levers](methodology/hybrid-systems/applications/03-structural-levers.md)
and [next analysis](methodology/hybrid-systems/applications/04-next-experiment.md)
use the completed stage-1 measurements to make changes to work placement,
representation, admission and ownership concrete.

For current run status, use the [experiment programme](briefs/experiment-programme.md):
stage 1 is done, stage 2 is being prepared, and scanner work and post-fix recalibration
have their own baselines. This companion introduces no additional workstation grid
or production adoption decision.

The historical MVA and factorial suggestions below have been superseded by the
[review decisions](briefs/character-review-decisions.md#0-where-all-three-agree).
The window is an admission ceiling in a finite pipeline, not a fixed circulating
population; exact product-form MVA is not justified for the actual system. Use MVA
only on a documented surrogate. The hybrid notes likewise distinguish semantic
validity, progress and robustness from a physical-time performance bound.

## Two points Jain's course adds to `briefs/character.md` (amendment, §52)

1. **Operational laws are the accounting identities.** Little's law, the utilisation law and the forced-flow law are the completeness checks ChatGPT Pro proposed. Because the admission window W = 4k holds a fixed number of blocks in flight, the decoder is a **closed** queueing network, the case **mean-value analysis** solves exactly for product-form networks. MVA is the analytic companion to the discrete-event simulation: cheap, and a check on it.
2. **Fractional factorial designs measure interactions without an exponential grid.** One-factor-at-a-time cannot see interactions (ChatGPT Pro's γ·xᵢxⱼ point), yet the four-arm result was itself an interaction. A 2^(9−4) resolution-IV design over the nine activities needs 32 runs. It estimates every main effect clear of two-factor interactions, with the interactions in aliased groups. It is proposed **at k = 8 and 10 only, in place of OAT there**, since those are the counts where control migrates. The cost: 2 anchors × 32 runs × 12 rounds = 768 observations, against OAT's 2 × 9 × 12 = 216. The Pathfinder is to judge whether the budget allows it, or whether to use a 16-run resolution-III design plus a fold-over.

