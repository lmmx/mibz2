# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-874/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c7e067d`](<https://github.com/lmmx/mibz2/commit/c7e067d35751e2e206f2c75436b28dade0475b32>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45544802022>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.302 s | 695 | 6.2 | 2.89× (2.87–2.91) | 7.25× (7.15–7.36) | 7.66× (7.53–7.80) | 8.65× (8.53–8.78) | 7.77× (7.67–7.87) | 7.95× (7.85–8.05) | 7.63× (7.52–7.74) | 2.26× (2.24–2.28) | 2.64× (2.59–2.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.180 s | 1,166 | 6.9 | 2.93× (2.76–3.11) | 6.65× (6.27–7.05) | 7.91× (7.34–8.51) | 8.07× (7.66–8.51) | 7.15× (6.76–7.57) | 13.35× (12.68–14.05) | 12.82× (12.17–13.50) | 3.79× (3.60–4.00) | 4.43× (4.19–4.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.148 s | 1,413 | 10.5 | 3.12× (2.99–3.27) | 4.86× (4.46–5.29) | 6.42× (5.92–6.95) | 6.10× (5.73–6.50) | 6.04× (5.67–6.42) | 16.14× (15.70–16.60) | 15.50× (15.15–15.87) | 4.59× (4.47–4.70) | 5.35× (5.24–5.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,342 | 3.5 | 0.56× (0.55–0.57) | 3.51× (3.35–3.68) | 4.43× (4.25–4.63) | 3.68× (3.57–3.79) | 6.45× (6.27–6.64) | 2.74× (2.68–2.80) | 2.75× (2.68–2.81) | 1.00× (0.97–1.02) | 2.07× (2.03–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,294 | 3.8 | 0.50× (0.47–0.52) | 3.02× (2.81–3.25) | 5.61× (5.40–5.82) | 3.66× (3.33–4.03) | 6.16× (5.77–6.58) | 2.62× (2.56–2.68) | 2.63× (2.56–2.69) | 0.95× (0.93–0.98) | 1.98× (1.91–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,270 | 3.8 | 0.46× (0.40–0.53) | 2.69× (2.26–3.20) | 5.33× (4.59–6.20) | 3.28× (2.84–3.78) | 5.73× (4.99–6.57) | 2.43× (2.11–2.80) | 2.43× (2.11–2.81) | 0.88× (0.76–1.02) | 1.83× (1.59–2.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,034 | 4.5 | 2.81× (2.75–2.88) | 3.60× (3.46–3.74) | 4.20× (4.04–4.37) | 4.60× (4.40–4.81) | 5.39× (5.20–5.58) | 3.05× (2.91–3.18) | 2.67× (2.57–2.77) | 2.13× (2.08–2.18) | 3.17× (3.04–3.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,294 | 5.9 | 2.11× (1.86–2.40) | 2.92× (2.52–3.37) | 4.10× (3.57–4.72) | 3.53× (3.13–3.98) | 5.44× (4.88–6.07) | 3.89× (3.45–4.40) | 3.41× (3.04–3.82) | 2.72× (2.41–3.07) | 4.05× (3.59–4.57) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,174 | 11.5 | 1.79× (1.70–1.88) | 1.96× (1.80–2.12) | 3.05× (2.87–3.23) | 2.14× (2.00–2.29) | 4.49× (4.15–4.86) | 3.52× (3.26–3.79) | 3.08× (2.86–3.31) | 2.46× (2.28–2.65) | 3.66× (3.40–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 97 | 44.8 | 1.41× (1.39–1.43) | 3.21× (3.07–3.35) | 2.88× (2.81–2.96) | 2.80× (2.72–2.87) | 3.14× (3.07–3.21) | 3.57× (3.45–3.69) | 3.45× (3.37–3.53) | 1.36× (1.34–1.39) | 1.55× (1.51–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 157 | 51.1 | 1.37× (1.17–1.59) | 2.98× (2.74–3.25) | 2.66× (2.49–2.84) | 3.29× (3.09–3.49) | 3.18× (2.98–3.39) | 5.56× (5.16–5.98) | 5.37× (5.08–5.67) | 2.12× (1.98–2.27) | 2.41× (2.24–2.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 188 | 69.0 | 1.31× (1.19–1.45) | 2.51× (2.26–2.79) | 2.48× (2.24–2.74) | 2.88× (2.63–3.16) | 3.28× (3.00–3.59) | 6.85× (6.22–7.55) | 6.62× (6.04–7.25) | 2.61× (2.36–2.90) | 2.97× (2.66–3.32) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.084 s | 114 | 38.2 | 1.51× (1.46–1.56) | 3.78× (3.59–3.98) | 3.05× (2.92–3.18) | 3.06× (2.92–3.21) | 3.53× (3.34–3.72) | 4.45× (4.24–4.66) | 4.71× (4.48–4.95) | 1.60× (1.54–1.66) | 1.86× (1.76–1.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.059 s | 163 | 45.3 | 1.24× (1.09–1.41) | 3.03× (2.67–3.43) | 2.68× (2.30–3.12) | 3.59× (3.10–4.16) | 3.21× (2.77–3.72) | 6.35× (5.61–7.20) | 6.74× (5.95–7.63) | 2.29× (2.02–2.58) | 2.65× (2.35–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 204 | 62.4 | 1.38× (1.31–1.46) | 2.91× (2.52–3.37) | 2.78× (2.58–2.99) | 3.49× (3.27–3.73) | 3.56× (3.11–4.09) | 8.21× (7.65–8.82) | 8.71× (7.99–9.49) | 2.96× (2.73–3.20) | 3.43× (3.17–3.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,005 | 5.0 | 0.61× (0.60–0.63) | 3.12× (3.03–3.22) | 4.48× (4.12–4.88) | 3.12× (3.03–3.22) | 7.23× (6.96–7.50) | 2.39× (2.28–2.50) | 2.49× (2.17–2.86) | 0.98× (0.96–1.00) | 2.09× (2.03–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 943 | 5.5 | 0.45× (0.38–0.54) | 2.75× (2.44–3.08) | 4.80× (4.03–5.72) | 2.70× (2.25–3.22) | 5.93× (4.96–7.09) | 2.03× (1.76–2.33) | 2.11× (1.68–2.66) | 0.83× (0.69–0.99) | 1.78× (1.47–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 918 | 5.5 | 0.44× (0.35–0.55) | 2.58× (2.08–3.18) | 4.48× (3.60–5.57) | 2.63× (2.12–3.25) | 5.99× (5.16–6.97) | 1.96× (1.56–2.44) | 2.04× (1.57–2.65) | 0.80× (0.65–0.98) | 1.72× (1.40–2.11) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.045 s | 311 | 14.2 | 1.84× (1.80–1.88) | 4.50× (4.34–4.66) | 4.87× (4.69–5.05) | 5.11× (4.82–5.41) | 5.33× (5.11–5.57) | 5.55× (5.22–5.89) | 5.02× (4.84–5.21) | 1.61× (1.56–1.66) | 1.91× (1.79–2.03) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.032 s | 438 | 18.8 | 1.50× (1.36–1.66) | 3.83× (3.38–4.34) | 4.44× (4.14–4.76) | 5.15× (4.77–5.57) | 4.78× (4.39–5.21) | 7.92× (7.49–8.38) | 7.18× (6.82–7.55) | 2.30× (2.21–2.39) | 2.73× (2.48–2.99) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.030 s | 470 | 28.7 | 1.41× (1.30–1.53) | 2.61× (2.39–2.84) | 3.63× (3.39–3.88) | 3.72× (3.32–4.17) | 4.25× (3.82–4.74) | 8.29× (7.58–9.07) | 7.51× (6.90–8.18) | 2.41× (2.21–2.63) | 2.85× (2.58–3.16) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.85–2.90) | 7.20× (7.10–7.31) | 7.61× (7.47–7.74) | 8.61× (8.48–8.74) | 7.73× (7.63–7.83) | 7.90× (7.81–8.00) | 7.59× (7.48–7.70) | 2.25× (2.23–2.27) | 2.61× (2.57–2.65) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.70–2.80) | 6.96× (6.73–7.19) | 7.37× (7.05–7.71) | 8.46× (8.21–8.72) | 7.21× (6.99–7.43) | 7.17× (7.01–7.34) | 6.89× (6.75–7.02) | 2.04× (2.00–2.08) | 2.37× (2.31–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.85–3.20) | 5.37× (5.01–5.76) | 6.09× (5.63–6.59) | 6.77× (6.39–7.16) | 5.88× (5.50–6.29) | 4.61× (4.48–4.74) | 4.43× (4.31–4.55) | 1.31× (1.27–1.36) | 1.52× (1.47–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.03× (2.97–3.09) | 3.84× (3.78–3.91) | 3.42× (3.34–3.50) | 5.92× (5.78–6.07) | 2.58× (2.53–2.62) | 2.58× (2.52–2.63) | 1.00× (0.97–1.03) | 1.83× (1.79–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.52–0.56) | 2.68× (2.60–2.77) | 4.72× (4.57–4.86) | 3.30× (3.00–3.63) | 5.46× (5.10–5.84) | 2.37× (2.31–2.43) | 2.37× (2.31–2.43) | 0.92× (0.90–0.94) | 1.68× (1.64–1.72) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.49× (0.43–0.56) | 2.68× (2.32–3.08) | 4.57× (3.91–5.34) | 3.01× (2.62–3.47) | 5.14× (4.49–5.90) | 2.24× (1.95–2.58) | 2.24× (1.94–2.58) | 0.87× (0.75–1.01) | 1.59× (1.37–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.61–2.73) | 3.10× (3.00–3.19) | 3.71× (3.61–3.81) | 4.32× (4.14–4.51) | 5.05× (4.88–5.22) | 2.89× (2.77–3.01) | 2.54× (2.44–2.64) | 2.04× (1.99–2.09) | 2.78× (2.72–2.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.21× (2.10–2.33) | 2.61× (2.46–2.78) | 3.16× (2.99–3.34) | 3.46× (3.24–3.71) | 4.04× (3.83–4.26) | 2.18× (2.05–2.33) | 1.92× (1.79–2.06) | 1.54× (1.46–1.64) | 2.10× (1.99–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.02× (1.91–2.14) | 1.76× (1.67–1.87) | 2.16× (2.04–2.28) | 2.20× (2.05–2.35) | 2.40× (2.19–2.63) | 1.13× (1.05–1.22) | 1.00× (0.92–1.08) | 0.80× (0.75–0.85) | 1.09× (1.03–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.38–1.42) | 3.12× (3.00–3.24) | 2.77× (2.70–2.83) | 2.76× (2.68–2.84) | 3.10× (3.03–3.17) | 3.52× (3.40–3.64) | 3.40× (3.32–3.48) | 1.35× (1.33–1.38) | 1.48× (1.45–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.24–1.36) | 2.90× (2.76–3.05) | 2.44× (2.37–2.51) | 3.15× (3.05–3.25) | 2.81× (2.69–2.94) | 3.06× (2.93–3.19) | 2.95× (2.87–3.04) | 1.18× (1.13–1.22) | 1.28× (1.24–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.35× (1.22–1.49) | 2.54× (2.28–2.83) | 2.10× (1.91–2.31) | 2.73× (2.48–3.02) | 2.67× (2.43–2.93) | 2.21× (2.00–2.43) | 2.13× (1.94–2.34) | 0.85× (0.76–0.94) | 0.93× (0.83–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.45–1.55) | 3.67× (3.48–3.87) | 2.96× (2.83–3.10) | 3.02× (2.88–3.17) | 3.47× (3.29–3.67) | 4.37× (4.16–4.59) | 4.63× (4.40–4.88) | 1.58× (1.52–1.65) | 1.77× (1.69–1.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.16–1.46) | 3.17× (2.79–3.61) | 2.63× (2.23–3.09) | 3.59× (3.08–4.18) | 3.03× (2.62–3.51) | 3.59× (3.15–4.10) | 3.80× (3.33–4.35) | 1.30× (1.15–1.48) | 1.45× (1.28–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.48–1.68) | 3.27× (2.85–3.74) | 2.47× (2.27–2.69) | 3.23× (3.02–3.46) | 3.18× (2.81–3.60) | 2.67× (2.44–2.92) | 2.83× (2.56–3.13) | 0.97× (0.88–1.07) | 1.08× (0.98–1.19) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.69) | 2.53× (2.47–2.59) | 3.50× (3.15–3.89) | 2.80× (2.71–2.88) | 6.30× (6.08–6.53) | 2.18× (2.09–2.28) | 2.27× (1.98–2.61) | 0.97× (0.95–1.00) | 1.52× (1.45–1.58) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.51× (0.42–0.61) | 2.13× (1.96–2.32) | 3.58× (2.96–4.33) | 2.36× (1.96–2.84) | 4.95× (4.11–5.95) | 1.76× (1.52–2.04) | 1.84× (1.45–2.33) | 0.79× (0.65–0.95) | 1.23× (1.01–1.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.48× (0.39–0.59) | 2.13× (1.75–2.60) | 3.50× (2.87–4.27) | 2.40× (1.97–2.93) | 5.22× (4.58–5.96) | 1.78× (1.45–2.19) | 1.85× (1.45–2.38) | 0.79× (0.66–0.96) | 1.24× (1.06–1.45) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.81× (1.77–1.85) | 4.25× (4.12–4.39) | 4.65× (4.50–4.80) | 4.96× (4.68–5.26) | 5.18× (4.96–5.41) | 5.39× (5.07–5.73) | 4.88× (4.71–5.06) | 1.59× (1.54–1.64) | 1.79× (1.68–1.90) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.51× (1.46–1.55) | 3.61× (3.42–3.80) | 3.85× (3.76–3.94) | 4.53× (4.23–4.84) | 4.22× (3.93–4.53) | 4.22× (4.02–4.43) | 3.82× (3.70–3.94) | 1.24× (1.20–1.29) | 1.40× (1.32–1.49) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.51× (1.40–1.63) | 2.76× (2.51–3.04) | 3.05× (2.89–3.21) | 3.46× (3.08–3.88) | 3.36× (3.01–3.74) | 2.59× (2.35–2.85) | 2.34× (2.15–2.56) | 0.76× (0.70–0.83) | 0.86× (0.78–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.79× (0.78–0.80) | 11.00× (10.99–11.01) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.36) | 0.49× (0.42–0.56) | 0.70× (0.67–0.74) | 0.59× (0.57–0.62) | 5.62× (5.38–5.86) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.35–0.38) | 0.53× (0.43–0.64) | 0.64× (0.62–0.67) | 0.57× (0.54–0.59) | 3.73× (3.57–3.89) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.45× (0.43–0.46) | 1.69× (1.68–1.69) | 0.66× (0.65–0.67) | 7.97× (7.94–8.00) | 0.65× (0.64–0.65) | 0.68× (0.67–0.68) | 0.73× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.71× (0.68–0.74) | 0.70× (0.65–0.77) | 1.65× (1.60–1.70) | 0.83× (0.80–0.86) | 5.95× (5.78–6.13) | 0.48× (0.47–0.50) | 0.50× (0.49–0.52) | 0.55× (0.53–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.65× (0.61–0.69) | 0.69× (0.64–0.74) | 1.17× (1.09–1.25) | 0.73× (0.68–0.78) | 3.42× (3.23–3.63) | 0.27× (0.25–0.28) | 0.28× (0.26–0.30) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.15) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.50) | 0.41× (0.40–0.42) | 0.89× (0.86–0.91) | 0.71× (0.69–0.72) | 3.07× (2.98–3.15) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.64–0.66) | 0.47× (0.46–0.47) | 0.89× (0.87–0.90) | 0.82× (0.80–0.85) | 2.58× (2.54–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.38× (0.37–0.39) | 0.83× (0.80–0.85) | 0.70× (0.68–0.72) | 2.84× (2.76–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.48× (0.45–0.51) | 0.81× (0.79–0.83) | 0.82× (0.79–0.86) | 2.33× (2.27–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.20–0.23) | 1.06× (1.06–1.06) | 0.75× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.21× (6.21–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.19–6.22) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.40) | 0.67× (0.67–0.67) | 6.40× (6.39–6.40) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.47× (0.45–0.50) | 0.40× (0.37–0.42) | 1.08× (1.02–1.14) | 0.72× (0.69–0.75) | 3.50× (3.34–3.67) | 0.21× (0.20–0.22) | 0.23× (0.21–0.24) | 0.34× (0.33–0.36) | 0.17× (0.16–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 56 MB | 0.51× (0.49–0.52) | 0.45× (0.42–0.49) | 0.91× (0.89–0.94) | 0.77× (0.76–0.78) | 2.33× (2.29–2.38) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 695 | 1.45 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,166 | 1.61 | 1.10× (1.09–1.12) | 84% | 1.07× (1.04–1.10) | 1.05× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,413 | 2.47 | 1.71× (1.66–1.77) | 51% | 1.28× (1.19–1.38) | 1.80× (1.69–1.91) |

