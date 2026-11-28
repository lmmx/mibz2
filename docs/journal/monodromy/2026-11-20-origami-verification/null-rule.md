# Null-model rule for claim 3, fixed before any null data is computed

**The statistic:** c, the number of non-trivial cycles of k = r u r⁻¹ u⁻¹ on a block.

**The nulls:**
- **N1:** uniformly random even derangements on R points, the size of k's support. k is a
  commutator, so it is even, and on its support it has no fixed points. 2,000 samples each at
  R = 51,184 and R = 128,732 (the six blocks' r range).
- **N2:** run-shuffled columns. Each of the six blocks keeps its multiset of (symbol, length)
  runs in a uniformly random order, with adjacent equal symbols merged. 20 samples per block.
- **N3:** the empirical distribution over all 233 large blocks of corpora/wikidata.bz2.
- **N4 (prose) and N5 (code):** the large blocks of corpora/prose.bz2, and of a bzip2 stream of
  the repository's own Rust sources (`src/`, `examples/` and `tests/`, concatenated, at -9). Both
  are reported with the same statistic.

**The decision:**
- "Few cone points is exceptional for Wikidata" is supported only if more than 25% of N3's blocks
  have c below the 5th percentile of **both** N1 (at the nearest R) and N2.
- If N3's median c lies inside N1's and N2's central 90%, the verdict is **not exceptional**.
- N4 and N5 are classed by the same test.
