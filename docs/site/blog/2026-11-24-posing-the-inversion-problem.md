---
title: "Posing the inversion problem"
image: assets/social/blog-2026-11-24-posing-the-inversion-problem.png
description: "The search for a new inversion algorithm became an eight-step programme, from a cost referee and measured ceilings to the model that will say where new mathematics fits."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Posing the inversion problem

The search for a new inversion algorithm became an eight-step programme, from a cost referee and measured ceilings to the model that will say where new mathematics fits.
{: .post-dek }

24 November 2026 · 14 min read · [Research synthesis](#8-where-things-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

The work began as a search for a new mathematical basis for undoing bzip2's block sort, and it has
become a programme in eight steps. The first two are done: a cost referee says what would count as
beating the floor ([What else moves together](2026-11-24-what-else-moves-together.md)), and oracle
measurements say where whole-decoder time can be won at all, up to 26% at ten workers, hidden until
now behind the serial scanner
([Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md)). The
middle steps treat the decoder as what it is, a closed queueing network, and apply the methods of
classical performance analysis to it: operational laws as completeness checks, designed experiments
across every worker count, and a model fitted on some counts and judged on the rest. That model's
purpose is the seventh step, to say which activities really control throughput at the operating
points that matter, and so where a new mathematical device would have to act to move the whole
decoder. The mathematics itself, the eighth step, already has a map of the routes ruled out and a
conservation law that bounds one class of them. Every figure here comes from
[the formulation report](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md) and the records it cites.

## 1. The programme

The chain follows [Raj Jain's course](https://www.cs.wustl.edu/~jain/iucee/index.html){ .il-ref }
on computer systems performance analysis, built on his *The Art of Computer Systems Performance
Analysis* (1991): metrics and workloads, then measurement, experimental design, queueing models and
simulation ([the programme](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/programme.md)).

| Step | What it is | Status |
|---|---|---|
| 1 | The task and the referee: what counts as success, in which currency | done |
| 2 | The toy problem: a fixed machine, workload and referee, with payoff ceilings by oracle | done; three ceilings unmeasured |
| 3 | The system's structure: activities, buffers, shared resources and the feedback loop | designed |
| 4 | Measurement across 1 to 20 workers | the scanner profile has run; results pending |
| 5 | Experimental design: a baseline at every count, then interventions at chosen counts | being built; the second stage awaits a design review |
| 6 | The model: operational laws, a simulation fitted and held out, a roofline per activity | after step 5's data |
| 7 | Where new mathematics fits: the activities with real control, and what a device must change there | after step 6 |
| 8 | The mathematics, aimed at step 7's targets | continuing |

## 2. Steps 1 and 2: the referee and the toy problem

The problem statement agreed by three independent answers asks for one uniform program, exact on
every column, that builds its structures from the column within fixed budgets and pays at most half
the old floor on every block. The currency is real machine cost: unpredictable memory reads (Q),
the length of the chain of reads that wait on one another (D), passes over the data (P, at most two)
and working memory (S), with the cost of building every structure included
([journal §33](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)). How candidates are found, by changing what moves together and by making
existing steps yield more, is the subject of the earlier post; the cost rules referee candidates
only once they exist.

Two statements made in earlier summaries, including ours, had to be withdrawn
([journal §42](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)):

- **G1 is not a lower bound on machine cost.** It bounds charges in a calculus of rigid interval
  moves; carrying it into machine cost needs a bridge theorem that holds for most of the old family
  but not all of it.
- **The 16% figure was never a ceiling.** It described one intervention, supplying the emitter's
  codes, at one thread. The four-arm test has since shown that the same intervention is worth
  nothing measurable at ten workers until the scanner is removed, and then a further 8%.

The referee's rulings close the remaining loopholes. A breakthrough means fewer than one
unpredictable read per [width-1 row](2026-11-24-halving-the-single-row-steps.md), with the target
at half of one, within two passes; a large cut in the chain of dependent reads is a real but lesser
find. The number of live cursors is limited to about one per letter of the alphabet, so a route that
needed some two thousand concurrent cursors to reach its figure was ruled inadmissible
([journal §49](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)).

The toy problem fixes the rest: an i9-10900K with AVX2, ten cores and twenty threads; the six
admitted blocks, with synthetic hex beside them; and that referee. Its payoff ceilings come from
the four-arm oracle ([journal §44](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)): emission is worth 15 to 17% at one to six workers, and at
ten workers the serial scanner is worth up to 19% on its own and 26% together with emission. The
decoder's work is divided into six targets, each tied to a stage: emission of the inversion (T1),
its construction (T2), move-to-front and run-length decoding (T3), the Huffman scan (T4), riders
across stages (T5) and the final run-length expansion and checksum (T6). Only T1, T4 and T5 have
ceilings so far ([the targets](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/math-targets/targets.tsv)).

## 3. Step 3: the decoder as a closed network

The scanner and its workers are only a coarse picture. The measurement plan, merged from three
independent answers to [a brief](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/decomposition.md), divides the decoder into thirteen activities, from the
input feeder through the scanner's table and Huffman decoding to the workers' move-to-front and run
formation (one fused activity), the three phases of Open's construction, emission, the fallback
route and the final checksum, and adds a serial tail in which a consumer takes finished blocks in
order and writes them out ([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §2; [journal §51](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)).

<figure class="bf-fig" id="closed-network">
<p class="bf-title">A fixed population of blocks, circulating</p>
<svg viewBox="0 0 720 300" role="img" aria-label="The decoder as a closed queueing network. A feeder passes input to a single scanner thread, which decodes headers, tables and Huffman codes. Blocks enter an admission window holding W = 4k blocks in flight, then k workers undo move-to-front and runs, invert and emit, then a consumer takes blocks in order and writes the output. A slot returns to the window only when the next block in order is taken, and finished output is capped at 12 MiB, so one slow early block can stall the scanner."><rect x="10" y="40" width="96" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="58.0" y="62" text-anchor="middle" style="font-size:13px">feeder</text><text class="bf-tm" x="58.0" y="80" text-anchor="middle" style="font-size:11px">reads and</text><text class="bf-tm" x="58.0" y="95" text-anchor="middle" style="font-size:11px">copies input</text><rect x="128" y="40" width="122" height="92" rx="8" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="189.0" y="62" text-anchor="middle" style="font-size:13px">scanner</text><text class="bf-tm" x="189.0" y="80" text-anchor="middle" style="font-size:11px">one thread:</text><text class="bf-tm" x="189.0" y="95" text-anchor="middle" style="font-size:11px">headers, tables,</text><text class="bf-tm" x="189.0" y="110" text-anchor="middle" style="font-size:11px">Huffman decode</text><rect x="272" y="40" width="170" height="92" rx="8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text class="bf-t-strong" x="357.0" y="62" text-anchor="middle" style="font-size:13px">admission window</text><text class="bf-tm" x="357.0" y="80" text-anchor="middle" style="font-size:11px">W = 4k blocks in flight:</text><text class="bf-tm" x="357.0" y="95" text-anchor="middle" style="font-size:11px">queued, running, or done</text><text class="bf-tm" x="357.0" y="110" text-anchor="middle" style="font-size:11px">but not yet taken in order</text><rect x="464" y="40" width="122" height="92" rx="8" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="525.0" y="62" text-anchor="middle" style="font-size:13px">k workers</text><text class="bf-tm" x="525.0" y="80" text-anchor="middle" style="font-size:11px">MTF and runs,</text><text class="bf-tm" x="525.0" y="95" text-anchor="middle" style="font-size:11px">inversion,</text><text class="bf-tm" x="525.0" y="110" text-anchor="middle" style="font-size:11px">emission, CRC</text><rect x="608" y="40" width="102" height="92" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="659.0" y="62" text-anchor="middle" style="font-size:13px">consumer</text><text class="bf-tm" x="659.0" y="80" text-anchor="middle" style="font-size:11px">takes blocks</text><text class="bf-tm" x="659.0" y="95" text-anchor="middle" style="font-size:11px">in order,</text><text class="bf-tm" x="659.0" y="110" text-anchor="middle" style="font-size:11px">writes output</text><path d="M106,86.0 L120,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="128,86.0 120,81.0 120,91.0" style="fill:var(--bf-frame)"/><path d="M250,86.0 L264,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="272,86.0 264,81.0 264,91.0" style="fill:var(--bf-frame)"/><path d="M442,86.0 L456,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="464,86.0 456,81.0 456,91.0" style="fill:var(--bf-frame)"/><path d="M586,86.0 L600,86.0" style="stroke:var(--bf-frame);stroke-width:1.8;fill:none"/><polygon points="608,86.0 600,81.0 600,91.0" style="fill:var(--bf-frame)"/><path d="M659,132 L659,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M659,184 L357,184" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><path d="M357,184 L357,141" style="stroke:var(--bf-steel);stroke-width:1.8;fill:none;stroke-dasharray:5 4"/><polygon points="357,133 352,141 362,141" style="fill:var(--bf-steel)"/><text class="bf-tm" x="508" y="202" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">a slot is freed only when the next block in order is taken,</text><text class="bf-tm" x="508" y="217" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">and finished output is capped at 12 MiB</text><text class="bf-tm" x="10" y="244" style="font-size:11.5px">A fixed population of blocks circulates, so the decoder is a closed network: one slow early block can hold slots</text><text class="bf-tm" x="10" y="260" style="font-size:11.5px">and stall the scanner, so a waiting scanner does not by itself show that the workers are too few.</text></svg>
<figcaption>Schematic of the measurement plan's structure (§2 of the plan). The admission window
holds a fixed number of blocks, so the decoder is a closed network, the case that classical
mean-value analysis solves.</figcaption>
</figure>

The feedback loop in the figure is what makes the network closed. Admission is limited by the
blocks issued but not yet taken in order, and by a 12 MiB threshold on finished output, so a block
that is queued, running, or finished out of order all hold a slot; one slow early block can stall
the scanner through the consumer, and a waiting scanner does not by itself show that the workers are
too few ([the outside answer](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/external/chatgpt-decomposition.md)). Two further corrections from that answer shape every
measurement. Requested threads are not actual workers, since a request for ten or more runs seven.
And the summation theorem of
[metabolic control analysis](https://en.wikipedia.org/wiki/Metabolic_control_analysis){ .il-ref }
is no test of completeness: for an activity whose rate is the minimum of two others, the control
coefficients sum to two under slowdowns and to zero under speedups.

Completeness comes instead from the operational laws of queueing theory, which hold on any measured
trace without assumptions about distributions: Little's law, the utilisation law U = X·S, and the
forced-flow law. The plan checks them explicitly for every activity and buffer
([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §7a). Because the window holds W = 4k blocks, the closed-network results apply
directly. Each activity's demand Dᵢ, its visits times its service time, gives asymptotic bounds on
throughput, X ≤ min(1/D_max, N/(D + Z)), with a knee at N\* = (D + Z)/D_max, beyond which queueing
is certain; the knee is the analytic counterpart of the worker count at which the scanner takes
over. Mean-value analysis, the exact recursion for closed product-form networks, sits beside the
simulation as a cheap check on it ([the methods](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md)).

## 4. Steps 4 and 5: measurement and design

The first stage measures a baseline at every worker count from 1 to 20, in twelve randomised complete
rounds, never as a rising sweep: 240 observations, each logging every block's route, stage
boundaries, queue transitions and the four kinds of waiting, with separate ledgers for wall states
and time on the CPU ([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §3). Because delay depends on the variability of service and
not only its mean, the plan reports the second moment of each activity's per-block service time as
well. In a single-server queue with random arrivals (M/G/1), the mean wait is
λE[S²]/(2(1 − ρ)), so in an illustrative case at 80% utilisation and a 10 ms mean service time, it is 20, 40 or 200 ms as the
service time's squared coefficient of variation goes from 0 to 1 to 9
([the outside answer on the courses](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-courses.md); [journal §54](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)). Long-tail blocks are the
mechanism behind the admission stalls.

The second stage perturbs the activities one at a time at six worker counts, 1, 6, 8, 10, 12 and 20,
slowing each by 20%, with 10% follow-ups to test linearity, using a calibrated busy loop that
touches no memory ([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §5). One-factor-at-a-time cannot see interactions, and the
four-arm result was itself an interaction. So at 8 and 10 workers, where control is expected to
change hands, the plan replaces it with a 2⁹⁻⁴ fractional factorial of resolution IV: 32 runs over
nine activities, which estimates every main effect clear of two-factor interactions, at 768
observations against 216 for one-at-a-time ([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §7a). Before that stage is frozen,
the design goes to three independent reviewers. They are asked which competing mechanisms it must
tell apart, the most informative design for about two to three hours of workstation time, how
measurement on the i9 could be biased, whether every input each model needs is logged, and what to
cut ([the design review](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character-design-review.md); [journal §55](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)).

## 5. Step 6: the model

The primary model is a discrete-event simulation of the network above, driven by each block's
measured service times in their original order, with a roofline per activity (compute, dependent
reads and bytes against what the machine allows at each worker count) to reject infeasible
predictions. Mean-value analysis and a fluid approximation of scanner rate against worker demand
give closed-form checks. The model is fitted on nine worker counts and judged once on the other
eleven, on the 10% follow-ups, on a held-out combined slowdown and on the four-arm arms, against a
target fixed in advance: at most 3% median and 5% maximum error on mean runtime, with the waiting
fractions within five points. A model that fits throughput but gets the queues wrong fails
([the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §7).

## 6. Step 7: where new mathematics fits

The model's purpose is to price a mathematical idea before anyone builds it. Modification analysis,
from [Mor Harchol-Balter's course](https://www.cs.cmu.edu/~harchol/Perfclass/class.html){ .il-ref } on performance modelling, asks what happens
to a closed system when one device's demand changes; a candidate device is then a change to one Dᵢ,
and the bounds and mean-value analysis say what it would buy the whole decoder
([the methods](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md); [the plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), §7b). A recitation in that course shows why this
matters: raising an LRU cache's hit ratio can lower throughput, by intensifying contention on the
hit path, so that measurement, simulation and a bound together trace a rise, a plateau and a
decline. It is a classroom version of the masked bottleneck, in which speeding up one stage did
nothing until another was cleared ([the outside answer on the courses](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-courses.md)).

Engineering leads wait for the same answer. Stream compaction, the `copy_if` that does not
autovectorise because each output position depends on every earlier predicate, can be done
branchlessly by compressing the selected lanes of a vector and advancing by the popcount of the mask
([the post that measured it](https://loonatick-src.github.io/posts/vectorized-copy-if-analysis/)). The workstation has AVX2 but not AVX-512, so the decoder
would use a shuffle table indexed by an 8-bit mask, or BMI2's `pext`, in the workers' filter steps:
run-record extraction, the thin and thick partition of Open's construction, and the appends of
emission. It lowers the unit price of those steps without changing what the algorithm does, and it
is held until the decomposition shows that one of them controls throughput
([the lead](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/leads/vectorised-compaction.md)).

## 7. Step 8: the mathematics

<figure class="bf-fig" id="three-corners">
<p class="bf-title">Every route so far wins one cost by paying another</p>
<svg viewBox="0 0 720 400" role="img" aria-label="Three families of routes around an empty target. Pointer doubling and list ranking need few unpredictable reads but 117 to about 600 passes. Lanes and landmark induction shorten the chain of dependent reads to 0.07 to 0.22 percent of the block but leave the reads unchanged. Runs and copy plans save reads only where text repeats. The target, under half a read per width-1 row within two passes with every charge counted, has no route yet; composed routes cost 1.3 to 2.8 reads per width-1 row."><rect x="20" y="16" width="250" height="112" rx="8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text class="bf-t-strong" x="145.0" y="40" text-anchor="middle" style="font-size:14px">Few reads, many passes</text><text class="bf-tm" x="145.0" y="62" text-anchor="middle" style="font-size:12px">pointer doubling, list ranking</text><text class="bf-tm" x="145.0" y="79" text-anchor="middle" style="font-size:12px">117 to about 600 passes</text><text class="bf-tm" x="145.0" y="96" text-anchor="middle" style="font-size:12px">against a budget of two</text><rect x="450" y="16" width="250" height="112" rx="8" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="575.0" y="40" text-anchor="middle" style="font-size:14px">Same reads, shorter chains</text><text class="bf-tm" x="575.0" y="62" text-anchor="middle" style="font-size:12px">lanes, landmark induction</text><text class="bf-tm" x="575.0" y="79" text-anchor="middle" style="font-size:12px">chain down to 0.07–0.22% of n</text><text class="bf-tm" x="575.0" y="96" text-anchor="middle" style="font-size:12px">reads per row unchanged</text><rect x="235" y="284" width="250" height="104" rx="8" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="360.0" y="308" text-anchor="middle" style="font-size:14px">Fewer reads where text repeats</text><text class="bf-tm" x="360.0" y="330" text-anchor="middle" style="font-size:12px">runs, copy plans</text><text class="bf-tm" x="360.0" y="347" text-anchor="middle" style="font-size:12px">nothing to share on novel</text><text class="bf-tm" x="360.0" y="364" text-anchor="middle" style="font-size:12px">random IDs, where width-1 rows live</text><rect x="250" y="150" width="220" height="104" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-ink);stroke-width:1.5;stroke-dasharray:6 4"/><text class="bf-t-strong" x="360.0" y="174" text-anchor="middle" style="font-size:14px">The target</text><text class="bf-tm" x="360.0" y="196" text-anchor="middle" style="font-size:12px">under 0.5 reads per width-1 row</text><text class="bf-tm" x="360.0" y="213" text-anchor="middle" style="font-size:12px">within two passes, all counted</text><text class="bf-t-strong" x="360.0" y="230" text-anchor="middle" style="font-size:12px">no route yet</text><path d="M210,128 L280,150" style="stroke:var(--bf-frame);stroke-width:1.6;stroke-dasharray:4 4;fill:none"/><text class="bf-tm" x="196" y="150" text-anchor="end" style="font-size:11.5px;font-style:italic">pays in passes</text><path d="M510,128 L440,150" style="stroke:var(--bf-frame);stroke-width:1.6;stroke-dasharray:4 4;fill:none"/><text class="bf-tm" x="524" y="150" text-anchor="start" style="font-size:11.5px;font-style:italic">saves waiting, not reads</text><path d="M360,284 L360,254" style="stroke:var(--bf-frame);stroke-width:1.6;stroke-dasharray:4 4;fill:none"/><text class="bf-tm" x="372" y="274" text-anchor="start" style="font-size:11.5px;font-style:italic">fails on novel content</text><text class="bf-tm" x="20" y="300" style="font-size:12px">Composed routes, every</text><text class="bf-tm" x="20" y="317" style="font-size:12px">charge counted: 1.3–2.8 reads</text><text class="bf-tm" x="20" y="334" style="font-size:12px">per width-1 row; Open's</text><text class="bf-tm" x="20" y="351" style="font-size:12px">own walk: 4.3–6.9</text></svg>
<figcaption>Schematic, with figures from the reviews (<a href="https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/neon-movement-critique.md">the six senses</a>,
<a href="https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/neon-movement-cycle4-critique.md">the fourth cycle</a>) and journal §§39 and 41. Routes that need few unpredictable
reads take far more than two passes; routes that shorten the chain of waiting reads leave the
reads where they were; routes that share reads need repeated text, which novel random IDs lack.
With every charge counted, no route reaches the middle.</figcaption>
</figure>

Twelve candidate senses of moving together were built, measured on the six real blocks and
reviewed, and none brought the reads below one per width-1 row once every charge was counted. The
best composed routes cost 1.3 to 2.8 reads per width-1 row all in, against 4.3 to 6.9 for [Open](2026-11-19-open-returns.md)'s own walk measured the same way ([the fourth cycle's review](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/neon-movement-cycle4-critique.md)). Pointer doubling and
independent-set contraction reported zero unpredictable reads, but at 117 to about 600 passes, and
building their inputs costs reads after all; both are known list-ranking algorithms for external
memory ([the review](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/neon-movement-critique.md)). Landmark induction, recounted, brought the chain of waiting reads to
0.07–0.22% of the block, a record that has been reproduced, but left the reads at about one per
row ([journal §41](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)). Four further objects, boundary chords, radix-fibre cones, quotient slabs and unitigs, were ruled out, two of them repeating an investigation from four days earlier ([the review](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/carbon-cycle1-critique.md)), and lazily contracting the thin runs was closed
by a rule registered in advance, because removing interior rows fragmented the order of reads
4 to 9 times over ([journal §49](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)).

<p class="il-rubric-head">Of the sweep that keeps its form and loses its compactness</p>

One result explains why so many routes fail in the same way. [Record 119](https://github.com/lmmx/mibz2/blob/8cf099eb24d9943417cfccdf3763b54fbcd9c26b/docs/journal/monodromy/2026-11-24-closure-problem.md) shows that a
sweep over the block is itself an induction, and that composing steps preserves the *form* of the
access schedule but not its *compactness* on high-entropy rows. On the first block, the shipped
decoder's next-row addresses fall into 2,148 orderly segments, 0.061 per width-1 row; one round of
composition shatters them into 44,814–52,319, some 21 to 24 times as many
([journal §42](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)). The law accounts for that figure, and settles the closure problem
negatively within its class: no cleverer sweep of this kind can compose without fragmenting.
Smaller results stand beside it ([the review of records 118–125](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reviews/monodromy-118-125.md)):

- a lower bound on failing steps for every interleaving of monotone cursors
  ([record 118](https://github.com/lmmx/mibz2/blob/ad169f87d657edca999d503b3637da1b6ca0a597/docs/journal/monodromy/2026-11-24-interleaved-orders.md));
- the block length, the run count, the number of width-1 rows and the run-width histogram are
  free by-products of the Huffman scan, and parsing blocks in parallel is exact, as
  [lbzip2](https://github.com/kjn/lbzip2) already does ([record 122](https://github.com/lmmx/mibz2/blob/c348f8f5135959cf6d3dbf318001797673ec7201/docs/journal/monodromy/2026-11-24-scanner-corrected.md));
- an exact composable summary of each chunk for move-to-front, run-length decoding and the run
  records ([record 121](https://github.com/lmmx/mibz2/blob/a3facefcb7d927e8284381a299c6a80f0b58409f/docs/journal/monodromy/2026-11-24-scanner-summary.md));
- building the structures by interval-exchange acquisition costs 0.10–0.34 reads per width-1
  row, which puts construction back at its true scale ([journal §49](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md)).

Five threads remain open. Acquisition is the binding cost on real blocks, and a search-free strip
contraction projects about 0.6 reads per width-1 row but is unmeasured
([record 120](https://github.com/lmmx/mibz2/blob/5d0795c81f073c44cf1b2f9d62f0ac190ca07dd3/docs/journal/monodromy/2026-11-24-thin-strip-acquisition.md)). A representation outside record 119's class could compose without
fragmenting, which by the law no cleverer sweep can. The relation that pairs the same offset in
repeated JSON records is open again, since the obstruction claimed against it proved invariance
only. Riders across stages are not excluded by the theorems against them, which rule out final
values before the block ends but not deferred ones. And at the scanner there is engineering to
do: a parallel scan gated by worker count, which the four-arm test prices at up to 19%.

## 8. Where things stand

The first two steps are done, and the structure is designed. The profile of the scanner across all
twenty worker counts, recording memory, scheduling and waiting for each decode, has run on the
workstation, and its results have not yet been read. The first stage of the characterisation, the
baseline at every count, is being built. The second stage waits on the design review, whose three
answers will be merged into the plan before it is frozen. Nothing in the model, and so nothing in
the ranking of where mathematics fits, has been measured yet; the mathematics continues meanwhile
against the targets as they stand.

The ceilings came from oracles run before any candidate was built, and the routes' figures from
reviews made independently of the work they judged; most of the headlines shrank under that review.
What remains is a referee for any candidate, a price for each stage, a record of the routes already
closed, and a plan to find out which stage the next idea must change.

---

**Sources:**

- [The programme](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/programme.md), [the formulation report](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md) and [the journal](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level.md), §§33–55.
- [The measurement plan](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character.md), [the design review](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/character-design-review.md), [the decomposition brief](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/briefs/decomposition.md) and [the outside answer on it](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/external/chatgpt-decomposition.md).
- The performance-analysis courses: [Jain](https://www.cs.wustl.edu/~jain/iucee/index.html), [Harchol-Balter's 15-857](https://www.cs.cmu.edu/~harchol/Perfclass/class.html) and [15-849b](https://www.cs.cmu.edu/~harchol/Perfclass/classtheory.html); our reading in [the methods note](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/methodology/performance-courses.md), and [the outside answer](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/external/chatgpt-performance-courses.md).
- [The targets](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/math-targets/targets.tsv) and [the compaction lead](https://github.com/lmmx/mibz2/blob/a4bef380a2585687d773064617d5713f8cdc66b5/docs/journal/2026-11-24-algorithm-level/leads/vectorised-compaction.md).
- The records and reviews cited in §7.
