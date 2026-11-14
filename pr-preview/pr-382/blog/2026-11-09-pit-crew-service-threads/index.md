[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/index.md>)

# mibz2 gets a pit crew

Handing each block's serial steps to two dedicated threads gives mibz2 a rare double win, decoding 4.5% faster on ten cores with 13% less CPU.

9 November 2026 · 16 min read · [Research in progress](<#8-where-do-things-stand>)

mibz2, our bzip2 decoder written in Rust, decodes the blocks of a file in parallel, each block on a team of threads. Most of a block's work can be shared among a team's threads, but two steps can't: the move-to-front step before the shared part, and the finish after it. In release 0.2.6, as the [last post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-07-pipelined-teams/index.md>) showed, each team's leader runs those steps itself while its two helpers work on the shared part. This post is about the next step: taking that one-thread work out of the teams altogether. Two threads of their own, which we call service threads, do it for every block, and the teams only do the shared part. They are the teams' pit crew, so the teams never have to leave the track.

**Threads asked for, and threads that decode.** A caller gives mibz2 a thread setting, and the setting is the budget of the decoding plan. It counts the decoding threads, the team members and any service threads, and nothing else. The support threads come on top of it: the scanner, which reads the file, and a few others that pass blocks along. At a setting of ten, 0.2.6 uses nine decoding threads, three teams of three, since teams of three can't use the tenth. The service layout uses eight: two teams of three plus the two service threads. So it inverts two blocks at a time instead of three.

**What we found.** On the ten-core machine we test on, the service layout decoded our test file 4.5% faster than 0.2.6's default and spent 12.7% less processor time. That comes from a registered run, one whose comparisons and decision rules were fixed before it ran. The simplest reading is that freeing every team member for the shared work more than pays for having one team fewer, as long as the pit crew keeps up. Two service threads did. One, tried only beside three teams, did not.

Three limits come with it. The layout is a prototype on a research branch, not mibz2's default. It was measured on one processor, an Intel Core i9-10900K, and principally on one file, a sample of the Wikidata dump. And the gain belongs to the layout as a package. Two teams without service threads were slower, so the service threads matter. But the runs changed several things at once, and which of them earns the gain is not yet separated.

The results in brief:

- **The registered gain:** 4.5% faster than the default, with 12.7% less processor time and 10 MB less peak memory. A later run, which didn't judge the comparison, measured 7.3%.
- **Two teams alone lose:** without service threads, two teams were slower than the default, so the gain doesn't come simply from running fewer teams.
- **One service thread was too few for three teams:** with one, three teams of three were 20% slower than the default, because that thread couldn't keep up. A second service thread beside the same three teams fixed that, but bought speed with more processor time: a trade, not a win. No run tried one service thread with two teams.
- **Preparing further ahead didn't detectably help:** letting the service threads prepare three or four blocks ahead, instead of two, was inconclusive. The teams still wait about a sixth of the time, and what they wait for isn't yet identified.
- **Not yet a default:** a safety review cleared the two research versions that were measured, not a production version. A default needs its own measurements and its own review.

The figure sets the comparisons with the default side by side. Each dot is a speed ratio, the default's elapsed time over the layout's, so above 1 is faster.

The service layout against the 0.2.6 default

Measured, the test machine, a thread setting of ten, 20 paired rounds each. Each dot is the geometric mean of a run's paired ratios, and its line the 95% interval of that estimate, within that run; it is not the range of individual rounds. Green dots are comparisons registered as decisions before their run; amber dots were reported beside them and never judged. The numbers in brackets are the pull requests of the runs.

[Section 1](<#1-what-does-a-service-thread-do>) shows what a service thread does, and [section 2](<#2-why-might-it-pay-and-why-might-it-not>) why that might pay and why it might not. Sections [3](<#3-why-isnt-one-service-thread-enough>) to [5](<#5-has-the-result-been-seen-again>) are the three runs: one service thread failing, two beating the default with the controls beside them, and the result seen again. [Section 6](<#6-what-still-holds-the-teams-back>) is about what still holds the teams back, and [section 7](<#7-what-does-it-cost-and-where-does-it-apply>) about the costs, the scope and what a default would need.

## 1\. What does a service thread do?

A bzip2 file is a series of blocks of up to 900 kB, each compressed with no reference to the others, so each can be decoded on its own. One thread, the scanner, reads the file and undoes the outermost layer of every block, the Huffman coding. The rest of a block's work is in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which puts the bytes back in their original order. It has two phases. The **build** fills a table with one 4-byte entry for each byte of the block, 3.6 MB for a full block. The **walk** follows the table from entry to entry. Both can be split among several threads.
3. **The finish:** putting the walk's pieces together, RLE1 (expanding runs of repeated bytes), and the CRC (checking the result against the block's checksum). Then the block is handed to the reader, in file order.

MTF and the finish run on one thread each. The inverse BWT is the expensive part, and the only part a team shares. In 0.2.6, at a setting of ten or more on a machine of the test machine's shape, the decoder runs three teams of three. Each team has a leader and two helpers. While the helpers build and walk one block, the leader finishes the block before it and runs MTF on the block after it, then joins the walk.

The service layout takes MTF and the finish away from the teams. Two service threads do them for every block, serving both teams. Each team's three threads only build and walk. The figure shows the two layouts at a setting of ten.

Who does what, at a thread setting of ten

A sketch of the layouts, from the decoder's code; it shows which thread does which stage, not timing. Each green outline is a team. Both layouts also run the scanner and a few bookkeeping threads, not drawn; the service layout adds one more of these, a feeder that passes the scanner's blocks to the service threads.

Work moves between the two kinds of thread through two queues. A service thread takes a block from the scanner, runs its MTF, and puts it on the **prepared queue**. A team takes the next prepared block, builds and walks its table, and puts the result on the **finishing queue**. A service thread then takes it from there, finishes it, and hands it on. The buffers each stage needs are a fixed set, reused from block to block, so the queues can't grow without bound.

A block's path through the service layout

A sketch of the stages and queues, not timing: the car is one block, and its drive shows the order of its stops, not how long each takes. Coral is work on a service thread (the pit crew), blue work on a team (on the track), green the scanner, the queues and the reader. A service thread that becomes free finishes a walked block first, unless the prepared queue is empty, in which case it prepares one.

A free service thread has to choose between preparing and finishing. Each time it picks its next job, it finishes first, because finishing frees buffers and gets output to the reader. The exception is an empty prepared queue: then it prepares, because an idle team wastes three threads. The rule acts only at that choice. A thread already in the middle of a finish isn't interrupted, so an empty queue isn't always refilled at once.

A service thread also won't start preparing while a set number of blocks already wait in the prepared queue. That keeps preparing from running ahead and holding buffers that finishing needs. The number is the **admission threshold**, and it is two, one per team. [Section 6](<#6-what-still-holds-the-teams-back>) comes back to it.

## 2\. Why might it pay, and why might it not?

**Why it might pay.** In 0.2.5's plain teams, a team's two helpers had nothing to do for roughly a third of each block, while the leader ran MTF and the finish alone. 0.2.6 fills that time by overlapping, at a price: the leader leaves the build, so the shared part of each block runs on one thread fewer. The figure shows the difference over time.

A team's time, before and after the pit crew

Schematic, from the decoder's code, not measured: the widths are not durations. Coral is the one-thread work (in the pits), blue the shared build and walk (on the track). n is the block the team is inverting. In the service layout the service threads' jobs come from both teams' blocks.

In the service layout no team member leaves the build and walk to run MTF or the finish. The teams can still wait for those stages, for a prepared block or for free buffers, as [section 6](<#6-what-still-holds-the-teams-back>) shows.

**Why it might not.** There are three costs. First, at a setting of ten the layout makes two teams, not three, so only two blocks are inverted at once. Second, all the one-thread work of every block goes through the service threads. If they can't keep up, both teams wait for them. Third, every block now crosses between threads twice, from a service thread to a team and back, and each hand-off takes some time.

Before the first run, a model built from earlier single-thread measurements put the one-thread work at about 0.57 nanoseconds per byte of output. At the rate the decoder then reached on this machine, that came to about 0.8 of one core. On paper, then, one service thread looked nearly enough. The model said itself that its inputs came from another machine and another layout.

## 3\. Why isn't one service thread enough?

The first run ([pull request 251](<https://github.com/lmmx/mibz2/pull/251>)) was registered. Its first attempt failed its noise check, which runs two copies of the same decoder under different names, and was discarded. An unchanged rerun was registered before it ran, and passed that check at 1.003. The test file is the Wikidata sample: 11.85 MB of bzip2 that decodes to 209.7 MB. Each arm decoded it 20 times, the arms taking turns in each round.

**Speed** here and below is the reference's elapsed time over the arm's, so above 1 means the arm is faster. **CPU** is the reference's processor time over the arm's, all threads added up, so above 1 means the arm spends less. Both are paired round by round. Each ratio is the geometric mean of the 20 rounds' ratios, and its 95% interval is the uncertainty of that estimate within one run. It is not the range of the individual rounds.

With one service thread beside three teams of three, at a setting of ten, the decoder was much slower than the default, on ten decoding threads to its nine. The trace showed why. The service thread was marked busy 96% of the time, finished blocks backed up behind it, and the teams waited for it. Moving the one-thread work onto one thread had made that thread the bottleneck. The model had put the work at 0.8 of a core, but in practice one thread couldn't keep up with three teams. No run tried one service thread with two teams. Two teams would change how much work reaches it, so that case is open.

The run's judged comparisons with the default, each arm written as teams × team size + service threads:

| arm, at its setting | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| 3 × 3 + 1, at ten | 0.799 (0.786–0.812) | 1.099 (1.079–1.118) |
| 3 × 3 + 2, at twelve | 1.058 (1.035–1.081) | 0.901 (0.888–0.914) |

The second row is two service threads beside three teams of three, at a setting of twelve. It was 5.8% faster than the default, but spent 11% more processor time and 8 MB more peak memory, on eleven decoding threads to the default's nine. A gain bought with more threads and more processor time is no reason to change a default, so we didn't pursue it.

The run also measured the layout this post is about, two teams of three plus two service threads, at a setting of ten. It was 4.5% faster than the default, with 12% less processor time and 10 MB less peak memory. But that comparison was reported, not registered as a decision. And the run had no plain two-team arm, so it couldn't say whether the gain came from the service threads or simply from running two teams. The next run was designed to answer both.

## 4\. Did two service threads beat the default?

The second run ([pull request 258](<https://github.com/lmmx/mibz2/pull/258>)) judged three comparisons, each against an arm chosen to answer one question:

- **Against the default:** is the service layout faster than 0.2.6 at a setting of ten?
- **Against plain two teams:** two teams of three without service threads, each leader doing its own MTF and finish, at the same setting. Do the service threads make the difference?
- **Two pipelined teams against the default:** 0.2.6's own pipelined teams, but two of them. Does today's decoder gain just from running two teams?

The run passed every validity check, including its noise check at 0.997. The judged comparisons:

| comparison (decoding threads) | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| service vs default (8 vs 9) | **1.045 (1.017–1.073)** | 1.146 (1.123–1.170) |
| service vs plain two teams (8 vs 6) | 1.402 (1.368–1.437) | 0.941 (0.924–0.959) |
| two pipelined teams vs default (6 vs 9) | 0.866 (0.851–0.881) | 1.214 (1.196–1.233) |

The service layout beat the default, by the margin the first run had reported. Its CPU ratio of 1.146 means it spent 0.873 of the default's processor time. That is 12.7% less, or between about 11% and 14.5% less over the interval. By the rule fixed before the run, a speed gain without a processor-time cost, together with a gain over plain two teams, was the outcome "for the service layout".

**The controls rule out the simplest explanation.** Two teams alone did not reproduce the gain. Plain two teams were far slower than the service layout, and 0.2.6's pipelined teams, cut to two, reached only 0.866 of the default's speed. So the win doesn't come from having two teams instead of three. The figure puts the four arms' costs side by side, from the run's medians.

Time, processor time and memory, at a thread setting of ten

Measured, the test machine, medians of 20 rounds of the registered run (pull request 258). The service layout (green) is the only arm both faster and cheaper than the default (blue); the two two-team arms without service threads are cheaper but slower.

Every two-team arm spent less processor time than the default. The service threads cost some of that back, since the service layout spent about 6% more than plain two teams, but it alone is faster than the default as well. The medians agree with the paired ratios without replacing them, because the ratios are computed round by round.

**What the controls don't separate.** The comparison with plain two teams measures everything the layout changes at once: two more threads, the one-thread work moved onto them, the feeder, the queues and the hand-offs. They were not varied separately. The run is evidence for the layout as a package, not for any one of its parts.

The model made before the run had predicted each control. Each came out on the predicted side, and each a little beyond it:

| control | predicted | measured |
| --- | ---: | ---: |
| plain two teams (s) | 0.180–0.188 | 0.193 |
| service vs plain (speed) | near 1.33 | 1.402 |
| two pipelined vs default | 0.82–0.85 | 0.866 |

## 5\. Has the result been seen again?

A third run ([pull request 271](<https://github.com/lmmx/mibz2/pull/271>)) asked another question, about the admission threshold. Beside its own comparisons, it ran the service layout against the default again, as a reported comparison. Its noise check came out at 0.994. The three runs that measured the layout against the default:

| run (pull request) | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| first (251), reported | 1.045 (1.023–1.068) | 1.138 (1.118–1.158) |
| second (258), registered | 1.045 (1.017–1.073) | 1.146 (1.123–1.170) |
| third (271), reported | 1.073 (1.041–1.106) | 1.169 (1.135–1.204) |

The third run makes the same comparison as the registered one, on the same machine and file. But it is another run, at a later commit of the prototype, beside other arms. It was never judged, so it doesn't replace the registered 1.045. What it adds is that the direction held. In all three runs the layout was faster than the default and spent 12–15% less processor time. Its peak memory, by the runs' medians, was 64 MB against the default's 74 each time.

The three estimates also say something about the intervals. Each is the uncertainty of its own run's estimate. The registered 1.045 and the later 1.073 differ by about half the width of either interval. Neither interval says how much the figure moves from one run to the next. The two runs also differ in the prototype's commit and in the arms beside it, so their difference isn't a clean measure of repeatability. It does suggest the figure can move by a few points. The fair summary is a gain of a few per cent, measured on one machine, not a precise 4.5% or 7.3%.

## 6\. What still holds the teams back?

The registered run traced the service layout as it decoded, over six further rounds. The two service threads were marked busy for about 70% and 67% of the run, far from the single service thread's 96%. Yet the teams spent 16.5% of their time idle, in waits that began with no prepared block available. Why would the teams wait for prepared blocks while the threads that prepare them have time to spare?

**Two gates.** A block passes two gates before a team can take it, and they are easy to confuse.

**Gate 1 is the admission threshold** from section 1. A service thread won't start preparing while two prepared blocks already wait. It is checked when a thread picks a job, so it limits when preparing starts, not how many blocks can wait. With both service threads preparing at once, three can.

**Gate 2 is the window,** which limits the blocks in flight. The scanner starts a new block only while fewer than eight are in progress, from scanning to output. A block gives up its place only when the reader takes it, and the reader takes blocks in file order.

**The first guess was gate 1.** With a threshold of two, a service thread can sit idle because two blocks already wait. But a service thread idle earlier is not the same as one idle when a team runs dry. Panel A of the timeline figure below shows how the two can come apart. Service thread 1 is idle at first, because gate 1 is shut. Later, when team 1 needs a block, both service threads are in the middle of finishes, and nothing is prepared until one of them ends.

**The test.** The third run raised the threshold from two to three, and to four, with everything else as in the registered run.

A higher admission threshold, against the service layout at two

Measured, the test machine, a thread setting of ten, 20 paired rounds each (pull request 271); the dot and line are the paired geometric-mean ratio and its 95% interval within the run. The CPU ratios were 1.006 (0.987–1.025) for three and 0.983 (0.958–1.009) for four, also spanning 1.

Both intervals include gains and losses of a few per cent, so neither change is shown to help or to hurt. The point estimate for four leans slower, and a loss of up to about 5% isn't excluded. Raising the threshold alone did not show a gain in this run.

**What the trace's labels can and can't say.** When the third run's result was first written up, its traces were read as showing that the bottleneck had moved, from waiting for prepared blocks to waiting for finishing. A review of the tracing code then showed that reading doesn't follow, and it was withdrawn ([the review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231>), and the correction in [pull request 289](<https://github.com/lmmx/mibz2/pull/289>)). A team's wait is labelled once, by what it saw when it started. So a wait that began with nothing prepared still counts entirely as a wait for preparation, even if a block was prepared before it ended. And a service thread's "busy" share is the time it was marked working, not processor time. The figures stand as recorded, but they don't identify the bottleneck.

**Two possible waits.** The review names two explanations that survive, and the figure draws each as a schematic. In panel A, both service threads are finishing when a team takes the last prepared block. In panel B, gate 2 is the one that's shut. Finished blocks stuck behind an earlier, unfinished one hold their places, so the scanner can't start a new block, and there is nothing to prepare.

Two ways a team can run out of prepared blocks

Possible mechanisms, not a measured attribution: schematic, and the widths are not durations. In A, the pit crew is free early but gate 1 keeps it from preparing; later both crews are mid-finish when a team needs a block. In B, the window's eight places are all held, mostly by blocks that are finished but must wait for block 3 to reach the reader first.

A higher threshold doesn't change the finish-first rule, interrupt a finish or free a place in the window. In panel A it could still help a little. Had service thread 1 prepared more blocks while it was idle, that stock might last through a short finish. This run didn't show that it does, and it doesn't pick between the two panels.

One reported, unjudged arm touched gate 2 as well. It raised the threshold to four and also widened the window to twelve. Against threshold four alone it measured 1.049, with an interval of 1.017 to 1.082. That arm was picked out after the run, so it is a lead for a future registered comparison, not a result. The next step is to count how often each kind of wait happens, without timing anything.

## 7\. What does it cost, and where does it apply?

**Processor time and memory.** The service layout finishes sooner than 0.2.6's default and uses less processor time, all threads added up: 12.7% less in the registered run. Processor seconds don't measure the work itself, in instructions or energy, and where the saving comes from isn't measured. Its peak memory was 64 MB against 74. Two teams hold two of the 3.6 MB tables at once rather than three, and the service threads add buffers of their own for the blocks queued between stages. Peak memory wasn't broken down by buffer.

**Fewer tables at once.** In an [earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) we measured that every extra block in progress slows the others. The leading explanation is the tables' competition for the processor's shared 20 MiB cache, with evidence from a simpler test program. Inverting two blocks at a time instead of three may help the service layout for that reason. These runs didn't measure it, and the comparison with two pipelined teams shows that fewer tables alone don't give the gain.

**Fewer decoding threads.** At a setting of ten the layout runs eight decoding threads, two of them service threads. Its support threads, the feeder among them, come on top. The layout depends on the service threads not saturating, and their margin in the registered run was the trace's 70%. Under other load on the same machine, or on a file with more small blocks or a different mix of stages, they could saturate first. Three teams might then slow down more gently. None of that is measured.

**Where it applies.** Everything here was measured on one processor shape, the i9-10900K with 10 cores and a 20 MiB last-level cache, decoding principally one file. The headline comparison is at a setting of ten; [section 3](<#3-why-isnt-one-service-thread-enough>) also reports three teams plus two service threads at twelve. The cloud machines that benchmark every change to mibz2 have four virtual CPUs, so they can't show the gain. They could show only that a change keyed to the ten-core shape leaves every other machine alone.

**What a default would need.** The service layout is an opt-in prototype on a research branch, reached only through hidden functions. The registered run's journal sets out what making it the default would take, in order:

1. A rule that picks it only where it was measured: the ten-core shape at a setting of exactly ten.
2. The usual paired benchmark on every change, to show the rule changes nothing elsewhere.
3. A registered run of the production build's default against the 0.2.6 release on the test machine.
4. A safety review of the production code, with fuzzing that reaches its new paths.

Any step can stop it.

**The safety review so far.** The service layout adds new concurrency: who owns each buffer, how blocks pass between threads, and what happens on cancellation, a panic or the end of the stream. An independent review of it ([decision D045](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806>)) found the ownership argument coherent. It still held further timing runs and any production use until the evidence was complete. That meant fuzzing shown to reach the layout's paths, and runs of Miri, a checker for undefined behaviour in Rust. A second review ([D053](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239>)) kept the hold, narrowed to how the fuzz evidence was counted and to unfinished Miri runs. Once that was repaired, the hold was lifted ([D057](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449>)) for the two versions of the code that were measured. The lifting states that it does not qualify a production candidate. A default would be new code, and needs its own review.

## 8\. Where do things stand?

The argument runs like this. In 0.2.6 a team's leader runs each block's one-thread work beside its helpers, which takes the leader out of the build. Moving that work onto threads of its own frees every team member for the shared part, at the cost of one team fewer. Beside three teams, one service thread couldn't keep up. Beside two teams, two could. In a registered run, the layout was 4.5% faster than the default, on one decoding thread fewer and with 12.7% less processor time. Two teams without service threads were slower, so the service threads are part of the gain. Which part of the layout earns it isn't separated. Preparing further ahead didn't detectably help, and what the teams still wait for isn't yet identified.

Two things come next. The first is a count, without timing, of the two possible waits in section 6. The second is the list of steps a default would need, starting with a rule that selects the layout only where it was measured.

The general lesson: work that can't be shared can be moved off the threads that share the rest, but the threads it moves to then set the pace. Here one was too few for three teams and two were enough for two, and only measuring showed it.

---

**Research records:**

- [The service core's first registered run: its result](<https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-08-service-core-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-07-service-core-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/1e0bf7dcde9f8dd77cc12b9d577554407ec29e33/service-core/20261108T103216Z.tar.gz>) (pull request 251)
- [Two teams and two service threads: the registered result](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-2x3-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/docs/journal/2026-11-08-service-2x3-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/4fccc02b9e95b8b9de5c3be48d0e86e37377de72/service-2x3/20261109T010708Z.tar.gz>) (pull request 258)
- [The admission threshold: the registered result](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-lookahead-result.md>), [its registration](<https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/docs/journal/2026-11-09-service-lookahead-registration.md>) and [its bundle](<https://github.com/lmmx/mibz2/blob/705319fd507d79ad545a593b5affa329275d1e00/service-lookahead/20261109T075939Z.tar.gz>) (pull request 271)
- [The review of the traces' labels](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231>) and [the journal's correction](<https://github.com/lmmx/mibz2/blob/f9284a4b96bb258454525599a447173e9b201df6/docs/journal/2026-11-09-service-lookahead-result.md>) (pull request 289)
- The safety reviews: [D045](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806>), [D053](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239>) and [D057](<https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449>)
- [Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-07-pipelined-teams/index.md>)
- An aside, unrelated to the service layout: a separate line of work, building the inverse BWT's table from the runs in bzip2's MTF codes ([an earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-bwt-runs-from-mtf-codes/index.md>)), had its first run on the test machine the same day. It was inconclusive, because the machine's speed changed during it ([its journal](<https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-run-fed-leader-workstation-result.md>)).
