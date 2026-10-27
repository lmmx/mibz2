# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`137c251`](<https://github.com/lmmx/mibz2/commit/137c251c91cfbd7b74dec69545e1ad2b6bf875b3>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40594289144>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.642 s | 326 | 11.3 | 1.87× (1.83–1.91) | 4.37× (4.26–4.49) | 4.22× (4.12–4.32) | 5.10× (4.97–5.22) | 4.38× (4.27–4.50) | 4.87× (4.70–5.05) | 4.25× (4.10–4.42) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.336 s | 624 | 20.1 | 1.68× (1.64–1.73) | 2.53× (2.44–2.62) | 3.24× (3.15–3.34) | 2.96× (2.90–3.03) | 2.65× (2.59–2.72) | 9.40× (9.06–9.74) | 8.20× (7.97–8.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.012 s | 1,619 | 2.6 | 0.74× (0.72–0.76) | 8.23× (8.02–8.45) | 7.52× (7.34–7.69) | 4.74× (4.61–4.87) | 4.91× (4.79–5.05) | 3.82× (3.73–3.91) | 3.78× (3.70–3.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,660 | 2.6 | 0.81× (0.75–0.88) | 4.17× (4.01–4.33) | 7.57× (7.28–7.87) | 4.87× (4.68–5.06) | 5.06× (4.89–5.24) | 3.90× (3.76–4.05) | 3.86× (3.72–4.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 654 | 6.0 | 2.31× (2.28–2.34) | 3.37× (3.33–3.41) | 3.09× (3.04–3.14) | 3.62× (3.58–3.67) | 2.67× (2.64–2.70) | 3.01× (2.96–3.05) | 1.98× (1.96–2.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 860 | 14.2 | 1.54× (1.45–1.64) | 2.16× (2.04–2.29) | 3.94× (3.73–4.16) | 1.97× (1.82–2.13) | 1.68× (1.58–1.78) | 3.86× (3.65–4.09) | 2.55× (2.41–2.70) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 57.8 | 1.22× (1.21–1.24) | 2.45× (2.42–2.47) | 2.37× (2.34–2.40) | 2.25× (2.22–2.28) | 2.20× (2.18–2.23) | 2.83× (2.78–2.88) | 2.79× (2.76–2.82) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 77.5 | 1.29× (1.25–1.34) | 2.70× (2.61–2.79) | 2.54× (2.45–2.63) | 2.67× (2.52–2.82) | 2.12× (2.05–2.18) | 6.24× (6.04–6.45) | 6.15× (5.96–6.36) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.159 s | 60 | 62.0 | 1.14× (1.12–1.15) | 2.54× (2.51–2.56) | 2.16× (2.14–2.18) | 2.08× (2.06–2.09) | 2.11× (2.07–2.15) | 3.00× (2.98–3.03) | 3.04× (3.01–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 84.2 | 1.24× (1.20–1.29) | 2.29× (2.11–2.49) | 2.15× (1.98–2.34) | 2.79× (2.68–2.90) | 1.96× (1.90–2.03) | 6.79× (6.55–7.05) | 6.89× (6.65–7.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 985 | 4.5 | 0.65× (0.64–0.67) | 5.01× (4.90–5.12) | 4.20× (4.12–4.29) | 3.46× (3.38–3.55) | 3.78× (3.63–3.94) | 2.68× (2.62–2.73) | 2.64× (2.59–2.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 971 | 4.5 | 0.63× (0.61–0.65) | 4.94× (4.78–5.11) | 8.97× (8.68–9.27) | 3.49× (3.38–3.62) | 3.72× (3.59–3.86) | 2.66× (2.58–2.75) | 2.63× (2.55–2.72) | yes |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 1 | 0.062 s | 198 | 19.2 | 1.33× (1.29–1.37) | 3.24× (3.13–3.35) | 3.24× (3.05–3.44) | 3.31× (3.19–3.44) | 3.17× (3.06–3.27) | 3.52× (3.41–3.62) | 3.46× (3.36–3.56) | yes |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 4 | 0.031 s | 392 | 33.0 | 1.34× (1.24–1.44) | 3.23× (3.13–3.32) | 3.20× (2.81–3.65) | 3.33× (3.14–3.53) | 2.42× (2.32–2.52) | 6.97× (6.77–7.19) | 6.86× (6.66–7.07) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.87× (1.83–1.90) | 4.33× (4.21–4.45) | 4.17× (4.08–4.27) | 5.08× (4.96–5.21) | 4.38× (4.26–4.49) | 4.86× (4.69–5.04) | 4.24× (4.09–4.41) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.84× (1.78–1.90) | 2.82× (2.76–2.88) | 3.27× (3.18–3.36) | 3.43× (3.34–3.52) | 2.92× (2.85–2.99) | 2.77× (2.68–2.86) | 2.42× (2.36–2.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.78× (0.76–0.80) | 3.99× (3.92–4.07) | 4.82× (4.71–4.94) | 4.23× (4.14–4.32) | 4.38× (4.29–4.48) | 3.43× (3.37–3.50) | 3.40× (3.34–3.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.83× (0.78–0.89) | 4.33× (4.17–4.49) | 6.16× (5.96–6.35) | 4.37× (4.22–4.54) | 4.48× (4.34–4.62) | 3.49× (3.37–3.61) | 3.46× (3.34–3.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.23× (2.20–2.26) | 2.56× (2.53–2.59) | 2.81× (2.76–2.87) | 3.46× (3.42–3.51) | 2.57× (2.54–2.60) | 2.89× (2.84–2.93) | 1.93× (1.90–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.79× (1.72–1.86) | 1.60× (1.53–1.66) | 1.96× (1.88–2.05) | 1.93× (1.81–2.05) | 1.34× (1.28–1.39) | 1.21× (1.16–1.27) | 0.81× (0.78–0.85) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.22× (1.20–1.24) | 2.32× (2.29–2.36) | 2.00× (1.97–2.02) | 2.23× (2.20–2.26) | 2.18× (2.16–2.21) | 2.80× (2.75–2.85) | 2.76× (2.73–2.80) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.42–1.48) | 2.27× (2.21–2.34) | 1.95× (1.90–2.00) | 2.62× (2.58–2.66) | 2.22× (2.11–2.33) | 2.10× (2.06–2.13) | 2.07× (2.04–2.09) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.13× (1.12–1.15) | 2.29× (2.26–2.32) | 1.88× (1.86–1.91) | 2.06× (2.04–2.08) | 2.09× (2.05–2.13) | 2.98× (2.95–3.00) | 3.02× (2.98–3.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.39× (1.37–1.42) | 2.20× (2.13–2.26) | 1.82× (1.71–1.95) | 2.60× (2.55–2.65) | 2.12× (2.05–2.18) | 2.20× (2.16–2.24) | 2.23× (2.18–2.28) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.72) | 2.75× (2.69–2.81) | 3.47× (3.39–3.54) | 3.06× (2.98–3.14) | 3.33× (3.20–3.46) | 2.40× (2.35–2.46) | 2.38× (2.33–2.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.69× (0.66–0.71) | 3.05× (2.85–3.26) | 4.21× (4.04–4.38) | 3.17× (3.07–3.28) | 3.31× (3.19–3.43) | 2.40× (2.33–2.47) | 2.37× (2.30–2.45) |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 1 | 1.32× (1.28–1.36) | 2.95× (2.84–3.06) | 2.99× (2.90–3.09) | 3.25× (3.13–3.37) | 3.10× (3.00–3.20) | 3.44× (3.34–3.54) | 3.38× (3.29–3.48) |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 4 | 1.36× (1.30–1.42) | 2.23× (2.11–2.35) | 2.62× (2.50–2.76) | 2.70× (2.64–2.76) | 2.27× (2.21–2.33) | 2.03× (1.99–2.08) | 2.00× (1.96–2.05) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.36–0.38) | 0.34× (0.33–0.35) | 1.11× (1.11–1.12) | 0.85× (0.85–0.86) | 11.37× (11.35–11.40) | 0.31× (0.31–0.32) | 0.33× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.40–0.40) | 0.49× (0.48–0.51) | 0.74× (0.73–0.75) | 0.63× (0.62–0.64) | 4.10× (4.08–4.11) | 0.11× (0.11–0.11) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.31) | 0.17× (0.16–0.18) | 0.79× (0.78–0.80) | 0.81× (0.80–0.82) | 4.23× (4.20–4.26) | 0.25× (0.24–0.25) | 0.27× (0.26–0.29) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.15× (0.15–0.15) | 1.71× (1.68–1.73) | 0.80× (0.80–0.81) | 4.20× (4.18–4.22) | 0.24× (0.24–0.25) | 0.27× (0.25–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.56–0.58) | 0.41× (0.40–0.41) | 1.53× (1.52–1.53) | 0.66× (0.65–0.66) | 7.44× (7.41–7.47) | 0.59× (0.59–0.60) | 0.62× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.60–0.62) | 0.60× (0.58–0.63) | 1.06× (1.03–1.08) | 0.85× (0.84–0.87) | 3.12× (3.07–3.18) | 0.24× (0.24–0.25) | 0.25× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.10) | 0.72× (0.72–0.72) | 4.90× (4.87–4.93) | 0.35× (0.35–0.35) | 0.36× (0.36–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.59× (0.57–0.61) | 0.41× (0.41–0.42) | 0.78× (0.76–0.80) | 0.73× (0.69–0.77) | 2.29× (2.24–2.34) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.43–0.45) | 0.30× (0.30–0.30) | 1.10× (1.09–1.10) | 0.81× (0.81–0.81) | 4.92× (4.90–4.94) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.56× (0.54–0.57) | 0.43× (0.41–0.46) | 0.73× (0.72–0.75) | 0.75× (0.71–0.78) | 2.14× (2.09–2.19) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 25 MB | 0.42× (0.41–0.43) | 0.20× (0.19–0.21) | 1.07× (1.05–1.09) | 0.73× (0.70–0.75) | 5.09× (5.02–5.16) | 0.32× (0.32–0.33) | 0.34× (0.33–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.24–0.27) | 0.18× (0.18–0.18) | 1.47× (1.44–1.50) | 0.70× (0.67–0.72) | 4.77× (4.76–4.78) | 0.30× (0.30–0.30) | 0.32× (0.31–0.32) |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 1 | 20 MB | 0.52× (0.51–0.52) | 0.37× (0.37–0.37) | 1.37× (1.36–1.37) | 0.58× (0.57–0.60) | 6.28× (6.25–6.31) | 0.43× (0.43–0.43) | 0.45× (0.45–0.45) |
| `text` (1.24 MB → 12.3 MB, 9.9×) | 4 | 56 MB | 0.49× (0.46–0.51) | 0.45× (0.42–0.49) | 0.89× (0.87–0.92) | 0.63× (0.60–0.67) | 2.27× (2.20–2.35) | 0.15× (0.15–0.16) | 0.16× (0.15–0.16) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.89 | 0.02 | 0.336 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 11.4% |
| MTF + RLE2 | 2.09 | 20.7% |
| T-vector build | 1.72 | 17.0% |
| walk (the one each build uses) | 4.70 | 46.5% |
| of which: segment assembly | 0.15 | 1.5% |
| RLE1 | 0.19 | 1.8% |
| CRC | 0.26 | 2.5% |
| **total** | 10.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h33m ± 11m52s |
| 4 threads | 48m34s ± 6m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m07s ± 4m33s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 32m07s ± 4m33s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 4.21 ns a byte of output at 4 threads (CPU, summed over them), 2.73 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,237,135 | 12,272,710 | \`db687d4c8af32f81…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3693.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
