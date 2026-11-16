# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-493/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`cdc1d17`](<https://github.com/lmmx/mibz2/commit/cdc1d1796cdbd0f31b236912bc0accafd161265c>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43875064736>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.323 s | 650 | 6.7 | 2.70× (2.64–2.76) | 6.89× (6.81–6.97) | 7.21× (7.11–7.32) | 8.21× (8.06–8.36) | 7.29× (7.20–7.39) | 7.43× (7.34–7.53) | 7.07× (6.97–7.17) | 2.14× (2.09–2.19) | 2.42× (2.38–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.194 s | 1,079 | 7.5 | 2.62× (2.47–2.79) | 5.88× (5.35–6.46) | 7.09× (6.62–7.59) | 6.98× (6.53–7.47) | 6.18× (5.76–6.64) | 12.01× (11.29–12.77) | 11.43× (10.75–12.14) | 3.46× (3.21–3.73) | 3.92× (3.67–4.18) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.165 s | 1,270 | 11.7 | 2.96× (2.86–3.07) | 4.39× (4.24–4.56) | 5.77× (5.56–5.99) | 5.19× (5.11–5.26) | 5.24× (5.12–5.36) | 14.67× (14.39–14.96) | 13.96× (13.72–14.21) | 4.23× (4.14–4.33) | 4.78× (4.68–4.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,375 | 3.5 | 0.57× (0.55–0.60) | 3.48× (3.39–3.56) | 4.40× (4.26–4.54) | 3.75× (3.53–4.00) | 6.50× (6.30–6.71) | 2.78× (2.70–2.87) | 2.75× (2.68–2.82) | 0.99× (0.97–1.02) | 2.09× (2.04–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,324 | 3.7 | 0.50× (0.49–0.51) | 2.96× (2.71–3.23) | 5.56× (5.37–5.76) | 3.74× (3.45–4.06) | 6.64× (5.80–7.59) | 2.69× (2.61–2.78) | 2.66× (2.59–2.73) | 0.96× (0.94–0.98) | 2.02× (1.98–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,316 | 3.6 | 0.51× (0.49–0.52) | 2.89× (2.63–3.17) | 5.66× (5.47–5.86) | 3.58× (3.50–3.66) | 6.34× (6.04–6.66) | 2.68× (2.61–2.76) | 2.65× (2.60–2.70) | 0.96× (0.93–0.98) | 2.01× (1.96–2.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,029 | 4.5 | 2.84× (2.74–2.94) | 3.70× (3.58–3.82) | 4.30× (4.17–4.44) | 4.53× (4.43–4.63) | 5.38× (5.20–5.56) | 2.98× (2.85–3.12) | 2.51× (2.43–2.59) | 2.12× (2.04–2.20) | 3.16× (3.06–3.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,233 | 6.3 | 1.85× (1.63–2.10) | 2.48× (2.02–3.04) | 3.58× (2.75–4.66) | 3.20× (2.63–3.90) | 4.79× (4.08–5.63) | 3.33× (2.80–3.96) | 2.80× (2.35–3.34) | 2.37× (1.98–2.84) | 3.53× (2.98–4.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,089 | 12.9 | 1.65× (1.58–1.73) | 1.76× (1.69–1.83) | 2.95× (2.58–3.37) | 1.94× (1.84–2.05) | 4.03× (3.90–4.16) | 3.19× (3.10–3.29) | 2.69× (2.60–2.78) | 2.27× (2.19–2.36) | 3.39× (3.27–3.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.087 s | 92 | 47.5 | 1.35× (1.34–1.37) | 3.02× (2.90–3.15) | 2.70× (2.63–2.77) | 2.65× (2.58–2.73) | 3.01× (2.94–3.08) | 3.36× (3.28–3.44) | 3.42× (3.35–3.49) | 1.31× (1.29–1.34) | 1.50× (1.47–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 147 | 54.9 | 1.30× (1.25–1.36) | 2.84× (2.72–2.96) | 2.65× (2.49–2.83) | 3.23× (3.01–3.46) | 3.01× (2.94–3.08) | 5.36× (5.22–5.51) | 5.46× (5.33–5.60) | 2.10× (2.05–2.15) | 2.40× (2.31–2.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 181 | 72.1 | 1.21× (1.05–1.40) | 2.28× (1.94–2.68) | 2.26× (1.95–2.63) | 2.56× (2.24–2.92) | 2.99× (2.60–3.43) | 6.15× (5.33–7.08) | 6.26× (5.44–7.20) | 2.40× (2.09–2.77) | 2.75× (2.42–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.091 s | 106 | 41.4 | 1.39× (1.26–1.54) | 3.55× (3.25–3.88) | 2.94× (2.73–3.16) | 2.84× (2.62–3.09) | 3.35× (3.04–3.68) | 4.19× (3.83–4.59) | 4.44× (4.07–4.84) | 1.49× (1.37–1.63) | 1.72× (1.59–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 168 | 47.9 | 1.27× (1.20–1.34) | 3.11× (2.89–3.34) | 2.65× (2.50–2.82) | 3.53× (3.28–3.81) | 3.17× (2.94–3.43) | 6.68× (6.20–7.20) | 7.07× (6.60–7.56) | 2.38× (2.22–2.54) | 2.74× (2.55–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.049 s | 196 | 68.2 | 1.36× (1.31–1.42) | 2.61× (2.45–2.77) | 2.45× (2.37–2.53) | 3.17× (3.09–3.25) | 3.04× (2.98–3.11) | 7.98× (7.73–8.23) | 8.44× (8.24–8.65) | 2.84× (2.77–2.91) | 3.27× (3.20–3.35) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 4.9 | 0.55× (0.44–0.70) | 2.85× (2.24–3.61) | 3.92× (3.09–4.98) | 3.14× (2.65–3.72) | 6.47× (5.13–8.16) | 2.08× (1.65–2.62) | 2.12× (1.68–2.68) | 0.87× (0.69–1.10) | 1.90× (1.50–2.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 962 | 5.3 | 0.52× (0.49–0.55) | 2.98× (2.89–3.07) | 5.43× (4.70–6.27) | 2.99× (2.87–3.11) | 6.60× (6.37–6.85) | 2.17× (2.08–2.27) | 2.22× (2.15–2.30) | 0.91× (0.88–0.93) | 1.99× (1.93–2.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 952 | 5.2 | 0.52× (0.48–0.58) | 2.98× (2.90–3.06) | 5.10× (4.94–5.26) | 2.99× (2.94–3.04) | 6.65× (6.45–6.86) | 2.16× (2.10–2.23) | 2.21× (2.15–2.27) | 0.90× (0.87–0.93) | 1.98× (1.92–2.03) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 0.047 s | 289 | 15.4 | 1.72× (1.69–1.75) | 4.28× (4.09–4.47) | 4.62× (4.54–4.71) | 4.94× (4.69–5.22) | 4.92× (4.84–4.99) | 5.00× (4.89–5.11) | 4.77× (4.68–4.86) | 1.52× (1.43–1.62) | 1.77× (1.64–1.91) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 0.037 s | 373 | 20.5 | 1.35× (1.28–1.42) | 3.19× (2.92–3.49) | 3.90× (3.76–4.06) | 4.32× (3.96–4.73) | 4.11× (3.94–4.29) | 6.62× (6.30–6.96) | 6.32× (6.05–6.60) | 2.02× (1.86–2.20) | 2.34× (2.18–2.52) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 0.031 s | 438 | 31.6 | 1.35× (1.18–1.54) | 2.41× (2.03–2.87) | 3.39× (2.85–4.03) | 3.13× (2.73–3.58) | 3.82× (3.27–4.47) | 7.14× (6.10–8.35) | 6.81× (5.87–7.91) | 2.18× (1.99–2.39) | 2.53× (2.12–3.02) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.69× (2.63–2.75) | 6.84× (6.77–6.92) | 7.16× (7.06–7.26) | 8.17× (8.02–8.32) | 7.26× (7.17–7.36) | 7.39× (7.30–7.49) | 7.04× (6.94–7.14) | 2.14× (2.09–2.19) | 2.39× (2.35–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.52× (2.47–2.58) | 6.29× (6.05–6.53) | 6.71× (6.58–6.86) | 7.61× (7.46–7.76) | 6.41× (6.28–6.54) | 6.63× (6.53–6.74) | 6.31× (6.19–6.44) | 1.92× (1.85–1.98) | 2.15× (2.10–2.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.85× (2.71–2.99) | 4.89× (4.76–5.03) | 5.53× (5.35–5.72) | 5.84× (5.75–5.94) | 5.16× (5.06–5.27) | 4.25× (4.14–4.36) | 4.04× (3.94–4.15) | 1.23× (1.19–1.27) | 1.37× (1.34–1.41) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.64) | 3.00× (2.94–3.07) | 3.92× (3.84–3.99) | 3.48× (3.26–3.71) | 5.93× (5.76–6.11) | 2.61× (2.52–2.70) | 2.57× (2.50–2.63) | 1.00× (0.97–1.02) | 1.80× (1.77–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.53–0.58) | 2.78× (2.65–2.92) | 4.85× (4.66–5.04) | 3.40× (3.21–3.60) | 5.93× (5.14–6.85) | 2.46× (2.34–2.60) | 2.43× (2.33–2.53) | 0.94× (0.91–0.98) | 1.70× (1.63–1.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 3.02× (2.94–3.10) | 5.02× (4.93–5.12) | 3.36× (3.29–3.44) | 5.82× (5.54–6.11) | 2.54× (2.45–2.62) | 2.50× (2.45–2.54) | 0.97× (0.95–0.99) | 1.75× (1.71–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.60–2.78) | 3.12× (3.03–3.21) | 3.74× (3.64–3.84) | 4.25× (4.16–4.34) | 5.03× (4.87–5.18) | 2.82× (2.70–2.95) | 2.39× (2.32–2.46) | 2.03× (1.96–2.10) | 2.71× (2.65–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.16× (1.92–2.44) | 2.43× (2.33–2.53) | 3.08× (2.74–3.46) | 3.20× (3.10–3.30) | 3.72× (3.59–3.85) | 2.01× (1.93–2.09) | 1.70× (1.64–1.76) | 1.44× (1.39–1.50) | 1.93× (1.86–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.88× (1.78–1.99) | 1.60× (1.50–1.70) | 2.08× (1.81–2.39) | 1.91× (1.82–2.00) | 2.16× (2.04–2.29) | 1.01× (0.96–1.08) | 0.86× (0.82–0.90) | 0.73× (0.69–0.77) | 0.97× (0.93–1.02) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.33–1.37) | 2.94× (2.83–3.06) | 2.61× (2.54–2.69) | 2.63× (2.55–2.71) | 2.98× (2.91–3.05) | 3.32× (3.24–3.40) | 3.37× (3.31–3.44) | 1.31× (1.29–1.34) | 1.42× (1.40–1.45) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.24–1.30) | 2.70× (2.56–2.85) | 2.36× (2.24–2.49) | 3.02× (2.82–3.23) | 2.62× (2.57–2.68) | 2.90× (2.84–2.97) | 2.96× (2.89–3.03) | 1.15× (1.13–1.17) | 1.25× (1.21–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.29× (1.12–1.47) | 2.44× (2.06–2.88) | 1.99× (1.67–2.37) | 2.53× (2.19–2.92) | 2.49× (2.17–2.85) | 2.04× (1.75–2.37) | 2.08× (1.78–2.42) | 0.81× (0.69–0.94) | 0.87× (0.76–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.39× (1.26–1.53) | 3.44× (3.17–3.74) | 2.85× (2.65–3.06) | 2.81× (2.59–3.05) | 3.30× (3.00–3.63) | 4.13× (3.78–4.52) | 4.37× (4.02–4.76) | 1.48× (1.36–1.61) | 1.64× (1.52–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.21–1.39) | 3.08× (2.86–3.31) | 2.45× (2.26–2.65) | 3.33× (3.10–3.58) | 2.86× (2.63–3.11) | 3.59× (3.31–3.90) | 3.80× (3.53–4.10) | 1.29× (1.20–1.39) | 1.43× (1.32–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.50–1.58) | 2.91× (2.76–3.07) | 2.16× (2.10–2.23) | 2.91× (2.84–2.99) | 2.73× (2.66–2.81) | 2.61× (2.55–2.68) | 2.76× (2.71–2.82) | 0.94× (0.92–0.95) | 1.04× (1.02–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.60× (0.48–0.77) | 2.30× (1.82–2.90) | 3.06× (2.42–3.87) | 2.81× (2.37–3.34) | 5.65× (4.47–7.13) | 1.90× (1.51–2.40) | 1.94× (1.54–2.45) | 0.87× (0.69–1.11) | 1.33× (1.05–1.69) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.56–0.67) | 2.33× (2.27–2.39) | 4.19× (3.70–4.74) | 2.72× (2.62–2.82) | 5.71× (5.54–5.89) | 1.96× (1.89–2.04) | 2.00× (1.95–2.06) | 0.90× (0.88–0.92) | 1.37× (1.33–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.53–0.67) | 2.49× (2.44–2.54) | 4.05× (3.97–4.13) | 2.82× (2.77–2.87) | 5.96× (5.80–6.13) | 2.02× (1.97–2.08) | 2.07× (2.02–2.12) | 0.93× (0.90–0.96) | 1.42× (1.39–1.44) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 1.69× (1.67–1.72) | 4.06× (3.89–4.23) | 4.40× (4.34–4.45) | 4.80× (4.56–5.07) | 4.78× (4.71–4.85) | 4.86× (4.76–4.96) | 4.63× (4.55–4.72) | 1.50× (1.42–1.59) | 1.65× (1.54–1.78) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 1.44× (1.40–1.47) | 3.30× (3.05–3.57) | 3.51× (3.43–3.59) | 4.19× (3.88–4.53) | 3.72× (3.59–3.86) | 3.69× (3.58–3.80) | 3.52× (3.42–3.61) | 1.14× (1.06–1.23) | 1.26× (1.17–1.35) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 1.40× (1.18–1.67) | 2.59× (2.16–3.11) | 2.94× (2.43–3.54) | 3.03× (2.63–3.50) | 3.08× (2.57–3.69) | 2.26× (1.91–2.68) | 2.15× (1.84–2.53) | 0.70× (0.63–0.78) | 0.77× (0.64–0.92) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.31–0.35) | 0.98× (0.98–0.98) | 0.79× (0.77–0.80) | 11.06× (11.04–11.08) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.41× (0.35–0.47) | 0.68× (0.66–0.71) | 0.58× (0.56–0.60) | 5.47× (5.25–5.70) | 0.13× (0.12–0.13) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.49× (0.42–0.57) | 0.60× (0.57–0.63) | 0.52× (0.50–0.54) | 3.47× (3.35–3.60) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.19) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.46× (0.44–0.47) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.02–8.07) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.71) | 0.62× (0.55–0.69) | 1.58× (1.54–1.63) | 0.80× (0.77–0.82) | 5.75× (5.59–5.91) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.70× (0.66–0.74) | 1.14× (1.10–1.19) | 0.71× (0.67–0.74) | 3.34× (3.22–3.46) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.40× (0.38–0.43) | 0.89× (0.86–0.92) | 0.71× (0.69–0.74) | 3.07× (2.97–3.17) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.82× (0.81–0.85) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.82× (0.82–0.83) | 5.23× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.39× (0.37–0.40) | 0.84× (0.81–0.87) | 0.73× (0.70–0.75) | 2.88× (2.79–2.98) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.45× (0.43–0.48) | 0.80× (0.79–0.81) | 0.82× (0.80–0.84) | 2.30× (2.27–2.33) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.24) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.77× (0.76–0.77) | 5.28× (5.26–5.30) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.67× (0.66–0.67) | 6.40× (6.38–6.42) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.36–0.42) | 1.01× (0.97–1.04) | 0.66× (0.63–0.70) | 3.24× (3.16–3.31) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 59 MB | 0.47× (0.46–0.49) | 0.43× (0.40–0.47) | 0.88× (0.84–0.92) | 0.72× (0.70–0.73) | 2.21× (2.17–2.26) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 650 | 1.55 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,079 | 1.73 | 1.11× (1.10–1.13) | 81% | 1.02× (0.99–1.06) | 1.04× (1.01–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,270 | 2.71 | 1.74× (1.71–1.78) | 49% | 1.24× (1.21–1.28) | 1.84× (1.75–1.94) |

