# Stop-and-walk adoption review

**One targeted correction: measure the production execution plan before calling the verdict ADOPTS.** At the reviewed head the forced arms do not represent the shipped i9 decoder at 12/20 requested threads. Keep those forced-worker readings as diagnostics, or replace the judged arms with the shipped plan/window with only the inverse kernel substituted. Re-freeze that concrete contract before data.

Assignment: [6920377932](https://github.com/lmmx/mibz2/pull/162#issuecomment-6920377932). Registration: [6920355683](https://github.com/lmmx/mibz2/pull/145#issuecomment-6920355683). Exact reviewed head: **5870fd3a20b69821536c072e22d650149bb080b6**. SPEC pin a2da49638ac54d7763de1c167bd054f67ffdc763. No timing or decoder implementation.

## Decisive source mismatch

In examples/ab.rs:1153–1182, lazy calls parallel_workers(t,4t,...) and stop15 calls parallel_workers_with(t,4t,...). src/parallel.rs:243–264 deliberately forces t workers. These are internally matched arms: same workers, same window, different kernel.

But src/parallel.rs:plan_with chooses Workers(phys) when requested threads exceed physical cores and phys lies in [LAZY_FROM_THREADS, WORKERS_BELOW_THREADS). At this head those constants are 8 and 11. Therefore, on ten physical cores:

| requested threads | forced arms' workers | shipped plan's workers |
|---:|---:|---:|
| 8 | 8 | 8 |
| 9 | 9 | 9 |
| 10 | 10 | 10 |
| 12 | 12 | 10 |
| 20 | 20 | 10 |

The code's comment explicitly describes avoiding extra SMT workers on the i9. The window function preserves requested slots for Workers plans; retain the actual production slot calculation too, rather than equating a capped 20-thread request with a 10-thread request. The benchmark's --spread gives one CPU per core only while t<=physical cores; 12/20 run unpinned. This is material for the proposed memory-pressure mechanism.

Consequently a gain only at forced20 could trigger adoption even if the actual capped10 production path never benefits; a forced20 loss could also reject a useful production change. Neither is a production-default verdict. This is the principal reason not to return PROCEED on the present registration.

## The three requested design questions

**Held spinner at ten.** The same spinner treatment on both arms makes the paired contrast admissible *for that environment*. It does not mathematically cancel an interaction with the kernels: the latency-bound walk and shipped writer can respond differently to shared-core contention. pick_hold selects a spare physical core first, otherwise an SMT sibling, and marks overlap. At ten the stated overlap is expected; at 12/20 this recipe disables the spinner (hold-max-threads=10). Archive the actual mask and overlap; do not treat counts as one homogeneous contention regime. I would not let an overlap-only gain establish an unqualified deployment benefit. In the corrected pre-data design, make the shared-core ten-thread case diagnostic or use a non-overlapping judged condition fixed before collection. Do not launch another run merely to recover a favourable result.

**Multiplicity.** Five two-sided 95% intervals are marginal intervals, not a 95% familywise gain test. Under independent, exact, symmetric null tests, the current probability of at least one FASTER and no SLOWER is
0.975^5 - 0.95^5 = **0.10731475585937511**.
Dependence can change this value; it is a counterexample, not an estimate for this i9. “Slower at none” does not remove selection. Only beta1.5 belongs in the gain family: beta1.4/1.6 must not be promoted after inspection.

Recommendation for the corrected freeze: Holm's one-sided gain tests across the five predeclared production cells, family alpha0.05 (or Bonferroni p<=0.01 for at least one gain, simpler). Keep the policy's loss screening unchanged and call overlap **inconclusive**, not proof of equality. This respects the standing pragmatic policy; I am not imposing statistical non-inferiority on every cell. A noisy loss can still evade a “no detected loss” screen, so make that limitation explicit.

**Workload.** Four concatenated copies extend duration and repeat cache/allocation regimes; they do not create four independent corpora. Beta and the design were developed against Wikidata. It is a relevant primary workload, but insufficient alone to claim general default performance. Add the already-defined text corpus as the modest held-out slower-at-none guard on the corrected production grid, plus its RSS check. A -6 recompression of Wikidata tests block-size sensitivity, not independent content; useful as diagnostic, not an equivalent replacement for text. No claim of exhaustive workload coverage follows. CI paired regressions, exact-head correctness/B5 and green CI remain separate adoption obligations under the policy.

## The checker also needs to enforce the frozen run

I executed only the Python adjudicator on fabricated data. The reviewed stop_walk.py returns **ADOPTS**, exit0, with:
- two rounds of lazy/stop15 at each of the five counts (20 total rows);
- stop14 and stop16 entirely absent;
- an empty dose file;
- synthetic equal outputs and constant stop15/reference wall ratio0.9.

The registered roster is **200 main rows and 80 dose rows**, before any additional workload. The program pairs intersections, accepts fewer than ten repetitions, overwrites duplicate (arm,threads,round) keys and omits corpus from that key. Adding text without correcting the key would mix/overwrite workloads. Missing output hashes can also compare equal as None. This must be fixed or independently checked before the printed verdict is evidence.

Reject missing/duplicate/unexpected keys, incomplete arm/corpus/thread/round sets, absent hashes/output sizes, nonfinite or nonpositive timing/RSS, and failed terminations. Compute from complete pairs; do not drop bad rows. The dose-of-doses currently zips lists rather than matching round IDs; align by round even for the reported statistic. A missing RSS value is invalid evidence, not a memory-growth exception for the Coordinator to waive. The recipe catches checker failure and still bundles: archive retention is fine, but successful submission is not a valid verdict.

The synthetic fixture/result are included alongside this report. They exercise the exact downloaded checker, not a rewritten approximation.

## Memory and scope

The registration's concrete rule makes median-of-per-run-peak RSS a **gate** at every judged count; the assignment's “recorded, not judged: peak RSS” is contradictory shorthand. Freeze the former explicitly. The +10% test is a ratio of medians, not the maximum observed peak; retain individual peaks. A policy justification of genuine growth is distinct from waiving absent measurements. The local 37/37 and 83/85 MB claims motivate the run; they do not establish memory at 8–20 threads.

The journals explain why the old n/32 stop failed and why the budget stop/one-lane emitter is worth measuring. The +12.3-instruction claim and VM kernel slowdown are not independently reproduced here. They neither qualify nor veto a whole-decoder gain at the deployed concurrency. I did not review new unsafe paths or certify B5.

Expected outcome: decide replacement of the shipped lazy kernel. Actual finding: the proposed high-count arms alter deployment topology, and the checker can accept incomplete evidence. The empirical speed gap remains unmeasured. What was learned: the relevant mechanism is pressure under the *capped production worker count*, not arbitrary requested-worker oversubscription. The next run has high integration value after that mismatch is removed; a forced20 win alone has only diagnostic value.

## Evidence

All repository source below was read at the exact reviewed head:
- .just/bench.just, bench-stop-walk;
- examples/ab.rs, arm dispatch, spread and held-spinner selection;
- src/parallel.rs, workers/worker_with, plan_with, window and constants;
- bench/stop_walk.py;
- docs/team/PERFORMANCE-POLICY.md;
- docs/journal/monodromy/2026-11-16-stop-walk-reopened.md;
- docs/journal/2026-11-16-lazy-stop-walk.md;
- docs/journal/release-attribution.md.

Migration head/digest verified; fresh inbox/cursor and routing audit passed. Research only on team-state/researcher; no production edits, timing, merge or release.
