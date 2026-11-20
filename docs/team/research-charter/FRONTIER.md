# The research frontier

*A living record. Research sessions read it at the start of every cycle and update it at the end
(METHOD.md §2, Learn). It holds:*
- *the questions that challenge the current primitives;*
- *the obstructions already known;*
- *the connections not yet explored;*
- *a log of cycles.*

*Evidence lives in [CONTEXT.md](CONTEXT.md) and the journals. This file points to it.*

## Foundational questions (track 1)

Each questions whether a current core operation is necessary. None is settled.

1. **Must the decoder follow an orbit at all?**
   - Every production kernel ends by emitting in LF-orbit order. Rows are visited one after
     another, or retained rows are walked one after another.
   - Is there a formulation in which output positions are computed rather than reached? For
     example, a potential or ranking function on intervals that places whole runs of output
     directly.
   - **Known obstruction:** text-order run labels determine the text (record 84). Computing them
     *is* emission, so any "placement" must be cheaper than the labels it implies.
2. **Is the first return the right object?**
   - Open computes returns to a retained *set* of rows. Repeated cuts shrink that set fast:
     110,273 → 17,657 → 3,916 rows on block 0.
   - What is the limit object of repeated cutting? Can it be computed directly from the band
     structure, without materialising the intermediate levels?
   - See OPEN-QUESTIONS.md §1 for the feasibility version.
3. **Can geometry replace rows?**
   - Rauzy–Veech propagates information through interval endpoints; emission still touches
     every retained row.
   - Is there an exact reconstruction whose work scales with the number of *structural events*
     (run boundaries, overlaps, bands) and whose only n-scale work is sequential byte writing?
   - **Known obstructions:**
     - copy replay shatters into 1.8-byte fragments (round 5);
     - the in-place Φ schedule needs run labels (record 84);
     - the forest construction was 15× slower than Cells (Astra investigation 3).
4. **Is the inverse BWT the right problem?**
   - The decoder must produce bytes, not a permutation.
   - Could information already in the compressed stream let the decoder skip parts of the
     inversion? Examples: the MTF/Huffman symbols, the RLE2 structure, or the block's relation
     to its neighbours.
   - Not yet examined by anyone.
5. **Copies from completed output.**
   - Astra C: the marked output can supply its own sources.
   - What is the cheapest possible discovery of "which rows spell bytes already written"?

## Known structural obstructions

Do not rediscover these; build on them.

