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

The latest record: [`2044ac2`](<https://github.com/lmmx/mibz2/commit/2044ac2b817cfd4f7f022d72c51a5463e359f332>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43785297654>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.524 s | 400 | 8.1 | 2.89× (2.87–2.90) | 6.58× (6.55–6.60) | 6.79× (6.75–6.84) | 7.36× (7.33–7.39) | 6.66× (6.63–6.69) | 7.29× (7.16–7.42) | 6.74× (6.71–6.78) | 2.13× (2.11–2.14) | 2.47× (2.46–2.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.306 s | 686 | 8.8 | 2.82× (2.72–2.93) | 6.08× (5.91–6.26) | 6.90× (6.64–7.16) | 6.76× (6.57–6.96) | 6.13× (5.92–6.33) | 12.53× (12.00–13.07) | 11.59× (11.24–11.96) | 3.65× (3.55–3.77) | 4.25× (4.12–4.38) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 859 | 13.7 | 2.92× (2.89–2.95) | 4.42× (4.37–4.48) | 5.48× (5.36–5.60) | 4.97× (4.91–5.03) | 5.03× (4.97–5.09) | 15.65× (15.27–16.04) | 14.48× (14.36–14.61) | 4.57× (4.51–4.62) | 5.31× (5.26–5.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 990 | 3.6 | 0.60× (0.59–0.61) | 3.62× (3.48–3.76) | 4.45× (4.38–4.52) | 3.59× (3.53–3.64) | 6.16× (6.06–6.25) | 2.99× (2.95–3.04) | 2.68× (2.64–2.72) | 1.00× (0.98–1.01) | 1.52× (1.49–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 983 | 3.7 | 0.58× (0.57–0.59) | 3.02× (2.99–3.05) | 6.05× (5.92–6.19) | 3.61× (3.58–3.64) | 6.14× (6.06–6.22) | 2.98× (2.95–3.01) | 2.67× (2.64–2.69) | 0.99× (0.97–1.01) | 1.51× (1.49–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 959 | 3.7 | 0.57× (0.56–0.58) | 2.96× (2.93–3.00) | 5.96× (5.79–6.14) | 3.52× (3.48–3.56) | 6.00× (5.92–6.08) | 2.91× (2.88–2.95) | 2.61× (2.58–2.64) | 0.97× (0.96–0.98) | 1.48× (1.46–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 585 | 5.9 | 2.75× (2.69–2.82) | 3.05× (3.03–3.07) | 3.94× (3.86–4.02) | 3.83× (3.81–3.85) | 4.15× (4.12–4.18) | 2.58× (2.56–2.59) | 2.13× (2.12–2.14) | 2.03× (2.02–2.04) | 3.03× (3.02–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 885 | 7.2 | 2.32× (2.23–2.41) | 2.96× (2.80–3.13) | 3.94× (3.79–4.10) | 3.57× (3.48–3.65) | 4.86× (4.73–4.99) | 3.89× (3.81–3.97) | 3.22× (3.14–3.29) | 3.06× (2.99–3.14) | 4.58× (4.48–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 782 | 14.2 | 1.81× (1.74–1.89) | 1.93× (1.79–2.07) | 2.98× (2.82–3.16) | 2.22× (2.12–2.34) | 3.87× (3.74–4.01) | 3.39× (3.26–3.52) | 2.80× (2.70–2.91) | 2.67× (2.57–2.77) | 3.99× (3.84–4.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 55.1 | 1.33× (1.33–1.34) | 2.67× (2.65–2.68) | 2.42× (2.37–2.46) | 2.40× (2.38–2.42) | 2.79× (2.78–2.81) | 3.07× (3.04–3.11) | 3.11× (3.09–3.12) | 1.20× (1.19–1.21) | 1.36× (1.36–1.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 98 | 61.5 | 1.24× (1.20–1.28) | 2.48× (2.36–2.59) | 2.32× (2.25–2.39) | 2.85× (2.76–2.94) | 2.85× (2.77–2.93) | 4.95× (4.78–5.11) | 5.00× (4.85–5.15) | 1.94× (1.88–2.00) | 2.20× (2.13–2.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 85.8 | 1.30× (1.26–1.34) | 2.08× (2.04–2.11) | 2.03× (1.94–2.12) | 2.40× (2.32–2.47) | 2.76× (2.71–2.82) | 5.84× (5.74–5.95) | 5.90× (5.76–6.05) | 2.29× (2.24–2.33) | 2.59× (2.53–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 65 | 50.2 | 1.48× (1.44–1.51) | 3.02× (2.98–3.06) | 2.54× (2.52–2.56) | 2.59× (2.58–2.60) | 3.08× (3.01–3.15) | 3.57× (3.56–3.59) | 3.86× (3.84–3.87) | 1.38× (1.37–1.39) | 1.67× (1.65–1.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 57.3 | 1.36× (1.27–1.47) | 2.69× (2.62–2.77) | 2.41× (2.36–2.47) | 3.21× (3.12–3.29) | 3.01× (2.95–3.07) | 5.83× (5.71–5.96) | 6.29× (6.16–6.43) | 2.25× (2.20–2.30) | 2.72× (2.65–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 82.9 | 1.42× (1.37–1.47) | 2.25× (2.20–2.30) | 2.20× (2.15–2.26) | 2.80× (2.75–2.85) | 2.72× (2.66–2.77) | 6.66× (6.55–6.77) | 7.18× (7.06–7.30) | 2.57× (2.52–2.62) | 3.10× (3.06–3.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.0 | 0.67× (0.66–0.68) | 3.05× (3.01–3.09) | 4.47× (4.41–4.53) | 3.14× (3.10–3.18) | 6.80× (6.71–6.90) | 2.55× (2.52–2.58) | 2.35× (2.32–2.38) | 0.98× (0.97–1.00) | 1.54× (1.52–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 707 | 5.4 | 0.57× (0.56–0.58) | 2.89× (2.85–2.94) | 4.98× (4.90–5.06) | 3.02× (2.98–3.06) | 6.47× (6.40–6.55) | 2.42× (2.38–2.45) | 2.23× (2.19–2.27) | 0.93× (0.91–0.95) | 1.46× (1.43–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 5.3 | 0.56× (0.55–0.57) | 2.87× (2.83–2.90) | 4.91× (4.85–4.97) | 2.97× (2.93–3.01) | 6.35× (6.25–6.44) | 2.38× (2.36–2.41) | 2.20× (2.17–2.22) | 0.92× (0.91–0.93) | 1.44× (1.42–1.45) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.071 s | 187 | 17.8 | 1.97× (1.96–1.99) | 4.15× (4.11–4.18) | 4.55× (4.51–4.59) | 4.52× (4.49–4.55) | 4.72× (4.68–4.76) | 5.28× (5.24–5.32) | 4.93× (4.89–4.97) | 1.50× (1.49–1.51) | 1.86× (1.83–1.90) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.050 s | 267 | 22.6 | 1.61× (1.57–1.64) | 3.37× (3.22–3.52) | 3.93× (3.83–4.02) | 4.14× (3.97–4.33) | 4.26× (4.15–4.37) | 7.56× (7.38–7.74) | 7.06× (6.90–7.23) | 2.15× (2.10–2.20) | 2.67× (2.60–2.74) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.044 s | 300 | 34.8 | 1.55× (1.50–1.61) | 2.55× (2.42–2.69) | 3.23× (3.14–3.33) | 3.24× (3.17–3.32) | 3.79× (3.69–3.89) | 8.42× (8.21–8.63) | 7.86× (7.67–8.06) | 2.40× (2.34–2.45) | 2.97× (2.89–3.06) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.87–2.89) | 6.54× (6.52–6.57) | 6.76× (6.72–6.80) | 7.33× (7.31–7.36) | 6.64× (6.61–6.66) | 7.26× (7.13–7.39) | 6.72× (6.68–6.76) | 2.12× (2.11–2.13) | 2.46× (2.44–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.74–2.76) | 6.24× (6.18–6.31) | 6.49× (6.46–6.52) | 7.08× (7.04–7.11) | 6.15× (6.12–6.19) | 6.67× (6.55–6.80) | 6.18× (6.16–6.19) | 1.95× (1.94–1.96) | 2.26× (2.25–2.27) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.87× (2.82–2.92) | 4.73× (4.68–4.79) | 5.10× (5.03–5.16) | 5.35× (5.29–5.41) | 4.78× (4.73–4.83) | 4.34× (4.25–4.42) | 4.01× (3.95–4.07) | 1.27× (1.25–1.28) | 1.47× (1.44–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.28× (3.24–3.32) | 4.08× (4.02–4.14) | 3.35× (3.30–3.40) | 5.69× (5.61–5.78) | 2.81× (2.77–2.86) | 2.53× (2.49–2.57) | 1.00× (0.99–1.01) | 1.05× (1.03–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.62–0.63) | 3.10× (3.07–3.14) | 5.43× (5.39–5.48) | 3.35× (3.32–3.38) | 5.60× (5.53–5.67) | 2.75× (2.73–2.78) | 2.48× (2.45–2.50) | 0.98× (0.96–1.00) | 1.03× (1.01–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.38× (3.33–3.43) | 5.39× (5.33–5.46) | 3.34× (3.30–3.37) | 5.56× (5.49–5.62) | 2.75× (2.72–2.78) | 2.47× (2.45–2.50) | 0.98× (0.97–0.99) | 1.02× (1.00–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.60–2.72) | 2.82× (2.80–2.83) | 3.59× (3.48–3.71) | 3.67× (3.66–3.69) | 3.97× (3.94–4.00) | 2.49× (2.47–2.51) | 2.07× (2.06–2.07) | 1.97× (1.96–1.98) | 2.73× (2.70–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.22–2.29) | 2.48× (2.43–2.53) | 3.05× (3.00–3.10) | 3.27× (3.22–3.31) | 3.36× (3.31–3.42) | 2.00× (1.97–2.04) | 1.66× (1.64–1.69) | 1.59× (1.56–1.61) | 2.20× (2.16–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.98–2.08) | 1.73× (1.68–1.77) | 2.19× (2.16–2.23) | 2.11× (2.07–2.14) | 2.12× (2.08–2.15) | 1.03× (1.01–1.05) | 0.85× (0.84–0.87) | 0.81× (0.80–0.83) | 1.13× (1.10–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.34) | 2.58× (2.57–2.59) | 2.36× (2.32–2.40) | 2.38× (2.36–2.39) | 2.77× (2.75–2.78) | 3.04× (3.01–3.08) | 3.07× (3.06–3.08) | 1.20× (1.19–1.21) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.23–1.26) | 2.42× (2.38–2.45) | 2.13× (2.10–2.16) | 2.72× (2.68–2.75) | 2.51× (2.48–2.54) | 2.70× (2.66–2.75) | 2.73× (2.69–2.77) | 1.07× (1.05–1.08) | 1.19× (1.18–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.39–1.48) | 2.18× (2.11–2.24) | 1.79× (1.75–1.83) | 2.37× (2.32–2.43) | 2.33× (2.28–2.39) | 1.94× (1.91–1.98) | 1.96× (1.92–2.01) | 0.77× (0.75–0.78) | 0.86× (0.84–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.47× (1.44–1.51) | 2.96× (2.93–3.00) | 2.48× (2.47–2.49) | 2.57× (2.55–2.58) | 3.05× (2.98–3.12) | 3.54× (3.52–3.56) | 3.82× (3.80–3.83) | 1.37× (1.36–1.39) | 1.63× (1.62–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.35) | 2.72× (2.69–2.75) | 2.25× (2.24–2.27) | 3.03× (2.99–3.06) | 2.69× (2.67–2.71) | 3.10× (3.09–3.12) | 3.35× (3.33–3.37) | 1.21× (1.20–1.22) | 1.43× (1.42–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.63) | 2.46× (2.42–2.51) | 1.87× (1.84–1.90) | 2.59× (2.55–2.63) | 2.43× (2.38–2.47) | 2.16× (2.12–2.19) | 2.32× (2.29–2.36) | 0.84× (0.82–0.85) | 0.99× (0.98–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.71–0.72) | 2.73× (2.69–2.76) | 3.61× (3.53–3.69) | 2.86× (2.84–2.89) | 6.06× (5.99–6.13) | 2.36× (2.34–2.37) | 2.18× (2.15–2.20) | 0.98× (0.97–1.00) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.51× (2.47–2.54) | 4.28× (4.22–4.34) | 2.76× (2.73–2.79) | 5.67× (5.61–5.74) | 2.18× (2.16–2.21) | 2.02× (1.99–2.05) | 0.91× (0.90–0.93) | 0.93× (0.89–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.71–2.79) | 4.29× (4.22–4.35) | 2.81× (2.77–2.84) | 5.72× (5.63–5.80) | 2.22× (2.19–2.24) | 2.05× (2.02–2.07) | 0.93× (0.91–0.94) | 0.94× (0.91–0.97) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.95× (1.93–1.96) | 3.97× (3.94–4.01) | 4.37× (4.32–4.42) | 4.42× (4.39–4.45) | 4.62× (4.58–4.66) | 5.16× (5.12–5.20) | 4.82× (4.79–4.86) | 1.49× (1.48–1.50) | 1.79× (1.77–1.81) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.65× (1.63–1.67) | 3.29× (3.24–3.34) | 3.62× (3.56–3.67) | 3.95× (3.88–4.01) | 3.72× (3.67–3.77) | 4.08× (4.02–4.14) | 3.81× (3.76–3.87) | 1.18× (1.16–1.19) | 1.42× (1.39–1.44) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.73× (1.69–1.77) | 2.61× (2.55–2.67) | 2.89× (2.82–2.96) | 3.09× (3.03–3.15) | 2.98× (2.92–3.05) | 2.66× (2.59–2.72) | 2.48× (2.43–2.53) | 0.77× (0.75–0.78) | 0.92× (0.90–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.28–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.78) | 11.04× (11.01–11.07) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.51× (0.47–0.56) | 0.70× (0.68–0.72) | 0.59× (0.57–0.60) | 5.58× (5.43–5.74) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.33–0.35) | 0.60× (0.53–0.67) | 0.60× (0.58–0.61) | 0.52× (0.50–0.54) | 3.49× (3.40–3.59) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.04) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.69× (0.66–0.73) | 1.60× (1.56–1.63) | 0.80× (0.78–0.82) | 5.79× (5.66–5.92) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.54× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.61–0.66) | 0.69× (0.62–0.77) | 1.14× (1.09–1.18) | 0.70× (0.67–0.73) | 3.36× (3.25–3.48) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.38× (0.35–0.42) | 0.92× (0.89–0.94) | 0.71× (0.69–0.74) | 3.17× (3.09–3.25) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.61–0.65) | 0.45× (0.43–0.46) | 0.85× (0.83–0.87) | 0.78× (0.75–0.81) | 2.46× (2.41–2.51) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.22–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.39× (0.37–0.41) | 0.86× (0.83–0.89) | 0.71× (0.68–0.75) | 2.96× (2.85–3.06) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.50× (0.48–0.53) | 0.80× (0.79–0.82) | 0.81× (0.78–0.84) | 2.33× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.33× (5.32–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.67× (0.67–0.68) | 6.43× (6.40–6.45) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.42× (0.38–0.45) | 1.07× (1.04–1.10) | 0.69× (0.67–0.70) | 3.36× (3.30–3.42) | 0.21× (0.20–0.21) | 0.22× (0.22–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.49× (0.47–0.50) | 0.49× (0.45–0.53) | 0.89× (0.87–0.91) | 0.73× (0.70–0.76) | 2.28× (2.20–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 686 | 2.73 | 1.09× (1.08–1.09) | 86% | 1.04× (1.03–1.05) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 859 | 4.22 | 1.67× (1.66–1.69) | 54% | 1.21× (1.20–1.22) | 1.67× (1.66–1.68) |

**Best:** mibz2 at 4 threads (859 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.22, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.69 | 0.02 | 0.09 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 12.6% |
| MTF + RLE2 | 1.59 | 17.7% |
| T-vector build | 1.52 | 17.0% |
| walk (the one each build uses) | 4.17 | 46.6% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.94 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.487 | 1.335 | (in its total) | 2.74× (2.73–2.75) |  |
| walk | 1.229 | 2.641 | (in its total) | 2.15× (2.14–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.24× (1.23–1.25) |  |
| inverse BWT, total | 1.761 | 4.029 | 3.264 | 2.29× (2.28–2.30) | 1.85× (1.85–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m44s |
| 4 threads | 35m29s ± 4m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m48s ± 3m31s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 24m48s ± 3m31s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.25 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,079 | 13,357,122 | \`cc146a11acfb9941…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
