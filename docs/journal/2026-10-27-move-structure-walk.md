# A length-capped move structure for ψ, walked as one chain and as the room walk (#42): the chain wins, the room walk loses

**Result: negative on #42's criterion.** On the Wikidata blocks, a length-capped move
structure's build plus a 32-lane room-style walk over it costs **+90% to +108%** against
today's `bwt::build` + `bwt::walk_segments_room`, locally, at one thread. The criterion was
−10% or better. The same structure walked as **one chain** from `orig_ptr` costs **−59% to
−63%** against the plain one-chain walk (`bwt::walk`). The structure makes a dependent
step cheaper and an interleaved step dearer, so the loss is a finding about the schedule
mibz2 walks with, not about compressing ψ.

Nothing here changes `src/`. The scratch example (`examples/movest.rs`) is kept on the
branch `scratch/move-structure` as the record; it is not for master.

Every finding is labelled:
- **ours, measured:** a figure from the local runs below;
- **interpretation:** ours, and untested.

## The setup (ours, measured)

- **Corpus:** the bench's `wikidata.bz2` (SHA-256 `82f65ace0731962d…`), the 233 blocks of
  ≥ 65,536 rows (209.7 M rows, 99.996%). The 8,895 rows in smaller blocks are skipped, as
  `walk_segments_room` leaves those to `walk`.
- **Machine:** the shared 4-core VM, "Intel Xeon Processor @ 2.80GHz", 1 MB L2 a core,
  33 MB L3, no hardware performance counters. One release build of master `3e5de26` (0.1.5)
  plus the example.
- **Timing:** each arm timed per block, best of 3, summed over blocks, in cycles per
  decoded byte at a nominal 2.8 GHz. All arms run on the same block back to back. Two full
  runs, the T-vector arms first in run 1 and the move arms first in run 2.
- **Checks:** every arm's output on every block is compared byte for byte with
  `bwt::walk`'s, and each block's RLE1-decoded bytes with its stored CRC.
- **Scale:** local figures run about twice CI's (local build 2.7 and room walk 9.3 c/B,
  against CI's ≈1.2–1.7 and ≈3.8–6.3, 1.5–2.4× as much). They are indicative only.

## The structure and the walks

- **ψ in the F formulation:** the walk emits F[row] and moves to ψ(row), starting at
  `orig_ptr`. That gives `bwt::walk`'s bytes, since L[ψ(k)] = F[k]. Each of ψ's input
  intervals lies in one F bucket, so an interval carries one output byte.
- **The intervals:** the runs of L, counting-sorted by byte into F order, each cut every
  c·n/r rows (the cap) from its own start. Each interval is one u64: idx (the interval
  holding its image, 23 bits), a segment-start flag, the byte, d (the image's offset in
  interval idx, 16 bits) and len (16 bits).
- **A lane's state** is one u64, offset << 32 | interval. A step reads its interval's record,
  fast-forwards while the offset is at least len, emits the byte, and sets the state to
  (offset + d) << 32 | idx with one AND, one AND and one add.
- **The fast-forward is lazy:** it runs when the record is next read, so the prefetch of
  R[idx] issued at the step covers it. In the "select" variant the first fast-forward is a
  branch-free select between R[x] and R[x + 1] (a sentinel record ends the array), and
  further ones are a loop.
- **The segmented walk** is `walk_segments_room_with`'s schedule (passes of `room` rounds
  with a shared offset, the same 512 pseudo-random starts with `orig_ptr` among them).
  Each start is its own one-row interval, flagged, so a segment ends when a lane's record
  has the flag. A lane taking a segment reads its start's interval from a table filled
  by the build: the handoff's state initialisation is one load.
- **The build** is a run scan of L (8 bytes at a time against the word a byte back),
  then three passes over the runs in L order: F positions and piece counts; pieces
  written into 256 per-byte streams; each piece's image merged against the interval
  starts for idx and d.
- **Multi-cycle blocks:** none. The chain from `orig_ptr` returns to it first after exactly
  n steps on all 233 blocks.

## Measured (ours, measured)

**T-vector arms** (run 1 / run 2, c/B):

