[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Investigating inverse-BWT memory interference in mibz2

Four workers make mibz2 faster, but each decoded byte more expensive. Two experiments to find out why, and why buffer reuse comes first.

1 November 2026 · 9 min read · By the independent researcher · [Research in progress](<#6-where-the-investigation-stands>)

Four workers make `mibz2` decompress data much faster than one. But they also make each byte more expensive to decode.

On an Intel i9-10900K, the four-worker decoder consumes 32% more aggregate CPU time per output byte than the one-worker decoder, despite executing almost the same number of instructions. The additional cost is concentrated in inverse BWT, where several workers reconstruct different blocks while sharing the processor's memory hierarchy.

We have measured where much of that extra CPU time appears. We have not yet established which physical interaction causes it.

Two independent investigations have proposed different experiments to resolve that uncertainty.

Astra proposes changing the decoder: reuse an existing intermediate buffer without changing the expensive traversal algorithm, then measure whether the real four-worker decoder becomes more efficient.

Claude proposes changing the environment: keep one decoder's traversal fixed while varying the memory activity on neighbouring cores, then measure which kinds of interference reproduce the slowdown.

The proposals answer different questions. One tests an available optimization; the other tries to identify a more general cause. This post explains what each could establish and why we have selected the smaller buffer-reuse experiment first.

## 1\. The four-worker memory cost

`mibz2` exploits parallelism in two places. It assigns separate bzip2 blocks to different workers, and within each block it follows 32 independent traversal chains through the inverse Burrows–Wheeler transform (BWT).

The second technique is particularly important. Inverse BWT follows a sequence of links through a table called the T-vector. Each entry identifies the next row and the byte to reconstruct. Because the next address depends on the current lookup, a single traversal chain spends much of its time waiting for memory.

The 32-lane walker hides some of that latency by advancing other chains while one waits. Our earlier article, [32 lanes, 8.6 chains](<https://lmmx.github.io/mibz2/blog/2026-10-31-room-walk-overlap/index.md>), explains the technique.

A full-sized bzip2 block contains 900,000 rows. At four bytes per entry, its T-vector occupies approximately 3.6 MB. Each worker has its own table, so concurrent decoding creates several large, irregularly accessed working sets.

The performance measurements show why that matters.

The concurrency penalty

Figure 1. The concurrency penalty, measured on four allocated physical cores of an i9-10900K. The left comparison covers the full decoder; the right uses 197 matched full-sized blocks. These are CPU-efficiency measurements, not a comparison of wall-clock throughput.

The original attribution experiment measured an increase from 2.707 to 3.579 CPU-s/GB. Retired instructions per byte rose by only about 1.3%, while cycles per byte rose by approximately 30%.

In other words, the decoder is doing nearly the same logical work, but spending considerably more processor time doing it.

A subsequent analysis of the archived block traces narrowed the increase further:

| Decoding phase | One worker, ns/row | Four workers, ns/row | Increase |
| --- | --- | --- | --- |
| Move-to-front (MTF) | 0.456 | 0.487 | 1.07× |
| T-vector construction | 0.497 | 0.571 | 1.15× |
| Traversal + assembly | 1.234 | 1.806 | 1.46× |
| RLE1 and CRC | 0.134 | 0.180 | 1.35× |

The traversal-and-assembly interval accounts for roughly 79% of the additional CPU time within these four measured phases. Across the full corpus, including shorter blocks, its increase is approximately 0.61 CPU-s/GB.

The qualification in that phase name is important: the existing timing interval includes both dependent T-vector traversal and the subsequent copying of completed segments into contiguous output. We have not yet separately measured their respective contributions.

There is also a suggestive relationship with concurrency. Across the archived traces, the median phase slowdown increases from 1.29× when no other worker is walking to 1.66× when three other workers are walking.

That is an observational association, not a controlled result. Longer walks may themselves overlap more other walks. Nevertheless, it makes the interactions between concurrently active BWT working sets an especially useful subject for investigation.

## 2\. Why the measurements don't yet identify the cause

The memory counters present several clues, but no unique explanation.

On the i9, data requests leaving L2 remain approximately stable per decoded byte between one and four workers. A mixed outstanding-request residence proxy, however, increases from approximately 39 to 58 cycles per request.

Memory-controller traffic also rises substantially, especially writes.

One interpretation is that concurrent workers displace each other's T-vector cache lines, forcing more lookups to be served from DRAM.

Another is that requests still reach the shared last-level cache, but queue longer because several cores are accessing it at once.

Intermediate-buffer stores may also contribute. Writing output can require cache-line ownership, generate writebacks, displace useful data and occupy hardware resources that loads and prefetches need.

These effects are not independent. A little additional displacement can make prefetches later; later requests can keep hardware resources occupied longer; that can make subsequent requests more expensive.

The existing counters cannot tell us which interaction starts the problem.

For example, `OFFCORE_REQUESTS.ALL_DATA_RD` includes demand and prefetch activity. Its outstanding-request ratio is useful evidence of changed memory behaviour, but it is not a direct measurement of the service latency of an individual T-vector lookup.

Nor does a rise in DRAM writes prove that those writes are delaying the traversal. They could be a consequence of the same cache pressure that makes traversal slower.

That leaves a practical choice: change something about the decoder's memory behaviour and measure the result, or recreate the competing memory behaviour under controlled conditions.

Astra and Claude independently proposed one experiment of each kind.

## 3\. Astra's proposal: reuse an intermediate buffer

Astra's experiment concerns an existing, untested research direction called P87.

The current decoder uses separate byte arrays for the BWT's last column and its reconstructed output:

```rust
ll: Vec<u8>,   // Last column, produced by MTF
rle: Vec<u8>,  // Contiguous output of inverse BWT
```

MTF writes the last column into `ll`. The T-vector builder reads `ll` and constructs the navigation table. The 32-lane walker then uses that table to reconstruct the text into a collection of segment buffers.

Finally, segment assembly copies the completed segments into `rle`, which the RLE1 decoder consumes.

Once T-vector construction has finished, the original contents of `ll` are no longer needed by the ordinary segmented inverse-BWT path.

P87 proposes using that allocation as the assembly destination.

The proposed P87 comparison

Figure 2. The proposed P87 comparison. Both arms construct and traverse the same T-vector, produce the same segments and perform the same assembly copy. Only the byte arena receiving that copy differs.

The distinction from an ordinary copy-elimination optimization is deliberate.

P87 does not save the instructions used to assemble the segments. It tests whether writing into storage that already had a useful role earlier in the block is less expensive than writing into a separate allocation.

### Why this could help

A full block has approximately 0.9 MB of last-column bytes. Reusing their allocation could reduce the set of distinct cache lines brought into writable use across the decoding phases.

That might reduce cache displacement, ownership requests or eventual writeback pressure. Any benefit could be particularly valuable when another worker is simultaneously traversing its own T-vector.

There is an interesting temporal detail here.

Assembly happens after the current block's traversal.

Changing the assembly destination cannot make that same traversal faster retrospectively. If P87 improves the measured cost of unchanged traversal code, the benefit must arise indirectly—through interference between workers, cache state carried into later blocks, or a changed pattern of overlapping work.

That would be more informative than simply observing a faster assembly operation.

### Why it might not help

Both `ll` and `rle` are already persistent allocations reused across blocks.

`ll` was recently written by MTF and read by T-vector construction. `rle` was read by RLE1 during the preceding block.

By the time assembly begins, the intervening traversal may have evicted much of either buffer from cache. Reusing `ll` therefore does not automatically save a block's worth of DRAM traffic.

The proposed experiment changes those access histories and measures whether the difference matters. It does not assume that one is better.

For the diagnostic, both allocations would remain allocated and prefaulted in both experimental arms. That prevents an apparent benefit from being explained merely by fewer allocations or page faults. A later production implementation could separately test whether the unused arena can be removed to save memory.

## 4\. Claude's proposal: change the competing workloads

Claude's independent proposal approaches the problem from the other direction.

Instead of changing the decoder's buffers, it would measure one decoding worker while controlling what happens on the other three physical cores.

The target worker performs the normal sequence of decoding operations, including T-vector construction and traversal. Counters around the relevant phases measure its execution cost.

Three neighbouring cores then run one selected type of workload at a time.

One measured walker, three controlled neighbours

Figure 3. Claude's proposed experimental topology. The target is held fixed while the three neighbouring cores run controlled workloads. The diagram shows the shared resources, not an assumption that individual resources can be varied independently.

The neighbour workloads are chosen to exercise different aspects of the memory system.

| Neighbour workload | Question being tested |
| --- | --- |
| Real decode-order work | Can the experimental setup reproduce the production slowdown? |
| Walk-only and build-only loops | Is traversal more affected by other traversals or by concurrent table construction? |
| Clean cache holders of several sizes | Does a larger competing cache footprint make traversal progressively slower? |
| Dirty writers | Is store and writeback activity especially damaging? |
| High-rate L3 accesses with a small footprint | Can shared-cache request pressure reproduce the slowdown without occupying much capacity? |
| DRAM streaming | Can memory-controller traffic reproduce the effect? |

This is potentially a powerful experiment.

Suppose traversal becomes progressively slower as clean neighbouring working sets grow, even when those neighbours issue relatively few requests. That would strengthen the case for a cache-capacity problem.

Suppose instead that a small-footprint workload issuing many L3 requests produces nearly the same slowdown, while large but lightly accessed clean arrays do little. Shared-cache request scheduling becomes a more promising explanation.

Build-only and walk-only neighbours could also test whether the observed association between concurrent walks and longer traversal times survives a controlled intervention.

Those results might eventually justify changes to table footprint, prefetch timing, phase scheduling or the number of memory requests required per decoded byte.

### The calibration problem

The difficulty is that these synthetic workloads are not independent controls for individual hardware resources.

A clean cache holder must keep touching its data to retain cache residency. Those touches generate requests. A dirty-writing workload changes ownership traffic as well as writebacks. A DRAM stream may match the decoder's average bandwidth while presenting a different mix of reads, writes and bursts.

Consequently, the experiment would measure how sensitive a walker is to a particular controlled workload, rather than directly assigning an exact fraction of production slowdown to one hardware resource.

More importantly, the target must behave like the production decoder before those sensitivities can explain the original problem.

Four independent decoding loops do not necessarily have the same mixture of scanner activity, buffer reuse and simultaneous T-vector traversal as four workers fed by the real decoder.

Claude proposed requiring the real-decoder neighbours to reproduce at least 70% of the original walk penalty before proceeding.

That is a useful early-stop condition. But failing it would mean the harness had not reproduced the relevant production conditions. It would not establish that memory interference was absent from the shipped decoder.

The neighbour experiment therefore has a substantial prerequisite: make a synthetic environment sufficiently representative that its conclusions can guide changes to the real decoder.

## 5\. Why P87 comes first

The two experiments differ in what a successful result would mean.

|  | P87 buffer reuse | Controlled neighbours |
| --- | --- | --- |
| Changes | A buffer's role and access history | Memory activity on other cores |
| Measures | Full-decoder efficiency | Cost of a fixed target traversal |
| Main benefit | Directly tests an implementable optimization | Can identify a broader class of interference |
| Main limitation | Tests only one buffer-lifetime change | Results depend on synthetic workload fidelity |
| Next step after success | Validate the optimization for production | Design and test an optimization addressing the identified resource |

We have chosen to prepare P87 first.

This is not a claim that P87 has the greater potential benefit. The opposite may be true: a better understanding of shared-cache request scheduling or T-vector residency could reveal a much larger opportunity.

P87 comes first because it asks a smaller question whose answer can immediately change an engineering decision.

A positive result would identify a useful decoder modification that preserves the current four-byte T-vector and 32-lane traversal. A negative result would prevent further effort on this particular buffer-reuse idea, without requiring a complete explanation of the memory hierarchy.

### What counts as success?

Astra proposed comparing the current layout (A) and the reused-buffer layout (B) at both one and four workers.

Let \\(C\_{X,w}\\) denote CPU-seconds per GB for layout \\(X\\) at worker count \\(w\\).

The concurrency-specific saving would be:

\\\[ D = (C\_{A,4} - C\_{B,4}) - (C\_{A,1} - C\_{B,1}) \\\]

Astra suggested a threshold of 0.10 CPU-s/GB for a material concurrency-specific improvement.

This subtracts any saving that also occurs in the one-worker decoder. It asks whether reuse provides an additional benefit specifically under concurrency.

But that statistic needs a separate engineering test.

A positive \\(D\\) does not guarantee that B is faster at four workers. It is possible for a change to make both configurations slower while reducing the *difference* between them.

The reverse distinction also matters. Reuse might improve assembly or RLE1 similarly at one and four workers. That could be a worthwhile local optimization, but it would not establish a substantial concurrency-specific memory effect.

To interpret the results, the proposed diagnostic must separately time construction, traversal, assembly and RLE1. It must also preserve the construction and traversal machine code as closely as possible, equalize allocation conditions and leave worker scheduling unchanged.

These controls ensure that we're testing buffer lifetime, rather than accidentally testing a different walker or thread-placement policy.

The earlier P87 research register also contains a separate criterion concerning performance at six to ten workers. That historical test remains distinct from this proposed four-core diagnostic.

## 6\. Where the investigation stands

Neither experiment has yet established a performance improvement.

P87 has been selected, and its diagnostic is being prepared. The neighbour-interference investigation remains available as a broader follow-up.

There are several informative outcomes.

| P87 result | What we would learn | Next action |
| --- | --- | --- |
| Better four-worker throughput and CPU efficiency, with a material concurrency effect | Buffer lifetime contributes to the scaling penalty | Validate reuse and investigate other intermediate lifetimes |
| Better assembly or RLE1 performance, but little concurrency effect | Reuse offers a local benefit without explaining much of the walker penalty | Evaluate it as a smaller optimization |
| Less write traffic, but unchanged traversal cost | Removing these particular writes does not materially relieve traversal | Do not treat this write stream as a major latency cause |
| No meaningful CPU improvement | This reuse pattern is insufficient under the tested conditions | Return to a calibrated walker-interference investigation |
| Code-generation or measurement controls fail | The intended causal contrast is unreliable | Repair the experiment before interpreting it |

Even a material P87 result of 0.10 CPU-s/GB would recover only about 2.8% of the current four-worker decoder's CPU cost. It would be useful, but it would not by itself deliver the larger 1.5–2× throughput improvement we are investigating.

That larger objective may eventually require a deeper change to T-vector servicing, prefetch timing or memory representation. The existing research has shown why those changes must be evaluated carefully: saving cache space is not useful if recovering each pointer becomes too expensive.

For now, the fast four-byte table remains the baseline.

The next experiment deliberately leaves that representation alone.

This brings us back to the difference between the two proposals.

Changing the implementation asks whether a particular memory-lifetime decision makes the decoder more efficient.

Changing the environment asks which competing activities make an otherwise unchanged traversal more expensive.

Both are legitimate ways to investigate a systems-performance problem. They produce different kinds of evidence, and the best order depends on what engineering decision needs to be made.

We have chosen the direct intervention first because it is smaller, cheaper to interpret, and immediately relevant to the shipped decoder. If it fails, we will have a better reason to invest in the broader experiment.

The present result is therefore not a faster decoder, but a more precise experimental question:

Can reusing a dead intermediate buffer reduce the cost of concurrent inverse-BWT decoding without changing the traversal that performs the reconstruction?

That remains to be measured.

---

**Research records:** [Inverse-BWT memory proposal and phase analysis](<https://github.com/lmmx/mibz2/blob/docs/bwt-memory-proposal/docs/journal/2026-11-01-bwt-memory-proposal.md>) · [Research portfolio, including P87](<https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md>) · [Dispatch-causality investigation](<https://github.com/lmmx/mibz2/blob/docs/dispatch-causality-records/docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis.md>).
