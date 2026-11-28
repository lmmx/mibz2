# Record 118: interleaved cursors. A lower bound, a construction, and a break of my own conjecture

*MONODROMY, 24 November 2026. Closes item 1 of record 116 §3. Labels: proved / argued / measured / conjectured. No cut in Q below a constant fraction is claimed. Relation C (channel #26) stays open: it is not in-place composition, so nothing here reaches it.*

## 1. Setting

The model is record 116's: independent uniform digits over σ symbols. A row is a point u = 0.c₀c₁c₂… of [0,1), and the next text position is Su = σu mod 1. A build sweep is a processing time τ on rows. The step into Su extends a chain iff τ(u) < τ(Su); otherwise it **fails** and a link is paid there or at the next row (record 112, B2). Let ε(τ) be the measure of failing u.

𝓜_k is the class of orders that are increasing on every k-digit block, the blocks interleaved in any way. Record 116's 𝒮_k (blocks one after another) is inside 𝓜_k.

## 2. Lower bound for every interleaving (proved in the model)

**Theorem I.** Let τ be increasing on each piece of a partition into B intervals. For every q,

ε(τ) ≥ A_σ(q) − σ(B−1)·σ^(−q), where A_σ(q) = σ^(−q) · Σ over Lyndon words w of length q of 0.w^∞.

If every (q−1)-digit block lies inside one piece, the second term is absent. So ε ≥ A_σ(k+1) on 𝓜_k.

*Proof.* Fix a Lyndon word w, with periodic points x_j = 0.(w_j w_{j+1}…)^∞. Let C_j be the block of w_j…w_{j+q−2} and D_j ⊂ C_j the block of w_j…w_{j+q−1}. S maps D_j affinely onto C_{j+1}: in block coordinates, the point at v′ in C_{j+1} has its preimage at h_j(v′) = (w_{j−1} + v′)/σ in C_j. The coordinate of x_j in C_j is ξ_j = x_{j−1}, and ξ_j = h_j(ξ_{j+1}).

Let f_j(t) be how far the cursor of C_j has got, and e_j = ξ_j − f_j its deficit. The step landing at cursor j+1's position is good iff cursor j has already passed its preimage: f_j > h_j(f_{j+1}), which is e_{j+1} > σ·e_j.

While every deficit is positive, a cursor i with the least deficit has e_i ≤ e_{i−1} < σ·e_{i−1}. So **every move of a least-deficit cursor is a failing step.** The least deficit starts at min_j ξ_j = 0.w^∞ and must reach 0, and it falls only when such a cursor moves. The failing preimages lie in the D_j, at scale σ^(−q). Blocks D_j of different Lyndon words are disjoint. A piece boundary spoils at most σ words. ∎

**Values.** A_σ(1) = 1/2 exactly. A_σ(q) ≈ 1/(q(q+1)) for large σ. For hex, A₁₆(2) = 0.156.

**Status.** Proved for the shift on [0,1). The transfer to a finite text is argued: preceding digits are independent of suffix order, so each deficit relation holds up to binomial fluctuation, for n ≫ σ^q. Pieces all decreasing follow by the symmetry u ↦ 1 − u. Mixed directions are not covered.

## 3. A construction: the min-key order (proved in the model; measured)

**Definition.** τ_j(u) = min(u, Su, …, S^j u), ties taken in text order. It is increasing on each j-digit block, so τ_j ∈ 𝓜_j.

**Proposition.** The step into Su fails iff S^(j+1)u is the least of u, Su, …, S^(j+1)u. So **ε(τ_j) = 1/(j+2).**

**Streamed.** For j = 1 the key is min(x, ψ(x)), and ψ on a symbol's bucket is a forward scan of L for that symbol. With σ pieces and σ gather cursors each, σ² cursors serve it: 256 for hex. No seeks.

**Measured,** n = 200,000 random hex digits (`.sim.py.txt`):

| order | failing steps | theory |
|---|---|---|
| row order | 0.5009 | 1/2 |
| min-key, j = 1 | 0.3338 | 1/3 |
| min-key, j = 2 | 0.2504 | 1/4 |
| min-key, j = 3 | 0.2004 | 1/5 |
| greedy 16-queue scheduler, best deadlock rule | 0.3353 | none |

**Prior art.** Keying on a local minimum in a window is deterministic coin tossing (Cole and Vishkin 1986) and minimizer sampling (Schleimer, Wilkerson and Aiken 2003; Roberts et al. 2004). Its use as a cursor schedule for in-place composition I have not seen, and I have not searched for it.

## 4. Break: record 116 §3.1

I conjectured that interleaving keeps 𝒮_k's bound. **That is false.** τ_k is in 𝓜_k and fails on 1/(k+2) of steps, below 𝒮_k's floor N_σ(k+1)/σ^(k+1) ≥ 1/(k+1). Interleaving buys one level of depth.

It buys no more than record 116 §4 already offered. A sequential order on (k+1)-digit blocks reaches about 1/(k+2) too, for σ^(k+2) seeks: 4,096 on hex at k = 1. So this is the same corner, reached without seeks.

## 5. Where the corner stands

| class, hex, K = 2 | failing fraction: best known | proved floor |
|---|---|---|
| row order | 1/2 | 1/2 |
| 16 first-digit pieces interleaved (256 cursors) | 1/3 | 0.156 |
| 256 two-digit blocks in sequence | about 1/3 | 0.336 (record 116) |
| any B monotone pieces, any interleaving | none | A₁₆(q) − 16(B−1)/16^q |

- **Q on 16 interleaved pieces:** at least 0.078·n_d by B2's halving, against n_d/6 with the min-key order.
- **The gap at depth 1** is 0.156 to 1/3. The greedy scheduler found nothing under 1/3. I conjecture 1/(k+2) is optimal on 𝓜_k.
- **In general:** each seek opens at most one new piece, so B ≤ cursors + Q. Theorem I then gives Q of order n/log²n for every in-place composition sweep, however its cursors interleave. Record 116 had n/log n for sequential blocks only.

**Obstruction write-up.** *Obstruction:* near each periodic orbit the cursors' deficits would have to grow by σ at every step around a closed cycle. *Scope:* in-place composition, pieces of one direction, independent digits. *Escape:* depth (more pieces) at 1/q² per level; or information not carried by composition. *Transferable insight:* an interleaved order is a monotone path in the cube of cursor positions, and its failures are the path's time below the periodic points.

## NEEDS

`NEEDS M-25` Rows per link after one in-place sweep in min-key order (j = 1) against row order, on the six admitted blocks. Rule, fixed before data: under 10% more rows per link on at least four blocks closes it.

---
*Corrected by record 126 (`2026-11-24-corrections-118-125.md`) after independent review.*
