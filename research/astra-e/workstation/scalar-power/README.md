# Blind pooled-contrast precision from the accepted edge run

This design calculation reads only the 16-round i9 edge/combined predecessor receipt, SHA-256 3a10607c7d4ff026f4f3a7241a6d72dff10f590c07fd9b19eab9a7c6fbed2559. At each round j, its observation is one unweighted mean of the four paired log wall ratios for W7, W8, W9 and W10. The 16 observations have sample SD 0.012709. Retaining their cross-W covariance matters; setting all covariances to zero would give 0.014046. The observed historical edge effect is subtracted. Every effect below is an assumed reduction in a prospective candidate's time relative to its matched predecessor. [Frozen archive](../evidence/edge-i9-683c22e.tar.gz) · [All four-W vectors](rounds.csv) · [Dispersion](dispersion.json)

| Planned rounds | Ratio halfwidth at 95% | Power at 0.44% | Power at 0.8% | Power at 2% |
|---:|---:|---:|---:|---:|
| 32 | 0.459% | 47.7% | 93.4% | >99.99% |
| 48 | 0.370% | 65.3% | 99.0% | >99.99% |

Success here means the upper endpoint of the usual **two-sided 95% paired t interval** on the pooled wall ratio is below 1. The prospective block-count unit is the complete round. The design treats rounds as independent normal log observations and transports their historical SD. The multiplicative halfwidth is exp(t₀.₉₇₅ × SD / √N) − 1. Directional power is the lower-tail noncentral-t probability under the stated wall reduction. The table addresses this single pooled comparison. Additional constraints on each W, correctness or RSS can lower the probability of promotion. The raw table includes the explicitly different one-sided 95% calculation. [Analytical cells](analytic.csv)

For 0.44%, 48 rounds still have about a one-in-three chance of an inconclusive interval under those assumptions. The first integer reaching 80% power at the point SD is 68. The 95% normal-iid interval for the historical SD itself is 0.00939–0.01967. At those endpoints 0.44% power spans 23–73% at N=32 and 33–89% at N=48. The original run can only price design sensitivity. Different executable pairs, rotor positions, machine load and long-range fluctuations can change its variance.

The resampling sensitivity uses 100,000 fixed-seed trials per N and design while preserving each observed four-W vector. Centering the aggregate and resampling it is algebraically identical to drawing its original four W values jointly and then collapsing. Separate designs draw complete rounds independently, draw each round from its two historical observations at the same phase of the eight-round rotor, or draw consecutive circular blocks of four. The last design has null directional decision rates above the nominal 2.5%; the 16-vector history has first-lag correlation 0.319. With only two complete rotor cycles these are diagnostics of sensitivity to ordering. They do not validate iid coverage on a future run. [Simulation cells and Monte Carlo errors](resampling.csv)

```sh
OPENBLAS_NUM_THREADS=1 python3 -B research/astra-e/workstation/scalar-power/power.py
```

The script needs Python, NumPy and SciPy. The recorded run used NumPy 2.3.5 and SciPy 1.17.0. [Source](power.py)
