# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-361/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`4035b83`](<https://github.com/lmmx/mibz2/commit/4035b83b016cac048025001ca32c77fea50a51cd>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42856209976>)

Intel(R) Xeon(R) 6973P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.580 s | 361 | 10.5 | 2.40× (2.37–2.44) | 5.57× (5.47–5.68) | 5.88× (5.75–6.01) | 6.46× (6.34–6.59) | 5.98× (5.87–6.09) | 6.17× (6.04–6.31) | 6.00× (5.88–6.13) | 1.83× (1.79–1.88) | 1.71× (1.67–1.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.326 s | 643 | 11.1 | 2.41× (2.28–2.55) | 5.20× (4.95–5.46) | 6.27× (5.95–6.60) | 6.28× (5.86–6.72) | 5.66× (5.34–6.01) | 10.96× (10.39–11.57) | 10.65× (10.10–11.24) | 3.25× (3.07–3.44) | 3.04× (2.88–3.21) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.273 s | 768 | 18.9 | 2.41× (2.38–2.44) | 4.43× (4.34–4.52) | 5.26× (5.14–5.39) | 5.14× (5.03–5.26) | 5.10× (4.99–5.20) | 13.17× (12.94–13.40) | 12.80× (12.58–13.02) | 3.90× (3.82–3.99) | 3.65× (3.58–3.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,084 | 3.8 | 0.61× (0.60–0.63) | 3.25× (3.11–3.38) | 4.31× (4.10–4.52) | 3.70× (3.50–3.92) | 6.27× (6.09–6.46) | 2.60× (2.51–2.70) | 2.74× (2.55–2.95) | 1.20× (1.16–1.24) | 1.12× (1.10–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,067 | 3.9 | 0.52× (0.50–0.53) | 2.80× (2.68–2.93) | 5.99× (5.76–6.23) | 3.56× (3.39–3.73) | 6.10× (5.93–6.27) | 2.59× (2.47–2.71) | 2.73× (2.56–2.90) | 1.19× (1.14–1.24) | 1.12× (1.08–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,044 | 3.9 | 0.53× (0.48–0.60) | 2.74× (2.65–2.83) | 6.07× (5.67–6.49) | 3.46× (3.31–3.61) | 6.03× (5.82–6.24) | 2.52× (2.43–2.60) | 2.65× (2.45–2.87) | 1.16× (1.12–1.19) | 1.09× (1.06–1.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 571 | 7.0 | 2.46× (2.36–2.56) | 3.11× (2.97–3.25) | 3.71× (3.59–3.83) | 4.08× (3.93–4.23) | 4.57× (4.37–4.77) | 2.97× (2.87–3.06) | 2.78× (2.65–2.91) | 1.91× (1.86–1.97) | 2.06× (2.01–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 856 | 8.6 | 1.99× (1.74–2.27) | 2.62× (2.32–2.96) | 3.89× (3.29–4.61) | 3.27× (2.95–3.63) | 4.66× (4.17–5.21) | 4.23× (3.79–4.72) | 3.96× (3.59–4.37) | 2.73× (2.45–3.05) | 2.93× (2.63–3.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 869 | 15.5 | 1.77× (1.67–1.87) | 1.96× (1.82–2.12) | 2.87× (2.75–3.00) | 2.36× (2.28–2.45) | 4.14× (3.94–4.35) | 4.53× (4.31–4.76) | 4.24× (3.97–4.53) | 2.92× (2.81–3.04) | 3.14× (3.03–3.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 59.1 | 1.35× (1.31–1.39) | 2.94× (2.84–3.04) | 2.69× (2.59–2.79) | 2.71× (2.63–2.78) | 3.00× (2.93–3.07) | 3.30× (3.21–3.40) | 3.42× (3.34–3.50) | 1.23× (1.20–1.26) | 1.24× (1.21–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 107 | 65.6 | 1.27× (1.23–1.31) | 2.74× (2.64–2.84) | 2.64× (2.55–2.73) | 3.39× (3.22–3.58) | 3.09× (2.98–3.20) | 5.46× (5.27–5.66) | 5.66× (5.44–5.88) | 2.03× (1.96–2.11) | 2.05× (1.96–2.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.061 s | 131 | 88.2 | 1.41× (1.34–1.49) | 2.63× (2.51–2.74) | 2.60× (2.50–2.69) | 3.15× (3.06–3.24) | 3.16× (3.03–3.29) | 6.71× (6.46–6.97) | 6.95× (6.75–7.15) | 2.49× (2.40–2.58) | 2.51× (2.44–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.136 s | 70 | 54.6 | 1.41× (1.37–1.45) | 3.27× (3.21–3.34) | 2.84× (2.79–2.90) | 2.87× (2.83–2.91) | 3.25× (3.20–3.29) | 3.95× (3.86–4.03) | 4.27× (4.22–4.32) | 1.43× (1.41–1.45) | 1.49× (1.47–1.50) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.081 s | 118 | 60.7 | 1.38× (1.26–1.51) | 3.06× (2.96–3.16) | 2.86× (2.72–3.00) | 3.70× (3.57–3.83) | 3.28× (3.21–3.34) | 6.62× (6.48–6.76) | 7.16× (7.02–7.31) | 2.40× (2.34–2.46) | 2.50× (2.44–2.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 136 | 87.0 | 1.36× (1.29–1.43) | 2.78× (2.62–2.94) | 2.57× (2.43–2.71) | 3.37× (3.20–3.55) | 3.01× (2.85–3.17) | 7.57× (7.11–8.06) | 8.19× (7.77–8.64) | 2.74× (2.58–2.92) | 2.85× (2.71–3.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 816 | 5.3 | 0.67× (0.62–0.71) | 2.86× (2.52–3.25) | 4.23× (4.00–4.48) | 2.98× (2.83–3.14) | 6.82× (6.48–7.17) | 2.33× (2.18–2.49) | 2.35× (2.24–2.48) | 1.06× (1.02–1.11) | 0.86× (0.82–0.89) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 766 | 5.7 | 0.51× (0.49–0.52) | 2.37× (2.32–2.41) | 5.39× (5.10–5.68) | 3.10× (2.83–3.39) | 6.47× (6.18–6.77) | 2.21× (2.12–2.31) | 2.23× (2.18–2.28) | 1.01× (0.99–1.03) | 0.81× (0.80–0.83) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 5.5 | 0.51× (0.50–0.53) | 2.64× (2.42–2.88) | 5.20× (4.91–5.52) | 3.00× (2.92–3.09) | 6.57× (6.34–6.81) | 2.26× (2.16–2.36) | 2.28× (2.21–2.35) | 1.03× (1.01–1.06) | 0.83× (0.81–0.84) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.066 s | 196 | 19.9 | 1.79× (1.73–1.84) | 4.02× (3.93–4.11) | 4.48× (4.37–4.60) | 4.52× (4.43–4.61) | 4.75× (4.62–4.89) | 4.80× (4.67–4.93) | 4.77× (4.62–4.91) | 1.44× (1.39–1.49) | 1.40× (1.36–1.45) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.042 s | 303 | 23.3 | 1.59× (1.52–1.66) | 3.50× (3.34–3.66) | 4.33× (4.16–4.50) | 4.61× (4.41–4.82) | 4.53× (4.46–4.60) | 7.45× (7.31–7.60) | 7.40× (7.23–7.59) | 2.23× (2.16–2.31) | 2.18× (2.11–2.25) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.039 s | 328 | 36.0 | 1.43× (1.37–1.49) | 2.67× (2.61–2.74) | 3.58× (3.41–3.75) | 3.69× (3.58–3.79) | 4.04× (3.91–4.18) | 8.05× (7.83–8.28) | 8.00× (7.75–8.25) | 2.41× (2.30–2.52) | 2.35× (2.26–2.45) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.40× (2.37–2.43) | 5.55× (5.45–5.66) | 5.85× (5.73–5.98) | 6.45× (6.32–6.57) | 5.96× (5.85–6.08) | 6.16× (6.03–6.29) | 5.98× (5.86–6.11) | 1.83× (1.79–1.87) | 1.70× (1.66–1.74) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.35× (2.30–2.40) | 5.43× (5.26–5.60) | 5.96× (5.83–6.10) | 6.57× (6.39–6.75) | 5.75× (5.58–5.93) | 5.87× (5.71–6.02) | 5.70× (5.57–5.84) | 1.74× (1.70–1.78) | 1.62× (1.59–1.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.23× (2.17–2.30) | 4.50× (4.40–4.60) | 4.85× (4.73–4.97) | 5.29× (5.15–5.43) | 4.74× (4.62–4.87) | 3.45× (3.38–3.53) | 3.35× (3.28–3.43) | 1.02× (1.00–1.05) | 0.96× (0.93–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.63–0.66) | 2.69× (2.62–2.76) | 3.75× (3.64–3.86) | 3.47× (3.28–3.67) | 5.83× (5.67–6.00) | 2.47× (2.38–2.56) | 2.60× (2.43–2.79) | 1.18× (1.14–1.22) | 0.82× (0.80–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.71× (2.61–2.81) | 5.11× (4.97–5.26) | 3.33× (3.19–3.49) | 5.63× (5.48–5.78) | 2.43× (2.32–2.54) | 2.56× (2.39–2.73) | 1.16× (1.11–1.21) | 0.81× (0.78–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.51–0.64) | 3.02× (2.91–3.13) | 5.23× (4.92–5.55) | 3.29× (3.16–3.42) | 5.63× (5.44–5.82) | 2.40× (2.32–2.48) | 2.52× (2.35–2.72) | 1.15× (1.11–1.19) | 0.80× (0.78–0.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.29–2.48) | 2.89× (2.83–2.95) | 3.39× (3.31–3.47) | 3.92× (3.79–4.07) | 4.39× (4.20–4.59) | 2.87× (2.78–2.96) | 2.68× (2.56–2.81) | 1.87× (1.81–1.92) | 1.84× (1.77–1.91) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.00–2.30) | 2.49× (2.35–2.64) | 3.04× (2.87–3.23) | 3.22× (3.05–3.39) | 3.68× (3.47–3.91) | 2.32× (2.22–2.42) | 2.17× (2.04–2.31) | 1.51× (1.42–1.60) | 1.49× (1.41–1.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.83× (1.78–1.89) | 1.73× (1.67–1.80) | 2.14× (2.08–2.21) | 2.11× (2.04–2.18) | 2.22× (2.15–2.30) | 1.29× (1.25–1.34) | 1.21× (1.15–1.27) | 0.84× (0.81–0.88) | 0.83× (0.80–0.85) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.30–1.39) | 2.86× (2.77–2.96) | 2.63× (2.55–2.71) | 2.69× (2.61–2.76) | 2.97× (2.90–3.05) | 3.27× (3.18–3.37) | 3.39× (3.31–3.47) | 1.23× (1.20–1.25) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.30) | 2.66× (2.58–2.75) | 2.43× (2.37–2.50) | 3.13× (3.03–3.25) | 2.76× (2.66–2.87) | 2.95× (2.86–3.04) | 3.05× (2.95–3.15) | 1.10× (1.07–1.14) | 1.08× (1.05–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.49–1.62) | 2.74× (2.67–2.81) | 2.38× (2.32–2.45) | 3.06× (3.01–3.12) | 2.76× (2.67–2.86) | 2.20× (2.15–2.26) | 2.28× (2.24–2.33) | 0.83× (0.80–0.85) | 0.81× (0.79–0.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.36–1.45) | 3.20× (3.14–3.26) | 2.78× (2.73–2.83) | 2.85× (2.81–2.88) | 3.22× (3.17–3.26) | 3.91× (3.83–4.00) | 4.23× (4.18–4.28) | 1.43× (1.41–1.45) | 1.44× (1.42–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.26–1.41) | 3.03× (2.95–3.11) | 2.64× (2.57–2.71) | 3.48× (3.40–3.56) | 2.99× (2.95–3.03) | 3.51× (3.41–3.61) | 3.79× (3.73–3.85) | 1.28× (1.25–1.31) | 1.29× (1.27–1.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.50× (1.43–1.59) | 2.95× (2.79–3.12) | 2.33× (2.21–2.47) | 3.21× (3.01–3.42) | 2.70× (2.56–2.86) | 2.41× (2.25–2.59) | 2.61× (2.46–2.77) | 0.88× (0.82–0.94) | 0.89× (0.83–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.66–0.76) | 2.29× (2.20–2.39) | 3.35× (3.14–3.59) | 2.74× (2.60–2.87) | 6.11× (5.83–6.41) | 2.17× (2.04–2.31) | 2.18× (2.08–2.29) | 1.05× (1.01–1.09) | 0.76× (0.73–0.79) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.55–0.58) | 2.12× (2.07–2.16) | 4.24× (4.10–4.39) | 2.86× (2.63–3.10) | 5.75× (5.49–6.03) | 2.03× (1.94–2.12) | 2.05× (2.00–2.09) | 0.98× (0.96–1.01) | 0.71× (0.68–0.74) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.59) | 2.59× (2.48–2.71) | 4.23× (4.12–4.35) | 2.83× (2.76–2.91) | 5.94× (5.74–6.15) | 2.11× (2.02–2.21) | 2.13× (2.06–2.19) | 1.02× (1.00–1.05) | 0.74× (0.71–0.77) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.77× (1.71–1.82) | 3.85× (3.77–3.94) | 4.32× (4.21–4.44) | 4.43× (4.35–4.52) | 4.66× (4.53–4.79) | 4.70× (4.58–4.83) | 4.67× (4.53–4.82) | 1.42× (1.37–1.48) | 1.34× (1.30–1.37) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.61× (1.57–1.64) | 3.48× (3.41–3.55) | 4.10× (4.00–4.21) | 4.29× (4.27–4.32) | 4.06× (4.01–4.11) | 4.04× (3.99–4.08) | 4.01× (3.95–4.07) | 1.22× (1.19–1.26) | 1.15× (1.13–1.17) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.71× (1.65–1.77) | 3.10× (3.03–3.17) | 3.42× (3.34–3.50) | 3.74× (3.59–3.88) | 3.53× (3.41–3.65) | 2.64× (2.55–2.73) | 2.62× (2.53–2.71) | 0.80× (0.77–0.83) | 0.75× (0.73–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.29× (0.28–0.30) | 1.05× (1.05–1.05) | 0.84× (0.84–0.85) | 11.91× (11.90–11.92) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 51 MB | 0.37× (0.37–0.38) | 0.48× (0.40–0.59) | 0.76× (0.75–0.78) | 0.65× (0.63–0.66) | 6.08× (5.99–6.18) | 0.14× (0.14–0.14) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.38–0.40) | 0.52× (0.45–0.60) | 0.69× (0.68–0.70) | 0.61× (0.60–0.63) | 4.05× (3.99–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.28× (4.27–4.28) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.01× (7.98–8.04) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.63× (0.57–0.68) | 1.61× (1.56–1.66) | 0.81× (0.79–0.84) | 5.89× (5.72–6.07) | 0.47× (0.46–0.48) | 0.50× (0.48–0.52) | 0.54× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.70× (0.63–0.77) | 1.14× (1.10–1.19) | 0.70× (0.67–0.73) | 3.40× (3.29–3.52) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.20× (5.19–5.20) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.50–0.53) | 0.40× (0.37–0.43) | 0.94× (0.91–0.97) | 0.73× (0.70–0.76) | 3.26× (3.16–3.36) | 0.20× (0.19–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.62–0.65) | 0.46× (0.44–0.48) | 0.87× (0.85–0.88) | 0.82× (0.80–0.83) | 2.52× (2.48–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.50× (0.48–0.53) | 0.38× (0.34–0.42) | 0.87× (0.84–0.90) | 0.71× (0.67–0.75) | 2.99× (2.89–3.11) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.58–0.61) | 0.47× (0.45–0.50) | 0.80× (0.78–0.81) | 0.83× (0.81–0.85) | 2.30× (2.26–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.29–5.34) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.33× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.67) | 6.42× (6.41–6.43) | 0.38× (0.38–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.41× (0.38–0.44) | 1.12× (1.10–1.14) | 0.72× (0.68–0.76) | 3.57× (3.52–3.62) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 55 MB | 0.50× (0.48–0.51) | 0.45× (0.42–0.49) | 0.92× (0.89–0.95) | 0.76× (0.72–0.79) | 2.38× (2.31–2.44) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 361 | 2.77 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 643 | 2.92 | 1.05× (1.02–1.08) | 89% | 1.03× (1.01–1.05) | 1.03× (1.01–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 768 | 4.99 | 1.78× (1.72–1.85) | 53% | 1.45× (1.43–1.46) | 1.66× (1.62–1.70) |

**Best:** mibz2 at 4 threads (768 MB/s); lbzip2 at 4 threads (174 MB/s); pbz2 at 4 threads (320 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.06, lbzip2 1t 0.04, lbzip2 2t 0.21, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.36, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.83 | 0.03 | 0.09 | 0.273 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.24 | 10.9% |
| MTF + RLE2 | 1.79 | 15.7% |
| T-vector build | 1.60 | 14.1% |
| walk (the one each build uses) | 6.31 | 55.6% |
| of which: segment assembly | 0.32 | 2.8% |
| RLE1 | 0.27 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 11.36 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.382 | 0.963 | (in its total) | 2.48× (2.39–2.57) |  |
| walk | 1.545 | 2.913 | (in its total) | 1.86× (1.81–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.084 | 0.077 | (in its total) | 0.90× (0.88–0.93) |  |
| inverse BWT, total | 2.011 | 3.953 | 2.066 | 1.94× (1.88–2.00) | 1.02× (0.99–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h25m ± 10m04s |
| 4 threads | 39m51s ± 5m38s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m22s ± 4m23s |
| the scanner alone (its serial part) | 14m24s ± 35.4s |
| the workers alone, over 4 | 30m22s ± 4m23s |

At 4 threads the workers bound it. Per byte: the scanner 8.4 ns a byte of bz2, the workers 3.98 ns a byte of output at 4 threads (CPU, summed over them), 2.31 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,559,615 | 12,840,880 | \`c564b1e6883ea19b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6973P-C, 4 cores, reported clock 3799.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
