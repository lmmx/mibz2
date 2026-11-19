# Sharing a backward probe across destination colours

One original BWT run lookup can advance every live colour interval that falls in that run. Savings depend on the **occupied original runs**, which can differ from both the number of colour/suffix states and the number of runs in the packed selector. Two ternary-or-larger byte-column families attain opposite outcomes for arbitrarily long shared suffixes. These are exact discovery counts. They exclude construction of the root sections and final decoding costs. [Constructive families](families.json) · [Earlier union result at 34310a6](https://github.com/lmmx/mibz2/tree/34310a6ad12e15733fda6e293106701366f74f1e/research/astra-e/formulations/f2-heaviest/union) · [Independent union audit at 51e334a](https://github.com/lmmx/mibz2/tree/51e334a151420751a8c4ba56d237706fe1530f88/research/astra-e/formulations/f2-heaviest/union-audit)

## Translations and termination

Fix a nonempty parent marker P. The selected set U consists of complete first-return type buckets in the parent section I(P), coalesced into h disjoint rank intervals Cⱼ. At a live spelling u let I(u,j) contain the rows reached by prepending u to a destination in Cⱼ, before a prior positive visit to U. Let Rᵤ be the sorted union of its nonempty colour intervals. Prepending a byte b maps the rows carrying b stably into ranks C[b]+Occ(b,x). Thus the nonempty colour images for a fixed b are intervals in the original colour order. On one physical BWT run the map is a single translation x↦x+δ. If two intervals lie in that run, the same δ applies to both regardless of the gap size. When the current run ends, jump directly to the next occupied rank. If runs are revisited with the same byte, append their pieces to the same child in rank order. Touching pieces of the same colour coalesce.

Every row in the enclosing hull at u starts with uP. Finite-prefix cylinders are intervals because Ψ preserves order among rows with the same first byte, including equal cyclic rotations. For nonempty u, if uP does not start with P, no member is in U. Otherwise the next P occurs at some positive offset t≤|u|, since another P is supplied at offset |u|. The first such t and the parent return u[:t] are fixed by uP, including overlaps. Membership in U is exactly selection of that return. Hence a nonempty child terminates or continues uniformly across all colours. The empty root needs the positive-return exception.

Inside a node, record the runs of L restricted to the packed Rᵤ. Its b-subsequence maps stably onto the child Rᵦᵤ. A terminal child has its exact tagged labels (bu,j) in child order. Merging child label streams through the recorded selector recovers the tagged incoming words at every root destination. For a given tag, match successive destination occurrences to increasing ranks of its terminal source interval. Stable LF resolves all equal spellings. Retaining a boundary between touching *different* colours keeps each destination tag and frequency available.

For an arbitrary primary, a scalar LF walk to the previous U visit supplies the exact within-return offset. If the primary repeats before hitting U, the method delegates that block. Otherwise expand to exactly n bytes from the saved offset, looping and truncating as necessary. A mark inside a hull gap cannot be assigned to an active colour. In L=10110010 with P=0 and selected words 0 and 011, the node u=1 occupies rows 4 and 6. The primary 5 belongs to a disjoint cycle. A filtered branch can also lose the simple mapping back to its destination ordinal. In L=01001010, root colour [0,3) sends destination rows 0 and 2 into child interval [0,2). [Dense checker and exact counterexamples](exhaustive.json)

## How many run lookups can be saved?

At u let M(u) be the sum of the numbers of physical runs meeting each colour separately, J(u) the number of distinct physical runs touched by Rᵤ, and S(u) the number of runs in the packed selector. Let C count nonempty colour/suffix states and D count live uncoloured suffixes, including the root. Sum M, J and S over the live nodes. Each occupied run touched by k colours saves exactly k−1 run lookups. Consequently

```text
D ≤ S ≤ J ≤ M ≤ C + r − 1
M/h ≤ J
```

Here r is the number of runs in the original L. Each separate colour interval pays one initial lookup and one extra for each physical seam in its interior. Live occupied intervals are globally disjoint. For a row outside U, its unique next forward U visit fixes the distance |u|, its spelling and its colour. A U row is live only at the root. An internal seam is therefore charged at most once over the entire trie. These bounds count lookups in this run-based grouping scheme.

A sorted sequence of disjoint (lo,hi,colour) intervals supports lazy integer shifts, splitting at a rank, concatenation and selection of the next occupied rank. Keeping the occupied mass in a subtree supplies the packed selector width even across a large gap. With a balanced search tree a mixed node can split off the prefix in each occupied physical run, shift it, and concatenate it onto that byte's child in O(J(u) log(h+1)) work. Merging a partial colour at a join handles a colour straddling several runs. A one-run hull can advance with one lazy shift. Implementations must still charge the coloured terminal buckets, the selectors, joins, distinct spellings, mark transfer, output and peak scratch space. The checker caches the physical run descriptor over colour boundaries and materializes the pieces for verification. It does not measure a tree implementation.

## When holes disappear

Define G(u)=|hull(Rᵤ)∖Rᵤ|. A gap rank in a b-child has a unique inverse in the parent hull between two live b rows. If it were live, it would belong to that child. Its parent is therefore a hole carrying b. Child inverse images are disjoint, so Σᵦ G(bu)≤G(u), including terminal children.

If every live byte at u equals b, S(u)=1. Between consecutive occupied physical runs there must be a gap containing at least one non-b rank. Those distinct gap ranks disappear from the sole child. Thus J(u)−1≤G(u)−G(bu). Summing over **all unary nodes** telescopes to at most G(root), even in a branching trie, since every node has nonnegative potential loss. The bound is attained. The same bound for all nodes fails in L=1010110100, P=0, selecting 0 and 011. Its root is [0,2)∪[3,5), with one hole, but the total J−S is two. A hole splitting equal occupied bytes can survive in another byte's child. [Witnesses](exhaustive.json)

Equal bytes on opposite sides of a gap need separate translations when the gap contains another byte. For L=101010 with the same P and selected words, occupied root rows 0 and 2 both carry 1. Their displacements are +3 and +2. The packed selector can merge them, while their physical rank map still needs the two offsets. Scanning every run of a hull can also cost arbitrarily more than visiting its occupied runs. A family with only two singleton colours requires two root lookups while the enclosing hull intersects any prescribed odd number of runs. [Gap construction](families.py)

## Sharpness and independent checks

Construct marker words starting with 0, followed by a fixed-width binary code on bytes 1 and 2, and ending with byte 3 repeated d times. Make each word a separate cyclic component and select every other code in sort order, giving h colours. The backward 3-spine has d steps, with h separate probes and one shared physical run per step. For fixed h and d→∞, J/M tends to 1/h. For the opposite construction, replace every intervening unselected code by d words ending in byte 4 and k copies of 3, for 0≤k<d. At each spine level every adjacent pair of selected colours has a surviving blocker carrying 4. The J/M ratio now tends to one. All (h−1)d initial hole rows account for the unary excess. Duplicating each selected cyclic word makes these the strictly most frequent types and introduces tied rotations, with the same run counts.

| h=8, d=64 | Colour states | Distinct suffixes | Separate M | Grouped J | Initial hole rows |
| --- | ---: | ---: | ---: | ---: | ---: |
| Shared original run | 552 | 80 | 552 | 104 | 7 |
| Intervening blockers | 552 | 80 | 552 | 545 | 448 |

The independent oracle enumerated all arbitrary binary last columns through length 11 and ternary columns through length 8, with parent lengths 1–4 and every selection having at least two coalesced components. All 17,576 selections recovered their exact forward marked links. Expansion matched dense Ψ for 136,756 primaries, including 91,904 whose period does not divide n. The other 23,594 primaries completed a cycle outside U. [Raw exhaustive counts](exhaustive.json) · [Construction source](families.py) · [Raw family counts](families.json)

```sh
d=research/astra-e/formulations/f2-heaviest/union/hull-audit
python3 -B "$d/check.py" > /tmp/astra-hull-exhaustive.json
cmp /tmp/astra-hull-exhaustive.json "$d/exhaustive.json"
python3 -B "$d/families.py" > /tmp/astra-hull-families.json
cmp /tmp/astra-hull-families.json "$d/families.json"
```
