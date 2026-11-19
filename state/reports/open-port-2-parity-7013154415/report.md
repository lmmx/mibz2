# #635 parity re-check

**HOLD at `2eff6f73` for three small schedule changes.** All ten previously flagged cells now clear the chosen +2% parity target. Three other cells fall below the 0.80 minimum when evaluated at the cutoffs the scorer actually reads. They clear the floor on the next round-count grid point under the same own-cell model. #631's NOT ADOPTED stands. [Checkpoint](https://github.com/lmmx/mibz2/pull/162#issuecomment-7013154415) · [Parity decision](https://github.com/lmmx/mibz2/pull/145#issuecomment-7013005249) · [Hand-back](https://github.com/lmmx/mibz2/pull/145#issuecomment-7013140208)

## Locked-rule parity

I verified the first evidence archive's SHA-256, `fb4771f5252a0d03c97b85d8b3103c7db605134075b86d2b7a45f22aef36a523`, and reproduced the pinned planner's `plan.json` byte-for-byte with 50,000 draws. The planner compares the simulation to the **unrounded** maximum of the three null quantiles during the search. It rounds the cutoff conservatively to four decimals for the schedule, without rechecking power at that value. The judge uses the serialized cutoff. Where the planner selected power close to 0.80, the upward rounding and reuse of the selection draws matter. The nominal 0.005 tolerance is applied to G in `max(0.80, G - 0.005)`. The 0.80 floor is a separate bound. [Plan search](https://github.com/lmmx/mibz2/blob/2eff6f7388927f9ccfde54283aded98b9cb71761/docs/journal/2026-11-19-open-port-2-registration/plan.py) · [Locked schedule](https://github.com/lmmx/mibz2/blob/2eff6f7388927f9ccfde54283aded98b9cb71761/bench/open-port-2-schedule.json) · [Scorer](https://github.com/lmmx/mibz2/blob/2eff6f7388927f9ccfde54283aded98b9cb71761/bench/open_port2.py)

I evaluated all 40 cells at their locked cutoffs and round counts with 200,000 fresh draws per model, using the same pilot residuals and mean-shift model. These are independent simulation draws, not new decoder measurements or new information about the i9's noise. All ten previously flagged cells meet their targets:

| Cell | Old G +2% | Locked G2 +2% | Target |
| --- | ---: | ---: | ---: |
| Held-out prose level 9, 10t | 1.0000 | 0.9967 | 0.9950 |
| Held-out prose level 9, 20t | 0.9205 | 0.9687 | 0.9155 |
| Held-out structured level 9, 20t | 0.9989 | 0.9995 | 0.9939 |
| Held-out Wikidata level 1, 8t | 0.9883 | 0.9982 | 0.9833 |
| Held-out Wikidata level 1, 10t | 1.0000 | 0.9984 | 0.9950 |
| Random, 8t | 0.9998 | 0.9988 | 0.9948 |
| Random, 20t | 0.9996 | 0.9992 | 0.9946 |
| Records, 8t | 0.9762 | 0.9820 | 0.9712 |
| Records, 20t | 0.9647 | 0.9840 | 0.9597 |
| Text, 20t | 0.8775 | 0.9442 | 0.8725 |

Three different cells miss the fixed 0.80 floor. A separate seed and one million own-cell draws per cell put their estimates consistently below it. Simulation standard error is about 0.00040 at that sample size. The table also shows a narrowly scoped repair screening. For each next grid point I estimated the max of the three null quantiles with one million draws per model, rounded up, and used a fresh million draws for the power estimate.

| Cell | Locked n / cutoff | +2% at 200k | +2% at 1m | Next-grid n / cutoff | Next-grid +2% |
| --- | --- | ---: | ---: | --- | ---: |
| Held-out structured level 9, 8t | 84 / 0.0161 | 0.79596 | 0.79561 | 96 / 0.0150 | 0.87383 |
| Held-out structured level 9, 10t | 84 / 0.0161 | 0.79746 | 0.79805 | 96 / 0.0150 | 0.87363 |
| Held-out Wikidata level 1, 20t | 72 / 0.0170 | 0.79795 | 0.79735 | 84 / 0.0161 | 0.87531 |

Evaluate the serialized cutoff during the grid search and validate the selected (n,c) on fresh simulation draws. Advancing only these cells adds 36 pairs, about 2 min 31 s at the existing 2.1 s/decode rate. The resulting schedule and size estimates should be checked at the new locked head. [Published plan](https://github.com/lmmx/mibz2/blob/2eff6f7388927f9ccfde54283aded98b9cb71761/docs/journal/2026-11-19-open-port-2-registration/plan.json)

## Sustained, size and schedule

- **S remains admissible under the ruling on the previous version.** At the locked 84 pairs and (c=0.0162), the own-cell +2% estimate is **0.8598**, versus **0.5476** for old S. There were zero null rejections in the 200,000 own-cell draws. Own-cell +1% power is about 0.025. In the all-guard pool and spike stress, the +2% estimates are 0.757 and 0.742. Those estimates define the scope of the 0.86 figure. Eight old pairs still limit what can be inferred about a longer back-to-back sequence.
- **Size is model-scoped.** At the serialized cutoffs, the fresh product-of-cell-pass estimates are **0.0110 / 0.0100 / 0.0313** in the three IID planning models, and **0.1419** under the 2% spike stress. The corresponding marginal-size sums are 0.0111 / 0.0101 / 0.0318 / 0.1527. The close agreement with the published numbers supports the calibration as stated. It does not establish a distribution-free 5% bound or a guarantee under stress. The maximum locked cutoff is 0.0170 and the spike witness's mean is 0.02405.
- The schedule uses 72–576 pairs per ordinary cell, 84 for S, and **4,932 extension pairs in 19 batches**. The extension takes 5 h 28 m 48 s in settling alone and about 5.75 h at 2.1 s per decode. Adding the 38 min decision run, 3 min sustained run, and 15 min context gives about 6.69 h before setup or variability, consistent with a 6.75 h estimate.

## Scorer and preflight

The compare from `90f8b2c2` shows no edits to either scorer, the recipe, or the frozen input manifest. The test file does change to move the spike fixture to the neutral text guard. All 30 checks pass independently, including the spike witness, both cross-file VOIDs and the scheduled-set tests. The new preflight at `7827d864` hashes to `bfbc1d4616b4c6d02c4c4ef56d27fb528b898c513799552e9e6c011fe997d0f8`. I verified 18 manifest entries, 16 agreeing output pairs and the 30-check receipt. No timed JSONL or retained run-queue measurements appear. [Diff](https://github.com/lmmx/mibz2/compare/90f8b2c2210af374c0531a5cc287c186f0474c04...2eff6f7388927f9ccfde54283aded98b9cb71761) · [DRY archive](https://github.com/lmmx/mibz2/blob/2eff6f7388927f9ccfde54283aded98b9cb71761/docs/journal/2026-11-19-open-port-2-registration/preflight/run-20261119T150859Z.tar.gz)
