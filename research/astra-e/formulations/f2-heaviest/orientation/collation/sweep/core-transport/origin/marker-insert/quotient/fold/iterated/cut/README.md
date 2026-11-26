# Stable delimiter sets as a marked grammar constructor

On ten Wikidata blocks the fixed greedy cut reached the entire marked orbit in **20–21 rounds**, compared with **61–255** for the protected ordinary fold. It created 221,522 grammar joins across the blocks, versus 435,821. It also created 56,641 alias classes and processed 509,345 nonempty directed class pairs. A native test would have to pay for both savings and setup. The current receipts measure structural work. CPU time, allocator high water and complete decoder throughput remain unmeasured for this implementation. [Frozen cut policy](PLAN.md) · [Theorem](PROOF.md) · [Ten-tape comparison](comparison.csv)

## Contraction and cost

The primary class is a singleton protected outside the target set. Each remaining class in source order chooses the side maximizing its crossing mass against previously assigned classes. Ties choose outside. Original input runs are split at old source-bucket boundaries. For each outside class, its occurrences acquire one consecutive source alias per destination type. The aliases entering a target bucket then absorb that target's spelling. A grammar node joins their two payloads.

All 208 real passes had zero self-transition mass. Each removed between one third and one half of its old ordinal rows. The first cut removed 52,083–63,393 rows per tape. Intermediate run count sometimes grew. The largest single-pass increase was 5.89%. Every tape ended with one source row containing a complete marked lap. Original byte offsets included 0, 1, 2, 3, 7, 13, 26 and 64. Every expanded byte matched the original dense marked permutation. [All passes](depths.csv) · [Marked output and input SHA-256](wikidata.csv)

| Across ten blocks | Ordinary protected fold | Forced delimiter set |
|---|---:|---:|
| Grammar-construction rounds | 1,111 | 208 |
| Old maximal runs entering the rounds | 1,484,391 | 508,700 |
| Old class counts entering the rounds | 609,903 | 320,437 |
| Grammar joins | 435,821 | 221,522 |
| Output expansion node visits | 872,393 | 443,201 |
| Output bytes from literal leaves | 1,980,569 | 1,293,494 |
| Output bytes copied from completed nodes | 7,019,346 | 7,706,421 |

The cut path computed 633,283 run/bucket intersections, built 540,379 refined RLE records and published 314,322 target-stream runs. The refined class counts sum to 377,078. The largest single refined inventory reached 17,942 runs and 11,328 source types. These record counts have no measured byte size or peak RSS attached. The Python reference uses dictionaries for the class graph and explicit lists for its streams. Its independent dense oracle also uses per-row arrays. A native comparison must pay for geometry, coloring, aliases, stable stream replay, grammar storage and output expansion on the same original input.

On frozen block 0 the old-run counts fell from 111,692 to 41,769. The cut also needed 52,741 run/bucket fragments, 45,282 refined runs, 42,155 distinct directed pairs and 25,329 target-stream runs. Its 17,892 joins compare with 38,140 ordinary joins. These operations overlap in the actual construction and carry different per-record costs. The independent dense checker validated 5,191,523 original/refined and contracted row partners across all ten tapes. [Per-tape costs](comparison.csv) · [Protected ordinary baseline](../anchor-wikidata.csv)

## Where the geometric bound becomes weak

For any N ≥ 1, let F=(a repeated N times, b), L=(b, a repeated N times) and mark the unique b. Stable matching makes one cycle through all N+1 rows. The anchor b must stay outside. The sole productive target set is {a}. It accepts just b→a, so the next b spelling appends one a and its successor configuration is the same with N−1 copies. Every admissible productive coloring consequently needs **exactly N rounds and N joins** from two original runs and two types. The first self mass is N−1, the off-diagonal mass is exactly two, and the first and every later step remove precisely one row. Assigning different colors to these same unsplit classes cannot fix this family. Covering long uniform chains would require a different decomposition or an independent repetition rule.

For a disjoint example take F=(a repeated N times, b, c) and L=(a repeated N times, c, b), marking b. The N fixed a rows cause arbitrarily large self mass. The entire marked b/c orbit resolves in one pass and the N+1 final rows account for those unvisited a cycles. Both families are checked through N=128 at an offset inside the protected word, including cyclic output past the marked orbit length. Neither family was observed in the measured cut histories. [Exact family program](obstruction.py) · [Results](obstruction.csv)

## Exhaustive and full-tape checks

The small panel enumerates every surjective incoming column up to six rows on at most three ordered labels, with label j carrying j+1 bytes. For each of 9,225 marked byte offsets it solves to the anchor's exact return and checks every output byte across the full original horizon. There are 7,616 disjoint-cycle cases and 5,819 nonintegral numbers of marked laps. The run verified 123,891 stable partners. A separate exhaustive one-step check enumerates **all** target subsets excluding the singleton anchor on all such columns, including empty or unproductive choices. It checks 37,773 cuts, 9,463 zero-deletion choices and 398,906 stable partners. The quarter-nonself progress guarantee uses the greedy coloring. An arbitrary cut can have zero mass. [Full marked checks](small.json) · [All subsets](all-cuts.json)

The constructor reuses source at 0cd5a2b for primary splitting, tracked-byte grammar expansion, original dense oracle and source data. The explicit-choice Fold constructor is d3d7657. The protected ordinary comparison file has SHA-256 18d49947ef2e5d7400978ad9085982a3b8a33b454f44d12ee0aa4e8e22607eff. The coloring policy was frozen at 36f7544 before examining tape results. [Constructor](cut.py) · [Comparison logic](compare.py) · [Independent subset check](all_cuts.py)

```bash
c=$(dirname "$(rg --files | rg 'quotient/fold/iterated/cut/PROOF.md$')")
tmp=$(mktemp -d)
python3 -B "$c/cut.py" real research/astra-e/dense/target/wikidata \
  --trace "$tmp/depths.csv" > "$tmp/panel.csv"
cmp "$tmp/depths.csv" "$c/depths.csv"
cmp "$tmp/panel.csv" "$c/wikidata.csv"
python3 -B "$c/cut.py" small --max-rows 6 | diff - "$c/small.json"
python3 -B "$c/all_cuts.py" | diff - "$c/all-cuts.json"
python3 -B "$c/obstruction.py" | diff - "$c/obstruction.csv"
python3 -B "$c/compare.py" | diff - "$c/comparison.csv"
```
