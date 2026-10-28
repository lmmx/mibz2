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

## Decisions

- **The thread count matters more than any decoder change seen here.** On this machine
  mibz2 at 4 workers is 2.1× its 20-worker throughput, and wikidata-pq decodes on every
  core by default (`WIKIDATA_BZ2_THREADS`).
  - A cap, or an adaptive worker count that finds the knee at run time, is the
    candidate.
  - CI's 4-vCPU runners can't measure a cap of 4 or more, so it needs a release-shaped
    paired comparison on a many-core machine, with the decoder sharing it with the
    pipeline's other stages. Not decided here.
- **#42 stays closed.** A smaller per-worker footprint is a plausible lever on the
  steeper fall, but nothing here shows it would pay for what it costs a worker. It
  reopens only on a paired experiment.
- **#10 stays closed.**
- **The investigation stops here.** The stopping rule was one targeted experiment after
  the gap result, and this was it. Left open:
  - the limiting resource (DRAM bandwidth against a queueing or L3 limit);
  - the plain-gap slow runs' mechanism.
