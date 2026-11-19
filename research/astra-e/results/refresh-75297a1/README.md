# Six references on the refreshed v1 scorer

All six entries were rebuilt at `a651e93f063737426cc19223d9b3ed8a2650fd8f` using upstream scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`. Each invocation fetched master and checked out its scorer projection. All printed v1 at `81585172f5fb9e7a59f330124779ba084fd6bf04` and passed 338 checks. Each arm was paired with its own 0.5.6 LazyCells and opend controls for nine rounds, two corpus copies per sample, two-second settles and 4W slots. The input SHA-256 was `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. The Xeon Platinum 8573C had a four-CPU quota. [Raw machine metadata and exact invocation](../v1p2-combined-confirm/manifest.txt)

| Candidate | W1 wall / Lazy [95%] | W4 wall / Lazy [95%] | Median RSS / Lazy W1, W4 |
|---|---|---|---|
| [combined](../v1p2-combined-confirm/) | 0.928 [0.782,1.102] | 0.877 [0.710,1.085] | 0.966, 0.844 |
| [final-slot](../v1p2-final-slot-confirm/) | 0.819 [0.565,1.188] | 0.767 [0.517,1.139] | 0.969, 0.877 |
| [edge](../v1p2-edge-confirm/) | 1.067 [0.882,1.291] | 0.943 [0.758,1.172] | 0.898, 0.750 |
| [guarded](../v1p2-guarded-confirm/) | 1.129 [0.755,1.689] | 1.807 [0.599,5.458] | 0.999, 0.960 |
| [preload](../v1p2-preload-confirm/) | 0.908 [0.770,1.071] | 0.778 [0.590,1.026] | 0.992, 0.916 |
| [compact](../v1p2-compact-confirm/) | 0.866 [0.566,1.326] | 0.959 [0.782,1.176] | 0.995, 0.916 |

Final-slot had the lowest point estimate at both W1 and W4. Edge had the lowest median peak RSS. Every candidate/Lazy wall interval contained parity in this series. In guarded W4 one matched ratio was 20.085 and the slowest candidate/reference observations were 62.20/47.09 seconds. The full sample remains in the register. The independently measured compact-preload gain over guarded POPCNT uses a directly paired five-arm CI run. [Direct candidate/opend timing, CPU, peaks and extrema](paired.csv) · [Independent CI](../../ci/evidence/compact-preload-44440381983.md)

Recompute all 24 contrasts with `python3 research/astra-e/results/refresh-75297a1/replay.py`. The scorer manifests beside each row contain exact invocations for the six original runs.
