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

The latest record: [`517eb70`](<https://github.com/lmmx/mibz2/commit/517eb7019e149b36f8c7f86ebecd941d666a886e>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45598543714>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.493 s | 425 | 7.7 | 3.05× (2.96–3.14) | 7.05× (6.86–7.24) | 7.28× (7.05–7.51) | 7.87× (7.66–8.09) | 7.12× (6.93–7.32) | 7.73× (7.53–7.93) | 7.08× (6.89–7.27) | 2.26× (2.20–2.32) | 2.62× (2.54–2.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.276 s | 761 | 8.2 | 3.05× (2.89–3.22) | 6.64× (6.18–7.12) | 7.41× (7.03–7.80) | 7.34× (6.90–7.82) | 6.59× (6.25–6.96) | 13.45× (12.75–14.20) | 12.33× (11.70–12.99) | 3.94× (3.72–4.16) | 4.56× (4.30–4.84) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.226 s | 929 | 12.8 | 3.20× (3.16–3.24) | 4.89× (4.76–5.03) | 6.11× (5.97–6.24) | 5.46× (5.37–5.55) | 5.66× (5.47–5.85) | 17.03× (16.82–17.25) | 15.61× (15.41–15.81) | 4.99× (4.94–5.03) | 5.78× (5.68–5.88) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 957 | 3.7 | 0.59× (0.58–0.59) | 3.55× (3.42–3.69) | 4.40× (4.25–4.56) | 3.50× (3.46–3.55) | 6.08× (6.00–6.17) | 2.91× (2.87–2.95) | 2.60× (2.57–2.63) | 1.00× (0.98–1.01) | 1.47× (1.45–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 913 | 4.0 | 0.54× (0.53–0.55) | 2.82× (2.78–2.85) | 5.83× (5.66–6.01) | 3.38× (3.33–3.42) | 5.79× (5.70–5.87) | 2.78× (2.75–2.82) | 2.49× (2.46–2.53) | 0.95× (0.93–0.97) | 1.41× (1.39–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 877 | 4.1 | 0.53× (0.51–0.55) | 2.73× (2.67–2.78) | 5.67× (5.54–5.79) | 3.24× (3.17–3.31) | 5.58× (5.48–5.68) | 2.68× (2.63–2.74) | 2.40× (2.35–2.45) | 0.92× (0.89–0.95) | 1.36× (1.33–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 577 | 6.0 | 2.62× (2.58–2.65) | 3.01× (2.98–3.04) | 3.85× (3.81–3.88) | 3.82× (3.78–3.85) | 4.17× (4.14–4.21) | 2.54× (2.52–2.56) | 2.11× (2.09–2.12) | 2.02× (2.00–2.04) | 3.00× (2.97–3.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 916 | 7.1 | 2.34× (2.20–2.48) | 2.93× (2.80–3.07) | 4.16× (4.01–4.32) | 3.48× (3.36–3.60) | 5.02× (4.84–5.20) | 3.97× (3.85–4.10) | 3.29× (3.19–3.40) | 3.16× (3.08–3.25) | 4.68× (4.53–4.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 836 | 13.7 | 2.00× (1.93–2.06) | 2.15× (2.00–2.30) | 3.25× (3.07–3.44) | 2.26× (2.18–2.34) | 4.28× (4.16–4.40) | 3.68× (3.57–3.79) | 3.05× (2.95–3.14) | 2.93× (2.83–3.03) | 4.34× (4.20–4.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.126 s | 64 | 51.9 | 1.39× (1.37–1.41) | 2.81× (2.77–2.85) | 2.52× (2.48–2.56) | 2.59× (2.55–2.62) | 2.97× (2.93–3.01) | 3.24× (3.20–3.29) | 3.22× (3.17–3.26) | 1.26× (1.24–1.28) | 1.44× (1.42–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 102 | 57.9 | 1.29× (1.21–1.37) | 2.59× (2.42–2.77) | 2.44× (2.28–2.62) | 3.11× (2.92–3.32) | 3.00× (2.82–3.19) | 5.18× (4.86–5.53) | 5.14× (4.81–5.48) | 2.01× (1.88–2.14) | 2.31× (2.16–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 128 | 78.3 | 1.45× (1.42–1.49) | 2.34× (2.29–2.39) | 2.34× (2.21–2.48) | 2.87× (2.83–2.92) | 3.15× (3.09–3.21) | 6.60× (6.44–6.76) | 6.54× (6.39–6.69) | 2.56× (2.50–2.61) | 2.94× (2.88–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.139 s | 69 | 47.7 | 1.51× (1.48–1.54) | 3.18× (3.15–3.21) | 2.68× (2.66–2.70) | 2.80× (2.78–2.83) | 3.22× (3.19–3.25) | 3.79× (3.78–3.81) | 3.98× (3.97–3.99) | 1.48× (1.44–1.51) | 1.77× (1.74–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 54.1 | 1.40× (1.38–1.43) | 2.96× (2.84–3.08) | 2.62× (2.55–2.69) | 3.61× (3.54–3.69) | 3.25× (3.19–3.31) | 6.34× (6.23–6.45) | 6.65× (6.55–6.76) | 2.46× (2.41–2.52) | 2.96× (2.89–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 78.5 | 1.52× (1.49–1.55) | 2.49× (2.41–2.56) | 2.45× (2.38–2.52) | 3.21× (3.13–3.30) | 3.09× (3.04–3.14) | 7.54× (7.44–7.64) | 7.91× (7.79–8.03) | 2.93× (2.86–3.01) | 3.52× (3.41–3.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 733 | 5.1 | 0.66× (0.66–0.67) | 3.01× (2.97–3.05) | 4.39× (4.33–4.46) | 3.11× (3.08–3.15) | 6.83× (6.74–6.92) | 2.52× (2.50–2.55) | 2.31× (2.28–2.33) | 1.00× (0.99–1.01) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 5.8 | 0.53× (0.53–0.54) | 2.74× (2.71–2.77) | 4.77× (4.62–4.93) | 2.88× (2.82–2.93) | 6.34× (6.26–6.42) | 2.29× (2.27–2.32) | 2.10× (2.08–2.12) | 0.91× (0.89–0.92) | 1.38× (1.36–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 5.8 | 0.53× (0.52–0.54) | 2.69× (2.64–2.75) | 4.73× (4.58–4.89) | 2.81× (2.77–2.85) | 6.17× (6.05–6.28) | 2.25× (2.20–2.29) | 2.05× (2.02–2.09) | 0.89× (0.87–0.90) | 1.35× (1.32–1.37) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 0.063 s | 192 | 17.4 | 2.03× (2.01–2.05) | 4.23× (4.18–4.28) | 4.66× (4.59–4.74) | 4.69× (4.62–4.76) | 4.94× (4.87–5.01) | 5.43× (5.35–5.51) | 4.94× (4.89–5.00) | 1.59× (1.56–1.62) | 1.95× (1.93–1.98) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 0.044 s | 270 | 22.7 | 1.54× (1.40–1.69) | 3.38× (3.24–3.53) | 4.14× (3.78–4.53) | 4.28× (3.95–4.63) | 4.14× (3.80–4.50) | 7.38× (6.83–7.98) | 6.72× (6.20–7.28) | 2.16× (1.98–2.35) | 2.66× (2.45–2.89) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 0.040 s | 304 | 34.9 | 1.58× (1.54–1.63) | 2.54× (2.39–2.69) | 3.57× (3.45–3.69) | 3.63× (3.43–3.85) | 3.86× (3.76–3.96) | 8.66× (8.45–8.87) | 7.88× (7.69–8.07) | 2.53× (2.44–2.62) | 3.12× (3.03–3.21) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.04× (2.95–3.13) | 7.01× (6.82–7.21) | 7.24× (7.01–7.47) | 7.85× (7.64–8.06) | 7.10× (6.91–7.30) | 7.70× (7.50–7.90) | 7.05× (6.87–7.24) | 2.26× (2.20–2.32) | 2.60× (2.52–2.69) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.00× (2.97–3.03) | 6.84× (6.73–6.96) | 7.10× (7.02–7.18) | 7.82× (7.77–7.88) | 6.71× (6.67–6.75) | 7.26× (7.24–7.28) | 6.65× (6.59–6.71) | 2.13× (2.11–2.14) | 2.45× (2.42–2.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.05× (2.99–3.11) | 5.19× (5.10–5.28) | 5.63× (5.48–5.78) | 5.83× (5.76–5.90) | 5.28× (5.13–5.43) | 4.68× (4.62–4.74) | 4.28× (4.21–4.36) | 1.37× (1.35–1.39) | 1.58× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.63) | 3.19× (3.14–3.23) | 3.99× (3.94–4.03) | 3.28× (3.24–3.32) | 5.63× (5.55–5.71) | 2.74× (2.70–2.77) | 2.46× (2.43–2.49) | 1.00× (0.98–1.01) | 1.03× (1.01–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.58) | 2.85× (2.81–2.89) | 5.01× (4.95–5.08) | 3.08× (3.04–3.13) | 5.19× (5.11–5.27) | 2.53× (2.50–2.57) | 2.28× (2.24–2.31) | 0.92× (0.90–0.94) | 0.95× (0.92–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 3.06× (2.99–3.12) | 4.97× (4.90–5.05) | 3.02× (2.96–3.08) | 5.08× (5.00–5.17) | 2.49× (2.44–2.54) | 2.24× (2.19–2.28) | 0.91× (0.88–0.93) | 0.93× (0.91–0.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.50–2.55) | 2.78× (2.75–2.80) | 3.48× (3.45–3.50) | 3.65× (3.62–3.69) | 3.99× (3.96–4.02) | 2.45× (2.43–2.47) | 2.04× (2.03–2.06) | 1.96× (1.95–1.98) | 2.69× (2.66–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.24–2.33) | 2.57× (2.53–2.62) | 3.12× (3.05–3.18) | 3.21× (3.14–3.27) | 3.47× (3.39–3.55) | 2.04× (1.99–2.08) | 1.70× (1.66–1.73) | 1.63× (1.60–1.66) | 2.23× (2.18–2.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.11× (2.02–2.21) | 1.83× (1.74–1.93) | 2.34× (2.23–2.45) | 2.17× (2.08–2.27) | 2.25× (2.15–2.36) | 1.08× (1.03–1.14) | 0.90× (0.86–0.95) | 0.87× (0.83–0.91) | 1.19× (1.13–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.40) | 2.74× (2.71–2.78) | 2.45× (2.42–2.49) | 2.56× (2.53–2.60) | 2.94× (2.90–2.98) | 3.21× (3.16–3.25) | 3.18× (3.14–3.22) | 1.25× (1.23–1.27) | 1.41× (1.39–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.30–1.33) | 2.58× (2.54–2.62) | 2.28× (2.25–2.31) | 3.05× (3.01–3.10) | 2.70× (2.66–2.73) | 2.89× (2.84–2.93) | 2.86× (2.82–2.91) | 1.13× (1.11–1.14) | 1.27× (1.25–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.53–1.60) | 2.42× (2.36–2.47) | 1.97× (1.93–2.00) | 2.71× (2.67–2.75) | 2.60× (2.56–2.63) | 2.13× (2.08–2.17) | 2.11× (2.07–2.15) | 0.83× (0.81–0.85) | 0.94× (0.92–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.47–1.53) | 3.12× (3.10–3.14) | 2.61× (2.59–2.63) | 2.78× (2.75–2.80) | 3.19× (3.16–3.21) | 3.75× (3.74–3.77) | 3.94× (3.93–3.95) | 1.47× (1.43–1.50) | 1.73× (1.70–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.40) | 2.87× (2.82–2.91) | 2.39× (2.36–2.42) | 3.33× (3.29–3.38) | 2.84× (2.80–2.89) | 3.30× (3.26–3.34) | 3.46× (3.42–3.50) | 1.29× (1.26–1.32) | 1.52× (1.50–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.58–1.65) | 2.57× (2.51–2.63) | 2.00× (1.95–2.05) | 2.84× (2.79–2.89) | 2.59× (2.56–2.63) | 2.29× (2.26–2.33) | 2.40× (2.37–2.44) | 0.90× (0.86–0.93) | 1.06× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.71× (2.66–2.76) | 3.62× (3.51–3.73) | 2.85× (2.82–2.88) | 6.10× (6.02–6.18) | 2.34× (2.31–2.36) | 2.15× (2.13–2.17) | 1.00× (0.99–1.01) | 0.99× (0.96–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.58–0.59) | 2.35× (2.32–2.37) | 4.00× (3.93–4.07) | 2.58× (2.55–2.62) | 5.45× (5.38–5.52) | 2.04× (2.02–2.06) | 1.87× (1.85–1.89) | 0.87× (0.86–0.89) | 0.86× (0.84–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.54× (2.50–2.59) | 3.97× (3.91–4.02) | 2.60× (2.56–2.63) | 5.42× (5.32–5.52) | 2.04× (2.00–2.08) | 1.87× (1.84–1.91) | 0.87× (0.86–0.89) | 0.86× (0.83–0.89) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 2.00× (1.97–2.02) | 4.10× (4.05–4.15) | 4.44× (4.39–4.49) | 4.57× (4.51–4.64) | 4.82× (4.76–4.88) | 5.29× (5.22–5.36) | 4.82× (4.77–4.87) | 1.57× (1.54–1.60) | 1.85× (1.82–1.88) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 1.67× (1.63–1.70) | 3.38× (3.30–3.46) | 3.72× (3.62–3.82) | 4.09× (4.01–4.18) | 3.82× (3.76–3.89) | 4.12× (4.03–4.20) | 3.75× (3.68–3.82) | 1.22× (1.19–1.25) | 1.44× (1.41–1.47) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 1.74× (1.66–1.83) | 2.71× (2.63–2.80) | 2.95× (2.84–3.07) | 3.18× (3.07–3.31) | 3.07× (3.01–3.13) | 2.65× (2.59–2.71) | 2.41× (2.36–2.46) | 0.79× (0.76–0.81) | 0.93× (0.90–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.97–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.52× (0.46–0.59) | 0.72× (0.69–0.74) | 0.60× (0.58–0.62) | 5.73× (5.56–5.91) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.39) | 0.77× (0.67–0.88) | 0.68× (0.66–0.69) | 0.59× (0.57–0.61) | 3.91× (3.81–4.02) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.18–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.95× (7.91–7.99) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.74× (0.70–0.79) | 1.69× (1.64–1.73) | 0.84× (0.82–0.86) | 6.11× (5.97–6.25) | 0.50× (0.49–0.51) | 0.52× (0.51–0.53) | 0.57× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.67–0.73) | 0.79× (0.73–0.87) | 1.24× (1.19–1.30) | 0.77× (0.73–0.80) | 3.66× (3.50–3.83) | 0.29× (0.28–0.30) | 0.30× (0.29–0.32) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.13× (5.13–5.14) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.39× (0.36–0.43) | 0.91× (0.88–0.94) | 0.71× (0.67–0.75) | 3.12× (3.03–3.22) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.65–0.67) | 0.47× (0.46–0.47) | 0.88× (0.87–0.90) | 0.83× (0.81–0.85) | 2.59× (2.54–2.64) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.81–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.40) | 0.83× (0.80–0.86) | 0.69× (0.65–0.72) | 2.85× (2.76–2.94) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.49× (0.45–0.52) | 0.80× (0.78–0.81) | 0.81× (0.79–0.83) | 2.30× (2.25–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.29–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.18× (6.17–6.19) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.18–6.24) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.65–0.66) | 6.38× (6.35–6.41) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 35 MB | 0.48× (0.46–0.50) | 0.43× (0.39–0.47) | 1.10× (1.05–1.15) | 0.69× (0.65–0.74) | 3.49× (3.36–3.63) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.33–0.36) | 0.17× (0.16–0.18) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 57 MB | 0.50× (0.48–0.52) | 0.49× (0.47–0.52) | 0.91× (0.88–0.94) | 0.70× (0.67–0.74) | 2.31× (2.23–2.39) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 425 | 2.36 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 761 | 2.52 | 1.06× (1.03–1.09) | 87% | 1.03× (1.02–1.05) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 929 | 3.95 | 1.65× (1.60–1.69) | 55% | 1.22× (1.20–1.24) | 1.65× (1.62–1.69) |

**Best:** mibz2 at 4 threads (929 MB/s); lbzip2 at 4 threads (192 MB/s); pbz2 at 4 threads (291 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.08 | 0.226 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.5% |
| MTF + RLE2 | 1.52 | 17.2% |
| T-vector build | 1.53 | 17.4% |
| walk (the one each build uses) | 4.30 | 48.9% |
| of which: segment assembly | 0.16 | 1.9% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.80 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.304 | (in its total) | 2.67× (2.66–2.67) |  |
| walk | 1.292 | 2.641 | (in its total) | 2.04× (2.03–2.06) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.057 | (in its total) | 1.20× (1.18–1.22) |  |
| inverse BWT, total | 1.829 | 4.005 | 3.238 | 2.19× (2.18–2.20) | 1.78× (1.76–1.79) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h12m ± 8m54s |
| 4 threads | 32m51s ± 4m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m32s ± 3m20s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 23m32s ± 3m20s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 3.08 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,369,642 | 12,022,894 | \`502528b59d9a8643…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
