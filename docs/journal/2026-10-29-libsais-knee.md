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

## The result (measured, the i9, d5460e9, 2026-10-29): the same knee, a shallower fall

**The run:** `just bench-libsais-knee`,
[`coop-20261029T192315Z.md`](2026-10-29-libsais-knee/coop-20261029T192315Z.md) and its
.jsonl, load 1.2 at the start.
- Every block of every arm matched the room walk's output, and no thread was misplaced.
- The ratios below are paired over the 20 rounds, recomputed from the raw rounds.

### One core a block: mibz2's one-core inversion (pN) against libsais's (sNx1)

| N blocks = cores | p MB/s | s MB/s | s / p, paired (95%) |
|--:|--:|--:|--:|
| 1 | 544 | 546 | 0.989 (0.974–1.004) |
| 2 | 1074 | 1037 | 0.960 (0.950–0.969) |
| 3 | 1284 | 1348 | 1.049 (1.039–1.060) |
| **4** | **1467** | **1552** | 1.062 (1.050–1.073) |
| 5 | 1302 | 1499 | 1.148 (1.139–1.158) |
| 6 | 1103 | 1239 | 1.121 (1.110–1.131) |
| 8 | 713 | 887 | 1.251 (1.239–1.263) |
| 10 | 571 | 746 | 1.320 (1.300–1.341) |

**Each family's falloff from its peak:**

| | at 5 | at 8 | at 10 |
|---|--:|--:|--:|
| p (against p4) | 0.887 (0.877–0.897) | 0.482 (0.476–0.488) | 0.383 (0.376–0.390) |
| s (against s4x1) | 0.960 (0.950–0.969) | 0.568 (0.563–0.573) | 0.476 (0.471–0.481) |

### Teams of 2: the library (lNx2) against libsais (sNx2)

| N blocks (2N cores) | l MB/s | s MB/s | s / l, paired (95%) |
|--:|--:|--:|--:|
| 1 | 890 | 692 | 0.774 (0.764–0.784) |
| 2 | 1514 | 1229 | 0.811 (0.803–0.820) |
| 3 | 2053 | 1563 | 0.760 (0.749–0.771) |
| **4** | **2328** | **1652** | 0.710 (0.700–0.721) |
| 5 | 1758 | 1273 | 0.725 (0.712–0.739) |

The falloff from 4 to 5 blocks is l 0.762 (0.746–0.779) and s 0.778 (0.766–0.791).

### The pre-registered readings

1. **Every family peaks at 4 blocks.** One core a block: p4 and s4x1 (s5x1 / s4x1 =
   0.960 excludes 1). Teams of 2: l4x2 and s4x2.
2. **"libsais falls off less": yes, one core a block.** s / p is 1.062 (1.050–1.073) at 4
   blocks, 1.251 (1.239–1.263) at 8 and 1.320 (1.300–1.341) at 10. The intervals don't
   overlap. At 10 blocks libsais keeps 0.48 of its peak, mibz2 0.38.
3. **"The knee moves": no, for either.** Both peak at the same N, so libsais has a higher,
   shallower curve with the same knee. Its knee is not in a different place.
4. **Teams of 2:** no difference in shape over the one step past the peak that 10 cores
   allow (0.762 against 0.778). The library leads at every N, by 1.23–1.41×.
5. **Reproduction:** s4x1 / p4 = 1.062 here, against the rematch's 1.075 (1 / 0.930).
   Within ±5%.
6. **No core shortage:** every added block brought its own core (N ≤ 10). Both curves
   fall rather than flatten, so more blocks cost throughput: contention for something
   shared.

### What it says, and doesn't, about the cause

**What the data say:**
- **Equal alone:** at one block the two kernels are equal (0.989).
- **Apart under contention:** they separate only as blocks are added: libsais 1.06× at 4
  and 1.32× at 10.
- **The difference is how much each block's work suffers from the others,** not the
  work's own speed.
- **What libsais does differently:** its walk makes 0.90 reads past L2 a byte against
  our 1.37 (ψ², two bytes a step; the rematch's counters at 3 × 3). Fewer requests to the
  shared L3 and DRAM per output byte is the obvious candidate for suffering less from
  each other block.

**What they don't say:**
- **The cause is not settled.** The two kernels' tables are close in size (n words each),
  so capacity also predicts the same knee position. And capacity is in doubt here: these
  native blocks (about 1.95 MB of table) knee at 4, as full 3.6 MB blocks do, not near 7
  (the section above).
- **The cause run decides it:** the same sweep at full blocks, and the memory controller
  and cache counters for both kernels at 2–10 blocks on both corpora
  (`just bench-libsais-knee-cause`).
- **Fewer reads per byte is a candidate, not a finding.** It is the most direct way libsais
  differs, and it would explain a shallower fall under either cause. The counters say
  whether its DRAM traffic and L3 miss rate grow more slowly with N than ours.

**For the decoder:** the shipped teams already sit at the knee's peak (4 × 2). A shallower
fall matters past the peak, where the policy doesn't go. The shipped decoder gains from
libsais's property only if fewer reads per byte also raise the peak; at 4 blocks they're
worth 1.06× on one core a block.