**Best:** mibz2 at 4 threads (1,413 MB/s); lbzip2 at 4 threads (308 MB/s); pbz2 at 4 threads (464 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.06, lbzip2 1t 0.08, lbzip2 2t 0.29, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.52. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.39 | 0.01 | 0.07 | 0.148 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4253 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 11.6% |
| MTF + RLE2 | 1.47 | 20.6% |
| T-vector build | 1.30 | 18.2% |
| walk (the one each build uses) | 3.10 | 43.6% |
| of which: segment assembly | 0.14 | 2.0% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.26 | 3.6% |
| **total** | 7.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.301 | 0.715 | (in its total) | 2.41× (2.38–2.44) |  |
| walk | 0.654 | 1.596 | (in its total) | 2.48× (2.45–2.52) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.030 | 0.032 | (in its total) | 1.09× (1.03–1.16) |  |
| inverse BWT, total | 0.987 | 2.344 | 1.341 | 2.42× (2.39–2.45) | 1.39× (1.36–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m12s ± 5m15s |
| 4 threads | 21m46s ± 3m08s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m18s ± 2m08s |
| the scanner alone (its serial part) | 7m08s ± 19.7s |
| the workers alone, over 4 | 14m18s ± 2m08s |

At 4 threads the workers bound it. Per byte: the scanner 4.1 ns a byte of bz2, the workers 1.87 ns a byte of output at 4 threads (CPU, summed over them), 1.22 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4252.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
