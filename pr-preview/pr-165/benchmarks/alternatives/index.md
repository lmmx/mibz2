# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-165/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`08e7fc3`](<https://github.com/lmmx/mibz2/commit/08e7fc3b46c7a6adb61ce0e70b64e9973b95cbb7>) (2026-11-04).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41930635613>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.425 s | 493 | 5.3 | 2.08× (1.99–2.18) | 5.29× (5.07–5.53) | 5.52× (5.34–5.70) | 6.28× (6.00–6.57) | 5.58× (5.36–5.81) | 5.89× (5.62–6.18) | 5.49× (5.26–5.73) | 1.63× (1.55–1.71) | 1.86× (1.78–1.96) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.332 s | 632 | 6.4 | 1.65× (1.54–1.76) | 3.74× (3.40–4.10) | 4.46× (4.16–4.78) | 4.46× (4.12–4.82) | 3.95× (3.65–4.28) | 7.70× (7.05–8.40) | 7.16× (6.61–7.76) | 2.13× (1.96–2.31) | 2.43× (2.24–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.287 s | 732 | 11.4 | 1.51× (1.45–1.57) | 2.48× (2.38–2.58) | 3.22× (3.06–3.39) | 2.99× (2.87–3.11) | 3.01× (2.90–3.13) | 8.46× (7.96–8.99) | 7.87× (7.45–8.32) | 2.34× (2.21–2.48) | 2.68× (2.52–2.84) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,320 | 2.2 | 0.56× (0.55–0.57) | 3.43× (3.31–3.56) | 4.73× (4.55–4.91) | 3.68× (3.61–3.76) | 6.59× (6.22–6.99) | 2.75× (2.70–2.79) | 2.79× (2.68–2.91) | 0.99× (0.98–1.01) | 2.02× (1.99–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,282 | 2.3 | 0.51× (0.50–0.52) | 3.31× (3.25–3.38) | 5.78× (5.57–6.00) | 3.60× (3.54–3.65) | 6.18× (6.08–6.28) | 2.70× (2.65–2.75) | 2.74× (2.64–2.85) | 0.97× (0.95–1.00) | 1.99× (1.95–2.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,274 | 2.2 | 0.54× (0.46–0.63) | 3.29× (3.20–3.37) | 5.65× (5.37–5.95) | 3.57× (3.48–3.65) | 6.21× (6.05–6.37) | 2.66× (2.59–2.74) | 2.71× (2.57–2.85) | 0.96× (0.94–0.99) | 1.96× (1.91–2.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.022 s | 881 | 3.2 | 2.45× (2.15–2.80) | 3.04× (2.70–3.43) | 3.53× (3.13–3.98) | 3.80× (3.36–4.31) | 4.53× (4.01–5.12) | 2.32× (2.06–2.62) | 2.27× (1.95–2.65) | 1.78× (1.57–2.01) | 2.60× (2.31–2.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,285 | 4.1 | 2.05× (1.80–2.34) | 2.75× (2.38–3.18) | 3.92× (3.51–4.38) | 3.33× (2.95–3.75) | 4.98× (4.48–5.54) | 3.37× (3.01–3.77) | 3.30× (2.86–3.80) | 2.58× (2.31–2.88) | 3.77× (3.38–4.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 947 | 8.9 | 1.40× (1.36–1.44) | 1.53× (1.45–1.61) | 2.22× (2.11–2.34) | 1.72× (1.63–1.81) | 3.54× (3.39–3.70) | 2.61× (2.48–2.74) | 2.55× (2.31–2.83) | 2.00× (1.92–2.07) | 2.92× (2.79–3.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.092 s | 87 | 30.4 | 1.25× (1.23–1.27) | 2.87× (2.80–2.95) | 2.54× (2.48–2.60) | 2.56× (2.43–2.69) | 2.84× (2.76–2.92) | 3.38× (3.27–3.49) | 3.18× (3.09–3.28) | 1.21× (1.18–1.24) | 1.41× (1.37–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 134 | 34.5 | 1.15× (1.09–1.22) | 2.54× (2.42–2.66) | 2.38× (2.29–2.47) | 3.00× (2.83–3.19) | 2.79× (2.66–2.92) | 5.27× (5.00–5.56) | 4.97× (4.78–5.17) | 1.89× (1.80–1.98) | 2.20× (2.11–2.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 164 | 48.5 | 1.18× (1.17–1.20) | 2.22× (2.15–2.30) | 2.25× (2.17–2.34) | 2.64× (2.57–2.72) | 2.88× (2.83–2.93) | 6.53× (6.33–6.74) | 6.15× (5.97–6.35) | 2.34× (2.29–2.39) | 2.73× (2.65–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.119 s | 81 | 32.7 | 1.08× (0.99–1.18) | 2.67× (2.46–2.90) | 2.21× (2.03–2.41) | 2.26× (2.09–2.44) | 2.50× (2.29–2.73) | 3.37× (3.09–3.67) | 3.43× (3.32–3.55) | 1.14× (1.05–1.24) | 1.31× (1.19–1.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 133 | 36.9 | 1.01× (0.98–1.05) | 2.43× (2.33–2.54) | 2.16× (2.04–2.29) | 2.90× (2.79–3.02) | 2.65× (2.52–2.77) | 5.68× (5.48–5.89) | 5.79× (5.59–5.99) | 1.93× (1.86–2.00) | 2.21× (2.10–2.31) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 167 | 50.3 | 1.15× (1.12–1.18) | 2.22× (2.15–2.29) | 2.18× (2.02–2.34) | 2.86× (2.77–2.95) | 2.71× (2.58–2.84) | 7.35× (7.23–7.46) | 7.49× (7.11–7.88) | 2.49× (2.45–2.54) | 2.86× (2.77–2.94) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 995 | 3.1 | 0.61× (0.60–0.62) | 3.09× (3.04–3.15) | 4.35× (4.16–4.55) | 3.16× (3.11–3.22) | 7.08× (6.97–7.20) | 2.36× (2.33–2.39) | 2.35× (2.32–2.38) | 0.97× (0.95–0.98) | 2.07× (2.03–2.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 962 | 3.2 | 0.50× (0.49–0.51) | 2.94× (2.86–3.02) | 5.05× (4.91–5.20) | 3.00× (2.92–3.08) | 6.63× (6.45–6.81) | 2.24× (2.19–2.29) | 2.23× (2.18–2.29) | 0.92× (0.90–0.94) | 1.96× (1.91–2.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 926 | 3.2 | 0.50× (0.49–0.51) | 2.89× (2.83–2.95) | 4.95× (4.84–5.06) | 2.94× (2.91–2.98) | 6.65× (6.33–6.98) | 2.19× (2.16–2.23) | 2.19× (2.15–2.22) | 0.90× (0.88–0.92) | 1.92× (1.88–1.96) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 0.053 s | 233 | 11.5 | 1.33× (1.29–1.37) | 3.08× (2.97–3.19) | 3.38× (3.26–3.51) | 3.45× (3.32–3.58) | 3.85× (3.66–4.05) | 3.62× (3.47–3.78) | 3.34× (3.21–3.48) | 1.13× (1.09–1.17) | 1.34× (1.28–1.40) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 0.037 s | 334 | 14.1 | 1.04× (0.95–1.14) | 2.68× (2.54–2.83) | 3.00× (2.68–3.36) | 3.34× (2.85–3.92) | 3.37× (3.06–3.71) | 5.15× (4.68–5.66) | 4.75× (4.32–5.23) | 1.60× (1.46–1.76) | 1.90× (1.71–2.12) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 0.033 s | 375 | 23.4 | 1.09× (1.06–1.12) | 1.97× (1.88–2.06) | 2.63× (2.49–2.77) | 2.78× (2.69–2.88) | 3.25× (3.15–3.34) | 6.04× (5.86–6.24) | 5.58× (5.41–5.75) | 1.88× (1.83–1.94) | 2.23× (2.14–2.33) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.08× (1.99–2.17) | 5.26× (5.04–5.50) | 5.48× (5.31–5.66) | 6.25× (5.98–6.54) | 5.56× (5.34–5.79) | 5.87× (5.60–6.16) | 5.47× (5.24–5.71) | 1.63× (1.55–1.71) | 1.85× (1.76–1.94) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.79× (1.69–1.89) | 4.45× (4.14–4.78) | 4.77× (4.55–5.00) | 5.43× (5.10–5.78) | 4.59× (4.29–4.92) | 4.77× (4.40–5.16) | 4.44× (4.12–4.78) | 1.32× (1.22–1.43) | 1.50× (1.39–1.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.62× (1.57–1.68) | 3.03× (2.89–3.18) | 3.41× (3.28–3.55) | 3.67× (3.51–3.84) | 3.21× (3.06–3.36) | 2.65× (2.45–2.88) | 2.47× (2.29–2.66) | 0.74× (0.68–0.80) | 0.83× (0.77–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.03× (2.97–3.08) | 3.88× (3.79–3.97) | 3.42× (3.34–3.50) | 6.04× (5.67–6.43) | 2.58× (2.52–2.63) | 2.62× (2.51–2.72) | 1.00× (0.98–1.02) | 1.81× (1.77–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.57) | 2.79× (2.75–2.83) | 4.91× (4.86–4.96) | 3.31× (3.27–3.36) | 5.59× (5.52–5.67) | 2.49× (2.45–2.53) | 2.53× (2.43–2.62) | 0.96× (0.94–0.99) | 1.75× (1.72–1.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.50–0.70) | 3.00× (2.94–3.07) | 4.95× (4.84–5.06) | 3.37× (3.30–3.44) | 5.73× (5.60–5.86) | 2.53× (2.47–2.59) | 2.56× (2.44–2.69) | 0.98× (0.95–1.00) | 1.77× (1.73–1.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.32× (2.04–2.64) | 2.59× (2.28–2.93) | 3.13× (2.75–3.57) | 3.56× (3.12–4.06) | 4.22× (3.71–4.81) | 2.20× (1.93–2.49) | 2.15× (1.84–2.52) | 1.70× (1.49–1.93) | 2.29× (2.02–2.60) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (1.97–2.20) | 2.45× (2.25–2.67) | 2.81× (2.70–2.93) | 3.08× (2.94–3.24) | 3.55× (3.38–3.72) | 1.80× (1.71–1.89) | 1.76× (1.61–1.93) | 1.39× (1.33–1.45) | 1.87× (1.78–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.58× (1.49–1.68) | 1.42× (1.32–1.54) | 1.71× (1.61–1.82) | 1.72× (1.61–1.83) | 1.92× (1.80–2.04) | 0.83× (0.78–0.89) | 0.82× (0.74–0.90) | 0.65× (0.61–0.69) | 0.87× (0.82–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.25× (1.22–1.27) | 2.78× (2.71–2.84) | 2.48× (2.42–2.54) | 2.53× (2.41–2.66) | 2.81× (2.73–2.88) | 3.33× (3.22–3.45) | 3.14× (3.04–3.24) | 1.21× (1.18–1.23) | 1.33× (1.28–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.19× (1.15–1.23) | 2.54× (2.49–2.58) | 2.22× (2.18–2.25) | 2.91× (2.86–2.97) | 2.53× (2.49–2.58) | 2.97× (2.89–3.06) | 2.80× (2.72–2.88) | 1.07× (1.05–1.09) | 1.18× (1.15–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.27× (1.25–1.30) | 2.31× (2.26–2.36) | 1.96× (1.88–2.05) | 2.55× (2.48–2.61) | 2.37× (2.31–2.43) | 2.13× (2.08–2.19) | 2.01× (1.94–2.08) | 0.77× (0.76–0.79) | 0.85× (0.83–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.08× (0.99–1.18) | 2.61× (2.40–2.83) | 2.14× (1.97–2.33) | 2.25× (2.08–2.43) | 2.48× (2.27–2.71) | 3.34× (3.06–3.63) | 3.40× (3.29–3.52) | 1.14× (1.06–1.24) | 1.27× (1.16–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.05) | 2.48× (2.43–2.53) | 2.01× (1.92–2.11) | 2.75× (2.70–2.81) | 2.40× (2.23–2.59) | 3.06× (3.00–3.12) | 3.12× (2.96–3.28) | 1.05× (1.03–1.07) | 1.16× (1.13–1.19) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.24× (1.21–1.27) | 2.36× (2.30–2.43) | 1.84× (1.70–1.99) | 2.53× (2.46–2.61) | 2.31× (2.21–2.41) | 2.27× (2.23–2.32) | 2.32× (2.20–2.44) | 0.78× (0.76–0.80) | 0.86× (0.84–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.55× (2.51–2.59) | 3.35× (3.30–3.41) | 2.85× (2.81–2.90) | 6.22× (6.08–6.36) | 2.17× (2.13–2.21) | 2.16× (2.12–2.20) | 0.97× (0.96–0.98) | 1.49× (1.46–1.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.31× (2.26–2.37) | 3.96× (3.86–4.06) | 2.74× (2.67–2.81) | 5.77× (5.63–5.92) | 2.03× (1.99–2.08) | 2.03× (1.98–2.07) | 0.91× (0.89–0.93) | 1.39× (1.36–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.46× (2.43–2.50) | 3.98× (3.91–4.05) | 2.79× (2.76–2.82) | 5.98× (5.69–6.29) | 2.06× (2.03–2.10) | 2.05× (2.02–2.09) | 0.92× (0.91–0.94) | 1.41× (1.39–1.43) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 1.32× (1.28–1.36) | 2.95× (2.84–3.07) | 3.19× (3.08–3.31) | 3.38× (3.25–3.51) | 3.77× (3.58–3.96) | 3.55× (3.39–3.71) | 3.28× (3.14–3.42) | 1.13× (1.08–1.17) | 1.24× (1.20–1.29) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 1.16× (1.13–1.18) | 2.51× (2.44–2.58) | 2.76× (2.67–2.85) | 3.08× (2.99–3.17) | 3.05× (2.95–3.15) | 2.89× (2.80–2.99) | 2.67× (2.59–2.76) | 0.92× (0.89–0.94) | 1.01× (0.98–1.05) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 1.14× (1.10–1.19) | 1.92× (1.85–1.99) | 2.22× (2.08–2.37) | 2.35× (2.26–2.45) | 2.37× (2.30–2.44) | 1.78× (1.73–1.84) | 1.65× (1.60–1.70) | 0.57× (0.55–0.58) | 0.62× (0.61–0.64) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.36× (0.35–0.37) | 1.05× (1.04–1.05) | 0.83× (0.82–0.85) | 11.81× (11.80–11.83) | 0.27× (0.27–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 50 MB | 0.37× (0.36–0.38) | 0.56× (0.50–0.64) | 0.79× (0.77–0.80) | 0.65× (0.64–0.66) | 6.24× (6.14–6.34) | 0.14× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.66× (0.54–0.81) | 0.74× (0.72–0.75) | 0.64× (0.62–0.66) | 4.28× (4.25–4.30) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.28–4.29) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.83× (0.83–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.18–0.19) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.83× (0.82–0.83) | 4.39× (4.39–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.48–0.49) | 0.43× (0.42–0.44) | 1.65× (1.64–1.65) | 0.63× (0.63–0.63) | 7.76× (7.74–7.78) | 0.62× (0.61–0.62) | 0.65× (0.65–0.65) | 0.71× (0.70–0.71) | 0.36× (0.35–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.64× (0.62–0.66) | 0.63× (0.58–0.68) | 1.53× (1.51–1.55) | 0.76× (0.75–0.77) | 5.53× (5.47–5.59) | 0.44× (0.43–0.44) | 0.46× (0.46–0.47) | 0.50× (0.50–0.51) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.58–0.61) | 0.68× (0.64–0.73) | 1.09× (1.07–1.11) | 0.70× (0.68–0.73) | 3.17× (3.14–3.20) | 0.24× (0.24–0.24) | 0.26× (0.25–0.26) | 0.28× (0.28–0.28) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.71× (0.71–0.71) | 5.09× (5.08–5.09) | 0.31× (0.31–0.32) | 0.33× (0.33–0.34) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.49–0.52) | 0.40× (0.36–0.43) | 0.93× (0.91–0.95) | 0.74× (0.72–0.76) | 3.23× (3.16–3.30) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.59× (0.57–0.61) | 0.42× (0.41–0.43) | 0.81× (0.79–0.83) | 0.74× (0.73–0.76) | 2.34× (2.29–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.12× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.16) | 0.80× (0.80–0.81) | 5.10× (5.09–5.10) | 0.31× (0.31–0.31) | 0.33× (0.32–0.34) | 0.50× (0.50–0.51) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.40× (0.40–0.41) | 0.87× (0.86–0.89) | 0.71× (0.68–0.74) | 3.00× (2.95–3.04) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.54× (0.53–0.56) | 0.43× (0.40–0.46) | 0.73× (0.72–0.74) | 0.74× (0.71–0.76) | 2.11× (2.08–2.14) | 0.12× (0.12–0.12) | 0.13× (0.13–0.13) | 0.20× (0.20–0.20) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.08× (1.08–1.09) | 0.76× (0.76–0.76) | 5.42× (5.41–5.43) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.62–1.63) | 0.77× (0.76–0.77) | 5.39× (5.37–5.41) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.21× (0.20–0.21) | 1.63× (1.63–1.63) | 0.77× (0.77–0.77) | 5.39× (5.38–5.41) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.44× (1.44–1.45) | 0.62× (0.62–0.63) | 6.52× (6.50–6.54) | 0.39× (0.39–0.39) | 0.41× (0.40–0.42) | 0.63× (0.63–0.64) | 0.32× (0.31–0.32) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 32 MB | 0.49× (0.47–0.50) | 0.42× (0.37–0.47) | 1.16× (1.12–1.21) | 0.73× (0.71–0.75) | 3.70× (3.60–3.81) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.36× (0.35–0.37) | 0.18× (0.17–0.19) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 55 MB | 0.49× (0.48–0.50) | 0.42× (0.38–0.46) | 0.93× (0.91–0.94) | 0.73× (0.72–0.75) | 2.36× (2.33–2.40) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 493 | 2.04 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 632 | 2.47 | 1.23× (1.18–1.28) | 65% | 1.04× (1.01–1.07) | 1.06× (1.03–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 732 | 4.41 | 2.21× (2.13–2.30) | 36% | 1.27× (1.19–1.36) | 1.73× (1.64–1.82) |

