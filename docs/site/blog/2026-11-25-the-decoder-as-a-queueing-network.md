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

25 November 2026 · 26 min read · [Research in progress](#13-what-runs-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Queueing theory describes a program as a network of stations through which jobs pass, and its
simplest laws hold whatever the stations compute. mibz2's decoder is such a network: one serial
scanner feeds parallel workers, and a consumer that writes blocks in order hands admission back to
the scanner. Applied to it, [the bottleneck law](#2-queueing-theory-in-the-measure-needed-here) predicted from four worker counts where adding
workers stops helping, at about 7.5 on our ten-core machine, and the measurement across every count
from 1 to 20 puts the crossover at 7.67. The same laws said what not to try: no reordering of the
queue can beat the busiest station's limit, and the decoder already runs within about 5% of it, so
gains must come from less work at the bottleneck or from a different topology. Modification
analysis, which changes one station in the model before anything is built, explains why speeding
one stage can show nothing until another is relieved, and why improvements have to be priced
together. The exact theory beyond the laws, product-form networks and mean-value analysis, does not
fit a decoder with ordered output and shared caches, so bounds and a simulation driven by the
measured trace do the predicting. Experiments that slow each station in turn then show which one
controls the decoder at each worker count: the workers' construction and emission below the knee,
and the scanner's Huffman decode at and above it. The point of all this is not a better thread
pool; queueing theory locates where new mathematics in the algorithm would pay. This post picks up where
[Posing the inversion problem](2026-11-24-posing-the-inversion-problem.md) left off.

**Takeaways**

- Queueing theory's operational laws use only measured quantities, with no assumptions about
  distributions: here they predicted the knee, audited the instruments, and bounded what any
  scheduling could gain.
- A two-station model fitted on four worker counts predicts the knee at 7.49 workers; the
  measured crossover is 7.67, and the decoder's best wall, 0.191 s, is at 8.
- Above the knee the decoder runs at 0.945 or better of the busiest server's limit at every count,
  so no queueing discipline can recover more than about 5%.
- Independent readings corrected the first: the public API never runs 20 workers on this machine,
  and the scanner spends about 8% of its demand on two avoidable copies.
- Stage 1 of the characterisation confirmed the switch from workers to scanner at exactly 8, and
  found that asking for 3, 5 or 7 threads selects an older kernel that is 61% slower at 7.
- Stage 2 showed the switch by intervention: at 6 workers construction and emission control the
  wall, at 8 and 10 only the scanner's Huffman decode does, and the wall behaves as the maximum of
  the two stations, so a faster scanner gains little at 8 while workers sharing cores cost 48%.
- Two claims were corrected: the price for decoding several symbols a lookup was for a change
  master already had, and a third symbol gained nothing; and the first held-out prediction passed
  only on wide intervals.

## 1. A simple observation

The four-arm test supplied each of the decoder's two stages for free, outside the clock. At ten
workers, supplying the codes for emission saved nothing measurable (0.976 of the shipped time),
supplying the scanner's output saved 19% (0.806), and supplying both saved 26% (0.741): a gain
invisible until another bottleneck was removed
([Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md)). An
observation like that asks for a model of the decoder as a system rather than as a list of kernels,
which is what the programme set out in the previous post is for.

## 2. Queueing theory, in the measure needed here

Queueing theory describes a system as stations, each a server or a group of servers with a queue in
front, and jobs that pass between them. The account here follows two teachers whose courses were
read in full for this work: Raj Jain's [course](https://www.cs.wustl.edu/~jain/iucee/index.html)
on his *The Art of Computer Systems Performance Analysis*, and Mor Harchol-Balter's
[15-857](https://www.cs.cmu.edu/~harchol/Perfclass/class.html) and its theory companion
[15-849b](https://www.cs.cmu.edu/~harchol/Perfclass/classtheory.html) at Carnegie Mellon, built on
her *Performance Modeling and Design of Computer Systems* ([our reading of all three](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md); [journal §53](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)).
A job's service time at a station is how long one visit takes, and its demand there, D = V·S, is
the number of visits it makes times the time each takes. A station's throughput X is the jobs it
completes a second, and its utilisation U is the fraction of time it is busy.

A few shapes recur. A single server has one queue and one processor. A parallel group has one queue
feeding several identical servers, as a thread pool does. A tandem puts stages in series, as a
pipeline does. A fork–join splits a job into parts that run in parallel and must all finish. A
network joins any of these, with routing between them and sometimes feedback. Networks come in two
kinds. In an open network, jobs arrive from outside at some rate and leave when they are done. In a
closed network, a fixed population circulates, and each job that finishes is replaced by another,
as in a batch system that keeps a set number of jobs in flight.

The decoder is a tandem: the scanner feeds a parallel group of k workers, and a consumer writes the
finished blocks in order. A new block is admitted only while fewer than W = 4k are in flight, and a
slot returns only when the consumer takes the next block in order, so admission credit circulates
even though each block passes through once.

<figure class="bf-fig" id="network">
<p class="bf-title">The decoder as a network of queues</p>
<svg viewBox="0 0 720 300" role="img" aria-label="The decoder as a network of queues, close to a closed one. A feeder passes input to a single scanner thread, which decodes headers, tables and Huffman codes. Blocks enter an admission window holding W = 4k blocks in flight, then k workers undo move-to-front and runs, invert and emit, then a consumer takes blocks in order and writes the output. A slot returns to the window only when the next block in order is taken, and finished output is capped at 12 MiB, so one slow early block can stall the scanner."><rect x="10" y="40" width="96" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="58.0" y="62" text-anchor="middle" style="font-size:13px">feeder</text><text class="bf-tm" x="58.0" y="80" text-anchor="middle" style="font-size:11px">reads and</text><text class="bf-tm" x="58.0" y="95" text-anchor="middle" style="font-size:11px">copies input</text><rect x="128" y="40" width="122" height="92" rx="8" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="189.0" y="62" text-anchor="middle" style="font-size:13px">scanner</text><text class="bf-tm" x="189.0" y="80" text-anchor="middle" style="font-size:11px">one thread:</text><text class="bf-tm" x="189.0" y="95" text-anchor="middle" style="font-size:11px">headers, tables,</text><text class="bf-tm" x="189.0" y="110" text-anchor="middle" style="font-size:11px">Huffman decode</text><rect x="272" y="40" width="170" height="92" rx="8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text class="bf-t-strong" x="357.0" y="62" text-anchor="middle" style="font-size:13px">admission window</text><text class="bf-tm" x="357.0" y="80" text-anchor="middle" style="font-size:11px">W = 4k blocks in flight:</text><text class="bf-tm" x="357.0" y="95" text-anchor="middle" style="font-size:11px">queued, running, or done</text><text class="bf-tm" x="357.0" y="110" text-anchor="middle" style="font-size:11px">but not yet taken in order</text><rect x="464" y="40" width="122" height="92" rx="8" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="525.0" y="62" text-anchor="middle" style="font-size:13px">k workers</text><text class="bf-tm" x="525.0" y="80" text-anchor="middle" style="font-size:11px">MTF and runs,</text><text class="bf-tm" x="525.0" y="95" text-anchor="middle" style="font-size:11px">inversion,</text><text class="bf-tm" x="525.0" y="110" text-anchor="middle" style="font-size:11px">emission, CRC</text><rect x="608" y="40" width="102" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="659.0" y="62" text-anchor="middle" style="font-size:13px">consumer</text><text class="bf-tm" x="659.0" y="80" text-anchor="middle" style="font-size:11px">takes blocks</text><text class="bf-tm" x="659.0" y="95" text-anchor="middle" style="font-size:11px">in order,</text><text class="bf-tm" x="659.0" y="110" text-anchor="middle" style="font-size:11px">writes output</text><path d="M106,86.0 L120,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="128,86.0 120,81.0 120,91.0" style="fill:var(--bf-frame)"/><path d="M250,86.0 L264,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="272,86.0 264,81.0 264,91.0" style="fill:var(--bf-frame)"/><path d="M442,86.0 L456,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="464,86.0 456,81.0 456,91.0" style="fill:var(--bf-frame)"/><path d="M586,86.0 L600,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="608,86.0 600,81.0 600,91.0" style="fill:var(--bf-frame)"/><path d="M659,132 L659,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M659,184 L357,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M357,184 L357,141" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><polygon points="357,133 352,141 362,141" style="fill:var(--bf-steel)"/><text class="bf-tm" x="508" y="202" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">a slot is freed only when the next block in order is taken,</text><text class="bf-tm" x="508" y="217" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">and finished output is capped at 12 MiB</text><text class="bf-tm" x="10" y="244" style="font-size:11.5px">Admission credit circulates, though each block passes through once: close to a closed network,</text><text class="bf-tm" x="10" y="260" style="font-size:11.5px">not one. One slow early block can hold slots and stall the scanner.</text></svg>
<figcaption>The structure the measurements follow. Admission credit circulates; the data passes
through once. Start-up, drain and the 12 MiB cap on finished output are where the decoder departs
from a true closed network.</figcaption>
</figure>

That makes the decoder close to a closed batch network, and this post uses that model, but only as
an approximation. Our own course mining withdrew the claim that the decoder is a closed system: it
is a finite pipeline with feedback-controlled admission, because a finite stream starts empty and
drains, and the byte cap and the in-order consumer make admission depend on the state of the
decoder ([the summary](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md)).

<p class="il-rubric-head">Of laws that need no distributions</p>

Most of what follows rests on the operational laws, which relate quantities measured over an
interval and assume nothing about how arrivals or service times are distributed (Jain, ch. 33;
Harchol-Balter, ch. 6 and 7). Four carry the weight:

- **The utilisation law,** U = X·D: a station's busy fraction is its throughput times its demand.
- **Little's law,** N = X·R: the mean number of jobs in a system is its throughput times the mean
  time each spends there ([Little, 1961](https://doi.org/10.1287/opre.9.3.383)).
- **The forced-flow law,** Xᵢ = Vᵢ·X: each station's throughput is the system's throughput times
  the visits a job makes to that station.
- **The bottleneck law:** no station can be busy more than all of the time, so the system's
  throughput is at most the capacity of its busiest station over that station's demand,
  X ≤ min cᵢ/Dᵢ, where cᵢ is the number of servers at station i.

Because these laws hold over any measured interval, they audit the instruments as well as describe
the system. Stage 1 logged every block's route, stage boundaries and queue transitions, and all nine
identities built from these laws held in every one of its 840 decodes, Little's law exactly
([§7](#7-stage-1-the-decoder-across-twenty-counts)). A trace that broke one would have meant a
missing or misplaced timestamp, not a surprising decoder. The laws catch errors of reasoning too:
an early sizing of the admission window from Little's law used a block's service time where its
residence time, waiting included, belonged, and it was withdrawn ([journal §61](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)).

<p class="il-rubric-head">Of asking what if before building</p>

The same laws price changes. Modification analysis (Harchol-Balter, ch. 7) changes one station's
demand in the model and recomputes the bounds, which prices an improvement before anything is built.
Its lesson is that a faster component need not make a faster system. The four-arm test of §1 is an
instance: removing emission's work alone changed nothing measurable at ten workers, because the
scanner was the bottleneck and the workers had slack, while removing the scanner's work exposed
emission's. One of the course's 2025 recitations makes the same point with a cache, where raising the
hit ratio can lower throughput by crowding the path that hits take ([journal §54](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)). It is also
why a cut at the bottleneck is worth less than it looks: once the scanner is relieved enough, the
workers bind, and the bottleneck migrates.

## 3. The first model, and its first test

The simplest model is a serial stage feeding k identical parallel servers, and the bottleneck law
bounds it directly. If each block costs the scanner s and a worker w, the scanner can finish at most
1/s blocks a second and the workers together k/w, so X(k) ≤ min(k/w, 1/s): the workers set the pace
until there are enough of them that the scanner cannot keep up, and from then on the scanner alone
does. The crossover falls where the two limits meet, at k\* = w/s. From the measurements at one to
four workers only, s = 106.6 µs and w = 798.3 µs a block, which puts the knee at 7.49 workers. Across all twenty counts the measured
crossover is 7.67, and a fit of c + max(b, a/k) gives a = 1.279 s, b = 0.178 s and c = 0.032 s
([journal §57](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md); [the data](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv)).

Nothing in that prediction was fitted to the knee: it is a general law applied to two measured
service times. The bound and the measured decoder are drawn together below; moving the sliders
shows how each service time moves the knee.

<figure class="bf-fig bf-twostation" id="two-station" data-x0="70" data-x1="700" data-y0="24" data-y1="304" data-ymax="20000">
<p class="bf-title">The bottleneck law, with the measured decoder</p>
<svg viewBox="0 0 720 360" role="img" aria-label="The two-station bound on throughput against workers from 1 to 20: the workers' limit k/w rises in a straight line until it meets the scanner's ceiling 1/s at the knee, 7.49 workers with the measured service times, 106.6 µs for the scanner and 798.3 µs for a worker; the measured decoder follows the bound closely, just below it, peaking at about 9,000 blocks a second at 8 workers."><defs><clipPath id="ts-clip"><rect x="70" y="24" width="630" height="280"/></clipPath></defs><line class="bf-rule" x1="70" x2="700" y1="304.0" y2="304.0" opacity="0.9"/><text class="bf-tm" x="62" y="308.0" text-anchor="end" style="font-size:12px">0k</text><line class="bf-rule" x1="70" x2="700" y1="234.0" y2="234.0" opacity="0.35"/><text class="bf-tm" x="62" y="238.0" text-anchor="end" style="font-size:12px">5k</text><line class="bf-rule" x1="70" x2="700" y1="164.0" y2="164.0" opacity="0.35"/><text class="bf-tm" x="62" y="168.0" text-anchor="end" style="font-size:12px">10k</text><line class="bf-rule" x1="70" x2="700" y1="94.0" y2="94.0" opacity="0.35"/><text class="bf-tm" x="62" y="98.0" text-anchor="end" style="font-size:12px">15k</text><line class="bf-rule" x1="70" x2="700" y1="24.0" y2="24.0" opacity="0.35"/><text class="bf-tm" x="62" y="28.0" text-anchor="end" style="font-size:12px">20k</text><text class="bf-tm" x="70.0" y="322" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="169.5" y="322" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="302.1" y="322" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="434.7" y="322" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="567.4" y="322" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="700.0" y="322" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="385" y="342" text-anchor="middle" style="font-size:12px">workers, k</text><text class="bf-tm" x="16" y="164" text-anchor="middle" transform="rotate(-90 16 164)" style="font-size:12px">blocks a second</text><g clip-path="url(#ts-clip)"><polyline class="bf-ts-work" points="70.0,286.5 73.3,284.7 76.6,283.0 79.9,281.2 83.3,279.4 86.6,277.7 89.9,275.9 93.2,274.2 96.5,272.4 99.8,270.7 103.2,268.9 106.5,267.2 109.8,265.4 113.1,263.7 116.4,261.9 119.7,260.2 123.1,258.4 126.4,256.6 129.7,254.9 133.0,253.1 136.3,251.4 139.6,249.6 142.9,247.9 146.3,246.1 149.6,244.4 152.9,242.6 156.2,240.9 159.5,239.1 162.8,237.4 166.2,235.6 169.5,233.9 172.8,232.1 176.1,230.3 179.4,228.6 182.7,226.8 186.1,225.1 189.4,223.3 192.7,221.6 196.0,219.8 199.3,218.1 202.6,216.3 205.9,214.6 209.3,212.8 212.6,211.1 215.9,209.3 219.2,207.5 222.5,205.8 225.8,204.0 229.2,202.3 232.5,200.5 235.8,198.8 239.1,197.0 242.4,195.3 245.7,193.5 249.1,191.8 252.4,190.0 255.7,188.3 259.0,186.5 262.3,184.7 265.6,183.0 268.9,181.2 272.3,179.5 275.6,177.7 278.9,176.0 282.2,174.2 285.5,172.5 288.8,170.7 292.2,169.0 295.5,167.2 298.8,165.5 302.1,163.7 305.4,161.9 308.7,160.2 312.1,158.4 315.4,156.7 318.7,154.9 322.0,153.2 325.3,151.4 328.6,149.7 331.9,147.9 335.3,146.2 338.6,144.4 341.9,142.7 345.2,140.9 348.5,139.1 351.8,137.4 355.2,135.6 358.5,133.9 361.8,132.1 365.1,130.4 368.4,128.6 371.7,126.9 375.1,125.1 378.4,123.4 381.7,121.6 385.0,119.9 388.3,118.1 391.6,116.4 394.9,114.6 398.3,112.8 401.6,111.1 404.9,109.3 408.2,107.6 411.5,105.8 414.8,104.1 418.2,102.3 421.5,100.6 424.8,98.8 428.1,97.1 431.4,95.3 434.7,93.6 438.1,91.8 441.4,90.0 444.7,88.3 448.0,86.5 451.3,84.8 454.6,83.0 457.9,81.3 461.3,79.5 464.6,77.8 467.9,76.0 471.2,74.3 474.5,72.5 477.8,70.8 481.2,69.0 484.5,67.2 487.8,65.5 491.1,63.7 494.4,62.0 497.7,60.2 501.1,58.5 504.4,56.7 507.7,55.0 511.0,53.2 514.3,51.5 517.6,49.7 520.9,48.0 524.3,46.2 527.6,44.4 530.9,42.7 534.2,40.9 537.5,39.2 540.8,37.4 544.2,35.7 547.5,33.9 550.8,32.2 554.1,30.4 557.4,28.7 560.7,26.9 564.1,25.2 567.4,24.0 570.7,24.0 574.0,24.0 577.3,24.0 580.6,24.0 583.9,24.0 587.3,24.0 590.6,24.0 593.9,24.0 597.2,24.0 600.5,24.0 603.8,24.0 607.2,24.0 610.5,24.0 613.8,24.0 617.1,24.0 620.4,24.0 623.7,24.0 627.1,24.0 630.4,24.0 633.7,24.0 637.0,24.0 640.3,24.0 643.6,24.0 646.9,24.0 650.3,24.0 653.6,24.0 656.9,24.0 660.2,24.0 663.5,24.0 666.8,24.0 670.2,24.0 673.5,24.0 676.8,24.0 680.1,24.0 683.4,24.0 686.7,24.0 690.1,24.0 693.4,24.0 696.7,24.0 700.0,24.0" style="fill:none;stroke:var(--bf-blue);stroke-width:1.5;stroke-dasharray:5 4"/><line class="bf-ts-scan" x1="70" x2="700" y1="172.7" y2="172.7" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><polyline class="bf-ts-bound" points="70.0,286.5 73.3,284.7 76.6,283.0 79.9,281.2 83.3,279.4 86.6,277.7 89.9,275.9 93.2,274.2 96.5,272.4 99.8,270.7 103.2,268.9 106.5,267.2 109.8,265.4 113.1,263.7 116.4,261.9 119.7,260.2 123.1,258.4 126.4,256.6 129.7,254.9 133.0,253.1 136.3,251.4 139.6,249.6 142.9,247.9 146.3,246.1 149.6,244.4 152.9,242.6 156.2,240.9 159.5,239.1 162.8,237.4 166.2,235.6 169.5,233.9 172.8,232.1 176.1,230.3 179.4,228.6 182.7,226.8 186.1,225.1 189.4,223.3 192.7,221.6 196.0,219.8 199.3,218.1 202.6,216.3 205.9,214.6 209.3,212.8 212.6,211.1 215.9,209.3 219.2,207.5 222.5,205.8 225.8,204.0 229.2,202.3 232.5,200.5 235.8,198.8 239.1,197.0 242.4,195.3 245.7,193.5 249.1,191.8 252.4,190.0 255.7,188.3 259.0,186.5 262.3,184.7 265.6,183.0 268.9,181.2 272.3,179.5 275.6,177.7 278.9,176.0 282.2,174.2 285.5,172.7 288.8,172.7 292.2,172.7 295.5,172.7 298.8,172.7 302.1,172.7 305.4,172.7 308.7,172.7 312.1,172.7 315.4,172.7 318.7,172.7 322.0,172.7 325.3,172.7 328.6,172.7 331.9,172.7 335.3,172.7 338.6,172.7 341.9,172.7 345.2,172.7 348.5,172.7 351.8,172.7 355.2,172.7 358.5,172.7 361.8,172.7 365.1,172.7 368.4,172.7 371.7,172.7 375.1,172.7 378.4,172.7 381.7,172.7 385.0,172.7 388.3,172.7 391.6,172.7 394.9,172.7 398.3,172.7 401.6,172.7 404.9,172.7 408.2,172.7 411.5,172.7 414.8,172.7 418.2,172.7 421.5,172.7 424.8,172.7 428.1,172.7 431.4,172.7 434.7,172.7 438.1,172.7 441.4,172.7 444.7,172.7 448.0,172.7 451.3,172.7 454.6,172.7 457.9,172.7 461.3,172.7 464.6,172.7 467.9,172.7 471.2,172.7 474.5,172.7 477.8,172.7 481.2,172.7 484.5,172.7 487.8,172.7 491.1,172.7 494.4,172.7 497.7,172.7 501.1,172.7 504.4,172.7 507.7,172.7 511.0,172.7 514.3,172.7 517.6,172.7 520.9,172.7 524.3,172.7 527.6,172.7 530.9,172.7 534.2,172.7 537.5,172.7 540.8,172.7 544.2,172.7 547.5,172.7 550.8,172.7 554.1,172.7 557.4,172.7 560.7,172.7 564.1,172.7 567.4,172.7 570.7,172.7 574.0,172.7 577.3,172.7 580.6,172.7 583.9,172.7 587.3,172.7 590.6,172.7 593.9,172.7 597.2,172.7 600.5,172.7 603.8,172.7 607.2,172.7 610.5,172.7 613.8,172.7 617.1,172.7 620.4,172.7 623.7,172.7 627.1,172.7 630.4,172.7 633.7,172.7 637.0,172.7 640.3,172.7 643.6,172.7 646.9,172.7 650.3,172.7 653.6,172.7 656.9,172.7 660.2,172.7 663.5,172.7 666.8,172.7 670.2,172.7 673.5,172.7 676.8,172.7 680.1,172.7 683.4,172.7 686.7,172.7 690.1,172.7 693.4,172.7 696.7,172.7 700.0,172.7" style="fill:none;stroke:var(--bf-steel);stroke-width:3"/><line class="bf-ts-kline" x1="285.2" x2="285.2" y1="172.7" y2="304" style="stroke:var(--bf-steel);stroke-width:1;stroke-dasharray:3 3"/><circle class="bf-ts-knee" cx="285.2" cy="172.7" r="5" style="fill:var(--bf-steel)"/></g><circle cx="70.0" cy="285.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="103.2" cy="268.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="136.3" cy="252.6" r="3.5" style="fill:var(--bf-green)"/><circle cx="169.5" cy="236.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="202.6" cy="219.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="235.8" cy="202.6" r="3.5" style="fill:var(--bf-green)"/><circle cx="268.9" cy="187.3" r="3.5" style="fill:var(--bf-green)"/><circle cx="302.1" cy="177.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="335.3" cy="181.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="368.4" cy="184.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="401.6" cy="181.3" r="3.5" style="fill:var(--bf-green)"/><circle cx="434.7" cy="181.7" r="3.5" style="fill:var(--bf-green)"/><circle cx="467.9" cy="181.2" r="3.5" style="fill:var(--bf-green)"/><circle cx="501.1" cy="182.0" r="3.5" style="fill:var(--bf-green)"/><circle cx="534.2" cy="189.8" r="3.5" style="fill:var(--bf-green)"/><circle cx="567.4" cy="192.4" r="3.5" style="fill:var(--bf-green)"/><circle cx="600.5" cy="200.5" r="3.5" style="fill:var(--bf-green)"/><circle cx="633.7" cy="200.4" r="3.5" style="fill:var(--bf-green)"/><circle cx="666.8" cy="199.9" r="3.5" style="fill:var(--bf-green)"/><circle cx="700.0" cy="199.2" r="3.5" style="fill:var(--bf-green)"/><text class="bf-t-strong bf-ts-kstar" x="700" y="40" text-anchor="end" style="font-size:15px">knee k* = w/s = 7.49 workers</text><text class="bf-tm bf-ts-ceil" x="700" y="60" text-anchor="end" style="font-size:13px">scanner ceiling 1/s = 9,381 blocks a second</text><rect x="367.05263157894734" y="213.6" width="328.94736842105266" height="76" rx="4" style="fill:var(--bf-surface);opacity:0.9"/><line x1="375.05263157894734" x2="397.05263157894734" y1="225.6" y2="225.6" style="stroke:var(--bf-steel);stroke-width:3"/><text class="bf-tm" x="405.05263157894734" y="229.6" style="font-size:12.5px">the bound, min(k/w, 1/s)</text><line x1="375.05263157894734" x2="397.05263157894734" y1="243.6" y2="243.6" style="stroke:var(--bf-blue);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-tm" x="405.05263157894734" y="247.6" style="font-size:12.5px">the workers' limit, k/w</text><line x1="375.05263157894734" x2="397.05263157894734" y1="261.6" y2="261.6" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-tm" x="405.05263157894734" y="265.6" style="font-size:12.5px">the scanner's limit, 1/s</text><circle cx="386.05263157894734" cy="279.6" r="3.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="405.05263157894734" y="283.6" style="font-size:12.5px">measured: 1,724 blocks over the median wall</text></svg>
<div class="bf-ts-controls" hidden><label>scanner, s <input class="bf-ts-s" type="range" min="50" max="250" step="0.1" value="106.6" aria-label="scanner service time, microseconds a block"> <output class="bf-ts-sv">106.6 µs</output></label><label>worker, w <input class="bf-ts-w" type="range" min="200" max="1600" step="0.1" value="798.3" aria-label="worker service time, microseconds a block"> <output class="bf-ts-wv">798.3 µs</output></label><button type="button" class="bf-ts-reset">Reset to the measured values</button></div>
<figcaption>The idealised bound from the operational laws, with no start-up, drain, contention or blocking, against the decoder measured on an i9-10900K (median of 20 decodes at each count). With JavaScript, the sliders set the two service times: a cheaper scanner raises the ceiling and moves the knee right, a cheaper worker steepens the rise and moves it left.</figcaption>
</figure>

In wall time, with the fitted curve, the same picture reads as follows.

<figure class="bf-fig" id="knee">
<p class="bf-title">Wall time against workers: the knee predicted from four counts</p>
<svg viewBox="0 0 720 400" role="img" aria-label="Decode wall time against workers from 1 to 20 on the i9-10900K. It falls as a over k, from 1.30 s at one worker, to a best of 0.191 s at 8 workers, stays near 0.197 s from 8 to 14, and steps up to about 0.23 s from 16. The knee predicted from counts 1 to 4 alone is 7.49 workers; the measured crossover is 7.67."><line x1="64" x2="696" y1="282.3" y2="282.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="286.3" text-anchor="end" style="font-size:12px">0.1</text><line x1="64" x2="696" y1="238.6" y2="238.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="242.6" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="194.9" y2="194.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="198.9" text-anchor="end" style="font-size:12px">0.3</text><line x1="64" x2="696" y1="151.1" y2="151.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="155.1" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="107.4" y2="107.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="111.4" text-anchor="end" style="font-size:12px">0.5</text><line x1="64" x2="696" y1="63.7" y2="63.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="67.7" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">0.7</text><text class="bf-tm" x="64.0" y="344" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="344" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="344" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="344" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="344" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="344" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="362" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="173" text-anchor="middle" transform="rotate(-90 16 173)" style="font-size:12px">decode wall time (s)</text><polyline points="64.0,20.0 67.3,20.0 70.7,20.0 74.0,20.0 77.3,20.0 80.6,20.0 84.0,20.0 87.3,20.0 90.6,20.0 93.9,20.0 97.3,32.5 100.6,45.8 103.9,57.9 107.2,68.9 110.6,79.1 113.9,88.4 117.2,97.0 120.5,104.9 123.9,112.3 127.2,119.2 130.5,125.6 133.9,131.7 137.2,137.3 140.5,142.6 143.8,147.6 147.2,152.3 150.5,156.7 153.8,160.9 157.1,164.9 160.5,168.7 163.8,172.2 167.1,175.6 170.4,178.9 173.8,182.0 177.1,184.9 180.4,187.8 183.7,190.5 187.1,193.1 190.4,195.5 193.7,197.9 197.1,200.2 200.4,202.4 203.7,204.5 207.0,206.5 210.4,208.5 213.7,210.4 217.0,212.2 220.3,213.9 223.7,215.6 227.0,217.2 230.3,218.8 233.6,220.4 237.0,221.8 240.3,223.3 243.6,224.7 246.9,226.0 250.3,227.3 253.6,228.6 256.9,229.8 260.3,231.0 263.6,232.1 266.9,233.3 270.2,234.2 273.6,234.2 276.9,234.2 280.2,234.2 283.5,234.2 286.9,234.2 290.2,234.2 293.5,234.2 296.8,234.2 300.2,234.2 303.5,234.2 306.8,234.2 310.1,234.2 313.5,234.2 316.8,234.2 320.1,234.2 323.5,234.2 326.8,234.2 330.1,234.2 333.4,234.2 336.8,234.2 340.1,234.2 343.4,234.2 346.7,234.2 350.1,234.2 353.4,234.2 356.7,234.2 360.0,234.2 363.4,234.2 366.7,234.2 370.0,234.2 373.3,234.2 376.7,234.2 380.0,234.2 383.3,234.2 386.7,234.2 390.0,234.2 393.3,234.2 396.6,234.2 400.0,234.2 403.3,234.2 406.6,234.2 409.9,234.2 413.3,234.2 416.6,234.2 419.9,234.2 423.2,234.2 426.6,234.2 429.9,234.2 433.2,234.2 436.5,234.2 439.9,234.2 443.2,234.2 446.5,234.2 449.9,234.2 453.2,234.2 456.5,234.2 459.8,234.2 463.2,234.2 466.5,234.2 469.8,234.2 473.1,234.2 476.5,234.2 479.8,234.2 483.1,234.2 486.4,234.2 489.8,234.2 493.1,234.2 496.4,234.2 499.7,234.2 503.1,234.2 506.4,234.2 509.7,234.2 513.1,234.2 516.4,234.2 519.7,234.2 523.0,234.2 526.4,234.2 529.7,234.2 533.0,234.2 536.3,234.2 539.7,234.2 543.0,234.2 546.3,234.2 549.6,234.2 553.0,234.2 556.3,234.2 559.6,234.2 562.9,234.2 566.3,234.2 569.6,234.2 572.9,234.2 576.3,234.2 579.6,234.2 582.9,234.2 586.2,234.2 589.6,234.2 592.9,234.2 596.2,234.2 599.5,234.2 602.9,234.2 606.2,234.2 609.5,234.2 612.8,234.2 616.2,234.2 619.5,234.2 622.8,234.2 626.1,234.2 629.5,234.2 632.8,234.2 636.1,234.2 639.5,234.2 642.8,234.2 646.1,234.2 649.4,234.2 652.8,234.2 656.1,234.2 659.4,234.2 662.7,234.2 666.1,234.2 669.4,234.2 672.7,234.2 676.0,234.2 679.4,234.2 682.7,234.2 686.0,234.2 689.3,234.2 692.7,234.2 696.0,234.2" style="fill:none;stroke:var(--bf-steel);stroke-width:2"/><line x1="279.8" x2="279.8" y1="20" y2="326" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-t-strong" x="285.8" y="36" style="font-size:12px;fill:var(--bf-amber)">knee predicted from counts 1–4: 7.49</text><text class="bf-tm" x="285.8" y="52" style="font-size:12px">measured crossover: 7.67</text><path d="M64.0,22 l-5,8 h10 z" style="fill:var(--bf-green)"/><text class="bf-tm" x="72.0" y="30" style="font-size:11px">1.30 s</text><circle cx="97.3" cy="27.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="130.5" cy="120.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="163.8" cy="170.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="197.1" cy="201.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="230.3" cy="221.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="263.6" cy="235.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="296.8" cy="242.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="330.1" cy="239.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="363.4" cy="238.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="396.6" cy="240.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="429.9" cy="239.7" r="4.5" style="fill:var(--bf-green)"/><circle cx="463.2" cy="240.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="496.4" cy="239.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="529.7" cy="233.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="562.9" cy="231.4" r="4.5" style="fill:var(--bf-green)"/><circle cx="596.2" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="629.5" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="662.7" cy="224.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="696.0" cy="225.3" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="137.2" y="129.3" style="font-size:12px">workers bind: wall ≈ a/k</text><text class="bf-tm" x="396.6" y="212.3" style="font-size:12px">scanner binds: wall ≈ b</text><text class="bf-tm" x="549.6" y="194.9" style="font-size:12px">step at 16: the scanner slows 23%</text><circle cx="74" cy="388" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="84" y="392" style="font-size:12px">measured (median of 20 decodes)</text><line x1="324" x2="354" y1="388" y2="388" style="stroke:var(--bf-steel);stroke-width:2"/><text class="bf-tm" x="362" y="392" style="font-size:12px">fitted two-station model, c + max(b, a/k)</text></svg>
<figcaption>Measured on an i9-10900K (10 cores, 20 threads), one corpus of 1,724 blocks and 839 MB
of output, median of 20 decodes at each count, unpinned. The fitted model (steel) has the workers'
asymptote a/k on the left and the scanner's b on the right; the dashed line is the knee predicted
from counts 1 to 4 alone. The wall at one worker, 1.30 s, is off the scale.</figcaption>
</figure>

The best wall is 0.191 s, at 8 workers (4,390 MB/s of output), and from 8 to 14 workers it stays
flat at 0.197 ± 0.005 s. At 16 and above it steps up 20%, to 0.230 s, because the scanner itself
slows by 23% while its share of the wall stays at 0.97. The scanner's work is 81% Huffman decoding,
at 1.41 ns a symbol at one worker and 1.51 ns from three. The second figure shows the switch from
the scanner's side: at one worker it spends most of its time waiting for a free slot, and from 8
workers it almost never waits.

<figure class="bf-fig" id="scanner-split">
<p class="bf-title">The scanner's time, working and waiting</p>
<svg viewBox="0 0 720 350" role="img" aria-label="The scanner thread's time against workers. Its work stays near 0.19 s from 3 to 15 workers and rises to about 0.23 s from 16. Its waiting for a free slot falls from 1.12 s at one worker to under a millisecond at 8 and beyond."><line x1="64" x2="696" y1="239.4" y2="239.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="243.4" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="202.9" y2="202.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="206.9" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="166.3" y2="166.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="170.3" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="129.7" y2="129.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="133.7" text-anchor="end" style="font-size:12px">0.8</text><line x1="64" x2="696" y1="93.1" y2="93.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="97.1" text-anchor="end" style="font-size:12px">1</text><line x1="64" x2="696" y1="56.6" y2="56.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="60.6" text-anchor="end" style="font-size:12px">1.2</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">1.4</text><text class="bf-tm" x="64.0" y="294" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="294" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="294" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="294" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="294" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="294" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="312" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="148" text-anchor="middle" transform="rotate(-90 16 148)" style="font-size:12px">scanner thread time (s)</text><rect x="54.2" y="244.2" width="19.6" height="31.8" style="fill:var(--bf-amber)"/><rect x="54.2" y="39.5" width="19.6" height="204.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="87.5" y="242.7" width="19.6" height="33.3" style="fill:var(--bf-amber)"/><rect x="87.5" y="151.6" width="19.6" height="91.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="120.7" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="120.7" y="191.4" width="19.6" height="50.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="154.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="154.0" y="212.1" width="19.6" height="30.0" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="187.3" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="187.3" y="225.2" width="19.6" height="16.9" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="220.5" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="220.5" y="233.5" width="19.6" height="8.6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="253.8" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="253.8" y="239.2" width="19.6" height="2.8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="287.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="287.0" y="241.8" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="320.3" y="241.6" width="19.6" height="34.4" style="fill:var(--bf-amber)"/><rect x="320.3" y="240.9" width="19.6" height="0.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="353.6" y="241.4" width="19.6" height="34.6" style="fill:var(--bf-amber)"/><rect x="353.6" y="240.9" width="19.6" height="0.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="386.8" y="241.3" width="19.6" height="34.7" style="fill:var(--bf-amber)"/><rect x="386.8" y="241.0" width="19.6" height="0.3" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="420.1" y="241.2" width="19.6" height="34.8" style="fill:var(--bf-amber)"/><rect x="420.1" y="241.0" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="453.4" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="453.4" y="240.9" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="486.6" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="519.9" y="240.8" width="19.6" height="35.2" style="fill:var(--bf-amber)"/><rect x="553.2" y="234.0" width="19.6" height="42.0" style="fill:var(--bf-amber)"/><rect x="586.4" y="234.7" width="19.6" height="41.3" style="fill:var(--bf-amber)"/><rect x="619.7" y="234.3" width="19.6" height="41.7" style="fill:var(--bf-amber)"/><rect x="652.9" y="234.6" width="19.6" height="41.4" style="fill:var(--bf-amber)"/><rect x="686.2" y="234.8" width="19.6" height="41.2" style="fill:var(--bf-amber)"/><text class="bf-tm" x="346.7" y="162.6" style="font-size:12px">from 8 workers the scanner almost never waits:</text><text class="bf-tm" x="346.7" y="178.6" style="font-size:12px">it is the bottleneck, and only less work there helps</text><rect x="68" y="330" width="12" height="12" style="fill:var(--bf-amber)"/><text class="bf-tm" x="86" y="341" style="font-size:12px">scanner working</text><rect x="234" y="330" width="12" height="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><text class="bf-tm" x="252" y="341" style="font-size:12px">scanner waiting for a free slot</text></svg>
<figcaption>Measured, same run. Below the knee the scanner waits for the workers to free a slot;
from 8 workers on it works without pause, so it is the bottleneck and only a lighter scanner moves
the wall.</figcaption>
</figure>

## 4. What scheduling can and cannot do

One question follows directly: could a better queueing discipline, the rule for which waiting block
is served next, do far better? The operational laws answer it before any code is written. No
discipline raises the ceiling, which the demands set; a discipline can only bring the decoder closer
to it ([the discipline lead](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md)). The decoder already runs at 0.945 to 0.998 of the busiest
server's limit at every worker count, so no reordering could recover more than about 5%, or 3% on
the plateau. The literature gives a further reason: scheduling matters much less in closed systems
than in open ones ([Schroeder, Wierman and Harchol-Balter, NSDI 2006](https://www.usenix.org/conference/nsdi-06/open-versus-closed-cautionary-tale)),
and the decoder's bounded admission puts it nearer the closed kind.

The techniques considered, and what became of each
([the discipline lead](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md); [the opportunities, with their corrections](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md)):

| Technique | What it would change | What became of it |
|---|---|---|
| Size-based scheduling: largest or shortest first, or assignment by size | big blocks stop becoming the straggler or the head of the line | not supported yet: the block mixture varies (C² about 0.98), but worker service per block was never measured |
| Helpers for the oldest unfinished block | shortens the block holding up in-order output | open: the helper machinery exists, and the trigger is the question |
| Admission by work rather than by count | a few huge blocks can no longer fill the window | open |
| Relaxed resequencing | finished blocks stop holding admission credit while they wait | limited by the output contract: a byte stream is written in order |
| Redundancy: run a slow block twice | helps when servers' delays are independent and variable | ruled out: here they are neither, and the copy costs the scarce resource |
| Heavy-tail and matrix-analytic models | exact models for heavy-tailed service | not needed on this corpus, which has no heavy tail; phase-type fits deferred, as simulation is cheaper |
| A parallel scanner | turns the serial station into a parallel group | the only row that raises the ceiling |

The last row is a different kind of change. A discipline moves the decoder closer to a fixed
ceiling; a change of topology moves the ceiling itself, and that is where the larger gains lie.

## 5. Product form, and where it stops

Beyond the operational laws lies a line of exact results for networks of queues. Jackson showed in
1957 that certain open networks have a product-form equilibrium: the joint distribution of the
queue lengths is a product of one factor per station, as if each station were alone
([Jackson, 1957](https://doi.org/10.1287/opre.5.4.518)). Gordon and Newell gave the same for closed
networks of exponential servers ([1967](https://doi.org/10.1287/opre.15.2.254)). Baskett, Chandy,
Muntz and Palacios extended product form to open, closed and mixed networks with several classes of
job and several service disciplines, the BCMP theorem ([1975](https://doi.org/10.1145/321879.321887)).
Reiser and Lavenberg's mean-value analysis then computed the mean queue lengths, response times and
throughputs of closed product-form networks by a recursion over the population, without the full
distribution ([1980](https://doi.org/10.1145/322186.322195)). MVA is the natural tool for the
decoder's knee, since it gives the smooth curve between the bottleneck law's two asymptotes, and it
was planned beside the simulation from the start ([journal §52](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)).

The design reviews found that the decoder breaks the assumptions product form needs
([journal §59](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md); [the opportunities](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md)):

| Product form assumes | The decoder has |
|---|---|
| a fixed population that circulates | an admission ceiling, W = 4k, on a finite stream that fills and drains |
| stations that release jobs independently | in-order retirement: a finished block waits for the one before it |
| service rates that don't depend on the rest of the system | workers that slow one another and the scanner through shared caches and cores |
| admission that doesn't depend on state | a cap on finished output that gates new scanning |
| service times of one kind | two classes of block, 792 tiny and 932 real, whose sizes are correlated along the stream |

So exact MVA applies only to a documented surrogate network, with its error against a fuller model
reported as approximation error, and the primary model is a discrete-event simulation driven by the
measured trace of every block. The operational laws survive intact, since they assume none of this.
The order of the reading is the useful part: the elementary laws held, the elegant exact theory did
not, and the simulation that replaces it is checked against the laws.

## 6. Reading the field, four times over

Three performance-analysis courses were read in full, with their sources saved, and then mined for
opportunities against these measurements by four readers, each writing an answer before seeing the
others ([the summary](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md)). The fourth, aimed at where mathematics rather than engineering fits, is the subject of [§8](#8-the-fourth-reading-and-a-design-space).

| Reader | What it found that the others had not |
|---|---|
| [The first reading](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md) | right-sizing the worker count; putting the bottleneck on the fastest core; mean-value analysis in place of the hard-max fit |
| [An outside model](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md) | the public API's actual worker count; bottleneck migration; the variability of block sizes |
| [A second agent](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md) | two avoidable copies in the scanner, about 8% of its demand; the same worker-count error |
| [The fourth reading](https://github.com/lmmx/mibz2/blob/7f18ccf719a0fd7eb863f8aa519a4fb8a8f2295b/docs/journal/monodromy/2026-11-25-course-mining-and-analysis-plan.md) | the scan as a chain at its latency floor; a design space for the scanner–worker boundary; construction as the largest worker target |

The disagreements were the useful part. The first reading had claimed that the Python binding's
default of 20 workers costs 20% of the wall. Two readers, independently, found that the public API
never runs 20 workers on this machine: a request for ten or more threads builds 7 workers with a
28-slot window, so the live question is 7 against 8, 0.207 s against 0.191 s, about 7.6% for under
1% more CPU ([journal §61](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). The cap of 7 was set because workers beyond it bought nothing; it
is a symptom of the scanner's limit rather than a reason to stop raising it.

The second agent's reading found engineering that the first had missed, both confirmed in the code
([journal §62](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). At the boundary between the feeder's 4 MiB chunks and the scanner's 2 MiB
look-ahead, the scanner copies the held part of its window on its own thread, about 10 ms at 8
workers, or 5.4% of its demand. And each block's symbol buffer is zero-filled again when it is
reused, about 5 ms more, driven by the 792 tiny blocks in the corpus. Together that is about 15 ms,
8% of the scanner's demand, with no algorithmic risk, and it has been routed for implementation as
an ordinary performance change.

The outside reading supplied the warning that prices every such change. A cut in the scanner's
work moves the bottleneck back to the workers: at 8 workers, a 5% cut in the scanner's Huffman
demand already reaches the workers' limit, so scanner and worker improvements must be priced
together, with the worker count retuned. Stage 1's figures then re-priced the copy fix itself: at 8
workers the workers sit within 0.3 ms of the scanner, so a 15 ms cut there is almost entirely
masked (0.1941 s against 0.1944 s), while at 10 workers it is worth about 7.5%
([journal §69](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). Those figures are instrumented, so a paired run at 8, 9 and 10 workers
decides between the two estimates.

## 7. Stage 1: the decoder across twenty counts

The first stage of the characterisation ran 840 grid decodes across every count from 1 to 20, with
every block's route, stage boundaries and queue transitions logged
([journal §66](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md); [the report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt)). All nine operational-law identities held in every
decode, Little's law exactly. What it found:

- **The two-station model holds.** The limit moves from the workers to the scanner at exactly 8
  workers, with a bootstrap interval of [8, 8], and a best wall of 0.1944 s there.
- **The scanner limits above the knee, by concurrent state.** The workers' idle time while the
  scanner is working rises from 0.035 of their time at 8 workers to 0.124 at 10 and 0.551 at 20;
  their idle time while it waits for input stays under 0.01.
- **The reader is not a co-bottleneck:** its demand is 0.017–0.035 ms a block against the scanner's
  0.099–0.130.
- **Contention inflates service per visit.** The scanner's Huffman decode costs 84 µs a block at
  one worker and 105–107 µs at 16 and above; emission rises from 526 µs at one worker to 816 at 14,
  and the memory controller's reads per decode from 219 MiB to 1,957 at 16, which at about 8 GB/s
  at 10 workers points to cache capacity rather than bandwidth.
- **The public API selects an older kernel below 8.** Asking for 3, 5 or 7 threads builds the
  eager kernel, because the lazy plan begins at 8 workers and the measured settings cover only 4,
  6 and 10 or more. At 7 threads that is 0.337 s and 140 MB, against 0.209 s and 54 MB for seven
  lazy workers: 61% slower, with 2.6 times the memory. It is a production default, so it goes to a
  registered recalibration across corpora.

The run flagged its own instrument as perturbing at 10 and 20 workers: the cost check's intervals
include 1 but are wide, so timings from the ledgers at high counts may be inflated by up to about
9%, while shares, counts and the identities are unaffected.

## 8. The fourth reading, and a design space

The fourth reading registered its plan for analysing stages 1 and 2 before stage 1 ran, and then
asked what the measurements imply about the scanner itself ([record 129](https://github.com/lmmx/mibz2/blob/7f18ccf719a0fd7eb863f8aa519a4fb8a8f2295b/docs/journal/monodromy/2026-11-25-course-mining-and-analysis-plan.md)). At 1.41 to
1.51 ns a symbol, the Huffman scan takes about 7 cycles a symbol, one first-level cache load's
latency: the scan is a single dependent chain running at its latency floor. The lever is
therefore fewer lookups a symbol, decoding several symbols with each lookup. A 10-bit window bounds
that at about 2.6 symbols a lookup, and the reading conjectures 13 to 17% off the best wall; the
cheapest test is to count cycles a symbol on the scanner at one worker, where about 7 cycles and no
cache misses would confirm the floor.

A companion record enumerated the ways the scanner's obligations could be divided between the
scanner and the workers, and scored each placement with the two-station law
([record 130](https://github.com/lmmx/mibz2/blob/9c3690f299574ea5296e189c8f3d74e7dcfb19af/docs/journal/monodromy/2026-11-25-design-space-and-identification.md)). Three placements lead, all conjectured: an index of block starts, so that
each worker scans its own block, at −15%; decoding several symbols a lookup, still serially, at
−17%; and both together at −19%. Its separating principle is to keep an obligation on a serial path
only if its state has no small summary, and to cut the path wherever the format resets that state.
One placement it rules out: decoding a single block's Huffman codes in parallel. The table is
chosen afresh every 50 symbols by a running symbol count that nothing in the stream
re-synchronises, so each chunk would have to carry at least 50 hypotheses, and a k-way split
loses for every k below 50; the obstruction is proved from the format, its cost argued.

The same record measured where the workers' time goes on a real block at one worker: construction
of the inversion's structures 45%, emission 38%, move-to-front and run records 10%, and the final
run-length expansion and checksum 6%, shares that hold at 8 and 16 workers. Construction, not
emission, is the largest target for mathematics in the workers. Of the two leads at the scanner,
decoding several symbols a lookup became an engineering prototype, and the index of block starts
the larger structural lead.

<p class="il-rubric-head">Of a price already paid</p>

The prototype found the fourth reading's price for decoding several symbols a lookup built on a
premise that was already out of date
([journal §82](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)). They assumed one symbol a lookup, but master's table has held up to two
symbols an entry since [#745](https://github.com/lmmx/mibz2/pull/745): on wikidata it takes 1.85
symbols a lookup, 84% of them pairs, so the 13 to 17% was the price of a change already made. The
prototype then tried a third symbol a lookup
([#870](https://github.com/lmmx/mibz2/pull/870)). Its output was byte-identical and its lookups
fell 12%, but its time did not: the scanner ran 0.98 to 1.01 of master's time a symbol on
wikidata, and slower on two other corpora: a lookup already costs about twice its dependency
chain, and the third symbol's gain was eaten by mispredictions and spills in the tail paths. It was rejected. What remains at the scanner is its copies,
about 2% of its time from counting each batch's bits at once, a core of its own, and the
structural answer, a scan split across cores, which removes the single serial core instead of
speeding it.

Stage 1's traces have also been analysed by program ([journal §71](https://github.com/lmmx/mibz2/blob/3ad28015eb83951d79fdb13cedb4b5bc8ba1d992/docs/journal/2026-11-24-algorithm-level.md); [the outputs](https://github.com/lmmx/mibz2/tree/3ad28015eb83951d79fdb13cedb4b5bc8ba1d992/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/analysis)).
The lower bound, the busiest of the scanner, the workers and the reader, is tight below the knee,
with the wall within 1.4% of it up to 6 workers, and loosest at the knee itself, 4 to 5% at 8 to 11
workers, where dependencies, starvation and blocking cost most. Issuing blocks is smooth, but taking
them is bursty: when a slow block at the head of the line finishes, a run of ready blocks is taken
at once, the signature of putting completed blocks back in order. These describe one corpus's
traces, not guarantees.

## 9. Stage 2: which activity controls the wall

Stage 1 inferred the switch at the knee from busy times; stage 2 tests it by intervention, in two
parts: the first unpinned ([journal §74](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md); [its report](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2/report.txt)), the second
with the scanner pinned to a core of its own ([journal §78](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md); [its report](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2b/report.txt)).
Each activity is slowed by 20% in turn, and the share of that slowing which reaches the wall, its
elasticity, says whether it controls the decode at that worker count: near 1 for the bottleneck,
near 0 for an activity with slack behind it.

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

- **Below the knee, the workers control it.** At 6 workers the two parts of construction (0.14
  and 0.15) and emission (0.37) control the wall, while the scanner's Huffman decode does not
  (−0.01).
- **At and above the knee, the scanner does.** At 8 and 10 workers only the scanner's Huffman
  decode controls the wall: 0.53 and 0.74 in the first part, and 0.795 [0.681, 0.908] and 0.577
  [0.420, 0.734] in the pinned second, whose intervals are two to four times narrower. Nothing in
  the workers resolves. This is stage 1's switch at 8, now shown by intervening rather than
  inferred.
- **The wall is a maximum, not a sum.** In the first part, the screen's coefficients, composed as
  a sum of logarithms, predicted an arm kept out of every fit, the Huffman decode and emission both
  slowed by 10%: 1.053 against 1.052 observed at 8 workers, each inside its interval. An earlier
  version of this post called that the programme's first validated predictor. The second part's
  narrower intervals refute it: the sum predicts 1.101 at 8 workers against 1.069 observed, and
  1.034 at 10 against 1.063, both outside, and the first part's pass came from its wide intervals.
  The larger of the two coefficients alone fits all four tests, and the surface at 8 workers, both
  stations slowed together, has a roundness of 0.500 [0.454, 0.548], the value of a sharp maximum.
  Since that model was chosen after the failure, it is registered as the predictor for the next
  held-out test, not counted as validated ([§11's distinction](#11-the-dynamical-layer)).
- **A faster scanner barely helps at 8.** Slowing the scanner at 8 workers costs 0.7 to 1.0 of
  each unit of slowing, but speeding it gains only 0.13 to 0.18, because the workers are close
  behind and the bottleneck moves to them at once. The wall also failed the registered test of
  scaling evenly with the paced activities, so these figures are descriptive rather than formal.
- **The queue shows which side binds.** Slowing the scanner by half over two hundred blocks
  empties the job queue at 8 workers, its mean depth falling from 4.4 to 0.3; slowing emission
  fills it instead, from 3.9 to 8.9. Either way the decoder recovers within 2 to 4 ms.
- **Where threads sit matters more than the gates.** Widening the window or the byte cap changes
  nothing beyond noise. Putting workers on the two hardware threads of shared cores, rather than
  on separate cores, costs 48% at 8 workers; a memory stream running alongside slows the wall by
  31 to 33%, and a pointer chase by 110 to 113%. Pinning the scanner to a quiet core alone runs at
  0.960 of the unpinned wall at 8 workers. With ten cores, 8 workers plus the scanner, reader and feeder already
  make eleven threads, so some sharing is unavoidable at 8 and certain at 10.

The first part priced the scanner's copy fix optimistically: cutting the Huffman decode's work by
8%, about the copies' share, implies a wall of 0.957 at 8 workers and 0.940 at 10. The second part
makes that an upper bound twice over, since a faster scanner gains little at 8 and more workers,
where it would gain, share more cores. Every scanner speed-up is therefore priced at the count it
will run at, with the worker count retuned, and the paired comparison of the copy fix decides it.

<p class="il-rubric-head">Of the queue as a reservoir</p>

The pulses read most simply in a fluid approximation, which treats a queue as a reservoir whose
level q changes at the rate work flows in less the rate it flows out, dq/dt = λ_in − μ_out. Fluid
models are among the theory course's techniques ([our reading](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md)),
but none has been fitted to the decoder; here the picture is a way to read the measurements. Slow
the scanner and the inflow falls below what the workers can take, so the reservoir drains and the
workers starve. Slow emission and the outflow falls instead, so it fills.

<figure class="bf-fig" id="reservoir">
<p class="bf-title">Which side of the queue binds</p>
<svg viewBox="0 0 720 344" role="img" aria-label="Two reservoirs, the job queue at 8 workers. Slowing the scanner by half drains it: mean depth 4.4 blocks before, 0.3 during, and the workers starve, the rate falling to 0.69 of the undosed run's. Slowing emission by half fills it: 3.9 before, 8.9 during, the rate 0.82 of the undosed run's."><text class="bf-t-strong" x="180" y="22" text-anchor="middle" style="font-size:15px">Slow the scanner</text><text class="bf-tm" x="180" y="40" text-anchor="middle" style="font-size:12px">slowed: the scanner, +50%</text><line x1="180" x2="180" y1="52" y2="84" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:4 3"/><path d="M174,83 l6,9 l6,-9 z" style="fill:var(--bf-amber)"/><text class="bf-tm" x="194" y="76.0" style="font-size:12px">in, from the scanner (slowed)</text><line x1="180" x2="180" y1="236" y2="268" style="stroke:var(--bf-frame);stroke-width:3"/><path d="M174,267 l6,9 l6,-9 z" style="fill:var(--bf-frame)"/><text class="bf-tm" x="194" y="260.0" style="font-size:12px">out, to the workers</text><rect x="102" y="227.9" width="156" height="4.1" style="fill:var(--bf-blue);opacity:0.55"/><path d="M100,96 L100,232 L260,232 L260,96" style="fill:none;stroke:var(--bf-ink);stroke-width:2"/><line x1="100" x2="260" y1="172.2" y2="172.2" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-tm" x="94" y="176.2" text-anchor="end" style="font-size:11.5px">before</text><text class="bf-t-strong" x="180" y="298" text-anchor="middle" style="font-size:16px">4.4 → 0.3 blocks</text><text class="bf-tm" x="180" y="318" text-anchor="middle" style="font-size:12px">mean queue depth: the workers are starved</text><text class="bf-tm" x="180" y="334" text-anchor="middle" style="font-size:12px">rate 0.69 of the undosed run’s</text><text class="bf-t-strong" x="540" y="22" text-anchor="middle" style="font-size:15px">Slow emission</text><text class="bf-tm" x="540" y="40" text-anchor="middle" style="font-size:12px">slowed: the workers' emission, +50%</text><line x1="540" x2="540" y1="52" y2="84" style="stroke:var(--bf-frame);stroke-width:3"/><path d="M534,83 l6,9 l6,-9 z" style="fill:var(--bf-frame)"/><text class="bf-tm" x="554" y="76.0" style="font-size:12px">in, from the scanner</text><line x1="540" x2="540" y1="236" y2="268" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:4 3"/><path d="M534,267 l6,9 l6,-9 z" style="fill:var(--bf-amber)"/><text class="bf-tm" x="554" y="260.0" style="font-size:12px">out, to the workers (slowed)</text><rect x="462" y="111.0" width="156" height="121.0" style="fill:var(--bf-blue);opacity:0.55"/><path d="M460,96 L460,232 L620,232 L620,96" style="fill:none;stroke:var(--bf-ink);stroke-width:2"/><line x1="460" x2="620" y1="179.0" y2="179.0" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-tm" x="454" y="183.0" text-anchor="end" style="font-size:11.5px">before</text><text class="bf-t-strong" x="540" y="298" text-anchor="middle" style="font-size:16px">3.9 → 8.9 blocks</text><text class="bf-tm" x="540" y="318" text-anchor="middle" style="font-size:12px">mean queue depth: work piles up</text><text class="bf-tm" x="540" y="334" text-anchor="middle" style="font-size:12px">rate 0.82 of the undosed run’s</text></svg>
<figcaption>Measured, stage 2a at 8 workers: each pulse slows one activity by 50% over
blocks 600 to 800. The tanks show the job queue's mean depth before (dashed) and during (filled); the rate is the
rate during the pulse over the undosed run's at the same places. The undosed run's own depth over
those places rises from 4.5 to 5.6, so the scanner's drain is the larger departure. Both recover
within 2 to 4 ms.</figcaption>
</figure>

<p class="il-rubric-head">Of noise that came from the clock</p>

Part of the noise in these runs came from the machine, not the decoder
([journal §81](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level.md)). The Pathfinder found the processor's clock at two levels, 4.91 GHz and a
low level of 4.09 to 4.56 GHz, with about a quarter of decodes caught low. A direct test traced it to
the idle pause each recipe takes before a timed decode: after 0.2 s idle a decode runs at 2.96 GHz
and takes 45% longer, and even after the standard 1 s pause, 40% of decodes run low. Spinning every
CPU for 100 ms before each timed decode removes the low state and cuts the spread of the log wall
from 4.4% to 1.1%, so a test needs about sixteen times fewer rounds for the same interval. Every
recipe at or above the knee now warms the machine this way. The finding is also a question for
users: a program that decodes after a short pause may meet the slow regime, and whether a decoder
should respond to it is open until more machines are measured.

## 10. Designing for a purpose

A question about method came next: whether the decoder could be studied the way systems biology
studies a regulatory network, with Uri Alon's network motifs as the obvious model. The answer that
held up was not his catalogue of recurring motifs but the line of work that asks which structures
can perform a given function at all ([the lead](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md); [the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-alon-motifs.md)).
[Ma and colleagues](https://doi.org/10.1016/j.cell.2009.06.013){ .il-ref } enumerated every
three-node network topology to find the few able to achieve biochemical adaptation, and
[Araujo and Liotta](https://doi.org/10.1038/s41467-018-04151-6){ .il-ref } later gave the
solution space a basis, showing that every network capable of robust perfect adaptation is built
from two kinds of module.
[Khammash](https://doi.org/10.1016/j.cels.2021.05.020){ .il-ref } states the programme in five
steps: specify the function, derive necessary conditions, characterise the solution space, check
realisability, and establish optimality ([an explanation](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-khammash.md)). Ours maps onto it closely:

| Khammash's step | Ours, already | What is missing |
|---|---|---|
| The function | the referee | |
| Necessary conditions | G1, and records 118 and 119 | a necessity statement in positive form |
| The solution space | | an enumeration, and then a basis |
| Realisability | the four-arm and profile ceilings | |
| Optimality | the performance work | |

The proposal that follows is to enumerate the referee's own class of decoders (small cursor
machines with a fixed number of passes that invert every short string over a tiny alphabet),
reduce the list by symmetry, cluster the survivors, look for the condition that separates them, and
use a SAT solver to prove "no survivor" within the class ([journal §65](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). Two ideas from the
same reading were adopted as method: the same function can be realised by different structures,
which is why the referee counts structure; and a representation change is a search for the hidden
coordinate in which the work cancels.

## 11. The dynamical layer

The reservoir is the first step from queues towards dynamics, and the theory course's fluid models
lead to it directly. The decoder is "digital on analogue": discrete events, a block admitted or a buffer released, on top
of continuous quantities such as occupancy and time. The language for that is the theory of hybrid
dynamical systems ([the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-dynamics.md); [journal §67](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)), and Goebel, Sanfelice and
Teel's book on it was read in full, with notes in our own words. Its plainest product is a list of
five levers. Any reorganisation of the decoder changes one of five things
([the levers](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md)):

1. how fast work runs (the flow map);
2. where work may proceed (the flow set);
3. when events may happen (the jump set);
4. what events do (the jump map);
5. what is represented (the state space).

<p class="il-rubric-head">Of a bound that was not a certificate</p>

One reading of the book went too far, and an independent review caught it before it guided any
experiment ([the review](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md); [journal §70](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). The bottleneck maximum, the largest of the
scanner's, the workers' and the reader's demands, is only a lower bound on the time a decode takes,
because it ignores dependencies, starvation and blocking. One block that takes a 1 s scan, a 1 s
decode and a 1 s delivery in series has a maximum of 1 s and takes 3. The notes now keep three
quantities apart: lower bounds, used to prune; predictors, used to rank, and validated on held-out
runs rather than by replaying a run with its own durations; and proved upper bounds, for which
backward induction over a finite, acyclic state graph is the concrete route.

## 12. How the work is done

The method is written down so that it can be reused
([the protocol](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/process/protocol.md)). It starts from a measured surprise rather than a technique, poses the
problem and its referee before any solution, and prefers a model that predicts from little data and
is checked on data it did not see. Sources are read in full and saved. Several readers and
reviewers answer independently before comparing, their answers are recorded verbatim, and decisions
are kept in separate files with their reasons. Wrong claims are corrected in place and in public,
naming who found the error, as the worker-count error and the lower bound above were. Every run
registers its decision rule before its data, changes are priced jointly because a relieved
bottleneck moves, and when it moves, the ideas it may have masked are tested again.

## 13. What runs next

Five runs are planned, in order ([the programme of experiments](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/briefs/experiment-programme.md)): the stage 1 baseline, now
done; the scanner's copy fix, as a paired comparison at 8, 9 and 10 workers; stage 2, which slows
each activity in turn to say which controls the wall at which count; the recalibration of the
worker count, including 7 against 8 and the kernel chosen below 8; and the joint frontier of scanner
and worker improvements together. Both parts of stage 2 are in ([§9](#9-stage-2-which-activity-controls-the-wall)), and the tests they priced, the copy fix and the trim to 8 workers, are being asked again with the clock warmed first. Each cost
it finds controlling will be classed as engineering, a matter of how the work is done, or as a
candidate for mathematics, a matter of what work is done.

Seen whole, the work moves through three levels. Algorithm analysis asks how much work a
computation needs. Queueing theory asks which of that work sets the speed of the whole. The
structural question, what organisation of the work could remove it or its dependencies, is where
the dynamical systems and the mathematics come in. Queueing theory is the bridge between the first
and the third: it says where new mathematics would pay.

---

**Sources:**

- The measurements: [the scanner profile](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv) (journal §57) and [stage 1's report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt) (journal §66).
- The readings: [the summary](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md), [the first reading with its corrections](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md), [the outside reading](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md) and [the second agent's](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md); journal §§61–62 and §69.
- The design thread: [structure and function](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md), with [the outside answer on motifs](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-alon-motifs.md) and [on Khammash](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-khammash.md); journal §§64–65.
- The dynamical layer: [the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-dynamics.md), [the levers](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md) and [the review that corrected them](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md); journal §§67–70.
- Queueing theory: [the three courses, as read](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md) (journal §§53–54); Jain, *The Art of Computer Systems Performance Analysis* (1991); Harchol-Balter, *Performance Modeling and Design of Computer Systems* (2013); the papers linked in §§2 and 5.
- [The discipline question](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md) (journal §56), [the protocol](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/process/protocol.md) and [the programme of experiments](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/briefs/experiment-programme.md).
- The fourth reading: [record 129](https://github.com/lmmx/mibz2/blob/7f18ccf719a0fd7eb863f8aa519a4fb8a8f2295b/docs/journal/monodromy/2026-11-25-course-mining-and-analysis-plan.md) and [record 130](https://github.com/lmmx/mibz2/blob/9c3690f299574ea5296e189c8f3d74e7dcfb19af/docs/journal/monodromy/2026-11-25-design-space-and-identification.md); stage 1 by program, [journal §71](https://github.com/lmmx/mibz2/blob/3ad28015eb83951d79fdb13cedb4b5bc8ba1d992/docs/journal/2026-11-24-algorithm-level.md).
- Stage 2: the first part's [report](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2/report.txt) and [analysis](https://github.com/lmmx/mibz2/tree/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2/analysis) (journal §74); the second's [report](https://github.com/lmmx/mibz2/blob/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2b/report.txt) and [analysis](https://github.com/lmmx/mibz2/tree/9c476fd302741471ea7eedb9b544afaf17c603c0/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage2b/analysis) (journal §78); the clock test, journal §81; the three-symbol prototype, journal §82 and [#870](https://github.com/lmmx/mibz2/pull/870).
- [The journal](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md).
