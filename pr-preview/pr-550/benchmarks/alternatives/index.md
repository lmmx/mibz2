# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-550/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`4a2b9b1`](<https://github.com/lmmx/mibz2/commit/4a2b9b1948562838893d7f9ec6adc4bed8d7eab2>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44120630737>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.541 s | 387 | 8.2 | 2.97× (2.91–3.04) | 6.90× (6.72–7.08) | 7.38× (7.15–7.62) | 7.68× (7.49–7.88) | 7.23× (6.84–7.64) | 7.59× (7.28–7.91) | 7.00× (6.78–7.23) | 2.18× (2.13–2.22) | 2.47× (2.44–2.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.344 s | 610 | 9.5 | 2.96× (2.79–3.15) | 6.52× (6.24–6.80) | 7.30× (6.97–7.65) | 6.89× (6.57–7.23) | 6.40× (6.16–6.64) | 12.23× (11.47–13.04) | 11.29× (10.64–11.98) | 3.51× (3.32–3.70) | 3.98× (3.76–4.21) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.283 s | 740 | 15.9 | 3.11× (2.93–3.31) | 5.07× (4.76–5.39) | 6.37× (5.90–6.89) | 5.29× (5.06–5.54) | 5.71× (5.53–5.90) | 14.78× (13.87–15.76) | 13.65× (12.91–14.42) | 4.24× (4.06–4.43) | 4.81× (4.58–5.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 907 | 3.9 | 0.58× (0.56–0.60) | 3.55× (3.33–3.78) | 4.40× (4.21–4.60) | 3.36× (3.23–3.50) | 6.02× (5.76–6.29) | 2.78× (2.68–2.88) | 2.47× (2.38–2.57) | 1.12× (1.02–1.23) | 1.35× (1.28–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.024 s | 842 | 4.2 | 0.56× (0.54–0.58) | 2.61× (2.40–2.83) | 5.75× (5.48–6.04) | 3.27× (3.10–3.46) | 5.72× (5.53–5.92) | 2.67× (2.52–2.83) | 2.38× (2.24–2.52) | 1.08× (1.01–1.16) | 1.30× (1.20–1.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 864 | 4.0 | 0.55× (0.51–0.59) | 2.63× (2.43–2.84) | 5.71× (5.42–6.00) | 3.23× (3.05–3.42) | 5.83× (5.55–6.12) | 2.64× (2.49–2.81) | 2.35× (2.22–2.49) | 1.07× (0.97–1.17) | 1.29× (1.19–1.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 528 | 6.4 | 2.74× (2.53–2.96) | 2.74× (2.54–2.96) | 3.64× (3.41–3.88) | 3.49× (3.19–3.80) | 4.00× (3.76–4.26) | 2.31× (2.12–2.53) | 1.95× (1.81–2.09) | 2.01× (1.88–2.15) | 2.73× (2.53–2.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.027 s | 718 | 8.7 | 1.99× (1.70–2.34) | 2.28× (1.95–2.67) | 3.56× (3.15–4.01) | 2.72× (2.33–3.18) | 4.13× (3.53–4.82) | 3.05× (2.62–3.56) | 2.57× (2.18–3.03) | 2.65× (2.27–3.10) | 3.60× (3.06–4.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.032 s | 622 | 17.6 | 1.65× (1.52–1.79) | 1.61× (1.39–1.86) | 2.66× (2.35–3.01) | 1.87× (1.63–2.15) | 3.35× (2.98–3.76) | 2.72× (2.36–3.12) | 2.29× (2.00–2.62) | 2.36× (2.10–2.65) | 3.20× (2.80–3.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.146 s | 55 | 58.9 | 1.30× (1.23–1.37) | 2.71× (2.53–2.89) | 2.51× (2.16–2.91) | 2.45× (2.22–2.70) | 2.76× (2.65–2.88) | 3.02× (2.89–3.16) | 3.08× (2.95–3.22) | 1.24× (1.21–1.27) | 1.39× (1.33–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.089 s | 90 | 65.0 | 1.24× (1.16–1.32) | 2.48× (2.35–2.62) | 2.36× (2.25–2.49) | 3.04× (2.82–3.28) | 2.87× (2.79–2.96) | 4.86× (4.63–5.09) | 4.96× (4.72–5.20) | 1.99× (1.93–2.05) | 2.24× (2.15–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 88.1 | 1.24× (1.18–1.30) | 2.30× (2.11–2.51) | 2.28× (2.08–2.50) | 2.78× (2.58–3.00) | 3.01× (2.77–3.26) | 5.81× (5.60–6.02) | 5.93× (5.69–6.18) | 2.39× (2.32–2.46) | 2.68× (2.57–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.151 s | 64 | 50.7 | 1.45× (1.37–1.54) | 3.02× (2.96–3.08) | 2.59× (2.54–2.65) | 2.69× (2.51–2.87) | 3.05× (2.86–3.26) | 3.56× (3.32–3.82) | 3.92× (3.66–4.21) | 1.42× (1.37–1.48) | 1.73× (1.65–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.099 s | 97 | 62.1 | 1.22× (1.14–1.30) | 2.61× (2.47–2.75) | 2.32× (2.22–2.41) | 3.08× (2.88–3.30) | 2.88× (2.75–3.02) | 5.47× (5.13–5.82) | 6.02× (5.59–6.49) | 2.18× (2.06–2.31) | 2.65× (2.51–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.085 s | 113 | 86.2 | 1.34× (1.28–1.39) | 2.28× (2.09–2.49) | 2.36× (2.11–2.65) | 3.03× (2.71–3.39) | 2.96× (2.65–3.32) | 6.38× (6.01–6.77) | 7.03× (6.55–7.54) | 2.55× (2.42–2.68) | 3.10× (2.94–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 667 | 5.5 | 0.64× (0.60–0.68) | 3.13× (2.92–3.36) | 4.04× (3.77–4.32) | 2.92× (2.77–3.09) | 6.65× (6.50–6.80) | 2.39× (2.26–2.53) | 2.22× (2.10–2.36) | 0.99× (0.93–1.05) | 1.36× (1.27–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 643 | 5.8 | 0.58× (0.54–0.62) | 2.55× (2.38–2.73) | 4.97× (4.71–5.24) | 2.81× (2.67–2.94) | 6.24× (5.99–6.51) | 2.27× (2.17–2.36) | 2.11× (2.02–2.20) | 0.94× (0.89–0.99) | 1.29× (1.20–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.017 s | 575 | 6.3 | 0.53× (0.50–0.55) | 2.44× (2.28–2.61) | 4.79× (4.58–5.01) | 2.65× (2.53–2.78) | 5.98× (5.72–6.24) | 2.15× (2.04–2.27) | 2.00× (1.90–2.11) | 0.89× (0.85–0.93) | 1.23× (1.14–1.32) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 0.077 s | 167 | 19.6 | 1.91× (1.86–1.97) | 4.09× (3.88–4.31) | 4.59× (4.21–5.02) | 4.39× (4.18–4.61) | 4.77× (4.52–5.03) | 5.09× (4.88–5.32) | 4.93× (4.70–5.17) | 1.49× (1.42–1.58) | 1.81× (1.77–1.85) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 0.054 s | 237 | 24.7 | 1.58× (1.51–1.65) | 3.47× (3.14–3.85) | 4.13× (3.67–4.65) | 4.20× (3.96–4.46) | 4.28× (4.06–4.51) | 7.16× (6.79–7.56) | 6.93× (6.61–7.27) | 2.10× (1.98–2.23) | 2.55× (2.44–2.65) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 0.048 s | 269 | 37.2 | 1.69× (1.43–2.00) | 3.10× (2.52–3.81) | 3.72× (3.29–4.21) | 3.69× (3.32–4.10) | 4.42× (3.77–5.18) | 8.01× (7.48–8.59) | 7.76× (7.33–8.20) | 2.35× (2.16–2.56) | 2.85× (2.67–3.04) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.96× (2.90–3.03) | 6.86× (6.68–7.04) | 7.35× (7.11–7.59) | 7.65× (7.46–7.84) | 7.20× (6.82–7.61) | 7.55× (7.25–7.87) | 6.97× (6.75–7.20) | 2.17× (2.12–2.22) | 2.45× (2.42–2.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.67–3.02) | 6.47× (6.28–6.67) | 6.92× (6.66–7.19) | 7.21× (6.91–7.52) | 6.36× (6.12–6.61) | 6.46× (6.03–6.91) | 5.96× (5.59–6.36) | 1.85× (1.75–1.96) | 2.09× (1.98–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (2.87–3.26) | 5.38× (5.07–5.72) | 6.01× (5.54–6.52) | 5.65× (5.40–5.92) | 5.46× (5.27–5.65) | 4.07× (3.81–4.35) | 3.76× (3.55–3.98) | 1.17× (1.11–1.23) | 1.32× (1.25–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.64) | 3.11× (2.99–3.24) | 3.88× (3.78–3.99) | 3.15× (3.03–3.27) | 5.56× (5.33–5.80) | 2.62× (2.53–2.71) | 2.34× (2.26–2.42) | 1.11× (1.02–1.22) | 0.99× (0.96–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.63) | 2.82× (2.67–2.98) | 5.11× (4.98–5.24) | 3.04× (2.89–3.20) | 5.22× (5.05–5.39) | 2.48× (2.35–2.62) | 2.22× (2.10–2.34) | 1.06× (0.99–1.13) | 0.94× (0.89–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.55–0.63) | 3.18× (3.02–3.34) | 5.12× (4.91–5.34) | 3.07× (2.91–3.24) | 5.39× (5.14–5.67) | 2.50× (2.36–2.65) | 2.24× (2.12–2.36) | 1.06× (0.98–1.16) | 0.95× (0.88–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.44–2.85) | 2.55× (2.36–2.74) | 3.34× (3.15–3.54) | 3.35× (3.07–3.65) | 3.83× (3.60–4.08) | 2.24× (2.06–2.44) | 1.89× (1.76–2.03) | 1.96× (1.83–2.09) | 2.51× (2.33–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.16× (2.03–2.30) | 2.20× (2.06–2.35) | 2.84× (2.72–2.96) | 2.70× (2.49–2.93) | 3.09× (2.96–3.24) | 1.70× (1.58–1.83) | 1.44× (1.33–1.55) | 1.48× (1.41–1.57) | 1.90× (1.78–2.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.77× (1.64–1.91) | 1.38× (1.19–1.59) | 1.83× (1.61–2.09) | 1.63× (1.40–1.90) | 1.75× (1.53–2.00) | 0.80× (0.69–0.93) | 0.68× (0.58–0.78) | 0.70× (0.61–0.80) | 0.90× (0.78–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.29× (1.23–1.36) | 2.64× (2.46–2.82) | 2.45× (2.11–2.84) | 2.42× (2.20–2.67) | 2.73× (2.63–2.85) | 2.99× (2.86–3.12) | 3.05× (2.92–3.19) | 1.24× (1.20–1.27) | 1.36× (1.29–1.42) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.15–1.30) | 2.47× (2.35–2.60) | 2.17× (2.06–2.29) | 2.84× (2.72–2.97) | 2.51× (2.43–2.60) | 2.63× (2.50–2.76) | 2.68× (2.55–2.82) | 1.09× (1.05–1.12) | 1.19× (1.14–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.32–1.50) | 2.53× (2.28–2.80) | 2.10× (1.87–2.35) | 2.81× (2.58–3.05) | 2.77× (2.44–3.15) | 1.97× (1.89–2.06) | 2.01× (1.91–2.12) | 0.82× (0.79–0.84) | 0.90× (0.85–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.36–1.53) | 2.97× (2.90–3.03) | 2.55× (2.50–2.60) | 2.66× (2.49–2.85) | 3.02× (2.83–3.23) | 3.52× (3.29–3.77) | 3.88× (3.62–4.16) | 1.41× (1.36–1.47) | 1.69× (1.61–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.24× (1.16–1.32) | 2.64× (2.49–2.80) | 2.14× (2.06–2.24) | 2.92× (2.72–3.12) | 2.57× (2.45–2.70) | 2.90× (2.72–3.09) | 3.19× (2.96–3.44) | 1.16× (1.11–1.22) | 1.39× (1.32–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.51× (1.44–1.58) | 2.58× (2.37–2.81) | 2.16× (1.90–2.46) | 2.94× (2.61–3.32) | 2.73× (2.38–3.13) | 2.07× (1.94–2.21) | 2.28× (2.11–2.46) | 0.83× (0.78–0.89) | 0.99× (0.93–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.65–0.73) | 2.55× (2.41–2.71) | 3.36× (3.19–3.54) | 2.67× (2.54–2.81) | 5.90× (5.78–6.02) | 2.20× (2.09–2.32) | 2.07× (1.95–2.18) | 0.99× (0.94–1.05) | 0.91× (0.86–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.65× (0.61–0.69) | 2.42× (2.32–2.52) | 4.26× (4.16–4.37) | 2.60× (2.49–2.72) | 5.50× (5.28–5.73) | 2.06× (1.98–2.14) | 1.94× (1.87–2.01) | 0.93× (0.89–0.97) | 0.85× (0.81–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.55–0.60) | 2.59× (2.45–2.73) | 4.08× (3.93–4.23) | 2.54× (2.43–2.66) | 5.41× (5.19–5.64) | 2.01× (1.92–2.12) | 1.89× (1.80–1.98) | 0.91× (0.87–0.95) | 0.83× (0.79–0.88) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 1.88× (1.83–1.94) | 3.92× (3.73–4.13) | 4.43× (4.07–4.83) | 4.29× (4.09–4.49) | 4.65× (4.42–4.90) | 4.97× (4.76–5.19) | 4.81× (4.59–5.04) | 1.48× (1.40–1.56) | 1.73× (1.67–1.78) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 1.64× (1.58–1.70) | 3.50× (3.15–3.88) | 3.83× (3.49–4.20) | 4.11× (3.85–4.38) | 3.83× (3.61–4.07) | 3.90× (3.72–4.10) | 3.78× (3.62–3.94) | 1.16× (1.09–1.23) | 1.36× (1.30–1.41) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 1.97× (1.65–2.36) | 3.47× (2.81–4.29) | 3.55× (3.16–3.98) | 3.54× (3.23–3.87) | 3.82× (3.17–4.60) | 2.53× (2.37–2.71) | 2.45× (2.32–2.59) | 0.75× (0.70–0.82) | 0.88× (0.82–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.25–0.28) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.97–11.04) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.53× (0.46–0.62) | 0.69× (0.67–0.71) | 0.57× (0.56–0.59) | 5.45× (5.33–5.57) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.34) | 0.55× (0.47–0.63) | 0.60× (0.58–0.62) | 0.52× (0.50–0.53) | 3.42× (3.32–3.53) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.17) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.70) | 0.66× (0.65–0.66) | 8.00× (7.99–8.01) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.70) | 0.77× (0.72–0.82) | 1.65× (1.60–1.69) | 0.83× (0.81–0.86) | 5.96× (5.80–6.11) | 0.48× (0.47–0.50) | 0.51× (0.49–0.52) | 0.55× (0.54–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.62–0.67) | 0.74× (0.70–0.78) | 1.18× (1.14–1.23) | 0.72× (0.69–0.75) | 3.43× (3.31–3.55) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.18× (5.16–5.20) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.38× (0.35–0.41) | 0.89× (0.87–0.92) | 0.70× (0.67–0.73) | 3.08× (3.00–3.16) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.65) | 0.45× (0.44–0.46) | 0.85× (0.84–0.86) | 0.80× (0.78–0.82) | 2.48× (2.45–2.52) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.17–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.35× (0.32–0.38) | 0.83× (0.82–0.85) | 0.67× (0.65–0.70) | 2.86× (2.80–2.92) | 0.18× (0.18–0.18) | 0.19× (0.19–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.47× (0.45–0.50) | 0.81× (0.80–0.82) | 0.82× (0.80–0.85) | 2.32× (2.29–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.36× (5.36–5.37) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.31× (5.30–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.66) | 6.38× (6.37–6.39) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 34 MB | 0.48× (0.46–0.49) | 0.43× (0.39–0.48) | 1.14× (1.10–1.18) | 0.73× (0.67–0.78) | 3.52× (3.40–3.65) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.35× (0.33–0.36) | 0.17× (0.17–0.18) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 56 MB | 0.49× (0.47–0.50) | 0.48× (0.44–0.52) | 0.90× (0.88–0.92) | 0.70× (0.68–0.72) | 2.31× (2.26–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 387 | 2.59 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 610 | 2.99 | 1.17× (1.12–1.23) | 81% | 1.10× (1.06–1.15) | 1.12× (1.10–1.15) |
| 4: all 4 CPUs, sharing SMT siblings | 740 | 5.01 | 1.86× (1.76–1.96) | 49% | 1.46× (1.33–1.59) | 1.91× (1.74–2.10) |

