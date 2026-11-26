# Smaller default-state tables from suffix-closed patterns

Choose any finite pattern family S containing the empty string and closed under dropping its first byte. Give each forward rotation its longest prefix w in S. Its shorter matching dictionary prefixes are exactly those prefixes of w that belong to S. After prepending a byte b, a nonempty dictionary prefix must be bu with u one of those matches. Thus the pair (b,w) determines the next logical state. The logical state for w can occupy multiple disjoint rank intervals because deeper matches remove subintervals.

Split each logical state into its maximal physical components. For one such interval J, the actual incoming positions labeled b map under stable LF onto one contiguous interval. All its images have the same next logical state by the preceding argument, so they lie in **one physical component**. This holds for every used b. Each component may independently choose the incoming byte with largest frequency and keep the interval and bitmap jump theorem. Subtracting child cylinders from parent cylinders gives at most 2|S|+1 physical components by counting left/right endpoints and coalescing adjacent equal owners.

The tested family includes all words of length ≤K occurring in at least τ rotations. It is suffix-closed by counting occurrences. A word failing the threshold has no frequent left extension. Starting at the entire column, overlay each surviving parent interval with true last-column runs and accumulate one histogram entry per observed byte. Each retained child begins at the stable LF image of the first contributing row, with width equal to its histogram count. Append child intervals to their byte bucket in parent order. Overlay the sorted active children on the preceding physical components, which also assigns the longest retained match in gaps where refinement stopped. The constructor takes the run columns, frequencies and active intervals only. The independent dense checker extends prefix equivalence one byte at a time through true forward LF, comparing the owner at every physical row. Reported setup counts charge all touched RLE intersections, skipped runs, overlay fragments, histogram entries and final projection. [Exact constructor and tests](states.cpp)

All 52,464 (column, depth, threshold) combinations on arbitrary ternary BWT columns of length up to seven agreed with the dense checker. On ten real inputs the fixed-depth K=8 construction made 815,844 physical states, predicted 95.45% of predecessors and allowed true eight-step jumps on 81.35% of bytes in marked order. **K=16, τ=16 needs 184,004 states** and allows jumps on **80.42%**, with 9.09% exception rows. Totals for building those boundaries are 4,263,667 RLE intersections and 1,920,237 overlay fragments, versus 6,139,471 and 3,544,124 for fixed K=8. K=8, τ=16 uses 130,437 states with 72.65% jump coverage. The threshold changes both the exception directory and the default table. Complete paid reconstruction, peak occupancy and timing remain necessary for choosing among them. [All thresholds](totals.csv) · [Per-block evidence](blocks.csv) · [Finite receipt](small.txt)

Reproduce the ten physical inputs by following the clock panel's marked-data export. The constructor reads only the last-column portion of each file. The trailing exception mask is unused. [Authenticated input export](../../panel/)

```sh
d=$(dirname "$(rg --files | rg '/jumps/variable/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/states.cpp" -o /tmp/astra-variable-states
/tmp/astra-variable-states | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-variable-states /tmp/astra-clock-wires "$d/../../panel/blocks.csv" | diff - "$d/blocks.csv"
python3 -B "$d/report.py" "$d/blocks.csv" | diff - "$d/totals.csv"
```
