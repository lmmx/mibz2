# Exact adaptive refinement and its realized prediction accuracy

A source interval's occurrences of byte a occupy one contiguous interval in the destination F bucket. Together the old sources tile that bucket. A cut at a tile boundary cannot split an old (source,byte,destination) histogram bin even if other such cuts are added simultaneously. Source intervals can then be segmented independently using their old pair labels. One pass selects an actual maximizing subset of these allowed seams for total hits minus λ per new seam. It uses one best ongoing segment per label, a shared restart score at allowed seams and persistent backpointers. Allowed endpoints arrive in sorted byte buckets. Histograms use timestamped or touched entries. Each constructor pass scans the original run/partition intersections and materializes both the translated atoms and optional starts. The dense marked LF in the panel audits the result independently and counts the achieved batches. [Proof, exact joint condition and sharp loss example](proof/) · [Native implementation](refine.cpp)

Across the ten original Wikidata blocks, starting at F and stopping when a pass chose no cuts, the terminal partitions have the following totals. The DP fragment count includes the final unsuccessful pass and excludes the cost of histogram formation, support collection and the dense audit.

| Charge per seam | Final states | Exceptions | Marked bytes in 8-step batches | All DP fragments | Successful passes per block |
|---:|---:|---:|---:|---:|---:|
| 1 | 122,189 | 1,522,329 | 3,844,328 | 6,673,841 | 7–10 |
| 4 | 44,859 | 1,731,602 | 3,640,232 | 5,656,673 | 6–8 |
| 16 | 17,558 | 2,033,503 | 3,253,864 | 5,537,937 | 5–8 |
| 64 | 6,937 | 2,430,121 | 2,810,504 | 4,749,898 | 5–6 |

At charge 1, three passes already yield 107,491 states and 1,637,180 exceptions. Convergence buys 114,851 more correct predictions. The final exception fraction is 16.91%, with 42.72% of marked bytes in eight-step batches. Regular width 64 at a similar state budget left 33.49% exceptions. The fixed prefix k=8 model uses 815,844 states, leaves 4.55% exceptions and fits 81.35% of bytes in eight-step batches. Restricting the candidate cuts to those safe before each pass preserves the optimality of the *individual pass*. Some profitable simultaneously self-supporting splits are invisible under that restriction. These results do not bound the best arbitrary physical partition, nor do they measure a full worker. [All 307 snapshots](blocks.csv) · [Regular controls](../) · [Fixed contexts](../../../../contexts/)

Exhaustive native comparison checked 8,190 starting-partition/charge combinations and 23,712 candidate subsets on all binary columns through length six, recomputing the actual joint prediction score. The separate proof verifier also covers ternary columns and additional penalties. All ten real marked laps agreed with independent rowwise LF and the predicted objective at every step. [Finite counts](small.txt)

```sh
d=$(dirname "$(rg --files | rg '/interval-states/refine/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/refine.cpp" -o /tmp/astra-select-states
/tmp/astra-select-states | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-select-states /tmp/astra-clock-wires | diff - "$d/blocks.csv"
```