| Obstruction | What it rules out | Evidence |
|---|---|---|
| Accelerated induction floor r; Cells at 1.013–1.019r | Better monotone induction *schedules* | records 79–80 |
| Run labels determine the text | Any plan that needs text-order run labels before emitting | record 84 |
| Direct Φ-copy fragments are cyclic (85–94% on dependency cycles) | In-place write-once Φ replay at F_self_ns | round 6, attainability |
| Building precomputed jumps per capped record costs about one Open | NSPC-style per-record contraction | #652 |
| The expansion to one entry per retained row is required, and cheapest in a sequential fill | Moving or compressing Open's queue expansion | #654 |
| The CPU already issues the next edge load early | Software-pipelining Open's walk | #656 |
| LIMIT=16 is best; per-block choice gains ≤ 1% even with hindsight | Thin/thick threshold tuning | #657 |
| Histogram non-identifiability: same run histogram, Θ(n)-different spelling cost | Predicting construction cost from run statistics alone | Neon (unverified by MONODROMY) |
| **Metadata-density obstruction:** 63–75% of Wikidata runs have width 1; any full-block IET/grammar machine (`Cells`, `Forest`, `Bipolar`/`Half-Bipolar`) holds 40–100 B/run across `r` runs (4–9 MB > 2 MiB LLC), paying 3.4–11× Open's DRAM B/B and 6.3–9.1× Open's L2 misses | Putting all `35k–97k` width-1 runs into a multi-field IET/grammar contraction | Neon Cycle 1 (`g5_falsifier.c`, 2026-11-20) |
| **No-widening of Open's `g`:** every piece of Open's `g = P ∘ LF_{L[S]}` with constant literal byte sits inside thin runs (mean width 1.6–2.2, all ≤ 16) | Width-based second Open cuts replicating Astra 3's forest cascade | Neon Cycle 1 (2026-11-20) |
| **Heavy-tailed thin excursions:** 80–86% of Open's `m` rows are thin→thin, with 58–78% of thin rows in excursions of length 8–102 (`p99 = 41–43`) | Inline-`u64` sorted-wavefront switch contraction on Open's thin rows | Neon Cycle 1 (`verify_monotone_and_triage.py`) |
| **Incompressible residual & permutation-entropy barrier:** arbitrary incompressible BWTs of length `m` embed into Wikidata-regime thin residuals; explicit bijection tables on `m = 204k` have entropy `log2(m!) ≈ 410 KiB > 256 KiB` L2, and `B`-slab message passing needs `K ≥ 32k` walkers just to break even on L2m | L2-resident bit-packed explicit `edges`, online SIMD rank on `v = L[S]`, and slab-bucketed multi-walker message passing | Neon Cycle 2 (`2026-11-20-neon-cycle2-backward-open.md`) |
| **L1-capped Monotone Recompression before Open:** 1 round (`|Σ_R| ≤ 27`) deletes 20% of `n` but cuts Open's `m` by only 14–17% while inflating `Σ` to `394–507` (breaking the 1-byte literal fast path on 83% of steps) | Prefixing Open with 1–6 rounds of monotone alphabet-cut BWT recompression | Neon Cycle 2 (`2026-11-20-neon-cycle2-backward-open.md`) |
| **LLC-resident streaming-exchange obstruction (Obligation #1):** Open's `edges[0..m]` (`360–817 KiB`) fits inside a `2 MiB` LLC (`~0.50–0.61m` L2 misses, mostly LLC hits). Any streaming/bucket-elimination replacement moving `C_B > 40 B/row` (read+write) incurs `> C_B/64 · m` L2 line transfers and spills the `2 MiB` LLC into DRAM. *Witness:* `BSSE-16K-slab` (`g5_bsse.c`, `1.27–1.66×` L2m, `1.69–2.51×` DRAM B/B, `1.76–2.08×` VM C time). *Scope:* multi-pass streaming, radix-bucketed, or external-memory list-ranking / Schur-elimination on `m` retained rows when `4m + U < LLC`. *Escape condition:* shrink `m` before building an `m`-scale map without per-run metadata density, OR break serial depth `D = m` in-place inside the `4m`-byte LLC-resident table without `> 2 MiB` queues or an extra `n`-byte copy, OR operate in `4m >> LLC`. | Multi-pass external-memory / Sibeyn-style bucketed Schur elimination replacing Open's walk on `m <= 250k` | Neon Cycle 3 (`2026-11-20-neon-cycle3-bucketed-schur.md`, `g5_bsse.c`) |
| **Band shattering of Open's `g` & `g_tilde` (`OPEN-QUESTIONS.md` §1):** Although `90.4–96.2%` of `m` lies in wide `P`-bands (`w > 16`, `B = 2,020–5,444`), composing `P` with `LF_v` in either order shatters `g` and `g_tilde` into `47,544–126,226` pieces (mean width `1.62–2.22`). *Witness:* `verify_obligations_3_5.py` on all 6 blocks. *Scope:* any `O(B)` second Open cut operating on `g` or `g_tilde` via `P`-bands. *Escape condition:* cut before composing with `LF_v` (or abandon the `LIMIT = 16` thin/thick partition for a spanning-forest/non-width section built without Astra 3's 15× overhead). | `O(B)` second-level Open cut on Open's band exchange | Neon Cycle 3 (`verify_obligations_3_5.py`) |
| **Raw `Φ`-copy cyclicity & ternary non-uniqueness (Obligation #3, `OPEN-QUESTIONS.md` §2):** `0.21–0.25r` raw `Φ`-copy intervals and `0.22–0.30r` overlaps are `O(r log r)` from `(SA[H], SA[D])` without run labels, and do *not* determine ternary texts (witnesses at `n = 4, 5`). However, `71.5–79.1%` of raw copies are trapped in mutual dependency cycles across all 6 blocks. *Witness:* `verify_obligations_3_5.py` on all 6 blocks and ternary `n <= 6`. *Scope:* acyclic in-place `Φ`-copy scheduling from `(SA[H], SA[D])` without run labels or peeling. *Escape condition:* compute an acyclic depth partition of raw `Φ`-copies without paying Record 84's run labels or Round 5's `3.4–8.7r` peeling shatter, AND acquire `(SA[H], SA[D])` cheaper than Cells' `1.68–1.87r` RV. | Direct acyclic `Φ`-overlap replay from `(SA[H], SA[D])` alone | Neon Cycle 3 (`verify_obligations_3_5.py`) |
| **Pre-column causal barrier (Obligation #5):** Bzip2 headers omit `C[0..256]`; before `EOB`, arbitrary suffix continuations shift `LF(i)` for every `L[i] > min(Σ)`. MTF locality is modest (`MTF == 1` covers `13.6–15.7%` of run heads; `MTF <= 3` covers `28.9–31.2%`), and `src/runs.rs` already decodes MTF+RLE2 directly to capped runs in AVX2. *Witness:* two bzip2 streams sharing any pre-`EOB` prefix with different trailing `RUNA` counts. *Scope:* all exact decoders of standard unmodified bzip2 streams. *Escape condition:* format change storing `C[0..256]`/order hints (out of scope), or traction fusion of `runs.rs` MTF directly into Open's run pass (eliminating `runs` buffer + `CAP=127` re-coalesce, not BWT inversion). | Skipping global BWT inversion using pre-`EOB` MTF/Huffman/RLE2 prefix information | Neon Cycle 3 (`2026-11-20-neon-cycle3-bucketed-schur.md`, `verify_obligations_3_5.py`) |

## Obligation register learnings (aligned with `origin/master` `f129af32`)

- **Obligation #1 (Dependent traversal of retained rows):** Cycle 3 proved & verified `BSSE` (`97,976` small cases + 6 blocks), eliminating all `> L2` random accesses (`Q_{>L2} = 0`), then falsified it (`1.27–1.66×` L2m, `1.69–2.51×` DRAM, `1.76–2.08×` VM C time), establishing the **LLC-Resident Streaming-Exchange Obstruction** (`C_B > 40 B/row` loses to an LLC-resident `4 B/row` walk). Also closed `OPEN-QUESTIONS.md` §1 (`P`-bands shatter to `47.5k–126.2k` pieces under `LF_v`).
- **Obligation #2 (Building the retained map):** VM uninstrumented `-O3` C diagnostic timing (`g5_bsse.c -DNO_CACHE_SIM`) calibrated Cycle 2's Backward Open against Shipped Forward Open: Backward Open ran `1.02–1.09×` Open despite winning on every 3-level cache metric (`0.82–0.88×` L2m, `0.72–0.84×` DRAM), because Forward Open's `wir` is already only `0.09–0.20 ms` (`5–7%`) while Backward Open's second record scan + branchy coalescing + `active[c]` lookup cost `0.59–1.21 ms` (meanwhile descending `emit` was `0.91–1.02×`, NOT slower).
- **Obligation #3 (Acquiring `Φ`-overlap coordinates):** Positive: `0.21–0.25r` raw copies & `0.22–0.30r` overlaps are `O(r log r)` from `(SA[H], SA[D])` without run labels, and do not determine ternary texts. Obstruction: `71.5–79.1%` cyclic on all 6 blocks.
- **Obligation #5 (Starting from the decoded column):** Moved from `unexplored` to bounded by the **Pre-Column Causal Barrier** (`C[0..256]` absent until `EOB`; `MTF == 1` only `13.6–15.7%` of run heads).

## Connections not yet explored

These are pointers, not results:
- Kempa (SODA 2019) and Agol–Hass–Thurston: sublinear routes to run-boundary samples. Could an
  o(n) sample construction feed a reconstruction?
- LCP-chain structure (Kärkkäinen–Kempa–Piątkowski): reducible and irreducible LCPs at run heads
  (deep-research-gap-chains).
- Stochastic complementation (Meyer 1989), the matrix form of Open's elimination: are there other
  complements, with other retained sets, with cheaper structure?
- Derived sequences and return substitutions for the finite marked BWT (Durand), applied to the
  repeated-cut limit.
- **Native evaluation of Queue-Free Backward Open (`g = P ∘ LF_{L[S]}`):** VM C diagnostic in Cycle 3 showed `BwdOpen` at `1.02–1.09×` Open (due to `wir` re-scanning capped records vs Forward's `0.09–0.20 ms` tight queue-read loop), while descending `emit` was `0.91–1.02×` Open. Still needs i9 / Rust verification if pursued.

## Cycle log

Append one entry per research cycle: the date, the session, what was explored, what was killed and
why, and what was promoted.

| Date | Session | Explored | Killed (why) | Promoted |
|---|---|---|---|---|
| 2026-11-19–20 | Neon rounds 1–7 and follow-ups; the Coordinator | parallel induction; boundary samples; copy replay; NSPC; DOE; Open micro-optimisations; LIMIT | see CONTEXT.md §4 | none to production; Open shipped (0.5.7) from the Astra line |
| 2026-11-20 | Neon Cycle 1 (`research/neon-bipolar-contraction`) | displacement-rank resolvent; Φ overlap transitions; second width cut on Open; thin-switch wavefront; Monotone Alphabet-Cut BWT Recompression; Bipolar & Quotient-Accelerated Zero-Sort Half-Bipolar Contraction | resolvent (Θ(rn)/Ω(n log n)); Φ overlaps (already pays Cells RV); second width cut (no-widening); thin wavefront (heavy tail len 8–102); Monotone Recomp (|Σ_R| blowup to 14k); Half-Bipolar failed G5 (6.3–9.1× L2m, 3.4–11× DRAM B/B vs Open due to Metadata-Density Obstruction) | Track 1 proved & verified results: Monotone BWT Recompression + Bipolar/Half-Bipolar (5,227 small + 6 blocks byte-exact; D collapsed 1000–2000× to 88–100); G5 diagnosis redirects Cycle 2 to L2-resident / low-metadata inversion |
| 2026-11-20 | Neon Cycle 2 (`research/neon-bipolar-contraction`) | Open phase L2/DRAM profile; Incompressible Residual Embedding; L1-capped Monotone Recomp + Open; 18-bit packed `edges`; direct SIMD rank on L2-resident `v = L[S]`; slab-bucketed multi-walker message passing; **Queue-Free Backward Open (`g = P ∘ LF_{L[S]}`)** | L1-capped Monotone + Open (only -15% `m`, breaks 1B literal fast path); 18-bit `edges` (`log2(m!) ≈ 410 KiB > L2`); online SIMD rank (2–3 dep loads + scans/step); slab-bucketed walkers (`K ≥ 32k` I/O bound) | **Queue-Free Backward Open** verified exact (`20,908` small cases + 6 blocks), G2 PASS, G5 cuts `(run+wir)` L2m `0.48–0.52×`, DRAM B/B `0.72–0.84×`, work `0.72–0.74×`, peak memory `0.75–0.84×` (borderline on strict 6/6 cutoff on block 0; recommended for Coordinator native i9 A/B test) |
| 2026-11-20 | Neon Cycle 3 (`research/neon-bipolar-contraction`, Mode: `reorientation`) | **Bucketed Sequential Schur Elimination + L2-Slabbed Emission (BSSE)** on Open's `g`; 1-wave directed-cut symbol elimination; band-level 2nd Open cut (`OPEN-QUESTIONS.md` §1); direct `(SA[H], SA[D]) -> Φ`-overlap graph (Obligation #3); pre-column bzip2 MTF/RLE2 pipeline (Obligation #5); multi-courier parallel Open walk | 1-wave SymCut (only `27–28%` cut, doubles edge width); Band-level 2nd Open cut (`P`-bands shatter to `47.5k–126.2k` pieces under `LF_v`); raw `Φ`-overlaps (`71.5–79.1%` cyclic); pre-column prefix skip (Causal Barrier: `C[0..256]` absent until `EOB`); **BSSE** failed G5 (`1.27–1.66×` L2m, `1.69–2.51×` DRAM B/B, `1.91–1.99×` work, `1.76–2.08×` VM C time) | Track 1 proved & verified results: BSSE Conservation + Exact Chain-Head Decomposition (`97,976` small cases + 6 blocks byte-exact; `Q_{>L2} = 0`), **LLC-Resident Streaming-Exchange Obstruction** on Obligation #1, Obligations #3 & #5 bounds, and VM C timing calibration of Cycle 2 Backward Open (`1.02–1.09×` Open due to `wir` record re-scan) |

