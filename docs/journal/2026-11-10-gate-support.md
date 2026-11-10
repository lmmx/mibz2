# Gate support: #316's qualification size, #324's fallback harm

10 November 2026. The Pathfinder, under the redirect (LOG 6762859780: integration over
exploration). Arithmetic and untimed traces only. Nothing registered, frozen, built into
`src/`, timed or queued.

## 1. #316: is there a smaller defensible programme than k = 12 × R480?

**Yes, by k and R only: k = 11 × R240.** 2,368 runner-minutes against 3,546 (−33%). All-cell
lower bound at 1.005: 0.895 on the strict τ, 0.834 on the 25-link τ. Nothing else in the
request's list is smaller and still the route's claim.

**Inputs:** §2.11's own (registration draft at 8fd2c557): `plan.json`'s per-cell τ, the pilot's
mean-free `menu.json` (largest LRV a cell), the bound U_c = exp(D̄_c + t(0.95, k − 1) · sd/√k),
the cost k × (135 + 0.3344 R). The script reproduces the draft's table at k 8 and 12
(`2026-11-10-gate-support/nicalc.py.txt`, `nicalc2.py.txt`, their outputs beside them).

**The fixed design on a finer grid** (lower bound 1 − Σ misses, strict τ / 25-link τ;
runner-minutes):

| R \ k | 9 | 10 | 11 | 12 |
|---|--:|--:|--:|--:|
| 160 | 0.575 / 0.473 (1697) | 0.702 / 0.626 (1885) | 0.790 / 0.730 (2074) | 0.851 / 0.803 (2262) |
| 200 | 0.672 / 0.573 (1817) | 0.784 / 0.708 (2019) | 0.857 / 0.796 (2221) | 0.905 / 0.856 (2423) |
| **240** | 0.735 / 0.637 (1937) | 0.833 / 0.757 (2153) | **0.895 / 0.834 (2368)** | 0.934 / 0.884 (2583) |
| 320 | 0.803 / 0.705 (2178) | 0.882 / 0.805 (2420) | 0.929 / 0.867 (2662) | 0.958 / 0.907 (2904) |
| 480 | 0.848 / 0.749 (2660) | 0.910 / 0.832 (2955) | 0.947 / 0.883 (3251) | 0.969 / 0.917 (3546) |

- **Why R falls and k doesn't:** a dispatch is 135 minutes of setup plus 0.3344 R. Setup is 46%
  of it at R480, 63% at R240. And the weakest cell's variance (pattern/4/cpu, σ_D 0.0125) is
  89% τ at R480. Rounds buy little; halving them costs 3.5 points at k = 12.
- **Clearing 0.80 on both τ sets:** k 11 × R240 (2,368). The grid is flat there: k 12 × R160
  also clears it, at 2,262 (4% less), but barely on the 25-link set (0.803). k 11 × R240 keeps
  margin on both. On the strict set alone: k 10 × R240, 0.833 at 2,153 (−39%).
- **The lower bound is close to the truth here.** The product of the cells' powers
  (independent cells) is 0.899 against 0.895 at k 11 × R240. Four cells carry 91% of the
  misses: pattern/4 CPU and wall, text/4/wall, random/2/wall.
- **Elapsed:** `records` at R240 is 8.9 minutes of rounds at the pilot's pace, 15.5 at 1.75×,
  plus 15 of setup: under 31 minutes, against 49 at R480. Eleven dispatches at the draft's
  hourly spacing: 11 hours. The spacing is an independence parameter and is not shortened here.
- **R240 is 60 whole cycles.** The planning term LRV/R leans on spans up to 60 rounds, four to a
  dispatch. Validity doesn't rest on it: the SD is taken across dispatches.

**The three options named, each priced:**

1. **Fewer cells: not within the route.** STEWARDSHIP condition 2: "every cell of the standing
   CI end-to-end bench", and "any omission or change to that standing grid is a separate
   prospective bench-policy decision, not an exclusion available within this route". Such a
   decision is the team's (RULES C6) and forward-only (C2). Taken now, to shrink one
   candidate's qualification, it is the exclusion that sentence forbids; and it could not
   apply to #316 if any run of the candidate on these cells has been seen.
   - What it would buy: without the four high-τ cells, k 8 × R240 gives 0.874. Three
     dispatches saved, by removing the four cells whose history is least stable: the cells a
     no-harm claim most needs.
