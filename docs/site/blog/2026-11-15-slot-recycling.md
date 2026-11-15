---
title: "Slot recycling: how mibz2 gets faster by reusing memory it no longer needs"
image: assets/social/blog-2026-11-15-slot-recycling.png
description: "mibz2's Cells induction now reuses the slot a finished letter leaves behind, touching 29.7% fewer slots and trimming peak memory by up to 14.6%."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Slot recycling: how mibz2 gets faster by reusing memory it no longer needs

mibz2's Cells induction now reuses the slot a finished letter leaves behind, touching 29.7% fewer slots and trimming peak memory by up to 14.6%.
{: .post-dek }

15 November 2026 · 13 min read · [Research result](#7-what-the-benchmarks-tell-us)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

## The idea in a nutshell

**Slot recycling makes mibz2 faster by recognising that, whenever its [Cells algorithm](2026-11-10-rauzy-veech-reconstruction.md) needs somewhere to put a new piece of information, another piece of information has just finished using its space.**

Previously, the algorithm reserved a fresh memory slot for the new piece, leaving the old slot behind. Now, it reuses the old slot immediately.

This is a deceptively simple improvement. The algorithm still performs the same mathematical operations and reconstructs exactly the same bytes. What changes is how much memory it needs to touch while doing so.

On the Wikidata benchmark, slot recycling:

- **Eliminates 29.7% of the slots previously touched** during Cells induction.
- **Reduces peak decoder memory usage by 10–15%.**
- **Makes four-thread decoding 1.7% faster** in the initial paired CI benchmark, without a statistically established speed change at one or two threads.

The important insight is that **we aren't making the mathematics smaller; we're making its physical representation more efficient**. We're taking advantage of something already guaranteed by the mathematics: one slot's useful lifetime ends just as another begins.

And because mibz2's performance is sensitive to memory contention, eliminating unnecessary memory accesses can matter more than shaving a few arithmetic instructions.

## 1. Why memory matters so much to mibz2

To understand why this is worthwhile, we need to understand what mibz2 spends much of its time doing.

Bzip2 compression uses the Burrows–Wheeler transform (BWT), which rearranges a block's bytes into an order that's easier to compress. Decompression must reverse that rearrangement and recover the original text.

The conventional way to reverse the BWT is to construct a permutation describing where each position leads, then follow that permutation to reconstruct the original sequence.

This is computationally awkward for modern processors.

Each step tells the processor where to look next. The address of the following memory access often cannot be known until the current access completes. If the relevant data isn't in cache, the processor may spend much of its time waiting for memory.

mibz2's original approach was to interleave several independent traversals, allowing the processor to have multiple memory operations in flight.

But the Cells algorithm takes a more radical approach.

**Instead of reconstructing the text by following the permutation one position at a time, Cells exploits the structure of the permutation itself.**

The inverse-BWT permutation has a special organisation: it acts as a translation over intervals corresponding to runs of equal bytes. That allows it to be treated as a finite interval-exchange system.

Using a form of Rauzy–Veech induction, Cells repeatedly simplifies this system. Each simplification combines pieces of the permutation and builds a compact description of the text they represent.

Eventually, the algorithm has enough information to reconstruct the original block.

This can require dramatically fewer algorithmic steps than following every position in the original permutation.

However, those steps have their own cost: Cells must maintain a changing representation of the intervals, their relationships and the words they represent.

That representation is where slot recycling operates.

## 2. What is a slot?

Imagine that Cells is manipulating a collection of pieces, which we'll call *letters*. Each letter represents an interval together with a word: the sequence of output bytes associated with it.

The induction maintains two orderings of these letters, corresponding to the two sides of the interval exchange.

We can think of them as:

- The **top order**, describing one arrangement of the pieces.
- The **bottom order**, describing the corresponding arrangement on the other side.

These aren't two complete copies of the decompressed text. They're two linked arrangements of the same mathematical pieces.

Each letter has a slot in each order.

<figure class="bf-fig">
<p class="bf-title">Two linked orders of the same letters</p>
<svg viewBox="0 0 720 312" role="img" aria-label="The top order holds the letters in one arrangement and the bottom order in another. Each letter has one slot in each order, and a dashed partner link joins the two. Each slot links to the slot before it, and the induction reads both orders from their fronts: A on top, B on the bottom."><text x="0" y="66" class="bf-ts">Top order</text><text x="0" y="84" class="bf-tm">A is at its front</text><text x="0" y="184" class="bf-ts">Bottom order</text><text x="0" y="202" class="bf-tm">B is at its front</text><text x="612.0" y="24" class="bf-tm" text-anchor="middle">front: read first</text><path d="M612.0 30V40" class="bf-arrow"/><text x="612.0" y="246" class="bf-tm" text-anchor="middle">front: read first</text><path d="M612.0 236V226" class="bf-arrow"/><path d="M282.0 92L502.0 166" class="bf-arrow" stroke-dasharray="4 4"/><path d="M392.0 92L282.0 166" class="bf-arrow" stroke-dasharray="4 4"/><path d="M502.0 92L612.0 166" class="bf-arrow" stroke-dasharray="4 4"/><path d="M612.0 92L392.0 166" class="bf-arrow" stroke-dasharray="4 4"/><text x="172.0" y="75" class="bf-tl" text-anchor="middle">…</text><rect x="240" y="46" width="84" height="44" rx="6" class="bf-slot"/><text x="282.0" y="74" class="bf-ts" text-anchor="middle">D</text><rect x="350" y="46" width="84" height="44" rx="6" class="bf-slot"/><text x="392.0" y="74" class="bf-ts" text-anchor="middle">C</text><rect x="460" y="46" width="84" height="44" rx="6" class="bf-slot"/><text x="502.0" y="74" class="bf-ts" text-anchor="middle">B</text><rect x="570" y="46" width="84" height="44" rx="6" class="bf-slot"/><text x="612.0" y="74" class="bf-ts" text-anchor="middle">A</text><path d="M237 68H217m5-5-5 5 5 5" class="bf-arrow"/><path d="M347 68H327m5-5-5 5 5 5" class="bf-arrow"/><path d="M457 68H437m5-5-5 5 5 5" class="bf-arrow"/><path d="M567 68H547m5-5-5 5 5 5" class="bf-arrow"/><text x="172.0" y="197" class="bf-tl" text-anchor="middle">…</text><rect x="240" y="168" width="84" height="44" rx="6" class="bf-slot"/><text x="282.0" y="196" class="bf-ts" text-anchor="middle">C</text><rect x="350" y="168" width="84" height="44" rx="6" class="bf-slot"/><text x="392.0" y="196" class="bf-ts" text-anchor="middle">A</text><rect x="460" y="168" width="84" height="44" rx="6" class="bf-slot"/><text x="502.0" y="196" class="bf-ts" text-anchor="middle">D</text><rect x="570" y="168" width="84" height="44" rx="6" class="bf-slot"/><text x="612.0" y="196" class="bf-ts" text-anchor="middle">B</text><path d="M237 190H217m5-5-5 5 5 5" class="bf-arrow"/><path d="M347 190H327m5-5-5 5 5 5" class="bf-arrow"/><path d="M457 190H437m5-5-5 5 5 5" class="bf-arrow"/><path d="M567 190H547m5-5-5 5 5 5" class="bf-arrow"/><path d="M0 278H704" class="bf-rule"/><path d="M26 300H4m5-5-5 5 5 5" class="bf-arrow"/><text x="34" y="304" class="bf-tm">each slot links to the slot before it in its order</text><path d="M366 300H392" class="bf-arrow" stroke-dasharray="4 4"/><text x="400" y="304" class="bf-tm">partner: the same letter’s slot in the other order</text></svg>
<figcaption>Conceptual: five slots of each order. Each letter has a slot in both orders, in a different arrangement, so the partner links cross. The induction reads each order from its front, where A and B are, and follows the links to the slots before them.</figcaption>
</figure>

A slot is a small record in memory. In the implementation shipped before this change, it occupies **20 bytes**, containing the information needed to navigate and manipulate the induction:

| Information | Purpose |
|---|---|
| Previous-slot link | Finds the next piece when advancing through the order |
| Interval length | Describes how many positions the piece covers |
| Word reference and length | Identifies the text represented by the piece |
| Partner-slot reference | Connects its position in one order to the corresponding position in the other |

<figure class="bf-fig">
<p class="bf-title">A slot is 20 bytes: five 4-byte fields</p>
<svg viewBox="0 0 720 250" role="img" aria-label="A slot of 20 bytes, five fields of 4 bytes each: bytes 0 to 4 the slot before it in its order; 4 to 8 the letter's length; 8 to 12 its word's length; 12 to 16 its word's offset; 16 to 20 the letter's slot in the other order. Bytes 8 to 16 are read as one 8-byte load. A 64-byte cache line holds 3.2 slots."><text x="40" y="22" class="bf-ttiny" text-anchor="middle">0</text><text x="168" y="22" class="bf-ttiny" text-anchor="middle">4</text><text x="296" y="22" class="bf-ttiny" text-anchor="middle">8</text><text x="424" y="22" class="bf-ttiny" text-anchor="middle">12</text><text x="552" y="22" class="bf-ttiny" text-anchor="middle">16</text><text x="680" y="22" class="bf-ttiny" text-anchor="middle">20</text><text x="32" y="22" class="bf-ttiny" text-anchor="end">byte</text><rect x="41" y="30" width="126" height="58" rx="5" class="bf-slot"/><text x="104" y="55" class="bf-ts" text-anchor="middle">slot before</text><text x="104" y="74" class="bf-tm" text-anchor="middle">4 B · link in its order</text><rect x="169" y="30" width="126" height="58" rx="5" class="bf-slot"/><text x="232" y="55" class="bf-ts" text-anchor="middle">letter’s length</text><text x="232" y="74" class="bf-tm" text-anchor="middle">4 B · positions left</text><rect x="297" y="30" width="126" height="58" rx="5" class="bf-slot"/><text x="360" y="55" class="bf-ts" text-anchor="middle">word’s length</text><text x="360" y="74" class="bf-tm" text-anchor="middle">4 B · in bytes</text><rect x="425" y="30" width="126" height="58" rx="5" class="bf-slot"/><text x="488" y="55" class="bf-ts" text-anchor="middle">word’s offset</text><text x="488" y="74" class="bf-tm" text-anchor="middle">4 B · bytes or a rule</text><rect x="553" y="30" width="126" height="58" rx="5" class="bf-slot"/><text x="616" y="55" class="bf-ts" text-anchor="middle">partner slot</text><text x="616" y="74" class="bf-tm" text-anchor="middle">4 B · in the other order</text><path d="M298 96V104H550V96M424.0 104V110" class="bf-arrow"/><text x="424.0" y="126" class="bf-tm" text-anchor="middle">the word: read as one 8-byte load</text><text x="360" y="150" class="bf-t-strong" text-anchor="middle">20 bytes, five 4-byte fields</text><path d="M0 172H720" class="bf-rule"/><text x="0" y="196" class="bf-ts">A 64-byte cache line</text><text x="0" y="214" class="bf-tm">holds 3.2 slots, so slots straddle lines</text><rect x="271" y="182" width="138" height="30" rx="4" class="bf-slot"/><text x="340" y="201" class="bf-tm" text-anchor="middle">slot 1</text><rect x="411" y="182" width="138" height="30" rx="4" class="bf-slot"/><text x="480" y="201" class="bf-tm" text-anchor="middle">slot 2</text><rect x="551" y="182" width="138" height="30" rx="4" class="bf-slot"/><text x="620" y="201" class="bf-tm" text-anchor="middle">slot 3</text><rect x="691" y="182" width="26" height="30" rx="4" class="bf-slot"/><rect x="717" y="182" width="84" height="30" rx="4" class="bf-track"/><path d="M270 218V224H718V218" class="bf-arrow"/><text x="494" y="240" class="bf-tm" text-anchor="middle">64 bytes</text><text x="760" y="201" class="bf-ttiny" text-anchor="middle">next line</text></svg>
<figcaption>The layout as <code>src/cells.rs</code> documents it, unchanged by #408. Every field is a 4-byte offset or length. The table above lists the word’s length and offset as one row, since the induction reads them together. Below, to scale: 64 bytes hold three whole slots and part of a fourth.</figcaption>
</figure>

The algorithm walks through these two orders, repeatedly taking pieces from their fronts.

As it goes, it transforms letters into *cells*: finished pieces whose words can be represented by literal bytes or by references to previously constructed words.

The slot records are the bookkeeping that makes this transformation possible.

And because there can be many millions of cells, even seemingly small inefficiencies in this bookkeeping accumulate.

## 3. How an induction step works

At each step, Cells considers two pieces, A and B, one encountered through each order.

Suppose their remaining interval lengths are:

- A has length **7**.
- B has length **3**.

The algorithm takes the smaller length, 3, and constructs a cell representing the corresponding portions of A and B. Its word is constructed by concatenating their associated words.

Now B has been exhausted, while A still has 4 positions remaining.

Conceptually:

<figure class="bf-fig">
<p class="bf-title">One unequal step: B ends, A goes on</p>
<svg viewBox="0 0 720 160" role="img" aria-label="Before the step, A has 7 positions and B has 3. The step cuts a cell of the smaller length, 3, from the front of both. Afterwards A goes on with 4 positions, B has ended, and the new cell has length 3 and word A's word followed by B's."><text x="0" y="16" class="bf-cap">Before the step</text><text x="0" y="57" class="bf-ts">A</text><rect x="28" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="56" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="84" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="112" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="140" y="40" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="168" y="40" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="196" y="40" width="24" height="24" rx="3" class="bf-cell-hi"/><text x="228" y="57" class="bf-tm">7</text><text x="0" y="97" class="bf-ts">B</text><rect x="140" y="80" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="168" y="80" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="196" y="80" width="24" height="24" rx="3" class="bf-cell-hi"/><text x="228" y="97" class="bf-tm">3</text><path d="M140 112V118H220V112" class="bf-cut"/><text x="180.0" y="136" class="bf-t-green" text-anchor="middle">the cell: min(7, 3) = 3</text><text x="180.0" y="154" class="bf-tm" text-anchor="middle">at the front of both</text><path d="M262 70H322m-6-6 6 6-6 6" class="bf-arrow"/><text x="292" y="58" class="bf-tm" text-anchor="middle">one step</text><text x="350" y="16" class="bf-cap">After the step</text><text x="350" y="57" class="bf-ts">A</text><rect x="378" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="406" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="434" y="40" width="24" height="24" rx="3" class="bf-slot"/><rect x="462" y="40" width="24" height="24" rx="3" class="bf-slot"/><text x="496" y="57" class="bf-t">goes on, with 4 left</text><text x="350" y="97" class="bf-ts">B</text><rect x="378" y="80" width="80" height="24" rx="3" class="bf-box" stroke-dasharray="4 3"/><text x="468" y="97" class="bf-t">has ended</text><text x="350" y="137" class="bf-ts">cell</text><rect x="408" y="120" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="436" y="120" width="24" height="24" rx="3" class="bf-cell-hi"/><rect x="464" y="120" width="24" height="24" rx="3" class="bf-cell-hi"/><text x="498" y="131" class="bf-t">length 3</text><text x="498" y="148" class="bf-tm">word: A’s word, then B’s</text></svg>
<figcaption>Conceptual: the example in the text. Each square is one position. The green squares become the cell; A’s other four stay A’s. B ends, so its slot in the bottom order is free from this step on.</figcaption>
</figure>

This is a simplification of the mathematical operation, but it captures the essential event.

One letter continues. The other finishes.

If A and B had equal remaining lengths, both would finish together. The interesting case for slot recycling is when their lengths differ, because the surviving letter will eventually produce another cell.

**That next cell needs a place in the other ordering.**

Previously, the algorithm reserved a fresh slot for it.

But notice what has just happened: the other letter, B, has finished. Its position at the front of that ordering is no longer needed.

This is where the opportunity appears.

## 4. The old approach: make room for a future cell

The original implementation was designed to avoid complicated rearrangements of the linked orders.

When a letter continued after an unequal-length step, Cells *promised* it a slot for its next cell. That slot came from additional space reserved beyond the original letters' slots.

This was straightforward and efficient in an important sense: it avoided having to move existing letters or reorganise the whole structure.

But every promise of a fresh slot extended the set of memory locations the induction would eventually touch.

Across millions of steps, many of these fresh locations occupied cache lines that had not previously been used.

Touching a new cache line may require the processor to bring it into cache, obtain write ownership and eventually evict or write back dirty data. These costs become particularly significant when several workers are competing for limited cache capacity.

Meanwhile, some original slots were already dead.

The algorithm had information about their former contents in registers, yet their memory locations were being left unused.

We had **space becoming available at precisely the moment we were reserving more space elsewhere**.

## 5. The new approach: recycle the slot that just died

Slot recycling changes how that promised location is chosen.

Returning to our example, B has just finished and A continues.

Previously, the next cell cut from A would receive a fresh slot in the bottom order.

Now it receives **B's former front slot in that same order**.

<figure class="bf-fig">
<p class="bf-title">The bottom order after B ends: where A’s next cell goes</p>
<svg viewBox="0 0 720 346" role="img" aria-label="The bottom order's slots in memory, after the step where B ends and A goes on. A's slot now holds the cell. Before #408, A's next cell was promised a fresh slot past the end of the letters' slots, in memory never touched before, and B's dead front slot was left. With #408, A's next cell is promised B's old front slot, which was just read, and the region ends at the letters' slots."><text x="0" y="14" class="bf-ts">Before #408: a fresh slot past the letters</text><text x="40.0" y="95" class="bf-tl" text-anchor="middle">…</text><rect x="84" y="66" width="80" height="46" rx="6" class="bf-slot"/><text x="124.0" y="86" class="bf-ts" text-anchor="middle">D</text><rect x="168" y="66" width="80" height="46" rx="6" class="bf-slot"/><text x="208.0" y="86" class="bf-ts" text-anchor="middle">A</text><text x="208.0" y="103" class="bf-tm" text-anchor="middle">now the cell</text><rect x="252" y="66" width="80" height="46" rx="6" class="bf-slot"/><text x="292.0" y="86" class="bf-ts" text-anchor="middle">C</text><rect x="336" y="66" width="80" height="46" rx="6" class="bf-track"/><text x="376.0" y="86" class="bf-ts" text-anchor="middle">B</text><text x="376.0" y="103" class="bf-tm" text-anchor="middle">dead, left</text><text x="376.0" y="130" class="bf-tm" text-anchor="middle">the front</text><text x="376.0" y="145" class="bf-tm" text-anchor="middle">just read</text><path d="M418.0 56V122" class="bf-tick"/><text x="426.0" y="130" class="bf-tm">end of the letters’ slots</text><rect x="426.0" y="62" width="286.0" height="54" rx="8" class="bf-lane"/><rect x="436.0" y="66" width="96" height="46" rx="6" class="bf-pill-ready"/><text x="484.0" y="86" class="bf-pill-ready-t" text-anchor="middle">A’s next</text><text x="484.0" y="103" class="bf-tm" text-anchor="middle">fresh slot</text><text x="544.0" y="86" class="bf-tm">room for one fresh</text><text x="544.0" y="103" class="bf-tm">slot per cell</text><path d="M208.0 62C208.0 26 484.0 26 484.0 60m-5-6 5 6 5-6" class="bf-arrow"/><text x="346.0" y="38" class="bf-tm" text-anchor="middle">A’s next cell is promised a slot never touched before</text><path d="M0 166H720" class="bf-rule"/><text x="0" y="192" class="bf-ts">With #408: the dead front slot is recycled</text><text x="40.0" y="273" class="bf-tl" text-anchor="middle">…</text><rect x="84" y="244" width="80" height="46" rx="6" class="bf-slot"/><text x="124.0" y="264" class="bf-ts" text-anchor="middle">D</text><rect x="168" y="244" width="80" height="46" rx="6" class="bf-slot"/><text x="208.0" y="264" class="bf-ts" text-anchor="middle">A</text><text x="208.0" y="281" class="bf-tm" text-anchor="middle">now the cell</text><rect x="252" y="244" width="80" height="46" rx="6" class="bf-slot"/><text x="292.0" y="264" class="bf-ts" text-anchor="middle">C</text><rect x="336" y="244" width="80" height="46" rx="6" class="bf-card-hi"/><text x="376.0" y="264" class="bf-t-green" text-anchor="middle">A’s next</text><text x="376.0" y="281" class="bf-tm" text-anchor="middle">was B’s</text><text x="376.0" y="308" class="bf-tm" text-anchor="middle">the front</text><text x="376.0" y="323" class="bf-tm" text-anchor="middle">just read</text><path d="M418.0 234V300" class="bf-tick"/><text x="426.0" y="308" class="bf-tm">end of the letters’ slots</text><path d="M208.0 240C208.0 204 376.0 204 376.0 238m-5-6 5 6 5-6" class="bf-act"/><text x="292.0" y="216" class="bf-t-green" text-anchor="middle">promised B’s old slot</text><text x="426.0" y="264" class="bf-t">no slot past here</text><text x="426.0" y="281" class="bf-tm">the region is letters × 20 B</text></svg>
<figcaption>Conceptual, after the step in the last figure. Slots are in memory order; the front is the slot the induction reads next, and it moves left as the induction goes on. The promised slot is linked in as the slot before A’s cell, so A’s next cell will be read there. The cell is also written to B’s slot in the top order (not drawn). When A ends instead, the same happens in the top order, with A’s front slot.</figcaption>
</figure>

This isn't an ordinary memory allocator discovering that a block of memory happens to be free. The algorithm knows, from the structure of its induction, that this specific slot has become available.

The implementation has three crucial facts in its favour.

**First, the finished letter cannot be encountered again through that old front slot.** It was the last remaining piece at the front of its order, and no surviving link leads back to the slot as that letter.

**Second, its corresponding position in the other order has already been replaced by the newly constructed cell.** The pairing and links have been updated to refer to the new structure.

**Third, everything the current operation still needs from the finished letter has already been read into registers.** The old slot's memory contents are no longer required.

Together, these facts establish that the slot can safely take on a new purpose.

Instead of advancing a cursor into fresh storage, the algorithm remembers the location of the slot it just consumed.

In the Rust implementation, two offsets, `fa` and `fb`, track the front slots in the respective orders. When one letter finishes and the other continues, the appropriate front offset becomes the promised location.

The surrounding induction remains essentially unchanged. It creates the same cells, constructs the same words and ultimately produces the same decompressed output.

The difference is that its slots are **reused rather than continually extended**.

That is why the name *slot recycling* is so precise.

## 6. How much does this save?

The builder [counted](https://github.com/lmmx/mibz2/pull/408) what happened across the 196 Wikidata blocks admitted by Cells in the benchmark corpus.

The result was substantial:

| Quantity | Before | After |
|---|---:|---:|
| Slots touched | 31,547,362 | 22,181,734 |
| Fresh slots required | 9,365,628 | 0 |
| Reduction in touched slots | — | **29.7%** |

<figure class="bf-fig">
<p class="bf-title">Slots the induction touches, over the 196 Wikidata blocks</p>
<svg viewBox="0 0 720 148" role="img" aria-label="A stacked bar: before, 22,181,734 letters' slots plus 9,365,628 fresh slots, 31,547,362 in all. After, the 22,181,734 letters' slots alone, 29.7% fewer, with no fresh slot. 9,365,628 slots of 20 bytes are 187.3 MB no longer touched."><text x="0" y="40" class="bf-ts">Before</text><rect x="92" y="22" width="421.9" height="30" rx="3" class="bf-slot"/><rect x="515.9" y="22" width="176.1" height="30" rx="3" class="bf-pill-ready"/><text x="100" y="42" class="bf-t">letters’ slots 22,181,734</text><text x="523.9" y="42" class="bf-pill-ready-t">fresh 9,365,628</text><text x="692" y="16" class="bf-t-strong" text-anchor="end">31,547,362</text><text x="0" y="90" class="bf-ts">After</text><rect x="92" y="72" width="421.9" height="30" rx="3" class="bf-slot"/><text x="100" y="92" class="bf-t">letters’ slots 22,181,734</text><text x="523.9" y="92" class="bf-t-green">−29.7%, no fresh slot</text><path d="M0 118H720" class="bf-rule"/><text x="0" y="140" class="bf-tm">9,365,628 slots × 20 B = 187.3 MB no longer touched, 0.89 B per output byte, over the 196 blocks</text></svg>
<figcaption>Counted, not timed: #408’s census of the benchmark’s Wikidata corpus, the 196 of its 431 blocks that Cells admits. The letters’ slots are two per letter, one in each order. Every unequal cell used to take one fresh slot.</figcaption>
</figure>

Each avoided slot is 20 bytes.

Across those blocks, this corresponds to approximately **187 million bytes of slot storage no longer touched**.

That doesn't mean precisely 187 million fewer bytes travel to DRAM. Cache lines, write allocation, reuse and eviction behaviour determine actual memory traffic.

But it does mean that millions of distinct slot locations are no longer needed.

The allocation can shrink too. The previous implementation reserved room for additional slots based on its cell budget. The new one needs only enough room for the original letters' slots in each order.

And this matters beyond the amount of memory allocated.

### Why fewer slots can mean faster decoding

Modern CPUs don't fetch arbitrary individual bytes directly from main memory for ordinary cached accesses. They work with cache lines, commonly 64 bytes.

A 20-byte slot is small, but enough slots spread over millions of addresses occupy a substantial working set.

When several workers decode different blocks simultaneously, each has its own induction state competing for space in the shared cache.

If their combined working sets are too large, useful data gets displaced, and later accesses may have to retrieve it from slower levels of the memory hierarchy.

Slot recycling attacks that problem directly.

The slot being recycled has only just been read, so its cache line may already be resident. The fresh location it replaces might otherwise require a first touch of a different line.

The immediate benefit isn't fewer mathematical operations. It's **less memory to keep resident and fewer distinct places to access**.

This explains why the change is especially interesting for parallel decoding.

## 7. What the benchmarks tell us

The initial paired CI benchmark measured the complete decoder, rather than just timing the induction function in isolation.

For Wikidata:

| Threads | Wall-time change | Peak memory change |
|---|---:|---:|
| 1 | +0.5% | −9.7% |
| 2 | −0.2% | −12.5% |
| 4 | **−1.7%** | **−14.6%** |

<figure class="bf-fig">
<p class="bf-title">The decoder on Wikidata, with and without slot recycling</p>
<svg viewBox="0 0 720 220" role="img" aria-label="Left: wall-time change with 95% intervals. At 1 thread +0.5% (−0.2%, +1.3%); at 2 threads −0.2% (−1.7%, +1.4%); at 4 threads −1.7% (−3.3%, −0.1%), the only interval that excludes zero. Right: peak memory from base to head, 33 to 30 MB (−9.7%), 71 to 62 MB (−12.5%) and 109 to 96 MB (−14.6%) at 1, 2 and 4 threads."><text x="0" y="14" class="bf-ts">Wall time, head against base</text><text x="380" y="14" class="bf-ts">Peak memory, base → head</text><text x="0" y="56" class="bf-t">1 thread</text><path d="M260.0 52H335.0M260.0 47v10M335.0 47v10" class="bf-quiet-ink"/><circle cx="295.0" cy="52" r="5" class="bf-head-ink"/><text x="295.0" y="41" class="bf-t-strong" text-anchor="middle">+0.5%</text><text x="380" y="56" class="bf-t">1 thread</text><rect x="460" y="39" width="55.0" height="11" rx="2" class="bf-track"/><rect x="460" y="53" width="50.0" height="11" rx="2" class="bf-green"/><text x="521.0" y="49" class="bf-tm">33 MB</text><text x="516.0" y="63" class="bf-t-green">30 MB, −9.7%</text><text x="0" y="100" class="bf-t">2 threads</text><path d="M185.0 96H340.0M185.0 91v10M340.0 91v10" class="bf-quiet-ink"/><circle cx="260.0" cy="96" r="5" class="bf-head-ink"/><text x="260.0" y="85" class="bf-t-strong" text-anchor="middle">−0.2%</text><text x="380" y="100" class="bf-t">2 threads</text><rect x="460" y="83" width="118.3" height="11" rx="2" class="bf-track"/><rect x="460" y="97" width="103.3" height="11" rx="2" class="bf-green"/><text x="584.3" y="93" class="bf-tm">71 MB</text><text x="569.3" y="107" class="bf-t-green">62 MB, −12.5%</text><text x="0" y="144" class="bf-t">4 threads</text><path d="M105.0 140H265.0M105.0 135v10M265.0 135v10" class="bf-act"/><circle cx="185.0" cy="140" r="5" class="bf-green"/><text x="185.0" y="129" class="bf-t-green" text-anchor="middle">−1.7%</text><text x="380" y="144" class="bf-t">4 threads</text><rect x="460" y="127" width="181.7" height="11" rx="2" class="bf-track"/><rect x="460" y="141" width="160.0" height="11" rx="2" class="bf-green"/><text x="647.7" y="137" class="bf-tm">109 MB</text><text x="626.0" y="151" class="bf-t-green">96 MB, −14.6%</text><path d="M270 30V162" class="bf-axis"/><path d="M70 162H370" class="bf-axis"/><path d="M70 162v4" class="bf-axis"/><text x="70" y="179" class="bf-ttiny" text-anchor="middle">−4%</text><path d="M120 162v4" class="bf-axis"/><text x="120" y="179" class="bf-ttiny" text-anchor="middle">−3%</text><path d="M170 162v4" class="bf-axis"/><text x="170" y="179" class="bf-ttiny" text-anchor="middle">−2%</text><path d="M220 162v4" class="bf-axis"/><text x="220" y="179" class="bf-ttiny" text-anchor="middle">−1%</text><path d="M270 162v4" class="bf-axis"/><text x="270" y="179" class="bf-ttiny" text-anchor="middle">0%</text><path d="M320 162v4" class="bf-axis"/><text x="320" y="179" class="bf-ttiny" text-anchor="middle">+1%</text><path d="M370 162v4" class="bf-axis"/><text x="370" y="179" class="bf-ttiny" text-anchor="middle">+2%</text><text x="220" y="195" class="bf-tm" text-anchor="middle">← faster · slower →</text><rect x="460" y="168" width="12" height="8" rx="2" class="bf-track"/><text x="478" y="176" class="bf-tm">base</text><rect x="520" y="168" width="12" height="8" rx="2" class="bf-green"/><text x="538" y="176" class="bf-tm">head, with #408</text></svg>
<figcaption>Measured: #408’s paired CI bench, 10 rounds on one runner (Intel Xeon Platinum 8573C, 4 CPUs on 2 cores), the whole decoder on the Wikidata corpus. Left: the change in median wall time with its 95% interval across the rounds; green where the interval excludes no change. Right: peak resident memory, each a reduction the bench reports as beyond noise. The <code>records</code> corpus was 3.4% slower at 4 threads in the same run.</figcaption>
</figure>

The four-thread wall-time improvement had a reported 95% confidence interval of −3.3% to −0.1%.

That is a modest but positive result on this runner. The one- and two-thread changes were indistinguishable from noise.

The memory reductions are much larger.

This pattern is consistent with the intended mechanism: reducing the working set matters more as concurrency increases.

But we should be precise about what has been established. The CI runner isn't the i9 workstation. It has a different cache hierarchy and different core-sharing conditions. The benchmark demonstrates a benefit on the tested machine, but it doesn't yet establish how much slot recycling helps the i9's known [concurrency plateau](2026-11-07-why-mibz2-stops-scaling.md).

There's also a secondary-workload caveat: the `records` corpus showed a four-thread regression in the same CI run. Correctness was preserved, but the change is not a universal speedup.

The valuable thing is that **the memory reduction is substantial without requiring additional expensive computation**. The extra bookkeeping is minimal, with the front positions already available to the induction.

## 8. Why this is different from field elision

The terminology initially caused some confusion because two related research ideas both used the word *elision*.

[MONODROMY](https://github.com/lmmx/mibz2/pull/410) investigated whether individual fields inside existing slots could be left unwritten or reconstructed from other information.

It found that approximately 25.5% of the induction's stored bytes were never subsequently read, and that certain mathematical identities could account for an even larger fraction.

That sounds promising, but almost every redundant field shares its 64-byte cache line with live information.

If you stop writing a redundant field while leaving the surrounding record layout unchanged, the processor may still need the same cache line.

So removing those field writes doesn't necessarily make the working set smaller.

**Slot recycling solves a different problem.**

It doesn't selectively remove fields from slots. It removes the need for whole additional slots.

That means entire ranges of memory no longer need to be touched, and the required working set can genuinely shrink.

This distinction is central:

- **Field elision:** store less information in the same locations.
- **Slot recycling:** use fewer locations to store the necessary information.
- **Equal-cell elision:** avoid constructing some cells altogether, replacing those operations with another reconstruction mechanism.

The third idea remains a separate mathematical opportunity. Slot recycling hasn't implemented it or disproved it.

## 9. Why the change is safe

Reusing memory in an algorithm implemented with unchecked offsets and `unsafe` Rust deserves particular care.

The crucial correctness obligation is not merely that a slot *looks* unused. It must be guaranteed that no later operation still needs its old contents.

The proof comes from the induction's ordering invariant: the consumed front slot has no surviving references as the old letter, and its necessary values have already been captured.

The implementation also narrows the slot regions to the exact number of original letters. Tests check that promised slots stay inside those regions, including cases where there is no spare room beyond them.

The builder compared the new induction against the original on exhaustive small inputs, accounting for the fact that physical slot offsets legitimately differ. The resulting words, rules, state and output agree.

The broader decoder tests, CRC checks and Miri validation passed before the change was merged.

This is important because the optimisation changes where state lives, **not what that state means**.

The resulting decoder remains subject to the same correctness requirements as before.

## 10. What this teaches us about improving mibz2

Slot recycling is a good example of why sophisticated algorithms still benefit from careful attention to mundane-looking implementation details.

The Cells algorithm is built on substantial mathematics: interval exchanges, Rauzy–Veech induction, permutation structure and compact representations of words.

That mathematics makes it possible to avoid much of the work required by conventional inverse-BWT traversal.

But the mathematical abstraction also introduces a physical representation: linked orders, slots, words and rules.

The performance of the resulting program depends on both levels.

An algorithm can have excellent asymptotic properties and still spend too much time managing its intermediate state.

Conversely, improving that state management can produce worthwhile gains without changing the underlying algorithm at all.

The particularly elegant aspect of slot recycling is that it derives the optimisation from a structural property of Cells itself.

**At every unequal induction step, the continued letter needs a future slot just as the exhausted letter releases one.**

Recognising that relationship removes a whole category of unnecessary memory use.

The successful change therefore sits at the intersection of mathematics and systems programming: a proof about object lifetimes becomes a more efficient memory layout, which becomes a measurable improvement in the complete decoder.

It also points towards the next questions.

MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could reduce the induction's modelled cache-line footprint by another 25%, although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier [failed 16-byte layout](https://github.com/lmmx/mibz2/pull/398) is a warning that compactness isn't automatically speed.

Meanwhile, measuring how memory traffic scales after slot recycling can help determine whether Cells is becoming competitive at higher thread counts.

Those are separate experiments. Slot recycling has already justified itself on its own evidence.

## The essential insight

**Slot recycling makes mibz2 better by exploiting the fact that the useful lifetime of one memory record ends exactly when another needs its space.**

It doesn't eliminate the induction steps. It doesn't change the text reconstruction. It doesn't require a more elaborate parallel algorithm.

It simply stops the program from touching memory it never needed to touch.

That's why a change that is small in code can remove nearly a third of the induction's touched slots — and why understanding the mathematical structure of an algorithm can yield practical improvements well beyond the mathematics itself.

---

*Implementation and evidence: [mibz2 PR #408](https://github.com/lmmx/mibz2/pull/408), [merged on 15 November](https://github.com/lmmx/mibz2/commit/381fbf1d5f76a7f1689e7546d254b7e2bfd664b6). For the distinction between slot recycling and field elision, see [MONODROMY's analysis, PR #410](https://github.com/lmmx/mibz2/pull/410).*
