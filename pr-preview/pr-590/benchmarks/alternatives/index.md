# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-590/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`29bb958`](<https://github.com/lmmx/mibz2/commit/29bb958d8d7565c6eb680f13cd79f86207c2fd2a>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44386613034>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.312 s | 672 | 6.3 | 2.91× (2.82–3.01) | 7.08× (6.95–7.22) | 7.50× (7.28–7.73) | 8.39× (8.17–8.62) | 7.33× (7.17–7.50) | 7.59× (7.44–7.74) | 7.34× (7.14–7.55) | 2.21× (2.14–2.28) | 2.53× (2.48–2.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.190 s | 1,102 | 7.3 | 2.90× (2.70–3.10) | 6.19× (5.78–6.63) | 7.73× (7.35–8.14) | 7.47× (7.15–7.81) | 6.60× (6.28–6.93) | 12.26× (11.72–12.82) | 11.85× (11.25–12.48) | 3.57× (3.35–3.80) | 4.09× (3.91–4.27) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.166 s | 1,266 | 11.2 | 3.00× (2.79–3.22) | 4.57× (4.17–5.00) | 6.19× (5.69–6.74) | 5.56× (5.15–6.01) | 5.33× (5.00–5.69) | 14.29× (13.61–15.01) | 13.82× (13.14–14.54) | 4.16× (3.98–4.35) | 4.77× (4.55–5.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,375 | 3.4 | 0.57× (0.55–0.58) | 3.52× (3.44–3.61) | 4.47× (4.30–4.66) | 3.67× (3.60–3.74) | 6.51× (6.33–6.69) | 2.79× (2.69–2.91) | 2.81× (2.63–3.00) | 1.01× (0.99–1.04) | 2.18× (2.06–2.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,347 | 3.6 | 0.55× (0.46–0.65) | 2.88× (2.69–3.08) | 5.79× (5.61–5.98) | 3.57× (3.51–3.62) | 6.30× (6.16–6.44) | 2.71× (2.65–2.78) | 2.73× (2.58–2.88) | 0.98× (0.94–1.03) | 2.12× (2.00–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,320 | 3.5 | 0.50× (0.49–0.52) | 2.84× (2.64–3.04) | 5.83× (5.57–6.10) | 3.53× (3.43–3.64) | 6.23× (6.00–6.47) | 2.67× (2.56–2.78) | 2.68× (2.50–2.87) | 0.97× (0.92–1.01) | 2.08× (1.95–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,032 | 4.5 | 2.95× (2.90–3.00) | 3.66× (3.52–3.80) | 4.49× (4.13–4.88) | 4.63× (4.53–4.73) | 5.36× (5.24–5.49) | 3.07× (2.91–3.25) | 2.67× (2.37–3.01) | 2.17× (2.09–2.24) | 3.21× (3.10–3.31) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,269 | 6.2 | 1.92× (1.63–2.26) | 2.55× (2.17–3.00) | 3.60× (3.05–4.25) | 3.26× (2.85–3.74) | 4.85× (4.21–5.60) | 3.55× (3.03–4.15) | 3.09× (2.51–3.80) | 2.50× (2.13–2.93) | 3.70× (3.19–4.29) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,120 | 12.2 | 1.62× (1.47–1.78) | 1.91× (1.63–2.25) | 2.68× (2.37–3.04) | 2.05× (1.69–2.48) | 4.26× (3.66–4.97) | 3.17× (2.76–3.63) | 2.75× (2.53–2.99) | 2.23× (1.98–2.52) | 3.30× (2.89–3.78) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.086 s | 94 | 46.2 | 1.35× (1.33–1.37) | 2.94× (2.88–3.01) | 2.71× (2.62–2.80) | 2.64× (2.58–2.69) | 2.94× (2.88–3.01) | 3.39× (3.29–3.49) | 3.35× (3.26–3.44) | 1.27× (1.23–1.31) | 1.50× (1.39–1.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.057 s | 140 | 54.5 | 1.22× (1.17–1.27) | 2.77× (2.59–2.97) | 2.46× (2.32–2.60) | 3.05× (2.70–3.43) | 2.75× (2.43–3.11) | 4.89× (4.23–5.66) | 4.84× (4.21–5.56) | 1.84× (1.59–2.11) | 2.17× (2.02–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 176 | 72.1 | 1.29× (1.18–1.41) | 2.52× (2.25–2.81) | 2.38× (2.20–2.57) | 2.73× (2.62–2.83) | 3.02× (2.82–3.22) | 6.32× (6.00–6.65) | 6.25× (5.99–6.52) | 2.37× (2.27–2.48) | 2.81× (2.57–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.089 s | 108 | 40.4 | 1.45× (1.41–1.49) | 3.61× (3.49–3.73) | 2.96× (2.87–3.04) | 2.94× (2.84–3.04) | 3.39× (3.25–3.54) | 4.24× (4.07–4.41) | 4.64× (4.39–4.91) | 1.52× (1.47–1.56) | 1.78× (1.71–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.061 s | 157 | 49.4 | 1.24× (1.19–1.30) | 3.03× (2.90–3.16) | 2.68× (2.55–2.83) | 3.58× (3.34–3.84) | 3.17× (3.04–3.30) | 6.26× (6.00–6.54) | 6.86× (6.44–7.31) | 2.24× (2.15–2.34) | 2.64× (2.50–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.050 s | 192 | 67.1 | 1.39× (1.34–1.44) | 2.69× (2.60–2.77) | 2.57× (2.36–2.80) | 3.29× (3.11–3.47) | 3.05× (2.94–3.16) | 7.64× (7.36–7.93) | 8.37× (7.91–8.85) | 2.73× (2.63–2.84) | 3.21× (3.06–3.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 4.9 | 0.61× (0.59–0.62) | 3.23× (3.03–3.44) | 4.30× (4.17–4.43) | 3.19× (3.01–3.39) | 7.03× (6.89–7.17) | 2.36× (2.28–2.44) | 2.34× (2.26–2.42) | 0.96× (0.94–0.99) | 2.10× (2.04–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 962 | 5.2 | 0.50× (0.48–0.52) | 3.05× (2.80–3.32) | 5.06× (4.85–5.29) | 3.07× (2.87–3.28) | 6.65× (6.45–6.85) | 2.22× (2.14–2.30) | 2.20× (2.12–2.29) | 0.91× (0.88–0.94) | 1.97× (1.90–2.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 966 | 5.1 | 0.50× (0.48–0.52) | 2.97× (2.91–3.04) | 5.08× (4.96–5.21) | 3.00× (2.93–3.07) | 6.97× (6.39–7.59) | 2.22× (2.14–2.30) | 2.21× (2.17–2.24) | 0.91× (0.89–0.93) | 1.98× (1.93–2.03) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.045 s | 288 | 15.2 | 1.74× (1.66–1.82) | 4.15× (3.90–4.41) | 4.80× (4.20–5.49) | 4.77× (4.36–5.22) | 4.93× (4.67–5.20) | 4.84× (4.56–5.13) | 4.62× (4.40–4.85) | 1.47× (1.42–1.52) | 1.80× (1.74–1.85) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.035 s | 373 | 20.6 | 1.51× (1.38–1.65) | 3.33× (3.03–3.66) | 4.00× (3.79–4.23) | 4.15× (3.92–4.39) | 4.19× (4.05–4.34) | 6.41× (6.10–6.74) | 6.12× (5.82–6.44) | 1.95× (1.87–2.03) | 2.38× (2.27–2.51) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.033 s | 397 | 31.9 | 1.31× (1.27–1.35) | 2.49× (2.34–2.64) | 3.25× (2.92–3.63) | 3.14× (2.95–3.34) | 3.81× (3.56–4.09) | 6.80× (6.52–7.09) | 6.49× (6.23–6.77) | 2.07× (1.99–2.15) | 2.53× (2.38–2.69) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.81–3.00) | 7.03× (6.91–7.17) | 7.45× (7.24–7.68) | 8.35× (8.13–8.57) | 7.30× (7.14–7.46) | 7.55× (7.40–7.70) | 7.30× (7.10–7.51) | 2.20× (2.13–2.28) | 2.51× (2.45–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.70× (2.58–2.83) | 6.41× (6.25–6.56) | 7.15× (6.96–7.36) | 7.93× (7.73–8.13) | 6.67× (6.51–6.83) | 6.60× (6.42–6.79) | 6.39× (6.20–6.58) | 1.93× (1.85–2.01) | 2.19× (2.14–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.94× (2.72–3.18) | 5.27× (4.79–5.81) | 6.07× (5.56–6.62) | 6.43× (5.91–6.99) | 5.42× (5.02–5.85) | 4.27× (4.05–4.49) | 4.13× (3.91–4.36) | 1.25× (1.19–1.30) | 1.42× (1.35–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.63) | 3.01× (2.93–3.09) | 3.89× (3.79–3.98) | 3.39× (3.32–3.46) | 5.92× (5.75–6.09) | 2.62× (2.50–2.74) | 2.62× (2.46–2.79) | 1.01× (0.99–1.04) | 1.84× (1.78–1.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.52–0.71) | 2.79× (2.69–2.90) | 4.96× (4.88–5.04) | 3.28× (3.23–3.33) | 5.68× (5.56–5.80) | 2.51× (2.43–2.58) | 2.51× (2.38–2.64) | 0.97× (0.93–1.01) | 1.76× (1.69–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 3.02× (2.93–3.10) | 4.98× (4.86–5.09) | 3.32× (3.23–3.41) | 5.73× (5.52–5.94) | 2.53× (2.41–2.65) | 2.53× (2.37–2.70) | 0.98× (0.94–1.02) | 1.77× (1.70–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.78× (2.72–2.84) | 3.18× (3.07–3.29) | 3.93× (3.61–4.28) | 4.32× (4.20–4.44) | 4.99× (4.87–5.12) | 2.90× (2.73–3.07) | 2.56× (2.25–2.90) | 2.06× (1.98–2.15) | 2.73× (2.67–2.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.05–2.34) | 2.44× (2.28–2.62) | 2.98× (2.80–3.17) | 3.25× (3.08–3.44) | 3.78× (3.62–3.95) | 2.08× (1.90–2.28) | 1.84× (1.60–2.12) | 1.48× (1.39–1.58) | 1.96× (1.84–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.82× (1.68–1.98) | 1.69× (1.47–1.96) | 1.93× (1.74–2.15) | 1.98× (1.71–2.31) | 2.29× (1.93–2.71) | 1.02× (0.89–1.16) | 0.90× (0.81–0.99) | 0.73× (0.65–0.81) | 0.96× (0.85–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.32–1.36) | 2.85× (2.80–2.90) | 2.62× (2.53–2.70) | 2.60× (2.55–2.66) | 2.90× (2.84–2.96) | 3.33× (3.23–3.44) | 3.30× (3.21–3.38) | 1.26× (1.23–1.30) | 1.44× (1.33–1.55) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.18–1.29) | 2.78× (2.58–3.00) | 2.26× (2.17–2.36) | 2.94× (2.63–3.29) | 2.45× (2.16–2.78) | 2.71× (2.35–3.14) | 2.68× (2.33–3.09) | 1.03× (0.89–1.19) | 1.17× (1.08–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.26–1.61) | 2.67× (2.37–3.00) | 2.08× (1.93–2.23) | 2.70× (2.59–2.81) | 2.62× (2.41–2.86) | 2.14× (2.02–2.28) | 2.12× (2.02–2.23) | 0.81× (0.77–0.86) | 0.92× (0.85–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.39–1.48) | 3.50× (3.37–3.63) | 2.85× (2.76–2.94) | 2.89× (2.79–2.99) | 3.33× (3.19–3.48) | 4.16× (3.99–4.33) | 4.55× (4.29–4.82) | 1.50× (1.45–1.55) | 1.71× (1.65–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.27–1.33) | 3.11× (3.00–3.22) | 2.52× (2.41–2.63) | 3.46× (3.27–3.67) | 2.94× (2.81–3.07) | 3.47× (3.35–3.60) | 3.80× (3.58–4.03) | 1.25× (1.20–1.30) | 1.43× (1.36–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.52–1.57) | 2.98× (2.84–3.12) | 2.28× (2.07–2.51) | 3.09× (2.95–3.24) | 2.78× (2.67–2.89) | 2.52× (2.43–2.61) | 2.76× (2.63–2.89) | 0.91× (0.88–0.94) | 1.04× (1.00–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.65–0.70) | 2.55× (2.45–2.65) | 3.42× (3.33–3.52) | 2.88× (2.71–3.06) | 6.17× (6.05–6.28) | 2.18× (2.07–2.31) | 2.15× (2.08–2.22) | 0.97× (0.95–1.00) | 1.48× (1.46–1.51) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.56–0.60) | 2.36× (2.19–2.54) | 4.10× (3.93–4.27) | 2.79× (2.62–2.97) | 5.75× (5.59–5.91) | 2.02× (1.92–2.12) | 1.99× (1.91–2.06) | 0.90× (0.88–0.92) | 1.37× (1.32–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.50× (2.43–2.56) | 4.09× (3.99–4.19) | 2.83× (2.75–2.90) | 6.21× (5.72–6.75) | 2.09× (1.99–2.21) | 2.06× (2.02–2.09) | 0.93× (0.91–0.95) | 1.42× (1.38–1.47) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.71× (1.64–1.79) | 3.94× (3.74–4.16) | 4.56× (3.98–5.23) | 4.64× (4.24–5.07) | 4.79× (4.54–5.05) | 4.70× (4.43–4.98) | 4.49× (4.27–4.71) | 1.46× (1.41–1.50) | 1.62× (1.58–1.67) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.45× (1.41–1.48) | 3.33× (3.05–3.63) | 3.57× (3.43–3.73) | 4.07× (3.89–4.26) | 3.70× (3.60–3.80) | 3.55× (3.39–3.71) | 3.39× (3.24–3.55) | 1.10× (1.05–1.15) | 1.23× (1.16–1.30) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.49× (1.43–1.54) | 2.73× (2.53–2.95) | 2.99× (2.65–3.37) | 3.24× (3.06–3.43) | 3.17× (2.91–3.46) | 2.30× (2.20–2.40) | 2.19× (2.09–2.30) | 0.71× (0.69–0.74) | 0.79× (0.74–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.76–0.79) | 10.99× (10.98–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.41–0.42) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.31× (0.30–0.33) | 0.46× (0.38–0.55) | 0.64× (0.61–0.68) | 0.55× (0.53–0.58) | 5.17× (4.90–5.45) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.18–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.34) | 0.48× (0.39–0.60) | 0.60× (0.57–0.62) | 0.52× (0.50–0.54) | 3.42× (3.28–3.56) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.44–0.46) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.02–8.06) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.64–0.68) | 0.66× (0.61–0.70) | 1.59× (1.54–1.63) | 0.80× (0.78–0.83) | 5.75× (5.58–5.92) | 0.47× (0.45–0.48) | 0.49× (0.47–0.50) | 0.53× (0.51–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.66) | 0.72× (0.65–0.78) | 1.13× (1.08–1.19) | 0.71× (0.68–0.74) | 3.32× (3.16–3.49) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.74) | 5.18× (5.17–5.20) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.42× (0.40–0.44) | 0.90× (0.86–0.94) | 0.72× (0.68–0.76) | 3.11× (2.98–3.25) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.86× (0.85–0.88) | 0.82× (0.79–0.85) | 2.54× (2.49–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.41) | 0.85× (0.83–0.88) | 0.72× (0.69–0.76) | 2.94× (2.85–3.03) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.50) | 0.79× (0.77–0.81) | 0.81× (0.78–0.84) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.30× (5.28–5.32) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.31× (5.29–5.32) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.66) | 6.37× (6.36–6.38) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.61–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 35 MB | 0.49× (0.48–0.50) | 0.45× (0.43–0.48) | 1.13× (1.08–1.18) | 0.74× (0.73–0.76) | 3.59× (3.51–3.67) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 54 MB | 0.52× (0.50–0.54) | 0.48× (0.44–0.53) | 0.95× (0.92–0.98) | 0.78× (0.75–0.81) | 2.41× (2.33–2.48) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 672 | 1.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,102 | 1.73 | 1.14× (1.11–1.18) | 81% | 1.04× (1.02–1.06) | 1.07× (0.99–1.14) |
| 4: all 4 CPUs, sharing SMT siblings | 1,266 | 2.64 | 1.77× (1.68–1.86) | 47% | 1.33× (1.22–1.44) | 1.79× (1.64–1.96) |

