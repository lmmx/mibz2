# 2026-11-16 · MONODROMY 35 · Free steps first is not step-optimal: on the binary column 110000111000 every free-first order takes 8 steps and the least is 7. The free closure is unique, but the least step count does not always pass through it. Free-first then the largest cell is up to 2 steps above the least on whole runs

Record 33's conjecture, refined after its correction. Is some free-first order always
step-optimal? Here every order means any coincidence; a free-first order is any choice among the
free steps, with any paid step when none is free.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## 1. The counts (C)

`mintail.rs.txt`, here, takes every column with at least 3 runs and computes:
- the least steps over every order;
- free-first then the largest cell;
- the least over every free-first order, by memoised search.

| columns | free-first, then the largest cell, above the least | **no free-first order optimal** |
|---|--:|--:|
| σ = 2, n ≤ 9 | 0 | 0 |
| σ = 2, n = 10 (1,004) | 1 (worst gap 1) | 0 |
| σ = 2, n = 11 (2,026) | 7 (worst gap **2**) | 0 |
| σ = 2, n = 12 (4,072) | 33 (worst gap **2**) | **1** (gap 1) |
| σ = 2, n = 13 (8,166) | 104 | **2** |
| σ = 3, n ≤ 9 (19,632 at n = 9) | 15 at n = 9; 2 at n = 8 (worst gap 1) | 0 |
| σ = 4, n ≤ 7 (16,308 at n = 7) | 0 | 0 |

**Two different failures:**
- *Choice:* at n = 10 and 11, free-first then the largest cell loses only through its choice among
  free steps, or its paid fallback. Another free-first order is optimal.
- *Principle:* from n = 12 there are columns where no free-first order is optimal. The principle
  itself fails.

## 2. The smallest counterexample (C, exact)

**The column.** 1 1 0 0 0 0 1 1 1 0 0 0 (σ = 2, n = 12), with a least of 7 steps; every free-first
order takes 8. Steps are written kind@coincidence[λ_A, λ_B]: T top wins, B bottom wins, E equal.

| | steps |
|---|---|
| optimal (7) | T@9[3, 2] · B@7[1, 3] · T@6[4, 2] free · E@2[2, 2] free · E@4[2, 2] free · B@12[1, 2] free · E@10[1, 1] free |
| free-first, then the largest cell (8) | E@12[3, 3] free · T@9[3, 2] · B@7[1, 2] · T@6[4, 1] · T@5[3, 1] free · B@2[1, 2] free · E@3[1, 1] free · E@1[1, 1] free |

**Where the step goes.**
- *Free-first's first move:* at the start the equal step at 12 (λ 3 = 3) is free, and every
  free-first order must take a free step there or elsewhere first.
- *The optimal order's first move:* it takes a paid step at 9 first. That leaves 12's letters
  unequal, so the step there later is a bottom-wins step whose loser lands on a boundary: free,
  and part of a cascade in which every later step is free.
- *What the free equal step at 12 costs:* taken first, it merges those two letters for good, and
  one more paid step is then needed later (T@6 and T@5 against T@6 alone).

**What that means.** A free step lowers the boundary count by one; it does not lower the paid steps
still needed. The step count is floor + paid, and a free step can raise the paid term that is left.
The free closure is still unique (#438): what fails is the step-optimality of passing through it.

## 3. What stands, and what it changes

**How near (C; `worst_gap_small.txt`).**
- *Free-first then the largest cell:* up to 2 steps above the least on whole runs (σ = 2, n = 11
  and 12), and 1 at σ = 3, n = 8.
- *The best free-first order:* 1 step above on the one n = 12 column where it fails. The gaps of
  the two n = 13 failures were not recorded.
- *Record 33's tail figures* (worst gap 1 after a right-end prefix) are a different measure and
  stand.

A first draft of this paragraph said "within one step" for free-first then the largest cell. The
n = 11 example in this program's own output (least 7, it takes 9) refuted that; corrected before
posting.

**For the decoder, nothing changes.**
- *The order switch:* record 32's tail order is near-optimal on this evidence. The compact tail
  that used it is parked (record 34).
- *Where the bound applies:* on real blocks the gap between free-first then the largest cell
  (1.28 r) and the least is unmeasured, and bounded on small columns by two steps a column.

**Open (Conj).** Is the least step count within a constant of free-first on every column, or does
the gap grow with n? The counts so far are 0 at n ≤ 9, 1 at n = 12 and 2 at n = 13. A family showing
growth would be the next refutation to look for.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-free-first/mintail.rs.txt examples/mintail.rs
cargo run --release --example mintail -- 2 13   # also 3 9 and 4 7; minutes to an hour
```

The output is `free_first_small.txt`. The σ = 2, n = 14 row was still running at the commit.
