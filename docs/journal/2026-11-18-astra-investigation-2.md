# Astra investigation 2 · Cutting the BWT open

18 November 2026. A frozen Wikidata block contains 51,184 runs. Its inverse can be computed by removing most of those runs from the geometry problem altogether. The remaining open exchange has 3,448 byte-labelled strips. Its solution, together with an ordinary stable counting sort on 89,874 retained rows, decodes all 899,931 pre-RLE1 bytes.

This is a real computational saving on Wikidata. It produces another tradeoff on the complete decoder. The strongest tested version took 17% more time than eager Cells on the calling thread and 9% more with four workers in paired whole-corpus measurements. Its complete frozen decode used 6.29 MB of peak additional requested heap against 10.35 MB for eager Cells. It also executed fewer instructions and incurred fewer simulated data-cache misses on each of three measured Wikidata blocks. The production ten-core SMT crossover remains open. [Executable and reproduction](2026-11-18-open-return/README.md) · [Measurement records](2026-11-18-open-return/evidence/)

## What the two inductions reveal together

Cells takes first returns to a shrinking interval. The adjacent-rank construction takes returns to a section defined by the order along trajectories. Both build the closed dynamics of their section. Record 55 identifies the relevant gap paths and relates batched passages through the same strip to Zorich-style acceleration. The two independent reviews verify the previous all-level interval bound, the spelling certificate and the complexity qualifications. The new question is what remains computable if one *cuts some arrows out of the permutation before solving it*. [Record 55](https://github.com/lmmx/mibz2/blob/4f4d827694f7150a2d77fb40bd1a2a1c9e00f730/docs/journal/monodromy/2026-11-18-rank-gap-verification.md) · [MONODROMY review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6976532126) · [Researcher review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6976571989) · [First investigation](2026-11-18-astra-investigation-1.md)

An early test ruled out a tempting shortcut. In text coordinates the lexicographic predecessor is a piecewise translation. Consecutive run-head coordinates delimit 10,954 nonempty copy spans covering 848,747 bytes on the frozen block. The byte equalities are exact. But assembling an entire span requires every other span it reads to be ready. Even granting all output coordinates and allowing a span to settle its internal overlaps for free, this rule settles just 3,123 spans covering 5,063 bytes. Most of the byte dependencies become cyclic when grouped this way. That ruled out this whole-span schedule as the source of a saving. [Exact screen](2026-11-18-open-return/evidence/screen.txt) · [MONODROMY’s earlier identification of the dual map](https://github.com/lmmx/mibz2/blob/4f4d827694f7150a2d77fb40bd1a2a1c9e00f730/docs/journal/monodromy/2026-11-16-origami-dictionary.md)

The more productive connection is **state elimination with exposed boundaries**, the deterministic version of the stochastic complement of a Markov chain. For a BWT it admits a particularly simple factorisation. Retain a set $S$ of source rows, with the primary among them. Erasing their arrows exposes entrances $E=\mathrm{LF}(S)$ and exits $S$. Between those entrances and exits the remaining permutation consists of disjoint directed paths and isolated cycles. The paths determine a permutation $P$ of the terminal indices and a word on each path. The internal wiring of the retained arrows is available by stable sorting the subcolumn $L[S]$.

**Proved.** In increasing coordinates on $S$, its labelled first return has underlying permutation

$$
                 g=P\circ\mathrm{LF}_{L[S]}.
$$

If $j$ is the index of a retained row, its return emits its own byte followed by the open path word at entrance $\mathrm{LF}_{L[S]}(j)$. Restricting a stable sort to selected sources proves the subcolumn identity. Following the open arrows proves the factorisation. The two factors can be constructed independently. The only interface information needed by the open factor is the complementary strips and their two occupied masks. This works for an arbitrary retained set and arbitrary marked columns. [Proof and marking convention](2026-11-18-open-return/proof.md#restricting-a-stable-match)

## Which work depends on the block?

Retain all runs of width at most $t$ and the primary row. Call the $H$ remaining pieces wide strips, counting the possible split at the primary. The narrow rows may contain many changes of byte. From the open side, consecutive narrow rows form a single terminal interval. The adjacent-gap argument bounds the number of return bands by $3H+\sigma$, where $\sigma$ counts the occupied byte values. The bound depends only on the complement and the occupied alphabet.

Each band has a width $b$ and a height $h$. It represents $b$ different occurrences of a passage of length $h$. Its letters can all differ. Tracing one representative pays $h$, while the part of the answer it spells has $bh$ bytes. Thus $U=\sum h$ measures discovery by tracing, and $A=\sum bh$ measures the area spelled. Their ratio measures how many ordered occurrences actually travel together. This provides an acceleration on blocks whose LF trajectories rarely revisit the *same* strip repeatedly. The section, $U$ and the endpoint step count depend on the construction. The labelled LF permutation, its run widths, the intersections between domain and image strips and the marked output are fixed by the input. Each work count prices a particular construction. [Band bound and one-strip example](2026-11-18-open-return/proof.md#how-many-pieces-survive-the-cut)

Two primitive twelve-row columns make the difference visible. Both have primary zero, four copies of each of three bytes, and ordered run widths $2,2,1,1,1,1,3,1$. At threshold two there is just one wide strip. In column `220021021110` its LF displacement is $-3$. Three ranks cross in parallel, costing one traced letter. In `110010202221` its displacement is $+1$. One rank takes three steps along the strip. Transverse multiplicity and repeated motion along one orbit are distinct properties even with identical width profiles. [Executable counterexamples](2026-11-18-open-return/screen.py)

**Counted on the inputs.** Threshold 16 was taken forward from the frozen sweep. The open paths there contain 810,057 byte occurrences. Before cutting at byte boundaries, 1,321 nonempty bands require 50,698 traced letters. The runnable one-sided decoder makes five additional nonempty bands and traces eleven more letters so each return includes its final narrow literal. Its complete correction has 3,204 bands including empty wide passages. [Frozen sweep](2026-11-18-open-return/evidence/frozen.csv)

| Fraction of original rows, per-block medians at width 16 | Frozen Wikidata | Wikidata, 233 large blocks | Prose, six blocks |
|---|---:|---:|---:|
| Narrow rows retained | 9.99% | 11.33% | 67.66% |
| Wide area $A/n$ | 90.01% | 88.67% | 32.34% |
| Wide representative letters $U/n$, including byte cuts | 5.63% | 5.49% | 2.10% |
| Mean multiplicity $A/U$ | 15.97 | 15.92 | 15.52 |
| Rows at colour changes, including the primary if needed | 3.40% | 4.32% | 29.10% |

The prose result is instructive. The wide passages there also have about sixteen occurrences per traced letter. They cover only a third of the block. Even within Wikidata, blocks 118 and 176 both have 900,000 rows and run densities of 0.06348 and 0.06342. Their tracing costs are 57,626 and 34,775 letters, and their open endpoint costs are 12,756 and 9,659 steps. These are zero-based ordinals in the pinned compressed file. The pair was chosen for its tracing-cost spread among large blocks within 0.00015 run density. [All block counts](2026-11-18-open-return/evidence/blocks.csv) · [Ranges at other widths](2026-11-18-open-return/evidence/widths.csv)

## Two ways to compute the same open map

The tracer starts at the first uncovered entrance and follows LF through the wide strips. The smallest remaining rightward distance to a strip end gives the width certified for its whole path. The tracer advances by that width. It needs only the wide strips and their intervening terminal intervals.

The endpoint constructor puts the entrances and exits at the front of an auxiliary interval exchange. Their connector arrows have empty words. It eliminates the original rows until only that prefix remains. At most $3H+2$ live records suffice, no matter how the narrow rows are wired. On the frozen block it starts with 7,786 records, makes 11,215 steps and creates 7,334 nonempty binary rules. It then materialises the correction’s word dictionary by copying previously expanded rules. The two constructors gave identical band and spelling-length counts at width 16 on all 431 Wikidata blocks and six prose blocks. Separate exhaustive checks compare every labelled return arrow, including arrows off the primary cycle. [Endpoint construction](2026-11-18-open-return/proof.md#computing-the-open-map-by-endpoint-induction) · [Arrow audits](2026-11-18-open-return/checks.rs)

The queued implementation expresses the stable sort directly. As the capped MTF run records arrive at the kernel, each narrow occurrence appends its retained index to the queue of its byte. Consecutive entries of those queues are then wired to consecutive exits of a correction band. A packed table holds the inverse return and its word code. Output follows the marked cycle, copying literal dictionary strings forwards. No suffix array, precomputed support, text-coordinate oracle or original-row LF table enters this route. The production scanner, run frontend, RLE1, block CRC and combined stream CRC all participate in the wall measurements. [Decoder source](2026-11-18-open-return/open_return.rs) · [Endpoint source](2026-11-18-open-return/endpoint.rs)

There is a more severe cut. Retain only a change between narrow and wide arrows, plus the mark. If $m=|S|$, the number of alternations in either direction is exactly $m-|S\cap E|$, measurable by intersecting two interval tilings before any word is known. The frozen block needs 15,278 narrow–wide pairs. The two-sided executable traverses their 30,556 individual arrows and traces 58,581 additional letters to construct the narrow words. In the repeated inverse-only measurement its emitter took 0.511 ms, about half the queued endpoint emitter’s 1.018 ms. Its constructor took 6.771 ms against 1.857 ms. Narrow-side discovery consumes the saving. The smaller closed cycle is real and this way of obtaining it loses. [Switch construction](2026-11-18-open-return/switches.rs) · [Phase measurements](2026-11-18-open-return/evidence/phases-frozen.csv)

The marked-output enumeration compares each of the three return constructors with Plain on 198,790 column, primary and width combinations. A separate suite checks the actual destination and word of every retained arrow for four constructor and queue combinations on 26,135 such triples. Complete-stream checks cover the frozen block, all 431 Wikidata blocks, all six prose blocks, and independently damaged block and stream CRCs. [Checks and output hashes](2026-11-18-open-return/evidence/checks.txt)

## What the machine gains

The comparison uses mibz2 0.5.5 at `dbe9b406`, with both the shipped eager and widened lazy kernels. The host exposes five Xeon physical cores, one hardware thread each, with a four-CPU quota. The whole-corpus runs rotate the three arms, compare every output, discard warm-up and include output allocation. Workers use the ordinary worker pool. The first row uses the calling-thread decoder. Wall times are medians in seconds for 209,700,196 output bytes. The last two columns are medians of ratios *within the same round*, which can differ from ratios of the displayed medians. [Timing observations](2026-11-18-open-return/evidence/timings.csv)

| Wikidata | Eager Cells | LazyCells | Queued open endpoint, width 16 | Open / eager, paired | Open / lazy, paired |
|---|---:|---:|---:|---:|---:|
| Calling thread, 11 measured rounds | 0.751 | 0.972 | 0.896 | 1.171 | 0.918 |
| Two workers, 9 rounds | 0.459 | 0.583 | 0.543 | 1.211 | 0.932 |
| Four workers, 11 rounds | 0.336 | 0.399 | 0.383 | 1.091 | 0.928 |

The queued endpoint beat lazy Cells by roughly 7–8% in these paired runs. Eager Cells retained the whole-corpus lead. A separate frozen width sweep sometimes favoured the open kernel, and the primary corpus result gives the more reliable workload verdict. On prose the paired calling-thread ratio against eager Cells was 1.455. A segmented interpreter which stages only residual word codes also lost its frozen trial. [Raw comparisons](2026-11-18-open-return/evidence/timings.csv) · [Per-block inverse measurements](2026-11-18-open-return/evidence/phases-wikidata.csv)

Peak additional requested heap for a fresh complete frozen decode was 6,287,363 bytes for the queued endpoint, 5,441,783 for the one-sided tracer without queues, 10,353,399 for eager Cells and 15,392,124 for LazyCells. The queued endpoint used 39% less than eager Cells. These figures include the ordinary decoder buffers and returned output. The 359,496-byte residual table is just one component. [Allocation ledger](2026-11-18-open-return/evidence/heap.csv)

| Complete frozen decode, Cachegrind | Eager Cells | Queued open endpoint |
|---|---:|---:|
| Instructions | 18,876,452 | 16,808,478 |
| Data reads | 3,459,231 | 3,954,946 |
| Data writes | 2,903,354 | 2,332,599 |
| Modelled L1 data misses | 336,697 | 246,007 |
| Modelled last-level data misses | 65,029 | 60,524 |

The two nearly equal-density blocks above also had fewer instructions and fewer modelled L1 and last-level data misses with the queued endpoint. Reads and writes move in different directions. Cachegrind models a single process and counts misses, while DRAM traffic and SMT throughput require the target hardware. On the measured i9, 0.5.5’s wider lazy admission already brought traffic from 10.70 to 8.05 bytes per output byte at seven workers. The measurements here establish no new i9 result. [Simulated counters](2026-11-18-open-return/evidence/cache.csv) · [Measured i9 baseline](https://github.com/lmmx/mibz2/blob/2fe9557/CHANGELOG.md)

The stable order of the retained occurrences and the labelled transport through their complement can be computed independently. It establishes which part of the real input can be solved with little active geometry, and why a small return section can still be expensive. On these Wikidata blocks this yields a working decoder with materially less state and fewer instructions on the measured blocks. The remaining cost shows up in building and walking the retained permutation. A throughput advance over eager Cells on the principal corpus is still unestablished.
