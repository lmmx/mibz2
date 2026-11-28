[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1363/blog/index.md>)

# Why a brief pause throttled our Intel i9, and how we calibrated memory noise

A brief pause between runs can trap a desktop processor below full boost, while parallel queue jitter dominates peak memory across six server chips.

28 November 2026 · 7 min read · [Research result](<#5-what-changes-for-every-future-benchmark>)

When you tune a parallel bzip2 decoder for 3% speedups, your measuring tape can lie louder than your code. Two recent surprises forced us to test our own benchmarks with the same rigour we apply to the decoder itself.

First, a ten-core Intel Core i9-10900K began showing sudden 18% slowdowns on identical binaries. The culprit was not the decoder, the compiler or the operating system scheduler. It was a 200-millisecond pause we had placed between runs to let the processor settle.

Second, a zero-allocation change tripped a strict peak-memory check on our continuous-integration runners. So we measured 27 zero-change benchmark runs across six server processor families. The results separated deterministic single-thread memory from multi-thread queue jitter by an order of magnitude.

**Takeaways**

- **A 200-millisecond pause triggered 100% of multi-core slow-clock runs.** Across 288 hardware-counter runs on an Intel Core i9-10900K, every multi-core slow-clock event happened after a 0.2 s settle pause. Pausing for 0 s or 2.0 s produced zero slow-clock runs out of 48 each.
- **The slow runs did identical work at 12% lower frequency.** Instructions per byte and cycles per instruction stayed completely flat. Achieved clock rate fell from 1.764 to 1.552 cycles per reference cycle. That shift explained an earlier release check that looked 5.7% to 9.5% faster purely from unequal clock-mode draws.
- **Pinning six workers to six cores cost an extra 17%.** A six-worker parallel decode spawns eight active threads including the reader and serial scanner. Leaving the run unpinned across ten cores ran 17.1% faster with identical executed instructions.
- **Identical binaries fail tight multi-thread memory gates almost every time.** We collected 5,670 paired observations across six AMD and Intel server chips. Every single zero-change run exceeded a 1.005× multi-thread peak-memory bound, and 40.7% exceeded 1.030×.
- **Single-thread peak memory is ten times quieter and catches real heap growth cleanly.** Run-to-run variation at one thread stays below 0.23%, and zero runs crossed 1.005×. Real per-worker heap additions stand out at 17 to 25 standard deviations at one thread and scale linearly with workers.

## 1\. The mystery of the bimodal desktop runs

Our ten-core Intel Core i9-10900K workstation normally repeats an 839 MB Wikidata decode within a fraction of a percent. Yet during release-scaling sweeps, the exact same binary occasionally took much longer than usual. In one check comparing versions 0.5.14 and 0.5.15, the older build landed in the slow cluster eight times out of ten at one thread. The newer build landed there only three times.

Pooled together, those unequal draws made 0.5.15 look 5.7% faster at one thread and 9.5% faster at twelve threads. Yet hardware cycle counts suggested the two builds were within one or two percent of each other. Something inside the benchmark harness was pushing runs into two distinct speed bands.

To isolate the cause without guessing, we ran a full factorial experiment on the workstation with hardware performance counters attached to every single run. We varied three settings at both one thread and six threads:

- **Core pinning:** pinned to physical cores or unpinned across all ten cores.
- **Settle pause:** zero, two tenths or two full seconds of idle sleep before each run.
- **Pair position:** first or second run inside an interleaved pair.

Twelve randomized rounds across all twelve combinations produced 144 runs per thread count, or 288 fully counter-instrumented decodes in total. Before each run launched, the harness also read the processor's current clock frequency straight from the kernel.

## 2\. How 200 milliseconds traps Intel hardware boost

The six-thread table pointed straight at one cell. Out of 144 six-thread runs, 13 crossed the slow-run threshold of 0.338 s. Not a single run did extra work. Zero runs had elevated instructions per byte, and zero had elevated cycles per instruction.

All 13 slow-clock runs landed in the pinned 0.2 s settle condition. Eleven of twelve occurred in the second position of a pair, and two of twelve occurred in the first. Every other cell had zero slow-clock runs. Neither zero-second settle nor two-second settle produced a single slow-clock run in 48 tries each.

| six-thread condition (12 rounds per cell) | median wall | pre-run clock | achieved clock ratio | instructions / byte | cycles / inst | slow-clock runs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pinned, 0.0 s settle (pos 0 / 1) | 0.298 / 0.297 s | 4.93 / 4.90 GHz | 1.763 / 1.767 | 19.52 / 19.55 | 0.335 / 0.334 | 0 / 0 |
| Pinned, 0.2 s settle, first | 0.303 s | 0.80 GHz | 1.760 | 19.53 | 0.335 | 2 of 12 |
| **Pinned, 0.2 s settle, second** | **0.351 s** | **0.80 GHz** | **1.552** | **19.51** | **0.334** | **11 of 12** |
| Pinned, 2.0 s settle (pos 0 / 1) | 0.299 / 0.299 s | 0.80 / 0.80 GHz | 1.766 / 1.764 | 19.55 / 19.54 | 0.334 / 0.334 | 0 / 0 |
| Unpinned, 0.0 s settle (pos 0 / 1) | 0.247 / 0.246 s | 5.16 / 4.90 GHz | 1.771 / 1.771 | 19.53 / 19.52 | 0.338 / 0.338 | 0 / 0 |
| Unpinned, 0.2 s settle, second | 0.260 s | 0.80 GHz | 1.749 | 19.50 | 0.337 | 0 of 12\* |
| Unpinned, 2.0 s settle (pos 0 / 1) | 0.248 / 0.249 s | 0.80 / 0.80 GHz | 1.761 / 1.749 | 19.53 / 19.53 | 0.337 / 0.338 | 0 / 0 |

*\*Unpinned 0.2 s runs showed the same dip in miniature at pair position 1. Their median slowed by 5.4%, with the slowest round hitting 0.282 s at a 1.531 clock ratio. That stayed below the pooled 0.338 s cutoff only because unpinned runs start 17% faster.*

Why does a 0.2 s pause slow the chip down when neither 0.0 s nor 2.0 s does? The pre-run clock readings explain the physics under Intel Hardware P-States with the powersave governor and balanced energy preference:

- **At 0.0 s settle**, the cores are still warm from the previous burst at 4.90 to 5.16 GHz. The next run starts straight at full turbo.
- **At 0.2 s settle**, 200 milliseconds is just long enough for idle cores to drop to the 800 MHz floor. Yet it catches the package mid-transition inside its short-window boost hysteresis right after the first run of a pair. Average achieved frequency across the decode stalls 12.1% below turbo, turning a 0.298 s decode into 0.351 s.
- **At 2.0 s settle**, the cores also sit at 800 MHz before launch. Two full seconds let the chip's short-window power and boost state reset cleanly. When work arrives, the cores snap straight back to 4.90 GHz.

Six-thread Wikidata decode time by pinning, settle pause and pair position

All 13 slow-clock runs at six threads occurred in the pinned 0.2 s settle cells, with 11 of 12 in pair position 1. Pausing 0.0 s or 2.0 s produced zero slow-clock runs out of 48 each. Leaving six workers unpinned also removes a 17.1% oversubscription tax because the parallel decoder runs eight active threads.

The same pattern held at one thread. Pinned 0.2 s settle explained 4 of 5 slow-clock runs, while pinned 2.0 s had zero in 24 runs. One thread also showed 26 slower runs at full turbo clock with identical instructions per byte. Those runs had 6.9% higher cycles per instruction, concentrated in early rounds, from ordinary fresh-process page and cache placement rather than clock drops.

Two bonus findings fell straight out of the same data. First, pinning a six-worker decode to six physical cores added a 17.1% wall-time tax even in the fast mode. Pinned runs took 0.298 s against 0.247 s unpinned. Because mibz2 pairs six workers with one bitstream scanner and one reader thread, eight active threads were time-sharing six cores.

Second, we re-analysed the earlier 0.5.14 versus 0.5.15 release bundle within each clock mode. Comparing fast runs against fast runs cut the single-thread gap from 5.7% to 1.8%. Within slow runs, the difference was 0.2%. Both figures agree with the frequency-free cycle count.

Hardware clock bins also resolve a subtle cross-thread puzzle on desktop Intel chips. Because all ten cores share one clock domain, single-core turbo sits near 5.28 GHz while four or more active cores run at 4.91 GHz. That 7.5% clock step alone explains most of why the serial scanner appears slower when eight workers run beside it. Only a small worker-coupled residual remains to test next.

## 3\. When identical binaries fail a memory gate

While we were untangling workstation clocks, a second measurement puzzle surfaced in continuous integration. A pull request allocated zero extra heap bytes and even removed an 8 KiB static table. Yet it failed a pre-registered 1.005× peak-memory check on a four-thread runner. Four-thread text showed 1.022× peak resident memory, while four-thread repeating patterns dropped to 0.974×.

Instead of relaxing one threshold by eye, we ran a strict-null calibration. We gathered 27 recent continuous-integration benchmark runs that touched zero Rust source files, meaning the head and baseline binaries were compiled from identical code. Each run executed 10 interleaved paired rounds across 7 corpora and 3 thread counts. Together they gave 5,670 paired observations across six server chips: AMD EPYC Zen 3, Zen 4 and Zen 5, plus Intel Xeon Ice Lake and Granite Rapids.

The false-alarm rates were startling. Across the fourteen multi-threaded cells, identical binaries tripped almost every tight point gate:

| candidate multi-thread peak-RSS gate | 1-thread false-positive runs | 2-thread / 4-thread false-positive runs |
| --- | ---: | ---: |
| Above 1.005× (+0.5%) | 0 of 27 (0.0%) | **27 of 27 (100.0%)** |
| Above 1.010× (+1.0%) | 0 of 27 (0.0%) | **25 of 27 (92.6%)** |
| Above 1.020× (+2.0%) | 0 of 27 (0.0%) | **17 of 27 (63.0%)** |
| Above 1.030× (+3.0%) | 0 of 27 (0.0%) | **11 of 27 (40.7%)** |
| Above 1.050× (+5.0%) | 0 of 27 (0.0%) | **3 of 27 (11.1%)** |
| Above 1.100× (+10.0%, project hard limit) | 0 of 27 (0.0%) | **0 of 27 (0.0%)** |

Even an uncorrected 95% confidence interval flagged an identical binary as using more memory on 10 of 27 runs (37.0%). Counting either direction, it false-alarmed on 13 of 27 runs (48.1%). Every single server CPU family exhibited the same multi-threaded spread, with the worst zero-change ratio reaching 1.0683×.

## 4\. Two physical regimes of peak memory

The null dataset did not merely show noise; it split cleanly into two distinct physical regimes.

At one thread, peak resident memory is nearly deterministic. Across all seven corpora and 27 runs, the run-to-run standard deviation sits between 0.13% and 0.23%. Not a single one-thread cell ever crossed 1.005×; the highest observed ratio was 1.0047×. Single-block inputs stay just as quiet at every thread count because only one block can be in flight at once.

Add multiple threads over multi-block files, and the run-to-run variation widens 7× to 20×. Run standard deviation jumps to between 0.94% and 2.59%, and single-pair spread reaches 9.1%. Thread scheduling shifts how many decoded output blocks sit queued ahead of the writer and how per-thread allocator arenas expand.

One workload made that queue effect visible to the naked eye. Across 540 four-thread runs on a repeating-pattern input, peak memory split cleanly around a 39 MiB valley. The low mode sat at 35 to 37 MiB, while the high mode sat at 42 to 44 MiB. Those extra 6 to 8 MiB are simply decoded blocks waiting briefly in the output queue when the writer lags. Whenever two identical runs landed on opposite sides of the 39 MiB valley, a single pair swung by 1.139× or 0.881×.

Strict-null peak-memory false alarms across six server processors and two noise regimes

Left: identical binaries trip every uncalibrated multi-thread point bound below 1.10×, while zero one-thread runs ever exceed 1.005×. Right: because single-thread peak memory is quiet to 0.23%, real per-worker heap additions fire at 17 to 25 standard deviations at one thread and scale linearly across 8 and 10 workers.

That two-regime split gives us a sharp diagnostic for real memory regressions. On our ten-core workstation, two earlier slab-allocating prototypes added +524 KiB per worker and +0.89 MiB per worker. At one thread, those modest additions stood out immediately at 17.4 and 24.8 standard deviations against a 0.18% pair spread. Across one, eight and ten workers, the first grew linearly from +0.58 MiB to +4.10 MiB to +5.29 MiB. The second grew from +0.89 MiB at one worker to +6.38 MiB at eight.

Queue jitter never behaves that way. It is silent at one thread and fluctuates unpredictably by two to six percent across multi-thread cells. Checking heap invariance at one thread and verifying linear growth across worker counts separates genuine footprint regressions from allocator weather.

## 5\. What changes for every future benchmark

Both investigations turned directly into permanent rules for how we measure mibz2 going forward:

- **Two-second settle pauses on physical workstations.** We replaced 0.2 s settle pauses with 2.0 s pauses so Intel hardware boost resets cleanly before every timed decode. We also avoid pinning parallel runs to fewer cores than their active thread count when measuring production scaling.
- **Mode-aware counter analysis beside pooled ratios.** Every workstation bundle now records hardware cycle and instruction counters. Those counters separate clock-frequency drops from real executed work and check whether both arms occupied the same clock mode.
- **Two-regime memory gates.** Zero-heap changes are checked tightly at one thread (within 1.010× or 0.5 MiB), where noise is below a quarter of a percent. Multi-thread continuous-integration cells use the calibrated noise bands and the project's 1.10× hard ceiling. Heap-adding designs must state their expected per-worker delta upfront and confirm linear scaling from one thread upward.

Pre-registered stop rules only protect your science if they do not cry wolf on normal hardware physics. Knowing how a 200-millisecond pause moves a desktop clock and how parallel queues move resident memory lets us keep our gates strict without chasing ghosts.

---

**Sources:**

- Intel Core i9-10900K clock-modes factorial probe and verdict: [\#1201](<https://github.com/lmmx/mibz2/pull/1201>), [verdict notebook](<https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-i9-clock-modes-verdict.md>), and archived bundle `origin/evidence` `d5658cfb` (`clock-modes/run-20261128T013907Z.tar.gz`).
- Earlier 0.5.14 vs 0.5.15 release-scaling bundle re-analysed by clock mode: `origin/evidence` `bce3914d` (`release-scaling/run-20261127T140323Z.tar.gz`).
- Multi-core turbo binning across active core counts: [cold readback notebook](<https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-explore-scanner-inflation-is-clock.md>).
- Strict-null peak-RSS calibration across 27 CI runs (6 server CPU models) and archived i9-10900K bundles: [calibration notebook](<https://github.com/lmmx/mibz2/blob/c7fa3284/docs/notebook/2026-11-28-rss-bound-calibration.md>), zero-heap bundles `77236472` ([\#1119](<https://github.com/lmmx/mibz2/pull/1119>)), `9de7b6e0` ([\#1034](<https://github.com/lmmx/mibz2/pull/1034>)), `9dacc085` ([\#1117](<https://github.com/lmmx/mibz2/pull/1117>)), and per-worker slab bundles `a6e427b1` ([\#1053](<https://github.com/lmmx/mibz2/pull/1053>)) and `e98948b6` ([\#1168](<https://github.com/lmmx/mibz2/pull/1168>)).
