---
title: "The decoder's true floor and the limits of a pooled scan"
image: assets/social/blog-2026-11-26-true-floor-and-pooled-scan.png
description: "Zero-copy stand-ins drop mibz2's plumbing floor to 5 milliseconds and uncover two hidden costs, while pooling the scan pays off only when workers have slack."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The decoder's true floor and the limits of a pooled scan

Zero-copy stand-ins drop mibz2's plumbing floor to 5 milliseconds and uncover two hidden costs, while pooling the scan pays off only when workers have slack.
{: .post-dek }

26 November 2026 · 18 min read · [Empirical results](#6-keeping-the-verdict-and-what-comes-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

In [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md), we replaced each stage of
mibz2's bzip2 decoder with a pre-recorded stand-in called an *oracle*. That factorial experiment
showed where new algorithms could still speed up decoding. Its technical companion,
[From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md), turned
those measurements into four candidate designs, including a
[pooled scan](2026-11-26-from-bottlenecks-to-better-algorithms.md#9-dissolving-the-serial-scanner).
Together, those two posts left two empirical questions open before any large redesign could start.
First, how much of the earlier all-ideal floor belonged to mibz2's pipeline rather than to oracle
copying? Second, would moving the serial Huffman scan into the worker pool unlock the 36% gain
measured when scanning and construction improve together?

A short recap grounds both questions for a reader arriving cold. Decoding a bzip2 file moves each
compressed block through six stages. One serial thread decodes
[Huffman symbols](2026-11-05-huffman-bit-buffer.md) (`S`). A pool of parallel workers then undoes
move-to-front coding and zero-run encoding (`M`) and builds the
[interval-exchange](2026-11-10-rauzy-veech-reconstruction.md) structures (`C`). Each worker walks
those structures and expands their words into bytes (`E`, called *emission* in
[Open Returns](2026-11-19-open-returns.md)), then finishes run-length expansion and checksums (`R`).
Finally, an in-order reader copies completed blocks to the caller (`D`). As shown in
[The decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md), adding workers
speeds decoding up until the
[bottleneck knee](2026-11-25-the-decoder-as-a-queueing-network.md#5-the-bottleneck-law) near 7.7
workers. Below the knee, workers set the pace. At eight workers, where mibz2 ships on our 10-core
Intel i9-10900K machine, the serial scanner and the workers are almost evenly matched.

Both open questions now have direct measurements on the same four-stream Wikidata file. That
workload holds 47.4 MB compressed and expands to 838.8 MB uncompressed. Removing oracle copying and
emission's leftover word expansion drops mibz2's all-ideal floor to roughly 5 ms at every worker
count. It also refutes the suspected memory-bandwidth bottleneck between 8 and 10 workers. At the
same time, it uncovers two large costs the first matrix could not see. Emission's word spelling
takes 298 ms at 1 worker, and delivery's serial copy-out takes 48 ms flat at every worker count.
Testing the pooled scan behind ideal construction then ends in a narrow kill under its
pre-registered gate. Pooling fails to remove scanning CPU; it merely moves that work onto workers
that are already busy below nine threads.

This post reports both experiments and their consequences.
[Section 1](#1-how-much-of-the-first-matrixs-floor-was-the-decoders) separates pipeline plumbing from
oracle copying and settles the memory-bandwidth question.
[Section 2](#2-what-remains-when-one-stage-at-a-time-is-real) measures each stage alone real and
quantifies the two newly exposed costs.
[Section 3](#3-designing-a-pooled-scan-and-testing-a-masked-stage) explains the prototype and why
we tested it behind ideal construction.
[Section 4](#4-the-kill-tests-verdict-kill-narrowly-on-k1) presents every gate from the kill-test
run.
[Section 5](#5-why-pooling-was-not-enough) derives the queueing arithmetic behind the narrow miss
and the 10-worker crossover.
[Section 6](#6-keeping-the-verdict-and-what-comes-next) explains why the frozen verdict stands and
sets out what we build next.

**Takeaways**

- **Pipeline plumbing is negligible.** With zero-copy stand-ins replacing every stage and word
  spelling removed too, decoding takes 5.2 to 5.7 ms across every worker count. That is 2.7% of the
  shipped 194.6 ms wall at 8 workers. Queues, locks and hand-offs need no work.
- **Memory bandwidth did not bind from 8 to 10 workers.** Removing 4.9 GB of oracle copying
  flattened the all-ideal wall with spelling kept to 42.4 ms at 8 workers and 40.5 ms at 10. Live
  memory traffic stayed at 13 to 17 GB/s. The earlier rise from 112 to 142 ms was a copying
  artifact.
- **The serial scan caps worker-only gains at 8% at 8 workers, and two hidden costs sit beneath it.**
  Keeping only the scan real takes 179.1 ms at 8 workers, or 92% of the shipped wall. Beneath it
  sits emission's scalable word spelling, taking 298 ms at 1 worker and 37 ms at 8. Delivery adds a
  flat 48 ms serial copy-out across all worker counts for 839 MB at roughly 17 GB/s.
- **The pooled scan missed its frozen gate narrowly, despite clean plumbing.** All 10 exactness
  tests passed with zero dropped blocks and zero fallbacks. Peak memory stayed within 1.4% (passing
  K3), and the AVX2 marker finder took 4.82 ms at 9.84 GB/s (under half of K4's limit). Behind ideal
  construction, pooling captured 52% of the scan's room at 8 workers and 48% at 9. Its 95%
  confidence ceilings missed K1's half-capture bar by 0.009 and 0.042.
- **Pooling moves scanning CPU onto workers instead of removing it, so it wins only from slack.**
  Pooled workers spent 173 to 193 ms scanning, matching the serial scanner's 171 to 190 ms. That
  extra load slowed standalone decoding below nine workers and broke even at 9 (`1.000`). Only at 10
  workers, where shipped workers sat 20.2% idle, did pooling win by 6.4%.

## 1. How much of the first matrix's floor was the decoder's?

In the first 44-arm matrix, making all six parts ideal left a steep residual wall. It took 391 ms at
1 worker and 94 ms at 6, then climbed from 112 ms at 8 to 142 ms at 10. Two artifacts contaminated
those readings. First, the original oracles copied about 4.9 GB of pre-recorded answers into the
decoder under global locks. Second, the emission oracle skipped the graph walk yet left word-to-byte
expansion running inside each worker. Because those costs were mixed together, the first matrix
could not tell whether mibz2's pipeline plumbing was slow. It also could not tell whether main
memory formed a third bottleneck above 8 workers.

To separate plumbing, copying and word spelling directly, we built zero-copy borrow oracles in
[PR #921](https://github.com/lmmx/mibz2/pull/921). We then ran a 16-minute floor diagnostic in
[PR #917](https://github.com/lmmx/mibz2/pull/917), archived in
[the floor results](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md).
Instead of copying recorded buffers, each borrow oracle hands the next stage a borrowed slice. We
also added a setting `F` (`E_full`) that skips word spelling alongside the graph walk.

<aside class="il-gloss" markdown>
**Zero-copy borrow oracle.** Rather than cloning recorded buffers under a lock for each block, a
borrow oracle hands the downstream stage a read-only reference to pre-recorded memory. That strips
replay traffic off the memory bus, though the downstream stage then reads cache lines that the real
upstream stage would have warmed.
</aside>

The run executed 648 decodes on the i9-10900K at six worker counts (`1` and `6` through `10`). Each
cell ran across 8 randomised blocks, with traces on the first 4 and the scanner pinned to CPU 6.
Every validity check passed. Every decode produced byte-identical output, and the scanner pin held
in all 216 pinned traces. Removed parts kept at most 0.15% of their real busy time, well under the
5% void threshold. No `F` arm called word spelling, and live memory-controller counters stayed
active throughout.

| Workers | Residual A (`SECMRD`, spelling kept) | Residual B (`SFCMRD`, plumbing alone) | Spelling `E_consume` (`A − B`) | Delivery copy-out (`SFCMR − B`) |
|---:|---:|---:|---:|---:|
| 1 | 303.9 [302.9, 305.0] ms | 5.7 [5.5, 5.8] ms | 298.2 [297.0, 299.4] ms | 48.5 [46.8, 50.3] ms |
| 6 | 54.0 [51.2, 56.7] ms | 5.3 [5.3, 5.4] ms | 48.7 [45.9, 51.4] ms | 47.9 [47.1, 48.8] ms |
| 7 | 46.7 [45.1, 48.2] ms | 5.2 [4.9, 5.5] ms | 41.5 [40.0, 43.0] ms | 48.5 [47.7, 49.3] ms |
| 8 | **42.4 [40.3, 44.4] ms** | **5.2 [5.0, 5.5] ms** | **37.2 [35.2, 39.1] ms** | **47.3 [46.4, 48.2] ms** |
| 9 | 42.9 [40.4, 45.3] ms | 5.3 [5.1, 5.5] ms | 37.6 [35.1, 40.1] ms | 49.1 [48.0, 50.2] ms |
| 10 | **40.5 [37.6, 43.5] ms** | **5.4 [5.3, 5.5] ms** | **35.2 [32.3, 38.0] ms** | **49.3 [46.8, 51.8] ms** |

Two results follow straight from that table. First, mibz2's pipeline plumbing is negligible. Arm
`SFCMRD` leaves every stage free and skips word spelling, matching Residual B in
[the companion's accounting](2026-11-26-from-bottlenecks-to-better-algorithms.md#3-accounting-for-the-remaining-computation).
Across all worker counts it takes just 5.2 to 5.7 ms. At 8 workers the block mean is 5.2 ms and the
median is 5.3 ms. That is 2.7% of the shipped 194.6 ms wall. In `SECMRD` traces, worker handovers
cost 0.96 ms per thread at 1 worker and 0.15 ms at 8. Both sit under half a percent of Residual A.
The design analysis had forecast 14 to 19 ms at 1 worker. Measured at 5.7 ms, that prediction is
scored a clean miss in the right direction, and plumbing is closed as a target.

Second, memory bandwidth did not bind between 8 and 10 workers. Arm `SECMRD` removes oracle copying
while keeping word spelling (Residual A). Its wall time falls from 54.0 ms at 6 workers to 46.7 ms
at 7. From 8 to 10 workers it stays flat at 42.4 ms, 42.9 ms and 40.5 ms (full 95% intervals sit in
the table above). Live hardware counters confirm why. Total memory traffic stays between 11.9 and
13.1 GB/s in `SFCMRD`. In `SECMRD` it stays between 15.2 and 16.6 GB/s across 8 to 10 workers.
The shipped decoder draws 15.1 GB/s at 10 workers, nowhere near saturation. The first matrix's climb
from 112 to 142 ms was caused by the old copying oracles. The three-station memory hypothesis is not
supported here.

<figure class="bf-fig" id="taring">
<p class="bf-title">Taring the scale: removing oracle copying exposes the 5 ms plumbing floor and each stage's isolated cost</p>
<!-- fig:taring -->
<svg viewBox="0 0 720 362" role="img" aria-label="Two-panel empirical chart from the zero-copy floor run at 8 workers. Panel A shows the all-ideal wall dropping from 112 ms in the first matrix to 42.4 ms with zero-copy oracles (Residual A), splitting into 37.2 ms of emission word spelling and 5.2 ms (2.7 percent) of pipeline plumbing (Residual B). An inset shows the 8 to 10 worker trend: the first matrix rose from 112 to 142 ms, whereas zero-copy Residual A stays flat at 42.4, 42.9, and 40.5 ms at 15 to 17 GB/s. Panel B shows each single stage real at 8 workers against the 194.6 ms shipped wall: scan 179.1 ms (92 percent), emission 98.1 ms, construction 83.6 ms (cold-store caveat), delivery 52.3 ms, MTF/RLE2 20.1 ms, RLE1/CRC about 14 ms, and nothing 5.3 ms."><text class="bf-t-strong" x="14.0" y="20.0" text-anchor="start" style="font-size:12px">A. Taring the 8-worker all-ideal floor</text><text class="bf-tm" x="14.0" y="35.0" text-anchor="start" style="font-size:10px">Removing 4.9 GB of oracle copies and word spelling (ms)</text><line x1="132.0" y1="48.0" x2="132.0" y2="158.0" style="stroke:var(--bf-vz-axis);stroke-width:1.0"/><text class="bf-tm" x="132.0" y="171.0" text-anchor="middle" style="font-size:9.5px">0</text><line x1="198.7" y1="48.0" x2="198.7" y2="158.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="198.7" y="171.0" text-anchor="middle" style="font-size:9.5px">50</text><line x1="265.3" y1="48.0" x2="265.3" y2="158.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="265.3" y="171.0" text-anchor="middle" style="font-size:9.5px">100</text><line x1="332.0" y1="48.0" x2="332.0" y2="158.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="332.0" y="171.0" text-anchor="middle" style="font-size:9.5px">150</text><text class="bf-tm" x="332.0" y="171.0" text-anchor="start" style="font-size:9.5px">ms</text><text class="bf-t-strong" x="125.0" y="62.0" text-anchor="end" style="font-size:9.8px">1st matrix all-ideal</text><text class="bf-tm" x="125.0" y="73.0" text-anchor="end" style="font-size:8.6px">copying oracles + spelling</text><path d="M132.0,55.0 L277.3,55.0 A4.0,4.0 0 0 1 281.3,59.0 L281.3,67.0 A4.0,4.0 0 0 1 277.3,71.0 L132.0,71.0 Z" style="fill:var(--bf-vz-2)"><title>First matrix all-ideal at 8 workers: 112 ms (includes ~4.9 GB oracle copying + word spelling)</title></path><text class="bf-t-strong" x="275.3" y="67.0" text-anchor="end" style="font-size:9.5px;fill:var(--bf-vz-2-ink)">112 ms</text><text class="bf-t-strong" x="125.0" y="90.0" text-anchor="end" style="font-size:9.8px">Zero-copy (SECMRD)</text><text class="bf-tm" x="125.0" y="101.0" text-anchor="end" style="font-size:8.6px">Residual A, spelling kept</text><path d="M132.0,83.0 L184.5,83.0 A4.0,4.0 0 0 1 188.5,87.0 L188.5,95.0 A4.0,4.0 0 0 1 184.5,99.0 L132.0,99.0 Z" style="fill:var(--bf-vz-1)"><title>Residual A (SECMRD, zero-copy) at 8 workers: 42.4 [40.3, 44.4] ms (15.2 GB/s)</title></path><text class="bf-t-strong" x="182.5" y="95.0" text-anchor="end" style="font-size:9.5px;fill:var(--bf-vz-1-ink)">42.4 ms</text><text class="bf-t-strong" x="125.0" y="118.0" text-anchor="end" style="font-size:9.8px">Word spelling (A-B)</text><text class="bf-tm" x="125.0" y="129.0" text-anchor="end" style="font-size:8.6px">E_consume inside workers</text><path d="M132.0,111.0 L177.6,111.0 A4.0,4.0 0 0 1 181.6,115.0 L181.6,123.0 A4.0,4.0 0 0 1 177.6,127.0 L132.0,127.0 Z" style="fill:var(--bf-vz-4)"><title>Emission word spelling (E_consume = A - B) at 8 workers: 37.2 [35.2, 39.1] ms (88% of Residual A)</title></path><text class="bf-t-strong" x="186.6" y="123.0" text-anchor="start" style="font-size:9.5px">37.2 ms</text><text class="bf-t-strong" x="125.0" y="146.0" text-anchor="end" style="font-size:9.8px">Plumbing (SFCMRD)</text><text class="bf-tm" x="125.0" y="157.0" text-anchor="end" style="font-size:8.6px">Residual B, all free</text><path d="M132.0,139.0 L134.9,139.0 A4.0,4.0 0 0 1 138.9,143.0 L138.9,151.0 A4.0,4.0 0 0 1 134.9,155.0 L132.0,155.0 Z" style="fill:var(--bf-vz-3)"><title>Residual B (SFCMRD, plumbing alone) at 8 workers: 5.2 [5.0, 5.5] ms (2.7% of shipped 194.6 ms)</title></path><text class="bf-t-strong" x="143.9" y="151.0" text-anchor="start" style="font-size:9.5px">5.2 ms (2.7%)</text><rect x="14.0" y="188.0" width="336.0" height="162.0" rx="7" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.1"></rect><text class="bf-t-strong" x="26.0" y="206.0" text-anchor="start" style="font-size:10.5px">8 to 10 workers: the rising floor vanished</text><text class="bf-tm" x="26.0" y="220.0" text-anchor="start" style="font-size:9.2px">All-ideal wall (ms); bars show 95% intervals</text><line x1="56.0" y1="324.0" x2="232.0" y2="324.0" style="stroke:var(--bf-vz-axis);stroke-width:1.0"/><text class="bf-tm" x="50.0" y="327.0" text-anchor="end" style="font-size:9.0px">0</text><line x1="56.0" y1="294.0" x2="232.0" y2="294.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="50.0" y="297.0" text-anchor="end" style="font-size:9.0px">50</text><line x1="56.0" y1="264.0" x2="232.0" y2="264.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="50.0" y="267.0" text-anchor="end" style="font-size:9.0px">100</text><line x1="56.0" y1="234.0" x2="232.0" y2="234.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="50.0" y="237.0" text-anchor="end" style="font-size:9.0px">150</text><line x1="56.0" y1="234.0" x2="56.0" y2="324.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="56.0" y="338.0" text-anchor="middle" style="font-size:9.5px">8W</text><line x1="144.0" y1="234.0" x2="144.0" y2="324.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="144.0" y="338.0" text-anchor="middle" style="font-size:9.5px">9W</text><line x1="232.0" y1="234.0" x2="232.0" y2="324.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="232.0" y="338.0" text-anchor="middle" style="font-size:9.5px">10W</text><line x1="56.0" y1="256.8" x2="232.0" y2="238.8" style="stroke:var(--bf-vz-2);stroke-width:2.0;stroke-dasharray:5 3"/><circle cx="56.0" cy="256.8" r="3.5" style="fill:var(--bf-vz-2)"><title>First matrix all-ideal at 8 workers: 112 ms (copying artifact)</title></circle><circle cx="232.0" cy="238.8" r="3.5" style="fill:var(--bf-vz-2)"><title>First matrix all-ideal at 10 workers: 142 ms (copying artifact)</title></circle><text class="bf-t-strong" x="239.0" y="241.8" text-anchor="start" style="font-size:9.2px;fill:var(--bf-amber)">1st matrix</text><text class="bf-tm" x="239.0" y="252.8" text-anchor="start" style="font-size:8.8px">112-&gt;142 ms</text><polyline points="56.0,298.6 144.0,298.3 232.0,299.7" style="fill:none;stroke:var(--bf-vz-1);stroke-width:2.0"/><line x1="56.0" y1="299.8" x2="56.0" y2="297.4" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="56.0" cy="298.6" r="3.5" style="fill:var(--bf-vz-1)"><title>Zero-copy SECMRD at 8 workers: 42.4 [40.3, 44.4] ms</title></circle><line x1="144.0" y1="299.8" x2="144.0" y2="296.8" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="144.0" cy="298.3" r="3.5" style="fill:var(--bf-vz-1)"><title>Zero-copy SECMRD at 9 workers: 42.9 [40.4, 45.3] ms</title></circle><line x1="232.0" y1="301.4" x2="232.0" y2="297.9" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="232.0" cy="299.7" r="3.5" style="fill:var(--bf-vz-1)"><title>Zero-copy SECMRD at 10 workers: 40.5 [37.6, 43.5] ms</title></circle><text class="bf-t-strong" x="239.0" y="295.7" text-anchor="start" style="font-size:9.2px;fill:var(--bf-blue)">Zero-copy (A)</text><text class="bf-tm" x="239.0" y="307.7" text-anchor="start" style="font-size:8.8px">42.4-&gt;40.5 ms</text><text class="bf-tm" x="239.0" y="318.7" text-anchor="start" style="font-size:8.5px">15-17 GB/s</text><polyline points="56.0,320.9 144.0,320.8 232.0,320.8" style="fill:none;stroke:var(--bf-vz-3);stroke-width:1.8"/><circle cx="56.0" cy="320.9" r="3.0" style="fill:var(--bf-vz-3)"><title>Zero-copy SFCMRD (plumbing) at 8 workers: 5.2 [5.0, 5.5] ms</title></circle><circle cx="144.0" cy="320.8" r="3.0" style="fill:var(--bf-vz-3)"><title>Zero-copy SFCMRD (plumbing) at 9 workers: 5.3 [5.1, 5.5] ms</title></circle><circle cx="232.0" cy="320.8" r="3.0" style="fill:var(--bf-vz-3)"><title>Zero-copy SFCMRD (plumbing) at 10 workers: 5.4 [5.3, 5.5] ms</title></circle><text class="bf-t-strong" x="239.0" y="322.8" text-anchor="start" style="font-size:8.8px;fill:var(--bf-green-ink)">Plumbing: 5.2-5.4</text><line x1="362.0" y1="12.0" x2="362.0" y2="356.0" style="stroke:var(--bf-frame);stroke-width:1.0"/><text class="bf-t-strong" x="376.0" y="20.0" text-anchor="start" style="font-size:12px">B. One stage real, all else free (8 workers)</text><text class="bf-tm" x="376.0" y="35.0" text-anchor="start" style="font-size:10px">Median wall time (ms) against shipped 194.6 ms</text><line x1="492.0" y1="50.0" x2="492.0" y2="308.0" style="stroke:var(--bf-vz-axis);stroke-width:1.0"/><text class="bf-tm" x="492.0" y="323.0" text-anchor="middle" style="font-size:9.5px">0</text><line x1="539.0" y1="50.0" x2="539.0" y2="308.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="539.0" y="323.0" text-anchor="middle" style="font-size:9.5px">50</text><line x1="586.0" y1="50.0" x2="586.0" y2="308.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="586.0" y="323.0" text-anchor="middle" style="font-size:9.5px">100</text><line x1="632.9" y1="50.0" x2="632.9" y2="308.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="632.9" y="323.0" text-anchor="middle" style="font-size:9.5px">150</text><line x1="679.9" y1="50.0" x2="679.9" y2="308.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="679.9" y="323.0" text-anchor="middle" style="font-size:9.5px">200</text><text class="bf-tm" x="696.0" y="323.0" text-anchor="start" style="font-size:9.5px">ms</text><line x1="674.8" y1="48.0" x2="674.8" y2="308.0" style="stroke:var(--bf-ink);stroke-width:1.3;stroke-dasharray:4 3"/><text class="bf-t-strong" x="670.8" y="58.0" text-anchor="end" style="font-size:9.0px">shipped 194.6 ms</text><text class="bf-t-strong" x="484.0" y="78.0" text-anchor="end" style="font-size:9.8px">Scan (FCMRD)</text><text class="bf-tm" x="484.0" y="89.0" text-anchor="end" style="font-size:8.6px">serial; 92% of wall</text><path d="M492.0,71.0 L656.3,71.0 A4.0,4.0 0 0 1 660.3,75.0 L660.3,85.0 A4.0,4.0 0 0 1 656.3,89.0 L492.0,89.0 Z" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 8 workers: median 179.1 ms (92% of shipped 194.6 ms; caps worker-only gains at &lt;=8%)</title></path><text class="bf-t-strong" x="654.3" y="84.0" text-anchor="end" style="font-size:9.6px;fill:var(--bf-vz-2-ink)">179.1 ms</text><text class="bf-t-strong" x="484.0" y="112.0" text-anchor="end" style="font-size:9.8px">Emission (SCMRD)</text><text class="bf-tm" x="484.0" y="123.0" text-anchor="end" style="font-size:8.6px">walk + word spelling</text><path d="M492.0,105.0 L580.2,105.0 A4.0,4.0 0 0 1 584.2,109.0 L584.2,119.0 A4.0,4.0 0 0 1 580.2,123.0 L492.0,123.0 Z" style="fill:var(--bf-vz-1)"><title>Emission alone real (SCMRD, walk + spelling) at 8 workers: median 98.1 ms</title></path><text class="bf-t-strong" x="578.2" y="118.0" text-anchor="end" style="font-size:9.6px;fill:var(--bf-vz-1-ink)">98.1 ms</text><text class="bf-t-strong" x="484.0" y="146.0" text-anchor="end" style="font-size:9.8px">Construction (SFMRD)*</text><text class="bf-tm" x="484.0" y="157.0" text-anchor="end" style="font-size:8.6px">cold-store caveat</text><path d="M492.0,139.0 L566.5,139.0 A4.0,4.0 0 0 1 570.5,143.0 L570.5,153.0 A4.0,4.0 0 0 1 566.5,157.0 L492.0,157.0 Z" style="fill:var(--bf-vz-1)"><title>Construction alone real (SFMRD) at 8 workers: median 83.6 ms (overstated because zero-copy borrow leaves walk reading cold)</title></path><text class="bf-t-strong" x="564.5" y="152.0" text-anchor="end" style="font-size:9.6px;fill:var(--bf-vz-1-ink)">83.6 ms*</text><text class="bf-t-strong" x="484.0" y="180.0" text-anchor="end" style="font-size:9.8px">Delivery (SFCMR)</text><text class="bf-tm" x="484.0" y="191.0" text-anchor="end" style="font-size:8.6px">serial 839 MB copy-out</text><path d="M492.0,173.0 L537.1,173.0 A4.0,4.0 0 0 1 541.1,177.0 L541.1,187.0 A4.0,4.0 0 0 1 537.1,191.0 L492.0,191.0 Z" style="fill:var(--bf-vz-4)"><title>Delivery alone real (SFCMR) at 8 workers: median 52.3 ms (47.3 ms copy-out above 5.2 ms floor; 17.5 GB/s)</title></path><text class="bf-t-strong" x="546.1" y="186.0" text-anchor="start" style="font-size:9.6px">52.3 ms</text><text class="bf-t-strong" x="484.0" y="214.0" text-anchor="end" style="font-size:9.8px">MTF / RLE2 (SFCRD)</text><text class="bf-tm" x="484.0" y="225.0" text-anchor="end" style="font-size:8.6px">parallel worker stage</text><path d="M492.0,207.0 L506.9,207.0 A4.0,4.0 0 0 1 510.9,211.0 L510.9,221.0 A4.0,4.0 0 0 1 506.9,225.0 L492.0,225.0 Z" style="fill:var(--bf-steel)"><title>MTF/RLE2 alone real (SFCRD) at 8 workers: median 20.1 ms</title></path><text class="bf-t-strong" x="515.9" y="220.0" text-anchor="start" style="font-size:9.6px">20.1 ms</text><text class="bf-t-strong" x="484.0" y="248.0" text-anchor="end" style="font-size:9.8px">RLE1 / CRC (R)</text><text class="bf-tm" x="484.0" y="259.0" text-anchor="end" style="font-size:8.6px">SECMD minus SECMRD</text><path d="M492.0,241.0 L501.3,241.0 A4.0,4.0 0 0 1 505.3,245.0 L505.3,255.0 A4.0,4.0 0 0 1 501.3,259.0 L492.0,259.0 Z" style="fill:var(--bf-steel)"><title>RLE1/CRC/output at 8 workers: SECMD 55.0 ms minus SECMRD 40.8 ms (~14 ms)</title></path><text class="bf-t-strong" x="510.3" y="254.0" text-anchor="start" style="font-size:9.6px">~14 ms</text><text class="bf-t-strong" x="484.0" y="282.0" text-anchor="end" style="font-size:9.8px">Nothing (SFCMRD)</text><text class="bf-tm" x="484.0" y="293.0" text-anchor="end" style="font-size:8.6px">plumbing alone (2.7%)</text><path d="M492.0,275.0 L493.0,275.0 A4.0,4.0 0 0 1 497.0,279.0 L497.0,289.0 A4.0,4.0 0 0 1 493.0,293.0 L492.0,293.0 Z" style="fill:var(--bf-vz-3)"><title>All stages free (SFCMRD) at 8 workers: median 5.3 ms (mean 5.2 [5.0, 5.5] ms, 2.7% of shipped)</title></path><text class="bf-t-strong" x="502.0" y="288.0" text-anchor="start" style="font-size:9.6px">5.3 ms</text><text class="bf-tm" x="376.0" y="344.0" text-anchor="start" style="font-size:9.0px">*Walk (A7) runs 98.9 vs 72.4 ms/worker on borrowed stores, overstating C.</text></svg>
<figcaption>Measured data (zero-copy floor run, i9-10900K, wikidata ×4; first-matrix baseline from the 44-arm run). Visual analogy: taring a scale removes the shipping crates (4.9 GB of oracle copies and unremoved word spelling), revealing how light the tray itself is (5.2 ms) and how heavy each stage is alone. Where the analogy stops: pipeline stages overlap in time rather than summing like static weights (179 + 98 + 84 ms exceeds the 195 ms shipped wall), and zero-copy borrowing leaves construction's store cache-cold for the walk.</figcaption>
</figure>

## 2. What remains when one stage at a time is real?

Once the scale is tared down to 5.3 ms, enabling one real stage at a time exposes its isolated cost.
The table below lists median wall times at 8 workers and 1 worker beside the shipped decoder.

| Single real stage (all else free) | Arm | Median wall at 8 workers | Median wall at 1 worker |
|---|---|---:|---:|
| Shipped decoder (all real) | `none` | 194.6 ms | 1,342.7 ms |
| Scan | `FCMRD` | 179.1 ms (92% of wall) | 169.7 ms |
| Emission (walk + word spelling) | `SCMRD` | 98.1 ms | 694.2 ms |
| Construction (cold-store caveat) | `SFMRD` | 83.6 ms | 609.5 ms |
| Delivery | `SFCMR` | **52.3 ms** | **54.7 ms** |
| MTF / RLE2 | `SFCRD` | 20.1 ms | 141.1 ms |
| RLE1 / CRC / output | `SECMD − SECMRD` | ~14 ms | ~66 ms |
| Nothing (plumbing alone) | `SFCMRD` | 5.3 ms (2.7% of wall) | 5.8 ms |

One caveat belongs right beside construction's row. When emission runs real against a zero-copy
borrow of construction (`SCMRD`), the emission walk (`A7`) reads construction's store cold from main
memory. In the shipped decoder, construction warms those lines in the worker's caches right before
emission walks them. In traces, `A7` takes 98.9 ms per worker at 8 workers in `SCMRD` against
72.4 ms as shipped. At 1 worker the gap is 697.9 ms against 545.2 ms. Construction's isolated row
(`SFMRD`) therefore overstates its standalone cost by roughly that warm-cache benefit. The serial
scan's row has no such cross-stage cache distortion.

Even with that caveat, three findings from this table reshape our priorities.

First, the serial scan imposes a hard ceiling at 8 workers. Running only the Huffman scan real
(`FCMRD`) takes 179.1 ms at 8 workers. That is 92% of the shipped 194.6 ms wall. Even if every
worker-side stage became free, 8-worker decoding could drop by at most 8% while scanning stays on
one thread. This sharpens the first matrix's `S × C` result into a strict ordering rule. Worker-side
optimisations cannot show their full gain at 8 workers until scanning leaves its serial station.

Second, emission's word spelling is a major hidden worker cost. Subtracting Residual B from Residual
A isolates `E_consume`, the loop that expands dictionary words into bytes. At 1 worker it takes
298.2 ms, which is 22% of the 1,342.7 ms decode. Across every worker count it accounts for 87 to 93%
of `SECMRD`. The trace ledger confirms those numbers directly at 299.7 ms for 1 worker and 39.8 ms
per thread at 8 workers. The first matrix missed this cost because its emission oracle left word
spelling in place. Because workers run spelling in parallel, its wall time scales down cleanly. It
drops from 298.2 ms at 1 worker to 37.2 ms at 8 and 35.2 ms at 10.

Third, delivery's copy-out is a flat 48 ms serial floor. Subtracting `SFCMRD` from `SFCMR` yields
47.3 to 49.3 ms at every worker count: 48.5 ms at 1 worker, 47.3 ms at 8, and 49.3 ms at 10.
On one thread, the in-order reader copies 838.8 MB of finished output to the caller at roughly
17 GB/s. That copy incurs about 5 million last-level cache load misses against 0.3 million in
`SFCMRD`. Adding workers does not shrink it. Today delivery overlaps with the slower scan and worker
stages, so it stays out of sight. Once scanning leaves its single thread and workers get faster,
this 48 ms serial copy becomes the next floor.

<figure class="bf-fig" id="reservoir">
<p class="bf-title">Draining the reservoir: scalable word spelling drops past delivery's 48 ms serial ledge</p>
<!-- fig:reservoir -->
<svg viewBox="0 0 720 320" role="img" aria-label="Line chart across 1, 6, 7, 8, 9, and 10 workers showing emission word spelling draining as workers increase, from 298.2 ms at 1 worker (22 percent of wall) down to 48.7, 41.5, 37.2, 37.6, and 35.2 ms, against three flat serial floors: scan alone at 169.7 to 179.1 ms, delivery serial copy-out at 47.3 to 49.3 ms (839 MB at roughly 17 GB/s), and pipeline plumbing at 5.2 to 5.7 ms."><text class="bf-t-strong" x="16.0" y="20.0" text-anchor="start" style="font-size:12px">Scalable word spelling drains past three flat serial ledges (1 to 10 workers)</text><text class="bf-tm" x="16.0" y="36.0" text-anchor="start" style="font-size:10px">Measured wall time (ms) on wikidata x4 (i9-10900K); vertical bars show 95% intervals</text><line x1="56.0" y1="278.0" x2="562.0" y2="278.0" style="stroke:var(--bf-vz-axis);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="281.5" text-anchor="end" style="font-size:9.8px">0</text><line x1="56.0" y1="243.3" x2="562.0" y2="243.3" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="246.8" text-anchor="end" style="font-size:9.8px">50</text><line x1="56.0" y1="208.6" x2="562.0" y2="208.6" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="212.1" text-anchor="end" style="font-size:9.8px">100</text><line x1="56.0" y1="173.9" x2="562.0" y2="173.9" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="177.4" text-anchor="end" style="font-size:9.8px">150</text><line x1="56.0" y1="139.2" x2="562.0" y2="139.2" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="142.8" text-anchor="end" style="font-size:9.8px">200</text><line x1="56.0" y1="104.6" x2="562.0" y2="104.6" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="108.1" text-anchor="end" style="font-size:9.8px">250</text><line x1="56.0" y1="69.9" x2="562.0" y2="69.9" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="48.0" y="73.4" text-anchor="end" style="font-size:9.8px">300</text><text class="bf-tm" x="16.0" y="52.0" text-anchor="start" style="font-size:9.8px">ms</text><line x1="145.0" y1="283.0" x2="150.0" y2="273.0" style="stroke:var(--bf-vz-axis);stroke-width:1.2"/><line x1="150.0" y1="283.0" x2="155.0" y2="273.0" style="stroke:var(--bf-vz-axis);stroke-width:1.2"/><line x1="86.0" y1="56.0" x2="86.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-t-strong" x="86.0" y="294.0" text-anchor="middle" style="font-size:10.2px">1W</text><line x1="212.0" y1="56.0" x2="212.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="212.0" y="294.0" text-anchor="middle" style="font-size:10.2px">6W</text><line x1="294.0" y1="56.0" x2="294.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="294.0" y="294.0" text-anchor="middle" style="font-size:10.2px">7W</text><line x1="376.0" y1="56.0" x2="376.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-t-strong" x="376.0" y="294.0" text-anchor="middle" style="font-size:10.2px">8W</text><line x1="458.0" y1="56.0" x2="458.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="458.0" y="294.0" text-anchor="middle" style="font-size:10.2px">9W</text><line x1="540.0" y1="56.0" x2="540.0" y2="278.0" style="stroke:var(--bf-vz-grid);stroke-width:0.8"/><text class="bf-tm" x="540.0" y="294.0" text-anchor="middle" style="font-size:10.2px">10W</text><text class="bf-tm" x="310.0" y="310.0" text-anchor="middle" style="font-size:9.8px">Worker threads (break between 1 and 6 workers)</text><polygon points="86.0,278.0 86.0,71.1 212.0,244.2 294.0,249.2 376.0,252.2 458.0,251.9 540.0,253.6 540.0,278.0" style="fill:var(--bf-vz-1);fill-opacity:0.12"/><polyline points="86.0,160.3 212.0,155.2 294.0,154.7 376.0,153.7 458.0,154.8 540.0,154.0" style="fill:none;stroke:var(--bf-vz-2);stroke-width:2.2;stroke-dasharray:6 3"/><circle cx="86.0" cy="160.3" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 1W: median 169.7 ms</title></circle><circle cx="212.0" cy="155.2" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 6W: median 177.0 ms</title></circle><circle cx="294.0" cy="154.7" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 7W: median 177.8 ms</title></circle><circle cx="376.0" cy="153.7" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 8W: median 179.1 ms</title></circle><circle cx="458.0" cy="154.8" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 9W: median 177.6 ms</title></circle><circle cx="540.0" cy="154.0" r="3.8" style="fill:var(--bf-vz-2)"><title>Scan alone real (FCMRD) at 10W: median 178.7 ms</title></circle><text class="bf-t-strong" x="94.0" y="151.3" text-anchor="start" style="font-size:9.4px;fill:var(--bf-amber)">169.7 ms</text><text class="bf-t-strong" x="376.0" y="143.7" text-anchor="middle" style="font-size:9.4px;fill:var(--bf-amber)">179.1 ms</text><polyline points="86.0,71.1 212.0,244.2 294.0,249.2 376.0,252.2 458.0,251.9 540.0,253.6" style="fill:none;stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="86.0" y1="72.0" x2="86.0" y2="70.3" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="86.0" cy="71.1" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 1W: 298.2 [297.0, 299.4] ms</title></circle><line x1="212.0" y1="246.2" x2="212.0" y2="242.3" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="212.0" cy="244.2" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 6W: 48.7 [45.9, 51.4] ms</title></circle><line x1="294.0" y1="250.2" x2="294.0" y2="248.2" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="294.0" cy="249.2" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 7W: 41.5 [40.0, 43.0] ms</title></circle><line x1="376.0" y1="253.6" x2="376.0" y2="250.9" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="376.0" cy="252.2" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 8W: 37.2 [35.2, 39.1] ms</title></circle><line x1="458.0" y1="253.6" x2="458.0" y2="250.2" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="458.0" cy="251.9" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 9W: 37.6 [35.1, 40.1] ms</title></circle><line x1="540.0" y1="255.6" x2="540.0" y2="251.6" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="540.0" cy="253.6" r="4.0" style="fill:var(--bf-vz-1)"><title>Emission word spelling (E_consume = A - B) at 10W: 35.2 [32.3, 38.0] ms</title></circle><text class="bf-t-strong" x="95.0" y="75.1" text-anchor="start" style="font-size:9.6px;fill:var(--bf-blue)">298.2 ms (22% of 1W wall)</text><text class="bf-t-strong" x="376.0" y="267.2" text-anchor="middle" style="font-size:9.2px;fill:var(--bf-blue)">37.2 ms</text><polyline points="86.0,244.4 212.0,244.8 294.0,244.4 376.0,245.2 458.0,243.9 540.0,243.8" style="fill:none;stroke:var(--bf-vz-4);stroke-width:2.2;stroke-dasharray:4 2"/><line x1="86.0" y1="245.5" x2="86.0" y2="243.1" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="86.0,240.4 90.0,244.4 86.0,248.4 82.0,244.4" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 1W: 48.5 [46.8, 50.3] ms (839 MB at ~17 GB/s)</title></polygon><line x1="212.0" y1="245.3" x2="212.0" y2="244.1" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="212.0,240.8 216.0,244.8 212.0,248.8 208.0,244.8" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 6W: 47.9 [47.1, 48.8] ms (839 MB at ~17 GB/s)</title></polygon><line x1="294.0" y1="244.9" x2="294.0" y2="243.8" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="294.0,240.4 298.0,244.4 294.0,248.4 290.0,244.4" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 7W: 48.5 [47.7, 49.3] ms (839 MB at ~17 GB/s)</title></polygon><line x1="376.0" y1="245.8" x2="376.0" y2="244.6" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="376.0,241.2 380.0,245.2 376.0,249.2 372.0,245.2" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 8W: 47.3 [46.4, 48.2] ms (839 MB at ~17 GB/s)</title></polygon><line x1="458.0" y1="244.7" x2="458.0" y2="243.2" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="458.0,239.9 462.0,243.9 458.0,247.9 454.0,243.9" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 9W: 49.1 [48.0, 50.2] ms (839 MB at ~17 GB/s)</title></polygon><line x1="540.0" y1="245.5" x2="540.0" y2="242.1" style="stroke:var(--bf-vz-4);stroke-width:1.4"/><polygon points="540.0,239.8 544.0,243.8 540.0,247.8 536.0,243.8" style="fill:var(--bf-surface);stroke:var(--bf-vz-4);stroke-width:1.6"><title>Delivery serial copy-out (SFCMR - SFCMRD) at 10W: 49.3 [46.8, 51.8] ms (839 MB at ~17 GB/s)</title></polygon><polyline points="86.0,274.0 212.0,274.3 294.0,274.4 376.0,274.4 458.0,274.3 540.0,274.3" style="fill:none;stroke:var(--bf-vz-3);stroke-width:2.0"/><circle cx="86.0" cy="274.0" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 1W: 5.7 [5.5, 5.8] ms</title></circle><circle cx="212.0" cy="274.3" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 6W: 5.3 [5.3, 5.4] ms</title></circle><circle cx="294.0" cy="274.4" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 7W: 5.2 [4.9, 5.5] ms</title></circle><circle cx="376.0" cy="274.4" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 8W: 5.2 [5.0, 5.5] ms</title></circle><circle cx="458.0" cy="274.3" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 9W: 5.3 [5.1, 5.5] ms</title></circle><circle cx="540.0" cy="274.3" r="3.2" style="fill:var(--bf-vz-3)"><title>Pipeline plumbing (SFCMRD) at 10W: 5.4 [5.3, 5.5] ms</title></circle><text class="bf-t-strong" x="556.0" y="151.0" text-anchor="start" style="font-size:10.0px;fill:var(--bf-amber)">Serial scan (FCMRD)</text><text class="bf-tm" x="556.0" y="164.0" text-anchor="start" style="font-size:9.2px">170-179 ms reef</text><text class="bf-t-strong" x="556.0" y="233.8" text-anchor="start" style="font-size:10.0px;fill:var(--bf-ultra)">Delivery copy-out (D)</text><text class="bf-tm" x="556.0" y="245.8" text-anchor="start" style="font-size:9.0px">47-49 ms flat (~17 GB/s)</text><text class="bf-t-strong" x="556.0" y="261.6" text-anchor="start" style="font-size:10.0px;fill:var(--bf-blue)">Word spelling (E_consume)</text><text class="bf-tm" x="556.0" y="273.6" text-anchor="start" style="font-size:9.0px">298 -&gt; 35 ms (scales)</text><text class="bf-t-strong" x="556.0" y="277.3" text-anchor="start" style="font-size:9.6px;fill:var(--bf-green-ink)">Plumbing: 5.2-5.7 ms</text><text class="bf-tm" x="222.0" y="205.2" text-anchor="start" style="font-size:9.2px">At 6W+ spelling drops below</text><text class="bf-tm" x="222.0" y="215.6" text-anchor="start" style="font-size:9.2px">delivery's 48 ms serial ledge</text><line x1="257.0" y1="220.4" x2="216.0" y2="240.5" style="stroke:var(--bf-vz-axis);stroke-width:1.1"/><polygon points="216.0,240.5 223.4,240.8 220.3,234.5" style="fill:var(--bf-vz-axis)"/></svg>
<figcaption>Measured data (zero-copy floor run, wikidata ×4 on i9-10900K; 95% intervals on E_consume, delivery copy-out and plumbing; medians on FCMRD). Visual analogy: adding workers drains the water level of parallel work (word spelling), exposing fixed rock ledges one after another — the 179 ms scan reef, the 48 ms delivery ledge, and the 5 ms riverbed. Where the analogy stops: shared-cache contention slightly raises single-thread scan time from 169.7 ms at 1 worker to 179.1 ms at 8, and delivery overlaps with worker execution until workers finish faster than the reader can copy out.</figcaption>
</figure>

## 3. Designing a pooled scan and testing a masked stage

With the floor settled and the 179.1 ms serial scan confirmed at 8 workers, our first build target
was clear. We set out to dissolve the serial scanner into the worker pool, as sketched in
[Section 9 of the companion post](2026-11-26-from-bottlenecks-to-better-algorithms.md#9-dissolving-the-serial-scanner).
Before writing code, we audited three earlier parallel-scan attempts in mibz2's history
([the registration, §2](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md)).

Each earlier prototype had failed for a specific structural reason:

1. **10 November (`bench-par-scan`).** One finder thread located block markers while workers decoded
   candidates end to end. The SIMD finder proved nearly free at 9.3 GB/s (0.19% of decode). Decoding
   sped up by 1.615× on level-1 files and 1.780× on records. On level-9 Wikidata, workers were still
   much slower than the scanner, so 10-worker speed sat at 0.981.
2. **22 November (`bench-parscan-c37`, [PR #741](https://github.com/lmmx/mibz2/pull/741)).** Workers
   scanned slices of the whole input for markers, decoded candidates, and passed blocks to a joiner
   thread. It beat shipped speed at 8 and 10 workers (`0.935` and `0.883`), yet slowed two to six
   workers (`1.11–1.13`). Worse, buffering the full input and holding $4 \times \text{threads}$
   blocks in flight inflated peak memory. Peak RSS climbed to 1.28× at 10 workers and 1.99× at 20.
3. **23 November (`bench-parscan-stream`, [PR #770](https://github.com/lmmx/mibz2/pull/770)).** A
   bounded streaming version capped memory by placing one lead scanner and two helper scanners ahead
   of the worker pool. Instead of speeding up, it slowed eight to twenty workers by 54 to 61%
   (`1.605×` at 8 workers). Per-thread CPU logs explained the collapse. The lead scanner spent about
   0.19 s and each helper added 0.08 s, nearly doubling total scan CPU against 0.17 s shipped. The
   three scanners serialised behind one chain head and left workers roughly 40% idle.

Our new prototype ([PR #932](https://github.com/lmmx/mibz2/pull/932)) avoided all three traps. As
compressed chunks arrive, the feeder thread runs an AVX2 nibble filter to locate candidate block
markers in bounded memory. No scanner thread, helper scanner or joiner thread exists. Each worker
claims one candidate inside the shipped admission window and runs `scan · M · C · E · R` end to end.
That eliminates the 190.6 MiB symbol queue between scanner and workers. The in-order reader accepts
whichever ready block starts at the chain's exact next bit offset. On any false or missed marker,
the reader falls back serially so exact bytes and exact errors always hold. At 1 worker, the
prototype keeps the shipped path unchanged.

<aside class="il-gloss" markdown>
**48-bit block marker.** Every bzip2 block opens with the first twelve digits of $\pi$ in
binary-coded decimal (`0x314159265359`). Because blocks are bit-packed rather than byte-aligned, a
boundary finder checks all eight bit shifts. The AVX2 nibble filter screens non-matching bytes at
nearly 10 GB/s, and the reader verifies every decoded block against the exact bit-offset chain.
</aside>

Testing this prototype fairly meant avoiding a measurement trap explained in
[Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md). The design
analysis had proposed killing any pooled scan that failed to beat shipped 8-worker decoding by 7%
standalone. That test was impossible by construction. Even an ideal zero-cost scanner saves only
3.2% alone at 8 workers, because construction still limits the workers.

Instead, we froze a paired-oracle kill test in
[the registration](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md)
before writing any code. We paired the pooled scan `P` with construction's zero-copy oracle `C`,
giving arm `PC`. We then compared `PC/C` against an ideal zero-copy scan behind `C` (`SC/C`). On a
log scale, this asks whether pooling captures at least half of the multiplicative speedup that an
ideal scan achieves once construction is free:

$$
\text{capture} = \frac{\ln(C / PC)}{\ln(C / SC)}, \qquad \text{K1 kills if } \text{UB}_{95\%}\!\left(\frac{PC}{C}\right) > \sqrt{\frac{SC}{C}}.
$$

<aside class="il-gloss" markdown>
**Log-scale capture.** Because speedups multiply rather than add, half the room between `C` (ratio
1) and `SC` (ratio $SC/C$) is the geometric mean $\sqrt{SC/C}$ rather than the arithmetic midpoint.
Requiring the 95% confidence ceiling of `PC/C` to stay at or below $\sqrt{SC/C}$ asks for statistical
evidence of at least 50% capture, not merely a favourable point estimate.
</aside>

Three companion gates guarded against earlier failure modes:

- **K2 (standalone cost).** Kill if the 95% confidence floor of `P/none` exceeds `1.03` at a judged
  count.
- **K3 (memory).** Kill if median peak RSS of `P/none` exceeds `1.10` at any count from 6 to 10.
- **K4 (finder speed).** Kill if the single-threaded finder takes over `10 ms` on the 47.4 MB input.

Section 1's floor run automatically set two switches in the registration before any prototype code
ran. Because Residual A stayed flat from 8 to 10 workers, both 8 and 9 became judged counts. Both
had to pass K1 and K2. Because Residual B was 2.7% (under 10%), capture remained capped rather than
treating plumbing removal as extra scan capture.

## 4. The kill test's verdict: KILL, narrowly, on K1

The kill test ran 480 decodes across 12 randomised blocks on the i9-10900K in 15 minutes
([the pooled-scan results](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md)).
The first 3 blocks were traced unpinned, and an 11-pass pinned benchmark timed the finder alone.
Every validity check passed. All 10 exactness tests passed on the test machine. Every pooled decode
at 2 or more workers was wholly pooled, with zero dropped blocks, zero fallbacks and zero degraded
decodes.

Every resource and plumbing gate passed as well. Peak memory stayed within 1.4% of shipped across
6 to 10 workers (`0.988` to `1.014`), passing **K3** cleanly. On CPU 4, the AVX2 marker finder
scanned the 47.4 MB input in a median 4.82 ms (interval `[4.81, 5.00] ms`). It processed 9.842 GB/s
and located all 1,724 block markers plus 48 stream-end markers, under half of **K4**'s 10 ms limit.
Standalone cost passed **K2** at both judged counts. At 8 workers `P/none` was `1.052`, yet its 95%
confidence floor (`1.023`) remained under `1.03`. At 9 workers `P/none` hit exact break-even
(`1.000`). In traces, candidate copying moved 45.2 MiB in 4.32 to 7.24 ms against a ~5 ms estimate.
In-decode marker finding took 4.97 to 7.35 ms. The 190.6 MiB symbol queue disappeared, and worker
idle share (`job_wait / life`) fell from 6.5–20.2% in `none` down to 1.2–2.5% in `P`.

Yet **K1 fired at both judged counts**, killing the prototype narrowly under the frozen rule.

| Workers | Status | `PC/C` [95% CI] | Bar $\sqrt{SC/C}$ | Ideal `SC/C` | Capture | Standalone `P/none` [95% CI] |
|---:|---|---:|---:|---:|---:|---:|
| 6 | reported | 1.065 [1.040, 1.090] | 0.940 | 0.884 | −0.51 | 1.105 [1.094, 1.115] |
| 7 | reported | 0.935 [0.922, 0.948] | 0.891 | 0.794 | 0.29 | 1.100 [1.087, 1.114] |
| **8** | **judged (K1)** | **0.862 [0.848, 0.876]** | **0.867** | 0.752 | **0.52** | 1.052 [1.023, 1.081] |
| **9** | **judged (K1)** | **0.869 [0.833, 0.906]** | **0.864** | 0.746 | **0.48** | 1.000 [0.983, 1.017] |
| 10 | reported | 0.848 [0.825, 0.873] | 0.877 | 0.769 | **0.63** | **0.936 [0.909, 0.964]** |

The margin was remarkably tight. Behind ideal construction at 8 workers, pooling cut median wall
time from 183.2 ms down to 158.1 ms, against 135.3 ms for an ideal scan. Its point estimate captured
52.1% of the scan's room. Even so, its 95% ceiling (`0.8758`) sat 0.009 above the `0.8670` bar. At 9
workers (`C = 184.2 ms`, `PC = 159.9 ms`, `SC = 140.2 ms`), capture fell to 48.1%. Its ceiling
(`0.9058`) missed the `0.8636` bar by 0.042. Only at 10 workers did pooling clear the bar cleanly.
There it achieved 62.7% capture behind `C` and a 6.4% standalone win (187.2 ms against 197.6 ms).

Standalone walls also missed the companion post's unscored forecast. The design analysis had
projected 172 to 175 ms at eight and nine workers. Measured block means were 207.5 ms at 8 workers
(`206.5` to `208.6` ms) and 195.2 ms at 9 workers (`192.3` to `198.1` ms).

<figure class="bf-fig" id="killtest">
<p class="bf-title">The pooled-scan kill test: paired-oracle capture (K1) and standalone crossover (K2)</p>
<!-- fig:killtest -->
<svg viewBox="0 0 720 324" role="img" aria-label="Two-panel interval strip chart of the pooled-scan kill test across 6 to 10 workers, with 8 and 9 workers highlighted as judged and 6, 7, and 10 marked reported. Panel A shows PC/C with 95 percent intervals against the ideal SC/C floor and K1 threshold sqrt(SC/C): at 8 workers PC/C is 0.862 [0.848, 0.876] versus bar 0.867 (capture 0.52, upper bound misses by 0.009, K1); at 9 workers 0.869 [0.833, 0.906] versus bar 0.864 (capture 0.48, upper bound misses by 0.042, K1); at 10 workers 0.848 [0.825, 0.873] (capture 0.63). Panel B shows standalone P/none: 1.105 at 6, 1.100 at 7, 1.052 [1.023, 1.081] at 8 (passes K2 1.03 lower-bound gate), 1.000 [0.983, 1.017] at 9, and 0.936 [0.909, 0.964] (-6.4 percent) at 10. K3 peak RSS stays within 1.4 percent and K4 finder is 4.82 ms."><text class="bf-t-strong" x="14.0" y="20.0" text-anchor="start" style="font-size:12px">A. Paired-oracle test behind ideal C (K1)</text><text class="bf-tm" x="14.0" y="35.0" text-anchor="start" style="font-size:9.8px">Ratio PC/C (95% CI) vs K1 bar sqrt(SC/C) and ideal SC/C</text><line x1="113.2" y1="52.0" x2="113.2" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="113.2" y="258.0" text-anchor="middle" style="font-size:9.2px">0.75</text><line x1="175.6" y1="52.0" x2="175.6" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="175.6" y="258.0" text-anchor="middle" style="font-size:9.2px">0.85</text><line x1="238.0" y1="52.0" x2="238.0" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="238.0" y="258.0" text-anchor="middle" style="font-size:9.2px">0.95</text><line x1="269.1" y1="52.0" x2="269.1" y2="244.0" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-t-strong" x="269.1" y="258.0" text-anchor="middle" style="font-size:9.2px">1.00</text><line x1="300.3" y1="52.0" x2="300.3" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="300.3" y="258.0" text-anchor="middle" style="font-size:9.2px">1.05</text><line x1="368.0" y1="12.0" x2="368.0" y2="274.0" style="stroke:var(--bf-frame);stroke-width:1.0"/><text class="bf-t-strong" x="382.0" y="20.0" text-anchor="start" style="font-size:12px">B. Standalone wall ratio P/none (K2)</text><text class="bf-tm" x="382.0" y="35.0" text-anchor="start" style="font-size:9.8px">Ratio P/none (95% CI); K2 kills if LB &gt; 1.03 at 8 or 9</text><line x1="440.4" y1="52.0" x2="440.4" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="440.4" y="258.0" text-anchor="middle" style="font-size:9.2px">0.90</text><line x1="491.6" y1="52.0" x2="491.6" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="491.6" y="258.0" text-anchor="middle" style="font-size:9.2px">0.95</text><line x1="542.7" y1="52.0" x2="542.7" y2="244.0" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/><text class="bf-t-strong" x="542.7" y="258.0" text-anchor="middle" style="font-size:9.2px">1.00</text><line x1="573.3" y1="52.0" x2="573.3" y2="244.0" style="stroke:var(--bf-vz-2);stroke-width:1.4;stroke-dasharray:3 2"/><text class="bf-t-strong" x="575.3" y="49.0" text-anchor="start" style="font-size:8.8px;fill:var(--bf-amber)">K2 1.03</text><line x1="593.8" y1="52.0" x2="593.8" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="593.8" y="258.0" text-anchor="middle" style="font-size:9.2px">1.05</text><line x1="644.9" y1="52.0" x2="644.9" y2="244.0" style="stroke:var(--bf-vz-grid);stroke-width:1.0"/><text class="bf-tm" x="644.9" y="258.0" text-anchor="middle" style="font-size:9.2px">1.10</text><text class="bf-tm" x="74.0" y="71.0" text-anchor="end" style="font-size:10.0px">6 workers</text><text class="bf-tm" x="74.0" y="83.0" text-anchor="end" style="font-size:8.5px">reported</text><polygon points="196.5,67.8 200.7,72.0 196.5,76.2 192.3,72.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.6"><title>6W ideal scan behind C (SC/C): 0.8836</title></polygon><line x1="231.7" y1="62.0" x2="231.7" y2="82.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><rect x="229.7" y="62.0" width="4.0" height="20.0" rx="0" style="fill:transparent"><title>6W K1 bar sqrt(SC/C): 0.9400</title></rect><line x1="294.2" y1="72.0" x2="325.2" y2="72.0" style="stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="294.2" y1="68.0" x2="294.2" y2="76.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><line x1="325.2" y1="68.0" x2="325.2" y2="76.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="309.4" cy="72.0" r="4.2" style="fill:var(--bf-vz-1)"><title>6W PC/C: 1.0646 [1.0401, 1.0898] vs bar 0.9400 (capture -0.51)</title></circle><text class="bf-tm" x="288.2" y="75.0" text-anchor="end" style="font-size:8.8px">cap -0.51</text><text class="bf-tm" x="412.0" y="75.0" text-anchor="end" style="font-size:9.6px">6W</text><line x1="639.0" y1="72.0" x2="660.0" y2="72.0" style="stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="639.0" y1="68.0" x2="639.0" y2="76.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><line x1="660.0" y1="68.0" x2="660.0" y2="76.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="649.5" cy="72.0" r="4.2" style="fill:var(--bf-vz-1)"><title>6W standalone P/none: 1.1045 [1.0942, 1.1148]</title></circle><text class="bf-tm" x="633.0" y="75.0" text-anchor="end" style="font-size:8.8px">1.105</text><text class="bf-tm" x="74.0" y="109.0" text-anchor="end" style="font-size:10.0px">7 workers</text><text class="bf-tm" x="74.0" y="121.0" text-anchor="end" style="font-size:8.5px">reported</text><polygon points="140.8,105.8 145.0,110.0 140.8,114.2 136.6,110.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.6"><title>7W ideal scan behind C (SC/C): 0.7942</title></polygon><line x1="201.3" y1="100.0" x2="201.3" y2="120.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><rect x="199.3" y="100.0" width="4.0" height="20.0" rx="0" style="fill:transparent"><title>7W K1 bar sqrt(SC/C): 0.8912</title></rect><line x1="220.5" y1="110.0" x2="237.0" y2="110.0" style="stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="220.5" y1="106.0" x2="220.5" y2="114.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><line x1="237.0" y1="106.0" x2="237.0" y2="114.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="228.7" cy="110.0" r="4.2" style="fill:var(--bf-vz-1)"><title>7W PC/C: 0.9351 [0.9221, 0.9484] vs bar 0.8912 (capture +0.29)</title></circle><text class="bf-tm" x="243.0" y="113.0" text-anchor="start" style="font-size:8.8px">cap +0.29</text><text class="bf-tm" x="412.0" y="113.0" text-anchor="end" style="font-size:9.6px">7W</text><line x1="631.7" y1="110.0" x2="659.2" y2="110.0" style="stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="631.7" y1="106.0" x2="631.7" y2="114.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><line x1="659.2" y1="106.0" x2="659.2" y2="114.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="645.3" cy="110.0" r="4.2" style="fill:var(--bf-vz-1)"><title>7W standalone P/none: 1.1004 [1.0871, 1.1140]</title></circle><text class="bf-tm" x="625.7" y="113.0" text-anchor="end" style="font-size:8.8px">1.100</text><rect x="10.0" y="131.0" width="348.0" height="34.0" rx="4" style="fill:var(--bf-amber-tint);fill-opacity:0.55"></rect><rect x="378.0" y="131.0" width="332.0" height="34.0" rx="4" style="fill:var(--bf-amber-tint);fill-opacity:0.55"></rect><text class="bf-t-strong" x="74.0" y="147.0" text-anchor="end" style="font-size:10.0px">8 workers</text><text class="bf-t-strong" x="74.0" y="159.0" text-anchor="end" style="font-size:8.5px;fill:var(--bf-amber)">judged</text><polygon points="114.2,143.8 118.4,148.0 114.2,152.2 110.0,148.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.6"><title>8W ideal scan behind C (SC/C): 0.7516</title></polygon><line x1="186.2" y1="138.0" x2="186.2" y2="158.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><rect x="184.2" y="138.0" width="4.0" height="20.0" rx="0" style="fill:transparent"><title>8W K1 bar sqrt(SC/C): 0.8670</title></rect><line x1="174.4" y1="148.0" x2="191.7" y2="148.0" style="stroke:var(--bf-vz-2);stroke-width:2.4"/><line x1="174.4" y1="144.0" x2="174.4" y2="152.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><line x1="191.7" y1="144.0" x2="191.7" y2="152.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><circle cx="182.9" cy="148.0" r="4.2" style="fill:var(--bf-vz-2)"><title>8W PC/C: 0.8618 [0.8481, 0.8758] vs bar 0.8670 (capture +0.52)</title></circle><text class="bf-t-strong" x="197.7" y="146.0" text-anchor="start" style="font-size:8.8px;fill:var(--bf-amber)">cap 0.52</text><text class="bf-t-strong" x="197.7" y="157.0" text-anchor="start" style="font-size:8.4px;fill:var(--bf-amber)">UB +0.009 (K1)</text><text class="bf-t-strong" x="412.0" y="151.0" text-anchor="end" style="font-size:9.6px">8W</text><line x1="566.5" y1="148.0" x2="625.7" y2="148.0" style="stroke:var(--bf-vz-2);stroke-width:2.4"/><line x1="566.5" y1="144.0" x2="566.5" y2="152.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><line x1="625.7" y1="144.0" x2="625.7" y2="152.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><circle cx="595.7" cy="148.0" r="4.2" style="fill:var(--bf-vz-2)"><title>8W standalone P/none: 1.0519 [1.0233, 1.0812]</title></circle><text class="bf-t-strong" x="630.7" y="151.0" text-anchor="start" style="font-size:8.8px">1.052 (K2 ok)</text><rect x="10.0" y="169.0" width="348.0" height="34.0" rx="4" style="fill:var(--bf-amber-tint);fill-opacity:0.55"></rect><rect x="378.0" y="169.0" width="332.0" height="34.0" rx="4" style="fill:var(--bf-amber-tint);fill-opacity:0.55"></rect><text class="bf-t-strong" x="74.0" y="185.0" text-anchor="end" style="font-size:10.0px">9 workers</text><text class="bf-t-strong" x="74.0" y="197.0" text-anchor="end" style="font-size:8.5px;fill:var(--bf-amber)">judged</text><polygon points="110.6,181.8 114.8,186.0 110.6,190.2 106.4,186.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.6"><title>9W ideal scan behind C (SC/C): 0.7459</title></polygon><line x1="184.1" y1="176.0" x2="184.1" y2="196.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><rect x="182.1" y="176.0" width="4.0" height="20.0" rx="0" style="fill:transparent"><title>9W K1 bar sqrt(SC/C): 0.8636</title></rect><line x1="164.8" y1="186.0" x2="210.4" y2="186.0" style="stroke:var(--bf-vz-2);stroke-width:2.4"/><line x1="164.8" y1="182.0" x2="164.8" y2="190.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><line x1="210.4" y1="182.0" x2="210.4" y2="190.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><circle cx="187.1" cy="186.0" r="4.2" style="fill:var(--bf-vz-2)"><title>9W PC/C: 0.8685 [0.8328, 0.9058] vs bar 0.8636 (capture +0.48)</title></circle><text class="bf-t-strong" x="216.4" y="184.0" text-anchor="start" style="font-size:8.8px;fill:var(--bf-amber)">cap 0.48</text><text class="bf-t-strong" x="216.4" y="195.0" text-anchor="start" style="font-size:8.4px;fill:var(--bf-amber)">UB +0.042 (K1)</text><text class="bf-t-strong" x="412.0" y="189.0" text-anchor="end" style="font-size:9.6px">9W</text><line x1="525.5" y1="186.0" x2="560.0" y2="186.0" style="stroke:var(--bf-vz-2);stroke-width:2.4"/><line x1="525.5" y1="182.0" x2="525.5" y2="190.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><line x1="560.0" y1="182.0" x2="560.0" y2="190.0" style="stroke:var(--bf-vz-2);stroke-width:1.5"/><circle cx="542.6" cy="186.0" r="4.2" style="fill:var(--bf-vz-2)"><title>9W standalone P/none: 0.9999 [0.9832, 1.0170]</title></circle><text class="bf-t-strong" x="566.0" y="189.0" text-anchor="start" style="font-size:9.0px">1.000 (even)</text><text class="bf-tm" x="74.0" y="223.0" text-anchor="end" style="font-size:10.0px">10 workers</text><text class="bf-tm" x="74.0" y="235.0" text-anchor="end" style="font-size:8.5px">reported</text><polygon points="125.3,219.8 129.5,224.0 125.3,228.2 121.1,224.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.6"><title>10W ideal scan behind C (SC/C): 0.7694</title></polygon><line x1="192.5" y1="214.0" x2="192.5" y2="234.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><rect x="190.5" y="214.0" width="4.0" height="20.0" rx="0" style="fill:transparent"><title>10W K1 bar sqrt(SC/C): 0.8772</title></rect><line x1="159.9" y1="224.0" x2="189.6" y2="224.0" style="stroke:var(--bf-vz-1);stroke-width:2.4"/><line x1="159.9" y1="220.0" x2="159.9" y2="228.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><line x1="189.6" y1="220.0" x2="189.6" y2="228.0" style="stroke:var(--bf-vz-1);stroke-width:1.5"/><circle cx="174.6" cy="224.0" r="4.2" style="fill:var(--bf-vz-1)"><title>10W PC/C: 0.8484 [0.8249, 0.8725] vs bar 0.8772 (capture +0.63)</title></circle><text class="bf-tm" x="195.6" y="227.0" text-anchor="start" style="font-size:8.8px">cap +0.63</text><text class="bf-tm" x="412.0" y="227.0" text-anchor="end" style="font-size:9.6px">10W</text><line x1="449.2" y1="224.0" x2="505.6" y2="224.0" style="stroke:var(--bf-vz-3);stroke-width:2.4"/><line x1="449.2" y1="220.0" x2="449.2" y2="228.0" style="stroke:var(--bf-vz-3);stroke-width:1.5"/><line x1="505.6" y1="220.0" x2="505.6" y2="228.0" style="stroke:var(--bf-vz-3);stroke-width:1.5"/><circle cx="476.9" cy="224.0" r="4.2" style="fill:var(--bf-vz-3)"><title>10W standalone P/none: 0.9357 [0.9086, 0.9637]</title></circle><text class="bf-t-strong" x="511.6" y="227.0" text-anchor="start" style="font-size:9.0px;fill:var(--bf-green-ink)">0.936 (-6.4%)</text><polygon points="22.0,272.0 26.0,276.0 22.0,280.0 18.0,276.0" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.5"></polygon><text class="bf-tm" x="31.0" y="279.0" text-anchor="start" style="font-size:8.8px">SC/C (ideal scan)</text><line x1="128.0" y1="271.0" x2="128.0" y2="281.0" style="stroke:var(--bf-vz-2);stroke-width:2.0"/><text class="bf-tm" x="134.0" y="279.0" text-anchor="start" style="font-size:8.8px">K1 bar sqrt(SC/C)</text><circle cx="232.0" cy="276.0" r="3.8" style="fill:var(--bf-vz-1)"></circle><text class="bf-tm" x="240.0" y="279.0" text-anchor="start" style="font-size:8.8px">PC/C (95% CI)</text><rect x="14.0" y="290.0" width="692.0" height="26.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.0"></rect><text class="bf-t-strong" x="360.0" y="307.0" text-anchor="middle" style="font-size:9.1px">Passing gates:  K2 standalone LB &lt;= 1.03 (1.023 at 8W)  ·  K3 peak RSS 0.988-1.014 (&lt;= 1.10)  ·  K4 finder 4.82 ms / 9.84 GB/s (&lt;= 10 ms)  ·  0 fallbacks</text></svg>
<figcaption>Measured data (pooled-scan kill test, i9-10900K, wikidata ×4, 12 randomised blocks). Shaded rows mark the two pre-registered judged counts (8 and 9 workers); unshaded rows (6, 7, 10) are reported only. Left: PC/C ratios and 95% intervals against the ideal-scan floor SC/C (hollow diamonds) and K1 threshold sqrt(SC/C) (amber bars). Right: standalone P/none ratios against break-even (1.00) and K2's 1.03 lower-bound gate.</figcaption>
</figure>

## 5. Why pooling was not enough

At first glance, missing K1 by 0.009 at 8 workers looks like noise or a small implementation tax.
The per-thread traces show it is neither. Pooling worked exactly as engineered, with zero duplicated
work, tiny worker idleness and roughly 5 ms of chain overhead. The narrow miss follows straight from
a conservation law on CPU time.

In traced decodes, the workers' summed scanning CPU (`A2 + A3`) in `P` matched the dedicated
scanner's CPU in `none` at every worker count:

| Workers | Pooled workers' scan CPU (`P`) | Dedicated scanner CPU (`none`) | Worker idle share (`none → P`) |
|---:|---:|---:|---:|
| 6 | 172.9 [172.2, 173.6] ms | 171.3 [170.2, 172.3] ms | 1.4% → 0.9% |
| 7 | 174.1 [172.5, 175.7] ms | 170.6 [170.3, 171.0] ms | 1.9% → 1.2% |
| 8 | **177.7 [175.1, 180.3] ms** | **177.5 [148.2, 206.8] ms** | **6.5% → 1.2%** |
| 9 | 182.8 [171.9, 193.8] ms | 173.4 [160.6, 186.3] ms | 10.1% → 1.6% |
| 10 | 192.7 [181.9, 203.6] ms | 189.6 [184.1, 195.2] ms | **20.2% → 2.5%** |

Unlike [PR #770](https://github.com/lmmx/mibz2/pull/770), which nearly doubled scanning CPU, this
prototype wasted nothing. It simply moved roughly 178 ms of scanning CPU off the dedicated scanner
core and spread it across the $k$ worker cores.

That move turns the two-station bottleneck model from
[Section 9 of the companion post](2026-11-26-from-bottlenecks-to-better-algorithms.md#9-dissolving-the-serial-scanner)
into a sharp wall-time comparison. Let $s$ denote total scanning CPU per decode and $w$ denote total
parallel worker CPU. Let $\epsilon$ denote the small residual from marker finding, candidate copying
and tail imbalance:

$$
T_{\text{serial}}(k) \approx \max\!\left(s,\ \frac{w}{k}\right), \qquad T_{\text{pooled}}(k) \approx \frac{s + w}{k} + \epsilon.
$$

Every measurement in Section 4 drops straight out of those two equations.

**Below the knee ($w/k \ge s$), pooling must lose.** Workers already set the wall clock. Adding
$s/k$ of scanning to each worker lengthens the decode by 22 to 30 ms. Deleting the 190.6 MiB symbol
queue cannot offset that, because Section 1 proved that all inter-stage plumbing costs only ~5 ms.
Standalone `P` therefore slowed 6 and 7 workers by 10% and slowed 8 workers by 5.2%. At 8 workers,
adding $s/8$ onto the workers' 185 ms load gives about 207 ms, matching the measured 207.5 ms mean.

**Above the knee ($w/k < s$), pooling wins only when workers have slack to absorb $s/k$.** Setting
$(s + w)/k < s$ yields the exact zero-overhead break-even point:

$$
k > 1 + \frac{w}{s}.
$$

With mibz2's measured knee $w/s \approx 7.67$, even zero-overhead pooling cannot break even until
$k \approx 8.7$ workers. Adding ~10 ms of marker finding and candidate copying moves break-even
squarely to $k = 9$, where `P/none` measured `1.000`. At $k = 10$, shipped workers sat 20.2% idle
waiting on the serial scanner. Absorbing $192.7 / 10 \approx 19.3\text{ ms}$ of scanning per worker
fit inside that idle slack. Worker idleness fell to 2.5%, and median wall time dropped by 6.4% from
197.6 ms to 187.2 ms.

**Behind ideal construction, the same $s/k$ term explains the K1 miss to the millisecond.** Making
construction ideal (`C`) shrinks $w$ enough that `PC` beats `C` by 14% at 8 workers. Even so, an
ideal scan (`SC`) deletes scanning CPU completely, whereas a pooled scan still pays $s/k$. At 8
workers, adding $177.7 / 8 = 22.2\text{ ms}$ to `SC`'s 135.3 ms predicts 157.5 ms against a measured
`PC` of 158.1 ms. At 9 workers, adding $182.8 / 9 = 20.3\text{ ms}$ to `SC`'s 140.2 ms predicts
160.5 ms against `PC` at 159.9 ms. That $s/k$ tax consumes almost half of the 48 ms gap between `C`
and `SC`, capping full pooling near 50% capture at eight and nine workers. K1's narrow miss was
structural, not unlucky.

<figure class="bf-fig" id="redistribution">
<p class="bf-title">Moving work vs. stealing slack: dedicated scanner, full pool, and slack-only hybrid</p>
<!-- fig:redistribution -->
<svg viewBox="0 0 720 376" role="img" aria-label="Capacity breakdown comparing dedicated scanning, full worker pooling, and a proposed slack-only hybrid at 8 and 10 workers. At 8 workers, shipped workers sit only 6.5 percent idle (192.9 ms wall), so moving 177.7 ms of scan CPU onto 8 workers adds 22.2 ms per worker and slows standalone decoding to 207.7 ms (+5.2 percent); behind ideal construction, the same 22.8 ms separates PC (158.1 ms) from SC (135.3 ms). At 10 workers, shipped workers sit 20.2 percent idle (197.6 ms wall), so absorbing 19.3 ms per worker of scan work into slack speeds decoding to 187.2 ms (-6.4 percent). A proposed slack-only hybrid keeps the serial scanner and lets idle workers scan ahead only when waiting."><text class="bf-t-strong" x="14.0" y="20.0" text-anchor="start" style="font-size:12px">Moving work vs. stealing slack: why pooling loses at 8 workers and wins at 10</text><text class="bf-tm" x="14.0" y="35.0" text-anchor="start" style="font-size:9.8px">Per-decode wall-time budgets (ms) from pooled-scan medians and traces; workers spend the same 173-193 ms scanning</text><rect x="10.0" y="48.0" width="700.0" height="148.0" rx="7" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.1"></rect><text class="bf-t-strong" x="22.0" y="66.0" text-anchor="start" style="font-size:10.5px">At 8 workers (balanced knee: dedicated scanner 177.5 ms, workers ~185 ms, only 6.5% idle)</text><line x1="152.0" y1="74.0" x2="152.0" y2="184.0" style="stroke:var(--bf-vz-axis);stroke-width:0.9"/><line x1="255.0" y1="74.0" x2="255.0" y2="184.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><line x1="358.1" y1="74.0" x2="358.1" y2="184.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><line x1="461.1" y1="74.0" x2="461.1" y2="184.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><line x1="564.2" y1="74.0" x2="564.2" y2="184.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><text class="bf-t-strong" x="144.0" y="89.0" text-anchor="end" style="font-size:9.6px">Shipped (none)</text><text class="bf-tm" x="144.0" y="100.0" text-anchor="end" style="font-size:8.5px">1 scanner + 8 workers</text><path d="M152.0,80.0 L519.8,80.0 A4.0,4.0 0 0 1 523.8,84.0 L523.8,94.0 A4.0,4.0 0 0 1 519.8,98.0 L152.0,98.0 Z" style="fill:var(--bf-vz-1)"><title>Shipped 8W workers busy: ~180 ms; worker idle share 6.5%</title></path><rect x="523.8" y="80.0" width="25.8" height="18.0" rx="0" style="fill:var(--bf-track);stroke:var(--bf-vz-axis);stroke-width:1.0;stroke-dasharray:3 2"><title>6.5% worker idle share waiting on serial scanner (none, 8W)</title></rect><text class="bf-t-strong" x="160.0" y="93.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-vz-1-ink)">workers: M · C · E · R (in parallel with 177.5 ms serial scan)</text><text class="bf-t-strong" x="555.5" y="93.0" text-anchor="start" style="font-size:9.6px">192.9 ms</text><text class="bf-t-strong" x="144.0" y="123.0" text-anchor="end" style="font-size:9.6px">Full pool (P)</text><text class="bf-tm" x="144.0" y="134.0" text-anchor="end" style="font-size:8.5px">8 workers absorb s/8</text><path d="M152.0,114.0 L519.0,114.0 A4.0,4.0 0 0 1 523.0,118.0 L523.0,128.0 A4.0,4.0 0 0 1 519.0,132.0 L152.0,132.0 Z" style="fill:var(--bf-vz-1)"><title>Worker M, C, E, R share at 8 workers</title></path><rect x="523.0" y="114.0" width="45.8" height="18.0" rx="0" style="fill:var(--bf-vz-2)"><title>Scan CPU moved onto 8 workers: 177.7 / 8 = 22.2 ms per worker</title></rect><path d="M568.7,114.0 L576.0,114.0 A4.0,4.0 0 0 1 580.0,118.0 L580.0,128.0 A4.0,4.0 0 0 1 576.0,132.0 L568.7,132.0 Z" style="fill:var(--bf-vz-4)"><title>Finder/copy/tail overhead (~5-6 ms)</title></path><text class="bf-t-strong" x="160.0" y="127.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-vz-1-ink)">workers: M · C · E · R</text><text class="bf-t-strong" x="545.8" y="127.0" text-anchor="middle" style="font-size:8.8px;fill:var(--bf-vz-2-ink)">+s/8</text><text class="bf-t-strong" x="586.0" y="127.0" text-anchor="start" style="font-size:9.5px;fill:var(--bf-amber)">207.7 ms (+5.2%)</text><text class="bf-t-strong" x="144.0" y="157.0" text-anchor="end" style="font-size:9.6px">Behind ideal C</text><text class="bf-tm" x="144.0" y="168.0" text-anchor="end" style="font-size:8.5px">SC vs PC (+s/8 gap)</text><path d="M152.0,148.0 L426.8,148.0 A4.0,4.0 0 0 1 430.8,152.0 L430.8,162.0 A4.0,4.0 0 0 1 426.8,166.0 L152.0,166.0 Z" style="fill:var(--bf-vz-3)"><title>SC (ideal scan + ideal C) at 8 workers: median 135.3 ms</title></path><path d="M430.8,148.0 L473.8,148.0 A4.0,4.0 0 0 1 477.8,152.0 L477.8,162.0 A4.0,4.0 0 0 1 473.8,166.0 L430.8,166.0 Z" style="fill:var(--bf-vz-2)"><title>PC minus SC at 8 workers: 158.1 - 135.3 = 22.8 ms (matches s/8 = 177.7/8 = 22.2 ms)</title></path><text class="bf-t-strong" x="160.0" y="161.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-vz-3-ink)">SC ideal: 135.3 ms</text><text class="bf-t-strong" x="454.3" y="161.0" text-anchor="middle" style="font-size:8.6px;fill:var(--bf-vz-2-ink)">+22.8</text><text class="bf-t-strong" x="483.8" y="161.0" text-anchor="start" style="font-size:9.4px">PC 158.1 ms (52% capture)</text><rect x="10.0" y="206.0" width="700.0" height="148.0" rx="7" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.1"></rect><text class="bf-t-strong" x="22.0" y="224.0" text-anchor="start" style="font-size:10.5px">At 10 workers (20.2% worker slack) — and the slack-only hybrid for all counts</text><line x1="152.0" y1="232.0" x2="152.0" y2="342.0" style="stroke:var(--bf-vz-axis);stroke-width:0.9"/><text class="bf-tm" x="152.0" y="366.0" text-anchor="middle" style="font-size:9.4px">0 ms</text><line x1="255.0" y1="232.0" x2="255.0" y2="342.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><text class="bf-tm" x="255.0" y="366.0" text-anchor="middle" style="font-size:9.4px">50 ms</text><line x1="358.1" y1="232.0" x2="358.1" y2="342.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><text class="bf-tm" x="358.1" y="366.0" text-anchor="middle" style="font-size:9.4px">100 ms</text><line x1="461.1" y1="232.0" x2="461.1" y2="342.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><text class="bf-tm" x="461.1" y="366.0" text-anchor="middle" style="font-size:9.4px">150 ms</text><line x1="564.2" y1="232.0" x2="564.2" y2="342.0" style="stroke:var(--bf-vz-grid);stroke-width:0.9"/><text class="bf-tm" x="564.2" y="366.0" text-anchor="middle" style="font-size:9.4px">200 ms</text><text class="bf-t-strong" x="144.0" y="247.0" text-anchor="end" style="font-size:9.6px">Shipped (none)</text><text class="bf-tm" x="144.0" y="258.0" text-anchor="end" style="font-size:8.5px">10W sit 20.2% idle</text><path d="M152.0,238.0 L473.0,238.0 A4.0,4.0 0 0 1 477.0,242.0 L477.0,252.0 A4.0,4.0 0 0 1 473.0,256.0 L152.0,256.0 Z" style="fill:var(--bf-vz-1)"><title>Shipped 10W workers busy (~79.8% of 197.6 ms wall)</title></path><rect x="477.0" y="238.0" width="82.2" height="18.0" rx="0" style="fill:var(--bf-track);stroke:var(--bf-vz-axis);stroke-width:1.1;stroke-dasharray:3 2"><title>20.2% worker idle share waiting on 189.6 ms serial scanner (none, 10W)</title></rect><text class="bf-t-strong" x="160.0" y="251.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-vz-1-ink)">10 workers busy (79.8%)</text><text class="bf-tm" x="518.0" y="251.0" text-anchor="middle" style="font-size:8.6px">20.2% idle</text><text class="bf-t-strong" x="565.2" y="251.0" text-anchor="start" style="font-size:9.6px">197.6 ms</text><text class="bf-t-strong" x="144.0" y="281.0" text-anchor="end" style="font-size:9.6px">Full pool (P)</text><text class="bf-tm" x="144.0" y="292.0" text-anchor="end" style="font-size:8.5px">scan fits inside slack</text><path d="M152.0,272.0 L481.9,272.0 A4.0,4.0 0 0 1 485.9,276.0 L485.9,286.0 A4.0,4.0 0 0 1 481.9,290.0 L152.0,290.0 Z" style="fill:var(--bf-vz-1)"><title>Worker M, C, E, R share at 10 workers</title></path><path d="M485.9,272.0 L521.6,272.0 A4.0,4.0 0 0 1 525.6,276.0 L525.6,286.0 A4.0,4.0 0 0 1 521.6,290.0 L485.9,290.0 Z" style="fill:var(--bf-vz-2)"><title>Scan CPU moved onto 10 workers: 192.7 / 10 = 19.3 ms per worker</title></path><path d="M525.6,272.0 L533.8,272.0 A4.0,4.0 0 0 1 537.8,276.0 L537.8,286.0 A4.0,4.0 0 0 1 533.8,290.0 L525.6,290.0 Z" style="fill:var(--bf-vz-4)"><title>Finder/copy/idle residual (2.5% idle)</title></path><text class="bf-t-strong" x="160.0" y="285.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-vz-1-ink)">10 workers: M · C · E · R</text><text class="bf-t-strong" x="505.6" y="285.0" text-anchor="middle" style="font-size:8.4px;fill:var(--bf-vz-2-ink)">+s/10</text><text class="bf-t-strong" x="543.8" y="285.0" text-anchor="start" style="font-size:9.5px;fill:var(--bf-green-ink)">187.2 ms (-6.4%)</text><text class="bf-t-strong" x="144.0" y="315.0" text-anchor="end" style="font-size:9.6px">Hybrid (proposed)</text><text class="bf-tm" x="144.0" y="326.0" text-anchor="end" style="font-size:8.5px;font-style:italic">design, not measured</text><rect x="152.0" y="306.0" width="346.2" height="18.0" rx="4" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;stroke-dasharray:4 2"><title>Proposed slack-only hybrid: keep dedicated scanner; workers scan ahead only from idle job_wait slack</title></rect><text class="bf-t-strong" x="160.0" y="319.0" text-anchor="start" style="font-size:9.1px;fill:var(--bf-green-ink)">keep serial scanner; idle workers scan ahead only when waiting</text><text class="bf-t-strong" x="504.2" y="319.0" text-anchor="start" style="font-size:9.2px;fill:var(--bf-green-ink)">no &lt;=8W tax</text></svg>
<figcaption>Measured budgets at 8 and 10 workers (pooled-scan medians and traces) beside the proposed slack-only hybrid (design, not measured). Visual analogy: a kitchen with one prep cook (S) and k line cooks (W). Dismissing the prep cook and splitting prep across the line cooks cuts waiting between courses (idle time falls from 6.5–20.2% to 1.2–2.5%), yet slows service whenever the line cooks are already flat out (k ≤ 8); service speeds up only when line cooks are waiting for prep (k = 10), or when only idle cooks step in to help (the hybrid). Where the analogy stops: CPU cores share an L3 cache, pay ~5 ms each for boundary finding and candidate copying, and still commit finished blocks strictly in order.</figcaption>
</figure>

## 6. Keeping the verdict and what comes next

Missing a pre-registered threshold by 0.009 at 8 workers can make a gate look overly strict. Here,
honouring the frozen verdict is both good discipline and the right engineering call.

<aside class="il-gloss" markdown>
**Pre-registered kill test.** Freezing the decision rule before running the benchmark stops a team
from moving the goalposts after seeing data. When a prototype misses a frozen gate narrowly, the
right move is to diagnose why it missed and register a better design, not to re-grade the old run.
</aside>

Section 5 showed that the 0.009 miss is not random noise. Dissolving the dedicated scanner forces
$k$ workers to absorb $s$ on top of $w$, which taxes the shipped 8-worker path whenever workers lack
slack. The same rule governs any streaming pipeline with a dedicated serial parser costing $s$ and a
worker pool costing $w$. Removing the parser breaks even at $k^* = 1 + w/s$ before overhead. It
loses whenever $k < 1 + w/s$ and wins only when $k > 1 + w/s$. To win on both sides of the knee, a
pipeline has two clean options. It can shrink $w$ until $1 + w/s$ drops below the target worker
count. Alternatively, it can keep the dedicated parser alive and let workers help only from idle
slack.

Together, the zero-copy floor run and the pooled-scan kill test close three blind alleys and lay out
a clear order for our next work:

- **Closed targets.** Pipeline plumbing (~5 ms, 2.7% of wall) needs no attention. The three-station
  memory-bandwidth hypothesis is not supported at 8–10 workers. Unconditional full pooling stops as
  built under its frozen K1 verdict.
- **Shrink worker demand $w$ first.** Lowering worker CPU speeds up 1–7 workers at once. It also
  pulls the break-even count $1 + w/s$ below 8 workers so scanning relief pays off where mibz2
  ships. Three worker-side targets are open: emission's 298 ms word spelling, folding move-to-front
  into construction's first pass, and forward wiring with multi-lane emission.
- **Test a slack-only hybrid scanner.** Keep the dedicated scanner thread so saturated workers at
  six to eight cores never pay a scanning tax. Let workers claim look-ahead candidates with the
  proven 4.82 ms AVX2 marker finder *only* when they would otherwise wait in `job_wait`. Scanning
  CPU is then spent strictly from slack. Full pooling can also be re-tested under a fresh
  registration once worker-side cuts lower $w$.
- **Prepare for delivery's 48 ms serial floor.** Once scanning leaves its single thread and worker
  cuts push compute near 100 ms, serial delivery takes over. Copying 839 MB on one thread at 17 GB/s
  then becomes nearly half of the remaining decode. Solving that is an API and ownership change
  rather than a kernel tweak: decoding into caller-supplied slices, parallelising copy-out, or
  lending completed buffers zero-copy.

---

**Sources:**

- Zero-copy floor ([PR #917](https://github.com/lmmx/mibz2/pull/917), [PR #921](https://github.com/lmmx/mibz2/pull/921)): [floor results](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md), [report](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/report.txt) and [raw table](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/zero-copy-floor.tsv).
- Pooled-scan kill test ([PR #932](https://github.com/lmmx/mibz2/pull/932), [PR #942](https://github.com/lmmx/mibz2/pull/942)): [registration](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md), [results](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md) and [report](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/results/report.txt).
- Earlier parallel scans: [PR #741](https://github.com/lmmx/mibz2/pull/741) and [PR #770](https://github.com/lmmx/mibz2/pull/770), summarised in [the registration](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md).
- Preceding posts: [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) and [From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md).
- Earlier decoder posts: [The decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md), [Masked bottlenecks](2026-11-24-the-scanner-hides-the-emitter.md), [Open Returns](2026-11-19-open-returns.md) and [Huffman decoding](2026-11-05-huffman-bit-buffer.md).
- Reproducible figures and measured surface: `docs/scripts/figures_floor_and_pooled_scan.py`, [surface.tsv](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/frontier/surface.tsv) and [dead-ends.tsv](https://github.com/lmmx/mibz2/blob/daf022450e56b08b5dc1ed400e09baf94981b559/frontier/dead-ends.tsv).
