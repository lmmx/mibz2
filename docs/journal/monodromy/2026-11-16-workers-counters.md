# 2026-11-16 · MONODROMY 42 · Two models of the worker-count gain, and what each predicts for LLC misses, L2 misses and L2 write-backs per byte at 9 → 8 → 7 → 6 workers, with the pattern that refutes each. Frozen before bench-workers-low's data

Asked by the Coordinator (23:11). The bench-workers-low run records, at 12 threads, LLC load misses,
L2 misses and L2 write-backs (`l2_lines_out.non_silent`) per byte of output at 9, 8, 7 and 6 lazy
workers. Record 41's wall prediction stands; this adds the counters' shapes.

**Labels:** **M** modelled · **Conj** conjecture.

**Thresholds,** used for both models:
- *sharp:* a change of at least 10% in one step;
- *flat:* within ±5% across the steps named.

Ratios are step to step: 8/9, 7/8 and 6/7.

## Model A, the capacity cliff (record 41)

**The mechanism.** Each worker's active set is about 2.5 MB. Above W* ≈ 8 the sets overflow the
i9's 20 MB inclusive L3, so lines are evicted before reuse. Each eviction also back-invalidates the
line in a core's L2, because the L3 is inclusive. Below W* the sets fit.

| counter, per byte | 9 → 8 | 8 → 7 | 7 → 6 |
|---|---|---|---|
| LLC load misses | **sharp fall** (≤ 0.90) | a fall or flat (≤ 1.05); the crossing may straddle 8 and 7, since other data share the L3 | **flat** |
| L2 misses | a fall, smaller than the LLC's: inclusive back-invalidations go | flat | flat |
| L2 write-backs | flat or a small fall: early write-backs forced by back-invalidation go | flat | flat |

**Wall (record 41):** 7 at 1.03 [0.97, 1.10], and 6 at 1.12 [1.03, 1.22], against 8.

**What refutes A,** any of:
1. LLC misses per byte flat (within ±5%) from 9 to 8 *and* from 8 to 7: no capacity spill was
   relieved there.
2. LLC misses per byte still falling sharply from 7 to 6: the sets do not fit at 7, so the working
   set is larger than modelled, or there is no single cliff.
3. Wall falling at 7 (≤ 0.97) while LLC misses per byte are flat from 8 to 7: the gain is not
   capacity.

## Model B, bandwidth or ring contention (the rival)

**The mechanism.** The misses a worker causes are a property of its own access pattern, not of how
many workers share the L3: each private L2 (256 KB) is far smaller than a worker's 2.5 MB set
whatever W is. What W changes is the queueing on shared paths, the ring bus and the L3 slices'
bandwidth, and so each miss's latency. Fewer workers means shorter queues, faster misses and fewer
stall cycles per miss. The number of misses per byte does not change.

This fits the counted facts record 41 could not:
- cutting bytes per letter did not move LLC misses (stop15: 1.000) or L2 misses (W12: 0.994);
- cutting workers did move the wall (#481, #510).

| counter, per byte | 9 → 8 | 8 → 7 | 7 → 6 |
|---|---|---|---|
| LLC load misses | **flat** | **flat** | **flat** |
| L2 misses | flat | flat | flat |
| L2 write-backs | flat | flat | flat |

**Its wall shape (M).** The per-worker speedup from shorter queues decays smoothly as the
utilisation of the shared path falls, with no cliff. Wall keeps falling past 8 while the speedup
per worker removed stays above W/(W − 1). Then it turns, gently.
- *My central estimate:* 7 at 0.98 [0.94, 1.02], and 6 at 1.03 [0.97, 1.10], against 8.
- *The confidence:* lower than A's, because B predicts no specific turning point. The shape of
  queueing delay against load is not counted here.

**What refutes B,** any of:
1. LLC misses or L2 misses per byte changing sharply (≥ 10%) at any step: misses per byte do
   depend on W, which is A's mechanism, not B's.
2. Wall falling at 7 by more than 9 → 8 did (below 0.95) while every counter per byte stays flat.
   Queueing relief should shrink as load falls, not grow.

## Reading the run

**One step's LLC ratio separates the models:** LLC misses per byte at 8/9 (and 7/8).
- *Sharp:* A, and W* sits where they flatten.
- *Flat:* B, and record 41's turning point at 8 is not supported.

**If both are refuted** (the counters flat but the wall moving more than B allows, or the counters
moving in a shape neither predicts), neither mechanism is the one. The next measurement is then
uncore occupancy: L3 or ring queue depth, or memory-controller busy, at the same worker counts.

**What I expect, labelled Conj:** B, from the counted fact that byte cuts never moved misses per
byte. Record 41's wall numbers stay frozen as A's. B's are given above, so each model is scored on
its own numbers.
