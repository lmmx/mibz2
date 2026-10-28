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

## Decisions

- **The worker count is the first lever.** On this machine mibz2 at 4 workers is 2.1× its
  20-worker throughput, and no decoder variant tested comes near it at more workers.
  wikidata-pq decodes on every core by default (`WIKIDATA_BZ2_THREADS`).
  - A cap is the baseline. Autodetect is a candidate beside it, with two-sided signals:
    reader starvation against sustained backpressure, with hysteresis, adding a worker only
    as a probe kept if delivered throughput rises.
  - Either needs a release-shaped paired comparison on a many-core machine. CI's 4 vCPUs
    can't measure it.
- **Lane and prefetch tuning: closed.**
- **The decoder-side direction is fewer DRAM bytes per output byte without slowing the
  walk:** #8 (fusion), #48 (copies, reopenable: its verdict was measured where the copies
  stay in cache), output-window bounds. Measure each at 8-10 workers as well as 1-4.
- **#42 stays closed.** The C/W model prices a smaller T-vector in hit share, not in the
  cost per worker it was measured to carry.
- **#10 stays closed.**
- **Left open:**
  - capacity against compulsory-traffic queueing, which the same content recompressed at
    smaller block sizes would separate (release dumps are -9, so that test explains and
    doesn't remedy);
  - the plain-gap slow runs' mechanism.
