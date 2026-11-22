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

The latest record: [`4a79f06`](<https://github.com/lmmx/mibz2/commit/4a79f0631e3c3e95e1b2d38090c75d6584e4817c>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45008723760>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.296 s | 708 | 6.4 | 2.89× (2.80–2.99) | 7.11× (6.83–7.41) | 7.39× (7.07–7.73) | 8.50× (8.33–8.69) | 7.57× (7.27–7.88) | 7.73× (7.41–8.06) | 7.41× (7.16–7.68) | 2.26× (2.20–2.32) | 2.60× (2.50–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.176 s | 1,190 | 7.2 | 2.97× (2.82–3.14) | 6.53× (6.32–6.74) | 8.25× (7.89–8.63) | 7.77× (7.43–8.12) | 6.98× (6.70–7.28) | 13.06× (12.45–13.70) | 12.53× (12.08–13.01) | 3.82× (3.64–4.01) | 4.39× (4.22–4.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.150 s | 1,399 | 11.5 | 2.96× (2.88–3.04) | 4.52× (4.43–4.61) | 6.25× (6.08–6.41) | 5.40× (5.26–5.55) | 5.60× (5.46–5.75) | 15.32× (15.07–15.58) | 14.70× (14.44–14.97) | 4.48× (4.39–4.57) | 5.15× (5.02–5.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,356 | 3.7 | 0.55× (0.53–0.57) | 3.60× (3.40–3.81) | 4.33× (4.17–4.49) | 3.59× (3.49–3.69) | 6.61× (6.44–6.78) | 2.77× (2.67–2.87) | 2.75× (2.68–2.82) | 1.01× (0.95–1.07) | 2.20× (2.03–2.37) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,278 | 4.1 | 0.50× (0.47–0.54) | 2.93× (2.62–3.28) | 6.05× (5.47–6.68) | 3.47× (3.31–3.63) | 6.17× (5.94–6.41) | 2.61× (2.48–2.74) | 2.59× (2.46–2.73) | 0.95× (0.88–1.03) | 2.07× (1.87–2.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,294 | 4.0 | 0.48× (0.46–0.51) | 2.91× (2.64–3.21) | 6.31× (5.60–7.11) | 3.53× (3.41–3.65) | 6.16× (5.89–6.43) | 2.61× (2.51–2.73) | 2.60× (2.49–2.71) | 0.95× (0.90–1.01) | 2.07× (1.87–2.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 998 | 4.9 | 2.79× (2.63–2.96) | 3.58× (3.42–3.74) | 4.29× (3.86–4.76) | 4.41× (4.12–4.71) | 5.39× (5.04–5.76) | 2.87× (2.75–3.01) | 2.49× (2.37–2.62) | 2.12× (1.98–2.26) | 3.12× (2.98–3.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,485 | 6.2 | 2.21× (2.00–2.45) | 2.92× (2.60–3.28) | 4.67× (4.05–5.39) | 3.77× (3.25–4.36) | 5.76× (5.11–6.48) | 3.99× (3.56–4.47) | 3.46× (3.08–3.89) | 2.94× (2.63–3.29) | 4.34× (3.89–4.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,273 | 11.9 | 1.97× (1.80–2.17) | 2.09× (1.97–2.22) | 3.55× (3.00–4.20) | 2.25× (2.13–2.38) | 4.78× (4.51–5.07) | 3.67× (3.46–3.88) | 3.18× (2.95–3.43) | 2.70× (2.56–2.84) | 3.99× (3.77–4.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.082 s | 97 | 47.5 | 1.33× (1.25–1.41) | 3.06× (2.89–3.24) | 2.72× (2.54–2.91) | 2.72× (2.58–2.87) | 3.09× (2.95–3.24) | 3.44× (3.26–3.63) | 3.45× (3.30–3.61) | 1.33× (1.26–1.42) | 1.51× (1.43–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 156 | 53.9 | 1.29× (1.23–1.37) | 2.99× (2.80–3.18) | 2.81× (2.60–3.03) | 3.42× (3.21–3.65) | 3.24× (3.08–3.40) | 5.60× (5.39–5.82) | 5.62× (5.38–5.87) | 2.17× (2.07–2.28) | 2.46× (2.29–2.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 190 | 73.5 | 1.32× (1.26–1.38) | 2.60× (2.47–2.73) | 2.56× (2.46–2.66) | 2.88× (2.77–3.00) | 3.29× (3.15–3.44) | 6.88× (6.58–7.20) | 6.91× (6.65–7.18) | 2.67× (2.53–2.81) | 3.02× (2.88–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.085 s | 114 | 40.7 | 1.53× (1.48–1.59) | 3.81× (3.66–3.98) | 3.17× (3.08–3.26) | 3.18× (3.11–3.25) | 3.64× (3.55–3.74) | 4.55× (4.45–4.65) | 4.84× (4.70–5.00) | 1.66× (1.61–1.70) | 1.90× (1.85–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.054 s | 179 | 48.2 | 1.37× (1.32–1.43) | 3.39× (3.21–3.58) | 3.05× (2.91–3.20) | 3.96× (3.80–4.12) | 3.55× (3.42–3.70) | 7.11× (6.85–7.37) | 7.57× (7.21–7.94) | 2.59× (2.48–2.70) | 2.97× (2.84–3.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.046 s | 210 | 69.8 | 1.45× (1.38–1.52) | 2.86× (2.70–3.02) | 2.64× (2.51–2.79) | 3.47× (3.33–3.61) | 3.30× (3.18–3.43) | 8.38× (8.06–8.71) | 8.92× (8.58–9.27) | 3.05× (2.92–3.19) | 3.50× (3.36–3.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 5.2 | 0.61× (0.55–0.67) | 3.10× (2.92–3.30) | 4.27× (4.01–4.55) | 3.08× (2.88–3.30) | 7.06× (6.64–7.50) | 2.36× (2.12–2.62) | 2.31× (2.17–2.46) | 0.94× (0.88–1.01) | 2.10× (1.95–2.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 948 | 5.9 | 0.46× (0.44–0.48) | 2.95× (2.74–3.19) | 5.82× (4.60–7.37) | 2.87× (2.73–3.01) | 6.61× (6.33–6.91) | 2.16× (1.97–2.38) | 2.12× (2.05–2.19) | 0.87× (0.82–0.92) | 1.93× (1.81–2.04) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 909 | 5.7 | 0.47× (0.43–0.50) | 2.81× (2.67–2.95) | 5.35× (4.67–6.12) | 2.91× (2.70–3.13) | 6.42× (6.08–6.78) | 2.12× (1.95–2.32) | 2.08× (1.92–2.25) | 0.85× (0.80–0.90) | 1.89× (1.77–2.01) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 0.046 s | 302 | 15.5 | 1.79× (1.75–1.83) | 4.45× (4.32–4.59) | 4.78× (4.58–4.98) | 4.94× (4.74–5.15) | 5.25× (5.05–5.46) | 5.20× (5.05–5.34) | 5.09× (4.93–5.26) | 1.56× (1.49–1.63) | 1.86× (1.76–1.96) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 0.032 s | 430 | 20.1 | 1.38× (1.28–1.49) | 3.71× (3.17–4.35) | 4.65× (4.28–5.05) | 4.76× (4.21–5.38) | 4.54× (4.22–4.89) | 7.19× (6.68–7.75) | 7.05× (6.58–7.55) | 2.16× (2.01–2.32) | 2.58× (2.38–2.79) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 0.026 s | 522 | 28.9 | 1.54× (1.49–1.60) | 2.84× (2.69–2.99) | 4.25× (3.99–4.52) | 3.96× (3.77–4.17) | 4.54× (4.40–4.68) | 9.00× (8.68–9.32) | 8.81× (8.52–9.12) | 2.70× (2.59–2.81) | 3.22× (3.03–3.43) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.79–2.98) | 7.06× (6.78–7.35) | 7.33× (7.01–7.67) | 8.46× (8.29–8.63) | 7.53× (7.23–7.83) | 7.69× (7.38–8.01) | 7.38× (7.13–7.63) | 2.25× (2.19–2.31) | 2.57× (2.47–2.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.80× (2.70–2.90) | 6.74× (6.62–6.87) | 7.41× (7.27–7.56) | 8.11× (7.92–8.31) | 6.92× (6.75–7.08) | 6.92× (6.70–7.15) | 6.64× (6.48–6.80) | 2.03× (1.97–2.09) | 2.32× (2.24–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.84× (2.76–2.92) | 4.94× (4.80–5.09) | 5.77× (5.54–6.02) | 5.98× (5.76–6.22) | 5.35× (5.17–5.52) | 4.35× (4.26–4.45) | 4.18× (4.02–4.34) | 1.27× (1.24–1.31) | 1.46× (1.39–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.58–0.62) | 3.04× (2.91–3.17) | 3.79× (3.69–3.89) | 3.33× (3.25–3.42) | 6.04× (5.89–6.20) | 2.59× (2.50–2.68) | 2.58× (2.52–2.64) | 1.01× (0.96–1.07) | 1.76× (1.70–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.51–0.58) | 2.63× (2.49–2.79) | 4.65× (4.42–4.90) | 3.12× (2.99–3.26) | 5.45× (5.26–5.65) | 2.35× (2.24–2.46) | 2.34× (2.22–2.46) | 0.92× (0.85–0.98) | 1.60× (1.49–1.71) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.49–0.54) | 2.93× (2.84–3.03) | 4.87× (4.71–5.03) | 3.25× (3.17–3.34) | 5.56× (5.36–5.76) | 2.42× (2.33–2.51) | 2.40× (2.32–2.49) | 0.94× (0.90–0.99) | 1.64× (1.55–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.65× (2.51–2.80) | 3.04× (2.91–3.18) | 3.80× (3.43–4.22) | 4.14× (3.89–4.40) | 5.05× (4.75–5.36) | 2.72× (2.61–2.84) | 2.38× (2.27–2.49) | 2.03× (1.90–2.16) | 2.66× (2.54–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.25–2.47) | 2.66× (2.54–2.79) | 3.27× (3.17–3.38) | 3.59× (3.40–3.80) | 4.19× (4.01–4.38) | 2.18× (2.09–2.27) | 1.90× (1.83–1.97) | 1.62× (1.55–1.69) | 2.12× (2.05–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (1.90–2.33) | 1.80× (1.72–1.88) | 2.22× (2.10–2.35) | 2.12× (2.00–2.24) | 2.46× (2.33–2.60) | 1.12× (1.07–1.18) | 0.98× (0.91–1.05) | 0.83× (0.79–0.89) | 1.09× (1.05–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.25–1.40) | 2.97× (2.81–3.14) | 2.63× (2.47–2.81) | 2.69× (2.55–2.83) | 3.05× (2.91–3.20) | 3.39× (3.22–3.58) | 3.41× (3.26–3.56) | 1.33× (1.25–1.41) | 1.44× (1.36–1.52) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.24–1.33) | 2.77× (2.70–2.85) | 2.46× (2.36–2.56) | 3.16× (3.05–3.28) | 2.81× (2.70–2.93) | 3.02× (2.94–3.10) | 3.03× (2.94–3.13) | 1.18× (1.13–1.23) | 1.28× (1.22–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.34–1.46) | 2.59× (2.49–2.70) | 2.12× (2.05–2.20) | 2.73× (2.62–2.85) | 2.66× (2.53–2.80) | 2.22× (2.12–2.32) | 2.23× (2.13–2.33) | 0.87× (0.83–0.91) | 0.94× (0.89–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.47–1.57) | 3.71× (3.57–3.86) | 3.08× (2.98–3.18) | 3.14× (3.07–3.21) | 3.59× (3.51–3.68) | 4.48× (4.39–4.58) | 4.77× (4.63–4.92) | 1.64× (1.60–1.69) | 1.83× (1.79–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.34–1.43) | 3.33× (3.24–3.42) | 2.69× (2.61–2.77) | 3.63× (3.54–3.72) | 3.14× (3.05–3.22) | 3.76× (3.64–3.88) | 4.00× (3.85–4.16) | 1.38× (1.33–1.43) | 1.53× (1.49–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.47–1.62) | 2.85× (2.69–3.02) | 2.15× (2.05–2.24) | 3.06× (2.94–3.19) | 2.78× (2.67–2.90) | 2.60× (2.49–2.71) | 2.76× (2.66–2.87) | 0.95× (0.91–1.00) | 1.06× (1.02–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.62–0.72) | 2.51× (2.37–2.65) | 3.42× (3.19–3.67) | 2.80× (2.65–2.96) | 6.24× (5.92–6.57) | 2.18× (1.98–2.41) | 2.14× (2.03–2.25) | 0.95× (0.90–1.01) | 1.37× (1.31–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.50–0.56) | 2.19× (2.06–2.33) | 3.87× (3.67–4.08) | 2.54× (2.43–2.65) | 5.57× (5.37–5.78) | 1.90× (1.75–2.07) | 1.86× (1.81–1.91) | 0.83× (0.79–0.87) | 1.19× (1.11–1.27) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.49–0.55) | 2.32× (2.25–2.38) | 3.93× (3.63–4.26) | 2.70× (2.53–2.89) | 5.69× (5.44–5.94) | 1.96× (1.81–2.11) | 1.91× (1.79–2.05) | 0.85× (0.81–0.90) | 1.23× (1.15–1.31) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 1.77× (1.73–1.80) | 4.22× (4.09–4.36) | 4.54× (4.35–4.73) | 4.81× (4.61–5.01) | 5.11× (4.91–5.31) | 5.06× (4.92–5.20) | 4.95× (4.80–5.11) | 1.54× (1.48–1.61) | 1.69× (1.63–1.76) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 1.49× (1.45–1.54) | 3.38× (3.26–3.50) | 3.80× (3.59–4.02) | 4.32× (4.17–4.47) | 4.05× (3.89–4.22) | 3.90× (3.78–4.04) | 3.83× (3.71–3.94) | 1.19× (1.15–1.23) | 1.31× (1.26–1.36) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 1.65× (1.59–1.72) | 2.81× (2.73–2.90) | 3.18× (3.09–3.27) | 3.59× (3.44–3.75) | 3.39× (3.33–3.46) | 2.71× (2.64–2.79) | 2.66× (2.59–2.73) | 0.83× (0.80–0.86) | 0.91× (0.87–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.97× (0.97–0.98) | 0.79× (0.76–0.81) | 10.99× (10.96–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.33–0.36) | 0.51× (0.44–0.60) | 0.71× (0.68–0.74) | 0.60× (0.58–0.63) | 5.68× (5.46–5.91) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.35–0.38) | 0.61× (0.49–0.76) | 0.66× (0.63–0.69) | 0.57× (0.55–0.60) | 3.81× (3.63–3.99) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.19× (0.17–0.21) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.43–0.46) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.95× (7.93–7.97) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.62× (0.57–0.67) | 1.60× (1.55–1.66) | 0.81× (0.78–0.84) | 5.81× (5.61–6.03) | 0.47× (0.46–0.49) | 0.49× (0.48–0.51) | 0.54× (0.52–0.56) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.68× (0.65–0.71) | 0.77× (0.73–0.80) | 1.22× (1.16–1.28) | 0.75× (0.72–0.79) | 3.56× (3.41–3.72) | 0.28× (0.27–0.29) | 0.29× (0.28–0.30) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.52) | 0.39× (0.36–0.42) | 0.89× (0.87–0.91) | 0.71× (0.69–0.74) | 3.08× (3.01–3.15) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.45–0.47) | 0.89× (0.87–0.90) | 0.83× (0.80–0.86) | 2.58× (2.53–2.64) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.46× (0.44–0.48) | 0.36× (0.34–0.38) | 0.81× (0.78–0.84) | 0.69× (0.65–0.72) | 2.78× (2.67–2.89) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.13–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.46–0.52) | 0.80× (0.79–0.82) | 0.82× (0.78–0.86) | 2.32× (2.27–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.05–1.06) | 0.76× (0.75–0.76) | 5.29× (5.28–5.30) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.90) | 6.19× (6.17–6.22) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.25× (0.23–0.28) | 1.87× (1.87–1.88) | 0.90× (0.90–0.91) | 6.22× (6.20–6.25) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.66–0.67) | 6.39× (6.37–6.40) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 35 MB | 0.48× (0.47–0.49) | 0.41× (0.37–0.45) | 1.10× (1.06–1.13) | 0.73× (0.71–0.75) | 3.54× (3.45–3.64) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 53 MB | 0.52× (0.50–0.53) | 0.43× (0.40–0.46) | 0.94× (0.92–0.97) | 0.79× (0.76–0.81) | 2.41× (2.36–2.47) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 708 | 1.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,190 | 1.58 | 1.11× (1.07–1.16) | 85% | 1.06× (1.04–1.08) | 1.08× (1.05–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 1,399 | 2.53 | 1.77× (1.69–1.85) | 50% | 1.24× (1.21–1.27) | 1.74× (1.67–1.81) |

**Best:** mibz2 at 4 threads (1,399 MB/s); lbzip2 at 4 threads (309 MB/s); pbz2 at 4 threads (476 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.44, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.41 | 0.01 | 0.07 | 0.150 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4532 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 12.3% |
| MTF + RLE2 | 1.42 | 21.1% |
| T-vector build | 1.31 | 19.5% |
| walk (the one each build uses) | 2.78 | 41.3% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.8% |
| **total** | 6.74 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.285 | 0.684 | (in its total) | 2.40× (2.39–2.41) |  |
| walk | 0.588 | 1.516 | (in its total) | 2.59× (2.57–2.61) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.024 | 0.028 | (in its total) | 1.14× (1.13–1.15) |  |
| inverse BWT, total | 0.898 | 2.228 | 1.271 | 2.49× (2.48–2.51) | 1.42× (1.40–1.43) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m26s ± 5m19s |
| 4 threads | 21m54s ± 3m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m35s ± 2m09s |
| the scanner alone (its serial part) | 6m42s ± 32.2s |
| the workers alone, over 4 | 14m35s ± 2m09s |

At 4 threads the workers bound it. Per byte: the scanner 3.9 ns a byte of bz2, the workers 1.91 ns a byte of output at 4 threads (CPU, summed over them), 1.20 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,732,203 | 13,845,587 | \`5d6d3769149d431f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4531.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
