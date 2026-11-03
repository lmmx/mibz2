---
title: "Revisiting the space–time tradeoff in BWT inversion"
image: assets/social/blog-2026-11-02-smaller-table-four-workers.png
description: "A packed table saved memory but cost too much to read. A cheaper compact lookup gives the first positive traversal result."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Revisiting the space–⁠time tradeoff in BWT inversion

In 2001, Seward showed why the fastest way to undo a Burrows–Wheeler transform could use more
instructions but less memory traffic. Twenty-five years later, that observation led us to
ask whether a smaller navigation table could do more work per step and still come out ahead
under contention. A first measurement of ours shows it doing so.
{: .post-dek }

2 November 2026 · 25 min read · [Result](#7-timing-the-walk-on-one-core-and-on-four)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

In 2001, [Julian Seward](https://en.wikipedia.org/wiki/Julian_Seward), the author of
[bzip2](https://en.wikipedia.org/wiki/Bzip2) and the original designer of
[Valgrind](https://valgrind.org/info/developers.html), was working at Microsoft Research in
Cambridge. That year he presented a paper at the IEEE Data Compression Conference,
*[Space-time tradeoffs in the Inverse B-W
Transform](https://doi.org/10.1109/DCC.2001.917175)*, on what it costs to undo the
[Burrows–Wheeler transform](https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform)
(BWT) that bzip2 is built on.

Compression with the BWT had been studied closely, but
*"decompression-time issues remain underinvestigated"*. He had two uses in mind that pull in opposite directions: booting a
Linux kernel from a compressed image needs little memory, and an on-the-fly disk compressor
needs speed. So he offers a family of algorithms
that trade one for the other. At one extreme, the BWT can be inverted, slowly, in one byte of
memory per byte of block. At the other:

> we argue that the maximum speed of BWT decompression is unavoidably constrained by the rate
> at which cache misses are serviced.

mibz2 is our bzip2 decoder, written for the Wikidata dumps, and that sentence could stand
over most of what we have written about it. Our posts on [32
lanes](2026-10-31-room-walk-overlap.md) and [memory-level
parallelism](2026-11-02-memory-level-parallelism.md) are about keeping many misses in flight
at once; [memory interference](2026-11-01-inverse-bwt-memory-interference.md) is about four
workers' misses competing for one memory system; [compact navigation
tables](2026-10-29-compact-navigation-tables.md) is about taking fewer misses with a smaller
table. Seward said as much in [bzip2's manual](https://sourceware.org/bzip2/manual/manual.html):
*"performance, both for compressing and decompressing, is largely determined by the speed at
which your machine can service cache misses."*

The post has two halves: how Seward's algorithms work and what he measured, then what happens
to his tradeoff in mibz2, where several blocks decode at once and share one cache. The
result in the second half is a controlled test of the walk alone, not yet of the whole
decoder.

<nav class="post-roadmap" aria-label="The post in nine sections">
<div class="rm-half">
<p class="post-roadmap-part">Seward's paper, 2001</p>
<ol>
<li><a href="#1-from-sorted-rotations-to-the-transformation-vector"><span class="rm-n">1</span><span class="rm-text"><span class="rm-t">From sorted rotations to the transformation vector</span><span class="rm-d">How a decoder gets <code>banana</code> back: a chain of lookups, each depending on the last</span></span></a></li>
<li><a href="#2-making-inversion-fast-basis-bw94-and-mergedtl"><span class="rm-n">2</span><span class="rm-text"><span class="rm-t">Making inversion fast: basis, bw94 and mergedTL</span><span class="rm-d">Keeping the byte and the link in one word</span></span></a></li>
<li><a href="#3-spending-less-memory-indexf-and-the-count-trees"><span class="rm-n">3</span><span class="rm-text"><span class="rm-t">Spending less memory: indexF and the count trees</span><span class="rm-d">Recomputing what the fast algorithms store</span></span></a></li>
<li><a href="#4-the-spacetime-tradeoff"><span class="rm-n">4</span><span class="rm-text"><span class="rm-t">The space–time tradeoff</span><span class="rm-d">Seward's measurements of memory against speed</span></span></a></li>
</ol>
</div>
<div class="rm-half">
<p class="post-roadmap-part">mibz2, 2026</p>
<ol class="rm-ours">
<li><a href="#5-four-blocks-sharing-one-cache"><span class="rm-n">5</span><span class="rm-text"><span class="rm-t">Four blocks sharing one cache</span><span class="rm-d">Why Seward's tradeoff changes when blocks decode at once</span></span></a></li>
<li><a href="#6-a-smaller-table-a-busier-lookup"><span class="rm-n">6</span><span class="rm-text"><span class="rm-t">A smaller table, a busier lookup</span><span class="rm-d">A little over half the space, more work per lookup</span></span></a></li>
<li><a href="#7-timing-the-walk-on-one-core-and-on-four"><span class="rm-n">7</span><span class="rm-text"><span class="rm-t">Timing the walk on one core and on four</span><span class="rm-d">The walk alone: slower on one core, faster on four</span></span></a></li>
<li><a href="#8-four-workers-cost-the-conventional-walk-far-more"><span class="rm-n">8</span><span class="rm-text"><span class="rm-t">Four workers cost the conventional walk far more</span><span class="rm-d">Why the usual table pays more for sharing the cache</span></span></a></li>
<li><a href="#9-the-table-still-has-to-be-built"><span class="rm-n">9</span><span class="rm-text"><span class="rm-t">The table still has to be built</span><span class="rm-d">A cheap way to construct it for the decoder</span></span></a></li>
</ol>
</div>
</nav>

## 1. From sorted rotations to the transformation vector

Start with the word `banana` and see what the Burrows–Wheeler transform does to it.

First, write out six versions of the word, each starting at a different letter and wrapping
round to the beginning when it reaches the end: `banana`, `ananab`, `nanaba`, `anaban`,
`nabana`, `abanan`. These are the word's *cyclic rotations*: the same six letters, each time
in a different order. (As permutations of the positions, they are the six powers of one, $\sigma = (0\;1\;2\;3\;4\;5)$
in cycle notation: $\sigma^0$, the identity, is the word itself, and $\sigma^k$ starts it $k$
letters in. Stacked
so that each is the one above shifted one place to the right, they would form a [circulant
matrix](https://en.wikipedia.org/wiki/Circulant_matrix); sorting only reorders those rows.) Then sort the six rotations alphabetically and stack them as rows:

<figure class="bf-fig">
<p class="bf-title">The six rotations of banana, sorted</p>
<svg viewBox="0 0 720 350" role="img" aria-label="The six rotations of banana, sorted alphabetically into rows 0 to 5: abanan, anaban, ananab, banana, nabana, nanaba. The first column reads a a a b n n; the last column reads n n b a a a. Row 3 is banana itself.">
<text x="96" y="38" class="bf-ts" text-anchor="middle">row</text>
<text x="179.0" y="38" class="bf-ts" text-anchor="middle">first</text>
<text x="499.0" y="38" class="bf-ts" text-anchor="middle">last</text>
<text x="96" y="79" class="bf-t" text-anchor="middle">0</text>
<rect x="150" y="56" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="214" y="56" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="278" y="56" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="342" y="56" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="406" y="56" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="56" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="79" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="96" y="119" class="bf-t" text-anchor="middle">1</text>
<rect x="150" y="96" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="214" y="96" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="278" y="96" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="342" y="96" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="406" y="96" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="96" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="119" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="96" y="159" class="bf-t" text-anchor="middle">2</text>
<rect x="150" y="136" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="214" y="136" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="278" y="136" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="342" y="136" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="406" y="136" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="136" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="159" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<text x="96" y="199" class="bf-t" text-anchor="middle">3</text>
<rect x="150" y="176" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="214" y="176" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="278" y="176" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="342" y="176" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="406" y="176" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="470" y="176" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="199" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="96" y="239" class="bf-t" text-anchor="middle">4</text>
<rect x="150" y="216" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="214" y="216" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="278" y="216" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="342" y="216" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="406" y="216" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="470" y="216" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="239" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="96" y="279" class="bf-t" text-anchor="middle">5</text>
<rect x="150" y="256" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="179.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="214" y="256" width="58" height="34" rx="5" class="bf-cell"/>
<text x="243.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="278" y="256" width="58" height="34" rx="5" class="bf-cell"/>
<text x="307.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="342" y="256" width="58" height="34" rx="5" class="bf-cell"/>
<text x="371.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="406" y="256" width="58" height="34" rx="5" class="bf-cell"/>
<text x="435.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="470" y="256" width="58" height="34" rx="5" class="bf-slot"/>
<text x="499.0" y="279" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="70" y="171" width="468" height="44" rx="7" class="bf-box" style="stroke-width: 2"/>
<text x="554" y="199" class="bf-t-strong">the original word</text>
<text x="554" y="279" class="bf-t-strong">nnbaaa, read down</text>
<path d="M530 274H548" class="bf-link"/>
<text x="70" y="318" class="bf-tm">The transform keeps the last column, nnbaaa, and the number of the row holding banana, 3.</text>
<text x="70" y="338" class="bf-tm">Nothing else of the square is stored.</text>
</svg>
<figcaption>The rotations of banana in alphabetical order. The first column is shaded green and the last blue; row 3, outlined, is banana itself.</figcaption>
</figure>

Read down the last column of the sorted rows, and it spells `nnbaaa`. The original `banana`,
$\sigma^0$, is in row 3, counting from zero. The transform keeps just those two things, the last column and
the row number, and stores nothing else. It doesn't make the block smaller; the reordered
letters compress better in bzip2's later stages, for a reason that comes below.

How can a decoder get `banana` back from only `nnbaaa` and 3? Each of Seward's algorithms
answers that question in a different way.

Seward names the pieces with letters, following Burrows and Wheeler's 1994 report *[A
Block-sorting Lossless Data Compression
Algorithm](https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf)*. On the same
square:

<figure class="bf-fig">
<p class="bf-title">Seward's names for the pieces</p>
<svg viewBox="0 0 720 360" role="img" aria-label="The same sorted square, labelled with Seward's symbols. M is the whole square of sorted rotations, N by N with N = 6. F is its first column, aaabnn. L is its last column, nnbaaa. S is the original block, banana, which sits in row I = 3.">
<text x="238" y="113" class="bf-tm" text-anchor="end">0</text>
<rect x="250" y="92" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="304" y="92" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="358" y="92" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="412" y="92" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="466" y="92" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="520" y="92" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="113" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="238" y="149" class="bf-tm" text-anchor="end">1</text>
<rect x="250" y="128" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="304" y="128" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="358" y="128" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="412" y="128" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="466" y="128" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="520" y="128" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="149" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="238" y="185" class="bf-tm" text-anchor="end">2</text>
<rect x="250" y="164" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="304" y="164" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="358" y="164" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="412" y="164" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="466" y="164" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="520" y="164" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="185" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<text x="238" y="221" class="bf-tm" text-anchor="end">3</text>
<rect x="250" y="200" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="304" y="200" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="358" y="200" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="412" y="200" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="466" y="200" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="520" y="200" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="221" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="238" y="257" class="bf-tm" text-anchor="end">4</text>
<rect x="250" y="236" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="304" y="236" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="358" y="236" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="412" y="236" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="466" y="236" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="520" y="236" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="238" y="293" class="bf-tm" text-anchor="end">5</text>
<rect x="250" y="272" width="49" height="31" rx="4" class="bf-cell-hi"/>
<text x="274.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="304" y="272" width="49" height="31" rx="4" class="bf-cell"/>
<text x="328.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="358" y="272" width="49" height="31" rx="4" class="bf-cell"/>
<text x="382.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="412" y="272" width="49" height="31" rx="4" class="bf-cell"/>
<text x="436.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="466" y="272" width="49" height="31" rx="4" class="bf-cell"/>
<text x="490.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="520" y="272" width="49" height="31" rx="4" class="bf-slot"/>
<text x="544.5" y="293" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="246" y="196" width="327" height="39" rx="6" class="bf-box" style="stroke-width: 2"/>
<text x="274.5" y="34" class="bf-t-strong" text-anchor="middle" style="font-style: italic; font-size: 20px">F</text>
<text x="274.5" y="54" class="bf-tm" text-anchor="middle">first column: aaabnn</text>
<path d="M274.5 61V88" class="bf-link"/>
<text x="544.5" y="34" class="bf-t-strong" text-anchor="middle" style="font-style: italic; font-size: 20px">L</text>
<text x="544.5" y="54" class="bf-tm" text-anchor="middle">last column: nnbaaa</text>
<path d="M544.5 61V88" class="bf-link"/>
<path d="M224 92H216V303H224" class="bf-link"/>
<text x="204" y="168" class="bf-t-strong" text-anchor="end" style="font-style: italic; font-size: 20px">M</text>
<text x="204" y="190" class="bf-tm" text-anchor="end">every rotation of S,</text>
<text x="204" y="207" class="bf-tm" text-anchor="end">sorted: N × N</text>
<text x="204" y="224" class="bf-tm" text-anchor="end">letters, N = 6</text>
<path d="M576 214H604" class="bf-link"/>
<text x="612" y="208" class="bf-t-strong" style="font-style: italic; font-size: 20px">S</text>
<text x="632" y="208" class="bf-tm">the block: banana</text>
<text x="612" y="232" class="bf-t-strong" style="font-style: italic; font-size: 20px">I</text>
<text x="632" y="232" class="bf-tm">its row: 3</text>
<text x="204" y="334" class="bf-tm">Stored by bzip2: L and I. Everything else here, the decoder has to work out.</text>
</svg>
<figcaption>The square of sorted rotations again, with each symbol pointing at what it names. In bzip2, N can be up to 900,000.</figcaption>
</figure>

So the decoder is given $L$ and $I$ and must produce $S$. The square has been a way to
understand the transform, but the decoder never builds it, and that matters for everything
that follows. For a block of a million bytes, $M$ would be a million rows of a million
letters: a terabyte. The decoder keeps only $L$, one byte a row, and whatever it needs to
find its way from one row to the next. None of Seward's algorithms builds $M$; what separates
them is how much of that navigation they store, and in what form. That is the space in his
space–time tradeoff.

**Why only $L$ is stored.** Of the square's two outer columns, bzip2 keeps only the last:

- **$F$** is the letters of the word in sorted order, so the counts alone fix it. A decoder
  holding $L$ can rebuild $F$ by counting, so it is never sent.
- **$L$** holds the letter that comes before each row's opening, with the rows sorted by what
  follows. Letters before similar text tend to repeat (in English, `he ` is often preceded by
  `t`), so $L$ has long runs, which bzip2's later stages compress well.

**Moving backwards through the word.** Look again at row 3, `banana`. Move its last letter to
the front and you get `abanan`. That is another rotation of the word, so it is already in the
square, at row 0. Do the same to `abanan` and you get `nabana`, at row 4. Each move steps one
letter backwards through the word, treating its end as joined to its beginning, as on a ring:

<figure class="bf-fig">
<p class="bf-title">Stepping backwards round the word</p>
<svg class="rg-fig" viewBox="0 0 720 350" role="img" aria-label="An animation on banana written round a circle. Each row of the square reads the word clockwise from one letter. Starting from row 3, which reads from position 0, each move of the last letter to the front starts the reading one letter earlier: row 0 from position 5, row 4 from 4, row 1 from 3, row 5 from 2, row 2 from 1. Each step lights the starting letter and the letter just before it, the row's last letter.">
<style>@keyframes rg-s0 { 0%, 0.00% { opacity: 0 } 2.84%, 10.80% { opacity: 1 } 13.64%, 100% { opacity: 0 } } @keyframes rg-l0 { 0%, 0.00% { opacity: 0 } 2.84%, 10.80% { opacity: 1 } 13.64%, 100% { opacity: 0 } } @keyframes rg-r0 { 0%, 0.00% { opacity: 0 } 2.84%, 10.80% { opacity: 1 } 13.64%, 100% { opacity: 0 } } @keyframes rg-a0 { 0%, 5.11% { opacity: 0 } 7.95%, 13.64% { opacity: 1 } 16.48%, 100% { opacity: 0 } } @keyframes rg-s1 { 0%, 13.64% { opacity: 0 } 16.48%, 24.43% { opacity: 1 } 27.27%, 100% { opacity: 0 } } @keyframes rg-l1 { 0%, 13.64% { opacity: 0 } 16.48%, 24.43% { opacity: 1 } 27.27%, 100% { opacity: 0 } } @keyframes rg-r1 { 0%, 13.64% { opacity: 0 } 16.48%, 24.43% { opacity: 1 } 27.27%, 100% { opacity: 0 } } @keyframes rg-a1 { 0%, 18.75% { opacity: 0 } 21.59%, 27.27% { opacity: 1 } 30.11%, 100% { opacity: 0 } } @keyframes rg-s2 { 0%, 27.27% { opacity: 0 } 30.11%, 38.07% { opacity: 1 } 40.91%, 100% { opacity: 0 } } @keyframes rg-l2 { 0%, 27.27% { opacity: 0 } 30.11%, 38.07% { opacity: 1 } 40.91%, 100% { opacity: 0 } } @keyframes rg-r2 { 0%, 27.27% { opacity: 0 } 30.11%, 38.07% { opacity: 1 } 40.91%, 100% { opacity: 0 } } @keyframes rg-a2 { 0%, 32.39% { opacity: 0 } 35.23%, 40.91% { opacity: 1 } 43.75%, 100% { opacity: 0 } } @keyframes rg-s3 { 0%, 40.91% { opacity: 0 } 43.75%, 51.70% { opacity: 1 } 54.55%, 100% { opacity: 0 } } @keyframes rg-l3 { 0%, 40.91% { opacity: 0 } 43.75%, 51.70% { opacity: 1 } 54.55%, 100% { opacity: 0 } } @keyframes rg-r3 { 0%, 40.91% { opacity: 0 } 43.75%, 51.70% { opacity: 1 } 54.55%, 100% { opacity: 0 } } @keyframes rg-a3 { 0%, 46.02% { opacity: 0 } 48.86%, 54.55% { opacity: 1 } 57.39%, 100% { opacity: 0 } } @keyframes rg-s4 { 0%, 54.55% { opacity: 0 } 57.39%, 65.34% { opacity: 1 } 68.18%, 100% { opacity: 0 } } @keyframes rg-l4 { 0%, 54.55% { opacity: 0 } 57.39%, 65.34% { opacity: 1 } 68.18%, 100% { opacity: 0 } } @keyframes rg-r4 { 0%, 54.55% { opacity: 0 } 57.39%, 65.34% { opacity: 1 } 68.18%, 100% { opacity: 0 } } @keyframes rg-a4 { 0%, 59.66% { opacity: 0 } 62.50%, 68.18% { opacity: 1 } 71.02%, 100% { opacity: 0 } } @keyframes rg-s5 { 0%, 68.18% { opacity: 0 } 71.02%, 93.75% { opacity: 1 } 96.59%, 100% { opacity: 0 } } @keyframes rg-l5 { 0%, 68.18% { opacity: 0 } 71.02%, 93.75% { opacity: 1 } 96.59%, 100% { opacity: 0 } } @keyframes rg-r5 { 0%, 68.18% { opacity: 0 } 71.02%, 93.75% { opacity: 1 } 96.59%, 100% { opacity: 0 } } @keyframes rg-b0 { 0%, 0.00% { opacity: 0 } 2.84%, 10.80% { opacity: 1 } 13.64%, 100% { opacity: 0 } } @keyframes rg-b1 { 0%, 13.64% { opacity: 0 } 16.48%, 24.43% { opacity: 1 } 27.27%, 100% { opacity: 0 } } @keyframes rg-b2 { 0%, 27.27% { opacity: 0 } 30.11%, 38.07% { opacity: 1 } 40.91%, 100% { opacity: 0 } } @keyframes rg-b3 { 0%, 40.91% { opacity: 0 } 43.75%, 51.70% { opacity: 1 } 54.55%, 100% { opacity: 0 } } @keyframes rg-b4 { 0%, 54.55% { opacity: 0 } 57.39%, 65.34% { opacity: 1 } 68.18%, 100% { opacity: 0 } } @keyframes rg-b5 { 0%, 68.18% { opacity: 0 } 71.02%, 93.75% { opacity: 1 } 96.59%, 100% { opacity: 0 } } @media (prefers-reduced-motion: reduce) { svg.rg-fig .rg-hl { animation: none !important; opacity: 0 } svg.rg-fig .rg-keep { animation: none !important; opacity: 1 } }</style>
<path d="M153.6 149.0A42 42 0 0 1 226.4 149.0" class="bf-grid" style="stroke-width: 1.5"/>
<path d="M226.4 149.0l-8 -2M226.4 149.0l-1 -8" class="bf-grid" style="stroke-width: 1.5"/>
<text x="190" y="178" class="bf-tm" text-anchor="middle">each row</text>
<text x="190" y="194" class="bf-tm" text-anchor="middle">reads clockwise</text>
<circle cx="190.0" cy="82.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<circle cx="266.2" cy="126.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<circle cx="266.2" cy="214.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<circle cx="190.0" cy="258.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<circle cx="113.8" cy="214.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<circle cx="113.8" cy="126.0" r="20" class="bf-cell" style="stroke: var(--bf-frame)"/>
<text x="190.0" y="120.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">0</text>
<text x="236.8" y="147.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">1</text>
<text x="236.8" y="201.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">2</text>
<text x="190.0" y="228.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">3</text>
<text x="143.2" y="201.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">4</text>
<text x="143.2" y="147.0" class="bf-ttiny" text-anchor="middle" style="fill: var(--bf-muted)">5</text>
<circle cx="190.0" cy="82.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s0 8.80s linear infinite"/>
<circle cx="113.8" cy="126.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l0 8.80s linear infinite"/>
<text x="190.0" y="25.0" class="bf-t-strong rg-keep" text-anchor="middle" style="animation: rg-r0 8.80s linear infinite">row 3</text>
<path d="M168.5 47.9A124 124 0 0 0 95.0 90.3M95.0 90.3L97.0 80.2M95.0 90.3L104.6 86.6" class="bf-link rg-keep" style="stroke-width: 2; animation: rg-a0 8.80s linear infinite"/>
<circle cx="113.8" cy="126.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s1 8.80s linear infinite"/>
<circle cx="113.8" cy="214.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l1 8.80s linear infinite"/>
<text x="60.1" y="100.0" class="bf-t-strong rg-keep" text-anchor="end" style="animation: rg-r1 8.80s linear infinite">row 0</text>
<path d="M73.5 127.6A124 124 0 0 0 73.5 212.4M73.5 212.4L65.7 205.7M73.5 212.4L75.1 202.2" class="bf-link rg-keep" style="stroke-width: 2; animation: rg-a1 8.80s linear infinite"/>
<circle cx="113.8" cy="214.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s2 8.80s linear infinite"/>
<circle cx="190.0" cy="258.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l2 8.80s linear infinite"/>
<text x="60.1" y="250.0" class="bf-t-strong rg-keep" text-anchor="end" style="animation: rg-r2 8.80s linear infinite">row 4</text>
<path d="M95.0 249.7A124 124 0 0 0 168.5 292.1M168.5 292.1L158.7 295.5M168.5 292.1L160.5 285.6" class="bf-link rg-keep" style="stroke-width: 2; animation: rg-a2 8.80s linear infinite"/>
<circle cx="190.0" cy="258.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s3 8.80s linear infinite"/>
<circle cx="266.2" cy="214.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l3 8.80s linear infinite"/>
<text x="190.0" y="325.0" class="bf-t-strong rg-keep" text-anchor="middle" style="animation: rg-r3 8.80s linear infinite">row 1</text>
<path d="M211.5 292.1A124 124 0 0 0 285.0 249.7M285.0 249.7L283.0 259.8M285.0 249.7L275.4 253.4" class="bf-link rg-keep" style="stroke-width: 2; animation: rg-a3 8.80s linear infinite"/>
<circle cx="266.2" cy="214.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s4 8.80s linear infinite"/>
<circle cx="266.2" cy="126.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l4 8.80s linear infinite"/>
<text x="319.9" y="250.0" class="bf-t-strong rg-keep" text-anchor="start" style="animation: rg-r4 8.80s linear infinite">row 5</text>
<path d="M306.5 212.4A124 124 0 0 0 306.5 127.6M306.5 127.6L314.3 134.3M306.5 127.6L304.9 137.8" class="bf-link rg-keep" style="stroke-width: 2; animation: rg-a4 8.80s linear infinite"/>
<circle cx="266.2" cy="126.0" r="20" class="bf-cell-hi rg-hl" style="stroke: var(--bf-green); animation: rg-s5 8.80s linear infinite"/>
<circle cx="190.0" cy="82.0" r="20" class="bf-slot rg-hl" style="stroke: var(--bf-steel); animation: rg-l5 8.80s linear infinite"/>
<text x="319.9" y="100.0" class="bf-t-strong rg-keep" text-anchor="start" style="animation: rg-r5 8.80s linear infinite">row 2</text>
<text x="190.0" y="88.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<text x="266.2" y="132.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="266.2" y="220.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="190.0" y="264.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="113.8" y="220.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="113.8" y="132.0" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="410" y="60" class="bf-t-strong">Each move starts the reading one letter earlier</text>
<rect x="402" y="75" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b0 8.80s linear infinite"/>
<text x="410" y="92" class="bf-tm">row 3 reads from position 0</text>
<text x="610" y="92" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">banan<tspan style="fill: var(--bf-steel); font-weight: 700">a</tspan></text>
<rect x="402" y="103" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b1 8.80s linear infinite"/>
<text x="410" y="120" class="bf-tm">row 0 reads from position 5</text>
<text x="610" y="120" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">abana<tspan style="fill: var(--bf-steel); font-weight: 700">n</tspan></text>
<rect x="402" y="131" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b2 8.80s linear infinite"/>
<text x="410" y="148" class="bf-tm">row 4 reads from position 4</text>
<text x="610" y="148" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">naban<tspan style="fill: var(--bf-steel); font-weight: 700">a</tspan></text>
<rect x="402" y="159" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b3 8.80s linear infinite"/>
<text x="410" y="176" class="bf-tm">row 1 reads from position 3</text>
<text x="610" y="176" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">anaba<tspan style="fill: var(--bf-steel); font-weight: 700">n</tspan></text>
<rect x="402" y="187" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b4 8.80s linear infinite"/>
<text x="410" y="204" class="bf-tm">row 5 reads from position 2</text>
<text x="610" y="204" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">nanab<tspan style="fill: var(--bf-steel); font-weight: 700">a</tspan></text>
<rect x="402" y="215" width="290" height="24" rx="5" class="bf-cell-hi rg-hl" style="animation: rg-b5 8.80s linear infinite"/>
<text x="410" y="232" class="bf-tm">row 2 reads from position 1</text>
<text x="610" y="232" class="bf-t" style="font-family: var(--md-code-font-family); font-size: 15px">anana<tspan style="fill: var(--bf-steel); font-weight: 700">b</tspan></text>
<text x="410" y="272" class="bf-tm">Each row's last letter, in L, is the letter just</text>
<text x="410" y="290" class="bf-tm">before its start: a, n, a, n, a, b, which is</text>
<text x="410" y="308" class="bf-tm">banana read backwards.</text>
</svg>
<figcaption>banana written round a circle, positions numbered from the top. Every row of the square reads the word clockwise from one letter (green), so its last letter (blue) is the one just before its first. Moving that last letter to the front starts the reading one letter earlier, so the walk steps anticlockwise: rows 3, 0, 4, 1, 5, 2.</figcaption>
</figure>

| Row | Move its last letter to the front | Lands on row |
|---|---|---|
| 3: `banana` | `a` + `banan` | 0: `abanan` |
| 0: `abanan` | `n` + `abana` | 4: `nabana` |
| 4: `nabana` | `a` + `naban` | 1: `anaban` |

The letters moved, `a`, `n`, `a`, are each row's last letter, in $L$, and they spell `banana`
backwards from its end. So a decoder could recover the whole word from $L$ alone, starting at
row $I$, if it knew where each move lands. Seward records the landing rows in $T$, the
*"transformation vector"*: here $T[3] = 0$, $T[0] = 4$ and $T[4] = 1$. As a permutation of the
rows, $T = (3\;0\;4\;1\;5\;2)$: one cycle through all six, so following it from row 3 visits
every row once before coming back, reading the whole word on the way. His theory section, *"a
more detailed exposition of Algorithm D in §2 of [1]"*, puts it in symbols:

\[
M[j, N-1]\; M[j, 0..N-2] = M[i, 0..N-1] \quad\Longrightarrow\quad T[j] = i.
\]

The left-hand side is row $j$ with its last letter moved to the front; $T[j]$ is the row where
that lands.

**Which row to land on.** The moved letter becomes the first letter of the landing row, so
the landing row is one of those that start with it in $F$. But `banana` has three `a`s, so
three rows start with `a`. Moving the `a` off the end of row 3 lands on row 0, not row 1 or 2;
how could a decoder know that without the square?

Because each letter's occurrences keep their order between the two columns. The first `a` in
$L$ (row 3's) lands on the first row starting with `a` (row 0), the second `a` in $L$ on the
second, and so on, for every letter. The sorting guarantees it: two rows that end in the same
letter keep their order when that letter moves to the front, because the rest of each row,
already in sorted order, still decides which comes first.

So the matching can be worked out from $L$ and a count of each letter. Because $F$ is sorted,
its letters occur in runs, three `a`s, one `b` and two `n`s, and those counts describe it
completely, much like [run-length encoding](https://en.wikipedia.org/wiki/Run-length_encoding).
Seward calls the count table $C$; once the counts are accumulated, it says where each letter's
run begins in $F$. The decoder's whole path, from what is stored back to the word:

<figure class="bf-fig">
<p class="bf-title">What the decoder rebuilds</p>
<svg viewBox="0 0 720 150" role="img" aria-label="What the decoder rebuilds, from left to right: counting the bytes of L gives C; expanding the counts gives F; matching each byte of L to the same occurrence in F gives T; walking T from row I gives S.">
<rect x="10" y="30" width="100" height="52" rx="6" class="bf-slot"/>
<text x="60.0" y="51" class="bf-t-strong" text-anchor="middle" style="font-style: italic">L</text>
<text x="60.0" y="70" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 13px">nnbaaa</text>
<rect x="155" y="30" width="100" height="52" rx="6" class="bf-cell"/>
<text x="205.0" y="51" class="bf-t-strong" text-anchor="middle" style="font-style: italic">C</text>
<text x="205.0" y="70" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 13px">a3 b1 n2</text>
<rect x="300" y="30" width="100" height="52" rx="6" class="bf-cell-hi"/>
<text x="350.0" y="51" class="bf-t-strong" text-anchor="middle" style="font-style: italic">F</text>
<text x="350.0" y="70" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 13px">aaabnn</text>
<rect x="445" y="30" width="100" height="52" rx="6" class="bf-cell"/>
<text x="495.0" y="51" class="bf-t-strong" text-anchor="middle" style="font-style: italic">T</text>
<text x="495.0" y="70" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 13px">4 5 3 0 1 2</text>
<rect x="590" y="30" width="120" height="52" rx="6" class="bf-cell"/>
<text x="650.0" y="51" class="bf-t-strong" text-anchor="middle" style="font-style: italic">S</text>
<text x="650.0" y="70" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 13px">banana</text>
<path d="M112 56L152 56M145 51L152 56L145 61" class="bf-link"/>
<text x="132.0" y="44" class="bf-tm" text-anchor="middle">count</text>
<path d="M257 56L297 56M290 51L297 56L290 61" class="bf-link"/>
<text x="277.0" y="44" class="bf-tm" text-anchor="middle">expand</text>
<path d="M402 56L442 56M435 51L442 56L435 61" class="bf-link"/>
<text x="422.0" y="44" class="bf-tm" text-anchor="middle">match</text>
<path d="M547 56L587 56M580 51L587 56L580 61" class="bf-link"/>
<text x="567.0" y="44" class="bf-tm" text-anchor="middle">walk</text>
<path d="M60 82V104H495V84M490 91L495 84L500 91" class="bf-link" style="stroke-dasharray: 4 3"/>
<text x="282" y="122" class="bf-tm" text-anchor="middle">L is used again to match each byte to its occurrence in F; the walk starts at row I = 3</text>
</svg>
<figcaption>Each arrow is a step explained in the rest of this section: counting, expanding the counts into F, matching occurrences to get T, and the walk.</figcaption>
</figure>

**Walking $T$.** Row $T[j]$ starts with the byte row $j$ ends with, which comes just before row $j$'s
first byte in $S$. So $F[T[i]]$ precedes $F[i]$. The decoder has $L$, not $F$, but $L[k] = F[T[k]]$, and
substituting gives a relation in $L$ alone:

\[
L[T[k]] \text{ immediately precedes } L[k] \text{ in } S.
\]

$L[I]$ is the last byte of $S$, so starting at $I$ and following $T$ gives $S$ from the end:

<figure class="bf-fig">
<p class="bf-title">Decoding banana by following T</p>
<svg class="wk-fig" viewBox="0 0 720 400" role="img" aria-label="An animation of the inverse transform on banana. Starting at row 3, the decoder reads L at the current row and writes it into the output from the end, then moves to the row T names: rows 3, 0, 4, 1, 5, 2, emitting a, n, a, n, a, b, which fill the output right to left as banana.">
<style>.wk2-move { transform-box: view-box; } @keyframes wk2-col { 0.00%, 8.14% { transform: translate(0px, 0px) } 11.63%, 19.77% { transform: translate(74px, 0px) } 23.26%, 31.40% { transform: translate(148px, 0px) } 34.88%, 43.02% { transform: translate(222px, 0px) } 46.51%, 54.65% { transform: translate(296px, 0px) } 58.14%, 66.28% { transform: translate(370px, 0px) } 100% { transform: translate(370px, 0px) } } @keyframes wk2-col-o { 0% { opacity: 1 } 66.86% { opacity: 1 } 69.77% { opacity: 0 } 97.09% { opacity: 0 } 100% { opacity: 1 } } @keyframes wk2-b0 { 0%, 4.07% { opacity: 0 } 6.98%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-b1 { 0%, 15.70% { opacity: 0 } 18.60%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-b2 { 0%, 27.33% { opacity: 0 } 30.23%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-b3 { 0%, 38.95% { opacity: 0 } 41.86%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-b4 { 0%, 50.58% { opacity: 0 } 53.49%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-b5 { 0%, 62.21% { opacity: 0 } 65.12%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-r0 { 0%, 0.00% { opacity: 0 } 2.91%, 11.63% { opacity: 1 } 14.53%, 100% { opacity: 0 } } @keyframes wk2-r1 { 0%, 11.63% { opacity: 0 } 14.53%, 23.26% { opacity: 1 } 26.16%, 100% { opacity: 0 } } @keyframes wk2-r2 { 0%, 23.26% { opacity: 0 } 26.16%, 34.88% { opacity: 1 } 37.79%, 100% { opacity: 0 } } @keyframes wk2-r3 { 0%, 34.88% { opacity: 0 } 37.79%, 46.51% { opacity: 1 } 49.42%, 100% { opacity: 0 } } @keyframes wk2-r4 { 0%, 46.51% { opacity: 0 } 49.42%, 58.14% { opacity: 1 } 61.05%, 100% { opacity: 0 } } @keyframes wk2-r5 { 0%, 58.14% { opacity: 0 } 61.05%, 73.84% { opacity: 1 } 76.74%, 100% { opacity: 0 } } @keyframes wk2-o0 { 0%, 4.07% { opacity: 0 } 6.98%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t0 { 0%, 0.00% { opacity: 0 } 2.91%, 8.72% { opacity: 1 } 11.63%, 100% { opacity: 0 } } @keyframes wk2-o1 { 0%, 15.70% { opacity: 0 } 18.60%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t1 { 0%, 11.63% { opacity: 0 } 14.53%, 20.35% { opacity: 1 } 23.26%, 100% { opacity: 0 } } @keyframes wk2-o2 { 0%, 27.33% { opacity: 0 } 30.23%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t2 { 0%, 23.26% { opacity: 0 } 26.16%, 31.98% { opacity: 1 } 34.88%, 100% { opacity: 0 } } @keyframes wk2-o3 { 0%, 38.95% { opacity: 0 } 41.86%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t3 { 0%, 34.88% { opacity: 0 } 37.79%, 43.60% { opacity: 1 } 46.51%, 100% { opacity: 0 } } @keyframes wk2-o4 { 0%, 50.58% { opacity: 0 } 53.49%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t4 { 0%, 46.51% { opacity: 0 } 49.42%, 55.23% { opacity: 1 } 58.14%, 100% { opacity: 0 } } @keyframes wk2-o5 { 0%, 62.21% { opacity: 0 } 65.12%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @keyframes wk2-t5 { 0%, 58.14% { opacity: 0 } 61.05%, 66.86% { opacity: 1 } 69.77%, 100% { opacity: 0 } } @keyframes wk2-done { 0%, 69.77% { opacity: 0 } 72.67%, 97.09% { opacity: 1 } 100.00%, 100% { opacity: 0 } } @media (prefers-reduced-motion: reduce) { svg.wk-fig [style*="animation"] { animation: none !important; opacity: 0 } svg.wk-fig .wk-out { opacity: 1 !important } }</style>
<text x="130" y="34" class="bf-tm" text-anchor="end">row visited</text>
<text x="130" y="74" class="bf-tm" text-anchor="end">byte read</text>
<rect x="144" y="8" width="52" height="84" rx="7" class="bf-box wk2-move" style="animation: wk2-col 8.60s ease-in-out infinite, wk2-col-o 8.60s linear infinite; stroke-width: 2"/>
<rect x="150" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="170.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">3</text>
<rect x="150" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b0 8.60s linear infinite"><rect x="150" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="170.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text></g>
<text x="201" y="34" class="bf-tm" text-anchor="middle">→</text>
<rect x="224" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="244.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">0</text>
<rect x="224" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b1 8.60s linear infinite"><rect x="224" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="244.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text></g>
<text x="275" y="34" class="bf-tm" text-anchor="middle">→</text>
<rect x="298" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="318.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">4</text>
<rect x="298" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b2 8.60s linear infinite"><rect x="298" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="318.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text></g>
<text x="349" y="34" class="bf-tm" text-anchor="middle">→</text>
<rect x="372" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="392.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">1</text>
<rect x="372" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b3 8.60s linear infinite"><rect x="372" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="392.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text></g>
<text x="423" y="34" class="bf-tm" text-anchor="middle">→</text>
<rect x="446" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="466.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">5</text>
<rect x="446" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b4 8.60s linear infinite"><rect x="446" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="466.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text></g>
<text x="497" y="34" class="bf-tm" text-anchor="middle">→</text>
<rect x="520" y="14" width="40" height="30" rx="4" class="bf-cell"/>
<text x="540.0" y="34" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">2</text>
<rect x="520" y="54" width="40" height="30" rx="4" class="bf-cell"/>
<g class="wk-out" style="animation: wk2-b5 8.60s linear infinite"><rect x="520" y="54" width="40" height="30" rx="4" class="bf-slot"/><text x="540.0" y="74" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text></g>
<g transform="translate(0 100)">
<text x="70" y="44" class="bf-ts" text-anchor="middle">j</text>
<text x="132" y="44" class="bf-ts" text-anchor="middle">L[j]</text>
<text x="202" y="44" class="bf-ts" text-anchor="middle">T[j]</text>
<rect x="44" y="166" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r0 8.60s linear infinite"/>
<rect x="44" y="58" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r1 8.60s linear infinite"/>
<rect x="44" y="202" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r2 8.60s linear infinite"/>
<rect x="44" y="94" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r3 8.60s linear infinite"/>
<rect x="44" y="238" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r4 8.60s linear infinite"/>
<rect x="44" y="130" width="210" height="36" rx="6" class="bf-cell-hi" style="animation: wk2-r5 8.60s linear infinite"/>
<text x="70" y="81" class="bf-t" text-anchor="middle">0</text>
<rect x="110" y="62" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="81" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="180" y="62" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="81" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">4</text>
<text x="70" y="117" class="bf-t" text-anchor="middle">1</text>
<rect x="110" y="98" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="117" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="180" y="98" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="117" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">5</text>
<text x="70" y="153" class="bf-t" text-anchor="middle">2</text>
<rect x="110" y="134" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="153" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="180" y="134" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="153" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">3</text>
<text x="70" y="189" class="bf-t" text-anchor="middle">3</text>
<rect x="110" y="170" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="189" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="180" y="170" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="189" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">0</text>
<text x="70" y="225" class="bf-t" text-anchor="middle">4</text>
<rect x="110" y="206" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="225" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="180" y="206" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="225" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">1</text>
<text x="70" y="261" class="bf-t" text-anchor="middle">5</text>
<rect x="110" y="242" width="44" height="28" rx="4" class="bf-slot"/>
<text x="132" y="261" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="180" y="242" width="44" height="28" rx="4" class="bf-cell"/>
<text x="202" y="261" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">2</text>
<text x="330" y="120" class="bf-ts">S, written from the end</text>
<rect x="330" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="355.0" y="214" class="bf-tm" text-anchor="middle">0</text>
<rect x="388" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="413.0" y="214" class="bf-tm" text-anchor="middle">1</text>
<rect x="446" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="471.0" y="214" class="bf-tm" text-anchor="middle">2</text>
<rect x="504" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="529.0" y="214" class="bf-tm" text-anchor="middle">3</text>
<rect x="562" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="587.0" y="214" class="bf-tm" text-anchor="middle">4</text>
<rect x="620" y="150" width="50" height="44" rx="5" class="bf-cell"/>
<text x="645.0" y="214" class="bf-tm" text-anchor="middle">5</text>
<g class="wk-out" style="animation: wk2-o0 8.60s linear infinite"><rect x="620" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="645.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">a</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t0 8.60s linear infinite">Step 1: row 3, emit L[3] = a into S[5]; next row T[3] = 0</text>
<g class="wk-out" style="animation: wk2-o1 8.60s linear infinite"><rect x="562" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="587.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">n</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t1 8.60s linear infinite">Step 2: row 0, emit L[0] = n into S[4]; next row T[0] = 4</text>
<g class="wk-out" style="animation: wk2-o2 8.60s linear infinite"><rect x="504" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="529.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">a</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t2 8.60s linear infinite">Step 3: row 4, emit L[4] = a into S[3]; next row T[4] = 1</text>
<g class="wk-out" style="animation: wk2-o3 8.60s linear infinite"><rect x="446" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="471.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">n</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t3 8.60s linear infinite">Step 4: row 1, emit L[1] = n into S[2]; next row T[1] = 5</text>
<g class="wk-out" style="animation: wk2-o4 8.60s linear infinite"><rect x="388" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="413.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">a</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t4 8.60s linear infinite">Step 5: row 5, emit L[5] = a into S[1]; next row T[5] = 2</text>
<g class="wk-out" style="animation: wk2-o5 8.60s linear infinite"><rect x="330" y="150" width="50" height="44" rx="5" class="bf-slot"/><text x="355.0" y="179" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 20px">b</text></g>
<text x="330" y="254" class="bf-t" style="animation: wk2-t5 8.60s linear infinite">Step 6: row 2, emit L[2] = b into S[0]; done</text>
<text x="330" y="254" class="bf-t-strong" style="animation: wk2-done 8.60s linear infinite">S = banana, recovered by following T from row I = 3</text>
</g>
</svg>
<figcaption>Across the top, the rows the walk visits and the byte it reads at each. Below, each step reads the byte in L at the current row, writes it into S from the end, and moves to the row T names. Every jump depends on the entry just read, which is why the walk is one dependent load after another.</figcaption>
</figure>

That is `banana` backwards, and it is the whole inner loop of the inverse transform: load the
entry at $p$, emit a byte, jump where it says. Where each jump lands depends on the content of
$S$, not on the last jump. In a million-byte block that is a million loads from places the
cache has no reason to hold.

To write $S$ forwards, Seward assumes the compressor reverses each block before sorting. bzip2
doesn't, which a footnote calls *"a design flaw"*; by the manual's account, *"the bzip2 format
was frozen before I properly and fully understood the performance consequences"*. Its decoder
builds the links the other way instead, writing $i$ into slot $s$ where Seward would set
$T[i] = s$ (`tt[cftab[uc]++] |= i << 8`), and following that $T^{-1}$ emits $S$ forwards. mibz2
does the same, so its navigation table holds $T^{-1}$: each entry points forwards, to the row
whose string starts one byte later in $S$.

**Building $T$ from counts.** Since the $k$-th $c$ in $L$ lands on the $k$-th row starting with
$c$, Seward can state the rule directly: *"if $c$ occurs $k$ times in
$L[0..j]$ and $L[j] = c$, then $T[j]$ must be $i$, where $i$ is the position of the $k$-th $c$ in $F$."*

<figure class="bf-fig">
<p class="bf-title">T matches each L byte to the same occurrence in F</p>
<svg viewBox="0 0 720 300" role="img" aria-label="T joins each position j of L to a position T[j] of F holding the same byte: the k-th n in L to the k-th n in F, and likewise for a and b. T is 4, 5, 3, 0, 1, 2.">
<text x="190" y="26" class="bf-ts" text-anchor="middle">L[j]</text>
<text x="490" y="26" class="bf-ts" text-anchor="middle">F[i]</text>
<text x="156" y="62" class="bf-tm" text-anchor="end">j = 0</text>
<rect x="170" y="40" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="61" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="470" y="40" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="61" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="524" y="62" class="bf-tm">i = 0</text>
<text x="156" y="100" class="bf-tm" text-anchor="end">j = 1</text>
<rect x="170" y="78" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="99" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="470" y="78" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="99" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="524" y="100" class="bf-tm">i = 1</text>
<text x="156" y="138" class="bf-tm" text-anchor="end">j = 2</text>
<rect x="170" y="116" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="137" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="470" y="116" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="137" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="524" y="138" class="bf-tm">i = 2</text>
<text x="156" y="176" class="bf-tm" text-anchor="end">j = 3</text>
<rect x="170" y="154" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="175" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="154" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="175" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<text x="524" y="176" class="bf-tm">i = 3</text>
<text x="156" y="214" class="bf-tm" text-anchor="end">j = 4</text>
<rect x="170" y="192" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="213" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="192" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="213" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="524" y="214" class="bf-tm">i = 4</text>
<text x="156" y="252" class="bf-tm" text-anchor="end">j = 5</text>
<rect x="170" y="230" width="40" height="30" rx="4" class="bf-slot"/>
<text x="190" y="251" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="470" y="230" width="40" height="30" rx="4" class="bf-cell-hi"/>
<text x="490" y="251" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="524" y="252" class="bf-tm">i = 5</text>
<path d="M214 55 C320 55 360 207 464 207" class="bf-link"/>
<text x="222" y="51" class="bf-ttiny">T[0] = 4</text>
<path d="M214 93 C320 93 360 245 464 245" class="bf-link"/>
<text x="222" y="89" class="bf-ttiny">T[1] = 5</text>
<path d="M214 131 C320 131 360 169 464 169" class="bf-link"/>
<text x="222" y="127" class="bf-ttiny">T[2] = 3</text>
<path d="M214 169 C320 169 360 55 464 55" class="bf-link"/>
<text x="222" y="165" class="bf-ttiny">T[3] = 0</text>
<path d="M214 207 C320 207 360 93 464 93" class="bf-link"/>
<text x="222" y="203" class="bf-ttiny">T[4] = 1</text>
<path d="M214 245 C320 245 360 131 464 131" class="bf-link"/>
<text x="222" y="241" class="bf-ttiny">T[5] = 2</text>
<text x="110" y="290" class="bf-tm">The k-th a in L is the k-th a in F, and so for every byte: the links never cross within one byte.</text>
</svg>
<figcaption>The k-th a in L goes to the k-th a in F, and likewise for n and b: T = 4 5 3 0 1 2, from counts and one scan.</figcaption>
</figure>

The $c$'s in $F$ form one bucket. Turning the counts in $C$ into running totals gives where each
bucket starts: $C[c]$ becomes the number of bytes smaller than $c$ (for `banana`, a 0, b 3,
n 4). So

\[
T[j] = C[c] + k - 1, \qquad c = L[j],
\]

and one scan of $L$ with a running count per byte value builds $T$. Within each byte value the
links increase with $j$. Every BWT has this ordering, and our compact table depends on it.

## 2. Making inversion fast: basis, bw94 and mergedTL

*"How fast can inversion be?"* Three algorithms, each refining the last, answer it.

**basis** is the theory written out: count, build $T$, walk. It reads $L$ three times and keeps $L$
(one byte a row) and $T$ (four) alive: $5N$ bytes.

**bw94**, named for Burrows and Wheeler's report, merges the first two passes by recording
$T'[i]$, the number of earlier occurrences of $L[i]$, while counting. That is $T$ without its
bucket's start, $T[i] = T'[i] + C[L[i]]$, and the walk adds the start back. Two passes, still
$5N$ bytes.

**mergedTL** comes from looking at bw94's walk, which reads $L[p]$ and $T'[p]$ at the same
arbitrary index: *"Unless both $T$ and $L$ fit in the machine's cache, this could cause two
(capacity) misses per iteration."* So it packs both into one 32-bit word, limiting $N$ to $2^{24}$:

\[
U[i] = 256 \cdot T'[i] + L[i].
\]

The name is literal: $T$ and $L$, merged. Each step unpacks the byte with a mask and the link
with a shift, which costs instructions, *"but the near-halving of the cache miss rate is
potentially valuable"*. The table is also a byte a row smaller: $4N$.

<figure class="bf-fig">
<p class="bf-title">What one row costs, bit by bit</p>
<svg viewBox="0 0 720 300" role="img" aria-label="What each algorithm stores per row, drawn to scale in bits. bw94: a 32-bit T prime in one array and the byte L in a second array. mergedTL: one 32-bit word, T prime in the top 24 bits and L in the low 8. indexF: T alone in 20 bits, a 16-bit array plus a 4-bit nibble, with the byte found by a binary search in the counts C. bzip2 and mibz2: one 32-bit word holding the reversed link in 24 bits and the byte in 8.">
<path d="M150 32V260" class="bf-grid"/>
<text x="150" y="26" class="bf-tm" text-anchor="middle">0 bits</text>
<path d="M246 32V260" class="bf-grid"/>
<text x="246" y="26" class="bf-tm" text-anchor="middle">8</text>
<path d="M342 32V260" class="bf-grid"/>
<text x="342" y="26" class="bf-tm" text-anchor="middle">16</text>
<path d="M438 32V260" class="bf-grid"/>
<text x="438" y="26" class="bf-tm" text-anchor="middle">24</text>
<path d="M534 32V260" class="bf-grid"/>
<text x="534" y="26" class="bf-tm" text-anchor="middle">32 bits</text>
<text x="86" y="60" class="bf-ts" text-anchor="end" style="font-family: var(--md-code-font-family); font-size: 13px">bw94</text>
<text x="140" y="60" class="bf-tm" text-anchor="end">T′[i]</text>
<rect x="150" y="40" width="381" height="30" rx="4" class="bf-cell-hi"/>
<text x="340.5" y="60" class="bf-t" text-anchor="middle" style="font-size: 12px">T′[i], displacement in its bucket</text>
<text x="546" y="60" class="bf-tm">array 1</text>
<text x="140" y="106" class="bf-tm" text-anchor="end">L[i]</text>
<rect x="150" y="86" width="93" height="30" rx="4" class="bf-slot"/>
<text x="196.5" y="106" class="bf-t" text-anchor="middle" style="font-size: 12px">L[i]</text>
<text x="546" y="106" class="bf-tm">array 2: a second line</text>
<text x="86" y="152" class="bf-ts" text-anchor="end" style="font-family: var(--md-code-font-family); font-size: 13px">mergedTL</text>
<text x="140" y="152" class="bf-tm" text-anchor="end">U[i]</text>
<rect x="150" y="132" width="285" height="30" rx="4" class="bf-cell-hi"/>
<text x="292.5" y="152" class="bf-t" text-anchor="middle" style="font-size: 12px">T′[i], 24 bits</text>
<rect x="438" y="132" width="93" height="30" rx="4" class="bf-slot"/>
<text x="484.5" y="152" class="bf-t" text-anchor="middle" style="font-size: 12px">L[i]</text>
<text x="546" y="152" class="bf-tm">one word: U = 256·T′ + L</text>
<text x="86" y="198" class="bf-ts" text-anchor="end" style="font-family: var(--md-code-font-family); font-size: 13px">indexF</text>
<text x="140" y="198" class="bf-tm" text-anchor="end">A[i]</text>
<rect x="150" y="178" width="189" height="30" rx="4" class="bf-cell-hi"/>
<text x="244.5" y="198" class="bf-t" text-anchor="middle" style="font-size: 12px">low 16 bits of T[i]</text>
<rect x="342" y="178" width="45" height="30" rx="4" class="bf-cell-hi"/>
<text x="364.5" y="198" class="bf-t" text-anchor="middle" style="font-size: 12px">4</text>
<text x="546" y="198" class="bf-tm">byte: binary search in C</text>
<text x="86" y="244" class="bf-ts" text-anchor="end" style="font-family: var(--md-code-font-family); font-size: 13px">bzip2, mibz2</text>
<text x="140" y="244" class="bf-tm" text-anchor="end">tt[s]</text>
<rect x="150" y="224" width="285" height="30" rx="4" class="bf-cell-hi"/>
<text x="292.5" y="244" class="bf-t" text-anchor="middle" style="font-size: 12px">i such that T[i] = s, 24 bits</text>
<rect x="438" y="224" width="93" height="30" rx="4" class="bf-slot"/>
<text x="484.5" y="244" class="bf-t" text-anchor="middle" style="font-size: 12px">L[s]</text>
<text x="546" y="244" class="bf-tm">the link reversed</text>
<text x="86" y="282" class="bf-tm">indexF's 20 bits sit in two arrays: 16 in one, and a 4-bit nibble packed two to a byte in another.</text>
</svg>
<figcaption>bw94 keeps T′ and L in separate arrays, so a step can miss twice. mergedTL packs both into one word. indexF keeps only T, in 20 bits, and finds the byte from the counts. bzip2 and mibz2 pack the byte with the reversed link.</figcaption>
</figure>

bzip2 uses mergedTL, with the reversed link stored as an absolute position, so its walk
needs no $C[c]$ addition. Its manual credits the change with a real gain: *"Single array
implementation of the inverse BWT. This significantly speeds up decompression, presumably
because it reduces the number of cache misses."*

## 3. Spending less memory: indexF and the count trees

*"Can we reduce memory use whilst retaining useful decompression speed? Interestingly, yes."*
The relation in $F$, $F[T[i]]$ precedes $F[i]$, needs only $T$ to walk. If the decoder reads the output bytes
from $F$ instead of $L$, it needs $L$ only while it builds $T$, so $T$ can be written over it. $F$ needn't
be stored either: it is sorted, so the 256 running totals in $C$ describe it, and $F[p]$ is the
byte whose bucket contains $p$. *"No free lunch, alas"*: finding it is a binary search in $C$.

**indexF**, named for indexing into $F$, keeps $T$ alone in $N$ 20-bit words (so $N \le 2^{20}$): a 16-bit
array plus a nibble array, $2.5N$ bytes. It starts the walk at $T[I]$, because reading $F$, the
byte $L[I]$ is found at $F[T[I]]$. bzip2 ships it as `-s`, *"a modified algorithm which only
requires 2.5 bytes per block byte … albeit at about half the normal speed"*, with the search
as `BZ2_indexIntoF`. What it gives up:

- **A search per byte,** eight or nine comparisons where mergedTL has a mask. The 1 KB of
  counts stays in cache, so the search costs instructions, not misses, and since the next
  position is $A[p]$ alone, a processor can overlap it with the next load.
- **Two arrays again.** The 20-bit entry is split between the 16-bit array and the nibbles,
  so a step can touch two cache lines, the cost mergedTL removed.
- **A cap of $2^{20}$ bytes per block,** which bzip2's 900,000-byte maximum fits.

**The trees.** *"It is instructive to ask just how little storage we can get away with"*: $L$
and the counts alone. Any $T[j]$ is $C[c]$ plus the number of $c$'s before $j$ in $L$, but counting
them means scanning $L$, $O(N)$ a step and $O(N^2)$ in all: *"absurdly inefficient"*. Seward buys
speed back with 256 binary trees of partial counts, one per byte value. Each node counts its
byte in an aligned power-of-two segment of $L$, an empty subtree costs nothing, and a node
budget sets how deep the trees grow. To count the $c$'s up to $j$, descend $c$'s tree, adding the
left child's count whenever $j$ lies to the right, and scan $L$ only for the last segment.

<figure class="bf-fig">
<p class="bf-title">A count tree answers a rank query</p>
<svg viewBox="0 0 720 330" role="img" aria-label="The count tree for byte a over L = n n b a a a, padded to 8 positions. The root covers positions 0 to 7 and counts 3 a's; its left child covers 0 to 3 with 1, its right child 4 to 7 with 2. Below them, 2 to 3 holds 1 and 4 to 5 holds 2; 0 to 1 and 6 to 7 hold none and are left empty. To count the a's in L positions 0 to 4: go right at the root, adding the left child's 1; go left to the node for 4 to 5; it has no children, so scan L position 4, which is an a. The count is 2.">
<path d="M387 62L267 100" class="bf-link"/>
<path d="M387 62L507 100" class="bf-link"/>
<path d="M267 132L207 166" class="bf-link"/>
<path d="M267 132L327 166" class="bf-link"/>
<path d="M507 132L447 166" class="bf-link"/>
<path d="M507 132L567 166" class="bf-link"/>
<rect x="341" y="30" width="92" height="32" rx="6" class="bf-cell-hi"/>
<text x="387" y="51" class="bf-t" text-anchor="middle" style="font-size: 13px">[0, 8) · 3</text>
<rect x="221" y="100" width="92" height="32" rx="6" class="bf-cell"/>
<text x="267" y="121" class="bf-t" text-anchor="middle" style="font-size: 13px">[0, 4) · 1</text>
<rect x="461" y="100" width="92" height="32" rx="6" class="bf-cell-hi"/>
<text x="507" y="121" class="bf-t" text-anchor="middle" style="font-size: 13px">[4, 8) · 2</text>
<rect x="281" y="166" width="92" height="32" rx="6" class="bf-cell"/>
<text x="327" y="187" class="bf-t" text-anchor="middle" style="font-size: 13px">[2, 4) · 1</text>
<rect x="401" y="166" width="92" height="32" rx="6" class="bf-cell-hi"/>
<text x="447" y="187" class="bf-t" text-anchor="middle" style="font-size: 13px">[4, 6) · 2</text>
<text x="207" y="187" class="bf-tm" text-anchor="middle">[0, 2) · none</text>
<text x="567" y="187" class="bf-tm" text-anchor="middle">[6, 8) · none</text>
<rect x="150" y="236" width="54" height="32" rx="4" class="bf-slot"/>
<text x="177.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="177.0" y="286" class="bf-tm" text-anchor="middle">0</text>
<rect x="210" y="236" width="54" height="32" rx="4" class="bf-slot"/>
<text x="237.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<text x="237.0" y="286" class="bf-tm" text-anchor="middle">1</text>
<rect x="270" y="236" width="54" height="32" rx="4" class="bf-slot"/>
<text x="297.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<text x="297.0" y="286" class="bf-tm" text-anchor="middle">2</text>
<rect x="330" y="236" width="54" height="32" rx="4" class="bf-slot"/>
<text x="357.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="357.0" y="286" class="bf-tm" text-anchor="middle">3</text>
<rect x="390" y="236" width="54" height="32" rx="4" class="bf-slot-hi"/>
<text x="417.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="417.0" y="286" class="bf-tm" text-anchor="middle">4</text>
<rect x="450" y="236" width="54" height="32" rx="4" class="bf-slot"/>
<text x="477.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<text x="477.0" y="286" class="bf-tm" text-anchor="middle">5</text>
<rect x="510" y="236" width="54" height="32" rx="4" class="bf-cell"/>
<text x="537.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px"></text>
<text x="537.0" y="286" class="bf-tm" text-anchor="middle">6</text>
<rect x="570" y="236" width="54" height="32" rx="4" class="bf-cell"/>
<text x="597.0" y="257" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px"></text>
<text x="597.0" y="286" class="bf-tm" text-anchor="middle">7</text>
<text x="136" y="257" class="bf-ts" text-anchor="end">L</text>
<text x="564" y="257" class="bf-tm" text-anchor="middle">padding</text>
<text x="30" y="318" class="bf-tm">How many a's in L[0..4]? Right at the root (+1 from [0, 4)), left to [4, 6), off the tree: scan L[4..4] (+1). Two.</text>
</svg>
<figcaption>The tree for byte a over L = nnbaaa, padded to 8 positions. Counting the a's up to position 4 takes one left-child count and a one-byte scan, so T[4] = C[a] + 2 − 1 = 1.</figcaption>
</figure>

**tree** stores a node in three 32-bit words. **treeopt** packs a node into 54 bits, two
words, and a leaf into one, fitting nearly twice as many nodes in the same space, which
halves the scans. With $L$, tree-1M and treeopt-1M take $2N$ bytes and treeopt-500k $1.5N$.

## 4. The space–time tradeoff

The machine was a 400 MHz AMD K6-III with 128 MB of memory and a 256 KB second-level cache.
Blocks were at most 10⁶ bytes, 1 MB, and each was inverted ten times. He tested six files of
at least a megabyte, four of them from the [Canterbury
Corpus](https://corpus.canterbury.ac.nz/descriptions/). The last column says what each shares
with mibz2's own [benchmark corpora](../benchmarks/index.md):

<div class="bf-data" markdown>

| File | MB | What it is | Closest in mibz2's corpora |
|---|--:|---|---|
| bible | 4.05 | The King James Bible: English prose | `text`, though ours is the crate's own source code, repeated, so more redundant |
| world192 | 2.47 | The CIA World Factbook, 1992: an English record per country | Wikidata's subject matter, though not its JSON format |
| ecoli | 4.64 | The *E. coli* genome: four letters, a, c, g and t, and notoriously hard to compress | No counterpart. Its tiny alphabet is the case the tree algorithms handle best |
| kennedy | 1.03 | An Excel spreadsheet, full of long runs of one byte | `runs`, and at the extreme `onebyte` |
| netscape | 7.68 | Netscape Navigator 4.75: x86 machine code | `records`, binary and structured |
| bashtgz | 1.71 | bash 2.04's source, gzipped: already compressed, so close to random | `random` |

</div>

Each algorithm's memory for a 1 MB block, and how fast it inverted each file, in MB/s. The
fastest on each file is in bold, and mergedTL is the fastest on every one:

<div class="bf-rates" markdown>

| | MB | bashtgz | bible | ecoli | kennedy | netscape | world192 | Mean |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| basis | 5.0 | 1.54 | 1.72 | 1.53 | 2.46 | 2.32 | 1.65 | 1.86 |
| bw94 | 5.0 | 1.58 | 1.77 | 1.57 | 2.54 | 2.41 | 1.69 | 1.93 |
| mergedTL | 4.0 | **2.49** | **2.76** | **2.48** | **4.72** | **3.51** | **2.65** | **3.10** |
| indexF | 2.5 | 1.20 | 1.36 | 1.30 | 1.71 | 1.67 | 1.31 | 1.43 |
| tree-1M | 2.0 | 0.032 | 0.211 | 0.325 | 0.099 | 0.044 | 0.207 | 0.153 |
| treeopt-1M | 2.0 | 0.059 | 0.301 | 0.342 | 0.167 | 0.099 | 0.298 | 0.211 |
| treeopt-500k | 1.5 | 0.032 | 0.218 | 0.339 | 0.100 | 0.036 | 0.198 | 0.154 |

</div>

The paper gives these numbers only as tables. Plotting each algorithm's average speed against
its memory shows the space–time tradeoff between them:

<figure class="bf-fig">
<p class="bf-title">Seward's measured speed–memory choices</p>
<svg viewBox="0 0 720 350" role="img" aria-label="Average inversion rate against memory for a one-megabyte block, from Seward's Tables 1 and 3. mergedTL: 4 MB, 3.10 MB/s. basis and bw94: 5 MB, 1.86 and 1.93. indexF: 2.5 MB, 1.43. The three tree variants: 1.5 to 2 MB, 0.15 to 0.21.">
<path d="M90 290.0H690" class="bf-grid"/>
<text x="80" y="294.0" class="bf-tm" text-anchor="end">0</text>
<path d="M90 215.7H690" class="bf-grid"/>
<text x="80" y="219.7" class="bf-tm" text-anchor="end">1</text>
<path d="M90 141.4H690" class="bf-grid"/>
<text x="80" y="145.4" class="bf-tm" text-anchor="end">2</text>
<path d="M90 67.1H690" class="bf-grid"/>
<text x="80" y="71.1" class="bf-tm" text-anchor="end">3</text>
<path d="M90.0 30V290" class="bf-grid"/>
<text x="90.0" y="310" class="bf-tm" text-anchor="middle">1</text>
<path d="M223.3 30V290" class="bf-grid"/>
<text x="223.3" y="310" class="bf-tm" text-anchor="middle">2</text>
<path d="M356.7 30V290" class="bf-grid"/>
<text x="356.7" y="310" class="bf-tm" text-anchor="middle">3</text>
<path d="M490.0 30V290" class="bf-grid"/>
<text x="490.0" y="310" class="bf-tm" text-anchor="middle">4</text>
<path d="M623.3 30V290" class="bf-grid"/>
<text x="623.3" y="310" class="bf-tm" text-anchor="middle">5</text>
<text x="390.0" y="334" class="bf-tm" text-anchor="middle">working memory, MB for a 10⁶-byte block (Table 1)</text>
<text x="22" y="160.0" class="bf-tm" text-anchor="middle" transform="rotate(-90 22 160.0)">average rate, MB/s (Table 3)</text>
<circle cx="623.3" cy="151.8" r="6.5" class="bf-dot-hi"/>
<circle cx="623.3" cy="146.6" r="6.5" class="bf-dot-hi"/>
<circle cx="490.0" cy="59.7" r="9" class="bf-dot-hi"/>
<circle cx="290.0" cy="183.8" r="6.5" class="bf-dot-hi"/>
<circle cx="223.3" cy="278.6" r="6.5" class="bf-dot-hi"/>
<circle cx="223.3" cy="274.3" r="6.5" class="bf-dot-hi"/>
<circle cx="156.7" cy="278.6" r="6.5" class="bf-dot-hi"/>
<text x="474.0" y="43.7" class="bf-t-strong" text-anchor="middle">mergedTL 3.10</text>
<text x="290.0" y="167.8" class="bf-t-strong" text-anchor="middle">indexF 1.43</text>
<text x="609.3" y="140.6" class="bf-t" text-anchor="end">bw94 1.93</text>
<text x="609.3" y="165.8" class="bf-t" text-anchor="end">basis 1.86</text>
<text x="190.0" y="254.3" class="bf-tm" text-anchor="middle">the three tree variants, 0.15–0.21</text>
</svg>
<figcaption>Each algorithm's average rate over the six files against its memory, from Seward's Tables 1 and 3, on a 400 MHz K6-III in 2001. The paper gives the numbers as tables; the plot is ours.</figcaption>
</figure>

It isn't a smooth curve on which each saved byte costs speed. mergedTL is *"on average 61% and
67% faster than bw94 and basis respectively, at the same time using less memory"*: it beats
both on both axes, by changing how the next link is read. indexF runs at 46% of its speed in
62.5% of its memory. Below that, half a byte a row less costs a factor of seven.

Seward then asks why. Simulating the K6-III's cache with cacheprof, his own profiler, he
counted what each does per byte of bible. bw94 runs 18 instructions, with 9 memory
references and 1.63 cache misses; mergedTL runs 20 instructions, with 8 references and 0.88
misses. So mergedTL does 11% more work and takes 54% as many misses. 89.4% of those misses are
the walk's reads of $U[p]$, and 78.6% of those reads miss, because $T$ holds *"a completely
arbitrary permutation"* that the decoder *"is obliged to traverse"*. Seward concludes that *"there appears to be
no way to avoid a miss rate approaching one per $L$-byte"*, and mergedTL is *"a realistic upper
bound on inversion performance"*.

kennedy inverts nearly twice as fast as bashtgz because its long
runs make $T$ point each row at its neighbour, which is usually already cached; random input
sets the floor near 2.5 MB/s. And the tree algorithms vary 10:1 across files, against mergedTL's
1.9:1, because a small alphabet such as ecoli's four letters lets a few deep trees shorten
every scan.

bzip2 itself decompresses bible at 67% of mergedTL's inversion rate, while also doing its
Huffman and move-to-front decoding, so *"no amount of re-engineering of bzip2 can increase its
decompression speed much further"*. The paper ends on a caveat:

> Such observations only hold true whilst cache sizes are much smaller than block sizes.
> Future growth in cache sizes may invalidate such findings.

## 5. Four blocks sharing one cache

The K6-III's cache held 256 KB, against mergedTL's 4 MB for one block. Our workstation has a
20 MiB last-level cache, and a full bzip2 block, up to 900,000 rows at 4 bytes each, needs a
3.6 MB table. Read literally, the caveat has come true.

<figure class="bf-fig">
<p class="bf-title">Caches against tables, then and now</p>
<svg viewBox="0 0 720 300" role="img" aria-label="Cache size against navigation table size, to one scale in megabytes. In 2001 the K6-III had a 256 KB cache and one block's mergedTL table took 4 MB, about sixteen times the cache. On our workstation the last-level cache is 20 MiB; one block's table is 3.6 MB, but four blocks' tables take 14.4 MB, 0.7 of the cache.">
<path d="M210 20V272" class="bf-grid"/>
<text x="210" y="290" class="bf-tm" text-anchor="middle">0 MB</text>
<path d="M330 20V272" class="bf-grid"/>
<text x="330" y="290" class="bf-tm" text-anchor="middle">5 MB</text>
<path d="M450 20V272" class="bf-grid"/>
<text x="450" y="290" class="bf-tm" text-anchor="middle">10 MB</text>
<path d="M570 20V272" class="bf-grid"/>
<text x="570" y="290" class="bf-tm" text-anchor="middle">15 MB</text>
<path d="M690 20V272" class="bf-grid"/>
<text x="690" y="290" class="bf-tm" text-anchor="middle">20 MB</text>
<text x="0" y="34" class="bf-tm" style="text-transform: uppercase; letter-spacing: 0.06em">2001</text>
<text x="200" y="62" class="bf-t" text-anchor="end">K6-III cache</text>
<rect x="210" y="47" width="6.1" height="22" rx="4" class="bf-box" style="stroke-width: 2"/>
<text x="224.1" y="63" class="bf-tm">256 KB</text>
<text x="200" y="96" class="bf-t" text-anchor="end">one block's table</text>
<rect x="210" y="81" width="96.0" height="22" rx="4" class="bf-slot"/>
<text x="314.0" y="97" class="bf-tm">4 MB: about 16 times the cache</text>
<text x="0" y="128" class="bf-tm" style="text-transform: uppercase; letter-spacing: 0.06em">2026</text>
<text x="200" y="156" class="bf-t" text-anchor="end">workstation cache</text>
<rect x="210" y="141" width="503.3" height="22" rx="4" class="bf-box" style="stroke-width: 2"/>
<text x="703.3" y="157" class="bf-tm" text-anchor="end">20 MiB</text>
<text x="200" y="190" class="bf-t" text-anchor="end">one block's table</text>
<rect x="210" y="175" width="86.4" height="22" rx="4" class="bf-slot"/>
<text x="304.4" y="191" class="bf-tm">3.6 MB</text>
<text x="200" y="224" class="bf-t" text-anchor="end">four blocks' tables</text>
<rect x="210" y="209" width="345.6" height="22" rx="4" class="bf-slot"/>
<text x="563.6" y="225" class="bf-tm">14.4 MB: 0.7 of the cache</text>
</svg>
<figcaption>One scale, in megabytes. In 2001 one block's table was far larger than the cache; today one fits several times over, but four take most of it. Table sizes are computed for full blocks, not measured cache occupancy.</figcaption>
</figure>

Two things kept Seward's sentence true anyway. A modern core keeps many misses in flight if
it is given independent loads, so mibz2's [walk](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L927-L961)
advances 32 separate pieces of the chain in turn and [runs 8.6 times as fast as one
chain](2026-10-31-room-walk-overlap.md).

And a decoder doesn't invert one block at a time.
Four workers keep four tables live, 14.4 MB before any of their other buffers, in one cache
they all share, and [decoding with four costs more CPU time per byte than decoding with
one](2026-11-01-inverse-bwt-memory-interference.md): the workers finish sooner together, but
compete for the memory system on the way. With the cache divided between blocks, caches are
again much smaller than the tables being walked.

On Seward's axes a smaller table is a step down in speed. With four blocks sharing a cache,
it also leaves more of the cache to the other three. Could the same links be stored in much
less space, and would the lighter memory load make up for the extra work of reading a
compressed entry? The first valid comparison answers both ways at once. The compact walk
costs 14.85% more CPU time on one core and 15.95% less with four workers walking together.
That result is about this traversal, measured on its own; what it means for the whole decoder
is still open.

## 6. A smaller table, a busier lookup

The [census of the navigation table](2026-10-29-compact-navigation-tables.md) found that
most of its links follow simple patterns. An earlier format built on that,
[FOR-14](https://github.com/lmmx/mibz2/pull/130), packed 14-bit offsets into fixed records.
It halved the table, but extracting the packed bits made each step so much longer that the
walk lost even with four workers, where it took 1.45 times as long as the ordinary walk.

Compact30 keeps each offset directly addressable instead. Its name means thirty rows per
record, not thirty-bit pointers. A 64-byte record, aligned to a cache line, holds thirty
16-bit offsets and a 4-byte header. In an ordinary record the header carries a base position
and the byte the thirty rows all output, and a row's next position is the base plus its
offset. A group of rows that start with different bytes, or whose next positions are too far
apart for 16-bit offsets, is stored instead as a RAW group: thirty full 4-byte entries,
padded to 128 bytes.

<figure class="bf-fig">
<p class="bf-title">One ordinary compact30 record</p>
<svg viewBox="0 0 720 138" role="img" aria-label="A 64-byte record: sixty bytes hold thirty 16-bit row offsets, and four bytes hold the header with the shared base and output byte. The next position is the base plus the row's offset.">
<rect x="4" y="24" width="666" height="50" rx="8" class="bf-cell"/><rect x="629" y="24" width="41" height="50" rx="4" class="bf-cell-hi"/>
<text x="312" y="45" class="bf-ts" text-anchor="middle">30 offsets, one per row</text><text x="312" y="62" class="bf-tm" text-anchor="middle">30 × 2 bytes = 60 bytes</text>
<text x="650" y="45" class="bf-ts" text-anchor="middle" style="font-size: 10px">H</text><text x="650" y="62" class="bf-tm" text-anchor="middle" style="font-size: 10px">4 B</text>
<text x="4" y="96" class="bf-t">Next position = the base in H + this row's offset</text>
<text x="4" y="120" class="bf-tm">H also holds the byte the record's rows output.</text>
</svg>
<figcaption>Thirty rows in one cache line. RAW groups, segment starts and parking are handled outside this ordinary case.</figcaption>
</figure>

On the measured corpus that brings the table from 4 bytes per row to 2.164, 45.9% less:
about 1.95 MB for a full block instead of 3.6 MB, and 7.79 MB instead of 14.4 MB for four
blocks at once.

Read against Seward's paper, the record combines three of its ideas. Its offsets rest on the
ordering that makes $T$ cheap to build: within one byte's bucket the links increase with the
row, so a group's links are a base plus small offsets, smallest where the byte's occurrences
in $L$ sit close together, as in kennedy's runs. In Wikidata's blocks they usually do: about
77% of a full block's rows fall in 32-row groups that each come from a single run in $L$. The
header's byte is indexF's idea, emitting from $F$ so that the walk needs no $L$, with one byte
stored per group in place of a search. And keeping the header and the offset in one cache
line is mergedTL's. At 2.164 bytes a row, the record sits between treeopt-1M and indexF on
Seward's memory axis, and aims at mergedTL's speed.

The step costs more, though. An ordinary compact30 step compiles to 21 instructions and four
data loads; the conventional step is 16 instructions and three loads. Two of compact30's
loads, the offset and the header, are both addressed from the current row, so neither waits
for the other, but they are still two loads where the conventional step needs one. The
[walker's journal](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md)
has the annotated disassembly.

So the question is which matters more when several cores share the machine: the extra
instructions, or the smaller table.

## 7. Timing the walk on one core and on four

The comparison times two versions of the same traversal and assembly. The control walks
mibz2's 4-byte table; compact30 walks its records with the same 32-lane walk. Every walk's
output was checked byte for byte against the decoder's.

Only the walk is timed. Before each walk, the control builds its table with the decoder's
own builder, while compact30 copies in records made beforehand by a reference builder, so
the comparison measures how fast each prepared table is walked, not what it costs to build.
The two preparations may also leave different data in cache.

The [protocol](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md)
used one corpus of 434 blocks (207 million rows) and 20 paired rounds, on one pinned core
or four. With four, each worker prepared its next block, all four started walking together,
and none checked its output until all had finished, with blocks dealt so that each worker
had about the same number of rows. CPU cost is the threads' own CPU time, summed, per row.
Completion time is the time from the first walk's start to the last walk's end in each step,
summed over the steps. Neither is the decoder's elapsed time.

| | Conventional | Compact30 | Compact30 ÷ conventional (95% interval) |
| --- | ---: | ---: | ---: |
| CPU time per row, one core | 1.221 ns | 1.402 ns | 1.148 (1.147–1.149) |
| CPU time per row, four workers | 2.055 ns | 1.727 ns | 0.840 (0.838–0.843) |
| Completion time per pass, four workers | 114.45 ms | 97.97 ms | 0.856 (0.852–0.860) |

The intervals describe variation across the twenty rounds, on one machine and one corpus.
Before the run, the protocol fixed what would justify building a direct constructor: the
upper end of the four-worker CPU ratio at most 0.865, and of the completion ratio at most
1.02. The upper ends were 0.843 and 0.860. The [analysis](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/c30-analysis.json)
and an [independent reconstruction from the individual walks](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/researcher-review/independent_review.md)
agree on every figure.

<figure class="bf-fig">
<p class="bf-title">CPU time per row, one core and four workers</p>
<svg viewBox="0 0 720 265" role="img" aria-label="CPU nanoseconds per row, lower is better. On one core the conventional walk costs 1.221 and compact30 1.402. With four workers the conventional walk costs 2.055 and compact30 1.727.">
<text x="154" y="20" class="bf-tm">CPU nanoseconds per row · lower is better</text>
<path d="M156 37V218M266 37V218M376 37V218M486 37V218M596 37V218" class="bf-grid"/>
<text x="156" y="234" class="bf-tm" text-anchor="middle">0</text><text x="266" y="234" class="bf-tm" text-anchor="middle">0.5</text><text x="376" y="234" class="bf-tm" text-anchor="middle">1.0</text><text x="486" y="234" class="bf-tm" text-anchor="middle">1.5</text><text x="596" y="234" class="bf-tm" text-anchor="middle">2.0</text>
<text x="8" y="77" class="bf-ts">One core</text><text x="8" y="163" class="bf-ts">Four workers</text>
<rect x="156" y="48" width="268.5" height="24" rx="4" class="bf-slot"/><rect x="156" y="79" width="308.4" height="24" rx="4" class="bf-cell-hi"/>
<rect x="156" y="134" width="452.1" height="24" rx="4" class="bf-slot"/><rect x="156" y="165" width="380.0" height="24" rx="4" class="bf-cell-hi"/>
<text x="431" y="65" class="bf-t-strong">1.221</text><text x="471" y="96" class="bf-t-strong">1.402</text><text x="615" y="151" class="bf-t-strong">2.055</text><text x="543" y="182" class="bf-t-strong">1.727</text>
<rect x="158" y="247" width="14" height="10" rx="2" class="bf-slot"/><text x="180" y="256" class="bf-tm">Conventional</text><rect x="312" y="247" width="14" height="10" rx="2" class="bf-cell-hi"/><text x="334" y="256" class="bf-tm">Compact30</text>
</svg>
<figcaption>Each walking thread's CPU time, summed and divided by rows. Building the tables is not included.</figcaption>
</figure>

The gain is spread across the workload. All 230 full-sized blocks were cheaper with compact30
at four workers, taking each block's average over the rounds, and compact30 used less CPU in
1,138 of the 1,140 full-sized four-worker steps. These are checks within one experiment, on
blocks that repeat each round, not independent trials. The 204 short blocks were slower with
compact30, but together they hold 8,324 rows, 0.004% of the total.

The run also had to show that the four walks started together, within 2% of each step's
length. An earlier complete run failed that check, because waking sleeping workers delayed
some walks by tens of microseconds, and it doesn't count. This run used a spinning start
barrier instead, with nothing else changed; its worst start skew was 0.023%.

## 8. Four workers cost the conventional walk far more

From one core to four workers, the conventional walk's CPU cost per row rises from 1.221 ns
to 2.055 ns, 68% more. Compact30's rises from 1.402 ns to 1.727 ns, 23% more. Both run under
the same four-worker conditions, and the conventional table pays much more for them.

FOR-14 showed the same pattern earlier: from one block to four, the conventional walk's cost
per row rose 56% and FOR-14's only 8%. Its smaller table removed most of the four-worker
penalty, but each of its steps started 1.40 ns per row behind, three times what it saved.
Compact30's step is cheap enough to come out ahead.

The obvious explanation is the table's size. Four conventional tables take about 14.4 MB,
four compact30 tables about 7.79 MB, and the test machine's four cores share a 20 MiB
last-level cache with everything else the workers use. Smaller tables could mean fewer
evictions and less contention for memory, and they could also change address-translation
costs.

<figure class="bf-fig">
<p class="bf-title">Navigation tables for four full blocks</p>
<svg viewBox="0 0 720 155" role="img" aria-label="Four conventional navigation tables total 14.4 MB; four compact30 tables total about 7.79 MB. These are table sizes, not measured cache occupancy or memory traffic.">
<text x="5" y="34" class="bf-ts">Conventional</text><rect x="150" y="16" width="474" height="28" rx="5" class="bf-slot"/><text x="634" y="35" class="bf-t-strong">14.4 MB</text>
<text x="5" y="89" class="bf-ts">Compact30</text><rect x="150" y="71" width="256.4" height="28" rx="5" class="bf-cell-hi"/><text x="417" y="90" class="bf-t-strong">7.79 MB</text>
<text x="5" y="129" class="bf-tm">One full-sized block per worker · table sizes, not cache occupancy</text>
</svg>
<figcaption>The tables' sizes on the measured corpus. How much of each stays in cache, and how much memory traffic each causes, was not measured.</figcaption>
</figure>

That explanation hasn't been tested. The run recorded no cache-miss or memory-controller
counters, and the two arms prepared their tables differently before each walk, which by
itself could change what is in cache when the walk starts. The size difference makes the
cache explanation credible; it doesn't measure it.

The result also stops short of the decoder. A decoding worker doesn't spend all its time
walking tables: it also reads input, reconstructs the last column, builds the table,
assembles segments and writes output. Some of its walks run alone, and others overlap two or
three neighbours rather than all four. So compact30's 14.85% loss on one core matters as
much as its gain at four, and how much of a whole decode runs under four-way contention
hasn't been measured.

## 9. The table still has to be built

The records in this comparison came from a reference builder that first computes every
row's successor in a full-sized temporary array, then converts the result into compact30
records. That kept construction out of the measurement, but a decoder can't afford it. A
direct builder has to produce the records from the block's last column and its byte counts,
without the temporary array.

It also has to compete with the [decoder's current builder](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L253-L318),
which places every row in one counting-sort pass and writes runs of sixteen equal bytes
sixteen entries at a time. Compact30's builder has to write group headers, check offsets
against the 16-bit limit, and move groups to RAW, and still stay close to that.

The traversal result sets the budget. A rough planning model puts the decoder's current
table building at 0.592 CPU-seconds per decoded GB, and the traversal with its assembly at
1.847. If b and w are compact30's building and traversal costs as fractions of today's, the
saving is about 0.592 × (1 − b) + 1.847 × (1 − w) CPU-seconds per GB, and the target is at
least 0.25.

If the four-worker traversal ratio carried over to the decoder, a builder costing the same
as today's would save about 0.295 CPU-seconds per GB. The builder could be up to 7.5% slower
than today's at the central estimate, or 6.7% using the conservative end of the interval.
That is a budget conditional on the traversal gain surviving inside the decoder, built on
approximate phase costs; meeting it would justify the next test, not settle the question.

The next step is a design for that builder, set out in the
[constructor brief](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md).
Its main difficulty is already clear. Groups that straddle two bytes' rows can be found from
the byte counts before any row is placed, but a group whose offsets turn out too wide is
only known when its last row lands, and has to move to RAW then. The builder has to produce
exactly the records tested here, and the first measurements will be whether it does and what
it costs. After that, a whole-decoder comparison can show how much of the four-worker gain
survives in practice.

For now, a smaller navigation table has beaten the conventional one in a controlled
four-worker traversal while losing on a single core. That is a strong reason to study the
memory contention behind it, and a concrete budget for building the table.

---

**Research records:** [Traversal protocol and the runs behind it](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md) · [This result's records](https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4) · [Independent reconstruction](https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/researcher-review) · [Compact30 walker](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/examples/c30walk.rs) · [FOR-14 trial](https://github.com/lmmx/mibz2/pull/130).
