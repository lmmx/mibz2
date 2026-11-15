# 2026-11-15 · MONODROMY 16 · The least live state: free-steps-first is beatable; greedy on the change in width holds 0.058 r on Wikidata, 46% below free-first, at 19% more steps

Invitation: QUEUED 6903426903. What is the least state any order of the induction must hold,
and which orders reach it? The i9 run (LOG 6903318530) makes the induction's peak live state
the lever at 8 to 10 threads.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

**Measures.**
- **W:** the most live letters at once whose word is longer than one byte.
- **Touched:** the most live letters at once whose length, place or word has changed. This is
  what a lazy kernel must hold.

**Orders.** One-cell right-handed Rauzy steps at any coincidence, as in
`2026-11-15-width-orders.md`. All of them give the same Lyndon tree.

## 1. Free-steps-first is not optimal (C, exact, on short columns)

`minwidth.rs.txt`, here, computes the exact least peak over every order of steps: a
memoised bottleneck search over states, which is acyclic because every step removes points. It
compares that with three orders:
- free steps first, else the right end;
- the right end;
- a greedy order: the least change in the live measure, then free, then rightmost.

The change a step makes is one of −1, 0 or +1, read off its two letters:
- the loser's word becoming non-trivial adds one;
- an equal step deleting a non-trivial A removes one;
- for touched, the winner's length changing also counts.

**W:**

| columns | count | free-first optimal | free-first worst gap | right end worst gap | greedy optimal | greedy worst gap | mean peak: optimum / free-first / right end / greedy |
|---|--:|--:|--:|--:|--:|--:|--:|
| binary, n = 12 | 4,094 | 43% | 4 | 3 | 83% | 2 | 1.85 / 2.58 / 2.45 / 2.02 |
| ternary, n = 8 | 6,558 | 55% | 3 | 3 | 92% | 1 | 1.51 / 2.03 / 1.92 / 1.59 |

Every length, and the touched measure, are in `minwidth_small.txt`. Touched behaves the same.

**So free-first is beatable,** and on short columns it is often worse than the right end. The
greedy order on the change in width comes within 1 or 2 of the optimum everywhere counted.

## 2. On real blocks (C, all 233 Wikidata blocks; 4 prose blocks)

`greedyw.rs.txt`, here, is the width-orders simulator with greedy orders driven by a lazy queue.
- **Its keys:** (change in W, not free, a tie-break).
- **Its re-keying:** after each step it re-keys the ends of the step's two letters, the fresh
  rows, and the coincidences waiting on them.
- **Its check:** every final word is checked to be a rotation of the block.

