# 2026-11-18 · MONODROMY 74 · Astra D (#600, memory traffic) verified independently. My own cache simulator, written from the stated model, reproduces 7.328 / 5.595 / 3.439 B/B and every DRAM count at 1, 2 and 4 MiB from the archived traces. The forward tiles, the RLE1 edits and the mixed-direction in-place repair are exact, and the final CRC matches on all 233 blocks. The model's total is calibrated to the i9's IMC within about 5–15%, but its read/write split is off. The final-slot repair is the variant more likely to cut time (I); a paired IMC-and-wall run decides

**The request.** The Coordinator's CHECKPOINT 6993273119: an independent verification of Astra D on #600 at
454f1fa8, `docs/journal/2026-11-18-astra-d-memory.md`.
- *Conduct:* I did not read the other Astra branches.

**Labels:** **P** · **C** · **I** · **Conj**.

**Method.**
1. **A cache simulator of my own** (`dsim.rs.txt` beside this record), written only from the README's
   stated model.
   - *Caches:* strictly inclusive, true LRU; L1D 32 KiB 8-way, L2 256 KiB 4-way, LLC 1, 2 or 4 MiB
     16-way; set index = line number mod sets.
   - *Writes:* stores write-allocate; dirty lines go back as full lines, and an outer eviction
     back-invalidates the inner copies.
   - *Boundaries:* private caches are flushed at the scanner/worker handoffs, and everything drains at
     the end.
   - *What I replayed:* the archived native traces (`frozen-native.tar.xz`) for all three arms. I did
     not use the study's simulator.
2. **The reconstructions, rebuilt in my model** (`mixedcheck.rs.txt`):
   - forward affine pieces from the runs alone, split at 128- and 256-row F tiles, with the serial
     forward walk and its descriptor scan;
   - RLE1 decoding, with the edit census and the study's three-step in-place repair simulated in one
     slot;
   - the bzip2 CRC against each block's stored CRC.

## Verdicts

| claim | verdict | basis |
|---|---|---|
| Frozen at 2 MiB: 7.328 / 5.595 / 3.439 DRAM B/B | **EXACT** | C: my simulator gives 7.3276 / 5.5954 / 3.4391 |
| Output equality, all three paths | **EXACT** | C: CRC equals the block CRC on all 233; in-place repair equals RLE1 decode on all 233; tiles walk = dense walk |
| Dirty-line ownership across buffer reuse | **EXACT** in total; the region split is a convention | C, P |
| Mixed-direction interval moves | **EXACT** | P (checked); C: 233 blocks (47 mixed-sign) and 200,000 fuzzed encodings (22,280 mixed-sign) |
| The forward successor and its serial probe count | **EXACT** | C |
| Peak storage | The allocation census arithmetic is consistent. **UNMEASURED** as RSS | |
| Work-against-traffic break-even | **EXACT** arithmetic. **The premise is questionable** (§3) | C, I |
| Calibration against the i9 IMC | **SUPPORTED** in total (within about 5–15%). **The read/write split disagrees** | C, I |

## 1. Exact reproductions (C)

**The cache ledger.** My simulator against `*-blocks.csv`, frozen, all nine arm × LLC cells:
- *What matches:* L2 requests, L2 misses, DRAM reads, write-allocates, dirty writes and terminal-drain
  writes are **identical**.
- *For example:* shipped at 2 MiB gives 310,678 / 181,269 / 801 / 51,117 / 51,127 / 32,686.
- *What that confirms:* the cache accounting, dirty-line conservation included, given the traces.
- *What it doesn't confirm:* the capture itself, which needs Valgrind and the native build.

**Ownership.** Per-line dirty conservation reproduces the totals exactly, so a written line is
neither double-counted nor missed.
- *The region attribution is a convention* (the most recent store owns the writeback), and the
  frozen table is consistent with it:
  - dirty writes sum to 51,127 and allocations to 51,117 across the rows;
  - positions and commands show dirty writes with zero allocations, because they reuse top-slot and
    rule lines;
  - entries and bottom slots allocate 6,026 lines but own only 33 dirty writes: inverse-output stores
    re-own those output-buffer lines. 8,070 + 6,026 ≈ 14,063 output lines.

**The forward tiles.**
- *Geometry:* frozen has **57,836 pieces and 7,031 headers.** 4 × (7,031 + 57,836 + 256) =
  **260,492 navigation bytes.** The ten blocks range 251,932–578,704.
- *Serial descriptor inspections* (from the tile's first piece to the piece containing the row, at
  each of the n walk steps): **4,148,528** at 128 rows, **7,476,656** at 256.
- *The walk:* 899,931 serial row transitions, one per output byte.

**RLE1 and the in-place repair.**
- *Frozen:* **34 edits** (22 contractions, 12 expansions, 0 neutral), and the lead peaks at **+79**.
- *The three-step repair* (left shifts forward, right shifts in reverse, then the marker fills)
  reproduces the RLE1 decode on **all 233** blocks. 47 of them have mixed-sign displacements.
- *Fuzzed:* 200,000 random valid encodings, 22,280 mixed-sign: 0 failures.
- *Final bytes:* the bzip2 CRC equals the stored block CRC on all 233.

## 2. Calibration against the i9 (C, I)

**The i9's IMC figures at 7 workers** (whole machine, over the Wikidata ×4 corpus):

