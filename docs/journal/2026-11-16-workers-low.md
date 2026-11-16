FROM: pathfinder
TO: coordinator, researcher
RE: After #510's run (`mibz2-occ-8-32` ADOPTS over production's 9/36, wall 0.951 / 0.955 / 0.951 at 12 / 16 / 20 threads): fewer workers past the cores, its registration, frozen before any data. `just bench-workers-low`, on `bench/workers-low`, stacked on #510
TYPE: REVIEW

**What it decides:** whether, past the i9's cores, fewer than eight lazy workers are faster still. #510's run found eight workers with 32 blocks in flight faster than production's nine with 36 at every judged count. The trend 10 → 9 → 8 has not yet turned. This run asks whether it turns at 8, 7 or 6. Every arm keeps 4 blocks in flight a worker, so each differs from the reference in the worker count alone:

| arm | workers | blocks in flight | role |
|---|--:|--:|---|
| `mibz2-occ-8-32` | 8 | 32 | **the reference:** the ratios' denominator, the candidate production after #510 |
| `mibz2-occ-7-28` | **7** | 28 | a candidate |
| `mibz2-occ-6-24` | **6** | 24 | a candidate |
| `mibz2` (production) | 9 | 36 | **context:** run, checked and reported against 8/32, not judged |

## The run (`just bench-workers-low`)

- **One build, all four arms in it**, by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, the repository's pin, checked), with `rustc -vV`, `cargo -vV` and the binary's SHA-256 recorded.
  - *Why one build:* the arms share every kernel and differ in plan arguments alone.
  - *How the arms are built:* `ab`'s `mibz2-occ-N-S` calls `BzReader::parallel_workers(N, S, lazy = true)`. From 8 workers on that is the very decoder `parallel_planned(Plan::Workers(N), S)` builds, as #510 ran it.
  - *The lazy letters are held below 8 workers.* The plan chooses a slot for every letter below `LAZY_FROM_THREADS` (8) workers. `planned(Workers(7), 28)` would therefore change the kernel as well as the worker count. The arms hold lazy letters so that 7 and 6 differ from 8 in the worker count alone. **An adoption of 7 or 6** would mean production's plan past the cores keeps lazy letters below 8 workers: a change to `trims_past_the_cores`' plan on the measured profile, not `Plan::Workers(7)` as it stands.
  - *A unit test (`the_occupancy_arms`)* checks, on the emulated i9 at 12, 16 and 20 threads:
    - `workers(N, S, .., true)` is `planned(Workers(N), S)` for #510's arms;
    - 7/28 and 6/24 are lazy, with 7 and 6 workers and 28 and 24 blocks;
    - `planned` alone would not be lazy at 7 or 6.
- **Cells:** 10 paired rounds.
  - *Corpora:* 4 copies of the Wikidata sample (`wikidata-x4`, the gain family), and text (`ab gen` at scale 16) as a loss and memory guard.
  - *Judged:* 12, 16 and 20 threads asked. No controls: every arm differs from the reference at every judged count.
- **No spinner:** no judged count leaves a core free (`--hold-max-threads 0`; the checker voids on any spinner recorded).
- **The masks:** each record's `cpus`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list` in the bundle.

## The rule (`bench/two_arm.py`, #510's, with another reference and a context arm)

- **A cell's step:** for each candidate, the rounds' paired wall ratios, the candidate over `mibz2-occ-8-32`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over **the six Wikidata cells of the two candidates together** (2 arms × 3 counts), family α = 0.05. A candidate's gain is established if Holm rejects at one or more of its cells.
- **Loss screen, each candidate:** none of its six judged cells (Wikidata and text) SLOWER. It finds only the losses its intervals resolve.
- **Memory gate, each candidate:** at every judged cell, the median over the rounds of each run's peak RSS, the candidate over `mibz2-occ-8-32`, at most 1.10.
- **A verdict for each candidate:**
  - *ADOPTS:* gain, no SLOWER cell, memory gate passed.
  - *HELD ON MEMORY:* gain and no SLOWER cell, memory gate failed; the Coordinator's under rule 4.
  - *A FINDING, NOT ADOPTED:* gain, but a cell SLOWER.
  - *NOT ADOPTED:* otherwise.
