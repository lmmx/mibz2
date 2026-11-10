# 2026-11-09: The cooperative waits' spin audit (untimed)

**Result: spinning in `src/coop.rs` does not explain the run-fed-leader bundle's two modes.**
The mechanism under test was that, under load, threads waiting in the pool spin and burn CPU
while the threads they wait for are descheduled. That would double wall and CPU together. Two
things rule it out:
- **The bundle.** In the slow runs every thread's CPU grew by the same factor, about 2.2–2.4×.
  That includes the scanner and the reading thread, which never spin in `coop.rs`. The waits
  account for none of that. Nothing in it shows oversubscription either.
- **The counts.** A wait's spin budget is a number of iterations, not a time. So the spins a
  decode can make are bounded by its waits times their budgets, about 3.3 million at three
  teams of three. Deliberate oversubscription on a 4-core VM did not raise them.

This is a mechanism audit, not a measurement. No timing from this VM is evidence, and none is
claimed.

**The question:** the Researcher's review 6750405677 on #145 (its report at e4ce5f46,
`state/reports/bimodality-advice-6750333637/report.md`) found that CPU and wall doubling together
rule out pure descheduling with unchanged work. It left spinning, contention, frequency and
executed-work differences open, and asked for a bounded, untimed accounting audit before any
timed registration.

## 1. What the bundle shows

The bundle is `run-fed-leader/20261109T201013Z` on the `evidence` branch at `65ab4b95`; its
`runs.jsonl` was read from the archive. It holds 140 records, seven arms × 20 rounds, all at
10 threads on the i9-10900K, all with the same output hash. A run is called fast here if its
wall is below 0.21 s: 29 runs (28 if 0.21 s is set a little lower). It is a descriptive split,
not a judgement.

| | fast (29) | slow (111) | slow / fast |
|---|---|---|---|
| leaders' CPU (`mibz2-0..2`, sum) | 0.313 s | 0.701 s | 2.24 |
| helpers' CPU (`mibz2-coop-p*`, six, sum) | 0.544 s | 1.198 s | 2.20 |
| scanner's CPU (`mibz2-scan`) | 0.075 s | 0.175 s | 2.33 |
| main thread's CPU (`main`) | 0.020 s | 0.048 s | 2.36 |
| feeder's CPU (`mibz2-read`) | 0.011 s | 0.010 s | (at the 0.01 s resolution) |
| CPU / wall | 6.9 | 7.3 | |
| `runq_s` (mean) | 0.15 ms | 0.40 ms | |
| `psi_cpu_ms` (mean) | 0.55 | 1.23 | |
| `others_busy` (mean) | 0.19 | 0.20 | |
| `load1` (mean) | 2.71 | 2.82 | |

- **The growth is even across the threads,** not concentrated on the waiting ones. The helpers
  spend part of each run waiting at the build's barrier, and the leader waits to join the walk.
  Spinning would grow their CPU more than the scanner's. The scanner, which only scans and
  waits on channels, grew by as much as anyone. So did the main thread, which waits on a
  `Condvar`.
- **Run-queue time is negligible in both modes.** It is under half a millisecond in runs of
  140–370 ms. The parallelism (CPU / wall) is the same, about 7. Every thread did the same
  work at about half the rate, and none waited longer to run.
- **The load of about 3 was the harness's own.** At a 0.26 s gap between runs, the runs' mean
  CPU demand over a run's cycle is about 3.6 CPUs. `load1` rose from 0.97 at the first run to
  about 3.1–3.5 within the minute, while `others_busy` stayed near 0.2 CPUs throughout. With
  about 12 busy threads on 20 logical CPUs, nothing in the bundle shows oversubscription: the
  condition the spin hypothesis needs.
- **The fast runs come in streaks across arms:** streak lengths 3, 1, 3, 1, 1, 1, 1, 1, 3, 3,
  3, 1, 3, 4. Arm order doesn't explain them.
- **No cause can be read from the bundle.** Its frequency snapshots are taken before and after
  each child, not during it, and are too coarse to separate frequency from SMT co-scheduling or
  shared-cache contention. An even per-thread slowdown at unchanged parallelism fits all three.
  Spinning and descheduling both predict an uneven pattern, so neither fits.

## 2. The instrument

`coop::wait_stats` (feature `wait-stats`, off by default) counts each wait at the four sites
in `src/coop.rs`:
- `pool_helper_idle`: a pool's helper waiting for the next run (2,000 spins, then park);
- `pool_caller_join`: the caller waiting for its helpers to end the run (2,000, then park);
- `walk_ready_helper` and `walk_ready_leader`: a pipelined team's helper, or its leader, waiting
  for the build's end (`BARRIER_SPINS`, 4,096, then park with lever A).

Each wait is tallied on the waiter's stack and added to its site's counts once, when it ends:
- the waits, and those whose condition held at once;
- those that ended within the spin budget, and those that parked or yielded;
- the spins, the spins of the waits that went on to park or yield anyway, the parks and the
  yields.

Without the feature, the tally is a zero-sized type and its calls are empty, so the decoder
builds as before.

