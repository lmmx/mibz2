# Compact Ψ, stage A: does a smaller table alone move the knee, and at what cost? (pre-registered)

- **Research only:** `examples/coop/compact.rs`, run as arms of `examples/coop.rs grid`. The decoder is unchanged.
- **Prompted by** the knee journal's footprint run (`2026-10-29-libsais-knee.md`, §3).
  - There, smaller bzip2 blocks moved the knee from 4 blocks to 8. The kernel's reads per byte were unchanged, and the L3 hit share followed live table bytes across both block sizes.
  - Smaller blocks shrink more than the table, though: L, the MTF output and the block count all change with them. So that run showed that **smaller blocks** extend the knee, not that a **smaller table** alone does.
- **This run shrinks the table and nothing else.** It uses the same blocks, the bench corpus's own Wikidata blocks. It measures two things:
  - whether the hit share and DRAM traffic follow the table's bytes as the curve predicts;
  - what reconstructing the information the smaller table drops costs.

## 1. The arms

All arms run one core a block. Each uses production's build and room walk, copied into the harness and made generic over a row's entry. The copy is needed because the library keeps its walk's scratch and table private.

| arm | a row's entry | bytes a row | a 900 K-row block's table | the byte the walk writes |
|---|---|--:|--:|---|
| `p` | production (the library itself) | 4 | 3.6 MB | from the entry |
| `o` | production's, in the copy: the control | 4 | 3.6 MB | from the entry |
| `v` | a 32-bit link, alone | 4 | 3.6 MB | from F |
| `u` | **a 24-bit link, alone** (rows < 2^20, START at bit 23) | **3** | **2.7 MB** | from F |
| `l` | the shipped cooperative kernel (`bwt::Cooperative<2>`), for reference | 4 | 3.6 MB | from the entry |

**What each comparison isolates:**
- **o against p:** the copy itself. They should be equal within noise. All compact comparisons are made against `o`, the same code.
- **v against o:** recovering the byte, plus a store-only build. `v`'s build has no init pass and no read-modify-write of the entry's byte.
- **u against v:** the packing alone. That is one unaligned 4-byte load, masked, where 3 rows in 64 cross a cache line, and the build's narrow store at each bucket's last row.
- **u against o, or p:** the whole cost or gain of the compact table.

**The byte from F** (`compact.rs`, module comment):
- At row r the walk writes L[r] = F[ψ⁻¹(r)], and ψ⁻¹(r) is the row the lane came from. So each lane carries its next byte.
- F[r] is looked up from r, the current row: a 32-row window's first byte (n/32 bytes, 28 KB a block), stepped past any bucket boundary inside the window.
- r is known before its own entry is loaded, so the lookup is off the dependent chain. It is extra work per step, not extra latency on the chain.

**Correctness (VM, before this registration):**
- **Every arm matches production byte for byte**, checked by hash in the grid's round −1. That holds for every block of 16 crafted inputs at levels 1 and 9, and of the bench corpus (431 blocks) and its bzip2 -3 recompression (700 blocks). The crafted inputs are: one byte, two bytes, a 2 MB run, period 3, period 1000, 2 MB random, 2.4 MB text-like, and 40 KB random.
- **The check was shown to catch a fault.** `u`'s narrow store at a bucket's last row was replaced by a wide one. The table's links were then corrupted, and the walk ran away until its memory cap stopped it. With the narrow store restored, every input passes again.

## 2. What is known before the run (the VM: indicative only)

- **Setup:** Xeon at 2.8 GHz, 4 cores, a **4 MB L2 a core** and a 33 MB L3. 5 rounds, unpinned.
- **Why this is the compact table's worst case:** a 3.6 MB table nearly fits in this L2, so the walk is bound by compute, with no memory latency to hide extra work behind.

| bench corpus, 1 block | MB/s | against p1 | build MB/s | walk MB/s |
|---|--:|--:|--:|--:|
| p1 | 203 | 1 | 948 | 259 |
| o1 | 201 | 0.983 | 927 | 260 |
| v1 | 200 | 0.945 | **1468** | 231 |
| u1 | 152 | 0.764 | 896 | 184 |

