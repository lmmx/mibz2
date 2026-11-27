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

The latest record: [`996dc3e`](<https://github.com/lmmx/mibz2/commit/996dc3e9cacf2d174c129a8a2f7b674eb9251afe>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46055917928>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.486 s | 432 | 7.5 | 3.19× (3.18–3.20) | 7.24× (7.20–7.28) | 7.46× (7.41–7.50) | 8.04× (8.00–8.08) | 7.25× (7.21–7.29) | 7.92× (7.88–7.97) | 7.41× (7.37–7.46) | 2.34× (2.33–2.36) | 2.72× (2.71–2.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.267 s | 785 | 8.0 | 3.28× (3.11–3.46) | 6.85× (6.55–7.16) | 7.79× (7.49–8.10) | 7.64× (7.41–7.88) | 6.90× (6.68–7.13) | 14.18× (13.74–14.64) | 13.27× (12.87–13.69) | 4.20× (4.06–4.33) | 4.87× (4.73–5.02) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.221 s | 947 | 12.6 | 3.24× (3.14–3.34) | 4.92× (4.79–5.04) | 6.06× (5.93–6.20) | 5.60× (5.45–5.75) | 5.65× (5.50–5.81) | 17.19× (16.82–17.57) | 16.08× (15.76–16.41) | 5.08× (4.98–5.19) | 5.91× (5.81–6.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.6 | 0.61× (0.60–0.61) | 3.72× (3.53–3.93) | 4.61× (4.45–4.77) | 3.60× (3.56–3.64) | 6.24× (6.17–6.32) | 3.00× (2.97–3.04) | 2.68× (2.65–2.71) | 1.02× (1.00–1.03) | 1.52× (1.50–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 930 | 3.9 | 0.55× (0.53–0.56) | 2.86× (2.80–2.93) | 5.65× (5.52–5.77) | 3.42× (3.34–3.49) | 5.82× (5.70–5.95) | 2.83× (2.77–2.89) | 2.53× (2.47–2.58) | 0.96× (0.93–0.98) | 1.43× (1.40–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 915 | 4.0 | 0.55× (0.54–0.56) | 2.83× (2.79–2.87) | 5.66× (5.54–5.78) | 3.34× (3.30–3.39) | 5.78× (5.67–5.89) | 2.79× (2.75–2.83) | 2.49× (2.46–2.52) | 0.94× (0.93–0.96) | 1.41× (1.39–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 595 | 5.8 | 2.70× (2.68–2.73) | 3.09× (3.07–3.12) | 4.00× (3.92–4.08) | 3.90× (3.86–3.94) | 4.26× (4.22–4.31) | 2.61× (2.59–2.63) | 2.16× (2.14–2.17) | 2.14× (2.11–2.17) | 3.08× (3.05–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 940 | 6.9 | 2.42× (2.24–2.61) | 3.03× (2.79–3.29) | 4.28× (3.96–4.62) | 3.54× (3.36–3.73) | 5.03× (4.80–5.28) | 4.03× (3.83–4.24) | 3.33× (3.17–3.50) | 3.30× (3.14–3.47) | 4.76× (4.53–5.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 852 | 13.8 | 2.01× (1.96–2.06) | 2.17× (2.02–2.32) | 3.28× (3.10–3.47) | 2.39× (2.31–2.48) | 4.24× (4.11–4.38) | 3.73× (3.66–3.81) | 3.09× (3.02–3.16) | 3.06× (2.98–3.14) | 4.41× (4.31–4.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.120 s | 67 | 49.5 | 1.51× (1.50–1.52) | 2.97× (2.96–2.99) | 2.68× (2.64–2.71) | 2.71× (2.69–2.72) | 3.14× (3.11–3.16) | 3.46× (3.43–3.50) | 3.48× (3.45–3.51) | 1.35× (1.33–1.37) | 1.53× (1.50–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 112 | 54.6 | 1.43× (1.32–1.55) | 2.89× (2.57–3.25) | 2.61× (2.41–2.82) | 3.34× (3.09–3.61) | 3.22× (2.98–3.48) | 5.65× (5.22–6.11) | 5.67× (5.24–6.14) | 2.20× (2.04–2.37) | 2.49× (2.33–2.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 133 | 74.6 | 1.51× (1.47–1.56) | 2.45× (2.42–2.48) | 2.38× (2.35–2.41) | 2.91× (2.86–2.97) | 3.25× (3.20–3.30) | 6.97× (6.86–7.08) | 7.00× (6.90–7.09) | 2.71× (2.68–2.74) | 3.07× (3.03–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.133 s | 72 | 45.7 | 1.68× (1.55–1.82) | 3.31× (3.28–3.35) | 2.80× (2.77–2.82) | 2.88× (2.85–2.91) | 3.35× (3.31–3.39) | 3.96× (3.92–4.00) | 4.30× (4.26–4.35) | 1.54× (1.52–1.55) | 1.84× (1.81–1.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.081 s | 119 | 51.1 | 1.48× (1.43–1.53) | 2.98× (2.88–3.09) | 2.69× (2.57–2.82) | 3.74× (3.48–4.02) | 3.30× (3.21–3.40) | 6.46× (6.26–6.66) | 7.01× (6.82–7.22) | 2.51× (2.45–2.56) | 3.00× (2.92–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 72.9 | 1.61× (1.58–1.65) | 2.62× (2.51–2.72) | 2.59× (2.50–2.68) | 3.32× (3.26–3.38) | 3.16× (3.11–3.21) | 7.80× (7.70–7.91) | 8.48× (8.34–8.62) | 3.03× (2.97–3.09) | 3.63× (3.55–3.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.0 | 0.66× (0.65–0.67) | 3.23× (3.01–3.47) | 4.43× (4.36–4.49) | 3.12× (3.08–3.17) | 6.80× (6.71–6.90) | 2.54× (2.50–2.57) | 2.34× (2.30–2.38) | 0.99× (0.97–1.01) | 1.52× (1.50–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 5.8 | 0.55× (0.53–0.56) | 2.78× (2.74–2.82) | 4.83× (4.71–4.95) | 2.94× (2.83–3.07) | 6.33× (6.23–6.43) | 2.33× (2.29–2.36) | 2.15× (2.12–2.18) | 0.91× (0.89–0.93) | 1.40× (1.38–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 662 | 5.7 | 0.54× (0.53–0.56) | 2.75× (2.70–2.79) | 4.70× (4.63–4.78) | 2.84× (2.79–2.89) | 6.17× (6.03–6.32) | 2.29× (2.25–2.33) | 2.11× (2.07–2.15) | 0.89× (0.87–0.91) | 1.38× (1.35–1.40) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.065 s | 230 | 14.5 | 2.20× (2.17–2.22) | 4.29× (4.24–4.33) | 4.71× (4.65–4.78) | 4.71× (4.67–4.76) | 4.89× (4.85–4.94) | 5.32× (5.27–5.36) | 5.04× (4.99–5.08) | 1.67× (1.65–1.68) | 2.08× (2.04–2.12) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.045 s | 332 | 18.8 | 1.75× (1.59–1.91) | 3.44× (3.17–3.73) | 4.02× (3.81–4.24) | 4.43× (4.04–4.87) | 4.25× (4.01–4.50) | 7.40× (6.98–7.83) | 7.01× (6.63–7.40) | 2.32× (2.19–2.45) | 2.89× (2.75–3.03) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.040 s | 370 | 28.9 | 1.67× (1.61–1.72) | 2.61× (2.48–2.73) | 3.59× (3.46–3.72) | 3.64× (3.55–3.74) | 3.93× (3.86–4.00) | 8.52× (8.40–8.65) | 8.08× (7.97–8.18) | 2.67× (2.62–2.72) | 3.33× (3.24–3.42) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.18× (3.17–3.20) | 7.20× (7.16–7.24) | 7.42× (7.37–7.47) | 8.01× (7.97–8.06) | 7.23× (7.18–7.27) | 7.89× (7.85–7.94) | 7.39× (7.34–7.43) | 2.34× (2.32–2.35) | 2.70× (2.69–2.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.06–3.10) | 6.91× (6.83–6.98) | 7.29× (7.24–7.34) | 7.91× (7.87–7.96) | 6.81× (6.77–6.84) | 7.42× (7.38–7.46) | 6.94× (6.92–6.97) | 2.20× (2.19–2.21) | 2.54× (2.53–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.11× (3.06–3.16) | 5.24× (5.20–5.27) | 5.62× (5.54–5.70) | 5.93× (5.87–6.00) | 5.27× (5.19–5.35) | 4.71× (4.67–4.76) | 4.41× (4.36–4.46) | 1.40× (1.38–1.41) | 1.61× (1.59–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.29× (3.23–3.34) | 4.17× (4.08–4.27) | 3.37× (3.33–3.41) | 5.77× (5.70–5.84) | 2.82× (2.79–2.85) | 2.53× (2.51–2.55) | 1.01× (1.00–1.03) | 1.07× (1.05–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.89× (2.83–2.96) | 5.06× (4.95–5.16) | 3.12× (3.05–3.19) | 5.23× (5.12–5.35) | 2.58× (2.52–2.63) | 2.31× (2.26–2.36) | 0.93× (0.90–0.95) | 0.98× (0.95–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.11× (3.06–3.16) | 5.04× (4.98–5.09) | 3.10× (3.07–3.13) | 5.24× (5.16–5.32) | 2.58× (2.55–2.61) | 2.31× (2.28–2.33) | 0.92× (0.91–0.94) | 0.98× (0.95–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.58–2.63) | 2.86× (2.84–2.88) | 3.65× (3.54–3.75) | 3.73× (3.70–3.77) | 4.08× (4.04–4.12) | 2.52× (2.49–2.54) | 2.09× (2.08–2.11) | 2.07× (2.05–2.10) | 2.76× (2.73–2.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.33–2.40) | 2.64× (2.56–2.72) | 3.22× (3.15–3.29) | 3.29× (3.23–3.35) | 3.54× (3.48–3.60) | 2.10× (2.06–2.14) | 1.75× (1.72–1.77) | 1.73× (1.69–1.77) | 2.31× (2.27–2.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.06–2.20) | 1.81× (1.74–1.89) | 2.26× (2.17–2.36) | 2.21× (2.11–2.30) | 2.19× (2.11–2.28) | 1.08× (1.03–1.13) | 0.90× (0.86–0.93) | 0.89× (0.85–0.93) | 1.18× (1.13–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.50× (1.49–1.51) | 2.90× (2.88–2.91) | 2.61× (2.58–2.63) | 2.68× (2.66–2.69) | 3.10× (3.07–3.13) | 3.42× (3.39–3.46) | 3.44× (3.41–3.47) | 1.34× (1.33–1.36) | 1.49× (1.48–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.43× (1.42–1.45) | 2.76× (2.71–2.81) | 2.41× (2.38–2.45) | 3.20× (3.16–3.24) | 2.85× (2.82–2.89) | 3.10× (3.06–3.13) | 3.11× (3.07–3.15) | 1.22× (1.20–1.24) | 1.35× (1.33–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.57–1.69) | 2.56× (2.52–2.59) | 2.06× (2.02–2.10) | 2.82× (2.77–2.87) | 2.71× (2.67–2.74) | 2.27× (2.24–2.31) | 2.28× (2.25–2.32) | 0.89× (0.88–0.90) | 0.99× (0.98–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.67× (1.54–1.81) | 3.24× (3.21–3.28) | 2.72× (2.70–2.75) | 2.85× (2.82–2.88) | 3.32× (3.28–3.35) | 3.92× (3.88–3.95) | 4.25× (4.21–4.30) | 1.53× (1.51–1.55) | 1.80× (1.77–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.53× (1.51–1.55) | 3.05× (3.03–3.08) | 2.54× (2.52–2.56) | 3.51× (3.47–3.55) | 3.02× (2.99–3.05) | 3.51× (3.48–3.53) | 3.81× (3.78–3.84) | 1.37× (1.35–1.39) | 1.61× (1.59–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.74× (1.67–1.82) | 2.75× (2.69–2.81) | 2.14× (2.09–2.19) | 3.00× (2.93–3.07) | 2.76× (2.72–2.81) | 2.46× (2.43–2.50) | 2.67× (2.63–2.72) | 0.96× (0.94–0.98) | 1.13× (1.11–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.72) | 2.73× (2.70–2.76) | 3.59× (3.51–3.68) | 2.85× (2.82–2.89) | 6.07× (5.99–6.14) | 2.34× (2.31–2.37) | 2.17× (2.14–2.21) | 0.99× (0.97–1.01) | 0.99× (0.94–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.58–0.61) | 2.36× (2.33–2.40) | 4.08× (4.02–4.14) | 2.65× (2.55–2.75) | 5.45× (5.36–5.54) | 2.07× (2.03–2.10) | 1.92× (1.89–1.94) | 0.87× (0.85–0.90) | 0.87× (0.84–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.59) | 2.57× (2.50–2.63) | 3.99× (3.93–4.06) | 2.62× (2.58–2.67) | 5.43× (5.31–5.56) | 2.08× (2.04–2.11) | 1.93× (1.89–1.97) | 0.88× (0.86–0.90) | 0.88× (0.84–0.92) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 2.16× (2.13–2.18) | 4.15× (4.11–4.19) | 4.54× (4.47–4.60) | 4.60× (4.55–4.64) | 4.77× (4.73–4.82) | 5.18× (5.14–5.23) | 4.91× (4.87–4.96) | 1.65× (1.63–1.66) | 1.99× (1.96–2.01) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.78× (1.75–1.81) | 3.34× (3.28–3.40) | 3.70× (3.63–3.78) | 4.07× (4.00–4.13) | 3.79× (3.73–3.84) | 3.99× (3.95–4.04) | 3.79× (3.73–3.84) | 1.27× (1.25–1.29) | 1.53× (1.50–1.56) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.81× (1.73–1.88) | 2.73× (2.63–2.82) | 2.99× (2.92–3.06) | 3.21× (3.12–3.31) | 3.04× (2.94–3.14) | 2.63× (2.57–2.70) | 2.50× (2.43–2.56) | 0.84× (0.82–0.85) | 1.01× (0.99–1.03) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.28–0.31) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.06× (11.03–11.10) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.36) | 0.52× (0.45–0.60) | 0.71× (0.69–0.72) | 0.60× (0.58–0.62) | 5.66× (5.51–5.81) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.39) | 0.73× (0.61–0.88) | 0.65× (0.61–0.69) | 0.56× (0.53–0.60) | 3.78× (3.57–4.01) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.27–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.87× (4.86–4.88) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.70) | 0.66× (0.65–0.66) | 8.00× (7.99–8.02) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.70–0.73) | 0.76× (0.69–0.83) | 1.71× (1.68–1.75) | 0.86× (0.83–0.88) | 6.21× (6.08–6.33) | 0.50× (0.49–0.51) | 0.53× (0.51–0.54) | 0.57× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.72) | 0.79× (0.75–0.84) | 1.26× (1.21–1.30) | 0.78× (0.75–0.81) | 3.68× (3.54–3.82) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.20× (5.17–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.38–0.43) | 0.90× (0.87–0.93) | 0.72× (0.70–0.75) | 3.13× (3.03–3.23) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.47× (0.46–0.48) | 0.90× (0.87–0.92) | 0.83× (0.81–0.86) | 2.61× (2.55–2.66) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.84) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.34× (0.32–0.37) | 0.83× (0.81–0.86) | 0.69× (0.66–0.71) | 2.86× (2.80–2.93) | 0.18× (0.17–0.18) | 0.19× (0.19–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.45–0.52) | 0.81× (0.79–0.82) | 0.83× (0.80–0.86) | 2.32× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.88) | 0.91× (0.90–0.91) | 6.22× (6.20–6.23) | 0.40× (0.40–0.41) | 0.42× (0.41–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.90–0.91) | 6.22× (6.20–6.25) | 0.40× (0.40–0.41) | 0.42× (0.41–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.40× (1.39–1.41) | 0.66× (0.65–0.66) | 6.42× (6.38–6.46) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.62–0.64) | 0.31× (0.30–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 36 MB | 0.46× (0.45–0.47) | 0.43× (0.38–0.48) | 1.05× (1.02–1.08) | 0.71× (0.69–0.73) | 3.45× (3.36–3.55) | 0.21× (0.20–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 54 MB | 0.51× (0.50–0.52) | 0.48× (0.43–0.53) | 0.94× (0.92–0.95) | 0.76× (0.74–0.79) | 2.40× (2.36–2.43) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 432 | 2.33 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 785 | 2.48 | 1.06× (1.06–1.07) | 89% | 1.02× (1.01–1.03) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 947 | 3.90 | 1.68× (1.66–1.69) | 54% | 1.22× (1.21–1.23) | 1.64× (1.62–1.66) |

**Best:** mibz2 at 4 threads (947 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (292 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.64 | 0.02 | 0.09 | 0.221 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.84 | 9.6% |
| MTF + RLE2 | 1.58 | 17.9% |
| T-vector build | 1.52 | 17.2% |
| walk (the one each build uses) | 4.35 | 49.2% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.84 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.512 | 1.299 | (in its total) | 2.54× (2.53–2.54) |  |
| walk | 1.266 | 2.682 | (in its total) | 2.11× (2.10–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.057 | (in its total) | 1.20× (1.18–1.21) |  |
| inverse BWT, total | 1.826 | 4.039 | 3.299 | 2.21× (2.19–2.22) | 1.81× (1.79–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m28s |
| 4 threads | 32m36s ± 4m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m17s ± 3m17s |
| the scanner alone (its serial part) | 10m20s ± 19.7s |
| the workers alone, over 4 | 23m17s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 6.0 ns a byte of bz2, the workers 3.05 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,218,404 | 14,849,042 | \`9257d589add281a9…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
