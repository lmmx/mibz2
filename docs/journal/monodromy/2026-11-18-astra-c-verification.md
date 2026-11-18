# 2026-11-18 · MONODROMY 72 · Astra C (#601, output-first reconstruction) verified independently. The word/exit theorem, the first-return clip and the f − 1 bound hold, and every frozen count reproduces exactly. My cursor, built from run arrays alone, is exact on all 233 blocks. The cache arithmetic checks. In my reading (I) it is unlikely to cut time at 7–10 contended workers against LazyCells: it trades write bandwidth, which is not binding, for a dependent-read chain about twice Cells'. A single-thread timing on frozen would decide whether to go further

**The request.** The Coordinator's CHECKPOINT 6992909993: an independent verification of Astra C on #601 at
bfa4564c, `docs/journal/2026-11-18-astra-c-output-first.md`.
- *Conduct:* I did not read the other Astra branches (a, b, d).

**Labels:** **P** · **C** · **I** · **Conj**.

**Method.** I added Astra C checks to my model (`mixedcheck.rs.txt` beside this record).
1. **Pieces of S = LF⁻¹** built *only* from the coalesced runs of L. Each run of byte c at L-position
   s maps to the F-interval [C[c] + rank, … + width), with displacement s − (C[c] + rank). No dense
   inversion is used.
2. **The dense walk** F[x], x ← S(x), compared with my independent LF inversion (record 66's
   `invert`).
3. **Exhaustive small checks** of the theorems.
4. **The comma-section cursor itself:** first-return certificates, the final clip, a direct-address
   table over A, and copies from the completed prefix. Run on all 233 large blocks.

## Verdicts

| claim | verdict | basis |
|---|---|---|
| The stable-select word/exit theorem | **EXACT** | P (checked), C: 438,967 equal-word pairs, 0 failures |
| The last interval clip at a section return | **EXACT** | P, C: clipped certificate = maximal (word, exit) class on all 101,475 anchor rows |
| The bound: at least f − 1 eventual entries, full period | **EXACT** | P (checked), C: all 233 blocks (230 at f − 1, 3 at f) |
| Recovery from the initial run arrays | **EXACT** | C: my run-only cursor is exact on all 233 blocks |
| Frozen: 206,638 bytes direct, 693,293 copied | **EXACT** | C: every `section.csv` field on all 10 blocks |
| Recognition and dependent loads | **EXACT** as counts (413,276 = 2 × 206,638 lookup stages, plus 31,117 probes) | C, P |
| Paired eager/lazy cache ledger (lazy: −37.2% 256 KiB transfers, +24.9% read fills) | Arithmetic **EXACT** from the study's tables. **UNMEASURED** in time | C |
| Every fact comes from the compressed input at its claimed cost | **Yes,** for the section cursor. One caveat: the *choice* of anchor (§2) | C, I |

## 1. The mathematics (P, C)

**The word/exit theorem.**
- *The proof:* S is a select on each F-bucket, so it is increasing there, and distances can't shrink.
  Two rows keep their distance through a step exactly when the whole range between their occurrences
  lies in one run of L. Induction then gives intervals, monotonicity, and "a shared additive exit
  ⇔ the same piece sequence".
- *Checked:* every k ≤ n + 2 on all 14,209 small marked columns (binary to length 9, ternary to 6).
  The rows spelling each word always form an interval with S^k increasing and S^k(x) − x
  nondecreasing. The shared-exit equivalence held on all **438,967** equal-word pairs. The study tested
  56,059 pairs under a different enumeration.
- *The witness:* `L = 010` reproduces.

**The first-return clip.** Pieces lie inside one F-bucket, so a path's certificate excludes earlier
returns automatically. The final clip to A − d makes every terminal row a member of A.
- *Checked:* for every byte present as the anchor, in every small column, the clipped certificate
  equals the maximal interval of anchor rows sharing the same (first-return word, exit). That held on
  all **101,475** anchor rows.
- *Is the clip needed?* Yes, in small cases: the section cursor relied on it 5,976 times over
  34,433 small runs (`L = 100` reproduces). On the 233 Wikidata blocks it is never strictly needed (0).

**The f − 1 bound.**
- *The proof:* a full-period orbit visits each of the f anchor rows once. Every miss but the final one
  publishes at least its own row.
- *Counted:* all 233 blocks are full period. **230 end with f − 1 entries and 3 with f.**
- *The no-reuse family* (an `a` followed by `b`ʲ for j = 1 … m) is P by the study's argument; I did
  not rerun it.

## 2. The costs: is everything obtained from the input as claimed? (C)

**My cursor uses only run arrays.** Its inputs are L's runs (byte and width), the byte histogram
(the section [C[comma], C[comma] + f)), the primary, and the anchor byte. It reproduced the study's
`section.csv` exactly on all **10** blocks:

| field | frozen |
|---|---|
| f | 31,117 |
| prefix | 6 |
| probes | 31,117 |
| hits | 25,865 (partial 0) |
| direct steps | 206,638 |
| copied bytes | **693,293** |
| certificates | 5,251 (3,949 singletons) |
| posted rows | 31,116 |
| first copy | offset 1,006 |
| last publication | offset 899,731 |

