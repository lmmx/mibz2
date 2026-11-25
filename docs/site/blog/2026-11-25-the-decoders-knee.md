---
title: "The decoder's knee"
image: assets/social/blog-2026-11-25-the-decoders-knee.png
description: "A two-station queue predicts from four worker counts where adding workers stops helping, and three independent readings turned that into experiments."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The decoder's knee

A two-station queue predicts from four worker counts where adding workers stops helping, and three independent readings turned that into experiments.
{: .post-dek }

25 November 2026 · 12 min read · [Research in progress](#8-what-runs-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2's decoder is one serial scanner feeding parallel workers, and the simplest model of such a
thing, a two-station queue, turns out to describe it almost exactly. Measured at one to four
workers, it predicts where adding workers stops helping, at about 7.5 on our ten-core machine, and
the measurement across every count from 1 to 20 puts the crossover at 7.67. Above that point the
scanner sets the speed, so reorganising how work is queued can recover only a few per cent; real
gains need less work at the bottleneck, and when they arrive the bottleneck moves back to the
workers, so improvements have to be priced together. Reading the performance-analysis literature
with several independent readers then caught an error in our own first reading and found about 8%
of the scanner's work in two avoidable copies. The experiments that follow are designed to say
which mechanism controls the decoder at which worker count, and so which costs belong to
engineering and which might yield to mathematics. This post picks up where
[Posing the inversion problem](2026-11-24-posing-the-inversion-problem.md) left off.

**Takeaways**

- A two-station model fitted on four worker counts predicts the knee at 7.49 workers; the
  measured crossover is 7.67, and the decoder's best wall, 0.191 s, is at 8.
- Above the knee the decoder runs at 0.945 or better of the busiest server's limit at every count,
  so no queueing discipline can recover more than about 5%.
- Independent readings corrected the first: the public API never runs 20 workers on this machine,
  and the scanner spends about 8% of its demand on two avoidable copies.
- Stage 1 of the characterisation confirmed the switch from workers to scanner at exactly 8, and
  found that asking for 3, 5 or 7 threads selects an older kernel that is 61% slower at 7.

## 1. A simple observation

The four-arm test supplied each of the decoder's two stages for free, outside the clock. At ten
workers, supplying the codes for emission saved nothing measurable (0.976 of the shipped time),
supplying the scanner's output saved 19% (0.806), and supplying both saved 26% (0.741): a gain
invisible until another bottleneck was removed
([Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md)). An
observation like that asks for a model of the decoder as a system rather than as a list of kernels,
which is what the programme set out in the previous post is for.

## 2. The first model, and its first test

The simplest model is a serial stage feeding k identical parallel servers. Each block costs the
scanner s and a worker w, so the decoder's wall time is set by the workers, as a/k, until enough
of them are running that the scanner cannot keep up, after which it is set by the scanner alone.
The crossover falls at k = w/s. From the measurements at one to four workers only, s = 106.6 µs and
w = 798.3 µs a block, which puts the knee at 7.49 workers. Across all twenty counts the measured
crossover is 7.67, and a fit of c + max(b, a/k) gives a = 1.279 s, b = 0.178 s and c = 0.032 s
([journal §57](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md); [the data](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv)).

<figure class="bf-fig" id="knee">
<p class="bf-title">Wall time against workers: the knee predicted from four counts</p>
<svg viewBox="0 0 720 400" role="img" aria-label="Decode wall time against workers from 1 to 20 on the i9-10900K. It falls as a over k, from 1.30 s at one worker, to a best of 0.191 s at 8 workers, stays near 0.197 s from 8 to 14, and steps up to about 0.23 s from 16. The knee predicted from counts 1 to 4 alone is 7.49 workers; the measured crossover is 7.67."><line x1="64" x2="696" y1="282.3" y2="282.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="286.3" text-anchor="end" style="font-size:12px">0.1</text><line x1="64" x2="696" y1="238.6" y2="238.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="242.6" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="194.9" y2="194.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="198.9" text-anchor="end" style="font-size:12px">0.3</text><line x1="64" x2="696" y1="151.1" y2="151.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="155.1" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="107.4" y2="107.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="111.4" text-anchor="end" style="font-size:12px">0.5</text><line x1="64" x2="696" y1="63.7" y2="63.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="67.7" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">0.7</text><text class="bf-tm" x="64.0" y="344" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="344" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="344" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="344" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="344" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="344" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="362" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="173" text-anchor="middle" transform="rotate(-90 16 173)" style="font-size:12px">decode wall time (s)</text><polyline points="64.0,20.0 67.3,20.0 70.7,20.0 74.0,20.0 77.3,20.0 80.6,20.0 84.0,20.0 87.3,20.0 90.6,20.0 93.9,20.0 97.3,32.5 100.6,45.8 103.9,57.9 107.2,68.9 110.6,79.1 113.9,88.4 117.2,97.0 120.5,104.9 123.9,112.3 127.2,119.2 130.5,125.6 133.9,131.7 137.2,137.3 140.5,142.6 143.8,147.6 147.2,152.3 150.5,156.7 153.8,160.9 157.1,164.9 160.5,168.7 163.8,172.2 167.1,175.6 170.4,178.9 173.8,182.0 177.1,184.9 180.4,187.8 183.7,190.5 187.1,193.1 190.4,195.5 193.7,197.9 197.1,200.2 200.4,202.4 203.7,204.5 207.0,206.5 210.4,208.5 213.7,210.4 217.0,212.2 220.3,213.9 223.7,215.6 227.0,217.2 230.3,218.8 233.6,220.4 237.0,221.8 240.3,223.3 243.6,224.7 246.9,226.0 250.3,227.3 253.6,228.6 256.9,229.8 260.3,231.0 263.6,232.1 266.9,233.3 270.2,234.2 273.6,234.2 276.9,234.2 280.2,234.2 283.5,234.2 286.9,234.2 290.2,234.2 293.5,234.2 296.8,234.2 300.2,234.2 303.5,234.2 306.8,234.2 310.1,234.2 313.5,234.2 316.8,234.2 320.1,234.2 323.5,234.2 326.8,234.2 330.1,234.2 333.4,234.2 336.8,234.2 340.1,234.2 343.4,234.2 346.7,234.2 350.1,234.2 353.4,234.2 356.7,234.2 360.0,234.2 363.4,234.2 366.7,234.2 370.0,234.2 373.3,234.2 376.7,234.2 380.0,234.2 383.3,234.2 386.7,234.2 390.0,234.2 393.3,234.2 396.6,234.2 400.0,234.2 403.3,234.2 406.6,234.2 409.9,234.2 413.3,234.2 416.6,234.2 419.9,234.2 423.2,234.2 426.6,234.2 429.9,234.2 433.2,234.2 436.5,234.2 439.9,234.2 443.2,234.2 446.5,234.2 449.9,234.2 453.2,234.2 456.5,234.2 459.8,234.2 463.2,234.2 466.5,234.2 469.8,234.2 473.1,234.2 476.5,234.2 479.8,234.2 483.1,234.2 486.4,234.2 489.8,234.2 493.1,234.2 496.4,234.2 499.7,234.2 503.1,234.2 506.4,234.2 509.7,234.2 513.1,234.2 516.4,234.2 519.7,234.2 523.0,234.2 526.4,234.2 529.7,234.2 533.0,234.2 536.3,234.2 539.7,234.2 543.0,234.2 546.3,234.2 549.6,234.2 553.0,234.2 556.3,234.2 559.6,234.2 562.9,234.2 566.3,234.2 569.6,234.2 572.9,234.2 576.3,234.2 579.6,234.2 582.9,234.2 586.2,234.2 589.6,234.2 592.9,234.2 596.2,234.2 599.5,234.2 602.9,234.2 606.2,234.2 609.5,234.2 612.8,234.2 616.2,234.2 619.5,234.2 622.8,234.2 626.1,234.2 629.5,234.2 632.8,234.2 636.1,234.2 639.5,234.2 642.8,234.2 646.1,234.2 649.4,234.2 652.8,234.2 656.1,234.2 659.4,234.2 662.7,234.2 666.1,234.2 669.4,234.2 672.7,234.2 676.0,234.2 679.4,234.2 682.7,234.2 686.0,234.2 689.3,234.2 692.7,234.2 696.0,234.2" style="fill:none;stroke:var(--bf-steel);stroke-width:2"/><line x1="279.8" x2="279.8" y1="20" y2="326" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text class="bf-t-strong" x="285.8" y="36" style="font-size:12px;fill:var(--bf-amber)">knee predicted from counts 1–4: 7.49</text><text class="bf-tm" x="285.8" y="52" style="font-size:12px">measured crossover: 7.67</text><path d="M64.0,22 l-5,8 h10 z" style="fill:var(--bf-green)"/><text class="bf-tm" x="72.0" y="30" style="font-size:11px">1.30 s</text><circle cx="97.3" cy="27.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="130.5" cy="120.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="163.8" cy="170.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="197.1" cy="201.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="230.3" cy="221.9" r="4.5" style="fill:var(--bf-green)"/><circle cx="263.6" cy="235.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="296.8" cy="242.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="330.1" cy="239.8" r="4.5" style="fill:var(--bf-green)"/><circle cx="363.4" cy="238.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="396.6" cy="240.0" r="4.5" style="fill:var(--bf-green)"/><circle cx="429.9" cy="239.7" r="4.5" style="fill:var(--bf-green)"/><circle cx="463.2" cy="240.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="496.4" cy="239.5" r="4.5" style="fill:var(--bf-green)"/><circle cx="529.7" cy="233.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="562.9" cy="231.4" r="4.5" style="fill:var(--bf-green)"/><circle cx="596.2" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="629.5" cy="224.1" r="4.5" style="fill:var(--bf-green)"/><circle cx="662.7" cy="224.6" r="4.5" style="fill:var(--bf-green)"/><circle cx="696.0" cy="225.3" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="137.2" y="129.3" style="font-size:12px">workers bind: wall ≈ a/k</text><text class="bf-tm" x="396.6" y="212.3" style="font-size:12px">scanner binds: wall ≈ b</text><text class="bf-tm" x="549.6" y="194.9" style="font-size:12px">step at 16: the scanner slows 23%</text><circle cx="74" cy="388" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="84" y="392" style="font-size:12px">measured (median of 20 decodes)</text><line x1="324" x2="354" y1="388" y2="388" style="stroke:var(--bf-steel);stroke-width:2"/><text class="bf-tm" x="362" y="392" style="font-size:12px">fitted two-station model, c + max(b, a/k)</text></svg>
<figcaption>Measured on an i9-10900K (10 cores, 20 threads), one corpus of 1,724 blocks and 839 MB
of output, median of 20 decodes at each count, unpinned. The fitted model (steel) has the workers'
asymptote a/k on the left and the scanner's b on the right; the dashed line is the knee predicted
from counts 1 to 4 alone. The wall at one worker, 1.30 s, is off the scale.</figcaption>
</figure>

The best wall is 0.191 s, at 8 workers (4,390 MB/s of output), and from 8 to 14 workers it stays
flat at 0.197 ± 0.005 s. At 16 and above it steps up 20%, to 0.230 s, because the scanner itself
slows by 23% while its share of the wall stays at 0.97. The scanner's work is 81% Huffman decoding,
at 1.41 ns a symbol at one worker and 1.51 ns from three. The second figure shows the switch from
the scanner's side: at one worker it spends most of its time waiting for a free slot, and from 8
workers it almost never waits.

<figure class="bf-fig" id="scanner-split">
<p class="bf-title">The scanner's time, working and waiting</p>
<svg viewBox="0 0 720 350" role="img" aria-label="The scanner thread's time against workers. Its work stays near 0.19 s from 3 to 15 workers and rises to about 0.23 s from 16. Its waiting for a free slot falls from 1.12 s at one worker to under a millisecond at 8 and beyond."><line x1="64" x2="696" y1="239.4" y2="239.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="243.4" text-anchor="end" style="font-size:12px">0.2</text><line x1="64" x2="696" y1="202.9" y2="202.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="206.9" text-anchor="end" style="font-size:12px">0.4</text><line x1="64" x2="696" y1="166.3" y2="166.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="170.3" text-anchor="end" style="font-size:12px">0.6</text><line x1="64" x2="696" y1="129.7" y2="129.7" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="133.7" text-anchor="end" style="font-size:12px">0.8</text><line x1="64" x2="696" y1="93.1" y2="93.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="97.1" text-anchor="end" style="font-size:12px">1</text><line x1="64" x2="696" y1="56.6" y2="56.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="60.6" text-anchor="end" style="font-size:12px">1.2</text><line x1="64" x2="696" y1="20.0" y2="20.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="56" y="24.0" text-anchor="end" style="font-size:12px">1.4</text><text class="bf-tm" x="64.0" y="294" text-anchor="middle" style="font-size:12px">1</text><text class="bf-tm" x="163.8" y="294" text-anchor="middle" style="font-size:12px">4</text><text class="bf-tm" x="296.8" y="294" text-anchor="middle" style="font-size:12px">8</text><text class="bf-tm" x="429.9" y="294" text-anchor="middle" style="font-size:12px">12</text><text class="bf-tm" x="562.9" y="294" text-anchor="middle" style="font-size:12px">16</text><text class="bf-tm" x="696.0" y="294" text-anchor="middle" style="font-size:12px">20</text><text class="bf-tm" x="380" y="312" text-anchor="middle" style="font-size:12px">workers</text><text class="bf-tm" x="16" y="148" text-anchor="middle" transform="rotate(-90 16 148)" style="font-size:12px">scanner thread time (s)</text><rect x="54.2" y="244.2" width="19.6" height="31.8" style="fill:var(--bf-amber)"/><rect x="54.2" y="39.5" width="19.6" height="204.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="87.5" y="242.7" width="19.6" height="33.3" style="fill:var(--bf-amber)"/><rect x="87.5" y="151.6" width="19.6" height="91.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="120.7" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="120.7" y="191.4" width="19.6" height="50.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="154.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="154.0" y="212.1" width="19.6" height="30.0" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="187.3" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="187.3" y="225.2" width="19.6" height="16.9" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="220.5" y="242.1" width="19.6" height="33.9" style="fill:var(--bf-amber)"/><rect x="220.5" y="233.5" width="19.6" height="8.6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="253.8" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="253.8" y="239.2" width="19.6" height="2.8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="287.0" y="242.0" width="19.6" height="34.0" style="fill:var(--bf-amber)"/><rect x="287.0" y="241.8" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="320.3" y="241.6" width="19.6" height="34.4" style="fill:var(--bf-amber)"/><rect x="320.3" y="240.9" width="19.6" height="0.7" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="353.6" y="241.4" width="19.6" height="34.6" style="fill:var(--bf-amber)"/><rect x="353.6" y="240.9" width="19.6" height="0.5" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="386.8" y="241.3" width="19.6" height="34.7" style="fill:var(--bf-amber)"/><rect x="386.8" y="241.0" width="19.6" height="0.3" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="420.1" y="241.2" width="19.6" height="34.8" style="fill:var(--bf-amber)"/><rect x="420.1" y="241.0" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="453.4" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="453.4" y="240.9" width="19.6" height="0.1" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/><rect x="486.6" y="241.0" width="19.6" height="35.0" style="fill:var(--bf-amber)"/><rect x="519.9" y="240.8" width="19.6" height="35.2" style="fill:var(--bf-amber)"/><rect x="553.2" y="234.0" width="19.6" height="42.0" style="fill:var(--bf-amber)"/><rect x="586.4" y="234.7" width="19.6" height="41.3" style="fill:var(--bf-amber)"/><rect x="619.7" y="234.3" width="19.6" height="41.7" style="fill:var(--bf-amber)"/><rect x="652.9" y="234.6" width="19.6" height="41.4" style="fill:var(--bf-amber)"/><rect x="686.2" y="234.8" width="19.6" height="41.2" style="fill:var(--bf-amber)"/><text class="bf-tm" x="346.7" y="162.6" style="font-size:12px">from 8 workers the scanner almost never waits:</text><text class="bf-tm" x="346.7" y="178.6" style="font-size:12px">it is the bottleneck, and only less work there helps</text><rect x="68" y="330" width="12" height="12" style="fill:var(--bf-amber)"/><text class="bf-tm" x="86" y="341" style="font-size:12px">scanner working</text><rect x="234" y="330" width="12" height="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><text class="bf-tm" x="252" y="341" style="font-size:12px">scanner waiting for a free slot</text></svg>
<figcaption>Measured, same run. Below the knee the scanner waits for the workers to free a slot;
from 8 workers on it works without pause, so it is the bottleneck and only a lighter scanner moves
the wall.</figcaption>
</figure>

One question follows directly: could a better queueing discipline, size-aware dispatch, say, or
helpers for a slow block, do far better? The operational laws answer it before any code is written.
No discipline raises the ceiling, which the demands set; a discipline can only bring the decoder
closer to it ([the discipline lead](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md)). And the decoder already runs at 0.945 to 0.998 of the
busiest server's limit at every worker count, so no reordering could recover more than about 5%,
or 3% on the plateau. The large gains lie in less work at the bottleneck, or more servers for it.

## 3. Reading the field, four times over

Three performance-analysis courses were read in full, with their sources saved, and then mined for
opportunities against these measurements by four readers, each writing an answer before seeing the
others ([the summary](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md)). Three of those readings are in; the fourth, aimed at where
mathematics rather than engineering fits, is still to come.

| Reader | What it found that the others had not |
|---|---|
| [The first reading](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md) | right-sizing the worker count; putting the bottleneck on the fastest core; mean-value analysis in place of the hard-max fit |
| [An outside model](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md) | the public API's actual worker count; bottleneck migration; the variability of block sizes |
| [A second agent](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md) | two avoidable copies in the scanner, about 8% of its demand; the same worker-count error |

The disagreements were the useful part. The first reading had claimed that the Python binding's
default of 20 workers costs 20% of the wall. Two readers, independently, found that the public API
never runs 20 workers on this machine: a request for ten or more threads builds 7 workers with a
28-slot window, so the live question is 7 against 8, 0.207 s against 0.191 s, about 7.6% for under
1% more CPU ([journal §61](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). The cap of 7 was set because workers beyond it bought nothing; it
is a symptom of the scanner's limit rather than a reason to stop raising it.

The second agent's reading found engineering that the first had missed, both confirmed in the code
([journal §62](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). At the boundary between the feeder's 4 MiB chunks and the scanner's 2 MiB
look-ahead, the scanner copies the held part of its window on its own thread, about 10 ms at 8
workers, or 5.4% of its demand. And each block's symbol buffer is zero-filled again when it is
reused, about 5 ms more, driven by the 792 tiny blocks in the corpus. Together that is about 15 ms,
8% of the scanner's demand, with no algorithmic risk, and it has been routed for implementation as
an ordinary performance change.

The outside reading supplied the warning that prices every such change. A cut in the scanner's
work moves the bottleneck back to the workers: at 8 workers, a 5% cut in the scanner's Huffman
demand already reaches the workers' limit, so scanner and worker improvements must be priced
together, with the worker count retuned. Stage 1's figures then re-priced the copy fix itself: at 8
workers the workers sit within 0.3 ms of the scanner, so a 15 ms cut there is almost entirely
masked (0.1941 s against 0.1944 s), while at 10 workers it is worth about 7.5%
([journal §69](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). Those figures are instrumented, so a paired run at 8, 9 and 10 workers
decides between the two estimates.

## 4. Stage 1: the decoder across twenty counts

The first stage of the characterisation ran 840 grid decodes across every count from 1 to 20, with
every block's route, stage boundaries and queue transitions logged
([journal §66](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md); [the report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt)). All nine operational-law identities held in every
decode, Little's law exactly. What it found:

- **The two-station model holds.** The limit moves from the workers to the scanner at exactly 8
  workers, with a bootstrap interval of [8, 8], and a best wall of 0.1944 s there.
- **The scanner limits above the knee, by concurrent state.** The workers' idle time while the
  scanner is working rises from 0.035 of their time at 8 workers to 0.124 at 10 and 0.551 at 20;
  their idle time while it waits for input stays under 0.01.
- **The reader is not a co-bottleneck:** its demand is 0.017–0.035 ms a block against the scanner's
  0.099–0.130.
- **Contention inflates service per visit.** The scanner's Huffman decode costs 84 µs a block at
  one worker and 105–107 µs at 16 and above; emission rises from 526 µs at one worker to 816 at 14,
  and the memory controller's reads per decode from 219 MiB to 1,957 at 16, which at about 8 GB/s
  at 10 workers points to cache capacity rather than bandwidth.
- **The public API selects an older kernel below 8.** Asking for 3, 5 or 7 threads builds the
  eager kernel, because the lazy plan begins at 8 workers and the measured settings cover only 4,
  6 and 10 or more. At 7 threads that is 0.337 s and 140 MB, against 0.209 s and 54 MB for seven
  lazy workers: 61% slower, with 2.6 times the memory. It is a production default, so it goes to a
  registered recalibration across corpora.

The run flagged its own instrument as perturbing at 10 and 20 workers: the cost check's intervals
include 1 but are wide, so timings from the ledgers at high counts may be inflated by up to about
9%, while shares, counts and the identities are unaffected.

## 5. Designing for a purpose

A question about method came next: whether the decoder could be studied the way systems biology
studies a regulatory network, with Uri Alon's network motifs as the obvious model. The answer that
held up was not his catalogue of recurring motifs but the line of work that asks which structures
can perform a given function at all ([the lead](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md); [the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-alon-motifs.md)).
[Ma and colleagues](https://doi.org/10.1016/j.cell.2009.06.013){ .il-ref } enumerated every
three-node network topology to find the few able to achieve biochemical adaptation, and
[Araujo and Liotta](https://doi.org/10.1038/s41467-018-04151-6){ .il-ref } later gave the
solution space a basis, showing that every network capable of robust perfect adaptation is built
from two kinds of module.
[Khammash](https://doi.org/10.1016/j.cels.2021.05.020){ .il-ref } states the programme in five
steps: specify the function, derive necessary conditions, characterise the solution space, check
realisability, and establish optimality ([an explanation](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-khammash.md)). Ours maps onto it closely:

| Khammash's step | Ours, already | What is missing |
|---|---|---|
| The function | the referee | |
| Necessary conditions | G1, and records 118 and 119 | a necessity statement in positive form |
| The solution space | | an enumeration, and then a basis |
| Realisability | the four-arm and profile ceilings | |
| Optimality | the performance work | |

The proposal that follows is to enumerate the referee's own class of decoders (small cursor
machines with a fixed number of passes that invert every short string over a tiny alphabet),
reduce the list by symmetry, cluster the survivors, look for the condition that separates them, and
use a SAT solver to prove "no survivor" within the class ([journal §65](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). Two ideas from the
same reading were adopted as method: the same function can be realised by different structures,
which is why the referee counts structure; and a representation change is a search for the hidden
coordinate in which the work cancels.

## 6. The dynamical layer

The decoder is "digital on analogue": discrete events, a block admitted or a buffer released, on top
of continuous quantities such as occupancy and time. The language for that is the theory of hybrid
dynamical systems ([the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-dynamics.md); [journal §67](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)), and Goebel, Sanfelice and
Teel's book on it was read in full, with notes in our own words. Its plainest product is a list of
five levers. Any reorganisation of the decoder changes one of five things
([the levers](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md)):

1. how fast work runs (the flow map);
2. where work may proceed (the flow set);
3. when events may happen (the jump set);
4. what events do (the jump map);
5. what is represented (the state space).

<p class="il-rubric-head">Of a bound that was not a certificate</p>

One reading of the book went too far, and an independent review caught it before it guided any
experiment ([the review](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md); [journal §70](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md)). The bottleneck maximum, the largest of the
scanner's, the workers' and the reader's demands, is only a lower bound on the time a decode takes,
because it ignores dependencies, starvation and blocking. One block that takes a 1 s scan, a 1 s
decode and a 1 s delivery in series has a maximum of 1 s and takes 3. The notes now keep three
quantities apart: lower bounds, used to prune; predictors, used to rank, and validated on held-out
runs rather than by replaying a run with its own durations; and proved upper bounds, for which
backward induction over a finite, acyclic state graph is the concrete route.

## 7. How the work is done

The method is written down so that it can be reused
([the protocol](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/process/protocol.md)). It starts from a measured surprise rather than a technique, poses the
problem and its referee before any solution, and prefers a model that predicts from little data and
is checked on data it did not see. Sources are read in full and saved. Several readers and
reviewers answer independently before comparing, their answers are recorded verbatim, and decisions
are kept in separate files with their reasons. Wrong claims are corrected in place and in public,
naming who found the error, as the worker-count error and the lower bound above were. Every run
registers its decision rule before its data, changes are priced jointly because a relieved
bottleneck moves, and when it moves, the ideas it may have masked are tested again.

## 8. What runs next

Five runs are planned, in order ([the programme of experiments](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/briefs/experiment-programme.md)): the stage 1 baseline, now
done; the scanner's copy fix, as a paired comparison at 8, 9 and 10 workers; stage 2, which slows
each activity in turn to say which controls the wall at which count; the recalibration of the
worker count, including 7 against 8 and the kernel chosen below 8; and the joint frontier of scanner
and worker improvements together. Stage 2 is running now, and none of its results are in. Each cost
it finds controlling will be classed as engineering, a matter of how the work is done, or as a
candidate for mathematics, a matter of what work is done.

---

**Sources:**

- The measurements: [the scanner profile](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-scan-profile.tsv) (journal §57) and [stage 1's report](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt) (journal §66).
- The readings: [the summary](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reports/2026-11-25-course-mining.md), [the first reading with its corrections](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/course-opportunities.md), [the outside reading](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-course-mining.md) and [the second agent's](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reviews/neon-course-mining.md); journal §§61–62 and §69.
- The design thread: [structure and function](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/structure-and-function.md), with [the outside answer on motifs](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-alon-motifs.md) and [on Khammash](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-khammash.md); journal §§64–65.
- The dynamical layer: [the outside answer](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-dynamics.md), [the levers](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md) and [the review that corrected them](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md); journal §§67–70.
- [The discipline question](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/leads/queueing-discipline.md) (journal §56), [the protocol](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/process/protocol.md) and [the programme of experiments](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level/briefs/experiment-programme.md).
- [The journal](https://github.com/lmmx/mibz2/blob/61f960932ea45b709b3a9aeca35599bc6bad21b3/docs/journal/2026-11-24-algorithm-level.md).
