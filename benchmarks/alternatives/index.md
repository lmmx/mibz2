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

The latest record: [`89a1213`](<https://github.com/lmmx/mibz2/commit/89a1213919a01bfdd44c2110084e8a60b14a90a7>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46072101902>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.385 s | 545 | 6.8 | 3.17× (3.16–3.19) | 7.30× (7.27–7.32) | 7.55× (7.52–7.58) | 8.58× (8.54–8.62) | 7.54× (7.52–7.56) | 7.88× (7.86–7.90) | 7.64× (7.60–7.67) | 2.35× (2.35–2.36) | 2.63× (2.62–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.217 s | 966 | 7.3 | 3.29× (3.17–3.40) | 6.63× (6.34–6.92) | 7.63× (7.30–7.97) | 7.97× (7.65–8.31) | 7.00× (6.75–7.25) | 13.67× (13.21–14.16) | 13.25× (12.78–13.73) | 4.08× (3.94–4.23) | 4.57× (4.40–4.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,162 | 11.5 | 3.31× (3.24–3.38) | 4.83× (4.72–4.93) | 5.87× (5.81–5.94) | 5.81× (5.73–5.89) | 5.74× (5.67–5.80) | 16.83× (16.58–17.08) | 16.30× (16.10–16.50) | 5.03× (4.96–5.10) | 5.62× (5.54–5.70) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,090 | 3.7 | 0.56× (0.55–0.57) | 3.34× (3.28–3.40) | 4.46× (4.38–4.54) | 3.50× (3.44–3.56) | 6.01× (5.89–6.13) | 2.59× (2.54–2.64) | 2.59× (2.55–2.64) | 0.96× (0.93–0.98) | 1.77× (1.63–1.93) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,039 | 4.0 | 0.50× (0.49–0.51) | 2.67× (2.62–2.72) | 5.35× (5.25–5.45) | 3.34× (3.28–3.40) | 5.63× (5.52–5.74) | 2.48× (2.44–2.52) | 2.48× (2.43–2.53) | 0.91× (0.89–0.94) | 1.69× (1.55–1.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,020 | 4.0 | 0.50× (0.49–0.51) | 2.63× (2.58–2.68) | 5.26× (5.16–5.36) | 3.29× (3.23–3.35) | 5.63× (5.50–5.75) | 2.43× (2.39–2.48) | 2.44× (2.39–2.48) | 0.90× (0.86–0.93) | 1.66× (1.51–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 772 | 5.1 | 2.83× (2.81–2.84) | 3.21× (3.19–3.22) | 4.02× (4.00–4.04) | 4.28× (4.26–4.29) | 4.73× (4.70–4.77) | 2.90× (2.88–2.92) | 2.37× (2.36–2.38) | 2.18× (2.17–2.19) | 3.20× (3.19–3.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,202 | 6.2 | 2.46× (2.41–2.52) | 3.28× (3.13–3.44) | 4.71× (4.38–5.08) | 4.00× (3.86–4.15) | 5.63× (5.49–5.76) | 4.55× (4.43–4.66) | 3.72× (3.63–3.81) | 3.42× (3.34–3.50) | 5.02× (4.90–5.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,043 | 12.2 | 2.01× (1.89–2.13) | 2.01× (1.76–2.29) | 3.20× (3.06–3.34) | 2.36× (2.23–2.50) | 4.36× (4.12–4.62) | 3.87× (3.65–4.11) | 3.17× (2.99–3.36) | 2.91× (2.76–3.08) | 4.28× (4.04–4.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 78 | 48.0 | 1.47× (1.42–1.52) | 2.84× (2.74–2.94) | 2.57× (2.47–2.67) | 2.66× (2.56–2.75) | 2.99× (2.89–3.10) | 3.18× (3.07–3.30) | 3.36× (3.25–3.48) | 1.33× (1.28–1.38) | 1.48× (1.42–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 53.4 | 1.44× (1.33–1.57) | 2.74× (2.60–2.88) | 2.57× (2.44–2.71) | 3.40× (3.20–3.60) | 3.14× (2.97–3.32) | 5.31× (5.02–5.61) | 5.61× (5.31–5.92) | 2.22× (2.10–2.34) | 2.47× (2.34–2.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 160 | 67.5 | 1.48× (1.44–1.52) | 2.36× (2.31–2.40) | 2.40× (2.28–2.52) | 2.94× (2.84–3.04) | 3.22× (3.15–3.30) | 6.59× (6.38–6.79) | 6.96× (6.75–7.17) | 2.75× (2.67–2.83) | 3.07× (2.97–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.110 s | 87 | 43.0 | 1.64× (1.62–1.65) | 3.32× (3.28–3.36) | 2.87× (2.83–2.91) | 3.00× (2.97–3.03) | 3.37× (3.34–3.39) | 3.98× (3.95–4.01) | 4.41× (4.38–4.45) | 1.62× (1.61–1.63) | 1.92× (1.91–1.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.068 s | 141 | 48.8 | 1.47× (1.43–1.51) | 3.08× (2.98–3.18) | 2.74× (2.66–2.82) | 3.79× (3.71–3.87) | 3.35× (3.26–3.45) | 6.51× (6.37–6.65) | 7.21× (7.06–7.37) | 2.65× (2.58–2.72) | 3.14× (3.07–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.054 s | 179 | 66.4 | 1.62× (1.56–1.68) | 2.57× (2.52–2.61) | 2.61× (2.52–2.71) | 3.52× (3.40–3.65) | 3.32× (3.22–3.43) | 8.11× (7.90–8.33) | 8.99× (8.72–9.26) | 3.30× (3.21–3.40) | 3.92× (3.81–4.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.63) | 3.43× (3.39–3.48) | 4.32× (4.26–4.38) | 3.07× (3.03–3.11) | 6.77× (6.62–6.93) | 2.26× (2.24–2.29) | 2.32× (2.28–2.35) | 0.94× (0.93–0.95) | 1.73× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 5.7 | 0.49× (0.48–0.50) | 2.37× (2.33–2.42) | 4.77× (4.68–4.86) | 2.84× (2.79–2.90) | 6.16× (6.03–6.28) | 2.08× (2.04–2.12) | 2.12× (2.08–2.17) | 0.86× (0.84–0.88) | 1.58× (1.55–1.61) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 752 | 5.7 | 0.49× (0.48–0.50) | 2.34× (2.31–2.38) | 4.69× (4.62–4.77) | 2.82× (2.78–2.86) | 6.11× (5.97–6.24) | 2.04× (2.01–2.07) | 2.09× (2.06–2.12) | 0.85× (0.84–0.86) | 1.56× (1.53–1.58) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.052 s | 286 | 13.3 | 2.02× (2.00–2.04) | 4.11× (4.07–4.15) | 4.61× (4.53–4.69) | 4.85× (4.81–4.90) | 5.05× (4.99–5.12) | 4.77× (4.72–4.81) | 4.92× (4.86–4.98) | 1.66× (1.64–1.67) | 1.95× (1.93–1.97) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.037 s | 399 | 17.2 | 1.52× (1.45–1.58) | 3.24× (3.09–3.41) | 3.97× (3.81–4.14) | 4.57× (4.27–4.90) | 4.37× (4.16–4.60) | 6.61× (6.31–6.93) | 6.83× (6.51–7.16) | 2.30× (2.19–2.41) | 2.71× (2.58–2.84) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.032 s | 464 | 26.5 | 1.58× (1.52–1.65) | 2.60× (2.49–2.72) | 3.42× (3.23–3.62) | 3.67× (3.53–3.82) | 4.12× (3.94–4.31) | 7.64× (7.37–7.91) | 7.88× (7.60–8.18) | 2.65× (2.56–2.75) | 3.13× (3.02–3.24) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.16× (3.15–3.18) | 7.26× (7.23–7.29) | 7.51× (7.47–7.54) | 8.55× (8.51–8.59) | 7.51× (7.49–7.53) | 7.85× (7.83–7.87) | 7.61× (7.57–7.64) | 2.35× (2.34–2.36) | 2.61× (2.60–2.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.04× (3.02–3.06) | 6.87× (6.82–6.92) | 7.16× (7.13–7.20) | 8.34× (8.30–8.39) | 7.02× (6.98–7.06) | 7.28× (7.23–7.32) | 7.05× (7.01–7.09) | 2.18× (2.17–2.19) | 2.42× (2.41–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.18× (3.07–3.29) | 5.20× (5.13–5.27) | 5.59× (5.48–5.71) | 6.29× (6.17–6.41) | 5.45× (5.35–5.57) | 4.66× (4.58–4.75) | 4.52× (4.43–4.61) | 1.40× (1.37–1.42) | 1.55× (1.52–1.58) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.86× (2.81–2.90) | 3.81× (3.76–3.86) | 3.28× (3.23–3.33) | 5.58× (5.48–5.67) | 2.45× (2.41–2.49) | 2.46× (2.42–2.50) | 0.96× (0.94–0.98) | 1.60× (1.57–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.54) | 2.53× (2.49–2.58) | 4.54× (4.46–4.62) | 3.05× (3.00–3.10) | 5.06× (4.97–5.16) | 2.27× (2.23–2.30) | 2.27× (2.23–2.31) | 0.89× (0.87–0.91) | 1.48× (1.45–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.52–0.53) | 2.77× (2.71–2.82) | 4.58× (4.49–4.68) | 3.07× (3.02–3.12) | 5.14× (5.04–5.25) | 2.27× (2.24–2.31) | 2.28× (2.24–2.32) | 0.89× (0.86–0.92) | 1.48× (1.45–1.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.70–2.73) | 2.96× (2.94–2.97) | 3.51× (3.49–3.52) | 4.07× (4.05–4.09) | 4.50× (4.46–4.54) | 2.78× (2.77–2.80) | 2.29× (2.27–2.30) | 2.11× (2.09–2.12) | 2.76× (2.75–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.40–2.48) | 2.67× (2.62–2.74) | 3.64× (3.45–3.84) | 3.58× (3.52–3.65) | 3.88× (3.82–3.94) | 2.32× (2.28–2.36) | 1.90× (1.87–1.94) | 1.75× (1.72–1.78) | 2.30× (2.27–2.34) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.18× (2.10–2.28) | 1.85× (1.77–1.93) | 2.39× (2.31–2.46) | 2.20× (2.15–2.26) | 2.28× (2.21–2.35) | 1.17× (1.13–1.21) | 0.96× (0.93–0.99) | 0.89× (0.86–0.91) | 1.16× (1.12–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.46× (1.41–1.51) | 2.74× (2.65–2.83) | 2.51× (2.41–2.61) | 2.63× (2.54–2.72) | 2.96× (2.86–3.07) | 3.15× (3.04–3.26) | 3.32× (3.22–3.44) | 1.32× (1.28–1.37) | 1.45× (1.40–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.36–1.42) | 2.64× (2.60–2.67) | 2.33× (2.29–2.38) | 3.16× (3.09–3.24) | 2.74× (2.68–2.80) | 2.86× (2.80–2.92) | 3.02× (2.96–3.08) | 1.20× (1.18–1.23) | 1.32× (1.29–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.67× (1.64–1.71) | 2.57× (2.52–2.62) | 2.17× (2.12–2.22) | 2.98× (2.92–3.03) | 2.81× (2.75–2.88) | 2.25× (2.21–2.29) | 2.38× (2.34–2.43) | 0.95× (0.93–0.96) | 1.04× (1.02–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.61–1.64) | 3.22× (3.18–3.27) | 2.81× (2.77–2.85) | 2.97× (2.94–3.00) | 3.33× (3.31–3.35) | 3.94× (3.91–3.97) | 4.36× (4.33–4.40) | 1.61× (1.60–1.62) | 1.86× (1.85–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.50× (1.46–1.53) | 2.98× (2.91–3.05) | 2.55× (2.49–2.60) | 3.58× (3.50–3.66) | 3.03× (2.94–3.12) | 3.47× (3.40–3.55) | 3.85× (3.77–3.93) | 1.42× (1.39–1.45) | 1.64× (1.60–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.76× (1.69–1.82) | 2.79× (2.74–2.85) | 2.21× (2.17–2.26) | 3.21× (3.14–3.28) | 2.93× (2.89–2.97) | 2.57× (2.53–2.61) | 2.85× (2.80–2.90) | 1.05× (1.04–1.07) | 1.21× (1.19–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.44× (2.41–2.48) | 3.36× (3.32–3.40) | 2.81× (2.78–2.84) | 6.05× (5.92–6.18) | 2.10× (2.08–2.13) | 2.15× (2.13–2.17) | 0.95× (0.93–0.96) | 1.35× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.11× (2.07–2.14) | 3.75× (3.68–3.83) | 2.57× (2.53–2.60) | 5.32× (5.22–5.42) | 1.86× (1.83–1.89) | 1.90× (1.87–1.93) | 0.84× (0.82–0.85) | 1.19× (1.17–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.31× (2.28–2.34) | 3.74× (3.66–3.81) | 2.61× (2.57–2.64) | 5.41× (5.30–5.52) | 1.88× (1.85–1.90) | 1.92× (1.89–1.95) | 0.84× (0.83–0.86) | 1.21× (1.19–1.23) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 1.98× (1.96–2.00) | 3.94× (3.90–3.97) | 4.41× (4.35–4.47) | 4.73× (4.69–4.77) | 4.93× (4.87–4.99) | 4.65× (4.61–4.69) | 4.79× (4.74–4.85) | 1.64× (1.62–1.65) | 1.81× (1.79–1.82) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.63× (1.60–1.65) | 3.19× (3.14–3.23) | 3.57× (3.53–3.62) | 4.20× (4.15–4.26) | 3.91× (3.86–3.97) | 3.60× (3.55–3.65) | 3.71× (3.66–3.77) | 1.27× (1.25–1.28) | 1.40× (1.38–1.42) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.74× (1.68–1.80) | 2.60× (2.54–2.66) | 2.94× (2.88–3.00) | 3.29× (3.23–3.36) | 3.13× (3.10–3.16) | 2.35× (2.32–2.39) | 2.43× (2.39–2.47) | 0.83× (0.82–0.84) | 0.91× (0.90–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.06× (11.05–11.07) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.36× (0.35–0.37) | 0.47× (0.41–0.53) | 0.73× (0.72–0.75) | 0.61× (0.59–0.63) | 5.85× (5.70–6.01) | 0.14× (0.14–0.14) | 0.15× (0.14–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.34–0.37) | 0.48× (0.43–0.54) | 0.62× (0.60–0.65) | 0.55× (0.53–0.58) | 3.67× (3.52–3.81) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.85× (4.85–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.69–1.71) | 0.66× (0.65–0.67) | 8.03× (7.99–8.07) | 0.66× (0.65–0.66) | 0.69× (0.68–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.68–0.72) | 0.69× (0.65–0.73) | 1.68× (1.64–1.71) | 0.84× (0.82–0.86) | 6.08× (5.95–6.22) | 0.50× (0.48–0.51) | 0.52× (0.51–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.67× (0.63–0.71) | 0.71× (0.65–0.79) | 1.20× (1.13–1.27) | 0.75× (0.70–0.79) | 3.53× (3.34–3.73) | 0.28× (0.26–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.18× (5.16–5.19) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.39× (0.36–0.42) | 0.87× (0.86–0.89) | 0.71× (0.69–0.72) | 3.03× (2.96–3.09) | 0.19× (0.19–0.19) | 0.20× (0.19–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.85–0.88) | 0.82× (0.80–0.83) | 2.51× (2.48–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.84× (0.81–0.88) | 0.72× (0.68–0.75) | 2.90× (2.79–3.02) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.46× (0.43–0.50) | 0.80× (0.79–0.81) | 0.81× (0.78–0.85) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.17–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.39) | 0.66× (0.65–0.66) | 6.40× (6.39–6.41) | 0.39× (0.38–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.30–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 35 MB | 0.49× (0.47–0.51) | 0.43× (0.39–0.47) | 1.11× (1.07–1.15) | 0.74× (0.70–0.79) | 3.63× (3.49–3.77) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.34–0.37) | 0.17× (0.17–0.18) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.46× (0.42–0.50) | 0.92× (0.90–0.95) | 0.76× (0.73–0.79) | 2.40× (2.33–2.48) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 545 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 966 | 1.98 | 1.08× (1.07–1.09) | 87% | 1.02× (1.01–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,162 | 3.11 | 1.68× (1.66–1.71) | 53% | 1.21× (1.20–1.21) | 1.69× (1.65–1.73) |

**Best:** mibz2 at 4 threads (1,162 MB/s); lbzip2 at 4 threads (241 MB/s); pbz2 at 4 threads (352 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.02, lbzip2 1t 0.07, lbzip2 2t 0.21, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.44, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.50 | 0.01 | 0.09 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 9.8% |
| MTF + RLE2 | 1.58 | 18.5% |
| T-vector build | 1.50 | 17.6% |
| walk (the one each build uses) | 4.16 | 48.8% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.52 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.069 | (in its total) | 2.73× (2.71–2.75) |  |
| walk | 1.065 | 2.081 | (in its total) | 1.95× (1.94–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.041 | (in its total) | 1.03× (0.99–1.07) |  |
| inverse BWT, total | 1.497 | 3.191 | 1.900 | 2.13× (2.11–2.15) | 1.27× (1.26–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m00s ± 6m42s |
| 4 threads | 26m14s ± 3m42s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m05s ± 2m35s |
| the scanner alone (its serial part) | 8m09s ± 32.2s |
| the workers alone, over 4 | 18m05s ± 2m35s |

At 4 threads the workers bound it. Per byte: the scanner 4.7 ns a byte of bz2, the workers 2.37 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,220,170 | 14,849,844 | \`3f9b2018b96fd2e0…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
