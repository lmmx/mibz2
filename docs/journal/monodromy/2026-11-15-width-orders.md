# 2026-11-15 · MONODROMY 7 · Width depends on the order of steps: eager free steps hold a third of the right end's live state, on the same Lyndon tree, bounded by the free closure, which looks like a normal form

Follows `2026-11-15-width.md`: the question there, its registered falsifier, and QUEUED
6900246233's hand-off condition.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## The question and its falsifier, as set before the data

The width W is the most live letters at once whose word is longer than one byte. The question
was whether W depends on the order of steps.

The falsifier (REVIEW 6900292266, before this count): if every order sits at 0.28 r or above,
width doesn't depend on order, and the lazy-letters hand-off is judged on the right end's
0.31 r alone.

## Answer (C): it does, by about 3×. The falsifier did not fire

**Method.** The archived schedules simulator
(`2026-11-10-rauzy-veech/schedules_geo.rs.txt`) was extended with the counters below and
three orders; the result is `width_orders.rs.txt`, here.
- **What it simulates:** one-cell right-handed Rauzy steps at any coincidence, with the rows
  never renumbered.
- **The check:** every final word is checked, by a 61-bit hash, to be a rotation of the block.
- **The counters:** at every step, the live letters with |w| > 1 (W), and the live letters
  touched in any way, whose length, place or word changed.

The second counter is the honest live state for an order that leaves remainders in the middle
of the domain. On every order it differs from W by under 0.01 r, median.

**Wikidata, all 233 blocks of at least 65,536 rows (per block ÷ r; full table in
`summary.txt`):**

