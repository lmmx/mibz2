# 2026-11-16 · MONODROMY 33 · How far can a tail order go? On every small column counted, free-first is step-optimal in the tail, and within 0.01 steps of optimal on whole runs. For the compact tail, whose joins cost 4 B each, steps are bytes, so record 32's tail order leaves nothing on the table

Record 32's next item: bound what any tail order can save beyond free-first then the largest
cell.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## 1. The exact optimum on small columns (C)

**The program.** `mintail.rs.txt`, here, from record 16's `minwidth`. It takes every column over σ
letters of length n with at least 3 runs, and:
- runs the right end for half its steps (the switch);
- computes the least number of steps to finish, over every order, by memoised search over states;
- compares it with free-first then the largest cell, free-first then the right end, and the right
  end.

The whole run is compared too. Output: `mintail_small.txt`.

| columns | tail: free-first, then largest cell, optimal on | tail: right end, worst gap | whole run: mean steps, right end / free-first-largest / least |
|---|--:|--:|---|
| σ = 2, n = 12 (4,072) | **all** | 1 | 7.460 / 7.130 / 7.121 |
| σ = 3, n = 9 (19,632) | **all** | 1 | 5.637 / 5.517 / 5.516 |
| σ = 4, n = 7 (16,308) | **all** | 0 | 4.131 / 4.093 / 4.093 |

**Every smaller n is the same:** free-first is optimal in the tail on every column. Free-first
then the right end ties free-first then the largest cell in every tail counted.

**Over whole runs,** free-first then the largest cell is within 0.009 steps of the least on
average (σ = 2, n = 12), and equal for σ = 4.

## 2. What it says

**The conjecture (Conj, on the evidence above).**
- *Free-first is step-optimal* in the tail after any right-end prefix, and within a fraction of a
  step of optimal from the start.
- *The reason it should hold:* free steps commute with every disjoint step and stay free (case 0
  of #438's confluence proof). A free step lowers the refinement's boundary count by one, and the
  step count is the floor plus the paid steps. So taking a free step first should never add a paid
  step later.
- *Not proved:* a free step and a paid step on shared letters.

**For the compact tail, steps are bytes.**
- *Why:* every tail join costs 4 B whatever its word's length (record 27). So the step-optimal tail
  order is also the byte-optimal one.
- *So record 32's order leaves nothing:* free-first then the largest cell, 0.311 r tail cells at a
  1.2 r switch, leaves essentially nothing for a better tail order to take.
- *What remains for memory* is the prefix before the switch, where the streams bind (record 29:
  free-first at the fronts saves 15% there, against the right end).

**On whole runs, the 1.28 r of free-first then the largest cell** (the Rauzy–Veech entry §5.6;
record 16) is, by this count, essentially the least step count: the "1.28 r against 1.81 r" gap
is the whole of what ordering can give in steps.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-tail-bound/mintail.rs.txt examples/mintail.rs
cargo run --release --example mintail -- 2 12 0.5   # also 3 9 and 4 7; minutes
```
