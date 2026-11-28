# 2026-11-23 · MONODROMY 92 · A finite exact hierarchy for the origami lift. Refinement is obstructed: a proved bound of one transport operation per run, confronted with six blocks, leaves Open at the priced optimum of its own family. Composition escapes the bound, but its one level that pays is the super-alphabet on Open's trace, a known object. Its cheapest falsifier is named

**The request.** The Coordinator's CHECKPOINT 7089327497 relays Louis's question of 23 November, prompted by Hévéa and the Nash–Kuiper process. It asks for a finite exact hierarchy S₀ → ⋯ → S_k of (LF, u). In it:
- each level should reduce the dependent walk;
- the structure each level needs should be cheap to get from the runs;
- the marked orbit should transport back without a global expansion;
- C_build + C_transport + C_emit should beat Open.

It asks for (a) a scoped primitive or (b) a sharp obstruction.

**Louis's note on this item** (this session, 23 November) moved the target, and this record follows it.
- **First,** prove a scoped theorem for exact first-return hierarchies whose descendants cannot outgrow their parent's width, and confront it with the six blocks.
- **Then** keep an explicit escape clause, and test composition rather than refinement.
- **The test:** do exact transport summaries that compose across thin strips need expensive global permutation information? Or can they be built cheaply from run records?

**His correction is adopted.**
- My first width figures were a census of the *input* thin runs, not of the geometry after Open's induction. §2 now measures the residual geometry itself.
- "2/w" was a heuristic, not a bound. It is withdrawn, and Theorem 4 replaces it.

