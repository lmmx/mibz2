# 2026-11-15 · MONODROMY 8 · The free closure is confluent: a normal form of the block, unique up to closed cycles, every maximal path the same length

Follows `2026-11-15-width-orders.md`. There, the free closure was conjectured confluent:
- no counterexample on 404,677 short columns under 10 disciplines, or on the 237 corpus blocks;
- the disjoint case proved;
- the cases where two steps share a letter left open.

This entry closes them.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## Setting

The state is that of the schedules simulator (`2026-11-10-rauzy-veech/schedules_geo.rs.txt`):
- **Letters.** Each letter X has a top interval I_X = [t_X, t_X + λ_X) and a bottom interval
  J_X = [b_X, b_X + λ_X). The top intervals tile the domain D, and so do the bottom ones.
- **Rows.** Rows are fixed. A removed cell is a hole in both tilings.
- **Coincidences.** A coincidence is a row c where a top interval (of A) and a bottom interval
  (of B ≠ A) both end. The right-handed step at c (§2 of the Rauzy–Veech entry) is one of three
  kinds:
  - **top wins**, λ_A > λ_B: the cell J_B = [c − λ_B, c) goes. A's top becomes
    [t_A, c − λ_B). B's word becomes w_A w_B, and B's bottom moves to the tail of J_A, at the
    landing row p = b_A + λ_A − λ_B. A's bottom becomes [b_A, p);
  - **bottom wins**, λ_A < λ_B: the mirror. A's word becomes w_A w_B, and A's top moves to the
    tail of I_B, at the landing row p = t_B + λ_B − λ_A;
  - **equal:** A is deleted, B's bottom takes A's bottom place, and B's word becomes w_A w_B.
- **Free steps.** A step is **free** if it is equal, or if its landing row is already a
  boundary of the other tiling: a top end for a top win, a bottom end for a bottom win. By
  §5.6, Theorem 4, these are exactly the steps that remove a boundary of the common refinement
  and add none.
- **Closed letters.** A letter with I_X = J_X is **closed**: a cycle of the exchange, closed
  whole. Its end row is no coincidence, because the only bottom interval ending there is its
  own. So it is inert: no later step reads or writes it.

## Theorem (P)

Run free steps in any order until none is left.
- **The final state is unique** up to where each closed letter lies and which rotation of its
  cyclic word it holds.
- **Every maximal sequence of free steps has the same length.**

So the free closure's letters, words (up to rotation, for closed ones) and step count are
invariants of the column.

On a BWT block whose one cycle does not close within the free steps, the closure is unique
exactly. That covers every corpus block: there it was exactly equal under every discipline,
with the step count 0.896 r at the median.

## Proof

Two facts, then four cases.

**Fact 1, boundaries.**
- *What a free step removes.* A free step at c₁ removes c₁ from the top ends and from the bottom
  ends, and removes no other end. The top end that moves (A's to c₁ − λ_B, or the loser's
  landing) lands on a row that was already a boundary. So afterwards the top ends contain
  every old top end except c₁, and likewise the bottom ends.
- *What it touches.* It touches only the fields of its own two letters.

**Fact 2, landing rows.**
- *Where they lie.* A top win's landing row lies strictly inside the winner's bottom interval,
  so it is no bottom end. A bottom win's lies strictly inside the winner's top interval, so it
  is no top end.
- *What follows.* The landing row of one step is never the other step's coincidence, which is
  both a top and a bottom end.

**The cases.** Let s₁ at c₁ (letters A₁ on top, B₁ below) and s₂ at c₂ ≠ c₁ (A₂, B₂) both be
free. A letter has one top end and one bottom end, so A₁ ≠ A₂ and B₁ ≠ B₂, and the letters
relate in exactly one of four ways.

**0. No common letter.** The lemma of `2026-11-15-width-orders.md`.
- Each step leaves the other's coincidence, letters and landing row in place (Facts 1 and 2),
  so the other stays free.
- The two orders give the same state, because each step reads and writes only its own two
  letters. It is a square.

**1. A₁ = B₂ = X, with B₁ = Y ≠ Z = A₂.** X ends on the top at c₁ and on the bottom at c₂. Write
x = λ_X, y = λ_Y, z = λ_Z, so that:
- I_X = [c₁ − x, c₁);
- J_Y = [c₁ − y, c₁);
- I_Z = [c₂ − z, c₂);
- J_X = [c₂ − x, c₂).

Number the kinds 0 top wins, 1 bottom wins, 2 equal, for (s₁, s₂). Four of the nine kinds
cannot both be free:

| kind | why both cannot be free |
|---|---|
| (1, 1) | s₂'s landing row c₁ − z lies inside J_Y (z < x < y), so it is no bottom end |
| (2, 1) | the same, with z < x = y |
| (0, 0) | s₁'s landing row c₂ − y lies inside I_Z (y < x < z), so it is no top end |
| (0, 2) | the same, with y < x = z |

The other five are squares. For each, both orders are written out below. Each order removes
the same rows, and leaves the same letters with the same places and words.

- **(1, 0):** X loses both steps, on opposite sides. Either order gives X the word w_Z w_X w_Y,
  its top at the tail of I_Y and its bottom at the tail of J_Z. The other step stays free by
  Facts 1 and 2.
- **(1, 2):**
  - *s₁ then s₂:* X takes w_X w_Y and moves to the tail of I_Y. Then Z is deleted, and X's
    bottom takes Z's place with the word w_Z w_X w_Y.
  - *s₂ then s₁:* the same moves in the other order.
