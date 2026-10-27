# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders, pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`), on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-27/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

The latest record: [`6aa70b1`](<https://github.com/lmmx/mibz2/commit/6aa70b141da1d2d36a60f9485680ba51895d0498>) (2026-10-27).

**mibz2 against pbz2 0.2.0 and lbzip2 version 2.5**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40520251997>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Time against mibz2** is the decoder's wall (or CPU) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster. 1-thread runs are pinned to one CPU.

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 wall / mibz2 | pbz2 CPU / mibz2 | lbzip2 wall / mibz2 | lbzip2 CPU / mibz2 | peak RSS: mibz2, pbz2, lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.805 s | 260 | 14.2 | 1.47× (1.46–1.49) | 1.47× (1.45–1.49) | 3.40× (3.27–3.55) | 3.36× (3.22–3.51) | 27, 10, 9 MB | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.412 s | 508 | 24.4 | 1.33× (1.29–1.37) | 1.48× (1.45–1.51) | 1.96× (1.91–2.00) | 2.23× (2.19–2.28) | 77, 31, 38 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,587 | 2.7 | 0.73× (0.71–0.74) | 0.76× (0.75–0.78) | 8.12× (7.92–8.33) | 3.94× (3.86–4.03) | 34, 10, 6 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,619 | 2.6 | 0.77× (0.75–0.79) | 0.80× (0.78–0.81) | 4.13× (4.05–4.21) | 4.31× (4.23–4.40) | 35, 7, 5 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 s | 435 | 8.9 | 1.51× (1.48–1.54) | 1.49× (1.46–1.52) | 2.23× (2.19–2.27) | 1.72× (1.69–1.75) | 18, 10, 7 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 s | 671 | 19.9 | 1.24× (1.20–1.28) | 1.24× (1.20–1.29) | 1.74× (1.71–1.77) | 1.13× (1.08–1.18) | 43, 26, 27 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 57.6 | 1.20× (1.15–1.24) | 1.19× (1.15–1.23) | 2.43× (2.34–2.52) | 2.29× (2.20–2.39) | 24, 11, 7 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 78.7 | 1.21× (1.15–1.28) | 1.36× (1.28–1.44) | 2.60× (2.46–2.75) | 2.10× (1.98–2.23) | 54, 32, 22 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.158 s | 61 | 61.5 | 1.12× (1.09–1.15) | 1.12× (1.09–1.14) | 2.53× (2.48–2.58) | 2.28× (2.24–2.32) | 25, 11, 7 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 140 | 86.4 | 1.29× (1.21–1.38) | 1.36× (1.28–1.45) | 2.20× (2.15–2.24) | 2.19× (2.15–2.23) | 60, 33, 27 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,005 | 4.4 | 0.65× (0.62–0.69) | 0.71× (0.69–0.74) | 5.16× (5.04–5.29) | 2.82× (2.76–2.88) | 24, 10, 5 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 966 | 4.5 | 0.64× (0.60–0.68) | 0.70× (0.66–0.75) | 4.94× (4.84–5.05) | 2.96× (2.91–3.00) | 26, 6, 5 MB | yes |
| `text` (875 kB → 12.1 MB, 13.8×) | 1 | 0.064 s | 190 | 20.0 | 1.21× (1.16–1.26) | 1.20× (1.15–1.25) | 3.12× (2.97–3.27) | 2.78× (2.65–2.91) | 20, 10, 7 MB | yes |
| `text` (875 kB → 12.1 MB, 13.8×) | 4 | 0.033 s | 369 | 34.1 | 1.12× (1.07–1.18) | 1.22× (1.18–1.27) | 3.07× (2.96–3.18) | 2.09× (1.95–2.23) | 54, 27, 26 MB | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.13 | 0.02 | 0.412 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h58m ± 14m38s |
| 4 threads | 1h00m ± 8m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 41m06s ± 5m53s |
| the scanner alone (its serial part) | 16m53s ± 1m07s |
| the workers alone, over 4 | 41m06s ± 5m53s |

At 4 threads the workers bound it. Per byte: the scanner 9.8 ns a byte of bz2, the workers 5.39 ns a byte of output at 4 threads (CPU, summed over them), 3.33 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 875,130 | 12,106,287 | \`fb686cd2815025e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0 (a dev-dependency, the harness's arm); lbzip2 version 2.5 - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
