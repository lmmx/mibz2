# The pair table built from runs: the registered CI run

9 November 2026. This is the result of the one run registered in
`docs/journal/2026-11-09-pair-from-runs.md` on #272. It was frozen at
`1c4d0f0714183f83798a0b746463ae34535fbc50`, the head carrying the C1 pin amendment.
- The Researcher said PROCEED (6731275111) and asked for the pin amendment (6731402367).
- Guardian D052 closed D051, and with it D046's STOP on the registration.

**The run:** `sweep.yml` run [42556998342](https://github.com/lmmx/mibz2/actions/runs/42556998342),
`tool: pair_runs`, 15 rounds.
- Runner: AMD EPYC 7763, 4 CPUs, a GHZ reading of 3.3091.
- The corpus is `wikidata.bz2`, with SHA-256 `82f65ace…c29915`.
- The artifact is `sweep-1c4d0f07…`.
- The single pinned copy is judged. The four contended copies are reported only.

**Validity:** the workflow's `bench/pair_runs_validity.py` gives **VALID** on the planned copy
set (`1`, `4-cpu0`…`4-cpu3`). Every copy exited 0.

## The judged readings (pinned copy, Wikidata)

| reading | value (95% interval) | registered stops | verdict |
|---|---|---|---|
| **Q1** R_build, run / row build | **0.6865** (0.6845, 0.6886) | positive ≤ 0.50 with hi < 0.60; stop lo > 0.80 | **INCONCLUSIVE** |
| **Q2** R_whole, R / P at the entry point | **1.8950** (1.8850, 1.9050) | positive ≤ 0.95, hi < 1, worst synthetic ≤ 1.05; stop lo > 1 | **NEGATIVE STOP** |
| M1 (exploratory) run − one-hop build | 3.457 nominal c/B (3.452, 3.463) | met if hi ≤ 1 | NOT MET |
| ref O / P (#34's row-built pair) | 2.3284 (2.3225, 2.3344) | — | — |
| ref R / O (run-built against row-built pair) | 0.8139 (0.8108, 0.8170) | — | — |

Both readings are what the registration's §7 expected: Q1 INCONCLUSIVE, Q2 NEGATIVE STOP.

The four contended copies agree in direction:
- Q1 is 0.78 to 0.80 in every copy.
- Q2 is 2.05 to 2.13.
- R / O is 0.87 to 0.93.

## Where the cycles go

These figures are from the pinned copy, on Wikidata. They are medians in nominal c/B, which is
wall time × the sampled clock, not counted cycles.
- **(a) The build alone,** per row of the 233 paired blocks:
  - the row build (#34) is 8.246;
  - the run build is **5.668**;
  - the one-hop build is 2.213.
- **(b) The whole entry point,** per output byte:
  - production P is **6.184**;
  - the run-built pair R is **11.705**;
  - the row-built pair O is 14.427.
- **R took 231 of 233 paired blocks** by the pair walk; the admission declined 2.

**The consumer is the larger cost, but this run doesn't isolate it.** Rows and output bytes are
almost the same count here (209.7 M each). So subtracting each build from its whole entry point
gives rough residuals. The Optimiser replayed the raw CSVs, paired by round (6731728422), in
nominal c/B per output byte:
- R − run build: **6.072 (6.024, 6.120)**;
- O − row build: **6.169 (6.140, 6.197)**;
- P − one-hop build: **3.984 (3.963, 4.004)**;
- (R − run) − (P − one): **2.088 (2.031, 2.146)**.

**These are not walk-only intervals.** They subtract separate sweeps, with different cache states,
setup and admission. The build covers 233 blocks, and R pairs only 231 of them. Synthetic text
shows how far off they can be: R declines every pair block there, yet R − run comes out at
−9.4 c/B.

So this run doesn't establish that even a free build would leave R at parity with P. The
first draft of this entry said it did, and that claim is withdrawn. What it does show is that the
pair path's residual is about 2.1 c/B above production's, by this arithmetic. A pre-built-table
consumer interval would need its own experiment.

**The run build beats the old row pair builder, never production's one-hop build.**
- It removes 31% of the row build's cost on Wikidata, and 19% of the whole pair path (R / O 0.81).
- **Against the row pair builder, it is far cheaper on run-heavy data:**
  - pattern 0.239 (0.237, 0.242);
  - one byte 0.343 (0.337, 0.350).
- **Against production's one-hop build (run / one, Optimiser replay),** it is:
  - Wikidata 2.563 (2.556, 2.569);
  - one byte 2.274 (2.185, 2.367);
  - pattern 1.009 (1.003, 1.015).

  On no corpus with paired blocks does the run pair builder beat the one-hop build.
- **It is far dearer where r/n ≈ 1:**
  - random 1.81;
  - records 3.45.

  The admission declined every block of those corpora, so their R / P stayed at 1.00 to 1.03.
  The admission works as registered.
- **Q2's synthetic screen fails** anyway: one byte 1.648, pattern 1.571.
  - On those corpora, the pair walk loses to production's room walk even with a nearly free build.
  - The one-hop build is 2.6× cheaper than the run build on Wikidata (M1).

## What this means

As registered, the run-built pair table with #34's consumer is **1.9× production at the entry
point**. Under the registration, this candidate pauses. It isn't the run-fed line closing
(PROTOCOL §11). There are two separate findings:
1. **The run-intersection build is a real saving over the row pair build** (31% on Wikidata,
   66–76% on run-heavy data). It is Q1 INCONCLUSIVE only because the registration set POSITIVE
   at 0.50. **It is not cheaper than production's one-hop build on any corpus.**
2. **The pair path's residual cost is about 2.1 c/B above production's,** by paired arithmetic
   across separate sweeps. That is not a measured consumer interval. A pair-table route needs a
   consumer measured cheaper per output byte than the room walk, on a pre-built table.

## Construction targets (PROTOCOL §11), as ranked by the Optimiser (6731728422)

1. **C31, the one-hop table filled directly from the run records.** Production's
   T[q] = L[q] | Ψ(q) << 8, filled from the emitted records with resumed cursors and no sort.
   - It drops the pair work: L reconstruction, the first-symbol directory, the composition
     metadata, and pair starts.
   - It keeps the checked counts, the prefixes, the F/L intersections and the room consumer.
   - It is judged against #269's own budget, Δ ≈ .218 + δ_direct − e − f, not against
     phase costs transplanted from this runner.
   - This is the Researcher's angle 1, direct F with resumed cursors.
2. **Matched emitter and fallback controls** (6730842547). These attack #269's one-worker loss
   without a new consumer.
3. **C37, staged pair rooms,** as a bounded consumer diagnostic on a pre-built table. To match
   the room walk on the residual proxy, it would need a reduction of more than 34.4%.
4. **C38, two-bit tags,** with or after C37, not as a standalone rescue. They remove the common
   dependent end load. The high-symbol directory load remains.

**Admission is not a target from this run.** The CSVs hold corpus totals, not per-block
timings. A selector between pair builders doesn't show any production benefit. The recoverable
saving on Wikidata is unmeasured.

Nothing here changes production.

**Revision (same day):** the consumer-cost and admission statements are corrected from the
Optimiser's replay (6731728422). The verdicts are unchanged.
