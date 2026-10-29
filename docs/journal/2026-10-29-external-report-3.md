# External research report 3: expert materials for the next inverse-BWT stage (verbatim)

A research input, not a finding of this repository. It was relayed by the maintainer on
2026-10-29 from an outside assistant's session, whose reply was wiped three times while
being written; this is what the maintainer salvaged. Its claims are checked against the
primary sources, against this journal's history and against the measurements in
[2026-10-29-expert-review.md](2026-10-29-expert-review.md). Its deferred directions are
filed as issues there (§6).

The salvage flattened the report's tables into runs of cells, and its inline links reached
us as placeholder characters (`￼`). Both are kept as received in the verbatim block. The
tables are reconstructed after it, apart from the original, with what the reconstruction
assumed.

## The report, verbatim

```text
Expert research for the next stage of mibz2
The literature supports several promising directions, but it also suggests we should stop treating inverse-BWT speed as one optimisation problem.
Our workstation measurements have exposed at least four distinct problems:
Dependent memory access: traversing a BWT permutation requires unpredictable lookups.
Cache capacity: too many independent T-vectors overwhelm the i9-10900K's shared L3.
Table construction: after cooperative walking accelerates, building the T-vector becomes a larger fraction of execution.
Parallel coordination: more cores help only when their work is balanced and doesn't introduce excessive synchronization.
I searched for expert material specifically addressing those mechanisms, prioritising primary papers, real implementations and Intel's measurement guidance.
The highest-value findings
1. libsais 2.12 — the implementation to study first Highest priority
Ilya Grebnov · October 2026 · Source code and release notes
Version 2.12.0 specifically introduced cyclic BWT inversion for bzip2 and significantly improved single- and multicore inverse BWT. Its documentation explicitly identifies bigram LF mapping, decoding two characters per dependent traversal.
That makes its current source considerably more relevant than generic SA-IS papers about constructing suffix arrays. ￼
2. Slashing the Time for BWT Inversion — the foundational performance paper
Juha Kärkkäinen, Dominik Kempa, Simon J. Puglisi · DCC 2012
This paper tackles exactly the phenomenon we're seeing: BWT inversion is slow because of unpredictable cache misses. Its reported approach reduces both the number of cache misses and their cost, with experimental speedups of 2.3–4× in the authors' setting.
It is particularly relevant to bigram traversal, segmented walks and exploiting repetition. Those figures are literature results, not a forecast for mibz2. ￼
3. Asynchronous Memory Access Chaining — a scheduling idea worth revisiting
AMAC · VLDB 2016
AMAC addresses irregular pointer-chasing workloads by retaining independent traversal state and scheduling new memory requests as earlier ones complete. Its key insight is that rigid lockstep traversal can lose efficiency when individual chains have different lengths.
This is relevant because our cooperative walk still has unequal per-thread work, and libsais occasionally exhibits a severe straggler. It does not demonstrate that AMAC will beat our current 32-lane walker; it gives us a principled alternative scheduling design. ￼
4. Intel's top-down and memory-access analysis guides
Intel VTune documentation · Hardware-performance methodology
Intel distinguishes L3-bound stalls, DRAM-bound stalls, store-bound execution and core-bound execution. This matters because a faster cooperative walk could result from fewer LLC misses, greater memory-level parallelism, or less per-step computation.
These guides support measuring those mechanisms independently rather than attributing every speedup to cache residency. ￼
Additional materials worth keeping in the research backlog

Parallel algorithms for Burrows–Wheeler compression and decompression⁠￼ (2014)  Expert material What it contributes Relevance
Formal decomposition of BWT decoding into parallel stages, including list ranking for inversion High for architecture; lower for immediate implementation
Software Prefetching for Indirect Memory Accesses⁠￼ (Ainsworth & Jones)  Compiler techniques for prefetching indirect, dependent accesses High for understanding prefetch limits
Original libbzip2 low-memory decoder⁠￼ ll16 + ll4 compact pointer representation Useful footprint reference; potentially expensive traversal
Bounding the Average Move Structure Query⁠￼ (Brown & Langmead, 2026) Smaller, faster interval representations for repetitive BWT permutations Medium; revisit only if run-structure measurements justify it
Intel Performance Counter Monitor⁠ DRAM traffic and processor memory-controller measurements High for validating the cache hypothesis

The 2026 move-structure paper is worth noting specifically. Its contribution is length capping: a simpler way to control interval sizes while keeping queries efficient. It reports faster construction and smaller LF representations on repetitive genomic collections. However, its index-oriented results do not establish faster one-shot decompression of conventional .bz2 blocks. ￼
Which research directions are most promising?
Here is how I'd rank them given our actual measurements, rather than paper headline speedups.

Priority Research direction What must be demonstrated
1 libsais cyclic bigram LF² Faster native walk without an offsetting build penalty
2 Production-speed cooperative kernel Remove the 4x1 versus p4 regression while preserving 3x3 scaling
3 Parallel T-vector construction Reduce 3x3's 42% build-time share
4 Memory-level parallelism and synchronization Explain the 3x3 win and identify whether coordination can improve
5 Dynamic lane scheduling / AMAC Better balance or latency hiding than fixed lockstep lanes
6 Smaller hot working set Move the cache-capacity knee without making each lookup much slower

There is one especially important distinction in the top two entries.
libsais currently beats our cooperative research implementation by about 23% in median throughput, but only beats production p4 by about 8% at four independent workers.
That suggests we should not assume the whole 23% comes from bigram LF². Some of it may be explained by the existing research walk's regression relative to production.
This is why the literature should guide isolated experiments, not a wholesale rewrite.

A focused research programme
The next experiments and their deciding evidence
Confirm native cooperative performance
Run the frozen p4 versus 4x2 and p4 versus 3x3 confirmations. No new source changes.
Characterise the libsais advantage
Compare p4, 4x1, 3x3, s4x1, s3x3, and the independent-worker scaling arms. Preserve per-round raw measurements.
Profile the actual bottlenecks
Collect cycles, instructions, LLC events, memory-stall metrics and memory bandwidth where supported. Confirm event availability on the i9 before relying on any specific PMU counter.
Test one algorithmic change at a time
First bigram LF²; next build optimisation; only then scheduling, compact storage or output-buffer reuse. Require complete build-plus-walk timing and correctness in each experiment.

What I’d send Claude

Please perform an expert-materials review for mibz2’s next inverse-BWT optimisation phase, using the existing journal and measured i9 results to avoid repeating previously rejected ideas.

Prioritise these primary sources:

1. Ilya Grebnov, libsais v2.12.0/v2.12.1 source, changelog and cyclic inverse-BWT implementation: https://github.com/IlyaGrebnov/libsais
2. Kärkkäinen, Kempa & Puglisi, “Slashing the Time for BWT Inversion” (DCC 2012): https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf
3. “Asynchronous Memory Access Chaining” (VLDB 2016).
4. Intel’s top-down microarchitecture and memory-access analysis guides.
5. “Parallel algorithms for Burrows–Wheeler compression and decompression” (2014).
6. Brown & Langmead, “Bounding the Average Move Structure Query” (SEA 2026).

The task is not to summarise the papers generally. Extract mechanisms that could improve standard bzip2 block inversion on an i9-10900K, especially under four or fewer simultaneously live T-vectors.

For libsais, identify the exact cyclic-inversion code path used by our harness, the bigram LF² construction, lookup packing, independent traversal scheduling, build costs and OpenMP coordination. Distinguish source-confirmed facts from hypotheses.

For the other materials, compare each proposed method against mibz2’s existing research history: pair-walk, precopy, move structures, compact T-vectors, lane scheduling and software prefetching. Mark previously rejected variants and explain what new workstation evidence would justify reopening them.

Produce a ranked experiment matrix with predicted benefit, likely cost, correctness risks, required counters and a specific pass/fail criterion.

Do not modify production code or PR #73. The two pre-registered cooperative confirmations take priority over all new experiments. Record the material in the research journal with citations and links.

The key strategic insight
There is a potentially powerful combination here:
Three complementary ways to reduce inversion time
Fewer live tables
Cooperative 3x3
More bytes per lookup
libsais LF²
Cheaper construction
Parallel T-vector build
Those mechanisms attack different costs, so their benefits might be complementary. They are not automatically additive: changing the mapping can also change build cost, cache pressure and synchronization.
My strongest recommendation from this search is to make libsais 2.12.1's cyclic inverse-BWT source the main implementation reference, the 2012 Kärkkäinen–Kempa–Puglisi paper the algorithmic reference, and Intel's memory-analysis methodology the evidence standard.
That is a more focused path than exploring new compression schemes or attempting another packed-table implementation without first understanding where the winning kernel spends its time.
```

