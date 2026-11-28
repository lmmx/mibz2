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

The latest record: [`8fecbab`](<https://github.com/lmmx/mibz2/commit/8fecbabb51d7cd49e8ebfeea59e55a24139dfb06>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46146356122>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.476 s | 440 | 7.4 | 3.17× (3.15–3.18) | 7.18× (7.14–7.21) | 7.39× (7.34–7.43) | 8.05× (8.00–8.10) | 7.22× (7.19–7.26) | 7.90× (7.86–7.94) | 7.28× (7.25–7.32) | 2.33× (2.31–2.34) | 2.72× (2.70–2.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.269 s | 778 | 7.9 | 3.10× (2.97–3.24) | 6.63× (6.34–6.93) | 7.56× (7.21–7.93) | 7.55× (7.21–7.90) | 6.76× (6.49–7.05) | 13.82× (13.26–14.40) | 12.74× (12.24–13.26) | 4.07× (3.90–4.25) | 4.76× (4.56–4.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.218 s | 963 | 12.5 | 3.21× (3.17–3.26) | 4.80× (4.72–4.88) | 6.05× (5.93–6.17) | 5.51× (5.44–5.59) | 5.53× (5.43–5.63) | 17.23× (16.98–17.49) | 15.89× (15.65–16.13) | 5.08× (4.99–5.16) | 5.94× (5.83–6.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 995 | 3.6 | 0.60× (0.59–0.61) | 3.73× (3.54–3.93) | 4.47× (4.41–4.54) | 3.59× (3.54–3.64) | 6.19× (6.07–6.30) | 3.00× (2.96–3.05) | 2.68× (2.64–2.72) | 1.01× (0.99–1.04) | 1.52× (1.50–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 924 | 4.0 | 0.54× (0.53–0.55) | 2.86× (2.82–2.90) | 5.63× (5.55–5.71) | 3.39× (3.35–3.44) | 5.80× (5.72–5.87) | 2.82× (2.78–2.86) | 2.52× (2.48–2.55) | 0.95× (0.92–0.98) | 1.43× (1.41–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 915 | 4.0 | 0.54× (0.54–0.55) | 2.89× (2.77–3.01) | 5.63× (5.49–5.78) | 3.36× (3.32–3.40) | 5.76× (5.65–5.88) | 2.79× (2.76–2.83) | 2.49× (2.46–2.52) | 0.94× (0.92–0.96) | 1.42× (1.40–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 611 | 5.6 | 2.73× (2.70–2.76) | 3.16× (3.13–3.19) | 4.06× (4.02–4.10) | 3.99× (3.95–4.03) | 4.30× (4.25–4.35) | 2.67× (2.63–2.71) | 2.20× (2.18–2.22) | 2.09× (2.07–2.11) | 3.15× (3.11–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 966 | 6.8 | 2.39× (2.10–2.72) | 2.90× (2.49–3.39) | 4.16× (3.80–4.55) | 3.64× (2.96–4.48) | 4.84× (4.23–5.53) | 3.93× (3.42–4.51) | 3.24× (2.83–3.70) | 3.08× (2.68–3.54) | 4.63× (4.04–5.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 863 | 13.1 | 2.04× (1.96–2.12) | 2.13× (1.96–2.33) | 3.22× (3.06–3.38) | 2.41× (2.27–2.57) | 4.27× (4.12–4.42) | 3.78× (3.63–3.93) | 3.12× (3.00–3.23) | 2.96× (2.86–3.07) | 4.45× (4.29–4.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.121 s | 66 | 50.0 | 1.44× (1.42–1.46) | 2.84× (2.80–2.88) | 2.59× (2.54–2.65) | 2.66× (2.62–2.70) | 3.03× (2.99–3.07) | 3.33× (3.28–3.38) | 3.33× (3.28–3.38) | 1.30× (1.28–1.32) | 1.49× (1.47–1.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 s | 110 | 55.3 | 1.49× (1.31–1.70) | 2.86× (2.70–3.02) | 2.62× (2.54–2.71) | 3.39× (3.31–3.48) | 3.22× (3.18–3.27) | 5.58× (5.52–5.65) | 5.58× (5.49–5.66) | 2.18× (2.15–2.21) | 2.50× (2.47–2.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.063 s | 128 | 77.2 | 1.42× (1.39–1.46) | 2.32× (2.24–2.42) | 2.28× (2.15–2.42) | 2.92× (2.76–3.10) | 3.07× (2.99–3.14) | 6.55× (6.39–6.71) | 6.54× (6.38–6.71) | 2.55× (2.49–2.62) | 2.93× (2.86–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.135 s | 71 | 46.3 | 1.56× (1.55–1.58) | 3.24× (3.21–3.27) | 2.71× (2.67–2.75) | 2.83× (2.82–2.85) | 3.27× (3.26–3.29) | 3.85× (3.81–3.88) | 4.09× (4.06–4.12) | 1.50× (1.49–1.50) | 1.81× (1.80–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.081 s | 119 | 52.3 | 1.45× (1.42–1.48) | 2.97× (2.91–3.04) | 2.71× (2.57–2.86) | 3.70× (3.62–3.78) | 3.42× (3.21–3.64) | 6.43× (6.31–6.54) | 6.84× (6.71–6.97) | 2.50× (2.46–2.54) | 3.03× (2.97–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 74.9 | 1.51× (1.46–1.56) | 2.51× (2.38–2.64) | 2.41× (2.30–2.52) | 3.22× (3.12–3.32) | 3.00× (2.91–3.09) | 7.35× (7.14–7.55) | 7.82× (7.57–8.07) | 2.86× (2.77–2.95) | 3.46× (3.36–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 752 | 5.0 | 0.67× (0.67–0.68) | 3.16× (3.00–3.32) | 4.53× (4.48–4.58) | 3.16× (3.13–3.19) | 6.88× (6.80–6.96) | 2.58× (2.55–2.60) | 2.36× (2.34–2.38) | 0.99× (0.98–1.00) | 1.56× (1.54–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 685 | 5.7 | 0.54× (0.53–0.55) | 2.78× (2.73–2.84) | 4.78× (4.68–4.88) | 2.88× (2.82–2.94) | 6.19× (6.02–6.37) | 2.32× (2.27–2.37) | 2.12× (2.08–2.17) | 0.89× (0.87–0.92) | 1.40× (1.37–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 673 | 5.6 | 0.55× (0.53–0.57) | 2.79× (2.76–2.82) | 4.77× (4.72–4.82) | 2.88× (2.85–2.91) | 6.15× (6.08–6.22) | 2.31× (2.29–2.34) | 2.12× (2.10–2.14) | 0.89× (0.88–0.91) | 1.40× (1.38–1.41) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 0.063 s | 238 | 14.0 | 2.25× (2.18–2.31) | 4.34× (4.21–4.47) | 4.74× (4.61–4.88) | 4.84× (4.71–4.97) | 5.00× (4.87–5.14) | 5.43× (5.28–5.58) | 4.91× (4.78–5.05) | 1.68× (1.63–1.73) | 2.11× (2.04–2.19) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 0.045 s | 333 | 18.1 | 1.59× (1.39–1.81) | 3.17× (2.68–3.76) | 3.77× (3.27–4.35) | 4.43× (3.78–5.18) | 3.92× (3.42–4.49) | 6.92× (6.03–7.94) | 6.26× (5.45–7.18) | 2.14× (1.87–2.44) | 2.69× (2.34–3.09) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 0.039 s | 384 | 27.8 | 1.70× (1.62–1.79) | 2.65× (2.51–2.80) | 3.59× (3.42–3.77) | 3.69× (3.55–3.83) | 3.91× (3.78–4.05) | 8.63× (8.34–8.93) | 7.81× (7.54–8.08) | 2.67× (2.58–2.76) | 3.36× (3.24–3.49) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.16× (3.14–3.17) | 7.14× (7.11–7.18) | 7.35× (7.30–7.40) | 8.02× (7.97–8.07) | 7.20× (7.16–7.23) | 7.87× (7.83–7.91) | 7.25× (7.22–7.29) | 2.32× (2.31–2.33) | 2.71× (2.69–2.72) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.07–3.09) | 6.91× (6.81–7.01) | 7.25× (7.15–7.35) | 7.95× (7.93–7.98) | 6.83× (6.82–6.85) | 7.40× (7.36–7.43) | 6.82× (6.79–6.85) | 2.18× (2.17–2.19) | 2.55× (2.53–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.03× (2.98–3.08) | 5.12× (4.97–5.28) | 5.55× (5.39–5.72) | 5.88× (5.70–6.08) | 5.18× (5.02–5.35) | 4.72× (4.58–4.85) | 4.35× (4.23–4.47) | 1.39× (1.35–1.43) | 1.62× (1.58–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.30× (3.23–3.36) | 4.09× (4.02–4.15) | 3.37× (3.32–3.41) | 5.74× (5.64–5.84) | 2.83× (2.79–2.88) | 2.54× (2.50–2.58) | 1.01× (0.98–1.04) | 1.07× (1.02–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.88× (2.84–2.92) | 5.01× (4.95–5.06) | 3.10× (3.06–3.14) | 5.20× (5.14–5.26) | 2.57× (2.53–2.61) | 2.30× (2.26–2.33) | 0.92× (0.89–0.94) | 0.97× (0.93–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.21× (3.08–3.34) | 5.05× (4.95–5.14) | 3.11× (3.07–3.15) | 5.24× (5.13–5.34) | 2.59× (2.55–2.62) | 2.31× (2.29–2.34) | 0.92× (0.90–0.94) | 0.97× (0.94–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.60–2.65) | 2.90× (2.87–2.93) | 3.62× (3.58–3.66) | 3.82× (3.78–3.85) | 4.11× (4.07–4.16) | 2.57× (2.54–2.61) | 2.13× (2.11–2.15) | 2.03× (2.01–2.05) | 2.82× (2.79–2.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.39× (2.36–2.43) | 2.67× (2.63–2.71) | 3.23× (3.18–3.28) | 3.38× (3.33–3.43) | 3.57× (3.52–3.62) | 2.15× (2.10–2.19) | 1.78× (1.75–1.81) | 1.69× (1.66–1.73) | 2.35× (2.32–2.39) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (1.98–2.24) | 1.89× (1.81–1.97) | 2.37× (2.26–2.48) | 2.20× (2.09–2.33) | 2.25× (2.15–2.34) | 1.12× (1.07–1.16) | 0.92× (0.89–0.96) | 0.88× (0.84–0.92) | 1.22× (1.17–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.42–1.46) | 2.80× (2.76–2.84) | 2.54× (2.49–2.60) | 2.63× (2.60–2.67) | 3.00× (2.96–3.04) | 3.30× (3.25–3.34) | 3.29× (3.25–3.34) | 1.29× (1.28–1.31) | 1.46× (1.44–1.48) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.38× (1.37–1.39) | 2.70× (2.67–2.73) | 2.37× (2.35–2.39) | 3.18× (3.16–3.20) | 2.79× (2.77–2.81) | 2.99× (2.97–3.01) | 2.99× (2.97–3.01) | 1.17× (1.17–1.18) | 1.33× (1.31–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.55–1.59) | 2.42× (2.38–2.46) | 1.98× (1.95–2.02) | 2.70× (2.67–2.74) | 2.57× (2.55–2.60) | 2.16× (2.14–2.18) | 2.15× (2.14–2.17) | 0.85× (0.84–0.86) | 0.96× (0.94–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.55× (1.54–1.56) | 3.18× (3.15–3.20) | 2.66× (2.63–2.69) | 2.81× (2.79–2.82) | 3.24× (3.23–3.26) | 3.81× (3.77–3.84) | 4.05× (4.02–4.08) | 1.49× (1.48–1.50) | 1.76× (1.75–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.45× (1.43–1.46) | 2.95× (2.92–2.98) | 2.45× (2.43–2.48) | 3.45× (3.42–3.49) | 3.01× (2.84–3.19) | 3.38× (3.35–3.41) | 3.59× (3.56–3.62) | 1.32× (1.31–1.34) | 1.56× (1.54–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.58–1.73) | 2.63× (2.53–2.72) | 2.08× (2.03–2.13) | 2.95× (2.86–3.04) | 2.68× (2.61–2.76) | 2.38× (2.32–2.44) | 2.53× (2.47–2.59) | 0.93× (0.91–0.96) | 1.10× (1.07–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.76× (2.72–2.79) | 3.62× (3.58–3.65) | 2.89× (2.85–2.93) | 6.14× (6.06–6.23) | 2.38× (2.35–2.41) | 2.19× (2.17–2.22) | 1.00× (0.99–1.01) | 0.99× (0.94–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.36× (2.32–2.40) | 4.00× (3.93–4.07) | 2.60× (2.56–2.64) | 5.34× (5.21–5.48) | 2.06× (2.02–2.10) | 1.90× (1.86–1.93) | 0.86× (0.84–0.88) | 0.86× (0.81–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.60× (2.56–2.64) | 4.01× (3.92–4.09) | 2.65× (2.61–2.69) | 5.39× (5.32–5.47) | 2.09× (2.06–2.12) | 1.92× (1.89–1.95) | 0.87× (0.86–0.89) | 0.87× (0.83–0.92) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 2.21× (2.15–2.27) | 4.20× (4.08–4.31) | 4.58× (4.45–4.71) | 4.72× (4.60–4.85) | 4.88× (4.75–5.01) | 5.29× (5.15–5.44) | 4.79× (4.66–4.93) | 1.66× (1.61–1.70) | 2.03× (1.97–2.09) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 1.82× (1.79–1.85) | 3.44× (3.38–3.51) | 3.77× (3.69–3.86) | 4.22× (4.11–4.33) | 3.89× (3.80–3.98) | 4.14× (4.05–4.24) | 3.75× (3.66–3.84) | 1.30× (1.27–1.32) | 1.59× (1.55–1.63) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 1.84× (1.78–1.89) | 2.73× (2.70–2.76) | 2.99× (2.95–3.04) | 3.26× (3.22–3.30) | 3.08× (3.04–3.11) | 2.69× (2.67–2.72) | 2.44× (2.41–2.46) | 0.84× (0.83–0.85) | 1.03× (1.02–1.05) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.98–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.35× (0.34–0.36) | 0.51× (0.43–0.61) | 0.74× (0.72–0.75) | 0.62× (0.60–0.64) | 5.82× (5.66–5.99) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.36× (0.35–0.38) | 0.55× (0.48–0.63) | 0.65× (0.62–0.68) | 0.58× (0.55–0.60) | 3.79× (3.63–3.96) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.96× (1.95–1.96) | 0.93× (0.93–0.94) | 4.85× (4.83–4.86) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.67) | 7.95× (7.93–7.97) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.70–0.73) | 0.78× (0.72–0.85) | 1.71× (1.68–1.74) | 0.86× (0.84–0.88) | 6.18× (6.08–6.29) | 0.50× (0.49–0.51) | 0.52× (0.51–0.54) | 0.57× (0.56–0.58) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.65× (0.62–0.69) | 0.72× (0.65–0.80) | 1.21× (1.14–1.28) | 0.74× (0.70–0.79) | 3.54× (3.33–3.77) | 0.28× (0.26–0.30) | 0.29× (0.27–0.31) | 0.32× (0.30–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.14× (5.13–5.16) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.52) | 0.41× (0.38–0.45) | 0.91× (0.88–0.94) | 0.73× (0.70–0.76) | 3.14× (3.04–3.24) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.66) | 0.46× (0.45–0.47) | 0.88× (0.87–0.89) | 0.82× (0.80–0.84) | 2.58× (2.55–2.61) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.21× (5.19–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.47) | 0.36× (0.33–0.39) | 0.83× (0.80–0.85) | 0.69× (0.67–0.72) | 2.84× (2.75–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.48× (0.45–0.51) | 0.80× (0.79–0.81) | 0.79× (0.77–0.81) | 2.31× (2.27–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.29× (5.28–5.29) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.91× (0.90–0.91) | 6.19× (6.16–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.23) | 1.87× (1.86–1.87) | 0.91× (0.90–0.91) | 6.18× (6.16–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.38) | 1.39× (1.38–1.39) | 0.66× (0.66–0.66) | 6.36× (6.35–6.38) | 0.38× (0.38–0.39) | 0.41× (0.41–0.41) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 35 MB | 0.47× (0.46–0.48) | 0.45× (0.40–0.49) | 1.07× (1.04–1.10) | 0.71× (0.69–0.73) | 3.54× (3.46–3.63) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.36) | 0.17× (0.16–0.17) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 56 MB | 0.51× (0.49–0.52) | 0.49× (0.46–0.52) | 0.91× (0.89–0.93) | 0.76× (0.73–0.79) | 2.34× (2.28–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 440 | 2.28 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 778 | 2.44 | 1.06× (1.06–1.07) | 87% | 1.03× (1.01–1.04) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 963 | 3.87 | 1.67× (1.62–1.71) | 55% | 1.20× (1.19–1.20) | 1.60× (1.57–1.63) |

**Best:** mibz2 at 4 threads (963 MB/s); lbzip2 at 4 threads (199 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.01, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.49, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.08 | 0.218 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.4% |
| MTF + RLE2 | 1.53 | 17.9% |
| T-vector build | 1.51 | 17.6% |
| walk (the one each build uses) | 4.18 | 48.9% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.56 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.510 | 1.299 | (in its total) | 2.55× (2.54–2.55) |  |
| walk | 1.239 | 2.647 | (in its total) | 2.13× (2.12–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.057 | (in its total) | 1.21× (1.19–1.24) |  |
| inverse BWT, total | 1.796 | 4.003 | 3.287 | 2.23× (2.22–2.24) | 1.83× (1.82–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m28s |
| 4 threads | 31m53s ± 4m30s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m57s ± 3m22s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 22m57s ± 3m22s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 3.01 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,269,316 | 15,087,510 | \`c5dc29887a3b9d42…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
