# Open's construction, stage by stage (measurement)

**Question.** Where does the shipped open kernel (`Kernel::construct::<INDUCE = true, QUEUE =
true>` then `emit`, `src/open.rs`, `src/open/endpoint.rs`) spend its time and memory traffic
inside construction? Is any one stage large enough (above about 15% of the block's total) to
justify a targeted replacement experiment? The earlier phase file
(`2026-11-18-open-return/evidence/phases-wikidata.csv`) split only build from write: build was
a median 60% (24-79%) of `qi16`'s total.

**Answer.** On these six blocks, construction is 52-64% of the shipped kernel's time, and no
single construction stage passes 15% except the **queued wiring**, once it is counted whole: the
queue fill hidden in the run pass plus the scatter into `edges` is **23-28%** of the total
(0.37-0.87 ms a block). The elimination (`eliminate`, rules included) is 5.5-12.1%, 8.5-18.5%
with its node setup and bands. The spelling is 4.3-9.4%. `emit`, the walk, is the largest stage
at 35.6-47.7%. The direct sequential wiring (`i16`, the arm the outside proposal DOE suggests)
costs more than the queued wiring on every block: its wiring stage is 0.64-1.45 ms, 33-41% of
its total, 1.6-1.8x the queued wiring's.

## Method

- **The clone.** `src/open/endpoint/phases.rs`, behind a new research feature `open-phases`
  (which implies `moonshot`), is a copy of `construct::<true, QUEUE>` and `emit` cut into eight
  `#[inline(never)]` stage functions with an `Instant` between each. The default build is
  unchanged: without the feature the module isn't compiled. `Runs::of_column` is public under
  the feature, as in `research/nspc-prototype` (#652), so the harness builds the 127-capped
  records from an exact column.
- **The one rearrangement.** Production spells each word (`Endpoint::append`, into `spelling`
  and `offsets`) inside the band loop at the end of `induced`. The clone defers the appends to a
  loop of their own, over the same words in the same order. The codes are the same
  (`255 + words`), and so is every append's view of `spelling`, so the output is identical.
- **Stages.** (a) `each_run`, the strips and, for `qi16`, the queues; (b) `starts` and the
  image order; (c1) the endpoint nodes, the two orders and their lists; (c2) `eliminate`,
  rule creation included; (c3) the bottoms and the bands; (d) the spelling; (e) the wiring that
  fills `edges`; (f) `emit`. The brief's (c) is c1 + c2 + c3.
- **Harness.** `examples/open_phases.rs DIR ROUNDS OUT`. Four arms: `qi16` and `i16`, each run
  as production runs it (`Kernel::run`, timed whole) and as the clone. Every output of every
  round is checked byte for byte against a dense-LF walk of the column, outside the timed
  span. 41 rounds after 2 warm-up rounds, the arms' order rotated each round, buffers kept
  between rounds. `taskset -c 3` on the shared 4-core cloud VM (Xeon @ 2.1 GHz; 48 KiB L1d and
  2 MiB L2 a core). Medians, with p10-p90 in the CSV. Local timings are indicative only.
- **Counters.** Elements and bytes each stage writes come from the kernel's buffers after the
  block. Live bytes are every buffer's capacity after the block (the kernel keeps them all).
  Instructions and cache misses are from callgrind (`--cache-sim=yes`, host-detected caches,
  so a 260 MiB simulated LL and LL misses are close to cold misses only), with
  `--toggle-collect` on one stage function at a time. The harness runs the clone twice per
  process (a warm-up and one run), and the CSV halves the counts.
- **Blocks.** The six admitted Wikidata columns (0, 2, 85, 143, 207, 210), as in #652.

## Checks

- **Byte-exact:** all four arms, all six blocks, all 43 rounds.
- **The clone against production:** the clone's stage sum is 97.5-98.6% of production's untimed
  total for `qi16`, and 101.6-103.9% for `i16`. The timer around the whole clone call agrees
  with the stage sum to 1 µs. So the split accounts for production's time within about 2.5%
  (`qi16`) and 4% (`i16`).

## Results

`qi16` (shipped), ms and share of the clone's total (%), medians of 41 rounds:

| block | a runs+queues | b image | c1 nodes | c2 elim | c3 bands | d spell | e wire | f emit | total (clone) | total (prod) |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 0.469 (29.9) | 0.006 (0.4) | 0.061 (3.9) | 0.187 (12.1) | 0.040 (2.5) | 0.146 (9.4) | 0.075 (4.8) | 0.558 (35.6) | 1.566 | 1.593 |
| 2 | 0.522 (27.9) | 0.006 (0.3) | 0.069 (3.6) | 0.228 (11.8) | 0.054 (2.8) | 0.173 (9.0) | 0.096 (5.0) | 0.734 (38.8) | 1.914 | 1.956 |
| 85 | 0.673 (35.8) | 0.003 (0.2) | 0.026 (1.4) | 0.106 (5.5) | 0.029 (1.6) | 0.080 (4.3) | 0.066 (3.4) | 0.902 (47.4) | 1.874 | 1.940 |
| 143 | 1.039 (28.4) | 0.010 (0.3) | 0.099 (2.7) | 0.358 (9.8) | 0.092 (2.5) | 0.278 (7.5) | 0.165 (4.5) | 1.601 (43.8) | 3.646 | 3.752 |
| 207 | 0.721 (31.5) | 0.007 (0.3) | 0.064 (2.7) | 0.227 (9.7) | 0.059 (2.5) | 0.166 (7.2) | 0.105 (4.6) | 0.942 (40.6) | 2.267 | 2.336 |
| 210 | 1.034 (33.0) | 0.006 (0.2) | 0.059 (1.8) | 0.199 (6.3) | 0.057 (1.8) | 0.145 (4.6) | 0.123 (4.0) | 1.498 (47.7) | 3.145 | 3.194 |

`i16` (direct sequential wiring), the same stages:

| block | a runs | b image | c1 nodes | c2 elim | c3 bands | d spell | e wire | f emit | total (clone) | total (prod) |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 0.169 (9.2) | 0.008 (0.4) | 0.061 (3.3) | 0.190 (10.3) | 0.039 (2.2) | 0.147 (8.1) | 0.644 (35.2) | 0.551 (30.2) | 1.832 | 1.792 |
| 2 | 0.180 (8.1) | 0.008 (0.3) | 0.070 (3.1) | 0.234 (10.5) | 0.054 (2.5) | 0.175 (7.9) | 0.712 (33.0) | 0.746 (33.6) | 2.193 | 2.171 |
| 85 | 0.214 (9.1) | 0.004 (0.1) | 0.028 (1.2) | 0.109 (4.6) | 0.030 (1.3) | 0.084 (3.5) | 0.942 (40.8) | 0.883 (38.7) | 2.317 | 2.245 |
| 143 | 0.334 (7.9) | 0.010 (0.2) | 0.097 (2.3) | 0.356 (8.5) | 0.092 (2.2) | 0.265 (6.4) | 1.435 (34.5) | 1.583 (37.8) | 4.193 | 4.109 |
| 207 | 0.261 (9.4) | 0.007 (0.2) | 0.063 (2.3) | 0.225 (8.2) | 0.059 (2.2) | 0.167 (6.1) | 0.995 (36.4) | 0.934 (34.2) | 2.732 | 2.588 |
| 210 | 0.355 (9.5) | 0.006 (0.2) | 0.055 (1.5) | 0.199 (5.3) | 0.057 (1.5) | 0.141 (3.8) | 1.451 (38.7) | 1.481 (39.3) | 3.756 | 3.626 |

**The wiring, whole.** In `qi16` the wiring is in two places: the run pass appends every
retained rank to its byte's queue (the queue fill), and (e) scatters the queues through the
bands into `edges`. The queue fill is the difference between the two arms' run passes, which
are otherwise the same code. Per block (ms, share of its arm's total; instructions per block
from callgrind):

| block | retained | queue fill | scatter (e) | queued wiring | `i16` wiring (e) | queue fill Ir | scatter Ir | `i16` wiring Ir |
|---|---|---|---|---|---|---|---|---|
| 0 | 89,874 | 0.300 | 0.075 | 0.374 (23.9%) | 0.644 (35.2%) | 2.31 M | 0.53 M | 1.48 M |
| 2 | 112,070 | 0.342 | 0.096 | 0.438 (22.9%) | 0.712 (33.0%) | 2.52 M | 0.67 M | 1.61 M |
| 85 | 156,404 | 0.459 | 0.066 | 0.525 (28.0%) | 0.942 (40.8%) | 3.74 M | 0.68 M | 2.28 M |
| 143 | 196,841 | 0.706 | 0.165 | 0.871 (23.9%) | 1.435 (34.5%) | 4.92 M | 1.12 M | 3.07 M |
| 207 | 135,921 | 0.461 | 0.105 | 0.566 (25.0%) | 0.995 (36.4%) | 3.83 M | 0.76 M | 2.43 M |
| 210 | 204,281 | 0.678 | 0.123 | 0.801 (25.5%) | 1.451 (38.7%) | 5.72 M | 0.96 M | 3.56 M |

The queued wiring writes the retained ranks twice: 4 B each into the queues
(0.36-0.82 MB), then 4 B each into `edges` (the same again). The queues add 0.55-1.23 MB of
live buffers over `i16` (live bytes 1.56-3.23 MB against 1.02-2.01 MB).

**Instructions and misses per stage** (`qi16`, per block: Ir in millions / D1 misses, reads
plus writes, in thousands / LL misses in thousands):

| block | a | b | c1 | c2 | c3 | d | e | f |
|---|---|---|---|---|---|---|---|---|
| 0 | 4.77 / 17.1 / 4.3 | 0.06 / 1.1 / 0.0 | 0.71 / 17.2 / 2.2 | 1.14 / 12.8 / 0.9 | 0.29 / 6.4 / 0.6 | 0.56 / 6.0 / 0.4 | 0.53 / 15.6 / 2.8 | 3.12 / 78.5 / 6.9 |
| 2 | 5.14 / 19.5 / 5.0 | 0.06 / 1.3 / 0.1 | 0.81 / 19.7 / 2.5 | 1.41 / 18.3 / 2.1 | 0.41 / 9.7 / 0.2 | 0.69 / 7.8 / 0.4 | 0.67 / 19.6 / 3.5 | 4.02 / 96.2 / 6.9 |
| 85 | 7.47 / 25.7 / 6.4 | 0.03 / 0.5 / 0.0 | 0.41 / 8.8 / 1.6 | 0.67 / 4.4 / 0.0 | 0.23 / 4.7 / 0.3 | 0.36 / 3.0 / 0.2 | 0.68 / 25.8 / 4.9 | 5.16 / 104.8 / 7.0 |
| 143 | 10.06 / 36.0 / 9.3 | 0.08 / 1.5 / 0.0 | 1.18 / 29.4 / 4.2 | 2.10 / 27.5 / 2.1 | 0.67 / 16.6 / 1.0 | 1.08 / 12.0 / 0.0 | 1.12 / 34.4 / 6.2 | 7.22 / 190.1 / 7.0 |
| 207 | 7.93 / 24.0 / 6.0 | 0.06 / 1.1 / 0.1 | 0.78 / 18.8 / 2.5 | 1.34 / 15.1 / 1.0 | 0.45 / 10.6 / 1.2 | 0.70 / 8.2 / 0.4 | 0.76 / 23.5 / 4.2 | 4.80 / 128.5 / 7.0 |
| 210 | 11.73 / 34.9 / 8.4 | 0.05 / 0.9 / 0.0 | 0.64 / 14.9 / 2.3 | 1.15 / 11.9 / 0.8 | 0.43 / 9.8 / 0.9 | 0.63 / 6.5 / 0.3 | 0.96 / 34.3 / 6.4 | 7.00 / 183.2 / 7.0 |

Element counts (per block, from `-counters.csv`): 1.6-5.0 k strips; 4.1-12.8 k open nodes;
7.1-21.3 k elimination steps; 4.1-12.6 k rules; 2.4-6.8 k bands; 1.2-3.6 k words, spelled into
32-58 KB.

## Reading

- **The run pass is mostly the queue fill.** Without the queues (`i16`) the run pass is
  0.17-0.36 ms. The queues add 0.30-0.71 ms and 2.3-5.7 M instructions, about 22-28
  instructions per retained row. Thin runs average 1.6-1.9 rows (retained rows against thin
  runs), so this is a per-run cost: each `queues[a].extend(m..m + w)` pays its capacity check,
  its range iterator and its loop setup for two elements. The callgrind lines put it in
  `raw_vec`, `range.rs` and `option.rs` inlined into the run pass. The scatter itself (e) is
  cheap: 3.4-5.0%.
- **The direct sequential wiring is not the cure.** `i16` saves the queue fill but pays a second
  `each_run` pass and a per-run band walk: its wiring is 1.6-1.8x the queued wiring in time on
  every block, and its total is 1.10-1.16x `qi16`'s production total here (the 233-block file found
  1.21x median). It saves 0.55-1.23 MB of live buffers.
- **The elimination** is 5.5-12.1% (c2), 8.5-18.5% with c1 and c3 (median about 15%), and
  falls as a share on the blocks with more retained rows (85, 210). **The spelling** is 4.3-9.4%.
  Neither is a clear target alone.
- **`emit` dominates the misses:** 78-190 k D1 misses a block, more than every construction
  stage together, from the walk's dependent loads over `edges` (0.36-0.82 MB: beyond L1, within
  this VM's 2 MiB L2). It is the largest stage, 35.6-47.7%, but it is the walk, not
  construction.
- **Surprises.** The clone is 1.4-2.5% faster than production for `qi16` (the deferred spelling
  is at least no slower), and 1.6-3.9% slower for `i16`; both are within the agreement asked
  for, but the `qi16` gap is consistent across blocks, which hints that the band loop with its
  inline appends is slightly worse laid out than two separate loops. Not tested further.

## Verdict for a replacement experiment

Only the **queued wiring** clears 15%: 23-28% of `qi16`'s total on all six blocks, 0.37-0.87
ms, 78-87% of it the queue fill. A targeted experiment that keeps the queued scatter but
replaces the per-run `Vec::extend` (for example, queues pre-sized from the per-byte counts and
written by index, or per-run segments instead of per-row ranks, since every queue is a
concatenation of runs of consecutive ranks) addresses the larger part; its ceiling is the queue
fill, 18-25% of the total. The direct sequential wiring (`i16`, DOE's proposal) is
measured and is worse. The elimination (about 15% with its setup) is the second candidate; the
spelling (under 10%) is not worth one alone.

## Files

- `2026-11-20-open-phase-split.csv`: every stage per block and arm: median, p10, p90, share,
  bytes written, and callgrind's counts per call. Also `total_phased`, `total_production`
  (its share column is the clone's total over production's, in %) and `total_phased_wall`.
- `2026-11-20-open-phase-split-counters.csv`: element counts and bytes per block and arm.
- `src/open/endpoint/phases.rs`, `examples/open_phases.rs` (feature `open-phases`).
- Reproduce: `cargo build --release --example open_phases --features open-phases`, then
  `taskset -c 3 target/release/examples/open_phases DIR 41 OUT`. Callgrind:
  `PHASES_ONLY=qi16 valgrind --tool=callgrind --cache-sim=yes
  --toggle-collect='*phases::stage_emit*' target/release/examples/open_phases DIR 1 x 0`.
