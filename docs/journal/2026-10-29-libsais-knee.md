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

## Footprint and bandwidth: what causes each knee (pre-registered, a second recipe)

**An anomaly already in the record.** The bench corpus's blocks are Wikimedia's own: 431
blocks averaging 486,543 rows (the harness counts 209.7 MB of BWT over 431 blocks; per
window, 440–731 K rows a block). That is about **1.95 MB of table a block, not 3.6 MB**.
The 3.6 MB of the workstation journal §9 was its lbzip2 -9 recompression's (≈233 blocks
of 900 K).
- **The two curves are the same shape:** on the native blocks the old decoder peaks at 4
  workers and falls off almost exactly as §9's 3.6 MB blocks do:

  | blocks | 4 → 6 workers | 6 → 8 workers | source |
  |---|--:|--:|---|
  | lbzip2 -9 (3.6 MB tables) | 1026 → 889 | 889 → 698 | §9 |
  | native (≈1.95 MB) | 1007 → 858 | 858 → 674 | the decoder A/B, coop-decoder-20261029T174657Z |
  | lbzip2 -6 (2.4 MB) | 1156 → 1327 | peak at 6 | §9 |

- **They should differ if table bytes set the knee:** a knee set by live table bytes
  (§9's 14.4 MB, 0.7 of the L3) would put the native blocks' knee near 7, beside -6's 6,
  not at -9's 4.
- **They are confounded:** the native blocks vary in size, come from 12 separate windows
  of the dump, and were measured in a different run from §9's.
- **The correction this forces:** the coop journal's §7.1 note ("five tables (18 MB) is
  0.9 of the L3"), the policy's `FULL_TABLE` rationale and the frontier page's
  `table_mb` for series 2 and 3 all used 3.6 MB a block on the native corpus. They
  overstate its live tables about 1.85×. The policy's cap of 4 teams agrees with the
  measurements on this corpus regardless (4 × 2 beat 5 × 2). Its stated reason is what
  doesn't hold, not its choice.

**The run, `just bench-libsais-knee-cause`** (needs lbzip2, and sudo for perf):
1. **The corpus at full blocks:** the same content recompressed with lbzip2 -9 (900 K
   blocks, 3.6 MB tables), as §9 made it (`corpora/wikidata-l9.bz2`).
2. **The one-core-a-block sweep on it,** at the grid boundary: p1–p10 and s1x1–s10x1, 20
   rounds. With `just bench-libsais-knee`'s native sweep this gives each kernel's curve at
   two footprints, over the same content and the same block counts.
3. **The counters on both corpora** for p and s at 2, 4, 6, 8 and 10 blocks (`perf stat`,
   1 and 21 rounds by difference, 2 repeats):
   - the memory controller's reads and writes, machine-wide;
   - the reads past L2, their wait and their number in flight;
   - the LLC's references and misses;
   - the super-queue-full cycles;
   - the fill-buffer rate.

**What each cause predicts (fixed now):**

| | capacity (live tables overflow the L3) | bandwidth or queueing (DRAM or the uncore saturates) |
|---|---|---|
| knee position against footprint | moves to more blocks as the tables shrink: native about 1.85× -9's N | about the same N on both corpora |
| L3 hit share of reads past L2, past the knee | falls | holds |
| DRAM bytes per output byte, past the knee | rises | holds |
| DRAM GB/s (bytes per output byte × MB/s) | rises with N, then falls with throughput | rises to a common ceiling across kernels and corpora, then flat |
| wait per read past L2 | rises with the misses | rises without more misses (queueing) |

- **The readings are taken at each kernel's own knee and two steps past it,** on each
  corpus.
- **Both causes, or neither, may show.** Each row is reported as it reads, and a mixed
  verdict is a result.
- **Your caveat holds:** a smaller block also changes locality, per-block overhead and
  traffic. So the counters, not the knee's position alone, carry the attribution.
- **libsais is read the same way.** If its knee has a different cause from ours, that is
  the finding that says which of its properties to borrow.

## The command

```sh
cd ~/dev/mibz2-coop
git fetch origin research/libsais-rematch
git checkout --detach origin/research/libsais-rematch
just bench-libsais-knee
```

It takes about 10 minutes. Upload the `coop-*.md` and its `.jsonl`.

Then, for the cause (about 35 minutes; lbzip2 and sudo):

```sh
sudo -v
just bench-libsais-knee-cause
```

Upload every new `coop-*.md` and `coop-counters-*.md`, and their `.jsonl`.
