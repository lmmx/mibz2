# 2026-11-15 · MONODROMY 12 · The free closure's structure: no local rule decides which stretches it closes; it is 42 rounds deep (median), half its steps in 2; the paid remainder; and the lower-bound thread, closed

The two open threads named in QUEUED 6901607680:
1. which equal steps need a paid step first, through the Cartesian tree of stretch minima;
2. the lower-bound note.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

**Inputs.** Wikidata, `corpora/wikidata.bz2` (SHA-256 `82f65ace…c29915`). Its 233 blocks of at
least 65,536 rows; every 8th (30 blocks) where marked.

**The program.** `ctree.rs.txt`, here, is the confluence entry's `free_closure` with five
additions:
- it records the rows each equal step removes;
- it computes the stretches of LF's cycle and their Cartesian tree;
- it counts the closure's dependency depth;
- it runs the closure in BSP rounds;
- it continues past the closure with one paid step at a time.

## 1. Which stretches the closure closes (C): no rule in a stretch's shape or its place in the tree

**How a stretch counts as closed.** Its least row was removed by an equal step of the free
closure. Over all 233 blocks this accounts for 0.984 of the closure's equal steps. An equal
step whose cell has several rows can hold the minima of several stretches, which is the rest.

The stretches are classed by:
- **shape:** a single row (p = s = 1); monotone (p = 1 or s = 1); other;
- **children:** their number of children in the Cartesian tree of stretch minima in cycle order
  (§5.6, Theorem 3 of the Rauzy–Veech entry).

| shape, children | stretches (233 blocks) | share closed by the free closure |
|---|--:|--:|
| single row, 0 | 4,411,483 | 0.938 |
| single row, 1 | 4,310,763 | 0.955 |
| single row, 2 | 3,083,257 | 0.973 |
| monotone, 0 | 285,654 | 0.303 |
| monotone, 1 | 653,347 | 0.263 |
| monotone, 2 | 890,781 | 0.323 |
| other, 0 | 132,991 | 0.281 |
| other, 1 | 517,469 | 0.193 |
| other, 2 | 855,857 | 0.274 |

**Every class is closed in part, so the closure is not decided by these features.**

**A correction to the thought experiment's working assumption.** A single-row stretch is a
coincidence, but its equal step also needs the two letters there to have equal lengths. So 3
to 6% of single-row stretches are not closed by free steps alone.

Together with `2026-11-15-compaction-rounds.md` §3 (the closure's equal steps lie strictly
between two per-stretch counts), this closes the search for a local rule. What decides is
global: which coincidences become equal through others' steps. The closure is still well
defined and unique (`2026-11-15-free-closure-confluence.md`). It just has no shortcut
description in these terms.

## 2. How deep the closure is (C)

**A step-by-step dependency depth is too conservative.** Chaining every step to the last step
on either of its letters, or on its rows, gives a depth of 21,568 / 25,532 / 104,468 (233
blocks). That is about half the step count, because a long-lived letter's top-end and
bottom-end steps are chained together although they are independent.

**BSP rounds, the honest measure.**
- *A round:* a maximal set of currently free steps on pairwise disjoint letters, taken greedily
  in row order. Such steps commute and stay free (case 0 of the confluence proof). The program
  asserts this on every step of every round, and it never fired.
- *How many (30 blocks):*

  | | min | median | max |
  |---|--:|--:|--:|
  | rounds to exhaust the closure | 20 | 42.5 | 168 |
  | free steps a round, average | 383 | 1,158 | 2,426 |
  | rounds to take half the closure's steps | 2 | 2 | 2 |

So the free phase, 0.85 to 0.94 r steps and about half of everything, is shallow. It is
comparable in depth to the 93 rounds of §5.4 of the Rauzy–Veech entry, and its steps are all
free.

## 3. The paid remainder (C, 30 blocks)

After the closure, the program repeats: one paid step at the rightmost coincidence, then free
BSP rounds until none is free.

| ÷ r, min / median / max | |
|---|--:|
| steps in the closure | 0.854 / 0.896 / 0.939 |
| paid steps after it | 0.162 / 0.213 / 0.270 |
| free steps after it | 0.248 / 0.327 / 0.390 |
| all steps | 1.363 / 1.437 / 1.495 (the right end: 1.837) |
| depth: closure rounds, plus each paid step and the rounds after it | 0.278 / 0.361 / 0.435 |

- **The schedule is cheap in steps:** 1.44 r. Interleaving free closures with right-end paid
  steps does nearly as well as "free first, else the right end" (1.46 r,
  `2026-11-15-width-orders.md`).
- **It is deep:** 0.21 r paid steps taken one at a time.
- **So the parallel value is in the free phase alone.** For one block on several cores, §5.4's
  all-steps rounds remain the shallower order. The free phase adds an order-free, shallow first
  half, consistent with them.

## 4. The lower-bound thread: closed, nothing worth a note

**What holds is either known or trivial.**
- *(P, trivial)* An interval-exchange presentation of LF needs at least as many intervals as LF
  has discontinuities. That is the runs, r of them.
- *(K)* The surface view of §9 of the Rauzy–Veech entry (genus about r/2) says only the same in
  other words: any presentation inside the interval-exchange class has about r pieces.
- *(Conj)* A presentation below r must leave that class: tunnels, or the forest. Each of those
  was priced already (§6.4 there; `2026-11-15-closure-intrinsic.md`).

**What would matter is out of reach of a short note.**
- *Time against space.* A lower bound in terms of r, with n ≫ r, for inversion with o(r) words
  of working memory, or with fewer than r random accesses. The permuting reduction of the
  thought experiment needs inputs whose r is about n, so it says nothing when r ≪ n.
- *The question is open, and no cheap falsifier presents itself.* Without one, it is closed
  here rather than left as an expired lead.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-closure-structure/ctree.rs.txt examples/ctree.rs
cargo run --release --example ctree -- corpora/wikidata.bz2 8   # §§2 and 3 in full; about 5 min
cargo run --release --example ctree -- corpora/wikidata.bz2 1   # needs the same program: slow
```

**The outputs.**
- `ctree_wikidata_all_closure.jsonl`: all 233 blocks, from the program before the BSP and
  remainder fields were added. It has §1's tallies and the step-by-step depth.
- `ctree_wikidata_every8.jsonl`: every 8th block, with every field.

**Their fields.**
- `by_shape`: per class, [stretches, closed].
- `bsp_rounds`, `rounds_to_half`: the closure's rounds.
- `remainder_paid_free_depth`: [paid steps, free steps after the closure, depth].
- `free_depth`: the conservative step-by-step depth.
