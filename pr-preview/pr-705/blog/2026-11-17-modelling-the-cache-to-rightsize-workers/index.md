[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/index.md>)

# Modelling the cache to rightsize workers

mibz2 now runs fewer workers than threads on a ten-core desktop and finishes sooner, while a cache model is tested to pick the count on any machine.

17 November 2026 · 14 min read · [Research in progress](<#5-what-shipped-and-what-the-model-still-needs>)

Shipped in mibz2 [0.5.4](<https://lmmx.github.io/mibz2/pr-preview/pr-705/benchmarks/history/#release-0-5-4>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#054---2026-11-17>))

mibz2 decodes a bzip2 file by handing its blocks to *workers*: threads that each decode a whole block at a time. How many workers to start is a choice mibz2 makes once, at startup, and the obvious answer, one per thread the caller asks for, is wrong. On our 10-core test machine, running fewer workers than threads first took 11.5% less wall time in pinned runs, or about 13% more throughput, and 12.7% less in a later unpinned run. We found that by measurement alone, sweeping selected worker counts at each thread count. A sweep like that has two faults. A change to the algorithm can change how much memory a worker uses, or how much processor time it needs, and with them the best count, so the sweep has to be redone. And it describes one machine: a policy measured on one processor's 20 MiB cache is "works on my machine" for everyone else.

So we are working to replace the measured policy with a model of what the workers share: the shared cache, the processor time left over after the decoder's other threads, and the processors themselves. When the algorithm changes, we would update the model's inputs instead of re-running the sweep; on another machine, the model would read that machine's caches and processors. The model is a proposal under test, not the decoder's policy. Its predictions are frozen before each new experiment (pre-registered: fixed before the data, and tested out of sample), so it can be shown wrong in public, and it has been. It reproduced the counts it was fitted to and, under pinning, the lower thread counts it was asked about next. At a smaller block size it named the wrong count.

**The simplest correct picture.** Fewer workers give up some parallelism, but each one gets more of the shared cache, so each byte it decodes costs fewer processor cycles. On this machine the cheaper bytes win, up to a point, and the point is roughly where the workers' data would fill most of the cache. Why exactly the bytes get cheaper is only partly known.

**What the sweep found.** The test machine is a ten-core Intel desktop (an i9-10900K), with 20 logical CPUs and a 20 MiB level-3 (L3) cache that all the cores share. On this chip the L3 is the last-level cache (LLC); the post says LLC in general statements and L3 for this machine. The test file is four copies of an 11.9 MB extract of a Wikidata dump, decoded in 10 paired rounds. Every worker in these runs uses the [lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-16-lazy-letters/index.md>) kernel. Below, *threads* means the number the caller asks for, *workers* the number mibz2 runs, and *cores* the physical cores. Each run is compared with a *reference plan* at the same thread count: the plan mibz2 0.5.2 used, one worker a thread up to 10 threads, and nine workers beyond the physical-core count (12 to 20 threads, the SMT regime, where a core runs two threads). Runs of up to 10 threads were limited to one hardware thread a core.

Fewer workers, less time: the test machine at 10 and 8 threads

Measured on an Intel i9-10900K, decoding four copies of an 11.9 MB Wikidata extract, 10 paired rounds a point, with the process limited to one hardware thread a core. Each point is the wall time against the plan of one worker a thread at the same thread count (10 workers at 10 threads, 8 at 8); the bars are 95% intervals over the rounds, on one machine. The hollow point (6 workers at 10 threads) was run as context and not judged. 5 workers at 8 threads, run later in the low-thread sweep, took 0.997 of 8's time: no gain.

At 10 threads, seven workers, with 28 blocks in flight, take 0.885 of the reference plan's time, with a quarter less peak memory. At 8 threads, six workers take 0.929. Past the cores, seven take 0.921, 0.930 and 0.923 of nine workers' time at 12, 16 and 20 threads. A second run tested these choices directly, beside two files that played no part in choosing them, binary records and text compressed with 100 kB blocks, and confirmed all three: 0.881 at 10 threads, 0.933 at 8, and 0.921 to 0.930 past the cores. No cell on either held-out file was judged slower, though on the small-block text every cell was inconclusive, which is not the same as no difference. All these runs of 10 threads or fewer were pinned, and below 10 threads that turned out to matter (section 5).

## What was found

- **From 10 threads up, fewer workers than threads take less time on the test machine, pinned or not.** 7 workers take 0.885 of the time at 10 threads pinned and 0.873 unpinned, and about 0.92 past the cores; confirmed on held-out files. Below 10 threads the pinned gains (0.929 at 8 threads) did not transfer to the full machine: unpinned they vanish, and competition for the CPU with the decoder's other threads is implicated ([section 1](<#1-what-does-a-worker-do-and-what-do-the-workers-share>)).
- **The counters point to the shared cache, not to the clock or idle cores.** From 10 workers to 7, the cycles per byte fall 25% and the LLC misses per byte 48%, while the instructions and the clock stay put. What exactly the remaining cost is made of is not settled ([section 2](<#2-why-are-fewer-workers-faster>)).
- **The model takes the smallest of a few bounds,** a heuristic fitted to these runs. It names 7 at 10 threads and 6 at 8 only with a tie-break fitted to those two counts: on its own cost curve, 8 workers are 0.7% cheaper than 7. The 8-thread case, measured pinned, implicates CPU competition, so it can't calibrate cache contention alone ([section 3](<#3-a-model-of-what-the-workers-share>)).
- **Its frozen prediction for smaller blocks failed.** At `bzip2 -3` it predicted the best count would rise to 9 or 10; it stayed at 7. Contention did shrink roughly with the footprint, but not as a function of how full the cache is alone ([section 4](<#4-the-test-that-failed-smaller-blocks>)).
- **What shipped is narrower than the model.** mibz2 0.5.4 trims on the test machine only, in two configurations: pinned to exactly one CPU on each of as many cores as threads, at 4, 6, 8 and 10 threads; or with the whole machine available, from 10 threads. Any other affinity mask keeps the old plan. No general selector has shipped ([section 5](<#5-what-shipped-and-what-the-model-still-needs>)).

The scoring of every frozen prediction, the contention hypotheses and the derivations are in the [technical notes](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/index.md>).

## 1\. What does a worker do, and what do the workers share?

Who does what in mibz2's parallel decoder

Conceptual. The thread names are the code's (src/parallel.rs); the kitchen labels are an analogy. In the kitchen, each cook prepares a whole dish at a station, and all of them spread their ingredients on one worktop; more cooks crowd it. The analogy stops at eviction: a cook's ingredients never vanish from a worktop, but a worker's data does leave the cache and has to be fetched again from main memory.

mibz2's parallel decoder ([`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L1-L22>)) has three kinds of thread. The *scanner* reads the compressed stream in order; a block's end is only known once its Huffman-coded symbols have been read, so this scanner reads them one block after another. Each *worker* then finishes one block on its own: the move-to-front step, the inverse Burrows–Wheeler transform, a run-length step and the block's checksum. The *reader* takes the finished blocks back in order and hands out their bytes. The decoder lets up to four blocks a worker be in flight at once, in one window shared by all of them: 28 blocks for seven workers.

Blocks decode independently, so more workers means more blocks at once. But the workers are not independent: they run on cores that share one L3.

The test machine's caches: private L2s, one shared L3

The test machine. The L1 caches, per core and smaller still, are left out. A worker's footprint is about eight times its core's L2, so it lives in the shared L3.

Each core has its own 256 KiB level-2 (L2) cache. Behind them sits the 20 MiB L3, shared by all ten. A lazy-letters worker rebuilding a 900 kB block has a *footprint* of about 2.1 MB: the size of what the kernel stores for the block, an estimate of the memory it reuses while it decodes. It is a proxy for the worker's working set, not a measurement of what it occupies in the LLC. That is far more than the L2, so the footprints meet in the L3.

> [!NOTE]
>
> **Working set, footprint and occupancy**
>
> A *working set* is the data a program reuses over some window of time. A *footprint* is an estimate of its size. *Occupancy* is how much of a shared cache a worker actually holds at a moment, which depends on the other workers. The standard tools for the first are reuse (stack) distances, introduced by [Mattson et al. (1970)](<https://doi.org/10.1147/sj.92.0078>), and the miss-ratio curves built from them; [Qureshi and Patt (2006)](<https://doi.org/10.1109/MICRO.2006.49>) use such curves to divide a shared LLC between programs. A curve from one trace is an approximation for a given replacement policy, not the exact miss count of every cache and schedule. mibz2 hasn't measured one yet.

Workers filling the shared L3

Workers: 7

Drag the slider to change the number of workers; the text below the bar says what the measurements show at that count. Each green block is one worker's footprint, about 2.1 MB for a lazy-letters worker on a 900 kB block, in the test machine's 20 MiB (21.0 MB) L3. This is capacity accounting, not a model of the misses: footprints overlap and change while a block decodes, and the cost rises smoothly as they grow, with no cliff at 100%. The dashed lines are the teams policy's 0.7 of the L3 and the fitted optimum's 0.8.

## 2\. Why are fewer workers faster?

Three explanations suggest themselves. The clock: fewer busy cores may each run faster. Idle time: with more workers than the processors can keep busy, some wait. The cache: each worker slows the others by evicting their data. The hardware counters at 10 threads tell these apart. They count per byte of output, against 10 workers.

Fewer workers, cheaper bytes: the work is the same, the shared-cache misses fall

Measured on the test machine at 10 threads, one copy of the Wikidata extract, 10 repetitions under perf stat, pinned one thread a core. L3 misses are the last-level cache's load misses. The clock (cycles over reference cycles) was 1.326 at every count, so the cycles are not falling because the cores slowed or sped up.

The clock is ruled out first: cycles over reference cycles, which measures the clock against its base rate, is 1.326 at every worker count. The work is the same: instructions per byte fall by under 2% from 10 workers to 7. The private L2 caches see about the same traffic, with L2 misses up about 3%. What changes is the shared cache. From 10 workers to 7, LLC misses per byte fall 48%, and cycles per byte fall 25%. The fall is smooth: an earlier prediction of a cliff, once the footprints fit, was refuted by exactly this shape.

**Lost parallelism against cheaper bytes.** The wall time is the cycles each byte costs, divided by how many processors are working on the bytes. At 10 workers each worker's CPU utilisation is 0.763, so 7.63 processors' worth are working; at 7 workers it is 0.925, 6.48 processors' worth. That is 15% less processing power. The cycles per byte fell 25%. So the wall time is \\(0.752 / 0.849 \\approx 0.886\\) of what it was, against 0.885 measured.

> [!NOTE]
>
> **Why this bookkeeping proves nothing by itself**
>
> When the clock is flat, cycles per byte divided by the CPUs at work equals the wall time almost by definition, so any cause would balance this sum. The evidence for the cache comes from the counters' pattern instead.

The evidence comes in three tiers. **Observed:** with fewer workers, the LLC misses per byte and the cycles per byte both fall, while the instructions, the clock and the private-cache traffic stay about the same. **Supported:** interference for the LLC's capacity contributes materially to the cost. **Under investigation:** what the rest is made of. A counters run has since measured the latency of loads that hit in the L3 falling from about 91 cycles to 65 as workers go from 10 to 7, which fits queueing in the L3 and the ring that links it to the cores; but it missed that hypothesis's predicted size, and traffic to main memory was not measured, so main memory isn't ruled out ([notes, section 2](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/#2-what-the-contention-associated-overhead-is-made-of>)).

> [!NOTE]
>
> **Interference, not (as far as we can tell) coherence**
>
> The workers decode independent blocks and don't write to each other's block data, so the main effect is interference in a shared cache's capacity and the paths to it. They do share a queue, hand-offs and allocator state, so some coherence traffic exists. Nothing in the counters points to it as the explanation, but they don't rule it out either.

## 3\. A model of what the workers share

The model takes the smallest of a few bounds, each counted in workers:

\\\[m = \\min(N,\\ W\_{L3},\\ 0.80\\,N,\\ W\_{\\mathrm{ser}}).\\\]

\\(N\\) is the logical CPUs the process may use; SMT enters through each worker's cost, not through this cap. \\(W\_{L3}\\) is a cache bound, summed over LLC-sharing domains \\(d\\) (one on this machine) as \\(\\sum\_d \\min(N\_d,\\ 0.8 \\times L\_d / F)\\), with \\(N\_d\\) a domain's logical CPUs, \\(L\_d\\) its LLC size and \\(F\\) the footprint: about 7.9 on this machine. \\(0.80\\,N\\) is a CPU bound, and \\(W\_{\\mathrm{ser}}\\) a serial bound, how many workers the scanner can keep fed, added after the small-block test of section 4. This is a resource-constrained heuristic: each bound is estimated separately and the smallest taken, and the resources interact. Predicting performance from shared-cache contention has a literature of its own; [Xu, Chen, Dick and Mao (2010)](<https://doi.org/10.1109/ISPASS.2010.5452065>) is the closest to this approach.

**The cache bound.** At 10 threads, the cycles per byte with \\(W\\) workers fit \\(c(W) = a + b\\,W^2\\) with \\(a \\approx 0.42\\) and \\(b \\approx 0.0068\\). These are normalised costs: the measured cycles per byte at 10 workers, about 23, count as 1. The fit is not forced through that point, and gives \\(c(10) = 0.422 + 0.0068 \\times 100 = 1.102\\). A refined form, \\(c(W) = a + b\\,W\\,W\_{\\mathrm{run}}\\) with \\(W\_{\\mathrm{run}}\\) the workers running at once, removes most of that residual (it gives 0.983); the quadratic is the simpler illustration. \\(a\\) is an extrapolated intercept, and the square is a fit: exponents from 1.5 to 2.5 fit these points about as well. If every worker is kept busy, the wall time goes as \\(a/W + b\\,W\\), smallest at \\(\\sqrt{a/b} \\approx 7.9\\), where the footprints fill about 0.8 of the L3. The 0.8 is a fitted constant, and the bound is a modelled throughput optimum, not a hard ceiling.

**The CPU bound.** A worker with a processor to itself has a CPU utilisation of 0.93; the rest of the time it waits on the pipeline. The scanner and reader use about \\(0.16\\,W\\) processors, and a fully subscribed set of processors leaves about \\(0.125\\,N\\) idle, a residual idle capacity whose cause isn't established. Solving \\(0.93\\,W = N - 0.16\\,W - 0.125\\,N\\) gives \\(0.80\\,N\\).

**One worked example, and the tie-break.** At 8 threads, the bounds give \\(m = \\min(8,\\ 7.9,\\ 6.4) = 6.4\\), so the candidates are 6 and 7. At 10 threads, \\(m = \\min(10,\\ 7.9,\\ 8.0) = 7.9\\), with candidates 7 and 8. The rule chooses between them on the fitted cost, and within 1% it takes the smaller. That tie-break was fitted to these two counts, not predicted: on the cost curve below, 8 workers cost \\(0.10715\\) against 7's \\(0.10789\\), 0.7% cheaper, and at 8 threads the model puts 6 and 7 level. The measured bests are 7 (8 was 1.9% slower) and 6 (7 was 5% slower). A separate, more conservative rule, sizing the footprints to 0.7 of the L3 as mibz2's teams policy does for its tables, also gives 7 at 10 threads; it is a different rule, not the fit rounded.

The fitted cost prefers 8 workers; the measured wall time prefers 7

Dashed: the fitted cost, *a*/*W* + *bW* (modelled), which assumes every worker is kept busy, against its value at 10 workers. Solid: the measured wall time at 10 threads against 10 workers (figure 1). The two needn't agree point by point, since the measured wall also includes the workers' CPU utilisation; the point is that the fit's cheapest whole count is 8, while the run's is 7.

The rule at two thread counts: bounds, the rule's pick and the measured best

Modelled, from measured inputs. The bars are the continuous bounds, in workers; the solid bar is the smallest, and the dashed line marks it. The rule then chooses between the whole numbers either side on the fitted cost, taking the smaller within 1%, a tie-break fitted to these two counts. The measured bests are from figure 1.

The same rule gives 5 workers at 6 threads and 3 at 4, and pinned runs found 5 and 3 best there, on the older (eager) kernel. The 8-thread result was already known when the CPU bound was written. All these points below 10 threads were pinned, which section 5 shows matters more than the model allowed.

**The contention–parallelism trade-off.** At the fit's optimum, the contention-associated overhead \\(b\\,W^2\\) equals the base cost \\(a\\), so it is half the total; for a general exponent \\(p\\) it is \\(1/p\\) of the total. At 7 workers the fit puts it at about 44% of the cost. That is a modelled cost share, not a measurement: it doesn't say how much time any other change could recover, and with other exponents it ranges from about 34% to 60%. The derivation, its sensitivity and what is known about the overhead's make-up are in the [notes](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/#1-the-trade-off-at-the-optimum>).

## 4\. The test that failed: smaller blocks

The test that could fail was the same file compressed with `bzip2 -3`, whose blocks are a third the size. Each worker's footprint shrinks with its block, to about 0.38 of its size at `-9`, so the cache bound moves from about 8 to about 22. Frozen beforehand: the trim disappears, and the best count at 10 threads is 9 or 10, with 7 workers no better than 0.98 of 10 workers' time. The three-bound model of the time agreed, predicting 9 workers, with 7 slower than 10.

Smaller blocks, a smaller gain, the same best count

Measured on the test machine at 10 threads, the process limited to one hardware thread a core, 10 paired rounds a point; bars are 95% intervals over the rounds, on one machine. The `-9` curve is the first 10-thread run (figure 1); the `-3` curve is the same Wikidata extract recompressed with 300 kB blocks. Each point is against 10 workers on the same file.

The best count stayed at 7, so the prediction failed. At 10 threads, 9, 8, 7 and 6 workers took 0.992, 0.987, 0.978 and 0.987 of 10 workers' time, and 4 workers took 1.343. The gain fell from about 12% to 2%, and 7 workers at 0.978 (0.973, 0.983) leave "no better than 0.98" borderline. Part of the model held up. Contention did shrink with the footprint: comparing slopes over the same worker counts at both block sizes, the contention term's exponent in the footprint comes out between 1.85 and 2.23, near the 2 it has in \\(W\\). But a sharper test, whether misses depend only on how full the cache is, failed at 3 workers: they were 34% below its prediction. That suggests contention depends on how many workers run at once as well as on their footprints.

What sets 7 at `-3` is a hypothesis: the scanner. Part of its work, and the reader's, is paid once a block, and a `-3` file has three times as many blocks a byte. An occupancy trace showed the workers idle waiting on the scanner 28% of the time at `-3` against 5% at `-9`, and throughput in the timed runs stayed near 2.5 GB/s from 6 workers to 10. But that trace came from a run that failed its own memory-ledger check and was not pinned like the timed runs, so it suggests the serial bound \\(W\_{\\mathrm{ser}}\\) without establishing it. The bound was added after the data, so it needs its own frozen test.

Every other frozen prediction, met and failed, is scored in the [notes' ledger](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/#3-the-ledger-of-frozen-predictions>). Selected:

| frozen prediction | measured | verdict |
| --- | --- | --- |
| at 10 threads, 7 workers at 0.93 (0.90, 0.98) of 10's time | 7, at 0.885 | count right, gain outside the interval |
| at 6 and 4 threads: 5 and 4 workers, with flat CPU utilisation | pinned: 5, then 3 eager workers at 4 (0.959); utilisation flat near 0.74 | right at 6; overtaken at 4; pinned only |
| 0.7 of the threads, rounded or floored | fails at 6 and 4 threads (rounded) and at all three counts (floored) | refuted |
| at `bzip2 -3`: the best count rises to 9 or 10 | 7 | refuted |
| at `bzip2 -3`: misses depend only on how full the cache is | 34% below at 3 workers | refuted |

## 5\. What shipped, and what the model still needs

**Status on 17 November 2026.** mibz2 0.5.4, released today, trims its workers on the test machine in two modes. Pinned to one logical CPU on each of exactly as many cores as threads, it runs 7 lazy-letters workers at 10 threads, 6 at 8, and 5 and 3 eager workers at 6 and 4 threads. Unpinned, it runs 7 lazy-letters workers from 10 threads, at 0.873 of 10 workers' time, and changes nothing below. It recognises the machine from its present and online CPUs. It is not the model: no general selector has shipped.

The two modes differ because the pinned gains below 10 threads did not transfer to the full machine. Every sweep of 10 threads or fewer ran pinned, so the workers shared their cores with the scanner, the reader and the main thread, and a worker fewer freed a core for them. Unpinned, those threads have CPUs of their own: at 8 threads 6 workers took 1.000 of 8's time, and at 4 threads 3 took 1.214 of 4's. That implicates competition for the CPU, the effect the CPU bound is meant to capture, but it doesn't rule a cache benefit in or out. A net ratio can hide both: a cache saving to 0.96 of the cycles and a parallelism loss of 1.0417 multiply to 1.000. So the pinned 6- and 8-thread points can't calibrate cache contention alone, and they come out of the contention fit. At 10 threads the gain survives unpinned, at 0.873, and the counters support interference in a shared resource, with the exact mechanism unresolved. (0.5.3 had read the machine through the process's affinity set, which a pinned 10-thread run limits to 10 logical CPUs, so its trim fired only unpinned; 0.5.4 fires in both.) Beyond the cores, 0.5.3's release run measured about 6.5% less wall time than 0.5.2 at 12 to 20 threads, with about 25% less processor time and 22% lower peak memory. 0.5.4 admits only the two configurations above: a 10-CPU mask spread over five cores' hardware threads, say, gets the old plan. Whether a general model should read topology machine-wide or through the affinity set is still open.

Two LLC-sharing domains: the limit is summed per domain

Conceptual: a processor with two LLC-sharing domains, as on many AMD chips (one per core complex, or CCX) or a two-socket server. F is a worker's footprint. The cache bound is the sum of each domain's bound, each capped by its logical CPUs, provided the workers are spread across the domains.

The same question arises on processors with several LLC-sharing domains: many AMD chips have one per core complex (CCX), and a two-socket server at least one per socket. Workers compete for an LLC only within their domain, though they still share main memory, so the cache bound sums over the domains the process may use, each capped by its own logical CPUs \\(N\_d\\). The current [`l3_bytes()`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L2831-L2846>) reads one file, the first processor's `cache/index3/size`. The planned fix finds the cache whose level is 3, groups the CPUs in the affinity set by the LLC they share (`shared_cpu_list`), and gives each worker a soft affinity to a domain. On a free GitHub runner with 2 cores, 4 logical CPUs and one 32 MiB L3, the rule gives 3.2 and so 3 or 4 workers; those runs are pending, and the [notes](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/#5-topology-and-transfer>) list the three results that would refute the model there.

The design is to detect once, decide once and then decode: read the logical CPUs, the LLC-sharing domains and the block size at startup, pick the count, and keep it, without tuning mid-stream, since streams are often short, timings noisy, and output timing should stay repeatable.

The chain, in short: a sweep found that fewer workers decode faster on the test machine; the counters pointed to the shared cache; a model of the cache, the processors and the scanner reproduces the counts it was fitted to, with a fitted tie-break; its first test at a new block size refuted its best count while its contention term roughly held; and an unpinned run showed that the gains below 10 threads don't transfer beyond pinning, where CPU competition is implicated, while the 10-thread gain survives, with its exact mechanism unresolved. A measured count describes the machine and the affinity it was measured with. A modelled count can be checked anywhere, and has to be.

---

**Research records:**

- [The calibration record: the sweep and the model](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/docs/journal/2026-11-17-worker-count-calibration.md>)
- Verdicts: [8 against 9 workers past the cores](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6931719814>), [7 against 8 past the cores, with counters](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6942020517>), [within the cores, with counters and the clock](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943194948>), [the confirming run with held-out files](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6944271342>), [the low-thread sweep](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945639925>), [the `bzip2 -3` run](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945320222>)
- Release: [0.5.3](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6946439743>) and [its scaling run](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6946756938>); [the unpinned run](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6949008682>); [0.5.4](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6952096740>)
- The model's records: [the rule as a spec](<https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-worker-rule.md>), [its scope](<https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-rule-scope.md>), [the `-3` scoring](<https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-level3-scoring.md>), [the miss-cost run scored](<https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md>)
- The [technical notes](<https://lmmx.github.io/mibz2/pr-preview/pr-705/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/index.md>), with every other record
