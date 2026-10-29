<!--
Provenance: the guardian chatbot's final conclusions on the succinct-structures review,
29 October 2026, written in answer to the user's request "State your conclusions, I'll have
them recorded", and relayed by the user into the working session to be stored verbatim.
The text below is unedited, including its own formatting and escapes. It is that
chatbot's view, not a measurement; the journal entry 2026-10-29-succinct-review.md (§9 and
§11) says which of its recommendations the census has since answered.
-->

Here is my consolidated research assessment, written as a standalone record for Claude and the mibz2 journals.

# Succinct-Structures Research Review — Final Conclusions

**Date:** 29 October 2026\
**Project:** mibz2 — high-performance bzip2 decoding\
**Status:** Research checkpoint complete; hypotheses preserved; no production change authorised.

## 1. Executive conclusion

The research has identified a credible opportunity to improve inverse-BWT throughput by exploiting the mathematical structure of its navigation permutation, Ψ, while preserving the memory-level parallelism of mibz2's cooperative decoder.

**The central question is not how tightly we can compress Ψ, but how compactly we can represent it while keeping the next navigation address inexpensive to compute.**

The strongest practical candidate remains a 24-bit link-only table with efficient output-byte recovery. A more ambitious candidate has emerged: a 16-bit-per-row, cache-line-oriented hybrid using frame-of-reference (FOR) and Elias–Fano (EF) encoding.

Neither candidate has demonstrated a performance improvement. Both require further evidence.

## 2. Established mathematical findings

mibz2's T-vector stores Ψ, the inverse of the standard LF mapping.

For a row j in first-column symbol bucket c:

[\
\Psi(j)=\operatorname{select}\_c(L,j-C[c]+1).\
]

Thus Ψ is equivalent to 256 sorted occurrence-position lists concatenated in first-column order.

**Ψ is monotonically increasing within each contiguous F-symbol interval.** This makes compressed sorted-integer sequences, posting-list structures and compressed-permutation literature directly relevant.

Given fixed symbol frequencies (n_c), the broader family of stable LF permutations has

[\
\frac{n!}{\prod_c n_c!}\
]

possible members, requiring

[\
\log_2\frac{n!}{\prod_c n_c!}\approx nH_0(L)\
]

bits to distinguish.

This is an information-theoretic characterization, not a bound on the space needed to support fast random navigation. Valid cyclic BWTs and periodic inputs introduce additional constraints.

The existing 32-bit-per-row representation deliberately stores redundant navigation information to avoid reconstructing links during traversal.

## 3. Findings established by mibz2 measurements

The i9-10900K has a shared 20 MB L3 cache. Cache residency materially affects independent-block inversion throughput.

The existing T-vector occupies four bytes per row, or approximately 3.6 MB for a maximum-size 900 KB block. The last column occupies an additional byte per row but is not randomly accessed during the walk.

The cooperative 3×3 configuration processes three live tables using nine cores, reducing cache pressure while overlapping pointer-chasing work.

The current cooperative walk uses multiple independent traversal lanes. Its performance depends on instruction throughput, memory-level parallelism, prefetch timing and shared-cache residency.

These measurements establish that reducing table footprint could matter near and beyond the cache-residency knee.

They do **not** establish that a smaller table will improve peak throughput. Extra decoding instructions, register pressure or delayed prefetch addresses could erase the benefit.

Several alternative navigation approaches have already performed poorly in interleaved traversal, including the move-structure walk, pair-walk and Precopy approaches. Their results caution against adding dependent lookup operations.

## 4. Historical prior art

The conceptual foundations are established rather than novel.

**Julian Seward, 2001 — Space-Time Tradeoffs in the Inverse B-W Transform**

Seward examined explicit navigation tables, merged symbol/link representations, link-only representations and structures that reconstruct navigation information on demand.

His `indexF` design stores links without preserving output bytes and recovers symbols from the first-column distribution.

libbzip2's small-memory mode implements this lineage, but its slowdown reflects several costs together: compact split storage, symbol search and pointer reversal. It is not a direct estimate of the cost of mibz2's proposed layout.

**Kärkkäinen and Puglisi, 2011 — Cache Friendly Burrows-Wheeler Inversion**

The work revisited earlier algorithms and explicitly considered parallel inversion through threads or out-of-order execution. It reinforces the importance of memory locality and the fact that space–time frontiers depend on hardware.

**libsais 2.12.1**

Source inspection by Claude established that its fast cyclic inverse-BWT implementation uses link-only two-step navigation and lookup-based symbol recovery. This is especially relevant because libsais is mibz2's strongest measured inversion reference.

**Compressed-permutation and posting-list literature**

Grossi–Vitter, Sadakane, Barbay–Navarro, Nishimoto–Tabei, Elias–Fano and partitioned posting-list research provide established techniques for exploiting increasing sequences and BWT runs.

The potential contribution from mibz2 is not the invention of link-only inversion or succinct permutations. It is a hardware-aware implementation and empirical evaluation under cooperative, memory-level-parallel traversal.

## 5. The primary practical hypothesis: 24-bit T-vector

A conventional four-byte table stores a next-row link, output byte and segment flag.

A link-only representation can potentially store a 20-bit row index and segment marker in three bytes, reducing the principal randomly accessed table by approximately 25%.

The output byte can be recovered from the sorted first column using compact lookup metadata.

Advantages to test:

- Smaller working set under shared-L3 contention.
- Potentially greater independent-block concurrency.
- Retention of direct row addressing.
- Possibility of overlapping symbol recovery with navigation loads.

Costs to measure:

- Additional symbol-recovery instructions and loads.
- Unaligned and cache-line-crossing accesses.
- Narrow or split stores during construction.
- Parallel write ownership and data-race safety.
- Effects on prefetch scheduling and register pressure.