**Readings:**
- **The copy holds:** `o` runs as `p` does.
- **v's build gains 1.55×** from storing the link alone. Recovering the byte costs about 14% of the walk here.
- **u loses that build gain**, to the bucket-end compare and the 3-byte stride, and its walk is another 20% slower (u/v 0.80).
- **On the i9 the costs may differ.** Its L2 is 256 KB, so a lone 3.6 MB table is already served from L3, at about 52 cycles a read (the knee journal's p2). Some of the extra work can hide under that wait.

## 3. Predictions (fixed now)

**The curve.** The footprint run's L3 hit share against live table bytes (blocks × the time-weighted table), from both corpora:

| live MB | 2.4 | 4.8 | 7.2 | 9.6 | 12.0 | 14.4 | 21.6 | 28.8 | 36.0 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| hit share | 0.987 | 0.983 | 0.977 | 0.961 | 0.940 | 0.929 | 0.794 | 0.640 | 0.539 |

**What the curve predicts for p and u** (linear between the points). The native corpus's time-weighted table is 3.6 MB a block for `p` and 2.7 MB for `u`:

| blocks | 4 | 5 | 6 | 8 | 10 |
|---|--:|--:|--:|--:|--:|
| p: live MB | 14.4 | 18.0 | 21.6 | 28.8 | 36.0 |
| p: predicted hit share | 0.929 | 0.862 | 0.794 | 0.640 | 0.539 |
| u: live MB | 10.8 | 13.5 | 16.2 | 21.6 | 27.0 |
| u: predicted hit share | 0.951 | 0.933 | 0.895 | 0.794 | 0.679 |

**Prediction 1 (the scientific question: does the table alone move the knee?):**
- `u`'s hit share at 6 and 8 blocks exceeds `p`'s at the same N by at least half the predicted difference: **u6 − p6 ≥ 0.05** and **u8 − p8 ≥ 0.077**.
- `u`'s throughput peaks at more blocks than `p`'s, which peaks at 4.
- **If it fails:** the curve isn't a function of table bytes alone, and footprint run reading 2's single curve is wrong for this kernel.

**Prediction 2 (the cost):**
- The paired whole-phase ratios at 1 block: **u1/o1**, **v1/o1** and **u1/v1**. These are reported, not gated.
- 1 block on the i9 is the table served from L3 with no contention: the computational cost of compactness as the decoder would pay it at low concurrency.

**Prediction 3 (the engineering question, at this stage):**
- `u`'s best configuration (the N with the highest median) against `p4` (production's peak), paired.
- **It passes if** the geometric mean is ≥ 1.05 and its 95% lower bound > 1.00.
- **The ceiling is bounded:** at the same table bytes, `u` would run 4/3 as many blocks as `p`, times its own per-block cost. With the VM's 0.76, that is about 1.0. So this gate is expected to be close, and a pass would need the i9 to hide part of the cost.

**What decides stage B** (the cooperative compact kernel: teams of 2 on 2.7 MB tables, 5 teams on 10 cores, then a full-decoder A/B against the shipped decoder, with throughput, peak RSS and CPU-seconds):
- **Go ahead if** prediction 1 holds and **u1/o1 ≥ 0.85**. At that cost, 5 teams at `l4x2`'s live table bytes are predicted to beat it: 5/4 × 0.85 ≈ 1.06. That is before the second limit seen at 10 cores (knee journal §3, reading 5), which may take some of it.
- **If prediction 1 holds but the cost is higher:** the 24-bit table moves the knee, a scientific result, at a cost that keeps it out of the decoder. It is recorded as such. The next candidate is a cheaper byte recovery, not a smaller table.
- **If prediction 1 fails:** compact Ψ is not pursued in this form.

**Two further cautions for the reading:**
- **The second limit:** `-3` turned at 8–10 blocks with its hit share still at 0.94. `u` at 10 blocks may meet the same limit, whatever its table does.
- **The comparison is per core:** the `l4x2` and `l5x2` rows show where the shipped cooperative kernel stands on the same cores (8 and 10). Stage A compares only one-core-a-block arms with each other.

## 4. The command (the i9)

