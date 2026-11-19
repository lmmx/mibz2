# Forward first returns from the original run column

A smaller prefix dictionary translated into less run work for the quote section on all ten systematic Wikidata tapes. The most frequent quote-return marker behaved differently. Across the ten compound sections, visiting 9.85% fewer dictionary nodes required 31.62% more run intersections in the forward direction. The difference is the number of disjoint intervals needed to represent each prefix's image under Ψ. [Exact counts and output digests](real.csv) · [Directional dictionary oracle](../wikidata.csv)

| Ten-tape total | Quote | Compound |
| --- | ---: | ---: |
| Proper prefixes visited forward | 672,559 | 1,168,644 |
| Proper suffixes visited backward | 908,546 | 1,296,362 |
| Current forward packets located | 904,652 | 1,807,860 |
| Forward stable-select run intersections | 988,959 | 1,874,223 |
| Backward contiguous frontier run intersections | 1,068,084 | 1,423,936 |
| Hops in independent band-at-a-time backward Direct | 1,700,623 | 2,996,813 |
| Final source/destination affine bands | 84,317 | 66,373 |
| Materialized distinct spelling bytes | 1,077,638 | 1,926,457 |

For frozen ordinal 0, the quote goes from 70,560 backward nodes to 49,806 forward nodes. Those forward nodes locate 65,679 packets and intersect 71,811 original select runs against 81,863 original runs in backward frontier discovery. The compound section goes from 100,644 to 83,123 nodes, yet produces 141,778 intersections against 110,061. The backward contiguous frontier begins one original-run search per proper suffix node. The two directional lookup counts are nearly equal in total on quote. Tags, original sorted source bands, returned destination, width, byte spelling and marked output were verified independently on all twenty section/tape combinations. [Exact constructor](forward.cpp) · [Backward comparison and verification](census.py)

## Representation and invariant

A stable counting pass places each original maximal L-run into its byte's slot in F order. If the run starts at L-row ℓ with width h and its first F rank is f, Ψ translates [f,f+h) to [ℓ,ℓ+h) by ℓ−f and emits the F byte. This costs two passes over the supplied runs and 256 counters per table. It constructs no reversed-text BWT, rank table over all rows, or explicit Ψ permutation. The quote section is its F bucket. The compound section uses the original-run heap to select the most frequent first quote-return word, with smallest source rank breaking ties. Heap work is charged separately; both directions receive the same selected section. [Run-only input adapter](../../../../../../docs/journal/2026-11-18-astra-c-output-first/inputs.py) · [Section selector and Direct oracle](../../union/direct.py)

A live packet (s,y,h) with prefix u certifies that every s+i, for 0≤i<h, has emitted u and reached y+i without re-entering U. At the root, y=s and the source packets are precisely U’s components. Intersect the current interval with the stable-select runs, append the F byte once for the associated trie edge, and translate each intersection. Split the result against the half-open component boundaries of U. Each hit supplies a source interval, affine target, terminal spelling and destination-component tag. Every remaining piece is a packet at the extended prefix. This permits partial hits even in arbitrary rank subsets where the historical uncut backward Direct oracle has no uniform-membership guarantee.

For a fixed emitted byte, Ψ is a strictly increasing integer function in F rank, since it selects consecutive occurrences of that byte in L. Thus the current intervals of each prefix arrive in sorted order, its next bytes arrive in byte order, and the children for one byte are consecutive in the construction stream. Two adjacent source packets with a positive image gap remain separated on every subsequent common extension: a strictly increasing integer map cannot shrink a positive gap below one. A run cut within a byte introduces a nonempty gap between the two L-runs. A source gap cannot be repaired by translating it. The constructor coalesces every same-prefix consecutive source/image pair it encounters. There were zero such coalescences in the exact real census. This persistence explains why counted prefix nodes alone can underprice construction.

Marked expansion walks forward from the original mark to the first visited section row, emitting the intervening F bytes. If the mark’s cycle misses U, the scientific constructor returns a refusal. Otherwise the stored affine return bands spell the cycle, truncated or repeated at precisely n output bytes. Across multiple cycles, Σ width × word length over the bands equals the number of original rows on all cycles meeting U. Each actual translation can be charged to a distinct source occurrence and offset on those disjoint arcs, giving an upper bound of that many translations. The two-word de Bruijn family gives a matching order-of-growth obstruction for explicit packet discovery. [Proof, counterexample and bounds](family.md)

