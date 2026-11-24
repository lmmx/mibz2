# 2026-11-24 · MONODROMY 110 · A construction: in-place composition in one row-order sweep builds word-carrying macro-edges of mean length e, not 2, with no walk. Two sweeps give e². The object to acquire is the macro-edge, not a copy: it needs no text positions

Answers channel entry #13. This is a construction, not a closure. Novelty is **not established**: in-place ("Gauss–Seidel") pointer jumping is folklore; I have not searched for its use on LF.

## 1. What a non-walking acquisition needs, and what carries it [proved]

- **Not copy(dst, src, len).** A copy needs two text positions, and positions are the walk. The same repeat can be used without positions as a **macro-edge**: (x → LF^ℓ(x), word of ℓ bytes). The consumer is Open's existing one: emit a word, jump.
- **The structure that carries it is the suffix interval.** Let D_w be the rows whose next ℓ emitted bytes are w, and u the reverse of w. Then LF^ℓ maps D_w, in order, onto the interval I_u of rows whose suffix begins with u (Lemma F iterated, record 104). So LF^ℓ(x) = start(I_u) + (rank of x in D_w): a stable counting sort by the ℓ-word. Φ-runs and toehold samples are positional; this is not.
- **The words come from L by distribution.** Column j of the sorted matrix, counted from the right, is L_j[x] = L[LF^{j−1}(x)], and L_{j+1} restricted to the rows of symbol c, in order, is L_j restricted to I_c, in order. So one pass with one sequential read cursor per symbol produces the next column. No dependent chain, no row is visited in orbit order.

**What it needs to know** is only this: an order on rows in which a row's successor has already been processed. Orbit order is the perfect one and is the answer itself. Row order is a free, imperfect one.

## 2. The construction: sweep composition

Hold for every row an edge (J[x], word[x]), initially (LF(x), L[x]). Make **one sweep in increasing row order**, and at each x replace its edge by its composition with the *current* edge of J[x]:

  word[x] ← word[x]·word[J[x]],  J[x] ← J[J[x]].

If J[x] > x its edge is still the original, and x gets length 2 (this is T²). If J[x] < x its edge was already lengthened in this same sweep, and x inherits all of it.

**Proposition S** [proved under exchangeable row ranks along the orbit]. After one sweep the edge at x has length 2 + d(x), where d(x) is the number of consecutive descents x > LF(x) > LF²(x) > … . P(d ≥ j) = 1/(j+1)!, so the mean length is 2 + (e − 2) = **e**. A second sweep in decreasing order multiplies again, to about e².

*In text terms:* a descent chain is a maximal run of text positions whose suffix ranks decrease going backward, so the sweep contracts each row through an **ascending run of the inverse suffix array**. Plain doubling (reading only old values) gets exactly 2 per pass.

**Checked** [measured, synthetic only; `2026-11-24-sweep-composition.gs.py.txt`], rows per link along the orbit:

| text | n | 1 sweep | 2 sweeps | doubling, 1 and 2 passes |
|---|---:|---:|---:|---|
| random hex | 60,000 | 2.76 | 7.51 | 2.00, 4.00 |
| GUIDs in a JSON template | 82,890 | 2.63 | 6.69 | 2.00, 4.00 |
| the same, words capped at 8 bytes | | 2.75 / 2.63 | 5.70 / 5.65 | |

**Instruction set:** sequential read of L with one cursor per symbol; in-place compose; nothing else. In the first sweep the read of J[x]'s edge is itself on a per-symbol sequential cursor, because LF is increasing on each symbol class.

## 3. Where it stands against the target: a near miss, named

- **Events.** The sweep reads every row's record, so under the convention I fixed in record 108 it is charged per row and is **not a yes**. I do not reopen that.
- **Under the lenient reading** (a streaming pass is free) with K = 2, record 108 put the best known single-pass count at m/2 ≥ 0.60·h₁. Sweep composition on Open's retained rows would give m/e ≈ **0.44·h₁ on the best census block** (m/h₁ = 1.207) and 0.95·h₁ at the median (2.586). **So it crosses δ = 1/2 on the easiest blocks only, and only under the reading I rejected.** It is not established that e survives on Open's retained map, whose order statistics differ from LF's.
- **Unit price** (residual b). It converts dependent walk steps into independent, sequentially streamed build steps at an exchange rate of e per pass instead of 2. T² was built at rate 2 and lost (E = 1.22, one implementation). Whether rate e pays is a measurement, not mine.
- **Utility.** The supplied-codes diagnostic (main-theorem §7) bounds any emitter gain to low worker counts.

**What would make it a hit:** a cheap order on rows closer to orbit order than row order is. Chain length is the length of the runs in which that order agrees with text order. Row order agrees with text order on ascending runs of suffix rank, mean e − 1 extra steps. Any key κ with κ(LF(x)) < κ(x) more often than half the time lengthens every chain geometrically. I have no such key; this is the search target, and it is a concrete one.

## 4. NEEDS

- **NEEDS M-17** on the six admitted blocks: run sweep composition (i) on LF over all rows and (ii) on Open's retained map, one ascending sweep and then one descending, words capped at 8 and at 16 bytes. Report rows per link along the orbit after each sweep, and the distribution of edge lengths. Decision rule, fixed now: if (ii) after one sweep gives fewer than 2.3 retained rows per link on four or more blocks, Proposition S's model does not transfer to the retained map and the near miss is withdrawn.