## The tables, reconstructed

The salvage kept each table's cells in order, so the reconstruction only puts them back
into rows and columns. Nothing is added to a cell. What each reconstruction assumed is
said under it.

### "Additional materials worth keeping in the research backlog"

| Expert material | What it contributes | Relevance |
|---|---|---|
| Parallel algorithms for Burrows–Wheeler compression and decompression (2014) | Formal decomposition of BWT decoding into parallel stages, including list ranking for inversion | High for architecture; lower for immediate implementation |
| Software Prefetching for Indirect Memory Accesses (Ainsworth & Jones) | Compiler techniques for prefetching indirect, dependent accesses | High for understanding prefetch limits |
| Original libbzip2 low-memory decoder | ll16 + ll4 compact pointer representation | Useful footprint reference; potentially expensive traversal |
| Bounding the Average Move Structure Query (Brown & Langmead, 2026) | Smaller, faster interval representations for repetitive BWT permutations | Medium; revisit only if run-structure measurements justify it |
| Intel Performance Counter Monitor | DRAM traffic and processor memory-controller measurements | High for validating the cache hypothesis |

Assumed:
- The header row ("Expert material | What it contributes | Relevance") reached us after the
  first row's first cell, and is moved back to the top.
- The five rows split where each title ends (the placeholder link characters mark them).

