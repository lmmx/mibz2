# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-348/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`2cc0829`](<https://github.com/lmmx/mibz2/commit/2cc0829613dccf0684022b85e97c2e61e4733413>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42798623752>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.372 s | 563 | 8.1 | 2.28× (2.22–2.35) | 5.71× (5.57–5.86) | 6.05× (5.82–6.29) | 6.80× (6.58–7.02) | 6.04× (5.79–6.30) | 6.29× (5.98–6.61) | 5.88× (5.65–6.12) | 1.79× (1.74–1.85) | 2.07× (2.00–2.15) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.225 s | 933 | 9.1 | 2.28× (2.10–2.48) | 5.02× (4.64–5.43) | 6.21× (5.79–6.66) | 6.12× (5.66–6.61) | 5.34× (4.92–5.80) | 10.33× (9.43–11.32) | 9.66× (8.90–10.50) | 2.95× (2.71–3.21) | 3.41× (3.12–3.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.198 s | 1,059 | 15.4 | 2.30× (2.19–2.41) | 3.46× (3.35–3.57) | 4.75× (4.51–5.01) | 4.27× (4.12–4.43) | 4.14× (3.93–4.36) | 11.62× (11.15–12.12) | 10.87× (10.42–11.35) | 3.32× (3.16–3.49) | 3.83× (3.67–4.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,394 | 3.6 | 0.57× (0.55–0.58) | 3.74× (3.34–4.18) | 4.45× (4.29–4.62) | 3.71× (3.64–3.78) | 6.70× (6.10–7.36) | 2.79× (2.70–2.89) | 2.94× (2.62–3.29) | 1.11× (0.90–1.37) | 2.21× (2.05–2.37) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,329 | 3.9 | 0.49× (0.45–0.54) | 2.84× (2.52–3.19) | 5.64× (5.17–6.16) | 3.65× (3.18–4.19) | 6.02× (5.56–6.51) | 2.64× (2.42–2.87) | 2.78× (2.40–3.21) | 1.05× (0.85–1.29) | 2.09× (1.89–2.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,342 | 3.7 | 0.51× (0.50–0.52) | 2.90× (2.70–3.11) | 5.77× (5.55–5.99) | 3.63× (3.58–3.68) | 6.30× (6.10–6.50) | 2.70× (2.65–2.74) | 2.84× (2.57–3.13) | 1.07× (0.86–1.33) | 2.13× (1.99–2.29) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 973 | 5.0 | 2.67× (2.54–2.80) | 3.37× (3.15–3.60) | 3.94× (3.74–4.15) | 4.31× (4.04–4.61) | 4.92× (4.56–5.31) | 2.87× (2.65–3.11) | 2.44× (2.33–2.56) | 2.03× (1.91–2.16) | 2.97× (2.82–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,375 | 6.6 | 2.13× (1.86–2.45) | 2.81× (2.38–3.32) | 4.05× (3.56–4.61) | 3.36× (2.97–3.79) | 5.07× (4.55–5.65) | 3.86× (3.35–4.46) | 3.29× (2.81–3.84) | 2.73× (2.42–3.09) | 4.00× (3.52–4.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,156 | 13.1 | 1.64× (1.54–1.76) | 1.80× (1.65–1.96) | 2.77× (2.48–3.10) | 2.12× (1.93–2.33) | 4.10× (3.73–4.51) | 3.37× (3.06–3.70) | 2.87× (2.60–3.16) | 2.38× (2.22–2.56) | 3.49× (3.21–3.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.084 s | 96 | 48.2 | 1.38× (1.26–1.51) | 3.10× (2.77–3.46) | 2.65× (2.44–2.88) | 2.61× (2.42–2.82) | 3.00× (2.78–3.23) | 3.31× (3.05–3.58) | 3.33× (3.04–3.64) | 1.28× (1.19–1.39) | 1.47× (1.34–1.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 150 | 54.7 | 1.33× (1.23–1.45) | 2.78× (2.67–2.91) | 2.59× (2.51–2.68) | 3.30× (3.18–3.43) | 3.09× (2.93–3.26) | 5.38× (5.15–5.62) | 5.42× (5.18–5.66) | 2.09× (2.01–2.17) | 2.39× (2.26–2.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 178 | 75.3 | 1.30× (1.25–1.35) | 2.41× (2.27–2.57) | 2.35× (2.16–2.54) | 2.82× (2.60–3.05) | 3.01× (2.88–3.14) | 6.34× (6.08–6.60) | 6.38× (6.15–6.62) | 2.46× (2.39–2.54) | 2.82× (2.73–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 41.7 | 1.49× (1.43–1.57) | 3.61× (3.45–3.78) | 3.08× (2.87–3.31) | 3.04× (2.93–3.16) | 3.43× (3.28–3.59) | 4.29× (4.13–4.45) | 4.57× (4.39–4.75) | 1.56× (1.51–1.62) | 1.81× (1.74–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 172 | 48.9 | 1.35× (1.29–1.41) | 3.26× (3.12–3.40) | 2.77× (2.67–2.87) | 3.78× (3.61–3.96) | 3.28× (3.18–3.38) | 6.81× (6.54–7.08) | 7.24× (6.99–7.51) | 2.48× (2.40–2.56) | 2.87× (2.75–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.048 s | 200 | 71.3 | 1.30× (1.16–1.46) | 2.47× (2.19–2.77) | 2.41× (2.22–2.61) | 3.07× (2.81–3.35) | 2.86× (2.58–3.18) | 7.33× (6.63–8.10) | 7.80× (7.06–8.62) | 2.67× (2.41–2.96) | 3.09× (2.80–3.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 5.2 | 0.59× (0.55–0.63) | 3.08× (2.86–3.31) | 4.26× (3.95–4.59) | 3.04× (2.83–3.27) | 6.71× (6.26–7.18) | 2.24× (2.08–2.41) | 2.22× (2.08–2.37) | 0.92× (0.88–0.97) | 2.05× (1.91–2.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,005 | 5.3 | 0.55× (0.46–0.65) | 3.09× (3.03–3.17) | 5.35× (5.21–5.48) | 3.05× (2.97–3.13) | 7.04× (6.33–7.82) | 2.26× (2.19–2.32) | 2.24× (2.17–2.30) | 0.93× (0.89–0.97) | 2.06× (2.02–2.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 5.3 | 0.48× (0.42–0.54) | 2.92× (2.51–3.40) | 5.15× (4.32–6.14) | 2.81× (2.52–3.13) | 6.47× (5.59–7.50) | 2.06× (1.82–2.32) | 2.04× (1.81–2.30) | 0.85× (0.74–0.96) | 1.88× (1.66–2.13) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.057 s | 216 | 21.6 | 1.27× (1.25–1.30) | 3.12× (3.02–3.22) | 3.37× (3.32–3.43) | 3.52× (3.44–3.61) | 3.67× (3.57–3.77) | 3.73× (3.63–3.83) | 3.56× (3.47–3.66) | 1.10× (1.08–1.12) | 1.38× (1.24–1.53) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.039 s | 313 | 26.3 | 1.00× (0.88–1.13) | 2.57× (2.25–2.95) | 2.81× (2.48–3.19) | 3.39× (2.98–3.84) | 3.31× (2.87–3.82) | 5.18× (4.62–5.80) | 4.95× (4.37–5.60) | 1.53× (1.35–1.73) | 1.91× (1.61–2.27) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.034 s | 369 | 39.6 | 1.10× (1.05–1.17) | 2.04× (1.91–2.17) | 2.67× (2.56–2.78) | 2.93× (2.83–3.03) | 3.25× (3.11–3.39) | 6.44× (6.27–6.62) | 6.16× (5.98–6.34) | 1.91× (1.84–1.97) | 2.38× (2.14–2.65) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.28× (2.22–2.34) | 5.68× (5.55–5.82) | 6.02× (5.79–6.25) | 6.77× (6.55–6.99) | 6.02× (5.77–6.28) | 6.26× (5.95–6.59) | 5.86× (5.63–6.09) | 1.79× (1.74–1.84) | 2.05× (1.98–2.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.18× (2.10–2.26) | 5.29× (5.16–5.42) | 5.74× (5.57–5.93) | 6.55× (6.37–6.74) | 5.45× (5.30–5.60) | 5.63× (5.41–5.86) | 5.27× (5.13–5.41) | 1.61× (1.56–1.66) | 1.84× (1.79–1.90) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.12× (2.05–2.19) | 3.72× (3.60–3.85) | 4.33× (4.12–4.54) | 4.61× (4.47–4.75) | 3.91× (3.71–4.11) | 3.23× (3.01–3.45) | 3.02× (2.84–3.21) | 0.92× (0.87–0.97) | 1.06× (1.01–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.63) | 3.19× (2.84–3.57) | 3.90× (3.77–4.03) | 3.44× (3.37–3.51) | 6.12× (5.57–6.72) | 2.61× (2.52–2.70) | 2.77× (2.43–3.15) | 1.11× (0.91–1.35) | 1.89× (1.78–2.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.51–0.59) | 2.82× (2.53–3.15) | 4.90× (4.55–5.27) | 3.37× (2.94–3.87) | 5.44× (5.07–5.83) | 2.43× (2.25–2.63) | 2.58× (2.21–3.01) | 1.03× (0.84–1.26) | 1.76× (1.61–1.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.06× (2.98–3.14) | 5.02× (4.89–5.15) | 3.42× (3.38–3.47) | 5.81× (5.63–6.00) | 2.55× (2.51–2.60) | 2.71× (2.41–3.04) | 1.08× (0.88–1.33) | 1.85× (1.75–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.33–2.70) | 2.85× (2.64–3.06) | 3.43× (3.18–3.69) | 4.03× (3.68–4.42) | 4.57× (4.15–5.02) | 2.70× (2.45–2.97) | 2.32× (2.23–2.41) | 1.93× (1.78–2.10) | 2.54× (2.37–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.01× (1.83–2.22) | 2.41× (2.01–2.88) | 2.90× (2.53–3.34) | 3.00× (2.68–3.36) | 3.42× (3.08–3.81) | 1.96× (1.71–2.25) | 1.69× (1.44–1.97) | 1.40× (1.24–1.59) | 1.85× (1.65–2.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.76× (1.61–1.94) | 1.59× (1.45–1.74) | 1.96× (1.78–2.16) | 1.87× (1.70–2.05) | 2.14× (1.92–2.38) | 1.04× (0.94–1.14) | 0.89× (0.79–1.02) | 0.74× (0.67–0.82) | 0.98× (0.91–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.25–1.51) | 2.99× (2.67–3.35) | 2.56× (2.34–2.79) | 2.57× (2.37–2.79) | 2.95× (2.73–3.20) | 3.25× (2.99–3.54) | 3.27× (2.98–3.59) | 1.28× (1.18–1.38) | 1.40× (1.28–1.54) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.22–1.44) | 2.69× (2.61–2.76) | 2.32× (2.27–2.38) | 3.11× (3.01–3.21) | 2.69× (2.58–2.80) | 2.92× (2.82–3.03) | 2.94× (2.83–3.06) | 1.15× (1.11–1.18) | 1.26× (1.20–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.38–1.49) | 2.62× (2.52–2.72) | 2.17× (2.06–2.27) | 2.82× (2.63–3.02) | 2.58× (2.50–2.66) | 2.15× (2.06–2.25) | 2.17× (2.09–2.25) | 0.85× (0.82–0.88) | 0.93× (0.89–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.42–1.56) | 3.52× (3.37–3.68) | 2.99× (2.77–3.22) | 3.01× (2.90–3.12) | 3.39× (3.24–3.55) | 4.23× (4.08–4.39) | 4.50× (4.33–4.68) | 1.55× (1.50–1.61) | 1.74× (1.67–1.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.33–1.39) | 3.24× (3.10–3.38) | 2.53× (2.44–2.62) | 3.53× (3.40–3.66) | 2.92× (2.83–3.00) | 3.64× (3.51–3.78) | 3.88× (3.74–4.01) | 1.34× (1.30–1.38) | 1.50× (1.43–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.45× (1.31–1.62) | 2.70× (2.42–3.01) | 2.06× (1.90–2.24) | 2.89× (2.66–3.14) | 2.56× (2.29–2.85) | 2.38× (2.16–2.62) | 2.53× (2.29–2.79) | 0.87× (0.79–0.97) | 0.98× (0.88–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.63× (0.57–0.69) | 2.42× (2.18–2.68) | 3.18× (2.86–3.53) | 2.67× (2.40–2.97) | 5.73× (5.16–6.36) | 2.01× (1.80–2.24) | 1.99× (1.80–2.21) | 0.91× (0.83–0.99) | 1.39× (1.25–1.55) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.53–0.78) | 2.33× (2.28–2.38) | 4.04× (3.92–4.17) | 2.78× (2.71–2.85) | 6.08× (5.46–6.77) | 2.03× (1.98–2.09) | 2.02× (1.96–2.08) | 0.92× (0.88–0.95) | 1.41× (1.38–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.45–0.61) | 2.43× (1.97–2.98) | 4.01× (3.27–4.92) | 2.59× (2.24–2.99) | 5.67× (4.76–6.75) | 1.89× (1.62–2.19) | 1.87× (1.61–2.17) | 0.85× (0.73–0.99) | 1.30× (1.12–1.52) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.26× (1.24–1.29) | 2.98× (2.91–3.06) | 3.22× (3.16–3.29) | 3.45× (3.37–3.53) | 3.59× (3.49–3.69) | 3.64× (3.55–3.74) | 3.49× (3.40–3.58) | 1.10× (1.08–1.12) | 1.28× (1.15–1.42) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.12× (1.07–1.17) | 2.52× (2.42–2.63) | 2.76× (2.65–2.87) | 3.25× (3.05–3.45) | 3.03× (2.83–3.25) | 2.93× (2.83–3.04) | 2.81× (2.69–2.93) | 0.88× (0.85–0.92) | 1.03× (0.92–1.15) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.24× (1.18–1.31) | 2.21× (2.06–2.36) | 2.38× (2.31–2.46) | 2.64× (2.54–2.74) | 2.57× (2.46–2.68) | 2.00× (1.94–2.06) | 1.91× (1.85–1.98) | 0.60× (0.58–0.62) | 0.70× (0.64–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.37× (0.36–0.38) | 1.07× (1.07–1.07) | 0.85× (0.84–0.87) | 12.07× (12.05–12.10) | 0.29× (0.28–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.56× (0.45–0.69) | 0.78× (0.77–0.80) | 0.66× (0.65–0.67) | 6.29× (6.18–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.66× (0.54–0.80) | 0.70× (0.70–0.71) | 0.62× (0.61–0.63) | 4.10× (4.09–4.12) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.46× (0.44–0.47) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.05× (8.03–8.08) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.67× (0.63–0.71) | 1.59× (1.55–1.63) | 0.79× (0.77–0.81) | 5.77× (5.62–5.91) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.67) | 0.67× (0.62–0.73) | 1.14× (1.09–1.20) | 0.71× (0.68–0.74) | 3.36× (3.22–3.51) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.47–0.52) | 0.40× (0.36–0.45) | 0.91× (0.86–0.95) | 0.73× (0.69–0.76) | 3.13× (2.98–3.30) | 0.20× (0.19–0.21) | 0.21× (0.19–0.22) | 0.31× (0.29–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.64–0.67) | 0.47× (0.46–0.47) | 0.89× (0.87–0.90) | 0.84× (0.82–0.86) | 2.58× (2.53–2.63) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.23× (5.22–5.24) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.51) | 0.39× (0.36–0.41) | 0.86× (0.81–0.91) | 0.72× (0.68–0.76) | 2.94× (2.77–3.12) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.27–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.52) | 0.80× (0.79–0.82) | 0.82× (0.80–0.84) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.25) | 1.08× (1.08–1.08) | 0.77× (0.77–0.77) | 5.41× (5.40–5.42) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.78× (0.77–0.78) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.41) | 1.47× (1.47–1.48) | 0.70× (0.69–0.70) | 6.66× (6.65–6.68) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.40× (0.35–0.45) | 1.04× (1.01–1.08) | 0.67× (0.65–0.69) | 3.29× (3.22–3.36) | 0.20× (0.20–0.21) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.47× (0.44–0.50) | 0.91× (0.89–0.94) | 0.73× (0.71–0.76) | 2.33× (2.27–2.38) | 0.14× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 563 | 1.78 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 933 | 2.01 | 1.11× (1.06–1.16) | 82% | 1.04× (1.02–1.06) | 1.07× (1.04–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 1,059 | 3.41 | 1.94× (1.85–2.03) | 46% | 1.27× (1.21–1.34) | 1.81× (1.70–1.92) |

