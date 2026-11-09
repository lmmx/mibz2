# Independent reader report

Basis: I read only the complete pinned `post.md`, including the visible SVG labels and captions. I did not follow links or consult project records. This is a source-based reading, not a viewport or rendering inspection. Reader model: programmer familiar with threads, queues and caches, unfamiliar with bzip2 internals and this project's history.

## Comprehension answers

### 1. Summary: layout and registered finding

The service layout separates per-block serial work from the teams doing the parallel inverse BWT. The scanner decodes Huffman symbols; either of two service threads prepares a block with MTF; a prepared queue feeds two teams of three; each team builds and walks the inverse-BWT table; a finishing queue returns the block to either service thread for assembly, RLE1 and CRC; the reader takes output in file order. Either service thread can do either serial job. At the headline setting of ten, this uses eight decoding threads, versus the default's nine in three teams of three. There are also scanner/bookkeeping threads. [Opening; §1]

In the registered second run, the service layout's speed ratio against 0.2.6 was **1.045 (95% interval 1.017–1.073)**. It used **12.7% less total processor time**, and median peak memory was **64 MB versus 74 MB**. The evidence is principally one Wikidata sample on a ten-core i9-10900K, not a general result for all machines/files. This is still a research prototype. [§4; §7]

### 2. Motivation: why move work when 0.2.6 already overlaps it?

0.2.6 improves on plain teams by having the leader finish the previous block and prepare the next while its helpers work on the current block. That hides some serial work, but the leader cannot participate in the current block's build while doing those jobs; it only joins the walk later. Moving the jobs to service threads lets all three team members build and walk. It also reduces simultaneous tables from three to two, which might reduce shared-cache competition, although this experiment does not establish that mechanism. [§1; §2; §7]

The costs are a whole team fewer, a shared serial-work capacity limit that can stall both teams, two cross-thread hand-offs per block, extra queues/buffers and a feeder, plus new concurrency obligations. The layout therefore needs measurement: a better staffed build need not outweigh less block parallelism or service bottlenecks. The lower CPU/memory measurements are net outcomes; they do not show that the new machinery is free. [§2; §4; §7]

### 3. Status: 1.045 or 1.073?

**1.045 is the registered result to use.** A first run reported 1.045 without judging that comparison; the second registered and judged it; the third reported 1.073 while testing admission thresholds. The third supports the same favorable direction, but was not the decision comparison and does not supersede the second. [§3–§5]

The difference matters because it discourages treating either percentage as a stable machine-independent constant. The intervals describe uncertainty inside each run, not variability between runs. The later run also used another prototype commit and another set of neighboring arms, so the difference is not a clean repeatability measurement. “A gain of a few per cent on this machine” is the defensible broad claim. [§5]

### 4. Justification: controls and limits

Plain two teams ask whether the service package improves on two teams that still do their own serial stages. Service beats that control by 1.402, but uses eight decoding threads rather than six and about 6% more processor time. Two pipelined teams ask whether simply reducing today's default to two teams helps; their speed ratio against default is 0.866, so it does not here. Together, the controls reject “two teams alone explains the win.” Only the service arm is both faster and cheaper in total processor time than default. [§4]

They do **not** isolate the effects of thread count, moving the serial work, feeder, queues, hand-offs or cache behavior. Nor do they prove that every component is necessary, or that the same arrangement wins elsewhere. The experiment supports the combined layout. [§4; §7]

### 5. Obvious failure: why not one service thread?

The one-thread model predicted serial work requiring around 0.8 core, but used measurements from another machine/layout. In the tested three-team-plus-one-service arrangement, the service thread was marked busy 96% of the time, finished work backed up and teams waited. Speed was only 0.799 of default despite ten decoding threads versus nine. One service thread became the throughput bottleneck. The trace's busy percentage is not CPU utilization, but the queue buildup and waiting make the failure intelligible. [§2; §3; §6]

A useful scope distinction: the directly demonstrated one-service failure has **three teams**, whereas the headline successful arrangement has **two teams and two service threads**. The post does not report a two-teams-plus-one-service test. I can conclude that one failed in the tested layout, not that it has been independently proved insufficient for every two-team layout. [§3]

### Held-out transfer: sixteen cores

