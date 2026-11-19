# First-arrival ownership for two-sided first returns

The two moving heads of a section arc can meet only at the current opposite head. A permanent occupancy bit and an immutable first-arrival record therefore recover the exact meeting. This eliminates opposite-live-set predecessor queries, eager splits and deletion. A second permanent bit makes retirement lazy. A section row has two separate root incidences, which need independent retirement. [Proof and root counterexample](proof.md)

The fixed largest-mass scheduler and marked writer come from the C++ tree control at `8f1be9d4067b9e0eb4d1bb627bf1c764a2786218`. Every constructor starts from original runs, section intervals and a mark. LF and Ψ construction, group priority, output-band sorting, mark location, escape for a disjoint marked cycle, allocation and byte emission all run inside the native comparison. The model covers the marked BWT reconstruction stage. The quote and compound interval selectors run outside this native panel. A complete bzip2 worker additionally handles compressed input, RLE1, CRCs, output transport and worker concurrency.

## Structural price

The ten systematic Wikidata tapes are ordinals 0 through 216 in steps of 24. The two sections are the quote interval and the most frequent nonempty quote-first-return word interval (called `frequent` in the tapes). The table sums all ten blocks in each column except the per-block maxima. The flat history uses 64-row pages. [Tape and mark hashes](../native/inputs.csv) · [Exact counters by block and page width](wikidata.csv) · [Original tree counters](../native/wikidata.csv)

| Quantity | Quote | Compound |
|---|---:|---:|
| First interior arrivals | 7,642,848 | 8,754,470 |
| Meetings at an interior / section incidence | 1,357,061 / 6 | 245,445 / 0 |
| Original map comparisons eliminated | 34,019,163 | 39,072,948 |
| Original eager residual splits eliminated | 23,653 | 9,722 |
| History page-clipped descriptors appended | 1,004,994 | 1,161,397 |
| Positive returned fragments / descriptor inspections | 104,149 / 736,562 | 69,164 / 515,433 |
| Bitmap word reads / writes | 3,206,115 / 1,109,147 | 3,566,040 / 1,230,561 |
| Lazy skipped ranges / rows | 24,281 / 924,241 | 10,480 / 42,156 |
| Whole packets discarded when their group was empty | 38,069 | 47,250 |
| Expanded cohorts / total cohorts | 538,868 / 565,504 | 604,132 / 633,817 |
| Published bands, tree / history | 84,317 / 104,153 | 66,373 / 69,164 |
| Max simultaneous packet nodes, tree / lazy history | 14,946 / 16,838 | 12,034 / 13,555 |
| Max historical descriptors on one block | 181,734 | 219,987 |

Smaller history pages shorten scans but fracture owner ranges. At P16/P64/P256 quote needs 1,321,116 / 1,004,994 / 925,928 descriptors, and makes 362,569 / 736,562 / 1,925,361 descriptor inspections. Compound shows the same tradeoff. The zero-mass and collision bitmaps need no mutable ordered set. Active packets, group masses, monotone cohort parent links, priority queue and destination/owner bands still cost space and instructions.

## Native reconstruction and lifetime price

Flat records store 16 bytes in a linked archive plus one head per page. Packed records use 8 bytes with 24 bits each for owner and cohort and 16 for the page position and width. Each page owns a vector. The bzip2 block ceiling is 900,000, and every cohort beyond the roots requires at least one fresh interior row. Both 24-bit fields suffice. Packed-reclaim compacts a full bucket under growth pressure using the mass-zero theorem. No eager packet split or opposite-live-set deletion returns.

| Local process CPU ratio, 16 paired laps | Quote (95% interval) | Compound (95% interval) |
|---|---:|---:|
| Flat history / tree | 0.607 (0.441–0.835) | 0.592 (0.570–0.616) |
| Packed history / flat history | 1.067 (0.928–1.228) | 1.007 (0.959–1.058) |
| Packed history / backward Direct | 6.853 (4.544–10.335) | 6.450 (6.219–6.689) |
| Packed-reclaim / packed history | 1.074 (1.042–1.107) | 1.055 (1.032–1.079) |

In the four-arm panel each cell has a warm-up and 16 laps on one pinned logical CPU. Each arm occupies each position four times. The separate two-arm panel uses balanced AB/BA/BA/AB ordering and measures the complete construction and emission through the end of the writer. The four-arm outer measurement also includes copying the result into the driver's common output type and destroying the constructor. Gold equality is checked for every timed arm outside its timed region. Each lap sums ten blocks before taking the log ratio. Intervals are Student t on the 16 paired log ratios. Direct uses its own indexed backward construction and writer from `fd2d04719fb696dd4b8d0b5cebeb000caf8032e6`. Its internal stage columns in the four-arm CSV use steady-clock time. The tabulated whole-arm CPU for every arm uses the process CPU clock. The wide quote interval limits identification of close layout differences. [Paired times](paired.csv) · [Four-arm summary](paired.json) · [Reclamation pairs](reclaim-paired.csv) · [Reclamation summary](reclaim.json)

