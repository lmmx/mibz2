# Moving individual return cuts without changing residual order

Let U=[a,b) lie within one first-column byte c. For each return-word type A choose a bit s(A). Move every source of A one Ψ step forward when its bit is one. The all-zero and all-one choices are the two uniform sections. A nonuniform choice can change the dictionary while keeping every induced successor in the same order-compressed slot.

**Complete feasibility criterion for one-bit type-dependent cuts.** For a return A followed by type B its new spelling is A with its first s(A) bytes removed and s(B) copies of c appended. Every B that can follow A must have the same bit for this spelling to be a function of A. This condition is necessary since appending a byte changes its length. Glue all such B together for each A.

Next consider two successive source ranks x and y=x+1. Moving both preserves their order because Ψ is strictly increasing on their common F-byte bucket. If Ψ(x)≥y, moving x requires moving y. If Ψ(y)≤x, moving y requires moving x. These adjacent implications suffice for strict order everywhere. A feasible selection is therefore a union of strongly connected components of this implication graph, closed under outgoing edges. Among these selections retain exactly those whose transformed spellings are nonempty and pairwise distinct.

These conditions are necessary and sufficient to keep one distinct spelling per old type and the original ordered first-return permutation. In orbit order each cut moves at most to the end of its old return arc. The cuts cannot overtake one another. Strict rank order ensures distinct cut positions, so no two cuts at adjacent arc boundaries coincide. The next cut is the image of the old successor, including on a one-cut cycle. The resulting arc spells precisely the formula above. This argument also covers return words of length one. To recover an actual primary, spell to the first U source and pay its optional first byte before continuing on the transformed section. A cycle missing U is handled by the original finite marked navigation.

**Nonuniform motion can beat both uniform choices.** For last column 1,0,0,0 and section [1,3) the original words are 0 and 010. Moving only the latter cuts at ranks 1 and 3 and changes them to 00 and 10. The proper-suffix count falls from three to two. Moving both would give 0 and 100 and still require three. This instance has two types and four dictionary bytes under all three policies.

The exhaustive program tries every bit vector independently of the implications on 9,216 small monochrome intervals. Every surviving graph assignment also matched the actual positive-return spelling and successor of each transformed source. Of 23,046 valid type assignments, 4,614 were nonuniform. On 532 sections the best nonuniform choice beat the uniform forward shift in suffix count. [Exhaustive witness and counts](small.json)

On the quote and frequent sections of the ten systematic Wikidata tapes, only 185 choices survive in total, 145 nonuniform. Eight of twenty sections allow only the two uniform choices. None of the 145 reduces suffix count below its own uniform shift. Both the complete physical mark and original output bytes were checked for every admissible choice. Only two tapes have a choice that simultaneously reduces suffix count against unshifted and component count against fully shifted. These results bound available freedom under the specified one-byte, type-preserving, rank-order-preserving model. Unrestricted shifts can expose context-dependent new types or require an ordered relabeling of row positions. [All 20 real cases and exact geometries](wikidata.csv) · [Uniform shift construction](../rotation.md)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/context
python3 "$d/shifts.py" | diff - "$d/small.json"
python3 "$d/shifts.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
```
