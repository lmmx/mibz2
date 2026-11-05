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

The latest record: [`0892322`](<https://github.com/lmmx/mibz2/commit/0892322c02fe9d82d8746ffa97765e27943fd5a5>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41966233769>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.586 s | 358 | 10.3 | 2.11× (2.09–2.13) | 4.92× (4.86–4.98) | 5.09× (5.04–5.13) | 5.84× (5.75–5.93) | 5.09× (5.00–5.18) | 5.70× (5.64–5.78) | 5.20× (5.03–5.38) | 1.58× (1.55–1.61) | 1.76× (1.74–1.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.389 s | 539 | 11.4 | 1.89× (1.84–1.96) | 3.91× (3.70–4.13) | 4.53× (4.35–4.72) | 4.66× (4.48–4.85) | 4.09× (3.96–4.22) | 8.54× (8.25–8.85) | 7.79× (7.41–8.19) | 2.37× (2.26–2.48) | 2.64× (2.54–2.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.312 s | 672 | 17.2 | 1.97× (1.92–2.01) | 2.91× (2.81–3.02) | 3.68× (3.54–3.82) | 3.68× (3.47–3.92) | 3.51× (3.40–3.61) | 10.65× (10.31–11.01) | 9.71× (9.35–10.09) | 2.95× (2.85–3.07) | 3.29× (3.17–3.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,096 | 3.7 | 0.57× (0.56–0.58) | 3.36× (3.32–3.41) | 4.49× (4.43–4.54) | 3.50× (3.46–3.54) | 6.07× (5.96–6.17) | 2.62× (2.59–2.66) | 2.60× (2.57–2.63) | 0.98× (0.96–1.00) | 1.89× (1.70–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,078 | 3.8 | 0.52× (0.51–0.53) | 2.78× (2.71–2.85) | 5.55× (5.41–5.69) | 3.47× (3.39–3.55) | 5.88× (5.73–6.03) | 2.59× (2.53–2.65) | 2.57× (2.50–2.64) | 0.97× (0.93–1.00) | 1.86× (1.66–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,058 | 3.8 | 0.52× (0.51–0.53) | 2.74× (2.68–2.81) | 5.46× (5.33–5.60) | 3.43× (3.36–3.50) | 5.84× (5.68–6.01) | 2.55× (2.49–2.61) | 2.53× (2.47–2.59) | 0.95× (0.92–0.99) | 1.83× (1.65–2.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 661 | 5.9 | 2.40× (2.38–2.43) | 2.76× (2.73–2.78) | 3.54× (3.33–3.76) | 3.67× (3.63–3.71) | 4.09× (4.05–4.13) | 3.07× (3.04–3.10) | 2.03× (2.01–2.04) | 1.90× (1.88–1.92) | 2.74× (2.72–2.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 954 | 7.2 | 1.89× (1.70–2.10) | 2.39× (2.18–2.63) | 3.80× (3.43–4.21) | 3.00× (2.72–3.31) | 4.24× (3.84–4.68) | 4.24× (3.86–4.66) | 2.80× (2.55–3.07) | 2.62× (2.38–2.88) | 3.78× (3.45–4.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 895 | 14.0 | 1.73× (1.70–1.77) | 1.77× (1.64–1.91) | 2.84× (2.68–3.01) | 2.01× (1.93–2.10) | 3.75× (3.61–3.89) | 4.13× (4.01–4.26) | 2.72× (2.65–2.81) | 2.55× (2.48–2.62) | 3.69× (3.58–3.80) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.122 s | 65 | 57.0 | 1.27× (1.26–1.28) | 2.41× (2.39–2.44) | 2.20× (2.15–2.24) | 2.28× (2.26–2.31) | 2.53× (2.51–2.55) | 2.88× (2.85–2.90) | 2.84× (2.81–2.86) | 1.15× (1.14–1.16) | 1.30× (1.28–1.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.075 s | 106 | 63.2 | 1.17× (1.12–1.22) | 2.25× (2.08–2.43) | 2.16× (2.04–2.30) | 2.77× (2.63–2.91) | 2.59× (2.47–2.71) | 4.61× (4.41–4.83) | 4.55× (4.35–4.76) | 1.84× (1.76–1.92) | 2.09× (2.00–2.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 78.1 | 1.29× (1.24–1.33) | 2.11× (2.02–2.20) | 2.10× (1.99–2.22) | 2.58× (2.49–2.67) | 2.85× (2.77–2.93) | 6.08× (5.91–6.25) | 6.00× (5.83–6.17) | 2.42× (2.35–2.50) | 2.75× (2.67–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.158 s | 61 | 61.2 | 1.16× (1.13–1.19) | 2.31× (2.26–2.36) | 2.03× (1.99–2.07) | 2.12× (2.08–2.16) | 2.33× (2.28–2.39) | 3.02× (2.95–3.09) | 3.06× (2.99–3.14) | 1.19× (1.16–1.23) | 1.39× (1.34–1.43) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.094 s | 102 | 66.4 | 1.06× (1.02–1.11) | 2.15× (2.06–2.23) | 2.00× (1.92–2.08) | 2.73× (2.63–2.85) | 2.38× (2.28–2.47) | 5.05× (4.83–5.27) | 5.12× (4.92–5.32) | 1.99× (1.89–2.10) | 2.32× (2.20–2.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 135 | 84.9 | 1.25× (1.22–1.27) | 2.01× (1.91–2.12) | 2.04× (1.99–2.10) | 2.74× (2.66–2.82) | 2.55× (2.48–2.61) | 6.72× (6.60–6.83) | 6.80× (6.69–6.92) | 2.65× (2.58–2.72) | 3.09× (3.02–3.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 823 | 5.1 | 0.61× (0.61–0.61) | 3.38× (3.36–3.40) | 4.24× (4.21–4.28) | 3.04× (3.02–3.07) | 6.60× (6.51–6.70) | 2.30× (2.28–2.31) | 2.27× (2.26–2.29) | 0.95× (0.93–0.96) | 1.70× (1.69–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 5.5 | 0.50× (0.49–0.51) | 2.39× (2.35–2.44) | 4.80× (4.71–4.89) | 2.87× (2.82–2.93) | 6.20× (6.11–6.30) | 2.15× (2.11–2.19) | 2.13× (2.09–2.17) | 0.89× (0.87–0.91) | 1.60× (1.57–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 772 | 5.3 | 0.50× (0.49–0.52) | 2.39× (2.34–2.43) | 4.77× (4.68–4.86) | 2.88× (2.83–2.92) | 6.13× (5.96–6.31) | 2.14× (2.11–2.17) | 2.12× (2.08–2.16) | 0.88× (0.87–0.90) | 1.59× (1.56–1.62) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 0.066 s | 185 | 20.4 | 1.37× (1.31–1.44) | 2.89× (2.74–3.04) | 3.20× (3.03–3.37) | 3.36× (3.19–3.53) | 3.58× (3.40–3.76) | 3.51× (3.33–3.69) | 3.42× (3.25–3.60) | 1.14× (1.08–1.20) | 1.33× (1.26–1.41) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 0.043 s | 280 | 23.6 | 1.18× (1.12–1.25) | 2.85× (2.21–3.67) | 3.03× (2.85–3.23) | 3.49× (3.32–3.66) | 3.50× (3.32–3.69) | 5.47× (5.20–5.76) | 5.34× (5.08–5.60) | 1.78× (1.68–1.88) | 2.08× (1.97–2.20) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 0.034 s | 355 | 34.3 | 1.33× (1.30–1.37) | 2.30× (2.13–2.50) | 2.86× (2.70–3.04) | 3.26× (3.16–3.36) | 3.69× (3.48–3.90) | 6.87× (6.70–7.04) | 6.69× (6.51–6.89) | 2.23× (2.15–2.31) | 2.61× (2.52–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.11× (2.09–2.13) | 4.90× (4.83–4.96) | 5.07× (5.03–5.12) | 5.82× (5.73–5.92) | 5.08× (4.99–5.17) | 5.69× (5.62–5.76) | 5.19× (5.01–5.37) | 1.58× (1.55–1.61) | 1.75× (1.73–1.77) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.07× (2.05–2.10) | 4.57× (4.46–4.69) | 4.83× (4.73–4.93) | 5.50× (5.38–5.61) | 4.64× (4.56–4.72) | 5.14× (5.03–5.25) | 4.69× (4.52–4.87) | 1.43× (1.39–1.47) | 1.58× (1.54–1.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.19× (2.14–2.24) | 3.59× (3.49–3.70) | 4.04× (3.90–4.18) | 4.48× (4.29–4.67) | 3.84× (3.72–3.97) | 3.37× (3.28–3.47) | 3.07× (2.98–3.17) | 0.94× (0.91–0.97) | 1.04× (1.00–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.88× (2.84–2.91) | 3.86× (3.81–3.90) | 3.28× (3.25–3.31) | 5.61× (5.52–5.70) | 2.48× (2.45–2.51) | 2.46× (2.43–2.49) | 0.98× (0.96–1.00) | 1.61× (1.59–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.67× (2.61–2.74) | 4.87× (4.76–4.99) | 3.23× (3.16–3.29) | 5.38× (5.25–5.50) | 2.41× (2.36–2.47) | 2.39× (2.34–2.45) | 0.95× (0.92–0.98) | 1.57× (1.54–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.95× (2.89–3.01) | 4.90× (4.80–5.00) | 3.27× (3.21–3.33) | 5.45× (5.32–5.59) | 2.43× (2.38–2.49) | 2.41× (2.36–2.46) | 0.96× (0.93–0.99) | 1.58× (1.55–1.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.32× (2.30–2.34) | 2.56× (2.54–2.57) | 3.17× (2.99–3.37) | 3.52× (3.48–3.55) | 3.92× (3.88–3.96) | 2.96× (2.93–2.98) | 1.97× (1.95–1.98) | 1.85× (1.83–1.87) | 2.40× (2.38–2.42) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.03–2.15) | 2.29× (2.25–2.33) | 3.09× (3.03–3.16) | 3.03× (2.97–3.10) | 3.27× (3.20–3.35) | 2.42× (2.36–2.48) | 1.61× (1.57–1.65) | 1.51× (1.48–1.54) | 1.96× (1.92–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.83–1.99) | 1.58× (1.51–1.65) | 2.09× (2.01–2.17) | 1.95× (1.86–2.04) | 1.96× (1.87–2.05) | 1.24× (1.19–1.30) | 0.83× (0.80–0.86) | 0.78× (0.75–0.81) | 1.01× (0.97–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.27× (1.26–1.28) | 2.34× (2.32–2.36) | 2.16× (2.12–2.19) | 2.27× (2.24–2.29) | 2.51× (2.49–2.53) | 2.85× (2.83–2.87) | 2.81× (2.79–2.83) | 1.14× (1.14–1.15) | 1.24× (1.24–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.20× (1.18–1.21) | 2.25× (2.23–2.26) | 2.00× (1.97–2.03) | 2.71× (2.67–2.74) | 2.34× (2.30–2.37) | 2.58× (2.55–2.60) | 2.54× (2.52–2.57) | 1.03× (1.02–1.05) | 1.13× (1.11–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.43–1.50) | 2.30× (2.25–2.35) | 1.94× (1.89–1.98) | 2.62× (2.58–2.65) | 2.50× (2.45–2.54) | 2.08× (2.05–2.11) | 2.05× (2.02–2.08) | 0.83× (0.82–0.85) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.16× (1.13–1.19) | 2.26× (2.20–2.32) | 1.98× (1.94–2.03) | 2.11× (2.07–2.15) | 2.32× (2.26–2.38) | 3.00× (2.93–3.07) | 3.04× (2.97–3.11) | 1.19× (1.16–1.23) | 1.35× (1.31–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.11× (1.11–1.12) | 2.19× (2.17–2.21) | 1.91× (1.90–1.92) | 2.66× (2.63–2.69) | 2.19× (2.17–2.22) | 2.77× (2.74–2.80) | 2.81× (2.79–2.82) | 1.10× (1.08–1.12) | 1.24× (1.23–1.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.41× (1.39–1.44) | 2.25× (2.18–2.33) | 1.80× (1.75–1.85) | 2.52× (2.45–2.59) | 2.30× (2.26–2.35) | 2.17× (2.15–2.20) | 2.20× (2.17–2.23) | 0.86× (0.85–0.88) | 0.97× (0.96–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.66) | 2.42× (2.39–2.45) | 3.36× (3.32–3.40) | 2.78× (2.76–2.80) | 5.88× (5.80–5.97) | 2.12× (2.11–2.14) | 2.11× (2.10–2.12) | 0.95× (0.94–0.97) | 1.33× (1.32–1.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.19× (2.16–2.22) | 3.96× (3.87–4.05) | 2.66× (2.63–2.70) | 5.50× (5.43–5.57) | 1.97× (1.95–2.00) | 1.96× (1.93–1.99) | 0.89× (0.87–0.90) | 1.24× (1.22–1.25) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.45× (2.40–2.50) | 3.99× (3.93–4.05) | 2.75× (2.70–2.79) | 5.61× (5.46–5.75) | 2.03× (1.99–2.06) | 2.01× (1.98–2.05) | 0.91× (0.89–0.93) | 1.27× (1.25–1.29) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 1.36× (1.30–1.43) | 2.79× (2.65–2.93) | 3.08× (2.93–3.24) | 3.29× (3.13–3.46) | 3.51× (3.34–3.69) | 3.44× (3.27–3.62) | 3.36× (3.20–3.53) | 1.13× (1.08–1.19) | 1.23× (1.17–1.29) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 1.31× (1.29–1.33) | 2.71× (2.44–3.01) | 2.91× (2.86–2.97) | 3.28× (3.25–3.31) | 3.19× (3.15–3.22) | 3.05× (3.00–3.09) | 2.97× (2.94–3.01) | 1.01× (0.99–1.02) | 1.09× (1.07–1.11) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 1.51× (1.47–1.55) | 2.37× (2.22–2.53) | 2.68× (2.57–2.79) | 2.81× (2.74–2.89) | 2.88× (2.73–3.03) | 2.08× (2.03–2.13) | 2.03× (1.98–2.08) | 0.69× (0.67–0.70) | 0.75× (0.72–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.35–0.39) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.23× (12.21–12.25) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 46 MB | 0.40× (0.39–0.41) | 0.47× (0.42–0.53) | 0.85× (0.83–0.87) | 0.72× (0.70–0.73) | 6.78× (6.64–6.93) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.26× (0.25–0.26) | 0.13× (0.13–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 69 MB | 0.44× (0.43–0.44) | 0.71× (0.61–0.83) | 0.78× (0.77–0.80) | 0.69× (0.67–0.71) | 4.60× (4.53–4.67) | 0.10× (0.10–0.11) | 0.11× (0.11–0.11) | 0.17× (0.17–0.17) | 0.09× (0.09–0.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.33–4.33) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.18× (8.16–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.67–0.68) | 0.67× (0.62–0.71) | 1.66× (1.65–1.68) | 0.82× (0.81–0.82) | 6.02× (5.97–6.07) | 0.48× (0.47–0.48) | 0.50× (0.50–0.51) | 0.55× (0.54–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.71× (0.65–0.78) | 1.18× (1.16–1.21) | 0.77× (0.75–0.80) | 3.49× (3.42–3.55) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 35 MB | 0.54× (0.52–0.55) | 0.45× (0.43–0.47) | 0.98× (0.96–1.01) | 0.78× (0.75–0.80) | 3.40× (3.33–3.48) | 0.21× (0.20–0.21) | 0.22× (0.21–0.22) | 0.34× (0.33–0.35) | 0.17× (0.17–0.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.64) | 0.45× (0.45–0.46) | 0.86× (0.85–0.87) | 0.80× (0.78–0.82) | 2.50× (2.46–2.54) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.28× (5.27–5.30) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.39× (0.35–0.43) | 0.92× (0.90–0.94) | 0.76× (0.73–0.80) | 3.16× (3.09–3.24) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.61) | 0.47× (0.45–0.49) | 0.80× (0.79–0.82) | 0.80× (0.77–0.84) | 2.32× (2.27–2.36) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 22 MB | 0.37× (0.37–0.37) | 0.24× (0.24–0.25) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.20–0.21) | 1.65× (1.64–1.65) | 0.78× (0.77–0.78) | 5.44× (5.42–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.78) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.41× (0.41–0.41) | 1.51× (1.51–1.52) | 0.65× (0.65–0.66) | 6.81× (6.80–6.82) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.66–0.67) | 0.33× (0.33–0.34) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 31 MB | 0.52× (0.51–0.52) | 0.43× (0.40–0.46) | 1.27× (1.24–1.29) | 0.77× (0.73–0.80) | 4.01× (3.96–4.05) | 0.24× (0.24–0.24) | 0.25× (0.25–0.26) | 0.39× (0.39–0.40) | 0.20× (0.19–0.20) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 48 MB | 0.55× (0.54–0.56) | 0.52× (0.48–0.57) | 1.05× (1.04–1.07) | 0.83× (0.81–0.85) | 2.67× (2.63–2.71) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 358 | 2.80 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 539 | 3.10 | 1.11× (1.09–1.12) | 75% | 1.03× (1.00–1.07) | 1.09× (1.07–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 672 | 4.70 | 1.69× (1.63–1.74) | 47% | 1.24× (1.21–1.26) | 1.75× (1.69–1.82) |

**Best:** mibz2 at 4 threads (672 MB/s); lbzip2 at 4 threads (228 MB/s); pbz2 at 4 threads (340 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.72, mibz2 4t 1.02, lbzip2 1t 0.07, lbzip2 2t 0.20, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.81 | 0.02 | 0.07 | 0.312 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3670 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.16 | 12.0% |
| MTF + RLE2 | 1.98 | 20.5% |
| T-vector build | 1.77 | 18.4% |
| walk (the one each build uses) | 4.29 | 44.4% |
| of which: segment assembly | 0.16 | 1.6% |
| RLE1 | 0.20 | 2.1% |
| CRC | 0.25 | 2.6% |
| **total** | 9.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.453 | 1.063 | (in its total) | 2.34× (2.33–2.35) |  |
| walk | 1.107 | 2.024 | (in its total) | 1.82× (1.80–1.83) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.040 | (in its total) | 1.06× (1.04–1.08) |  |
| inverse BWT, total | 1.598 | 3.127 | 1.871 | 1.95× (1.93–1.96) | 1.17× (1.16–1.18) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h24m ± 10m16s |
| 4 threads | 45m33s ± 6m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m32s ± 4m21s |
| the scanner alone (its serial part) | 12m05s ± 30.2s |
| the workers alone, over 4 | 29m32s ± 4m21s |

At 4 threads the workers bound it. Per byte: the scanner 7.0 ns a byte of bz2, the workers 3.87 ns a byte of output at 4 threads (CPU, summed over them), 2.39 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,664,767 | 12,147,816 | \`95a0f4fad2b4a029…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3669.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
