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

The latest record: [`2fe9557`](<https://github.com/lmmx/mibz2/commit/2fe9557a8ca923cdd382227c4c873cacb4b515ff>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44290297997>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.418 s | 501 | 7.4 | 2.94× (2.91–2.97) | 6.75× (6.69–6.81) | 6.99× (6.91–7.08) | 7.90× (7.85–7.96) | 6.99× (6.91–7.07) | 7.30× (7.25–7.36) | 7.04× (6.98–7.11) | 2.17× (2.15–2.18) | 2.42× (2.40–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.242 s | 868 | 8.1 | 3.00× (2.84–3.18) | 6.05× (5.79–6.33) | 6.95× (6.67–7.26) | 7.22× (6.86–7.60) | 6.38× (6.08–6.70) | 12.46× (11.93–13.02) | 12.02× (11.46–12.60) | 3.70× (3.53–3.87) | 4.13× (3.95–4.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.202 s | 1,041 | 12.5 | 2.91× (2.87–2.95) | 4.41× (4.31–4.52) | 5.47× (5.35–5.59) | 5.31× (5.17–5.46) | 5.22× (5.12–5.32) | 15.16× (14.90–15.43) | 14.62× (14.32–14.92) | 4.50× (4.42–4.58) | 5.03× (4.94–5.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,111 | 3.7 | 0.57× (0.56–0.57) | 3.40× (3.35–3.45) | 4.53× (4.46–4.60) | 3.55× (3.50–3.60) | 6.11× (6.05–6.18) | 2.64× (2.60–2.67) | 2.64× (2.60–2.67) | 1.03× (0.90–1.18) | 1.75× (1.63–1.88) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.53× (0.52–0.54) | 2.83× (2.76–2.91) | 5.66× (5.51–5.80) | 3.53× (3.46–3.61) | 5.90× (5.76–6.03) | 2.62× (2.56–2.69) | 2.62× (2.56–2.69) | 1.02× (0.88–1.18) | 1.74× (1.65–1.84) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,096 | 3.7 | 0.54× (0.52–0.56) | 2.80× (2.75–2.86) | 5.59× (5.48–5.71) | 3.51× (3.44–3.58) | 6.00× (5.91–6.09) | 2.59× (2.54–2.64) | 2.59× (2.54–2.64) | 1.01× (0.88–1.15) | 1.72× (1.61–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 745 | 5.3 | 2.77× (2.71–2.83) | 3.11× (3.10–3.13) | 3.95× (3.80–4.11) | 4.15× (4.12–4.18) | 4.56× (4.50–4.61) | 2.81× (2.79–2.82) | 2.30× (2.29–2.31) | 2.12× (2.11–2.13) | 3.10× (3.08–3.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,114 | 6.5 | 2.30× (2.11–2.51) | 2.91× (2.70–3.13) | 4.42× (4.11–4.76) | 3.59× (3.34–3.87) | 5.12× (4.81–5.44) | 4.09× (3.85–4.34) | 3.35× (3.15–3.56) | 3.09× (2.90–3.28) | 4.52× (4.25–4.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 963 | 13.0 | 1.83× (1.77–1.90) | 1.85× (1.69–2.04) | 3.04× (2.88–3.21) | 2.25× (2.11–2.39) | 4.13× (3.91–4.36) | 3.57× (3.46–3.69) | 2.93× (2.83–3.03) | 2.70× (2.62–2.79) | 3.95× (3.82–4.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.6 | 1.39× (1.38–1.40) | 2.69× (2.67–2.71) | 2.45× (2.41–2.49) | 2.52× (2.50–2.54) | 2.84× (2.81–2.87) | 3.03× (3.00–3.05) | 3.19× (3.17–3.22) | 1.26× (1.25–1.27) | 1.43× (1.40–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 117 | 58.3 | 1.30× (1.28–1.33) | 2.52× (2.37–2.67) | 2.40× (2.34–2.45) | 3.09× (3.04–3.14) | 2.88× (2.81–2.96) | 4.85× (4.75–4.96) | 5.12× (5.01–5.24) | 2.02× (1.98–2.07) | 2.29× (2.24–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 142 | 77.8 | 1.32× (1.30–1.34) | 2.19× (2.15–2.23) | 2.20× (2.11–2.30) | 2.61× (2.56–2.65) | 2.88× (2.82–2.94) | 5.89× (5.78–6.00) | 6.21× (6.11–6.32) | 2.45× (2.41–2.49) | 2.78× (2.70–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.2 | 1.53× (1.52–1.54) | 3.09× (3.06–3.12) | 2.67× (2.62–2.72) | 2.81× (2.79–2.84) | 3.12× (3.09–3.15) | 3.71× (3.67–3.74) | 4.14× (4.07–4.21) | 1.52× (1.50–1.54) | 1.81× (1.79–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 52.6 | 1.37× (1.34–1.39) | 2.83× (2.74–2.91) | 2.55× (2.49–2.61) | 3.60× (3.52–3.68) | 3.11× (3.06–3.16) | 6.05× (5.95–6.14) | 6.76× (6.61–6.91) | 2.48× (2.45–2.52) | 2.96× (2.90–3.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 74.5 | 1.39× (1.35–1.44) | 2.24× (2.16–2.31) | 2.23× (2.14–2.32) | 3.06× (2.98–3.14) | 2.86× (2.77–2.94) | 6.94× (6.74–7.14) | 7.75× (7.54–7.97) | 2.85× (2.77–2.92) | 3.39× (3.28–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 5.0 | 0.63× (0.62–0.64) | 3.50× (3.46–3.54) | 4.39× (4.34–4.44) | 3.14× (3.10–3.18) | 6.83× (6.71–6.95) | 2.31× (2.28–2.33) | 2.35× (2.32–2.38) | 0.96× (0.94–0.98) | 1.78× (1.74–1.83) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 810 | 5.3 | 0.54× (0.49–0.59) | 2.49× (2.45–2.54) | 5.00× (4.91–5.09) | 2.97× (2.93–3.02) | 6.45× (6.34–6.56) | 2.18× (2.14–2.22) | 2.22× (2.18–2.26) | 0.91× (0.89–0.93) | 1.69× (1.63–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 800 | 5.2 | 0.52× (0.51–0.54) | 2.47× (2.42–2.52) | 4.95× (4.86–5.04) | 2.96× (2.91–3.01) | 6.32× (6.18–6.45) | 2.16× (2.12–2.19) | 2.19× (2.15–2.23) | 0.90× (0.88–0.92) | 1.66× (1.60–1.73) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 0.056 s | 231 | 16.5 | 1.83× (1.81–1.84) | 4.02× (3.96–4.08) | 4.37× (4.32–4.42) | 4.60× (4.57–4.64) | 4.80× (4.77–4.83) | 4.75× (4.71–4.80) | 4.87× (4.84–4.90) | 1.51× (1.51–1.52) | 1.81× (1.80–1.83) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 0.038 s | 334 | 20.7 | 1.51× (1.34–1.70) | 3.21× (2.86–3.59) | 3.67× (3.35–4.02) | 4.24× (3.88–4.63) | 4.27× (3.90–4.68) | 6.65× (6.15–7.19) | 6.81× (6.27–7.40) | 2.12× (1.95–2.30) | 2.54× (2.34–2.76) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 0.034 s | 380 | 31.7 | 1.48× (1.43–1.54) | 2.43× (2.35–2.52) | 3.22× (3.10–3.34) | 3.49× (3.36–3.64) | 3.98× (3.89–4.08) | 7.76× (7.54–7.98) | 7.95× (7.72–8.18) | 2.47× (2.40–2.54) | 2.96× (2.87–3.06) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.90–2.96) | 6.72× (6.66–6.77) | 6.96× (6.87–7.04) | 7.87× (7.82–7.93) | 6.96× (6.89–7.04) | 7.28× (7.22–7.33) | 7.02× (6.95–7.08) | 2.16× (2.15–2.18) | 2.40× (2.38–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.82× (2.76–2.88) | 6.28× (6.22–6.35) | 6.55× (6.49–6.62) | 7.61× (7.54–7.68) | 6.41× (6.37–6.46) | 6.66× (6.62–6.70) | 6.42× (6.37–6.47) | 1.98× (1.97–1.99) | 2.20× (2.18–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.89× (2.82–2.96) | 4.89× (4.77–5.01) | 5.32× (5.22–5.42) | 5.92× (5.78–6.07) | 5.12× (5.05–5.20) | 4.34× (4.28–4.39) | 4.18× (4.13–4.23) | 1.29× (1.27–1.30) | 1.43× (1.41–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.60–0.61) | 2.90× (2.86–2.94) | 3.89× (3.78–4.00) | 3.32× (3.28–3.37) | 5.66× (5.59–5.72) | 2.49× (2.45–2.52) | 2.49× (2.46–2.53) | 1.03× (0.90–1.16) | 1.62× (1.60–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.73× (2.65–2.81) | 4.94× (4.86–5.03) | 3.27× (3.20–3.34) | 5.37× (5.26–5.49) | 2.43× (2.38–2.48) | 2.43× (2.38–2.49) | 1.00× (0.87–1.15) | 1.58× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.56–0.60) | 3.04× (2.99–3.10) | 4.97× (4.88–5.07) | 3.33× (3.27–3.39) | 5.58× (5.51–5.65) | 2.46× (2.41–2.50) | 2.46× (2.42–2.51) | 1.01× (0.89–1.15) | 1.60× (1.57–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.65× (2.59–2.71) | 2.86× (2.85–2.88) | 3.49× (3.33–3.65) | 3.95× (3.92–3.97) | 4.33× (4.28–4.38) | 2.69× (2.67–2.70) | 2.21× (2.20–2.22) | 2.05× (2.04–2.05) | 2.67× (2.64–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.25–2.37) | 2.56× (2.48–2.64) | 3.37× (3.28–3.46) | 3.35× (3.26–3.44) | 3.66× (3.56–3.75) | 2.17× (2.11–2.23) | 1.79× (1.74–1.84) | 1.65× (1.61–1.70) | 2.15× (2.09–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.02× (1.93–2.12) | 1.68× (1.60–1.77) | 2.24× (2.16–2.32) | 2.03× (1.97–2.09) | 2.20× (2.09–2.31) | 1.10× (1.06–1.13) | 0.90× (0.88–0.93) | 0.84× (0.81–0.86) | 1.09× (1.06–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.38–1.39) | 2.59× (2.58–2.61) | 2.39× (2.36–2.42) | 2.50× (2.48–2.51) | 2.81× (2.78–2.84) | 2.99× (2.97–3.01) | 3.16× (3.14–3.18) | 1.26× (1.25–1.26) | 1.38× (1.38–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.30) | 2.42× (2.38–2.45) | 2.15× (2.13–2.18) | 2.92× (2.88–2.97) | 2.53× (2.49–2.57) | 2.64× (2.61–2.68) | 2.79× (2.76–2.82) | 1.11× (1.10–1.12) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.43–1.49) | 2.29× (2.25–2.34) | 1.94× (1.91–1.98) | 2.62× (2.57–2.67) | 2.48× (2.42–2.53) | 1.98× (1.94–2.02) | 2.09× (2.06–2.13) | 0.83× (0.82–0.85) | 0.92× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.53) | 3.00× (2.98–3.02) | 2.62× (2.58–2.66) | 2.79× (2.76–2.81) | 3.09× (3.06–3.12) | 3.67× (3.63–3.70) | 4.10× (4.03–4.16) | 1.51× (1.49–1.53) | 1.75× (1.73–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.40) | 2.76× (2.71–2.81) | 2.36× (2.32–2.40) | 3.33× (3.26–3.41) | 2.77× (2.73–2.82) | 3.21× (3.15–3.26) | 3.58× (3.51–3.66) | 1.32× (1.31–1.34) | 1.53× (1.51–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.53–1.63) | 2.49× (2.43–2.55) | 1.98× (1.93–2.04) | 2.87× (2.77–2.97) | 2.59× (2.52–2.66) | 2.28× (2.22–2.34) | 2.55× (2.48–2.62) | 0.94× (0.92–0.96) | 1.09× (1.06–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.48× (2.45–2.52) | 3.45× (3.39–3.51) | 2.86× (2.83–2.90) | 6.08× (5.96–6.20) | 2.14× (2.11–2.16) | 2.18× (2.15–2.20) | 0.97× (0.95–0.99) | 1.38× (1.36–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.56–0.67) | 2.26× (2.23–2.29) | 4.02× (3.96–4.09) | 2.74× (2.71–2.78) | 5.68× (5.60–5.76) | 1.99× (1.97–2.02) | 2.03× (2.00–2.06) | 0.90× (0.88–0.93) | 1.29× (1.26–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.51× (2.47–2.54) | 4.07× (4.00–4.14) | 2.80× (2.76–2.84) | 5.73× (5.62–5.84) | 2.02× (2.00–2.05) | 2.06× (2.03–2.10) | 0.92× (0.90–0.93) | 1.31× (1.29–1.33) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 1.80× (1.79–1.82) | 3.83× (3.77–3.88) | 4.21× (4.18–4.24) | 4.49× (4.46–4.53) | 4.69× (4.66–4.72) | 4.64× (4.59–4.69) | 4.75× (4.72–4.78) | 1.50× (1.49–1.51) | 1.64× (1.63–1.65) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 1.53× (1.51–1.54) | 3.18× (3.14–3.21) | 3.52× (3.48–3.55) | 4.15× (4.09–4.21) | 3.86× (3.79–3.93) | 3.72× (3.67–3.76) | 3.81× (3.77–3.84) | 1.20× (1.19–1.21) | 1.31× (1.30–1.33) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 1.65× (1.61–1.69) | 2.57× (2.51–2.63) | 2.87× (2.78–2.97) | 3.31× (3.25–3.37) | 3.16× (3.10–3.21) | 2.44× (2.39–2.48) | 2.50× (2.46–2.53) | 0.79× (0.78–0.80) | 0.86× (0.85–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.78–0.79) | 11.00× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.31–0.34) | 0.44× (0.37–0.54) | 0.67× (0.64–0.70) | 0.57× (0.55–0.59) | 5.36× (5.13–5.60) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.32–0.35) | 0.51× (0.45–0.58) | 0.59× (0.56–0.62) | 0.52× (0.50–0.55) | 3.47× (3.32–3.62) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.70) | 0.69× (0.63–0.75) | 1.62× (1.58–1.67) | 0.82× (0.80–0.84) | 5.87× (5.73–6.03) | 0.47× (0.46–0.49) | 0.50× (0.49–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.68× (0.65–0.71) | 1.14× (1.10–1.19) | 0.70× (0.66–0.74) | 3.35× (3.21–3.51) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.72–0.73) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.52) | 0.40× (0.37–0.43) | 0.90× (0.85–0.94) | 0.73× (0.69–0.76) | 3.10× (2.95–3.26) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.29–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.64) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.80× (0.78–0.82) | 2.48× (2.43–2.53) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.17–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.40) | 0.84× (0.82–0.87) | 0.72× (0.69–0.75) | 2.90× (2.80–2.99) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.51) | 0.80× (0.79–0.82) | 0.82× (0.79–0.85) | 2.32× (2.28–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.36) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.77× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.42) | 0.67× (0.66–0.67) | 6.44× (6.43–6.45) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 36 MB | 0.46× (0.46–0.47) | 0.40× (0.37–0.44) | 1.08× (1.07–1.10) | 0.73× (0.71–0.74) | 3.42× (3.37–3.47) | 0.21× (0.21–0.21) | 0.22× (0.22–0.23) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 55 MB | 0.51× (0.50–0.52) | 0.45× (0.42–0.48) | 0.92× (0.91–0.93) | 0.75× (0.73–0.78) | 2.37× (2.32–2.41) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 501 | 2.00 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 868 | 2.19 | 1.09× (1.08–1.10) | 85% | 1.02× (1.01–1.04) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,041 | 3.38 | 1.68× (1.66–1.70) | 52% | 1.22× (1.20–1.25) | 1.66× (1.62–1.69) |

**Best:** mibz2 at 4 threads (1,041 MB/s); lbzip2 at 4 threads (235 MB/s); pbz2 at 4 threads (357 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.21, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.44, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.54 | 0.01 | 0.07 | 0.202 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3699 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.7% |
| MTF + RLE2 | 1.53 | 17.3% |
| T-vector build | 1.50 | 16.9% |
| walk (the one each build uses) | 4.27 | 48.1% |
| of which: segment assembly | 0.19 | 2.2% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.87 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.061 | (in its total) | 2.71× (2.70–2.71) |  |
| walk | 1.046 | 2.079 | (in its total) | 1.98× (1.98–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.041 | 0.042 | (in its total) | 1.00× (0.96–1.03) |  |
| inverse BWT, total | 1.480 | 3.182 | 1.898 | 2.15× (2.14–2.15) | 1.29× (1.27–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m59s |
| 4 threads | 29m22s ± 4m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m28s ± 2m45s |
| the scanner alone (its serial part) | 11m12s ± 30.2s |
| the workers alone, over 4 | 19m28s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 2.55 ns a byte of output at 4 threads (CPU, summed over them), 1.63 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,527,905 | 12,873,504 | \`d928dbd828b8a8a4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3699.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
