# Sliding the section across every rank

For a fixed residual mass m, each original-rank window U=[a,a+m) induces a dictionary of positive first-return words. Let K be its number of distinct raw words, D their total byte length and Q the number of distinct proper suffixes, including the empty suffix. The frozen block has one Ψ cycle, 899,931 ranks and m=24,165. Of all 875,767 windows, exactly 1,912 have both smaller D and smaller Q than the most frequent quote-return section. Only two distinct points on the complete Pareto frontier strictly improve both coordinates. [Full exact spectrum](frozen-spectrum.csv.xz) · [All 208 Pareto pairs](pareto.csv) · [Disjoint improving ranges and selection rules](summary.json)

| Frozen window start | K | D | Q |
|---|---:|---:|---:|
| Named compound 21,229 | 2,950 | 146,183 | 100,644 |
| Minimum D, start 151,103 | 3,499 | 131,203 | 107,545 |
| Minimum Q, start 324,490 | 5,285 | 198,455 | 79,253 |
| Jointly better Pareto point 333,831 | 3,757 | 144,611 | 92,898 |
| Jointly better Pareto point 333,910 | 3,758 | 144,685 | 92,889 |
| F-column colon base 336,204 | 2,958 | 145,489 | 96,944 |
| Ψ image of named compound 336,231 | 2,950 | 146,183 | 97,695 |

Start 333,831 minimizes max(D/D_named,Q/Q_named), breaking ties by Q, D and then original rank. This is computed with integer cross-products. Its extra 807 word types could add to publication and output costs. A histogram already gives the F-column colon base. Of the 172 distinct windows beginning or ending at any F-byte boundary, this was the sole start improving both frozen costs. Selecting colon because it wins here uses this block for training. The least-fragmented L-column window retains the same D and reduces Q, a case explained by exact conjugacy below. Counting L runs alone does not determine either price. There are 377 windows with the named section's count of 198 runs, spanning Q=96,195–172,334. Width selection, dictionary discovery, ordering, mark transfer and reconstruction remain part of the decoder's paid work.

## Arc surgery on labelled permutations

Choose an arbitrary forward origin on each permutation cycle and give its ranks increasing offsets in orbit order. For each cycle keep its selected offsets in circular order. Between consecutive selected offsets u and v the arc spells exactly one return word. Its length is the positive cyclic distance, which equals the cycle length when u=v. An unselected cycle contributes nothing.

Erasing one rank x removes arcs prev→x and x→next and inserts prev→next. When x was the sole selected rank, remove the unique whole-cycle arc. When exactly two ranks were selected, prev=next and the merged arc now runs the full cycle. Insertion performs the inverse operation, including creating a whole-cycle arc on an empty cycle. Advancing a fixed-width window erases a and inserts a+m. The two ranks can lie on different cycles. The operations change at most six occurrence keys per advance. Empty sections in the intermediate state have K=D=Q=0. This argument uses neither stable sorting of bytes nor a single marked cycle.

Reverse and double the byte itinerary of each cycle and concatenate the results. The full reversed word of any arc is a substring in the doubled copy of its own cycle. Its start is determined by the destination offset and its length by the positive cyclic distance. The length never exceeds one copy. Cross-cycle separators are unnecessary because every comparison is capped before the queried arc ends.

## Updating both prices in one dictionary

Order the distinct **full reversed** words lexicographically, storing occurrence counts for duplicate words. Let E(x,y)=LCP(x,y)−[x is a prefix of y] for adjacent distinct keys x<y. For K>0,

```text
Q = 1 + D − K − sum E(x,y) over adjacent pairs.
```

The nonempty nodes of the full trie number D−Σ LCP(x,y), by inserting keys in lexical order. Passing to the union of all proper prefixes removes exactly the full terminal leaves. A key is a nonleaf iff its immediate successor properly extends it. There are K−Σ[x prefixes y] terminal leaves. Adding the empty prefix proves the formula. Keeping just the full keys avoids relying on the lexical order of individually shortened strings. A first insertion or last deletion changes at most three ordered-neighbor terms. An occurrence-count change with no dictionary membership change leaves both D and Q alone.

The native implementation forms a suffix array of the doubled tape and a Kasai LCP array. A minimum query uses a 32-entry monotone bit mask inside each block and a sparse table across complete blocks. Equality and comparison of finite substrings are exact. The array construction uses counting-sort prefix doubling, O(n log n) time on 2n+1 positions. Including the block sparse table, the offline index has O(n+(n/32) log n) words. After indexing, window initialization takes O(m log(m+1)) time and the remaining windows each take O(log(m+1)) time with O(m) live dictionary and selected-offset entries. Cycle coordinate records and per-cycle set headers cost O(n). Writing all prices costs Ω(n−m+1) output operations. This bound concerns the full sweep rather than selecting a section for one production block.

On the frozen panel the cycle decomposition took 0.062 s, exact suffix indexing 1.259 s, initialization 0.025 s, successive updates plus CSV writing 1.853 s, and dense checks 0.304 s. The independent native process used 68,164 KiB at peak. These costs price an offline structural oracle. A single-block decoder cannot assume the dense orbit or the index for free. [Process receipt](resources.txt) · [Stage receipt](stages.txt)

## Verification and reproduction

The substring index was also checked against direct lexical and byte comparisons across 32-row RMQ block boundaries on constant, periodic, low-alphabet and arbitrary-byte tapes. [Index receipt](index-check.txt)

The exhaustive tests cover every nonempty sliding window of every permutation through size five with binary outgoing byte labels, plus every binary stable column through size seven. All 67,384 comparisons matched explicit independently built dictionaries and separately sorted truncated proper keys, including 53,964 windows on multi-cycle permutations. On the full frozen block 19 preselected windows passed the independent native dense check. A second implementation stable-sorts the original column in Python and spells all return arcs at the named section, both per-metric extrema, the two improving Pareto points and the bounded selection rules. It checks the final prices and the actual rotated return words. [Small permutation receipt](small.txt) · [Dense checks](dense-checks.csv) · [Independent real dictionary receipt](rotation-frozen.json) · [Shift theorem and limitations](rotation.md)

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep
out=$(mktemp -d)
g++ -std=c++20 -O3 -Wall -Wextra -Wpedantic "$d/sweep.cpp" -o "$out/sweep"
"$out/sweep" | diff - "$d/small.txt"
g++ -std=c++20 -O3 -Wall -Wextra -Wpedantic "$d/index_test.cpp" -o "$out/index-test"
"$out/index-test" | diff - "$d/index-check.txt"
python3 -B "$d/input.py" research/astra-e/dense/target/wikidata/0000.irec.xz "$out/column" | diff - "$d/frozen-input.txt"
python3 -B "$d/measure.py" "$out/resources.txt" nice -n 10 taskset -c 3 "$out/sweep" "$out/column" 24165 21229 "$out/spectrum.csv" "$out/checks.csv" 2> "$out/stages.txt"
diff "$out/checks.csv" "$d/dense-checks.csv"
xz -dc "$d/frozen-spectrum.csv.xz" | cmp - "$out/spectrum.csv"
python3 -B "$d/summarize.py" "$d/frozen-spectrum.csv.xz" "$out/column" 24165 21229 "$out"
diff "$out/pareto.csv" "$d/pareto.csv"
diff "$out/summary.json" "$d/summary.json"
python3 -B "$d/rotation.py" | diff - "$d/rotation-small.json"
python3 -B "$d/rotation.py" "$out/column" "$out/summary.json" | diff - "$d/rotation-frozen.json"
```
