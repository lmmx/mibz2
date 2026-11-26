# Exact return collation from arbitrary physical rank intervals

Partition all original row ranks into contiguous intervals, with state S(x) denoting the interval containing x. In each source state choose a most frequent pair (incoming byte, state of the LF destination), breaking ties lexicographically. A row is exceptional when its actual pair differs. For any fixed source interval and byte, stable occurrence ranks form a single contiguous LF image. Intersecting it with a destination state gives the complete positive target interval for that pair. Counting all pairs from R incoming runs and C states costs at most R+2(C−1) translated fragments. Each source seam cuts at most one original run, as does the inverse of each target seam, since the translated original runs partition the target. Histograms, their maxima, and access to the interval endpoints must also be paid. This reference uses a dictionary and binary search.

## Ordinary BWT of the returns

Move the section of exceptional rows U one LF step to V=LF(U). Enumerate its rows in physical order. For a forward first return from y to z in V of length d, assign the key

    (d, F[y], S(y), F[z], S(z), S(Ψz)).

Beginning at S(z), the d−1 interior backward steps follow its deterministic modal transitions. Their bytes in reverse order, preceded by F[y], give the entire forward word W. The same recursion determines every intermediate state. The prescribed F[y], S(y) and state of Ψy decide whether Ψy is exceptional. The corresponding final fields decide whether Ψz is exceptional. For d=1 the adjacent endpoints supply these tests directly.

Each realized key defines a convex interval of original ranks. Intersect the lexical cylinder W·F[z] with the inverse images of the prescribed states at steps 0 through d+1. Within that cylinder each Ψ power through d+1 is increasing by stable matching on its shared prefix. Each state condition therefore cuts out an interval. Conversely every row in the intersection has the same two exceptional endpoints and entirely modal interior. This uses the bytes *and* successor states in the predictor. Thus the intersection is exactly the given return class and Ψ^d is increasing there. Put its class ID in the incoming column at z. Numbering classes by first source occurrence makes ordinary stable BWT inversion on that column exactly the induced permutation. The same marked-phase and missed-section rules as the parent recursive decoder apply. [Marked inverse and missed sections](../recursive/)

Across 15,587 binary and ternary column/partition combinations (95,213 marked starts), the full classes passed convexity, exact spelling and stable successor tests. Omitting the first byte failed on L=1100100 with intervals ending at 2,7. Omitting S(z) failed on L=0110010 with ends 2,3,7. Neither witness proves necessity for partitions constrained to refine F. The checker finds genuine spelling, convexity or order failures and tries each individual deletion except distance. [Exact witnesses](small.txt)

## Ten authenticated original columns

| Tile width | States | Classification fragments | Exceptions | Distinct classes | Reduced L runs | Bytes in marked 8-step batches |
|---:|---:|---:|---:|---:|---:|---:|
| 16 | 562,495 | 1,698,434 | 2,714,231 | 1,471,844 | 1,481,823 | 2,096,200 |
| 64 | 140,629 | 948,184 | 3,014,184 | 852,372 | 906,609 | 1,977,208 |
| 256 | 35,160 | 755,273 | 3,362,681 | 582,912 | 727,761 | 1,718,776 |
| 1024 | 8,790 | 706,710 | 3,968,020 | 401,708 | 680,602 | 1,252,688 |

All seven widths covered all 8,999,915 physical rows across the ten blocks. The harness checks the marked expansion after reaching the first boundary against ordinary dense Ψ. It separately checks each spelling and successor, and the number of Ψ steps from the original mark. The complete parent recursive phase tests cover general period restoration. Width 16 leaves 30.16% exceptions, versus 4.55% in the forward-prefix k=8 model. The affordable one-sweep classifier alone does not justify timing this regular-grid worker. Adaptive splits can now be assessed under the same collation theorem. [All 70 configurations](blocks.csv) · [Adaptive refinement](refine/) · [Input identities and wire builder](../../../panel/)

```sh
d=$(dirname "$(rg --files | rg '/collation/interval-states/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/intervals.cpp" -o /tmp/astra-interval-returns
/tmp/astra-interval-returns | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-interval-returns /tmp/astra-clock-wires | diff - "$d/blocks.csv"
```