**Best:** mibz2 at 4 threads (1,266 MB/s); lbzip2 at 4 threads (280 MB/s); pbz2 at 4 threads (421 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.88, mibz2 4t 1.25, lbzip2 1t 0.08, lbzip2 2t 0.31, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.58. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.41 | 0.01 | 0.07 | 0.166 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4236 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 41.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 13.8% |
| MTF + RLE2 | 1.45 | 19.8% |
| T-vector build | 1.32 | 18.0% |
| walk (the one each build uses) | 3.18 | 43.4% |
| of which: segment assembly | 0.15 | 2.0% |
| RLE1 | 0.16 | 2.1% |
| CRC | 0.25 | 3.4% |
| **total** | 7.32 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.305 | 0.730 | (in its total) | 2.42× (2.36–2.47) |  |
| walk | 0.668 | 1.624 | (in its total) | 2.47× (2.42–2.52) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.030 | 0.033 | (in its total) | 1.05× (0.99–1.12) |  |
| inverse BWT, total | 1.002 | 2.387 | 1.340 | 2.41× (2.36–2.46) | 1.34× (1.32–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m34s ± 5m08s |
| 4 threads | 24m15s ± 3m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m06s ± 2m18s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 15m06s ± 2m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 1.98 ns a byte of output at 4 threads (CPU, summed over them), 1.19 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,747 | 13,092,730 | \`9b1907f0616f895e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4235.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