| Maximum across individual blocks, quote / compound | Tree | Flat | Packed | Packed-reclaim | Direct |
|---|---:|---:|---:|---:|---:|
| Isolated process peak, KiB | 16,596 / 19,248 | 19,584 / 23,552 | 18,140 / 21,496 | 15,764 / 19,240 | 9,460 / 9,236 |
| Tracked requested constructor bytes | 11,354,312 / 17,507,640 | 16,514,368 / 22,433,632 | 14,130,252 / 20,864,092 | 13,327,340 / 19,502,076 | uninstrumented |
| Descriptor capacity, bytes | — | 4,194,304 / 4,194,304 | 1,930,368 / 2,344,816 | 783,520 / 981,488 | — |

The fresh-process measurement reads `/proc/self/status` VmHWM inside each executable. It includes the native program, its input and expected output. The earlier four-arm and single-arm receipts retain a `rusage_maxrss_kib` field that reflects the larger process high-water mark inherited at spawn from the Python driver. The table uses the independent VmHWM receipt. The tracked allocator covers registered C++ vectors, packets, edges, cohorts and bands. Direct uses a different allocator and has no comparable tracked-byte value. Descriptor capacities exclude the outer page vector headers. [Fresh process measurements](fresh-proc.csv)

Reclamation scanned 1,050,408 records to discard 556,942 on quote, and 1,194,560 to discard 616,312 on compound. Positive descriptor inspections decreased only from 736,562 to 720,794 and 515,433 to 491,696. Reclamation buys a smaller archive at the measured CPU cost above. Asynchronous ownership recovers the same schedule while eliminating the live tree, yet the complete native marked inverse is still substantially slower than backward Direct. Neither layout justifies an i9 decoder performance submission on this evidence. A change to the reconstructed object, such as emitting directly owned spelling tokens, can be judged separately against Direct.

## Correctness and reproduction

The exhaustive native check verifies 146,752 labelled permutation/section cases and 763,396 marked outputs. It includes arbitrary labelled permutations through size 5 and binary last columns through size 7 with every nonempty marked section. It compares seam destinations, left/right depths and joining bytes with the frozen tree, and checks output against independently traversed marked cycles. A second set of 312 random permutation/page/section cases crosses 16-, 64- and 256-row boundaries and checks 45,672 marks on packed-reclaim. Real Wikidata checks cover all source endpoints, including the source owner, destination owner, spelling depths and join byte at each endpoint. Heap choices are deterministic under the proven mass invariant. The real checks assert matching pop, repush, expanded-state and traversed-edge counts as well as exact bytes. [Exhaustive receipt](exhaustive.txt) · [Page boundary receipt](packed-page-check.txt) · [Packed real seam receipt](real-seams.csv)

Run from the repository root with the systematic run tapes previously generated by the Astra C input recipe. The paths below use the campaign's local tape cache. Set CPU to an available pinned logical CPU for replication. Benchmarks use g++ 14.2.0, C++20 and -O2. The original four-arm timing binary SHA-256 was `9870033b4cd717f1e62c213d906a2b89d01744055806485ec45bded666ec65c8`. The fresh-process match and reclamation binaries were `fc326d70b6cd2d0d3b3818d56a779d5f3dfab4fef8505fe60674b456edf4f89c` and `01d9475b5f548312372af969559209083a09ca93829f03c31308aa5cc7ff979f`. Later match source adds the optional process high-water counter. [Systematic tape inputs and frozen control](../native/README.md)

```bash
p=research/astra-e/formulations/f2-heaviest/orientation/seams/async/history
t=research/astra-e/dense/target/wikidata
out=$(mktemp -d)
bash "$p/build.sh" "$out/bin"
"$out/bin/history" > "$out/exhaustive.txt"
"$out/bin/properties" > "$out/packed-page-check.txt"
python3 -B "$p/real.py" "$out/bin/history" "$t" "$out/wikidata.csv"
python3 -B "$p/verify.py" "$out/bin/properties" "$t" > "$out/real-seams.csv"
taskset -c 3 python3 -B "$p/price.py" "$out/bin/match" "$t" "$out"
taskset -c 3 python3 -B "$p/reprice.py" "$out/bin" "$t" "$out"
```
