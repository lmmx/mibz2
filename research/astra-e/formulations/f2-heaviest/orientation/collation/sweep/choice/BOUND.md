# What any one-byte shift can save, even with an independent choice per occurrence

Suppose every return from a nonempty section starts with the same byte c, and write its words cV. Move any subset of its individual cut positions forward by one step. Require that the new cuts remain distinct. Different occurrences of the same type may acquire different new labels and different old types may coalesce. The result below concerns only Q, the number of distinct proper suffixes including the empty one. It places no restriction on the number of rank components and assumes free knowledge of the cuts.

Let P be the set of nonempty proper suffixes of the tails V. If at least one V is nonempty, the completely shifted dictionary has Q_full=2+|P|. Its nonempty proper suffixes are c and each u·c for u in P.

For any partial shift, fix an occurrence whose tail has a proper suffix u∈P. Its new word consists of that same tail, an optional c at its beginning, and an optional c at its end. Regardless of the source bit, at least one of u and u·c is a nonempty **proper** suffix. This is true even if the occurrence receives a context-dependent label or merges with another word. The new suffix set covers every edge (u,u·c). Appending c has a unique successor and at most one predecessor, so these edges form disjoint finite paths. Their minimum vertex cover is computable by a two-state path recurrence.

There is one further compulsory state when a move affects a cycle that has a nonempty tail. Follow original cuts backward from a moved cut. If its predecessor is unmoved, the new incoming word ends in c and has length at least two. If its predecessor is moved and has a nonempty tail, the same conclusion holds. Otherwise keep following moved one-byte predecessors. Finiteness and the nonempty-tail hypothesis force one of the first two cases. Thus c is a proper suffix and must be included in the cover. If shifts are confined to cycles with only one-byte words, distinct cuts on each such cycle force its bits to be constant and the dictionary is unchanged. The resulting lower bound is the lesser of Q_original and one plus the cardinality of a minimum cover containing c. If all tails are empty, Q=1.

**When this is sharp by inspection.** If no member of P ends in c, the edge endpoints u and u·c for different u are disjoint. None is c. An effective move must pay for every edge and for c and the empty suffix. The unchanged dictionary costs at least Q_full by the [uniform rotation theorem](../rotation.md). Hence moving every cut minimizes Q among *all* collision-free one-byte assignments, including cases with physical interleaving and context-dependent spellings.

This sufficient condition holds on all ten quote sections and all ten most-frequent-return sections of the systematic Wikidata tapes. For example Q_full on frozen is 67,930 at quote and 97,695 at frequent. Searching a larger family of one-byte shifts on these blocks cannot lower this lookup count. The price of reaching those optima, their dictionary sizes, alternative decompositions and longer shifts are separate questions. [All twenty achieved lower bounds](bound-wikidata.csv)

Collisions between u and another tail suffix ending in c can make a mixed move genuinely better. The binary four-rank example in the [ordered movement experiment](../context/) realizes a strict improvement. Across 17,064 positive assignments of 3,584 small monochrome sections, the path lower bound was attained on 3,042 sections and never exceeded the actual optimum. On 262 sections some choice beat the full shift. [Exhaustive counts and counterexample](bound-small.json) · [Path recurrence and test](bound.py)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/choice
python3 "$d/bound.py" | diff - "$d/bound-small.json"
python3 "$d/bound.py" research/astra-e/dense/target/wikidata | diff - "$d/bound-wikidata.csv"
```
