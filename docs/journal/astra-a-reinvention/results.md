# What the whole-domain operation costs

## Exact reconstruction and structure

The operation learns a variable-length word and its exit row by joining two already emitted spans. In the selected version a word must apply across an entire original run image. A failed containment permanently settles that word. The reference uses only capped MTF run records, the block length and the primary. It emits forward pre-RLE1 bytes. [Proof and formulas](shortcuts.md) · [Executable](whole.py)

| Quantity | Frozen | All 233 large blocks |
|---|---:|---:|
| Pre-RLE1 bytes | 899,931 | 209,697,662 |
| Cells binary cuts under forced induction | 92,570 | 27,439,756 |
| Successful whole-domain compositions | 25,283 | 6,379,770 |
| Compositions where both operands already span multiple bytes | 9,319 | 2,200,612 |
| Permanent failed-extension certificates | 6,902 | 2,020,860 |
| Output spans | 117,399 | 34,619,893 |
| Copies from completed output | 40,731 | 11,134,412 |
| Bytes supplied by those copies | 823,263 | 186,212,181 |
| Copies of at most 16 bytes | 30,404 | 8,854,424 |

Every pre-RLE1 byte agrees with pinned Plain inversion. Independent per-block libbzip2 checks after RLE1 cover 209,691,304 final bytes. The stream SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. The 198 smaller blocks in this 431-block stream are outside the large-block population. The frozen block also matches the pinned compressed envelope, last column and final hash. [Input manifest](evidence/input.csv) · [All whole-domain results and digests](evidence/whole.csv) · [Split-model control](evidence/shortcut.csv)

A successful update adds k bytes to the certified word on w rows. Summing wk gives 4,054,290 row-steps on frozen and 856,600,801 on the corpus. These are consequences of the proofs across all entrances. Actual savings depend on which entrances the walk visits and on the cost of finding them. The corpus includes a source copy of 8,676 bytes. The longest one actually issued on frozen is 431 bytes.

The fixed-domain arrays take 688,864 bytes on frozen, plus 112,772 bytes of delivered records and 899,931 bytes for the inverse output. Across 233 blocks the array footprint ranges from 563,996 to 1,385,756 bytes, with median 719,192. Seven or ten frozen workers would touch 4,822,048 or 6,888,640 bytes of this metadata alongside their input and output. The number of wide roots can be counted in the first record pass, so the model needs no geometric growth during decoding.

**Recognition and admission.** All 233 large blocks take the pinned run-record frontend. The existing entropy decode, symbol sampling, RUNA/RUNB and MTF processing precede both kernels. Each candidate pays for two passes over the capped records. The first validates their ranges and tiling and accumulates byte and root counts. The second scatters the forward roots. Colour selection and bigram statistics belong only to the rejected experiments. The sparse and density gates can be applied to the input record count before either pass. On this corpus the lazy pre-gates accept exactly the 228 blocks lazy Cells finishes. The eager density gate rejects 35 and two more (202 and 206) exceed its subsequent induction budget. An alternative completion cannot use those two budget outcomes as a free admission test. We report structural checks for all blocks and compare costs only on actual Cells admissions. A production policy beyond this population remains open. [Pinned frontend](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/src/runs.rs) · [Historical block gates](../2026-11-18-cells-live-reuse/blocks.csv)

## Pinned comparison and address model

The priced sample is every 24th large block. Eager admits eight of the ten, lazy nine. Ordinal 24 compares as a forced structural control for both, and ordinal 144 for eager. The shared-worker comparison uses actual, separate large blocks in the fixed order 0, 48, 72, 96, 120, 168, 192, 216, 1, 2. The last two extend the eight jointly admitted spaced blocks to ten workers. Both are selected by ordinal before pricing. Worker counts in the simulation are the actual numbers of concurrent blocks. The pinned stream policy can trim the number requested by the caller.