| source | B/B | read + write |
|---|---|---|
| open-return `dram.txt`, 0.5.6 | **7.97–8.10** | 4.64–4.71 + 3.34–3.39 |
| bench-sustain, 0.5.5 | 8.00 / 7.94 | |
| Record 53's fit, 4.42 + 0.074·W² | 8.05 | |

**The model, weighted to the corpus.** Over the Wikidata ×4 corpus 228 of 233 large blocks are
Cells-admitted, and 5 fall back to T (the small blocks carry negligible output). Using the study's
per-block replays:

| | 2 MiB LLC (the 10-worker share) | 4 MiB LLC |
|---|---|---|
| Cells blocks, mean (read + write) | 8.47 (4.31 + 4.16) | 8.11 (4.08 + 4.02) |
| T blocks | 39.6 | 17.0 |
| **Weighted** | **9.14 B/B** | **8.30 B/B** |

**Verdict: SUPPORTED.**
- *The total:* at the 7-worker share of the 20 MB L3 (about 2.9 MiB), the model's 8.3–9.1 brackets
  the measured ~8.0 within about 5–15%.
- *Frozen's 7.33* is below the corpus figure because frozen is a light Cells block; the 8 eligible
  blocks average 7.58.

**But the read/write split disagrees.**

| | model | measured |
|---|---|---|
| Writes | ~4.1 B/B | ~3.35 B/B |
| Reads | ~4.2 B/B | ~4.7 B/B |

My reading (I) of each gap:
- *Writes:* the model's **terminal drain** charges every dirty line at the end of the block (32,686 of
  51,127 frozen writes). In steady state a worker reuses the same buffers, so some dirty lines are
  overwritten in cache before they are evicted.
- *Reads:* the model excludes the consumer's copy and hash of the output (the README notes it), which
  is up to about 1 B/B of reads.

**What follows:** the model's **savings are mostly write and RFO traffic**, the part it may overstate.
On frozen 7.33 → 5.60 is about one output lifetime. The real IMC saving may be smaller than modelled.

**The dependence on W.** The model's capacity sensitivity is weaker than the machine's. Record 53
measured 14.11 B/B at 10 workers, against the model's 9.1 at the 2 MiB share. The single-block replay
leaves out other workers' interference, prefetch over-fetch and real L3 conflicts.

## 3. Which variant could cut wall time at 7–10 contended workers? (I)

**What binds, from the records.** Record 71 measured 20–24.5 GB/s at the controller against a
~32 GB/s ceiling at 7–8 workers; the knee record put 7 workers at about 75% of the ceiling and 10 at
about 86%. **The binding resource at 7 is latency of dependent loads,** records 47–53. Bandwidth
starts to matter near 10.

**The study's price is a bandwidth price.** It credits each saved byte at W × 4.8 / BW cycles: 2.71
events per byte at 7 workers, 3.36 at 10. That presumes bandwidth binds, which is true at most near
10. At 7, a saved byte is worth less than that price.

**The forward tiles: no (I).**
- *Its work:* 899,931 serial row transitions per block, with 4.6 descriptor inspections each and
  82.3 instruction events per output byte, against 22.4 for shipped.
- *Its chain:* it lengthens the dependent chain about 10-fold against Cells (92,570 cuts).
- *Even at* L2-hit latency, the serial walk costs more than the whole shipped kernel.
- *Its traffic saving* (3.9 B/B) lands mostly on bandwidth that isn't binding at 7.

**The final-slot repair: the more likely of the two to cut time (I),** and its gain would be small.
- *Its extra work:* +0.58 events per byte (+2.6% of instructions).
- *Its extra loads:* +25.6k LLC hits per block, in **streaming** span moves (prefetchable, high
  memory-level parallelism), not on the induction's dependent chain.
- *What it removes:* about one output lifetime of RFO and writeback traffic.
- *A possible route to a real gain:* fewer RFOs means less **fill-buffer occupancy.** Record 51
  attributed 53% of sampled load latency to fill-buffer hits, and the dependent loads of every worker
  compete for those buffers. Reducing RFOs could shorten their queueing even where bandwidth is slack.
  That is the mechanism-consistent route.
- *My expected size:* ≤ 2–3% at 7 workers, possibly more at 10, where bandwidth nears its ceiling.
  The model-overstated write saving (§2) argues for the low end.

**The cheapest measurement that decides it.** A paired `ab` run on the i9.
- *The arms:* shipped against final-slot repair, built kernel-only (`parallel_lazy_as`-style;
  record 71).
- *The cells:* unpinned 7/28 and 10/40, at the 2 s settle, 12 rounds, on Wikidata ×4.
- *Per decode:* IMC read and write bytes (`counters.py`), wall, `l1d_pend_miss.fb_full` and
  `offcore_requests_outstanding` cycles, and peak RSS.
- *Predicted under my reading:* IMC drops by about 1–1.7 B/B, mainly in writes; FB-full cycles fall;
  wall falls 0–3% at 7, more at 10.
- *Refuted if:*
  - IMC drops by less than 0.7 B/B, which means the model overstated the saving; or
  - traffic drops but wall and FB-full cycles don't, which means the traffic wasn't on the critical
    path.
- *The tiles need no machine run.* Their work ratio already rules them out at 7–10 workers.

**Peak storage.** The allocation census (at 7/28: 217.7 / 213.1 / 101.2 MB requested) is a
round-robin heap model, not RSS. Record 64's finding that 8 workers cost 13–14% more RSS makes a
measured RSS necessary in the paired run.

**What this decides:** nothing in production. The final-slot repair is a candidate for the paired run.
