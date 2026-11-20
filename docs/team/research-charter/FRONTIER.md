# The research frontier

*A living record. Research sessions read it at the start of every cycle and update it at the end
(METHOD.md §2, Learn). It holds:*
- *the questions that challenge the current primitives;*
- *the obstructions already known;*
- *the connections not yet explored;*
- *a log of cycles.*

*Evidence lives in [CONTEXT.md](CONTEXT.md) and the journals. This file points to it.*

## Pending mathematical leads

This is an inbox for directions Louis introduces between cycles. A lead is an invitation, not an
assignment: Neon reads it at the next cycle boundary and decides how it should shape the next
cycle. Following one needs no approval. Neon updates the status.

| Mathematical doorway | Why it might expose a new representation or primitive | References | Status |
|---|---|---|---|
| **The origami lift:** a BWT block as a branched torus cover with monodromy (LF, +1) | Surface operations, and directions with a transport law, unavailable in the one-dimensional exchange. MONODROMY verified the construction (record 86); the cone points alone cannot decode | [journal](../../journal/2026-11-20-origami-lift.md); [Carbon cycle 1](../../journal/2026-11-20-carbon-cycle1-word-transport.md) | examined (Carbon cycle 1): Out(F₂) global frames cannot reduce piece count below r−\|X\| or step count below 1/row (Thm 1.1); ⟨LF,+1⟩ is primitive on 6/6 blocks so no intermediate covers exist (Thm 1.2) |
| **Illegal turns and folding** (Bestvina–Handel, Stallings) | If a graph self-map of the run dynamics can be built cheaply, its illegal-turn structure might isolate the few branching decisions that carry the ordering, resolved collectively by folds. A self-map must be constructed first; without one, "illegal turn" is undefined | Bestvina–Handel (Ann. Math. 1992); Feighn–Handel; Rafiqi–Zeytin, arXiv:2610.13347; [Carbon cycle 1](../../journal/2026-11-20-carbon-cycle1-word-transport.md) | examined (Carbon cycle 1): any LF-invariant cellular subdivision V ⊇ D of a primitive block has V = {0..n−1} (zero illegal turns, Thm 3.1); homotopy to D loses offsets (`cbaa` vs `cbbaa`); sub-interval return recovers RV |
| **Ribbon graphs and rotation systems** | Restoring the cyclic order at each vertex encodes a surface embedding. Faces are cycles of σ∘α. Could ordered paths be assembled collectively from the embedding? Caution: plain face traversal may just be LF in disguise | Standard combinatorial-map literature; [Carbon cycle 1](../../journal/2026-11-20-carbon-cycle1-word-transport.md) | examined (Carbon cycle 1): unweighted (V,E,σ,α,C,p_run) fails identifiability (`aaabaa$` vs `aabaaa$`); restoring arc widths yields the measured train track (splitting = RV) |
| **Splitting integer-weighted train tracks, and contraction** | Dunfield–Thurston count the components of weighted multicurves by splitting bands, which they identify with Rauzy induction, without tracing the curves. Is there an exact analogue that yields the marked ordered itinerary? Also: parallel list and tree contraction carrying ordered values | Dunfield–Thurston (2006), §7.3; Shun, Gu, Blelloch, Fineman and Gibbons; Cong–Bader; [Carbon cycle 1](../../journal/2026-11-20-carbon-cycle1-word-transport.md); [Carbon cycle 2](../../journal/2026-11-20-carbon-cycle2-bvsmc.md) | candidate ready for Coordinator audit (Carbon cycle 2): `BVSMC-1P` (`0.627–0.781×` Open VM total, `0.550–0.778×` `emit`) & zero-arena `BVSMC-2P` (`0 B` pool, `0.699–0.818×` Open VM total) break the ROB branch-flush obstruction |
| **Agol–Hass–Thurston interval-pairing reduction** | Its input, integer intervals paired by translation, matches a block's runs, and the algorithm resolves global structure without walking points. It computes component counts, not order. Can a word-carrying version recover the marked orbit more cheaply than Cells or Open? | Agol, Hass and Thurston (2006); MONODROMY [AHT steelman](../../journal/monodromy/2026-11-14-aht-steelman/reading-notes.md); [Carbon cycle 1](../../journal/2026-11-20-carbon-cycle1-word-transport.md) | examined (MONODROMY 2026-11-14 + Carbon cycle 1): gcd-merger/ℤᵈ-sums lose order; word-carrying transmissions save 2/79,349 steps (0.0025%) on Open's thick complement (`endpoint.rs`) |

