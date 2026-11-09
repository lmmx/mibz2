# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-270/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`d245a2d`](<https://github.com/lmmx/mibz2/commit/d245a2d853209ccac9501d2724192e106d29c096>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42546541673>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.356 s | 590 | 7.3 | 2.31× (2.25–2.38) | 5.83× (5.65–6.02) | 6.06× (5.86–6.26) | 6.95× (6.74–7.16) | 6.18× (5.98–6.39) | 6.54× (6.35–6.73) | 6.03× (5.84–6.23) | 1.84× (1.78–1.91) | 2.14× (2.07–2.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.214 s | 982 | 8.5 | 2.31× (2.11–2.52) | 4.97× (4.43–5.57) | 5.97× (5.30–6.73) | 6.04× (5.35–6.82) | 5.38× (4.77–6.06) | 10.47× (9.27–11.83) | 9.66× (8.60–10.86) | 2.96× (2.64–3.31) | 3.43× (3.07–3.84) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.198 s | 1,059 | 15.0 | 2.19× (2.06–2.32) | 3.36× (3.19–3.54) | 4.53× (4.21–4.88) | 4.02× (3.89–4.15) | 4.00× (3.80–4.21) | 11.79× (11.35–12.26) | 10.88× (10.45–11.34) | 3.33× (3.18–3.48) | 3.87× (3.72–4.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,404 | 3.4 | 0.57× (0.55–0.58) | 3.62× (3.50–3.74) | 4.58× (4.35–4.82) | 3.77× (3.62–3.94) | 6.60× (6.37–6.84) | 2.80× (2.70–2.90) | 2.81× (2.70–2.92) | 1.04× (0.98–1.11) | 2.17× (2.10–2.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,360 | 3.6 | 0.50× (0.49–0.52) | 2.85× (2.68–3.03) | 5.70× (5.53–5.87) | 3.72× (3.46–4.00) | 6.20× (6.02–6.39) | 2.70× (2.64–2.76) | 2.70× (2.63–2.77) | 1.00× (0.94–1.07) | 2.09× (2.03–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,379 | 3.4 | 0.51× (0.50–0.52) | 2.97× (2.62–3.36) | 5.71× (5.58–5.83) | 3.66× (3.59–3.72) | 6.39× (6.18–6.60) | 2.70× (2.63–2.77) | 2.71× (2.64–2.78) | 1.01× (0.93–1.09) | 2.09× (2.04–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 998 | 4.6 | 2.67× (2.60–2.75) | 3.46× (3.29–3.64) | 4.01× (3.80–4.22) | 4.35× (4.24–4.46) | 5.12× (4.95–5.31) | 2.65× (2.58–2.71) | 2.44× (2.36–2.51) | 2.13× (1.90–2.39) | 3.10× (3.02–3.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,400 | 6.1 | 2.06× (1.94–2.19) | 2.94× (2.71–3.20) | 3.97× (3.68–4.28) | 3.62× (3.37–3.89) | 5.39× (5.07–5.74) | 3.66× (3.42–3.92) | 3.37× (3.14–3.62) | 2.94× (2.58–3.36) | 4.28× (4.01–4.56) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,192 | 11.9 | 1.64× (1.58–1.70) | 1.90× (1.82–1.98) | 2.72× (2.61–2.83) | 2.11× (1.97–2.26) | 4.13× (3.92–4.35) | 3.14× (2.99–3.29) | 2.89× (2.76–3.02) | 2.52× (2.23–2.86) | 3.67× (3.50–3.85) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.078 s | 102 | 42.7 | 1.44× (1.39–1.49) | 3.19× (3.10–3.28) | 2.85× (2.78–2.92) | 2.87× (2.80–2.93) | 3.18× (3.12–3.25) | 3.72× (3.63–3.82) | 3.57× (3.50–3.63) | 1.41× (1.35–1.46) | 1.55× (1.53–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 156 | 49.6 | 1.26× (1.21–1.31) | 2.84× (2.71–2.97) | 2.60× (2.50–2.69) | 3.30× (3.18–3.43) | 3.12× (3.02–3.23) | 5.71× (5.48–5.95) | 5.47× (5.27–5.67) | 2.16× (2.06–2.26) | 2.38× (2.28–2.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 191 | 66.9 | 1.23× (1.01–1.50) | 2.33× (1.96–2.76) | 2.20× (1.84–2.64) | 2.74× (2.24–3.35) | 2.91× (2.42–3.50) | 6.44× (5.34–7.77) | 6.17× (5.13–7.40) | 2.43× (2.01–2.95) | 2.68× (2.22–3.24) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.085 s | 114 | 38.3 | 1.42× (1.35–1.50) | 3.57× (3.37–3.78) | 2.97× (2.78–3.18) | 3.05× (2.88–3.22) | 3.40× (3.17–3.65) | 4.53× (4.30–4.77) | 4.60× (4.35–4.87) | 1.57× (1.50–1.64) | 1.83× (1.73–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.054 s | 177 | 43.8 | 1.27× (1.12–1.45) | 3.06× (2.66–3.52) | 2.61× (2.31–2.96) | 3.69× (3.23–4.21) | 3.16× (2.81–3.55) | 6.93× (6.12–7.84) | 7.04× (6.24–7.93) | 2.40× (2.13–2.70) | 2.80× (2.45–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 205 | 62.3 | 1.35× (1.31–1.40) | 2.57× (2.47–2.67) | 2.54× (2.43–2.64) | 3.29× (3.17–3.42) | 3.09× (3.03–3.16) | 8.42× (8.18–8.66) | 8.55× (8.24–8.87) | 2.91× (2.82–3.02) | 3.41× (3.29–3.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,058 | 4.7 | 0.62× (0.61–0.63) | 3.41× (3.01–3.87) | 4.51× (4.43–4.59) | 3.15× (3.04–3.26) | 7.13× (6.82–7.46) | 2.37× (2.30–2.45) | 2.33× (2.28–2.39) | 0.98× (0.96–1.01) | 2.17× (2.13–2.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 990 | 5.1 | 0.50× (0.49–0.51) | 3.06× (3.03–3.09) | 5.29× (5.24–5.34) | 3.21× (2.86–3.61) | 6.71× (6.57–6.84) | 2.24× (2.18–2.29) | 2.20× (2.16–2.24) | 0.92× (0.91–0.94) | 2.04× (2.02–2.06) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 976 | 5.0 | 0.50× (0.49–0.51) | 3.03× (2.95–3.10) | 5.20× (5.07–5.33) | 3.00× (2.93–3.07) | 6.80× (6.57–7.05) | 2.20× (2.13–2.28) | 2.17× (2.10–2.24) | 0.91× (0.89–0.93) | 2.01× (1.96–2.06) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 0.054 s | 231 | 19.1 | 1.29× (1.27–1.31) | 3.22× (3.15–3.30) | 3.44× (3.35–3.52) | 3.60× (3.48–3.72) | 3.88× (3.74–4.03) | 4.14× (4.04–4.25) | 3.60× (3.48–3.73) | 1.18× (1.09–1.28) | 1.32× (1.30–1.34) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 0.035 s | 354 | 22.5 | 1.13× (1.00–1.28) | 2.67× (2.38–3.01) | 3.18× (2.83–3.56) | 3.37× (3.05–3.71) | 3.56× (3.23–3.91) | 6.04× (5.48–6.64) | 5.25× (4.70–5.87) | 1.72× (1.50–1.98) | 1.92× (1.74–2.12) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 0.030 s | 413 | 33.6 | 1.20× (1.10–1.32) | 2.21× (2.09–2.33) | 2.73× (2.63–2.83) | 2.99× (2.77–3.24) | 3.43× (3.32–3.56) | 7.27× (6.95–7.60) | 6.32× (5.97–6.69) | 2.07× (1.87–2.30) | 2.31× (2.21–2.43) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.31× (2.24–2.37) | 5.80× (5.62–5.98) | 6.02× (5.83–6.22) | 6.92× (6.72–7.13) | 6.16× (5.95–6.37) | 6.51× (6.32–6.70) | 6.01× (5.82–6.20) | 1.84× (1.78–1.91) | 2.13× (2.05–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.13× (2.07–2.19) | 5.27× (5.11–5.43) | 5.64× (5.46–5.82) | 6.52× (6.34–6.71) | 5.56× (5.38–5.75) | 5.78× (5.56–6.00) | 5.33× (5.18–5.49) | 1.63× (1.60–1.67) | 1.89× (1.82–1.96) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.00× (1.88–2.11) | 3.51× (3.34–3.70) | 4.01× (3.70–4.34) | 4.22× (4.05–4.40) | 3.63× (3.43–3.84) | 3.18× (3.04–3.34) | 2.94× (2.79–3.09) | 0.90× (0.85–0.95) | 1.04× (1.00–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.64) | 3.12× (3.00–3.24) | 3.97× (3.85–4.10) | 3.51× (3.35–3.68) | 6.07× (5.82–6.32) | 2.63× (2.53–2.74) | 2.64× (2.53–2.75) | 1.06× (0.98–1.14) | 1.84× (1.78–1.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.58) | 2.80× (2.74–2.86) | 4.88× (4.73–5.03) | 3.45× (3.20–3.72) | 5.62× (5.46–5.79) | 2.50× (2.45–2.54) | 2.50× (2.45–2.55) | 1.00× (0.92–1.09) | 1.75× (1.71–1.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.57) | 3.30× (2.78–3.92) | 5.01× (4.89–5.14) | 3.45× (3.38–3.51) | 5.89× (5.70–6.09) | 2.56× (2.49–2.62) | 2.56× (2.49–2.63) | 1.03× (0.94–1.13) | 1.79× (1.75–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.54× (2.47–2.62) | 2.98× (2.89–3.08) | 3.56× (3.44–3.69) | 4.10× (3.99–4.21) | 4.82× (4.65–4.99) | 2.53× (2.46–2.60) | 2.33× (2.25–2.40) | 2.05× (1.82–2.30) | 2.63× (2.53–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.03–2.14) | 2.47× (2.41–2.54) | 2.93× (2.88–2.98) | 3.27× (3.19–3.35) | 3.77× (3.65–3.88) | 1.93× (1.87–1.99) | 1.78× (1.72–1.83) | 1.56× (1.40–1.74) | 2.01× (1.92–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.84× (1.76–1.92) | 1.67× (1.58–1.77) | 1.98× (1.89–2.07) | 2.00× (1.93–2.07) | 2.15× (2.07–2.23) | 0.99× (0.95–1.02) | 0.91× (0.88–0.94) | 0.80× (0.71–0.90) | 1.03× (0.98–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.39–1.49) | 3.08× (2.99–3.17) | 2.77× (2.71–2.83) | 2.83× (2.77–2.90) | 3.14× (3.08–3.20) | 3.67× (3.58–3.76) | 3.51× (3.45–3.58) | 1.40× (1.34–1.46) | 1.48× (1.45–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.21–1.28) | 2.78× (2.72–2.83) | 2.40× (2.35–2.45) | 3.16× (3.08–3.24) | 2.75× (2.67–2.82) | 3.15× (3.09–3.21) | 3.01× (2.95–3.07) | 1.20× (1.15–1.25) | 1.27× (1.23–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.37× (1.12–1.66) | 2.43× (2.05–2.88) | 1.99× (1.66–2.38) | 2.66× (2.19–3.23) | 2.45× (2.04–2.94) | 2.17× (1.79–2.63) | 2.08× (1.73–2.50) | 0.83× (0.68–1.00) | 0.87× (0.72–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.42× (1.34–1.49) | 3.46× (3.27–3.67) | 2.88× (2.71–3.07) | 3.01× (2.84–3.18) | 3.36× (3.13–3.60) | 4.47× (4.24–4.70) | 4.54× (4.29–4.80) | 1.56× (1.49–1.63) | 1.74× (1.65–1.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.28× (1.12–1.45) | 3.10× (2.68–3.58) | 2.46× (2.15–2.81) | 3.44× (3.01–3.93) | 2.87× (2.52–3.26) | 3.78× (3.29–4.33) | 3.84× (3.36–4.39) | 1.32× (1.16–1.50) | 1.48× (1.29–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.47–1.60) | 2.88× (2.77–3.00) | 2.20× (2.12–2.29) | 3.08× (2.93–3.24) | 2.81× (2.70–2.92) | 2.81× (2.72–2.91) | 2.86× (2.73–2.99) | 0.98× (0.95–1.02) | 1.10× (1.05–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.70) | 2.67× (2.37–3.02) | 3.38× (3.26–3.50) | 2.87× (2.78–2.96) | 6.30× (6.03–6.58) | 2.19× (2.13–2.27) | 2.15× (2.10–2.21) | 0.98× (0.96–1.01) | 1.51× (1.47–1.56) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.30× (2.26–2.34) | 3.98× (3.91–4.04) | 2.96× (2.60–3.38) | 5.83× (5.71–5.95) | 2.02× (1.97–2.08) | 1.99× (1.95–2.02) | 0.91× (0.89–0.92) | 1.40× (1.38–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.51× (2.46–2.56) | 4.03× (3.89–4.17) | 2.83× (2.77–2.89) | 6.12× (5.91–6.33) | 2.07× (2.00–2.14) | 2.03× (1.96–2.10) | 0.93× (0.91–0.95) | 1.43× (1.39–1.47) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 1.28× (1.27–1.30) | 3.05× (3.00–3.11) | 3.27× (3.21–3.33) | 3.52× (3.40–3.65) | 3.81× (3.66–3.96) | 4.06× (3.95–4.17) | 3.53× (3.41–3.65) | 1.18× (1.09–1.27) | 1.24× (1.22–1.26) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 1.16× (1.14–1.19) | 2.69× (2.57–2.83) | 2.98× (2.82–3.15) | 3.31× (3.23–3.39) | 3.19× (3.11–3.28) | 3.38× (3.26–3.50) | 2.94× (2.84–3.03) | 0.98× (0.91–1.05) | 1.03× (1.00–1.05) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 1.30× (1.16–1.46) | 2.17× (2.10–2.24) | 2.42× (2.35–2.49) | 2.66× (2.61–2.71) | 2.64× (2.59–2.69) | 2.27× (2.21–2.33) | 1.97× (1.88–2.07) | 0.66× (0.60–0.72) | 0.69× (0.67–0.71) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.36× (0.35–0.38) | 1.08× (1.08–1.08) | 0.86× (0.84–0.88) | 12.18× (12.16–12.19) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.37× (0.37–0.38) | 0.55× (0.46–0.66) | 0.78× (0.77–0.80) | 0.67× (0.65–0.68) | 6.30× (6.19–6.41) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.60× (0.50–0.71) | 0.71× (0.70–0.72) | 0.61× (0.60–0.62) | 4.08× (4.06–4.10) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.30–4.31) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.44× (4.44–4.44) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.17) | 1.79× (1.79–1.79) | 0.84× (0.83–0.84) | 4.43× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.67) | 8.15× (8.12–8.17) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.63–0.71) | 0.67× (0.61–0.74) | 1.60× (1.54–1.66) | 0.79× (0.76–0.83) | 5.79× (5.58–6.00) | 0.46× (0.44–0.47) | 0.48× (0.46–0.50) | 0.52× (0.51–0.54) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.61× (0.59–0.63) | 0.71× (0.66–0.76) | 1.13× (1.09–1.17) | 0.74× (0.72–0.75) | 3.30× (3.20–3.41) | 0.25× (0.24–0.26) | 0.26× (0.26–0.27) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.25× (5.25–5.26) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.41× (0.38–0.43) | 0.92× (0.87–0.96) | 0.72× (0.68–0.77) | 3.17× (3.02–3.32) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.46× (0.45–0.46) | 0.87× (0.86–0.89) | 0.81× (0.79–0.84) | 2.54× (2.49–2.58) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 39 MB | 0.50× (0.49–0.52) | 0.41× (0.39–0.43) | 0.90× (0.88–0.92) | 0.75× (0.72–0.77) | 3.08× (3.01–3.16) | 0.19× (0.18–0.19) | 0.20× (0.20–0.20) | 0.30× (0.30–0.31) | 0.16× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.61) | 0.48× (0.45–0.52) | 0.80× (0.79–0.82) | 0.81× (0.79–0.84) | 2.32× (2.28–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.23–0.25) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.77) | 5.40× (5.39–5.42) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.42× (5.40–5.44) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.67× (0.67–0.67) | 6.76× (6.75–6.78) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 34 MB | 0.47× (0.45–0.49) | 0.46× (0.43–0.51) | 1.17× (1.13–1.21) | 0.72× (0.69–0.75) | 3.61× (3.52–3.71) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 50 MB | 0.54× (0.52–0.56) | 0.50× (0.46–0.55) | 1.03× (1.00–1.05) | 0.81× (0.79–0.84) | 2.59× (2.52–2.66) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 590 | 1.70 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 982 | 1.98 | 1.13× (1.08–1.17) | 80% | 1.02× (1.00–1.05) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,059 | 3.51 | 2.04× (1.99–2.10) | 45% | 1.24× (1.19–1.29) | 1.77× (1.67–1.87) |

