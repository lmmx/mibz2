# Arithmetic progressions of physical ranks

Partition the ranks into residue classes modulo s. Within each class, take the longest prefix whose successive output coordinates have constant difference modulo n. The one-lane exchange proof applies independently to every class. Each additional class gets free endpoints, and the real census gives each block its best tested stride without charging selection or metadata.

The positive control consists of 79 four-run BWTs with n=2d. The physical optimum needs d affine pieces while the parity decomposition requires at most four pieces for every tested d. On the ten Wikidata columns, choosing the most favorable stride among 1–64, 128 and 256 retained less than 1% implied coordinates in aggregate. Nine blocks selected s=1. The remaining block selected s=3 and implied only 277 of its 900,000 coordinates. The 50% screen failed on every block. The numerical inverse still admits sparse, level-specific affine maps. Small numbers of global congruence classes generally leave the full composed output-position permutation expensive on this corpus. [Every stride and block](results.csv) · [Per-block optimum](summary.json) · [Sharp small family and independent residue theory](../../../periodic/)

```sh
d=$(dirname "$(rg --files | rg '/backwards/slopes/lanes/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/lanes.cpp" -o /tmp/astra-rank-lanes
python3 -B "$d/check.py" /tmp/astra-rank-lanes research/astra-e/dense/target/wikidata | diff - "$d/results.csv"
```
