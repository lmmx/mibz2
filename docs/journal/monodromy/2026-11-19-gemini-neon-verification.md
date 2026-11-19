# 2026-11-19 · MONODROMY 78 · Gemini 4 Neon's three theorems verified independently. Theorem 3's batches are exact under the stated condition, but the condition is stronger than the mathematics needs. The mark rule is incomplete, and its parallelism is limited by uneven components, not by conflicts. Theorem 1 holds, refining Astra E's h. Theorem 2's Fibonacci indexing is off by one. "Cells is strictly serial" is refuted by the shipped kernel's fetch-ahead

**The request.** The Coordinator's CHECKPOINT 7005355308. The source is PR #627, branch
`docs/journal-gemini-neon` at f6a1425e, file `docs/journal/2026-11-19-gemini-neon/response.md`.
The findings are for Louis.

**Labels:** **P** · **C** · **I**.

**Method.** I wrote my own checkers from the response's definitions, without the PR's
`check_theorem1.py` or any other reviewer's verdict. They are kept beside this record as `.txt`:
- `neon3.py`: the coordinate model of the K-front induction;
- `neon3c.py`: exact removed sets, serial orders, and the pointer model with its mark;
- `neon3scale.py`: batch widths on the frozen column;
- `neon1.py`: Theorem 1;
- `neon2.py`: the Fibonacci cylinders.

What the models cover:
- *Coordinate model:* any column L, so tied rotations and several cycles are included. Pins are
  any subset of the internal C[c]. Every start row p carries the mark.
- *Comparison:* every batched step is checked against direct first returns along LF on the
  surviving rows, covering destinations, return words and the mark.
- *Pointer model:* the response's own representation, with no coordinates. It uses global dom and
  cod lists with a sentinel after each component. Each front is executed against the pre-state,
  then the writes are merged.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 3.1 | Component conservation; pinned boundaries stay in D ∩ C; Δr ≤ 0 | **VERIFIED** (no interval ever crosses a pin) |
| 3.2 | Under the six-interval condition, a batch is exact and confluent | **VERIFIED**. 8,049 batches were exact, and every serial order agrees |
| 3.2′ | The sufficient condition | **SHARPENED**. The mathematics needs only that no front's A is another's B. The other four clauses are pointer hygiene, needed only for a simultaneous rewrite |
| 3.3 | Mark transport | **VERIFIED WITH CORRECTION**. In codomain coordinates the stated rule loses the mark; one relabel clause is missing |
| 3.4 | "> 99% of batches run all 16 fronts": B-way MLP | **REFUTED on the frozen column**. Mean width is 3.84 at K = 16, bounded by the heaviest component |
| 1 | r(T_X′) ≤ r + (c − 1) + \|∂X′ \ (D ∪ C)\|, and the cylinder refinement | **VERIFIED WITH CORRECTION** (the definition of t). No violation in 2,220,143 sections or 47,062 cylinders |
| 2.1 | Kac: Σ τ = n_cov | **VERIFIED**. Its scope is schedules that trace every row |
| 2.2 | Fibonacci separation | **VERIFIED WITH CORRECTION**. The prefix length and the range of d are each off by one |
| 4 | "Cells is strictly serial, MLP = 1" | **REFUTED**. The shipped eager kernel fetches 12 slots ahead in both orders |

## Theorem 3: multi-boundary parallel Rauzy–Veech (P, C)

**Counted.** I took all columns over 2 letters to length 8, 3 letters to length 7 and 4 letters
to length 6. For each I took every subset of the internal alphabet boundaries as pins and every
start row.
- *Runs:* 242,216 full runs.
- *Batches tested:* at every state of every run, every subset of two or more active fronts.
  There were 866,779 such batches.
- *Advancing the run:* between tests, the run advances by a greedy batch under the response's
  condition, or by a closure.

| batch class | batches | exact simultaneously (X′ = X ∖ ∪Y_j, every arrow, word and mark) | all serial orders agree (n ≤ 6) | pointer model exact |
|---|--:|--:|--:|--:|
| the response's six-interval condition | 8,049 | **8,049** | 6,424 / 6,424 | **8,049**: no write conflict, no stale read |
| only {A_j} ∩ {B_j′} = ∅ (j ≠ j′) | 232,163 | **232,163** | 176,361 / 176,361 | 113,670; 118,493 corrupt the lists |
| some A_j = B_j′ | 626,567 | 305,002 | 425,373 / 499,438 | 0 |

