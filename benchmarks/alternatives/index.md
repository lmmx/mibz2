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

The latest record: [`94d4eb0`](<https://github.com/lmmx/mibz2/commit/94d4eb040a3a68f5683385c47074b973775f6535>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44300624416>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.526 s | 399 | 8.2 | 2.92× (2.88–2.97) | 6.72× (6.67–6.76) | 6.97× (6.87–7.07) | 7.59× (7.44–7.75) | 6.78× (6.72–6.83) | 7.35× (7.28–7.41) | 6.93× (6.77–7.09) | 2.17× (2.14–2.20) | 2.49× (2.45–2.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.304 s | 689 | 8.9 | 2.85× (2.70–3.00) | 6.00× (5.73–6.28) | 6.85× (6.44–7.28) | 6.94× (6.58–7.32) | 6.10× (5.72–6.52) | 12.35× (11.49–13.27) | 11.64× (10.87–12.47) | 3.65× (3.39–3.93) | 4.18× (3.91–4.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.246 s | 852 | 13.9 | 3.04× (2.97–3.10) | 4.65× (4.45–4.86) | 5.65× (5.50–5.80) | 5.12× (4.93–5.32) | 5.14× (5.01–5.28) | 15.73× (15.43–16.04) | 14.84× (14.36–15.32) | 4.65× (4.53–4.78) | 5.33× (5.19–5.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 998 | 3.6 | 0.61× (0.60–0.62) | 3.80× (3.61–3.99) | 4.49× (4.43–4.55) | 3.62× (3.57–3.68) | 6.24× (6.12–6.36) | 3.04× (2.99–3.08) | 2.70× (2.67–2.74) | 1.02× (1.01–1.03) | 1.53× (1.51–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 983 | 3.7 | 0.59× (0.58–0.60) | 3.03× (3.01–3.06) | 6.07× (5.95–6.20) | 3.61× (3.58–3.64) | 6.16× (6.09–6.24) | 3.01× (2.98–3.04) | 2.68× (2.66–2.70) | 1.01× (0.99–1.03) | 1.52× (1.51–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 946 | 3.7 | 0.57× (0.56–0.58) | 2.93× (2.89–2.97) | 5.94× (5.74–6.15) | 3.47× (3.42–3.52) | 5.97× (5.85–6.10) | 2.89× (2.85–2.94) | 2.57× (2.54–2.61) | 0.97× (0.95–1.00) | 1.46× (1.44–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 578 | 6.0 | 2.64× (2.58–2.71) | 3.01× (2.98–3.05) | 3.92× (3.80–4.04) | 3.86× (3.81–3.92) | 4.15× (4.03–4.27) | 2.55× (2.51–2.58) | 2.11× (2.09–2.13) | 2.03× (2.01–2.06) | 3.00× (2.96–3.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 830 | 7.5 | 2.01× (1.76–2.29) | 2.53× (2.18–2.95) | 3.77× (3.24–4.40) | 3.15× (2.75–3.62) | 4.33× (3.77–4.97) | 3.49× (3.02–4.02) | 2.89× (2.52–3.31) | 2.78× (2.44–3.17) | 4.11× (3.56–4.73) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 784 | 13.9 | 1.90× (1.80–2.00) | 2.04× (1.90–2.19) | 3.01× (2.85–3.18) | 2.11× (2.00–2.24) | 3.89× (3.66–4.12) | 3.39× (3.20–3.59) | 2.81× (2.65–2.98) | 2.71× (2.55–2.87) | 3.99× (3.76–4.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 61 | 54.2 | 1.38× (1.37–1.39) | 2.73× (2.70–2.76) | 2.46× (2.43–2.50) | 2.52× (2.49–2.54) | 2.86× (2.83–2.89) | 3.17× (3.13–3.21) | 3.23× (3.17–3.30) | 1.24× (1.22–1.26) | 1.44× (1.41–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 98 | 61.7 | 1.28× (1.25–1.32) | 2.57× (2.47–2.68) | 2.38× (2.29–2.48) | 3.12× (3.04–3.21) | 2.95× (2.87–3.03) | 5.09× (4.96–5.22) | 5.19× (4.98–5.41) | 1.99× (1.92–2.07) | 2.30× (2.21–2.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 82.5 | 1.32× (1.28–1.36) | 2.20× (2.09–2.32) | 2.12× (2.03–2.21) | 2.58× (2.51–2.65) | 2.81× (2.74–2.88) | 6.04× (5.86–6.21) | 6.15× (5.90–6.41) | 2.36× (2.27–2.46) | 2.73× (2.62–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.145 s | 66 | 49.8 | 1.50× (1.48–1.51) | 3.10× (3.03–3.18) | 2.58× (2.53–2.63) | 2.67× (2.65–2.69) | 3.09× (3.04–3.13) | 3.64× (3.61–3.66) | 4.02× (3.96–4.07) | 1.44× (1.40–1.48) | 1.70× (1.67–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 107 | 56.6 | 1.38× (1.32–1.45) | 2.81× (2.71–2.91) | 2.48× (2.42–2.53) | 3.43× (3.33–3.53) | 3.04× (2.94–3.14) | 5.94× (5.80–6.09) | 6.56× (6.36–6.76) | 2.35× (2.29–2.40) | 2.77× (2.69–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 126 | 79.6 | 1.44× (1.37–1.52) | 2.26× (2.21–2.31) | 2.28× (2.21–2.35) | 3.00× (2.91–3.11) | 2.82× (2.75–2.90) | 6.91× (6.76–7.06) | 7.63× (7.47–7.78) | 2.73× (2.63–2.83) | 3.22× (3.13–3.31) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 5.1 | 0.67× (0.66–0.68) | 3.31× (3.08–3.56) | 4.43× (4.36–4.51) | 3.14× (3.08–3.20) | 6.80× (6.70–6.89) | 2.60× (2.54–2.66) | 2.36× (2.32–2.40) | 0.99× (0.97–1.00) | 1.53× (1.50–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 690 | 5.5 | 0.55× (0.53–0.58) | 2.80× (2.69–2.92) | 4.87× (4.61–5.13) | 2.93× (2.81–3.06) | 6.30× (5.99–6.64) | 2.40× (2.32–2.47) | 2.17× (2.08–2.27) | 0.91× (0.87–0.95) | 1.41× (1.35–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 5.4 | 0.55× (0.54–0.57) | 2.80× (2.75–2.86) | 4.93× (4.80–5.06) | 2.91× (2.87–2.96) | 6.21× (6.11–6.30) | 2.39× (2.33–2.45) | 2.17× (2.12–2.21) | 0.91× (0.89–0.92) | 1.40× (1.38–1.43) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 0.071 s | 182 | 18.3 | 1.94× (1.90–1.99) | 4.10× (3.88–4.33) | 4.44× (4.33–4.55) | 4.56× (4.34–4.79) | 4.72× (4.53–4.91) | 5.20× (5.03–5.38) | 4.85× (4.71–5.00) | 1.51× (1.47–1.56) | 1.87× (1.81–1.93) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 0.049 s | 266 | 22.9 | 1.59× (1.50–1.69) | 3.40× (3.02–3.83) | 3.89× (3.67–4.13) | 4.24× (3.88–4.63) | 4.19× (3.93–4.47) | 7.50× (7.04–7.99) | 7.00× (6.61–7.40) | 2.18× (2.05–2.31) | 2.69× (2.53–2.86) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 0.042 s | 304 | 35.3 | 1.59× (1.50–1.68) | 2.52× (2.40–2.65) | 3.19× (3.05–3.33) | 3.42× (3.27–3.57) | 3.81× (3.70–3.92) | 8.65× (8.33–8.98) | 8.07× (7.85–8.29) | 2.51× (2.45–2.57) | 3.10× (2.99–3.22) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.92× (2.87–2.96) | 6.69× (6.64–6.73) | 6.93× (6.83–7.03) | 7.57× (7.41–7.72) | 6.75× (6.69–6.81) | 7.32× (7.26–7.39) | 6.90× (6.74–7.07) | 2.17× (2.14–2.20) | 2.47× (2.44–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.76× (2.76–2.77) | 6.25× (6.22–6.29) | 6.59× (6.53–6.65) | 7.33× (7.20–7.47) | 6.24× (6.18–6.30) | 6.68× (6.56–6.81) | 6.30× (6.15–6.45) | 1.98× (1.94–2.02) | 2.26× (2.22–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.96× (2.89–3.03) | 4.95× (4.79–5.12) | 5.24× (5.06–5.42) | 5.48× (5.32–5.64) | 4.89× (4.76–5.02) | 4.34× (4.29–4.39) | 4.09× (4.01–4.17) | 1.28× (1.26–1.30) | 1.47× (1.44–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.32× (3.27–3.38) | 4.12× (4.06–4.17) | 3.38× (3.33–3.44) | 5.77× (5.66–5.88) | 2.85× (2.81–2.89) | 2.55× (2.52–2.59) | 1.02× (1.01–1.03) | 1.08× (1.05–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.62–0.65) | 3.12× (3.09–3.16) | 5.43× (5.39–5.47) | 3.34× (3.32–3.37) | 5.61× (5.54–5.67) | 2.77× (2.75–2.80) | 2.48× (2.46–2.50) | 0.99× (0.97–1.01) | 1.05× (1.02–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.41× (3.33–3.48) | 5.40× (5.31–5.49) | 3.29× (3.24–3.34) | 5.54× (5.43–5.65) | 2.73× (2.69–2.77) | 2.45× (2.41–2.48) | 0.98× (0.96–1.00) | 1.03× (1.00–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.55× (2.49–2.62) | 2.79× (2.76–2.81) | 3.54× (3.44–3.65) | 3.70× (3.65–3.76) | 3.97× (3.86–4.09) | 2.46× (2.43–2.49) | 2.05× (2.03–2.07) | 1.98× (1.95–2.00) | 2.69× (2.67–2.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.07× (1.80–2.38) | 2.32× (2.02–2.68) | 2.85× (2.46–3.30) | 2.91× (2.51–3.37) | 3.09× (2.67–3.57) | 1.84× (1.59–2.14) | 1.53× (1.32–1.77) | 1.48× (1.29–1.70) | 2.02× (1.74–2.34) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.07–2.17) | 1.76× (1.69–1.83) | 2.21× (2.14–2.29) | 2.10× (2.04–2.15) | 2.10× (2.03–2.18) | 1.03× (1.00–1.06) | 0.86× (0.83–0.89) | 0.83× (0.80–0.85) | 1.13× (1.09–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.67× (2.65–2.69) | 2.40× (2.36–2.43) | 2.49× (2.47–2.52) | 2.83× (2.80–2.86) | 3.14× (3.10–3.18) | 3.20× (3.14–3.26) | 1.24× (1.22–1.26) | 1.39× (1.36–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.48× (2.43–2.53) | 2.20× (2.14–2.26) | 2.92× (2.87–2.97) | 2.60× (2.55–2.66) | 2.75× (2.70–2.81) | 2.81× (2.72–2.90) | 1.09× (1.06–1.12) | 1.22× (1.18–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.43–1.56) | 2.36× (2.23–2.50) | 1.95× (1.82–2.10) | 2.59× (2.53–2.64) | 2.45× (2.40–2.50) | 2.06× (1.99–2.12) | 2.10× (2.02–2.17) | 0.81× (0.79–0.84) | 0.91× (0.88–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.48–1.50) | 3.05× (2.97–3.12) | 2.52× (2.48–2.56) | 2.65× (2.63–2.67) | 3.06× (3.01–3.11) | 3.60× (3.58–3.63) | 3.98× (3.92–4.03) | 1.43× (1.39–1.47) | 1.66× (1.64–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.33–1.48) | 2.77× (2.69–2.85) | 2.29× (2.25–2.34) | 3.22× (3.16–3.29) | 2.72× (2.65–2.78) | 3.15× (3.10–3.21) | 3.48× (3.40–3.56) | 1.25× (1.21–1.30) | 1.45× (1.42–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.64× (1.60–1.69) | 2.57× (2.53–2.61) | 1.98× (1.92–2.05) | 2.81× (2.74–2.89) | 2.59× (2.52–2.66) | 2.27× (2.22–2.31) | 2.50× (2.45–2.56) | 0.90× (0.87–0.94) | 1.05× (1.02–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.78× (2.74–2.83) | 3.60× (3.55–3.65) | 2.87× (2.82–2.91) | 6.05× (5.98–6.12) | 2.39× (2.34–2.45) | 2.18× (2.15–2.22) | 0.99× (0.97–1.01) | 0.99× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.59–0.64) | 2.48× (2.40–2.56) | 4.20× (4.05–4.35) | 2.71× (2.62–2.81) | 5.57× (5.33–5.83) | 2.18× (2.13–2.24) | 1.99× (1.92–2.06) | 0.90× (0.87–0.94) | 0.90× (0.86–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.59–0.62) | 2.74× (2.69–2.80) | 4.21× (4.14–4.29) | 2.77× (2.72–2.82) | 5.60× (5.52–5.69) | 2.22× (2.17–2.28) | 2.03× (1.99–2.07) | 0.92× (0.90–0.94) | 0.92× (0.89–0.95) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 1.92× (1.88–1.97) | 4.00× (3.78–4.24) | 4.27× (4.15–4.39) | 4.48× (4.26–4.71) | 4.63× (4.45–4.83) | 5.11× (4.93–5.29) | 4.77× (4.62–4.92) | 1.50× (1.45–1.55) | 1.78× (1.73–1.84) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 1.68× (1.63–1.72) | 3.32× (3.24–3.40) | 3.62× (3.54–3.71) | 4.05× (3.94–4.16) | 3.73× (3.65–3.82) | 4.11× (4.00–4.22) | 3.84× (3.76–3.92) | 1.21× (1.18–1.23) | 1.44× (1.40–1.47) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 1.69× (1.63–1.76) | 2.61× (2.53–2.69) | 2.88× (2.79–2.97) | 3.19× (3.08–3.31) | 2.97× (2.89–3.05) | 2.64× (2.55–2.74) | 2.47× (2.41–2.53) | 0.78× (0.76–0.79) | 0.92× (0.89–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.29× (0.28–0.30) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.97–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.36) | 0.52× (0.44–0.61) | 0.70× (0.67–0.73) | 0.59× (0.57–0.62) | 5.63× (5.42–5.85) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 88 MB | 0.34× (0.33–0.35) | 0.69× (0.63–0.75) | 0.61× (0.59–0.64) | 0.53× (0.52–0.55) | 3.56× (3.44–3.68) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.82) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.02× (8.00–8.05) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.71× (0.67–0.75) | 1.62× (1.58–1.65) | 0.81× (0.79–0.83) | 5.87× (5.75–5.99) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.63–0.66) | 0.75× (0.71–0.79) | 1.16× (1.14–1.18) | 0.71× (0.70–0.73) | 3.41× (3.35–3.48) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.17× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.51) | 0.40× (0.38–0.43) | 0.90× (0.88–0.92) | 0.72× (0.70–0.74) | 3.12× (3.06–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.80× (0.79–0.82) | 2.50× (2.45–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.18× (5.18–5.19) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.48× (0.46–0.50) | 0.36× (0.34–0.40) | 0.84× (0.82–0.87) | 0.69× (0.66–0.73) | 2.90× (2.80–3.00) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.49× (0.46–0.52) | 0.81× (0.79–0.82) | 0.82× (0.80–0.84) | 2.32× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.30) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.43–1.43) | 0.67× (0.66–0.67) | 6.45× (6.43–6.47) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 36 MB | 0.47× (0.45–0.48) | 0.40× (0.37–0.43) | 1.09× (1.07–1.12) | 0.71× (0.68–0.73) | 3.41× (3.33–3.48) | 0.21× (0.21–0.21) | 0.22× (0.22–0.23) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 56 MB | 0.49× (0.47–0.50) | 0.49× (0.46–0.52) | 0.90× (0.88–0.92) | 0.69× (0.68–0.71) | 2.30× (2.26–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 399 | 2.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 689 | 2.76 | 1.10× (1.08–1.11) | 84% | 1.03× (1.00–1.05) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 852 | 4.28 | 1.69× (1.67–1.71) | 54% | 1.25× (1.21–1.29) | 1.72× (1.67–1.76) |

**Best:** mibz2 at 4 threads (852 MB/s); lbzip2 at 4 threads (184 MB/s); pbz2 at 4 threads (280 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.70 | 0.02 | 0.09 | 0.246 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.7% |
| MTF + RLE2 | 1.52 | 17.0% |
| T-vector build | 1.52 | 17.0% |
| walk (the one each build uses) | 4.23 | 47.3% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.94 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.302 | (in its total) | 2.65× (2.64–2.65) |  |
| walk | 1.283 | 2.691 | (in its total) | 2.10× (2.09–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.058 | (in its total) | 1.19× (1.17–1.21) |  |
| inverse BWT, total | 1.825 | 4.049 | 3.311 | 2.22× (2.21–2.23) | 1.82× (1.81–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m54s |
| 4 threads | 35m56s ± 5m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m19s ± 3m35s |
| the scanner alone (its serial part) | 13m41s ± 32.2s |
| the workers alone, over 4 | 25m19s ± 3m35s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 3.32 ns a byte of output at 4 threads (CPU, summed over them), 2.07 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,527,905 | 12,873,504 | \`d928dbd828b8a8a4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
