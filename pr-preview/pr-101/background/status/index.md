# Status

*As of 29 October 2026.* This page is a snapshot of where the research stands. The evidence is in the [journals](<https://lmmx.github.io/mibz2/pr-preview/pr-101/background/journals/index.md>) and the pull requests linked below.

**In short:** - A faster way to run the inverse BWT on a many-core machine has been found, confirmed and explained. - **It is not in the released decoder yet.** mibz2 0.1.5 decodes as it did, and every figure here is the inversion stage alone, not whole decompression.

## The starting point: a knee at four workers

- mibz2's decoder gives each worker its own block. The inverse BWT then walks that block's table: about 3.6 MB for a full 900 kB block, one random load per output byte.
- On a 10-core i9-10900K, with a 20 MB shared L3, throughput peaks at **4 workers** and falls beyond them. Eight tables don't fit in the L3, and the walk's loads go to DRAM.

## The finding: cooperative inversion

Several cores share one block's table: the table is built together and walked together, each core taking segments of the walk from a shared counter. An arm is written *N*x*k*: *N* blocks in flight, *k* cores on each. Production's independent workers are *p4*, *p8* and so on.

Measured on the i9 in the research harness ([PR \#73](<https://github.com/lmmx/mibz2/pull/73>)):

| arm | what runs | inversion MB/s | against p4 (paired, 95% interval) |
| --- | --- | ---: | ---: |
| p4 | production, 4 independent workers | 1,465 | 1 |
| p8 | production, 8 independent workers | 662 | 0.47 |
| 4x2 | 4 blocks, 2 cores each | 1,808 | **1.230** (1.217–1.243) |
| 3x3 | 3 blocks, 3 cores each (9 cores) | 1,942 | **1.323** (1.312–1.335) |
| s3x3 | libsais 2.12.1, 3 blocks on 3 threads each | 2,355 | 1.59 |

- **The confirmation was registered before the runs.**
- 4x2 and 3x3 were each run alone against p4, 20 paired rounds each.
- Each beat p4 in all 20 rounds, and every block's output was checked.
- 4x2 reproduced its discovery figure within 0.3%.
- 3x3 came in 2.3% under its discovery figure, as picking the best of 19 arms would predict.
- **[libsais](<https://github.com/IlyaGrebnov/libsais>)'s inverse BWT is still about 20% ahead** at 3x3 (1.195× ours, paired). It gains from cooperation by the same factor as ours, about 1.5×. The gap between them is the kernel.

## Why it works: two mechanisms, measured

A hardware-counter study on the same machine ran each arm alone under `perf stat`, with counters that include the walk's software prefetches. It was 588 runs, none multiplexed ([PR \#84](<https://github.com/lmmx/mibz2/pull/84>)).

| arm | L3 hit share of data reads past L2 | reads in flight, machine-wide | wait per read |
| --- | ---: | ---: | ---: |
| p4 | 94.5% | 28.1 | 67 cycles |
| p8 | 65.7% | 70.0 | 366 cycles |
| 4x2 | 94.9% | 38.8 | 64 cycles |
| 3x3 | 96.7% | 41.9 | 66 cycles |

- **Residency.**
- With eight cores either way, four shared tables (4x2) stay in the L3 where eight independent ones (p8) don't. The hit share rises from 66% to 95%, and each read waits 64 cycles instead of 366.
- **Overlap.**
- With four tables either way, the second core on each (4x2 against p4) leaves the hit share and the wait unchanged.
- It puts 1.38× as many reads in flight.
- **3x3 has both:** a higher hit share than p4, a third fewer reads missing the L3 per byte, and 1.49× the reads in flight.
- **The counters account for the timings.** Little's law on them (reads in flight over the wait, against reads per byte) reproduces each arm's measured throughput to within 0–11%.

## What it points at

The cooperative kernel retires more instructions per output byte than production's or libsais's:

| kernel | instructions per output byte |
| --- | ---: |
| production (p4) | 23.8 |
| cooperative, one core a block (4x1) | 31.8 |
| cooperative, 3x3 | 35.3 |
| libsais, one core a block (s4x1) | 21.4 |
| libsais, 3x3 | 23.2 |

- **Its 4x1 penalty is instructions, not memory.**
- The cooperative kernel on one core a block (4x1) runs at 0.87× production.
- Its reads and waits are within 10% of production's; it executes a third more instructions.
- **libsais's lead is fewer reads.**
- It makes 0.86–0.89 reads past L2 per byte, against our 1.40–1.55. That is one dependent load for two output bytes.
- It does so within about 23 instructions a byte.
- mibz2's own two-byte walk (\#6) wins only past the knee. At its best it reaches 0.86× the one-byte walk's best.

## What's next

| order | work | what it is for |
| ---: | --- | --- |
| 1 | [\#77](<https://github.com/lmmx/mibz2/issues/77>): the cooperative kernel's instructions, attributed to code by `perf annotate` ([PR \#85](<https://github.com/lmmx/mibz2/pull/85>)) before any change | production's instructions a byte, and production's speed at one core a block |
| 2 | [\#78](<https://github.com/lmmx/mibz2/issues/78>), [\#81](<https://github.com/lmmx/mibz2/issues/81>): the build's recount and extra pass | a cheaper shared build: 43% of 3x3's time |
| 3 | [\#79](<https://github.com/lmmx/mibz2/issues/79>): a smaller table | more teams resident at once (a scheduling gain, not one for 3x3 itself) |
| later | a two-byte walk within production's instruction budget | libsais's fewer reads a byte |

**Still open in the counter record:** - 3 short checks of the raw L3 event's accounting, with their criteria fixed in advance. It counts a steady 23–39% more requests than `offcore_requests.all_data_rd`, and the likely cause is prefetches into L3. - The study's conclusions rest on the arm-to-arm directions, which a second, independent pair of counters reproduces.

**Before any of this reaches a release:** - the cooperative build and walk must pass the decoder's correctness checks: the segment oracle, fuzzing and Miri on the shared table; - the whole decoder must be A/B'd at 4–10 cores, where an inversion-stage gain counts only as far as it survives; - other CPUs must be measured too. CI's 4-core runners can't show a gain that only exists past four workers, so the workstation measures it.
