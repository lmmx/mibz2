# 2026-11-14 · MONODROMY 4 · The offline locality bound for Lyndon-tree-preserving induction orders: CLOSED

Assignment: QUEUED 6883303780 on #145, the C1a falsifier registered in LOG 6883302716 before
any data. Untimed: traces and an LRU line model, no kernel, no timing. Labels: **P** proved ·
**C** counted · **Conj** conjecture. Method, models, verification and limits:
`2026-11-14-locality-bound/method.md`; per-block results and sources beside it.

## The rule, as registered

G = misses(B) / misses(R) at 5 MiB, n-weighted over the census cohort (the 196 admitted
Wikidata blocks of `research/cells-census` b5617a8). R is the right-end order as shipped; B the
best legal order found offline that keeps the Lyndon tree. WORTH AN ONLINE STEELMAN if
G ≤ 0.70; CLOSED if G ≥ 0.85; PARKED between.

## Result: G = 0.9999986. CLOSED, and no order could do otherwise

| misses per output byte (64 B lines, LRU, cold per block) | 1.25 MiB | 5 MiB | 2 MiB |
|---|--:|--:|--:|
| R, as shipped | 0.08326 | 0.07124 | 0.07477 |
| compulsory (first touch): the same for every order | 0.07124 | 0.07124 | 0.07124 |
| B, greedy over the ready steps, window 64 | 0.08261 | 0.07124 | 0.07430 |
| **G** (fixed layout) | 0.9922 | **0.9999986** | 0.9937 |
| G (layout re-allocated in B's order) | 0.9917 | 0.9999990 | 0.9939 |
| **floor on G for any order** = compulsory / R | 0.8557 | 0.9999986 | 0.9528 |

- **P:** with the layout fixed, every order touches the same set of lines, so no order misses
  fewer times than that set's size. That floor is above 0.85 at all three capacities: the
  verdict is CLOSED whatever order is found.
- **C:** at 5 MiB, R already misses only on first touch on 195 of 196 blocks. A block's
  induction footprint is 3.24 / 3.79 / 5.29 MiB (min, median, max); the one block over 5 MiB
  (block 158) has 18 extra misses, which B removes.

## Which orders keep the tree (C on small words; Conj in general)

Every legal schedule of right-handed steps was enumerated on all primitive words over 2 letters
to length 10 (1,964 words, 212,150,400 schedules) and 3 letters to length 7 (3,222 words,
3,666,028). The schedules that give R's derivation tree are exactly the linear extensions of
R's step dependency graph (each step after the last earlier step on either of its letters):
10,435 = 10,435 and 8,207 = 8,207, equal as sets on every word. On the real blocks each B was
checked directly, not through this claim: it is a linear extension and keeps every byte-exact
read/write hazard of R, so the slots, arena and rules end byte-identical (196 × 3 orders at
window 64, 25 × 2 at window 4,096; no failure).

## Cross-checks

- The traces come from a port of master's `slots` and `induce_cells` with the kernel's exact
  layout; records, letters and cells equal the census on all 196 blocks (20,456,299 cells).
- Against the memtrace census (`research/cells-memory` 95b98eda, SLOT 24): stores 182,381,648
  against 182,381,648; distinct induction lines 14,540,786 against 14,539,858; 2 MiB misses
  after `slots` 0.985 of memtrace's.
- The LRU model matches a hand-computed trace and a naive LRU on 60,000 random accesses.

## What it says

Reordering cannot reach the bytes: within one block, at a worker's L3 share, the induction
already misses only on its first touch of each line, and every Lyndon-tree-preserving order
touches the same lines. The lever is the footprint itself, the number of distinct lines, which
is what the slot diet, elision and a shared working set act on. Under the registered rule,
locality scheduling for case (a) goes on the Rejected list.

## Limits

- The cache model is fully associative LRU, cold at each block, one block alone, buffer base
  64-byte aligned. It does not model several workers sharing L3 or the buffer staying warm from
  block to block.
- The greedy's window was 64 on all blocks and 4,096 only on every 8th block at 1.25 and 2 MiB,
  where it did worse than R (it postpones steps that bring a new line). No exact search on
  truncated blocks: the compulsory floor stands in for it, and is attained at 5 MiB.
- Case (b), a different grammar, was not in scope. Wikidata only.
