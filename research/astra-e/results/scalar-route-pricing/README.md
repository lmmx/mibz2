# Scalar first-cap shortcut versus edge on three Wikidata blocks

Against the measured edge worker, the scalar worker retired 1.55–1.70% fewer modeled instructions on each of three preselected blocks. The modeled L2, L3 and DRAM byte transfers changed by less than 0.174%. The maximum past-L2 dependence chain on ordinary block 85 grew from 247 to 288 at W10 even as modeled cycles fell. The counters price specific decoder work on these blocks. The paired workstation experiment in #620 measures the sustained effect.

## Measurement and publication order

The pre-data plan was committed at 04:07:34 UTC on 2026-11-19 and verified on the research branch before measurement. Collection ran from 04:10:36.679958 to 04:11:08.986443 UTC. The raw files and both executables were published at commit `a7e9ebed0054fa4cbe23dbef5b431b5aa8e50c06`. No scalar i9 result was disclosed to this investigator by the end of collection. At 04:11 UTC the campaign observer found master at the Coordinator queue commit `3df195b`, no scalar ready tags, and the previous edge evidence `683c22e`. The actual start time on the independent workstation was unknown. [Pre-data plan](PLAN.md) · [Manifest](manifest.json) · [Timestamped raw counters](runs.jsonl)

Before this round the designated worktree `/workspace/work/astra-scalar-route-scorer` at `e0a84bf` fetched master and checked out its `bench/moonshot`, `.just` and `Justfile`. Master was `3df195b`, the scorer path revision was `75297a1` and the scorer printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The scalar arm used `d0a42980e896211906234c7679b38760ae44a46e`. The edge arm used `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. Each was built in a separate immutable-source checkout using that same projection and Rust 1.99.1. Both arms were rerun for every cell. The archive contains both executed binaries and the manifest identifies each executable, generated research source, library source tree and tool by hash. [Archive](workers.tar.gz) · [Manifest](manifest.json)

Each arm independently extracted zero-based Wikidata ordinals 1, 85 and 165 from the same canonical compressed corpus. The two executables produced byte-identical records. The upstream extractor deduplicates a repeated ordinal, so the harness concatenated separately extracted records to implement the planned warm-1/measure-1 as well as warm-1/measure-85 and warm-1/measure-165. In its uncounted sizing pass the official driver decodes the selected block once. It then warms the simulated caches on block 1 and measures a single decode of the selected block, including MTF, inverse, RLE1 and block CRC. Successful decoded lengths were 900000, 899988 and 900000 bytes respectively. The upstream paired correctness gates are separate. [Symbol tapes](symbols/) · [Runner](run.py) · [Scalar gate](../v1p2-cap-batch-scalar-gate/score/summary.txt) · [Edge gate](../v1p2-edge-confirm/score/summary.txt)

Chaingrind ran single-threaded at nice +15 on logical CPU 4. W7 used 2925 KiB of modeled LLC per worker, 42-cycle L3 and 495-cycle DRAM. W10 used 2048 KiB, 42 cycles and 665 cycles respectively. Each pair used identical model configuration. Worker contention, scan/admission scheduling and concurrent block output are outside this model.

## Work and transfers

All entries are ratios of scalar to edge for the same marked block and nominal W setting. Each transfer column counts bytes moved per output byte at that level. The cycle column comes from the scorer's execution model.

| W | Block | Instructions | L2 transfers | L3 transfers | DRAM transfers | Modeled cycles |
|--:|--:|--:|--:|--:|--:|--:|
| 7 | 1 | .9845 | .9989 | 1.0004 | .9983 | .9857 |
| 7 | 85 | .9837 | .9993 | .9998 | .9993 | .9944 |
| 7 | 165 | .9830 | 1.0000 | 1.0000 | 1.0000 | .9935 |
| 10 | 1 | .9845 | .9989 | 1.0004 | 1.0001 | .9949 |
| 10 | 85 | .9837 | .9993 | .9998 | 1.0001 | .9946 |
| 10 | 165 | .9830 | 1.0000 | 1.0000 | 1.0000 | .9944 |

## Dependent misses and overlap

Arrows give absolute counts or lengths from edge to scalar. Other columns give scalar/edge ratios. The past-L2 link count covers dependent loads that miss L2. Useful overlap per link is averaged over the dependent DRAM links shown. DRAM memory-level parallelism (MLP) is summed modeled load latency divided by DRAM busy cycles.

| W | Block | Past-L2 links ratio | Longest past-L2 chain | DRAM links | Longest DRAM chain | DRAM MLP ratio | Useful overlap per link ratio |
|--:|--:|--:|--:|--:|--:|--:|--:|
| 7 | 1 | 1.0056 | 363 → 284 | 33 → 33 | 13 → 12 | 1.0042 | 1.0791 |
| 7 | 85 | 1.0005 | 931 → 924 | 27 → 26 | 11 → 10 | 1.0009 | 1.0444 |
| 7 | 165 | 1.0000 | 880 → 880 | 26 → 27 | 12 → 12 | 1.0005 | .9781 |
| 10 | 1 | 1.0062 | 283 → 278 | 33 → 33 | 13 → 12 | 1.0042 | 1.0791 |
| 10 | 85 | 1.0004 | 247 → 288 | 27 → 26 | 11 → 10 | 1.0021 | 1.0444 |
| 10 | 165 | 1.0000 | 741 → 741 | 55 → 56 | 12 → 12 | 1.0002 | .9968 |

On dense block 165 the scalar arm added exactly one dependent DRAM link under each W setting. Those small denominators explain why a near-zero change in transfers can coexist with a visible change in average useful overlap. Raw per-arm data also retain read, write and drain traffic, store-buffer stalls and the sums behind these averages. [All per-arm metrics](metrics.csv) · [All matched differences and ratios](contrasts.csv)

## Replay

Use the original Rust 1.99.1 environment, Valgrind 3.24.0 including development files, a new workspace path and the canonical corpus with SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. The replay driver pins the historical scorer projection. Future comparison rounds require the normal master refresh. Build paths are embedded in the scorer executable, so the binary archive provides the exact executables used here. The plan-publication fields in the manifest refer to the original collection.

```bash
source /workspace/work/env.sh
source /workspace/work/proxy-tools/env.sh
bash research/astra-e/results/scalar-route-pricing/reproduce.sh \
  /workspace/work/astra-scalar-route-replay /workspace/work/astra-e/corpora/wikidata.bz2
```

To regenerate only the two CSV files, supply a checkout of the recorded scorer projection and a copy of this directory to `python3 -B report.py SCORER_CHECKOUT DATA_COPY`. The script checks the scorer module hash, symbol tape hashes, measured output lengths, pair coverage and nominal parameter settings before reformatting the raw counters. [Build script](build.sh) · [Replay driver](reproduce.sh) · [Metric extraction](report.py)
