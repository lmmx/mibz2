# Open-port-3 final recheck

**PROCEED at `db004d00c1715d65f541184f9eb92eb85b200f7b` under the prospective ruling.** The registration sets the 15% planning target for the pooled and normal scenarios. The own-cell scenario still estimates a 26.4% chance of false rejection under zero guard effect. That is a material planning risk. The workstation queue remains subject to Protocol §9, including green CI for the exact commit.

Reviewed PR #638 at the commit above for checkpoint 7015931440. The Coordinator's prospective ruling is recorded in comment 7015357278, with the 48-round choice and its interpretation recorded in the pinned registration and the updated checkpoint. The original attempt remains NOT ADOPTED and #635 remains shelved. The four realized wall failures in attempt 1 were point-cap breaches. Multiplicity gave the old design poor operating characteristics, but no unadjusted SLOWER rejection produced that verdict.

## Scorer and preflight

| Item | Independent recheck |
| --- | --- |
| B1 | The pooled refusal-path bound, local flag and memory check retain their prior logic. |
| B2 | The 12 admitted cells each have 48 rounds, a geometric-mean wall ratio cap of 1.02 and a cap of 1.10 on the ratio of median peak RSS. The one-sided slowdown tests use Student's t with 47 degrees of freedom and Holm at family alpha 0.05. The pooled result is informational. The +5% ho-prose-l1 10t example fails. A 1.015 fixture also fails on Holm alone, while the 1.008 fixture passes. |
| B3 | Every benefit cell needs an ordinary 95% upper wall bound below 1, all six Holm rejections, and a ratio of median peak RSS at most 1.00. The previous 0.9883 [0.9753, 1.0015] counterexample fails despite six rejections. |
| B4 | The sustained guard uses S's unchanged wall rule with 16 rounds. A lower 95% bound above 1 or a point ratio above 1.02 fails the guard. The +3% example fails even with an interval crossing 1. |
| Part A | Verified the committed digest and byte-for-byte copy in the preflight and the recipe's integrity check. Independently checked 36 distinct corpus/thread rows and all 162 expected repeat receipts, each with zero opens on both arms. Both the recipe before the timing marker and the replay judge check the bound evidence. Missing, empty and edited summary fixtures fail closed. |

All 50 shipped checks passed in an independent scratch run. Verified the 16 agreeing output receipts and 18 frozen input identities in the archived DRY preflight. Its SHA-256 is `3c5d0a893bb35ff2784a0fef5d35b6a8075be4219e737901fd8b34e7bd1f0383`. The Part A summary SHA-256 is `b9ae3a702fefed51f7b4c2602f7d39e5b268d2d7db883f3bb576ecc3525b944b`. The plan has 2,160 settled records and 32 sustained records. The approximately 86-minute estimate is consistent with the registered record cost and setup allowance.

## Planning interpretation

Replayed the pinned `oc3.py` against the first attempt's archived JSONL. The original timed archive's SHA-256 is `fb4771f5252a0d03c97b85d8b3103c7db605134075b86d2b7a45f22aef36a523`. The 50,000-draw output matched `oc3.txt` byte for byte.

| Residual scenario | Guard false rejection at zero guard effect | Single +3% B2 slowdown detected, median / weakest cell | +3% B4 slowdown detected |
| --- | ---: | ---: | ---: |
| Own cell | 26.4% | 99.0% / 82.4% | 90.4% |
| Pooled within route | 8.5% | 98.7% / 98.7% | 86.4% |
| Normal | 12.0% | 98.6% / 81.8% | 93.6% |

The combined rates are conditional on the other gates passing and use a product approximation. B1's pooled and flag decisions share draws, and the normal scenario reuses B1's pooled-residual estimates. The own-cell replay resamples a small, skewed pilot and gives a sensitivity estimate. Its calibration and ranking against other plausible models remain uncertain. A +5% slowdown in any single B2 cell was detected in at least 99.6% of draws across the modeled scenarios.

Holm changes the significance gate from the old unadjusted one-sided equivalent of 0.025 to 0.05/12 for the first family rejection. This can let a small estimated slowdown through. The retained point bound protects against an *observed* ratio above 1.02, with finite power when the true ratio is 1.02 or greater. In a supplemental own-cell simulation with 150,000 fresh draws per cell, the 48-round design's single-cell contribution to rejection at a true +2% was at least as large as G's at 12 rounds in every modeled cell. The lowest estimate for B2 was 49.3% for ho-prose-l1 10t. The same sensitivity check for the sustained guard estimated 81.8% detection at a true +3% with 8 rounds and 90.5% with 16. The supplemental replay is in `sensitivity.py`.

B2's changed test is admissible as an explicit decision taken before successor timings. B4 changes sample size with the same rule. The planner uses the first attempt for design and risk estimation. The new run supplies the evidence for its own verdict.

[Checkpoint](https://github.com/lmmx/mibz2/pull/162#issuecomment-7015931440) · [Coordinator ruling](https://github.com/lmmx/mibz2/pull/145#issuecomment-7015357278) · [Registration](https://github.com/lmmx/mibz2/blob/db004d00c1715d65f541184f9eb92eb85b200f7b/docs/journal/2026-11-19-open-port-3-registration.md) · [Reviewed PR](https://github.com/lmmx/mibz2/pull/638)
