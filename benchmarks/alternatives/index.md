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

The latest record: [`bfa1231`](<https://github.com/lmmx/mibz2/commit/bfa12319d16a2cd68b66ce6746904dbc082bcc8a>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46037528989>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.386 s | 543 | 6.8 | 3.22× (3.14–3.29) | 7.30× (7.22–7.38) | 7.56× (7.50–7.62) | 8.71× (8.48–8.95) | 7.66× (7.41–7.91) | 7.97× (7.82–8.12) | 7.74× (7.55–7.94) | 2.37× (2.34–2.40) | 2.64× (2.61–2.67) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.220 s | 952 | 7.4 | 3.28× (3.20–3.35) | 6.92× (6.57–7.30) | 7.72× (7.49–7.95) | 8.14× (7.84–8.45) | 7.18× (6.93–7.44) | 13.97× (13.45–14.51) | 13.57× (12.96–14.21) | 4.16× (4.01–4.31) | 4.62× (4.47–4.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,156 | 11.8 | 3.20× (3.11–3.30) | 4.91× (4.67–5.16) | 5.99× (5.66–6.33) | 5.98× (5.70–6.26) | 5.70× (5.51–5.89) | 16.86× (16.31–17.44) | 16.39× (15.81–16.99) | 5.02× (4.89–5.16) | 5.58× (5.43–5.74) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,070 | 3.8 | 0.55× (0.55–0.56) | 3.26× (3.22–3.30) | 4.41× (4.27–4.55) | 3.45× (3.42–3.48) | 5.97× (5.89–6.06) | 2.54× (2.51–2.58) | 2.55× (2.52–2.59) | 0.93× (0.91–0.96) | 2.00× (1.81–2.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 993 | 4.2 | 0.49× (0.48–0.49) | 2.58× (2.53–2.62) | 5.16× (5.07–5.25) | 3.26× (3.21–3.32) | 5.54× (5.44–5.64) | 2.40× (2.36–2.44) | 2.41× (2.37–2.45) | 0.88× (0.86–0.90) | 1.88× (1.71–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,018 | 4.0 | 0.49× (0.47–0.51) | 2.58× (2.51–2.66) | 5.16× (5.01–5.30) | 3.27× (3.18–3.37) | 5.67× (5.49–5.86) | 2.40× (2.33–2.46) | 2.41× (2.34–2.48) | 0.88× (0.85–0.91) | 1.88× (1.68–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 763 | 5.2 | 2.80× (2.77–2.82) | 3.16× (3.12–3.19) | 3.95× (3.90–4.00) | 4.23× (4.18–4.28) | 4.70× (4.61–4.79) | 2.86× (2.82–2.90) | 2.34× (2.31–2.37) | 2.19× (2.14–2.23) | 3.15× (3.11–3.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,174 | 6.3 | 2.47× (2.34–2.61) | 3.13× (3.01–3.27) | 4.64× (4.44–4.86) | 3.80× (3.76–3.84) | 5.71× (5.52–5.91) | 4.43× (4.38–4.48) | 3.62× (3.58–3.66) | 3.38× (3.31–3.46) | 4.87× (4.82–4.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,032 | 12.5 | 2.05× (1.95–2.15) | 2.04× (1.86–2.25) | 3.34× (3.15–3.55) | 2.32× (2.20–2.44) | 4.51× (4.25–4.77) | 3.92× (3.71–4.14) | 3.20× (3.03–3.38) | 2.99× (2.84–3.16) | 4.31× (4.08–4.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.103 s | 78 | 48.5 | 1.50× (1.48–1.52) | 2.86× (2.80–2.91) | 2.58× (2.54–2.62) | 2.69× (2.65–2.72) | 3.06× (2.97–3.15) | 3.22× (3.18–3.27) | 3.37× (3.32–3.42) | 1.35× (1.33–1.38) | 1.53× (1.49–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 128 | 54.1 | 1.39× (1.30–1.48) | 2.65× (2.45–2.85) | 2.57× (2.40–2.75) | 3.33× (3.13–3.54) | 3.16× (2.90–3.43) | 5.19× (4.84–5.57) | 5.43× (5.07–5.81) | 2.18× (2.03–2.34) | 2.47× (2.31–2.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 71.8 | 1.48× (1.43–1.53) | 2.48× (2.41–2.56) | 2.53× (2.32–2.76) | 3.03× (2.89–3.17) | 3.31× (3.24–3.38) | 6.67× (6.54–6.81) | 6.97× (6.85–7.10) | 2.80× (2.73–2.87) | 3.17× (3.07–3.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.113 s | 85 | 44.0 | 1.62× (1.60–1.64) | 3.26× (3.21–3.30) | 2.83× (2.77–2.89) | 2.96× (2.91–3.01) | 3.38× (3.24–3.53) | 3.96× (3.89–4.04) | 4.33× (4.24–4.41) | 1.63× (1.59–1.67) | 1.93× (1.90–1.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 140 | 49.3 | 1.48× (1.42–1.56) | 3.01× (2.91–3.12) | 2.83× (2.74–2.92) | 3.81× (3.66–3.96) | 3.38× (3.23–3.53) | 6.49× (6.32–6.67) | 7.08× (6.89–7.29) | 2.67× (2.61–2.72) | 3.16× (3.07–3.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.057 s | 170 | 68.3 | 1.57× (1.53–1.60) | 2.64× (2.45–2.85) | 2.66× (2.44–2.91) | 3.45× (3.38–3.53) | 3.31× (3.15–3.49) | 7.87× (7.71–8.03) | 8.59× (8.39–8.78) | 3.23× (3.13–3.34) | 3.83× (3.73–3.93) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 810 | 5.2 | 0.62× (0.59–0.64) | 3.32× (3.26–3.38) | 4.16× (4.08–4.24) | 3.00× (2.94–3.05) | 6.52× (6.35–6.69) | 2.19× (2.16–2.23) | 2.24× (2.20–2.28) | 0.93× (0.90–0.95) | 1.67× (1.64–1.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 738 | 6.0 | 0.48× (0.47–0.49) | 2.28× (2.25–2.31) | 4.64× (4.49–4.80) | 2.75× (2.72–2.78) | 5.99× (5.86–6.12) | 2.00× (1.97–2.03) | 2.05× (2.01–2.08) | 0.85× (0.83–0.86) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 741 | 5.8 | 0.48× (0.48–0.49) | 2.29× (2.26–2.33) | 4.65× (4.47–4.84) | 2.78× (2.74–2.82) | 6.04× (5.89–6.20) | 2.01× (1.98–2.03) | 2.05× (2.03–2.07) | 0.85× (0.83–0.87) | 1.53× (1.50–1.55) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 0.050 s | 285 | 13.4 | 2.02× (1.99–2.04) | 4.09× (4.05–4.13) | 4.58× (4.52–4.64) | 4.81× (4.77–4.85) | 5.12× (5.04–5.20) | 4.75× (4.67–4.82) | 4.85× (4.78–4.92) | 1.66× (1.63–1.70) | 1.86× (1.81–1.91) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 0.036 s | 394 | 17.9 | 1.50× (1.36–1.66) | 3.19× (2.88–3.53) | 3.67× (3.29–4.08) | 4.06× (3.67–4.50) | 4.39× (3.97–4.87) | 6.32× (5.72–6.98) | 6.45× (5.83–7.15) | 2.21× (1.99–2.46) | 2.47× (2.25–2.72) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 0.031 s | 452 | 27.1 | 1.63× (1.57–1.69) | 2.63× (2.43–2.85) | 3.42× (3.26–3.60) | 3.62× (3.54–3.70) | 4.22× (4.09–4.36) | 7.58× (7.35–7.82) | 7.74× (7.50–7.99) | 2.65× (2.60–2.71) | 2.96× (2.88–3.06) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.21× (3.13–3.28) | 7.26× (7.18–7.34) | 7.51× (7.45–7.57) | 8.68× (8.45–8.91) | 7.63× (7.38–7.88) | 7.93× (7.78–8.08) | 7.71× (7.52–7.91) | 2.37× (2.34–2.40) | 2.62× (2.59–2.65) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.01–3.09) | 7.01× (6.83–7.19) | 7.17× (7.03–7.31) | 8.39× (8.26–8.52) | 7.07× (6.91–7.24) | 7.31× (7.15–7.47) | 7.10× (6.90–7.31) | 2.18× (2.13–2.23) | 2.41× (2.38–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (2.97–3.15) | 5.23× (5.04–5.43) | 5.59× (5.37–5.82) | 6.36× (6.14–6.60) | 5.37× (5.26–5.48) | 4.64× (4.53–4.75) | 4.51× (4.39–4.63) | 1.38× (1.36–1.41) | 1.53× (1.51–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.82× (2.78–2.85) | 3.80× (3.70–3.89) | 3.24× (3.21–3.27) | 5.54× (5.46–5.62) | 2.41× (2.37–2.44) | 2.42× (2.39–2.45) | 0.94× (0.92–0.96) | 1.58× (1.56–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.52× (0.51–0.53) | 2.46× (2.42–2.50) | 4.49× (4.42–4.56) | 2.99× (2.95–3.03) | 4.99× (4.91–5.08) | 2.20× (2.17–2.23) | 2.21× (2.18–2.24) | 0.86× (0.83–0.88) | 1.44× (1.42–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.50–0.53) | 2.74× (2.66–2.81) | 4.53× (4.42–4.64) | 3.05× (2.97–3.14) | 5.18× (5.01–5.35) | 2.24× (2.18–2.30) | 2.25× (2.19–2.32) | 0.87× (0.84–0.90) | 1.47× (1.42–1.51) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.67–2.70) | 2.92× (2.89–2.95) | 3.51× (3.46–3.56) | 4.02× (3.98–4.07) | 4.46× (4.39–4.54) | 2.74× (2.71–2.78) | 2.26× (2.23–2.28) | 2.11× (2.07–2.16) | 2.74× (2.69–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.40× (2.37–2.44) | 2.63× (2.58–2.69) | 3.52× (3.48–3.57) | 3.48× (3.42–3.55) | 3.85× (3.77–3.95) | 2.25× (2.21–2.29) | 1.85× (1.81–1.89) | 1.73× (1.69–1.78) | 2.24× (2.20–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.03–2.21) | 1.78× (1.71–1.86) | 2.33× (2.26–2.41) | 2.14× (2.05–2.23) | 2.22× (2.13–2.30) | 1.12× (1.08–1.17) | 0.92× (0.89–0.96) | 0.86× (0.83–0.90) | 1.12× (1.07–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.47–1.51) | 2.77× (2.73–2.81) | 2.52× (2.48–2.56) | 2.66× (2.62–2.69) | 3.03× (2.94–3.12) | 3.18× (3.14–3.23) | 3.33× (3.28–3.38) | 1.35× (1.32–1.37) | 1.48× (1.46–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.38× (1.35–1.42) | 2.62× (2.55–2.70) | 2.35× (2.29–2.42) | 3.16× (3.10–3.21) | 2.80× (2.64–2.98) | 2.84× (2.77–2.91) | 2.97× (2.91–3.03) | 1.20× (1.17–1.23) | 1.32× (1.29–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.62× (1.58–1.67) | 2.56× (2.45–2.67) | 2.25× (2.05–2.46) | 2.91× (2.83–2.99) | 2.76× (2.67–2.86) | 2.18× (2.14–2.22) | 2.28× (2.24–2.31) | 0.92× (0.90–0.94) | 1.01× (0.99–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.61× (1.59–1.63) | 3.16× (3.12–3.21) | 2.76× (2.70–2.81) | 2.93× (2.88–2.97) | 3.35× (3.20–3.49) | 3.92× (3.85–3.99) | 4.27× (4.19–4.35) | 1.62× (1.58–1.66) | 1.86× (1.84–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.43–1.54) | 2.96× (2.87–3.05) | 2.58× (2.54–2.62) | 3.56× (3.43–3.70) | 2.99× (2.87–3.12) | 3.43× (3.34–3.53) | 3.74× (3.64–3.85) | 1.42× (1.39–1.44) | 1.63× (1.60–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.68–1.77) | 2.89× (2.66–3.13) | 2.34× (2.05–2.66) | 3.11× (3.01–3.21) | 2.95× (2.79–3.12) | 2.52× (2.46–2.57) | 2.75× (2.70–2.80) | 1.04× (1.01–1.07) | 1.20× (1.17–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.64–0.69) | 2.39× (2.35–2.44) | 3.32× (3.24–3.39) | 2.74× (2.70–2.79) | 5.83× (5.70–5.97) | 2.04× (2.01–2.07) | 2.09× (2.05–2.12) | 0.94× (0.92–0.96) | 1.33× (1.30–1.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.52–0.54) | 2.04× (2.01–2.06) | 3.67× (3.58–3.78) | 2.48× (2.45–2.51) | 5.17× (5.05–5.28) | 1.79× (1.76–1.82) | 1.83× (1.80–1.86) | 0.82× (0.81–0.84) | 1.16× (1.15–1.18) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.52–0.53) | 2.27× (2.25–2.30) | 3.76× (3.65–3.87) | 2.57× (2.53–2.61) | 5.35× (5.23–5.48) | 1.84× (1.82–1.86) | 1.88× (1.86–1.91) | 0.85× (0.83–0.86) | 1.20× (1.18–1.22) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 1.99× (1.96–2.01) | 3.90× (3.87–3.93) | 4.38× (4.36–4.40) | 4.69× (4.65–4.72) | 4.99× (4.92–5.06) | 4.62× (4.55–4.69) | 4.72× (4.66–4.79) | 1.64× (1.61–1.67) | 1.79× (1.78–1.81) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 1.59× (1.57–1.61) | 3.16× (3.07–3.25) | 3.53× (3.44–3.62) | 4.07× (3.98–4.16) | 3.87× (3.79–3.96) | 3.51× (3.45–3.56) | 3.58× (3.52–3.64) | 1.24× (1.21–1.28) | 1.36× (1.34–1.38) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 1.71× (1.64–1.77) | 2.61× (2.43–2.81) | 2.99× (2.82–3.18) | 3.26× (3.15–3.37) | 3.10× (3.03–3.17) | 2.30× (2.27–2.33) | 2.35× (2.31–2.39) | 0.82× (0.80–0.83) | 0.89× (0.88–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.80) | 11.08× (11.04–11.13) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.37) | 0.48× (0.43–0.53) | 0.73× (0.71–0.76) | 0.63× (0.60–0.65) | 5.84× (5.63–6.06) | 0.14× (0.14–0.15) | 0.15× (0.14–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.33–0.37) | 0.57× (0.47–0.69) | 0.62× (0.59–0.66) | 0.55× (0.53–0.57) | 3.64× (3.47–3.83) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.69–1.71) | 0.66× (0.66–0.67) | 8.02× (7.96–8.07) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.68–0.72) | 0.69× (0.65–0.73) | 1.68× (1.63–1.72) | 0.85× (0.82–0.87) | 6.09× (5.92–6.26) | 0.50× (0.48–0.51) | 0.52× (0.50–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.68× (0.65–0.70) | 0.77× (0.72–0.82) | 1.23× (1.18–1.28) | 0.76× (0.73–0.79) | 3.61× (3.47–3.75) | 0.28× (0.27–0.29) | 0.30× (0.28–0.31) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.72–0.73) | 5.18× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.89× (0.87–0.91) | 0.72× (0.70–0.74) | 3.09× (3.02–3.17) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.82× (0.79–0.84) | 2.55× (2.50–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.20× (5.19–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.47) | 0.38× (0.38–0.39) | 0.83× (0.82–0.85) | 0.67× (0.65–0.69) | 2.85× (2.79–2.91) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.48× (0.44–0.51) | 0.80× (0.79–0.81) | 0.82× (0.79–0.84) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.91× (0.90–0.91) | 6.21× (6.20–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.20× (6.19–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.40× (1.40–1.41) | 0.66× (0.65–0.66) | 6.42× (6.39–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 38 MB | 0.45× (0.43–0.46) | 0.41× (0.39–0.44) | 1.01× (0.98–1.04) | 0.67× (0.65–0.69) | 3.33× (3.23–3.43) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.15–0.16) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 54 MB | 0.50× (0.49–0.51) | 0.42× (0.40–0.44) | 0.93× (0.92–0.95) | 0.77× (0.75–0.79) | 2.38× (2.34–2.42) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 543 | 1.85 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 952 | 1.99 | 1.09× (1.07–1.10) | 88% | 1.05× (1.01–1.09) | 1.03× (1.01–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,156 | 3.18 | 1.71× (1.69–1.73) | 53% | 1.23× (1.18–1.28) | 1.63× (1.59–1.68) |

**Best:** mibz2 at 4 threads (1,156 MB/s); lbzip2 at 4 threads (236 MB/s); pbz2 at 4 threads (359 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.52 | 0.01 | 0.09 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 9.8% |
| MTF + RLE2 | 1.57 | 18.7% |
| T-vector build | 1.49 | 17.7% |
| walk (the one each build uses) | 4.08 | 48.4% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.43 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.061 | (in its total) | 2.70× (2.69–2.71) |  |
| walk | 1.044 | 2.050 | (in its total) | 1.96× (1.96–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.041 | (in its total) | 1.13× (1.11–1.14) |  |
| inverse BWT, total | 1.474 | 3.151 | 1.875 | 2.14× (2.13–2.15) | 1.27× (1.26–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m15s ± 6m45s |
| 4 threads | 26m35s ± 3m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m33s ± 2m40s |
| the scanner alone (its serial part) | 8m09s ± 32.2s |
| the workers alone, over 4 | 18m33s ± 2m40s |

At 4 threads the workers bound it. Per byte: the scanner 4.7 ns a byte of bz2, the workers 2.43 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,101,202 | 14,161,338 | \`702f0be27457325c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
