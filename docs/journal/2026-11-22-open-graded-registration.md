# bench-open-graded: registration

*22 November 2026. Written and frozen before any data, on `bench/open-graded`. The recipe is
`just bench-open-graded` (`.just/bench.just`); the verdict is `bench/open_graded.py`'s, tested by
`bench/test_open_graded.py`. The recipe, the rule and the verdict script are bench-open-relay8's
(#707, `docs/journal/2026-11-21-open-relay8-registration.md` on `bench/open-relay8`), themselves
bench-open-diet's (#693), bench-open-r4's (#680), bench-open-lanes's (#676) and
bench-open-vec16's (#669), with the same median peak RSS report. Only the builds differ.*

**The decision it settles:** whether #714 (`perf/open-wormhole-15-17`, "perf(open): GradedHandle
and Prefix15 in construction (Wormhole cycles 15, 17)") merges, that is, whether its two
construction changes reach the decoder's default on the i9. It is an ordinary merge under
PERFORMANCE-POLICY.md, measured on the one machine where production runs `OpenCells` on the
i9's cache: CI's bench never runs Open there (`docs/journal/2026-11-20-open-stage-attribution.md`).

## The hypothesis

#714 changes `src/open.rs`, `src/open/endpoint.rs`, `src/open/spirals.rs` and
`src/open/checks.rs` (the last two test code), in safe Rust, with no allocator, admission,
routing or emit-loop change. Two commits:
- **GradedHandle** (`106343520`, Wormhole cycle 15): each live node handle carries the word's
  exact length up to 1,022 (grade 1,023 falls back to the full rule length), and known tail
  removals update the sentinel directly. It targets S2b, the elimination.
- **Prefix15** (`26a5f6aed`, Wormhole cycle 17): Inline16 entries are built during
  construction (up to 15 excursion bytes plus the retained byte), so the separate post-wiring
  dictionary pass S3b goes. Longer words keep memoized spelling; the band scan keeps #692's
  exact-size reservations.

**The native evidence (#714's body).** Measured in a shared container (AMD EPYC 9V45, four-CPU
quota), against master's kernel in the same binary, 12 fresh-process rounds, 95% intervals:
- **The kernel at one thread:** both commits together 0.963–0.980× master's kernel on the six
  measured blocks (block-0 0.963 [0.956, 0.970] to block-210 0.980 [0.975, 0.985]; A/A
  0.995–1.000).
- **Four concurrent workers:** a worker's six-block pass **0.973 [0.960, 0.988]** (A/A 1.004
  [1.001, 1.007]).
- **By stage:** S2b 0.772–0.793× on every block; S3b removed (7.7–21.9 µs a block); S3a
  4.6–7.3% longer, carrying the direct dictionary construction. Emit (S5) is unchanged.
- **The whole Wikidata stream** (`OpenCells` forced at one and four workers): unresolved. At
  four workers 0.937 [0.862, 1.019]; at one 1.040 [0.972, 1.113].
- **Memory:** the retained scratch after each block is byte for byte master's (579,990 to
  1,200,839 B); the short DFS uses a fixed 16-entry stack array and adds no heap.

**Claim:** a construction-stage gain of about 2–4% of the Open kernel's time. Construction (S1
to S4) works on the block's own arrays; #714 removes work there (part of S2b, all of S3b) and
leaves the kernel's memory exactly as it was. The memory-diet precedent says such a kernel
gain should hold under the i9's contention: #692's native four-worker pass, 0.960 [0.934,
0.987], held on the i9 at 0.952, 0.917 and 0.931 at 6, 8 and 10 threads
(`open-diet/run-20261121T121714Z`, ADOPTS). #714's four-worker pass, 0.973 [0.960, 0.988],
already holds under four workers natively. If it holds on the i9, production's decoder gets
faster at 6, 8 and 10 threads by the kernel's share of the wall. The cause is a hypothesis,
not shown.