The model replays the Study 7 eager and lazy setup and induction traces. It maps their region-relative addresses back onto the pinned release buffers, reusing bottom storage as output and the top slots as the writer's positions. It follows the marked spine, the one reverse rule scan and the deferred copy pass in program order. It includes the position fill, both 8-byte fields of a deferred command, the literal table, short payload stores, every source line of a copy and every destination line. The ledger credits Cells with free common capped-record reads and validation scans and charges the candidate for its records. Both arms get reusable buffer capacity. Allocation-time zero fills, allocator/OS traffic and memcpy's internal instruction order are outside the model. Stored data are written and tracked as dirty. The earlier review's one-line ambiguity between commands of one parent does not arise in the shipped last-use schedule. [Baseline adapter](baseline.py) · [Pinned owner writer](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/src/cells_owned.rs)

**Single-worker model.** All data accesses issue 64-byte line references. A cold, fully associative LRU uses write allocation and drains dirty lines at completion. Each capacity is replayed separately. The 2 MiB capacity represents one tenth of nominal shared capacity only as a sensitivity. Q8 charges ceil(span bytes/8) to each addressed operand, counting both reads and writes. This is a resident-memory work metric with an explicit price for the many short emissions.

| Admitted sample | Addressed bytes, Cells → whole | Q8 units, Cells → whole | 32 KiB transfers, Cells → whole | 256 KiB transfers, Cells → whole | 2 MiB transfers, Cells → whole |
|---|---:|---:|---:|---:|---:|
| Eager, 8 | 140,448,366 → 79,020,344 | 23,435,164 → 12,312,809 | 4,030,989 → 2,346,708 | 2,687,197 → 1,212,345 | 736,017 → 414,260 |
| Lazy, 9 | 157,110,988 → 96,785,593 | 28,145,467 → 15,155,735 | 4,771,771 → 2,938,596 | 2,650,880 → 1,494,775 | 670,098 → 483,129 |

Every admitted sample block favours whole domains in total line transfers at each of the four capacities tested. Frozen illustrates a cost hidden by the sum. At 32 KiB whole domains have 198,333 read fills, above lazy's 152,136. Their 25,093 store fills and 54,895 dirty writebacks are below lazy's 96,425 and 185,712. Of whole domains' 317,081 cursor-dependent metadata reads on frozen, 172,771 miss at 32 KiB and 58,485 at 256 KiB. Data-dependent output-source reads are additional. Across the nine lazy sample blocks the dependent metadata reads alone miss 694,309 times at 256 KiB. Charging one extra unit to each of those misses consumes 60% of the 1,156,105-transfer advantage if a transfer costs one unit. A premium of 1.67 transfer units per such miss exhausts it when every baseline stall is credited as free. This is a sensitivity bound for that price model. [All line counts](evidence/cache.csv) · [Addressed operands by phase](evidence/operands.csv)

**Interleaved blocks.** Each worker has a 256 KiB private LRU and its own address space for block buffers. They share one fully associative, noninvalidating 20 MiB LRU. One line-reference opportunity per active worker per round defines the interleaving. On a private miss the worker writes back a dirty eviction first, then requests its line. A complete private writeback can allocate dirty in the shared cache without reading memory. The count below adds shared demand reads and final dirty writebacks to memory, including all private drains.

| Workers | Eager Cells | Lazy Cells | Splitting domains | Whole domains | Whole reduction vs lazy |
|---:|---:|---:|---:|---:|---:|
| 7 | 615,487 | 455,564 | 523,950 | 360,596 | 20.85% |
| 8 | 710,200 | 523,984 | 603,374 | 414,260 | 20.94% |
| 9 | 807,759 | 589,314 | 681,360 | 467,049 | 20.75% |
| 10 | 915,089 | 655,508 | 766,564 | 521,084 | 20.51% |

At ten workers the split variant takes 16.94% more model memory traffic than lazy Cells even though it finishes in fewer visits than whole domains. Retaining a smaller family of provable words changes that outcome. At seven workers the whole-domain advantage is 94,968 lines (6,077,952 bytes) for the seven blocks together. [Interleaved receipts](evidence/teams.csv) · [Cache model](cache.cpp)

