---
title: "The mathematical structure of inverse-BWT navigation"
image: assets/social/blog-2026-11-08-inverse-bwt-navigation.png
description: "The inverse BWT's next-row map is one shift per run of equal bytes, so it can be stored per run rather than per row. Whether that beats mibz2's table is unproven."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The mathematical structure of inverse-BWT navigation

The inverse BWT's next-row map is one shift per run of equal bytes, so it can be stored per
run rather than per row. Whether that beats mibz2's table is unproven.
{: .post-dek }

8 November 2026 · 14 min read · [Open question](#6-what-would-it-cost-mibz2-and-what-would-settle-it)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

To undo the [Burrows–Wheeler transform](https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform)
(BWT), a bzip2 decoder follows a chain through each block: from one row of the transform's
sorted matrix to the next, emitting one byte at each step. mibz2, our bzip2 decoder, stores
that chain as a table with one navigation entry per row, which we call the T-vector. Each
entry is four bytes, holding the next row and the byte to emit. At the maximum bzip2 block
size of 900,000 bytes, the T-vector occupies 3.6 MB. The inverse BWT is the expensive part of
decoding a block, because each step's address comes from the entry just read; mibz2 walks
many such chains at once so that their memory reads overlap.
[An earlier post](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md) works the
transform through on a small example.

**The entries of that table are not independent.** When the transformed word `banana` is
decoded, rows 0, 1 and 2, the three that emit `a`, have next rows 3, 4 and 5. One descriptor
covers all three: the first row, the first next row and the length. Every run of equal bytes in the BWT's output works the same way,
so the whole table can be described by one shift per run, $r$ of them, instead of one entry
per row, $n$ of them. **A walk can follow those shifts without searching for the right one
at each step,** by remembering which run it is in. Data structures that do this, *move
structures*, are published, with proofs:
[Nishimoto and Tabei (2021)](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2021.101)
gave constant-time steps in $O(r)$ machine words, and
[Brown and Langmead (2026)](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html)
inverted a whole BWT in $O(n)$ time with $O(r)$ additional working space, starting from a
run-length BWT representation.

**What is not proved** is that any of them beats mibz2's 32-lane, four-byte-per-row
implementation in elapsed CPU cycles. Everything in this post is mathematics and published
theory: nothing in it has been built or measured in mibz2, and it changes nothing in the
decoder. Nor has $r$ been counted on mibz2's blocks; [section 3](#3-when-do-the-shifts-save-nothing)
derives an upper bound from an earlier measurement.

[Section 1](#1-what-does-the-next-row-map-look-like-for-banana) shows both ideas on `banana`,
and [section 2](#2-why-does-every-run-give-one-shift) proves the first in general.
[Section 3](#3-when-do-the-shifts-save-nothing) shows when it saves nothing. Sections
[4](#4-how-does-a-walk-move-between-intervals-in-constant-time) and
[5](#5-can-a-whole-block-be-inverted-this-way) give the published theory, and
[section 6](#6-what-would-it-cost-mibz2-and-what-would-settle-it) what it would cost mibz2 and
what would settle it. Secondary results are in a
[technical appendix](2026-11-08-inverse-bwt-navigation-notes.md).

## 1. What does the next-row map look like for `banana`?

Suppose you want to compress the word `banana` with bzip2. The BWT writes down each of its
six rotations, sorts them, and keeps two things: the last column of the sorted list, $L$, and
the number of the row holding the original word, here 3. The first column, $F$, is the same
bytes in sorted order. The last column of the table, the next row, is explained below it.

| Row | Rotation | Next row |
| ---: | --- | ---: |
| 0 | `abanan` | 3 |
| 1 | `anaban` | 4 |
| 2 | `ananab` | 5 |
| 3 | `banana` | 2 |
| 4 | `nabana` | 0 |
| 5 | `nanaba` | 1 |

Reading down the rotations' last letters gives $L=\texttt{nnbaaa}$, and their first letters
give $F=\texttt{aaabnn}$. The next row of row $i$, written $\Psi(i)$,
is the row whose rotation is row $i$'s rotation with its first byte moved to the end. Row 3,
`banana`, becomes `ananab`, which is row 2. To decode, start at row 3, emit $F[3]$, move to
$\Psi(3)$, and repeat. The rows visited are

$$
3\to2\to5\to1\to4\to0,
$$

and the bytes emitted are `banana`. mibz2's T-vector stores exactly this: for each row, $F[i]$
and $\Psi(i)$.

Moving a rotation's first byte to its end puts that byte last, so $\Psi(i)$ is always a row
whose $L$ byte is $F[i]$: the three `a` rows of $F$ go to the three `a` positions of $L$. Which
`a` goes to which, and why that lets three entries be stored as one, is what the next figure
works out. It takes four steps on the same six rows, and each step's sentence says what changed.

<figure class="bf-fig">
<p class="bf-title">Why consecutive copies of a byte go to consecutive rows, and three arrows become one descriptor</p>
<div class="bf-frames">
<div class="bf-frame">
<svg viewBox="0 0 340 230" role="img" aria-label="First column F, aaabnn, and last column L, nnbaaa, rows 0 to 5, each byte numbered by its copy. Row 0, the first a of F, points to position 3, the first a of L; row 1, the second a, points to position 4. Row 2, the third a, has a question mark."><text x="75" y="14" class="bf-tm" text-anchor="middle">first column F</text><text x="265" y="14" class="bf-tm" text-anchor="middle">last column L</text><rect x="50" y="26" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="45" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="26" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="45" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="44" class="bf-tm" text-anchor="end">0</text><text x="298" y="44" class="bf-tm">0</text><rect x="50" y="60" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="79" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="60" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="79" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="78" class="bf-tm" text-anchor="end">1</text><text x="298" y="78" class="bf-tm">1</text><rect x="50" y="94" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="113" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><rect x="240" y="94" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="113" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="112" class="bf-tm" text-anchor="end">2</text><text x="298" y="112" class="bf-tm">2</text><rect x="50" y="128" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="147" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="128" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="147" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="146" class="bf-tm" text-anchor="end">3</text><text x="298" y="146" class="bf-tm">3</text><rect x="50" y="162" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="181" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="162" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="181" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="180" class="bf-tm" text-anchor="end">4</text><text x="298" y="180" class="bf-tm">4</text><rect x="50" y="196" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="215" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="196" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="215" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><text x="42" y="214" class="bf-tm" text-anchor="end">5</text><text x="298" y="214" class="bf-tm">5</text><path d="M104 40L236 142M228.1 140.7L236 142L232.7 134.7" class="bf-icon-green"/><path d="M104 74L236 176M228.1 174.7L236 176L232.7 168.7" class="bf-icon-green"/><path d="M104 108L150 108M142.9 111.8L150 108L142.9 104.2" class="bf-arrow" stroke-dasharray="4 4"/><text x="162" y="114" class="bf-ts">?</text></svg>
<p class="bf-step"><strong>1. Number the copies.</strong> <em>F</em> is <em>L</em>'s six bytes, sorted. The small numbers count each byte's copies from the top of its column. Position <em>j</em> of <em>L</em> is the last byte of row <em>j</em>, so a position is also a row. The table gives row 0's next row as 3 and row 1's as 4: the 1st <code>a</code> of <em>F</em> goes to the 1st <code>a</code> of <em>L</em>, and the 2nd to the 2nd. Before reading on, predict where row 2 goes.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 256" role="img" aria-label="Rows 0 to 2 are a followed by banan, naban and nanab. Moving the a to the end gives banan, naban and nanab followed by a, rows 3 to 5, in the same order, so the first, second and third a of F go to the first, second and third a of L."><text x="78" y="14" class="bf-tm" text-anchor="middle">rows that start with a</text><text x="262" y="14" class="bf-tm" text-anchor="middle">rows that end with a</text><text x="14" y="44" class="bf-tm" text-anchor="end">0</text><rect x="22" y="26" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="35" y="45" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><text x="54" y="45" class="bf-mono">banan</text><text x="286" y="147" class="bf-mono" text-anchor="end">banan</text><rect x="292" y="128" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="305" y="147" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><text x="326" y="146" class="bf-tm">3</text><path d="M112 40L228 142M220.2 140.2L228 142L225.2 134.5" class="bf-icon-green"/><text x="14" y="78" class="bf-tm" text-anchor="end">1</text><rect x="22" y="60" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="35" y="79" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><text x="54" y="79" class="bf-mono">naban</text><text x="286" y="181" class="bf-mono" text-anchor="end">naban</text><rect x="292" y="162" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="305" y="181" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><text x="326" y="180" class="bf-tm">4</text><path d="M112 74L228 176M220.2 174.2L228 176L225.2 168.5" class="bf-icon-green"/><text x="14" y="112" class="bf-tm" text-anchor="end">2</text><rect x="22" y="94" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="35" y="113" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><text x="54" y="113" class="bf-mono">nanab</text><text x="286" y="215" class="bf-mono" text-anchor="end">nanab</text><rect x="292" y="196" width="26" height="28" rx="5" class="bf-cell-hi"/><text x="305" y="215" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><text x="326" y="214" class="bf-tm">5</text><path d="M112 108L228 210M220.2 208.2L228 210L225.2 202.5" class="bf-icon-green"/><text x="170" y="248" class="bf-tm" text-anchor="middle">the same five letters in the same order</text></svg>
<p class="bf-step"><strong>2. Why the order is kept.</strong> Rows 0–2 are the rotations that start with <code>a</code>, sorted by the five letters after it. Moving the <code>a</code> to the end gives the rotations that end with <code>a</code>. Those sort by the same five letters, so they come in the same order. The <em>k</em>th <code>a</code> of <em>F</em> goes to the <em>k</em>th <code>a</code> of <em>L</em>, and row 2 goes to 5. The argument holds for every byte of every block.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 296" role="img" aria-label="The three arrows from F rows 0 to 2, now dashed, are covered by one band to L positions 3 to 5, shift plus 3. Below, one descriptor: source 0, destination 3, length 3, byte a."><text x="75" y="14" class="bf-tm" text-anchor="middle">first column F</text><text x="265" y="14" class="bf-tm" text-anchor="middle">last column L</text><rect x="50" y="26" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="45" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="26" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="45" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="44" class="bf-tm" text-anchor="end">0</text><text x="298" y="44" class="bf-tm">0</text><rect x="50" y="60" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="79" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="60" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="79" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="78" class="bf-tm" text-anchor="end">1</text><text x="298" y="78" class="bf-tm">1</text><rect x="50" y="94" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="113" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><rect x="240" y="94" width="50" height="28" rx="5" class="bf-cell"/><text x="262" y="113" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="112" class="bf-tm" text-anchor="end">2</text><text x="298" y="112" class="bf-tm">2</text><rect x="50" y="128" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="147" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="128" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="147" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="146" class="bf-tm" text-anchor="end">3</text><text x="298" y="146" class="bf-tm">3</text><rect x="50" y="162" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="181" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="162" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="181" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="180" class="bf-tm" text-anchor="end">4</text><text x="298" y="180" class="bf-tm">4</text><rect x="50" y="196" width="50" height="28" rx="5" class="bf-cell"/><text x="72" y="215" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="196" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="215" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><text x="42" y="214" class="bf-tm" text-anchor="end">5</text><text x="298" y="214" class="bf-tm">5</text><path d="M103 26L237 128L237 224L103 122Z" class="bf-band"/><path d="M104 40L236 142M228.1 140.7L236 142L232.7 134.7" class="bf-arrow" stroke-dasharray="4 4"/><path d="M104 74L236 176M228.1 174.7L236 176L232.7 168.7" class="bf-arrow" stroke-dasharray="4 4"/><path d="M104 108L236 210M228.1 208.7L236 210L232.7 202.7" class="bf-arrow" stroke-dasharray="4 4"/><text x="170" y="214" class="bf-ts" text-anchor="middle">shift +3</text><rect x="18" y="244" width="76" height="50" rx="6" class="bf-card-hi"/><text x="56" y="261" class="bf-tm" text-anchor="middle">source</text><text x="56" y="284" class="bf-num" text-anchor="middle">0</text><rect x="96" y="244" width="76" height="50" rx="6" class="bf-card-hi"/><text x="134" y="261" class="bf-tm" text-anchor="middle">destination</text><text x="134" y="284" class="bf-num" text-anchor="middle">3</text><rect x="174" y="244" width="76" height="50" rx="6" class="bf-card-hi"/><text x="212" y="261" class="bf-tm" text-anchor="middle">length</text><text x="212" y="284" class="bf-num" text-anchor="middle">3</text><rect x="252" y="244" width="76" height="50" rx="6" class="bf-card"/><text x="290" y="261" class="bf-tm" text-anchor="middle">byte</text><text x="290" y="284" class="bf-num" text-anchor="middle">a</text></svg>
<p class="bf-step"><strong>3. A run collapses the arrows.</strong> The three copies of <code>a</code> in <em>L</em> are adjacent, at positions 3–5: they are one run. So consecutive rows of <em>F</em> go to consecutive positions, and each arrow is the one above it moved down a row. One descriptor holds all three: where the rows start in <em>F</em>, where their next rows start, and how many there are. The shift, +3, is destination minus source. Had the copies of <code>a</code> in <em>L</em> been split into two runs, the <code>a</code> rows of <em>F</em> would split into two descriptors.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 346" role="img" aria-label="All three runs as bands: interval A, F rows 0 to 2, to L positions 3 to 5, shift plus 3; interval B, row 3, to position 2, shift minus 1; interval N, rows 4 and 5, to positions 0 and 1, shift minus 4. A table lists the three descriptors. Six next-row entries become three descriptors."><text x="75" y="14" class="bf-tm" text-anchor="middle">first column F</text><text x="265" y="14" class="bf-tm" text-anchor="middle">last column L</text><rect x="50" y="26" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="45" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="26" width="50" height="28" rx="5" class="bf-cell-steel"/><text x="262" y="45" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="44" class="bf-tm" text-anchor="end">0</text><text x="298" y="44" class="bf-tm">0</text><rect x="50" y="60" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="79" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="60" width="50" height="28" rx="5" class="bf-cell-steel"/><text x="262" y="79" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="78" class="bf-tm" text-anchor="end">1</text><text x="298" y="78" class="bf-tm">1</text><rect x="50" y="94" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="72" y="113" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><rect x="240" y="94" width="50" height="28" rx="5" class="bf-cell-amber"/><text x="262" y="113" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="112" class="bf-tm" text-anchor="end">2</text><text x="298" y="112" class="bf-tm">2</text><rect x="50" y="128" width="50" height="28" rx="5" class="bf-cell-amber"/><text x="72" y="147" class="bf-t-strong" text-anchor="middle">b<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="128" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="147" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">1</tspan></text><text x="42" y="146" class="bf-tm" text-anchor="end">3</text><text x="298" y="146" class="bf-tm">3</text><rect x="50" y="162" width="50" height="28" rx="5" class="bf-cell-steel"/><text x="72" y="181" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">1</tspan></text><rect x="240" y="162" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="181" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">2</tspan></text><text x="42" y="180" class="bf-tm" text-anchor="end">4</text><text x="298" y="180" class="bf-tm">4</text><rect x="50" y="196" width="50" height="28" rx="5" class="bf-cell-steel"/><text x="72" y="215" class="bf-t-strong" text-anchor="middle">n<tspan class="bf-tm" dy="4">2</tspan></text><rect x="240" y="196" width="50" height="28" rx="5" class="bf-cell-hi"/><text x="262" y="215" class="bf-t-strong" text-anchor="middle">a<tspan class="bf-tm" dy="4">3</tspan></text><text x="42" y="214" class="bf-tm" text-anchor="end">5</text><text x="298" y="214" class="bf-tm">5</text><text x="8" y="79" class="bf-ts">A</text><text x="8" y="147" class="bf-ts">B</text><text x="8" y="198" class="bf-ts">N</text><path d="M103 162L237 26L237 88L103 224Z" class="bf-band-steel"/><path d="M103 26L237 128L237 224L103 122Z" class="bf-band"/><path d="M103 128L237 94L237 122L103 156Z" class="bf-band-amber"/><text x="118" y="78" class="bf-ts">+3</text><text x="124" y="144" class="bf-ts">−1</text><text x="124" y="214" class="bf-ts">−4</text><text x="22" y="248" class="bf-tm"></text><text x="110" y="248" class="bf-tm" text-anchor="middle">source</text><text x="180" y="248" class="bf-tm" text-anchor="middle">destination</text><text x="245" y="248" class="bf-tm" text-anchor="middle">length</text><text x="300" y="248" class="bf-tm" text-anchor="middle">byte</text><text x="22" y="268" class="bf-ts">A</text><text x="110" y="268" class="bf-t-strong" text-anchor="middle">0</text><text x="180" y="268" class="bf-t-strong" text-anchor="middle">3</text><text x="245" y="268" class="bf-t-strong" text-anchor="middle">3</text><text x="300" y="268" class="bf-t-strong" text-anchor="middle">a</text><text x="22" y="288" class="bf-ts">B</text><text x="110" y="288" class="bf-t-strong" text-anchor="middle">3</text><text x="180" y="288" class="bf-t-strong" text-anchor="middle">2</text><text x="245" y="288" class="bf-t-strong" text-anchor="middle">1</text><text x="300" y="288" class="bf-t-strong" text-anchor="middle">b</text><text x="22" y="308" class="bf-ts">N</text><text x="110" y="308" class="bf-t-strong" text-anchor="middle">4</text><text x="180" y="308" class="bf-t-strong" text-anchor="middle">0</text><text x="245" y="308" class="bf-t-strong" text-anchor="middle">2</text><text x="300" y="308" class="bf-t-strong" text-anchor="middle">n</text><path d="M14 322H326" class="bf-rule"/><text x="170" y="340" class="bf-ts" text-anchor="middle">6 next-row entries become 3 descriptors</text></svg>
<p class="bf-step"><strong>4. One descriptor per run of <em>L</em>.</strong> The <code>b</code> at position 2 receives row 3, a shift of −1, and the run <code>nn</code> receives rows 4–5, a shift of −4. We call the three source intervals A, B and N, after their bytes. In <em>F</em> they come in byte order, in <em>L</em> in position order. Six next-row entries become three descriptors. That counts entries, not bytes: a descriptor holds more than a T-vector entry (section 2.1).</p>
</div>
</div>
<figcaption>Every frame shows the six rows of banana. Green, amber and blue mark the bytes a, b and n. Conceptual, worked by hand from the table above.</figcaption>
</figure>

Written out, the whole map is

$$
\Psi(i)=\begin{cases}i+3,&0\le i<3,\\i-1,&i=3,\\i-4,&4\le i<6.\end{cases}
$$

**Three row entries become one descriptor:** a source start, a destination start and a length,
with the byte the rows emit. Six table entries become three descriptors. The three pieces are
not special to `banana`: they are the three runs of $L$, `nn`, `b` and `aaa`. That is the first
result of this post. **The next-row map is a set of shifted intervals, one for each run of the
last column.**

### 1.1 Walking the intervals without a search

Storing intervals instead of rows raises a question. To find the next row of row $i$, a
decoder would first have to find which interval holds $i$, which is a search among the $r$
starting rows. A walk doesn't need it, because it can carry the interval's name along with
the row. Each descriptor also records one more value, its *anchor*: the interval its
destinations begin in. Interval A's destinations begin at row 3, which lies in interval B. The
next figure takes one step of the walk, from row 1, using the descriptors of the last figure.

<figure class="bf-fig">
<p class="bf-title">One step of the walk: from row 1 in interval A to row 4 in interval N</p>
<div class="bf-frames">
<div class="bf-frame">
<svg viewBox="0 0 340 264" role="img" aria-label="Rows 0 to 5 with bytes aaabnn grouped into intervals A, B and N, and a descriptor table: A, rows 0 to 2, next rows start at 3, which is in B; B, row 3, next row 2, in A; N, rows 4 and 5, next rows start at 0, in A. Row 1 and descriptor A are highlighted. The state is row 1, interval A; emit a."><text x="68" y="14" class="bf-tm" text-anchor="middle">rows</text><rect x="46" y="34" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="51" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="50" class="bf-tm" text-anchor="end">0</text><rect x="46" y="64" width="44" height="24" rx="5" class="bf-cell-hi"/><rect x="44" y="62" width="48" height="28" rx="6" class="bf-outline"/><text x="68" y="81" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="80" class="bf-tm" text-anchor="end">1</text><rect x="46" y="94" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="111" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="110" class="bf-tm" text-anchor="end">2</text><rect x="46" y="124" width="44" height="24" rx="5" class="bf-cell-amber"/><text x="68" y="141" class="bf-t-strong" text-anchor="middle">b</text><text x="40" y="140" class="bf-tm" text-anchor="end">3</text><rect x="46" y="154" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="171" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="170" class="bf-tm" text-anchor="end">4</text><rect x="46" y="184" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="201" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="200" class="bf-tm" text-anchor="end">5</text><text x="6" y="81" class="bf-ts">A</text><text x="6" y="141" class="bf-ts">B</text><text x="6" y="186" class="bf-ts">N</text><text x="200" y="22" class="bf-tm" text-anchor="middle">source</text><text x="258" y="22" class="bf-tm" text-anchor="middle">destination</text><text x="312" y="22" class="bf-tm" text-anchor="middle">anchor</text><rect x="146" y="40" width="190" height="26" rx="6" class="bf-card-hi"/><text x="152" y="58" class="bf-ts">A</text><text x="200" y="58" class="bf-t-strong" text-anchor="middle">0–2</text><text x="258" y="58" class="bf-t-strong" text-anchor="middle">3</text><text x="312" y="58" class="bf-t-strong" text-anchor="middle">B</text><text x="152" y="88" class="bf-ts">B</text><text x="200" y="88" class="bf-t-strong" text-anchor="middle">3</text><text x="258" y="88" class="bf-t-strong" text-anchor="middle">2</text><text x="312" y="88" class="bf-t-strong" text-anchor="middle">A</text><text x="152" y="118" class="bf-ts">N</text><text x="200" y="118" class="bf-t-strong" text-anchor="middle">4–5</text><text x="258" y="118" class="bf-t-strong" text-anchor="middle">0</text><text x="312" y="118" class="bf-t-strong" text-anchor="middle">A</text><rect x="10" y="228" width="320" height="34" rx="8" class="bf-card"/><text x="170" y="250" class="bf-ts" text-anchor="middle">state: row 1, interval A · emit a</text></svg>
<p class="bf-step"><strong>1. The state.</strong> The walk holds a row and the interval it lies in: row 1, interval A. It emits A's byte, <code>a</code>, and reads A's descriptor directly, by its name.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 264" role="img" aria-label="From row 1, descriptor A gives the next row as its first next row, 3, plus row 1's offset from A's first row, 0: 3 plus 1 minus 0 is 4. An arrow goes from row 1 to row 4."><text x="68" y="14" class="bf-tm" text-anchor="middle">rows</text><rect x="46" y="34" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="51" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="50" class="bf-tm" text-anchor="end">0</text><rect x="46" y="64" width="44" height="24" rx="5" class="bf-cell-hi"/><rect x="44" y="62" width="48" height="28" rx="6" class="bf-outline"/><text x="68" y="81" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="80" class="bf-tm" text-anchor="end">1</text><rect x="46" y="94" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="111" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="110" class="bf-tm" text-anchor="end">2</text><rect x="46" y="124" width="44" height="24" rx="5" class="bf-cell-amber"/><text x="68" y="141" class="bf-t-strong" text-anchor="middle">b</text><text x="40" y="140" class="bf-tm" text-anchor="end">3</text><rect x="46" y="154" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="171" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="170" class="bf-tm" text-anchor="end">4</text><rect x="46" y="184" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="201" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="200" class="bf-tm" text-anchor="end">5</text><text x="6" y="81" class="bf-ts">A</text><text x="6" y="141" class="bf-ts">B</text><text x="6" y="186" class="bf-ts">N</text><text x="200" y="22" class="bf-tm" text-anchor="middle">source</text><text x="258" y="22" class="bf-tm" text-anchor="middle">destination</text><text x="312" y="22" class="bf-tm" text-anchor="middle">anchor</text><rect x="146" y="40" width="190" height="26" rx="6" class="bf-card-hi"/><text x="152" y="58" class="bf-ts">A</text><text x="200" y="58" class="bf-t-strong" text-anchor="middle">0–2</text><text x="258" y="58" class="bf-t-strong" text-anchor="middle">3</text><text x="312" y="58" class="bf-t-strong" text-anchor="middle">B</text><text x="152" y="88" class="bf-ts">B</text><text x="200" y="88" class="bf-t-strong" text-anchor="middle">3</text><text x="258" y="88" class="bf-t-strong" text-anchor="middle">2</text><text x="312" y="88" class="bf-t-strong" text-anchor="middle">A</text><text x="152" y="118" class="bf-ts">N</text><text x="200" y="118" class="bf-t-strong" text-anchor="middle">4–5</text><text x="258" y="118" class="bf-t-strong" text-anchor="middle">0</text><text x="312" y="118" class="bf-t-strong" text-anchor="middle">A</text><path d="M94 76.0C128 76.0 128 166.0 96 166.0M103.0 161.5L96 166.0L103.0 170.5" class="bf-icon-green"/><rect x="10" y="228" width="320" height="34" rx="8" class="bf-card"/><text x="170" y="250" class="bf-ts" text-anchor="middle">next row = 3 + (1 − 0) = 4</text></svg>
<p class="bf-step"><strong>2. The arithmetic.</strong> Row 1 is one row past A's source start, 0. A's destination starts at 3, so the next row is 3 + (1 − 0) = 4. The next step needs to know which interval holds row 4.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 264" role="img" aria-label="Descriptor A records that its next rows start in interval B. The walk checks only B: B is row 3 alone, so row 4 is not in B."><text x="68" y="14" class="bf-tm" text-anchor="middle">rows</text><rect x="46" y="34" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="51" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="50" class="bf-tm" text-anchor="end">0</text><rect x="46" y="64" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="81" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="80" class="bf-tm" text-anchor="end">1</text><rect x="46" y="94" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="111" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="110" class="bf-tm" text-anchor="end">2</text><rect x="46" y="124" width="44" height="24" rx="5" class="bf-cell-amber"/><rect x="44" y="122" width="48" height="28" rx="6" class="bf-outline"/><text x="68" y="141" class="bf-t-strong" text-anchor="middle">b</text><text x="40" y="140" class="bf-tm" text-anchor="end">3</text><rect x="46" y="154" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="171" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="170" class="bf-tm" text-anchor="end">4</text><rect x="46" y="184" width="44" height="24" rx="5" class="bf-cell-steel"/><text x="68" y="201" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="200" class="bf-tm" text-anchor="end">5</text><text x="6" y="81" class="bf-ts">A</text><text x="6" y="141" class="bf-ts">B</text><text x="6" y="186" class="bf-ts">N</text><text x="200" y="22" class="bf-tm" text-anchor="middle">source</text><text x="258" y="22" class="bf-tm" text-anchor="middle">destination</text><text x="312" y="22" class="bf-tm" text-anchor="middle">anchor</text><rect x="146" y="40" width="190" height="26" rx="6" class="bf-card-hi"/><text x="152" y="58" class="bf-ts">A</text><text x="200" y="58" class="bf-t-strong" text-anchor="middle">0–2</text><text x="258" y="58" class="bf-t-strong" text-anchor="middle">3</text><text x="312" y="58" class="bf-t-strong" text-anchor="middle">B</text><rect x="298" y="42" width="28" height="22" rx="5" class="bf-outline"/><text x="152" y="88" class="bf-ts">B</text><text x="200" y="88" class="bf-t-strong" text-anchor="middle">3</text><text x="258" y="88" class="bf-t-strong" text-anchor="middle">2</text><text x="312" y="88" class="bf-t-strong" text-anchor="middle">A</text><text x="152" y="118" class="bf-ts">N</text><text x="200" y="118" class="bf-t-strong" text-anchor="middle">4–5</text><text x="258" y="118" class="bf-t-strong" text-anchor="middle">0</text><text x="312" y="118" class="bf-t-strong" text-anchor="middle">A</text><rect x="44" y="152" width="48" height="28" rx="6" class="bf-outline"/><text x="98" y="140" class="bf-tm">B ends</text><text x="98" y="170" class="bf-ts">row 4 is not in B</text><rect x="10" y="228" width="320" height="34" rx="8" class="bf-card"/><text x="170" y="250" class="bf-ts" text-anchor="middle">anchor B: is 4 in B (row 3 only)? no</text></svg>
<p class="bf-step"><strong>3. The anchor.</strong> A's descriptor names the interval its destination starts in: B. A's destinations start at row 3 and count upwards, so row 4 is in B or in an interval after it. The walk compares row 4 with B's end only. B is row 3 alone, so row 4 lies past it.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 290" role="img" aria-label="The walk steps forward one interval from B to N, rows 4 and 5, which holds row 4. The new state is row 4, interval N, which emits n. No search of all the interval starts was needed."><text x="68" y="14" class="bf-tm" text-anchor="middle">rows</text><rect x="46" y="34" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="51" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="50" class="bf-tm" text-anchor="end">0</text><rect x="46" y="64" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="81" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="80" class="bf-tm" text-anchor="end">1</text><rect x="46" y="94" width="44" height="24" rx="5" class="bf-cell-hi"/><text x="68" y="111" class="bf-t-strong" text-anchor="middle">a</text><text x="40" y="110" class="bf-tm" text-anchor="end">2</text><rect x="46" y="124" width="44" height="24" rx="5" class="bf-cell-amber"/><text x="68" y="141" class="bf-t-strong" text-anchor="middle">b</text><text x="40" y="140" class="bf-tm" text-anchor="end">3</text><rect x="46" y="154" width="44" height="24" rx="5" class="bf-cell-steel"/><rect x="44" y="152" width="48" height="28" rx="6" class="bf-outline"/><text x="68" y="171" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="170" class="bf-tm" text-anchor="end">4</text><rect x="46" y="184" width="44" height="24" rx="5" class="bf-cell-steel"/><rect x="44" y="182" width="48" height="28" rx="6" class="bf-outline"/><text x="68" y="201" class="bf-t-strong" text-anchor="middle">n</text><text x="40" y="200" class="bf-tm" text-anchor="end">5</text><text x="6" y="81" class="bf-ts">A</text><text x="6" y="141" class="bf-ts">B</text><text x="6" y="186" class="bf-ts">N</text><text x="200" y="22" class="bf-tm" text-anchor="middle">source</text><text x="258" y="22" class="bf-tm" text-anchor="middle">destination</text><text x="312" y="22" class="bf-tm" text-anchor="middle">anchor</text><text x="152" y="58" class="bf-ts">A</text><text x="200" y="58" class="bf-t-strong" text-anchor="middle">0–2</text><text x="258" y="58" class="bf-t-strong" text-anchor="middle">3</text><text x="312" y="58" class="bf-t-strong" text-anchor="middle">B</text><text x="152" y="88" class="bf-ts">B</text><text x="200" y="88" class="bf-t-strong" text-anchor="middle">3</text><text x="258" y="88" class="bf-t-strong" text-anchor="middle">2</text><text x="312" y="88" class="bf-t-strong" text-anchor="middle">A</text><rect x="146" y="100" width="190" height="26" rx="6" class="bf-card-hi"/><text x="152" y="118" class="bf-ts">N</text><text x="200" y="118" class="bf-t-strong" text-anchor="middle">4–5</text><text x="258" y="118" class="bf-t-strong" text-anchor="middle">0</text><text x="312" y="118" class="bf-t-strong" text-anchor="middle">A</text><text x="98" y="170" class="bf-ts">row 4 is in N</text><rect x="10" y="228" width="320" height="34" rx="8" class="bf-card-hi"/><text x="170" y="250" class="bf-ts" text-anchor="middle">new state: row 4, interval N · emit n</text><text x="170" y="282" class="bf-tm" text-anchor="middle">not needed: a search of all starts 0, 3, 4</text></svg>
<p class="bf-step"><strong>4. One step forward.</strong> The interval after B is N, rows 4–5, and it holds row 4. The new state is row 4 in N. The walk compared row 4 with B's and N's boundaries, not with every interval's start. Here one step forward was enough. In a large block one interval's destinations can cross many intervals, and section 4 shows how splitting intervals bounds those steps by a constant. A constant number of steps is still not one memory read.</p>
</div>
</div>
<figcaption>The rows on the left are F, grouped into the intervals of the last figure; the table is their descriptors, each with its anchor. Conceptual, worked by hand.</figcaption>
</figure>

Repeated from row 3, the same actions visit row 3 in B, then 2 in A, 5 in N, 1 in A, 4 in N and
0 in A, and emit `banana`. **Carrying the interval as traversal state turns each step into
arithmetic and a short forward step, with no search.** This is the idea behind move structures
([section 4](#4-how-does-a-walk-move-between-intervals-in-constant-time)).

The difference is between *random row access*, where any row can be asked for at any time,
and *stateful successor navigation*, where each question follows from the last answer. mibz2
needs mostly the second. Every lane of its walk follows a path and keeps state, which is why
move structures are relevant to it.

## 2. Why does every run give one shift?

The `banana` table follows from two facts: $\Psi(i)$ lands on a copy of $F[i]$ in $L$, and
equal bytes keep their order. Let

$$
L=L[0]L[1]\cdots L[n-1]
$$

be the BWT last column of a block, and let

$$
\Sigma=\{c_1<c_2<\cdots<c_\sigma\}
$$

be its distinct characters, ordered by byte value. Write $n_c$ for the frequency of character
$c$, and define $C(c)=\sum_{d<c}n_d$, the number of bytes smaller than $c$: the row where
$c$'s block of rows in $F$ begins. The first column is

$$
F=c_1^{n_{c_1}}c_2^{n_{c_2}}\cdots c_\sigma^{n_{c_\sigma}}.
$$

Define zero-based rank and select by

$$
\operatorname{rank}_c(L,j)=|\{i<j:L[i]=c\}|
$$

and

$$
\operatorname{select}_c(L,k)=\text{position of the }(k+1)\text{st occurrence of }c.
$$

The usual backward LF mapping is

$$
\operatorname{LF}(j)=C(L[j])+\operatorname{rank}_{L[j]}(L,j).
$$

mibz2 uses the inverse direction, which we shall consistently call $\Psi$:

$$
\boxed{\Psi=\operatorname{LF}^{-1}}
$$

and therefore

$$
\boxed{\Psi(C(c)+k)=\operatorname{select}_c(L,k).}\tag{1}
$$

In words: the $(k+1)$st $c$ of $F$ goes to the $(k+1)$st $c$ of $L$. That is the stable-sort
identity the `banana` table showed. For each row $i$, mibz2 needs to recover the next row
$\Psi(i)$ and emit the byte $F[i]$, while preserving the bookkeeping required by its segmented
traversal: mibz2 cuts each block's walk into segments that start at known rows, walks them in
parallel and joins them. That is the mathematical operation being optimised. The byte $F[i]$
needs no storage per row, since the histogram fixes it
([appendix section 1](2026-11-08-inverse-bwt-navigation-notes.md#1-navigation-as-a-select-problem)).

Now let $r$ be the number of maximal equal-character runs in $L$, and $r_c$ the number of runs
containing character $c$, so that $r=\sum_{c\in\Sigma}r_c$. Consider a run of character $c$
beginning at position $p$, with length $m$:

$$
L[p:p+m]=c^m.
$$

Let $k=\operatorname{rank}_c(L,p)$ be the number of earlier occurrences of $c$. The run
corresponds to the interval of rows in $F$ starting at $a=C(c)+k$. By equation (1), for every
$0\le t<m$, $\Psi(a+t)=p+t$. Therefore

$$
\boxed{\Psi(i)=i+(p-a),\qquad a\le i<a+m.}\tag{2}
$$

The entire run is represented by one constant displacement. Every character occurrence belongs
to exactly one run. Their corresponding intervals partition the domain of $\Psi$:

$$
\boxed{\begin{gathered}\Psi\text{ admits a partition into }r\text{ intervals,}\\\text{on each of which it is a translation.}\end{gathered}}\tag{3}
$$

In `banana`, the run `aaa` starts at $p=3$, and the `a` rows start at $a=0$: the shift is
$+3$, as the figure showed. In combinatorics, such a map is a *discrete interval-exchange
permutation*. This identification is more operationally useful than membership in a
weak-order lattice, because it tells us exactly how to evaluate the permutation.

**The $r$ pieces are an upper bound, not always the fewest.** For the valid cyclic BWT of
`abc`, $L=\texttt{cab}$, $F=\texttt{abc}$ and $r=3$, but $\Psi=(1,2,0)$ has just two maximal
translated intervals: rows $[0,2)$ have displacement $+1$, and row $[2,3)$ has displacement
$-2$. The partition of equation (3) is the one the runs induce. Adjacent pieces can share a
shift.

### 2.1 What the intervals cost to store

For each interval $t$, store its starting position $a_t$ and first destination $p_t$. Order
the intervals by their starting positions, $0=a_0<a_1<\cdots<a_r=n$. Then

$$
\boxed{\Psi(i)=p_t+(i-a_t)\quad\text{for}\quad a_t\le i<a_{t+1}.}\tag{4}
$$

No per-row destination array is required. For $n\le900{,}000$, each position fits in 20 bits,
so the basic representation uses approximately $B_{\mathrm{interval}}\approx40r$ bits, two
20-bit values per interval, plus indexing and metadata. The T-vector uses
$B_{\mathrm{T}}=32n$ bits. Ignoring indexing overhead, the interval representation is smaller
whenever $40r<32n$, or

$$
\boxed{\frac rn<0.8.}
$$

This is only a storage comparison for basic descriptors, not a complete performance or
working-memory bound. A practical move structure needs additional information to locate the
next interval. Nevertheless, it proves there is a potentially much smaller, exact
representation whenever the BWT contains sufficiently long runs.

## 3. When do the shifts save nothing?

When the runs are short. The valid cyclic BWT of `abba` is $L=\texttt{baba}$: four rows and
four runs, $r=n=4$. Its descriptors take $40r=160$ bits, more than the T-vector's $32n=128$
bits, before any index is added. The next figure sets it beside `banana`.

<figure class="bf-fig">
<p class="bf-title">Rows against runs: the T-vector's bits and the basic descriptors' bits</p>
<div class="bf-frames">
<div class="bf-frame">
<svg viewBox="0 0 340 168" role="img" aria-label="banana: L = nnbaaa, 6 rows and 3 runs. The T-vector takes 32n = 192 bits; basic run descriptors take 40r = 120 bits."><text x="14" y="18" class="bf-ts">L = nnbaaa</text><text x="326" y="18" class="bf-tm" text-anchor="end">n = 6 rows, r = 3 runs</text><rect x="14" y="30" width="30" height="26" rx="5" class="bf-cell-steel"/><text x="29" y="48" class="bf-t-strong" text-anchor="middle">n</text><rect x="46" y="30" width="30" height="26" rx="5" class="bf-cell-steel"/><text x="61" y="48" class="bf-t-strong" text-anchor="middle">n</text><rect x="86" y="30" width="30" height="26" rx="5" class="bf-cell-amber"/><text x="101" y="48" class="bf-t-strong" text-anchor="middle">b</text><rect x="126" y="30" width="30" height="26" rx="5" class="bf-cell-hi"/><text x="141" y="48" class="bf-t-strong" text-anchor="middle">a</text><rect x="158" y="30" width="30" height="26" rx="5" class="bf-cell-hi"/><text x="173" y="48" class="bf-t-strong" text-anchor="middle">a</text><rect x="190" y="30" width="30" height="26" rx="5" class="bf-cell-hi"/><text x="205" y="48" class="bf-t-strong" text-anchor="middle">a</text><text x="14" y="80" class="bf-t">T-vector, 32n = 192 bits</text><rect x="14" y="87" width="300.0" height="16" rx="3" class="bf-lat-hi"/><text x="14" y="126" class="bf-t">descriptors, 40r = 120 bits</text><rect x="14" y="133" width="187.5" height="16" rx="3" class="bf-green"/></svg>
<p class="bf-step"><strong>banana.</strong> Three runs in six rows. The descriptors take 120 bits, against the T-vector's 192.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 340 168" role="img" aria-label="abba: L = baba, 4 rows and 4 runs. The T-vector takes 32n = 128 bits; basic run descriptors take 40r = 160 bits."><text x="14" y="18" class="bf-ts">L = baba</text><text x="326" y="18" class="bf-tm" text-anchor="end">n = 4 rows, r = 4 runs</text><rect x="14" y="30" width="30" height="26" rx="5" class="bf-cell-amber"/><text x="29" y="48" class="bf-t-strong" text-anchor="middle">b</text><rect x="54" y="30" width="30" height="26" rx="5" class="bf-cell-hi"/><text x="69" y="48" class="bf-t-strong" text-anchor="middle">a</text><rect x="94" y="30" width="30" height="26" rx="5" class="bf-cell-amber"/><text x="109" y="48" class="bf-t-strong" text-anchor="middle">b</text><rect x="134" y="30" width="30" height="26" rx="5" class="bf-cell-hi"/><text x="149" y="48" class="bf-t-strong" text-anchor="middle">a</text><text x="14" y="80" class="bf-t">T-vector, 32n = 128 bits</text><rect x="14" y="87" width="200.0" height="16" rx="3" class="bf-lat-hi"/><text x="14" y="126" class="bf-t">descriptors, 40r = 160 bits</text><rect x="14" y="133" width="250.0" height="16" rx="3" class="bf-green"/></svg>
<p class="bf-step"><strong>abba.</strong> Every run is one byte long, so there is a descriptor per row. Four descriptors take 160 bits, against 128.</p>
</div>
</div>
<figcaption>Bits from the formulas of section 2.1, 32 per row and 40 per run, drawn to one scale. They count basic descriptors only: a move structure also stores anchors and an index. Not measured sizes.</figcaption>
</figure>

So run-based storage is not guaranteed to be smaller. The
comparison is of descriptors alone. A practical move structure stores more than this, so its
break-even point lies below $r/n=0.8$, by an amount nobody has measured for mibz2.

Nor does a block that compresses well necessarily have few runs. The two simplest measures, the zero-order entropy
$H_0(L)$ of the byte frequencies and $r$, answer different questions: a block can have low
entropy and many short runs, or high entropy and few long runs
([appendix section 3](2026-11-08-inverse-bwt-navigation-notes.md#3-space-bounds)). The run
count itself has a precise meaning: exactly $n-r$ pairs of adjacent rows within a character's
block have consecutive next rows
([appendix section 2](2026-11-08-inverse-bwt-navigation-notes.md#2-consecutive-entries-and-sparse-corrections)).

**How small is $r$ on mibz2's blocks?** It has not been counted. One earlier figure bounds it.
On the main Wikidata corpus, 207,006,515 rows in 434 blocks, the builder of compact30, our
research table that packs thirty rows into each 64-byte record, wrote 74.3% of rows through its bulk emitter, as 5,127,300 complete 30-row records, each covered by one
run of $L$ ([the 4 November post, section 6](2026-11-04-compact30-constructor.md#6-most-rows-never-take-the-per-row-path)).
Each of those runs holds at least one such record, so there are at most 5,127,300 of them.
The remaining 53.2 million rows can lie in at most 53.2 million further runs. So, across the
corpus, $r\le58.3$ million and $r/n\le0.28$. This is derived, not measured, and it is an
aggregate: it says nothing about the spread from block to block, and the true $r$ may be much
smaller.

## 4. How does a walk move between intervals in constant time?

The interval-exchange permutation of section 2 is precisely the sort of permutation handled by
*move structures*, introduced by Nishimoto and Tabei. Their 2021 result establishes
$O(1)$-time move queries with $O(r)$ words of space after balancing represented intervals.

An ordinary query is $i\mapsto\Psi(i)$. An interval representation must locate the interval
containing $i$, potentially requiring an expensive predecessor search: finding the last
interval start at or before $i$. Instead retain the pair
$(i,t)$, where $t$ identifies the interval. The next row follows directly:

$$
j=p_t+(i-a_t).
$$

We then need the interval $t'$ containing $j$. A move structure stores, for each interval, the
interval its destinations begin in (its anchor, in section 1.1), and advances through
descriptors to locate $j$,
as the `banana` walk stepped from B to N. Unbalanced intervals may require multiple advances.
The crucial theorem is that intervals can be split into $O(r)$ balanced pieces, guaranteeing a
constant number of advances:

$$
\boxed{(i,t)\xrightarrow{O(1)}(\Psi(i),t')}\tag{5}
$$

using $O(r)$ words in the word-RAM model, the standard model in which reading one machine
word is one step. Carrying $t$ is essential. This is not
one-memory-access arbitrary-row lookup. The theorem shows that compact representation does not
imply logarithmic lookup time. That removes a theoretical objection, not the hardware concern
about several serial descriptor loads: an $O(1)$ bound in the RAM model is not a claim of one
load, or of fewer cycles.

mibz2 already maintains per-lane state. It walks 32 interleaved lanes, each following one
segment, so that 32 independent memory reads can be in flight at once
([32 lanes, 8.6 chains](2026-10-31-room-walk-overlap.md)), and it supports parallel work
across blocks. A move structure adds a run/interval
identifier to each lane, giving state

$$
(\text{row},\text{interval},\text{segment state}).
$$

A fresh segment start may require predecessor search once. For $P$ segment starts initialised
with binary search, seeding costs $O(P\log r)$; later transitions maintain interval state. The
structure therefore removes general predecessor queries from the hot path. It does not remove
memory dependencies: a move query may load several descriptors. It does change the algorithmic
problem from repeated searches to constant-time state transitions.

## 5. Can a whole block be inverted this way?

Brown and Langmead's *Bounding the Average Move Structure Query for Faster and Smaller RLBWT
Permutations* (SEA 2026) studies *length capping*. (RLBWT is the run-length BWT: the last
column $L$ stored as its $r$ runs, each a character and a length.) Set the average interval
length to $n/r$; for constant $c$, split until every interval is of length at most

$$
\boxed{L_{\max}=c\frac nr.}\tag{6}
$$

The number of additional intervals is approximately at most $r/c$, so $r'\le r+r/c$, up to
rounding. Their Theorem 2 bounds aggregate work over all $n$ distinct positions, not strict
worst-case cost per query. The number of interval advances is bounded by a constant multiple
of $n$:

$$
\boxed{T_{\mathrm{all\ moves}}=O(n).}\tag{7}
$$

For a single-cycle permutation, full traversal visits every position once. The mathematical
argument can be set-based, not necessarily requiring one serial traversal order. They also show
$O(r)$-time/space construction of a capped structure for LF or its inverse from RLBWT. Their
Theorem 9 establishes

$$
\boxed{\begin{gathered}\text{BWT inversion time}=O(n),\\\text{additional working space}=O(r).\end{gathered}}\tag{8}
$$

This is a complexity theorem for inversion, not merely compressibility.

**How directly does it apply to bzip2?** The theorem starts from run-length BWT. mibz2
reconstructs the byte sequence $L$ after MTF (undoing bzip2's move-to-front coding, the stage
before the inverse BWT), so discovering the runs involves examining $n$ bytes. Given $L$ and
its counts, the runs can be enumerated in $O(n)$ time, and the capped structure built from them
in $O(r)$, giving an $O(n+r)=O(n)$ path with $O(r)$ navigation storage, when the input sequence
and output buffer are accounted for separately. This does not make the entire decompressor
$O(r)$ total memory. Linear asymptotic construction does not mean lower constant cost than
mibz2's fused counting-sort builder.

The single-cycle amortisation argument applies directly to primitive strings, those that
are not a repetition of a shorter string. Periodic cyclic
BWTs may have several cycles
([appendix section 6](2026-11-08-inverse-bwt-navigation-notes.md#6-cycle-structure-and-necklaces));
mibz2's segmented traversal needs to preserve applicable assumptions. The balanced 2021
structure gives a per-move guarantee independent of single-cycle amortisation, at additional
construction complexity. Brown and Langmead's [Orbit library](https://github.com/drnatebrown/orbit)
implements move-structure variants, but is not a verified drop-in replacement for mibz2's Rust
T-vector.

So there exists a mathematically established family of navigation structures whose storage
scales with BWT runs rather than rows, and that supports efficient repeated successor traversal
without a fresh general predecessor search on each step.

## 6. What would it cost mibz2, and what would settle it?

A smaller table helps mibz2 only through a chain of costs. Fewer bytes mean less of the cache
taken by each block's walk. But each step then reads one or more descriptors instead of one
entry, and the builder must find the runs and write the descriptors before the walk begins.
What the 32 lanes need is many independent reads in flight at once, and a step that reads two
descriptors in turn can have two waits on its path, not one. Three resources stay separate:

$$
\boxed{\begin{gathered}\text{navigation storage},\\\text{construction work},\\\text{critical-path memory dependencies}.\end{gathered}}
$$

A representation may improve one and worsen another. The mathematical space/time bounds do not
equate them. The representations the theory offers, and what each costs, are compared in
[appendix section 9](2026-11-08-inverse-bwt-navigation-notes.md#9-the-design-space-and-the-results).

The same intervals bear on mibz2's other work. In compact30, every 30-row group lying inside
one interval is guaranteed an ordinary record, and these groups can be counted from the runs
without building the table
([appendix section 4](2026-11-08-inverse-bwt-navigation-notes.md#4-what-the-intervals-say-about-compact30)).
Taking $k$ steps at once needs at most $1+k(r-1)$ intervals
([appendix section 5](2026-11-08-inverse-bwt-navigation-notes.md#5-several-steps-at-once)).

The central research question changes from *how much can we compress the existing navigation
table?* to:

**What is the least expensive stateful navigation representation for a BWT interval-exchange
permutation, while preserving high memory-level parallelism across simultaneous traversals?**

Two measurements would settle whether it is worth building. The first is the run count of each
block on mibz2's corpora, and so the size of the descriptors, block by block, against the
T-vector's. The second, only if those are small, is a 32-lane walk over a move structure, timed
against the T-vector walk with the cost of building each counted. Neither has been done. The
most consequential result to investigate further is the move structure, especially Brown and
Langmead's 2026 length-capping construction. It is the closest existing theoretical answer to
the problem mibz2 solves.

## 7. The argument in brief

The next-row map sends the $k$th copy of each byte in $F$ to its $k$th copy in $L$, so each
run of $L$ is one shifted interval, and the map is at most $r$ of them. A walk that carries its
interval needs no search to take its next step, and move structures make that step take
constant time in $O(r)$ words. Brown and Langmead turn this into a linear-time inversion with
run-bounded working space.

**What is proved:** the inverse-BWT navigation problem supports representations and algorithms
with run-bounded space, efficient repeated traversal, and (in the RLBWT model) asymptotically
optimal inversion time.

**What is not proved for mibz2:** that these beat its 32-lane, four-byte-per-row implementation
in elapsed CPU cycles once table construction, segmented starts, emitted characters and
shared-cache behaviour are counted. The permutree thesis identifies a legitimate algebraic
neighbourhood, but the directly applicable algorithmic theorems come from succinct data
structures and run-length BWT
([appendix section 7](2026-11-08-inverse-bwt-navigation-notes.md#7-what-does-the-permutree-thesis-contribute)).

The general lesson is that a table walked in order can often be replaced by less data plus
state carried by the walker, since each step already knows where the last one was.

---

**Sources:**

- [Technical appendix](2026-11-08-inverse-bwt-navigation-notes.md): the secondary derivations,
  bounds and checks, with their equations.
- [Daniel Tamayo Jiménez, *Combinatorics of Permutreehedra and Geometry of s-Permutahedra*](https://arxiv.org/abs/2310.19732),
  PhD thesis (2023): weak order and parabolic quotients.
- [Takaaki Nishimoto and Yasuo Tabei, *Optimal-Time Queries on BWT-runs Compressed Indexes*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2021.101),
  ICALP 2021, LIPIcs 198, 101:1–101:15: constant-time move structures.
- [Nathaniel K. Brown and Ben Langmead, *Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations*](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html),
  SEA 2026, LIPIcs 371, paper 9: length capping, and inversion in $O(n)$ time and $O(r)$ space.
- [Hideo Bannai, Dominik Köppl and Zsuzsanna Lipták, *A Survey of the Bijective Burrows–Wheeler Transform*](https://drops.dagstuhl.de/storage/01oasics/oasics-vol131-manzinis-festschrift/html/OASIcs.Manzini.2/OASIcs.Manzini.2.html)
  (2025), in the Festschrift for Giovanni Manzini, OASIcs 131: cyclic BWT and necklace enumeration.
- [Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, *Slashing the Time for BWT Inversion*](https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/),
  DCC 2012, pp. 99–108: multi-chain and multi-character inversion.
- [*Building a smaller bzip2 decoding table: from 49 instructions to 12*](2026-11-04-compact30-constructor.md)
  (4 November 2026): compact30 architecture and builder evidence.
