# 2026-11-15 · MONODROMY 10 · Iterated forest compaction on pieces: 1.75 r pieces and 0.76 r cells of work in 9 to 15 parallel rounds; and the free closure is not a per-stretch property

Follows `2026-11-15-closure-intrinsic.md`, whose side finding was that iterated compaction
collapses a block in 5 to 18 rounds. As counted there, a round walked every row it removed:
depth, not work. Here the work is counted on pieces.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

**Inputs.** Wikidata: all 233 blocks of at least 65,536 rows of `corpora/wikidata.bz2` (SHA-256
`82f65ace…c29915`). Prose: 4 blocks of `corpora/prose.bz2` (`c8f4299a…1833`).

## 1. A round on pieces

**The state.** A permutation σ of m points, in compressed coordinates: the first return of LF
to what is kept. Each point carries the word of its return.
- A **piece** is a maximal stretch of consecutive points on which σ is a translation and the
  word is one.
- A piece's **cells** are the pieces its image meets.

**Variant 0's round.** Remove every piece whose image lies inside one piece (a unique
successor). Then take the first return to the rest.

**Lemma (P).** In a variant-0 round:
1. The new pieces inside a kept piece P number at most the cells of P.
2. So the round's new pieces number at most the cells leaving the kept pieces.

*Proof.* Take a point range of P that maps into one piece X.
- *If X is kept*, the range returns there.
- *If X is removed*, X's whole image lies inside one piece Y (that is why X was removed), and σ
  is a translation on X. So the range maps on, contiguous and in order, into Y. Repeat while
  the piece is removed.
- *The chain cannot cycle.* A cycle of removed pieces would be invariant. A one-cycle σ with
  kept points has none.
- *So* the range returns as one contiguous, translated range with one word: the words along
  the chain, concatenated. P is cut only where its first image crosses a piece boundary. ∎

**What a round costs (Conj on the constant: no kernel built).**
- *The work:* one pass over the pieces and their cells, plus each removed piece's chain.
  Memoised, each removed piece is resolved once, by pointer jumping on (destination, offset,
  word).
- *The grammar:* the words are made as rules, one per concatenation, as Cells' are.
- *Parallelism:* within a round every piece is independent.

**Checked (C).** On every round of every Wikidata block (2,483 rounds), the next round's pieces
are at most the cells counted. They are equal on 495 rounds; on the rest, adjacent new pieces
with equal words and consecutive images merge, by a hash, in the counting program.

## 2. Counts (C)

`fixwork.rs.txt`, here, is `2026-11-15-closure-intrinsic/fixpoint.rs.txt` with a per-round
count of pieces and cells.

| per block ÷ r, min / median / max | Wikidata (233 blocks) | prose (4 blocks) |
|---|--:|--:|
| variant 0: pieces summed over the rounds | 1.461 / **1.754** / 2.038 | 2.053 / 2.077 / 2.142 |
| variant 0: cells summed over the rounds | 0.463 / **0.762** / 1.054 | 1.077 / 1.102 / 1.169 |
| variant 0: rounds | 9 / **11** / 15 | 8 / 9.5 / 10 |
| variant 0: letters left at the fixpoint | 2 / 6 / 72 | 6 / 16 / 42 |
| variant 2 (unique successor or predecessor): pieces summed | 1.297 / 1.466 / 1.609 | 1.724 / 1.733 / 1.758 |
| variant 2: rounds | 4 / 6 / 7 | 5 / 5 / 9 |
| variant 2: letters left ÷ r | 0.000 / 0.000 / 0.024 | about 0 |

**The rounds shrink fast.** Block 0, variant 0, the pieces each round ÷ r: 1.000, 0.286, 0.142,
0.101, 0.083, 0.061, 0.039, 0.025, 0.013, 0.007, 0.002, then 0.

**The comparison (C, from the Rauzy–Veech entry §5).**
- *Cells:* 1.837 r steps on Wikidata. A step is cheap, but the steps form chains 3,861 deep at
  the median, and the dependency order is 93 rounds.
- *Variant 0:* about 1.75 r pieces plus 0.76 r cells of work, in about 11 rounds. Every round
  is a data-parallel pass over arrays of pieces.
- *What remains after the fixpoint:* at most 72 letters, an exchange of a few letters to finish
  directly.

**Why §5.4's rounds lost, and why this is a different construction.** The rounds closed in §5.4
(1.99× on one thread) were independent sets of segments, with every letter read again in every
round: 2.4 to 3.8 r letter-rounds in all. Here a piece removed in a round is never read again,
so the rounds' work sums to under 2.5 r.

**Conj, for the Pathfinder: one block on several cores.** This is a structural reason to revisit
it, which Cooperative Cells' closure did not have: about 11 rounds of depth and work linear in r.
- *Its falsifiers:*
  - the per-piece constant: a round reads a piece, its cells and its chain's memo;
  - the cost of building each round's arrays;
  - the barrier between rounds.

  Each would need to stay within the slack that Cells' one sequential chain leaves on idle
  cores.
- *Its context:* it matters only where a block lacks a core of its own, that is above 10
  threads on the i9, which is where the teams take over.

## 3. The free closure is not a per-stretch property (C)

`2026-11-15-closure-intrinsic.md` §2 put the closure's size at 1 + (the equal steps that need a
paid step first). By §5.5 of the Rauzy–Veech entry, each equal step closes one stretch: the
piece of LF's cycle between consecutive run starts.

`stretches.rs.txt`, here, cuts the cycle and tallies each stretch's length and its records p
(minima so far, forward) and s (backward).

| ÷ r, min / median / max (233 blocks) | |
|---|--:|
| equal steps inside the free closure (r − its letters) | 0.7194 / 0.8155 / 0.9252 |
| single-row stretches (p = s = 1) | 0.7027 / 0.7773 / 0.8670 |
| single-row plus monotone (p = 1 or s = 1) | 0.8696 / 0.8972 / 0.9606 |

**The closure's equal steps lie strictly between those two counts on every block, and equal
neither on any.**
- *Some longer stretches close for free:* 0.012 to 0.061 r more than the single-row ones.
- *Not every monotone stretch does.*
- *So reaching the closure is not a property of a stretch's own rank shape.* Stretches unlock
  one another through shared letters, as the confluence proof's shared-letter cases already
  suggest.

**Open (Conj):** the dependency between stretches. Which stretch's equal step waits on which
paid step? Graph that dependency; ask whether the closure is its set of nodes with no paid
ancestor.

## Reproduce

```
for f in fixwork stretches; do
  cp docs/journal/monodromy/2026-11-15-compaction-rounds/$f.rs.txt examples/$f.rs
done
cargo run --release --example fixwork   -- corpora/wikidata.bz2 1   # also corpora/prose.bz2
cargo run --release --example stretches -- corpora/wikidata.bz2
```

The outputs are beside this entry: `fixwork_wikidata.jsonl`, `fixwork_prose.jsonl` and
`stretches_wikidata.jsonl`. Per variant, `fixwork` prints [letters left, rounds, pieces summed];
per round, the pieces; and, for variant 0, the cells bound's check and the summed cells.
