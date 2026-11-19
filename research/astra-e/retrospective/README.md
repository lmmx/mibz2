# Retrospective on independent measurements

The combined narrow, short-copy and lookahead decoder is the one candidate in this snapshot with an independent i9 result. Across W7, W8 and W10 the v1 proxy's mean absolute log error was 0.01491. The observed ratios lie in all three prespecified model intervals. W9 has a target measurement and no model estimate. With only one candidate per W, target rank agreement has zero eligible pairs. [Prospective proxy and measured intervals](../results/combined-v1-proxy/heldout-i9.csv) · [Exact official report](official.txt)

| Workers | v0 proxy | v1 proxy | Target i9 candidate / LazyCells [paired 95%] |
| ---: | ---: | ---: | ---: |
| 7 | 0.89352 | 0.91015 | 0.89823 [0.88756, 0.90903] |
| 8 | 0.87952 | 0.91177 | 0.91173 [0.90305, 0.92049] |
| 9 | — | — | 0.90312 [0.88962, 0.91681] |
| 10 | 0.82768 | 0.91548 | 0.88709 [0.87955, 0.89469] |

The raw v0 forecast was committed at `62bde54` at 00:32 UTC. The complete v1 forecast was committed at `2337830` at 01:27 UTC and published through `047540d` before the workstation run. The run bundle began at 01:31 UTC and was published at `c381da2` at 01:43 UTC on 2026-11-19. The comparison reuses the measured run for both forecast vintages. The mean absolute log errors, 0.03685 for v0 and 0.01491 for v1, characterize these three cells of this decoder. [Audited builds, source paths and unrounded cells](audit.json) · [Target receipt](../results/combined-v1-proxy/i9-receipt.json)

## Evidence and join coverage

The frozen data commit is `d7703b020c10bf41099501d40b07dae92db5aa9a`. The reader came from upstream scorer path revision `75297a1d57b2bdf262c3a19f3064848dedf2c493`. It saw 122 forecast rows and 31 distinct independent run/worker cells from nine CI runs at W1/2/4 and the measured i9 run at W7/8/9/10. Four previously structured CI receipts were retained. The five additions cover combined/v1, edge-budget, dynamic delimiter refill, F5 combined word actions, and guarded POPCNT. All comparisons archived for each selected candidate, including the frozen predecessor arms, remain in the receipts. The official reader consumes the selected-candidate/LazyCells contrast. [Verified CI receipts](ci-receipts.json) · [Source ZIPs and raw runs](../ci/evidence)

Of the 148 printed rows, 82 describe forecasts without an independent match at that W. Ten independent cells lack a local forecast at that W, including all three F5 CI cells, since its completed local score at the snapshot was a correctness-only gate. The other 56 rows pair a prediction with an independent measurement. Of these, the reader joined 28 by SHA-256, 11 by directory, and 17 by path suffix. All 56 have matching entry-file digests independently traceable from the registers or score manifests and archived CI/i9 evidence. For the combined program the generated-kernel inputs and shipping source were also checked across both local forecast commits and the target commit. [Detailed joins, digests and unmatched measurements](joins.csv) · [Audit code](audit.py)

The local register reader iterates the measured wall cells. The combined proxy-only register has empty walls, and the summary reader emits its proxy predictions without an entry digest. The single target receipt is beside its v1 forecast, so its three target pairs enter official calibration through a directory join. The v0 forecast joins the same receipt by path and the official report excludes it from calibration. Three of the nine v0 proxy rows in the official “other hosts” line are these target-machine path-only joins. The separate source audit identifies them in the table above. Both complete forecasts keep the original empty timing arrays. [Reader implementation](../../../bench/moonshot/eval.py) · [Proxy manifest](../results/combined-v1-proxy/manifest.txt)

| Official retrospective group | Rows | Mean absolute log error | Same-W, same-host-string rank pairs correct / eligible |
| --- | ---: | ---: | ---: |
| v0 local wall versus other hosts | 10 | 0.14608 | 5/6 |
| v0 proxy outside official calibration | 9 | 0.02851 | 3/3 |
| v1 local wall versus other hosts | 28 | 0.17137 | 54/70 |
| v1 proxy outside official calibration | 6 | 0.03579 | 3/3 |
| v1 proxy in official target calibration | 3 | 0.01491 | 0/0 |

Repeated register entries, repeated CI runs, and different local forecasts for one source can all contribute multiple rows. For example, the v1 cross-host wall total has 16 distinct tuples of forecast build, candidate path, W, predicted value and independent run among its 28 rows. The reader groups ranks by the literal host label, which varies across some receipts for the same processor model. Its joining and ranking functions do not enforce forecast chronology, require distinct candidate identities or filter out close/noisy screens. Only the combined target comparison above has its prospective order separately established here. Local ratios versus CI remain descriptive. CI spans different CPUs and worker counts from the target. [Machine-readable counts](audit.json) · [Unrounded paired wall and RSS data](ci-receipts.json)

## Reproduce offline

A full repository clone with the pinned campaign history, Python 3 and `just` suffices. From a checkout containing this retrospective, run the following. The audit program stays in the current checkout while the evaluator reads an isolated checkout of the immutable data snapshot. The CI verifier rebuilds five human receipts from the original raw data and checks stored ratios for the four earlier receipts. The i9 verifier replays all 256 timing rows using the original assessor and scorer Python files, checks all 338 candidate gate cases and 98 cases for each reference, and reproduces the archived decision exactly.

```sh
retro_dir="$(pwd)/research/astra-e/retrospective"
retro_tmp="$(mktemp -d)"
git worktree add --detach "$retro_tmp/data" d7703b020c10bf41099501d40b07dae92db5aa9a
(
  cd "$retro_tmp/data" || exit 1
  python3 research/astra-e/retrospective/receipts.py &&
  LC_ALL=C just moonshot-eval research/astra-e > "$retro_tmp/official.txt" &&
  python3 "$retro_dir/audit.py" "$retro_tmp/replay" &&
  cmp "$retro_tmp/official.txt" "$retro_dir/official.txt" &&
  cmp "$retro_tmp/replay/joins.csv" "$retro_dir/joins.csv" &&
  cmp "$retro_tmp/replay/audit.json" "$retro_dir/audit.json"
)
```

The official command's stdout also matched a direct invocation with three different Python hash seeds. The snapshot includes the first 16-round combined re-score after the upstream evaluator landed. Later scores or workstation runs can change counts in a live run. The raw snapshot and report remain available for comparison.
