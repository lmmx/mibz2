[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Service threads: moving mibz2's serial work out of its teams

Two teams of three, fed by two threads that only prepare and finish blocks, beat 0.2.6 by 4.5% on ten cores with 13% less processor time. It is not the default.

9 November 2026 · 15 min read · [Research in progress](<#8-where-do-things-stand>)

mibz2, our bzip2 decoder written in Rust, decodes each block of a file with a team of threads. Part of each block's work can be shared among a team's threads, and part can't: the move-to-front step before the shared part, and the finish after it, run on one thread. The [last post](<https://lmmx.github.io/mibz2/blog/2026-11-07-pipelined-teams/index.md>) showed how release 0.2.6 hides much of that one-thread work, by having each team's leader do it while the other two threads work on another block. This post is about the next step: taking that work out of the teams altogether. Two threads of their own, which we call service threads, do the one-thread work of every block, and the teams only do the shared part. On ten threads that makes two teams of three plus two service threads, where 0.2.6 runs three teams of three.

On the ten-core machine we test on, this layout decoded our test file faster than 0.2.6's default and spent less processor time doing it. In a run whose comparisons and decision rules were fixed before it ran, it was 4.5% faster (a speed ratio, the default's elapsed time over the layout's, of 1.045, 95% interval 1.017 to 1.073), on one decoding thread fewer, with 12.7% less processor time and 10 MB less peak memory. A later run, which made the same comparison beside other work without judging it, measured 1.073. Two things did not work. With only one service thread, the layout was much slower than the default, at 0.799: that one thread could not keep up, and the teams waited for it. And letting the service threads prepare more blocks ahead of the teams, three or four instead of two, did not detectably help: both settings came out inconclusive.

The figure sets these results side by side. Each dot is a speed ratio, with its 95% interval as a line; to the right of the line at 1 is faster.

The service layout against the 0.2.6 default, and the admission threshold

Measured, the test machine, ten threads, 20 paired rounds each, with 95% intervals that describe the spread within each run. Green dots are comparisons registered as decisions before their run; amber dots were reported beside them and never judged. The numbers in brackets are the pull requests of the runs. The lower panel compares the service layout with itself, at a higher admission threshold, the number of blocks the service threads prepare ahead of the teams: both intervals span 1, so neither change is shown to help or hurt.

The service layout is not the decoder's default, and is not in any release. There are three reasons. It has been measured on one processor, an Intel Core i9-10900K, and principally one file, a sample of the Wikidata dump. What causes its gain is not separated: the runs measure the layout as a whole. And an independent safety review has accepted the evidence only for the two research versions that were measured, and states that this does not qualify a version for release.

[Section 1](<#1-what-does-a-service-thread-do>) shows what a service thread does, and [section 2](<#2-why-might-it-pay-and-why-might-it-not>) why that might pay and why it might not. [Section 3](<#3-why-isnt-one-service-thread-enough>) is the first measurement, where one service thread failed, and [section 4](<#4-did-two-service-threads-beat-the-default>) the registered run with two. [Section 5](<#5-has-the-result-been-seen-again>) separates that result from the later one. [Section 6](<#6-what-still-holds-the-teams-back>) is about what still holds the teams back, and [section 7](<#7-what-does-it-cost-and-where-does-it-apply>) about the costs, the scope and what a default would need.

## 1\. What does a service thread do?

A bzip2 file is a series of blocks of up to 900 kB, each compressed with no reference to the others, so each can be decoded on its own. One thread, the scanner, reads the file and undoes the outermost layer of every block, the Huffman coding. The rest of a block's work is in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which puts the bytes back in their original order. It has two phases. The **build** fills a table with one 4-byte entry for each byte of the block, 3.6 MB for a full block, and the **walk** follows the table from entry to entry. Both can be split among several threads.
3. **The finish:** putting the walk's pieces together, RLE1 (expanding runs of repeated bytes), and the CRC (checking the result against the block's checksum). Then the block is handed to the reader, in file order.

MTF and the finish run on one thread each. The inverse BWT is the expensive part, and the only part a team shares. In 0.2.6, on a machine of the test machine's shape given ten or more threads, the decoder runs three teams of three. Each team has a leader and two helpers. While the helpers build and walk one block, the leader finishes the block before it and runs MTF on the block after it, then joins the walk.

The service layout takes MTF and the finish away from the teams. Two service threads do them for every block, serving both teams. Each team's three threads only build and walk. The figure shows the two layouts on ten threads.

Who does what, on ten threads

A sketch of the layouts, from the decoder's code; it shows which thread does which stage, not timing. Each green outline is a team. Both layouts also run the scanner and a few bookkeeping threads, not drawn; the service layout adds one more of these, a feeder that passes the scanner's blocks to the service threads.

Work moves between the two kinds of thread through two queues. A service thread takes a block from the scanner, runs its MTF, and puts it on the **prepared queue**. A team takes the next prepared block, builds and walks its table, and puts the result on the **finishing queue**. A service thread then takes it from there, finishes it, and hands it on. The buffers each stage needs are a fixed set, reused from block to block, so the queues can't grow without bound.

A block's path through the service layout

A sketch of the stages and queues, not timing. Blue boxes are work on a service thread, green work on a team, grey a queue. A service thread that becomes free finishes a walked block first, unless the prepared queue is empty, in which case it prepares one.

A free service thread has to choose between preparing and finishing. Its rule is to finish first, because finishing frees buffers and gets output to the reader, but never to leave the prepared queue empty while a block could be prepared, because an idle team wastes three threads. It also stops preparing once a set number of blocks are waiting in the prepared queue, so that preparing can't run far ahead and hold buffers that finishing needs. That number is the **admission threshold**, and it is two, one a team. [Section 6](<#6-what-still-holds-the-teams-back>) comes back to it.

## 2\. Why might it pay, and why might it not?

**Why it might pay.** In 0.2.5's plain teams, a team's two helpers had nothing to do for roughly a third of each block, while the leader ran MTF and the finish alone. 0.2.6 fills that time by overlapping, but at a price: the leader leaves the build, so the shared part of each block runs on one thread fewer and gets slower. In the service layout no team member does the one-thread work, so all three threads of a team build and walk every block, and none of them waits for MTF or the finish.

**Why it might not.** There are three costs. First, ten threads now make two teams, not three, so only two blocks are inverted at once instead of three. Second, all the one-thread work of every block goes through the service threads. If they can't keep up, both teams wait for them. Third, every block now crosses between threads twice, from a service thread to a team and back, and each hand-off takes some time.

Before the first run, a model built from earlier single-thread measurements put the one-thread work at about 0.57 nanoseconds per byte of output. At the rate the decoder then reached on this machine, that came to about 0.8 of one core, so one service thread looked, on paper, nearly enough. The model said itself that its inputs came from another machine and another layout.

## 3\. Why isn't one service thread enough?

The first run ([pull request 251](<https://github.com/lmmx/mibz2/pull/251>)) was registered: its comparisons, its possible outcomes and what each would mean were fixed before it ran. Its first attempt failed its noise check, two copies of the same decoder run under different names, and was discarded; an unchanged rerun was registered before it ran, and passed that check at 1.003. Each arm decoded the Wikidata sample, 11.85 MB of bzip2 that decodes to 209.7 MB, 20 times, the arms taking turns in each round.

"Speed" here and below is the reference's elapsed time over the arm's, paired round by round, so above 1 means the arm is faster. "CPU" is the reference's processor time over the arm's, all threads added up, so above 1 means the arm spends less. Each 95% interval describes the spread across the 20 rounds of one run, on one machine and one file.

With one service thread beside three teams of three, ten decoding threads in all, the decoder was much slower than 0.2.6's default at ten threads: a speed of 0.799 (0.786–0.812). The trace showed why. The service thread was busy 96% of the time, finished blocks backed up behind it, and the teams waited for it. Moving the one-thread work onto one thread had made that thread the bottleneck. The model had put that work at 0.8 of a core; in practice one thread couldn't keep up.

Two service threads were measured in the same run, in two places. Beside three teams of three, on twelve threads, the layout was 5.8% faster than the default, 1.058 (1.035–1.081), but spent 11% more processor time and 8 MB more peak memory. It also used eleven decoding threads to the default's nine. A gain bought with more threads and more processor time is no reason to change a default, and we didn't pursue it.

The other place was the layout this post is about: two teams of three plus two service threads, on ten threads, eight decoding threads in all. It was 1.045 (1.023–1.068) as fast as the default, with 12% less processor time and 64 MB of peak memory against 74 MB. But that comparison was reported, not registered as a decision. And the run had no plain two-team arm, so it couldn't say whether the gain came from the service threads or simply from running two teams instead of three. The next run was designed to answer both.

## 4\. Did two service threads beat the default?

The second run ([pull request 258](<https://github.com/lmmx/mibz2/pull/258>)) judged three comparisons, each against an arm chosen to answer one question:

- **Against the default:** is the service layout faster than 0.2.6 at ten threads?
- **Against plain two teams:** two teams of three without service threads, each leader doing its own MTF and finish, at the same budget. Do the service threads make the difference?
- **Two pipelined teams against the default:** 0.2.6's own pipelined teams, but two of them. Does today's decoder gain just from running two teams?

The run passed every validity check, including its noise check at 0.997 (0.971–1.023). The judged comparisons:

| comparison (decoding threads) | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| service vs default (8 vs 9) | **1.045 (1.017–1.073)** | 1.146 (1.123–1.170) |
| service vs plain two teams (8 vs 6) | 1.402 (1.368–1.437) | 0.941 (0.924–0.959) |
| two pipelined teams vs default (6 vs 9) | 0.866 (0.851–0.881) | 1.214 (1.196–1.233) |

The service layout beat the default, by the margin the first run had reported. The figure near the top of the post shows all three runs. Its CPU ratio of 1.146 means it spent 1/1.146 = 0.873 of the default's processor time: 12.7% less, or between about 11% and 14.5% less over the interval. By the rule fixed before the run, a speed gain without a processor-time cost, together with a gain over plain two teams, was the outcome "for the service layout".

The controls rule out the simplest explanation. Two teams alone did not reproduce the gain: plain two teams were far slower than the service layout, and 0.2.6's pipelined teams, cut to two, reached only 0.866 of the default's speed. So it is not having two teams instead of three that wins here.

What the controls don't do is say which part of the service layout earns the gain. The comparison with plain two teams measures everything the layout changes at once: two more threads, the one-thread work moved onto them, the feeder, the queues and the hand-offs. They were not varied separately.

The run's medians, over 20 rounds each, give the costs side by side:

| arm (leaders + helpers + service) | elapsed (s) | CPU (s) | peak (MB) |
| --- | ---: | ---: | ---: |
| 0.2.6 default (3 + 6 + 0) | 0.144 | 1.026 | 74 |
| service layout (2 + 4 + 2) | 0.137 | 0.895 | 64 |
| two pipelined teams (2 + 4 + 0) | 0.166 | 0.850 | 57 |
| plain two teams (2 + 4 + 0) | 0.193 | 0.847 | 52 |
| plain three teams, 0.2.5 (3 + 6 + 0) | 0.153 | 0.985 | 67 |

The medians agree with the paired figures, but they aren't a substitute for them: the speed and CPU ratios above are computed round by round.

The model before the run had predicted plain two teams at 0.180–0.188 s, the gain over them near 1.33, and two pipelined teams at 0.82–0.85 of the default. Each came out on the predicted side, and each a little beyond it: plain two teams slower, at 0.193 s, the gain over them larger, at 1.40, and two pipelined teams a little better, at 0.866.

## 5\. Has the result been seen again?

A third run ([pull request 271](<https://github.com/lmmx/mibz2/pull/271>)) asked another question, about the admission threshold. Beside its own comparisons, it ran the service layout against the default again, as a reported comparison: 1.073 (1.041–1.106) for speed, and 1.169 (1.135–1.204) for processor time, so about 14.5% less. Its noise check came out at 0.994.

That is the same comparison as the registered one, on the same machine and file, but in another run, at a later commit of the prototype, beside other arms. It was never judged, so it does not replace the registered 1.045. What it adds is that the direction held. Across the three runs that measured the layout against the default, it was faster each time, by 1.045, 1.045 and 1.073, and spent 12–15% less processor time each time. Its peak memory was 64 MB against 74 MB in all three.

The three estimates also say something about the intervals. Each describes the spread within its own run. The registered 1.045 and the later 1.073 differ by about half the width of either interval. Neither interval says how much the figure moves from one run to the next, and the difference between them is a sign that it moves by a few points. The fair summary is a gain of a few per cent, measured on one machine, not a precise 4.5% or 7.3%.

## 6\. What still holds the teams back?

The registered run traced the service layout as it decoded, over six further rounds. Two figures stood out. Each service thread was marked busy for about 70% and 67% of the run, so neither was close to the 96% of the single service thread. Yet the teams spent 16.5% of their time idle, in waits that began with no prepared block available.

That combination suggested the admission threshold. With two service threads and a threshold of two, a service thread stops preparing as soon as two blocks are waiting, even if it has nothing else to do. In fact up to three can be waiting, since a second service thread can start preparing while the first is still at it, but the point stands: the teams could run dry while a service thread sat idle.

So the third run raised the threshold from two to three, and to four, with everything else as in the registered run. Neither was shown to be faster or slower:

| threshold (vs two) | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| three | 0.992 (0.968–1.016) | 1.006 (0.987–1.025) |
| four | 0.979 (0.951–1.008) | 0.983 (0.958–1.009) |

Both intervals include gains and losses of a few per cent, so neither change is shown to help or to hurt. The point estimate for four leans slower, and a loss of up to about 5% isn't excluded. The threshold alone is not the change that helps. Among the run's reported, unjudged arms, one raised the threshold to four and also let more blocks be in flight at once, twelve instead of eight. Against four alone, it was 1.049 (1.017–1.082). That arm was picked out after the run, so it is a lead for a future registered comparison, not a result.

**What the trace's labels can and can't say.** When the third run's result was first written up, its traces were read as showing that the bottleneck had moved, from waiting for prepared blocks to waiting for finishing. A review of the tracing code then showed that reading doesn't follow, and it was withdrawn ([the review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231>), and the correction in [pull request 289](<https://github.com/lmmx/mibz2/pull/289>), not yet merged). A team's wait is labelled once, by what it saw when it started. A wait that began with nothing prepared, and went on after a block had been prepared, still counts entirely as a wait for preparation. And a service thread's "busy" share is the time it was marked working, not processor time. The figures stand as recorded. They don't identify the bottleneck.

The review names two explanations that survive. Both service threads can be finishing blocks at the moment a team takes the last prepared one, and a finish isn't interrupted, so nothing prepares the next block until one of them is done. Separately, the scanner may start a new block only when fewer than eight are in flight, and a block stops counting only when the reader takes it, in file order. Blocks that are ready but stuck behind an earlier, unfinished one can therefore hold up the scanner while the teams wait. Raising the threshold helps with neither. The next step is to count how often each one happens, without timing anything.

## 7\. What does it cost, and where does it apply?

The figure puts the three costs side by side, from the registered run's medians: elapsed time, the processor time of all threads added up, and peak memory.

Time, processor time and memory, ten threads

Measured, the test machine, medians of 20 rounds of the registered run (pull request 258). The service layout (green) is the only arm both faster and cheaper than the default (blue); the two two-team arms without service threads are cheaper but slower.

**Processor time and memory.** The service layout finishes sooner and does less work in total than 0.2.6's default: 12.7% less processor time in the registered run, and 64 against 74 MB of peak memory. Where the saving comes from isn't measured. The medians in [section 4](<#4-did-two-service-threads-beat-the-default>) show that every two-team arm spent less processor time than the default, between 0.85 and 0.90 s against 1.03 s, and that the service threads cost some of it back: the service layout spent about 6% more than plain two teams. Two teams hold two of the 3.6 MB tables at once rather than three, and the service threads add buffers of their own for the blocks queued between stages. Peak memory wasn't broken down by buffer.

**Fewer tables at once.** In an [earlier post](<https://lmmx.github.io/mibz2/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) we measured that every extra block in progress slows the others, and that the leading explanation is the tables' competition for the processor's shared 20 MiB cache, with evidence from a simpler test program. Inverting two blocks at a time instead of three may help the service layout for that reason. These runs didn't measure it, and the comparison with two pipelined teams shows that fewer tables alone don't give the gain.

**Fewer decoding threads.** A caller who asks for ten threads gets eight decoding, two of them service threads, plus the feeder. The layout depends on the service threads not saturating, and their margin in the registered run was the trace's 70%. Under other load on the same machine, or on a file with more small blocks or a different mix of stages, they could saturate first, where three teams might slow down more gently. None of that is measured.

**Where it applies.** Everything here was measured on one processor shape: the i9-10900K, with 10 cores and a 20 MiB last-level cache, at ten threads, decoding principally one file. The cloud machines that benchmark every change to mibz2 have four virtual CPUs, so they can't show the gain. They could show only that a change keyed to the ten-core shape leaves every other machine alone.

**What a default would need.** The service layout is an opt-in prototype on a research branch, reached only through hidden functions. The registered run's journal sets out what making it the default would take, in order: a rule that picks it only where it was measured, the ten-core shape at exactly ten threads; the usual paired benchmark on every change, to show the rule changes nothing elsewhere; a registered run of the production build's default against the 0.2.6 release on the test machine; and a safety review of the production code, with fuzzing that reaches its new paths. Any step can stop it.

**The safety review so far.** The service layout adds new concurrency: who owns each buffer, how blocks pass between threads, what happens on cancellation, a panic or the end of the stream. An independent review of it ([decision D045](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806>)) found the ownership argument coherent but held further timing runs and any production use until the evidence was complete: fuzzing shown to reach the layout's paths, and runs of Miri, a checker for undefined behaviour in Rust. A second review ([D053](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239>)) kept the hold, narrowed to how the fuzz evidence was counted and to unfinished Miri runs. Once that was repaired, the hold was lifted ([D057](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449>)) for the two versions of the code that were measured. The lifting states that it does not qualify a production candidate. A default would be new code, and needs its own review.

## 8\. Where do things stand?

The argument runs like this. In 0.2.6 a team's leader does each block's one-thread work beside its helpers, which gained 7.2% over 0.2.5 but takes the leader out of the build. Moving that work onto threads of its own frees every team member for the shared part, at the cost of one team fewer. One service thread couldn't keep up, and the layout was 20% slower than the default. Two could: two teams of three plus two service threads were 4.5% faster than the default in a registered run, on eight decoding threads instead of nine, with 12.7% less processor time and 10 MB less memory. A later, unjudged run measured 7.3%. Two teams without service threads were slower, so the service threads are part of the gain, though which part of the layout earns it isn't separated. Raising the admission threshold didn't detectably help, and the traces that seemed to explain why turned out not to identify the bottleneck.

The result has the same limits as the last post's: one machine, principally one file, and a prototype, not a default. Two things come next. A count, without timing, of the two surviving explanations for the teams' waits. And the steps above that a default would need, the first of which is a rule that selects the layout only where it was measured.

A separate line of work, building the inverse BWT's table from the runs in bzip2's MTF codes ([an earlier post](<https://lmmx.github.io/mibz2/blog/2026-11-08-bwt-runs-from-mtf-codes/index.md>)), had its first run on the test machine on the same day. That run was inconclusive, because the machine's speed changed during it: some rounds took about twice as long, with every thread's processor time doubled ([its journal](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-run-fed-leader-workstation-result.md>)). It says nothing about the service layout, for or against.

The general lesson: work that can't be shared can be moved off the threads that share the rest, but the threads it moves to then set the pace. Here one was too few and two were enough, and only measuring showed which.

---

**Research records:**

- [The service core's first registered run: its result](<https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-08-service-core-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-07-service-core-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/1e0bf7dcde9f8dd77cc12b9d577554407ec29e33/service-core/20261108T103216Z.tar.gz>) (pull request 251)
- [Two teams and two service threads: the registered result](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-2x3-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/docs/journal/2026-11-08-service-2x3-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/4fccc02b9e95b8b9de5c3be48d0e86e37377de72/service-2x3/20261109T010708Z.tar.gz>) (pull request 258)
- [The admission threshold: the registered result](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-lookahead-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/docs/journal/2026-11-09-service-lookahead-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/705319fd507d79ad545a593b5affa329275d1e00/service-lookahead/20261109T075939Z.tar.gz>) (pull request 271)
- [The review of the traces' labels](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231>) and [the journal's correction](<https://github.com/lmmx/mibz2/blob/f9284a4b96bb258454525599a447173e9b201df6/docs/journal/2026-11-09-service-lookahead-result.md>) (pull request 289)
- The safety reviews: [D045](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806>), [D053](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239>) and [D057](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449>)
- [Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores](<https://lmmx.github.io/mibz2/blog/2026-11-07-pipelined-teams/index.md>)
