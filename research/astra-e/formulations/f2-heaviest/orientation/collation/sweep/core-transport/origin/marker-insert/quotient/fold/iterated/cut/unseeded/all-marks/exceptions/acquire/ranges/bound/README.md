# The number of elementary exceptions controls the weighted return map

Fix an arbitrary nonempty stable BWT column, any primary, and one depth k. Let r count maximal runs in the original physical L column, K count its nonempty equal k-future intervals, and s count shared interior boundaries between those two partitions. Their common refinement has H = r+K−1−s pieces. Within each context c let m(c) count its pieces carrying its chosen modal predecessor. Every context has at least one. Set δ=1 if the primary's predecessor is modal there, and δ=0 otherwise.

The number P of elementary affine births, before coalescing labels from different source pieces, is exactly

    P = r−1−s − Σ_c (m(c)−1) + δ.

Each of the H−Σ m(c) nonmodal pieces is one birth interval, including the piece containing the primary if that row is already exceptional. Every row in a piece has the same predecessor and k-future, so its LF image has a common k-future and stays within one target context. A synthetic primary adds one singleton birth. This proves the identity, including k=0. Every active context contains a birth, and a birth has one source context. Thus the number A of active contexts satisfies A ≤ P ≤ r−s. The expression depends on actual modal fragments, not exception mass.

Given the ordered context intervals and their modal bytes, a linear overlay computes the pieces. LF image locations increase with physical source position separately for each predecessor byte. Maintaining one monotone target cursor per byte costs O(K+H+256) visits in total. At positive k the sorted F bucket of each byte contains its destination contexts, with no context straddling buckets. At k=0 there is only one context. The complete default component and its translations through all unanimous contexts cost O(K+H) state and geometric work. Cycles consisting only of event-free states are closed under LF. By finite injectivity their rows have no external predecessor, so no live packet can reach them. [Unanimous paths](../../SMOOTH.md)

On the contracted graph the range proof now uses A states and P elementary births. The result has O(P) canonical or intermediate return bands with arbitrary positive weights. After smooth contraction it requires O(P log(P+1)) balanced-sequence and final publication work. The complete bound from the supplied context intervals is O(K+H+P log(P+1)) time and O(K+H+P) space, or O(P) active sequence space after preparing the contracted inputs. It does not give the k-context partition for free. The measured code can use general binary search and fixed-priority treaps; the worst-case time bound describes monotone input cursors and a worst-case balanced sequence implementation. [Range proof](../PROOF.md) · [Paid native result](../native/README.md)

## A primitive lower bound at every depth

Use two bytes a<b. Write a cyclic word with m≥2 successive copies of b, each followed by an a-gap, of lengths g₁=k+1, g₂=k+2, …, gₘ=k+m in that order. Mark the start of the first a-gap. A period smaller than the word length would permute b positions while preserving the gap lengths. The unique longest gap fixes one b, proving primitivity.

All m preceding-b occurrences lie in the context aᵏ. At positive k it has Σᵢ(gᵢ−k+1) = m(m+3)/2 rows. Its modal byte is a. At k=0 its single context contains all n rows and also strictly favors a. Every other context at positive k has unanimous predecessor a. Thus the event set is exactly these m gap starts. The primary is an existing event, and there is just one active state. Its m returns have the distinct weights gᵢ+1. Any representation by source intervals with constant return weight therefore needs m records, even if its within-band partner representation is unrestricted.

For the exact elementary count, the physical rows starting in a are sorted first by descending distance t to the next b. The t=gⱼ group has one row from each gap i≥j, with a preceding b precisely at i=j. For j<m, after the shared aᵗb these rows sort by decreasing length of their next a-gap, hence in old gap order m−1,m−2,…,j,m. Their unique b precursor is immediately before the group's final a precursor. The largest group by distance (t=gₘ) is a singleton immediately followed by the b at the start of the next group (j=m−1). These are the only adjacent b precursors. All smaller-distance groups with t<g₁ contain only a precursors, as do the b-starting rotations. The column begins with b and ends with a. Thus P=m−1 and r=2m−2. Its m canonical bands attain linear order in P while the event fraction m/n tends to zero at fixed k. This is an output-size lower bound for explicit constant-weight bands. More structured descriptions of the individual weights have other possible costs.

## Verification

Exhaustion through binary size seven and ternary size four covered 9,789 marked context models. The upper bound P=r−s was attained 6,527 times, including 554 positive-depth cases with P≥3. Independent cyclic-rotation BWTs of 20 lower-bound examples gave the predicted run/elementary counts and every dense LF return weight. On the ten systematic original columns, counting elementary physical source pieces before geometry coalescing gives the following totals. [Exact tiny outcomes](small.json) · [30 real cases](real.csv) · [Executable and input hash checks](check.py)

| k | K | A | Elementary P | r−s | Canonical bands |
|---:|---:|---:|---:|---:|---:|
| 4 | 471,228 | 97,686 | 262,557 | 292,982 | 269,448 |
| 8 | 815,844 | 23,497 | 121,043 | 143,812 | 127,360 |
| 16 | 1,170,188 | 18,467 | 94,275 | 112,514 | 101,276 |

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
d="$e/acquire/ranges/bound"
python3 -B "$d/check.py" | diff - "$d/small.json"
python3 -B "$d/check.py" "$original" "$models" | diff - "$d/real.csv"
```
