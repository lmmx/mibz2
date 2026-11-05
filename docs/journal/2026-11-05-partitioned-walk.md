# 2026-11-05: The radix-partitioned walk, rejected; cross-chain lockstep, parked

**The construction.** Split the T-vector into P regions sized to a share of L2, queue each lane's
next step in the queue of the region it falls in, and serve one region at a time while it is
L2-resident. Random L3 and DRAM misses would become L2 hits plus sequential queue traffic.

**Novelty.** No partitioned or region-queued walk appears in the journals or the branches.

**How it was tested.** Two rival routes, then a collision between them. All of it was
untimed: listings, callgrind and trace-driven cache simulation. Every result is in
`2026-11-05-partitioned-walk/`:
- **Route A:** an analytic design with a scratch prototype.
- **Route B:** a trace-driven simulation of master's real access order.
- **The collision:** a third simulator, confronting both routes with master's staged walk and
  with compact30's C-resolve.

**The data.**
- Held-out Wikidata blocks 1, 41, 233 and 400, n = 900,000 rows each. Every simulated walk was
  asserted to give byte-identical output.
- The simulator matches callgrind on production `room_segments` within 0.5% for D1 and LL.
- It reproduces the fused-postwalk journal's 839,609 LL misses on block 1 to +0.02%.

## Result: rejected

The premise that failed: that the 4-thread walk on CI is bound by random-miss latency or
occupancy, which reordering could turn into L2 hits at small cost. It fails twice.

**1. The binding term is per-byte work.**
- The census (`2026-11-05-binding-census.md`) finds the four-copy walk 63–70% a lane-independent
  term that tracks per-byte work; the miss term is 32–38%.
- The construction triples instructions per byte, about 31–35 against master's 10.4–10.65
  (OBSERVED: callgrind and listings). It also doubles memory operations per step, about 10
  against 5.
- It can act only on the miss term.

**2. It loses even on misses, with the starts held fixed.**
- Neither route ran the control: master's room walk, with no queues, given the same dense,
  row-sorted starts.
- Run in the collision, that control captures 74% to over 100% of partitioning's T-vector
  saving.
- Partitioning's own margin over it is 0.10 / 0.05 / 0.046 / −0.035 misses per byte (SMT pair at
  512 KB / SMT pair at 1 MB / alone at 512 KB / alone at 1 MB). Its traced queue, log and
  metadata bill is 0.21–0.29 per byte, larger in every regime.
- That is before the de-interleave of the log (at least 0.25 per byte) and the join (at least
  0.14 per byte), both INFERRED.

Its best cell, an SMT pair on a 512 KB L2, is −10.6% in traced misses. With the omitted passes
added it becomes about +30%. **Planning figure: 0 recoverable.** No prototype or CI A/B was run.

**Per walk, misses per output byte** (OBSERVED in the collision's simulator):

| regime | master (512 segments) | room walk, dense sorted starts | partitioned, P16 | partitioned, takes shuffled |
|---|--:|--:|--:|--:|
| SMT pair, 512 KB (7763) | 0.926 | 0.756 | 0.828 | 1.133 |
| SMT pair, 1 MB (9V74) | 0.860 | 0.697 | 0.792 | 1.089 |
| alone, 512 KB | 0.860 | 0.696 | 0.79–0.80 | |
| alone, 1 MB | 0.718 | 0.581 | 0.718 (P8) | |

**Compact30 does not rescue it.**
- The two compose on chain count: compact30's halved line count allows fewer regions.
- They cancel on misses: the misses partitioning targets are the ones compact30's capacity and
  start order already remove.
- They conflict on the binding resource: both add per-step core work.
- Together they are worse than either alone.

**The routes disagreed, and on what.**
- **The decisive disagreement** is where the T-vector saving comes from. Both routes credited it
  to partitioning; it was start density times take order.
- **The bookkeeping bill:** route B left out the log's de-interleave and the join. Route A's
  total was the closer one.
- **The reuse constants:** both routes' formulas were off. The collision's
  fill = Ein(λ0)/λ0, with λ0 = S·2P/((P−1)·N_lines), reproduces route A's observed cold fills
  within 0.03.

## Parked: cross-chain lockstep (dense, row-sorted starts)

**The collision's third possibility.**
- Taking many short segments in row order makes the 32 live lanes walk chains that share a
  long prefix. Chains from adjacent sorted starts walk rows within 16 of each other for a mean
  of 39.5 of their first 64 steps at 122k starts (OBSERVED).
- T-vector misses per byte fall from 0.859 to 0.584, alone at 512 KB. For an SMT pair they fall
  from 0.926 to 0.756 per walk.
- **Novelty.** Partitioning only ever had it inside its arms; neither route had it as a
  construction.

**Why it is parked, not built.**
- **What segments cost.** CI's sweeps (runs 41989592674 and 41989596336; `2026-11-05-coresident-walk/`)
  put 2048 segments at +13.5% (9V74) and +15.5% (9V45) alone against 512. That is about
  230–340 cycles per added segment (INFERRED).
- **What they would save.** At about 10^5 starts the whole miss saving is worth only 6–9
  cycles a segment.

**The revival gate.**
1. **A lean per-segment path, at most about 10 cycles a segment by instruction count and
   simulated misses (untimed).** It needs:
   - START flags set by the build;
   - deferred joins;
   - output appended to one arena in finish order;
   - the join as a second-level walk.
2. **Then one registered sweep row.** room/32/L1 with that path at 16k segments, against
   room/32/L1/512, at four copies, 5 rounds, on two Zen 4/5 runners. It passes only if:
   - the four-copy cell is faster beyond noise on at least 2 of the 4 CPUs in both runs;
   - its mean is faster;
   - no CPU is flagged slower.

**Its successor front.** Why a segment costs 230–340 cycles today is a cheaper question. It is
open as a front, with the gate above as its falsifier.

**Partitioning itself** would be reconsidered only if both hold:
- the lockstep row wins;
- a four-copy sweep shows a heavier per-byte kernel not amplified more (the miss term rising to
  about 70% or more of the four-copy walk).
