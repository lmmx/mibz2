# Scaling past four workers on a 10-core workstation (#39)

The four-thread ceiling entry (`2026-10-27-four-thread-ceiling.md`) left a residual: past
the SMT penalty, CI's runners pointed at a shared limit beyond the core. This entry
records how that was pursued on a 10-core workstation, what it found, and where it was
stopped.

Labels:
- **measured:** a figure from the workstation, run by its owner, with its file;
- **source:** a primary source, cited;
- **hypothesis:** ours, untested.

The machine: an i9-10900K (10 cores, 20 CPUs, 20 MB L3) with 4 × 32 GB DDR4-2666 (dual
rank, two DIMMs a channel, two channels; a theoretical 42.7 GB/s). Displays are on a
discrete GPU. The corpus is the benchmark's Wikidata corpus (sha256 `82f65ace…`; 11.9 MB
compressed, 210 MB decoded, 431 blocks).

## 1. The first plateau was another program's traffic

- **Believed:** throughput plateaued at about 470 MB/s from 6 threads (occupancy sweep
  and placement runs, 2026-10-27).
- **Changed by (measured, `just bench-imc`):**
  - with the browser and other applications open, the memory controller read about
    16 GiB/s at idle;
  - with them closed, idle traffic was ≈0;
  - the 6-thread decode then ran at 884–907 MB/s, against 471 before.
- **Now:** the busy-desktop runs measured the decoder sharing DRAM with someone else.
  Their paired placement comparisons stand (both arms shared the load); their absolute
  levels don't.

## 2. Quiet, throughput falls past 4–6 workers, at a constant clock

- **Measured (bench-imc, two quiet runs, before and after a reboot):**
  - 6 → 8 → 10 threads: 884–907 → 706–708 → 634–653 MB/s;
  - cycles/ref-cycles is 1.326 (4.9 GHz) in every reading at 6–10 threads;
  - instructions are constant (7.12–7.23e9), and IPC falls 1.06 → 0.50;
  - DRAM reads rise to 18.6–19.2 GiB/s at 8 threads and stay flat at 10, while read
    bytes per output byte keep rising (0.4 at 1 thread, 20 at 6, 29 at 8, 31–32 at 10);
  - total traffic is about 26–27 GiB/s, about two-thirds of the theoretical peak.
- **Now:** the fall is stalls, not extra work or a lower clock. The registered
  bandwidth-bound reading (achieved bandwidth flat as demand grows) is met from 8 to 10
  threads.
- **Not shown:** that 26–27 GiB/s is this memory system's practical limit for scattered
  mixed traffic, rather than a queueing limit before it.

## 3. The calibration that seemed to fail was the script's error

