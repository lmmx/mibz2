# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders, pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`), on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

The latest record: [`7574f2c`](<https://github.com/lmmx/mibz2/commit/7574f2c4410800ea4b1d66d05b5f35f03bff29d7>) (2026-10-27).

**mibz2 against pbz2 0.2.0 and lbzip2 version 2.5**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40526627798>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Time against mibz2** is the decoder's wall (or CPU) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster. 1-thread runs are pinned to one CPU.

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 wall / mibz2 | pbz2 CPU / mibz2 | lbzip2 wall / mibz2 | lbzip2 CPU / mibz2 | peak RSS: mibz2, pbz2, lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.775 s | 271 | 13.7 | 1.51× (1.48–1.54) | 1.51× (1.48–1.54) | 3.56× (3.47–3.65) | 3.53× (3.44–3.62) | 28, 10, 9 MB | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.392 s | 535 | 25.0 | 1.42× (1.39–1.45) | 1.47× (1.44–1.50) | 2.06× (2.02–2.09) | 2.23× (2.20–2.26) | 77, 31, 38 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,587 | 2.7 | 0.73× (0.70–0.76) | 0.77× (0.74–0.79) | 8.12× (7.86–8.40) | 3.98× (3.88–4.08) | 34, 10, 6 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,633 | 2.6 | 0.79× (0.74–0.84) | 0.81× (0.78–0.85) | 4.13× (4.07–4.19) | 4.28× (4.23–4.33) | 35, 7, 5 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 s | 433 | 8.9 | 1.53× (1.50–1.55) | 1.50× (1.49–1.51) | 2.24× (2.22–2.26) | 1.73× (1.72–1.75) | 18, 10, 7 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.030 s | 656 | 20.2 | 1.20× (1.13–1.27) | 1.19× (1.13–1.26) | 1.67× (1.60–1.73) | 1.13× (1.06–1.20) | 43, 26, 27 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.126 s | 64 | 58.9 | 1.21× (1.18–1.25) | 1.21× (1.17–1.25) | 2.40× (2.39–2.42) | 2.28× (2.27–2.29) | 24, 11, 7 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 80.1 | 1.24× (1.20–1.29) | 1.39× (1.36–1.42) | 2.63× (2.55–2.71) | 2.17× (2.14–2.20) | 54, 32, 23 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.162 s | 59 | 63.1 | 1.11× (1.09–1.13) | 1.11× (1.09–1.13) | 2.49× (2.47–2.51) | 2.24× (2.22–2.26) | 25, 11, 7 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 138 | 86.1 | 1.25× (1.20–1.30) | 1.34× (1.29–1.40) | 2.17× (2.11–2.23) | 2.19× (2.15–2.23) | 59, 33, 23 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 990 | 4.5 | 0.65× (0.63–0.67) | 0.70× (0.69–0.72) | 5.05× (4.93–5.17) | 2.78× (2.70–2.86) | 24, 10, 5 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 4.5 | 0.64× (0.61–0.67) | 0.69× (0.67–0.72) | 4.98× (4.85–5.12) | 2.95× (2.88–3.02) | 26, 6, 5 MB | yes |
| `text` (897 kB → 12.1 MB, 13.5×) | 1 | 0.061 s | 198 | 19.2 | 1.27× (1.25–1.29) | 1.26× (1.24–1.28) | 3.33× (3.27–3.39) | 2.96× (2.91–3.02) | 20, 10, 7 MB | yes |
| `text` (897 kB → 12.1 MB, 13.5×) | 4 | 0.032 s | 372 | 32.3 | 1.17× (1.13–1.22) | 1.29× (1.24–1.34) | 3.13× (3.07–3.18) | 2.12× (2.05–2.19) | 46, 27, 24 MB | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 1.12 | 0.02 | 0.392 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h54m ± 14m46s |
| 4 threads | 57m36s ± 8m08s |
| 4 threads, the sample's overlap gap as a fixed cost | 41m37s ± 6m07s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 41m37s ± 6m07s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 5.46 ns a byte of output at 4 threads (CPU, summed over them), 3.41 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 16 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 896,995 | 12,067,111 | \`a3f2c5c9e5f9eafe…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3692.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0 (a dev-dependency, the harness's arm); lbzip2 version 2.5 - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
