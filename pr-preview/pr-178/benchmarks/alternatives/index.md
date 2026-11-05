# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-178/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`17daef2`](<https://github.com/lmmx/mibz2/commit/17daef2349fb0acd37dc0135f45d09e737171cc2>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41967809107>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.400 s | 524 | 5.0 | 2.23× (2.18–2.28) | 5.48× (5.37–5.59) | 5.73× (5.57–5.88) | 6.53× (6.37–6.70) | 5.80× (5.62–5.99) | 6.16× (6.00–6.34) | 5.73× (5.54–5.93) | 1.73× (1.68–1.78) | 1.97× (1.94–2.00) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.276 s | 760 | 5.5 | 1.89× (1.80–1.97) | 4.21× (4.11–4.31) | 4.99× (4.83–5.15) | 4.99× (4.81–5.16) | 4.44× (4.29–4.60) | 8.84× (8.52–9.17) | 8.22× (7.91–8.54) | 2.48× (2.39–2.58) | 2.82× (2.74–2.90) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.236 s | 890 | 8.8 | 1.79× (1.69–1.90) | 2.88× (2.72–3.04) | 3.70× (3.48–3.94) | 3.45× (3.25–3.67) | 3.52× (3.29–3.76) | 10.40× (9.90–10.92) | 9.67× (9.15–10.21) | 2.92× (2.78–3.07) | 3.32× (3.13–3.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,375 | 2.1 | 0.56× (0.52–0.60) | 3.49× (3.24–3.77) | 4.40× (4.05–4.79) | 3.62× (3.37–3.88) | 6.39× (5.98–6.84) | 2.76× (2.56–2.97) | 2.75× (2.55–2.95) | 0.97× (0.91–1.04) | 2.12× (1.93–2.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,342 | 2.2 | 0.51× (0.50–0.52) | 2.93× (2.76–3.11) | 5.65× (5.50–5.80) | 3.60× (3.51–3.69) | 6.23× (6.06–6.40) | 2.75× (2.67–2.83) | 2.73× (2.67–2.80) | 0.97× (0.94–1.00) | 2.11× (1.97–2.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,356 | 2.1 | 0.52× (0.51–0.53) | 3.01× (2.79–3.25) | 5.86× (5.49–6.25) | 3.67× (3.57–3.77) | 6.30× (6.16–6.43) | 2.76× (2.69–2.83) | 2.75× (2.68–2.82) | 0.97× (0.94–1.01) | 2.12× (1.96–2.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 931 | 3.0 | 2.70× (2.66–2.75) | 3.36× (3.26–3.46) | 3.92× (3.82–4.03) | 4.18× (4.11–4.25) | 4.99× (4.64–5.36) | 2.56× (2.52–2.60) | 2.35× (2.29–2.41) | 1.99× (1.92–2.07) | 2.92× (2.86–2.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,315 | 3.8 | 1.92× (1.74–2.12) | 2.62× (2.40–2.86) | 3.63× (3.01–4.38) | 3.26× (2.88–3.69) | 4.87× (4.32–5.51) | 3.32× (2.96–3.73) | 3.06× (2.69–3.47) | 2.59× (2.34–2.86) | 3.79× (3.40–4.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,117 | 7.5 | 1.64× (1.56–1.71) | 1.97× (1.71–2.27) | 2.62× (2.42–2.84) | 2.00× (1.89–2.12) | 4.16× (3.92–4.41) | 3.07× (2.91–3.24) | 2.83× (2.66–3.01) | 2.39× (2.23–2.56) | 3.51× (3.31–3.72) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.088 s | 91 | 29.1 | 1.23× (1.09–1.38) | 2.79× (2.49–3.12) | 2.52× (2.25–2.81) | 2.55× (2.26–2.89) | 2.80× (2.48–3.16) | 3.25× (2.91–3.63) | 3.14× (2.82–3.50) | 1.28× (1.06–1.55) | 1.35× (1.20–1.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.055 s | 145 | 32.8 | 1.20× (1.03–1.39) | 2.52× (2.19–2.90) | 2.38× (2.07–2.74) | 3.14× (2.80–3.52) | 2.86× (2.45–3.33) | 5.19× (4.53–5.96) | 5.02× (4.39–5.74) | 2.05× (1.67–2.52) | 2.16× (1.91–2.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.046 s | 175 | 43.9 | 1.24× (1.19–1.30) | 2.30× (2.21–2.39) | 2.32× (2.25–2.39) | 2.72× (2.60–2.85) | 3.04× (2.89–3.19) | 6.64× (6.45–6.84) | 6.42× (6.22–6.63) | 2.62× (2.28–3.01) | 2.76× (2.68–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 31.7 | 1.09× (1.00–1.18) | 2.73× (2.50–2.99) | 2.26× (2.07–2.46) | 2.31× (2.09–2.56) | 2.54× (2.33–2.76) | 3.50× (3.16–3.87) | 3.52× (3.26–3.79) | 1.18× (1.08–1.29) | 1.34× (1.24–1.45) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.068 s | 141 | 34.3 | 1.12× (1.07–1.17) | 2.67× (2.43–2.93) | 2.25× (2.13–2.37) | 3.17× (3.00–3.36) | 2.68× (2.55–2.82) | 6.11× (5.80–6.43) | 6.14× (5.77–6.53) | 2.05× (1.94–2.17) | 2.34× (2.21–2.47) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 175 | 46.8 | 1.16× (1.12–1.20) | 2.34× (2.17–2.53) | 2.14× (2.06–2.23) | 2.91× (2.84–2.98) | 2.74× (2.67–2.81) | 7.70× (7.40–8.01) | 7.73× (7.42–8.05) | 2.59× (2.52–2.66) | 2.95× (2.84–3.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,000 | 3.1 | 0.55× (0.42–0.70) | 2.78× (2.17–3.57) | 3.85× (3.00–4.94) | 2.79× (2.16–3.61) | 6.34× (4.94–8.14) | 2.09× (1.63–2.69) | 2.07× (1.61–2.67) | 0.87× (0.67–1.13) | 1.86× (1.45–2.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 957 | 3.2 | 0.47× (0.42–0.54) | 2.90× (2.50–3.36) | 4.81× (4.26–5.43) | 2.81× (2.49–3.18) | 6.22× (5.48–7.06) | 2.11× (1.86–2.38) | 2.09× (1.84–2.37) | 0.88× (0.77–1.00) | 1.87× (1.66–2.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 948 | 3.1 | 0.50× (0.48–0.51) | 2.91× (2.83–3.00) | 5.01× (4.87–5.16) | 2.92× (2.80–3.05) | 6.47× (6.26–6.68) | 2.18× (2.12–2.25) | 2.16× (2.10–2.23) | 0.91× (0.88–0.94) | 1.94× (1.89–2.00) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 0.049 s | 250 | 10.8 | 1.47× (1.37–1.58) | 3.29× (3.23–3.36) | 3.50× (3.42–3.59) | 3.65× (3.59–3.72) | 3.99× (3.93–4.05) | 3.98× (3.84–4.13) | 3.71× (3.53–3.91) | 1.20× (1.18–1.22) | 1.46× (1.42–1.50) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 0.034 s | 362 | 12.8 | 1.15× (0.95–1.38) | 2.69× (2.26–3.21) | 2.96× (2.48–3.55) | 3.44× (3.05–3.88) | 3.66× (3.05–4.41) | 5.52× (4.68–6.51) | 5.15× (4.38–6.05) | 1.67× (1.43–1.95) | 2.02× (1.74–2.36) | yes |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 0.028 s | 434 | 19.0 | 1.17× (1.10–1.24) | 2.21× (2.03–2.41) | 2.70× (2.56–2.85) | 3.07× (2.88–3.28) | 3.46× (3.28–3.65) | 6.82× (6.35–7.33) | 6.36× (6.11–6.63) | 2.06× (1.95–2.18) | 2.50× (2.37–2.64) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.22× (2.17–2.28) | 5.45× (5.34–5.56) | 5.69× (5.54–5.85) | 6.51× (6.35–6.67) | 5.78× (5.60–5.97) | 6.14× (5.97–6.31) | 5.71× (5.52–5.91) | 1.73× (1.68–1.78) | 1.96× (1.93–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.10× (2.06–2.15) | 5.14× (5.02–5.27) | 5.45× (5.35–5.55) | 6.30× (6.11–6.49) | 5.32× (5.11–5.54) | 5.65× (5.49–5.82) | 5.26× (5.11–5.41) | 1.59× (1.55–1.64) | 1.80× (1.74–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.02× (1.89–2.15) | 3.69× (3.47–3.92) | 4.08× (3.80–4.38) | 4.46× (4.16–4.77) | 3.92× (3.65–4.21) | 3.42× (3.23–3.62) | 3.18× (2.99–3.38) | 0.96× (0.91–1.02) | 1.09× (1.02–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.55–0.66) | 2.95× (2.68–3.25) | 3.78× (3.45–4.13) | 3.33× (3.02–3.67) | 5.81× (5.29–6.37) | 2.56× (2.32–2.83) | 2.55× (2.31–2.81) | 0.97× (0.89–1.07) | 1.81× (1.61–2.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.79× (2.72–2.86) | 4.87× (4.76–4.97) | 3.33× (3.23–3.43) | 5.63× (5.48–5.79) | 2.54× (2.47–2.61) | 2.52× (2.46–2.59) | 0.96× (0.93–1.00) | 1.80× (1.69–1.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.06× (2.99–3.13) | 5.24× (4.87–5.64) | 3.45× (3.36–3.55) | 5.80× (5.68–5.92) | 2.61× (2.54–2.67) | 2.59× (2.53–2.66) | 0.99× (0.96–1.02) | 1.85× (1.73–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.58× (2.54–2.63) | 2.88× (2.83–2.93) | 3.44× (3.39–3.49) | 3.96× (3.88–4.05) | 4.72× (4.37–5.10) | 2.45× (2.40–2.50) | 2.27× (2.20–2.34) | 1.93× (1.85–2.01) | 2.54× (2.47–2.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (1.99–2.24) | 2.50× (2.35–2.65) | 3.00× (2.65–3.39) | 3.24× (3.04–3.46) | 3.71× (3.47–3.97) | 1.90× (1.79–2.02) | 1.76× (1.63–1.89) | 1.49× (1.40–1.59) | 1.96× (1.84–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.79× (1.73–1.86) | 1.77× (1.48–2.12) | 1.91× (1.79–2.04) | 1.99× (1.83–2.16) | 2.14× (2.00–2.29) | 0.96× (0.90–1.03) | 0.89× (0.82–0.96) | 0.75× (0.70–0.81) | 0.99× (0.92–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.22× (1.09–1.37) | 2.70× (2.43–3.01) | 2.45× (2.19–2.73) | 2.53× (2.25–2.85) | 2.77× (2.47–3.11) | 3.22× (2.90–3.57) | 3.11× (2.81–3.45) | 1.28× (1.06–1.55) | 1.30× (1.17–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.20× (1.09–1.33) | 2.55× (2.35–2.77) | 2.21× (2.03–2.40) | 3.09× (2.87–3.32) | 2.56× (2.32–2.84) | 2.92× (2.66–3.21) | 2.83× (2.58–3.09) | 1.16× (0.98–1.38) | 1.18× (1.08–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.38× (1.32–1.43) | 2.44× (2.38–2.50) | 2.00× (1.94–2.06) | 2.70× (2.59–2.81) | 2.54× (2.44–2.64) | 2.24× (2.19–2.29) | 2.17× (2.12–2.22) | 0.89× (0.76–1.04) | 0.90× (0.89–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.09× (1.00–1.18) | 2.68× (2.46–2.92) | 2.20× (2.02–2.40) | 2.29× (2.07–2.54) | 2.51× (2.31–2.73) | 3.46× (3.14–3.83) | 3.48× (3.23–3.74) | 1.17× (1.08–1.28) | 1.29× (1.19–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.14× (1.06–1.22) | 2.62× (2.56–2.67) | 2.10× (2.07–2.13) | 3.03× (2.94–3.13) | 2.43× (2.39–2.48) | 3.32× (3.18–3.46) | 3.33× (3.22–3.44) | 1.12× (1.10–1.15) | 1.24× (1.21–1.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.27× (1.23–1.31) | 2.51× (2.34–2.70) | 1.83× (1.79–1.87) | 2.61× (2.57–2.65) | 2.36× (2.31–2.41) | 2.44× (2.35–2.53) | 2.45× (2.36–2.55) | 0.83× (0.81–0.84) | 0.91× (0.89–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.58× (0.46–0.74) | 2.22× (1.75–2.83) | 2.92× (2.29–3.72) | 2.49× (1.94–3.20) | 5.50× (4.31–7.03) | 1.90× (1.49–2.44) | 1.89× (1.48–2.41) | 0.87× (0.67–1.11) | 1.30× (1.02–1.65) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.50–0.62) | 2.26× (1.95–2.61) | 3.78× (3.39–4.21) | 2.57× (2.31–2.86) | 5.42× (4.85–6.05) | 1.91× (1.72–2.13) | 1.90× (1.70–2.12) | 0.87× (0.78–0.98) | 1.30× (1.17–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.57) | 2.44× (2.39–2.50) | 3.92× (3.83–4.01) | 2.75× (2.65–2.86) | 5.81× (5.64–5.98) | 2.05× (2.00–2.10) | 2.03× (1.98–2.09) | 0.93× (0.91–0.96) | 1.39× (1.36–1.43) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 1.45× (1.36–1.56) | 3.11× (3.05–3.17) | 3.34× (3.28–3.40) | 3.56× (3.50–3.62) | 3.89× (3.83–3.96) | 3.88× (3.74–4.03) | 3.63× (3.44–3.82) | 1.20× (1.18–1.22) | 1.32× (1.29–1.35) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 1.26× (1.11–1.43) | 2.67× (2.40–2.97) | 2.86× (2.56–3.20) | 3.34× (3.10–3.59) | 3.33× (2.93–3.78) | 3.13× (2.80–3.50) | 2.92× (2.59–3.30) | 0.97× (0.87–1.07) | 1.06× (0.96–1.18) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 1.32× (1.25–1.38) | 2.28× (2.10–2.48) | 2.50× (2.38–2.63) | 2.68× (2.56–2.80) | 2.70× (2.59–2.81) | 2.14× (2.01–2.29) | 2.00× (1.91–2.10) | 0.66× (0.63–0.69) | 0.73× (0.70–0.76) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.38) | 1.08× (1.08–1.08) | 0.86× (0.85–0.88) | 12.23× (12.21–12.26) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 46 MB | 0.40× (0.39–0.40) | 0.61× (0.53–0.69) | 0.84× (0.83–0.86) | 0.70× (0.69–0.72) | 6.70× (6.62–6.77) | 0.15× (0.15–0.16) | 0.17× (0.17–0.17) | 0.25× (0.25–0.26) | 0.13× (0.13–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 70 MB | 0.43× (0.42–0.43) | 0.69× (0.56–0.86) | 0.78× (0.77–0.78) | 0.68× (0.67–0.70) | 4.54× (4.50–4.57) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) | 0.17× (0.17–0.17) | 0.09× (0.09–0.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.33–4.34) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.85) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.74–1.74) | 0.66× (0.66–0.67) | 8.19× (8.17–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.76) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.69× (0.65–0.73) | 1.65× (1.62–1.69) | 0.82× (0.81–0.84) | 6.00× (5.88–6.11) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.55× (0.53–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.66) | 0.71× (0.67–0.76) | 1.18× (1.15–1.21) | 0.78× (0.77–0.80) | 3.47× (3.41–3.54) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.74–0.74) | 5.29× (5.28–5.29) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 35 MB | 0.54× (0.52–0.55) | 0.46× (0.45–0.47) | 0.99× (0.96–1.01) | 0.78× (0.76–0.80) | 3.41× (3.33–3.49) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.17–0.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.85–0.90) | 0.82× (0.80–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 38 MB | 0.51× (0.50–0.52) | 0.41× (0.38–0.43) | 0.92× (0.92–0.93) | 0.78× (0.76–0.80) | 3.16× (3.14–3.19) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.31× (0.31–0.31) | 0.16× (0.16–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 56 MB | 0.58× (0.56–0.59) | 0.47× (0.44–0.50) | 0.77× (0.76–0.79) | 0.77× (0.74–0.80) | 2.23× (2.18–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.46× (5.45–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.78× (0.78–0.78) | 5.45× (5.44–5.47) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.78) | 5.45× (5.44–5.47) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.41× (0.40–0.41) | 1.51× (1.51–1.52) | 0.65× (0.64–0.65) | 6.82× (6.80–6.84) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 0.66× (0.66–0.67) | 0.34× (0.33–0.34) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 2 | 31 MB | 0.51× (0.50–0.52) | 0.50× (0.45–0.56) | 1.27× (1.25–1.29) | 0.77× (0.74–0.81) | 3.96× (3.95–3.97) | 0.24× (0.23–0.24) | 0.25× (0.25–0.25) | 0.39× (0.38–0.39) | 0.20× (0.19–0.20) |
| `text` (1.66 MB → 12.1 MB, 7.3×) | 4 | 48 MB | 0.55× (0.54–0.56) | 0.51× (0.45–0.58) | 1.05× (1.03–1.06) | 0.82× (0.79–0.85) | 2.67× (2.64–2.71) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 524 | 1.92 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 760 | 2.11 | 1.09× (1.05–1.13) | 72% | 1.02× (1.01–1.04) | 1.03× (1.01–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 890 | 3.40 | 1.80× (1.68–1.92) | 42% | 1.22× (1.20–1.23) | 1.63× (1.57–1.69) |

**Best:** mibz2 at 4 threads (890 MB/s); lbzip2 at 4 threads (314 MB/s); pbz2 at 4 threads (501 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.31, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.44, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.56 | 0.01 | 0.08 | 0.236 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.64 | 13.8% |
| MTF + RLE2 | 1.07 | 22.9% |
| T-vector build | 0.90 | 19.3% |
| walk (the one each build uses) | 1.84 | 39.4% |
| of which: segment assembly | 0.07 | 1.5% |
| RLE1 | 0.10 | 2.1% |
| CRC | 0.15 | 3.3% |
| **total** | 4.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.333 | 0.740 | (in its total) | 2.21× (2.19–2.24) |  |
| walk | 0.704 | 1.655 | (in its total) | 2.33× (2.29–2.38) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.027 | 0.031 | (in its total) | 1.12× (1.09–1.16) |  |
| inverse BWT, total | 1.065 | 2.426 | 1.367 | 2.27× (2.23–2.30) | 1.28× (1.27–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m53s ± 7m00s |
| 4 threads | 34m23s ± 5m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m35s ± 3m17s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 20m35s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 2.70 ns a byte of output at 4 threads (CPU, summed over them), 1.60 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,664,767 | 12,147,816 | \`95a0f4fad2b4a029…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
