---
title: "Stages built to fit together: a new shape for the decoder's workers"
image: assets/social/blog-2026-11-27-stages-built-to-fit-together.png
description: "Rebuilt so that each stage hands the next exactly what it needs, the prototype stages of mibz2's workers save time that adds up almost exactly."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Stages built to fit together: a new shape for the decoder's workers

Rebuilt so that each stage hands the next exactly what it needs, the prototype stages of mibz2's workers save time that adds up almost exactly.
{: .post-dek }

27 November 2026 · 14 min read · [Research in progress](#7-where-the-pieces-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 is a parallel [bzip2](https://sourceware.org/bzip2/) decoder written in Rust. A bzip2 file is a sequence of independent
blocks of up to 900,000 bytes, and mibz2 decodes them in three kinds of thread. One thread, the
scanner, reads the compressed bits in order and decodes each block's Huffman codes, because a
block's start can only be found by finishing the one before it. A pool of workers then turns
each block's symbols back into text, one block per worker, in parallel. A reader hands the
finished blocks to the caller in order. The project's reference machine is a ten-core Intel i9-10900K. There the shipped decoder turns
838.8 MB of a Wikidata dump back into text in 194.6 ms with eight workers, about 4.31 GB/s, and in
1,306 ms with one.

Each worker runs a chain of stages, and for most of the project each stage was improved on its
own. That has a cost which is easy to miss. A stage inherits the form of its input from the
stage before it and must hand its output in the form the next one expects, so a stage can be
locked into a slow design by a neighbour. The fastest way to walk the text, for instance, writes
it in pieces, and that only pays if everything downstream can take pieces. This post reports a
redesign of the workers' stages as a set, each shaped around the others, and what it measured.

**Takeaways**

- **The text is now walked forward in four lanes at once,** from sixteen starting points chosen
  in advance. Construction's wiring and the walk together take 31–36% less time, close to a
  version given its starting points for free.
- **The output stays in pieces until the caller reads it.** That lets the run expansion and the
  checksum run inside each piece while it is still in cache, writing 53% fewer bytes, and
  handing pieces to the caller costs nothing measurable.
- **The middle of construction was rebuilt around fixed-size tables,** which lets the compiler
  remove every bounds check in safe Rust; with two layout changes it takes 27–29% less time.
- **Composed, the pieces add up.** All four worker changes together cut a worker's time by 25.6%
  on warm caches, matching the sum of their separate savings to within a tenth of a percent, and
  by 21–25% on cold caches.
- **Two hardware limits set the shape.** A 256 KiB private L2 cache is why each lane writes into
  32 KiB slabs, and x86-64's small register file is why there are four lanes, not eight.
- **These are prototype measurements on a cloud test machine.** Two pieces are in the code behind
  experimental build options and two are in review. The one timed on the i9 so far, the fixed-size
  middle, was 1.8% faster with one worker and unchanged at eight and ten, short of its bar. The
  6 GB/s that the project aims for at eight workers is a target, not a result.

[Section 1](#1-where-a-workers-time-goes) shows where a worker's time goes. [Section 2](#2-walking-the-cycle-in-four-lanes)
explains the four-lane walk and the order statistics that set its sixteen starting points, and
[section 3](#3-keeping-the-output-in-pieces) why its output is never stitched back together.
[Section 4](#4-a-fixed-size-middle-for-construction) covers the middle of construction and
[section 5](#5-feeding-construction-without-a-megabyte-buffer) the stage before it. [Section 6](#6-composing-the-worker)
composes the worker and shows the two hardware limits, and [section 7](#7-where-the-pieces-stand)
says where each piece stands and what eight workers need.

<figure class="bf-fig" id="pipeline">
<p class="bf-title">The decoder's stages, and what replaces each</p>
<!-- fig:pipeline -->
<svg viewBox="0 0 720 496" role="img" aria-label="The decoder as a pipeline: a serial scanner feeds parallel workers, whose stages are undoing move-to-front, construction in three parts, the emission walk, and run expansion with the checksum, then the reader. Each stage is shown with its one-worker time on the i9-10900K and the redesign that replaces it, with its measured effect on the test machine; emission, expansion and the reader are joined by ordered slices instead of one contiguous buffer."><text class="bf-t-strong" x="18.0" y="22.0" text-anchor="start" style="font-size:12px">Stage</text><text class="bf-t-strong" x="246.0" y="22.0" text-anchor="end" style="font-size:12px">i9, 1 worker</text><text class="bf-t-strong" x="266.0" y="22.0" text-anchor="start" style="font-size:12px">What replaces it</text><text class="bf-t-strong" x="702.0" y="22.0" text-anchor="end" style="font-size:12px">Effect on the test VM</text><rect x="18" y="34" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:2"><title>Scanner (serial)</title></rect><text class="bf-t-strong" x="26.0" y="51.0" text-anchor="start" style="font-size:12px">Scanner (serial)</text><text class="bf-tm" x="26.0" y="66.0" text-anchor="start" style="font-size:11px">Huffman codes, block headers</text><text class="bf-t" x="246.0" y="59.0" text-anchor="end" style="font-size:12px">173 ms</text><text class="bf-t" x="266.0" y="51.0" text-anchor="start" style="font-size:11.5px">Headers parsed with whole-run and</text><text class="bf-t" x="266.0" y="66.0" text-anchor="start" style="font-size:11.5px">multi-symbol table lookups</text><text class="bf-t-strong" x="702.0" y="59.0" text-anchor="end" style="font-size:11.5px">header setup 32–38% less</text><line x1="102" y1="75" x2="102" y2="87" style="stroke:var(--bf-muted);stroke-width:1.5"/><polygon points="98,83 106,83 102,89" style="fill:var(--bf-muted)"/><rect x="18" y="90" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-1);stroke-width:2"><title>Undo move-to-front</title></rect><text class="bf-t-strong" x="26.0" y="107.0" text-anchor="start" style="font-size:12px">Undo move-to-front</text><text class="bf-tm" x="26.0" y="122.0" text-anchor="start" style="font-size:11px">and the zero-run code</text><text class="bf-t" x="246.0" y="115.0" text-anchor="end" style="font-size:12px">132 ms</text><text class="bf-t" x="266.0" y="107.0" text-anchor="start" style="font-size:11.5px">Runs batched in 2 KiB of L1;</text><text class="bf-t" x="266.0" y="122.0" text-anchor="start" style="font-size:11.5px">a guard zone ends the slow path</text><text class="bf-t-strong" x="702.0" y="115.0" text-anchor="end" style="font-size:11.5px">12–19% less, both stages</text><line x1="102" y1="131" x2="102" y2="143" style="stroke:var(--bf-muted);stroke-width:1.5"/><polygon points="98,139 106,139 102,145" style="fill:var(--bf-muted)"/><rect x="18" y="146" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-1);stroke-width:2"><title>Construction, first pass</title></rect><text class="bf-t-strong" x="26.0" y="163.0" text-anchor="start" style="font-size:12px">Construction, first pass</text><text class="bf-tm" x="26.0" y="178.0" text-anchor="start" style="font-size:11px">runs into strips</text><text class="bf-t" x="246.0" y="171.0" text-anchor="end" style="font-size:12px">200 ms</text><text class="bf-t" x="266.0" y="163.0" text-anchor="start" style="font-size:11.5px">fed straight from the batch:</text><text class="bf-t" x="266.0" y="178.0" text-anchor="start" style="font-size:11.5px">no 1 MiB buffer between them</text><line x1="102" y1="187" x2="102" y2="199" style="stroke:var(--bf-muted);stroke-width:1.5"/><polygon points="98,195 106,195 102,201" style="fill:var(--bf-muted)"/><rect x="18" y="202" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-2);stroke-width:2"><title>Construction, middle</title></rect><text class="bf-t-strong" x="26.0" y="219.0" text-anchor="start" style="font-size:12px">Construction, middle</text><text class="bf-tm" x="26.0" y="234.0" text-anchor="start" style="font-size:11px">interval-exchange induction</text><text class="bf-t" x="246.0" y="227.0" text-anchor="end" style="font-size:12px">216 ms</text><text class="bf-t" x="266.0" y="219.0" text-anchor="start" style="font-size:11.5px">Fixed 65,536-entry tables, split</text><text class="bf-t" x="266.0" y="234.0" text-anchor="start" style="font-size:11.5px">fields, generation-tagged caches</text><text class="bf-t-strong" x="702.0" y="227.0" text-anchor="end" style="font-size:11.5px">27–29% less</text><line x1="102" y1="243" x2="102" y2="255" style="stroke:var(--bf-muted);stroke-width:1.5"/><polygon points="98,251 106,251 102,257" style="fill:var(--bf-muted)"/><rect x="18" y="258" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:2"><title>Construction, wiring</title></rect><text class="bf-t-strong" x="26.0" y="275.0" text-anchor="start" style="font-size:12px">Construction, wiring</text><text class="bf-tm" x="26.0" y="290.0" text-anchor="start" style="font-size:11px">links between retained rows</text><text class="bf-t" x="246.0" y="283.0" text-anchor="end" style="font-size:12px">161 ms</text><text class="bf-t" x="266.0" y="275.0" text-anchor="start" style="font-size:11.5px">Forward links only, SIMD-built;</text><text class="bf-t" x="266.0" y="290.0" text-anchor="start" style="font-size:11.5px">16 anchors marked in a spare bit</text><text class="bf-t-strong" x="702.0" y="283.0" text-anchor="end" style="font-size:11.5px">together: 31–36% less</text><line x1="102" y1="299" x2="102" y2="311" style="stroke:var(--bf-muted);stroke-width:1.5"/><polygon points="98,307 106,307 102,313" style="fill:var(--bf-muted)"/><rect x="18" y="314" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:2"><title>Emission walk</title></rect><text class="bf-t-strong" x="26.0" y="331.0" text-anchor="start" style="font-size:12px">Emission walk</text><text class="bf-tm" x="26.0" y="346.0" text-anchor="start" style="font-size:11px">the text, byte by byte</text><text class="bf-t" x="246.0" y="339.0" text-anchor="end" style="font-size:12px">490 ms</text><text class="bf-t" x="266.0" y="331.0" text-anchor="start" style="font-size:11.5px">Four lanes walk the cycle at once</text><text class="bf-t" x="266.0" y="346.0" text-anchor="start" style="font-size:11.5px">into 32 KiB slabs, kept as slices</text><line x1="102" y1="355" x2="102" y2="367" style="stroke:var(--bf-muted);stroke-width:1.5" stroke-dasharray="4 3"/><polygon points="98,363 106,363 102,369" style="fill:var(--bf-muted)"/><text class="bf-tm" x="112.0" y="366.0" text-anchor="start" style="font-style:italic;font-size:11px">ordered slices</text><rect x="18" y="370" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:2"><title>Run expansion, checksum</title></rect><text class="bf-t-strong" x="26.0" y="387.0" text-anchor="start" style="font-size:12px">Run expansion, checksum</text><text class="bf-tm" x="26.0" y="402.0" text-anchor="start" style="font-size:11px">first-stage runs, CRC-32</text><text class="bf-t" x="246.0" y="395.0" text-anchor="end" style="font-size:12px">76 ms</text><text class="bf-t" x="266.0" y="387.0" text-anchor="start" style="font-size:11.5px">Fused into each slab in place;</text><text class="bf-t" x="266.0" y="402.0" text-anchor="start" style="font-size:11.5px">four CRCs combined at the end</text><text class="bf-t-strong" x="702.0" y="395.0" text-anchor="end" style="font-size:11.5px">53% fewer bytes written</text><line x1="102" y1="411" x2="102" y2="423" style="stroke:var(--bf-muted);stroke-width:1.5" stroke-dasharray="4 3"/><polygon points="98,419 106,419 102,425" style="fill:var(--bf-muted)"/><text class="bf-tm" x="112.0" y="422.0" text-anchor="start" style="font-style:italic;font-size:11px">ordered slices</text><rect x="18" y="426" width="168" height="40" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:2"><title>Reader</title></rect><text class="bf-t-strong" x="26.0" y="443.0" text-anchor="start" style="font-size:12px">Reader</text><text class="bf-tm" x="26.0" y="458.0" text-anchor="start" style="font-size:11px">blocks back in order</text><text class="bf-t" x="266.0" y="443.0" text-anchor="start" style="font-size:11.5px">Accepts up to 16 slices a block</text><text class="bf-t" x="266.0" y="458.0" text-anchor="start" style="font-size:11.5px">in one 160-byte record</text><text class="bf-t-strong" x="702.0" y="451.0" text-anchor="end" style="font-size:11.5px">costs nothing measurable</text><line x1="10" y1="90" x2="10" y2="410" style="stroke:var(--bf-frame);stroke-width:2"/><text class="bf-tm" x="18.0" y="482.0" text-anchor="start" style="font-size:11px">Parallel workers run the six middle stages, one block each. Dashed arrows: output passed on as slices, never stitched.</text></svg>
<figcaption>The middle column is measured on the i9-10900K with one worker on the Wikidata sample, the shipped decoder. The right-hand column is measured on the cloud test machine, in prototypes, against the shipped stage on the same machine; ranges span compiler settings and cache states. The worker stages run in parallel, one block per worker.</figcaption>
</figure>

## 1. Where a worker's time goes

bzip2 compresses a block in several layers, and decoding undoes them in reverse order: the scanner
removes the Huffman coding, and the worker the rest. The scanner's Huffman decoding yields symbols in a move-to-front code with runs of zeros abbreviated; the worker's first stage
undoes both, recovering the last column of the block's sorted rotations, the
[Burrows–Wheeler transform](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md). The next
two stages invert that transform, which is where most of the time goes. bzip2 also shortened every
run of four or more equal bytes before sorting, so the last worker stage expands those runs again
and checks the block's CRC-32 checksum.

On Wikidata, mibz2 inverts the transform with [Open](2026-11-19-open-returns.md), a kernel that
keeps only about one row in eight of the sorted table, the rows in short runs, and works out
where the walk goes from each kept row to the next and which bytes it spells on the way.
Construction does that in three parts. A first pass groups the runs into strips. A middle stage
shrinks the map between kept rows by [interval-exchange induction](2026-11-10-rauzy-veech-reconstruction.md),
recording the words it skips as rules in a dictionary. A last part wires each kept row to the
next. Emission then walks that cycle of kept rows, writing a word at each step.

<aside class="il-gloss" markdown>**Interval-exchange induction** is a classical move from the study of
dynamical systems, herein turned to decoding. Where one run of rows lands on the end of another,
the rows it lands on tell the decoder nothing new, so they can be deleted, and the run that landed
there jumps straight to where they would have gone, recording a two-letter word for its two moves. Repeated, it shrinks the block's map while the
dictionary of words grows; [the post that introduced it](2026-11-10-rauzy-veech-reconstruction.md)
works it by hand on a three-byte block.</aside>

At one worker on the i9, a worker is busy for 1,280 ms, and the walk alone takes 490 ms of it. The
three parts of construction take 200, 216 and 161 ms in order. Undoing move-to-front takes 132 ms,
the run expansion and checksum 76 ms, and the scanner spends 173 ms on the same input. Those
proportions say where a redesign must start: the walk, and the wiring it depends on.

Every new measurement below comes from one cloud test machine, a two-core virtual machine on an
Intel Xeon. Like the i9, it has 32 KiB of L1 data cache and 256 KiB of L2 per core; its shared L3
holds 45 MiB. Each
figure is the median of seven interleaved repeats of the CPU time the timed thread itself used, so
that time stolen by other tenants does not count. Runs with warm caches reuse one worker's
buffers; cold runs rotate through more buffers than the L3 holds and flush it in between. The
corpus is the 233 blocks of a 210 MB Wikidata sample that the shipped decoder sends to Open, and
every variant produced byte-identical output on all of them.

## 2. Walking the cycle in four lanes

The shipped decoder stores each link in the cycle as the XOR of its two neighbours, which lets
emission start at one marked row and walk in both directions at once. That gives two chains of
memory reads in flight, but each step must know where it came from, and the XOR links can only be
built one scattered update at a time. The redesign keeps only each row's successor. Those links
are built by a simple vectorised loop that is 2.7–3.0 times faster than the XOR wiring, and they
allow the cycle to be walked [forward from any point](2026-11-26-from-bottlenecks-to-better-algorithms.md#8-forward-wiring-and-multi-lane-emission).

The walk is a chain of dependent reads: the next row's address is only known once the current
row has arrived. A single chain therefore spends most of its time waiting on memory. Four chains
walked in one loop keep four reads in flight, so the processor overlaps their waits. For that the
cycle must be cut into pieces in advance, which needs starting points. Rather than store them
separately, the redesign hides them in the links. Every Wikidata block needs at most 29 bits for a
link and its word, so bit 31 is free. Setting it on sixteen evenly spaced rows costs sixteen
stores, about 30 ns a block. The loop loads all four lanes' next links and tests their top bits
with one branch, which fires about sixteen times a block, once in seven thousand steps. When a
lane reaches a marked row it closes its piece and takes the next unstarted one.

The number of starting points comes from order statistics. The pieces between evenly spaced rows
of the sorted table are not even pieces of the text, because the text visits those rows in an
order that is, for this purpose, close to random. The pieces then behave like the gaps between $S$ points thrown at random on
a circle of length $m$, and the expected longest of $S$ such gaps is

$$
\mathbb{E}[L_{\max}] = \frac{m}{S}\,H_S, \qquad H_S = 1 + \tfrac12 + \dots + \tfrac1S.
$$

With four points and four lanes, the walk is not done until the longest piece is, and that piece
averages $H_4 = 2.08$ times an even quarter. Measured over the 233 blocks, it is 2.081 times.
With sixteen points, a lane that finishes a short piece takes another, and the longest piece falls
to 0.82 of a quarter. On average 3.15 of the four lanes are busy, against 1.90 with four points.
Beyond sixteen the gain is small, while each extra point adds a piece to hand on.

<figure class="bf-fig" id="anchors">
<p class="bf-title">Random gaps, predicted and measured</p>
<!-- fig:anchors -->
<svg viewBox="0 0 720 300" role="img" aria-label="Longest arc of the cycle, as a multiple of an even quarter share, against the number of anchors: the order-statistics prediction 4 H_S over S, and the values measured over 233 Wikidata blocks, which lie on it: 2.08 at 4 anchors, 1.31 at 8, 0.82 at 16, 0.51 at 32. Below, the average number of lanes busy rises from 1.9 at 4 anchors to 3.15 at 16 and 3.49 at 32."><line x1="90" y1="220.0" x2="690" y2="220.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="82.0" y="224.0" text-anchor="end" style="font-size:11px">0.0×</text><line x1="90" y1="182.5" x2="690" y2="182.5" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="82.0" y="186.5" text-anchor="end" style="font-size:11px">0.5×</text><line x1="90" y1="145.0" x2="690" y2="145.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="82.0" y="149.0" text-anchor="end" style="font-size:11px">1.0×</text><line x1="90" y1="107.5" x2="690" y2="107.5" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="82.0" y="111.5" text-anchor="end" style="font-size:11px">1.5×</text><line x1="90" y1="70.0" x2="690" y2="70.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="82.0" y="74.0" text-anchor="end" style="font-size:11px">2.0×</text><line x1="90" y1="145.0" x2="690" y2="145.0" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:5 3"/><text class="bf-tm" x="690.0" y="139.0" text-anchor="end" style="font-size:11px">an even quarter of the cycle</text><polyline points="108.8,36.7 127.5,63.7 146.2,83.0 165.0,97.5 183.8,108.9 202.5,118.1 221.2,125.7 240.0,132.1 258.8,137.6 277.5,142.4 296.2,146.6 315.0,150.3 333.8,153.6 352.5,156.6 371.2,159.3 390.0,161.7 408.8,164.0 427.5,166.0 446.2,167.9 465.0,169.7 483.8,171.3 502.5,172.8 521.2,174.2 540.0,175.5 558.8,176.8 577.5,177.9 596.2,179.0 615.0,180.1 633.8,181.0 652.5,182.0 671.2,182.8 690.0,183.7" style="fill:none;stroke:var(--bf-vz-2);stroke-width:2"><title>prediction: 4 H_S / S</title></polyline><circle cx="127.5" cy="63.9" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>4 anchors: measured 2.081×, predicted 2.083×</title></circle><circle cx="202.5" cy="121.7" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>8 anchors: measured 1.310×, predicted 1.359×</title></circle><circle cx="277.5" cy="142.8" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>12 anchors: measured 1.029×, predicted 1.034×</title></circle><circle cx="352.5" cy="158.7" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>16 anchors: measured 0.818×, predicted 0.845×</title></circle><circle cx="502.5" cy="174.3" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>24 anchors: measured 0.609×, predicted 0.629×</title></circle><circle cx="652.5" cy="182.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>32 anchors: measured 0.507×, predicted 0.507×</title></circle><text class="bf-t" x="137.5" y="67.9" text-anchor="start" style="font-size:11.5px">measured 2.08×, predicted 2.08×</text><text class="bf-t-strong" x="354.5" y="180.7" text-anchor="start" style="font-size:12px">16 anchors: 0.82×</text><circle cx="490" cy="22" r="5" style="fill:var(--bf-vz-1)"/><text class="bf-tm" x="500.0" y="26.0" text-anchor="start" style="font-size:11px">measured</text><line x1="570" y1="22" x2="590" y2="22" style="stroke:var(--bf-vz-2);stroke-width:2"/><text class="bf-tm" x="596.0" y="26.0" text-anchor="start" style="font-size:11px">4 H(S) / S</text><text class="bf-t-strong" x="18.0" y="26.0" text-anchor="start" style="font-size:12px">Longest arc ÷ even share</text><text class="bf-t" x="127.5" y="238.0" text-anchor="middle" style="font-size:12px">4</text><text class="bf-t-strong" x="127.5" y="270.0" text-anchor="middle" style="font-size:12px">1.90</text><text class="bf-t" x="202.5" y="238.0" text-anchor="middle" style="font-size:12px">8</text><text class="bf-t-strong" x="202.5" y="270.0" text-anchor="middle" style="font-size:12px">2.65</text><text class="bf-t" x="277.5" y="238.0" text-anchor="middle" style="font-size:12px">12</text><text class="bf-t-strong" x="277.5" y="270.0" text-anchor="middle" style="font-size:12px">2.91</text><text class="bf-t" x="352.5" y="238.0" text-anchor="middle" style="font-size:12px">16</text><text class="bf-t-strong" x="352.5" y="270.0" text-anchor="middle" style="font-size:12px">3.15</text><text class="bf-t" x="502.5" y="238.0" text-anchor="middle" style="font-size:12px">24</text><text class="bf-t-strong" x="502.5" y="270.0" text-anchor="middle" style="font-size:12px">3.36</text><text class="bf-t" x="652.5" y="238.0" text-anchor="middle" style="font-size:12px">32</text><text class="bf-t-strong" x="652.5" y="270.0" text-anchor="middle" style="font-size:12px">3.49</text><text class="bf-tm" x="82.0" y="238.0" text-anchor="end" style="font-size:11px">anchors</text><text class="bf-tm" x="82.0" y="270.0" text-anchor="end" style="font-size:11px">lanes busy</text><text class="bf-tm" x="82.0" y="284.0" text-anchor="end" style="font-size:11px">(of four)</text></svg>
<figcaption>Measured over the 233 Wikidata blocks: the longest piece of the cycle as a multiple of an even quarter (points), against the order-statistics prediction $4H_S/S$ (line), and the average number of the four lanes busy. That the points sit on the line is consistent with the rows' order along the text being as good as random for this purpose.</figcaption>
</figure>

Each lane writes its bytes into a 32 KiB slab inside the block's own output buffer, and a lane
that takes a new piece continues in the same slab, so the pieces need no extra memory. On the test
machine, wiring and walk together took 31–32% less time than the shipped pair with warm caches and
33–36% less with cold caches. A version handed its sixteen starting points for free, without the
marking or the extra branch, shows what the marking costs. With cold caches the real design reached
97.4% of its speed on one build and matched it on the other. With warm caches it was 7–9% slower
than that ideal.

## 3. Keeping the output in pieces

The four lanes leave a block's text as about 42 pieces, in the right order but at scattered places
in the buffer. The obvious next step is to copy them together into one contiguous block, which is
what the rest of the decoder expects. The redesign leaves them where they are, for the sake of the stage that comes next.

The last worker stage expands bzip2's first-stage runs: four equal bytes followed by a count of
further copies. On Wikidata, 89.4% of those counts are zero, so expansion mostly shrinks a piece,
and 86.4% of the lanes' pieces change length. Stitching them first therefore moves 74.9% of the
block's bytes, and expanding them after the copy reads everything again. The redesign instead
expands each lane's output in place, 4 KiB at a time as the lane writes it, which is safe because
a shrinking expansion never overtakes the bytes it reads. The rare growth spills a short tail to a
small side buffer. The CRC-32 is computed over each 64 KiB of finished output with the processor's
carry-less multiply while it is still in L2, and the four lanes' checksums are combined at the end
of the block.

<aside class="il-gloss" markdown>

**Combining checksums** is possible because a CRC is the remainder of a
polynomial division over GF(2), the field of two elements, in which addition is XOR. The CRC of two
pieces joined is the first piece's CRC multiplied by a power of $x$ set by the second's length, plus
the second's CRC, all modulo the CRC's polynomial. Four lanes' checksums thereby become the block's
in microseconds.

</aside>

Over the Wikidata workload this writes 53.4% fewer bytes than separate passes. With warm caches
it took 11.4% less time than walking in four lanes and then expanding separately, and 7.3% less
with cold caches, where it matched the time of a pass that only computes the checksum. Stitching
first undid the gain: the same fused design, followed by a contiguous copy, wrote only 18.6% fewer
bytes, was slower than the separate passes with warm caches and barely faster with cold ones.

The pieces must then travel to the caller, and the reader expected one contiguous buffer per
block. A redesigned hand-over passes a 160-byte record holding the buffer and up to sixteen
ranges, with no extra allocation. Timed across six real ways of reading the output, from a
zero-copy buffered reader to scatter-filling 256 KiB reads, delivery in pieces stayed within
measurement noise of a single contiguous slice, while stitching first cost a third to four-fifths
more. Reserving room for the in-place expansion, 4 KiB a lane, raises the decoder's peak memory by
1.7%, with no block on Wikidata outgrowing its buffer.

## 4. A fixed-size middle for construction

The middle of construction, the induction and its dictionary, is the second-largest piece of a
worker's time, and earlier changes to it, each tried alone, had been projected at no more than
2.5% of a decode. The redesign changed all three of its parts at once.

The induction keeps a node for each surviving run, with links to its neighbours, a width and a
word, and every step edits a few neighbouring nodes. A Wikidata block has about 9,700 nodes on
average, far below 65,536. The redesign therefore stores each field in its own array of exactly
65,536 entries and indexes it with a 16-bit integer. The compiler can then prove every index in
range and drops every bounds check, in safe Rust with no unsafe code. That alone saved about 8 ms
over the same design with growable vectors, and a block with more nodes keeps the old path. Separate arrays, rather than one record per node, keep
the links a step edits densely packed in cache. They beat 24- and 32-byte records by 4–7 ms. Short words of up to fifteen bytes live inline in a 16-byte integer, which cut
the rules the step allocates by 54.6%, and steps between runs of equal width now update in place.
A single forward pass replaced the setup's scattered reads, and the dictionary's per-block caches
are reused across blocks with a generation tag instead of being cleared, which removes 24 KiB of
zeroing a block.

Each part fell. Setup went from 26.5 to 15.4 ms and the dictionary from 40.7 to 34.8 ms. The steps
went from 64.2 to 43.8 ms, or from 24.1 to 16.4 ns a step. The middle stage took 27.4–28.8% less time in all four
combinations of compiler setting and emission design. All of construction and emission together
took 5.4–7.6% less. One tempting shortcut failed instructively. Dropping the deduplication of
dictionary entries saved another 5.5 ms but made the dictionary 73.3% larger. The walk reads the
dictionary at every step, so it slowed by 1.5–2.1% and gave the time back. The fixed
arrays reserve 1.75 MiB of address space a worker, of which a block touches about 273 KiB on
average.

On the i9 the redesign has since been timed against the shipped decoder, fifteen paired rounds on
the Wikidata sample. With one worker it was 1.8% faster (a ratio of 0.982, interval 0.978–0.987),
against a registered bar of 3%. At eight and ten workers it made no measurable difference, and it
used 4–5% more memory. So it stays behind its build option. The stage is about a sixth of a
worker's time on the i9 (216 of 1,280 ms), so a 28% cut could move a one-worker decode by about 5%
at most. The i9 gave 1.8%, and why the rest didn't arrive is not yet known. At eight workers the
scanner, not the workers, sets the pace: the
[complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) again.

## 5. Feeding construction without a megabyte buffer

The first worker stage undoes move-to-front and hands construction's first pass a list of runs,
up to a megabyte of it. Streaming each run straight into the first pass looks like an obvious
saving, yet a direct attempt made the two stages 1.6–4.3% slower. The machine code showed why. The
move-to-front loop already uses all fifteen general-purpose registers that x86-64 offers a
program, and inlining the first pass into it pushed the loop's hottest values out to the stack,
to be reloaded on every symbol.

The redesign keeps the two loops apart but close. The first stage packs up to 512 runs into a 2 KiB
buffer that stays in the L1 cache, a byte and a count in each 32-bit entry. The first pass then
drains it in a separate tight loop. A second change removed most of a slow path that the first pass
took 417,041 times on the workload. In 84.3% of those calls the only reason was that a 16-wide
vector write might cross the end of a 512-entry chunk. Reserving the last sixteen entries of each chunk as a guard zone lets the
write go ahead unchecked and cut those calls 6.2-fold. Together the two stages took 12.1% less
time on the baseline build and 19.1% less with code compiled for the native processor, and the
megabyte buffer is gone.

## 6. Composing the worker

Changes measured one at a time can fight once combined, over registers or over cache. A prototype
worker therefore ran all four changes together, from the scanner's symbols to verified,
checksummed bytes, adding one at a time, with each change also run alone for comparison.

<figure class="bf-fig" id="ladder">
<p class="bf-title">One worker's time as the redesigned stages are added</p>
<!-- fig:ladder -->
<svg viewBox="0 0 720 300" role="img" aria-label="Stacked bars of one worker's CPU time over 233 Wikidata blocks on the test VM as each redesigned piece is added: 737 ms shipped, 679 with batched move-to-front, 650 with the fixed-size middle stage, 553 with the four-lane walk kept as slices, and 548 with fused expansion and checksum. The sum of the four pieces' separate savings predicts 548 ms."><rect x="206" y="10" width="12" height="12" rx="2" style="fill:var(--bf-vz-1)"/><text class="bf-tm" x="223.0" y="20.0" text-anchor="start" style="font-size:11px">move-to-front, first pass</text><rect x="389.0" y="10" width="12" height="12" rx="2" style="fill:var(--bf-vz-2)"/><text class="bf-tm" x="406.0" y="20.0" text-anchor="start" style="font-size:11px">middle stage</text><rect x="494.0" y="10" width="12" height="12" rx="2" style="fill:var(--bf-vz-3)"/><text class="bf-tm" x="511.0" y="20.0" text-anchor="start" style="font-size:11px">wiring</text><rect x="563.0" y="10" width="12" height="12" rx="2" style="fill:var(--bf-vz-4)"/><text class="bf-tm" x="580.0" y="20.0" text-anchor="start" style="font-size:11px">walk, expansion, CRC</text><line x1="206.0" y1="34" x2="206.0" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="206.0" y="252.0" text-anchor="middle" style="font-size:11px">0</text><line x1="330.0" y1="34" x2="330.0" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="330.0" y="252.0" text-anchor="middle" style="font-size:11px">200</text><line x1="454.0" y1="34" x2="454.0" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="454.0" y="252.0" text-anchor="middle" style="font-size:11px">400</text><line x1="578.0" y1="34" x2="578.0" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="578.0" y="252.0" text-anchor="middle" style="font-size:11px">600</text><line x1="702.0" y1="34" x2="702.0" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="702.0" y="252.0" text-anchor="middle" style="font-size:11px">800 ms</text><text class="bf-t" x="198.0" y="59.0" text-anchor="end" style="font-size:12px">Shipped worker</text><rect x="206.0" y="44" width="127.9" height="22" style="fill:var(--bf-vz-1)"><title>move-to-front, first pass: 206.4 ms</title></rect><rect x="333.9" y="44" width="81.4" height="22" style="fill:var(--bf-vz-2)"><title>middle stage: 131.3 ms</title></rect><rect x="415.3" y="44" width="63.9" height="22" style="fill:var(--bf-vz-3)"><title>wiring: 103.0 ms</title></rect><rect x="479.2" y="44" width="186.9" height="22" style="fill:var(--bf-vz-4)"><title>walk, expansion, CRC: 301.5 ms</title></rect><text class="bf-t-strong" x="669.0" y="59.0" text-anchor="start" style="font-size:12px">737 ms</text><text class="bf-t" x="198.0" y="97.0" text-anchor="end" style="font-size:12px">+ batched move-to-front</text><rect x="206.0" y="82" width="94.8" height="22" style="fill:var(--bf-vz-1)"><title>move-to-front, first pass: 152.8 ms</title></rect><rect x="300.8" y="82" width="80.1" height="22" style="fill:var(--bf-vz-2)"><title>middle stage: 129.2 ms</title></rect><rect x="380.8" y="82" width="53.5" height="22" style="fill:var(--bf-vz-3)"><title>wiring: 86.2 ms</title></rect><rect x="434.3" y="82" width="186.7" height="22" style="fill:var(--bf-vz-4)"><title>walk, expansion, CRC: 301.1 ms</title></rect><text class="bf-t-strong" x="632.8" y="97.0" text-anchor="start" style="font-size:12px">679 ms</text><text class="bf-t" x="198.0" y="135.0" text-anchor="end" style="font-size:12px">+ fixed-size middle stage</text><rect x="206.0" y="120" width="100.5" height="22" style="fill:var(--bf-vz-1)"><title>move-to-front, first pass: 162.1 ms</title></rect><rect x="306.5" y="120" width="58.4" height="22" style="fill:var(--bf-vz-2)"><title>middle stage: 94.2 ms</title></rect><rect x="364.9" y="120" width="58.2" height="22" style="fill:var(--bf-vz-3)"><title>wiring: 93.8 ms</title></rect><rect x="423.1" y="120" width="189.0" height="22" style="fill:var(--bf-vz-4)"><title>walk, expansion, CRC: 304.8 ms</title></rect><text class="bf-t-strong" x="615.2" y="135.0" text-anchor="start" style="font-size:12px">650 ms</text><text class="bf-t" x="198.0" y="173.0" text-anchor="end" style="font-size:12px">+ four-lane walk, slices</text><rect x="206.0" y="158" width="96.4" height="22" style="fill:var(--bf-vz-1)"><title>move-to-front, first pass: 155.4 ms</title></rect><rect x="302.4" y="158" width="66.6" height="22" style="fill:var(--bf-vz-2)"><title>middle stage: 107.4 ms</title></rect><rect x="368.9" y="158" width="21.0" height="22" style="fill:var(--bf-vz-3)"><title>wiring: 33.8 ms</title></rect><rect x="389.9" y="158" width="162.7" height="22" style="fill:var(--bf-vz-4)"><title>walk, expansion, CRC: 262.4 ms</title></rect><text class="bf-t-strong" x="555.0" y="173.0" text-anchor="start" style="font-size:12px">553 ms</text><text class="bf-t" x="198.0" y="211.0" text-anchor="end" style="font-size:12px">+ fused expansion, CRC</text><rect x="206.0" y="196" width="103.7" height="22" style="fill:var(--bf-vz-1)"><title>move-to-front, first pass: 167.2 ms</title></rect><rect x="309.7" y="196" width="59.0" height="22" style="fill:var(--bf-vz-2)"><title>middle stage: 95.1 ms</title></rect><rect x="368.7" y="196" width="21.4" height="22" style="fill:var(--bf-vz-3)"><title>wiring: 34.5 ms</title></rect><rect x="390.0" y="196" width="155.9" height="22" style="fill:var(--bf-vz-4)"><title>walk, expansion, CRC: 251.5 ms</title></rect><text class="bf-t-strong" x="552.0" y="211.0" text-anchor="start" style="font-size:12px">548 ms</text><line x1="545.7" y1="190" x2="545.7" y2="226" style="stroke:var(--bf-ink);stroke-width:2;stroke-dasharray:3 2"/><text class="bf-tm" x="541.7" y="236.0" text-anchor="end" style="font-size:11px">sum of the four separate savings predicts 548 ms</text><text class="bf-tm" x="206.0" y="272.0" text-anchor="start" style="font-size:11px">CPU time of one worker over 233 Wikidata blocks (test VM, baseline x86-64, caches warm)</text></svg>
<figcaption>Measured on the cloud test machine, baseline build, caches warm: one worker's CPU time over the 233 Wikidata blocks, split by stage. The dashed mark is the shipped time less the sum of the four changes' separate savings. Stage medians need not sum exactly to the total's median.</figcaption>
</figure>

With warm caches on the baseline build, the composed worker took 548.5 ms against the shipped
737.1 ms, or 25.6% less. The separate savings predicted 548.0 ms. On the native build the composed
worker did better than the sum, by 3.8% of the shipped time; with cold caches on the baseline build
it fell short of the sum by 2.0%. The fused expansion also halved the memory the pieces would
otherwise need. Without it a worker kept 6.7 MiB of scratch for raw slabs and expanded output; with
it, 3.4 MiB, against 2.8 MiB for the shipped worker.

Two hardware limits then set the design's dimensions. The first is the L2 cache. Doubling the
lanes' slabs from 32 to 64 KiB, to compact and checksum half as often, made the walk and expansion
slower in every build and cache state, by up to 22 ms. Four lanes of 64 KiB slabs fill all 256 KiB of a
core's private L2, on the test machine and on the i9 alike, so each slab's start is evicted while
the lanes still chase links through the rest of the table, and the in-place expansion then reads
from L3. At 32 KiB the active slabs take half of L2 and the links have the rest. Marking the
expansion routine as cold, so the compiler stops inlining it five times into the lane loop, saved
another 5–16 ms. The final composed worker took 21.6–24.6% less time than the shipped one with cold
caches, and 27.4% less with warm caches on the native build.

The second limit is the register file. A core of either machine can keep ten misses from its L1
cache outstanding at once, so six, eight or ten lanes looked possible. More lanes were indeed busy, up to 7.7 on average with ten,
yet every wider design was slower. With cold caches, total worker time rose by 8.0% at six lanes,
10.8% at eight and 14.4% at ten.
At eight lanes the compiler had to spill all eight loaded links to the stack and reload the lanes'
write positions on every step. The table mostly sits in L3 rather than main memory, so the extra
overlap could not repay those stack round trips. Splitting the lanes' chasing from
their writing, to free registers, was slower still, because it stopped the processor overlapping
the dictionary copies with the misses. Four lanes was the fastest width tried, and the eight-lane
machine code shows why: at eight, the lanes' working state no longer fits in the registers.

## 7. Where the pieces stand

Each dimension of the design now has a reason. There are four lanes because of the registers,
32 KiB slabs because of L2 and sixteen starting points because of the order statistics. The output
stays in pieces because expansion changes their lengths, and the induction has a fixed size because
its nodes stay far below 65,536. None of it is yet the default decoder.

| Piece | State on 27 November |
|---|---|
| Fixed-size middle of construction | In the code, behind an experimental build option; on the i9, 1.8% faster with one worker and unchanged at eight and ten, short of its bar, so not adopted |
| Four-lane walk from sixteen marked rows | In the code, behind an experimental build option, in the form that stitches its pieces; workstation test to come |
| Batched move-to-front with a guard zone | Implemented, in review |
| Faster block-header parsing in the scanner | Implemented, in review |
| Fused expansion and checksum; delivery in pieces | Prototypes on the test machine |
| The composed worker | A prototype on the test machine |

The scanner received one change of its own. Parsing each block's header, its table choices and
Huffman code lengths, took about a tenth of the scanner's time. Most table choices repeat the
previous table, so one leading-zero count now reads a whole run of them. Most code lengths differ
from the last by a delta coded in one or three bits, so a 4 KiB table decodes about 3.5 of them a
lookup. Header
parsing took a third less time, 31.6–37.7% across builds, and the whole scanner 4.4–7.4% less.

Eight workers are where the project's aim lies. Decoding the 838.8 MB in about 140 ms would be
6 GB/s, against 194.6 ms today. There the [scanner and workers limit the decode together](2026-11-25-the-decoder-as-a-queueing-network.md),
so shrinking only the workers moves the limit to the scanner. One design under study lets idle
workers scan ahead from the slack between their own blocks, while the reader thread, idle 91.6% of
the time, finds the block boundaries. A model of the decoder replayed against the i9's own timings
puts eight-worker time at about the scanner's and eight workers' busy time shared nine ways, plus
4.85 ms. It reached 140 ms only with a scanner change the i9 since declined to adopt and worker
times below what the composed worker reaches with cold caches. With today's figures our
arithmetic on the same model gives about 147–152 ms, a route to the target rather than its arrival.

The next measurements are on the i9: each piece as it reaches the code, then the composed worker,
then eight workers together. The general lesson is a modest one. When stages are redesigned as a
set, a change that looks unprofitable alone, such as walking in pieces, can be the one that makes
its neighbours' gains possible.

---

**Sources:**

- The i9-10900K baseline: [the zero-copy floor run](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md) (838.8 MB in 194.6 ms at eight workers) and [the decoder baseline](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/frontier/DECODER.md); one-worker stage times from [the stage-1 report](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt).
- The four-lane walk: [the sampled-starting-point probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md) (#1050, #1057); the build behind an experimental option, [#1110](https://github.com/lmmx/mibz2/pull/1110), with [its registration](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-forward-emission/registration.md).
- Fused expansion and checksum: [the fusion probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/forward-rle1-crc-fusion.md) (#1045). Delivery in pieces: [the delivery probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/vectored-delivery-boundary.md) (#1058).
- The middle of construction: [the combined redesign probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-26-explore/a6b-combined-redesign.md) (#1046); the build behind an experimental option, [#1053](https://github.com/lmmx/mibz2/pull/1053), with [its registration](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-a6b-endpoint-build/registration.md); the first workstation attempt, [recorded as incomplete](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/frontier/deltas/2026-11-27-a6b-workstation-incomplete.md).
- Batched move-to-front: [the register-spill dissection](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/mtf-fold-and-outq-dissection.md) (#1048) and [the batching probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/mtf-l1-batch-rescue-probe.md) (#1055); the implementation in review, [#1132](https://github.com/lmmx/mibz2/pull/1132).
- The composed worker: [the first composition](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/composed-worker-stack-probe.md) (#1062), [the slab-size and outlining round](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/composed-worker-v2-probe.md) (#1067) and [the lane-count and layout probe](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/w4-v3-mlp-layout-probe.md) (#1076).
- The scanner's headers: [the header dissection](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/scanner-a2-dissection.md) (#1069); the implementation in review, [#1117](https://github.com/lmmx/mibz2/pull/1117).
- Eight workers: [the slack-only scanning study](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-27-explore/slack-hybrid-scan-feasibility.md) (#1056) and [its registration](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/docs/journal/2026-11-24-algorithm-level/pooled-scan/hybrid-registration.md) (#1142); the scanner change not adopted, [the i9 verdict](https://github.com/lmmx/mibz2/blob/6e8cc8e0f9c54dcca55b89cbf9f70802b1f30ab1/frontier/deltas/2026-11-27-scanner-seam-window-i9-verdict.md) (#1118).
