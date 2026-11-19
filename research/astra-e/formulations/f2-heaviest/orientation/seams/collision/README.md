# Fixed-clock collision of two live marked-rank frontiers

Let S be any union of selected ranks, Ψ the stable inverse of LF, and F the sorted column. Starting a strand at x in S, define y as its first positive return to S. Its emitted word consists of F at x and each intervening rank, stopping immediately before y. The implementation receives initial last-column runs and S. It discovers y, word length and the spelling by walking.

## Why the two walks meet correctly

Every permutation cycle with a visit decomposes into disjoint directed arcs between consecutive visits. Different arcs can share section endpoints, but their interiors share no rank. Put a forward walker on every source and a backward walker on every destination. These two roles at a section rank belong to different incident arcs, except on a cycle with a single visit. Keep both role sets live initially.

Advance all live forward roles by Ψ, spelling the byte of F they depart. Intersect their new ranks with the stationary backward roles and retire coincidences on both sides. Next advance all live backward roles by LF, spelling L at their departure, and retire both roles on each new coincidence. Repeat in that order. After k half-steps, precisely the arcs of length greater than k remain. Indeed, on the first half-step a hit on a stationary section rank completes exactly a length-one arc. Until the meeting, the two walkers stay in their own arc. At later steps either an unadvanced backward endpoint is at its own destination or both roles are in disjoint open interiors. Advancing one side by one edge can first hit the other at equality, without crossing an unobserved edge. This proves the invariant by induction. A return of length t retires at forward depth ceil(t/2) and backward depth floor(t/2). Repeating text creates distinct ranks and causes no additional meetings.

Record the forward starting anchor, the backward destination anchor and the forward spelling followed by the reverse of the backward spelling. The map on S is a permutation. The sum of paid individual edge advances in the two roles equals the number of original rows on cycles covered by S. Alternation saves no individual edge walks relative to a one-sided return construction. It changes which walks can share an affine translation, index probe or dictionary prefix.

For an arbitrary mark, the two-sided constructor rewinds on retained LF until it finds a section rank. It emits its recorded word starting at the saved offset and then follows return bands to produce exactly n bytes. A return to the mark before reaching S proves the mark's cycle is disjoint. The LF phase then reads that marked cycle in reverse and repeats or truncates its forward period. Empty sections use this same path. The forward-only control seeks the next visit on its retained Ψ map and saves the prefix it traverses. Neither procedure assumes cycle length divides n.

## What each control pays

The C++ constructor splits a contiguous packet on each physical affine LF/Ψ run. A packet carries the contiguous current ranks, equally translated contiguous anchors and one interned spelling ID. Packet coalescence requires adjacency in both rank spaces and equal spelling. In collision mode, sorting moved packets by row enables a simultaneous ordered intersection, which also splits surviving packets. The recorded words use separate prefix and backward-suffix hash tries, joined and materialized at a meeting. A final source sort enables contiguous band coalescence and output lookup. The one-direction controls use the same index, packet splitting, word interning, affine bands, final sort and output API. A fixed S permits each packet to query section membership independently and skip per-depth frontier sorting. These are matched controls for the rank-frontier design. The separate indexed Direct implementation at fd2d047 uses a different construction.

The C++ oracle stable-sorts the complete L column into Ψ, checks the first positive return, destination, and every spelled byte for every section row, and reconstructs the marked orbit. The small audit covers every binary L of lengths 1–8 and ternary L through length 6, every rank subset, and every mark. It includes multicycle inputs, equal rotations, singleton and full sections and marks on cycles disjoint from S. The ten-tape driver also checks every output against the independent Python stable inversion. This experiment stops at the marked BWT inverse. It does not give a complete bzip2 timing or a block/stream CRC receipt.

## Reproduction

From this directory, compile with assertions enabled. The ten-tape driver reads only the initial top runs and mark from each irec projection. Quote is the byte's F interval, and compound is the most frequent direct quote return's source interval, selected with the existing best-first LF selector. Both section bounds are fixed before any of the three constructors run.

```sh
g++ -std=c++17 -O3 -Wall -Wextra -Werror collision.cpp -o /tmp/astra-collision
/tmp/astra-collision small
python3 -B real.py /tmp/astra-collision ../../../../../dense/target/wikidata /tmp/astra-collision-results --laps 6
```

The initial frozen-screen.csv uses dense, unlabelled occupancy and ideal adjacency coalescence. It omits anchor identity, spelling identity and paid ordering. Native paid work is retained separately in the marked experiment.
