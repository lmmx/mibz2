[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-798/blog/index.md>)

# Where the decoding work goes: random IDs in repetitive JSON

Tracing each costly row back to the text it spells shows that JSON keys cost almost nothing, while random statement IDs claim two thirds of the hardest rows.

20 November 2026 · 9 min read · [Open question](<#5-what-comes-next>)

**What we measured.** Undoing bzip2's Burrows–Wheeler transform (BWT) means rebuilding a block's text one byte at a time from a sorted table of its rows. Each byte of the text comes from exactly one row. Some rows are cheap: they belong to a large group of rows that the decoder moves together. Others are expensive: they stand alone, and the decoder has to visit each of them in turn, waiting on memory every time. Until now we have counted the expensive rows but never asked *which text they spell*. So we decoded six blocks of the Wikidata dump, followed every row back to the byte it produces, and sorted those bytes by kind: JSON keys, numbers, words, and the long random identifiers that Wikidata attaches to its records.

**What we found.** The repetitive part of the JSON, which looks like the obvious target, is already nearly free: keys such as `"mainsnak"` are about a third of the text but about 1% of the rows that stand alone. Those rows belong overwhelmingly to random strings. The 36-character IDs that Wikidata gives each statement are about 5% of the text and about 62% of the lone rows. The 40-character hashes that identify a reference are a different case: each one is used several times, so its rows come in small groups, one row per use.

**Why it matters.** It says where a better decoder must win. Collapsing repeated structure is what the current decoder already does well. What remains splits into random strings that repeat, which could be copied once one copy is decoded, and random strings that don't, which no amount of recognising structure can help with. This post builds the measurement up from one example ([section 1](<#1-one-snippet-character-by-character>)), gives the totals over six blocks ([section 2](<#2-the-totals-over-six-blocks>)), explains why random strings behave this way ([section 3](<#3-why-random-strings-stand-alone>)), says why we went looking ([section 4](<#4-why-look-at-the-text-at-all>)), and ends with the questions it opens ([section 5](<#5-what-comes-next>)).

## 1\. One snippet, character by character

A quick recap of the setting, which [Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-798/blog/2026-11-19-open-returns/index.md>) works through in full. bzip2 sorts all the rotations of a block of text and stores the last column of the sorted table. The decoder rebuilds the text by walking that column backwards: from each row, a step called LF finds the row holding the previous byte. Equal bytes on neighbouring rows form a **run**. A run's rows move together under LF, so mibz2's current kernel, Open, carries a wide run as one unit. The rows of narrow runs it keeps individually and walks one by one; they are about a tenth of a block's rows, and walking them is most of the decode's cost. Below we call a run of fewer than 16 rows **thin**, which is close to, though not exactly, the set of rows Open keeps.

Here is what that looks like on real text. The figure shows three snippets of block 0. Each character is shaded by the width of the run that holds it, so a wide cell is cheap and a dark cell costs a step of its own.

Three snippets of block 0, each character shaded by the width of its run

Measured on block 0 of the admitted Wikidata blocks. The JSON keys sit in runs of hundreds to tens of thousands of rows. Every byte of a hash used six times sits in a run of six or seven. A statement ID's characters sit in runs of one or two from its first digit.

The three snippets behave in three different ways:

- **The JSON skeleton** (`"mainsnak":{"snaktype":"value","property":"P`) sits mostly in runs of 763 to 26,407 rows. The same bytes, followed by the same text, appear thousands of times in the block, so their rows lie side by side in the sorted table and travel as one group.
- **A reference hash used six times** sits in runs of six (twice seven). Each of its 40 hex digits is followed by the same rest-of-hash in all six places, so the six rows sort together. They form a thin run: six rows, not one, but far from the thousands of the skeleton.
- **A statement ID** sits in runs of one or two from its first digit. Its 32 hex digits are random, and no other text in the block continues the same way, so almost every row is alone.

## 2\. The totals over six blocks

The snippet is one example. The measurement covers every byte of the six blocks, about 5.4 million bytes. A short script ([`costmap.py`](<https://github.com/lmmx/mibz2/blob/2b2499d89/docs/journal/2026-11-20-working-backwards/costmap.py>)) classifies each byte by a pattern: statement IDs, 40-digit hashes, entity IDs such as `Q42`, other numbers, JSON keys, other letters, and the remaining punctuation. The chart compares each kind's share of the text with its share of the thin-run rows and of the one-row runs.

Share of the text against share of the expensive rows, by kind of text

Measured; each bar is the mean over the six blocks. The classes are pattern matches, and "thin" means a run of fewer than 16 rows, an approximation to the rows Open keeps. The per-block figures are in the [research journal](<https://github.com/lmmx/mibz2/blob/2b2499d89/docs/journal/2026-11-20-working-backwards.md>).

The per-block ranges tell the same story as the means. Statement IDs are 3–8.5% of the text in each block but 54–67% of the one-row runs. JSON keys are 33–38% of the text but 1–4% of the thin-run rows. IDs and hashes together hold 32–53% of the thin-run rows. The other kinds, numbers, words and entity IDs, cost at most about twice their share of the text. A statement ID's share of the one-row runs is about thirteen times its share of the text.

## 3\. Why random strings stand alone

The table's rows are sorted by the text that follows each byte. Two rows sit next to each other when their following text agrees for a long stretch. A run of width 6 therefore means six places in the block where the same byte is followed by the same text; a run of width 1 means that combination happens once.

Repeated structure gives wide runs: a key that appears 3,000 times, followed by the same punctuation and the same next key, puts 3,000 rows in one run. That is why the skeleton is cheap. It is also why bzip2 compresses this kind of text well: a few wide runs describe a lot of text.

A random string does the opposite. A statement ID is 32 random hexadecimal digits. The row of a byte inside it is sorted by the text that follows that byte, which is the rest of the ID. Even four random hex digits allow 65,536 continuations, and the block holds only about 760 IDs, so that following text is almost always unique and the row stands alone. The same is true of a hash the first time it is used. But a hash used six times has six rows for each digit, all followed by identical text, so it forms runs of six. Its cost is in the thin runs, not the one-row runs, which is exactly where the chart puts it: in block 0, 841 hash fields hold only 137 distinct hashes.

Three kinds of text, three kinds of cost

Conceptual. The three kinds need three different treatments, which is why one decoder that treats every thin row alike leaves something on the table.

None of this is special to Wikidata. JSON from logs, APIs and database dumps has the same shape: a repetitive skeleton with random identifiers and hashes threaded through it. The measurement uses Wikidata's text, but the conclusion is about that shape.

## 4\. Why look at the text at all?

The question came from a contrast with the [Hutter Prize](<http://prize.hutter1.net/>), which rewards the smallest self-extracting description of a gigabyte of English Wikipedia. Its rules allow decompression to take tens of hours on one core. mibz2 is the mirror image: the description is fixed by bzip2, and the goal is the least computation to recover the text. A short description is not the same as a cheap one to use. Our own research has met that gap directly: an earlier experiment shrank the decoder's lookup table to a compact form and found the decode 1.7 to 2.8 times slower, because each step took several dependent lookups instead of one.

One Hutter winner in particular suggested the question. STARLIT ([Margaritov, 2021](<https://github.com/amargaritov/starlit>)) did not improve the compressor at all. It reordered Wikipedia's articles so that similar articles sat together, and the unchanged compressor then did better. bzip2's encoder has already done a reordering of its own, sorting every byte by the text that follows it. We asked what that reordering exposes about the text, working backwards from the structure we know the text has to what it costs to decode. The first answer is the one above: the structure we expected to exploit is already exploited, and the cost sits in the parts of the text that have no structure.

## 5\. What comes next

The measurement turns a vague idea, a "zoo" of decoding methods each suited to a different kind of text, into three concrete questions for the research line:

1. **Repeated random strings.** A hash's six copies spell the same 40 bytes. If the decoder could find out cheaply that a thin run's rows are copies of one another, it could decode one copy and write the others. Finding that out from what the decoder already holds, before the text exists, is the hard part, and its cost must be counted.
2. **Unique random strings.** Statement IDs have nothing to share, so their one-row runs may be a true floor for any method that works by recognising repetition. Is there an operation on them that is not a step-by-step walk, or can we prove there isn't one, for a stated class of methods?
3. **A method per kind.** Before building any classifier, measure the best case: if each block, or each part of a block, could use whichever method suits it best, how much would that save? An earlier attempt to tune Open per block found less than 1% to gain even with hindsight, so this is the number to know first.

These are now recorded as a lead for the research line. The measurement script is in the [research journal](<https://github.com/lmmx/mibz2/blob/2b2499d89/docs/journal/2026-11-20-working-backwards.md>), and anyone can rerun it on the six blocks.

---

**Research records:**

- [Working backwards from the text](<https://github.com/lmmx/mibz2/blob/2b2499d89/docs/journal/2026-11-20-working-backwards.md>): the discussion, the per-block table and `costmap.py`.
- [The run network](<https://github.com/lmmx/mibz2/blob/2b2499d89/docs/journal/2026-11-20-run-network.md>): the runs as a graph, and the longest chains of thin runs.
- [Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-798/blog/2026-11-19-open-returns/index.md>): the current kernel, and which rows it keeps.
- [The Hutter Prize](<http://prize.hutter1.net/>) and [STARLIT](<https://github.com/amargaritov/starlit>).
