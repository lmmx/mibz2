# Full 16-block calibration of the combined narrow endpoint, immediate words and lookahead

The scorer selected the middle ordinal of each of 16 equal spans from the 431-block Wikidata stream. Its regular calibration grid is W1, W2, W4, W7, W8 and W10. Each row models a single active worker at that W's cache share and memory latencies. The independent i9 schedule also includes W9. The scorer executes four reference arms (LazyCells, opend, open and eager Cells) and the candidate from the same binary. It warms the first sample outside the region, then measures all 16 starting after entropy decoding. Decoder work in the region includes inversion, RLE1 expansion and block CRC. [Raw counters](score/proxy.json) · [Scorer report and calibration](score/summary.txt)

## Modeled ratios

| W | Candidate / lazy cycles | Candidate / opend cycles | Candidate / opend instructions | Candidate / opend DRAM B/B | Candidate / opend dependent past-L2 links/B | Reference range |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 0.8898 | 0.9883 | 0.9769 | 0.9776 | 0.9988 | within |
| 2 | 0.8966 | 0.9876 | 0.9769 | 0.9776 | 0.9986 | within |
| 4 | 0.8904 | 0.9830 | 0.9769 | 0.9371 | 0.9985 | outside, dependent DRAM links/B |
| 7 | 0.8935 | 0.9507 | 0.9769 | 0.8630 | 0.9986 | within |
| 8 | 0.8795 | 0.9623 | 0.9769 | 0.9328 | 0.9973 | within |
| 10 | 0.8277 | 0.9633 | 0.9769 | 0.9633 | 0.9972 | within |

V0's W7, W8 and W10 point estimates favor the candidate over opend by 4.9%, 3.8% and 3.7%. It uses 2.3% fewer instructions/B. Dependent past-L2 links/B fall only 0.1–0.3%. The fitted W7 opend/lazy cell reproduces .940 and all held-out comparisons land on the recorded side of equality. The model recovers 6 of 9 held-out pair orderings. At W10 it assigns opend/lazy .859, well beyond the historical i9 ratio .933. The apparent candidate shortening at W10 inherits this model error. W4's dependent DRAM link count falls outside the reference envelope. The official gate passed 98 cases. This score includes neither process timing nor peak RSS. [Target comparisons](../../prior.md) · [Exact local rehearsal](../../workstation/rehearsals/narrow-short-0694800.tar.gz)

Scorer v1 landed after this score began. This receipt belongs to v0; v1 samples blocks and estimates latency differently.

## Populations

The 16 sampled ordinals are 13, 40, 67, 94, 121, 148, 175, 202, 228, 255, 282, 309, 336, 363, 390 and 417. The route counts below use the benchmark's run-front flag and its exact gate `20 × capped records < 3 × rows`. In the full sample the two arms that use this gate spend nearly every output byte in their Open kernel, even though six of sixteen blocks take other paths. The earlier two-block density stress sample gives close to half its output to the gated fallback. Its reversal against LazyCells should be interpreted with that population in mind. Fractions here weight output byte counts; elapsed work also depends on the per-block costs and schedules.

| Population | Admitted Open | Dense fallback | Column route | Admitted output share |
| --- | ---: | ---: | ---: | ---: |
| Frozen block 1 | 1 block, 900,000 B | 0 | 0 | 100.0000% |
| Density pair 41,85 | 1 block, 899,988 B | 1 block, 899,971 B | 0 | 50.0005% |
| Official 16 | 10 blocks, 8,999,668 B | 4 blocks, 82 B | 2 blocks, 18 B | 99.9989% |
| Entire Wikidata corpus | 228 blocks, 205,191,352 B | 155 blocks, 4,508,379 B | 48 blocks, 465 B | 97.8499% |

The population cross-check is reproducible with `python3 research/astra-e/results/combined-full-proxy/routes.py`. [Route receipt](routes.json) · [Source census](https://github.com/lmmx/mibz2/blob/4f7d0235025940bd3d744463152515d189eb13dd/docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv) · [Earlier W7 partial screen](../../proxy/narrow-short-ahead-w7/README.md)

## Reproduction

The build commit `b981d78984a195408501ff7fb495f77a1075be02` is the immediate parent of workstation recipe `0694800bf4b2ef5ece720f7c43b8ab0ed7dced9b`. Their sole source-tree difference is the seven lines registering the two workstation recipes in `.just/bench.just`. The four generated and included candidate files match the source hashes independently recorded at the recipe freeze. Recompilation in the detached parent checkout gives libbzip2 tests access to the live source tree from which the harness assembles its test fixture. The workstation recipe branch retains its registration. [Input, generated source, harness, tool and extracted block hashes](sources.json) · [Toolchain, scorer pin and commands](manifest.txt)

Use Rust 1.99.1 and Valgrind 3.24.0 with its static tool libraries, and place the frozen Wikidata compressed file at `corpora/wikidata.bz2`. In a fresh checkout of the build commit, the following command fetches the current master scorer and runs the gate and proxy. The recorded scorer path was `9c31461`, with no separate version printed. New scorer versions require scoring the reference arms and preserved leading candidate together.

```sh
nice -n 12 bash research/astra-e/score.sh combined-full-proxy \
  research/astra-e/narrow-short/ahead.rs --target-dir /tmp/astra-e-combined-full-target \
  --quick --no-timing --proxy-blocks 16
```
