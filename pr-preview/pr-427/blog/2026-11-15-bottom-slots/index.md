[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-427/blog/index.md>)

# Read once, stored once: one copy of each fragment's details, not two

Proving that each fragment's details are read only once lets mibz2 store them once, trimming its bookkeeping by 30% and its decoding time by up to 6%.

15 November 2026 · 12 min read · [Research result](<#6-how-much-faster-is-the-decoder>)

[The last post](<https://lmmx.github.io/mibz2/pr-preview/pr-427/blog/2026-11-15-slot-recycling/index.md>) followed Cells, one of mibz2's ways of rebuilding a block of bzip2 text. Cells cuts the block into fragments and keeps a small record for each one. The record says how long the fragment is, where its text is kept and how to reach its neighbours. Cells sees every fragment in two different arrangements at once, so each fragment had two records. Until now, both held the same length and the same text. This post shows why the second copy was never needed, and what mibz2 0.3.3 gained by dropping it.

## The idea in brief

**A fragment's details are read exactly once, and a short proof shows that one stored copy is always there when they are.** Cells works through both arrangements from one end, and reaches each fragment in each of them at some point. The details are read only at the first of those two visits. If the first visit is in the arrangement whose record keeps the details, they are read there and the other copy is never used. If it is in the other arrangement, the first has not reached the fragment yet. Its copy is still intact, and the decoder can read it from there.

So 0.3.3 keeps the details in one record only. The other record shrinks from 20 bytes to 8: it keeps its link to its neighbour, and a pointer to the full record. Here is what that gives, and what it costs:

- **30% less memory for the records**, counted on the benchmark's Wikidata corpus, and about a fifth less for the rebuild's whole working area, which also holds the text itself.
- **A faster decoder at one to four threads.** On a 10-core Intel desktop, 0.3.3 took 4.1%, 3.9% and 6.5% less wall time than 0.3.2 at one, two and four threads. On the project's 2-core CI runner, only the four-thread gain (4.2%) is beyond noise.
- **The price is an extra memory read.** On 53.7% of record reads, the processor must read the small record to learn where the full one is before it can read the details. That could have cost more than it saved; on these measurements it did not.
- **More threads can now usefully decode a block each.** mibz2 switches from giving each thread a block of its own to having threads share blocks at a fixed thread count, the *team threshold*. After 0.3.3, a block per thread was faster up to nine threads on the desktop, and a change moving the threshold is under review. That shift is the work of every improvement to Cells so far, not of this change alone.

[Section 1](<#1-why-does-every-fragment-need-two-records>) builds the two arrangements from a three-byte example, and why each fragment needs a record in both. Sections 2 to 4 show the new layout, why it is safe, and what the extra read costs. Sections 5 to 7 give the measurements: memory, speed, and the team threshold.

## 1\. Why does every fragment need two records?

bzip2 compresses each block of text after rearranging it with the Burrows–Wheeler transform (BWT). The BWT sorts all the rotations of the block and keeps the last byte of each, which tends to put equal bytes next to each other. To decompress, the decoder must undo that rearrangement.

Take the block `aab`. Sorted, its rotations are `aab`, `aba` and `baa`. The BWT's output is their last bytes: `baa`. Sorting the output gives `aab`, the first bytes of the same rotations. Number the sorted rotations 0 to 2 and call them *rows*. The decoder knows which byte of the output goes with which byte of the sorted list, because equal bytes keep their order. The output's first `a` is the sorted list's first `a`, its second `a` is the second, and `b` is `b`. This pairing is called LF, and following it from row to row spells out the block, one byte per step. That is the usual way to decode, one position at a time.

LF has a pattern that Cells uses. The output `baa` falls into *runs* of equal bytes: one `b`, then two `a`s. Within a run, consecutive rows go to consecutive rows, so each run moves as a whole. Name the runs B and A. In the output they lie in the order B, A. Where LF sends them, in the sorted list, they lie in the order A, B.

The block `aab`: the same two runs, in two orders

Worked by hand: the BWT’s output `baa` above, the same bytes sorted below. Each band joins a run to the rows LF sends it to. The bands cross because the runs swap places: B, A on top; A, B below.

These are the two arrangements. The code calls the runs as they lie in the output the **top order**, and the same runs where LF sends them the **bottom order**. There are still only two intervals, A and B; each has a place in both orders. Cells calls each such interval a *letter*: these are the fragments of the opening paragraph. Each letter has a length, the number of rows it covers. It also has a *word*, the text those rows stand for: at the start, A's word is `a` and B's is `b`.

Cells rebuilds the text by shrinking this picture rather than walking it. Each step looks at the letters at the right-hand end of both orders, the *fronts*. Here they are A on top, two rows long, and B below, one row long. The longer letter, A, gives up as many rows as the shorter one has, and B's word grows to cover the row A lost: A's `a` followed by B's own `b`, giving `ab`. The next step joins the two remaining one-row letters, and the word `aab` is the whole block. [The earlier post on Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-427/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) works this through in full. On a real block, a letter that covers one row can have a word thousands of bytes long. A step's result is called a *cell*, and it takes the place of the letters it was made from.

Cells keeps each order as a chain of small records called *slots*, one per letter in each order. The chain leads to the next letter at each front, and a link between a letter's two slots leads to its place in the other order. Before 0.3.3, both of a letter's slots had the same five 4-byte fields:

| Field | What it holds |
| --- | --- |
| Previous slot | the slot before this one in its order |
| Length | how many rows the letter covers |
| Word length and word offset | where its text is kept, and how long it is |
| Partner | the same letter's slot in the other order |

The previous-slot link and the partner say where this slot sits: its neighbour in its own order, and its counterpart in the other. The length and the word describe the letter itself, and they were stored in both slots.

## 2\. What changed in 0.3.3?

The top slot stays exactly as it was. The bottom slot keeps only two fields: its previous-slot link, and its partner, which is the letter's top slot. The length and word are no longer stored there; when the bottom front needs them, it reads them from the top slot.

A letter’s two slots, before and after 0.3.3

The layout as `src/cells.rs` documents it, drawn to scale: every field is a 4-byte offset or length, and the word is its length and offset, read together as one 8-byte load. Amber marks the fields the bottom slot used to repeat. After the change, the bottom slot’s second field is the letter’s top slot, and the green arrow is the read that replaces the copy.

Each letter used to cost 40 bytes of slots, 20 in each order. It now costs 28: 20 on top and 8 below. That is 0.700 of the slot memory before, and it matches the count on the Wikidata corpus exactly ([section 5](<#5-how-much-memory-does-it-remove>)).

## 3\. How do we know the top slot still has the details?

Cells reuses slots as it goes: when a letter is used up, its slots are overwritten with the new cell. So a fair worry is that the bottom slot might point at a top slot that has already been overwritten.

The answer is an invariant, proved in the project's [analysis of which stored bytes are ever read](<https://github.com/lmmx/mibz2/pull/410>) (its rule R4) and checked there with no disagreements in 31.5 million reads:

**A letter's details are read once, at whichever front reaches it first. Its slot in the other order is then overwritten by the letter's own first cell without being read.**

That leaves two cases.

Whichever front reaches a letter first, its top slot is intact

Conceptual: one letter, X, with a slot in each order. Each order is read from its right-hand end, its front, and the induction moves leftwards. Green marks where X’s length and word are read. In both cases they are read once, and from the top slot.

In the first case, the top front reaches the letter first. The details are read straight from the top slot. The bottom slot's old copy would never have been read, because the letter's first cell overwrites it.

In the second case, the bottom front reaches the letter first. The top front has not reached the letter yet, so nothing has overwritten its top slot. Cells follows the bottom slot's partner field, reads the length and word from the top slot, and goes on.

Neither case depends on which front arrives first, so the details can always live on the same side. The bottom slot cannot go altogether, though. The bottom order still needs its own chain of links, and each letter still needs a way to its place in the other order. Those two fields are the slot's own state, not copies.

## 4\. What does the saving cost?

Before, reading a bottom slot gave everything at once. Now the processor reads the bottom slot to learn which top slot to go to, and only then can it read the length and word. The second read's address depends on the first read's result. Such a *dependent load* cannot start until the first has finished, which limits how much work the processor can overlap.

Counted on the Wikidata corpus, 17.2 million of the 32.0 million slot reads come from the bottom front, so **53.7% of slot reads gain a dependent load**.

This had gone wrong before. An earlier layout with [16-byte slots](<https://github.com/lmmx/mibz2/pull/398>) added one dependent load per letter read and cost 4.4% more processor time at four threads. Three things make this one cheaper:

1. The bottom slot's link and partner arrive together in one 8-byte read.
2. The top slot it leads to is the one the same step writes the new cell into, so that memory is needed anyway.
3. The decoder already fetched slots 12 steps ahead of the front. It now fetches the top slot ahead through the bottom one, and the letter's word six steps after that.

The extra read is also off the chain that finds the next slot: the next bottom slot's address comes from the same 8-byte read as the partner. The change was therefore expected to pay mostly when several threads compete for memory, and to break about even on one thread.

## 5\. How much memory does it remove?

The count is from [the change's pull request](<https://github.com/lmmx/mibz2/pull/414>), on the benchmark's Wikidata corpus. Of its 431 blocks, Cells takes 196; the others go to mibz2's ordinary decoder.

The rebuild’s memory, before and after, on the Wikidata corpus

Counted, not timed: \#414’s count over the 196 blocks Cells decodes, at one thread. The working area is the slots, the words’ text and the rules; only the slots shrink.

The slots shrink by exactly the 40-to-28 ratio. The whole working area also holds the words' text and the rules that build long words from shorter ones, and those are unchanged. So the working area shrinks by less: it takes 21.6% fewer cache lines, and the median block's goes from 2.90 MiB to 2.27 MiB.

## 6\. How much faster is the decoder?

Two measurements compare 0.3.3 with 0.3.2, decoding the same Wikidata corpus. Both are paired: each round runs both versions in alternation, and the figure is the change in wall time with its 95% interval.

Wall time, 0.3.3 against 0.3.2, decoding the Wikidata corpus

Measured: 10 paired rounds on each machine. The CI runner is an AMD EPYC 9V45 with 4 virtual CPUs on 2 cores; the desktop is an Intel i9-10900K with 10 cores. Each mark is the change in wall time with its 95% interval; green where the interval excludes no change. The desktop’s record gives ratios, 0.3.2 ÷ 0.3.3: 1.043, 1.040 and 1.070 at one, two and four threads; the chart shows each as the share of wall time saved, 1 − 1/ratio.

The desktop's record gives ratios of wall times, which the chart converts to the project's usual form, the share of wall time saved. At four threads the desktop also used about 7% less processor time. At 8 to 20 threads nothing changed, because there mibz2 0.3.3 did not use Cells at all: it shared each block between a team of threads.

As expected, the gain is clearest with several blocks decoding at once. The desktop shows a gain at one and two threads too; the CI runner does not.

## 7\. Why does this move the team threshold?

mibz2 decodes a file's blocks in parallel in one of two ways. With few threads, each thread takes a block of its own and decodes it with Cells; these threads are *workers*. With many threads, threads share blocks in *teams*, which split one block's work between them. Teams cost more coordination, but they keep fewer blocks in memory at once. The thread count where mibz2 switches from workers to teams is the **team threshold**. It was set at six threads on 28 October, and not measured again until after 0.3.3.

Every saving in a worker's memory makes it cheaper to run more workers side by side. So after 0.3.3 the project measured the two directly on the same desktop: a block per thread against the shipped decoder, which used teams from six threads.

A block per thread against teams, on the Intel desktop after 0.3.3

Measured: 10 paired rounds on the i9-10900K, the shipped decoder (teams from six threads) against a block per thread. Below 1 the workers are faster; green where the 95% interval is wholly below 1. The record gives no interval at ten threads. CPU is the same ratio for processor time; the last column is peak memory, teams → workers.

Workers were faster at every count from six to nine threads, taking 13.9% less wall time at six and 3.6% less at nine. At ten threads they took 15.3% longer, and three teams of three keep the lead. Workers also use more memory, since more blocks are in flight: roughly double, though under 200 MB. At nine threads they used 12.4% more processor time. That is over the project's 10% limit on extra processor time for such a move, so the choice went to the maintainer. He chose workers below ten threads, where each thread has a core of its own, and [the change](<https://github.com/lmmx/mibz2/pull/425>) is now a pull request under review.

This run measures Cells as it is now, after every improvement since 0.3.0, including slot recycling in 0.3.2 and the smaller bottom slots in 0.3.3. It shows that today's Cells beats the teams up to nine threads on this machine. It does not show how much of the shift is due to the bottom slots alone: no run compared the two layouts at six to nine threads.

## What it teaches

The change began from a question about the algorithm, not the machine: when is each stored value actually read? Once the answer was "exactly once, at the first front to arrive", the layout followed. It is not ordinary deduplication, because the slots are rewritten all the time. What makes it safe is that the one remaining copy is guaranteed to be intact at the one moment it is read.

The same question applies wherever a structure keeps two views of the same item. If only one view's copy is ever read, the other can become a pointer. Whether that pays then depends on the extra read, which here was cheap because the memory it reaches was needed anyway.

---

*The change and its count are in [mibz2 PR \#414](<https://github.com/lmmx/mibz2/pull/414>). The CI and desktop measurements are in the [0.3.3 release notes](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md>), and the desktop run's record is [\#420](<https://github.com/lmmx/mibz2/pull/420>). The team threshold's change is [PR \#425](<https://github.com/lmmx/mibz2/pull/425>).*
