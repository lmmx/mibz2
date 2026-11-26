# Unequal clocks can meet without knowing the return spelling

The input to the C++ constructor is a run-encoded original column and a union of nonempty, disjoint rank intervals. The constructor recovers each first-positive-return section partner, how far to move, and the exact edge labels. It takes no spelling, return length, dictionary, source-to-destination correspondence, dense LF/Ψ table, or decoded text as input. The separate verifier expands ranks and checks the resulting links one occurrence at a time.

## Invariant and construction

A permutation cycle touching the section decomposes into its successive positive first-return arcs. Give each such arc two live heads at its ends. A head stores its section owner and the current cohort (direction and emitted path). Before completion it lies at distance f from the forward origin or b backwards from the destination, where f+b<l for arc length l. The arc interiors are disjoint, even between cycles. For an interior row occupied by both directions, the two owners therefore belong to the same arc. At a section row before any movement, two roots may coincide without having crossed an edge. The constructor queries opposite occupancy *after* a positive move. Arrival at a section row implies that the moving head consumed the entire arc, so its opposite is still at that row with depth zero. These rules include one-edge returns and a section containing one rank on a cycle.

A cohort consists exactly of live occurrences with its directional path. The two directions start with one cohort apiece. An expansion moves all its live ranks, then groups survivors by the new byte and publishes these children in increasing byte order. Killed opposite owners immediately decrement their cohort mass. Removing dead cohorts and refreshing stale heap priorities reproduces the exact globally heaviest-live rule with the prescribed depth, direction and birth ties. All occurrences of one word experience the same two directional histories by induction, because the symbol and completion test depend only on the common spelling and depths. Thus rank-only states and heap pops can be checked against the independent dictionary scheduler. Matching actual partners requires the rank intersection.

Each indexed interval maps current rows to section owner ranks with slope one. Clipping to one original translation piece preserves slope one and supplies the crossing byte. Intersect the translated interval with the opposite rank map. A hit yields a published band of forward and backward owner pairs in the same relative order. Its spelling is the forward group path, crossing byte and reversed backward group path. Remove exactly the hit ranks from the opposite front, splitting an interval if necessary. Each remaining gap continues in the moving direction. Intrusive cohort lists enumerate only that cohort's live packets. No global rank sweep occurs.

Induction on positive moves proves all published first-return links and labels are exact. Each arc consumes exactly its length in expanded occurrence steps, independent of how often either direction moves. Their sum is the size of all section-touching cycles, bounded by the original row count. Every productive expansion advances at least one live occurrence, so termination follows. For marked reconstruction, follow original Ψ to the first section hit, or until n bytes. Then emit the recovered words and follow the affine partner; clip the last emission to exactly n bytes. This includes cycles disjoint from the section and periods that do not divide n.

Two arrays of r original affine pieces cost O(r) setup. Let P be all input packets popped, T the count after translation clipping, J the number of resulting hits and gaps, H the peak number of active packets, G the number of groups, and A the number of published affine bands. The implementation uses O(P log r) original-piece search comparisons, O((P+T+J) log(H+1)) rank-index comparisons, O(heap pops × log(G+1)) heap work and O(A log(A+1)) output-band sorting. Its space is O(r+H+G+A), plus n output bytes when emitted. Groups retain directional backpointers for output. No claim about inevitable search complexity follows from use of ordered maps.

## Receipts

All 20 systematic block/section cases exactly match the independent spelling model's states **and** heap pops. The dense verifier checked every source, unique destination, emitted byte, mark and final marked output. Its stable-sort column projection matches the frozen marked SHA-256 in the original evidence. Small tests enumerate all permutations and binary edge labels through five rows, all binary stable columns through seven rows, every nonempty section subset and every mark. There were 763,396 marked outputs, including 73,586 tested sections omitting at least one whole cycle.

| Frozen section | States | Source packets | Clipped translations / opposite seeks | Tree key compares | Published bands | Max live map nodes | Constructor bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| Quote | 38,889 | 61,548 | 70,479 | 2,424,040 | 6,133 | 5,795 | 5,183,464 |
| Frequent | 42,465 | 70,863 | 78,676 | 2,742,225 | 5,060 | 5,552 | 5,195,128 |

Across all ten blocks the tree needed 2.23–6.77 million comparisons for quote and 2.52–8.16 million for frequent. Live-map peaks ranged from 5,020 to 14,946 intervals. No coalescence occurred with this exact owner/cohort and input layout. This is an observation, not a general invariant. The full phase also pays indexed translation lookup, heap work, immutable group storage, band sorting and marked output. A materially different way to determine neighboring live rows might save tree work; the rank construction alone does not predict a throughput gain.

The allocated-byte counters include both directional translation arrays, allocated standard-library map nodes, group and heap capacities, published bands and vector growth transients. The marked peak additionally covers the emitted n-byte vector. The counters exclude the caller's original input runs, selection, expected output, verification orbit/words, allocator metadata, and stack. Input runs cost 8r bytes here. Recorded process CPU is a single unpinned instrumentation screen per block, not a paired worker result. The section interval itself is computed from the original runs by the separate selector and its work is logged in inputs.csv. The source verifies without reading a cut optimum or return-word dictionary; real.py checks only the finished measured statistics against the model.

## Reproduction

Run at the repository root with a C++20 compiler and Python 3. The input directory contains the archived ten systematic irec tapes.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native
g++ -std=c++20 -O3 -Wall -Wextra -pedantic "$d/ranks.cpp" -o /tmp/astra-async-ranks
/tmp/astra-async-ranks
python3 -B "$d/real.py" /tmp/astra-async-ranks research/astra-e/dense/target/wikidata /tmp/astra-async
```

[Exhaustive receipt](small.txt) · [20 cost and identity checks](wikidata.csv) · [Input checksums and selection](inputs.csv) · [Native rank constructor](ranks.cpp) · [Independent dictionary scheduler](../schedule.py)
