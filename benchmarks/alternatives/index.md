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

The latest record: [`96fbc51`](<https://github.com/lmmx/mibz2/commit/96fbc51c105900b86ec83f68c23d1dd29c97817b>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44407047969>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.507 s | 413 | 6.3 | 3.07× (3.05–3.10) | 6.83× (6.79–6.88) | 7.05× (6.99–7.11) | 7.64× (7.62–7.67) | 6.87× (6.84–6.91) | 7.48× (7.38–7.58) | 6.89× (6.82–6.96) | 2.22× (2.20–2.25) | 2.60× (2.59–2.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.296 s | 707 | 6.9 | 2.97× (2.80–3.16) | 6.21× (5.85–6.58) | 7.06× (6.70–7.43) | 6.96× (6.63–7.31) | 6.28× (5.97–6.59) | 12.56× (11.99–13.16) | 11.58× (11.05–12.13) | 3.74× (3.56–3.92) | 4.37× (4.17–4.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.242 s | 866 | 10.8 | 3.01× (2.95–3.07) | 4.56× (4.44–4.68) | 5.63× (5.47–5.80) | 5.12× (5.04–5.20) | 5.12× (4.99–5.26) | 15.62× (15.33–15.91) | 14.39× (14.14–14.65) | 4.65× (4.58–4.71) | 5.44× (5.35–5.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 998 | 2.9 | 0.61× (0.60–0.62) | 3.85× (3.70–4.02) | 4.64× (4.50–4.79) | 3.64× (3.59–3.68) | 6.22× (6.15–6.30) | 3.03× (2.99–3.07) | 2.71× (2.68–2.75) | 1.03× (1.01–1.05) | 1.53× (1.51–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 968 | 3.0 | 0.57× (0.56–0.58) | 2.98× (2.91–3.04) | 6.09× (5.92–6.26) | 3.55× (3.49–3.62) | 6.08× (5.97–6.19) | 2.94× (2.88–3.00) | 2.63× (2.58–2.69) | 1.00× (0.98–1.03) | 1.49× (1.45–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 957 | 3.0 | 0.57× (0.56–0.58) | 2.94× (2.89–3.00) | 5.93× (5.79–6.07) | 3.55× (3.39–3.71) | 6.00× (5.89–6.10) | 2.90× (2.85–2.95) | 2.59× (2.55–2.63) | 0.99× (0.96–1.02) | 1.46× (1.44–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 571 | 4.8 | 2.86× (2.71–3.02) | 2.97× (2.93–3.01) | 3.81× (3.76–3.86) | 3.77× (3.72–3.82) | 4.08× (4.03–4.13) | 2.55× (2.48–2.61) | 2.09× (2.06–2.12) | 2.00× (1.97–2.03) | 2.96× (2.92–3.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 871 | 6.0 | 2.28× (2.22–2.34) | 3.13× (2.71–3.61) | 3.99× (3.84–4.16) | 3.79× (3.25–4.42) | 4.76× (4.54–4.98) | 3.86× (3.70–4.03) | 3.16× (3.05–3.28) | 3.03× (2.91–3.16) | 4.49× (4.32–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 751 | 11.7 | 1.79× (1.71–1.87) | 1.87× (1.76–1.98) | 2.92× (2.75–3.10) | 2.13× (1.98–2.28) | 3.71× (3.59–3.85) | 3.30× (3.19–3.41) | 2.70× (2.61–2.80) | 2.59× (2.50–2.69) | 3.84× (3.70–3.97) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.136 s | 59 | 44.9 | 1.33× (1.32–1.35) | 2.63× (2.59–2.67) | 2.39× (2.33–2.44) | 2.41× (2.39–2.44) | 2.76× (2.72–2.79) | 3.06× (3.02–3.10) | 3.03× (3.01–3.05) | 1.20× (1.18–1.23) | 1.37× (1.34–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 50.3 | 1.27× (1.25–1.29) | 2.49× (2.44–2.53) | 2.33× (2.29–2.37) | 3.14× (2.94–3.35) | 2.88× (2.82–2.93) | 5.01× (4.93–5.08) | 4.96× (4.89–5.02) | 1.97× (1.94–2.00) | 2.24× (2.20–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 69.4 | 1.31× (1.26–1.36) | 2.13× (2.06–2.20) | 2.13× (2.02–2.25) | 2.56× (2.43–2.70) | 2.76× (2.69–2.83) | 5.91× (5.75–6.06) | 5.85× (5.72–5.99) | 2.33× (2.25–2.40) | 2.64× (2.56–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 65 | 40.6 | 1.47× (1.45–1.48) | 2.99× (2.94–3.04) | 2.52× (2.47–2.56) | 2.60× (2.58–2.63) | 3.02× (2.99–3.05) | 3.56× (3.52–3.59) | 3.80× (3.77–3.84) | 1.39× (1.37–1.41) | 1.67× (1.65–1.70) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 106 | 46.5 | 1.34× (1.24–1.45) | 2.73× (2.42–3.09) | 2.32× (2.19–2.47) | 3.32× (3.10–3.56) | 2.90× (2.74–3.07) | 5.63× (5.32–5.96) | 6.02× (5.69–6.37) | 2.20× (2.08–2.33) | 2.65× (2.49–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 121 | 66.4 | 1.37× (1.33–1.41) | 2.20× (2.14–2.27) | 2.18× (2.11–2.25) | 2.90× (2.83–2.97) | 2.72× (2.67–2.76) | 6.64× (6.52–6.75) | 7.10× (6.96–7.24) | 2.60× (2.54–2.65) | 3.12× (3.04–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 746 | 4.0 | 0.67× (0.66–0.69) | 3.12× (2.95–3.29) | 4.46× (4.38–4.54) | 3.15× (3.10–3.20) | 6.84× (6.73–6.96) | 2.56× (2.52–2.60) | 2.34× (2.30–2.38) | 0.98× (0.97–1.00) | 1.54× (1.51–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 702 | 4.4 | 0.57× (0.56–0.58) | 2.88× (2.84–2.92) | 5.08× (4.89–5.28) | 3.00× (2.95–3.06) | 6.44× (6.34–6.54) | 2.41× (2.38–2.45) | 2.21× (2.17–2.24) | 0.93× (0.92–0.94) | 1.45× (1.43–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 699 | 4.2 | 0.57× (0.57–0.58) | 2.88× (2.84–2.92) | 4.91× (4.85–4.98) | 2.97× (2.93–3.02) | 6.39× (6.29–6.49) | 2.40× (2.37–2.43) | 2.19× (2.16–2.22) | 0.92× (0.90–0.94) | 1.44× (1.42–1.46) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.071 s | 185 | 14.5 | 1.94× (1.88–2.01) | 4.01× (3.87–4.15) | 4.38× (4.20–4.56) | 4.41× (4.27–4.55) | 4.60× (4.43–4.79) | 5.12× (4.93–5.31) | 4.62× (4.46–4.79) | 1.49× (1.45–1.54) | 1.83× (1.76–1.90) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.050 s | 264 | 17.9 | 1.71× (1.52–1.91) | 3.23× (2.98–3.50) | 3.94× (3.67–4.21) | 4.17× (4.00–4.35) | 4.17× (3.94–4.41) | 7.29× (6.92–7.69) | 6.59× (6.24–6.97) | 2.13× (2.00–2.27) | 2.61× (2.47–2.76) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.045 s | 288 | 27.4 | 1.48× (1.43–1.53) | 2.39× (2.27–2.51) | 3.13× (2.97–3.30) | 3.17× (3.04–3.31) | 3.64× (3.52–3.77) | 8.01× (7.81–8.21) | 7.24× (7.07–7.42) | 2.34× (2.28–2.40) | 2.86× (2.79–2.94) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.06× (3.04–3.09) | 6.80× (6.77–6.84) | 7.02× (6.96–7.08) | 7.62× (7.59–7.65) | 6.85× (6.81–6.88) | 7.45× (7.35–7.55) | 6.87× (6.80–6.94) | 2.22× (2.19–2.25) | 2.58× (2.57–2.60) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.92× (2.89–2.96) | 6.46× (6.37–6.54) | 6.74× (6.67–6.81) | 7.39× (7.28–7.50) | 6.38× (6.30–6.46) | 6.77× (6.69–6.86) | 6.24× (6.16–6.32) | 2.02× (1.99–2.05) | 2.35× (2.32–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.01× (2.95–3.08) | 5.00× (4.89–5.10) | 5.38× (5.27–5.49) | 5.63× (5.52–5.75) | 4.98× (4.86–5.11) | 4.42× (4.32–4.53) | 4.07× (3.99–4.16) | 1.32× (1.28–1.35) | 1.53× (1.51–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.34× (3.30–3.37) | 4.17× (4.10–4.24) | 3.40× (3.36–3.44) | 5.75× (5.69–5.82) | 2.85× (2.82–2.88) | 2.56× (2.54–2.59) | 1.03× (1.01–1.06) | 1.08× (1.06–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.06× (3.01–3.11) | 5.37× (5.33–5.41) | 3.29× (3.24–3.34) | 5.54× (5.45–5.62) | 2.72× (2.67–2.77) | 2.44× (2.40–2.49) | 0.99× (0.97–1.01) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.33× (3.27–3.39) | 5.36× (5.27–5.45) | 3.34× (3.20–3.49) | 5.53× (5.44–5.62) | 2.72× (2.68–2.77) | 2.45× (2.42–2.48) | 0.99× (0.96–1.01) | 1.03× (1.01–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.75× (2.61–2.90) | 2.74× (2.71–2.78) | 3.46× (3.42–3.50) | 3.62× (3.57–3.66) | 3.90× (3.85–3.95) | 2.46× (2.40–2.52) | 2.03× (2.00–2.06) | 1.94× (1.91–1.98) | 2.67× (2.63–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.19–2.29) | 2.46× (2.38–2.55) | 3.04× (2.97–3.12) | 3.12× (3.05–3.18) | 3.30× (3.20–3.41) | 1.99× (1.93–2.04) | 1.64× (1.60–1.68) | 1.57× (1.53–1.62) | 2.16× (2.11–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.92–2.10) | 1.70× (1.62–1.78) | 2.19× (2.11–2.27) | 2.00× (1.92–2.09) | 2.07× (1.99–2.15) | 1.03× (0.99–1.07) | 0.85× (0.81–0.88) | 0.81× (0.79–0.84) | 1.12× (1.07–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.34) | 2.56× (2.53–2.59) | 2.34× (2.29–2.38) | 2.39× (2.37–2.42) | 2.73× (2.70–2.76) | 3.03× (3.00–3.06) | 3.00× (2.98–3.03) | 1.20× (1.18–1.23) | 1.33× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.27) | 2.39× (2.36–2.43) | 2.11× (2.08–2.14) | 2.84× (2.80–2.88) | 2.50× (2.46–2.53) | 2.69× (2.65–2.73) | 2.67× (2.63–2.71) | 1.07× (1.06–1.08) | 1.19× (1.17–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.40–1.47) | 2.20× (2.12–2.27) | 1.79× (1.74–1.85) | 2.47× (2.42–2.52) | 2.34× (2.29–2.39) | 1.97× (1.92–2.01) | 1.95× (1.89–2.01) | 0.78× (0.76–0.80) | 0.87× (0.85–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.45–1.47) | 2.93× (2.90–2.98) | 2.47× (2.43–2.51) | 2.58× (2.56–2.61) | 2.99× (2.97–3.02) | 3.52× (3.49–3.55) | 3.77× (3.73–3.80) | 1.39× (1.37–1.40) | 1.63× (1.61–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.27–1.39) | 2.65× (2.53–2.78) | 2.20× (2.10–2.29) | 3.10× (2.95–3.24) | 2.62× (2.51–2.74) | 3.03× (2.90–3.17) | 3.24× (3.10–3.40) | 1.19× (1.14–1.25) | 1.41× (1.34–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.56× (1.50–1.61) | 2.47× (2.40–2.53) | 1.89× (1.85–1.94) | 2.74× (2.68–2.80) | 2.46× (2.41–2.51) | 2.17× (2.12–2.23) | 2.32× (2.26–2.39) | 0.86× (0.83–0.88) | 1.01× (0.98–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.73) | 2.73× (2.69–2.77) | 3.61× (3.57–3.65) | 2.88× (2.83–2.92) | 6.10× (6.00–6.19) | 2.36× (2.32–2.39) | 2.17× (2.13–2.20) | 0.99× (0.98–1.00) | 1.00× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.63–0.64) | 2.50× (2.48–2.53) | 4.29× (4.23–4.35) | 2.76× (2.73–2.80) | 5.67× (5.60–5.75) | 2.19× (2.16–2.21) | 2.01× (1.99–2.03) | 0.92× (0.91–0.93) | 0.92× (0.89–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.62–0.63) | 2.80× (2.76–2.84) | 4.28× (4.23–4.33) | 2.82× (2.78–2.85) | 5.76× (5.68–5.84) | 2.23× (2.20–2.25) | 2.05× (2.03–2.07) | 0.93× (0.92–0.95) | 0.94× (0.92–0.97) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.92× (1.86–1.98) | 3.88× (3.75–4.01) | 4.21× (4.05–4.37) | 4.32× (4.18–4.45) | 4.50× (4.33–4.68) | 5.00× (4.82–5.18) | 4.52× (4.37–4.68) | 1.48× (1.44–1.52) | 1.75× (1.69–1.81) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.69× (1.65–1.74) | 3.33× (3.24–3.42) | 3.69× (3.57–3.81) | 4.12× (4.05–4.20) | 3.79× (3.72–3.85) | 4.11× (4.04–4.18) | 3.72× (3.66–3.78) | 1.22× (1.19–1.24) | 1.44× (1.42–1.46) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.71× (1.63–1.79) | 2.71× (2.64–2.78) | 2.94× (2.85–3.02) | 3.23× (3.16–3.30) | 3.07× (3.01–3.13) | 2.69× (2.65–2.73) | 2.43× (2.39–2.48) | 0.80× (0.78–0.81) | 0.94× (0.93–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.30× (0.28–0.33) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.51× (0.45–0.57) | 0.69× (0.67–0.71) | 0.57× (0.55–0.59) | 5.44× (5.26–5.62) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.57× (0.49–0.66) | 0.59× (0.57–0.62) | 0.53× (0.50–0.56) | 3.48× (3.33–3.64) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.66) | 8.04× (8.02–8.07) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.68) | 0.71× (0.65–0.77) | 1.63× (1.61–1.64) | 0.82× (0.81–0.83) | 5.89× (5.84–5.94) | 0.48× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.54) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.63× (0.60–0.66) | 0.69× (0.65–0.73) | 1.12× (1.08–1.16) | 0.69× (0.65–0.72) | 3.30× (3.17–3.44) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.53) | 0.42× (0.39–0.44) | 0.91× (0.89–0.94) | 0.72× (0.70–0.75) | 3.15× (3.08–3.22) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.44–0.48) | 0.87× (0.85–0.90) | 0.81× (0.78–0.84) | 2.54× (2.46–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.18× (5.18–5.19) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.41) | 0.85× (0.82–0.88) | 0.72× (0.68–0.76) | 2.90× (2.80–3.02) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.47× (0.46–0.49) | 0.80× (0.79–0.82) | 0.81× (0.78–0.84) | 2.31× (2.26–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.30) | 0.34× (0.34–0.35) | 0.36× (0.35–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.36× (0.35–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.66) | 6.37× (6.36–6.39) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.62) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 34 MB | 0.49× (0.49–0.50) | 0.42× (0.39–0.46) | 1.15× (1.12–1.17) | 0.73× (0.71–0.75) | 3.62× (3.58–3.67) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.17–0.18) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 56 MB | 0.51× (0.49–0.52) | 0.50× (0.46–0.54) | 0.92× (0.90–0.94) | 0.74× (0.72–0.77) | 2.35× (2.31–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 413 | 2.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 707 | 2.65 | 1.10× (1.09–1.11) | 84% | 1.04× (1.03–1.06) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 866 | 4.12 | 1.69× (1.66–1.71) | 52% | 1.24× (1.22–1.26) | 1.66× (1.62–1.70) |

**Best:** mibz2 at 4 threads (866 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (287 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.22, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.51, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.68 | 0.02 | 0.08 | 0.242 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2610 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.90 | 12.4% |
| MTF + RLE2 | 1.28 | 17.5% |
| T-vector build | 1.22 | 16.8% |
| walk (the one each build uses) | 3.43 | 47.0% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.23 | 3.1% |
| CRC | 0.21 | 2.8% |
| **total** | 7.29 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.306 | (in its total) | 2.66× (2.64–2.68) |  |
| walk | 1.233 | 2.652 | (in its total) | 2.15× (2.13–2.18) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.057 | (in its total) | 1.28× (1.25–1.30) |  |
| inverse BWT, total | 1.770 | 4.015 | 3.313 | 2.27× (2.25–2.29) | 1.87× (1.86–1.89) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m23s |
| 4 threads | 35m20s ± 4m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m11s ± 3m27s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 24m11s ± 3m27s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.17 ns a byte of output at 4 threads (CPU, summed over them), 1.96 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,747 | 13,092,730 | \`9b1907f0616f895e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2609.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