**Best:** mibz2 at 4 threads (1,270 MB/s); lbzip2 at 4 threads (297 MB/s); pbz2 at 4 threads (436 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.23, lbzip2 1t 0.08, lbzip2 2t 0.27, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.42 | 0.01 | 0.07 | 0.165 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4306 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 14.5% |
| MTF + RLE2 | 1.47 | 20.9% |
| T-vector build | 1.29 | 18.3% |
| walk (the one each build uses) | 2.88 | 41.0% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.6% |
| **total** | 7.02 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.297 | 0.715 | (in its total) | 2.44× (2.39–2.48) |  |
| walk | 0.632 | 1.623 | (in its total) | 2.58× (2.52–2.63) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.030 | (in its total) | 1.13× (1.07–1.18) |  |
| inverse BWT, total | 0.954 | 2.369 | 1.312 | 2.49× (2.44–2.54) | 1.39× (1.36–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 47m10s ± 5m24s |
| 4 threads | 23m53s ± 3m22s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m19s ± 2m11s |
| the scanner alone (its serial part) | 8m44s ± 29.4s |
| the workers alone, over 4 | 15m19s ± 2m10s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.01 ns a byte of output at 4 threads (CPU, summed over them), 1.26 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,698,958 | 13,676,554 | \`b30556180238088f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4305.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
