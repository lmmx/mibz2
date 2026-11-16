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

The latest record: [`4b8a207`](<https://github.com/lmmx/mibz2/commit/4b8a2071b352ef125e9b41e32d2f85e89899cb50>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43943330683>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.402 s | 521 | 5.8 | 3.34× (3.33–3.35) | 7.62× (7.59–7.66) | 8.16× (8.12–8.21) | 9.04× (9.01–9.08) | 8.18× (8.14–8.22) | 8.53× (8.49–8.57) | 8.35× (8.30–8.40) | 2.59× (2.59–2.60) | 2.32× (2.31–2.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.225 s | 931 | 6.2 | 3.24× (3.00–3.51) | 6.94× (6.51–7.40) | 8.36× (7.78–8.98) | 8.31× (7.68–8.99) | 7.54× (6.97–8.15) | 14.71× (13.59–15.91) | 14.39× (13.31–15.56) | 4.47× (4.14–4.83) | 4.01× (3.71–4.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.189 s | 1,108 | 9.7 | 3.30× (3.24–3.37) | 6.07× (5.96–6.19) | 7.15× (7.02–7.28) | 7.09× (6.97–7.21) | 6.92× (6.80–7.05) | 18.05× (17.71–18.40) | 17.67× (17.30–18.05) | 5.49× (5.39–5.59) | 4.92× (4.82–5.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,227 | 2.7 | 0.65× (0.63–0.66) | 3.12× (3.05–3.20) | 4.46× (4.35–4.57) | 3.81× (3.72–3.90) | 6.40× (6.27–6.52) | 2.81× (2.75–2.87) | 2.84× (2.78–2.90) | 1.12× (1.10–1.13) | 1.26× (1.23–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,212 | 2.8 | 0.56× (0.54–0.57) | 3.07× (3.00–3.14) | 6.20× (6.05–6.34) | 3.77× (3.69–3.85) | 6.22× (6.08–6.36) | 2.75× (2.69–2.82) | 2.78× (2.72–2.85) | 1.09× (1.06–1.13) | 1.24× (1.21–1.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,223 | 2.7 | 0.57× (0.56–0.58) | 3.12× (3.05–3.19) | 5.85× (5.68–6.02) | 3.82× (3.75–3.90) | 6.30× (6.15–6.46) | 2.79× (2.74–2.85) | 2.82× (2.77–2.88) | 1.11× (1.09–1.13) | 1.26× (1.23–1.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 644 | 4.9 | 2.87× (2.83–2.91) | 3.32× (3.29–3.35) | 3.99× (3.91–4.07) | 4.35× (4.31–4.40) | 4.75× (4.69–4.80) | 3.17× (3.13–3.21) | 2.95× (2.91–2.98) | 1.99× (1.94–2.05) | 2.09× (1.98–2.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 938 | 5.8 | 2.25× (2.06–2.47) | 2.92× (2.62–3.24) | 3.99× (3.62–4.41) | 3.57× (3.23–3.94) | 4.78× (4.38–5.22) | 4.55× (4.17–4.97) | 4.23× (3.86–4.63) | 2.86× (2.62–3.12) | 3.00× (2.82–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,034 | 10.3 | 1.97× (1.86–2.09) | 2.16× (2.05–2.29) | 3.11× (2.89–3.35) | 2.57× (2.35–2.81) | 4.31× (4.10–4.54) | 5.01× (4.76–5.26) | 4.65× (4.43–4.88) | 3.14× (2.94–3.36) | 3.29× (3.06–3.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 s | 70 | 43.1 | 1.36× (1.36–1.37) | 3.02× (3.00–3.04) | 2.72× (2.67–2.77) | 2.79× (2.78–2.80) | 3.07× (3.05–3.08) | 3.40× (3.39–3.42) | 3.50× (3.49–3.52) | 1.24× (1.24–1.25) | 1.24× (1.24–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 115 | 47.5 | 1.34× (1.12–1.60) | 2.69× (2.47–2.93) | 2.65× (2.46–2.86) | 3.24× (2.99–3.51) | 3.02× (2.79–3.26) | 5.36× (4.95–5.80) | 5.52× (5.09–5.98) | 1.95× (1.80–2.12) | 1.96× (1.81–2.12) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 141 | 63.6 | 1.38× (1.35–1.41) | 2.71× (2.65–2.78) | 2.61× (2.56–2.66) | 3.22× (3.16–3.27) | 3.20× (3.14–3.27) | 6.89× (6.77–7.01) | 7.09× (6.98–7.20) | 2.51× (2.47–2.55) | 2.52× (2.47–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 s | 76 | 39.8 | 1.41× (1.40–1.42) | 3.34× (3.31–3.37) | 2.83× (2.81–2.85) | 2.96× (2.95–2.97) | 3.29× (3.28–3.30) | 4.06× (4.05–4.07) | 4.36× (4.34–4.38) | 1.45× (1.44–1.46) | 1.53× (1.52–1.53) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 129 | 43.6 | 1.32× (1.29–1.34) | 3.23× (2.97–3.50) | 2.87× (2.81–2.94) | 3.84× (3.75–3.93) | 3.30× (3.24–3.37) | 6.85× (6.73–6.98) | 7.36× (7.23–7.50) | 2.44× (2.40–2.49) | 2.58× (2.54–2.62) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 61.8 | 1.43× (1.39–1.46) | 2.91× (2.85–2.97) | 2.72× (2.66–2.78) | 3.56× (3.50–3.61) | 3.17× (3.11–3.23) | 8.10× (7.94–8.26) | 8.71× (8.53–8.89) | 2.89× (2.84–2.94) | 3.05× (2.99–3.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 930 | 3.7 | 0.69× (0.67–0.71) | 2.86× (2.82–2.91) | 3.96× (3.89–4.02) | 3.27× (3.24–3.30) | 6.96× (6.87–7.06) | 2.45× (2.41–2.48) | 2.48× (2.45–2.51) | 1.04× (1.01–1.06) | 0.98× (0.96–0.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 889 | 4.0 | 0.54× (0.52–0.56) | 2.72× (2.62–2.83) | 5.54× (5.32–5.77) | 3.15× (3.05–3.26) | 6.70× (6.48–6.93) | 2.32× (2.24–2.40) | 2.36× (2.28–2.44) | 0.98× (0.95–1.02) | 0.92× (0.89–0.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 885 | 3.9 | 0.55× (0.54–0.56) | 2.72× (2.67–2.78) | 5.53× (5.41–5.64) | 3.18× (3.13–3.23) | 6.66× (6.50–6.82) | 2.32× (2.28–2.35) | 2.35× (2.31–2.39) | 0.98× (0.96–1.01) | 0.92× (0.91–0.94) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 0.057 s | 214 | 14.4 | 1.80× (1.76–1.85) | 4.06× (3.97–4.16) | 4.61× (4.50–4.72) | 4.67× (4.57–4.78) | 4.85× (4.74–4.97) | 5.02× (4.90–5.14) | 4.98× (4.87–5.09) | 1.45× (1.42–1.48) | 1.41× (1.38–1.45) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 0.038 s | 316 | 17.5 | 1.43× (1.35–1.52) | 3.29× (3.01–3.61) | 4.42× (4.10–4.77) | 4.48× (4.09–4.91) | 4.30× (3.92–4.71) | 7.24× (6.61–7.93) | 7.18× (6.56–7.85) | 2.09× (1.91–2.29) | 2.04× (1.86–2.23) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 0.032 s | 376 | 25.5 | 1.54× (1.48–1.60) | 2.96× (2.82–3.10) | 4.20× (4.06–4.34) | 4.26× (4.12–4.40) | 4.27× (4.11–4.43) | 8.91× (8.62–9.22) | 8.83× (8.55–9.13) | 2.58× (2.49–2.66) | 2.51× (2.43–2.60) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.33× (3.32–3.34) | 7.58× (7.54–7.62) | 8.12× (8.07–8.16) | 9.01× (8.98–9.04) | 8.15× (8.11–8.19) | 8.50× (8.46–8.54) | 8.32× (8.27–8.36) | 2.59× (2.58–2.59) | 2.30× (2.29–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.19× (3.17–3.20) | 7.32× (7.27–7.37) | 8.06× (8.02–8.10) | 8.81× (8.76–8.87) | 7.64× (7.59–7.69) | 7.84× (7.79–7.90) | 7.67× (7.62–7.73) | 2.39× (2.38–2.40) | 2.13× (2.11–2.14) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.27× (3.20–3.34) | 6.70× (6.61–6.80) | 7.15× (7.05–7.26) | 7.82× (7.71–7.94) | 6.97× (6.87–7.07) | 5.07× (4.99–5.14) | 4.96× (4.89–5.03) | 1.54× (1.52–1.57) | 1.37× (1.35–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.67–0.69) | 2.87× (2.80–2.93) | 3.85× (3.78–3.93) | 3.55× (3.47–3.63) | 5.90× (5.79–6.00) | 2.64× (2.58–2.69) | 2.67× (2.61–2.72) | 1.10× (1.09–1.12) | 0.82× (0.80–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.62) | 2.81× (2.76–2.86) | 5.38× (5.28–5.48) | 3.47× (3.41–3.54) | 5.66× (5.54–5.77) | 2.54× (2.49–2.60) | 2.57× (2.52–2.63) | 1.07× (1.04–1.10) | 0.79× (0.77–0.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.59–0.62) | 3.27× (3.18–3.37) | 5.40× (5.31–5.48) | 3.59× (3.53–3.66) | 5.83× (5.70–5.97) | 2.64× (2.59–2.68) | 2.67× (2.62–2.71) | 1.10× (1.08–1.13) | 0.82× (0.80–0.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.77× (2.74–2.81) | 3.07× (3.04–3.10) | 3.61× (3.57–3.64) | 4.18× (4.14–4.21) | 4.55× (4.50–4.60) | 3.06× (3.03–3.09) | 2.84× (2.82–2.87) | 1.94× (1.89–1.99) | 1.91× (1.88–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.52× (2.47–2.57) | 2.79× (2.72–2.86) | 3.40× (3.34–3.46) | 3.65× (3.58–3.73) | 3.98× (3.90–4.06) | 2.60× (2.54–2.65) | 2.41× (2.36–2.46) | 1.65× (1.61–1.69) | 1.62× (1.59–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.09–2.17) | 1.94× (1.90–1.98) | 2.38× (2.35–2.42) | 2.33× (2.27–2.39) | 2.45× (2.39–2.51) | 1.47× (1.44–1.50) | 1.37× (1.34–1.40) | 0.93× (0.91–0.96) | 0.92× (0.90–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.35–1.36) | 2.92× (2.90–2.93) | 2.68× (2.64–2.71) | 2.76× (2.75–2.78) | 3.04× (3.02–3.05) | 3.37× (3.35–3.39) | 3.47× (3.45–3.48) | 1.24× (1.23–1.24) | 1.22× (1.22–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.25–1.32) | 2.79× (2.72–2.86) | 2.55× (2.49–2.61) | 3.24× (3.17–3.31) | 2.83× (2.77–2.90) | 3.04× (2.97–3.11) | 3.13× (3.06–3.20) | 1.12× (1.09–1.14) | 1.10× (1.08–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.51–1.60) | 2.96× (2.89–3.03) | 2.48× (2.40–2.56) | 3.23× (3.14–3.32) | 2.89× (2.82–2.96) | 2.30× (2.25–2.36) | 2.37× (2.31–2.43) | 0.85× (0.82–0.87) | 0.83× (0.81–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.40–1.41) | 3.27× (3.26–3.28) | 2.78× (2.78–2.79) | 2.93× (2.92–2.94) | 3.26× (3.25–3.28) | 4.02× (4.01–4.03) | 4.32× (4.30–4.34) | 1.44× (1.44–1.45) | 1.45× (1.45–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.34–1.35) | 3.14× (3.11–3.17) | 2.69× (2.67–2.71) | 3.63× (3.60–3.66) | 3.06× (3.04–3.08) | 3.66× (3.63–3.69) | 3.94× (3.90–3.97) | 1.31× (1.31–1.32) | 1.32× (1.32–1.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.55–1.63) | 3.19× (3.11–3.27) | 2.41× (2.34–2.49) | 3.36× (3.25–3.47) | 2.89× (2.81–2.97) | 2.58× (2.51–2.64) | 2.77× (2.69–2.84) | 0.92× (0.90–0.95) | 0.93× (0.91–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.71–0.75) | 2.49× (2.46–2.52) | 3.46× (3.40–3.52) | 2.97× (2.93–3.01) | 6.19× (6.11–6.26) | 2.25× (2.22–2.28) | 2.29× (2.26–2.32) | 1.03× (1.01–1.05) | 0.77× (0.76–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.62) | 2.28× (2.21–2.36) | 4.39× (4.27–4.51) | 2.84× (2.77–2.92) | 5.84× (5.68–6.01) | 2.09× (2.03–2.14) | 2.12× (2.07–2.18) | 0.96× (0.93–0.98) | 0.71× (0.69–0.74) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.66× (2.58–2.76) | 4.48× (4.42–4.53) | 2.96× (2.92–3.00) | 5.99× (5.86–6.11) | 2.15× (2.12–2.17) | 2.18× (2.16–2.21) | 0.98× (0.96–1.00) | 0.74× (0.72–0.75) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 1.78× (1.74–1.82) | 3.95× (3.87–4.04) | 4.42× (4.33–4.52) | 4.57× (4.47–4.67) | 4.74× (4.64–4.85) | 4.90× (4.79–5.02) | 4.86× (4.76–4.97) | 1.44× (1.40–1.47) | 1.35× (1.32–1.38) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 1.57× (1.54–1.60) | 3.46× (3.40–3.52) | 4.09× (4.03–4.16) | 4.34× (4.26–4.42) | 4.09× (4.02–4.17) | 4.09× (4.02–4.17) | 4.06× (3.99–4.13) | 1.20× (1.18–1.22) | 1.12× (1.10–1.14) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 1.80× (1.74–1.86) | 3.22× (3.12–3.33) | 3.56× (3.46–3.66) | 3.87× (3.79–3.96) | 3.70× (3.62–3.78) | 2.77× (2.71–2.83) | 2.74× (2.69–2.80) | 0.81× (0.79–0.83) | 0.76× (0.75–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.27× (0.26–0.29) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.06× (11.04–11.08) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.32–0.36) | 0.52× (0.43–0.63) | 0.71× (0.67–0.74) | 0.59× (0.56–0.62) | 5.57× (5.31–5.83) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.32–0.34) | 0.50× (0.42–0.60) | 0.59× (0.57–0.60) | 0.52× (0.50–0.54) | 3.46× (3.38–3.55) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.42× (0.40–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.66) | 8.04× (8.01–8.08) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.62× (0.56–0.69) | 1.61× (1.56–1.67) | 0.81× (0.78–0.84) | 5.85× (5.66–6.03) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.71× (0.65–0.77) | 1.14× (1.10–1.18) | 0.72× (0.68–0.75) | 3.41× (3.31–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.43) | 0.91× (0.89–0.94) | 0.71× (0.69–0.74) | 3.15× (3.06–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.48) | 0.87× (0.85–0.90) | 0.81× (0.79–0.84) | 2.54× (2.47–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.40) | 0.86× (0.83–0.90) | 0.71× (0.67–0.76) | 2.95× (2.83–3.07) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.50× (0.48–0.52) | 0.82× (0.81–0.83) | 0.83× (0.81–0.85) | 2.37× (2.34–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.20–0.23) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.28–5.30) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.40× (6.38–6.42) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.34× (0.31–0.36) | 0.97× (0.94–1.00) | 0.62× (0.61–0.64) | 3.05× (2.98–3.13) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.44× (0.40–0.49) | 0.88× (0.86–0.91) | 0.71× (0.69–0.73) | 2.26× (2.21–2.30) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 521 | 1.93 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 931 | 2.09 | 1.08× (1.08–1.09) | 86% | 1.05× (1.04–1.05) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,108 | 3.24 | 1.68× (1.65–1.70) | 53% | 1.48× (1.47–1.49) | 1.65× (1.63–1.67) |

**Best:** mibz2 at 4 threads (1,108 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (332 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.24, lbzip2 1t 0.04, lbzip2 2t 0.22, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.51 | 0.02 | 0.08 | 0.189 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2988 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.88 | 10.7% |
| MTF + RLE2 | 1.28 | 15.7% |
| T-vector build | 1.17 | 14.3% |
| walk (the one each build uses) | 4.54 | 55.6% |
| of which: segment assembly | 0.23 | 2.9% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.10 | 1.3% |
| **total** | 8.17 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.368 | 0.924 | (in its total) | 2.51× (2.51–2.52) |  |
| walk | 1.434 | 2.719 | (in its total) | 1.90× (1.90–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.072 | (in its total) | 0.91× (0.90–0.91) |  |
| inverse BWT, total | 1.881 | 3.715 | 1.968 | 1.98× (1.97–1.98) | 1.05× (1.05–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 58m36s ± 6m43s |
| 4 threads | 27m42s ± 3m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m22s ± 2m37s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 18m22s ± 2m37s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.41 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,391,318 | 12,161,387 | \`09626dbf7dffd3f2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 2987.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