A reuse sensitivity processes three waves, rotates the blocks among workers by one position per wave, preserves each worker's buffer addresses and all caches, then drains once. With a full 20 MiB available and seven workers, the three-wave total is 369,915 lines for whole domains and 544,170 for lazy. At ten workers it is 535,835 and 999,704. With 10 MiB available the corresponding totals are 812,217 vs 1,345,079 and 1,376,706 vs 2,367,222. The very low warm-wave counts in the 20 MiB case depend on the isolated kernel retaining its metadata and output lines. The entropy frontend, RLE1, CRC, published buffers, conflict misses, prefetch, coherence and the actual i9 access order can all change residency. These experiments establish a structural traffic candidate and leave end-to-end throughput unmeasured. [Per-wave reads, writes and final drain](evidence/warm.csv)

## Arithmetic and output sensitivity

The fixed-domain pass still performs two or three serial metadata reads per visit and its output has many small spans. On the 228 lazy admissions it emits 33,219,053 spans against 7,896,055 for the one-scan writer. Its 10,782,634 output copies cover 182,761,375 bytes, against 2,467,541 copies covering 159,017,604 bytes. A lower transfer total can coexist with a slower serial path.

The following additive resident model counts Q8 memory work and charges scalar decisions explicitly. The coefficients are model prices. Hardware instruction counts require a native implementation. Let m be physical input records, r original domains, u singleton domains, w wide domains, t tiles, p visits, U and V singleton and wide visits, I installations and D permanent failures.

| Whole-domain component | Scalar units assigned |
|---|---:|
| Capped-record scans | 10m, five per scan for byte, length, change test, width accumulation and loop |
| Setup per domain | 13r for histogram/width recognition, cursor updates, scatter and start-mask arithmetic, plus 2u for packing a singleton and 8w for wide addressing and field packing |
| Prefix and tile counts | 3t + 768 |
| Every visit | 16p, including tile/bit selection, mask/popcount and root arithmetic, tag/emit tests, logical cursor and carry control |
| Class-specific reads and dispatch | 2U + 6V for extracting singleton or packed wide fields |
| Candidate tests and publication | 5(I+D) for endpoints, tests and dispatch, plus 7I for composing lengths, shifts, source and packing and 2D for marking a failure |

For Cells, take the pinned Study 7 setup-plus-induction scalar count after subtracting its field operations. Add the Q8 ledger including its writer. As a low reference, give the writer free scalar control. A second reference charges a scan of R rules, E=R−1+K rule edges, L=R+1−K terminals and K copies. It assigns 6R for walking backward and deriving the right position, 4R for the two operands' references and arena tests, E same-descriptor tests, L terminal lengths, 2(R−1) owner index operations, 2K command checks/updates, and 9K for command decoding, positions, span checks and iteration. Either formulation leaves compiler fusion, spills and additional short-copy dispatch open. [Exact multiplicities and formulas](costs.py) · [Per-block price inputs](evidence/resident.csv) · [Previous scalar schedule](../2026-11-18-cells-endpoint-sweep/ledger.py)

| Population | Whole-domain units | Cells with free writer control | Cells with priced writer control |
|---|---:|---:|---:|
| Eager, 8 | 48,487,963 | 43,281,706 | 47,707,111 |
| Lazy, 9 | 60,369,266 | 60,201,299 | 65,506,392 |

With the priced writer, resident work rises 1.64% against eager and falls 7.84% against lazy. Against lazy the nominal margin disappears at 5.34 extra units per excess output span. Against eager the 780,852 additional resident units break even at 0.53 units per saved 256 KiB transfer in this additive model. Variable-length copy setup, dependent misses and work overlapping between workers will determine the actual prices.

## Handoff to MONODROMY and the Researcher

The proposition for independent verification is that permanent-domain composition can discover enough long returns from two already written spans to cut shared memory traffic on real Wikidata blocks. Recompute the frozen 25,283 installations, 6,902 terminal failures, 117,399 visits and exact bytes from the capped input. The five-row counterexample in the small receipt tests the distinction between a known next byte and a known exit. The adversarial performance question is whether the greater number of source-copy dispatches and dependent reads erases the transfer saving. CI and the i9 can settle timing once the structural comparison has been independently reproduced. [Reproduction driver](reproduce.sh) · [Small counterexamples](evidence/small.json)
