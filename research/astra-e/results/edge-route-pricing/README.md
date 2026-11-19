# Pricing the edge policy on the one sampled route change

On Wikidata stream ordinal 165, switching from the combined decoder’s dense fallback to its bounded edge-table route increased instructions by **52.0%**. Modeled cycles fell by **43.0%, 51.7% and 62.2%** with the standard W7, W8 and W10 cache settings. These six simulations use the actual two binaries on the same 900,000 output bytes. At W10 the model counted 58,833 dependent DRAM loads for combined and 51 for edge. L3 transfer bytes fell from 79.71 to 25.19 per output byte at each W. [Raw observations](runs.jsonl) · [Exact binary, tape, scorer and sample identities](manifest.json)

| Workers | Changed block edge / combined cycles | Changed block DRAM bytes edge / combined | Conditional edge / combined, nominal cycles | Conditional edge / combined, feedback | Conditional edge / LazyCells, feedback |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 0.57002 | 0.35530 | 0.96811 | 0.96590 | 0.87911 |
| 8 | 0.48278 | 0.30288 | 0.95677 | 0.95441 | 0.87019 |
| 10 | 0.37780 | 0.25688 | 0.93536 | 0.93822 | 0.85890 |

The last three columns are counterfactuals. Each replaces just three of the original 960 rows, one nominal observation at each high worker count. All other rows come from the archived combined/LazyCells run, including every elevated-latency observation. The model uses these to estimate demand-dependent memory latency. With this assumption, total weighted instruction count increases 1.48% at every W. DRAM bytes fall 5.86%, 6.15% and 7.55%. The nominal cycle-price change explains most of the predicted reduction. W9 lies outside the model grid. [Unrounded counterfactual and all metrics](counterfactual.json) · [Original 960 observations](../combined-v1-proxy/score/proxy-runs.jsonl)

Ordinal 165 is the only changed block in the official 48-full/16-tiny sample; the complete input contains four other route changes. No sampled changed block enters the elevated-latency subset. Charging the old slope, dispatch, unchanged-block execution and memory behavior to the new binary is the assumption behind the conditional columns. The official paired bootstrap assigns exactly unit edge/combined ratio to at least 177 of 400 draws under this substitution. Its 95% paired percentile interval must contain 1. An official full edge proxy would require measurements of the complete new binary on all scheduled cells. [Proof and exact draw counts](bootstrap.md) · [Whole-corpus route census](../../edge-budget/census.csv)

The repeated combined executable matched the canonical program SHA-256 and symbol tape. Instructions matched exactly. Twenty-two individual counter fields differed per W. The largest absolute change in combined block cycles was 19,567 out of 52,068,290 at W10 (0.038%). The control substitution changed the aggregate combined/LazyCells model ratio by at most 0.000018. Executable invocation and runtime placement were not controlled to identical addresses, and the cause of the small counter differences was not isolated. Both arm comparisons above use the fresh control. [Counter-level differences against the original run](counterfactual.json)

The diagnostic was preregistered at `0533de2`, the inputs and analysis at `3a2a297`, and the six observations and point estimates at `a6baa5e`. At the recorded 02:18:35 UTC cutoff on 2026-11-19, the public evidence branch still pointed to the earlier combined run, with both edge ready tags present. This is a prospective diagnostic relative to publication of the edge results. Each simulated W is the cache and latency setting for one instrumented worker. Independent target timing and RSS use the full Wikidata workload under concurrent workers. [Cutoff receipt](receipt-status.txt) · [Target protocol](../../workstation/edge-request.md)

## Replay

The pricing script used the master refresh at `be74d57`, whose last scorer-path change was `75297a1`. The scorer printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The scoring core, calibration matrix and chaingrind source have the same hashes as the original v1 run. The edge input is the scorer build at `a651e93`, with 338 gate cases. Combined uses its original build at `6a8a366`. Both candidates' generated module closures match the respective target arms. [Source audit](source-audit.md) · [Transitive source hashes](source-audit.json)

From the repository root, reproduce the fixed-point readout and the bootstrap proof offline. Both commands read the archived data and the scorer at the pinned revision.

```sh
git worktree add --detach /tmp/astra-edge-model d7703b020c10bf41099501d40b07dae92db5aa9a
edge_study=research/astra-e/results/edge-route-pricing
python3 "$edge_study/read.py" /tmp/astra-edge-model "$edge_study/runs.jsonl" | cmp - "$edge_study/counterfactual.json"
python3 "$edge_study/bootstrap.py" | cmp - "$edge_study/bootstrap.json"
```

Fresh simulations take the two frozen worker executables, the scorer's chaingrind runtime directory, the verified compressed Wikidata corpus, a directory for extracted symbol tapes, and a new output directory. The original command used one CPU at nice 19. The runner validates the warm/measure tape SHA-256 on reuse or extraction. The model instruments the worker from saved entropy-decoded symbols, preallocates its buffers, and warms with stream ordinal 1. It does not time the full process or measure peak RSS.

```sh
nice -n 19 taskset -c 4 python3 "$edge_study/run.py" /tmp/astra-edge-model \
  "$EDGE_BINARY" "$COMBINED_BINARY" "$CHAINGRIND_RUNTIME_DIR" "$WIKIDATA_BZ2" \
  /tmp/astra-edge-tapes /tmp/astra-edge-fresh
```

The original scorer and binary build receipts identify the compiler, learned model and source preparation for each candidate. [Edge build receipt](../v1p2-edge-confirm/manifest.txt) · [Combined build and toolchain](../combined-v1-proxy/README.md)
