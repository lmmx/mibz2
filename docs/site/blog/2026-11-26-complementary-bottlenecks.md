---
title: "Complementary bottlenecks: reading the ideal-parts matrix"
image: assets/social/blog-2026-11-26-complementary-bottlenecks.png
description: "At eight workers the scanner and construction must be relieved together, and the matrix is enough to start designing for that."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Complementary bottlenecks: reading the ideal-parts matrix

At eight workers the scanner and construction must be relieved together, and the matrix is enough to start designing for that.
{: .post-dek }

26 November 2026 · 14 min read · [Research synthesis](#7-whats-left-to-measure-and-why-it-closes)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

The ideal-parts matrix made each part of mibz2's decoder free in turn, and in combination, to find
which computations would have to become cheaper for a mathematical improvement to make decompression
faster. Read correctly, its strongest result is a pair: at eight workers, supplying the serial
scanner's output or construction's structures alone saves 3–6% of the wall, while supplying both saves
36%, so the two have to be relieved together and the saving cannot be split between them. Below the
knee construction is the large opportunity, but it is three sub-phases, and an idea that changes one of
them can only spend that one's budget. And the floor the matrix reported is not yet the decoder's,
because the oracle for emission left most of emission in place. This post reads the matrix with the
help of ChatGPT's review of it ([the review, verbatim](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md)), which corrected several of our own first
readings; it follows [the queueing-network post](2026-11-25-the-decoder-as-a-queueing-network.md) and
[its companion](2026-11-25-the-decoder-as-a-queueing-network-notes.md), and does not repeat them.

**Takeaways**

- At 8 workers, making the scanner or construction free alone saves 3–6%; both together save 36%,
  from 195 to 125 ms, 1.56 times the throughput. They are a complementary pair, and the saving cannot
  be split between them.
- Below [the knee](2026-11-25-the-decoder-as-a-queueing-network.md#5-the-bottleneck-law), construction is the big room (−42% at one worker), but it is three sub-phases, and
  any one idea has only its own sub-phase's budget.
- Emission's oracle left most of emission in place, so the decoder's floor is not yet measured; a
  follow-up run ([#917](https://github.com/lmmx/mibz2/pull/917)) measures it.
- The matrix is enough to start designing. The measurements that remain each decide one thing and then
  close.

## 1. Three questions, and the one the matrix answers

Three questions about a decoder's performance are easy to run together.

| Question | What answers it | What it tells you |
|---|---|---|
| Where does the time go? | profiling, activity timings | how much work there is, and where |
| What controls the wall now? | scaling runs, interventions that slow an activity | which constraints bind in this configuration |
| What would be worth making cheaper? | making parts ideal, alone and together | the room an improvement could have, including room hidden behind another bottleneck |

They differ because the decoder overlaps its work. If the scanner and the workers could each get
through the file in about 185 ms, making either much faster would leave the other still taking 185 ms,
while slowing either would slow the whole. So sensitivity to a slowdown does not measure the gain from a
speedup. The earlier experiments answered the second question, which stage controls the wall at which
worker count ([§11 of the queueing post](2026-11-25-the-decoder-as-a-queueing-network.md#11-testing-the-explanation-by-intervention));
the matrix was built to answer the third, because that is the one that chooses targets for mathematics
([why the matrix](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/README.md)).

The decoder it studies has six parts, which the matrix names by letter.

<figure class="bf-fig" id="pipeline">
<p class="bf-title">The decoder's six parts</p>
<svg viewBox="0 0 720 220" role="img" aria-label="The decoder's pipeline. One scanner thread decodes each block's Huffman symbols into a queue of scanned blocks; each worker takes one block through MTF and RLE2, construction, emission, and RLE1 and CRC; the reader delivers blocks in order, and taking the next block releases an admission slot back to the scanner. Every stage shares the caches and memory."><text class="bf-t-strong" x="16.0" y="20.0" text-anchor="start" style="font-size:13px">One block's path through the decoder, and the shared memory every stage leans on</text><rect x="20" y="60" width="120" height="58" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="80.0" y="84.0" text-anchor="middle" style="font-size:13px">S  scan</text><text class="bf-tm" x="80.0" y="102.0" text-anchor="middle" style="font-size:11.5px">one thread</text><line x1="142" y1="89" x2="168" y2="89" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="168.0,89.0 159.0,84.0 159.0,94.0" style="fill:var(--bf-frame)"/><rect x="170" y="68" width="80" height="42" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-tm" x="210.0" y="87.0" text-anchor="middle" style="font-size:11.5px">scanned</text><text class="bf-tm" x="210.0" y="101.0" text-anchor="middle" style="font-size:11.5px">blocks</text><line x1="252" y1="89" x2="276" y2="89" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="276.0,89.0 267.0,84.0 267.0,94.0" style="fill:var(--bf-frame)"/><rect x="278" y="54" width="300" height="70" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-tm" x="428.0" y="72.0" text-anchor="middle" style="font-size:11.5px">each worker, one block at a time</text><rect x="290" y="80" width="62" height="38" rx="4" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="321.0" y="96.0" text-anchor="middle" style="font-size:13px">M</text><text class="bf-tm" x="321.0" y="111.0" text-anchor="middle" style="font-size:9.5px">MTF, RLE2</text><rect x="362" y="80" width="62" height="38" rx="4" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="393.0" y="96.0" text-anchor="middle" style="font-size:13px">C</text><text class="bf-tm" x="393.0" y="111.0" text-anchor="middle" style="font-size:9.5px">construction</text><line x1="353" y1="99" x2="361" y2="99" style="stroke:var(--bf-blue);stroke-width:1.8"/><polygon points="361.0,99.0 352.0,94.0 352.0,104.0" style="fill:var(--bf-blue)"/><rect x="434" y="80" width="62" height="38" rx="4" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="465.0" y="96.0" text-anchor="middle" style="font-size:13px">E</text><text class="bf-tm" x="465.0" y="111.0" text-anchor="middle" style="font-size:9.5px">emission</text><line x1="425" y1="99" x2="433" y2="99" style="stroke:var(--bf-blue);stroke-width:1.8"/><polygon points="433.0,99.0 424.0,94.0 424.0,104.0" style="fill:var(--bf-blue)"/><rect x="506" y="80" width="62" height="38" rx="4" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.5"/><text class="bf-t-strong" x="537.0" y="96.0" text-anchor="middle" style="font-size:13px">R</text><text class="bf-tm" x="537.0" y="111.0" text-anchor="middle" style="font-size:9.5px">RLE1, CRC</text><line x1="497" y1="99" x2="505" y2="99" style="stroke:var(--bf-blue);stroke-width:1.8"/><polygon points="505.0,99.0 496.0,94.0 496.0,104.0" style="fill:var(--bf-blue)"/><line x1="580" y1="89" x2="604" y2="89" style="stroke:var(--bf-frame);stroke-width:1.8"/><polygon points="604.0,89.0 595.0,84.0 595.0,94.0" style="fill:var(--bf-frame)"/><rect x="606" y="60" width="96" height="58" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text class="bf-t-strong" x="654.0" y="84.0" text-anchor="middle" style="font-size:13px">D  delivery</text><text class="bf-tm" x="654.0" y="102.0" text-anchor="middle" style="font-size:11.5px">in order</text><path d="M654,120 L654,152 L80,152 L80,122" style="fill:none;stroke:var(--bf-green);stroke-width:1.6;stroke-dasharray:5 4"/><polygon points="80,120 75,129 85,129" style="fill:var(--bf-green)"/><text class="bf-tm" x="367.0" y="146.0" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-green)">taking the next block in order releases an admission slot</text><rect x="20" y="176" width="682" height="34" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-tm" x="361.0" y="198.0" text-anchor="middle" style="font-size:12px">shared caches and memory: every stage competes for them</text><line x1="80" x2="80" y1="124" y2="176" style="stroke:var(--bf-frame);stroke-width:1.2;stroke-dasharray:2 3"/><line x1="428" x2="428" y1="126" y2="176" style="stroke:var(--bf-frame);stroke-width:1.2;stroke-dasharray:2 3"/><line x1="654" x2="654" y1="124" y2="176" style="stroke:var(--bf-frame);stroke-width:1.2;stroke-dasharray:2 3"/></svg>
<figcaption>Schematic, after the review's diagram. Each letter is one part of the ideal-parts matrix; the dashed line is the admission credit the in-order reader returns.</figcaption>
</figure>

The scanner (S) parses each block's coding tables and decodes its [Huffman symbols](2026-11-05-huffman-bit-buffer.md), on one thread. Each
worker then takes one block through four stages: move-to-front and the second run-length stage (M),
which turn the symbols into the block's last column; construction (C), which builds the structures for
undoing the block sort, its retained rows, [interval-exchange](2026-11-10-rauzy-veech-reconstruction.md) reductions, word descriptions and graph
links; emission (E), which follows those structures and expands their word codes into bytes; and the
final run-length expansion and checksum (R). The reader (D) delivers blocks to the caller in order
([the parts](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md)). This is the finite pipeline with feedback-controlled admission described
in the queueing post, and the hybrid-systems description fits it: between events, threads make
progress at rates set by contention; at events, blocks are admitted, dispatched, finished and taken
([the formulation](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md)).
That description is a modelling proposal, not a validated model, and it need not become a prerequisite
for the work the matrix points to.

## 2. A $2^p$ design over oracles

Each of the six parts has two settings. Real runs the ordinary computation; ideal supplies a correct
intermediate result recorded beforehand, prepared outside the timed decode, which the downstream stages
consume. Writing $p$ for the number of factors (the post keeps $k$ for workers), six parts give
$2^6 = 64$ combinations. Upstream stages still run when a downstream one is ideal; otherwise supplying
the final output would remove almost the whole decoder, and no change could be attributed to one stage.

Testing each part alone would have missed the central result. The combinations ask whether
construction becomes more worth improving once the scanner is improved, which is an interaction, and an
interaction is what a factorial design exists to reveal. The run used 44 combinations at each worker
count: 32 forming a half fraction of resolution VI, which estimates main effects and two-part
interactions without confusing them with each other, and the 12 with a single part ideal or a single
part real, which the half fraction lacks and which were added before any data so that the headline
room could be read from measured arms rather than a fitted model ([the design](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md);
[clarification 8](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md); [the companion's account](2026-11-25-the-decoder-as-a-queueing-network-notes.md#10-experimental-design-and-the-ideal-parts-matrix)).

The worker counts were 1, 6, 8 and 10: one worker exposes most of the workers' computation directly in
the wall (the scanner and reader are still separate threads), six lies below the crossover, eight at the
balance point and ten above it. The same binary ran every arm, with twelve randomised blocks of rounds,
the scanner pinned, and a 100 ms spin before each decode to keep the clock high; every arm held the same
oracle stores, the all-real baseline included. The run took 2,492 decodes over 4 h 15 min, every output
matched, the pin held in all 352 traces, and the removed activities' remaining busy time was 0.00–0.19%
of their real time ([the results](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md)). Its scope is the i9-10900K, the repeated Wikidata workload
and this arrangement: the stores took 495 MiB, so even the all-real arm ran in a different memory
environment from an ordinary decode ([the report](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt)).

## 3. The scanner and construction, in milliseconds

The four arms that matter most, as times rather than ratios:

| Workers | All real | Construction supplied | Scanner supplied | Both supplied |
|---:|---:|---:|---:|---:|
| 1 | 1,306 | 756 | 1,319 | 751 |
| 6 | 243 | 183 | 237 | 143 |
| 8 | 195 | 184 | 189 | 125 |
| 10 | 196 | 186 | 180 | 149 |

<figure class="bf-fig" id="interaction">
<p class="bf-title">The scanner and construction: a complementary pair</p>
<svg viewBox="0 0 520 450" role="img" aria-label="Interaction plots of wall time at 1, 6, 8 and 10 workers. At 8 workers: everything real 195 ms, the scanner supplied 189, construction supplied 184, both supplied 125. At 1 worker: 1,306, 1,319, 756 and 751. At 6: 243, 237, 183 and 143. At 10: 196, 180, 186 and 149."><text class="bf-t-strong" x="16.0" y="20.0" text-anchor="start" style="font-size:12.5px">Wall time in ms; the scanner real or supplied, construction real or supplied</text><text class="bf-t-strong" x="134.0" y="50.0" text-anchor="middle" style="font-size:13.5px">1 worker</text><line class="bf-rule" x1="28" x2="240" y1="202" y2="202"/><text class="bf-tm" x="72.0" y="218.0" text-anchor="middle" style="font-size:12px">S real</text><text class="bf-tm" x="196.0" y="218.0" text-anchor="middle" style="font-size:12px">S supplied</text><line x1="72" x2="196" y1="83.7" y2="81.5" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="72" cy="83.7" r="4.5" style="fill:var(--bf-amber)"/><circle cx="196" cy="81.5" r="4.5" style="fill:var(--bf-amber)"/><text class="bf-tm" x="64.0" y="87.7" text-anchor="end" style="font-size:12.5px">1,306</text><text class="bf-tm" x="204.0" y="85.5" text-anchor="start" style="font-size:12.5px">1,319</text><line x1="72" x2="196" y1="177.5" y2="178.5" style="stroke:var(--bf-blue);stroke-width:2.5"/><circle cx="72" cy="177.5" r="4.5" style="fill:var(--bf-blue)"/><circle cx="196" cy="178.5" r="4.5" style="fill:var(--bf-blue)"/><text class="bf-tm" x="64.0" y="181.5" text-anchor="end" style="font-size:12.5px">756</text><text class="bf-t-strong" x="204.0" y="182.5" text-anchor="start" style="font-size:12.5px">751</text><text class="bf-t-strong" x="394.0" y="50.0" text-anchor="middle" style="font-size:13.5px">6 workers</text><line class="bf-rule" x1="288" x2="500" y1="202" y2="202"/><text class="bf-tm" x="332.0" y="218.0" text-anchor="middle" style="font-size:12px">S real</text><text class="bf-tm" x="456.0" y="218.0" text-anchor="middle" style="font-size:12px">S supplied</text><line x1="332" x2="456" y1="81.5" y2="87.6" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="332" cy="81.5" r="4.5" style="fill:var(--bf-amber)"/><circle cx="456" cy="87.6" r="4.5" style="fill:var(--bf-amber)"/><text class="bf-tm" x="324.0" y="85.5" text-anchor="end" style="font-size:12.5px">243</text><text class="bf-tm" x="464.0" y="91.6" text-anchor="start" style="font-size:12.5px">237</text><line x1="332" x2="456" y1="139.6" y2="178.5" style="stroke:var(--bf-blue);stroke-width:2.5"/><circle cx="332" cy="139.6" r="4.5" style="fill:var(--bf-blue)"/><circle cx="456" cy="178.5" r="4.5" style="fill:var(--bf-blue)"/><text class="bf-tm" x="324.0" y="143.6" text-anchor="end" style="font-size:12.5px">183</text><text class="bf-t-strong" x="464.0" y="182.5" text-anchor="start" style="font-size:12.5px">143</text><text class="bf-t-strong" x="134.0" y="244.0" text-anchor="middle" style="font-size:13.5px">8 workers</text><line class="bf-rule" x1="28" x2="240" y1="396" y2="396"/><text class="bf-tm" x="72.0" y="412.0" text-anchor="middle" style="font-size:12px">S real</text><text class="bf-tm" x="196.0" y="412.0" text-anchor="middle" style="font-size:12px">S supplied</text><line x1="72" x2="196" y1="275.5" y2="284.0" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="72" cy="275.5" r="4.5" style="fill:var(--bf-amber)"/><circle cx="196" cy="284.0" r="4.5" style="fill:var(--bf-amber)"/><text class="bf-tm" x="64.0" y="279.5" text-anchor="end" style="font-size:12.5px">195</text><text class="bf-tm" x="204.0" y="288.0" text-anchor="start" style="font-size:12.5px">189</text><line x1="72" x2="196" y1="290.7" y2="372.5" style="stroke:var(--bf-blue);stroke-width:2.5"/><circle cx="72" cy="290.7" r="4.5" style="fill:var(--bf-blue)"/><circle cx="196" cy="372.5" r="4.5" style="fill:var(--bf-blue)"/><text class="bf-tm" x="64.0" y="294.7" text-anchor="end" style="font-size:12.5px">184</text><text class="bf-t-strong" x="204.0" y="376.5" text-anchor="start" style="font-size:12.5px">125</text><text class="bf-t-strong" x="394.0" y="244.0" text-anchor="middle" style="font-size:13.5px">10 workers</text><line class="bf-rule" x1="288" x2="500" y1="396" y2="396"/><text class="bf-tm" x="332.0" y="412.0" text-anchor="middle" style="font-size:12px">S real</text><text class="bf-tm" x="456.0" y="412.0" text-anchor="middle" style="font-size:12px">S supplied</text><line x1="332" x2="456" y1="275.5" y2="307.9" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="332" cy="275.5" r="4.5" style="fill:var(--bf-amber)"/><circle cx="456" cy="307.9" r="4.5" style="fill:var(--bf-amber)"/><text class="bf-tm" x="324.0" y="279.5" text-anchor="end" style="font-size:12.5px">196</text><text class="bf-tm" x="464.0" y="311.9" text-anchor="start" style="font-size:12.5px">180</text><line x1="332" x2="456" y1="296.9" y2="372.5" style="stroke:var(--bf-blue);stroke-width:2.5"/><circle cx="332" cy="296.9" r="4.5" style="fill:var(--bf-blue)"/><circle cx="456" cy="372.5" r="4.5" style="fill:var(--bf-blue)"/><text class="bf-tm" x="324.0" y="300.9" text-anchor="end" style="font-size:12.5px">186</text><text class="bf-t-strong" x="464.0" y="376.5" text-anchor="start" style="font-size:12.5px">149</text><line x1="70" x2="94" y1="434" y2="434" style="stroke:var(--bf-amber);stroke-width:2.5"/><text class="bf-tm" x="100.0" y="438.0" text-anchor="start" style="font-size:12.5px">construction real</text><line x1="270" x2="294" y1="434" y2="434" style="stroke:var(--bf-blue);stroke-width:2.5"/><text class="bf-tm" x="300.0" y="438.0" text-anchor="start" style="font-size:12.5px">construction supplied</text></svg>
<figcaption>Measured: the full ideal-parts matrix, scanner pinned; each point is a geometric-mean wall over blocks, the ratios in the report times the all-real walls. Parallel lines would mean the two parts act independently; at 6 and 8 workers the lines diverge, so each part matters far more once the other is supplied.</figcaption>
</figure>

Below the crossover construction is the large opportunity: supplying it cuts the wall by 42% at one
worker and 25% at six, with the scanner's supply doing nothing at one worker. That is a strong reason
to study construction, though not a promise that any one improvement to it can capture the whole.

At eight workers the opportunity is conditional. Either part alone saves 6 or 11 ms; together they save
70 ms. Construction supplied by itself saves 5.6% of the ordinary wall; supplied once the scanner is
already supplied, it saves 34% of that smaller wall. The denominator changed because the question
did. Adding the separately measured savings would badly underestimate the pair, and there is no basis
for assigning half of the joint 36% to each: the result describes a complementary pair, and the
factorial interaction, which averages over the other parts' settings, is the formal statement of what
these four numbers show directly ([the report](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt)).

Scanner and construction is the strongest pair, not the only route. Supplying the scanner and the
tested emission work saves about 16% at eight workers with construction still real, and a change of
representation could touch several parts at once. The other single parts matter in their own way.
Emission's tested intervention saves 16.5% at one worker and 14.9% at six, with a correction that §4
explains. Move-to-front and the second run-length stage save 7.5% at one worker and 8.4% at six, smaller
than construction but real, and a natural partner for it. The final run-length work and delivery show
little room alone in this arrangement, which could change once upstream constraints are relieved.

And adding workers can hurt even with the obvious work removed: with the scanner and construction both
supplied, going from eight workers to ten slows the decode from 125 to 149 ms. Shared cache and memory
contention is the plausible explanation, but this run cannot separate the decoder's own contention from
the traffic of its replay stores, so a third bottleneck in memory bandwidth remains a hypothesis
([Antigravity's second pass, §6.1](https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md)).

<p class="il-rubric-head">Of two phrases that mislead</p>

"Construction explains 80%" means 80% of the variation in log wall across the factorial's arms at one
worker. It does not mean construction takes 80% of the runtime: the activity timings put construction at
578 ms of a 1,283 ms one-worker decode, about 45% ([stage 1's report](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt)). The same distinction
explains how the scanner can carry much of the variation at eight workers while relieving it alone
changes the baseline only a little: many arms have already made the workers' work cheap, which exposes
the scanner. And "UNRESOLVED" is a label, not a shrug. The registered ROOM label needed the whole
interval to show a saving of at least 5%; construction at eight workers saves 5.6%, with an interval of
about 4.4–6.9%, so it misses that threshold. No cell met the full MASKED rule either, which needs
essentially no effect alone; the scanner–construction interaction is clear all the same
([the rules](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md)).

## 4. What "all ideal" left behind

Emission does two kinds of work: it decides which word code comes next by following the reconstruction
graph, and it expands that code into bytes. Its oracle supplies the codes in output order, which removes
the graph walk but still expands the words. Both the delivery of codes and the expansion that remains
are charged to the ledger entry the report calls emission's handover, and earlier measurements split
it ([Antigravity's second pass, §3.1](https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md)):

| Inside emission's "handover", at one worker | Time |
|---|---:|
| delivering the supplied codes | 25.8 ms |
| expanding the codes into bytes | 259.4 ms |
| together | 285 ms |

About nine tenths of it is real decoder work, not copying; the results first called it copying, and the
review corrected that. Two things follow. Emission's 16.5% is not a ceiling on what improving emission
could save, and the configuration with every part ideal still did a large share of emission.

| Workers | Wall with every part nominally ideal |
|---:|---:|
| 1 | 391 ms |
| 6 | 94 ms |
| 8 | 112 ms |
| 10 | 142 ms |

These times mix the remaining word expansion, the replay copies, the oracles' locking, contention and
the pipeline's own overhead, so they are not an irreducible floor, and subtracting handover times from
them does not give one either, because work overlaps and different arms have different critical paths.
The follow-up separates two quantities. Residual A uses zero-copy stand-ins with the word expansion still
done; Residual B removes the expansion too. Antigravity forecasts Residual B at 14–19 ms, an argued
estimate the run will test, not a measurement, and not a real decompression time: a real decoder must
still acquire the answers ([the zero-copy build](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/antigravity/prompt-build-zero-copy.md)). Its
purpose is bounded: to stop experimental copying and retained emission work being mistaken for a limit of
the decoder. It does not touch the scanner–construction finding.

## 5. Where mathematics could help

Mathematical work could help at two scales. The first improves one component: an induction order with
fewer steps, cheaper construction of dictionary entries, a better way to split a dependency chain. Each
has a specific budget, the time of the component it changes.

The second changes the representation so that more work is shared or avoided, which is closer to the
programme's aim ([the aim](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/essence.md)). The current inversion kernel already exploits one
observation: stretches of the inverse transform move together, so it keeps some rows, summarises the
stretches between them as words, builds the resulting structure and follows it. A more powerful
representation could reduce how many objects construction creates, how many links it wires, how many
dependent lookups emission makes, how many words it expands and how much intermediate data is written
and read again, so one mathematical change could relieve construction and emission together. The two
directions the programme names are contraction, recognising that several operations are the same
computation and composing them once, and enrichment, making existing work yield information that makes
later work unnecessary. Both must keep closure: after composition the representation must stay compact
and cheap to use, or the saving has only moved. The cost model that counts addressed reads, dependent
depth, passes and stored state keeps a candidate from winning in one currency while paying in another
([the formulation](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/reports/2026-11-24-from-search-to-formulated-problem.md)). The matrix supplies
the practical test: does a proposed reduction touch a large demand, and what companion change lets it
reach the output?

## 6. What each concrete change would touch

Construction is three sub-phases, and stage 1's activity timings at one worker give each its time.

<figure class="bf-fig" id="budget">
<p class="bf-title">Construction's budget at one worker</p>
<svg viewBox="0 0 720 168" role="img" aria-label="A bar of one worker's busy time, 1,280 ms, at one worker: MTF and RLE2 132 ms; construction's three sub-phases, the first pass 200 ms, induction and words 216 ms and wiring 161 ms, together 578 ms; emission 490 ms; RLE1 and CRC 76 ms."><text class="bf-t-strong" x="16.0" y="20.0" text-anchor="start" style="font-size:13px">Where one worker's 1,280 ms goes: construction is three sub-phases</text><rect x="20.0" y="64" width="70.1" height="46" style="fill:var(--bf-steel);opacity:0.45;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="55.1" y="84.0" text-anchor="middle" style="font-size:12.5px">A5</text><text class="bf-tm" x="55.1" y="100.0" text-anchor="middle" style="font-size:11px">132 ms</text><text class="bf-tm" x="55.1" y="128.0" text-anchor="middle" style="font-size:11px">MTF, RLE2</text><rect x="90.1" y="64" width="106.2" height="46" style="fill:var(--bf-blue);opacity:0.95;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="143.2" y="84.0" text-anchor="middle" style="font-size:12.5px">A6a</text><text class="bf-tm" x="143.2" y="100.0" text-anchor="middle" style="font-size:11px">200 ms</text><text class="bf-tm" x="143.2" y="128.0" text-anchor="middle" style="font-size:11px">first pass</text><rect x="196.4" y="64" width="114.8" height="46" style="fill:var(--bf-blue);opacity:0.75;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="253.8" y="84.0" text-anchor="middle" style="font-size:12.5px">A6b</text><text class="bf-tm" x="253.8" y="100.0" text-anchor="middle" style="font-size:11px">216 ms</text><text class="bf-tm" x="253.8" y="128.0" text-anchor="middle" style="font-size:11px">induction, words</text><rect x="311.1" y="64" width="85.5" height="46" style="fill:var(--bf-blue);opacity:0.55;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="353.9" y="84.0" text-anchor="middle" style="font-size:12.5px">A6c</text><text class="bf-tm" x="353.9" y="100.0" text-anchor="middle" style="font-size:11px">161 ms</text><text class="bf-tm" x="353.9" y="128.0" text-anchor="middle" style="font-size:11px">wiring</text><rect x="396.7" y="64" width="260.3" height="46" style="fill:var(--bf-amber);opacity:0.6;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="526.8" y="84.0" text-anchor="middle" style="font-size:12.5px">A7</text><text class="bf-tm" x="526.8" y="100.0" text-anchor="middle" style="font-size:11px">490 ms</text><text class="bf-tm" x="526.8" y="128.0" text-anchor="middle" style="font-size:11px">emission</text><rect x="657.0" y="64" width="40.4" height="46" style="fill:var(--bf-steel);opacity:0.35;stroke:var(--bf-surface);stroke-width:1"/><text class="bf-t-strong" x="677.2" y="84.0" text-anchor="middle" style="font-size:12.5px">A9</text><text class="bf-tm" x="677.2" y="100.0" text-anchor="middle" style="font-size:11px">76 ms</text><text class="bf-tm" x="677.2" y="128.0" text-anchor="middle" style="font-size:11px">RLE1, CRC</text><rect x="697.3" y="64" width="2.7" height="46" style="fill:var(--bf-frame);opacity:0.3;stroke:var(--bf-surface);stroke-width:1"/><path d="M90.1,56 L90.1,48 L396.7,48 L396.7,56" style="fill:none;stroke:var(--bf-blue);stroke-width:1.6"/><text class="bf-t-strong" x="243.4" y="42.0" text-anchor="middle" style="font-size:12px;fill:var(--bf-blue)">construction, C: 578 ms, about 45% of the 1,283 ms decode</text><text class="bf-tm" x="361.0" y="156.0" text-anchor="middle" style="font-size:12.5px">An idea that changes one sub-phase can save at most that sub-phase's time.</text></svg>
<figcaption>Measured: stage 1, one worker, seconds a decode for the mean worker (A5 132, A6a 200, A6b 216, A6c 161, A7 490, A9 76 ms of 1,280 ms busy; the decode's wall is 1,283 ms). An indicative split: its baseline differs slightly from the matrix's.</figcaption>
</figure>

The first pass over runs (A6a, 200 ms) classifies them and fills the counts and queues; setup,
interval-exchange induction and the dictionary of words (A6b, 216 ms) follow; and wiring the
retained-row graph takes 161 ms (A6c). They come from stage 1, whose baseline differs slightly from the
matrix's, so they are an indicative split rather than an exact partition of its 1,306 ms. That alone
explains why "construction has 42% room" does not mean a better induction schedule could save 42%.

<p class="il-rubric-head">Of the induction schedule's budget</p>

The induction steps have a precise count. For every schedule of the cell steps studied,

$$
\text{steps} = r_2 - 1 + U,
$$

where $r_2$ counts the distinct digrams of run letters, so $r_2 - 1$ is the floor, and $U$ counts the
extra boundaries a schedule creates on the way ([the record, §5.6](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/monodromy/2026-11-10-rauzy-veech.md)). Today's order makes 1.812
steps per original run on Wikidata; the best recorded order, free steps first and otherwise the largest
cell, makes 1.279; the floor is 1.220.

<figure class="bf-fig" id="rauzy">
<p class="bf-title">The induction schedule's budget</p>
<svg viewBox="0 0 720 268" role="img" aria-label="Bars of induction steps per original run: today's right-end order 1.812, the best recorded order 1.279, the floor 1.220. Below, the budget: A6b's 216 ms at one worker times a 29.4% cut is at most 64 ms, 4.9% of the wall; the 3% gate is 39 ms, so about 62% of A6b would have to be step time."><text class="bf-t-strong" x="16.0" y="20.0" text-anchor="start" style="font-size:13px">Induction steps per original run, and what the best known order could save</text><text class="bf-tm" x="290.0" y="59.0" text-anchor="end" style="font-size:12px">right end (today's order)</text><rect x="300" y="44" width="308.0" height="20" rx="3" style="fill:var(--bf-amber);opacity:0.75"/><text class="bf-t-strong" x="616.0" y="59.0" text-anchor="start" style="font-size:12.5px">1.812</text><text class="bf-tm" x="290.0" y="89.0" text-anchor="end" style="font-size:12px">free steps first, else the largest cell</text><rect x="300" y="74" width="217.4" height="20" rx="3" style="fill:var(--bf-blue);opacity:0.75"/><text class="bf-t-strong" x="525.4" y="89.0" text-anchor="start" style="font-size:12.5px">1.279</text><text class="bf-tm" x="290.0" y="119.0" text-anchor="end" style="font-size:12px">floor for this class of steps, r<tspan baseline-shift="sub" font-size="9">2</tspan> − 1</text><rect x="300" y="104" width="207.4" height="20" rx="3" style="fill:var(--bf-frame);opacity:0.75"/><text class="bf-t-strong" x="515.4" y="119.0" text-anchor="start" style="font-size:12.5px">1.220</text><text class="bf-tm" x="290.0" y="158.0" text-anchor="end" style="font-size:12px">A6b at one worker</text><text class="bf-tm" x="306.0" y="158.0" text-anchor="start" style="font-size:12.5px">216 ms</text><text class="bf-tm" x="290.0" y="180.0" text-anchor="end" style="font-size:12px">× the step cut, 1 − 1.279/1.812</text><text class="bf-tm" x="306.0" y="180.0" text-anchor="start" style="font-size:12.5px">× 29.4%</text><text class="bf-t-strong" x="290.0" y="202.0" text-anchor="end" style="font-size:12px">= the most it could save, if all of A6b scaled with steps</text><text class="bf-t-strong" x="306.0" y="202.0" text-anchor="start" style="font-size:12.5px">≈ 64 ms, 4.9% of 1,306 ms</text><text class="bf-tm" x="290.0" y="224.0" text-anchor="end" style="font-size:12px">the 3% gate</text><text class="bf-tm" x="306.0" y="224.0" text-anchor="start" style="font-size:12.5px">≈ 39 ms</text><text class="bf-t-strong" x="290.0" y="246.0" text-anchor="end" style="font-size:12px">so the share of A6b that must be step time</text><text class="bf-t-strong" x="306.0" y="246.0" text-anchor="start" style="font-size:12.5px">≈ 62%, before the new order's own cost</text></svg>
<figcaption>Counts from the Rauzy–Veech record §5.6 on all 233 Wikidata blocks; the budget is the review's arithmetic against the matrix's 1,306 ms one-worker wall. The counts are for the earlier full run-letter exchange, not yet the current kernel's object.</figcaption>
</figure>

The known order cuts the step count by

$$
1 - \frac{1.279}{1.812} \approx 29.4\%.
$$

If every millisecond of A6b fell in proportion, that would save $216 \times 0.294 \approx 64$ ms, about
4.9% of the one-worker matrix wall. More realistically,

$$
\text{saving} \approx 216\ \text{ms} \times f_{\text{steps}} \times 29.4\% - \text{the new schedule's cost},
$$

with $f_{\text{steps}}$ the share of A6b spent on steps that a better order removes. The written gate is
3% of the wall, about 39 ms, so roughly 62% of A6b would have to be such step time before paying for the
schedule; the census measures that share and decides it ([the census](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md)).
Two transfer questions remain: the 1.812 and 1.279 counts are for the earlier full run-letter interval
exchange, while the current kernel eliminates thick strips and keeps thin rows, so the census must
recount on the actual object; and a lower-step order can lose the locality and cheap sweep order the
present one has. The 1.220 floor is the floor of this class of steps, not a bound on inverting the
transform. The surface-dynamics exploration behind it is an offline instrument: it may search expensive
alternatives, but what ships must be a cheap rule decided from information construction already has.

Two concrete designs from Antigravity address other parts of construction's budget, both proposals with
explicit kill tests and no measured gain yet ([its designs, §5](https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md)). One feeds construction directly
from the move-to-front stage: the current interface caps runs at 127 and construction then recombines
them into maximal runs, so a direct path would skip that materialisation (`src/runs.rs`, `src/open.rs`).
The other changes the graph wiring together with emission, replacing bidirectional links with forward
ones walked several at a time, which could simplify A6c's wiring and give emission more independent
memory accesses (`src/open/outq.rs`, the emitters in `src/open.rs`). The second crosses the stage
boundary, which is the matrix's lesson in small: a different consumer lets a cheaper structure be built.

The scanner's room is a matter of where work is placed. A pooled scan would find and check block
boundaries, hand workers compressed blocks, and let each worker scan and reconstruct its own, keeping
in-order delivery (`src/parallel.rs`, `src/huff.rs`). The scan would still be done, only spread across
cores, so the scanner oracle's deletion of the work does not predict its gain. Antigravity forecasts 130
to 145 ms for a pooled scan with construction improved, against the 195 ms baseline; the measured 125 ms
endpoint makes that a sensible scale to aim at, but does not validate the forecast, and a redesign of
that size moves the best worker count too.

## 7. What's left to measure, and why it closes

The broad priorities are settled enough to start designing, and each remaining measurement has one
decision to make ([the plan](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/plan.md)):

| Question | Why the answer changes a decision |
|---|---|
| What remains once replay copying and word expansion are taken out? | says whether the pipeline's overhead deserves work, and corrects the apparent floor |
| How much of A6b is induction steps rather than setup and dictionary work? | says whether a better induction order can pass its gate |
| Does a proposed representation or pipeline change win as a complete path? | says whether to develop that implementation |

The first is the zero-copy floor run, nine arms at 1, 6, 7, 8, 9 and 10 workers with every handover
zero-copy, held until the zero-copy oracles are built
([#917](https://github.com/lmmx/mibz2/pull/917)); it is a focused follow-up, not another 44-arm matrix.
The second is the induction census. The third is each design's own kill test, and the designs for the
pooled scan, the folded first pass and forward emission may start now, alongside the two closing
measurements. The offline tools, network calculus for bounds under stated assumptions, uiCA for local
instruction costs, and surface dynamics for structure, are instruments with bounded purposes, not
prerequisites. One distinction is worth keeping in the record: a multi-lane walk can shorten dependency
chains, and make the decoder much faster, without reducing the number of addressed reads, leaving the
research question about reads open; both outcomes should be written down as what they are.

The matrix supplied the reason to do the constructive work: choose a candidate that removes a measured
cost, account for what it costs to obtain its new representation, and test the whole path it touches.
The floor and sub-phase measurements will sharpen that choice, and then close.


---

**Sources:**

- The review: [ChatGPT's review of the ideal-parts work, verbatim](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md), and [the corrections it made](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/reviews/README.md); journal §91.
- The matrix: [the results](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md), [the report](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/report.txt), [the parts](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/parts.md), [the design](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/design.md) and [the registration](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/ideal-parts/registration.md); journal §89.
- Construction's sub-phases: [stage 1's report](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt).
- The induction counts: [the Rauzy–Veech record, §5.6](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/monodromy/2026-11-10-rauzy-veech.md); [the census](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/rauzy-census/README.md).
- The designs and forecasts: [Antigravity's second pass](https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md), at 7fab624e on `research/neon-movement`; journal §90.
- [The plan](https://github.com/lmmx/mibz2/blob/e9240bbacabff3047b8cbdd978df26077b65a403/docs/journal/2026-11-24-algorithm-level/plan.md), and the zero-copy floor run, [#917](https://github.com/lmmx/mibz2/pull/917).
