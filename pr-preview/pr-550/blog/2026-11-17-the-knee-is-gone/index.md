[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-550/blog/index.md>)

# The knee is gone: how mibz2 stopped losing speed to more threads

Four releases that run fewer workers than threads have smoothed mibz2's slide past eight threads into a level plateau near 2 GB of output a second.

17 November 2026 · 8 min read · [Research result](<#figures>)

Asking mibz2 for more threads no longer makes it slower. mibz2 is our bzip2 decoder, written in Rust. A bzip2 file is a series of blocks that decode independently, so mibz2 decodes several blocks at once, on as many threads as it is given. On 7 November, [Why mibz2 stops scaling at eight threads](<https://lmmx.github.io/mibz2/pr-preview/pr-550/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) reported that on our test machine the decoder got faster up to eight threads and no faster after. That point, where a scaling curve bends from rising to flat, is its *knee*. The post traced part of the cause to the blocks in progress competing for the processor's shared cache.

By mibz2 0.5.0, released on 16 November, the knee had become a fall. On the test machine, a ten-core Intel desktop (an i9-10900K, with 20 logical CPUs and a 20 MiB level-3 cache shared by all ten cores), 0.5.0 took 16–18% longer to decode the same file with 12, 16 or 20 threads than with 8. Four releases since, 0.5.1 to 0.5.4, have removed that fall. From 8 to 20 threads, 0.5.4 decodes between 1.95 and 2.06 GB of output a second, fastest at ten threads.

*Draft in progress: the body's prose follows in further commits. The figures below are final drafts.*

## Figures

Throughput against threads, five releases on the test machine

Measured on the test machine: the median of 10 paired rounds per point, decoding an 11.9 MB extract of a Wikidata dump to 209.7 MB of output. Each release is taken from its own [release-scaling run](<https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/>) (0.2.4 from 6 November; 0.5.0, 0.5.1 from 16 November; 0.5.3, 0.5.4 from 17 November). The runs pin 10 threads or fewer to one CPU a core, and give 12 threads or more the whole machine.

What mibz2 runs at each thread count, release by release

From the decoder's thread names, recorded in each release's release-scaling run, and the release notes. A team (amber) shares one block between several threads; a worker (green) decodes a whole block on its own. A thick border marks a cell that changed from the release before. Where a cell has two counts, the first is the plan when the process is pinned one CPU a core, which is how the release-scaling runs measure 10 threads or fewer, and the second the plan on the whole machine.

The fall past eight threads, release by release

Measured on the test machine. Each mark is a release's wall time at 10, 12, 16 or 20 threads divided by its own wall time at 8 threads, as the geometric mean of 10 rounds, paired by round, with its 95% interval across those rounds (one run, one machine, one file). Right of the axis is slower than 8 threads. 0.4.0 comes from 0.5.0's run, where it was the older release. 0.5.3's 10-thread mark is its pinned plan of 10 workers. Sources: the [release-scaling runs](<https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/>).

More workers, more memory traffic for every byte decoded

Measured on the test machine, decoding four copies of the Wikidata extract. Green: reads that missed the L3, counted as 64-byte lines, per byte of output, in the [knee diagnostic](<https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T123939Z.tar.gz>) of 17 November (the whole machine, except the hollow point, pinned). Amber: reads and writes counted at the memory controller, on an older decoder that held a slot for every letter of a block ([memory-controller run](<https://github.com/lmmx/mibz2/blob/evidence/cells-imc/run-20261114T180428Z.tar.gz>), 14 November). The two runs used different decoders and counters, so they are drawn side by side, not joined. The analogy stops where caches differ from kitchens: data is not used up, it is copied in 64-byte lines, and whether the pantry door is the limit is the open question.

At the plateau: how close is the road to memory to full?

Decoded output and reads that miss the L3 are measured (knee diagnostic, 7 workers, four copies of the Wikidata extract, the whole machine), and so is the read ceiling, a separate test that streams memory as fast as it can. Reads plus writes is an estimate: the reads scaled by the older decoder's ratio of writes to reads at 8 to 10 workers, 0.74 to 0.78. The 51.2 GB/s line is an assumption: it is the peak of two channels of DDR4-3200, and the test machine's memory rating is not recorded. Only a reading of the memory controller's counters at the plateau settles whether memory is full.