*These references come from Louis's literature search and were checked against sources in `2026-11-14-aht-steelman` and Carbon cycle 1.*

## Obligation register

These are the expensive classes of work that current decoders pay. Candidates come and go; an
obligation stays open until it is removed or bounded with a proof. Update a row only when a cycle
has learned something concrete (METHOD.md §3a).

When a failure exposes a significant obstruction, record three things under its row:
- the concrete **witness**;
- its exact **scope**: which algorithms, representations and cost model it binds;
- the **escape condition**: which assumption a different formulation would have to drop.

An obstruction with no stated scope is not established. Not every failure yields one.

| Obligation | Evidence | Open question | Status |
|---|---|---|---|
| Dependent traversal of retained rows | Open: emission 36–48% of time (#653). Carbon cycle 1 exposed the **ROB branch-flush obstruction** on branched 8-lane walks (`g5_ssmc.c`). Carbon cycle 2 (`g5_bvsmc.c`) broke it via uniform-16 SIMD prefix splicing: `BVSMC-1P(8,48)` achieves `0.550–0.778×` `emit` (`0.627–0.781×` VM total; `0.748–0.905×` vs strengthened 1-lane `Open+U16Q`), and zero-arena `BVSMC-2P(8,64)` (`0 B` pool) achieves `0.611–0.877×` `emit` (`0.699–0.818×` VM total). Sub-cutoff `emit <= 0.62×` held on 4/6 higher-`m/n` blocks, bounded on `block-0` (`0.778×`, `m=0.10n`) by `810 KB` of `>16B` thick-excursion `memcpy`s | Does `BVSMC-2P` (`0 B` pool) or `BVSMC-1P` (`1.23 MiB` pool) win natively on `i9-10900K` across 1–20 threads? | candidate ready for native check (Carbon cycle 2) |
| Building the retained map | Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). `#659` (`Backward Open`) failed (`1.10–1.15×`). Carbon cycle 2 removed `#659`'s second `records` scan via a `2 B/run` `thin_pieces` stream + AVX2 sequential fill (`0.739–0.770×` `qi16` construct), then showed in a fairness control that giving `qi16` the same fixed-16 AVX2 iota push (`U16Q`, `store_seq16`) achieves `0.718–0.763×` `qi16` construct: `queues[256]` is an L1-resident radix coalescer (`20×` fewer outer loop transitions than thin-run order), and `24–28%` of `construct` was variable-`w` (`1..16`) loop-exit mispredictions | Does `U16Q` (`store_seq16` padded queue fill, `0 B` extra $n$-scale memory) transfer natively to `src/open.rs`? | positive finding (`U16Q`); open for deeper elimination |
| Acquiring the Φ-overlap coordinates | The overlap structure is small: 1.25–1.37r transitions on the six blocks (Neon cycle 1). The known route via run-boundary positions costs Cells' 1.68–1.87r steps (record 81) | Can those coordinates come out of a cheaper computation, or out of an earlier representation? | open; positive measurement |
| Full-block metadata | Grammar and interval-exchange machines carry 40–100 B per run across all r runs, mostly width-1 runs (Neon cycle 1; Astra D) | Can a primitive avoid per-run global state? | open |
| Starting from the decoded column | Every kernel begins from run records after Huffman, MTF and RLE2 decoding | Does an earlier representation carry ordering information more cheaply? | unexplored |
| Writing every output byte | Unavoidable; sets the floor | (bounds the others) | floor |

## Steering log

Each entry records a change of direction, written before the cycle it governs, and its observed
effect afterwards. Louis owns the decision at the bottom of each entry: hold heading, correct, or
reorient.

