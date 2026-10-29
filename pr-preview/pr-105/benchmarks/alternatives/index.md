# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-105/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`fa5e887`](<https://github.com/lmmx/mibz2/commit/fa5e88763d330351fc8ee88b6d67c1e6d2d06879>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40914822342>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.806 s | 260 | 12.4 | 1.91× (1.90–1.92) | 4.28× (4.25–4.31) | 4.52× (4.48–4.55) | 4.86× (4.82–4.91) | 4.34× (4.29–4.40) | 4.64× (4.58–4.70) | 4.41× (4.37–4.46) | 1.40× (1.39–1.41) | 1.63× (1.61–1.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.521 s | 402 | 13.9 | 1.68× (1.65–1.72) | 3.51× (3.41–3.62) | 4.14× (4.01–4.26) | 4.10× (3.95–4.25) | 3.64× (3.55–3.73) | 7.14× (6.95–7.32) | 6.79× (6.62–6.96) | 2.15× (2.09–2.20) | 2.51× (2.44–2.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.411 s | 510 | 21.2 | 1.73× (1.69–1.76) | 2.68× (2.62–2.74) | 3.32× (3.24–3.40) | 3.01× (2.95–3.06) | 3.07× (3.01–3.13) | 9.03× (8.89–9.18) | 8.59× (8.45–8.74) | 2.72× (2.67–2.77) | 3.17× (3.12–3.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 913 | 3.9 | 0.57× (0.56–0.58) | 3.70× (3.55–3.85) | 4.62× (4.52–4.72) | 3.36× (3.30–3.43) | 6.31× (6.17–6.45) | 2.66× (2.61–2.72) | 2.49× (2.44–2.55) | 1.02× (1.00–1.04) | 1.41× (1.38–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 920 | 3.9 | 0.57× (0.55–0.58) | 2.85× (2.81–2.90) | 6.17× (6.03–6.32) | 3.44× (3.38–3.50) | 6.38× (6.26–6.50) | 2.70× (2.65–2.74) | 2.53× (2.48–2.57) | 1.04× (1.01–1.06) | 1.43× (1.41–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 911 | 3.9 | 0.57× (0.56–0.58) | 2.82× (2.79–2.86) | 6.03× (5.95–6.11) | 3.39× (3.33–3.46) | 6.33× (6.25–6.42) | 2.66× (2.62–2.69) | 2.49× (2.45–2.52) | 1.02× (1.00–1.04) | 1.41× (1.39–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.046 s | 430 | 7.8 | 1.97× (1.94–2.00) | 2.25× (2.23–2.28) | 2.85× (2.73–2.98) | 2.94× (2.90–2.98) | 3.37× (3.32–3.42) | 1.86× (1.84–1.89) | 1.57× (1.55–1.60) | 1.51× (1.49–1.52) | 2.25× (2.22–2.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.029 s | 685 | 9.3 | 1.80× (1.67–1.93) | 2.38× (2.13–2.66) | 3.19× (2.98–3.41) | 2.77× (2.50–3.07) | 4.12× (3.86–4.40) | 2.90× (2.73–3.07) | 2.45× (2.30–2.60) | 2.34× (2.20–2.50) | 3.49× (3.29–3.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.031 s | 627 | 18.3 | 1.51× (1.46–1.56) | 1.68× (1.62–1.73) | 2.50× (2.37–2.64) | 1.84× (1.77–1.90) | 3.53× (3.42–3.63) | 2.68× (2.61–2.76) | 2.27× (2.20–2.34) | 2.17× (2.11–2.23) | 3.23× (3.14–3.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.155 s | 52 | 63.4 | 1.16× (1.12–1.19) | 2.29× (2.23–2.35) | 2.06× (2.01–2.12) | 2.08× (2.02–2.13) | 2.47× (2.38–2.55) | 2.59× (2.53–2.66) | 2.68× (2.61–2.76) | 1.03× (1.01–1.06) | 1.16× (1.14–1.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.096 s | 84 | 70.6 | 1.12× (1.10–1.13) | 2.18× (2.13–2.24) | 2.10× (2.04–2.16) | 2.64× (2.55–2.74) | 2.62× (2.59–2.66) | 4.25× (4.18–4.33) | 4.41× (4.34–4.47) | 1.69× (1.66–1.73) | 1.91× (1.88–1.95) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.077 s | 104 | 92.8 | 1.22× (1.19–1.25) | 1.98× (1.89–2.07) | 1.90× (1.87–1.92) | 2.34× (2.30–2.38) | 2.74× (2.70–2.78) | 5.32× (5.27–5.37) | 5.51× (5.43–5.59) | 2.12× (2.08–2.15) | 2.39× (2.36–2.41) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.205 s | 47 | 69.6 | 1.05× (1.02–1.08) | 2.22× (2.16–2.28) | 1.81× (1.76–1.87) | 1.84× (1.79–1.90) | 2.19× (2.12–2.26) | 2.63× (2.56–2.71) | 2.79× (2.71–2.87) | 1.00× (0.97–1.03) | 1.16× (1.13–1.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.118 s | 81 | 76.1 | 0.99× (0.92–1.06) | 2.07× (1.94–2.21) | 1.82× (1.70–1.95) | 2.58× (2.43–2.74) | 2.30× (2.15–2.46) | 4.41× (4.13–4.71) | 4.68× (4.38–4.99) | 1.68× (1.56–1.80) | 1.95× (1.82–2.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.093 s | 103 | 101.2 | 1.21× (1.18–1.24) | 1.95× (1.90–2.00) | 1.90× (1.84–1.96) | 2.54× (2.47–2.62) | 2.47× (2.43–2.52) | 5.86× (5.77–5.94) | 6.21× (6.09–6.32) | 2.22× (2.20–2.25) | 2.58× (2.55–2.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 676 | 5.4 | 0.63× (0.62–0.63) | 2.99× (2.75–3.25) | 4.12× (4.08–4.17) | 2.90× (2.87–2.93) | 7.06× (7.00–7.12) | 2.29× (2.27–2.32) | 2.16× (2.14–2.19) | 0.97× (0.95–0.99) | 1.41× (1.39–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 658 | 5.7 | 0.55× (0.54–0.57) | 2.71× (2.67–2.75) | 5.14× (4.87–5.42) | 2.86× (2.81–2.90) | 6.97× (6.83–7.11) | 2.22× (2.19–2.26) | 2.10× (2.06–2.13) | 0.94× (0.92–0.96) | 1.37× (1.34–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 652 | 5.5 | 0.56× (0.55–0.56) | 2.70× (2.67–2.73) | 4.98× (4.73–5.24) | 2.84× (2.81–2.88) | 6.89× (6.82–6.96) | 2.21× (2.18–2.24) | 2.08× (2.06–2.10) | 0.93× (0.91–0.95) | 1.36× (1.34–1.37) | yes |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 1 | 0.091 s | 136 | 24.3 | 1.35× (1.32–1.37) | 2.78× (2.73–2.83) | 3.09× (3.03–3.14) | 2.97× (2.92–3.02) | 3.29× (3.23–3.35) | 3.19× (3.14–3.24) | 3.13× (3.07–3.19) | 1.01× (0.99–1.03) | 1.23× (1.20–1.26) | yes |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 2 | 0.059 s | 212 | 28.6 | 1.18× (1.13–1.23) | 2.38× (2.30–2.47) | 3.00× (2.91–3.10) | 3.14× (2.95–3.33) | 3.34× (3.19–3.49) | 4.98× (4.79–5.17) | 4.88× (4.70–5.07) | 1.58× (1.52–1.64) | 1.92× (1.82–2.02) | yes |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 4 | 0.047 s | 266 | 41.5 | 1.32× (1.26–1.39) | 2.11× (2.03–2.19) | 2.82× (2.70–2.93) | 2.79× (2.72–2.86) | 3.36× (3.29–3.43) | 6.32× (6.23–6.41) | 6.20× (6.12–6.28) | 2.00× (1.96–2.04) | 2.44× (2.39–2.48) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.91× (1.90–1.92) | 4.27× (4.23–4.30) | 4.50× (4.47–4.54) | 4.85× (4.81–4.90) | 4.34× (4.29–4.39) | 4.63× (4.57–4.69) | 4.40× (4.36–4.45) | 1.39× (1.38–1.40) | 1.62× (1.61–1.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.78× (1.74–1.81) | 3.94× (3.85–4.03) | 4.26× (4.17–4.35) | 4.61× (4.51–4.71) | 3.94× (3.88–4.01) | 4.11× (4.02–4.20) | 3.91× (3.83–4.00) | 1.24× (1.21–1.26) | 1.44× (1.41–1.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.88× (1.84–1.91) | 3.14× (3.10–3.18) | 3.38× (3.33–3.43) | 3.54× (3.48–3.59) | 3.15× (3.10–3.20) | 2.71× (2.66–2.76) | 2.58× (2.53–2.62) | 0.82× (0.80–0.83) | 0.95× (0.93–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.62) | 3.13× (3.07–3.19) | 4.05× (3.96–4.13) | 3.17× (3.10–3.23) | 5.87× (5.73–6.02) | 2.52× (2.47–2.58) | 2.37× (2.32–2.43) | 1.02× (1.00–1.04) | 1.08× (1.03–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 2.99× (2.94–3.05) | 5.38× (5.26–5.50) | 3.20× (3.15–3.26) | 5.84× (5.74–5.95) | 2.51× (2.47–2.56) | 2.36× (2.32–2.40) | 1.02× (1.00–1.04) | 1.08× (1.03–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.31× (3.25–3.36) | 5.40× (5.29–5.51) | 3.24× (3.19–3.30) | 5.91× (5.84–5.98) | 2.53× (2.50–2.56) | 2.37× (2.34–2.41) | 1.02× (1.01–1.04) | 1.09× (1.05–1.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.93× (1.90–1.95) | 2.17× (2.14–2.19) | 2.66× (2.57–2.74) | 2.86× (2.83–2.89) | 3.27× (3.22–3.31) | 1.83× (1.80–1.85) | 1.55× (1.53–1.57) | 1.49× (1.47–1.50) | 2.06× (2.03–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.75× (1.71–1.80) | 1.97× (1.90–2.05) | 2.45× (2.38–2.52) | 2.49× (2.42–2.56) | 2.82× (2.74–2.90) | 1.51× (1.47–1.54) | 1.28× (1.25–1.31) | 1.23× (1.19–1.26) | 1.70× (1.65–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.57× (1.48–1.67) | 1.51× (1.44–1.58) | 1.76× (1.68–1.84) | 1.67× (1.61–1.74) | 1.78× (1.71–1.85) | 0.79× (0.75–0.82) | 0.67× (0.64–0.70) | 0.64× (0.61–0.67) | 0.89× (0.85–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.15× (1.12–1.19) | 2.26× (2.20–2.31) | 2.01× (1.96–2.06) | 2.06× (2.01–2.12) | 2.45× (2.37–2.53) | 2.57× (2.51–2.64) | 2.66× (2.59–2.74) | 1.03× (1.01–1.06) | 1.13× (1.10–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.13× (1.11–1.14) | 2.18× (2.15–2.20) | 1.94× (1.92–1.96) | 2.52× (2.50–2.55) | 2.29× (2.27–2.32) | 2.34× (2.32–2.36) | 2.42× (2.40–2.45) | 0.94× (0.93–0.95) | 1.03× (1.02–1.04) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.28–1.40) | 2.04× (1.99–2.09) | 1.72× (1.69–1.74) | 2.33× (2.29–2.37) | 2.28× (2.25–2.31) | 1.78× (1.77–1.79) | 1.84× (1.83–1.86) | 0.71× (0.71–0.72) | 0.78× (0.77–0.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.05× (1.02–1.08) | 2.19× (2.13–2.25) | 1.78× (1.73–1.83) | 1.84× (1.78–1.89) | 2.18× (2.11–2.24) | 2.61× (2.54–2.69) | 2.77× (2.69–2.85) | 1.00× (0.97–1.03) | 1.14× (1.12–1.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.04) | 2.13× (2.12–2.15) | 1.74× (1.72–1.76) | 2.50× (2.46–2.55) | 2.09× (2.08–2.10) | 2.42× (2.41–2.43) | 2.56× (2.55–2.58) | 0.92× (0.91–0.94) | 1.06× (1.04–1.07) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.32× (1.29–1.35) | 2.04× (1.98–2.11) | 1.59× (1.55–1.64) | 2.26× (2.21–2.31) | 2.09× (2.04–2.13) | 1.82× (1.80–1.84) | 1.93× (1.91–1.95) | 0.70× (0.68–0.71) | 0.80× (0.78–0.81) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.56× (2.52–2.60) | 3.48× (3.43–3.53) | 2.68× (2.64–2.71) | 6.34× (6.28–6.41) | 2.14× (2.11–2.17) | 2.03× (2.00–2.05) | 0.97× (0.96–0.99) | 0.96× (0.91–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.63) | 2.41× (2.38–2.45) | 4.22× (4.15–4.30) | 2.65× (2.62–2.68) | 6.16× (6.06–6.26) | 2.03× (2.00–2.06) | 1.93× (1.90–1.95) | 0.93× (0.91–0.95) | 0.91× (0.87–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.61–0.62) | 2.72× (2.67–2.77) | 4.29× (4.23–4.35) | 2.74× (2.70–2.78) | 6.31× (6.23–6.38) | 2.09× (2.06–2.12) | 1.98× (1.95–2.00) | 0.95× (0.93–0.97) | 0.93× (0.90–0.97) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 1 | 1.34× (1.31–1.36) | 2.66× (2.61–2.71) | 2.98× (2.94–3.03) | 2.93× (2.89–2.98) | 3.24× (3.19–3.30) | 3.14× (3.09–3.19) | 3.08× (3.03–3.14) | 1.01× (0.99–1.03) | 1.19× (1.16–1.21) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 2 | 1.24× (1.22–1.27) | 2.40× (2.34–2.46) | 2.77× (2.72–2.82) | 2.82× (2.77–2.86) | 2.84× (2.79–2.90) | 2.68× (2.63–2.72) | 2.63× (2.58–2.67) | 0.86× (0.84–0.88) | 1.01× (0.99–1.03) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 4 | 1.37× (1.28–1.46) | 2.12× (2.04–2.19) | 2.41× (2.35–2.48) | 2.37× (2.31–2.44) | 2.39× (2.33–2.44) | 1.86× (1.83–1.88) | 1.82× (1.79–1.85) | 0.60× (0.58–0.61) | 0.70× (0.69–0.71) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.32× (0.31–0.33) | 1.17× (1.17–1.17) | 0.84× (0.84–0.84) | 11.90× (11.88–11.92) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.58× (0.49–0.68) | 0.86× (0.85–0.87) | 0.66× (0.66–0.67) | 6.33× (6.27–6.40) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.74× (0.65–0.85) | 0.78× (0.78–0.79) | 0.64× (0.63–0.65) | 4.29× (4.27–4.31) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.29–4.29) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.42) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.41) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.10× (0.10–0.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.43× (0.42–0.43) | 1.64× (1.64–1.65) | 0.63× (0.63–0.64) | 7.81× (7.79–7.82) | 0.61× (0.61–0.62) | 0.65× (0.65–0.65) | 0.71× (0.71–0.72) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.62× (0.62–0.62) | 0.62× (0.58–0.67) | 1.54× (1.53–1.55) | 0.87× (0.86–0.87) | 5.57× (5.55–5.59) | 0.44× (0.43–0.44) | 0.46× (0.46–0.46) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.59–0.63) | 0.66× (0.63–0.69) | 1.11× (1.09–1.13) | 0.86× (0.85–0.87) | 3.24× (3.20–3.28) | 0.25× (0.24–0.25) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.38× (0.38–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.71× (0.71–0.71) | 5.07× (5.06–5.08) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.51× (0.50–0.52) | 0.40× (0.37–0.44) | 0.94× (0.94–0.95) | 0.75× (0.72–0.77) | 3.28× (3.25–3.30) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.58–0.60) | 0.42× (0.41–0.42) | 0.80× (0.79–0.81) | 0.75× (0.74–0.76) | 2.35× (2.33–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.81× (0.80–0.81) | 5.11× (5.10–5.12) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.51) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.38× (0.36–0.41) | 0.87× (0.87–0.88) | 0.72× (0.70–0.75) | 3.01× (2.99–3.02) | 0.18× (0.18–0.18) | 0.19× (0.19–0.19) | 0.30× (0.30–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.55–0.57) | 0.47× (0.44–0.49) | 0.75× (0.73–0.76) | 0.76× (0.73–0.78) | 2.16× (2.13–2.20) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.20× (0.20–0.20) | 1.09× (1.08–1.09) | 0.76× (0.76–0.76) | 5.43× (5.42–5.44) | 0.35× (0.34–0.35) | 0.36× (0.36–0.36) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.39× (5.39–5.40) | 0.34× (0.34–0.34) | 0.36× (0.36–0.36) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.20–0.20) | 1.63× (1.63–1.64) | 0.77× (0.77–0.77) | 5.42× (5.40–5.43) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.46× (1.46–1.47) | 0.61× (0.61–0.62) | 6.60× (6.58–6.62) | 0.39× (0.39–0.39) | 0.41× (0.40–0.41) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 2 | 32 MB | 0.50× (0.50–0.50) | 0.44× (0.39–0.50) | 1.31× (1.28–1.34) | 0.73× (0.71–0.76) | 3.82× (3.80–3.85) | 0.23× (0.22–0.23) | 0.24× (0.23–0.24) | 0.37× (0.37–0.37) | 0.18× (0.18–0.18) |
| `text` (1.54 MB → 12.4 MB, 8.0×) | 4 | 52 MB | 0.51× (0.50–0.53) | 0.50× (0.46–0.56) | 1.05× (1.03–1.07) | 0.74× (0.71–0.78) | 2.50× (2.47–2.53) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.23–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 260 | 3.85 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 402 | 4.30 | 1.13× (1.10–1.15) | 77% | 1.04× (1.03–1.05) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 510 | 6.59 | 1.71× (1.68–1.74) | 49% | 1.26× (1.24–1.27) | 1.68× (1.66–1.70) |

