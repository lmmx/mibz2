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

The latest record: [`575fa98`](<https://github.com/lmmx/mibz2/commit/575fa98a9e80701c4a42c0799017cdc75691fe46>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46166170705>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.480 s | 437 | 7.5 | 3.14× (3.05–3.23) | 7.18× (6.99–7.38) | 7.33× (7.13–7.54) | 8.01× (7.78–8.24) | 7.20× (7.01–7.40) | 7.83× (7.61–8.06) | 7.22× (7.02–7.43) | 2.32× (2.26–2.37) | 2.72× (2.64–2.80) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.267 s | 786 | 8.0 | 3.24× (3.11–3.38) | 6.82× (6.50–7.16) | 7.65× (7.28–8.04) | 7.62× (7.32–7.94) | 6.90× (6.63–7.17) | 13.92× (13.41–14.46) | 12.84× (12.35–13.35) | 4.12× (3.97–4.27) | 4.83× (4.62–5.05) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.220 s | 953 | 12.7 | 3.20× (3.16–3.24) | 4.97× (4.87–5.08) | 6.10× (5.98–6.23) | 5.59× (5.53–5.65) | 5.67× (5.59–5.75) | 17.34× (17.14–17.54) | 15.99× (15.81–16.18) | 5.13× (5.09–5.17) | 6.02× (5.92–6.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 976 | 3.7 | 0.60× (0.59–0.61) | 3.68× (3.49–3.87) | 4.50× (4.35–4.66) | 3.57× (3.52–3.62) | 6.18× (6.10–6.27) | 2.96× (2.92–3.01) | 2.65× (2.61–2.69) | 1.00× (0.99–1.02) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 924 | 4.0 | 0.54× (0.53–0.55) | 2.83× (2.78–2.88) | 5.62× (5.51–5.74) | 3.38× (3.32–3.44) | 5.78× (5.67–5.89) | 2.79× (2.74–2.84) | 2.50× (2.46–2.54) | 0.95× (0.92–0.97) | 1.41× (1.39–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 893 | 4.0 | 0.53× (0.53–0.54) | 2.78× (2.74–2.83) | 5.51× (5.39–5.64) | 3.30× (3.25–3.35) | 5.67× (5.58–5.77) | 2.74× (2.69–2.78) | 2.45× (2.41–2.49) | 0.93× (0.91–0.95) | 1.39× (1.36–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 604 | 5.7 | 2.98× (2.87–3.09) | 3.14× (3.11–3.16) | 4.02× (3.99–4.05) | 3.97× (3.94–4.00) | 4.32× (4.28–4.36) | 2.64× (2.63–2.66) | 2.20× (2.18–2.21) | 2.09× (2.07–2.11) | 3.12× (3.10–3.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 922 | 7.0 | 2.38× (2.32–2.45) | 3.12× (2.83–3.45) | 4.31× (4.05–4.58) | 3.58× (3.51–3.65) | 5.10× (4.98–5.23) | 4.06× (3.97–4.14) | 3.37× (3.30–3.44) | 3.21× (3.14–3.27) | 4.79× (4.69–4.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 856 | 13.7 | 2.01× (1.93–2.09) | 2.15× (2.04–2.27) | 3.21× (3.02–3.41) | 2.41× (2.31–2.51) | 4.32× (4.15–4.50) | 3.72× (3.59–3.87) | 3.09× (2.98–3.22) | 2.94× (2.84–3.05) | 4.40× (4.23–4.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.122 s | 65 | 50.4 | 1.45× (1.44–1.46) | 2.90× (2.88–2.92) | 2.61× (2.58–2.63) | 2.68× (2.66–2.70) | 3.06× (3.03–3.09) | 3.35× (3.32–3.37) | 3.34× (3.32–3.36) | 1.30× (1.29–1.32) | 1.49× (1.48–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 s | 110 | 56.1 | 1.39× (1.33–1.46) | 2.82× (2.64–3.00) | 2.62× (2.48–2.77) | 3.41× (3.20–3.63) | 3.23× (3.05–3.42) | 5.55× (5.26–5.85) | 5.53× (5.24–5.83) | 2.16× (2.05–2.28) | 2.46× (2.34–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 128 | 76.6 | 1.45× (1.41–1.48) | 2.35× (2.29–2.40) | 2.31× (2.22–2.40) | 2.89× (2.80–2.97) | 3.15× (3.08–3.21) | 6.63× (6.51–6.75) | 6.61× (6.49–6.73) | 2.58× (2.53–2.64) | 2.95× (2.90–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.137 s | 70 | 46.9 | 1.54× (1.52–1.57) | 3.23× (3.18–3.29) | 2.73× (2.68–2.77) | 2.81× (2.77–2.86) | 3.27× (3.22–3.32) | 3.83× (3.77–3.89) | 4.08× (3.99–4.18) | 1.48× (1.46–1.50) | 1.77× (1.74–1.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 116 | 52.4 | 1.43× (1.39–1.46) | 2.97× (2.88–3.06) | 2.62× (2.54–2.69) | 3.70× (3.63–3.78) | 3.31× (3.23–3.39) | 6.38× (6.23–6.53) | 6.80× (6.62–6.98) | 2.46× (2.40–2.53) | 2.95× (2.88–3.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 76.3 | 1.52× (1.49–1.56) | 2.54× (2.47–2.61) | 2.41× (2.35–2.46) | 3.24× (3.17–3.31) | 3.06× (3.02–3.11) | 7.50× (7.40–7.60) | 8.00× (7.85–8.15) | 2.90× (2.86–2.93) | 3.47× (3.42–3.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.1 | 0.67× (0.66–0.67) | 3.11× (2.95–3.27) | 4.45× (4.42–4.47) | 3.14× (3.12–3.16) | 6.89× (6.82–6.97) | 2.55× (2.53–2.56) | 2.34× (2.32–2.35) | 0.99× (0.98–1.01) | 1.53× (1.52–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 5.8 | 0.53× (0.50–0.57) | 2.71× (2.59–2.83) | 4.83× (4.52–5.18) | 2.83× (2.71–2.96) | 6.23× (5.92–6.56) | 2.26× (2.17–2.37) | 2.08× (1.98–2.17) | 0.88× (0.84–0.93) | 1.36× (1.30–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 660 | 5.8 | 0.54× (0.53–0.54) | 2.73× (2.71–2.75) | 4.67× (4.64–4.71) | 2.83× (2.81–2.86) | 6.20× (6.15–6.25) | 2.27× (2.25–2.29) | 2.08× (2.07–2.10) | 0.89× (0.88–0.90) | 1.37× (1.36–1.38) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 0.064 s | 236 | 14.2 | 2.25× (2.23–2.26) | 4.43× (4.40–4.47) | 4.84× (4.80–4.89) | 4.85× (4.81–4.88) | 5.02× (4.99–5.06) | 5.42× (5.38–5.46) | 4.92× (4.87–4.97) | 1.68× (1.67–1.70) | 2.17× (2.13–2.22) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 0.047 s | 322 | 18.6 | 1.61× (1.50–1.72) | 3.31× (3.01–3.65) | 3.84× (3.63–4.06) | 4.32× (4.02–4.63) | 4.14× (3.88–4.43) | 7.16× (6.70–7.66) | 6.50× (6.09–6.95) | 2.23× (2.08–2.38) | 2.87× (2.69–3.07) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 0.041 s | 368 | 29.0 | 1.64× (1.59–1.69) | 2.58× (2.51–2.65) | 3.18× (3.07–3.30) | 3.67× (3.61–3.73) | 3.88× (3.82–3.95) | 8.49× (8.30–8.69) | 7.71× (7.52–7.90) | 2.64× (2.58–2.69) | 3.41× (3.28–3.54) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.13× (3.04–3.23) | 7.15× (6.96–7.35) | 7.30× (7.10–7.51) | 7.99× (7.76–8.22) | 7.18× (6.99–7.38) | 7.81× (7.59–8.03) | 7.20× (7.00–7.41) | 2.31× (2.25–2.37) | 2.70× (2.62–2.79) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.06–3.10) | 6.90× (6.83–6.97) | 7.22× (7.16–7.27) | 7.94× (7.88–7.99) | 6.88× (6.84–6.92) | 7.36× (7.31–7.42) | 6.79× (6.74–6.85) | 2.18× (2.16–2.19) | 2.55× (2.51–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.09× (3.05–3.14) | 5.17× (5.10–5.24) | 5.52× (5.48–5.57) | 5.84× (5.79–5.90) | 5.22× (5.18–5.26) | 4.67× (4.63–4.70) | 4.31× (4.27–4.34) | 1.38× (1.37–1.40) | 1.62× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.25× (3.19–3.30) | 4.06× (4.00–4.11) | 3.33× (3.28–3.37) | 5.71× (5.63–5.79) | 2.78× (2.74–2.82) | 2.50× (2.46–2.53) | 1.00× (0.99–1.02) | 1.10× (1.03–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.58) | 2.86× (2.81–2.90) | 5.01× (4.94–5.09) | 3.08× (3.03–3.14) | 5.19× (5.09–5.28) | 2.54× (2.50–2.58) | 2.28× (2.25–2.32) | 0.92× (0.90–0.94) | 1.01× (0.95–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.56) | 3.13× (3.06–3.21) | 4.95× (4.88–5.03) | 3.06× (3.01–3.11) | 5.15× (5.07–5.23) | 2.53× (2.49–2.57) | 2.27× (2.24–2.31) | 0.91× (0.89–0.93) | 1.00× (0.94–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.86× (2.76–2.96) | 2.88× (2.86–2.90) | 3.60× (3.57–3.63) | 3.80× (3.77–3.82) | 4.13× (4.09–4.16) | 2.54× (2.53–2.56) | 2.12× (2.11–2.14) | 2.02× (2.00–2.04) | 2.80× (2.77–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.33× (2.28–2.39) | 2.62× (2.56–2.69) | 3.18× (3.09–3.27) | 3.28× (3.21–3.36) | 3.51× (3.43–3.59) | 2.07× (2.02–2.12) | 1.73× (1.69–1.77) | 1.65× (1.61–1.68) | 2.28× (2.23–2.33) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.09× (2.03–2.16) | 1.78× (1.73–1.83) | 2.27× (2.21–2.33) | 2.09× (2.01–2.16) | 2.16× (2.11–2.22) | 1.06× (1.03–1.09) | 0.88× (0.86–0.91) | 0.84× (0.82–0.86) | 1.16× (1.13–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.43–1.45) | 2.82× (2.80–2.84) | 2.53× (2.52–2.55) | 2.65× (2.64–2.67) | 3.03× (3.00–3.05) | 3.31× (3.29–3.33) | 3.30× (3.28–3.32) | 1.30× (1.29–1.31) | 1.46× (1.44–1.47) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.37× (1.36–1.38) | 2.69× (2.66–2.71) | 2.37× (2.36–2.38) | 3.20× (3.13–3.26) | 2.79× (2.77–2.81) | 2.99× (2.97–3.01) | 2.98× (2.96–3.00) | 1.17× (1.16–1.18) | 1.31× (1.31–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.51–1.59) | 2.47× (2.41–2.53) | 2.00× (1.96–2.03) | 2.76× (2.70–2.83) | 2.62× (2.58–2.67) | 2.18× (2.14–2.22) | 2.17× (2.14–2.21) | 0.85× (0.84–0.87) | 0.96× (0.94–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.51–1.56) | 3.17× (3.12–3.21) | 2.65× (2.60–2.70) | 2.79× (2.75–2.83) | 3.24× (3.19–3.29) | 3.79× (3.73–3.84) | 4.04× (3.94–4.13) | 1.47× (1.45–1.49) | 1.73× (1.71–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.45× (1.44–1.46) | 2.97× (2.93–3.00) | 2.48× (2.46–2.50) | 3.46× (3.42–3.49) | 2.96× (2.93–2.99) | 3.40× (3.38–3.42) | 3.62× (3.56–3.69) | 1.32× (1.31–1.33) | 1.56× (1.54–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.61–1.70) | 2.64× (2.61–2.67) | 2.03× (2.00–2.06) | 2.90× (2.84–2.96) | 2.64× (2.60–2.68) | 2.35× (2.32–2.37) | 2.50× (2.45–2.55) | 0.91× (0.90–0.92) | 1.07× (1.06–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.74× (2.72–2.77) | 3.57× (3.54–3.60) | 2.86× (2.84–2.88) | 6.13× (6.10–6.16) | 2.34× (2.33–2.36) | 2.16× (2.16–2.17) | 0.99× (0.98–1.00) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.55–0.61) | 2.40× (2.23–2.58) | 3.98× (3.81–4.17) | 2.54× (2.44–2.64) | 5.35× (5.10–5.62) | 2.00× (1.92–2.09) | 1.85× (1.77–1.93) | 0.85× (0.81–0.89) | 0.86× (0.81–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.57× (2.53–2.61) | 3.97× (3.93–4.02) | 2.60× (2.57–2.63) | 5.43× (5.37–5.48) | 2.05× (2.02–2.07) | 1.89× (1.87–1.91) | 0.87× (0.86–0.88) | 0.88× (0.85–0.91) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 2.21× (2.19–2.23) | 4.24× (4.21–4.27) | 4.63× (4.59–4.67) | 4.73× (4.70–4.77) | 4.90× (4.87–4.94) | 5.29× (5.25–5.33) | 4.80× (4.76–4.85) | 1.66× (1.65–1.68) | 2.03× (2.02–2.04) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 1.78× (1.75–1.80) | 3.37× (3.29–3.44) | 3.71× (3.65–3.77) | 4.11× (4.06–4.15) | 3.83× (3.78–3.89) | 4.02× (3.96–4.08) | 3.65× (3.60–3.71) | 1.26× (1.25–1.28) | 1.54× (1.52–1.56) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 1.76× (1.71–1.81) | 2.65× (2.60–2.69) | 2.93× (2.86–3.00) | 3.14× (3.07–3.22) | 2.97× (2.90–3.05) | 2.58× (2.53–2.64) | 2.35× (2.29–2.41) | 0.81× (0.79–0.83) | 0.99× (0.97–1.01) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.27–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (11.00–11.03) | 0.26× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.51× (0.43–0.62) | 0.73× (0.70–0.76) | 0.61× (0.59–0.63) | 5.76× (5.56–5.97) | 0.14× (0.13–0.14) | 0.15× (0.14–0.16) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.35× (0.34–0.37) | 0.69× (0.58–0.82) | 0.63× (0.60–0.66) | 0.55× (0.52–0.58) | 3.67× (3.50–3.85) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.93–0.93) | 4.83× (4.82–4.85) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.68) | 0.66× (0.66–0.66) | 7.92× (7.90–7.94) | 0.65× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.70× (0.68–0.73) | 0.76× (0.72–0.81) | 1.69× (1.63–1.75) | 0.85× (0.82–0.89) | 6.12× (5.91–6.33) | 0.50× (0.48–0.51) | 0.52× (0.50–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.68–0.72) | 0.78× (0.72–0.85) | 1.27× (1.23–1.30) | 0.79× (0.76–0.82) | 3.74× (3.65–3.83) | 0.29× (0.29–0.30) | 0.31× (0.30–0.31) | 0.34× (0.33–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.14× (5.14–5.15) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.39× (0.35–0.42) | 0.91× (0.89–0.93) | 0.74× (0.72–0.75) | 3.14× (3.06–3.21) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.87–0.89) | 0.82× (0.80–0.84) | 2.56× (2.52–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.84) | 5.21× (5.19–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.39× (0.37–0.41) | 0.86× (0.84–0.88) | 0.72× (0.69–0.74) | 2.96× (2.89–3.02) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.59× (0.58–0.60) | 0.48× (0.45–0.51) | 0.78× (0.77–0.79) | 0.78× (0.76–0.81) | 2.26× (2.23–2.28) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.30× (5.28–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.25) | 1.86× (1.86–1.87) | 0.90× (0.89–0.90) | 6.16× (6.14–6.18) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.86× (1.86–1.87) | 0.90× (0.90–0.91) | 6.18× (6.16–6.19) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.66× (0.66–0.66) | 6.37× (6.35–6.38) | 0.38× (0.38–0.39) | 0.41× (0.41–0.41) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.39× (0.36–0.43) | 1.02× (1.00–1.05) | 0.69× (0.67–0.71) | 3.38× (3.32–3.45) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.33–0.34) | 0.16× (0.16–0.17) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 58 MB | 0.47× (0.46–0.49) | 0.45× (0.41–0.49) | 0.88× (0.86–0.89) | 0.71× (0.68–0.73) | 2.23× (2.19–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 437 | 2.30 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 786 | 2.46 | 1.06× (1.03–1.09) | 89% | 1.02× (1.02–1.03) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 953 | 3.91 | 1.67× (1.63–1.72) | 55% | 1.21× (1.20–1.22) | 1.65× (1.63–1.67) |

**Best:** mibz2 at 4 threads (953 MB/s); lbzip2 at 4 threads (192 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.02, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.09 | 0.220 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.2% |
| MTF + RLE2 | 1.54 | 17.6% |
| T-vector build | 1.51 | 17.3% |
| walk (the one each build uses) | 4.34 | 49.7% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.73 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.509 | 1.302 | (in its total) | 2.56× (2.55–2.57) |  |
| walk | 1.260 | 2.668 | (in its total) | 2.11× (2.10–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.055 | (in its total) | 1.20× (1.18–1.21) |  |
| inverse BWT, total | 1.816 | 4.027 | 3.291 | 2.21× (2.20–2.23) | 1.81× (1.80–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m53s |
| 4 threads | 31m58s ± 4m30s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m30s ± 3m19s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 23m30s ± 3m19s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 3.08 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,277,600 | 15,175,604 | \`e0c1d543c056b938…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