I would not predict that exactly two teams plus two service threads beats the default merely because it won here. It still supplies only eight decoding threads; a sixteen-core machine may have a different useful level of block parallelism, cache capacity/contention and service-to-team rate balance. The post does not tell me what the default selects on that machine. [§2; §7]

I would first establish the actual default layout and specify the thread-setting comparison, then run paired comparisons of that default with the service plan on the sixteen-core machine. Measure elapsed time, summed processor time, peak memory, validity/noise controls and variation across repeated runs and representative block/file mixes. To select a layout rather than just test one, vary team/service counts, keeping the comparisons explicit, and inspect queue occupancy and correctly attributed waits to see whether serial service, table contention or in-flight/output ordering limits throughput. Cache evidence would be needed to claim a cache mechanism; wall-clock gains alone would not supply it. A production candidate also needs its own safety review. These are measurements I would require, not predictions supplied by the article. [§3–§7]

## Reading experience

**Where to start.** Start with the opening explanation and “Threads asked for,” then read §1 and §2. I do not need the preceding blog post. The opening gives a usable question and result before technical detail; the navigation paragraph is clear. A result-focused reader can start with §4, but a newcomer should not skip the layout and timing explanation.

**First three sections' pace.** §1 introduces exactly enough bzip2 vocabulary for the argument. The ordered stages and explicit explanation that either service thread does either job prevent a false one-preparer/one-finisher model. It is the longest setup because it also introduces scheduling priority and the threshold, but those details pay off in §6. §2 is the clearest causal section: the leader's absence from the build makes the point of the redesign concrete, and the three costs make it a tradeoff. §3 slows down for noise checks and ratio definitions before the first failure, appropriately for an experimental post. The opening result prose, bullet list and overview chart do repeat the same finding before §1; this is noticeable, but I did not lose the argument or require a cut to understand it.

**Stuck points, by level.**

- **Symbol:** None blocking. MTF, BWT, RLE1, CRC, `n`, speed and CPU ratios are introduced. CPU's reciprocal direction requires attention, and §4's conversion to 0.873 helps.
- **Statement:** “A caller tells mibz2 how many threads it may use” sounds like a hard cap, yet at setting ten the opening has nine decoding threads plus a scanner plus bookkeeping, and §3 has ten decoding threads plus those others. I understand the arm comparisons, but cannot reconcile the setting with total OS thread count from this wording. Clarifying whether it is a work-budget setting rather than a literal total-thread limit would resolve this local ambiguity.
- **Statement:** The opening's “one service thread is too few” is broader than the directly shown three-team experiment. The table itself is clear; the limitation above is what I would retain.
- **Step:** No missing step between scanner, queues, teams, service threads and ordered output. §6 explains why the threshold can be two while three prepared blocks exist: the check controls admission, not a strict queue-length ceiling.
- **Chain:** The main benefit/cost/control chain is complete. The remaining bottleneck is deliberately unresolved, not an explanatory failure. I can reconstruct both §6 candidate waits and see why raising gate 1 need not fix gate 2.
- **Beyond:** The text equips me to withhold a sixteen-core generalization and propose the right measurements. It does not equip me to choose the winning sixteen-core configuration; it explicitly should not.

**Fragmentation, collapse or omitted steps.** I did not find a broken main chain. The evidence is separated sensibly into one-service failure, registered success with controls, and later corroboration. The model-versus-control table at the end of §4 is a small detour: it adds historical prediction detail after the principal conclusion is already secure. §7's sequence of safety decisions is also more detailed than needed to understand current status, but its final boundary is clear. The final unrelated research-record aside does not affect the argument and can simply be skipped.

**Dense passages and visuals.** Based on their text and captions, the existing allocation diagram, team timeline and two-wait figure address the main conceptual difficulties. The §2 timeline is particularly useful for the distinction between overlap and freeing the leader. §6 is the densest passage, with priority, admission, finite in-flight window, ordered release and imperfect trace labels; its A/B figure already supplies the needed concrete cases. I would not request another general diagram. If anything needs an additional compact visual aid, it is the thread-budget ambiguity: a small accounting table separating setting, decoding, scanner, feeder and bookkeeping counts/roles would clarify that statement. I cannot assess text size, clipping, mobile layout, color contrast or actual visual scanning from source alone.