`just bench-compact-psi`: about 10 minutes.
- **The grid:** `p4` first as the reference, then p1–p10, o1 and o4, v1, v4 and v6, u1–u10, l4x2 and l5x2. 20 rounds, pinned, every block checked in round −1.
- **The counters:** reads, LLC and the memory controller; 2 repeats. On p4, p6, p8, u4, u6, u8, u10 and v6.

Upload the `coop-*.md`, the `coop-counters-*.md`, their `.jsonl`, and the console.

## 5. The result (measured, the i9, b99b92b, 2026-10-29)

**The run:** `just bench-compact-psi`. Its files are in `2026-10-29-compact-psi/`: the grid
`coop-20261029T215136Z`, the counters `coop-counters-20261029T215323Z`, their `.jsonl`, and
the console.
- **The grid:** 20 rounds, pinned, no round misplaced.
- **The check:** every arm's every block was compared with the room walk's output in round −1.
- **The counted runs** are within 1% of their grid medians.

### 5.1 The two questions, kept apart

**Scientific: does a smaller table alone move the knee? Yes.**
- The same blocks were used, with only the entry changed.
- The 24-bit table's throughput peaks at **6 blocks** (1,615 MB/s); production's peaks at
  **4** (1,461). Paired, u6/u5 is 1.089 (1.083–1.095) and u6/u8 is 1.332, against p5/p4's 0.885.
- **The hit share moved by the bytes alone.** Live table bytes predicted it within 0.031 at
  every N, on a curve fitted to block-size data from another run:

  | | u4 | u6 | u8 | u10 | p6 | p8 | v6 |
  |---|--:|--:|--:|--:|--:|--:|--:|
  | live MB | 10.8 | 16.2 | 21.6 | 27.0 | 21.6 | 28.8 | 21.6 |
  | predicted hit share | 0.951 | 0.895 | 0.794 | 0.679 | 0.794 | 0.640 | 0.794 |
  | measured | 0.971 | 0.926 | 0.809 | 0.686 | 0.793 | 0.638 | 0.814 |

- **v is the control that pins it to the bytes.** v holds the same 3.6 MB as p, with the
  byte recovered as u's is. Its hit share at 6 blocks (0.814) is p's (0.793), not u's
  (0.926).

**Engineering: is this representation worth shipping? Not as it stands.**
- At one block it costs a third of the speed.
- At its peak it reaches 0.687× the shipped cooperative kernel on fewer cores (below).

### 5.2 The pre-registered tests

| test | registered | measured | result |
|---|---|--:|---|
| 1a: hit share, u6 − p6 | ≥ 0.05 | **0.133** (0.926 − 0.793) | **met** |
| 1b: hit share, u8 − p8 | ≥ 0.077 | **0.171** (0.809 − 0.638) | **met** |
| 1c: u peaks at more blocks than p | u's peak N > 4 | **6** | **met** |
| control: o against p | ≈ 1 | o1/p1 1.004 (0.999–1.008), o4/p4 1.004 (0.994–1.015) | **the copy holds** |
| 2: the cost at one block (reported) | — | u1/o1 **0.681** (0.678–0.684); v1/o1 0.829 (0.816–0.842); u1/v1 0.821 (0.810–0.833) | reported |
| 3: u's best against p4 | ≥ 1.05, lower bound > 1.00 | u6/p4 **1.100** (1.093–1.108) | **met** |
| stage B: test 1 holds and u1/o1 ≥ 0.85 | both | 1 holds; u1/o1 0.681 | **not met: no stage B on this design** |

**Against the shipped cooperative kernel, in the same grid** (not gated):
- u6/l4x2 is 0.687 (0.680–0.695), on 6 cores against 8.
- u6/l5x2 is 0.921 (0.907–0.936).
- l5x2/l4x2 is 0.746: five full-table teams overflow the cache, as in the release run.

**The stage B rule's reasoning checked a second way:**
- At nearly equal live table bytes, u5 (13.5 MB) per block against p4 (14.4 MB) per block
  is 1.010 × 4/5 = 0.81.
- Five compact teams against four full ones would then be about 5/4 × 0.81 ≈ 1.01× l4x2.
  That is before the second limit at 10 cores, which l5x2 shows is real.