The full runs:
- every mark was output with the right rotation;
- no front was ever off a boundary;
- no interval crossed a pin;
- no step raised r.

### What the proof gets right, and the condition it actually uses

**The proof's step 2 holds.** It shows the cuts do not chain: T_X(Y_j) ∩ Y_j′ = ∅.
- T_X(Y_j) ⊆ cod(A_j) in every case.
- Y_j′ ⊆ cod(B_j′) in every case: it is B_j′'s codomain in Case 1, a tail of it in Case 2, and
  all of it in Case 3.

  The response says "cod(B_j′) or dom(A_j′)". But dom(A_j′) ⊆ cod(B_j′) whenever it is the cut,
  so the cod(B_j′) half is all the argument needs.
- So T_X(Y_j) ∩ Y_j′ ≠ ∅ exactly when A_j = B_j′. Both sets end at b_j′ in the same codomain
  interval, so they always meet then.
- With A_j ≠ B_j′ for every ordered pair, no front changes another front's A, B, length or word.
  So every return time is at most 2, and the batch equals any serial order.

**The exact condition is weaker than the response's.** It is
**{A_j : j ∈ S} ∩ {B_j : j ∈ S} = ∅ across distinct fronts**. That certifies 29.8 times as many
batches as the six-interval condition does (232,163 + 8,049 against 8,049).

**The other four clauses (cod_next(A), cod_prev(B), dom_next(B), dom_prev(A)) are not
mathematics.** They make a write-from-the-pre-state pointer rewrite safe.
- *Batches meeting only the weak condition:* executed that way, 118,493 corrupt the lists.
- *Executed serially:* every one is exact, because each pointer is read fresh, as a pipelined
  single thread would read it.
- *The smallest such corruption:* L = (1,0,2,1), pin {1}. The fronts are (A,B) = (0,1) and (3,2),
  both Case 3, and cod_next(A₁) = A₂. Each front rewires that one neighbour, so the second
  front's stale read writes through a deleted interval.

**A_j = B_j′ cannot be dropped.** The smallest witness is L = (1,0) with pin {1}.
- A₁ = B₂ and A₂ = B₁, so both rows are cut together.
- X′ = ∅ and no closure is recorded: the 2-cycle and its mark are lost.
- In serial order, the first cut leaves a closure on the second front, which outputs the word.
  This is not a counterexample to the theorem, which excludes it.

**Pinned boundaries.** "b_j stays in D ∩ C" holds for a simple reason:
- every new domain lies inside an old domain, and every new codomain inside an old codomain;
- a codomain of a c-run never leaves [C[c], C[c+1]);
- so no interval crosses a pin, and each component's surviving right end is the end of exactly one
  domain and one codomain interval.

### The mark (P, C)

**Statement 3 is the right rule for a point that is cut.** It applies when p ∈ Y_j: the mark
moves to T_X(p), at offset θ in the new codomain of B_j, with δ′ = δ + |ω(A_j)|.

**It is not the whole rule in the coordinates the response uses.** A coordinate-free mark is
(interval, offset) in the codomain list, which is what "offset θ inside the codomain of B_j"
means. Two clauses must be added:
- *Case 1, not cut:* if p lies in the tail of A_j's codomain (offset ≥ λ(A_j) − λ(B_j)), that tail
  now belongs to B_j. The mark becomes (B_j, θ − (λ(A_j) − λ(B_j))), with δ unchanged.
- *Case 3, not cut:* a mark on A_j's codomain moves to B_j at the same offset, with δ unchanged.

**What the literal rule does.** It loses the mark in 93 of the 8,049 conflict-free batches. With
the two clauses there are none. This has nothing to do with batching: it fails on single steps
too.
- *The smallest instance:* L = (1,0,0,3,2), p = 1, pin {3}.
- Front 1 is Case 1 with λ(A) = 2 and λ(B) = 1. Row 1 is the tail of A's codomain.
- After the step, row 1 belongs to B's codomain, but the literal rule leaves the mark on A at
  offset 1, past A's new length.

**Codomain coordinates are the right choice, and they need no walk.**
- *Case 1:* p ∈ Y means p's codomain interval is B_j.
- *Case 2:* p ∈ Y means p's codomain interval is B_j, with offset ≥ λ(B_j) − λ(A_j).
- *Case 3:* p ∈ Y means p's codomain interval is B_j.

So each step costs O(1). In domain coordinates the rule would not close: T_X(p) is known only by
its codomain place.

### The parallelism claim, on the frozen column (C)