**Best:** mibz2 at 4 threads (510 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (295 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.64, mibz2 4t 0.93, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.37. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.16 | 0.02 | 0.08 | 0.411 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3226 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m10s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.22 | 10.6% |
| MTF + RLE2 | 2.23 | 19.3% |
| T-vector build | 2.25 | 19.5% |
| walk (the one each build uses) | 5.30 | 46.0% |
| of which: segment assembly | 0.17 | 1.5% |
| RLE1 | 0.28 | 2.4% |
| CRC | 0.26 | 2.2% |
| **total** | 11.52 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.690 | 1.300 | (in its total) | 1.89× (1.88–1.89) |  |
| walk | 1.555 | 2.645 | (in its total) | 1.69× (1.67–1.71) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.055 | (in its total) | 1.11× (1.09–1.12) |  |
| inverse BWT, total | 2.294 | 4.001 | 3.375 | 1.74× (1.72–1.75) | 1.47× (1.46–1.48) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h57m ± 14m09s |
| 4 threads | 1h00m ± 8m31s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m29s ± 6m01s |
| the scanner alone (its serial part) | 16m44s ± 32.9s |
| the workers alone, over 4 | 42m29s ± 6m01s |

At 4 threads the workers bound it. Per byte: the scanner 9.7 ns a byte of bz2, the workers 5.57 ns a byte of output at 4 threads (CPU, summed over them), 3.30 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,541,677 | 12,396,365 | \`ba393c505b831ede…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3225.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
