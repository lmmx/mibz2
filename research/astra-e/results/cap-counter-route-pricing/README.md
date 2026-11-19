# What changes when the worker recovers its counters from queues?

On all three Wikidata blocks, queue-counter recovery removed about **3–4% of dynamic loads and 6.7–7.8% of stores**. It removed only 0.04–0.32% of retired instructions. The warmed traces show little corresponding change in cache-line transfers. The first-cap shortcut and queue recovery saved almost exactly the same number of instructions with either switch setting for the other. This accounts for an improvement in scan work. The incremental whole-worker timing remains unresolved. [Native factorial](../../front-fusion/cap-counter/README.md) · [Paired local score](../v1p2-cap-counter-confirm/score/summary.txt)

## Matched records and scope

The pre-data registration was frozen at 05:02:55 UTC on 2026-11-19. Collection ran 05:07:13–05:08:32 UTC. The 14-block local paired native result and local scorer were known. Independent factorial CI was in progress at registration. This analysis used no independent CI or i9 outcome. The source pins and input selection preceded the measurements. [Plan](PLAN.md) · [Raw records and exact commands](runs.jsonl) · [Manifest](manifest.json)

Immediately before the round the designated worktree fetched origin/master and checked out its scorer, .just and Justfile. Imported master was `3df195b10e9fb6f86f0c06ad9fad1feb992406d5`, the scorer-path log returned `75297a1`, and the scorer printed `scorer v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The raw manifest records the collection worktree HEAD `4b5dbc0`, SHA-256 of every relevant scorer file, worker, input, Valgrind wrapper, Valgrind runtime and Chaingrind binary.

Four frozen workers ran on each of three zero-based block ordinals under both W7 and W10 model settings. E is historical edge. S adds scalar first-cap. C adds queue-counter recovery on edge. B enables both. All E and S binaries are byte-identical to the prior scalar/edge study. C and B were independently built from `50b8ecb8946a498601e3fd894ed84a2687df6cc8`, with Cargo locked release, one build job, rustc 1.99.1 and LLVM 23.1.1. The three newer worker sources share a library tree. The archived E pin predates the research symbol-extraction interface in that tree. Retaining the exact E and S executables permits a same-binary comparison across studies. Instruction-layout effects are part of these executable comparisons. [Source generation and switches](../../front-fusion/cap-counter/generate.py) · [Control binary archive](../scalar-route-pricing/workers.tar.gz) · [C and B archive](counter-workers.tar.xz)

| Worker | Immutable source | Executable SHA-256 prefix |
|---|---|---|
| E | badb2aec3624bd65606c429b2e6f79e9d9d0050f | 34951c18521f |
| S | d0a42980e896211906234c7679b38760ae44a46e | b31166f10e33 |
| C | 50b8ecb8946a498601e3fd894ed84a2687df6cc8, counter_edge.rs | 70f37c5c2c93 |
| B | 50b8ecb8946a498601e3fd894ed84a2687df6cc8, both.rs | b036bd62a753 |

Each executable separately extracted the three records from compressed Wikidata SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Single-record SHA-256 and the concatenation of block 1 with each measured record matched the previously archived bytes. Each worker's official proxy decoded its measured block once for allocation sizing, warmed the model on block 1, then counted the complete post-Huffman worker including MTF, inverse, RLE1 and block CRC. The official proxy confirmed two records, execution under Chaingrind and the following output counts for all four binaries in both calibrations. The scorer separately passed 338 libbzip2 correctness cases for the combined worker. [Original input archive](../scalar-route-pricing/symbols/) · [Corpus geometry](../../front-fusion/geometry.csv) · [Correctness receipt](../v1p2-cap-counter-confirm/score/summary.txt)

| Block | Marked row | Capped runs | Queued rows | Measured output bytes | Block CRC |
|---:|---:|---:|---:|---:|---|
| 1 | 264045 | 56386 | 89874 | 900000 | d4a25a5c |
| 85 | 880651 | 67594 | 101980 | 899988 | ab859da1 |
| 165 | 569113 | 140594 | 221139 | 900000 | 543f70d1 |

Blocks 1 and 85 are ordinary admissions. Block 165 is one of the additional edge-budget admissions. The six cells used a single instrumented worker per invocation on logical CPU 4 at nice +15, with Valgrind 3.24.0. Per-worker modeled LLC/L3 latency/DRAM latency were 2925 KiB/42/495 cycles for W7 and 2048 KiB/42/665 for W10. Hardware IPC, concurrent bandwidth, output interleaving, wall time and process peak RSS require separate measurements.

## Instruction work, memory instructions and modeled stalls

Switch effects below are signed counts for one decode. Each pair is the change C−E followed by B−S. Both calibrations give identical numbers. The last column is the additive retired-instruction interaction (B−S)−(C−E). [All 264 metric-by-cell contrasts](contrasts.csv)

| Block | Instructions | Loads | Stores | Instructions B vs E | Instruction interaction |
|---:|---:|---:|---:|---:|---:|
| 1 | −5,188 / −5,189 | −81,556 / −81,557 | −91,766 / −91,766 | −1.592% | −1 |
| 85 | −10,822 / −10,823 | −101,409 / −101,410 | −113,234 / −113,234 | −1.703% | −1 |
| 165 | −99,228 / −99,236 | −255,564 / −255,565 | −263,907 / −263,911 | −2.007% | −8 |

The scalar switch alone saved 199,692, 244,423 and 539,976 instructions on blocks 1, 85 and 165. Queue recovery saved its most instructions per block on the denser additional admission. Its memory-instruction savings agree with recovering retained occurrences from existing per-byte queue lengths, which avoids repeatedly updating counts for them. The model recorded fewer store-buffer stall cycles for C versus E in all six cells (−2.14% to −4.47%) and B versus S (−2.38% to −4.25%). The ordered native 14-block factorial found B/E CPU time 0.9663 [0.9388, 0.9946] and B/S 0.9763 [0.9514, 1.0018]. The latter interval includes equal time. The modeled cycle advantages below are smaller than those native point estimates, under different sample and weighting rules. [Native stages and contrasts](../../front-fusion/cap-counter/results/contrasts.csv)

| Model | Block | Modeled cycles C/E change | B/S change | B/E change | Cycle interaction | DRAM read lines C−E / B−S | DRAM write lines C−E / B−S |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 7 | 1 | −0.579% | −0.630% | −1.327% | −2,867 | +18 / −83 | +29 / −76 |
| 7 | 85 | −0.824% | −0.813% | −1.483% | +1,328 | −9 / −13 | 0 / +1 |
| 7 | 165 | −1.300% | −1.255% | −1.897% | +8,942 | −1 / +4 | +5 / +12 |
| 10 | 1 | −1.275% | −1.243% | −1.756% | +3,038 | −3 / −11 | +1 / +1 |
| 10 | 85 | −1.085% | −1.059% | −1.601% | +2,949 | −6 / −4 | +2 / +8 |
| 10 | 165 | −1.321% | −1.267% | −1.821% | +11,945 | +16 / +32 | +2 / +8 |

The L2 transfer differences in C−E and B−S were respectively +41/+45, +6/−9 and +20/+113 lines by block in either calibration. At L3 they were +40/+46, +83/+114 and +97/+114. All these conditional changes have magnitude below 0.056%. DRAM read and write deltas change direction by comparison or cell. The simultaneous reduction in load/store instruction count and near-constant cache-line volume distinguish the two mechanisms.

## Dependency and in-flight diagnostics

Entries within each cell are in E/S/C/B order. Useful overlap samples are dependent DRAM loads with an observation. Useful overlap per sample counts other modeled DRAM-load intervals overlapping that observation. Modeled DRAM memory-level parallelism (MLP) is summed load latency divided by modeled DRAM busy cycles. Raw past-L2 link counts and the numerators and denominators behind each ratio are retained. [All 24 per-arm metrics](metrics.csv)

| W | Block | Longest past-L2 chain | Longest DRAM chain | Overlap samples | Useful overlap/sample | DRAM MLP |
|---:|---:|---|---|---|---|---|
| 7 | 1 | 284/284/283/284 | 13/12/12/12 | 33/33/34/32 | 6.515/7.030/6.912/6.562 | 2.566/2.579/2.584/2.574 |
| 7 | 85 | 924/924/924/924 | 11/10/11/12 | 27/26/28/28 | 6.444/6.731/5.857/5.679 | 2.562/2.568/2.570/2.559 |
| 7 | 165 | 880/880/880/880 | 12/12/13/13 | 26/27/30/29 | 7.346/7.185/5.733/6.483 | 2.638/2.639/2.640/2.636 |
| 10 | 1 | 278/278/277/278 | 13/12/12/12 | 33/33/34/32 | 6.515/7.030/6.912/6.562 | 2.594/2.601/2.603/2.597 |
| 10 | 85 | 288/288/288/288 | 11/10/11/12 | 27/26/28/28 | 6.444/6.731/6.000/5.679 | 2.593/2.593/2.599/2.585 |
| 10 | 165 | 741/741/741/741 | 12/12/13/13 | 55/56/59/58 | 3.655/3.643/3.085/3.414 | 2.285/2.286/2.288/2.284 |

The B/S comparison has lower MLP and lower useful overlap in all six cells while the model assigns B fewer cycles. On block 165 its longest DRAM chain increases from 12 to 13. None of the six cells has more than 59 useful-overlap samples per arm. The data provide little basis for ranking precise changes in a rare dependent-load path.

The two historical control executables also show variation between runs with exactly the same tool, tape and model parameters. Across those twelve repeated cells, the largest observed magnitude of a difference was five instructions and 45,180 modeled cycles (0.735% on W7 block 1 edge). The earlier W10/85 maximum past-L2 chain of 247 for edge and 288 for scalar became 288 for both. W7/1 edge moved from 363 to 284 while scalar remained at 284. Useful-overlap numerators and denominators remained identical in all twelve repeats. This repeat evidence limits precision on small cycle, transfer and maximum-chain differences. The cause of those differences remains unverified. [Current versus prior per-cell counts](control-replay.csv) · [Verified comparison](replay_check.py)

The previous nine-round local W4 combined/Lazy wall ratio was 0.79375 [0.73179, 0.86096], with median peak RSS ratio 0.72670. Those clocks cover concurrent complete decoding. The mechanism tested here is a plausible explanation for part of the scan gain and represents only three selected blocks. The incremental effect versus scalar requires a paired independent comparison. The campaign target still requires sustained i9 W7–10 throughput and no additional peak memory. [Local scorer](../v1p2-cap-counter-confirm/score/summary.txt)

## Reproduction

From the research checkout, using a **new** scratch directory and the original corpus, the default recipe extracts the exact four archived executables, verifies their provenance, re-extracts the three real inputs separately with each worker, checks their hashes and executes the 24-cell round. Set `MODE=build` to rebuild C and B from source. Rust stores build paths in the binaries. Rebuilds in a different directory therefore have distinct byte identities. Set `CHECK_ONLY=1` to verify the inputs, tools and workers without new measurements. This recipe imports the historical scorer for reproduction. A new scored comparison requires refreshing master before collection.

```bash
source /workspace/work/env.sh
source /workspace/work/proxy-tools/env.sh
study=research/astra-e/results/cap-counter-route-pricing
bash "$study/reproduce.sh" /workspace/work/astra-counter-trace-replay "$PWD/corpora/wikidata.bz2"
python3 -B "$study/report.py" "$PWD" "$study"
python3 -B "$study/replay_check.py" "$PWD"
```

The environment used GCC 14.2.0 and Valgrind 3.24.0 for the model. The historical tool is reused when present. A fresh tool build canonicalizes the debug source directory and verifies the recorded runtime SHA-256 values. The default archive replay preflight and reconstruction of that exact debugger binary were checked independently. [Build](build.sh) · [Historical driver](reproduce.sh) · [Capture](run.py) · [Metric definitions and checks](report.py)
