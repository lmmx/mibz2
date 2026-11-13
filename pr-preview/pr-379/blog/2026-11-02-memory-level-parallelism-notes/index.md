← [Memory-level parallelism in mibz2](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-02-memory-level-parallelism/index.md>)

# Technical appendix: MLP, ILP and the measurements

This appendix is for readers who want the qualifications and definitions that [the main post](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-02-memory-level-parallelism/index.md>) leaves out to keep its argument moving. Its sections can be read in any order. Nothing new was measured for either page: every result is attributed to the experiment that produced it, and repository sources are pinned to the revisions in the [source register](<#11-source-register-and-revision-pins>), inspected on 2 November 2026. Numbers in square brackets refer to the register.

## 1\. The disagreement about ILP and MLP

The main post reproduces the replies to Lemire's post \[1\] and explains the mechanism they argue about. Taking the claims one at a time:

| Claim | Assessment |
| --- | --- |
| Interleaving increases available ILP. | Yes. Independent loads are independent instructions. |
| It "breaks" the hard dependencies. | Imprecise: every dependency within each chain remains. Independent chains are supplied alongside them. |
| IPC and instruction count should be measured. | Sensible. They characterise the change but don't by themselves prove its cause. |
| Allocation and cache residency matter. | Yes. A random workload hitting L2 and one going to DRAM can give very different curves. |
| The dependencies are "memory loads, not instructions". | Loose: loads are instructions. The useful distinction is whether the critical delay is a memory access or ordinary computation. |
| Superscalar execution is a prerequisite for several outstanding loads. | Too strong as a general statement (section 2). |
| An in-order processor can't benefit this way. | Also too strong. It depends on the design and on the order of the instructions. |

### Reviewing, reproducing and explaining

Three questions are easily run together. Review asks whether the algorithm, source, method and interpretation can be inspected; here they can, as Lemire said. Historical reproduction asks whether the exact 2022 executable, machine state, page mappings, compiler output and run conditions can be recreated, which needs more provenance than source code. Causal attribution asks whether the speedup establishes the mechanism claimed and rules out the alternatives, which needs suitable measurements or interventions. Open source helps with all three and settles only the first. A fair summary: the benchmark is a legitimate way to characterise overlapping dependent traversals, and the stronger readings of its output need more evidence.

## 2\. What a core needs to keep several loads in flight

| Property | What it describes |
| --- | --- |
| Superscalar issue | Starting more than one instruction in a cycle |
| Out-of-order issue | Starting a later ready instruction before an earlier unready one |
| Nonblocking memory machinery | Letting another request progress while an earlier miss is unresolved |
| Memory-level parallelism | Memory operations overlapping in time, at a stated level and for a stated population |

A core can issue one instruction per cycle and still keep several long-lived requests outstanding across many cycles. Berkeley's original Rocket paper describes a single-issue, in-order scalar pipeline with a nonblocking data cache that supports MLP \[9\]. Arm's Cortex-A55 optimisation guide documents three outstanding data-cache misses under particular pipeline-spacing conditions, despite in-order issue \[8\].

That doesn't mean any in-order core matches an aggressive out-of-order one. An in-order design is much more sensitive to software ordering: if a dependent use blocks issue before the next independent load is reached, the independence can't help, so grouping independent loads before their uses matters. Out-of-order execution widens the processor's ability to find and exploit independence on its own; it isn't what defines MLP.

## 3\. The historical benchmark in detail

The linked repository is `lemire/testingmlp`. Its [last published revision before the article](<https://github.com/lemire/testingmlp/tree/23569032cd784366247c0381e4d532c9a82f3263>) dates from January 2021. That establishes the public implementation available at the time; it doesn't prove the author's exact local checkout or command line. Reading today's changed benchmark as if it were the 2022 implementation would be unsafe. A later post by the same author \[29\] uses the evolved benchmark, and its settings shouldn't be conflated with the 2022 test. The 2022 post reports approximately:

| Reported quantity | Ice Lake system | Graviton 3 system |
| --- | ---: | ---: |
| Single-lane access time | 90 ns | 120 ns |
| Peak implied bandwidth | 12 GB/s | 9 GB/s |
| Described simultaneous loads | about 25 | about 19 |

with Ubuntu 22.04, GCC 11 and 4 KiB pages. These are measurements of those configurations, not permanent limits attached to the processor names.

### Construction

The historical default is an array of 33,554,432 `uint64_t` entries, 256 MiB. A second array of the same size supports setup, outside the timed traversal. Sattolo's algorithm builds a random permutation that is one long cycle, and the lanes' starting positions are spaced along it. The test uses array-index links and several traversal states, not separately allocated list nodes \[2\].

The generator emits a distinct function for each lane count, with one local state variable per lane. The inner loop does one dependent lookup for each lane, then repeats. Total useful accesses are approximately fixed at \\(H(K) = K \\lfloor H/K \\rfloor\\), so adding lanes changes the schedule of roughly the same accesses rather than multiplying the work. Loop control is amortised over more accesses, and register allocation can change \[3\].

The timing takes the fastest of five wall-clock trials, and the traversal is kept observable through its returned result. The code collects no retired instructions, actual CPU cycles, request occupancy, cache-source information, page-walk counters or memory-controller traffic \[2\].

### The bandwidth figure

The program computes an implied bandwidth: completed logical accesses times 64 bytes, divided by time. Each architectural load reads eight bytes, so the calculation assumes one cache-line transfer per logical access; it doesn't measure DRAM traffic. The source divides by powers of 1024 while printing MB/s, a small ambiguity in the unit label \[2\].

Actual traffic differs with cache hits, several entries sharing a line, merged requests, hardware prefetches, page-table reads and other memory activity. Eight 64-bit entries fit in one 64-byte line, so a large random array doesn't guarantee a DRAM transaction for every load. And with several lanes, the ns/hit column is elapsed time divided by aggregate accesses: the reciprocal of throughput, not the service latency of one request.

### Questions the benchmark leaves open

The design deliberately makes long-latency dependent loads dominant and uses independent chains to expose overlap. That's a sensible microbenchmark. A stricter review would also ask how much work the compiler adds, where requests are served, whether translation limits the result, how stable the curve is, and how loaded latency changes as lanes are added.

## 4\. Saturation, translation and read size

The historical code reports a possible number of parallel paths when another lane improves time by less than one percent. A noisy point, a spill threshold or a different resource limit can produce such a knee, so a plateau is an observation that needs interpreting \[2\]. The four quantities often called MLP (software independence, hardware capacity, achieved occupancy, speedup) need not be equal: a program may have 32 active lanes and eight outstanding requests on average; a ten-entry resource may contribute to a system that supports a different count at another boundary; and demand loads, hardware prefetches and page walks may occupy overlapping but different resources.

Travis Downs, a collaborator on the benchmark, explains in the 2019 discussion that the core, the uncore, the memory controller and DRAM all affect the observed overlap. Taking the smallest queue capacity is insufficient because a request holds each resource for a different time. The same discussion shows page-size changes materially moving the saturation point \[10\].

### Translation

A 256 MiB region spans 65,536 pages at 4 KiB or 128 at 2 MiB. Those counts are arithmetic; the performance that follows depends on the TLBs and page walkers. A TLB miss may need memory accesses of its own before the data access can go ahead.

The historical README warns about this and recommends transparent huge pages. The source requests `MADV_HUGEPAGE`, while the article says the tests used small pages. These aren't in conflict: a request for huge pages doesn't establish the pages obtained, Linux can fall back to ordinary pages, and the effective backing has to be checked \[4, 30\].

For an application comparison, default page behaviour can legitimately be part of the system being compared. To isolate a core's data-miss machinery, translation has to be controlled or measured separately. Both are useful experiments; they answer different questions.

### Read size

There are several granularities: the operand size, the cache-line fill, the useful payload consumed, and the page used for translation. A four-byte lookup can need a 64-byte fill, two adjacent four-byte values may share that line, and an access crossing a line boundary may need two. Widening an operand from four to eight bytes doesn't automatically double the memory cost, while growing a node from one line to two can matter a great deal.

The right bandwidth denominator depends on the question. Useful decoded bytes describe the application's productivity; transferred cache lines describe a memory interface; retired loads describe instructions. None substitutes for the others.

## 5\. Throughput models

### A lower-bound envelope

With \\(K\\) independent chains, recurrence latency \\(L\\), core execution work per step \\(u\\), sustainable issue or execution rate \\(P\\), bytes transferred per step \\(b\\) and sustainable bandwidth \\(B\\), an optimistic envelope is

\\\[ \\text{cycles per useful step} \\ge \\max\\left(\\frac{L}{K},\\ \\frac{u}{P},\\ \\frac{b}{B}\\right) \\\]

with \\(L\\) in cycles, \\(P\\) in operations per cycle and \\(B\\) in bytes per cycle. Separate port limits may matter more than a single aggregate \\(P\\). It's a reasoning aid, not a calibrated simulator: it leaves out branch recovery, burstiness, queueing and imperfect scheduling.

For a specific resource \\(j\\), if a useful step needs \\(m\_j\\) requests that each hold the resource for \\(W\_j\\) seconds, and it has \\(Q\_j\\) entries, then

\\\[ \\text{useful steps per second} \\le \\frac{Q\_j}{m\_j W\_j} \\\]

That is why taking \\(\\min(Q\_j)\\) across unrelated queues is wrong: their occupancy times and request populations differ. A small queue passed through quickly can sustain more throughput than a large one whose entries stay occupied.

### How fewer instructions can help

Once enough chains are exposed that \\(L/K\\) no longer dominates, an instruction change may:

- shorten the dependency that produces the next address;
- reduce load/store or address-generation pressure;
- reduce front-end work;
- reduce register pressure and spills;
- fit more useful independent loads into the finite instruction window;
- reduce branch or bookkeeping overhead;
- change prefetch timing.

These are different mechanisms. The finite window is the most relevant here: with many independent chains in the source, the processor still has to reach their loads before its tracking resources fill, so removing intervening work can raise the MLP achieved without changing the memory hierarchy. This interaction is central to the ISCA MLP work and to Cimple \[5, 7\].

More lanes can also make things worse. Lane state may spill to memory, code may grow, output streams may take cache space and write buffers, scheduling can get more expensive, and more concurrency may raise memory queueing and so \\(L\\) itself. AMAC and Cimple arrange independent memory-bound tasks within one thread and confront exactly these costs. Their value here is the mechanism and the experimental framing, not their headline speedups or optimal group sizes \[6, 7\].

### IPC is a ratio, not a score

Over one interval, \\(\\mathrm{IPC} = I/C\\), and cycles per useful byte \\(= (I/B)/\\mathrm{IPC}\\), where \\(B\\) is useful output. A hypothetical change:

| Version | Instructions/byte | IPC | Cycles/byte |
| --- | ---: | ---: | ---: |
| A | 12 | 3.0 | 4.0 |
| B | 8 | 2.5 | 3.2 |

B uses 20% fewer cycles per byte with lower IPC. Conversely, extra easy work can raise IPC without raising useful throughput. A rule like "IPC below one means memory-bound" is too crude; Intel's and Arm's methodologies use several categories and follow-up measurements \[14, 31\].

An instruction can expand into several back-end operations, and fusion can change some of the accounting without removing all of the work, so two sequences with similar instruction counts can load the ports, store machinery and front end quite differently. For the walk, the useful questions are which instructions delay the next table address, which consume the resource that currently limits it, which make memory accesses only to maintain lane state, and which occupy the window while more useful loads wait to become visible.

Misses per thousand instructions can mislead during an instruction optimisation: halve the instructions with misses per output byte unchanged, and MPKI doubles. Track misses and requests per BWT row or per final output byte as well.

### Little's law needs a boundary and a population

For a stable, consistently measured request population, \\(\\overline{N} = \\lambda \\overline{W}\\): average outstanding requests equal the completion rate times the average residence time. If each counted request transfers one line of \\(S\\) bytes, line bandwidth is about \\(\\overline{N} S / \\overline{W}\\). But a retired load, a fill-buffer allocation, an L2-miss transaction and a DRAM read needn't be the same population, and the latency of a lightly loaded serial chain needn't be the latency of a heavily loaded multi-chain test: queueing, the source of the line and prefetching all change it. Higher occupancy can mean more useful overlap, or slower service.

## 6\. Counter definitions and their populations

Let \\(O\\) be an occupancy counter that adds the number of qualifying outstanding requests every cycle, \\(A\\) the number of matching requests, \\(P\\) the cycles with at least one pending, and \\(C\\) all relevant cycles.

| Ratio | Units | Meaning |
| --- | --- | --- |
| \\(O/A\\) | cycles per request | Average residence time |
| \\(O/P\\) | requests | Average occupancy, given that a request is pending |
| \\(O/C\\) | requests | Average occupancy over all measured cycles |

These follow from what is accumulated. Before applying them, check that numerator and denominator cover the same requests, interval, privilege filters and hardware scope, and handle requests already outstanding at the start of a short window or still outstanding at its end.

### The L1 demand-associated metric

Intel defines an MLP metric as `L1D_PEND_MISS.PENDING` divided by `L1D_PEND_MISS.PENDING_CYCLES` \[11, 12\]. Its event definition concerns demand-associated fill buffers. A fill a prefetch started may begin to count once a demand access joins it, and page-walk reads and split accesses complicate the correspondence with source-level loads. It isn't a count of independent BWT chains, of total prefetch concurrency or of DRAM requests.

In our archive the conditional value stays near 2.18 from one worker to four while the share of cycles with a pending demand miss rises from about 12% to 30% \[24\]. As an illustration, \\(2.18 \\times 0.12 \\approx 0.26\\) and \\(2.18 \\times 0.30 \\approx 0.65\\): an unchanged conditional average beside a large rise in occupancy over all cycles. Exact aggregation should use the matching raw totals, and because these are process-wide figures, a shift in the time spent in each phase moves them too.

When comparing workers, sum the numerators and denominators before forming a weighted aggregate. That gives a cycle-weighted average per logical processor, not the total number of requests outstanding at once across the machine. An unweighted average of per-thread ratios can give short-lived or lightly loaded threads too much weight.

### The offcore ratio

For the relevant Intel event family, `OFFCORE_REQUESTS_OUTSTANDING.ALL_DATA_RD` integrates outstanding data reads from an L2 miss to completion at the superqueue boundary. Divided by `OFFCORE_REQUESTS.ALL_DATA_RD`, it gives cycles per request. The population mixes demand reads, prefetches and relevant page-walk reads, and can include requests served by the LLC, so it isn't automatically DRAM latency \[11\].

Our archived median rises from roughly 39 to 58 cycles per request with four workers (an arithmetic mean gives about 41 to 58) \[24, 25\]. That's consistent with a longer average residence for the counted population. It doesn't prove that an unchanged set of useful BWT requests each became slower by that amount: the source mix, phase mix and request mix can all change.

### Prefetch and demand-only counts

A line can arrive before its demand use, or a demand access can join a fill already in flight. A retired demand-load classification describes that load's encounter with the hierarchy, not necessarily the line's full history. `MEM_LOAD_RETIRED.FB_HIT` says a load merged into a still-pending fill for the same line; the initiating request may have been demand or prefetch, so it doesn't show that the corresponding share of work was supplied by prefetch \[11\]. For the walk, which prefetches every next row, counting only demand loads that miss in L3 can leave out much of the expensive traffic.

### Two withdrawn readings

The corrected mibz2 memory proposal withdraws an earlier estimate that about 98% of walk traffic was prefetch, because retired demand loads reaching L3 and all offcore reads are different populations. It also withdraws reading an inferred count near six as measured, constant walk MLP: the assumed request rate, a mixed process-wide latency proxy and the stage time partly constructed the apparent agreement \[25\]. Those corrections stand.

### Counter values need causal context

More outstanding requests might mean more useful parallel work, or slower service. More DRAM writes might cause contention, or result from evicting dirty data. Fewer retired demand misses might reflect effective prefetching while total traffic grows. Events can also overlap or nest, so adding several stall counts is invalid when one cycle belongs to more than one; and sampling has different semantics from counting. A coherent set of counters is evidence for distinguishing hypotheses, not a self-interpreting explanation.

## 7\. A measurement programme for mibz2

The aim is to resolve the remaining ambiguity without repeating experiments that are already closed, such as the 32-lane unroll.

### Objectives, phases and axes

For the production decoder the objectives are wall-clock decoded throughput, CPU-seconds per GB, behaviour across blocks and memory footprint; for understanding the walk, actual cycles, nanoseconds, requests, instructions and traffic per BWT row. A stage improvement and a decoder improvement are separate results.

Measure table construction, traversal, segment assembly and the later RLE and CRC work separately, and keep the whole-decoder test, using the production sequence of blocks and buffer lifetimes: a prebuilt table traversed repeatedly can be much warmer than one met during real decoding. Keep lane count, segment count, prefetch policy, worker count, lane-state layout and output-buffer lifetimes as separate axes. More lanes try to improve overlap within a core; more workers add cores and pressure on shared resources.

### What to collect

| Measurement | What it helps answer |
| --- | --- |
| Actual cycles and ns per BWT row | Did useful service improve, and did the frequency change? |
| Retired instructions per row | Did dynamic instruction work fall? |
| Branch misses per row | Did recovery costs change? |
| Load/store and resource-pressure evidence | Did the change reduce the resource that matters? |
| Requests and supplier information per row | How much counted traffic is there, and where is it served? |
| Conditional occupancy | How many requests overlap when any is pending? |
| Pending-cycle fraction and unconditional occupancy | How much of the time is that activity going on? |
| Matching residence-time estimate | Are requests taking longer to finish? |
| Whole-decoder throughput and CPU cost | Does the change improve the actual program? |

No single row is a verdict; the combination should support or contradict a prediction written down beforehand:

| Hypothesis | Result that would support it |
| --- | --- |
| Too few independent loads become visible | Better scheduling raises the useful request rate and cuts stage time, without extra traffic. |
| Lane bookkeeping consumes a limiting resource | A controlled layout or instruction change reduces that resource and the native stage cycles, with similar request behaviour. |
| Translation is significant | Verified mapping changes move page-walk activity and stage time together. |
| LLC residency deteriorates | Supplier and replacement evidence changes with a controlled live footprint. |
| Shared request service is congested | Residence time and slowdown respond to request pressure even with the footprint controlled. |
| Output traffic interferes with traversal | A targeted change to output lifetime or write traffic improves the affected stage and the decoder. |

None of these interventions is pure: changing instructions changes request timing, changing page size can change placement, and changing traffic changes queueing. Record those consequences rather than assume them away.

### Track A: the machine and the method

Compare a single chain, an unrolled version of that same chain, and several interleaved independent chains, holding useful accesses fixed. That separates amortised loop control from inter-chain overlap more cleanly than comparing one chain only with many. Sweep the active working set through the cache levels and beyond the LLC, saying whether total footprint or footprint per chain is held fixed; vary node spacing independently of payload size; compare verified page mappings, not just allocation requests; and establish single-core behaviour before adding cores. Inspect the generated loop at each important lane count, and keep the executable, compiler flags, the input permutation or its hash, affinity, page backing and raw timings.

### Track B: the decoder's limiting work

Use the production workload for any shipping decision, keep cold and warm microbenchmarks if useful and label them, and keep the whole-decoder benchmark beside every stage measurement, because removing a copy, changing a buffer or growing the code can affect another phase or the other workers.

### Counter mechanics and the existing harnesses

Small Linux perf event groups can collect matching counters together. For per-thread phase measurements, bracket a substantial block or batch, use consistent filters, keep the enabled and running times, and reject unscheduled groups. Subtract the time metadata as well as the counts when taking interval deltas: resetting counts doesn't reset all the time accounting. Measure the cost of an empty bracket and keep the uninstrumented timing \[17, 18\]. Prefer a few purpose-built groups to one oversized collection that multiplexes heavily, since a scaled count assumes the scheduled intervals represent the whole interval.

The current sweep covers several lane counts, prefetch hints, and room against flagged walkers; it keeps decode order and has an optional separate PMU pass, and its byte denominator is BWT rows. The phases example reports per final decoded byte. Both report c/B by converting elapsed seconds with a supplied nominal clock, which isn't a PMU cycle count; a fixed multiplier cancels in a ratio, but those values shouldn't be mixed with PMU cycles or across byte definitions \[26, 27\].

## 8\. What the tools can establish

**Disassembly** shows whether independent loads stay visible, whether lane state is reloaded, whether bounds checks or spills remain, and where code grew. It can't show the cache residency or service time a run experiences. Static intuition from the source is especially unreliable for a compiler-managed array of lane state, so archive the binary behind each timed result.

**LLVM-MCA** helps investigate dependencies and execution-resource pressure, but it doesn't predict a workload's cache behaviour and its default load latencies are optimistic \[15\]. Treat it as a source of candidate explanations and lower bounds; a prediction that disagrees with measured time tells you about the model's assumptions.

**Cachegrind** gives reproducible instruction counts that make regressions easy to catch, but its cache and branch simulations are basic \[16\]. A deterministic instruction-count improvement is evidence of less work executed under that tool, not of native overlap, prefetch timeliness or contention.

**Hardware counters** observe defined events; they don't label causes. The CPU model, event definition, filtering and errata all matter, and Intel and Arm events with similar names needn't count equivalent things \[13, 14\]. Cross-ISA IPC is unsuitable as a standalone score, since x86 and AArch64 express the same work in different instruction sequences. Top-down categories narrow a problem without replacing a study of the useful work and the intervention \[31\].

## 9\. Open questions

**Prefetch lead time is coupled to everything else.** The walk prefetches one link ahead, but the time before demand returns to a lane depends on the lane count and on the time spent advancing the other lanes. Removing bookkeeping shortens that interval, so an optimisation may consume a prefetched line sooner and expose more of its latency even while cutting instructions. That is a hypothesis drawn from the schedule, not a claim that it caused the archived unrolling result; testing it needs timing and request-population evidence. A second unknown hop can't be prefetched by adding another hint: its address has to be discovered first, or a representation has to precompute a shortcut, with its own build cost, footprint and cache behaviour.

**Local and end-to-end objectives diverge.** For a phase taking fraction \\(f\\) of total time, speeding it up by \\(s\\) gives an idealised whole-program speedup of \\(1 / ((1 - f) + f/s)\\), if nothing else changes, and in a concurrent decoder something else usually does. Wall throughput and CPU cost can also diverge: four workers finish sooner while using more CPU-seconds per GB.

**What remains unknown.** The public article doesn't establish the exact 2022 binary and runtime configuration, and its measured knee can't identify a hardware queue's capacity. For mibz2, the archived evidence doesn't yet divide the four-worker penalty among LLC capacity, request congestion, writeback interference, prefetch timing and translation, and the static address-port model alone doesn't establish the limiting resource of the unrolled candidate.

## 10\. Further reading

**Kärkkäinen, Kempa and Puglisi, Slashing the Time for BWT Inversion** \[19\]. The algorithm itself: reducing costly accesses and overlapping inversions from several starting positions, which separates how much expensive work there is from how it's scheduled.

**Chou, Fahs and Abraham, Microarchitecture Optimizations for Exploiting Memory-Level Parallelism** \[5\]. Why finite instruction windows, execution structure and branches bound the memory overlap a core can expose.

**Kocberber, Falsafi and Grot, Asynchronous Memory Access Chaining** \[6\]. Suspended memory-bound tasks, uneven progress and the cost of scheduling them. Its speedups aren't predictions for BWT.

**Kiriansky et al., Cimple: Instruction and Memory Level Parallelism** \[7\]. ILP and MLP as working together, including the limits from intervening instructions and execution windows.

**Travis Downs on the earlier benchmark** \[10\]. Several resources along the path, different occupancy times, and translation, for this same benchmark family.

**Intel's event and metric definitions** \[11, 12\]. What each ratio counts. Reading the exact numerator and denominator is worth more than any secondary name for a metric.

## 11\. Source register and revision pins

Primary publications, original benchmark implementations, manufacturer and tool documentation, and the repository's own experimental records. Publication dates follow the documents' own contents.

### The exchange and the benchmark

- **\[1\]** Daniel Lemire, [Memory-level parallelism: Intel Ice Lake versus Amazon Graviton 3](<https://lemire.me/blog/2022/06/07/memory-level-parallelism-intel-ice-lake-versus-amazon-graviton-3/>), 7 June 2022.
- **\[2\]** `lemire/testingmlp`, [`testingmlp.cpp`](<https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/testingmlp.cpp>): timing, allocation and bandwidth, revision `23569032cd784366247c0381e4d532c9a82f3263`.
- **\[3\]** `lemire/testingmlp`, [`gen.py`](<https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/gen.py>): generated lanes, same revision.
- **\[4\]** `lemire/testingmlp`, [`README.md`](<https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/README.md>): the page-walk limitation, same revision.

### Research and architecture

- **\[5\]** Yuan Chou, Brian Fahs and Santosh Abraham, [Microarchitecture Optimizations for Exploiting Memory-Level Parallelism](<https://pages.cs.wisc.edu/~markhill/restricted/isca04_mlp.pdf>), ISCA 2004.
- **\[6\]** Onur Kocberber, Babak Falsafi and Boris Grot, [Asynchronous Memory Access Chaining](<https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf>), PVLDB 9(4), 2015, pp. 252–263.
- **\[7\]** Vladimir Kiriansky, Haoran Xu, Martin Rinard and Saman Amarasinghe, [Cimple: Instruction and Memory Level Parallelism](<https://people.csail.mit.edu/rinard/paper/pact18.pdf>), PACT 2018.
- **\[8\]** Arm, [Cortex-A55 Software Optimization Guide](<https://documentation-service.arm.com/static/5f1fe66bbb903e39c84d7d75>), version 3.0, section 5.1, 2018.
- **\[9\]** Yunsup Lee et al., [A 45nm 1.3GHz 16.7 Double-Precision GFLOPS/W RISC-V Processor with Vector Accelerators](<https://people.eecs.berkeley.edu/~krste/papers/eos18-esscirc2014.pdf>), ESSCIRC 2014.
- **\[10\]** Daniel Lemire, with discussion by Travis Downs, [Memory-level parallelism: Intel Skylake versus Intel Cannonlake](<https://lemire.me/blog/2019/01/01/memory-level-parallelism-intel-skylake-versus-intel-cannonlake/>), 1 January 2019; Downs's comment of 4 January.

### Counter and tool definitions

- **\[11\]** Intel, [Skylake-family core event definitions](<https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json>), used by the applicable Comet Lake model mapping; retrieved file blob `ae95ceeb47ac00d4398cfbc8c6e09a017207c550` (a blob identity, not a commit).
- **\[12\]** Intel, [Skylake derived metric definitions](<https://github.com/intel/perfmon/blob/main/SKL/metrics/skylake_metrics.json>), including the conditional MLP and offcore metrics; retrieved blob `2e7894d889c19bc81091c23a1ad662da61fd830f`.
- **\[13\]** Intel, [Ice Lake Server core-event documentation](<https://perfmon-events.intel.com/platforms/icelakex/core-events/core/>).
- **\[14\]** Arm, [Neoverse V1 Core: Performance Analysis Methodology](<https://armkeil.blob.core.windows.net/developer/Files/pdf/white-paper/neoverse-v1-core-performance-analysis.pdf>).
- **\[15\]** LLVM, [llvm-mca command guide](<https://llvm.org/docs/CommandGuide/llvm-mca.html>), including its memory-model limitations.
- **\[16\]** Valgrind, [Cachegrind manual](<https://valgrind.org/docs/manual/cg-manual.html>), overview and simulation limits.
- **\[17\]** Linux perf project, [perf-stat manual](<https://man7.org/linux/man-pages/man1/perf-stat.1.html>).
- **\[18\]** Linux man-pages, [perf\_event\_open manual](<https://man7.org/linux/man-pages/man2/perf_event_open.2.html>).

### BWT inversion

- **\[19\]** Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, [Slashing the Time for BWT Inversion](<https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf>), DCC 2012, pp. 99–108.

### mibz2 source and experimental records

- **\[20\]** [`src/bwt.rs`](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs>), master at `13e9854805eed8944ecdd7c9330e30971b6bf52d`.
- **\[21\]** [32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-31-room-walk-overlap/index.md>), 31 October 2026 ([at `13e9854`](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/site/blog/2026-10-31-room-walk-overlap.md>)).
- **\[22\]** [The unrolled room walk against the assembly-level prescription](<https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-astra-fidelity/README.md>), archive `270333c2d32fe16dc24ae7ab64f3697d87939418`.
- **\[23\]** [The unrolled room walk's full-decoder trial on the test workstation: results](<https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/results.md>), same archive.
- **\[24\]** [Decoder attribution, one worker against four: analysis and final erratum](<https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md>), archive `33e0e4ebac554bae03795bfb76e86440b33bae6b`.
- **\[25\]** [Inverse-BWT memory proposal, with its corrections](<https://github.com/lmmx/mibz2/blob/0156e1dad5ce4ebca78b4cb3d00276d3b6d64611/docs/journal/2026-11-01-bwt-memory-proposal.md>), archive `0156e1dad5ce4ebca78b4cb3d00276d3b6d64611`.
- **\[26\]** [`examples/sweep.rs`](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/examples/sweep.rs>), master at `13e9854`.
- **\[27\]** [`examples/phases.rs`](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/examples/phases.rs>), same revision.
- **\[28\]** [Investigating inverse-BWT memory interference in mibz2](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-01-inverse-bwt-memory-interference/index.md>), 1 November 2026 ([at `13e9854`](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/site/blog/2026-11-01-inverse-bwt-memory-interference.md>)).

### Later and supporting material

- **\[29\]** Daniel Lemire, [Memory-level parallelism: AMD is the king](<https://lemire.me/blog/2026/07/25/memory-level-parallelism-amd-is-the-king/>), 25 July 2026: the benchmark's later evolution, not the 2022 settings.
- **\[30\]** Linux kernel, [Transparent Hugepage Support](<https://docs.kernel.org/admin-guide/mm/transhuge.html>).
- **\[31\]** Intel, [Top-down Microarchitecture Analysis Method](<https://www.intel.com/content/www/us/en/docs/vtune-profiler/cookbook/2024-0/top-down-microarchitecture-analysis-method.html>), VTune Profiler Performance Analysis Cookbook.
