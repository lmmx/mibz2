[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/index.md>)

# Revisiting the space–⁠time tradeoff in BWT inversion

Our first compact navigation table saved memory but made each lookup too costly. A redesigned record gives up a little density to make navigation cheaper—and the compact traversal is now faster under contention. Building it for the full decoder is the next test.

2 November 2026 · 12 min read · [Research result](<#the-first-positive-traversal-result>)

In 2001, Julian Seward published *[Space-time Tradeoffs in the Inverse B-W Transform](<https://doi.org/10.1109/DCC.2001.917175>)*. The problem is still recognisable in a modern bzip2 decoder. The Burrows–Wheeler transform (BWT) rearranges a block’s characters to make them easier to compress. To recover their original order, the decoder repeatedly looks up a row in a navigation table; the value it finds tells it which row to visit next. Keeping the information readily available makes each step quick, but consumes memory. Representing it more compactly saves space and usually makes each lookup more involved.

Seward investigated several implementations of that exchange. In a merged representation, the next-row information and the output character are kept together for fast traversal. An alternative, *indexF*, retrieves the character from the sorted first column rather than storing it beside every link. It saves working memory at the cost of more work during inversion. He also argued that inversion speed is limited by how fast cache misses can be serviced. Our [frontier page](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/frontier/index.md>) sets out the `mergedTL`/`indexF` distinction and where mibz2 sits on it. This is not merely a theoretical concern: [bzip2's own manual](<https://sourceware.org/bzip2/manual/manual.html>) records that a single-array inverse BWT "significantly speeds up decompression, presumably because it reduces the number of cache misses". Later research, including [Kärkkäinen, Kempa and Puglisi (2012)](<https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf>), attacked the number and cost of those misses more directly.

But the trade-off looks different when we change what is competing for memory.

In a [29 October journal entry](<https://github.com/lmmx/mibz2/blob/f384fb424b19e5e5e1c5e27559dd0eb916e98123/docs/journal/2026-10-29-frontier.md>), we asked whether a navigation structure that was slower for one isolated block might be faster when several blocks were being decoded concurrently. Four workers have four navigation tables, all using the same shared cache and the hardware behind it. A smaller representation might relieve that competition enough to pay for the additional instructions needed to read it.

The hope was not simply to choose another point along a space-for-speed compromise. It was to improve the [machine-level throughput–memory trade-off](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/frontier/index.md>): to see whether less navigation storage and greater completed throughput could coexist under the same workload. The difference is important. A faster traversal on a prebuilt table is evidence for this possibility; a faster *complete decoder* with lower measured memory use would be the stronger result.

Our earlier [investigation of memory interference](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-11-01-inverse-bwt-memory-interference/index.md>) made that question concrete. Four-worker decoding finished sooner than one-worker decoding but used substantially more total CPU time per decoded byte, despite retiring nearly the same instructions. We had a reason to suspect a shared-memory penalty. We did not yet know whether a compact representation could reduce it.

## The problem was never just storing fewer bytes

The conventional mibz2 navigation table, or *T-vector*, stores a next-row position and an output byte in a 32-bit entry. [The production traversal](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L927-L955>) advances through those entries by dependent loads. A maximum-sized block has 900,000 rows, so its table occupies 3.6 MB. Four such tables account for 14.4 MB before output buffers or other live data. The workstation used in the experiment has a 20 MiB last-level cache shared by its cores. That makes the table footprint worth investigating—but cache capacity alone cannot establish where individual loads are served.

In [*Compact navigation tables for the inverse BWT: early findings*](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-29-compact-navigation-tables/index.md>), we started with a simpler question: why four bytes for every row? The next-row mapping is not arbitrary. Within each group of rows beginning with the same character, successor positions are ordered, because construction preserves the order of equal characters. If thirty successive rows point to positions such as 1,000, 1,002 and 1,005, storing one base and the small differences may cost much less than storing every absolute position.

The first corpus census found substantial regularity. That made compression plausible, but we still needed a structure with random access to an individual successor. The decoder cannot decompress a whole group of pointers every time it wants to follow one: the next address is unknown until the current lookup completes. Our [32-lane walk](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-31-room-walk-overlap/index.md>) already overlaps work from independent segments to hide some of that wait. A compressed format must preserve that ability, not introduce another dependent search before each step.

The first attempt tested that constraint directly. FOR-14 packed thirty-two 14-bit offsets into each 64-byte record, with separate full-width entries for exceptions. Its [frozen experiment](<https://github.com/lmmx/mibz2/blob/7d7962f/docs/journal/2026-11-01-for14-stage1.md>) found just 2.076 logical bytes per row, a little over half the conventional table's size. That was good news for storage; it wasn't good news for traversal.

Each lookup now needed to find a fourteen-bit field within a packed record, reconstruct the next position and maintain a carried output byte. The compiled ordinary path contained 38 instructions, five data loads and three stores, against the conventional walk's 16 instructions, three loads and two stores. These are static hot-path counts, not the number of instructions retired per decoded byte. But they show why compactness cannot be judged on bytes alone: the decoder pays the cost of retrieving a *single* successor at every step.

And the first measurement went the wrong way. With one block, FOR-14's traversal took 2.676 ns per row against 1.275 ns for its control. With four blocks traversed concurrently, it took 2.899 ns against 1.991 ns. The smaller representation's cost rose by only about 8% from one block to four, while the conventional control's rose by 56%. This was the behaviour we had hoped a smaller table might show under contention—but FOR-14 started 1.40 ns per row behind. It still finished the four-block work about 1.45 times slower than its control. The experimental format was stopped, even though the broader hypothesis survived.

Rather than rush into another timing campaign, the subsequent [packed-token variants](<https://github.com/lmmx/mibz2/pull/130>) attacked the decoding overhead. Two alternatives compiled to 31 and 27 instructions per ordinary step, each with four loads and two stores. Their outputs were checked for correctness, but neither was timed. They established that some of the machinery could be removed; they did not establish another performance result.

The next design made a different space–time choice, not just a different instruction schedule. Instead of tightly packing thirty-two 14-bit fields, compact30 puts thirty directly addressable 16-bit offsets into sixty bytes and uses the remaining four bytes of a 64-byte cache-line record for a shared header. The header holds the base successor label and the character for rows in that group. The ordinary step adds the selected offset to the base. Groups crossing a character boundary, or too wide for the offsets, use a separate RAW representation with full entries in groups padded to 128 bytes.

Two details remove work that FOR-14 had carried into every step. First, an ordinary 16-bit field needs no packed-bit extraction. Second, the walker carries only its current row: the emitted character comes from that row's header, not from an output-byte token carried between lookups. Both the header and the chosen offset can be addressed from the current row, without waiting for one load to discover the other's address. The compiled step still performs more work than the conventional walk, but far less than the first packed version. The [annotated machine-code review](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md>) records the final path.

A compact record leaves four bytes for the base and output byte

Sixty-four bytes encode thirty ordinary row transitions. Exceptional groups have a separate representation. This shows only the common case; segment starts and parking are handled by the walker.

The engineering progression can be read in the compiled ordinary paths:

| Navigation layout | Instructions | Data loads | Stores | Evidence |
| --- | ---: | ---: | ---: | --- |
| Conventional 32-bit table | **16** | 3 | 2 | Production baseline |
| FOR-14, packed offsets | **38** (37 with BMI2) | 5 | 3 | Measured; rejected |
| FOR-14 token variants | **31 / 27** | 4 | 2 | Correct and compiled; **not timed** |
| Refined 30-row, 16-bit offsets | **21** | 4 | 2 | Measured; the first to pass |

The designs did not share an identical encoding or experimental protocol; the instruction counts are not points on a controlled timing curve. The FOR-14 trial's reported time ratios and compact30's later thread-CPU ratios are separate measurements, each against its own control. In particular, we have no performance measurements for the intermediate 31- and 27-instruction variants. What the sequence does show is the sustained effort to make compact *navigation* practical, instead of optimising storage in isolation.

We deliberately allowed the final record to grow a little: 2.164 logical bytes per row, about 4.2% larger than FOR-14's 2.076, while retaining a 45.9% reduction from the shipped four-byte entry. The [30-row reference census](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-census.md>) established exact reconstruction of 1,513 blocks across eleven corpora. Its original token packing and alignment were subsequently refined, and the finished [experimental walker](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md>) passed separate correctness and machine-code reviews. The census was not a timing result for that final walker.

At 21 instructions and four data loads, the compact30 common step is still more expensive than production's 16-instruction, three-load step when considered by itself. The question was whether lowering the table's shared-memory cost could more than compensate for that extra work as several cores traversed at once.

## The first positive traversal result

We compared the shipped T-vector traversal with the frozen compact30 walker on the established 434-block Wikidata corpus, covering 207 million inverse-BWT rows. Each output was checked against the production decoder's result. The [registered protocol](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md>) paired the two implementations across twenty measured rounds, first with one pinned physical core and then with four workers starting their traversals together.

The timed interval includes traversal and assembly of the decoded BWT block, not table construction. Before each walk, production builds its actual table; compact30 instead copies a reference-built compact image into its arena. Those preparation operations are excluded from the timer and may leave different cache states. What this experiment can tell us is how the two *prepared* implementations behave under its stated conditions—not the eventual cost or cache state of a direct compact30 builder.

The CPU-time figures add up how long the walking threads actually spent executing and divide by BWT rows. The completion figure instead measures the earliest start to the latest finish in each synchronized step, summed across the corpus. Neither includes table preparation or output validation. The 95% intervals describe paired variation across the twenty rounds, not a guarantee for other machines or workloads.

| Prepared-table traversal | Conventional | Compact30 | Compact ÷ conventional (95% interval) |
| --- | ---: | ---: | ---: |
| CPU time per BWT row, one core | 1.221 ns | 1.402 ns | **1.148** (1.147–1.149) |
| CPU time per BWT row, four concurrent workers | 2.055 ns | 1.727 ns | **0.840** (0.838–0.843) |
| Four-worker lockstep completion, one corpus pass | 114.45 ms | 97.97 ms | **0.856** (0.852–0.860) |

Extra lookup work is repaid under concurrent traversal

CPU time per row is measured for each walking thread and summed. Compact30 is slower alone but faster with four workers. The chart shows the reversal, not its cause; construction is excluded.

The reversal repays the design effort in a very specific setting. On one core the compact walker spends 14.85% more CPU time per row. With four concurrent walkers it spends 15.95% less, and their summed lockstep completion time is 14.40% lower. That completion measure is not a measurement of whole-file decoding speed.

The absolute times help explain the crossover without pretending to identify its cause. On one core, compact30 starts with an extra 0.181 ns per row of traversal cost. When the workload changes to four concurrent walkers, the conventional walk's cost increases by 0.834 ns per row, against 0.325 ns for compact30. The difference between those increases is 0.509 ns—larger than compact30's initial 0.181 ns disadvantage—leaving it about 0.328 ns per row ahead in the four-worker measurement. That is arithmetic on the two measured conditions. It does not isolate cache misses, memory traffic or instruction execution as the cause of the different increases.

This was not a result carried by a few convenient blocks. Of the 230 full-sized blocks, all 230 had lower four-worker traversal CPU cost with compact30 when averaged over rounds. In 1,138 of the 1,140 full-sized four-worker steps, the total CPU time was lower as well. The 204 short blocks were slower, but accounted for just 8,324 of the corpus's 207 million rows. The same blocks recurred in each round, so these counts are robustness checks within one experiment, not hundreds of independent replications. The complete [capture and independent reconstruction](<https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4>) are preserved.

It took some care to obtain a trustworthy comparison. Earlier attempts were stopped by toolchain and disassembly checks, and one complete capture was invalid because the four walks were not starting sufficiently close together. A subsequent, separately registered change to the start barrier corrected that condition without changing the walker or relaxing the threshold. In the valid capture every registered check passed. By the decision rule fixed before the run, that was enough, for the first time with a compact table, to justify building a real constructor. It was not a case for putting the experimental walker into the decoder.

## A promising traversal is not yet a better decoder

The measured crossover is the one we had asked about in October: a table that loses when traversed in isolation can become favourable when multiple workers compete for the machine's memory resources. What the experiment does not tell us is why their costs increase so differently.

A smaller shared navigation footprint is the leading hypothesis: approximately 7.79 MB for four full compact30 tables rather than 14.4 MB for four conventional tables. But this figure is calculated logical table storage, not measured cache occupancy, peak process memory or DRAM traffic. The experiment did not collect the counters needed to distinguish fewer evictions from translation effects, request-service pressure or the differing cache state left by table preparation. The contention-related advantage is observed; its precise hardware cause is not isolated.

This result is therefore narrower, and more valuable, than simply saying that compression made decompression faster. At a fixed concurrent traversal workload, we have measured a representation that uses fewer logical table bytes and completes the traversal with less CPU time. That is a new favourable point for this stage of the computation, despite more work per lookup. We have *not* yet demonstrated an outward shift in the full decoder's throughput–memory frontier—the set of complete-decoder configurations that are not beaten on both speed and measured memory use.

There is one more reason not to conflate the two. A real decoding worker also reconstructs the last column, builds the navigation table, assembles output and performs further processing. Workers are not guaranteed to be traversing at the same time. The 14.85% isolated traversal penalty could matter whenever only one or two are walking. The four-worker saving cannot be multiplied through the whole program without measuring that mixture.

The next necessary piece is construction. The current reference builder first creates a full temporary successor table, then converts it to compact30 records. A production builder must generate the frozen records directly from the block's last column, competing with the [current single-pass counting-sort builder](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L253-L318>). In the [registered planning model](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md#6-the-construction-budget-and-the-decision-rules-fixed-before-any-run>), retaining the measured four-worker traversal advantage would permit a direct builder costing at most about 6.7% more than production's, using the conservative confidence-interval endpoint. That allowance is conditional on the gain surviving construction and the decoder's natural concurrency; it is not a prediction that the final decoder will meet its target.

So we have passed the first difficult test of the idea, not finished it. The [direct-constructor design brief](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md>) now asks whether the same representation can be built affordably, without the full temporary table. If it can, the decisive comparison will return to the original question: does compact navigation give the complete decoder more throughput for less measured memory, against the best conventional configuration?

The original gamble was that reduced navigation storage could outweigh the instructions needed to read it, once several BWT traversals had to share a memory system. For prepared-table traversal under controlled contention, it has. FOR-14 showed that a small table could still be too costly to navigate; the refined, less densely packed design has now shown a positive result. The direct constructor and the complete decoder remain the tests of whether that progress becomes a practical improvement.

---

Research state, 2 November 2026: the prepared-table traversal comparison is valid, and justified the constructor work. Direct-constructor cost, full-decoder throughput and peak memory, and the cause of the contention advantage have not yet been established. New results should be added with their own dates and measurement boundaries; this result and its source archive remain unchanged.
