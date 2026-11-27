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

The latest record: [`eb1dd22`](<https://github.com/lmmx/mibz2/commit/eb1dd22c00da8e58a69dd1614385c933e6e846be>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46077259916>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 546 | 4.8 | 3.12× (3.00–3.25) | 7.09× (6.85–7.35) | 7.36× (7.08–7.65) | 8.38× (8.08–8.70) | 7.39× (7.12–7.66) | 7.69× (7.42–7.97) | 7.43× (7.16–7.71) | 2.30× (2.21–2.38) | 2.59× (2.49–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 956 | 5.1 | 3.25× (3.09–3.42) | 6.63× (6.39–6.89) | 7.72× (7.41–8.05) | 8.03× (7.80–8.28) | 7.06× (6.89–7.23) | 13.59× (13.21–13.98) | 13.12× (12.75–13.51) | 4.06× (3.94–4.18) | 4.57× (4.44–4.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.179 s | 1,170 | 8.2 | 3.25× (3.18–3.33) | 4.80× (4.73–4.86) | 6.00× (5.82–6.18) | 5.85× (5.76–5.94) | 5.78× (5.70–5.87) | 16.70× (16.47–16.94) | 16.13× (15.89–16.37) | 4.99× (4.92–5.05) | 5.62× (5.55–5.69) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,075 | 2.6 | 0.56× (0.55–0.57) | 3.28× (3.21–3.35) | 4.37× (4.28–4.47) | 3.44× (3.37–3.51) | 6.02× (5.90–6.15) | 2.55× (2.49–2.60) | 2.54× (2.49–2.60) | 0.94× (0.91–0.96) | 1.89× (1.69–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,055 | 2.8 | 0.51× (0.50–0.52) | 2.69× (2.64–2.74) | 5.39× (5.29–5.50) | 3.39× (3.32–3.45) | 5.72× (5.62–5.82) | 2.50× (2.46–2.55) | 2.50× (2.45–2.55) | 0.92× (0.90–0.94) | 1.86× (1.65–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,002 | 2.9 | 0.49× (0.47–0.50) | 2.57× (2.49–2.64) | 5.12× (4.98–5.28) | 3.22× (3.13–3.31) | 5.54× (5.42–5.66) | 2.38× (2.31–2.45) | 2.37× (2.31–2.45) | 0.87× (0.83–0.91) | 1.77× (1.60–1.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 763 | 3.6 | 2.76× (2.69–2.83) | 3.15× (3.07–3.22) | 4.01× (3.81–4.22) | 4.19× (4.09–4.30) | 4.72× (4.59–4.86) | 2.84× (2.77–2.91) | 2.32× (2.27–2.38) | 2.13× (2.08–2.18) | 3.13× (3.06–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,143 | 4.4 | 2.37× (1.99–2.84) | 2.86× (2.51–3.25) | 4.31× (3.80–4.90) | 3.61× (3.14–4.14) | 5.13× (4.48–5.87) | 4.10× (3.60–4.66) | 3.36× (2.95–3.81) | 3.07× (2.70–3.50) | 4.52× (3.98–5.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,086 | 8.6 | 2.09× (2.02–2.16) | 2.04× (1.88–2.21) | 3.40× (3.19–3.62) | 2.47× (2.31–2.64) | 4.60× (4.44–4.77) | 4.05× (3.90–4.21) | 3.32× (3.19–3.45) | 3.04× (2.92–3.16) | 4.48× (4.30–4.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 79 | 33.5 | 1.47× (1.41–1.52) | 2.81× (2.71–2.92) | 2.56× (2.46–2.66) | 2.65× (2.56–2.75) | 3.01× (2.90–3.12) | 3.17× (3.05–3.30) | 3.36× (3.23–3.48) | 1.33× (1.29–1.37) | 1.47× (1.41–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 129 | 37.8 | 1.41× (1.37–1.45) | 2.80× (2.58–3.03) | 2.63× (2.55–2.71) | 3.43× (3.32–3.53) | 3.17× (3.10–3.24) | 5.31× (5.18–5.45) | 5.62× (5.48–5.77) | 2.23× (2.18–2.28) | 2.46× (2.40–2.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 161 | 48.1 | 1.52× (1.48–1.56) | 2.34× (2.28–2.41) | 2.38× (2.29–2.47) | 2.97× (2.91–3.04) | 3.32× (3.23–3.40) | 6.66× (6.49–6.85) | 7.05× (6.86–7.24) | 2.79× (2.73–2.86) | 3.09× (3.00–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.110 s | 87 | 30.2 | 1.63× (1.62–1.64) | 3.28× (3.23–3.33) | 2.86× (2.83–2.89) | 2.97× (2.95–3.00) | 3.41× (3.28–3.54) | 3.97× (3.93–4.01) | 4.41× (4.38–4.44) | 1.60× (1.59–1.62) | 1.93× (1.91–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 144 | 34.2 | 1.54× (1.43–1.66) | 3.17× (2.86–3.51) | 2.78× (2.70–2.85) | 3.87× (3.70–4.05) | 3.38× (3.29–3.47) | 6.56× (6.41–6.71) | 7.29× (7.12–7.46) | 2.65× (2.60–2.70) | 3.19× (3.12–3.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.057 s | 170 | 46.5 | 1.56× (1.52–1.61) | 2.46× (2.38–2.54) | 2.46× (2.37–2.56) | 3.37× (3.28–3.47) | 3.15× (3.08–3.23) | 7.71× (7.53–7.90) | 8.56× (8.36–8.77) | 3.11× (3.04–3.19) | 3.74× (3.66–3.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 837 | 3.6 | 0.60× (0.55–0.64) | 3.32× (3.09–3.56) | 4.17× (3.89–4.47) | 2.97× (2.77–3.18) | 6.46× (6.04–6.92) | 2.18× (2.04–2.34) | 2.23× (2.07–2.39) | 0.91× (0.85–0.98) | 1.67× (1.55–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 749 | 4.1 | 0.48× (0.47–0.49) | 2.31× (2.25–2.37) | 4.63× (4.52–4.75) | 2.77× (2.71–2.83) | 6.11× (5.93–6.29) | 2.02× (1.97–2.06) | 2.05× (2.00–2.11) | 0.84× (0.82–0.86) | 1.54× (1.50–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 741 | 4.0 | 0.49× (0.48–0.50) | 2.31× (2.28–2.35) | 4.64× (4.57–4.71) | 2.77× (2.73–2.82) | 6.05× (5.93–6.18) | 2.02× (1.99–2.05) | 2.05× (2.02–2.09) | 0.84× (0.82–0.86) | 1.54× (1.52–1.56) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.051 s | 290 | 9.3 | 2.03× (2.03–2.04) | 4.16× (4.15–4.17) | 4.58× (4.52–4.65) | 4.88× (4.87–4.90) | 5.13× (5.09–5.17) | 4.81× (4.79–4.82) | 4.95× (4.93–4.98) | 1.69× (1.65–1.72) | 1.98× (1.97–1.98) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.036 s | 414 | 11.9 | 1.58× (1.54–1.63) | 3.36× (3.22–3.50) | 4.06× (3.98–4.14) | 4.61× (4.49–4.74) | 4.58× (4.50–4.67) | 6.88× (6.77–6.99) | 7.09× (6.97–7.21) | 2.42× (2.35–2.48) | 2.83× (2.78–2.87) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.032 s | 459 | 17.9 | 1.58× (1.49–1.67) | 2.44× (2.34–2.56) | 3.47× (3.26–3.69) | 3.71× (3.52–3.90) | 4.03× (3.90–4.16) | 7.53× (7.29–7.77) | 7.76× (7.53–7.99) | 2.64× (2.54–2.76) | 3.09× (3.00–3.19) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (2.99–3.24) | 7.05× (6.81–7.31) | 7.32× (7.05–7.61) | 8.35× (8.05–8.66) | 7.36× (7.09–7.63) | 7.66× (7.39–7.94) | 7.40× (7.13–7.68) | 2.29× (2.21–2.38) | 2.57× (2.47–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.03× (3.00–3.06) | 6.89× (6.83–6.95) | 7.23× (7.16–7.31) | 8.36× (8.32–8.40) | 7.07× (7.01–7.13) | 7.23× (7.19–7.27) | 6.98× (6.94–7.03) | 2.16× (2.15–2.17) | 2.42× (2.40–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.05× (2.98–3.13) | 5.13× (5.07–5.20) | 5.57× (5.48–5.65) | 6.23× (6.12–6.33) | 5.42× (5.32–5.51) | 4.57× (4.52–4.63) | 4.42× (4.37–4.47) | 1.37× (1.35–1.38) | 1.53× (1.51–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.80× (2.74–2.85) | 3.73× (3.66–3.80) | 3.23× (3.17–3.29) | 5.59× (5.48–5.70) | 2.41× (2.37–2.46) | 2.41× (2.36–2.46) | 0.94× (0.92–0.97) | 1.58× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.55) | 2.56× (2.51–2.60) | 4.63× (4.55–4.70) | 3.10× (3.04–3.15) | 5.14× (5.05–5.22) | 2.29× (2.25–2.33) | 2.29× (2.25–2.33) | 0.89× (0.87–0.91) | 1.50× (1.47–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.53) | 2.73× (2.65–2.80) | 4.52× (4.42–4.63) | 3.01× (2.94–3.09) | 5.09× (5.00–5.19) | 2.23× (2.17–2.29) | 2.23× (2.17–2.29) | 0.87× (0.84–0.91) | 1.46× (1.43–1.50) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.65× (2.59–2.72) | 2.90× (2.83–2.96) | 3.51× (3.33–3.70) | 4.00× (3.90–4.09) | 4.49× (4.37–4.62) | 2.72× (2.66–2.79) | 2.24× (2.19–2.30) | 2.06× (2.02–2.11) | 2.69× (2.63–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.34–2.54) | 2.63× (2.56–2.70) | 3.50× (3.44–3.57) | 3.49× (3.43–3.55) | 3.80× (3.73–3.87) | 2.26× (2.22–2.29) | 1.86× (1.83–1.89) | 1.71× (1.68–1.73) | 2.23× (2.20–2.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.20× (2.11–2.29) | 1.83× (1.73–1.94) | 2.42× (2.33–2.50) | 2.24× (2.14–2.33) | 2.31× (2.22–2.40) | 1.18× (1.13–1.22) | 0.97× (0.93–1.01) | 0.89× (0.86–0.92) | 1.16× (1.12–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.46× (1.41–1.51) | 2.73× (2.63–2.83) | 2.50× (2.40–2.60) | 2.62× (2.53–2.72) | 2.98× (2.87–3.09) | 3.14× (3.02–3.26) | 3.32× (3.20–3.44) | 1.32× (1.28–1.37) | 1.45× (1.40–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.38× (1.36–1.40) | 2.62× (2.59–2.64) | 2.35× (2.29–2.41) | 3.15× (3.11–3.19) | 2.73× (2.71–2.76) | 2.84× (2.80–2.88) | 3.00× (2.96–3.04) | 1.20× (1.19–1.21) | 1.31× (1.30–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.60× (1.53–1.68) | 2.53× (2.50–2.57) | 2.15× (2.13–2.18) | 2.93× (2.88–2.98) | 2.83× (2.78–2.88) | 2.22× (2.18–2.25) | 2.34× (2.31–2.38) | 0.94× (0.92–0.95) | 1.03× (1.01–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.62× (1.61–1.63) | 3.20× (3.17–3.23) | 2.78× (2.75–2.81) | 2.94× (2.92–2.97) | 3.37× (3.25–3.50) | 3.93× (3.89–3.96) | 4.36× (4.33–4.39) | 1.59× (1.58–1.61) | 1.85× (1.83–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.50× (1.48–1.52) | 2.99× (2.95–3.02) | 2.54× (2.51–2.57) | 3.57× (3.53–3.61) | 3.01× (2.98–3.04) | 3.47× (3.44–3.50) | 3.85× (3.81–3.89) | 1.41× (1.40–1.42) | 1.64× (1.62–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.73–1.81) | 2.74× (2.68–2.81) | 2.18× (2.13–2.23) | 3.15× (3.08–3.22) | 2.88× (2.81–2.95) | 2.54× (2.48–2.59) | 2.81× (2.76–2.87) | 1.03× (1.01–1.05) | 1.20× (1.17–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.60–0.69) | 2.35× (2.20–2.52) | 3.26× (3.04–3.51) | 2.71× (2.54–2.90) | 5.78× (5.42–6.16) | 2.03× (1.90–2.17) | 2.07× (1.94–2.21) | 0.92× (0.86–0.98) | 1.32× (1.24–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.52–0.54) | 2.05× (2.00–2.10) | 3.65× (3.59–3.72) | 2.51× (2.46–2.56) | 5.29× (5.15–5.42) | 1.80× (1.77–1.84) | 1.84× (1.80–1.88) | 0.82× (0.80–0.83) | 1.17× (1.15–1.20) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.53) | 2.28× (2.24–2.33) | 3.68× (3.63–3.73) | 2.57× (2.53–2.61) | 5.37× (5.27–5.48) | 1.85× (1.83–1.88) | 1.89× (1.86–1.92) | 0.84× (0.83–0.85) | 1.20× (1.17–1.23) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 2.00× (2.00–2.01) | 3.95× (3.93–3.97) | 4.41× (4.39–4.43) | 4.76× (4.75–4.77) | 5.00× (4.96–5.04) | 4.68× (4.67–4.70) | 4.83× (4.81–4.85) | 1.67× (1.63–1.70) | 1.83× (1.82–1.84) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.66× (1.63–1.70) | 3.24× (3.19–3.30) | 3.61× (3.57–3.66) | 4.26× (4.18–4.33) | 3.99× (3.93–4.06) | 3.66× (3.60–3.71) | 3.77× (3.71–3.83) | 1.30× (1.27–1.34) | 1.43× (1.41–1.45) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.74× (1.69–1.79) | 2.61× (2.55–2.67) | 2.98× (2.91–3.05) | 3.36× (3.27–3.45) | 3.21× (3.16–3.25) | 2.42× (2.39–2.44) | 2.49× (2.47–2.51) | 0.86× (0.84–0.88) | 0.94× (0.93–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.33× (0.32–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.80) | 11.07× (11.05–11.08) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.37) | 0.46× (0.39–0.54) | 0.72× (0.69–0.75) | 0.61× (0.58–0.64) | 5.77× (5.53–6.01) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.35–0.39) | 0.56× (0.44–0.70) | 0.66× (0.63–0.68) | 0.57× (0.54–0.61) | 3.83× (3.65–4.01) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.69–1.71) | 0.66× (0.65–0.67) | 8.02× (7.98–8.06) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.68–0.73) | 0.68× (0.62–0.75) | 1.67× (1.62–1.72) | 0.83× (0.81–0.86) | 6.04× (5.86–6.22) | 0.49× (0.48–0.51) | 0.51× (0.49–0.53) | 0.56× (0.54–0.58) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.67–0.72) | 0.77× (0.73–0.80) | 1.26× (1.23–1.30) | 0.77× (0.75–0.79) | 3.69× (3.59–3.80) | 0.29× (0.28–0.30) | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.19× (5.17–5.21) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.50) | 0.39× (0.37–0.42) | 0.89× (0.87–0.91) | 0.71× (0.70–0.73) | 3.07× (3.01–3.13) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.87) | 0.81× (0.78–0.83) | 2.50× (2.46–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.21–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.46–0.49) | 0.37× (0.34–0.41) | 0.84× (0.81–0.87) | 0.72× (0.69–0.74) | 2.89× (2.79–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.46× (0.43–0.49) | 0.80× (0.79–0.81) | 0.82× (0.80–0.85) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.22× (6.22–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.19× (6.18–6.20) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.47× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.46× (6.44–6.48) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 34 MB | 0.51× (0.50–0.52) | 0.46× (0.41–0.50) | 1.14× (1.11–1.17) | 0.78× (0.75–0.81) | 3.76× (3.69–3.84) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.37× (0.37–0.38) | 0.18× (0.18–0.18) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 50 MB | 0.55× (0.53–0.57) | 0.47× (0.43–0.52) | 1.00× (0.97–1.03) | 0.83× (0.80–0.85) | 2.56× (2.49–2.64) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.24× (0.24–0.25) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 546 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 956 | 1.98 | 1.06× (1.02–1.10) | 88% | 1.03× (1.03–1.04) | 1.03× (1.01–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,170 | 3.14 | 1.68× (1.62–1.73) | 54% | 1.22× (1.20–1.23) | 1.64× (1.60–1.68) |

**Best:** mibz2 at 4 threads (1,170 MB/s); lbzip2 at 4 threads (244 MB/s); pbz2 at 4 threads (358 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.50 | 0.01 | 0.09 | 0.179 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.58 | 9.9% |
| MTF + RLE2 | 1.11 | 18.8% |
| T-vector build | 1.05 | 17.9% |
| walk (the one each build uses) | 2.85 | 48.4% |
| of which: segment assembly | 0.10 | 1.7% |
| RLE1 | 0.13 | 2.3% |
| CRC | 0.18 | 3.0% |
| **total** | 5.90 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.068 | (in its total) | 2.73× (2.71–2.75) |  |
| walk | 1.049 | 2.036 | (in its total) | 1.94× (1.93–1.95) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.041 | (in its total) | 1.13× (1.11–1.15) |  |
| inverse BWT, total | 1.475 | 3.145 | 1.868 | 2.13× (2.12–2.14) | 1.26× (1.26–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m50s ± 7m08s |
| 4 threads | 26m09s ± 3m41s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m16s ± 2m35s |
| the scanner alone (its serial part) | 8m18s ± 30.2s |
| the workers alone, over 4 | 18m16s ± 2m35s |

At 4 threads the workers bound it. Per byte: the scanner 4.8 ns a byte of bz2, the workers 2.39 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,226,448 | 14,878,554 | \`a7f759d119bce1b7…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
