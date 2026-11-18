# Timed inversion from ready run records

The opt-in `astra_e` example measures the complete `invert_runs` call, including construction, recovery of output positions and writing the RLE1-coded inverse. The input scanner, MTF front, independent byte comparison, RLE1 expansion, CRC, output drain and all other threads are outside the clock. Every arm lives in one process with its own reusable decoder state and output buffer. This stage experiment cannot measure per-arm peak RSS or the target workstation's concurrent block throughput.

The recipe verifies the pinned 209,700,196-byte-output Wikidata stream by compressed SHA-256 before launching the example. The example independently derives the eligible stream indices and checks the 11 frozen ordinals. Eligible blocks take the production run front, have at least 65,536 rows and satisfy `20 × capped records < 3 × rows`. Of these 228 blocks it takes the first and the midpoint of each decile. This omits refusals by design. Open widths 8, 16 and 32 all use the queued endpoint; on this admitted sample `opend16` is exactly opend. Eager controls and experimental arms run their own internal policies.

The first sweep warms every decoder and checks every result. Each recorded pass rotates arm order by pass and block number. Over eleven measurement passes every arm visits each position exactly once for each block. Each output must equal the precomputed LazyCells bytes after the clock stops. All raw per-call nanoseconds are retained. The report sums the 11 blocks per arm in each pass and then takes paired geometric ratios of those sums.

```sh
d=research/astra-e/stage
bash "$d/run.sh" corpora/wikidata.bz2 11 > laps.csv 2> manifest.txt
python3 "$d/report.py" laps.csv
```
