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

The latest record: [`3df195b`](<https://github.com/lmmx/mibz2/commit/3df195b10e9fb6f86f0c06ad9fad1feb992406d5>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44455734475>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.507 s | 414 | 5.9 | 3.03× (3.01–3.04) | 6.89× (6.86–6.91) | 7.09× (7.06–7.13) | 7.69× (7.65–7.73) | 6.91× (6.88–6.95) | 7.55× (7.52–7.57) | 6.93× (6.91–6.95) | 2.22× (2.21–2.23) | 2.59× (2.57–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.293 s | 715 | 6.6 | 2.87× (2.75–2.99) | 6.24× (5.83–6.68) | 7.02× (6.61–7.46) | 6.82× (6.46–7.21) | 6.17× (5.84–6.52) | 12.55× (11.85–13.28) | 11.52× (10.89–12.20) | 3.69× (3.49–3.91) | 4.31× (4.07–4.56) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 858 | 10.2 | 2.95× (2.91–3.00) | 4.50× (4.42–4.59) | 5.61× (5.46–5.76) | 5.06× (5.01–5.12) | 5.06× (4.98–5.14) | 15.63× (15.43–15.84) | 14.36× (14.17–14.55) | 4.60× (4.55–4.65) | 5.37× (5.27–5.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 966 | 2.8 | 0.60× (0.59–0.61) | 3.64× (3.46–3.82) | 4.36× (4.30–4.42) | 3.53× (3.48–3.57) | 6.11× (6.03–6.20) | 2.94× (2.90–2.98) | 2.63× (2.59–2.67) | 1.01× (0.99–1.02) | 1.48× (1.46–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 971 | 2.8 | 0.58× (0.56–0.59) | 2.96× (2.88–3.04) | 6.01× (5.86–6.15) | 3.54× (3.46–3.62) | 6.02× (5.88–6.17) | 2.93× (2.85–3.01) | 2.61× (2.54–2.69) | 1.00× (0.97–1.03) | 1.48× (1.44–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 943 | 2.8 | 0.57× (0.56–0.58) | 2.92× (2.87–2.97) | 5.93× (5.81–6.05) | 3.48× (3.41–3.54) | 5.94× (5.83–6.06) | 2.88× (2.83–2.93) | 2.57× (2.52–2.62) | 0.98× (0.95–1.02) | 1.45× (1.43–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 570 | 4.5 | 2.59× (2.57–2.61) | 2.97× (2.94–2.99) | 3.86× (3.74–3.98) | 3.77× (3.73–3.81) | 4.08× (4.04–4.12) | 2.50× (2.49–2.52) | 2.07× (2.06–2.09) | 1.99× (1.97–2.01) | 2.95× (2.93–2.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 850 | 5.5 | 2.09× (1.98–2.22) | 2.66× (2.42–2.92) | 3.78× (3.58–3.99) | 3.15× (2.98–3.34) | 4.51× (4.28–4.74) | 3.60× (3.42–3.79) | 2.98× (2.83–3.15) | 2.87× (2.72–3.02) | 4.25× (4.04–4.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 766 | 11.1 | 1.86× (1.80–1.91) | 1.88× (1.75–2.03) | 2.93× (2.79–3.07) | 2.12× (2.05–2.19) | 3.87× (3.78–3.95) | 3.38× (3.31–3.44) | 2.80× (2.74–2.86) | 2.69× (2.64–2.74) | 3.98× (3.90–4.07) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 60 | 41.7 | 1.34× (1.32–1.35) | 2.66× (2.62–2.70) | 2.39× (2.36–2.43) | 2.46× (2.44–2.48) | 2.80× (2.77–2.83) | 3.07× (3.04–3.11) | 3.05× (3.03–3.08) | 1.19× (1.17–1.21) | 1.37× (1.35–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.085 s | 95 | 47.2 | 1.24× (1.23–1.26) | 2.48× (2.37–2.59) | 2.28× (2.22–2.33) | 2.95× (2.87–3.03) | 2.82× (2.76–2.89) | 4.86× (4.75–4.98) | 4.83× (4.72–4.95) | 1.89× (1.84–1.94) | 2.17× (2.11–2.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 64.1 | 1.30× (1.27–1.34) | 2.08× (2.03–2.13) | 2.03× (1.94–2.13) | 2.51× (2.44–2.57) | 2.80× (2.74–2.86) | 5.89× (5.74–6.05) | 5.85× (5.72–5.99) | 2.29× (2.23–2.34) | 2.62× (2.54–2.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 38.3 | 1.46× (1.43–1.48) | 2.98× (2.92–3.05) | 2.50× (2.45–2.54) | 2.59× (2.54–2.65) | 2.99× (2.95–3.04) | 3.52× (3.46–3.57) | 3.75× (3.69–3.81) | 1.37× (1.34–1.40) | 1.65× (1.61–1.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 43.3 | 1.37× (1.35–1.40) | 2.74× (2.66–2.83) | 2.48× (2.41–2.54) | 3.47× (3.37–3.57) | 3.09× (3.02–3.16) | 5.91× (5.83–5.98) | 6.29× (6.22–6.37) | 2.30× (2.27–2.33) | 2.77× (2.73–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 62.4 | 1.41× (1.37–1.47) | 2.24× (2.15–2.32) | 2.18× (2.10–2.25) | 2.91× (2.83–2.99) | 2.78× (2.68–2.89) | 6.67× (6.52–6.82) | 7.10× (6.91–7.29) | 2.59× (2.54–2.65) | 3.13× (3.03–3.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 3.9 | 0.65× (0.62–0.68) | 2.99× (2.79–3.21) | 4.27× (4.09–4.45) | 3.03× (2.91–3.15) | 6.65× (6.37–6.93) | 2.47× (2.36–2.58) | 2.25× (2.15–2.35) | 0.96× (0.91–1.01) | 1.47× (1.41–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 683 | 4.2 | 0.57× (0.55–0.59) | 2.81× (2.77–2.86) | 4.87× (4.74–5.01) | 2.96× (2.90–3.02) | 6.38× (6.28–6.48) | 2.36× (2.31–2.42) | 2.16× (2.12–2.19) | 0.92× (0.89–0.95) | 1.41× (1.39–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 673 | 4.2 | 0.56× (0.54–0.57) | 2.76× (2.72–2.81) | 4.72× (4.64–4.81) | 2.88× (2.83–2.94) | 6.23× (6.14–6.32) | 2.32× (2.29–2.34) | 2.11× (2.08–2.15) | 0.90× (0.88–0.93) | 1.38× (1.36–1.41) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.070 s | 187 | 13.4 | 1.98× (1.96–2.00) | 4.19× (4.12–4.25) | 4.45× (4.41–4.50) | 4.54× (4.49–4.59) | 4.75× (4.70–4.80) | 5.27× (5.21–5.32) | 4.80× (4.76–4.84) | 1.52× (1.50–1.53) | 1.88× (1.86–1.90) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.049 s | 265 | 17.1 | 1.58× (1.48–1.68) | 3.45× (3.26–3.65) | 3.83× (3.58–4.11) | 4.08× (3.82–4.37) | 4.21× (3.94–4.51) | 7.30× (6.84–7.80) | 6.66× (6.23–7.11) | 2.11× (1.96–2.26) | 2.60× (2.44–2.78) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.045 s | 293 | 26.3 | 1.58× (1.52–1.65) | 2.40× (2.30–2.50) | 3.18× (3.07–3.30) | 3.20× (3.14–3.28) | 3.74× (3.65–3.83) | 8.26× (8.08–8.45) | 7.53× (7.37–7.69) | 2.38× (2.34–2.43) | 2.94× (2.88–3.01) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.02× (3.00–3.03) | 6.85× (6.83–6.87) | 7.06× (7.03–7.09) | 7.66× (7.62–7.70) | 6.89× (6.86–6.92) | 7.52× (7.49–7.54) | 6.91× (6.89–6.92) | 2.22× (2.20–2.23) | 2.58× (2.56–2.60) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.85× (2.83–2.87) | 6.38× (6.31–6.45) | 6.72× (6.69–6.76) | 7.32× (7.27–7.38) | 6.31× (6.28–6.35) | 6.81× (6.77–6.86) | 6.26× (6.21–6.30) | 2.01× (1.99–2.02) | 2.33× (2.31–2.36) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.92× (2.88–2.95) | 4.89× (4.81–4.97) | 5.29× (5.22–5.36) | 5.52× (5.42–5.62) | 4.86× (4.80–4.93) | 4.39× (4.32–4.46) | 4.03× (3.98–4.09) | 1.29× (1.27–1.32) | 1.51× (1.49–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.64) | 3.23× (3.19–3.27) | 4.02× (3.98–4.07) | 3.30× (3.26–3.34) | 5.66× (5.58–5.73) | 2.77× (2.73–2.80) | 2.48× (2.45–2.52) | 1.01× (0.99–1.02) | 1.03× (1.02–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.05× (2.97–3.13) | 5.33× (5.23–5.44) | 3.28× (3.22–3.35) | 5.48× (5.37–5.60) | 2.71× (2.64–2.77) | 2.43× (2.36–2.49) | 0.98× (0.95–1.01) | 1.01× (0.99–1.03) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.36× (3.31–3.41) | 5.34× (5.27–5.41) | 3.29× (3.24–3.35) | 5.51× (5.43–5.59) | 2.72× (2.68–2.76) | 2.44× (2.40–2.48) | 0.99× (0.96–1.01) | 1.02× (1.00–1.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.48–2.52) | 2.73× (2.71–2.76) | 3.48× (3.39–3.59) | 3.61× (3.58–3.65) | 3.91× (3.87–3.95) | 2.42× (2.40–2.44) | 2.01× (2.00–2.03) | 1.94× (1.92–1.95) | 2.66× (2.63–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.21× (2.18–2.25) | 2.48× (2.42–2.56) | 3.01× (2.96–3.06) | 3.09× (3.03–3.15) | 3.31× (3.24–3.39) | 1.97× (1.93–2.01) | 1.63× (1.60–1.67) | 1.57× (1.54–1.61) | 2.16× (2.12–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.85–2.06) | 1.72× (1.67–1.77) | 2.18× (2.11–2.26) | 2.00× (1.92–2.08) | 2.05× (1.99–2.11) | 1.01× (0.98–1.03) | 0.84× (0.81–0.86) | 0.81× (0.78–0.83) | 1.11× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.35) | 2.60× (2.56–2.63) | 2.32× (2.29–2.36) | 2.44× (2.42–2.45) | 2.77× (2.74–2.80) | 3.04× (3.01–3.08) | 3.02× (3.00–3.05) | 1.19× (1.17–1.21) | 1.34× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.23–1.27) | 2.42× (2.38–2.45) | 2.13× (2.10–2.16) | 2.84× (2.80–2.88) | 2.52× (2.48–2.55) | 2.69× (2.65–2.72) | 2.67× (2.63–2.71) | 1.05× (1.03–1.07) | 1.18× (1.16–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.44–1.51) | 2.26× (2.23–2.28) | 1.83× (1.80–1.85) | 2.54× (2.50–2.58) | 2.40× (2.36–2.45) | 1.99× (1.97–2.02) | 1.98× (1.95–2.01) | 0.78× (0.77–0.79) | 0.88× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.43–1.47) | 2.93× (2.88–2.99) | 2.44× (2.40–2.49) | 2.57× (2.52–2.62) | 2.97× (2.92–3.02) | 3.49× (3.43–3.54) | 3.71× (3.65–3.77) | 1.37× (1.34–1.40) | 1.61× (1.59–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.40) | 2.71× (2.68–2.74) | 2.26× (2.24–2.29) | 3.16× (3.13–3.19) | 2.72× (2.69–2.75) | 3.11× (3.08–3.14) | 3.31× (3.27–3.35) | 1.22× (1.20–1.24) | 1.44× (1.42–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.66) | 2.50× (2.45–2.55) | 1.92× (1.86–1.98) | 2.75× (2.67–2.83) | 2.48× (2.41–2.56) | 2.20× (2.14–2.26) | 2.34× (2.28–2.40) | 0.86× (0.84–0.88) | 1.02× (0.99–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.71× (2.66–2.76) | 3.54× (3.48–3.61) | 2.82× (2.78–2.87) | 6.05× (5.95–6.14) | 2.32× (2.28–2.37) | 2.13× (2.10–2.17) | 0.99× (0.96–1.02) | 0.99× (0.95–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.65) | 2.46× (2.42–2.50) | 4.21× (4.15–4.28) | 2.73× (2.68–2.78) | 5.62× (5.53–5.71) | 2.15× (2.10–2.19) | 1.97× (1.94–2.00) | 0.91× (0.88–0.95) | 0.91× (0.87–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.59–0.62) | 2.68× (2.64–2.71) | 4.15× (4.08–4.22) | 2.73× (2.68–2.78) | 5.62× (5.54–5.70) | 2.15× (2.13–2.17) | 1.98× (1.94–2.01) | 0.91× (0.89–0.94) | 0.92× (0.87–0.96) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.96× (1.94–1.98) | 3.98× (3.94–4.03) | 4.31× (4.27–4.35) | 4.44× (4.39–4.49) | 4.64× (4.60–4.69) | 5.15× (5.09–5.20) | 4.69× (4.65–4.73) | 1.50× (1.49–1.52) | 1.80× (1.78–1.82) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.66× (1.62–1.69) | 3.31× (3.25–3.38) | 3.61× (3.54–3.68) | 4.06× (3.98–4.14) | 3.78× (3.71–3.85) | 4.07× (3.99–4.15) | 3.71× (3.64–3.78) | 1.19× (1.17–1.21) | 1.42× (1.40–1.45) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.75× (1.68–1.82) | 2.61× (2.55–2.67) | 2.86× (2.82–2.90) | 3.16× (3.10–3.21) | 2.97× (2.92–3.03) | 2.63× (2.58–2.68) | 2.40× (2.36–2.44) | 0.77× (0.75–0.78) | 0.92× (0.90–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.28× (0.27–0.30) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.97–11.03) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.42) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.35) | 0.47× (0.40–0.55) | 0.69× (0.66–0.72) | 0.58× (0.55–0.60) | 5.44× (5.23–5.66) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.67× (0.60–0.75) | 0.60× (0.57–0.62) | 0.52× (0.50–0.54) | 3.45× (3.30–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.02–8.05) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.73× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.69× (0.62–0.76) | 1.62× (1.58–1.66) | 0.81× (0.79–0.84) | 5.87× (5.72–6.02) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.53× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.67) | 0.71× (0.68–0.73) | 1.17× (1.14–1.21) | 0.72× (0.70–0.73) | 3.43× (3.35–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.17× (5.15–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.44) | 0.90× (0.87–0.93) | 0.73× (0.71–0.75) | 3.12× (3.02–3.22) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.82× (0.79–0.86) | 2.57× (2.52–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.34× (0.33–0.34) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.50) | 0.38× (0.35–0.41) | 0.85× (0.81–0.88) | 0.71× (0.67–0.75) | 2.90× (2.78–3.03) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.51× (0.48–0.53) | 0.81× (0.79–0.82) | 0.83× (0.79–0.87) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.42–1.43) | 0.66× (0.66–0.67) | 6.45× (6.44–6.47) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.62–0.64) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 37 MB | 0.46× (0.45–0.47) | 0.43× (0.42–0.44) | 1.06× (1.04–1.08) | 0.70× (0.68–0.73) | 3.38× (3.31–3.46) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.50× (0.47–0.54) | 0.91× (0.88–0.93) | 0.74× (0.70–0.77) | 2.32× (2.26–2.37) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 414 | 2.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 715 | 2.68 | 1.10× (1.10–1.11) | 83% | 1.03× (1.02–1.04) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 858 | 4.17 | 1.71× (1.69–1.74) | 52% | 1.22× (1.21–1.23) | 1.65× (1.63–1.68) |

**Best:** mibz2 at 4 threads (858 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (289 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.20, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.67 | 0.02 | 0.09 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.4% |
| MTF + RLE2 | 1.20 | 17.5% |
| T-vector build | 1.14 | 16.7% |
| walk (the one each build uses) | 3.22 | 47.3% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.8% |
| **total** | 6.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.306 | (in its total) | 2.66× (2.64–2.67) |  |
| walk | 1.260 | 2.686 | (in its total) | 2.13× (2.12–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.057 | (in its total) | 1.20× (1.19–1.21) |  |
| inverse BWT, total | 1.800 | 4.049 | 3.305 | 2.25× (2.24–2.26) | 1.83× (1.83–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m20s |
| 4 threads | 35m36s ± 5m01s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m14s ± 3m26s |
| the scanner alone (its serial part) | 14m16s ± 26.3s |
| the workers alone, over 4 | 24m13s ± 3m26s |

At 4 threads the workers bound it. Per byte: the scanner 8.3 ns a byte of bz2, the workers 3.18 ns a byte of output at 4 threads (CPU, summed over them), 1.95 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
