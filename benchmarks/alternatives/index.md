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

The latest record: [`814d461`](<https://github.com/lmmx/mibz2/commit/814d461b790ef24eabb2be707f9e8354e4c180f6>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45933900314>)

Intel(R) Xeon(R) 6973P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.390 s | 538 | 7.8 | 3.33× (3.27–3.40) | 8.01× (7.79–8.24) | 8.19× (7.94–8.45) | 9.13× (8.84–9.43) | 8.50× (8.29–8.72) | 8.58× (8.49–8.68) | 8.37× (8.19–8.55) | 2.57× (2.52–2.62) | 2.34× (2.29–2.38) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.218 s | 963 | 8.4 | 3.37× (3.24–3.49) | 7.43× (7.04–7.85) | 8.73× (8.16–9.34) | 8.56× (8.15–8.99) | 7.95× (7.55–8.37) | 15.08× (14.30–15.91) | 14.70× (13.94–15.50) | 4.51× (4.26–4.78) | 4.11× (3.91–4.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,167 | 12.9 | 3.42× (3.34–3.49) | 6.55× (6.22–6.89) | 7.36× (7.04–7.69) | 7.21× (6.84–7.59) | 7.34× (7.06–7.64) | 18.50× (17.48–19.58) | 18.04× (17.05–19.08) | 5.54× (5.20–5.89) | 5.04× (4.78–5.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,153 | 4.0 | 0.61× (0.59–0.63) | 3.17× (3.00–3.33) | 4.35× (4.20–4.51) | 3.44× (3.29–3.60) | 6.20× (5.97–6.43) | 2.49× (2.44–2.55) | 2.55× (2.48–2.62) | 1.16× (1.14–1.18) | 1.19× (1.16–1.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,084 | 4.3 | 0.49× (0.48–0.51) | 2.79× (2.73–2.85) | 5.71× (5.53–5.90) | 3.27× (3.14–3.42) | 5.89× (5.71–6.09) | 2.35× (2.30–2.40) | 2.40× (2.33–2.48) | 1.10× (1.08–1.12) | 1.13× (1.10–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,093 | 4.3 | 0.50× (0.48–0.51) | 2.77× (2.68–2.87) | 5.61× (5.49–5.74) | 3.18× (3.10–3.26) | 5.79× (5.62–5.96) | 2.33× (2.26–2.40) | 2.38× (2.30–2.47) | 1.09× (1.05–1.12) | 1.12× (1.08–1.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 619 | 7.1 | 2.60× (2.55–2.64) | 2.97× (2.81–3.14) | 3.62× (3.47–3.78) | 3.98× (3.79–4.17) | 4.61× (4.45–4.79) | 2.96× (2.82–3.11) | 2.74× (2.63–2.87) | 1.96× (1.91–2.02) | 1.98× (1.89–2.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 970 | 8.5 | 2.28× (2.10–2.47) | 3.09× (2.67–3.57) | 4.10× (3.79–4.43) | 3.51× (3.23–3.81) | 5.14× (4.80–5.49) | 4.58× (4.24–4.96) | 4.25× (3.94–4.59) | 3.04× (2.87–3.22) | 3.07× (2.84–3.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 973 | 15.3 | 1.85× (1.76–1.94) | 2.07× (1.96–2.19) | 3.16× (2.98–3.35) | 2.30× (2.18–2.42) | 4.35× (4.12–4.59) | 4.59× (4.24–4.96) | 4.25× (3.94–4.59) | 3.04× (2.87–3.22) | 3.08× (2.84–3.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 75 | 56.6 | 1.40× (1.39–1.41) | 3.23× (3.18–3.29) | 2.95× (2.91–2.98) | 3.00× (2.98–3.01) | 3.35× (3.33–3.38) | 3.63× (3.60–3.66) | 3.66× (3.64–3.68) | 1.28× (1.27–1.29) | 1.33× (1.30–1.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 129 | 61.3 | 1.41× (1.35–1.47) | 3.09× (3.00–3.19) | 3.11× (2.95–3.28) | 3.74× (3.64–3.83) | 3.55× (3.46–3.63) | 6.17× (5.94–6.40) | 6.22× (5.99–6.47) | 2.17× (2.09–2.25) | 2.25× (2.18–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 154 | 86.4 | 1.50× (1.44–1.55) | 2.98× (2.90–3.07) | 2.91× (2.81–3.02) | 3.48× (3.37–3.59) | 3.59× (3.49–3.68) | 7.47× (7.28–7.67) | 7.54× (7.36–7.74) | 2.63× (2.57–2.69) | 2.73× (2.64–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.119 s | 81 | 52.9 | 1.44× (1.42–1.46) | 3.42× (3.28–3.58) | 2.96× (2.84–3.08) | 3.03× (2.90–3.18) | 3.44× (3.27–3.61) | 4.14× (3.93–4.37) | 4.40× (4.21–4.61) | 1.44× (1.37–1.51) | 1.52× (1.46–1.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 140 | 57.5 | 1.31× (1.21–1.42) | 3.39× (3.10–3.70) | 3.04× (2.83–3.27) | 4.01× (3.75–4.28) | 3.52× (3.26–3.79) | 7.23× (6.73–7.77) | 7.69× (7.15–8.27) | 2.52× (2.30–2.76) | 2.65× (2.43–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 165 | 83.9 | 1.48× (1.43–1.54) | 3.24× (3.09–3.41) | 2.94× (2.84–3.03) | 3.86× (3.74–3.98) | 3.52× (3.45–3.60) | 8.85× (8.56–9.14) | 9.40× (9.09–9.73) | 3.08× (2.96–3.20) | 3.24× (3.10–3.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 881 | 5.4 | 0.67× (0.66–0.68) | 2.73× (2.67–2.79) | 3.75× (3.66–3.84) | 2.91× (2.84–2.98) | 6.88× (6.68–7.09) | 2.20× (2.15–2.25) | 2.24× (2.19–2.30) | 1.10× (1.05–1.15) | 0.94× (0.92–0.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 816 | 6.0 | 0.50× (0.48–0.51) | 2.52× (2.45–2.59) | 5.15× (4.93–5.38) | 2.84× (2.67–3.03) | 6.61× (6.22–7.02) | 2.02× (1.96–2.08) | 2.06× (1.99–2.13) | 1.01× (0.96–1.07) | 0.86× (0.84–0.88) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 769 | 6.3 | 0.49× (0.46–0.52) | 2.55× (2.31–2.82) | 5.01× (4.80–5.22) | 2.67× (2.59–2.76) | 6.21× (6.06–6.37) | 1.93× (1.88–1.98) | 1.97× (1.92–2.02) | 0.97× (0.92–1.02) | 0.82× (0.80–0.84) | yes |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 1 | 0.056 s | 248 | 17.4 | 1.89× (1.87–1.91) | 4.15× (4.09–4.21) | 4.62× (4.52–4.71) | 4.66× (4.57–4.76) | 5.08× (5.01–5.15) | 4.98× (4.81–5.16) | 4.80× (4.68–4.93) | 1.53× (1.50–1.55) | 1.50× (1.42–1.59) | yes |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 2 | 0.037 s | 374 | 21.2 | 1.53× (1.38–1.69) | 3.44× (3.13–3.77) | 4.08× (3.69–4.51) | 4.26× (3.85–4.73) | 4.57× (4.17–5.01) | 7.26× (6.49–8.11) | 7.00× (6.31–7.76) | 2.22× (2.01–2.46) | 2.18× (1.94–2.46) | yes |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 4 | 0.033 s | 419 | 32.9 | 1.61× (1.56–1.65) | 3.07× (2.95–3.19) | 3.70× (3.57–3.84) | 3.89× (3.76–4.04) | 4.55× (4.44–4.66) | 8.54× (8.14–8.97) | 8.24× (7.94–8.55) | 2.62× (2.54–2.70) | 2.57× (2.43–2.73) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.32× (3.26–3.39) | 7.97× (7.74–8.20) | 8.15× (7.90–8.40) | 9.10× (8.81–9.40) | 8.47× (8.26–8.69) | 8.55× (8.45–8.65) | 8.34× (8.16–8.52) | 2.56× (2.51–2.61) | 2.32× (2.27–2.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.19× (3.10–3.27) | 7.54× (7.19–7.92) | 8.01× (7.55–8.49) | 8.81× (8.43–9.21) | 7.84× (7.51–8.19) | 7.84× (7.47–8.22) | 7.64× (7.30–7.99) | 2.35× (2.24–2.46) | 2.12× (2.04–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.37× (3.30–3.45) | 7.14× (6.86–7.44) | 7.36× (7.15–7.57) | 7.95× (7.69–8.21) | 7.36× (7.16–7.57) | 5.22× (5.01–5.43) | 5.09× (4.91–5.27) | 1.56× (1.50–1.63) | 1.41× (1.37–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.62–0.67) | 2.71× (2.61–2.81) | 3.69× (3.56–3.83) | 3.24× (3.11–3.38) | 5.77× (5.57–5.98) | 2.37× (2.31–2.42) | 2.43× (2.36–2.49) | 1.16× (1.14–1.18) | 0.82× (0.80–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.51–0.55) | 2.41× (2.35–2.47) | 4.70× (4.54–4.87) | 3.01× (2.89–3.14) | 5.33× (5.18–5.50) | 2.16× (2.11–2.22) | 2.22× (2.15–2.29) | 1.06× (1.04–1.07) | 0.75× (0.73–0.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.54) | 2.84× (2.77–2.91) | 4.67× (4.57–4.77) | 2.96× (2.88–3.03) | 5.28× (5.12–5.44) | 2.17× (2.11–2.23) | 2.22× (2.15–2.30) | 1.06× (1.02–1.09) | 0.75× (0.71–0.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.48–2.56) | 2.84× (2.71–2.97) | 3.38× (3.25–3.52) | 3.84× (3.65–4.03) | 4.45× (4.28–4.62) | 2.87× (2.73–3.02) | 2.66× (2.55–2.79) | 1.92× (1.87–1.97) | 1.85× (1.79–1.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.26–2.37) | 2.55× (2.49–2.61) | 3.16× (3.00–3.34) | 3.30× (3.15–3.44) | 3.83× (3.72–3.94) | 2.41× (2.32–2.50) | 2.24× (2.16–2.31) | 1.61× (1.59–1.64) | 1.56× (1.52–1.59) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.88–2.06) | 1.75× (1.65–1.85) | 2.24× (2.15–2.34) | 2.16× (2.08–2.25) | 2.35× (2.25–2.46) | 1.34× (1.28–1.40) | 1.24× (1.19–1.30) | 0.90× (0.86–0.93) | 0.87× (0.83–0.90) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.38–1.40) | 3.15× (3.10–3.19) | 2.88× (2.85–2.91) | 2.97× (2.95–2.98) | 3.32× (3.29–3.35) | 3.59× (3.56–3.61) | 3.62× (3.60–3.64) | 1.27× (1.26–1.28) | 1.25× (1.24–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.37× (1.32–1.41) | 2.99× (2.94–3.04) | 2.79× (2.66–2.93) | 3.47× (3.42–3.53) | 3.09× (3.04–3.13) | 3.26× (3.17–3.36) | 3.30× (3.20–3.40) | 1.16× (1.13–1.19) | 1.13× (1.10–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.51–1.64) | 3.00× (2.90–3.09) | 2.58× (2.49–2.67) | 3.29× (3.18–3.39) | 3.00× (2.91–3.10) | 2.35× (2.28–2.41) | 2.37× (2.31–2.43) | 0.83× (0.81–0.85) | 0.82× (0.79–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.46) | 3.36× (3.23–3.49) | 2.89× (2.78–3.00) | 3.01× (2.87–3.15) | 3.41× (3.24–3.58) | 4.10× (3.89–4.33) | 4.36× (4.17–4.56) | 1.44× (1.37–1.51) | 1.46× (1.40–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.25–1.43) | 3.27× (3.10–3.45) | 2.83× (2.67–3.00) | 3.73× (3.52–3.96) | 3.18× (2.98–3.41) | 3.83× (3.61–4.06) | 4.07× (3.84–4.32) | 1.34× (1.24–1.45) | 1.36× (1.25–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.55–1.67) | 3.35× (3.22–3.49) | 2.59× (2.51–2.68) | 3.51× (3.38–3.65) | 3.09× (3.00–3.19) | 2.74× (2.61–2.89) | 2.92× (2.79–3.05) | 0.96× (0.92–1.00) | 0.98× (0.93–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.71–0.72) | 2.24× (2.18–2.30) | 3.25× (3.18–3.32) | 2.69× (2.63–2.75) | 6.19× (5.98–6.41) | 2.07× (2.03–2.11) | 2.10× (2.06–2.14) | 1.09× (1.05–1.14) | 0.74× (0.73–0.75) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.53–0.58) | 2.05× (1.98–2.12) | 4.04× (3.94–4.15) | 2.59× (2.44–2.75) | 5.77× (5.48–6.07) | 1.83× (1.78–1.89) | 1.86× (1.80–1.92) | 0.97× (0.92–1.02) | 0.65× (0.63–0.68) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.51–0.56) | 2.44× (2.30–2.58) | 3.85× (3.70–4.01) | 2.48× (2.40–2.57) | 5.52× (5.39–5.66) | 1.79× (1.74–1.83) | 1.82× (1.77–1.86) | 0.95× (0.91–0.99) | 0.64× (0.61–0.67) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 1 | 1.87× (1.84–1.89) | 3.97× (3.93–4.00) | 4.46× (4.37–4.56) | 4.57× (4.48–4.66) | 4.97× (4.91–5.04) | 4.87× (4.71–5.04) | 4.70× (4.59–4.81) | 1.51× (1.49–1.54) | 1.43× (1.37–1.49) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 2 | 1.63× (1.61–1.65) | 3.43× (3.40–3.47) | 4.01× (3.98–4.04) | 4.23× (4.17–4.28) | 4.16× (4.12–4.20) | 4.05× (3.90–4.20) | 3.90× (3.78–4.02) | 1.26× (1.23–1.28) | 1.19× (1.14–1.24) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 4 | 1.76× (1.72–1.80) | 3.12× (3.02–3.22) | 3.46× (3.36–3.57) | 3.70× (3.58–3.82) | 3.54× (3.46–3.62) | 2.62× (2.49–2.76) | 2.53× (2.43–2.62) | 0.81× (0.79–0.84) | 0.77× (0.74–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.28× (0.27–0.29) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.96–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.49× (0.43–0.57) | 0.72× (0.69–0.74) | 0.61× (0.58–0.64) | 5.73× (5.54–5.92) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.35–0.38) | 0.46× (0.43–0.50) | 0.64× (0.61–0.66) | 0.57× (0.54–0.60) | 3.77× (3.63–3.92) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.68× (1.68–1.68) | 0.66× (0.65–0.66) | 7.93× (7.91–7.94) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.73× (0.72–0.74) | 0.71× (0.67–0.76) | 1.72× (1.69–1.76) | 0.86× (0.84–0.88) | 6.25× (6.14–6.37) | 0.51× (0.50–0.52) | 0.53× (0.52–0.55) | 0.58× (0.57–0.59) | 0.29× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.71) | 0.71× (0.67–0.76) | 1.24× (1.20–1.28) | 0.77× (0.74–0.80) | 3.71× (3.59–3.82) | 0.29× (0.28–0.30) | 0.31× (0.30–0.32) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.15–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.13) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.41) | 0.89× (0.86–0.92) | 0.69× (0.65–0.73) | 3.07× (2.98–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.66) | 0.47× (0.46–0.48) | 0.88× (0.87–0.90) | 0.82× (0.80–0.85) | 2.57× (2.52–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.44–0.48) | 0.33× (0.31–0.35) | 0.82× (0.79–0.85) | 0.68× (0.65–0.71) | 2.82× (2.73–2.91) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.49× (0.45–0.53) | 0.80× (0.79–0.82) | 0.83× (0.81–0.85) | 2.33× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.19–0.20) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.32× (5.30–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.18–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.17–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.39) | 0.65× (0.65–0.66) | 6.34× (6.33–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.39× (0.36–0.42) | 1.08× (1.04–1.12) | 0.71× (0.69–0.73) | 3.49× (3.41–3.57) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.26 MB → 13.8 MB, 6.1×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.46× (0.43–0.50) | 0.94× (0.91–0.96) | 0.77× (0.74–0.79) | 2.40× (2.35–2.46) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 538 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 963 | 2.00 | 1.09× (1.04–1.14) | 88% | 1.03× (1.00–1.07) | 1.05× (1.02–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,167 | 3.06 | 1.64× (1.58–1.70) | 54% | 1.47× (1.43–1.51) | 1.67× (1.62–1.71) |

**Best:** mibz2 at 4 threads (1,167 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (344 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.09, lbzip2 1t 0.04, lbzip2 2t 0.23, lbzip2 4t 0.27, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.48 | 0.02 | 0.08 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4201 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.05 | 9.1% |
| MTF + RLE2 | 1.76 | 15.3% |
| T-vector build | 1.64 | 14.3% |
| walk (the one each build uses) | 6.64 | 57.7% |
| of which: segment assembly | 0.34 | 2.9% |
| RLE1 | 0.27 | 2.4% |
| CRC | 0.15 | 1.3% |
| **total** | 11.50 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.349 | 0.873 | (in its total) | 2.49× (2.41–2.57) |  |
| walk | 1.462 | 2.793 | (in its total) | 1.91× (1.88–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.92× (0.91–0.93) |  |
| inverse BWT, total | 1.891 | 3.739 | 1.922 | 1.97× (1.94–2.01) | 1.02× (0.99–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m09s ± 6m54s |
| 4 threads | 26m34s ± 3m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m49s ± 2m36s |
| the scanner alone (its serial part) | 8m44s ± 41.6s |
| the workers alone, over 4 | 17m49s ± 2m36s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.34 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,260,569 | 13,838,398 | \`e9896b813104a306…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6973P-C, 4 cores, reported clock 4200.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
