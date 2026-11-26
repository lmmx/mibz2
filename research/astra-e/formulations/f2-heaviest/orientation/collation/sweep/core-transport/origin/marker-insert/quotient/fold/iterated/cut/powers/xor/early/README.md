# The first paid cut already exceeds scalar's complete inverse

A complete cut is a valid handoff point. The residual's stable permutation maps each surviving source to its next surviving source in physical order. The corresponding positive grammar word spells exactly the original bytes until that successor. The protected source is a singleton, so its bucket base gives the marked residual row and its byte offset is zero. Walking from this base to the original byte horizon handles repeated shorter cycles and an unfinished final word. Each surviving type has positive frequency. The sum of its dictionary spelling lengths is therefore at most the weighted sum over residual rows, itself at most the original row count. A normalized, disjoint constant cycle may have been removed. [Earlier cut and power proofs](../../PROOF.md) · [Weighted residual and handoff source](prefix.cpp)

The frozen audit paid original cap assembly, relabeling, seed, literals, then every successive gap-coarsened graph-free selection and publication. It sampled cumulative thread CPU at each complete state and paired the whole trajectory with the same frozen scalar-cap call on the same original block. Twenty-four paired rounds each contain the ten fixed original blocks. A separate untimed pass materialized the residual dictionary, built its stable dense permutation and reproduced the exact original bytes at all 261 intermediate states, including depth zero and terminal states. Every state carries its own true compressed mark. [Prerecorded decision](PLAN.md) · [Precommitted calibration donor](../capped/build.json) · [Geometry and marks](geometry.csv)

| Complete prefix | Median ten-block CPU (ms) | Geometric prefix / full scalar [95%] |
|---|---:|---:|
| Run assembly, seed and literals only | 8.62 | 0.316 [0.300, 0.333] |
| One cut | 48.17 | 1.693 [1.645, 1.742] |
| Two cuts | 80.98 | 2.825 [2.731, 2.922] |
| Three cuts | 110.87 | 3.876 [3.750, 4.007] |

The scalar time median in this panel is 28.77 ms per ten blocks. One cut already costs 1.51 to 2.17 times scalar across individual blocks. The lower 95% endpoint exceeds 1.37 for each of the ten. If the original cap merge, relabeling, seed and initial grammar allocation were all free, the incremental first cut alone still costs 1.373 [1.322, 1.426] times scalar. These local paired estimates concern the specified C++ track. Faster cut constructors or a different computational object require their own pricing. [All raw pairs](prefix.csv) · [Per-block and depth statistics](summary.json)

The first paid cut reduces 8,999,915 source rows to 5,974,644, with 567,612 incoming runs and 688–1,088 ordinal types per block. Every first-cut word has length 1 or 2, the ten explicit dictionaries require just 12,793 bytes in total, and the weighted row count exactly reproduces the original total here. The remaining work could cost zero and every positive-depth handoff would still miss the scalar budget. The predeclared residual timing gate therefore has no qualifying depth. Depth zero has the original permutation, plus a paid seed. [Complete structural witness](geometry.csv)

Rebuild the calibrated capped harness as in its parent README. Then run the following from the repository root, using the same `x` and `s` paths. CPU 1 was used for the committed one-thread panel. The binary also implements a complete dense weighted residual method for independently measuring a qualifying list of depths on another construction.

```bash
python3 "$x/early/build.py" "$s/capped" "$s/refs/scalar-build/release" "$s/early"
taskset -c 1 nice -n 10 "$s/early/panel" corpora/wikidata.bz2 "$s/early" geometry
taskset -c 1 nice -n 10 "$s/early/panel" corpora/wikidata.bz2 "$s/early" prefix
python3 "$x/early/receipt.py" | diff - "$x/early/summary.json"
```