2. **A pooled-cell estimand: a different claim.** The 42-cell mean's σ_D is 0.0009; k = 3
   passes with power 1.000. A pooled geometric mean at 1.02 admits one cell at
   1.02^42 = 2.30× with the rest neutral. The route's claim is "no cell is worse by more than
   the margin".
3. **The τ calibration folded in: nothing to fold, and the fold that exists costs more.**
   - **The fixed design needs no A/A campaign.** U_c uses the k dispatches' own SD. τ sizes k
     and nothing else. The open calibration (t at small k under the runner-model mixture) is a
     computation on the neutral links: no runner-minutes.
   - **The fold that is exact:** Stein's two-stage procedure. k0 dispatches fix every cell's
     s0²; the total k is a registered function of the variances alone,
     k = clamp(k0, k_max, max_c ⌈s0_c² (t_α + t_β)² / Δ²⌉); the bound uses s0 and t(k0 − 1).
     No mean is read to size it, and under the draft's own model (normal iid dispatch means)
     its size is exact: 0.0502 simulated at nominal 0.05, 10⁶ draws, at the largest- and
     smallest-σ cells, strict and doubled τ.
   - **It is larger.** The maximum of 42 variances on 5–7 df overshoots, and the quantile is
     t(k0 − 1). At R480, k0 8, β 0.05:

     | cap | strict τ: all-cell, E[k], minutes | 25-link τ | τ doubled |
     |---|---|---|---|
     | 12 | 0.944, 11.5, 3407 | 0.886, 11.8, 3489 | 0.111, 12.0, 3546 |
     | 16 | 0.976, 13.7, 4037 | 0.949, 14.6, 4320 | 0.266, 16.0, 4728 |
     | 24 | 0.981, 14.9, 4407 | 0.973, 17.2, 5090 | 0.593, 24.0, 7090 |

     Fixed k 12 × R480: 0.969 at 3,546. Stein under the same cap: 0.944 at 3,407.
   - **What it buys is robustness to a wrong τ,** at a price: with τ doubled the fixed design's
     lower bound is 0 through k 14 (product 0.15 at k 12) and Stein's cap 24 reaches 0.59 for
     7,090 minutes.

**Also priced, not proposed:**
- **An early stop for success** (group-sequential): at k 6 the all-cell pass probability at the
  full α is 0.24 (R480). An interim saves under 0.24 × 6 = 1.4 dispatches in expectation and
  raises the final quantile.
- **Cells sharing a runner** (the intersection-union test needs no independence between
  cells): it would cut the 135 minutes of setup. It changes §2.2's fixed partition and each
  cell's cache and thermal history, and runner-minutes are free on this repository. Elapsed
  hours bind, and sharing lengthens them.

**What no size here covers:**
- **τ doubled:** lower bound 0 at every k ≤ 14, any R.
- **The true ratio.** All-cell lower bound at R240, strict / 25-link:

  | true ratio | k 8 | k 10 | k 11 | k 12 |
  |---|--:|--:|--:|--:|
  | 1.0000 | 0.957 / 0.916 | 0.992 / 0.975 | 0.997 / 0.986 | 0.999 / 0.992 |
  | 1.0025 | 0.853 / 0.782 | 0.959 / 0.919 | 0.979 / 0.949 | 0.989 / 0.967 |
  | 1.0050 | 0.581 / 0.445 | 0.833 / 0.757 | 0.895 / 0.834 | 0.934 / 0.884 |
  | 1.0075 | 0.000 / 0.000 | 0.463 / 0.293 | 0.608 / 0.479 | 0.714 / 0.611 |

  The planning value 1.005 is §2.4's and is not moved here.
- **Validity exposure, absent from every power above.** One shard failing after its
  measurement started makes the qualification INVALID, with no replacement. There are 9k such
  jobs: P(all valid) = (1 − q)^(9k). At q 0.2%: 0.82 (k 11), 0.81 (k 12), 0.75 (k 16). At q 1%:
  0.37, 0.34, 0.24. q is not estimated here. Shorter jobs and fewer dispatches both lower it.

**Recommendation:** k = 11 × R240 if 0.80 at 1.005 on both τ sets is the bar. k = 12 × R240
(2,583, −27%; 0.934 / 0.884) if the draft's margin above 0.80 is wanted. Either is §2.11's
design with two numbers changed: same unit, bound, admissions, schedule and claim.
