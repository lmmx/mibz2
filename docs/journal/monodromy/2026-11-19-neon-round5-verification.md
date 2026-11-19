# 2026-11-19 · MONODROMY 81 · Neon's weight-only Rauzy–Veech position theorem verified exhaustively, several cycles and λ > 1 bands included; a band's tails are anchored on its right endpoint's cycle. It settles boundary gaps only: every gap's depth follows from the positions, but a single interior gap costs one search per row of run offset (mean 1,644 on block 0, against a walked depth of 262). Its "1.015 r accelerated steps" are record 80's winner-run count, a lower-bound accounting that no O(1)-per-step kernel executes: its own quotient never fires on Wikidata

**The request.** The Coordinator's CHECKPOINT 7007538945 (#346). The source is PR #629, branch
`docs/neon-round4` at ef09f966, file `docs/journal/2026-11-19-gemini-neon/response-5.md`, §2.

I did not open `verify_rv_copy.py.txt`, `round5_output.txt` or the README's round-5 section
before these checks.

**Labels:** **P** · **C** · **I**.

**Scripts** (beside this record as `.txt`):
- `neon5.py`: Theorem 1. It loads record 78's `neon3.py`.
- `neon5gap.py`: every gap's depth.
- `neon5acc.py`: step counts.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 1 | Integer-only accelerated RV recovers the orbit positions of all run heads, all run tails and the mark, including quotient steps, several cycles and periodic bands | **VERIFIED WITH CORRECTION.** 0 failures in 70,706 runs (60,159 with several cycles; 27,877 bands with λ > 1; 10,485 quotient steps). The correction: on a band with λ > 1, the tails resolve on the cycle through the band's right endpoint, which is not the heads' cycle. That frame is phase-aligned, as §2.2's parenthesis says, but "identical cycle" holds only at λ = 1 |
| 1.4 | Boundary-gap depths ℓ(c), c ∈ C̄, from sorted head positions | **VERIFIED** |
| 2 | Does this settle the separation depth of every gap? | **Boundary gaps only, as a cheap result.** Every gap's depth is determined, with one successor search per row of run offset. That is exact on all 848,747 interior gaps of block 0, but costs more than walking the gap for a single query |
| 3 | "The exact same ≈ 1.015 r accelerated steps" | **REFUTED.** Neon's quotient condition holds 0 times on blocks 0, 85 and 210, so its accelerated count equals the raw count (92,571 on block 0). 1.015 r is record 80's winner-run count, which no kernel executes in O(1) a step |

## 1. Theorem 1 (P, C)

**What I implemented.** §2.1 exactly as written:
- *State per interval:* `len`, `wlen`, `h_shift`, `t_tok` and `t_shift`.
- *Forests:* head and tail merge forests with their deltas.
- *The mark:* (ip, θ, δ).
- *Quotients:* in Case 1 when A = cod_prev(B), in Case 2 when B = dom_prev(A).

My coordinate model supplies only the geometry: which A, B and neighbours, and executing the q
raw steps.

**What I checked.**
- *After every accelerated step:*
  - (H): d_I lies on the same cycle as h(I), and pos(d_I) − pos(h(I)) ≡ h_shift[I].
  - (T), for every live token.
  - w(I) = the return-word length.
  - The mark: p₀ = T^δ(the row at offset θ of ip).
  - That the q raw steps of a quotient step keep the same pair and case.
- *At the end:*
  - Each head, tail and the mark, resolved through its forest to a closure root, equals the
    direct LF walk on the same cycle, modulo the cycle length.
  - Every band's cycles have length w(A).

**Coverage.** Every column over 2 letters to length 10, 3 letters to length 7 and 4 letters to
length 6, and every start row: 70,706 runs, 60,159 of them with several cycles.

| | count |
|---|--:|
| quotient steps (q ≥ 2) | 10,485 |
| closures with λ > 1 | 27,877, each with a tail root; 11,481 holding the mark |
| failures of (H), (T), w, the mark, quotient pairs, head, tail or mark positions | **0** |

**The proof.**
- *Each transfer is the geometry:*
  - Case 1: r_old(A) = T^{w(B)}(r(B)); steps 2…q leave the tails alone, since r(A) is then
    interior.
  - Case 2: h(A) and r(A) move back by w(B) a step while w(B) stays fixed; the two tokens meet at
    r_old(B).
  - Case 3a: h(A) = T^{w(B)}(h(B)), and likewise for r.
- *The mark's chunk index k* gives the right δ: at step k + 1 the loser's word has length
  w(B) + k·w(A).

**The one correction is to the frame at a band with λ > 1.**
- *Which cycles:* the band's λ cycles are distinct. Heads lie only on the cycle through h(A); the
  others are passive, meeting no head. Tails resolve on the cycle through r(A) = h(A) + λ − 1.
- *Why that is still well defined:* every band row's return word is ω(A), so the λ cycles spell
  the same word in phase. Measuring each from its own band row is consistent, as the
  parenthesis in §2.2 says.
