# The inverse BWT as an induced interval exchange: about 2.7 dependent steps a run, not one a row

10 November 2026. The Pathfinder, at Louis's request for a new mathematical treatment of the
permutation. One proposition, a counting probe that rebuilds every block by it, and what
would kill it. Counted, not timed. `examples/iet_census.rs`.

## 1. The proposition

**LF is an interval exchange.** On a run of L (rows of one byte), LF is a translation: row
x of a run starting at s goes to LF(s) + (x − s). The runs' images tile [0, n). So LF is an
exchange of r intervals, r the runs of L. The n-entry table stores a translation n times.

**An interval moves as one point.** Rows [x, x + w) inside one run move rigidly and all emit
that run's byte. One LF evaluation advances all w walks. They stay one piece until a run
boundary falls inside their image.

**Induce on an interval J = [a, a + W).** Start one piece, J itself. Advance pieces, cutting
one where a run boundary or an edge of J falls inside it, until each has returned to J.
- The leaves are the pieces of the first-return map R_J. It is again an interval exchange.
- Each leaf carries its *return word*, the bytes emitted on the way. All its rows share it.
- Kac: the return times, weighted by the leaves' widths, sum to n. Every row of the block is
  on exactly one leaf's path, once.
- So the block is the orbit of R_J from the walk's first row: one step and one copied word a
  return. W steps, each a `memcpy`.

**Induce again.** R_J is an interval exchange on W points. Induce it on a J₂ inside J; its
words are concatenations of R_J's. A tower of inductions is a grammar for the block, built
without visiting its rows, and the decode ends as copying.

**The claim.** The dependent work of inverting a BWT is proportional to its runs, not its
rows. Everything else is sequential copying.

## 2. Why it is true, and what is classical

- **Translation on a run:** LF(x) = C(c) + (copies of c before x). Consecutive rows of one
  byte have consecutive ranks.
- **The induced map of an exchange of r intervals on a subinterval is an exchange of at most
  r + 2:** each discontinuity is pulled back to J once, plus J's two edges. Rauzy's
  induction. Classical in dynamics.
- **Word complexity at most (r − 1)k + 1:** the number of distinct k-grams of an orbit's
  coding. The same statement as the known bound of kr distinct substrings for a text of r
  runs.
- **The two-run case is a rotation:** its codings are the Sturmian words, whose BWT is b^p a^q.
  A Thue–Morse word of length 2^k has 2k runs. These are the low end of the family, a test
  bed and not a model of Wikidata.
- **Related work I know of:** the move structure (LF as r translations, walked a row at a
  time); BWT tunnelling; precopying; a grammar of size O(r log(n/r)) from the runs;
  inversion in O(n / log n + r polylog n). **What I believe is new:** first-return induction
  as the decoding algorithm, with pieces as the unit of work, so that the n-entry table is
  never built. Not searched today; a prior-art check is owed.

## 3. Counted

Every block below is rebuilt from the tower and compared byte for byte with `bwt::walk`. A
tower "ρ × L" is L inductions, each on the middle 1/ρ of the level below. **Dependent
steps** are piece-steps at every level plus the top map's returns; the row walk takes n.

| input | rows | runs a row | best tower | dependent steps ÷ n | steps a run | row walk ÷ this |
|---|--:|--:|---|--:|--:|--:|
| Thue–Morse, 2^20 symbols | 1,048,576 | 0.0001 | 4 × 5 | 0.0024 | 24.6 | 417× |
| `wikidata`, level 9, 20 blocks | 17,999,801 | 0.0716 | 8 × 3 | 0.190 | 2.65 | 5.3× |
| `text`, 9 blocks | 7,608,762 | 0.3012 | 8 × 1 | 0.811 | 2.69 | 1.2× |

- **Wikidata by tower:** 4 × 1: 0.358. 8 × 1: 0.256. 4 × 2: 0.223. 8 × 2: 0.193. 4 × 4: 0.192.
  8 × 3: 0.190. The first level carries 0.108–0.131 of n; each further level about half the
  last.
- **About 2.7 steps a run on both natural corpora.** Any decoder reads each run's byte once:
  r steps is a floor. The row walk sits 14× above it on Wikidata; this sits 2.7×.
- **Copying:** the return words materialised at all levels are 0.68 n bytes for 8 × 2 and
  1.58 n for 8 × 3, before the block's own n.
- **A single deep induction decoheres:** on Wikidata at W = n/1024 the pieces are nearly
  single rows (0.82–0.95 n steps). Shallow levels, stacked, are what work.

## 4. What it would change

| | today | the interval engine |
|---|---|---|
| table | n entries, 3.6 MB a 900 kB block | r entries and an n-bit run map: about 0.4 MB on Wikidata |
| build | n writes | r writes, from #324's run records as they arrive |
| dependent steps | n | about 2.7 r: 0.19 n on Wikidata |
| the rest | — | copies of return words: 1.7–2.6 n bytes, sequential |
| a worker's live set | 3.6 MB table and its buffers | under 1 MB on Wikidata |

**Where it wins (projected, not timed):** inputs of few runs a row. Wikidata is the primary
corpus and has 0.07. A live set under 1 MB is the share-nothing regime the i9's level-2 run
showed scaling in.

**Where it does not:** `text` (0.81 n steps, each dearer than a row step) and high-entropy
data. They keep the row walk. The choice is by runs a row, known when the front ends.

## 5. What would falsify it

1. **The step's price.** A piece-step is dearer than a row step: a lookup in the run map, a
   rigidity test, a return test, a split now and then. At 0.19 n steps it must cost under
   about 5 row-steps to win on instructions, less to win clearly. A kernel timed against the
   room walk on Wikidata blocks decides. This is the next thing built.
2. **The copies.** 1.7–2.6 n bytes in words of 8 to a few hundred bytes. If the short words
   dominate, copying is not cheap.
3. **Branches.** A return about every 8 steps and a split about every 10 are data-dependent.
   Their mispredictions are the event cost that sank the copy walk.
4. **Prior art.** If this is a known inversion algorithm, the claim of novelty goes and the
   numbers stay.

## 6. Next

- **The kernel,** on the run-record representation, timed locally and then on CI. This
  container's L2 holds the whole n-entry table, so a local win is conservative and a local
  loss is not decisive.
- **Induce on a class of bytes instead of an interval.** J as the rows whose suffix starts
  with a byte of a class is a union of whole buckets. A rigid piece lands inside one bucket,
  so it never straddles J, and "returned" is a test on the byte just emitted. Return words
  become the tokens between class bytes. Expected to cut the first level's steps; uncounted.

## Reproduce

```
cargo build --release --example iet_census
target/release/examples/iet_census corpora/wikidata.bz2 12      # every 12th block
```
One JSON line a block; each tower's per-level `[steps, leaves, splits, words, longest]`, the
steps before the walk enters each J, and the top map's returns.
