# Which whole source buckets can be retired from random lookup?

The reverse deterministic closure predeclares complete-query buckets using the first and last incoming position for each source class and known bucket boundaries. The smallest exhaustive test checks 9,075 finite columns at every delimiter and 59,358 primary rows. The direct successor relation identifies 11,973 complete bucket instances in these configurations. The two-endpoint criterion verifies exactly 12,969, including cases where only a proper subset of a mixed parent has one common continuation. Its cycle bound correctly rejects 1,986 cases with a nonreturning endpoint. Every asserted trie parent, full interval, depth, and exceptional mark rule passed. [Closure proof](PLAN.md) · [Extremal proof](ENDPOINTS.md) · [Small closure](small.json) · [Small endpoints](endpoint-small.json)

| Ten-block sum | Root rows | Certified types / rows | Whole original runs entirely certified | Certified / remaining bucket–run intersections |
|---|---:|---:|---:|---:|
| Named | 245,445 | 1,422 / 444,662 | 9,499 of 84,317 | 10,680 / 105,928 |
| Quotient h4 | 282,172 | 26,966 / 532,837 | 59,270 of 84,427 | 85,934 / 30,932 |
| Quotient h8 | 288,648 | 27,672 / 545,785 | 61,619 of 84,995 | 88,929 / 28,536 |
| Quotient h12 | 299,675 | 31,460 / 567,200 | 65,613 of 84,641 | 96,533 / 20,596 |

The maximum certified depth is four on the quotient panel. At h4, 22,609 certified nonroot classes have width one. Splitting a run at boundaries of classes makes the total number of bucket–run intersections larger than the original RLE count. Each candidate still pays for streaming those intersections into its prepared child/selector representation. [All forty closures](wikidata.csv)

| Ten-block sum | All exact full classes / rows | Extra select queries with no positive cache | Extra select queries with positive cache |
|---|---:|---:|---:|
| Named | 31,121 / 558,753 | 91,802 | 58,560 |
| Quotient h4 | 32,148 / 600,940 | 36,509 | 15,135 |
| Quotient h8 | 32,173 / 610,825 | 32,631 | 12,485 |
| Quotient h12 | 32,174 / 622,012 | 13,211 | 899 |

The known first and last incoming occurrence of every source ID gives its first two images for free *after their scan*. The select counts above charge the remaining forward queries. For example, h12 still uses 35,625 synchronous endpoint comparison steps, base lookup and preprocessing as well as the 899 extra selects. The direct implementation builds an 84,641-record select index on this h12 panel. These structural costs cannot be read as a complete-time ratio. [Complete endpoint counts](endpoint-wikidata.csv)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual/certified
python3 -B "$d/closure.py" | diff - "$d/small.json"
python3 -B "$d/closure.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
python3 -B "$d/endpoints.py" | diff - "$d/endpoint-small.json"
python3 -B "$d/endpoints.py" research/astra-e/dense/target/wikidata | diff - "$d/endpoint-wikidata.csv"
```

## Native construction and smaller certified sets

The complete native forest removes most h4/h12 rank records, but it also expands many true unary paths and loses the matched complete CPU comparison. At h4 the ordinary 34,994-node trie became 61,816 physical nodes. One branching generation retains the ordinary node count and almost all of the rank-record reduction. At h12 it uses just ten source buckets per tape and 19,490 records across ten, compared with 19,441 for the full 31,460-source forest. A fixed producer-supplied right-brace child reproduced most of the remaining structural gain on all 219 unseen admissions. The separate measurements retain the observed CPU and peak-memory limits. [Full forest](native/README.md) · [Exact unary excess and depth-one bound](PRUNE.md) · [Sparse native price](sparse-native/README.md) · [Held-out literal and CPU](literal/README.md)

The complement index has at most one possible LF-continuous boundary partner per original source class. This allows exact size prediction without constructing that index, and an O(Kt) optimization for all optional-child budgets up to t. All thirty measured full-forest record counts agreed. [Bridge theorem](BRIDGES.md) · [Optimal cardinality budget](SUBSET.md)
