[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-650/blog/index.md>)

# Open Returns: picking the decoder block by block

Open rebuilds each block from a few retained rows and the words their returns spell, and the decoder chooses it wherever an inexpensive run count predicts a gain.

19 November 2026 · 6 min read · [Research result](<#3-what-it-bought>)

mibz2 0.5.7 ships a second way to undo the Burrows–Wheeler transform, Open, and a rule for when to use it. On the blocks the rule admits, Open keeps only some of the block's rows and decodes along their *returns*: where the LF walk next lands among the kept rows, and the bytes it spells on the way. The rule itself is two integer comparisons on a count the decoder already has.

**The takeaways:**

- **On a ten-core i9, 0.5.7 is 7–11% faster than 0.5.6 from 8 to 20 threads on Wikidata, with about 23% less peak memory.** Below 8 threads nothing changes: Open runs only on the parallel workers' route.
- **The rule is a prediction, not a guarantee.** It picks Open when a block's run records make up between 1/64 and 15% of its rows. Inside that band Open measured faster; outside it, up to 5× slower.
- **The band's edges were not tuned for Open.** They are LazyCells' existing admission lines, reused, so the release adds no new constant.
- **One machine, one workload.** The figures are from one i9-10900K and the Wikidata sample, and the first registered test of Open failed on its guard rule before a second, prospective one passed.

[Cutting the loop open](<https://lmmx.github.io/mibz2/pr-preview/pr-650/blog/2026-11-18-cutting-the-loop-open/index.md>) introduced the open-return idea. This post is about what shipped.

## 1\. What Open does

A BWT block's last column falls into runs of one repeated byte. Open sorts them by length:

- **thin runs,** of at most 16 rows, keep every row;
- **thick runs** lose their interiors. Open removes them with an auxiliary interval exchange, and if the block's primary row sits inside one, that row is kept on its own, so the walk has an exact starting point.

For each kept row, Open builds its *first return*: where the LF walk next lands among the kept rows, and the word of bytes it spells on the way there. Output then follows these returns, appending whole words, instead of visiting every row one at a time.

Illustrative: a block of six runs. The 80- and 40-row runs are thick, so their rows are not kept; the others are thin, so all their rows are. Each kept row records its first return and the bytes on the way, drawn as arcs. The 16-row limit is Open's own internal parameter; it is separate from the gate in section 2, which decides whether Open runs at all.

The saving is in what the decoder has to hold and visit. Long runs are cheap to describe by their returns, so a block with many long runs keeps few rows and spells long words.

## 2\. The gate: two comparisons on a count already in hand

Open has fixed costs: it builds its partition of the block, its return map and its words before writing a byte. Where a block has few long runs to eliminate, those costs don't pay. So 0.5.7 decides per block, from the block's length *n* and its number of run records *R*:

Open when 64·R ≥ n  and 20·R \< 3·n,  that is,  1/64 ≤ R/n \< 0.15.

*R* counts the encoded run records, which the decoder already holds. It is not quite the number of maximal runs, *r*: a record covers at most 127 rows, so a 500-row run takes four records. The test needs no extra pass over the block.

The gate, on the record density R/n. Blocks below 1/64 or at 15% and above keep LazyCells, the kernel 0.5.6 shipped; blocks between take Open. The two dots are worked examples: 50,000 records in a 900,000-row block is 5.6%, inside; 300,000 records is 33%, outside.

Why this band? It is where LazyCells already admits blocks, and a block-by-block census found it is also where Open wins:

| corpus | blocks | where they fall | Open ÷ LazyCells, time |
| --- | ---: | --- | ---: |
| Wikidata | 228 | inside | **0.972** |
| Wikidata | 155 | above 15% (small blocks, 2% of the rows) | 1.005 |
| text | 48 | above 15% | 5.11 |
| pattern | 88 | below 1/64 | 1.55 |
| one byte repeated | 2 | below 1/64 | 1.87 |

The 228 admitted Wikidata blocks hold 205.2 million of the sample's 209.7 million rows. Every block decoded identically both ways. The census ran on one thread in a cloud container; the i9 figures come from separate runs ([the census](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-19-open-admission.md>)).

The density is a cheap proxy, not a cost model. Two blocks with the same *R/n* can have very different run lengths and returns, and Open could win on one and lose on the other. The band is a conservative routing rule backed by measurement, and the fallback keeps the established kernel wherever Open hasn't shown it helps.

## 3\. What it bought

The release's acceptance run on the i9-10900K compared 0.5.7's kernel with 0.5.6 directly. On four copies of the Wikidata sample it took 0.954, 0.934 and 0.940 of 0.5.6's wall time at 8, 10 and 20 threads. Its peak memory was 0.76–0.78 of 0.5.6's, and held-out Wikidata it had not been tuned on behaved the same. The release-on-release run that every release gets agrees:

Measured on the i9-10900K, 10 paired rounds each (release-scaling/run-20261119T203443Z): 0.5.6's wall time over 0.5.7's on the Wikidata sample, with 95% intervals. At 1–4 threads the intervals span 1, as they should: Open is not on that route. From 8 threads up, 0.5.7 is 7–11% faster. At 8 threads its peak memory is 96 MB against 125 MB.

**The caveats, plainly:**

- **The first registered test failed.** Its guard rule compared single point estimates with a 2% bound across 39 noisy cells, a rule a change with no effect would fail about 97% of the time. Its four failures were all on corpora where no block reaches Open. That verdict stands. A second, separately registered test, with the guard rule fixed in advance, passed.
- **The gate is empirical.** It says where Open measured faster, not that it is faster for every block inside the band.
- **The 6-thread setting also changed.** A recalibration for the new kernel moved 6 threads on the i9 to the same lazy route.

Open is on crates.io and PyPI as mibz2 0.5.7; the [CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md>) has the full figures.
