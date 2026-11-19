# Frozen route prices against the target i9

The conditional forecast predicted a wall-time gain from admitting the five dense blocks at each worker count it covered. The i9 confirmed the direction against the frozen combined writer. The forecast overstated the incremental gain by **1.30, 1.89 and 2.02 percentage points** at W7, W8 and W10. W9 resolved a measured gain with a wall ratio of 0.97627 [0.95528, 0.99773]; the simulator has no W9 cell. [Frozen points](counterfactual.json) · [Unrounded observations and errors](heldout-i9.csv)

| W | Edge / combined forecast | Edge / combined i9 [95%] | Log error | Edge / Lazy forecast | Edge / Lazy i9 [95%] | Log error |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 0.96590 | 0.97891 [0.97199, 0.98588] | −0.01338 | 0.87911 | 0.86703 [0.85996, 0.87416] | +0.01383 |
| 8 | 0.95441 | 0.97335 [0.96331, 0.98348] | −0.01965 | 0.87019 | 0.88363 [0.87236, 0.89504] | −0.01532 |
| 9 | — | 0.97627 [0.95528, 0.99773] | — | — | 0.89698 [0.87876, 0.91559] | — |
| 10 | 0.93822 | 0.95846 [0.94295, 0.97423] | −0.02135 | 0.85890 | 0.87591 [0.86133, 0.89073] | −0.01961 |

The forecast column uses the feedback calculation frozen in the study. The signed errors are ln(forecast / measured), consistent with the official retrospective. The table brackets are intervals on the measured paired log ratios. Every frozen feedback point lies outside the corresponding i9 timing interval. Sampling uncertainty in the conditional diagnostic is a different quantity. In 177 of the 400 frozen bootstrap draws, the one changed sample is absent, forcing the direct edge/combined ratio to one in those draws. [Bootstrap multiplicities](bootstrap.json)

The two errors differ because the absolute edge/Lazy forecast also prices the combined/Lazy reference. Within these same sixteen i9 rounds the combined/Lazy forecasts versus observations were 0.91015 versus 0.88571 at W7, 0.91176 versus 0.90782 at W8 and 0.91546 versus 0.91387 at W10. Log errors add exactly when composing these ratios. The W7 combined/Lazy log error of +0.02722 outweighs the edge/combined error of −0.01338, reversing the sign of the edge/Lazy error. The reference had smaller errors at W8 and W10. This decomposition uses the simultaneously measured combined arm. The CSV also preserves the frozen nominal-latency values as diagnostics, alongside the feedback values selected prospectively. [Full table](heldout-i9.csv)

## Data lineage

The six chaingrind executions, one per program and W on ordinal 165, and the unrounded predictions were committed at `a6baa5ebd58c5aa5c67cd4e2af06847df09d58da` at 02:18:36 UTC. The measured bundle opened at 02:21:17. A remote-ref check at 02:22:04 puts the freeze in the public branch ancestry while the evidence branch held the dry receipt. The independent target archive arrived at 02:33:19, evidence `683c22e98aa41e42f90209063aa3650b224c595f`, archive SHA-256 `3a10607c7d4ff026f4f3a7241a6d72dff10f590c07fd9b19eab9a7c6fbed2559`. [Publication record](publication.txt) · [Raw simulation rows](runs.jsonl) · [Target archive](../../workstation/evidence/edge-i9-683c22e.tar.gz)

The replay regenerates the entire frozen counterfactual JSON from its original inputs. Each 960-row reconstruction replaces **three** nominal rows, one at W7, W8 and W10, and carries **957** from the archived combined/Lazy run at `23378304c93e85b131afb85f5e86adf6bd71d099`. Ordinal 165 is the only route switch among the 48 full and 16 tiny sampled blocks. Ordinals 41, 56, 270 and 278 are absent. All high-latency slope observations, Lazy anchors and unchanged-block execution are inherited. The changed-block cycles and DRAM counters concern the instrumented inverse worker. No target per-block timing, DRAM traffic or RSS prediction was obtained from this substitution. [Study design](PLAN.md) · [Sample, binary identities and calibration digests](manifest.json)

The forecast used local edge build `a651e93` and the archived combined build `6a8a366`. The measured edge and combined source trees were `badb2ae` and `8c9640d`. The linked candidate modules were matched independently. The local edge library has an unused research-only byte-count accessor; the local and target executables have distinct digests. The target fetched the same scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493` used in the diagnostic. The printed versions depend on the checkout HEAD. [Source and binary audit](source-audit.md) · [Scorer-readable target receipt](../v1p2-edge-confirm/i9-receipt.json)

The replay verifies the evidence-branch digest, ordered arm/build identities, all 256 raw samples, full candidate gates, paired wall and peak intervals, medians and maxima. The official evaluator reads four target measurements for this source. Its local register contains W1 and W4 timings. The conditional route prices stay in the held-out table; they do not create a full-program proxy entry or an accuracy ranking.

```sh
study=research/astra-e/results/edge-route-pricing
python3 -B "$study/heldout.py" /tmp/astra-edge-heldout
cmp /tmp/astra-edge-heldout/heldout-i9.csv "$study/heldout-i9.csv"
cmp /tmp/astra-edge-heldout/i9-receipt.json research/astra-e/results/v1p2-edge-confirm/i9-receipt.json
python3 -B bench/moonshot/eval.py research/astra-e/results/v1p2-edge-confirm
```
