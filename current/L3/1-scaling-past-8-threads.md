# 1. Scaling beyond eight threads

- **Doing:** resumed analysis of the scaling limit and reported opt-in pipeline prototyping.
- **In order to:** 0, further correct whole-decoder throughput.
- **Status:** doing, at Louis's direction recorded in [6699486962](https://github.com/lmmx/mibz2/pull/145#issuecomment-6699486962); no new i9 timing tonight. Earlier dispatch pause is superseded, not a dependency that survived release.

## Completed evidence: 1.1, 1.4, 1.5 and 1.6

Three-team candidates and the production policy passed their respective two-run rules; #234 merged and shipped in 0.2.5. Ten-thread production-policy speed ratios were 1.089 (1.075–1.103) and 1.088 (1.076–1.101); confirmation CPU 0.89 and peak memory 0.81. These are recorded verdicts, not fresh Observer adjudication. [Full prior evidence and limits](https://github.com/lmmx/mibz2/blob/bd440eac0c946aa3026ff13747984d9291eee25c/current/L3/1-scaling-past-8-threads.md).

Release notes #245 merged at e98976910672c8c9cbf5980ccaea402ccf5a420d. [Release-scaling files](https://github.com/lmmx/mibz2/tree/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.5) are now present on master. The [mechanism entry](https://github.com/lmmx/mibz2/blob/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/docs/journal/2026-11-07-knee-mechanism.md) reports 0.2.4 taking 11.9% longer at ten threads, 8.1–8.7% longer at twelve to twenty. This completes the former informative collection wait, without changing the policy's registered verdict. Raw timing was not reanalysed here; live History deployment was not checked.

## 1.2 Mechanism: analysis under review

- **Purpose:** explain the scaling limit to direct the next intervention, in order to serve 1.
- **Evidence as synthesised by the Coordinator:** at fixed team counts, extra helpers barely change throughput; archived isolated build/walk counters show table-byte pressure on last-level cache; traced mean live tables are 2.59–2.75 for four teams; placement and workspace matter. Full numerical claims E1–E7 are in the [entry](https://github.com/lmmx/mibz2/blob/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/docs/journal/2026-11-07-knee-mechanism.md).
- **Interpretation, not direct measurement of the complete mechanism:** fewer simultaneously live tables reduce contention enough to offset one fewer team. E4 attributes 31–35% outside active-table time to leader serial stages. E3 attributes flat helper scaling to memory saturation. The [Researcher brief](https://github.com/lmmx/mibz2/pull/162#issuecomment-6699480548) explicitly asks whether those attributions are valid and whether alternatives explain the observations.
- **Status:** review dispatched; no review result within this watermark. The Observer does not promote disputed or unchecked derivations to established causal findings.
- **Limits:** isolated-kernel counters, traced occupancy and timed production results have different boundaries. A successful policy does not isolate its mechanism; earlier control-offset and hardware/workload limits remain.

## 1.3 Overlap and placement: resumed

- **Doing:** Coordinator reports an opt-in prototype on research/team-pipeline. GitHub returned 404 for that ref; execution is reported, code/test completion not independently established.
- **In order to:** 1, overlap leader preparation/finishing with helpers' build/walk while retaining one table per team.
- **Prediction:** H1 pipelined 3 × 4 at twelve threads versus today's 3 × 3 at ten: 1.10–1.30×. Prediction only; greater table duty cycle may consume the gain.
- **Dependency:** Researcher review before registration freeze. Later target-machine tests require Louis; no new workstation run tonight. Preparation is not blocked on him.
- **Ranking:** Coordinator places H1 overlap first, H2 physical-core placement next, H3 smaller tables under teams third, H4 non-table hot-set reduction last. This records that ranking, not an Observer recommendation.
- **Source mismatch:** the entry's “Done here” section lists prototype, tests and registration, while its H1 section and newer dispatch describe prototyping and review before freeze. Completion is unconfirmed; use the explicit current dispatch, not that heading as proof of finished work.

## Accepted, not aimed at

The released policy's lower processor use and memory accompany its throughput gain. They do not establish the proposed pipeline's cost or qualify another change.