- **The choice:** if both ADOPT, the one with the lowest geometric mean of its three Wikidata steps. If neither does, the reference, 8 workers with 32 in flight, stays.
- **The context arm** (`--context mibz2`): production's 9/36. It is in the roster and checked like every arm: the records, outputs, pinning and spinner. Its steps over 8/32 are printed with their intervals and memory. It enters no Holm family, no loss screen, no memory gate, no verdict and no choice. It shows, in the same run, where #510's 9 → 8 step stands beside 8 → 7 → 6.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing (the roster is 240 at R = 10, the context arm's included); a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing;
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* past the cores, not unpinned;
  - *a spinner* recorded anywhere.

  `bench/test_two_arm.py` (CI, 66 checks: #510's 57 unchanged, and 9 for this run) tests each verdict against another reference, and the context arm: reported but never judged, SLOWER or over the memory gate without effect, its records missing, run but not named, or named as the reference or a candidate (each VOID), its pinning checked.

**The false-positive cost, stated:** Holm over six cells holds the family's error at 0.05 across both candidates.

## The diagnostic (reported, not judged)

At 12 threads only, for 9/36, 8/32, 7/28 and 6/24, under `perf stat` as this user (`bench/counters.py --groups`, 10 reps, the arms alternating in order; one copy of the Wikidata sample, unpinned as the judged runs are past the cores), per byte of output:
- `LLC-load-misses`;
- `l2_rqsts.miss`;
- `l2_lines_out.non_silent`;
- `cycles` and `instructions` beside them, for the IPC.

The groups: `cycles,instructions,l2_rqsts.miss,l2_lines_out.non_silent` and `cycles,LLC-load-misses`, so that nothing is multiplexed on the i9's four counters a thread.
- *An event `perf list` does not name on the workstation is dropped,* and the bundle's `events.txt` says so.
- *The diagnostic is skipped,* and says so, without perf or with `kernel.perf_event_paranoid` above 1.
- *It is VOID* on a missing or duplicated run, a failed decode, an event not counted or multiplexed, or the arms' outputs differing.

The report gives each arm's median per byte and each arm over 8/32, the median of the reps' paired ratios. It decides nothing; the verdict stands whatever it reads. `bench/test_counters.py` (CI) drives it with a stub perf: named groups and three candidates read, groups not the run's, and a candidate's runs missing, each VOID.

## The prediction, scored after the run, not used by the rule

MONODROMY's record 41 on PR #497 (`docs/journal/monodromy/2026-11-16-workers-prediction.md`), frozen before this run's data: **W\* = 8 (interval 7 to 9)**. Against 8 workers, at each of 12, 16 and 20 threads:
- 7 workers at **1.03 [0.97, 1.10]**;
- 6 workers at **1.12 [1.03, 1.22]**.

It is scored against this run's Wikidata steps under §11a's calibration once the bundle is read. Record 42 (`2026-11-16-workers-counters.md`, the same PR) gives each of its two models' counter shapes at 9 → 8 → 7 → 6. It is scored against the diagnostic, in the same way. Neither enters the verdict.

**End to end on this VM** (1 copy, 2 rounds, 12 threads): see the PR for the result at its head. It must VOID on the machine record alone (4 cores and 4 CPUs, not 10 and 20). The diagnostic is skipped here (`kernel.perf_event_paranoid` 2).

**On the workstation:** about 15 minutes.
- *The judged run:* 240 runs, one build, as #510's (about 11 minutes there).
- *The diagnostic:* 80 more decodes, one copy each, at 12 threads (4 arms × 10 reps × 2 groups).

It queues after bench-workers-window.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any adoption.
