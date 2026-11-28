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

The latest record: [`3b146d7`](<https://github.com/lmmx/mibz2/commit/3b146d72e6732ffb665234f702b61442b0d7c8e7>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46100418230>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.483 s | 434 | 7.5 | 3.19× (3.15–3.23) | 7.21× (7.14–7.28) | 7.48× (7.36–7.60) | 8.15× (8.09–8.20) | 7.21× (7.18–7.25) | 7.86× (7.80–7.92) | 7.36× (7.34–7.39) | 2.35× (2.33–2.36) | 2.71× (2.69–2.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.264 s | 793 | 8.0 | 3.17× (3.09–3.25) | 6.83× (6.68–6.99) | 7.97× (7.74–8.20) | 7.95× (7.60–8.32) | 7.00× (6.84–7.17) | 14.14× (13.80–14.49) | 13.24× (12.97–13.52) | 4.22× (4.13–4.31) | 4.88× (4.79–4.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.219 s | 957 | 12.6 | 3.32× (3.19–3.46) | 5.09× (4.89–5.30) | 6.30× (5.99–6.63) | 5.80× (5.49–6.13) | 5.67× (5.58–5.76) | 17.19× (16.91–17.48) | 16.10× (15.89–16.31) | 5.13× (5.05–5.21) | 5.94× (5.86–6.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 3.7 | 0.60× (0.59–0.61) | 3.60× (3.45–3.76) | 4.46× (4.35–4.57) | 3.55× (3.49–3.61) | 6.17× (6.07–6.26) | 2.94× (2.89–2.99) | 2.64× (2.59–2.68) | 1.01× (0.99–1.03) | 1.49× (1.47–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 915 | 4.0 | 0.55× (0.54–0.55) | 2.84× (2.80–2.88) | 5.83× (5.65–6.01) | 3.40× (3.36–3.44) | 5.83× (5.76–5.91) | 2.81× (2.76–2.85) | 2.52× (2.48–2.55) | 0.96× (0.94–0.98) | 1.42× (1.40–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 909 | 4.0 | 0.54× (0.53–0.55) | 2.81× (2.75–2.86) | 5.74× (5.56–5.91) | 3.33× (3.26–3.41) | 5.79× (5.68–5.90) | 2.76× (2.70–2.82) | 2.47× (2.42–2.53) | 0.95× (0.92–0.97) | 1.40× (1.37–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 594 | 5.8 | 2.76× (2.68–2.85) | 3.08× (3.03–3.13) | 3.94× (3.88–3.99) | 4.01× (3.95–4.06) | 4.26× (4.20–4.32) | 2.59× (2.55–2.63) | 2.15× (2.12–2.19) | 2.06× (2.03–2.09) | 3.06× (3.01–3.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 867 | 7.0 | 2.12× (1.83–2.46) | 2.52× (2.17–2.94) | 3.69× (3.15–4.32) | 3.19× (2.73–3.74) | 4.44× (3.81–5.17) | 3.51× (3.01–4.09) | 2.92× (2.51–3.40) | 2.80× (2.40–3.26) | 4.15× (3.56–4.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 843 | 13.8 | 2.08× (2.00–2.17) | 2.00× (1.84–2.18) | 3.20× (3.05–3.35) | 2.41× (2.29–2.54) | 4.31× (4.19–4.45) | 3.71× (3.59–3.84) | 3.09× (2.98–3.20) | 2.96× (2.86–3.07) | 4.39× (4.24–4.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.121 s | 66 | 50.0 | 1.47× (1.46–1.49) | 2.93× (2.90–2.95) | 2.66× (2.60–2.71) | 2.71× (2.63–2.80) | 3.12× (3.05–3.19) | 3.42× (3.37–3.48) | 3.49× (3.43–3.56) | 1.35× (1.32–1.38) | 1.54× (1.51–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 109 | 55.7 | 1.40× (1.35–1.45) | 2.74× (2.66–2.83) | 2.60× (2.53–2.67) | 3.32× (3.21–3.44) | 3.21× (3.12–3.31) | 5.57× (5.41–5.74) | 5.68× (5.48–5.89) | 2.20× (2.14–2.27) | 2.51× (2.44–2.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.061 s | 132 | 75.9 | 1.51× (1.45–1.58) | 2.45× (2.34–2.55) | 2.41× (2.32–2.50) | 2.94× (2.84–3.05) | 3.26× (3.18–3.33) | 6.89× (6.74–7.06) | 7.03× (6.91–7.15) | 2.73× (2.66–2.79) | 3.11× (3.04–3.18) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.136 s | 70 | 46.7 | 1.57× (1.56–1.59) | 3.26× (3.22–3.31) | 2.76× (2.71–2.81) | 2.84× (2.81–2.88) | 3.30× (3.26–3.35) | 3.89× (3.85–3.93) | 4.23× (4.18–4.28) | 1.52× (1.50–1.55) | 1.86× (1.83–1.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 117 | 52.7 | 1.44× (1.37–1.50) | 2.89× (2.78–3.01) | 2.62× (2.52–2.72) | 3.63× (3.48–3.79) | 3.30× (3.12–3.48) | 6.35× (6.12–6.59) | 6.90× (6.66–7.16) | 2.49× (2.39–2.58) | 3.03× (2.89–3.18) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 139 | 74.5 | 1.55× (1.52–1.58) | 2.58× (2.52–2.64) | 2.55× (2.52–2.58) | 3.31× (3.25–3.37) | 3.15× (3.10–3.20) | 7.68× (7.56–7.80) | 8.35× (8.21–8.49) | 3.01× (2.96–3.06) | 3.66× (3.58–3.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 733 | 5.1 | 0.66× (0.65–0.68) | 3.20× (2.98–3.44) | 4.37× (4.28–4.45) | 3.11× (3.06–3.15) | 6.83× (6.67–6.98) | 2.51× (2.46–2.56) | 2.31× (2.27–2.36) | 1.00× (0.97–1.02) | 1.51× (1.48–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 660 | 5.8 | 0.53× (0.52–0.54) | 2.74× (2.69–2.78) | 4.69× (4.61–4.77) | 2.85× (2.81–2.90) | 6.21× (6.12–6.30) | 2.29× (2.25–2.33) | 2.11× (2.07–2.15) | 0.91× (0.88–0.93) | 1.37× (1.35–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 649 | 5.8 | 0.53× (0.53–0.54) | 2.69× (2.67–2.72) | 4.61× (4.56–4.65) | 2.80× (2.77–2.83) | 6.09× (6.03–6.14) | 2.25× (2.22–2.27) | 2.07× (2.05–2.09) | 0.89× (0.87–0.91) | 1.35× (1.34–1.36) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 0.065 s | 231 | 14.5 | 2.21× (2.17–2.25) | 4.34× (4.20–4.48) | 4.74× (4.66–4.83) | 4.73× (4.69–4.77) | 4.94× (4.86–5.01) | 5.32× (5.23–5.41) | 5.03× (4.97–5.10) | 1.66× (1.64–1.68) | 2.05× (2.02–2.09) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 0.046 s | 324 | 18.9 | 1.76× (1.56–1.98) | 3.41× (3.14–3.71) | 3.94× (3.72–4.18) | 4.36× (4.05–4.68) | 4.20× (4.00–4.42) | 7.33× (6.94–7.75) | 6.94× (6.61–7.28) | 2.29× (2.17–2.42) | 2.83× (2.66–3.01) | yes |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 0.040 s | 377 | 29.1 | 1.72× (1.63–1.82) | 2.63× (2.54–2.71) | 3.63× (3.57–3.69) | 3.72× (3.65–3.79) | 4.04× (3.96–4.12) | 8.62× (8.40–8.85) | 8.16× (7.98–8.34) | 2.69× (2.63–2.76) | 3.32× (3.23–3.43) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.18× (3.14–3.22) | 7.17× (7.11–7.24) | 7.44× (7.32–7.56) | 8.12× (8.06–8.17) | 7.19× (7.16–7.22) | 7.83× (7.77–7.89) | 7.33× (7.31–7.36) | 2.34× (2.33–2.35) | 2.70× (2.67–2.72) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.06–3.11) | 6.89× (6.82–6.95) | 7.32× (7.16–7.48) | 8.15× (7.88–8.42) | 6.84× (6.78–6.90) | 7.33× (7.26–7.40) | 6.87× (6.84–6.89) | 2.19× (2.17–2.21) | 2.52× (2.50–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.11–3.27) | 5.33× (5.15–5.51) | 5.67× (5.54–5.80) | 6.00× (5.89–6.10) | 5.28× (5.20–5.36) | 4.66× (4.60–4.73) | 4.37× (4.31–4.43) | 1.39× (1.37–1.41) | 1.60× (1.58–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.22× (3.16–3.27) | 4.04× (3.98–4.09) | 3.32× (3.27–3.37) | 5.70× (5.62–5.78) | 2.77× (2.72–2.81) | 2.49× (2.45–2.52) | 1.01× (0.99–1.03) | 1.08× (1.03–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.87× (2.83–2.91) | 5.06× (4.98–5.14) | 3.10× (3.05–3.14) | 5.22× (5.15–5.29) | 2.55× (2.51–2.59) | 2.29× (2.26–2.33) | 0.93× (0.91–0.95) | 1.00× (0.95–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.14× (3.07–3.21) | 5.02× (4.91–5.13) | 3.08× (3.01–3.15) | 5.23× (5.14–5.33) | 2.54× (2.49–2.59) | 2.28× (2.24–2.33) | 0.92× (0.90–0.95) | 0.99× (0.95–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.60–2.73) | 2.85× (2.81–2.88) | 3.58× (3.53–3.63) | 3.84× (3.79–3.89) | 4.08× (4.03–4.13) | 2.50× (2.47–2.53) | 2.09× (2.07–2.12) | 2.01× (1.98–2.03) | 2.76× (2.72–2.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.34–2.40) | 2.58× (2.54–2.62) | 3.17× (3.13–3.21) | 3.35× (3.31–3.39) | 3.52× (3.47–3.56) | 2.06× (2.04–2.09) | 1.73× (1.71–1.75) | 1.66× (1.64–1.68) | 2.28× (2.25–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.15× (2.04–2.26) | 1.79× (1.72–1.86) | 2.31× (2.23–2.39) | 2.15× (2.07–2.22) | 2.20× (2.12–2.27) | 1.06× (1.03–1.10) | 0.89× (0.86–0.92) | 0.85× (0.83–0.88) | 1.18× (1.14–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.46× (1.45–1.48) | 2.84× (2.82–2.86) | 2.59× (2.54–2.63) | 2.68× (2.60–2.77) | 3.09× (3.01–3.16) | 3.38× (3.33–3.44) | 3.45× (3.39–3.51) | 1.35× (1.32–1.38) | 1.49× (1.47–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.40× (1.39–1.42) | 2.71× (2.69–2.74) | 2.38× (2.36–2.42) | 3.18× (3.15–3.22) | 2.83× (2.79–2.86) | 3.05× (3.01–3.09) | 3.11× (3.05–3.17) | 1.21× (1.19–1.24) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.57–1.69) | 2.58× (2.47–2.70) | 2.10× (1.99–2.23) | 2.85× (2.80–2.90) | 2.69× (2.65–2.72) | 2.24× (2.21–2.27) | 2.28× (2.24–2.33) | 0.89× (0.87–0.91) | 0.99× (0.97–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.57× (1.55–1.58) | 3.20× (3.17–3.23) | 2.70× (2.66–2.75) | 2.82× (2.79–2.85) | 3.27× (3.23–3.31) | 3.85× (3.81–3.89) | 4.18× (4.13–4.23) | 1.52× (1.49–1.54) | 1.81× (1.78–1.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.44–1.49) | 2.94× (2.90–2.98) | 2.45× (2.42–2.49) | 3.48× (3.40–3.56) | 2.97× (2.85–3.10) | 3.40× (3.35–3.45) | 3.70× (3.65–3.75) | 1.34× (1.32–1.36) | 1.60× (1.56–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.70× (1.66–1.73) | 2.77× (2.71–2.84) | 2.09× (2.05–2.12) | 3.02× (2.96–3.08) | 2.74× (2.66–2.81) | 2.41× (2.38–2.45) | 2.62× (2.58–2.66) | 0.95× (0.93–0.97) | 1.13× (1.11–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.69× (2.66–2.73) | 3.52× (3.48–3.57) | 2.83× (2.79–2.87) | 6.07× (5.95–6.19) | 2.32× (2.27–2.36) | 2.14× (2.10–2.18) | 1.00× (0.97–1.03) | 0.97× (0.94–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.33× (2.30–2.37) | 4.01× (3.96–4.06) | 2.57× (2.54–2.60) | 5.35× (5.28–5.43) | 2.03× (2.01–2.06) | 1.88× (1.85–1.91) | 0.88× (0.85–0.90) | 0.85× (0.83–0.87) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.56) | 2.53× (2.50–2.57) | 3.95× (3.91–4.00) | 2.58× (2.55–2.60) | 5.32× (5.27–5.36) | 2.03× (2.00–2.05) | 1.87× (1.85–1.89) | 0.87× (0.86–0.89) | 0.84× (0.83–0.86) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 2.17× (2.13–2.22) | 4.18× (4.06–4.31) | 4.58× (4.52–4.64) | 4.62× (4.58–4.66) | 4.82× (4.75–4.89) | 5.19× (5.10–5.28) | 4.91× (4.85–4.98) | 1.64× (1.63–1.66) | 1.98× (1.97–1.99) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 1.78× (1.74–1.83) | 3.35× (3.23–3.49) | 3.67× (3.58–3.76) | 4.04× (3.96–4.13) | 3.76× (3.67–3.85) | 3.97× (3.88–4.07) | 3.76× (3.67–3.85) | 1.26× (1.22–1.29) | 1.52× (1.48–1.55) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 1.84× (1.75–1.93) | 2.69× (2.63–2.75) | 3.02× (2.94–3.11) | 3.23× (3.17–3.28) | 3.04× (2.96–3.13) | 2.59× (2.55–2.63) | 2.45× (2.42–2.49) | 0.82× (0.81–0.83) | 0.99× (0.98–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.28–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.02× (10.98–11.06) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.50× (0.41–0.61) | 0.72× (0.70–0.74) | 0.59× (0.57–0.61) | 5.68× (5.50–5.86) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.37× (0.36–0.39) | 0.69× (0.58–0.81) | 0.66× (0.64–0.69) | 0.58× (0.56–0.61) | 3.86× (3.72–4.02) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.66–0.67) | 7.96× (7.92–8.01) | 0.65× (0.64–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.72) | 0.76× (0.68–0.84) | 1.70× (1.66–1.73) | 0.85× (0.83–0.87) | 6.14× (6.01–6.27) | 0.50× (0.49–0.51) | 0.52× (0.51–0.54) | 0.57× (0.56–0.58) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.68–0.73) | 0.76× (0.70–0.83) | 1.26× (1.24–1.29) | 0.77× (0.75–0.80) | 3.71× (3.65–3.77) | 0.29× (0.29–0.30) | 0.31× (0.30–0.31) | 0.33× (0.33–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.52) | 0.40× (0.36–0.43) | 0.89× (0.86–0.92) | 0.70× (0.67–0.74) | 3.09× (2.97–3.20) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.65–0.69) | 0.47× (0.46–0.48) | 0.89× (0.88–0.91) | 0.84× (0.83–0.86) | 2.60× (2.55–2.65) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.22× (5.20–5.25) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.38× (0.36–0.40) | 0.84× (0.82–0.86) | 0.71× (0.70–0.73) | 2.89× (2.82–2.95) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.51) | 0.79× (0.78–0.81) | 0.81× (0.78–0.83) | 2.29× (2.25–2.34) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.31× (5.28–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.17× (6.15–6.19) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.17× (6.15–6.18) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.38–1.40) | 0.66× (0.66–0.67) | 6.37× (6.35–6.40) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 2 | 36 MB | 0.46× (0.44–0.47) | 0.37× (0.34–0.40) | 1.04× (1.00–1.08) | 0.69× (0.66–0.72) | 3.41× (3.29–3.54) | 0.21× (0.20–0.22) | 0.22× (0.22–0.23) | 0.34× (0.33–0.35) | 0.16× (0.16–0.17) |
| `text` (2.24 MB → 14.9 MB, 6.7×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.45× (0.42–0.47) | 0.89× (0.87–0.92) | 0.72× (0.69–0.75) | 2.30× (2.23–2.37) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 434 | 2.31 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 793 | 2.47 | 1.07× (1.06–1.07) | 90% | 1.03× (1.01–1.04) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 957 | 3.89 | 1.68× (1.66–1.70) | 55% | 1.25× (1.20–1.30) | 1.68× (1.63–1.74) |

**Best:** mibz2 at 4 threads (957 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.51, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.08 | 0.219 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3242 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.3% |
| MTF + RLE2 | 1.54 | 17.7% |
| T-vector build | 1.51 | 17.4% |
| walk (the one each build uses) | 4.27 | 49.2% |
| of which: segment assembly | 0.16 | 1.9% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.67 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.514 | 1.305 | (in its total) | 2.54× (2.53–2.55) |  |
| walk | 1.305 | 2.846 | (in its total) | 2.17× (2.15–2.18) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.060 | 0.064 | (in its total) | 1.06× (1.02–1.09) |  |
| inverse BWT, total | 1.878 | 4.217 | 3.421 | 2.23× (2.22–2.25) | 1.81× (1.81–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m34s |
| 4 threads | 32m08s ± 4m31s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m34s ± 3m20s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 23m34s ± 3m20s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 3.09 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,243,613 | 14,938,930 | \`64eba10b2c1f54ed…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3242.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