**Best:** mibz2 at 4 threads (1,059 MB/s); lbzip2 at 4 threads (316 MB/s); pbz2 at 4 threads (484 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.13, lbzip2 1t 0.07, lbzip2 2t 0.33, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.53, pbz2 4t 0.58. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.58 | 0.01 | 0.09 | 0.198 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4278 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 14.1% |
| MTF + RLE2 | 1.48 | 20.6% |
| T-vector build | 1.48 | 20.5% |
| walk (the one each build uses) | 2.82 | 39.3% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.25 | 3.5% |
| **total** | 7.18 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.332 | 0.723 | (in its total) | 2.15× (2.06–2.25) |  |
| walk | 0.678 | 1.627 | (in its total) | 2.38× (2.27–2.50) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.032 | 0.033 | (in its total) | 1.06× (0.98–1.16) |  |
| inverse BWT, total | 1.042 | 2.383 | 1.331 | 2.27× (2.16–2.38) | 1.26× (1.21–1.32) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 52m23s ± 6m13s |
| 4 threads | 29m03s ± 4m13s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m53s ± 3m08s |
| the scanner alone (its serial part) | 9m19s ± 32.2s |
| the workers alone, over 4 | 20m53s ± 3m08s |

At 4 threads the workers bound it. Per byte: the scanner 5.4 ns a byte of bz2, the workers 2.74 ns a byte of output at 4 threads (CPU, summed over them), 1.41 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,094,880 | 12,489,382 | \`0cafc9c0ea86fcad…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4277.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
