# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-442/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`968b59d`](<https://github.com/lmmx/mibz2/commit/968b59d3c60e35ab301753a02473c3124e27e1c8>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43754694535>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.523 s | 401 | 8.1 | 2.89× (2.86–2.93) | 6.58× (6.52–6.64) | 6.74× (6.68–6.80) | 7.34× (7.28–7.41) | 6.59× (6.52–6.67) | 7.19× (7.12–7.26) | 6.66× (6.61–6.71) | 2.15× (2.13–2.17) | 2.48× (2.45–2.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.296 s | 708 | 8.9 | 2.84× (2.78–2.91) | 6.10× (5.91–6.28) | 6.91× (6.68–7.16) | 6.83× (6.61–7.06) | 6.19× (6.03–6.35) | 12.54× (12.22–12.87) | 11.62× (11.31–11.93) | 3.74× (3.66–3.83) | 4.32× (4.22–4.42) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.245 s | 856 | 13.7 | 2.88× (2.84–2.93) | 4.42× (4.19–4.68) | 5.42× (5.30–5.55) | 4.90× (4.84–4.97) | 4.94× (4.90–4.99) | 15.34× (15.15–15.52) | 14.21× (14.05–14.37) | 4.58× (4.52–4.64) | 5.28× (5.21–5.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 983 | 3.6 | 0.60× (0.60–0.61) | 3.80× (3.63–3.98) | 4.48× (4.38–4.58) | 3.58× (3.55–3.61) | 6.15× (6.08–6.21) | 3.00× (2.97–3.02) | 2.67× (2.64–2.70) | 1.02× (1.00–1.03) | 1.51× (1.50–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 3.8 | 0.57× (0.56–0.58) | 2.96× (2.89–3.03) | 5.99× (5.86–6.11) | 3.53× (3.46–3.60) | 6.02× (5.93–6.10) | 2.93× (2.87–3.00) | 2.61× (2.56–2.67) | 1.00× (0.98–1.02) | 1.48× (1.45–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 959 | 3.7 | 0.54× (0.50–0.59) | 2.83× (2.59–3.10) | 5.57× (5.09–6.08) | 3.36× (3.07–3.68) | 5.78× (5.29–6.32) | 2.79× (2.56–3.06) | 2.49× (2.28–2.72) | 0.95× (0.87–1.03) | 1.41× (1.29–1.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 588 | 5.8 | 2.90× (2.80–3.00) | 3.06× (3.04–3.08) | 3.93× (3.90–3.96) | 3.85× (3.84–3.87) | 4.17× (4.15–4.20) | 2.58× (2.56–2.60) | 2.15× (2.09–2.21) | 2.09× (2.07–2.10) | 3.05× (3.03–3.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 845 | 7.4 | 2.19× (2.05–2.33) | 2.83× (2.53–3.16) | 4.01× (3.76–4.27) | 3.39× (3.20–3.60) | 4.62× (4.33–4.92) | 3.72× (3.50–3.96) | 3.10× (2.92–3.30) | 3.01× (2.83–3.21) | 4.40× (4.13–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 781 | 14.6 | 1.83× (1.78–1.89) | 1.99× (1.83–2.17) | 2.95× (2.80–3.10) | 2.18× (2.09–2.26) | 3.85× (3.75–3.94) | 3.40× (3.30–3.50) | 2.83× (2.71–2.95) | 2.75× (2.67–2.83) | 4.01× (3.89–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 53.0 | 1.38× (1.36–1.40) | 2.72× (2.69–2.75) | 2.41× (2.37–2.44) | 2.47× (2.44–2.51) | 2.89× (2.86–2.92) | 3.16× (3.12–3.19) | 3.19× (3.13–3.25) | 1.23× (1.22–1.24) | 1.42× (1.41–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 98 | 60.0 | 1.29× (1.25–1.33) | 2.50× (2.39–2.61) | 2.30× (2.20–2.39) | 2.89× (2.77–3.01) | 2.88× (2.77–2.99) | 4.99× (4.79–5.19) | 5.04× (4.82–5.27) | 1.94× (1.87–2.01) | 2.25× (2.17–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 118 | 83.7 | 1.36× (1.33–1.39) | 2.17× (2.13–2.22) | 2.10× (2.01–2.19) | 2.48× (2.41–2.54) | 2.84× (2.77–2.91) | 6.02× (5.90–6.14) | 6.08× (5.93–6.24) | 2.34× (2.31–2.38) | 2.71× (2.67–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 48.8 | 1.52× (1.49–1.54) | 3.07× (3.04–3.10) | 2.54× (2.53–2.56) | 2.65× (2.63–2.66) | 3.13× (3.05–3.21) | 3.66× (3.64–3.69) | 3.92× (3.86–3.97) | 1.42× (1.41–1.43) | 1.70× (1.69–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.6 | 1.36× (1.22–1.51) | 2.68× (2.42–2.98) | 2.36× (2.15–2.58) | 3.15× (2.90–3.41) | 2.93× (2.69–3.21) | 5.73× (5.25–6.25) | 6.13× (5.60–6.71) | 2.22× (2.03–2.42) | 2.67× (2.44–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 124 | 79.9 | 1.41× (1.38–1.46) | 2.22× (2.16–2.28) | 2.11× (2.06–2.15) | 2.79× (2.69–2.88) | 2.72× (2.65–2.78) | 6.70× (6.53–6.87) | 7.17× (6.97–7.36) | 2.59× (2.54–2.64) | 3.12× (3.04–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 5.0 | 0.67× (0.66–0.67) | 3.19× (3.01–3.38) | 4.47× (4.41–4.53) | 3.13× (3.11–3.16) | 6.82× (6.72–6.92) | 2.55× (2.52–2.58) | 2.34× (2.31–2.37) | 0.98× (0.96–1.00) | 1.54× (1.52–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 707 | 5.4 | 0.58× (0.57–0.59) | 2.90× (2.83–2.96) | 5.04× (4.85–5.25) | 3.02× (2.97–3.07) | 6.48× (6.35–6.60) | 2.42× (2.36–2.47) | 2.22× (2.17–2.27) | 0.93× (0.91–0.95) | 1.46× (1.43–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 694 | 5.3 | 0.57× (0.56–0.58) | 2.86× (2.80–2.93) | 4.90× (4.80–5.01) | 2.96× (2.90–3.02) | 6.38× (6.26–6.51) | 2.38× (2.32–2.43) | 2.18× (2.13–2.23) | 0.91× (0.89–0.94) | 1.44× (1.40–1.48) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 0.068 s | 186 | 17.9 | 1.98× (1.96–2.00) | 4.09× (4.04–4.15) | 4.41× (4.36–4.47) | 4.47× (4.40–4.53) | 4.68× (4.63–4.74) | 5.21× (5.14–5.27) | 4.78× (4.70–4.87) | 1.50× (1.48–1.52) | 1.92× (1.89–1.95) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 0.049 s | 261 | 23.6 | 1.58× (1.51–1.66) | 3.18× (3.04–3.32) | 3.70× (3.57–3.83) | 4.13× (4.03–4.23) | 4.09× (3.94–4.24) | 7.26× (7.01–7.51) | 6.66× (6.41–6.92) | 2.09× (2.02–2.16) | 2.67× (2.58–2.77) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 0.044 s | 291 | 35.0 | 1.48× (1.43–1.53) | 2.33× (2.23–2.44) | 3.00× (2.92–3.08) | 3.20× (3.15–3.25) | 3.62× (3.56–3.67) | 8.19× (8.06–8.32) | 7.52× (7.36–7.69) | 2.35× (2.32–2.39) | 3.02× (2.97–3.07) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.89× (2.85–2.92) | 6.55× (6.49–6.61) | 6.71× (6.65–6.77) | 7.32× (7.25–7.39) | 6.57× (6.49–6.64) | 7.17× (7.10–7.24) | 6.64× (6.59–6.69) | 2.14× (2.12–2.16) | 2.46× (2.43–2.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.74× (2.71–2.77) | 6.12× (6.03–6.20) | 6.45× (6.37–6.53) | 6.99× (6.91–7.07) | 6.08× (6.00–6.15) | 6.53× (6.43–6.63) | 6.05× (5.95–6.15) | 1.95× (1.92–1.98) | 2.24× (2.21–2.27) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.83× (2.79–2.87) | 4.73× (4.63–4.84) | 5.13× (5.06–5.21) | 5.32× (5.25–5.39) | 4.74× (4.66–4.82) | 4.29× (4.21–4.37) | 3.98× (3.90–4.05) | 1.28× (1.26–1.30) | 1.47× (1.45–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.30× (3.27–3.33) | 4.11× (4.03–4.19) | 3.36× (3.33–3.40) | 5.70× (5.63–5.77) | 2.82× (2.79–2.86) | 2.53× (2.50–2.56) | 1.02× (1.00–1.03) | 1.08× (1.04–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.04× (2.99–3.10) | 5.34× (5.25–5.42) | 3.28× (3.22–3.34) | 5.50× (5.43–5.57) | 2.71× (2.66–2.77) | 2.43× (2.38–2.48) | 0.98× (0.96–1.00) | 1.04× (1.01–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.54–0.63) | 3.25× (3.01–3.52) | 5.15× (4.75–5.58) | 3.19× (2.94–3.47) | 5.37× (4.95–5.84) | 2.64× (2.43–2.87) | 2.37× (2.17–2.57) | 0.95× (0.88–1.03) | 1.01× (0.91–1.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.69–2.89) | 2.82× (2.80–2.83) | 3.54× (3.53–3.55) | 3.69× (3.67–3.71) | 3.99× (3.97–4.02) | 2.49× (2.47–2.51) | 2.08× (2.03–2.14) | 2.02× (2.01–2.04) | 2.74× (2.70–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.18× (2.05–2.32) | 2.46× (2.30–2.62) | 2.96× (2.80–3.14) | 3.17× (3.00–3.35) | 3.24× (3.05–3.44) | 1.93× (1.82–2.05) | 1.62× (1.52–1.72) | 1.57× (1.48–1.67) | 2.12× (2.00–2.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.92–2.04) | 1.71× (1.64–1.79) | 2.19× (2.12–2.26) | 2.11× (2.04–2.17) | 2.06× (1.98–2.13) | 1.02× (0.98–1.05) | 0.85× (0.82–0.88) | 0.83× (0.80–0.85) | 1.12× (1.09–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.39) | 2.65× (2.61–2.68) | 2.36× (2.33–2.39) | 2.45× (2.41–2.48) | 2.86× (2.83–2.89) | 3.12× (3.09–3.16) | 3.16× (3.10–3.22) | 1.23× (1.21–1.24) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.31) | 2.47× (2.43–2.51) | 2.18× (2.15–2.20) | 2.79× (2.74–2.83) | 2.57× (2.53–2.61) | 2.76× (2.72–2.81) | 2.79× (2.74–2.85) | 1.08× (1.07–1.10) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.43–1.54) | 2.23× (2.17–2.29) | 1.81× (1.77–1.85) | 2.46× (2.40–2.53) | 2.40× (2.34–2.46) | 2.00× (1.96–2.04) | 2.02× (1.97–2.08) | 0.78× (0.77–0.80) | 0.88× (0.87–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.49–1.53) | 3.01× (2.98–3.03) | 2.50× (2.49–2.52) | 2.62× (2.61–2.64) | 3.10× (3.02–3.18) | 3.63× (3.60–3.65) | 3.88× (3.82–3.93) | 1.41× (1.40–1.42) | 1.66× (1.66–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.30–1.46) | 2.69× (2.58–2.81) | 2.23× (2.14–2.31) | 2.98× (2.86–3.11) | 2.66× (2.56–2.77) | 3.09× (2.98–3.21) | 3.30× (3.16–3.45) | 1.20× (1.16–1.25) | 1.42× (1.36–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.55–1.64) | 2.46× (2.40–2.51) | 1.89× (1.84–1.93) | 2.65× (2.59–2.71) | 2.46× (2.41–2.51) | 2.20× (2.15–2.25) | 2.35× (2.28–2.42) | 0.86× (0.83–0.88) | 1.01× (0.99–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.75× (2.72–2.77) | 3.57× (3.52–3.62) | 2.87× (2.84–2.90) | 6.10× (5.99–6.20) | 2.36× (2.32–2.40) | 2.18× (2.14–2.21) | 0.99× (0.97–1.01) | 0.99× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.63–0.65) | 2.51× (2.48–2.55) | 4.25× (4.16–4.34) | 2.76× (2.73–2.80) | 5.68× (5.58–5.78) | 2.18× (2.14–2.23) | 2.01× (1.97–2.05) | 0.91× (0.89–0.94) | 0.92× (0.89–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.76× (2.72–2.80) | 4.30× (4.21–4.39) | 2.80× (2.75–2.85) | 5.77× (5.67–5.87) | 2.21× (2.17–2.26) | 2.04× (2.00–2.08) | 0.93× (0.91–0.95) | 0.93× (0.90–0.97) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 1.95× (1.93–1.97) | 3.93× (3.89–3.98) | 4.26× (4.22–4.30) | 4.37× (4.31–4.43) | 4.58× (4.53–4.63) | 5.09× (5.02–5.15) | 4.67× (4.59–4.76) | 1.48× (1.47–1.50) | 1.77× (1.75–1.80) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 1.61× (1.58–1.63) | 3.16× (3.11–3.21) | 3.46× (3.42–3.51) | 3.81× (3.76–3.86) | 3.61× (3.55–3.66) | 3.91× (3.86–3.97) | 3.60× (3.53–3.66) | 1.14× (1.13–1.15) | 1.36× (1.35–1.38) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 1.69× (1.62–1.76) | 2.56× (2.50–2.63) | 2.81× (2.73–2.89) | 3.06× (2.98–3.14) | 2.95× (2.88–3.02) | 2.60× (2.54–2.66) | 2.39× (2.33–2.45) | 0.76× (0.74–0.78) | 0.91× (0.88–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.30× (0.29–0.32) | 0.98× (0.97–0.98) | 0.78× (0.78–0.78) | 11.04× (11.02–11.06) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.35) | 0.52× (0.47–0.59) | 0.71× (0.69–0.74) | 0.59× (0.58–0.61) | 5.65× (5.46–5.84) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.57× (0.48–0.68) | 0.60× (0.57–0.62) | 0.52× (0.50–0.55) | 3.48× (3.35–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.03× (8.01–8.05) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.70× (0.66–0.75) | 1.62× (1.60–1.65) | 0.81× (0.79–0.82) | 5.89× (5.80–5.99) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.55× (0.54–0.56) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.66) | 0.67× (0.62–0.74) | 1.15× (1.10–1.20) | 0.70× (0.67–0.72) | 3.36× (3.23–3.49) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.50–0.53) | 0.41× (0.37–0.45) | 0.92× (0.89–0.95) | 0.74× (0.72–0.77) | 3.18× (3.08–3.29) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.67) | 0.45× (0.44–0.47) | 0.87× (0.85–0.89) | 0.82× (0.79–0.84) | 2.53× (2.47–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.21× (5.20–5.23) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.39) | 0.84× (0.81–0.88) | 0.70× (0.66–0.74) | 2.88× (2.77–3.00) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.50× (0.47–0.53) | 0.80× (0.79–0.82) | 0.81× (0.79–0.83) | 2.32× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.37–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.21× (0.20–0.21) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.31× (5.29–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.33× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.67) | 6.39× (6.36–6.42) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.30–0.31) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 38 MB | 0.43× (0.42–0.45) | 0.38× (0.35–0.42) | 1.03× (1.01–1.06) | 0.66× (0.65–0.68) | 3.21× (3.13–3.29) | 0.19× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.15× (0.15–0.16) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 58 MB | 0.48× (0.46–0.50) | 0.45× (0.41–0.50) | 0.89× (0.85–0.92) | 0.69× (0.66–0.73) | 2.24× (2.17–2.31) | 0.13× (0.12–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 401 | 2.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 708 | 2.73 | 1.10× (1.08–1.12) | 87% | 1.03× (1.02–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 856 | 4.22 | 1.67× (1.63–1.71) | 53% | 1.21× (1.18–1.23) | 1.64× (1.60–1.67) |

**Best:** mibz2 at 4 threads (856 MB/s); lbzip2 at 4 threads (198 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.20, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.50, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.69 | 0.02 | 0.07 | 0.245 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.9% |
| MTF + RLE2 | 1.52 | 17.2% |
| T-vector build | 1.52 | 17.2% |
| walk (the one each build uses) | 4.09 | 46.4% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.483 | 1.301 | (in its total) | 2.69× (2.69–2.70) |  |
| walk | 1.190 | 2.602 | (in its total) | 2.19× (2.18–2.20) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.053 | (in its total) | 1.20× (1.19–1.21) |  |
| inverse BWT, total | 1.718 | 3.956 | 3.228 | 2.31× (2.30–2.31) | 1.88× (1.87–1.89) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m48s |
| 4 threads | 35m46s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m53s ± 3m34s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 24m53s ± 3m34s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.26 ns a byte of output at 4 threads (CPU, summed over them), 2.06 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,524,062 | 12,693,642 | \`8a23ef84eb869287…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
