# Independent scalar i9 statistical receipt

The registered scalar first-cap worker passed the incremental criterion on the i9-10900K. Its equal-weight W7–10 wall-time ratio against the edge worker was **0.994625 [0.990779, 0.998485]**. All four memory checks by worker count passed against 0.5.6 LazyCells. Its measured throughput was **1.137× to 1.164× LazyCells**, below the 1.25× objective at each worker count. [Timed archive](evidence/scalar-i9-1ac706d.tar.gz) · [Independent decisions](scalar-independent-i9.json)

The target ran 48 rounds of four arms at each of W7, W8, W9 and W10, for 768 measurements. Each produced 838,800,784 decoded bytes from four Wikidata copies. The treatment was scalar at `d0a42980e896211906234c7679b38760ae44a46e`, and the edge control was `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. The edge executable supplied its LazyCells and opend anchors. Archive SHA-256 was `a5ec503104fb95412e6452dc821ed261a02bdda3234a97fa8b9334c47f46ae5f`. The campaign's separate authentication checked the source, compiler, scorer, oracle gates, target and input. [Independent authentication](evidence/scalar-i9-1ac706d.json) · [Frozen design](scalar-plan.md)

## Wall and CPU

Each interval uses 48 same-round log ratios, the df47 two-sided Student 95% interval, and exponentiation. The pooled primary first averages the W7–10 log ratios **within each round**. The verifier at `1f2bd98` obtained its critical value 2.01174051372977 by integrating the Student density after a trigonometric transformation. It used only standard-library code and the raw rows. The resulting pooled and individual intervals, per-W peak checks and decisions agreed with the published decision within 2×10⁻¹². The four estimates have separate simultaneous obligations under the registered decision rule. [Independent verifier](scalar_metrics.py)

| W | Scalar / edge wall [95%] | Scalar / opend wall [95%] | Scalar / Lazy wall [95%] | Throughput vs Lazy | Scalar / edge CPU [95%] | Scalar faster than edge |
|---:|---|---|---|---:|---|---:|
| 7 | .995108 [.993142, .997077] | .932808 [.929653, .935974] | .870223 [.867226, .873231] | 1.149× | .9956 [.9935, .9978] | 37/48 |
| 8 | .996255 [.989432, 1.003125] | .941888 [.937471, .946325] | .879699 [.874466, .884963] | 1.137× | .9989 [.9957, 1.0022] | 31/48 |
| 9 | .999513 [.992743, 1.006330] | .931735 [.923710, .939831] | .876325 [.868121, .884607] | 1.141× | .9936 [.9890, .9984] | 26/48 |
| 10 | .987661 [.975904, .999560] | .922457 [.913433, .931570] | .858968 [.850807, .867207] | 1.164× | 1.0055 [.9934, 1.0177] | 25/48 |

The scalar/edge wall intervals exclude one at W7 and W10 and include it at W8 and W9. All four upper endpoints meet the 1.02 incremental guard. At W10, the scalar worker had a lower paired wall ratio with a higher CPU point estimate and CPU/wall occupancy ratio of 1.0180. The geomean uses the size as well as the sign of round differences. The W10 upper wall endpoint lies close to equality. The archive contains wall seconds, CPU seconds and peak RSS. It contains no hardware instruction, cache or stall event counts. All six pairings at each W have separate wall, CPU, derived CPU/wall occupancy and memory estimates, 48-round intervals, absolute medians and pair counts. [All 24 comparisons](scalar-independent-i9.csv)

## Memory

The preregistered peak rule checks the paired scalar/Lazy upper bound, the ratio of separate peak medians, and observed maxima. All three passed at each W. Scalar used a smaller observed peak than Lazy in each of the 192 matched pairs. VmHWM values below are KiB.

| W | Paired RSS scalar/Lazy upper | Median scalar / Lazy | Median ratio | Max scalar / Lazy | Max edge |
|---:|---:|---:|---:|---:|---:|
| 7 | .698868 | 192,338 / 276,716 | .695074 | 194,316 / 298,208 | 196,244 |
| 8 | .691742 | 202,246 / 294,110 | .687654 | 205,356 / 331,772 | 205,532 |
| 9 | .676573 | 210,594 / 314,968 | .668620 | 212,532 / 331,712 | 213,672 |
| 10 | .673634 | 219,882 / 330,500 | .665301 | 223,760 / 360,400 | 223,152 |

Relative to edge, each paired upper RSS ratio exceeded one. At W10 the scalar median exceeded edge by 190 KiB and the maximum exceeded edge by 608 KiB. Those comparisons matter for a claim about added memory versus the preceding implementation. The target uses LazyCells as its memory reference. The largest scalar/Lazy upper wall ratio was .884963 at W8; even the smallest upper, .867207 at W10, exceeded the required .80.

## Reproduce

```bash
d=research/astra-e/workstation
python3 -B "$d/scalar_metrics.py" --self-test
python3 -B "$d/scalar_metrics.py" "$d/evidence/scalar-i9-1ac706d.tar.gz" \
  --csv /tmp/astra-scalar-i9-pairs.csv > /tmp/astra-scalar-i9-audit.json
cmp /tmp/astra-scalar-i9-pairs.csv "$d/scalar-independent-i9.csv"
cmp /tmp/astra-scalar-i9-audit.json "$d/scalar-independent-i9.json"
```

The synthetic checks exercise a favorable pooled result defeated by a single W, three isolated memory failures, malformed records and common same-round noise that reverses the result if 192 worker-count observations are treated as independent. As registered, all intervals here use the variability across 48 round summaries or matched pairs.
