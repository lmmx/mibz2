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

The latest record: [`228696d`](<https://github.com/lmmx/mibz2/commit/228696dca797ef201138c3d6daf7e8159b834408>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43738032930>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.530 s | 396 | 8.2 | 2.89× (2.84–2.93) | 6.51× (6.44–6.57) | 6.72× (6.63–6.81) | 7.28× (7.22–7.35) | 6.57× (6.49–6.66) | 7.17× (7.07–7.26) | 6.71× (6.63–6.79) | 2.12× (2.10–2.14) | 2.49× (2.46–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.303 s | 692 | 9.0 | 2.82× (2.68–2.98) | 5.94× (5.58–6.32) | 6.82× (6.49–7.16) | 6.62× (6.31–6.95) | 6.07× (5.79–6.37) | 12.33× (11.76–12.92) | 11.54× (10.99–12.12) | 3.65× (3.47–3.83) | 4.28× (4.07–4.49) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.247 s | 848 | 13.8 | 2.92× (2.85–2.99) | 4.38× (4.29–4.46) | 5.48× (5.30–5.66) | 4.94× (4.85–5.02) | 5.05× (4.95–5.16) | 15.35× (15.09–15.61) | 14.37× (14.09–14.65) | 4.54× (4.47–4.62) | 5.32× (5.24–5.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 983 | 3.6 | 0.60× (0.59–0.61) | 3.65× (3.48–3.83) | 4.57× (4.44–4.70) | 3.59× (3.54–3.65) | 6.20× (6.12–6.28) | 2.98× (2.93–3.04) | 2.67× (2.62–2.71) | 1.02× (1.00–1.03) | 1.51× (1.48–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 971 | 3.7 | 0.57× (0.55–0.58) | 2.97× (2.91–3.03) | 6.00× (5.80–6.20) | 3.54× (3.48–3.62) | 6.05× (5.95–6.16) | 2.93× (2.87–3.00) | 2.62× (2.57–2.68) | 1.00× (0.97–1.03) | 1.49× (1.45–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 962 | 3.7 | 0.58× (0.56–0.59) | 2.97× (2.94–3.01) | 6.04× (5.84–6.24) | 3.53× (3.49–3.58) | 6.07× (5.96–6.18) | 2.93× (2.89–2.97) | 2.62× (2.58–2.65) | 1.00× (0.97–1.02) | 1.48× (1.46–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 581 | 5.9 | 2.93× (2.80–3.07) | 3.03× (3.00–3.05) | 3.88× (3.85–3.91) | 3.83× (3.80–3.86) | 4.17× (4.15–4.20) | 2.56× (2.54–2.58) | 2.12× (2.10–2.15) | 2.03× (2.01–2.05) | 3.01× (2.99–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 871 | 7.5 | 2.24× (2.13–2.36) | 2.71× (2.61–2.81) | 4.03× (3.78–4.31) | 3.39× (3.16–3.65) | 4.66× (4.45–4.88) | 3.73× (3.57–3.90) | 3.09× (2.96–3.24) | 2.96× (2.81–3.11) | 4.39× (4.19–4.60) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 775 | 14.6 | 1.86× (1.82–1.90) | 2.02× (1.90–2.15) | 2.92× (2.81–3.03) | 2.21× (2.10–2.34) | 3.91× (3.83–4.00) | 3.41× (3.34–3.48) | 2.83× (2.77–2.88) | 2.70× (2.64–2.76) | 4.01× (3.92–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 60 | 54.5 | 1.37× (1.36–1.39) | 2.71× (2.68–2.74) | 2.46× (2.41–2.52) | 2.44× (2.41–2.46) | 2.85× (2.82–2.87) | 3.13× (3.10–3.16) | 3.16× (3.12–3.19) | 1.24× (1.22–1.25) | 1.43× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 61.5 | 1.30× (1.28–1.32) | 2.52× (2.46–2.58) | 2.38× (2.34–2.42) | 2.96× (2.92–3.00) | 2.99× (2.95–3.03) | 5.10× (5.03–5.18) | 5.15× (5.08–5.21) | 2.02× (1.97–2.06) | 2.33× (2.26–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 85.7 | 1.31× (1.27–1.36) | 2.11× (2.05–2.16) | 2.06× (1.99–2.12) | 2.45× (2.37–2.53) | 2.81× (2.76–2.87) | 5.93× (5.78–6.08) | 5.98× (5.84–6.12) | 2.34× (2.28–2.40) | 2.70× (2.60–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 50.0 | 1.49× (1.48–1.51) | 3.01× (2.99–3.03) | 2.56× (2.53–2.59) | 2.59× (2.57–2.61) | 3.09× (3.01–3.18) | 3.60× (3.57–3.64) | 3.90× (3.88–3.93) | 1.41× (1.39–1.44) | 1.72× (1.70–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 111 | 56.2 | 1.39× (1.33–1.45) | 2.78× (2.69–2.88) | 2.48× (2.38–2.58) | 3.24× (3.14–3.33) | 3.09× (3.00–3.19) | 5.96× (5.78–6.14) | 6.46× (6.27–6.65) | 2.34× (2.26–2.42) | 2.85× (2.78–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 124 | 81.8 | 1.42× (1.38–1.46) | 2.28× (2.24–2.33) | 2.18× (2.11–2.26) | 2.80× (2.73–2.87) | 2.76× (2.70–2.82) | 6.75× (6.62–6.89) | 7.31× (7.16–7.47) | 2.65× (2.58–2.71) | 3.23× (3.15–3.31) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 5.1 | 0.67× (0.66–0.67) | 3.04× (3.02–3.07) | 4.44× (4.40–4.48) | 3.13× (3.10–3.16) | 6.82× (6.74–6.90) | 2.54× (2.52–2.57) | 2.35× (2.32–2.37) | 0.98× (0.96–0.99) | 1.53× (1.52–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 697 | 5.4 | 0.56× (0.55–0.57) | 2.86× (2.81–2.91) | 5.01× (4.82–5.20) | 2.98× (2.94–3.03) | 6.47× (6.34–6.61) | 2.39× (2.35–2.43) | 2.20× (2.17–2.23) | 0.92× (0.90–0.93) | 1.44× (1.41–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.3 | 0.56× (0.55–0.57) | 2.86× (2.82–2.90) | 4.95× (4.83–5.06) | 2.97× (2.93–3.01) | 6.45× (6.36–6.54) | 2.38× (2.34–2.41) | 2.19× (2.16–2.23) | 0.91× (0.90–0.93) | 1.43× (1.41–1.45) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.070 s | 182 | 18.4 | 1.96× (1.94–1.97) | 3.98× (3.94–4.01) | 4.38× (4.34–4.42) | 4.41× (4.37–4.45) | 4.66× (4.61–4.72) | 5.15× (5.11–5.20) | 4.84× (4.79–4.89) | 1.49× (1.47–1.50) | 1.91× (1.89–1.92) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.049 s | 258 | 23.1 | 1.49× (1.36–1.63) | 3.04× (2.82–3.27) | 3.56× (3.24–3.91) | 3.88× (3.50–4.31) | 3.91× (3.55–4.31) | 6.93× (6.31–7.61) | 6.50× (5.91–7.16) | 2.00× (1.82–2.19) | 2.56× (2.33–2.81) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.044 s | 290 | 35.5 | 1.50× (1.44–1.56) | 2.35× (2.23–2.48) | 3.10× (3.03–3.18) | 3.25× (3.15–3.35) | 3.66× (3.58–3.74) | 8.16× (7.99–8.34) | 7.66× (7.48–7.84) | 2.35× (2.30–2.40) | 3.02× (2.95–3.08) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.83–2.92) | 6.47× (6.41–6.54) | 6.69× (6.60–6.78) | 7.26× (7.19–7.33) | 6.55× (6.46–6.64) | 7.14× (7.04–7.24) | 6.69× (6.61–6.77) | 2.12× (2.09–2.14) | 2.47× (2.44–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.71–2.78) | 6.10× (5.99–6.22) | 6.45× (6.36–6.53) | 6.96× (6.86–7.05) | 6.06× (5.99–6.13) | 6.52× (6.43–6.62) | 6.11× (6.02–6.20) | 1.93× (1.91–1.96) | 2.26× (2.23–2.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.86–2.93) | 4.76× (4.67–4.84) | 5.12× (5.03–5.21) | 5.38× (5.30–5.45) | 4.83× (4.76–4.90) | 4.29× (4.23–4.35) | 4.02× (3.97–4.07) | 1.27× (1.26–1.28) | 1.48× (1.47–1.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.27× (3.22–3.32) | 4.09× (4.05–4.13) | 3.36× (3.31–3.40) | 5.73× (5.65–5.81) | 2.80× (2.76–2.85) | 2.52× (2.48–2.56) | 1.02× (1.01–1.03) | 1.08× (1.06–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.64) | 3.06× (3.00–3.12) | 5.35× (5.27–5.44) | 3.29× (3.24–3.35) | 5.53× (5.45–5.61) | 2.72× (2.67–2.77) | 2.44× (2.40–2.49) | 0.99× (0.96–1.01) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.40× (3.32–3.49) | 5.44× (5.35–5.53) | 3.33× (3.29–3.37) | 5.60× (5.50–5.70) | 2.75× (2.71–2.79) | 2.47× (2.44–2.50) | 1.00× (0.98–1.02) | 1.06× (1.03–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.82× (2.69–2.95) | 2.79× (2.77–2.82) | 3.52× (3.49–3.54) | 3.66× (3.63–3.70) | 3.99× (3.97–4.01) | 2.47× (2.44–2.49) | 2.06× (2.03–2.08) | 1.97× (1.95–1.99) | 2.72× (2.69–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.23–2.34) | 2.47× (2.41–2.54) | 3.04× (2.98–3.09) | 3.15× (3.09–3.20) | 3.33× (3.29–3.38) | 1.98× (1.94–2.01) | 1.65× (1.62–1.67) | 1.58× (1.55–1.60) | 2.18× (2.14–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.92–2.03) | 1.71× (1.65–1.76) | 2.17× (2.11–2.23) | 2.04× (2.00–2.08) | 2.08× (2.02–2.13) | 1.00× (0.98–1.03) | 0.84× (0.82–0.86) | 0.80× (0.78–0.82) | 1.11× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.35–1.38) | 2.63× (2.60–2.65) | 2.40× (2.36–2.44) | 2.41× (2.39–2.44) | 2.82× (2.79–2.84) | 3.10× (3.07–3.13) | 3.12× (3.09–3.16) | 1.23× (1.22–1.25) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.29) | 2.45× (2.41–2.48) | 2.17× (2.15–2.18) | 2.74× (2.71–2.76) | 2.56× (2.54–2.59) | 2.72× (2.70–2.74) | 2.75× (2.72–2.77) | 1.08× (1.07–1.10) | 1.21× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.38–1.47) | 2.25× (2.20–2.31) | 1.83× (1.77–1.90) | 2.40× (2.35–2.46) | 2.38× (2.34–2.42) | 1.98× (1.93–2.02) | 2.00× (1.95–2.04) | 0.79× (0.77–0.81) | 0.88× (0.86–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.47–1.50) | 2.96× (2.94–2.99) | 2.51× (2.47–2.54) | 2.57× (2.55–2.59) | 3.06× (2.98–3.15) | 3.57× (3.54–3.60) | 3.86× (3.84–3.89) | 1.41× (1.39–1.43) | 1.68× (1.66–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.40) | 2.77× (2.76–2.79) | 2.31× (2.29–2.33) | 3.07× (3.03–3.11) | 2.78× (2.75–2.80) | 3.18× (3.16–3.21) | 3.45× (3.42–3.48) | 1.25× (1.24–1.27) | 1.50× (1.48–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.55–1.64) | 2.51× (2.44–2.59) | 1.87× (1.81–1.94) | 2.62× (2.55–2.69) | 2.48× (2.41–2.56) | 2.20× (2.15–2.25) | 2.38× (2.32–2.45) | 0.87× (0.84–0.90) | 1.03× (1.01–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.74× (2.70–2.77) | 3.57× (3.53–3.61) | 2.85× (2.82–2.87) | 6.04× (5.97–6.11) | 2.34× (2.31–2.36) | 2.17× (2.14–2.19) | 0.98× (0.97–0.99) | 1.02× (0.98–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.53× (2.44–2.62) | 4.26× (4.20–4.32) | 2.75× (2.71–2.78) | 5.71× (5.60–5.82) | 2.17× (2.14–2.21) | 2.01× (1.99–2.04) | 0.91× (0.89–0.92) | 0.94× (0.90–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.76× (2.72–2.80) | 4.32× (4.25–4.38) | 2.81× (2.78–2.85) | 5.82× (5.75–5.89) | 2.21× (2.19–2.24) | 2.05× (2.03–2.08) | 0.93× (0.91–0.94) | 0.96× (0.92–1.00) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.93× (1.91–1.94) | 3.88× (3.85–3.91) | 4.23× (4.20–4.27) | 4.31× (4.27–4.35) | 4.56× (4.51–4.61) | 5.04× (4.99–5.08) | 4.73× (4.69–4.77) | 1.47× (1.46–1.48) | 1.78× (1.75–1.80) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.67× (1.64–1.69) | 3.25× (3.18–3.32) | 3.56× (3.47–3.64) | 3.88× (3.80–3.96) | 3.70× (3.63–3.77) | 3.99× (3.91–4.08) | 3.75× (3.67–3.83) | 1.17× (1.14–1.20) | 1.41× (1.38–1.44) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.75× (1.65–1.85) | 2.69× (2.59–2.79) | 2.95× (2.84–3.06) | 3.14× (3.04–3.24) | 3.10× (2.98–3.23) | 2.65× (2.55–2.74) | 2.49× (2.40–2.57) | 0.77× (0.74–0.80) | 0.93× (0.90–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.28–0.31) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.06× (11.03–11.09) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.30–0.33) | 0.43× (0.37–0.50) | 0.66× (0.63–0.68) | 0.55× (0.53–0.57) | 5.23× (5.03–5.44) | 0.12× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 97 MB | 0.32× (0.31–0.33) | 0.58× (0.49–0.68) | 0.57× (0.55–0.58) | 0.49× (0.47–0.51) | 3.32× (3.23–3.42) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.17) | 1.77× (1.76–1.78) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (8.00–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.69× (0.65–0.72) | 1.61× (1.58–1.64) | 0.80× (0.78–0.81) | 5.83× (5.73–5.93) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.63–0.68) | 0.75× (0.70–0.80) | 1.16× (1.13–1.19) | 0.71× (0.69–0.72) | 3.43× (3.36–3.51) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.39× (0.36–0.43) | 0.92× (0.89–0.94) | 0.73× (0.70–0.76) | 3.17× (3.08–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.46× (0.44–0.47) | 0.87× (0.86–0.89) | 0.80× (0.77–0.84) | 2.54× (2.48–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.21–5.24) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.37× (0.34–0.41) | 0.86× (0.83–0.89) | 0.71× (0.67–0.75) | 2.97× (2.86–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.47–0.52) | 0.80× (0.79–0.81) | 0.81× (0.78–0.84) | 2.30× (2.26–2.33) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.39× (5.36–5.41) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.76) | 5.28× (5.26–5.30) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.76× (0.76–0.77) | 5.31× (5.30–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.43) | 0.67× (0.66–0.67) | 6.40× (6.38–6.43) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.41× (0.38–0.45) | 1.06× (1.03–1.08) | 0.68× (0.67–0.70) | 3.29× (3.22–3.36) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.49× (0.46–0.53) | 0.92× (0.89–0.95) | 0.75× (0.73–0.77) | 2.32× (2.26–2.39) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 396 | 2.54 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 692 | 2.77 | 1.09× (1.08–1.11) | 86% | 1.03× (1.02–1.04) | 1.04× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 848 | 4.26 | 1.67× (1.64–1.69) | 54% | 1.22× (1.21–1.24) | 1.67× (1.65–1.70) |

**Best:** mibz2 at 4 threads (848 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.16, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.69 | 0.02 | 0.09 | 0.247 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.6% |
| MTF + RLE2 | 1.52 | 17.0% |
| T-vector build | 1.51 | 17.0% |
| walk (the one each build uses) | 4.21 | 47.2% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.92 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.486 | 1.303 | (in its total) | 2.68× (2.67–2.69) |  |
| walk | 1.239 | 2.636 | (in its total) | 2.12× (2.11–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.056 | (in its total) | 1.26× (1.24–1.28) |  |
| inverse BWT, total | 1.769 | 3.996 | 3.265 | 2.25× (2.24–2.26) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m58s |
| 4 threads | 36m06s ± 5m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m04s ± 3m33s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 25m04s ± 3m33s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 2.09 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,509,359 | 12,625,378 | \`78d0692041033556…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
