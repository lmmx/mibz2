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

Wikidata required 793,903 indexed quote seeks with 13,365,888 contiguous run increments. The original binary seeks made 12,775,593 comparisons. More comparisons in contiguous memory still reduced quote discovery time in this screen. The sums of per-tape median discovery times were 112.56 ms for the binary/sorted control and 47.66 ms with both indices. For compound they were 157.25 and 64.81 ms. The source scatter zero-initializes its table at setup, then adds one ownership fill per section row and removed 1,404,961 and 1,041,941 sort comparisons respectively. The largest per-tape sum of retained construction-array capacities was 6,552,268 bytes on quote and 7,068,684 on compound with both indices. These capacities include the live-frontier buffers, select table, source and page directories, trie, bands and dictionary. Input runs, output, stack, temporary reallocations and allocator bookkeeping are additional. [Per-tape capacities and phase breakdown](summary.json)

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

## Matched native backward control

Backward Direct starts at each outstanding destination interval, follows LF across original maximal runs, reduces the candidate width at each run end, and stops at first section return. In the two tested structured sections its final width is uniform. The native control prices this whole operation, copying each *band's* word once (including repetitions), storing its byte range in the band and scattering to a 32-bit source table. Its original-L predecessor index has one entry per 64 source rows, matching the granularity of forward's F-select pages. All 20 sets of descriptors, width-weighted word multiplicities, original marked outputs and digests matched Python Direct before timing. The bitwise equality check also runs after every timed sample. [Native LF control](direct_native.hpp) · [Standalone comparison](direct-native-check.json)

| Forward / native backward Direct | Quote CPU [paired 95%] | Compound CPU [paired 95%] |
| --- | ---: | ---: |
| Indexed forward with trie-wide offset and length tables | 1.1757 [1.1363, 1.2165] | 1.3106 [1.2474, 1.3771] |
| Indexed forward with term ranges written into the trie | 1.1811 [1.1227, 1.2424] | 1.2978 [1.2185, 1.3823] |

Each row is a separate two-arm 16-lap crossover on CPU 3. The schedule repeats AB, BA, BA, AB per tape and section. The ratios compare each forward version with its concurrent backward arm. Their intervals do not supply a direct comparison of the two forward versions. The direct control collected 16,630,795 quote and 17,204,304 compound LF page increments. It writes 1,700,623 and 2,996,813 spelling bytes respectively; the forward constructors retain only one copy per distinct word. Forward's savings in materialized bytes did not pay for its discovery and terminal dictionaries in these complete section operations. [Paired native ratios](directions.json) · [All 640 first-run observations](directions.csv) · [Second run ratios](recycled-directions.json) · [Second run observations](recycled-directions.csv)

The second variant eliminates both trie-wide spelling-index arrays. Each child has a strictly larger node ID than its parent. Visit terminal nodes in decreasing ID order. Once a terminal's parent chain is read to spell it, no subsequent unprocessed path can include that node. Its two fields can immediately become the offset and length of the spelling. A final pass transfers those ranges to the terminal bands. A component tag is recoverable from its destination when auditing the bands. This saved up to 1,084,832 bytes per tape on quote and 2,026,704 on compound. The new maximum retained capacities are 5,467,436 and 5,041,980 bytes. All marked outputs and ordered tagged bands matched the indexed version on all 20 real cells, 33,717 small datasets and 732 page-boundary fixtures. [Real comparisons and retained bytes](recycled-verified.csv) · [Independent small-case checks](recycled-check.json)

The measured backward controls and the recycled source can be rebuilt and run with the variables from the reproduction recipe above. The complete unmodified indexed pair first ran at `fb58a94`, archived before the separate recycled run at `8a15a25`.

```sh
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic "$study/direction_price.cpp" -o "$work/directions"
python3 -B "$study/check.py" "$work/forward" RECYCLED_BANDS > "$work/recycled-check.json"
python3 -B "$study/recycle_check.py" "$work/forward" "$work/wikidata" "$work/recycled-verified.csv"
taskset -c 3 python3 -B "$study/directions.py" "$work/directions" "$work/wikidata" "$work"
taskset -c 3 python3 -B "$study/directions.py" "$work/directions" "$work/wikidata" "$work" --recycled
```

## Sharing one displacement across a fragmented list

A fragmented group sometimes has an affine translation even though its packets can never coalesce. Let the enclosing current interval be [a,z). If the F-select run containing a also contains z−1, every packet has the same displacement δ and emits the same byte. If [a+δ,z+δ) misses U, all packets survive. Keep the source list in place, append that shared trie byte, add δ to one group offset and try again. Materialize the offset only when a run boundary or possible section re-entry ends the stretch. This skips the per-packet payments in the de Bruijn lower bound using an implicit representation. At q=7,k=32 it could save 3,416 of the 3,649 independent packet inputs on that witness. [Exact pre-implementation certificate census](hull.csv) · [Falsifier](hull.py)

Across the ten real tapes, the executable shared 18,465 quote groups and 69,844 compound groups. It removed 75,318 and 394,691 net per-packet operations after charging one shared operation per accepted group. The previously predicted count matched exactly on every tape and section. All first-return bands and marked outputs matched the unshared version. Skipping those operations left the count and location of **every full per-page select search** intact. The first packet in an accepted group already located that run; every later packet passed the original retained-run check. For a rejected hull the implementation reuses its first page lookup in the ordinary packet path. Both arms therefore execute the same page seeks and page-run increments, an identity verified separately at every real cell. The savings arise in retained-run checks, intermediate packet copies and loop work. [Actual probes, failed checks and deferred groups](lifted-verified.csv) · [Small-case and page-edge validation](lifted-check.json)

| 24-lap matched CPU ratio [95%] | Quote | Compound |
| --- | ---: | ---: |
| Shared displacement / unshared recycled forward | 1.0616 [0.9126, 1.2350] | 0.9810 [0.9015, 1.0676] |
| Shared displacement / native backward Direct | 1.2594 [1.1041, 1.4366] | 1.2577 [1.1187, 1.4141] |

The three arms shared the same native binary, CPU 3 and tape. They occupied all three positions equally over 24 alternating forward/reverse rounds per section. The interval uses the paired sums across the ten inputs and 23 degrees of freedom. The contrast between the two forward versions remains unresolved. The loss against backward Direct is present for both selected sections. This screen provides no whole-worker candidate for independent production benchmarking. [All 1,440 timed outputs](lifted-price.csv) · [Unrounded intervals and phases](lifted-price.json) · [Paired harness](lift_price.py)

A stronger follow-up would need to avoid locating different select runs, or avoid constructing the prefix states that cause those locations. A falsifier is to census normalized packet layouts **spanning multiple select runs** across different prefixes and count reused run/offset vectors only when they resolve a full indexed seek. It must charge recognizing a shared layout, its memory, and first-return boundary checks. Simple one-run translation has exhausted that particular source of savings.

```sh
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic -DFORWARD_HULL "$study/forward.cpp" -o "$work/hull-count"
python3 -B "$study/hull.py" "$work/hull-count" "$work/wikidata" "$work/hull.csv"
cmp "$work/hull.csv" "$study/hull.csv"
python3 -B "$study/check.py" "$work/forward" HULL_BANDS > "$work/lifted-check.json"
taskset -c 3 python3 -B "$study/lift_price.py" "$work/forward" "$work/directions" "$work/wikidata" "$work"
```
