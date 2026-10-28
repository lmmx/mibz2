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

The latest record: [`2fa57b4`](<https://github.com/lmmx/mibz2/commit/2fa57b405d1f3696521948db3ec32dff0bfc91a0>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40627232313>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.0.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.770 s | 272 | 11.9 | 1.89× (1.85–1.94) | 4.45× (4.35–4.55) | 4.31× (4.21–4.43) | 5.05× (4.93–5.17) | 3.99× (3.90–4.08) | 4.53× (4.40–4.67) | 4.37× (4.22–4.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.434 s | 483 | 12.2 | 1.79× (1.73–1.86) | 3.81× (3.65–3.98) | 4.56× (4.42–4.72) | 4.41× (4.26–4.56) | 3.78× (3.68–3.89) | 7.99× (7.71–8.29) | 7.70× (7.35–8.08) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.372 s | 564 | 19.3 | 1.79× (1.76–1.82) | 2.72× (2.65–2.80) | 3.51× (3.43–3.60) | 3.08× (3.02–3.14) | 2.83× (2.77–2.89) | 9.46× (9.22–9.71) | 9.12× (8.81–9.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,562 | 2.4 | 0.85× (0.83–0.87) | 5.68× (5.41–5.97) | 7.17× (6.93–7.41) | 5.20× (5.08–5.33) | 5.57× (5.42–5.72) | 4.36× (4.24–4.48) | 4.08× (3.97–4.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,429 | 2.6 | 0.82× (0.79–0.84) | 4.39× (4.29–4.49) | 7.33× (7.13–7.53) | 4.79× (4.70–4.89) | 5.08× (4.97–5.19) | 3.99× (3.91–4.08) | 3.74× (3.66–3.82) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,418 | 2.6 | 0.82× (0.81–0.84) | 4.39× (4.30–4.48) | 7.32× (7.20–7.44) | 4.77× (4.68–4.86) | 5.10× (4.99–5.21) | 3.98× (3.90–4.06) | 3.73× (3.65–3.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 521 | 6.5 | 2.50× (2.43–2.57) | 2.72× (2.68–2.76) | 3.20× (3.12–3.27) | 3.51× (3.47–3.55) | 2.46× (2.44–2.48) | 2.19× (2.17–2.21) | 1.83× (1.81–1.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 s | 700 | 8.3 | 1.74× (1.49–2.02) | 2.14× (1.88–2.44) | 2.97× (2.60–3.40) | 2.64× (2.31–3.02) | 2.03× (1.81–2.28) | 2.82× (2.50–3.19) | 2.36× (2.08–2.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 727 | 15.6 | 1.71× (1.67–1.75) | 1.92× (1.84–1.99) | 2.71× (2.65–2.76) | 2.11× (2.07–2.15) | 1.83× (1.80–1.85) | 3.06× (3.01–3.12) | 2.56× (2.52–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.148 s | 54 | 60.7 | 1.19× (1.14–1.24) | 2.42× (2.39–2.44) | 2.07× (2.04–2.10) | 2.19× (2.17–2.22) | 2.14× (2.11–2.16) | 2.75× (2.72–2.78) | 2.82× (2.80–2.85) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.090 s | 89 | 66.0 | 1.08× (1.04–1.13) | 2.37× (2.18–2.57) | 2.14× (2.03–2.25) | 2.75× (2.61–2.90) | 2.03× (1.96–2.11) | 4.49× (4.32–4.67) | 4.61× (4.44–4.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 112 | 85.6 | 1.22× (1.19–1.24) | 2.03× (1.96–2.09) | 1.97× (1.90–2.04) | 2.50× (2.40–2.61) | 2.06× (2.01–2.11) | 5.79× (5.68–5.91) | 5.95× (5.84–6.07) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.197 s | 49 | 67.0 | 1.04× (1.03–1.04) | 2.36× (2.33–2.39) | 1.84× (1.81–1.87) | 1.96× (1.94–1.98) | 1.97× (1.96–1.98) | 2.80× (2.78–2.81) | 2.92× (2.90–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.114 s | 84 | 72.5 | 0.98× (0.93–1.04) | 2.27× (1.99–2.59) | 1.86× (1.76–1.97) | 2.75× (2.60–2.90) | 1.86× (1.76–1.97) | 4.68× (4.45–4.93) | 4.89× (4.65–5.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.090 s | 106 | 96.0 | 1.13× (1.10–1.16) | 1.95× (1.90–2.01) | 1.81× (1.76–1.86) | 2.60× (2.51–2.70) | 1.83× (1.78–1.88) | 6.07× (5.92–6.22) | 6.34× (6.16–6.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 976 | 3.9 | 0.78× (0.76–0.80) | 4.21× (3.96–4.48) | 5.16× (5.02–5.29) | 3.88× (3.80–3.97) | 4.48× (4.37–4.60) | 3.09× (3.02–3.16) | 2.91× (2.85–2.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 840 | 4.5 | 0.67× (0.65–0.68) | 3.48× (3.40–3.56) | 5.18× (5.06–5.30) | 3.41× (3.33–3.50) | 3.83× (3.76–3.89) | 2.69× (2.63–2.75) | 2.53× (2.48–2.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 851 | 4.4 | 0.68× (0.66–0.70) | 3.53× (3.43–3.64) | 5.32× (5.13–5.51) | 3.59× (3.27–3.95) | 3.92× (3.80–4.03) | 2.71× (2.64–2.79) | 2.55× (2.48–2.63) | yes |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 1 | 0.081 s | 150 | 22.0 | 1.39× (1.36–1.41) | 3.00× (2.96–3.05) | 3.18× (3.12–3.24) | 3.09× (3.00–3.18) | 2.76× (2.71–2.80) | 3.47× (3.40–3.54) | 3.38× (3.31–3.44) | yes |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 2 | 0.051 s | 239 | 25.2 | 1.16× (1.04–1.29) | 2.49× (2.21–2.80) | 3.14× (2.78–3.55) | 3.40× (2.85–4.04) | 2.45× (2.20–2.72) | 5.32× (4.77–5.94) | 5.17× (4.64–5.77) | yes |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 4 | 0.041 s | 296 | 36.3 | 1.25× (1.17–1.34) | 2.08× (1.97–2.20) | 2.69× (2.55–2.84) | 3.03× (2.90–3.16) | 2.17× (2.07–2.28) | 6.89× (6.66–7.14) | 6.70× (6.47–6.94) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.89× (1.85–1.93) | 4.44× (4.34–4.54) | 4.30× (4.19–4.41) | 5.04× (4.92–5.16) | 3.98× (3.89–4.08) | 4.52× (4.39–4.66) | 4.36× (4.21–4.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.93× (1.87–1.99) | 4.13× (4.03–4.24) | 4.64× (4.54–4.74) | 4.85× (4.72–4.99) | 4.08× (3.98–4.18) | 4.43× (4.29–4.58) | 4.27× (4.11–4.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.95× (1.93–1.98) | 3.12× (3.08–3.16) | 3.58× (3.50–3.66) | 3.58× (3.56–3.60) | 3.12× (3.09–3.14) | 2.81× (2.75–2.87) | 2.71× (2.64–2.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.87× (0.85–0.89) | 4.98× (4.83–5.14) | 5.96× (5.75–6.17) | 4.69× (4.57–4.82) | 5.02× (4.87–5.17) | 3.95× (3.84–4.08) | 3.71× (3.59–3.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.84× (0.81–0.87) | 4.36× (4.27–4.46) | 6.26× (6.13–6.39) | 4.30× (4.21–4.39) | 4.53× (4.43–4.63) | 3.57× (3.49–3.65) | 3.35× (3.27–3.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.84× (0.82–0.85) | 4.95× (4.84–5.06) | 6.42× (6.31–6.54) | 4.39× (4.30–4.47) | 4.61× (4.52–4.71) | 3.64× (3.56–3.71) | 3.41× (3.34–3.48) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.43× (2.36–2.50) | 2.57× (2.54–2.60) | 2.93× (2.88–2.97) | 3.40× (3.36–3.44) | 2.40× (2.38–2.42) | 2.14× (2.11–2.16) | 1.80× (1.77–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.90× (1.83–1.97) | 2.15× (2.10–2.21) | 2.62× (2.58–2.66) | 2.80× (2.73–2.86) | 1.96× (1.91–2.01) | 1.67× (1.63–1.72) | 1.41× (1.37–1.44) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.79× (1.76–1.83) | 1.64× (1.58–1.69) | 1.94× (1.89–1.98) | 1.91× (1.86–1.96) | 1.37× (1.33–1.40) | 0.89× (0.86–0.91) | 0.75× (0.73–0.77) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.18× (1.13–1.24) | 2.38× (2.34–2.41) | 2.03× (2.00–2.06) | 2.18× (2.16–2.21) | 2.12× (2.10–2.15) | 2.73× (2.70–2.76) | 2.80× (2.77–2.83) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.12× (1.11–1.12) | 2.31× (2.28–2.34) | 2.01× (2.00–2.03) | 2.71× (2.64–2.78) | 2.04× (2.02–2.06) | 2.52× (2.51–2.53) | 2.59× (2.58–2.60) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.35–1.38) | 2.17× (2.12–2.22) | 1.79× (1.74–1.84) | 2.51× (2.44–2.59) | 2.10× (2.05–2.15) | 1.94× (1.92–1.96) | 1.99× (1.97–2.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.04× (1.04–1.04) | 2.33× (2.30–2.35) | 1.80× (1.77–1.82) | 1.95× (1.93–1.97) | 1.96× (1.95–1.97) | 2.78× (2.77–2.79) | 2.90× (2.88–2.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.01× (1.00–1.03) | 2.24× (2.20–2.29) | 1.79× (1.78–1.81) | 2.60× (2.56–2.64) | 1.88× (1.86–1.90) | 2.57× (2.55–2.60) | 2.69× (2.66–2.71) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.26× (1.25–1.28) | 2.12× (2.08–2.16) | 1.57× (1.54–1.60) | 2.38× (2.32–2.44) | 1.91× (1.88–1.94) | 1.94× (1.91–1.97) | 2.02× (1.99–2.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.81× (0.79–0.82) | 3.51× (3.46–3.57) | 4.34× (4.23–4.45) | 3.46× (3.39–3.54) | 3.97× (3.88–4.07) | 2.79× (2.72–2.85) | 2.63× (2.58–2.68) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.72× (0.70–0.73) | 2.98× (2.93–3.03) | 4.30× (4.21–4.40) | 3.09× (3.03–3.15) | 3.39× (3.35–3.43) | 2.40× (2.35–2.44) | 2.26× (2.22–2.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.72× (0.70–0.74) | 3.44× (3.34–3.54) | 4.47× (4.37–4.58) | 3.35× (3.06–3.67) | 3.57× (3.46–3.67) | 2.49× (2.42–2.56) | 2.35× (2.28–2.42) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 1 | 1.38× (1.36–1.40) | 2.92× (2.87–2.97) | 3.05× (2.98–3.12) | 3.05× (2.96–3.14) | 2.72× (2.68–2.77) | 3.42× (3.35–3.49) | 3.33× (3.26–3.39) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 2 | 1.30× (1.27–1.33) | 2.62× (2.58–2.65) | 2.94× (2.90–2.98) | 3.09× (3.04–3.13) | 2.50× (2.48–2.53) | 3.00× (2.96–3.04) | 2.92× (2.88–2.96) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 4 | 1.38× (1.32–1.44) | 2.14× (2.08–2.21) | 2.51× (2.42–2.60) | 2.61× (2.52–2.69) | 2.13× (2.08–2.19) | 2.10× (2.06–2.15) | 2.04× (2.00–2.09) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 27 MB | 0.37× (0.37–0.37) | 0.31× (0.30–0.32) | 1.13× (1.12–1.13) | 0.84× (0.82–0.85) | 11.50× (11.46–11.53) | 0.30× (0.30–0.30) | 0.31× (0.31–0.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.37× (0.37–0.38) | 0.38× (0.37–0.39) | 0.80× (0.79–0.81) | 0.65× (0.65–0.65) | 6.03× (5.99–6.07) | 0.16× (0.16–0.16) | 0.16× (0.16–0.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.40–0.40) | 0.50× (0.48–0.52) | 0.74× (0.72–0.75) | 0.61× (0.61–0.62) | 4.08× (4.06–4.10) | 0.11× (0.11–0.11) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.29× (0.29–0.29) | 0.17× (0.17–0.18) | 0.79× (0.79–0.80) | 0.80× (0.79–0.81) | 4.24× (4.21–4.28) | 0.24× (0.24–0.24) | 0.25× (0.24–0.25) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 35 MB | 0.21× (0.20–0.23) | 0.14× (0.14–0.15) | 1.73× (1.73–1.73) | 0.80× (0.79–0.81) | 4.17× (4.17–4.18) | 0.24× (0.24–0.24) | 0.24× (0.24–0.25) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.14× (0.14–0.14) | 1.72× (1.70–1.74) | 0.80× (0.79–0.81) | 4.19× (4.18–4.20) | 0.24× (0.24–0.24) | 0.24× (0.24–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.56× (0.56–0.57) | 0.40× (0.40–0.40) | 1.55× (1.55–1.56) | 0.61× (0.59–0.63) | 7.48× (7.45–7.51) | 0.58× (0.58–0.58) | 0.59× (0.59–0.60) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 25 MB | 0.62× (0.62–0.63) | 0.53× (0.49–0.57) | 1.44× (1.42–1.46) | 0.84× (0.83–0.86) | 5.29× (5.21–5.37) | 0.41× (0.40–0.41) | 0.42× (0.41–0.42) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.62× (0.60–0.64) | 0.65× (0.63–0.68) | 1.08× (1.06–1.10) | 0.86× (0.85–0.87) | 3.19× (3.15–3.24) | 0.24× (0.24–0.24) | 0.25× (0.24–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.50× (0.50–0.50) | 0.30× (0.30–0.30) | 1.10× (1.10–1.11) | 0.70× (0.68–0.72) | 4.91× (4.89–4.93) | 0.34× (0.34–0.34) | 0.35× (0.34–0.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.52× (0.52–0.53) | 0.38× (0.34–0.41) | 0.91× (0.90–0.92) | 0.75× (0.74–0.76) | 3.23× (3.19–3.28) | 0.22× (0.21–0.22) | 0.22× (0.22–0.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.41× (0.41–0.42) | 0.79× (0.78–0.80) | 0.74× (0.70–0.77) | 2.32× (2.28–2.36) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.50× (0.50–0.50) | 0.30× (0.30–0.30) | 1.11× (1.11–1.12) | 0.78× (0.77–0.80) | 4.97× (4.95–4.99) | 0.34× (0.33–0.34) | 0.34× (0.34–0.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.50× (0.49–0.51) | 0.36× (0.33–0.39) | 0.86× (0.85–0.87) | 0.69× (0.66–0.72) | 3.03× (2.98–3.08) | 0.20× (0.20–0.20) | 0.21× (0.20–0.21) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.56× (0.55–0.58) | 0.45× (0.43–0.48) | 0.73× (0.72–0.75) | 0.74× (0.71–0.77) | 2.14× (2.09–2.19) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.42× (0.41–0.43) | 0.20× (0.19–0.21) | 1.07× (1.04–1.10) | 0.77× (0.75–0.80) | 5.29× (5.17–5.41) | 0.32× (0.32–0.33) | 0.34× (0.33–0.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 26 MB | 0.26× (0.25–0.27) | 0.17× (0.17–0.17) | 1.48× (1.46–1.51) | 0.70× (0.68–0.72) | 4.75× (4.73–4.78) | 0.29× (0.29–0.29) | 0.30× (0.30–0.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.25× (0.25–0.26) | 0.17× (0.17–0.17) | 1.50× (1.47–1.52) | 0.69× (0.67–0.71) | 4.77× (4.75–4.80) | 0.29× (0.29–0.30) | 0.30× (0.30–0.31) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 1 | 20 MB | 0.51× (0.51–0.51) | 0.37× (0.37–0.37) | 1.38× (1.38–1.39) | 0.61× (0.58–0.63) | 6.30× (6.27–6.32) | 0.42× (0.41–0.42) | 0.43× (0.42–0.43) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 2 | 34 MB | 0.51× (0.50–0.51) | 0.38× (0.35–0.42) | 1.26× (1.24–1.28) | 0.70× (0.69–0.72) | 3.65× (3.61–3.69) | 0.24× (0.24–0.25) | 0.25× (0.25–0.25) |
| `text` (1.46 MB → 12.2 MB, 8.3×) | 4 | 53 MB | 0.53× (0.51–0.55) | 0.45× (0.42–0.49) | 1.01× (0.99–1.03) | 0.72× (0.70–0.74) | 2.42× (2.38–2.46) | 0.15× (0.15–0.16) | 0.16× (0.15–0.16) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 272 | 3.68 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 483 | 3.76 | 1.02× (1.01–1.03) | 88% | 0.95× (0.94–0.96) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 564 | 5.95 | 1.61× (1.58–1.63) | 52% | 1.13× (1.11–1.15) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (564 MB/s); lbzip2 at 4 threads (211 MB/s); pbz2 at 4 threads (317 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.53, mibz2 4t 0.84, lbzip2 1t 0.10, lbzip2 2t 0.18, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.11, pbz2 4t 0.24. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 7%, lbzip2 2t 5%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 1.04 | 0.02 | 0.08 | 0.372 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3242 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.22 | 10.6% |
| MTF + RLE2 | 2.24 | 19.4% |
| T-vector build | 2.25 | 19.5% |
| walk (the one each build uses) | 5.31 | 46.0% |
| of which: segment assembly | 0.18 | 1.5% |
| RLE1 | 0.28 | 2.4% |
| CRC | 0.26 | 2.2% |
| **total** | 11.56 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h53m ± 14m13s |
| 4 threads | 54m19s ± 7m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 38m03s ± 5m27s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 38m03s ± 5m27s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 4.99 ns a byte of output at 4 threads (CPU, summed over them), 3.22 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,464,596 | 12,225,569 | \`fa458b0423a2b53f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 3.0.0 (bench/report.py), 10 rounds, threads 1, 2, 4