On all **233** blocks its output equals the dense walk, and the dense walk equals LF inversion.
**Nothing in it is recovered from the induction it claims to avoid.**

**The other counts:**
- *The bare select-scan closed form,* Σ(i + 1)w_i − (h + 1) over 127-capped records, gives
  **32,808,502,739** on frozen: exact.
- *The linked arm's figures* (33,214 probes, 152,373 linked reads) are the study's. I did not rebuild
  that arm; it was stopped anyway.

**One caveat: the anchor.** The comma was picked after a diagnostic screen that *did* decode the
frozen block (the first table, with dense orbits and free recognition).
- *For a decoder,* the anchor must be chosen from the histogram alone. That rule is unspecified.
- *Off JSON* the choice matters: the no-reuse family, or an anchor absent from the marked cycle, makes
  the cursor walk every byte.
- *So:* a production design needs an input-only anchor rule and an admission gate.

**The cache ledger (C, from `populations.csv`).** The arithmetic reproduces.
- *256 KiB, lazy:* transfers 2,683,457 → 1,686,483 (0.628). Demand-read fills 874,917 → 1,092,392
  (1.249). Break-even 4.58.
- *32 KiB:* break-even 0.62.

**Not in the summary, but in the same table:** at the capacities relevant to a shared 20 MB L3,
**demand-read fills also fall.**

| | transfers | read fills |
|---|---|---|
| Lazy, 2 MiB | 0.789 | 0.573 |
| Lazy, one-seventh of 20 MiB | 0.827 | 0.945 |
| Eager, 256 KiB | 0.506 | 0.926 |

## 3. Does it plausibly cut time at 7–10 contended workers? (I)

**What the model says the cursor changes, per block on the lazy population:**
- *It removes mostly write traffic:* dirty writebacks at every level. Transfers fall 0.63–0.83.
- *It adds L2-miss reads:* about +24k per block at the 256 KiB level, which become L3 hits.
- *It barely changes DRAM-level read fills:* 0.945 at the shared-L3 share.

**The serial chain is about twice Cells'.**
- *The cursor:* every direct byte costs two *dependent* lookups (rank page, then run payload, which
  gives the next row). That is a median of **0.57 dependent stages per output byte** over 233 blocks
  (0.49 on frozen).
- *Cells,* roughly: one or two dependent link reads per cut plus the writer's chains, about
  0.2–0.25 per byte (92,570 cuts and 9,628 copies on frozen).

**What the records say is binding at 7–10 workers.**
- *Not DRAM bandwidth:* record 71 measured 20–24.5 GB/s against a ~32 GB/s ceiling.
- *Latency of dependent loads is:* records 47–53, where L3-hit latency rises with the contended miss
  rate.
- *Hence:* the cursor's principal modelled saving (dirty-line bandwidth) targets a slack resource.
  Its costs are extra L3-hit reads on a dependent chain about twice as long, in exactly the resource
  that inflates under contention. That is the **opposite** of the shape record 71 credited for opend's
  gain (streaming, high memory-level parallelism, less latency-sensitive).

**My view (I).** **It is unlikely to cut time against LazyCells at 7–10 contended workers.**
- *Where it could help:* at one thread, or against eager Cells at low W. The census puts eager Cells
  at about 1.06 ns per row and a plain dependent walk at about 1.63 ns per row. The cursor walks only
  27% of bytes (median) and copies the rest, so a single-thread cost below Cells' is plausible.
- *Why that isn't evidence:* L1 and L2 read fills rise 1.8–1.9× at 32 KiB, and the 256 KiB read
  fills rise against lazy.

**The measurement that would decide it.** In two stages.
1. **Cheap, first:** a native Rust prototype of the section cursor, timed single-thread on the frozen
   block and on the ten sample blocks in ns per row.
   - *Compared with:* the eager and lazy kernels on the same machine (the census's c_whole ≈ 1.06 ns
     per row eager; ns_walk 1.63).
   - *The stop rule:* if it isn't below the lazy kernel at one thread, stop; contention would only
     widen the gap.
2. **If it passes:** a kernel-swap arm (the `parallel_lazy_as` construction; record 71) against
   LazyCells.
   - *Workers:* W ∈ {1, 4, 7, 8, 10} on the i9, at the 2 s settle.
   - *Per-decode counters:* cycles, instructions, `mem_load_retired.l2_miss`, `l3_hit` and `l3_miss`,
     `cycle_activity.stalls_l2_miss` and `stalls_l3_miss`, and IMC bytes.
   - *Predicted by my reading:* the cursor's L2-miss stalls per byte exceed lazy's and grow faster with
     W, so the wall ratio *rises* with W. That is the opposite of opend.
   - *Refuted if* the ratio falls with W, or if its L2-miss stall cycles per byte don't exceed lazy's.

**What this decides:** nothing in production. The mathematics is exact and the cursor is a correct,
input-only decoder for full-period blocks with a present anchor. Whether to time it is the
Coordinator's call.