| order (Wikidata) | steps ÷ r | W ÷ r | touched ÷ r |
|---|--:|--:|--:|
| right end (Cells') | 1.291 / 1.837 / 2.082 | 0.270 / 0.318 / 0.361 | 0.270 / 0.318 / 0.361 |
| free-first, told from links, else the right end (the lazy kernel's candidate) | 1.185 / 1.473 / 1.643 | 0.056 / 0.107 / 0.177 | 0.065 / 0.107 / 0.177 |
| free-first, else the largest cell | 1.137 / 1.280 / 1.382 | 0.056 / 0.080 / 0.141 | 0.065 / 0.081 / 0.143 |
| (free, change in W, rightmost) | 1.253 / 1.499 / 1.683 | 0.056 / 0.077 / 0.127 | 0.057 / 0.079 / 0.129 |
| **(free, change in W), six LIFO buckets** | 1.313 / 1.564 / 1.809 | 0.057 / **0.077** / 0.127 | 0.059 / 0.078 / 0.129 |
| **(change in W, free, the largest cell)** | 1.464 / 1.745 / 1.981 | 0.040 / **0.058** / 0.092 | 0.048 / 0.065 / 0.094 |
| (change in W, free), six LIFO buckets | 1.542 / 1.887 / 2.160 | 0.047 / 0.060 / 0.101 | 0.048 / 0.064 / 0.106 |

Each cell is min / median / max.

| order (prose, 4 blocks; medians) | steps ÷ r | W ÷ r | touched ÷ r |
|---|--:|--:|--:|
| right end | 1.839 | 0.373 | 0.373 |
| free-first, told from links | 1.619 | 0.215 | 0.220 |
| (free, change in W), LIFO buckets | 1.713 | 0.208 | 0.216 |
| (change in W, free, the largest cell) | 1.834 | **0.116** | **0.148** |

**What this says.**
- **The trade.** Putting the change in width first takes the live state from free-first's
  0.107 r to 0.058 r on Wikidata (−46%), and from 0.215 r to 0.116 r on prose. It costs steps:
  1.75 r against 1.47 r.
- **The cheap point.** (free, change in W) keeps free-first's priority and only breaks ties by
  the change in width. Its live state is 0.077 r, 28% below free-first's, for 6% more steps.
  - *No priority queue:* its key takes six values, so six LIFO stacks serve it. The rightmost
    tie-break bought nothing measurable: 0.077 either way.
- **The free closure is no floor (P, by these counts).** The closure bounds free-first's width
  (`2026-11-15-width-orders.md`), but the change-in-width-first orders hold less than the
  closure's 0.185 r letters, because they never pass through the closure.

## 3. Lower bounds: none on real blocks, and why

**What holds (P).** No step adds a letter, and the live non-trivial letters change by −1, 0 or
+1 a step. So W counts letters made non-trivial and not yet deleted. Every letter but the last
is deleted exactly once, by an equal step as its A.

**What a lower bound needs.** An argument that some letters must be made non-trivial before
others can be deleted. That is a pebbling problem on the derivation's classes. Pebbling bounds
are known for trees (Strahler-type) and are hard for DAGs. Here the moves are constrained by
the exchange's geometry, not just by the tree.
- *I have no bound that is not trivial.*
- *The short-column optima (§1) are the only exact reference.* There the greedy order is within
  2 of optimal, and its mean is 9% above the optimum's (binary, n = 12).

**Open (Conj).** On Wikidata the least peak is below 0.058 r and above 0. Is it Θ(r) at all, or
is it governed by something like the depth of the Lyndon tree? The short-column optima grow
slowly with n: a mean of 1.85 at n = 12.

## 4. How few bits the live set needs (Conj, pointers)

- **Equal words.** Live letters carrying one word have adjacent images (`2026-11-15-equal-words.md`).
  The ring of 8 recent words shares about 32% of the word store, and applies to the live set
  unchanged.
- **Original letters.** These are derivable from the run records (R1 to R3). The lazy kernel
  already keeps them implicit.
- **Nothing new found here.** The live letters' fields are a length, a word reference and two
  order links; none was found derivable beyond those. (`2026-11-15-elision-bounds.md`'s
  derivable rules were for the slots, not the live set.)

## For the Pathfinder

Two orders sit on the front between steps and live state, beside the free-first order now being
priced in the lazy kernel (6903395401):
- **(free, change in W) on six LIFO buckets.** The same machinery as free-first: the key is a
  free bit and the change, −1, 0 or +1, read from the step's two letters. 28% less live state
  for about 6% more steps (1.56 r against 1.47 r).
- **(change in W, free, the largest cell).** 46% less live state than free-first for 19% more
  steps (1.75 r). Picking the largest cell needs a priority within each bucket.

Which pays at 10 threads is a clock question: live state against steps. The i9 run says live
state is the lever there.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-least-width/minwidth.rs.txt examples/minwidth.rs
cargo run --release --example minwidth -- 2 12   # also 3 8; minutes
cp docs/journal/monodromy/2026-11-15-least-width/greedyw.rs.txt examples/greedyw.rs
cargo run --release --example greedyw -- corpora/wikidata.bz2 1 65536 1,6,10   # about 3 min
```

**The outputs.** `minwidth_small.txt`; `greedyw_wikidata.jsonl` and `greedyw_prose.jsonl`.
- `greedy{v}` = [steps, W, touched, letters at W's peak]. The variants are 1 (free, change,
  rightmost), 3 (change, free, largest cell), 4 (free, change, LIFO) and 5 (change, free, LIFO).
- The baseline policies are those of `width_orders` (1, 6 and 10).
