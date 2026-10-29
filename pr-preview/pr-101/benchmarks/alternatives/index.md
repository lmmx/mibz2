# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-101/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`06a2221`](<https://github.com/lmmx/mibz2/commit/06a22213d94d029f10e139100b4165dfcc398bfb>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40898300411>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.814 s | 258 | 10.3 | 1.90× (1.88–1.92) | 4.27× (4.22–4.32) | 4.50× (4.45–4.54) | 4.79× (4.73–4.85) | 4.35× (4.30–4.41) | 4.62× (4.55–4.69) | 4.32× (4.27–4.36) | 1.39× (1.38–1.41) | 1.63× (1.61–1.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.526 s | 399 | 11.4 | 1.68× (1.61–1.75) | 3.56× (3.48–3.65) | 4.12× (4.01–4.23) | 4.00× (3.92–4.09) | 3.66× (3.59–3.73) | 7.19× (7.03–7.36) | 6.72× (6.58–6.86) | 2.17× (2.12–2.22) | 2.54× (2.49–2.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.412 s | 509 | 17.4 | 1.76× (1.72–1.79) | 2.73× (2.71–2.76) | 3.38× (3.32–3.45) | 3.06× (3.00–3.12) | 3.09× (3.04–3.13) | 9.17× (9.06–9.28) | 8.56× (8.45–8.67) | 2.76× (2.73–2.80) | 3.24× (3.19–3.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 922 | 3.1 | 0.56× (0.54–0.59) | 3.66× (3.47–3.86) | 4.58× (4.36–4.80) | 3.35× (3.19–3.51) | 6.31× (6.02–6.62) | 2.63× (2.51–2.76) | 2.47× (2.35–2.59) | 1.01× (0.96–1.06) | 1.40× (1.33–1.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 920 | 3.2 | 0.56× (0.55–0.57) | 2.85× (2.81–2.88) | 6.10× (6.03–6.18) | 3.43× (3.39–3.48) | 6.39× (6.30–6.47) | 2.69× (2.66–2.72) | 2.52× (2.49–2.55) | 1.03× (1.01–1.05) | 1.43× (1.41–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 901 | 3.2 | 0.55× (0.54–0.56) | 2.79× (2.76–2.82) | 5.96× (5.89–6.03) | 3.36× (3.31–3.40) | 6.32× (6.24–6.40) | 2.62× (2.59–2.65) | 2.46× (2.43–2.49) | 1.00× (0.98–1.03) | 1.39× (1.37–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.046 s | 423 | 6.5 | 1.92× (1.88–1.95) | 2.20× (2.15–2.24) | 2.79× (2.67–2.92) | 2.77× (2.73–2.82) | 3.28× (3.22–3.34) | 1.81× (1.78–1.85) | 1.54× (1.51–1.57) | 1.52× (1.49–1.55) | 2.19× (2.15–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.029 s | 683 | 7.8 | 1.74× (1.62–1.87) | 2.23× (2.06–2.42) | 3.19× (3.03–3.36) | 2.63× (2.52–2.75) | 4.13× (3.92–4.35) | 2.88× (2.72–3.04) | 2.44× (2.31–2.57) | 2.40× (2.28–2.53) | 3.47× (3.28–3.66) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.032 s | 621 | 15.1 | 1.50× (1.47–1.52) | 1.64× (1.57–1.72) | 2.52× (2.40–2.64) | 1.82× (1.79–1.87) | 3.55× (3.50–3.61) | 2.69× (2.66–2.72) | 2.28× (2.25–2.30) | 2.25× (2.23–2.27) | 3.24× (3.20–3.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.157 s | 51 | 52.7 | 1.15× (1.14–1.16) | 2.31× (2.28–2.33) | 2.07× (2.05–2.09) | 2.07× (2.05–2.08) | 2.48× (2.45–2.50) | 2.62× (2.60–2.64) | 2.60× (2.58–2.62) | 1.04× (1.03–1.05) | 1.17× (1.16–1.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.095 s | 84 | 57.7 | 1.10× (1.07–1.14) | 2.16× (2.10–2.22) | 2.04× (1.96–2.13) | 2.61× (2.52–2.71) | 2.63× (2.54–2.73) | 4.29× (4.15–4.44) | 4.26× (4.12–4.40) | 1.70× (1.65–1.77) | 1.91× (1.85–1.98) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 106 | 75.5 | 1.23× (1.19–1.26) | 1.94× (1.91–1.98) | 1.94× (1.87–2.00) | 2.40× (2.30–2.52) | 2.77× (2.71–2.83) | 5.42× (5.29–5.55) | 5.38× (5.25–5.50) | 2.15× (2.10–2.21) | 2.41× (2.36–2.47) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.206 s | 46 | 57.5 | 1.06× (1.05–1.07) | 2.25× (2.23–2.27) | 1.84× (1.82–1.85) | 1.84× (1.83–1.85) | 2.27× (2.19–2.35) | 2.66× (2.63–2.68) | 2.73× (2.71–2.74) | 1.01× (1.00–1.02) | 1.15× (1.13–1.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.123 s | 78 | 62.4 | 1.00× (0.98–1.02) | 2.13× (2.04–2.21) | 1.86× (1.82–1.90) | 2.56× (2.49–2.64) | 2.40× (2.33–2.48) | 4.50× (4.39–4.60) | 4.62× (4.51–4.73) | 1.71× (1.66–1.75) | 1.94× (1.90–1.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.094 s | 102 | 83.5 | 1.19× (1.18–1.21) | 1.93× (1.87–1.98) | 1.85× (1.78–1.92) | 2.51× (2.46–2.56) | 2.47× (2.43–2.51) | 5.82× (5.74–5.91) | 5.98× (5.88–6.08) | 2.21× (2.18–2.24) | 2.52× (2.47–2.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 667 | 4.5 | 0.63× (0.59–0.68) | 3.20× (2.97–3.45) | 4.03× (3.96–4.09) | 2.88× (2.83–2.93) | 7.04× (6.93–7.16) | 2.25× (2.22–2.29) | 2.11× (2.08–2.14) | 0.97× (0.94–1.00) | 1.38× (1.36–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 656 | 4.7 | 0.54× (0.53–0.54) | 2.70× (2.66–2.73) | 5.11× (4.88–5.34) | 2.84× (2.80–2.89) | 6.90× (6.78–7.03) | 2.21× (2.18–2.25) | 2.08× (2.04–2.11) | 0.95× (0.93–0.98) | 1.36× (1.34–1.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 4.6 | 0.54× (0.53–0.56) | 2.68× (2.62–2.75) | 5.00× (4.76–5.26) | 2.82× (2.76–2.88) | 6.84× (6.66–7.02) | 2.20× (2.15–2.25) | 2.06× (2.01–2.11) | 0.95× (0.93–0.96) | 1.35× (1.32–1.38) | yes |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 1 | 0.092 s | 134 | 20.2 | 1.33× (1.31–1.35) | 2.77× (2.73–2.82) | 3.08× (3.03–3.12) | 2.95× (2.91–3.00) | 3.29× (3.24–3.35) | 3.18× (3.13–3.22) | 2.98× (2.93–3.04) | 0.97× (0.96–0.98) | 1.22× (1.20–1.23) | yes |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 2 | 0.057 s | 217 | 23.1 | 1.17× (1.12–1.23) | 2.45× (2.32–2.58) | 2.97× (2.84–3.10) | 3.08× (2.92–3.25) | 3.38× (3.25–3.52) | 5.03× (4.83–5.24) | 4.73× (4.53–4.93) | 1.54× (1.48–1.60) | 1.93× (1.85–2.01) | yes |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 4 | 0.046 s | 266 | 34.0 | 1.25× (1.21–1.29) | 2.09× (1.98–2.22) | 2.77× (2.71–2.83) | 2.84× (2.73–2.95) | 3.40× (3.34–3.47) | 6.31× (6.22–6.40) | 5.93× (5.84–6.01) | 1.93× (1.90–1.96) | 2.42× (2.38–2.45) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.90× (1.87–1.92) | 4.25× (4.20–4.30) | 4.48× (4.44–4.53) | 4.78× (4.72–4.84) | 4.35× (4.29–4.40) | 4.61× (4.55–4.68) | 4.31× (4.26–4.35) | 1.39× (1.38–1.41) | 1.62× (1.61–1.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.79× (1.75–1.82) | 3.96× (3.89–4.05) | 4.27× (4.18–4.36) | 4.57× (4.49–4.66) | 3.96× (3.90–4.03) | 4.15× (4.07–4.23) | 3.87× (3.80–3.95) | 1.25× (1.23–1.28) | 1.46× (1.43–1.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.87× (1.83–1.91) | 3.17× (3.12–3.23) | 3.42× (3.36–3.49) | 3.58× (3.51–3.64) | 3.16× (3.12–3.21) | 2.74× (2.72–2.77) | 2.56× (2.53–2.59) | 0.83× (0.82–0.84) | 0.97× (0.95–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.57–0.63) | 3.10× (2.96–3.24) | 4.01× (3.83–4.20) | 3.16× (3.01–3.31) | 5.88× (5.61–6.17) | 2.50× (2.38–2.63) | 2.35× (2.25–2.47) | 1.01× (0.97–1.06) | 1.04× (0.99–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.98× (2.96–3.01) | 5.32× (5.28–5.37) | 3.20× (3.16–3.23) | 5.84× (5.78–5.91) | 2.50× (2.47–2.53) | 2.35× (2.33–2.38) | 1.01× (0.99–1.03) | 1.04× (1.01–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.27× (3.23–3.31) | 5.28× (5.22–5.35) | 3.20× (3.16–3.23) | 5.88× (5.81–5.95) | 2.49× (2.46–2.52) | 2.34× (2.32–2.37) | 1.01× (0.98–1.03) | 1.03× (1.01–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.88× (1.85–1.91) | 2.12× (2.08–2.15) | 2.60× (2.50–2.70) | 2.70× (2.65–2.74) | 3.18× (3.13–3.24) | 1.78× (1.75–1.81) | 1.51× (1.48–1.54) | 1.49× (1.46–1.52) | 2.02× (1.98–2.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.73× (1.69–1.76) | 1.93× (1.88–1.97) | 2.43× (2.38–2.47) | 2.41× (2.36–2.46) | 2.80× (2.75–2.86) | 1.50× (1.46–1.53) | 1.27× (1.25–1.30) | 1.26× (1.23–1.28) | 1.70× (1.66–1.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.56× (1.50–1.62) | 1.46× (1.41–1.52) | 1.76× (1.71–1.81) | 1.66× (1.61–1.72) | 1.74× (1.69–1.79) | 0.78× (0.76–0.80) | 0.66× (0.64–0.68) | 0.65× (0.64–0.67) | 0.88× (0.86–0.90) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.15× (1.14–1.16) | 2.27× (2.25–2.28) | 2.03× (2.02–2.05) | 2.05× (2.04–2.06) | 2.46× (2.44–2.48) | 2.60× (2.58–2.62) | 2.58× (2.56–2.60) | 1.04× (1.03–1.05) | 1.12× (1.11–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.12× (1.11–1.13) | 2.20× (2.18–2.22) | 1.94× (1.93–1.96) | 2.53× (2.51–2.54) | 2.31× (2.30–2.32) | 2.38× (2.37–2.39) | 2.36× (2.35–2.37) | 0.95× (0.94–0.96) | 1.02× (1.02–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.30–1.42) | 2.09× (2.05–2.14) | 1.74× (1.71–1.77) | 2.35× (2.27–2.43) | 2.31× (2.26–2.36) | 1.83× (1.79–1.86) | 1.81× (1.78–1.85) | 0.73× (0.71–0.75) | 0.79× (0.77–0.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.06× (1.05–1.07) | 2.22× (2.21–2.23) | 1.81× (1.79–1.82) | 1.83× (1.82–1.84) | 2.26× (2.18–2.34) | 2.64× (2.62–2.66) | 2.71× (2.70–2.72) | 1.01× (1.00–1.02) | 1.13× (1.12–1.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.05) | 2.15× (2.13–2.18) | 1.76× (1.73–1.78) | 2.46× (2.43–2.49) | 2.13× (2.11–2.15) | 2.43× (2.42–2.44) | 2.49× (2.47–2.52) | 0.93× (0.91–0.94) | 1.04× (1.03–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.30× (1.28–1.32) | 2.05× (2.00–2.10) | 1.58× (1.55–1.60) | 2.27× (2.23–2.32) | 2.08× (2.05–2.11) | 1.82× (1.80–1.83) | 1.87× (1.85–1.89) | 0.69× (0.68–0.70) | 0.78× (0.77–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.55× (2.52–2.58) | 3.45× (3.41–3.49) | 2.66× (2.62–2.70) | 6.34× (6.26–6.43) | 2.11× (2.09–2.13) | 1.99× (1.96–2.01) | 0.97× (0.95–0.98) | 0.95× (0.92–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.40× (2.36–2.43) | 4.17× (4.12–4.22) | 2.62× (2.58–2.66) | 6.07× (5.98–6.17) | 2.02× (1.99–2.04) | 1.90× (1.87–1.93) | 0.92× (0.91–0.94) | 0.91× (0.88–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.61) | 2.67× (2.60–2.74) | 4.24× (4.15–4.33) | 2.71× (2.66–2.76) | 6.23× (6.08–6.38) | 2.07× (2.03–2.12) | 1.95× (1.91–1.99) | 0.95× (0.92–0.97) | 0.93× (0.91–0.96) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 1 | 1.32× (1.30–1.35) | 2.67× (2.63–2.71) | 2.98× (2.93–3.03) | 2.91× (2.87–2.96) | 3.25× (3.19–3.30) | 3.13× (3.09–3.17) | 2.94× (2.89–2.99) | 0.97× (0.96–0.99) | 1.17× (1.16–1.19) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 2 | 1.25× (1.24–1.26) | 2.45× (2.43–2.47) | 2.78× (2.74–2.82) | 2.84× (2.80–2.88) | 2.91× (2.87–2.94) | 2.73× (2.70–2.75) | 2.56× (2.53–2.59) | 0.85× (0.84–0.85) | 1.02× (1.01–1.03) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 4 | 1.37× (1.32–1.42) | 2.08× (2.04–2.12) | 2.39× (2.34–2.44) | 2.37× (2.32–2.42) | 2.42× (2.36–2.49) | 1.86× (1.84–1.88) | 1.75× (1.73–1.77) | 0.58× (0.57–0.58) | 0.70× (0.69–0.71) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.28–0.31) | 1.17× (1.17–1.18) | 0.84× (0.84–0.84) | 11.89× (11.88–11.91) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 50 MB | 0.37× (0.37–0.38) | 0.57× (0.50–0.64) | 0.86× (0.85–0.87) | 0.67× (0.65–0.68) | 6.30× (6.21–6.39) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.76× (0.67–0.87) | 0.79× (0.78–0.80) | 0.64× (0.63–0.65) | 4.30× (4.28–4.31) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.80× (0.80–0.80) | 4.29× (4.29–4.30) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.91× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.18–0.19) | 0.16× (0.15–0.16) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.48–0.49) | 0.43× (0.42–0.43) | 1.64× (1.64–1.65) | 0.63× (0.63–0.64) | 7.81× (7.79–7.83) | 0.62× (0.61–0.62) | 0.65× (0.64–0.65) | 0.71× (0.70–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.62× (0.61–0.64) | 0.64× (0.59–0.70) | 1.54× (1.52–1.56) | 0.87× (0.85–0.88) | 5.58× (5.51–5.65) | 0.44× (0.43–0.44) | 0.46× (0.45–0.47) | 0.51× (0.50–0.51) | 0.25× (0.24–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.60× (0.59–0.61) | 0.64× (0.60–0.69) | 1.12× (1.10–1.13) | 0.86× (0.85–0.87) | 3.25× (3.21–3.28) | 0.25× (0.24–0.25) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.38× (0.38–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.71× (0.71–0.71) | 5.08× (5.07–5.09) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.44× (0.43–0.44) | 0.93× (0.92–0.94) | 0.73× (0.71–0.76) | 3.25× (3.22–3.28) | 0.20× (0.19–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.58–0.61) | 0.42× (0.41–0.43) | 0.81× (0.80–0.83) | 0.75× (0.71–0.79) | 2.38× (2.33–2.43) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.81× (0.80–0.81) | 5.11× (5.11–5.12) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.50) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.38× (0.36–0.41) | 0.87× (0.87–0.88) | 0.71× (0.69–0.74) | 3.03× (3.00–3.05) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.54× (0.53–0.55) | 0.43× (0.41–0.46) | 0.74× (0.72–0.75) | 0.74× (0.73–0.76) | 2.14× (2.10–2.18) | 0.12× (0.12–0.13) | 0.13× (0.13–0.13) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.09× (1.09–1.09) | 0.76× (0.76–0.76) | 5.43× (5.43–5.44) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.19–0.20) | 1.63× (1.62–1.63) | 0.77× (0.76–0.77) | 5.40× (5.39–5.41) | 0.34× (0.34–0.34) | 0.36× (0.36–0.36) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.64) | 0.77× (0.77–0.77) | 5.41× (5.40–5.43) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.39× (0.39–0.39) | 1.45× (1.45–1.45) | 0.62× (0.62–0.62) | 6.55× (6.55–6.55) | 0.39× (0.38–0.39) | 0.41× (0.41–0.41) | 0.64× (0.64–0.64) | 0.31× (0.31–0.31) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 2 | 31 MB | 0.52× (0.52–0.53) | 0.48× (0.43–0.52) | 1.34× (1.30–1.38) | 0.76× (0.75–0.77) | 3.91× (3.87–3.96) | 0.23× (0.23–0.23) | 0.25× (0.24–0.25) | 0.38× (0.38–0.39) | 0.19× (0.18–0.19) |
| `text` (1.5 MB → 12.4 MB, 8.2×) | 4 | 51 MB | 0.53× (0.52–0.54) | 0.50× (0.47–0.53) | 1.07× (1.05–1.09) | 0.78× (0.75–0.80) | 2.54× (2.51–2.57) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.24× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 258 | 3.89 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 399 | 4.32 | 1.11× (1.08–1.14) | 78% | 1.04× (1.03–1.05) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 509 | 6.58 | 1.68× (1.65–1.71) | 50% | 1.25× (1.24–1.27) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (509 MB/s); lbzip2 at 4 threads (185 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.63, mibz2 4t 0.94, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.52, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.16 | 0.02 | 0.08 | 0.412 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2648 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m14s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.00 | 10.0% |
| MTF + RLE2 | 1.84 | 18.4% |
| T-vector build | 1.86 | 18.6% |
| walk (the one each build uses) | 4.86 | 48.6% |
| of which: segment assembly | 0.16 | 1.6% |
| RLE1 | 0.23 | 2.3% |
| CRC | 0.21 | 2.1% |
| **total** | 10.00 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.695 | 1.304 | (in its total) | 1.88× (1.87–1.88) |  |
| walk | 1.674 | 2.679 | (in its total) | 1.59× (1.57–1.62) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.056 | 0.058 | (in its total) | 1.05× (1.03–1.07) |  |
| inverse BWT, total | 2.424 | 4.042 | 3.501 | 1.66× (1.64–1.68) | 1.44× (1.43–1.46) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h58m ± 14m23s |
| 4 threads | 59m57s ± 8m26s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m12s ± 5m55s |
| the scanner alone (its serial part) | 16m35s ± 32.2s |
| the workers alone, over 4 | 42m12s ± 5m55s |

At 4 threads the workers bound it. Per byte: the scanner 9.6 ns a byte of bz2, the workers 5.53 ns a byte of output at 4 threads (CPU, summed over them), 3.35 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,502,031 | 12,368,727 | \`b92cb4c4cbeb8dfc…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2648.5 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
