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

**The consumer is the loss, not the build.** Rows and output bytes are almost the same count here
(209.7 M each). So subtracting each build from its whole entry point gives a rough consumer cost:
- R − run build ≈ 11.7 − 5.7 ≈ **6.0 c/B**;
- O − row build ≈ 14.4 − 8.2 ≈ **6.2 c/B**.

This is arithmetic across two medians, not a measured interval. It says that #34's pair walk alone
costs about what production's whole entry point does, build included. **A free build would still
leave R near parity with P**, so no builder can carry Q2 on this consumer. That is the transferred
prediction from #34, now measured on the current baseline.

**The run build itself does what it was meant to.**
- It removes 31% of the row build's cost on Wikidata, and 19% of the whole pair path (R / O 0.81).
- **It is far cheaper on run-heavy data:**
  - pattern 0.239 (0.237, 0.242);
  - one byte 0.343 (0.337, 0.350).
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
1. **The run-intersection build is a real saving over the row build** (31% on Wikidata, 66–76%
   on run-heavy data). It is only Q1 INCONCLUSIVE because the registration set POSITIVE at 0.50.
2. **The pair consumer costs about 6 c/B on its own,** about all of production's entry point. Any
   pair-table route needs a consumer that is cheaper per output byte than the room walk. A
   cheaper builder alone can't do it.

## Construction targets (PROTOCOL §11)

1. **The consumer first.** A pair consumer at or below the room walk's per-byte cost. Candidates:
   - the Optimiser's C37 staged pair rooms;
   - the C38 two-bit tags (bits 28–29), which drop the first-byte lookup.

   Gate: show the consumer's per-byte cost under production's walk on a pre-built table, before
   any builder is joined to it.
2. **The run build for the single-hop table,** not the pair table. Wikidata's run build is 5.7 c/B
   of rows. The question is whether the same intersection walk can fill production's one-hop
   T-vector more cheaply than production's build. This is the Researcher's angle 1, direct F with
   resumed cursors, and it is the next construction already queued.
3. **Admission by measured benefit:**
   - the run build wins three to four times on pattern and one-byte blocks;
   - it loses two to three times at r/n ≈ 1.

   A per-block rule on r/n (or cells/r) for the builder alone is cheap to test from these counts.

Nothing here changes production. These targets go to the Optimiser and the Researcher in the same
wake.