| Arm | Run 1 | Run 2 |
|---|--:|--:|
| `build` | 2.684 | 2.717 |
| `walk` (one chain) | 63.773 | 61.950 |
| `walk_segments_room` | 9.348 | 9.255 |
| `build` + `walk_segments_room` | **12.032** | **11.972** |

**Move-structure arms** (run 1 / run 2, c/B; percentages against the T-vector arm in the
same run):

| Arm | c = 2 | c = 4 | c = 8 |
|---|--:|--:|--:|
| run scan of L | 0.944 / 0.936 | 0.934 / 0.926 | 0.940 / 0.945 |
| build from runs, no starts | 6.837 / 6.690 | 6.421 / 6.245 | 6.212 / 6.145 |
| build from runs, 512 starts | 8.053 / 8.016 | 7.260 / 7.102 | 6.779 / 6.686 |
| move build (scan + runs, 512 starts) | 9.167 / 9.187 | 8.411 / 8.200 | 7.859 / 7.750 |
| **(1)** one chain, select | 25.191 / 25.234 (**−60.5% / −59.3%**) | 24.228 / 23.800 (**−62.0% / −61.6%**) | 23.487 / 23.124 (**−63.2% / −62.7%**) |
| (1) one chain, loop | 25.972 / 25.517 | 24.675 / 24.257 | 23.888 / 23.739 |
| (2) room, 32 lanes, loop | 21.492 / 21.588 | 22.681 / 22.953 | 24.235 / 24.588 |
| (2) room, 32 lanes, select | 14.441 / 14.726 | 15.587 / 15.921 | 17.353 / 17.709 |
| (2) room, 24 lanes, select | 13.935 / 13.845 | 14.957 / 15.664 | 17.034 / 17.100 |
| (2) room, 16 lanes, select | 13.880 / 13.520 | 15.161 / 15.475 | 17.091 / 17.246 |
| **(2)** the best room walk above | 13.880 / 13.520 (**+48.5% / +46.1%**) | 14.957 / 15.475 (**+60.0% / +67.2%**) | 17.034 / 17.100 (**+82.2% / +84.8%**) |
| **(3)** move build + best room | 23.047 / 22.707 (**+91.5% / +89.7%**) | 23.368 / 23.675 (**+94.2% / +97.8%**) | 24.892 / 24.851 (**+106.9% / +107.6%**) |
| run-list build + best room | 21.933 / 21.536 (+82.3% / +79.9%) | 22.216 / 22.577 (+84.6% / +88.6%) | 23.812 / 23.786 (+97.9% / +98.7%) |

The last row leaves out the run scan, as if MTF handed over the run list.

**The structure and the walk's fast-forwards** (identical in both runs):

| | c = 2 | c = 4 | c = 8 |
|---|--:|--:|--:|
| intervals / n, no starts | 0.1020 | 0.0862 | 0.0787 |
| intervals / n, 512 starts | 0.1030 | 0.0872 | 0.0798 |
| size, 8-byte records | 0.824 B/row | 0.698 B/row | 0.638 B/row |
| a mean 900k-row block | 742 kB | 628 kB | 574 kB |
| the largest 900k-row block | 2,005 kB | 1,724 kB | 1,584 kB |
| fast-forwards a step, one chain | 0.341 | 0.415 | 0.532 |
| fast-forwards a step, room walk | 0.352 | 0.437 | 0.573 |
| steps with ≥ 1 / ≥ 2, room walk | 0.272 / 0.028 | 0.283 / 0.045 | 0.292 / 0.069 |

The chain's intervals and fast-forwards match #42's counter (0.0859 and 0.416 at c = 4).
The fast-forwards are counted in the walk itself.

**The segment handoffs:**
- 119,103 handoffs, 511 a block. Each is one load of a table filled by the build.
- The starts add 921–944 intervals a block (two boundaries each).
- They add 0.011–0.041 fast-forwards a step (the room walk's row against the chain's above).
- They add 0.54–1.33 c/B to the build (512 starts against none).
- **Anchoring matters:** cutting at a start and restarting the cap's count there shifted
  the rest of that run's cut points. At c = 4, 512 starts then took one block from 0.415
  to 0.493 fast-forwards a step. With the cut points anchored at each run's own start,
  the same block's figure was 0.428.

