[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/index.md>)

# Fast alone, slow in a crowd: which speed-ups survive many cores

Walking each block in eight lanes made our newest decoding loop 13% faster on one thread, yet 5% slower once eight workers shared the processor.

21 November 2026 · 12 min read · [Research result](<#6-what-comes-next>)

Shipped in mibz2 [0.5.9](<https://lmmx.github.io/mibz2/pr-preview/pr-790/benchmarks/history/#release-0-5-9>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#059---2026-11-21>))

[Sixteen at a stroke](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/2026-11-21-sixteen-at-a-stroke/index.md>) described the one change in mibz2 0.5.8. Inside Open, the decoder's newest way of undoing the Burrows–Wheeler transform (BWT), short copies of varying length became fixed sixteen-wide writes, so the processor no longer has to guess where each copy ends. Open got about a fifth faster on one thread, and the whole decoder 5% faster at ten threads on the i9-10900K, the ten-core desktop processor we test releases on. This post follows that change, and the candidate meant to succeed it, from one thread to the whole machine.

**The setting, briefly.** bzip2 cuts a file into blocks of up to 900 kB, and mibz2 decodes several blocks at once, one per *worker* thread. Undoing each block's BWT is most of the work, and mibz2 has two ways to do it. Cells, the older, is the faster for a lone worker. Open keeps only the rows of a block's narrow runs, about a tenth of a Wikidata block, and lets the rest travel in groups ([Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/2026-11-19-open-returns/index.md>)). It does more arithmetic than Cells but holds less in memory at once, and that pays when many workers share the processor's caches ([Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/2026-11-16-lazy-letters/index.md>) measured why). So mibz2 runs Open only with many workers: from 8 on most machines, and on the i9 at 6 threads and from 8 up.

**Why the question matters.** A speed-up is cheap to measure on one thread. Production runs Open only when the cores are crowded, sharing one cache and one path to memory, and a gain that holds for a lone worker can shrink or reverse once every core competes for those.

**What we found.** Four runs on the i9 answer the question for 0.5.8 and its successor:

- **0.5.8 keeps its gain on the whole machine.** It is 7% faster than 0.5.7 at 8 threads and 3–5% faster from 10 to 20, with less processor time. At 1, 2 and 4 threads nothing changed, because Open doesn't run there.
- **Splitting one block across a team of threads still never pays.** Teams of two or three were slower at every thread count from 4 to 20.
- **0.5.8 moved one of the decoder's choices.** At exactly 4 threads, Open with four workers now beats Cells by 5.5%. Our cost model had called that cell a near-tie.
- **The successor won alone and lost in a crowd.** It walks each block in eight interleaved lanes, and it made Open 13% faster on one thread, as large a gain as 0.5.8's own when the two are timed side by side. On the i9 it was 5% faster with 6 workers, level with 7 and 5% slower with 8, so it was not adopted.

Alone and in a crowd: two changes to Open, by how many Open workers run at once

Measured. The first column is Open's kernel time alone, on one thread in a cloud container, over the whole Wikidata sample: 0.5.8's change is the inverse of the build before it at 1.128 (1.074, 1.184), and the eight-lane candidate 0.872 (0.820, 0.928) of 0.5.8. The other columns are the whole decoder's wall time on the i9, 16 paired rounds each, at 6, 10 and 8 threads, where production runs 6, 7 and 8 Open workers. The two kinds of measurement differ, so read each line's slope, not the step across the dashed divider.

**Why the larger one-thread gain lost.** Open's last step is a walk through a table, and each entry it reads names the next one. A single walk therefore can't start a lookup until the previous one has come back from memory, and while it waits, most of the memory system's capacity sits idle. The eight lanes cut the walk into pieces and step through each in turn, so up to eight lookups are in flight at once. On one thread that capacity was free. With eight workers on the i9 it is shared, and the lanes plausibly compete for it, along with the wider tables the candidate also brought. That is our account, not a measurement ([section 4](<#4-why-did-a-13-one-thread-win-lose-on-eight-cores>) says what would test it). 0.5.8's change instead removed instructions and wrong guesses, work each core does on its own. Its gain also shrank as workers were added, but it never reversed.

[Section 1](<#1-how-did-058-do-on-the-whole-machine>) gives 0.5.8 against 0.5.7 across the machine, and [section 2](<#2-do-teams-of-threads-win-anywhere-yet>) the check on teams. [Section 3](<#3-did-058-change-how-many-workers-the-decoder-runs>) shows the one choice that moved, beside its prediction. [Section 4](<#4-why-did-a-13-one-thread-win-lose-on-eight-cores>) takes the eight lanes from one thread to eight, and [section 5](<#5-which-one-thread-wins-have-survived>) sets each recent candidate's result alone beside its result in a crowd. [Section 6](<#6-what-comes-next>) says what comes next.

All the i9 figures come from one machine: an Intel i9-10900K with 10 cores and 20 hardware threads, 256 KiB of level-2 cache per core and 20 MiB of level-3 (L3) cache shared by all ten. The workload is the benchmark's Wikidata sample, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB.

## 1\. How did 0.5.8 do on the whole machine?

Before each release we time it against the last one on the i9, from 1 thread to 20. Each of ten rounds runs both releases back to back, and their ratio is taken within the round, so slow drift in the machine cancels out. The 95% intervals describe the spread across those rounds, on this one machine and corpus.

0.5.8 against 0.5.7 on the i9, by thread count

Measured on the i9-10900K, decoding the Wikidata sample, 10 paired rounds per count. The run's summary records 0.5.7's time over 0.5.8's; the figure shows the inverse, so that below 1 means 0.5.8 is faster. In the upper band the decoder runs Cells only, so 0.5.8's change can't reach it; in the lower band it runs Open.

**Below 8 threads, no change was detected,** as the decoder's routing predicts. At 1 thread mibz2 decodes serially with Cells, and at 2 and 4 threads it runs that many workers, still with Cells, so 0.5.8's change to Open never executes. Open starts at 8 workers, or at 6 threads on the i9, a count this grid doesn't include. [The stage timing](<https://github.com/lmmx/mibz2/blob/bb98c9958c5d5158e3d23d1da1939b747eef6391/docs/journal/2026-11-20-open-stage-attribution.md>) traced the routing, and it also explains why CI's benchmark never saw the gain: CI times 1 to 4 threads, where Open never runs.

**From 8 threads, every interval lies below 1.** At 8 threads 0.5.8 took 0.934 of 0.5.7's time (0.925, 0.943). In the run's own terms, 0.5.7 took 1.071 times as long, and 1.052 times at 10 threads, the default. Processor time fell with the wall time, by 3–7%.

The gain is largest at 8 threads, where the decoder runs 8 Open workers. From 10 threads up it runs 7, the i9's *trim*: fewer workers than threads, which finishes sooner on this machine ([Modelling the cache to rightsize workers](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/index.md>)). The decoder's wall time hardly moves from 8 threads to 20 (0.079 to 0.084 seconds), so threads past 8 buy no more speed on this machine.

The change's own i9 test, on four copies of the sample end to end, had measured 0.974 at 8 threads and 0.947 at 10. The two runs agree in direction, though at 8 threads they differ by more than their intervals, which cover each run's own rounds only. Peak memory rose by 2–6%, on peaks of about 100 MB, because the new queues stay allocated between blocks. At 2 threads, where nothing changed, the peaks also differ by 2 MB, so part of that is noise. For scale, lbzip2, the established parallel bzip2 decoder, took 6.3 times as long as 0.5.8 at 8 threads.

## 2\. Do teams of threads win anywhere yet?

A *team* is several threads that cooperate on one block, instead of one worker per block ([mibz2 gets a pit crew](<https://lmmx.github.io/mibz2/pr-preview/pr-790/blog/2026-11-09-pit-crew-service-threads/index.md>)). Teams would pay if blocks became slow enough, or the machine wide enough, that cooperating beat working independently. Each release checks that threshold on the i9 at ten thread counts from 4 to 20, with ten paired rounds each. Each figure in the table is that decoder's wall time over the shipped decoder's, so above 1 means slower:

| threads | teams of 2 | teams of 3 | one block per thread, untrimmed |
| ---: | ---: | ---: | ---: |
| 4 | 1.374 | 2.007 | 1.016 |
| 6 | 1.438 | 1.664 | 1.195 |
| 8 | 1.630 | 1.888 | 1.061 |
| 10 | 1.974 | 1.599 | 1.054 |
| 12 | 2.184 | 1.720 | 1.124 |
| 16 | 2.674 | 1.981 | 1.135 |
| 20 | 3.149 | 2.242 | 1.160 |

The counts not shown, 5, 7 and 9, follow the same pattern. The closest any team came was teams of two at 7 threads, at 1.213 (1.190, 1.235). The shipped choice also beat one worker per thread without the trim, or matched it within noise, at every count. Teams stay unused.

## 3\. Did 0.5.8 change how many workers the decoder runs?

On the i9 the decoder doesn't simply run one worker per thread. For each thread count, and for whether the program is pinned to particular CPUs or free to use all twenty (*unpinned*), it has a measured choice of worker count and method. A release that changes what a block costs may move those choices, so each release re-measures them. There are 26 cells, 12 pinned and 14 unpinned, and each compares the shipped choice with its neighbours: a worker fewer or more, or the other method. The main corpus is four copies of the Wikidata sample, with generated text and random bytes as guards. Testing many cells at once invites a win by chance, so the rule applies Holm's correction across them before calling any gain.

**One cell moved.** Unpinned at exactly 4 threads, Open with four workers took 0.945 of the time Cells did (0.923, 0.969), with a corrected p of 1.7 × 10⁻⁴. No change was detected on either guard, and its peak memory was 0.927 of Cells'. Every other cell kept its shipped choice. [\#678](<https://github.com/lmmx/mibz2/pull/678>) makes the change for the next release.

Before the run, our cost model froze its predictions. It is a contention model, fitted to the i9's earlier runs, of how worker counts compete for the shared cache and the processor. It predicted that no choice would change, and it named this cell a near-tie at 1.02 (0.97, 1.06). 0.5.7's run had measured Open at 1.073 of Cells here, and 0.5.8's cheaper Open, it reasoned, would close most of that gap.

One cell, three readings: Open against Cells at 4 threads, unpinned

Measured (solid) and predicted (hollow, dashed), on the i9 with four copies of the Wikidata sample, 12 paired rounds per run. The prediction was frozen before 0.5.8's run. The lower interval lies wholly below the predicted one.

**The miss is in one input, not in the model's structure.** For 0.5.8's gain at 4 workers, the model assumed 0.92 to 0.97, since only the 6-thread figure, 0.922, had been measured. Dividing the two runs' readings, 0.945 by 1.073, puts it nearer 0.88, a figure derived from two separate runs rather than measured. The model had flagged that input as having no basis, and its own scoring rule counts a miss there against the input, not the model. The miss anticipates section 4: a kernel's gain at one worker count is not its gain at another, and here it was larger with fewer workers.

## 4\. Why did a 13% one-thread win lose on eight cores?

The candidate came from our research line's C harness, which tests ideas on Open's kernel in isolation before they are ported to the decoder ([\#675](<https://github.com/lmmx/mibz2/pull/675>)). It changes two things.

**The first is the walk in eight lanes.** Open's last step follows its table from one kept row to the next, writing each row's word of text as it goes. Every step needs the entry the previous step pointed to, so the walk is a chain of dependent lookups. When an entry isn't already in cache, the core waits for it, and nothing else in the walk can proceed. The candidate cuts the chain into dozens of pieces, marking each cut so that a lane can tell where its piece ends, and walks eight pieces at once, one step of each in turn. While one lane waits for memory, the other seven do their steps.

Why eight lanes help a lone worker: waits that overlap

Conceptual timing, not measured values, and idealised: it assumes every lookup misses the cache and that memory can serve eight at once. In practice most lookups hit, and the walk is only part of Open's time, so the gain on one thread was 13%, not eightfold.

The pages add work of their own. Each lane's output goes into 4 KiB pages of output space not yet written, or of an overflow area past its end, and the pages are moved into order once the walk ends.

**The second is a cheaper way of building the table, with a wider cut-off.** The construction, a new elimination rule that the research line calls R4, stores only a backward link per entry and reuses a field that is no longer needed for coordinates, so it needs one array fewer. It also writes short words, up to 16 bytes, straight into the table instead of a separate store. Alongside it, Open keeps runs of up to 24 rows instead of 16, so it keeps more rows, and its queues are filled 24 rows at a time.

**On one thread, natively, the pair made Open 13% faster.** In the Rust decoder, on a cloud container, over the whole Wikidata sample, Open took 0.872 of 0.5.8's time (0.820, 0.928). That is eight interleaved runs of each build, paired by run. The lanes alone gave 0.917, and R4 with the wider cut-off 0.925. For comparison, the same timing puts 0.5.8's own change at 0.887 of the build before it (the last post's "about a fifth" came from an earlier set of runs). The candidate also made Open 12% faster than Cells on one thread, where 0.5.8's Open was only 3% faster. Indicative runs of the whole decoder on the same shared container, with Open forced on, already showed the gain thinning: 0.914 at 1 worker, 0.924 at 2 and 0.972 at 4, the last within noise.

**The rule for the i9 was registered before any data.** The test ran production's decoder, unpinned, at 6, 8 and 10 threads, where it runs 6, 8 and 7 Open workers, on four copies of the sample with 16 paired rounds. The candidate would be adopted only if 10 threads, the default, were faster beyond noise, neither 6 nor 8 slower, and peak memory within 10% everywhere. The registration itself expected the whole-decoder gain to be "smaller with more workers sharing memory bandwidth" ([the registration](<https://github.com/lmmx/mibz2/blob/6173c14b2eafb1c69404f020c8ca80fcbc1412d1/docs/journal/2026-11-21-open-lanes-registration.md>)).

The results, as the candidate's time over 0.5.8's, with 95% intervals over the 16 rounds:

| threads | Open workers | wall time | processor time | peak memory | step |
| ---: | ---: | ---: | ---: | ---: | --- |
| 6 | 6 | 0.950 (0.944, 0.955) | 0.953 | 1.019 | faster |
| 8 | 8 | 1.049 (1.023, 1.075) | 1.009 | 1.036 | slower |
| 10 | 7 | 1.012 (0.991, 1.033) | 0.980 | 1.010 | no change detected |

Ten threads was not faster and eight was slower, so the verdict was not adopted. A diagnostic run with the worker count fixed agreed: 0.946 at 6 workers and 1.044 at 8. Read by worker count, the results make one sequence, 0.950 with 6, 1.012 with 7 and 1.049 with 8, and each worker added took away more of the gain.

**What plausibly happened.** On one thread, the lanes used capacity the memory system had spare: it can serve many outstanding requests at once, and a single dependent walk asks for one at a time. With eight workers each walking eight lanes, up to 64 lookups compete for the same L3 cache and the same memory bandwidth, alongside every other part of eight decodes. Capacity that was idle for one worker is already in use. The candidate also widened each worker's working set. Its wider cut-off keeps more rows, and on the six Wikidata blocks examined most closely its overflow pages took 184 to 216 KiB per worker, close to the 256 KiB of level-2 cache each core has. Both effects would plausibly push more data out to the shared L3, where every worker meets.

Fast alone, slow in a crowd: one doorway, shared by every worker

Conceptual, a plausible account and not a measurement. The doorway stands for the L3 cache and the path to memory that all ten cores share. The analogy stops holding in two places: a cache serves many requests at once up to a limit rather than one at a time, and each core also has its own 256 KiB level-2 cache that no other worker touches. What it shows is the direction of the effect: capacity that one worker can borrow is already spent when eight need it.

This account fits the order of the results, and the registration foresaw its direction, but we have not counted cache misses or memory traffic on the i9 at these worker counts. That measurement would confirm or refute it. One detail it doesn't yet explain is that at 10 threads the candidate used 2% less processor time while its wall time stayed level.

## 5\. Which one-thread wins have survived?

Each candidate passes three tests in turn. The C harness times the idea on Open's kernel alone. The native test times it in the Rust decoder on one thread. The i9 test times the whole decoder with the many workers production runs. The research line keeps a record of every candidate that reached the native test ([the fidelity ledger](<https://github.com/lmmx/mibz2/blob/bb98c9958c5d5158e3d23d1da1939b747eef6391/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>)). These are its five most recent, each figure a time ratio against the Open of its day:

| candidate | C harness, one thread | native, one thread | i9, many workers | outcome |
| --- | --- | --- | --- | --- |
| fixed-width queue fill and 16-byte emit | kernel 0.83–0.86 | kernel 0.86–0.91 | 0.947 at 10 threads, with the next row | shipped in 0.5.8 |
| queues with a counter, kept at full length | 0.77–0.81, with run fusion | queue pays: front end and Open 384–398 ms, from 470–477 | 0.947 at 10 threads, with the row above | shipped in 0.5.8 |
| run fusion in the front end (move-to-front and run-length decoding) | 0.92–0.96 | adds 20–36 ms | not tested | lost natively |
| one flat array for all the queues | 0.44–0.57 | 0.990 (0.949, 1.033) | not tested | overlapped the counter queues |
| eight lanes, R4 and the 24-row cut-off | 0.43–0.57 | 0.872 (0.820, 0.928) | 0.950, 1.049, 1.012 at 6, 8, 10 threads | lost on the i9 |
| R4 alone, 16-row cut-off | (part of the row above) | about 0.95 | 0.956, 0.982, 0.985 at 6, 8, 10 threads | not adopted: 10 threads just inconclusive |

The C harness overstated every gain, partly because it measured against older versions of Open. The native test caught two of the five. Run fusion cost more in decode order than it saved, and the flat array removed a cost the counter queues had already removed. The i9 caught the third failure, the one neither earlier test could see, because both run one thread. The two changes that survived, 0.5.8's, reduce what a core computes and guesses. The one that failed on the i9 worked by keeping more memory requests in flight and by widening its tables.

## 6\. What comes next

The next candidate keeps the half of \#675 that the account above doesn't implicate. It is R4 alone, with the serial walk and the 16-row cut-off ([\#679](<https://github.com/lmmx/mibz2/pull/679>)). It keeps R4's smaller tables and drops both parts that widen the working set, the lanes and the wider cut-off. Natively it is indicative only so far: Open at about 0.95 of 0.5.8's time on one thread, faster in five of six rounds on a shared container. Its i9 test ([\#680](<https://github.com/lmmx/mibz2/pull/680>)) used the same rule as the eight lanes, registered before its data, and came back as this post was being finished. R4 alone was faster at 6 threads, 0.956 (0.946, 0.966), and at 8, 0.982 (0.967, 0.997), with less CPU at every count. But at 10 threads, the count the rule judges, it read 0.985 (0.970, 1.001): an interval that misses excluding 1 by a thousandth. Under the rule that is not adopted, and the rule isn't relaxed after the data. It is the closest call so far, and the first candidate whose gain shrank with more workers without reversing, which fits the account above.

mibz2 0.5.9 has gone out with the 4-thread change of section 3 alone.

The chain, in short, runs as follows. 0.5.8's fixed-width writes cut work each core does alone, and their gain held from 8 threads to 20, though it shrank as workers were added. The eight lanes won more on one thread by overlapping waits for memory, and lost it as the workers sharing that memory grew from 6 to 8. In between, the recalibration moved the one choice where Open's gain was larger than assumed, at 4 workers. Each result says that a kernel's gain depends on how many workers run it.

The general lesson is about where a speed-up comes from. A change that removes instructions or wrong guesses spends nothing the other cores need, so it tends to survive crowding. A change that gains by using more of the shared cache or memory system spends exactly what the other cores need, so its one-thread gain says little about many. The same test applies beyond Open. Prefetching, larger lookup tables and wider buffers all draw on that shared budget, and each would need its i9 run before its one-thread result could be trusted.

---

**Research records:** [0.5.8's release-scaling run](<https://github.com/lmmx/mibz2/tree/672fa192b35ae6f088d1909a0bc5d2ac754a4cc8/release-scaling>) and [its record on master](<https://github.com/lmmx/mibz2/tree/bb98c9958c5d5158e3d23d1da1939b747eef6391/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.8>); [the team-threshold run](<https://github.com/lmmx/mibz2/tree/06c0209e/crossover>); [the stage attribution](<https://github.com/lmmx/mibz2/blob/bb98c9958c5d5158e3d23d1da1939b747eef6391/docs/journal/2026-11-20-open-stage-attribution.md>); [the recalibration's registration](<https://github.com/lmmx/mibz2/blob/c695b930687aac71fcedcd338511f84ce330af2c/docs/journal/2026-11-21-recalibration-058-registration.md>), [its evidence](<https://github.com/lmmx/mibz2/tree/d353b473/recalibrate-058>) and its change (\#678); [the contention model's frozen predictions](<https://github.com/lmmx/mibz2/blob/e42fc32f42588a5ea23b3a0329788cc098261126/docs/journal/monodromy/2026-11-21-recalibration-058-predictions.md>); [the eight lanes, natively](<https://github.com/lmmx/mibz2/blob/8ec1a92ced6e03200561c133b16fc88e07a053d5/docs/journal/2026-11-21-open-lanes.md>) (\#675); [the i9 test's registration](<https://github.com/lmmx/mibz2/blob/6173c14b2eafb1c69404f020c8ca80fcbc1412d1/docs/journal/2026-11-21-open-lanes-registration.md>) (\#676) and [its evidence](<https://github.com/lmmx/mibz2/tree/94477725/open-lanes>); [0.5.8's own i9 test](<https://github.com/lmmx/mibz2/tree/2d6e5503c2c49db8e79d889e084da7569291ef5d/open-vec16>); [the fidelity ledger](<https://github.com/lmmx/mibz2/blob/bb98c9958c5d5158e3d23d1da1939b747eef6391/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>).
