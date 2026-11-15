---
title: "Read once, stored once: one copy of each fragment's details, not two"
image: assets/social/blog-2026-11-15-bottom-slots.png
description: "Proving that each fragment's details are read only once lets mibz2 store them once, trimming its bookkeeping by 30% and its decoding time by up to 6%."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Read once, stored once: one copy of each fragment's details, not two

Proving that each fragment's details are read only once lets mibz2 store them once, trimming its bookkeeping by 30% and its decoding time by up to 6%.
{: .post-dek }

15 November 2026 · 11 min read · [Research result](#6-how-much-faster-is-the-decoder)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[The last post](2026-11-15-slot-recycling.md) showed how one of mibz2's ways of rebuilding a block of bzip2 text, called Cells, now hands each new fragment of text the record its finished neighbour has just vacated. Those records, called slots, are the decoder's bookkeeping: for every fragment, its length, where its text is kept, and how to reach the fragments around it. Cells keeps every fragment in two arrangements at once, so each fragment had two 20-byte slots, and both held the same length and the same text. This post is about removing that second copy. mibz2 0.3.3 shrinks the second slot to 8 bytes that point back at the first.

## The idea in brief

**The second copy was never needed, and a short proof shows why.** A fragment's details are read exactly once during the rebuild, from whichever of its two slots the algorithm reaches first. If that is the first slot, the second copy is never read. If it is the second, the first slot has not been touched yet, so it still holds the same details. Either way, one copy is enough.

So the second slot now keeps only what is genuinely its own: its link to the slot before it, and the position of its partner. When the algorithm needs the fragment's length and text there, it follows the partner and reads them from the first slot.

Here is what that gives, and what it costs:

- **30% less slot memory**, counted on the benchmark's Wikidata corpus: 443.6 MB of slots become 310.5 MB, and the whole working area of the rebuild shrinks by about a fifth.
- **A faster decoder at one to four threads.** On a 10-core Intel desktop, 0.3.3 took 4.1%, 3.9% and 6.5% less wall time than 0.3.2 at one, two and four threads. On the project's 2-core CI runner, only the four-thread gain (4.2%) is beyond noise.
- **The price is an extra memory read.** On 53.7% of slot reads, the processor must read one slot to learn where the other is before it can read the details. That could have cost more than it saved; on these measurements it did not.
- **More threads can now usefully decode a block each.** mibz2 switches from giving each thread its own block to sharing blocks between threads at a fixed thread count, the *team threshold*. After 0.3.3, one block per thread was faster up to nine threads on the desktop, and a change to move the threshold is now under review. That shift is the work of every improvement to Cells so far, not of this change alone.

Section 1 recaps Cells and its two arrangements. Sections 2 and 3 show the new layout and why it is safe. Section 4 weighs the extra read, and sections 5 to 7 give the measurements: memory, speed, and the team threshold.

## 1. Why does every fragment have two slots?

bzip2 compresses each block of text after rearranging it with the Burrows–Wheeler transform (BWT), which sorts the block's rotations so that equal bytes bunch together. To decompress, the decoder must undo that rearrangement. The usual way follows a chain through the block one position at a time, and each step must wait for the memory read before it.

Cells, introduced in [an earlier post](2026-11-10-rauzy-veech-reconstruction.md), works on whole stretches of the block instead. It cuts the block into pieces it calls *letters*. Each letter is a stretch of positions with a length, and a *word*: the output text that stretch stands for. Cells then repeatedly takes one letter from the front of each of two arrangements, the **top order** and the **bottom order**, and joins them into a finished piece, a *cell*, whose word is the two words put together. The shorter of the two letters is used up. The longer one goes on with what is left.

Both orders contain the same letters, arranged differently. So each letter has one slot in each order, and the two slots point at each other as *partners*. Before 0.3.3, both slots were laid out the same way, in five 4-byte fields:

| Field | What it holds |
|---|---|
| Previous slot | the slot before this one in its order |
| Length | how many positions the letter still covers |
| Word length and word offset | where its text is kept, and how long it is |
| Partner | the same letter's slot in the other order |

The length and the word are the letter's own details. The previous-slot link and the partner describe where this particular slot sits. Only the first kind was duplicated.

## 2. What changed in 0.3.3?

The top slot stays exactly as it was. The bottom slot keeps only two fields: its previous-slot link, and its partner, which is the letter's top slot. Its length and word are no longer stored there.

<figure class="bf-fig">
<p class="bf-title">A letter’s two slots, before and after 0.3.3</p>
<svg viewBox="0 0 480 396" role="img" aria-label="Before 0.3.3, each letter had a 20-byte top slot and a 20-byte bottom slot, each holding the slot before it (4 bytes), the letter's length (4 bytes), its word's length and offset (8 bytes) and its partner (4 bytes); the length and word were the same in both. In 0.3.3 the top slot is unchanged and the bottom slot is 8 bytes: the slot before it and the letter's top slot, from which the length and word are read. Per letter, 40 bytes become 28, 0.700 of before."><text x="84" y="12" class="bf-ttiny" text-anchor="middle">0</text><text x="156" y="12" class="bf-ttiny" text-anchor="middle">4</text><text x="228" y="12" class="bf-ttiny" text-anchor="middle">8</text><text x="372" y="12" class="bf-ttiny" text-anchor="middle">16</text><text x="444" y="12" class="bf-ttiny" text-anchor="middle">20</text><text x="76" y="12" class="bf-ttiny" text-anchor="end">byte</text><text x="0" y="40" class="bf-cap">Before</text><text x="0" y="69" class="bf-ts">Top</text><text x="0" y="85" class="bf-tm">20 B</text><rect x="85" y="52" width="70" height="38" rx="4" class="bf-slot"/><text x="120" y="69" class="bf-t-strong" text-anchor="middle">before</text><text x="120" y="83" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="157" y="52" width="70" height="38" rx="4" class="bf-slot"/><text x="192" y="69" class="bf-t-strong" text-anchor="middle">length</text><text x="192" y="83" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="229" y="52" width="142" height="38" rx="4" class="bf-slot"/><text x="300" y="69" class="bf-t-strong" text-anchor="middle">word</text><text x="300" y="83" class="bf-ttiny" text-anchor="middle">length, offset · 8 B</text><rect x="373" y="52" width="70" height="38" rx="4" class="bf-slot"/><text x="408" y="69" class="bf-t-strong" text-anchor="middle">partner</text><text x="408" y="83" class="bf-ttiny" text-anchor="middle">4 B</text><text x="0" y="115" class="bf-ts">Bottom</text><text x="0" y="131" class="bf-tm">20 B</text><rect x="85" y="98" width="70" height="38" rx="4" class="bf-slot"/><text x="120" y="115" class="bf-t-strong" text-anchor="middle">before</text><text x="120" y="129" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="157" y="98" width="70" height="38" rx="4" class="bf-pill-ready"/><text x="192" y="115" class="bf-t-strong" text-anchor="middle">length</text><text x="192" y="129" class="bf-ttiny" text-anchor="middle">copy</text><rect x="229" y="98" width="142" height="38" rx="4" class="bf-pill-ready"/><text x="300" y="115" class="bf-t-strong" text-anchor="middle">word</text><text x="300" y="129" class="bf-ttiny" text-anchor="middle">copy of the top's</text><rect x="373" y="98" width="70" height="38" rx="4" class="bf-slot"/><text x="408" y="115" class="bf-t-strong" text-anchor="middle">partner</text><text x="408" y="129" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="157" y="49" width="214" height="90" rx="6" class="bf-box" stroke-dasharray="2 3"/><text x="264" y="154" class="bf-tm" text-anchor="middle">the same length and word, stored twice</text><path d="M0 176H480" class="bf-rule"/><text x="0" y="202" class="bf-cap">After, in 0.3.3</text><text x="0" y="231" class="bf-ts">Top</text><text x="0" y="247" class="bf-tm">20 B</text><rect x="85" y="214" width="70" height="38" rx="4" class="bf-slot"/><text x="120" y="231" class="bf-t-strong" text-anchor="middle">before</text><text x="120" y="245" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="157" y="214" width="70" height="38" rx="4" class="bf-slot"/><text x="192" y="231" class="bf-t-strong" text-anchor="middle">length</text><text x="192" y="245" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="229" y="214" width="142" height="38" rx="4" class="bf-slot"/><text x="300" y="231" class="bf-t-strong" text-anchor="middle">word</text><text x="300" y="245" class="bf-ttiny" text-anchor="middle">length, offset · 8 B</text><rect x="373" y="214" width="70" height="38" rx="4" class="bf-slot"/><text x="408" y="231" class="bf-t-strong" text-anchor="middle">partner</text><text x="408" y="245" class="bf-ttiny" text-anchor="middle">4 B</text><text x="0" y="303" class="bf-ts">Bottom</text><text x="0" y="319" class="bf-t-green">8 B</text><rect x="85" y="286" width="70" height="38" rx="4" class="bf-cell-hi"/><text x="120" y="303" class="bf-t-strong" text-anchor="middle">before</text><text x="120" y="317" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="157" y="286" width="70" height="38" rx="4" class="bf-cell-hi"/><text x="192" y="303" class="bf-t-strong" text-anchor="middle">top slot</text><text x="192" y="317" class="bf-ttiny" text-anchor="middle">4 B</text><rect x="229" y="286" width="214" height="38" rx="4" class="bf-box" stroke-dasharray="4 3"/><text x="336" y="302" class="bf-tm" text-anchor="middle">gone</text><text x="336" y="317" class="bf-tm" text-anchor="middle">12 B no longer stored</text><path d="M192 284C192 264 228 278 228 256" class="bf-act"/><path d="M223 263L228 255L233 263Z" class="bf-head"/><text x="242" y="274" class="bf-t-green">length and word read from the top slot</text><path d="M0 346H480" class="bf-rule"/><text x="0" y="372" class="bf-ts">Per letter, both orders</text><text x="0" y="390" class="bf-tm">0.700 of the slot memory before</text><text x="480" y="382" class="bf-big-green" text-anchor="end">40 B → 28 B</text></svg>
<figcaption>The layout as <code>src/cells.rs</code> documents it, drawn to scale: every field is a 4-byte offset or length, and the word is its length and offset, read together as one 8-byte load. Amber marks the fields the bottom slot used to repeat. After the change, the bottom slot’s second field is the letter’s top slot, and the green arrow is the read that replaces the copy.</figcaption>
</figure>

Each letter used to cost 40 bytes of slots, 20 in each order. It now costs 28: 20 on top and 8 below. That is 0.700 of the slot memory before, and it matches the count on the Wikidata corpus exactly (section 5).

## 3. How do we know the top slot still has the details?

Cells overwrites slots as it goes: when a letter is used, its slots are reused for the new cell. So a fair objection is that the bottom slot might point at a top slot that has already been overwritten.

The answer is an invariant, proved in the project's [analysis of which stored bytes are ever read](https://github.com/lmmx/mibz2/pull/410) (its rule R4) and checked there with no disagreements in 31.5 million reads:

**A letter's details are read once, on whichever side the induction reaches first. The slot on the other side is then overwritten by the letter's own first cell without being read.**

That leaves two cases.

<figure class="bf-fig">
<p class="bf-title">Whichever front reaches a letter first, its top slot is intact</p>
<svg viewBox="0 0 480 404" role="img" aria-label="Two cases for a letter X. Case A: the top front reaches X first and reads its length and word from X's top slot; X's 8-byte bottom slot is never read for them. Case B: the bottom front reaches X first; the top front has not yet reached X, so X's top slot is still intact, and the bottom slot's partner field leads to it, where the length and word are read."><text x="0" y="14" class="bf-ts">Case A: the top front reaches X first</text><text x="0" y="31" class="bf-tm">The details are read from the top slot, where they live.</text><text x="0" y="67" class="bf-t">Top</text><text x="0" y="127" class="bf-t">Bottom</text><text x="131" y="67" class="bf-tl" text-anchor="middle">…</text><rect x="176" y="48" width="70" height="28" rx="4" class="bf-slot"/><text x="211" y="67" class="bf-t-strong" text-anchor="middle">D</text><rect x="256" y="48" width="70" height="28" rx="4" class="bf-cell-hi"/><text x="291" y="67" class="bf-t-strong" text-anchor="middle">X · 20 B</text><rect x="336" y="48" width="70" height="28" rx="4" class="bf-slot"/><text x="371" y="67" class="bf-t-strong" text-anchor="middle">B</text><text x="131" y="127" class="bf-tl" text-anchor="middle">…</text><rect x="176" y="108" width="70" height="28" rx="4" class="bf-track"/><text x="211" y="127" class="bf-t-strong" text-anchor="middle">X · 8 B</text><rect x="256" y="108" width="70" height="28" rx="4" class="bf-slot"/><text x="291" y="127" class="bf-t-strong" text-anchor="middle">C</text><rect x="336" y="108" width="70" height="28" rx="4" class="bf-slot"/><text x="371" y="127" class="bf-t-strong" text-anchor="middle">A</text><path d="M211.0 78L131.0 106" class="bf-arrow" stroke-dasharray="4 4"/><text x="211" y="30" class="bf-tm" text-anchor="middle">top front</text><path d="M211.0 34V45M207.0 40L211.0 45L215.0 40" class="bf-arrow"/><text x="371" y="164" class="bf-tm" text-anchor="middle">bottom front</text><path d="M371.0 150V139M367.0 144L371.0 139L375.0 144" class="bf-arrow"/><text x="418" y="60" class="bf-t-green">reads X's</text><text x="418" y="74" class="bf-t-green">details here</text><text x="418" y="120" class="bf-tm">X's bottom slot</text><text x="418" y="134" class="bf-tm">is never read</text><path d="M0 182H480" class="bf-rule"/><text x="0" y="210" class="bf-ts">Case B: the bottom front reaches X first</text><text x="0" y="227" class="bf-tm">The top front is still behind X, so X's top slot is intact.</text><text x="0" y="263" class="bf-t">Top</text><text x="0" y="323" class="bf-t">Bottom</text><text x="131" y="263" class="bf-tl" text-anchor="middle">…</text><rect x="176" y="244" width="70" height="28" rx="4" class="bf-slot"/><text x="211" y="263" class="bf-t-strong" text-anchor="middle">D</text><rect x="256" y="244" width="70" height="28" rx="4" class="bf-cell-hi"/><text x="291" y="263" class="bf-t-strong" text-anchor="middle">X · 20 B</text><rect x="336" y="244" width="70" height="28" rx="4" class="bf-slot"/><text x="371" y="263" class="bf-t-strong" text-anchor="middle">B</text><text x="131" y="323" class="bf-tl" text-anchor="middle">…</text><rect x="176" y="304" width="70" height="28" rx="4" class="bf-cell-hi"/><text x="211" y="323" class="bf-t-strong" text-anchor="middle">X · 8 B</text><rect x="256" y="304" width="70" height="28" rx="4" class="bf-slot"/><text x="291" y="323" class="bf-t-strong" text-anchor="middle">C</text><rect x="336" y="304" width="70" height="28" rx="4" class="bf-slot"/><text x="371" y="323" class="bf-t-strong" text-anchor="middle">A</text><path d="M211.0 274L131.0 302" class="bf-arrow" stroke-dasharray="4 4"/><text x="371" y="226" class="bf-tm" text-anchor="middle">top front</text><path d="M371.0 230V241M367.0 236L371.0 241L375.0 236" class="bf-arrow"/><text x="131" y="360" class="bf-tm" text-anchor="middle">bottom front</text><path d="M131.0 346V335M127.0 340L131.0 335L135.0 340" class="bf-arrow"/><path d="M141.0 301L205.0 279" class="bf-act"/><path d="M210.0 274L199.0 277L206.0 285Z" class="bf-head"/><text x="418" y="256" class="bf-t-green">untouched:</text><text x="418" y="270" class="bf-t-green">read from here</text><text x="418" y="316" class="bf-tm">follows its</text><text x="418" y="330" class="bf-tm">partner up</text><path d="M0 378H480" class="bf-rule"/><path d="M2 396H26" class="bf-arrow" stroke-dasharray="4 4"/><text x="34" y="400" class="bf-tm">partner: X's slot in the other order</text></svg>
<figcaption>Conceptual: one letter, X, with its slots in both orders; the induction reads each order from its front, the slot drawn under the arrow. Green marks where X’s length and word are read. In both cases they are read once, and from the top slot.</figcaption>
</figure>

In the first case, the top front reaches the letter first. The details are read straight from the top slot. The bottom slot's old copy would never have been read, because the letter's first cell overwrites it.

In the second case, the bottom front reaches the letter first. The top front has not reached the letter yet, so nothing has written to its top slot. Cells follows the bottom slot's partner field, reads the length and word from the top slot, and carries on.

Neither case depends on predicting which front arrives first. Either order is safe, so the details can always live on the same side. The bottom slot cannot disappear altogether, though: the bottom order still needs its own chain of links, and the two orders still need to know each other's positions. Those two fields are the slot's own state, not copies.

## 4. What does the saving cost?

Before, reading a bottom slot gave everything at once. Now the processor reads the bottom slot to learn which top slot to go to, and only then can it read the length and word. The second read's address depends on the first read's result. Such a *dependent load* cannot start until the first has finished, which limits how much work the processor can overlap.

Counted on the Wikidata corpus, 17.2 million of the 32.0 million slot reads come from the bottom side, so **53.7% of slot reads gain a dependent load**.

This had gone wrong before. An earlier layout with [16-byte slots](https://github.com/lmmx/mibz2/pull/398) added one dependent load per letter read and cost 4.4% more processor time at four threads. Three things make this one cheaper:

1. The bottom slot's link and partner arrive together in one 8-byte read.
2. The top slot it leads to is the one the same step writes the new cell into, so that memory is needed anyway.
3. The decoder already fetched slots 12 steps ahead. It now fetches the top slot ahead through the bottom one, and the letter's word six steps after that.

The extra read is also off the chain that finds the next slot: the next bottom slot's address comes from the same 8-byte read as the partner. So the saving was expected to outweigh the cost mostly when several threads compete for memory, and to break about even on one thread.

## 5. How much memory does it remove?

The count is from [the change's pull request](https://github.com/lmmx/mibz2/pull/414), on the benchmark's Wikidata corpus: 431 blocks, of which Cells takes 196 (the others go to mibz2's ordinary decoder).

<figure class="bf-fig">
<p class="bf-title">The rebuild’s memory, before and after, on the Wikidata corpus</p>
<svg viewBox="0 0 480 224" role="img" aria-label="Counted on the Wikidata corpus's 196 blocks that Cells decodes. Slot bytes in both orders: 443.6 MB before, 310.5 MB after, 0.700. Cache lines of the whole working area: 9.64 million before, 7.56 million after, 0.784. Median working area per block: 2.90 MiB before, 2.27 MiB after."><text x="0" y="14" class="bf-ts">Slot bytes, both orders</text><rect x="0" y="24" width="300" height="14" rx="2" class="bf-track"/><text x="308" y="35" class="bf-tm">before: 443.6 MB</text><rect x="0" y="42" width="210" height="14" rx="2" class="bf-green"/><text x="218" y="53" class="bf-t-green">310.5 MB · ×0.700</text><text x="0" y="92" class="bf-ts">64-byte lines of the working area</text><rect x="0" y="102" width="300" height="14" rx="2" class="bf-track"/><text x="308" y="113" class="bf-tm">before: 9.64 M</text><rect x="0" y="120" width="235.2" height="14" rx="2" class="bf-green"/><text x="243.2" y="131" class="bf-t-green">7.56 M · ×0.784</text><text x="0" y="170" class="bf-ts">Working area per block, median</text><rect x="0" y="180" width="300" height="14" rx="2" class="bf-track"/><text x="308" y="191" class="bf-tm">before: 2.90 MiB</text><rect x="0" y="198" width="234.828" height="14" rx="2" class="bf-green"/><text x="242.828" y="209" class="bf-t-green">2.27 MiB · ×0.783</text></svg>
<figcaption>Counted, not timed: #414’s count over the 196 blocks Cells decodes, at one thread. The working area is the slots, the words’ text and the rules; only the slots shrink.</figcaption>
</figure>

| | Before | After | After ÷ before |
|---|--:|--:|--:|
| Slot bytes, both orders | 443.6 MB | 310.5 MB | 0.700 |
| 64-byte lines of the whole working area | 9.64 M | 7.56 M | 0.784 |
| Working area per block, median | 2.90 MiB | 2.27 MiB | |

The slots shrink by exactly the 40-to-28 ratio. The whole working area, which also holds the words' text and the rules that build long words from shorter ones, shrinks by 21.6%, because those other parts are unchanged.

## 6. How much faster is the decoder?

Two measurements compare 0.3.3 with 0.3.2, decoding the same Wikidata corpus. Both are paired: each round runs both versions in alternation, and the figure is the change in wall time with its 95% interval.

<figure class="bf-fig">
<p class="bf-title">Wall time, 0.3.3 against 0.3.2, decoding the Wikidata corpus</p>
<svg viewBox="0 0 480 316" role="img" aria-label="Change in wall time from 0.3.2 to 0.3.3 on the Wikidata corpus, with 95% intervals. CI runner: −1.4% (−5.0, +2.4) at 1 thread, −1.2% (−4.5, +2.1) at 2, −4.2% (−4.9, −3.4) at 4. Intel desktop: −4.1% (−6.9, −1.3) at 1, −3.9% (−6.0, −1.7) at 2, −6.5% (−8.3, −4.8) at 4."><text x="0" y="18" class="bf-ts">CI runner, 2 cores</text><text x="0" y="44" class="bf-t">1 thread</text><path d="M264.3 40H433.4M264.3 35v10M433.4 35v10" class="bf-quiet-ink"/><circle cx="346.6" cy="40" r="5" class="bf-head-ink"/><text x="346.571" y="31" class="bf-t-strong" text-anchor="middle">−1.4%</text><text x="0" y="80" class="bf-t">2 threads</text><path d="M275.7 76H426.6M275.7 71v10M426.6 71v10" class="bf-quiet-ink"/><circle cx="351.1" cy="76" r="5" class="bf-head-ink"/><text x="351.143" y="67" class="bf-t-strong" text-anchor="middle">−1.2%</text><text x="0" y="116" class="bf-t">4 threads</text><path d="M266.6 112H300.9M266.6 107v10M300.9 107v10" class="bf-act"/><circle cx="282.6" cy="112" r="5" class="bf-green"/><text x="282.571" y="103" class="bf-t-green" text-anchor="middle">−4.2%</text><text x="0" y="154" class="bf-ts">Intel desktop, 10 cores</text><text x="0" y="180" class="bf-t">1 thread</text><path d="M220.9 176H348.9M220.9 171v10M348.9 171v10" class="bf-act"/><circle cx="284.9" cy="176" r="5" class="bf-green"/><text x="284.857" y="167" class="bf-t-green" text-anchor="middle">−4.1%</text><text x="0" y="216" class="bf-t">2 threads</text><path d="M241.4 212H339.7M241.4 207v10M339.7 207v10" class="bf-act"/><circle cx="289.4" cy="212" r="5" class="bf-green"/><text x="289.429" y="203" class="bf-t-green" text-anchor="middle">−3.9%</text><text x="0" y="252" class="bf-t">4 threads</text><path d="M188.9 248H268.9M188.9 243v10M268.9 243v10" class="bf-act"/><circle cx="230.0" cy="248" r="5" class="bf-green"/><text x="230" y="239" class="bf-t-green" text-anchor="middle">−6.5%</text><path d="M378.6 28V276" class="bf-axis"/><path d="M150 276H470" class="bf-axis"/><path d="M150.0 276v4" class="bf-axis"/><text x="150" y="293" class="bf-ttiny" text-anchor="middle">−10%</text><path d="M195.7 276v4" class="bf-axis"/><text x="195.714" y="293" class="bf-ttiny" text-anchor="middle">−8%</text><path d="M241.4 276v4" class="bf-axis"/><text x="241.429" y="293" class="bf-ttiny" text-anchor="middle">−6%</text><path d="M287.1 276v4" class="bf-axis"/><text x="287.143" y="293" class="bf-ttiny" text-anchor="middle">−4%</text><path d="M332.9 276v4" class="bf-axis"/><text x="332.857" y="293" class="bf-ttiny" text-anchor="middle">−2%</text><path d="M378.6 276v4" class="bf-axis"/><text x="378.571" y="293" class="bf-ttiny" text-anchor="middle">0%</text><path d="M424.3 276v4" class="bf-axis"/><text x="424.286" y="293" class="bf-ttiny" text-anchor="middle">+2%</text><path d="M470.0 276v4" class="bf-axis"/><text x="470" y="293" class="bf-ttiny" text-anchor="middle">+4%</text><text x="150" y="310" class="bf-tm">← less wall time with 0.3.3</text></svg>
<figcaption>Measured: 10 paired rounds on each machine. The CI runner is an AMD EPYC 9V45 with 4 virtual CPUs on 2 cores; the desktop is an Intel i9-10900K. Each mark is the change in wall time with its 95% interval; green where the interval excludes no change.</figcaption>
</figure>

| Threads | CI runner (2 cores) | Intel desktop (10 cores) |
|--:|--:|--:|
| 1 | −1.4% (−5.0, +2.4) | −4.1% (−6.9, −1.3) |
| 2 | −1.2% (−4.5, +2.1) | −3.9% (−6.0, −1.7) |
| 4 | **−4.2% (−4.9, −3.4)** | **−6.5% (−8.3, −4.8)** |

The CI runner is an AMD EPYC 9V45 with 4 virtual CPUs on 2 cores. The desktop is an Intel i9-10900K. Its figures are reported as wall-time ratios (0.3.2 ÷ 0.3.3: 1.043, 1.040 and 1.070); the table converts them to the project's usual form, the share of wall time saved. At four threads the desktop also used about 7% less processor time. At 8 to 20 threads nothing changed, because there mibz2 0.3.3 did not use Cells at all: it shared each block between a team of threads.

As expected, the gain is clearest with several workers running at once. The desktop shows a gain at one and two threads too; the CI runner does not.

## 7. Why does this move the team threshold?

mibz2 decodes a file's blocks in parallel in two ways. With few threads, each thread takes a block of its own and decodes it with Cells; these are *workers*. With many threads, threads share blocks in *teams*, which split one block's work between them. Teams cost more coordination, but they keep fewer blocks in memory at once. The thread count where mibz2 switches from workers to teams is the **team threshold**. It was set at six threads on 28 October, before most of Cells' memory savings.

Every saving in each worker's memory makes it cheaper to run more workers side by side. So after 0.3.3, the project measured the two directly on the same desktop: workers against the shipped decoder, which used teams from six threads.

<figure class="bf-fig">
<p class="bf-title">One block per thread against teams, on the Intel desktop after 0.3.3</p>
<svg viewBox="0 0 480 266" role="img" aria-label="On the Intel desktop after 0.3.3, wall time of one block per thread (workers) divided by the shipped decoder's teams: 0.861 at 6 threads, 0.889 at 7, 0.923 at 8, 0.964 at 9, each interval below 1, and 1.153 at 10 threads, where three teams of three are faster. CPU ratios 0.88, 1.00, 0.97, 1.12 and 1.34. Peak memory, teams to workers: 67 to 133, 65 to 149, 81 to 168, 80 to 188 and 73 to 205 MB."><text x="0" y="14" class="bf-tm">Threads</text><text x="186" y="14" class="bf-tm" text-anchor="middle">wall time, workers ÷ teams</text><text x="352" y="14" class="bf-tm" text-anchor="middle">CPU</text><text x="480" y="14" class="bf-tm" text-anchor="end">peak MB</text><text x="0" y="48" class="bf-ts">6</text><path d="M101.6 44H112.5M101.6 39v10M112.5 39v10" class="bf-act"/><circle cx="106.8" cy="44" r="5" class="bf-green"/><text x="106.77" y="35" class="bf-t-green" text-anchor="middle">0.861</text><text x="352" y="48" class="bf-t" text-anchor="middle">0.88</text><text x="480" y="48" class="bf-t" text-anchor="end">67 → 133</text><text x="0" y="88" class="bf-ts">7</text><path d="M112.5 84H133.6M112.5 79v10M133.6 79v10" class="bf-act"/><circle cx="122.7" cy="84" r="5" class="bf-green"/><text x="122.73" y="75" class="bf-t-green" text-anchor="middle">0.889</text><text x="352" y="88" class="bf-t" text-anchor="middle">1.00</text><text x="480" y="88" class="bf-t" text-anchor="end">65 → 149</text><text x="0" y="128" class="bf-ts">8</text><path d="M137.6 124H146.1M137.6 119v10M146.1 119v10" class="bf-act"/><circle cx="142.1" cy="124" r="5" class="bf-green"/><text x="142.11" y="115" class="bf-t-green" text-anchor="middle">0.923</text><text x="352" y="128" class="bf-t" text-anchor="middle">0.97</text><text x="480" y="128" class="bf-t" text-anchor="end">81 → 168</text><text x="0" y="168" class="bf-ts">9</text><path d="M160.9 164H170.0M160.9 159v10M170.0 159v10" class="bf-act"/><circle cx="165.5" cy="164" r="5" class="bf-green"/><text x="165.48" y="155" class="bf-t-green" text-anchor="middle">0.964</text><text x="352" y="168" class="bf-t-strong" text-anchor="middle">1.12</text><text x="480" y="168" class="bf-t" text-anchor="end">80 → 188</text><text x="0" y="208" class="bf-ts">10</text><circle cx="273.2" cy="204" r="5" class="bf-head-ink"/><text x="273.21" y="195" class="bf-t-strong" text-anchor="middle">1.153</text><text x="352" y="208" class="bf-t-strong" text-anchor="middle">1.34</text><text x="480" y="208" class="bf-t" text-anchor="end">73 → 205</text><path d="M186.0 26V226" class="bf-axis"/><path d="M72 226H300" class="bf-axis"/><path d="M72.0 226v4" class="bf-axis"/><text x="72" y="243" class="bf-ttiny" text-anchor="middle">0.8</text><path d="M129.0 226v4" class="bf-axis"/><text x="129" y="243" class="bf-ttiny" text-anchor="middle">0.9</text><path d="M186.0 226v4" class="bf-axis"/><text x="186" y="243" class="bf-ttiny" text-anchor="middle">1.0</text><path d="M243.0 226v4" class="bf-axis"/><text x="243" y="243" class="bf-ttiny" text-anchor="middle">1.1</text><path d="M300.0 226v4" class="bf-axis"/><text x="300" y="243" class="bf-ttiny" text-anchor="middle">1.2</text><text x="72" y="260" class="bf-t-green">← workers faster</text><text x="300" y="260" class="bf-tm" text-anchor="end">teams faster →</text></svg>
<figcaption>Measured: 10 paired rounds on the i9-10900K, the shipped decoder (teams from six threads) against one block per thread. Below 1 the workers are faster; green where the 95% interval is wholly below 1. The record gives no interval at ten threads. CPU is the same ratio for processor time; the last column is peak memory, teams → workers.</figcaption>
</figure>

| Threads | Workers ÷ teams, wall | Workers' saving | CPU ratio | Peak memory, teams → workers |
|--:|--:|--:|--:|--:|
| 6 | 0.861 | 13.9% | 0.884 | 67 → 133 MB |
| 7 | 0.889 | 11.1% | 0.997 | 65 → 149 MB |
| 8 | 0.923 | 7.7% | 0.967 | 81 → 168 MB |
| 9 | 0.964 | 3.6% | 1.124 | 80 → 188 MB |
| 10 | 1.153 | −15.3% | 1.341 | 73 → 205 MB |

Workers were faster at every count from six to nine threads, and slower at ten, where three teams of three keep the lead. They also use more memory, since more blocks are in flight: roughly double, though under 200 MB. At nine threads they used 12.4% more processor time, which put the choice to the maintainer. He chose to move the threshold to ten threads, and [the change](https://github.com/lmmx/mibz2/pull/425) is now a pull request under review.

This run measures Cells as it now is, after every improvement since 0.3.0, including slot recycling in 0.3.2 and the smaller bottom slots in 0.3.3. It shows that the current Cells beats the teams up to nine threads. It does not show how much of that shift is due to the bottom slots alone: no run compared the two layouts at six to nine threads.

## What it teaches

The change began from a question about the algorithm, not the machine: when is each stored value actually read? Once the answer was "exactly once, on one side", the layout followed. This is not ordinary deduplication, because the slots are being rewritten all the time; what makes it safe is that the one remaining copy is guaranteed to be intact at the one moment it is read.

The same question applies wherever a structure keeps two views of the same item. If the proof shows that only one view's copy is ever read, the other can become a reference. Whether that pays then depends on the extra read, which here was cheap because the referenced memory was needed anyway.

---

*Implementation and evidence: [mibz2 PR #414](https://github.com/lmmx/mibz2/pull/414) (the change and its count), the [0.3.3 release notes](https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md) (the CI and desktop measurements, [#420](https://github.com/lmmx/mibz2/pull/420)), and [PR #425](https://github.com/lmmx/mibz2/pull/425) (the team threshold).*
