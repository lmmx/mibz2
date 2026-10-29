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
