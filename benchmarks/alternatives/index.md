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

The latest record: [`c7a8380`](<https://github.com/lmmx/mibz2/commit/c7a838030f1628aba7d987697ea09d3417218c91>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43656201372>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.335 s | 626 | 7.2 | 2.66× (2.60–2.72) | 6.77× (6.63–6.91) | 7.02× (6.86–7.18) | 8.10× (7.80–8.42) | 7.09× (6.88–7.31) | 7.45× (7.15–7.76) | 7.04× (6.80–7.29) | 2.07× (2.03–2.11) | 2.43× (2.38–2.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.203 s | 1,036 | 8.4 | 2.73× (2.59–2.87) | 5.97× (5.65–6.31) | 7.35× (6.86–7.88) | 6.95× (6.65–7.27) | 6.20× (5.96–6.45) | 12.00× (11.38–12.65) | 11.35× (10.80–11.93) | 3.34× (3.20–3.48) | 3.92× (3.75–4.09) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.186 s | 1,128 | 13.8 | 2.73× (2.58–2.90) | 4.28× (4.03–4.53) | 5.98× (5.56–6.43) | 5.11× (4.75–5.50) | 5.16× (4.85–5.49) | 13.39× (12.73–14.08) | 12.66× (12.05–13.30) | 3.72× (3.59–3.86) | 4.37× (4.20–4.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,356 | 3.7 | 0.57× (0.56–0.58) | 3.55× (3.43–3.67) | 4.54× (4.35–4.73) | 3.75× (3.65–3.86) | 6.44× (6.29–6.58) | 2.89× (2.65–3.16) | 2.87× (2.61–3.16) | 1.01× (0.97–1.04) | 2.09× (2.05–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,290 | 4.0 | 0.54× (0.47–0.62) | 2.79× (2.60–2.99) | 5.75× (5.41–6.10) | 3.56× (3.44–3.67) | 6.23× (5.95–6.53) | 2.76× (2.55–2.99) | 2.75× (2.52–2.99) | 0.96× (0.92–1.01) | 2.00× (1.92–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,266 | 3.9 | 0.50× (0.48–0.52) | 3.07× (2.84–3.32) | 5.70× (5.40–6.01) | 3.50× (3.40–3.60) | 6.18× (6.00–6.36) | 2.71× (2.51–2.91) | 2.69× (2.46–2.94) | 0.94× (0.89–1.00) | 1.96× (1.89–2.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 980 | 5.0 | 2.83× (2.64–3.05) | 3.44× (3.19–3.71) | 4.12× (3.93–4.31) | 4.46× (4.23–4.69) | 5.12× (4.88–5.38) | 3.07× (2.73–3.45) | 2.53× (2.41–2.67) | 2.08× (2.00–2.18) | 3.08× (2.93–3.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,264 | 7.2 | 1.98× (1.78–2.21) | 2.74× (2.27–3.32) | 3.88× (3.41–4.40) | 3.30× (2.92–3.73) | 4.93× (4.33–5.62) | 3.78× (3.30–4.34) | 3.12× (2.74–3.56) | 2.57× (2.25–2.93) | 3.80× (3.32–4.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,074 | 13.3 | 1.73× (1.61–1.85) | 1.83× (1.71–1.96) | 2.92× (2.70–3.16) | 2.16× (1.91–2.45) | 4.28× (3.86–4.75) | 3.44× (3.05–3.88) | 2.84× (2.64–3.06) | 2.34× (2.18–2.51) | 3.45× (3.21–3.71) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.088 s | 91 | 50.2 | 1.36× (1.28–1.45) | 3.00× (2.80–3.22) | 2.66× (2.49–2.85) | 2.54× (2.37–2.71) | 2.95× (2.70–3.21) | 3.29× (3.05–3.55) | 3.40× (3.16–3.65) | 1.29× (1.21–1.38) | 1.45× (1.34–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.058 s | 139 | 59.2 | 1.22× (1.17–1.26) | 2.62× (2.51–2.74) | 2.64× (2.30–3.04) | 3.15× (2.93–3.39) | 2.89× (2.76–3.02) | 4.97× (4.75–5.20) | 5.14× (4.97–5.31) | 1.95× (1.87–2.04) | 2.20× (2.07–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.047 s | 170 | 77.0 | 1.28× (1.22–1.35) | 2.37× (2.25–2.48) | 2.63× (2.32–2.99) | 2.92× (2.63–3.24) | 3.24× (3.02–3.47) | 6.22× (5.98–6.48) | 6.43× (6.22–6.65) | 2.44× (2.37–2.52) | 2.75× (2.65–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.093 s | 104 | 44.1 | 1.44× (1.39–1.50) | 3.57× (3.43–3.72) | 2.98× (2.88–3.09) | 2.91× (2.81–3.01) | 3.50× (3.22–3.80) | 4.19× (4.07–4.31) | 4.65× (4.48–4.83) | 1.57× (1.49–1.65) | 1.74× (1.68–1.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.061 s | 156 | 52.5 | 1.27× (1.20–1.35) | 3.11× (2.92–3.31) | 2.61× (2.43–2.81) | 3.37× (3.14–3.62) | 3.04× (2.84–3.26) | 6.19× (5.76–6.66) | 6.88× (6.34–7.46) | 2.32× (2.20–2.45) | 2.56× (2.37–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.050 s | 191 | 73.5 | 1.37× (1.33–1.42) | 2.69× (2.47–2.92) | 2.58× (2.47–2.70) | 3.20× (3.07–3.32) | 3.06× (2.99–3.14) | 7.70× (7.46–7.96) | 8.56× (8.21–8.92) | 2.89× (2.74–3.04) | 3.19× (3.10–3.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,036 | 5.1 | 0.62× (0.60–0.63) | 3.25× (3.01–3.51) | 4.36× (4.22–4.51) | 3.16× (3.08–3.24) | 7.61× (6.60–8.77) | 2.33× (2.28–2.39) | 2.36× (2.30–2.43) | 0.98× (0.96–1.01) | 2.11× (2.05–2.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 966 | 5.6 | 0.51× (0.50–0.53) | 2.97× (2.90–3.05) | 5.08× (4.94–5.21) | 3.03× (2.95–3.12) | 6.87× (6.54–7.22) | 2.19× (2.14–2.23) | 2.22× (2.16–2.28) | 0.92× (0.90–0.95) | 1.98× (1.93–2.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 939 | 5.5 | 0.51× (0.49–0.52) | 2.97× (2.89–3.06) | 5.16× (4.97–5.37) | 2.99× (2.89–3.11) | 6.66× (6.47–6.86) | 2.18× (2.12–2.24) | 2.21× (2.14–2.28) | 0.92× (0.89–0.95) | 1.98× (1.92–2.03) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.044 s | 281 | 16.5 | 1.82× (1.67–1.97) | 4.31× (4.14–4.49) | 4.62× (4.45–4.78) | 4.65× (4.54–4.76) | 4.91× (4.80–5.01) | 4.87× (4.76–4.99) | 4.89× (4.76–5.03) | 1.54× (1.46–1.62) | 1.74× (1.65–1.83) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.034 s | 364 | 22.5 | 1.26× (0.98–1.62) | 3.07× (2.59–3.63) | 3.62× (3.24–4.04) | 4.15× (3.43–5.03) | 3.84× (3.25–4.53) | 5.87× (5.04–6.83) | 5.89× (4.99–6.95) | 1.85× (1.54–2.23) | 2.09× (1.76–2.48) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.032 s | 383 | 33.9 | 1.25× (1.20–1.30) | 2.48× (2.16–2.85) | 3.20× (2.87–3.57) | 3.33× (3.04–3.66) | 3.71× (3.36–4.09) | 6.66× (6.39–6.95) | 6.69× (6.35–7.05) | 2.10× (1.95–2.27) | 2.37× (2.28–2.47) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.65× (2.59–2.71) | 6.72× (6.59–6.86) | 6.97× (6.81–7.13) | 8.06× (7.76–8.38) | 7.06× (6.85–7.27) | 7.41× (7.11–7.72) | 7.01× (6.77–7.25) | 2.07× (2.02–2.11) | 2.41× (2.36–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.51× (2.44–2.58) | 6.06× (5.94–6.19) | 6.80× (6.41–7.22) | 7.28× (7.00–7.57) | 6.13× (5.97–6.29) | 6.35× (6.09–6.62) | 6.00× (5.80–6.21) | 1.77× (1.72–1.82) | 2.06× (1.99–2.14) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.61× (2.47–2.75) | 4.67× (4.50–4.84) | 5.65× (5.20–6.13) | 5.71× (5.37–6.08) | 5.09× (4.75–5.44) | 3.85× (3.68–4.03) | 3.64× (3.49–3.80) | 1.07× (1.03–1.12) | 1.25× (1.20–1.31) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.63) | 3.06× (3.00–3.13) | 3.90× (3.85–3.95) | 3.47× (3.35–3.60) | 5.86× (5.74–5.99) | 2.71× (2.48–2.96) | 2.71× (2.45–3.00) | 1.00× (0.97–1.04) | 1.80× (1.76–1.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.53–0.67) | 2.70× (2.60–2.80) | 4.90× (4.71–5.11) | 3.26× (3.16–3.36) | 5.60× (5.35–5.87) | 2.55× (2.34–2.77) | 2.55× (2.32–2.80) | 0.94× (0.90–0.99) | 1.69× (1.63–1.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.57) | 2.97× (2.87–3.08) | 4.92× (4.75–5.10) | 3.30× (3.21–3.38) | 5.69× (5.53–5.85) | 2.57× (2.38–2.77) | 2.57× (2.33–2.83) | 0.95× (0.90–1.01) | 1.71× (1.63–1.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.50–2.92) | 2.99× (2.84–3.16) | 3.67× (3.52–3.83) | 4.19× (3.98–4.40) | 4.81× (4.59–5.03) | 2.91× (2.60–3.26) | 2.42× (2.30–2.54) | 2.00× (1.92–2.09) | 2.71× (2.60–2.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.02× (1.96–2.09) | 2.40× (2.29–2.52) | 2.89× (2.82–2.97) | 3.15× (3.06–3.24) | 3.56× (3.44–3.69) | 2.05× (1.87–2.26) | 1.71× (1.63–1.78) | 1.41× (1.36–1.47) | 1.91× (1.86–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.85–2.03) | 1.69× (1.55–1.84) | 2.02× (1.88–2.16) | 2.08× (1.89–2.29) | 2.25× (2.03–2.49) | 1.08× (0.95–1.24) | 0.90× (0.83–0.97) | 0.74× (0.69–0.81) | 1.01× (0.93–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.27–1.44) | 2.94× (2.74–3.15) | 2.57× (2.39–2.75) | 2.51× (2.34–2.68) | 2.91× (2.67–3.17) | 3.24× (3.01–3.50) | 3.35× (3.12–3.60) | 1.29× (1.21–1.37) | 1.38× (1.27–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.21–1.26) | 2.65× (2.58–2.72) | 2.46× (2.14–2.84) | 2.88× (2.80–2.96) | 2.60× (2.54–2.66) | 2.77× (2.69–2.85) | 2.86× (2.82–2.91) | 1.10× (1.08–1.12) | 1.18× (1.14–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.39–1.53) | 2.66× (2.54–2.79) | 2.39× (2.05–2.79) | 2.96× (2.72–3.22) | 2.97× (2.67–3.30) | 2.14× (2.09–2.20) | 2.22× (2.15–2.29) | 0.85× (0.82–0.88) | 0.92× (0.87–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.38–1.49) | 3.47× (3.33–3.62) | 2.91× (2.79–3.03) | 2.87× (2.78–2.97) | 3.45× (3.18–3.75) | 4.13× (4.01–4.25) | 4.59× (4.42–4.76) | 1.56× (1.49–1.64) | 1.67× (1.62–1.73) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.33–1.41) | 3.23× (3.16–3.31) | 2.48× (2.39–2.57) | 3.32× (3.20–3.45) | 2.84× (2.76–2.93) | 3.46× (3.34–3.57) | 3.84× (3.71–3.97) | 1.30× (1.24–1.38) | 1.40× (1.36–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.44–1.60) | 3.04× (2.76–3.34) | 2.33× (2.22–2.46) | 3.07× (2.92–3.24) | 2.76× (2.64–2.89) | 2.52× (2.40–2.66) | 2.80× (2.65–2.97) | 0.95× (0.89–1.02) | 1.02× (0.98–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.69) | 2.58× (2.38–2.78) | 3.36× (3.26–3.47) | 2.81× (2.72–2.91) | 6.58× (5.70–7.59) | 2.11× (2.06–2.16) | 2.15× (2.08–2.21) | 0.98× (0.96–1.00) | 1.45× (1.40–1.50) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.58–0.61) | 2.35× (2.29–2.41) | 4.21× (4.05–4.37) | 2.75× (2.68–2.83) | 5.92× (5.64–6.22) | 1.96× (1.92–2.00) | 2.00× (1.94–2.05) | 0.91× (0.89–0.93) | 1.35× (1.31–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.59) | 2.55× (2.47–2.62) | 4.22× (4.04–4.41) | 2.80× (2.70–2.90) | 5.93× (5.77–6.10) | 2.03× (1.97–2.09) | 2.06× (2.00–2.13) | 0.94× (0.91–0.97) | 1.39× (1.35–1.43) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.79× (1.65–1.94) | 4.06× (3.92–4.21) | 4.39× (4.25–4.53) | 4.51× (4.41–4.62) | 4.76× (4.66–4.87) | 4.73× (4.63–4.84) | 4.75× (4.62–4.88) | 1.52× (1.44–1.61) | 1.60× (1.55–1.65) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.34× (1.18–1.52) | 3.08× (2.67–3.54) | 3.40× (3.03–3.82) | 3.91× (3.39–4.51) | 3.57× (3.16–4.03) | 3.35× (3.00–3.73) | 3.36× (2.97–3.80) | 1.08× (0.93–1.24) | 1.13× (1.01–1.27) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 1.51× (1.43–1.59) | 2.88× (2.49–3.33) | 3.40× (2.99–3.86) | 3.37× (3.03–3.75) | 3.39× (3.02–3.80) | 2.31× (2.25–2.37) | 2.32× (2.24–2.40) | 0.74× (0.70–0.79) | 0.78× (0.76–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 33 MB | 0.26× (0.26–0.26) | 0.29× (0.28–0.30) | 0.84× (0.84–0.85) | 0.68× (0.67–0.69) | 9.54× (9.53–9.56) | 0.23× (0.23–0.23) | 0.25× (0.25–0.25) | 0.37× (0.37–0.37) | 0.19× (0.18–0.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 78 MB | 0.25× (0.24–0.26) | 0.40× (0.36–0.44) | 0.52× (0.49–0.54) | 0.43× (0.41–0.44) | 4.07× (3.91–4.23) | 0.10× (0.09–0.10) | 0.10× (0.10–0.11) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 135 MB | 0.23× (0.22–0.25) | 0.37× (0.31–0.43) | 0.43× (0.39–0.46) | 0.37× (0.34–0.40) | 2.43× (2.25–2.62) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.09× (0.09–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.46× (0.44–0.47) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.03–8.06) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.62–0.70) | 0.67× (0.63–0.72) | 1.58× (1.50–1.67) | 0.78× (0.74–0.83) | 5.73× (5.42–6.05) | 0.46× (0.44–0.49) | 0.49× (0.47–0.52) | 0.53× (0.50–0.56) | 0.26× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.60–0.67) | 0.71× (0.67–0.75) | 1.16× (1.10–1.22) | 0.70× (0.67–0.73) | 3.35× (3.20–3.50) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.42× (0.41–0.44) | 0.91× (0.87–0.96) | 0.73× (0.70–0.76) | 3.15× (3.02–3.29) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.45–0.46) | 0.87× (0.86–0.89) | 0.82× (0.80–0.83) | 2.53× (2.49–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.23× (5.22–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.39× (0.37–0.40) | 0.85× (0.82–0.89) | 0.71× (0.69–0.74) | 2.93× (2.81–3.06) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.45× (0.43–0.47) | 0.81× (0.79–0.83) | 0.82× (0.80–0.83) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.43) | 0.66× (0.65–0.66) | 6.43× (6.41–6.45) | 0.39× (0.39–0.39) | 0.43× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 35 MB | 0.49× (0.48–0.50) | 0.43× (0.40–0.47) | 1.14× (1.11–1.16) | 0.74× (0.72–0.75) | 3.58× (3.51–3.66) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.46× (0.43–0.50) | 0.92× (0.91–0.94) | 0.72× (0.70–0.75) | 2.36× (2.31–2.40) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 626 | 1.60 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,036 | 1.87 | 1.17× (1.13–1.20) | 81% | 1.05× (1.01–1.10) | 1.11× (1.07–1.14) |
| 4: all 4 CPUs, sharing SMT siblings | 1,128 | 3.07 | 1.93× (1.86–2.00) | 45% | 1.34× (1.25–1.43) | 1.90× (1.82–1.98) |

**Best:** mibz2 at 4 threads (1,128 MB/s); lbzip2 at 4 threads (264 MB/s); pbz2 at 4 threads (410 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.13, lbzip2 1t 0.08, lbzip2 2t 0.35, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.44, pbz2 4t 0.54. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.01 | 0.08 | 0.186 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4483 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 13.8% |
| MTF + RLE2 | 1.51 | 20.0% |
| T-vector build | 1.33 | 17.7% |
| walk (the one each build uses) | 3.20 | 42.4% |
| of which: segment assembly | 0.14 | 1.9% |
| RLE1 | 0.16 | 2.1% |
| CRC | 0.26 | 3.5% |
| **total** | 7.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.303 | 0.714 | (in its total) | 2.36× (2.32–2.39) |  |
| walk | 0.682 | 1.605 | (in its total) | 2.36× (2.32–2.40) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.032 | 0.034 | (in its total) | 1.09× (1.04–1.14) |  |
| inverse BWT, total | 1.019 | 2.353 | 1.372 | 2.32× (2.28–2.36) | 1.33× (1.30–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 48m38s ± 5m47s |
| 4 threads | 27m04s ± 3m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m18s ± 2m45s |
| the scanner alone (its serial part) | 8m18s ± 30.2s |
| the workers alone, over 4 | 18m18s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 4.8 ns a byte of bz2, the workers 2.40 ns a byte of output at 4 threads (CPU, summed over them), 1.32 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,443,515 | 12,305,874 | \`da24cf0eff3da816…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4482.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