- So no gain is predicted, as the gate implied.

### 5.3 Where the cost is (paired, 20 rounds)

| | whole phase | build | walk |
|---|--:|--:|--:|
| v1/o1: the byte from F, and a store-only build | 0.829 | **1.279** | **0.721** |
| u1/v1: the 24-bit packing | 0.821 | 0.786 | 0.830 |
| v4/o4 | 0.950 | 1.490 | **0.839** |
| u4/v4 | 0.940 | 0.823 | **0.969** |
| u6/p6 | 1.460 | 2.476 | 1.258 |
| u8/p8 | 1.742 | 2.827 | 1.553 |

**Readings (measured unless marked):**
1. **Recovering the byte is the larger cost where it matters.**
   - At one block, the F lookup costs 28% of the walk and the packing 17%.
   - At 4 blocks, with memory waits to hide behind, the packing costs only 3% of the walk,
     but the F lookup still costs 16%.
   - Under memory pressure, then, the F lookup is the cost to attack.
2. **The store-only build is a gain in its own right:** 1.28× at 1 block and 1.49× at 4,
   in `v`. It drops production's init pass and the read-modify-write of each entry.
   - In u the 3-byte stride and the bucket-end compare take the gain back at 1 block.
   - Past the knee, u's build is 2.5–2.8× production's (u6/p6, u8/p8), because it moves
     fewer DRAM bytes.
3. **u makes 26% more reads past L2 per byte** than p at 4 blocks (1.765 against 1.399).
   v makes 5% more at 6 (1.454 against 1.382).
   - So the extra reads belong to the packing, not the F lookup.
   - Loads that cross a cache line (3 rows in 64) explain at most about 5 points of the 26.
   - The rest is **not explained** by these counters, which count build and walk together.
4. **The knee is where DRAM traffic reaches the plateau** (derived: counted MB/s × DRAM
   bytes per output byte):

   | | p4 | p6 | p8 | u4 | u6 | u8 | u10 | v6 |
   |---|--:|--:|--:|--:|--:|--:|--:|--:|
   | DRAM GB/s, read + write | 14.3 | 28.0 | 28.5 | 6.3 | 18.0 | 28.3 | 28.3 | 27.3 |
   | cycles a read waits | 66.8 | 137.5 | 361.4 | 52.3 | 61.4 | 145.5 | 337.3 | 116.7 |

   - u6 is still at 18 GB/s with a 61-cycle wait; u8 is at the plateau. p reaches it at 6.
   - The memory probe brackets the plateau: 26 GB/s of random reads, 34 of streaming (knee
     journal, "The memory probe").

### 5.4 What it changes

- **The original question is answered.** The cache knee moves when the navigation table's
  representation alone shrinks: by 2 blocks for a 25% smaller table. The hit share
  followed the table's bytes as the block-size curve predicted. This holds whether or not
  any compact table ships.
- **This representation is not promoted:** it fails stage B's cost gate, and is 0.687× the
  shipped kernel at its peak. Per the registration, the next candidate is **a cheaper byte
  recovery, not a smaller table.** The F lookup's 16% walk cost at 4 blocks is the target:
  - **one load, not two:** a window entry that holds the byte and a bucket-boundary flag,
    with the boundary table read only for flagged windows (about 1% of them);
  - **a smaller window table** (64- or 128-row windows: 14 or 7 KB), more likely to stay
    in L1;
  - **the same F lookup in the cooperative kernel**, whose two-core teams may hide more of
    it. That is stage B's design, run only if the lookup gets cheaper.
- **A production-facing side result:**
  - The store-only build's 1.28–1.49× (`v`) suggests a candidate: production's one-core
    build without its init pass, the byte read from L at the slot, as the cooperative
    kernel's fused build already does (#81, E11).
  - It is the plan on every machine below 6 physical cores, CI's runners included, so the
    CI bench can measure it.
  - Not yet measured: the fused one-core build keeps the byte in the entry, so it is not v's
    build, and its gain is unknown. A research harness arm, `x4x1`, ran at about p4 with
    the cooperative walk, which conflates the two.