## Where the cost went

- **The room walk is bound by instructions per step, and the move step is ~2.6× longer.**
  - The compiled common path is 16 instructions a step in `walk_segments_room`.
  - It is about 41 in the move room walk (select variant): the state load and store, the
    record and its neighbour, the length compare, two selects, the second fast-forward's
    compare, the flag test, the state update, the prefetch and the byte store.
- **The lane count barely matters.** 16, 24 and 32 lanes are within 10% of each other.
  The move room walk is only 1.4–1.9× faster than the move chain, while the T-vector
  room walk is 6.7–6.8× faster than its chain.
- **The fast-forward as a branch costs about 7 c/B more than the select** (6.9–7.1, the
  32-lane loop rows against the 32-lane select rows). It is taken on 27–29% of steps.
- **The move walk alone is over today's build and walk together.** The best room walk
  (13.5–17.1 c/B) exceeds 12.0 c/B, so no build speed meets the criterion.
- **The build** is 7.8–9.2 c/B against 2.7. The run scan is 0.93 c/B; the three O(r)
  passes take about 30–40 cycles a run each (timed inside the build, over all its calls).

**Interpretation:**
- **What the chain shows:** one dependent chain waits on each load, so an L2-sized table
  saves most of each step's latency. Brown & Langmead's setting is this one.
- **What the room walk shows:** with 32 lanes and the prefetch, the T-vector walk's misses
  are already in flight, and its cost is its instructions. The move structure keeps every
  step and adds ~25 instructions to each. That is #34's and #36's lesson again: the walk is
  bound by instructions per step.
- **"Fits in L2" is not "runs from L2":** the mean block's structure (0.57–0.74 MB) fits a
  1 MB L2, but the largest blocks' (1.6–2.0 MB) don't, and the walk's output (0.9 MB a
  block) streams through the same L2.
- **The build** was not tuned. Its loops branch on run lengths and its first pass divides
  by the cap. A tuned O(r) build would still leave the walk above today's total.

## What this closes, and what it doesn't

- **Closed:** replacing the T-vector with a length-capped move structure under the room
  walk's schedule, on one thread, at c = 2, 4 and 8, with the build from L's runs or from
  a given run list.
- **Not closed (a finding about the schedule):** the move structure cuts a single chain's
  cost by 59–63%. A schedule with fewer independent chains in flight would gain where the
  room walk loses, or one bound by memory rather than instructions. Two untested cases:
  - four workers sharing the L3 and memory (#41's sweep is the instrument);
  - a host where the T-vector's misses are slower.
- **One unreplicated observation:** in a 6-block, best-of-1 check during contention on the
  shared machine, `walk_segments_room` measured 19.4 c/B, while the move room walk's
  select variants measured 16.0–20.3. The full runs above saw no such contention.
- **The MTF-side run list** (#42's comment, point 4) is not needed for this verdict. The
  walk alone exceeds today's total.

## Limits

- Local timings on a shared VM, two runs. The CI bench decides performance, and none was
  run, since the criterion failed by a wide margin.
- The records are 8 bytes, not bit-packed (0.70 B/row at c = 4 against the counter's
  0.695). A bit-packed record would need more decoding instructions in the step that
  bounds the walk.
- One thread only. Four threads were not measured.
- **Best of 3 on the same block is a warm state no decode reaches.** Found after this
  run, in the sweep ([#45](https://github.com/lmmx/mibz2/pull/45)): walking a block again
  in the same layout runs faster than the first walk, by up to a quarter for
  `walk_segments_room` at 512 segments on this machine (4.05 → 3.00 ns/B by the fourth
  call). Each arm here kept its best of three repeats, so each was measured warm. The bias
  should favour the larger working set (the 3.6 MB T-vector) over the 0.6 MB structure,
  which flatters today's arms. Taking the whole quarter off them, (3) would still be
  about +50% or more, so the verdict stands; the percentages above are not decode-state
  figures.
