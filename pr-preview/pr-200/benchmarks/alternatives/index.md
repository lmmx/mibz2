# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-200/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`d1fd440`](<https://github.com/lmmx/mibz2/commit/d1fd440596963ff2406c132f43f196fa51f78745>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41980640481>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.802 s | 261 | 11.5 | 1.98× (1.96–2.00) | 4.42× (4.39–4.44) | 4.73× (4.70–4.76) | 5.34× (5.30–5.37) | 4.74× (4.72–4.77) | 5.08× (5.05–5.12) | 4.90× (4.87–4.93) | 1.52× (1.51–1.53) | 1.42× (1.41–1.43) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.426 s | 492 | 12.0 | 2.05× (1.85–2.27) | 4.24× (3.94–4.56) | 5.12× (4.58–5.73) | 5.14× (4.70–5.62) | 4.52× (4.13–4.94) | 9.04× (8.27–9.89) | 8.71× (7.95–9.55) | 2.70× (2.47–2.95) | 2.53× (2.31–2.77) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.343 s | 612 | 18.8 | 2.35× (2.31–2.40) | 4.03× (3.96–4.09) | 4.63× (4.54–4.72) | 4.70× (4.56–4.84) | 4.49× (4.39–4.59) | 11.89× (11.72–12.05) | 11.45× (11.30–11.61) | 3.55× (3.50–3.60) | 3.32× (3.28–3.37) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 935 | 3.5 | 0.58× (0.57–0.60) | 3.32× (3.28–3.37) | 4.25× (4.14–4.37) | 3.75× (3.70–3.80) | 6.35× (6.01–6.72) | 2.80× (2.76–2.83) | 2.79× (2.76–2.83) | 1.15× (1.12–1.18) | 0.96× (0.95–0.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 935 | 3.5 | 0.57× (0.53–0.60) | 2.87× (2.82–2.93) | 5.73× (5.55–5.91) | 3.80× (3.73–3.87) | 6.20× (6.08–6.31) | 2.82× (2.76–2.88) | 2.81× (2.75–2.87) | 1.15× (1.12–1.19) | 0.97× (0.95–0.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 917 | 3.5 | 0.54× (0.54–0.55) | 2.89× (2.74–3.05) | 5.72× (5.54–5.91) | 3.70× (3.63–3.77) | 6.13× (6.01–6.24) | 2.74× (2.70–2.78) | 2.74× (2.69–2.78) | 1.12× (1.10–1.15) | 0.94× (0.93–0.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.041 s | 482 | 6.5 | 2.70× (2.68–2.73) | 3.26× (3.24–3.27) | 3.73× (3.70–3.75) | 4.28× (4.26–4.31) | 4.63× (4.59–4.67) | 3.12× (3.10–3.14) | 2.85× (2.84–2.87) | 2.00× (1.97–2.04) | 2.00× (1.99–2.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.026 s | 754 | 7.7 | 2.28× (2.16–2.40) | 2.93× (2.70–3.18) | 3.95× (3.73–4.18) | 3.65× (3.47–3.84) | 4.97× (4.69–5.26) | 4.78× (4.52–5.07) | 4.37× (4.13–4.63) | 3.07× (2.89–3.27) | 3.07× (2.90–3.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 761 | 13.9 | 2.05× (1.97–2.14) | 2.09× (1.94–2.26) | 3.05× (2.82–3.30) | 2.47× (2.35–2.59) | 4.16× (3.98–4.36) | 4.88× (4.66–5.11) | 4.46× (4.26–4.67) | 3.14× (3.01–3.27) | 3.13× (2.99–3.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.164 s | 49 | 62.0 | 1.25× (1.23–1.27) | 2.45× (2.40–2.50) | 2.14× (2.09–2.18) | 2.21× (2.17–2.26) | 2.50× (2.45–2.54) | 2.89× (2.84–2.94) | 2.98× (2.92–3.03) | 1.27× (1.24–1.29) | 1.26× (1.24–1.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.095 s | 84 | 66.1 | 1.21× (1.17–1.27) | 2.36× (2.25–2.48) | 2.20× (2.11–2.29) | 2.91× (2.79–3.04) | 2.66× (2.55–2.76) | 4.88× (4.70–5.07) | 5.02× (4.83–5.21) | 2.13× (2.06–2.22) | 2.12× (2.03–2.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 106 | 85.6 | 1.37× (1.33–1.41) | 2.36× (2.30–2.43) | 2.32× (2.24–2.39) | 2.83× (2.77–2.89) | 2.87× (2.80–2.94) | 6.20× (6.05–6.36) | 6.38× (6.22–6.54) | 2.71× (2.65–2.78) | 2.70× (2.66–2.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.206 s | 47 | 64.9 | 1.11× (1.03–1.20) | 2.41× (2.22–2.61) | 1.99× (1.85–2.13) | 2.05× (1.89–2.21) | 2.32× (2.14–2.51) | 2.99× (2.75–3.24) | 3.18× (2.94–3.45) | 1.26× (1.16–1.37) | 1.27× (1.18–1.37) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.112 s | 86 | 67.4 | 1.15× (1.13–1.16) | 2.47× (2.44–2.49) | 2.18× (2.14–2.23) | 3.16× (3.04–3.28) | 2.65× (2.58–2.72) | 5.63× (5.54–5.71) | 6.00× (5.90–6.10) | 2.37× (2.34–2.41) | 2.39× (2.35–2.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.091 s | 106 | 89.7 | 1.35× (1.32–1.39) | 2.46× (2.39–2.52) | 2.20× (2.14–2.27) | 3.02× (2.93–3.12) | 2.71× (2.64–2.78) | 7.00× (6.85–7.16) | 7.46× (7.29–7.64) | 2.95× (2.88–3.03) | 2.98× (2.90–3.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 702 | 4.8 | 0.62× (0.61–0.63) | 2.89× (2.86–2.91) | 4.24× (4.20–4.28) | 3.22× (3.19–3.25) | 6.76× (6.67–6.84) | 2.42× (2.39–2.45) | 2.42× (2.40–2.45) | 1.05× (1.03–1.06) | 1.46× (1.44–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.1 | 0.52× (0.51–0.53) | 2.82× (2.69–2.95) | 4.73× (4.67–4.80) | 3.10× (3.06–3.13) | 6.46× (6.35–6.57) | 2.30× (2.26–2.34) | 2.31× (2.28–2.33) | 1.00× (0.98–1.02) | 1.39× (1.37–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 671 | 5.0 | 0.53× (0.53–0.54) | 2.78× (2.73–2.83) | 4.78× (4.70–4.86) | 3.12× (3.07–3.18) | 6.46× (6.34–6.58) | 2.32× (2.28–2.36) | 2.32× (2.28–2.37) | 1.00× (0.99–1.02) | 1.40× (1.38–1.42) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 0.083 s | 145 | 21.1 | 1.46× (1.44–1.47) | 3.17× (3.15–3.19) | 3.55× (3.51–3.60) | 3.61× (3.59–3.63) | 3.77× (3.75–3.79) | 3.73× (3.69–3.76) | 3.82× (3.77–3.88) | 1.18× (1.17–1.19) | 1.16× (1.11–1.21) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 0.050 s | 241 | 23.4 | 1.27× (1.20–1.35) | 2.88× (2.72–3.04) | 3.51× (3.23–3.80) | 3.96× (3.86–4.06) | 3.83× (3.60–4.08) | 6.02× (5.67–6.40) | 6.17× (5.80–6.57) | 1.90× (1.79–2.03) | 1.87× (1.72–2.03) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 0.042 s | 288 | 33.7 | 1.47× (1.41–1.53) | 2.66× (2.52–2.81) | 3.13× (3.01–3.25) | 3.74× (3.64–3.84) | 3.96× (3.67–4.27) | 7.36× (7.21–7.52) | 7.55× (7.32–7.79) | 2.33× (2.27–2.39) | 2.29× (2.19–2.39) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.98× (1.96–2.00) | 4.40× (4.38–4.43) | 4.71× (4.68–4.74) | 5.33× (5.29–5.36) | 4.74× (4.71–4.76) | 5.07× (5.04–5.11) | 4.89× (4.86–4.92) | 1.52× (1.50–1.53) | 1.41× (1.40–1.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.00× (1.98–2.01) | 4.46× (4.37–4.55) | 4.92× (4.85–4.98) | 5.50× (5.46–5.53) | 4.62× (4.60–4.65) | 4.88× (4.85–4.90) | 4.70× (4.67–4.73) | 1.46× (1.45–1.47) | 1.36× (1.35–1.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.17–2.25) | 4.18× (4.11–4.24) | 4.36× (4.31–4.41) | 4.85× (4.78–4.92) | 4.21× (4.17–4.25) | 3.13× (3.08–3.18) | 3.02× (2.97–3.07) | 0.94× (0.92–0.95) | 0.87× (0.86–0.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.63) | 2.87× (2.82–2.91) | 3.96× (3.87–4.05) | 3.54× (3.49–3.60) | 5.96× (5.63–6.30) | 2.67× (2.63–2.71) | 2.66× (2.62–2.69) | 1.14× (1.11–1.17) | 0.90× (0.89–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.58–0.64) | 2.88× (2.82–2.94) | 5.22× (5.11–5.34) | 3.57× (3.50–3.64) | 5.76× (5.65–5.87) | 2.66× (2.60–2.72) | 2.65× (2.59–2.70) | 1.13× (1.10–1.17) | 0.90× (0.88–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.58) | 3.21× (3.16–3.27) | 5.15× (5.07–5.24) | 3.53× (3.47–3.59) | 5.75× (5.64–5.85) | 2.62× (2.58–2.66) | 2.61× (2.57–2.65) | 1.12× (1.10–1.14) | 0.89× (0.87–0.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.61–2.66) | 3.03× (3.02–3.05) | 3.55× (3.53–3.57) | 4.15× (4.12–4.17) | 4.48× (4.45–4.51) | 3.03× (3.02–3.04) | 2.78× (2.76–2.79) | 1.96× (1.93–1.99) | 1.86× (1.85–1.88) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.31–2.39) | 2.73× (2.66–2.79) | 3.21× (3.14–3.27) | 3.54× (3.47–3.61) | 3.86× (3.79–3.92) | 2.54× (2.50–2.59) | 2.33× (2.29–2.37) | 1.65× (1.61–1.68) | 1.56× (1.53–1.60) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.18× (2.13–2.22) | 1.84× (1.77–1.92) | 2.27× (2.20–2.34) | 2.28× (2.22–2.34) | 2.32× (2.25–2.39) | 1.40× (1.36–1.44) | 1.28× (1.24–1.33) | 0.91× (0.88–0.93) | 0.86× (0.84–0.89) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.25× (1.23–1.27) | 2.40× (2.36–2.45) | 2.10× (2.06–2.14) | 2.20× (2.16–2.24) | 2.48× (2.44–2.53) | 2.87× (2.82–2.92) | 2.95× (2.90–3.01) | 1.26× (1.24–1.29) | 1.23× (1.22–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.21–1.24) | 2.40× (2.36–2.44) | 2.08× (2.05–2.11) | 2.79× (2.76–2.83) | 2.40× (2.37–2.43) | 2.69× (2.66–2.72) | 2.77× (2.73–2.80) | 1.18× (1.16–1.21) | 1.15× (1.13–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.42–1.56) | 2.60× (2.56–2.64) | 2.12× (2.07–2.18) | 2.85× (2.81–2.89) | 2.55× (2.53–2.58) | 2.06× (2.04–2.08) | 2.12× (2.10–2.14) | 0.91× (0.89–0.92) | 0.88× (0.87–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.11× (1.03–1.20) | 2.38× (2.19–2.58) | 1.95× (1.82–2.09) | 2.04× (1.89–2.20) | 2.31× (2.13–2.49) | 2.97× (2.74–3.22) | 3.17× (2.92–3.43) | 1.26× (1.16–1.36) | 1.25× (1.15–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.14× (1.13–1.15) | 2.46× (2.44–2.48) | 2.01× (1.99–2.03) | 2.85× (2.81–2.89) | 2.35× (2.32–2.38) | 2.93× (2.90–2.96) | 3.13× (3.09–3.16) | 1.24× (1.22–1.26) | 1.23× (1.21–1.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.48× (1.47–1.50) | 2.70× (2.66–2.73) | 2.03× (1.98–2.08) | 2.83× (2.77–2.88) | 2.43× (2.41–2.46) | 2.21× (2.18–2.24) | 2.36× (2.32–2.39) | 0.94× (0.92–0.95) | 0.93× (0.92–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.66) | 2.38× (2.35–2.41) | 3.47× (3.43–3.50) | 2.98× (2.95–3.00) | 6.12× (6.04–6.20) | 2.26× (2.23–2.29) | 2.27× (2.24–2.30) | 1.04× (1.03–1.06) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.31× (2.17–2.47) | 4.17× (4.13–4.21) | 2.89× (2.86–2.92) | 5.82× (5.73–5.91) | 2.13× (2.09–2.16) | 2.13× (2.11–2.16) | 0.98× (0.96–1.00) | 0.78× (0.77–0.79) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.62× (2.54–2.70) | 4.23× (4.16–4.30) | 2.98× (2.93–3.02) | 5.90× (5.81–5.99) | 2.18× (2.15–2.21) | 2.18× (2.14–2.22) | 1.00× (0.99–1.02) | 0.80× (0.79–0.81) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 1.45× (1.43–1.46) | 3.06× (3.04–3.08) | 3.45× (3.42–3.48) | 3.55× (3.53–3.57) | 3.71× (3.69–3.73) | 3.67× (3.64–3.70) | 3.76× (3.71–3.81) | 1.17× (1.17–1.18) | 1.09× (1.09–1.10) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 1.36× (1.28–1.43) | 2.82× (2.68–2.98) | 3.30× (3.11–3.50) | 3.64× (3.58–3.70) | 3.39× (3.19–3.60) | 3.23× (3.04–3.43) | 3.31× (3.12–3.51) | 1.03× (0.97–1.10) | 0.96× (0.91–1.02) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 1.65× (1.56–1.74) | 2.88× (2.81–2.95) | 3.16× (3.09–3.23) | 3.34× (3.27–3.40) | 3.29× (3.11–3.49) | 2.28× (2.25–2.31) | 2.34× (2.29–2.39) | 0.73× (0.71–0.75) | 0.68× (0.67–0.69) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.29× (0.29–0.29) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.24× (12.22–12.25) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.39× (0.38–0.39) | 0.52× (0.44–0.61) | 0.80× (0.79–0.82) | 0.67× (0.66–0.69) | 6.53× (6.45–6.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.61× (0.57–0.65) | 0.73× (0.72–0.74) | 0.65× (0.63–0.66) | 4.29× (4.26–4.31) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.33–4.34) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.66) | 8.18× (8.15–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.76) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.64× (0.61–0.68) | 1.64× (1.61–1.67) | 0.81× (0.79–0.82) | 5.95× (5.84–6.07) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.63–0.66) | 0.70× (0.64–0.76) | 1.18× (1.15–1.21) | 0.77× (0.75–0.79) | 3.51× (3.43–3.59) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.52× (0.50–0.54) | 0.42× (0.39–0.45) | 0.96× (0.93–0.99) | 0.72× (0.69–0.76) | 3.31× (3.20–3.42) | 0.20× (0.20–0.21) | 0.21× (0.21–0.22) | 0.33× (0.32–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.62× (0.61–0.64) | 0.45× (0.45–0.46) | 0.86× (0.85–0.88) | 0.80× (0.79–0.82) | 2.52× (2.48–2.55) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.29× (5.29–5.30) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.35–0.42) | 0.89× (0.86–0.93) | 0.72× (0.68–0.76) | 3.05× (2.95–3.16) | 0.18× (0.18–0.19) | 0.19× (0.18–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.61× (0.60–0.63) | 0.48× (0.47–0.50) | 0.82× (0.81–0.84) | 0.83× (0.81–0.86) | 2.38× (2.33–2.43) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.48× (5.47–5.48) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.65) | 0.78× (0.77–0.78) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.65) | 0.78× (0.77–0.78) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.41× (0.41–0.41) | 1.51× (1.51–1.52) | 0.65× (0.65–0.66) | 6.83× (6.81–6.84) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 0.66× (0.66–0.66) | 0.33× (0.33–0.34) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 30 MB | 0.52× (0.51–0.53) | 0.42× (0.39–0.46) | 1.28× (1.26–1.31) | 0.79× (0.75–0.83) | 4.03× (3.97–4.09) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) | 0.39× (0.38–0.40) | 0.20× (0.19–0.20) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 49 MB | 0.56× (0.54–0.57) | 0.56× (0.52–0.60) | 1.03× (1.03–1.04) | 0.82× (0.79–0.84) | 2.65× (2.63–2.68) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 261 | 3.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 492 | 3.99 | 1.04× (1.03–1.05) | 89% | 1.05× (1.03–1.08) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 612 | 6.27 | 1.62× (1.59–1.65) | 58% | 1.54× (1.52–1.55) | 1.81× (1.78–1.84) |

