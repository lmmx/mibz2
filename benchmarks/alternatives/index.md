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

The latest record: [`757a91b`](<https://github.com/lmmx/mibz2/commit/757a91b72eeaa127ad55a100a9a06bd28f3028f2>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46105280057>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.477 s | 439 | 7.5 | 3.17× (3.16–3.17) | 7.28× (7.27–7.30) | 7.41× (7.37–7.44) | 8.23× (8.21–8.25) | 7.34× (7.32–7.37) | 7.96× (7.90–8.02) | 7.46× (7.44–7.48) | 2.33× (2.33–2.34) | 2.73× (2.71–2.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.269 s | 781 | 8.0 | 3.20× (2.99–3.42) | 6.76× (6.41–7.13) | 7.52× (7.25–7.81) | 7.73× (7.44–8.02) | 6.84× (6.60–7.09) | 13.97× (13.52–14.44) | 13.10× (12.65–13.57) | 4.10× (3.95–4.24) | 4.79× (4.63–4.96) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.216 s | 969 | 12.6 | 3.21× (3.15–3.27) | 4.89× (4.82–4.97) | 6.00× (5.92–6.08) | 5.57× (5.50–5.64) | 5.58× (5.51–5.66) | 17.49× (17.16–17.81) | 16.39× (16.17–16.62) | 5.12× (5.06–5.19) | 6.00× (5.91–6.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 988 | 3.6 | 0.60× (0.59–0.60) | 3.78× (3.59–3.97) | 4.48× (4.44–4.52) | 3.58× (3.55–3.61) | 6.19× (6.16–6.22) | 3.01× (2.98–3.03) | 2.68× (2.65–2.70) | 1.01× (0.99–1.02) | 1.52× (1.51–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 924 | 4.0 | 0.55× (0.53–0.56) | 2.86× (2.82–2.90) | 5.65× (5.58–5.73) | 3.40× (3.35–3.45) | 5.82× (5.72–5.92) | 2.82× (2.79–2.86) | 2.52× (2.48–2.55) | 0.95× (0.93–0.96) | 1.43× (1.41–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 913 | 4.0 | 0.55× (0.54–0.56) | 2.85× (2.81–2.89) | 5.60× (5.52–5.69) | 3.37× (3.31–3.43) | 5.77× (5.69–5.86) | 2.80× (2.76–2.84) | 2.49× (2.45–2.53) | 0.94× (0.92–0.95) | 1.42× (1.40–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 611 | 5.7 | 2.73× (2.71–2.75) | 3.17× (3.15–3.19) | 4.07× (3.98–4.16) | 4.10× (4.08–4.13) | 4.31× (4.27–4.35) | 2.67× (2.65–2.68) | 2.19× (2.18–2.21) | 2.10× (2.08–2.11) | 3.16× (3.14–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 901 | 6.9 | 2.25× (2.03–2.49) | 2.76× (2.52–3.01) | 3.94× (3.48–4.47) | 3.48× (3.18–3.82) | 4.82× (4.43–5.25) | 3.83× (3.50–4.19) | 3.15× (2.88–3.45) | 3.01× (2.75–3.30) | 4.54× (4.15–4.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 875 | 13.4 | 2.08× (2.02–2.14) | 2.11× (1.95–2.28) | 3.29× (3.13–3.46) | 2.44× (2.34–2.54) | 4.33× (4.20–4.46) | 3.79× (3.69–3.89) | 3.12× (3.04–3.19) | 2.98× (2.91–3.05) | 4.49× (4.37–4.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.119 s | 67 | 49.2 | 1.49× (1.48–1.51) | 3.00× (2.99–3.02) | 2.66× (2.62–2.71) | 2.75× (2.73–2.77) | 3.16× (3.14–3.18) | 3.49× (3.46–3.51) | 3.52× (3.49–3.54) | 1.35× (1.34–1.35) | 1.54× (1.53–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 113 | 55.1 | 1.41× (1.32–1.51) | 2.76× (2.56–2.98) | 2.59× (2.43–2.76) | 3.38× (3.16–3.62) | 3.22× (3.03–3.43) | 5.64× (5.30–6.00) | 5.69× (5.35–6.05) | 2.18× (2.05–2.31) | 2.49× (2.34–2.64) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 133 | 75.2 | 1.48× (1.44–1.52) | 2.44× (2.40–2.48) | 2.29× (2.23–2.36) | 2.91× (2.86–2.97) | 3.20× (3.14–3.26) | 6.88× (6.76–7.01) | 6.95× (6.82–7.07) | 2.66× (2.62–2.70) | 3.04× (2.98–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.133 s | 72 | 45.8 | 1.59× (1.57–1.61) | 3.30× (3.26–3.34) | 2.77× (2.73–2.81) | 2.90× (2.88–2.93) | 3.33× (3.30–3.36) | 3.95× (3.91–4.00) | 4.26× (4.23–4.30) | 1.51× (1.50–1.52) | 1.82× (1.81–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 117 | 51.4 | 1.44× (1.38–1.50) | 2.96× (2.87–3.06) | 2.63× (2.51–2.76) | 3.74× (3.58–3.90) | 3.27× (3.20–3.35) | 6.41× (6.26–6.56) | 6.91× (6.73–7.08) | 2.45× (2.39–2.50) | 2.96× (2.88–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.067 s | 142 | 74.3 | 1.54× (1.51–1.58) | 2.57× (2.47–2.68) | 2.48× (2.41–2.55) | 3.39× (3.27–3.51) | 3.13× (3.06–3.19) | 7.74× (7.57–7.91) | 8.34× (8.16–8.53) | 2.95× (2.89–3.02) | 3.57× (3.50–3.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 760 | 4.9 | 0.68× (0.66–0.69) | 3.11× (3.07–3.15) | 4.57× (4.50–4.63) | 3.17× (3.13–3.22) | 6.94× (6.88–7.00) | 2.60× (2.56–2.63) | 2.39× (2.36–2.42) | 0.99× (0.98–1.01) | 1.57× (1.55–1.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 678 | 5.7 | 0.54× (0.53–0.55) | 2.80× (2.75–2.86) | 4.81× (4.72–4.91) | 2.89× (2.84–2.95) | 6.29× (6.17–6.40) | 2.33× (2.29–2.37) | 2.15× (2.11–2.19) | 0.89× (0.88–0.91) | 1.41× (1.38–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 669 | 5.7 | 0.54× (0.54–0.55) | 2.77× (2.74–2.80) | 4.75× (4.69–4.80) | 2.85× (2.82–2.89) | 6.14× (6.07–6.22) | 2.30× (2.27–2.33) | 2.12× (2.10–2.14) | 0.88× (0.87–0.89) | 1.39× (1.37–1.40) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.063 s | 238 | 14.1 | 2.25× (2.23–2.27) | 4.47× (4.39–4.56) | 4.80× (4.76–4.84) | 4.89× (4.86–4.93) | 5.06× (5.01–5.12) | 5.46× (5.41–5.52) | 5.17× (5.13–5.22) | 1.69× (1.67–1.70) | 2.09× (2.07–2.11) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.045 s | 334 | 18.1 | 1.80× (1.64–1.98) | 3.32× (3.00–3.67) | 4.05× (3.75–4.38) | 4.40× (4.07–4.76) | 4.17× (3.80–4.57) | 7.32× (6.68–8.02) | 6.93× (6.33–7.60) | 2.26× (2.07–2.48) | 2.80× (2.55–3.07) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.040 s | 375 | 29.3 | 1.66× (1.61–1.72) | 2.60× (2.49–2.71) | 3.57× (3.46–3.68) | 3.66× (3.58–3.73) | 3.90× (3.81–3.99) | 8.59× (8.40–8.79) | 8.14× (7.97–8.31) | 2.66× (2.60–2.71) | 3.28× (3.21–3.36) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.16× (3.15–3.16) | 7.25× (7.23–7.26) | 7.37× (7.34–7.41) | 8.20× (8.18–8.22) | 7.31× (7.29–7.34) | 7.93× (7.87–7.99) | 7.43× (7.41–7.46) | 2.33× (2.32–2.33) | 2.71× (2.70–2.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.06–3.08) | 6.89× (6.82–6.96) | 7.18× (7.15–7.21) | 8.12× (8.01–8.24) | 6.87× (6.84–6.90) | 7.44× (7.39–7.49) | 6.97× (6.95–7.01) | 2.18× (2.18–2.19) | 2.54× (2.53–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.03× (2.96–3.11) | 5.17× (5.07–5.27) | 5.52× (5.42–5.62) | 5.89× (5.78–5.99) | 5.18× (5.08–5.28) | 4.74× (4.69–4.80) | 4.45× (4.37–4.52) | 1.39× (1.37–1.42) | 1.62× (1.59–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.64) | 3.32× (3.28–3.36) | 4.13× (4.08–4.18) | 3.37× (3.35–3.40) | 5.78× (5.73–5.83) | 2.85× (2.82–2.88) | 2.54× (2.52–2.57) | 1.01× (0.99–1.02) | 1.06× (1.03–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.89× (2.85–2.92) | 5.04× (4.96–5.12) | 3.11× (3.07–3.15) | 5.24× (5.16–5.33) | 2.58× (2.55–2.61) | 2.30× (2.27–2.33) | 0.91× (0.89–0.93) | 0.96× (0.93–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.16× (3.12–3.20) | 5.04× (4.98–5.10) | 3.13× (3.08–3.18) | 5.24× (5.17–5.30) | 2.59× (2.55–2.62) | 2.31× (2.28–2.34) | 0.91× (0.90–0.93) | 0.96× (0.93–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.62–2.66) | 2.92× (2.90–2.94) | 3.69× (3.63–3.75) | 3.95× (3.92–3.98) | 4.14× (4.10–4.18) | 2.58× (2.56–2.60) | 2.13× (2.12–2.15) | 2.04× (2.02–2.06) | 2.83× (2.80–2.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.35–2.41) | 2.65× (2.61–2.69) | 3.23× (3.19–3.27) | 3.45× (3.41–3.49) | 3.61× (3.58–3.65) | 2.13× (2.11–2.16) | 1.76× (1.74–1.78) | 1.68× (1.66–1.70) | 2.34× (2.31–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.17× (2.11–2.23) | 1.84× (1.78–1.91) | 2.32× (2.27–2.37) | 2.16× (2.11–2.22) | 2.22× (2.16–2.27) | 1.09× (1.06–1.11) | 0.90× (0.88–0.92) | 0.86× (0.84–0.88) | 1.19× (1.16–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.47–1.50) | 2.92× (2.90–2.94) | 2.62× (2.58–2.65) | 2.73× (2.71–2.74) | 3.13× (3.10–3.15) | 3.45× (3.43–3.48) | 3.48× (3.46–3.51) | 1.34× (1.33–1.35) | 1.50× (1.49–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.38–1.40) | 2.72× (2.71–2.74) | 2.39× (2.38–2.41) | 3.22× (3.19–3.24) | 2.83× (2.81–2.86) | 3.08× (3.05–3.10) | 3.10× (3.08–3.13) | 1.20× (1.19–1.20) | 1.34× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.59× (1.54–1.63) | 2.53× (2.48–2.58) | 2.06× (2.00–2.11) | 2.83× (2.76–2.91) | 2.68× (2.62–2.74) | 2.28× (2.23–2.33) | 2.30× (2.25–2.35) | 0.89× (0.86–0.91) | 0.99× (0.97–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.58× (1.56–1.60) | 3.25× (3.22–3.28) | 2.72× (2.69–2.75) | 2.88× (2.85–2.90) | 3.30× (3.27–3.33) | 3.91× (3.87–3.96) | 4.22× (4.18–4.25) | 1.50× (1.49–1.52) | 1.78× (1.77–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.44–1.55) | 3.02× (2.99–3.04) | 2.53× (2.48–2.59) | 3.59× (3.50–3.69) | 2.99× (2.96–3.03) | 3.49× (3.44–3.54) | 3.76× (3.73–3.80) | 1.34× (1.33–1.35) | 1.59× (1.57–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.61–1.73) | 2.71× (2.64–2.79) | 2.08× (2.03–2.13) | 3.01× (2.93–3.09) | 2.72× (2.67–2.77) | 2.44× (2.40–2.49) | 2.63× (2.59–2.68) | 0.94× (0.92–0.95) | 1.11× (1.09–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.82× (2.77–2.87) | 3.67× (3.63–3.72) | 2.93× (2.88–2.97) | 6.25× (6.20–6.30) | 2.42× (2.39–2.46) | 2.24× (2.21–2.27) | 1.00× (0.98–1.01) | 1.03× (0.97–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.36× (2.32–2.40) | 4.00× (3.95–4.06) | 2.60× (2.56–2.65) | 5.41× (5.32–5.51) | 2.07× (2.04–2.11) | 1.91× (1.88–1.94) | 0.85× (0.84–0.87) | 0.88× (0.84–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.59× (2.54–2.63) | 3.98× (3.95–4.01) | 2.62× (2.58–2.65) | 5.38× (5.31–5.44) | 2.08× (2.05–2.10) | 1.92× (1.89–1.94) | 0.85× (0.84–0.87) | 0.88× (0.84–0.92) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.21× (2.19–2.23) | 4.27× (4.23–4.32) | 4.65× (4.60–4.69) | 4.79× (4.75–4.83) | 4.95× (4.90–5.01) | 5.34× (5.29–5.40) | 5.06× (5.01–5.11) | 1.67× (1.66–1.69) | 2.02× (2.00–2.04) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.85× (1.76–1.94) | 3.44× (3.40–3.48) | 3.80× (3.70–3.91) | 4.22× (4.15–4.28) | 3.91× (3.86–3.96) | 4.14× (4.09–4.20) | 3.92× (3.87–3.98) | 1.30× (1.28–1.32) | 1.57× (1.55–1.59) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.78× (1.74–1.82) | 2.60× (2.55–2.65) | 2.89× (2.84–2.93) | 3.11× (3.07–3.15) | 2.94× (2.90–2.99) | 2.58× (2.55–2.62) | 2.45× (2.41–2.48) | 0.81× (0.80–0.82) | 0.98× (0.96–0.99) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.29–0.30) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.03× (10.99–11.06) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.47× (0.42–0.53) | 0.73× (0.70–0.75) | 0.61× (0.59–0.63) | 5.76× (5.57–5.96) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.33–0.38) | 0.73× (0.61–0.87) | 0.64× (0.60–0.68) | 0.56× (0.51–0.61) | 3.70× (3.47–3.94) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.82–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.70) | 0.66× (0.66–0.66) | 7.98× (7.92–8.03) | 0.65× (0.64–0.66) | 0.68× (0.67–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.70× (0.68–0.72) | 0.72× (0.66–0.79) | 1.69× (1.64–1.73) | 0.84× (0.82–0.86) | 6.11× (5.94–6.29) | 0.50× (0.48–0.51) | 0.52× (0.51–0.53) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.69× (0.66–0.72) | 0.76× (0.72–0.80) | 1.23× (1.20–1.27) | 0.76× (0.73–0.79) | 3.64× (3.54–3.74) | 0.29× (0.28–0.29) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.14–5.15) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.40× (0.38–0.42) | 0.88× (0.85–0.91) | 0.70× (0.68–0.73) | 3.05× (2.95–3.15) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.65–0.69) | 0.47× (0.46–0.48) | 0.89× (0.87–0.91) | 0.84× (0.82–0.86) | 2.60× (2.53–2.66) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.22× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.35–0.42) | 0.87× (0.85–0.89) | 0.72× (0.69–0.76) | 2.99× (2.91–3.07) | 0.19× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.62) | 0.50× (0.47–0.53) | 0.81× (0.80–0.82) | 0.84× (0.80–0.87) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.29–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.87) | 0.89× (0.88–0.89) | 6.16× (6.13–6.18) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.05× (1.04–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.38) | 1.38× (1.38–1.39) | 0.65× (0.65–0.66) | 6.35× (6.34–6.36) | 0.38× (0.38–0.39) | 0.41× (0.41–0.41) | 0.62× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 36 MB | 0.46× (0.44–0.47) | 0.41× (0.39–0.44) | 1.04× (1.00–1.08) | 0.69× (0.67–0.71) | 3.43× (3.32–3.55) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.34× (0.33–0.35) | 0.16× (0.16–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 58 MB | 0.48× (0.47–0.49) | 0.42× (0.39–0.44) | 0.87× (0.85–0.89) | 0.70× (0.66–0.75) | 2.24× (2.19–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 439 | 2.29 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 781 | 2.43 | 1.07× (1.06–1.07) | 88% | 1.01× (1.01–1.02) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 969 | 3.86 | 1.67× (1.64–1.70) | 55% | 1.19× (1.18–1.20) | 1.61× (1.59–1.63) |

**Best:** mibz2 at 4 threads (969 MB/s); lbzip2 at 4 threads (197 MB/s); pbz2 at 4 threads (301 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 0.99, lbzip2 1t 0.09, lbzip2 2t 0.28, lbzip2 4t 0.61, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.08 | 0.216 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3267 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.4% |
| MTF + RLE2 | 1.55 | 17.9% |
| T-vector build | 1.52 | 17.6% |
| walk (the one each build uses) | 4.22 | 48.8% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.508 | 1.301 | (in its total) | 2.56× (2.56–2.57) |  |
| walk | 1.231 | 2.632 | (in its total) | 2.14× (2.12–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.053 | (in its total) | 1.15× (1.14–1.15) |  |
| inverse BWT, total | 1.785 | 3.986 | 3.235 | 2.23× (2.22–2.24) | 1.81× (1.80–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m27s |
| 4 threads | 31m40s ± 4m27s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m15s ± 3m18s |
| the scanner alone (its serial part) | 9m19s ± 32.2s |
| the workers alone, over 4 | 23m15s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.4 ns a byte of bz2, the workers 3.05 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,259,804 | 15,026,718 | \`b798cab9d5ca7cca…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3266.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
