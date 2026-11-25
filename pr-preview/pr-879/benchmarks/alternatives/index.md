# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-879/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`aa041e2`](<https://github.com/lmmx/mibz2/commit/aa041e25d742736ea731c5f48fe088c6f33e9737>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45551998020>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.429 s | 488 | 7.4 | 3.38× (3.33–3.43) | 7.57× (7.46–7.69) | 7.96× (7.85–8.08) | 8.98× (8.90–9.07) | 8.09× (7.98–8.20) | 8.59× (8.46–8.72) | 8.25× (8.10–8.40) | 2.65× (2.61–2.69) | 2.40× (2.37–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.236 s | 887 | 7.8 | 3.42× (3.21–3.63) | 7.21× (6.79–7.64) | 8.48× (7.98–9.01) | 8.64× (8.11–9.21) | 7.67× (7.26–8.10) | 15.11× (14.26–16.01) | 14.52× (13.69–15.39) | 4.66× (4.40–4.93) | 4.23× (4.00–4.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,073 | 12.1 | 3.64× (3.53–3.76) | 6.40× (6.22–6.58) | 7.26× (7.07–7.46) | 7.26× (7.06–7.46) | 7.13× (6.95–7.31) | 18.77× (18.32–19.22) | 18.03× (17.45–18.63) | 5.79× (5.63–5.95) | 5.25× (5.10–5.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,042 | 3.7 | 0.55× (0.54–0.57) | 3.23× (3.08–3.38) | 4.24× (4.13–4.36) | 3.56× (3.43–3.70) | 5.97× (5.85–6.10) | 2.65× (2.56–2.74) | 2.63× (2.52–2.74) | 1.11× (1.08–1.15) | 1.08× (1.04–1.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,000 | 4.0 | 0.49× (0.46–0.53) | 2.59× (2.47–2.71) | 5.36× (5.07–5.68) | 3.40× (3.21–3.59) | 5.70× (5.40–6.02) | 2.49× (2.33–2.66) | 2.47× (2.35–2.59) | 1.05× (0.98–1.12) | 1.02× (0.98–1.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 964 | 4.0 | 0.50× (0.48–0.53) | 2.95× (2.76–3.15) | 5.49× (5.27–5.72) | 3.43× (3.24–3.63) | 5.75× (5.47–6.04) | 2.49× (2.35–2.64) | 2.47× (2.35–2.59) | 1.05× (1.01–1.09) | 1.02× (0.97–1.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 579 | 6.5 | 2.89× (2.81–2.97) | 3.37× (3.27–3.47) | 3.93× (3.81–4.05) | 4.44× (4.31–4.57) | 4.82× (4.70–4.94) | 3.60× (3.49–3.71) | 2.97× (2.87–3.07) | 2.12× (2.07–2.17) | 2.13× (2.07–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 916 | 7.6 | 2.49× (2.38–2.60) | 2.94× (2.79–3.09) | 4.14× (3.92–4.38) | 3.94× (3.70–4.20) | 5.53× (5.32–5.74) | 5.67× (5.42–5.93) | 4.67× (4.43–4.92) | 3.34× (3.21–3.46) | 3.35× (3.21–3.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,021 | 13.4 | 2.35× (2.25–2.44) | 2.21× (2.05–2.39) | 3.57× (3.41–3.73) | 2.76× (2.61–2.93) | 4.88× (4.66–5.11) | 6.18× (5.92–6.44) | 5.09× (4.93–5.24) | 3.63× (3.51–3.76) | 3.65× (3.51–3.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 59.2 | 1.34× (1.30–1.38) | 2.79× (2.69–2.91) | 2.50× (2.38–2.62) | 2.59× (2.50–2.68) | 2.89× (2.80–2.98) | 3.17× (3.06–3.28) | 3.31× (3.21–3.42) | 1.27× (1.23–1.31) | 1.27× (1.22–1.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 108 | 61.9 | 1.33× (1.28–1.38) | 2.81× (2.68–2.96) | 2.64× (2.51–2.78) | 3.44× (3.29–3.59) | 3.19× (3.08–3.31) | 5.50× (5.33–5.69) | 5.75× (5.55–5.96) | 2.20× (2.12–2.28) | 2.20× (2.13–2.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 133 | 85.0 | 1.48× (1.43–1.54) | 2.63× (2.53–2.74) | 2.58× (2.49–2.67) | 3.26× (3.14–3.39) | 3.24× (3.15–3.32) | 6.78× (6.56–7.02) | 7.09× (6.86–7.33) | 2.71× (2.63–2.79) | 2.71× (2.65–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.139 s | 69 | 52.6 | 1.45× (1.43–1.47) | 3.32× (3.25–3.38) | 2.76× (2.71–2.80) | 2.90× (2.88–2.92) | 3.27× (3.21–3.33) | 4.01× (3.96–4.05) | 4.27× (4.22–4.33) | 1.56× (1.54–1.59) | 1.56× (1.52–1.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.081 s | 118 | 58.3 | 1.39× (1.33–1.45) | 3.14× (3.03–3.26) | 2.81× (2.69–2.93) | 3.91× (3.73–4.11) | 3.37× (3.24–3.50) | 6.82× (6.57–7.08) | 7.27× (6.99–7.56) | 2.66× (2.56–2.76) | 2.66× (2.57–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.066 s | 146 | 80.9 | 1.61× (1.55–1.68) | 3.09× (2.99–3.19) | 2.85× (2.78–2.93) | 3.72× (3.64–3.80) | 3.39× (3.31–3.48) | 8.53× (8.35–8.72) | 9.10× (8.92–9.29) | 3.33× (3.23–3.43) | 3.33× (3.25–3.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 784 | 5.1 | 0.60× (0.57–0.64) | 3.25× (3.08–3.42) | 4.08× (3.86–4.31) | 3.12× (2.95–3.29) | 6.65× (6.27–7.05) | 2.28× (2.18–2.39) | 2.35× (2.24–2.46) | 1.01× (0.98–1.05) | 0.84× (0.80–0.88) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 746 | 5.6 | 0.51× (0.48–0.53) | 2.47× (2.24–2.71) | 4.73× (4.53–4.94) | 2.96× (2.86–3.06) | 6.45× (5.99–6.93) | 2.17× (2.07–2.26) | 2.23× (2.14–2.32) | 0.96× (0.91–1.01) | 0.80× (0.77–0.82) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 719 | 5.6 | 0.49× (0.46–0.51) | 2.70× (2.42–3.01) | 4.47× (4.25–4.70) | 2.85× (2.75–2.95) | 5.94× (5.73–6.15) | 2.05× (1.97–2.13) | 2.11× (2.04–2.18) | 0.91× (0.88–0.95) | 0.75× (0.73–0.78) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 0.059 s | 205 | 18.0 | 1.92× (1.86–1.98) | 4.16× (4.06–4.27) | 4.64× (4.52–4.77) | 4.82× (4.69–4.97) | 5.04× (4.92–5.16) | 5.18× (5.05–5.31) | 5.08× (4.95–5.23) | 1.58× (1.53–1.62) | 1.54× (1.51–1.58) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 0.038 s | 312 | 21.6 | 1.65× (1.52–1.79) | 3.49× (3.37–3.61) | 4.75× (4.63–4.88) | 5.04× (4.83–5.26) | 4.73× (4.60–4.87) | 7.89× (7.75–8.03) | 7.74× (7.59–7.90) | 2.40× (2.35–2.45) | 2.35× (2.31–2.39) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 0.034 s | 359 | 32.7 | 1.66× (1.60–1.73) | 2.94× (2.83–3.06) | 3.96× (3.81–4.10) | 4.38× (4.19–4.59) | 4.39× (4.19–4.60) | 9.05× (8.70–9.41) | 8.89× (8.57–9.22) | 2.75× (2.64–2.87) | 2.70× (2.60–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.37× (3.32–3.42) | 7.53× (7.42–7.65) | 7.93× (7.81–8.04) | 8.96× (8.87–9.04) | 8.06× (7.95–8.17) | 8.56× (8.43–8.69) | 8.22× (8.07–8.38) | 2.64× (2.60–2.68) | 2.38× (2.35–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.30× (3.25–3.35) | 7.42× (7.30–7.55) | 8.11× (8.00–8.22) | 9.04× (8.92–9.16) | 7.75× (7.63–7.87) | 8.04× (7.91–8.17) | 7.72× (7.59–7.86) | 2.48× (2.44–2.52) | 2.24× (2.21–2.27) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.61× (3.47–3.75) | 7.07× (6.81–7.33) | 7.21× (6.92–7.51) | 8.00× (7.66–8.36) | 7.17× (6.88–7.47) | 5.29× (5.06–5.54) | 5.08× (4.88–5.30) | 1.63× (1.57–1.70) | 1.47× (1.42–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.60) | 2.75× (2.63–2.87) | 3.73× (3.66–3.81) | 3.37× (3.25–3.50) | 5.62× (5.52–5.73) | 2.53× (2.45–2.62) | 2.51× (2.40–2.62) | 1.11× (1.07–1.15) | 0.87× (0.83–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.49–0.56) | 2.52× (2.40–2.64) | 4.66× (4.40–4.93) | 3.17× (3.00–3.34) | 5.25× (4.98–5.53) | 2.32× (2.17–2.48) | 2.30× (2.19–2.41) | 1.01× (0.95–1.09) | 0.80× (0.74–0.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.50–0.56) | 3.00× (2.85–3.17) | 4.73× (4.54–4.93) | 3.24× (3.06–3.42) | 5.34× (5.07–5.62) | 2.35× (2.22–2.49) | 2.33× (2.22–2.44) | 1.03× (0.99–1.07) | 0.81× (0.77–0.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.80× (2.73–2.89) | 3.12× (3.03–3.23) | 3.64× (3.55–3.73) | 4.29× (4.16–4.42) | 4.66× (4.54–4.78) | 3.49× (3.38–3.60) | 2.88× (2.79–2.99) | 2.07× (2.02–2.12) | 1.97× (1.92–2.02) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.56× (2.49–2.64) | 2.76× (2.69–2.83) | 3.37× (3.28–3.47) | 3.72× (3.61–3.83) | 4.20× (4.11–4.30) | 2.96× (2.86–3.06) | 2.45× (2.36–2.53) | 1.76× (1.71–1.81) | 1.67× (1.65–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.34× (2.22–2.47) | 1.99× (1.89–2.09) | 2.43× (2.34–2.52) | 2.37× (2.26–2.49) | 2.56× (2.44–2.69) | 1.68× (1.61–1.76) | 1.39× (1.33–1.45) | 1.00× (0.95–1.05) | 0.95× (0.90–1.00) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.29–1.38) | 2.74× (2.63–2.84) | 2.44× (2.33–2.54) | 2.57× (2.48–2.66) | 2.86× (2.78–2.96) | 3.14× (3.03–3.26) | 3.28× (3.18–3.39) | 1.26× (1.22–1.30) | 1.24× (1.20–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.27–1.34) | 2.74× (2.66–2.83) | 2.42× (2.35–2.50) | 3.20× (3.11–3.29) | 2.83× (2.75–2.91) | 2.96× (2.90–3.01) | 3.09× (3.02–3.16) | 1.19× (1.16–1.22) | 1.16× (1.13–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.51–1.66) | 2.80× (2.69–2.92) | 2.32× (2.22–2.42) | 3.15× (3.03–3.27) | 2.78× (2.69–2.87) | 2.18× (2.09–2.27) | 2.27× (2.19–2.37) | 0.87× (0.84–0.91) | 0.86× (0.83–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.47) | 3.25× (3.19–3.31) | 2.69× (2.66–2.72) | 2.88× (2.85–2.90) | 3.25× (3.19–3.31) | 3.97× (3.93–4.02) | 4.24× (4.18–4.29) | 1.56× (1.53–1.58) | 1.53× (1.50–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.41× (1.35–1.46) | 3.13× (3.10–3.16) | 2.62× (2.57–2.68) | 3.61× (3.55–3.66) | 3.07× (3.02–3.11) | 3.60× (3.55–3.65) | 3.84× (3.77–3.90) | 1.41× (1.39–1.43) | 1.39× (1.38–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.70× (1.65–1.75) | 3.23× (3.12–3.34) | 2.43× (2.37–2.49) | 3.36× (3.26–3.47) | 2.96× (2.88–3.05) | 2.59× (2.53–2.65) | 2.76× (2.70–2.83) | 1.02× (0.99–1.04) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.61–0.68) | 2.34× (2.22–2.46) | 3.41× (3.25–3.59) | 2.90× (2.75–3.06) | 6.07× (5.74–6.43) | 2.15× (2.06–2.25) | 2.21× (2.11–2.31) | 1.01× (0.98–1.05) | 0.78× (0.75–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.53–0.58) | 2.18× (2.08–2.29) | 4.00× (3.88–4.13) | 2.72× (2.64–2.80) | 5.70× (5.31–6.13) | 1.97× (1.89–2.06) | 2.03× (1.95–2.11) | 0.93× (0.88–0.98) | 0.72× (0.70–0.74) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.50–0.55) | 2.49× (2.41–2.58) | 3.83× (3.67–3.98) | 2.69× (2.59–2.78) | 5.39× (5.21–5.58) | 1.92× (1.85–1.99) | 1.97× (1.91–2.03) | 0.90× (0.87–0.94) | 0.70× (0.68–0.72) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 1.90× (1.84–1.96) | 3.98× (3.88–4.08) | 4.48× (4.34–4.62) | 4.73× (4.59–4.88) | 4.94× (4.82–5.06) | 5.07× (4.95–5.21) | 4.98× (4.84–5.13) | 1.56× (1.52–1.61) | 1.45× (1.41–1.50) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 1.68× (1.64–1.71) | 3.53× (3.46–3.60) | 4.14× (4.04–4.24) | 4.55× (4.45–4.66) | 4.25× (4.15–4.36) | 4.23× (4.16–4.31) | 4.16× (4.08–4.24) | 1.30× (1.27–1.34) | 1.21× (1.18–1.24) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 1.92× (1.88–1.97) | 3.25× (3.15–3.34) | 3.54× (3.46–3.61) | 3.94× (3.83–4.05) | 3.79× (3.66–3.93) | 2.84× (2.77–2.90) | 2.78× (2.73–2.84) | 0.87× (0.86–0.89) | 0.81× (0.79–0.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.97× (10.96–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.36× (0.35–0.37) | 0.49× (0.42–0.57) | 0.73× (0.70–0.75) | 0.61× (0.60–0.63) | 5.82× (5.67–5.99) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.34–0.39) | 0.54× (0.48–0.60) | 0.66× (0.61–0.70) | 0.57× (0.54–0.61) | 3.83× (3.59–4.09) | 0.09× (0.08–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.43) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.95× (7.93–7.97) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.64× (0.59–0.69) | 1.68× (1.64–1.73) | 0.84× (0.82–0.86) | 6.11× (5.96–6.26) | 0.50× (0.49–0.51) | 0.52× (0.51–0.53) | 0.57× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.71× (0.69–0.73) | 0.73× (0.69–0.78) | 1.26× (1.24–1.27) | 0.77× (0.75–0.79) | 3.76× (3.71–3.81) | 0.29× (0.29–0.30) | 0.31× (0.31–0.31) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.13) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.51) | 0.37× (0.34–0.40) | 0.89× (0.87–0.92) | 0.70× (0.67–0.72) | 3.10× (3.01–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.65) | 0.45× (0.45–0.46) | 0.86× (0.85–0.88) | 0.80× (0.78–0.84) | 2.52× (2.47–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.19× (5.17–5.21) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.45× (0.43–0.46) | 0.36× (0.34–0.38) | 0.80× (0.78–0.82) | 0.67× (0.64–0.69) | 2.74× (2.67–2.81) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.27× (0.27–0.28) | 0.14× (0.13–0.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.49) | 0.80× (0.79–0.82) | 0.82× (0.80–0.84) | 2.32× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.20–0.23) | 1.06× (1.05–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.16–6.22) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.65× (0.65–0.65) | 6.34× (6.31–6.36) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 33 MB | 0.51× (0.50–0.52) | 0.40× (0.37–0.42) | 1.16× (1.14–1.19) | 0.76× (0.72–0.79) | 3.72× (3.65–3.80) | 0.23× (0.22–0.23) | 0.25× (0.24–0.25) | 0.37× (0.36–0.38) | 0.18× (0.18–0.19) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 50 MB | 0.55× (0.54–0.57) | 0.49× (0.46–0.52) | 1.01× (0.99–1.03) | 0.79× (0.77–0.82) | 2.58× (2.53–2.63) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 488 | 2.06 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 887 | 2.17 | 1.06× (1.05–1.08) | 88% | 1.05× (1.04–1.06) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,073 | 3.36 | 1.62× (1.55–1.69) | 55% | 1.52× (1.50–1.53) | 1.73× (1.70–1.76) |

