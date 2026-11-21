# bench-open-relay8: registration

*21 November 2026. Written and frozen before any data, on `bench/open-relay8`. The recipe is
`just bench-open-relay8` (`.just/bench.just`); the verdict is `bench/open_relay8.py`'s, tested by
`bench/test_open_relay8.py`. The recipe, the rule and the verdict script are bench-open-diet's
(#693, `docs/journal/2026-11-21-open-diet-registration.md`), themselves bench-open-r4's (#680),
bench-open-lanes's (#676) and bench-open-vec16's (#669), with the same median peak RSS report.
Only the builds differ.*

**The decision it settles:** whether #705 (`perf/open-relay8`, "perf(open): eight emitters per
worker, cuts on edges (Wormhole Relay8)") merges, that is, whether the relay reaches the
decoder's default on the i9. It is an ordinary merge under PERFORMANCE-POLICY.md, measured on
the one machine where its case can show: CI's bench never runs Open on the i9's cache
(`docs/journal/2026-11-20-open-stage-attribution.md`), and the container it was built in has a
2 MiB L2 a core.

## The hypothesis

#705 changes `src/open.rs` and adds `src/open/relay.rs` (with its tests), in safe Rust, with no
allocator or routing change and no heap. `emit` takes the relay when the block's codes are
narrow (no `extra`) and it has at least 1,024 retained rows; any other walk, and any walk the
relay hands back, is the serial walk unchanged. The relay:
- **cuts** the walk at the retained rows `j` with `((j ^ mark) & cm) == 0` (k = 8, 128 to 256
  cuts), using no bit of `edges`;
- **relays** it: lane 0 emits from the mark, seven scouts measure the unclaimed segments, lane 0
  jumps the scouted ones, which become tasks, and eight emitters run the tasks;
- **restores** each piece's first 16 bytes exactly at the end.

So a worker walks about 1.87m loads where the serial walk does m (about 0.87m extra), but eight
chains at a time instead of one.

**The native evidence (#705's body and `docs/journal/2026-11-21-open-relay8.md`).** Measured in a
shared 4-vCPU container (a Xeon with 2 MiB of L2 per core), against master's kernel in the same
binary, 12 rounds of fresh processes, 95% intervals:
- **Over the corpus at four workers** (431 blocks, 6 rounds): a pass is **1.040 [1.018,
  1.063]**, 4% slower (A/A 1.003 [0.984, 1.022]).
- **End to end, `bzcat` on the corpus** (10 rounds): neutral. At 8 threads wall 1.009 [0.966,
  1.053], peak RSS 0.984 [0.948, 1.022]; at 10 threads wall 1.020 [0.977, 1.066], peak RSS 1.017.
  That box has 4 cores, so these counts oversubscribe it.
- **By block** (one thread, the kernel): 1.084 and 1.064 on blocks 0 and 2 (m 90–112k), about
  1.00 on blocks 85 and 207 (m 136–156k), 0.958 and 0.953 on blocks 143 and 210 (m about 200k).
  The walk alone: 1.13 and 1.10 on blocks 0 and 2, 0.81–0.82 on blocks 143 and 210.
- **Why:** on that box every block's `edges` (351–798 KiB) stays in L2. The serial walk is
  latency-bound at 6.0 to 7.8 ns a step; the relay is bound by instruction throughput at a
  near-constant 6.5–6.8 ns per row of m. It wins only where the chain is long and slow.

**Claim:** the i9-10900K has 256 KiB of L2 a core, so `edges` (351–798 KiB on the six measured
blocks) misses to the shared 20 MiB L3, and the serial chain pays about 42 cycles a step. The relay overlaps
eight such chains for about 0.87m extra loads, which also go to L3. If the latency the relay
hides outweighs the extra loads' cost (instructions, and L3 bandwidth shared by 6 to 8 workers),
the decoder gets faster where production runs `OpenCells`; if not, it gets slower, as it did
natively. The run decides which effect wins. The cause is a hypothesis, not shown.

**The risk, named:** the corpus's median full block has m = 101,795 (quartiles 95,236 and
115,544). That is the regime where the relay lost natively (blocks 0 and 2, 1.06–1.08 in the
kernel, 1.10–1.13 in the walk alone); only 25 of the 228 blocks Open admits at these builds
have 150,000 rows or more, where it won. Natively the corpus pass lost 4%. And #675's lesson,
a one-thread win that reversed at 8 i9 workers through shared cache and memory, cuts both
ways: #705 adds no heap, so #675's mechanism does not apply, but its extra loads all land on
the shared L3.

## What is compared

- **Head:** #705's head, `perf/open-relay8` at `a3dc038b433ca1ba442f8ce5ebd7e02acae9e145`. It is
  its merge-base plus one commit, which changes `src/open.rs`, adds `src/open/relay.rs` and
  `src/open/relay/tests.rs`, and adds the native harness under
  `docs/journal/2026-11-21-open-relay8/`, which is not compiled.
- **Base:** `c68b85c710e693d7eb3011b306dc3cc844731e59`, #705's merge-base with master (#702's
  merge). Its src is v0.5.10's: the Open kernel with the memory diet (#692).
- **Base and master differ.** Master's head when the recipe was written,
  `90e671966` (#703, "perf(open): admit dense blocks (width certificate) with producer buffers
  in edges"), merged after #705 branched, and it is not an ancestor of HEAD. bench-open-r4's
  ancestry check (`git merge-base --is-ancestor BASE HEAD`) is kept unchanged, so the base is
  the merge-base, which HEAD's history contains. Between the base and master, `examples`,
  `Cargo.toml`, `Cargo.lock` and `build.rs` are identical; `src` differs, in the Open kernel
  alone, by #703: `src/open.rs`, `src/open/b5.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`,
  and the new `src/open/dense.rs`, `src/open/mux.rs` and `src/open/spirals.rs` (841 lines added,
  67 removed). (#704, between them too, is docs only.) **So this run measures Relay8 against
  its own base, not against master's head.** Neither build has #703's dense admission, so the
  same blocks take Open in both, and none of the dense blocks #703 admits is in either.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`, or
  the run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/relay.rs` and
    `src/open/relay/tests.rs`. The bundle keeps the list as `src-changed.txt`.
  - Both builds use rustc 1.99.1 (HEAD's `rust-toolchain.toml`), and the head's harness drives
    the run. `examples/ab.rs` is the same at both.
- **What it predicts:** if the L3 latency wins, a whole-decoder wall gain at 10 threads (7
  workers), 8 (8 workers) and 6 (6 workers), every worker running the relay on its narrow
  blocks. If the extra loads win, a loss at each, perhaps growing with workers as they share the
  L3. For scale, on the same grid: bench-open-diet (`open-diet/run-20261121T121714Z`) measured
  0.952, 0.917 and 0.931 at 6, 8 and 10 threads, ADOPTS; bench-open-r4 measured 0.956, 0.982
  and 0.985 [0.970, 1.001], NOT ADOPTED by 0.001 at 10.
- **Memory, reported and expected flat:** #705 adds 0 B of heap and about 10 KiB of stack a
  worker. The verdict prints each count's median peak RSS at the base and the head, in MiB, and
  their difference. Only the 1.10 gate decides anything.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431
  blocks, 209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do, so that each run is long against process start.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8 and 10 threads. Production runs
  `OpenCells` on the i9 at each of these (unchanged since bench-open-diet; 0.5.10's
  recalibration moved no trim):
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

bench-open-diet's (R4's), unchanged. A count's **step** is the 16 rounds' paired wall ratios,
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
- **Admission:** at these builds Open's admission is the density line, unchanged by #705, so the
  same blocks take Open in both builds. The relay applies to every one of them with narrow codes
  and at least 1,024 retained rows.
- **Master's #703:** the run says nothing about Relay8 on the dense blocks #703 admits, or on
  #703's kernel. #705 merges onto master cleanly (`git merge-tree`), so an ADOPTS would merge
  Relay8 onto a kernel this run did not measure it on. The Coordinator weighs that at the merge;
  this run does not decide it.
- **What a result separates:** an ADOPTS shows the whole-decoder gain, not its cause. A gain
  that grows with the blocks' m, or a loss that grows with workers, would be consistent with
  the L3 claim or with contention for it; neither shows it.
- **The head's branch:** the recipe fetches HEAD from `perf/open-relay8` if the workstation
  lacks it. A force-push that drops a3dc038b from every ref on origin would stop the run at the
  fetch, not change what it measures.