The existing #79 pilot already implemented a four-byte link-only control and an F-window lookup. Its results do not isolate byte-recovery cost because the traversal loop and start-marker handling also differ.

The first necessary control is therefore **the same traversal kernel with and without embedded output bytes**, not another independently designed lookup algorithm.

A smaller table must be compared with the existing design at each configuration's own optimum concurrency, not merely at identical worker counts.

## 6. The more ambitious hypothesis: 16-bit cache-line-oriented Ψ

Because Ψ consists of sorted occurrence lists, groups of 32 consecutive entries within an F-symbol interval may admit compact encoding.

The proposed fixed-record architecture allocates one aligned 64-byte record per 32 entries, giving a nominal footprint of 16 bits per row.

Two ordinary record types are proposed:

**Frame of reference, FOR-14**

Store a base position and 32 offsets of 14 bits each.

Applicable when the positions' span is no greater than 16,383.

Random access requires packed-field extraction and addition, avoiding rank/select operations.

**Bounded Elias–Fano, EF-12**

Store 12-bit low parts and a 64-bit unary high-part vector.

Recover high parts using an in-register select operation, potentially involving BMI2 PDEP and trailing-zero count.

This supports substantially wider spans than FOR, at greater decoding cost.

**Exceptions**

Groups crossing F-symbol boundaries or exceeding the encoding limits require another representation and possibly an additional dependent memory access.

The nominal footprint of a full-size table would fall from 3.6 MB to 1.8 MB before exception storage.

However, the current hybrid layout is **a design sketch, not a validated encoding**. Its metadata budget, exception representation, segment markers, select arithmetic, safe loads and construction method require exact specification and proof.

A 64-byte record does not imply a single load instruction or zero additional dependency latency.

Similarly, writing complete cache lines does not automatically eliminate read-for-ownership traffic with ordinary cached stores.

This proposal should remain an unscheduled research candidate until its correctness and actual corpus fit are established.

## 7. The correct evaluation framework

Traditional succinct structures are often compared by space and asymptotic query time.

For mibz2, the useful evaluation dimensions are:

1. Effective bits per row, including all metadata and exceptions.
2. Construction time and memory traffic.
3. Instructions and execution resources per navigation step.
4. Dependent memory-load depth.
5. Delay from obtaining the current row to issuing the next prefetch or load.
6. Cache-line footprint and metadata residency.
7. Peak total throughput at optimal concurrency.
8. Correctness for every supported BWT configuration.

The critical metric is **the processor-level dependency graph**, not merely the number of operations.

Two independent loads, two serial dependent loads and one load followed by substantial decoding are fundamentally different designs.

A theoretically constant-time succinct operation may still be expensive in CPU cycles.

The overall performance target is the complete relevant pipeline, with construction and traversal evaluated separately for causal understanding.

## 8. The next research measurement

Before implementing the hybrid representation, perform an untimed corpus analysis.

For each block, reconstruct or inspect Ψ and partition it into global groups of 32 rows.

Record:

- Fraction of groups qualifying for FOR-14.
- Fraction qualifying for EF-12 but not FOR-14.
- Fraction requiring exceptions.
- Frequency and character of F-symbol boundary crossings.
- Span distributions by block size and effective alphabet.
- Effective storage required for exceptions and metadata.

This is a feasibility test, not a benchmark demonstrating speed.

If ordinary records rarely qualify, or exceptions are excessive, the hybrid should be deprioritised without implementation.

If the distribution is favourable, the next step is an exact bit-layout and dependency-path audit, followed by a targeted microbenchmark only if the design still appears competitive.

## 9. Engineering priorities remain unchanged

**Immediate priority: #81 cooperative-build experiment.**

The existing single-core #81 experiment measured approximately 1.027× faster construction on CI, below its registered success threshold and without an end-to-end decoder gain.

A distinct cooperative experiment remains justified because the copy phase and synchronisation costs differ from the single-core implementation. Its outcome must be recorded against its own pre-registered criteria.

**Next: #79 compact-table audit.**

Inspect the existing F-window lookup, measure the F-bucket boundary distribution, and isolate byte recovery using a matched traversal kernel.

Only then consider a 24-bit implementation.

**Later: 16-bit FOR/EF hybrid.**

Preserve it as a separate theoretical and feasibility candidate. Do not allow it to displace the simpler controls.

The MTF register-residency idea remains a separate deferred lead.

No production integration follows from these research hypotheses alone.

## 10. Final assessment

The research has clarified three different questions:

**Information theory:** How much information is required to represent the BWT navigation permutation?

**Data-structure engineering:** How much additional space is needed to recover any navigation link efficiently?

**Microarchitecture:** How should that additional information be arranged so that CPUs can sustain many independent traversals with minimal waiting?

The third question is where mibz2's empirical work may contribute something distinctive.

The most promising design principle is:

**Compress navigation information in units that preserve predictable addressing and memory-level parallelism, even when that requires spending more bits than the information-theoretic minimum.**

The 24-bit table is a practical test of this principle. The 16-bit cache-line hybrid is a more ambitious test.

Both may fail. A negative result, correctly measured, would still improve our understanding of the space–time–parallelism trade-off.

**Research verdict:** The succinct-structures investigation has succeeded in producing a grounded, historically informed research programme. It has not yet produced a demonstrated optimisation. Preserve the hypotheses, complete the existing controlled experiments, and let measured performance decide which representation—if any—belongs in mibz2.

One point I would particularly want preserved is that the 16-bit hybrid is not yet a verified working layout. Its appeal is strong enough to justify analysing the distribution of Ψ values, but not strong enough to justify skipping the simpler #79 controls.

That distinction keeps the conclusions scientifically useful rather than turning an elegant theoretical idea into an assumed performance result.
