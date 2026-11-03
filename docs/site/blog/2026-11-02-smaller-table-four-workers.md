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

2 November 2026 · 31 min read · [Research result](#the-first-positive-traversal-result)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[Julian Seward](https://en.wikipedia.org/wiki/Julian_Seward) wrote
[bzip2](https://en.wikipedia.org/wiki/Bzip2), first released in 1996, and was the original
designer and author of [Valgrind](https://valgrind.org/info/developers.html). In 2001, while
at Microsoft Research in Cambridge, he presented a ten-page paper at the IEEE Data
Compression Conference: *[Space-time tradeoffs in the Inverse B-W
Transform](https://doi.org/10.1109/DCC.2001.917175)*. bzip2 compresses with the
[Burrows–Wheeler transform](https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform)
(BWT), and decompressing means undoing it. Work on the BWT had concentrated on compression,
and "decompression-time issues remain underinvestigated". Seward had two uses in mind that
pull in opposite directions: booting a Linux kernel from a compressed image needs little
memory, and an on-the-fly disk compressor needs speed. So the paper offers a family of
algorithms that trade one for the other. At one extreme, it shows the BWT can be inverted,
slowly, in one byte of memory per byte of block. At the other:

> we argue that the maximum speed of BWT decompression is unavoidably constrained by the rate
> at which cache misses are serviced.

mibz2 is our bzip2 decoder, written to decompress the Wikidata dumps, and most of this blog
has been about that rate:

- [32 lanes](2026-10-31-room-walk-overlap.md) keep many misses in flight at once;
- [memory-level parallelism](2026-11-02-memory-level-parallelism.md) is what limits how many
  can overlap;
- [memory interference](2026-11-01-inverse-bwt-memory-interference.md) is what happens when
  four workers share the memory system;
- [compact navigation tables](2026-10-29-compact-navigation-tables.md) take fewer misses by
  making the table smaller.

Those are the parts of one rate: how many misses a decoded byte costs, how many can be
serviced at once, and what else is asking. We came to them one experiment at a time, and
Seward's sentence names what they share. He left the same thought in [bzip2's
manual](https://sourceware.org/bzip2/manual/manual.html): "performance, both for compressing
and decompressing, is largely determined by the speed at which your machine can service cache
misses. … I imagine bzip2 will perform best on machines with very large caches."

This post is about a compact table that revisits Seward's tradeoff, and the first result it
has given. The table only makes sense against his paper, so the first half of the post works
through the paper, with a six-byte example done by hand, his tables, and his results plotted
as the tradeoff they describe.

## From sorted rotations to the transformation vector

Seward takes his notation from Burrows and Wheeler's 1994 report, *[A Block-sorting Lossless
Data Compression Algorithm](https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf)*,
and assumes the reader knows it. Here it is, with `banana` as the block:

| Symbol | Seward's definition | For `banana` |
|---|---|---|
| N | the number of bytes in the block, "typically in the range of one million" | 6 |
| S | the block itself, S[0..N−1] | `banana` |
| M | an N × N matrix of bytes: every rotation of S, sorted | the six rows below |
| L | the last column of M: L[i] = M[i, N−1] | `nnbaaa` |
| F | the first column of M: F[i] = M[i, 0] | `aaabnn` |
| I | the row of M that holds S: S = M[I, 0..N−1] | 3 |
| T | a vector of N integers, each in 0..N−1, the "transformation vector" | 4 5 3 0 1 2 |
| C | 256 counts of the byte values, or their running totals | a 3, b 1, n 2 |

The decoder is given N, L and I, and must produce S.

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
<figcaption>M for S = banana, with F highlighted in green and L in blue, and T beside each row. Row 3, outlined, is S itself.</figcaption>
</figure>

A rotation moves some bytes from the front of a string to its back. Write out every rotation
of `banana` and sort the six strings, and the result is M. Row 3 is `banana` itself, so
I = 3. A decoder never holds M. The inverse transform recovers just enough of it, from L, to
read off row I.

**Why the first and last columns.** Every row of M is S read round from some starting point,
so in every row the last byte is the one that comes just before the first, going round S. Row
3, `banana`, ends in `a` and starts with `b`, and going round, the final `a` comes before the
opening `b`. That is the one relation between the columns the whole paper builds on: L[i]
comes just before F[i] in S. The two columns carry different information:

- **F** is the bytes of S in sorted order, `aaabnn`, so the counts alone fix it: three `a`,
  one `b`, two `n`. A decoder holding L can rebuild F by counting, so it is never sent.
- **L** holds, for each row, the byte that precedes that row's opening context, with the rows
  sorted by context. Bytes that precede similar contexts tend to repeat (in English, `he ` is
  often preceded by `t`), so L has long runs of equal bytes, which bzip2's later stages
  compress.

So L is what a bzip2 block carries, with I to say which row is S. What the decoder needs is a
way from "each L byte comes before its F byte" to the order of S, and that is the vector T.

**Defining T.** Seward's theory section is, in his words, "a more detailed exposition of
Algorithm D in §2 of [1]", the decompression algorithm of Burrows and Wheeler's report. "The
increased detail exposes a range of possible implementations of the inverse transformation",
and every algorithm in the rest of the paper is one of them. "Undoing the BWT hinges on
establishing the relationship between the F and L vectors, and recording this in the
'transformation vector' T." T is defined by

\[
M[j, N-1]\; M[j, 0..N-2] = M[i, 0..N-1] \quad\Longrightarrow\quad T[j] = i.
\]

Read the left-hand side as: take row j and move its last byte to the front. A rotation of a
rotation of S is another rotation of S, so the result is some row i of M, and T[j] records
which. Row 0, `abanan`, becomes `nabana`, which is row 4, so T[0] = 4. Row 3, `banana`,
becomes `abanan`, row 0, so T[3] = 0. Moving the last byte to the front starts the string one
byte earlier in S, so T[j] is the row whose string starts one byte before row j's, and
following T walks S backwards.

**Walking T.** Row T[j] starts with the byte that row j ends with, and that byte comes just
before row j's first byte in S:

\[
F[T[i]] \text{ immediately precedes } F[i] \text{ in } S.
\]

The decoder has L, not F. But row k's last byte is row T[k]'s first, L[k] = F[T[k]], and
putting T[k] in place of i turns both sides into L:

\[
L[T[k]] \text{ immediately precedes } L[k] \text{ in } S.
\]

Every term is now something the decoder holds. Row I is S itself, so L[I] is the last byte of
S, and starting there and following T gives S from the end. For `banana`, from p = I = 3:

| step | p | emit L[p] | next p = T[p] |
|--:|--:|:-:|--:|
| 0 | 3 | `a` | 0 |
| 1 | 0 | `n` | 4 |
| 2 | 4 | `a` | 1 |
| 3 | 1 | `n` | 5 |
| 4 | 5 | `a` | 2 |
| 5 | 2 | `b` | |

The walk emits `ananab`: `banana`, backwards. This walk is the whole of the inverse
transform's inner loop: load the entry at p, emit a byte, jump to the position it names.
Where each jump lands depends on the content of S, not on where the last one landed. In a
block of a million bytes that is a million loads from places the cache has no particular
reason to hold, and the paper's measurements are about what that costs.

**Backwards, and bzip2's "design flaw".** "Generating S backwards increases the
decompressor's storage requirements since the whole of S has to be created before any of it
can be output. The obvious solution is to reverse S in the compressor, before sorting." The
paper assumes that from here on, so each algorithm's walk writes S forwards. bzip2 doesn't
reverse its blocks, and a footnote says so: "As is the case with the bzip2 compressor – a
design flaw." bzip2's manual explains how that happened: "the bzip2 format was frozen before I
properly and fully understood the performance consequences of doing so." Its decoder
compensates by building the links the other way round:

```c
/*-- compute the T^(-1) vector --*/
for (i = 0; i < nblock; i++) {
   uc = (UChar)(s->tt[i] & 0xff);
   s->tt[s->cftab[uc]] |= (i << 8);
   s->cftab[uc]++;
}
```

Where Seward would set T[i] = s, this writes i into slot s, so slot s records where it came
from. That is T⁻¹, and following it emits S forwards: for `banana`, T⁻¹ = 3 4 5 2 0 1, and
from T⁻¹[I] = 2 the walk reads L[2], L[5], L[1], L[4], L[0], L[3], which spells `banana`.
mibz2's builder does the same, writing `ll[s] | i << 8` into slot s.

**Building T from counts.** A decoder can't compute T from M, which it doesn't have. Seward
shows T comes from L with one counting pass and one scan, from two observations. The rows of M
are sorted. And two rows that end in the same byte keep their order when that byte moves to
the front: they then start with the same byte, so the rest of each original row decides their
order, and those rows were already sorted. So if L[j] is the k-th occurrence of byte c in L,
the row it rotates into is the k-th row starting with c: "if c occurs k times in L[0..j] and
L[j] = c, then T[j] must be i, where i is the position of the k-th c in F."

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
<figcaption>The k-th a in L goes to the k-th a in F, and likewise for n and b, which gives T = 4 5 3 0 1 2 from counts and one scan.</figcaption>
</figure>

F is sorted, so the c's in F are one contiguous bucket, starting at C[c], the number of bytes
smaller than c. For `banana`, C[a] = 0, C[b] = 3 and C[n] = 4. Then

\[
T[j] = C[c] + k - 1, \qquad c = L[j].
\]

Scan L once with a running count per byte value, and each byte's next free slot in its bucket
is its T entry. Within one byte value the links increase with j: the k-th c in L goes to the
k-th slot of c's bucket. Every BWT has this ordering, and our compact table depends on it.

## Making inversion fast: basis, bw94 and mergedTL

"Newcomers to BWT-based compression are often delighted at the improvement in compression
ratio over LZ77/78 schemes, but dismayed at the speed, especially the speed of decompression.
How fast can inversion be?" Seward answers with three algorithms, each refining the last.

**basis** is the theory written out. Count each byte value in L and turn the counts into
running totals, so C[c] is where c's bucket starts in F. Then set T[i] = C[L[i]] for each i,
incrementing C[L[i]] as it goes. Then start at p = I and repeat S[i] = L[p], p = T[p]. It reads
L three times and keeps L (one byte a row) and T (four) alive: 5N bytes.

**bw94** is named for Burrows and Wheeler's 1994 report, which merges the first two passes.
While counting, record how many of each byte have been seen so far: T′[i], the number of
earlier occurrences of L[i]. That is T without its bucket's start, T[i] = T′[i] + C[L[i]], or
in Seward's words "displacements relative to the start of each character run in F". The walk
adds the start back, p = T′[p] + C[L[p]]. Two passes, still 5N bytes.

**mergedTL** comes from looking at bw94's walk: "Each iteration indexes both L and T′ at some
arbitrary point, but both at the same index p. Unless both T and L fit in the machine's cache,
this could cause two (capacity) misses per iteration." The fix puts both in one 32-bit word,
limiting N to 2²⁴:

\[
U[i] = 256 \cdot T'[i] + L[i].
\]

The name is literal: T (as T′) and L, merged into one array. Each step loads u = U[p], takes
c = u mod 256 as the byte, and moves to p = u div 256 + C[c]. The mod and div are a mask and a
shift. Unpacking costs some instructions, "but the near-halving of the cache miss rate is
potentially valuable", and the table is a byte a row smaller: 4N.

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

mergedTL is "employed in bzip2", in a form suited to its unreversed blocks: bzip2's word at
slot s holds L[s] in the low 8 bits and the reversed link above, as an absolute position, so
its walk needs no C[c] addition. The manual lists the change among bzip2 0.9.0's improvements:
"Single array implementation of the inverse BWT. This significantly speeds up decompression,
presumably because it reduces the number of cache misses."

## Spending less memory: indexF and the trees

"Can we reduce memory use whilst retaining useful decompression speed? Interestingly, yes."
The way in is the first form of the walking relation, which is about F: F[T[i]] precedes F[i].
The algorithms so far use its L form only because L is what the decoder holds, and that
obliges them to keep L alive through the whole walk. Read the bytes from F instead, and L is
needed only while T is built, so T can be written over it. F needn't be stored either. It is
sorted, so the 256 running totals in C describe it, and F[p] is the byte whose bucket contains
p. "No free lunch, alas: subscripting F then becomes a binary search in the cumulative
frequency table, rather than a cheap array indexing operation."

**indexF** keeps one array, A, of N 20-bit words, so N ≤ 2²⁰. A starts out holding L. After
counting, each A[i] is overwritten with T[i]. The walk starts at p = A[I] and repeats
S[i] = indexF(p), p = A[p], where indexF(p) is the binary search: "Since C has 256 entries this
search always completes in nine steps." The name is the operation, and Seward's table of
algorithms describes it in five words: "Indexes into F rather than L". The walk starts at
T[I] rather than I, because reading F, the byte L[I] is found at F[T[I]]. For an unreversed
block Seward adds a pass that reverses the pointers in A, turning T into T⁻¹ in place. The
20-bit words are a 16-bit array plus a byte array holding the top 4 bits two to a byte: 2.5N
bytes.

bzip2 ships this as its `-s` option, documented as "a modified algorithm which only requires
2.5 bytes per block byte … albeit at about half the normal speed". Its source has the 16-bit
array `ll16`, the nibble array `ll4`, the pointer reversal ("Compute T^(-1) by pointer
reversal on T"), and the search:

```c
Int32 BZ2_indexIntoF ( Int32 indx, Int32 *cftab )
{
   Int32 nb, na, mid;
   nb = 0;
   na = 256;
   do {
      mid = (nb + na) >> 1;
      if (indx >= cftab[mid]) nb = mid; else na = mid;
   }
   while (na - nb != 1);
   return nb;
}
```

Seward presents the cost as one trade, a search for memory. It has several parts:

- **A block-size cap** of 2²⁰ bytes. bzip2's largest block, 900,000 bytes, fits.
- **A search per output byte:** eight or nine dependent comparisons where mergedTL has a mask.
  The search runs over 1 KB of counts, which stays in cache, so it costs instructions, not
  misses.
- **The search is off the chain.** mergedTL's next position depends on the byte it just
  unpacked; indexF's is A[p], the link alone. A processor that runs ahead can overlap the
  search for one byte with the load for the next.
- **Two arrays again.** The 20-bit entry is split between the 16-bit array and the nibbles,
  so a step can touch two cache lines, the cost mergedTL was built to remove. The arrays are
  smaller, though, at 2N and N/2 bytes against 4N.
- **An extra pass** over A, for a block that wasn't reversed.

**The trees.** "It is instructive to ask just how little storage we can get away with, and the
answer is surprising: only L and the 'compressed' frequency-count version of F are actually
needed." Any entry of T can be computed when it's needed, as C[c] plus the number of c's in L
before position j. Counting means scanning L, so each step costs O(N) and the walk O(N²):
"absurdly inefficient", but it is the minimal-storage baseline, N bytes of L plus the counts.

Seward buys speed back with memory by storing partial counts. Tables of counts for halves or
quarters of L would shorten the scan, but waste space, because the shorter the segment, the
more of its 256 counts are zero. Instead he grows 256 binary trees, one per byte value:

- **A node** holds a count and two child pointers, and covers a segment of L. An empty subtree
  means the byte doesn't occur there, and costs nothing to store.
- **Segments are aligned powers of two.** With Q the least power of two at least N, each root
  covers L[0..Q−1], its children the two halves, and so on.
- **A node budget** sets how deep the trees grow, and so the trade between memory and scan
  length. The trees can stop at any depth and still answer correctly.

Each deepening pass scans L once, counting every byte value in the two halves of each
segment, so all 256 trees grow together. To count the c's in L[0..j], descend c's tree with a
running count: go left when j is in the left half; otherwise add the left child's count and go
right; on falling off the bottom, scan L from the start of the last segment to j.

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
<figcaption>The tree for byte a over L = nnbaaa, padded to 8 positions. Counting the a's up to position 4 takes one left-child count and a one-byte scan. With the 2 a's up to it, the a at L[4] goes to T[4] = C[a] + 2 − 1 = 1.</figcaption>
</figure>

**tree** stores a node as three 32-bit words, so 1 MB holds 83,333 nodes. **treeopt** fits
nearly twice as many in the same space. With N ≤ 2²⁰ and about 10⁶/12 nodes, a node needs
17 + 20 + 17 = 54 bits (two child indices and a count), packed into two words, and a leaf,
having no children, gets one. "Scanning is much more expensive than tree-building", so the
doubled node count, which halves the average scan, is worth the complication. The versions
tested were tree-1M and treeopt-1M, each with 1 MB of nodes (2N in all, with L), and
treeopt-500k (1.5N).

## What Seward measured

The experiments ran on a 400 MHz AMD K6-III with 128 MB of memory and a 256 KB second-level
cache, under Linux 2.0.36. Blocks were capped at 10⁶ bytes. Each block was read from disk and
then inverted ten times, into an array that was never written out.

Seward's first table gives each algorithm's memory for a 10⁶-byte block, not counting the at
most 2 kB of counts:

| Algorithm | Memory | Seward's description |
|---|--:|---|
| `basis` | 5000 kB | Straightforward derivation from theory |
| `bw94` | 5000 kB | As per [1], does 2 passes, not 3 |
| `mergedTL` | 4000 kB | As bw94 but more cache-friendly |
| `indexF` | 2500 kB | Indexes into F rather than L |
| `tree-1M` | 2000 kB | Computes T incrementally; 1 megabyte extra space |
| `treeopt-1M` | 2000 kB | More space-economical version of tree-1M |
| `treeopt-500k` | 1500 kB | As treeopt-1M but with only 500k auxiliary space |

His second table lists six test files, all at least a megabyte, since smaller files "don't
stress algorithms or memory systems realistically". Four come from the [Canterbury
Corpus](https://corpus.canterbury.ac.nz/descriptions/). mibz2's
[benchmark](../benchmarks/index.md) has its own corpora, and the last column names the
nearest, loosely:

| File | kB | What it is | Nearest in mibz2's corpora |
|---|--:|---|---|
| bible | 4047.4 | The King James Bible | `text`, though ours is the crate's own sources repeated, and more redundant |
| world192 | 2473.4 | The CIA World Factbook, 1992 | none. Country records in English: Wikidata's subject matter, not its format |
| ecoli | 4638.7 | The *E. coli* genome, in four letters | none |
| kennedy | 1029.7 | An Excel spreadsheet | `runs`, and at the extreme `onebyte` and `pattern` |
| netscape | 7678.8 | Netscape Navigator 4.75, an x86 executable | `records`, binary |
| bashtgz | 1708.8 | bash 2.04's source, gzipped | `random`: gzip's output is close to random bytes |

His third table gives the inversion rates in MB/s. mergedTL is the fastest on every file:

| | basis | bw94 | mergedTL | indexF | tree-1M | treeopt-1M | treeopt-500k |
|---|--:|--:|--:|--:|--:|--:|--:|
| bashtgz | 1.54 | 1.58 | 2.49 | 1.20 | 0.032 | 0.059 | 0.032 |
| bible | 1.72 | 1.77 | 2.76 | 1.36 | 0.211 | 0.301 | 0.218 |
| ecoli | 1.53 | 1.57 | 2.48 | 1.30 | 0.325 | 0.342 | 0.339 |
| kennedy | 2.46 | 2.54 | 4.72 | 1.71 | 0.099 | 0.167 | 0.100 |
| netscape | 2.32 | 2.41 | 3.51 | 1.67 | 0.044 | 0.099 | 0.036 |
| world192 | 1.65 | 1.69 | 2.65 | 1.31 | 0.207 | 0.298 | 0.198 |
| Average | 1.86 | 1.93 | 3.10 | 1.43 | 0.153 | 0.211 | 0.154 |

The paper has no figure. Plotting the averages against the memory draws the tradeoff of its
title:

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

"mergedTL is the clear speed winner, on average 61% and 67% faster than bw94 and basis
respectively, at the same time using less memory." So the plot isn't a smooth curve on which
each saved byte costs speed: mergedTL beats the two 5N designs on both axes, by changing how
the next link is read. indexF runs at 46% of mergedTL's speed in 62.5% of its memory, and
"still inverts well over a megabyte per second". Below it, half a byte a row less costs a
factor of seven in rate, and Seward expects the trees "to remain no more than proof-of-concept
implementations".

Seward closes the results with three questions the rates raise, and his answers carry the
paper's argument about cache misses.

**Why is mergedTL so much faster than bw94, when it does more work to unpack?** He inverted
bible under cacheprof, his own cache profiler, simulating the K6-III's 256 KB cache, and
counted per byte:

| | instructions | memory references | misses |
|---|--:|--:|--:|
| bw94 | 18.00 | 9.00 | 1.63 |
| mergedTL | 20.00 | 8.00 | 0.88 |

mergedTL executes 11% more instructions, takes 54% as many misses, and runs 56% faster on
bible. 89.4% of its misses come from the walk's reads of U[p], one read per byte, and 78.6% of
those reads miss. T "effectively contains the inverse permutation to that created by the BWT's
sorting operation", and nothing ties the sorted order of rotations to their order in S, so
"there appears to be no way to avoid a miss rate approaching one per L-byte" once the block is
much larger than the cache. Hence "it is plausible to argue that algorithm mergedTL represents
a realistic upper bound on inversion performance": the introduction's sentence, with numbers
under it.

**If misses set the rate, why does kennedy invert nearly twice as fast as bashtgz?** Runs.
kennedy has long runs of one byte, and the rotations within a run sort in order, so T maps
each of them to its neighbour. A walk through a run moves to the next entry, which is usually
already in the cache. Random input has nothing like this and sets the floor: bashtgz, ecoli
and world192 invert at 2.49, 2.48 and 2.65 MB/s.

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
<figcaption>An illustration, not data. Following T through an arbitrary permutation lands somewhere new each step. Through a run of equal bytes, as in kennedy, T points next door.</figcaption>
</figure>

**Why do the tree algorithms vary 10:1 across the files, against 1.9:1 for mergedTL?**
Alphabet size. ecoli has four distinct bytes, so the node budget goes to four deep trees and
the scans are short. bashtgz uses all 256 byte values, so each tree gets a 64th of the budget
and the scans are long.

Then the bound is put to use: bzip2 decompresses bible at 1.86 MB/s, 67% of mergedTL's
inversion rate, while also doing its Huffman and move-to-front decoding, which "suggest that
no amount of re-engineering of bzip2 can increase its decompression speed much further". The
paper's last paragraph is two sentences:

> Such observations only hold true whilst cache sizes are much smaller than block sizes.
> Future growth in cache sizes may invalidate such findings.

## The same tradeoff, with four blocks live

The K6-III's cache held 256 KB, against mergedTL's 4 MB for one block. The workstation behind
our measurements has a 20 MiB last-level cache, and a 900,000-byte bzip2 block needs a 3.6 MB
table. Read literally, the caveat has come true: one block's table fits several times over.

Two things kept the sentence from the introduction relevant anyway. A modern core keeps many
misses in flight at once, if it is given independent loads to issue, so the rate at which
misses are serviced depends on how many the program exposes. One chain through T exposes
none, since each address comes from the load before; mibz2's walk follows 32 chains at once
and runs [8.6 times as fast as one](2026-10-31-room-walk-overlap.md). And a decoder for a
multi-gigabyte file doesn't invert one block at a time. Four workers keep four tables live,
14.4 MB in one cache all four cores share, and on the workstation throughput falls off at
[about that much live table](../benchmarks/frontier.md), 0.7 of its L3. With the cache
divided between blocks, Seward's condition, caches much smaller than the tables being walked,
holds again. His axes are one block's rate and one block's memory. With four blocks sharing a
cache, a smaller table also leaves more of the cache to the other three.

In a [29 October journal
entry](https://github.com/lmmx/mibz2/blob/f384fb424b19e5e5e1c5e27559dd0eb916e98123/docs/journal/2026-10-29-frontier.md),
we asked whether a navigation structure that was slower for one isolated block might be
faster when several blocks were being decoded concurrently. Four workers have four
navigation tables, all using the same shared cache and the hardware behind it. A smaller
representation might relieve that competition enough to pay for the additional instructions
needed to read it.

The hope was not simply to choose another point along a space-for-speed compromise. It was
to improve the [machine-level throughput–memory trade-off](../benchmarks/frontier.md): to
see whether less navigation storage and greater completed throughput could coexist under the
same workload. The difference is important. A faster traversal on a prebuilt table is
evidence for this possibility; a faster *complete decoder* with lower measured memory use
would be the stronger result.

Our earlier [investigation of memory
interference](2026-11-01-inverse-bwt-memory-interference.md) made that question concrete.
Four-worker decoding finished sooner than one-worker decoding but used substantially more
total CPU time per decoded byte, despite retiring nearly the same instructions. We had a
reason to suspect a shared-memory penalty. We did not yet know whether a compact
representation could reduce it.

## The problem was never just storing fewer bytes

The conventional mibz2 navigation table, or *T-vector*, is bzip2's form of mergedTL: it
stores a next-row position and an output byte in a 32-bit entry. [The production
traversal](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L927-L955)
advances through those entries by dependent loads. A maximum-sized block has 900,000 rows,
so its table occupies 3.6 MB. Four such tables account for 14.4 MB before output buffers or
other live data. The workstation used in the experiment has a 20 MiB last-level cache shared
by its cores. That makes the table footprint worth investigating—but cache capacity alone
cannot establish where individual loads are served.

In [*Compact navigation tables for the inverse BWT: early
findings*](2026-10-29-compact-navigation-tables.md), we started with a simpler question: why
four bytes for every row? The next-row mapping is not arbitrary. Within each group of rows
beginning with the same character, successor positions are ordered, because construction
preserves the order of equal characters. If thirty successive rows point to positions such
as 1,000, 1,002 and 1,005, storing one base and the small differences may cost much less
than storing every absolute position.

The first corpus census found substantial regularity. That made compression plausible, but
we still needed a structure with random access to an individual successor. The decoder
cannot decompress a whole group of pointers every time it wants to follow one: the next
address is unknown until the current lookup completes. Our [32-lane
walk](2026-10-31-room-walk-overlap.md) already overlaps work from independent segments to
hide some of that wait. A compressed format must preserve that ability, not introduce
another dependent search before each step.

The first attempt tested that constraint directly. FOR-14 packed thirty-two 14-bit offsets
into each 64-byte record, with separate full-width entries for exceptions. Its [frozen
experiment](https://github.com/lmmx/mibz2/blob/7d7962f/docs/journal/2026-11-01-for14-stage1.md)
found just 2.076 logical bytes per row, a little over half the conventional table's size.
That was good news for storage; it wasn't good news for traversal.

Each lookup now needed to find a fourteen-bit field within a packed record, reconstruct the
next position and maintain a carried output byte. The compiled ordinary path contained 38
instructions, five data loads and three stores, against the conventional walk's 16
instructions, three loads and two stores. These are static hot-path counts, not the number
of instructions retired per decoded byte. But they show why compactness cannot be judged on
bytes alone: the decoder pays the cost of retrieving a *single* successor at every step.

And the first measurement went the wrong way. With one block, FOR-14's traversal took 2.676
ns per row against 1.275 ns for its control. With four blocks traversed concurrently, it
took 2.899 ns against 1.991 ns. The smaller representation's cost rose by only about 8% from
one block to four, while the conventional control's rose by 56%. This was the behaviour we
had hoped a smaller table might show under contention—but FOR-14 started 1.40 ns per row
behind. It still finished the four-block work about 1.45 times slower than its control. The
experimental format was stopped, even though the broader hypothesis survived.

Rather than rush into another timing campaign, the subsequent [packed-token
variants](https://github.com/lmmx/mibz2/pull/130) attacked the decoding overhead. Two
alternatives compiled to 31 and 27 instructions per ordinary step, each with four loads and
two stores. Their outputs were checked for correctness, but neither was timed. They
established that some of the machinery could be removed; they did not establish another
performance result.

The next design made a different space–time choice, not just a different instruction
schedule. Instead of tightly packing thirty-two 14-bit fields, compact30 puts thirty
directly addressable 16-bit offsets into sixty bytes and uses the remaining four bytes of a
64-byte cache-line record for a shared header. The header holds the base successor label and
the character for rows in that group. The ordinary step adds the selected offset to the
base. Groups crossing a character boundary, or too wide for the offsets, use a separate RAW
representation with full entries in groups padded to 128 bytes.

Two details remove work that FOR-14 had carried into every step. First, an ordinary 16-bit
field needs no packed-bit extraction. Second, the walker carries only its current row: the
emitted character comes from that row's header, not from an output-byte token carried
between lookups. Both the header and the chosen offset can be addressed from the current
row, without waiting for one load to discover the other's address. The compiled step still
performs more work than the conventional walk, but far less than the first packed version.
The [annotated machine-code
review](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md)
records the final path.

<figure class="bf-fig">
<p class="bf-title">A compact record leaves four bytes for the base and output byte</p>
<svg viewBox="0 0 720 138" role="img" aria-label="A 64-byte record: sixty bytes hold thirty 16-bit row offsets, and four bytes hold the shared base and output byte. The next position is the base plus the row's offset.">
<rect x="4" y="24" width="666" height="50" rx="8" class="bf-cell"/><rect x="629" y="24" width="41" height="50" rx="4" class="bf-cell-hi"/>
<text x="312" y="45" class="bf-ts" text-anchor="middle">30 directly addressed offsets</text><text x="312" y="62" class="bf-tm" text-anchor="middle">30 × 2 bytes = 60 bytes</text>
<text x="650" y="45" class="bf-ts" text-anchor="middle" style="font-size: 10px">H</text><text x="650" y="62" class="bf-tm" text-anchor="middle" style="font-size: 10px">4 B</text>
<text x="4" y="96" class="bf-t">Ordinary step: next position = base from H + this row's offset</text>
<text x="4" y="120" class="bf-tm">H also supplies the output byte; exceptional groups use separate RAW entries.</text>
</svg>
<figcaption>Sixty-four bytes encode thirty ordinary row transitions. Exceptional groups have a separate representation. This shows only the common case; segment starts and parking are handled by the walker.</figcaption>
</figure>

The engineering progression can be read in the compiled ordinary paths:

| Navigation layout | Instructions | Data loads | Stores | Evidence |
| --- | ---: | ---: | ---: | --- |
| Conventional 32-bit table | **16** | 3 | 2 | Production baseline |
| FOR-14, packed offsets | **38** (37 with BMI2) | 5 | 3 | Measured; rejected |
| FOR-14 token variants | **31 / 27** | 4 | 2 | Correct and compiled; **not timed** |
| Refined 30-row, 16-bit offsets | **21** | 4 | 2 | Measured; the first to pass |

The designs did not share an identical encoding or experimental protocol; the instruction
counts are not points on a controlled timing curve. The FOR-14 trial's reported time ratios
and compact30's later thread-CPU ratios are separate measurements, each against its own
control. In particular, we have no performance measurements for the intermediate 31- and
27-instruction variants. What the sequence does show is the sustained effort to make compact
*navigation* practical, instead of optimising storage in isolation.

We deliberately allowed the final record to grow a little: 2.164 logical bytes per row,
about 4.2% larger than FOR-14's 2.076, while retaining a 45.9% reduction from the shipped
four-byte entry. The [30-row reference
census](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-census.md)
established exact reconstruction of 1,513 blocks across eleven corpora. Its original token
packing and alignment were subsequently refined, and the finished [experimental
walker](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md)
passed separate correctness and machine-code reviews. The census was not a timing result for
that final walker.

Read against Seward's paper, the record combines three of its ideas. Its offsets rest on the
ordering behind T's construction: within one byte's bucket the links increase with the row,
so a group's links are a base plus small non-negative offsets, smallest where the byte's
occurrences in L sit close together, as in kennedy's runs. In Wikidata's blocks they usually
do: about 77% of a full block's rows fall in 32-row groups that each come from a single run
in L. The F byte in the header is indexF's idea, emitting from the first column so that L
isn't needed during the walk, with one stored byte per group in place of a binary search. And
keeping the header and the offset in one 64-byte line is mergedTL's: an ordinary step reads
one cache line. At 2.164 bytes a row, the record sits on Seward's memory axis between
treeopt-1M and indexF, and aims at mergedTL's speed.

At 21 instructions and four data loads, the compact30 common step is still more expensive
than production's 16-instruction, three-load step when considered by itself. The question
was whether lowering the table's shared-memory cost could more than compensate for that
extra work as several cores traversed at once.

## The first positive traversal result

We compared the shipped T-vector traversal with the frozen compact30 walker on the
established 434-block Wikidata corpus, covering 207 million inverse-BWT rows. Each output
was checked against the production decoder's result. The [registered
protocol](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md)
paired the two implementations across twenty measured rounds, first with one pinned physical
core and then with four workers starting their traversals together.

The timed interval includes traversal and assembly of the decoded BWT block, not table
construction. Before each walk, production builds its actual table; compact30 instead copies
a reference-built compact image into its arena. Those preparation operations are excluded
from the timer and may leave different cache states. What this experiment can tell us is how
the two *prepared* implementations behave under its stated conditions—not the eventual cost
or cache state of a direct compact30 builder.

The CPU-time figures add up how long the walking threads actually spent executing and divide
by BWT rows. The completion figure instead measures the earliest start to the latest finish
in each synchronized step, summed across the corpus. Neither includes table preparation or
output validation. The 95% intervals describe paired variation across the twenty rounds, not
a guarantee for other machines or workloads.

| Prepared-table traversal | Conventional | Compact30 | Compact ÷ conventional (95% interval) |
| --- | ---: | ---: | ---: |
| CPU time per BWT row, one core | 1.221 ns | 1.402 ns | **1.148** (1.147–1.149) |
| CPU time per BWT row, four concurrent workers | 2.055 ns | 1.727 ns | **0.840** (0.838–0.843) |
| Four-worker lockstep completion, one corpus pass | 114.45 ms | 97.97 ms | **0.856** (0.852–0.860) |

<figure class="bf-fig">
<p class="bf-title">Extra lookup work is repaid under concurrent traversal</p>
<svg viewBox="0 0 720 265" role="img" aria-label="CPU nanoseconds per BWT row, lower is better. On one core the conventional walker costs 1.221 and compact30 1.402. With four concurrent workers the conventional walker costs 2.055 and compact30 1.727.">
<text x="154" y="20" class="bf-tm">CPU nanoseconds per BWT row · lower is better</text>
<path d="M156 37V218M266 37V218M376 37V218M486 37V218M596 37V218" class="bf-grid"/>
<text x="156" y="234" class="bf-tm" text-anchor="middle">0</text><text x="266" y="234" class="bf-tm" text-anchor="middle">0.5</text><text x="376" y="234" class="bf-tm" text-anchor="middle">1.0</text><text x="486" y="234" class="bf-tm" text-anchor="middle">1.5</text><text x="596" y="234" class="bf-tm" text-anchor="middle">2.0</text>
<text x="8" y="77" class="bf-ts">One core</text><text x="8" y="163" class="bf-ts">Four workers</text>
<rect x="156" y="48" width="268.5" height="24" rx="4" class="bf-slot"/><rect x="156" y="79" width="308.4" height="24" rx="4" class="bf-cell-hi"/>
<rect x="156" y="134" width="452.1" height="24" rx="4" class="bf-slot"/><rect x="156" y="165" width="380.0" height="24" rx="4" class="bf-cell-hi"/>
<text x="431" y="65" class="bf-t-strong">1.221</text><text x="471" y="96" class="bf-t-strong">1.402</text><text x="615" y="151" class="bf-t-strong">2.055</text><text x="543" y="182" class="bf-t-strong">1.727</text>
<rect x="158" y="247" width="14" height="10" rx="2" class="bf-slot"/><text x="180" y="256" class="bf-tm">Conventional</text><rect x="312" y="247" width="14" height="10" rx="2" class="bf-cell-hi"/><text x="334" y="256" class="bf-tm">Compact30</text>
</svg>
<figcaption>CPU time per row is measured for each walking thread and summed. Compact30 is slower alone but faster with four workers. The chart shows the reversal, not its cause; construction is excluded.</figcaption>
</figure>

The reversal repays the design effort in a very specific setting. On one core the compact
walker spends 14.85% more CPU time per row. With four concurrent walkers it spends 15.95%
less, and their summed lockstep completion time is 14.40% lower. That completion measure is
not a measurement of whole-file decoding speed.

The absolute times help explain the crossover without pretending to identify its cause. On
one core, compact30 starts with an extra 0.181 ns per row of traversal cost. When the
workload changes to four concurrent walkers, the conventional walk's cost increases by 0.834
ns per row, against 0.325 ns for compact30. The difference between those increases is 0.509
ns—larger than compact30's initial 0.181 ns disadvantage—leaving it about 0.328 ns per row
ahead in the four-worker measurement. That is arithmetic on the two measured conditions. It
does not isolate cache misses, memory traffic or instruction execution as the cause of the
different increases.

This was not a result carried by a few convenient blocks. Of the 230 full-sized blocks, all
230 had lower four-worker traversal CPU cost with compact30 when averaged over rounds. In
1,138 of the 1,140 full-sized four-worker steps, the total CPU time was lower as well. The
204 short blocks were slower, but accounted for just 8,324 of the corpus's 207 million rows.
The same blocks recurred in each round, so these counts are robustness checks within one
experiment, not hundreds of independent replications. The complete [capture and independent
reconstruction](https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4)
are preserved.

It took some care to obtain a trustworthy comparison. Earlier attempts were stopped by
toolchain and disassembly checks, and one complete capture was invalid because the four
walks were not starting sufficiently close together. A subsequent, separately registered
change to the start barrier corrected that condition without changing the walker or relaxing
the threshold. In the valid capture every registered check passed. By the decision rule
fixed before the run, that was enough, for the first time with a compact table, to justify
building a real constructor. It was not a case for putting the experimental walker into the
decoder.

## A promising traversal is not yet a better decoder

The measured crossover is the one we had asked about in October: a table that loses when
traversed in isolation can become favourable when multiple workers compete for the machine's
memory resources. What the experiment does not tell us is why their costs increase so
differently.

A smaller shared navigation footprint is the leading hypothesis: approximately 7.79 MB for
four full compact30 tables rather than 14.4 MB for four conventional tables. But this figure
is calculated logical table storage, not measured cache occupancy, peak process memory or
DRAM traffic. The experiment did not collect the counters needed to distinguish fewer
evictions from translation effects, request-service pressure or the differing cache state
left by table preparation. The contention-related advantage is observed; its precise
hardware cause is not isolated.

This result is therefore narrower, and more valuable, than simply saying that compression
made decompression faster. At a fixed concurrent traversal workload, we have measured a
representation that uses fewer logical table bytes and completes the traversal with less CPU
time. That is a new favourable point for this stage of the computation, despite more work
per lookup. We have *not* yet demonstrated an outward shift in the full decoder's
throughput–memory frontier—the set of complete-decoder configurations that are not beaten on
both speed and measured memory use.

There is one more reason not to conflate the two. A real decoding worker also reconstructs
the last column, builds the navigation table, assembles output and performs further
processing. Workers are not guaranteed to be traversing at the same time. The 14.85%
isolated traversal penalty could matter whenever only one or two are walking. The
four-worker saving cannot be multiplied through the whole program without measuring that
mixture.

The next necessary piece is construction. The current reference builder first creates a full
temporary successor table, then converts it to compact30 records. A production builder must
generate the frozen records directly from the block's last column, competing with the
[current single-pass counting-sort
builder](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L253-L318).
In the [registered planning
model](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md#6-the-construction-budget-and-the-decision-rules-fixed-before-any-run),
retaining the measured four-worker traversal advantage would permit a direct builder costing
at most about 6.7% more than production's, using the conservative confidence-interval
endpoint. That allowance is conditional on the gain surviving construction and the decoder's
natural concurrency; it is not a prediction that the final decoder will meet its target.

So we have passed the first difficult test of the idea, not finished it. The
[direct-constructor design
brief](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md)
now asks whether the same representation can be built affordably, without the full temporary
table. If it can, the decisive comparison will return to the original question: does compact
navigation give the complete decoder more throughput for less measured memory, against the
best conventional configuration?

The original gamble was that reduced navigation storage could outweigh the instructions
needed to read it, once several BWT traversals had to share a memory system. For
prepared-table traversal under controlled contention, it has. FOR-14 showed that a small
table could still be too costly to navigate; the refined, less densely packed design has now
shown a positive result. The direct constructor and the complete decoder remain the tests of
whether that progress becomes a practical improvement.

---

Research state, 2 November 2026: the prepared-table traversal comparison is valid, and
justified the constructor work. Direct-constructor cost, full-decoder throughput and peak
memory, and the cause of the contention advantage have not yet been established. New results
should be added with their own dates and measurement boundaries; this result and its source
archive remain unchanged.