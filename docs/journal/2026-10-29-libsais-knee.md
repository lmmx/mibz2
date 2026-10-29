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

### What the data already in hand say about the cause (before the cause run)

The user's caveat, recorded: the one-core and two-core curves spend different core
budgets at each N, so their peaks alone don't isolate cooperation's effect. Read with the
counters already measured, though, they narrow the cause.

1. **The knee follows live blocks, not cores or threads.**
   - One core a block peaks at 4 blocks on 4 cores; teams of 2 peak at 4 blocks on 8 cores.
   - The same holds for libsais: s4x1 on 4 cores, s4x2 on 8.
   - So whatever binds grows with the number of live blocks: their tables, their buffers,
     or traffic per block. Thread or core count alone doesn't explain it.
2. **A fixed DRAM bandwidth ceiling at p4's peak is ruled out.** The memory controller's
   bytes (the 18:48 counter run, `coop-counters-20261029T184803Z`; 1- and 21-round runs by
   difference, medians of 3; the timed rounds' own seconds):

   | arm | MB/s | DRAM read B / output B | DRAM read GB/s | read + write GB/s |
   |---|--:|--:|--:|--:|
   | p4 (at its peak) | 1407 | 6.57 | 9.2 | 13.6 |
   | l4x2 (4 blocks, at its peak) | 2216 | 6.61 | 14.7 | 21.6 |
   | l3x3 | 2428 | 4.19 | 10.2 | 14.4 |
   | s3x3 | 2231 | 3.07 | 6.9 | 11.1 |

   - At 4 live blocks, the teams move 1.6× p4's DRAM traffic and are still rising, so the
     link isn't what stops p at 4.
   - p4 and l4x2 read about the same DRAM bytes per output byte (6.6). The same 4 live
     blocks miss about as much per byte whether 4 cores or 8 walk them.
3. **Past the knee, misses per byte multiply.**
   - The coop journal §10.4's full counter run (research/coop-fused, the native corpus) put
     p8 at 34.4 DRAM bytes read per output byte, against p4's 7.7, about 4.5×. That is about
     23 GB/s of reads at its 662 MB/s, plus 6.5 GB/s of writes.
   - The fall past 4 is a rise in misses per byte, the signature of the hot data no longer
     fitting. The DRAM traffic it creates may then press on bandwidth as well.
   - Those §10.4 figures are from a different run on the research kernels, so they're
     indicative here. The cause run measures p and s at 2–10 blocks in one run.
4. **The open contradiction.**
   - "The hot data stops fitting at 4 blocks" fits points 1–3, but on these native blocks 4
     tables are about 7.8 MB, 0.38 of the L3, not §9's 14.4 MB.
   - So either the hot footprint per block is not the table alone (the walk's output
     buffers, L and the build's scatter add to it), or something per block other than
     capacity binds.
   - The cause run's second corpus (full 3.6 MB blocks) separates them. If both corpora
     knee at 4 blocks again with misses per byte rising past it, the binding footprint
     isn't the table's size. If full blocks knee earlier, it is.

## The cause run (measured, the i9, 4db1e3f, 2026-10-29): the second corpus wasn't a second footprint, and the memory link has a ceiling

**The files:**
- the lbzip2 -9 grid: [`coop-20261029T193653Z.md`](2026-10-29-libsais-knee/coop-20261029T193653Z.md);
- the counters on the native corpus: [`coop-counters-20261029T193811Z.md`](2026-10-29-libsais-knee/coop-counters-20261029T193811Z.md);
- the counters on the lbzip2 -9 corpus: [`coop-counters-20261029T195025Z.md`](2026-10-29-libsais-knee/coop-counters-20261029T195025Z.md);
- the .jsonl beside each.

No pass multiplexed or failed, and every counted run ran within 2.6% of its confirmed MB/s.

### 1. lbzip2 -9 didn't change the footprint, and the average was the wrong measure

- **Block counts:** the lbzip2 -9 corpus has 421 blocks against the native 431 (the user spotted this independently). The two corpora's curves and counters agree to about 1%:
  - p4 1463 against 1470 MB/s;
  - L3 hit share at p10 0.537 against 0.538;
  - DRAM reads per output byte at p10 41.0 against 41.2.
- **The block-size report** (`coop sizes`, new) on a 45 MB JSON sample, on this VM:
  - **lbzip2 -9:** 101 blocks, alternating full and tiny: 49.5% at the 900 K ceiling, the rest mostly around 100 rows (p10 103, median 509,889).
  - **bzip2 -9:** 51 blocks, 98% full.
  - **bzip2 -3:** 152 blocks of 300 K.
- **What a worker holds:**
  - a block's inversion takes time in proportion to its rows, so a worker's table averaged over time is Σn² / Σn rows, not the mean;
  - by that measure lbzip2 -9 and bzip2 -9 are the same, 3.595 MB a worker, on a second sample;
  - the mean (1.80 MB for lbzip2 -9) counts the tiny blocks as if they took as long as the full ones.
- **What follows, pending the i9's own report:**
  - The native corpus is likely lbzip2-made and bimodal the same way: 431 blocks averaging 486 K rows fits half full, half tiny.
  - If so, its time-weighted footprint is about 3.6 MB a worker, not 1.95 MB. Then:
    - this entry's "anomaly" was not one;
    - the frontier page's correction (#110) was wrong and should be reverted;
    - the workstation journal §9's nominal table sizes (lbzip2 -9/-6/-3/-1) were right in the time-weighted sense;
    - its knee at about 14.4 MB of live tables (0.7 of the 20 MB L3) stands.
  - Not reverted until `coop sizes` runs on the i9's corpora.

### 2. The memory controller: a common ceiling past the knee

Read + write, from the counted MB/s and the DRAM bytes per output byte (native corpus; the
lbzip2 -9 corpus within 1.5% throughout):

| blocks | mibz2 DRAM B / output B | mibz2 GB/s | libsais DRAM B / output B | libsais GB/s | L3 hit share (mibz2, libsais) | wait per read past L2 (cycles) |
|--:|--:|--:|--:|--:|--:|--:|
| 2 | 2.5 | 2.7 | 1.7 | 1.8 | 0.979, 0.980 | 52, 53 |
| 4 | 9.5 | 14.1 | 9.4 | 14.7 | 0.930, 0.907 | 67, 69 |
| 6 | 25.4 | 28.0 | 22.8 | 28.6 | 0.794, 0.748 | 135, 175 |
| 8 | 40.5 | 28.6 | 33.1 | 29.3 | 0.644, 0.596 | 359, 393 |
| 10 | 50.7 | 28.8 | 38.8 | 29.0 | 0.538, 0.504 | 620, 621 |

**What the counters show:**
- **One ceiling for both kernels:** from 6 blocks on, both reach about 28–29 GB/s read +
  write, on both corpora, though their read/write mixes differ (at 10 blocks, mibz2 23.4
  read + 5.4 write, libsais 20.7 + 8.3).
- **Past the ceiling, throughput is ceiling ÷ DRAM bytes per output byte.** At 29 GB/s that
  predicts p10 at 572 MB/s (measured 567) and s10x1 at 747 (746).
- **libsais's shallower fall is its lower DRAM bytes per output byte** (38.8 against 50.7
  at 10 blocks), from fewer reads past L2 per byte (0.77 against 1.30).
- **Before the ceiling, between 4 and 6 blocks:**
  - the L3 hit share falls (0.93 → 0.79 for mibz2);
  - DRAM bytes per output byte nearly triple (9.5 → 25.4);
  - the wait per read doubles as the link fills.

**What it doesn't establish:**
- **The ceiling is a plateau in this workload, not yet the machine's.** The probe below
  measures the machine with a walk-shaped random pattern and a sequential copy.
- **It doesn't say why misses begin between 4 and 6 blocks.** Capacity (about 4 × 3.6 =
  14.4 MB of tables, time-weighted, plus the walk's other data) is the reading the
  corrected footprint supports. The footprint run tests it.
- **The working hypothesis, a chain:** more live blocks → the hot data outgrows the L3 →
  more DRAM bytes per output byte → the link saturates → throughput is the ceiling over
  that traffic. Each link is measured except the first's cause.

## Footprint, measured: the next run (pre-registered)

`just bench-libsais-knee-footprint` (needs bzip2 and sudo):
1. **`coop sizes` on every corpus:** native, lbzip2 -9, bzip2 -9 and bzip2 -3. The
   distributions, the share at the ceiling, and the time-weighted table a worker holds.
   The native corpus's figure decides whether #110 is reverted.
2. **The one-core-a-block sweep** (p1–p10, s1x1–s10x1, 10 rounds) on two corpora:
   - **bzip2 -9:** uniform full blocks, the same time-weighted footprint as lbzip2 -9
     (3.6 MB). It is the control for bimodal against uniform blocks.
   - **bzip2 -3:** uniform 300 K blocks, 1.2 MB. The genuine contrast.
3. **The counters** (reads, LLC, memory controller; 2 repeats) on both, for p2–p10, s4x1
   and s8x1.
4. **The machine's memory ceiling** (`bench/dram.py`, examples/dram.rs), at 1–10 threads:
   - a walk-shaped probe: 32 dependent chains a thread through a 512 MB random cycle, a
     line read, a prefetch and a byte written a step;
   - a sequential copy;
   - each under `perf stat -a`, counting only between the probe's own enable and disable
     (after its setup).

**What capacity predicts (fixed now):**
- **bzip2 -9 knees where native and lbzip2 -9 do,** at 4 blocks.
- **bzip2 -3's peak shifts substantially to more blocks.** Capacity alone would put it at
  about 3× (about 12 at 1.2 MB, beyond these 10 cores). It doesn't have to keep scaling
  through 10 blocks (the coordinator's softening): other shared data and bandwidth can
  impose another limit. The evidence is the shift, not unlimited scaling.
- **Its L3 hit share holds near 0.93 well past 4 blocks,** and its DRAM bytes per output
  byte stay low there.
- **Occupancy is read two ways:** `coop sizes` reports the time-weighted table a worker
  holds, and a simulated concurrent occupancy (N workers taking blocks in decode order,
  each block taking time in proportion to its rows: the live tables' total averaged over
  time, and its 90th percentile). The two can differ when large blocks cluster.

**Against capacity:** bzip2 -3 also knees at 4–6 blocks, with misses per byte rising as
fast. That would point at something per block other than table bytes.

**The ceiling, read three ways:**
1. **The walk probe saturates near 29 GB/s:** the decoders' plateau is the machine's
   ceiling for this access pattern.
2. **The probe sustains well above 29:** the decoders stop short of the machine's ceiling
   for their own reasons (request concurrency, latency).
3. **The copy reaches far more than the walk probe:** expected for streaming. It shows the
   access pattern's cost, not a contradiction.

## Footprint, measured (the i9, 0eeeab4, 2026-10-29): smaller tables move the knee

**The run:** `just bench-libsais-knee-footprint`. Its files are beside this journal: the two
grid reports (`coop-20261029T203034Z`, bzip2 -9; `coop-20261029T203114Z`, bzip2 -3), the two
counter reports (`coop-counters-20261029T203142Z`, -9; `-203650Z`, -3), their .jsonl, and
the console (`knee-footprint-console.log`, with the `coop sizes` reports).
- Every block of every arm matched the room walk's output, and no thread was misplaced.
- The memory probe (`bench/dram.py`) crashed on reading its own output, a parsing bug in
  the script, since fixed. perf exited cleanly, so its control handshake worked. It is
  to be rerun alone.

### 1. The block sizes, measured (`coop sizes` on the i9)

| corpus | blocks | rows a block: p10 / median / max | at the 900 K ceiling | mean table | time-weighted table a worker holds | 4 workers' live tables (simulated, mean = p90) |
|---|--:|---|--:|--:|--:|--:|
| native (Wikimedia's) | 431 | 10 / 899,938 / 900,000 | 54.1% (198 blocks under 100 K) | 1.946 MB | **3.600 MB** | 14.4 MB |
| lbzip2 -9 | 421 | 10 / 899,967 / 900,000 | 55.3% | 1.992 MB | 3.600 MB | 14.4 MB |
| bzip2 -9 | 234 | 899,981 throughout | 99.6% | 3.585 MB | 3.600 MB | 14.4 MB |
| bzip2 -3 | 700 | 299,981 throughout | 0% | 1.198 MB | **1.200 MB** | 4.8 MB |

**What the table settles:**
- **The native corpus is bimodal:** full blocks and tiny ones, as lbzip2 makes. A worker
  holds a full 3.6 MB table nearly all the time.
- **So this entry's "anomaly" was not one,** and the frontier page's correction (#110) was
  wrong. It is reverted.
- **The workstation journal §9's table sizes stand** in the time-weighted sense, and so does
  its knee at about 14.4 MB of live tables (0.7 of the 20 MB L3).
- **The lesson for every footprint figure:** weight block sizes by time (Σn² / Σn), never
  by count.

### 2. The sweep (one core a block, MTF outside the timing, 10 rounds; median MB/s)

| N blocks = cores | bzip2 -9: mibz2 | bzip2 -9: libsais | bzip2 -3: mibz2 | bzip2 -3: libsais |
|--:|--:|--:|--:|--:|
| 1 | 542 | 563 | 539 | 520 |
| 2 | 1065 | 1052 | 1084 | 1009 |
| 3 | 1283 | 1364 | 1403 | 1414 |
| 4 | **1479** | **1608** | 1855 | 1855 |
| 5 | 1309 | 1566 | 2119 | 2212 |
| 6 | 1113 | 1373 | 2615 | 2669 |
| 8 | 706 | 999 | **3279** | 3386 |
| 10 | 570 | 836 | 3077 | **3596** |

### The pre-registered readings

1. **bzip2 -9 knees where the native and lbzip2 -9 corpora do,** at 4 blocks (mibz2 p5/p4
   0.891, libsais s5/s4 0.974). Uniform and bimodal blocks of the same time-weighted
   footprint behave alike: **met**.
2. **bzip2 -3's peak moves substantially to more blocks: met.**
   - **mibz2 peaks at 8 blocks** (3279 MB/s, 2.2× its bzip2 -9 peak), and falls only
     slightly at 10 (0.94 of the peak, p10 / p8 from the medians).
   - **libsais is still rising at 10** (3596 MB/s).
   - Smaller tables let both kernels use more cores before the knee. The knee is not fixed
     at 4 blocks: on the evidence that registered it, it is set by the live tables.
3. **Not a pure table-bytes constant.**
   - **The knees in live table bytes:** mibz2's knee sits between 4 and 5 blocks at 3.6 MB
     (14.4–18 MB of tables), and between 8 and 10 at 1.2 MB (9.6–12 MB).
   - **So small blocks knee at fewer table bytes than full ones.** Something besides the
     table also scales with the number of live blocks: each block's L, its walk's output
     buffers, its segment bookkeeping, and the extra cores' own traffic.
   - **Table bytes are the dominant control, not the only one.** The counters (§3 below)
     place the extra cost: not in the cache's hit share, which follows table bytes, but in
     what each request costs when 10 cores share the uncore.
4. **libsais at bzip2 -3** stays at or ahead of mibz2 from 4 blocks on, and keeps rising
   where mibz2 turns. That fits its lower traffic per byte.

**What it means for the project:**
- **The original objective is directly supported:** a smaller live table moves the knee to
  more concurrent blocks. Here, a 3× smaller table moved mibz2's peak from 4 to 8 blocks
  and raised it 2.2×.
- **Compact Ψ is the in-decoder version of this experiment.** Shrinking the table without
  changing the block size is what would move the knee on the full-block files most users
  have. The measured knee shift is the case for it, to be shown again once a compact table
  exists.
- **A level-aware policy is now indicated.**
  - `Plan::for_threads` caps teams as if every block were a full 900 K one. On small-block
    files (bzip2 -1 to -6) the knee is at more blocks, so the policy leaves cores idle there.
  - The stream header's level gives the block size before any block is decoded.
  - This needs its own decoder A/B before any change ships.

### 3. The counters, -9 against -3 (measured; 2 repeats, 20 rounds a run, none multiplexed)

**The counted runs are the grid's runs.** Each counted run is within 1.2% of its own
corpus's grid median: -9 p4 1462 against 1479, p10 566 against 570; -3 p8 3257 against
3279, p10 3097 against 3077. The report's "confirmed" column compared the -3 runs with
the bench corpus's medians, so its ratios (1.2–5.6) are meaningless for -3.
`coop_counters.py` now leaves that column blank for any other corpus.

**The pre-registered counter reading, met:** at bzip2 -3 the L3 hit share holds at 0.94 or
more through 10 blocks, and DRAM bytes per output byte stay below bzip2 -9's at 4 blocks.

Every counted cell, ordered by live table bytes (blocks × the time-weighted table). The
first five figures are from the counter reports. DRAM GB/s and the per-block figures are
derived: counted MB/s × DRAM bytes per byte, and machine-wide figures ÷ blocks.

| live tables | corpus, arm | L3 hit share | DRAM B / output B (read + write) | DRAM GB/s | cycles a read past L2 waits | MB/s a block | reads in flight a block |
|--:|---|--:|--:|--:|--:|--:|--:|
| 2.4 MB | -3 p2 | 0.987 | 1.35 | 1.5 | 50.8 | 541 | 6.7 |
| 4.8 MB | -3 p4 | 0.983 | 1.77 | 3.3 | 57.9 | 458 | 6.8 |
| 7.2 MB | -9 p2 | 0.979 | 2.54 | 2.7 | 52.2 | 534 | 7.3 |
| 7.2 MB | -3 p6 | 0.975 | 2.77 | 7.2 | 58.6 | 434 | 6.7 |
| 9.6 MB | -3 p8 | 0.961 | 4.45 | 14.5 | 63.8 | 407 | 6.8 |
| 12.0 MB | -3 p10 | 0.940 | 6.84 | 21.2 | 82.2 | 310 | 6.5 |
| 14.4 MB | -9 p4 | 0.929 | 9.74 | 14.2 | 66.6 | 366 | 6.9 |
| 21.6 MB | -9 p6 | 0.794 | 25.6 | 28.1 | 135.5 | 183 | 7.0 |
| 28.8 MB | -9 p8 | 0.640 | 40.9 | 28.8 | 357.1 | 88 | 8.7 |
| 36.0 MB | -9 p10 | 0.539 | 50.5 | 28.6 | 623.8 | 57 | 9.4 |
| 4.8 MB | -3 s4x1 (libsais) | 0.982 | 1.30 | 2.4 | 51.9 | 460 | 3.8 |
| 9.6 MB | -3 s8x1 (libsais) | 0.954 | 3.86 | 13.1 | 56.9 | 425 | 3.9 |
| 14.4 MB | -9 s4x1 (libsais) | 0.900 | 9.69 | 15.4 | 68.5 | 398 | 4.7 |
| 28.8 MB | -9 s8x1 (libsais) | 0.625 | 29.5 | 29.3 | 332.7 | 124 | 6.8 |

**Readings (measured unless marked):**
1. **The kernel asks the cache for the same thing at both sizes.** Data reads past L2 per
   output byte are 1.25–1.29 at -3 and 1.30–1.40 at -9 (mibz2); 0.79 and 0.80–0.85
   (libsais). One block alone runs at the same speed at both sizes (p1: 539 and 542 MB/s).
   What the intervention changed is where those reads are served, not how many there are.
2. **The hit share follows live table bytes, across both corpora.** At the one table size
   both corpora reach, 7.2 MB (-9 at 2 blocks, -3 at 6), the hit shares are 0.979 and
   0.975. The -3 points (9.6 and 12 MB) fall between -9's 7.2 and 14.4 MB points. That is
   consistent with a single curve in live table bytes, from one exact overlap. It is not
   proof: one point can't exclude a small offset from the blocks' other data.
3. **-3 never reaches the bandwidth plateau.** Its DRAM traffic peaks at 21 GB/s, at 10
   blocks. -9 sits at 28–29 GB/s from 6 blocks on, and so does libsais at 8. Whether 29 is
   the machine's ceiling is the probe's question (to be rerun).
4. **An accounting identity, not a model.** Per thread, MB/s = reads in flight ÷ cycles a
   read waits ÷ reads per byte × clock. The counters satisfy it by construction (wait =
   occupancy ÷ requests). It is still a useful split. Reads in flight a block are a kernel
   constant (6.5–7 for mibz2 up to 6 blocks, about 4 for libsais), and reads per byte barely
   move. **So the knee is the wait:** -9's waits go 52 → 67 → 136 → 357 → 624 cycles, and
   -3's go 51 → 58 → 59 → 64 → 82. It also explains the libsais lead at -9, 8 blocks (1.41×
   per block): libsais keeps fewer reads in flight but makes 0.60× the reads per byte.
5. **-3's turn from 8 to 10 blocks is mostly the build, and comes with a 29% longer wait at
   a hit share still at 0.94** (paired, 10 rounds, 95%):

   | -3, p10 against p8 | speed ratio |
   |---|--:|
   | build + walk | 0.948 (0.937–0.959) |
   | build | 0.891 (0.877–0.905) |
   | walk | 0.978 (0.968–0.989) |
   | libsais s10x1 against s8x1 | 1.060 (1.049–1.070) |

   - **DRAM traffic:** writes per byte almost double from 8 to 10 blocks (1.16 → 2.17), and
     reads rise 1.42× (3.29 → 4.67).
   - **Compared with -9 at 4 blocks:** -3 at 10 blocks has a slightly higher hit share
     (0.940 against 0.929) but waits longer (82 against 67 cycles), at 21 against 14 GB/s.
   - **Hypothesis, not established:** the cost of a request grows with the number of
     cores loading the shared L3, ring and memory queues, apart from the hit share. That
     would explain why -3 turns at fewer table bytes (9.6–12 MB) than -9 (14.4–18 MB).
   - **Also a hypothesis:** the build's scattered stores into the table (write-allocate,
     then write-back once the lines are evicted) are the first to suffer. That fits the
     doubled DRAM writes and the build taking most of the loss.
   - **Not measured here:** these counters cover build + walk together.
6. **libsais at -3 is still rising at 10 blocks** (s10x1/s8x1 1.060). Its hit share at 8
   blocks (0.954) is close to mibz2's (0.961). It waits less (57 against 64 cycles) and
   makes fewer reads per byte. There is no build phase to suffer.

**What this changes in the readings above:**
- **Reading 3 of §2 is narrowed.** "Something besides the table also scales with the number
  of live blocks" was placed in the blocks' other data. The counters don't show that: the
  hit share tracks table bytes across both sizes. The extra cost at -3, 10 blocks shows up
  instead in each request's wait at nearly the same hit share. That is a property of the
  number of cores, not of the bytes a block holds (hypothesis 5).
- **The causal claim stands, qualified as before.** Changing the bzip2 level changes the
  block's L, its MTF output and its block count (700 against 234) as well as its table.
  The counters remove the most direct alternative: the kernel's demand per byte is
  unchanged, and the hit share moves with table bytes.
- **For compact Ψ:**
  - A smaller table at a fixed block size should move the hit-share curve the way these
    corpora did, if reading 2's single curve holds.
  - Its payoff is capped by reading 5: past about 8 cores the wait rises at a high hit
    share.
  - Compact Ψ also adds decode work per read. Reading 4's identity says that is paid
    only if it raises reads per byte, or lowers the reads in flight a thread keeps.

### 4. The next experiment this indicates (proposed; to be pre-registered before it runs)

**Iso-footprint lines** separate table bytes from the number of blocks. bzip2's levels give
tables of 0.4 MB a level step (level n: n × 100 K rows, 4 bytes a row). The same live table
bytes are reachable at several N:
- **7.2 MB:** -9 at 2 blocks, -6 at 3, -3 at 6, -2 at 9;
- **14.4 MB:** -9 at 4, -6 at 6, -4 at 9.

**What each outcome would mean:**
- **If the hit share and the wait are flat along each line:** table bytes alone set the
  knee, and compact Ψ's ceiling can be read off the level sweep. A 20-bit Ψ for 900 K-row
  blocks is 2.25 MB, close to -6's 2.4.
- **If the wait rises with N along a line at a constant hit share:** hypothesis 5 holds,
  and the cores' own load is a second limit that no table shrinkage removes.

**Also wanted:** a fixed-N cut across levels 2–9 (N = 8 and 10) gives throughput against
table bytes at a constant number of cores. That is compact Ψ's predicted ceiling, before
its decode cost.
