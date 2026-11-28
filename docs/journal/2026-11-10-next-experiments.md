# What to run next: four workstation experiments and three CI ones

10 November 2026. The Pathfinder, at Louis's instruction in this session: "get them sent to the
coordinator with an instruction to get experiments onto the workstation queue ASAP (CI runs are
fine too but the workstation is the blocker for merging so now that we are trying to achieve
asserted gains ASAP this is priority, we absolutely want to get this in record time now GO)".

Each item has a rule set before its data and an integration decision at its end. Figures are
recounted from archived bundles; nothing here was timed.

## W1. The cold start: a decode begun on an idle machine runs 2.2× slower

**The fact** (`two-modes/20261110T090731Z`, SHA-256 `8473bab8…da84856`, recounted from
`gaps/runs.jsonl`):
- after a 0.25 s idle gap, 26 of 30 runs are slow; with no gap, 0 of 30;
- slow: wall 0.303 s, CPU 2.27 s. Fast: wall 0.136 s, CPU 1.008 s. Every thread grows alike
  (0.09 → 0.23 s); run-queue wait is 0.0002–0.0004 s in both;
- the gap just before decides it, not history: gap after no-gap 15 of 16 slow, no-gap after gap
  0 of 16;
- the two probe passes had the same 0.26 s gaps and were fast 120 of 120 (100 probed, 20 not).
  Inference, not shown: the harness was at work in those gaps and asleep in the gap block's,
  so a machine that is not idle stays fast. That would also be why no slow run was probed.

**Why it is a product matter, not a bench artefact.** A command-line decode, or a library call
from an idle program, starts after an idle gap. On this machine that decode takes 0.30 s, not
0.14 s.

**The run:** the gap block's harness, the production binary, the Wikidata sample, 10 threads,
30 runs an arm, order randomised from a fixed seed, a 0.25 s gap before every run unless stated.

| arm | what | models |
|---|---|---|
| G | the gap alone | control, slow expected |
| Z | no gap | control, fast expected |
| P | the child held to CPUs 0–9, one a physical core (`taskset`, no code) | pinning a thread a core |
| W | 20 spinners on CPUs 0–19 for 20 ms, ending as the child launches | a pool that spins before it decodes |
| B | one spinner on CPU 19 through the gap and the run | a machine that is not idle |
| L | the input four times over, after a gap; output by 100 ms window | how long the slow state lasts |

Read passively on every run, with no spinning probe: `scaling_cur_freq` of CPUs 0–19 at the
child's exit, and each decoder thread's last CPU.