- **(2, 0):**
  - *s₁ first:* X is deleted, and Y takes J_X with the word w_X w_Y. Then at c₂, (Z, Y) with
    z > y: Y takes w_Z w_X w_Y and moves to the tail of J_Z.
  - *s₂ first:* X takes w_Z w_X and moves to the tail of J_Z. Then at c₁, (X, Y) is equal: X is
    deleted, and Y takes X's new bottom place with w_Z w_X w_Y.
- **(2, 2):** either way Z and X are deleted, and Y takes Z's bottom place with w_Z w_X w_Y.
- **(0, 1):**
  - *Its kind forces z = y.* s₁ free needs c₂ − y to be a top end, so y ≥ z, since otherwise it
    lies inside I_Z. s₂ free needs c₁ − z to be a bottom end, so z ≥ y, since otherwise it lies
    inside J_Y.
  - *s₁ first:* X shrinks on both sides to x − y, and Y takes w_X w_Y and J_Y's new place
    [c₂ − y, c₂). Then at c₂, (Z, Y) is equal: Z is deleted, and Y takes Z's bottom place with
    w_Z w_X w_Y.
  - *s₂ first:* Z takes w_Z w_X and moves to [c₁ − z, c₁), and X shrinks to x − z. Then at c₁,
    (Z, Y) is equal: Z is deleted, and Y takes Z's bottom place with w_Z w_X w_Y.
  - *The result is the same:* with z = y, X ends as [c₁ − x, c₁ − y) on top and
    [c₂ − x, c₂ − y) below in both orders.

**2. B₁ = A₂.** Case 1 with the steps' names swapped.

**3. A₁ = B₂ = X and B₁ = A₂ = Y.**
- *The configuration:* I_X and J_Y end at c₁, and I_Y and J_X end at c₂. The kinds are forced,
  since x > y at c₁ means y < x at c₂: they are (0, 1), (1, 0) or (2, 2).
- *Either step closes a letter.* For example, s₁ with x > y: Y takes w_X w_Y and moves its
  bottom to [c₂ − y, c₂) = I_Y, so Y is closed. The other cases are alike.
- *The two orders.* s₂ instead closes Y (or X) with the rotated word w_Y w_X, the same length,
  and the place [c₁ − y, c₁). Everything else is identical: in both, X ends as
  [c₁ − x, c₁ − y) on top and [c₂ − x, c₂ − y) below.
- So the two results differ only by where the inert closed letter lies, and by its rotation.

**Conclusion.**
- *Termination:* every free step removes one boundary (§5.6, Theorem 4), so there are finitely
  many steps.
- *Local confluence:* by the cases, it holds modulo the closed letters, and the joins are one
  step on each side (cases 0 to 2) or none (case 3). Closed letters are inert, so the relation
  "equal up to closed letters' places and rotations" commutes with every step.
- *Uniqueness:* Newman's lemma on the quotient gives a unique normal form.
- *Equal lengths:* local confluence with equal-length joins gives every maximal path the same
  length. ∎

## Machine check (C)

`square.rs.txt` (here) tests each claim of the proof at every state, not only at the final one.
- **Coverage:** every column over 2 letters to length 14, 3 to 9, 4 to 7 and 5 to 6.
- **Paths:** three seeded random maximal free paths each.
- **What it checks:** at every state along every path, every ordered pair of free steps is
  classified by relation and kinds, and checked to be a square, or in case 3 to close a cycle
  and agree up to it.
- **Output:** `square_output.txt`.

| alphabet, max length | ordered pairs | squares | case 3 (closing, equal up to rotation) | otherwise |
|---|--:|--:|--:|--:|
| 2, 14 | 4,322,196 | 4,042,698 | 279,498 | 0 |
| 3, 9 | 3,434,430 | 3,129,050 | 305,380 | 0 |
| 4, 7 | 1,741,700 | 1,531,570 | 210,130 | 0 |
| 5, 6 | 1,164,480 | 991,358 | 173,122 | 0 |

The kinds that occur are exactly those the proof allows, in every alphabet:
- case 1: (0, 1), (1, 0), (1, 2), (2, 0), (2, 2);
- case 2: their mirror;
- case 3: (0, 1), (1, 0), (2, 2).

## What it means

**The tunnels' exchange is canonical.** The free closure is the exchange left once the tunnels
are contracted. It is now a well-defined object of the block.
- *Its size:* 0.185 r letters median on Wikidata (0.075 to 0.281 r).
- *Its cost:* 0.896 r steps.
- Both are fixed by the block, however a kernel orders its free steps.

**For width** (`2026-11-15-width-orders.md`):
- Every free-first order passes through this same exchange.
- After it, the live state is bounded by its size (P there).
- Before it, the bound was counted on every block (C there).

**For a kernel.** Free steps can be taken in whatever order is cheapest: a stack, a sweep or
batches. Independent free steps can also go to different threads. Case 0 says they commute;
cases 1 and 2 say they join in one step each.

**Open.**
- *An intrinsic description (Conj).* Unitig compaction of forced adjacencies would give the
  closure without simulating it.
- *Which letters survive?* A characterisation in terms of the BWT's runs, with its size as a
  repetitiveness measure to compare with r, z and δ.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-free-closure-confluence/square.rs.txt examples/square.rs
cargo run --release --example square -- 2 14   # also 3 9, 4 7, 5 6; minutes each
```