**Best:** mibz2 at 4 threads (740 MB/s); lbzip2 at 4 threads (149 MB/s); pbz2 at 4 threads (248 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.12, lbzip2 1t 0.09, lbzip2 2t 0.40, lbzip2 4t 0.51, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.81 | 0.03 | 0.09 | 0.283 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3176 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m12s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.18 | 12.4% |
| MTF + RLE2 | 1.60 | 16.8% |
| T-vector build | 1.60 | 16.8% |
| walk (the one each build uses) | 4.58 | 48.1% |
| of which: segment assembly | 0.19 | 2.0% |
| RLE1 | 0.30 | 3.2% |
| CRC | 0.26 | 2.8% |
| **total** | 9.52 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.509 | 1.343 | (in its total) | 2.64× (2.62–2.65) |  |
| walk | 1.319 | 2.687 | (in its total) | 2.02× (1.98–2.05) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.053 | 0.071 | (in its total) | 1.29× (1.21–1.37) |  |
| inverse BWT, total | 1.882 | 4.106 | 3.556 | 2.16× (2.13–2.20) | 1.88× (1.85–1.91) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h20m ± 9m33s |
| 4 threads | 41m13s ± 6m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m37s ± 4m43s |
| the scanner alone (its serial part) | 14m24s ± 54.7s |
| the workers alone, over 4 | 29m36s ± 4m43s |

At 4 threads the workers bound it. Per byte: the scanner 8.4 ns a byte of bz2, the workers 3.88 ns a byte of output at 4 threads (CPU, summed over them), 2.15 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,519,058 | 12,810,756 | \`6ae766e8f83bdb21…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3176.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