**The risk, named:** the gain may be too small to clear the 10-thread interval. It is a 2–4%
kernel gain, diluted in the whole decoder by everything that is not the Open kernel, and it
brings no memory change, which is what made the diet's gain grow under contention. bench-open-r4
(#679), with a native Open-stage gain of about 5%, measured 0.985 [0.970, 1.001] at 10 threads
and was NOT ADOPTED by 0.001. A real gain of this size can land INCONCLUSIVE at 10 threads; the
rule is not relaxed for that.

## What is compared

- **Head:** #714's head, `perf/open-wormhole-15-17` at
  `26a5f6aed23a430c7447616782c0a2b6db946e2f`. It is its merge-base plus two commits, which
  change `src/open.rs`, `src/open/checks.rs`, `src/open/endpoint.rs` and `src/open/spirals.rs`
  (230 lines added, 37 removed) and nothing else.
- **Base:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5`, #714's merge-base with master (#713's
  merge). Its src is v0.5.11's: the Open kernel with the memory diet (#692) and dense-block
  admission (#703). It is an ancestor of HEAD; bench-open-r4's ancestry check
  (`git merge-base --is-ancestor BASE HEAD`) is kept unchanged.
- **Base and master agree.** When the recipe was written, master's head *was* the base,
  `8ff1187af`, so the base's build inputs (`src`, `examples`, `Cargo.toml`, `Cargo.lock`,
  `build.rs`, `rust-toolchain.toml`) equal current master's. **So this run measures #714 against
  master's head.** Should master move before the run, the run still measures #714 against its
  own base; the Coordinator weighs any later master change to `src` at the merge.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`, or
  the run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/checks.rs`,
    `src/open/endpoint.rs` and `src/open/spirals.rs`. The bundle keeps the list as
    `src-changed.txt`.
  - Both builds use rustc 1.99.1 (`rust-toolchain.toml`, the same at both), and the head's
    harness drives the run. `examples/ab.rs` is the same at both.
- **What it predicts:** if the kernel gain holds under contention, a whole-decoder wall gain at
  10 threads (7 workers), 8 (8 workers) and 6 (6 workers), smaller than the kernel's 2–4%. If
  contention erodes it, a level step at each. A loss would be unexplained by the native
  evidence. For scale, on the same grid: bench-open-diet measured 0.952, 0.917 and 0.931 at 6,
  8 and 10 threads, ADOPTS; bench-open-r4 measured 0.956, 0.982 and 0.985 [0.970, 1.001], NOT
  ADOPTED by 0.001 at 10.
- **Memory, reported and expected flat:** #714 adds 0 B of heap (the retained scratch is byte
  for byte the base's) and a 16-entry stack array. The verdict prints each count's median peak
  RSS at the base and the head, in MiB, and their difference. Only the 1.10 gate decides
  anything.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431
  blocks, 209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do, so that each run is long against process start.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8 and 10 threads. Production runs
  `OpenCells` on the i9 at each of these (unchanged since bench-open-diet: between v0.5.10 and
  the base, `src` changed in the Open kernel alone, by #703):
  - 6 threads: 6 lazy workers;
  - 8 threads: 8 lazy workers;
  - 10 threads (production's default on the i9): 7 workers, `trimmed`, unpinned.

  Production also runs `OpenCells` at exactly 4 threads unpinned (#678). That count is not in
  this run, which keeps bench-open-diet's grid so the results compare directly.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **The diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, the workers' lazy
  route with plan and window fixed, at 6 and 8 threads, 16 rounds. It is reported as steps
  only.
- **Cost:** about 15–20 minutes of workstation time with the two builds, below
  PERFORMANCE-POLICY §4a's 30-minute line.

## The rule

bench-open-relay8's (R4's), unchanged. A count's **step** is the 16 rounds' paired wall ratios,
head over base: their geometric mean and its 95% interval (Student's t on the logs, 15 degrees
of freedom). A step is FASTER if the interval lies below 1, SLOWER if it lies above 1, and
INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER (its interval lies below 1);
  - neither 6 nor 8 threads is SLOWER;
  - at each of 6, 8 and 10 threads, the median peak RSS over the rounds, head over base, is
    at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: they are reported with their intervals, and only a loss
  there counts against adoption.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the
  base and the head in MiB with their difference.

**Limits:**
- **The corpus:** Wikidata alone, the decoder's primary workload and where Open admits most of
  the rows. `text` is not in this run.
- **Admission:** unchanged by #714. At these builds Open admits all 233 full run-route blocks
  (#703's dense admission is in both), so the same blocks take Open in both builds, and #714's
  construction runs on every one of them.
- **What a result separates:** an ADOPTS shows the whole-decoder gain, not its cause or which
  of the two commits gives it. The native evidence does not resolve Prefix15's end-to-end share
  either.
- **The head's branch:** the recipe fetches HEAD from `perf/open-wormhole-15-17` if the
  workstation lacks it. A force-push that drops 26a5f6ae from every ref on origin would stop the
  run at the fetch, not change what it measures.
