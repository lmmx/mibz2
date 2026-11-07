---
title: "Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores"
image: assets/social/blog-2026-11-07-pipelined-teams.png
description: "While a team's helpers invert one block, its leader finishes the last and prepares the next: 7% faster than 0.2.5 on ten cores, for 5% more processor time."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores

While a team's helpers invert one block, its leader finishes the last and prepares the next:
7% faster than 0.2.5 on ten cores, for 5% more processor time.
{: .post-dek }

7 November 2026 · 13 min read · [Research result](#7-where-do-things-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2, our bzip2 decoder written in Rust, decodes a file with small groups of threads that we
call teams. A bzip2 file is a series of blocks that can be decoded independently, and each
team decodes one block at a time. One thread in the team, the leader, does some of each
block's work alone, and the team's other threads, its helpers, wait for it. That waiting is
time the decoder could spend decoding. Our new release, 0.2.6, removes much of it.

In 0.2.6, a team works on three blocks at once. While its helpers do the expensive middle part
of one block, the leader finishes the block before it and prepares the block after it. On the
ten-core machine we test on, with ten threads, this is 7.2% faster than 0.2.5 (95% interval
5.4% to 8.9%), and 6.5–7.0% faster at 11 to 20 threads. It is not cheaper: it uses about 5%
more processor time in total, and 7–8 MB more memory at its peak.

That it helps at all is not obvious. In 0.2.5 the leader takes part in the expensive middle
part. Pipelining takes it out, so that part runs on one thread fewer and gets slower. The 7% is
what is left after paying for that. The first version of the pipeline gained only 4%, for 9%
more processor time. Two refinements made it worth shipping: the leader keeps the pipeline
full after a small block, and threads that wait go to sleep instead of staying on the
processor.

The [previous post](2026-11-07-why-mibz2-stops-scaling.md) explained why 0.2.5 split ten
threads into three teams of three, and proposed this pipeline as the next experiment. This post
is its result. [Section 1](#1-what-does-a-team-do-with-one-block) shows what a team does with
one block, and [section 2](#2-what-does-pipelining-change) what pipelining changes and why it
might not pay. [Section 3](#3-what-did-the-first-pipeline-gain-and-why-so-little) is the first
pipeline's measurement, [section 4](#4-what-did-the-two-refinements-change) the two
refinements, and [section 5](#5-did-the-new-default-hold-up-against-025) the run that
confirmed the release against 0.2.5. [Section 6](#6-what-does-it-cost-and-where-does-it-apply)
covers the cost and where it applies.

## 1. What does a team do with one block?

bzip2 compresses a file in blocks of up to 900 kB each, each compressed with no reference to
the others, so each can be decoded on its own. One thread, the scanner, reads the file and
undoes the outermost layer of every block, the Huffman coding. The rest of a block's work is
done by a team, in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the
   block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which bzip2 used to reorder the
   block's bytes so that similar ones sit together. This puts them back in their original
   order.
3. **The finish:** RLE1, which expands the runs of repeated bytes that bzip2 shortened first
   of all, and the CRC, which checks the result against the block's checksum. Then the block's
   output is handed on, in file order.

The inverse BWT is the expensive part, and the only part a team shares. It has two phases. The
**build** fills a table with one 4-byte entry for each byte of the block, saying where the next
byte of the original text is. For a full block this table, which we call the T-vector, is
3.6 MB. The **walk** then follows the table from entry to entry, writing out a byte at each
step. Each step's address comes from the entry just read, so mibz2 starts many walks at known
points through the block, and the team's threads share them.
[An earlier post](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md) works the
transform through on a small example.

In 0.2.5, the **leader** does the block's MTF alone, then all three threads build and walk the
table, then the leader puts the walk's pieces together and does the finish alone. The two
**helpers** take part only in the build and the walk, and wait the rest of the time. In traces
of four teams, the time outside the inverse BWT was 29–33% of a block's decode
([the previous post, section 4](2026-11-07-why-mibz2-stops-scaling.md#4-why-dont-more-threads-on-one-block-help)).
So for roughly a third of each block, a team's helpers had nothing to do.

The test machine is an Intel Core i9-10900K, with 10 cores (20 threads) and a 20 MiB
last-level cache shared by all of them. On a machine of that shape, given ten or more threads,
0.2.5 runs three teams of three, nine decoding threads in all, plus the scanner. We write that
plan as 3 × 3.

## 2. What does pipelining change?

**The idea:** have the leader do its lone work while the helpers do the shared part, instead
of before and after it. In 0.2.6, while the two helpers build and walk block A, the
leader finishes the block before it, Z, and runs MTF on the block after it, B. When it has done
both, it joins the helpers in walking A. When A's walk ends, the helpers start building B,
whose MTF is already done, and the leader finishes A. The figure shows the two layouts side by
side.

<figure class="bf-fig">
<p class="bf-title">A team of three, in 0.2.5 and in 0.2.6</p>
<svg viewBox="0 0 720 346" role="img" aria-label="Two timelines of a team of three threads. Above, 0.2.5: the leader runs a block's MTF, all three build and walk its table, then the leader finishes the block alone while the two helpers wait. Below, 0.2.6: the two helpers build and walk one block while the leader finishes the previous block and runs the next block's MTF, then joins the walk."><text x="0.0" y="18.0" class="bf-ts">0.2.5: the plain team. The helpers wait while the leader works alone</text><text x="0.0" y="53.0" class="bf-tm">leader</text><text x="0.0" y="87.0" class="bf-tm">helper</text><text x="0.0" y="121.0" class="bf-tm">helper</text><rect x="76" y="34" width="52" height="28" rx="4" class="bf-pill-next"/><text x="102.0" y="52.5" class="bf-t" text-anchor="middle">MTF A</text><rect x="76" y="68" width="52" height="28" rx="4" class="bf-track"/><rect x="76" y="102" width="52" height="28" rx="4" class="bf-track"/><rect x="130" y="34" width="62" height="28" rx="4" class="bf-card-hi"/><text x="161.0" y="52.5" class="bf-t" text-anchor="middle">build A</text><rect x="130" y="68" width="62" height="28" rx="4" class="bf-card-hi"/><text x="161.0" y="86.5" class="bf-t" text-anchor="middle">build A</text><rect x="130" y="102" width="62" height="28" rx="4" class="bf-card-hi"/><text x="161.0" y="120.5" class="bf-t" text-anchor="middle">build A</text><rect x="194" y="34" width="100" height="28" rx="4" class="bf-card-hi"/><text x="244.0" y="52.5" class="bf-t" text-anchor="middle">walk A</text><rect x="194" y="68" width="100" height="28" rx="4" class="bf-card-hi"/><text x="244.0" y="86.5" class="bf-t" text-anchor="middle">walk A</text><rect x="194" y="102" width="100" height="28" rx="4" class="bf-card-hi"/><text x="244.0" y="120.5" class="bf-t" text-anchor="middle">walk A</text><rect x="296" y="34" width="70" height="28" rx="4" class="bf-pill-next"/><text x="331.0" y="52.5" class="bf-t" text-anchor="middle">finish A</text><rect x="296" y="68" width="70" height="28" rx="4" class="bf-track"/><rect x="296" y="102" width="70" height="28" rx="4" class="bf-track"/><rect x="368" y="34" width="52" height="28" rx="4" class="bf-pill-next"/><text x="394.0" y="52.5" class="bf-t" text-anchor="middle">MTF B</text><rect x="368" y="68" width="52" height="28" rx="4" class="bf-track"/><rect x="368" y="102" width="52" height="28" rx="4" class="bf-track"/><rect x="422" y="34" width="62" height="28" rx="4" class="bf-card-hi"/><text x="453.0" y="52.5" class="bf-t" text-anchor="middle">build B</text><rect x="422" y="68" width="62" height="28" rx="4" class="bf-card-hi"/><text x="453.0" y="86.5" class="bf-t" text-anchor="middle">build B</text><rect x="422" y="102" width="62" height="28" rx="4" class="bf-card-hi"/><text x="453.0" y="120.5" class="bf-t" text-anchor="middle">build B</text><rect x="486" y="34" width="100" height="28" rx="4" class="bf-card-hi"/><text x="536.0" y="52.5" class="bf-t" text-anchor="middle">walk B</text><rect x="486" y="68" width="100" height="28" rx="4" class="bf-card-hi"/><text x="536.0" y="86.5" class="bf-t" text-anchor="middle">walk B</text><rect x="486" y="102" width="100" height="28" rx="4" class="bf-card-hi"/><text x="536.0" y="120.5" class="bf-t" text-anchor="middle">walk B</text><rect x="588" y="34" width="70" height="28" rx="4" class="bf-pill-next"/><text x="623.0" y="52.5" class="bf-t" text-anchor="middle">finish B</text><rect x="588" y="68" width="70" height="28" rx="4" class="bf-track"/><rect x="588" y="102" width="70" height="28" rx="4" class="bf-track"/><text x="76.0" y="156.0" class="bf-tm">Two blocks, A and B, inverted and finished</text><path d="M76 162H676" class="bf-rule"/><text x="0.0" y="198.0" class="bf-ts">0.2.6: pipelined. The leader finishes one block and prepares the next</text><text x="0.0" y="233.0" class="bf-tm">leader</text><text x="0.0" y="267.0" class="bf-tm">helper</text><text x="0.0" y="301.0" class="bf-tm">helper</text><rect x="76" y="248" width="90" height="28" rx="4" class="bf-card-hi"/><text x="121.0" y="266.5" class="bf-t" text-anchor="middle">build A</text><rect x="168" y="248" width="144" height="28" rx="4" class="bf-card-hi"/><text x="240.0" y="266.5" class="bf-t" text-anchor="middle">walk A</text><rect x="76" y="282" width="90" height="28" rx="4" class="bf-card-hi"/><text x="121.0" y="300.5" class="bf-t" text-anchor="middle">build A</text><rect x="168" y="282" width="144" height="28" rx="4" class="bf-card-hi"/><text x="240.0" y="300.5" class="bf-t" text-anchor="middle">walk A</text><rect x="76" y="214" width="70" height="28" rx="4" class="bf-pill-next"/><text x="111.0" y="232.5" class="bf-t" text-anchor="middle">finish Z</text><rect x="148" y="214" width="52" height="28" rx="4" class="bf-pill-next"/><text x="174.0" y="232.5" class="bf-t" text-anchor="middle">MTF B</text><rect x="202" y="214" width="110" height="28" rx="4" class="bf-card-hi"/><text x="257.0" y="232.5" class="bf-t" text-anchor="middle">walk A</text><rect x="314" y="248" width="90" height="28" rx="4" class="bf-card-hi"/><text x="359.0" y="266.5" class="bf-t" text-anchor="middle">build B</text><rect x="406" y="248" width="144" height="28" rx="4" class="bf-card-hi"/><text x="478.0" y="266.5" class="bf-t" text-anchor="middle">walk B</text><rect x="314" y="282" width="90" height="28" rx="4" class="bf-card-hi"/><text x="359.0" y="300.5" class="bf-t" text-anchor="middle">build B</text><rect x="406" y="282" width="144" height="28" rx="4" class="bf-card-hi"/><text x="478.0" y="300.5" class="bf-t" text-anchor="middle">walk B</text><rect x="314" y="214" width="70" height="28" rx="4" class="bf-pill-next"/><text x="349.0" y="232.5" class="bf-t" text-anchor="middle">finish A</text><rect x="386" y="214" width="52" height="28" rx="4" class="bf-pill-next"/><text x="412.0" y="232.5" class="bf-t" text-anchor="middle">MTF C</text><rect x="440" y="214" width="110" height="28" rx="4" class="bf-card-hi"/><text x="495.0" y="232.5" class="bf-t" text-anchor="middle">walk B</text><text x="76.0" y="336.0" class="bf-tm">The same two blocks inverted; B is finished during the next block's build</text></svg>
<figcaption>A sketch of the stages and who runs them, from the decoder's code; bar lengths are illustrative, not measured. Blue is the leader's lone work, green the shared build and walk, grey a thread waiting. "Finish" is RLE1 and the CRC, and in 0.2.5 also putting the walk's pieces together; in 0.2.6 the last thread out of the walk does that.</figcaption>
</figure>

We call this **overlap**: a block's MTF or finish runs at the same time as its team's build or
walk of another block. It needs a second set of MTF buffers and a second buffer for an
assembled block in each team, about 3 MB a team.

**Why it might not pay.** The build and the walk lose a thread. In 0.2.5 three threads build
each table. In 0.2.6 two do, and the leader joins only the walk, and only once its own work is
done. So the shared part gets slower, and the question is whether the time saved by overlap is
larger than the time lost there. The previous post put that in a simple model, and found that
if the shared part ran about 1.5 to 1.6 times slower without the leader, the gain would be
zero
([section 7](2026-11-07-why-mibz2-stops-scaling.md#7-what-will-we-try-next)). Nothing measured
then said how much slower it would run.

## 3. What did the first pipeline gain, and why so little?

We tested the first version of the pipeline against the plain teams at the same plan, 3 × 3 on
ten threads. The test was registered: its comparisons, its possible outcomes and what each
would mean were fixed before the run. Each arm decoded a sample
of the Wikidata dump, 11.85 MB of bzip2 that decodes to 209.7 MB, 20 times, the arms taking
turns in each round. "Speed" here and below is the plain arm's elapsed time over the
pipelined arm's, paired round by round, so above 1 means faster. Its 95% interval describes
the spread across the 20 rounds of one run, on one machine and one corpus.

The pipeline was faster: 1.042 (1.016–1.068). But it used about 9% more processor time, the
time all threads spent running added up (a paired ratio of 0.918, 0.895–0.942). In the
model's terms, the result fits a shared part running about 1.4 to 1.55 times slower without the
leader, depending on what the hand-offs between threads are assumed to cost. That is a fit,
not a measurement, and it is close to the break-even point.

The traces showed why the gain was small. **Only about a third of blocks overlapped:** in 147 of
the sample's 431 blocks, the leader's MTF ran during its own team's build or walk of another
block. Two features of the first version kept the rest from overlapping.

**Small blocks emptied the pipeline.** A block of fewer than 262,144 bytes (256 KiB) is not
worth sharing among three threads, so its team's leader decodes it whole, on one thread. 198 of
the sample's 431 blocks are that small. In the first version, the leader took at most one new
block while its helpers worked. If that block was small, it decoded it and prepared nothing
else. The next block then had to be prepared with the helpers waiting, as in 0.2.5. Each small
block cost about one block's overlap.

**Waiting threads never slept.** A thread waiting for the build to end checked a flag 4,096
times, then asked the operating system in a loop to run something else instead (a yield). When
nothing else is waiting to run, a yield returns at once, so the waiting thread stays on its core
and its wait counts as processor time.

## 4. What did the two refinements change?

Each refinement was tested on its own against the first pipeline, at 3 × 3 on ten threads and
at 3 × 4 on twelve, with its question and outcomes fixed before the run. The figures below are
the ten-thread ones; the twelve-thread results agree.

**Small: the leader keeps the pipeline primed.** After a small block, the leader now takes the
next queued block too, and runs its MTF in the same stretch. It keeps going until it has
prepared a block for its team, or finds none waiting. It never waits for a new block to arrive:
a leader that held an unfinished block while waiting for the scanner could leave the two
waiting on each other.

<figure class="bf-fig">
<p class="bf-title">A small block in the pipeline, before and after the change</p>
<svg viewBox="0 0 720 282" role="img" aria-label="Two timelines of a pipelined team that meets a small block S. Before: the leader decodes S alone and stages nothing, so after the run it finishes block A and runs block B's MTF while the helpers wait. After: the leader also runs B's MTF in the same run, the helpers wait briefly for it, and the next run starts with B ready."><text x="0.0" y="18.0" class="bf-ts">Before: the leader stages nothing after a small block</text><text x="0.0" y="53.0" class="bf-tm">leader</text><text x="0.0" y="87.0" class="bf-tm">helpers</text><rect x="76" y="68" width="174" height="28" rx="4" class="bf-card-hi"/><text x="163.0" y="86.5" class="bf-t" text-anchor="middle">build, walk A</text><rect x="76" y="34" width="52" height="28" rx="4" class="bf-pill-next"/><text x="102.0" y="52.5" class="bf-t" text-anchor="middle">finish Z</text><rect x="130" y="34" width="102" height="28" rx="4" class="bf-pill-ready"/><text x="181.0" y="52.5" class="bf-t" text-anchor="middle">small block S</text><rect x="234" y="34" width="16" height="28" rx="4" class="bf-track"/><rect x="252" y="34" width="62" height="28" rx="4" class="bf-pill-next"/><text x="283.0" y="52.5" class="bf-t" text-anchor="middle">finish A</text><rect x="316" y="34" width="56" height="28" rx="4" class="bf-pill-next"/><text x="344.0" y="52.5" class="bf-t" text-anchor="middle">MTF B</text><rect x="252" y="68" width="120" height="28" rx="4" class="bf-track"/><text x="312.0" y="86.5" class="bf-t" text-anchor="middle">wait</text><rect x="374" y="68" width="174" height="28" rx="4" class="bf-card-hi"/><text x="461.0" y="86.5" class="bf-t" text-anchor="middle">build, walk B</text><rect x="374" y="34" width="56" height="28" rx="4" class="bf-pill-next"/><text x="402.0" y="52.5" class="bf-t" text-anchor="middle">MTF C</text><rect x="432" y="34" width="116" height="28" rx="4" class="bf-card-hi"/><text x="490.0" y="52.5" class="bf-t" text-anchor="middle">walk B</text><text x="548.0" y="128.0" class="bf-tm" text-anchor="end">block B inverted</text><text x="0.0" y="168.0" class="bf-ts">After: the leader takes the next queued job too</text><text x="0.0" y="203.0" class="bf-tm">leader</text><text x="0.0" y="237.0" class="bf-tm">helpers</text><rect x="76" y="218" width="174" height="28" rx="4" class="bf-card-hi"/><text x="163.0" y="236.5" class="bf-t" text-anchor="middle">build, walk A</text><rect x="76" y="184" width="52" height="28" rx="4" class="bf-pill-next"/><text x="102.0" y="202.5" class="bf-t" text-anchor="middle">finish Z</text><rect x="130" y="184" width="102" height="28" rx="4" class="bf-pill-ready"/><text x="181.0" y="202.5" class="bf-t" text-anchor="middle">small block S</text><rect x="234" y="184" width="56" height="28" rx="4" class="bf-pill-next"/><text x="262.0" y="202.5" class="bf-t" text-anchor="middle">MTF B</text><rect x="252" y="218" width="38" height="28" rx="4" class="bf-track"/><rect x="292" y="218" width="174" height="28" rx="4" class="bf-card-hi"/><text x="379.0" y="236.5" class="bf-t" text-anchor="middle">build, walk B</text><rect x="292" y="184" width="62" height="28" rx="4" class="bf-pill-next"/><text x="323.0" y="202.5" class="bf-t" text-anchor="middle">finish A</text><rect x="356" y="184" width="56" height="28" rx="4" class="bf-pill-next"/><text x="384.0" y="202.5" class="bf-t" text-anchor="middle">MTF C</text><rect x="414" y="184" width="52" height="28" rx="4" class="bf-card-hi"/><text x="440.0" y="202.5" class="bf-t" text-anchor="middle">walk</text><text x="466.0" y="278.0" class="bf-tm" text-anchor="end">block B inverted</text></svg>
<figcaption>A sketch, not a measurement: bar lengths are illustrative. S is a small block, which the leader decodes whole; amber marks it. After the change the helpers can wait a little at the end of a run, while the leader finishes B's MTF.</figcaption>
</figure>

It worked as intended. Overlapped blocks rose from 144 to 236.5 of 431 in this run (medians
over six traced rounds; the first run counted 147), and the pipeline got faster: 1.043 (1.020–1.066), with no change detected in
processor time, 1.007 (0.987–1.028). It has a cost, which the figure shows: a leader busy with
more blocks joins the walk later. Of the 233 blocks large enough for the team to invert
together, those whose walk the leader joined late rose from 62 to 113. The registered outcomes had expected a later join to cancel the gain,
and had no entry for a later join with a gain. Our reading, not a registered verdict, is that
the later join is real and paid for, and that the extra overlap outweighs it, on this machine
and corpus.

**Park: waiting threads go to sleep.** A waiting thread still checks its flag 4,096 times,
then, instead of yielding, it adds itself to a list and parks: it sleeps until woken, using no
processor time. The thread that ends the build wakes every thread on the list. Waking a
parked thread takes a few microseconds, which a thread that never slept doesn't pay.

Parking left the elapsed time unchanged, 0.999 (0.981–1.017), and cut processor time by about
5% (a ratio of 1.055, 1.039–1.071). The saving could have come from spinning less before
parking, rather than from parking. A control arm that only cut the checks from 4,096 to 2,000,
still yielding, changed neither figure: 1.004 for speed and 1.006 for processor time, both
intervals holding 1. So the saving is from parking.

<figure class="bf-fig">
<p class="bf-title">Each change against the first pipeline, 3 × 3 on ten threads</p>
<svg viewBox="0 0 720 356" role="img" aria-label="Each change against the plain pipeline, three teams of three on ten threads. Speed: park 0.999, the spin control 1.004, small 1.043, both 1.038. CPU saved: park 1.055, the spin control 1.006, small 1.007, both 1.032. Small and both are faster; park and both spend less processor time."><text x="0.0" y="18.0" class="bf-ts">Speed: the plain pipeline's time over the lever's</text><path d="M120.0 32V152" class="bf-grid"/><text x="120.0" y="168.0" class="bf-tm" text-anchor="middle">0.96</text><path d="M253.3 32V152" class="bf-rule"/><text x="253.3" y="168.0" class="bf-tm" text-anchor="middle">1.00</text><path d="M386.7 32V152" class="bf-grid"/><text x="386.7" y="168.0" class="bf-tm" text-anchor="middle">1.04</text><path d="M520.0 32V152" class="bf-grid"/><text x="520.0" y="168.0" class="bf-tm" text-anchor="middle">1.08</text><text x="0.0" y="51.5" class="bf-t">park</text><path d="M190.0 47.0H310.0" class="bf-cut"/><circle cx="250.0" cy="47.0" r="5" class="bf-dot"/><text x="536.0" y="51.5" class="bf-t">0.999 (0.981–1.017)</text><text x="0.0" y="81.5" class="bf-t">spin control</text><path d="M200.0 77.0H333.3" class="bf-cut"/><circle cx="266.7" cy="77.0" r="5" class="bf-dot"/><text x="536.0" y="81.5" class="bf-t">1.004 (0.984–1.024)</text><text x="0.0" y="111.5" class="bf-t">small</text><path d="M320.0 107.0H473.3" class="bf-cut"/><circle cx="396.7" cy="107.0" r="5" class="bf-dot-hi"/><text x="536.0" y="111.5" class="bf-t-strong">1.043 (1.020–1.066)</text><text x="0.0" y="141.5" class="bf-t">both</text><path d="M323.3 137.0H436.7" class="bf-cut"/><circle cx="380.0" cy="137.0" r="5" class="bf-dot-hi"/><text x="536.0" y="141.5" class="bf-t-strong">1.038 (1.021–1.055)</text><text x="0.0" y="196.0" class="bf-ts">CPU saved: the plain pipeline's CPU over the lever's</text><path d="M120.0 210V330" class="bf-grid"/><text x="120.0" y="346.0" class="bf-tm" text-anchor="middle">0.96</text><path d="M253.3 210V330" class="bf-rule"/><text x="253.3" y="346.0" class="bf-tm" text-anchor="middle">1.00</text><path d="M386.7 210V330" class="bf-grid"/><text x="386.7" y="346.0" class="bf-tm" text-anchor="middle">1.04</text><path d="M520.0 210V330" class="bf-grid"/><text x="520.0" y="346.0" class="bf-tm" text-anchor="middle">1.08</text><text x="0.0" y="229.5" class="bf-t">park</text><path d="M383.3 225.0H490.0" class="bf-cut"/><circle cx="436.7" cy="225.0" r="5" class="bf-dot-hi"/><text x="536.0" y="229.5" class="bf-t-strong">1.055 (1.039–1.071)</text><text x="0.0" y="259.5" class="bf-t">spin control</text><path d="M206.7 255.0H343.3" class="bf-cut"/><circle cx="273.3" cy="255.0" r="5" class="bf-dot"/><text x="536.0" y="259.5" class="bf-t">1.006 (0.986–1.027)</text><text x="0.0" y="289.5" class="bf-t">small</text><path d="M210.0 285.0H346.7" class="bf-cut"/><circle cx="276.7" cy="285.0" r="5" class="bf-dot"/><text x="536.0" y="289.5" class="bf-t">1.007 (0.987–1.028)</text><text x="0.0" y="319.5" class="bf-t">both</text><path d="M313.3 315.0H406.7" class="bf-cut"/><circle cx="360.0" cy="315.0" r="5" class="bf-dot-hi"/><text x="536.0" y="319.5" class="bf-t-strong">1.032 (1.018–1.046)</text></svg>
<figcaption>Measured, the test machine, 20 paired rounds each, with 95% intervals. Both ratios are the first pipeline's figure over the changed one's, so above 1 is better: faster, or less processor time. Green dots are intervals wholly above 1. The spin control cuts the checks before a yield, and doesn't park.</figcaption>
</figure>

**Both together** were about as fast as small alone, 1.038 (1.021–1.055), and saved less
processor time than parking alone, 1.032 (1.018–1.046) against 1.055. A likely reason is that a
busier leader leaves less waiting to sleep through, but that is not established.

**Against the plain teams,** in the same run, the pipeline with both changes was 1.072
(1.053–1.091) as fast as plain 3 × 3, for about 5% more processor time (a paired ratio of
0.951). Its peak memory was 75 MB, against 67 MB for the plain teams of 0.2.5's default. That
is the version 0.2.6 ships.

One thing these runs can't do is split the gain into its causes. Some of it may come from the
overlap, and some from the way the leader joins the walk as an extra thread once its own work
is done. The runs measure the total.

## 5. Did the new default hold up against 0.2.5?

The 1.072 came from a comparison that was reported, not registered as a decision. So before
release we fixed one more test, with its pass rule set before any run: the 0.2.6 build against
the 0.2.5 release, each decoding as a caller gets it by default, at 8, 9, 10, 11, 12, 16 and
20 threads, 20 paired rounds each. To pass, ten threads had to be at least 1.03 as fast, with
its interval above 1. At 11 to 20 threads, each count's interval had to rule out being more
than 2% slower. At 8 and 9 threads
both builds run four teams of two, unchanged, so those two counts check that nothing else
moved. The prediction was 1.07–1.08 at ten threads, about the same above, and no change at 8
and 9.

The run passed. The table gives each thread count's plan and speed, with the processor time
and peak memory of 0.2.6 over 0.2.5's (medians of the 20 rounds):

| threads | plan, 0.2.5 → 0.2.6 | speed (95%) | processor time | peak memory |
| ---: | --- | ---: | ---: | ---: |
| 8 | 4 × 2, unchanged | 1.002 (0.991–1.013) | 0.99 | 1.00 |
| 9 | 4 × 2, unchanged | 1.001 (0.991–1.012) | 1.00 | 1.00 |
| 10 | 3 × 3, now pipelined | **1.072 (1.054–1.089)** | 1.04 | 1.10 |
| 11 | 3 × 3, now pipelined | 1.070 (1.055–1.085) | 1.05 | 1.12 |
| 12 | 3 × 3, now pipelined | 1.069 (1.054–1.085) | 1.05 | 1.10 |
| 16 | 3 × 3, now pipelined | 1.065 (1.045–1.085) | 1.05 | 1.13 |
| 20 | 3 × 3, now pipelined | 1.070 (1.048–1.092) | 1.04 | 1.12 |

<figure class="bf-fig">
<p class="bf-title">0.2.6 against 0.2.5, by the threads asked for</p>
<svg viewBox="0 0 720 306" role="img" aria-label="The speed of 0.2.6 against 0.2.5 at 8 to 20 threads. At 8 and 9 threads, where both run four teams of two, 1.002 and 1.001. At 10 threads 1.072, and at 11, 12, 16 and 20 threads 1.070, 1.069, 1.065 and 1.070, each interval above 1."><path d="M70 230.0H700" class="bf-grid"/><text x="64.0" y="234.0" class="bf-tm" text-anchor="end">0.98</text><path d="M70 196.7H700" class="bf-rule"/><text x="64.0" y="200.7" class="bf-tm" text-anchor="end">1.00</text><path d="M70 163.3H700" class="bf-grid"/><text x="64.0" y="167.3" class="bf-tm" text-anchor="end">1.02</text><path d="M70 130.0H700" class="bf-grid"/><text x="64.0" y="134.0" class="bf-tm" text-anchor="end">1.04</text><path d="M70 96.7H700" class="bf-grid"/><text x="64.0" y="100.7" class="bf-tm" text-anchor="end">1.06</text><path d="M70 63.3H700" class="bf-grid"/><text x="64.0" y="67.3" class="bf-tm" text-anchor="end">1.08</text><path d="M70 30.0H700" class="bf-grid"/><text x="64.0" y="34.0" class="bf-tm" text-anchor="end">1.10</text><text x="0.0" y="16.0" class="bf-tm">speed against 0.2.5 (above 1 is faster)</text><path d="M90.0 211.7V175.0" class="bf-cut"/><circle cx="90.0" cy="193.3" r="5" class="bf-dot"/><text x="90.0" y="167.0" class="bf-tm" text-anchor="middle">1.002</text><text x="90.0" y="250.0" class="bf-t" text-anchor="middle">8</text><path d="M190.0 211.7V176.7" class="bf-cut"/><circle cx="190.0" cy="195.0" r="5" class="bf-dot"/><text x="190.0" y="168.7" class="bf-tm" text-anchor="middle">1.001</text><text x="190.0" y="250.0" class="bf-t" text-anchor="middle">9</text><path d="M290.0 106.7V48.3" class="bf-cut"/><circle cx="290.0" cy="76.7" r="7" class="bf-dot-hi"/><text x="290.0" y="40.3" class="bf-t-strong" text-anchor="middle">1.072</text><text x="290.0" y="250.0" class="bf-t" text-anchor="middle">10</text><path d="M390.0 105.0V55.0" class="bf-cut"/><circle cx="390.0" cy="80.0" r="5" class="bf-dot-hi"/><text x="390.0" y="47.0" class="bf-tm" text-anchor="middle">1.070</text><text x="390.0" y="250.0" class="bf-t" text-anchor="middle">11</text><path d="M490.0 106.7V55.0" class="bf-cut"/><circle cx="490.0" cy="81.7" r="5" class="bf-dot-hi"/><text x="490.0" y="47.0" class="bf-tm" text-anchor="middle">1.069</text><text x="490.0" y="250.0" class="bf-t" text-anchor="middle">12</text><path d="M590.0 121.7V55.0" class="bf-cut"/><circle cx="590.0" cy="88.3" r="5" class="bf-dot-hi"/><text x="590.0" y="47.0" class="bf-tm" text-anchor="middle">1.065</text><text x="590.0" y="250.0" class="bf-t" text-anchor="middle">16</text><path d="M690.0 116.7V43.3" class="bf-cut"/><circle cx="690.0" cy="80.0" r="5" class="bf-dot-hi"/><text x="690.0" y="35.3" class="bf-tm" text-anchor="middle">1.070</text><text x="690.0" y="250.0" class="bf-t" text-anchor="middle">20</text><text x="390.0" y="270.0" class="bf-tm" text-anchor="middle">threads asked for</text><path d="M76.0 282H204.0" class="bf-axis"/><text x="140.0" y="298.0" class="bf-tm" text-anchor="middle">4 × 2, unchanged</text><path d="M276.0 282H704.0" class="bf-axis"/><text x="490.0" y="298.0" class="bf-tm" text-anchor="middle">3 × 3: plain in 0.2.5, pipelined in 0.2.6</text></svg>
<figcaption>Measured, the test machine, 20 paired rounds a thread count, with 95% intervals. Speed is 0.2.5's elapsed time over 0.2.6's. Green dots are intervals wholly above 1.</figcaption>
</figure>

From the run's records, at ten threads the median decode of the sample took 0.144 s against
0.156 s, about 1,460
against 1,350 MB/s of output. Processor time rose from 0.98 s to 1.02 s, and peak memory from
69 to 76 MB. The outcome matched the prediction for speed and peak memory (1.10–1.13 against about 1.12),
and came in a little under it for processor time (1.04–1.05 against 1.05–1.06).
Above ten threads the plan is the same three teams, so the gain is the same: a request for
twenty threads still decodes on nine, plus the scanner.

## 6. What does it cost, and where does it apply?

**The trade-off.** 0.2.6 finishes sooner, but it doesn't do less work: it spends about 4–5%
more processor time and 7–8 MB more peak memory than 0.2.5 on this workload. Elapsed time
improves because work that used to run while two threads waited now runs beside them. The
extra memory is each team's second set of MTF buffers and second assembled block.

**Where it applies.** Only the 3 × 3 plan is pipelined, and the decoder picks that plan only on
a machine with 10 physical cores and a 20 MiB last-level cache, given 10 or more threads. Every
other plan and every other machine shape decodes exactly as 0.2.5 did. The output is the same
bytes in every case: the pipelined path is tested byte for byte against libbzip2, and on
corrupt and truncated input, by forcing the plan on any machine. But its speed has been
measured on one machine, the test machine, and one corpus, Wikidata. Another processor of the
same shape gets the pipeline too, measured or not. The cloud machines that benchmark every
change to mibz2 have 4 virtual CPUs on 2 physical cores, so they never take this path. They
show only that nothing else moved. Thread counts of 13 to 15, 17 to 19 and above 20 were not
measured, though they run the same plan.

**The plain teams are still available.** A caller that prefers lower processor time can ask
for 0.2.5's three plain teams directly, with
[`BzReader::parallel_planned`](https://docs.rs/mibz2/0.2.6/mibz2/struct.BzReader.html#method.parallel_planned)
and the plan
[`Plan::Teams`](https://docs.rs/mibz2/0.2.6/mibz2/parallel/enum.Plan.html):

```rust
use mibz2::{parallel::Plan, BzReader, Kernels};

// three plain teams of three, 12 blocks in flight (the default's window at this plan)
let reader = BzReader::parallel_planned(file, Plan::Teams { teams: 3, k: 3 }, 12, Kernels::detect());
```

## 7. Where do things stand?

The argument runs like this. In 0.2.5, a team's two helpers waited through about a third of
each block, while the leader did MTF and the finish alone. 0.2.6 has the leader do that work
while the helpers build and walk another block. That takes the leader out of the build, which
slows it, so the first version gained only 4%, for 9% more processor time. Two changes made it
pay: preparing the next block after a small one raised overlap from about a third of blocks to
about half, and putting waiting threads to sleep took back most of the extra processor time.
A registered run of the release against 0.2.5 confirmed 7.2% at ten threads, and 6.5–7.0% at
11 to 20.

0.2.5's three teams were 8.8% faster than 0.2.4's four teams of two at ten threads, and 0.2.6
is 7.2% faster than 0.2.5. Multiplied together, the two would put 0.2.6 about 17% ahead of
0.2.4 there. That is a projection from two separate runs, not a measurement: a direct
comparison of the releases has not yet run.

What remains open is in two places. The previous post's question, why each extra block in
progress slows the others, is unchanged by this release. And the pipeline still leaves the
leader's serial work inside each team. The next question is whether moving that work onto
threads of its own, outside the teams, would do better on the same number of cores
([issue 251](https://github.com/lmmx/mibz2/issues/251)). Nothing about it has been measured
yet.

The general lesson: when part of a job can't be shared, it can still run beside the part that
is. Whether that pays depends on what the shared part loses to make room, and here the answer
was found only by measuring.

---

**Research records:**

- [The first pipeline: its result](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-team-pipeline-result.md)
  and [its registration](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-team-pipeline-registration.md)
- [The two refinements: their result](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipeline-levers-result.md)
  and [their registration](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipeline-levers-registration.md)
- [The confirmation against 0.2.5: its registration](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-07-pipelined-default-registration.md)
  and [its bundle](https://github.com/lmmx/mibz2/blob/0b8650970fbb2d61360a98f6b694a55c46f62d80/pipelined-default/20261107T105448Z.tar.gz)
- [The 0.2.6 changelog](https://github.com/lmmx/mibz2/blob/838890ab/CHANGELOG.md)
- [The three-team policy, 0.2.5 against 0.2.4](https://github.com/lmmx/mibz2/blob/838890ab/docs/journal/2026-11-06-three-team-policy.md)
