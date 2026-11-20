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
- **L2-resident compact representations of Open's residual `g = P ∘ LF_{L[S]}`:** the entire thin
  sub-column `v = L[S]` is only 88–199 KiB (fits inside 256 KiB L2!), yet Open expands it to two
  360–817 KiB `u32` arrays (`queues` + `edges`). Can `g` be inverted directly inside L2 without
  `queues` or `edges`, or via a different low-metadata algebraic/combinatorial representation?

## Cycle log

Append one entry per research cycle: the date, the session, what was explored, what was killed and
why, and what was promoted.

| Date | Session | Explored | Killed (why) | Promoted |
|---|---|---|---|---|
| 2026-11-19–20 | Neon rounds 1–7 and follow-ups; the Coordinator | parallel induction; boundary samples; copy replay; NSPC; DOE; Open micro-optimisations; LIMIT | see CONTEXT.md §4 | none to production; Open shipped (0.5.7) from the Astra line |
| 2026-11-20 | Neon Cycle 1 (`research/neon-bipolar-contraction`) | displacement-rank resolvent; Φ overlap transitions; second width cut on Open; thin-switch wavefront; Monotone Alphabet-Cut BWT Recompression; Bipolar & Quotient-Accelerated Zero-Sort Half-Bipolar Contraction | resolvent (Θ(rn)/Ω(n log n)); Φ overlaps (already pays Cells RV); second width cut (no-widening); thin wavefront (heavy tail len 8–102); Monotone Recomp (|Σ_R| blowup to 14k); Half-Bipolar failed G5 (6.3–9.1× L2m, 3.4–11× DRAM B/B vs Open due to Metadata-Density Obstruction) | Track 1 proved & verified results: Monotone BWT Recompression + Bipolar/Half-Bipolar (5,227 small + 6 blocks byte-exact; D collapsed 1000–2000× to 88–100); G5 diagnosis redirects Cycle 2 to L2-resident / low-metadata inversion |
