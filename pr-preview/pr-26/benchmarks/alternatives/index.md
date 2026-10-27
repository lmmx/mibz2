# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-26/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

| decoder | what runs | threads |
| --- | --- | --- |
| pbz2 | Soof Golan's Rust decoder: `DecoderReader`, or `ParallelDecoder::with_threads` | block-parallel |
| lbzip2 | the C program: `lbzip2 -dc -n T`, read through a pipe | block-parallel |
| indexed\_bzip2 | C++, from Python: `IndexedBzip2File(path, parallelization=T)` (`bench/ib.py`; the interpreter's start left out) | block-parallel |
| fbz | Answer.AI's Rust decoder: `fbz::Reader`, `threads: T` | block-parallel |
| lbzip2-rs | the `lbzip2` crate (pure Rust): `StreamingBz2Read`, on the CPUs it may run on (it takes no thread count) | block-parallel |
| libbzip2 | the reference C library, 1.0.8: the `bzip2` crate's bundled source, `MultiBzDecoder` (as `bzip2 -dc` and Python's `bz2` decode) | single-threaded |
| libbz2-rs | libbz2-rs-sys, the pure-Rust port of libbzip2, through the `bzip2` crate 0.6 | single-threaded |

**Single-threaded decoders run at 1 thread only.** libbzip2 and libbz2-rs have no thread count, so they never decode on 4 threads. At 4 threads, their figures are their 1-thread runs against mibz2's at 4 threads: in the chart, hollow points with dashed whiskers, under "single-threaded (1 thread)"; in the tables, marked ¹.

Not included: `bzip2 -dc` and Python's `bz2` module, which are libbzip2 itself (its arm stands for them); and bz2k, mibz2's own prototype, which it replaced.

The latest record: [`6aa70b1`](<https://github.com/lmmx/mibz2/commit/6aa70b141da1d2d36a60f9485680ba51895d0498>) (2026-10-27).

**mibz2 against pbz2 and lbzip2**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40520251997>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.805 s | 260 | 14.2 | 1.47× (1.46–1.49) | 3.40× (3.27–3.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.412 s | 508 | 24.4 | 1.33× (1.29–1.37) | 1.96× (1.91–2.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,587 | 2.7 | 0.73× (0.71–0.74) | 8.12× (7.92–8.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,619 | 2.6 | 0.77× (0.75–0.79) | 4.13× (4.05–4.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 s | 435 | 8.9 | 1.51× (1.48–1.54) | 2.23× (2.19–2.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 s | 671 | 19.9 | 1.24× (1.20–1.28) | 1.74× (1.71–1.77) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 57.6 | 1.20× (1.15–1.24) | 2.43× (2.34–2.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 78.7 | 1.21× (1.15–1.28) | 2.60× (2.46–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.158 s | 61 | 61.5 | 1.12× (1.09–1.15) | 2.53× (2.48–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 140 | 86.4 | 1.29× (1.21–1.38) | 2.20× (2.15–2.24) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,005 | 4.4 | 0.65× (0.62–0.69) | 5.16× (5.04–5.29) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 966 | 4.5 | 0.64× (0.60–0.68) | 4.94× (4.84–5.05) | yes |
| `text` (875 kB → 12.1 MB, 13.8×) | 1 | 0.064 s | 190 | 20.0 | 1.21× (1.16–1.26) | 3.12× (2.97–3.27) | yes |
| `text` (875 kB → 12.1 MB, 13.8×) | 4 | 0.033 s | 369 | 34.1 | 1.12× (1.07–1.18) | 3.07× (2.96–3.18) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 |
| --- | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.47× (1.45–1.49) | 3.36× (3.22–3.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.48× (1.45–1.51) | 2.23× (2.19–2.28) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.76× (0.75–0.78) | 3.94× (3.86–4.03) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.80× (0.78–0.81) | 4.31× (4.23–4.40) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.49× (1.46–1.52) | 1.72× (1.69–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.24× (1.20–1.29) | 1.13× (1.08–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.19× (1.15–1.23) | 2.29× (2.20–2.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.28–1.44) | 2.10× (1.98–2.23) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.12× (1.09–1.14) | 2.28× (2.24–2.32) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.36× (1.28–1.45) | 2.19× (2.15–2.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.69–0.74) | 2.82× (2.76–2.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.70× (0.66–0.75) | 2.96× (2.91–3.00) |
| `text` (875 kB → 12.1 MB, 13.8×) | 1 | 1.20× (1.15–1.25) | 2.78× (2.65–2.91) |
| `text` (875 kB → 12.1 MB, 13.8×) | 4 | 1.22× (1.18–1.27) | 2.09× (1.95–2.23) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 |
| --- | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 27 MB | 0.37× (0.37–0.38) | 0.34× (0.33–0.35) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.40–0.41) | 0.49× (0.47–0.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.30–0.30) | 0.17× (0.16–0.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.15× (0.15–0.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.57–0.58) | 0.41× (0.41–0.41) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.59–0.63) | 0.61× (0.57–0.65) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.42× (0.41–0.43) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.55× (0.54–0.56) | 0.43× (0.41–0.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.43× (0.41–0.45) | 0.21× (0.20–0.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.25× (0.25–0.25) | 0.18× (0.18–0.18) |
| `text` (875 kB → 12.1 MB, 13.8×) | 1 | 20 MB | 0.52× (0.51–0.53) | 0.37× (0.37–0.37) |
| `text` (875 kB → 12.1 MB, 13.8×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.46× (0.42–0.51) |

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

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 875,130 | 12,106,287 | \`fb686cd2815025e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 1.0.0 (bench/report.py), 10 rounds, threads 1, 4
