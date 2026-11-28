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

The latest record: [`0ec1d59`](<https://github.com/lmmx/mibz2/commit/0ec1d590bd774526d824bdf33da380af650df323>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46103672541>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.481 s | 436 | 7.5 | 3.14× (3.11–3.16) | 7.17× (7.11–7.22) | 7.32× (7.26–7.38) | 8.08× (8.03–8.13) | 7.17× (7.13–7.22) | 7.82× (7.76–7.88) | 7.30× (7.23–7.38) | 2.31× (2.30–2.33) | 2.68× (2.68–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.273 s | 768 | 8.0 | 3.23× (3.05–3.41) | 6.75× (6.47–7.05) | 7.81× (7.48–8.16) | 7.84× (7.69–8.00) | 6.89× (6.74–7.04) | 13.87× (13.60–14.14) | 12.96× (12.62–13.30) | 4.10× (4.00–4.20) | 4.76× (4.66–4.86) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.219 s | 957 | 12.6 | 3.21× (3.15–3.28) | 4.90× (4.80–5.01) | 6.11× (6.04–6.18) | 5.57× (5.44–5.69) | 5.56× (5.46–5.67) | 17.11× (16.79–17.43) | 15.98× (15.69–16.28) | 5.06× (4.96–5.16) | 5.87× (5.77–5.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 966 | 3.7 | 0.60× (0.59–0.61) | 3.66× (3.48–3.86) | 4.44× (4.34–4.54) | 3.55× (3.51–3.59) | 6.15× (6.07–6.23) | 2.96× (2.92–3.01) | 2.65× (2.61–2.68) | 1.01× (0.99–1.03) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 915 | 4.0 | 0.55× (0.54–0.55) | 2.83× (2.78–2.88) | 5.66× (5.55–5.78) | 3.38× (3.33–3.43) | 5.81× (5.74–5.89) | 2.80× (2.76–2.84) | 2.50× (2.46–2.54) | 0.95× (0.94–0.97) | 1.42× (1.39–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 907 | 4.0 | 0.54× (0.53–0.55) | 2.80× (2.75–2.85) | 5.55× (5.41–5.69) | 3.33× (3.27–3.39) | 5.78× (5.68–5.89) | 2.76× (2.72–2.81) | 2.47× (2.42–2.51) | 0.94× (0.93–0.96) | 1.40× (1.37–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 602 | 5.7 | 3.02× (2.89–3.15) | 3.14× (3.12–3.15) | 4.05× (3.99–4.12) | 4.07× (4.05–4.09) | 4.33× (4.30–4.37) | 2.65× (2.63–2.67) | 2.19× (2.17–2.20) | 2.08× (2.07–2.09) | 3.12× (3.10–3.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 947 | 6.9 | 2.37× (2.23–2.51) | 3.02× (2.67–3.41) | 4.08× (3.83–4.36) | 3.60× (3.37–3.84) | 5.08× (4.78–5.40) | 4.05× (3.80–4.31) | 3.34× (3.14–3.55) | 3.17× (2.98–3.38) | 4.76× (4.48–5.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 832 | 13.1 | 1.99× (1.86–2.13) | 2.10× (1.88–2.34) | 3.06× (2.88–3.26) | 2.29× (2.15–2.44) | 4.14× (3.88–4.43) | 3.63× (3.43–3.85) | 3.00× (2.82–3.18) | 2.85× (2.69–3.03) | 4.28× (4.03–4.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.119 s | 67 | 48.9 | 1.48× (1.47–1.49) | 2.96× (2.93–3.00) | 2.65× (2.60–2.70) | 2.71× (2.69–2.73) | 3.13× (3.10–3.16) | 3.44× (3.41–3.47) | 3.49× (3.44–3.54) | 1.34× (1.33–1.35) | 1.53× (1.52–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 112 | 54.8 | 1.45× (1.41–1.49) | 2.85× (2.80–2.89) | 2.70× (2.67–2.73) | 3.47× (3.43–3.50) | 3.31× (3.28–3.35) | 5.74× (5.69–5.80) | 5.83× (5.74–5.92) | 2.23× (2.21–2.26) | 2.55× (2.52–2.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 s | 135 | 75.1 | 1.48× (1.45–1.50) | 2.42× (2.37–2.47) | 2.33× (2.26–2.40) | 2.89× (2.84–2.94) | 3.24× (3.19–3.28) | 6.85× (6.75–6.96) | 6.95× (6.82–7.09) | 2.66× (2.63–2.70) | 3.04× (3.00–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.134 s | 72 | 45.9 | 1.57× (1.55–1.59) | 3.27× (3.21–3.34) | 2.73× (2.68–2.78) | 2.86× (2.82–2.90) | 3.31× (3.25–3.36) | 3.89× (3.85–3.93) | 4.22× (4.17–4.27) | 1.51× (1.49–1.52) | 1.82× (1.80–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 115 | 51.8 | 1.50× (1.39–1.61) | 3.01× (2.83–3.19) | 2.62× (2.53–2.72) | 3.83× (3.69–3.98) | 3.24× (3.12–3.36) | 6.31× (6.09–6.53) | 6.84× (6.60–7.10) | 2.44× (2.35–2.54) | 2.96× (2.86–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 75.0 | 1.57× (1.54–1.61) | 2.55× (2.50–2.61) | 2.49× (2.42–2.55) | 3.36× (3.28–3.43) | 3.13× (3.08–3.18) | 7.65× (7.53–7.77) | 8.30× (8.16–8.44) | 2.96× (2.92–3.01) | 3.58× (3.52–3.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.0 | 0.67× (0.66–0.68) | 3.17× (2.98–3.38) | 4.44× (4.41–4.47) | 3.16× (3.10–3.22) | 6.88× (6.83–6.93) | 2.55× (2.53–2.57) | 2.35× (2.33–2.36) | 0.99× (0.98–1.00) | 1.53× (1.52–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.8 | 0.55× (0.54–0.55) | 2.77× (2.75–2.79) | 4.76× (4.72–4.79) | 2.89× (2.86–2.93) | 6.25× (6.19–6.31) | 2.32× (2.30–2.34) | 2.14× (2.12–2.15) | 0.90× (0.89–0.91) | 1.39× (1.38–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 5.8 | 0.53× (0.51–0.55) | 2.69× (2.60–2.79) | 4.67× (4.63–4.71) | 2.79× (2.70–2.89) | 6.05× (5.87–6.24) | 2.25× (2.17–2.33) | 2.07× (2.00–2.14) | 0.87× (0.84–0.91) | 1.35× (1.30–1.40) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.064 s | 235 | 14.2 | 2.21× (2.18–2.24) | 4.30× (4.23–4.37) | 4.71× (4.63–4.79) | 4.78× (4.70–4.86) | 4.97× (4.89–5.06) | 5.35× (5.27–5.43) | 5.06× (4.98–5.13) | 1.67× (1.64–1.69) | 2.07× (2.04–2.10) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.046 s | 330 | 18.4 | 1.64× (1.50–1.80) | 3.35× (3.07–3.66) | 3.95× (3.64–4.29) | 4.31× (3.94–4.72) | 4.24× (3.83–4.69) | 7.19× (6.61–7.83) | 6.80× (6.26–7.39) | 2.24× (2.07–2.43) | 2.78× (2.55–3.03) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.041 s | 370 | 29.8 | 1.68× (1.64–1.72) | 2.60× (2.56–2.63) | 3.56× (3.44–3.69) | 3.67× (3.56–3.77) | 3.91× (3.82–4.00) | 8.44× (8.26–8.62) | 7.98× (7.81–8.15) | 2.63× (2.57–2.69) | 3.26× (3.22–3.30) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.13× (3.10–3.16) | 7.13× (7.08–7.19) | 7.28× (7.22–7.35) | 8.05× (8.00–8.10) | 7.14× (7.10–7.19) | 7.79× (7.73–7.85) | 7.28× (7.20–7.35) | 2.31× (2.29–2.32) | 2.67× (2.66–2.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.11× (3.00–3.21) | 6.91× (6.85–6.96) | 7.32× (7.17–7.48) | 8.17× (8.03–8.31) | 6.83× (6.79–6.86) | 7.29× (7.23–7.35) | 6.81× (6.76–6.87) | 2.16× (2.15–2.17) | 2.50× (2.48–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.05× (3.02–3.09) | 5.16× (5.11–5.22) | 5.56× (5.50–5.63) | 5.87× (5.79–5.94) | 5.16× (5.07–5.25) | 4.64× (4.57–4.70) | 4.33× (4.27–4.40) | 1.37× (1.36–1.39) | 1.59× (1.57–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.25× (3.19–3.32) | 4.04× (3.99–4.09) | 3.33× (3.29–3.36) | 5.69× (5.64–5.75) | 2.79× (2.76–2.83) | 2.50× (2.47–2.53) | 1.01× (1.00–1.02) | 1.05× (1.03–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.58–0.59) | 2.87× (2.83–2.91) | 5.00× (4.93–5.07) | 3.09× (3.04–3.14) | 5.22× (5.17–5.28) | 2.56× (2.52–2.59) | 2.29× (2.25–2.33) | 0.92× (0.91–0.94) | 0.97× (0.94–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.56–0.57) | 3.17× (3.10–3.25) | 5.02× (4.92–5.12) | 3.09× (3.05–3.14) | 5.26× (5.18–5.35) | 2.57× (2.53–2.61) | 2.30× (2.26–2.34) | 0.93× (0.91–0.94) | 0.97× (0.94–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.90× (2.79–3.02) | 2.89× (2.87–2.91) | 3.63× (3.56–3.71) | 3.89× (3.87–3.91) | 4.14× (4.11–4.18) | 2.55× (2.54–2.57) | 2.12× (2.10–2.13) | 2.02× (2.01–2.04) | 2.79× (2.76–2.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.39× (2.35–2.42) | 2.65× (2.62–2.68) | 3.22× (3.18–3.27) | 3.42× (3.38–3.47) | 3.59× (3.55–3.63) | 2.12× (2.10–2.15) | 1.76× (1.74–1.78) | 1.68× (1.66–1.70) | 2.32× (2.28–2.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.25× (2.17–2.34) | 1.91× (1.81–2.01) | 2.41× (2.31–2.51) | 2.26× (2.16–2.36) | 2.29× (2.20–2.39) | 1.12× (1.08–1.17) | 0.93× (0.89–0.97) | 0.89× (0.85–0.93) | 1.23× (1.18–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.47× (1.46–1.48) | 2.88× (2.86–2.91) | 2.60× (2.56–2.64) | 2.68× (2.67–2.70) | 3.10× (3.07–3.12) | 3.40× (3.37–3.43) | 3.45× (3.40–3.50) | 1.33× (1.32–1.34) | 1.49× (1.48–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.40× (1.39–1.42) | 2.74× (2.71–2.77) | 2.42× (2.40–2.44) | 3.25× (3.22–3.28) | 2.85× (2.83–2.87) | 3.06× (3.04–3.08) | 3.10× (3.07–3.14) | 1.20× (1.19–1.21) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.57–1.64) | 2.52× (2.48–2.56) | 2.05× (2.03–2.08) | 2.83× (2.81–2.86) | 2.68× (2.65–2.70) | 2.23× (2.21–2.26) | 2.27× (2.24–2.30) | 0.88× (0.87–0.88) | 0.98× (0.97–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.56× (1.54–1.58) | 3.21× (3.15–3.27) | 2.68× (2.64–2.72) | 2.83× (2.79–2.87) | 3.28× (3.22–3.33) | 3.85× (3.81–3.89) | 4.17× (4.12–4.22) | 1.50× (1.48–1.52) | 1.77× (1.75–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.45–1.47) | 2.99× (2.95–3.03) | 2.48× (2.45–2.52) | 3.51× (3.44–3.58) | 2.95× (2.91–2.98) | 3.40× (3.35–3.46) | 3.69× (3.64–3.74) | 1.33× (1.31–1.35) | 1.56× (1.55–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.69× (1.63–1.74) | 2.68× (2.62–2.73) | 2.07× (2.04–2.10) | 2.96× (2.90–3.03) | 2.68× (2.66–2.70) | 2.38× (2.35–2.40) | 2.58× (2.55–2.61) | 0.93× (0.92–0.94) | 1.09× (1.08–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.72× (2.67–2.77) | 3.58× (3.55–3.60) | 2.88× (2.84–2.93) | 6.14× (6.08–6.19) | 2.36× (2.33–2.39) | 2.17× (2.15–2.19) | 0.99× (0.98–1.01) | 1.03× (0.98–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.59–0.60) | 2.36× (2.34–2.38) | 4.01× (3.97–4.05) | 2.60× (2.57–2.63) | 5.38× (5.32–5.44) | 2.06× (2.04–2.08) | 1.90× (1.88–1.92) | 0.87× (0.86–0.88) | 0.90× (0.85–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.58) | 2.53× (2.43–2.63) | 3.93× (3.84–4.03) | 2.58× (2.48–2.67) | 5.32× (5.15–5.50) | 2.04× (1.96–2.12) | 1.88× (1.81–1.95) | 0.86× (0.83–0.89) | 0.89× (0.83–0.95) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.17× (2.15–2.20) | 4.16× (4.09–4.22) | 4.54× (4.46–4.63) | 4.67× (4.59–4.74) | 4.85× (4.78–4.93) | 5.21× (5.14–5.29) | 4.93× (4.86–5.00) | 1.65× (1.63–1.67) | 1.99× (1.96–2.02) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.78× (1.72–1.83) | 3.42× (3.27–3.56) | 3.70× (3.58–3.82) | 4.15× (3.99–4.32) | 3.87× (3.69–4.06) | 4.00× (3.85–4.15) | 3.78× (3.64–3.92) | 1.26× (1.22–1.30) | 1.52× (1.47–1.58) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.74× (1.68–1.80) | 2.59× (2.53–2.64) | 2.88× (2.81–2.94) | 3.10× (3.04–3.17) | 2.90× (2.84–2.97) | 2.51× (2.46–2.55) | 2.37× (2.32–2.41) | 0.79× (0.77–0.81) | 0.96× (0.94–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.30× (0.29–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.04× (10.99–11.08) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.59× (0.50–0.70) | 0.72× (0.70–0.74) | 0.60× (0.58–0.62) | 5.72× (5.59–5.87) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.36× (0.34–0.38) | 0.70× (0.59–0.84) | 0.64× (0.61–0.68) | 0.57× (0.54–0.60) | 3.74× (3.55–3.93) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.81–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.66–0.67) | 7.95× (7.93–7.96) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.70–0.73) | 0.75× (0.69–0.82) | 1.71× (1.67–1.74) | 0.85× (0.83–0.87) | 6.18× (6.05–6.32) | 0.50× (0.49–0.51) | 0.53× (0.52–0.54) | 0.57× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.62–0.71) | 0.75× (0.68–0.82) | 1.18× (1.11–1.26) | 0.73× (0.69–0.78) | 3.48× (3.27–3.71) | 0.27× (0.26–0.29) | 0.29× (0.27–0.31) | 0.31× (0.29–0.33) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.72× (0.72–0.73) | 5.15× (5.13–5.16) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.38× (0.35–0.41) | 0.90× (0.87–0.93) | 0.73× (0.70–0.75) | 3.11× (3.02–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.82× (0.79–0.84) | 2.56× (2.51–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.84× (0.83–0.84) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.48) | 0.36× (0.33–0.39) | 0.83× (0.81–0.86) | 0.71× (0.69–0.73) | 2.86× (2.79–2.94) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.52) | 0.79× (0.78–0.80) | 0.78× (0.75–0.82) | 2.28× (2.24–2.31) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.19) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.89) | 6.17× (6.15–6.18) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.38–1.39) | 0.65× (0.65–0.66) | 6.36× (6.35–6.37) | 0.38× (0.38–0.39) | 0.41× (0.40–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 35 MB | 0.47× (0.46–0.49) | 0.46× (0.41–0.51) | 1.06× (1.03–1.10) | 0.71× (0.68–0.74) | 3.53× (3.43–3.65) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.17× (0.16–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 58 MB | 0.49× (0.47–0.50) | 0.42× (0.39–0.46) | 0.87× (0.85–0.90) | 0.71× (0.67–0.74) | 2.24× (2.19–2.30) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 436 | 2.31 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 768 | 2.47 | 1.07× (1.06–1.08) | 89% | 1.03× (1.03–1.04) | 1.06× (1.03–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 957 | 3.89 | 1.68× (1.65–1.70) | 55% | 1.22× (1.20–1.23) | 1.64× (1.62–1.66) |

**Best:** mibz2 at 4 threads (957 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.00, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.09 | 0.219 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.4% |
| MTF + RLE2 | 1.54 | 18.1% |
| T-vector build | 1.52 | 17.8% |
| walk (the one each build uses) | 4.11 | 48.3% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.51 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.511 | 1.306 | (in its total) | 2.51× (2.46–2.57) |  |
| walk | 1.232 | 2.629 | (in its total) | 2.09× (2.02–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.054 | (in its total) | 1.18× (1.16–1.20) |  |
| inverse BWT, total | 1.788 | 3.987 | 3.287 | 2.19× (2.12–2.25) | 1.80× (1.73–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m31s |
| 4 threads | 32m04s ± 4m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m26s ± 3m19s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 23m25s ± 3m19s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 3.07 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,259,804 | 15,026,718 | \`b798cab9d5ca7cca…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
