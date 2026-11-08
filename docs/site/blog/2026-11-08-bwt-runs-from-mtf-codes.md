---
title: "Reading the BWT's runs off bzip2's move-to-front codes"
image: assets/social/blog-2026-11-08-bwt-runs-from-mtf-codes.png
description: "bzip2's move-to-front codes mark where each run of the BWT's last column starts, so the runs could be listed without writing the column. It is not built."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Reading the BWT's runs off bzip2's move-to-front codes

bzip2's move-to-front codes mark where each run of the BWT's last column starts, so the runs
could be listed without writing the column. It is not built.
{: .post-dek }

8 November 2026 · 11 min read · [Open question](#6-where-does-this-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Before a bzip2 decoder can undo the Burrows–Wheeler transform (BWT), it has to rebuild the
transform's output: a column of bytes, $L$, one byte for each of the block's $n$ rows, up to
900,000 of them. bzip2 stores that column in coded form. Its *move-to-front* (MTF) stage
writes each byte as a small number, its position in a list of recently used bytes, and it
writes repeats of the last byte as a separate count. mibz2, our bzip2 decoder, turns those
codes back into all $n$ bytes of $L$, then reads $L$ again to build its table for the inverse
BWT, the T-vector, at four bytes a row.

[The previous post](2026-11-08-inverse-bwt-navigation.md) showed that the inverse BWT's
next-row map needs only one record per *run* of $L$, a stretch of one repeated byte, rather
than one entry per row. Published structures built from those records, *move structures*,
step from row to row without a search. To build one, a decoder needs the runs.

**This post's observation is that bzip2's codes already list the runs.** A code that names a
byte other than the last one starts a run; a repeat count extends it. Nothing else starts or
ends a run. So the runs, and from them a move structure, could be built from the codes
directly, without writing the $n$-byte column and scanning it again. In theory, building them
would take time and memory that grow with the number of codes and runs rather than with $n$.
The inversion that follows still takes time proportional to $n$, as any inversion must, and
that bound is already published.

That observation is not new to mibz2: it was recorded in
[issue #42](https://github.com/lmmx/mibz2/issues/42#issuecomment-6451443792) in October.
The same issue then built and timed a move structure for mibz2's walk, and it lost by 90–108%,
a loss that a run list handed over for free would not have changed (section 5). That
experiment did not time a builder fused with MTF. What this post
adds is the mechanism worked through, its edge cases, what it would cost in theory, and a
search for prior work. **Nothing here has been built or timed, and it changes nothing in the
decoder.** The costs in section 4 are our own, unproven. This is a draft for a researcher's
review, which is to follow.

[Section 1](#1-how-does-bzip2-write-banana) decodes the codes of `banana` by hand.
[Section 2](#2-why-does-every-literal-start-a-run) proves that the codes give every run, and
section 3 turns the runs into a move structure. [Section 4](#4-what-would-it-cost-in-theory)
gives the cost in theory and [section 5](#5-what-has-mibz2-measured) what mibz2 has measured.
[Section 6](#6-where-does-this-stand) says what is open.

<figure class="bf-fig">
<p class="bf-title">Two routes from bzip2's symbols to a move structure</p>
<div class="bf-frames bf-frames-pair">
<div class="bf-frame">
<svg viewBox="0 0 196 266" role="img" aria-label="Today's route: symbols, then the last column L written as n bytes, then the runs found by scanning L, then a move structure built from the runs."><text x="8" y="14" class="bf-cap">today</text><rect x="8" y="24" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="43" class="bf-t-strong" text-anchor="middle">Symbols</text><text x="98" y="59" class="bf-tm" text-anchor="middle">decoded from Huffman codes</text><path d="M98 68V82" class="bf-quiet-ink"/><path d="M95.3 82.1L98.0 88.0L100.7 82.1Z" class="bf-head-ink"/><rect x="8" y="88" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="107" class="bf-t-strong" text-anchor="middle">The last column, L</text><text x="98" y="123" class="bf-tm" text-anchor="middle">n bytes written</text><path d="M98 132V146" class="bf-quiet-ink"/><path d="M95.3 146.1L98.0 152.0L100.7 146.1Z" class="bf-head-ink"/><rect x="8" y="152" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="171" class="bf-t-strong" text-anchor="middle">The runs of L</text><text x="98" y="187" class="bf-tm" text-anchor="middle">found by scanning L</text><path d="M98 196V210" class="bf-quiet-ink"/><path d="M95.3 210.1L98.0 216.0L100.7 210.1Z" class="bf-head-ink"/><rect x="8" y="216" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="235" class="bf-t-strong" text-anchor="middle">A move structure</text><text x="98" y="251" class="bf-tm" text-anchor="middle">built from the runs</text></svg>
<p class="bf-step"><strong>1.</strong> A decoder that wants a move structure would write all <em>n</em> bytes of <em>L</em>, then scan them to find the runs.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 196 266" role="img" aria-label="The proposed route: symbols, then the runs of L read off the symbols, then a move structure. L is never written."><text x="8" y="14" class="bf-cap">proposed</text><rect x="8" y="24" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="43" class="bf-t-strong" text-anchor="middle">Symbols</text><text x="98" y="59" class="bf-tm" text-anchor="middle">decoded from Huffman codes</text><path d="M98 68V82" class="bf-quiet-ink"/><path d="M95.3 82.1L98.0 88.0L100.7 82.1Z" class="bf-head-ink"/><rect x="8" y="88" width="180" height="44" rx="10" class="bf-card-hi"/><text x="98" y="107" class="bf-t-strong" text-anchor="middle">The runs of L</text><text x="98" y="123" class="bf-tm" text-anchor="middle">read off the symbols</text><path d="M98 132V146" class="bf-quiet-ink"/><path d="M95.3 146.1L98.0 152.0L100.7 146.1Z" class="bf-head-ink"/><rect x="8" y="152" width="180" height="44" rx="10" class="bf-card"/><text x="98" y="171" class="bf-t-strong" text-anchor="middle">A move structure</text><text x="98" y="187" class="bf-tm" text-anchor="middle">built from the runs</text></svg>
<p class="bf-step"><strong>2.</strong> The symbols already mark where each run starts, so the runs could be listed directly and <em>L</em> never written.</p>
</div>
</div>
<figcaption>Conceptual. mibz2 builds its T-vector from <em>L</em>, not a move structure; neither route to a move structure is in the decoder, and the proposed one has not been built or timed.</figcaption>
</figure>

## 1. How does bzip2 write `banana`?

Suppose you compress the word `banana` with bzip2. The BWT sorts the word's six rotations and
keeps their last letters, $L=\texttt{nnbaaa}$, and the number of the row that holds the
word itself
([the earlier post works it through](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md)).
The MTF stage then codes $L$ one byte at a time.

It starts from the list of the bytes the block uses, in byte order: `a b n`. To write a byte,
it writes the byte's position in the list, counting from 0, and moves the byte to the front.
The first byte, `n`, is at position 2, so bzip2 writes 2, and the list becomes `n a b`. The
next `n` is now at the front, position 0.

Position 0 means "the same byte again", and bzip2 never writes it as a number. It counts a
stretch of zeros and writes the count in two symbols, `RUNA` and `RUNB`. They are digits worth
1 and 2 in the first place, 2 and 4 in the second, 4 and 8 in the third, and so on, added up:
`RUNA` is one zero, `RUNB` two, `RUNA RUNA` three. The other symbols, the numbers 1 and up,
we call *literals*. (In the file, every symbol is stored one higher, to make room for
`RUNA` and `RUNB` at 0 and 1.)

Coding all of `nnbaaa` this way gives five symbols, followed by an end-of-block symbol:

| Byte of $L$ | List before | Symbol | List after |
| --- | --- | --- | --- |
| `n` | `a b n` | 2 | `n a b` |
| `n` | `n a b` | `RUNA` (one zero) | `n a b` |
| `b` | `n a b` | 2 | `b n a` |
| `a`, `a`, `a` | `b n a` | 2, then `RUNB` (two zeros) | `a b n` |

The decoder does the same in reverse: a literal $j$ takes the byte at position $j$, writes
it, and moves it to the front; a count of zeros writes that many copies of the front byte.
[mibz2's scalar loop](https://github.com/lmmx/mibz2/blob/68d934b/src/mtf.rs#L32-L74) is about
forty lines long, and it writes every byte into $L$ and counts each byte value as it goes.

<figure class="bf-fig">
<p class="bf-title">From <code>banana</code>'s symbols to its runs, and to the next-row map</p>
<div class="bf-frames bf-frames-3">
<div class="bf-frame">
<svg viewBox="0 0 196 180" role="img" aria-label="The symbols 2, RUNA, 2, 2, RUNB. Each literal 2, in green, writes one new byte of L: n at 0, b at 2, a at 3. RUNA repeats n once, at 1; RUNB repeats a twice, at 4 and 5."><text x="4" y="14" class="bf-cap">symbols</text><rect x="5" y="24" width="34" height="26" rx="8" class="bf-pill-hi"/><text x="22" y="41" class="bf-t-strong" text-anchor="middle">2</text><rect x="43" y="24" width="34" height="26" rx="8" class="bf-pill"/><text x="60" y="41" class="bf-tm" text-anchor="middle">RUNA</text><rect x="81" y="24" width="34" height="26" rx="8" class="bf-pill-hi"/><text x="98" y="41" class="bf-t-strong" text-anchor="middle">2</text><rect x="119" y="24" width="34" height="26" rx="8" class="bf-pill-hi"/><text x="136" y="41" class="bf-t-strong" text-anchor="middle">2</text><rect x="157" y="24" width="34" height="26" rx="8" class="bf-pill"/><text x="174" y="41" class="bf-tm" text-anchor="middle">RUNB</text><text x="4" y="122" class="bf-cap">last column L</text><path d="M22 54C22 91.0 33 91.0 33 128" class="bf-act"/><path d="M30.3 128.1L33.0 134.0L35.7 128.1Z" class="bf-head"/><path d="M60 54C60 91.0 59 91.0 59 128" class="bf-quiet"/><path d="M56.3 128.1L59.0 134.0L61.7 128.1Z" class="bf-head-q"/><path d="M98 54C98 91.0 85 91.0 85 128" class="bf-act"/><path d="M82.3 128.1L85.0 134.0L87.7 128.1Z" class="bf-head"/><path d="M136 54C136 91.0 111 91.0 111 128" class="bf-act"/><path d="M108.3 128.1L111.0 134.0L113.7 128.1Z" class="bf-head"/><path d="M174 54C174 91.0 137 91.0 137 128" class="bf-quiet"/><path d="M134.3 128.1L137.0 134.0L139.7 128.1Z" class="bf-head-q"/><path d="M174 54C174 91.0 163 91.0 163 128" class="bf-quiet"/><path d="M160.3 128.1L163.0 134.0L165.7 128.1Z" class="bf-head-q"/><text x="33" y="154" class="bf-ch" text-anchor="middle">n</text><text x="33" y="174" class="bf-ix" text-anchor="middle">0</text><text x="59" y="154" class="bf-ch-dim" text-anchor="middle">n</text><text x="59" y="174" class="bf-ix" text-anchor="middle">1</text><text x="85" y="154" class="bf-ch" text-anchor="middle">b</text><text x="85" y="174" class="bf-ix" text-anchor="middle">2</text><text x="111" y="154" class="bf-ch" text-anchor="middle">a</text><text x="111" y="174" class="bf-ix" text-anchor="middle">3</text><text x="137" y="154" class="bf-ch-dim" text-anchor="middle">a</text><text x="137" y="174" class="bf-ix" text-anchor="middle">4</text><text x="163" y="154" class="bf-ch-dim" text-anchor="middle">a</text><text x="163" y="174" class="bf-ix" text-anchor="middle">5</text></svg>
<p class="bf-step"><strong>1.</strong> Each green literal writes a byte that differs from the one before, so it starts a run. <code>RUNA</code> and <code>RUNB</code> repeat the last byte: one copy and two.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 196 196" role="img" aria-label="L, n n b a a a, split into its runs: n times 2, b times 1, a times 3. Below, the run records: byte n, start 0, length 2, k 0; byte b, start 2, length 1, k 0; byte a, start 3, length 3, k 0."><text x="33" y="18" class="bf-ix" text-anchor="middle">0</text><text x="33" y="42" class="bf-ch" text-anchor="middle">n</text><text x="59" y="18" class="bf-ix" text-anchor="middle">1</text><text x="59" y="42" class="bf-ch" text-anchor="middle">n</text><text x="85" y="18" class="bf-ix" text-anchor="middle">2</text><text x="85" y="42" class="bf-ch" text-anchor="middle">b</text><text x="111" y="18" class="bf-ix" text-anchor="middle">3</text><text x="111" y="42" class="bf-ch" text-anchor="middle">a</text><text x="137" y="18" class="bf-ix" text-anchor="middle">4</text><text x="137" y="42" class="bf-ch" text-anchor="middle">a</text><text x="163" y="18" class="bf-ix" text-anchor="middle">5</text><text x="163" y="42" class="bf-ch" text-anchor="middle">a</text><path d="M23 52V56H69V52" class="bf-seg-on"/><text x="46" y="72" class="bf-segl-on" text-anchor="middle">n × 2</text><path d="M75 52V56H95V52" class="bf-seg-on"/><text x="85" y="72" class="bf-segl-on" text-anchor="middle">b × 1</text><path d="M101 52V56H173V52" class="bf-seg-on"/><text x="137" y="72" class="bf-segl-on" text-anchor="middle">a × 3</text><path d="M4 86H192" class="bf-rule"/><text x="4" y="104" class="bf-cap">run records</text><text x="26" y="124" class="bf-ix" text-anchor="middle">byte</text><text x="74" y="124" class="bf-ix" text-anchor="middle">start</text><text x="122" y="124" class="bf-ix" text-anchor="middle">length</text><text x="170" y="124" class="bf-ix" text-anchor="middle">k</text><text x="26" y="146" class="bf-eq-l" text-anchor="middle">n</text><text x="74" y="146" class="bf-eq-l" text-anchor="middle">0</text><text x="122" y="146" class="bf-eq-l" text-anchor="middle">2</text><text x="170" y="146" class="bf-eq-l" text-anchor="middle">0</text><text x="26" y="168" class="bf-eq-l" text-anchor="middle">b</text><text x="74" y="168" class="bf-eq-l" text-anchor="middle">2</text><text x="122" y="168" class="bf-eq-l" text-anchor="middle">1</text><text x="170" y="168" class="bf-eq-l" text-anchor="middle">0</text><text x="26" y="190" class="bf-eq-l" text-anchor="middle">a</text><text x="74" y="190" class="bf-eq-l" text-anchor="middle">3</text><text x="122" y="190" class="bf-eq-l" text-anchor="middle">3</text><text x="170" y="190" class="bf-eq-l" text-anchor="middle">0</text></svg>
<p class="bf-step"><strong>2.</strong> So the symbols give one record per run: its byte, where it starts, its length, and <em>k</em>, the copies of that byte before it.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 196 178" role="img" aria-label="F, a a a b n n, above L. A green ribbon takes F rows 0 to 2 to L positions 3 to 5, the run a times 3. Grey curves take row 3 to position 2 and rows 4 and 5 to positions 0 and 1."><text x="6" y="46" class="bf-lab">F</text><text x="6" y="146" class="bf-lab">L</text><rect x="28" y="27" width="76" height="26" rx="13" class="bf-tint"/><rect x="106" y="127" width="76" height="26" rx="13" class="bf-tint"/><text x="40" y="18" class="bf-ix" text-anchor="middle">0</text><text x="40" y="46" class="bf-ch" text-anchor="middle">a</text><text x="40" y="146" class="bf-ch-dim" text-anchor="middle">n</text><text x="40" y="170" class="bf-ix" text-anchor="middle">0</text><text x="66" y="18" class="bf-ix" text-anchor="middle">1</text><text x="66" y="46" class="bf-ch" text-anchor="middle">a</text><text x="66" y="146" class="bf-ch-dim" text-anchor="middle">n</text><text x="66" y="170" class="bf-ix" text-anchor="middle">1</text><text x="92" y="18" class="bf-ix" text-anchor="middle">2</text><text x="92" y="46" class="bf-ch" text-anchor="middle">a</text><text x="92" y="146" class="bf-ch-dim" text-anchor="middle">b</text><text x="92" y="170" class="bf-ix" text-anchor="middle">2</text><text x="118" y="18" class="bf-ix" text-anchor="middle">3</text><text x="118" y="46" class="bf-ch-dim" text-anchor="middle">b</text><text x="118" y="146" class="bf-ch" text-anchor="middle">a</text><text x="118" y="170" class="bf-ix" text-anchor="middle">3</text><text x="144" y="18" class="bf-ix" text-anchor="middle">4</text><text x="144" y="46" class="bf-ch-dim" text-anchor="middle">n</text><text x="144" y="146" class="bf-ch" text-anchor="middle">a</text><text x="144" y="170" class="bf-ix" text-anchor="middle">4</text><text x="170" y="18" class="bf-ix" text-anchor="middle">5</text><text x="170" y="46" class="bf-ch-dim" text-anchor="middle">n</text><text x="170" y="146" class="bf-ch" text-anchor="middle">a</text><text x="170" y="170" class="bf-ix" text-anchor="middle">5</text><path d="M118 60C118 87.0 92 87.0 92 114" class="bf-quiet"/><path d="M89.3 114.1L92.0 120.0L94.7 114.1Z" class="bf-head-q"/><path d="M144 60C144 87.0 40 87.0 40 114" class="bf-quiet"/><path d="M37.3 114.1L40.0 120.0L42.7 114.1Z" class="bf-head-q"/><path d="M170 60C170 87.0 66 87.0 66 114" class="bf-quiet"/><path d="M63.3 114.1L66.0 120.0L68.7 114.1Z" class="bf-head-q"/><path d="M29 58C29 98.0 107 82.0 107 118H181C181 82.0 103 98.0 103 58Z" class="bf-ribbon"/><text x="107" y="95" class="bf-eq bf-halo" text-anchor="middle">0 → 3</text></svg>
<p class="bf-step"><strong>3.</strong> At the block's end the counts fix where each byte's rows begin: <code>a</code> at 0. The record <code>a</code>, start 3, <em>k</em> 0 becomes rows 0–2 → positions 3–5.</p>
</div>
</div>
<figcaption>Conceptual, worked by hand and checked by a script (see the PR). Green marks the literal symbols in the first frame and the run <code>aaa</code> in the last. The block-end symbol, which follows <code>RUNB</code>, is left out.</figcaption>
</figure>

## 2. Why does every literal start a run?

Because the front of the list is always the byte just written. A literal names a position of
1 or more, and every byte in the list is different, so a literal always writes a byte that
differs from the one before it. A count of zeros always repeats it. So a run of $L$ is exactly
one literal followed by at most one count, and the run's length is one plus the count.

That makes the number of runs, $r$, a count of the literals, with one exception, which comes
from how the list starts. Before the first byte is written, the front of the list is the
block's smallest byte, which hasn't been written yet. If $L$ starts with that byte, bzip2
writes it as position 0, so the block's first run starts with a count, not a literal. The
column of `aaaa` is `aaaa`, and bzip2 codes it as `RUNB RUNA` (two, plus two in the second
place): one run, no literal at all. So

$$
r=(\text{number of literals})+[\,L\text{ starts with the block's smallest byte}\,].
$$

The other edge cases are smaller. A count is never zero: each `RUNA` or `RUNB` adds at least
one, so a count that is present extends its run. The block's last run ends at the end-of-block
symbol, so a decoder that lists runs must close the open run there; mibz2's loop already
writes a pending count before it checks for the end. And a list of runs needn't be maximal to
be correct. A decoder that cuts a run in two where it crosses a checkpoint, as mibz2's
checkpointed MTF loop does, would give two records with the same byte, which still describe $L$
exactly. It only costs a record.

We checked the argument with a short script that models bzip2's MTF and `RUNA`/`RUNB` rules
(not mibz2's code). On 20,009 strings, random ones and periodic ones among them, it listed the
runs from the symbols, compared them with the runs of the column computed directly, and
undid the transform from the runs alone. Every case matched, the first-run exception
included.

## 3. How do the runs become a move structure?

From each run, the decoder needs four numbers: its byte $c$, its start $p$ in $L$, its length
$\ell$, and $k$, the number of copies of $c$ written before it. The MTF decoder already counts
each byte value as it writes, so $k$ is that count at the run's start. In `banana` every run is
its byte's first, so every $k$ is 0. In the column `baba`, of the word `abba`, the second
`b`-run has $k=1$.

The previous post showed which rows a run takes over
([its section 2](2026-11-08-inverse-bwt-navigation.md#2-why-does-every-run-give-one-shift)).
Write $C(c)$ for the number of bytes in the block smaller than $c$: the row where $c$'s rows
begin in $F$, the sorted first column. Then the run's rows begin at $a=C(c)+k$, and the
next-row map $\Psi$ sends each of them to the matching position of the run:

$$
\Psi(a+t)=p+t,\qquad 0\le t<\ell.
$$

There is one wait. $C(c)$ depends on the counts of every smaller byte, so it is known only at
the end of the block. The records can be written while decoding, with $k$, and given their
rows once the counts are final. In `banana` the counts are `a` 3, `b` 1, `n` 2, so the `a`
rows begin at 0, `b` at 3 and `n` at 4: the record (`a`, start 3, length 3, $k$ 0) becomes
rows 0–2 to positions 3–5, as in the last frame above.

A move structure also needs each interval's *anchor*, the interval its destination falls in,
so that a walk can carry its interval along with its row
([the previous post's section 1.1](2026-11-08-inverse-bwt-navigation.md#11-walking-the-intervals-without-a-search)).
The records arrive in $L$ order, so their destinations rise. Sorted by byte, a counting sort
over the $r$ records, their row starts rise too. One merge of the two lists finds every
anchor. Brown and Langmead build a *length-capped* structure, which splits long intervals so
that the walk's average step is bounded, from exactly such a run list in $O(r)$ time and space
([SEA 2026, Lemma 6](https://arxiv.org/abs/2602.11029)).

<figure class="bf-fig">
<p class="bf-title">Three run records, three intervals of a move structure</p>
<div class="bf-frames bf-frames-pair">
<div class="bf-frame">
<svg viewBox="0 0 196 178" role="img" aria-label="Rows 0 to 5 in F order, a a a b n n, cut into three intervals: A, rows 0 to 2, destination 3; B, row 3, destination 2; N, rows 4 and 5, destination 0. The counts a 3, b 1, n 2 give first rows a 0, b 3, n 4."><text x="6" y="16" class="bf-cap">rows, in F order</text><text x="6" y="64" class="bf-lab">F</text><text x="40" y="64" class="bf-ch" text-anchor="middle">a</text><text x="40" y="40" class="bf-ix" text-anchor="middle">0</text><text x="66" y="64" class="bf-ch" text-anchor="middle">a</text><text x="66" y="40" class="bf-ix" text-anchor="middle">1</text><text x="92" y="64" class="bf-ch" text-anchor="middle">a</text><text x="92" y="40" class="bf-ix" text-anchor="middle">2</text><text x="118" y="64" class="bf-ch-dim" text-anchor="middle">b</text><text x="118" y="40" class="bf-ix" text-anchor="middle">3</text><text x="144" y="64" class="bf-ch-dim" text-anchor="middle">n</text><text x="144" y="40" class="bf-ix" text-anchor="middle">4</text><text x="170" y="64" class="bf-ch-dim" text-anchor="middle">n</text><text x="170" y="40" class="bf-ix" text-anchor="middle">5</text><path d="M30 74V78H102V74" class="bf-seg-a-on"/><text x="66" y="96" class="bf-segl-on" text-anchor="middle">A</text><text x="66" y="116" class="bf-eq" text-anchor="middle">→ 3</text><path d="M108 74V78H128V74" class="bf-seg-on"/><text x="118" y="96" class="bf-segl-on" text-anchor="middle">B</text><text x="118" y="116" class="bf-eq" text-anchor="middle">→ 2</text><path d="M134 74V78H180V74" class="bf-seg-on"/><text x="157" y="96" class="bf-segl-on" text-anchor="middle">N</text><text x="157" y="116" class="bf-eq" text-anchor="middle">→ 0</text><text x="6" y="148" class="bf-ix">counts a 3 · b 1 · n 2</text><text x="6" y="168" class="bf-ix">first rows a 0 · b 3 · n 4</text></svg>
<p class="bf-step"><strong>1.</strong> Sorting the records by byte, a counting sort over <em>r</em> records, puts them in <em>F</em> order. Each starts at its byte's first row plus <em>k</em>.</p>
</div>
<div class="bf-frame">
<svg viewBox="0 0 196 186" role="img" aria-label="The move structure's table. A: rows 0 to 2, to 3, anchor B. B: row 3, to 2, anchor A. N: rows 4 to 5, to 0, anchor A. The anchor is the interval holding the destination row."><text x="6" y="16" class="bf-cap">move structure</text><text x="20" y="40" class="bf-ix" text-anchor="middle"></text><text x="66" y="40" class="bf-ix" text-anchor="middle">rows</text><text x="116" y="40" class="bf-ix" text-anchor="middle">to</text><text x="168" y="40" class="bf-ix" text-anchor="middle">anchor</text><text x="20" y="66" class="bf-segl-on" text-anchor="middle">A</text><text x="66" y="66" class="bf-eq-l" text-anchor="middle">0–2</text><text x="116" y="66" class="bf-eq-l" text-anchor="middle">3</text><text x="168" y="66" class="bf-eq-l" text-anchor="middle">B</text><text x="20" y="94" class="bf-segl-on" text-anchor="middle">B</text><text x="66" y="94" class="bf-eq-l" text-anchor="middle">3</text><text x="116" y="94" class="bf-eq-l" text-anchor="middle">2</text><text x="168" y="94" class="bf-eq-l" text-anchor="middle">A</text><text x="20" y="122" class="bf-segl-on" text-anchor="middle">N</text><text x="66" y="122" class="bf-eq-l" text-anchor="middle">4–5</text><text x="116" y="122" class="bf-eq-l" text-anchor="middle">0</text><text x="168" y="122" class="bf-eq-l" text-anchor="middle">A</text><path d="M6 140H190" class="bf-rule"/><text x="6" y="162" class="bf-ix">anchor: the interval holding</text><text x="6" y="178" class="bf-ix">the destination row</text></svg>
<p class="bf-step"><strong>2.</strong> Each interval's anchor is the interval its destination falls in. The destinations, read in <em>L</em> order, rise, so one merge with the interval starts finds every anchor.</p>
</div>
</div>
<figcaption>Conceptual: the table the earlier post's walk reads, built here from the records alone. A real block would also split its long intervals (length capping), which this example doesn't need.</figcaption>
</figure>

## 4. What would it cost, in theory?

The opportunity is concrete. While undoing MTF and the `RUNA`/`RUNB` counts, a decoder could
keep one record per maximal run, merging a literal with the count after it (and any pieces of
one run it was made to cut), together with a count per byte value. At the block's end it
computes each $C(c)$, gives each record its row start $a=C(c)+k$, and builds a run-bounded move
structure from the records. The $n$-byte column is never written or scanned. What follows is
our own candidate accounting of what that would cost. It is not a published result, and the
review is to check it.

Write $m$ for the number of symbols in a block and $\sigma$ for the number of byte values it
uses, at most 256. On a word RAM, the usual model in which one machine word is read or written
in one step, with each symbol handled in constant time:

1. **Building the records** would take $O(m+r+\sigma)$ time: one pass over the symbols, a
   counting sort of the $r$ records by byte, and the counts. It would hold $O(r+\sigma)$ words
   of working memory: the records and a count per byte value.
2. **Composed with Brown and Langmead,** whose preparation takes $O(r)$ time and space and whose
   inversion takes $O(n)$ time from a run-length BWT
   ([SEA 2026, Lemma 6 and Theorem 9](https://arxiv.org/abs/2602.11029)), the inverse BWT would
   take $O(m+n+r+\sigma)$ time and $O(r+\sigma)$ extra working memory.

That memory figure leaves out the compressed input, the $n$ output bytes, the block's Huffman
tables and other metadata, and the decoder's other buffers. It is not a memory bound for a
whole bzip2 decompressor, which also undoes bzip2's first stage, a run-length code applied
before the BWT, and checks a CRC. And the time is still $O(n)$, because $n$ bytes must be
written. **Brown and Langmead already invert in $O(n)$ time with $O(r)$ extra space,** so the
composition is no new inversion bound. What it would change is the route: no $n$-byte column
between MTF and the move structure.

Each of those statements has conditions:

1. **A fixed alphabet.** Moving a byte to the front of the list costs up to $\sigma$ steps. For
   bytes that is a constant, and mibz2 does most moves with a few vector shuffles; for a large
   alphabet it would not be.
2. **The symbols streamed.** mibz2 decodes a block's Huffman codes into an array of symbols
   first, which is $O(m)$ memory, before MTF reads it
   ([`Symbols`](https://github.com/lmmx/mibz2/blob/68d934b/src/block.rs#L23-L34)). The
   $O(r+\sigma)$ figure needs Huffman decoding and MTF fused, or the array counted.
3. **No end marker.** Brown and Langmead's text ends in a unique terminator, `$`. bzip2's BWT
   sorts rotations with no terminator and stores the start row instead. A periodic block's
   map then has several cycles
   ([the appendix, section 6](2026-11-08-inverse-bwt-navigation-notes.md#6-cycle-structure-and-necklaces)),
   and their bound on the average step assumes one. This needs checking before their theorems
   are applied to bzip2.

**A sharper bound on building the records, still to be proved.** A run of length $\ell$ costs
one literal and about $\log_2\ell$ count digits, since each digit doubles the place value.
Summed over the runs, and since the logarithm grows ever more slowly, the total would be largest
when all runs have the same length, which suggests

$$
m\le r\left(1+\log_2\frac nr\right)+1,
$$

with the $+1$ for the end-of-block symbol. The script of section 2 found no case that breaks
it, but this is a sketch, not a checked proof: bzip2's exact symbol grammar, the first and last
runs, and merged runs all have to be accounted for. If it holds, building the records takes
time sublinear in $n$ on blocks with long runs. That is not sublinear inversion, which writes
$n$ bytes regardless, and it may not be new: it follows from the input having coded the runs
already.

## 5. What has mibz2 measured?

**How small is $r$?** It has been counted once. In October, an uncommitted scratch counter ran
over the bench's Wikidata corpus, `wikidata.bz2` (SHA-256 `82f65ace…`), and checked its map
against the decoder's on every block. The 233 blocks of at least 65,536 rows hold 209.7
million rows, 99.996% of the corpus, and 15,148,086 runs: $r/n=0.0722$, about 14 rows a run.
Block by block, $r/n$ ran from 0.053 to 0.200, with a median of 0.062. 71% of the rows lie in
runs of 256 or more
([the journal](https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md),
[issue #42](https://github.com/lmmx/mibz2/issues/42)). That was one local run, on one corpus.
The previous post said $r$ had not been counted. It had, on that corpus. Its upper bound,
$r/n\le0.28$, is on a different one, the primary Wikidata corpus `wikidata-20261026.bz2` of 434
blocks, where $r$ has still not been counted. That post now says so.

At $n/r=13.85$ the bound of section 4 gives at most about 4.8 symbols a run, or about 0.35
symbols a row. That is derived from the run count, not measured.

**Would a move structure help mibz2?** Issue #42 built one, length-capped as Brown and Langmead
describe, and timed it, locally on a shared four-core virtual machine, at one thread, on those 233 blocks, over two runs
([the write-up](https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-move-structure-walk.md),
[the result](https://github.com/lmmx/mibz2/issues/42#issuecomment-6452138621)). Walked as one
chain, it took about 60% less time than the T-vector's chain. Walked as mibz2 walks, 32 chains
interleaved so that their memory reads overlap, it lost:

| Cycles per byte, local (run 1 / run 2) | Cost |
| --- | ---: |
| T-vector: build + 32-lane walk | 12.03 / 11.97 |
| Move structure: the scan of $L$ for runs | 0.94 / 0.94 |
| Move structure: build from the runs | 6.7–8.1 |
| Move structure: the best interleaved walk alone | 13.5–17.1 |
| Move structure: build + walk, as if MTF handed over the runs | +80% to +99% against the T-vector |

The scratch runs were about twice CI's figures, so they are indicative only. **The walk alone
cost more than the T-vector's build and walk together**, because each step ran about 41
instructions against 16. So reading the runs off the codes cannot rescue that design: the row
"as if MTF handed over the runs" is that case, and it still lost. It would save the scan, under
one cycle a byte, and the writes of $L$, which nobody has timed separately. No builder fused
with MTF was timed: that measured row removes the scan, nothing more. The result is for that
one structure and that walk, on one machine and one corpus.

## 6. Where does this stand?

**Is any of it new?** Not the observation, which mibz2 recorded in issue #42. The rules it
rests on are bzip2's format, as Julian Seward designed it, and they are visible in any
decoder: [Go's](https://go.dev/src/compress/bzip2/bzip2.go) or the
[Wuffs format notes](https://github.com/google/wuffs/blob/main/std/bzip2/README.md), which
describe `RUNA` and `RUNB` as repeating "the 0th element". We searched for work that builds a
run-length BWT or a move structure from MTF- or run-coded input, and found none. That is not
evidence that none exists. The nearest work builds a run-length BWT from other inputs: from
the plain text in space bounded by $r$
([Nishimoto, Kanda and Tabei, ICALP 2022](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2022.99)),
from an LZ77 parse
([Kempa and Kociumaka, FOCS 2020](https://arxiv.org/abs/1910.10631)), and a move structure
for any permutation in $O(r)$ time
([Brown, Sanaullah, Zhang and Langmead, 2026 preprint](https://arxiv.org/abs/2603.22147)).

**What is open** is the speculative question: does bzip2's coded form give a stronger
algorithm or bound than a generic run-length BWT does? One point bears on it. The symbols and
the column determine each other exactly, so the symbols hold no information the runs lack.
Any gain would have to come from what the decoder computes cheaply on the way: avoiding the
expansion of $L$, its scan and an $n$-byte temporary, which is a shortcut in computation, not
extra information about the permutation. For example, a literal's value, for a byte already
written, is the number of distinct bytes written since that byte last appeared. Whether that
buys anything is the question. The research question beyond simply composing known results
is whether, for a carefully defined model of access to the coded input, there is a
construction guarantee, a lower bound or a streaming constraint that doesn't already follow
from known run-length BWT and move-structure results.

**How sure are we?** Fairly sure of the run-listing argument, which section 2 proves and a
script checked on a model of bzip2's rules. Sure of the costs in section 4 only under their
conditions, which need the review. Not sure at all that any of it is new, beyond what the
format already implies.

**What a review should check:** the run argument and its edge cases (the first byte, each
change of byte, the end of the block), a proof or refutation of the bound on $m$, the input
model and memory accounting of section 4, whether Brown and Langmead's results hold for a BWT
without a terminator, the prior-art search, and which result, if any, would be new.

A researcher's review of this post is to follow. Until then it is a draft argument, not a
validated result.

## 7. The argument in brief

bzip2's MTF codes write a literal exactly where a run of the BWT's last column starts, and a
count of repeats for the rest of it, so the runs can be listed from the codes, with one
exception at the block's start. Each run's byte, start, length and earlier count, with the
block's byte counts, give its interval of the next-row map, and a merge gives the anchors a
move structure needs. In our unproven accounting, the records take $O(m+r+\sigma)$ time and
$O(r+\sigma)$ words, without the $n$-byte column, and perhaps time sublinear in $n$ on long
runs; the inversion stays $O(n)$, as Brown and Langmead already have it. In mibz2's measured setting it would not help, since
the interleaved walk over a move structure already costs more than today's whole build and
walk.

The general point is that a stage's input is often already in the form a later stage wants,
and the cheapest conversion is the one never done; whether it pays depends on what the later
stage costs.

---

**Sources:**

- [Issue #42](https://github.com/lmmx/mibz2/issues/42): the run counts, the
  [observation](https://github.com/lmmx/mibz2/issues/42#issuecomment-6451443792) that MTF gives
  the runs, and the [timed move structure](https://github.com/lmmx/mibz2/issues/42#issuecomment-6452138621).
- [The run-structures journal](https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md)
  and [the move-structure walk journal](https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-move-structure-walk.md)
  (27 October 2026).
- [*The mathematical structure of inverse-BWT navigation*](2026-11-08-inverse-bwt-navigation.md)
  and its [technical appendix](2026-11-08-inverse-bwt-navigation-notes.md).
- [Nathaniel K. Brown and Ben Langmead, *Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations*](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html),
  SEA 2026, LIPIcs 371, paper 9 ([arXiv:2602.11029](https://arxiv.org/abs/2602.11029)).
- [Nathaniel K. Brown, Ahsan Sanaullah, Shaojie Zhang and Ben Langmead, *Optimal-Time Move Structure Construction*](https://arxiv.org/abs/2603.22147),
  arXiv:2603.22147 (2026).
- [Takaaki Nishimoto, Shunsuke Kanda and Yasuo Tabei, *An Optimal-Time RLBWT Construction in BWT-Runs Bounded Space*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2022.99),
  ICALP 2022, LIPIcs 229, 99:1–99:20.
- [Dominik Kempa and Tomasz Kociumaka, *Resolution of the Burrows–Wheeler Transform Conjecture*](https://arxiv.org/abs/1910.10631),
  FOCS 2020.
- bzip2's MTF and `RUNA`/`RUNB` rules: [Go's `compress/bzip2`](https://go.dev/src/compress/bzip2/bzip2.go),
  the [Wuffs bzip2 notes](https://github.com/google/wuffs/blob/main/std/bzip2/README.md) and
  [mibz2's `mtf.rs`](https://github.com/lmmx/mibz2/blob/68d934b/src/mtf.rs).
