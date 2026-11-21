# Timed inversion from ready run records

The opt-in `astra_e` example measures the complete `invert_runs` call, including construction, recovery of output positions and writing the RLE1-coded inverse. The input scanner, MTF front, independent byte comparison, RLE1 expansion, CRC, output drain and all other threads are outside the clock. Every arm lives in one process with its own reusable decoder state and output buffer. This stage experiment cannot measure per-arm peak RSS or the target workstation's concurrent block throughput.

The recipe verifies the pinned 209,700,196-byte-output Wikidata stream by compressed SHA-256 before launching the example. The example independently derives the eligible stream indices and checks the 11 frozen ordinals. Eligible blocks take the production run front, have at least 65,536 rows and satisfy `20 × capped records < 3 × rows`. Of these 228 blocks it takes the first and the midpoint of each decile. This omits refusals by design. Open widths 8, 16 and 32 all use the queued endpoint; on this admitted sample `opend16` is exactly opend. Eager controls and experimental arms run their own internal policies.

The first sweep warms every decoder and checks every result. Each recorded pass rotates arm order by pass and block number. Over eleven measurement passes every arm visits each position exactly once for each block. Each output must equal the precomputed LazyCells bytes after the clock stops. All raw per-call nanoseconds are retained. The report sums the 11 blocks per arm in each pass and then takes paired geometric ratios of those sums.

```sh
d=research/astra-e/stage
bash "$d/run.sh" corpora/wikidata.bz2 11 > laps.csv 2> manifest.txt
python3 "$d/report.py" laps.csv
```

## First recorded pass set

The run used example commit `0f25dee737432feafb03b8b14dc10afbf5c141ad`. The source digest of the compressed input and binary digest are in [manifest.txt](manifest.txt). All 1,331 recorded outputs and 121 warm-up outputs matched LazyCells. Every block/arm/round combination appears once in [laps.csv](laps.csv). Eleven pass sums per arm produce [summary.csv](summary.csv). [contrasts.csv](contrasts.csv) also gives log-t intervals across the eleven paired passes and a sensitivity statistic based on summing blockwise medians. These describe this one shared-process measurement.

| Inversion comparison | Geometric mean of paired time ratios | 95% log-t interval |
| --- | ---: | ---: |
| Shipped eager / opend16 | 0.831 | 0.754–0.915 |
| Opend16 / LazyCells | 0.877 | 0.846–0.909 |
| Page tracer / opend16 | 1.026 | 0.944–1.116 |
| Two-step / page tracer | 1.021 | 0.906–1.151 |
| Conjugate / opend16 | 1.057 | 0.943–1.185 |
| Opposed / opend16 | 1.390 | 1.146–1.687 |
| Packed eager / opend16 | 1.333 | 1.182–1.504 |
| Open width 8 / opend16 | 1.148 | 0.968–1.362 |
| Open width 32 / opend16 | 1.097 | 0.949–1.269 |

The shipped eager kernel had the best local stage result. Opend16 also beat LazyCells on every pass. Refining the tracer's partition almost halved its structural dependent probes across the separate ten-block structural sample, but the paired stage comparison found no advantage once compilation, byte spelling, residual wiring and writing were included. The opposed and packed eager arms had a material stage regression here. All three tested open widths incur the same caller and eligibility costs on these block indices. This run supplies no evidence favoring a threshold change. Whole-corpus worker and i9 comparisons remain separate.

```sh
python3 "$d/contrasts.py" "$d/laps.csv"
```
