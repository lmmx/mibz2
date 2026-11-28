---
title: "How shrinking one thread's scratch memory sped up a different thread"
image: assets/social/blog-2026-11-28-shrinking-worker-scratch-speeds-the-scanner.png
description: "Shrinking each worker's slab scratch removes almost every extra cache stall on the serial scanner, making the whole eight-worker decode faster."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# How shrinking one thread's scratch memory sped up a different thread

Shrinking each worker's slab scratch removes almost every extra cache stall on the serial scanner, making the whole eight-worker decode faster.
{: .post-dek }

28 November 2026 · 6 min read · [Research result](#5-a-sharper-rule-for-multi-core-optimization)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Shrinking a thread's private scratch buffer usually saves RAM and keeps that same thread warm in
cache. You rarely expect it to speed up a completely different thread on another physical core
without changing a single line of that thread's code.

Yet that is what just happened on our ten-core Intel Core i9-10900K. Behind an experimental
feature flag, our four-lane forward Burrows–Wheeler walk already sped up individual workers by 17%
([walking forward across four lanes](2026-11-28-four-lanes-forward.md)). Once the workers outran
the serial Huffman scanner at eight threads, hardware counters caught a surprise. Under the
experimental walk, the scanner took 6.5% more cycles at eight workers than at four. Under the
default decoder, it rose only 2.3%.

We replaced each worker's recycled slab pool with a strictly bounded 80 KiB arena in safe Rust. It
writes 4 KiB chunks into the output buffer and permutes them in place. Rerunning the exact same
hardware counter test erased 99.1% of the scanner's excess cycles. Eight-worker peak memory dropped
below baseline, and pinned-scanner throughput jumped from 4.69 GB/s to 4.96 GB/s.

**Takeaways**

- **Worker scratch memory was stalling a separate core.** Behind an experimental feature flag, our
  first four-lane walk kept about a megabyte of recycled 32 KiB slabs per worker. Those extra
  6.53 MiB at eight workers overflowed the shared 20 MiB L3 cache. They more than doubled L3-miss
  stalls inside the serial scanner on core zero.
- **An 80 KiB bounded arena uses 4 KiB slabs and in-place reordering.** Each active lane owns one
  4 KiB slab, or 16 KiB in total, fitting inside half of L1 data cache. Full chunks flush straight
  into the output buffer and permute in place with zero extra allocation. A bounded 64 KiB buffer
  holds only short segment tails.
- **Bounding the arena removed 99.1% of the scanner excess.** At flat clock from four to eight
  workers, the experimental scanner's cycle rise fell from 6.47% to 2.35%. That matches baseline's
  2.31% within four hundredths of a percent.
- **Pinned eight-worker throughput reached 4.96 GB/s in less RAM than baseline.** Clearing 31.5
  million extra stall cycles cut Wikidata decode time from 0.1787 s to 0.1691 s. That beats
  baseline's 0.1910 s by 11.1% while using 1.32 MiB less peak memory.
- **Every megabyte past the L3 cliff taxes the whole pipeline.** Near the shared cache ceiling,
  each extra megabyte of active eight-worker scratch cost the serial scanner about 4.9 million
  cycles.

## 1. The mystery on core zero

Parallel bzip2 decompression in mibz2 splits into two coupled stations. One serial scanner thread
parses the bitstream and decodes Huffman symbols. A pool of worker threads then inverts each block's
Burrows–Wheeler transform, undoes run-length encoding and checks block checksums.

When we added an experimental four-lane forward walk to the workers, eight-worker Wikidata decoding
flipped from worker-bound to scanner-bound. Per-worker busy time dropped cleanly, yet whole-file
wall time did not fall as far as an ideal-scanner oracle predicted. Even more puzzling, the serial
scanner looked 15% slower at eight workers than at one worker, even though its code had not changed.

First we separated clock scaling from true contention. Our ten-core chip drops its turbo clock from
5.28 GHz at one active core to 4.91 GHz once four cores run. That single step explains roughly 7.5
percentage points of the apparent slowdown between one worker and four. From four workers to eight,
however, achieved frequency stays flat at 1.326 cycles per reference cycle.

To isolate what happened at constant clock, we pinned the serial scanner exclusively to core zero
and confined the workers to the other nine cores. We attached Linux hardware counters to core zero
across twelve interleaved rounds on 839 MB of Wikidata. Every round compared the default build
against the experimental walk at one, four and eight workers.

The counters ruled out extra work immediately. Every arm and thread count retired the exact same
1,427.0 million instructions. Four-worker cycles also matched one-worker cycles within 0.03%.
Between four workers and eight at constant clock, however, the two builds diverged sharply:

- Under the **default decoder**, scanner cycles rose by 2.30%, adding 17.4 million cycles. Of that
  increase, 88.4% (15.4 million cycles) came from L3-miss stalls.
- Under the **experimental unbounded-slab walk**, scanner cycles jumped by 6.47%, adding 49.3
  million cycles. Scanner IPC fell from 1.872 to 1.759. L3-miss stalls accounted for 76.4% of the
  rise, or 37.6 million cycles — 2.44 times the baseline increment.

Something the workers were touching on cores one through nine was evicting the scanner's live state
from the shared last-level cache on core zero.

## 2. Why cold slabs still crowd a shared 20 MiB L3

Looking at resident memory pointed straight to the suspect. The first version of the experimental
four-lane walk wrote each lane into recycled 32 KiB slabs before stitching them into the output
buffer. A full bzip2 block holds up to 900 KiB before final run-length decoding. Each worker
therefore built a pool of about thirty 32 KiB slabs — nearly a full megabyte of extra scratch held
alongside the output buffer.

At eight workers, that recycled pool added 6.53 MiB of peak resident memory over baseline. Earlier
we assumed recycled slabs were mostly harmless because a worker only writes a handful of them at any
instant ([walking forward across four lanes](2026-11-28-four-lanes-forward.md)). On a processor with
a 20 MiB shared L3 cache, that assumption broke down.

Eight workers cycling through a megabyte of slab memory apiece touch roughly 8 MiB of extra dirty
cache lines across every wave of blocks. Workers also hold inversion tables, input symbol buffers
and queued output blocks. Together, those extra megabytes push active traffic past the 20 MiB L3
ceiling. Cache replacement then evicts core zero's Huffman tables and input streams, forcing the
scanner to stall on DRAM.

<figure class="bf-fig" id="scanner-l3-collapse">
<p class="bf-title">Core-zero scanner cycle rise and L3-miss stalls from four to eight workers at constant clock</p>
<!-- fig:scanner-l3-collapse -->
<svg viewBox="0 0 720 314" role="img" aria-label="Two-panel bar chart comparing core-zero scanner contention from four to eight workers on an Intel Core i9-10900K decoding 839 MB of Wikidata. Left panel shows scanner cycle rise from four to eight workers: default baseline +2.31 percent, experimental unbounded slabs +6.47 percent, and experimental 80 KiB bounded slabs +2.35 percent (99.1 percent of excess eliminated). Right panel shows the 4W to 8W L3-miss-stall increment in millions of cycles: default baseline +15.2M, experimental unbounded slabs +37.6M (2.44x baseline), and experimental 80 KiB bounded slabs +17.7M (1.16x baseline)."><rect x="14" y="14" width="338" height="264" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="28" y="38" style="font-size:11.5px">Scanner cycle rise (4W → 8W, flat clock)</text><line x1="126" y1="56" x2="126" y2="236" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><line x1="176" y1="56" x2="176" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="226" y1="56" x2="226" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="276" y1="56" x2="276" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="126" y="254" text-anchor="middle" style="font-size:10px">0%</text><text class="bf-tm" x="176" y="254" text-anchor="middle" style="font-size:10px">+2.5%</text><text class="bf-tm" x="226" y="254" text-anchor="middle" style="font-size:10px">+5.0%</text><text class="bf-tm" x="276" y="254" text-anchor="middle" style="font-size:10px">+7.5%</text><text class="bf-tm" x="118" y="92" text-anchor="end" style="font-size:10.5px">Default base</text><rect x="126" y="76" width="46.2" height="24" rx="3" style="fill:var(--bf-vz-2)"><title>Default baseline 8W/4W scanner cycles: +2.31% (+17.5M cycles)</title></rect><text class="bf-t-strong" x="179" y="92" style="font-size:10.5px">+2.31%</text><text class="bf-tm" x="118" y="150" text-anchor="end" style="font-size:10.5px">Unbounded slabs</text><text class="bf-tm" x="118" y="164" text-anchor="end" style="font-size:9.5px">(~1 MiB/worker)</text><rect x="126" y="136" width="129.4" height="24" rx="3" style="fill:var(--bf-vz-4)"><title>Experimental unbounded 32 KiB slab pool 8W/4W scanner cycles: +6.47% (+49.3M cycles)</title></rect><text class="bf-t-strong" x="262" y="152" style="font-size:10.5px">+6.47%</text><text class="bf-tm" x="118" y="208" text-anchor="end" style="font-size:10.5px">Bounded 80 KiB</text><text class="bf-tm" x="118" y="222" text-anchor="end" style="font-size:9.5px">(in-place 4 KiB)</text><rect x="126" y="194" width="47.0" height="24" rx="3" style="fill:var(--bf-vz-1)"><title>Experimental bounded 80 KiB arena 8W/4W scanner cycles: +2.35% (+17.8M cycles; 99.1% excess removed)</title></rect><text class="bf-t-strong" x="180" y="210" style="font-size:10.5px">+2.35% (−99.1% excess)</text><rect x="368" y="14" width="338" height="264" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="382" y="38" style="font-size:11.5px">Scanner L3-miss stalls added (4W → 8W)</text><line x1="480" y1="56" x2="480" y2="236" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><line x1="535" y1="56" x2="535" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="590" y1="56" x2="590" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="645" y1="56" x2="645" y2="236" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="480" y="254" text-anchor="middle" style="font-size:10px">0M</text><text class="bf-tm" x="535" y="254" text-anchor="middle" style="font-size:10px">+15M</text><text class="bf-tm" x="590" y="254" text-anchor="middle" style="font-size:10px">+30M</text><text class="bf-tm" x="645" y="254" text-anchor="middle" style="font-size:10px">+45M</text><text class="bf-tm" x="472" y="92" text-anchor="end" style="font-size:10.5px">Default base</text><rect x="480" y="76" width="55.8" height="24" rx="3" style="fill:var(--bf-vz-2)"><title>Default baseline 4W to 8W L3-miss-stall increment: +15.2M cycles</title></rect><text class="bf-t-strong" x="542" y="92" style="font-size:10.5px">+15.2M (1.00×)</text><text class="bf-tm" x="472" y="150" text-anchor="end" style="font-size:10.5px">Unbounded slabs</text><text class="bf-tm" x="472" y="164" text-anchor="end" style="font-size:9.5px">(+6.53 MiB 8W)</text><rect x="480" y="136" width="138.0" height="24" rx="3" style="fill:var(--bf-vz-4)"><title>Unbounded slab pool 4W to 8W L3-miss-stall increment: +37.6M cycles (2.44x baseline)</title></rect><text class="bf-t-strong" x="624" y="152" style="font-size:10.5px">+37.6M (2.44×)</text><text class="bf-tm" x="472" y="208" text-anchor="end" style="font-size:10.5px">Bounded 80 KiB</text><text class="bf-tm" x="472" y="222" text-anchor="end" style="font-size:9.5px">(−1.32 MiB 8W)</text><rect x="480" y="194" width="64.8" height="24" rx="3" style="fill:var(--bf-vz-1)"><title>Bounded 80 KiB arena 4W to 8W L3-miss-stall increment: +17.7M cycles (1.16x baseline)</title></rect><text class="bf-t-strong" x="551" y="210" style="font-size:10.5px">+17.7M (1.16×)</text><text class="bf-tm" x="706" y="302" text-anchor="end" style="font-size:10.5px">Intel Core i9-10900K, core-0 pinned scanner, 12 interleaved rounds on 839 MB Wikidata.</text></svg>
<figcaption>Left: shrinking the experimental worker arena to 80 KiB cuts the scanner's 4W-to-8W
cycle jump from +6.47% back to +2.35%, matching baseline's +2.31%. Right: core-zero L3-miss stalls
drop from 2.44× baseline down to 1.16×, confirming shared-cache eviction as the cause.</figcaption>
</figure>

Notice the falsifiable prediction that hypothesis makes. Dividing 31.8 million extra scanner cycles
by 6.53 MiB of extra eight-worker memory yields about 4.9 million cycles per megabyte. If worker
scratch memory caused those stalls, shrinking the slab pool under 100 KiB should erase the entire
4.17-point gap. The experimental scanner's ratio should fall straight back into the 1.023 to 1.030
band.

## 3. Bounding the arena to 80 KiB with in-place permutation

How do you keep four independent forward lanes writing out of order without storing a megabyte of
temporary slabs? The answer is to use the output buffer that the caller has already allocated.

Inside the safe-Rust experimental forward walk, we replaced the growable slab pool with a strictly
bounded per-worker arena of at most 80 KiB:

1. **Four L1-resident lane slabs (16 KiB total).** Each of the four active lanes owns one fixed
   4 KiB buffer. Together they fit inside half of a 32 KiB L1 data cache, keeping every lane's write
   cursor hot without spilling into L2 or L3.
2. **Direct chunk flushing into the output buffer.** Whenever a lane fills its 4 KiB slab, it copies
   that chunk straight to the end of the output buffer. It then records one byte naming the owning
   segment and reuses the same 4 KiB slab.
3. **A bounded tail buffer (at most 64 KiB).** When a lane hits the next anchor, it copies any
   remaining partial bytes into a small contiguous tail buffer. Because a block has at most sixteen
   segments and each tail is strictly smaller than 4 KiB, the tail buffer can never exceed 64 KiB.
4. **In-place cycle-following permutation.** Once all sixteen segments finish, the output buffer
   holds every full 4 KiB chunk, interleaved in completion order rather than cycle order. Counting
   chunks per segment gives each chunk its exact target slot. A cycle-following loop then permutes
   those chunks in place inside the output buffer. It reuses one 4 KiB lane slab as its temporary
   slot, with zero extra allocation and zero unsafe code.
5. **One backward pass to insert the tails.** Finally, a single right-to-left shift slides each
   segment's contiguous chunks into their exact final byte offsets and copies each segment's short
   tail right behind them.

Because slabs never pile up across a block, retained scratch drops from roughly 1 MiB per worker to
at most 80 KiB. That is more than a tenfold reduction.

## 4. The causal test on physical silicon

Before touching the workstation, we froze three strict verdicts for the excess scanner rise over
baseline. **EXPLAINED** required an excess at or below 1.0 percentage point and L3-stall growth
within 1.5× of baseline. **PARTIAL** covered 1.0 to 3.0 points, and **NOT EXPLAINED** began at 3.0
points. Then we reran the exact same pinned-scanner hardware-counter benchmark unchanged on our
Intel Core i9-10900K.

The verdict came back **EXPLAINED** with room to spare.

<div class="bf-data" markdown>

| Metric | Base | Pool | **80 KiB** |
|---|---:|---:|---:|
| 8W/4W | 1.0231× | 1.0647× | **1.0235×** |
| Excess | control | +4.17 pp | **+0.04 pp** |
| L3 stalls | +15.2M | +37.6M | **+17.7M** |
| IPC | 1.841 | 1.759 | **1.840** |
| RSS (MiB) | 104.6 | 111.1 | **103.3** |
| Wall (s) | 0.1910 | 0.1787 | **0.1691** |
| GB/s | 4.39 | 4.69 | **4.96** |

</div>

Three numbers stand out side by side:

- **The control reproduced to four decimal places.** Baseline's 8W/4W scanner cycle ratio came in
  at 1.0231×, matching the earlier run's 1.0230× almost identically. Its 95% confidence interval ran
  from 1.0199× to 1.0263×.
- **99.1% of the scanner contention vanished.** The bounded experimental walk's 8W/4W scanner ratio
  dropped from 1.0647× to 1.0235×, with a 95% confidence interval of 1.0202× to 1.0268×. Excess over
  baseline fell from 4.17 percentage points to 0.04 points. Paired directly at eight workers, the
  bounded scanner ran at 0.9998× of baseline's cycles. Both arms finished at nearly identical IPC
  (1.840 vs 1.841) and LLC load misses (1.002×).
- **Peak memory fell below baseline, and eight-worker speed jumped 5.4%.** Peak memory at eight
  workers dropped by 7.85 MiB compared with the unbounded pool. It finished at 103.3 MiB, which is
  1.32 MiB *below* baseline's 104.6 MiB. Clearing 31.5 million extra stall cycles cut pinned
  eight-worker wall time from 0.1787 s to 0.1691 s. That lifted throughput from 4.69 GB/s to
  4.96 GB/s, cutting wall time 11.1% below baseline (12.9% higher throughput).

<figure class="bf-fig" id="bounded-throughput-rss">
<p class="bf-title">Eight-worker Wikidata throughput and peak memory: baseline vs unbounded and bounded lanes</p>
<!-- fig:bounded-throughput-rss -->
<svg viewBox="0 0 720 306" role="img" aria-label="Two-panel chart comparing eight-worker Wikidata decoding on an Intel Core i9-10900K with a core-0 pinned scanner. Left panel shows real throughput in GB/s: default baseline 4.39 GB/s, experimental unbounded slabs 4.69 GB/s (+6.8 percent), and experimental 80 KiB bounded slabs 4.96 GB/s (+12.9 percent throughput, -11.1 percent wall time). Right panel shows eight-worker peak resident memory relative to baseline: default baseline 0.0 MiB (104.6 MiB), experimental unbounded slabs +6.53 MiB, and experimental 80 KiB bounded slabs -1.32 MiB (103.3 MiB)."><rect x="14" y="14" width="338" height="256" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="28" y="38" style="font-size:11.5px">8-worker throughput (GB/s, higher is better)</text><line x1="126" y1="56" x2="126" y2="228" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><line x1="196" y1="56" x2="196" y2="228" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="266" y1="56" x2="266" y2="228" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="126" y="246" text-anchor="middle" style="font-size:10px">3.5</text><text class="bf-tm" x="196" y="246" text-anchor="middle" style="font-size:10px">4.5</text><text class="bf-tm" x="266" y="246" text-anchor="middle" style="font-size:10px">5.5 GB/s</text><text class="bf-tm" x="118" y="92" text-anchor="end" style="font-size:10.5px">Default base</text><rect x="126" y="76" width="62.4" height="24" rx="3" style="fill:var(--bf-vz-2)"><title>Default baseline 8W pinned-scanner throughput: 4.392 GB/s (0.1910 s)</title></rect><text class="bf-t-strong" x="195" y="92" style="font-size:10.5px">4.39 GB/s</text><text class="bf-tm" x="118" y="148" text-anchor="end" style="font-size:10.5px">Unbounded slabs</text><rect x="126" y="132" width="83.6" height="24" rx="3" style="fill:var(--bf-vz-4)"><title>Experimental unbounded slabs 8W throughput: 4.69 GB/s (0.1787 s)</title></rect><text class="bf-t-strong" x="216" y="148" style="font-size:10.5px">4.69 GB/s</text><text class="bf-tm" x="118" y="204" text-anchor="end" style="font-size:10.5px">Bounded 80 KiB</text><rect x="126" y="188" width="102.3" height="24" rx="3" style="fill:var(--bf-vz-1)"><title>Experimental bounded 80 KiB arena 8W throughput: 4.961 GB/s (0.1691 s, -11.10% wall vs base)</title></rect><text class="bf-t-strong" x="235" y="204" style="font-size:10.5px">4.96 GB/s (+12.9%)</text><rect x="368" y="14" width="338" height="256" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="382" y="38" style="font-size:11.5px">8-worker peak memory delta vs baseline</text><line x1="484" y1="56" x2="484" y2="228" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="534" y1="56" x2="534" y2="228" style="stroke:var(--bf-vz-axis);stroke-width:1.4"/><line x1="584" y1="56" x2="584" y2="228" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="634" y1="56" x2="634" y2="228" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="484" y="246" text-anchor="middle" style="font-size:10px">−4</text><text class="bf-tm" x="534" y="246" text-anchor="middle" style="font-size:10px">0 MiB</text><text class="bf-tm" x="584" y="246" text-anchor="middle" style="font-size:10px">+4</text><text class="bf-tm" x="634" y="246" text-anchor="middle" style="font-size:10px">+8 MiB</text><text class="bf-tm" x="476" y="92" text-anchor="end" style="font-size:10.5px">Default base</text><rect x="532.5" y="76" width="3" height="24" rx="1" style="fill:var(--bf-vz-2)"><title>Default baseline 8W peak RSS: 104.61 MiB (0.00 MiB delta)</title></rect><text class="bf-tm" x="543" y="92" style="font-size:10.5px">0.0 MiB (104.6 MiB)</text><text class="bf-tm" x="476" y="148" text-anchor="end" style="font-size:10.5px">Unbounded slabs</text><rect x="534" y="132" width="81.6" height="24" rx="3" style="fill:var(--bf-vz-4)"><title>Unbounded 32 KiB slab pool 8W peak RSS delta: +6.53 MiB</title></rect><text class="bf-t-strong" x="622" y="148" style="font-size:10.5px">+6.53 MiB</text><text class="bf-tm" x="476" y="204" text-anchor="end" style="font-size:10.5px">Bounded 80 KiB</text><rect x="517.5" y="188" width="16.5" height="24" rx="3" style="fill:var(--bf-vz-1)"><title>Bounded 80 KiB arena 8W peak RSS delta: -1.32 MiB (103.29 MiB, 0.9900x baseline)</title></rect><text class="bf-t-strong" x="543" y="204" style="font-size:10.5px">−1.32 MiB (103.3 MiB)</text><text class="bf-tm" x="706" y="294" text-anchor="end" style="font-size:10.5px">Left: real pinned-scanner 8W throughput. Right: 8W peak RSS delta (−7.85 MiB from bounding).</text></svg>
<figcaption>Left: bounding worker scratch lifts real pinned-scanner eight-worker throughput from
4.69 GB/s to 4.96 GB/s (nearly matching the earlier 4.99 GB/s ideal-scanner ceiling). Right:
eight-worker peak resident memory drops 7.85 MiB and finishes 1.32 MiB below baseline.</figcaption>
</figure>

The single-worker and four-worker gains stayed completely intact as well. At one worker, the
bounded experimental walk decoded Wikidata in 1.0905 s against baseline's 1.3092 s. That is 17.1%
faster at identical memory. At four workers, it finished in 0.3010 s against 0.3626 s, running
16.8% faster in 0.7% less memory.

## 5. A sharper rule for multi-core optimization

This pair of experiments settles a subtle lesson about multi-stage pipelines on shared-cache
processors. When you speed up parallel workers until a serial front-end binds, memory footprint on
the workers stops being a separate concern from front-end latency. Even temporary scratch buffers
that stay private to each worker flow through the same shared L3 cache as the serial stage.

On our ten-core Intel workstation, pushing eight workers a few megabytes past the 20 MiB L3 cliff
silently stole 4% of the serial scanner's speed. Each extra megabyte of active working set cost
roughly 4.9 million scanner cycles. Keeping the four-lane slabs inside L1 data cache and reordering
chunks in place broke that coupling completely.

The bounded four-lane walk remains behind an experimental feature flag today. Its next step is a
full unpinned multi-corpus adoption run across Wikidata and six guard corpora. Only after that check
passes will we promote it into mibz2's default path.

---

**Sources:**

- Bounded 80 KiB per-worker slab arena with in-place 4 KiB chunk permutation: [#1312](https://github.com/lmmx/mibz2/pull/1312) (`src/open/lanes.rs`).
- Pre-bound core-zero PMU contention probe (CONFIRMED): [#1304](https://github.com/lmmx/mibz2/pull/1304), [verdict notebook](https://github.com/lmmx/mibz2/blob/540fb142/docs/notebook/2026-11-28-scanner-contention-residual-verdict.md), and archived bundle `origin/evidence` `a7ac01d8` (`scanner-contention/run-20261128T052216Z.tar.gz`).
- Post-bound causal recheck on Intel Core i9-10900K (EXPLAINED): [#1326](https://github.com/lmmx/mibz2/pull/1326), [verdict notebook](https://github.com/lmmx/mibz2/blob/540fb142/docs/notebook/2026-11-28-scanner-contention-bounded-recheck-verdict.md), and archived bundle `origin/evidence` `1950f77f` (`scanner-contention/run-20261128T065932Z.tar.gz`, SHA-256 `3857bbc196c499e28c8103736d4eda22a2ad27e55f779c78c1bc14efe2c29c82`).
- Synthesis derivation of the working-set contention slope (~4.9M cycles/MiB): [twenty-fifth synthesis pass](https://github.com/lmmx/mibz2/blob/540fb142/docs/notebook/2026-11-28-synthesis-twenty-fifth-pass.md).