**Best:** mibz2 at 4 threads (612 MB/s); lbzip2 at 4 threads (152 MB/s); pbz2 at 4 threads (260 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.66, mibz2 4t 1.08, lbzip2 1t 0.05, lbzip2 2t 0.20, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.37. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.14 | 1.06 | 0.03 | 0.08 | 0.343 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3001 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.26 | 11.5% |
| MTF + RLE2 | 2.16 | 19.8% |
| T-vector build | 1.75 | 16.0% |
| walk (the one each build uses) | 5.32 | 48.8% |
| of which: segment assembly | 0.28 | 2.6% |
| RLE1 | 0.28 | 2.5% |
| CRC | 0.14 | 1.3% |
| **total** | 10.90 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.570 | 1.336 | (in its total) | 2.34× (2.33–2.35) |  |
| walk | 1.623 | 2.998 | (in its total) | 1.85× (1.85–1.86) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.093 | 0.084 | (in its total) | 0.90× (0.87–0.93) |  |
| inverse BWT, total | 2.285 | 4.422 | 2.398 | 1.94× (1.93–1.94) | 1.05× (1.04–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h56m ± 13m38s |
| 4 threads | 49m58s ± 7m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 38m33s ± 5m28s |
| the scanner alone (its serial part) | 19m38s ± 32.9s |
| the workers alone, over 4 | 38m33s ± 5m28s |

At 4 threads the workers bound it. Per byte: the scanner 11.4 ns a byte of bz2, the workers 5.05 ns a byte of output at 4 threads (CPU, summed over them), 3.19 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,643,407 | 12,093,030 | \`8abcc49bd9d13146…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3001.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
