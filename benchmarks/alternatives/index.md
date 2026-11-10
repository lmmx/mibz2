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

The latest record: [`6a3aeb1`](<https://github.com/lmmx/mibz2/commit/6a3aeb19dfe10b3c1556e14850f046d7b2b66874>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42797772902>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.531 s | 395 | 6.6 | 2.44× (2.38–2.50) | 5.54× (5.43–5.64) | 5.80× (5.67–5.92) | 6.50× (6.39–6.61) | 5.68× (5.53–5.84) | 5.97× (5.78–6.17) | 5.67× (5.50–5.84) | 1.83× (1.77–1.88) | 1.95× (1.92–1.98) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.316 s | 663 | 7.4 | 2.40× (2.28–2.52) | 5.07× (4.82–5.34) | 5.77× (5.39–6.19) | 5.87× (5.55–6.21) | 5.28× (5.01–5.56) | 10.21× (9.56–10.89) | 9.69× (9.08–10.34) | 3.12× (2.96–3.29) | 3.33× (3.17–3.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.266 s | 788 | 12.4 | 2.43× (2.33–2.54) | 3.60× (3.44–3.77) | 4.48× (4.23–4.76) | 4.21× (4.01–4.43) | 4.27× (4.00–4.55) | 11.80× (11.07–12.57) | 11.20× (10.54–11.89) | 3.61× (3.40–3.83) | 3.85× (3.69–4.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,102 | 2.6 | 0.59× (0.52–0.67) | 3.34× (3.27–3.41) | 4.54× (4.30–4.79) | 3.50× (3.43–3.57) | 5.97× (5.82–6.12) | 2.60× (2.54–2.65) | 2.60× (2.54–2.65) | 0.97× (0.95–0.99) | 1.93× (1.74–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,067 | 2.7 | 0.52× (0.49–0.56) | 2.81× (2.65–2.98) | 5.50× (5.31–5.70) | 3.46× (3.34–3.59) | 5.85× (5.64–6.07) | 2.56× (2.46–2.65) | 2.56× (2.46–2.65) | 0.96× (0.92–1.00) | 1.90× (1.75–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,075 | 2.6 | 0.52× (0.51–0.53) | 2.74× (2.68–2.80) | 5.50× (5.36–5.65) | 3.43× (3.36–3.50) | 5.79× (5.66–5.92) | 2.53× (2.47–2.59) | 2.53× (2.47–2.59) | 0.95× (0.91–0.99) | 1.88× (1.71–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 686 | 4.0 | 2.53× (2.49–2.57) | 2.86× (2.80–2.92) | 3.57× (3.50–3.64) | 3.83× (3.75–3.91) | 4.24× (4.07–4.41) | 2.62× (2.54–2.70) | 2.11× (2.07–2.15) | 2.04× (1.96–2.13) | 2.85× (2.79–2.91) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 982 | 4.9 | 2.03× (1.85–2.22) | 2.54× (2.36–2.74) | 3.98× (3.69–4.30) | 3.15× (2.93–3.40) | 4.57× (4.23–4.94) | 3.70× (3.41–4.02) | 2.98× (2.77–3.21) | 2.89× (2.64–3.16) | 4.03× (3.74–4.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 916 | 9.7 | 1.83× (1.74–1.93) | 1.82× (1.71–1.95) | 2.83× (2.69–2.98) | 2.12× (2.00–2.26) | 3.83× (3.71–3.95) | 3.47× (3.36–3.58) | 2.80× (2.71–2.89) | 2.71× (2.59–2.83) | 3.77× (3.65–3.90) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 s | 70 | 37.6 | 1.38× (1.36–1.40) | 2.63× (2.58–2.69) | 2.38× (2.34–2.41) | 2.48× (2.45–2.52) | 2.74× (2.70–2.78) | 2.94× (2.90–2.98) | 3.02× (2.99–3.05) | 1.25× (1.23–1.26) | 1.42× (1.39–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 112 | 42.8 | 1.25× (1.19–1.30) | 2.45× (2.27–2.64) | 2.28× (2.19–2.38) | 2.95× (2.81–3.11) | 2.72× (2.60–2.85) | 4.56× (4.29–4.85) | 4.68× (4.42–4.96) | 1.93× (1.83–2.05) | 2.20× (2.05–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 54.3 | 1.35× (1.32–1.37) | 2.30× (2.16–2.45) | 2.34× (2.26–2.42) | 2.73× (2.65–2.80) | 3.01× (2.92–3.11) | 6.02× (5.92–6.11) | 6.18× (6.09–6.28) | 2.55× (2.51–2.59) | 2.91× (2.84–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.121 s | 79 | 33.3 | 1.52× (1.45–1.59) | 2.98× (2.86–3.10) | 2.60× (2.49–2.71) | 2.73× (2.62–2.84) | 3.03× (2.90–3.15) | 3.64× (3.49–3.79) | 3.89× (3.74–4.04) | 1.57× (1.50–1.63) | 1.81× (1.73–1.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.076 s | 126 | 38.5 | 1.31× (1.25–1.37) | 2.66× (2.51–2.81) | 2.44× (2.33–2.56) | 3.39× (3.22–3.57) | 2.92× (2.80–3.05) | 5.72× (5.49–5.96) | 6.12× (5.86–6.39) | 2.46× (2.33–2.61) | 2.84× (2.71–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 153 | 52.9 | 1.41× (1.37–1.46) | 2.41× (2.19–2.65) | 2.47× (2.20–2.77) | 3.15× (3.03–3.27) | 2.94× (2.77–3.12) | 7.03× (6.84–7.22) | 7.52× (7.30–7.74) | 3.03× (2.91–3.15) | 3.49× (3.40–3.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 3.6 | 0.62× (0.61–0.63) | 3.40× (3.35–3.46) | 4.27× (4.19–4.34) | 3.07× (3.02–3.13) | 6.60× (6.49–6.72) | 2.29× (2.24–2.33) | 2.29× (2.25–2.34) | 0.95× (0.93–0.98) | 1.71× (1.68–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 781 | 3.9 | 0.51× (0.50–0.52) | 2.42× (2.36–2.49) | 4.86× (4.73–4.98) | 2.91× (2.83–2.99) | 6.25× (6.15–6.36) | 2.16× (2.10–2.22) | 2.17× (2.11–2.23) | 0.90× (0.87–0.93) | 1.62× (1.58–1.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 772 | 3.8 | 0.51× (0.50–0.52) | 2.42× (2.39–2.45) | 4.82× (4.76–4.89) | 2.91× (2.87–2.95) | 6.22× (6.11–6.33) | 2.14× (2.09–2.19) | 2.15× (2.12–2.18) | 0.89× (0.88–0.91) | 1.61× (1.59–1.63) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.076 s | 162 | 16.4 | 1.33× (1.31–1.36) | 2.87× (2.81–2.93) | 3.24× (3.16–3.32) | 3.36× (3.28–3.44) | 3.51× (3.42–3.59) | 3.50× (3.41–3.60) | 3.44× (3.34–3.54) | 1.19× (1.13–1.25) | 1.27× (1.24–1.30) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.049 s | 252 | 19.7 | 1.10× (1.01–1.19) | 2.52× (2.33–2.73) | 2.85× (2.67–3.05) | 3.40× (3.16–3.65) | 3.24× (3.02–3.47) | 5.20× (4.91–5.51) | 5.10× (4.67–5.56) | 1.77× (1.62–1.93) | 1.89× (1.75–2.03) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.041 s | 302 | 28.2 | 1.23× (1.14–1.34) | 2.11× (1.91–2.33) | 2.71× (2.52–2.92) | 3.11× (2.89–3.36) | 3.26× (3.04–3.49) | 6.37× (5.96–6.82) | 6.25× (5.95–6.56) | 2.17× (1.98–2.37) | 2.31× (2.13–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.43× (2.38–2.49) | 5.51× (5.41–5.62) | 5.77× (5.65–5.89) | 6.48× (6.37–6.59) | 5.67× (5.51–5.82) | 5.95× (5.76–6.15) | 5.65× (5.49–5.82) | 1.82× (1.77–1.88) | 1.94× (1.91–1.96) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.29× (2.23–2.35) | 5.15× (5.01–5.29) | 5.38× (5.16–5.61) | 6.06× (5.89–6.24) | 5.24× (5.06–5.43) | 5.36× (5.16–5.58) | 5.09× (4.91–5.28) | 1.64× (1.56–1.73) | 1.75× (1.70–1.79) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.22× (2.12–2.33) | 3.69× (3.51–3.88) | 4.06× (3.77–4.37) | 4.32× (4.08–4.58) | 3.89× (3.58–4.22) | 3.11× (2.92–3.31) | 2.95× (2.78–3.13) | 0.95× (0.90–1.01) | 1.01× (0.97–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.56–0.72) | 2.87× (2.81–2.92) | 3.94× (3.72–4.18) | 3.27× (3.21–3.34) | 5.53× (5.39–5.67) | 2.45× (2.40–2.50) | 2.45× (2.40–2.51) | 0.98× (0.95–1.00) | 1.61× (1.57–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.54–0.61) | 2.71× (2.58–2.84) | 4.87× (4.73–5.02) | 3.22× (3.12–3.33) | 5.35× (5.17–5.53) | 2.38× (2.30–2.46) | 2.38× (2.30–2.46) | 0.95× (0.91–0.99) | 1.56× (1.53–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 2.93× (2.86–3.01) | 4.99× (4.87–5.11) | 3.26× (3.20–3.33) | 5.39× (5.29–5.50) | 2.41× (2.36–2.46) | 2.41× (2.36–2.46) | 0.96× (0.92–0.99) | 1.58× (1.56–1.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.44× (2.41–2.48) | 2.66× (2.61–2.72) | 3.21× (3.15–3.28) | 3.67× (3.59–3.74) | 4.05× (3.90–4.22) | 2.52× (2.45–2.60) | 2.05× (2.01–2.09) | 1.98× (1.91–2.06) | 2.51× (2.45–2.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.16× (2.10–2.22) | 2.38× (2.28–2.47) | 3.17× (3.08–3.27) | 3.13× (3.04–3.22) | 3.40× (3.29–3.51) | 2.04× (1.99–2.10) | 1.66× (1.61–1.71) | 1.61× (1.53–1.68) | 2.03× (1.97–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.87–2.01) | 1.60× (1.54–1.66) | 2.11× (2.05–2.17) | 1.88× (1.81–1.95) | 1.97× (1.92–2.03) | 1.03× (1.00–1.06) | 0.83× (0.81–0.86) | 0.81× (0.78–0.84) | 1.03× (1.00–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.39) | 2.55× (2.50–2.60) | 2.32× (2.29–2.35) | 2.46× (2.43–2.49) | 2.71× (2.67–2.76) | 2.91× (2.87–2.95) | 2.99× (2.96–3.01) | 1.24× (1.23–1.26) | 1.35× (1.33–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.24–1.29) | 2.42× (2.31–2.54) | 2.13× (2.09–2.18) | 2.86× (2.83–2.88) | 2.47× (2.41–2.52) | 2.55× (2.52–2.59) | 2.62× (2.59–2.66) | 1.09× (1.08–1.10) | 1.18× (1.17–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.46–1.49) | 2.43× (2.27–2.60) | 2.04× (1.98–2.11) | 2.67× (2.60–2.75) | 2.60× (2.47–2.73) | 1.99× (1.96–2.03) | 2.05× (2.02–2.08) | 0.85× (0.84–0.87) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.45–1.58) | 2.92× (2.81–3.03) | 2.54× (2.44–2.65) | 2.70× (2.60–2.81) | 3.00× (2.88–3.12) | 3.60× (3.45–3.75) | 3.85× (3.71–3.99) | 1.56× (1.49–1.63) | 1.77× (1.70–1.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.39) | 2.71× (2.66–2.76) | 2.33× (2.29–2.37) | 3.26× (3.20–3.33) | 2.70× (2.67–2.73) | 3.14× (3.10–3.19) | 3.36× (3.32–3.40) | 1.36× (1.33–1.39) | 1.55× (1.53–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.56–1.64) | 2.62× (2.46–2.78) | 2.21× (1.97–2.50) | 3.01× (2.89–3.15) | 2.71× (2.53–2.90) | 2.30× (2.24–2.36) | 2.46× (2.40–2.52) | 0.99× (0.96–1.03) | 1.13× (1.10–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.45× (2.39–2.51) | 3.43× (3.37–3.49) | 2.80× (2.76–2.85) | 5.88× (5.78–5.98) | 2.11× (2.08–2.16) | 2.13× (2.09–2.17) | 0.96× (0.94–0.98) | 1.35× (1.32–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.23× (2.17–2.28) | 4.01× (3.90–4.12) | 2.68× (2.61–2.75) | 5.50× (5.42–5.59) | 1.96× (1.91–2.02) | 1.97× (1.93–2.02) | 0.89× (0.86–0.92) | 1.25× (1.22–1.28) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.57) | 2.50× (2.45–2.55) | 4.14× (4.05–4.23) | 2.77× (2.74–2.81) | 5.68× (5.58–5.77) | 2.02× (1.98–2.06) | 2.03× (2.01–2.06) | 0.91× (0.90–0.92) | 1.29× (1.27–1.30) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.33× (1.30–1.35) | 2.80× (2.76–2.85) | 3.13× (3.06–3.20) | 3.31× (3.23–3.38) | 3.45× (3.37–3.54) | 3.45× (3.36–3.54) | 3.38× (3.29–3.48) | 1.19× (1.13–1.25) | 1.21× (1.19–1.22) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.20× (1.17–1.23) | 2.51× (2.42–2.60) | 2.77× (2.70–2.83) | 3.19× (3.12–3.26) | 2.96× (2.89–3.02) | 2.86× (2.77–2.96) | 2.81× (2.75–2.86) | 0.99× (0.95–1.02) | 1.00× (0.98–1.03) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.35× (1.27–1.44) | 2.29× (2.06–2.53) | 2.55× (2.39–2.74) | 2.82× (2.59–3.06) | 2.65× (2.48–2.83) | 1.95× (1.82–2.10) | 1.92× (1.81–2.02) | 0.67× (0.61–0.74) | 0.68× (0.64–0.74) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.38) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.09× (12.08–12.10) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.39) | 0.54× (0.49–0.60) | 0.78× (0.77–0.80) | 0.66× (0.65–0.67) | 6.28× (6.17–6.39) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.39–0.40) | 0.63× (0.53–0.76) | 0.71× (0.69–0.73) | 0.61× (0.60–0.63) | 4.11× (4.05–4.17) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.83–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.69) | 0.68× (0.62–0.74) | 1.60× (1.57–1.64) | 0.79× (0.77–0.81) | 5.80× (5.68–5.93) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.67) | 0.72× (0.68–0.76) | 1.17× (1.15–1.20) | 0.71× (0.69–0.73) | 3.43× (3.36–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.23× (5.22–5.24) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.42) | 0.90× (0.86–0.94) | 0.71× (0.67–0.76) | 3.11× (2.98–3.25) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.47) | 0.87× (0.84–0.89) | 0.82× (0.79–0.84) | 2.54× (2.49–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.23× (5.22–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.44–0.47) | 0.37× (0.35–0.39) | 0.82× (0.80–0.84) | 0.67× (0.63–0.70) | 2.82× (2.75–2.89) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.48× (0.44–0.52) | 0.81× (0.80–0.83) | 0.83× (0.80–0.86) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.23–0.25) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.41× (5.40–5.42) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.21) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.36× (5.34–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.62) | 0.77× (0.77–0.78) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.40× (0.40–0.40) | 1.47× (1.46–1.47) | 0.69× (0.69–0.70) | 6.63× (6.63–6.64) | 0.40× (0.40–0.40) | 0.43× (0.42–0.44) | 0.65× (0.65–0.65) | 0.32× (0.32–0.32) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.36–0.42) | 1.03× (1.00–1.06) | 0.67× (0.65–0.69) | 3.27× (3.21–3.33) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.50× (0.49–0.52) | 0.47× (0.43–0.52) | 0.92× (0.89–0.95) | 0.74× (0.72–0.76) | 2.34× (2.26–2.41) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 395 | 2.54 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 663 | 2.85 | 1.11× (1.08–1.14) | 85% | 1.04× (1.01–1.06) | 1.04× (1.00–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 788 | 4.77 | 1.92× (1.84–2.00) | 49% | 1.28× (1.23–1.34) | 1.75× (1.67–1.84) |

**Best:** mibz2 at 4 threads (788 MB/s); lbzip2 at 4 threads (215 MB/s); pbz2 at 4 threads (319 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.82 | 0.02 | 0.09 | 0.266 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m00s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.79 | 11.6% |
| MTF + RLE2 | 1.12 | 16.6% |
| T-vector build | 1.10 | 16.2% |
| walk (the one each build uses) | 3.43 | 50.7% |
| of which: segment assembly | 0.21 | 3.2% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.18 | 2.7% |
| **total** | 6.78 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.063 | (in its total) | 2.71× (2.67–2.75) |  |
| walk | 1.084 | 2.069 | (in its total) | 1.92× (1.88–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.044 | (in its total) | 0.90× (0.83–0.98) |  |
| inverse BWT, total | 1.528 | 3.175 | 1.895 | 2.09× (2.05–2.14) | 1.24× (1.22–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m23s |
| 4 threads | 39m28s ± 5m49s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m38s ± 4m39s |
| the scanner alone (its serial part) | 11m56s ± 26.3s |
| the workers alone, over 4 | 30m38s ± 4m39s |

At 4 threads the workers bound it. Per byte: the scanner 6.9 ns a byte of bz2, the workers 4.01 ns a byte of output at 4 threads (CPU, summed over them), 2.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
