<!--
Provenance: the research chatbot's final conclusions on the succinct-structures review,
29 October 2026, written in answer to the user's request "State your conclusions, I'll have
them recorded", and relayed by the user into the working session to be stored verbatim.
The text below is unedited, including its own formatting, escapes and citation markers
("ibwt_sub.PDF" is its copy of Seward 2001). It is that chatbot's view, not a
measurement; the journal entry 2026-10-29-succinct-review.md (§9 and §11) says which of its
open questions the census has since answered.
-->

# Research conclusions: Succinct Data Structures for High-Performance Inverse BWT

mibz2 research record — 29 October 2026

## Executive conclusion

The most promising application of succinct data structures to mibz2 is not conventional compressed indexing, but the development of compact, directly navigable representations of the inverse-BWT permutation that preserve memory-level parallelism.

Our investigation progressed from classical rank/select structures to a more specialized problem: representing the monotone subsequences of the BWT permutation using substantially less memory, while retaining inexpensive random access to individual successor positions.

We identified two concrete representation candidates:

1. 24-bit link-only T-vector: a relatively conservative design that eliminates embedded output symbols and recovers them from the sorted first column, F.
2. 16-bit-per-row cache-line-blocked representation: an exploratory hybrid of frame-of-reference and Elias–Fano coding, storing 32 permutation values in a directly addressable 64-byte record.

The first has strong historical precedent. The second is more ambitious and remains a theoretical design requiring validation.

Neither has demonstrated a performance advantage over mibz2's existing implementation.

The central research contribution is a more precise formulation of the optimization problem:

\\[ \boxed{ \text{How much representational redundancy is worth retaining to maximize parallel permutation-navigation throughput?} } \\]

## 1. Established mathematical conclusions

### 1.1 The inverse-BWT permutation contains substantial exploitable structure

The standard LF mapping is:

\\[ LF(i)=C[L[i]]+\operatorname{rank}\_{L[i]}(L,i) \\]

Its inverse is:

\\[ \Psi(j)=\operatorname{select}\_{F[j]}(L,j-C[F[j]]+1) \\]

Consequently, the LF permutation is increasing among positions containing the same symbol, while Ψ consists of monotone occurrence lists associated with contiguous character intervals in F.

This is the mathematical foundation of compressed suffix arrays, succinct permutation representations and our proposed select-oriented structures.

### 1.2 The permutation and BWT encode equivalent combinatorial information

Given symbol frequencies \\(n_c\\), stable LF permutations correspond bijectively to arrangements of the BWT symbols.

The number of possible arrangements is:

\\[ \frac{n!}{\prod_c n_c!} \\]

Their information content is:

\\[ \log_2\frac{n!}{\prod_c n_c!} \approx nH_0(L) \\]

up to lower-order terms for fixed alphabets.

For a byte alphabet, this is at most approximately eight bits per row.

By comparison, mibz2's current directly navigable T-vector stores 32 bits per row.

The discrepancy is computational redundancy deliberately retained to make navigation fast.

This count describes the broader family of stable mappings; the universe of valid single-cycle BWTs under particular conventions may be more restricted.

### 1.3 Symbol-class monotonicity and run translation are distinct properties

Two structural properties must not be conflated:

- Symbol-class monotonicity: LF preserves order among all occurrences of one symbol.
- Contiguous-run translation: Within a run of identical BWT symbols, successive LF destinations differ by one.

The former enables rank/select representations and monotone occurrence lists.

The latter enables run-length-compressed mappings and move structures.

mibz2's negative move-structure experiments provide practical evidence against one implementation of the second approach, but do not eliminate the first.

## 2. Conclusions from Julian Seward's 2001 paper

Our reading of Space-Time Tradeoffs in the Inverse B-W Transform established three particularly relevant findings.

### 2.1 `indexF` is the historical ancestor of link-only storage

Seward demonstrated that the explicit transformation vector can overwrite the original BWT symbols because output characters can instead be recovered from F.

His `indexF` algorithm stores 20-bit links and performs a binary search over cumulative frequencies to reconstruct output bytes.

The key insight is:

Navigation information and output symbols need not be stored together.

For mibz2, the modern question is whether inexpensive F lookup can eliminate Seward's historical symbol-recovery penalty.&#x20;

ibwt_sub.PDF



### 2.2 Memory locality can outweigh additional instructions

Seward's `mergedTL` executes approximately 11% more instructions than `bw94` in his reported comparison, yet suffers substantially fewer cache misses and achieves approximately 61% higher average throughput.

This is a historical demonstration that optimizing instruction count and optimizing execution time are different objectives.&#x20;

ibwt_sub.PDF

ibwt_sub.PDF



It supports investigating compact representations under shared-cache pressure, but does not predict that a particular packed representation will win.

### 2.3 `treeopt` anticipates succinct-indexing techniques

Seward also explores eliminating the explicit transformation vector and reconstructing navigation values through auxiliary character-frequency trees.

