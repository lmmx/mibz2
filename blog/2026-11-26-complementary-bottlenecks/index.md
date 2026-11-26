[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Complementary bottlenecks: reading the ideal-parts matrix

At eight workers the scanner and construction must be relieved together, and the matrix is enough to start designing for that.

26 November 2026 · 17 min read · [Research synthesis](<#7-what-remains-to-measure-and-why-it-ends>)

[The decoder as a queueing network](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/index.md>) described mibz2's decoder as a pipeline. A single thread, the scanner, decodes each compressed block's Huffman codes in order, and a pool of workers rebuilds the original text of the blocks in parallel from what the scanner produces. Because the two stages run at the same time, a part's share of the running time does not tell us what making it faster would gain: if the workers are nearly as slow as the scanner, a faster scanner simply hands the limit to the workers. So to learn what a better algorithm for some part could buy, we measured it directly. For each part of the decoder we built an *oracle*, a stand-in that hands the decoder that part's correct output, prepared before the clock starts, so that the part costs almost nothing; then we timed every combination of parts made free in this way, at one, six, eight and ten workers. That experiment, the ideal-parts matrix, is the subject of this post.

Its clearest finding concerns a pair of stages. At eight workers, the count mibz2 now builds by default on this machine, freeing the scanner alone gains 3% and freeing construction alone, the stage in each worker that builds the structures used to undo bzip2's block sort, gains 6%, because whichever of the two is left still holds the decode back. Freeing both gains 36%, taking a decode from 195 ms to 125 ms. The scanner and construction are complementary bottlenecks: the gain belongs to the pair, and there is no meaningful way to divide it between them. With fewer workers the picture is different. At one worker, construction alone is worth 42% of the time, but construction is three distinct sub-phases, so an idea that improves one of them can only ever claim that sub-phase's share. And the matrix cannot yet say how fast the decoder could ultimately run, because the oracle for emission, the last stage, which writes out the bytes, removed only part of that stage's work.

An independent review of the experiment, recorded in full in the research journal, corrected several of our first readings of these results, and this post follows its account ([the review](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>); [the corrections](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/README.md>)). It begins with the question the matrix was built to answer, then how it was designed, what it found, what its "everything free" configuration left behind, where new mathematics could help, and what remains to be measured.

**Takeaways**

- At eight workers, freeing the scanner or construction alone gains 3–6%, while freeing both gains 36% (195 → 125 ms, 1.56 times the throughput); the two are a complementary pair, and the gain cannot be split between them.
- Below [the knee](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/#5-the-bottleneck-law>), where the workers rather than the scanner set the pace, construction is the largest opportunity (42% at one worker), but it is three sub-phases, and any one idea has only its own sub-phase's budget.
- The oracle for emission still expanded codes into bytes, so the decoder's true floor has not been measured; a short follow-up run ([\#917](<https://github.com/lmmx/mibz2/pull/917>)) measures it.
- The matrix is enough to start designing. Each measurement that remains settles one decision and then stops.

## 1\. The question the matrix answers

Three questions about a decoder's speed are easy to confuse, and each needs a different experiment.

| Question | What answers it | What it tells you |
| --- | --- | --- |
| Where does the time go? | profiling, activity timings | how much work there is, and where |
| What limits the decode now? | scaling runs, and slowing one activity at a time | which constraints bind in the present configuration |
| What would be worth making cheaper? | making parts free, alone and together | the room an improvement could have, including room hidden behind another bottleneck |

The second and third differ because the decoder overlaps its work, so slowing a stage and speeding it up are not mirror images. If the scanner and the workers could each get through a file in about 185 ms, slowing either would slow the whole decode, yet speeding either would leave the other still taking 185 ms. The [earlier experiments](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/#11-testing-the-explanation-by-intervention>) answered the second question, which stage limits the decode at which worker count. The matrix was built for the third, because that is the one that chooses targets for mathematics ([its purpose](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/README.md>)).

The matrix divides the decoder into six parts, each named by a letter.

The decoder's six parts

Schematic, after the review's diagram. Each letter is one part of the ideal-parts matrix; the dashed line is the admission credit the in-order reader returns.

The scanner (S) reads each block's coding tables and decodes its [Huffman symbols](<https://lmmx.github.io/mibz2/blog/2026-11-05-huffman-bit-buffer/index.md>) on its own thread. A worker then takes the block through four stages in turn: the move-to-front and second run-length stages (M), which turn the symbols back into the block's last column; construction (C), which builds the structures for undoing the block sort, namely a set of retained rows, the [interval-exchange](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) reductions between them, the words those reductions spell, and the links that join them; emission (E), which follows those structures and expands the words into bytes; and the final run-length expansion and checksum (R). The reader (D) then delivers finished blocks to the caller in order ([the parts](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>)). This is the [finite pipeline with feedback-controlled admission](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/#4-the-decoder-drawn-as-stations>) of the queueing post, in which the reader's progress returns the admission slots that let the scanner start new blocks.

## 2\. Every combination of free parts

Each of the six parts has two settings. Real runs the ordinary computation. Ideal replaces it with an oracle, whose correct output was recorded beforehand and is consumed by the stages downstream. Writing \\(p\\) for the number of parts (this post keeps \\(k\\) for workers), six parts give \\(2^6 = 64\\) combinations, a factorial design in the sense of the [companion post's account](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network-notes/#10-experimental-design-and-the-ideal-parts-matrix>). When a later stage is made ideal the earlier ones still run, since otherwise supplying the final output would remove almost the whole decoder and no change could be attributed to one stage.

Making each part free on its own would have missed the central result, which is about combinations: whether construction becomes more worth improving once the scanner has been improved. That dependence is an *interaction*, and revealing interactions is what a factorial design exists to do. The run used 44 of the 64 combinations at each worker count. Thirty-two of them form a half fraction of resolution VI, which estimates every part's effect and every pair's interaction without confusing them with one another; the other twelve are the arms with a single part ideal or a single part real, which that half fraction lacks and which were added, before any data was taken, so that each part's room could be read from a measured arm rather than from a fitted model ([the design](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>); [the registration, clarification 8](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>)).

The worker counts were chosen to straddle the knee. One worker exposes nearly all of the workers' computation in the wall time (the scanner and reader still have threads of their own); six sits below the crossover where the scanner takes over, eight at the balance point and ten above it. Every arm ran the same binary, over twelve randomised blocks of rounds, with the scanner pinned to one core and a 100 ms spin before each decode to keep the processor's clock from dropping; and every arm held the same oracle stores, the all-real baseline included. The run took 2,492 decodes over four and a quarter hours. Every output matched the reference, the scanner's pin held in all 352 traced decodes, and the activities an oracle was meant to remove kept at most 0.19% of their usual busy time ([the results](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)). The findings belong to that setting: the i9-10900K, a repeated Wikidata workload, and stores totalling 495 MiB, which means even the all-real arm ran in a different memory environment from an ordinary decode ([the report](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>)).

## 3\. The scanner and construction, in milliseconds

The four arms that matter most, given as times rather than ratios:

| Workers | All real | Construction free | Scanner free | Both free |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1,306 | 756 | 1,319 | 751 |
| 6 | 243 | 183 | 237 | 143 |
| 8 | 195 | 184 | 189 | 125 |
| 10 | 196 | 186 | 180 | 149 |

The scanner and construction: a complementary pair

Measured: the full ideal-parts matrix, scanner pinned; each point is a geometric-mean wall over blocks, the ratios in the report times the all-real walls. Parallel lines would mean the two parts act independently; at 6 and 8 workers the lines diverge, so each part matters far more once the other is supplied.

Below the crossover construction is plainly the large opportunity: freeing it cuts the decode by 42% at one worker and 25% at six, while freeing the scanner does nothing at one worker. That is a strong reason to study construction, though not a promise that any single improvement to it can capture the whole.

At eight workers the opportunity becomes conditional. Freeing either part alone saves 6 or 11 ms, freeing both saves 70 ms, and the same pattern holds more weakly at six and ten. Put as ratios, freeing construction saves 5.6% of the ordinary decode, but once the scanner is already free it saves 34% of that shorter decode; the denominator changed because the question did. Adding the separately measured savings would badly underestimate the pair, and there is no basis for assigning half of the joint 36% to each part. The factorial interaction, which averages over the settings of the other four parts, is the formal statement of what these four numbers show directly ([the report](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>)).

The scanner and construction are the strongest pair, though not the only route. Freeing the scanner together with the tested part of emission saves about 16% at eight workers with construction still real, and a change of representation might relieve several parts at once. The remaining parts matter in their own ways. Emission's tested oracle saves 16.5% at one worker and 14.9% at six, a figure §4 qualifies. The move-to-front and second run-length stages save 7.5% at one worker and 8.4% at six, smaller than construction but real, and a natural partner for it. The final expansion and delivery show little room on their own in this arrangement, though that could change once the stages before them are relieved.

Adding workers can also hurt even after the obvious work is gone: with the scanner and construction both free, going from eight workers to ten slows the decode from 125 ms to 149 ms. Contention for the shared caches and memory is the plausible explanation, but this run cannot separate the decoder's own contention from the traffic of the oracles' replay stores, so memory bandwidth as a third bottleneck remains a hypothesis ([the design analysis, §6.1](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)).

Of two figures easily misread

The statement that construction "explains 80%" at one worker refers to 80% of the variation in the logarithm of the decode time across the factorial's arms, not to 80% of the running time. The activity timings put construction at 578 ms of a 1,283 ms one-worker decode, about 45% ([stage 1's report](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>)). The same distinction explains how the scanner can account for much of the variation at eight workers while freeing it alone barely moves the baseline: many arms have already made the workers' work cheap, and so expose the scanner. The label UNRESOLVED needs reading with the same care. The registered rule called a part's room ROOM only if its whole confidence interval showed a saving of at least 5%; construction at eight workers saves 5.6%, with an interval of roughly 4.4–6.9%, so it falls just short of the rule rather than showing no effect ([the rules](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>)).

## 4\. What "everything free" left behind

Emission does two kinds of work. It decides which word comes next by following the structure construction built, and it expands each word into bytes. Its oracle supplies the words in output order, which removes the first kind of work but leaves the second, and the decoder's activity ledger charged both the delivery of the supplied words and the expansion that remained to a single entry. Earlier measurements divide that entry ([the design analysis, §3.1](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)):

| Inside emission's oracle entry, at one worker | Time |
| --- | ---: |
| delivering the supplied words | 25.8 ms |
| expanding the words into bytes | 259.4 ms |
| together | 285 ms |

About nine tenths of it is real decoding work rather than copying, although the results first described it as copying, a mistake the review caught. Two consequences follow: emission's 16.5% is not a ceiling on what improving emission could save, and the configuration with every part nominally free still did most of emission's work.

| Workers | Decode time with every part nominally free |
| ---: | ---: |
| 1 | 391 ms |
| 6 | 94 ms |
| 8 | 112 ms |
| 10 | 142 ms |

These times therefore mix the remaining word expansion with the oracles' copying and locking, with contention and with the pipeline's own overhead, so they are not an irreducible floor; nor does subtracting the oracles' times from them give one, because the work overlaps and different arms have different critical paths. A short follow-up run separates the pieces. In its first measurement every oracle hands over its output without copying, but the words are still expanded; in the second the expansion is removed as well, leaving only the cost of the pipeline itself. The design analysis forecasts that second residue at 14–19 ms, an estimate the run will test rather than a measurement, and not a real decompression time, since a real decoder must still compute the answers the oracles supply ([journal §§90–91](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level.md>)). The run's purpose is narrow: to keep experimental copying and leftover emission work from being mistaken for a limit of the decoder. It leaves the scanner–construction finding untouched.

## 5\. Where mathematics could help

New mathematics could help at two scales. The first improves a single component: an order of induction steps that needs fewer of them, a cheaper way to build the dictionary of words, a better way to split a chain of dependent steps. Each such idea has a definite budget, the time of the component it changes.

The second changes the representation itself, so that more of the work is shared or avoided, and this is closer to the programme's aim ([its statement](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/essence.md>)). The present kernel already rests on one such observation: long stretches of the inverse transform move together, so it keeps only some rows, summarises each stretch between them as a word, builds the resulting structure and follows it. A more powerful representation could reduce how many objects construction creates, how many links it wires, how many dependent lookups emission makes, how many words it expands and how much intermediate data is written and read back, so that a single mathematical change could relieve construction and emission at once. The programme names two directions for such a change: *contraction*, which recognises that several operations are the same computation and performs it once, and *enrichment*, which makes existing work yield information that renders later work unnecessary. Either must preserve *closure*: after composition the representation must stay compact and cheap to use, or the saving has merely been moved. The cost model that counts addressed reads, depth of dependent access, passes and stored state exists to stop a candidate winning in one currency while paying in another ([the formulation](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md>)). The matrix adds a practical test: does a proposed reduction touch a large demand, and what companion change lets that reduction reach the output?

## 6\. What each concrete change would touch

Construction's three sub-phases have very different budgets, and the activity timings from the first stage of the characterisation give each its time at one worker.

Construction's budget at one worker

Measured: stage 1, one worker, seconds a decode for the mean worker (A5 132, A6a 200, A6b 216, A6c 161, A7 490, A9 76 ms of 1,280 ms busy; the decode's wall is 1,283 ms). An indicative split: its baseline differs slightly from the matrix's.

The first pass over the block's runs (A6a, 200 ms) classifies them and fills the counts and queues the later phases need; setup, the interval-exchange induction and the dictionary of words follow (A6b, 216 ms); and wiring the graph of retained rows takes the remaining 161 ms (A6c). These figures come from an earlier run with a slightly different baseline, so they are an indicative division of construction's time rather than an exact partition of the matrix's 1,306 ms. Even so they show why "construction has 42% room" cannot mean that a better induction schedule could save 42%.

Of the induction schedule's budget

The induction is the step most open to mathematics, and its cost has a precise count. For every schedule of the steps studied,

\\\[ \\text{steps} = r\_2 - 1 + U, \\\]

where \\(r\_2\\) counts the distinct pairs of adjacent run letters, so that \\(r\_2 - 1\\) is a floor no schedule can beat, and \\(U\\) counts the extra boundaries a particular schedule creates along the way ([the induction record, §5.6](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/monodromy/2026-11-10-rauzy-veech.md>)). The schedule used today takes 1.812 steps for each original run on Wikidata; the best schedule recorded, which takes the steps that create no new boundary first and otherwise the largest cell, takes 1.279; the floor is 1.220.

The induction schedule's budget

Counts from the Rauzy–Veech record §5.6 on all 233 Wikidata blocks; the budget is the review's arithmetic against the matrix's 1,306 ms one-worker wall. The counts are for the earlier full run-letter exchange, not yet the current kernel's object.

The better schedule cuts the number of steps by

\\\[ 1 - \\frac{1.279}{1.812} \\approx 29.4\\%. \\\]

Even if every millisecond of A6b fell in proportion, that would save \\(216 \\times 0.294 \\approx 64\\) ms, about 4.9% of the one-worker decode. A more realistic estimate is

\\\[ \\text{saving} \\approx 216\\ \\text{ms} \\times f\_{\\text{steps}} \\times 29.4\\% - \\text{the new schedule's cost}, \\\]

where \\(f\_{\\text{steps}}\\) is the share of A6b spent on steps a better schedule removes. The project's gate for pursuing the idea is 3% of the decode, about 39 ms, so roughly 62% of A6b would have to be such step time before the new schedule's own cost is paid; measuring that share is the job of a short census ([the census](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>)). Two further questions stand between the counts and the decoder. The counts were taken on the earlier form of the interval exchange over all run letters, whereas the present kernel eliminates thick stretches and keeps thin rows, so the census must count again on the object the decoder actually builds. And a schedule with fewer steps can lose the memory locality and the cheap sweep order of the present one. The 1.220 floor bounds this class of steps; it is not a bound on inverting the transform.

Two concrete designs address other parts of construction's budget, each written with a test that would reject it, and neither measured yet ([the designs, §5](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)). The first would feed construction directly from the move-to-front stage. At present that stage caps runs at 127 and construction then joins them back into maximal runs, so a direct path would skip an intermediate copy of the data (`src/runs.rs`, `src/open.rs`). The second changes the graph's wiring and emission together, replacing links walked in both directions with links walked forward only, several walks at a time, which could simplify A6c and give emission more independent memory accesses to overlap (`src/open/outq.rs`, the emitters in `src/open.rs`). The second design crosses a stage boundary, which is the matrix's lesson in miniature: a different consumer allows a cheaper structure to be built.

The scanner's room calls for a change in where its work is done rather than in how much there is. A pooled scan would find and check the block boundaries first, then hand each worker a compressed block to scan and rebuild on its own, keeping delivery in order (`src/parallel.rs`, `src/huff.rs`). The scanning would still be done, only spread across cores, so the scanner oracle's outright removal of it does not predict what pooling would gain. The design analysis forecasts 130–145 ms for a pooled scan combined with an improved construction, against the 195 ms baseline; the measured 125 ms endpoint makes that a sensible scale to aim at without validating the forecast, and a redesign of that size would move the best worker count as well.

## 7\. What remains to measure, and why it ends

The priorities are settled well enough to begin designing, and each measurement that remains has a single decision to make ([the plan](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/plan.md>)).

| Question | Why its answer changes a decision |
| --- | --- |
| What remains once the oracles' copying and the word expansion are removed? | whether the pipeline's own overhead deserves work, and what the decoder's floor really is |
| How much of A6b is induction steps rather than setup and dictionary work? | whether a better induction schedule can pass its gate |
| Does a proposed representation or pipeline change win across the whole path it touches? | whether to develop that implementation further |

The first is the zero-copy run, nine arms at 1, 6, 7, 8, 9 and 10 workers with every oracle handing over its output without copying, waiting only for those oracles to be built ([\#917](<https://github.com/lmmx/mibz2/pull/917>)); it is a focused follow-up, not a second matrix. The second is the induction census. The third is each design's own rejection test, and the designs for the pooled scan, the folded first pass and forward-only emission can begin now, alongside the two closing measurements. The offline tools in reserve, network calculus for bounds under stated assumptions, a static model of instruction costs, and the surface-dynamics exploration, have bounded purposes and are not prerequisites. One distinction is worth keeping in the record: a walk in several lanes can shorten chains of dependent steps, and so make the decoder much faster, without reducing the number of addressed reads, which would leave the research question about reads open; both outcomes deserve to be written down as what they are.

The matrix has supplied the reason to begin the constructive work: choose a candidate that removes a measured cost, account for what it costs to obtain its new representation, and test the whole path it touches. The floor and sub-phase measurements will sharpen that choice, and then they will stop.

---

**Sources:**

- The review of the experiment, [recorded in full](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>), and [the corrections it made](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/README.md>); journal §91.
- The matrix: [the results](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>), [the report](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>), [the parts](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>), [the design](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>) and [the registration](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>); journal §89.
- Construction's sub-phases: [stage 1's report](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>).
- The induction counts: [the induction record, §5.6](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/monodromy/2026-11-10-rauzy-veech.md>), and [the census](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>).
- The designs and forecasts: [the design analysis](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>); journal §90.
- [The plan](<https://github.com/lmmx/mibz2/blob/97b61ebbe623a1994c3f03aa6ff404d3af93c493/docs/journal/2026-11-24-algorithm-level/plan.md>), and the zero-copy run, [\#917](<https://github.com/lmmx/mibz2/pull/917>).