### "Which research directions are most promising?"

| Priority | Research direction | What must be demonstrated |
|--:|---|---|
| 1 | libsais cyclic bigram LF² | Faster native walk without an offsetting build penalty |
| 2 | Production-speed cooperative kernel | Remove the 4x1 versus p4 regression while preserving 3x3 scaling |
| 3 | Parallel T-vector construction | Reduce 3x3's 42% build-time share |
| 4 | Memory-level parallelism and synchronization | Explain the 3x3 win and identify whether coordination can improve |
| 5 | Dynamic lane scheduling / AMAC | Better balance or latency hiding than fixed lockstep lanes |
| 6 | Smaller hot working set | Move the cache-capacity knee without making each lookup much slower |

Assumed: each line after the header is one row, its first token the priority. The
direction's name ends where the requirement's verb begins.

### "A focused research programme: the next experiments and their deciding evidence"

| Step | Experiment | Deciding evidence |
|--:|---|---|
| 1 | Confirm native cooperative performance | Run the frozen p4 versus 4x2 and p4 versus 3x3 confirmations. No new source changes. |
| 2 | Characterise the libsais advantage | Compare p4, 4x1, 3x3, s4x1, s3x3, and the independent-worker scaling arms. Preserve per-round raw measurements. |
| 3 | Profile the actual bottlenecks | Collect cycles, instructions, LLC events, memory-stall metrics and memory bandwidth where supported. Confirm event availability on the i9 before relying on any specific PMU counter. |
| 4 | Test one algorithmic change at a time | First bigram LF²; next build optimisation; only then scheduling, compact storage or output-buffer reuse. Require complete build-plus-walk timing and correctness in each experiment. |

Assumed:
- It was a sequence of titled steps (a heading line, then its detail) rather than a
  table, and is shown as one.
- The step numbers are the order given.

### "Three complementary ways to reduce inversion time"

| Mechanism | Approach |
|---|---|
| Fewer live tables | Cooperative 3x3 |
| More bytes per lookup | libsais LF² |
| Cheaper construction | Parallel T-vector build |

Assumed: alternate lines are the mechanism and the approach that delivers it, in pairs.
