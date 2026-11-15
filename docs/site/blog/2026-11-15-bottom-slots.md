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

> TODO figure: before/after slot layout (40 B → 28 B per letter).

Each letter used to cost 40 bytes of slots, 20 in each order. It now costs 28: 20 on top and 8 below. That is 0.700 of the slot memory before, and it matches the count on the Wikidata corpus exactly (section 5).

## 3. How do we know the top slot still has the details?

Cells overwrites slots as it goes: when a letter is used, its slots are reused for the new cell. So a fair objection is that the bottom slot might point at a top slot that has already been overwritten.

The answer is an invariant, proved in the project's [analysis of which stored bytes are ever read](https://github.com/lmmx/mibz2/pull/410) (its rule R4) and checked there with no disagreements in 31.5 million reads:

**A letter's details are read once, on whichever side the induction reaches first. The slot on the other side is then overwritten by the letter's own first cell without being read.**

That leaves two cases.

> TODO figure: case A (top front first) and case B (bottom front first).

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

> TODO figure: slot bytes 443.6 → 310.5 MB; induction buffer lines 9.64 M → 7.56 M; median buffer per block 2.90 → 2.27 MiB.

| | Before | After | After ÷ before |
|---|--:|--:|--:|
| Slot bytes, both orders | 443.6 MB | 310.5 MB | 0.700 |
| 64-byte lines of the whole working area | 9.64 M | 7.56 M | 0.784 |
| Working area per block, median | 2.90 MiB | 2.27 MiB | |

The slots shrink by exactly the 40-to-28 ratio. The whole working area, which also holds the words' text and the rules that build long words from shorter ones, shrinks by 21.6%, because those other parts are unchanged.

## 6. How much faster is the decoder?

Two measurements compare 0.3.3 with 0.3.2, decoding the same Wikidata corpus. Both are paired: each round runs both versions in alternation, and the figure is the change in wall time with its 95% interval.

> TODO figure: CI and i9 wall-time change with intervals at 1, 2, 4 threads.

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

> TODO figure: workers ÷ shipped wall at 6–10 threads, with CPU and peak memory.

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
