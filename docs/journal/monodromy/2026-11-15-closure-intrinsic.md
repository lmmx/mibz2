# 2026-11-15 · MONODROMY 9 · What the free closure is not: no successor-graph compaction, no function of δ; and iterated compaction collapses a block in 5 to 18 rounds

Follows `2026-11-15-free-closure-confluence.md`, which proved the free closure a normal form of
the block and left two questions open:
- an intrinsic description, conjecturally a compaction of forced adjacencies;
- its size against the repetitiveness measures.

Both are answered here in the negative, with one finding on the side.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

**Inputs.** Wikidata (`corpora/wikidata.bz2`, SHA-256 `82f65ace…c29915`), its blocks of at least
65,536 rows.
- The closure's letters per block come from `2026-11-15-width-orders/confluence_wikidata.jsonl`.
- The programs are beside this entry as `.rs.txt`, with their outputs. They use only the
  crate's public API, as in the earlier entries.

## 1. Not one round of forest contraction (C, every 8th block: 30)

`roots.rs.txt` takes the first return of LF to the top intervals of the runs that branch, and
counts its letters: stretches of consecutive rows with one itinerary whose returns are
consecutive. A run branches in one of three senses:
- **forward:** its image is not inside one run;
- **backward:** its top interval is not inside one image;
- **either.**

| roots | the induced map's letters ÷ the closure's, min / median / max |
|---|--:|
| forward | 1.41 / 1.59 / 1.82 |
| backward | 1.54 / 1.71 / 1.94 |
| either | 1.61 / 1.81 / 2.07 |

## 2. Not the fixpoint of compaction either (C, the same 30 blocks)

`fixpoint.rs.txt` repeats one round until nothing more is removed. A round removes every piece
that has a unique successor piece (variant 0), a unique predecessor (1), or either (2), and
re-induces on the rest.

| variant | letters at the fixpoint, min / median / max | rounds |
|---|--:|--:|
| 0, unique successor | 2 / 9 / 72 | 9 to 13 |
| 1, unique predecessor | 2 / 6 / 28 | 14 to 18 |
| 2, either | 2 / 32 / 560 | 5 to 6 |

The closure, on the same blocks, holds 7,911 to 27,443 letters. So the closure is far from
these fixpoints, and from §1's single round.

**What a free step needs that compaction ignores (P, from the step's definition in the
confluence entry).** A free top win absorbs B into A only when both hold:
- J_B is the *tail* of I_A, ending at a coincidence;
- the landing row in J_A is already a top boundary.

Having a unique successor is not enough. The condition is positional, in the rows'
lexicographic order: it is the right-standard (Lyndon) side of the induction, not a property
of the successor graph. The conjectured unitig description is **refuted**.

**What the closure is, in counts (P).**
- **Its letters.** Only equal steps remove letters, and a BWT block's induction makes exactly
  r − 1 of them (§2 of the Rauzy–Veech entry). So the closure's letter count is
  1 + (the equal steps that only a paid step can make ready).
- **The open question,** in these terms: which equal steps need a paid step first?

## 3. Not a function of δ (C, all 233 blocks)

`delta.rs.txt` computes δ = max over k of d_k / k, exactly, for each block's cyclic text:
- d_k is the number of distinct cyclic factors of length k;
- d_k comes from the rotations' LCP array, by Kasai over the cycle.

| per block | min / median / max |
|---|--:|
| δ ÷ r | 0.158 / 0.186 / 0.195 |
| the closure's letters ÷ r | 0.075 / 0.185 / 0.281 |
| the closure's letters ÷ δ | 0.39 / 0.99 / 1.77 |
| the k where δ is reached | 5 on every block |

**The median of about 1 is a coincidence.**
- δ ÷ r is nearly constant from block to block, while the closure ÷ r varies 3.7×.
- log closure correlates with log r at 0.908, and with log δ at 0.878.
- log r and log δ correlate at 0.996.

So δ explains none of the closure beyond r. For scale, the earlier entries' figures on these
blocks: LZ77 0.295 r phrases, RePair 0.530 r, the Lyndon SLP 0.852 r.

## The side finding: iterated compaction collapses a block in 5 to 18 rounds (C; Conj on its use)

**The count.** §2's fixpoints reduce a whole Wikidata block (r ≈ 50 to 130 k) to 2 to 560
letters, in 5 to 18 rounds.
- Each round is the first return to the pieces that branch.
- The rounds are mutually independent within a round, so each is a parallel step.
- A fixpoint of 2 letters is one Euclid-like exchange away from the cycle.

**The comparison.**
- Right-end induction's dependency order is 93 rounds (median, §5.4 of the Rauzy–Veech entry).
- "The tower taken twice" (§6.4 there) was the first two of these rounds, followed by a walk.

**As counted here, a round walks every row it removes** (O(rows), not O(pieces)). So this is
a depth result, not a work result.

**Conj, untested:** the same rounds can be computed on pieces with their words, the forest of
§6.4 by rank. If so, the work is the sum of the pieces over the rounds, and a block inverts in
about 10 parallel rounds.

This is a structural reason to look at one block on several cores again, which Cooperative
Cells closed. Both statements are conjectures until the piece-level round is counted.

## Reproduce

```
for f in roots fixpoint delta; do
  cp docs/journal/monodromy/2026-11-15-closure-intrinsic/$f.rs.txt examples/$f.rs
done
cargo run --release --example roots    -- corpora/wikidata.bz2 8
cargo run --release --example fixpoint -- corpora/wikidata.bz2 8
cargo run --release --example delta    -- corpora/wikidata.bz2 1
```