**Best:** mibz2 at 4 threads (732 MB/s); lbzip2 at 4 threads (299 MB/s); pbz2 at 4 threads (480 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.01, lbzip2 1t 0.07, lbzip2 2t 0.28, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.78 | 0.01 | 0.08 | 0.287 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.63 | 13.6% |
| MTF + RLE2 | 1.15 | 24.8% |
| T-vector build | 0.90 | 19.3% |
| walk (the one each build uses) | 1.72 | 37.0% |
| of which: segment assembly | 0.07 | 1.5% |
| RLE1 | 0.09 | 2.0% |
| CRC | 0.15 | 3.3% |
| **total** | 4.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.325 | 0.719 | (in its total) | 2.20× (2.15–2.26) |  |
| walk | 0.676 | 1.628 | (in its total) | 2.39× (2.30–2.48) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.027 | 0.030 | (in its total) | 1.11× (1.06–1.16) |  |
| inverse BWT, total | 1.026 | 2.379 | 1.325 | 2.30× (2.23–2.37) | 1.27× (1.25–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h02m ± 8m18s |
| 4 threads | 43m53s ± 6m58s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m59s ± 5m08s |
| the scanner alone (its serial part) | 10m11s ± 41.6s |
| the workers alone, over 4 | 28m59s ± 5m08s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.80 ns a byte of output at 4 threads (CPU, summed over them), 1.73 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,655,880 | 12,373,150 | \`a86d8c50ea2ccd84…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
