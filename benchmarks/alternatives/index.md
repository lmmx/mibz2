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

The latest record: [`c7fa328`](<https://github.com/lmmx/mibz2/commit/c7fa32844884c5049cb2515d6a975c3388862ea7>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46111425166>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.484 s | 433 | 7.5 | 3.17× (3.07–3.28) | 7.11× (6.91–7.31) | 7.31× (7.09–7.54) | 8.04× (7.82–8.26) | 7.18× (6.95–7.43) | 7.83× (7.60–8.07) | 7.32× (7.07–7.58) | 2.33× (2.25–2.41) | 2.68× (2.62–2.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.269 s | 781 | 8.1 | 3.10× (2.89–3.32) | 6.68× (6.27–7.11) | 7.65× (7.18–8.15) | 7.60× (7.13–8.11) | 6.81× (6.34–7.31) | 13.70× (12.90–14.55) | 12.80× (11.95–13.72) | 4.07× (3.79–4.36) | 4.69× (4.42–4.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.223 s | 942 | 12.9 | 3.28× (3.20–3.36) | 4.98× (4.88–5.08) | 6.14× (5.88–6.41) | 5.61× (5.49–5.74) | 5.68× (5.49–5.87) | 17.07× (16.75–17.39) | 15.95× (15.58–16.32) | 5.07× (4.97–5.16) | 5.84× (5.76–5.92) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 957 | 3.7 | 0.60× (0.58–0.61) | 3.61× (3.44–3.79) | 4.51× (4.33–4.70) | 3.52× (3.47–3.58) | 6.13× (6.03–6.23) | 2.93× (2.88–2.97) | 2.62× (2.57–2.66) | 1.01× (0.98–1.03) | 1.48× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 911 | 4.1 | 0.54× (0.53–0.56) | 2.81× (2.75–2.87) | 5.90× (5.78–6.02) | 3.37× (3.31–3.43) | 5.78× (5.66–5.90) | 2.78× (2.73–2.84) | 2.49× (2.43–2.54) | 0.96× (0.94–0.98) | 1.41× (1.37–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 893 | 4.1 | 0.54× (0.53–0.55) | 2.75× (2.72–2.79) | 5.81× (5.71–5.91) | 3.28× (3.24–3.31) | 5.76× (5.69–5.84) | 2.72× (2.68–2.76) | 2.43× (2.40–2.47) | 0.94× (0.92–0.95) | 1.37× (1.35–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 584 | 5.9 | 2.92× (2.80–3.04) | 3.05× (3.00–3.10) | 3.91× (3.84–3.97) | 3.97× (3.91–4.04) | 4.25× (4.18–4.31) | 2.63× (2.53–2.73) | 2.13× (2.09–2.17) | 2.04× (2.01–2.08) | 3.03× (2.98–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 914 | 7.1 | 2.33× (2.22–2.44) | 2.92× (2.70–3.15) | 4.23× (4.12–4.35) | 3.76× (3.50–4.05) | 4.99× (4.78–5.20) | 4.03× (3.81–4.26) | 3.27× (3.12–3.42) | 3.13× (3.00–3.27) | 4.65× (4.45–4.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 848 | 14.2 | 2.07× (2.00–2.15) | 2.16× (2.02–2.30) | 3.27× (3.14–3.41) | 2.42× (2.31–2.54) | 4.31× (4.16–4.46) | 3.79× (3.63–3.95) | 3.07× (2.98–3.17) | 2.94× (2.86–3.04) | 4.38× (4.24–4.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.122 s | 66 | 50.1 | 1.49× (1.48–1.51) | 2.95× (2.92–2.98) | 2.65× (2.59–2.72) | 2.69× (2.65–2.72) | 3.13× (3.09–3.18) | 3.43× (3.38–3.47) | 3.47× (3.43–3.51) | 1.36× (1.33–1.38) | 1.57× (1.55–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 108 | 55.4 | 1.44× (1.39–1.49) | 2.79× (2.72–2.86) | 2.61× (2.53–2.68) | 3.36× (3.27–3.45) | 3.25× (3.17–3.34) | 5.61× (5.47–5.76) | 5.68× (5.54–5.83) | 2.22× (2.15–2.30) | 2.57× (2.49–2.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.061 s | 132 | 75.6 | 1.49× (1.47–1.52) | 2.42× (2.35–2.48) | 2.37× (2.28–2.45) | 2.96× (2.90–3.02) | 3.27× (3.21–3.32) | 6.82× (6.68–6.97) | 6.91× (6.78–7.03) | 2.70× (2.62–2.79) | 3.12× (3.04–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.135 s | 71 | 46.4 | 1.60× (1.56–1.64) | 3.29× (3.24–3.34) | 2.76× (2.74–2.78) | 2.87× (2.83–2.91) | 3.32× (3.27–3.37) | 3.92× (3.87–3.96) | 4.26× (4.21–4.31) | 1.56× (1.52–1.60) | 1.83× (1.81–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 52.4 | 1.46× (1.41–1.51) | 3.06× (2.97–3.15) | 2.67× (2.56–2.78) | 3.77× (3.58–3.96) | 3.36× (3.25–3.47) | 6.45× (6.26–6.65) | 7.01× (6.82–7.21) | 2.56× (2.47–2.67) | 3.02× (2.93–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 72.8 | 1.61× (1.56–1.65) | 2.62× (2.44–2.82) | 2.51× (2.44–2.58) | 3.39× (3.30–3.47) | 3.21× (3.10–3.32) | 7.75× (7.55–7.95) | 8.42× (8.21–8.64) | 3.08× (2.96–3.20) | 3.62× (3.51–3.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 730 | 5.1 | 0.67× (0.66–0.67) | 3.07× (2.94–3.21) | 4.40× (4.33–4.46) | 3.17× (3.02–3.34) | 6.83× (6.78–6.89) | 2.53× (2.50–2.55) | 2.32× (2.30–2.35) | 0.98× (0.98–0.99) | 1.52× (1.50–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 658 | 5.9 | 0.53× (0.53–0.54) | 2.71× (2.67–2.76) | 4.77× (4.62–4.93) | 2.85× (2.80–2.90) | 6.22× (6.13–6.31) | 2.27× (2.23–2.32) | 2.09× (2.05–2.13) | 0.89× (0.87–0.91) | 1.36× (1.34–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 647 | 5.8 | 0.53× (0.52–0.54) | 2.67× (2.63–2.72) | 4.79× (4.59–5.00) | 2.78× (2.74–2.82) | 6.14× (6.02–6.25) | 2.23× (2.21–2.26) | 2.05× (2.02–2.08) | 0.87× (0.86–0.88) | 1.34× (1.32–1.36) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.066 s | 226 | 14.7 | 2.21× (2.17–2.25) | 4.28× (4.19–4.38) | 4.69× (4.62–4.76) | 4.72× (4.62–4.82) | 4.91× (4.82–5.01) | 5.28× (5.18–5.38) | 4.98× (4.90–5.07) | 1.67× (1.63–1.71) | 2.10× (2.08–2.13) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.047 s | 320 | 19.2 | 1.69× (1.63–1.74) | 3.74× (3.27–4.29) | 4.13× (3.75–4.54) | 4.49× (4.25–4.74) | 4.24× (4.12–4.37) | 7.29× (7.10–7.49) | 6.89× (6.70–7.08) | 2.31× (2.24–2.38) | 2.90× (2.83–2.98) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.041 s | 367 | 28.8 | 1.72× (1.68–1.78) | 2.66× (2.51–2.82) | 3.68× (3.54–3.83) | 3.78× (3.54–4.03) | 3.99× (3.87–4.12) | 8.54× (8.32–8.76) | 8.06× (7.87–8.26) | 2.70× (2.61–2.79) | 3.40× (3.26–3.54) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.16× (3.06–3.27) | 7.07× (6.87–7.28) | 7.28× (7.06–7.50) | 8.01× (7.79–8.23) | 7.15× (6.92–7.40) | 7.80× (7.57–8.04) | 7.29× (7.04–7.55) | 2.32× (2.24–2.40) | 2.66× (2.60–2.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (2.97–3.15) | 6.78× (6.61–6.96) | 7.22× (6.99–7.45) | 7.94× (7.72–8.18) | 6.77× (6.53–7.02) | 7.24× (7.07–7.42) | 6.77× (6.54–7.00) | 2.15× (2.08–2.23) | 2.47× (2.41–2.53) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.07× (3.01–3.14) | 5.18× (5.06–5.31) | 5.58× (5.49–5.67) | 5.89× (5.83–5.95) | 5.25× (5.14–5.37) | 4.62× (4.56–4.69) | 4.32× (4.23–4.41) | 1.37× (1.34–1.41) | 1.58× (1.55–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.65) | 3.20× (3.15–3.26) | 4.03× (3.94–4.11) | 3.30× (3.24–3.36) | 5.68× (5.57–5.79) | 2.76× (2.70–2.81) | 2.47× (2.42–2.52) | 1.01× (0.98–1.03) | 1.07× (1.05–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.84× (2.79–2.90) | 5.03× (4.94–5.13) | 3.07× (3.01–3.13) | 5.18× (5.08–5.28) | 2.53× (2.48–2.58) | 2.27× (2.22–2.32) | 0.93× (0.90–0.95) | 0.98× (0.95–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.07× (3.04–3.11) | 4.96× (4.90–5.03) | 3.03× (3.00–3.06) | 5.22× (5.16–5.28) | 2.51× (2.48–2.55) | 2.25× (2.22–2.28) | 0.92× (0.90–0.93) | 0.97× (0.95–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.81× (2.70–2.93) | 2.81× (2.76–2.86) | 3.55× (3.50–3.60) | 3.81× (3.74–3.88) | 4.06× (4.00–4.13) | 2.54× (2.44–2.64) | 2.07× (2.03–2.11) | 1.98× (1.95–2.02) | 2.74× (2.70–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.36× (2.32–2.40) | 2.61× (2.53–2.69) | 3.19× (3.15–3.24) | 3.33× (3.27–3.40) | 3.49× (3.41–3.57) | 2.09× (2.02–2.17) | 1.71× (1.67–1.75) | 1.64× (1.60–1.67) | 2.26× (2.22–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.94–2.12) | 1.78× (1.74–1.81) | 2.22× (2.17–2.27) | 2.08× (2.01–2.16) | 2.13× (2.08–2.18) | 1.05× (1.02–1.09) | 0.86× (0.84–0.88) | 0.82× (0.81–0.84) | 1.14× (1.12–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.47–1.50) | 2.86× (2.83–2.90) | 2.59× (2.54–2.64) | 2.66× (2.63–2.69) | 3.09× (3.05–3.13) | 3.38× (3.34–3.43) | 3.42× (3.38–3.47) | 1.35× (1.33–1.37) | 1.51× (1.50–1.52) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.43× (1.39–1.47) | 2.71× (2.68–2.74) | 2.40× (2.38–2.43) | 3.20× (3.17–3.24) | 2.84× (2.81–2.87) | 3.04× (3.00–3.08) | 3.08× (3.04–3.11) | 1.21× (1.19–1.24) | 1.35× (1.33–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.59–1.66) | 2.58× (2.50–2.65) | 2.10× (2.04–2.15) | 2.82× (2.78–2.87) | 2.71× (2.65–2.77) | 2.23× (2.19–2.27) | 2.26× (2.22–2.30) | 0.89× (0.87–0.91) | 0.99× (0.98–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.59× (1.55–1.63) | 3.23× (3.18–3.28) | 2.70× (2.67–2.73) | 2.84× (2.80–2.88) | 3.29× (3.24–3.34) | 3.87× (3.83–3.92) | 4.21× (4.17–4.26) | 1.55× (1.51–1.59) | 1.79× (1.77–1.81) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.44–1.49) | 3.01× (2.95–3.08) | 2.49× (2.44–2.54) | 3.50× (3.43–3.56) | 2.99× (2.92–3.07) | 3.42× (3.36–3.48) | 3.72× (3.66–3.78) | 1.37× (1.33–1.41) | 1.58× (1.55–1.62) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.76× (1.73–1.78) | 2.86× (2.70–3.03) | 2.14× (2.08–2.20) | 3.10× (3.03–3.18) | 2.81× (2.74–2.88) | 2.46× (2.41–2.50) | 2.67× (2.63–2.72) | 0.98× (0.95–1.02) | 1.14× (1.11–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.73× (2.70–2.76) | 3.56× (3.49–3.63) | 2.90× (2.75–3.06) | 6.10× (6.04–6.17) | 2.34× (2.31–2.37) | 2.16× (2.12–2.19) | 0.99× (0.98–1.00) | 0.99× (0.94–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.32× (2.28–2.36) | 3.98× (3.92–4.03) | 2.57× (2.52–2.62) | 5.35× (5.27–5.43) | 2.02× (1.99–2.06) | 1.86× (1.83–1.89) | 0.85× (0.84–0.87) | 0.86× (0.82–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.53× (2.48–2.59) | 4.02× (3.88–4.16) | 2.57× (2.53–2.61) | 5.41× (5.31–5.52) | 2.03× (2.01–2.06) | 1.87× (1.84–1.90) | 0.86× (0.85–0.87) | 0.86× (0.81–0.91) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.17× (2.14–2.21) | 4.12× (4.04–4.20) | 4.51× (4.44–4.59) | 4.61× (4.52–4.70) | 4.79× (4.70–4.88) | 5.15× (5.06–5.24) | 4.86× (4.78–4.95) | 1.64× (1.62–1.67) | 2.00× (1.97–2.02) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.79× (1.76–1.81) | 3.33× (3.29–3.38) | 3.73× (3.59–3.87) | 4.07× (3.97–4.18) | 3.78× (3.68–3.88) | 3.93× (3.87–4.00) | 3.71× (3.66–3.77) | 1.25× (1.23–1.27) | 1.52× (1.50–1.55) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.82× (1.76–1.89) | 2.73× (2.65–2.80) | 3.01× (2.91–3.12) | 3.23× (3.09–3.37) | 3.06× (2.95–3.16) | 2.59× (2.52–2.67) | 2.45× (2.38–2.51) | 0.83× (0.80–0.85) | 1.01× (0.98–1.03) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.28–0.32) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.97–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.34× (0.33–0.36) | 0.48× (0.40–0.57) | 0.72× (0.69–0.74) | 0.60× (0.58–0.61) | 5.66× (5.44–5.90) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.38) | 0.68× (0.56–0.82) | 0.64× (0.61–0.67) | 0.56× (0.53–0.59) | 3.73× (3.56–3.90) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.19) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.70) | 0.66× (0.66–0.67) | 7.96× (7.91–8.01) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.73) | 0.76× (0.71–0.82) | 1.72× (1.69–1.74) | 0.86× (0.84–0.87) | 6.21× (6.12–6.30) | 0.50× (0.50–0.51) | 0.53× (0.52–0.53) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.71× (0.69–0.73) | 0.82× (0.79–0.86) | 1.27× (1.25–1.30) | 0.78× (0.76–0.80) | 3.75× (3.69–3.81) | 0.29× (0.29–0.30) | 0.31× (0.30–0.31) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.39× (0.35–0.43) | 0.89× (0.86–0.92) | 0.71× (0.68–0.74) | 3.09× (2.99–3.20) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.47× (0.46–0.47) | 0.89× (0.88–0.91) | 0.84× (0.82–0.86) | 2.60× (2.56–2.64) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.13–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.84× (0.83–0.84) | 5.23× (5.21–5.26) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.40) | 0.85× (0.82–0.88) | 0.69× (0.66–0.73) | 2.92× (2.83–3.01) | 0.18× (0.18–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.62) | 0.48× (0.46–0.51) | 0.79× (0.78–0.80) | 0.80× (0.77–0.84) | 2.28× (2.24–2.31) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.75× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.89× (0.89–0.90) | 6.19× (6.16–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.16× (6.14–6.19) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.41× (6.40–6.42) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 35 MB | 0.48× (0.47–0.49) | 0.44× (0.40–0.48) | 1.09× (1.06–1.13) | 0.72× (0.70–0.73) | 3.58× (3.51–3.64) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 55 MB | 0.50× (0.49–0.52) | 0.49× (0.45–0.52) | 0.93× (0.91–0.95) | 0.77× (0.76–0.79) | 2.38× (2.34–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 433 | 2.32 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 781 | 2.49 | 1.08× (1.04–1.12) | 87% | 1.03× (1.02–1.05) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 942 | 3.98 | 1.69× (1.62–1.75) | 54% | 1.24× (1.20–1.27) | 1.64× (1.59–1.69) |

**Best:** mibz2 at 4 threads (942 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (287 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.01, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.66 | 0.02 | 0.09 | 0.223 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3238 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.4% |
| MTF + RLE2 | 1.53 | 17.9% |
| T-vector build | 1.51 | 17.6% |
| walk (the one each build uses) | 4.15 | 48.6% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.508 | 1.301 | (in its total) | 2.56× (2.55–2.57) |  |
| walk | 1.233 | 2.642 | (in its total) | 2.15× (2.14–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.056 | (in its total) | 1.24× (1.20–1.28) |  |
| inverse BWT, total | 1.786 | 4.003 | 3.282 | 2.25× (2.24–2.26) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m50s |
| 4 threads | 32m40s ± 4m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m41s ± 3m22s |
| the scanner alone (its serial part) | 9m36s ± 32.2s |
| the workers alone, over 4 | 23m41s ± 3m22s |

At 4 threads the workers bound it. Per byte: the scanner 5.6 ns a byte of bz2, the workers 3.10 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,260,864 | 15,030,132 | \`de55d3765b0096e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3238.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
