---
title: "The decoder as a queueing network"
image: assets/social/blog-2026-11-25-the-decoder-as-a-queueing-network.png
description: "Modelled as stations and queues, the decoder shows in advance where more workers stop helping and which experiments to run next."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The decoder as a queueing network

Modelled as stations and queues, the decoder shows in advance where more workers stop helping and which experiments to run next.
{: .post-dek }

25 November 2026 · 28 min read · [Research in progress](#15-what-runs-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

At ten workers, supplying mibz2's decoder with the codes its emission step would otherwise compute
saved nothing measurable, yet the same supply saved 8% once the serial scanner had been relieved
as well. A profile cannot explain that, because it says where time is spent, not which work
decides when a decode finishes; queueing theory is the discipline built for the second question,
and a month of our earlier performance work had been asking it under other names. This post
follows the attempt to apply the theory properly, in order: what we expected it to give (a
characterisation of the decoder as a system, predictions of what a change would buy, and a search
for a better organisation of its work), what each part did when tried, and where it led. The
[bottleneck law](#5-the-bottleneck-law) predicted from four worker counts where adding workers stops helping. Little's law
decided what the instruments had to record and then audited them. Modification analysis explained
the vanishing improvement and changed how every improvement is priced. Load dependence gave one
name to a month of interference findings. The exact theory of product-form networks fitted a
surrogate of the decoder rather than the decoder, so bounds, interventions and a simulation still
to be built do the predicting. And the destination is an experiment that makes each part of the
decoder ideal in turn, to map where new mathematics could pay, including where the present
structure hides it.

**Takeaways**

- The first queueing law tried, the [bottleneck law](#5-the-bottleneck-law), predicted the knee at 7.49 workers from
  measurements at one to four; the measured crossover is 7.67, and the best wall, 0.191 s, is at
  8.
- The theory's simplest laws held and its most elegant results did not: operational identities
  held exactly in 840 traced decodes, while product-form models fitted only a simplified surrogate
  of the decoder, within 1 to 4 ms.
- An improvement's value depends on what else is improved: emission's room is 15–17% below the
  knee, hidden at it, and appears above it mostly behind a relieved scanner. Improvements are now
  priced in combination.
- Where threads sit matters as much as what they compute: workers on the two hardware threads of
  shared cores cost 48% at 8 workers, and two merged changes took 16–37% and 7% off by choosing
  the kernel and the worker count better.
- Even with the scanner and emission both made ideal, the decoder keeps 0.82–0.84 of its wall,
  which is where the next measurements, and the search for new mathematics, are aimed.

The theory itself, and the context it came from, is set out in a
[technical appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md), so that this post can
follow the mibz2 effort on its own terms.

## 1. An improvement whose value vanished

The question began with an oracle. The inversion work had a way to supply the codes that the
emission step would otherwise compute, prepared outside the timed decode, which turns "what if
emission's work were cheap?" into a measurement ([Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md);
[journal §44](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). With one or six workers, the supply saved 17% and 15% of the wall (0.830 and
0.850). With ten, its effect was unresolved, 0.976 with an interval across 1. Two explanations fit
that: the work might offer little at ten workers, or something else might be stopping the time it
saved from reaching the end of the decode. A second oracle told them apart. It supplied the
scanner's output instead, and then both together, with every output checked against the reference
across twenty paired rounds.

<figure class="bf-fig" id="interaction">
<p class="bf-title">One change, two values: the four-arm oracle at 10 workers</p>
<svg viewBox="0 0 720 352" role="img" aria-label="A square of four decoder configurations at 10 workers. Everything real takes 1.000; with emission's codes supplied 0.976, unresolved; with the scanner's output supplied 0.806; with both supplied 0.741. Supplying the codes is worth nothing along the top edge and 8% along the bottom, once the scanner has been relieved."><rect x="65.0" y="23.0" width="170" height="74" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-tm" x="150" y="52" text-anchor="middle" style="font-size:12.5px">everything real</text><text class="bf-t-strong" x="150" y="76" text-anchor="middle" style="font-size:15px">1.000</text><rect x="385.0" y="23.0" width="170" height="74" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-tm" x="470" y="52" text-anchor="middle" style="font-size:12.5px">emission's codes supplied</text><text class="bf-t-strong" x="470" y="76" text-anchor="middle" style="font-size:15px">0.976 [0.936, 1.017]</text><rect x="65.0" y="213.0" width="170" height="74" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-tm" x="150" y="242" text-anchor="middle" style="font-size:12.5px">scanner's output supplied</text><text class="bf-t-strong" x="150" y="266" text-anchor="middle" style="font-size:15px">0.806 [0.782, 0.831]</text><rect x="385.0" y="213.0" width="170" height="74" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-tm" x="470" y="242" text-anchor="middle" style="font-size:12.5px">both supplied</text><text class="bf-t-strong" x="470" y="266" text-anchor="middle" style="font-size:15px">0.741 [0.717, 0.766]</text><line x1="239.0" y1="60" x2="379.0" y2="60" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="379.0,60.0 370.0,55.0 370.0,65.0" style="fill:var(--bf-frame)"/><text class="bf-tm" x="310.0" y="50" text-anchor="middle" style="font-size:12px">supply emission's codes</text><text class="bf-t-strong" x="310.0" y="78" text-anchor="middle" style="font-size:13px;fill:var(--bf-blue)">−2.4%, unresolved</text><line x1="239.0" y1="250" x2="379.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="379.0,250.0 370.0,245.0 370.0,255.0" style="fill:var(--bf-frame)"/><text class="bf-tm" x="310.0" y="240" text-anchor="middle" style="font-size:12px">supply emission's codes</text><text class="bf-t-strong" x="310.0" y="268" text-anchor="middle" style="font-size:13px;fill:var(--bf-green)">−8%</text><text class="bf-tm" x="310.0" y="284" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-green)">0.920 [0.901, 0.939]</text><line x1="150" y1="101.0" x2="150" y2="207.0" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="150.0,207.0 155.0,198.0 145.0,198.0" style="fill:var(--bf-frame)"/><text class="bf-tm" x="138" y="159.0" text-anchor="end" style="font-size:12px">supply the scanner: −19%</text><line x1="470" y1="101.0" x2="470" y2="207.0" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="470.0,207.0 475.0,198.0 465.0,198.0" style="fill:var(--bf-frame)"/><text class="bf-tm" x="482" y="159.0" text-anchor="start" style="font-size:12px">−24%</text><text class="bf-tm" x="360.0" y="322" text-anchor="middle" style="font-size:12.5px">The same change, the codes supplied for emission, is worth nothing on the top edge and 8% on the bottom.</text><text class="bf-tm" x="360.0" y="340" text-anchor="middle" style="font-size:12.5px">At 1 and 6 workers, with the workers the bottleneck, it is worth 17% and 15% (0.830, 0.850).</text></svg>
<figcaption>Measured: journal §44, wall time over everything real, 95% intervals; the oracles' preparation is outside the clock and every output matches the reference.</figcaption>
</figure>

Once the scanner was relieved, supplying emission's codes was worth 8% (0.920 [0.901, 0.939] of
the scanner-supplied wall). So the same computation was worth improving in one configuration and
apparently worthless in another, and the question that follows is not where the time goes but
which work determines when the decode finishes. The 26% with both supplied measures room under an
experimental arrangement; finding an algorithm that obtains the supplied information cheaply is a
separate problem, and keeping the two apart is what lets such experiments direct mathematical work.

## 2. Queueing questions under other names

None of this was new territory. For a month the decoder's performance work had been asking
queueing questions without the vocabulary, because parallelising a program is largely a matter of
rearranging how its work queues: which stage hands what to whom, how many jobs are in flight, and
who waits when.

<figure class="bf-fig" id="timeline">
<p class="bf-title">Queueing questions under other names</p>
<svg viewBox="0 0 480 574" role="img" aria-label="A timeline from 27 October to 24 November of the decoder's performance questions, each with its name in queueing theory: the serial station, the knee, load-dependent service, capacity provisioning, Little's law, waiting and utilisation, admission and head-of-line blocking, discipline and placement, tandem stations and a masked bottleneck."><text class="bf-t-strong" x="16" y="18" text-anchor="start" style="font-size:13px">Performance questions, and their names in queueing theory</text><line x1="66" x2="66" y1="30" y2="556" style="stroke:var(--bf-frame);stroke-width:2"/><text class="bf-t-strong" x="54" y="43" text-anchor="end" style="font-size:12.5px">27 Oct</text><circle cx="66" cy="39" r="5" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="80" y="43" text-anchor="start" style="font-size:13px;fill:var(--bf-amber)">serial station</text><text class="bf-tm" x="80" y="61" text-anchor="start" style="font-size:12px">on CI, 4 workers give 1.5–1.9×: the serial scan limits, not the window</text><text class="bf-t-strong" x="54" y="85" text-anchor="end" style="font-size:12.5px">28 Oct</text><circle cx="66" cy="81" r="5" style="fill:var(--bf-blue)"/><text class="bf-t-strong" x="80" y="85" text-anchor="start" style="font-size:13px;fill:var(--bf-blue)">the knee; load-dependent service</text><text class="bf-tm" x="80" y="103" text-anchor="start" style="font-size:12px">the decoder peaks at 4 workers; IPC halves as workers are added</text><text class="bf-t-strong" x="54" y="127" text-anchor="end" style="font-size:12.5px">29 Oct</text><circle cx="66" cy="123" r="5" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="80" y="127" text-anchor="start" style="font-size:13px;fill:var(--bf-green)">capacity provisioning</text><text class="bf-tm" x="80" y="145" text-anchor="start" style="font-size:12px">teams of two, as many as keep the tables near 0.7 of the L3</text><text class="bf-t-strong" x="54" y="169" text-anchor="end" style="font-size:12.5px">31 Oct</text><circle cx="66" cy="165" r="5" style="fill:var(--bf-steel)"/><text class="bf-t-strong" x="80" y="169" text-anchor="start" style="font-size:13px;fill:var(--bf-steel)">Little's law</text><text class="bf-tm" x="80" y="187" text-anchor="start" style="font-size:12px">lookups in flight = latency × rate inside one worker's walk</text><text class="bf-t-strong" x="54" y="211" text-anchor="end" style="font-size:12.5px">1 Nov</text><circle cx="66" cy="207" r="5" style="fill:var(--bf-steel)"/><text class="bf-t-strong" x="80" y="211" text-anchor="start" style="font-size:13px;fill:var(--bf-steel)">waiting time and utilisation</text><text class="bf-tm" x="80" y="229" text-anchor="start" style="font-size:12px">slow job claims are mostly waiting behind busy workers</text><text class="bf-t-strong" x="54" y="253" text-anchor="end" style="font-size:12.5px">1 Nov</text><circle cx="66" cy="249" r="5" style="fill:var(--bf-blue)"/><text class="bf-t-strong" x="80" y="253" text-anchor="start" style="font-size:13px;fill:var(--bf-blue)">load-dependent service</text><text class="bf-tm" x="80" y="271" text-anchor="start" style="font-size:12px">four workers spend 32% more CPU a byte on the same instructions</text><text class="bf-t-strong" x="54" y="295" text-anchor="end" style="font-size:12.5px">5 Nov</text><circle cx="66" cy="291" r="5" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="80" y="295" text-anchor="start" style="font-size:13px;fill:var(--bf-green)">admission; head-of-line blocking</text><text class="bf-tm" x="80" y="313" text-anchor="start" style="font-size:12px">the window from 2 to 4 slots a thread: 8–10% off the wall</text><text class="bf-t-strong" x="54" y="337" text-anchor="end" style="font-size:12.5px">5 Nov</text><circle cx="66" cy="333" r="5" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="80" y="337" text-anchor="start" style="font-size:13px;fill:var(--bf-green)">discipline and placement</text><text class="bf-tm" x="80" y="355" text-anchor="start" style="font-size:12px">co-run placement: thread pairs, a walk token, interleaving</text><text class="bf-t-strong" x="54" y="379" text-anchor="end" style="font-size:12.5px">7 Nov</text><circle cx="66" cy="375" r="5" style="fill:var(--bf-blue)"/><text class="bf-t-strong" x="80" y="379" text-anchor="start" style="font-size:13px;fill:var(--bf-blue)">the knee; a cycle time of max(S, αP)</text><text class="bf-tm" x="80" y="397" text-anchor="start" style="font-size:12px">each block in flight slows the others; three teams beat four</text><text class="bf-t-strong" x="54" y="421" text-anchor="end" style="font-size:12.5px">9 Nov</text><circle cx="66" cy="417" r="5" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="80" y="421" text-anchor="start" style="font-size:13px;fill:var(--bf-amber)">tandem stations; bottleneck</text><text class="bf-tm" x="80" y="439" text-anchor="start" style="font-size:12px">a pit crew of service threads: the single thread became the bottleneck</text><text class="bf-t-strong" x="54" y="463" text-anchor="end" style="font-size:12.5px">17 Nov</text><circle cx="66" cy="459" r="5" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="80" y="463" text-anchor="start" style="font-size:13px;fill:var(--bf-green)">capacity provisioning</text><text class="bf-tm" x="80" y="481" text-anchor="start" style="font-size:12px">seven workers of ten finish sooner</text><text class="bf-t-strong" x="54" y="505" text-anchor="end" style="font-size:12.5px">21 Nov</text><circle cx="66" cy="501" r="5" style="fill:var(--bf-blue)"/><text class="bf-t-strong" x="80" y="505" text-anchor="start" style="font-size:13px;fill:var(--bf-blue)">load-dependent service</text><text class="bf-tm" x="80" y="523" text-anchor="start" style="font-size:12px">fast alone, slow in a crowd</text><text class="bf-t-strong" x="54" y="547" text-anchor="end" style="font-size:12.5px">24 Nov</text><circle cx="66" cy="543" r="5" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="80" y="547" text-anchor="start" style="font-size:13px;fill:var(--bf-amber)">a masked bottleneck</text><text class="bf-tm" x="80" y="565" text-anchor="start" style="font-size:12px">the scanner hides the emitter</text></svg>
<figcaption>From the journals and posts linked in this section. Colours group the names: amber the serial station and bottleneck, blue load dependence, green provisioning and admission, steel Little's law and waiting.</figcaption>
</figure>

The first knee was measured on 28 October: the whole decoder peaked at 4 threads, at 1,013 MB/s,
and fell to 0.63 of that by 10; from 6 threads to 10 its instructions stayed constant while
instructions per cycle halved ([journal](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-10-28-workstation-scaling.md)). Each added worker made every worker slower, which queueing theory calls a
load-dependent service rate, and the same finding recurs as 32% more CPU a byte for four
workers ([memory interference](2026-11-01-inverse-bwt-memory-interference.md)), as kernels that win
alone and lose in a crowd ([fast alone, slow in a crowd](2026-11-21-fast-alone-slow-in-a-crowd.md)),
and as the reason a lazier kernel with 28% more instructions wins from 8 threads
([lazy letters](2026-11-16-lazy-letters.md)). Slow job claims turned out to be ordinary waiting
behind busy workers ([block dispatch](2026-11-01-block-dispatch.md)). Little's law was already in
use, to count the lookups in flight inside one worker's walk
([32 lanes, 8.6 chains](2026-10-31-room-walk-overlap.md)). Widening the admission window from 2 to
4 slots a worker took 8–10% off the wall on CI, because workers had been idle behind a block still
decoding or not yet read ([journal](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-05-window.md)). Teams of threads, pipelined teams and a crew of
service threads were all ways of restructuring stations
([why mibz2 stops scaling](2026-11-07-why-mibz2-stops-scaling.md);
[pipelined teams](2026-11-07-pipelined-teams.md); [a pit crew](2026-11-09-pit-crew-service-threads.md)),
and in the last of them a single service thread busy 96% of the time had become the bottleneck.
Rightsizing to seven workers of ten was capacity provisioning under load dependence
([modelling the cache](2026-11-17-modelling-the-cache-to-rightsize-workers.md)).

Underneath all of it sat one structural fact. The scanner that decodes the Huffman codes is a
single thread working in order, because a block's end is found only by reading up to its
end-of-block symbol, and the next block begins straight after it
([the bit buffer](2026-11-05-huffman-bit-buffer.md)). Everything else runs in parallel behind it.
The previous post called the scanner the doorman and the workers the assemblers; this one gives
that picture its mathematics.

## 3. What the theory was asked for

The aim was set out before any reading began: a performance analysis in Jain's manner, which would
yield a model of the program in the way systems biology models a regulatory network, which would in
turn locate where new mathematical devices could fit ([journal §52](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). That asks three things
of queueing theory. It should characterise the decoder as it is, as a system of stations rather than
a list of kernels. It should predict what a change would buy before the change is built. And it
should help search for a different organisation of the work, if a better one exists.

The theory came from two teachers. Raj Jain's [course](https://www.cs.wustl.edu/~jain/iucee/index.html)
follows his *The Art of Computer Systems Performance Analysis* (1991), and Mor Harchol-Balter's
[15-857](https://www.cs.cmu.edu/~harchol/Perfclass/class.html) at Carnegie Mellon, with its theory
companion [15-849b](https://www.cs.cmu.edu/~harchol/Perfclass/classtheory.html), follows her
*Performance Modeling and Design of Computer Systems* (2013). What was read is narrower than
reading three courses: every module's topic list, six of Jain's slide decks in full (the systematic
approach, experimental design, fractional factorials, simulation analysis, the operational laws and
mean-value analysis), the textbook's table of contents, the 2025 schedule with its readings, and the
theory course's topic and book lists, all saved with notes ([what was read](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md)).
Four readers then mined that material against the decoder's measurements independently, and their
disagreements corrected the first reading in several places that recur below
([the summary](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md); [journal §§58–62](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 4. The decoder, drawn as stations

In queueing theory's terms a job here is a block of the compressed file, and a station is a resource
that serves it: the scanner, the pool of workers, the reader. Each block makes a demand on each
station, the time it needs there, and the stations' throughputs and utilisations follow from those
demands; the vocabulary, and the shapes in which queues are combined, are set out in the
[technical appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md#2-vocabulary-and-shapes).

The decoder is a tandem: the scanner feeds a parallel group of k workers, each of which takes a
whole block, and a reader hands the finished blocks to the caller in order. Its admission is
precise in the code ([`src/parallel.rs`](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/src/parallel.rs#L399-L408)). With issued counting the blocks the
scanner has finished and taken the blocks the reader has taken, the scanner waits while
issued − taken has reached W = 4k, or while 12 MiB of finished output is unread and at least
k + 2 blocks are ahead. The block it is scanning is not yet counted, and a slot frees when the
reader takes the next block in order, before it has copied any of it out
([taken advances](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/src/parallel.rs#L983-L986)).

<figure class="bf-fig" id="network">
<p class="bf-title">The decoder as a network of queues</p>
<svg viewBox="0 0 720 300" role="img" aria-label="The decoder as a network of queues, close to a closed one. A feeder passes input to a single scanner thread, which decodes headers, tables and Huffman codes. Blocks enter an admission window holding W = 4k blocks in flight, then k workers undo move-to-front and runs, invert and emit, then a consumer takes blocks in order and writes the output. A slot returns to the window only when the next block in order is taken, and finished output is capped at 12 MiB, so one slow early block can stall the scanner."><rect x="10" y="40" width="96" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="58.0" y="62" text-anchor="middle" style="font-size:13px">feeder</text><text class="bf-tm" x="58.0" y="80" text-anchor="middle" style="font-size:11px">reads and</text><text class="bf-tm" x="58.0" y="95" text-anchor="middle" style="font-size:11px">copies input</text><rect x="128" y="40" width="122" height="92" rx="8" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="189.0" y="62" text-anchor="middle" style="font-size:13px">scanner</text><text class="bf-tm" x="189.0" y="80" text-anchor="middle" style="font-size:11px">one thread:</text><text class="bf-tm" x="189.0" y="95" text-anchor="middle" style="font-size:11px">headers, tables,</text><text class="bf-tm" x="189.0" y="110" text-anchor="middle" style="font-size:11px">Huffman decode</text><rect x="272" y="40" width="170" height="92" rx="8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text class="bf-t-strong" x="357.0" y="62" text-anchor="middle" style="font-size:13px">admission window</text><text class="bf-tm" x="357.0" y="80" text-anchor="middle" style="font-size:11px">W = 4k blocks in flight:</text><text class="bf-tm" x="357.0" y="95" text-anchor="middle" style="font-size:11px">queued, running, or done</text><text class="bf-tm" x="357.0" y="110" text-anchor="middle" style="font-size:11px">but not yet taken in order</text><rect x="464" y="40" width="122" height="92" rx="8" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="525.0" y="62" text-anchor="middle" style="font-size:13px">k workers</text><text class="bf-tm" x="525.0" y="80" text-anchor="middle" style="font-size:11px">MTF and runs,</text><text class="bf-tm" x="525.0" y="95" text-anchor="middle" style="font-size:11px">inversion,</text><text class="bf-tm" x="525.0" y="110" text-anchor="middle" style="font-size:11px">emission, CRC</text><rect x="608" y="40" width="102" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="659.0" y="62" text-anchor="middle" style="font-size:13px">consumer</text><text class="bf-tm" x="659.0" y="80" text-anchor="middle" style="font-size:11px">takes blocks</text><text class="bf-tm" x="659.0" y="95" text-anchor="middle" style="font-size:11px">in order,</text><text class="bf-tm" x="659.0" y="110" text-anchor="middle" style="font-size:11px">writes output</text><path d="M106,86.0 L120,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="128,86.0 120,81.0 120,91.0" style="fill:var(--bf-frame)"/><path d="M250,86.0 L264,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="272,86.0 264,81.0 264,91.0" style="fill:var(--bf-frame)"/><path d="M442,86.0 L456,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="464,86.0 456,81.0 456,91.0" style="fill:var(--bf-frame)"/><path d="M586,86.0 L600,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="608,86.0 600,81.0 600,91.0" style="fill:var(--bf-frame)"/><path d="M659,132 L659,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M659,184 L357,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M357,184 L357,141" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><polygon points="357,133 352,141 362,141" style="fill:var(--bf-steel)"/><text class="bf-tm" x="508" y="202" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">a slot is freed only when the next block in order is taken,</text><text class="bf-tm" x="508" y="217" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">and finished output is capped at 12 MiB</text><text class="bf-tm" x="10" y="244" style="font-size:11.5px">Admission credit circulates, though each block passes through once: close to a closed network,</text><text class="bf-tm" x="10" y="260" style="font-size:11.5px">not one. One slow early block can hold slots and stall the scanner.</text></svg>
<figcaption>The structure the measurements follow. Admission credit circulates; the data passes
through once. Start-up, drain and the 12 MiB cap on finished output are where the decoder departs
from a true closed network.</figcaption>
</figure>

That makes four distinct kinds of waiting, with different remedies: the scanner waiting for an
admission slot, a scanned block waiting for a worker, a worker waiting for a scanned block, and a
finished block waiting behind an earlier one that is not finished. More slots can keep workers busy
in the second and third cases; in the fourth they only hold more memory behind a straggler.

The return arrow is why the decoder was first described as a closed network, in which a fixed
population of jobs circulates. That description was withdrawn
([the summary](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md)). The decoder is a finite pipeline with
feedback-controlled admission: a finite stream starts empty and drains, W is a ceiling rather than a
population, and the byte cap and the in-order reader make admission depend on the decoder's state.
A closed network whose population is the admission permits is a useful surrogate, provided its
relation to the real blocks is stated whenever it is used.

## 5. The bottleneck law

The operational laws relate quantities measured over an interval and assume nothing about how
arrivals or service times are distributed (Jain, ch. 33; Harchol-Balter, ch. 6–7). The utilisation
law, U = X·D, says a station's busy fraction is its throughput times its demand. No station can be
busy more than all of the time, so U ≤ 1 gives the bottleneck law: the system's throughput is at
most the capacity of its busiest station over that station's demand.

Applied to a scanner taking s a block and k workers each taking w, the scanner can finish at most
1/s blocks a second and the workers together k/w, so X(k) ≤ min(k/w, 1/s). The workers set the pace
until there are enough of them that the scanner cannot keep up, and from then on the scanner alone
does; the two limits meet at k\* = w/s. From measurements at one to four workers only, s = 106.6 µs
and w = 798.3 µs a block, which puts the knee at 7.49 workers. The measured crossing is 7.67, and
the best wall, 0.191 s, is at 8 ([journal §57](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the data](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv)). Nothing was fitted to
the knee: two service times measured where the workers bind located the region where the scanner
takes over.

<figure class="bf-fig bf-twostation" id="two-station" data-x0="70" data-x1="700" data-y0="24" data-y1="304" data-ymax="20000">
<p class="bf-title">The bottleneck law, with the measured decoder</p>
<svg viewBox="0 0 720 360" role="img" aria-label="The two-station bound on throughput against workers from 1 to 20: the workers' limit k/w rises in a straight line until it meets the scanner's ceiling 1/s at the knee, 7.49 workers with the measured service times, 106.6 µs for the scanner and 798.3 µs for a worker; the measured decoder follows the bound closely, just below it, peaking at about 9,000 blocks a second at 8 workers."><defs><clipPath id="ts-clip"><rect x="70" y="24" width="630" height="280"/></clipPath></defs><line class="bf-rule" x1="70" x2="700" y1="304.0" y2="304.0" opacity="0.9"/><text class="bf-tm" x="62" y="308.0" text-anchor="end" style="font-size:12px">0k</text><line class="bf-rule" x1="70" x2="700" y1="234.0" y2="234.0" opacity="0.35"/><text class="bf-tm" x="62" y="238.0" text-anchor="end" style="font-size:12px">5k</text><line class="bf-rule" x1="70" x2="700" y1="164.0" y2="164.0" opacity="0.35"/><text class="bf-tm" x="62" y="168.0" text-anchor="end" style="font-size:12px">10k</text><line class="bf-rule" x1="70" x2="700" y1="94.0" y2="94.0" opacity="0.35"/><text class="bf-tm" x="62" y="98.0" text-anchor="end" style="font-size:12px">15k</text><line class="bf-rule" x1="70" x2="700" y1="24.0" y2="24.0" opacity="0.35"/><text class="bf-tm" x="62" y="28.0" text-anchor="end" style="font-size:12px">20k</text><text class="bf-tm" x="70.0" y="322" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="169.5" y="322" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="302.1" y="322" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="434.7" y="322" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="567.4" y="322" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="700.0" y="322" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="385" y="342" text-anchor="middle" style="font-size:12px">workers, k</text><text class="bf-tm" x="16" y="164" text-anchor="middle" transform="rotate(-90 16 164)" style="font-size:12px">blocks a second</text><g clip-path="url(#ts-clip)"><polyline class="bf-ts-work" points="70.0,286.5 73.3,284.7 76.6,283.0 79.9,281.2 83.3,279.4 86.6,277.7 89.9,275.9 93.2,274.2 96.5,272.4 99.8,270.7 103.2,268.9 106.5,267.2 109.8,265.4 113.1,263.7 116.4,261.9 119.7,260.2 123.1,258.4 126.4,256.6 129.7,254.9 133.0,253.1 136.3,251.4 139.6,249.6 142.9,247.9 146.3,246.1 149.6,244.4 152.9,242.6 156.2,240.9 159.5,239.1 162.8,237.4 166.2,235.6 169.5,233.9 172.8,232.1 176.1,230.3 179.4,228.6 182.7,226.8 186.1,225.1 189.4,223.3 192.7,221.6 196.0,219.8 199.3,218.1 202.6,216.3 205.9,214.6 209.3,212.8 212.6,211.1 215.9,209.3 219.2,207.5 222.5,205.8 225.8,204.0 229.2,202.3 232.5,200.5 235.8,198.8 239.1,197.0 242.4,195.3 245.7,193.5 249.1,191.8 252.4,190.0 255.7,188.3 259.0,186.5 262.3,184.7 265.6,183.0 268.9,181.2 272.3,179.5 275.6,177.7 278.9,176.0 282.2,174.2 285.5,172.5 288.8,170.7 292.2,169.0 295.5,167.2 298.8,165.5 302.1,163.7 305.4,161.9 308.7,160.2 312.1,158.4 315.4,156.7 318.7,154.9 322.0,153.2 325.3,151.4 328.6,149.7 331.9,147.9 335.3,146.2 338.6,144.4 341.9,142.7 345.2,140.9 348.5,139.1 351.8,137.4 355.2,135.6 358.5,133.9 361.8,132.1 365.1,130.4 368.4,128.6 371.7,126.9 375.1,125.1 378.4,123.4 381.7,121.6 385.0,119.9 388.3,118.1 391.6,116.4 394.9,114.6 398.3,112.8 401.6,111.1 404.9,109.3 408.2,107.6 411.5,105.8 414.8,104.1 418.2,102.3 421.5,100.6 424.8,98.8 428.1,97.1 431.4,95.3 434.7,93.6 438.1,91.8 441.4,90.0 444.7,88.3 448.0,86.5 451.3,84.8 454.6,83.0 457.9,81.3 461.3,79.5 464.6,77.8 467.9,76.0 471.2,74.3 474.5,72.5 477.8,70.8 481.2,69.0 484.5,67.2 487.8,65.5 491.1,63.7 494.4,62.0 497.7,60.2 501.1,58.5 504.4,56.7 507.7,55.0 511.0,53.2 514.3,51.5 517.6,49.7 520.9,48.0 524.3,46.2 527.6,44.4 530.9,42.7 534.2,40.9 537.5,39.2 540.8,37.4 544.2,35.7 547.5,33.9 550.8,32.2 554.1,30.4 557.4,28.7 560.7,26.9 564.1,25.2 567.4,24.0 570.7,24.0 574.0,24.0 577.3,24.0 580.6,24.0 583.9,24.0 587.3,24.0 590.6,24.0 593.9,24.0 597.2,24.0 600.5,24.0 603.8,24.0 607.2,24.0 610.5,24.0 613.8,24.0 617.1,24.0 620.4,24.0 623.7,24.0 627.1,24.0 630.4,24.0 633.7,24.0 637.0,24.0 640.3,24.0 643.6,24.0 646.9,24.0 650.3,24.0 653.6,24.0 656.9,24.0 660.2,24.0 663.5,24.0 666.8,24.0 670.2,24.0 673.5,24.0 676.8,24.0 680.1,24.0 683.4,24.0 686.7,24.0 690.1,24.0 693.4,24.0 696.7,24.0 700.0,24.0" style="fill:none;stroke:var(--bf-blue);stroke-width:1.5;stroke-dasharray:5 4"/><line class="bf-ts-scan" x1="70" x2="700" y1="172.7" y2="172.7" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><polyline class="bf-ts-bound" points="70.0,286.5 73.3,284.7 76.6,283.0 79.9,281.2 83.3,279.4 86.6,277.7 89.9,275.9 93.2,274.2 96.5,272.4 99.8,270.7 103.2,268.9 106.5,267.2 109.8,265.4 113.1,263.7 116.4,261.9 119.7,260.2 123.1,258.4 126.4,256.6 129.7,254.9 133.0,253.1 136.3,251.4 139.6,249.6 142.9,247.9 146.3,246.1 149.6,244.4 152.9,242.6 156.2,240.9 159.5,239.1 162.8,237.4 166.2,235.6 169.5,233.9 172.8,232.1 176.1,230.3 179.4,228.6 182.7,226.8 186.1,225.1 189.4,223.3 192.7,221.6 196.0,219.8 199.3,218.1 202.6,216.3 205.9,214.6 209.3,212.8 212.6,211.1 215.9,209.3 219.2,207.5 222.5,205.8 225.8,204.0 229.2,202.3 232.5,200.5 235.8,198.8 239.1,197.0 242.4,195.3 245.7,193.5 249.1,191.8 252.4,190.0 255.7,188.3 259.0,186.5 262.3,184.7 265.6,183.0 268.9,181.2 272.3,179.5 275.6,177.7 278.9,176.0 282.2,174.2 285.5,172.7 288.8,172.7 292.2,172.7 295.5,172.7 298.8,172.7 302.1,172.7 305.4,172.7 308.7,172.7 312.1,172.7 315.4,172.7 318.7,172.7 322.0,172.7 325.3,172.7 328.6,172.7 331.9,172.7 335.3,172.7 338.6,172.7 341.9,172.7 345.2,172.7 348.5,172.7 351.8,172.7 355.2,172.7 358.5,172.7 361.8,172.7 365.1,172.7 368.4,172.7 371.7,172.7 375.1,172.7 378.4,172.7 381.7,172.7 385.0,172.7 388.3,172.7 391.6,172.7 394.9,172.7 398.3,172.7 401.6,172.7 404.9,172.7 408.2,172.7 411.5,172.7 414.8,172.7 418.2,172.7 421.5,172.7 424.8,172.7 428.1,172.7 431.4,172.7 434.7,172.7 438.1,172.7 441.4,172.7 444.7,172.7 448.0,172.7 451.3,172.7 454.6,172.7 457.9,172.7 461.3,172.7 464.6,172.7 467.9,172.7 471.2,172.7 474.5,172.7 477.8,172.7 481.2,172.7 484.5,172.7 487.8,172.7 491.1,172.7 494.4,172.7 497.7,172.7 501.1,172.7 504.4,172.7 507.7,172.7 511.0,172.7 514.3,172.7 517.6,172.7 520.9,172.7 524.3,172.7 527.6,172.7 530.9,172.7 534.2,172.7 537.5,172.7 540.8,172.7 544.2,172.7 547.5,172.7 550.8,172.7 554.1,172.7 557.4,172.7 560.7,172.7 564.1,172.7 567.4,172.7 570.7,172.7 574.0,172.7 577.3,172.7 580.6,172.7 583.9,172.7 587.3,172.7 590.6,172.7 593.9,172.7 597.2,172.7 600.5,172.7 603.8,172.7 607.2,172.7 610.5,172.7 613.8,172.7 617.1,172.7 620.4,172.7 623.7,172.7 627.1,172.7 630.4,172.7 633.7,172.7 637.0,172.7 640.3,172.7 643.6,172.7 646.9,172.7 650.3,172.7 653.6,172.7 656.9,172.7 660.2,172.7 663.5,172.7 666.8,172.7 670.2,172.7 673.5,172.7 676.8,172.7 680.1,172.7 683.4,172.7 686.7,172.7 690.1,172.7 693.4,172.7 696.7,172.7 700.0,172.7" style="fill:none;stroke:var(--bf-steel);stroke-width:3"/><line class="bf-ts-kline" x1="285.2" x2="285.2" y1="172.7" y2="304" style="stroke:var(--bf-steel);stroke-width:1;stroke-dasharray:3 3"/><circle class="bf-ts-knee" cx="285.2" cy="172.7" r="5" style="fill:var(--bf-steel)"/></g><circle cx="70.0" cy="285.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="103.2" cy="268.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="136.3" cy="252.6" r="3.5" style="fill:var(--bf-green)"/><circle cx="169.5" cy="236.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="202.6" cy="219.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="235.8" cy="202.6" r="3.5" style="fill:var(--bf-green)"/><circle cx="268.9" cy="187.3" r="3.5" style="fill:var(--bf-green)"/><circle cx="302.1" cy="177.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="335.3" cy="181.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="368.4" cy="184.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="401.6" cy="181.3" r="3.5" style="fill:var(--bf-green)"/><circle cx="434.7" cy="181.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="467.9" cy="181.2" r="3.5" style="fill:var(--bf-green)"/><circle cx="501.1" cy="182.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="534.2" cy="189.8" r="3.5" style="fill:var(--bf-green)"/><circle cx="567.4" cy="192.4" r="3.5" style="fill:var(--bf-green)"/><circle cx="600.5" cy="200.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="633.7" cy="200.4" r="3.5" style="fill:var(--bf-green)"/><circle cx="666.8" cy="199.9" r="3.5" style="fill:var(--bf-green)"/><circle cx="700.0" cy="199.2" r="3.5" style="fill:var(--bf-green)"/><text class="bf-t-strong bf-ts-kstar" x="700" y="40" text-anchor="end" style="font-size:15px">knee k* = w/s = 7.49 workers</text><text class="bf-tm bf-ts-ceil" x="700" y="60" text-anchor="end" style="font-size:13px">scanner ceiling 1/s = 9,381 blocks a second</text><rect x="367.05263157894734" y="213.6" width="328.94736842105266" height="76" rx="4" style="fill:var(--bf-surface);opacity:0.9"/><line x1="375.05263157894734" x2="397.05263157894734" y1="225.6" y2="225.6" style="stroke:var(--bf-steel);stroke-width:3"/><text class="bf-tm" x="405.05263157894734" y="229.6" style="font-size:12.5px">the bound, min(k/w, 1/s)</text><line x1="375.05263157894734" x2="397.05263157894734" y1="243.6" y2="243.6" style="stroke:var(--bf-blue);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-tm" x="405.05263157894734" y="247.6" style="font-size:12.5px">the workers' limit, k/w</text><line x1="375.05263157894734" x2="397.05263157894734" y1="261.6" y2="261.6" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-tm" x="405.05263157894734" y="265.6" style="font-size:12.5px">the scanner's limit, 1/s</text><circle cx="386.05263157894734" cy="279.6" r="3.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="405.05263157894734" y="283.6" style="font-size:12.5px">measured: 1,724 blocks over the median wall</text></svg>
<div class="bf-ts-controls" hidden><label>scanner, s <input class="bf-ts-s" type="range" min="50" max="250" step="0.1" value="106.6" aria-label="scanner service time, microseconds a block"> <output class="bf-ts-sv">106.6 µs</output></label><label>worker, w <input class="bf-ts-w" type="range" min="200" max="1600" step="0.1" value="798.3" aria-label="worker service time, microseconds a block"> <output class="bf-ts-wv">798.3 µs</output></label><button type="button" class="bf-ts-reset">Reset to the measured values</button></div>
<figcaption>The idealised bound from the operational laws, with no start-up, drain, contention or blocking, against the decoder measured on an i9-10900K (median of 20 decodes at each count). With JavaScript, the sliders set the two service times: a cheaper scanner raises the ceiling and moves the knee right, a cheaper worker steepens the rise and moves it left.</figcaption>
</figure>

In wall time, the same picture reads as follows.

<figure class="bf-fig" id="knee">
<p class="bf-title">Wall time against workers: the knee predicted from four counts</p>
<svg viewBox="0 0 720 400" role="img" aria-label="Decode wall time against workers from 1 to 20 on the i9-10900K. It falls as a over k, from 1.30 s at one worker, to a best of 0.191 s at 8 workers, stays near 0.197 s from 8 to 14, and steps up to about 0.23 s from 16. The knee predicted from counts 1 to 4 alone is 7.49 workers; the measured crossover is 7.67."><line x1="64" x2="696" y1="282.3" y2="282.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="286.3" text-anchor="end" style="font-size:12px">0.1</text><line x1="64" x2="696" y1="238.6" y2="238.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="242.6" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="194.9" y2="194.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="198.9" text-anchor="end" style="font-size:12px">0.3</text><line x1="64" x2="696" y1="151.1" y2="151.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="155.1" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="107.4" y2="107.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="111.4" text-anchor="end" style="font-size:12px">0.5</text><line x1="64" x2="696" y1="63.7" y2="63.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="67.7" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">0.7</text><text class="bf-tm" x="64.0" y="344" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="344" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="344" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="344" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="344" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="344" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="362" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="173" text-anchor="middle" transform="rotate(-90 16 173)" style="font-size:12px">decode wall time (s)</text><polyline points="64.0,20.0 67.3,20.0 70.7,20.0 74.0,20.0 77.3,20.0 80.6,20.0 84.0,20.0 87.3,20.0 90.6,20.0 93.9,20.0 97.3,32.5 100.6,45.8 103.9,57.9 107.2,68.9 110.6,79.1 113.9,88.4 117.2,97.0 120.5,104.9 123.9,112.3 127.2,119.2 130.5,125.6 133.9,131.7 137.2,137.3 140.5,142.6 143.8,147.6 147.2,152.3 150.5,156.7 153.8,160.9 157.1,164.9 160.5,168.7 163.8,172.2 167.1,175.6 170.4,178.9 173.8,182.0 177.1,184.9 180.4,187.8 183.7,190.5 187.1,193.1 190.4,195.5 193.7,197.9 197.1,200.2 200.4,202.4 203.7,204.5 207.0,206.5 210.4,208.5 213.7,210.4 217.0,212.2 220.3,213.9 223.7,215.6 227.0,217.2 230.3,218.8 233.6,220.4 237.0,221.8 240.3,223.3 243.6,224.7 246.9,226.0 250.3,227.3 253.6,228.6 256.9,229.8 260.3,231.0 263.6,232.1 266.9,233.3 270.2,234.2 273.6,234.2 276.9,234.2 280.2,234.2 283.5,234.2 286.9,234.2 290.2,234.2 293.5,234.2 296.8,234.2 300.2,234.2 303.5,234.2 306.8,234.2 310.1,234.2 313.5,234.2 316.8,234.2 320.1,234.2 323.5,234.2 326.8,234.2 330.1,234.2 333.4,234.2 336.8,234.2 340.1,234.2 343.4,234.2 346.7,234.2 350.1,234.2 353.4,234.2 356.7,234.2 360.0,234.2 363.4,234.2 366.7,234.2 370.0,234.2 373.3,234.2 376.7,234.2 380.0,234.2 383.3,234.2 386.7,234.2 390.0,234.2 393.3,234.2 396.6,234.2 400.0,234.2 403.3,234.2 406.6,234.2 409.9,234.2 413.3,234.2 416.6,234.2 419.9,234.2 423.2,234.2 426.6,234.2 429.9,234.2 433.2,234.2 436.5,234.2 439.9,234.2 443.2,234.2 446.5,234.2 449.9,234.2 453.2,234.2 456.5,234.2 459.8,234.2 463.2,234.2 466.5,234.2 469.8,234.2 473.1,234.2 476.5,234.2 479.8,234.2 483.1,234.2 486.4,234.2 489.8,234.2 493.1,234.2 496.4,234.2 499.7,234.2 503.1,234.2 506.4,234.2 509.7,234.2 513.1,234.2 516.4,234.2 519.7,234.2 523.0,234.2 526.4,234.2 529.7,234.2 533.0,234.2 536.3,234.2 539.7,234.2 543.0,234.2 546.3,234.2 549.6,234.2 553.0,234.2 556.3,234.2 559.6,234.2 562.9,234.2 566.3,234.2 569.6,234.2 572.9,234.2 576.3,234.2 579.6,234.2 582.9,234.2 586.2,234.2 589.6,234.2 592.9,234.2 596.2,234.2 599.5,234.2 602.9,234.2 606.2,234.2 609.5,234.2 612.8,234.2 616.2,234.2 619.5,234.2 622.8,234.2 626.1,234.2 629.5,234.2 632.8,234.2 636.1,234.2 639.5,234.2 642.8,234.2 646.1,234.2 649.4,234.2 652.8,234.2 656.1,234.2 659.4,234.2 662.7,234.2 666.1,234.2 669.4,234.2 672.7,234.2 676.0,234.2 679.4,234.2 682.7,234.2 686.0,234.2 689.3,234.2 692.7,234.2 696.0,234.2" style="fill:none;stroke:var(--bf-steel);stroke-width:2"/><line x1="279.8" x2="279.8" y1="20" y2="326" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-t-strong" x="285.8" y="36" style="font-size:12px;fill:var(--bf-amber)">knee predicted from counts 1–4: 7.49</text><text class="bf-tm" x="285.8" y="52" style="font-size:12px">measured crossover: 7.67</text><path d="M64.0,22 l-5,8 h10 z" style="fill:var(--bf-green)"/><text class="bf-tm" x="72.0" y="30" style="font-size:11px">1.30 s</text><circle cx="97.3" cy="27.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="130.5" cy="120.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="163.8" cy="170.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="197.1" cy="201.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="230.3" cy="221.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="263.6" cy="235.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="296.8" cy="242.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="330.1" cy="239.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="363.4" cy="238.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="396.6" cy="240.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="429.9" cy="239.7" r="4.5" style="fill:var(--bf-green)"/><circle cx="463.2" cy="240.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="496.4" cy="239.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="529.7" cy="233.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="562.9" cy="231.4" r="4.5" style="fill:var(--bf-green)"/><circle cx="596.2" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="629.5" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="662.7" cy="224.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="696.0" cy="225.3" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="137.2" y="129.3" style="font-size:12px">workers bind: wall ≈ a/k</text><text class="bf-tm" x="396.6" y="212.3" style="font-size:12px">scanner binds: wall ≈ b</text><text class="bf-tm" x="549.6" y="194.9" style="font-size:12px">step at 16: the scanner slows 23%</text><circle cx="74" cy="388" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="84" y="392" style="font-size:12px">measured (median of 20 decodes)</text><line x1="324" x2="354" y1="388" y2="388" style="stroke:var(--bf-steel);stroke-width:2"/><text class="bf-tm" x="362" y="392" style="font-size:12px">fitted two-station model, c + max(b, a/k)</text></svg>
<figcaption>Measured on an i9-10900K (10 cores, 20 threads), one corpus of 1,724 blocks and 839 MB
of output, median of 20 decodes at each count, unpinned. The fitted model (steel) has the workers'
asymptote a/k on the left and the scanner's b on the right; the dashed line is the knee predicted
from counts 1 to 4 alone. The wall at one worker, 1.30 s, is off the scale.</figcaption>
</figure>

<figure class="bf-fig" id="scanner-split">
<p class="bf-title">The scanner's time, working and waiting</p>
<svg viewBox="0 0 720 350" role="img" aria-label="The scanner thread's time against workers. Its work stays near 0.19 s from 3 to 15 workers and rises to about 0.23 s from 16. Its waiting for a free slot falls from 1.12 s at one worker to under a millisecond at 8 and beyond."><line x1="64" x2="696" y1="239.4" y2="239.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="243.4" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="202.9" y2="202.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="206.9" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="166.3" y2="166.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="170.3" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="129.7" y2="129.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="133.7" text-anchor="end" style="font-size:12px">0.8</text><line x1="64" x2="696" y1="93.1" y2="93.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="97.1" text-anchor="end" style="font-size:12px">1</text><line x1="64" x2="696" y1="56.6" y2="56.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="60.6" text-anchor="end" style="font-size:12px">1.2</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">1.4</text><text class="bf-tm" x="64.0" y="294" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="294" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="294" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="294" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="294" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="294" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="312" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="148" text-anchor="middle" transform="rotate(-90 16 148)" style="font-size:12px">scanner thread time (s)</text><rect x="54.2" y="244.2" width="19.6" height="31.8" style="fill:var(--bf-amber)"/><rect x="54.2" y="39.5" width="19.6" height="204.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="87.5" y="242.7" width="19.6" height="33.3" style="fill:var(--bf-amber)"/><rect x="87.5" y="151.6" width="19.6" height="91.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="120.7" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="120.7" y="191.4" width="19.6" height="50.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="154.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="154.0" y="212.1" width="19.6" height="30.0" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="187.3" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="187.3" y="225.2" width="19.6" height="16.9" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="220.5" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="220.5" y="233.5" width="19.6" height="8.6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="253.8" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="253.8" y="239.2" width="19.6" height="2.8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="287.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="287.0" y="241.8" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="320.3" y="241.6" width="19.6" height="34.4" style="fill:var(--bf-amber)"/><rect x="320.3" y="240.9" width="19.6" height="0.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="353.6" y="241.4" width="19.6" height="34.6" style="fill:var(--bf-amber)"/><rect x="353.6" y="240.9" width="19.6" height="0.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="386.8" y="241.3" width="19.6" height="34.7" style="fill:var(--bf-amber)"/><rect x="386.8" y="241.0" width="19.6" height="0.3" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="420.1" y="241.2" width="19.6" height="34.8" style="fill:var(--bf-amber)"/><rect x="420.1" y="241.0" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="453.4" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="453.4" y="240.9" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="486.6" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="519.9" y="240.8" width="19.6" height="35.2" style="fill:var(--bf-amber)"/><rect x="553.2" y="234.0" width="19.6" height="42.0" style="fill:var(--bf-amber)"/><rect x="586.4" y="234.7" width="19.6" height="41.3" style="fill:var(--bf-amber)"/><rect x="619.7" y="234.3" width="19.6" height="41.7" style="fill:var(--bf-amber)"/><rect x="652.9" y="234.6" width="19.6" height="41.4" style="fill:var(--bf-amber)"/><rect x="686.2" y="234.8" width="19.6" height="41.2" style="fill:var(--bf-amber)"/><text class="bf-tm" x="346.7" y="162.6" style="font-size:12px">from 8 workers the scanner almost never waits:</text><text class="bf-tm" x="346.7" y="178.6" style="font-size:12px">it is the bottleneck, and only less work there helps</text><rect x="68" y="330" width="12" height="12" style="fill:var(--bf-amber)"/><text class="bf-tm" x="86" y="341" style="font-size:12px">scanner working</text><rect x="234" y="330" width="12" height="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><text class="bf-tm" x="252" y="341" style="font-size:12px">scanner waiting for a free slot</text></svg>
<figcaption>Measured, same run. Below the knee the scanner waits for the workers to free a slot;
from 8 workers on it works without pause, so it is the bottleneck and only a lighter scanner moves
the wall.</figcaption>
</figure>

Three qualifications keep the law honest. First, a station is a resource, not a function: the
scanner's header parsing, table building and Huffman decoding share one thread, so their demands
add, and drawing them as separate parallel stations would invent capacity. Second, the law bounds
throughput; it does not give a completion time. One block that takes a 1 s scan, a 1 s decode and a
1 s delivery in series has a largest demand of 1 s and takes 3 s, so the bottleneck maximum is a
lower bound on the wall, not a forecast of it ([journal §70](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). A global fit of
c + max(b, a/k) to all twenty counts is accordingly less exact than the four-count prediction: its
crossover is 7.17, its residuals run about −13 ms from 8 to 14 workers and +21 ms from 17, and its
32 ms constant has no identified cause. Third, the textbook's knee for closed systems,
N\* = (D + Z)/D_max, is a knee in the population of jobs, not in the number of servers; the decoder
couples the two through W = 4k, but they are different variables ([the appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md#3-the-operational-laws)).

## 6. Little's law, and what the instruments must record

Little's law, L = X·R, says the mean number of jobs in a system is its throughput times the mean
time each spends there ([Little, 1961](https://doi.org/10.1287/opre.9.3.383); Jain, ch. 30;
Harchol-Balter, ch. 6). For a finite trace it is a statement about counting the same thing twice.

<figure class="bf-fig" id="little">
<p class="bf-title">Little's law as two counts of one area</p>
<svg viewBox="0 0 720 358" role="img" aria-label="Little's law as two ways of counting the same quantity. Above, the number of jobs present over time, with the area under it shaded; below, each of five jobs' residence as a bar. The area equals the sum of the bars, so the mean number present equals throughput times mean residence. The jobs are illustrative."><polygon points="60.0,150.0 60.0,150.0 91.5,150.0 91.5,122.5 123.0,122.5 123.0,95.0 217.5,95.0 217.5,67.5 280.5,67.5 280.5,95.0 312.0,95.0 312.0,67.5 375.0,67.5 375.0,95.0 438.0,95.0 438.0,122.5 469.5,122.5 469.5,95.0 595.5,95.0 595.5,122.5 627.0,122.5 627.0,150.0 690.0,150.0 690.0,150.0" style="fill:var(--bf-blue);opacity:0.3"/><polyline points="60.0,150.0 91.5,150.0 91.5,122.5 123.0,122.5 123.0,95.0 217.5,95.0 217.5,67.5 280.5,67.5 280.5,95.0 312.0,95.0 312.0,67.5 375.0,67.5 375.0,95.0 438.0,95.0 438.0,122.5 469.5,122.5 469.5,95.0 595.5,95.0 595.5,122.5 627.0,122.5 627.0,150.0 690.0,150.0" style="fill:none;stroke:var(--bf-blue);stroke-width:2"/><line class="bf-rule" x1="60" x2="690" y1="150" y2="150"/><text class="bf-tm" x="52" y="154.0" text-anchor="end" style="font-size:11px">0</text><text class="bf-tm" x="52" y="126.5" text-anchor="end" style="font-size:11px">1</text><text class="bf-tm" x="52" y="99.0" text-anchor="end" style="font-size:11px">2</text><text class="bf-tm" x="52" y="71.5" text-anchor="end" style="font-size:11px">3</text><text class="bf-tm" x="52" y="44.0" text-anchor="end" style="font-size:11px">4</text><text class="bf-t-strong" x="20" y="24" text-anchor="start" style="font-size:13px">Count the jobs present at every instant, and add up the area under the curve ...</text><text class="bf-t-strong" x="20" y="186" text-anchor="start" style="font-size:13px">... or add up how long each job stayed. For a trace that starts and ends empty, the two totals are equal.</text><rect x="91.5" y="200" width="189.0" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/><text class="bf-tm" x="85.5" y="211" text-anchor="end" style="font-size:11px">job 1</text><rect x="123.0" y="220" width="252.0" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/><text class="bf-tm" x="117.0" y="231" text-anchor="end" style="font-size:11px">job 2</text><rect x="217.5" y="240" width="220.5" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/><text class="bf-tm" x="211.5" y="251" text-anchor="end" style="font-size:11px">job 3</text><rect x="312.0" y="260" width="283.5" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/><text class="bf-tm" x="306.0" y="271" text-anchor="end" style="font-size:11px">job 4</text><rect x="469.5" y="280" width="157.5" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/><text class="bf-tm" x="463.5" y="291" text-anchor="end" style="font-size:11px">job 5</text><line class="bf-rule" x1="60" x2="690" y1="304" y2="304"/><text class="bf-tm" x="375.0" y="322" text-anchor="middle" style="font-size:12px">time</text><text class="bf-tm" x="360.0" y="346" text-anchor="middle" style="font-size:12.5px">Divide both by the length of the trace: mean number present L = throughput X × mean residence R.</text></svg>
<figcaption>Schematic, with five illustrative jobs. The finite-trace form is the one stage 1 checks: each buffer's occupancy integral against the sum of its blocks' residences.</figcaption>
</figure>

That form forces precise questions on the instruments. Which population is meant: blocks waiting for
a worker, all blocks issued and not yet taken, or finished blocks waiting to be read? Each has its
own residence time, and mixing one population with another's interval gives a plausible number that
describes no queue. An early sizing of the admission window made exactly that mistake: throughput
times the sum of the service demands suggested 8 or 9 slots, but the law needs residence time,
waiting included, and gives a mean occupancy rather than a safe maximum, so the figure was withdrawn
([journal §61](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the correction](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md)).

The law also shaped the tracing that stage 1 of the characterisation used ([#841](https://github.com/lmmx/mibz2/pull/841)).
It keeps a live occupancy integral for each of seven buffers, from admitted to consuming, rebuilds
each block's residence from per-thread ledgers, and checks that the two agree exactly, as integers.
A design review had warned beforehand that the decoder marks some transitions after releasing a lock
and others under it, so sorting events by timestamp could produce an impossible queue history; every
transition therefore takes a sequence number under one lock, with one clock read for both
([the review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-design-review.md); [journal §§59–60](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). Across 840 decodes, all
nine identities built from these laws held in every one, Little's law exactly ([journal §66](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).
A trace that broke one would have meant a missing or misplaced event, not a surprising decoder. The
identities establish consistency, not cost: the ledgers themselves perturbed the timing at 10 workers
and above (1.040 at 10, an interval across 1 but wide), so every later comparison judged walls with
them off.

## 7. Modification analysis: asking what if before building

Modification analysis changes one station's demand in the model, or the number of its servers,
and recomputes which constraint binds (Harchol-Balter, ch. 7, "Modification Analysis: What-if for
Closed Systems"). Jain's operational-laws deck sets the same exercise as homework: make one
component twice as fast or twice as slow, and find which becomes the bottleneck. Its lesson is that a
faster component need not make a faster system, and that the gain from improving one station depends
on the others.

<figure class="bf-fig" id="migration">
<p class="bf-title">Why improvements must be priced together</p>
<svg viewBox="0 0 720 272" role="img" aria-label="Illustrative bottleneck migration. Baseline: scanner 100, workers 95, wall about 100. Cut the scanner 20%: 80 and 95, wall 95, so only 5 of the 20 saved reach the wall. Cut the workers 20%: 100 and 76, wall still 100. Cut both: 80 and 76, wall 80."><text class="bf-t-strong" x="20" y="24" text-anchor="start" style="font-size:13px">Work a block needs from each station (illustrative numbers, not measurements)</text><text class="bf-t-strong" x="138" y="62" text-anchor="end" style="font-size:13px">baseline</text><rect x="150" y="44" width="460.0" height="14" style="fill:var(--bf-amber);opacity:0.75"/><rect x="150" y="62" width="437.0" height="14" style="fill:var(--bf-blue);opacity:0.75"/><line x1="610.0" x2="610.0" y1="40" y2="80" style="stroke:var(--bf-ink);stroke-width:2"/><text class="bf-t-strong" x="618.0" y="66" text-anchor="start" style="font-size:13px">wall ≈ 100</text><text class="bf-t-strong" x="138" y="112" text-anchor="end" style="font-size:13px">scanner −20%</text><rect x="150" y="94" width="368.0" height="14" style="fill:var(--bf-amber);opacity:0.75"/><rect x="150" y="112" width="437.0" height="14" style="fill:var(--bf-blue);opacity:0.75"/><line x1="587.0" x2="587.0" y1="90" y2="130" style="stroke:var(--bf-ink);stroke-width:2"/><text class="bf-t-strong" x="595.0" y="116" text-anchor="start" style="font-size:13px">wall ≈ 95</text><text class="bf-t-strong" x="138" y="162" text-anchor="end" style="font-size:13px">workers −20%</text><rect x="150" y="144" width="460.0" height="14" style="fill:var(--bf-amber);opacity:0.75"/><rect x="150" y="162" width="349.6" height="14" style="fill:var(--bf-blue);opacity:0.75"/><line x1="610.0" x2="610.0" y1="140" y2="180" style="stroke:var(--bf-ink);stroke-width:2"/><text class="bf-t-strong" x="618.0" y="166" text-anchor="start" style="font-size:13px">wall ≈ 100</text><text class="bf-t-strong" x="138" y="212" text-anchor="end" style="font-size:13px">both −20%</text><rect x="150" y="194" width="368.0" height="14" style="fill:var(--bf-amber);opacity:0.75"/><rect x="150" y="212" width="349.6" height="14" style="fill:var(--bf-blue);opacity:0.75"/><line x1="518.0" x2="518.0" y1="190" y2="230" style="stroke:var(--bf-ink);stroke-width:2"/><text class="bf-t-strong" x="526.0" y="216" text-anchor="start" style="font-size:13px">wall ≈ 80</text><rect x="150" y="250" width="14" height="10" style="fill:var(--bf-amber);opacity:0.75"/><text class="bf-tm" x="170" y="260" text-anchor="start" style="font-size:12px">scanner</text><rect x="250" y="250" width="14" height="10" style="fill:var(--bf-blue);opacity:0.75"/><text class="bf-tm" x="270" y="260" text-anchor="start" style="font-size:12px">workers</text><text class="bf-tm" x="350" y="260" text-anchor="start" style="font-size:12px">the wall follows the longer bar, when the stages overlap well</text></svg>
<figcaption>Illustrative numbers, chosen to show the arithmetic, under the approximation that the wall follows the larger stage when stages overlap well.</figcaption>
</figure>

The four-arm test is that table measured. With the scanner binding, emission's cut cannot reach the
wall; with the scanner relieved, it can. The converse also holds: a cut in the scanner's work moves
the bottleneck back to the workers, and at 8 workers a 5% cut in the Huffman decode's demand already
reaches the workers' limit ([journal §61](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). So improvements are priced in combination, with the
worker count retuned after each, and the research target becomes a surface over combinations rather
than a list of functions ranked by their share of the profile. A worker-side improvement that is
masked today is not worthless: it is a conditional opportunity, and it may also let the decoder reach
the same wall with fewer cores. The 2025 course gave a session to the same analysis applied to a real
caching system.

## 8. Load dependence: the month's findings, named

The first approximation treats s and w as constants. Real cores share caches, memory bandwidth,
execution units and clock frequency, so a service centre's rate can depend on how many jobs are
present; Jain's chapter on queueing networks names load-dependent centres among its types. Writing
the bound as X(k) ≤ min(1/s(k), k/w(k)) is a reminder that adding a worker raises the pool's nominal
capacity and can slow the station that already limits the output.

The measurements show it at every scale. The scanner's work per decode stays near 0.186 s up to 8
workers and rises 23%, to 0.229 s, at 16, and the wall steps up 20% with it ([journal §57](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).
Stage 1 measured the same inflation per visit: the Huffman decode costs 84 µs a block at one worker
and 105–107 µs at 16 and above, and emission 526 µs at one worker and 816 µs at 14. With the scanner
pinned to its own core, putting the workers on the two hardware threads of shared cores rather than
on separate cores costs 22% at 4 workers and 48% at 8, a stream of memory traffic running alongside
slows the wall by 33% at 8 workers, and a pointer chase by 113%; giving the scanner a quiet core
alone takes the wall to 0.960 of the unpinned one ([journal §78](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). These establish the effects
of particular placements; they do not decompose every earlier residual.

This is also where the earlier placement work belongs. A design study on 5 November priced four
ways to keep the memory-heavy inverse walk from colliding on one core: thread pairs specialised by
stage, a token allowing one walk per core at a time, interleaving other work into the walk's stalls,
and choosing which ready blocks run together. The first three were modelled, the token in a small
discrete-event simulation, and nothing was timed ([the study](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-05-corun-placement.md)). Its sharpest finding was a
shared assumption: every design treated the walk's own footprint as fixed. A kernel that crowds the
cache less changes the demand itself rather than arranging its competitors, which is the route the
lazy kernel later took.

<p class="il-rubric-head">Of noise that came from the clock</p>

Even the measuring was load-dependent. After a short idle pause the processor drops to a low clock
and climbs back slowly: after 0.2 s idle a decode runs at 2.96 GHz and takes 45% longer, and after
the standard 1 s pause, 40% of decodes still run low. Spinning every CPU for 100 ms before each timed
decode removes the low state and cuts the spread of the log wall from 4.4% to 1.1%
([journal §81](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [#874](https://github.com/lmmx/mibz2/pull/874)). That was a repair to the
apparatus, not a faster decoder. It leaves a question for users, since a program that decodes after
a short pause may meet the slow regime, and whether a decoder should respond is open until more
machines are measured.

## 9. What scheduling can and cannot do

With demands measured, the next question was whether a different order of work, a different
queueing discipline, could close the gap to the bound. The operational laws answer part of it before
any code is written: no discipline raises the ceiling the demands set, and the decoder already runs
at 0.945 to 0.998 of the busiest station's limit at every count, so reordering alone could recover
at most about 5%, or 3% on the plateau ([journal §57](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the discipline lead](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md)). That holds
only while the demands stay fixed. A policy that improves cache locality or keeps memory-heavy phases
apart changes the demands themselves, and the clearest example is the twenty-worker decode, which ran
at 0.972 of its bound and still finished 17% sooner with eight workers ([the outside reading](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md)).

The candidates were size-based scheduling, helpers for the oldest unfinished block, admission by work
rather than by count, relaxed resequencing and redundancy; each, with where it comes from and what
became of it, is in [the appendix's table](2026-11-25-the-decoder-as-a-queueing-network-notes.md#7-the-disciplines-considered). None was adopted, and
only one, a parallel scanner, would raise the ceiling rather than approach it.

Variability is the other reason to keep distributions rather than means. In a single queue with
general service times the mean wait grows with the second moment of service time, not only its mean
(Harchol-Balter, ch. 23; [the appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md#6-variability-and-open-against-closed)), so long blocks delay everything
behind them. The corpus has two classes of
block: 792 of its 1,724 are under 10 kbit and hold 0.1% of the bits, and they are exactly the blocks
that skip the main inversion route. Across the mixture the squared coefficient of variation of size
is about 0.98; among the real blocks alone it is about 0.07, and their sizes are correlated along the
stream (lag-1 r = 0.74) ([journal §§60–61](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the opportunities](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md)). Which of those numbers
matters depends on the question. A paper on the 2025 reading list warns that scheduling policies
behave very differently in closed systems than in open ones
([Schroeder, Wierman and Harchol-Balter, NSDI 2006](https://www.usenix.org/conference/nsdi-06/open-versus-closed-cautionary-tale)),
which argues for modelling the admission feedback faithfully, not for declaring scheduling
unimportant.

The table's last row is a different kind of change. A discipline moves the decoder closer to a fixed
ceiling; a change of topology moves the ceiling.

## 10. Product form, mean-value analysis and a surrogate

The bottleneck law gives the two asymptotes; near the knee, finite populations and variability round
the corner, and a natural next step is a model that computes the curve. The theory has one: product-form networks, from Jackson's open networks through Gordon and Newell's
closed ones to the BCMP theorem, which make a network's equilibrium solvable, and Reiser and
Lavenberg's mean-value analysis, which computes a closed network's mean queue lengths and throughput
by adding one job at a time ([the appendix, with the papers](2026-11-25-the-decoder-as-a-queueing-network-notes.md#8-product-form-and-mean-value-analysis)).

It was tried. One reader computed exact mean-value analysis for a two-station closed surrogate, with
a population of N = 4k, a scanner taking 107.9 µs a block and a pool of workers taking 815 µs each,
whose rate grows with the blocks present up to k, and no fitted constant ([the review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md)):

| Workers | Surrogate | Measured wall |
|---|---|---|
| 3 | 0.4683 s | 0.4693 s |
| 5 | 0.2811 s | 0.2840 s |
| 7 | 0.2035 s | 0.2069 s |
| 8 | 0.1887 s | 0.1911 s |

That is within 1 to 4 ms at every count from 3 to 8, which is better than the hard-max fit and shows
the rounded knee is largely what a closed network of this shape produces. Its limits are as clear.
It converts a stationary throughput into a file's completion time, so start-up, drain, in-order taking
and the byte cap are outside it, and its residual cannot all be called contention. The design reviews
listed where the decoder departs from what product form needs: a ceiling rather than a population, a
finite stream, in-order taking, contention between workers, a byte cap that gates scanning, and two
classes of block ([the appendix's table](2026-11-25-the-decoder-as-a-queueing-network-notes.md#8-product-form-and-mean-value-analysis)).

BCMP itself is broader than its reputation, admitting general service distributions under some
disciplines and some load dependence, so none of these is a blanket disqualification; what was never
established is a compatible representation of this decoder's particular gates, ordering and
contention. Exact mean-value analysis is therefore applied to a documented surrogate, with its error
reported as approximation error, and the faithful model is a discrete-event simulation driven by
every block's measured trace ([journal §§59–60](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). That simulation has been specified, not
built.

<figure class="bf-fig" id="models">
<p class="bf-title">From laws to simulation: what each kind of model assumes</p>
<svg viewBox="0 0 720 278" role="img" aria-label="A ladder of four kinds of model. Operational laws assume nothing about distributions and held in all 840 decodes. Product-form networks and mean-value analysis give exact means for a surrogate network, which came within 1 to 4 ms of the measured wall. A trace-driven simulation would keep the decoder's own rules; it is proposed, not built. Interventions test the predictions."><text class="bf-t-strong" x="20" y="22" text-anchor="start" style="font-size:13px">Each rung assumes more, or tests more; the measurements decide between them</text><rect x="20" y="34" width="680" height="50" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="36" y="54" text-anchor="start" style="font-size:14px">operational laws</text><text class="bf-tm" x="36" y="72" text-anchor="start" style="font-size:12px">assume nothing about distributions: bounds, and identities a trace must satisfy</text><text class="bf-t-strong" x="684" y="54" text-anchor="end" style="font-size:12px;fill:var(--bf-green)">held: nine identities in 840 decodes</text><rect x="20" y="96" width="680" height="50" rx="6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text class="bf-t-strong" x="36" y="116" text-anchor="start" style="font-size:14px">product form and MVA</text><text class="bf-tm" x="36" y="134" text-anchor="start" style="font-size:12px">assume a network the decoder only resembles: exact means for that network</text><text class="bf-t-strong" x="684" y="116" text-anchor="end" style="font-size:12px;fill:var(--bf-steel)">a surrogate within 1–4 ms of the wall at 3–8 workers</text><rect x="20" y="158" width="680" height="50" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="36" y="178" text-anchor="start" style="font-size:14px">trace-driven simulation</text><text class="bf-tm" x="36" y="196" text-anchor="start" style="font-size:12px">keeps the decoder's own rules: the window, the byte cap, in-order taking</text><text class="bf-t-strong" x="684" y="178" text-anchor="end" style="font-size:12px;fill:var(--bf-amber)">proposed, not built</text><rect x="20" y="220" width="680" height="50" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="36" y="240" text-anchor="start" style="font-size:14px">intervention</text><text class="bf-tm" x="36" y="258" text-anchor="start" style="font-size:12px">slow, pin or supply one part, and see whether the prediction moves with it</text><text class="bf-t-strong" x="684" y="240" text-anchor="end" style="font-size:12px;fill:var(--bf-blue)">stage 2 and the ideal-parts matrix</text></svg>
<figcaption>The status column is the record's: journal §66 for the identities, the course-mining review for the surrogate, journal §§59–60 for the simulation, §§74–88 for the interventions.</figcaption>
</figure>

## 11. Testing the explanation by intervention

Accounting says which resources look busy; an intervention asks whether changing one produces the
predicted effect. Stage 1 measured the decoder at every count from 1 to 20, with every block's route,
stage boundaries and queue transitions logged ([journal §66](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt)). The limit
moves from the workers to the scanner at exactly 8 workers, with a bootstrap interval of [8, 8]; the
workers' idle time while the scanner works rises from 0.035 of their time at 8 to 0.551 at 20, while
their idle time waiting on input stays under 0.01; and the reader is no co-bottleneck, at 0.017–0.035
ms a block against the scanner's 0.099–0.130.

Stage 2 slowed each activity in turn with a calibrated loop of dependent arithmetic, which adds
occupancy without adding memory traffic, so its elasticities describe the response to that
intervention rather than an exact scaling of the original code
([the decisions](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/briefs/character-review-decisions.md)). Below the knee the workers'
construction and emission controlled the wall; at 8 and 10 only the scanner's Huffman decode did,
and with the scanner pinned its intervals narrowed two to four times ([journal §§74, 78](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

<figure class="bf-fig" id="screen">
<p class="bf-title">Which activity controls the wall, at 6, 8 and 10 workers</p>
<svg viewBox="0 0 520 472" role="img" aria-label="Stage 2's screen at 6, 8 and 10 workers. At 6, construction (0.14, 0.15) and emission (0.37) control the wall and the scanner's Huffman decode does not; at 8 and 10 only the scanner's Huffman decode controls it (0.79, 0.58)."><text class="bf-t-strong" x="236" y="20" style="font-size:15px">6 workers</text><line class="bf-rule" x1="236.0" x2="236.0" y1="30" y2="126" opacity="0.35"/><line class="bf-rule" x1="357.8" x2="357.8" y1="30" y2="126" opacity="0.9"/><line class="bf-rule" x1="479.6" x2="479.6" y1="30" y2="126" opacity="0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="47.0" text-anchor="end">scanner: Huffman decode</text><line x1="353.1" x2="359.7" y1="42.0" y2="42.0" style="stroke:var(--bf-amber);stroke-width:2;opacity:0.35"/><circle cx="356.4" cy="42.0" r="4" style="fill:var(--bf-amber);opacity:0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="71.0" text-anchor="end">workers: construction, first pass</text><line x1="371.2" x2="378.9" y1="66.0" y2="66.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:1"/><circle cx="375.0" cy="66.0" r="5" style="fill:var(--bf-blue);opacity:1"/><text class="bf-tm" style="font-size:14px" x="224" y="95.0" text-anchor="end">workers: construction, setup</text><line x1="370.5" x2="382.3" y1="90.0" y2="90.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:1"/><circle cx="376.4" cy="90.0" r="5" style="fill:var(--bf-blue);opacity:1"/><text class="bf-tm" style="font-size:14px" x="224" y="119.0" text-anchor="end">workers: emission</text><line x1="396.0" x2="408.8" y1="114.0" y2="114.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:1"/><circle cx="402.4" cy="114.0" r="5" style="fill:var(--bf-blue);opacity:1"/><text class="bf-t-strong" x="236" y="164" style="font-size:15px">8 workers (scanner pinned)</text><line class="bf-rule" x1="236.0" x2="236.0" y1="174" y2="270" opacity="0.35"/><line class="bf-rule" x1="357.8" x2="357.8" y1="174" y2="270" opacity="0.9"/><line class="bf-rule" x1="479.6" x2="479.6" y1="174" y2="270" opacity="0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="191.0" text-anchor="end">scanner: Huffman decode</text><line x1="440.8" x2="468.5" y1="186.0" y2="186.0" style="stroke:var(--bf-amber);stroke-width:2;opacity:1"/><circle cx="454.6" cy="186.0" r="5" style="fill:var(--bf-amber);opacity:1"/><text class="bf-tm" style="font-size:14px" x="224" y="215.0" text-anchor="end">workers: construction, first pass</text><line x1="344.8" x2="381.0" y1="210.0" y2="210.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="362.9" cy="210.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="239.0" text-anchor="end">workers: construction, setup</text><line x1="338.2" x2="374.7" y1="234.0" y2="234.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="356.4" cy="234.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="263.0" text-anchor="end">workers: emission</text><line x1="365.8" x2="402.1" y1="258.0" y2="258.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="384.0" cy="258.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-t-strong" x="236" y="308" style="font-size:15px">10 workers (scanner pinned)</text><line class="bf-rule" x1="236.0" x2="236.0" y1="318" y2="414" opacity="0.35"/><line class="bf-rule" x1="357.8" x2="357.8" y1="318" y2="414" opacity="0.9"/><line class="bf-rule" x1="479.6" x2="479.6" y1="318" y2="414" opacity="0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="335.0" text-anchor="end">scanner: Huffman decode</text><line x1="409.0" x2="447.2" y1="330.0" y2="330.0" style="stroke:var(--bf-amber);stroke-width:2;opacity:1"/><circle cx="428.1" cy="330.0" r="5" style="fill:var(--bf-amber);opacity:1"/><text class="bf-tm" style="font-size:14px" x="224" y="359.0" text-anchor="end">workers: construction, first pass</text><line x1="324.5" x2="365.2" y1="354.0" y2="354.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="344.8" cy="354.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="383.0" text-anchor="end">workers: construction, setup</text><line x1="301.7" x2="364.1" y1="378.0" y2="378.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="332.9" cy="378.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-tm" style="font-size:14px" x="224" y="407.0" text-anchor="end">workers: emission</text><line x1="307.4" x2="353.7" y1="402.0" y2="402.0" style="stroke:var(--bf-blue);stroke-width:2;opacity:0.35"/><circle cx="330.6" cy="402.0" r="4" style="fill:var(--bf-blue);opacity:0.35"/><text class="bf-tm" style="font-size:13px" x="236.0" y="430" text-anchor="middle">−1</text><text class="bf-tm" style="font-size:13px" x="357.8" y="430" text-anchor="middle">0</text><text class="bf-tm" style="font-size:13px" x="479.6" y="430" text-anchor="middle">1</text><text class="bf-tm" style="font-size:13px" x="504" y="468" text-anchor="end">elasticity of the wall, with 95% intervals</text></svg>
<figcaption>Measured: each activity slowed 20% alone; 6 workers from stage 2a (6 blocks), 8 and 10
from stage 2b with the scanner pinned to its own core (16 blocks). Solid where the report finds an
activity controls the wall, pale where it doesn't resolve or is negligible. The scanner's Huffman
decode is amber, the workers' activities blue. At 10 workers several worker coefficients fall below
0, so slowing them speeds the wall, which the run reads as contention and leaves unexplained in
detail.</figcaption>
</figure>

The sharper test was composition. Composed as a sum of logarithms, the one-at-a-time coefficients
predicted an arm kept out of every fit, the Huffman decode and emission both slowed 10%; the first
run's wide intervals passed it, and the pinned run's refuted it.

| Workers | Observed | Sum of logarithms | Larger coefficient alone |
|---|---|---|---|
| 8 | 1.069 [1.049, 1.090] | 1.101, outside | 1.079, inside |
| 10 | 1.063 [1.031, 1.097] | 1.034, outside | 1.057, inside |

A maximum fits, and the surface at 8 workers, both stations slowed together, has a roundness of
0.500 [0.454, 0.548], the value of a sharp maximum. Because the maximum was chosen after the failure,
it is registered as the predictor for the next held-out test rather than counted as validated. A
second test tried to infer the gain from speeding a part by slowing everything else and rescaling,
and its check that the wall scales evenly failed at all four cells. At a crossover the harm from
slowing a station cannot be inverted into the gain from speeding it: at 8 workers slowing the scanner
costs 0.7 to 1.0 of each unit, but speeding it gains only 0.13 to 0.18, because the workers are close
behind and take over at once.

The pulse experiments gave the dynamics. A fluid approximation treats a queue as a reservoir whose
level changes at the rate work flows in less the rate it flows out, dq/dt = λ_in − μ_out, bounded
below by an empty queue and above by the window; it appears in the theory course's topics, and no
fluid model has been fitted to the decoder.

<figure class="bf-fig" id="reservoir">
<p class="bf-title">Which side of the queue binds</p>
<svg viewBox="0 0 720 344" role="img" aria-label="Two reservoirs, the job queue at 8 workers. Slowing the scanner by half drains it: mean depth 4.4 blocks before, 0.3 during, and the workers starve, the rate falling to 0.69 of the undosed run's. Slowing emission by half fills it: 3.9 before, 8.9 during, the rate 0.82 of the undosed run's."><text class="bf-t-strong" x="180" y="22" text-anchor="middle" style="font-size:15px">Slow the scanner</text><text class="bf-tm" x="180" y="40" text-anchor="middle" style="font-size:12px">slowed: the scanner, +50%</text><line x1="180" x2="180" y1="52" y2="84" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:4 3"/><path d="M174,83 l6,9 l6,-9 z" style="fill:var(--bf-amber)"/><text class="bf-tm" x="194" y="76.0" style="font-size:12px">in, from the scanner (slowed)</text><line x1="180" x2="180" y1="236" y2="268" style="stroke:var(--bf-frame);stroke-width:3"/><path d="M174,267 l6,9 l6,-9 z" style="fill:var(--bf-frame)"/><text class="bf-tm" x="194" y="260.0" style="font-size:12px">out, to the workers</text><rect x="102" y="227.9" width="156" height="4.1" style="fill:var(--bf-blue);opacity:0.55"/><path d="M100,96 L100,232 L260,232 L260,96" style="fill:none;stroke:var(--bf-ink);stroke-width:2"/><line x1="100" x2="260" y1="172.2" y2="172.2" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-tm" x="94" y="176.2" text-anchor="end" style="font-size:11.5px">before</text><text class="bf-t-strong" x="180" y="298" text-anchor="middle" style="font-size:16px">4.4 → 0.3 blocks</text><text class="bf-tm" x="180" y="318" text-anchor="middle" style="font-size:12px">mean queue depth: the workers are starved</text><text class="bf-tm" x="180" y="334" text-anchor="middle" style="font-size:12px">rate 0.69 of the undosed run’s</text><text class="bf-t-strong" x="540" y="22" text-anchor="middle" style="font-size:15px">Slow emission</text><text class="bf-tm" x="540" y="40" text-anchor="middle" style="font-size:12px">slowed: the workers' emission, +50%</text><line x1="540" x2="540" y1="52" y2="84" style="stroke:var(--bf-frame);stroke-width:3"/><path d="M534,83 l6,9 l6,-9 z" style="fill:var(--bf-frame)"/><text class="bf-tm" x="554" y="76.0" style="font-size:12px">in, from the scanner</text><line x1="540" x2="540" y1="236" y2="268" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:4 3"/><path d="M534,267 l6,9 l6,-9 z" style="fill:var(--bf-amber)"/><text class="bf-tm" x="554" y="260.0" style="font-size:12px">out, to the workers (slowed)</text><rect x="462" y="111.0" width="156" height="121.0" style="fill:var(--bf-blue);opacity:0.55"/><path d="M460,96 L460,232 L620,232 L620,96" style="fill:none;stroke:var(--bf-ink);stroke-width:2"/><line x1="460" x2="620" y1="179.0" y2="179.0" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-tm" x="454" y="183.0" text-anchor="end" style="font-size:11.5px">before</text><text class="bf-t-strong" x="540" y="298" text-anchor="middle" style="font-size:16px">3.9 → 8.9 blocks</text><text class="bf-tm" x="540" y="318" text-anchor="middle" style="font-size:12px">mean queue depth: work piles up</text><text class="bf-tm" x="540" y="334" text-anchor="middle" style="font-size:12px">rate 0.82 of the undosed run’s</text></svg>
<figcaption>Measured, stage 2a at 8 workers: each pulse slows one activity by 50% over
blocks 600 to 800. The tanks show the job queue's mean depth before (dashed) and during (filled); the rate is the
rate during the pulse over the undosed run's at the same places. The undosed run's own depth over
those places rises from 4.5 to 5.6, so the scanner's drain is the larger departure. Both recover
within 2 to 4 ms.</figcaption>
</figure>

## 12. What reached the code

The analysis is only as good as what it changes, and the record keeps the outcomes distinct.

| What the model said | What was done | Outcome |
|---|---|---|
| The threads requested, the workers built and the kernel chosen are different variables | at 5 and 7 requested threads, build the lazy kernel the larger counts already used | 0.840 and 0.631 of the wall, with 58% and 63% less memory; merged ([#868](https://github.com/lmmx/mibz2/pull/868)) |
| The knee is near 7.5, and the cap of 7 workers sits below it | 8 workers from 10 requested threads, measured with the clock warmed | 0.929 and 0.932 at 10 and 12 threads, for 2–3% more CPU; merged ([#878](https://github.com/lmmx/mibz2/pull/878)) |
| About 15 ms of the scanner's work, 8% of its demand, is two avoidable copies | stop re-zeroing symbol buffers, and copy less at the seam between input chunks | not adopted at the judged count, faster by about 2% at 10 and 12; open ([#846](https://github.com/lmmx/mibz2/pull/846), [#857](https://github.com/lmmx/mibz2/pull/857), [#861](https://github.com/lmmx/mibz2/pull/861)) |
| The scan is one dependent chain, so fewer lookups a symbol should help | three symbols a lookup instead of two | lookups fell 12% and time did not; rejected ([#870](https://github.com/lmmx/mibz2/pull/870)) |
| A smaller saving inside the same lookup loop | count each batch's bits at once | about 2% of scanner time locally, unresolved on the whole decode; held ([#879](https://github.com/lmmx/mibz2/pull/879)) |

Two details are instructive. The first readings assumed the Python binding ran 20 workers on this
machine; two readers found it built 7, so the live question became 7 against 8, and that is the
change that merged ([journal §61](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). And the multi-symbol idea was priced against a baseline
that no longer existed: the decoder's table had held up to two symbols an entry since a change merged
on 22 November ([#745](https://github.com/lmmx/mibz2/pull/745)), taking 1.85 symbols a lookup on
wikidata, so the third symbol was all that was left to test, and fewer operations of one kind did
not lower the total demand ([journal §82](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 13. Towards an ideal form

The overnight work found useful engineering, but the programme's point was to characterise the
decoder's shape with each part treated as a toy problem with ideal behaviour, and to hand that shape
to the search for new mathematics ([journal §87](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the matrix](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/README.md)). Three questions
need keeping apart. A profile asks where time is spent; a slowing intervention asks which changes
hurt completion now; an ideal-part experiment asks how much completion could improve if a computation
became cheap, which is what keeps a masked idea visible as a target.

The ideal-parts matrix makes six parts ideal in combination: the Huffman scan (S), emission (E),
construction of the inversion's structures (C), the move-to-front and run-length stage (M), the final
expansion, checksum and output work (R), and delivery to the caller (D). Its first slice, the scanner
and emission at 1, 6, 8 and 10 workers, has run ([slice A](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/slice-a/README.md)).

<figure class="bf-fig" id="slicea">
<p class="bf-title">The ideal-parts matrix, slice A: the scanner and emission</p>
<svg viewBox="0 0 720 304" role="img" aria-label="Ideal-parts slice A. With emission ideal the wall falls to 0.827 and 0.849 at 1 and 6 workers, where the scanner ideal does nothing; at 8 each alone is about 0.96 but both together 0.827; at 10 the scanner ideal gives 0.919 and emission 0.975. With both ideal the wall is still 0.82 to 0.84 at every count."><line class="bf-rule" x1="70" x2="700" y1="211.1" y2="211.1" opacity="0.35"/><text class="bf-tm" x="62" y="215.1111111111111" text-anchor="end" style="font-size:11.5px">0.80</text><line class="bf-rule" x1="70" x2="700" y1="172.2" y2="172.2" opacity="0.35"/><text class="bf-tm" x="62" y="176.22222222222226" text-anchor="end" style="font-size:11.5px">0.85</text><line class="bf-rule" x1="70" x2="700" y1="133.3" y2="133.3" opacity="0.35"/><text class="bf-tm" x="62" y="137.33333333333331" text-anchor="end" style="font-size:11.5px">0.90</text><line class="bf-rule" x1="70" x2="700" y1="94.4" y2="94.4" opacity="0.35"/><text class="bf-tm" x="62" y="98.44444444444449" text-anchor="end" style="font-size:11.5px">0.95</text><line class="bf-rule" x1="70" x2="700" y1="55.6" y2="55.6" opacity="0.9"/><text class="bf-tm" x="62" y="59.55555555555557" text-anchor="end" style="font-size:11.5px">1.00</text><rect x="92.0" y="55.6" width="28" height="-0.8" style="fill:var(--bf-amber);opacity:0.8"/><text class="bf-tm" x="106.0" y="68.77777777777789" text-anchor="middle" style="font-size:10.5px">1.001</text><rect x="128.0" y="55.6" width="28" height="134.6" style="fill:var(--bf-blue);opacity:0.8"/><text class="bf-tm" x="142.0" y="204.11111111111117" text-anchor="middle" style="font-size:10.5px">.827</text><rect x="164.0" y="55.6" width="28" height="140.0" style="fill:var(--bf-green);opacity:0.8"/><text class="bf-tm" x="178.0" y="209.5555555555556" text-anchor="middle" style="font-size:10.5px">.820</text><text class="bf-t-strong" x="142.0" y="268" text-anchor="middle" style="font-size:12.5px">1 worker</text><rect x="249.5" y="55.6" width="28" height="13.2" style="fill:var(--bf-amber);opacity:0.8"/><text class="bf-tm" x="263.5" y="82.7777777777778" text-anchor="middle" style="font-size:10.5px">.983</text><rect x="285.5" y="55.6" width="28" height="117.4" style="fill:var(--bf-blue);opacity:0.8"/><text class="bf-tm" x="299.5" y="187.00000000000003" text-anchor="middle" style="font-size:10.5px">.849</text><rect x="321.5" y="55.6" width="28" height="142.3" style="fill:var(--bf-green);opacity:0.8"/><text class="bf-tm" x="335.5" y="211.88888888888894" text-anchor="middle" style="font-size:10.5px">.817</text><text class="bf-t-strong" x="299.5" y="268" text-anchor="middle" style="font-size:12.5px">6 workers</text><rect x="407.0" y="55.6" width="28" height="28.0" style="fill:var(--bf-amber);opacity:0.8"/><text class="bf-tm" x="421.0" y="97.5555555555556" text-anchor="middle" style="font-size:10.5px">.964</text><rect x="443.0" y="55.6" width="28" height="31.1" style="fill:var(--bf-blue);opacity:0.8"/><text class="bf-tm" x="457.0" y="100.66666666666671" text-anchor="middle" style="font-size:10.5px">.960</text><rect x="479.0" y="55.6" width="28" height="134.6" style="fill:var(--bf-green);opacity:0.8"/><text class="bf-tm" x="493.0" y="204.11111111111117" text-anchor="middle" style="font-size:10.5px">.827</text><text class="bf-t-strong" x="457.0" y="268" text-anchor="middle" style="font-size:12.5px">8 workers</text><rect x="564.5" y="55.6" width="28" height="63.0" style="fill:var(--bf-amber);opacity:0.8"/><text class="bf-tm" x="578.5" y="132.55555555555554" text-anchor="middle" style="font-size:10.5px">.919</text><rect x="600.5" y="55.6" width="28" height="19.4" style="fill:var(--bf-blue);opacity:0.8"/><text class="bf-tm" x="614.5" y="89.00000000000003" text-anchor="middle" style="font-size:10.5px">.975</text><rect x="636.5" y="55.6" width="28" height="126.0" style="fill:var(--bf-green);opacity:0.8"/><text class="bf-tm" x="650.5" y="195.5555555555556" text-anchor="middle" style="font-size:10.5px">.838</text><text class="bf-t-strong" x="614.5" y="268" text-anchor="middle" style="font-size:12.5px">10 workers</text><text class="bf-t-strong" x="20" y="22" text-anchor="start" style="font-size:13px">Wall with a part made ideal, over everything real (pinned scanner; bars hang down from 1)</text><rect x="80" y="284" width="14" height="10" style="fill:var(--bf-amber);opacity:0.8"/><text class="bf-tm" x="100" y="294" text-anchor="start" style="font-size:12px">the scanner ideal</text><rect x="270" y="284" width="14" height="10" style="fill:var(--bf-blue);opacity:0.8"/><text class="bf-tm" x="290" y="294" text-anchor="start" style="font-size:12px">emission ideal</text><rect x="460" y="284" width="14" height="10" style="fill:var(--bf-green);opacity:0.8"/><text class="bf-tm" x="480" y="294" text-anchor="start" style="font-size:12px">both ideal: the floor</text></svg>
<figcaption>Measured: ideal-parts slice A, scanner pinned, wall over everything real at each count; 95% intervals are in the linked results. At 8 workers each part alone is unresolved but the two together reach 0.827.</figcaption>
</figure>

Below the knee only emission has room, 15–17%. At the knee the two co-limit: each alone is about 0.96,
unresolved, but each saves about 14% once the other is ideal, an interaction of −0.028 [−0.038,
−0.018] on the log scale, the four-arm test's masking measured at the balance point. Above it the
scanner has room (0.919) and emission's appears mostly behind it. And the floor is high: with both
ideal the decoder keeps 0.82–0.84 of its wall at every count, so most of what remains lies in the four
parts this slice left real, together with the handover cost of supplying emission's codes, 286 to
418 ms summed over workers. An ideal part keeps an experimental definition: supplied intermediates
bring their own storage and copying, upstream stages still run when only a later one is ideal, and the
results measure room within an arrangement, not what any real algorithm will reach.

The full design had to be widened before it ran: its registered half fraction of the 64 combinations
lacked the arms with a single part ideal, on which room is defined, so the matrix runs 44 arms at each
count, the 32 of the fraction for the effects and the 12 odd ones for room, read directly
([clarification 8](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md); [the appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md#10-experimental-design-and-the-ideal-parts-matrix)).

"Ideal form" has three meanings, and the programme needs all three: a tractable model of the decoder,
an ideal-part configuration as a counterfactual, and a realisable organisation that meets the output
contract and the machine's limits. The third is where the structure itself comes into question.

<figure class="bf-fig" id="organisations">
<p class="bf-title">Where the serial boundary sits</p>
<svg viewBox="0 0 720 252" role="img" aria-label="Two organisations of the decoder. Today: a serial Huffman scan feeds scanned blocks to parallel inversion, then in-order output. A question: find block starts first, then let workers both scan and invert in parallel, then in-order output."><text class="bf-t-strong" x="20" y="32" text-anchor="start" style="font-size:13px">Today</text><rect x="20" y="40" width="150" height="44" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="95.0" y="67.0" text-anchor="middle" style="font-size:12.5px">serial Huffman scan</text><rect x="196" y="40" width="150" height="44" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="271.0" y="67.0" text-anchor="middle" style="font-size:12.5px">scanned blocks</text><line x1="172" y1="62" x2="193" y2="62" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="193.0,62.0 184.0,57.0 184.0,67.0" style="fill:var(--bf-frame)"/><rect x="372" y="40" width="150" height="44" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="447.0" y="67.0" text-anchor="middle" style="font-size:12.5px">parallel inversion</text><line x1="348" y1="62" x2="369" y2="62" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="369.0,62.0 360.0,57.0 360.0,67.0" style="fill:var(--bf-frame)"/><rect x="548" y="40" width="150" height="44" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="623.0" y="67.0" text-anchor="middle" style="font-size:12.5px">in-order output</text><line x1="524" y1="62" x2="545" y2="62" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="545.0,62.0 536.0,57.0 536.0,67.0" style="fill:var(--bf-frame)"/><text class="bf-t-strong" x="20" y="128" text-anchor="start" style="font-size:13px">A question</text><rect x="20" y="136" width="150" height="44" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="95.0" y="163.0" text-anchor="middle" style="font-size:12.5px">find block starts</text><rect x="196" y="136" width="150" height="44" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="271.0" y="163.0" text-anchor="middle" style="font-size:12.5px">block queue</text><line x1="172" y1="158" x2="193" y2="158" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="193.0,158.0 184.0,153.0 184.0,163.0" style="fill:var(--bf-frame)"/><rect x="372" y="136" width="150" height="44" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="447.0" y="155.5" text-anchor="middle" style="font-size:12.5px">parallel scan</text><text class="bf-t-strong" x="447.0" y="170.5" text-anchor="middle" style="font-size:12.5px">and inversion</text><line x1="348" y1="158" x2="369" y2="158" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="369.0,158.0 360.0,153.0 360.0,163.0" style="fill:var(--bf-frame)"/><rect x="548" y="136" width="150" height="44" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="623.0" y="163.0" text-anchor="middle" style="font-size:12.5px">in-order output</text><line x1="524" y1="158" x2="545" y2="158" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="545.0,158.0 536.0,153.0 536.0,163.0" style="fill:var(--bf-frame)"/><text class="bf-tm" x="360.0" y="222" text-anchor="middle" style="font-size:12.5px">The second row moves the scan behind the parallel boundary. Whether it pays depends on what finding</text><text class="bf-tm" x="360.0" y="240" text-anchor="middle" style="font-size:12.5px">the starts costs, what state must be carried across, and how the new demands compete for the same cores.</text></svg>
<figcaption>Schematic. The second row is a question for the search, with its costs unknown.</figcaption>
</figure>

The fourth reader enumerated the ways the scanner's obligations could be divided between the scanner
and the workers and scored each with the two-station law ([record 130](https://github.com/lmmx/mibz2/blob/9c3690f299574ea5296e189c8f3d74e7dcfb19af/docs/journal/monodromy/2026-11-25-design-space-and-identification.md)). Its principle is to
keep an obligation on a serial path only if its state has no small summary, and to cut the path
wherever the format resets that state. Decoding a single block's codes in parallel is ruled out: the
table is chosen afresh every 50 symbols by a running count that nothing in the stream re-synchronises,
so each chunk would have to carry at least 50 hypotheses, an obstruction proved from the format with
its cost argued. An index of block starts, so that each worker scans its own block, remains the larger
structural lead. The same record measured where the workers' time goes on a real block at one worker:
construction 45%, emission 38%, the move-to-front stage 10% and the final expansion and checksum 6%,
which makes construction, not emission, the largest target for mathematics in the workers.

Two readings give the search its language. Systems biology's question of which structures can perform
a given function at all, set out as a five-step programme by
[Khammash](https://doi.org/10.1016/j.cels.2021.05.020){ .il-ref }, frames the search for a better
decoder as the search for a better network. And the theory of hybrid dynamical systems, the natural
continuation of the reservoir picture, says any reorganisation changes one of five things: how fast
work runs, where work may proceed, when events may happen, what events do, or what is represented
([the appendix](2026-11-25-the-decoder-as-a-queueing-network-notes.md#12-the-design-thread)). Thread pairs, tokens, a parallel scan, a fused stage or a
new representation are all instances; the formalism organises the search, and predictions and
optimality claims still need their own evidence.

## 14. How the work is done

The method is written down so that it can be reused
([the protocol](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/process/protocol.md)). It starts from a measured surprise rather than a technique,
poses the problem and its referee before any solution, and prefers a model that predicts from little
data and is checked on data it did not see. Sources are saved, and several readers and reviewers
answer independently before comparing. Every run registers its decision rule before its data. Wrong
claims are corrected in place and in public, as the worker count, the closed-network description,
the lower bound, the log-additive predictor and the multi-symbol price were here.

## 15. What runs next

The full ideal-parts matrix comes next, with construction, the move-to-front stage, the final work
and delivery each given an oracle, at 1, 6, 8 and 10 workers, and its results become the map of where
mathematics could pay: improvements useful now, improvements hidden behind another part, combinations
that expose room, and changes that save cores or memory without moving the wall. The scanner's copy
fixes are being asked again with the clock warmed.

Seen whole, the work moves through three levels. Algorithm analysis asks how much work a computation
needs. Queueing theory asks which of that work sets the speed of the whole. The structural question,
what organisation of the work could remove it or its dependencies, is where the dynamical systems and
the mathematics come in. Queueing theory is the bridge between the first and the third: it says where
new mathematics would pay.


---

**Sources:**

- The oracle and the knee: journal §§44 and 57, [the scanner profile](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv), and [Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md).
- The reading: [the courses as read](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md); the readers' answers, [the summary](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md), [the opportunities with their corrections](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md) and [the discipline lead](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md); journal §§52–62.
- The characterisation: [stage 1's report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt) and the stage 2 reports, journal §§66, 71, 74 and 78; the clock, §81.
- The code: [#841](https://github.com/lmmx/mibz2/pull/841), [#868](https://github.com/lmmx/mibz2/pull/868), [#870](https://github.com/lmmx/mibz2/pull/870), [#874](https://github.com/lmmx/mibz2/pull/874), [#878](https://github.com/lmmx/mibz2/pull/878), [#879](https://github.com/lmmx/mibz2/pull/879); journal §§76–86.
- The ideal-parts matrix: [its directory](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts), journal §§87–88.
- The design thread: [record 130](https://github.com/lmmx/mibz2/blob/9c3690f299574ea5296e189c8f3d74e7dcfb19af/docs/journal/monodromy/2026-11-25-design-space-and-identification.md), [structure and function](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md), [the hybrid-systems levers](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md).
- [The journal](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md).
