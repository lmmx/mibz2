FROM: pathfinder
TO: coordinator, researcher
RE: After #515's run (within the cores, fewer lazy workers than threads win: 7/28 at 10 threads 0.885 of production's ten, 6/24 at 8 threads 0.929): the lower counts mapped, its registration, frozen before any data. `just bench-workers-sweep-low`, on `bench/workers-sweep-low`, stacked on #515
TYPE: REVIEW

**What it is for:** the data for a production rule, **workers = f(threads, cores)**, before any such rule is written. So far there are three readings on the i9-10900K (10 cores, 20 CPUs, a 20 MiB L3):
- *Past the cores* (#514, `bench-workers-low`): 7 lazy workers with 28 blocks in flight beat 8 with 32 (0.971), and 8 beat 9.
- *Within the cores* (#515, `bench-workers-at-cores`, LOG 6943194948 on #145):
  - at 10 threads, 7/28 is **0.885** of production's 10 workers;
  - at 8 threads, 6/24 is **0.929** and 7/28 is **0.975** of production's 8 (5 was not run).
- *The pattern so far:* the best count is about **0.7 × threads**, or equally **threads − 2** at 8 and 10 (the decoder's scanner and consumer share the T CPUs a run is pinned to with its W workers).

This run asks what holds below: at 8, 6 and 4 threads asked. The two readings of the pattern part at 4 threads (0.7 × 4 = 2.8; 4 − 2 = 2) and at 6 they agree (4).

## Production's plan at these counts (`Plan::for_threads`)

On the i9, `plan_with(T, 10, 20 MiB)` is `Workers(T)` at 4, 6 and 8 threads asked (fewer threads than `WORKERS_BELOW_THREADS`, within the cores), with the window as asked, 4T blocks (`window`, untrimmed), and never the service core (that is 10 threads only). But **its kernel differs by count**: `Plan::lazy` is lazy letters from `LAZY_FROM_THREADS` (8) workers only.
- *At 8 threads:* 8 lazy workers, 32 blocks (as in #515).
- *At 6 threads:* **6 eager workers** (`Cells`, a slot for every letter), 24 blocks.
- *At 4 threads:* **4 eager workers**, 16 blocks.

Every occupancy arm is lazy (`ab`'s `mibz2-occ-N-S`, `BzReader::parallel_workers(N, S, lazy = true)`). So at 6 and 4, a candidate with fewer workers differs from production in its kernel *and* its count. **The lazy controls** separate the two: production's workers and window with lazy letters (6/24 at 6 threads, 4/16 at 4). Each is judged as a candidate too, so a gain is split into the kernel's (control over production) and the count's (candidate over control, read from the two steps).

## The arms and the cells

| threads asked | the reference | the candidates (all lazy, 4 blocks a worker) |
|--:|---|---|
| 8 | `mibz2`: 8 lazy, 32 blocks | `mibz2-occ-6-24`, `mibz2-occ-5-20` |
| 6 | `mibz2`: **6 eager**, 24 blocks | `mibz2-occ-6-24` (**the lazy control**), `mibz2-occ-5-20`, `mibz2-occ-4-16` |
| 4 | `mibz2`: **4 eager**, 16 blocks | `mibz2-occ-4-16` (**the lazy control**), `mibz2-occ-3-12`, `mibz2-occ-2-8` |

Eight judged cells on the Wikidata corpus, eight on text. 6/24 at 8 threads repeats #515's best cell there; 5/20 is the count #515 did not run.

## The run (`just bench-workers-sweep-low`)

- **One build, all six arms in it**, by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, the repository's pin, checked), with `rustc -vV`, `cargo -vV` and the binary's SHA-256 recorded.
  - *A unit test (`the_occupancy_arms_low`)* checks, on the emulated i9 at 4, 6 and 8 threads asked: `Plan::for_threads(T)` is `Workers(T)`; production's decoder is not the service core, has T workers and 4T blocks, and is eager at 4 and 6 and lazy at 8 (the shape of `planned(Workers(T), 4T)`); the lazy control `workers(T, 4T, .., true)` differs from it in the kernel alone; and each arm from 2 to 6 workers at 4 blocks a worker is lazy with its own count and window.
  - *An integration test (`the_low_occupancy_arms_one_output`, tests/lazy_workers.rs)* decodes the lazy-worker test streams (admitted blocks, blocks over the Cells budget, the admission witnesses, a short pattern, a block with a wrong CRC, a stream cut short) with 2 to 6 lazy workers and 4N blocks: one output and one error, the Segmented workers' own.
  - `ab`'s `occupancy_arms_parse` names 5/20, 4/16, 3/12 and 2/8.
- **One `ab run` a count, with that count's arms alone**, all appending to one file. Crossing every arm with every count (`ab run`'s default, #515's design) would spend most of the time on cells that judge nothing (2 workers at 8 threads): about 33 minutes against about 17. Within a count, the arms alternate round by round, as before; the counts run one after another (8, then 6, then 4), so each count's paired comparison is interleaved and no step crosses counts.
- **Cells:** 8 paired rounds (not 10: see the cost).
  - *Corpora:* 4 copies of the Wikidata sample (`wikidata-x4`, the gain family), and text (`ab gen` at scale 16) as a loss and memory guard.
  - *Judged:* 8, 6 and 4 threads asked. No controls in `two_arm.py`'s sense (no count where every arm builds production's decoder).
- **Pinning, as production's within-the-cores runs have it:** `ab run --spread`, each run pinned to one CPU on each of T distinct cores.
- **The spinner, as bench-cores-less-one holds it:** on a core the run leaves free (`--hold-cpu auto --hold-max-threads 9`), so at every count here. `HOLD_CPU=off` runs none, and the checker is told so.
- **The masks:** each record's `cpus` and `held_spinner`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list` in the bundle.

## The rule (`bench/two_arm.py`, #515's, with a sparse roster)

- **A cell's step:** for each candidate at a count it is judged at, the rounds' paired wall ratios, the candidate over production's `mibz2`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over **the eight judged Wikidata cells together**, family α = 0.05. A candidate's gain at a count is established if Holm rejects at its cell there.
- **Each count is read on its own,** since production's plan is chosen count by count. For each candidate at each count it is judged at:
  - *loss screen:* neither of its two cells there (Wikidata and text) SLOWER;
  - *memory gate:* in both corpora there, the median over the rounds of each run's peak RSS, the candidate over production, at most 1.10.
- **A verdict for each candidate at each count:** ADOPTS (gain, no SLOWER cell, memory gate passed); HELD ON MEMORY (gain and no SLOWER cell, memory gate failed: the Coordinator's under PERFORMANCE-POLICY rule 4); A FINDING, NOT ADOPTED (gain, but a cell SLOWER); NOT ADOPTED (otherwise).
- **The choice, at each count:** of the candidates that ADOPT there, the one with the lowest geometric mean of its Wikidata step there; the lazy control is one of them. If none does, production's plan at that count stays.
- **The sparse roster (`--sparse`, new here):** at each count the roster is production and the candidates judged there, no other arm. A record of a candidate at a count it is not judged at is outside the roster.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing (the roster is 176 at R = 8: 11 arm-count pairs, 2 corpora); a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing;
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* not one CPU on each of T distinct cores;
  - *the spinner:* none recorded, or malformed, or not the run's own, or on a decoder core;
  - *the judging:* a `--judge-at` arm not a candidate, a count not judged, a candidate without one, a judged count with no candidate, or `--sparse` without `--judge-at` or with control counts.

  `bench/test_two_arm.py` (CI, 90 checks: #515's 81 unchanged, and 9 for this run) tests a choice at each count from each count's own arms, the lazy control's gain judged at 4 while a loss at 6 stays at 6, an arm run where it is not judged, a judged cell or the reference at a count missing, the crossed roster rejecting a sparse run, `--sparse` without `--judge-at` or with controls, and the spinner checked at 4 threads.

**The false-positive cost, stated:** Holm over eight cells holds the family's error at 0.05 across all three counts and all candidates. With 8 rounds, not 10, the power at a small step is lower than #515's: its interval is about 17% wider at the same spread (t at 7 degrees of freedom over √8, against t at 9 over √10).

**What an adoption would change:** nothing by itself. This run's verdicts are evidence for the rule workers = f(threads, cores), which needs its own adoption registration, with the readings of #514, #515 and this run together. That rule would set `Plan::for_threads` (`plan_with`) on the measured i9 profile (as `trims_past_the_cores` and `service_target` admit it), at the counts measured, with the window at 4 blocks a worker and lazy letters below `LAZY_FROM_THREADS` (8) workers, as the arms have them. A lazy control that ADOPTS at 6 or 4 is a finding about `LAZY_FROM_THREADS` within the cores, for the same registration. Every other machine keeps its plan; a wider reach needs its own run.

## The diagnostic (reported, not judged)

For MONODROMY's record 44 (below), which needs cycles a byte and the busy share for every arm at every judged count. Under `perf stat` as this user (`bench/counters.py --groups`, 6 reps, the arms alternating in order; one copy of the Wikidata sample, pinned one CPU a core as the judged runs are), one `counters.py` run a count:

| threads | arms counted | groups |
|--:|---|---|
| 8 | production (8 lazy), 6/24, 5/20 | the first |
| 6 | production (6 eager), 6/24, 5/20, 4/16, **3/12** | both |
| 4 | production (4 eager), 4/16, 3/12, 2/8 | the first |

3/12 at 6 threads is counted, not judged, so that W = 6 down to 3 is read at one count.

- **The first group,** `cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock` (#515's), per byte of output:
  - `cycles` a byte;
  - `cycles` over `ref-cycles`, the effective clock over the reference clock;
  - `instructions`, for the IPC; `l2_rqsts.miss`;
  - **a busy share a worker:** `task-clock` over (the child's wall × W), W from the arm's name, and T for production (`--workers mibz2=T`). task-clock counts every thread of the process, so the share is an upper bound on the workers' own.
- **The second group,** `cycles,LLC-load-misses`, at 6 threads only: LLC misses a byte, record 43's discriminating counter at W = 4.

Nothing is multiplexed. An event `perf list` does not name is dropped and said so; the diagnostic is skipped and says so without perf or with `kernel.perf_event_paranoid` above 1; a count's report is VOID on #515's conditions, and the others stand. Each count's report (`counters-T.txt`) gives each arm's median per byte, each arm over production, and the IPC, clock and busy share. It decides nothing; the verdict stands whatever it reads. Production against its lazy control at 6 and 4 is the kernel's counter difference at equal W. `bench/test_counters.py` (CI, 15 checks: #515's 14 and 1 for this run) checks the busy share at 4 threads for W = 4 to 2 with one group and production given 4 workers.

## Predictions, scored after the run, not used by the rule

- **MONODROMY's record 44** on PR #497 (`docs/journal/monodromy/2026-11-17-workers-low-threads.md`, at abf03d26; LOG 6943291920 on #145), frozen before this run's data, is the prediction this run scores. It reads the residual of record 43 as the workers' busy share (wall ∝ c(W) / (W · u)), with c(W) = 0.423 + 0.00676 W² from W = 6 to 9, calibrated (×0.65 on the gains) on #515's two points. It predicts **the best W is 6 at 8 threads, 5 at 6 and 4 at 4**, against the 0.7 × T rule's 6, 4 and 3:
  - *8 threads:* 5/20 against the lazy 8 (production) 0.951 [0.92, 0.99];
  - *6 threads:* 5/20 0.97 [0.94, 1.00] of the lazy control and 1.00 [0.95, 1.05] of production; 4/16 1.01 [0.97, 1.05] and 1.04 [0.99, 1.09];
  - *4 threads:* 3/12 1.04 [1.00, 1.08] of the lazy control and 1.13 [1.07, 1.18] of production; 2/8 1.26 [1.15, 1.35] and 1.37 [1.25, 1.47]; the lazy control 1.09 [1.04, 1.13] of production (production stays better).
  - *Refuted if* 4 workers beat 5 by more than 2% at 6 threads, or 3 beat 4 by more than 2% at 4. Its counters: cycles a byte at W = 3 to 5 against 0.423 + 0.00676 W², and the busy share at T workers near saturation at 6 and 4 threads (deficit at most 0.05).

  It is scored against this run's Wikidata steps and the diagnostic's cycles a byte and busy shares, by count. It enters no verdict.
- **MONODROMY's record 43** on PR #497 (`docs/journal/monodromy/2026-11-17-workers-model.md`, at 64bb132b), frozen before #515's data, predicted that **a 6-core machine with a 12 MB L3 wants W\* = 4, or 5 within 2%**, with 6 workers 3 to 11% slower than 4. This run is not that machine: 6 threads on the i9 share a 20 MiB L3, so it bears on the prediction only as far as the model's own assumption (contention scaling with W against the L3's size) lets 6 threads of a 10-core stand for it. Its third refuter (the 6-core's best W at 5 or 6, not 4) is read here with that caveat, and its discriminating counter, LLC misses a byte at W = 4, is in the diagnostic.
- **The record's wall fit as it stands** (wall(W) = 3.080/W + 0.0783 W, fitted past the cores; its least at W = 6.3), applied here to W ≤ T (derived for this registration, not the record's own claim): at 8 threads, 6/24 0.972 and 5/20 0.997 of 8 workers; at 6, 5/20 1.025 and 4/16 1.10 of 6; at 4, 3/12 1.17 and 2/8 1.57 of 4. That is, fewer workers lose at 6 and 4 threads. The **0.7 × threads** pattern says 4 at 6 and 3 at 4; **threads − 2** says 4 at 6 and 2 at 4.

These are scored against the Wikidata steps once the bundle is read (the lazy controls' steps taken out, so the count's effect is read against lazy workers at T). None enters the verdict.

**End to end on this VM** (1 copy, 2 rounds, 2 reps): see the PR for the result at its head. It must VOID on the machine record alone (4 cores and 4 CPUs, not 10 and 20). The diagnostic is skipped here (`kernel.perf_event_paranoid` 2).

## The cost on the workstation: about 26 minutes

Scaled from #515's (200 judged runs at 8 and 10 threads in about 11 minutes, and its diagnostic's estimate of 5 minutes for 100 one-copy decodes at 10 threads), a run's wall about inversely proportional to its workers:
- *The judged run:* 176 runs, one build: about 17 minutes (8 threads about 3, 6 threads about 5, 4 threads about 9).
- *The diagnostic:* 102 one-copy decodes (3 arms × 6 reps at 8 threads, 5 arms × 6 reps × 2 groups at 6, 4 arms × 6 reps at 4): about 9 minutes, most of it at 4 and 6 threads.

ROUNDS and REPS are 8 and 6, not #515's 10 and 10, at the Coordinator's instruction, to keep the run near 25 minutes; at 10 and 10 it would be about 37. Its levers if the i9 runs longer than this: REPS=5 (about a minute and a half less).

The queue entry is the Coordinator's, on master.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any rule built on it.
