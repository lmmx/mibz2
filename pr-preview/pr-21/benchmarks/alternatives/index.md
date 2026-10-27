# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders, pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`), on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-21/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

The latest record: [`935008e`](<https://github.com/lmmx/mibz2/commit/935008ef555807f3cdda6ca4a079f1ed95dd6f1c>) (2026-10-27).

**mibz2 against pbz2 0.2.0 and lbzip2 version 2.5**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40503297184>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Time against mibz2** is the decoder's wall (or CPU) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster. 1-thread runs are pinned to one CPU.

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 wall / mibz2 | pbz2 CPU / mibz2 | lbzip2 wall / mibz2 | lbzip2 CPU / mibz2 | peak RSS: mibz2, pbz2, lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.342 s | 156 | 15.7 | 1.13× (1.11–1.14) | 1.13× (1.11–1.14) | 2.59× (2.56–2.61) | 2.56× (2.53–2.59) | 27, 10, 8 MB | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.580 s | 362 | 23.0 | 1.21× (1.17–1.24) | 1.30× (1.26–1.34) | 1.87× (1.80–1.94) | 2.07× (2.02–2.12) | 76, 31, 39 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,282 | 2.2 | 0.72× (0.70–0.74) | 0.75× (0.73–0.77) | 6.44× (6.24–6.64) | 4.12× (4.01–4.24) | 34, 10, 5 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,294 | 2.2 | 0.78× (0.76–0.80) | 0.80× (0.78–0.83) | 6.53× (6.33–6.73) | 4.50× (4.38–4.63) | 34, 7, 5 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.087 s | 226 | 11.1 | 1.13× (1.09–1.17) | 1.13× (1.09–1.17) | 1.17× (1.16–1.18) | 1.15× (1.14–1.16) | 18, 10, 7 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.035 s | 556 | 16.2 | 1.27× (1.22–1.31) | 1.30× (1.27–1.33) | 1.42× (1.38–1.47) | 1.21× (1.18–1.25) | 41, 26, 28 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.189 s | 42 | 58.4 | 0.95× (0.92–0.97) | 0.95× (0.92–0.97) | 2.10× (2.05–2.16) | 1.86× (1.81–1.91) | 24, 11, 7 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.080 s | 100 | 73.2 | 1.14× (1.10–1.17) | 1.26× (1.19–1.33) | 1.89× (1.86–1.92) | 1.95× (1.90–2.00) | 53, 33, 22 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.232 s | 41 | 59.7 | 0.94× (0.93–0.95) | 0.94× (0.93–0.95) | 2.12× (2.04–2.20) | 1.97× (1.93–2.00) | 24, 11, 7 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.098 s | 98 | 79.8 | 1.12× (1.09–1.15) | 1.22× (1.20–1.23) | 2.13× (2.02–2.24) | 1.98× (1.94–2.01) | 58, 33, 26 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 816 | 3.5 | 0.69× (0.67–0.71) | 0.73× (0.71–0.75) | 4.11× (3.98–4.25) | 2.97× (2.89–3.05) | 24, 10, 5 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 778 | 3.8 | 0.65× (0.64–0.66) | 0.69× (0.69–0.70) | 3.93× (3.87–3.99) | 3.05× (2.99–3.11) | 26, 6, 4 MB | yes |
| `text` (844 kB → 12 MB, 14.3×) | 1 | 0.097 s | 125 | 20.0 | 1.13× (1.11–1.14) | 1.12× (1.11–1.14) | 2.62× (2.59–2.66) | 2.40× (2.38–2.42) | 19, 10, 7 MB | yes |
| `text` (844 kB → 12 MB, 14.3×) | 4 | 0.046 s | 262 | 32.0 | 1.12× (1.09–1.15) | 1.25× (1.20–1.30) | 2.22× (2.17–2.27) | 1.93× (1.85–2.01) | 53, 27, 24 MB | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.15 | 1.64 | 0.02 | 0.580 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 3h16m ± 24m36s |
| 4 threads | 1h24m ± 12m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 1h00m ± 9m03s |
| the scanner alone (its serial part) | 22m24s ± 32.2s |
| the workers alone, over 4 | 1h00m ± 9m03s |

At 4 threads the workers bound it. Per byte: the scanner 13.0 ns a byte of bz2, the workers 7.93 ns a byte of output at 4 threads (CPU, summed over them), 5.71 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 843,950 | 12,040,337 | \`e83affa50bed1994…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0 (a dev-dependency, the harness's arm); lbzip2 version 2.5 - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