**Rule, before the data** (fast is wall under 0.21 s, as registered for #318):
- P fast in at least 25 of 30: pin a thread a core on the target-gated path. Integrate.
- W fast in at least 25 of 30: a start-up spin in the pool, target-gated. Integrate.
- B alone: nothing for the library; the bench's settle changes; record it.
- L and the passive readings are reported, not judged.

**Cost:** about 180 runs of 0.15–0.6 s: three minutes.

## W2. #324 on the i9, where the default plan runs

**The gap:** #324's gains are CI's, at 1, 2 and 4 threads, and they shrink with threads
(Wikidata −12.8 to −15.8% at 1, −7.7 to −10.3% at 4). The i9's default is ten threads on the
team path. Whether the adopted gain reaches it is unmeasured.

**The run:** `bench-team-ports`' recipe with two commits changed: #324's merge candidate against
its base. Wikidata and text, 10 paired rounds, threads 1, 2, 4, 8, 10, 12, 16, 20.

**Rule:** under `PERFORMANCE-POLICY.md`: a primary corpus slower beyond noise at 10 threads goes
to Louis before release; anything else ships as decided. The figures go in the changelog.

## W3. The cooperative ports: already queued. Read one more number from its bundle.

`release-scaling/run-20261110T091259Z` (SHA-256 `71d029b3…469b9d`), 0.2.7: 406 MB/s at 1 thread,
1,514 at 10, 1,412–1,466 at 12–20.
- At 10 threads the decode uses 0.99 CPU-seconds of 1.39 core-seconds: **71% of ten cores.** At
  20 threads, 36%.
- From the gap block's fast rows: leaders about 70% busy, helpers 55–60%, the scan thread 52%.

So the ports address instructions, and idle cores are the other lever. `bench-team-ports`'
bundle carries `thread_cpu`: report each arm's busy share beside its wall. No new run.

## W4. The parallel scan against the production default

**The fact:** `par-scan/20261110T030608Z`, VALID: find over scan 1.615 on `wikidata-l1` and
1.780 on `records` at 10 workers. It sits in `examples/par_scan.rs`. Both arms were the
share-nothing harness; the production default on those files was not an arm.

**The run:** `bench-par-scan`'s recipe with one arm added: `ab child mibz2` at 10 threads.
Inputs `wikidata-l1`, `records`, and `wikidata` as the control. 10 paired rounds.

**Rule:** find at least 1.15× the default on `wikidata-l1` or `records`, its interval excluding
1: build the selector (by the block's bound) as a production path. Otherwise drop it.

## C1. One walk direction (CI, one thread)

`examples/phases.rs`: the backward table filled row by row beside `bwt::build`, on `random` and
`records` blocks. **Rule:** fill plus backward walk within noise of build plus forward walk:
the successor walks backward on every block, and the fallback, its regression and the
admission cliff go together.

## C2. Admission by length (CI)

`takes_runs` on the block's rows, not its bound, with a corpus cell holding a 200–440 kB
high-entropy block. **Rule:** that cell within the policy's 5%. Closes the +10–11% disclosed
with #324's adoption.

## C3. `BufRead` on the reader (CI, API)

The reader has none; a line or JSON consumer copies every decoded byte again through a
`BufReader`. No bench cell sees it. **Rule:** the existing cells unchanged; a `lines()` consumer
measured once.

## Order for the queue

W1, then W2, then `bench-team-ports` as queued, then W4. W1 is three minutes and can go ahead
of everything.

## Results read (10 November, later)

**W3, the ports' busy share** (`release-scaling/run-20261110T095909Z`, SHA-256 `a34231fb…dd84a8`,
checked). Wikidata, medians of 10 rounds; busy share is CPU-seconds over ten cores' wall.

| threads | ports MB/s | base MB/s | ports CPU s | base CPU s | busy, ports | busy, base |
|--:|--:|--:|--:|--:|--:|--:|
| 8 | 1,343 | 1,292 | 1.01 | 1.05 | 0.81 | 0.81 |
| 10 | 1,557 | 1,504 | 0.95 | 0.99 | 0.71 | 0.71 |
| 12 | 1,504 | 1,454 | 1.00 | 1.03 | 0.71 | 0.72 |
| 16 | 1,506 | 1,446 | 1.00 | 1.03 | 0.72 | 0.71 |
| 20 | 1,497 | 1,462 | 1.00 | 1.03 | 0.71 | 0.72 |

- **The ports saved CPU (about 4%) and left the idle where it was:** 29% of ten cores at every
  thread count from 10 to 20, in both arms.
- **By role at 10 threads (base):** three leaders 0.31 CPU-s together, 0.10 each of a 0.139 s
  wall (74% busy); three first helpers 0.26 (62%); three second helpers 0.26 (62%); the scan
  thread 0.06 (40%).
- **The plan does not grow past ten threads:** the same eleven roles at 12, 16 and 20.

**W2, #324 on the i9** (`release-scaling/run-20261110T101103Z`, SHA-256 `2ec7d76d…dad321`,
checked): 451 against 411 MB/s at 1 thread, 1,119 against 1,036 at 4, 1,497 against 1,514 at
10. The gain stops where the team path starts.
- **Four plain workers at 94% busy give 1,119 MB/s for 0.70 CPU-s. Ten threads in teams give
  1,497 for 1.00 CPU-s at 71% busy:** 1.43× the CPU a byte, and three cores' worth idle.

**W1, the cold start** (`cold-start/20261110T102700Z`, as the Coordinator recorded it): the held
spinner alone kept the decode fast, 30 of 30. Pinning and the start-up spin did not. By the
rule set before it: nothing for the library; the bench's settle changes.
