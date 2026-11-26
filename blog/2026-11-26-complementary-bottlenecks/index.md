[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Complementary bottlenecks: reading the ideal-parts matrix

At eight workers the scanner and construction must be relieved together, and the matrix is enough to start designing for that.

26 November 2026 · 14 min read · [Research synthesis](<#7-whats-left-to-measure-and-why-it-closes>)

The ideal-parts matrix made each part of mibz2's decoder free in turn, and in combination, to find which computations would have to become cheaper for a mathematical improvement to make decompression faster. Read correctly, its strongest result is a pair: at eight workers, supplying the serial scanner's output or construction's structures alone saves 3–6% of the wall, while supplying both saves 36%, so the two have to be relieved together and the saving cannot be split between them. Below the knee construction is the large opportunity, but it is three sub-phases, and an idea that changes one of them can only spend that one's budget. And the floor the matrix reported is not yet the decoder's, because the oracle for emission left most of emission in place. This post reads the matrix with the help of ChatGPT's review of it ([the review, verbatim](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>)), which corrected several of our own first readings; it follows [the queueing-network post](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/index.md>) and [its companion](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network-notes/index.md>), and does not repeat them.

**Takeaways**

- At 8 workers, making the scanner or construction free alone saves 3–6%; both together save 36%, from 195 to 125 ms, 1.56 times the throughput. They are a complementary pair, and the saving cannot be split between them.
- Below [the knee](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/#5-the-bottleneck-law>), construction is the big room (−42% at one worker), but it is three sub-phases, and any one idea has only its own sub-phase's budget.
- Emission's oracle left most of emission in place, so the decoder's floor is not yet measured; a follow-up run ([\#917](<https://github.com/lmmx/mibz2/pull/917>)) measures it.
- The matrix is enough to start designing. The measurements that remain each decide one thing and then close.

## 1\. Three questions, and the one the matrix answers

Three questions about a decoder's performance are easy to run together.

| Question | What answers it | What it tells you |
| --- | --- | --- |
| Where does the time go? | profiling, activity timings | how much work there is, and where |
| What controls the wall now? | scaling runs, interventions that slow an activity | which constraints bind in this configuration |
| What would be worth making cheaper? | making parts ideal, alone and together | the room an improvement could have, including room hidden behind another bottleneck |

They differ because the decoder overlaps its work. If the scanner and the workers could each get through the file in about 185 ms, making either much faster would leave the other still taking 185 ms, while slowing either would slow the whole. So sensitivity to a slowdown does not measure the gain from a speedup. The earlier experiments answered the second question, which stage controls the wall at which worker count ([§11 of the queueing post](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network/#11-testing-the-explanation-by-intervention>)); the matrix was built to answer the third, because that is the one that chooses targets for mathematics ([why the matrix](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/README.md>)).

The decoder it studies has six parts, which the matrix names by letter.

The decoder's six parts

Schematic, after the review's diagram. Each letter is one part of the ideal-parts matrix; the dashed line is the admission credit the in-order reader returns.

The scanner (S) parses each block's coding tables and decodes its [Huffman symbols](<https://lmmx.github.io/mibz2/blog/2026-11-05-huffman-bit-buffer/index.md>), on one thread. Each worker then takes one block through four stages: move-to-front and the second run-length stage (M), which turn the symbols into the block's last column; construction (C), which builds the structures for undoing the block sort, its retained rows, [interval-exchange](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) reductions, word descriptions and graph links; emission (E), which follows those structures and expands their word codes into bytes; and the final run-length expansion and checksum (R). The reader (D) delivers blocks to the caller in order ([the parts](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>)). This is the finite pipeline with feedback-controlled admission described in the queueing post, and the hybrid-systems description fits it: between events, threads make progress at rates set by contention; at events, blocks are admitted, dispatched, finished and taken ([the formulation](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md>)). That description is a modelling proposal, not a validated model, and it need not become a prerequisite for the work the matrix points to.

## 2\. A \\(2^p\\) design over oracles

Each of the six parts has two settings. Real runs the ordinary computation; ideal supplies a correct intermediate result recorded beforehand, prepared outside the timed decode, which the downstream stages consume. Writing \\(p\\) for the number of factors (the post keeps \\(k\\) for workers), six parts give \\(2^6 = 64\\) combinations. Upstream stages still run when a downstream one is ideal; otherwise supplying the final output would remove almost the whole decoder, and no change could be attributed to one stage.

Testing each part alone would have missed the central result. The combinations ask whether construction becomes more worth improving once the scanner is improved, which is an interaction, and an interaction is what a factorial design exists to reveal. The run used 44 combinations at each worker count: 32 forming a half fraction of resolution VI, which estimates main effects and two-part interactions without confusing them with each other, and the 12 with a single part ideal or a single part real, which the half fraction lacks and which were added before any data so that the headline room could be read from measured arms rather than a fitted model ([the design](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>); [clarification 8](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>); [the companion's account](<https://lmmx.github.io/mibz2/blog/2026-11-25-the-decoder-as-a-queueing-network-notes/#10-experimental-design-and-the-ideal-parts-matrix>)).

The worker counts were 1, 6, 8 and 10: one worker exposes most of the workers' computation directly in the wall (the scanner and reader are still separate threads), six lies below the crossover, eight at the balance point and ten above it. The same binary ran every arm, with twelve randomised blocks of rounds, the scanner pinned, and a 100 ms spin before each decode to keep the clock high; every arm held the same oracle stores, the all-real baseline included. The run took 2,492 decodes over 4 h 15 min, every output matched, the pin held in all 352 traces, and the removed activities' remaining busy time was 0.00–0.19% of their real time ([the results](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)). Its scope is the i9-10900K, the repeated Wikidata workload and this arrangement: the stores took 495 MiB, so even the all-real arm ran in a different memory environment from an ordinary decode ([the report](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>)).

## 3\. The scanner and construction, in milliseconds

The four arms that matter most, as times rather than ratios:

| Workers | All real | Construction supplied | Scanner supplied | Both supplied |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1,306 | 756 | 1,319 | 751 |
| 6 | 243 | 183 | 237 | 143 |
| 8 | 195 | 184 | 189 | 125 |
| 10 | 196 | 186 | 180 | 149 |

The scanner and construction: a complementary pair

Measured: the full ideal-parts matrix, scanner pinned; each point is a geometric-mean wall over blocks, the ratios in the report times the all-real walls. Parallel lines would mean the two parts act independently; at 6 and 8 workers the lines diverge, so each part matters far more once the other is supplied.

Below the crossover construction is the large opportunity: supplying it cuts the wall by 42% at one worker and 25% at six, with the scanner's supply doing nothing at one worker. That is a strong reason to study construction, though not a promise that any one improvement to it can capture the whole.

At eight workers the opportunity is conditional. Either part alone saves 6 or 11 ms; together they save 70 ms. Construction supplied by itself saves 5.6% of the ordinary wall; supplied once the scanner is already supplied, it saves 34% of that smaller wall. The denominator changed because the question did. Adding the separately measured savings would badly underestimate the pair, and there is no basis for assigning half of the joint 36% to each: the result describes a complementary pair, and the factorial interaction, which averages over the other parts' settings, is the formal statement of what these four numbers show directly ([the report](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>)).

Scanner and construction is the strongest pair, not the only route. Supplying the scanner and the tested emission work saves about 16% at eight workers with construction still real, and a change of representation could touch several parts at once. The other single parts matter in their own way. Emission's tested intervention saves 16.5% at one worker and 14.9% at six, with a correction that §4 explains. Move-to-front and the second run-length stage save 7.5% at one worker and 8.4% at six, smaller than construction but real, and a natural partner for it. The final run-length work and delivery show little room alone in this arrangement, which could change once upstream constraints are relieved.

And adding workers can hurt even with the obvious work removed: with the scanner and construction both supplied, going from eight workers to ten slows the decode from 125 to 149 ms. Shared cache and memory contention is the plausible explanation, but this run cannot separate the decoder's own contention from the traffic of its replay stores, so a third bottleneck in memory bandwidth remains a hypothesis ([Antigravity's second pass, §6.1](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)).

Of two phrases that mislead

"Construction explains 80%" means 80% of the variation in log wall across the factorial's arms at one worker. It does not mean construction takes 80% of the runtime: the activity timings put construction at 578 ms of a 1,283 ms one-worker decode, about 45% ([stage 1's report](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>)). The same distinction explains how the scanner can carry much of the variation at eight workers while relieving it alone changes the baseline only a little: many arms have already made the workers' work cheap, which exposes the scanner. And "UNRESOLVED" is a label, not a shrug. The registered ROOM label needed the whole interval to show a saving of at least 5%; construction at eight workers saves 5.6%, with an interval of about 4.4–6.9%, so it misses that threshold. No cell met the full MASKED rule either, which needs essentially no effect alone; the scanner–construction interaction is clear all the same ([the rules](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>)).

## 4\. What "all ideal" left behind

Emission does two kinds of work: it decides which word code comes next by following the reconstruction graph, and it expands that code into bytes. Its oracle supplies the codes in output order, which removes the graph walk but still expands the words. Both the delivery of codes and the expansion that remains are charged to the ledger entry the report calls emission's handover, and earlier measurements split it ([Antigravity's second pass, §3.1](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)):

| Inside emission's "handover", at one worker | Time |
| --- | ---: |
| delivering the supplied codes | 25.8 ms |
| expanding the codes into bytes | 259.4 ms |
| together | 285 ms |

About nine tenths of it is real decoder work, not copying; the results first called it copying, and the review corrected that. Two things follow. Emission's 16.5% is not a ceiling on what improving emission could save, and the configuration with every part ideal still did a large share of emission.

| Workers | Wall with every part nominally ideal |
| ---: | ---: |
| 1 | 391 ms |
| 6 | 94 ms |
| 8 | 112 ms |
| 10 | 142 ms |

These times mix the remaining word expansion, the replay copies, the oracles' locking, contention and the pipeline's own overhead, so they are not an irreducible floor, and subtracting handover times from them does not give one either, because work overlaps and different arms have different critical paths. The follow-up separates two quantities. Residual A uses zero-copy stand-ins with the word expansion still done; Residual B removes the expansion too. Antigravity forecasts Residual B at 14–19 ms, an argued estimate the run will test, not a measurement, and not a real decompression time: a real decoder must still acquire the answers ([the zero-copy build](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/antigravity/prompt-build-zero-copy.md>)). Its purpose is bounded: to stop experimental copying and retained emission work being mistaken for a limit of the decoder. It does not touch the scanner–construction finding.

## 5\. Where mathematics could help

Mathematical work could help at two scales. The first improves one component: an induction order with fewer steps, cheaper construction of dictionary entries, a better way to split a dependency chain. Each has a specific budget, the time of the component it changes.

The second changes the representation so that more work is shared or avoided, which is closer to the programme's aim ([the aim](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/essence.md>)). The current inversion kernel already exploits one observation: stretches of the inverse transform move together, so it keeps some rows, summarises the stretches between them as words, builds the resulting structure and follows it. A more powerful representation could reduce how many objects construction creates, how many links it wires, how many dependent lookups emission makes, how many words it expands and how much intermediate data is written and read again, so one mathematical change could relieve construction and emission together. The two directions the programme names are contraction, recognising that several operations are the same computation and composing them once, and enrichment, making existing work yield information that makes later work unnecessary. Both must keep closure: after composition the representation must stay compact and cheap to use, or the saving has only moved. The cost model that counts addressed reads, dependent depth, passes and stored state keeps a candidate from winning in one currency while paying in another ([the formulation](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md>)). The matrix supplies the practical test: does a proposed reduction touch a large demand, and what companion change lets it reach the output?

## 6\. What each concrete change would touch

Construction is three sub-phases, and stage 1's activity timings at one worker give each its time.

Construction's budget at one worker

Measured: stage 1, one worker, seconds a decode for the mean worker (A5 132, A6a 200, A6b 216, A6c 161, A7 490, A9 76 ms of 1,280 ms busy; the decode's wall is 1,283 ms). An indicative split: its baseline differs slightly from the matrix's.

The first pass over runs (A6a, 200 ms) classifies them and fills the counts and queues; setup, interval-exchange induction and the dictionary of words (A6b, 216 ms) follow; and wiring the retained-row graph takes 161 ms (A6c). They come from stage 1, whose baseline differs slightly from the matrix's, so they are an indicative split rather than an exact partition of its 1,306 ms. That alone explains why "construction has 42% room" does not mean a better induction schedule could save 42%.

Of the induction schedule's budget

The induction steps have a precise count. For every schedule of the cell steps studied,

\\\[ \\text{steps} = r\_2 - 1 + U, \\\]

where \\(r\_2\\) counts the distinct digrams of run letters, so \\(r\_2 - 1\\) is the floor, and \\(U\\) counts the extra boundaries a schedule creates on the way ([the record, §5.6](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/monodromy/2026-11-10-rauzy-veech.md>)). Today's order makes 1.812 steps per original run on Wikidata; the best recorded order, free steps first and otherwise the largest cell, makes 1.279; the floor is 1.220.

The induction schedule's budget

Counts from the Rauzy–Veech record §5.6 on all 233 Wikidata blocks; the budget is the review's arithmetic against the matrix's 1,306 ms one-worker wall. The counts are for the earlier full run-letter exchange, not yet the current kernel's object.

The known order cuts the step count by

\\\[ 1 - \\frac{1.279}{1.812} \\approx 29.4\\%. \\\]

If every millisecond of A6b fell in proportion, that would save \\(216 \\times 0.294 \\approx 64\\) ms, about 4.9% of the one-worker matrix wall. More realistically,

\\\[ \\text{saving} \\approx 216\\ \\text{ms} \\times f\_{\\text{steps}} \\times 29.4\\% - \\text{the new schedule's cost}, \\\]

with \\(f\_{\\text{steps}}\\) the share of A6b spent on steps that a better order removes. The written gate is 3% of the wall, about 39 ms, so roughly 62% of A6b would have to be such step time before paying for the schedule; the census measures that share and decides it ([the census](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>)). Two transfer questions remain: the 1.812 and 1.279 counts are for the earlier full run-letter interval exchange, while the current kernel eliminates thick strips and keeps thin rows, so the census must recount on the actual object; and a lower-step order can lose the locality and cheap sweep order the present one has. The 1.220 floor is the floor of this class of steps, not a bound on inverting the transform. The surface-dynamics exploration behind it is an offline instrument: it may search expensive alternatives, but what ships must be a cheap rule decided from information construction already has.

Two concrete designs from Antigravity address other parts of construction's budget, both proposals with explicit kill tests and no measured gain yet ([its designs, §5](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>)). One feeds construction directly from the move-to-front stage: the current interface caps runs at 127 and construction then recombines them into maximal runs, so a direct path would skip that materialisation (`src/runs.rs`, `src/open.rs`). The other changes the graph wiring together with emission, replacing bidirectional links with forward ones walked several at a time, which could simplify A6c's wiring and give emission more independent memory accesses (`src/open/outq.rs`, the emitters in `src/open.rs`). The second crosses the stage boundary, which is the matrix's lesson in small: a different consumer lets a cheaper structure be built.

The scanner's room is a matter of where work is placed. A pooled scan would find and check block boundaries, hand workers compressed blocks, and let each worker scan and reconstruct its own, keeping in-order delivery (`src/parallel.rs`, `src/huff.rs`). The scan would still be done, only spread across cores, so the scanner oracle's deletion of the work does not predict its gain. Antigravity forecasts 130 to 145 ms for a pooled scan with construction improved, against the 195 ms baseline; the measured 125 ms endpoint makes that a sensible scale to aim at, but does not validate the forecast, and a redesign of that size moves the best worker count too.

## 7\. What's left to measure, and why it closes

The broad priorities are settled enough to start designing, and each remaining measurement has one decision to make ([the plan](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/plan.md>)):

| Question | Why the answer changes a decision |
| --- | --- |
| What remains once replay copying and word expansion are taken out? | says whether the pipeline's overhead deserves work, and corrects the apparent floor |
| How much of A6b is induction steps rather than setup and dictionary work? | says whether a better induction order can pass its gate |
| Does a proposed representation or pipeline change win as a complete path? | says whether to develop that implementation |

The first is the zero-copy floor run, nine arms at 1, 6, 7, 8, 9 and 10 workers with every handover zero-copy, held until the zero-copy oracles are built ([\#917](<https://github.com/lmmx/mibz2/pull/917>)); it is a focused follow-up, not another 44-arm matrix. The second is the induction census. The third is each design's own kill test, and the designs for the pooled scan, the folded first pass and forward emission may start now, alongside the two closing measurements. The offline tools, network calculus for bounds under stated assumptions, uiCA for local instruction costs, and surface dynamics for structure, are instruments with bounded purposes, not prerequisites. One distinction is worth keeping in the record: a multi-lane walk can shorten dependency chains, and make the decoder much faster, without reducing the number of addressed reads, leaving the research question about reads open; both outcomes should be written down as what they are.

The matrix supplied the reason to do the constructive work: choose a candidate that removes a measured cost, account for what it costs to obtain its new representation, and test the whole path it touches. The floor and sub-phase measurements will sharpen that choice, and then close.

---

**Sources:**

- The review: [ChatGPT's review of the ideal-parts work, verbatim](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>), and [the corrections it made](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/README.md>); journal §91.
- The matrix: [the results](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>), [the report](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>), [the parts](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>), [the design](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>) and [the registration](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>); journal §89.
- Construction's sub-phases: [stage 1's report](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>).
- The induction counts: [the Rauzy–Veech record, §5.6](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/monodromy/2026-11-10-rauzy-veech.md>); [the census](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md>).
- The designs and forecasts: [Antigravity's second pass](<https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md>), at 7fab624e on `research/neon-movement`; journal §90.
- [The plan](<https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/plan.md>), and the zero-copy floor run, [\#917](<https://github.com/lmmx/mibz2/pull/917>).