**What was run.** The frozen column has n = 899,931 and r = 51,184. Pins are the internal
alphabet boundaries nearest equal mass (Variant A). Each round runs the greedy batch of active
fronts that meets the condition. `neon3scale.py` is the pointer model at full size.

| K (distinct pins + 1) | condition | cells | rounds (critical path) | mean width | rounds running every active front |
|--:|---|--:|--:|--:|--:|
| 1 | | **92,570** | 92,570 | 1.00 | |
| 16 | six-interval | 95,118 | 24,743 | **3.84** | 0.920 |
| 16 | exact (weak) | 95,752 | 23,989 | 3.99 | 1.000 |
| 28 (32 asked) | six-interval | 93,469 | 23,900 | 3.91 | 0.945 |
| 16, pins balanced by runs | six-interval | 101,078 | 26,255 | 3.85 | 0.963 |

**K = 1 reproduces production.** It gives exactly block 0's 92,570 right-end steps (README, Block
0), which cross-checks the model.

**The limit is occupancy, not conflict.**
- *How many fronts are live:* in 85% of K = 16 rounds, five or fewer.
- *Why:* the components empty at very different rates. The heaviest component does 25.6% of all
  cells (24,318), and it alone sets the 24,743 rounds.
- *Why the pins can't fix it:* pins can only fall on alphabet boundaries, and the `"` bucket alone
  is 15.1% of the rows.
- *Upper bound:* the critical-path speed-up is about cells ÷ max_j cells_j, near 3.9. Pinning adds
  0.97% to 9.2% cells.

**The response's 3B²/r_live union bound** treats touched sets as random. Measured, conflicts
block a front in 8% of K = 16 rounds under the six-interval condition, and never under the exact
condition. That is not the binding cost.

**Not checked.**
- *The optional Zorich acceleration:* it is not part of the theorem.
- *Peak live width under multi-boundary cuts:* the response flags this as open itself.

## Theorem 1: the discontinuity budget (P, C)

**The proof holds, with one definition corrected.** As written, t is "the smallest index where
the pair fails to remain an adjacent pair that either both stay outside or both return".

**Why that is wrong.** LF keeps a pair adjacent across a run seam whenever x − 1 is the last c
and x is the first c′, the next letter present. The pair then reads different bytes without ever failing that
test.
- *The smallest instance:* L = (0,1), X′ = {0,1}. It has two pieces, but the literal definition
  finds no split.

**The correction.** Take t as the least s ≥ 0 with T^s(x) ∈ D, or with s ≥ 1 and either point of
the pair in X′. The proof's Type A case (d ∈ D ∩ int X′, so t = 0) already uses this definition.
With it the injection is sound:
- *Type A:* each d ∈ D charges at most one pair, at t = 0 if d ∈ int X′. Otherwise the pair is
  fixed by the first backward hit, T^(−σ′(d))(d).
- *Type B:* each b ∈ ∂X′ ∖ (D ∪ C) charges one pair. b ∉ C because the pair stayed adjacent
  through the previous step.

**Counted.**

| class | sections | violations | tight | the bound below Astra E's |
|---|--:|--:|--:|--:|
| Every dIET with n ≤ 7: every composition, every block order, a label per interval; every nonempty X′ | 1,603,027 | **0** | 23,149 | 1,574,308 |
| LF of every binary column to length 9 and ternary to length 7 (all three letters); every nonempty X′ | 617,116 | **0** | 12,118 | 587,406 |
| Word cylinders I(P), \|P\| = 1–3, on the same LF columns | 47,062 | **0** against t ≤ r | | |

The 7-cycle example checks by hand.
- *The map:* 0→5→3→1→6→4→2→0, with D = {2,3} and C = {1,5}.
- *After removing [2,3):* the pieces shift by +4, −1, −3 and −2, so r = 4. That meets
  3 + 1 + 0 with equality.

**How it relates to Astra E (record 77, §1).**
- *f2-bounds* proves t ≤ r + k − 1 + h, with h ≤ 2k counting every membership boundary.
  Theorem 1 is the same bound with h replaced by h′ = |∂X′ ∖ (D ∪ C)| ≤ h: a boundary already
  at a run seam or an image seam is free.
- *f2-dual's interval bound* is r + e, with e counting the endpoints inside an LF image (∉ C).
  Theorem 1 also frees an endpoint at a domain seam, and it extends the bound to c components.
