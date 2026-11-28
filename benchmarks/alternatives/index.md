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

The latest record: [`6a56e75`](<https://github.com/lmmx/mibz2/commit/6a56e7544ade15c53ac2b35eb046f2ea76ac8b10>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46093980808>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.386 s | 544 | 4.8 | 3.19× (3.18–3.21) | 7.28× (7.25–7.31) | 7.65× (7.60–7.70) | 8.69× (8.64–8.75) | 7.56× (7.51–7.61) | 7.90× (7.86–7.95) | 7.63× (7.60–7.67) | 2.37× (2.35–2.39) | 2.65× (2.64–2.67) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.220 s | 952 | 5.2 | 3.25× (3.09–3.42) | 6.71× (6.36–7.07) | 7.86× (7.62–8.10) | 8.07× (7.85–8.30) | 7.04× (6.82–7.27) | 13.72× (13.29–14.16) | 13.25× (12.81–13.70) | 4.11× (3.97–4.26) | 4.60× (4.48–4.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,160 | 8.3 | 3.36× (3.19–3.54) | 4.90× (4.64–5.17) | 6.15× (5.97–6.35) | 5.96× (5.65–6.29) | 5.73× (5.50–5.96) | 16.62× (16.00–17.28) | 16.06× (15.52–16.62) | 4.98× (4.79–5.19) | 5.58× (5.37–5.79) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,108 | 2.6 | 0.57× (0.56–0.58) | 3.38× (3.31–3.44) | 4.50× (4.42–4.59) | 3.52× (3.46–3.59) | 6.02× (5.91–6.14) | 2.62× (2.57–2.67) | 2.62× (2.57–2.67) | 0.96× (0.94–0.99) | 1.89× (1.70–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,070 | 2.8 | 0.52× (0.50–0.53) | 2.74× (2.68–2.81) | 5.49× (5.36–5.63) | 3.44× (3.35–3.52) | 5.77× (5.65–5.89) | 2.55× (2.49–2.61) | 2.55× (2.49–2.61) | 0.93× (0.91–0.96) | 1.84× (1.65–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,039 | 2.8 | 0.51× (0.50–0.51) | 2.68× (2.66–2.71) | 5.35× (5.30–5.41) | 3.37× (3.33–3.40) | 5.63× (5.56–5.70) | 2.48× (2.46–2.51) | 2.48× (2.46–2.51) | 0.91× (0.89–0.94) | 1.79× (1.62–1.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 763 | 3.6 | 2.83× (2.74–2.92) | 3.17× (3.15–3.19) | 3.96× (3.93–4.00) | 4.23× (4.20–4.26) | 4.63× (4.60–4.65) | 2.87× (2.84–2.90) | 2.34× (2.32–2.36) | 2.16× (2.15–2.17) | 3.16× (3.13–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,188 | 4.3 | 2.42× (2.33–2.51) | 3.25× (2.91–3.63) | 4.84× (4.48–5.24) | 3.87× (3.72–4.03) | 5.54× (5.31–5.78) | 4.46× (4.30–4.62) | 3.64× (3.51–3.77) | 3.36× (3.24–3.48) | 4.91× (4.73–5.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,062 | 8.6 | 2.07× (1.97–2.17) | 1.94× (1.72–2.17) | 3.41× (3.25–3.58) | 2.49× (2.34–2.65) | 4.44× (4.28–4.61) | 3.96× (3.79–4.13) | 3.23× (3.10–3.37) | 2.99× (2.87–3.11) | 4.36× (4.18–4.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.103 s | 78 | 33.9 | 1.49× (1.46–1.51) | 2.85× (2.80–2.90) | 2.56× (2.51–2.60) | 2.69× (2.64–2.73) | 2.97× (2.93–3.02) | 3.18× (3.13–3.23) | 3.35× (3.29–3.41) | 1.35× (1.33–1.38) | 1.55× (1.51–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 130 | 37.8 | 1.44× (1.40–1.49) | 2.78× (2.68–2.88) | 2.64× (2.56–2.72) | 3.44× (3.36–3.51) | 3.17× (3.08–3.26) | 5.33× (5.19–5.47) | 5.61× (5.47–5.76) | 2.27× (2.21–2.33) | 2.59× (2.53–2.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 162 | 49.5 | 1.51× (1.47–1.54) | 2.44× (2.38–2.50) | 2.48× (2.38–2.59) | 2.99× (2.94–3.05) | 3.33× (3.22–3.44) | 6.64× (6.51–6.76) | 6.99× (6.84–7.15) | 2.83× (2.78–2.88) | 3.23× (3.13–3.33) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 86 | 30.6 | 1.65× (1.64–1.67) | 3.29× (3.27–3.31) | 2.85× (2.80–2.90) | 3.00× (2.98–3.02) | 3.33× (3.30–3.36) | 3.96× (3.94–3.99) | 4.37× (4.35–4.39) | 1.69× (1.66–1.71) | 1.96× (1.93–1.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 138 | 34.6 | 1.47× (1.42–1.52) | 2.93× (2.81–3.06) | 2.66× (2.58–2.75) | 3.71× (3.61–3.82) | 3.24× (3.15–3.34) | 6.33× (6.14–6.51) | 6.98× (6.78–7.18) | 2.69× (2.60–2.78) | 3.12× (3.01–3.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 171 | 47.4 | 1.60× (1.55–1.66) | 2.57× (2.51–2.64) | 2.53× (2.45–2.62) | 3.52× (3.36–3.68) | 3.26× (3.17–3.35) | 7.87× (7.68–8.08) | 8.69× (8.46–8.91) | 3.35× (3.24–3.47) | 3.89× (3.76–4.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 3.5 | 0.62× (0.62–0.63) | 3.45× (3.42–3.49) | 4.34× (4.29–4.39) | 3.10× (3.07–3.13) | 6.69× (6.64–6.75) | 2.29× (2.26–2.31) | 2.33× (2.30–2.35) | 0.94× (0.93–0.96) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 4.0 | 0.50× (0.49–0.51) | 2.41× (2.34–2.48) | 4.81× (4.71–4.90) | 2.86× (2.81–2.92) | 6.20× (6.06–6.34) | 2.10× (2.06–2.14) | 2.14× (2.09–2.18) | 0.87× (0.85–0.89) | 1.60× (1.57–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 3.9 | 0.50× (0.50–0.51) | 2.40× (2.37–2.43) | 4.80× (4.74–4.86) | 2.87× (2.84–2.90) | 6.20× (6.10–6.30) | 2.10× (2.07–2.13) | 2.13× (2.10–2.16) | 0.87× (0.85–0.88) | 1.59× (1.57–1.62) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.052 s | 286 | 9.4 | 2.02× (2.00–2.05) | 4.10× (4.05–4.14) | 4.61× (4.54–4.69) | 4.89× (4.83–4.94) | 5.00× (4.95–5.04) | 4.79× (4.73–4.86) | 4.93× (4.89–4.97) | 1.67× (1.64–1.70) | 1.95× (1.93–1.97) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.036 s | 411 | 12.2 | 1.58× (1.52–1.64) | 3.37× (3.09–3.68) | 4.10× (3.90–4.30) | 4.60× (4.43–4.78) | 4.50× (4.25–4.75) | 6.84× (6.54–7.16) | 7.03× (6.73–7.34) | 2.38× (2.27–2.50) | 2.78× (2.66–2.90) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.032 s | 473 | 18.9 | 1.64× (1.59–1.68) | 2.71× (2.58–2.84) | 3.62× (3.42–3.82) | 3.74× (3.55–3.93) | 4.12× (3.94–4.31) | 7.82× (7.41–8.26) | 8.04× (7.65–8.45) | 2.72× (2.57–2.88) | 3.18× (3.03–3.33) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.18× (3.17–3.20) | 7.24× (7.21–7.27) | 7.61× (7.56–7.65) | 8.66× (8.60–8.71) | 7.53× (7.48–7.58) | 7.87× (7.82–7.91) | 7.60× (7.57–7.63) | 2.36× (2.34–2.38) | 2.63× (2.62–2.65) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (2.97–3.14) | 6.81× (6.61–7.01) | 7.20× (7.00–7.41) | 8.29× (8.08–8.51) | 6.93× (6.75–7.11) | 7.16× (7.00–7.33) | 6.92× (6.75–7.09) | 2.15× (2.09–2.21) | 2.40× (2.34–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.16× (3.07–3.24) | 5.17× (4.97–5.37) | 5.64× (5.52–5.77) | 6.24× (6.07–6.42) | 5.32× (5.20–5.45) | 4.52× (4.40–4.65) | 4.37× (4.26–4.48) | 1.36× (1.32–1.40) | 1.51× (1.47–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.88× (2.83–2.93) | 3.91× (3.82–4.02) | 3.29× (3.24–3.35) | 5.57× (5.48–5.67) | 2.48× (2.43–2.52) | 2.48× (2.43–2.52) | 0.96× (0.94–0.99) | 1.62× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.56) | 2.60× (2.55–2.65) | 4.67× (4.59–4.75) | 3.13× (3.07–3.20) | 5.17× (5.08–5.27) | 2.32× (2.28–2.37) | 2.32× (2.28–2.37) | 0.90× (0.88–0.93) | 1.52× (1.48–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.53–0.54) | 2.85× (2.82–2.88) | 4.67× (4.60–4.74) | 3.13× (3.10–3.17) | 5.13× (5.06–5.20) | 2.31× (2.29–2.34) | 2.31× (2.29–2.34) | 0.90× (0.88–0.92) | 1.51× (1.49–1.53) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.63–2.80) | 2.92× (2.90–2.95) | 3.50× (3.47–3.54) | 4.03× (3.99–4.07) | 4.40× (4.37–4.42) | 2.75× (2.72–2.78) | 2.26× (2.24–2.28) | 2.09× (2.07–2.10) | 2.73× (2.70–2.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.42× (2.39–2.46) | 2.66× (2.59–2.74) | 3.56× (3.50–3.61) | 3.53× (3.48–3.58) | 3.84× (3.76–3.92) | 2.29× (2.26–2.32) | 1.88× (1.85–1.91) | 1.74× (1.71–1.77) | 2.27× (2.23–2.32) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.22× (2.16–2.28) | 1.79× (1.73–1.86) | 2.39× (2.34–2.45) | 2.18× (2.14–2.22) | 2.26× (2.21–2.30) | 1.16× (1.14–1.19) | 0.96× (0.93–0.98) | 0.88× (0.87–0.90) | 1.16× (1.13–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.46–1.50) | 2.75× (2.70–2.80) | 2.51× (2.47–2.55) | 2.66× (2.62–2.70) | 2.94× (2.90–2.99) | 3.14× (3.09–3.19) | 3.31× (3.25–3.37) | 1.35× (1.32–1.37) | 1.47× (1.44–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.36–1.42) | 2.62× (2.56–2.67) | 2.32× (2.27–2.37) | 3.15× (3.09–3.21) | 2.70× (2.64–2.76) | 2.81× (2.76–2.87) | 2.96× (2.90–3.03) | 1.21× (1.18–1.24) | 1.31× (1.29–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.59–1.63) | 2.54× (2.48–2.59) | 2.16× (2.09–2.24) | 2.91× (2.86–2.96) | 2.79× (2.71–2.88) | 2.18× (2.15–2.21) | 2.30× (2.26–2.33) | 0.93× (0.92–0.95) | 1.02× (1.00–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.64× (1.63–1.66) | 3.21× (3.18–3.23) | 2.80× (2.76–2.85) | 2.97× (2.95–2.98) | 3.29× (3.26–3.32) | 3.92× (3.89–3.94) | 4.32× (4.30–4.34) | 1.68× (1.65–1.70) | 1.89× (1.87–1.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.51× (1.46–1.57) | 2.96× (2.88–3.04) | 2.52× (2.46–2.59) | 3.57× (3.47–3.67) | 2.95× (2.88–3.02) | 3.43× (3.34–3.52) | 3.78× (3.68–3.88) | 1.47× (1.42–1.52) | 1.66× (1.60–1.71) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.76× (1.68–1.84) | 2.79× (2.72–2.86) | 2.22× (2.17–2.28) | 3.19× (3.10–3.27) | 2.91× (2.83–3.00) | 2.53× (2.48–2.59) | 2.79× (2.73–2.86) | 1.08× (1.05–1.12) | 1.22× (1.19–1.26) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.49× (2.47–2.51) | 3.41× (3.38–3.44) | 2.83× (2.81–2.85) | 5.97× (5.92–6.02) | 2.12× (2.11–2.14) | 2.16× (2.14–2.18) | 0.95× (0.94–0.96) | 1.37× (1.36–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.12× (2.09–2.16) | 3.79× (3.72–3.86) | 2.57× (2.53–2.61) | 5.33× (5.23–5.42) | 1.87× (1.84–1.90) | 1.90× (1.87–1.93) | 0.84× (0.82–0.86) | 1.21× (1.19–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.37× (2.34–2.40) | 3.85× (3.77–3.94) | 2.65× (2.61–2.68) | 5.47× (5.39–5.55) | 1.92× (1.89–1.95) | 1.95× (1.93–1.98) | 0.86× (0.85–0.87) | 1.24× (1.22–1.26) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 1.99× (1.97–2.01) | 3.94× (3.90–3.97) | 4.39× (4.35–4.44) | 4.77× (4.70–4.83) | 4.87× (4.82–4.91) | 4.67× (4.61–4.74) | 4.80× (4.76–4.84) | 1.65× (1.62–1.68) | 1.83× (1.81–1.84) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.66× (1.63–1.68) | 3.24× (3.19–3.29) | 3.65× (3.60–3.71) | 4.24× (4.18–4.31) | 3.93× (3.81–4.06) | 3.65× (3.58–3.72) | 3.75× (3.69–3.80) | 1.29× (1.26–1.32) | 1.43× (1.40–1.45) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.72× (1.64–1.80) | 2.64× (2.58–2.71) | 2.99× (2.90–3.08) | 3.23× (3.12–3.33) | 3.06× (2.98–3.14) | 2.33× (2.25–2.41) | 2.39× (2.32–2.46) | 0.82× (0.79–0.85) | 0.91× (0.89–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.32× (0.31–0.34) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.09× (11.06–11.11) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.36) | 0.44× (0.37–0.53) | 0.71× (0.68–0.74) | 0.59× (0.57–0.62) | 5.67× (5.48–5.88) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.35–0.38) | 0.61× (0.50–0.74) | 0.65× (0.62–0.67) | 0.56× (0.53–0.59) | 3.77× (3.61–3.93) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.66–0.66) | 8.02× (7.99–8.05) | 0.66× (0.65–0.66) | 0.68× (0.67–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.73) | 0.69× (0.65–0.74) | 1.71× (1.68–1.74) | 0.85× (0.83–0.87) | 6.19× (6.08–6.31) | 0.51× (0.49–0.52) | 0.52× (0.52–0.53) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.67× (0.64–0.71) | 0.72× (0.64–0.80) | 1.22× (1.17–1.27) | 0.74× (0.70–0.79) | 3.56× (3.39–3.74) | 0.28× (0.27–0.29) | 0.29× (0.28–0.30) | 0.32× (0.30–0.34) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.18× (5.17–5.20) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.89× (0.87–0.91) | 0.71× (0.69–0.73) | 3.08× (3.02–3.15) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.66× (0.64–0.67) | 0.47× (0.46–0.47) | 0.88× (0.87–0.90) | 0.83× (0.82–0.85) | 2.57× (2.53–2.62) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.21–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.42) | 0.86× (0.83–0.90) | 0.72× (0.69–0.76) | 2.98× (2.87–3.09) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.48× (0.46–0.50) | 0.80× (0.79–0.81) | 0.80× (0.78–0.82) | 2.31× (2.30–2.33) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.22) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.22) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.39) | 0.66× (0.66–0.66) | 6.41× (6.40–6.41) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.64) | 0.30× (0.30–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 35 MB | 0.47× (0.46–0.49) | 0.42× (0.39–0.45) | 1.08× (1.04–1.11) | 0.72× (0.70–0.74) | 3.54× (3.44–3.63) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.17× (0.16–0.17) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 54 MB | 0.51× (0.50–0.52) | 0.46× (0.43–0.49) | 0.93× (0.91–0.94) | 0.74× (0.71–0.76) | 2.38× (2.35–2.42) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 544 | 1.85 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 952 | 2.01 | 1.10× (1.08–1.12) | 87% | 1.03× (1.02–1.05) | 1.05× (1.04–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,160 | 3.19 | 1.74× (1.70–1.78) | 53% | 1.24× (1.20–1.28) | 1.73× (1.69–1.76) |

**Best:** mibz2 at 4 threads (1,160 MB/s); lbzip2 at 4 threads (237 MB/s); pbz2 at 4 threads (341 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.46, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.53 | 0.02 | 0.07 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.59 | 9.6% |
| MTF + RLE2 | 1.11 | 18.2% |
| T-vector build | 1.06 | 17.3% |
| walk (the one each build uses) | 3.02 | 49.4% |
| of which: segment assembly | 0.12 | 2.0% |
| RLE1 | 0.14 | 2.2% |
| CRC | 0.18 | 3.0% |
| **total** | 6.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.060 | (in its total) | 2.70× (2.68–2.72) |  |
| walk | 1.087 | 2.066 | (in its total) | 1.89× (1.88–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.041 | 0.044 | (in its total) | 1.07× (1.05–1.09) |  |
| inverse BWT, total | 1.519 | 3.172 | 1.897 | 2.08× (2.06–2.10) | 1.25× (1.24–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m12s ± 6m39s |
| 4 threads | 26m45s ± 3m54s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m12s ± 2m45s |
| the scanner alone (its serial part) | 8m44s ± 41.6s |
| the workers alone, over 4 | 19m12s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.52 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,228,219 | 14,890,034 | \`9ad9de226af45720…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
