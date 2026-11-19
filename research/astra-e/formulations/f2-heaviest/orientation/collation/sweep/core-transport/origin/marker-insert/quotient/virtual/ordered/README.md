# Feasibility of a monotone source-bucket scanner

A graph with one vertex per live query, a parent-before-child edge, and adjacent interval ordering edges inside each source bucket is acyclic exactly when the chosen cursor directions permit a schedule. Exact enumeration of all independently oriented buckets on 85,476 finite ternary column/delimiter combinations through length nine found 9,896 with a direct contradiction inside one class, and 128 more where each class's ancestor constraints are individually consistent but their global scheduling requirements conflict. The smallest coupled case is L=(1,2,0,0,2,1,2,0), d=1. In its recorded numbering class 0 needs descending and class 2 needs ascending. The resulting cycle runs 1→3 (parent), 3→2 (class 2), 2→4→5 (parents), 5→1 (class 0). [Exact graph criterion](PLAN.md) · [Complete enumeration and witnesses](small.json)

| Ten-block sum | Queries | Full buckets reached in one query, types / rows | Locally contradictory buckets, types / queried rows | Queries in those buckets | Compulsory fixed-direction resets |
|---|---:|---:|---:|---:|---:|
| Named Compact | 69,999 | 31,121 / 558,753 | 30 / 260,373 | 7,343 | ≥35 |
| Quotient h4 | 125,936 | 32,148 / 600,940 | 13 / 138,147 | 21,352 | ≥13 |
| Quotient h8 | 124,506 | 32,173 / 610,825 | 12 / 135,969 | 20,071 | ≥12 |
| Quotient h12 | 124,504 | 32,174 / 622,012 | 12 / 135,969 | 20,071 | ≥12 |

All 40 graphs cover every token row. These full-bucket counts include the pivot and use oracle knowledge of the completed trie. The quotient's opposite same-class ancestor requirements occur on different trie branches, so they force a reset for a fixed orientation but do not force a sign change along any single branch. Allowing arbitrary random access in just the conflicting buckets and choosing one of the unforced directions left 59,679 unresolved nodes at h4. That count describes a particular heuristic assignment; the direct contradictions are orientation-independent. The remaining material cost is finding ranks or prebuilding the per-class summaries without already possessing that completed trie. [All forty observed order graphs](wikidata.csv) · [Graph construction](cursor_order.py)

A realizable five-run primitive family forces unbounded changes of direction for a single cursor. A four-run single-cycle family requires exactly q independent cursors for q² compulsory singleton queries in one class, even with complete knowledge of the ideal assignment. These obstructions apply directly to the online parent-first construction with monotone token-source cursors. [Five-run construction](FAMILY.md) · [Sharp four-run construction](LANES.md)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual/ordered
python3 -B "$d/cursor_order.py" | diff - "$d/small.json"
python3 -B "$d/cursor_order.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
python3 -B "$d/family.py" | diff - "$d/family.csv"
python3 -B "$d/lanes.py" | diff - "$d/lanes.csv"
```

For N queried source intervals whose start order is known in advance, the sharp worst-case number of freely directed monotone cursors is the largest k with k(k+1)/2≤N. The matching upper bound constructs an assignment. An extremal permutation is realized as the unique compulsory c-query sequence on one primitive marked BWT over four token symbols. Thus the square-root growth seen in the four-run family has the general worst-case order even when a different order of discovery could otherwise be available. [Sharp general result and realizable extremizers](EXTREMAL.md)
