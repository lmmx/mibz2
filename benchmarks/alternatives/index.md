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
| bz2ngrs | Softleif's safe-Rust port of bz2-ng, at a pinned commit: `DecompressReader` | single-threaded (one thread a stream) |
| bz2ng | bz2-ng's C, at a pinned commit: its `bzip2 -dc`, read through a pipe (the reference bz2ngrs ports) | single-threaded |

**Single-threaded decoders run at 1 thread only.** libbzip2, libbz2-rs, bz2ngrs and bz2ng have no thread count, so they never decode on 4 threads. At 4 threads, their figures are their 1-thread runs against mibz2's at 4 threads: in the chart, hollow points with dashed whiskers, under "single-threaded (1 thread)"; in the tables, marked ¹.

**mibz2's inverse BWT against bz2ngrs's and libsais's.** Below the decoders' tables, the kernels run on the same Wikidata blocks, with the build, the walk and the output's assembly timed apart where a kernel allows it. libsais 2.12.1 (Ilya Grebnov) is the fastest inverse BWT measured here. It is a library of BWT algorithms, not a bzip2 decoder, so it appears in that table only, never in the chart. Its build and walk happen inside one call, so it shows a total. bz2ngrs and mibz2 store the same successor link in each 32-bit entry, but a different byte beside it: bz2ngrs the row's first-column byte, mibz2 its last-column byte (lbzip2's packing). The table says which phase a whole-decoder difference comes from ([the study](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-bz2ngrs.md>)).

Not included: `bzip2 -dc` and Python's `bz2` module, which are libbzip2 itself (its arm stands for them); and bz2k, mibz2's own prototype, which it replaced.

The latest record: [`15c289b`](<https://github.com/lmmx/mibz2/commit/15c289b576168241c4c0f47ce1414f2743f0ffb4>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46095511583>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.485 s | 432 | 7.5 | 3.18× (3.14–3.23) | 7.28× (7.20–7.36) | 7.58× (7.48–7.68) | 8.18× (8.13–8.24) | 7.31× (7.23–7.39) | 7.94× (7.85–8.03) | 7.47× (7.40–7.53) | 2.35× (2.33–2.38) | 2.71× (2.69–2.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.275 s | 761 | 8.2 | 3.15× (2.85–3.48) | 6.73× (6.34–7.14) | 7.59× (7.04–8.19) | 7.65× (7.12–8.21) | 6.78× (6.33–7.27) | 13.62× (12.78–14.51) | 12.80× (12.00–13.65) | 4.03× (3.80–4.28) | 4.64× (4.36–4.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 937 | 12.8 | 3.26× (3.05–3.47) | 5.00× (4.56–5.48) | 6.10× (5.79–6.42) | 5.53× (5.25–5.82) | 5.66× (5.35–5.98) | 16.80× (15.79–17.87) | 15.79× (14.95–16.68) | 4.97× (4.69–5.28) | 5.73× (5.39–6.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 948 | 3.8 | 0.60× (0.58–0.61) | 3.51× (3.38–3.65) | 4.60× (4.45–4.75) | 3.51× (3.46–3.57) | 6.09× (6.00–6.18) | 2.91× (2.87–2.96) | 2.61× (2.56–2.65) | 1.00× (0.98–1.03) | 1.47× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 924 | 4.0 | 0.55× (0.53–0.56) | 2.88× (2.77–3.00) | 6.05× (5.90–6.20) | 3.41× (3.33–3.50) | 5.85× (5.72–5.98) | 2.81× (2.74–2.88) | 2.51× (2.45–2.57) | 0.97× (0.94–1.00) | 1.42× (1.38–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 873 | 4.2 | 0.53× (0.51–0.54) | 2.73× (2.67–2.78) | 5.75× (5.59–5.91) | 3.25× (3.18–3.32) | 5.66× (5.53–5.79) | 2.69× (2.63–2.74) | 2.40× (2.35–2.45) | 0.92× (0.89–0.96) | 1.36× (1.33–1.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 586 | 5.9 | 2.85× (2.73–2.97) | 3.03× (2.99–3.07) | 3.88× (3.83–3.93) | 3.94× (3.89–4.00) | 4.23× (4.17–4.29) | 2.56× (2.53–2.59) | 2.11× (2.09–2.14) | 2.05× (2.04–2.06) | 3.02× (2.98–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 907 | 7.0 | 2.20× (2.05–2.37) | 2.73× (2.47–3.01) | 4.06× (3.76–4.38) | 3.43× (3.14–3.75) | 4.76× (4.34–5.21) | 3.76× (3.46–4.08) | 3.10× (2.85–3.37) | 3.00× (2.77–3.26) | 4.42× (4.07–4.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 812 | 14.3 | 2.01× (1.96–2.06) | 2.11× (1.97–2.26) | 3.28× (3.12–3.44) | 2.41× (2.27–2.55) | 4.21× (4.11–4.31) | 3.56× (3.46–3.67) | 2.94× (2.85–3.03) | 2.85× (2.74–2.96) | 4.19× (4.06–4.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 51.0 | 1.48× (1.47–1.49) | 2.92× (2.89–2.95) | 2.66× (2.62–2.69) | 2.77× (2.75–2.79) | 3.08× (3.04–3.12) | 3.39× (3.36–3.43) | 3.43× (3.40–3.46) | 1.35× (1.33–1.37) | 1.54× (1.51–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 108 | 56.4 | 1.42× (1.39–1.45) | 2.83× (2.74–2.92) | 2.64× (2.58–2.69) | 3.48× (3.41–3.56) | 3.27× (3.18–3.36) | 5.61× (5.50–5.73) | 5.67× (5.55–5.79) | 2.24× (2.19–2.28) | 2.54× (2.48–2.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.061 s | 131 | 76.4 | 1.47× (1.41–1.55) | 2.46× (2.33–2.60) | 2.33× (2.21–2.46) | 2.95× (2.83–3.07) | 3.23× (3.10–3.37) | 6.79× (6.57–7.01) | 6.85× (6.63–7.08) | 2.70× (2.60–2.81) | 3.07× (2.95–3.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.137 s | 70 | 47.2 | 1.58× (1.57–1.60) | 3.24× (3.21–3.28) | 2.76× (2.72–2.80) | 2.95× (2.92–2.98) | 3.29× (3.24–3.35) | 3.90× (3.86–3.94) | 4.27× (4.23–4.32) | 1.56× (1.53–1.58) | 1.86× (1.84–1.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 118 | 52.9 | 1.45× (1.39–1.51) | 2.98× (2.87–3.08) | 2.69× (2.60–2.78) | 3.81× (3.67–3.95) | 3.29× (3.19–3.39) | 6.42× (6.24–6.60) | 7.03× (6.84–7.23) | 2.56× (2.47–2.66) | 3.06× (2.98–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 138 | 74.7 | 1.55× (1.52–1.58) | 2.59× (2.51–2.68) | 2.47× (2.40–2.55) | 3.33× (3.24–3.43) | 3.12× (3.07–3.17) | 7.59× (7.47–7.72) | 8.32× (8.19–8.45) | 3.03× (2.95–3.11) | 3.62× (3.55–3.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 5.2 | 0.66× (0.65–0.67) | 3.03× (2.89–3.18) | 4.33× (4.28–4.37) | 3.08× (3.06–3.11) | 6.78× (6.73–6.83) | 2.50× (2.47–2.52) | 2.30× (2.28–2.32) | 0.98× (0.96–0.99) | 1.50× (1.48–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 649 | 6.0 | 0.52× (0.51–0.54) | 2.67× (2.62–2.72) | 4.82× (4.61–5.04) | 2.81× (2.76–2.86) | 6.12× (6.00–6.24) | 2.24× (2.20–2.28) | 2.07× (2.03–2.10) | 0.88× (0.86–0.89) | 1.34× (1.32–1.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 652 | 5.8 | 0.53× (0.52–0.54) | 2.69× (2.66–2.73) | 4.73× (4.54–4.93) | 2.82× (2.79–2.86) | 6.09× (6.01–6.17) | 2.25× (2.22–2.28) | 2.07× (2.05–2.10) | 0.88× (0.86–0.90) | 1.35× (1.33–1.37) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 0.066 s | 228 | 14.7 | 2.20× (2.14–2.26) | 4.26× (4.10–4.42) | 4.67× (4.53–4.80) | 4.71× (4.57–4.85) | 4.89× (4.75–5.04) | 5.27× (5.12–5.42) | 4.95× (4.81–5.09) | 1.65× (1.61–1.69) | 2.09× (2.00–2.18) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 0.046 s | 323 | 19.2 | 1.73× (1.64–1.83) | 3.42× (3.27–3.57) | 4.06× (3.95–4.16) | 4.49× (4.27–4.73) | 4.22× (4.11–4.33) | 7.41× (7.15–7.68) | 6.96× (6.75–7.17) | 2.32× (2.24–2.40) | 2.94× (2.80–3.08) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 0.040 s | 370 | 29.8 | 1.65× (1.58–1.72) | 2.66× (2.52–2.81) | 3.63× (3.43–3.85) | 3.64× (3.48–3.80) | 4.02× (3.93–4.11) | 8.47× (8.17–8.79) | 7.96× (7.70–8.22) | 2.65× (2.54–2.77) | 3.36× (3.21–3.52) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.17× (3.13–3.22) | 7.24× (7.17–7.32) | 7.54× (7.45–7.64) | 8.15× (8.10–8.21) | 7.28× (7.20–7.36) | 7.91× (7.82–8.00) | 7.44× (7.38–7.50) | 2.35× (2.32–2.37) | 2.69× (2.67–2.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.14× (2.97–3.31) | 6.91× (6.78–7.05) | 7.24× (7.13–7.35) | 8.00× (7.91–8.09) | 6.84× (6.75–6.93) | 7.28× (7.17–7.40) | 6.85× (6.75–6.95) | 2.16× (2.12–2.20) | 2.48× (2.45–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.14–3.25) | 5.34× (5.21–5.49) | 5.79× (5.66–5.92) | 6.00× (5.89–6.12) | 5.43× (5.32–5.54) | 4.70× (4.60–4.79) | 4.41× (4.34–4.49) | 1.39× (1.37–1.41) | 1.60× (1.57–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.19× (3.15–3.24) | 4.04× (3.97–4.10) | 3.29× (3.25–3.33) | 5.64× (5.56–5.71) | 2.74× (2.70–2.78) | 2.46× (2.43–2.50) | 1.00× (0.98–1.03) | 1.03× (1.00–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.60) | 2.91× (2.79–3.03) | 5.14× (4.99–5.29) | 3.11× (3.05–3.18) | 5.24× (5.14–5.34) | 2.55× (2.50–2.60) | 2.29× (2.24–2.34) | 0.93× (0.91–0.96) | 0.96× (0.93–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.57) | 3.06× (2.99–3.13) | 4.99× (4.85–5.12) | 3.02× (2.96–3.08) | 5.15× (5.03–5.26) | 2.49× (2.44–2.54) | 2.23× (2.19–2.28) | 0.91× (0.88–0.94) | 0.93× (0.91–0.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.74× (2.63–2.86) | 2.80× (2.77–2.83) | 3.57× (3.53–3.60) | 3.78× (3.73–3.83) | 4.05× (3.99–4.10) | 2.47× (2.45–2.50) | 2.05× (2.03–2.07) | 1.99× (1.98–2.01) | 2.74× (2.70–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.21–2.41) | 2.53× (2.39–2.68) | 3.16× (3.00–3.33) | 3.29× (3.12–3.47) | 3.46× (3.28–3.65) | 2.03× (1.93–2.14) | 1.68× (1.60–1.77) | 1.63× (1.56–1.71) | 2.24× (2.13–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.08× (2.01–2.15) | 1.79× (1.73–1.85) | 2.25× (2.18–2.32) | 2.06× (2.00–2.12) | 2.13× (2.06–2.19) | 1.03× (1.01–1.06) | 0.86× (0.83–0.88) | 0.83× (0.81–0.85) | 1.14× (1.11–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.47× (1.46–1.48) | 2.85× (2.82–2.87) | 2.59× (2.55–2.63) | 2.74× (2.72–2.77) | 3.04× (3.01–3.08) | 3.35× (3.32–3.39) | 3.38× (3.35–3.42) | 1.34× (1.32–1.37) | 1.49× (1.47–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.40× (1.38–1.43) | 2.69× (2.65–2.73) | 2.40× (2.36–2.43) | 3.27× (3.23–3.31) | 2.83× (2.77–2.89) | 3.01× (2.96–3.05) | 3.04× (2.99–3.09) | 1.21× (1.18–1.23) | 1.33× (1.31–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.52–1.62) | 2.60× (2.51–2.70) | 2.09× (2.04–2.14) | 2.84× (2.78–2.89) | 2.71× (2.63–2.78) | 2.22× (2.19–2.25) | 2.24× (2.21–2.28) | 0.89× (0.88–0.91) | 0.99× (0.97–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.57× (1.56–1.59) | 3.17× (3.14–3.21) | 2.70× (2.66–2.75) | 2.93× (2.90–2.96) | 3.26× (3.21–3.31) | 3.86× (3.81–3.90) | 4.22× (4.18–4.27) | 1.55× (1.52–1.58) | 1.81× (1.80–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.47× (1.44–1.50) | 2.96× (2.92–3.01) | 2.50× (2.46–2.54) | 3.57× (3.53–3.62) | 2.96× (2.92–3.00) | 3.43× (3.38–3.47) | 3.75× (3.70–3.80) | 1.38× (1.35–1.40) | 1.61× (1.58–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.71× (1.67–1.76) | 2.79× (2.75–2.83) | 2.13× (2.08–2.18) | 3.08× (3.00–3.16) | 2.79× (2.73–2.84) | 2.44× (2.40–2.49) | 2.68× (2.63–2.72) | 0.98× (0.96–1.00) | 1.15× (1.13–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.68× (2.66–2.71) | 3.54× (3.50–3.58) | 2.81× (2.79–2.83) | 6.04× (5.99–6.09) | 2.30× (2.28–2.32) | 2.13× (2.11–2.14) | 0.98× (0.96–1.00) | 0.95× (0.93–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.59) | 2.28× (2.24–2.32) | 3.98× (3.88–4.08) | 2.52× (2.48–2.56) | 5.25× (5.16–5.34) | 1.98× (1.95–2.02) | 1.83× (1.81–1.86) | 0.84× (0.83–0.86) | 0.82× (0.80–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.54× (2.50–2.58) | 3.98× (3.93–4.03) | 2.59× (2.55–2.62) | 5.32× (5.25–5.39) | 2.03× (2.00–2.06) | 1.88× (1.85–1.90) | 0.86× (0.84–0.88) | 0.84× (0.82–0.86) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 2.16× (2.10–2.22) | 4.11× (3.98–4.23) | 4.50× (4.39–4.60) | 4.60× (4.47–4.73) | 4.78× (4.64–4.92) | 5.14× (5.00–5.29) | 4.83× (4.70–4.96) | 1.63× (1.59–1.67) | 1.99× (1.92–2.06) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 1.81× (1.77–1.84) | 3.33× (3.25–3.40) | 3.70× (3.65–3.76) | 4.08× (4.01–4.16) | 3.76× (3.72–3.81) | 3.99× (3.91–4.06) | 3.75× (3.69–3.80) | 1.27× (1.24–1.29) | 1.54× (1.50–1.59) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 1.78× (1.75–1.81) | 2.76× (2.69–2.83) | 3.05× (2.95–3.16) | 3.16× (3.07–3.25) | 3.03× (2.97–3.09) | 2.56× (2.52–2.60) | 2.40× (2.37–2.43) | 0.81× (0.80–0.83) | 0.99× (0.96–1.02) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.29× (0.27–0.30) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.02× (10.98–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.50× (0.44–0.57) | 0.72× (0.70–0.73) | 0.59× (0.57–0.61) | 5.65× (5.48–5.82) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.35–0.39) | 0.72× (0.68–0.77) | 0.65× (0.62–0.69) | 0.56× (0.53–0.59) | 3.79× (3.62–3.97) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.67–1.70) | 0.66× (0.65–0.67) | 7.96× (7.90–8.03) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.72) | 0.73× (0.68–0.77) | 1.69× (1.66–1.73) | 0.85× (0.83–0.87) | 6.15× (6.01–6.29) | 0.50× (0.49–0.51) | 0.53× (0.51–0.54) | 0.57× (0.56–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.68× (0.67–0.70) | 0.77× (0.72–0.82) | 1.25× (1.22–1.28) | 0.75× (0.72–0.78) | 3.66× (3.57–3.75) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.74) | 5.15× (5.13–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.90× (0.87–0.93) | 0.70× (0.67–0.74) | 3.10× (3.01–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.47× (0.46–0.48) | 0.89× (0.87–0.90) | 0.83× (0.80–0.85) | 2.58× (2.54–2.62) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.84× (0.83–0.84) | 5.22× (5.21–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.37× (0.34–0.40) | 0.84× (0.81–0.87) | 0.71× (0.67–0.74) | 2.89× (2.78–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.50× (0.47–0.54) | 0.80× (0.79–0.82) | 0.80× (0.78–0.81) | 2.31× (2.27–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.30× (5.28–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.15–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.87) | 0.89× (0.89–0.90) | 6.17× (6.14–6.19) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.66× (0.66–0.66) | 6.35× (6.34–6.36) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.63) | 0.30× (0.30–0.30) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 38 MB | 0.45× (0.43–0.47) | 0.39× (0.36–0.42) | 1.03× (0.99–1.07) | 0.70× (0.66–0.75) | 3.40× (3.25–3.56) | 0.21× (0.20–0.22) | 0.22× (0.21–0.23) | 0.34× (0.32–0.35) | 0.16× (0.15–0.17) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.46× (0.42–0.50) | 0.90× (0.88–0.92) | 0.75× (0.73–0.76) | 2.32× (2.28–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 432 | 2.32 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 761 | 2.53 | 1.09× (1.07–1.10) | 86% | 1.04× (1.02–1.06) | 1.07× (1.02–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 937 | 3.92 | 1.69× (1.66–1.71) | 53% | 1.24× (1.21–1.28) | 1.69× (1.66–1.73) |

**Best:** mibz2 at 4 threads (937 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (280 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.02, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.66 | 0.02 | 0.08 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3251 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.3% |
| MTF + RLE2 | 1.54 | 17.6% |
| T-vector build | 1.52 | 17.5% |
| walk (the one each build uses) | 4.32 | 49.5% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.74 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.511 | 1.304 | (in its total) | 2.55× (2.54–2.57) |  |
| walk | 1.268 | 2.653 | (in its total) | 2.10× (2.08–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.059 | (in its total) | 1.25× (1.23–1.27) |  |
| inverse BWT, total | 1.826 | 4.016 | 3.314 | 2.20× (2.19–2.22) | 1.82× (1.81–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m36s |
| 4 threads | 33m34s ± 5m11s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m52s ± 3m24s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 23m52s ± 3m24s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 3.13 ns a byte of output at 4 threads (CPU, summed over them), 2.01 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,243,613 | 14,938,930 | \`64eba10b2c1f54ed…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3251.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