**Best:** mibz2 at 4 threads (1,073 MB/s); lbzip2 at 4 threads (167 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.09, lbzip2 1t 0.04, lbzip2 2t 0.23, lbzip2 4t 0.29, pbz2 1t 0.00, pbz2 2t 0.36, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.54 | 0.02 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3600 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 59.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.95 | 9.2% |
| MTF + RLE2 | 1.66 | 16.0% |
| T-vector build | 1.63 | 15.8% |
| walk (the one each build uses) | 5.72 | 55.2% |
| of which: segment assembly | 0.29 | 2.8% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.37 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.438 | 1.147 | (in its total) | 2.61× (2.59–2.63) |  |
| walk | 1.496 | 2.992 | (in its total) | 2.00× (1.99–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.075 | (in its total) | 0.94× (0.93–0.95) |  |
| inverse BWT, total | 2.013 | 4.215 | 2.163 | 2.09× (2.08–2.10) | 1.07× (1.06–1.08) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h02m ± 7m24s |
| 4 threads | 28m34s ± 4m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m32s ± 2m56s |
| the scanner alone (its serial part) | 9m54s ± 26.3s |
| the workers alone, over 4 | 19m32s ± 2m56s |

At 4 threads the workers bound it. Per byte: the scanner 5.7 ns a byte of bz2, the workers 2.56 ns a byte of output at 4 threads (CPU, summed over them), 1.72 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,367,452 | 12,009,010 | \`eb0f47ca286822c7…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3600.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
