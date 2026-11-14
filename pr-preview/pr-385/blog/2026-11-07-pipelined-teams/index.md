[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/index.md>)

# Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores

While a team's helpers invert one block, its leader finishes the last and prepares the next: 7% faster than 0.2.5 on ten cores, for 5% more processor time.

7 November 2026 · 13 min read · [Research result](<#7-where-do-things-stand>)

mibz2, our bzip2 decoder written in Rust, decodes a file with small groups of threads that we call teams. A bzip2 file is a series of blocks that can be decoded independently, and each team decodes one block at a time. One thread in the team, the leader, does some of each block's work alone, and the team's other threads, its helpers, wait for it. That waiting is time the decoder could spend decoding. Our new release, 0.2.6, removes much of it.

In 0.2.6, a team works on three blocks at once. While its helpers do the expensive middle part of one block, the leader finishes the block before it and prepares the block after it. On the ten-core machine we test on, with ten threads, this is 7.2% faster than 0.2.5 (95% interval 5.4% to 8.9%), and 6.5–7.0% faster at 11 to 20 threads. It is not cheaper: it uses about 5% more processor time in total, and 7–8 MB more memory at its peak.

That it helps at all is not obvious. In 0.2.5 the leader takes part in the expensive middle part. Pipelining takes it out, so that part runs on one thread fewer and gets slower. The 7% is what is left after paying for that. The first version of the pipeline gained only 4%, for 9% more processor time. Two refinements made it worth shipping: the leader keeps the pipeline full after a small block, and threads that wait go to sleep instead of staying on the processor.

The [previous post](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) explained why 0.2.5 split ten threads into three teams of three, and proposed this pipeline as the next experiment. This post is its result. [Section 1](<#1-what-does-a-team-do-with-one-block>) shows what a team does with one block, and [section 2](<#2-what-does-pipelining-change>) what pipelining changes and why it might not pay. [Section 3](<#3-what-did-the-first-pipeline-gain-and-why-so-little>) is the first pipeline's measurement, [section 4](<#4-what-did-the-two-refinements-change>) the two refinements, and [section 5](<#5-did-the-new-default-hold-up-against-025>) the run that confirmed the release against 0.2.5. [Section 6](<#6-what-does-it-cost-and-where-does-it-apply>) covers the cost and where it applies.

## 1\. What does a team do with one block?

bzip2 compresses a file in blocks of up to 900 kB each, each compressed with no reference to the others, so each can be decoded on its own. One thread, the scanner, reads the file and undoes the outermost layer of every block, the Huffman coding. The rest of a block's work is done by a team, in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which bzip2 used to reorder the block's bytes so that similar ones sit together. This puts them back in their original order.
3. **The finish:** RLE1, which expands the runs of repeated bytes that bzip2 shortened first of all, and the CRC, which checks the result against the block's checksum. Then the block's output is handed on, in file order.

The inverse BWT is the expensive part, and the only part a team shares. It has two phases. The **build** fills a table with one 4-byte entry for each byte of the block, saying where the next byte of the original text is. For a full block this table, which we call the T-vector, is 3.6 MB. The **walk** then follows the table from entry to entry, writing out a byte at each step. Each step's address comes from the entry just read, so mibz2 starts many walks at known points through the block, and the team's threads share them. [An earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>) works the transform through on a small example.

In 0.2.5, the **leader** does the block's MTF alone, then all three threads build and walk the table, then the leader puts the walk's pieces together and does the finish alone. The two **helpers** take part only in the build and the walk, and wait the rest of the time. In traces of four teams, the time outside the inverse BWT was 29–33% of a block's decode ([the previous post, section 4](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-07-why-mibz2-stops-scaling/#4-why-dont-more-threads-on-one-block-help>)). So for roughly a third of each block, a team's helpers had nothing to do.

The test machine is an Intel Core i9-10900K, with 10 cores (20 threads) and a 20 MiB last-level cache shared by all of them. On a machine of that shape, given ten or more threads, 0.2.5 runs three teams of three, nine decoding threads in all, plus the scanner. We write a plan as teams × threads per team, so this one is 3 × 3.

## 2\. What does pipelining change?

**The idea:** have the leader do its lone work while the helpers do the shared part, instead of before and after it. In 0.2.6, while the two helpers build and walk block A, the leader finishes the block before it, Z, and runs MTF on the block after it, B. When it has done both, it joins the helpers in walking A. When A's walk ends, the helpers start building B, whose MTF is already done, and the leader finishes A. The figure shows the two layouts side by side.

A team of three, in 0.2.5 and in 0.2.6

A sketch of the stages and who runs them, from the decoder's code; bar lengths are illustrative, not measured. L is the leader and H a helper. Blue is the leader's lone work, green the shared build and walk, grey a thread waiting. "Finish" is RLE1 and the CRC, and in 0.2.5 also putting the walk's pieces together; in 0.2.6 the last thread out of the walk does that.

We call this **overlap**: a block's MTF or finish runs at the same time as its team's build or walk of another block. It needs a second set of MTF buffers and a second buffer for an assembled block in each team, about 3 MB a team.

**Why it might not pay.** The build and the walk lose a thread. In 0.2.5 three threads build each table. In 0.2.6 two do, and the leader joins only the walk, and only once its own work is done. So the shared part gets slower, and the question is whether the time saved by overlap is larger than the time lost there. The previous post put that in a simple model, and found that if the shared part ran about 1.5 to 1.6 times slower without the leader, the gain would be zero ([section 7](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-07-why-mibz2-stops-scaling/#7-what-will-we-try-next>)). Nothing measured then said how much slower it would run.

## 3\. What did the first pipeline gain, and why so little?

We tested the first version of the pipeline against the plain teams at the same plan, 3 × 3 on ten threads. The test was registered: its comparisons, its possible outcomes and what each would mean were fixed before the run. Each arm decoded a sample of the Wikidata dump, 11.85 MB of bzip2 that decodes to 209.7 MB, 20 times, the arms taking turns in each round. "Speed" here and below is the plain arm's elapsed time over the pipelined arm's, paired round by round, so above 1 means faster. Its 95% interval describes the spread across the 20 rounds of one run, on one machine and one corpus.

The pipeline was faster: 1.042 (1.016–1.068). But it used about 9% more processor time, the time all threads spent running added up (a paired ratio of 0.918, 0.895–0.942). In the model's terms, the result fits a shared part running about 1.4 to 1.55 times slower without the leader, depending on what the hand-offs between threads are assumed to cost. That is a fit, not a measurement, and it is close to the break-even point.

The traces showed why the gain was small. **Only about a third of blocks overlapped:** in 147 of the sample's 431 blocks, the leader's MTF ran during its own team's build or walk of another block. Two features of the first version kept the rest from overlapping.

**Small blocks emptied the pipeline.** A block of fewer than 262,144 bytes (256 KiB) is not worth sharing among three threads, so its team's leader decodes it whole, on one thread. 198 of the sample's 431 blocks are that small. In the first version, the leader took at most one new block while its helpers worked. If that block was small, it decoded it and prepared nothing else. The next block then had to be prepared with the helpers waiting, as in 0.2.5. Each small block cost about one block's overlap.

**Waiting threads never slept.** A thread waiting for the build to end (a helper that had finished its share, or the leader waiting to join the walk) checked a flag 4,096 times, then asked the operating system in a loop to run something else instead (a yield). When nothing else is waiting to run, a yield returns at once, so the waiting thread stays on its core and its wait counts as processor time.

## 4\. What did the two refinements change?

Each refinement was tested on its own against the first pipeline, at 3 × 3 on ten threads and at 3 × 4 on twelve, with its question and outcomes fixed before the run. The figures below are the ten-thread ones; the twelve-thread results agree.

**Small: the leader keeps the pipeline primed.** After a small block, the leader now takes the next queued block too, and runs its MTF in the same stretch. It keeps going until it has prepared a block for its team, or finds none waiting. It never waits for a new block to arrive: a leader that held an unfinished block while waiting for the scanner could leave the two waiting on each other.

A small block in the pipeline, before and after the change

A sketch, not a measurement: bar lengths are illustrative. L is the leader, H the two helpers, and S a small block, which the leader decodes whole. After the change the helpers can wait a little at the end of a run, while the leader finishes B's MTF.

It worked as intended. Overlapped blocks rose from 144 to 236.5 of 431 in this run (medians over six traced rounds; the first run counted 147), and the pipeline got faster: 1.043 (1.020–1.066), with no change detected in processor time, 1.007 (0.987–1.028). It has a cost, which the figure shows: a leader busy with more blocks joins the walk later. Of the 233 blocks large enough for the team to invert together, those whose walk the leader joined late rose from 62 to 113. The registered outcomes had expected a later join to cancel the gain, and had no entry for a later join with a gain. Our reading, not a registered verdict, is that the later join is real and paid for, and that the extra overlap outweighs it, on this machine and corpus.

**Park: waiting threads go to sleep.** A waiting thread still checks its flag 4,096 times, then, instead of yielding, it adds itself to a list and parks: it sleeps until woken, using no processor time. The thread that ends the build wakes every thread on the list. Waking a parked thread takes a few microseconds, which a thread that never slept doesn't pay.

Parking left the elapsed time unchanged, 0.999 (0.981–1.017), and cut processor time by about 5% (a ratio of 1.055, 1.039–1.071). The saving could have come from spinning less before parking, rather than from parking. A control arm that only cut the checks from 4,096 to 2,000, still yielding, changed neither figure: 1.004 for speed and 1.006 for processor time, both intervals holding 1. So the saving is from parking.

Each change against the first pipeline, 3 × 3 on ten threads

Measured, the test machine, 20 paired rounds each, with 95% intervals. Both ratios are the first pipeline's figure over the changed one's, so above 1 is better: faster, or less processor time. Green dots are intervals wholly above 1. The spin control cuts the checks before a yield, and doesn't park.

**Both together** were about as fast as small alone, 1.038 (1.021–1.055), and saved less processor time than parking alone, 1.032 (1.018–1.046) against 1.055. A likely reason is that a busier leader leaves less waiting to sleep through, but that is not established.

**Against the plain teams,** in the same run, the pipeline with both changes was 1.072 (1.053–1.091) as fast as plain 3 × 3, for about 5% more processor time (a paired ratio of 0.951). Its peak memory was 75 MB, against 67 MB for the plain teams of 0.2.5's default. That is the version 0.2.6 ships.

One thing these runs can't do is split the gain into its causes. Some of it may come from the overlap, and some from the way the leader joins the walk as an extra thread once its own work is done. The runs measure the total.

## 5\. Did the new default hold up against 0.2.5?

The 1.072 came from a comparison that was reported, not registered as a decision. So before release we fixed one more test, with its pass rule set before any run: the 0.2.6 build against the 0.2.5 release, each decoding as a caller gets it by default, at 8, 9, 10, 11, 12, 16 and 20 threads, 20 paired rounds each. To pass, ten threads had to be at least 1.03 as fast, with its interval above 1. At 11 to 20 threads, each count's interval had to rule out being more than 2% slower. At 8 and 9 threads both builds run four teams of two, unchanged, so those two counts check that nothing else moved. The prediction was 1.07–1.08 at ten threads, about the same above, and no change at 8 and 9.

The run passed. The table gives each thread count's plan in 0.2.6 and its speed against 0.2.5. "CPU" is 0.2.6's processor time over 0.2.5's, and "memory" its peak memory over 0.2.5's, each from the medians of the 20 rounds. 0.2.5 ran the same plans, unpipelined:

| threads | plan | speed (95%) | CPU | memory |
| ---: | --- | ---: | ---: | ---: |
| 8 | 4 × 2 | 1.002 (0.991–1.013) | 0.99 | 1.00 |
| 9 | 4 × 2 | 1.001 (0.991–1.012) | 1.00 | 1.00 |
| 10 | 3 × 3, pipelined | **1.072 (1.054–1.089)** | 1.04 | 1.10 |
| 11 | 3 × 3, pipelined | 1.070 (1.055–1.085) | 1.05 | 1.12 |
| 12 | 3 × 3, pipelined | 1.069 (1.054–1.085) | 1.05 | 1.10 |
| 16 | 3 × 3, pipelined | 1.065 (1.045–1.085) | 1.05 | 1.13 |
| 20 | 3 × 3, pipelined | 1.070 (1.048–1.092) | 1.04 | 1.12 |

0.2.6 against 0.2.5, by the threads asked for

Measured, the test machine, 20 paired rounds a thread count, with 95% intervals. Speed is 0.2.5's elapsed time over 0.2.6's, so above 1 is faster. Green dots are intervals wholly above 1.

From the run's records, at ten threads the median decode of the sample took 0.144 s against 0.156 s, about 1,460 against 1,350 MB/s of output. Processor time rose from 0.98 s to 1.02 s, and peak memory from 69 to 76 MB. The outcome matched the prediction for speed and peak memory (1.10–1.13 against about 1.12), and came in a little under it for processor time (1.04–1.05 against 1.05–1.06). Above ten threads the plan is the same three teams, and the measured gain is about the same. A request for twenty threads still decodes on nine, plus the scanner.

## 6\. What does it cost, and where does it apply?

**The trade-off.** 0.2.6 finishes sooner, but it doesn't do less work: it spends about 4–5% more processor time and 7–8 MB more peak memory than 0.2.5 on this workload. Elapsed time improves because work that used to run while two threads waited now runs beside them. The extra memory is each team's second set of MTF buffers and second assembled block.

**Where it applies.** Only the 3 × 3 plan is pipelined, and the decoder picks that plan only on a machine with 10 physical cores and a 20 MiB last-level cache, given 10 or more threads. Every other plan and every other machine shape decodes exactly as 0.2.5 did. The output is the same bytes in every case: the pipelined path is tested byte for byte against libbzip2, and on corrupt and truncated input, by forcing the plan on any machine. But its speed has been measured on one machine, the test machine, and one corpus, Wikidata. Another processor of the same shape gets the pipeline too, measured or not. The cloud machines that benchmark every change to mibz2 have 4 virtual CPUs on 2 physical cores, so they never take this path. They show only that nothing else moved. Thread counts of 13 to 15, 17 to 19 and above 20 were not measured, though they run the same plan.

**The plain teams are still available.** A caller that prefers lower processor time can ask for 0.2.5's three plain teams directly, with [`BzReader::parallel_planned`](<https://docs.rs/mibz2/0.2.6/mibz2/struct.BzReader.html#method.parallel_planned>) and the plan [`Plan::Teams`](<https://docs.rs/mibz2/0.2.6/mibz2/parallel/enum.Plan.html>):

```rust
use mibz2::{parallel::Plan, BzReader, Kernels};

// three plain teams of three, as in 0.2.5
let reader = BzReader::parallel_planned(
    file,
    Plan::Teams { teams: 3, k: 3 },
    12, // blocks in flight: the default's window at this plan
    Kernels::detect(),
);
```

## 7\. Where do things stand?

The argument runs like this. In 0.2.5, a team's two helpers waited through about a third of each block, while the leader did MTF and the finish alone. 0.2.6 has the leader do that work while the helpers build and walk another block. That takes the leader out of the build, which slows it, so the first version gained only 4%, for 9% more processor time. Two changes made it pay: preparing the next block after a small one raised overlap from about a third of blocks to about half, and putting waiting threads to sleep took back about half of the extra processor time. A registered run of the release against 0.2.5 confirmed 7.2% at ten threads, and 6.5–7.0% at 11 to 20.

0.2.5's three teams were 8.8–8.9% faster than 0.2.4's four teams of two at ten threads, and 0.2.6 is 7.2% faster than 0.2.5. Multiplied together, the two would put 0.2.6 about 17% ahead of 0.2.4 there. That is a projection from two separate runs, not a measurement: a direct comparison of the releases has not yet run.

What remains open is in two places. The previous post's question, why each extra block in progress slows the others, is unchanged by this release. And the pipeline still leaves the leader's serial work inside each team. The next question is whether moving that work onto threads of its own, outside the teams, would do better on the same number of cores ([pull request 251](<https://github.com/lmmx/mibz2/pull/251>)). Nothing about it has been measured yet.

The general lesson: when part of a job can't be shared, it can still run beside the part that is. Whether that pays depends on what the shared part loses to make room, and here the answer was found only by measuring.

---

**Research records:**

- [The first pipeline: its result](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-team-pipeline-result.md>) and [its registration](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-team-pipeline-registration.md>)
- [The two refinements: their result](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipeline-levers-result.md>) and [their registration](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipeline-levers-registration.md>)
- [The confirmation against 0.2.5: its registration](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipelined-default-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/0b8650970fbb2d61360a98f6b694a55c46f62d80/pipelined-default/20261107T105448Z.tar.gz>)
- [The 0.2.6 changelog](<https://github.com/lmmx/mibz2/blob/838890ab/CHANGELOG.md>)
- [The three-team policy, 0.2.5 against 0.2.4](<https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-06-three-team-policy.md>)
