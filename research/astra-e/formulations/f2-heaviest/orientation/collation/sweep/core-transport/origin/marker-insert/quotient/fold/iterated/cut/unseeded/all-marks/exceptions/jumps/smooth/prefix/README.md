# Copying from the destination context

Let L be any last column with stable LF, Ψ its inverse and F its stable first column. For every physical row u, F[LF(u)] = L[u]. If y = LFᵈ(x), the forward spelling of the d backward transitions from x to y is

    F[y], F[Ψ(y)], …, F[Ψ^(d−1)(y)].

Indeed Ψʲ(y) = LF^(d−j)(x). This uses the actual stable permutation, so periodic ties, distinct LF cycles and repeated rotations require no uniqueness assumption. If y is in a class with fixed forward k-symbol prefix P and d≤k, the entire word equals P[0:d]. All such paths reaching that class share the same prefix, including paths with branches or different original rows. A backward walk taking only its first r≤d transitions writes the **suffix** P[d−r:d] when the full segment satisfies d≤k. Alternatively, the r-step destination supplies its own prefix of length r when r≤k. For d>k, reporting the full destination's prefix alone leaves the later d−k output symbols undetermined by this identity. Physical poststate and cyclic mark phase must still be transported.

At a smooth→branch boundary a single prefix per branching context can therefore replace all short incoming heavy-chain spellings. It can also include the preceding branching step, merging short branch-to-branch segments. A real block's primary may split a cyclic segment, and membership in a section is required to locate its predecessor or successor. For corpus counts the registered ten inputs have primitive physical marked laps. The native census walks from the actual marked row, verifies the full cycle and the previously authenticated bytes, and measures whole *cyclic* segments. The marked finite horizon can split one such segment. For instance the K16 cyclic count differs from the marked smooth-group census by one.

| Cyclic intervals over ten blocks | K4 | K8 | K16 |
|---|---:|---:|---:|
| Smooth stretches, eligible / all | 1,689,632 / 1,933,051 | 397,902 / 708,398 | 192,334 / 376,878 |
| Eligible smooth bytes | 3,411,682 | 1,595,181 | 1,206,041 |
| Heavy-chain copies these eligible stretches need | 1,835,516 | 458,576 | 210,258 |
| Literal capacity for these short prefix spellings | 125,636 B | 38,710 B | 44,059 B |
| Branch-to-branch returns, eligible / all | 3,390,912 / 3,768,860 | 786,014 / 1,108,546 | 380,217 / 575,661 |
| Eligible branch-return output bytes | 6,264,478 | 2,284,907 | 1,411,858 |
| Existing branch-byte stores and chain copies for eligible returns | 5,053,038 | 1,217,031 | 575,252 |
| Literal capacity for short branch-return prefix spellings | 195,498 B | 58,148 B | 57,168 B |

These capacities sum the largest eligible request for each participating context independently in each block. They exclude context lookup, LF destinations, length, offsets, long spellings and mark. At K16 the short smooth case collapses 210,258 heavy-chain copies into 192,334 and saves 17,924 across 575,908 original cyclic chain copies. Joining the preceding branch step would save 195,035 copy/store operations across 1,151,569 original operations. The already prepared K16 terminal took 1.279 times the complete scalar inverse, and direct context acquisition has a separate measured cost. This census supports a compact source of short return words, without establishing a competitive original inverse. [Whole-interval timing](../native/bench/README.md) · [All block and key counts](data.csv)

## Shifted sparse exception section

The proposed second BWT selects each physical row y = LF(x) where x has a nonmodal predecessor. The selected row belongs to the forward (k+1)-symbol cylinder with pattern L[x] followed by the k-symbol class of x. A forward return word from y to the next boundary z of length d≤k+1 equals the first d bytes of the **source pattern at y**. In an LF traversal starting at z and ending at y, y is the destination. A decoder visiting the sparse source ordinal could use its cylinder's prefix without generating d default transitions if it already has access to this source-pattern identity. A token carrying only the forward next pattern still needs to recover the source or regenerate the word. This theorem does not establish sparse LF order, adjacent classes, return construction or primary phase. [Sparse BWT plan](../../../return/collation/PLAN.md)

| Sparse forward return screen over 8,999,915 bytes | K4 | K8 | K16 |
|---|---:|---:|---:|
| Whole words representable by source prefix | 557,682 / 1,122,585 | 170,469 / 409,432 | 86,933 / 229,013 |
| Raw bytes in those complete words | 1,371,182 (15.24%) | 564,812 (6.28%) | 342,801 (3.81%) |
| Distinct participating source patterns, summed per block | 136,045 | 42,941 | 32,482 |
| Sum of maximum requested eligible prefix lengths | 290,375 B | 106,134 B | 102,365 B |
| Total leading bytes if long words also contribute their first k+1 | 4,195,697 | 2,715,479 | 2,758,161 |

The last row can contribute only a known leading substring to a long word. The remaining substring, its arrangement and references must be provided by another representation. All checked copies were compared with the actual marked raw bytes using a dictionary extracted from F and Ψ at one representative per exact pattern. This structural oracle allocates dense row arrays and omits their cost from any proposed worker.

```sh
d=$(dirname "$(rg --files | rg '/jumps/smooth/prefix/count.cpp$')")
g++ -std=c++20 -O3 -Wall -Wextra -Werror "$d/count.cpp" -o /tmp/astra-prefix-lf
python3 "$d/panel.py" /tmp/astra-prefix-lf /tmp/astra-cut-caps | diff - "$d/data.csv"
```
