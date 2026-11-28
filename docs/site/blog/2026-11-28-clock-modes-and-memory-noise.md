---
title: "Why a brief pause throttled our Intel i9, and how we calibrated memory noise"
image: assets/social/blog-2026-11-28-clock-modes-and-memory-noise.png
description: "A brief pause between runs can trap a desktop processor below full boost, while parallel queue jitter dominates peak memory across six server chips."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Why a brief pause throttled our Intel i9, and how we calibrated memory noise

A brief pause between runs can trap a desktop processor below full boost, while parallel queue jitter dominates peak memory across six server chips.
{: .post-dek }

28 November 2026 · 7 min read · [Research result](#5-what-changes-for-every-future-benchmark)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

When you tune a parallel bzip2 decoder for 3% speedups, your measuring tape can lie louder than your
code. Two recent surprises forced us to test our own benchmarks with the same rigour we apply to
the decoder itself.

First, a ten-core Intel Core i9-10900K began showing sudden 18% slowdowns on identical binaries.
The culprit was not the decoder, the compiler or the operating system scheduler. It was a
200-millisecond pause we had placed between runs to let the processor settle.

Second, a zero-allocation change tripped a strict peak-memory check on our continuous-integration
runners. So we measured 27 zero-change benchmark runs across six server processor families. The
results separated deterministic single-thread memory from multi-thread queue jitter by an order of
magnitude.

**Takeaways**

- **A 200-millisecond pause triggered 100% of multi-core slow-clock runs.** Across 288 hardware-counter
  runs on an Intel Core i9-10900K, every multi-core slow-clock event happened after a 0.2 s settle
  pause. Pausing for 0 s or 2.0 s produced zero slow-clock runs out of 48 each.
- **The slow runs did identical work at 12% lower frequency.** Instructions per byte and cycles per
  instruction stayed completely flat. Achieved clock rate fell from 1.764 to 1.552 cycles per
  reference cycle. That shift explained an earlier release check that looked 5.7% to 9.5% faster
  purely from unequal clock-mode draws.
- **Pinning six workers to six cores cost an extra 17%.** A six-worker parallel decode spawns eight
  active threads including the reader and serial scanner. Leaving the run unpinned across ten cores
  ran 17.1% faster with identical executed instructions.
- **Identical binaries fail tight multi-thread memory gates almost every time.** We collected 5,670
  paired observations across six AMD and Intel server chips. Every single zero-change run exceeded
  a 1.005× multi-thread peak-memory bound, and 40.7% exceeded 1.030×.
- **Single-thread peak memory is ten times quieter and catches real heap growth cleanly.** Run-to-run
  variation at one thread stays below 0.23%, and zero runs crossed 1.005×. Real per-worker heap
  additions stand out at 17 to 25 standard deviations at one thread and scale linearly with workers.

## 1. The mystery of the bimodal desktop runs

Our ten-core Intel Core i9-10900K workstation normally repeats an 839 MB Wikidata decode within a
fraction of a percent. Yet during release-scaling sweeps, the exact same binary occasionally took
much longer than usual. In one check comparing versions 0.5.14 and 0.5.15, the older build landed
in the slow cluster eight times out of ten at one thread. The newer build landed there only three
times.

Pooled together, those unequal draws made 0.5.15 look 5.7% faster at one thread and 9.5% faster at
twelve threads. Yet hardware cycle counts suggested the two builds were within one or two percent
of each other. Something inside the benchmark harness was pushing runs into two distinct speed
bands.

To isolate the cause without guessing, we ran a full factorial experiment on the workstation with
hardware performance counters attached to every single run. We varied three settings at both one
thread and six threads:

- **Core pinning:** pinned to physical cores or unpinned across all ten cores.
- **Settle pause:** zero, two tenths or two full seconds of idle sleep before each run.
- **Pair position:** first or second run inside an interleaved pair.

Twelve randomized rounds across all twelve combinations produced 144 runs per thread count, or 288
fully counter-instrumented decodes in total. Before each run launched, the harness also read the
processor's current clock frequency straight from the kernel.

## 2. How 200 milliseconds traps Intel hardware boost

The six-thread table pointed straight at one cell. Out of 144 six-thread runs, 13 crossed the
slow-run threshold of 0.338 s. Not a single run did extra work. Zero runs had elevated instructions
per byte, and zero had elevated cycles per instruction.

All 13 slow-clock runs landed in the pinned 0.2 s settle condition. Eleven of twelve occurred in
the second position of a pair, and two of twelve occurred in the first. Every other cell had zero
slow-clock runs. Neither zero-second settle nor two-second settle produced a single slow-clock run
in 48 tries each.

| six-thread condition (12 rounds per cell) | median wall | pre-run clock | achieved clock ratio | instructions / byte | cycles / inst | slow-clock runs |
|---|---:|---:|---:|---:|---:|---:|
| Pinned, 0.0 s settle (pos 0 / 1) | 0.298 / 0.297 s | 4.93 / 4.90 GHz | 1.763 / 1.767 | 19.52 / 19.55 | 0.335 / 0.334 | 0 / 0 |
| Pinned, 0.2 s settle, first | 0.303 s | 0.80 GHz | 1.760 | 19.53 | 0.335 | 2 of 12 |
| **Pinned, 0.2 s settle, second** | **0.351 s** | **0.80 GHz** | **1.552** | **19.51** | **0.334** | **11 of 12** |
| Pinned, 2.0 s settle (pos 0 / 1) | 0.299 / 0.299 s | 0.80 / 0.80 GHz | 1.766 / 1.764 | 19.55 / 19.54 | 0.334 / 0.334 | 0 / 0 |
| Unpinned, 0.0 s settle (pos 0 / 1) | 0.247 / 0.246 s | 5.16 / 4.90 GHz | 1.771 / 1.771 | 19.53 / 19.52 | 0.338 / 0.338 | 0 / 0 |
| Unpinned, 0.2 s settle, second | 0.260 s | 0.80 GHz | 1.749 | 19.50 | 0.337 | 0 of 12* |
| Unpinned, 2.0 s settle (pos 0 / 1) | 0.248 / 0.249 s | 0.80 / 0.80 GHz | 1.761 / 1.749 | 19.53 / 19.53 | 0.337 / 0.338 | 0 / 0 |

*\*Unpinned 0.2 s runs showed the same dip in miniature at pair position 1. Their median slowed by
5.4%, with the slowest round hitting 0.282 s at a 1.531 clock ratio. That stayed below the pooled
0.338 s cutoff only because unpinned runs start 17% faster.*

Why does a 0.2 s pause slow the chip down when neither 0.0 s nor 2.0 s does? The pre-run clock
readings explain the physics under Intel Hardware P-States with the powersave governor and balanced
energy preference:

- **At 0.0 s settle**, the cores are still warm from the previous burst at 4.90 to 5.16 GHz. The
  next run starts straight at full turbo.
- **At 0.2 s settle**, 200 milliseconds is just long enough for idle cores to drop to the 800 MHz
  floor. Yet it catches the package mid-transition inside its short-window boost hysteresis right
  after the first run of a pair. Average achieved frequency across the decode stalls 12.1% below
  turbo, turning a 0.298 s decode into 0.351 s.
- **At 2.0 s settle**, the cores also sit at 800 MHz before launch. Two full seconds let the chip's
  short-window power and boost state reset cleanly. When work arrives, the cores snap straight back
  to 4.90 GHz.

<figure class="bf-fig" id="clock-modes-grid">
<p class="bf-title">Six-thread Wikidata decode time by pinning, settle pause and pair position</p>
<!-- fig:clock-modes-grid -->
<svg viewBox="0 0 720 338" role="img" aria-label="Horizontal bar chart of six-thread Wikidata decode times across pinning, settle pause and pair position on an Intel Core i9-10900K. Pinned runs: 0.0 s settle takes 0.298 s with 0 of 24 slow; 0.2 s settle first takes 0.303 s with 2 of 12 slow; 0.2 s settle second jumps to 0.351 s with 11 of 12 slow and 12.1 percent lower clock; 2.0 s settle takes 0.299 s with 0 of 24 slow. Unpinned runs run 17.1 percent faster: 0.0 s settle takes 0.247 s, 0.2 s settle second rises to 0.260 s, and 2.0 s settle takes 0.248 s with 0 of 24 slow."><line x1="186.0" y1="30.0" x2="186.0" y2="294.0" style="stroke:var(--bf-vz-axis);stroke-width:1.5"/><line x1="276.0" y1="30.0" x2="276.0" y2="294.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="366.0" y1="30.0" x2="366.0" y2="294.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="456.0" y1="30.0" x2="456.0" y2="294.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="546.0" y1="30.0" x2="546.0" y2="294.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="489.9" y1="30.0" x2="489.9" y2="294.0" style="stroke:var(--bf-vz-2);stroke-width:1.4;stroke-dasharray:4,3"/><text class="bf-tm" x="489.9" y="21.0" text-anchor="middle" style="font-size:10px">slow cutoff 0.338 s</text><text class="bf-tm" x="186.0" y="310.0" text-anchor="middle" style="font-size:10.5px">0.00 s</text><text class="bf-tm" x="276.0" y="310.0" text-anchor="middle" style="font-size:10.5px">0.10</text><text class="bf-tm" x="366.0" y="310.0" text-anchor="middle" style="font-size:10.5px">0.20</text><text class="bf-tm" x="456.0" y="310.0" text-anchor="middle" style="font-size:10.5px">0.30</text><text class="bf-tm" x="546.0" y="310.0" text-anchor="middle" style="font-size:10.5px">0.40 s</text><text class="bf-t-strong" x="18.0" y="45.0" style="font-size:11px">Pinned to 6 cores (8 threads on 6 cores)</text><text class="bf-tm" x="178.0" y="70.0" text-anchor="end" style="font-size:10.5px">Settle 0.0 s (pos 0/1)</text><rect x="186.0" y="57.0" width="268.2" height="17.0" rx="3" style="fill:var(--bf-vz-1)"><title>Pinned 0.0 s settle: median 0.298 s, 0 of 24 slow</title></rect><text class="bf-tm" x="460.2" y="69.5" style="font-size:10.5px">0.298 s · 0/24 slow</text><text class="bf-tm" x="178.0" y="96.0" text-anchor="end" style="font-size:10.5px">Settle 0.2 s, first</text><rect x="186.0" y="83.0" width="272.7" height="17.0" rx="3" style="fill:var(--bf-vz-2);opacity:0.78"><title>Pinned 0.2 s settle, first in pair: median 0.303 s, 2 of 12 slow</title></rect><text class="bf-tm" x="464.7" y="95.5" style="font-size:10.5px">0.303 s · 2/12 slow</text><text class="bf-t-strong" x="178.0" y="122.0" text-anchor="end" style="font-size:10.5px">Settle 0.2 s, second</text><rect x="186.0" y="109.0" width="316.2" height="17.0" rx="3" style="fill:var(--bf-vz-2)"><title>Pinned 0.2 s settle, second in pair: median 0.351 s, 11 of 12 slow, -12.1% clock</title></rect><text class="bf-t-strong" x="508.2" y="121.5" style="font-size:10.5px">0.351 s · 11/12 slow (−12% clk)</text><text class="bf-tm" x="178.0" y="148.0" text-anchor="end" style="font-size:10.5px">Settle 2.0 s (pos 0/1)</text><rect x="186.0" y="135.0" width="269.1" height="17.0" rx="3" style="fill:var(--bf-vz-3)"><title>Pinned 2.0 s settle: median 0.299 s, 0 of 24 slow</title></rect><text class="bf-tm" x="461.1" y="147.5" style="font-size:10.5px">0.299 s · 0/24 slow</text><line x1="18.0" y1="168.0" x2="702.0" y2="168.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="18.0" y="188.0" style="font-size:11px">Unpinned across 10 cores (−17.1% faster fast mode)</text><text class="bf-tm" x="178.0" y="213.0" text-anchor="end" style="font-size:10.5px">Settle 0.0 s (pos 0/1)</text><rect x="186.0" y="200.0" width="222.3" height="17.0" rx="3" style="fill:var(--bf-vz-1)"><title>Unpinned 0.0 s settle: median 0.247 s, 0 of 24 slow</title></rect><text class="bf-tm" x="414.3" y="212.5" style="font-size:10.5px">0.247 s · 0/24 slow</text><text class="bf-tm" x="178.0" y="239.0" text-anchor="end" style="font-size:10.5px">Settle 0.2 s, second</text><rect x="186.0" y="226.0" width="234.4" height="17.0" rx="3" style="fill:var(--bf-vz-2);opacity:0.78"><title>Unpinned 0.2 s settle, second in pair: median 0.260 s, +5.4% HWP dip</title></rect><text class="bf-tm" x="426.4" y="238.5" style="font-size:10.5px">0.260 s · +5.4% HWP dip</text><text class="bf-tm" x="178.0" y="265.0" text-anchor="end" style="font-size:10.5px">Settle 2.0 s (pos 0/1)</text><rect x="186.0" y="252.0" width="223.2" height="17.0" rx="3" style="fill:var(--bf-vz-3)"><title>Unpinned 2.0 s settle: median 0.248 s, 0 of 24 slow</title></rect><text class="bf-tm" x="415.2" y="264.5" style="font-size:10.5px">0.248 s · 0/24 slow</text><text class="bf-tm" x="702.0" y="330.0" text-anchor="end" style="font-size:10.5px">Intel Core i9-10900K, 839 MB Wikidata, 144 perf-measured 6T runs (lower is faster)</text></svg>
<figcaption>All 13 slow-clock runs at six threads occurred in the pinned 0.2 s settle cells,
with 11 of 12 in pair position 1. Pausing 0.0 s or 2.0 s produced zero slow-clock runs out of 48
each. Leaving six workers unpinned also removes a 17.1% oversubscription tax because the parallel
decoder runs eight active threads.</figcaption>
</figure>

The same pattern held at one thread. Pinned 0.2 s settle explained 4 of 5 slow-clock runs, while
pinned 2.0 s had zero in 24 runs. One thread also showed 26 slower runs at full turbo clock with
identical instructions per byte. Those runs had 6.9% higher cycles per instruction, concentrated in
early rounds, from ordinary fresh-process page and cache placement rather than clock drops.

Two bonus findings fell straight out of the same data. First, pinning a six-worker decode to six
physical cores added a 17.1% wall-time tax even in the fast mode. Pinned runs took 0.298 s against
0.247 s unpinned. Because mibz2 pairs six workers with one bitstream scanner and one reader thread,
eight active threads were time-sharing six cores.

Second, we re-analysed the earlier 0.5.14 versus 0.5.15 release bundle within each clock mode.
Comparing fast runs against fast runs cut the single-thread gap from 5.7% to 1.8%. Within slow
runs, the difference was 0.2%. Both figures agree with the frequency-free cycle count.

Hardware clock bins also resolve a subtle cross-thread puzzle on desktop Intel chips. Because all
ten cores share one clock domain, single-core turbo sits near 5.28 GHz while four or more active
cores run at 4.91 GHz. That 7.5% clock step alone explains most of why the serial scanner appears
slower when eight workers run beside it. Only a small worker-coupled residual remains to test next.

## 3. When identical binaries fail a memory gate

While we were untangling workstation clocks, a second measurement puzzle surfaced in continuous
integration. A pull request allocated zero extra heap bytes and even removed an 8 KiB static table.
Yet it failed a pre-registered 1.005× peak-memory check on a four-thread runner. Four-thread text
showed 1.022× peak resident memory, while four-thread repeating patterns dropped to 0.974×.

Instead of relaxing one threshold by eye, we ran a strict-null calibration. We gathered 27 recent
continuous-integration benchmark runs that touched zero Rust source files, meaning the head and
baseline binaries were compiled from identical code. Each run executed 10 interleaved paired rounds
across 7 corpora and 3 thread counts. Together they gave 5,670 paired observations across six
server chips: AMD EPYC Zen 3, Zen 4 and Zen 5, plus Intel Xeon Ice Lake and Granite Rapids.

The false-alarm rates were startling. Across the fourteen multi-threaded cells, identical binaries
tripped almost every tight point gate:

| candidate multi-thread peak-RSS gate | 1-thread false-positive runs | 2-thread / 4-thread false-positive runs |
|---|---:|---:|
| Above 1.005× (+0.5%) | 0 of 27 (0.0%) | **27 of 27 (100.0%)** |
| Above 1.010× (+1.0%) | 0 of 27 (0.0%) | **25 of 27 (92.6%)** |
| Above 1.020× (+2.0%) | 0 of 27 (0.0%) | **17 of 27 (63.0%)** |
| Above 1.030× (+3.0%) | 0 of 27 (0.0%) | **11 of 27 (40.7%)** |
| Above 1.050× (+5.0%) | 0 of 27 (0.0%) | **3 of 27 (11.1%)** |
| Above 1.100× (+10.0%, project hard limit) | 0 of 27 (0.0%) | **0 of 27 (0.0%)** |

Even an uncorrected 95% confidence interval flagged an identical binary as using more memory on 10
of 27 runs (37.0%). Counting either direction, it false-alarmed on 13 of 27 runs (48.1%). Every
single server CPU family exhibited the same multi-threaded spread, with the worst zero-change ratio
reaching 1.0683×.

## 4. Two physical regimes of peak memory

The null dataset did not merely show noise; it split cleanly into two distinct physical regimes.

At one thread, peak resident memory is nearly deterministic. Across all seven corpora and 27 runs,
the run-to-run standard deviation sits between 0.13% and 0.23%. Not a single one-thread cell ever
crossed 1.005×; the highest observed ratio was 1.0047×. Single-block inputs stay just as quiet at
every thread count because only one block can be in flight at once.

Add multiple threads over multi-block files, and the run-to-run variation widens 7× to 20×. Run
standard deviation jumps to between 0.94% and 2.59%, and single-pair spread reaches 9.1%. Thread
scheduling shifts how many decoded output blocks sit queued ahead of the writer and how per-thread
allocator arenas expand.

One workload made that queue effect visible to the naked eye. Across 540 four-thread runs on a
repeating-pattern input, peak memory split cleanly around a 39 MiB valley. The low mode sat at 35
to 37 MiB, while the high mode sat at 42 to 44 MiB. Those extra 6 to 8 MiB are simply decoded
blocks waiting briefly in the output queue when the writer lags. Whenever two identical runs landed
on opposite sides of the 39 MiB valley, a single pair swung by 1.139× or 0.881×.

<figure class="bf-fig" id="rss-noise-calibration">
<p class="bf-title">Strict-null peak-memory false alarms across six server processors and two noise regimes</p>
<!-- fig:rss-noise-calibration -->
<svg viewBox="0 0 720 314" role="img" aria-label="Two-panel chart from 27 strict-null CI runs across six server CPUs and workstation heap tests. Left panel shows multi-thread false-positive rate across candidate peak-memory gates: 100 percent above 1.005x, 92.6 percent above 1.010x, 63.0 percent above 1.020x, 40.7 percent above 1.030x, 11.1 percent above 1.050x, and 0 percent above 1.100x. Right panel contrasts noise against real per-worker heap additions: single-thread null run standard deviation is 0.23 percent (max +0.47 percent), multi-thread null run standard deviation is 2.59 percent (worst run +6.83 percent), while real 524 KiB and 0.89 MiB per-worker slabs stand out at 17.4 sigma and 24.8 sigma at one thread and grow linearly to +4.88 percent and +5.91 percent at 8 to 10 workers."><rect x="14" y="14" width="336" height="264" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="28" y="36" style="font-size:11.5px">Multi-thread false alarms (27 null runs)</text><line x1="94" y1="50" x2="94" y2="238" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><line x1="179" y1="50" x2="179" y2="238" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="264" y1="50" x2="264" y2="238" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="94" y="254" text-anchor="middle" style="font-size:10px">0%</text><text class="bf-tm" x="179" y="254" text-anchor="middle" style="font-size:10px">50%</text><text class="bf-tm" x="264" y="254" text-anchor="middle" style="font-size:10px">100%</text><text class="bf-tm" x="86" y="70" text-anchor="end" style="font-size:10.5px">&gt; 1.005×</text><rect x="94" y="57" width="170.0" height="17" rx="3" style="fill:var(--bf-vz-2)"><title>Above 1.005x: 27 of 27 null runs (100.0%)</title></rect><text class="bf-t-strong" x="270" y="69.5" style="font-size:10.5px">100%</text><text class="bf-tm" x="86" y="100" text-anchor="end" style="font-size:10.5px">&gt; 1.010×</text><rect x="94" y="87" width="157.4" height="17" rx="3" style="fill:var(--bf-vz-2)"><title>Above 1.010x: 25 of 27 null runs (92.6%)</title></rect><text class="bf-tm" x="257.4" y="99.5" style="font-size:10.5px">92.6%</text><text class="bf-tm" x="86" y="130" text-anchor="end" style="font-size:10.5px">&gt; 1.020×</text><rect x="94" y="117" width="107.1" height="17" rx="3" style="fill:var(--bf-vz-2);opacity:0.85"><title>Above 1.020x: 17 of 27 null runs (63.0%)</title></rect><text class="bf-tm" x="207.1" y="129.5" style="font-size:10.5px">63.0%</text><text class="bf-tm" x="86" y="160" text-anchor="end" style="font-size:10.5px">&gt; 1.030×</text><rect x="94" y="147" width="69.2" height="17" rx="3" style="fill:var(--bf-vz-2);opacity:0.75"><title>Above 1.030x: 11 of 27 null runs (40.7%)</title></rect><text class="bf-tm" x="169.2" y="159.5" style="font-size:10.5px">40.7%</text><text class="bf-tm" x="86" y="190" text-anchor="end" style="font-size:10.5px">&gt; 1.050×</text><rect x="94" y="177" width="18.9" height="17" rx="3" style="fill:var(--bf-vz-1)"><title>Above 1.050x: 3 of 27 null runs (11.1%)</title></rect><text class="bf-tm" x="118.9" y="189.5" style="font-size:10.5px">11.1%</text><text class="bf-tm" x="86" y="220" text-anchor="end" style="font-size:10.5px">&gt; 1.100×</text><rect x="94" y="207" width="2.0" height="17" rx="1" style="fill:var(--bf-vz-3)"><title>Above 1.100x: 0 of 27 null runs (0.0%)</title></rect><text class="bf-t-strong" x="102" y="219.5" style="font-size:10.5px">0.0% (0/27)</text><rect x="366" y="14" width="340" height="264" rx="6" style="fill:var(--bf-track);stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-t-strong" x="380" y="36" style="font-size:11.5px">Null noise vs real per-worker heap growth</text><line x1="484" y1="50" x2="484" y2="238" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><line x1="544" y1="50" x2="544" y2="238" style="stroke:var(--bf-vz-grid);stroke-width:1"/><line x1="604" y1="50" x2="604" y2="238" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="484" y="254" text-anchor="middle" style="font-size:10px">0%</text><text class="bf-tm" x="544" y="254" text-anchor="middle" style="font-size:10px">+4%</text><text class="bf-tm" x="604" y="254" text-anchor="middle" style="font-size:10px">+8%</text><text class="bf-tm" x="476" y="68" text-anchor="end" style="font-size:10px">1T null max</text><rect x="484" y="56" width="7.1" height="16" rx="3" style="fill:var(--bf-vz-3)"><title>1T null maximum across 27 runs: +0.47% (run SD 0.13-0.23%)</title></rect><text class="bf-tm" x="497" y="68" style="font-size:10px">+0.47% (SD 0.23%)</text><text class="bf-tm" x="476" y="100" text-anchor="end" style="font-size:10px">2T/4T null SD</text><rect x="484" y="88" width="38.9" height="16" rx="3" style="fill:var(--bf-vz-2);opacity:0.8"><title>Multi-thread null run SD: up to 2.59% (7-20x wider)</title></rect><text class="bf-tm" x="529" y="100" style="font-size:10px">2.59% (7–20× wider)</text><text class="bf-tm" x="476" y="132" text-anchor="end" style="font-size:10px">2T/4T null max</text><rect x="484" y="120" width="102.5" height="16" rx="3" style="fill:var(--bf-vz-2)"><title>Multi-thread null worst run: +6.83% with zero code change</title></rect><text class="bf-t-strong" x="592.5" y="132" style="font-size:10px">+6.83% (null)</text><line x1="380" y1="152" x2="692" y2="152" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="476" y="176" text-anchor="end" style="font-size:10px">+524 KiB/w (1T)</text><rect x="484" y="164" width="11.6" height="16" rx="3" style="fill:var(--bf-vz-1)"><title>Real +524 KiB/worker slab at 1T: +0.77% (+0.58 MiB, t = +17.4)</title></rect><text class="bf-t-strong" x="501.6" y="176" style="font-size:10px">+0.77% (+17.4σ)</text><text class="bf-tm" x="476" y="206" text-anchor="end" style="font-size:10px">+524 KiB/w (10T)</text><rect x="484" y="194" width="73.2" height="16" rx="3" style="fill:var(--bf-vz-1)"><title>Real +524 KiB/worker slab at 10T: +4.88% (+5.29 MiB, t = +24.5)</title></rect><text class="bf-tm" x="563.2" y="206" style="font-size:10px">+4.88% (+5.29 MiB)</text><text class="bf-tm" x="476" y="234" text-anchor="end" style="font-size:10px">+0.89 MiB/w (8T)</text><rect x="484" y="222" width="88.7" height="16" rx="3" style="fill:var(--bf-vz-4)"><title>Real +0.89 MiB/worker slabs at 8T: +5.91% (+6.38 MiB, t = +22.7; +24.8 sigma at 1T)</title></rect><text class="bf-tm" x="578.7" y="234" style="font-size:10px">+5.91% (+24.8σ 1T)</text><text class="bf-tm" x="706" y="302" text-anchor="end" style="font-size:10.5px">Left: 5,670 paired CI null observations (6 CPU models). Right: CI null vs i9-10900K real slabs.</text></svg>
<figcaption>Left: identical binaries trip every uncalibrated multi-thread point bound below
1.10×, while zero one-thread runs ever exceed 1.005×. Right: because single-thread peak memory is
quiet to 0.23%, real per-worker heap additions fire at 17 to 25 standard deviations at one thread
and scale linearly across 8 and 10 workers.</figcaption>
</figure>

That two-regime split gives us a sharp diagnostic for real memory regressions. On our ten-core
workstation, two earlier slab-allocating prototypes added +524 KiB per worker and +0.89 MiB per
worker. At one thread, those modest additions stood out immediately at 17.4 and 24.8 standard
deviations against a 0.18% pair spread. Across one, eight and ten workers, the first grew linearly
from +0.58 MiB to +4.10 MiB to +5.29 MiB. The second grew from +0.89 MiB at one worker to +6.38 MiB
at eight.

Queue jitter never behaves that way. It is silent at one thread and fluctuates unpredictably by two
to six percent across multi-thread cells. Checking heap invariance at one thread and verifying
linear growth across worker counts separates genuine footprint regressions from allocator weather.

## 5. What changes for every future benchmark

Both investigations turned directly into permanent rules for how we measure mibz2 going forward:

- **Two-second settle pauses on physical workstations.** We replaced 0.2 s settle pauses with 2.0 s
  pauses so Intel hardware boost resets cleanly before every timed decode. We also avoid pinning
  parallel runs to fewer cores than their active thread count when measuring production scaling.
- **Mode-aware counter analysis beside pooled ratios.** Every workstation bundle now records
  hardware cycle and instruction counters. Those counters separate clock-frequency drops from real
  executed work and check whether both arms occupied the same clock mode.
- **Two-regime memory gates.** Zero-heap changes are checked tightly at one thread (within 1.010×
  or 0.5 MiB), where noise is below a quarter of a percent. Multi-thread continuous-integration
  cells use the calibrated noise bands and the project's 1.10× hard ceiling. Heap-adding designs
  must state their expected per-worker delta upfront and confirm linear scaling from one thread
  upward.

Pre-registered stop rules only protect your science if they do not cry wolf on normal hardware
physics. Knowing how a 200-millisecond pause moves a desktop clock and how parallel queues move
resident memory lets us keep our gates strict without chasing ghosts.

---

**Sources:**

- Intel Core i9-10900K clock-modes factorial probe and verdict: [#1201](https://github.com/lmmx/mibz2/pull/1201), [verdict notebook](https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-i9-clock-modes-verdict.md), and archived bundle `origin/evidence` `d5658cfb` (`clock-modes/run-20261128T013907Z.tar.gz`).
- Earlier 0.5.14 vs 0.5.15 release-scaling bundle re-analysed by clock mode: `origin/evidence` `bce3914d` (`release-scaling/run-20261127T140323Z.tar.gz`).
- Multi-core turbo binning across active core counts: [cold readback notebook](https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-explore-scanner-inflation-is-clock.md).
- Strict-null peak-RSS calibration across 27 CI runs (6 server CPU models) and archived i9-10900K bundles: [calibration notebook](https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-rss-bound-calibration.md), zero-heap bundles `77236472` ([#1119](https://github.com/lmmx/mibz2/pull/1119)), `9de7b6e0` ([#1034](https://github.com/lmmx/mibz2/pull/1034)), `9dacc085` ([#1117](https://github.com/lmmx/mibz2/pull/1117)), and per-worker slab bundles `a6e427b1` ([#1053](https://github.com/lmmx/mibz2/pull/1053)) and `e98948b6` ([#1168](https://github.com/lmmx/mibz2/pull/1168)).
