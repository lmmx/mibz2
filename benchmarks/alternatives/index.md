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

The latest record: [`8b25e7c`](<https://github.com/lmmx/mibz2/commit/8b25e7c5f1f6437461782d25513f3a6586057c68>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45004397300>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.294 s | 712 | 3.7 | 2.74× (2.63–2.85) | 6.92× (6.62–7.24) | 7.24× (6.93–7.57) | 8.27× (7.92–8.63) | 7.39× (7.10–7.69) | 7.58× (7.28–7.90) | 7.22× (6.90–7.55) | 2.17× (2.10–2.25) | 2.51× (2.40–2.63) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.171 s | 1,225 | 4.0 | 2.75× (2.61–2.90) | 6.24× (5.87–6.64) | 7.39× (7.02–7.78) | 7.53× (7.27–7.80) | 6.69× (6.36–7.05) | 12.88× (12.11–13.71) | 12.26× (11.52–13.05) | 3.69× (3.48–3.92) | 4.27× (4.05–4.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.145 s | 1,447 | 6.2 | 2.77× (2.73–2.81) | 4.45× (4.35–4.56) | 5.69× (5.55–5.84) | 5.34× (5.22–5.47) | 5.48× (5.40–5.55) | 15.74× (15.31–16.19) | 14.98× (14.54–15.45) | 4.51× (4.42–4.60) | 5.22× (5.05–5.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,444 | 2.0 | 0.58× (0.56–0.59) | 3.68× (3.62–3.75) | 4.56× (4.48–4.65) | 3.73× (3.65–3.80) | 6.66× (6.46–6.86) | 2.84× (2.77–2.92) | 2.84× (2.77–2.92) | 1.00× (0.97–1.03) | 2.21× (2.18–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,394 | 2.1 | 0.51× (0.50–0.52) | 2.84× (2.76–2.92) | 5.82× (5.65–5.99) | 3.65× (3.56–3.73) | 6.27× (6.11–6.44) | 2.73× (2.66–2.81) | 2.73× (2.66–2.81) | 0.96× (0.93–0.99) | 2.13× (2.07–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,365 | 2.1 | 0.50× (0.49–0.52) | 2.87× (2.74–3.00) | 5.71× (5.51–5.93) | 3.55× (3.44–3.66) | 6.54× (5.95–7.18) | 2.69× (2.62–2.76) | 2.69× (2.60–2.78) | 0.95× (0.92–0.98) | 2.09× (2.02–2.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,092 | 2.6 | 2.89× (2.84–2.95) | 3.72× (3.43–4.03) | 4.28× (4.04–4.53) | 4.59× (4.48–4.70) | 5.46× (5.34–5.58) | 3.11× (2.99–3.25) | 2.69× (2.63–2.75) | 2.18× (2.13–2.22) | 3.38× (3.32–3.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,556 | 3.4 | 2.34× (1.83–3.00) | 3.05× (2.65–3.51) | 4.06× (3.49–4.73) | 3.99× (3.28–4.87) | 5.73× (4.95–6.62) | 4.17× (3.61–4.82) | 3.60× (3.13–4.14) | 2.92× (2.53–3.36) | 4.53× (3.94–5.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,202 | 7.1 | 1.65× (1.56–1.74) | 2.01× (1.85–2.17) | 2.76× (2.64–2.88) | 2.18× (1.94–2.46) | 4.21× (4.02–4.42) | 3.46× (3.28–3.64) | 2.98× (2.85–3.12) | 2.42× (2.29–2.55) | 3.76× (3.60–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.079 s | 102 | 26.0 | 1.36× (1.33–1.40) | 3.11× (3.02–3.20) | 2.78× (2.71–2.85) | 2.75× (2.67–2.83) | 3.08× (3.02–3.14) | 3.45× (3.35–3.55) | 3.48× (3.33–3.64) | 1.33× (1.30–1.37) | 1.55× (1.50–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 158 | 29.6 | 1.24× (1.16–1.32) | 2.79× (2.58–3.01) | 2.58× (2.37–2.79) | 3.22× (2.92–3.56) | 2.97× (2.80–3.14) | 5.27× (4.99–5.58) | 5.32× (5.02–5.64) | 2.04× (1.92–2.16) | 2.37× (2.24–2.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.040 s | 198 | 39.2 | 1.29× (1.28–1.31) | 2.48× (2.40–2.57) | 2.42× (2.35–2.48) | 2.85× (2.79–2.92) | 3.18× (3.10–3.25) | 6.75× (6.61–6.89) | 6.81× (6.56–7.07) | 2.61× (2.55–2.67) | 3.03× (2.94–3.12) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.081 s | 118 | 22.4 | 1.50× (1.46–1.54) | 3.75× (3.66–3.84) | 3.10× (3.04–3.16) | 3.14× (3.05–3.23) | 3.71× (3.33–4.14) | 4.50× (4.33–4.67) | 4.68× (4.59–4.78) | 1.62× (1.58–1.65) | 1.86× (1.81–1.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.053 s | 181 | 26.6 | 1.28× (1.23–1.34) | 3.21× (3.05–3.38) | 2.76× (2.66–2.87) | 3.72× (3.57–3.87) | 3.28× (3.18–3.39) | 6.95× (6.64–7.28) | 7.23× (7.01–7.47) | 2.50× (2.40–2.60) | 2.87× (2.75–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 214 | 36.7 | 1.34× (1.29–1.39) | 2.59× (2.47–2.72) | 2.46× (2.36–2.56) | 3.30× (3.18–3.43) | 3.12× (3.04–3.21) | 8.23× (7.89–8.59) | 8.57× (8.32–8.83) | 2.96× (2.88–3.03) | 3.40× (3.30–3.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,075 | 2.8 | 0.61× (0.60–0.63) | 3.28× (3.21–3.35) | 4.57× (4.47–4.67) | 3.15× (3.08–3.21) | 7.05× (6.82–7.28) | 2.33× (2.28–2.38) | 2.34× (2.29–2.40) | 0.97× (0.95–0.99) | 2.19× (2.15–2.24) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,026 | 3.0 | 0.54× (0.47–0.62) | 3.17× (3.12–3.23) | 5.39× (5.16–5.63) | 3.07× (3.01–3.13) | 6.98× (6.64–7.34) | 2.25× (2.21–2.29) | 2.27× (2.23–2.30) | 0.94× (0.92–0.96) | 2.12× (2.09–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,010 | 2.9 | 0.51× (0.50–0.52) | 3.12× (3.06–3.18) | 5.17× (4.85–5.52) | 2.99× (2.92–3.07) | 6.66× (6.54–6.79) | 2.20× (2.16–2.24) | 2.22× (2.19–2.26) | 0.92× (0.90–0.93) | 2.08× (2.04–2.11) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.043 s | 316 | 8.5 | 1.75× (1.72–1.78) | 4.28× (4.20–4.37) | 4.62× (4.51–4.72) | 4.78× (4.71–4.86) | 5.00× (4.92–5.08) | 5.16× (5.09–5.23) | 4.93× (4.88–4.97) | 1.57× (1.55–1.58) | 1.85× (1.80–1.91) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.030 s | 446 | 11.0 | 1.43× (1.31–1.55) | 3.44× (3.31–3.58) | 4.17× (3.99–4.37) | 4.41× (4.22–4.61) | 4.57× (4.41–4.73) | 7.32× (7.06–7.59) | 6.99× (6.77–7.21) | 2.22× (2.13–2.31) | 2.63× (2.53–2.74) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.029 s | 471 | 17.3 | 1.27× (1.14–1.42) | 2.47× (2.24–2.72) | 3.16× (2.80–3.56) | 3.12× (2.83–3.44) | 3.73× (3.38–4.12) | 7.50× (6.80–8.27) | 7.16× (6.50–7.89) | 2.27× (2.05–2.52) | 2.70× (2.45–2.97) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.73× (2.62–2.84) | 6.88× (6.58–7.19) | 7.19× (6.88–7.52) | 8.23× (7.88–8.59) | 7.36× (7.07–7.65) | 7.55× (7.25–7.86) | 7.18× (6.87–7.51) | 2.17× (2.09–2.24) | 2.49× (2.38–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.62× (2.58–2.66) | 6.56× (6.49–6.64) | 6.90× (6.80–7.01) | 7.97× (7.87–8.07) | 6.81× (6.75–6.87) | 7.00× (6.84–7.15) | 6.66× (6.52–6.80) | 2.01× (1.98–2.04) | 2.31× (2.26–2.36) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.73× (2.68–2.79) | 5.00× (4.92–5.09) | 5.45× (5.34–5.57) | 6.12× (5.99–6.24) | 5.44× (5.33–5.56) | 4.62× (4.54–4.69) | 4.39× (4.31–4.47) | 1.32× (1.30–1.35) | 1.52× (1.48–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.64) | 3.11× (3.03–3.18) | 3.91× (3.84–3.99) | 3.48× (3.41–3.55) | 6.15× (5.96–6.35) | 2.68× (2.61–2.75) | 2.68× (2.60–2.76) | 1.00× (0.98–1.03) | 1.88× (1.84–1.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.58) | 2.79× (2.72–2.87) | 4.89× (4.78–5.00) | 3.36× (3.29–3.43) | 5.67× (5.53–5.82) | 2.52× (2.46–2.58) | 2.52× (2.46–2.58) | 0.94× (0.92–0.97) | 1.77× (1.72–1.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.56) | 3.03× (2.95–3.11) | 5.00× (4.87–5.13) | 3.34× (3.24–3.44) | 6.03× (5.50–6.62) | 2.54× (2.48–2.61) | 2.54× (2.46–2.63) | 0.95× (0.92–0.99) | 1.78× (1.73–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.75× (2.70–2.80) | 3.28× (3.11–3.46) | 3.87× (3.71–4.03) | 4.31× (4.21–4.41) | 5.12× (5.01–5.22) | 2.96× (2.84–3.08) | 2.56× (2.50–2.61) | 2.09× (2.04–2.14) | 2.80× (2.75–2.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.45× (2.04–2.95) | 2.70× (2.64–2.76) | 3.19× (3.10–3.29) | 3.57× (3.44–3.70) | 4.16× (3.98–4.34) | 2.29× (2.20–2.39) | 1.98× (1.93–2.03) | 1.62× (1.56–1.67) | 2.17× (2.09–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.89× (1.79–2.00) | 1.78× (1.64–1.93) | 2.06× (1.94–2.18) | 2.11× (2.00–2.22) | 2.26× (2.13–2.40) | 1.12× (1.06–1.18) | 0.97× (0.92–1.02) | 0.79× (0.75–0.83) | 1.06× (1.00–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.32–1.39) | 2.99× (2.90–3.07) | 2.71× (2.65–2.77) | 2.72× (2.64–2.80) | 3.04× (2.98–3.10) | 3.40× (3.31–3.50) | 3.44× (3.29–3.58) | 1.33× (1.29–1.36) | 1.46× (1.42–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.27) | 2.80× (2.71–2.90) | 2.43× (2.30–2.56) | 3.07× (3.03–3.12) | 2.70× (2.67–2.73) | 3.00× (2.97–3.02) | 3.02× (2.92–3.13) | 1.17× (1.15–1.19) | 1.29× (1.27–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.40–1.45) | 2.58× (2.55–2.61) | 2.09× (2.05–2.13) | 2.79× (2.73–2.84) | 2.66× (2.59–2.72) | 2.26× (2.24–2.28) | 2.28× (2.20–2.36) | 0.88× (0.87–0.90) | 0.97× (0.95–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.45–1.53) | 3.63× (3.55–3.71) | 2.98× (2.92–3.04) | 3.10× (3.01–3.19) | 3.67× (3.29–4.09) | 4.44× (4.27–4.61) | 4.62× (4.53–4.71) | 1.60× (1.57–1.64) | 1.78× (1.74–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.29–1.35) | 3.23× (3.18–3.29) | 2.55× (2.50–2.60) | 3.53× (3.47–3.60) | 2.97× (2.92–3.02) | 3.78× (3.67–3.90) | 3.94× (3.85–4.03) | 1.37× (1.34–1.40) | 1.52× (1.48–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.46–1.58) | 2.88× (2.80–2.97) | 2.17× (2.10–2.24) | 3.08× (2.99–3.18) | 2.80× (2.76–2.83) | 2.71× (2.62–2.81) | 2.83× (2.77–2.88) | 0.98× (0.96–1.00) | 1.09× (1.06–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.69) | 2.55× (2.48–2.62) | 3.35× (3.30–3.41) | 2.87× (2.81–2.94) | 6.26× (6.06–6.46) | 2.15× (2.11–2.19) | 2.18× (2.13–2.22) | 0.98× (0.96–1.00) | 1.54× (1.46–1.62) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.54–0.73) | 2.35× (2.26–2.44) | 4.04× (3.91–4.18) | 2.79× (2.72–2.85) | 6.05× (5.75–6.36) | 2.02× (1.98–2.06) | 2.04× (2.01–2.07) | 0.92× (0.90–0.94) | 1.44× (1.37–1.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.55–0.61) | 2.51× (2.46–2.56) | 4.02× (3.92–4.12) | 2.81× (2.74–2.88) | 5.98× (5.89–6.08) | 2.06× (2.02–2.09) | 2.08× (2.04–2.11) | 0.94× (0.92–0.95) | 1.47× (1.41–1.53) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.72× (1.69–1.75) | 4.06× (4.00–4.12) | 4.39× (4.32–4.46) | 4.65× (4.57–4.73) | 4.86× (4.78–4.94) | 5.02× (4.95–5.09) | 4.79× (4.74–4.84) | 1.54× (1.53–1.56) | 1.66× (1.64–1.68) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.46× (1.34–1.60) | 3.38× (3.19–3.58) | 3.69× (3.57–3.82) | 4.13× (4.05–4.22) | 3.91× (3.82–4.00) | 3.91× (3.80–4.02) | 3.74× (3.66–3.82) | 1.21× (1.17–1.25) | 1.30× (1.26–1.33) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.41× (1.29–1.53) | 2.48× (2.27–2.70) | 2.73× (2.49–2.98) | 3.09× (2.85–3.34) | 2.90× (2.67–3.14) | 2.40× (2.22–2.61) | 2.30× (2.12–2.49) | 0.74× (0.68–0.81) | 0.80× (0.73–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.80× (0.79–0.81) | 10.95× (10.92–10.98) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.48× (0.42–0.56) | 0.65× (0.64–0.67) | 0.56× (0.54–0.58) | 5.26× (5.11–5.41) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.43× (0.38–0.48) | 0.59× (0.56–0.63) | 0.51× (0.48–0.54) | 3.46× (3.30–3.64) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.27–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.68) | 0.66× (0.65–0.66) | 7.94× (7.92–7.95) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.66× (0.61–0.72) | 1.58× (1.54–1.63) | 0.80× (0.77–0.82) | 5.75× (5.58–5.93) | 0.47× (0.45–0.48) | 0.49× (0.47–0.50) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.62× (0.58–0.66) | 0.68× (0.63–0.74) | 1.10× (1.04–1.17) | 0.67× (0.64–0.71) | 3.21× (3.06–3.37) | 0.25× (0.24–0.26) | 0.26× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.15–1.16) | 0.73× (0.72–0.73) | 5.13× (5.11–5.15) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.49–0.54) | 0.42× (0.40–0.44) | 0.93× (0.89–0.96) | 0.74× (0.70–0.78) | 3.22× (3.10–3.35) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.83× (0.80–0.85) | 2.54× (2.48–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.16× (5.15–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.39× (0.37–0.41) | 0.84× (0.81–0.88) | 0.72× (0.69–0.75) | 2.91× (2.80–3.02) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.49) | 0.80× (0.78–0.81) | 0.82× (0.79–0.85) | 2.30× (2.26–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.33× (5.32–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.28× (5.26–5.29) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.59) | 0.77× (0.76–0.77) | 5.29× (5.28–5.30) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.68× (0.67–0.68) | 6.41× (6.40–6.43) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.38× (0.34–0.41) | 0.96× (0.94–0.99) | 0.60× (0.58–0.63) | 3.12× (3.05–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 60 MB | 0.45× (0.43–0.47) | 0.40× (0.37–0.42) | 0.82× (0.79–0.86) | 0.69× (0.66–0.72) | 2.11× (2.03–2.20) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.09–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 712 | 1.41 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,225 | 1.55 | 1.08× (1.03–1.13) | 85% | 1.03× (1.01–1.05) | 1.04× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,447 | 2.37 | 1.64× (1.57–1.71) | 52% | 1.19× (1.16–1.22) | 1.64× (1.61–1.67) |

**Best:** mibz2 at 4 threads (1,447 MB/s); lbzip2 at 4 threads (326 MB/s); pbz2 at 4 threads (521 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.90, mibz2 4t 1.26, lbzip2 1t 0.07, lbzip2 2t 0.35, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 15%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.35 | 0.01 | 0.07 | 0.145 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 36.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.59 | 14.8% |
| MTF + RLE2 | 0.83 | 20.9% |
| T-vector build | 0.76 | 18.9% |
| walk (the one each build uses) | 1.59 | 39.8% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.09 | 2.2% |
| CRC | 0.15 | 3.7% |
| **total** | 4.00 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.286 | 0.688 | (in its total) | 2.39× (2.36–2.42) |  |
| walk | 0.600 | 1.534 | (in its total) | 2.52× (2.44–2.60) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.029 | (in its total) | 1.08× (1.03–1.14) |  |
| inverse BWT, total | 0.913 | 2.253 | 1.258 | 2.44× (2.38–2.50) | 1.37× (1.33–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m52s ± 5m22s |
| 4 threads | 21m06s ± 2m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m01s ± 1m52s |
| the scanner alone (its serial part) | 8m00s ± 32.9s |
| the workers alone, over 4 | 13m01s ± 1m52s |

At 4 threads the workers bound it. Per byte: the scanner 4.6 ns a byte of bz2, the workers 1.71 ns a byte of output at 4 threads (CPU, summed over them), 1.18 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
