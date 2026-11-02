---
title: "Memory-level parallelism: measurement notes and source record"
description: "Methodology behind the memory-level parallelism post: the 2022 benchmark in detail, throughput models, Intel counter definitions and their populations, a measurement programme for mibz2, and the full source register."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Memory-level parallelism: measurement notes and source record

The methodology behind [Memory-level parallelism in
mibz2](2026-11-02-memory-level-parallelism.md): the 2022 benchmark in detail, throughput
models, counter definitions and their populations, a measurement programme for mibz2, and the
full source register.
{: .post-dek }

2 November 2026 · 16 min read · [Source register](#8-source-register-and-revision-pins)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

These notes accompany [the main post](2026-11-02-memory-level-parallelism.md). Where the post
already explains something fully, they link back to it rather than repeat it. They examine
the screenshots reproduced in the post, the historical benchmark behind Daniel Lemire's June
2022 article, primary architecture literature, vendor counter documentation, and the source
and archived experiments in `lmmx/mibz2`. Repository inspection is pinned to the revisions in
the [source register](#8-source-register-and-revision-pins); sources were inspected on
2 November 2026. Performance results are attributed to their original experiments; no new
native benchmark of mibz2 was run for these notes. Numbers in square brackets refer to the
register.

## 1. The historical benchmark in detail

The linked repository is `lemire/testingmlp`. Its last published revision before the article
is `23569032cd784366247c0381e4d532c9a82f3263`, from January 2021. That establishes the public
implementation available at the time; it doesn't prove the author's exact local checkout or
command line. Reading today's changed benchmark as if it were the 2022 implementation would be
unsafe. A later post by the same author, [29], uses the evolved benchmark, and its settings
shouldn't be conflated with the 2022 test.

### Construction

The historical default is an array of 33,554,432 `uint64_t` entries, 256 MiB. A second array of
the same size supports setup, outside the timed traversal. Sattolo's algorithm builds a random
permutation that is one long cycle, and the lanes' starting positions are spaced along it. The
test uses array-index links and several traversal states, not separately allocated list nodes
[2].

The generator emits a distinct function for each lane count, with one local state variable per
lane. The inner loop does one dependent lookup for each lane, then repeats. Total useful
accesses are approximately fixed at $H(K) = K \lfloor H/K \rfloor$, so adding lanes changes the
schedule of roughly the same accesses rather than multiplying the work. Loop control is
amortised over more accesses, and register allocation can change [3].

The timing takes the fastest of five wall-clock trials, and the traversal is kept observable
through its returned result. The code collects no retired instructions, actual CPU cycles,
request occupancy, cache-source information, page-walk counters or memory-controller traffic
[2].

### The bandwidth figure

The program computes an implied bandwidth: completed logical accesses times 64 bytes, divided
by time. Each architectural load reads eight bytes, so the calculation assumes one cache-line
transfer per logical access; it doesn't measure DRAM traffic. The source divides by powers of
1024 while printing MB/s, a small ambiguity in the unit label [2].

Actual traffic differs with cache hits, several entries sharing a line, merged requests,
hardware prefetches, page-table reads and other memory activity. Eight 64-bit entries fit in one
64-byte line, so a large random array doesn't guarantee a DRAM transaction for every load. And
with several lanes, the ns/hit column is elapsed time divided by aggregate accesses: the
reciprocal of throughput, not the service latency of one request.

### What a fair review asks

The design deliberately makes long-latency dependent loads dominant and uses independent chains
to expose overlap. That's a sensible microbenchmark. A stricter review would also ask how much
work the compiler adds, where requests are served, whether translation limits the result, how
stable the curve is, and how loaded latency changes as lanes are added. The appropriate
conclusion is neither that the method is invalid nor that other measurements are unnecessary:
it's a useful characterisation with a well-defined workload and limits to its interpretation.

## 2. Saturation, translation and read size

The historical code reports a possible number of parallel paths when another lane improves
time by less than one percent. A noisy point, a spill threshold or a different resource limit
can produce such a knee, so a plateau is an observation that needs interpreting [2]. The main
post lists [the four quantities often called MLP](2026-11-02-memory-level-parallelism.md#4-four-different-things-called-mlp),
which need not be equal: a program may have 32 active lanes and eight outstanding requests on
average; a ten-entry resource may contribute to a system that supports a different count at
another boundary; and demand loads, hardware prefetches and page walks may occupy overlapping
but different resources.

Travis Downs, a collaborator on the benchmark, explains in the 2019 discussion that the core,
the uncore, the memory controller and DRAM all affect the observed overlap. Taking the smallest
queue capacity is insufficient because a request holds each resource for a different time. The
same discussion shows page-size changes materially moving the saturation point [10].

### Translation

A 256 MiB region spans 65,536 pages at 4 KiB or 128 at 2 MiB. Those counts are arithmetic; the
performance that follows depends on the TLBs and page walkers. A TLB miss may need memory
accesses of its own before the data access can go ahead.

The historical README warns about this and recommends transparent huge pages. The source
requests `MADV_HUGEPAGE`, while the article says the tests used small pages. These aren't in
conflict: a request for huge pages doesn't establish the pages obtained, Linux can fall back to
ordinary pages, and the effective backing has to be checked [4, 30].

For an application comparison, default page behaviour can legitimately be part of the system
being compared. To isolate a core's data-miss machinery, translation has to be controlled or
measured separately. Both are useful experiments; they answer different questions.

### Read size

The exchange's question about how large each read is matters because there are several
granularities: the operand size, the cache-line fill, the useful payload consumed, and the page
used for translation. A four-byte lookup can need a 64-byte fill, two adjacent four-byte values
may share that line, and an access crossing a line boundary may need two. Widening an operand
from four to eight bytes doesn't automatically double the memory cost, while growing a node
from one line to two can matter a great deal.

The right bandwidth denominator depends on the question. Useful decoded bytes describe the
application's productivity; transferred cache lines describe a memory interface; retired loads
describe instructions. None substitutes for the others.

## 3. Throughput models

### A lower-bound envelope

With $K$ independent chains, recurrence latency $L$, core execution work per step $u$,
sustainable issue or execution rate $P$, bytes transferred per step $b$ and sustainable
bandwidth $B$, an optimistic envelope is

$$
\text{cycles per useful step} \ge \max\left(\frac{L}{K},\ \frac{u}{P},\ \frac{b}{B}\right)
$$

with $L$ in cycles, $P$ in operations per cycle and $B$ in bytes per cycle. Separate port limits
may matter more than a single aggregate $P$. It's a reasoning aid, not a calibrated simulator:
it leaves out branch recovery, burstiness, queueing and imperfect scheduling.

For a specific resource $j$, if a useful step needs $m_j$ requests that each hold the resource
for $W_j$ seconds, and it has $Q_j$ entries, then

$$
\text{useful steps per second} \le \frac{Q_j}{m_j W_j}
$$

That is why taking $\min(Q_j)$ across unrelated queues is wrong: their occupancy times and
request populations differ.

### The bottleneck moves

With one chain, $L/K$ may dominate, and removing independent bookkeeping can do little because
the core is waiting for the next address. More chains lower that floor until something else
dominates. The main post lists [the distinct ways fewer instructions can then
help](2026-11-02-memory-level-parallelism.md#5-why-instructions-matter-and-when-counting-them-misleads).

More lanes can also make things worse. Lane state may spill to memory, code may grow, output
streams may take cache space and write buffers, scheduling can get more expensive, and more
concurrency may raise memory queueing and so $L$ itself. AMAC and Cimple arrange independent
memory-bound tasks within one thread and confront exactly these scheduling costs and window
limits. Their value here is the mechanism and the experimental framing, not their headline
speedups or optimal group sizes [6, 7].

### Little's law needs a boundary and a population

For a stable, consistently measured request population:

```text
average outstanding requests = completed requests per second × average seconds per request
```

If each counted request transfers one line of $S$ bytes, the line bandwidth is about
outstanding × $S$ ÷ latency. But an application load, a fill-buffer allocation and an offcore
transaction needn't correspond one to one.

Time-weighted occupancy needs no CPU-specific assumption. Count the outstanding requests on
every cycle and add the counts: a request present for 50 cycles contributes 50. Dividing the
sum by total cycles gives average occupancy; dividing it by the number of matching requests
gives average residence time, with the boundaries handled appropriately. The main post's
[figure](2026-11-02-memory-level-parallelism.md#9-counters-and-their-denominators) works one
small trace through all three ratios.

Substituting unloaded serial latency for loaded multi-chain latency is only an approximation:
queueing, the source of the line and prefetching can all change latency. A higher outstanding
count can mean slower service rather than more useful progress, so occupancy should be read
together with throughput and residence time.

## 4. Counter definitions and their populations

The main post covers [the three
denominators](2026-11-02-memory-level-parallelism.md#9-counters-and-their-denominators), the
conditional L1 metric, the offcore ratio and `FB_HIT`. These notes add the definitions and the
cases that need most care.

### The L1 demand-associated metric

Intel defines an MLP metric as `L1D_PEND_MISS.PENDING` divided by
`L1D_PEND_MISS.PENDING_CYCLES`. Its event definition concerns demand-associated fill buffers. A
fill a prefetch started may begin to count once a demand access joins it, and page-walk reads
and split accesses complicate the correspondence with source-level loads. It isn't a direct
count of independent BWT chains or of DRAM requests [11, 12].

When comparing workers, sum the numerators and denominators before forming a weighted
aggregate. That gives a cycle-weighted average per logical processor, not the total number of
requests outstanding at once across the machine. An unweighted average of per-thread ratios can
hide the fact that some threads or phases account for much more of the time.

### The offcore ratio

For the relevant Intel event family, `OFFCORE_REQUESTS_OUTSTANDING.ALL_DATA_RD` integrates
outstanding data reads from an L2 miss to completion at the superqueue boundary. Divided by
`OFFCORE_REQUESTS.ALL_DATA_RD`, it gives cycles per request. The population mixes demand reads,
prefetches and relevant page-walk reads, and can include requests served by the LLC, so it
isn't automatically DRAM latency [11].

### Prefetch and demand-only counts

A line can arrive before its demand use, or a demand access can join a fill already in flight.
A retired demand-load classification describes that load's encounter with the hierarchy, not
necessarily the line's full history or where it ultimately came from. `MEM_LOAD_RETIRED.FB_HIT`
says a load merged into a still-pending fill for the same line; the initiating request may have
been demand or prefetch, so it doesn't show that the corresponding share of useful work was
supplied by prefetch [11].

### Two withdrawn readings

The corrected mibz2 memory proposal withdraws an earlier estimate that about 98% of walk
traffic was prefetch, because retired demand loads reaching L3 and all offcore reads are
different populations. It also withdraws reading an inferred count near six as measured,
constant walk MLP: the assumed request rate, a mixed process-wide latency proxy and the stage
time partly constructed the apparent agreement [25]. Those corrections stand.

### Counter values need causal context

More outstanding requests might mean more useful parallel work, or slower service. More DRAM
writes might cause contention, or result from evicting dirty data. Fewer retired demand misses
might reflect effective prefetching while total traffic grows. A high cache-hit fraction may
describe a small, unrepresentative subset of all requests.

Events can also overlap or nest: adding several stall counts is invalid when one cycle belongs
to more than one event. Sampling has different semantics from counting, and a sampled
instruction address may not identify every contributing cause. A coherent set of counters is
evidence for distinguishing hypotheses, not a self-interpreting explanation.

## 5. A measurement programme for mibz2

The aim is to resolve the remaining ambiguity without repeating experiments the repository has
already closed. There are two tracks: a synthetic characterisation to understand a machine,
and a production-shaped stage investigation to understand this decoder. The main post gives
[the measurement sheet and the
predictions](2026-11-02-memory-level-parallelism.md#10-what-the-next-experiment-needs-to-distinguish);
these notes add how each track would be run.

### Track A: the machine and the method

Compare a single chain, an unrolled version of that same chain, and several interleaved
independent chains, holding useful accesses fixed. That separates amortised loop control from
inter-chain overlap more cleanly than comparing one chain only with many.

Sweep the active working set through the cache levels and beyond the LLC, and say whether total
footprint or footprint per chain is held fixed. Vary node spacing independently of payload
size. Compare verified page mappings, not just allocation requests. Establish single-core
behaviour before adding physical cores.

Inspect the generated loop at each important lane count, recording spills, branches,
instruction size and memory-operation counts. Keep the executable, compiler flags, the input
permutation or its hash, affinity, page backing and raw timings. These controls make a knee
easier to interpret without pretending to isolate one hardware structure perfectly.

### Track B: the decoder's limiting work

Use the production sequence of blocks and buffer lifetimes. A prebuilt table traversed
repeatedly can be much warmer than one met during decoding; keep cold and warm
microbenchmarks if useful, label the conditions, and use the production workload for the
shipping decision. Separate table construction, traversal, segment assembly and the later RLE
and CRC work, and keep the whole-decoder benchmark, because removing a copy, changing a buffer
or growing the code can affect another phase or the other workers. Keep lane count, segment
count, prefetch policy and worker count as separate axes.

### Measurement mechanics

Small Linux perf event groups can collect matching counters together. For per-thread phase
measurements, bracket a substantial block or batch, use consistent filters, keep the enabled and
running times, and reject unscheduled groups. Subtract the time metadata as well as the counts
when taking interval deltas: resetting counts doesn't reset all the time accounting. Measure
the cost of an empty bracket, and keep the original uninstrumented timing [17, 18].

Prefer a few purpose-built groups to one oversized collection that multiplexes heavily. A
scaled count assumes the scheduled intervals represent the whole interval, which becomes
fragile for short or changing phases. Weigh any gain in precision against the instrumentation's
cost and its effect on code layout.

The current sweep covers several lane counts, prefetch hints, and room against flagged walkers;
it keeps decode order and has an optional separate PMU pass. Its byte denominator is BWT rows.
The phases example reports per final decoded byte, and its c/B column is a nominal-clock
conversion [26, 27]. Those distinctions should be recorded beside every result.

## 6. Tools: further notes

The main post covers [what each tool can
establish](2026-11-02-memory-level-parallelism.md#11-what-the-tools-can-and-cant-establish).
Three further points:

- Static intuition from the source is especially unreliable for a compiler-managed array of
  lane state. A high-level refactor may produce identical machine code, or a small change may
  alter register allocation across the loop. Archive the binary that produced each timed
  result.
- An LLVM-MCA prediction that disagrees with measured time tells you about missing assumptions
  in the model; it doesn't show that the measurement must be wrong [15].
- Cross-ISA IPC is unsuitable as a standalone score: x86 and AArch64 can express the same
  useful computation in different instruction sequences. Compare useful work per unit time,
  then use each ISA's own counters to explain the result [13, 14].

Top-down categories narrow a problem; they don't replace studying the useful work and the
intervention. A lower-level metric may overlap another or rest on an approximation, and a high
retiring fraction can still include avoidable work [31].

## 7. Open questions

### Prefetch lead time is coupled to everything else

The walk prefetches one link ahead, but the time before demand returns to a lane depends on the
lane count and on the time spent advancing the other lanes. Removing bookkeeping can shorten
that interval, so an optimisation may consume a prefetched line sooner and expose more of its
latency, even while cutting instructions. That is a hypothesis drawn from the schedule, not a
claim that it caused the archived unrolling result; testing it needs timing and request-population
evidence. It is also why lane count and instruction cost shouldn't be tuned independently.

A second unknown pointer hop usually can't be prefetched by adding another hint: its address
has to be discovered first, or a representation has to precompute a shortcut. Shortcuts change
build cost, footprint and cache behaviour, and their full cost has to be counted.

### Local and end-to-end objectives

For a phase taking fraction $f$ of total time, speeding it up by $s$ gives an idealised
whole-program speedup of $1 / ((1 - f) + f/s)$, if nothing else changes. In a concurrent
decoder that condition matters: changing one phase can alter shared-memory pressure, scheduling
and its overlap with other phases. Wall throughput and CPU cost can also diverge: four workers
may finish sooner while using more CPU-seconds per GB, and a small CPU saving may barely change
a saturated pipeline's elapsed time.

### What remains unknown

The public article doesn't establish the exact 2022 binary and full runtime configuration, and
its measured knee can't uniquely identify a hardware queue's capacity. For mibz2, the archived
evidence doesn't yet divide the four-worker penalty among LLC capacity, request congestion,
writeback interference, prefetch timing and translation; nor does the static address-port
model alone establish the limiting resource of the unrolled candidate. These are bounded,
testable uncertainties.

## 8. Source register and revision pins

Primary publications, original benchmark implementations, manufacturer and tool documentation,
and the repository's own experimental records. Publication dates follow the documents' own
contents.

### The exchange and the benchmark

- **[1]** Daniel Lemire, [Memory-level parallelism: Intel Ice Lake versus Amazon Graviton 3](https://lemire.me/blog/2022/06/07/memory-level-parallelism-intel-ice-lake-versus-amazon-graviton-3/), 7 June 2022.
- **[2]** `lemire/testingmlp`, [`testingmlp.cpp`](https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/testingmlp.cpp): timing, allocation and bandwidth, revision `23569032cd784366247c0381e4d532c9a82f3263`.
- **[3]** `lemire/testingmlp`, [`gen.py`](https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/gen.py): generated lanes, same revision.
- **[4]** `lemire/testingmlp`, [`README.md`](https://github.com/lemire/testingmlp/blob/23569032cd784366247c0381e4d532c9a82f3263/README.md): the page-walk limitation, same revision.

### Research and architecture

- **[5]** Yuan Chou, Brian Fahs and Santosh Abraham, [Microarchitecture Optimizations for Exploiting Memory-Level Parallelism](https://pages.cs.wisc.edu/~markhill/restricted/isca04_mlp.pdf), ISCA 2004.
- **[6]** Onur Kocberber, Babak Falsafi and Boris Grot, [Asynchronous Memory Access Chaining](https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf), PVLDB 9(4), 2015, pp. 252–263.
- **[7]** Vladimir Kiriansky, Haoran Xu, Martin Rinard and Saman Amarasinghe, [Cimple: Instruction and Memory Level Parallelism](https://people.csail.mit.edu/rinard/paper/pact18.pdf), PACT 2018.
- **[8]** Arm, [Cortex-A55 Software Optimization Guide](https://documentation-service.arm.com/static/5f1fe66bbb903e39c84d7d75), version 3.0, section 5.1, 2018.
- **[9]** Yunsup Lee et al., [A 45nm 1.3GHz 16.7 Double-Precision GFLOPS/W RISC-V Processor with Vector Accelerators](https://people.eecs.berkeley.edu/~krste/papers/eos18-esscirc2014.pdf), ESSCIRC 2014.
- **[10]** Daniel Lemire, with discussion by Travis Downs, [Memory-level parallelism: Intel Skylake versus Intel Cannonlake](https://lemire.me/blog/2019/01/01/memory-level-parallelism-intel-skylake-versus-intel-cannonlake/), 1 January 2019; Downs's comment of 4 January.

### Counter and tool definitions

- **[11]** Intel, [Skylake-family core event definitions](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json), used by the applicable Comet Lake model mapping; retrieved file blob `ae95ceeb47ac00d4398cfbc8c6e09a017207c550` (a blob identity, not a commit).
- **[12]** Intel, [Skylake derived metric definitions](https://github.com/intel/perfmon/blob/main/SKL/metrics/skylake_metrics.json), including the conditional MLP and offcore metrics; retrieved blob `2e7894d889c19bc81091c23a1ad662da61fd830f`.
- **[13]** Intel, [Ice Lake Server core-event documentation](https://perfmon-events.intel.com/platforms/icelakex/core-events/core/).
- **[14]** Arm, [Neoverse V1 Core: Performance Analysis Methodology](https://armkeil.blob.core.windows.net/developer/Files/pdf/white-paper/neoverse-v1-core-performance-analysis.pdf).
- **[15]** LLVM, [llvm-mca command guide](https://llvm.org/docs/CommandGuide/llvm-mca.html), including its memory-model limitations.
- **[16]** Valgrind, [Cachegrind manual](https://valgrind.org/docs/manual/cg-manual.html), overview and simulation limits.
- **[17]** Linux perf project, [perf-stat manual](https://man7.org/linux/man-pages/man1/perf-stat.1.html).
- **[18]** Linux man-pages, [perf_event_open manual](https://man7.org/linux/man-pages/man2/perf_event_open.2.html).

### BWT inversion

- **[19]** Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, [Slashing the Time for BWT Inversion](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf), DCC 2012, pp. 99–108.

### mibz2 source and experimental records

- **[20]** [`src/bwt.rs`](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs), master at `13e9854805eed8944ecdd7c9330e30971b6bf52d`.
- **[21]** [32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles](2026-10-31-room-walk-overlap.md), 31 October 2026 ([at `13e9854`](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/site/blog/2026-10-31-room-walk-overlap.md)).
- **[22]** [The unrolled room walk against the assembly-level prescription](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-astra-fidelity/README.md), archive `270333c2d32fe16dc24ae7ab64f3697d87939418`.
- **[23]** [The unrolled room walk's full-decoder trial on the i9-10900K: results](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/results.md), same archive.
- **[24]** [Decoder attribution on the i9-10900K: analysis and final erratum](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md), archive `33e0e4ebac554bae03795bfb76e86440b33bae6b`.
- **[25]** [Inverse-BWT memory proposal, with its corrections](https://github.com/lmmx/mibz2/blob/0156e1dad5ce4ebca78b4cb3d00276d3b6d64611/docs/journal/2026-11-01-bwt-memory-proposal.md), archive `0156e1dad5ce4ebca78b4cb3d00276d3b6d64611`.
- **[26]** [`examples/sweep.rs`](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/examples/sweep.rs), master at `13e9854`.
- **[27]** [`examples/phases.rs`](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/examples/phases.rs), same revision.
- **[28]** [Investigating inverse-BWT memory interference in mibz2](2026-11-01-inverse-bwt-memory-interference.md), 1 November 2026 ([at `13e9854`](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/site/blog/2026-11-01-inverse-bwt-memory-interference.md)).

### Later and supporting material

- **[29]** Daniel Lemire, [Memory-level parallelism: AMD is the king](https://lemire.me/blog/2026/07/25/memory-level-parallelism-amd-is-the-king/), 25 July 2026: the benchmark's later evolution, not the 2022 settings.
- **[30]** Linux kernel, [Transparent Hugepage Support](https://docs.kernel.org/admin-guide/mm/transhuge.html).
- **[31]** Intel, [Top-down Microarchitecture Analysis Method](https://www.intel.com/content/www/us/en/docs/vtune-profiler/cookbook/2024-0/top-down-microarchitecture-analysis-method.html), VTune Profiler Performance Analysis Cookbook.
