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

The latest record: [`d06d41c`](<https://github.com/lmmx/mibz2/commit/d06d41cce49db5bc14016aa8a5bbb5047309faad>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43616587665>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.411 s | 510 | 6.6 | 3.28× (3.21–3.36) | 7.48× (7.30–7.65) | 7.94× (7.77–8.11) | 8.79× (8.59–8.99) | 7.99× (7.82–8.17) | 8.35× (8.18–8.53) | 8.14× (7.97–8.31) | 2.57× (2.51–2.62) | 2.28× (2.22–2.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.232 s | 904 | 7.2 | 3.27× (3.20–3.34) | 6.97× (6.84–7.10) | 8.53× (8.32–8.73) | 8.33× (8.20–8.47) | 7.59× (7.45–7.73) | 14.85× (14.57–15.13) | 14.47× (14.24–14.71) | 4.56× (4.48–4.64) | 4.05× (3.98–4.12) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.198 s | 1,060 | 11.3 | 3.30× (3.19–3.42) | 6.00× (5.90–6.09) | 6.98× (6.91–7.06) | 6.92× (6.84–7.00) | 6.75× (6.68–6.82) | 17.56× (17.35–17.76) | 17.11× (16.90–17.33) | 5.39× (5.34–5.45) | 4.79× (4.74–4.83) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.012 s | 1,695 | 2.2 | 0.74× (0.68–0.79) | 4.11× (3.83–4.41) | 5.92× (5.52–6.35) | 4.65× (4.32–4.99) | 7.88× (7.32–8.48) | 3.63× (3.39–3.90) | 3.67× (3.42–3.94) | 1.19× (1.09–1.30) | 1.66× (1.55–1.79) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.013 s | 1,515 | 2.5 | 0.67× (0.64–0.71) | 3.81× (3.63–4.01) | 7.10× (6.84–7.36) | 4.34× (4.13–4.56) | 7.29× (6.92–7.68) | 3.37× (3.20–3.55) | 3.41× (3.24–3.58) | 1.10× (1.04–1.17) | 1.54× (1.46–1.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,465 | 2.5 | 0.65× (0.61–0.69) | 3.63× (3.42–3.86) | 6.80× (6.31–7.32) | 4.19× (3.95–4.45) | 7.03× (6.60–7.47) | 3.21× (3.02–3.40) | 3.24× (3.05–3.44) | 1.05× (0.97–1.14) | 1.47× (1.37–1.56) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 710 | 4.9 | 3.12× (3.01–3.23) | 3.62× (3.51–3.74) | 4.07× (3.93–4.21) | 4.62× (4.48–4.77) | 4.98× (4.84–5.13) | 3.39× (3.29–3.50) | 3.15× (3.05–3.25) | 2.11× (2.04–2.18) | 2.21× (2.14–2.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,089 | 5.7 | 2.69× (2.51–2.88) | 3.44× (3.23–3.66) | 4.65× (4.38–4.93) | 4.32× (3.83–4.87) | 5.65× (5.19–6.15) | 5.34× (5.03–5.67) | 4.96× (4.66–5.27) | 3.33× (3.11–3.56) | 3.48× (3.24–3.73) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,174 | 10.4 | 2.33× (2.23–2.43) | 2.52× (2.44–2.61) | 3.38× (3.26–3.51) | 2.94× (2.74–3.16) | 4.77× (4.60–4.94) | 5.73× (5.55–5.92) | 5.32× (5.14–5.50) | 3.57× (3.45–3.69) | 3.73× (3.60–3.87) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 72 | 47.0 | 1.39× (1.38–1.40) | 3.07× (3.04–3.11) | 2.77× (2.74–2.81) | 2.88× (2.86–2.90) | 3.18× (3.16–3.20) | 3.51× (3.48–3.53) | 3.64× (3.62–3.67) | 1.28× (1.27–1.28) | 1.28× (1.28–1.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 122 | 50.9 | 1.36× (1.33–1.39) | 2.98× (2.89–3.08) | 2.89× (2.81–2.98) | 3.50× (3.43–3.58) | 3.25× (3.19–3.31) | 5.88× (5.76–6.01) | 6.11× (5.99–6.24) | 2.14× (2.10–2.18) | 2.15× (2.11–2.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.054 s | 147 | 69.2 | 1.39× (1.34–1.44) | 2.81× (2.73–2.89) | 2.68× (2.58–2.79) | 3.24× (3.13–3.36) | 3.25× (3.16–3.34) | 7.10× (6.88–7.33) | 7.38× (7.16–7.60) | 2.58× (2.50–2.66) | 2.60× (2.51–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.122 s | 79 | 43.2 | 1.44× (1.43–1.45) | 3.47× (3.43–3.51) | 2.92× (2.91–2.93) | 3.05× (3.03–3.07) | 3.42× (3.40–3.44) | 4.16× (4.15–4.18) | 4.74× (4.72–4.76) | 1.49× (1.48–1.50) | 1.55× (1.52–1.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 47.2 | 1.31× (1.27–1.34) | 3.13× (3.05–3.21) | 2.87× (2.80–2.94) | 3.84× (3.77–3.92) | 3.30× (3.22–3.39) | 6.89× (6.73–7.06) | 7.84× (7.64–8.05) | 2.47× (2.41–2.53) | 2.56× (2.47–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.059 s | 162 | 66.2 | 1.48× (1.45–1.51) | 3.06× (2.98–3.13) | 2.88× (2.81–2.95) | 3.69× (3.58–3.79) | 3.34× (3.26–3.41) | 8.57× (8.40–8.75) | 9.75× (9.56–9.95) | 3.07× (3.01–3.13) | 3.18× (3.12–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.008 s | 1,212 | 3.2 | 0.75× (0.72–0.78) | 3.65× (3.52–3.80) | 5.13× (4.94–5.33) | 3.91× (3.76–4.05) | 8.23× (7.89–8.58) | 3.03× (2.91–3.14) | 3.07× (2.96–3.20) | 1.11× (1.06–1.15) | 1.24× (1.19–1.29) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,010 | 3.8 | 0.61× (0.59–0.63) | 3.15× (3.01–3.28) | 5.84× (5.57–6.12) | 3.43× (3.26–3.61) | 7.13× (6.82–7.46) | 2.60× (2.49–2.71) | 2.64× (2.53–2.76) | 0.95× (0.91–1.00) | 1.07× (1.02–1.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,000 | 3.8 | 0.60× (0.58–0.62) | 3.07× (2.97–3.17) | 5.48× (5.11–5.89) | 3.38× (3.26–3.49) | 7.07× (6.76–7.39) | 2.53× (2.46–2.61) | 2.57× (2.50–2.65) | 0.93× (0.89–0.97) | 1.04× (1.01–1.07) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.055 s | 224 | 15.3 | 1.88× (1.87–1.89) | 4.43× (4.40–4.46) | 4.83× (4.79–4.86) | 4.94× (4.90–4.98) | 5.06× (5.02–5.11) | 5.32× (5.29–5.35) | 5.23× (5.20–5.26) | 1.54× (1.53–1.54) | 1.48× (1.47–1.49) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.036 s | 344 | 18.2 | 1.69× (1.50–1.89) | 3.73× (3.64–3.83) | 4.53× (4.40–4.67) | 5.02× (4.88–5.16) | 4.74× (4.64–4.84) | 8.25× (8.07–8.43) | 8.11× (7.94–8.29) | 2.38× (2.33–2.43) | 2.29× (2.24–2.34) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.030 s | 406 | 27.2 | 1.64× (1.59–1.69) | 3.13× (3.04–3.23) | 3.82× (3.66–3.99) | 4.49× (4.37–4.60) | 4.39× (4.27–4.52) | 9.64× (9.43–9.86) | 9.48× (9.29–9.69) | 2.78× (2.72–2.84) | 2.68× (2.62–2.74) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.28× (3.20–3.35) | 7.45× (7.28–7.62) | 7.91× (7.74–8.08) | 8.77× (8.57–8.97) | 7.97× (7.80–8.15) | 8.33× (8.16–8.50) | 8.12× (7.95–8.29) | 2.56× (2.51–2.62) | 2.26× (2.21–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.14× (3.10–3.17) | 7.16× (7.05–7.26) | 7.94× (7.83–8.04) | 8.58× (8.49–8.68) | 7.49× (7.41–7.58) | 7.68× (7.58–7.79) | 7.49× (7.40–7.58) | 2.36× (2.34–2.39) | 2.08× (2.06–2.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.21× (3.14–3.27) | 6.56× (6.42–6.70) | 6.97× (6.83–7.12) | 7.61× (7.45–7.77) | 6.83× (6.69–6.97) | 4.91× (4.81–5.02) | 4.79× (4.69–4.89) | 1.51× (1.48–1.54) | 1.33× (1.31–1.36) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.76× (0.70–0.82) | 3.67× (3.42–3.95) | 4.83× (4.50–5.18) | 4.31× (4.02–4.63) | 7.26× (6.75–7.81) | 3.40× (3.17–3.65) | 3.43× (3.20–3.68) | 1.17× (1.07–1.28) | 1.00× (0.94–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.71× (0.67–0.74) | 3.47× (3.31–3.64) | 6.43× (6.13–6.75) | 4.03× (3.84–4.22) | 6.70× (6.37–7.03) | 3.14× (2.99–3.29) | 3.17× (3.03–3.31) | 1.08× (1.02–1.14) | 0.92× (0.88–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.67× (0.64–0.72) | 3.78× (3.62–3.95) | 6.16× (5.83–6.52) | 3.95× (3.74–4.17) | 6.53× (6.16–6.92) | 3.02× (2.86–3.20) | 3.06× (2.89–3.23) | 1.04× (0.96–1.12) | 0.89× (0.84–0.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.04× (2.94–3.14) | 3.34× (3.23–3.45) | 3.87× (3.75–4.00) | 4.48× (4.34–4.63) | 4.82× (4.68–4.97) | 3.30× (3.20–3.40) | 3.06× (2.97–3.16) | 2.07× (2.00–2.14) | 2.04× (1.99–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.84× (2.77–2.92) | 3.10× (3.01–3.19) | 3.76× (3.66–3.86) | 4.05× (3.95–4.14) | 4.35× (4.25–4.44) | 2.89× (2.83–2.95) | 2.68× (2.62–2.75) | 1.81× (1.77–1.86) | 1.78× (1.75–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.43× (2.34–2.52) | 2.08× (2.01–2.15) | 2.57× (2.48–2.67) | 2.56× (2.49–2.64) | 2.63× (2.56–2.70) | 1.61× (1.55–1.66) | 1.49× (1.44–1.54) | 1.01× (0.98–1.04) | 0.99× (0.97–1.01) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.38–1.39) | 3.01× (2.99–3.02) | 2.73× (2.71–2.76) | 2.86× (2.84–2.88) | 3.16× (3.14–3.18) | 3.48× (3.46–3.50) | 3.62× (3.59–3.64) | 1.27× (1.27–1.28) | 1.25× (1.24–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.33× (1.31–1.36) | 2.94× (2.89–2.99) | 2.67× (2.62–2.71) | 3.36× (3.30–3.42) | 2.99× (2.94–3.04) | 3.20× (3.14–3.26) | 3.32× (3.26–3.38) | 1.17× (1.15–1.19) | 1.14× (1.12–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.59× (1.52–1.66) | 3.02× (2.94–3.09) | 2.60× (2.49–2.71) | 3.32× (3.21–3.43) | 3.05× (2.95–3.15) | 2.40× (2.33–2.47) | 2.49× (2.42–2.57) | 0.88× (0.85–0.90) | 0.86× (0.83–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.45) | 3.40× (3.38–3.42) | 2.86× (2.85–2.88) | 3.03× (3.01–3.05) | 3.39× (3.38–3.41) | 4.13× (4.12–4.15) | 4.70× (4.68–4.72) | 1.49× (1.48–1.49) | 1.49× (1.48–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.32–1.39) | 3.23× (3.17–3.30) | 2.76× (2.69–2.82) | 3.70× (3.61–3.80) | 3.16× (3.09–3.23) | 3.73× (3.65–3.83) | 4.25× (4.14–4.36) | 1.34× (1.31–1.38) | 1.34× (1.31–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.65× (1.61–1.70) | 3.31× (3.21–3.42) | 2.57× (2.49–2.65) | 3.46× (3.37–3.55) | 3.12× (3.04–3.20) | 2.72× (2.65–2.79) | 3.09× (3.01–3.17) | 0.98× (0.95–1.00) | 0.98× (0.95–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.78× (0.75–0.81) | 3.05× (2.95–3.17) | 4.05× (3.90–4.21) | 3.54× (3.42–3.67) | 7.33× (7.03–7.65) | 2.78× (2.67–2.88) | 2.81× (2.71–2.92) | 1.09× (1.04–1.13) | 0.92× (0.89–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.66× (0.63–0.68) | 2.63× (2.53–2.73) | 4.90× (4.75–5.05) | 3.15× (3.01–3.30) | 6.36× (6.11–6.63) | 2.38× (2.29–2.47) | 2.41× (2.32–2.51) | 0.93× (0.89–0.97) | 0.79× (0.76–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.64× (0.62–0.66) | 2.96× (2.80–3.12) | 4.83× (4.66–5.00) | 3.20× (3.10–3.31) | 6.50× (6.23–6.77) | 2.38× (2.32–2.45) | 2.41× (2.35–2.48) | 0.93× (0.89–0.97) | 0.79× (0.76–0.81) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.87× (1.85–1.88) | 4.23× (4.19–4.27) | 4.68× (4.65–4.72) | 4.86× (4.82–4.90) | 4.98× (4.93–5.03) | 5.24× (5.20–5.27) | 5.15× (5.12–5.18) | 1.53× (1.52–1.53) | 1.41× (1.40–1.42) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.67× (1.65–1.70) | 3.77× (3.72–3.82) | 4.39× (4.32–4.46) | 4.64× (4.58–4.71) | 4.34× (4.29–4.40) | 4.43× (4.37–4.48) | 4.36× (4.31–4.41) | 1.29× (1.28–1.31) | 1.20× (1.18–1.21) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 1.88× (1.79–1.99) | 3.48× (3.36–3.61) | 3.86× (3.73–3.99) | 4.12× (4.00–4.24) | 3.95× (3.83–4.07) | 2.99× (2.91–3.08) | 2.94× (2.86–3.03) | 0.87× (0.85–0.90) | 0.81× (0.79–0.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.24× (0.22–0.26) | 0.81× (0.80–0.81) | 0.64× (0.64–0.65) | 9.11× (9.10–9.13) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 75 MB | 0.25× (0.23–0.27) | 0.33× (0.28–0.40) | 0.52× (0.48–0.56) | 0.43× (0.39–0.46) | 4.08× (3.81–4.37) | 0.10× (0.09–0.10) | 0.11× (0.10–0.11) | 0.16× (0.15–0.17) | 0.08× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 129 MB | 0.24× (0.21–0.26) | 0.38× (0.30–0.48) | 0.42× (0.37–0.46) | 0.37× (0.33–0.41) | 2.46× (2.21–2.75) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.09× (0.08–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.41–0.44) | 1.70× (1.69–1.71) | 0.65× (0.65–0.66) | 8.03× (7.99–8.06) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.62–0.67) | 0.60× (0.53–0.68) | 1.57× (1.51–1.63) | 0.78× (0.75–0.80) | 5.67× (5.47–5.87) | 0.46× (0.44–0.48) | 0.48× (0.47–0.50) | 0.53× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.61–0.65) | 0.71× (0.65–0.77) | 1.14× (1.10–1.17) | 0.70× (0.68–0.72) | 3.37× (3.28–3.47) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.53) | 0.42× (0.40–0.44) | 0.90× (0.86–0.94) | 0.71× (0.66–0.76) | 3.10× (2.95–3.25) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.44–0.47) | 0.87× (0.84–0.89) | 0.80× (0.78–0.83) | 2.51× (2.45–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.38× (0.36–0.40) | 0.85× (0.83–0.87) | 0.71× (0.69–0.73) | 2.89× (2.82–2.96) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.48× (0.44–0.51) | 0.82× (0.81–0.82) | 0.82× (0.79–0.85) | 2.34× (2.32–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.29–5.32) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.33× (5.31–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.66–0.67) | 6.43× (6.41–6.46) | 0.39× (0.39–0.39) | 0.43× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 36 MB | 0.46× (0.45–0.47) | 0.39× (0.36–0.42) | 1.06× (1.03–1.09) | 0.68× (0.67–0.70) | 3.33× (3.26–3.41) | 0.20× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 56 MB | 0.50× (0.48–0.52) | 0.46× (0.41–0.51) | 0.92× (0.90–0.94) | 0.74× (0.72–0.75) | 2.34× (2.28–2.39) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 510 | 1.97 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 904 | 2.15 | 1.08× (1.05–1.12) | 89% | 1.04× (1.03–1.05) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,060 | 3.37 | 1.70× (1.64–1.75) | 53% | 1.49× (1.48–1.50) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (1,060 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (326 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.15, lbzip2 1t 0.04, lbzip2 2t 0.16, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.54 | 0.02 | 0.09 | 0.198 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3364 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.97 | 10.5% |
| MTF + RLE2 | 1.40 | 15.0% |
| T-vector build | 1.35 | 14.5% |
| walk (the one each build uses) | 5.24 | 56.4% |
| of which: segment assembly | 0.27 | 2.8% |
| RLE1 | 0.22 | 2.4% |
| CRC | 0.12 | 1.3% |
| **total** | 9.29 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.372 | 0.931 | (in its total) | 2.51× (2.50–2.51) |  |
| walk | 1.477 | 2.816 | (in its total) | 1.91× (1.90–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.92× (0.91–0.92) |  |
| inverse BWT, total | 1.928 | 3.820 | 1.987 | 1.98× (1.98–1.99) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 7m08s |
| 4 threads | 28m45s ± 4m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m17s ± 2m45s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 19m17s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 2.53 ns a byte of output at 4 threads (CPU, summed over them), 1.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,439,877 | 12,292,642 | \`3897e199bbd7462e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3363.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