- **Seen:** the first memcpy calibration read 4.7× the copy in reads and 3.3× in writes.
- **Changed by:** `perf bench mem memcpy` without `-f` runs all three of its x86-64
  implementations, so 60 GiB was copied, not 20 (#57).
- **Now:** corrected, the readings are 1.56–1.58× reads and 1.11× writes, inside the
  expected 1–2× and about 1×. The counters are calibrated.

## 4. The occupancy instrument's slow runs: an idle gap before a run

- **Seen:** at one thread count, occupancy runs split into fast and slow, the slow
  costing about 1.5× the CPU a block with the same occupancy and no run-queue wait.
- **Falsified (measured):** heat. Package ≤ 53 °C and ≤ 72 W under turbostat; a 6-thread
  run with no heavier runs before it still varied.
- **Measured (`just bench-regimes`, 12 interleaved rounds at 6 threads):**

  | cell | slow runs | clock (cycles/ref) |
  |--|--|--|
  | plain, 0.15 s gap before the run (the instrument's) | 5 / 12 | not counted |
  | plain, no gap | 0 / 12 | not counted |
  | `sudo perf`, 0.15 s gap | 0 / 12, all about 7% slower | 1.255–1.317 |
  | `sudo perf`, no gap | 0 / 12 | 1.326 every run |

  Without the gap the two arms are alike (0.997×, 95% 0.987–1.006).
- **Now:** a release decodes without pauses, so the gap-free figures are the relevant
  ones. The instrument now runs back to back by default (`--gap` to reproduce, #59).
- **Not shown:** the mechanism of the severe plain-gap runs. They have no hardware
  counters, so a lower clock after idle and more stall cycles aren't separated.
  Recorded, not pursued.

## 5. Beside other decoders: fastest everywhere, steepest past its peak

- **Measured (`just bench-scaling`, `ab run` back to back, 6 rounds; same bytes from
  every run):**

  | threads | 2 | 4 | 6 | 8 | 10 | 20 |
  |--|--:|--:|--:|--:|--:|--:|
  | mibz2 MB/s | 705 | **1013** | 894 | 714 | 645 | 486 |
  | pbz2 MB/s | 351 | 603 | **668** | 497 | 449 | 399 |
  | lbzip2 MB/s | 161 | 298 | 379 | **416** | 416 | 411 |
  | mibz2 vs its 4 | 0.69 | 1 | 0.87 | 0.70 (0.66–0.75) | 0.63 (0.62–0.65) | 0.47 |
  | pbz2 vs its 4 | 0.58 | 1 | 1.08 | 0.84 (0.81–0.87) | 0.74 (0.73–0.75) | 0.66 |
  | lbzip2 vs its 4 | 0.55 | 1 | 1.27 | 1.38 (1.32–1.43) | 1.35 (1.28–1.42) | 1.38 |
  | mibz2 CPU vs its 2 | 1 | 1.35 | 2.28 | 3.62 | 4.67 | 11.8 |
  | pbz2 CPU vs its 2 | 1 | 1.13 | 1.53 | 2.45 | 3.30 | 7.66 |
  | lbzip2 CPU vs its 2 | 1 | 1.06 | 1.24 | 1.55 | 1.94 | 3.53 |

  At 2 threads, a worker delivers about 352 MB/s in mibz2, 176 in pbz2 and 80 in lbzip2.
- **Against the registered predictions:** neither clean outcome.
  - **The shared knee is falsified:** lbzip2 keeps scaling to 8 and stays flat to 20.
  - **The mibz2-specific outcome isn't met cleanly:** pbz2 also declines past its peak,
    and lbzip2 scales while decoding at a quarter of mibz2's rate a worker.
  - The registered rule for a mixed result applies: no conclusion beyond the intervals.
- **Now:**
  - mibz2 is not slow: it leads at every thread count, and its 20-thread figure beats
    lbzip2's best.
  - Its fall past the peak is the steepest of the three, as is its per-worker CPU
    inflation. The inflation is ordered by each decoder's rate a worker (mibz2 > pbz2 >
    lbzip2), which fits a shared resource loaded by the rate a worker makes demands
    (**hypothesis**). It is equally consistent with mibz2 touching more memory a byte.
  - Different decoders do different work a block (BWT inversion, buffering, output
    path), so the comparison doesn't identify the limiting resource.

## 6. Did CI's runners select for the knee? Versions and walk settings

- **Believed (hypothesis):** every change was accepted on 1- and 4-thread figures from
  2-core AMD runners. Changes that kept more misses in flight (32 lanes with each lane's
  next entry prefetched, #5 and #21) might have bought low-concurrency speed at the cost
  of the many-core curve.
- **Measured (`just bench-variants`, v0.1.1-v0.1.5 and this build with 16 or 8 lanes,
  prefetch off, into L2 or non-temporal, back to back, 6 rounds, the same bytes from every
  run):**
  - No variant's best beats this build's (1005 MB/s at 4 workers):
    - v0.1.5 1.006× (0.972-1.040), pf-l2 0.989×, v0.1.4 0.976×;
    - lanes16 0.955× (0.928-0.983), pf-off 0.881×, lanes8 0.853×, pf-nta 0.610×.
  - Fewer lanes and no prefetch lose by more at 8-10 workers: at 10, lanes16 0.86×, pf-off
    0.76×, lanes8 0.71× this build's throughput.
  - At 10 workers, every variant fast enough reaches one ceiling: 634-650 MB/s for this
    build, v0.1.3, v0.1.4, v0.1.5 and pf-l2. At 8 workers it's 692-717 MB/s.
  - The older releases fall less relative to their own 4 workers (v0.1.1 0.85× at 8), but
    from lower peaks.
  - v0.1.2 → v0.1.3 (#21, #24) gains at every thread count. v0.1.3 → v0.1.5 (#7, #37)
    gains 14-20% at 2-4 workers and nothing measurable at 6-10, without losing there.
- **Now:**
  - The lane and prefetch remedy is falsified. The CI-chosen setting is the best tested on
    this Intel machine too.
  - The history steepened the normalised curve by raising its low end, not by lowering its
    high end.
  - A worker needs its outstanding misses to approach the shared limit. What costs is the
    number of such workers.

## 7. Where the added cycles go: L3 hit share, then the memory system's queue

- **Measured (`bench-imc core --events mem`, process-scoped, multiplexed: the least
  coverage 43%, scaled estimates):** from 2 to 10 workers:
  - reads leaving L2 per output byte stay at about 1.45-1.5;
  - the demand-load L3 hit share falls 0.96 → 0.75 → 0.53 → 0.41 → 0.37;
  - L3 misses per KB rise 1.2 → 32 (flat from 8 to 10);
  - queue occupancy per request past L2 rises 44 → 298 cycles. That is a pressure proxy,
    not a measured latency.
  - cycles stalled on an L3 miss reach 24%, about 30% of the cycles added;
  - page-walk activity stays at 1-2.5% of cycles.
- **A model the data fit (hypothesis):** random reads over a working set larger than the
  cache, hit share ≈ C / W. The hit share times the workers' total T-vector size is 5.8-7.2
  MB from 4 to 10 workers, an effective 6-7 MB of the 20 MB L3 for the walk.
- **A mechanism it exposes (hypothesis, testable):**
  - Bytes written per output byte at the memory controller rise from 0.3 (1 worker) to
    11.5-12.8 (6-10).
  - That is close to what the decode writes: the T-vector, the last column, and the
    output's copies through segment assembly (#48 traced five).
  - These stay in cache at 1-4 workers, where #48 measured them, and become DRAM traffic on
    the shared limit past the knee.
- **#40's huge-page idea is dropped:** walk activity is far under its 10% criterion.

## 8. The last session: clean counters, stages under contention, walk gates

`just bench-session`: imc-20261028T013015Z, contend-20261028T013108Z,
variants-20261028T013629Z.

- **Measured (each pair of events counted in its own pass, 100% coverage):**
  - The multiplexed profile's trends hold within a few percent. From 4 to 10 workers:
    - the demand L3 hit share falls 0.76 → 0.38;
    - occupancy per read past L2 rises 63 → 292 cycles.
  - **Capacity:** demand reads that miss L3 rise from 10.2 to 47.1 per KB. Most of the
    rise is from 4 to 8 workers (×2.7, ×1.6, then ×1.07).
  - **Queueing past the LLC:** the occupancy per demand read that misses L3 rises from 396
    to 858 cycles (81 → 175 ns at 4.9 GHz). It keeps growing after capacity has stopped.
    From the event definitions this includes uncore queueing as well as DRAM service.
  - **Not the limit:**
    - the per-core request queue: superqueue-full cycles fall 1.5% → 0.3%;
    - page walks: 1-2% of cycles, so #40 is closed;
    - prefetch coverage: it drops only a little, with demand reads going from 4.2% to
      6.2% of reads past L2.
- **Measured (contention proxy: N pinned processes of `phases.rs`, time per byte against
  one alone):**
  - The build and walk carry 84-93% of the added time at every count. At 10 copies the
    walk is ×9.9 and the build ×9.0, while MTF + RLE2 is ×1.8 and the scanner ×1.45.
  - Segment assembly (×13.6) and RLE1 (×10.2) are the most sensitive relatively, but 3%
    and 3% of the time per byte at 10 copies. That bounds #48's possible recovery at high
    concurrency to a few percent.
  - This is stage sensitivity under concurrent independent decodes, not the parallel
    decoder's own breakdown.
- **Measured (gates: at most K workers in `invert`, the build and the walk together):**
  - With 6 permits, throughput rises 1.13× (1.09-1.17) at 8 workers and 1.21×
    (1.15-1.27) at 10. With 4 permits, 1.17× at 10. With 2-3 permits it's no better or
    worse.
  - No gate's best beats this build's best, 1019 MB/s at 4 workers: the best gated
    figure, 782 MB/s, is 0.77× of it.
  - Phase-aware scheduling is dropped as a remedy beyond the worker cap.
  - Gate3 at 10 workers holds about as many blocks in `invert` as 4 ungated workers do,
    yet gives 712 MB/s. That points at total live footprint (the waiting workers' scratch,
    the wider window) rather than active walks alone (inference). The condvar also wakes
    waiters out of block order.
- **Now:** the knee is memory contention in the T-vector phase, in two stages:
  1. L3 capacity, as live T-vectors outgrow it (4 to about 8 workers);
  2. queueing past the LLC (6 to 10).

  The lever is how many decodes are live at once.

## 9. Block size: the knee follows the live T-vectors, until the scanner binds

`just bench-blocks`: blocks-20261028T015837Z, quiet, after a reboot.

The corpus's decoded bytes were recompressed with `lbzip2 -1/-3/-6/-9`, and each file
decoded to the same 209,700,196 bytes. The runs used 2-10 workers, 6 interleaved rounds
and one run per round. Each run is short, 0.12-0.33 s, and per-thread CPU comes in 10 ms
ticks, so the thread splits are coarse.

Pre-registered (#39):
- **Capacity:** the best worker count moves up as the blocks shrink, as about
  6.5 MB / T-vector: 16, 5, 3 and 2 workers, capped by the cores.
- **Queueing past the LLC:** it stays near 4.
- **Both:** it moves less than inversely.

**Measured:**

| level | T-vector | compressed | blocks (est.) | 2 | 4 | 6 | 8 | 10 | best |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| -9 | 3.6 MB | 11.86 MB | ≈233 | 707 | **1026** | 889 | 698 | 645 | 4 |
| -6 | 2.4 MB | 12.64 MB | ≈350 | 687 | 1156 | **1327** | 1013 | 931 | 6 |
| -3 | 1.2 MB | 14.42 MB | ≈700 | 673 | 1230 | 1654 | **1729** | 1604 | 6-8 |
| -1 | 0.4 MB | 19.03 MB | ≈2,100 | 608 | 1207 | **1464** | 1457 | 1458 | 6-10 |

The values are MB/s medians. The block counts are 210 MB divided by the block size, not
counted.

- **The best worker count moves up as the blocks shrink: 4, 6, 6-8, then a plateau.**
  - The neighbours are resolved on the per-round paired ratios:
    - at -9, 4 workers beat 6 by 1.15× (1.12-1.18);
    - at -6, 6 beat 8 by 1.30× (1.21-1.39);
    - at -3, 6 against 8 is unresolved (1.03×, 0.84-1.26);
    - at -1, 6, 8 and 10 tie (within ±4%).
  - So the knee isn't fixed, and the queueing-only reading is out.
- **From -9 to -6, the knee moves inversely with the block size**, within the grid's
  2-worker resolution. The blocks are 1.5× smaller and the knee 1.5× higher.
  - Both knees sit at the same aggregate T-vector, 4 × 3.6 = 6 × 2.4 = 14.4 MB, about
    0.7 of the 20 MB L3.
  - The registered constant was wrong by about 2.2×. 6.5 MB was the fitted C of the hit
    share C/W, not the aggregate where adding a worker stops paying.
  - Taken alone, the move fits capacity better than "both".
- **The cost of each block's work, the most direct evidence.** Worker CPU-seconds for the
  whole decode, from 4 to 10 workers:

  | level | 4 workers | 10 workers | ratio |
  |---|--:|--:|--:|
  | -9 | 0.64 | 2.46 | ×3.9 |
  | -6 | 0.48 | 1.53 | ×3.2 |
  | -3 | 0.48 | 0.56 | ×1.17 |
  | -1 | 0.52 | 0.42 | flat |

  - Per-block overheads (tables, headers, setup) don't depend on how many workers run, so
    they can't produce this. The inflation is the product of concurrency and block size,
    which is what a shared-cache footprint predicts.
  - The confounds: smaller blocks also change the number of blocks, the Huffman tables,
    the ratio and the run structure, so this supports the working-set reading without
    proving it alone.
- **Early warning on few cores.** At -9, total CPU rises 1.36× from 2 to 4 workers (worker
  CPU 1.48×), while throughput still rises 1.45×. At -1 the same step costs 1.01×.
  - The CPU per byte at a few workers already separated the big-block case, before the
    knee. CI reports CPU against the base at 1 and 4 threads, but not CPU per byte against
    the arm's own lower thread count.
  - On CI's runner, 4 threads are 2 cores with SMT (`2026-10-27-four-thread-ceiling.md`).
    The SMT penalty and cache sharing are confounded there.
- **Smaller blocks cost more at low concurrency and win at high concurrency.**
  - At 2 workers -9 is fastest (707 MB/s against 608 at -1). The scanner's CPU rises with
    the compressed bytes and the block count: 0.05 s at -9, 0.12 s at -1.
  - From 4 workers up, every smaller level beats -9. The best peak is -3's: 1654-1729 MB/s,
    1.6-1.7× -9's best.
  - Smaller blocks don't lose at every worker count. The locality gain outweighs the
    per-block overheads from 4 workers up.
- **The -1 plateau (inference):**
  - workers are busy 29-46% of the wall at 6-10;
  - the serial scanner uses 0.09 s of the 0.143 s wall, and startup and the last block's
    decode take most of the rest.

  This is consistent with the scanner bound #10 describes. It isn't shown. Not reaching
  the predicted 16 workers doesn't falsify capacity: the grid stops at 10, and the scanner
  caps first.
- **Now:**
  - capacity is the first-order mechanism of the knee;
  - the knee sits at an aggregate live T-vector of about 0.7 × L3 on this machine;
  - queueing past the LLC remains what the counters show beyond it (section 8);
  - per-worker footprint is a real lever: a decoder with a -3-sized hot set on -9 data
    would have room for about 1.6× at the knee, an upper bound, since the other block
    effects come with it.

## Decisions

- **The worker count is the first lever** (lmmx/mibz2#65). A principled default to test:
  workers = min(physical cores, k × L3 / (4 B × the stream header's block size)), with k
  ≈ 0.7 fitted here (section 9). That gives 4 at -9 and 6 at -6, the measured optima.
  A scanner cap sits above it for small blocks. On this machine mibz2 at 4 workers is 2.1× its
  20-worker throughput, and no decoder variant tested comes near it at more workers.
  wikidata-pq decodes on every core by default (`WIKIDATA_BZ2_THREADS`).
  - A cap is the baseline. Autodetect is a candidate beside it, with two-sided signals:
    reader starvation against sustained backpressure, with hysteresis, adding a worker only
    as a probe kept if delivered throughput rises.
  - Either needs a release-shaped paired comparison on a many-core machine. CI's 4 vCPUs
    can't measure it.
- **Lane and prefetch tuning: closed.**
- **The decoder-side direction is fewer bytes live per worker in the T-vector phase,
  without slowing the walk:**
  - #8's fusion takes the last column's write and its re-read out of that phase;
  - output-window bounds.

  Measure each at 8-10 workers as well as 1-4. #48 stays closed: the contention proxy
  bounds what the output copies could recover to a few percent.
- **Phase-aware gating and TLB work (#40) are closed.**
- **#42 stays closed.** The C/W model prices a smaller T-vector in hit share, not in the
  cost per worker it was measured to carry.
- **#10 stays closed.**
- **Left open:**
  - the per-worker footprint lever (section 9). #42's move structure, at 0.6-1.1 B/row,
    would hold a -9 block in a -3 block's footprint or less. Its risk is unchanged:
    instructions per step. If it's ever revisited, measure it at 8-10 workers, not only
    1-4. It stays closed until then.
  - the plain-gap slow runs' mechanism.