His `treeopt` implementation packs tree nodes and uses storage more economically, substantially improving upon his unoptimized tree representation.

The approach is closely related in spirit to rank/select-based navigation, although it is much slower than direct-table inversion in his measurements.&#x20;

ibwt_sub.PDF

ibwt_sub.PDF



Historical conclusion: Seward had already explored the essential space–time spectrum, but modern memory-level parallelism introduces a performance dimension his experiments did not evaluate.

## 3. Candidate A: 24-bit link-only representation

At the maximum bzip2 block size of 900,000 rows:

| Representation                    | Main-table footprint |
| --------------------------------- | -------------------- |
| Current 32-bit combined entries   | 3.60 MB              |
| Proposed 24-bit link-only entries | 2.70 MB              |
| Reduction                         | 25%                  |

A 20-bit row index and segment marker fit comfortably within 24 bits.

Output bytes would be recovered from F-derived metadata.

The strongest arguments for this architecture are:

- Smaller cache footprint under concurrent decoding.
- Potential elimination of redundant construction work.
- Preservation of direct indexed navigation.
- Possibility of overlapping F-symbol recovery with the principal link load.

The risks are additional instructions, unaligned access, split-cache-line reads, parallel-store complexity and lookup interference with the 32-lane schedule.

### The F-bucket observation

Because F is sorted over at most 256 symbols, it has at most 255 symbol transitions.

With 32-row buckets in a full 900,000-row block:

\\[ \text{Rows in mixed buckets}\le255\times32=8160 \\]

Therefore at least 99.09% of rows belong to uniform-symbol buckets.

This establishes a potentially cheap common case for F-byte recovery.

However, #79 already contains an F-window lookup. The uniform-bucket observation establishes an opportunity for analysis, not proof that a new lookup is superior.

## 4. Candidate B: 16-bit cache-line-blocked Ψ

This is the most ambitious representation we developed.

The objective is to encode 32 Ψ values in one directly addressable 64-byte cache-line record.

That achieves:

\\[ \frac{512\text{ bits}}{32}=16\text{ bits per row} \\]

before exceptions and auxiliary storage.

At maximum block size, the base footprint would be approximately 1.80 MB, half the existing T-vector.

### 4.1 Frame-of-reference format

For 32 increasing positions:

\\[ p_0\<p_1<\cdots\<p\_{31} \\]

Store a base position and 32 offsets of 14 bits.

A proposed bit allocation is:

| Field               | Bits |
| ------------------- | ---- |
| Base position       | 20   |
| Segment flags       | 32   |
| Symbol              | 8    |
| Format selector     | 4    |
| 32 × 14-bit offsets | 448  |
| Total               | 512  |

It works when:

\\[ p\_{31}-p_0\le16383 \\]

Retrieval requires a packed-field extraction and addition.

This is simpler than ordinary Elias–Fano random access.

### 4.2 Elias–Fano format

For wider monotone spans, another 512-bit layout is possible:

| Field                  | Bits |
| ---------------------- | ---- |
| Base position          | 20   |
| Segment flags          | 32   |
| Symbol                 | 8    |
| Format selector        | 4    |
| Unary high-part vector | 64   |
| 32 × 12-bit low parts  | 384  |
| Total                  | 512  |

The position is reconstructed using:

\\[ p_i=base+ \left((select_1(H,i)-i)\ll12\right) +\ell_i \\]

With a 64-bit high-part vector, select could potentially be implemented using in-register BMI2 operations such as `PDEP` and trailing-zero count.

This avoids a separate general-purpose select directory, although it still adds operations to the navigation dependency chain.

### 4.3 Exceptions and limitations

The proposed 16-bit footprint is conditional.

Important unresolved issues include:

- Records crossing F-symbol boundaries.
- Monotone spans exceeding the selected encoding's limits.
- Exception storage and lookup penalties.
- Record-construction cost.
- Segment-flag representation.
- Packed-field extraction cost.
- CPU-specific BMI2 performance.
- Dependency depth after loading a record.

One particularly useful mathematical result is that, for 32-row records, no more than 255 records can contain an F-symbol boundary.

This limits the frequency of boundary exceptions, though not span exceptions.

Research status: coherent candidate representation, not a validated practical data structure or demonstrated optimization.

## 5. The correct performance model

Traditional succinct structures are commonly evaluated by space and asymptotic query time.

For mibz2, that is insufficient.

Our proposed evaluation dimensions are:

\\[ \boxed{ \begin{aligned} &\text{Bits per row}\\\ &\text{Construction cost}\\\ &\text{Instructions per navigation}\\\ &\text{Dependent-load depth}\\\ &\text{Memory traffic}\\\ &\text{Cache-resident metadata}\\\ &\text{Peak throughput at optimal concurrency} \end{aligned} } \\]

In particular, \\(O(1)\\) navigation does not imply one load or low latency.

The difference between one unpredictable load, two independent loads and two dependent loads is fundamental.

A compact structure may be inferior to a larger one if its decoding instructions reduce the processor's ability to sustain outstanding memory requests.

