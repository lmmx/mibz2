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

2 November 2026 · 22 min read · [Result](#7-the-four-worker-reversal)
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

The post has two halves. The first works through how Seward's algorithms undo the
transform. In [section 1](#1-from-sorted-rotations-to-the-transformation-vector) we decode a
toy example by hand, recovering the word `banana`, to show that the decoder rebuilds the text
by following a permutation of the block's positions, one lookup at a time, and that this
chain of lookups is where the cache misses come from. [Sections
2](#2-making-inversion-fast-basis-bw94-and-mergedtl) and
[3](#3-spending-less-memory-indexf-and-the-trees) cover his algorithms, which are different
ways of storing that permutation, and [section 4](#4-what-seward-measured) plots the speed and
memory he measured for each, which shows the space–time tradeoff between them.

The second half is ours. [Section 5](#5-the-same-tradeoff-with-four-blocks-live) explains why
four cores sharing one cache change the picture. [Section
6](#6-a-smaller-table-a-busier-lookup) describes a compact way of storing the permutation that
costs more per lookup, and [sections 7](#7-the-four-worker-reversal) and
[8](#8-four-workers-cost-the-conventional-walk-far-more) measure it: when one core walks it
alone, it is slower than mibz2's usual table, as Seward's tradeoff would predict, and when
four cores walk their tables at once, it is faster. [Section
9](#9-the-table-still-has-to-be-built) sets out what a decoder still needs before it could use
it: a way to build the table cheaply.

## 1. From sorted rotations to the transformation vector

Imagine bzip2 compressing the six bytes `banana`. The transform doesn't shrink them: it
reorders them into `nnbaaa` and records the number 3. bzip2's later stages compress that
output, and decoding, once it has undone those stages, has to get `banana` back from
`nnbaaa` and 3. Seward's notation, which he takes from Burrows and Wheeler's
1994 report *[A Block-sorting Lossless Data Compression
Algorithm](https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf)*, names every
piece:

| Symbol | Meaning | In the example |
|---|---|---|
| $N$ | the number of bytes in the block, *"typically in the range of one million"* | 6 |
| $S$ | the original block, which decoding recovers | `banana` |
| $M$ | an $N \times N$ matrix: every rotation of $S$, sorted | the six rows below |
| $L$ | the last column of $M$, which bzip2 transmits | `nnbaaa` |
| $F$ | the first column of $M$ | `aaabnn` |
| $I$ | the row of $M$ that holds $S$, also transmitted | 3 |
| $T$ | the *"transformation vector"*: $N$ integers, each in $0..N-1$ | 4 5 3 0 1 2 |
| $C$ | a count per byte value, or, as used below, where each byte's run starts in $F$ | counts a 3, b 1, n 2; starts a 0, b 3, n 4 |

The decoder is given $L$ and $I$, and must produce $S$: from `nnbaaa` and 3, it must get back to
`banana`.

<figure class="bf-fig">
<p class="bf-title">The six rotations of banana, sorted</p>
<svg viewBox="0 0 720 360" role="img" aria-label="The six rotations of banana, sorted into rows 0 to 5: abanan, anaban, ananab, banana, nabana, nanaba. The first column F reads a a a b n n; the last column L reads n n b a a a. Row 3 is banana itself, so I = 3. The T column reads 4, 5, 3, 0, 1, 2.">
<text x="60" y="36" class="bf-ts" text-anchor="middle">j</text>
<text x="139.0" y="36" class="bf-ts" text-anchor="middle">F</text>
<text x="270.0" y="36" class="bf-tm" text-anchor="middle">the sorted rotations, M</text>
<text x="459.0" y="36" class="bf-ts" text-anchor="middle">L</text>
<text x="588" y="36" class="bf-ts" text-anchor="middle">T[j]</text>
<text x="60" y="75" class="bf-t" text-anchor="middle">0</text>
<rect x="110" y="52" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="174" y="52" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="238" y="52" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="302" y="52" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="366" y="52" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="430" y="52" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="564" y="52" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="75" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">4</text>
<text x="60" y="115" class="bf-t" text-anchor="middle">1</text>
<rect x="110" y="92" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="174" y="92" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="238" y="92" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="302" y="92" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="366" y="92" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="430" y="92" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="564" y="92" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="115" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">5</text>
<text x="60" y="155" class="bf-t" text-anchor="middle">2</text>
<rect x="110" y="132" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="174" y="132" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="238" y="132" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="302" y="132" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="366" y="132" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="430" y="132" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="564" y="132" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="155" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">3</text>
<text x="60" y="195" class="bf-t" text-anchor="middle">3</text>
<rect x="110" y="172" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="174" y="172" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="238" y="172" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="302" y="172" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="366" y="172" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="430" y="172" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="564" y="172" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="195" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">0</text>
<text x="60" y="235" class="bf-t" text-anchor="middle">4</text>
<rect x="110" y="212" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="174" y="212" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="238" y="212" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="302" y="212" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="366" y="212" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="430" y="212" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="564" y="212" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="235" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">1</text>
<text x="60" y="275" class="bf-t" text-anchor="middle">5</text>
<rect x="110" y="252" width="58" height="34" rx="5" class="bf-cell-hi"/>
<text x="139.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="174" y="252" width="58" height="34" rx="5" class="bf-cell"/>
<text x="203.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="238" y="252" width="58" height="34" rx="5" class="bf-cell"/>
<text x="267.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">n</text>
<rect x="302" y="252" width="58" height="34" rx="5" class="bf-cell"/>
<text x="331.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="366" y="252" width="58" height="34" rx="5" class="bf-cell"/>
<text x="395.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">b</text>
<rect x="430" y="252" width="58" height="34" rx="5" class="bf-slot"/>
<text x="459.0" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">a</text>
<rect x="564" y="252" width="48" height="34" rx="5" class="bf-cell"/>
<text x="588" y="275" class="bf-t" text-anchor="middle" style="font-family: var(--md-code-font-family); font-size: 15px">2</text>
<rect x="34" y="167" width="586" height="44" rx="7" class="bf-box" style="stroke-width: 2"/>
<text x="634" y="195" class="bf-t-strong">I = 3</text>
<text x="34" y="314" class="bf-tm">Row 3 is banana itself. In every row, the L byte comes just before the F byte, going round the string.</text>
<text x="34" y="334" class="bf-tm">A bzip2 block carries L = nnbaaa and I = 3, and nothing else of M; T is rebuilt from them.</text>
</svg>
<figcaption>M for S = banana, with F in green, L in blue and T beside each row. Row 3, outlined, is S itself.</figcaption>
</figure>

A rotation moves some bytes from the front of a string to its back. Sort every rotation of
`banana` and the result is $M$. Row 3 is `banana` itself, so $I = 3$. A decoder never holds $M$:
the inverse transform recovers just enough of it, from $L$, to read off row $I$.

**Why the first and last columns.** Every row of $M$ is $S$ read round from some starting point,
so in every row the last byte comes just before the first, going round $S$. Row 3 ends in `a`
and starts with `b`, and the final `a` of `banana` comes, going round, before its opening
`b`. That is the relation the whole paper builds on: $L[i]$ comes just before $F[i]$ in $S$. The
two columns carry different information:

- **$F$** is the bytes of $S$ in sorted order, so the counts alone fix it. A decoder holding $L$
  can rebuild $F$ by counting, so it is never sent.
- **$L$** holds the byte that precedes each row's opening context, with the rows sorted by
  context. Bytes before similar contexts tend to repeat (in English, `he ` is often preceded
  by `t`), so $L$ has long runs, which bzip2's later stages compress.

So a bzip2 block carries $L$, with $I$ to say which row is $S$, and the decoder rebuilds the order
of $S$ from them with $T$.

**Defining $T$.** Seward's theory section is *"a more detailed exposition of Algorithm D in §2
of [1]"*, Burrows and Wheeler's decompression algorithm, because *"the increased detail
exposes a range of possible implementations"*. Everything rests on *"establishing the
relationship between the $F$ and $L$ vectors, and recording this in the 'transformation vector'
$T$"*:

\[
M[j, N-1]\; M[j, 0..N-2] = M[i, 0..N-1] \quad\Longrightarrow\quad T[j] = i.
\]

Take row $j$ and move its last byte to the front. That is another rotation of $S$, so it is some
row $i$ of $M$, and $T[j]$ records which. Row 0, `abanan`, becomes `nabana`, row 4, so $T[0] = 4$.
Moving the last byte to the front starts the string one byte earlier in $S$, so following $T$
walks $S$ backwards.

**Walking $T$.** Row $T[j]$ starts with the byte row $j$ ends with, which comes just before row $j$'s
first byte in $S$. So $F[T[i]]$ precedes $F[i]$. The decoder has $L$, not $F$, but $L[k] = F[T[k]]$, and
substituting gives a relation in $L$ alone:

\[
L[T[k]] \text{ immediately precedes } L[k] \text{ in } S.
\]

$L[I]$ is the last byte of $S$, so starting at $I$ and following $T$ gives $S$ from the end:

```text
rows visited:  3 → 0 → 4 → 1 → 5 → 2
bytes read:    a   n   a   n   a   b
```

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

**Building $T$ from counts.** The rows of $M$ are sorted, and two rows ending in the same byte
keep their order when that byte moves to the front, since the rest of each row decides it.
So the $k$-th $c$ in $L$ rotates into the $k$-th row starting with $c$: *"if $c$ occurs $k$ times in
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

The $c$'s in $F$ form one bucket, starting at $C[c]$, the number of bytes smaller than $c$, so

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

## 3. Spending less memory: indexF and the trees

*"Can we reduce memory use whilst retaining useful decompression speed? Interestingly, yes."*
The relation in $F$, $F[T[i]]$ precedes $F[i]$, needs only $T$ to walk. Read the output bytes from $F$
instead of $L$, and $L$ is needed only while $T$ is built, so $T$ can be written over it. $F$ needn't
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

## 4. What Seward measured

The machine was a 400 MHz AMD K6-III with 128 MB of memory and a 256 KB second-level cache.
Each block, at most 10⁶ bytes, was inverted ten times. How much memory each algorithm needs
for a 10⁶-byte block:

| Algorithm | Memory | Seward's description |
|---|--:|---|
| `basis` | 5000 kB | Straightforward derivation from theory |
| `bw94` | 5000 kB | As per [1], does 2 passes, not 3 |
| `mergedTL` | 4000 kB | As bw94 but more cache-friendly |
| `indexF` | 2500 kB | Indexes into $F$ rather than $L$ |
| `tree-1M` | 2000 kB | Computes $T$ incrementally; 1 megabyte extra space |
| `treeopt-1M` | 2000 kB | More space-economical version of tree-1M |
| `treeopt-500k` | 1500 kB | As treeopt-1M but with only 500k auxiliary space |

He tested them on six files of at least a megabyte, four from the [Canterbury
Corpus](https://corpus.canterbury.ac.nz/descriptions/). The last column names the nearest of
mibz2's [benchmark corpora](../benchmarks/index.md), loosely:

| File | kB | What it is | Nearest in mibz2's corpora |
|---|--:|---|---|
| bible | 4047.4 | The King James Bible | `text`, though ours is source code, repeated |
| world192 | 2473.4 | The CIA World Factbook, 1992 | none: Wikidata's subject matter, not its format |
| ecoli | 4638.7 | The *E. coli* genome, in four letters | none |
| kennedy | 1029.7 | An Excel spreadsheet | `runs`, and at the extreme `onebyte` |
| netscape | 7678.8 | Netscape Navigator 4.75, an x86 executable | `records`, binary |
| bashtgz | 1708.8 | bash 2.04's source, gzipped | `random` |

How fast each algorithm inverted each file, in MB/s. mergedTL is the fastest on every one:

| | basis | bw94 | mergedTL | indexF | tree-1M | treeopt-1M | treeopt-500k |
|---|--:|--:|--:|--:|--:|--:|--:|
| bashtgz | 1.54 | 1.58 | 2.49 | 1.20 | 0.032 | 0.059 | 0.032 |
| bible | 1.72 | 1.77 | 2.76 | 1.36 | 0.211 | 0.301 | 0.218 |
| ecoli | 1.53 | 1.57 | 2.48 | 1.30 | 0.325 | 0.342 | 0.339 |
| kennedy | 2.46 | 2.54 | 4.72 | 1.71 | 0.099 | 0.167 | 0.100 |
| netscape | 2.32 | 2.41 | 3.51 | 1.67 | 0.044 | 0.099 | 0.036 |
| world192 | 1.65 | 1.69 | 2.65 | 1.31 | 0.207 | 0.298 | 0.198 |
| Average | 1.86 | 1.93 | 3.10 | 1.43 | 0.153 | 0.211 | 0.154 |

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
counted per byte of bible:

| | instructions | memory references | misses |
|---|--:|--:|--:|
| bw94 | 18.00 | 9.00 | 1.63 |
| mergedTL | 20.00 | 8.00 | 0.88 |

mergedTL runs 11% more instructions and takes 54% as many misses. 89.4% of those misses are
the walk's reads of $U[p]$, and 78.6% of those reads miss, because $T$ holds *"a completely
arbitrary permutation"* that the decoder *"is obliged to traverse"*. So *"there appears to be
no way to avoid a miss rate approaching one per $L$-byte"*, and mergedTL is *"a realistic upper
bound on inversion performance"*.

kennedy inverts nearly twice as fast as bashtgz because its long
runs make $T$ point each row at its neighbour, which is usually already cached; random input
sets the floor near 2.5 MB/s. And the tree algorithms vary 10:1 across files, against mergedTL's
1.9:1, because a small alphabet such as ecoli's four letters lets a few deep trees shorten
every scan.

<figure class="bf-fig">
<p class="bf-title">Why the walk misses, and why runs help</p>
<svg viewBox="0 0 720 300" role="img" aria-label="Two walks through a 24-entry table. Above, the links form an arbitrary permutation, so each step jumps far across the table. Below, a run of equal bytes makes each entry point at its neighbour, so steps stay within a few entries.">
<text x="30" y="36" class="bf-ts">An arbitrary permutation: every step lands somewhere new</text>
<rect x="30" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="57" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="84" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="111" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="138" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="165" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="192" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="219" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="246" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="273" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="300" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="327" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="354" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="381" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="408" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="435" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="462" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="489" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="516" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="543" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="570" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="597" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="624" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="651" y="100" width="23" height="22" rx="3" class="bf-cell"/>
<path d="M122.5 98Q284.5 -14 446.5 98" class="bf-link"/>
<rect x="111" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M446.5 98Q392.5 46 338.5 98" class="bf-link"/>
<rect x="435" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M338.5 98Q406.0 38 473.5 98" class="bf-link"/>
<rect x="327" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M473.5 98Q298.0 -14 122.5 98" class="bf-link"/>
<rect x="462" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M122.5 98Q284.5 -14 446.5 98" class="bf-link"/>
<rect x="111" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M446.5 98Q392.5 46 338.5 98" class="bf-link"/>
<rect x="435" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M338.5 98Q406.0 38 473.5 98" class="bf-link"/>
<rect x="327" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M473.5 98Q298.0 -14 122.5 98" class="bf-link"/>
<rect x="462" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<rect x="111" y="100" width="23" height="22" rx="3" class="bf-cell-hi"/>
<text x="30" y="168" class="bf-ts">A run of equal bytes: each step lands next door</text>
<rect x="30" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="57" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="84" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="111" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="138" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="165" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="192" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="219" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="246" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="273" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="300" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="327" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="354" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="381" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="408" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="435" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="462" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="489" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="516" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="543" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="570" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="597" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="624" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<rect x="651" y="232" width="23" height="22" rx="3" class="bf-cell"/>
<path d="M257.5 230Q271.0 202 284.5 230" class="bf-link"/>
<rect x="246" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M284.5 230Q298.0 202 311.5 230" class="bf-link"/>
<rect x="273" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M311.5 230Q325.0 202 338.5 230" class="bf-link"/>
<rect x="300" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M338.5 230Q352.0 202 365.5 230" class="bf-link"/>
<rect x="327" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M365.5 230Q379.0 202 392.5 230" class="bf-link"/>
<rect x="354" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M392.5 230Q406.0 202 419.5 230" class="bf-link"/>
<rect x="381" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M419.5 230Q433.0 202 446.5 230" class="bf-link"/>
<rect x="408" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<path d="M446.5 230Q460.0 202 473.5 230" class="bf-link"/>
<rect x="435" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<rect x="462" y="232" width="23" height="22" rx="3" class="bf-cell-hi"/>
<text x="30" y="276" class="bf-tm">Each cell is one table entry; arcs are successive steps of the walk. With a block much larger than the cache, the upper walk misses on almost every step.</text>
</svg>
<figcaption>An illustration, not data. Through an arbitrary permutation each step lands somewhere new; through a run of equal bytes, as in kennedy, T points next door.</figcaption>
</figure>

bzip2 itself decompresses bible at 67% of mergedTL's inversion rate, while also doing its
Huffman and move-to-front decoding, so *"no amount of re-engineering of bzip2 can increase its
decompression speed much further"*. The paper ends on a caveat:

> Such observations only hold true whilst cache sizes are much smaller than block sizes.
> Future growth in cache sizes may invalidate such findings.

## 5. The same tradeoff, with four blocks live

The K6-III's cache held 256 KB, against mergedTL's 4 MB for one block. Our workstation has a
20 MiB last-level cache, and a full bzip2 block, up to 900,000 rows at 4 bytes each, needs a
3.6 MB table. Read literally, the caveat has come true.

Two things kept Seward's sentence true anyway. A modern core keeps many misses in flight if
it is given independent loads, so mibz2's [walk](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L927-L961)
advances 32 separate pieces of the chain in turn and [runs 8.6 times as fast as one
chain](2026-10-31-room-walk-overlap.md). And a decoder doesn't invert one block at a time.
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

## 7. The four-worker reversal

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
