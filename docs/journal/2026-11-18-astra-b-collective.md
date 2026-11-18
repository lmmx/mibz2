# Astra B · Can dependent LF boundary interactions be evaluated collectively?

18 November 2026. Independent study based on mibz2 0.5.6 at `1edb29fd`.

**Yes, on identifiable regions. None of the constructions tested gives a convincing net reduction on Wikidata.** One exact family replaces a linear number of dependent cuts with a modular translation and three output fills. A second formula contracts all equal-byte excursions directly from the compressed input. Across ten real blocks that formula saves 39,549 of 1,236,053 cuts (3.20%). Its constructor then intersects 664,730 ranges to supply the residual map. Even with favourable credit for every induction saving, materializing its input and result costs more than it saves on each block. The obstruction measured here belongs to the representations tested. [Proofs](2026-11-18-astra-b-collective/proofs.md) · [Verification and cost definitions](2026-11-18-astra-b-collective/README.md)

## Starting hypotheses and experiment

The opening commit set three tests. First, compare LF with the strongest modular translation. Second, allow a different translation on every tile of an equal-width tiling and compute the cycle order algebraically. Third, contract exactly aligned paths as ranges, with no scalar exception record. After the frozen screen I tested a separate ordering property inside each first-column byte bucket.

The twelve hashed windows reproduce the pinned 11,854,933-byte Wikidata input. The shipped scanner and run front supplied the capped records directly. Frozen is large-block ordinal 0, stream block index 1, with 899,931 rows, 56,386 records and primary 264,045. The other nine blocks are every 24th large block through ordinal 216. Eight of the ten satisfy eager admission, nine satisfy lazy admission. The diagnostic marked induction reproduces the historical eager and lazy geometry field counts separately on every control. [Input and output hashes](2026-11-18-astra-b-collective/verified.csv) · [Pinned production sources](https://github.com/lmmx/mibz2/tree/1edb29fdc3d42dfddfc3d60e46456b611323760c/src)

The previous endpoint construction recovers each child at $F-\min(a,b)$ and sends about 99% of sampled newborns outside its active group. The boundary-orbit profile explains variation in the existing workload. The previous writer studies show how a saving in fields can lose to dependent reads. These observations motivate charging order, construction, spelling and traffic separately. [Endpoint study](2026-11-18-cells-endpoint-sweep.md) · [Verified arrivals](https://github.com/lmmx/mibz2/blob/276db91c46df7027534785562a89bdac669e635c/docs/journal/monodromy/2026-11-18-endpoint-sweep-verification.md) · [Boundary profile](https://github.com/lmmx/mibz2/blob/276db91c46df7027534785562a89bdac669e635c/docs/journal/monodromy/2026-11-18-q-profile.md) · [One-scan writer](2026-11-18-one-scan-writer.md) · [Owner schedules](2026-11-18-cells-owner-schedules.md) · [Verified ownership](https://github.com/lmmx/mibz2/blob/276db91c46df7027534785562a89bdac669e635c/docs/journal/monodromy/2026-11-18-owner-schedules-verification.md) · [First-occurrence experiment](2026-11-18-first-occurrence-writer.md)

## An exact replacement for scheduling

For LF permutation $T$ and reference permutation $Q$, let $E$ contain the disagreements and the primary. Put $S=Q^{-1}T$, and let $N$ take the next *positive* occurrence of $E$ along a $Q$ cycle. The exact return of $T$ to $E$ is $N\circ S$. With $u=S(e)$ and gap $g$ from $u$ to $N(u)$, the LF spelling is the sorted first-column byte at each of $Qu,\ldots,Q^g u$. A schedule of the intermediate reconstruction events plays no part in either identity.

When $Q$ permutes equal-width tiles with a cyclic shift of the lanes inside each tile, one lap of a tile cycle adds a single known lane shift. A gcd and a prefix sum give every row's cycle and phase. Reading exceptions by tile-cycle position and then by lap supplies their successors without a comparison sort. This still creates one successor per scalar exception. In the pure-rotation family $L=1^{n-1}0$, primary $n-2$, the run scan certifies $T(x)=x+1\pmod n$. The forward output $1^{n-2}01$ takes three fills. At $n=4,096$ this replaces 4,095 right-end cuts from an input of 34 capped records. [Factorisation proof](2026-11-18-astra-b-collective/proofs.md) · [Executable arithmetic and marked returns](2026-11-18-astra-b-collective/factor.py)

**Frozen screen.** The best global rotation agrees on 26,413 rows and misses 873,518. For tile widths 2, 4, 8, 16, 32 and 64 the run overlay bounds the agreement of *every* choice of tile partners, even granting arbitrary within-tile permutations. The six lower bounds on disagreements are 219,982, 250,289, 243,757, 260,701, 266,524 and 284,516. This avoids constructing a matching to find out that it cannot rescue the scalar index. The explicit return walk would issue at least 219,982 serial successor loads, before spelling the gaps. Cells supplies its two fronts in 143,755 refills. A refill and a successor load have different costs.

A packed model allowing eight bytes per tile and eight bytes per scalar exception needs at least **37,807 distinct 64-byte lines** on frozen, taking the best of the six rigid widths. This already grants free discovery, perfect packing, free row-to-exception addressing and free spelling. The archived setup-plus-induction footprints including words are 34,634 lines eager and 23,343 lazy. The scalar-index line stopped here. [Frozen oracle bounds](2026-11-18-astra-b-collective/frozen.csv) · [Other blocks](2026-11-18-astra-b-collective/sample.csv) · [Packed line counts](2026-11-18-astra-b-collective/ledger.csv)

## Parallel paths through whole tiles

A full tile inside one byte run whose LF image is another aligned tile can carry all its lanes through the same translation. Protect the primary's tile. Remove the clean sources, redirect incoming paths to their first retained tile and attach their common word. For each input run, two endpoint divisions identify the whole range of clean tiles. The difference of two ordered interval unions gives all path heads. Along a representative head, the nearest group boundary limits how many successive tiles can make the same trip. This computes a bundle directly from the original runs. It pays for every group search and for joining the resulting intervals to the retained input.

| Marked inversion | Frozen Cells | Frozen width 2 | Ten blocks, Cells | Ten blocks, width 2 |
|---|---:|---:|---:|---:|
| Initial marked bands | 51,184 | 53,430 | 690,484 | 715,805 |
| Right-end cuts | 92,570 | 93,746 | 1,236,053 | 1,190,715 |
| Stored W16 rules | 28,710 | 35,211 | 359,764 | 384,871 |
| Writer copy bytes | 735,582 | 701,528 | 6,838,153 | 6,532,251 |

On frozen, width 2 reduces support to **439,965 rows**. All six widths increase residual cuts and rules there, even with free transfer discovery. The extension to the prespecified sample changes the picture. Width 2 reduces cuts on eight blocks, including a saving of 13,462 on ordinal 144. This justified checking the strongest range version on those blocks. [Frozen residuals](2026-11-18-astra-b-collective/frozen-residual.csv) · [Per-block residuals at every width](2026-11-18-astra-b-collective/sample-residual.csv)

The range construction reduces frozen's 109,627 individual paths to 5,667 bundles, discovered with 11,206 interior group visits. Across ten it needs 69,413 bundles, 132,944 interior visits, 3,545,532 comparisons in dependent predecessor searches, and 722,975 range intersections to attach the words and destinations. The binary search touches only 406 distinct index lines on frozen and fills each once in a 32 KiB ideal cache. A comparison at the next address still depends on the previous result. Across ten blocks the 45,338 saved cuts also come with 25,107 extra stored rules. This stops the measured construction, including on the blocks where retaining fewer rows helped. [Range constructor and per-block visits](2026-11-18-astra-b-collective/packets.csv) · [Index traffic](2026-11-18-astra-b-collective/ledger.csv)

## Equal-byte excursions in two ordered queues

Inside a first-column bucket for byte $c$, all LF edges that stay in the bucket have one contiguous image. The number of $c$ bytes preceding the bucket determines a split. On one side internal arrows move toward higher ranks, on the other toward lower ranks. In the appropriate scan direction a queue of entrance indices rotates at an internal row and loses its front at an exit. For a span of $w$ internal rows and $a$ live entrances, division adds $\lfloor w/a\rfloor$ to every live path length. Only the remaining prefix changes places. Exit ranges carry their destination and their exact repetition count. A mark in the middle of an excursion requires its entrance and an offset. On 11000 the zero bucket's entrances 1 and 2 exit at 1 and 0, so their association must survive the computation. [Proof of the split, update and mark](2026-11-18-astra-b-collective/proofs.md) · [Range-queue implementation](2026-11-18-astra-b-collective/queue_ranges.py)

| Exact count | Frozen | Ten blocks |
|---|---:|---:|
| Internal equal-byte rows | 15,756 | 157,654 |
| Rows covered by whole revolutions | 7 | 6,132 |
| Queue fragments popped | 6,996 | 89,349 |
| Exit ranges stored and subsequently looked up | 4,640 | 58,968 |
| Retained-input / transfer intersections | 49,187 | 664,730 |
| Residual cuts | 89,708 | 1,196,504 |
| Residual stored rules | 28,683 | 359,396 |

The cut reduction is 3.05–3.55% per block. On frozen the writer still reaches 9,628 copies of 735,582 bytes. Across ten, stored rules decline by 368 and copied bytes by 96. In the 4,096-row rotation example a single internal span covers 4,094 visits and the queue quotient leaves one cut. On frozen the arithmetic compresses only seven visits into whole revolutions. [Frozen queue counts](2026-11-18-astra-b-collective/frozen-queues.csv) · [Other blocks](2026-11-18-astra-b-collective/sample-queues.csv)

## Cost model and stopping decision

The staged queue constructor writes and reads 48,853 kept source records and 48,931 residual records on frozen. Counting **each 12-byte record as one access** gives 195,568 accesses before the range index, queue updates or bottom-order construction. The geometry and word operations saved, with an additional setup credit for each removed input band, are 73,068 eager and 72,838 lazy. Over the eight eager-admitted blocks the comparison is 1,661,348 staged accesses against 635,595 credited operations. Over the nine lazy-admitted blocks it is 2,140,008 against 792,511. The staged count exceeds the credit on *every* sample block. This comparison already grants the residual its bottom order.

In a separate concrete layout, the two staged arrays and an open-addressed range index touch 22,431 distinct cache lines on frozen. They incur 73,306 fills plus dirty writebacks with a cold, fully associative 256 KiB cache. The frozen input itself spans 1,763 lines of capped records, the output 14,062. A static bitmap ranking the residual bottom ends would add 237,172 mask/directory field accesses before scattering the bottom handles. The line trace excludes queue nodes, those order arrays, the residual induction and the writer. Actual shared-cache, DRAM and latency effects at 7–10 workers remain unmeasured. [Layouts, component boundaries and formulas](2026-11-18-astra-b-collective/README.md) · [Per-block ledger](2026-11-18-astra-b-collective/ledger.csv)

**Scope of the negative result.** The disagreement bound applies to a scalar index with this reference family and grid. The tile and equal-byte results price specified range constructors. A fused consumer could avoid materializing some records. Its remaining question is how to provide both interval orders and marked words with lower discovery and traffic costs. The observations give no lower bound on such a consumer. The investigation stopped at structural checks, before a production kernel or a native performance trial.

## Verification

Every primary of every binary column through length 9 and ternary column through length 6 was checked, 14,209 cases in total. These include 12,192 short periods, 6,346 periods that do not divide the block length, 34,053 equality cuts and 11,006 discarded closures. The tests check transfer destinations and words at every retained row, with 3,686 marks inside equal-byte words. The arithmetic reference passed 85,254 marked comparisons. Another 358 marked stress cases include high bytes, periodic texts and unaligned final tiles. All ten real blocks reproduce the independent libbz2 output after RLE1. Their compressed, pre-RLE1 and decoded digests are recorded. [Small counterexamples and receipts](2026-11-18-astra-b-collective/small.json) · [Real-block checks](2026-11-18-astra-b-collective/verified.csv) · [Reproduction](2026-11-18-astra-b-collective/README.md)
