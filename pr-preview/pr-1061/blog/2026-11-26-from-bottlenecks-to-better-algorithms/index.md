[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/index.md>) · Companion to [Complementary bottlenecks](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-complementary-bottlenecks/index.md>)

# From bottlenecks to better algorithms: induction schedules, deforestation, forward lanes and a pooled scan

How the matrix's lessons could become new algorithms, from what the experiment left behind to four designs and the tests that would reject them.

26 November 2026 · 24 min read · [Research synthesis](<#10-comparing-the-candidate-algorithms>)

**The experiment this companion builds on** made each part of mibz2's decoder free in turn and in combination, and found where a new algorithm would pay. [Complementary bottlenecks →](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-complementary-bottlenecks/index.md>)

[Complementary bottlenecks](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-complementary-bottlenecks/index.md>) asks how to discover where a new algorithm would matter in mibz2's decoder, and answers it by experiment: each part's work was replaced by a recorded answer, alone and in combination, and the decode timed. This companion asks the next question: what might those algorithms look like, and how would we know whether one works? It holds the machinery the main post uses without stopping to develop: the formal description of the decoder as a system, the statistics of the factorial experiment, the accounting of what the experiment left behind, the mathematical principles and their known limits, the currencies in which a candidate is priced, and four concrete designs with their forecasts, costs and rejection tests. The general theory of performance analysis, operational laws and factorial design included, is in the earlier companion, [Beneath the queueing network](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-25-the-decoder-as-a-queueing-network-notes/index.md>), and is not repeated here.

Every saving in sections 6 to 9 is a forecast. The designs come from a second pass over the experiment's results, referred to here as the design analysis ([the design analysis](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)); none has been built or measured, and each is written with the smallest measurement that would reject it.

## 1\. The formal design space

The main post describes the decoder as a hybrid dynamical system and lists six things a redesign can change. Formally, such a system is a quadruple \\(\\mathcal H = (C, F, D, G)\\). Its state \\(x\\) *flows*, \\(\\dot x \\in F(x)\\), while it lies in the flow set \\(C\\), and *jumps*, \\(x^+ \\in G(x)\\), while it lies in the jump set \\(D\\) ([the decoder as a hybrid system](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md>)). For the decoder, the state holds each thread's remaining work and mode, the blocks each thread owns, the set of finished blocks waiting for the reader, and the admission counter, the number of blocks issued but not yet taken. That counter is an integer tested under a lock, so the state is partly discrete and must stay so in any model.

Between events each busy thread's remaining work falls at a rate set by contention; that is \\(F\\). The events are admitting a block and starting its scan, issuing it to the workers, a worker starting and finishing it, and the reader taking and consuming it; each has a guard in \\(D\\) and an effect in \\(G\\). Admission is checked before scanning: the scanner may start a block only while fewer than \\(W\\) blocks are in flight, and not while the bytes waiting for the reader have reached a cap with more than \\(k + 1\\) blocks in flight. A worker may take any queued block, so \\(G\\) is set-valued, and a choice within it is a policy.

The six levers are changes to these ingredients, and each comes with a known risk ([the levers](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md>)):

| Lever | What it changes | What to watch |
| --- | --- | --- |
| **The flow map \\(F\\)** | how fast each activity runs | speeding one rate moves the bottleneck to another term |
| **The flow set \\(C\\)** | where work may proceed, and where a thread must wait | thin, open flow sets are fragile under perturbation |
| **The jump set \\(D\\)** | when an event may happen | the admission guard is an exact integer test; widening it from \\(\< W\\) to \\(\\le W\\) breaks the window |
| **The jump map \\(G\\)** | what an event does: dispatch, routing | a choice within \\(G\\) is a policy, judged by a guarantee rather than by one trace |
| **Ownership** | who holds which buffer, and when it is released | every allocation needs an owner, and the earliest unfinished block must still get resources |
| **The state space** | what is represented | a new representation must be written in the same format, so it can be compared by the same measure |

A design search over these levers needs three different quantities, and confusing them was an early mistake in the project's own notes. A *lower bound* says no execution can finish earlier: for the decoder, \\(L = \\max(S, W/k, R)\\), the largest of the scanner's, the workers' and the reader's total service. A *predictor* estimates the completion time, and is valid only after it has been checked on runs it was not fitted to. An *upper certificate* guarantees that every permitted execution finishes within a bound. The bottleneck maximum is the first of these, not the third: a single block that takes 1 s to scan, 1 s to decode and 1 s to deliver has \\(L = 1\\) s but finishes in 3 s, because the maximum ignores serial dependencies, starvation and indivisible jobs. A route to a true certificate exists for a finite, acyclic expansion of the state graph with a defensible upper duration on every edge, by backward induction over the longest path, but none has yet been built for the decoder. Section 5 returns to the four kinds of number this distinction produces.

## 2\. Quantifying complementary bottlenecks

The experiment coded each part as \\(-1\\) when real and \\(+1\\) when ideal, and analysed the logarithm of the decode time, \\(y\\), so that effects that multiply the time add. For two parts \\(P\\) and \\(Q\\), the interaction coefficient is the difference of differences, scaled as in Jain's sign table:

\\\[ q\_{PQ} = \\tfrac14\\left(y\_0 - y\_P - y\_Q + y\_{PQ}\\right), \\\]

where \\(y\_0\\) is all real, \\(y\_P\\) and \\(y\_Q\\) have one part ideal and \\(y\_{PQ}\\) both. In the 32-run fraction the same coefficient is read from the sign table, averaged over the settings of the other four parts. The rule registered before the data labelled an interaction INTERACTS if its 95% interval excluded zero and ADDITIVE if it lay within \\(\\pm 0.01\\) ([the registration](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>)).

The fraction's defining relation, \\(I = +SECMRD\\), fixes what can be separated. Multiplying the relation by any effect gives that effect's *alias*, the effect it cannot be told apart from: \\(S = ECMRD\\), so each main effect is aliased with a five-part interaction; \\(SC = EMRD\\), so each pair with a four-part one; and \\(SEC = MRD\\), so the three-part interactions are aliased with each other in pairs. The analysis assumes the four- and five-part interactions negligible. The aliased three-part pairs are reported as a share of the variation, and that share grows with the worker count: 0% at one worker, 2% at six, 5% at eight and 6% at ten ([the report](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>)).

The allocation of the variation in log time shows where each part's influence lies across the experiment's configurations.

| Workers | S | E | C | M | R | D | pairs | aliased triples | residual |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0% | 14% | **80%** | 4% | 0% | 0% | 2% | 0% | 0.2% |
| 6 | 22% | 8% | **46%** | 2% | 0% | 0% | 20% | 2% | 0.4% |
| 8 | **46%** | 1% | 19% | 1% | 1% | 4% | 22% | 5% | 2% |
| 10 | **53%** | 0% | 9% | 1% | 3% | 10% | 18% | 6% | 1% |

This is a share of variation across hypothetical decoders, not a share of running time. Construction's 80% at one worker sits beside 578 ms of a 1,283 ms decode, about 45% of the time; the scanner's 46% at eight workers sits beside a 3.2% saving when it alone is supplied, because in most of the fraction's configurations other parts are already ideal and the scanner is exposed. The rooms in the main post's matrix are measured directly, from the twelve added configurations with one part ideal or one part real, and labelled by a second registered rule: ROOM if the upper end of the 95% interval is at most 0.95, NO ROOM if the interval lies within \\(\[0.98, 1.02\]\\), and UNRESOLVED otherwise. Each room is a geometric mean of paired ratios over twelve randomised blocks of rounds; the scanner was pinned to one core, every core spun for 100 ms before each decode to hold the clock, and the parts an oracle removed kept at most 0.19% of their usual busy time ([the design](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>); [the results](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)).

The scanner–construction interaction dominates: \\(q\_{SC} = -0.110\\) at six workers, interval \\(\[-0.112, -0.108\]\\), carrying 16.9% of the variation, and \\(-0.071\\) at eight, 13.1%; at one worker it is \\(-0.002\\). Three others are clearly present: the scanner with delivery at eight and ten workers, whose difference of differences \\(4q\\) is \\(-0.13\\) and \\(-0.14\\); construction with delivery, \\(-0.11\\) and \\(-0.08\\); and emission with construction at one worker, \\(-0.19\\). A scanner that is no longer the limit exposes delivery behind it, and an emission stage that no longer walks construction's structures changes what construction is worth.

The main post's two-station model explains the shape of the scanner–construction pair. With scanner demand \\(S\\) and worker demand \\(W\\) shared by \\(k\\) workers, removing fractions \\(a\\) and \\(b\\) of each gives

\\\[ T(a, b) = \\max\\!\\big(S(1-a),\\ W(1-b)/k\\big), \\\]

and at balance, \\(S = W/k\\), the reduction \\(1 - T/T(0,0) = \\min(a, b)\\): its contours are L-shaped. The model overstates the measured pair, 45% against 36%, and cannot produce the observed slowdown from eight to ten workers with both parts supplied, 125 ms to 149 ms. The design analysis revises it to three stations, with the shared L3 cache and memory bandwidth as the third: active workers tax the pinned scanner by about 6% through contention, and once the parts are made fast, memory binds, which would account for the all-ideal configuration's rise from 112 ms to 142 ms between eight and ten workers ([the design analysis, §§1 and 6](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)). That is a hypothesis. The matrix cannot separate the decoder's own contention from the traffic of the oracles' stores, which is the next section's subject.

Measured on 26 November, the zero-copy floor run found no sign of a memory-bandwidth third station between eight and ten workers. Memory traffic stayed at 13–17 GB/s, far below the machine's bandwidth. Without oracle copying, the all-ideal wall did not rise towards ten workers ([§3](<#3-accounting-for-the-remaining-computation>); [the true-floor post](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-true-floor-and-pooled-scan/#1-how-much-of-the-first-matrixs-floor-was-the-decoders>)).

## 3\. Accounting for the remaining computation

Every oracle hands its recorded answer to the decoder, and the handover was timed in a ledger entry of its own. The entries, per thread, with each part alone ideal:

| Workers | S (ms) | E (ms) | C (ms) | M (ms) | R (ms) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 15.3 | 285.5 | 30.2 | 9.2 | 52.4 |
| 6 | 20.3 | 52.8 | 6.1 | 1.9 | 10.6 |
| 8 | 24.9 | 41.8 | 5.0 | 1.8 | 10.4 |
| 10 | 33.6 | 38.1 | 4.5 | 1.8 | 11.7 |

The scanner's handover more than doubles from one worker to ten although its work is the same, which makes it a thermometer for contention. Emission's entry is the large one, and the design analysis found that it is mostly not a handover: the oracle's code times both the copy of the supplied words and the expansion of those words into bytes, which the oracle leaves in place. Earlier four-arm measurements of the same workload divide it, at one worker, into 25.8 ms of delivery and 259.4 ms of expansion, and in the same nine-to-one proportion at six and ten workers ([the design analysis, §3.1](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)).

With that division, the design analysis decomposes the all-ideal configuration by arithmetic. The scanner's handover runs concurrently with the workers', so at one worker the critical path is the workers' handovers and expansion, 377 ms of the 391 ms wall, leaving about 14 ms of everything else.

| Quantity, per decode (ms) | 1 worker | 6 workers | 8 workers | 10 workers |
| --- | ---: | ---: | ---: | ---: |
| measured all-ideal wall | 391 | 94 | 112 | 142 |
| *Residual A:* expansion plus the pipeline, zero-copy oracles | \~273 | \~60–62 | \~51–55 | \~50–54 |
| *Residual B:* the pipeline alone, expansion removed | \~14 | \~12–15 | \~14–17 | \~15–19 |
| simultaneous copying and lock contention | \~0 | \~8–11 | \~36–39 | \~67–71 |

Every row below the first is argued, not measured. Residual B is the pipeline's own cost: the feeder, the channels between threads, the column path for small blocks, start-up, draining and minor page faults. It is engineering, not mathematics. The remaining expansion in Residual A is partly mathematical: it performs about 106 million dictionary lookups and variable-length stores on the workload, a number set by how many rows the kernel retains, so a contraction that retained fewer rows while keeping its dictionary small would cut it. And there is a physical floor beneath both. The workload's 838.8 MB of output must be written at least once; at an assumed 40 GB/s of memory bandwidth that takes about 21 ms, less if the output stays in the L3 cache for a caller that reads it at once.

The last row explains the rising floor. In the all-ideal configuration the oracles copy about 4.9 GB back to back, and every block takes several global locks on the oracles' stores; at 38–42 GB/s that traffic alone takes 115–145 ms. The follow-up run is built to remove it: its oracles hand over their answers without copying, and one configuration skips the expansion as well, so that Residuals A and B are measured rather than inferred ([the floor run](<https://github.com/lmmx/mibz2/pull/917>)).

Measured on 26 November, the zero-copy floor run put both residuals beside the design analysis's forecasts, which stay above as registered before the data ([the floor results](<https://github.com/lmmx/mibz2/blob/311923efd5ac7b113a9f944db34f41172c096816/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md>); [the true-floor post](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-true-floor-and-pooled-scan/#1-how-much-of-the-first-matrixs-floor-was-the-decoders>); [the main post's note](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-26-complementary-bottlenecks/#6-what-the-experiment-could-not-remove>)):

| Per decode (ms) | 1 worker | 6 workers | 8 workers | 10 workers |
| --- | ---: | ---: | ---: | ---: |
| Residual A, forecast | \~273 | \~60–62 | \~51–55 | \~50–54 |
| Residual A, measured | 303.9 \[302.9, 305.0\] | 54.0 \[51.2, 56.7\] | 42.4 \[40.3, 44.4\] | 40.5 \[37.6, 43.5\] |
| Residual B, forecast | \~14 | \~12–15 | \~14–17 | \~15–19 |
| Residual B, measured | 5.7 \[5.5, 5.8\] | 5.3 \[5.3, 5.4\] | 5.2 \[5.0, 5.5\] | 5.4 \[5.3, 5.5\] |

*Measured rows are block means with 95% intervals over eight randomised blocks, from one run on one machine and one workload. Forecast rows and the copying-and-contention row above remain argued.*

Residual B was the run's scored prediction under the rule frozen before the data ([the floor run](<https://github.com/lmmx/mibz2/pull/917>)). It measured 5.2–5.7 ms across one to ten workers, or 2.7% of the shipped 194.6 ms decode at eight. That misses low at every count — whether compared against the floor report's 14–19 ms one-worker label or the per-count forecasts above — and closes pipeline plumbing as a target. Residual A was reported unscored. At one worker it exceeds its forecast because that configuration reads the oracles' stores cold from main memory: about 55–65 ms of its 298 ms word-spelling figure is that traffic, not the decoder's work ([the cold-store reread](<https://github.com/lmmx/mibz2/blob/2aaded3a3ba00f140d2621813e87a365758e975b/docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md>), [PR \#952](<https://github.com/lmmx/mibz2/pull/952>)). From eight to ten workers Residual A stays flat (42.4, 42.9 and 40.5 ms), confirming that the earlier rise to 142 ms was the old oracles' copying. The previous table's last forecast row, for simultaneous copying and lock contention, is not re-derived here. The matrix and the floor run are separate runs, so subtracting across them would mix their noise.

## 4\. Contraction, enrichment and closure

The main post introduces the two principles. Here are their limits, which matter as much as the principles do.

**Contraction and its floor.** The interval-exchange kernel contracts along runs of equal bytes in the last column: adjacent rows holding the same byte map to adjacent rows under LF, so a run moves as one interval. The floor called G1 says that rows with no such neighbour have nothing to contract with *in that sense*. Its scope is narrow: it bounds the operations charged in a calculus of rigid interval moves, not the decoder's unpredictable memory reads, and carrying it to machine cost needs a bridge theorem that holds for most of the old family of algorithms but not all of it ([Posing the inversion problem, §2](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-posing-the-inversion-problem/#2-steps-1-and-2-the-referee-and-the-toy-problem>); [the programme's statement](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/essence.md>)). An implementation may keep the same interval facts and acquire or consume them more cheaply.

**Closure and its negative answer.** Composing the retained-row map once more, so that one step does the work of two, was the obvious next contraction. A representation of the composed map has to do three things: keep the successor lookups, the word payloads and the access schedule cheap; stay compact after composition on real blocks; and be built without following the transitions it replaces. On the first real block it failed the second: the composed map's access pattern needed 21 to 24 times as many address segments as the original, and its word storage nearly tripled. A conservation law in a later record explains why: a composition keeps the *form* of the access schedule but not its compactness on high-entropy rows. That answers the closure problem negatively, but only within the class of sweeps the record defines ([the journal, §§42 and 48](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level.md>)). It closes one route, not the idea of a different representation, and so the search turns to other notions of moving together: up to a consistent relabelling of bytes, along repeated stretches of the text, or through a recurrence among the induction's own steps.

**Enrichment and its obstruction.** An enrichment carries an extra quantity along an existing step, as dual numbers carry derivatives, and is worth having only if the quantity is cheaply acquired and is not the answer in disguise; a row's position in the text fails, since the decode exists to compute it. One tempting site is closed. Moving construction's work upstream into the serial scanner would let the scanner's idle knowledge of the block shorten the workers' job, but the final LF targets cannot be known before a block's last byte: changing that byte shifts the targets of 15.4% to 50.0% of the rows on every Wikidata block tested. And the Huffman scan, before move-to-front, determines every run length but none of the byte identities: permuting the symbol map with identical codes changes the target of every row ([the design analysis, §4](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)). These are limits on acquiring the *final* answer early. They are not an impossibility result for a summary gathered during the scan and finished cheaply once the block ends, which remains open.

## 5\. The four currencies of computation

A candidate is priced in real machine cost, with the cost of building every structure it needs included ([the formulation](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md>); [Posing the inversion problem](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-posing-the-inversion-problem/#2-steps-1-and-2-the-referee-and-the-toy-problem>)):

- \\(Q\\), the unpredictable memory reads, whose address depends on data;
- \\(D\\), the length of the longest chain of reads that must wait on one another;
- \\(P\\), the passes over the data, with at most two allowed;
- \\(S\\), the working state held at once.

The currencies separate two kinds of success. A large reduction in \\(D\\), more independent reads in flight, can make the decoder much faster on real hardware and is a real result. The programme's breakthrough is a reduction in \\(Q\\): fewer than one unpredictable read for each row with no same-byte neighbour, with half of one as the target, within two passes. The number of live cursors is limited to about one per letter of the alphabet, so a route that needs thousands of concurrent cursors to reach its figure is inadmissible. Charging acquisition matters most: an idea that removes reads in the decode by reading everything once more beforehand has moved its cost, not removed it.

The work's numbers are of four kinds, and they are not competing estimates of one quantity.

| Kind | What it establishes | Example |
| --- | --- | --- |
| **Lower bound** | under stated assumptions, a computation cannot cost less | any order of the induction's steps takes at least 1.220 steps for each run (§6) |
| **Oracle measurement** | an observed counterfactual, with particular work supplied | 125 ms at eight workers with the scanner and construction supplied |
| **Predictor** | an estimate of what a concrete design would cost | 130–145 ms for a pooled scan with cheaper construction (§9) |
| **Upper bound** | under stated assumptions, completion takes no longer | none proved yet for this decoder |

Lower bounds prune the space of designs, oracle measurements say where the opportunity is, predictors rank the candidates, and upper bounds, when they exist, guarantee a result.

Numbers about one decode at eight workers, by kind

The same axis, different kinds of object. The lower bound is the design analysis's arithmetic for writing the output once at an assumed 40 GB/s; the oracle points are measured; the bars are forecasts, untested. The induction's 1.220 is a bound on steps, not milliseconds, so it has no place on this axis. No upper bound has been proved.

The four candidates below touch different parts of the workers' time, and what they touch bounds what they can be worth.

What each proposal touches, at one worker

Measured bar: one worker's busy time at one worker, by activity, from stage 1 of the characterisation (its wall, 1,283 ms, differs slightly from the matrix's 1,306 ms). Brackets: the activities each proposal changes, with the design analysis's forecasts. The pooled scan changes the scanner's thread, which this bar does not include.

## 6\. Alternative induction schedules

The induction is the part of construction most open to mathematics, and its cost has an exact count. For every order in which its steps can be taken,

\\\[ \\text{steps} = r\_2 - 1 + U, \\\]

where \\(r\_2\\) counts the distinct pairs of adjacent run letters, so that \\(r\_2 - 1\\) is a floor no order can beat, and \\(U\\) counts the extra boundaries a particular order creates along the way ([the induction record, §5.6](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/monodromy/2026-11-10-rauzy-veech.md>)). On all 233 Wikidata blocks, the historical order, which always takes the step at the right end, needs 1.812 steps for each run of the original column. An order that takes first the steps that create no new boundary, and otherwise the largest cell, needs 1.279, and the floor is 1.220. These counts were taken on the earlier full exchange over all run letters, not on the object the present kernel builds, which eliminates thick stretches and keeps thin rows.

The better order cuts the steps by \\(1 - 1.279/1.812 \\approx 29.4\\%\\). If the whole induction sub-phase, 216 ms at one worker, scaled with the step count, that would save \\(216 \\times 0.294 \\approx 63.5\\) ms, 4.9% of the 1,306 ms decode. Setup and the dictionary of words do not depend on the order, so writing \\(f\\) for the share of the sub-phase whose cost scales with the number of steps, and \\(c\\) for the new order's own cost,

\\\[ \\text{net saving} \\approx 216\\ \\text{ms} \\times f \\times 29.4\\% - c . \\\]

The gate for pursuing the idea is 3% of the decode, 39.2 ms at one worker.

The economics of a cheaper induction schedule

Arithmetic on measured inputs: the 29.4% step cut from the induction record, the 216 ms sub-phase from stage 1 and the 1,306 ms wall. It assumes the historical cut transfers to the present kernel's object and that the reducible work scales in proportion to steps. The gate is crossed at 62%, 70% and 77% of the sub-phase for 0, 5 and 10 ms of scheduling overhead; at most 63.5 ms can be saved.

The schedule passes its gate only if at least about 62% of the sub-phase is step time that falls in proportion, before paying for the scheduling; with 5 ms of overhead the threshold is 70%. A short census measures that share on the object the decoder actually builds, and the idea stops if the share times the cut is under 3% ([the census](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>)). A schedule with fewer steps can also lose the memory locality and the cheap sweep order of the present one, which the census's timings would show. This is the narrowest of the four proposals and its ceiling is modest, but the census that settles it is cheap.

## 7\. Deforestation across stages

The move-to-front stage produces the block's runs capped at 127 bytes each, and construction's first pass immediately joins them back into maximal runs. Between the two sits an intermediate array, written in full and read back.

The direct fold: removing an intermediate representation

Design, not measured: today's three stages and the proposed two. The extra records and the traffic are the design analysis's counts on the Wikidata workload; the saving is its forecast.

The fold removes the intermediate representation where the fast path allows it: the move-to-front loop emits each maximal run once, and construction accumulates it directly into the counts and queues its later phases need, splitting only the run that contains the block's original-row pointer. Functional programmers call this [*deforestation*](<https://en.wikipedia.org/wiki/Deforestation_(computer_science)>), eliminating the intermediate data structure between two transformations. The present two-pass path stays as the fallback for blocks the kernel declines.

The capped runs add 3,990 to 5,284 records a block, 3.5–10.2% of the runs. Removing them saves one pass and about 360 MB of reads and writes over the workload, and 180 KB of state per worker. In the four currencies, \\(Q\\) and \\(D\\) are unchanged, since the move-to-front permutation is the same serial chain, and \\(P\\) and \\(S\\) fall. The forecast is 110–145 ms at one worker, about 8.5–11% of the workers' time, most of it in construction's first pass. The risk is branch misprediction in the move-to-front loop on the test for short runs, mitigated by how lopsided the run lengths are: in the workload's hexadecimal groups 83–85% of runs have length one, and in its template groups 98% exceed sixteen. The rejection test is a ten-minute profile that splits construction's first pass: if pushing rows into the queues takes more than 70% of it and the capping and rejoining take under 50 ms, the fold is dropped in favour of the wiring ([the design analysis, §5](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)).

## 8\. Forward wiring and multi-lane emission

Construction's last sub-phase wires the retained rows into a cycle, and emission walks that cycle to write the text. Today each link is stored as the XOR of its two neighbours, \\(\\text{link}(i) = \\text{prev}(i) \\oplus \\text{next}(i)\\), which saves memory and lets emission walk from one mark in both directions at once. But a step along an XOR link must know where it came from, so each walk is a chain of dependent reads, only two of them in flight at a time, and wiring the links costs some 106 million scattered read-modify-writes on the Wikidata workload.

The proposal stores the successor alone, \\(\\text{link}(i) = \\text{next}(i)\\), which can be written sequentially, and walks the cycle forward only, in several lanes at once. The cycle is cut at \\(K\\) anchors chosen uniformly at random; each lane walks one arc from its anchor to the next, and a lane that finishes takes the next unvisited anchor. The arcs come out in the wrong order, so a final pass stitches them together and performs the run-length expansion and checksum segment by segment.

Two ends of one cycle, against forward lanes from many anchors

Conceptual: one cycle of retained rows. Right, an illustrative draw of eight anchors walked by four lanes; the timeline shows each lane's arcs. With only twice as many anchors as lanes, one lane still finishes well after an even split, which is why the design takes four times as many. The expected longest arc is the order-statistics result in the text.

There is real mathematics in choosing \\(K\\). Uniformly placed anchors cut a cycle of length \\(m\\) into uneven arcs, and the walk is not finished until its longest arc is. In the continuous-spacing model the expected longest of \\(K\\) arcs is

\\\[ \\mathbb{E}\[L\_{\\max}\] \\approx m\\,\\frac{H\_K}{K}, \\\]

where \\(H\_K = 1 + \\tfrac12 + \\dots + \\tfrac1K\\) is the \\(K\\)-th harmonic number. With as many anchors as lanes, eight of each, the longest arc averages \\(0.340\\,m\\), nearly three times an even share, and the eight lanes behave like three. With four times as many anchors, \\(K = 32\\), it falls to \\(0.127\\,m\\), almost exactly the even share of \\(m/8\\). That is why the design takes many more anchors than lanes.

One change relieves several costs. Construction's wiring becomes sequential; emission gains several independent chains of reads to overlap, which the design analysis estimates lifts the reads in flight from about two to between six and seven; the backward expansion of words disappears; and the final pass can consume the segments directly. The forecast is about 120 ms from the wiring, 110–140 ms from the walk and 52–78 ms from the forward expansion, 280–340 ms in all at one worker, 21–26% of the workers' time, at the price of 1.0–1.5 MB of scratch per worker. In the four currencies, \\(D\\) falls by about a factor of three while \\(Q\\) is unchanged. The rejection test is a standalone harness on the 932 blocks of the workload that the kernel decodes: if the new path does not save at least 190 ms at one worker, because memory bandwidth saturates with four lanes, say, or because managing the segments eats the wiring's gain, it is dropped for a simpler prefetch on the present links ([the design analysis, §5](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)).

This proposal crosses a stage boundary, and it illustrates the experiment's lesson in miniature: a different consumer allows a cheaper structure to be built. It also illustrates the currencies. A walk in several lanes can make the decoder much faster without reducing the number of unpredictable reads, and if it does, the research question about reads remains open. Both outcomes deserve to be recorded as what they are.

## 9\. Dissolving the serial scanner

The scanner oracle deleted the Huffman work from the decode. A pooled scan would not delete it; it would change where it is done. Today one thread decodes every block's Huffman codes in turn and hands the symbols to the workers through a queue. Pooled, a fast pass first finds where each block starts, and each worker then scans its own block before rebuilding it.

The pooled scan: the same work, redistributed

Design, not measured. The pooled scan removes the serial station and the queue between it and the workers; the Huffman work itself remains, spread across the workers. A false marker inside compressed data is caught because each block must end exactly where the next begins.

**Finding the blocks.** A bzip2 block begins with a 48-bit marker, the digits of \\(\\pi\\) in binary-coded decimal, but blocks are not byte-aligned, so the search runs over all eight bit alignments of the compressed file. The design analysis estimates 1–2 ms on the 47 MB workload with vector instructions, and bounds the chance of a false match by \\(8\\,|B|\\cdot 2^{-48}\\) for a file of \\(|B|\\) bytes, about one in a million here. A false match is caught rather than trusted: each block's decode must end exactly where the next block's marker begins, and on any mismatch the decoder falls back to a serial continuation from the true end.

**Redistributing the work.** Eight or nine workers each own a small scanning scratch and a window of a few megabytes of compressed input, and run scan, move-to-front, construction, emission and the final expansion locally before publishing their block; the count stays below ten to keep the working set inside the 20 MiB L3 cache. The symbols queue between scanner and workers, about 400 MB of traffic over the workload, disappears.

**Why it interacts with construction.** If a block costs \\(s\\) on the scanner and \\(w\\) on a worker, a serial scanner caps the throughput at \\(1/s\\) blocks per unit time however many workers there are, while \\(k\\) pooled workers, each doing both jobs, complete about \\(k/(s + w)\\). The knee puts \\(w\\) at about \\(7.5\\,s\\), so at eight or nine workers, where the two stations are nearly balanced, pooling alone gains little in this model. As construction gets cheaper, though, \\(w\\) shrinks and the pooled rate rises, while the serial cap does not move. The pooled scan and cheaper construction are the main post's complementary pair, made into designs.

**Why not enrich the scanner instead.** Pushing construction's work into the serial scanner would lengthen the station already on the critical path; a fused Huffman and move-to-front scanner tried earlier slowed the scan from 74.5 ms to 107.9 ms and the whole decode by 18.5%. And §4's obstructions forbid computing the final LF targets before a block ends. Dissolving the boundary is the one move that touches both the scanner and construction; the induction and the wiring must still be attacked inside the workers ([the design analysis, §4](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)).

The forecast is 172–175 ms at eight or nine pooled workers on its own, 10–12% below today's 195 ms, and 130–145 ms combined with cheaper construction. The measured 125 ms with both parts supplied makes that a sensible scale to aim at, though it does not validate the forecast. The rejection tests are a five-minute timing of the marker search, which fails above 10 ms single-threaded, and a minimal prototype at eight and nine workers, which fails unless it beats 195 ms by at least 7% while keeping peak memory under 75 MB.

## 10\. Comparing the candidate algorithms

| Proposal | Changes | Forecast saving | Extra cost | Rejected if |
| --- | --- | --- | --- | --- |
| **Induction schedule** | the order of the induction's steps | at most 63.5 ms at one worker; gate 39.2 ms | the schedule's own bookkeeping | step share × cut \< 3% of the decode |
| **Direct fold** | MTF and construction's first pass | 110–145 ms at one worker | branch mispredictions in the MTF loop | queue pushes dominate the first pass, and capping and rejoining take \< 50 ms |
| **Forward wiring with lanes** | wiring, emission, final expansion | 280–340 ms at one worker | 1.0–1.5 MB scratch a worker; stitching the arcs | saves \< 190 ms at one worker |
| **Pooled scan** | where the Huffman scan runs | 172–175 ms at eight workers; 130–145 ms with cheaper construction | the marker search; a compressed look-ahead window | marker search \> 10 ms, or \< 7% faster than 195 ms |

All four are forecasts by one analysis, with different degrees of support, and the one-worker figures say nothing directly about eight workers, where the scanner limits. Read with the experiment, they sort themselves. The fold and the forward wiring reduce the workers' demand and so lower the knee; the pooled scan removes the serial limit above it; and the largest prize lies in combining a worker-side reduction with the pooled scan. The induction schedule is narrow, but its census is the cheapest of the four measurements to run.

Three mathematical questions remain open behind them. The first is closure: whether some representation of composed retained-row transitions stays compact after composition, which §4's negative result rules out only within one class. The second is a notion of moving together other than equal adjacent bytes, which would change what the induction contracts along. The third is a deferred summary: a quantity gathered during the scan and finished cheaply once a block ends, which §4's obstructions leave open. A candidate that answered any of them would be priced in the four currencies of §5 and aimed, through the main post's matrix, at the part of the decoder where it could pay.

---

**Sources:**

- The decoder as a system: [the hybrid-system note](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md>) and [the levers](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md>).
- The experiment and its statistics: [the registration](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>), [the design](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>), [the results](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>) and [the report](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>); journal §89.
- The residual accounting, the obstructions and the four designs: [the design analysis](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>); journal §90.
- Contraction, enrichment and closure: [the programme's statement](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/essence.md>); journal §§42 and 48.
- The currencies: [the formulation](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md>).
- The induction counts: [the induction record, §5.6](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/monodromy/2026-11-10-rauzy-veech.md>), and [the census](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>).
- The independent review of the experiment, [recorded in full](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>), and its reviews of the main post, [recorded in full](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt-on-the-post.md>); journal §91.
- [The floor run](<https://github.com/lmmx/mibz2/pull/917>).
