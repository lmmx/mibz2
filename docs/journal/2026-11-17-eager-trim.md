FROM: pathfinder
TO: coordinator, researcher
RE: After #521's low sweep (LOG 6945639925 on #145) and MONODROMY's record 49 (LOG 6945704457): eager or lazy at trimmed worker counts below 8. Its registration, frozen before any data. `just bench-eager-trim`, on `bench/eager-trim`, stacked on #521
TYPE: REVIEW

**The question:** at a trimmed worker count below 8, should a worker invert with eager Cells (a slot for every letter) or with lazy letters?

**Why it is open:**
- *Production's plan:* below `LAZY_FROM_THREADS` (8) workers, `Plan::Workers(N)` runs **eager** Cells (`Plan::lazy`).
- *Every trimmed arm measured so far was lazy* (`ab`'s `mibz2-occ-N-S`), in #514, #515 and #521.
- *A rule workers = f(threads, cores)* would choose 5 workers at 6 threads and 3 at 4. Through `Plan::Workers(N)`, those would run eager, a combination never measured.
- *What the low sweep found* (LOG 6945639925):
  - at 6 threads, lazy 5/20 took **0.963** of production's eager 6/24, and lazy 6/24 took **1.040**;
  - at 4 threads, production's eager 4/16 stayed best.

  So at equal W = T the eager kernel beat the lazy one at 6 threads, and the trimmed count won with the lazy kernel.

## Production at each count, on this branch

This branch has master merged in (at 3d011dd5, after #524). On the i9 (10 cores, 20 CPUs, a 20 MiB L3), `StreamDecoder::new` builds:

| threads asked | production's decoder | how |
|--:|---|---|
| 6 | **eager**, 6 workers, 24 blocks | `Plan::for_threads(6)` = `Workers(6)`, below `LAZY_FROM_THREADS` |
| 4 | **eager**, 4 workers, 16 blocks | `Workers(4)`, the same |
| 10 | **lazy**, 7 workers, 28 blocks | `trims_to_seven` (#524): the plan is `Workers(10)`, and the default replaces it with `workers(7, 28, k, true)` |

**At 10 threads, lazy 7/28 is production's own decoder.** `the_eager_occupancy_arms` and `the_occupancy_arms_within_the_cores` check that on the emulated i9: `new(10, 40)` has the same kernel, threads and window as `mibz2-occ-7-28`, `workers(7, 28, k, true)`. Its cell at 10 threads is therefore an **A/A check** (see the rule).

(#521's branch alone still had the old production at 10 threads: ten lazy workers with 40 blocks. Its occupancy tests asserted that, and after the merge they assert #524's seven. That change is the merge commit's.)

## The arms

- **The eager occupancy arms, new:** `ab`'s `mibz2-oce-N-S`, `BzReader::parallel_workers(N, S, k, lazy = false)`, so `StreamDecoder::workers(N, S, k, false)`. Below 8 workers this is the very decoder `planned(Plan::Workers(N), S)` builds, production's eager plan at N workers and S blocks. Each differs from its lazy twin `mibz2-occ-N-S` in the kernel alone.
- **Tests:**
  - *`the_eager_occupancy_arms` (unit, emulated i9),* for 5/20 at 6 threads, 3/12 at 4 and 7/28 at 10:
    - the eager arm has the `Cells` kernel, N workers and S blocks, and is not the service core;
    - it is shaped as `planned(Workers(N), S)`, and `Plan::Workers(N)` is not lazy;
    - its lazy twin differs in the kernel alone;
    - at 6 and 4 threads, production is eager with T workers and 4T blocks, so the arm differs from it in the count (and the window scaled with it) alone;
    - at 10 threads, production is the lazy twin's decoder.
  - *`the_eager_occupancy_arms_one_output` (tests/lazy_workers.rs):* 7, 5 and 3 eager workers with 4N blocks decode the lazy-worker test streams to one output and one error, the Segmented workers' own.
  - *`ab`'s `eager_occupancy_arms_parse`:* it parses `mibz2-oce-5-20`, `-3-12` and `-7-28`, keeps the two families apart, and rejects malformed names.

| threads asked | the reference | the candidates (4 blocks a worker) |
|--:|---|---|
| 6 | `mibz2`: eager 6, 24 blocks | `mibz2-oce-5-20` (eager), `mibz2-occ-5-20` (lazy) |
| 4 | `mibz2`: eager 4, 16 blocks | `mibz2-oce-3-12` (eager) |
| 10 | `mibz2`: lazy 7, 28 blocks | `mibz2-oce-7-28` (eager), `mibz2-occ-7-28` (lazy: production's decoder, the A/A) |

That makes five judged cells on the Wikidata corpus and five on text. Lazy 5/20 at 6 threads repeats #521's cell, side by side with its eager twin. Lazy 3/12 at 4 is not run: #521 measured it, and the eager count is the question there.

## The run (`just bench-eager-trim`)

As #521's, count by count:
- **One build** with every arm in it, by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, the repository's pin, checked). `rustc -vV`, `cargo -vV` and the binary's SHA-256 are recorded.
- **One `ab run` per count,** with that count's arms alone (6, then 4, then 10 threads), all appending to one file. Within a count the arms alternate round by round.
- **The corpora:** 4 copies of the Wikidata sample (`wikidata-x4`, the gain family), and text (`ab gen` at scale 16) as a loss and memory guard.
- **8 paired rounds.**
- **Pinning:** `ab run --spread`, each run pinned to one CPU on each of T distinct cores.
- **The spinner:** on a core the run leaves free at 6 and 4 threads (`--hold-cpu auto --hold-max-threads 9`). At 10 threads every core is the run's, so none is held, and the checker requires that none is recorded. `HOLD_CPU=off` runs none, and the checker is told so.
- **The masks:** each record's `cpus` and `held_spinner`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list`, are kept in the bundle.

## The rule (`bench/two_arm.py`, #521's: `--judge-at --sparse`, production the reference)

- **A cell's step:** for each candidate at its count, the rounds' paired wall ratios, the candidate over production's `mibz2`: the geometric mean and its 95% interval (Student's t on the logs). It is FASTER if the interval lies below 1, SLOWER if above, and INCONCLUSIVE otherwise.
- **Gain:** Holm's one-sided tests over **the five judged Wikidata cells together**, family α = 0.05.
- **Each count is read on its own.** For each candidate there:
  - *the SLOWER screen:* neither of its two cells (Wikidata and text) is SLOWER;
  - *the memory gate:* in both corpora, the median over the rounds of each run's peak RSS, the candidate over production, is at most 1.10.
- **A verdict for each candidate:**
  - ADOPTS: a gain, no SLOWER cell, and the memory gate passed;
  - HELD ON MEMORY: a gain and no SLOWER cell, but the memory gate failed (the Coordinator's to weigh under PERFORMANCE-POLICY rule 4);
  - A FINDING, NOT ADOPTED: a gain, but a cell SLOWER;
  - NOT ADOPTED: otherwise.
- **A choice for each count:** of the candidates that ADOPT there, the lowest geometric mean of its Wikidata step. If none adopts, production stays at that count.
- **VOID** (exit 1, no verdict), on #521's conditions:
  - the records: one missing, duplicated or outside the sparse roster (128 at R = 8: 8 arm-count pairs × 2 corpora × 8 rounds), a run that did not succeed, a non-finite figure, or a corpus whose outputs differ;
  - the machine: not 10 cores and 20 CPUs with each core's CPUs listed, or under a quota;
  - the pinning, or the spinner;
  - the judging arguments.
- **The A/A cell at 10 threads** (lazy 7/28, production's own decoder) is judged as a candidate, as the brief asks, and is in the Holm family. It can change nothing in production. Its reading, stated now:
  - *INCONCLUSIVE, as expected:* the harness held at 10 threads.
  - *FASTER or SLOWER, or ADOPTS:* the 10-thread cells moved without a change. Eager 7/28's verdict at 10 is then reported, but nothing is acted on before a repeat.

  This reading adds nothing to the rule's arithmetic.
- **Reported, not judged:** the eager arm over its lazy twin at equal counts, `--pair mibz2-oce-5-20/mibz2-occ-5-20` (6 threads) and `--pair mibz2-oce-7-28/mibz2-occ-7-28` (10 threads). For each, both corpora's paired-round step and the peak-memory ratio are printed. This is new in `two_arm.py`. A pair enters no Holm family, no SLOWER screen, no memory gate, no verdict and no choice, and is VOID if malformed, not among the run's arms, or never run together.
- **Tests:** `bench/test_two_arm.py` (CI) has 99 checks: #521's 90 unchanged, and 9 for this run. They cover:
  - a choice at each count, with the pairs printed at 6 and 10 and not at 4;
  - a SLOWER pair that judges nothing;
  - no pairs without `--pair`;
  - `--pair` without `--judge-at`, malformed, naming one arm twice, naming an arm not in the run, or naming two arms never run together;
  - the lazy twin missing at 10 threads.

**The false-positive cost, stated:** Holm over five cells holds the family's error at 0.05 across the three counts. The A/A cell takes one of the five places. With 8 rounds, as in #521, the 95% interval is about 17% wider than #515's 10 at the same spread.

**Memory, flagged beforehand:** an eager worker holds a slot for every letter. Eager 7/28 against production's lazy 7/28 at 10 threads is the cell most likely to meet the 1.10 memory gate, and if it does, its verdict is HELD ON MEMORY. At 6 and 4 threads, production is itself eager with more workers, so the eager arms hold fewer slots than it does.

**What an adoption would change:** nothing by itself. These verdicts are evidence for the kernel half of the rule workers = f(threads, cores), and `LAZY_FROM_THREADS` stays as it is. The rule would then be written in its own adoption registration, with #514, #515, #521 and this run together:
- If eager 5/20 is chosen at 6 threads, the rule can route trimmed counts through `Plan::Workers(N)` unchanged, eager below 8.
- If lazy 5/20 is chosen, the trimmed counts need the lazy kernel held explicitly, as `trims_to_seven` does at 7.

## The diagnostics (reported, not judged)

- **The thread split** (`bench/thread_split.py`, from the judged records' per-thread samples, at no run cost). For each arm, count by count, it reports the workers' busy share and the non-workers' CPU over wall. `--threads N` (new, tested) reads one count alone, so production's W can be given at that count: 6 at 6 threads, 4 at 4, and **7 at 10**, not T, since #524. The eager arms' W is read from their names (`mibz2-oce-N-S`, tested).
- **The cycles counters** (`bench/counters.py --groups`, as in #521) run under `perf stat`, with one copy of the Wikidata sample, pinned one CPU a core, and REPS repetitions:
  - *the arms:* at each count, production and every arm judged there;
  - *the group:* `cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock`, each event kept only if `perf list` names it;
  - *the figures:* cycles a byte, IPC, cycles over ref-cycles, and a busy share a worker from task-clock (production's W given as above). The eager arm's column is marked `e5-20`.

  The counters are skipped, and say so, without perf or with `kernel.perf_event_paranoid` above 1. Eager against lazy at equal W is the kernel's counter difference that record 49's cost model (Δa = 1.67, Δb = 0.030) predicts in sign.

## Predictions, scored after the run, not used by the rule

**MONODROMY's record 49** (PR #497, branch `claude/sharp-meitner-cx0v78` at 543185ea, `docs/journal/monodromy/2026-11-17-worker-rule.md`; LOG 6945704457 on #145) was frozen before this run's data. Its kernel model (eager cheaper at small W · W_run, lazy from W · W_run ≥ 56) predicts that eager wins at the trimmed counts:
- *eager 5 at 6 threads:* **0.90 [0.86, 0.95]** against production, better than lazy 5's 0.963 (0.962 in the record);
- *eager 3 at 4 threads:* **0.99 [0.95, 1.03]** against production;
- *eager 7 against lazy 7 at 10 threads:* **0.98 [0.95, 1.02]** (the `--pair` line).

Its own refuter: **if eager 5 does not beat lazy 5, the kernel cost model is wrong,** and the kernel stays chosen by measurement. These are scored against the Wikidata steps (and the pair line at 10) once the bundle is read. None enters a verdict.

**End to end on this VM** (`just bench-eager-trim 1 2 2`: 1 copy, 2 rounds, 2 reps; 4 cores, 4 CPUs): it ran in 2.5 minutes, build included. All 32 records were present (8 arm-count pairs × 2 corpora × 2 rounds), with one output per corpus. The verdict is **VOID**, as it must be here:
- the machine record is not the registered profile (4 cores and 4 CPUs, not 10 and 20);
- at 6 and 4 threads, the spinner shared a decoder core, since the VM has no free core.

The thread split printed at each count, with W = 5, 3 and 7 read from the arms' names. The counters were skipped and said so (`kernel.perf_event_paranoid` 2). Off the i9, production at 10 threads is ten workers (the split sees 10 worker threads), since `trims_to_seven` holds on the i9 alone. On the i9 it is seven, and the split's "worker threads seen" shows which.

## The cost on the workstation: about 14 minutes

The estimate is scaled from #515's judged runs (200 at 8 and 10 threads in about 11 minutes, so about 26 worker-seconds a run, with a run's wall taken as about 26 / W s), and from #521's diagnostic estimate (about 5 s a one-copy decode under `perf stat`):
- *the build:* about a minute, incremental after the bench-data fetch;
- *the judged runs:* 128 in about 11 minutes. That is about 4 at 6 threads (48 runs at W = 6, 5, 5), about 4 at 4 (32 runs at W = 4, 3) and about 3 at 10 (48 runs at W = 7);
- *the counters:* 8 arm-counts × 4 reps = 32 one-copy decodes, about 2 to 3 minutes.

REPS is 4, not #521's 6, to keep the run under 15 minutes. At 6 it would be about 15 to 16. ROUNDS is 8, as in #521.

The queue entry is the Coordinator's, on master.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any rule built on it.
