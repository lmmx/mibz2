---
title: "Beneath the queueing network: operational laws, modification analysis and product form"
image: assets/social/blog-2026-11-25-the-decoder-as-a-queueing-network-notes.png
description: "The theory under the story, from counting what waits and finding the bottleneck to pricing a change before it is built."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md) · Companion to [The decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md)
{: .post-kicker }

# Beneath the queueing network: operational laws, modification analysis and product form

The theory under the story, from counting what waits and finding the bottleneck to pricing a change before it is built.
{: .post-dek }

25 November 2026 · 16 min read · [Research in progress](#13-source-register)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

<div class="post-companion" data-role="companion-of" markdown>

**The story this companion goes beneath** tells how mibz2 came to apply queueing theory to its decoder,
and what each part of the theory did when it was tried. [The decoder as a queueing network&nbsp;→](2026-11-25-the-decoder-as-a-queueing-network.md)

</div>

This companion holds the theory and context that [the main
post](2026-11-25-the-decoder-as-a-queueing-network.md) uses without stopping to develop, so that the
post can tell the story of the mibz2 effort on its own terms. Its sections can be read in any order.
Nothing new was measured for either page. Where a statement rests on the course material, the section
says which part of it; where it rests only on an outside reader's account, it says so; and the
repository's own notes ([the journal](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)) are linked at the commit the post was written from.

## 1. What was read

The theory came from two teachers' courses: Raj Jain's [course on performance
analysis](https://www.cs.wustl.edu/~jain/iucee/index.html), which follows his *The Art of Computer
Systems Performance Analysis* (1991), and Mor Harchol-Balter's
[15-857](https://www.cs.cmu.edu/~harchol/Perfclass/class.html) at Carnegie Mellon with its theory
companion [15-849b](https://www.cs.cmu.edu/~harchol/Perfclass/classtheory.html), which follow her
*Performance Modeling and Design of Computer Systems* (2013) ([the reading](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md);
[the saved sources](https://github.com/lmmx/mibz2/tree/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/sources)). Read in full: the three course pages, and six of Jain's
slide decks, on common mistakes and the systematic approach, experimental design, fractional
factorials, the analysis of simulation results, the operational laws, and mean-value analysis. Read as
outlines: the topic lists of every Jain module, the textbook's table of contents, the 2025 schedule of
15-857 with its homework and optional readings, and the 2003 theory course's topic and book lists. Not
read: the lecture videos, the textbook's chapters themselves, the other decks and the 2025 handouts.
Chapter numbers below are those the saved sources give. An outside model also answered the same
questions independently and is recorded verbatim, with its claims marked where they are its alone
([its answer](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-courses.md); [journal §54](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 2. Vocabulary and shapes

A **job** is a unit of work whose progress is followed: here, a block of the compressed file. A
**station**, or service centre, is a resource that performs work for jobs: a server, or a group of
identical servers sharing one queue. A **queue** holds jobs that cannot yet be served. A **discipline**
decides which waiting job is served next: first come first served, by priority, by size, or shared.

Per job, a station has a **service time** $S$ (one visit's work), a **visit ratio** $V$ (visits per job)
and a **demand** $D = VS$. Over an interval, the system has a **throughput** $X$ (jobs completed a
second), each station a utilisation $U$ (fraction of time busy), a mean population $N$ and a mean
**residence** $R$ (time from arrival to departure, waiting included). Residence is not service time:
the gap between them is the waiting, and confusing the two is the commonest error in applying the laws
below.

<figure class="bf-fig" id="shapes">
<p class="bf-title">The shapes queues come in</p>
<svg viewBox="0 0 720 234" role="img" aria-label="Five shapes of queueing system: a single server, a parallel group of servers sharing a queue, a tandem of stages in series, a fork and join, and a feedback loop that returns permits. The decoder combines a tandem, a parallel group and feedback."><text class="bf-t-strong" x="72" y="24" text-anchor="middle" style="font-size:13px">single server</text><text class="bf-tm" x="72" y="42" text-anchor="middle" style="font-size:11.5px">one queue, one processor</text><text class="bf-t-strong" x="216" y="24" text-anchor="middle" style="font-size:13px">parallel group</text><text class="bf-tm" x="216" y="42" text-anchor="middle" style="font-size:11.5px">one queue, k servers</text><text class="bf-t-strong" x="360" y="24" text-anchor="middle" style="font-size:13px">tandem</text><text class="bf-tm" x="360" y="42" text-anchor="middle" style="font-size:11.5px">stages in series</text><text class="bf-t-strong" x="504" y="24" text-anchor="middle" style="font-size:13px">fork and join</text><text class="bf-tm" x="504" y="42" text-anchor="middle" style="font-size:11.5px">parts that must all finish</text><text class="bf-t-strong" x="648" y="24" text-anchor="middle" style="font-size:13px">feedback</text><text class="bf-tm" x="648" y="42" text-anchor="middle" style="font-size:11.5px">progress returns permits</text><rect x="26" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="39" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="52" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><line x1="64" y1="110" x2="82" y2="110" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="82.0,110.0 73.0,105.0 73.0,115.0" style="fill:var(--bf-frame)"/><circle cx="100" cy="110" r="16" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><rect x="160" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="173" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="186" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><line x1="198" y1="110" x2="222" y2="80" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="222.0,80.0 212.5,83.9 220.3,90.2" style="fill:var(--bf-frame)"/><line x1="198" y1="110" x2="222" y2="110" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="222.0,110.0 213.0,105.0 213.0,115.0" style="fill:var(--bf-frame)"/><line x1="198" y1="110" x2="222" y2="140" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="222.0,140.0 220.3,129.8 212.5,136.1" style="fill:var(--bf-frame)"/><circle cx="240" cy="80" r="13" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><circle cx="240" cy="110" r="13" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><circle cx="240" cy="140" r="13" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><rect x="300" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="313" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><line x1="326" y1="110" x2="338" y2="110" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="338.0,110.0 329.0,105.0 329.0,115.0" style="fill:var(--bf-frame)"/><circle cx="352" cy="110" r="13" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><rect x="370" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="383" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><line x1="396" y1="110" x2="404" y2="110" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="404.0,110.0 395.0,105.0 395.0,115.0" style="fill:var(--bf-frame)"/><circle cx="418" cy="110" r="13" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><line x1="452" y1="110" x2="476" y2="84" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="476.0,84.0 466.2,87.2 473.6,94.0" style="fill:var(--bf-frame)"/><line x1="452" y1="110" x2="476" y2="136" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="476.0,136.0 473.6,126.0 466.2,132.8" style="fill:var(--bf-frame)"/><circle cx="490" cy="80" r="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><circle cx="490" cy="140" r="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><line x1="504" y1="84" x2="528" y2="106" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="528.0,106.0 524.7,96.2 518.0,103.6" style="fill:var(--bf-frame)"/><line x1="504" y1="136" x2="528" y2="114" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="528.0,114.0 518.0,116.4 524.7,123.8" style="fill:var(--bf-frame)"/><rect x="530" y="98" width="26" height="24" rx="4" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="543" y="114" text-anchor="middle" style="font-size:12px">J</text><text class="bf-t-strong" x="446" y="114" text-anchor="end" style="font-size:12px">F</text><rect x="600" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><rect x="613" y="102" width="10" height="16" rx="2" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><line x1="626" y1="110" x2="640" y2="110" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="640.0,110.0 631.0,105.0 631.0,115.0" style="fill:var(--bf-frame)"/><circle cx="656" cy="110" r="14" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><path d="M672,110 L700,110 L700,154 L596,154 L596,122" style="fill:none;stroke:var(--bf-green);stroke-width:1.6;stroke-dasharray:5 4"/><polygon points="596,120 591,129 601,129" style="fill:var(--bf-green)"/><text class="bf-tm" x="648" y="172" text-anchor="middle" style="font-size:11px;fill:var(--bf-green)">a permit returns</text><text class="bf-tm" x="360.0" y="204" text-anchor="middle" style="font-size:12.5px">The decoder uses three of them: a tandem (scanner, then workers), a parallel group (the workers)</text><text class="bf-tm" x="360.0" y="222" text-anchor="middle" style="font-size:12.5px">and feedback (the in-order reader returns admission). Each block goes to one worker, so it is not a fork and join.</text></svg>
<figcaption>Schematic. Circles are servers and small boxes queued jobs.</figcaption>
</figure>

Networks are **open** when jobs arrive from outside at some rate and leave when done, **closed** when
a fixed population circulates, and **mixed** when they have both. Closed networks come in two kinds
(Harchol-Balter, §2.6): **interactive**, where each job "thinks" for a time $Z$ between visits, as a
user at a terminal does, and **batch**, where $Z = 0$ and a finished job is replaced at once. The decoder
is neither exactly. Its admission window is a ceiling, not a population, its stream is finite, and its
in-order reader and byte cap make admission depend on its state: a finite pipeline with
feedback-controlled admission, for which a closed network of permits is a surrogate
([the summary](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md); [the design review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-design-review.md)).

## 3. The operational laws

The operational laws relate quantities measured over an interval of observation, and assume nothing
about the distributions of arrivals or service times (Jain, ch. 33; Harchol-Balter, ch. 6–7). Each
holds exactly for any interval in which the counts balance.

| Law | Statement | What it says |
|---|---|---|
| Utilisation | $U_i = X D_i$ | a station's busy fraction is throughput times its demand |
| Little | $N = X R$ | mean population is throughput times mean residence |
| Forced flow | $X_i = V_i X$ | each station's throughput is the system's times its visit ratio |
| General response time | $R = \sum_i V_i R_i$ | a job's residence is the sum over stations of visits times residence per visit |
| Interactive response time | $R = N/X - Z$ | in a closed interactive system, residence follows from population, throughput and think time |

**Little's law on a finite trace.** The law is usually stated for long-run averages
([Little, 1961](https://doi.org/10.1287/opre.9.3.383)), but on a trace that starts and ends empty it
is an identity: the area under the number-present curve equals the sum of the jobs' residences, and
dividing both by the trace's length gives $N = XR$. Sample-path analysis, on the theory course's book
list, makes such statements for deterministic traces. Stage 1's tracing checks exactly this form, for
each of seven buffers, as an integer equality ([#841](https://github.com/lmmx/mibz2/pull/841)); the main
post draws it ([Little's law as two counts of one area](2026-11-25-the-decoder-as-a-queueing-network.md#little)).

**Bounds.** Since no $U_i$ exceeds 1, the utilisation law gives $X \le 1/D_{\max}$, where $D_{\max}$ is the largest
demand at a single-server station, or $c_i/D_i$ for a station of $c_i$ servers: the **bottleneck law**
(Jain, ch. 33). For a closed interactive system of population $N$, the **asymptotic bounds** are

$$
\begin{gathered}
X(N) \le \min\!\left(\frac{N}{D + Z},\ \frac{1}{D_{\max}}\right), \\
R(N) \ge \max\left(D,\ N D_{\max} - Z\right),
\end{gathered}
$$

with $D$ the total demand. The first term in each is the light-load regime, in which a job rarely
queues; the second is the bottleneck regime. They cross at the **population knee**
$N^* = (D + Z)/D_{\max}$ (Harchol-Balter, §7.2), beyond which some job is certainly queueing. $N^*$ is a
knee in the number of jobs, not in the number of servers. The decoder's knee $k^* = w/s$ is a crossing
of two capacities as servers are added, and the decoder ties population to servers through $W = 4k$,
but the two are different variables, which is why the main post derives its knee from the bottleneck
law rather than from $N^*$. **Balanced job bounds** (Jain, ch. 34) tighten the asymptotic bounds by
comparing the system with one whose demands are all equal.

**Not a completion time.** For $B$ blocks, $\max(Bs,\ Bw/k)$ bounds the wall from below but ignores
start-up, drain, uneven assignment and in-order delivery. One block with a 1 s scan, a 1 s decode and
a 1 s delivery done in series has a largest demand of 1 s and takes 3 s ([journal §70](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 4. Modification analysis

Harchol-Balter's chapter 7, "Modification Analysis: What-if for Closed Systems", asks what a change to
one station's demand, or to its number of servers, does to the bounds, before the change is built.
Jain's deck on the operational laws sets it as homework: make one component twice as fast or twice
as slow, and say which becomes the bottleneck. The book's opening questions are of the same kind,
among them whether one machine of speed $s$ beats $n$ machines each of speed $s/n$, and whether doubling
both the arrival and service rates leaves the mean response time unchanged. The 2025 course gave a
session to modification analysis applied to a real caching system; its handout was not read here.

The general lesson is the main post's: with stages that overlap, the wall follows the larger demand,
so a cut at a non-binding station does nothing until the binding one is relieved, and a cut at the
binding station is worth only the distance to the next. Improvements are therefore priced in
combination, and a masked improvement is a conditional opportunity rather than a worthless one. The
ideal-parts matrix (section 10) measures exactly these conditional values.

## 5. Load-dependent service

Jain distinguishes types of service centre: fixed-capacity centres, whose rate is constant; delay
centres, with no queueing; and load-dependent centres, whose rate depends on how many jobs are
present (ch. 32). A pool of $k$ identical servers is the simplest load-dependent centre, its rate rising
as $\min(n, k)/w$ with $n$ jobs present, and that is how the surrogate of section 8 models the workers.
Contention adds a second kind: on real cores the rate per server falls as more servers run, because
they share caches, memory bandwidth, execution units and clock frequency. Writing the decoder's bound
as $X(k) \le \min\left(1/s(k),\ k/w(k)\right)$ carries both. The measurements of each, from the profile, stage 1 and
stage 2b, are in the main post's section 8.

## 6. Variability, and open against closed

Means are not enough wherever jobs queue. In the M/G/1 queue, with Poisson arrivals and a general
service time $S$ at arrival rate $\lambda$ and utilisation $\rho$, the mean wait is

$$
\begin{aligned}
\mathbb{E}[W_q] &= \frac{\lambda\, \mathbb{E}[S^2]}{2(1 - \rho)} \\
&= \frac{\rho}{1 - \rho} \cdot \frac{1 + C_S^2}{2} \cdot \mathbb{E}[S],
\end{aligned}
$$

the Pollaczek–Khinchine formula (Harchol-Balter, ch. 23). The wait depends on the second moment of
service, through the squared coefficient of variation $C_S^2$. An outside reader's illustration: at 80%
utilisation and a 10 ms mean, the mean wait is 20, 40 or 200 ms as $C_S^2$ is 0, 1 or 9
([journal §54](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). The same chapter's inspection paradox explains why: a random arrival is more
likely to land during a long service than a short one. The formula describes an open single queue,
not the decoder; it motivates keeping distributions of service time for each activity, as the
characterisation was amended to report.

The corpus behind the decoder's measurements has two classes of block. 792 of its 1,724 blocks are
under 10 kbit and hold 0.1% of the bits, and they are exactly the blocks that skip the main inversion
route ([journal §§60, 66](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)). Across the mixture $C^2$ of block size is about 0.98; among the real
blocks alone it is about 0.07, and their sizes are correlated along the stream, lag-1 $r = 0.74$
([the opportunities](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md)). Which matters depends on the question: the mixture's for head-of-line
effects, the real blocks' for the workers' service.

**Open against closed.** A paper on the 2025 reading list shows that scheduling policies behave very
differently in closed systems than in open ones, because in a closed system a slow job also delays the
arrival of the next ([Schroeder, Wierman and Harchol-Balter, NSDI 2006](https://www.usenix.org/conference/nsdi-06/open-versus-closed-cautionary-tale)).
For the decoder that argues for modelling the admission feedback faithfully. It is not a licence to
treat the decoder as a fixed-population closed network, nor to declare scheduling unimportant
([the outside reading](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md)).

## 7. The disciplines considered

The operational laws bound what any discipline can do: none raises the ceiling the demands set, and
the decoder runs at 0.945 to 0.998 of its busiest station's limit, so reordering alone could recover
at most about 5%, or 3% on the plateau, while demands stay fixed. A discipline that changes the demands,
by improving locality or keeping memory-heavy phases apart, is not bound by that figure: the
twenty-worker decode ran at 0.972 of its bound and still finished 17% sooner with eight workers
([journal §57](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md); [the discipline lead](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md); [the outside reading](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md)).

| Technique | Known from | What it would change | What became of it |
|---|---|---|---|
| Size-based scheduling: longest first (LPT), shortest first (SJF, SRPT), assignment by size (SITA) | makespan scheduling; Harchol-Balter, ch. 20, 24, 28–33 | big blocks stop becoming the straggler or the head of the line | not supported yet: worker service per block was never measured |
| Helpers for the oldest unfinished block | priority and least-attained scheduling; work stealing | shortens the block holding up in-order output | open: the team machinery exists, and the trigger is the question |
| Admission by work rather than by count | load-dependent populations; byte-based flow control | a few huge blocks can no longer fill the window | open |
| Relaxed resequencing | resequencing buffers | finished blocks stop holding admission credit while they wait | limited by the output contract |
| Redundancy and power-of-d choices | the Redundancy-d work | helps when servers' delays are independent and variable | ruled out: here they are neither, and duplicating costs the scarce resource |
| Heavy-tail and matrix-analytic models | Harchol-Balter, ch. 20–21 | exact models for heavy-tailed service | not needed on this corpus; phase-type fits deferred, as simulation is cheaper |
| More than one scanner | cross-block parallel parsing, as lbzip2 does | turns the serial station into a parallel group | the only row that raises the ceiling: a change of topology |

## 8. Product form and mean-value analysis

**The lineage.** Jackson showed that open networks of exponential servers with probabilistic routing
have a product-form equilibrium: the joint distribution of queue lengths is a product of one factor
per station, as if each were an independent queue
([1957](https://doi.org/10.1287/opre.5.4.518)). Gordon and Newell gave the closed analogue, with a
normalising constant that couples the stations through the fixed population
([1967](https://doi.org/10.1287/opre.15.2.254)). Baskett, Chandy, Muntz and Palacios extended product
form to open, closed and mixed networks with several job classes and four kinds of station: first
come first served with exponential service, and processor sharing, last come first served with
pre-emption, and infinite servers with general service distributions; the paper also admits some load
dependence ([BCMP, 1975](https://doi.org/10.1145/321879.321887); Harchol-Balter, ch. 22; Jain, ch. 32).
Product form makes a network solvable; it says nothing about whether the network is a good design.

**Mean-value analysis.** For a closed product-form network, Reiser and Lavenberg's mean-value analysis
computes the means without the normalising constant
([1980](https://doi.org/10.1145/322186.322195); Jain, ch. 34; Harchol-Balter, §19.3). Its key is the
arrival theorem: a job arriving at a station sees the network in its equilibrium with one job fewer.
Starting from an empty network and adding one job at a time, for single-server stations:

1. residence per visit: $R_i(n) = S_i\left(1 + Q_i(n - 1)\right)$, or $R_i(n) = S_i$ at a delay station;
2. throughput: $X(n) = n \big/ \left(Z + \sum_i V_i R_i(n)\right)$;
3. queue lengths: $Q_i(n) = X(n)\, V_i R_i(n)$.

Load-dependent stations need the marginal probability of each queue length rather than the mean
alone, a standard extension of the same recursion.

**The surrogate.** One reader computed exact mean-value analysis for a two-station closed surrogate
of the decoder: a scanner of 107.9 µs a block, a load-dependent pool of workers taking 815 µs each
with rate $\min(n, k)/w$, a population of $N = 4k$, and no fitted constant. Converted to the wall for the
corpus's 1,724 blocks, it gives 0.4683, 0.2811, 0.2035 and 0.1887 s at 3, 5, 7 and 8 workers, against
0.4693, 0.2840, 0.2069 and 0.1911 s measured: within 1 to 4 ms ([the review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md)).
No solver script for it is in the repository; the figures are the review's, and recomputing them from
the same parameters reproduces them to within 0.0001 s. The decoder departs from product form in
ways the surrogate cannot represent:

| Product form assumes | The decoder has |
|---|---|
| a fixed population that circulates | an admission ceiling, $W = 4k$, on a finite stream that fills and drains |
| stations that release jobs independently | in-order taking: a finished block waits for the one before it |
| service rates set by the station alone | workers that slow one another and the scanner through shared caches and cores |
| admission that does not depend on state | a byte cap on finished output that gates new scanning |
| one class of job | two classes of block, whose sizes are correlated along the stream |

None of these is a blanket disqualification of BCMP, which is broader than its reputation; what was
never established is a compatible representation of this decoder's gates, ordering and contention. So
exact mean-value analysis is applied only to a documented surrogate, with its error reported as
approximation error, and the faithful model is a trace-driven discrete-event simulation, specified
and not yet built ([journal §§59–60](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 9. Fluid approximations

A fluid model treats a queue as a reservoir whose level $q$ changes at the rate work flows in less the
rate it flows out,

$$
\frac{dq}{dt} = \lambda_{\text{in}} - \mu_{\text{out}},
$$

held at zero when the queue is empty and at the ceiling when
admission stops. It appears among the theory course's techniques. The decoder's pulse experiments
read naturally in it: slowing the scanner drains the job queue, slowing emission fills it, and each
recovers within milliseconds. No fluid model has been fitted to the decoder. The same picture, flows
between discrete events, is the starting point of the hybrid-systems reading in section 12.

## 10. Experimental design, and the ideal-parts matrix

Jain's chapters 16–23 give the vocabulary. A **factor** is a variable the experiment sets, at
**levels**; a **response** is what is measured. An **interaction** is present when one factor's
effect depends on another's level, and Jain lists ignoring interactions, and varying one factor at a
time, among the common mistakes. Run-time ratios suit a **multiplicative model**, analysed on
logarithms (ch. 18).

**The $2^2$ design and its sign table.** With two factors at two levels, S and E, each real (−) or ideal
(+), the four arms give the effects on log wall $y$ as

$$
\begin{aligned}
q_S &= \tfrac14\left(-y_0 + y_S - y_E + y_{SE}\right), \\
q_E &= \tfrac14\left(-y_0 - y_S + y_E + y_{SE}\right), \\
q_{SE} &= \tfrac14\left(y_0 - y_S - y_E + y_{SE}\right),
\end{aligned}
$$

and the **allocation of variation** splits the total sum of squares among S, E, SE and the residual
(ch. 17). The four-arm oracle test and slice A of the ideal-parts matrix are both this design. In
slice A at 8 workers $q_{SE}$ is −0.028 [−0.038, −0.018], and the allocation gives 37% to S, 40% to E and
13% to their interaction ([slice A](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/slice-a/README.md)).

**Fractional factorials.** With six parts there are $2^6 = 64$ arms. A half fraction keeps 32, chosen by
a **defining relation**: $I = SECMRD$ keeps the arms whose six signs multiply to +1, that is, those with
an even number of ideal parts. Each effect is then **confounded** with its complement in the relation,
$S$ with $ECMRD$, $SE$ with $CMRD$, and so on, and the design's **resolution** is the length of the shortest
word in the relation, here VI: each main effect is confounded only with a five-part interaction, and
each two-part interaction only with a four-part one (ch. 19).

The registered matrix needed more than that fraction. A part's room is defined as that part alone
ideal over all real, and its share of the floor as all but that part ideal over all ideal, and both
are arms with an odd number of ideal parts, which the fraction lacks. Predicting them from the
fraction's model would assume every interaction of three or more parts is zero, and slice A's two-part
interactions make that unsafe. So the matrix runs the 32 fraction arms plus the 6 singles and the 6
five-part arms, 44 at each count: the sign table on the 32 gives the effects, and room, floor share
and masking are read directly from the measured odd arms, which are never fitted into the effects
([clarification 8](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md); [the design](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md)).

**The full matrix.** The 44 arms ran at 1, 6, 8 and 10 workers ([the results](https://github.com/lmmx/mibz2/blob/caf59b0268eec8db2f7e7ccb569de745c4e08d3f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md)). On log wall,
the allocation of variation over the 32 fraction arms gives construction 80% at one worker and 46% at
six, the scanner 46% at eight and 53% at ten, and the two-part interactions 18–22% from six workers
up; the three-part interactions, aliased in pairs, take 2–6%. The largest interaction is the scanner
with construction, $q_{SC} = -0.110$ [−0.112, −0.108] at six workers and −0.071 at eight, where $4q$, the
difference of differences, is −0.44 and −0.28. No part earns the MASKED label, because each
masked-looking part keeps more than ±2% of room alone, which leaves it UNRESOLVED rather than NO ROOM;
in substance, construction at eight and ten and the scanner at six and eight are masked behind each
other.

**What ideal means.** An ideal part is an oracle: its output is prepared outside the timed decode and
supplied. That brings its own costs, such as the handover of supplied codes, 286 to 418 ms summed over
workers in slice A, and in the full matrix enough that the all-ideal floor is mostly handover; upstream stages still run when only a later one is ideal; and the results measure
room within an arrangement, not what a real algorithm will reach.

## 11. Simulation: verification and validation

Jain's chapter 25 separates two questions about a simulation. **Verification** asks whether the
program implements the intended model, by modular design, traces, continuity tests and degenerate
cases. **Validation** asks whether the model represents the system, by expert intuition, measurements
of the real system and theoretical results. A trace-driven simulation of the decoder would be verified
against exact small cases and the operational identities, and validated against the twenty measured
walls and held-out arms. Because the decoder's stream is finite, the start-up and drain are part of the
answer rather than a transient to remove, which makes its simulation a terminating one.

## 12. The design thread

Two readings give the search for a better organisation its language.

**Structure and function.** Systems biology's useful question here is not Alon's catalogue of
recurring network motifs but which structures can perform a given function at all
([the lead](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md)).
[Ma and colleagues](https://doi.org/10.1016/j.cell.2009.06.013){ .il-ref } enumerated every
three-node network to find the few able to achieve adaptation;
[Araujo and Liotta](https://doi.org/10.1038/s41467-018-04151-6){ .il-ref } gave that solution space a
basis; and [Khammash](https://doi.org/10.1016/j.cels.2021.05.020){ .il-ref } states the programme in
five steps.

| Khammash's step | mibz2's, already | What is missing |
|---|---|---|
| The function | the referee | |
| Necessary conditions | G1, and records 118 and 119 | a necessity statement in positive form |
| The solution space | | an enumeration, and then a basis |
| Realisability | the four-arm and profile ceilings | |
| Optimality | the performance work | |

The proposal is to enumerate the referee's own class of decoders, small cursor machines with a fixed
number of passes, reduce the list by symmetry, and use a SAT solver to prove "no survivor" within the
class ([journal §65](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

**Hybrid dynamical systems.** Goebel, Sanfelice and Teel's theory describes a system that flows
continuously between discrete events, which is the decoder: occupancy and time flow, blocks are
admitted and released ([the levers](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md)). Any
reorganisation changes one of five things: how fast work runs (the flow map), where work may proceed
(the flow set), when events may happen (the jump set), what events do (the jump map), or what is
represented (the state space). One reading of the book claimed the bottleneck maximum as a certificate
of completion time; an independent review corrected it to a lower bound, and the notes now keep lower
bounds, validated predictors and proved upper bounds apart ([journal §70](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md)).

## 13. Source register

All repository links are pinned to bfacdf6e, the master commit the post was written from.

- **The courses:** [the reading](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md), [the saved sources](https://github.com/lmmx/mibz2/tree/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/methodology/sources), and the outside answers on [the courses](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-courses.md) and [the resources](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-resources.md).
- **The readers:** [the summary](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md), [the opportunities with corrections](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md), [the discipline lead](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md), [the outside course mining](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md), [the second agent's review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md), and [the design review](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-design-review.md).
- **The ideal-parts matrix:** [the design](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md), [the registration](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md) and [slice A](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/slice-a/README.md).
- **An outside reconstruction** of this story, recorded verbatim with corrections: [ChatGPT Pro's narrative](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level/external/chatgpt-queueing-narrative.md).
- **The papers:** Little (1961), Jackson (1957), Gordon and Newell (1967), Baskett, Chandy, Muntz and Palacios (1975), Reiser and Lavenberg (1980), Schroeder, Wierman and Harchol-Balter (2006), each linked where it is used.
- [The journal](https://github.com/lmmx/mibz2/blob/bfacdf6e7fcd23e20dede638738af36c94d2e199/docs/journal/2026-11-24-algorithm-level.md).