## Charging the index and writer

The baseline locates the first F-select run for a monotone packet by first trying a retained or adjacent run, then using binary search if needed. It sorts the terminal bands by original source and performs one binary search per subsequent token. The page variant records the first select run at every 64th F row and scans forward from that entry on a finger miss. The source-table variant allocates a packed section coordinate for every U row and fills it once with its owning band. Mark and return traversal then read a direct source entry, and the sort disappears. The native crossover independently toggles both changes. [Indexed structure and exact section-write counts](indexed.csv) · [Unrounded phase measurements](price.csv)

| Indexed variant / binary-and-sorted control | Quote CPU [paired 95%] | Compound CPU [paired 95%] |
| --- | ---: | ---: |
| Select pages only | 0.6777 [0.6132, 0.7489] | 0.5510 [0.5287, 0.5743] |
| Direct source table only | 0.9396 [0.8744, 1.0097] | 0.9336 [0.9089, 0.9590] |
| Both | 0.5585 [0.5353, 0.5827] | 0.5008 [0.4870, 0.5149] |

This 16-lap, four-arm native crossover ran on CPU 3 of the local Xeon Platinum 8573C with G++ 14.2, at one thread. Each block and section got one untimed warm invocation per arm. Each tested arm occupied every position four times with reversed order in alternate four-round groups. Each lap pairs the sums over the ten tapes, with the 95% Student interval computed on paired log ratios. The native section operation constructs the select index from original runs, discovers and orders the returns, materializes distinct spellings, transfers the mark, emits all bytes, and drops its constructor arrays. Input parsing, selecting U, JSON and result verification are outside the timer. Every timed output matched the untimed control. Whole-worker concurrency, CRC, output delivery, process RSS and the i9 remain unmeasured for this formulation. [Summaries and source](summary.json) · [Pricing protocol](price.py)

Wikidata required 793,903 indexed quote seeks with 13,365,888 contiguous run increments. The original binary seeks made 12,775,593 comparisons. More comparisons in contiguous memory still reduced quote discovery time in this screen. The sums of per-tape median discovery times were 112.56 ms for the binary/sorted control and 47.66 ms with both indices. For compound they were 157.25 and 64.81 ms. The source scatter zero-initializes its table at setup, then added one ownership fill per section row and removed 1,404,961 and 1,041,941 sort comparisons respectively. The largest per-tape sum of retained construction-array capacities was 6,552,268 bytes on quote and 7,068,684 on compound with both indices. These capacities include the live-frontier buffers, select table, source and page directories, trie, bands and dictionary. Input runs, output, stack, temporary reallocations and allocator bookkeeping are additional. [Per-tape capacities and phase breakdown](summary.json)

## Checks and reproduction

The small-case sweep covers 33,717 marked datasets, including overlapping word markers, genuine cyclic BWT inputs, arbitrary last columns, multiple LF cycles, and marks on cycles disjoint from U. Both forward implementations agreed with rowwise LF and Ψ first-return oracles on the 26,193 outputs and 7,524 refusals. The 940 arbitrary sections outside backward Direct’s uniform-interval domain have exact classified witnesses. The indexed version also passed 732 page-boundary cases. [Original witness receipt](native-check.json) · [Indexed and page-boundary receipt](indexed-check.json)

From the repository root use the following. The tape revision is `2e4b34e086ac95e8218f1ee822a4652c8db5bb19`. Each real row records the digest of its tape and output. The program receives only the original maximal runs, mark and selected intervals.

```sh
study=research/astra-e/formulations/f2-heaviest/orientation/forward
work="$(mktemp -d)"
git archive 2e4b34e086ac95e8218f1ee822a4652c8db5bb19 wikidata | tar -x -C "$work"
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic "$study/forward.cpp" -o "$work/forward"
python3 -B "$study/check.py" "$work/forward" > "$work/check.json"
python3 -B "$study/check.py" "$work/forward" INDEXED_BANDS > "$work/indexed-check.json"
python3 -B "$study/census.py" "$work/forward" "$work/wikidata" "$work/real.csv"
python3 -B "$study/family.py" "$work/forward" | cmp - "$study/family.csv"
taskset -c 3 python3 -B "$study/price.py" "$work/forward" "$work/wikidata" "$work" --rounds 16
python3 -B "$study/summarize.py" | cmp - "$study/summary.json"
```