- *For any column, not only u^q:* this holds because all band rows share ω(A), whatever the
  column.
- *What "cycle ID" must mean:* a tail's cycle is the band's right-endpoint cycle, not the root's
  head cycle. 46,527 tails are on a different cycle from their run's head root.

## 2. Scope: can a gap's separation depth be found without walking its path? (P, C)

**The setting.** Take depth(x) = min{j ≥ 0 : LF^j(x) ∈ D̄} for the gap between rows x − 1 and x.

**What Theorem 1 settles.** It gives the r boundary depths ℓ(c), for c ∈ C̄, from the sorted
head positions, in O(r log r) after the induction.

**Every other gap lies on an arc.** For x ∉ D̄ ∪ C̄, depth(x) = 1 + depth(LF x), so
x = LF^i(c) on the arc of some c ∈ C̄, and depth(x) = ℓ(c) − i. That needs i, which is x's
position on its arc: a quantity the head and tail positions do not give directly.

**It can be recovered without LF-walking, but serially along the run.**
- *The mechanism:* if pos(x − 1) is known, the first tail at or after it on x − 1's orbit is
  LF^{depth(x)}(x − 1). This is the first point where the pair separates; it cannot hit a tail
  earlier, or x would hit a head earlier.
- *One step:* so depth(x) is one successor search over that cycle's sorted tail positions. The
  tail's head, tail + 1, then fixes pos(x) = pos(head) − depth(x).
- *Over a run:* starting from the run's head, whose position is known, each gap of the run costs
  one search. This is the r-index's Φ⁻¹ step.

**Counted.**
- *Small columns:* exact on every column of the same sizes in which every cycle meets a head
  (7,307 columns, 5,609 with several cycles).
- *Columns with a passive cycle:* 3,478 are excluded. There a gap that never separates has
  infinite depth, and the next gap's frame must be taken by phase alignment. Not checked.
- *Block 0:* exact on all **848,747** interior gaps, with 848,747 searches.

**The cost of one gap.** A gap at run offset o needs o searches, unless SA access is built: the
O(r log(n/r))-space structure of Gagie, Navarro and Prezza (JACM 2020) gives O(log(n/r)) per
query. That is literature, not checked here.

| block 0 | value |
|---|--:|
| mean run offset over interior gaps (searches for one gap) | **1,643.5** (max 26,406) |
| mean depth over gaps (LF steps to walk one gap) | **262.2** (max 4,713) |

**So, for one gap,** the head and tail positions cost about 6× more steps than walking its path.

**For all gaps,** a single backward sweep over the cycle already gives every depth in O(n). The
answer is therefore: **boundary gaps, yes, in O(r); every gap, determined, but not cheaper than
the walk per query.** There is no counterexample to "determined", because L and the positions
fix the text.

## 3. The step counts (C, I)

**Counted** with `neon5acc.py`, K = 1, Neon's quotient condition evaluated at every step:

| block | raw steps | Case 1 / 2 steps with A = cod_prev(B) or B = dom_prev(A) | Neon's accelerated steps | winner runs (record 80) |
|--:|--:|--:|--:|--:|
| 0 | 92,571 = 1.809 r | **0** of 41,387 | 92,571 | 51,849 = 1.013 r |
| 85 | 140,544 = 1.771 r | 0 of 61,169 | 140,544 | 80,701 = 1.017 r |
| 210 | 216,769 = 1.684 r | 0 of 88,037 | 216,769 | 130,546 = 1.014 r |

**Why the two counts differ.** They count different things.
- *Neon's quotient* repeats the same pair (A, B). That needs the loser's new codomain, the tail
  of A's, to land back at the right end, which happens only when cod(A) immediately precedes
  cod(B). On these blocks it never does.
  - In small columns it fires (10,485 times above), mostly on short multi-cycle columns.
- *Record 80's winner run* keeps the winner and changes the loser at each step. It was defined to
  prove S_Z ≥ r: each run deletes at most one interval.

**A winner run is not one O(1) step.** Each loser B_i takes its own chunk of A's tail: its own
weight update `wlen[B_i] += w(A)`, its own relink after A (the losers end up in reverse order),
and possibly its own tail-token transfer. Finding where the run ends means summing the losers'
lengths against λ(A). So a run of k losers costs Θ(k).

**Weight-only RV therefore executes 1.68–1.87 r steps, the same count as Cells.**
- *Lighter steps, interpretation:* it writes no words or rules, so each step is lighter. That is
  the only saving in the induction, and it is unmeasured.
- *The cost model's line is wrong:* §4's "exact same ≈ 1.015 r accelerated steps" should read
  1.68–1.87 r.

## What this decides

Nothing in production.
- *Theorem 1* is a correct O(r)-memory way to sample head, tail and mark positions, at Cells'
  step count.
- *The open question* is answered for boundary gaps. For a single interior gap it is not answered
  cheaply.
- *Theorem 2's copy replay* (not audited here) is what would have to pay for it.