| order | steps S | W min / median / max | W pooled | letters alive at W's peak |
|---|--:|--:|--:|--:|
| right end (Cells') | 1.837 | 0.270 / **0.318** / 0.361 | 0.317 | 0.451 |
| leftmost coincidence | 2.199 | 0.305 / 0.345 / 0.377 | 0.344 | 0.633 |
| rightmost and leftmost in turn | 1.965 | 0.308 / 0.337 / 0.359 | 0.336 | 0.556 |
| a coincidence after a pseudo-random row | 1.642 | 0.268 / 0.296 / 0.323 | 0.297 | 0.564 |
| largest cell (RePair's choice) | 1.468 | 0.251 / 0.270 / 0.302 | 0.271 | 0.443 |
| equal steps first, else the largest cell | 1.468 | 0.139 / 0.171 / 0.223 | 0.173 | 0.274 |
| **free steps first, else the right end** | 1.463 | 0.056 / **0.105** / 0.177 | 0.106 | 0.140 |
| **the same, free told from links alone** | 1.473 | 0.056 / **0.107** / 0.177 | 0.107 | 0.126 |
| free steps first, else the largest cell | 1.280 | 0.056 / 0.080 / 0.141 | 0.083 | 0.157 |
| free steps first, else the smallest cell | 2.263 | 0.056 / 0.108 / 0.177 | 0.106 | 0.122 |
| free steps first, else the leftmost | 1.669 | 0.060 / 0.124 / 0.191 | 0.124 | 0.151 |

Per block, right end ÷ (free first, told from links, else the right end): 2.04 / 2.98 / 4.84.

**Prose, 4 blocks:** right end 0.373 r; every free-first order 0.211 to 0.215 r.

The right end's exact W here (0.318 r median, 0.361 at most) is a little above the
1/32-grid figure of `2026-11-15-width.md` (0.311 r), which was a lower bound by construction.

## What drives it (C, and one Conj)

**The paid step's rule barely matters (C).**
- **Free first, whatever the paid step:** W lies in 0.080 to 0.124 r, while S ranges from
  1.28 to 2.26 r. "Free first, else the smallest cell" has 2.263 r steps and still W = 0.108 r.
- **Without free first:** every rule lies in 0.27 to 0.35 r.

**Equal steps alone recover part (C).** Equal steps first, else the largest cell: 0.171 r.

So width is set by how soon a letter is closed once it can be closed, not by how many steps
are taken. Both kinds of free step remove a boundary of the common refinement, as
`2026-11-10-rauzy-veech.md` §5.6, Theorem 4, shows: equal steps, and steps whose loser lands
on a boundary.

**The orders keep the tree (P).** Every order counted is a schedule of one-cell right-handed
steps, and §5.6 shows such schedules leave the grammar's words unchanged. They differ only in
when each rule is made and how long each letter lives.

**Conj, the link to the tunnels.** Taken to exhaustion, the free steps leave 0.176 r letters
(§5.6, 8 blocks). Under free-first orders, the letters alive at W's peak are 0.12 to 0.16 r. So
the eager width is about the size of the exchange left once the tunnels are contracted: the
(1, 1) chains, and Baier's and Alanko et al.'s tunnels. In the language of the thought
experiment, threads 1 and 3 meet: contracting the tunnels is what an eager free order does as
it goes.

The conjecture is W(free first) ≤ letters(after free closure) + O(paid transient). Its
falsifier is a block where the eager W exceeds the closure's letter count.

**The conjecture checked (P + C, added the same day).** The program now also records, at the
first paid step (when no free step is ready: the free closure), the letters alive, the
non-trivial ones and W so far. The output is `closure_wikidata.jsonl`, all 233 blocks, orders
0, 1 and 10.

| order | letters at the closure, ÷ r, median | W ÷ those letters, min / median / max | blocks with W above them |
|---|--:|--:|--:|
| free first, else the right end | 0.186 | 0.518 / 0.571 / 0.741 | 0 |
| free first, else the largest cell | 0.186 | 0.394 / 0.445 / 0.741 | 0 |
| free first, told from links, else the right end | 0.191 | 0.498 / 0.560 / 0.712 | 0 |

- **P, after the closure.** No step adds a letter: unequal steps keep the count, and equal
  steps remove one. So from the closure on, the live non-trivial letters are at most the
  letters alive at the closure.
- **C, before the closure.** The non-trivial letters never exceed that count either, on any
  block. W is reached before the closure on only 1 or 2 blocks.
- **So, on this corpus, W(free first) ≤ the closure's letters.** That is 0.186 r median,
  matching §5.6's 0.176 r on 8 blocks. The second half of the conjecture is proved; the first
  half is counted.
- **Confluence of the free closure: no counterexample (C), proof open (Conj).** Free steps
  (equal steps, and steps whose loser lands on a boundary, tested exactly) were run to
  exhaustion under several disciplines. Exhaustion was then checked by brute force over every
  coincidence.
  - **The disciplines:** LIFO, FIFO and seeded random choices of the next ready coincidence.
  - **The program:** `free_closure.rs.txt`, here.
  - **The resulting exchanges are compared exactly:** every live letter's top start, length,
    bottom start, word hash and word length.
  - **The one exception is a closed letter.** That is a cycle of the exchange closed whole,
    whose top interval is its own image. It is compared up to rotation, by its length and word
    length only: which of its symmetric cuts goes last moves its place and its word's starting
    point. `[1, 0]` is the smallest case: the closure ends at `ab` or `ba`.

  | input | disciplines | different closures |
  |---|---|--:|
  | all 233 Wikidata blocks, 4 prose blocks | 6 (LIFO, FIFO, 4 random) | 0 (equal exactly: no cycle closes) |
  | every column over 2 letters to length 16, 3 to 10, 4 to 8, 5 to 7: 404,677 columns | 10 (LIFO, FIFO, 8 random) | 0 |

  The step count to the closure is the same under every discipline too: 0.896 r median on
  Wikidata. The closure leaves 0.185 r letters (0.075 / 0.185 / 0.281 r, min / median / max).
  - **So, conjecturally, the free closure is a normal form of the block:** the exchange with
    every tunnel contracted. Its size and its step count are block invariants, whatever order
    a kernel takes its free steps in.
  - **Proof route (Conj).** Free steps terminate: each removes a boundary of the common
    refinement (§5.6, Theorem 4). By Newman's lemma, local confluence then suffices.
  - **Commuting pairs.** Two free steps on disjoint letters commute (§5.3).
  - **What must still be shown:**
    - the other step stays free afterwards: no boundary but the merged one ceases, so the
      only case is a step whose landing row is the other's coincidence;
    - the shared-letter cases join.

  Not done here.

**Which orders were not counted.**
- Left-end induction proper, with left-handed steps. §5.6 says it is the right end under the
  reversed alphabet order.
- Branching (two-sided, left- and right-handed) induction. The "two-sided" row above
  alternates rightmost and leftmost *right-handed* steps.
- Cylinder-first orders.

## What the space would cost (Conj: priced from counts, nothing built)

**What a lazy kernel would hold.** It would keep original letters implicit: their fields come
from the run records and a stable-sort index. It would materialise a letter at its first
change. Its live slot state is then:
- free first, told from links, else the right end: about 0.107 r letters;
- the right end: about 0.318 r;
- Cells today: r.

**The price is the worklist.** With free steps told from links alone, else the right end
(medians a run; geo's counters):
- 3.31 ends of segments tested, 1.21 of which found a free step;
- 0.55 parked to wait for a landing row;
- steps taken where the step before left off: 0.51 r of 1.47 r.

These are the costs §8.2 of the Rauzy–Veech entry priced at about 150 instructions a run,
against the sweep's 115, on one thread. Two things have changed since:
- the step count also falls, 1.837 → 1.473 r;
- the state that has to be held falls about 3×.

Whether that pays at ten workers sharing a last level is a clock question. As built today, the
right end's sweep is also what makes Cells' misses only first touches (#397), and an eager
order gives up that sweep.

**The hand-off, per QUEUED 6900246233.** The width depends on the order, so the lazy-letters
hand-off should carry both figures:
- with the right end kept: about 0.32 r live;
- with eager free steps, told from links alone: about 0.11 r live, and S 1.47 r instead of
  1.84 r, at the worklist's price.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-width-orders/width_orders.rs.txt examples/width_orders.rs
cargo run --release --example width_orders -- corpora/wikidata.bz2 1 65536 > wikidata.jsonl   # about 15 min
cargo run --release --example width_orders -- corpora/prose.bz2 1 65536 > prose.jsonl
python3 docs/journal/monodromy/2026-11-15-width-orders/summary.py.txt wikidata.jsonl
```

**The per-policy fields, per block.** Each policy prints two arrays.
- Geo's array: [steps, not reducing, tests, tests that found a free step, parked, steps where
  the last left off, paid steps].
- `<policy>_width`: [W, touched, letters alive at W's peak].

**What is beside this entry.**
- `wikidata.jsonl`, `prose.jsonl`: the outputs.
- `summary.txt`: the summaries.
- `width_orders.rs.txt`: the program, diffed from `schedules_geo.rs.txt`. It adds the counters
  (with the closure fields as the last three of `_width`) and three policies: 7 leftmost, 8
  rightmost and leftmost in turn, 9 a pseudo-random row (xorshift, fixed seed). `wikidata.jsonl`
  and `prose.jsonl` were made before the closure fields were added: their `_width` arrays have
  three entries.
- `closure_wikidata.jsonl`: orders 0, 1 and 10, with the closure fields.
- `free_closure.rs.txt`: the confluence test (`--small SIGMA MAX` for every short column;
  `FILE.bz2 EVERY` for the blocks; `--show 1,0` prints one column's closures).
  `confluence_wikidata.jsonl`, `confluence_prose.jsonl`, `confluence_small.txt`: its outputs.

**Corpora** (SHA-256 as in `2026-11-15-width.md`): `wikidata.bz2` `82f65ace…c29915`,
`prose.bz2` `c8f4299a…1833`.
