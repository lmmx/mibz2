# The build's windows cut into runs: registration and instruction count

Registered before any timing of this branch. The decision rule may be amended only in a commit
made before the first bench comment on this PR, and each amendment says what changed and why.

## What was tried before, and why this is not a retry

- **#169** (`2026-11-05-build-run-vectors.md`): the 16 entries of an all-equal window written
  as four SSE2 vectors. −2.4% of `decode_block`'s instructions, the build stage −3.1% at
  1 thread beyond noise, no `wikidata` figure beyond noise. Closed as not shown.
- **#23**: the build in four interleaved quarters, +11–12%. Rejected.
- **The run-fed line** (#266, #269, #277): the table filled from MTF's run records. Its census
  found that the build already stores 82% of Wikidata's rows through the 16-row path, and its
  stage run found the build "not store-bound" (R_floor 0.321, R_warm 1.010): what binds is its
  latency or its instructions, not its stores.
- **The census** (`2026-11-05-binding-census.md`, item 7) disfavours a write-combining build:
  the build's co-run excess tracks its low IPC, not its number of store streams.
- **The ISA probe** (#201): an x86-64-v3 build buys at most −4.5% of the build.

So no store-side lever (write-combining, prefetchw, two-pass bucketing) is tried here. This
change goes after a latency the earlier work did not name.

## The mechanism

`build_one_pass` takes the rows 16 at a time. A window of 16 equal bytes is one cursor step.
Any other window takes 16 one-row steps, and each step loads its byte's cursor `ftab[c]`,
writes the entry and stores `ftab[c] + 1`. `ftab` is an indexed array, so it lives in memory.
When a row's byte equals its predecessor's, that load must wait for the store just made: a
store-to-load forward, about 4–7 cycles on the runners' cores, on a chain as long as the run.

**How much of it there is (counted, `window_stats.rs`, all 473 blocks of
`wikidata-independent.bz2`, 235.8M rows):**
- 83.4% of rows are in all-equal windows (the run-fed census's 82%).
- The other 16.6% are in 2.45M mixed windows, 6.8 runs each on average, bimodal: 2–4 runs
  (windows at a run's edge, 47% of mixed windows) or 13–16 (literal stretches).
- **22.6M rows (9.6% of all rows, 58% of mixed-window rows) equal their predecessor inside a
  mixed window.** Each is one link of a store-forward chain today. At about 6 cycles a link,
  serial, that is up to about 0.58 cycles a row, against a whole build of about 1.4–2.2
  cycles a row on CI (the run-fed stage tables; the ISA probe's 2.17 c/B on the 7763).
  Out-of-order overlap with other rows hides part of it; how much is what this PR measures.

**The change.** A window's run heads come from one SSE2 compare of the window with itself
shifted by a byte (`run_heads`: bit j set where row j starts a run). Then:
- one run (the old 16-row path): one cursor step of 16 and one 16-wide write, now SSE2
  (`run16`, #169's kernel);
- at most `RUNS` = 6 runs: each run is one cursor step of its length L and one 16-wide
  write at its slot. The entries above the run's end (L..16) are garbage in slots of the same
  bucket above its new cursor, each overwritten by the step that later reaches it. A run with
  fewer than 16 slots left in its bucket (0.7% of mixed-window runs) takes one-row steps;
- more than 6 runs: the one-row steps as before. Those windows are mostly unequal neighbours,
  so their chains are short.

The safety argument changes from "every slot written exactly once" to "every slot last written
with its entry": the end check (every cursor at its bucket's end) still proves it, since every
garbage slot lies above some step's end inside its bucket, and the cursor passed it later.
Writes stay inside `[ftab[c], lim[c]) ⊆ [0, n)`. Refused counts still leave the table empty.
Other targets than x86_64 take a scalar `run_heads` and `run16` (the same table, checked by
the tests with the x86 paths compiled out, locally).

## Whole decoder: instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), both arms
built with the driver of `2026-11-05-deferred-flag-walk/` (rustc 1.99.1, fat LTO,
`codegen-units = 1`), on `wikidata-independent.bz2` (sha256 prefix `7f5c9a0044e5a305`).
Master is b68171223. Profiles and outputs are in `cg/` (`base-*` master, `head-*` this branch).

| block | master | this branch | change | build, master → branch | walk | output |
|---|--:|--:|--:|--:|--:|---|
| 0 (54 B) | 7,037 | 7,117 | +80 | 3,200 → 3,280 | — | identical |
| 1 | 19,258,148 | 18,652,917 | −605,231 (−3.14%) | 5,250,956 → 4,645,725 (−11.5%) | 9,493,915 both | identical |
| 41 | 18,608,504 | 17,945,904 | −662,600 (−3.56%) | 5,103,413 → 4,440,813 (−13.0%) | 9,303,521 both | identical |
| 97 (56 B) | 7,481 | 7,561 | +80 | 3,368 → 3,448 | — | identical |
| 233 | 19,053,756 | 18,550,598 | −503,158 (−2.64%) | 5,038,522 → 4,535,364 (−10.0%) | 9,464,794 both | identical |
| 400 | 18,620,609 | 17,988,154 | −632,455 (−3.40%) | 5,026,554 → 4,394,099 (−12.6%) | 9,362,918 both | identical |
| 472 (70 B) | 8,252 | 8,357 | +105 | 3,585 → 3,690 | — | identical |

- **Build, walk, rest:** the whole difference is in the build. The walk is identical to the
  instruction; the rest (MTF, the call) is unchanged. The short blocks' +80 to +105 is the
  window loop's set-up on a block of three or four windows.
- **What the instructions are, by `RUNS` on block 1** (`cg/runs*-b1.out`): 1 (= #169's
  all-equal path only) 4,632,711; 3: 4,615,193; 4: 4,581,583; **6: 4,645,725**; 8: 4,757,576;
  16: 5,713,723. So **almost all of the instruction saving is #169's**, which CI already
  showed is invisible end to end. At `RUNS` = 6 the run-by-run path costs +13k instructions
  over `RUNS` = 1 on block 1 (+0.07% of the decode). **This PR's claim is not its
  instruction count.** It is the chain it removes.
- **Branches** (callgrind's simulated predictor, block 1, `cg/br-*-b1.out`): the build's
  conditional branches 564k → 376k, its simulated mispredictions 10.5k → 15.5k (+5.0k). At
  about 15 cycles each that is about +75k cycles, against about 86k chain links removed on
  the block (9.6% of 900k rows) at 4–7 cycles each, about 350–600k cycles if they were
  serial. The simulator is not the runners' predictor.

## The prediction and its falsifier

- **The build stage** (`phases`, 1 thread): **−7% to −20%**. Its instructions fall 10–13%,
  but #169's same instruction saving alone moved the stage −3.1%, so the chain must supply the
  rest.
- **`wikidata`:** 1 thread −1% to −3% wall; 4 threads −1% to −4% (the build's co-run
  inflation, ×1.4–2.3, should make a latency cut count more there, per the census's item 7).
- **No other stage moves.**
- **Falsifier:** if the build stage gains no more than #169's −3.1% (its point estimate at or
  above −4%), the store-forward chain is not what costs the build, and the mechanism is
  refuted, whatever the end-to-end figures say. A build stage that is slower refutes it too.

## The decision rule

- **Correctness (required):** CI green, Miri included (`miri_build_one_pass` reaches the
  run-by-run path, the near-end fallback and refused counts); the same bytes on every corpus
  in every bench run; the build's tests pass, with their inputs now including runs of 3–7.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point estimate
  is not slower in the other.
- **No harm (required):** as in `2026-11-05-deferred-flag-walk.md`. A wall row flagged slower
  is settled by one more run on the same head, with at most three runs.
- **No retries to pass:** if no run shows a gain beyond noise, the PR is closed and this entry
  records the negative result.
