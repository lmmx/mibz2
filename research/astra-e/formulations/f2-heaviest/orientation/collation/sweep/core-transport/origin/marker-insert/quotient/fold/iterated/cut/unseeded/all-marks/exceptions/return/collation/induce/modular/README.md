# A single marked return word from modular decimation

Let a current interval exchange carry an incoming word on every translated source strip. Choose q≥2 and retain the ranks x congruent to p modulo q, where p is the current primary. These ranks become the first interval by arranging all q residue groups on both axes in the same order, starting with the retained group. Inside each group preserve the old order. An old source strip [a,a+w) maps increasingly to [b,b+w). For offsets i from zero to min(q,w)−1, the rows a+i+qt map to b+i+qt. After the coordinate change this entire progression is one ordinary interval translation of width 1+floor((w−1−i)/q). A single stable sweep of the previous top and bottom lists puts these pieces in the new orders, without sorting ranks, touching each old row or building an alphabet histogram at later levels.

Ordinary weighted tail induction then yields the first return to the selected group, keeping both new orders and concatenating original backward spellings on every join. Number its ranks from zero; the new primary is floor(p/q), and m′=1+floor((m−1−(p mod q))/q). Repeat until only the marked rank remains. First return is transitive on nested sections. The sole word then spells the first physical LF loop from p back to p once, ignoring disconnected cycles. Reverse it to recover the forward marked lap. If the lap has d<n rows, repeat its bytes to n, including the partial final copy. The final word has no residual permutation to traverse.

Every tail step charges a decrease in the current population. Across the tower they telescope to at most n−1 and create at most n−1 joins. Each reorder takes at most min(m,qB) progression pieces for B current bands. For fixed q, the geometrically decreasing populations give linear total construction work and storage. A sharper bound on progression objects is distinct from the bound on strict contractions. Every grammar node points only to older nodes. During reverse spelling remember its first completed occurrence in the actual output. A subsequent occurrence copies directly from the prior disjoint span. An explicit stack suffices even for a highly skewed parse. There are at most two child visits per reachable distinct join plus one root visit, and at most n output bytes to place. [Tower and memoized terminal](modular.hpp)

| q | Total emitted progression pieces | Strict operations | Equal operations | Total joins | Terminal visits | Bytes copied from a prior spelling |
|---:|---:|---:|---:|---:|---:|---:|
| 2 | 3,024,669 | 770,813 | 1,230,346 | 2,001,159 | 4,002,328 | 7,758,796 |
| 4 | 2,143,244 | 840,237 | 1,393,569 | 2,233,806 | 4,467,622 | 7,537,481 |
| 8 | 2,007,171 | 931,251 | 1,593,348 | 2,524,599 | 5,049,208 | 7,215,014 |
| 16 | 2,138,805 | 1,050,340 | 1,880,933 | 2,931,273 | 5,862,556 | 6,785,984 |

All four complete towers emitted the original 8,999,915 marked pre-RLE bytes. The finite verifier independently measured the true lap and byte stream for 85,254 marked column/radix cases, including 38,076 nondivisor cases. Radix two used the most progression pieces but the fewest joins and terminal visits. A complete timed reconstruction from the same capped input is required to rank those costs. The next direct question is whether canonical width-q strips can be recovered without materializing high-volume intermediates. [All 418 layer observations](levels.csv) · [Finite gate](small.txt)

```sh
d=$(dirname "$(rg --files | rg '/induce/modular/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/check.cpp" -o /tmp/astra-modular-induction
/tmp/astra-modular-induction | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-modular-induction /tmp/astra-clock-wires | diff - "$d/levels.csv"
```
