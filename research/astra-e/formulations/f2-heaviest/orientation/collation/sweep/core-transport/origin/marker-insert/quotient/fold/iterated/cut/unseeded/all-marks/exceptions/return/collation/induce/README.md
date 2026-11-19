# Directly induce a sparse physical section and publish its spellings

Let V be any chosen set of row ranks together with the original primary. Let C stably move V first, preserving order inside V and its complement. The conjugated map C·LF·C⁻¹ is an interval exchange. An original affine L-run splits where its source or its image crosses a membership boundary; the images partition the axis, so t membership seams add at most 2t cuts across all R original runs. For both axes we order selected atoms first. Original source order supplies the top list. Stable order within each original L-byte bucket supplies the bottom list. The retained rows are now the exact prefix [0,|V|).

Induce by truncating the two interval orders at their common right end until the desired prefix remains. In a distinct-tip step let b label the last bottom interval and t the last top interval. The removed suffix has width the minimum of the two tip widths and remaining deletion budget. Each path through it enters along b and leaves along t. Concatenate the backward words b,t and reconnect those suffix strips. A partial final deletion can create one extra label. A coincident tip is an identity return on its own interval in the current map and has no external entrances. Such deleted cycles cannot contain the retained primary. Each step removes at least one row. The final paired affine strips contain the full first-return map and backward spelling of each strip. They require no context states, queues or donor chronology.

For the ordinary reduced BWT, number distinct **LF target strips** in target order (the source order of forward returns). Place each ordinal on its LF source strip in the incoming column. Stability reconstructs the exact induced map. Reverse the LF spelling for forward output. Traversing from the retained primary emits one true physical lap; repeating a short lap, including a final prefix when necessary, supplies the original n-byte marked horizon. The proof implementation compares the ordinary LF and all individual spellings against true physical unit transitions. An explicit stack avoids dependence on the machine recursion limit. [Induction core](induce.hpp) · [Independent dense oracle](check.cpp) · [Independent orientation and lower-bound review](review/)

| Retained section including mark | Retained rows | Initial atoms | Strict steps | Equal steps | Remaining affine strips |
|---|---:|---:|---:|---:|---:|
| True shifted depth-eight exceptions | 409,441 | 703,241 | 364,965 | 557,534 | 145,707 |
| All original run heads | 690,484 | 984,889 | 548,786 | 294,405 | 690,484 |
| Whole L-runs of width ≤4 | 784,776 | 716,497 | 346,558 | 99,171 | 617,326 |
| Prefix of supplied exception count | 409,441 | 690,519 | 415,305 | 608,307 | 82,212 |
| Sixteen least frequent present F buckets | 6,745 | 690,521 | 541,182 | 686,411 | 4,110 |

All 50 real cases reproduced the entire marked block and the total physical return spelling on the ten original columns. The 30,557 nonempty tiny sections covered 598,921 marked oracle bytes, including disconnected cycles. The production comparison must charge selection, coalescing capped runs, conjugation, surgery, memory and writing through a complete marked lap. The true exceptions above and the matched prefix count were supplied to expose the attainable geometry. The deployable native test instead uses a fixed n/22 prefix and the last two cheap intrinsic policies (short L-runs and rare F buckets). [Per-block counters](blocks.csv) · [Finite counts](small.txt) · [Native experiment](native/)

```sh
d=$(dirname "$(rg --files | rg '/collation/induce/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/check.cpp" -o /tmp/astra-induce-section
/tmp/astra-induce-section | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-induce-section /tmp/astra-clock-wires | diff - "$d/blocks.csv"
```
