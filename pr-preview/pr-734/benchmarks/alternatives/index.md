# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-734/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`426b4d7`](<https://github.com/lmmx/mibz2/commit/426b4d7240c6a2a785de75cb7a206a053684560c>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44982318413>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.401 s | 523 | 5.0 | 3.04× (3.02–3.05) | 6.96× (6.95–6.98) | 7.22× (7.17–7.27) | 8.21× (8.18–8.24) | 7.19× (7.17–7.21) | 7.56× (7.52–7.60) | 7.12× (7.10–7.15) | 2.25× (2.24–2.26) | 2.52× (2.51–2.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.234 s | 896 | 5.5 | 3.04× (2.85–3.23) | 6.07× (5.61–6.56) | 7.07× (6.61–7.56) | 7.24× (6.77–7.74) | 6.38× (5.95–6.84) | 12.53× (11.73–13.39) | 11.81× (11.01–12.66) | 3.73× (3.48–4.00) | 4.18× (3.90–4.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.193 s | 1,090 | 8.4 | 2.97× (2.93–3.01) | 4.45× (4.35–4.54) | 5.42× (5.29–5.55) | 5.42× (5.34–5.50) | 5.30× (5.23–5.37) | 15.66× (15.47–15.85) | 14.76× (14.60–14.92) | 4.67× (4.61–4.72) | 5.22× (5.15–5.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 2.5 | 0.57× (0.56–0.58) | 3.40× (3.33–3.47) | 4.54× (4.44–4.64) | 3.54× (3.47–3.62) | 6.08× (5.92–6.24) | 2.67× (2.57–2.76) | 2.64× (2.58–2.70) | 0.96× (0.92–1.00) | 1.80× (1.65–1.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,111 | 2.6 | 0.53× (0.52–0.54) | 2.82× (2.75–2.89) | 5.66× (5.52–5.80) | 3.55× (3.47–3.63) | 5.94× (5.81–6.07) | 2.65× (2.60–2.70) | 2.62× (2.56–2.69) | 0.95× (0.92–0.99) | 1.79× (1.64–1.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,081 | 2.6 | 0.53× (0.52–0.54) | 2.81× (2.76–2.86) | 5.62× (5.51–5.72) | 3.52× (3.45–3.59) | 5.96× (5.83–6.10) | 2.63× (2.53–2.73) | 2.60× (2.55–2.65) | 0.95× (0.92–0.98) | 1.78× (1.64–1.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 751 | 3.7 | 2.74× (2.71–2.77) | 3.12× (3.09–3.14) | 3.94× (3.85–4.03) | 4.16× (4.12–4.19) | 4.56× (4.52–4.61) | 2.81× (2.79–2.83) | 2.30× (2.28–2.32) | 2.11× (2.09–2.12) | 3.10× (3.08–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,051 | 4.5 | 2.21× (2.03–2.39) | 2.79× (2.53–3.09) | 4.25× (3.86–4.68) | 3.44× (3.18–3.73) | 4.89× (4.55–5.26) | 3.92× (3.65–4.22) | 3.22× (2.98–3.47) | 2.94× (2.73–3.17) | 4.34× (4.03–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 961 | 8.8 | 1.87× (1.81–1.94) | 1.94× (1.81–2.08) | 2.99× (2.75–3.24) | 2.15× (2.07–2.24) | 4.04× (3.95–4.14) | 3.62× (3.54–3.70) | 2.97× (2.89–3.04) | 2.72× (2.65–2.79) | 4.00× (3.91–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 36.2 | 1.38× (1.37–1.38) | 2.65× (2.62–2.68) | 2.41× (2.39–2.43) | 2.50× (2.49–2.51) | 2.82× (2.79–2.85) | 2.99× (2.96–3.02) | 2.99× (2.98–3.00) | 1.25× (1.24–1.25) | 1.39× (1.37–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 41.0 | 1.24× (1.20–1.28) | 2.41× (2.34–2.47) | 2.27× (2.19–2.35) | 3.03× (2.91–3.15) | 2.81× (2.73–2.90) | 4.72× (4.57–4.88) | 4.72× (4.59–4.86) | 1.97× (1.91–2.03) | 2.20× (2.12–2.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 53.8 | 1.33× (1.27–1.40) | 2.09× (2.02–2.18) | 2.12× (1.99–2.25) | 2.64× (2.49–2.81) | 2.88× (2.74–3.03) | 5.89× (5.59–6.20) | 5.89× (5.61–6.19) | 2.46× (2.35–2.58) | 2.74× (2.61–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 32.3 | 1.53× (1.51–1.55) | 3.06× (3.02–3.10) | 2.69× (2.64–2.73) | 2.78× (2.76–2.80) | 3.21× (3.07–3.36) | 3.71× (3.68–3.73) | 3.90× (3.87–3.93) | 1.51× (1.49–1.53) | 1.80× (1.79–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 133 | 37.0 | 1.34× (1.25–1.42) | 2.71× (2.51–2.92) | 2.49× (2.34–2.66) | 3.46× (3.24–3.69) | 3.07× (2.95–3.20) | 5.92× (5.55–6.32) | 6.23× (5.86–6.62) | 2.41× (2.25–2.58) | 2.88× (2.70–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 151 | 52.7 | 1.41× (1.38–1.43) | 2.22× (2.16–2.28) | 2.22× (2.14–2.30) | 3.03× (2.98–3.09) | 2.85× (2.79–2.91) | 6.93× (6.78–7.09) | 7.29× (7.14–7.44) | 2.82× (2.75–2.89) | 3.37× (3.29–3.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 3.5 | 0.64× (0.63–0.64) | 3.52× (3.50–3.54) | 4.43× (4.40–4.47) | 3.13× (3.12–3.15) | 6.76× (6.67–6.85) | 2.32× (2.31–2.34) | 2.35× (2.34–2.37) | 0.95× (0.94–0.96) | 1.77× (1.76–1.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 806 | 3.7 | 0.52× (0.51–0.53) | 2.50× (2.47–2.54) | 5.04× (4.96–5.12) | 2.98× (2.94–3.02) | 6.37× (6.28–6.46) | 2.19× (2.16–2.23) | 2.22× (2.19–2.25) | 0.90× (0.88–0.92) | 1.67× (1.65–1.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 794 | 3.7 | 0.51× (0.50–0.53) | 2.44× (2.38–2.49) | 4.88× (4.77–5.00) | 2.92× (2.85–2.99) | 6.23× (6.09–6.37) | 2.13× (2.08–2.18) | 2.15× (2.10–2.20) | 0.87× (0.85–0.89) | 1.62× (1.58–1.66) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.058 s | 232 | 11.5 | 1.83× (1.82–1.84) | 4.01× (3.97–4.06) | 4.37× (4.35–4.40) | 4.61× (4.59–4.64) | 4.78× (4.74–4.82) | 4.77× (4.71–4.83) | 4.63× (4.60–4.66) | 1.54× (1.50–1.59) | 1.75× (1.74–1.76) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.042 s | 318 | 15.1 | 1.44× (1.37–1.52) | 3.10× (2.97–3.23) | 3.84× (3.69–3.99) | 4.23× (4.04–4.44) | 4.17× (4.01–4.33) | 6.61× (6.31–6.93) | 6.42× (6.18–6.67) | 2.14× (2.06–2.22) | 2.43× (2.34–2.52) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.036 s | 369 | 22.4 | 1.47× (1.43–1.52) | 2.34× (2.23–2.46) | 3.14× (3.03–3.25) | 3.35× (3.22–3.49) | 3.86× (3.75–3.98) | 7.59× (7.44–7.74) | 7.37× (7.20–7.53) | 2.45× (2.39–2.52) | 2.78× (2.73–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.03× (3.01–3.04) | 6.93× (6.91–6.95) | 7.18× (7.13–7.22) | 8.18× (8.15–8.21) | 7.16× (7.15–7.18) | 7.53× (7.49–7.57) | 7.10× (7.08–7.12) | 2.25× (2.24–2.25) | 2.50× (2.49–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.80× (2.73–2.87) | 6.36× (6.18–6.54) | 6.63× (6.46–6.81) | 7.74× (7.56–7.93) | 6.48× (6.30–6.67) | 6.77× (6.58–6.95) | 6.38× (6.21–6.55) | 2.02× (1.96–2.08) | 2.24× (2.18–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.92× (2.87–2.97) | 4.87× (4.82–4.92) | 5.26× (5.21–5.31) | 5.97× (5.91–6.03) | 5.14× (5.10–5.19) | 4.44× (4.40–4.49) | 4.19× (4.15–4.23) | 1.33× (1.31–1.34) | 1.47× (1.46–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.62) | 2.90× (2.83–2.97) | 3.84× (3.76–3.92) | 3.32× (3.26–3.39) | 5.64× (5.50–5.79) | 2.52× (2.44–2.62) | 2.50× (2.45–2.55) | 0.96× (0.93–1.00) | 1.63× (1.59–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.73× (2.67–2.78) | 4.90× (4.81–4.99) | 3.30× (3.23–3.37) | 5.43× (5.33–5.54) | 2.47× (2.42–2.51) | 2.44× (2.39–2.50) | 0.94× (0.91–0.97) | 1.59× (1.56–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.02× (2.97–3.08) | 4.95× (4.86–5.04) | 3.34× (3.28–3.40) | 5.55× (5.43–5.67) | 2.50× (2.41–2.60) | 2.48× (2.43–2.52) | 0.95× (0.93–0.98) | 1.61× (1.58–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.61–2.66) | 2.87× (2.85–2.90) | 3.46× (3.33–3.59) | 3.96× (3.93–4.00) | 4.35× (4.31–4.39) | 2.70× (2.68–2.72) | 2.22× (2.21–2.24) | 2.04× (2.02–2.06) | 2.67× (2.64–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.30–2.37) | 2.59× (2.52–2.67) | 3.44× (3.39–3.48) | 3.43× (3.38–3.49) | 3.73× (3.65–3.80) | 2.22× (2.18–2.26) | 1.83× (1.79–1.87) | 1.68× (1.64–1.71) | 2.19× (2.15–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.92–2.16) | 1.77× (1.72–1.83) | 2.31× (2.22–2.39) | 2.12× (2.03–2.21) | 2.19× (2.12–2.27) | 1.13× (1.09–1.17) | 0.93× (0.90–0.96) | 0.85× (0.83–0.88) | 1.12× (1.08–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.57× (2.56–2.58) | 2.36× (2.33–2.39) | 2.48× (2.47–2.49) | 2.79× (2.76–2.82) | 2.96× (2.93–2.99) | 2.96× (2.95–2.97) | 1.25× (1.24–1.25) | 1.37× (1.36–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.28) | 2.40× (2.35–2.44) | 2.12× (2.09–2.15) | 2.88× (2.83–2.92) | 2.50× (2.46–2.54) | 2.60× (2.57–2.63) | 2.60× (2.56–2.64) | 1.09× (1.08–1.11) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.40–1.49) | 2.26× (2.23–2.29) | 1.92× (1.89–1.95) | 2.60× (2.56–2.65) | 2.47× (2.43–2.52) | 1.98× (1.94–2.02) | 1.98× (1.95–2.01) | 0.83× (0.82–0.85) | 0.91× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.54) | 2.99× (2.97–3.01) | 2.61× (2.56–2.66) | 2.75× (2.74–2.77) | 3.18× (3.04–3.33) | 3.67× (3.65–3.69) | 3.86× (3.83–3.88) | 1.50× (1.48–1.52) | 1.73× (1.72–1.74) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.27–1.42) | 2.68× (2.53–2.83) | 2.29× (2.17–2.43) | 3.23× (3.05–3.42) | 2.72× (2.61–2.84) | 3.13× (2.96–3.32) | 3.30× (3.13–3.48) | 1.28× (1.21–1.36) | 1.48× (1.40–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.51–1.67) | 2.50× (2.44–2.57) | 1.98× (1.93–2.04) | 2.87× (2.79–2.94) | 2.60× (2.51–2.68) | 2.29× (2.23–2.36) | 2.41× (2.34–2.48) | 0.94× (0.92–0.96) | 1.08× (1.05–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.68–0.69) | 2.49× (2.47–2.51) | 3.44× (3.39–3.49) | 2.86× (2.85–2.88) | 6.03× (5.95–6.11) | 2.16× (2.14–2.17) | 2.18× (2.17–2.19) | 0.96× (0.95–0.97) | 1.39× (1.38–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.28× (2.25–2.30) | 3.99× (3.95–4.04) | 2.75× (2.72–2.78) | 5.63× (5.56–5.70) | 2.01× (1.98–2.03) | 2.03× (2.00–2.05) | 0.89× (0.88–0.91) | 1.29× (1.28–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.48× (2.42–2.53) | 3.99× (3.90–4.08) | 2.77× (2.71–2.83) | 5.68× (5.57–5.78) | 2.01× (1.97–2.05) | 2.03× (1.99–2.07) | 0.89× (0.88–0.91) | 1.29× (1.27–1.32) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.80× (1.79–1.82) | 3.81× (3.80–3.83) | 4.17× (4.15–4.19) | 4.51× (4.48–4.54) | 4.67× (4.63–4.71) | 4.66× (4.60–4.72) | 4.53× (4.50–4.56) | 1.53× (1.48–1.57) | 1.65× (1.63–1.67) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.46× (1.41–1.51) | 3.08× (2.98–3.18) | 3.36× (3.24–3.49) | 4.02× (3.88–4.15) | 3.65× (3.53–3.77) | 3.56× (3.41–3.72) | 3.46× (3.33–3.59) | 1.17× (1.13–1.20) | 1.26× (1.22–1.30) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.60× (1.55–1.65) | 2.48× (2.40–2.56) | 2.78× (2.71–2.84) | 3.22× (3.14–3.31) | 3.05× (2.97–3.15) | 2.39× (2.32–2.47) | 2.32× (2.26–2.39) | 0.78× (0.76–0.80) | 0.84× (0.83–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.32× (0.31–0.34) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.96× (10.92–10.99) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.33) | 0.46× (0.39–0.54) | 0.65× (0.63–0.68) | 0.56× (0.54–0.58) | 5.27× (5.07–5.47) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.32–0.34) | 0.51× (0.42–0.61) | 0.58× (0.56–0.61) | 0.51× (0.49–0.53) | 3.42× (3.28–3.57) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.76) | 0.83× (0.83–0.83) | 4.37× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.92–7.97) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.66× (0.60–0.72) | 1.58× (1.53–1.64) | 0.80× (0.78–0.83) | 5.76× (5.55–5.97) | 0.47× (0.45–0.49) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.64× (0.62–0.66) | 0.68× (0.64–0.72) | 1.14× (1.10–1.18) | 0.70× (0.68–0.72) | 3.37× (3.27–3.46) | 0.27× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.14× (5.12–5.16) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.36–0.41) | 0.88× (0.85–0.91) | 0.70× (0.68–0.73) | 3.05× (2.96–3.16) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.44–0.47) | 0.87× (0.85–0.88) | 0.81× (0.79–0.83) | 2.52× (2.47–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.16× (5.14–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.35× (0.33–0.38) | 0.83× (0.82–0.85) | 0.69× (0.65–0.73) | 2.87× (2.80–2.94) | 0.18× (0.17–0.18) | 0.19× (0.19–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.61× (0.59–0.62) | 0.47× (0.44–0.51) | 0.80× (0.78–0.81) | 0.82× (0.79–0.85) | 2.31× (2.27–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.34× (5.33–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.58× (1.57–1.58) | 0.76× (0.76–0.76) | 5.23× (5.22–5.25) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.76× (0.76–0.77) | 5.28× (5.26–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.67× (0.67–0.68) | 6.41× (6.39–6.43) | 0.40× (0.40–0.40) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.38× (0.36–0.41) | 0.99× (0.96–1.01) | 0.63× (0.61–0.65) | 3.13× (3.09–3.17) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 60 MB | 0.48× (0.46–0.50) | 0.42× (0.37–0.48) | 0.87× (0.84–0.90) | 0.71× (0.69–0.73) | 2.19× (2.14–2.25) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 523 | 1.92 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 896 | 2.11 | 1.11× (1.08–1.14) | 83% | 1.02× (1.01–1.03) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,090 | 3.25 | 1.69× (1.68–1.71) | 52% | 1.19× (1.18–1.20) | 1.63× (1.60–1.66) |

**Best:** mibz2 at 4 threads (1,090 MB/s); lbzip2 at 4 threads (245 MB/s); pbz2 at 4 threads (368 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.21, lbzip2 1t 0.06, lbzip2 2t 0.24, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.51 | 0.02 | 0.09 | 0.193 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.9% |
| MTF + RLE2 | 1.07 | 17.7% |
| T-vector build | 1.05 | 17.3% |
| walk (the one each build uses) | 2.84 | 46.8% |
| of which: segment assembly | 0.10 | 1.6% |
| RLE1 | 0.13 | 2.2% |
| CRC | 0.18 | 3.0% |
| **total** | 6.06 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.063 | (in its total) | 2.70× (2.68–2.71) |  |
| walk | 1.049 | 2.068 | (in its total) | 1.97× (1.95–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.041 | (in its total) | 0.95× (0.91–0.99) |  |
| inverse BWT, total | 1.484 | 3.172 | 1.880 | 2.13× (2.11–2.15) | 1.27× (1.25–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 58m19s ± 6m40s |
| 4 threads | 28m10s ± 3m57s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m20s ± 2m36s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 18m20s ± 2m36s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.40 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
