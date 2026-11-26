# Meeting inside a return word

The earlier directional optimum chooses one end for each whole return word. A more permissive experiment allows its forward and backward spellings to meet inside any word. If its seam is after t bytes, pay the distinct forward prefixes of lengths 0 through t−1 and backward suffixes of lengths 0 through |w|−t−1. An orientation can now reuse a common prefix in one group of words and common suffixes in another group that overlaps it.

## Exact spelling oracle

Build a bipartite graph with the proper prefixes on the forward side and proper suffixes on the backward side. For every byte at position i in a word, require at least one of its adjacent contexts: the prefix ending just before i, or the suffix beginning just after i. Activating a context requires all its ancestors on that side. The unit-cost closed vertex cover of these requirements is an ordinary s–t cut. Give forward nodes a unit arc to the sink, backward nodes a unit arc from the source, and each required context pair an infinite arc from its backward to its forward node. Add infinite ancestry arcs in the respective orientations. Each seam assignment induces a feasible cut. In the converse direction, the active prefixes cover an initial segment of a word and the active suffixes cover a final segment. Coverage of every position guarantees those two segments meet or overlap, and a seam in the overlap satisfies the paid sets. This proves exactness. The solver recovers actual seams and independently reprices them from strings on all inputs. [Cut solver](cut.py) · [469 exhaustive all-seam optimizations](small.txt) · [20 real dictionaries](wikidata.csv)

The optimum on frozen is 38,836 queries for quote returns and 42,380 for the most frequent quote return section, versus the earlier one-end-per-word optima of 48,631 and 71,853. Nearly all words use an interior seam. A solver with the full dictionary and free pairing of the two halves is an optimistic cost oracle. A usable decoder needs to find the actual partner of each half, including equal spellings and repeated rotations.

## A common clock that does not need the lengths

Start both directions at every section row. On each tick advance all active forward endpoints by one Ψ step or all active backward endpoints by one LF step, depending on the shared clock. Retire coincident endpoints after each positive step. Between successive section visits on a permutation cycle, the endpoints walk toward one another on the unique arc. A length-L arc finishes on tick L. Its seam is after the number of forward ticks t(L) in the first L ticks. The endpoints supply an exact source/destination matching by original rank. Equal rotations retain distinct ranks. Cycles missing the section use the ordinary marked fallback.

The fit excludes the nine nondevelopment tapes and uses the longest-word identity proved in the preregistration. Prefix or suffix activation can be charged at the longest dictionary length using that context, because the number of forward and of backward steps can each only grow. Consequently a shortest path on (word length, forward ticks used) optimizes every common clock on the development dictionary. At each length one predecessor represents a forward tick, the other a backward tick. On each tiny input all feasible direction clocks matched the dynamic program. [Preregistered costing and decision rule](CLOCK.md) · [Solver](clock.py) · [Frozen clocks](clock.json) · [All small checks](clock-small.txt)

| Nine excluded tapes | Backward only | Alternating from forward | Frozen direction clock | Arbitrary per-word seams | Both prespecified margins, out of nine |
|---|---:|---:|---:|---:|---:|
| Quote returns | 837,986 | 558,005 | 531,955 | 499,502 | 5 |
| Most frequent quote return | 1,195,718 | 711,234 | 667,315 | 560,790 | 6 |

Neither learned schedule met the eight-of-nine rule. The alternating clock already saved 33.4% and 40.5% of ideal backward spelling queries in the two classes. Learning reduced those counts by another 4.7% and 6.2% in aggregate. Its actual usefulness depends on the costs of split intervals, the two frontier orderings and concrete endpoint intersections. [Every held-out result](clock.csv)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams
python3 "$d/cut.py"
python3 "$d/cut.py" research/astra-e/dense/target/wikidata > /tmp/astra-seams.csv
cmp /tmp/astra-seams.csv "$d/wikidata.csv"
python3 "$d/clock.py"
python3 "$d/clock.py" research/astra-e/dense/target/wikidata --fit | diff - "$d/clock.json"
python3 "$d/clock.py" research/astra-e/dense/target/wikidata | diff - "$d/clock.csv"
```
