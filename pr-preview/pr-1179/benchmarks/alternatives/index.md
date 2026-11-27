# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1179/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`37dc02b`](<https://github.com/lmmx/mibz2/commit/37dc02bc5b15cc393d5b44b59c630e2d7c3dc332>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46043702198>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.480 s | 437 | 7.5 | 3.09× (2.98–3.20) | 7.05× (6.82–7.29) | 7.23× (7.00–7.47) | 7.85× (7.60–8.12) | 7.09× (6.87–7.32) | 7.74× (7.49–8.01) | 7.22× (6.98–7.47) | 2.29× (2.22–2.38) | 2.64× (2.56–2.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.273 s | 768 | 8.0 | 3.08× (2.77–3.42) | 6.59× (6.04–7.20) | 7.58× (6.92–8.29) | 7.37× (6.77–8.03) | 6.61× (6.08–7.19) | 13.38× (12.30–14.56) | 12.48× (11.46–13.58) | 3.97× (3.64–4.32) | 4.56× (4.21–4.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.223 s | 942 | 12.6 | 3.22× (3.15–3.28) | 4.82× (4.75–4.89) | 6.09× (5.95–6.24) | 5.51× (5.43–5.60) | 5.49× (5.42–5.56) | 16.96× (16.77–17.15) | 15.81× (15.63–15.99) | 5.02× (4.96–5.09) | 5.78× (5.73–5.82) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 976 | 3.6 | 0.60× (0.60–0.61) | 3.68× (3.49–3.88) | 4.43× (4.39–4.46) | 3.57× (3.54–3.60) | 6.18× (6.13–6.24) | 2.98× (2.96–3.00) | 2.66× (2.65–2.68) | 1.02× (1.00–1.03) | 1.51× (1.49–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 920 | 4.0 | 0.55× (0.54–0.57) | 2.85× (2.79–2.90) | 5.71× (5.58–5.85) | 3.40× (3.35–3.46) | 5.80× (5.72–5.88) | 2.81× (2.76–2.86) | 2.52× (2.48–2.56) | 0.96× (0.94–0.99) | 1.42× (1.40–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 909 | 4.0 | 0.55× (0.54–0.55) | 2.81× (2.77–2.86) | 5.66× (5.49–5.84) | 3.35× (3.29–3.40) | 5.77× (5.67–5.87) | 2.77× (2.73–2.81) | 2.48× (2.44–2.52) | 0.95× (0.93–0.97) | 1.40× (1.38–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 603 | 5.7 | 2.74× (2.71–2.77) | 3.13× (3.09–3.17) | 4.03× (3.98–4.07) | 3.93× (3.88–3.98) | 4.27× (4.22–4.33) | 2.64× (2.61–2.67) | 2.19× (2.16–2.21) | 2.15× (2.11–2.19) | 3.12× (3.09–3.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 927 | 6.9 | 2.42× (2.16–2.71) | 3.00× (2.79–3.21) | 4.23× (3.94–4.55) | 3.56× (3.35–3.78) | 5.00× (4.75–5.28) | 3.98× (3.80–4.18) | 3.30× (3.13–3.47) | 3.25× (3.08–3.43) | 4.71× (4.49–4.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 877 | 13.6 | 2.03× (1.96–2.11) | 2.14× (1.97–2.32) | 3.26× (3.07–3.46) | 2.41× (2.34–2.49) | 4.30× (4.15–4.46) | 3.77× (3.65–3.89) | 3.12× (3.02–3.22) | 3.07× (2.98–3.17) | 4.46× (4.33–4.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 50.6 | 1.44× (1.42–1.46) | 2.89× (2.85–2.92) | 2.57× (2.53–2.61) | 2.60× (2.57–2.63) | 3.02× (2.99–3.06) | 3.31× (3.28–3.35) | 3.37× (3.33–3.41) | 1.30× (1.29–1.32) | 1.50× (1.47–1.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 109 | 56.3 | 1.41× (1.39–1.43) | 2.87× (2.65–3.11) | 2.63× (2.60–2.67) | 3.40× (3.31–3.49) | 3.23× (3.18–3.29) | 5.57× (5.49–5.65) | 5.66× (5.58–5.75) | 2.19× (2.16–2.22) | 2.51× (2.47–2.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 129 | 77.2 | 1.46× (1.43–1.49) | 2.34× (2.27–2.41) | 2.27× (2.19–2.36) | 2.78× (2.72–2.85) | 3.13× (3.05–3.21) | 6.53× (6.40–6.66) | 6.64× (6.52–6.77) | 2.57× (2.52–2.62) | 2.95× (2.87–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.136 s | 71 | 46.6 | 1.57× (1.56–1.58) | 3.22× (3.20–3.24) | 2.70× (2.67–2.73) | 2.79× (2.77–2.81) | 3.28× (3.25–3.31) | 3.85× (3.82–3.88) | 4.21× (4.17–4.25) | 1.49× (1.48–1.51) | 1.80× (1.78–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 53.8 | 1.40× (1.31–1.49) | 2.91× (2.81–3.02) | 2.55× (2.41–2.71) | 3.56× (3.31–3.84) | 3.16× (2.96–3.36) | 6.13× (5.73–6.56) | 6.70× (6.26–7.18) | 2.37× (2.22–2.54) | 2.87× (2.68–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 75.5 | 1.56× (1.52–1.61) | 2.43× (2.37–2.49) | 2.42× (2.35–2.49) | 3.27× (3.19–3.35) | 3.06× (3.02–3.11) | 7.48× (7.38–7.59) | 8.18× (8.06–8.30) | 2.90× (2.86–2.94) | 3.51× (3.45–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 755 | 5.0 | 0.68× (0.67–0.68) | 3.09× (3.05–3.12) | 4.53× (4.47–4.59) | 3.17× (3.13–3.21) | 6.91× (6.85–6.97) | 2.58× (2.55–2.62) | 2.38× (2.36–2.41) | 1.00× (0.99–1.01) | 1.55× (1.53–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.8 | 0.55× (0.54–0.56) | 2.78× (2.75–2.82) | 4.78× (4.72–4.84) | 2.89× (2.86–2.93) | 6.21× (6.13–6.29) | 2.32× (2.29–2.35) | 2.14× (2.12–2.17) | 0.90× (0.89–0.91) | 1.40× (1.38–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 656 | 5.8 | 0.56× (0.52–0.59) | 2.71× (2.67–2.75) | 4.64× (4.57–4.71) | 2.82× (2.78–2.86) | 6.05× (5.93–6.17) | 2.25× (2.22–2.29) | 2.08× (2.04–2.11) | 0.87× (0.86–0.89) | 1.35× (1.33–1.37) | yes |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 1 | 0.060 s | 236 | 14.2 | 2.20× (2.17–2.23) | 4.36× (4.28–4.43) | 4.82× (4.74–4.91) | 4.77× (4.69–4.85) | 5.00× (4.92–5.09) | 5.38× (5.28–5.48) | 5.07× (4.97–5.16) | 1.68× (1.67–1.70) | 2.16× (2.10–2.21) | yes |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 2 | 0.045 s | 317 | 19.1 | 1.70× (1.63–1.78) | 3.50× (3.12–3.92) | 3.87× (3.72–4.03) | 4.33× (3.85–4.88) | 4.34× (4.18–4.51) | 7.26× (6.99–7.54) | 6.84× (6.58–7.11) | 2.27× (2.19–2.36) | 2.91× (2.80–3.02) | yes |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 4 | 0.039 s | 368 | 29.2 | 1.69× (1.61–1.78) | 2.57× (2.51–2.62) | 3.35× (3.19–3.52) | 3.44× (3.35–3.54) | 4.01× (3.92–4.11) | 8.37× (8.19–8.56) | 7.89× (7.72–8.06) | 2.62× (2.56–2.69) | 3.36× (3.25–3.46) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.08× (2.97–3.19) | 7.02× (6.80–7.26) | 7.20× (6.97–7.44) | 7.82× (7.57–8.09) | 7.06× (6.84–7.29) | 7.71× (7.46–7.98) | 7.19× (6.95–7.44) | 2.29× (2.21–2.37) | 2.62× (2.55–2.69) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (2.83–3.29) | 6.80× (6.45–7.16) | 7.12× (6.73–7.53) | 7.76× (7.37–8.17) | 6.67× (6.34–7.01) | 7.16× (6.80–7.54) | 6.68× (6.34–7.04) | 2.13× (2.02–2.24) | 2.43× (2.32–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (3.01–3.12) | 5.17× (5.10–5.25) | 5.55× (5.46–5.65) | 5.89× (5.80–5.97) | 5.16× (5.08–5.24) | 4.66× (4.59–4.72) | 4.35× (4.29–4.40) | 1.38× (1.36–1.40) | 1.58× (1.55–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.64) | 3.25× (3.22–3.29) | 4.06× (4.02–4.09) | 3.34× (3.31–3.37) | 5.72× (5.67–5.78) | 2.80× (2.78–2.82) | 2.51× (2.50–2.53) | 1.02× (1.00–1.03) | 1.06× (1.05–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.88× (2.83–2.93) | 5.06× (4.98–5.15) | 3.11× (3.05–3.16) | 5.21× (5.14–5.28) | 2.57× (2.52–2.61) | 2.30× (2.26–2.35) | 0.93× (0.91–0.96) | 0.98× (0.96–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.20× (3.11–3.28) | 5.08× (4.91–5.25) | 3.10× (3.04–3.16) | 5.23× (5.12–5.33) | 2.56× (2.51–2.60) | 2.29× (2.25–2.34) | 0.93× (0.91–0.95) | 0.97× (0.95–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.61–2.67) | 2.88× (2.85–2.91) | 3.61× (3.56–3.65) | 3.76× (3.71–3.81) | 4.08× (4.03–4.13) | 2.54× (2.51–2.57) | 2.12× (2.09–2.14) | 2.08× (2.04–2.13) | 2.78× (2.74–2.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.35–2.39) | 2.64× (2.57–2.71) | 3.23× (3.19–3.28) | 3.30× (3.26–3.33) | 3.56× (3.50–3.61) | 2.10× (2.07–2.13) | 1.75× (1.73–1.77) | 1.72× (1.69–1.76) | 2.30× (2.27–2.33) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.04–2.23) | 1.84× (1.76–1.92) | 2.32× (2.23–2.42) | 2.26× (2.18–2.34) | 2.21× (2.13–2.30) | 1.09× (1.05–1.13) | 0.91× (0.87–0.94) | 0.89× (0.86–0.93) | 1.19× (1.14–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.43× (1.42–1.45) | 2.79× (2.76–2.82) | 2.52× (2.48–2.55) | 2.57× (2.54–2.60) | 2.99× (2.96–3.03) | 3.28× (3.24–3.32) | 3.34× (3.30–3.37) | 1.30× (1.28–1.32) | 1.45× (1.43–1.47) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.37× (1.36–1.39) | 2.67× (2.64–2.70) | 2.35× (2.33–2.37) | 3.13× (3.08–3.18) | 2.78× (2.75–2.81) | 2.96× (2.93–2.99) | 3.01× (2.98–3.04) | 1.17× (1.16–1.19) | 1.31× (1.29–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.52–1.65) | 2.43× (2.39–2.47) | 1.98× (1.93–2.02) | 2.69× (2.64–2.74) | 2.58× (2.53–2.63) | 2.14× (2.10–2.19) | 2.18× (2.14–2.22) | 0.85× (0.83–0.87) | 0.95× (0.93–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.56× (1.55–1.57) | 3.17× (3.15–3.19) | 2.66× (2.63–2.69) | 2.76× (2.74–2.78) | 3.24× (3.21–3.27) | 3.81× (3.78–3.84) | 4.16× (4.12–4.20) | 1.48× (1.47–1.50) | 1.76× (1.74–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.42–1.45) | 2.89× (2.85–2.94) | 2.41× (2.38–2.45) | 3.36× (3.30–3.41) | 2.86× (2.83–2.90) | 3.31× (3.26–3.36) | 3.62× (3.56–3.67) | 1.29× (1.27–1.31) | 1.53× (1.50–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.71× (1.68–1.73) | 2.66× (2.63–2.70) | 2.03× (2.00–2.06) | 2.91× (2.84–2.98) | 2.65× (2.62–2.68) | 2.35× (2.32–2.38) | 2.57× (2.53–2.60) | 0.91× (0.90–0.93) | 1.08× (1.07–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.75× (2.71–2.80) | 3.59× (3.53–3.65) | 2.89× (2.84–2.94) | 6.14× (6.06–6.23) | 2.38× (2.34–2.42) | 2.20× (2.17–2.23) | 1.00× (0.98–1.02) | 0.98× (0.95–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.36× (2.33–2.40) | 4.01× (3.96–4.07) | 2.60× (2.57–2.64) | 5.34× (5.26–5.42) | 2.06× (2.03–2.09) | 1.90× (1.88–1.93) | 0.87× (0.85–0.88) | 0.84× (0.82–0.87) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.55–0.62) | 2.56× (2.51–2.61) | 3.97× (3.91–4.04) | 2.60× (2.56–2.64) | 5.33× (5.23–5.42) | 2.05× (2.02–2.08) | 1.89× (1.86–1.92) | 0.86× (0.85–0.88) | 0.84× (0.82–0.86) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 1 | 2.16× (2.13–2.19) | 4.17× (4.10–4.24) | 4.57× (4.50–4.65) | 4.65× (4.57–4.73) | 4.88× (4.79–4.96) | 5.24× (5.14–5.33) | 4.94× (4.85–5.03) | 1.66× (1.65–1.68) | 1.99× (1.96–2.02) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 2 | 1.72× (1.69–1.75) | 3.29× (3.23–3.34) | 3.63× (3.57–3.70) | 3.98× (3.92–4.05) | 3.75× (3.69–3.82) | 3.90× (3.82–3.98) | 3.68× (3.60–3.75) | 1.24× (1.22–1.26) | 1.48× (1.45–1.51) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 4 | 1.83× (1.75–1.92) | 2.69× (2.62–2.76) | 2.96× (2.85–3.07) | 3.13× (3.05–3.21) | 2.98× (2.91–3.07) | 2.54× (2.49–2.59) | 2.39× (2.35–2.43) | 0.81× (0.79–0.82) | 0.97× (0.95–0.98) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.28–0.31) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.07× (11.04–11.09) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.33× (0.33–0.34) | 0.52× (0.43–0.64) | 0.70× (0.69–0.72) | 0.58× (0.57–0.59) | 5.56× (5.45–5.68) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.38) | 0.72× (0.63–0.83) | 0.64× (0.61–0.68) | 0.56× (0.54–0.59) | 3.72× (3.54–3.91) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.28× (4.27–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.94) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.70) | 0.66× (0.66–0.66) | 8.01× (8.00–8.03) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.68–0.71) | 0.75× (0.70–0.82) | 1.67× (1.64–1.71) | 0.85× (0.82–0.87) | 6.06× (5.92–6.21) | 0.49× (0.48–0.50) | 0.51× (0.50–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.68× (0.65–0.72) | 0.74× (0.69–0.79) | 1.24× (1.18–1.32) | 0.76× (0.73–0.80) | 3.62× (3.46–3.80) | 0.28× (0.27–0.30) | 0.30× (0.28–0.31) | 0.32× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.74× (0.73–0.74) | 5.18× (5.16–5.20) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.40× (0.36–0.43) | 0.89× (0.86–0.92) | 0.71× (0.69–0.74) | 3.08× (2.97–3.18) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.47× (0.45–0.49) | 0.88× (0.86–0.89) | 0.82× (0.80–0.85) | 2.55× (2.51–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.21× (5.20–5.23) | 0.32× (0.32–0.33) | 0.34× (0.33–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.35× (0.32–0.38) | 0.84× (0.81–0.86) | 0.71× (0.70–0.73) | 2.87× (2.78–2.97) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.51× (0.48–0.53) | 0.80× (0.78–0.82) | 0.80× (0.79–0.82) | 2.31× (2.26–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.20–6.23) | 0.41× (0.41–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.91× (0.90–0.91) | 6.21× (6.19–6.23) | 0.41× (0.41–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.41× (1.40–1.41) | 0.66× (0.65–0.66) | 6.42× (6.40–6.45) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.30–0.31) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 2 | 38 MB | 0.45× (0.44–0.47) | 0.42× (0.40–0.45) | 1.02× (0.99–1.06) | 0.69× (0.66–0.72) | 3.35× (3.27–3.44) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.12 MB → 14.3 MB, 6.7×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.48× (0.44–0.53) | 0.91× (0.89–0.93) | 0.75× (0.73–0.77) | 2.35× (2.29–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 437 | 2.30 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 768 | 2.47 | 1.08× (1.05–1.10) | 86% | 1.04× (1.04–1.05) | 1.07× (1.01–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 942 | 3.90 | 1.66× (1.58–1.73) | 55% | 1.22× (1.21–1.23) | 1.65× (1.62–1.67) |

**Best:** mibz2 at 4 threads (942 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.02, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 2%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.63 | 0.02 | 0.09 | 0.223 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3245 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 9.9% |
| MTF + RLE2 | 1.58 | 18.3% |
| T-vector build | 1.51 | 17.5% |
| walk (the one each build uses) | 4.15 | 48.1% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.517 | 1.299 | (in its total) | 2.51× (2.51–2.52) |  |
| walk | 1.222 | 2.632 | (in its total) | 2.15× (2.14–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.23× (1.22–1.25) |  |
| inverse BWT, total | 1.784 | 3.985 | 3.273 | 2.23× (2.22–2.24) | 1.83× (1.82–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m55s |
| 4 threads | 32m29s ± 4m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m10s ± 3m17s |
| the scanner alone (its serial part) | 10m02s ± 35.4s |
| the workers alone, over 4 | 23m10s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 5.8 ns a byte of bz2, the workers 3.04 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,119,930 | 14,285,640 | \`686c0f88fb2daec0…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