Conversely, a compact structure may win despite extra instructions if its reduced footprint improves cache residency across concurrent blocks.

## 6. Implications of mibz2's existing measurements

The performance results communicated by the engineering agents provide several useful constraints.

- The 32-lane cooperative walk has already recovered substantial latency-hiding opportunities.
- Reducing pointer-chase counts has not consistently helped: ψ², Precopy and move-structure experiments lost in the tested configurations.
- Removing redundant instructions and construction work has been more promising.
- Cache residency is a measured constraint, with an observed knee around 14.4 MB of concurrently live T-vectors on the tested workstation.
- A more compact representation could shift that knee, but its speedup would not automatically compound with existing cooperative scheduling gains.

These are reported experimental findings, not independent measurements performed in this research conversation.

They suggest that candidate representations should be evaluated for whole-decoder throughput, not solely per-query latency or compression ratio.

## 7. Research literature identified

The following works provide the principal theoretical foundation.

| Work                                                                                                       | Relevance                                                            |
| ---------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Seward (2001), Space-Time Tradeoffs in the Inverse B-W Transform                                           | Direct and compact inverse-BWT representations                       |
| Kärkkäinen & Puglisi (2011), Cache Friendly Burrows-Wheeler Inversion                                      | Re-examines inversion performance under different memory hierarchies |
| Sadakane's compressed suffix-array work                                                                    | Monotone Ψ representations and compressed navigation                 |
| Barbay & Navarro (2009/2013), Compressed Representations of Permutations                                   | Permutations formed from shuffled increasing subsequences            |
| Barbay, Gagie, Navarro & Nekrich (2010), Alphabet Partitioning for Compressed Rank/Select and Applications | Compressed sequence representations and efficient select             |
| Nishimoto & Tabei (2021), Optimal-Time Queries on BWT-Runs Compressed Indexes                              | Run-compressed LF navigation using move structures                   |
| Classical Elias–Fano literature                                                                            | Space-efficient monotone sequences with indexed retrieval            |

The mathematical representation problem has substantial prior art.

The potential research contribution lies in evaluating or specializing these techniques for highly parallel, one-shot inverse-BWT traversal.

## 8. Unresolved research questions

These are the questions I would preserve for future investigation.

1\. Can a 24-bit direct-link representation improve total decode throughput through reduced construction work and shared-cache residency, despite extra symbol-recovery instructions?

2\. What fraction of real Ψ occurrence-list blocks fit FOR-14 and bounded EF-12 formats? How does that fraction vary with alphabet size, compression characteristics and block size?

3\. Can a 64-byte fixed record provide sufficiently cheap random Ψ access to compete with an ordinary `u32` load?

4\. Can we design compact monotone-permutation records whose decoding is primarily register arithmetic, rather than further dependent memory loads?

5\. Is a hybrid of frame-of-reference, Elias–Fano and exceptional absolute links preferable to a single encoding scheme?

6\. Can a direct-indexed select structure exploit the contiguous F-symbol intervals more effectively than a general-purpose succinct sequence representation?

7\. Does a different physical organization of navigation metadata enable greater memory-level parallelism even if it uses more bits than an entropy-optimal structure?

8\. Where does the space–throughput Pareto frontier lie as concurrent table residency, lane count and CPU microarchitecture change?

## Final research position

Our findings support a clear hierarchy of confidence.

| Conclusion                                                           | Confidence                                                         |
| -------------------------------------------------------------------- | ------------------------------------------------------------------ |
| LF/Ψ monotonicity enables compact representations                    | Mathematically established                                         |
| Seward's `indexF` establishes link-only historical precedent         | Directly established by the paper                                  |
| F's limited symbol boundaries permit efficient coarse lookup schemes | Mathematically established                                         |
| Compact tables can improve cache-residency opportunities             | Supported by working-set analysis; performance remains conditional |
| 24-bit link-only tables can outperform existing mibz2                | Unproven                                                           |
| The proposed 16-bit hybrid records are practical on real BWTs        | Unproven                                                           |
| Those records can outperform direct links under 32-lane MLP          | Unproven                                                           |

The central discovery is not a new information-theoretic lower bound, nor an obviously superior compression algorithm.

It is the possibility of a different point on the space–time frontier: a representation that retains enough redundant information for direct, parallel navigation while exploiting the monotone structure of Ψ to occupy substantially fewer cache lines.

Classical succinct structures minimize representation size subject to query guarantees. Seward explored memory consumption against inverse-BWT execution time. mibz2 introduces a compelling third dimension: the ability to overlap many independent, latency-sensitive navigation operations.

The research direction can therefore be summarized in one question:

> Can the monotone structure of the inverse-BWT permutation be compressed into cache-line-addressable records that preserve high memory-level parallelism, producing greater whole-decoder throughput than a flat array of 32-bit links?

That is the principal hypothesis worth carrying forward.

It remains speculative—but it is mathematically grounded, linked to established literature, and precise enough to investigate empirically without confusing theoretical compactness with actual performance.
