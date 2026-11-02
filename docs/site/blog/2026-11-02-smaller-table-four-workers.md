---
title: "A smaller inverse-BWT table wins with four workers"
image: assets/social/blog-2026-11-02-smaller-table-four-workers.png
description: "Its compressed records cost 15% more CPU time on one core and 16% less with four workers. Next: can it be built cheaply enough?"
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# A smaller inverse-BWT table wins with four workers

A compressed navigation table for the inverse BWT costs 15% more CPU time on one core, and
saves 16% when four workers traverse at once. We can now measure the advantage; we still
have to explain it, and to build the table cheaply.
{: .post-dek }

2 November 2026 · 10 min read · [Traversal result](#the-four-worker-reversal)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

When bzip2 compresses a block, it rearranges the bytes with the Burrows–Wheeler transform
(BWT). Putting them back in order means following a chain of row numbers: each lookup in a
table says which row to visit next, and until that entry arrives the next address is
unknown. mibz2's decoder [walks that chain](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L927-L961)
for every block it decodes.

It already hides some of the waiting. As [32 lanes, 8.6 chains](2026-10-31-room-walk-overlap.md)
describes, the walk advances 32 separate pieces of the chain in turn, so the processor can
work on one while another waits for its entry. The next problem appears when four cores
each walk a different block. [Investigating inverse-BWT memory interference](2026-11-01-inverse-bwt-memory-interference.md)
found that decoding with four workers costs more CPU time per byte than decoding with one:
the workers finish sooner together, but compete for the shared memory system on the way.

A full block has up to 900,000 rows, and mibz2's navigation table stores 4 bytes for each:
3.6 MB per block, or 14.4 MB for four blocks at once, before any of the workers' other
buffers. Could the same links be stored in much less space, and would the lighter memory
load make up for the extra work of reading a compressed entry?

The first valid comparison answers both ways at once. The compact walk costs 14.85% more
CPU time on one core and 15.95% less with four workers walking together. That result is
about this traversal, measured on its own; what it means for the whole decoder is still
open.

## A smaller table, a busier lookup

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

The step costs more, though. An ordinary compact30 step compiles to 21 instructions and four
data loads; the conventional step is 16 instructions and three loads. Two of compact30's
loads, the offset and the header, are both addressed from the current row, so neither waits
for the other, but they are still two loads where the conventional step needs one. The
[walker's journal](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md)
has the annotated disassembly.

So the question is which matters more when several cores share the machine: the extra
instructions, or the smaller table.

## The four-worker reversal

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

## Four workers cost the conventional walk far more

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

## The table still has to be built

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
