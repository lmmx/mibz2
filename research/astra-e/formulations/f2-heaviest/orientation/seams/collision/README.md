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

## Ten-tape labelled comparison

Each of the three orders appears first and last twice in six round permutations, per section per tape. paid/laps.csv preserves every binary execution and paid/summary.csv sums per-tape median thread CPU or deterministic integer costs (max for peak packet slots). The instrumented release build uses GCC 14.2, assertions enabled, on shared Xeon Platinum 8573C. The clock excludes independent checking and destruction, and covers each method's actual maps, transfers, tries, assembly, final band sort and marked emission. The section-selection Python CPU is shared among arms and recorded separately in paid/inputs.csv. Peak packet slots count simultaneous vector capacities only. Each slot takes 16 bytes. A map uses one 16-byte piece per initial run per direction. Dictionaries, map bucket storage, outputs and process memory are additional.

| Section, method | Affine pieces | Front + final sort comparisons | Join steps | Seek comparisons | Constructor seconds | Output seconds |
|---|---:|---:|---:|---:|---:|---:|
| Quote backward | 1,585,376 | 1,336,744 | 0 | 19,244,563 | .317 | .057 |
| Quote forward | 988,959 | 1,404,961 | 0 | 12,344,975 | .225 | .055 |
| Quote collision | 1,091,492 | 15,545,400 | 2,049,813 | 8,417,083 | .287 | .062 |
| Compound backward | 2,119,342 | 1,023,132 | 0 | 25,426,088 | .443 | .015 |
| Compound forward | 1,874,223 | 1,041,941 | 0 | 17,118,774 | .381 | .014 |
| Compound collision | 1,364,367 | 18,685,021 | 2,604,366 | 9,770,406 | .338 | .017 |

All 360 runs matched every dense-oracle return and the external marked output. Quote midpoint used 10.4% more affine pieces than forward and lost constructor CPU. Compound midpoint used 27.2% fewer pieces and won the matched packet constructors. Dynamic ordering, simultaneous survivor splitting, a second directional index and additional packet working set are measurable costs. The high cost of repeated comparison sorting motivates one fixed-clock radix-order variant and an end-to-end check against indexed backward Direct.

## Full inverse reference and paid radix ordering

The complete reference links indexed backward Direct at fd2d04719fb696dd4b8d0b5cebeb000caf8032e6 (direct_native.hpp SHA-256 ca83b7e86111c055f7ed55b25e1e476cb2caa13425a2eb1cbec910877eb4e953). All three arms compile in one release translation unit. They take identical raw runs, mark and preselected section. The fixed-clock builds erase diagnostic counter increments in hot loops. Their maps, dynamic frontiers, tries, sorting, dictionary materialization, output lookups and buffer writes are all timed, including construction-state destruction. Direct constructs its LF page index, backward descriptors, spelling arena and source directory, then expands the marked output. This reference supports both requested section families. The input projection, common Python section recognizer, binary reading and post-timer byte checks are outside the comparison. Complete here means initial L runs and supplied section to n marked inverse bytes.

The second fixed-clock arm applies two stable 10-bit counting passes when a frontier contains at least 1,024 packets and ranks fit in 20 bits. Each pass counts, prefixes and scatters whole packets. It uses one temporary packet array and a 1,024-counter table. Smaller fronts and final source-band ordering use the same comparison sort. The clock, owner labels, meeting rule, spelling representation and emitted output stay fixed.

| Section | Paired fixed sort / Direct | Paired fixed radix / Direct | Paired radix / sort | Peak KiB fixed sort, radix, Direct |
|---|---:|---:|---:|---:|
| Quote | 4.775 | 4.103 | .859 | 16,564 / 16,564 / 9,416 |
| Compound | 4.892 | 4.419 | .903 | 19,092 / 19,092 / 9,192 |

The ratios use geometric means of CPU ratios in 60 matched laps per section, with all six possible orderings on each tape and an independently expanded byte-by-byte reference check on every arm. Maximum absolute process high-water marks come from Linux /proc/self/status in 60 additional fresh executions, one tape, arm, and section each. Each process includes input runs in both identical starter formats and the reference output. matched/laps.csv records all 360 complete CPU samples; its running process high-water values span multiple arms and are superseded by matched/memory.csv for memory comparisons. matched/inputs.csv identifies the inputs. matched/summary.csv retains separate per-tape contrasts.

The radix change recovers part of the ordering cost on both families. The remaining complete-inverse penalty and peak memory reject these packet constructors as an integration candidate. The compound split count remains a positive structural result. A future adaptive seam needs to change the cost of owner, spelling and endpoint assembly as well as the meeting positions. These are single-thread, shared-cloud marked-inverse observations. No worker throughput or i9 speedup is inferred.

```sh
g++ -std=c++17 -O3 -DNDEBUG -Wall -Wextra -Werror match.cpp -o /tmp/astra-collision-match
python3 -B real.py /tmp/astra-collision-match ../../../../../dense/target/wikidata /tmp/astra-collision-match-results --match
python3 matched_report.py /tmp/astra-collision-match-results
ASTRA_AUDIT_RADIX=1 python3 -B real.py /tmp/astra-collision ../../../../../dense/target/wikidata /tmp/astra-collision-radix-proof --laps 1
```

An additional assertion-enabled radix pass reconstructed and verified every source, destination and first-return byte across the same 20 tape/section combinations. radix-check/laps.csv reproduces every baseline geometry counter apart from ordering comparisons. Frontier plus final-band comparisons fell from 15,545,400 to 3,553,591 for quote and from 18,685,021 to 4,219,913 for compound. The instrumented packet-slot column excludes the short-lived radix scratch. Fresh-process VmHWM includes it.
