# Physical displacement colors on the original LF runs

A deterministic level color removes **2,993,038 of 8,999,915** original Wikidata rows in one marked pass. A zero-to-one crossing can never neighbor another removed source. Every contracted successor therefore spells either one or two old backward steps. The exact best phase vector for each block removes only **39 additional rows**. The earlier greedy type cut removes 3,025,271 and the forced-matching type cut removes 3,170,769 on the same ten marked blocks. The latter is a realizable first-pass choice, with no claim of optimality over type cuts. [Per-block phase graphs](graphs.csv) · [Earlier type construction](../../../../../../../../matching/README.md)

| First-layer evidence, summed over blocks | Physical rows or graph size |
|---|---:|
| Maximal original L runs | 690,484 |
| Source/image overlay tiles | 841,933 |
| Distinct signed level pairs | 744 |
| Nonfixed edges whose endpoints share a level | 2,966,751 |
| Guaranteed removals, integer lower bound per block | 2,991,670 |
| Ascending conditional-expectation greedy | 2,993,038 |
| Exact level optimum | 2,993,077 |
| Upper bound granting each level pair its own preferred sign | 2,993,820 |
| Upper bound granting each original overlay tile its own preferred sign | 3,130,831 |
| Prior anchored forced-matching type cut | 3,170,769 |

For the last upper bound, suppose each original L-run has its own independent phase. This even allows breaking old equal-level crossing edges. On each overlay tile exactly one source-run and target-run phase are compared, so its crossing count is bounded by max(D,width−D) for its measured raw bit disparity D. Sum these maxima, halve and round down separately per block. This grants mutually incompatible choices at shared run endpoints. It upper-bounds phases chosen on individual original runs, phases shared only across connected components of equal-level edges, and the simpler one-bit-per-level scheme. The optimistic bound remains 39,938 physical removals below the prior feasible matched type result on the pooled input. A selector that designates additional within-run subpieces would have a different size and information budget. [Every signed pair](pairs.csv) · [Run-only calculation](graph.cpp)

The compact index gives exact local survivor rank and select from two unshifted bit tests on each overlay tile, even when a physical selection checkerboards through many intervals. The finite verifier checks every phase, mark and returned word on the short inputs, and the extremal two-run family proves that explicit translation bands can grow to one record per surviving row. Across the ten real greedy passes there are **4,239,792** maximal normalized affine-word bands, against **559,695** with the forced-matched type cut. Nearly all the extra boundaries already occur in the normalized translations. The implicit map trades those materialized records for searching the original-run tile index on demand. A recursive calculation must account for the cost of resolving prior-layer selectors, targets and spellings. No hardware measurements were taken for these alternatives. [Exact construction and counterexample](PROOF.md) · [Dense corpus verification and representation counts](corpus/)

The exact signed graph needs only original coalesced cap runs and the stable F-bucket image cursors. The offline phase audit uses Gray-code enumeration on 15–18 active levels per original block. Its combinatorial time is separate from the deterministic linear-work conditional expectation selector. The dense state-by-state checker and previous type artifacts independently test the compact map. To replay the graph and finite receipts from the repository root:

```sh
d=$(dirname "$(rg --files | rg '/induce/modular/color/PROOF.md$')")
g++ -std=c++20 -O3 -Wall -Wextra -Werror "$d/check.cpp" -o /tmp/astra-color-small
/tmp/astra-color-small | diff - "$d/small.txt"
g++ -std=c++20 -O3 -Wall -Wextra -Werror "$d/graph.cpp" -o /tmp/astra-color-graph
python3 -B "$d/graph.py" /tmp/astra-color-graph /tmp/astra-cut-caps > /tmp/astra-graphs.csv 2> /tmp/astra-signed-pairs.csv
diff /tmp/astra-graphs.csv "$d/graphs.csv"
diff /tmp/astra-signed-pairs.csv "$d/pairs.csv"
```
