[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/index.md>)

# Modelling the cache to rightsize workers

Rather than re-measure every worker count whenever the decoder changes, mibz2 sizes its workers from a model of the cache and cores they share.

17 November 2026 · 13 min read · [Research in progress](<#6-what-changes-for-users-and-what-comes-next>)

mibz2 decodes a bzip2 file by handing its blocks to *workers*: threads that each decode a whole block at a time. How many workers to start is a choice mibz2 makes once, at startup, and the obvious answer, one per thread the caller asks for, is wrong. On our 10-core test machine, running fewer workers than threads decodes up to 11.5% faster. We found that by measurement alone, timing every worker count at each thread count. A sweep like that has two faults. It must be redone every time the decoder changes, because each change to the algorithm changes how much memory a worker uses, and with it the best count. And it describes one machine: a policy measured on one processor's 20 MiB cache is "works on my machine" for everyone else. So we are replacing the measured policy with a model of what the workers share: the processor's shared cache, the processor time left over after the decoder's other threads, and the cores. When the algorithm changes, we update the model's inputs, such as a worker's memory footprint, instead of re-running the sweep. On another machine, the model reads that machine's cache and cores. Its predictions are written down and frozen before each new experiment, so it can be shown wrong in public.

**What the sweep found.** The machine is an Intel i9-10900K: 10 cores, 20 hardware threads and a 20 MiB level-3 (L3) cache that all the cores share. The test file is four copies of an 11.9 MB extract of a Wikidata dump, decoded in 10 paired rounds against mibz2's current plan at the same thread count. Every worker in these runs uses the [lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/2026-11-16-lazy-letters/index.md>) kernel.

Fewer workers, less time: the i9 at 10 and 8 threads

Measured on an Intel i9-10900K, decoding four copies of an 11.9 MB Wikidata extract, 10 paired rounds a point, one worker a core. Each point is the wall time against the plan of one worker a thread at the same thread count (10 workers at 10 threads, 8 at 8); the bars are 95% intervals over the rounds, on one machine. The hollow point (6 workers at 10 threads) was run as context and not judged. 5 workers at 8 threads, run later in the low-thread sweep, took 0.997 of 8's time: no gain.

At 10 threads, today's mibz2 runs 10 workers. Seven take 0.885 of the time, 11.5% faster, with a quarter less peak memory. At 8 threads, six workers take 0.929 of the time of eight. Past the 10 cores (12 to 20 threads), today's mibz2 already runs one worker fewer than the cores, nine; seven take 0.921, 0.930 and 0.923 of its time at 12, 16 and 20 threads. A second run then tested these choices directly, beside two files that played no part in choosing them: binary records, and text compressed with 100 kB blocks. It confirmed all three: 0.881 at 10 threads, 0.933 at 8, and 0.921 to 0.930 past the cores, with 17% to 27% less peak memory. Neither held-out file decoded slower anywhere, though on the small-block text the difference was too small to call.

## What was found

The takeaways, each shown in a section below:

- **Fewer workers than threads is faster on the i9, at every count measured from 6 threads up.** At 10 threads, 7 workers take 0.885 of the wall time (11.5% faster) with a quarter less memory. At 8 threads, 6 workers take 0.929. Past the cores, 7 workers take 0.92 to 0.93 of 9's time. A confirming run, with files held out from the choice, reproduced all three. Below that, 5 workers take 0.963 of today's time at 6 threads, and at 4 threads nothing beats today's four ([section 1](<#1-what-does-a-worker-do-and-what-do-the-workers-share>)).
- **The cause is contention for the shared cache, not idle cores.** The clock speed is the same at every count. Going from 10 workers to 7, the processor cycles spent per byte fall by 25%, and the misses in the shared cache per byte fall by 48%, while the instructions per byte barely change ([section 2](<#2-why-are-fewer-workers-faster>)).
- **The model takes the smallest of three limits:** \\(W^\* \\approx \\min(\\text{cores},\\ \\text{cache limit},\\ \\text{CPU budget})\\). At 8 threads the CPU budget, about 6.6 workers, binds before the cache limit, about 8, which is why 6 workers beat 7 there ([section 3](<#3-a-model-of-three-limits>)).
- **The model must predict new evidence, not only fit old.** Its predictions for 4, 6 and 8 threads, and for files compressed with a smaller block size (`bzip2 -3`), were frozen before those runs. The `-3` run has refuted part of it: contention shrank with the footprint as the model required, but the best count stayed at 7 for a different reason, the serial scanner, so the model gains a fourth limit. At 4, 6 and 8 threads it named the best count every time, and a fixed fraction of the threads failed ([section 4](<#4-how-could-the-model-be-wrong>)).
- **Production changes only where it was measured and reviewed.** Everywhere else keeps today's behaviour until the model passes its transfer tests. The decoder will detect the machine once, decide once, and not adapt mid-stream ([section 6](<#6-what-changes-for-users-and-what-comes-next>)).

## 1\. What does a worker do, and what do the workers share?

Who does what in mibz2's parallel decoder

Conceptual. The thread names are the code's (src/parallel.rs); the kitchen labels are an analogy. In the kitchen, each cook prepares a whole dish at a station, and all of them spread their ingredients on one worktop; more cooks crowd it. The analogy stops at eviction: a cook's ingredients never vanish from a worktop, but a worker's data does leave the cache and has to be fetched again from main memory.

mibz2's parallel decoder ([`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L1-L22>)) has three kinds of thread. The *scanner* reads the compressed stream in order. A block's end is only known once its Huffman-coded symbols have been read, so this step can't be split. Each *worker* then finishes one block on its own: the move-to-front step, the inverse Burrows–Wheeler transform, a run-length step and the block's checksum. The *reader* takes the finished blocks back in order and hands out their bytes. Each worker keeps up to four blocks in flight, so seven workers hold 28.

Blocks decode independently, so more workers means more blocks at once. But the workers are not independent. They run on cores that share one L3 cache.

The i9's caches: private L2s, one shared L3

The Intel i9-10900K the measurements were made on. The L1 caches, per core and smaller still, are left out. A worker's footprint is about eight times its core's L2, so it lives in the shared L3.

Each core has its own small level-2 (L2) cache, 256 KiB. Behind them sits the 20 MiB L3, shared by all ten. Data a worker needs that isn't in either comes from main memory, which takes several times longer. A lazy-letters worker rebuilding a 900 kB block uses about 2.1 MB of memory it touches over and over: its *footprint*. That is far more than the L2, so the footprints meet in the L3.

Workers filling the shared L3

Workers: 7

Drag the slider to change the number of workers; the text below the bar says what the measurements show at that count. Each green block is one worker's footprint, about 2.1 MB for a lazy-letters worker on a 900 kB block, in the i9's 20 MiB (21.0 MB) L3. This is capacity accounting, not a model of the misses: footprints overlap and change while a block decodes, and the cost rises smoothly as they grow, with no cliff at 100%. The dashed lines are the teams policy's 0.7 of the L3 and the fitted optimum's 0.8.

## 2\. Why are fewer workers faster?

There are three explanations a reader might reach for. One is the clock: a processor running fewer cores may run each faster. The other is idle time: with more workers than the processor can keep busy, some wait. A third is the cache: each worker slows the others by evicting their data. The hardware counters at 10 threads tell these apart. They count, per byte of output, against 10 workers.

Fewer workers, cheaper bytes: the work is the same, the shared-cache misses fall

Measured on the i9 at 10 threads, one copy of the Wikidata extract, 10 repetitions under perf stat, pinned one thread a core. L3 misses are the last-level cache's load misses. The clock (cycles over reference cycles) was 1.326 at every count, so the cycles are not falling because the cores slowed or sped up.

The clock is ruled out first. Cycles over reference cycles, which measures the clock against its base rate, is 1.326 at every worker count. The work is the same: instructions per byte fall by under 2% from 10 workers to 7. The private L2 caches see the same traffic, with L2 misses within 3%. What changes is the shared cache. From 10 workers to 7, misses in the last-level cache (the L3) per byte fall by 48%, and the cycles per byte fall by 25%. Each worker gets cheaper as fewer share the L3.

The fall is smooth, with no cliff. Each worker removed makes the others' misses a little rarer. An earlier prediction that the cost would fall off a cliff once the footprints fit was refuted by exactly this shape.

**The wall time, as bookkeeping.** The wall time is the cycles each byte costs, divided by how many processors are working on the bytes. At 10 workers each worker is busy 0.763 of the time, so 7.63 processors' worth are working; at 7 workers each is busy 0.925 of the time, 6.48 processors' worth. That is 15% less processing power. The cycles per byte fell 25%. So the wall time is \\(0.752 / 0.849 \\approx 0.886\\) of what it was, against 0.885 measured. Fewer workers lose some parallelism and gain more back in cheaper bytes, up to a point.

This bookkeeping is close to an identity when the clock is flat, so it explains nothing on its own: any cause would balance it. The evidence for the cache is the counters' pattern: same instructions, same clock, same private-cache traffic, and fewer shared-cache misses.

## 3\. A model of three limits

The model's answer is \\(W^\* \\approx \\min(C,\\ W\_{L3},\\ W\_{\\mathrm{CPU}})\\): the cores available \\(C\\), the cache limit \\(W\_{L3}\\) and the CPU budget \\(W\_{\\mathrm{CPU}}\\), each counted in workers. The smallest binds, since plenty of one resource doesn't make up for running out of another. A fourth limit, how fast the scanner can feed the workers, was added after the `bzip2 -3` run ([section 4](<#what-the-bzip2-3-run-showed>)); it doesn't bind at the default block size.

**The cache limit.** At 10 threads, the cycles per byte with \\(W\\) workers fit \\(c(W) = a + b\\,W^2\\), with \\(a \\approx 0.42\\) the cost of a worker alone and \\(b\\,W^2\\) the contention (\\(b \\approx 0.0068\\)). The square comes from each of \\(W\\) workers missing more often in proportion to the \\(W\\) footprints sharing the cache. Throughput is then proportional to \\(W / (a + b\\,W^2)\\), which peaks where \\(b\\,W^2 = a\\): where contention costs as much as the work itself. That is \\(W = \\sqrt{a/b} \\approx 7.9\\). At 2.1 MB a worker, 7.9 footprints fill about 0.8 of the 20 MiB L3. The fits with other exponents put the peak in the same place, 0.77 to 0.84 of the L3, so with \\(L\_3\\) the L3's size and \\(F\\) a worker's footprint, the rule is \\(W\_{L3} \\approx 0.8 \\times L\_3 / F\\). The 0.8 is a fitted constant, not a law. mibz2's teams policy already sizes its tables to about 0.7 of the L3, and the simple candidate rule frozen before the next runs uses that 0.7, which gives 7.

**The CPU budget.** A worker with a processor to itself is busy 0.93 of the time; the rest it waits on the rest of the pipeline. The scanner and reader use about one processor (1.08 counted at 10 threads). And a fully subscribed set of processors leaves some idle, about 1.25 at 10 threads, from the scheduler's slack. So the workers can use at most \\(N - \\nu - \\iota\\) processors, with \\(N\\) the processors allowed, \\(\\nu\\) the non-worker threads' use and \\(\\iota\\) the slack, and the budget in workers is \\(W\_{\\mathrm{CPU}} \\approx (N - \\nu - \\iota) / 0.93\\). This reproduces the measured busy shares at 10 threads: 0.767, 0.851, 0.930, 0.930 and 0.930 for 10 down to 6 workers, against 0.763, 0.857, 0.931, 0.925 and 0.931 measured.

An earlier version of this term was derived from the wall times and the cycles, and the review of the runs pointed out that it was close to the identity above, so its match was not independent evidence. The term is now built only from the processor time each thread was counted using.

The model at two thread counts: the smallest limit binds

Modelled, from measured inputs. The bars are in workers, with the arithmetic in the text below; the solid bar is the one that binds, and the dashed line marks it. The non-worker load at 10 threads (1.08 processors) was counted; at 8 threads (0.85) it is the model's estimate, scaled with throughput. The slack is 0.125 of the processors allowed. A count is a whole number, so the model takes the whole number below the binding limit.

**At 10 threads,** the budget is \\((10 - 1.08 - 1.25) / 0.93 \\approx 8.2\\) workers, the cache limit is 7.9, and the cache binds: 7 workers, as measured. **At 8 threads,** the budget is \\((8 - 0.85 - 1.0) / 0.93 \\approx 6.6\\), below the cache limit, so the budget binds: 6 workers, as measured. A seventh would get only part of a processor while adding a whole footprint to the cache. **Past the cores,** with 20 processors the budget is about 17, and the cache limit gives 7 again.

The model misses one thing at 8 threads. It puts 7 workers level with 6, but 7 measured 5% slower (0.975 against 0.929). And fitting the data it was built from is no proof.

### Half the cycles go to contention, and that's the optimum

The cache limit says more than where the peak is. If each worker's cycles per byte are \\(c(W) = a + b\\,W^2\\), the wall time with \\(W\\) workers goes as \\(c(W)/W = a/W + b\\,W\\). The first term falls with more workers and the second rises, and their sum is smallest at \\(W^\* = \\sqrt{a/b}\\), exactly where \\(b\\,W^2 = a\\). So at the best worker count, contention takes as many cycles as the work itself: about half of all cycles.

On the i9 at 7 workers, the fit puts contention at \\(b\\,W^2 = 0.0068 \\times 49 \\approx 0.33\\) of \\(c(7) \\approx 0.76\\), about 44% of the cycles. That is a little under half, because a worker count is a whole number, and 7 sits just below the fit's optimum of 7.9. A large share of cycles lost to contention is therefore what a well-chosen worker count looks like, not a sign of waste. It also gives the size of the prize for removing contention some other way than by cutting workers: if it vanished, 7 workers would need \\(a \\approx 0.42\\) of the cycles instead of 0.76, and run about 1.8 times faster.

The 44% is soft. The cost of one worker alone, \\(a\\), is extrapolated from fits over 6 to 9 workers. And the exponent isn't pinned: fits with \\(W^{1.5}\\) to \\(W^{2.5}\\) in place of \\(W^2\\) match the data equally well, and move the share at 7 workers from about 60% to about 34%. The low-thread sweep, with 2 to 5 workers, measured the cost close to one worker alone; the fit of \\(a\\) to those points is still to be reported.

A shrunken footprint has already shown what those contention cycles are not. A variant of the lazy-letters kernel that stores its rules in 12 bytes instead of 16 shrinks each worker's footprint. With 9 workers past the cores, it took 0.98 to 0.99 of the wall time. With 7 it cut the shared-cache misses per byte by 7.4%, and the wall time didn't move: 1.012, 0.996 and 1.002 of the 16-byte kernel's at 12, 16 and 20 threads, none of them beyond noise, so it was not adopted. Once the worker count fits the cache, the remaining contention cycles are not paid in footprint bytes or in the number of misses. The counters already hint at why. From 10 workers to 7, each shared-cache miss avoided saved about 589 cycles, more than a whole uncontended trip to main memory (390 to 490 cycles on this machine), so even if no two misses overlapped, fewer misses would explain at most two thirds to four fifths of the saving, and with the overlap the kernel is built for, far less. Our working conjecture ([set out in the model's journal](<https://github.com/lmmx/mibz2/blob/0e7c6888bcba2ce8dd3056c587a23be958d6de4f/docs/journal/monodromy/2026-11-17-contention-counters.md>)) is that the rest is the 0.69 L3 accesses each byte makes. They are 35 times as many as the misses and don't change with the worker count, but each one becomes 4 to 8 cycles dearer as more workers queue in the L3 and on the ring that links it to the cores. Main memory is not the bottleneck: its traffic is estimated at 4 to 9 GB/s, 10% to 20% of the machine's peak. Fewer workers shorten the queue; a smaller footprint doesn't change the number of L3 accesses, which is why the 12-byte rules did nothing at 7 workers. If this holds, two kinds of change could win back some of the 44% without cutting workers: fewer L3 accesses a byte, from a hot set that fits in the 256 KiB L2, and fewer accesses in flight at once, by staggering the workers' access-heavy phases. It is a conjecture, under test: its predictions were frozen before a counters run that measures the latency of L3 hits and the traffic to main memory.

## 4\. How could the model be wrong?

Every prediction is frozen before its run and scored after it. So far:

- a capacity *cliff*, once the footprints fit, was refuted by the smooth fall in misses;
- contention for the ring that links the cores, or for memory bandwidth, was refuted on its counter pattern: it predicted flat misses, and they fell by 41% from 8 workers to 6;
- at 10 threads the model named 7 workers, which was right, but it predicted 0.93 of the time, and 0.885 was measured: the gain was larger than it allowed.

**Frozen for the low-thread sweep** (4, 6 and 8 threads):

| threads | model | model, rival slack | simple rule (\\(0.7 \\times\\) threads) | measured best |
| ---: | ---: | ---: | ---: | --- |
| 8 | 6 | 6 | 6 | 6 |
| 6 | 5 | 5 | 4 | 5 |
| 4 | 4 | 3 | 3 | 4 (today's plan) |

**The low-thread sweep** has run, and the model's predictions held. Against today's plan at each count:

Below 8 threads: 5 workers win at 6 threads, and nothing beats today's 4 at 4

Measured on the i9, decoding four copies of the Wikidata extract, 10 paired rounds a point, one worker a core; the bar is the 95% interval over the rounds. At 6 and 4 threads today's plan runs the older (eager) kernel, one worker a thread, so the 6-worker point at 6 threads and the 4-worker point at 4 threads measure the lazy-letters kernel alone. The 2-worker point at 4 threads, 1.533, is off the chart.

At 8 threads, 6 workers took 0.925 of 8's time, a third confirmation of that count, and 5 workers gained nothing (0.997). At 6 threads, 5 lazy-letters workers took 0.963 of today's time, even though the lazy-letters kernel alone costs 4% there (6 lazy workers took 1.040). At 4 threads, nothing beat today's plan: 4, 3 and 2 lazy workers took 1.098, 1.113 and 1.533. The model named 6, 5 and 4 workers, and was right at all three counts. Its frozen ratio for 4 lazy workers against today's plan, 1.09 (1.04, 1.13), came out at 1.098. The busy share a worker stayed flat, at 0.742, 0.725 and 0.740 at 8, 6 and 4 threads, as the model predicted and the rival did not, so the rival is refuted.

**A fixed fraction of the threads is refuted.** The simple rule, 0.7 of the threads rounded, picks 4 workers at 6 threads and 3 at 4, and each choice costs about 11% to 12% against the best: well past the 2% allowed. Rounding down instead fails at all three counts. The rule that survives is the model's smallest of limits.

The rival keeps the slack \\(\\iota\\) fixed at 1.25 processors rather than in proportion to \\(N\\) (the model takes \\(\\iota = 0.125\\,N\\)). The two disagree at 4 threads, and the counters can tell them apart without timing anything: each worker's busy share with as many workers as threads stays about 0.76 under the model, against 0.56, 0.67 and 0.74 at 4, 6 and 8 threads under the rival.

A prediction passes at a thread count if it is within one worker of the best measured, and its choice costs at most 2% against the best: \\(\\mathrm{wall}(W\_{\\text{predicted}}) / \\mathrm{wall}(W\_{\\text{best}}) - 1 \\le 2\\%\\).

**The test that can fail.** The same file compressed with `bzip2 -3` has blocks a third the size. If a footprint shrinks with its block, to about a third, the cache limit moves from about 8 to about 24. The model predicts that the trim disappears: at 10 threads the best count is 9 or 10, and 7 workers are no better than 0.98 of 10's time. If contention were set by memory bandwidth instead, 7 would still win at about 0.885, as now.

**The free CI machines.** GitHub's free x86 runners have 4 processors on 2 cores and a 32 MiB L3. There the cache limit is about 12.8 and never binds, so the model predicts that 4 workers stay best (or 3 within 2%), and that today's plan stays. With only 2 to 4 workers possible, "within one worker" fails there only on an outcome the existing data already rule out, so the runners are judged on three refuters instead: the model is wrong if 3 workers beat 4 by more than 2%, if 2 come within 3% of 4, or if any lazy-letters plan beats today's by more than 2%.

### What the `bzip2 -3` run showed

The test that could fail did fail, in part. Recompressed at `-3`, each worker's buffers shrank to 0.355 of their size at `-9` (5.97 MB against 16.81 MB). At 10 threads, against 10 workers, 9, 8, 7 and 6 workers took 0.992, 0.987, 0.978 and 0.987 of the wall time, and 4 workers took 1.343, slower.

Smaller blocks, a smaller gain, the same best count

Measured on the i9 at 10 threads, one worker a core, 10 paired rounds a point; bars are 95% intervals over the rounds, on one machine. The `-9` curve is the first 10-thread run (figure 1); the `-3` curve is the same Wikidata extract recompressed with 300 kB blocks. Each point is against 10 workers on the same file.

The best count is still 7, but the gain from trimming fell from about 12% to 2%. Per byte at 7 workers, the cycles fell to 0.935 of 10 workers' and the shared-cache misses to 0.635, with the clock flat at 1.326. Scored against what was frozen before the run:

- "the trim disappears, and the best count is 9 or 10": **refuted**, it is 7;
- "7 workers are no better than 0.98 of 10's time": borderline, at 0.978 (0.973, 0.983);
- the bandwidth rival, "7 at about 0.885, as at `-9`": **refuted**;
- the three-limit model as frozen, which put the cache limit near 22 and the CPU budget near 8.8, so 9 workers, with 7 slower than 10: **refuted**. At 10 workers, 2.6 of the 10 processors sat idle, so no CPU budget was binding.

**What held: contention shrinks with the footprint.** If contention depends only on how full the L3 is, the contention term \\(b\\) should shrink with the footprint as \\(F^k\\), with \\(k\\) the same exponent that \\(W\\) has, about 2. Comparing the slopes of the cycles and of the misses per byte over the same range of workers at both block sizes gives \\(k\\) between 1.7 and 2.1. The contention term transfers.

**What failed: the best count is 7 at both sizes for different reasons.** At `-9` the cache limit and the CPU budget bind. At `-3` the scanner does. Part of its work, and the reader's, is paid once a block, and a `-3` file has three times as many blocks a byte, so that part weighs three times as much. With 10 workers, the workers sat idle waiting on the scanner 28% of the time at `-3`, against 5% at `-9`, and the throughput stayed at about 2.5 GB/s from 6 workers to 10. Past that point an extra worker adds only contention, to its own cycles and to the scanner's, which is the 2% that 7 workers save.

**The corrected form** adds that limit:

\\\[W^\* \\approx \\min(C,\\ W\_{L3},\\ W\_{\\mathrm{CPU}},\\ W\_{\\mathrm{ser}}),\\\]

where \\(W\_{\\mathrm{ser}}\\) is the fewest workers that keep up with the scanner: about 7 at `-3` on the i9, between 6.5 and 7.5. It needs one more input, the scanner's cost per block, a constant of the kernel measured once. Block size then enters the model twice, through the footprint in \\(W\_{L3}\\) and through the scanner in \\(W\_{\\mathrm{ser}}\\).

Two caveats. The idle shares come from an occupancy trace whose run failed its own memory-ledger check, so they are reported but flagged; the throughput plateau and the 2.6 idle processors in the timed runs support the reading independently. And the 0.355 is the ratio of the workers' total buffers, not of a measured hot set; if the hot set shrank by a different factor, \\(k\\) moves with it. A sharper test of whether contention depends only on how full the L3 is was frozen before the low-thread sweep. The `-3` result changes no production plan.

> [!NOTE]
>
> **Results pending**
>
> The free CI runners have not yet reported, and the 8- and 6-thread counts await their review before any production change. This section will give each beside its result.

## 5\. What changes on other machines?

Two clusters, two caches: the limit is summed per cluster

Conceptual: a processor with two L3 clusters, as on many AMD chips or a two-socket server. F is a worker's footprint. The cache limit is the sum of each cluster's limit, provided the workers are spread across the clusters.

Many processors have more than one L3. AMD's chips group their cores into clusters with an L3 each, and a two-socket server has one per socket. Workers compete only within a cluster. So the cache limit becomes a sum over the clusters the process may use: \\(W\_{L3} = \\sum\_d \\min(C\_d,\\ 0.8 \\times L\_d / F)\\), over each cluster \\(d\\) with \\(C\_d\\) cores and an L3 of \\(L\_d\\).

Today's [`l3_bytes()`](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/src/parallel.rs#L2831-L2846>) reads one file, the first processor's `cache/index3/size`. It assumes the cache numbered 3 is the L3, rather than finding the one whose level is 3, and it reads one cluster. The fix groups the processors the process may use by the L3 they share (`shared_cpu_list`) and reads each one's size. The sum holds only if workers are spread across the clusters, so each worker gets a soft affinity to a cluster, not a processor. The CPU budget stays one number for the process.

## 6\. What changes for users, and what comes next

The decoder will detect once, decide once and then decode. At startup it reads the cores, the L3 clusters and the processors it may use, and the block size from the stream's header. It then picks the count and keeps it. It won't tune itself mid-stream: streams are often short, timings are noisy, and the output's timing would stop being repeatable.

Production changes only where the change was measured and reviewed. On the i9's own profile, seven workers past the cores and at 10 threads passed their review. At 8 and 6 threads, the sweep found 6 and 5 workers faster, and those counts wait for their review. At 4 threads today's plan stays. Everywhere else, today's behaviour stays until the model passes its transfer tests.

The plan change for the i9 has merged ([\#524](<https://github.com/lmmx/mibz2/pull/524>)) and goes out in the next release.

The chain, in short: a sweep found that fewer workers decode faster on the i9; the counters showed why, contention for the shared cache; a model of the cache, the processors and the cores reproduces the counts; and the model now has to predict machines and block sizes it hasn't seen. Its first such test, at a smaller block size, confirmed its contention term and refuted its best count, which added a limit it lacked. A count that is measured describes the machine it was measured on. A count that is modelled can be checked anywhere.

---

**Research records:**

- [The calibration record: the sweep and the model](<https://github.com/lmmx/mibz2/blob/33e7225ba6147fb480168dc54fe30727f0b93aea/docs/journal/2026-11-17-worker-count-calibration.md>)
- Verdicts: [8 against 9 workers past the cores](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6931719814>), [7 against 8 past the cores, with counters](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6942020517>), [within the cores, with counters and the clock](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943194948>)
- [The confirming run, with held-out files](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6944271342>)
- [The review of the three runs](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943617731>), with the direct 7-against-9 contrast and the correction to the busy term
- [The 12-byte rules at 7 workers: fewer misses, no faster](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6944610173>)
- [The `bzip2 -3` run's verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945320222>), and [the model's scoring of it, with the fourth limit](<https://github.com/lmmx/mibz2/blob/be614017696167f2f84ed7dafee600afc1afa8b6/docs/journal/monodromy/2026-11-17-level3-scoring.md>)
- [The low-thread sweep's verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6945639925>)
- [What the contention cycles are: predictions frozen before the counters run](<https://github.com/lmmx/mibz2/blob/0e7c6888bcba2ce8dd3056c587a23be958d6de4f/docs/journal/monodromy/2026-11-17-contention-counters.md>)
- [The simple candidate rule and how predictions are scored](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6943635048>)
- The model: [its form and the `bzip2 -3` test](<https://github.com/lmmx/mibz2/blob/8afbb204008adbc0df7692e0132124d110bb21ed/docs/journal/monodromy/2026-11-17-workers-machine-model.md>), and [the CPU budget and the frozen predictions](<https://github.com/lmmx/mibz2/blob/8afbb204008adbc0df7692e0132124d110bb21ed/docs/journal/monodromy/2026-11-17-workers-cpu-budget.md>)
