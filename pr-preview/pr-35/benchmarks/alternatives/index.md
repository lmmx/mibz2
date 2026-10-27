# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-35/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`be3b66b`](<https://github.com/lmmx/mibz2/commit/be3b66b16c388062f5340cccb236052b911c289d>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40528212820>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 1.1.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.823 s | 255 | 14.5 | 1.48× (1.43–1.53) | 3.46× (3.37–3.56) | 3.34× (3.25–3.43) | 4.05× (3.89–4.21) | 3.47× (3.37–3.58) | 3.78× (3.66–3.89) | 3.34× (3.24–3.43) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.398 s | 527 | 25.2 | 1.45× (1.40–1.50) | 2.12× (2.04–2.20) | 2.71× (2.64–2.78) | 2.48× (2.43–2.53) | 2.28× (2.22–2.33) | 7.73× (7.61–7.84) | 6.82× (6.72–6.93) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.012 s | 1,606 | 2.6 | 0.76× (0.72–0.80) | 8.43× (7.97–8.91) | 7.66× (7.24–8.10) | 4.83× (4.58–5.09) | 5.00× (4.75–5.27) | 3.95× (3.67–4.24) | 3.85× (3.64–4.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,633 | 2.6 | 0.76× (0.73–0.79) | 4.35× (3.68–5.15) | 7.68× (6.90–8.56) | 4.76× (4.64–4.88) | 4.96× (4.83–5.10) | 3.85× (3.68–4.02) | 3.75× (3.65–3.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 s | 432 | 8.9 | 1.51× (1.48–1.55) | 2.21× (2.17–2.26) | 2.03× (1.98–2.07) | 2.38× (2.35–2.42) | 1.79× (1.72–1.86) | 2.00× (1.92–2.09) | 1.30× (1.28–1.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.030 s | 650 | 20.4 | 1.20× (1.12–1.27) | 1.65× (1.58–1.72) | 3.00× (2.87–3.13) | 1.50× (1.43–1.57) | 1.32× (1.26–1.37) | 3.00× (2.85–3.16) | 1.95× (1.87–2.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 63 | 59.8 | 1.19× (1.18–1.21) | 2.38× (2.37–2.40) | 2.31× (2.29–2.33) | 2.21× (2.19–2.23) | 2.16× (2.15–2.18) | 2.78× (2.74–2.83) | 2.74× (2.71–2.77) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 139 | 80.4 | 1.25× (1.21–1.30) | 2.60× (2.50–2.70) | 2.45× (2.36–2.55) | 2.61× (2.52–2.70) | 2.07× (1.99–2.15) | 6.07× (5.80–6.34) | 5.98× (5.77–6.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.162 s | 59 | 63.1 | 1.07× (0.98–1.17) | 2.39× (2.18–2.62) | 2.03× (1.85–2.23) | 1.97× (1.80–2.17) | 2.00× (1.83–2.19) | 2.83× (2.58–3.10) | 2.89× (2.64–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 138 | 86.7 | 1.23× (1.19–1.28) | 2.36× (2.14–2.60) | 2.16× (1.96–2.38) | 2.78× (2.71–2.85) | 1.93× (1.89–1.98) | 6.80× (6.62–6.98) | 6.96× (6.78–7.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 962 | 4.5 | 0.63× (0.60–0.66) | 4.93× (4.84–5.02) | 4.44× (3.73–5.28) | 3.41× (3.32–3.49) | 3.67× (3.61–3.72) | 2.63× (2.59–2.68) | 2.61× (2.56–2.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 930 | 4.7 | 0.60× (0.58–0.61) | 4.68× (4.57–4.78) | 8.49× (8.30–8.68) | 3.26× (3.18–3.34) | 3.57× (3.48–3.66) | 2.52× (2.46–2.57) | 2.49× (2.43–2.55) | yes |
| `text` (930 kB → 12.1 MB, 13.0×) | 1 | 0.062 s | 196 | 19.4 | 1.26× (1.23–1.29) | 3.24× (3.13–3.35) | 3.24× (3.01–3.49) | 3.39× (3.28–3.51) | 3.09× (2.99–3.20) | 3.23× (3.10–3.36) | 3.25× (3.14–3.35) | yes |
| `text` (930 kB → 12.1 MB, 13.0×) | 4 | 0.034 s | 358 | 34.2 | 1.13× (1.07–1.19) | 2.98× (2.87–3.09) | 2.95× (2.61–3.33) | 2.96× (2.84–3.08) | 2.16× (2.08–2.24) | 5.91× (5.68–6.14) | 5.93× (5.72–6.15) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.48× (1.43–1.53) | 3.42× (3.32–3.52) | 3.30× (3.22–3.39) | 4.04× (3.88–4.20) | 3.47× (3.36–3.57) | 3.77× (3.65–3.89) | 3.33× (3.23–3.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.49× (1.47–1.50) | 2.27× (2.20–2.34) | 2.62× (2.56–2.67) | 2.72× (2.66–2.78) | 2.38× (2.33–2.43) | 2.16× (2.11–2.20) | 1.91× (1.87–1.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.79× (0.75–0.83) | 4.11× (3.92–4.30) | 5.04× (4.70–5.41) | 4.30× (4.11–4.49) | 4.45× (4.26–4.66) | 3.54× (3.32–3.78) | 3.46× (3.30–3.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.79× (0.77–0.82) | 4.28× (4.10–4.48) | 6.08× (5.82–6.35) | 4.29× (4.19–4.39) | 4.40× (4.30–4.51) | 3.46× (3.31–3.61) | 3.38× (3.29–3.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.49× (1.46–1.53) | 1.71× (1.69–1.74) | 1.90× (1.86–1.93) | 2.33× (2.30–2.37) | 1.76× (1.69–1.83) | 1.96× (1.88–2.05) | 1.29× (1.27–1.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.24× (1.18–1.30) | 1.12× (1.06–1.18) | 1.38× (1.31–1.45) | 1.36× (1.29–1.44) | 0.96× (0.92–1.01) | 0.87× (0.81–0.94) | 0.57× (0.55–0.60) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.19× (1.18–1.20) | 2.28× (2.26–2.30) | 1.97× (1.94–1.99) | 2.19× (2.17–2.21) | 2.15× (2.13–2.16) | 2.76× (2.71–2.80) | 2.72× (2.69–2.75) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.38–1.42) | 2.16× (2.11–2.21) | 1.88× (1.84–1.92) | 2.52× (2.47–2.58) | 2.18× (2.09–2.27) | 2.04× (2.00–2.07) | 2.01× (1.96–2.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.07× (0.98–1.17) | 2.17× (1.99–2.37) | 1.79× (1.63–1.96) | 1.96× (1.79–2.15) | 1.99× (1.82–2.17) | 2.81× (2.56–3.07) | 2.87× (2.63–3.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.32× (1.28–1.37) | 2.24× (2.14–2.34) | 1.79× (1.69–1.89) | 2.54× (2.48–2.60) | 2.02× (1.99–2.06) | 2.13× (2.09–2.17) | 2.18× (2.15–2.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.66–0.71) | 2.72× (2.68–2.76) | 3.54× (3.41–3.66) | 3.02× (2.96–3.08) | 3.24× (3.21–3.28) | 2.37× (2.34–2.40) | 2.35× (2.32–2.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.65× (0.64–0.66) | 2.83× (2.75–2.91) | 3.99× (3.87–4.13) | 2.98× (2.92–3.06) | 3.19× (3.12–3.27) | 2.28× (2.23–2.33) | 2.26× (2.21–2.30) |
| `text` (930 kB → 12.1 MB, 13.0×) | 1 | 1.25× (1.23–1.28) | 2.89× (2.80–2.99) | 2.98× (2.86–3.11) | 3.32× (3.22–3.43) | 3.03× (2.93–3.13) | 3.16× (3.04–3.29) | 3.18× (3.08–3.28) |
| `text` (930 kB → 12.1 MB, 13.0×) | 4 | 1.18× (1.13–1.24) | 2.06× (1.99–2.13) | 2.48× (2.37–2.60) | 2.52× (2.44–2.59) | 2.13× (2.06–2.20) | 1.82× (1.77–1.87) | 1.83× (1.78–1.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.36–0.38) | 0.33× (0.33–0.34) | 1.11× (1.11–1.11) | 0.85× (0.85–0.85) | 11.37× (11.33–11.40) | 0.31× (0.31–0.32) | 0.32× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.40–0.41) | 0.49× (0.47–0.51) | 0.74× (0.73–0.75) | 0.64× (0.63–0.65) | 4.11× (4.05–4.16) | 0.11× (0.11–0.11) | 0.12× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.30) | 0.17× (0.16–0.19) | 0.79× (0.78–0.80) | 0.81× (0.79–0.82) | 4.21× (4.16–4.26) | 0.25× (0.24–0.26) | 0.25× (0.25–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.21× (0.19–0.22) | 0.15× (0.15–0.15) | 1.71× (1.69–1.73) | 0.81× (0.80–0.82) | 4.21× (4.19–4.23) | 0.25× (0.24–0.26) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.57–0.58) | 0.41× (0.40–0.41) | 1.53× (1.52–1.53) | 0.65× (0.65–0.66) | 7.38× (7.35–7.41) | 0.59× (0.59–0.59) | 0.61× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.62× (0.61–0.63) | 0.62× (0.60–0.64) | 1.08× (1.06–1.10) | 0.86× (0.85–0.87) | 3.18× (3.14–3.23) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.09) | 0.72× (0.72–0.72) | 4.89× (4.86–4.92) | 0.35× (0.35–0.35) | 0.36× (0.36–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.42× (0.41–0.43) | 0.79× (0.78–0.81) | 0.74× (0.71–0.77) | 2.33× (2.28–2.38) | 0.16× (0.16–0.16) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.45× (0.44–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.10) | 0.81× (0.81–0.81) | 4.93× (4.91–4.94) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.55× (0.54–0.57) | 0.43× (0.41–0.45) | 0.73× (0.71–0.75) | 0.74× (0.71–0.78) | 2.13× (2.08–2.18) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 25 MB | 0.42× (0.41–0.43) | 0.21× (0.20–0.22) | 1.04× (1.01–1.08) | 0.76× (0.72–0.79) | 5.14× (5.02–5.26) | 0.33× (0.32–0.34) | 0.34× (0.33–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.25× (0.25–0.25) | 0.18× (0.17–0.18) | 1.49× (1.46–1.51) | 0.68× (0.67–0.70) | 4.77× (4.75–4.79) | 0.30× (0.30–0.31) | 0.32× (0.31–0.32) |
| `text` (930 kB → 12.1 MB, 13.0×) | 1 | 20 MB | 0.52× (0.52–0.53) | 0.37× (0.37–0.37) | 1.38× (1.38–1.39) | 0.61× (0.60–0.62) | 6.32× (6.30–6.34) | 0.43× (0.43–0.43) | 0.45× (0.44–0.45) |
| `text` (930 kB → 12.1 MB, 13.0×) | 4 | 47 MB | 0.60× (0.58–0.62) | 0.50× (0.47–0.54) | 1.09× (1.07–1.11) | 0.84× (0.82–0.86) | 2.76× (2.73–2.78) | 0.18× (0.18–0.18) | 0.19× (0.19–0.19) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 1.13 | 0.02 | 0.398 |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h58m ± 15m34s |
| 4 threads | 57m53s ± 8m12s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m07s ± 6m14s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 42m07s ± 6m14s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 5.52 ns a byte of output at 4 threads (CPU, summed over them), 3.53 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 16 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 929,803 | 12,075,715 | \`02cc915e32c76b71…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3693.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 1.1.0 (bench/report.py), 10 rounds, threads 1, 4
