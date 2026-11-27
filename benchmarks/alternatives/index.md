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

The latest record: [`7a7a207`](<https://github.com/lmmx/mibz2/commit/7a7a207adfc2fee830b1b46f7fa6707d61271f0a>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45959147988>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.486 s | 432 | 6.2 | 3.13× (3.11–3.15) | 7.23× (7.16–7.29) | 7.45× (7.37–7.52) | 8.07× (7.99–8.15) | 7.29× (7.22–7.35) | 7.90× (7.83–7.97) | 7.39× (7.32–7.45) | 2.31× (2.30–2.33) | 2.67× (2.64–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.274 s | 764 | 6.6 | 3.14× (3.06–3.21) | 6.88× (6.64–7.13) | 7.85× (7.61–8.09) | 7.71× (7.50–7.93) | 6.93× (6.76–7.11) | 14.07× (13.73–14.42) | 13.16× (12.84–13.49) | 4.12× (4.02–4.23) | 4.76× (4.65–4.88) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 937 | 10.5 | 3.20× (3.16–3.24) | 4.95× (4.82–5.09) | 6.43× (5.98–6.91) | 5.62× (5.50–5.74) | 5.72× (5.56–5.88) | 17.17× (16.95–17.40) | 16.06× (15.85–16.28) | 5.03× (4.96–5.11) | 5.81× (5.71–5.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 962 | 3.0 | 0.60× (0.59–0.61) | 3.75× (3.56–3.95) | 4.46× (4.32–4.61) | 3.55× (3.51–3.59) | 6.16× (6.10–6.23) | 2.95× (2.91–2.98) | 2.63× (2.60–2.66) | 1.03× (1.00–1.06) | 1.49× (1.48–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 913 | 3.3 | 0.54× (0.53–0.55) | 2.82× (2.78–2.85) | 5.83× (5.66–6.00) | 3.40× (3.33–3.47) | 5.80× (5.74–5.87) | 2.78× (2.75–2.82) | 2.49× (2.45–2.52) | 0.97× (0.95–1.00) | 1.41× (1.39–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 905 | 3.3 | 0.56× (0.52–0.60) | 2.81× (2.76–2.85) | 5.78× (5.57–6.01) | 3.35× (3.29–3.40) | 5.81× (5.72–5.91) | 2.76× (2.72–2.81) | 2.47× (2.43–2.51) | 0.96× (0.93–1.00) | 1.40× (1.38–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 585 | 4.8 | 2.73× (2.65–2.81) | 3.03× (2.98–3.08) | 3.88× (3.82–3.95) | 3.85× (3.78–3.91) | 4.20× (4.14–4.26) | 2.56× (2.52–2.60) | 2.12× (2.09–2.15) | 2.04× (2.02–2.07) | 3.01× (2.97–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 891 | 5.8 | 2.32× (2.14–2.51) | 2.84× (2.69–3.00) | 4.21× (3.90–4.54) | 3.37× (3.15–3.60) | 4.83× (4.54–5.15) | 3.85× (3.62–4.09) | 3.19× (3.00–3.39) | 3.07× (2.89–3.26) | 4.53× (4.25–4.82) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 841 | 11.4 | 1.98× (1.91–2.04) | 2.19× (2.06–2.33) | 3.27× (3.08–3.48) | 2.39× (2.29–2.49) | 4.29× (4.14–4.45) | 3.70× (3.58–3.83) | 3.06× (2.96–3.16) | 2.95× (2.85–3.05) | 4.35× (4.21–4.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.122 s | 66 | 41.3 | 1.48× (1.47–1.49) | 2.95× (2.91–2.98) | 2.66× (2.63–2.69) | 2.68× (2.65–2.70) | 3.13× (3.10–3.17) | 3.45× (3.41–3.49) | 3.48× (3.46–3.51) | 1.33× (1.32–1.35) | 1.53× (1.51–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 s | 110 | 45.6 | 1.44× (1.41–1.46) | 2.79× (2.73–2.85) | 2.65× (2.59–2.71) | 3.40× (3.32–3.47) | 3.29× (3.21–3.37) | 5.72× (5.59–5.86) | 5.78× (5.65–5.91) | 2.21× (2.17–2.25) | 2.54× (2.48–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.061 s | 132 | 62.0 | 1.51× (1.48–1.55) | 2.43× (2.38–2.49) | 2.41× (2.38–2.44) | 2.92× (2.87–2.97) | 3.23× (3.19–3.28) | 6.92× (6.84–7.00) | 6.99× (6.89–7.09) | 2.68× (2.63–2.72) | 3.07× (3.00–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.136 s | 70 | 38.4 | 1.59× (1.57–1.60) | 3.27× (3.23–3.31) | 2.77× (2.73–2.80) | 2.82× (2.81–2.83) | 3.32× (3.29–3.35) | 3.91× (3.88–3.93) | 4.24× (4.19–4.28) | 1.52× (1.50–1.53) | 1.82× (1.80–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 42.9 | 1.51× (1.37–1.65) | 2.93× (2.84–3.02) | 2.66× (2.55–2.79) | 3.62× (3.48–3.75) | 3.30× (3.20–3.41) | 6.40× (6.19–6.62) | 6.95× (6.71–7.19) | 2.49× (2.41–2.57) | 2.99× (2.88–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 60.7 | 1.64× (1.60–1.68) | 2.54× (2.47–2.61) | 2.51× (2.43–2.59) | 3.32× (3.26–3.37) | 3.20× (3.14–3.25) | 7.80× (7.69–7.92) | 8.46× (8.31–8.62) | 3.03× (2.98–3.08) | 3.65× (3.56–3.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 722 | 4.2 | 0.66× (0.65–0.67) | 3.11× (2.90–3.33) | 4.35× (4.27–4.42) | 3.08× (3.03–3.13) | 6.76× (6.63–6.90) | 2.50× (2.46–2.54) | 2.30× (2.26–2.34) | 0.98× (0.96–1.00) | 1.43× (1.29–1.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 4.8 | 0.53× (0.53–0.54) | 2.73× (2.70–2.76) | 4.68× (4.62–4.74) | 2.85× (2.82–2.89) | 6.20× (6.11–6.30) | 2.29× (2.26–2.31) | 2.11× (2.08–2.13) | 0.90× (0.89–0.91) | 1.31× (1.17–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 4.7 | 0.54× (0.54–0.55) | 2.74× (2.71–2.76) | 4.68× (4.64–4.72) | 2.84× (2.82–2.87) | 6.22× (6.16–6.29) | 2.29× (2.26–2.31) | 2.10× (2.08–2.12) | 0.90× (0.89–0.91) | 1.31× (1.18–1.45) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 0.078 s | 229 | 11.9 | 2.19× (2.13–2.25) | 4.32× (4.24–4.41) | 4.72× (4.62–4.83) | 4.77× (4.67–4.88) | 4.88× (4.78–4.97) | 5.39× (5.27–5.51) | 5.11× (5.01–5.22) | 1.65× (1.60–1.70) | 2.08× (2.01–2.14) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 0.053 s | 339 | 15.0 | 1.80× (1.74–1.86) | 3.67× (3.41–3.94) | 4.30× (4.14–4.46) | 4.60× (4.40–4.80) | 4.44× (4.30–4.59) | 8.07× (7.82–8.33) | 7.65× (7.41–7.90) | 2.47× (2.39–2.56) | 3.11× (2.99–3.23) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 0.046 s | 390 | 23.0 | 1.79× (1.71–1.87) | 2.81× (2.60–3.04) | 3.74× (3.65–3.84) | 3.61× (3.51–3.71) | 3.91× (3.82–4.00) | 9.16× (8.98–9.35) | 8.69× (8.51–8.87) | 2.81× (2.75–2.87) | 3.53× (3.42–3.64) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.12× (3.10–3.14) | 7.19× (7.13–7.25) | 7.41× (7.34–7.49) | 8.04× (7.96–8.12) | 7.26× (7.20–7.32) | 7.87× (7.80–7.94) | 7.36× (7.30–7.42) | 2.31× (2.29–2.33) | 2.65× (2.62–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.03–3.07) | 6.91× (6.81–7.00) | 7.31× (7.21–7.42) | 7.99× (7.88–8.10) | 6.87× (6.83–6.90) | 7.38× (7.34–7.42) | 6.90× (6.87–6.94) | 2.17× (2.15–2.18) | 2.49× (2.47–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (3.02–3.11) | 5.24× (5.19–5.29) | 5.68× (5.57–5.79) | 5.93× (5.85–6.00) | 5.34× (5.22–5.47) | 4.66× (4.61–4.72) | 4.36× (4.31–4.41) | 1.37× (1.35–1.38) | 1.57× (1.55–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.24× (3.20–3.28) | 4.06× (3.99–4.14) | 3.33× (3.29–3.36) | 5.70× (5.65–5.76) | 2.77× (2.74–2.81) | 2.49× (2.46–2.51) | 1.03× (1.00–1.05) | 1.06× (1.04–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.85× (2.81–2.89) | 5.03× (4.99–5.08) | 3.10× (3.04–3.17) | 5.21× (5.16–5.26) | 2.54× (2.51–2.57) | 2.27× (2.25–2.30) | 0.94× (0.92–0.96) | 0.97× (0.95–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.54–0.62) | 3.15× (3.10–3.20) | 5.07× (4.96–5.19) | 3.10× (3.04–3.15) | 5.26× (5.17–5.36) | 2.55× (2.50–2.60) | 2.28× (2.24–2.33) | 0.94× (0.91–0.98) | 0.97× (0.95–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.55–2.71) | 2.79× (2.75–2.84) | 3.53× (3.48–3.58) | 3.69× (3.63–3.74) | 4.02× (3.97–4.07) | 2.48× (2.44–2.51) | 2.06× (2.03–2.09) | 1.98× (1.96–2.01) | 2.74× (2.70–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.25–2.40) | 2.61× (2.48–2.75) | 3.17× (3.06–3.28) | 3.22× (3.12–3.33) | 3.48× (3.36–3.60) | 2.05× (1.99–2.12) | 1.71× (1.65–1.76) | 1.64× (1.59–1.70) | 2.27× (2.19–2.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.99–2.15) | 1.79× (1.70–1.88) | 2.27× (2.20–2.35) | 2.10× (2.02–2.20) | 2.19× (2.11–2.27) | 1.05× (1.01–1.09) | 0.88× (0.85–0.91) | 0.84× (0.82–0.87) | 1.17× (1.13–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.47–1.48) | 2.89× (2.85–2.93) | 2.58× (2.55–2.61) | 2.65× (2.63–2.67) | 3.10× (3.07–3.13) | 3.41× (3.37–3.45) | 3.44× (3.42–3.47) | 1.33× (1.32–1.34) | 1.49× (1.48–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.39–1.43) | 2.69× (2.64–2.73) | 2.38× (2.35–2.42) | 3.17× (3.12–3.22) | 2.82× (2.78–2.85) | 3.05× (3.00–3.11) | 3.08× (3.03–3.13) | 1.19× (1.17–1.21) | 1.33× (1.31–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.64× (1.60–1.68) | 2.56× (2.52–2.61) | 2.06× (2.01–2.10) | 2.86× (2.80–2.93) | 2.70× (2.66–2.74) | 2.26× (2.23–2.30) | 2.29× (2.24–2.33) | 0.88× (0.87–0.90) | 0.99× (0.97–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.58× (1.57–1.59) | 3.20× (3.17–3.23) | 2.70× (2.66–2.74) | 2.79× (2.78–2.81) | 3.29× (3.26–3.31) | 3.86× (3.84–3.88) | 4.19× (4.14–4.23) | 1.51× (1.50–1.53) | 1.78× (1.76–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.47–1.49) | 2.98× (2.95–3.02) | 2.49× (2.47–2.52) | 3.47× (3.43–3.52) | 2.99× (2.97–3.02) | 3.45× (3.41–3.49) | 3.74× (3.69–3.80) | 1.35× (1.34–1.36) | 1.59× (1.57–1.62) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.76× (1.73–1.78) | 2.76× (2.72–2.80) | 2.12× (2.09–2.15) | 3.01× (2.97–3.06) | 2.77× (2.73–2.81) | 2.44× (2.41–2.47) | 2.65× (2.61–2.69) | 0.96× (0.94–0.97) | 1.13× (1.11–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.70× (2.65–2.75) | 3.56× (3.48–3.64) | 2.82× (2.78–2.87) | 6.03× (5.92–6.16) | 2.31× (2.27–2.35) | 2.14× (2.10–2.17) | 0.99× (0.97–1.00) | 0.98× (0.94–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.58–0.59) | 2.34× (2.32–2.35) | 4.00× (3.95–4.04) | 2.57× (2.54–2.61) | 5.35× (5.27–5.43) | 2.03× (2.01–2.05) | 1.88× (1.86–1.90) | 0.87× (0.86–0.88) | 0.86× (0.83–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.59× (2.55–2.63) | 4.02× (3.98–4.06) | 2.62× (2.59–2.65) | 5.47× (5.41–5.52) | 2.07× (2.05–2.09) | 1.92× (1.89–1.94) | 0.88× (0.87–0.89) | 0.88× (0.84–0.92) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 2.15× (2.10–2.21) | 4.18× (4.11–4.26) | 4.57× (4.46–4.68) | 4.68× (4.58–4.78) | 4.78× (4.69–4.87) | 5.28× (5.17–5.39) | 5.01× (4.91–5.11) | 1.63× (1.59–1.68) | 2.00× (1.95–2.06) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 1.86× (1.82–1.89) | 3.54× (3.46–3.63) | 3.86× (3.79–3.93) | 4.30× (4.21–4.40) | 3.91× (3.84–3.98) | 4.26× (4.17–4.34) | 4.04× (3.96–4.12) | 1.32× (1.29–1.34) | 1.62× (1.57–1.66) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 1.87× (1.81–1.93) | 2.84× (2.77–2.91) | 3.11× (3.07–3.16) | 3.38× (3.30–3.47) | 3.13× (3.08–3.19) | 2.75× (2.70–2.81) | 2.61× (2.56–2.66) | 0.85× (0.83–0.87) | 1.04× (1.02–1.07) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.30× (0.28–0.31) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.07× (11.06–11.09) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.52× (0.42–0.64) | 0.73× (0.71–0.75) | 0.61× (0.59–0.63) | 5.77× (5.60–5.94) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.36× (0.34–0.37) | 0.70× (0.57–0.86) | 0.64× (0.61–0.67) | 0.56× (0.54–0.59) | 3.71× (3.56–3.86) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.18–0.18) | 1.97× (1.97–1.97) | 0.93× (0.92–0.93) | 4.87× (4.86–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.71) | 0.66× (0.66–0.67) | 8.03× (7.99–8.06) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.75) | 0.78× (0.72–0.84) | 1.72× (1.68–1.77) | 0.86× (0.84–0.89) | 6.24× (6.07–6.41) | 0.51× (0.49–0.52) | 0.53× (0.52–0.55) | 0.58× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.71) | 0.77× (0.71–0.84) | 1.25× (1.22–1.29) | 0.77× (0.75–0.80) | 3.69× (3.58–3.80) | 0.29× (0.28–0.30) | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.72–0.73) | 5.17× (5.15–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.38× (0.35–0.41) | 0.91× (0.87–0.94) | 0.72× (0.69–0.75) | 3.13× (3.03–3.23) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.66–0.68) | 0.47× (0.46–0.48) | 0.89× (0.88–0.90) | 0.83× (0.81–0.86) | 2.59× (2.55–2.63) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.47) | 0.36× (0.34–0.39) | 0.83× (0.80–0.85) | 0.67× (0.64–0.71) | 2.84× (2.76–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.50× (0.47–0.53) | 0.80× (0.78–0.81) | 0.82× (0.80–0.84) | 2.31× (2.26–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.31–5.35) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.90–0.90) | 6.20× (6.16–6.24) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.88–1.89) | 0.90× (0.89–0.91) | 6.23× (6.20–6.25) | 0.41× (0.41–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.68× (0.68–0.69) | 6.51× (6.49–6.54) | 0.38× (0.38–0.39) | 0.41× (0.40–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 39 MB | 0.45× (0.43–0.48) | 0.45× (0.41–0.48) | 1.02× (0.97–1.08) | 0.67× (0.64–0.71) | 3.42× (3.25–3.61) | 0.20× (0.19–0.21) | 0.22× (0.21–0.23) | 0.33× (0.31–0.35) | 0.16× (0.15–0.17) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 58 MB | 0.48× (0.47–0.49) | 0.48× (0.45–0.51) | 0.88× (0.86–0.90) | 0.73× (0.71–0.75) | 2.30× (2.26–2.34) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 432 | 2.33 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 764 | 2.49 | 1.07× (1.05–1.08) | 89% | 1.02× (1.01–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 937 | 3.95 | 1.69× (1.67–1.70) | 54% | 1.23× (1.21–1.25) | 1.66× (1.63–1.68) |

**Best:** mibz2 at 4 threads (937 MB/s); lbzip2 at 4 threads (189 MB/s); pbz2 at 4 threads (292 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.00, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.66 | 0.02 | 0.08 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2664 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.70 | 9.9% |
| MTF + RLE2 | 1.24 | 17.7% |
| T-vector build | 1.25 | 17.7% |
| walk (the one each build uses) | 3.41 | 48.4% |
| of which: segment assembly | 0.13 | 1.9% |
| RLE1 | 0.23 | 3.3% |
| CRC | 0.21 | 3.0% |
| **total** | 7.05 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.512 | 1.303 | (in its total) | 2.55× (2.54–2.55) |  |
| walk | 1.262 | 2.645 | (in its total) | 2.10× (2.08–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.20× (1.19–1.21) |  |
| inverse BWT, total | 1.821 | 4.005 | 3.286 | 2.20× (2.19–2.21) | 1.80× (1.79–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h10m ± 8m33s |
| 4 threads | 32m39s ± 4m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m45s ± 3m22s |
| the scanner alone (its serial part) | 10m11s ± 29.4s |
| the workers alone, over 4 | 23m45s ± 3m22s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.11 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,762,129 | 17,993,791 | \`eb77d838a1e70624…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2664.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
