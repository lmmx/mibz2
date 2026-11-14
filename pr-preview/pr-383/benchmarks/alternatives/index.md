# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-383/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`6dec95b`](<https://github.com/lmmx/mibz2/commit/6dec95ba6aaf59d5134e5ab119fd52433e2032ee>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43529976995>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.525 s | 399 | 6.2 | 2.91× (2.89–2.92) | 6.60× (6.55–6.65) | 6.79× (6.73–6.86) | 7.38× (7.32–7.45) | 6.66× (6.60–6.72) | 7.26× (7.20–7.32) | 6.78× (6.73–6.83) | 2.16× (2.12–2.20) | 2.51× (2.50–2.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.318 s | 659 | 6.9 | 2.70× (2.53–2.88) | 5.66× (5.37–5.97) | 6.56× (6.22–6.92) | 6.43× (6.10–6.79) | 5.82× (5.50–6.15) | 11.80× (11.16–12.49) | 11.03× (10.43–11.66) | 3.51× (3.30–3.74) | 4.09× (3.86–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.267 s | 785 | 11.4 | 2.65× (2.61–2.68) | 4.08× (3.99–4.17) | 5.08× (4.96–5.20) | 4.62× (4.45–4.79) | 4.88× (4.58–5.20) | 14.14× (13.93–14.35) | 13.21× (13.02–13.41) | 4.21× (4.13–4.29) | 4.90× (4.83–4.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 2.7 | 0.60× (0.59–0.61) | 3.66× (3.49–3.84) | 4.50× (4.39–4.61) | 3.59× (3.54–3.64) | 6.18× (6.09–6.27) | 2.99× (2.95–3.03) | 2.67× (2.63–2.71) | 1.01× (1.00–1.03) | 1.52× (1.49–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 973 | 2.8 | 0.57× (0.56–0.58) | 2.99× (2.94–3.04) | 5.99× (5.87–6.12) | 3.56× (3.50–3.62) | 6.06× (5.98–6.15) | 2.95× (2.90–3.00) | 2.64× (2.59–2.68) | 1.00× (0.98–1.02) | 1.50× (1.47–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 971 | 2.8 | 0.57× (0.57–0.58) | 3.00× (2.98–3.02) | 5.89× (5.85–5.94) | 3.55× (3.52–3.58) | 6.11× (6.05–6.17) | 2.94× (2.92–2.96) | 2.63× (2.61–2.65) | 1.00× (0.99–1.01) | 1.49× (1.48–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 583 | 4.4 | 2.86× (2.79–2.94) | 3.04× (3.02–3.07) | 3.91× (3.88–3.94) | 3.83× (3.79–3.86) | 4.14× (4.11–4.17) | 2.56× (2.54–2.58) | 2.13× (2.11–2.14) | 2.03× (2.01–2.04) | 3.03× (3.01–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 881 | 5.5 | 2.23× (2.13–2.34) | 2.90× (2.63–3.19) | 4.06× (3.87–4.26) | 3.55× (3.44–3.66) | 4.77× (4.59–4.97) | 3.81× (3.66–3.96) | 3.16× (3.04–3.28) | 3.01× (2.90–3.13) | 4.51× (4.33–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 763 | 10.7 | 1.82× (1.75–1.90) | 1.98× (1.82–2.16) | 2.91× (2.74–3.10) | 2.20× (2.10–2.31) | 3.83× (3.66–4.00) | 3.33× (3.18–3.49) | 2.77× (2.65–2.89) | 2.64× (2.52–2.75) | 3.95× (3.77–4.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 41.6 | 1.33× (1.31–1.36) | 2.63× (2.59–2.66) | 2.38× (2.32–2.44) | 2.39× (2.36–2.42) | 2.78× (2.74–2.82) | 3.07× (3.03–3.11) | 3.11× (3.08–3.15) | 1.19× (1.18–1.21) | 1.36× (1.34–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.084 s | 96 | 47.4 | 1.26× (1.22–1.30) | 2.44× (2.37–2.51) | 2.31× (2.24–2.39) | 2.87× (2.77–2.96) | 2.85× (2.78–2.92) | 4.94× (4.83–5.05) | 5.01× (4.87–5.15) | 1.92× (1.87–1.97) | 2.19× (2.13–2.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 63.5 | 1.30× (1.27–1.32) | 2.12× (2.04–2.21) | 2.04× (1.97–2.11) | 2.43× (2.36–2.50) | 2.74× (2.70–2.79) | 5.87× (5.78–5.96) | 5.95× (5.87–6.04) | 2.28× (2.24–2.32) | 2.61× (2.56–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 38.1 | 1.46× (1.43–1.49) | 2.97× (2.92–3.01) | 2.47× (2.43–2.52) | 2.55× (2.51–2.58) | 3.00× (2.95–3.06) | 3.56× (3.50–3.62) | 3.82× (3.77–3.87) | 1.36× (1.34–1.38) | 1.67× (1.63–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 43.1 | 1.34× (1.31–1.37) | 2.73× (2.65–2.82) | 2.41× (2.34–2.48) | 3.24× (3.13–3.35) | 3.02× (2.94–3.10) | 5.89× (5.74–6.05) | 6.32× (6.14–6.52) | 2.25× (2.19–2.31) | 2.77× (2.67–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 62.3 | 1.39× (1.35–1.43) | 2.26× (2.22–2.30) | 2.19× (2.14–2.24) | 2.80× (2.74–2.85) | 2.75× (2.70–2.79) | 6.77× (6.64–6.90) | 7.26× (7.16–7.38) | 2.59× (2.55–2.62) | 3.18× (3.13–3.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 3.8 | 0.68× (0.67–0.68) | 3.14× (2.99–3.29) | 4.50× (4.46–4.54) | 3.16× (3.13–3.19) | 6.83× (6.74–6.92) | 2.56× (2.54–2.59) | 2.36× (2.34–2.39) | 0.99× (0.98–1.00) | 1.54× (1.53–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 4.1 | 0.56× (0.55–0.57) | 2.87× (2.82–2.92) | 4.93× (4.84–5.02) | 2.99× (2.94–3.05) | 6.46× (6.35–6.57) | 2.40× (2.35–2.44) | 2.21× (2.17–2.25) | 0.93× (0.91–0.95) | 1.44× (1.42–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 702 | 4.0 | 0.57× (0.56–0.58) | 2.89× (2.86–2.93) | 4.96× (4.89–5.02) | 3.00× (2.97–3.04) | 6.46× (6.38–6.54) | 2.40× (2.37–2.44) | 2.22× (2.19–2.25) | 0.93× (0.91–0.95) | 1.45× (1.43–1.47) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.071 s | 183 | 13.7 | 1.94× (1.91–1.96) | 4.02× (3.95–4.10) | 4.39× (4.32–4.47) | 4.40× (4.33–4.46) | 4.61× (4.54–4.68) | 5.15× (5.08–5.23) | 4.83× (4.76–4.90) | 1.49× (1.47–1.51) | 1.84× (1.82–1.87) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.049 s | 264 | 17.2 | 1.58× (1.47–1.69) | 3.20× (3.06–3.35) | 3.76× (3.52–4.02) | 3.90× (3.64–4.17) | 4.09× (3.83–4.37) | 7.17× (6.72–7.65) | 6.72× (6.26–7.21) | 2.07× (1.93–2.21) | 2.57× (2.40–2.74) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.046 s | 284 | 26.4 | 1.46× (1.42–1.51) | 2.41× (2.31–2.52) | 3.06× (2.95–3.16) | 3.19× (3.07–3.32) | 3.63× (3.55–3.71) | 8.00× (7.85–8.15) | 7.49× (7.33–7.66) | 2.31× (2.25–2.36) | 2.86× (2.81–2.92) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.89–2.92) | 6.57× (6.52–6.62) | 6.76× (6.70–6.83) | 7.36× (7.30–7.42) | 6.63× (6.57–6.70) | 7.23× (7.17–7.29) | 6.76× (6.71–6.81) | 2.16× (2.12–2.19) | 2.50× (2.48–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.67× (2.62–2.73) | 5.95× (5.84–6.07) | 6.25× (6.11–6.39) | 6.80× (6.62–6.99) | 5.89× (5.75–6.04) | 6.34× (6.19–6.49) | 5.92× (5.81–6.04) | 1.89× (1.85–1.93) | 2.19× (2.13–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.57× (2.52–2.63) | 4.32× (4.26–4.37) | 4.70× (4.59–4.81) | 4.91× (4.78–5.05) | 4.40× (4.34–4.46) | 3.90× (3.85–3.94) | 3.64× (3.60–3.68) | 1.16× (1.14–1.19) | 1.35× (1.32–1.37) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.27× (3.22–3.32) | 4.12× (4.04–4.20) | 3.35× (3.31–3.40) | 5.71× (5.63–5.79) | 2.81× (2.77–2.85) | 2.52× (2.49–2.56) | 1.01× (1.00–1.02) | 1.08× (1.05–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.10× (3.03–3.16) | 5.38× (5.30–5.46) | 3.30× (3.25–3.36) | 5.54× (5.47–5.62) | 2.73× (2.69–2.78) | 2.46× (2.41–2.50) | 0.98× (0.97–1.00) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.41× (3.36–3.46) | 5.44× (5.37–5.52) | 3.34× (3.31–3.37) | 5.63× (5.56–5.70) | 2.76× (2.74–2.79) | 2.48× (2.46–2.50) | 0.99× (0.98–1.00) | 1.06× (1.03–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.76× (2.69–2.84) | 2.80× (2.79–2.82) | 3.50× (3.48–3.53) | 3.67× (3.64–3.70) | 3.97× (3.94–3.99) | 2.48× (2.46–2.49) | 2.07× (2.05–2.08) | 1.97× (1.96–1.98) | 2.71× (2.68–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.23–2.29) | 2.49× (2.42–2.56) | 3.07× (3.04–3.10) | 3.30× (3.27–3.34) | 3.38× (3.33–3.42) | 2.00× (1.97–2.03) | 1.67× (1.65–1.69) | 1.59× (1.57–1.61) | 2.19× (2.16–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.95–2.13) | 1.77× (1.70–1.84) | 2.24× (2.15–2.32) | 2.15× (2.05–2.25) | 2.15× (2.07–2.24) | 1.04× (1.00–1.08) | 0.87× (0.83–0.90) | 0.83× (0.79–0.86) | 1.14× (1.09–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.35) | 2.56× (2.53–2.59) | 2.33× (2.28–2.38) | 2.37× (2.34–2.40) | 2.76× (2.72–2.79) | 3.04× (3.00–3.08) | 3.08× (3.05–3.12) | 1.19× (1.18–1.21) | 1.33× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.22–1.26) | 2.38× (2.34–2.42) | 2.11× (2.08–2.14) | 2.70× (2.66–2.73) | 2.49× (2.47–2.52) | 2.68× (2.65–2.71) | 2.72× (2.67–2.76) | 1.05× (1.04–1.06) | 1.18× (1.16–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.40–1.49) | 2.24× (2.19–2.30) | 1.84× (1.80–1.87) | 2.42× (2.37–2.47) | 2.38× (2.34–2.42) | 2.00× (1.97–2.04) | 2.03× (1.98–2.08) | 0.79× (0.77–0.80) | 0.88× (0.86–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.42–1.48) | 2.91× (2.87–2.96) | 2.43× (2.39–2.46) | 2.53× (2.49–2.56) | 2.97× (2.92–3.03) | 3.53× (3.46–3.59) | 3.78× (3.74–3.83) | 1.36× (1.34–1.38) | 1.63× (1.60–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.36) | 2.73× (2.69–2.77) | 2.26× (2.24–2.28) | 3.01× (2.98–3.05) | 2.70× (2.66–2.73) | 3.13× (3.08–3.17) | 3.35× (3.31–3.40) | 1.20× (1.19–1.21) | 1.45× (1.42–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.52–1.58) | 2.46× (2.40–2.51) | 1.88× (1.84–1.92) | 2.59× (2.53–2.65) | 2.45× (2.40–2.49) | 2.17× (2.12–2.23) | 2.33× (2.28–2.39) | 0.84× (0.82–0.85) | 1.01× (0.98–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.72) | 2.78× (2.76–2.80) | 3.61× (3.57–3.65) | 2.88× (2.86–2.91) | 6.09× (6.01–6.16) | 2.36× (2.34–2.38) | 2.19× (2.17–2.21) | 1.00× (0.99–1.01) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.50× (2.46–2.54) | 4.26× (4.21–4.31) | 2.75× (2.70–2.80) | 5.69× (5.60–5.79) | 2.17× (2.13–2.22) | 2.02× (1.98–2.06) | 0.92× (0.90–0.93) | 0.92× (0.88–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.79× (2.75–2.84) | 4.38× (4.32–4.44) | 2.83× (2.80–2.87) | 5.82× (5.74–5.90) | 2.23× (2.20–2.27) | 2.07× (2.04–2.10) | 0.94× (0.92–0.96) | 0.94× (0.91–0.98) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.91× (1.88–1.94) | 3.91× (3.85–3.97) | 4.22× (4.16–4.29) | 4.30× (4.24–4.37) | 4.51× (4.45–4.58) | 5.04× (4.97–5.11) | 4.72× (4.66–4.79) | 1.47× (1.45–1.49) | 1.77× (1.75–1.79) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.66× (1.63–1.68) | 3.27× (3.22–3.31) | 3.57× (3.54–3.61) | 3.94× (3.88–4.00) | 3.72× (3.68–3.77) | 4.03× (3.99–4.08) | 3.78× (3.73–3.84) | 1.18× (1.16–1.19) | 1.42× (1.40–1.43) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.67× (1.60–1.75) | 2.64× (2.57–2.72) | 2.85× (2.78–2.92) | 3.10× (3.04–3.17) | 3.01× (2.93–3.09) | 2.64× (2.57–2.71) | 2.47× (2.41–2.54) | 0.77× (0.75–0.79) | 0.93× (0.91–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.34) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.93× (11.90–11.95) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.51× (0.45–0.59) | 0.75× (0.74–0.77) | 0.63× (0.62–0.64) | 5.98× (5.94–6.03) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.38–0.38) | 0.68× (0.59–0.79) | 0.68× (0.67–0.69) | 0.59× (0.57–0.60) | 3.95× (3.93–3.97) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.78) | 0.84× (0.83–0.84) | 4.38× (4.36–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.71) | 0.66× (0.66–0.66) | 8.02× (8.00–8.05) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.65–0.68) | 0.70× (0.64–0.76) | 1.60× (1.56–1.64) | 0.80× (0.78–0.82) | 5.80× (5.66–5.94) | 0.47× (0.45–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.59–0.66) | 0.71× (0.66–0.76) | 1.12× (1.07–1.17) | 0.69× (0.66–0.72) | 3.31× (3.16–3.46) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.36× (0.33–0.40) | 0.90× (0.87–0.92) | 0.71× (0.69–0.73) | 3.10× (3.02–3.17) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.80× (0.77–0.83) | 2.53× (2.47–2.58) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.50) | 0.38× (0.36–0.41) | 0.86× (0.84–0.88) | 0.71× (0.68–0.75) | 2.96× (2.89–3.03) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.51× (0.48–0.53) | 0.80× (0.79–0.81) | 0.81× (0.78–0.84) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.21× (0.20–0.21) | 1.08× (1.08–1.08) | 0.77× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.77) | 5.30× (5.27–5.32) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.62) | 0.77× (0.77–0.78) | 5.33× (5.31–5.35) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.67) | 6.39× (6.37–6.41) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 35 MB | 0.48× (0.48–0.49) | 0.41× (0.38–0.45) | 1.15× (1.12–1.18) | 0.74× (0.72–0.77) | 3.58× (3.51–3.64) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 54 MB | 0.51× (0.49–0.53) | 0.53× (0.49–0.58) | 0.93× (0.92–0.95) | 0.74× (0.70–0.78) | 2.37× (2.32–2.42) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 399 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 659 | 2.83 | 1.14× (1.11–1.17) | 81% | 1.03× (1.03–1.04) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 785 | 4.66 | 1.86× (1.83–1.89) | 49% | 1.22× (1.21–1.23) | 1.65× (1.62–1.67) |

**Best:** mibz2 at 4 threads (785 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.78 | 0.02 | 0.08 | 0.267 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.6% |
| MTF + RLE2 | 1.20 | 17.7% |
| T-vector build | 1.15 | 16.9% |
| walk (the one each build uses) | 3.17 | 46.8% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.9% |
| **total** | 6.77 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.488 | 1.306 | (in its total) | 2.68× (2.67–2.69) |  |
| walk | 1.228 | 2.659 | (in its total) | 2.17× (2.16–2.18) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.056 | (in its total) | 1.22× (1.20–1.24) |  |
| inverse BWT, total | 1.762 | 4.022 | 3.292 | 2.29× (2.28–2.30) | 1.87× (1.86–1.87) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m34s |
| 4 threads | 39m04s ± 5m30s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m52s ± 3m57s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 27m52s ± 3m57s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.65 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