`examples/wait_audit.rs` decodes the bench's Wikidata corpus (`bench/wikidata.json`, SHA-256
checked) with the default's own constructor, `BzReader::parallel_pipelined_levers` with park
and small. It prints one JSON line of counts a round. Every round must give the same bytes.

## 3. The counts

The VM: 4 vCPUs, no SMT. The load was four busy shell loops in separate processes (`+4hogs`).
Each row is the mean of 3 rounds. "Spins/wait" counts only the waits whose condition didn't
hold at once. "Wasted" is the share of the spins whose wait went on to park anyway.

| plan, load | site | waits | parked | spins/wait | wasted |
|---|---|---|---|---|---|
| 3×3 idle | helper idle | 472 | 92.3% | 1,987 | 98.4% |
| | caller join | 233 | 65.4% | 1,996 | 99.7% |
| | walk, helper | 233 | 97.3% | 4,063 | 98.1% |
| | walk, leader | 233 | 76.8% | 4,057 | 98.9% |
| 3×3 +4hogs | helper idle | 472 | 93.9% | 1,970 | 97.0% |
| | caller join | 233 | 61.4% | 1,994 | 99.8% |
| | walk, helper | 233 | 97.6% | 4,068 | 98.2% |
| | walk, leader | 233 | 84.4% | 4,065 | 99.1% |
| 1×4 idle | helper idle | 702 | 94.2% | 1,973 | 95.8% |
| | walk, helper | 466 | 92.8% | 4,034 | 94.2% |
| | walk, leader | 233 | 53.4% | 3,867 | 94.1% |
| 1×4 +4hogs | helper idle | 702 | 92.8% | 1,944 | 96.1% |
| | walk, helper | 466 | 98.0% | 4,081 | 98.3% |
| | walk, leader | 233 | 76.1% | 4,040 | 98.4% |

All spins, per decode:

| plan | idle | +4hogs |
|---|---|---|
| 3×3 | 2.88 M | 2.96 M |
| 1×4 | 4.10 M | 4.31 M |
| 1×3 | 2.82 M | 2.80 M |

The raw lines are in `2026-11-09-spin-audit/counts.jsonl`, and `summarise.py` beside them makes
these tables' figures from them.

- **The spins per decode don't grow under oversubscription.** They moved by +3%, +5% and −0.5%.
  A wait spins at most its budget, and a decode's waits are fixed by its blocks: 233 team runs
  for the 233 blocks at or above `MIN_COOP`. So the spins are bounded at about 3.3 M a decode
  at 3×3, whatever the load.
- **What load does change is how often the leader parks.** At its join, the share of waits
  that parked rose from 53% to 76% (1×4), from 77% to 84% (3×3) and from 64% to 72% (1×3). A
  park costs a futex wait and an unpark, not CPU in a loop.
- **Most spins are spent for nothing, idle or loaded.** 94–100% of each site's spins belong to
  waits that parked anyway. That is a separate, idle-machine question (section 4). It is not
  the bimodality.
- **The spin CPU is an estimate.** It is the spins times one spin's cost, measured on an idle
  thread of this VM at about 9 ns. That gives about 0.03 s a decode. The cost printed in the loaded rounds, 15–18 ns, was timed
  while the hogs ran: it is wall time a spin, not CPU, and isn't used. On the i9 the cost per
  spin is not measured. Taking Intel's documented PAUSE latency for Skylake-derived cores,
  about 140 cycles, at 3.5–4.7 GHz (an assumption), the bound of 3.3 M spins is about
  0.10–0.13 CPU-s a decode. The slow runs' excess was about 1.1–1.2 CPU-s, an order of
  magnitude more, and it was spread over threads that don't spin.

## 4. What is left open

- **The bimodality's cause** remains unresolved. The bundle can't separate core frequency during
  the child, SMT co-scheduling of the decode's threads, and shared-cache or memory contention.
  Each would slow every thread evenly at the same parallelism, as observed. Telling them apart
  needs in-child evidence: `perf stat` cycles and instructions per thread, APERF/MPERF, or each
  thread's CPU and its sibling's occupancy. Any such run is a registration for the team to make.
  This audit makes none.
- **The spin budget's idle cost** is not established. Nearly every spin precedes a park on this
  VM. On the i9, by the estimate above, the budget could be a few percent to 10% of a fast run's
  CPU. Whether it ends a useful share of waits there (10 cores, no oversubscription) is not
  known from this VM. The spin-budget control (`Levers::spin2k`,
  `2026-11-07-pipeline-levers-result.md`) measured a shorter budget for the walk's waits, not
  the pool's. It is a candidate only.

**What changed:** nothing in the decoder's default build. There is a feature-gated counter in
`src/coop.rs` and an example, `examples/wait_audit.rs`, with a test of the counts under the
feature. The VM run: `wait_audit corpora/wikidata.bz2 --teams T --k K --rounds 3 --levers both`
at 1×4, 3×3 and 1×3, each idle and then beside four `while :; do :; done` processes. No fix was
prototyped: the mechanism it would fix was not found.