| Applied | Target | Intervention | Intended response | Observed response | Decision |
|---|---|---|---|---|---|
| f5961bfb, for Neon cycle 3 | The search kept returning to classical permutation inversion: induced sorting, pair recompression, the direction duality (audit, branch `audit/neon-cycles-1-2`) | Make computational obligations the unit of research; reorientation by default; the obligation register; the bzip2 pipeline before the decoded column made admissible | Investigations that replace ordering operations collectively, rather than reorganise them | pending cycle 3 | pending |
| 6d9acc87, for Neon cycle 3 onward | The obligation register was being used to generate ideas as well as judge them; agents self-classified; cost models were uncalibrated (0 of 6 promoted candidates passed natively) | Generation from mathematical structure, with obligations kept for evaluation; classification by the auditors; cost models treated as diagnostic; witness, scope and escape condition recorded for obstructions. Methodology frozen until evidence arrives | Mechanisms from structure, not from rephrased cost lists | pending | pending |

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
   - Examined in Carbon cycle 3 (`triage_cycle3.py`): `queues[a]` shatters `2.4k–6.7k` bands into `47.5k–125.8k` affine pieces of $g^{-1}$ (`max_w = 24..33`, only `10..45` pieces have `w > 16`), and cutting all $w \ge 2$ pieces lands within `0.15–0.50%` of single-level `Open<LIMIT=1>` while paying both constructions.
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
   - Closed in Carbon cycle 3 (`g5_smcc.c`): even an uncharged omniscient oracle (`SMCC-Oracle`) skipping `36.2–53.0%` of `Open::emit` pointer steps runs at `0.931–1.021×` `Open` total (`1.035–1.062×` slower `emit` on 3/6 blocks) because random read misses across `880 KiB` `out[0..pos)` + 3-way dispatch cancel the skipped `edges[]` loads.

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
| Computing LF from a compact rank structure instead of storing the edges costs about four chained dependent loads per step against Open's one | Implicit-rank (FM-index-style) successors as a replacement for Open's explicit edges, single-threaded. This is the known medium-space inverse BWT trade (Lauther–Lukovszki 2005; Kärkkäinen–Puglisi, ESA 2010) | Neon cycle 5, `research/neon-bipolar-contraction` 4598c5bf: BSO-512 2.48–2.80× Open in its C harness; the Coordinator reproduced 1.71–2.44× on another machine. Its memory and DRAM savings (0.36–0.75×) don't pay for it |
| Histogram non-identifiability: same run histogram, Θ(n)-different spelling cost | Predicting construction cost from run statistics alone | Neon (unverified by MONODROMY) |
| ROB branch-flush on multi-cursor symbolic walks: per-step literal-vs-phrase branching (`14–23%` phrases) across 8 interleaved lanes flushes in-flight L2/L3 loads (`0.075–0.202 ms` walk-only vs `0.655–1.668 ms` branched `SSMC`) | Branching multi-lane return-map walkers (`SSMC-Open`, `segments.rs`) | Carbon cycle 1, `g5_ssmc.c` (unverified by MONODROMY) |
| Output-copy source-footprint & 3-way dispatch ceiling: even an uncharged omniscient oracle (`SMCC-Oracle`) skipping `36.2–53.0%` of `Open::emit` steps via `out[src_pos..]` runs at `0.805–1.062×` `emit` (`0.931–1.021×` Open total; slower on 3/6 blocks) | Output-sourced copy decoders over `Open`'s retained walk (`Astra C`, `OPEN-QUESTIONS.md` §4, `SMCC`, `working-backwards.md` Q2) | Carbon cycle 3, `g5_smcc.c` (unverified by MONODROMY) |

## Connections not yet explored

These are pointers, not results:
- **The run network** ([journal](../../journal/2026-11-20-run-network.md)): LF as a graph on runs, built in O(r).
  - Its cycle rank, 0.21–0.25r, is fixed by record 63's identity E = 2r − 1 − C, and equals the copy count k plus one.
  - 88–89% of runs are pass-throughs, leaving a series-reduced core of about 0.11r runs.
  - With order-preserving switches it is a measured train track.
  - Census (bd781368): the core has no small separators, though it bisects 4–8× better than random. As a retained set for first returns it is worse than Open's.
  - Open question: is there an operation on the core that is not a first-return walk?
- **The origami lift** ([journal](../../journal/2026-11-20-origami-lift.md)): a BWT block is a branched cover of the torus with monodromy (LF, +1). MONODROMY verified it (record 86). Carbon cycle 1 proved Out(F₂) global-frame piece rigidity and monodromy primitivity on all 6 blocks (unverified by MONODROMY).
- Kempa (SODA 2019) and Agol–Hass–Thurston: sublinear routes to run-boundary samples. Could an
  o(n) sample construction feed a reconstruction?
