---
title: "Compact navigation tables for the inverse BWT: early findings"
image: assets/social/blog-2026-10-29-compact-navigation-tables.png
description: "Most of the inverse BWT's navigation table follows simple patterns: what a census of real blocks found, and what comes next."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Compact navigation tables for the inverse BWT: early findings

Most of the inverse BWT's navigation table follows simple patterns: what a census of real
blocks found, and what comes next.
{: .post-dek }

29 October 2026 · 10 min read · [Work in progress](#what-comes-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

This research began with a simple question:

> Can succinct data structures make my bzip2 decoder faster?

The answer so far is that the large table the decoder uses to reconstruct text stores more than it needs to, and much of what it stores follows simple patterns.

This is no longer only theory: a census of the benchmark corpus's actual data shows those patterns.

What follows starts from the beginning and assumes no background in compressed indexes.

## 1. The decoder follows a chain of pointers

When bzip2 compresses some text, it performs the Burrows–Wheeler Transform (BWT).

The BWT rearranges the characters in a way that helps compression. But recovering the original text requires following a particular sequence of positions.

Imagine an enormous collection of numbered cards.

Each card tells you which card to visit next.

<figure class="bf-fig bf-chain" data-next="3 5 0 6 1 2 4">
<p class="bf-title">A miniature pointer chain</p>
<ol class="bf-cards" aria-label="Seven cards, each holding the number of the next card"><li class="bf-cardx is-current"><span class="bf-cl">Card 0</span><span class="bf-cv">3</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 1</span><span class="bf-cv">5</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 2</span><span class="bf-cv">0</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 3</span><span class="bf-cv">6</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 4</span><span class="bf-cv">1</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 5</span><span class="bf-cv">2</span><span class="bf-ca" aria-hidden="true">→</span></li><li class="bf-cardx"><span class="bf-cl">Card 6</span><span class="bf-cv">4</span><span class="bf-ca" aria-hidden="true">→</span></li></ol>
<div class="bf-walk">
<div class="bf-walk-head" aria-live="polite" hidden><span>Currently at card <strong class="bf-at">0</strong></span><span class="bf-count">1 of 7 visited</span></div>
<div class="bf-buttons" hidden><button type="button" class="bf-btn bf-follow">Follow pointer →</button><button type="button" class="bf-btn bf-reset">Reset</button></div>
<p class="bf-path">Path: <span class="bf-trail">0 → 3 → 6 → 4 → 1 → 5 → 2</span></p>
</div>
<figcaption>Each card contains the number of the next card. The highlighted card is the current position: follow the pointers from card 0 until every card has been visited.</figcaption>
</figure>

Your decoder uses a table called the T-vector, which is essentially a much larger, more sophisticated collection of such cards.

For each position, it stores:

- Where to go next.
- The character associated with that position.
- A marker used by your cooperative traversal algorithm.

At maximum bzip2 block size, this table occupies approximately 3.6 MB.

mibz2's 32-lane implementation already tackles one major cost: instead of following one chain and repeatedly waiting for memory, you follow many independent chains at once.

That's memory-level parallelism (MLP).

So far, so good.

But then we asked a different question.

> Why do we need four bytes for every card?

## 2. The cards aren't arbitrary

Suppose I give you these two sequences of destinations:

<figure class="bf-fig">
<p class="bf-title">Two groups of eight pointers</p>
<svg viewBox="0 0 720 176" role="img" aria-label="Group A: 427, 18, 903, 61, 712, 295, 4, 846. Group B: 427 to 434, consecutive."><text x="0" y="18" class="bf-tm">Group A: arbitrary values</text><rect x="0" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="42.375" y="52" class="bf-t-strong" text-anchor="middle">427</text><rect x="90.75" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="133.125" y="52" class="bf-t-strong" text-anchor="middle">18</text><rect x="181.5" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="223.875" y="52" class="bf-t-strong" text-anchor="middle">903</text><rect x="272.25" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="314.625" y="52" class="bf-t-strong" text-anchor="middle">61</text><rect x="363" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="405.375" y="52" class="bf-t-strong" text-anchor="middle">712</text><rect x="453.75" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="496.125" y="52" class="bf-t-strong" text-anchor="middle">295</text><rect x="544.5" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="586.875" y="52" class="bf-t-strong" text-anchor="middle">4</text><rect x="635.25" y="28" width="84.75" height="38" rx="6" class="bf-cell"/><text x="677.625" y="52" class="bf-t-strong" text-anchor="middle">846</text><text x="0" y="96" class="bf-tm">Group B: consecutive values</text><rect x="0" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="42.375" y="130" class="bf-t-strong" text-anchor="middle">427</text><rect x="90.75" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="133.125" y="130" class="bf-t-strong" text-anchor="middle">428</text><rect x="181.5" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="223.875" y="130" class="bf-t-strong" text-anchor="middle">429</text><rect x="272.25" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="314.625" y="130" class="bf-t-strong" text-anchor="middle">430</text><rect x="363" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="405.375" y="130" class="bf-t-strong" text-anchor="middle">431</text><rect x="453.75" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="496.125" y="130" class="bf-t-strong" text-anchor="middle">432</text><rect x="544.5" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="586.875" y="130" class="bf-t-strong" text-anchor="middle">433</text><rect x="635.25" y="106" width="84.75" height="38" rx="6" class="bf-cell-hi"/><text x="677.625" y="130" class="bf-t-strong" text-anchor="middle">434</text><text x="0" y="170" class="bf-tm">Group B in one line: base 427, then add the position (+0, +1, … +7)</text></svg>
</figure>

For Group A, storing eight values individually is reasonable.

But Group B?

You only need to remember:

> Start at 427 and add the position within the group.

One starting number replaces eight separate numbers.

The permutation your inverse BWT decoder follows, called Ψ (psi), isn't a completely arbitrary permutation.

Because of how the BWT is constructed, it consists of increasing sequences of positions.

That means consecutive entries within a particular character group always point to increasing positions.

They won't necessarily be consecutive like Group B—but they're at least sorted.

And that gives us opportunities to compress them.

This is where succinct data structures entered the picture.

## 3. Succinct structures are about storing the pattern, not the individual facts

In ordinary programming, we tend to store arrays of values directly.

A succinct representation asks:

> Can I describe the arrangement of these values more economically and reconstruct any particular value when needed?

For example, rather than storing:

```
1000, 1002, 1005, 1008, 1010
```

you might store:

```
base = 1000
```

and:

```
offsets = 0, 2, 5, 8, 10
```

The offsets are much smaller than the absolute values, so they may require fewer bits.

This is called frame-of-reference encoding, or FOR.

Another technique, Elias–Fano, is especially suited to sorted integer sequences. It separates the higher and lower parts of each number and compresses the high parts using the sequence's monotonicity.

Neither is magic; you save memory at the cost of some work to recover an individual value.

And that creates our main tension:

> A more compact table is useful only if reconstructing its pointers doesn't make the decoder slower than before.

## 4. What the census found

Initially, we had no idea how well these representations would fit your actual BWT data.

Perhaps the values were increasing but widely scattered, making compression awkward.

So Claude wrote a census program to inspect them.

It found more structure than the theory alone guarantees.

<figure class="bf-fig">
<p class="bf-title">What Claude reported about your benchmark corpus</p>
<svg viewBox="0 0 720 150" role="img" aria-label="77.6% of rows are in simple consecutive groups; 99.51% fit compact FOR or Elias–Fano records."><text x="0" y="20" class="bf-ts">Simple consecutive groups</text><text x="720" y="22" class="bf-big-green" text-anchor="end">77.6%</text><rect x="0" y="32" width="720" height="13" rx="6.5" class="bf-track"/><rect x="0" y="32" width="558.72" height="13" rx="6.5" class="bf-green"/><text x="0" y="64" class="bf-tm">Rows where a complete group of pointers can potentially be represented by one base plus a simple offset.</text><text x="0" y="104" class="bf-ts">Fit compact FOR or Elias–Fano</text><text x="720" y="106" class="bf-big" text-anchor="end">99.51%</text><rect x="0" y="116" width="720" height="13" rx="6.5" class="bf-track"/><rect x="0" y="116" width="716.472" height="13" rx="6.5" class="bf-steel"/><text x="0" y="148" class="bf-tm">Only 0.49% of rows required exceptions under the proposed 32-value encoding scheme.</text><path d="M0 78.5H720" class="bf-rule"/></svg>
<figcaption>These two percentages overlap: they aren't two separate populations.</figcaption>
</figure>

The important observation is that more than three-quarters of the measured rows may be represented using a simple arithmetic rule.

That is much better than we could have assumed from the mathematics alone.

It's the difference between saying:

> This encoding should work in principle.

And:

> We've examined the actual pointers, and most of them exhibit precisely the structure this encoding is designed to exploit.

It does not yet mean the decoder will be faster. But it establishes that there is something real to exploit.

## 5. Why the CPU's cache lines became central

Now we reach the hardware connection.

Modern CPUs don't fetch arbitrary isolated bytes from main memory one at a time. They work with cache lines, commonly 64 bytes.

Imagine a cache line as a little tray of data.

If your next pointer lives in that tray, the processor can fetch the tray and extract what it needs.

The first design asks:

> What if we could store 32 pointers in exactly one 64-byte cache line?

<figure class="bf-fig">
<p class="bf-title">The fixed-cache-line proposal</p>
<svg viewBox="0 0 720 232" role="img" aria-label="One 64-byte cache-line record holding 32 pointers: 2 bytes per pointer on average."><rect x="0.5" y="0.5" width="719" height="178" rx="8" class="bf-box"/><text x="16" y="26" class="bf-tm">One cache-line record</text><text x="704" y="27" class="bf-ts" text-anchor="end">64 bytes</text><rect x="16" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="57.6875" y="56" class="bf-ttiny" text-anchor="middle">0</text><rect x="102.375" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="144.062" y="56" class="bf-ttiny" text-anchor="middle">1</text><rect x="188.75" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="230.438" y="56" class="bf-ttiny" text-anchor="middle">2</text><rect x="275.125" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="316.812" y="56" class="bf-ttiny" text-anchor="middle">3</text><rect x="361.5" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="403.188" y="56" class="bf-ttiny" text-anchor="middle">4</text><rect x="447.875" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="489.562" y="56" class="bf-ttiny" text-anchor="middle">5</text><rect x="534.25" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="575.938" y="56" class="bf-ttiny" text-anchor="middle">6</text><rect x="620.625" y="40" width="83.375" height="24" rx="3" class="bf-slot"/><text x="662.312" y="56" class="bf-ttiny" text-anchor="middle">7</text><rect x="16" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="57.6875" y="83" class="bf-ttiny" text-anchor="middle">8</text><rect x="102.375" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="144.062" y="83" class="bf-ttiny" text-anchor="middle">9</text><rect x="188.75" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="230.438" y="83" class="bf-ttiny" text-anchor="middle">10</text><rect x="275.125" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="316.812" y="83" class="bf-ttiny" text-anchor="middle">11</text><rect x="361.5" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="403.188" y="83" class="bf-ttiny" text-anchor="middle">12</text><rect x="447.875" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="489.562" y="83" class="bf-ttiny" text-anchor="middle">13</text><rect x="534.25" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="575.938" y="83" class="bf-ttiny" text-anchor="middle">14</text><rect x="620.625" y="67" width="83.375" height="24" rx="3" class="bf-slot"/><text x="662.312" y="83" class="bf-ttiny" text-anchor="middle">15</text><rect x="16" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="57.6875" y="110" class="bf-ttiny" text-anchor="middle">16</text><rect x="102.375" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="144.062" y="110" class="bf-ttiny" text-anchor="middle">17</text><rect x="188.75" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="230.438" y="110" class="bf-ttiny" text-anchor="middle">18</text><rect x="275.125" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="316.812" y="110" class="bf-ttiny" text-anchor="middle">19</text><rect x="361.5" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="403.188" y="110" class="bf-ttiny" text-anchor="middle">20</text><rect x="447.875" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="489.562" y="110" class="bf-ttiny" text-anchor="middle">21</text><rect x="534.25" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="575.938" y="110" class="bf-ttiny" text-anchor="middle">22</text><rect x="620.625" y="94" width="83.375" height="24" rx="3" class="bf-slot"/><text x="662.312" y="110" class="bf-ttiny" text-anchor="middle">23</text><rect x="16" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="57.6875" y="137" class="bf-ttiny" text-anchor="middle">24</text><rect x="102.375" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="144.062" y="137" class="bf-ttiny" text-anchor="middle">25</text><rect x="188.75" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="230.438" y="137" class="bf-ttiny" text-anchor="middle">26</text><rect x="275.125" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="316.812" y="137" class="bf-ttiny" text-anchor="middle">27</text><rect x="361.5" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="403.188" y="137" class="bf-ttiny" text-anchor="middle">28</text><rect x="447.875" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="489.562" y="137" class="bf-ttiny" text-anchor="middle">29</text><rect x="534.25" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="575.938" y="137" class="bf-ttiny" text-anchor="middle">30</text><rect x="620.625" y="121" width="83.375" height="24" rx="3" class="bf-slot"/><text x="662.312" y="137" class="bf-ttiny" text-anchor="middle">31</text><text x="16" y="166" class="bf-tm">32 logical pointers represented inside a single physical record.</text><text x="0" y="222" class="bf-tl">Average space per pointer</text><text x="720" y="224" class="bf-big" text-anchor="end">2 bytes</text></svg>
</figure>

Your current representation uses four bytes per pointer.

So the new design could halve the navigation table's storage, before exception overhead.

And crucially, the record can be found immediately from the current row number.

No searching for where the record was stored.

This is very important because your existing decoder's greatest strength is how quickly it can issue independent memory requests.

A compact representation that introduces multiple dependent searches could destroy that advantage.

The fixed-size record tries to avoid precisely that.

## 6. A second design: a two-tier layout

If 77.6% of rows belong to simple consecutive groups, why devote an entire 64-byte record to each of those groups?

Most of the record would contain information we don't actually need.

So we proposed a two-tier layout.

<figure class="bf-fig">
<p class="bf-title">Two-tier Ψ navigation</p>
<svg viewBox="0 0 720 312" role="img" aria-label="The row number selects an 8-byte directory entry. For a translation group the answer is base plus offset, from the entry alone; for other groups an extra record is loaded and decoded."><rect x="0" y="0" width="720" height="150" rx="10" class="bf-panel"/><text x="360" y="30" class="bf-ts" text-anchor="middle">Current row number</text><path d="M0 0v9m-4-4 4 4 4-4" transform="translate(360 40)" class="bf-arrow"/><rect x="16.5" y="62.5" width="687" height="70" rx="8" class="bf-box"/><text x="360" y="92" class="bf-ts" text-anchor="middle">Small 8-byte directory entry</text><text x="360" y="114" class="bf-tm" text-anchor="middle">Contains format, markers and either a base or payload address</text><path d="M360 150V161H177V171M360 161H543V171" class="bf-link"/><path d="M173 167l4 4 4-4M539 167l4 4 4-4" class="bf-link"/><rect x="0" y="176" width="354" height="136" rx="10" class="bf-card-hi"/><text x="177" y="204" class="bf-t-green" text-anchor="middle">Translation group</text><path d="M0 0v9m-4-4 4 4 4-4" transform="translate(177 214)" class="bf-arrow"/><text x="177" y="246" class="bf-ts" text-anchor="middle">Base + offset</text><text x="177" y="266" class="bf-tm" text-anchor="middle">The answer is already in the directory entry</text><circle cx="177" cy="290" r="9" class="bf-icon-green"/><path d="M173 290l3 3 5-6" class="bf-icon-green"/><rect x="366" y="176" width="354" height="136" rx="10" class="bf-panel"/><text x="543" y="204" class="bf-t-strong" text-anchor="middle">Other group</text><path d="M0 0v9m-4-4 4 4 4-4" transform="translate(543 214)" class="bf-arrow"/><text x="543" y="246" class="bf-ts" text-anchor="middle">Load extra record</text><text x="543" y="266" class="bf-tm" text-anchor="middle">Decode FOR / Elias–Fano / exception</text><path d="M534 284h18M534 290h18M534 296h18" class="bf-icon"/></svg>
</figure>

It is a compromise.

For the common case, the small directory entry already gives us the next pointer.

For the less common case, we fetch extra information.

Because the directory is tiny compared with the current T-vector, it may fit much more comfortably in cache.

And because we don't allocate a full 64-byte record for the simple groups, the storage could become much smaller.

Our initial calculation suggested something in the neighbourhood of 5.6 bits per row, before additional exceptions and bookkeeping.

That's not a measured final footprint. It's an optimistic model built from the reported distribution.

The attraction is real, but so is the risk: whenever we need the second record, we introduce another memory dependency.

## 7. Why a smaller table could make mibz2 faster

Suppose you're decoding one full-sized bzip2 block.

Your current T-vector is 3.6 MB.

But mibz2 can work on multiple blocks concurrently.

If you have several full-size blocks in flight, their tables compete for space in the CPU's shared cache.

<figure class="bf-fig">
<p class="bf-title">Eight simultaneous full-block tables</p>
<svg viewBox="0 0 720 334" role="img" aria-label="Table storage for eight full blocks: current 28.8 MB, 24-bit 21.6 MB, 16-bit fixed 14.5 MB, two-tier about 5 MB (estimate)."><path d="M56 286H720" class="bf-axis"/><text x="46" y="290" class="bf-tm" text-anchor="end">0 MB</text><path d="M56 218.5H720" class="bf-grid"/><text x="46" y="222.5" class="bf-tm" text-anchor="end">8 MB</text><path d="M56 151H720" class="bf-grid"/><text x="46" y="155" class="bf-tm" text-anchor="end">16 MB</text><path d="M56 83.5H720" class="bf-grid"/><text x="46" y="87.5" class="bf-tm" text-anchor="end">24 MB</text><path d="M56 16H720" class="bf-grid"/><text x="46" y="20" class="bf-tm" text-anchor="end">32 MB</text><rect x="87" y="43" width="104" height="243" rx="6" class="bf-bar"/><text x="139" y="33" class="bf-t-strong" text-anchor="middle">28.8 MB</text><text x="139" y="308" class="bf-ts" text-anchor="middle">Current</text><text x="139" y="326" class="bf-tm" text-anchor="middle">32-bit T-vector</text><rect x="253" y="103.75" width="104" height="182.25" rx="6" class="bf-bar"/><text x="305" y="93.75" class="bf-t-strong" text-anchor="middle">21.6 MB</text><text x="305" y="308" class="bf-ts" text-anchor="middle">24-bit</text><text x="305" y="326" class="bf-tm" text-anchor="middle">direct links</text><rect x="419" y="163.656" width="104" height="122.344" rx="6" class="bf-bar"/><text x="471" y="153.656" class="bf-t-strong" text-anchor="middle">14.5 MB</text><text x="471" y="308" class="bf-ts" text-anchor="middle">16-bit fixed</text><text x="471" y="326" class="bf-tm" text-anchor="middle">cache-line records</text><rect x="585" y="243.812" width="104" height="42.1875" rx="6" class="bf-bar-est"/><text x="637" y="233.812" class="bf-t-strong" text-anchor="middle">about 5 MB</text><text x="637" y="308" class="bf-ts" text-anchor="middle">Two-tier</text><text x="637" y="326" class="bf-tm" text-anchor="middle">estimate</text></svg>
<figcaption>Illustrative table storage for eight full 900 KB blocks. The two-tier estimate (dashed) excludes additional exception and alignment overhead.</figcaption>
</figure>

Earlier measurements on a 10-core workstation already show evidence of a shared-cache residency threshold.

When the live tables become too large, the processor must fetch more data from slower parts of the memory hierarchy.

Smaller tables would let more of them stay resident in the same cache capacity.

We might see fewer expensive memory misses.

Even if each individual pointer is slightly more expensive to decode, the overall decoder could become faster.

This is the same kind of trade-off Julian Seward studied in 2001.

But your work adds modern multicore concurrency and 32-lane traversal.

The goal is not to have the smallest possible table. It's to make the whole machine finish decoding sooner.

## 8. Open questions

There are three substantial unanswered questions.

**First, how much does decoding a compressed pointer cost?** With the current T-vector, the CPU loads a 32-bit value and extracts the next row. With a compressed record, it may need shifts, masks, a format test and sometimes an additional memory access. Even a small amount of extra work matters when repeated millions of times.

**Second, how expensive is construction?** The compressed table must be built from the BWT before it can be traversed. A representation that makes traversal faster but takes twice as long to build might lose overall.

**Third, does the 77.6% translation property hold broadly?** We know it appears in your benchmark corpus. We don't yet know how stable it is across different types of compressed data, and it's not a universal mathematical guarantee.

The agents have enough evidence to investigate these questions seriously, but not enough to claim a faster decoder.

## 9. What is known and what is not

| Finding | Status |
|---|---|
| Ψ has increasing sequences that can be compressed | Mathematically established |
| Actual benchmark data contains abundant consecutive-position groups | Measured by Claude's census, as reported |
| Fixed 64-byte records can represent the overwhelming majority of measured groups | Strongly supported by the census; complete encoding still needs validation |
| A two-tier directory could shrink storage further | New design hypothesis |
| Reduced working sets can improve cache residency | Supported by your existing performance measurements |
| Either compressed design will beat the current decoder | Unknown |

And the most important distinction of all:

> We've found evidence that the data is compressible in a form suited to fast navigation. We haven't yet proved that navigating the compressed representation is faster.

These are separate claims.

## 10. The underlying idea

The underlying point is that the T-vector—the enormous table of navigation pointers—isn't fundamentally necessary in its current form.

It's a convenient way to store answers to questions.

The BWT already contains the information from which those answers can be derived.

Succinct data structures teach us to exploit that fact.

But your work adds an interesting twist:

> Rather than pursuing the absolutely smallest possible encoding, we're looking for a carefully chosen amount of redundancy that allows the CPU to recover answers quickly.

In other words, sometimes storing a little unnecessary information makes the whole algorithm faster.

That's what makes the problem so subtle.

A theoretician might ask:

> How close can we get to the minimum number of bits?

A systems programmer might ask:

> How few instructions can we execute per pointer?

Your research asks:

> How should we arrange the information so that the CPU can follow millions of pointers with as little waiting as possible?

That brings information theory, succinct data structures, cache architecture and instruction scheduling together in one problem.

## Where we've arrived

<figure class="bf-fig">
<p class="bf-title">Our research journey</p>
<svg viewBox="0 0 720 340" role="img" aria-label="Five steps: Seward 2001; compressed permutation research; the 16-bit hypothesis; the corpus census (77.6% translation groups); the two-tier idea."><path d="M15 30V66" class="bf-link"/><circle cx="15" cy="14" r="13" class="bf-dot"/><text x="15" y="19" class="bf-t-strong" text-anchor="middle">1</text><text x="44" y="19" class="bf-ts">Seward, 2001</text><text x="44" y="42" class="bf-t">Showed that inverse-BWT navigation can trade memory for computation.</text><path d="M15 98V134" class="bf-link"/><circle cx="15" cy="82" r="13" class="bf-dot"/><text x="15" y="87" class="bf-t-strong" text-anchor="middle">2</text><text x="44" y="87" class="bf-ts">Compressed permutation research</text><text x="44" y="110" class="bf-t">Established that the sorted structure of Ψ can be represented economically.</text><path d="M15 166V202" class="bf-link"/><circle cx="15" cy="150" r="13" class="bf-dot"/><text x="15" y="155" class="bf-t-strong" text-anchor="middle">3</text><text x="44" y="155" class="bf-ts">Our 16-bit hypothesis</text><text x="44" y="178" class="bf-t">Proposed fitting 32 navigation values inside one cache line.</text><path d="M15 234V270" class="bf-link"/><circle cx="15" cy="218" r="13" class="bf-dot-hi"/><text x="15" y="223" class="bf-t-strong" text-anchor="middle">4</text><text x="44" y="223" class="bf-ts">Claude's corpus census</text><text x="44" y="246" class="bf-t">Found that 77.6% of rows qualify for simple translation groups, making a cheap fast path plausible.</text><circle cx="15" cy="286" r="13" class="bf-dot-hi"/><text x="15" y="291" class="bf-t-strong" text-anchor="middle">5</text><text x="44" y="291" class="bf-ts">Our latest two-tier idea</text><text x="44" y="314" class="bf-t">Proposed storing complete answers for simple groups directly in tiny descriptors,</text><text x="44" y="332" class="bf-t">fetching larger records only when needed.</text></svg>
</figure>

## What comes next

The census shows there is structure to exploit. It says nothing yet about speed, and the
next steps are the measurements that will. They run in order, one controlled comparison
at a time.

<figure class="bf-fig">
<p class="bf-title">The roadmap, in order</p>
<svg viewBox="0 0 720 560" role="img" aria-label="Six steps: the single-core fused build, measured at 1.027 times, below its floor; the cooperative fused build, ready and awaiting the workstation run; an audit of the F-byte lookup; embedded byte versus F recovery; a navigation-cost harness; and a decoder prototype only if justified."><path d="M15 30V94" class="bf-link"/><circle cx="15" cy="14" r="13" class="bf-dot"/><text x="15" y="19" class="bf-t-strong" text-anchor="middle">1</text><text x="44" y="19" class="bf-ts">Single-core fused build (#93)</text><rect x="278.4" y="4" width="124.4" height="21" rx="10.5" class="bf-pill-done"/><text x="340.6" y="18.5" class="bf-pill-done-t" text-anchor="middle">Measured: 1.027×</text><text x="44" y="44" class="bf-t">Removing the table fill and scattering straight into the T-vector gained 1.027× on CI, under the</text><text x="44" y="62" class="bf-t">pre-registered 1.03 floor. Kept as a negative result.</text><path d="M15 126V208" class="bf-link"/><circle cx="15" cy="110" r="13" class="bf-dot-ready"/><text x="15" y="115" class="bf-t-strong" text-anchor="middle">2</text><text x="44" y="115" class="bf-ts">Cooperative fused build (#81)</text><rect x="278.4" y="100" width="246" height="21" rx="10.5" class="bf-pill-ready"/><text x="401.4" y="114.5" class="bf-pill-ready-t" text-anchor="middle">Ready: awaiting the workstation run</text><text x="44" y="140" class="bf-t">The cooperative build has a copy phase and a synchronisation point that fusion may remove. Its</text><text x="44" y="158" class="bf-t">arm is built and checked word for word (#95). Pre-registered: a pass needs a build speed-up of</text><text x="44" y="176" class="bf-t">at least 1.15×; below 1.03× is a fail.</text><path d="M15 240V286" class="bf-link"/><circle cx="15" cy="224" r="13" class="bf-dot"/><text x="15" y="229" class="bf-t-strong" text-anchor="middle">3</text><text x="44" y="229" class="bf-ts">Audit the F-byte lookup (#79)</text><rect x="278.4" y="214" width="47.6" height="21" rx="10.5" class="bf-pill-next"/><text x="302.2" y="228.5" class="bf-pill-next-t" text-anchor="middle">Next</text><text x="44" y="254" class="bf-t">Read the generated code of the existing F-window lookup that recovers each output byte.</text><path d="M15 318V382" class="bf-link"/><circle cx="15" cy="302" r="13" class="bf-dot"/><text x="15" y="307" class="bf-t-strong" text-anchor="middle">4</text><text x="44" y="307" class="bf-ts">Embedded byte vs F recovery (#79)</text><rect x="308.8" y="292" width="47.6" height="21" rx="10.5" class="bf-pill-next"/><text x="332.6" y="306.5" class="bf-pill-next-t" text-anchor="middle">Next</text><text x="44" y="332" class="bf-t">One traversal kernel, lane scheduler and start-marker handling; only how the output byte is</text><text x="44" y="350" class="bf-t">recovered changes. That isolates the cost of symbol recovery before a 24-bit table.</text><path d="M15 414V478" class="bf-link"/><circle cx="15" cy="398" r="13" class="bf-dot"/><text x="15" y="403" class="bf-t-strong" text-anchor="middle">5</text><text x="44" y="403" class="bf-ts">Navigation-cost harness</text><rect x="232.8" y="388" width="54" height="21" rx="10.5" class="bf-pill-later"/><text x="259.8" y="402.5" class="bf-pill-later-t" text-anchor="middle">Later</text><text x="44" y="428" class="bf-t">Direct links, fixed 64-byte records and two-tier descriptors on a matched pointer-chasing</text><text x="44" y="446" class="bf-t">workload, the second dependent load included.</text><circle cx="15" cy="494" r="13" class="bf-dot"/><text x="15" y="499" class="bf-t-strong" text-anchor="middle">6</text><text x="44" y="499" class="bf-ts">Decoder prototype</text><rect x="187.2" y="484" width="98.8" height="21" rx="10.5" class="bf-pill-later"/><text x="236.6" y="498.5" class="bf-pill-later-t" text-anchor="middle">If justified</text><text x="44" y="524" class="bf-t">Construction, traversal, exceptions and concurrency together, before any change to the</text><text x="44" y="542" class="bf-t">production decoder.</text></svg>
<figcaption>Each step's prediction and pass and fail thresholds are written down before it runs. Status on 29 October 2026.</figcaption>
</figure>

Three table designs are kept as candidates. None is built yet, and they will compete on
measured navigation cost, not on size alone:

- **24-bit direct links:** the simple reference, a quarter smaller than today's table.
- **Fixed 64-byte records:** 32 pointers to a cache line, 16.16 bits a row on the corpus.
- **Two-tier descriptors:** 5.70 bits a row on the corpus, but 12–36% of steps need a
  second, dependent load, and on less regular inputs the design loses its advantage.

This post will be updated as the results come in. The work is tracked on
[#81](https://github.com/lmmx/mibz2/issues/81) and
[#79](https://github.com/lmmx/mibz2/issues/79).

## The bottom line

We've moved from:

> "Maybe succinct data structures could save us some memory."

To:

> "We have evidence that most of the permutation can be represented by simple rules, and we have several concrete ways of exploiting those rules while protecting the memory-level parallelism that makes mibz2 fast."

The agents now have a real engineering question to settle: whether those smaller representations can make the CPU finish its work faster.

The working hypothesis, in one line:

> A decoder may get faster not by following its pointers more efficiently, but by not storing most of them individually.

---

!!! note "Since this was written"

    The census has since been refined and the designs' sizes counted exactly. On the corpus's
    233 full blocks:

    - 77.39% of rows are in groups that are one run, ranging from 63.7% to 88.0% per block;
    - the fixed 64-byte records come to 16.16 bits a row;
    - the two-tier design comes to 5.70 bits a row, where 12–36% of steps need its second
      lookup.

    On other inputs, such as random or already-compressed data, runs are rare. There the
    two-tier design is larger than the fixed records, while the fixed records still fit at
    about 16.3 bits a row. The details, sources and the full discussion are in the research
    journal (`docs/journal/2026-10-29-succinct-review.md`, [#94](https://github.com/lmmx/mibz2/pull/94))
    and on [#79](https://github.com/lmmx/mibz2/issues/79).
