# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-531/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c066f79`](<https://github.com/lmmx/mibz2/commit/c066f7923c2e6769b940e396b325c9431daa464e>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44062556880>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.416 s | 504 | 7.4 | 2.92× (2.90–2.94) | 6.71× (6.68–6.75) | 6.94× (6.92–6.96) | 7.87× (7.86–7.89) | 6.92× (6.90–6.93) | 7.27× (7.24–7.29) | 7.00× (6.97–7.03) | 2.17× (2.15–2.19) | 2.42× (2.41–2.43) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.237 s | 885 | 8.0 | 2.91× (2.74–3.10) | 6.02× (5.75–6.32) | 6.93× (6.58–7.31) | 7.18× (6.83–7.55) | 6.28× (5.96–6.62) | 12.35× (11.73–13.01) | 11.90× (11.29–12.54) | 3.68× (3.51–3.87) | 4.11× (3.90–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.197 s | 1,066 | 12.1 | 2.91× (2.87–2.96) | 4.28× (4.20–4.37) | 5.35× (5.14–5.56) | 5.32× (5.20–5.44) | 5.15× (5.07–5.24) | 15.29× (15.02–15.56) | 14.73× (14.46–15.01) | 4.56× (4.46–4.66) | 5.09× (5.00–5.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 3.6 | 0.57× (0.56–0.58) | 3.43× (3.38–3.47) | 4.58× (4.52–4.63) | 3.57× (3.52–3.62) | 6.07× (5.99–6.15) | 2.66× (2.62–2.69) | 2.68× (2.62–2.74) | 0.98× (0.96–1.01) | 1.77× (1.65–1.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,102 | 3.7 | 0.53× (0.51–0.54) | 2.81× (2.74–2.87) | 5.62× (5.49–5.76) | 3.52× (3.44–3.60) | 5.93× (5.79–6.07) | 2.60× (2.54–2.67) | 2.62× (2.56–2.69) | 0.96× (0.93–0.99) | 1.73× (1.63–1.84) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,108 | 3.6 | 0.52× (0.50–0.55) | 2.78× (2.64–2.93) | 5.81× (5.30–6.37) | 3.48× (3.30–3.67) | 5.89× (5.59–6.21) | 2.57× (2.44–2.70) | 2.59× (2.45–2.74) | 0.95× (0.90–1.00) | 1.71× (1.57–1.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 751 | 5.2 | 2.75× (2.73–2.76) | 3.12× (3.11–3.13) | 4.05× (3.84–4.27) | 4.20× (4.15–4.24) | 4.56× (4.52–4.60) | 2.81× (2.80–2.83) | 2.30× (2.29–2.31) | 2.11× (2.10–2.12) | 3.11× (3.10–3.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,126 | 6.5 | 2.12× (1.84–2.43) | 2.79× (2.41–3.23) | 4.14× (3.51–4.87) | 3.47× (3.02–3.99) | 5.06× (4.31–5.95) | 3.92× (3.41–4.51) | 3.21× (2.79–3.70) | 2.95× (2.56–3.39) | 4.34× (3.77–4.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 952 | 13.0 | 1.83× (1.75–1.91) | 1.74× (1.56–1.95) | 3.04× (2.90–3.18) | 2.15× (2.02–2.27) | 3.94× (3.79–4.11) | 3.54× (3.41–3.67) | 2.89× (2.79–3.00) | 2.66× (2.56–2.75) | 3.91× (3.77–4.05) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 72 | 51.8 | 1.37× (1.37–1.38) | 2.63× (2.59–2.67) | 2.38× (2.37–2.40) | 2.50× (2.49–2.52) | 2.79× (2.76–2.81) | 2.98× (2.97–2.99) | 3.12× (3.10–3.15) | 1.24× (1.23–1.25) | 1.38× (1.36–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 117 | 58.2 | 1.32× (1.18–1.48) | 2.43× (2.31–2.55) | 2.31× (2.20–2.44) | 3.05× (2.92–3.19) | 2.83× (2.71–2.96) | 4.77× (4.60–4.95) | 5.00× (4.81–5.20) | 1.98× (1.91–2.06) | 2.21× (2.12–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 75.9 | 1.33× (1.31–1.35) | 2.16× (2.07–2.25) | 2.11× (2.02–2.21) | 2.64× (2.59–2.69) | 2.90× (2.86–2.94) | 5.92× (5.82–6.02) | 6.21× (6.11–6.31) | 2.46× (2.43–2.50) | 2.75× (2.69–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.1 | 1.53× (1.52–1.54) | 3.06× (3.03–3.09) | 2.67× (2.64–2.71) | 2.79× (2.77–2.81) | 3.11× (3.09–3.13) | 3.70× (3.68–3.72) | 4.06× (4.04–4.09) | 1.49× (1.48–1.50) | 1.80× (1.79–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 132 | 52.6 | 1.39× (1.33–1.46) | 2.80× (2.71–2.89) | 2.53× (2.47–2.59) | 3.54× (3.45–3.63) | 3.08× (3.00–3.16) | 5.99× (5.84–6.15) | 6.58× (6.42–6.75) | 2.42× (2.35–2.49) | 2.91× (2.84–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 74.0 | 1.37× (1.33–1.42) | 2.20× (2.12–2.30) | 2.20× (2.10–2.31) | 3.05× (2.96–3.14) | 2.80× (2.71–2.88) | 6.86× (6.68–7.06) | 7.54× (7.33–7.76) | 2.77× (2.68–2.87) | 3.34× (3.25–3.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 5.0 | 0.64× (0.61–0.66) | 3.50× (3.43–3.56) | 4.39× (4.31–4.47) | 3.13× (3.08–3.18) | 6.68× (6.52–6.84) | 2.30× (2.26–2.35) | 2.35× (2.30–2.40) | 0.96× (0.93–0.98) | 1.77× (1.72–1.81) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 810 | 5.3 | 0.52× (0.50–0.53) | 2.48× (2.43–2.52) | 4.97× (4.87–5.07) | 2.96× (2.90–3.02) | 6.30× (6.17–6.43) | 2.17× (2.12–2.22) | 2.21× (2.16–2.26) | 0.90× (0.88–0.93) | 1.66× (1.62–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.52× (0.52–0.53) | 2.48× (2.44–2.52) | 4.97× (4.89–5.05) | 3.00× (2.91–3.08) | 6.33× (6.19–6.47) | 2.17× (2.13–2.20) | 2.21× (2.17–2.25) | 0.90× (0.87–0.93) | 1.66× (1.62–1.70) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 0.054 s | 229 | 16.6 | 1.82× (1.81–1.83) | 3.96× (3.94–3.98) | 4.35× (4.31–4.39) | 4.58× (4.55–4.60) | 4.79× (4.76–4.82) | 4.74× (4.69–4.78) | 4.83× (4.81–4.85) | 1.50× (1.49–1.51) | 1.71× (1.67–1.75) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 0.037 s | 330 | 21.2 | 1.44× (1.35–1.53) | 3.12× (3.03–3.21) | 3.65× (3.50–3.80) | 4.39× (4.24–4.53) | 4.22× (4.09–4.35) | 6.72× (6.57–6.88) | 6.86× (6.68–7.04) | 2.13× (2.08–2.19) | 2.43× (2.34–2.53) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 0.034 s | 366 | 32.8 | 1.44× (1.39–1.48) | 2.34× (2.20–2.49) | 2.91× (2.80–3.02) | 3.45× (3.36–3.54) | 3.74× (3.63–3.86) | 7.52× (7.34–7.69) | 7.66× (7.45–7.88) | 2.38× (2.32–2.44) | 2.72× (2.63–2.81) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.90–2.93) | 6.68× (6.65–6.71) | 6.90× (6.88–6.92) | 7.84× (7.83–7.86) | 6.89× (6.88–6.91) | 7.24× (7.21–7.27) | 6.98× (6.95–7.00) | 2.16× (2.14–2.18) | 2.40× (2.39–2.41) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.74–2.76) | 6.21× (6.10–6.32) | 6.53× (6.43–6.63) | 7.55× (7.45–7.65) | 6.33× (6.26–6.40) | 6.60× (6.53–6.68) | 6.36× (6.28–6.45) | 1.97× (1.94–2.00) | 2.19× (2.16–2.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.82–2.98) | 4.82× (4.73–4.91) | 5.20× (5.11–5.29) | 5.91× (5.81–6.01) | 5.06× (4.97–5.15) | 4.39× (4.33–4.46) | 4.23× (4.17–4.29) | 1.31× (1.29–1.33) | 1.46× (1.44–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.96× (2.89–3.03) | 3.90× (3.81–3.99) | 3.34× (3.30–3.39) | 5.62× (5.55–5.69) | 2.51× (2.48–2.54) | 2.53× (2.47–2.60) | 0.98× (0.96–1.01) | 1.63× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.59) | 2.71× (2.66–2.77) | 4.91× (4.83–4.99) | 3.28× (3.21–3.35) | 5.43× (5.31–5.56) | 2.43× (2.38–2.48) | 2.45× (2.39–2.51) | 0.95× (0.92–0.98) | 1.58× (1.55–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.59) | 2.98× (2.85–3.11) | 5.15× (4.67–5.67) | 3.31× (3.15–3.47) | 5.50× (5.25–5.76) | 2.45× (2.34–2.56) | 2.48× (2.35–2.61) | 0.96× (0.92–1.01) | 1.60× (1.53–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.63–2.65) | 2.87× (2.86–2.89) | 3.54× (3.33–3.78) | 4.00× (3.95–4.04) | 4.34× (4.30–4.38) | 2.70× (2.68–2.72) | 2.22× (2.21–2.23) | 2.04× (2.03–2.05) | 2.68× (2.66–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.23–2.31) | 2.49× (2.41–2.57) | 3.32× (3.25–3.40) | 3.37× (3.28–3.46) | 3.62× (3.56–3.68) | 2.16× (2.12–2.21) | 1.78× (1.74–1.82) | 1.64× (1.60–1.67) | 2.14× (2.10–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.89–2.14) | 1.71× (1.63–1.79) | 2.25× (2.16–2.35) | 2.11× (2.01–2.22) | 2.12× (2.03–2.21) | 1.10× (1.05–1.15) | 0.91× (0.87–0.95) | 0.83× (0.80–0.87) | 1.09× (1.04–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.37) | 2.56× (2.55–2.58) | 2.33× (2.32–2.34) | 2.48× (2.46–2.49) | 2.76× (2.73–2.78) | 2.95× (2.93–2.96) | 3.09× (3.06–3.11) | 1.23× (1.22–1.25) | 1.36× (1.35–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.25–1.28) | 2.40× (2.37–2.44) | 2.13× (2.10–2.16) | 2.90× (2.86–2.95) | 2.49× (2.45–2.54) | 2.61× (2.57–2.64) | 2.73× (2.70–2.77) | 1.09× (1.08–1.11) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.45–1.51) | 2.32× (2.28–2.36) | 1.97× (1.90–2.05) | 2.63× (2.58–2.67) | 2.48× (2.44–2.53) | 2.00× (1.97–2.03) | 2.10× (2.06–2.13) | 0.84× (0.83–0.85) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.53) | 2.99× (2.97–3.00) | 2.60× (2.58–2.63) | 2.77× (2.75–2.78) | 3.08× (3.06–3.10) | 3.66× (3.64–3.68) | 4.02× (3.99–4.04) | 1.49× (1.47–1.50) | 1.73× (1.72–1.74) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.34–1.44) | 2.72× (2.66–2.79) | 2.32× (2.27–2.38) | 3.30× (3.22–3.38) | 2.73× (2.66–2.80) | 3.16× (3.09–3.24) | 3.47× (3.39–3.55) | 1.28× (1.25–1.32) | 1.49× (1.46–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.66) | 2.52× (2.45–2.60) | 2.01× (1.95–2.08) | 2.90× (2.80–3.00) | 2.60× (2.52–2.68) | 2.31× (2.24–2.38) | 2.54× (2.46–2.61) | 0.94× (0.91–0.97) | 1.09× (1.06–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.71) | 2.46× (2.42–2.51) | 3.41× (3.33–3.50) | 2.85× (2.80–2.90) | 5.94× (5.80–6.09) | 2.13× (2.09–2.17) | 2.17× (2.13–2.22) | 0.97× (0.94–0.99) | 1.37× (1.34–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.24× (2.20–2.27) | 3.95× (3.89–4.01) | 2.72× (2.67–2.77) | 5.56× (5.45–5.67) | 1.98× (1.94–2.02) | 2.02× (1.98–2.06) | 0.90× (0.88–0.92) | 1.28× (1.26–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.50× (2.46–2.54) | 4.05× (3.97–4.13) | 2.84× (2.76–2.92) | 5.75× (5.63–5.87) | 2.04× (2.01–2.07) | 2.08× (2.05–2.11) | 0.92× (0.90–0.95) | 1.31× (1.30–1.33) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 1.79× (1.79–1.80) | 3.77× (3.75–3.79) | 4.17× (4.13–4.21) | 4.46× (4.44–4.48) | 4.67× (4.65–4.69) | 4.62× (4.57–4.67) | 4.71× (4.69–4.72) | 1.49× (1.47–1.50) | 1.63× (1.62–1.65) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 1.52× (1.46–1.58) | 3.11× (3.07–3.15) | 3.43× (3.40–3.47) | 4.05× (4.03–4.08) | 3.75× (3.71–3.80) | 3.63× (3.60–3.67) | 3.70× (3.67–3.73) | 1.17× (1.16–1.18) | 1.28× (1.27–1.30) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 1.63× (1.59–1.68) | 2.51× (2.44–2.58) | 2.79× (2.73–2.86) | 3.14× (3.08–3.20) | 3.06× (3.00–3.13) | 2.36× (2.32–2.40) | 2.40× (2.37–2.43) | 0.76× (0.75–0.77) | 0.83× (0.82–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.34× (0.33–0.35) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.42) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.40× (0.34–0.47) | 0.68× (0.66–0.70) | 0.58× (0.56–0.60) | 5.48× (5.35–5.61) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.33–0.35) | 0.53× (0.43–0.64) | 0.60× (0.58–0.62) | 0.54× (0.52–0.56) | 3.54× (3.42–3.67) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.28× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.02× (8.00–8.05) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.62× (0.58–0.66) | 1.61× (1.58–1.64) | 0.81× (0.80–0.83) | 5.85× (5.75–5.96) | 0.48× (0.47–0.48) | 0.50× (0.49–0.51) | 0.53× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.67× (0.61–0.74) | 1.13× (1.08–1.18) | 0.70× (0.67–0.74) | 3.34× (3.19–3.49) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.18× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.43× (0.41–0.45) | 0.94× (0.91–0.97) | 0.74× (0.71–0.78) | 3.26× (3.15–3.37) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.66) | 0.45× (0.44–0.47) | 0.86× (0.84–0.88) | 0.81× (0.79–0.84) | 2.51× (2.44–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.39× (0.36–0.41) | 0.86× (0.83–0.89) | 0.73× (0.69–0.76) | 2.96× (2.85–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.52) | 0.81× (0.80–0.82) | 0.81× (0.78–0.84) | 2.34× (2.32–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.78) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.66–0.67) | 6.38× (6.36–6.39) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.36× (0.33–0.39) | 0.98× (0.96–1.01) | 0.64× (0.62–0.66) | 3.10× (3.02–3.18) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 56 MB | 0.49× (0.49–0.50) | 0.45× (0.41–0.49) | 0.90× (0.88–0.92) | 0.74× (0.72–0.75) | 2.29× (2.25–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 504 | 1.99 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 885 | 2.18 | 1.10× (1.08–1.11) | 85% | 1.02× (1.01–1.03) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,066 | 3.29 | 1.65× (1.62–1.68) | 53% | 1.19× (1.18–1.20) | 1.64× (1.61–1.67) |

**Best:** mibz2 at 4 threads (1,066 MB/s); lbzip2 at 4 threads (248 MB/s); pbz2 at 4 threads (363 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.25, lbzip2 1t 0.06, lbzip2 2t 0.29, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.52 | 0.02 | 0.07 | 0.197 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3693 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.9% |
| MTF + RLE2 | 1.55 | 17.9% |
| T-vector build | 1.49 | 17.2% |
| walk (the one each build uses) | 4.07 | 47.0% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.065 | (in its total) | 2.70× (2.66–2.74) |  |
| walk | 1.050 | 2.064 | (in its total) | 1.96× (1.93–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.042 | 0.041 | (in its total) | 0.97× (0.95–0.99) |  |
| inverse BWT, total | 1.485 | 3.171 | 1.912 | 2.13× (2.10–2.15) | 1.28× (1.27–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m50s |
| 4 threads | 28m48s ± 4m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m46s ± 2m41s |
| the scanner alone (its serial part) | 11m47s ± 19.7s |
| the workers alone, over 4 | 18m46s ± 2m41s |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 2.46 ns a byte of output at 4 threads (CPU, summed over them), 1.60 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 6 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,434,784 | 12,361,720 | \`c83a65c1574133bd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3692.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