- LCP-chain structure (Kärkkäinen–Kempa–Piątkowski): reducible and irreducible LCPs at run heads
  (deep-research-gap-chains). Examined in Carbon cycle 2: `r − |D̄∩C̄| = 10,954–26,964` (`20.8–24.9%` of `r`), reducing under even a free `(SA, LCP)` oracle to Round 6's cyclic Φ-copy graph (`85–94%` cyclic).
- Stochastic complementation (Meyer 1989), the matrix form of Open's elimination: Carbon cycle 2 proved width-thresholding (`w > LIMIT`) strictly minimises retained row count `m` among all run-union complements of size `H` (`3.2–4.9×` fewer rows than byte-class complements on all 6 blocks). Non-run-union complements remain open.
- Derived sequences and return substitutions for the finite marked BWT (Durand), applied to the
  repeated-cut limit: Carbon cycle 2 verified (`873` columns) that `k`-step Wheeler Myhill–Nerode equivalence equals Record 84's refinement tower (`n` states at closure on primitive blocks), while nested interval return morphisms reduce to `Cells` or `Astra Inv. 3`.

## Cycle log

Append one entry per research cycle: the date, the session, what was explored, what was killed and
why, and what was promoted.

| Date | Session | Explored | Killed (why) | Promoted |
|---|---|---|---|---|
| 2026-11-19–20 | Neon rounds 1–7 and follow-ups; the Coordinator | parallel induction; boundary samples; copy replay; NSPC; DOE; Open micro-optimisations; LIMIT | see CONTEXT.md §4 | none to production; Open shipped (0.5.7) from the Astra line |
| 2026-11-20 | Neon cycles 3–5 (branch `research/neon-bipolar-contraction`, 4598c5bf; not yet audited beyond BSO's timing) | bucketed Schur; unsplit forest; output-sourced anchors; bit-sliced implicit Open (BSO) | anchors (width-1 runs collapse interval certificates); BSO (serial rank latency, above) | none; Neon paused |
| 2026-11-20 | Carbon cycle 1 (branch `research/carbon-word-transport`, 3337daac) | Out(F₂) origami transport & covers; ribbon-graph faces; BH graph self-maps; AHT spiral transmission on Open; k-gram Ψ buckets; sentinel-spliced monoid contraction (`SSMC-Open`) | Out(F₂) (piece rigidity + primitive monodromy); ribbon faces (`aaabaa$` vs `aabaaa$`); BH cellular maps (\|V\|=n on primitive blocks); AHT on Open (2/79,349 steps); k-gram Ψ (scattered codomain); branched `SSMC-Open` (G5 FAIL: ROB branch-flush kills 8-lane MLP) | none; exposed ROB branch-flush obstruction & 92.5–94.4% ≤16B word concentration for Cycle 2 |
| 2026-11-20 | Carbon cycle 2 (branch `research/carbon-word-transport`, 8b54b024) | non-width Meyer complements; Wheeler Myhill–Nerode & Durand hierarchy; KKP irreducible bridges; `thin_pieces` backward radix granularity vs `U16Q`; `BVSMC-1P` (`1.23 MiB` pool) & zero-arena `BVSMC-2P` (`0 B` pool) | byte-class Meyer (`3.2–4.9×` more rows than width cut); Wheeler MN (== Record 84 tower); KKP bridges (`85–94%` cyclic Φ-copies); queue-free `thin_pieces` (`U16Q` matches `0.72–0.76×` construct via L1 radix coalescing) | `BVSMC-1P` (`0.627–0.781×` Open VM total; Coordinator rerun `0.66–0.78×` on 5/6) & `BVSMC-2P` (`0 B` pool) + `U16Q` handed to Coordinator for native Rust validation |
| 2026-11-20 | Carbon cycle 3 (branch `research/carbon-word-transport`) | recursive band cut on $g^{-1}$; UUID-v4 shared-format surrogate conjecture; `LIMIT -> 1` deduplicated dictionary scaling; `SMCC-Paid` (same-cache-line lockstep self-modifying Open) & `SMCC-Oracle` | recursive band cut (`47.5k–125.8k` scattered pieces, matches `Open<1>`); UUID-v4 format (`+3.2` isolated singletons/UUID); `SMCC-Paid` (`1.13–1.25×` Open) & `SMCC-Oracle` (`0.931–1.021×` Open even with free `36–53%` step skipping: random `out[]` L2/L3 reads + 3-way dispatch) | none; closed `OPEN-QUESTIONS.md` §1 & §4 and `working-backwards.md` Q2 with exact oracle ceiling |
