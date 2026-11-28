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

The latest record: [`c3b8f70`](<https://github.com/lmmx/mibz2/commit/c3b8f708314fb99f803c9f44c0a09b42751bee80>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46166937081>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.381 s | 550 | 6.8 | 3.15× (3.11–3.18) | 7.23× (7.16–7.29) | 7.51× (7.42–7.59) | 8.54× (8.45–8.64) | 7.49× (7.41–7.57) | 7.86× (7.77–7.95) | 7.42× (7.35–7.49) | 2.35× (2.33–2.38) | 2.63× (2.61–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.215 s | 976 | 7.3 | 3.16× (2.95–3.38) | 6.47× (6.07–6.89) | 7.46× (6.97–7.98) | 7.91× (7.33–8.53) | 6.83× (6.40–7.30) | 13.38× (12.54–14.28) | 12.64× (11.81–13.52) | 4.01× (3.74–4.28) | 4.49× (4.20–4.79) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.177 s | 1,182 | 11.5 | 3.24× (3.18–3.30) | 4.79× (4.68–4.91) | 5.96× (5.79–6.14) | 5.88× (5.68–6.10) | 5.74× (5.53–5.95) | 16.70× (16.29–17.12) | 15.77× (15.37–16.18) | 5.00× (4.87–5.13) | 5.60× (5.46–5.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,078 | 3.7 | 0.56× (0.55–0.56) | 3.31× (3.27–3.35) | 4.42× (4.36–4.47) | 3.46× (3.42–3.51) | 5.94× (5.86–6.01) | 2.58× (2.55–2.61) | 2.58× (2.54–2.61) | 0.95× (0.92–0.98) | 1.86× (1.69–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,031 | 4.1 | 0.49× (0.49–0.50) | 2.64× (2.59–2.68) | 5.28× (5.19–5.36) | 3.33× (3.27–3.39) | 5.57× (5.46–5.69) | 2.46× (2.42–2.50) | 2.45× (2.41–2.49) | 0.90× (0.88–0.93) | 1.77× (1.61–1.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,015 | 4.0 | 0.49× (0.49–0.50) | 2.61× (2.58–2.65) | 5.21× (5.14–5.29) | 3.30× (3.23–3.36) | 5.52× (5.46–5.59) | 2.43× (2.39–2.46) | 2.42× (2.39–2.46) | 0.89× (0.86–0.93) | 1.75× (1.58–1.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 764 | 5.2 | 2.79× (2.77–2.82) | 3.17× (3.14–3.21) | 4.04× (3.93–4.16) | 4.24× (4.19–4.28) | 4.69× (4.61–4.78) | 2.88× (2.84–2.91) | 2.34× (2.32–2.37) | 2.15× (2.13–2.18) | 3.16× (3.13–3.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,167 | 6.2 | 2.37× (2.26–2.48) | 3.04× (2.80–3.30) | 4.56× (4.19–4.97) | 3.71× (3.43–4.01) | 5.38× (5.01–5.79) | 4.29× (3.99–4.62) | 3.50× (3.26–3.76) | 3.22× (2.99–3.45) | 4.72× (4.39–5.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,083 | 12.3 | 2.10× (2.06–2.14) | 2.11× (1.95–2.29) | 3.35× (3.20–3.51) | 2.47× (2.35–2.59) | 4.70× (4.52–4.89) | 4.11× (4.04–4.18) | 3.35× (3.28–3.42) | 3.08× (3.01–3.14) | 4.51× (4.42–4.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.101 s | 79 | 47.5 | 1.50× (1.49–1.51) | 2.88× (2.83–2.94) | 2.64× (2.59–2.68) | 2.74× (2.72–2.76) | 3.06× (3.03–3.09) | 3.25× (3.23–3.27) | 3.34× (3.31–3.37) | 1.36× (1.35–1.38) | 1.53× (1.50–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 130 | 53.1 | 1.42× (1.34–1.50) | 2.80× (2.59–3.02) | 2.60× (2.49–2.72) | 3.45× (3.27–3.65) | 3.13× (2.99–3.28) | 5.26× (5.03–5.50) | 5.41× (5.17–5.65) | 2.21× (2.11–2.31) | 2.48× (2.37–2.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 161 | 71.2 | 1.48× (1.44–1.52) | 2.42× (2.29–2.56) | 2.38× (2.28–2.49) | 2.91× (2.83–2.99) | 3.21× (3.13–3.29) | 6.51× (6.32–6.70) | 6.69× (6.49–6.89) | 2.73× (2.65–2.82) | 3.06× (2.94–3.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 87 | 43.4 | 1.64× (1.63–1.65) | 3.28× (3.25–3.31) | 2.87× (2.81–2.92) | 2.98× (2.95–3.01) | 3.39× (3.29–3.48) | 3.95× (3.92–3.98) | 4.24× (4.20–4.28) | 1.61× (1.59–1.63) | 1.91× (1.90–1.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.066 s | 146 | 48.9 | 1.49× (1.43–1.55) | 3.03× (2.91–3.15) | 2.78× (2.66–2.90) | 3.86× (3.71–4.01) | 3.37× (3.25–3.51) | 6.53× (6.29–6.79) | 7.01× (6.75–7.29) | 2.66× (2.55–2.77) | 3.17× (3.05–3.29) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 174 | 67.8 | 1.61× (1.55–1.68) | 2.50× (2.40–2.61) | 2.58× (2.46–2.70) | 3.45× (3.32–3.59) | 3.22× (3.15–3.29) | 7.85× (7.64–8.06) | 8.42× (8.21–8.64) | 3.19× (3.12–3.27) | 3.80× (3.71–3.90) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 837 | 5.1 | 0.62× (0.61–0.62) | 3.41× (3.37–3.46) | 4.29× (4.24–4.35) | 3.06× (3.02–3.10) | 6.69× (6.56–6.83) | 2.25× (2.23–2.28) | 2.29× (2.26–2.32) | 0.94× (0.92–0.95) | 1.71× (1.69–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 781 | 5.7 | 0.50× (0.49–0.51) | 2.39× (2.34–2.45) | 4.80× (4.69–4.92) | 2.90× (2.81–3.00) | 6.13× (5.92–6.35) | 2.09× (2.04–2.15) | 2.13× (2.09–2.17) | 0.87× (0.85–0.89) | 1.59× (1.56–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 763 | 5.7 | 0.49× (0.49–0.50) | 2.36× (2.33–2.39) | 4.72× (4.66–4.78) | 2.83× (2.79–2.86) | 6.06× (5.97–6.15) | 2.06× (2.03–2.09) | 2.09× (2.07–2.11) | 0.85× (0.84–0.86) | 1.57× (1.55–1.58) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 0.052 s | 290 | 13.2 | 1.98× (1.86–2.12) | 3.98× (3.73–4.26) | 4.52× (4.23–4.82) | 4.81× (4.52–5.11) | 4.90× (4.59–5.24) | 4.68× (4.38–4.99) | 4.60× (4.31–4.92) | 1.62× (1.52–1.73) | 1.90× (1.78–2.04) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 0.037 s | 409 | 17.5 | 1.53× (1.46–1.61) | 3.18× (2.98–3.39) | 3.83× (3.56–4.11) | 4.45× (4.17–4.75) | 4.35× (4.08–4.64) | 6.58× (6.19–7.00) | 6.48× (6.09–6.90) | 2.28× (2.14–2.43) | 2.68× (2.50–2.87) | yes |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 0.033 s | 464 | 26.8 | 1.61× (1.55–1.66) | 2.58× (2.49–2.66) | 3.17× (3.05–3.30) | 3.78× (3.71–3.86) | 4.06× (3.93–4.19) | 7.65× (7.51–7.78) | 7.53× (7.40–7.67) | 2.65× (2.60–2.69) | 3.11× (3.02–3.20) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.10–3.17) | 7.19× (7.13–7.26) | 7.46× (7.38–7.55) | 8.51× (8.41–8.61) | 7.46× (7.38–7.54) | 7.82× (7.74–7.91) | 7.39× (7.32–7.46) | 2.35× (2.32–2.37) | 2.61× (2.59–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.04× (3.01–3.06) | 6.80× (6.75–6.85) | 7.15× (7.07–7.22) | 8.33× (8.25–8.41) | 6.96× (6.88–7.03) | 7.24× (7.17–7.31) | 6.84× (6.76–6.91) | 2.17× (2.15–2.19) | 2.42× (2.39–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.10× (3.03–3.17) | 5.14× (5.07–5.21) | 5.58× (5.45–5.72) | 6.27× (6.15–6.40) | 5.43× (5.22–5.66) | 4.61× (4.55–4.67) | 4.36× (4.31–4.41) | 1.38× (1.37–1.40) | 1.54× (1.52–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.59–0.60) | 2.84× (2.79–2.88) | 3.80× (3.76–3.85) | 3.25× (3.21–3.29) | 5.51× (5.44–5.58) | 2.44× (2.41–2.47) | 2.44× (2.41–2.47) | 0.95× (0.93–0.98) | 1.59× (1.58–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.53–0.54) | 2.51× (2.47–2.56) | 4.50× (4.43–4.58) | 3.05× (3.00–3.10) | 5.02× (4.92–5.13) | 2.25× (2.21–2.28) | 2.24× (2.21–2.28) | 0.88× (0.86–0.90) | 1.47× (1.45–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.53) | 2.76× (2.72–2.80) | 4.57× (4.47–4.67) | 3.07× (3.01–3.12) | 5.04× (4.98–5.10) | 2.26× (2.23–2.30) | 2.26× (2.23–2.29) | 0.88× (0.85–0.92) | 1.48× (1.46–1.50) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.66–2.70) | 2.93× (2.90–2.95) | 3.56× (3.42–3.71) | 4.03× (3.99–4.08) | 4.46× (4.38–4.54) | 2.76× (2.73–2.79) | 2.26× (2.24–2.28) | 2.08× (2.06–2.11) | 2.74× (2.70–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.45× (2.31–2.59) | 2.64× (2.59–2.70) | 3.55× (3.47–3.63) | 3.52× (3.44–3.60) | 3.85× (3.76–3.94) | 2.28× (2.23–2.33) | 1.87× (1.83–1.91) | 1.72× (1.69–1.75) | 2.26× (2.21–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.02–2.18) | 1.81× (1.76–1.86) | 2.37× (2.31–2.43) | 2.15× (2.11–2.20) | 2.29× (2.21–2.37) | 1.16× (1.14–1.18) | 0.95× (0.93–0.97) | 0.88× (0.86–0.89) | 1.15× (1.13–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 2.81× (2.77–2.85) | 2.57× (2.52–2.62) | 2.71× (2.69–2.73) | 3.02× (2.99–3.05) | 3.21× (3.19–3.23) | 3.30× (3.27–3.33) | 1.36× (1.35–1.37) | 1.49× (1.48–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.43× (1.39–1.46) | 2.67× (2.64–2.69) | 2.36× (2.34–2.38) | 3.23× (3.19–3.27) | 2.77× (2.75–2.79) | 2.88× (2.86–2.91) | 2.96× (2.94–2.99) | 1.22× (1.21–1.23) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.59–1.66) | 2.46× (2.40–2.52) | 2.08× (2.03–2.14) | 2.87× (2.79–2.94) | 2.72× (2.66–2.77) | 2.16× (2.12–2.20) | 2.22× (2.17–2.26) | 0.91× (0.89–0.93) | 1.00× (0.98–1.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.62–1.64) | 3.19× (3.16–3.23) | 2.79× (2.74–2.85) | 2.95× (2.92–2.97) | 3.35× (3.26–3.44) | 3.90× (3.87–3.94) | 4.19× (4.15–4.23) | 1.60× (1.58–1.62) | 1.85× (1.84–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.47× (1.43–1.51) | 2.93× (2.85–3.02) | 2.50× (2.43–2.57) | 3.55× (3.45–3.65) | 2.96× (2.87–3.05) | 3.42× (3.32–3.52) | 3.67× (3.56–3.78) | 1.40× (1.35–1.45) | 1.62× (1.57–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.65–1.79) | 2.74× (2.69–2.78) | 2.19× (2.15–2.24) | 3.10× (3.04–3.17) | 2.83× (2.78–2.87) | 2.50× (2.46–2.54) | 2.69× (2.65–2.73) | 1.02× (1.00–1.04) | 1.19× (1.17–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.43× (2.39–2.46) | 3.36× (3.33–3.40) | 2.80× (2.77–2.82) | 5.98× (5.88–6.08) | 2.09× (2.07–2.11) | 2.13× (2.11–2.15) | 0.94× (0.93–0.95) | 1.36× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.10× (2.06–2.15) | 3.72× (3.64–3.80) | 2.60× (2.52–2.68) | 5.26× (5.10–5.42) | 1.86× (1.82–1.90) | 1.89× (1.86–1.92) | 0.84× (0.82–0.85) | 1.20× (1.18–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.31× (2.29–2.34) | 3.72× (3.65–3.79) | 2.61× (2.58–2.63) | 5.35× (5.27–5.43) | 1.88× (1.86–1.90) | 1.91× (1.89–1.93) | 0.85× (0.84–0.86) | 1.22× (1.20–1.24) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 1.95× (1.83–2.08) | 3.86× (3.63–4.12) | 4.28× (4.02–4.55) | 4.69× (4.41–4.98) | 4.78× (4.49–5.10) | 4.56× (4.28–4.86) | 4.49× (4.21–4.79) | 1.60× (1.50–1.71) | 1.81× (1.69–1.93) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 1.59× (1.56–1.61) | 3.11× (3.04–3.18) | 3.49× (3.42–3.56) | 4.10× (4.02–4.19) | 3.80× (3.74–3.87) | 3.51× (3.44–3.57) | 3.46× (3.39–3.52) | 1.23× (1.21–1.25) | 1.39× (1.34–1.43) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 1.71× (1.67–1.74) | 2.57× (2.52–2.62) | 2.92× (2.82–3.03) | 3.23× (3.16–3.30) | 3.07× (3.00–3.13) | 2.31× (2.26–2.35) | 2.27× (2.23–2.31) | 0.81× (0.79–0.82) | 0.91× (0.88–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.78–0.79) | 11.00× (10.99–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.43× (0.36–0.51) | 0.72× (0.69–0.74) | 0.61× (0.59–0.62) | 5.74× (5.57–5.92) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.35–0.38) | 0.48× (0.41–0.57) | 0.64× (0.61–0.67) | 0.56× (0.53–0.59) | 3.73× (3.59–3.89) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.93× (0.92–0.93) | 4.83× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.94× (1.94–1.95) | 0.93× (0.92–0.93) | 4.82× (4.81–4.83) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.66–0.67) | 7.96× (7.92–7.99) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.66–0.72) | 0.68× (0.62–0.74) | 1.65× (1.58–1.71) | 0.83× (0.80–0.86) | 5.98× (5.75–6.22) | 0.49× (0.47–0.51) | 0.51× (0.49–0.53) | 0.56× (0.54–0.58) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.68–0.73) | 0.76× (0.72–0.80) | 1.25× (1.23–1.27) | 0.77× (0.76–0.79) | 3.69× (3.64–3.74) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 0.33× (0.33–0.34) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.74) | 5.15× (5.13–5.18) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.51) | 0.42× (0.41–0.43) | 0.91× (0.89–0.93) | 0.74× (0.72–0.76) | 3.14× (3.07–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.66) | 0.45× (0.45–0.46) | 0.87× (0.85–0.88) | 0.82× (0.81–0.84) | 2.53× (2.49–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.84) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.48) | 0.36× (0.34–0.39) | 0.83× (0.80–0.85) | 0.70× (0.66–0.74) | 2.85× (2.77–2.93) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.51) | 0.79× (0.78–0.80) | 0.82× (0.78–0.86) | 2.28× (2.25–2.31) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.05–1.06) | 0.76× (0.75–0.76) | 5.29× (5.29–5.30) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.89–0.91) | 6.20× (6.17–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.85× (1.85–1.86) | 0.90× (0.89–0.90) | 6.15× (6.13–6.17) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.38) | 0.66× (0.66–0.66) | 6.36× (6.35–6.37) | 0.39× (0.38–0.39) | 0.41× (0.41–0.41) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.38× (0.36–0.42) | 0.99× (0.96–1.01) | 0.67× (0.65–0.68) | 3.29× (3.21–3.36) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.28 MB → 15.2 MB, 6.7×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.44× (0.40–0.47) | 0.88× (0.86–0.90) | 0.74× (0.71–0.76) | 2.28× (2.23–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 550 | 1.82 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 976 | 1.98 | 1.08× (1.06–1.10) | 85% | 1.02× (1.02–1.03) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,182 | 3.10 | 1.70× (1.67–1.72) | 53% | 1.21× (1.20–1.23) | 1.67× (1.63–1.72) |

**Best:** mibz2 at 4 threads (1,182 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (361 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.39, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.51 | 0.02 | 0.07 | 0.177 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3697 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.6% |
| MTF + RLE2 | 1.48 | 17.7% |
| T-vector build | 1.49 | 17.8% |
| walk (the one each build uses) | 4.12 | 49.4% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.34 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.065 | (in its total) | 2.72× (2.68–2.75) |  |
| walk | 1.035 | 2.037 | (in its total) | 1.97× (1.94–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.040 | (in its total) | 1.05× (1.03–1.08) |  |
| inverse BWT, total | 1.467 | 3.143 | 1.879 | 2.15× (2.12–2.18) | 1.28× (1.27–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m39s ± 6m44s |
| 4 threads | 26m12s ± 3m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m31s ± 2m37s |
| the scanner alone (its serial part) | 7m43s ± 30.2s |
| the workers alone, over 4 | 18m31s ± 2m37s |

At 4 threads the workers bound it. Per byte: the scanner 4.5 ns a byte of bz2, the workers 2.43 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,277,600 | 15,175,604 | \`e0c1d543c056b938…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3696.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
