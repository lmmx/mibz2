[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/index.md>)

# Raising the knee: scaling throughput to the limit of physical cores

mibz2 0.5.4 now speeds up until all ten physical cores of its test processor are busy, then holds level on the whole machine up to twenty threads.

17 November 2026 · 13 min read · [Research result](<#6-what-comes-next-for-the-knee>)

[Ten days ago](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-07-why-mibz2-stops-scaling/index.md>), mibz2's problem was that giving its bzip2 decoder more work at once could make it slower. The decoder processes a compressed file in independent blocks, so it can decode several simultaneously. But each block has its own data structures, and those structures compete for the processor's shared cache. More blocks promise more parallelism while making each block more expensive. Past a certain point the extra cost wins. The throughput curve bends over: the *scaling knee*.

The default decoder once dealt with that risk by stopping at a fixed number of decoding teams. It could avoid a fall, but only by declining to use the extra threads. Since then, mibz2 has changed both how it reconstructs a block and how many workers it starts. Version 0.5.4 puts the result in view. Our test machine is a ten-core Intel desktop (an i9-10900K, with 20 logical CPUs and a 20 MiB level-3 cache that all ten cores share). On it, ten requested threads decode a 210 MB Wikidata sample in **0.102 seconds**, the fastest median in the release sweep. The knee is still there, but it has been raised and softened. It is raised because the best point has moved from eight threads to ten, the machine's physical-core count. It is softened because asking for more now costs little or nothing. In 0.4.0 and 0.5.0, 12 to 20 threads took 11–18% longer than the best point. In 0.5.4's release sweep they take 4–5% longer by the medians, but that sweep pins ten threads and gives twelve or more the whole machine. With placement held constant, 10, 12 and 20 threads decode in the same time, to within half a per cent. It is not straightened: the extra hardware threads add nothing yet. At 12 to 20 requested threads 0.5.4 runs seven active workers, not twelve or twenty. Memory bandwidth is close to binding there: the plateau moves 75% of the memory traffic the machine can sustain, but that is not yet a hard ceiling.

Throughput against threads, five releases on the test machine

Measured on the test machine: the median of 10 paired rounds per point, decoding an 11.9 MB extract of a Wikidata dump to 209.7 MB of output. Each release is taken from its own [release-scaling run](<https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/>) (0.2.4 from 6 November; 0.5.0, 0.5.1 from 16 November; 0.5.3, 0.5.4 from 17 November). The runs pin 10 threads or fewer to one CPU a core, and give 12 threads or more the whole machine.

**The result is a flatter *default* curve, not the disappearance of the knee or of contention.** The decoder now starts fewer workers than threads in the measured configurations where extra workers made decoding slower. In effect, 0.5.4 caps its workers at the number that pays off: seven, from ten requested threads. The cost of competing for the cache remains; it is now part of the plan rather than a surprise in the benchmark. Here is how we reached that result, what the release run establishes, and what the plateau still leaves open.

## 1\. What was the knee?

A bzip2 file is made of blocks that can be decoded independently. mibz2 has two ways to spend its threads. In a *team*, several threads share the work of one block. With *workers*, each thread decodes a block of its own. The second arrangement can keep more cores busy, but it also puts more blocks' working data in the shared last-level cache (the L3 on the test machine).

The first scaling investigation measured the trade-off directly. On the test machine, each additional block in progress slowed the others. Three teams of three decoded faster than four teams of two. Version 0.2.5 adopted that smaller number of teams and gained about 9% at ten threads against 0.2.4, in two registered runs.

> [!NOTE]
>
> **Levelling off is not the same as falling**
>
> The old **shipped** curve mostly *levelled off*: version 0.2.4 ran four teams of two from eight requested threads upwards, instead of allowing concurrency to keep growing. The falling side of the knee was visible when experiments forced too many blocks to run at once. The policy avoided that fall by leaving extra threads unused. It had contained the problem, not made more concurrency productive.

The kernel changed next. [Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) reconstructed the text from compact rules rather than the old row-by-row navigation table. [Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-16-lazy-letters/index.md>) then kept working state for only the letters that needed it, reducing the amount each worker touched. Those gains reached the workers, which ran below ten threads, but not the teams, which still ran from ten. So by 0.5.0 the shipped curve really did fall: eight threads ran eight workers, ten or more ran three teams of three, and 12 to 20 threads took 16–18% longer than 8. In 0.5.1, workers with lazy letters displaced teams at ten threads and above. What was now faster than teams could still become slower if too many workers ran together. That was the next knee to control.

What mibz2 runs at each thread count, release by release

From the decoder's thread names, recorded in each release's release-scaling run, and the release notes. A team (amber) shares one block between several threads; a worker (green) decodes a whole block on its own. A thick border marks a cell that changed from the release before. Where a cell has two counts, the first is the plan when the process is pinned one CPU a core, which is how the release-scaling runs measure 10 threads or fewer, and the second the plan on the whole machine.

## 2\. What does the new curve look like?

The [release-scaling benchmark](<https://lmmx.github.io/mibz2/pr-preview/pr-692/benchmarks/history/index.md>) decodes the same Wikidata sample on the test machine at each requested thread count. It reports the time to produce about 210 MB of output. Lower is better. These are the medians from the 0.5.3 and 0.5.4 release runs:

| threads requested | 0.5.3, seconds | 0.5.4, seconds |
| ---: | ---: | ---: |
| 1 | 0.387 | 0.385 |
| 2 | 0.244 | 0.241 |
| 4 | 0.148 | 0.144 |
| 8 | 0.110 | 0.105 |
| **10** | **0.115** | **0.102** |
| 12 | 0.107 | 0.106 |
| 16 | 0.107 | 0.106 |
| 20 | 0.106 | 0.108 |

The runs of ten threads or fewer are pinned one logical CPU per physical core; the higher counts use the whole machine. The paired version comparisons preserve placement within each count.

The shape matters more than the last decimal place. In 0.5.3, asking for ten threads made the decoder slower than asking for eight. At twelve and above it recovered. In 0.5.4, **ten threads are the fastest point**, while twelve to twenty sit between 0.106 and 0.108 seconds. Across eight to twenty threads that is **1.95–2.06 GB/s of decoded output**. Twenty is still about 5% slower than ten by these medians. As the box below explains, most or all of that is a change of placement, not of thread count.

The paired comparison is stronger evidence for the version change than the medians alone. Across ten paired rounds, 0.5.3 took **1.123 times** 0.5.4's wall time at ten threads (95% interval 1.094–1.152), or about 11% less elapsed time in 0.5.4. At eight threads the ratio was **1.045** (1.038–1.053). The new version also used less processor time and peak memory at those counts. At one, two and twelve to twenty threads, the release comparison detected no change. The apparent improvement at four threads was too small to resolve from the run's variation.

These comparisons show a real improvement at eight and ten threads. They do not by themselves establish that ten is statistically faster than every other count within 0.5.4; the table shows the median ordering, not paired confidence intervals between different thread counts. The figure below gives those within-release comparisons against eight threads, for selected releases since 0.2.4. It leaves out 0.2.5 to 0.3.3. The history between was not one continuous fall. In 0.2.5, ten threads took 0.918 of eight threads' wall time (95% interval 0.910–0.927).

> [!NOTE]
>
> **The decline past ten is a change of placement**
>
> Paired round by round within 0.5.4's run, the higher counts took longer than ten threads:
>
> | threads | wall time against ten threads (95% interval) |
> | ---: | ---: |
> | 12 | 1.049 (1.022–1.076) |
> | 16 | 1.044 (1.025–1.064) |
> | 20 | 1.074 (1.031–1.117) |
>
> Past ten, extra threads would land on the cores' second hardware threads. But the run also changes placement at that point. Ten threads ran pinned to one logical CPU a core. Twelve to twenty ran on the whole machine, with the same seven workers. So the decline can't be put down to hyperthreading.
>
> The knee diagnostic did hold placement constant: it ran every count on the whole machine, in one session. Its medians, in seconds, with the interquartile range for four copies:
>
> | threads | four copies of the sample | one copy |
> | ---: | ---: | ---: |
> | 10 | 0.3661 (0.365–0.368) | 0.1013 |
> | 12 | 0.3657 (0.365–0.369) | 0.1009 |
> | 20 | 0.3664 (0.364–0.370) | 0.1010 |

Its build falls between 0.5.3 and 0.5.4, which run the same plan on the whole machine. A release-build sweep from eight to twenty threads, all on the whole machine, would confirm it for 0.5.4 itself.

Placement held constant: faster up to the cores, level beyond

Measured on the test machine. Solid lines: the [knee diagnostic](<https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T123939Z.tar.gz>) of 17 November, every count on the whole machine in one session, 10 rounds a point; the dots are medians and the bars the interquartile range, decoding one copy or four copies of the Wikidata sample. Its build (399d33d6) falls between 0.5.3 and 0.5.4; on the whole machine both releases run the same plan, 4 and 8 workers at 4 and 8 threads and 7 workers from 10. Dashed: 0.5.4's release sweep, medians of 10 rounds, pinned one CPU a core up to 10 threads (filled) and on the whole machine from 12 (hollow). The vertical axis starts at 1.3 GB/s.

The fall past eight threads, release by release

Measured on the test machine. Each mark is a release's wall time at 10, 12, 16 or 20 threads divided by its own wall time at 8 threads, as the geometric mean of 10 rounds, paired by round, with its 95% interval across those rounds (one run, one machine, one file). Right of the axis is slower than 8 threads. 0.4.0 comes from 0.5.0's run, where it was the older release. 0.5.3's 10-thread mark is its pinned plan of 10 workers. Selected releases: 0.2.5 to 0.3.3 are not shown. Sources: the [release-scaling runs](<https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/>).

## 3\. Why did ten threads dip, and what fixed it?

The ten-thread regression in 0.5.3 was a placement mistake in the planner. Version 0.5.3 tried to recognise the test machine from the CPUs the *process was allowed to use*. With the process pinned to ten CPUs, it no longer recognised the full machine and declined to apply its seven-worker limit. It started **ten workers**, with forty blocks allowed in flight.

> [!NOTE]
>
> **How the release runs place their threads**
>
> The release harness pins runs of ten threads or fewer to one logical CPU per physical core. At ten requested threads, the process may use CPUs 0–9: one logical CPU from each physical core. Its workers, scanner and reader share that set. Runs of twelve threads or more may use all 20 logical CPUs. Pinning is how mibz2 runs under `taskset` or in a container limited to some CPUs; the whole machine is how it usually runs.

Unpinned, 0.5.3 did apply the limit: a later check of production on the whole machine found it building seven workers in all six runs. That check, run after 0.5.3 shipped, also timed seven workers against ten directly, with the counts set explicitly, and seven took 0.873 of ten workers' time (95% interval 0.866–0.880). The release benchmark's ten-thread dip was therefore not evidence that the seven-worker idea failed. It was evidence that the intended plan had not run.

Version 0.5.4 [separates the two questions](<https://github.com/lmmx/mibz2/pull/537>). It identifies the machine from all present and online CPUs, but still reads the process's affinity to decide which measured plan applies. Pinned to ten cores, it now starts **seven lazy-letters workers**, with 28 blocks in flight. Pinned to eight, it starts six, and pinned to six and four it trims one worker as well. Those three pinned trims did not carry over to the whole machine: in the same check, six workers at eight threads took 1.000 of eight workers' time, the six-thread trim was no faster, and the four-thread trim took 1.21 times as long. Competition for CPUs with the decoder's scanner and reader, which share the workers' cores when the process is pinned, is implicated, so 0.5.4 applies those trims only when pinned. The ten-thread trim holds in both modes; why exactly it pays is not settled.

The machine-wide check is cached once per process. An early release check caught repeated hardware discovery adding overhead to short decodes; that was fixed before 0.5.4 shipped. The result is a narrower policy than a universal formula: it applies to the test machine and its tested placement modes, not automatically to every ten-core processor.

## 4\. Does the plateau mean memory bandwidth is full?

Nearly, but not yet. The plateau uses 75% of the memory bandwidth the machine can sustain, and every worker past seven adds traffic faster than it adds output.

With ten workers reduced to seven, the [worker-count study](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/index.md>) found about **half as many last-level-cache misses per decoded byte** and **25% fewer cycles per byte**. Instructions and clock frequency stayed approximately level.

At ten workers, each was busy about 76% of the time; at seven, each was busy about 92%. The smaller group offered less aggregate worker time, yet it decoded each byte much more cheaply. That is why removing workers can make a parallel program finish sooner. The utilisation arithmetic alone does not establish the cause; the cache counters supply the evidence for interference.

A diagnostic run on the plateau itself rules out the simplest alternatives. Decoding four copies of the sample on the whole machine, the workers compute 93–95% of the time at every thread count, so no serial stage, such as the scanner that reads the file or the reader that hands bytes back in order, is starving them. What rises with the workers is memory traffic. Counted at the memory controller, reads and writes come to 7.5 bytes for every byte of output at four workers, 11.0 at seven and 12.4 at eight.

More workers, more memory traffic for every byte decoded

Measured on the test machine at the memory controller, reads and writes per byte of output. Green: the current decoder, decoding four copies of the Wikidata extract, in the [knee repeat with the memory-controller counters](<https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T230729Z.tar.gz>) of 17 November (the whole machine, except the hollow point, pinned). Amber: an older decoder that held a slot for every letter of a block, over 15 repeated decodes of the single Wikidata sample a reading ([memory-controller run](<https://github.com/lmmx/mibz2/blob/evidence/cells-imc/run-20261114T180428Z.tar.gz>), 14 November). The two runs used different decoders and workloads, so they are drawn side by side, not joined. The first knee diagnostic counted only reads that missed the L3, 6.9, 11.0 and 12.7 bytes at 4, 7 and 8 workers, close to the green totals. The analogy stops where caches differ from kitchens: data is not used up, it is copied in 64-byte lines.

It is tempting to read the two-gigabyte-per-second plateau as a DRAM bandwidth limit, so we measured it. A repeat of the diagnostic with the memory controller's own counters found the seven-worker plateau moving 24.8 GB/s of DRAM traffic: 11.0 bytes per output byte, of which 7.0 are reads and 4.0 writes. A separate test that streams memory as fast as it can reaches 33.1 GB/s on the same counters, so the plateau runs at 75% of the measured ceiling. On an older version of the Cells kernel, on 14 November, the same counters measured 22.7 and 24.2 bytes per output byte at eight and ten workers. The decoder has roughly halved its DRAM bytes per output byte since.

More workers push the traffic up faster than the output. Eight workers reach 83% of the ceiling at 12.4 bytes per output byte, and ten pinned workers 85% at 13.9, and both decode more slowly than seven. So bandwidth is close to binding, and every worker past seven spends it on traffic rather than output. It is not the hard ceiling at seven: the registered rule called bandwidth the limit only at 80% of the ceiling or more, and 75% leaves the question open.

At the plateau: how close is memory traffic to its ceiling?

Measured on the test machine, decoding four copies of the Wikidata extract ([knee repeat with the memory-controller counters](<https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T230729Z.tar.gz>), 17 November). Each bar is DRAM reads plus writes counted at the memory controller, the median of 10 rounds, with idle traffic subtracted. The ceiling is measured too: the best median of a separate test that streams memory as fast as it can (`ab stream`, its scale kernel), read on the same counters. At 7 workers the decoder writes 2.27 GB of output a second; at 8 and at 10 pinned workers it is slower than at 7, while moving more memory traffic.

There is another reason to be precise about the word *plateau*. At twelve to twenty **requested** threads on this machine, 0.5.4 still runs **seven decoding workers**. The decoder is not starting twelve, sixteen or twenty competing workers; the plan has capped the number of blocks being decoded simultaneously. A nearly flat result there means the planner has kept the most expensive part of the curve out of the default path. It does not prove that the hardware would scale if twenty independent decoding workers were set loose.

## 5\. How would we straighten the knee?

Straightening the knee means a curve that keeps rising past the ten physical cores. The second hardware thread on each core (simultaneous multithreading, which Intel calls hyperthreading) would then add speed. Today it adds none. Three routes could change that. Each is a hypothesis with a measurement that tests it; none is a result yet.

**Cheaper bytes per worker.** At seven workers, the reads that miss the L3 come to 11 bytes for every byte of output. A smaller hot set per worker would cut that: the store and the tables the kernel keeps for a block. So would less traffic on the output side. One piece of the output side is folding the block's checksum into the final pass. That is an open research arm ([\#535](<https://github.com/lmmx/mibz2/pull/535>)), and its measured 1.5–3% gain is not yet settled. Fewer bytes moved per output byte raise the number of workers that pays, so more workers, and eventually the second hardware threads, can contribute. With bandwidth close to binding, this is the route that raises the ceiling. The test: bytes per output byte and wall time by worker count, before and after each cut.

**Latency hiding with the second hardware threads.** The counters run at ten threads measured about 2.2 reads that missed the L3 in flight at once, whenever any were. That is little memory-level parallelism. If the plateau is bound by latency rather than bandwidth, two workers sharing a core can overlap each other's misses: one computes while the other waits. That is the classic case where hyperthreading helps code that waits on memory. The test: the same number of workers placed as pairs on shared cores, against one worker a core.

**Helpers on the second hardware threads.** The decoder's other threads already gain from spare CPUs. Decoding four copies of the sample with four threads took 0.405 seconds on the whole machine, against 0.535 seconds pinned. Pinned, the scanner, reader and main thread share the workers' cores. The second hardware threads could host prefetch or helper threads that bring the next block's data into cache ahead of the workers. The test: a helper arm against none, at the same worker count, with the misses per output byte counted.

**Which first.** The memory-controller run in §4 puts the plateau at 75% of the measured bandwidth ceiling, with traffic per byte rising faster than output past seven workers. That makes cheaper bytes per worker the main route: every byte saved per output byte is room for more workers before the ceiling. Latency hiding with the second hardware threads stays open, since a quarter of the bandwidth is still unused at seven workers.

## 6\. What comes next for the knee?

The other task is to make the planner portable. The measured seven-worker choice is not a general law. The cache model under test correctly recovered some counts on this machine, then predicted the wrong optimum for smaller bzip2 blocks. Other processors have different core counts, cache sizes, cache-sharing domains and memory systems. The right worker count needs to follow those facts, with tests that can prove a proposed rule wrong.

For now the release result is a good one, with a precise boundary. **mibz2 no longer punishes a request for more threads with a large fall in throughput on this measured workload and machine.** In the release sweep the penalty past the best point is 4–5%, down from 11–18%. With placement held constant, no decline is seen. It gets there by doing *less work in parallel*, more efficiently. The old knee exposed a bad trade-off between concurrency and interference. Version 0.5.4 runs on the better side of it.

What remains is a flat stretch past the cores to turn into a rising curve, a ceiling to raise, and a policy to generalise. That is a more useful problem than a curve that falls whenever we give it another core.

---

**Research records:**

- [The original eight-thread scaling investigation](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-07-why-mibz2-stops-scaling/index.md>), [lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-16-lazy-letters/index.md>), and [the cache-model post](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/index.md>) with its [technical notes](<https://lmmx.github.io/mibz2/pr-preview/pr-692/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/index.md>)
- [0.5.3's release-scaling run and the pinned ten-thread diagnosis](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6946756938>), [the unpinned control](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6949008682>), [the knee diagnostic](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6948385717>) and [0.5.4's release-scaling run](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6952656453>)
- [The worker-plan fix](<https://github.com/lmmx/mibz2/pull/537>), [the once-per-process machine read](<https://github.com/lmmx/mibz2/pull/545>), and the [release attribution ledger](<https://github.com/lmmx/mibz2/blob/745a7381ab72ca07675ddfcb000c4156f926c0eb/docs/journal/release-attribution.md>)
- The recorded runs: [release scaling](<https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling>), and on the `evidence` branch the knee diagnostic (`knee/run-20261117T123939Z`), the unpinned control (`unpinned/run-20261117T130353Z`), the counters runs (`miss-cost/run-20261117T104648Z`) and the memory-controller runs (`cells-imc/run-20261114T180428Z` and `knee/run-20261117T230729Z`)
