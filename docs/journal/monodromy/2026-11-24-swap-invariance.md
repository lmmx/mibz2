# 2026-11-24 · MONODROMY 101 · M-4(a), M-5: the swap-invariance lemma. A valid flip is a transposition of two column bytes (Hamming distance 2, not 3); it swaps two adjacent text blocks. An input blind to it cannot place any row outside origPtr's arc. Block-histogram positional rules need one block boundary per disjoint flip. Pglue's floor is one event per segment

Notation: column L of length n, σ = LF_L (stable sort of L), o = origPtr, pos(x) = the step at which the walk from o visits row x. L is *valid* iff σ is an n-cycle.

## 1. The flip [proved; checked by brute force, 12,397 candidates, 0 failures: `2026-11-24-swap-invariance.flip.py.txt`]

Let i < j, a = L[i] ≠ b = L[j], and let L′ be L with those two bytes exchanged. Counts are unchanged. Suppose (i, j) holds exactly one further a-or-b row, m.

- **Type B** (L[m] = b): σ′ = σ∘(i m j). **Type A** (L[m] = a): σ′ = σ∘(i j m). Every other row keeps its successor.
- *Proof.* Type B: the b-slots of rows m, j are s, s+1. After the exchange the b-rows in [i, j] are i, m in that order, so σ′(i) = s = σ(m), σ′(m) = s+1 = σ(j), and j takes i's a-slot, σ′(j) = σ(i). No other rank changes. Type A is the mirror. ∎
- **Validity.** σ∘(p q r) is an n-cycle iff p, q, r occur in that cyclic order on σ's orbit. So L′ is valid iff the orbit order is i→m→j (type B), i→j→m (type A).
- **Effect.** Write the orbit from i as i·X·p·Y·q·Z, with (p, q) = (m, j) or (j, m). The new orbit is i·Y·q·X·p·Z: **two adjacent text blocks exchanged.** Call A₀ = Z∪{i}, A₁ = X∪{p}, A₂ = Y∪{q} the flip's arcs.
- **Lemma 1 (who moves).** pos′(x) = pos(x) iff x lies in the same arc as o. Every other row shifts by ± an arc length, nonzero mod n. At least two of three arcs move.

**Premise correction (break, against the handoff and record 100 §3a's wording):** the count-preserving flip is at Hamming distance **2**, not a 3-cycle of symbols. "3" is the number of rows whose *successor* changes. Smallest instance: `caab` ↔ `cbaa` (texts aabc, abac). Adjacent exchanges (j = i+1) are never valid (sign). Record 100 §3a's conclusion is unaffected.

## 2. The swap-invariance lemma [proved]

A *rule* is a map from an input ι(L, o) to a set of facts; it is *sound* if every fact it outputs at (L, o) holds at (L, o). A valid flip φ is *blind* for ι if ι(L, o) = ι(φL, o).

**Lemma 2.** If φ is blind for ι, every fact a sound rule outputs at L holds at both L and φL. Hence:
1. it fixes pos(x) for no x outside o's arc of φ (Lemma 1);
2. it fixes neither the byte nor the successor of i or j, nor the successor of m.

*Proof.* Same input, same output; soundness in both worlds. ∎

**What the rule may still say:** facts stable under φ. In particular a rigid fact (LF^h x = x + t, word w) whose h-step path avoids {i, m, j} is stable. So the lemma closes *positions*, and successors only at flip rows. It does not close 𝓡-type facts emitted in bulk; those are closed only where the row is itself a flip endpoint (item 2).

## 3. The block-keyed cell [proved, given ν′; ν′ is a NEEDS]

**Λ_agg(𝒫, D):** rules whose input is the symbol histogram of each block of an interval partition 𝒫 of the rows, plus the full records of a row set D, plus n and o. A flip is blind iff i, j ∉ D and no boundary of 𝒫 lies in (i, j].

For a row x let **ν′(x)** = the largest number of valid flips, with pairwise disjoint column spans [i, j], whose arcs separate x from o.

**Theorem 3.** A sound rule of Λ_agg(𝒫, D) that fixes pos(x) has |𝒫| + 2|D| > ν′(x).
*Proof.* Each of the ν′(x) flips must not be blind (Lemma 2.1). Disjoint spans need distinct boundaries or distinct rows of D. ∎

**Reading.** If ν′(x) = Θ(n), a histogram rule that places x has blocks of average size O(1): its "aggregate" is the column. Coarse aggregates (counts, C[], rank samples every B ≫ 32 rows, frequency profiles) place **no** row outside o's cell U = ⋂_φ arc_φ(o).

**How many flips [conjectured under model I plus orbit-order independence].** For bytes i.i.d. uniform on 16 symbols near x, the expected number of candidate flips with x as an endpoint is 4·(15/4) = 15 (types A and B, x as i or as j; each b ≠ a qualifies with probability 1/4). Each is valid with probability ≈ 1/2 (measured 0.51 on random small columns). So essentially every hex width-1 row is a flip endpoint, ν′ ≈ c·n with c of order 1/32 per hex row, and |U| ≈ 1/c.

**What Theorem 3 is not.** It does not charge rows (handoff's caution). A rule reading Θ(n) fine-grained inputs once and placing many rows is outside it. That residue is exactly E2-global with fine input, Neon's cell. The lemma's use against Neon: any construction whose positional step reads only coarse aggregates is dead on every x ∉ U.

## 4. M-5: Pglue's floor [proved as a count, given clause 1]

A *segment* is a maximal run of consecutive orbit rows of one stream. seg_R(c) ≤ h₁R, seg_N ≤ h₁N; with wide rows as a third class, switches ≤ 2·min(seg_R, seg_N + seg_W).

**Proposition 4.** In any glued decoder, each segment whose start position is not fixed by a multi-row fact costs one event. By Lemma 2 no Λ_agg fact with ν′ > |𝒫| + 2|D| fixes it. So c₁ ≥ seg_R(c) − |U| over 𝓡 ∪ local ∪ Λ_agg, **even if everything inside a segment were free.**

**Size [argued from record 96's 0.88].** A GUID has 5 groups; within a group, breaks occur at the 12% non-width-1 nibbles. seg_R ≈ 5 + 0.12·0.88·27 ≈ 7.9 per GUID against ≈ 27 R-rows: a floor **aR ≥ 0.29** from glue alone. With h₁R ≈ 0.8·h₁ that is 0.23·h₁: not by itself above 1 − δ, so Pglue does not decide the theorem; it removes about half the margin.

## 5. NEEDS

- **NEEDS M-4** on each of the 6 admitted blocks (then 𝓕): enumerate type-A and type-B flips (§1) and their validity by orbit order. Report (i) the fraction of width-1 hex rows that are an endpoint of ≥ 1 valid flip; (ii) a greedy disjoint-span count ν′ of valid flips, as ν′/n; (iii) |U|, the rows in o's arc for every valid flip; (iv) the distribution of gaps between consecutive flip cut points along the orbit.
- **NEEDS M-5** on the same blocks: seg_R, seg_N, seg_W under σ_hex, and seg_R/h₁R.
