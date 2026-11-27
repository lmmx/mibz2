[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/index.md>)

# Complementary bottlenecks: reading the ideal-parts matrix

Making the parts of mibz2's decoder free in turn and in combination shows where new algorithms would pay off and which two stages are worth far more together.

26 November 2026 · 21 min read · [Research synthesis](<#8-from-characterisation-to-invention>)

A bzip2 file is decoded by undoing, block by block, a sorting of the text called the Burrows–Wheeler transform. For most of its history that undoing has been a walk: one row of the sorted table at a time, each step a lookup that depends on the one before. This month mibz2 found [a different way back](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-10-rauzy-veech-reconstruction/index.md>). Adjacent rows that hold the same byte take the same step, so a whole stretch of them can be moved at once, and the classical tool for doing that, the induction that mathematicians use to study interval exchanges, rebuilds the text from a few retained rows and the words between them. It was a mathematical invention, not a tuning, and it raised a question that tuning cannot answer: where in the decoder could another invention of that kind make a real difference?

That question is harder than it sounds, because a decoder is not a list of costs. mibz2 decodes on many cores at once: one thread scans each compressed block's Huffman codes, a pool of workers rebuilds the blocks' text in parallel, and a reader hands the blocks back in order. The stages overlap, they share caches and memory, and they hold one another back. A stage that takes a large share of the running time may be worth little to improve, because another stage would take over as the limit; a stage that looks cheap may be worth a great deal once something else is relieved. Which algorithm is worth inventing therefore depends on the whole system, and we treated it as a system: in the spirit of systems biology, which studies a living system as a network of interacting parts through quantitative models fitted to measurement, and with the experimental method of Raj Jain's course on [computer systems performance analysis](<https://www.cs.wustl.edu/~jain/iucee/index.html>), which teaches how to design such interventions so that a few runs reveal how many factors combine.

This post follows that work: the decoder as a system and the levers for reorganising it; the experiment that measured its counterfactual limits; what that found, including a pair of stages whose value appears only together and a residue it could not remove; and what mathematics could change. How the resulting algorithms would work is the subject of a technical companion.

**Takeaways**

- Below the knee, where adding a worker still speeds the decode up, building the inversion's structures (construction) is the largest opportunity: 42% of a one-worker decode.
- At eight workers the scanner and construction are complementary: supplying either alone saves 3–6%, supplying both 36% (195 → 125 ms). An improvement measured alone can look worthless because the system cannot yet use it.
- The stand-in for emission still expanded every word into bytes, so the decoder's true floor is still to be measured.
- New mathematics can do less work (contraction) or make existing work yield more (enrichment); four candidate algorithms follow, none yet built.

**A technical companion** develops the machinery: the interaction coefficients and their aliasing, the accounting of what the oracles left behind, the four cost currencies, and the four candidate algorithms with their forecasts, costs and rejection tests. [From bottlenecks to better algorithms →](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/index.md>)

## 1\. Where could another mathematical breakthrough matter?

The programme behind this post set itself a large aim: not another few percent, but a decoder that undoes the block sort with fundamentally less work ([its statement](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/essence.md>)). The interval-exchange kernel showed that such things exist. It came from a change of viewpoint, from following one row at a time to moving whole stretches of rows that behave alike, and nothing in a profile of the old walk would have suggested it. A second invention of that kind will not be found by measuring either. But measurement can say where one would pay, and so where to look, and that is the purpose of everything in this post.

The programme laid out the path as a chain ([the programme](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/programme.md>)): fix the ambition and its referee; characterise the system, how computation, concurrency and memory combine into elapsed time; measure counterfactuals, what would be gained if particular work became cheap; read the results as mathematical targets; and only then invent. The first two steps are the subject of [Posing the inversion problem](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-24-posing-the-inversion-problem/index.md>) and [The decoder as a queueing network](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-25-the-decoder-as-a-queueing-network/index.md>). This post is about the third and fourth.

Three questions about a decoder's speed are easily confused, and each needs a different experiment.

| Question | What answers it | What it tells you |
| --- | --- | --- |
| Where does the time go? | profiling, activity timings | how much work there is, and where |
| What limits the decode now? | scaling runs, and slowing one activity at a time | which constraints bind in the present configuration |
| What would be worth making cheaper? | making parts free, alone and together | the room an improvement could have, including room hidden behind another bottleneck |

The second and third differ because the decoder overlaps its work: if the scanner and the workers each needed 185 ms for a file, slowing either would slow the decode, yet speeding either would leave the other still taking 185 ms. The [earlier experiments](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-25-the-decoder-as-a-queueing-network/#11-testing-the-explanation-by-intervention>) answered the second question; choosing what to invent needs the third.

## 2\. From an algorithm to a dynamical system

The decoder's work divides into six parts, each named by a letter ([the parts](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>)).

The decoder's six parts

Schematic. Each letter is one part of the experiment in §3; the dashed line is the admission credit that the in-order reader returns to the scanner.

The scanner (S) reads each block's coding tables and decodes its [Huffman symbols](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-05-huffman-bit-buffer/index.md>) on a thread of its own. A worker then takes the block through four stages. The move-to-front and second run-length stages (M) turn the symbols back into the block's last column. Construction (C) builds the structures for undoing the block sort: the retained rows, the interval-exchange reductions between them, the words those reductions spell and the links that join them. Emission (E) follows those structures and expands the words into bytes, and the final run-length expansion and checksum (R) finish the block. The reader (D) delivers finished blocks to the caller in order, and as it takes each one it returns an admission slot that lets the scanner start another, so the pipeline regulates itself through a feedback loop.

Described this way, the decoder is a *hybrid dynamical system*: between events, each thread makes progress at a rate set by its work and by contention for the shared caches and memory; at events, blocks are admitted, taken, finished and handed over. The description earns its keep because it lists exactly what a redesign can change ([the levers](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md>)).

| What can change | Example in mibz2 |
| --- | --- |
| **Flow rates:** how fast each activity runs | a faster Huffman decode; a cheaper induction |
| **Where work may proceed,** and where a thread must wait | the admission window; the in-order reader |
| **When events may happen** | when a worker may take a block; when the scanner may admit one |
| **What an event does** | which worker takes a block; how a block is routed |
| **Who owns the information** | the buffers between scanner, workers and reader |
| **The state space itself** | interval-exchange induction in place of the row walk |

The last row is the one this post cares most about. The first five reorganise an existing computation: they make it faster, move it, or schedule it differently. The last changes what the computation consists of. The interval-exchange kernel was such a change, and the search this post serves is for another. The experiment described next works mostly through the first lever, by pushing chosen demands to zero, but its results can motivate a change to any of them, and the proposals in §7 pull several at once. The formal version, with what each lever risks, is in [the companion's first section](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#1-the-formal-design-space>).

The queueing view of the same system gives the vocabulary for its limits. The scanner is a single server; the workers form a pool; the decode's throughput is bounded by whichever saturates first. From one to four workers that bound predicted the *knee*, the worker count past which adding workers stops helping, at 7.49 workers; the measured knee was 7.67 ([the bottleneck law](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-25-the-decoder-as-a-queueing-network/#5-the-bottleneck-law>)). Eight workers, just past it, is now mibz2's default on the test machine. Below the knee the workers set the pace; above it the scanner does.

## 3\. Measuring a counterfactual

Profiling measures what the decoder does. To learn what it *would* do if some part became cheap, the part has to be made cheap. For each of the six parts we built an *oracle*: a stand-in that hands the decoder that part's correct output, recorded in an earlier run, so that the part's own computation is skipped while everything downstream proceeds as usual. Each part then has two settings. *Real* runs the ordinary computation; *ideal* replaces it with the oracle. When a later stage is made ideal the earlier ones still run, so that the effect of each part can be attributed to it alone.

**An oracle is not free.** Copying a recorded answer into the decoder still takes time and memory bandwidth. The experiment timed each oracle's handover in its own ledger entry, so its cost could be read beside the result, and §6 returns to what that cost hid.

Six parts with two settings each give \\(2^6 = 64\\) configurations. Making each part ideal on its own would have answered only half the question. The central issue is whether parts *interact*: whether construction becomes more worth improving once the scanner has been improved. An interaction is a property of combinations, and the method for measuring it is the *factorial design*, the core of Jain's experimental method and the subject of the [queueing companion's tenth section](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-25-the-decoder-as-a-queueing-network-notes/#10-experimental-design-and-the-ideal-parts-matrix>). A well-chosen half of the 64 answers almost everything the whole would.

The half is chosen by a rule called the *defining relation*, here \\(I = +SECMRD\\). Code each part as \\(-1\\) when real and \\(+1\\) when ideal; the rule keeps the configurations in which the product of all six codes is \\(+1\\), which are exactly those with an even number of parts ideal: none, two, four or all six. That gives 32 configurations in which no part's effect, and no pair's interaction, is confounded with another; what remains confounded are interactions among three or more parts, which are less likely to matter. A design with this property is said to have *resolution VI*, and [the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#2-quantifying-complementary-bottlenecks>) works through what is confounded with what, and how the effects are computed.

How the 64 configurations were sampled

Design, not data: the experiment's registration, clarification 8. Each square is one configuration of real and ideal parts; hover for which. The half fraction keeps the even rows; the added rows are the configurations with one part ideal or one part real; the three-ideal row was not run. The fraction separates every main effect and every two-part interaction from one another; three-part interactions stay confounded in pairs.

The half fraction has a gap: it contains no configuration with exactly one part ideal, so it estimates each part's average effect over many settings of the others but never measures today's decoder with one part made free. Before any data was taken, the twelve configurations with one part ideal or one part real were added, making 44 at each worker count ([the registration, clarification 8](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>)). They give each part's *room*: the decode time with that part alone ideal over the time with every part real.

The worker counts straddle the knee: one, six, eight and ten. The run took 2,492 decodes over four and a quarter hours on an Intel i9-10900K and a repeated Wikidata workload, every output byte-identical to the shipped decoder's ([the results](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)); the controls, and the rule registered in advance for labelling each room ROOM, NO ROOM or UNRESOLVED, are in [the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#2-quantifying-complementary-bottlenecks>). One condition belongs in every reading: each configuration held all the oracles' recorded answers in memory, 495 MiB of them, so even the all-real configuration ran in a different memory environment from an ordinary decode.

## 4\. A map of the remaining opportunity

The whole of the experiment's first result fits in one table: each part made ideal alone, at each worker count, as the percentage by which it shortens the decode.

Elapsed-time reduction when each part alone is made ideal

Measured: the full matrix, scanner pinned; each cell is one minus the room, with its 95% interval beneath and the registered label where it is ROOM or NO ROOM (unmarked cells are UNRESOLVED). Positive values are faster. ¹ Emission's oracle removes the walk through construction's structures, not the expansion of words into bytes (§6), so its row understates emission's room.

Several things are plain at once. Construction's opportunity is large below the knee and shrinks as workers are added: 42.1% of a one-worker decode, 24.6% at six, but only 5.6% at eight. The scanner's moves the other way, from nothing at one worker to 8.1% at ten, the only ROOM label above the knee. Emission holds a substantial opportunity below the knee, 16.5% and 14.9%, and the table understates it, for the reason §6 explains. The move-to-front and second run-length stages are smaller but real, 7.5% at one worker and 8.4% at six, and they sit directly upstream of construction. The final run-length expansion and delivery show little room of their own. The last stages are not where the next large gain will come from.

The factorial analysis reads the same data a second way, dividing the variation in the logarithm of the decode time among the parts, and the two readings are easily confused. Construction "explains" 80% of that variation at one worker, but it takes about 45% of the running time; at eight workers the scanner explains 46% of the variation, yet supplying it alone saves 3.2%. Variation measures a part's average importance across many hypothetical decoders, room its value in today's ([the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#2-quantifying-complementary-bottlenecks>)).

**UNRESOLVED is not "no effect".** A part has ROOM when its whole 95% interval shows a saving of at least 5%. Construction at eight workers saves 5.6%, with an interval of 4.4–6.9%: a real, modest saving that misses the label.

What the table cannot show is how the parts depend on one another. At eight workers every part's room is small, and a reader of this table alone would conclude that the decoder has little left to give there. Two-part interactions carry about a fifth of the variation from six to ten workers, and one of them changes that conclusion entirely.

## 5\. Complementary bottlenecks

The interaction that dominates the experiment is between the scanner and construction. Its four configurations, as times rather than ratios:

| Workers | All real (ms) | Construction supplied (ms) | Scanner supplied (ms) | Both supplied (ms) |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1,306 | 756 | 1,319 | 751 |
| 6 | 243 | 183 | 237 | 143 |
| 8 | 195 | 184 | 189 | 125 |
| 10 | 196 | 186 | 180 | 149 |

At eight workers, supplying the scanner saves 6 ms and supplying construction saves 11 ms. Supplying both saves 70 ms, taking the decode from 195 ms to 125 ms: 1.56 times the throughput. The two parts are *complementary bottlenecks*: each holds the decode back as soon as the other is relieved, so neither shows its value alone. The registered analysis finds their interaction strongest at six and eight workers and indistinguishable from nothing at one ([the coefficients](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#2-quantifying-complementary-bottlenecks>)).

The scanner and construction, alone and together

Measured: each point is the decode time over the all-real time at the same worker count, on one shared logarithmic scale; hover for milliseconds. On a log scale, parallel lines mean the two parts' effects multiply independently, which is the registered analysis's sense of "no interaction". At one worker the lines are parallel; at six and eight they diverge, so each part matters far more once the other is supplied.

The interaction changes what a discovery is worth. Suppose a mathematical idea made construction free. Measured in today's decoder at eight workers, it would save 5.6%, a result easily filed as disappointing. Measured in a decoder whose scanner had been relieved, the same idea would save 33.9% of the time that remained. The idea is the same in both cases; the system around it is not.

How much of the decode construction can remove

Measured: the share of the decode removed by supplying construction, with the ordinary scanner and with the scanner supplied. Each percentage is relative to its own starting configuration, so the walls are given beside it: at eight workers, 195 → 184 ms against 189 → 125 ms.

**A disappointing whole-decoder benchmark is therefore not evidence that an algorithmic improvement lacks value.** It may be evidence that the system cannot yet exploit it. The converse is a research direction in its own right: mathematical ideas that produced excellent local results and little whole-decoder gain may deserve another look once the scanner has been restructured. Nor is the pair the only route: supplying the scanner with emission's oracle saves about 16% at eight workers, and a change of representation could relieve several parts at once ([the results, §5](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)). What the experiment establishes is that at eight workers relieving the scanner and construction together exposes a far larger opportunity than either alone; it does not establish that no other combination could.

A simple model makes the pattern intuitive. Suppose the scanner must do \\(S\\) milliseconds of work on a file, and the workers \\(W\\) milliseconds between them, shared among \\(k\\) workers. If the two stages overlap perfectly, the elapsed time is set by whichever is slower:

\\\[ T \\approx \\max\\left(S,\\ \\frac{W}{k}\\right). \\\]

If a change removes a fraction \\(a\\) of the scanner's work and \\(b\\) of the workers', the time becomes \\(\\max\\big(S(1-a),\\ W(1-b)/k\\big)\\). When the two are balanced, as they nearly are at eight workers, the reduction is simply the smaller of \\(a\\) and \\(b\\): removing scanner work alone gains nothing until the workers' share is removed too, and the other way round.

Complementary bottlenecks in a two-station model

Illustrative two-station model, not a fitted prediction of mibz2, with the scanner and the workers balanced at 185 ms. The shading is the model's reduction in elapsed time; its contours are L-shaped, so only the diagonal gains. The four points are the measured eight-worker configurations, with construction placed at 45% of the workers' work, its share in the activity timings.

The model reproduces the shape of the measurement, but it overstates the pair, predicting 45% where 36% was measured. It treats the scanner and the workers as independent, yet they share caches and memory bandwidth, and with both parts supplied, going from eight workers to ten *slows* the decode from 125 ms to 149 ms. Memory as a third station is the natural repair, still a hypothesis, since this experiment cannot separate the decoder's own contention from the traffic of the oracles' stores; [the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#2-quantifying-complementary-bottlenecks>) sets it out.

Measured on 26 November, the zero-copy floor run found no sign of a memory-bandwidth third station between eight and ten workers. Memory traffic stayed at 13–17 GB/s, far below the machine's bandwidth. The earlier rise to 142 ms was the old oracles' copying ([§6](<#6-what-the-experiment-could-not-remove>); [the true-floor post](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-true-floor-and-pooled-scan/#1-how-much-of-the-first-matrixs-floor-was-the-decoders>)).

## 6\. What the experiment could not remove

The configuration with every part ideal should reveal what remains of the decoder when all of its work has been supplied: its floor. It took 391 ms at one worker, 94 ms at six, 112 ms at eight and 142 ms at ten ([the results, §4](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>)). Those are not floors, for two reasons, and the first concerns emission.

Emission does two kinds of work. It decides which word comes next, by following the structure that construction built, and it expands each word into bytes. Its oracle supplies the words in output order, which removes the first kind of work but not the second, yet the activity ledger charged the delivery and the remaining expansion to one entry, first read as the oracle's copying cost. The design analysis, a second pass over the results, found the mistake by reading the oracle's code ([the design analysis, §3.1](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>)), and an independent review verified it and drew its consequences ([the review, §4](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>)).

| Inside emission's oracle entry, at one worker | Time (ms) |
| --- | ---: |
| delivering the supplied words | 25.8 |
| expanding the words into bytes | 259.4 |
| together | 285.2 |

About nine tenths of the entry is real decoding work. So emission's 16.5% is not a ceiling on what improving emission could save, and the configuration with every part nominally ideal still did most of emission's work.

The second reason is the oracles' own cost. At one worker the workers' handovers, emission's word expansion included, account for 377 ms of the all-ideal configuration's 391 ms. At eight and ten workers, many threads copy large recorded buffers at once, and the all-ideal wall *rises* from 112 ms to 142 ms while each thread's handover barely changes, which is what contention for memory bandwidth would produce. Neither effect is the decoder's.

A short follow-up run is designed to separate the pieces ([the floor run](<https://github.com/lmmx/mibz2/pull/917>)). Its oracles hand over their answers without copying, and it runs two floors: one in which the words are still expanded, and one in which the expansion is removed as well, leaving only the cost of the pipeline itself: its queues, locks and thread hand-offs. The design analysis forecasts that residue at roughly 12 to 19 ms at six to ten workers, a forecast for the run to test rather than a measurement, and not a decompression time, since a real decoder must still compute the answers the oracles supply; the arithmetic is in [the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#3-accounting-for-the-remaining-computation>). The run's purpose is narrow, to keep experimental copying and leftover emission work from being mistaken for a limit of the decoder, and it leaves the scanner–construction result untouched.

Measured on 26 November, the zero-copy floor run put the pipeline alone at 5.2–5.7 ms across one to ten workers, with every part supplied and the words not expanded. At eight workers it took 5.2 \[5.0, 5.5\] ms, which is 2.7% of the shipped 194.6 ms decode. That is about a third of the design analysis's 12–19 ms forecast, kept above beside the measurement as registered before the data. Under the run's frozen rule, the forecast scored a miss on the low side, closing pipeline plumbing as a target. With word expansion kept and oracle copying removed, the decode took 42.4 ms at eight workers. It did not rise towards ten (42.9 ms at nine, 40.5 ms at ten), confirming that the climb to 142 ms was the old oracles' copying. Full figures and intervals are in [the floor results](<https://github.com/lmmx/mibz2/blob/311923efd5ac7b113a9f944db34f41172c096816/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md>), [the true-floor post](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-true-floor-and-pooled-scan/#1-how-much-of-the-first-matrixs-floor-was-the-decoders>) and [the companion's table](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#3-accounting-for-the-remaining-computation>).

## 7\. What mathematics could change

The experiment says where time could be saved. It does not say how. A part can be made cheaper by doing its present computation faster, and that is engineering; or by finding a different computation with the same result, and that is where mathematics enters. The programme works with two principles for finding such computations, and a third idea that constrains them both ([the programme's statement](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/essence.md>); [What else moves together](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-24-what-else-moves-together/index.md>)).

### Contraction: changing the unit of computation

The interval-exchange kernel rests on one observation. When the decoder undoes the block sort, each row of the sorted table maps to another by a rule called LF, and adjacent rows holding the same byte map to adjacent rows. So a run of equal bytes can be moved as one interval rather than row by row, and an induction that repeatedly composes these interval moves, and recurses on what is left, rebuilds the text from far fewer steps than there are rows.

Contraction: one step for a run instead of one for each row

Schematic, not measured counts: a ten-row last column, L = bbaaacaabb, with LF computed exactly. On the left each row has its own step; on the right each run of equal bytes maps to a contiguous interval of the sorted column in one step. The vertical bars mark the run boundaries, the rows the kernel keeps.

This is *contraction along composition*: recognise that several computations are the same, perform it once, and repeat on what remains. Its two ingredients are a notion of "these move together", here adjacency with an equal byte, and a rule for composing what moves together. The question for further invention is whether a different notion exists: rows that move together up to a consistent relabelling, a repeated stretch of the *text* rather than of the rows, a recurrence among the induction's own steps, or, in the programme's broadest phrasing, transformations that are secretly the same computation expressed in different coordinates.

### Closure

Any contraction must survive its own success: after two computations are composed, the result must still be cheap to use, or the work has merely been deferred. That property is *closure*. Two negative results bound the search, a floor on the present notion of moving together and a proof that one natural way of composing the retained rows loses its compactness, but both rule out only squeezing harder on the same notion, not a different representation ([their scope](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#4-contraction-enrichment-and-closure>)). That is why the search looks outside the existing family.

### Enrichment: making existing work yield more

The second principle came from automatic differentiation. A program that computes a function can be run over a richer algebra, the *dual numbers*, in which every value carries its derivative alongside it, and multiplication follows the product rule:

\\\[ (a, a')\\,(b, b') = (ab,\\ a'b + ab'). \\\]

The ordinary computation proceeds unchanged, and the extra information comes out with it at small cost. In an earlier project of the maintainer's, exact gradients obtained this way made the optimiser of a library that flattens photographed pages about eleven times faster on single images ([page-dewarp](<https://github.com/lmmx/page-dewarp>)), not by doing the same work faster but by making a better algorithm possible. *Enrichment along composition* asks the same of the decoder: what extra quantity could ride along an existing step at little cost, and would it make a later stage cheap or unnecessary? A sharper form of the question is which later operation is necessary only because an earlier stage failed to keep, or cannot cheaply use, information it already met.

There are four natural sites for a rider: the Huffman decode, the move-to-front stage, the induction and construction. The test is strict: a rider must be cheaply acquired and must not simply be the answer in disguise, as each row's position in the text would be. Every candidate is then judged by its real machine cost, in four currencies that keep a saving in one from being paid for in another ([the companion, §5](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#5-the-four-currencies-of-computation>)).

### Four candidates, and what they answer

The measurements point to four concrete algorithms, each named by its mechanism. None has been built or measured; each comes with a forecast and with the smallest measurement that would reject it, and [the companion](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/index.md>) develops them in turn.

| Proposed change | What it could achieve | The finding it answers |
| --- | --- | --- |
| [**An induction schedule** with fewer steps](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#6-alternative-induction-schedules>) | fewer construction steps in the induction | construction's room below the knee |
| [**A direct fold** from move-to-front into construction](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#7-deforestation-across-stages>) | no intermediate array of capped runs | construction's room, and the move-to-front stage's beside it |
| [**Forward wiring with lanes**](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#8-forward-wiring-and-multi-lane-emission>) | simpler wiring, several independent walks in emission | construction's and emission's room together |
| [**A pooled scan**](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-from-bottlenecks-to-better-algorithms/#9-dissolving-the-serial-scanner>) | no serial scanning station | the scanner's room above the knee, and its pairing with construction |

The first three reduce the workers' demand and so lower the knee; the pooled scan removes the serial limit above it. At eight workers neither kind alone is likely to capture much, for the reason that supplying the scanner or construction alone saved little; the largest prize is a worker-side reduction combined with the pooled scan, the complementary pair of §5 made into designs.

## 8\. From characterisation to invention

The experiment set out to say where another mathematical invention would pay, and it has. Below the knee, construction and emission hold most of the opportunity, with the move-to-front stages a real third. Above it, the scanner does, and at eight workers the scanner and construction form a complementary pair whose joint value is many times what either shows alone. The last stages hold little. The decoder's true floor is still unknown, because the experiment's emission oracle left the word expansion in place.

Measured on 26 November, that floor is now known ([§6](<#6-what-the-experiment-could-not-remove>)). The pipeline alone takes 5.2–5.7 ms across one to ten workers. Once every other part is supplied, what binds at eight workers is the serial scan alone: 179.1 ms of 194.6 ms, or 92% of the wall ([the true-floor post](<https://lmmx.github.io/mibz2/pr-preview/pr-1031/blog/2026-11-26-true-floor-and-pooled-scan/#2-what-remains-when-one-stage-at-a-time-is-real>)).

Each measurement that remains has a single decision to make, and then it stops ([the plan](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/plan.md>)).

| Question | Why its answer changes a decision |
| --- | --- |
| What remains once the oracles' copying and the word expansion are removed? | whether the pipeline's own overhead deserves work, and what the decoder's floor really is |
| How much of the induction sub-phase scales with the number of steps? | whether a better induction schedule can pass its gate |
| Does a proposed design win across the whole path it touches? | whether to develop that implementation further |

The first is a zero-copy floor run, the second a census of the induction's steps, the third each design's own rejection test; the designs themselves can begin now.

What the experiment makes possible is a different kind of search. Before it, a mathematical idea was judged by whether it made one part of the decoder faster, and an idea that did so without moving the whole decode looked like a failure. Now each candidate can be aimed at a measured opportunity, priced against the part it changes, and paired with whatever companion change lets that saving reach the output. The matrix was the means. The subject is how a scientific understanding of a computational system can direct the search for cheaper algorithms, and what that search should look for next: not only faster stages, but representations in which the stages' work is shared, avoided, or never needed at all.

---

**Sources:**

- The motivation: [the programme's statement](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/essence.md>), [the programme](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/programme.md>), [the levers](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md>); Raj Jain, [*Computer Systems Performance Analysis*](<https://www.cs.wustl.edu/~jain/iucee/index.html>).
- The experiment: [the parts](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md>), [the design](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md>), [the registration](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md>), [the results](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md>) and [the report](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt>); journal §89.
- Construction's share: [stage 1's report](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>).
- The emission accounting: [the design analysis](<https://github.com/lmmx/mibz2/blob/7fab624eb204c591b0a72c925cf90bbad401396a/research/neon-movement/IDEAL-PARTS-2.md>); journal §90.
- The independent review of the experiment, [recorded in full](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md>), and its review of an earlier version of this post, [recorded in full](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt-on-the-post.md>); journal §91.
- [The plan](<https://github.com/lmmx/mibz2/blob/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/plan.md>), and [the floor run](<https://github.com/lmmx/mibz2/pull/917>).
