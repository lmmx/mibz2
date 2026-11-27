# Synthesis, sixteenth pass: the i9's slow mode is the production default, and #1110 gates the Cells route

Read since the fifteenth pass (#1189, `b97cd27`): merged #1190, #1191, #1192, #1196, #1198, #1199; open
#1194, #1197, #1200, #1201, #1202, #1204 (their deltas and claim files at their heads). Raw rows re-read
from `origin/evidence 5e3f28cc` (`text-knee/run-20261127T172728Z.tar.gz`, sha256 `786084e2…ab7d4`,
checked). Nothing re-run. Recurring claim `synthesise`.

**Not re-scored.** `text-knee-i9`'s CONFIRMED stands (C2). Every number below marked *regrouped* is a
reading of that bundle's raw rows, not a verdict.

## 1. The text-knee "scanner slow mode" is the i9's all-threads mode (b), and it is on Wikidata too

#1204 reports text walls bimodal (~195 vs ~265 ms) with the scan thread 35–60% slower, and opens
`scanner-slow-mode-i9` asking whether the scanner shares its core with a worker (SMT) or runs at a low
clock. The raw rows answer part of that:

- **Wikidata x4 is bimodal in the same bundle** (*regrouped*, slow = wall > 1.2x the cell's minimum):
  | k | fast walls (ms) | slow median (ms) | slow/fast | slow runs |
  |---:|---|---:|---:|---:|
  | 4 | 356–363 | 476 | 1.33 | 5/9 |
  | 5 | 286–307 | 405 | 1.38 | 6/9 |
  | 6 | 244 | 359 | 1.47 | 7/9 |
  | 7 | 212–213 | 326 | 1.53 | 7/9 |
  | 8 | 191–196 | 293 | 1.52 | 15/18 |
  | 9 | 183–192 | 276 | 1.47 | 6/9 |
  Across docs_md and wikidata, 87/126 runs (69%) are slow.
- **All threads slow together.** The scan/worker CPU ratio is unchanged between modes on Wikidata
  (k = 8: 0.93 fast, 0.89–1.04 slow) and moves by at most ~12% on docs_md (k = 5: 1.21 → 1.36), while
  wall and per-thread CPU move by 1.33–1.58x. SMT co-residency of the scanner with one worker would
  change that ratio; this does not. The mode is the fifteenth pass's mode (b): unpinned (`cpus: all`,
  as this recipe ran), ~1.45x, every thread scaled.
- **It is not a power budget or warm-up.** The second run of the bundle (0.56 s in, 2 CPU-s of
  history) is already slow; slow fraction does not trend with round (6–11 of 14 per round), position
  in round, the previous corpus, or CPU-seconds in the preceding 10 s (3/8 at <5 CPU-s, 74/105 at >20).
  Pre-run state is uniform (powersave, `balance_performance`, ~870 MHz idle, 35 °C), and every run
  had a 100 ms 20-CPU prespin. Only the two runs after a ~21 s gap were both fast (n = 2).

**What it means:** the production default on the i9 is unpinned, so the default decoder spends most
runs in a mode ~1.5x slower than it can run. Fast-mode Wikidata x4 at the shipped 8-worker count is
191–196 ms (≈4.3 GB/s of output); the recorded median is ~290 ms. No open code lever is worth that
much. `i9-clock-modes` (#1201, frozen, in review) is the claim that can find the cause; raised to P1.

**For `scanner-slow-mode-i9`:** the SMT branch is ruled out by the ratio above; the clock branch is
`i9-clock-modes`' question. It should wait on that bundle rather than run its own probe.

**For #1201's reader (not a change to its frozen rule):** the stop test classifies a slow run as
`slow_work` ("contention or placement, not clock") when its cycles-per-byte rise ≥1.2x. A low
*uncore* (ring/L3) clock under `powersave`/EPP would also raise core cycles per byte, with the core
clock unchanged, on a decoder this memory-bound. A `PASS (mode is real work)` should therefore be read
as "not core frequency", not as "the decoder does more work"; `bench/pmu_probe.py`'s uncore events are
the follow-up that would separate the two.

**Within-mode, the text verdict's substance survives:** fast-mode docs_md is flat at 188–200 ms for
k = 4..9 (scanner-bound from k ≤ 4), repo_json rises 185 → 231 ms from k = 6 to 8 (lowest k = 8 run),
src_rs 167 → 219 ms from k = 5 to 8. So more workers do cost wall on text within a mode; the
docs_md knee of 8 is the mode-mix artefact #1204 names. Wikidata's fast mode gains ~3% from k = 8 to 9.

**For every queued or pending i9 recipe that runs unpinned at ≥4 threads:** a paired ratio between a
fast and a slow child is 1.5 or 0.67. Medians over ≥9 pairs survive a 70/30 mix only if the mix is the
same in both arms. Pinned bundles were unimodal at ≥4T (fifteenth pass); prefer pinned arms, and
report the #1201 within-mode estimator beside the registered statistic, descriptively.

## 2. `forward-anchor-lanes` (#1110) now gates the whole Cells route, not one arm

Four records written independently converge on one decision:

- **#1194 (EagerCells explore):** with `--features forward-anchor-lanes`, OpenCells beats Cells at
  t = 1, 2, 3 on base and JCC builds (−2.0 / −5.0 / −10.9% base; RSS 0.46–0.61x). Without it, Cells
  still wins at t = 1 (+9.2% disjoint).
- **`plan-schedule-and-lazy-threshold`:** OpenCells already beats EagerCells from k = 4 on the i9.
- **#1190 (page layer):** the only page-layer cost left is the Cells route's arena growth at t ≤ 6
  (1,321–3,640 `mprotect`s, 2.2–2.8x the faults of 8T).
- **Fifteenth pass:** the EagerCells 4T cell is one cause behind #1034's CI flag, the i9's lowest
  lbzip2 ratio and the 5/7T cliff.
- **#1197 (lanes):** the safe disjoint-slab walk (`lanes-disjoint-slabs-build`, −11% of `W_open`
  L3-cold, −96 KiB/worker) and the fused forward RLE1+CRC on slab slices (−15% of `W_open`) are built
  on #1110's lanes.

So the i9 kill test of #1110 (`forward-emission-run`, #1168) decides whether the Cells route can be
retired at every thread count, which removes the 4T anomaly, the Cells-route arena churn, the 6 MiB
`cells.buf`, and opens two worker-side builds. Raised to P1: one verdict unblocks three idea claims
and the plan change. If its recipe times k = 8 unpinned, §1's caveat applies.

The block-tail convergence of the fifteenth pass (RLE1 + CRC: #1179, #1183, `rle1-2load-filter-build`,
`forward-rle1-crc-vectored-build`) gains a fifth member in #1197's fused forward AVX2 RLE1 + CRC on
ordered slab slices. All five touch the same output bytes; their gains will not add.

## 3. glibc hides three costs that the shipped macOS wheels pay

Three explores each found glibc absorbing a cost, and each says so only for its own axis:

- **#1192 axis C:** `src/block.rs:195`/`src/stream.rs:161` drop the 1.80 MiB `syms` buffer on every
  `Item::End`. glibc's dynamic `M_MMAP_THRESHOLD` ratchets above it after stream 0, so it is free; under
  a fixed 128 KiB threshold, keeping `syms` cuts multi-stream scan CPU −32..−36% and full t = 1 decode
  −6.7..−8.6% (disjoint).
- **#1192 axis B:** `Vec` growth in `decode()` is free only because glibc grows via `mremap`.
- **#1190:** arena growth one page at a time on the Cells route.
- **#1200:** even warm, a single-use `StreamDecoder` pays 3,103–10,937 minor faults and 10–40 ms of
  system time per call from thread spawn and arena churn.

`CI.yml` ships two macOS wheels, whose allocator uses fixed size-class thresholds and returns large
blocks to the VM system on free. The `syms` result is the one with a measured fixed-threshold figure
and a ~3-line fix; opened `allocator-fixed-threshold-buffers` (idea, P3) to measure it on a macOS CI
runner and under `MALLOC_MMAP_THRESHOLD_`, together with #1192's `Seq::read_to_end` direct decode.

Also: #1192 axis C shows `StreamDecoder` already keeps every worker and `Scratch` across 50–200
concatenated streams at zero restart cost. #1200's `DecoderPool` (needs Louis for its public shape)
can reuse that machinery; its sysfs-topology cache is internal and needs no API decision, so it can be
split off if the pool waits.

## Claims touched

- `i9-clock-modes`: P2 → P1 (§1: the mode is the shipped unpinned default's majority state, ~1.5x).
- `forward-emission-run`: P2 → P1 (§2: gates three idea claims and the plan change).
- New `allocator-fixed-threshold-buffers` (idea, P3).
- `scanner-slow-mode-i9` (on #1204, not yet merged): recommend it wait on `i9-clock-modes`.
