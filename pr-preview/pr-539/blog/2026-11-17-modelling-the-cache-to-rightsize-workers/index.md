[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-539/blog/index.md>)

# Modelling the cache to rightsize workers

Rather than re-measure every worker count whenever the decoder changes, mibz2 sizes its workers from a model of the cache and cores they share.

17 November 2026 · 24 min read · [Research in progress](<#6-what-changes-for-users-and-what-comes-next>)

mibz2 decodes a bzip2 file by handing its blocks to *workers*: threads that each decode a whole block at a time. How many workers to start is a choice mibz2 makes once, at startup, and the obvious answer, one per thread the caller asks for, is wrong. On our 10-core test machine, running fewer workers than threads takes up to 11.5% less wall time, or about 13% more throughput. We found that by measurement alone, timing every worker count at each thread count. A sweep like that has two faults. It must be redone every time the decoder changes, because each change to the algorithm changes how much memory a worker uses, and with it the best count. And it describes one machine: a policy measured on one processor's 20 MiB cache is "works on my machine" for everyone else. So we are working to replace the measured policy with a model of what the workers share: the processor's shared cache, the processor time left over after the decoder's other threads, and the cores. When the algorithm changes, we would update the model's inputs, such as a worker's memory footprint, instead of re-running the sweep. On another machine, the model would read that machine's cache and cores.

The model is a proposal under test, not yet the decoder's policy. Its predictions are written down and frozen before each new experiment (pre-registered: fixed before the data, and tested out of sample), so it can be shown wrong in public, and it has been both right and wrong already. It named the best count at three thread counts it hadn't seen. At a smaller block size it named the wrong one, which exposed a limit it lacked.

**What the sweep found.** The machine is an Intel i9-10900K: 10 cores, 20 hardware threads and a 20 MiB level-3 (L3) cache that all the cores share. On this chip the L3 is the last-level cache (LLC), so the two names mean the same cache here; the post says LLC in general statements and L3 for the i9. The test file is four copies of an 11.9 MB extract of a Wikidata dump, decoded in 10 paired rounds against mibz2's current plan at the same thread count. Every worker in these runs uses the [lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-539/blog/2026-11-16-lazy-letters/index.md>) kernel. Below, *threads* means the number the caller asks for, *workers* the number mibz2 runs, and *cores* the physical cores; each core has two hardware threads, and each run's threads were limited to one hardware thread a core where the threads fit.

Fewer workers, less time: the i9 at 10 and 8 threads

Measured on an Intel i9-10900K, decoding four copies of an 11.9 MB Wikidata extract, 10 paired rounds a point, with the process limited to one hardware thread a core. Each point is the wall time against the plan of one worker a thread at the same thread count (10 workers at 10 threads, 8 at 8); the bars are 95% intervals over the rounds, on one machine. The hollow point (6 workers at 10 threads) was run as context and not judged. 5 workers at 8 threads, run later in the low-thread sweep, took 0.997 of 8's time: no gain.

At 10 threads, today's mibz2 runs 10 workers. Seven, with 28 blocks in flight, take 0.885 of the time, 11.5% less, with a quarter less peak memory. At 8 threads, six workers take 0.929 of the time of eight. Beyond the physical-core count (12 to 20 threads, the SMT regime, where a core runs two threads), today's mibz2 already runs one worker fewer than the cores, nine; past the cores, seven take 0.921, 0.930 and 0.923 of its time at 12, 16 and 20 threads. A second run then tested these choices directly, beside two files that played no part in choosing them: binary records, and text compressed with 100 kB blocks. It confirmed all three: 0.881 at 10 threads, 0.933 at 8, and 0.921 to 0.930 past the cores, with 17% to 27% less peak memory. No cell on either held-out file was judged slower. On the small-block text every cell was inconclusive, which is not the same as no difference.

## What was found

The takeaways, each shown in a section below:

- **Fewer workers than threads is faster on the i9, at every count measured from 6 threads up.** At 10 threads, 7 workers take 0.885 of the wall time (11.5% less) with a quarter less memory. At 8 threads, 6 workers take 0.929. Past the cores, 7 workers take 0.92 to 0.93 of 9's time. A confirming run, with files held out from the choice, reproduced all three. Below that, 5 workers take 0.963 of today's time at 6 threads, and at 4 threads nothing beats today's four ([section 1](<#1-what-does-a-worker-do-and-what-do-the-workers-share>)).
- **The counters point to contention for the shared cache, not to the clock or idle cores.** The clock speed is the same at every count. Going from 10 workers to 7, the processor cycles spent per byte fall by 25%, and the misses in the shared cache per byte fall by 48%, while the instructions per byte barely change. Which part of that contention costs the time is not yet known: a kernel with a smaller footprint cut the misses again at 7 workers and saved no measurable time ([section 2](<#2-why-are-fewer-workers-faster>), [section 3](<#3-a-model-of-three-limits>)).
- **The model takes the smallest of three bounds:** \\(W^\* \\approx \\min(\\text{cores},\\ \\text{cache limit},\\ \\text{CPU budget})\\), a resource-constrained heuristic fitted to these runs. At 8 threads the CPU budget, about 6.6 workers, binds before the cache limit, about 8, which matches 6 workers beating 7 there, though the model puts the two nearly level where 7 measured 5% slower. At 10 threads the fit's own cost curve slightly prefers 8 to the measured 7 ([section 3](<#3-a-model-of-three-limits>)).
- **The model must predict new evidence, not only fit old.** Its predictions for 4, 6 and 8 threads, and for files compressed with a smaller block size (`bzip2 -3`), were frozen before those runs. At 4, 6 and 8 threads it named the best count every time, and a fixed fraction of the threads failed. At `-3` its predicted best count, 9 or 10, failed: the best stayed at 7. Contention did shrink with the footprint as the model required, and the leading explanation for the rest, a limit set by the serial scanner, is a hypothesis now under test ([section 4](<#4-how-could-the-model-be-wrong>)).
- **Production changes only where it was measured and reviewed.** The change that has merged is scoped to the measured i9 profile, not the model. Everywhere else keeps today's behaviour until the model passes its transfer tests. The design is to detect the machine once, decide once, and not adapt mid-stream ([section 6](<#6-what-changes-for-users-and-what-comes-next>)).

## 1\. What does a worker do, and what do the workers share?

Who does what in mibz2's parallel decoder

Conceptual. The thread names are the code's (src/parallel.rs); the kitchen labels are an analogy. In the kitchen, each cook prepares a whole dish at a station, and all of them spread their ingredients on one worktop; more cooks crowd it. The analogy stops at eviction: a cook's ingredients never vanish from a worktop, but a worker's data does leave the cache and has to be fetched again from main memory.

mibz2's parallel decoder ([`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L1-L22>)) has three kinds of thread. The *scanner* reads the compressed stream in order. A block's end is only known once its Huffman-coded symbols have been read, so this scanner reads them one block after another. Each *worker* then finishes one block on its own: the move-to-front step, the inverse Burrows–Wheeler transform, a run-length step and the block's checksum. The *reader* takes the finished blocks back in order and hands out their bytes. Each worker finishes one block at a time. The decoder lets up to four blocks a worker be in flight at once, in one window shared by all of them: 28 blocks for seven workers.

Blocks decode independently, so more workers means more blocks at once. But the workers are not independent. They run on cores that share one L3 cache.

The i9's caches: private L2s, one shared L3

The Intel i9-10900K the measurements were made on. The L1 caches, per core and smaller still, are left out. A worker's footprint is about eight times its core's L2, so it lives in the shared L3.

Each core has its own small level-2 (L2) cache, 256 KiB. Behind them sits the 20 MiB L3, shared by all ten. Data a worker needs that isn't in either comes from main memory, which takes several times longer. A lazy-letters worker rebuilding a 900 kB block has a *footprint* of about 2.1 MB: an estimate of the memory it reuses while it decodes, used as a proxy for its working set, not a measurement of what it occupies in the LLC. That is far more than the L2, so the footprints meet in the L3.

> [!NOTE]
>
> **Working set, footprint and occupancy**
>
> Three measures are easy to conflate. A *working set* is the data a program reuses over some window of time. A *footprint* is an estimate of its size; here, 2.1 MB a worker. *Occupancy* is how much of the shared cache a worker actually holds at a moment, which depends on what the other workers are doing. The standard tools for the first are reuse (stack) distances, introduced by [Mattson et al. (1970)](<https://doi.org/10.1147/sj.92.0078>), and the miss-ratio curves built from them, which give the misses at every cache size in one pass; [Qureshi and Patt (2006)](<https://doi.org/10.1109/MICRO.2006.49>) use such curves to divide a shared LLC between programs. mibz2 hasn't measured either yet, so its footprint is a proxy.

Workers filling the shared L3

Workers: 7

Drag the slider to change the number of workers; the text below the bar says what the measurements show at that count. Each green block is one worker's footprint, about 2.1 MB for a lazy-letters worker on a 900 kB block, in the i9's 20 MiB (21.0 MB) L3. This is capacity accounting, not a model of the misses: footprints overlap and change while a block decodes, and the cost rises smoothly as they grow, with no cliff at 100%. The dashed lines are the teams policy's 0.7 of the L3 and the fitted optimum's 0.8.

## 2\. Why are fewer workers faster?

There are three explanations a reader might reach for. One is the clock: a processor running fewer cores may run each faster. The other is idle time: with more workers than the processor can keep busy, some wait. A third is the cache: each worker slows the others by evicting their data. The hardware counters at 10 threads tell these apart. They count, per byte of output, against 10 workers.

Fewer workers, cheaper bytes: the work is the same, the shared-cache misses fall

Measured on the i9 at 10 threads, one copy of the Wikidata extract, 10 repetitions under perf stat, pinned one thread a core. L3 misses are the last-level cache's load misses. The clock (cycles over reference cycles) was 1.326 at every count, so the cycles are not falling because the cores slowed or sped up.

The clock is ruled out first. Cycles over reference cycles, which measures the clock against its base rate, is 1.326 at every worker count. The work is the same: instructions per byte fall by under 2% from 10 workers to 7. The private L2 caches see about the same traffic, with L2 misses up by about 3%. What changes is the shared cache. From 10 workers to 7, misses in the last-level cache (the L3) per byte fall by 48%, and the cycles per byte fall by 25%. Each worker gets cheaper as fewer share the L3.

The fall is smooth, with no cliff. Each worker removed makes the others' misses a little rarer. An earlier prediction that the cost would fall off a cliff once the footprints fit was refuted by exactly this shape.

**The wall time, as bookkeeping.** The wall time is the cycles each byte costs, divided by how many processors are working on the bytes. At 10 workers each worker's CPU utilisation is 0.763, so 7.63 processors' worth are working; at 7 workers it is 0.925, 6.48 processors' worth. That is 15% less processing power. The cycles per byte fell 25%. So the wall time is \\(0.752 / 0.849 \\approx 0.886\\) of what it was, against 0.885 measured. Fewer workers lose some parallelism and gain more back in cheaper bytes, up to a point.

> [!NOTE]
>
> **Why the bookkeeping proves nothing by itself**
>
> When the clock is flat, cycles per byte divided by the CPUs at work equals the wall time almost by definition, so any cause would balance this sum. The evidence for the cache comes from the counters' pattern instead.

The evidence comes in three tiers. **Observed:** with fewer workers, the LLC misses per byte and the cycles per byte both fall, while the instructions, the clock and the private-cache traffic stay about the same. **Supported:** interference for the LLC's capacity contributes materially to the cost. **Under investigation:** how much of the rest is the latency of LLC hits and queueing on the paths to the LLC, and how much is main memory; section 3 sets out the two hypotheses, and the counters run that decides between them is pending.

> [!NOTE]
>
> **Interference, not coherence**
>
> The workers decode independent blocks and don't write to each other's data, so this is interference in a shared cache's capacity and bandwidth, not cache-coherence traffic or false sharing. Nothing in the counters suggests coherence effects.

## 3\. A model of three limits

The model's answer is \\(W^\* \\approx \\min(C,\\ W\_{L3},\\ W\_{\\mathrm{CPU}})\\): the cores available \\(C\\), the cache limit \\(W\_{L3}\\) and the CPU budget \\(W\_{\\mathrm{CPU}}\\), each counted in workers. The smallest binds, since plenty of one resource doesn't make up for running out of another. It is a resource-constrained heuristic: each bound is estimated separately and the minimum taken, not the solution of one joint optimisation, and the resources can interact. Predicting performance from shared-cache contention has a literature of its own; [Xu, Chen, Dick and Mao (2010)](<https://doi.org/10.1109/ISPASS.2010.5452065>) is the closest to this approach. This simple form is for workers that each have a core to themselves. In the SMT regime, the full model doesn't cap the count at \\(C\\): it counts a core running two workers as \\(\\sigma\\) cores' worth, with \\(\\sigma\\) fitted between 1.24 and 1.38 on the CI runners, and scores each count by that yield. That is how it can predict 4 workers on a runner with 2 cores and 4 hardware threads. A fourth limit, how fast the scanner can feed the workers, is the leading hypothesis after the `bzip2 -3` run ([section 4](<#what-the-bzip2-3-run-showed>)); if it holds, it doesn't bind at the default block size.

**The cache limit,** a cache-derived concurrency bound. At 10 threads, the cycles per byte with \\(W\\) workers fit \\(c(W) = a + b\\,W^2\\). The costs are normalised, so that \\(c(10) = 1\\) stands for 10 workers' cycles per byte (about 23 cycles a byte counted); \\(a \\approx 0.42\\) is the fit's intercept, an extrapolated cost with no contention, and \\(b\\,W^2\\) the contention (\\(b \\approx 0.0068\\)). The square is a fitted approximation, not a derivation: exponents from 1.5 to 2.5 fit these points about as well. If every worker is kept busy, the wall time goes as \\(c(W)/W = a/W + b\\,W\\), smallest at \\(W = \\sqrt{a/b} \\approx 7.9\\). At 2.1 MB a worker, 7.9 footprints fill about 0.8 of the 20 MiB L3. The fits with other exponents put the peak in the same place, 0.77 to 0.84 of the L3, so with \\(L\_3\\) the L3's size and \\(F\\) a worker's footprint, the rule is \\(W\_{L3} \\approx 0.8 \\times L\_3 / F\\). The 0.8 is a fitted constant, not a law, and the limit is a modelled throughput optimum, not a hard ceiling: past it, extra workers still run, but they cost more than they add.

**7.9 does not round down to 7.** Under this same fit, 8 workers cost \\(0.422/8 + 0.0068 \\times 8 = 0.10715\\) and 7 cost 0.10789, so the fit prefers 8, by 0.7%. Seven is the *measured* best. A separate, more conservative rule, which sizes the footprints to 0.7 of the L3 as mibz2's teams policy does for its tables, also gives 7, and it is the simple rule frozen before the next runs. That agreement is between two rules, not a derivation from the fit.

The fitted cost prefers 8 workers; the measured wall time prefers 7

Dashed: the fitted cost, *a*/*W* + *bW* (modelled), which assumes every worker is kept busy, against its value at 10 workers. Solid: the measured wall time at 10 threads against 10 workers (figure 1). The two needn't agree point by point, since the measured wall also includes the workers' CPU utilisation; the point is that the fit's cheapest whole count is 8, while the run's is 7.

**The CPU budget,** a CPU-capacity bound. A worker with a processor to itself has a CPU utilisation of 0.93; the rest of the time it waits on the rest of the pipeline. The scanner and reader use about one processor (1.08 counted at 10 threads). And a fully subscribed set of processors leaves some idle, about 1.25 at 10 threads, a residual idle capacity whose cause isn't established. So the workers can use at most \\(N - \\nu - \\iota\\) processors, with \\(N\\) the logical CPUs in the process's affinity set, \\(\\nu\\) the non-worker threads' use and \\(\\iota\\) the idle capacity, and the budget in workers is \\(W\_{\\mathrm{CPU}} \\approx (N - \\nu - \\iota) / 0.93\\). This reproduces the measured per-worker CPU utilisation at 10 threads: 0.767, 0.851, 0.930, 0.930 and 0.930 for 10 down to 6 workers, against 0.763, 0.857, 0.931, 0.925 and 0.931 measured.

> [!NOTE]
>
> **A correction to this term**
>
> An earlier version of this term was derived from the wall times and the cycles, and the review of the runs pointed out that it was close to the identity of section 2, so its match was not independent evidence. The term is now built from each thread's counted processor time. That makes it a separate measurement, but it is still calibrated to these runs, not an independent test.

The model at two thread counts: the smallest bound binds

Modelled, from measured inputs. The bars are in workers, with the arithmetic in the text below; the solid bar is the one that binds, and the dashed line marks it. The non-worker load at 10 threads (1.08 processors) was counted; at 8 threads (0.85) it is the model's estimate, scaled with throughput. The idle capacity is 0.125 of the logical CPUs in the affinity set. The limits are continuous. The measured best counts are 7 and 6; taking the whole number below a limit is a convention, not something the fit implies (see the cost curve above).

**At 10 threads,** the budget is \\((10 - 1.08 - 1.25) / 0.93 \\approx 8.2\\) workers and the cache limit is 7.9, so the cache limit is the smaller; the measured best is 7, while the fit alone would slightly prefer 8. **At 8 threads,** the budget is \\((8 - 0.85 - 1.0) / 0.93 \\approx 6.6\\), below the cache limit, so the budget binds: 6 workers, the measured best. A seventh would get only part of a processor while adding a whole footprint to the cache. **Past the cores,** with 20 hardware threads allowed and each counted as one processor, the budget is about 17, so the cache limit is the smaller again, and 7 is again the measured best.

The model misses one thing at 8 threads. It puts 7 workers level with 6, but 7 measured 5% slower (0.975 against 0.929). And fitting the data it was built from is no proof.

### The contention–parallelism trade-off at the optimum

The fit says more than where the peak is. With \\(c(W) = a + b\\,W^2\\), the wall time goes as \\(a/W + b\\,W\\): the first term falls with more workers and the second rises, and their sum is smallest at \\(W^\* = \\sqrt{a/b}\\), exactly where \\(b\\,W^2 = a\\). So at the fit's optimum, the contention-associated overhead equals the base cost: half the total. It is a fitted term, not counted stall cycles. With a general exponent, \\(c(W) = a + b\\,W^p\\), the optimum falls where \\(a = (p - 1)\\,b\\,W^p\\), and the overhead's share there is \\(1/p\\): half at \\(p = 2\\), two thirds at 1.5, 40% at 2.5.

On the i9 at 7 workers, the fit puts the contention-associated overhead at \\(b\\,W^2 = 0.0068 \\times 49 \\approx 0.33\\) of \\(c(7) \\approx 0.76\\), about 44% of the cost: a little under half, because 7 is below the fit's optimum of 7.9. These are the model's cost shares, not a measurement. A large overhead share is what the fit expects at a well-chosen count, and by itself it shows neither that the cost is avoidable nor that it isn't. The same numbers size the prize for removing contention some other way than by cutting workers: in the model, 7 workers would need \\(a \\approx 0.42\\) instead of 0.76, a cost ratio of about 1.8. That would become a 1.8-times speedup only if contention could be removed with nothing else changing, which nothing has shown.

The 44% is soft. The intercept \\(a\\) is extrapolated from fits over 6 to 9 workers. And the exponent isn't pinned: fits with \\(W^{1.5}\\) to \\(W^{2.5}\\) in place of \\(W^2\\) match the data about as well, and move the share at 7 workers from about 60% to about 34%, and the cost ratio from 2.5 to 1.5. That range is the sensitivity to the model's form, not a confidence interval. The low-thread sweep, with 2 to 5 workers, measured the cost close to one worker alone; the fit of \\(a\\) to those points is still to be reported.

> [!NOTE]
>
> **A familiar curve, a different cause**
>
> Throughput proportional to \\(W / (a + b\\,W^2)\\) has the shape of [Gunther's Universal Scalability Law](<https://arxiv.org/abs/0808.1431>): a rational function that rises, peaks and then falls (retrograde scaling). The resemblance is in the shape only. In the USL the quadratic term models coherency between processors; here it is fitted to interference in a shared cache, so this is not the USL.

**What the contention-associated overhead is made of is not yet known.** A variant of the lazy-letters kernel that stores its rules in 12 bytes instead of 16 shrinks each worker's footprint. With 9 workers past the cores, it took 0.98 to 0.99 of the wall time. With 7 it cut the shared-cache misses per byte to 0.926 at 12 threads, with 1.6% more instructions, and no change in the wall time was detected: 1.012 (0.998, 1.028), 0.996 (0.987, 1.005) and 1.002 (0.985, 1.018) of the 16-byte kernel's at 12, 16 and 20 threads, so it was not adopted. That is consistent with the saved misses having overlapped with other work and been offset by the extra instructions. It doesn't identify what the rest of the contention is.

Across the 10-thread arms, the cycles saved per shared-cache miss avoided come to about 589 from 10 workers to 7. That is a slope across arms, not a cost per miss: it also moves with the cost of the misses that remain, with overlap, and with every other cost that changes with the count. Set against an *assumed* 80 to 100 ns for an uncontended trip to main memory, 390 to 490 cycles, it suggests that fewer misses explain at most two thirds to four fifths of the saving, and less if misses overlap, as the kernel is built to make them. The model's [frozen predictions for a counters run](<https://github.com/lmmx/mibz2/blob/0e7c6888bcba2ce8dd3056c587a23be958d6de4f/docs/journal/monodromy/2026-11-17-contention-counters.md>) set out two hypotheses for the rest: queueing in the L3 and the ring that links it to the cores (the leading one), and main-memory latency rising with load.

|  | measured | L3 queueing (leading) | memory latency |
| --- | --- | --- | --- |
| LLC misses a byte, 10 → 7 | fall to 0.52 | fall, but the smaller part of the cost | fall, and the larger part |
| L3-hit load latency | not yet measured | falls 10% to 25% (5 to 10 cycles) | flat within 5% |
| memory traffic, 10 workers | not yet measured | 4 to 9 GB/s, 10% to 20% of peak | 60% of peak or more |
| L3-miss stalls, share of saving | not yet measured | at most 45% | at least 60% |
| 12-byte rules, 7 workers | misses 0.926, no time change detected | fits: L3 accesses a byte unchanged | needs the saved misses to be overlapped |

The stall row follows the counter-based, top-down attribution of [Yasin (2014)](<https://doi.org/10.1109/ISPASS.2014.6844459>). Under the leading hypothesis, the cost is the 0.69 L3 accesses each byte makes (counted as L2 misses, a proxy), 35 times as many as the misses, each 4 to 8 cycles dearer as more workers queue. The traffic figures are predictions, and hardware prefetch traffic is unknown, so main memory isn't ruled out. If the hypothesis holds, two kinds of change could win back some of the modelled overhead without cutting workers: fewer L3 accesses a byte, from a hot set that fits in the 256 KiB L2, and fewer accesses in flight at once, by staggering the workers' access-heavy phases. The counters run that decides between them is pending.

## 4\. How could the model be wrong?

Every prediction is frozen before its run and scored after it. Here they all are, beside what was measured:

| frozen prediction | measured | verdict |
| --- | --- | --- |
| a capacity *cliff*: misses flatten from 7 workers to 6 | misses fell a further 28% | refuted |
| ring or bandwidth contention with flat misses | misses fell 41% from 8 workers to 6 | refuted (this rival only) |
| at 10 threads, 7 workers at 0.93 (0.90, 0.98) of 10's time | 7 workers, at 0.885 | count right, gain outside the interval |
| at 8, 6 and 4 threads: 6, 5 and 4 workers | 6, 5, and today's 4 | right at all three |
| a rival idle term: 3 workers at 4 threads; falling per-worker utilisation | today's 4; utilisation flat at about 0.74 | refuted |
| 0.7 of the threads, rounded: 6, 4 and 3 workers | 6, 5 and today's 4 | fails at 6 and 4 threads |
| at `bzip2 -3`, 10 threads: the best count is 9 or 10 | 7 | refuted |
| at `bzip2 -3`: 7 workers no better than 0.98 of 10's time | 0.978 (0.973, 0.983) | borderline |
| at `bzip2 -3`: 7 workers at about 0.885, if bandwidth-bound | 0.978 | refuted |
| at `bzip2 -3`, the three-limit model: 9 workers, 7 at about 1.18 of 10's time | 7 best, at 0.978 | refuted |
| the free CI runners: 4 workers, and today's plan stays | not yet run | pending |

The ring and bandwidth rival refuted here is one specific form, in which misses stayed flat; other ring or bandwidth mechanisms are not ruled out, and one of them is the leading hypothesis of section 3.

### The low-thread sweep

The predictions frozen for 4, 6 and 8 threads, beside the counts measured best:

| threads | model | model, rival idle term | simple rule (\\(0.7 \\times\\) threads) | measured best |
| ---: | ---: | ---: | ---: | --- |
| 8 | 6 | 6 | 6 | 6 |
| 6 | 5 | 5 | 4 | 5 |
| 4 | 4 | 3 | 3 | 4 (today's plan) |

The rival keeps the idle capacity \\(\\iota\\) fixed at 1.25 processors rather than in proportion to \\(N\\) (the model takes \\(\\iota = 0.125\\,N\\)). The two disagree at 4 threads, and the counters can tell them apart without a further timing experiment: the per-worker CPU utilisation with as many workers as threads stays about 0.76 under the model, against 0.56, 0.67 and 0.74 at 4, 6 and 8 threads under the rival.

The model passes at a thread count if it is within one worker of the best measured, and its choice costs at most 2% against the best: \\(\\mathrm{wall}(W\_{\\text{predicted}}) / \\mathrm{wall}(W\_{\\text{best}}) - 1 \\le 2\\%\\). The simple rule was frozen with the 2% penalty alone. Neither is an adoption rule.

Below 8 threads: 5 workers win at 6 threads, and nothing beats today's 4 at 4

Measured on the i9, decoding four copies of the Wikidata extract, 10 paired rounds a point, the process limited to one hardware thread a core; the bar is the 95% interval over the rounds. At 6 and 4 threads today's plan runs the older (eager) kernel, one worker a thread, so the 6-worker point at 6 threads and the 4-worker point at 4 threads measure the lazy-letters kernel alone. The 2-worker point at 4 threads, 1.533, is off the chart.

The baselines differ by count. At 8 threads today's plan is 8 lazy-letters workers: 6 took 0.925 (0.918, 0.933) of its time, a third confirmation of that count, and 5 gained nothing (0.997). At 6 and 4 threads today's plan is the older eager kernel, one worker a thread. At 6 threads, 5 lazy-letters workers took 0.963 (0.948, 0.977) of its time, even though the lazy-letters kernel alone costs 4% there (6 lazy workers took 1.040). At 4 threads, nothing beat it: 4, 3 and 2 lazy workers took 1.098, 1.113 and 1.533, so today's eager 4 stays best. The model named 6, 5 and 4 workers, which match the best lazy counts at all three, and at 4 threads it also predicted that today's plan stays. Its frozen ratio for 4 lazy workers against today's plan, 1.09 (1.04, 1.13), came out at 1.098. The per-worker CPU utilisation stayed flat, at 0.742, 0.725 and 0.740 at 8, 6 and 4 threads, as the model predicted; the rival's falling utilisation, and its 3 workers at 4 threads, are refuted.

**A fixed fraction of the threads is refuted.** The simple rule, 0.7 of the threads rounded, picks 4 workers at 6 threads and 3 at 4, and each choice costs about 11% to 12% against the best: well past the 2% allowed. Rounding down instead fails at all three counts. The rule that survives these counts is the model's smallest of limits; the 8- and 6-thread counts now await their review before any production change.

### What the `bzip2 -3` run showed

This was the test that could fail. The same file compressed with `bzip2 -3` has blocks a third the size, so if a footprint shrinks with its block, the cache limit moves from about 8 to over 20. Frozen beforehand: the trim disappears, the best count at 10 threads is 9 or 10, and 7 workers are no better than 0.98 of 10's time; if contention were set by memory bandwidth instead, 7 would still win at about 0.885. It failed, in part. Recompressed at `-3`, each worker's buffers shrank to 0.355 of their size at `-9` (5.97 MB against 16.81 MB). At 10 threads, against 10 workers, 9, 8, 7 and 6 workers took 0.992, 0.987, 0.978 and 0.987 of the wall time, and 4 workers took 1.343, slower.

Smaller blocks, a smaller gain, the same best count

Measured on the i9 at 10 threads, the process limited to one hardware thread a core, 10 paired rounds a point; bars are 95% intervals over the rounds, on one machine. The `-9` curve is the first 10-thread run (figure 1); the `-3` curve is the same Wikidata extract recompressed with 300 kB blocks. Each point is against 10 workers on the same file.

The best count is still 7, so the predicted 9 or 10 is refuted. The gain from trimming fell from about 12% to 2%, and 0.978 (0.973, 0.983) leaves "no better than 0.98" borderline, since the interval contains 0.98. The bandwidth rival's 0.885 is refuted. The three-limit model of section 3, applied as frozen, put the cache limit near 22 and the CPU budget near 8.8, so 9 workers, with 7 slower than 10, about 1.18: refuted too, wrong in direction. At 10 workers, 2.6 of the 10 processors sat idle, so no CPU budget was binding. Per byte at 7 workers, the cycles fell to 0.935 of 10 workers' and the shared-cache misses to 0.635, with the clock flat at 1.326.

**What held: contention shrank with the footprint.** If contention depends only on how full the L3 is, the contention term \\(b\\) should shrink with the footprint as \\(F^k\\), with \\(k\\) the same exponent that \\(W\\) has, about 2. Comparing the slopes of the cycles and of the misses per byte over the same range of workers at both block sizes gives \\(k\\) between 1.7 and 2.1. The contention term's scaling with the footprint held.

**What failed, and a hypothesis for why.** The best count is 7 at both sizes, but the model's limits can't be the reason at `-3`. The leading hypothesis is that the scanner binds there. Part of its work, and the reader's, is paid once a block, and a `-3` file has three times as many blocks a byte, so that part weighs three times as much. An occupancy trace with 10 workers showed them idle waiting on the scanner 28% of the time at `-3`, against 5% at `-9`, and the timed runs' throughput stayed at about 2.5 GB/s from 6 workers to 10. If the scanner binds, an extra worker past that point adds only contention, to its own cycles and to the scanner's, which would be the 2% that 7 workers save.

**A revised form** adds that limit:

\\\[W^\* \\approx \\min(C,\\ W\_{L3},\\ W\_{\\mathrm{CPU}},\\ W\_{\\mathrm{ser}}),\\\]

where \\(W\_{\\mathrm{ser}}\\) is the fewest workers that keep up with the scanner: about 7 at `-3` on the i9, between 6.5 and 7.5. It needs one more input, the scanner's cost per block, a constant of the kernel measured once. Block size then enters the model twice, through the footprint in \\(W\_{L3}\\) and through the scanner in \\(W\_{\\mathrm{ser}}\\). This is a new model, written after the data, so it needs its own frozen test; the frozen scores above stand.

Two caveats. The idle shares come from an occupancy trace whose run failed its own memory-ledger check and was not limited to one hardware thread a core, unlike the timed runs, so they suggest the scanner limit but don't establish it; the throughput plateau and the 2.6 idle processors in the timed runs are consistent with it. And the 0.355 is the ratio of the workers' total buffers, not of a measured hot set; if the hot set shrank by a different factor, \\(k\\) moves with it. A sharper test of whether contention depends only on how full the L3 is was frozen before the low-thread sweep. The `-3` result changes no production plan.

### The free CI runners

The free GitHub x86 runners sampled so far (AMD EPYC 7763, 9V74 and 9V45) report 4 hardware threads on 2 cores and a 32 MiB L3. There the cache limit is about 12.8 and never binds, so the model predicts, through its SMT yield, that 4 workers stay best (or 3 within 2%), and that today's plan stays. With only 2 to 4 workers possible, "within one worker" fails there only on an outcome the existing data already rule out, so the runners are judged on three refuters instead: the model is wrong if 3 workers beat 4 by more than 2%, if 2 come within 3% of 4, or if any lazy-letters plan beats today's by more than 2%. These runs are pending.

> [!NOTE]
>
> **Results pending**
>
> Still to come: the free CI runners; the counters run that decides between the contention hypotheses of section 3; and the review of the 8- and 6-thread counts, before any production change there.

## 5\. What changes on other machines?

Two LLC-sharing domains: the limit is summed per domain

Conceptual: a processor with two LLC-sharing domains, as on many AMD chips (one per core complex, or CCX) or a two-socket server. F is a worker's footprint. The cache limit is the sum of each domain's limit, provided the workers are spread across the domains.

Many processors have more than one LLC. Each is shared by a group of cores, an *LLC-sharing domain*: on many AMD chips, a core complex (CCX); on a two-socket server, at least one per socket. Workers compete for an LLC only within their domain, though they may still share main memory and the paths to it. So the cache limit becomes a sum over the domains the process may use: \\(W\_{L3} = \\sum\_d \\min(C\_d,\\ 0.8 \\times L\_d / F)\\), over each domain \\(d\\) with \\(C\_d\\) cores and an LLC of \\(L\_d\\).

Today's [`l3_bytes()`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L2831-L2846>) reads one file, the first processor's `cache/index3/size`. It assumes the cache numbered 3 is the L3, rather than finding the one whose level is 3, and it reads one domain. The planned fix groups the processors the process may use by the L3 they share (`shared_cpu_list`) and reads each one's size. The sum holds only if workers are spread across the domains, so each worker gets a soft affinity to a domain, not a processor. The CPU budget stays one number for the process.

## 6\. What changes for users, and what comes next

The design is to detect once, decide once and then decode. At startup the decoder would read the cores, the LLC-sharing domains and the logical CPUs in its affinity set, and the block size from the stream's header. It would then pick the count and keep it, without tuning itself mid-stream: streams are often short, timings are noisy, and the output's timing would stop being repeatable.

Production changes only where the change was measured and reviewed. On the i9's own profile, seven workers past the cores and at 10 threads passed their review. At 8 and 6 threads, the sweep found 6 and 5 workers faster, and those counts wait for their review. At 4 threads today's plan stays. Everywhere else, today's behaviour stays until the model passes its transfer tests.

The plan change for the i9 has merged ([\#524](<https://github.com/lmmx/mibz2/pull/524>)) and goes out in the next release. It is scoped to the measured i9 profile, seven lazy workers from 10 threads asked, and it is not the model: no general selector has shipped.

The chain, in short: a sweep found that fewer workers decode faster on the i9; the counters pointed to contention for the shared cache; a model of the cache, the processors and the cores reproduces the counts it was fitted to and predicted three thread counts it hadn't seen; and its first test at a new block size refuted its best count while its contention term held, which points to a limit it lacked. A count that is measured describes the machine it was measured on. A count that is modelled can be checked anywhere.

---

**Research records:**

- [The calibration record: the sweep and the model](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/docs/journal/2026-11-17-worker-count-calibration.md>)
- Verdicts: [8 against 9 workers past the cores](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6931719814>), [7 against 8 past the cores, with counters](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6942020517>), [within the cores, with counters and the clock](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943194948>)
- [The confirming run, with held-out files](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6944271342>)
- [The review of the three runs](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943617731>), with the direct 7-against-9 contrast and the correction to the busy term
- [The 12-byte rules at 7 workers: fewer misses, no faster](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6944610173>)
- [The `bzip2 -3` run's verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945320222>), and [the model's scoring of it, with the fourth limit](<https://github.com/lmmx/mibz2/blob/be614017696167f2f84ed7dafee600afc1afa8b6/docs/journal/monodromy/2026-11-17-level3-scoring.md>)
- [The low-thread sweep's verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945639925>)
- [What the contention-associated overhead is: predictions frozen before the counters run](<https://github.com/lmmx/mibz2/blob/0e7c6888bcba2ce8dd3056c587a23be958d6de4f/docs/journal/monodromy/2026-11-17-contention-counters.md>)
- [The simple candidate rule and how predictions are scored](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943635048>)
- The model: [its form and the `bzip2 -3` test](<https://github.com/lmmx/mibz2/blob/8afbb204008adbc0df7692e0132124d110bb21ed/docs/journal/monodromy/2026-11-17-workers-machine-model.md>), and [the CPU budget and the frozen predictions](<https://github.com/lmmx/mibz2/blob/8afbb204008adbc0df7692e0132124d110bb21ed/docs/journal/monodromy/2026-11-17-workers-cpu-budget.md>)

**Further reading:**

- N. J. Gunther, ["A General Theory of Computational Scalability Based on Rational Functions"](<https://arxiv.org/abs/0808.1431>), 2008: the Universal Scalability Law.
- M. K. Qureshi and Y. N. Patt, ["Utility-Based Cache Partitioning"](<https://doi.org/10.1109/MICRO.2006.49>), MICRO 2006: shared-LLC capacity and miss rates.
- C. Xu, X. Chen, R. P. Dick and Z. M. Mao, ["Cache Contention and Application Performance Prediction for Multi-Core Systems"](<https://doi.org/10.1109/ISPASS.2010.5452065>), ISPASS 2010.
- A. Yasin, ["A Top-Down Method for Performance Analysis and Counters Architecture"](<https://doi.org/10.1109/ISPASS.2014.6844459>), ISPASS 2014.
- R. L. Mattson, J. Gecsei, D. R. Slutz and I. L. Traiger, ["Evaluation Techniques for Storage Hierarchies"](<https://doi.org/10.1147/sj.92.0078>), IBM Systems Journal, 1970.
