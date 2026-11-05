---
title: "Huffman decoding in mibz2: keeping the bit buffer in a register"
image: assets/social/blog-2026-11-05-huffman-bit-buffer.png
description: "The Huffman decoder wrote its bit buffer to memory and read it back at every lookup. Kept in a register, the stage ran 13% faster on Intel and 5% on AMD."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Huffman decoding in mibz2: keeping the bit buffer in a register

The Huffman decoder wrote its bit buffer to memory and read it back at every lookup. Kept in
a register, the stage ran 13% faster on Intel and 5% on AMD.
{: .post-dek }

5 November 2026 · 16 min read · [Research result](#9-what-the-benchmark-measured)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

A decoder reads a compressed file one byte at a time, but the codes inside a bzip2 file
aren't whole bytes. They are Huffman codes: each is a run of bits of its own length, one bit
for a very common symbol and a dozen or more for a rare one, packed end to end with nothing
between them. So a decoder keeps a bit buffer, a small holding area for bits it has read from
the file but not yet decoded. In mibz2, our bzip2 decoder written in Rust, the buffer is a
single 64-bit integer. The decoder reads each code off its top end, shifts the rest up
behind it, and tops it up with more bytes from the file when it runs low.

This post is about where that integer was kept. It held the same bits and the decoder did
the same work, but the compiled code wrote the buffer to memory after every code and read it
back before the next. Even from the processor's nearest cache, a read of a value just
written waits several cycles for the write, and every code waited that long. Rearranged so
the compiler could keep the buffer in a register, the thread that reads the codes, which
mibz2 calls the scanner, ran 12.9% faster on an Intel Xeon 8370C CI runner and 5.0% faster on an AMD EPYC 9V74.

The benchmark decodes a fixed sample of the Wikidata dump: 12 slices of it, 210 MB once
decoded. On the Intel runner a one-thread decode of that sample ran 2.7% faster. The scanner
is about a tenth of the decode, so its own saving accounts for about half of that figure;
the rest is within the noise of the other stages. On the AMD runner no change in the whole
decode was detected. Some AMD cores can serve a read of an address just written without the
usual wait, which leaves less to remove there. On Intel the cut fell short of the 15–25%
predicted for it.

[Section 1](#1-what-a-bit-buffer-is) shows a bit buffer at work on a small example and as
mibz2's code. [Section 2](#2-reading-a-code-with-a-lookup-table) is how the decoder reads a
code with a lookup table, and why each code waits for the last.
[Section 3](#3-where-this-sits-in-bzip2-and-in-mibz2) places this in bzip2's format and in
mibz2's decoder. Sections [4](#4-what-the-census-found) to
[6](#6-the-change-the-state-as-a-value) are how the stored buffer was found, why the compiler
put it there, and the change. Sections [7](#7-what-was-predicted-before-any-timing) to
[9](#9-what-the-benchmark-measured) give the prediction, the correctness checks and what the
benchmark measured, run by run.

## 1. What a bit buffer is

Huffman coding gives each symbol a code of its own length: short for common symbols, long
for rare ones. The codes are chosen so that no code is the beginning of another, which is
what lets them be packed end to end with no separators. Reading from the start, the bits
seen so far can only ever be the beginning of one code.

<figure class="bf-fig">
<p class="bf-title">Reading a toy Huffman code</p>
<svg viewBox="0 0 720 218" role="img" aria-label="A toy code of four symbols: A is 0, B is 10, C is 110 and D is 111. The bits 0110100111 read as A, C, B, A, D. Each code's start is known only once the code before it has been read."><text x="0" y="18" class="bf-ts">The code</text><rect x="0" y="30" width="62" height="34" rx="6" class="bf-card-hi"/><text x="31" y="52" class="bf-t-strong" text-anchor="middle">A  0</text><rect x="72" y="30" width="72" height="34" rx="6" class="bf-box"/><text x="108" y="52" class="bf-t-strong" text-anchor="middle">B  10</text><rect x="154" y="30" width="82" height="34" rx="6" class="bf-box"/><text x="195" y="52" class="bf-t-strong" text-anchor="middle">C  110</text><rect x="246" y="30" width="82" height="34" rx="6" class="bf-box"/><text x="287" y="52" class="bf-t-strong" text-anchor="middle">D  111</text><text x="344" y="44" class="bf-tm">A is the most common symbol,</text><text x="344" y="60" class="bf-tm">so it gets the shortest code</text><text x="0" y="98" class="bf-ts">The bits, and how they read</text><rect x="0" y="110" width="46" height="36" class="bf-cell-hi"/><text x="23" y="133" class="bf-t-strong" text-anchor="middle">0</text><rect x="52" y="110" width="138" height="36" class="bf-cell"/><text x="121" y="133" class="bf-t-strong" text-anchor="middle">1  1  0</text><rect x="196" y="110" width="92" height="36" class="bf-cell"/><text x="242" y="133" class="bf-t-strong" text-anchor="middle">1  0</text><rect x="294" y="110" width="46" height="36" class="bf-cell-hi"/><text x="317" y="133" class="bf-t-strong" text-anchor="middle">0</text><rect x="346" y="110" width="138" height="36" class="bf-cell"/><text x="415" y="133" class="bf-t-strong" text-anchor="middle">1  1  1</text><text x="23" y="166" class="bf-t" text-anchor="middle">A</text><text x="121" y="166" class="bf-t" text-anchor="middle">C</text><text x="242" y="166" class="bf-t" text-anchor="middle">B</text><text x="317" y="166" class="bf-t" text-anchor="middle">A</text><text x="415" y="166" class="bf-t" text-anchor="middle">D</text><path d="M30 182c30 16 60 16 88 0" class="bf-arrow"/><path d="M126 182c34 16 80 16 112 0" class="bf-arrow"/><path d="M246 182c20 16 52 16 68 0" class="bf-arrow"/><path d="M322 182c30 16 60 16 90 0" class="bf-arrow"/><text x="500" y="124" class="bf-tm">each arrow: where the next</text><text x="500" y="140" class="bf-tm">code starts is known only</text><text x="500" y="156" class="bf-tm">once this one has been read</text></svg>
<figcaption>A made-up code, not one of bzip2's. The ten bits 0110100111 hold five symbols. Their boundaries are found only by reading the codes in order.</figcaption>
</figure>

Take a made-up alphabet of four symbols, A to D, where A is the most common: A gets `0`, B
`10`, C `110` and D `111`. The ten bits `0110100111` then read as A, C, B, A, D. Notice what
the reader had to do. It could only tell that C began at the second bit once it had read A,
and only tell where B began once it had read C. Where a code starts depends on the lengths
of all the codes before it.

A computer, though, reads the file in bytes, eight bits at a time, and these codes don't
line up with bytes. The bit buffer bridges the two. mibz2 loads the file's next bytes into a
64-bit integer, `buf`, with the first unread bit at the top (the left end, written in
binary). Two more numbers go with it, and together they are the reader's whole state,
[`Bits`](https://docs.rs/mibz2/0.2.2/mibz2/bits/struct.Bits.html):

| Field | Type | What it holds |
| --- | --- | --- |
| `pos` | `usize` | the next byte of input to load |
| `buf` | `u64` | the stream's next bits, the first of them in the most significant bit |
| `live` | `u32` | how many of `buf`'s top bits are valid |

Three operations act on it, each a line or two of Rust:

- **Peek** at the next $n$ bits without consuming them: `buf >> (64 - n)`. For the next ten
  bits that is `buf >> 54`, which moves the top ten bits to the bottom of the number.
- **Skip** $n$ bits once a code of that length has been read: `buf <<= n; live -= n;`. The
  code's bits fall off the top, and the next code moves up to take their place.
- **Refill** when the buffer runs low: load the eight bytes at `pos`, OR them into `buf`
  just below the bits still live, and advance `pos` by as many whole bytes as fitted. That
  leaves at least 56 bits live.

<figure class="bf-fig bf-bitbuf" data-bits="0110100111" data-codes="0:A 10:B 110:C 111:D">
<p class="bf-title">A bit buffer, read one code at a time</p>
<ol class="bf-bits" aria-label="The buffer's unread bits, first bit on the left"><li class="bf-bit">0</li><li class="bf-bit">1</li><li class="bf-bit">1</li><li class="bf-bit">0</li><li class="bf-bit">1</li><li class="bf-bit">0</li><li class="bf-bit">0</li><li class="bf-bit">1</li><li class="bf-bit">1</li><li class="bf-bit">1</li></ol>
<p class="bf-bits-note">The toy code's ten bits at the front of the buffer, the next unread bit on the left. Highlighted: the code at the front.</p>
<dl class="bf-buf-state" aria-live="polite" hidden><dt>Code at the front</dt><dd class="bf-buf-next">A = 0</dd><dt>Unread bits (<code>live</code>)</dt><dd class="bf-buf-live">10</dd><dt>Last step</dt><dd><code class="bf-buf-shift">nothing consumed yet</code></dd><dt>Decoded so far</dt><dd class="bf-buf-out">none yet</dd></dl>
<div class="bf-buttons" hidden><button type="button" class="bf-btn bf-read">Read the next code →</button><button type="button" class="bf-btn bf-reset">Reset</button></div>
<p class="bf-buf-static">Read in order: <code>0</code> is A, then <code>110</code> is C, <code>10</code> is B, <code>0</code> is A and <code>111</code> is D. Each read shifts the buffer left by the code's length, which brings the next code to the front.</p>
<figcaption>Each read peeks at the front, finds the code that starts there, then shifts the buffer left by its length. The dotted cells are the space the shift leaves at the end, which a refill fills with more of the file. In mibz2 the buffer is 64 bits wide, not ten.</figcaption>
</figure>

So the buffer is just an integer and a few shifts. It isn't an array or a special piece of
hardware, and the whole state is three numbers: `pos`, `buf` and `live`.


## 2. Reading a code with a lookup table

Reading a code a bit at a time, as in the toy example, would take a step for every bit. So
fast Huffman decoders, mibz2 among them, read a code with a lookup table: they peek at the
next several bits and look up which code those bits begin with. A bzip2 block is coded with
a few different codes, called code tables, rather than one ([section
3](#3-where-this-sits-in-bzip2-and-in-mibz2) shows why). For each of them, mibz2 builds a
[lookup table of 1,024
entries](https://github.com/lmmx/mibz2/blob/264ae5bb/src/block.rs#L45-L72), one 32-bit entry for
each possible value of the next ten bits. Ten bits hold most of bzip2's codes but not all,
since a code can be up to 20 bits long, so an entry says one of four things. Usually the ten
bits begin with one whole code, and the entry gives its symbol and length. Often they begin
with two whole codes, and the entry gives both symbols and their combined length. If the
code is longer than ten bits, the entry sends the decoder to a slow path that reads it a bit
at a time, as libbzip2 (the reference bzip2 library) does. And if no valid code begins with these bits, the block is
corrupt.

<figure class="bf-fig">
<p class="bf-title">The bit buffer, and one table lookup</p>
<svg viewBox="0 0 720 344" role="img" aria-label="A 64-bit buffer: its top 10 bits index a 1,024-entry table, 46 more bits are live, and 8 are not yet valid. The entry gives one or two symbols and the bits they take, a longer code or an invalid one. Shifting the buffer left by those bits makes the next index, so each lookup waits for the last."><path d="M40 30v-6H600v6" class="bf-axis"/><text x="320" y="16" class="bf-tm" text-anchor="middle">live = 56: bits loaded and not yet used</text><rect x="40" y="34" width="100" height="34" rx="4" class="bf-card-hi"/><rect x="140" y="34" width="460" height="34" class="bf-slot"/><rect x="600" y="34" width="80" height="34" rx="4" class="bf-track"/><path d="M200 34v34M280 34v34M360 34v34M440 34v34M520 34v34" class="bf-rule"/><text x="90" y="56" class="bf-t-strong" text-anchor="middle">next 10 bits</text><text x="370" y="56" class="bf-t" text-anchor="middle">46 more live bits</text><text x="640" y="56" class="bf-tm" text-anchor="middle">not valid yet</text><path d="M90 72V104m-5-5 5 5 5-5" class="bf-arrow"/><rect x="20" y="108" width="220" height="78" rx="8" class="bf-card"/><text x="34" y="134" class="bf-ts">Table lookup</text><text x="34" y="155" class="bf-tm">1,024 entries of 32 bits,</text><text x="34" y="172" class="bf-tm">one for each 10-bit prefix</text><path d="M244 147H260m-5-5 5 5-5 5" class="bf-arrow"/><rect x="266" y="108" width="434" height="78" rx="8" class="bf-card"/><text x="282" y="134" class="bf-ts">The entry says what those bits begin with</text><text x="282" y="155" class="bf-tm">one symbol, or two whose codes both fit, and the bits they take;</text><text x="282" y="172" class="bf-tm">or a code longer than 10 bits (slow path), or an invalid code</text><path d="M483 190V212m-5-5 5 5 5-5" class="bf-arrow"/><rect x="266" y="216" width="434" height="62" rx="8" class="bf-card-hi"/><text x="282" y="241" class="bf-ts">Shift buf left by those bits; take them from live</text><text x="282" y="262" class="bf-tm">the next lookup's index is the top 10 bits of the new buf</text><path d="M262 247H10V51H34m-5-5 5 5-5 5" class="bf-arrow"/><text x="20" y="302" class="bf-t">Every lookup needs the last one's shift: these steps form one chain.</text><text x="20" y="322" class="bf-tm">refill, after every four lookups: load the 8 bytes at pos below the live bits, and</text><text x="20" y="338" class="bf-tm">advance pos by whole bytes until at least 56 bits are live</text></svg>
<figcaption>Conceptual, drawn just after a refill. The divisions in the bar are bytes. A refill leaves 56 to 63 bits live; four lookups use at most 40 of them.</figcaption>
</figure>

On the Wikidata sample, about 83% of lookups decode a pair
([census](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census/serial-stages.md)).
The fast loop refills, makes four lookups, then refills again. Symbols come in groups of 50
that share a code table (section 3), and the fast loop runs while at least eight symbols of
the group remain, because four lookups can produce eight; a group's last few symbols go
through a slower loop.

One lookup is four steps:

1. take the top ten bits of `buf`;
2. load the table entry they index;
3. take the code's length from the entry;
4. shift `buf` left by that length.

Step 1 of the next lookup needs the result of step 4 of this one. However many instructions
the processor could run at once, it can't start a lookup before the last one has finished
its shift. So the time a lookup takes is the latency of this chain: the time each step waits
for the one before it. Any step added to the chain adds its latency to every lookup.

## 3. Where this sits in bzip2 and in mibz2

The bit buffer is the first thing a bzip2 decoder touches, because Huffman coding is the
last of bzip2's four compression steps, and a decoder undoes them in reverse. bzip2 splits
its input into blocks of up to 900 kB and compresses each block on its own:

<figure class="bf-fig">
<p class="bf-title">How bzip2 compresses a block, and how a decoder undoes it</p>
<svg viewBox="0 0 720 178" role="img" aria-label="Compression runs left to right: the block's bytes, then RLE1 shortens long runs, the Burrows–Wheeler transform reorders the bytes, move-to-front and RLE2 turn them into mostly small symbols, and Huffman coding writes the symbols as bit codes. Decoding runs the same steps right to left, starting with the Huffman codes."><rect x="0" y="20" width="104" height="70" rx="8" class="bf-card"/><text x="52" y="48" class="bf-t-strong" text-anchor="middle">Bytes</text><text x="52" y="68" class="bf-tm" text-anchor="middle">the block,</text><text x="52" y="82" class="bf-tm" text-anchor="middle">≤ 900 kB</text><path d="M106 55h14m-5-5 5 5-5 5" class="bf-arrow"/><rect x="124" y="20" width="126" height="70" rx="8" class="bf-box"/><text x="187" y="48" class="bf-t-strong" text-anchor="middle">RLE1</text><text x="187" y="68" class="bf-tm" text-anchor="middle">long runs of a</text><text x="187" y="82" class="bf-tm" text-anchor="middle">byte shortened</text><path d="M252 55h14m-5-5 5 5-5 5" class="bf-arrow"/><rect x="270" y="20" width="126" height="70" rx="8" class="bf-box"/><text x="333" y="48" class="bf-t-strong" text-anchor="middle">BWT</text><text x="333" y="68" class="bf-tm" text-anchor="middle">bytes reordered,</text><text x="333" y="82" class="bf-tm" text-anchor="middle">like by like</text><path d="M398 55h14m-5-5 5 5-5 5" class="bf-arrow"/><rect x="416" y="20" width="140" height="70" rx="8" class="bf-box"/><text x="486" y="48" class="bf-t-strong" text-anchor="middle">MTF + RLE2</text><text x="486" y="68" class="bf-tm" text-anchor="middle">bytes become mostly</text><text x="486" y="82" class="bf-tm" text-anchor="middle">small symbols</text><path d="M558 55h14m-5-5 5 5-5 5" class="bf-arrow"/><rect x="576" y="20" width="144" height="70" rx="8" class="bf-card-hi"/><text x="648" y="48" class="bf-t-strong" text-anchor="middle">Huffman</text><text x="648" y="68" class="bf-tm" text-anchor="middle">symbols written</text><text x="648" y="82" class="bf-tm" text-anchor="middle">as bit codes</text><path d="M60 12H700" class="bf-axis"/><text x="360" y="8" class="bf-tm" text-anchor="middle">compressing: left to right</text><path d="M716 118H8m5-5-5 5 5 5" class="bf-arrow"/><text x="360" y="140" class="bf-ts" text-anchor="middle">decoding: right to left, Huffman first</text><text x="360" y="162" class="bf-tm" text-anchor="middle">this post is about the first decoding step, reading the bit codes back</text></svg>
<figcaption>The four steps of bzip2's compression of one block. Each is undone in the opposite order, so a decoder starts with the Huffman codes.</figcaption>
</figure>

The first step shortens any run of four or more identical bytes to four bytes and a count:
bzip2's first run-length coding, RLE1. The second, the Burrows–Wheeler transform (BWT),
reorders the block's bytes so that bytes which occur in similar contexts end up next to one
another. [An earlier post](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md) works
through it with the word `banana`. The reordered block is full of repeats, and the third step,
move-to-front coding (MTF), turns repeats into small numbers. It keeps a list of byte values
ordered by how recently each was seen, and replaces each byte by its position in that list:
a byte that just appeared becomes 0, one seen a little earlier 1, and so on. Runs of zeros
are then written compactly with two special symbols, RUNA and RUNB: the second run-length
coding, RLE2.

What comes out is a sequence of symbols from a small alphabet: RUNA, RUNB, one symbol for
each list position other than 0, and an end-of-block symbol. A block that uses 100 distinct
byte values has an alphabet of 102 symbols. The block is mostly runs and small positions,
so a few symbols are very common and most are rare. That imbalance is what the fourth step,
Huffman coding, turns into fewer bits.

A bzip2 block uses not one code but several. The encoder builds between two and six code
tables, each suited to a different mix of symbols. It switches between them every 50
symbols, picking for each group of 50 the table that codes it in the fewest bits. So the
block begins with a header that tells the decoder how to follow along. It says which byte
values occur in the block, which fixes the alphabet. It holds each code table, stored as the
length of every symbol's code: the codes themselves follow from their lengths by a fixed
rule, so the lengths are all that need storing. And it holds a selector for each group of 50
symbols, naming the table that group was coded with. bzip2's codes are at most 20 bits long.

<figure class="bf-fig">
<p class="bf-title">A bzip2 block: the header, then its symbols in groups of 50</p>
<svg viewBox="0 0 720 196" role="img" aria-label="A block's header lists the byte values used, the selectors and the code tables (two to six of them). After the header come the Huffman-coded symbols in groups of 50; each group is read with the table its selector names, for example table 2, then table 1, then table 2 again, until the end-of-block symbol."><text x="0" y="18" class="bf-ts">The header</text><rect x="0" y="30" width="196" height="54" rx="8" class="bf-card"/><text x="14" y="53" class="bf-t-strong">Byte values used</text><text x="14" y="72" class="bf-tm">which of the 256 occur</text><rect x="206" y="30" width="236" height="54" rx="8" class="bf-card"/><text x="220" y="53" class="bf-t-strong">Selectors</text><text x="220" y="72" class="bf-tm">a table number for each 50 symbols</text><rect x="452" y="30" width="268" height="54" rx="8" class="bf-card"/><text x="466" y="53" class="bf-t-strong">Code tables, 2 to 6</text><text x="466" y="72" class="bf-tm">each as every symbol's code length</text><text x="0" y="116" class="bf-ts">The symbols, as Huffman codes</text><rect x="0" y="128" width="166" height="40" class="bf-cell-hi"/><text x="83" y="153" class="bf-t" text-anchor="middle">50 symbols · table 2</text><rect x="172" y="128" width="166" height="40" class="bf-cell"/><text x="255" y="153" class="bf-t" text-anchor="middle">50 symbols · table 1</text><rect x="344" y="128" width="166" height="40" class="bf-cell-hi"/><text x="427" y="153" class="bf-t" text-anchor="middle">50 symbols · table 2</text><rect x="516" y="128" width="66" height="40" class="bf-cell"/><text x="549" y="153" class="bf-t" text-anchor="middle">…</text><rect x="588" y="128" width="132" height="40" class="bf-cell"/><text x="654" y="153" class="bf-t" text-anchor="middle">… end of block</text><text x="0" y="190" class="bf-tm">The selectors decide which table reads each group. The last group stops at the end-of-block symbol.</text></svg>
<figcaption>Simplified: the header also holds the block's checksum and where the inverse BWT starts. The selectors are themselves compactly coded. The table numbers shown are an example.</figcaption>
</figure>

So reading a block's symbols means reading its header, then, for each group of 50, taking
the table its selector names and reading codes with it one after another, until the
end-of-block symbol.


mibz2 splits a decode between two kinds of thread, as the [post on block
dispatch](2026-11-01-block-dispatch.md) describes. One thread, the scanner, reads the stream
from the start. For each block it reads the header and decodes all the block's symbols. This
can only be one thread working in order: a block's end is found only by reading up to its
end-of-block symbol, and the next block begins straight after it. The scanner then hands each
block's symbols to a worker, and several workers run at once. A worker does the rest. It
undoes the move-to-front coding and RLE2 together, inverts the Burrows–Wheeler transform in
two parts (it builds a table, then walks it), then expands RLE1 and checks the block's CRC,
a checksum stored in its header.

<figure class="bf-fig">
<p class="bf-title">Where Huffman decoding sits in mibz2</p>
<svg viewBox="0 0 720 442" role="img" aria-label="A compressed stream feeds the scanner thread, which reads each block's header and Huffman-decodes its symbols at 1.19 cycles a byte. A worker then undoes move-to-front (1.79), builds the inverse BWT's table (1.66), walks it (6.19) and expands runs and checks the CRC (0.44), before the bytes are handed back in order."><rect x="110" y="0" width="500" height="58" rx="8" class="bf-card"/><text x="126" y="26" class="bf-ts">Compressed stream</text><text x="126" y="47" class="bf-tm">bzip2 blocks, each a header and its Huffman-coded symbols</text><path d="M360 62V84m-5-5 5 5 5-5" class="bf-arrow"/><rect x="40" y="88" width="640" height="110" rx="12" class="bf-panel"/><text x="360" y="114" class="bf-ts" text-anchor="middle">The scanner thread · every block, in order</text><rect x="58" y="128" width="190" height="54" rx="8" class="bf-box"/><text x="153" y="151" class="bf-t-strong" text-anchor="middle">Block header</text><text x="153" y="170" class="bf-tm" text-anchor="middle">bytes used, code tables</text><rect x="262" y="128" width="400" height="54" rx="8" class="bf-card-hi"/><text x="462" y="151" class="bf-t-strong" text-anchor="middle">Huffman decoding: this post</text><text x="462" y="170" class="bf-tm" text-anchor="middle">1.19 c/B, about 10.5% of a one-thread decode</text><path d="M360 202V224m-5-5 5 5 5-5" class="bf-arrow"/><rect x="40" y="228" width="640" height="140" rx="12" class="bf-panel"/><text x="360" y="254" class="bf-ts" text-anchor="middle">A worker · one block at a time, several workers at once</text><rect x="58" y="268" width="142" height="80" rx="8" class="bf-box"/><text x="129" y="293" class="bf-t-strong" text-anchor="middle">MTF + RLE2</text><text x="129" y="312" class="bf-tm" text-anchor="middle">undo move-to-front</text><text x="129" y="333" class="bf-tm" text-anchor="middle">1.79 c/B</text><rect x="212" y="268" width="142" height="80" rx="8" class="bf-box"/><text x="283" y="293" class="bf-t-strong" text-anchor="middle">Table build</text><text x="283" y="312" class="bf-tm" text-anchor="middle">inverse BWT, part 1</text><text x="283" y="333" class="bf-tm" text-anchor="middle">1.66 c/B</text><rect x="366" y="268" width="142" height="80" rx="8" class="bf-box"/><text x="437" y="293" class="bf-t-strong" text-anchor="middle">Walk</text><text x="437" y="312" class="bf-tm" text-anchor="middle">inverse BWT, part 2</text><text x="437" y="333" class="bf-tm" text-anchor="middle">6.19 c/B</text><rect x="520" y="268" width="142" height="80" rx="8" class="bf-box"/><text x="591" y="293" class="bf-t-strong" text-anchor="middle">RLE1 + CRC</text><text x="591" y="312" class="bf-tm" text-anchor="middle">expand runs, check</text><text x="591" y="333" class="bf-tm" text-anchor="middle">0.44 c/B</text><path d="M360 372V394m-5-5 5 5 5-5" class="bf-arrow"/><rect x="110" y="398" width="500" height="42" rx="8" class="bf-card"/><text x="360" y="424" class="bf-ts" text-anchor="middle">Decoded bytes, handed back in stream order</text></svg>
<figcaption>Measured: the base build's one-thread stage costs on the Intel Xeon 8370C runner, in cycles per byte of output at the runner's reported clock, 11.28 c/B in all (the second benchmark run of section 9). At one thread the stages run one after another on one CPU; at four, the scanner's thread runs beside the workers.</figcaption>
</figure>

At one thread every stage runs on the same CPU, one after another, so the scanner's share of
the time is its share of the decode: on the Intel runner, 1.19 of 11.28 cycles per output
byte, about 10.5%. At four threads the scanner runs alongside the workers. All four of the
runner's CPUs are then busy, so the scanner's CPU time still costs wall time, at about a
quarter of its own rate. These shares come from [a
census](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md)
we took of what limits the decoder's speed on CI's runners, stage by stage. Its journal
entry calls it the binding census, after the resource that binds each stage: the one whose
limit it runs into first, whether that is the processor's instructions, memory, or waiting
on another thread.

## 4. What the census found

The [census](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md)
asked what limits today's decoder on CI's runners. One of its lines of inquiry took the
stages that run in sequence, and read the scanner's machine code. In the four-lookup loop,
each lookup ended by storing `buf` to memory, and the next began by loading it back.

A store followed by a load from the same address costs few instructions, and it doesn't wait
for the cache: the processor hands the stored value straight to the load. That hand-over is
called store-to-load forwarding. It still takes time, about 4–5 cycles on Intel's Ice Lake
cores such as the 8370C's, and here it was part of the lookup chain. The census estimated a lookup
at about 13–15 cycles with the reload, against about 8–9 with `buf` in a register. These
figures are estimates, from the listing, the instruction counts and the measured cycles, not
counts of events: CI's runners expose no hardware performance counters.

<figure class="bf-fig">
<p class="bf-title">One lookup's chain, before and after</p>
<svg viewBox="0 0 720 214" role="img" aria-label="Before: reload buf through a store-to-load forward, shift right, load the entry, mask the length, shift buf left and store it, about 13 to 15 cycles. After: the same steps without the reload or the store, about 8 to 9 cycles."><text x="0" y="16" class="bf-ts">Before: buf kept in memory</text><rect x="0" y="28" width="150" height="48" rx="8" class="bf-pill-ready"/><text x="75" y="49" class="bf-t-strong" text-anchor="middle">reload buf</text><text x="75" y="67" class="bf-tm" text-anchor="middle">a forward, ~4–5 cycles</text><path d="M152 52h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="166" y="28" width="118" height="48" rx="8" class="bf-box"/><text x="225" y="49" class="bf-t-strong" text-anchor="middle">shift right</text><text x="225" y="67" class="bf-tm" text-anchor="middle">the top 10 bits</text><path d="M286 52h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="300" y="28" width="118" height="48" rx="8" class="bf-box"/><text x="359" y="49" class="bf-t-strong" text-anchor="middle">table load</text><text x="359" y="67" class="bf-tm" text-anchor="middle">the entry</text><path d="M420 52h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="434" y="28" width="118" height="48" rx="8" class="bf-box"/><text x="493" y="49" class="bf-t-strong" text-anchor="middle">mask</text><text x="493" y="67" class="bf-tm" text-anchor="middle">the length</text><path d="M554 52h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="568" y="28" width="152" height="48" rx="8" class="bf-pill-ready"/><text x="644" y="49" class="bf-t-strong" text-anchor="middle">shift left, store</text><text x="644" y="67" class="bf-tm" text-anchor="middle">for the next lookup</text><text x="0" y="96" class="bf-tm">about 13–15 cycles a lookup on Ice Lake</text><text x="0" y="132" class="bf-ts">After: buf held in a register</text><rect x="0" y="144" width="150" height="48" rx="8" class="bf-box"/><text x="75" y="173" class="bf-tm" text-anchor="middle">nothing to reload</text><rect x="166" y="144" width="118" height="48" rx="8" class="bf-box"/><text x="225" y="165" class="bf-t-strong" text-anchor="middle">shift right</text><text x="225" y="183" class="bf-tm" text-anchor="middle">the top 10 bits</text><path d="M286 168h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="300" y="144" width="118" height="48" rx="8" class="bf-box"/><text x="359" y="165" class="bf-t-strong" text-anchor="middle">table load</text><text x="359" y="183" class="bf-tm" text-anchor="middle">the entry</text><path d="M420 168h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="434" y="144" width="118" height="48" rx="8" class="bf-box"/><text x="493" y="165" class="bf-t-strong" text-anchor="middle">mask</text><text x="493" y="183" class="bf-tm" text-anchor="middle">the length</text><path d="M554 168h10m-4-4 4 4-4 4" class="bf-arrow"/><rect x="568" y="144" width="152" height="48" rx="8" class="bf-card-hi"/><text x="644" y="165" class="bf-t-strong" text-anchor="middle">shift left</text><text x="644" y="183" class="bf-tm" text-anchor="middle">stays in a register</text><text x="0" y="210" class="bf-tm">about 8–9 cycles a lookup</text></svg>
<figcaption>Estimated, not measured: the census's reading of the listing for Ice Lake. The measured saving is in section 9.</figcaption>
</figure>

Across the four runner models the census measured, the scanner took 6.9–10.3% of the
four-thread wall time (7–10% in its summary) and 11–14% of the one-thread time. It estimated
that removing the forward would recover 0.5–4.1% of the four-thread wall, and 1.7–5.6% at one
thread, and called it "The cheapest and most certain mechanism on this front."

## 5. Why the compiler kept the buffer in memory

Nothing in the Rust asked for a store. The loop calls the reader's methods, which are small
and marked `#[inline(always)]`, so their bodies are compiled into the loop. Two calls are not
inlined: the slow path for codes longer than ten bits, and the slow refill near the end of
the input. Both are marked `#[cold]`, and on master before the change both took the reader by
reference, as `&mut Bits`
([`decode_entry_slow(e, br)`](https://github.com/lmmx/mibz2/blob/66a9dedd/src/block.rs#L343),
[`refill_slow(&mut self)`](https://github.com/lmmx/mibz2/blob/66a9dedd/src/bits.rs#L49-L50)).

Passing a reference to a function that isn't inlined lets the reader's address escape. As far
as the compiler (LLVM) can tell, that function might read or change the reader, so the
reader's fields have to be in memory, at that address, whenever such a call can happen. The
calls are rare, but one can happen at any lookup. A second constraint follows from the first.
The loop writes each decoded symbol through a pointer into the output, `out[n]`, and once the
reader's address has escaped, LLVM must assume such a write might change it too. So it loads
`buf` again after the writes
([registration](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md)).

This is one of the four steps, from master before the change, as the [archived
listing](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register/asm-excerpt.txt)
gives it (the bench binary, rustc 1.99.0, in Intel syntax: destination first). `r15` holds
the reader's address:

```text
ab240  mov    rsi,QWORD PTR [r15+0x18]                              bits.rs:74
ab244  mov    rax,rsi                                               bits.rs:74
ab247  shr    rax,0x36                                              bits.rs:74
ab24b  mov    edx,DWORD PTR [r9+rax*4]                              block.rs:326
ab24f  mov    eax,edx                                               block.rs:331
ab251  shr    eax,0xb                                               block.rs:331
ab254  and    eax,0x3                                               block.rs:331
ab257  je     ab450 <mibz2::block::read_item+0x12c0>                block.rs:331
ab25d  mov    ecx,edx                                               block.rs:332
ab25f  and    ecx,0x1f                                              block.rs:332
ab262  shl    rsi,cl                                                bits.rs:81
ab265  mov    QWORD PTR [r15+0x18],rsi                              bits.rs:81
ab269  sub    DWORD PTR [r15+0x20],ecx                              bits.rs:82
```

| Instructions | What they do |
| --- | --- |
| `mov rsi,QWORD PTR [r15+0x18]` | load `buf` back from the reader in memory: the forward |
| `mov`, `shr rax,0x36` | its top ten bits (a shift right by 54), the index |
| `mov edx,DWORD PTR [r9+rax*4]` | load the table entry |
| `mov`, `shr`, `and`, `je` | the entry's symbol count: none means the slow path |
| `mov`, `and ecx,0x1f` | the bits to consume, the entry's low five bits |
| `shl rsi,cl` | consume them |
| `mov QWORD PTR [r15+0x18],rsi` | store `buf` back to the reader |
| `sub DWORD PTR [r15+0x20],ecx` | take them from `live`, in memory |

Over a four-lookup iteration the registration counts five stores and three reloads of `buf`,
and four read-modify-writes of `live` in memory: about one forward on the chain per lookup.

## 6. The change: the state as a value

The change, [#204](https://github.com/lmmx/mibz2/pull/204), moves the three fields into a
small struct that can be copied, `Reg`, and has the loop hold one in a local variable
([`src/bits.rs`](https://github.com/lmmx/mibz2/blob/264ae5bb/src/bits.rs#L6-L24)):

```rust
pub struct Bits<'a> {
    data: &'a [u8],
    r: Reg,
}

#[derive(Clone, Copy)]
pub(crate) struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}
```

- **The logic moves unchanged.** Refill, peek and skip become methods of `Reg`.
- **The cold paths take the state by value** and return the state after:
  `decode_entry_slow(&self, e: u32, r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg)`,
  and `refill_slow(mut self, data: &[u8]) -> Reg`. No call receives the state's address, so
  the compiler can keep its fields in registers, and pass them into a cold call and back
  like any other arguments.
- **The loop works on a copy.** [`read_item`](https://docs.rs/mibz2/0.2.2/mibz2/block/fn.read_item.html) takes the state out of the reader before the
  symbol loop, with `let (data, mut r) = br.parts();`, and writes it back once, with
  `br.set(r)`, on every way out of the loop.
- **The loop is rotated.** The refill moves from the top of the four-lookup loop to its
  foot. With the refill at the top, the state was carried from one pass of the loop to the
  next in memory, which cost a store and a reload every refill. The refills happen at the same points as before:

```rust
if left >= 2 * STEPS {
    r.refill(data);
    loop {
        for _ in 0..STEPS {
            step!(true);
        }
        if left < 2 * STEPS {
            break;
        }
        r.refill(data);
    }
}
```

`Bits`'s public methods keep their signatures, and the block header is parsed as before.
This is the same step in the new build, from the same listing:

```text
ab582  shr    rcx,0x36                                              bits.rs:71
ab586  mov    rdx,QWORD PTR [rsp+0x28]                              bits.rs:71
ab58b  mov    edx,DWORD PTR [rdx+rcx*4]                             block.rs:335
ab58e  mov    r9d,edx                                               block.rs:340
ab591  shr    r9d,0xb                                               block.rs:340
ab595  and    r9d,0x3                                               block.rs:340
ab599  je     ab853 <mibz2::block::read_item+0x1643>                block.rs:340
ab59f  mov    ecx,edx                                               block.rs:341
ab5a1  and    ecx,0x1f                                              block.rs:341
ab5a4  shl    rdi,cl                                                bits.rs:78
```

`buf` is in `rdi`, and the step ends at `shl rdi,cl`, with no store. The loop's only stores
now are the symbol writes. It still loads values that don't change during the loop from the
stack, such as the table's address (`[rsp+0x28]`). Nothing in the loop stores to them, so
those loads wait on no forward, and their addresses don't depend on the chain.

## 7. What was predicted, before any timing

Before any CI timing, the change was registered: its mechanism, the measurements that need
no timing, a prediction and a decision rule, committed before any CI timing of the branch
([registration](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md)).

Instruction counts from callgrind, which counts every instruction a program executes, on the
bench corpus:

| Instructions | Before | After | Change |
| --- | ---: | ---: | ---: |
| Per four-lookup iteration | 153.8 | 144.8 | −5.9% |
| The whole scanner | 746.2 million | 727.4 million | −2.5% |
| Per byte of output | 3.56 | 3.47 | −2.5% |

The forwards also leave the lookup chain: four in each iteration, about 0.94 a lookup on
average. The registration then estimated the saving at about $F$ cycles a lookup, where $F$ is
the store-to-load forwarding latency on the runner, plus the 2.5% of the scanner's
instructions:

| Figure | Predicted |
| --- | --- |
| Scanner stage, one thread, Intel Ice Lake (8370C), $F$ about 4–5 | 15–25% faster |
| Scanner stage, one thread, AMD and Intel Golden Cove (8573C) | 0–25% faster |
| Wikidata wall, one thread, Intel | 1.5–3.5% faster |
| Wikidata wall, four threads | 0.5–2.5% faster |
| MTF, table build, walk, RLE1, CRC | unchanged: their code is untouched |

The AMD range is wide because some cores can rename a store and a later load of the same
address: they recognise the pair and hand the value over without the forward's latency.
Whether CI's AMD runners do that for this code wasn't known.

The decision rule had four parts:

- **Correctness, required:** CI green, and the same bytes on every corpus in every bench run.
- **Benefit, required:** in at least one run, the Wikidata one-thread wall or CPU faster
  beyond noise, or the one-thread scanner stage faster beyond noise with the whole decoder's
  one-thread point estimate not slower; and the same figure not slower in the other run.
- **Refutation:** a scanner cut under 10% on an Intel runner, or any other stage moving
  beyond noise, would refute the mechanism.
- **No harm, required:** a wall figure flagged slower is settled by one more run on the same
  head. Flagged again, the rule fails. At most three runs, with each run's CPU named.

## 8. Is it still the same decoder?

The change has to give exactly the same symbols, and fail in exactly the same way on corrupt
input. Before timing:

- **A differential** ran master's `bits.rs` and `block.rs` against the new ones, outside the
  repository, on 52 files and 164,076 cases of intact and mutated blocks. It compared 9.15
  billion symbols, every case was identical, and all 11 error kinds that can be reached were
  exercised.
- **A scanner-only driver** gives the same 24,979,299 symbols, with the same hash.
- **New tests:**
    - the reader's state stepped as the loop steps it, compared with the input's bits;
    - 4,000 fully written blocks, with real or corrupt codes up to 20 bits, bit flips and
      cuts. Each is compared with a reference that decodes one lookup at a time: the
      symbols, the error, the end bit and whether it read past the input;
    - 300 corrupted level-1 streams, compared with libbzip2.

In both benchmark runs, every corpus decoded to the same bytes with both builds, and CI
passed.

## 9. What the benchmark measured

The CI benchmark decodes every corpus with the base build and the PR's build in alternating
order, ten paired rounds a run. It runs on a GitHub-hosted runner with four virtual CPUs: two
physical cores, each running two hardware threads (SMT siblings). The CPU model varies from
run to run. It ran
twice on #204's final head, 53cc38db: once on an AMD EPYC 9V74 (Zen 4) and once on an Intel
Xeon 8370C (Ice Lake). Each change below is the PR's build over the base: the geometric mean
of the rounds' paired ratios, with its 95% interval in percent beneath it. Negative is
faster. "Wall" is the elapsed time to decode the Wikidata sample.

| Figure | Predicted | Intel 8370C | AMD 9V74 |
| --- | --- | ---: | ---: |
| Scanner, one thread | Intel: −15% to −25%<br>AMD: 0 to −25% | −12.9%<br>(−13.9, −11.9) | −5.0%<br>(−6.3, −3.6) |
| Scanner, cycles a byte | | 1.19 → 1.04 | 1.18 → 1.12 |
| Wall, one thread | Intel: −1.5% to −3.5% | −2.7%<br>(−4.3, −1.1) | +1.0%<br>(−0.7, +2.6) |
| Wall, two threads | none | −2.5%<br>(−4.6, −0.3) | −1.3%<br>(−4.0, +1.5) |
| Wall, four threads | −0.5% to −2.5% | −1.3%<br>(−5.6, +3.3) | −0.3%<br>(−3.2, +2.6) |
| Other stages | no change | none moved | none moved |

**On Intel, the stage cut missed its prediction.** It was 12.9%, against 15–25%. That is
above the 10% the rule set as the refutation, so a saving of the predicted kind was there,
but smaller than the model estimated. By our arithmetic, which is derived rather than
measured, the stage saved 0.15 cycles per output byte at the runner's reported clock. At the
census's 0.0646 lookups per byte, that is about 2.3 cycles a lookup, against the 4–5 the
model estimated. With no hardware counters on CI, the benchmark can't say which part of the
model was wrong.

**On AMD, the cut was smaller still, as the mechanism allowed.** It was 5.0%, about 0.9
cycles a lookup by the same arithmetic, and inside the 0–25% registered for a core that may
rename the forward. That fits renaming, but no counter shows it.

**One-thread decoding got faster on Intel, and no change was detected on AMD.** On Intel the
whole one-thread decode was 2.7% faster, inside the predicted 1.5–3.5%. The scanner's own
saving explains about half of that point estimate: 0.15 of 11.28 cycles per byte, 1.3%. The
rest is within the noise of the other stages, none of which moved beyond noise. On AMD the
one-thread decode was +1.0% (−0.7%, +2.6%), with no change detected either way.

**One run's interval understates how much a figure moves between runs.** Each interval above
describes the spread between one run's ten rounds. A/A controls, benchmark runs on merges
that left the decoder alone, show the one-thread figure also moving by about 0.7% from run
to run. With that spread added, the Intel one-thread result is −2.7% (−4.8%, −0.6%): still
faster.

**At four threads, no change was detected on either runner.** The prediction, 0.5–2.5%, is
smaller than the benchmark can resolve at four threads, where the intervals here are 6–9
points wide. At four threads the scanner thread's CPU time fell from 0.15 to 0.12 seconds on
Intel and from 0.09 to 0.08 on AMD, out of about 1.6 and 1.1 seconds in all. No figure was
flagged slower in either run.

Under its rule, the change passed. Correctness held. The Intel run showed the benefit, with
the one-thread wall and CPU both faster beyond noise, and the AMD run's same figure was not
slower. The Intel stage cut was above the refutation threshold, no other stage moved, and
nothing was flagged as a harm.

PR #204 met its registered decision rule and was merged on 5 November 2026, as 264ae5bb.

## 10. What it shows

The Huffman codes, the tables and the bits consumed are the same as before, and the scanner
runs only 2.5% fewer instructions. What changed is the chain each lookup waits on. On Intel,
2.5% fewer instructions gave a 12.9% shorter stage, because what was removed, the forward,
was part of that chain. Elsewhere in mibz2 the opposite has happened: in the [post on memory-level
parallelism](2026-11-02-memory-level-parallelism.md#4-cutting-instructions-helped-the-walk-but-not-four-worker-throughput),
removing a third of the walk's instructions made no difference to four workers. An
instruction count predicts time where the instructions sit on the critical path.

The pattern is easy to recreate. A loop carries a value from one iteration to the next, and
also passes a reference to it to a function that isn't inlined, such as a cold path that
takes `&mut self`. The value's address escapes, so it lives in memory, and every iteration
costs a store and a reload. Passing the state by value and taking it back keeps it in
registers. The census found two other store-to-load chains, in MTF's list and in the table
build's byte cursors. Those are different: there the data lives in memory because the
algorithm keeps it there. The bit buffer had no reason to be in memory, only a reference that
let it escape.

A decode that takes more symbols a lookup is being prototyped separately, and would combine
with this change. The census estimated that a wider table, now, would take at most about another
10% off the scanner, which is under 1% of the four-thread wall.

---

**Sources:**

- the change: [#204](https://github.com/lmmx/mibz2/pull/204), and its
  [registration](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md),
  committed before any CI timing;
- the listings: [`asm-excerpt.txt`](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register/asm-excerpt.txt),
  and the instruction counts per address in
  [`cg/`](https://github.com/lmmx/mibz2/tree/264ae5bb/docs/journal/2026-11-05-huffman-register/cg);
- the code before: [`src/bits.rs`](https://github.com/lmmx/mibz2/blob/66a9dedd/src/bits.rs)
  and [`src/block.rs`](https://github.com/lmmx/mibz2/blob/66a9dedd/src/block.rs) at 66a9dedd;
  after: [`src/bits.rs`](https://github.com/lmmx/mibz2/blob/264ae5bb/src/bits.rs) and
  [`src/block.rs`](https://github.com/lmmx/mibz2/blob/264ae5bb/src/block.rs) at 264ae5bb;
- the scanner's share and the estimates: [the census of what limits the
  decoder](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md)
  and its [serial stages](https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census/serial-stages.md);
- the two benchmark runs: [run 42057920798](https://github.com/lmmx/mibz2/actions/runs/42057920798),
  attempts 1 (AMD EPYC 9V74) and 2 (Intel Xeon 8370C), reported on
  [#204](https://github.com/lmmx/mibz2/pull/204).