**Best:** mibz2 at 4 threads (1,059 MB/s); lbzip2 at 4 threads (308 MB/s); pbz2 at 4 threads (456 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.14, lbzip2 1t 0.08, lbzip2 2t 0.36, lbzip2 4t 0.45, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.56. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.56 | 0.01 | 0.08 | 0.198 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4522 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.06 | 13.9% |
| MTF + RLE2 | 1.62 | 21.2% |
| T-vector build | 1.38 | 18.1% |
| walk (the one each build uses) | 3.14 | 41.1% |
| of which: segment assembly | 0.13 | 1.8% |
| RLE1 | 0.17 | 2.2% |
| CRC | 0.26 | 3.4% |
| **total** | 7.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.297 | 0.701 | (in its total) | 2.36× (2.35–2.38) |  |
| walk | 0.650 | 1.567 | (in its total) | 2.40× (2.37–2.43) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.030 | 0.032 | (in its total) | 1.08× (1.03–1.14) |  |
| inverse BWT, total | 0.977 | 2.298 | 1.292 | 2.35× (2.32–2.38) | 1.31× (1.30–1.33) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m38s ± 6m35s |
| 4 threads | 29m35s ± 4m27s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m55s ± 3m18s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 20m55s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 2.74 ns a byte of output at 4 threads (CPU, summed over them), 1.48 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4522.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
