---
title: "Open Returns: picking the decoder block by block"
image: assets/social/blog-2026-11-19-open-returns.png
description: "Open rebuilds each block from a few retained rows and the words their returns spell, and the decoder chooses it wherever an inexpensive run count predicts a gain."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Open Returns: picking the decoder block by block

Open rebuilds each block from a few retained rows and the words their returns spell, and the decoder chooses it wherever an inexpensive run count predicts a gain.
{: .post-dek }

19 November 2026 · 6 min read · [Research result](#3-what-it-bought)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 0.5.7 ships a second way to undo the Burrows–Wheeler transform, Open, and a rule for when to use it. On the blocks the rule admits, Open keeps only some of the block's rows and decodes along their *returns*: where the LF walk next lands among the kept rows, and the bytes it spells on the way. The rule itself is two integer comparisons on a count the decoder already has.

**The takeaways:**

- **On a ten-core i9, 0.5.7 is 7–11% faster than 0.5.6 from 8 to 20 threads on Wikidata, with about 23% less peak memory.** Below 8 threads nothing changes: Open runs only on the parallel workers' route.
- **The rule is a prediction, not a guarantee.** It picks Open when a block's run records make up between 1/64 and 15% of its rows. Inside that band Open measured faster; outside it, up to 5× slower.
- **The band's edges were not tuned for Open.** They are LazyCells' existing admission lines, reused, so the release adds no new constant.
- **One machine, one workload.** The figures are from one i9-10900K and the Wikidata sample, and the first registered test of Open failed on its guard rule before a second, prospective one passed.

[Cutting the loop open](2026-11-18-cutting-the-loop-open.md) introduced the open-return idea. This post is about what shipped.

## 1. What Open does

A BWT block's last column falls into runs of one repeated byte. Open sorts them by length:

- **thin runs,** of at most 16 rows, keep every row;
- **thick runs** lose their interiors. Open removes them with an auxiliary interval exchange, and if the block's primary row sits inside one, that row is kept on its own, so the walk has an exact starting point.

For each kept row, Open builds its *first return*: where the LF walk next lands among the kept rows, and the word of bytes it spells on the way there. Output then follows these returns, appending whole words, instead of visiting every row one at a time.

<figure class="bf-fig" id="thin-thick">
<svg viewBox="0 0 720 300" role="img" aria-label="An illustrative block of six runs: A 4 rows, B 12, C 80, D 3, E 40, F 9. Runs of at most 16 rows are thin and every row is kept; the 80- and 40-row runs are thick and their interiors are eliminated. Each kept row stores its first return among the kept rows and the word spelled on the way, drawn as arcs."><text x="30.0" y="26.0" class="bf-tm" text-anchor="start" style="font-size:15px">the BWT's last column, one block, split into runs</text><rect x="30.0" y="40.0" width="17.8" height="46.0" rx="3" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1;opacity:1"/><text x="38.9" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:12px;font-weight:600">A×4</text><rect x="47.8" y="40.0" width="53.5" height="46.0" rx="3" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1;opacity:1"/><text x="74.6" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:15px;font-weight:600">B×12</text><rect x="101.4" y="40.0" width="356.8" height="46.0" rx="3" style="fill:var(--bf-steel-tint);stroke:var(--bf-frame);stroke-width:1;opacity:1"/><text x="279.7" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:15px;font-weight:600">C×80</text><rect x="458.1" y="40.0" width="13.4" height="46.0" rx="3" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1;opacity:1"/><text x="464.8" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:12px;font-weight:600">D×3</text><rect x="471.5" y="40.0" width="178.4" height="46.0" rx="3" style="fill:var(--bf-steel-tint);stroke:var(--bf-frame);stroke-width:1;opacity:1"/><text x="560.7" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:15px;font-weight:600">E×40</text><rect x="649.9" y="40.0" width="40.1" height="46.0" rx="3" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1;opacity:1"/><text x="669.9" y="68.0" class="bf-ts" text-anchor="middle" style="font-size:15px;font-weight:600">F×9</text><rect x="30.0" y="104.0" width="16.0" height="14.0" rx="2" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1;opacity:1"/><text x="52.0" y="116.0" class="bf-ts" text-anchor="start" style="font-size:15px">thin (at most 16 rows): every row kept</text><rect x="370.0" y="104.0" width="16.0" height="14.0" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-frame);stroke-width:1;opacity:1"/><text x="392.0" y="116.0" class="bf-ts" text-anchor="start" style="font-size:15px">thick: its interior eliminated</text><text x="30.0" y="148.0" class="bf-tm" text-anchor="start" style="font-size:15px">rows Open keeps, and the word each one returns with</text><circle cx="34.5" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="43.4" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="56.8" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="74.6" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="92.4" cy="174" r="5" style="fill:var(--bf-blue)"/><line x1="105.4" y1="174" x2="454.1" y2="174" style="stroke:var(--bf-frame);stroke-dasharray:4 4"/><circle cx="461.5" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="468.1" cy="174" r="5" style="fill:var(--bf-blue)"/><line x1="475.5" y1="174" x2="645.9" y2="174" style="stroke:var(--bf-frame);stroke-dasharray:4 4"/><circle cx="659.9" cy="174" r="5" style="fill:var(--bf-blue)"/><circle cx="680.0" cy="174" r="5" style="fill:var(--bf-blue)"/><path d="M 34.5 180 Q 248.0 255.2 461.5 180" style="fill:none;stroke:var(--bf-green);stroke-width:1.6"/><path d="M 74.6 180 Q 377.3 276.6 680.0 180" style="fill:none;stroke:var(--bf-green);stroke-width:1.6"/><path d="M 468.1 180 Q 262.4 253.4 56.8 180" style="fill:none;stroke:var(--bf-green);stroke-width:1.6"/><text x="30.0" y="272.0" class="bf-ts" text-anchor="start" style="font-size:15px">Each kept row stores where LF next lands among the kept rows, and the bytes it spells on the way.</text><text x="30.0" y="292.0" class="bf-ts" text-anchor="start" style="font-size:15px">Output follows these returns, not the eliminated rows.</text></svg>
<figcaption>Illustrative: a block of six runs. The 80- and 40-row runs are thick, so their rows are not kept; the others are thin, so all their rows are. Each kept row records its first return and the bytes on the way, drawn as arcs. The 16-row limit is Open's own internal parameter; it is separate from the gate in section 2, which decides whether Open runs at all.</figcaption>
</figure>

The saving is in what the decoder has to hold and visit. Long runs are cheap to describe by their returns, so a block with many long runs keeps few rows and spells long words.

## 2. The gate: two comparisons on a count already in hand

Open has fixed costs: it builds its partition of the block, its return map and its words before writing a byte. Where a block has few long runs to eliminate, those costs don't pay. So 0.5.7 decides per block, from the block's length *n* and its number of run records *R*:

<div class="bf-eq" markdown>
Open when&nbsp; 64·R ≥ n &nbsp;and&nbsp; 20·R &lt; 3·n, &nbsp;that is, &nbsp;1/64 ≤ R/n &lt; 0.15.
</div>

*R* counts the encoded run records, which the decoder already holds. It is not quite the number of maximal runs, *r*: a record covers at most 127 rows, so a 500-row run takes four records. The test needs no extra pass over the block.

<figure class="bf-fig" id="gate">
<svg viewBox="0 0 720 250" role="img" aria-label="The gate. On a log axis of record density R over n, blocks below 1/64 and at or above 15% keep LazyCells; blocks in between take Open. A block of 900,000 rows with 50,000 records sits at 5.6%, inside; one with 300,000 records sits at 33%, outside."><rect x="40.0" y="60.0" width="234.3" height="56.0" rx="4" style="fill:var(--bf-steel-tint);stroke:var(--bf-frame);stroke-width:1;opacity:1"/><rect x="274.3" y="60.0" width="257.7" height="56.0" rx="4" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1;opacity:1"/><rect x="532.0" y="60.0" width="158.0" height="56.0" rx="4" style="fill:var(--bf-steel-tint);stroke:var(--bf-frame);stroke-width:1;opacity:1"/><text x="157.1" y="85.0" class="bf-ts" text-anchor="middle" style="font-size:17px;font-weight:600">LazyCells</text><text x="157.1" y="106.0" class="bf-tm" text-anchor="middle" style="font-size:15px">very few records</text><text x="403.1" y="85.0" class="bf-t-green" text-anchor="middle" style="font-size:17px;font-weight:700">Open</text><text x="403.1" y="106.0" class="bf-tm" text-anchor="middle" style="font-size:15px">Wikidata lives here</text><text x="611.0" y="85.0" class="bf-ts" text-anchor="middle" style="font-size:17px;font-weight:600">LazyCells</text><text x="611.0" y="106.0" class="bf-tm" text-anchor="middle" style="font-size:15px">many records: text</text><line x1="274.3" y1="48" x2="274.3" y2="128" style="stroke:var(--bf-ink);stroke-width:2"/><text x="274.3" y="40.0" class="bf-ts" text-anchor="middle" style="font-size:16px;font-weight:600">1/64 ≈ 1.6%</text><line x1="532.0" y1="48" x2="532.0" y2="128" style="stroke:var(--bf-ink);stroke-width:2"/><text x="532.0" y="40.0" class="bf-ts" text-anchor="middle" style="font-size:16px;font-weight:600">15%</text><line x1="144.4" y1="128" x2="144.4" y2="134" style="stroke:var(--bf-frame)"/><text x="144.4" y="152.0" class="bf-tm" text-anchor="middle" style="font-size:15px">0.5%</text><line x1="223.4" y1="128" x2="223.4" y2="134" style="stroke:var(--bf-frame)"/><text x="223.4" y="152.0" class="bf-tm" text-anchor="middle" style="font-size:15px">1%</text><line x1="406.8" y1="128" x2="406.8" y2="134" style="stroke:var(--bf-frame)"/><text x="406.8" y="152.0" class="bf-tm" text-anchor="middle" style="font-size:15px">5%</text><line x1="485.8" y1="128" x2="485.8" y2="134" style="stroke:var(--bf-frame)"/><text x="485.8" y="152.0" class="bf-tm" text-anchor="middle" style="font-size:15px">10%</text><line x1="611.0" y1="128" x2="611.0" y2="134" style="stroke:var(--bf-frame)"/><text x="611.0" y="152.0" class="bf-tm" text-anchor="middle" style="font-size:15px">30%</text><text x="365.0" y="176.0" class="bf-tm" text-anchor="middle" style="font-size:15px">record density R/n (log scale)</text><circle cx="418.8" cy="208" r="6" style="fill:var(--bf-ink)"/><text x="418.8" y="236.0" class="bf-ts" text-anchor="middle" style="font-size:15px">50k records in 900k rows: 5.6%</text><circle cx="623.0" cy="208" r="6" style="fill:var(--bf-ink)"/><text x="623.0" y="236.0" class="bf-ts" text-anchor="middle" style="font-size:15px">300k: 33%</text></svg>
<figcaption>The gate, on the record density R/n. Blocks below 1/64 or at 15% and above keep LazyCells, the kernel 0.5.6 shipped; blocks between take Open. The two dots are worked examples: 50,000 records in a 900,000-row block is 5.6%, inside; 300,000 records is 33%, outside.</figcaption>
</figure>

Why this band? It is where LazyCells already admits blocks, and a block-by-block census found it is also where Open wins:

| corpus | blocks | where they fall | Open ÷ LazyCells, time |
|---|--:|---|--:|
| Wikidata | 228 | inside | **0.972** |
| Wikidata | 155 | above 15% (small blocks, 2% of the rows) | 1.005 |
| text | 48 | above 15% | 5.11 |
| pattern | 88 | below 1/64 | 1.55 |
| one byte repeated | 2 | below 1/64 | 1.87 |

The 228 admitted Wikidata blocks hold 205.2 million of the sample's 209.7 million rows. Every block decoded identically both ways. The census ran on one thread in a cloud container; the i9 figures come from separate runs ([the census](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-19-open-admission.md)).

The density is a cheap proxy, not a cost model. Two blocks with the same *R/n* can have very different run lengths and returns, and Open could win on one and lose on the other. The band is a conservative routing rule backed by measurement, and the fallback keeps the established kernel wherever Open hasn't shown it helps.

## 3. What it bought

The release's acceptance run on the i9-10900K compared 0.5.7's kernel with 0.5.6 directly. On four copies of the Wikidata sample it took 0.954, 0.934 and 0.940 of 0.5.6's wall time at 8, 10 and 20 threads. Its peak memory was 0.76–0.78 of 0.5.6's, and held-out Wikidata it had not been tuned on behaved the same. The release-on-release run that every release gets agrees:

<figure class="bf-fig" id="release-scaling">
<svg viewBox="0 0 720 330" role="img" aria-label="Release scaling on the i9-10900K: 0.5.6 wall over 0.5.7 wall with 95% intervals. 1 thread 0.988, 2 threads 0.991, 4 threads 1.011, all spanning 1; 8 threads 1.108, 10 threads 1.073, 12 threads 1.081, 16 threads 1.074, 20 threads 1.091, all above 1."><line x1="148.5" y1="20" x2="148.5" y2="276" style="stroke:var(--bf-frame);stroke-width:1"/><text x="148.5" y="296.0" class="bf-tm" text-anchor="middle" style="font-size:15px">0.95</text><line x1="291.0" y1="20" x2="291.0" y2="276" style="stroke:var(--bf-frame);stroke-width:2"/><text x="291.0" y="296.0" class="bf-tm" text-anchor="middle" style="font-size:15px">1.00</text><line x1="433.5" y1="20" x2="433.5" y2="276" style="stroke:var(--bf-frame);stroke-width:1"/><text x="433.5" y="296.0" class="bf-tm" text-anchor="middle" style="font-size:15px">1.05</text><line x1="576.0" y1="20" x2="576.0" y2="276" style="stroke:var(--bf-frame);stroke-width:1"/><text x="576.0" y="296.0" class="bf-tm" text-anchor="middle" style="font-size:15px">1.10</text><text x="405.0" y="320.0" class="bf-tm" text-anchor="middle" style="font-size:15px">0.5.6's wall time ÷ 0.5.7's, Wikidata (above 1: 0.5.7 is faster)</text><text x="106.0" y="41.0" class="bf-ts" text-anchor="end" style="font-size:15px">1 thread</text><line x1="157.1" y1="36" x2="362.2" y2="36" style="stroke:var(--bf-steel);stroke-width:2"/><circle cx="256.8" cy="36" r="6" style="fill:var(--bf-steel)"/><text x="372.2" y="41.0" class="bf-tm" text-anchor="start" style="font-size:14px">0.988</text><text x="106.0" y="71.0" class="bf-ts" text-anchor="end" style="font-size:15px">2 threads</text><line x1="177.0" y1="66" x2="353.7" y2="66" style="stroke:var(--bf-steel);stroke-width:2"/><circle cx="265.4" cy="66" r="6" style="fill:var(--bf-steel)"/><text x="363.7" y="71.0" class="bf-tm" text-anchor="start" style="font-size:14px">0.991</text><text x="106.0" y="101.0" class="bf-ts" text-anchor="end" style="font-size:15px">4 threads</text><line x1="234.0" y1="96" x2="413.6" y2="96" style="stroke:var(--bf-steel);stroke-width:2"/><circle cx="322.3" cy="96" r="6" style="fill:var(--bf-steel)"/><text x="423.6" y="101.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.011</text><text x="106.0" y="131.0" class="bf-ts" text-anchor="end" style="font-size:15px">8 threads</text><line x1="541.8" y1="126" x2="653.0" y2="126" style="stroke:var(--bf-green);stroke-width:2"/><circle cx="598.8" cy="126" r="6" style="fill:var(--bf-green)"/><text x="663.0" y="131.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.108</text><text x="106.0" y="161.0" class="bf-ts" text-anchor="end" style="font-size:15px">10 threads</text><line x1="464.9" y1="156" x2="533.3" y2="156" style="stroke:var(--bf-green);stroke-width:2"/><circle cx="499.1" cy="156" r="6" style="fill:var(--bf-green)"/><text x="543.3" y="161.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.073</text><text x="106.0" y="191.0" class="bf-ts" text-anchor="end" style="font-size:15px">12 threads</text><line x1="419.2" y1="186" x2="630.2" y2="186" style="stroke:var(--bf-green);stroke-width:2"/><circle cx="521.9" cy="186" r="6" style="fill:var(--bf-green)"/><text x="640.2" y="191.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.081</text><text x="106.0" y="221.0" class="bf-ts" text-anchor="end" style="font-size:15px">16 threads</text><line x1="402.1" y1="216" x2="604.5" y2="216" style="stroke:var(--bf-green);stroke-width:2"/><circle cx="501.9" cy="216" r="6" style="fill:var(--bf-green)"/><text x="614.5" y="221.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.074</text><text x="106.0" y="251.0" class="bf-ts" text-anchor="end" style="font-size:15px">20 threads</text><line x1="464.9" y1="246" x2="641.6" y2="246" style="stroke:var(--bf-green);stroke-width:2"/><circle cx="550.4" cy="246" r="6" style="fill:var(--bf-green)"/><text x="651.6" y="251.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.091</text></svg>
<figcaption>Measured on the i9-10900K, 10 paired rounds each (release-scaling/run-20261119T203443Z): 0.5.6's wall time over 0.5.7's on the Wikidata sample, with 95% intervals. At 1–4 threads the intervals span 1, as they should: Open is not on that route. From 8 threads up, 0.5.7 is 7–11% faster. At 8 threads its peak memory is 96 MB against 125 MB.</figcaption>
</figure>

**The caveats, plainly:**

- **The first registered test failed.** Its guard rule compared single point estimates with a 2% bound across 39 noisy cells, a rule a change with no effect would fail about 97% of the time. Its four failures were all on corpora where no block reaches Open. That verdict stands. A second, separately registered test, with the guard rule fixed in advance, passed.
- **The gate is empirical.** It says where Open measured faster, not that it is faster for every block inside the band.
- **The 6-thread setting also changed.** A recalibration for the new kernel moved 6 threads on the i9 to the same lazy route.

Open is on crates.io and PyPI as mibz2 0.5.7; the [CHANGELOG](https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md) has the full figures.
