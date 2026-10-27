# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders, pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`), on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

The latest record: [`68e5942`](<https://github.com/lmmx/mibz2/commit/68e594248560bee658a1dbfcf22bcc7bb5c2bde0>) (2026-10-27).

**mibz2 against pbz2 0.2.0 and lbzip2 version 2.5**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40515875368>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Time against mibz2** is the decoder's wall (or CPU) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster. 1-thread runs are pinned to one CPU.

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 wall / mibz2 | pbz2 CPU / mibz2 | lbzip2 wall / mibz2 | lbzip2 CPU / mibz2 | peak RSS: mibz2, pbz2, lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.729 s | 288 | 15.2 | 1.19× (1.14–1.23) | 1.19× (1.14–1.23) | 3.03× (2.94–3.12) | 2.99× (2.90–3.08) | 26, 8, 9 MB | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.378 s | 555 | 27.1 | 1.10× (1.07–1.14) | 1.14× (1.10–1.19) | 1.90× (1.83–1.98) | 2.15× (2.08–2.21) | 72, 30, 68 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 3.9 | 0.57× (0.49–0.68) | 0.63× (0.52–0.76) | 3.16× (3.11–3.21) | 2.80× (2.75–2.84) | 33, 9, 6 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,250 | 3.9 | 0.49× (0.48–0.50) | 0.54× (0.53–0.55) | 3.15× (3.08–3.23) | 2.87× (2.80–2.94) | 32, 6, 5 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 539 | 8.5 | 1.56× (1.42–1.72) | 1.54× (1.39–1.71) | 2.81× (2.73–2.88) | 1.81× (1.72–1.91) | 17, 8, 7 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 764 | 20.8 | 1.10× (1.04–1.16) | 1.15× (1.12–1.19) | 1.93× (1.84–2.02) | 1.08× (1.04–1.12) | 40, 25, 26 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 s | 74 | 60.3 | 1.05× (1.04–1.07) | 1.05× (1.04–1.07) | 2.77× (2.71–2.84) | 2.37× (2.31–2.43) | 23, 9, 7 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.053 s | 150 | 85.2 | 1.04× (1.01–1.08) | 1.16× (1.11–1.22) | 2.52× (2.18–2.91) | 2.12× (2.03–2.21) | 51, 31, 22 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.138 s | 69 | 63.8 | 0.94× (0.90–0.98) | 0.94× (0.90–0.98) | 2.54× (2.50–2.59) | 2.32× (2.28–2.36) | 23, 9, 7 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 149 | 91.8 | 1.03× (0.99–1.07) | 1.11× (1.06–1.16) | 2.37× (2.32–2.42) | 2.16× (2.05–2.27) | 57, 32, 26 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 926 | 5.5 | 0.57× (0.56–0.58) | 0.63× (0.62–0.64) | 4.66× (4.62–4.71) | 2.31× (2.28–2.35) | 23, 8, 5 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 897 | 5.6 | 0.47× (0.46–0.48) | 0.54× (0.53–0.56) | 4.48× (4.37–4.59) | 2.28× (2.22–2.34) | 23, 5, 5 MB | yes |
| `text` (856 kB → 12.1 MB, 14.1×) | 1 | 0.055 s | 218 | 20.7 | 1.12× (1.06–1.18) | 1.12× (1.06–1.18) | 2.91× (2.69–3.16) | 2.62× (2.54–2.71) | 18, 9, 7 MB | yes |
| `text` (856 kB → 12.1 MB, 14.1×) | 4 | 0.033 s | 363 | 38.0 | 0.91× (0.83–0.99) | 1.01× (0.96–1.07) | 2.28× (1.76–2.96) | 1.78× (1.70–1.87) | 46, 26, 23 MB | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.01 | 0.01 | 0.378 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h47m ± 13m24s |
| 4 threads | 55m55s ± 8m11s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m26s ± 5m48s |
| the scanner alone (its serial part) | 15m43s ± 26.3s |
| the workers alone, over 4 | 37m26s ± 5m48s |

At 4 threads the workers bound it. Per byte: the scanner 9.1 ns a byte of bz2, the workers 4.91 ns a byte of output at 4 threads (CPU, summed over them), 3.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 855,578 | 12,074,837 | \`c5bc082b847602ba…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4370.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0 (a dev-dependency, the harness's arm); lbzip2 version 2.5 - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