- *The r + 2 witness:* every contiguous section attaining r + 2 has both endpoints outside D ∪ C,
  as Theorem 1 requires. That covers all 34 LF and 6 dIET cases here. f2-dual's own witness,
  L = 111000 with S = [1,5), has D = C = {3}, so Theorem 1 gives 2 + 2 = 4, its exact count.
- *What is new:* nothing in the count is new except the D-exclusion and the c-component form. The
  cylinder refinement is f2-bounds' t ≤ r, by the same argument.

## Theorem 2 (P, C)

**(1) Kac.** Σ_{x∈X₁} τ(x) = n_cov(X₁) is exact: the excursions partition every cycle that meets
X₁.

**Its scope is narrow.** It applies only to schedules that compute τ for every row of X₁ by
stepping T₀. It is not a lower bound on discovering T_{X₁}.
- f2-dual's reverse discovery finds the band boundaries of the frozen column's token section in
  65,678 probes, and labels them in 128,155 more, against n = 899,931.
- Cells reaches its sections in S = 92,570 interval steps.

**(2) Fibonacci.** With f₁ = 1, f₂ = 2 and F₁ = a, F₂ = ab, every BWT is two runs and one cycle.
I computed every prefix cylinder of F_q for q = 3–16: 4,162 prefix lengths.

**What holds, without exception:**
- *The range:* for 2 ≤ d ≤ q − 1, take any prefix P with f_d − 1 ≤ |P| ≤ f_{d+1} − 2.
- *The cylinder:* |I(P)| = f_{q−d+1}, with exactly two pieces.
- *The return times:* f_d and f_{d−1}, so W = f_{d+1}.
- *At d = q − 1:* |I(P)| = 2 and W = f_q = n.

**What the response states instead.**
- *Its prefix length:* f_d − 2, which is the top of the d − 1 range. That gives return times
  (f_{d−1}, f_{d−2}) and |I| = f_{q−d+2}. The stated pair (f_d, f_{d−1}) never occurs at that
  length, for any q ≤ 16.
- *Its range:* 1 ≤ d ≤ q − 2, where d = 1 is a negative length.
- *Its d = q − 2 example:* "(|X_d| = 2)" is wrong. Its own occurrence count f_{q−d+1} gives f₃ = 3
  rows there (the Coordinator's point), and its prefix length gives f₄ = 5.

**The separation survives the correction.**
- *Explicit:* the return words have length f_q = Θ(φ^q) explicitly.
- *Factored:* O(q) Rauzy–Veech subtractions, with one concatenation rule each.

This is the standard SLP compression of Fibonacci words, and it is correct.

**The "diagnostic consequence" paragraph is interpretation and unmeasured.** It covers the 2.36×
from skipping token levels and the 2.35× exception acquisition. Nothing in the theorem measures
either.

## Item 4: "Cells is strictly serial, MLP = 1" (C, I)

**Refuted for the shipped kernel** (`src/cells.rs` at master 0c852a07).
- *The macros:* `induce_cells` runs `ahead!` and `ahead_bottom!` at every fresh front pair, and
  again inside the step (lines 855–873, 887–888, 977, 996).
- *Why the addresses are known early:* each order's slots lie contiguously in its order, so the
  letter `FAR` = 12 slots before an order's last can be read from memory before the serial chain
  reaches it. There is no pointer chase.
- *What is fetched:* the macros hint the slot where that letter's cells will be written and its
  word. On the bottom order, the word `NEAR` = 6 slots on is read through the top slot fetched
  earlier.
- *The overlap:* up to four outstanding hints a cell, for letters a dozen steps ahead.
- *What it bought:* 9.8 → 7.2–7.7 ns a cell, about a quarter
  (`docs/journal/monodromy/2026-11-10-rauzy-veech.md`, §6.2 table).

**What is serial.** Only the length comparison (ra against rb) is, and it runs in registers.

**The lazy kernel** (`src/cells_lazy_induce.rs`, `src/cells_lazy.rs`) has no prefetch: by this
measure it is the one with MLP of 1.

**Interpretation, not measured: Theorem 3's room for gain is smaller than the response assumes.**
- *Against the shipped kernel:* it competes with a 12-deep fetch-ahead, not with MLP 1.
- *Instruction cost:* the cell is already about 64 instructions.
- *Width:* the frozen column offers a mean width under 4.

## What this decides

Nothing in production.
- *Theorem 3:* a correct, sharpened construction, and a measured ceiling of about 3.9-way.
- *Theorem 1:* a modest sharpening of Astra E's bound.
- *Theorem 2:* a corrected example.
- *Item 4:* a refuted premise.
