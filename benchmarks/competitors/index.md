# Against other decoders

mibz2 against pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`) on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1 is the other decoder taking longer than mibz2. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

The latest record: [`2a68359`](<https://github.com/lmmx/mibz2/commit/2a68359b1e09c620bfb056411b7a89f16a1251a6>) (2026-10-27).

**mibz2 against pbz2 0.2.0 and lbzip2 version 2.5**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40500845444>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Time against mibz2** is the decoder's wall (or CPU) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster. 1-thread runs are pinned to one CPU.

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 wall / mibz2 | pbz2 CPU / mibz2 | lbzip2 wall / mibz2 | lbzip2 CPU / mibz2 | peak RSS: mibz2, pbz2, lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.527 s | 137 | 17.8 | 0.98× (0.97–0.98) | 0.98× (0.97–0.98) | 2.27× (2.25–2.30) | 2.26× (2.24–2.28) | 27, 10, 8 MB | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.685 s | 306 | 29.2 | 0.99× (0.98–1.00) | 1.01× (1.00–1.02) | 1.48× (1.45–1.50) | 1.57× (1.55–1.59) | 76, 31, 39 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,338 | 2.1 | 0.76× (0.73–0.79) | 0.79× (0.76–0.81) | 6.78× (6.53–7.04) | 4.28× (4.13–4.43) | 34, 10, 6 MB | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,329 | 2.1 | 0.78× (0.77–0.79) | 0.80× (0.79–0.82) | 6.69× (6.58–6.81) | 4.59× (4.52–4.66) | 33, 7, 5 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.085 s | 232 | 10.8 | 1.01× (1.00–1.02) | 1.01× (1.00–1.02) | 1.20× (1.19–1.21) | 1.18× (1.17–1.18) | 18, 10, 7 MB | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.043 s | 461 | 19.4 | 1.04× (1.01–1.06) | 1.05× (1.03–1.08) | 1.19× (1.16–1.21) | 1.00× (0.98–1.02) | 42, 26, 27 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.192 s | 42 | 59.3 | 0.89× (0.87–0.92) | 0.90× (0.87–0.92) | 2.06× (2.00–2.12) | 1.82× (1.76–1.87) | 24, 11, 7 MB | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.085 s | 94 | 79.5 | 1.02× (0.99–1.04) | 1.11× (1.09–1.14) | 1.80× (1.76–1.83) | 1.76× (1.74–1.78) | 53, 32, 22 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.237 s | 40 | 60.9 | 0.87× (0.86–0.89) | 0.87× (0.86–0.89) | 2.09× (2.04–2.14) | 1.91× (1.88–1.94) | 24, 11, 7 MB | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.103 s | 93 | 85.2 | 1.00× (0.98–1.03) | 1.07× (1.03–1.10) | 1.97× (1.94–1.99) | 1.78× (1.73–1.82) | 59, 33, 26 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 820 | 3.5 | 0.66× (0.64–0.68) | 0.71× (0.70–0.73) | 4.15× (4.07–4.22) | 2.99× (2.93–3.05) | 24, 10, 5 MB | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 784 | 3.7 | 0.63× (0.62–0.64) | 0.68× (0.67–0.70) | 3.96× (3.91–4.00) | 3.08× (3.01–3.16) | 26, 6, 4 MB | yes |
| `text` (778 kB → 12 MB, 15.5×) | 1 | 0.100 s | 120 | 20.8 | 1.01× (1.00–1.02) | 1.01× (1.00–1.02) | 2.52× (2.51–2.53) | 2.28× (2.27–2.29) | 19, 10, 7 MB | yes |
| `text` (778 kB → 12 MB, 15.5×) | 4 | 0.050 s | 241 | 35.0 | 0.94× (0.91–0.96) | 1.04× (0.99–1.08) | 2.03× (1.97–2.09) | 1.59× (1.57–1.61) | 53, 27, 25 MB | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.15 | 2.16 | 0.02 | 0.685 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 3h42m ± 28m14s |
| 4 threads | 1h40m ± 14m03s |
| 4 threads, the sample's overlap gap as a fixed cost | 1h18m ± 11m02s |
| the scanner alone (its serial part) | 21m49s ± 41.6s |
| the workers alone, over 4 | 1h18m ± 11m02s |

At 4 threads the workers bound it. Per byte: the scanner 12.7 ns a byte of bz2, the workers 10.25 ns a byte of output at 4 threads (CPU, summed over them), 6.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 14 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 777,944 | 12,019,425 | \`05591211e20d28df…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0 (a dev-dependency, the harness's arm); lbzip2 version 2.5 - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
