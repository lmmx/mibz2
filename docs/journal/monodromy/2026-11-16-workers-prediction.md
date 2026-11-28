# 2026-11-16 · MONODROMY 41 · A prediction, before the data, for the 8 / 7 / 6 worker run: W* = 8 (interval 7 to 9). Against 8 workers, 7 is 1.03 [0.97, 1.10] and 6 is 1.12 [1.03, 1.22], at each of 12, 16 and 20 threads. One working-set model; the counter that would refute it

Asked by the Coordinator (23:09). It is to be scored under §11a's calibration against the
Pathfinder's run of 8 / 7 / 6 lazy workers at 4 a worker, with 8 workers and 32 in flight as the
reference. Written before any of that run's data.

**Labels:** **C** counted, with its source · **M** modelled · **Conj** conjecture.

## 1. What is known

- **Removing a worker past the cores paid twice (C).**

  | step | wall | per-worker speedup implied |
  |---|--:|--:|
  | 10 → 9 workers (#481) | 0.956 | (10/9)/0.956 = **1.16** |
  | 9 → 8 workers (#510; 0.951 / 0.955 / 0.951 at 12 / 16 / 20 threads) | about 0.952 | (9/8)/0.952 = **1.18** |

  CPU fell 13% and peak memory 10%: it is contention, not idle cores.
- **The workers bind past the cores (C):** 91–92% busy at 10 workers, with the scanner at 61–69%
  (LOG 6906620039).
- **But a byte cut did not move LLC misses (C):** stop15 removed 5 B a letter and LLC load misses
  stayed at 1.000 at 10 threads (LOG 6925144719). W12's 2.29 B left L2 misses at 0.994 (the
  Coordinator's 22:38 message).

## 2. The model (M)

**A worker's active working set, per Wikidata block.**
- *Lazy letters' state:* about 29.5 B a letter: words 15.6, top slots 6.36, entries 4.0, bottom
  slots 2.54 (in the output buffer), bytes 1.0. The source is LOG 6907104251 and the records.
- *The block:* about 56.7 k letters (r/n ≈ 0.063, n = 900 k), so about 1.67 MB.
- *The output buffer:* 0.9 MB, less the 0.14 MB of bottom slots already counted.
- *The run records:* about 0.1 MB.

So about **2.5 MB a worker** (2.4 MB with W12's 2.29 B a letter). Queued blocks' symbols are
streamed once, so they are not counted as active.

**Against the i9's 20 MB shared L3,** W workers' active sets fill it at W ≈ 20 / 2.5 = **8**. The
scanner, the consumer and the system share it too, so the effective crossing is a little lower.

**The mechanism assumed: a capacity cliff (Conj).**
- *Above about 8 workers:* the sets spill the L3, and each worker removed buys a large per-worker
  speedup, the 16–18% measured.
- *Below about 8:* the sets fit, little contention is left to relieve, and removing a worker
  costs its share of throughput, W/(W − 1).

## 3. The prediction (M), against 8 workers, the same at 12, 16 and 20 threads

The thread count past the cores does not change the number of workers, and #510 read nearly the
same ratio at all three.

| arm | predicted wall | interval I stand behind | reasoning |
|---|--:|---|---|
| **W\*** | **8** | 7 to 9 | the capacity crossing at about 20 / 2.5 MB |
| 7 workers | **1.03** | [0.97, 1.10] | 8/7 = 1.143 of throughput lost, against a per-worker speedup of 1.04 to 1.18. The upper end is #510's step repeating; the lower end is no contention left. |
| 6 workers | **1.12** | [1.03, 1.22] | 8/6 = 1.333 lost, against a compounded speedup of 1.09 to 1.30 |

**What would make me wrong.**
- *The trend continuing* (7 at about 0.95 or below) means the contention is not a capacity cliff
  near 8. It is a resource whose contention falls steadily with each worker removed: L3 or ring
  bandwidth, or an inclusive L3 invalidating the L2s.
- *That fits the counted fact that a byte cut did not move LLC misses,* and capacity does not fit
  it. I rank the capacity model first only because it is the one that predicts a turning point.

## 4. Can the model predict the turning point? Partly, and what would let it

**The capacity model predicts W\* ≈ 8 from the working set.** But it is in tension with LLC misses
not moving under byte cuts. If capacity were the mechanism, cutting a worker's set by 5 B a letter
(about 0.28 MB a block, 3% of 10 workers' sets) should have moved LLC misses a little.

**The measurement that decides it.** LLC load misses, L2 misses and L2 write-backs
(`l2_lines_out.non_silent`), each a byte, at 10, 9 and 8 workers, in one counters run of the
shipped decoder.
- *LLC misses a byte falling steeply from 9 to 8* means capacity: the turning point is about
  20 MB divided by the working set, so W* ≈ 8 and smaller sets push it up.
- *LLC misses flat while wall falls* means bandwidth or ring contention, and no working-set model
  predicts the turning point. Only the 8 / 7 / 6 run itself, or a bandwidth counter (uncore L3 or
  memory-controller occupancy), can locate it.
