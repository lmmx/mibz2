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

The latest record: [`3a9a654`](<https://github.com/lmmx/mibz2/commit/3a9a654ff5fe48b540703e8a41f8878d988147a0>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43793028608>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.410 s | 512 | 5.8 | 3.40× (3.37–3.42) | 7.75× (7.68–7.82) | 8.29× (8.22–8.36) | 9.14× (9.07–9.21) | 8.30× (8.24–8.36) | 8.65× (8.58–8.73) | 8.43× (8.36–8.50) | 2.69× (2.67–2.71) | 2.36× (2.34–2.38) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.235 s | 890 | 6.2 | 3.22× (3.12–3.32) | 6.98× (6.81–7.15) | 8.51× (8.24–8.79) | 8.40× (8.18–8.63) | 7.59× (7.39–7.80) | 15.05× (14.65–15.46) | 14.65× (14.27–15.05) | 4.68× (4.56–4.80) | 4.10× (3.99–4.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.195 s | 1,078 | 9.8 | 3.32× (3.27–3.37) | 6.12× (6.02–6.21) | 7.30× (7.17–7.44) | 7.15× (7.03–7.28) | 6.95× (6.86–7.05) | 18.25× (17.99–18.52) | 17.77× (17.50–18.04) | 5.67× (5.59–5.75) | 4.98× (4.91–5.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,212 | 2.7 | 0.63× (0.62–0.65) | 3.11× (3.05–3.18) | 4.43× (4.33–4.52) | 3.81× (3.74–3.88) | 6.38× (6.28–6.49) | 2.82× (2.76–2.87) | 2.85× (2.79–2.91) | 1.13× (1.11–1.16) | 1.26× (1.23–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,205 | 2.7 | 0.55× (0.54–0.56) | 3.08× (3.02–3.13) | 6.20× (6.09–6.31) | 3.77× (3.71–3.84) | 6.25× (6.13–6.37) | 2.78× (2.73–2.84) | 2.82× (2.76–2.88) | 1.12× (1.09–1.15) | 1.24× (1.22–1.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,198 | 2.7 | 0.55× (0.54–0.56) | 3.06× (2.99–3.13) | 6.11× (5.95–6.28) | 3.76× (3.69–3.84) | 6.24× (6.13–6.36) | 2.77× (2.70–2.83) | 2.80× (2.74–2.86) | 1.11× (1.08–1.14) | 1.24× (1.21–1.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 623 | 4.9 | 2.58× (2.54–2.63) | 3.23× (3.20–3.25) | 3.91× (3.88–3.94) | 4.24× (4.21–4.27) | 4.62× (4.59–4.65) | 3.11× (3.09–3.14) | 2.87× (2.84–2.90) | 2.03× (2.01–2.05) | 2.13× (2.01–2.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 987 | 5.9 | 2.32× (2.11–2.54) | 2.99× (2.81–3.18) | 4.24× (3.93–4.56) | 3.76× (3.47–4.06) | 5.03× (4.74–5.35) | 4.78× (4.49–5.08) | 4.41× (4.15–4.68) | 3.12× (2.93–3.31) | 3.26× (3.03–3.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 978 | 10.7 | 1.96× (1.85–2.07) | 2.14× (2.02–2.26) | 3.12× (2.99–3.27) | 2.42× (2.36–2.49) | 4.24× (4.11–4.38) | 4.89× (4.76–5.02) | 4.51× (4.38–4.65) | 3.19× (3.09–3.29) | 3.34× (3.17–3.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.116 s | 69 | 43.0 | 1.37× (1.36–1.37) | 3.07× (3.05–3.09) | 2.79× (2.77–2.81) | 2.85× (2.83–2.87) | 3.11× (3.10–3.13) | 3.45× (3.44–3.47) | 3.55× (3.54–3.57) | 1.26× (1.25–1.26) | 1.24× (1.22–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 113 | 46.7 | 1.30× (1.26–1.34) | 2.89× (2.81–2.97) | 2.78× (2.71–2.84) | 3.38× (3.30–3.46) | 3.15× (3.07–3.23) | 5.71× (5.57–5.85) | 5.88× (5.73–6.03) | 2.08× (2.03–2.13) | 2.04× (1.98–2.11) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 64.0 | 1.39× (1.37–1.42) | 2.74× (2.67–2.82) | 2.69× (2.58–2.81) | 3.20× (3.13–3.28) | 3.25× (3.18–3.32) | 7.02× (6.87–7.16) | 7.23× (7.08–7.37) | 2.56× (2.50–2.61) | 2.51× (2.44–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.128 s | 75 | 39.6 | 1.42× (1.41–1.43) | 3.43× (3.40–3.46) | 2.95× (2.92–2.98) | 3.03× (3.01–3.05) | 3.34× (3.33–3.36) | 4.11× (4.08–4.14) | 4.40× (4.37–4.43) | 1.47× (1.46–1.47) | 1.50× (1.49–1.51) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.077 s | 125 | 43.3 | 1.25× (1.18–1.32) | 3.00× (2.86–3.14) | 2.82× (2.69–2.96) | 3.67× (3.49–3.86) | 3.16× (3.01–3.32) | 6.66× (6.31–7.03) | 7.13× (6.76–7.52) | 2.38× (2.26–2.50) | 2.44× (2.31–2.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 153 | 61.0 | 1.43× (1.40–1.47) | 2.98× (2.89–3.09) | 2.83× (2.76–2.90) | 3.55× (3.44–3.66) | 3.23× (3.15–3.30) | 8.27× (8.07–8.47) | 8.85× (8.64–9.06) | 2.95× (2.88–3.02) | 3.02× (2.96–3.09) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 905 | 3.8 | 0.68× (0.66–0.70) | 2.87× (2.69–3.06) | 3.92× (3.76–4.09) | 3.24× (3.19–3.29) | 6.81× (6.69–6.94) | 2.42× (2.39–2.46) | 2.47× (2.44–2.50) | 1.03× (1.01–1.05) | 0.96× (0.94–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 862 | 4.0 | 0.53× (0.51–0.54) | 2.69× (2.61–2.76) | 5.46× (5.30–5.61) | 3.11× (3.04–3.18) | 6.64× (6.50–6.79) | 2.33× (2.28–2.38) | 2.37× (2.31–2.43) | 0.99× (0.96–1.02) | 0.93× (0.89–0.97) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 877 | 3.8 | 0.53× (0.52–0.54) | 2.69× (2.64–2.75) | 5.36× (5.14–5.59) | 3.15× (3.09–3.21) | 6.55× (6.42–6.69) | 2.33× (2.29–2.37) | 2.37× (2.33–2.41) | 0.99× (0.97–1.01) | 0.93× (0.91–0.94) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.063 s | 212 | 14.2 | 1.82× (1.78–1.87) | 4.22× (4.09–4.36) | 4.67× (4.57–4.77) | 4.76× (4.63–4.89) | 4.88× (4.74–5.02) | 5.06× (4.93–5.19) | 4.99× (4.87–5.11) | 1.49× (1.45–1.53) | 1.43× (1.40–1.47) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.041 s | 330 | 16.8 | 1.50× (1.34–1.67) | 3.46× (3.10–3.87) | 4.34× (3.88–4.86) | 4.40× (3.96–4.89) | 4.40× (3.93–4.92) | 7.42× (6.62–8.33) | 7.32× (6.52–8.21) | 2.19× (1.95–2.45) | 2.10× (1.87–2.36) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.035 s | 387 | 25.2 | 1.64× (1.60–1.68) | 3.22× (3.14–3.30) | 3.95× (3.77–4.14) | 4.07× (4.00–4.15) | 4.48× (4.39–4.58) | 9.20× (8.99–9.41) | 9.07× (8.88–9.25) | 2.71× (2.64–2.77) | 2.61× (2.55–2.67) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.38× (3.36–3.41) | 7.71× (7.64–7.78) | 8.25× (8.18–8.31) | 9.10× (9.03–9.18) | 8.27× (8.21–8.33) | 8.62× (8.55–8.69) | 8.39× (8.33–8.46) | 2.68× (2.67–2.70) | 2.34× (2.32–2.36) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.18× (3.15–3.21) | 7.31× (7.21–7.41) | 8.10× (8.04–8.17) | 8.82× (8.74–8.91) | 7.64× (7.56–7.71) | 7.95× (7.88–8.02) | 7.74× (7.66–7.82) | 2.47× (2.45–2.49) | 2.16× (2.14–2.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.26× (3.22–3.30) | 6.65× (6.56–6.75) | 7.18× (7.10–7.27) | 7.81× (7.72–7.91) | 6.94× (6.84–7.03) | 5.07× (5.00–5.15) | 4.94× (4.87–5.02) | 1.58× (1.56–1.60) | 1.38× (1.36–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.65–0.68) | 2.86× (2.81–2.91) | 3.87× (3.81–3.93) | 3.55× (3.49–3.61) | 5.88× (5.80–5.97) | 2.65× (2.61–2.69) | 2.68× (2.63–2.73) | 1.12× (1.10–1.15) | 0.81× (0.80–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.81× (2.77–2.85) | 5.42× (5.33–5.51) | 3.46× (3.41–3.50) | 5.65× (5.57–5.73) | 2.56× (2.52–2.60) | 2.59× (2.55–2.63) | 1.09× (1.05–1.12) | 0.78× (0.77–0.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.23× (3.14–3.34) | 5.47× (5.37–5.57) | 3.56× (3.50–3.61) | 5.80× (5.71–5.89) | 2.62× (2.57–2.68) | 2.66× (2.61–2.70) | 1.11× (1.08–1.14) | 0.80× (0.79–0.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.46–2.54) | 3.01× (2.98–3.03) | 3.53× (3.50–3.56) | 4.08× (4.04–4.11) | 4.44× (4.40–4.47) | 3.01× (2.98–3.04) | 2.78× (2.75–2.80) | 1.98× (1.96–2.00) | 1.90× (1.86–1.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.33× (2.29–2.37) | 2.72× (2.67–2.76) | 3.32× (3.26–3.38) | 3.54× (3.48–3.60) | 3.88× (3.83–3.94) | 2.54× (2.49–2.59) | 2.34× (2.30–2.38) | 1.67× (1.64–1.70) | 1.60× (1.57–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.99–2.09) | 1.82× (1.77–1.87) | 2.28× (2.24–2.32) | 2.25× (2.20–2.30) | 2.35× (2.30–2.40) | 1.41× (1.38–1.44) | 1.30× (1.27–1.33) | 0.93× (0.90–0.95) | 0.89× (0.87–0.91) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.35–1.37) | 2.99× (2.97–3.00) | 2.73× (2.71–2.74) | 2.83× (2.80–2.85) | 3.08× (3.06–3.10) | 3.42× (3.40–3.43) | 3.52× (3.50–3.54) | 1.25× (1.25–1.26) | 1.22× (1.21–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.31) | 2.85× (2.80–2.90) | 2.58× (2.53–2.64) | 3.24× (3.18–3.31) | 2.85× (2.79–2.91) | 3.11× (3.05–3.17) | 3.20× (3.14–3.27) | 1.14× (1.12–1.16) | 1.11× (1.09–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.50–1.56) | 2.90× (2.86–2.93) | 2.47× (2.40–2.54) | 3.16× (3.12–3.21) | 2.88× (2.84–2.92) | 2.28× (2.25–2.32) | 2.35× (2.32–2.38) | 0.84× (0.83–0.85) | 0.81× (0.80–0.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.41–1.42) | 3.37× (3.34–3.39) | 2.88× (2.85–2.91) | 3.00× (2.99–3.02) | 3.32× (3.30–3.33) | 4.07× (4.04–4.10) | 4.36× (4.33–4.38) | 1.46× (1.46–1.47) | 1.45× (1.45–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.32–1.36) | 3.17× (3.12–3.22) | 2.73× (2.69–2.76) | 3.62× (3.58–3.66) | 3.06× (3.02–3.10) | 3.71× (3.66–3.76) | 3.97× (3.92–4.02) | 1.33× (1.31–1.35) | 1.32× (1.31–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.53–1.66) | 3.28× (3.22–3.35) | 2.56× (2.49–2.63) | 3.40× (3.33–3.47) | 2.99× (2.94–3.04) | 2.65× (2.60–2.70) | 2.84× (2.79–2.89) | 0.95× (0.94–0.97) | 0.95× (0.93–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 2.46× (2.42–2.50) | 3.42× (3.38–3.46) | 2.93× (2.88–2.98) | 6.02× (5.91–6.14) | 2.22× (2.19–2.26) | 2.26× (2.23–2.30) | 1.02× (1.00–1.04) | 0.75× (0.73–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.26× (2.23–2.30) | 4.39× (4.31–4.47) | 2.82× (2.78–2.86) | 5.79× (5.68–5.90) | 2.10× (2.06–2.13) | 2.14× (2.10–2.17) | 0.96× (0.94–0.99) | 0.71× (0.69–0.72) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.61× (2.49–2.73) | 4.44× (4.37–4.52) | 2.97× (2.92–3.01) | 5.94× (5.85–6.04) | 2.18× (2.15–2.21) | 2.22× (2.19–2.25) | 1.00× (0.99–1.01) | 0.74× (0.72–0.75) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.80× (1.76–1.85) | 4.05× (3.95–4.17) | 4.52× (4.42–4.63) | 4.66× (4.54–4.79) | 4.78× (4.65–4.91) | 4.95× (4.83–5.08) | 4.88× (4.77–5.00) | 1.48× (1.44–1.51) | 1.36× (1.32–1.40) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.61× (1.59–1.64) | 3.60× (3.54–3.66) | 4.24× (4.18–4.30) | 4.47× (4.42–4.52) | 4.15× (4.09–4.21) | 4.23× (4.16–4.31) | 4.17× (4.10–4.25) | 1.26× (1.24–1.28) | 1.16× (1.14–1.18) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.78× (1.72–1.83) | 3.24× (3.16–3.33) | 3.60× (3.50–3.70) | 3.94× (3.86–4.01) | 3.61× (3.55–3.68) | 2.80× (2.74–2.87) | 2.76× (2.71–2.82) | 0.84× (0.82–0.85) | 0.77× (0.75–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.27–0.30) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.03× (11.01–11.06) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.36) | 0.50× (0.42–0.59) | 0.71× (0.68–0.74) | 0.60× (0.57–0.62) | 5.64× (5.44–5.85) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.32–0.35) | 0.47× (0.43–0.52) | 0.59× (0.57–0.62) | 0.53× (0.51–0.55) | 3.49× (3.37–3.62) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.35–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.38× (4.37–4.40) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.01× (7.99–8.04) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.67–0.70) | 0.63× (0.59–0.68) | 1.64× (1.61–1.68) | 0.82× (0.80–0.83) | 5.94× (5.81–6.08) | 0.48× (0.47–0.49) | 0.51× (0.49–0.52) | 0.55× (0.54–0.57) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.67× (0.63–0.72) | 1.14× (1.10–1.17) | 0.71× (0.69–0.73) | 3.37× (3.29–3.46) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.41× (0.39–0.44) | 0.91× (0.88–0.94) | 0.69× (0.66–0.72) | 3.14× (3.04–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.65) | 0.45× (0.44–0.47) | 0.86× (0.83–0.89) | 0.80× (0.76–0.83) | 2.50× (2.42–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.22× (5.21–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.38× (0.34–0.41) | 0.88× (0.85–0.92) | 0.73× (0.69–0.76) | 3.00× (2.88–3.12) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.62× (0.61–0.64) | 0.51× (0.48–0.55) | 0.83× (0.81–0.84) | 0.86× (0.84–0.88) | 2.38× (2.34–2.42) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.77× (0.77–0.77) | 5.37× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.26–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.29–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.42× (1.41–1.42) | 0.68× (0.67–0.68) | 6.42× (6.41–6.44) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.35–0.44) | 1.06× (1.03–1.09) | 0.68× (0.66–0.69) | 3.27× (3.21–3.34) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 58 MB | 0.48× (0.47–0.50) | 0.49× (0.44–0.54) | 0.89× (0.88–0.90) | 0.74× (0.72–0.76) | 2.26× (2.22–2.31) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 512 | 1.96 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 890 | 2.12 | 1.08× (1.08–1.09) | 87% | 1.03× (1.02–1.03) | 1.02× (1.02–1.02) |
| 4: all 4 CPUs, sharing SMT siblings | 1,078 | 3.36 | 1.70× (1.68–1.72) | 53% | 1.47× (1.46–1.47) | 1.63× (1.62–1.65) |

**Best:** mibz2 at 4 threads (1,078 MB/s); lbzip2 at 4 threads (176 MB/s); pbz2 at 4 threads (325 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.21, lbzip2 1t 0.04, lbzip2 2t 0.20, lbzip2 4t 0.26, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.53 | 0.02 | 0.08 | 0.195 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2929 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 58.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.87 | 10.4% |
| MTF + RLE2 | 1.26 | 15.1% |
| T-vector build | 1.16 | 13.9% |
| walk (the one each build uses) | 4.78 | 57.3% |
| of which: segment assembly | 0.24 | 2.8% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.10 | 1.3% |
| **total** | 8.35 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.943 | (in its total) | 2.53× (2.52–2.53) |  |
| walk | 1.541 | 2.972 | (in its total) | 1.93× (1.93–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.074 | (in its total) | 0.91× (0.91–0.91) |  |
| inverse BWT, total | 1.994 | 3.989 | 2.038 | 2.00× (2.00–2.00) | 1.02× (1.02–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m45s ± 6m52s |
| 4 threads | 28m20s ± 4m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m10s ± 2m44s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 19m10s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.51 ns a byte of output at 4 threads (CPU, summed over them), 1.60 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,079 | 13,357,122 | \`cc146a11acfb9941…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 2929.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
