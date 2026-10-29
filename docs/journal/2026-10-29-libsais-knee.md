# The knee, matched: does libsais fall off less as concurrent blocks rise, and why?

**Status:** pre-registered on 2026-10-29, before any run. The results go below, under
their own heading.

## The question, kept apart from the decoder work

The cooperative decoder raised throughput around the shared-cache knee (workstation
journal §9: mibz2's workers peak at 4 blocks, about 14.4 MB of live tables, 0.7 of the
i9's 20 MB L3, and fall off past it). It did that by keeping fewer tables live. It did not
show that the knee itself moves.

This experiment asks whether a different inversion, libsais 2.12.1's, degrades less as
concurrent blocks rise, at the same core budget and on the same blocks.
- **If it does,** the next question is which of its properties buys that: fewer reads past
  L2 per byte (0.90 against our 1.37: its bigram ψ² walk), its working set, or its
  prefetch distance.
- **This is the original knee objective.** It is distinct from the MTF-overlap work, which
  improves the shipped decoder within the knee.

## What is run

The rematch's harness (`examples/coop.rs grid`, PR #107) at this branch's head:
- **Boundary:** build + walk (the MTF outside the timing), every thread pinned, one CPU a
  physical core, 20 rounds, the arms' order rotated and reversed by round.
- **Blocks:** the bench corpus's 431 Wikidata blocks; every block of every arm checked
  against the room walk's output, untimed.

| family | arms | blocks live | cores |
|---|---|--:|--:|
| one core a block: mibz2 | p1, p2, p3, p4, p5, p6, p8, p10 | N | N |
| one core a block: libsais | s1x1 … s6x1, s8x1, s10x1 | N | N |
| teams of 2: mibz2 (v0.2.0's library) | l1x2 … l5x2 | N | 2N |
| teams of 2: libsais | s1x2 … s5x2 | N | 2N |

**Each block's working set** (computed from source, not measured):
- **mibz2's T-vector:** n × 4 bytes, 3.6 MB a full block, plus the walk's segment buffers.
- **libsais's:** its `A` array, n + 1 words (3.6 MB, given by the caller), plus its
  context's bucket and marks tables (about a quarter of a MB). So the tables are close in
  size, and a difference in falloff would come from the access pattern, not the footprint.

## How it will be read (fixed now)

1. **Each family's curve:** median MB/s against N, its peak N, and its falloff (MB/s at
   the largest N over MB/s at the peak).
2. **The matched comparison, at each N:** the paired ratio libsais / mibz2 within a family
   (sNx1 / pN; sNx2 / lNx2), with its 95% interval, from the raw rounds.
3. **"libsais falls off less"** if:
   - the one-core-a-block ratio sNx1 / pN at N = 8 and 10 exceeds its value at N = 4;
   - by more than both intervals' half-widths (the intervals at 4 and at 10 don't
     overlap).

   Otherwise its curve is the same shape at a different height.
4. **"The knee moves"** (for either implementation, against p's) only if:
   - its peak N is above p's peak N;
   - with its MB/s at that N above p's at p's peak.

   A higher curve with the same peak N is not a moved knee.
5. **Teams of 2:** the same readings on lNx2 and sNx2. N ≤ 5 (10 cores), so only the
   left side of the curve is visible.
6. **The p and s arms at N = 4 reproduce** the rematch's p4 and s4x1 (their ratio was 0.93
   in both passes), within about ±5%.

7. **A core ceiling is ruled out by the design, a capacity knee is not proven by it.**
   - **No core shortage:** in the one-core-a-block family every added block brings its own
     physical core, and N never exceeds the 10 there are. Total MB/s falling as N rises
     can't be a shortage of cores. It is contention for something shared: L3 capacity,
     L3 or uncore bandwidth, or DRAM bandwidth.
   - **How mibz2's was attributed to capacity:** the workstation journal §9 moved its knee
     with the block size (the knee tracked live table bytes, not N).
   - **libsais's knee, if it has one,** gets no such attribution from this run. If its
     falloff differs from ours, the follow-up registers the same sweep at a smaller block
     size (the corpus recompressed at a lower level, as §9 did). Moving with table bytes
     says capacity; staying at the same N says bandwidth.
   - **Separately labelled:** a curve that merely flattens (more cores buying nothing) is
     reported apart from one that falls (more blocks costing throughput).

**Not read from this run:** why. If libsais falls off less, the counters at N = 4 and 10
(p and s) are the next run: DRAM bytes and reads past L2 a byte, which say whether its
fewer reads are what spares the L3. That run isn't registered until this one says it's
needed.

## The command

```sh
cd ~/dev/mibz2-coop
git fetch origin research/libsais-rematch
git checkout --detach origin/research/libsais-rematch
just bench-libsais-knee
```

It takes about 10 minutes. Upload the `coop-*.md` and its `.jsonl`.