**Labels:** **[proved]** · **[measured]** (exact counts on the six blocks, or timings where stated) · **[conjectured]** (including **[modelled]** cost arithmetic on other people's measured prices).

**Inputs.** The six authenticated columns of `data/admitted-bwt` (blocks 0, 2, 85, 143, 207, 210, with their primaries). The scripts and their outputs sit beside this record in [`2026-11-23-refinement-and-composition/`](2026-11-23-refinement-and-composition/).

**Validation of the counter:**
- Its τ = 0 case reproduces Cells' **92,571** right-end steps on block 0 exactly (record 80).
- Its τ = 16 case reproduces Open's **89,874** retained rows and **3,448** thick strips (`2026-11-18-astra-investigation-2.md`).

## 1. The model, and why a hierarchy collapses to one level

**Model R (exact first-return refinement):**
- **The levels:** nested retained sets [n] = S₀ ⊇ S₁ ⊇ ⋯ ⊇ S_k ∋ p. Level i is the first-return map T_i of LF to S_i, with return words w_i(s), the labels emitted from s up to its return.
- **A level's pieces:** its maximal intervals of adjacent rows of S_i with one return word.
- **The decoder** walks T_k from p, one dependent step per visit, and emits the stored words.
- **The construction** is done by *transport operations*. Each one carries one rigid interval of rows through one translation piece. Rauzy–Veech steps, Open's endpoint-induction steps and representative-tracer steps are all of this kind.

**Lemma 1 (collapse) [proved].** The levels add nothing to the walk.
- *The statement:* the first return of T_i to S_{i+1} is T_{i+1}. The return words compose by substitution: w_{i+1}(s) = w_i(s) w_i(T_i s) ⋯ w_i(T_i^{j−1} s), where j is the first time T_i^j s lies in S_{i+1}.
- *The proof:* the marked orbit visits S_{i+1} in the same order whether or not the visits to S_i \ S_{i+1} are watched.
- *So:* any finite hierarchy in model R has the walk and output of the one-level trace onto S_k. Its levels change only how the return words are computed, as a grammar for Open's band dictionary. This is Neon's U_cat (all contraction frameworks are traces) in its multiscale form.

**Lemma 2 (jump-walk universality) [proved].** This covers any exact decoder whose serial step moves from a row to a later row of the marked orbit and emits the stored word in between.
- *The statement:* such a decoder is the trace walk on the set of rows it visits.
- *The proof:* let V be the visited set. Each step lands on the next element of V along the orbit, so on the first return to V.
- *Scope:* it covers §6.2's whole family. A Nielsen image (r, u r⁻¹), or any SL(2, ℤ) direction, read back to the horizontal by jumps, is a trace on its visited set. Moving through the family changes only the construction cost, never the walk.

## 2. Width cannot grow, and the residual geometry measured

**Lemma 3 (width monotonicity; Louis's lemma) [proved].**
- *The basic fact:* adjacent rows x and x + 1 have the same next label iff they lie in one run. Then LF(x + 1) = LF(x) + 1, by stable LF.
- *By induction:* two adjacent rows have equal next-j labels iff they stay adjacent, inside one run, for all j steps.
- *Consequently:*
  - every piece at every level lies inside one run of L at every step of its itinerary;
  - its width is at most the narrowest run it crosses;
  - a descendant piece extends its parent's word, so it is a sub-interval of the parent. **Width never increases down a descendant chain.**
- *When S is a union of whole runs, plus p:*
  - two rows travelling together are in S together, so they return at the same times;
  - so the pieces of T^k refine those of T^{k−1};
  - **pieces(T^k) ≥ pieces(T) ≥ the number of runs that meet S**, for every k.

**The residual geometry after Open's induction [measured].** The pieces of Open's return map T_S (S = runs of width ≤ 16, plus p) against the input thin runs:

| block | thin runs (input) | pieces of T_S | excess | mean width | row-weighted width |
|---|--:|--:|--:|--:|--:|
| 0 | 47,736 | 47,908 | +0.36% | 1.88 | 4.59 |
| 2 | 50,705 | 50,973 | +0.53% | 2.20 | 5.34 |
| 85 | 77,810 | 78,022 | +0.27% | 2.00 | 4.53 |
| 143 | 103,288 | 103,906 | +0.60% | 1.89 | 4.31 |
| 210 | 125,923 | 126,289 | +0.29% | 1.62 | 3.78 |
| 207 | 83,637 | 84,020 | +0.46% | 1.62 | 3.48 |

**What the table shows:**
- *Lemma 3 says* the residual can only be finer than the input.
- *It is finer by 0.27–0.60%.* The input census is the residual geometry to within that margin. That is now measured, not assumed.

**The asymmetry Louis named [measured].**
- *The narrow runs:* 85–93% of the thin runs have at most 3 rows. They hold 50–70% of the thin rows.
- *The rest:* 7–15% of thin runs (5,077–11,375 a block) have 4–16 rows, with a piece-weighted mean of 6.9–7.6. They hold 30–50% of the thin rows.

## 3. The obstruction for refinement

**Theorem 4 [proved].** It holds in model R for any final set S.
- *The count:* D(S) = |S| walk steps + O(S) transport operations, with
  - **O(S) ≥ #{runs R : R ⊄ S}**, and so
  - **D(S) ≥ Σ_{R ⊆ S} w_R + #{R ⊄ S} ≥ r**.
- *For the endpoint induction Open and Cells run:* K(S) ≥ H(S) + s(S). Here H is the number of complement strips and s the number of components of S.
- *The proof:*
  - Every row of a run R ⊄ S lies on the orbit between two visits to S. Its successor must be supplied by some transport operation over an interval containing it. By Lemma 3 that interval lies in one run, so each such run takes at least one operation.
  - For the endpoint form: each elementary step consumes rows from one domain piece. Domain pieces are only ever split. The H + s original domain pieces of the auxiliary exchange must all be consumed.
- *The parameterisation Louis asked for:* moving a set 𝓡 of thin runs from the walk into transport saves Σ w_R walk steps and costs at least |𝓡| operations. So **the walk steps removed per transport operation are at most the piece-weighted mean width of 𝓡, never its row-weighted mean.**

**Confronted with the six blocks** (moving every thin run into transport, from Open's τ = 16 to Cells' τ = 0):

| block | r | thin rows m₁₆ | ceiling: rows per op (piece mean) | measured right-end steps K₀ − K₁₆ | measured steps per thin run (bound ≥ 1) | measured rows per step | share of ceiling |
|---|--:|--:|--:|--:|--:|--:|--:|
| 0 | 51,184 | 89,874 | 1.88 | 81,356 | 1.70 | 1.10 | 59% |
| 2 | 54,532 | 112,070 | 2.21 | 88,236 | 1.74 | 1.27 | 58% |
| 85 | 79,375 | 156,404 | 2.01 | 133,414 | 1.71 | 1.17 | 58% |
| 143 | 108,335 | 196,841 | 1.91 | 172,571 | 1.67 | 1.14 | 60% |
| 210 | 128,732 | 204,281 | 1.62 | 204,629 | 1.63 | 1.00 | 62% |
| 207 | 87,111 | 135,921 | 1.63 | 139,752 | 1.67 | 0.97 | 60% |

**What it shows [measured]:**
- *The thin set as a whole:* transporting the thin rows costs **0.79–1.03 right-end steps per walk step removed**.
- *Against the bound:* right-end induction runs at 1.63–1.74 operations a thin run, against the proved floor of 1. It achieves 58–62% of the ceiling.

**By width class** (marginal right-end steps per walk row removed, from the threshold sweep, [`threshold-sweep.txt`](2026-11-23-refinement-and-composition/threshold-sweep.txt)) [measured]:

| class (rows) | ≤ 1 | 2 | 3 | 4 | 5–8 | 9–16 | 17–32 | 33–64 |
|---|---|---|---|---|---|---|---|---|
| steps per row | 1.04–1.21 | 1.30–1.93 | 0.70–1.36 | 0.54–0.91 | 0.48–0.71 | 0.28–0.42 | 0.12–0.23 | 0.08–0.12 |

**In unit cost the claim "transport costs at least the dependent steps it removes" is false for wide classes [measured].**
- *The curve:* m + K is nearly flat in τ.
- *Its minimum* is at τ = 2–3, at **0.70–0.90 of Open's τ = 16 count** on every block.
- *So:* counting steps alone, an intermediate threshold beats both Cells and Open by 10–30%. This is the "intermediate regularity" of Louis's question: how much order is kept literally. It exists, and it is interior.

**Priced, it is Open [modelled].** Neon's handoff measured:
- a Rauzy–Veech step at **22.2 ns** (`DtWidestOpen` against RV);
- OpenCells' emit at **4.8–5.8 ns a retained row** on the i9.

**What those prices mean:**
- *Break-even:* 0.22–0.26 steps a row, which is 3.8–4.6 rows per operation.
- *The proved part:* runs of at most 3 rows can never pay at that price, whatever the induction. One operation per run (Theorem 4) saves at most 3 rows. These runs are 50–70% of the walk.
- *The measured part:* the 4–8 class costs 0.48–0.71 steps a row, and the 9–16 class 0.28–0.42. Neither pays. The 17–32 class, at 0.12–0.23, does pay to transport.
- *So:* the priced optimum of the refinement family is **τ ≈ 16, Open's own threshold**.
- *The edge:* blocks 0 and 2 sit at 0.28 against 0.26, close to the line.

**The verdict for refinement: (b), in model R.**
- *Proved:* the hierarchy collapses to one trace (Lemma 1), and widths cannot grow (Lemma 3). Each run moved into transport costs at least one operation (Theorem 4), so at RV's price the runs of 1–3 rows can never be bought out.
- *Measured, then modelled:* no width class at or below 16 pays.
- *Classification:* Open (τ = 16) and Cells (τ = 0) are the two ends of this one family. By the frozen rule, every member is a known object.

**The Hévéa reading.**
- *Corrugations are refinement.* Here refinement only fragments: no level can manufacture a wider strip from narrower ones. The analogy fails at its most attractive point, as Louis expected.

**The escape clause (stated, as Louis required).** Theorem 4 does **not** rule out:
1. operations that compose non-adjacent transport maps (T^k, or merges across thick strips);
2. memoising words across pieces;
3. changes of coordinates (rank coordinates on S);
4. pointer jumping or list ranking by independent gathers;
5. decoders that emit out of orbit order at computed offsets.

§4 tests 1–3 by measurement, and 4 is the candidate.

## 4. Composition: summaries that compose across thin strips

Measured on Open's retained set, S = runs of width ≤ 16 plus p, for k = 1–8 ([`residual-and-composites.txt`](2026-11-23-refinement-and-composition/residual-and-composites.txt)).

**(3) Coordinates [measured].**
- *The test:* in retained-rank coordinates, consecutive retained rows can merge across a thick strip when their images are consecutive ranks.
- *The result:* rank pieces are only 0.35–0.87% fewer than geometric pieces. The coordinate change buys under 1%.

**(1) Composite transports [proved, then measured].**
- *Lower bound:* by Lemma 3, pieces(T^k) ≥ pieces(T) ≥ the number of thin runs for every k [proved].
- *How slowly they fragment:* pieces(T⁸) is only **1.33–1.39 times the number of thin runs** (0.63–0.84 m) [measured].
- *So:* an exact transport summary of span k on the retained walk has Θ(#thin runs) pieces at every span to 8.
- *The answer to Louis's question, in two parts:*
  - **In size, composition needs no global permutation information beyond one entry per thin run.**
  - **No summary can be smaller than that:** none is o(#thin runs), which is m/1.6–m/2.2 [proved].
- *Construction:* building T^k from T is independent lookups, one per piece or row per level. It is not a walk. But it is not obtainable from run records in o(m), since its output alone has ≥ m/2.2 pieces.

**(2) Symbolic sharing [measured].** Distinct k-return words, as a share of m:

| k | 1 | 2 | 3 | 4 | 8 |
|---|---|---|---|---|---|
| distinct words / m | 0.4–1.0% | 3.0–5.0% | 14–20% | 40–52% | 58–78% |

- *Up to span 3:* words are heavily shared, so a 2-word dictionary is 3–5% of m.
- *From span 4:* words are mostly unique.
- *What sharing buys:* it saves construction and storage. It saves no dependent depth, since a shared word does not share a destination.

**(4) The candidate: the squared retained map.**
- **Its law (exact) [proved]:**
  - T²(s) = T(T(s)), with the word w(s) w(T s);
  - walked two-ended, bijective, it takes **m/4 dependent rounds instead of Open's m/2** (Law 4);
  - the second word is read at T(s), off the critical path;
  - its dictionary is pairs of Open's band-word ids, so there is no expansion.
- **Its cost [proved as a count]:**
  - one pass of m independent gathers, N2[i] = T[T[i]];
  - an m-entry, 4-byte table;
  - for wire_xor, the same for T⁻¹.
- **Its pre-screen** ([`prescreen.txt`](2026-11-23-refinement-and-composition/prescreen.txt)) [measured, *not on the i9*]:
  - *Setup:* this session's 2.3 GHz Xeon VM, synthetic uniform random m-cycles. Law 3 makes these a fair stand-in for the retained core.
  - *The result:* gather plus chase came to **0.60–0.65 of the two-chain chase at m = 90–114k, and 0.73–0.89 at m = 156–204k.**
  - *A wide 16-byte layout lost* (1.19–1.86), because the table left L2. The falsifier must use the narrow layout.
  - *A second level (T⁴) added nothing over T²:* it read 0.62–1.05 against 0.63–0.85. **The composition hierarchy stops at one level.**
- **Against Open [modelled]:**
  - *The ceiling:* the gain is capped by Law 2's walk-latency pool, 2.65 ms of OpenCells' 10.02 ms on Neon's six-block set. If the latency halved, the ceiling would be about 1.3 ms, before the gather.
  - *On the i9:* L2 is 256 KiB, below every block's table, so the i9 should sit in the pre-screen's spilled regime.
  - **I expect a walk ratio of 0.85–0.95 there, a net 1–4% at best [conjectured].**
- **Classification:** this is Kärkkäinen, Kempa and Puglisi's super-alphabet (LF², 2012) applied to Open's trace instead of the full walk. **By the frozen rule it is a known object, so it counts as (b) for this construction.** It is not a new primitive.
- **What is new is only where it is applied.**
  - *The prior evidence against it:* the ψ² port (#34) lost on every corpus, random +49%. It ran on the full walk with 32 lanes, where the walk was instruction-bound.
  - *The evidence for it:* Bloom's measurements show 2-wide steps helping at 4–8 streams, random data included. Open's two-chain, L2-spilling walk is in that regime.

**The cheapest falsifier.**
- **The test:** run [`squared-trace-prescreen.c.txt`](2026-11-23-refinement-and-composition/squared-trace-prescreen.c.txt) on the i9. It is single-threaded, takes seconds and changes no decoder. Use m ∈ {89,874; 114,000; 156,404; 204,281}.
- **The candidate is dead if** the ratio is ≥ 0.95 at every m. It could not then clear wire_xor's second direction and the gather within Law 2's pool.
- **If it survives,** the next test is a T² arm in `bench-emit-cost` on OpenCells' real tables.
- **Deciding whether to run it is the Coordinator's call.** It needs a C5 workstation run.

## 5. Louis's four conditions, answered

1. **Does each refinement reduce the dependent walk?**
   - *Refinement:* yes, by |R|. But each run moved costs at least one operation [proved], and measured 1.63–1.74.
   - *Composition:* it halves the rounds once, then stops paying [measured, pre-screen].
2. **Is the structure cheap from the runs?**
   - *Refinement:* yes, O(r) from run records.
   - *Composition:* no summary is smaller than one entry per thin run [proved]. T² is m independent gathers from T, not o(m) from run records.
3. **Can the orbit transport back without a global expansion?**
   - Yes in both: substitution of return words, or pairs of word ids [proved].
4. **Does it beat Open?**
   - *Refinement:* not when priced [modelled]. Open is that family's priced optimum. In unit cost an interior τ = 2–3 is 10–30% shorter [measured]; the price decides.
   - *Composition:* possibly, by one level, a few percent at most [conjectured], with the falsifier above.

## 6. Neon's laws, faced

- **Law 1 (under 5.8 ns an operation):** RV's 22.2 ns breaks it, and Theorem 4 turns it into a per-run statement. The T² pass is about 0.5–0.9 ns a row on the pre-screen: one gather, with no sort and no probe.
- **Law 2 (the 2.65 ms pool):** it caps composition outright. The candidate is sized against it, not against the 10 ms total.
- **Law 3 (Haar-random core):** T² does not beat the miss floor. It adds m misses off the critical path and halves the dependent ones. Law 3 is also what justifies the synthetic pre-screen.
- **Law 4 (bijectivity, two ends):** T² keeps both. It must be built for T and T⁻¹ so that wire_xor stays.

## 7. What would change this

**Theorem 4 would fall to** an exact operation that moves a run's rows without acting on an interval inside that run. Lemma 3 says no first-return operation does.

**The pricing would move with the costs:**
- an RV step under about 7 ns would make the 4–16 classes pay (0.71 steps a row × 7 ns ≤ 5.3 ns);
- a walk row above about 16 ns would do the same.

**The composition result would fall** to a transport summary of span ≥ 2 with o(#thin runs) pieces. That is impossible by Lemma 3 for a union-of-runs S. For an S that cuts runs it is open, but Theorem 4 already charges such cuts.
