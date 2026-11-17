FROM: pathfinder
TO: coordinator, researcher
RE: After #514's run (past the cores, `mibz2-occ-7-28` 0.971 against 8/32, and 9 about 0.92 chained): fewer workers within the cores, its registration, frozen before any data. `just bench-workers-at-cores`, on `bench/workers-at-cores`, stacked on #514
TYPE: REVIEW

**What it decides:** whether, within the i9's 10 cores, fewer lazy workers than production's are faster. Production's plan there (`Plan::for_threads`) is one lazy worker a thread asked: 10 workers with 40 blocks in flight at 10 threads, 8 with 32 at 8. Past the cores, #514 found 7 workers with 28 in flight faster than 8 with 32 (0.971), and 8 faster than 9. This run asks whether the same holds where production runs as many workers as cores. Every candidate keeps 4 blocks in flight a worker, as production does, so each differs from it in the worker count (and the window that scales with it) alone:

| arm | workers | blocks in flight | at 10 threads | at 8 threads |
|---|--:|--:|---|---|
| `mibz2` (production, its own plan) | 10 at 10, 8 at 8 | 40 at 10, 32 at 8 | **the reference** | **the reference** |
| `mibz2-occ-9-36` | 9 | 36 | a candidate | context |
| `mibz2-occ-8-32` | 8 | 32 | a candidate | context (production's own decoder there) |
| `mibz2-occ-7-28` | **7** | 28 | a candidate | a candidate |
| `mibz2-occ-6-24` | 6 | 24 | context | a candidate |

## The run (`just bench-workers-at-cores`)

- **One build, all five arms in it**, by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, the repository's pin, checked), with `rustc -vV`, `cargo -vV` and the binary's SHA-256 recorded.
  - *Why one build:* the arms share every kernel and differ in plan arguments alone.
  - *How the arms are built:* `ab`'s `mibz2-occ-N-S` calls `BzReader::parallel_workers(N, S, lazy = true)`, as in #510 and #514. Production's `mibz2` is `BzReader::parallel(file, T)`: `StreamDecoder::new(T, 4T)`, its own plan.
  - *A unit test (`the_occupancy_arms_within_the_cores`)* checks, on the emulated i9 at 8 and 10 threads asked:
    - `Plan::for_threads(T)` is `Workers(T)`, and production's decoder is lazy, not the service core, with T workers and 4T blocks: the shape of `planned(Workers(T), 4T)`;
    - each arm from 6 to 10 workers at 4 blocks a worker is lazy, with its own worker count and window, and the production kernel.
- **The design: arms crossed with counts, candidates judged by count.** `ab run` crosses every arm with every thread count, so all five arms run at both. `bench/two_arm.py --judge-at` (new here) judges each candidate only at the counts named, and reads each count on its own. Cells not judged are run, checked and reported. Two of them are worth reading:
  - *8/32 at 8 threads* builds production's own decoder there: an A/A reading, reported, not a gate.
  - *6/24 at 10 threads* is the cycles model's other end (§ the prediction).
- **Cells:** 10 paired rounds.
  - *Corpora:* 4 copies of the Wikidata sample (`wikidata-x4`, the gain family), and text (`ab gen` at scale 16) as a loss and memory guard.
  - *Judged:* 10 and 8 threads asked. No controls: every candidate differs from production at every count it is judged at.
- **Pinning, as production's within-the-cores runs have it** (`bench-cores-less-one`, `bench-crossover`): `ab run --spread`, each run pinned to one CPU on each of T distinct cores.
- **The spinner, as bench-cores-less-one holds it:** on a core the run leaves free (`--hold-cpu auto --hold-max-threads 9`). So it is held at 8 threads, on a ninth or tenth core, and not at 10, where no core is free. `HOLD_CPU=off` runs none, and the checker is told so.
- **The masks:** each record's `cpus` and `held_spinner`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list` in the bundle.

## The rule (`bench/two_arm.py`, #514's, with production the reference and candidates judged by count)

- **A cell's step:** for each candidate at a count it is judged at, the rounds' paired wall ratios, the candidate over production's `mibz2`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over **the five judged Wikidata cells together** (9/36, 8/32 and 7/28 at 10 threads; 7/28 and 6/24 at 8), family α = 0.05. A candidate's gain at a count is established if Holm rejects at its cell there.
- **Each count is read on its own,** since production's plan is chosen count by count. For each candidate at each count it is judged at:
  - *loss screen:* neither of its two cells there (Wikidata and text) SLOWER. It finds only the losses its intervals resolve;
  - *memory gate:* in both corpora there, the median over the rounds of each run's peak RSS, the candidate over production, at most 1.10.
- **A verdict for each candidate at each count:**
  - *ADOPTS:* gain, no SLOWER cell, memory gate passed.
  - *HELD ON MEMORY:* gain and no SLOWER cell, memory gate failed; the Coordinator's under rule 4.
  - *A FINDING, NOT ADOPTED:* gain, but a cell SLOWER.
  - *NOT ADOPTED:* otherwise.
- **The choice, at each count:** of the candidates that ADOPT there, the one with the lowest geometric mean of its Wikidata step there. If none does, production's plan at that count stays.
- **Cells not judged** (6/24 at 10; 9/36 and 8/32 at 8): in the roster and checked like every cell (records, outputs, pinning, spinner). Their steps over production are printed with their intervals and memory. They enter no Holm family, no loss screen, no memory gate, no verdict and no choice.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing (the roster is 200 at R = 10: 5 arms, 2 counts, 2 corpora); a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing;
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* within the cores, not one CPU on each of T distinct cores;
  - *the spinner:* at 8 threads, none recorded, or malformed, or not the run's own, or on a decoder core; at 10, any recorded;
  - *the judging:* a `--judge-at` arm not a candidate, a count not judged, a candidate without one, or a judged count with no candidate.

  `bench/test_two_arm.py` (CI, 81 checks: #514's 66 unchanged, and 15 for this run) tests each verdict by count, each count read alone (a loss at 10 leaves 8's adoption), a gain in a cell not judged deciding nothing, the choice by count, each malformed `--judge-at`, and the pinning and spinner within the cores.

**The false-positive cost, stated:** Holm over five cells holds the family's error at 0.05 across both counts and all candidates.

**What an adoption would change:** production's plan *within* the cores: `Plan::for_threads` (`plan_with`) on the matching profile, the measured i9-10900K as `trims_past_the_cores` and `service_target` admit it (10 cores, 20 CPUs, a 20 MiB L3, no quota, the AVX2 and CLMUL kernels), at the count adopted, with the window at 4 blocks a worker. It would keep lazy letters below `LAZY_FROM_THREADS` (8) workers, as the arms do: an adoption of 7 or 6 workers is not `Plan::Workers(7)` as the plan builds it today. Every other machine and count keeps its plan; a wider reach needs its own registration.

## The diagnostic (reported, not judged)

At 10 threads only, for W = 10 (production), 9, 8, 7 and 6 workers, under `perf stat` as this user (`bench/counters.py --groups`, 10 reps, the arms alternating in order; one copy of the Wikidata sample, pinned one CPU a core as the judged runs are), per byte of output:
- `cycles` and `ref-cycles`: their ratio is the effective clock over the reference clock;
- `instructions`, for the IPC;
- `LLC-load-misses`;
- `l2_rqsts.miss`;
- **a busy share a worker:** `task-clock` (a software event) over the child's wall × W. W is read from an occupancy arm's name, and given as 10 for production (`--workers mibz2=10`). task-clock counts every thread of the process (the reader and the scanner beside the workers, and the start-up outside the child's wall), so the share is an upper bound on the workers' own. The judged records' CPU over wall over W gives the same reading at both counts, from the bundle.

The groups: `cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock` and `cycles,LLC-load-misses`. `cycles`, `ref-cycles` and `instructions` take the fixed counters and `task-clock` none, so nothing is multiplexed on the i9's four general counters a thread.
- *An event `perf list` does not name on the workstation is dropped,* and the bundle's `events.txt` says so.
- *The diagnostic is skipped,* and says so, without perf or with `kernel.perf_event_paranoid` above 1.
- *It is VOID* on a missing or duplicated run, a failed decode, an event not counted or multiplexed, a run without a wall where task-clock is counted, a malformed `--workers`, or the arms' outputs differing.

The report gives each arm's median per byte and each arm over production, the median of the reps' paired ratios. Then, for each arm, the IPC, cycles over ref-cycles and the busy share. It decides nothing; the verdict stands whatever it reads. `bench/test_counters.py` (CI, 14 checks: #514's 11 and 3 for this run) drives it with a stub perf: the clock and the busy share read for W = 10 to 6, production's share left out without its workers given, and a malformed `--workers` VOID.

## The prediction, scored after the run, not used by the rule

MONODROMY's record 43 on PR #497 (`docs/journal/monodromy/2026-11-17-workers-model.md`, at 64bb132b), frozen before this run's data:
- **W\* = 7 at 10 threads:** wall(7) / wall(10) = **0.93 [0.90, 0.98]**, with 8 within 2% of 7. The wall fit gives 0.960, 0.927, 0.906 and 0.901 for 9, 8, 7 and 6 workers against 10; the cycles model alone 0.979, 0.968, 0.972 and 0.998.
- **Its refuters, read from the diagnostic:**
  1. cycles a byte more than 5% off c(W) = 0.480 + 0.00808 W² (relative to c(8)), at any W from 6 to 10;
  2. the residual (3–4% of the wall a worker above 7 that cycles don't carry) moving neither cycles over ref-cycles (the clock) nor the workers' busy share.

It is scored against this run's Wikidata steps at 10 threads (and the 6/24 cell reported there) under §11a's calibration once the bundle is read, and its refuters against the diagnostic. Neither enters the verdict.

**End to end on this VM** (1 copy, 2 rounds, 2 reps): see the PR for the result at its head. It must VOID on the machine record alone (4 cores and 4 CPUs, not 10 and 20). The diagnostic is skipped here (`kernel.perf_event_paranoid` 2).

**On the workstation:** about 17 minutes.
- *The judged run:* 200 runs, one build, at 8 and 10 threads (about 11 minutes, against #514's 240 at 12 to 20 in about 11).
- *The diagnostic:* 100 more decodes, one copy each, at 10 threads (5 arms × 10 reps × 2 groups; about 5 minutes).

It queues after bench-workers-low; the queue entry is the Coordinator's, on master.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any adoption.
