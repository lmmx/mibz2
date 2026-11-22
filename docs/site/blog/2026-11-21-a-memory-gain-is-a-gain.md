---
title: "Admitting dense blocks to the Open kernel: 29% less peak memory"
image: assets/social/blog-2026-11-21-a-memory-gain-is-a-gain.png
releases: ["0.5.11"]
description: "A bounded admission band takes five dense blocks off a 5.6 MiB-per-worker fallback, and the decoder runs slightly faster in over a quarter less memory."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Admitting dense blocks to the Open kernel: 29% less peak memory

A bounded admission band takes five dense blocks off a 5.6 MiB-per-worker fallback, and the decoder runs slightly faster in over a quarter less memory.
{: .post-dek }

21 November 2026 · 10 min read · [Research result](#5-what-transferred-from-the-research-line)
{: .post-meta }

Shipped in mibz2 [0.5.11](../benchmarks/history.md#release-0-5-11) ([changelog](https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0511---2026-11-21))
{: .post-shipped }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 is a bzip2 decoder in Rust. Most of its time goes on inverting the Burrows–Wheeler transform (BWT) of each block. Its fastest kernel for that, Open, used to refuse blocks with many short runs. On our Wikidata benchmark, five of the 233 full-size blocks were refused. They fell back to an older kernel that allocates about 5.6 MiB per worker, and that fallback set the decoder's peak memory. In mibz2 0.5.11, Open admits those blocks under a cap on its first pass ([#703](https://github.com/lmmx/mibz2/pull/703)).

**The effect** was measured on the i9-10900K we test releases on, which has ten cores and a 20 MiB shared L3. Over ten paired rounds, 0.5.11 against 0.5.10, on the Wikidata benchmark:

| threads | peak memory (RSS) | wall time | median peak RSS |
| ---: | ---: | ---: | ---: |
| 8 | 0.738 [0.710, 0.767] | 0.976 [0.965, 0.987] | 87.0 → 63.4 MiB |
| 10 | 0.709 [0.691, 0.728] | 0.960 [0.956, 0.965] | 93.9 → 66.5 MiB |

The output is byte-identical on every block.
The change, #703, is the only decoder change between the two releases. The intervals are 95% intervals of the paired ratio, over this one machine and corpus.

**The takeaways:**

- **The cause was a routing gap, not the kernel's cost.** Open's admission test refused blocks whose density r/n, runs per byte, exceeded 0.15. Five full-size Wikidata blocks sit between 0.151 and 0.206, and they paid for a full-table fallback.
- **A bounded band closes the gap.** Blocks with r/n from 0.15 to 0.28 are admitted if Open's first pass stays under a cap on its retained rows and thick strips. The five fit under it.
- **Two short arguments make the port safe.** No elimination step ever widens a strip, so the widest initial strip bounds every later step. And three working buffers can live inside an array that is dead until a late phase.
- **The memory claim transferred; the speed predictions did not.** The research line that proposed the change predicted the dense blocks would take under half the cycles. Natively, on one thread, they take 12–43% longer than on the fallback. The speed that does appear on the i9 comes from the smaller working set, not from the kernel being faster on these blocks.

## 1. Why did five blocks take the expensive path?

bzip2 cuts a file into blocks of up to 900 kB and applies the BWT to each. The transform reorders a block so that bytes with similar contexts sit together, and equal bytes form *runs*. mibz2 decodes several blocks at once, one per worker thread, and each worker needs its own scratch memory.

Open keeps only the rows of a block's narrow runs and moves each wide run as one *strip* ([Open returns](2026-11-19-open-returns.md)). Since 0.5.10 its scratch is about 1.2 MiB per worker, small enough for seven to ten workers to share the i9's 20 MiB L3. A block with many short runs leaves more rows for Open to keep. Open's admission test therefore used density, r/n, and refused blocks above 0.15. A refused block went to LazyCells and then Segmented, which builds a full table of the block at about 5.6 MiB per worker.

The benchmark is an 11.9 MB extract of a Wikidata dump: 431 blocks, of which 233 are full-size blocks on the route Open serves. Open took 228 of them. The other five have densities from 0.151 to 0.206. A worker that met one held the fallback's table, so the worst worker's memory was set by these five.

<figure class="bf-fig" id="routing">
<p class="bf-title">Where the 233 full-size blocks go, before and after the dense band</p>
<svg viewBox="0 0 720 300" role="img" aria-label="Measured routing. Before, the 228 blocks below r/n 0.15 go to Open and five blocks between 0.151 and 0.206 are refused and go to LazyCells then Segmented at about 5.6 MiB per worker. After, a dense band from 0.15 to 0.28 admits the same five blocks to Open within its cap."><line x1="120" y1="40" x2="120" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="120" y="268" class="bf-tm" text-anchor="middle">0.00</text><line x1="210.0" y1="40" x2="210.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="210.0" y="268" class="bf-tm" text-anchor="middle">0.05</text><line x1="300.0" y1="40" x2="300.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="300.0" y="268" class="bf-tm" text-anchor="middle">0.10</text><line x1="390.0" y1="40" x2="390.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="390.0" y="268" class="bf-tm" text-anchor="middle">0.15</text><line x1="480.0" y1="40" x2="480.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="480.0" y="268" class="bf-tm" text-anchor="middle">0.20</text><line x1="570.0" y1="40" x2="570.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="570.0" y="268" class="bf-tm" text-anchor="middle">0.25</text><line x1="660.0" y1="40" x2="660.0" y2="250" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="660.0" y="268" class="bf-tm" text-anchor="middle">0.30</text><text x="390" y="290" class="bf-tm" text-anchor="middle" style="font-size:12px">block density r/n: runs per byte of the block</text><rect x="120" y="40" width="270.0" height="210" style="fill:var(--bf-steel-tint);stroke:none"/><rect x="390.0" y="40" width="234.0" height="210" style="fill:var(--bf-green-tint);stroke:none"/><line x1="390.0" y1="30" x2="390.0" y2="250" style="stroke:var(--bf-ink);stroke-width:2"/><line x1="624.0" y1="30" x2="624.0" y2="250" style="stroke:var(--bf-green);stroke-width:2;stroke-dasharray:5 3"/><text x="390.0" y="24" class="bf-tm" text-anchor="middle">old line 0.15</text><text x="624.0" y="24" class="bf-tm" text-anchor="middle">dense band ends 0.28</text><text x="108" y="99" class="bf-ts" text-anchor="end" style="font-size:13px">0.5.10</text><text x="108" y="199" class="bf-ts" text-anchor="end" style="font-size:13px">#703</text><text x="255.0" y="82" class="bf-t-strong" text-anchor="middle">228 blocks: Open</text><text x="255.0" y="100" class="bf-tm" text-anchor="middle">(count gates; not plotted)</text><text x="255.0" y="182" class="bf-t-strong" text-anchor="middle">228 blocks: Open</text><text x="255.0" y="200" class="bf-tm" text-anchor="middle">(unchanged)</text><circle cx="391.62" cy="112" r="5" style="fill:var(--bf-amber)"/><circle cx="391.62" cy="212" r="5" style="fill:var(--bf-green)"/><circle cx="393.24" cy="112" r="5" style="fill:var(--bf-amber)"/><circle cx="393.24" cy="212" r="5" style="fill:var(--bf-green)"/><circle cx="398.82" cy="112" r="5" style="fill:var(--bf-amber)"/><circle cx="398.82" cy="212" r="5" style="fill:var(--bf-green)"/><circle cx="401.16" cy="112" r="5" style="fill:var(--bf-amber)"/><circle cx="401.16" cy="212" r="5" style="fill:var(--bf-green)"/><circle cx="490.44" cy="112" r="5" style="fill:var(--bf-amber)"/><circle cx="490.44" cy="212" r="5" style="fill:var(--bf-green)"/><text x="432.0" y="80" class="bf-t" text-anchor="start" style="font-size:12px">5 refused → LazyCells → Segmented</text><text x="432.0" y="97" class="bf-tm" text-anchor="start">about 5.6 MiB per worker</text><text x="432.0" y="180" class="bf-t" text-anchor="start" style="font-size:12px">5 admitted to Open, within the cap</text><text x="432.0" y="197" class="bf-tm" text-anchor="start">199k–232k rows, 568–2,729 strips</text><text x="396.0" y="134" class="bf-ttiny" text-anchor="start">41, 278, 56, 165</text><text x="490.44" y="134" class="bf-ttiny" text-anchor="middle">270</text><text x="396.0" y="234" class="bf-ttiny" text-anchor="start">41, 278, 56, 165</text><text x="490.44" y="234" class="bf-ttiny" text-anchor="middle">270</text></svg>
<figcaption>Measured routing on the benchmark's 431 blocks (#703's census). The 228 blocks below r/n 0.15 are admitted by Open's count gates in both versions; their individual densities are not plotted. The five above the line are plotted at their measured r/n. The 150 tiny blocks and the 48 blocks on the column route are unchanged and not shown.</figcaption>
</figure>

## 2. How is a dense block admitted safely?

The band admits a block with r/n between 0.15 and 0.28 if Open's first pass stays under a cap. The cap is min(256,000, 0.3n) retained rows and 16,384 thick strips. The check sits on the queues' cold path, so the first pass's hot loop is unchanged. Once a block passes either limit, a sticky flag stops the pass, and the block goes to LazyCells as before. Blocks the old test admitted keep their route and no cap. The five dense blocks fit, at up to 231,600 retained rows.

A denser block also means more elimination work, so the port needs a bound on it. Open eliminates intervals by Rauzy–Veech steps: each step replaces the wider of two strips with the difference of their widths. No step ever makes a strip wider, so the widest strip at the start bounds every later step. A block whose strips are all at most 32,768 rows wide therefore runs the plain elimination loop unchanged. A block that fails this certificate takes a contracted loop. It handles a long *spiral*, a run of steps that peels one strip against the same partner, as one step, building its word by repeated squaring.

On Wikidata, no block fails the certificate, so the contracted loop never runs. It is a bound on the worst case, not a speed-up.

## 3. Where do the extra buffers live?

Admitting the dense blocks must not bring back the memory it saves. The research line's second idea, *EdgesMux*, places working buffers inside the `edges` array, which the kernel does not write until a late phase. Its lifetime argument (Theorem 19.1 of the [cycle 19 journal](https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md)) is that each buffer's last read comes before `edges` is first written. Its verifier poisons each buffer after its last read, over 1,000 runs, and the output stays exact.

Most of this was already in 0.5.10, which put the elimination's nodes in `edges` ([#692](https://github.com/lmmx/mibz2/pull/692)). The port moves the remaining buffer, the strips' image order, into the first words of `edges`, worth 27–28 KiB a worker. Its tests poison each region of `edges` after its last read, and the output stays exact. Poisoned before that point, as a control, the output breaks.

## 4. What is the measured effect?

Two measurements bear on it: the port's own in-process A/B, in a 4-vCPU cloud container, and the release-scaling run on the i9.

The A/B runs master's kernel, unchanged, in the same binary as the candidate, and alternates the two. An A/A control, built from master's own tree, shows the noise floor. There are 12 rounds with 95% intervals.

| measure | A/A control | #703 against master |
| --- | ---: | ---: |
| kernel heap, worst worker, 10 workers (KiB) | – | 6,804.5 → 1,268.8 (0.186×) |
| 1 thread, six blocks Open already took | 0.993–1.019 | 1.003–1.009 |
| 1 thread, the five newly admitted blocks | 0.987–1.019 | 1.120–1.427 |
| 4 workers, time in the five blocks | 1.004 [0.994, 1.015] | 0.925 [0.909, 0.942] |
| 4 workers, a whole pass | 1.007 [1.000, 1.013] | 1.003 [0.991, 1.016] |
| end to end, 8 threads: peak memory | – | 0.741 [0.714, 0.768] |
| end to end, 8 threads: wall time | – | 0.978 [0.928, 1.031] |
| end to end, 10 threads: peak memory | – | 0.783 [0.754, 0.812] |
| end to end, 10 threads: wall time | – | 1.002 [0.957, 1.050] |

Ratios are #703's time or memory over master's. On a fresh kernel, each of the five blocks drops from 5.6 MiB to about 1 MiB.

The worst worker's kernel heap falls to under a fifth because no worker now holds the fallback's table. End to end, the process's peak memory falls by about a quarter, and in the container wall time does not move beyond noise. The five blocks are under 4% of a pass, and the container's cache is not the i9's shared L3.

On the i9, the release-scaling run compared 0.5.11 with 0.5.10 from 1 to 20 threads. At 1, 2 and 4 threads nothing changed beyond noise. From 8 threads up, 0.5.10 took 2.5% to 5.6% longer, with every interval excluding no change. Peak memory fell as in the container.

<figure class="bf-fig" id="footprint">
<p class="bf-title">Per-worker memory against each worker's share of the i9's 20 MiB L3</p>
<svg viewBox="0 0 720 310" role="img" aria-label="Measured per-worker memory in KiB against the i9's L3 share per worker, 2,926 at 7 workers and 2,048 at 10. Scratch on admitted blocks: 0.5.9 4,236, 0.5.10 1,200. Worst worker's kernel heap at 10 workers: 0.5.10 6,804.5 including the dense blocks' fallback, #703 1,268.8."><line x1="200.0" y1="30" x2="200.0" y2="262" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="200.0" y="280" class="bf-tm" text-anchor="middle">0</text><line x1="330.0" y1="30" x2="330.0" y2="262" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="330.0" y="280" class="bf-tm" text-anchor="middle">2,000</text><line x1="460.0" y1="30" x2="460.0" y2="262" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="460.0" y="280" class="bf-tm" text-anchor="middle">4,000</text><line x1="590.0" y1="30" x2="590.0" y2="262" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="590.0" y="280" class="bf-tm" text-anchor="middle">6,000</text><line x1="720.0" y1="30" x2="720.0" y2="262" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="720.0" y="280" class="bf-tm" text-anchor="middle">8,000</text><text x="450" y="300" class="bf-tm" text-anchor="middle" style="font-size:12px">KiB per worker</text><rect x="200.0" y="50" width="275.34000000000003" height="26" rx="3" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/><text x="190" y="67" class="bf-t" text-anchor="end" style="font-size:11.5px">0.5.9: scratch, admitted blocks</text><text x="481.34000000000003" y="67" class="bf-t-strong" text-anchor="start" style="font-size:12px">4,236</text><rect x="200.0" y="95" width="78.0" height="26" rx="3" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"/><text x="190" y="112" class="bf-t" text-anchor="end" style="font-size:11.5px">0.5.10: scratch, admitted blocks</text><text x="284.0" y="112" class="bf-t-strong" text-anchor="start" style="font-size:12px">1,200</text><rect x="200.0" y="165" width="442.2925" height="26" rx="3" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text x="190" y="182" class="bf-t" text-anchor="end" style="font-size:11.5px">0.5.10: worst worker, incl. dense fallback</text><text x="648.2925" y="182" class="bf-t-strong" text-anchor="start" style="font-size:12px">6,804.5</text><rect x="200.0" y="210" width="82.47199999999998" height="26" rx="3" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.5"/><text x="190" y="227" class="bf-t" text-anchor="end" style="font-size:11.5px">#703: worst worker</text><text x="288.472" y="227" class="bf-t-strong" text-anchor="start" style="font-size:12px">1,268.8</text><line x1="10" y1="140" x2="710" y2="140" style="stroke:var(--bf-frame);stroke-width:1"/><line x1="390.19" y1="30" x2="390.19" y2="262" style="stroke:var(--bf-ink);stroke-width:2;stroke-dasharray:6 3"/><line x1="333.12" y1="30" x2="333.12" y2="262" style="stroke:var(--bf-ink);stroke-width:2;stroke-dasharray:2 3"/><text x="394.19" y="40" class="bf-tm" text-anchor="start">share at 7 workers: 2,926</text><text x="329.12" y="40" class="bf-tm" text-anchor="end">10 workers: 2,048</text></svg>
<figcaption>Measured, two measures. The upper pair is per-worker scratch on the blocks Open admits (0.5.10's changelog, from #692). The lower pair is the worst worker's kernel heap over the whole corpus at 10 workers (#703's harness), which includes the fallback's table when a worker meets a refused block. The dashed lines are 20 MiB divided by 7 and by 10 workers.</figcaption>
</figure>

The wall-time gain on the i9 is consistent with the cache account. With ten workers, each worker's share of the L3 is about 2 MiB. A worker holding a 6.8 MiB fallback evicts its neighbours' working sets, while a worker at 1.3 MiB fits in its share. The container, with a different cache, could not show this. That is our account of the i9 result, not a measurement of cache misses.

## 5. What transferred from the research line?

The change came from Mechanisms, one of our AI research lines, whose cycle 19 built a candidate called `AhtEdgesMux16`. It measured the candidate in its own harness against mibz2 0.5.8. The harness includes a contention model, a simulator that charges each worker a share of the cache.

<figure class="bf-fig" id="claims">
<p class="bf-title">The research line's claims against the native measurements</p>
<svg viewBox="0 0 720 582" role="img" aria-label="Claims against native results. Dense blocks' time: the model claimed 0.42 to 0.46; in the container 1.12 to 1.43 on one thread and 0.925 with four workers. Worst worker's heap: claimed 0.135 against 0.5.8, measured 0.186 against 0.5.10. Whole decoder at 8 threads: container peak memory 0.741, wall 0.978; i9 peak memory 0.738, wall 0.976. At 10 threads: container 0.783 and 1.002; i9 0.709 and 0.960."><line x1="710.0" y1="30" x2="710.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="710.0" y="552" class="bf-tm" text-anchor="middle">1.6</text><line x1="650.0" y1="30" x2="650.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="650.0" y="552" class="bf-tm" text-anchor="middle">1.4</text><line x1="590.0" y1="30" x2="590.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="590.0" y="552" class="bf-tm" text-anchor="middle">1.2</text><line x1="530.0" y1="30" x2="530.0" y2="534" style="stroke:var(--bf-frame);stroke-width:2"/><text x="530.0" y="552" class="bf-tm" text-anchor="middle">1.0</text><line x1="470.0" y1="30" x2="470.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="470.0" y="552" class="bf-tm" text-anchor="middle">0.8</text><line x1="410.0" y1="30" x2="410.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="410.0" y="552" class="bf-tm" text-anchor="middle">0.6</text><line x1="350.0" y1="30" x2="350.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="350.0" y="552" class="bf-tm" text-anchor="middle">0.4</text><line x1="290.0" y1="30" x2="290.0" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="290.0" y="552" class="bf-tm" text-anchor="middle">0.2</text><line x1="230" y1="30" x2="230" y2="534" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="230" y="552" class="bf-tm" text-anchor="middle">0.0</text><text x="10" y="44" class="bf-t-strong" text-anchor="start" style="font-size:12px">dense blocks, time</text><text x="222" y="68" class="bf-tm" text-anchor="end" style="font-size:11px">claim: model, 8–10 workers</text><line x1="355.7" y1="64" x2="367.4" y2="64" style="stroke:var(--bf-claude);stroke-width:4"/><text x="222" y="96" class="bf-tm" text-anchor="end" style="font-size:11px">container: 1 thread</text><line x1="566.0" y1="92" x2="658.1" y2="92" style="stroke:var(--bf-amber);stroke-width:4"/><text x="222" y="124" class="bf-tm" text-anchor="end" style="font-size:11px">container: 4 workers</text><line x1="502.7" y1="120" x2="512.5999999999999" y2="120" style="stroke:var(--bf-blue);stroke-width:4"/><circle cx="507.5" cy="120" r="5" style="fill:var(--bf-blue)"/><line x1="10" y1="142" x2="710" y2="142" style="stroke:var(--bf-frame);stroke-width:1"/><text x="10" y="162" class="bf-t-strong" text-anchor="start" style="font-size:12px">worst worker's heap</text><text x="222" y="186" class="bf-tm" text-anchor="end" style="font-size:11px">claim: vs 0.5.8</text><circle cx="270.5" cy="182" r="5" style="fill:var(--bf-claude)"/><text x="222" y="214" class="bf-tm" text-anchor="end" style="font-size:11px">container: vs 0.5.10, 10 workers</text><circle cx="285.8" cy="210" r="5" style="fill:var(--bf-green)"/><line x1="10" y1="232" x2="710" y2="232" style="stroke:var(--bf-frame);stroke-width:1"/><text x="10" y="252" class="bf-t-strong" text-anchor="start" style="font-size:12px">whole decoder, 8 threads</text><text x="222" y="276" class="bf-tm" text-anchor="end" style="font-size:11px">container: peak memory</text><line x1="444.2" y1="272" x2="460.4" y2="272" style="stroke:var(--bf-green);stroke-width:4"/><circle cx="452.3" cy="272" r="5" style="fill:var(--bf-green)"/><text x="222" y="304" class="bf-tm" text-anchor="end" style="font-size:11px">container: wall time</text><line x1="508.40000000000003" y1="300" x2="539.3" y2="300" style="stroke:var(--bf-blue);stroke-width:4"/><circle cx="523.4" cy="300" r="5" style="fill:var(--bf-blue)"/><text x="222" y="332" class="bf-tm" text-anchor="end" style="font-size:11px">i9: peak memory</text><line x1="443.0" y1="328" x2="460.1" y2="328" style="stroke:var(--bf-green);stroke-width:4"/><circle cx="451.4" cy="328" r="5" style="fill:var(--bf-green)"/><text x="222" y="360" class="bf-tm" text-anchor="end" style="font-size:11px">i9: wall time</text><line x1="519.5" y1="356" x2="526.1" y2="356" style="stroke:var(--bf-blue);stroke-width:4"/><circle cx="522.8" cy="356" r="5" style="fill:var(--bf-blue)"/><line x1="10" y1="378" x2="710" y2="378" style="stroke:var(--bf-frame);stroke-width:1"/><text x="10" y="398" class="bf-t-strong" text-anchor="start" style="font-size:12px">whole decoder, 10 threads</text><text x="222" y="422" class="bf-tm" text-anchor="end" style="font-size:11px">container: peak memory</text><line x1="456.2" y1="418" x2="473.6" y2="418" style="stroke:var(--bf-green);stroke-width:4"/><circle cx="464.9" cy="418" r="5" style="fill:var(--bf-green)"/><text x="222" y="450" class="bf-tm" text-anchor="end" style="font-size:11px">container: wall time</text><line x1="517.0999999999999" y1="446" x2="545.0" y2="446" style="stroke:var(--bf-blue);stroke-width:4"/><circle cx="530.6" cy="446" r="5" style="fill:var(--bf-blue)"/><text x="222" y="478" class="bf-tm" text-anchor="end" style="font-size:11px">i9: peak memory</text><line x1="437.29999999999995" y1="474" x2="448.4" y2="474" style="stroke:var(--bf-green);stroke-width:4"/><circle cx="442.7" cy="474" r="5" style="fill:var(--bf-green)"/><text x="222" y="506" class="bf-tm" text-anchor="end" style="font-size:11px">i9: wall time</text><line x1="516.8" y1="502" x2="519.5" y2="502" style="stroke:var(--bf-blue);stroke-width:4"/><circle cx="518.0" cy="502" r="5" style="fill:var(--bf-blue)"/><text x="470" y="572" class="bf-tm" text-anchor="middle" style="font-size:12px">new ÷ old (below 1: less time or memory)</text></svg>
<figcaption>Ratios, new over old; below 1 is better. Claims are the line's own (cycle 19 journal), against 0.5.8 and from its contention model. Container figures are #703's, against 0.5.10. i9 figures are 0.5.11 against 0.5.10 from the release-scaling run's records, paired by round. Bars are 95% intervals where they exist.</figcaption>
</figure>

**The memory claim held,** against a newer baseline. The line measured the ten-worker maximum heap falling from 9,979 KiB to 1,346 KiB against 0.5.8. By 0.5.10 the admitted blocks were already at about 1.3 MiB. The gain that remains is the dense blocks' fallback, which no longer allocates its table.

**The speed predictions did not.** The contention model predicted the dense blocks would take under half the cycles, with a cache share of about 2 MiB per worker. Natively, on one thread, Open is slower than the fallback on these blocks. With the cache to itself, the fallback's full table is fast. With four workers in the container, Open takes 7.5% less time on them. On the i9 the whole decoder is faster from 8 threads up. That gain is spread over all blocks, through the smaller working set, rather than concentrated in the five. Its other claims shrank too. EdgesMux was worth 432 KiB against 0.5.8, but most of it had already shipped in 0.5.10. And the spiral contraction never runs on Wikidata.

**Correctness.** Every block of the corpus, and six authenticated reference blocks, decode byte for byte as the plain inverse does. The `bzcat` example at 4, 8 and 10 threads gives the same SHA-256 as `bzip2 -dc`. The correctness gate passes its 338 cases, and eight hand-made mutations of the new code were each caught by the tests. CI's paired bench at 1 to 4 threads found no change in Wikidata's wall time beyond noise.

The change shipped in mibz2 0.5.11.

---

**Research records:**
[Mechanisms' cycle 19 journal](https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md);
[the port's journal](https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission.md) and [its results](https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission/results.txt) (#703);
[0.5.11's release-scaling records on the i9](https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.11) and [its attribution row](https://github.com/lmmx/mibz2/blob/master/docs/journal/release-attribution.md);
[0.5.10's memory diet](https://github.com/lmmx/mibz2/pull/692).
