# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1110/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`b889740`](<https://github.com/lmmx/mibz2/commit/b88974077b5d4a125a8a6e722475d77129735117>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45952032179>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.290 s | 724 | 5.8 | 2.99× (2.94–3.04) | 7.41× (7.30–7.52) | 7.66× (7.58–7.75) | 8.80× (8.70–8.91) | 7.75× (7.68–7.81) | 8.06× (8.00–8.13) | 7.69× (7.55–7.82) | 2.33× (2.30–2.37) | 2.68× (2.65–2.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.163 s | 1,288 | 6.3 | 3.07× (2.96–3.18) | 6.78× (6.61–6.96) | 8.07× (7.86–8.28) | 8.26× (8.05–8.48) | 7.44× (7.22–7.66) | 14.08× (13.69–14.49) | 13.43× (12.94–13.93) | 4.07× (3.95–4.21) | 4.68× (4.55–4.82) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.137 s | 1,530 | 9.7 | 2.93× (2.86–2.99) | 4.75× (4.69–4.81) | 6.17× (6.01–6.34) | 5.81× (5.71–5.91) | 5.86× (5.74–5.98) | 16.95× (16.59–17.33) | 16.16× (15.69–16.65) | 4.91× (4.78–5.04) | 5.64× (5.50–5.78) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,413 | 3.2 | 0.58× (0.56–0.59) | 3.65× (3.55–3.76) | 4.52× (4.39–4.65) | 3.76× (3.69–3.83) | 6.66× (6.55–6.76) | 2.86× (2.80–2.91) | 2.83× (2.77–2.89) | 1.00× (0.98–1.01) | 2.19× (2.13–2.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,324 | 3.6 | 0.50× (0.48–0.51) | 2.84× (2.67–3.03) | 5.61× (5.40–5.83) | 3.53× (3.45–3.62) | 6.15× (6.02–6.27) | 2.65× (2.59–2.72) | 2.63× (2.57–2.69) | 0.92× (0.90–0.95) | 2.04× (1.97–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,320 | 3.6 | 0.49× (0.48–0.51) | 2.76× (2.60–2.94) | 5.50× (5.36–5.63) | 3.52× (3.43–3.61) | 6.11× (5.92–6.29) | 2.62× (2.55–2.70) | 2.60× (2.52–2.68) | 0.91× (0.89–0.94) | 2.01× (1.97–2.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,060 | 4.2 | 2.96× (2.91–3.01) | 3.71× (3.50–3.93) | 4.37× (4.28–4.47) | 4.64× (4.54–4.75) | 5.45× (5.34–5.57) | 3.12× (3.01–3.23) | 2.62× (2.57–2.66) | 2.20× (2.16–2.24) | 3.33× (3.23–3.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,474 | 5.2 | 2.17× (1.95–2.42) | 3.02× (2.71–3.36) | 4.15× (3.63–4.76) | 3.78× (3.35–4.26) | 5.67× (5.03–6.39) | 4.21× (3.83–4.63) | 3.53× (3.19–3.91) | 2.97× (2.68–3.29) | 4.49× (4.03–5.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.014 s | 1,436 | 10.1 | 1.87× (1.61–2.17) | 2.12× (1.81–2.49) | 3.07× (2.58–3.65) | 2.25× (1.91–2.67) | 4.63× (3.97–5.40) | 3.84× (3.25–4.52) | 3.22× (2.75–3.76) | 2.71× (2.31–3.17) | 4.09× (3.48–4.80) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 103 | 41.1 | 1.45× (1.43–1.46) | 3.28× (3.26–3.31) | 2.94× (2.91–2.98) | 2.90× (2.85–2.95) | 3.29× (3.23–3.35) | 3.67× (3.61–3.73) | 3.72× (3.67–3.77) | 1.41× (1.40–1.43) | 1.60× (1.56–1.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.048 s | 167 | 46.7 | 1.38× (1.32–1.43) | 3.10× (2.94–3.27) | 2.80× (2.66–2.95) | 3.60× (3.31–3.92) | 3.30× (3.16–3.45) | 5.90× (5.65–6.16) | 5.99× (5.74–6.26) | 2.28× (2.18–2.37) | 2.57× (2.47–2.68) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.039 s | 207 | 62.4 | 1.40× (1.36–1.43) | 2.64× (2.57–2.71) | 2.63× (2.52–2.75) | 3.05× (2.96–3.15) | 3.40× (3.33–3.48) | 7.33× (7.15–7.52) | 7.45× (7.27–7.63) | 2.83× (2.76–2.90) | 3.20× (3.08–3.32) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.083 s | 116 | 36.7 | 1.51× (1.50–1.53) | 3.82× (3.78–3.87) | 3.12× (3.09–3.15) | 3.18× (3.15–3.20) | 3.57× (3.52–3.61) | 4.57× (4.52–4.61) | 4.92× (4.87–4.97) | 1.64× (1.62–1.67) | 1.90× (1.86–1.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.052 s | 186 | 41.8 | 1.48× (1.31–1.67) | 3.66× (3.27–4.10) | 2.95× (2.86–3.05) | 4.04× (3.93–4.15) | 3.48× (3.38–3.59) | 7.43× (7.26–7.60) | 8.01× (7.83–8.18) | 2.67× (2.60–2.75) | 3.09× (3.01–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.043 s | 224 | 59.0 | 1.43× (1.37–1.50) | 2.76× (2.62–2.91) | 2.63× (2.49–2.78) | 3.53× (3.41–3.65) | 3.30× (3.17–3.43) | 8.64× (8.31–8.99) | 9.32× (8.97–9.68) | 3.11× (3.00–3.23) | 3.59× (3.46–3.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,081 | 4.4 | 0.64× (0.63–0.65) | 3.36× (3.29–3.45) | 4.70× (4.58–4.83) | 3.22× (3.15–3.30) | 7.33× (7.22–7.45) | 2.44× (2.42–2.46) | 2.46× (2.42–2.49) | 1.00× (0.98–1.03) | 2.25× (2.20–2.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 5.2 | 0.49× (0.48–0.50) | 3.02× (2.94–3.11) | 5.07× (4.53–5.68) | 2.91× (2.84–2.98) | 6.61× (6.49–6.73) | 2.18× (2.16–2.21) | 2.20× (2.17–2.22) | 0.90× (0.88–0.91) | 2.01× (1.95–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 943 | 5.2 | 0.49× (0.48–0.50) | 2.92× (2.86–2.99) | 4.82× (4.48–5.20) | 2.89× (2.83–2.95) | 6.44× (6.29–6.59) | 2.11× (2.06–2.16) | 2.12× (2.07–2.18) | 0.87× (0.84–0.89) | 1.94× (1.90–1.99) | yes |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 1 | 0.046 s | 387 | 11.1 | 2.01× (1.98–2.05) | 4.52× (4.44–4.60) | 4.91× (4.81–5.02) | 5.22× (5.10–5.34) | 5.33× (5.22–5.44) | 5.36× (5.25–5.48) | 5.19× (5.10–5.28) | 1.72× (1.70–1.75) | 2.01× (1.97–2.06) | yes |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 2 | 0.034 s | 532 | 14.5 | 1.46× (1.30–1.63) | 3.43× (3.04–3.86) | 4.10× (3.64–4.63) | 4.49× (4.05–4.98) | 4.38× (3.94–4.88) | 7.15× (6.45–7.94) | 6.92× (6.20–7.73) | 2.30× (2.06–2.56) | 2.69× (2.40–3.01) | yes |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 4 | 0.028 s | 634 | 22.4 | 1.52× (1.46–1.58) | 2.66× (2.51–2.81) | 3.69× (3.53–3.85) | 3.73× (3.56–3.91) | 4.23× (4.05–4.43) | 8.71× (8.32–9.11) | 8.43× (8.04–8.83) | 2.80× (2.66–2.94) | 3.27× (3.10–3.45) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.98× (2.93–3.03) | 7.36× (7.25–7.47) | 7.61× (7.53–7.70) | 8.77× (8.67–8.87) | 7.71× (7.65–7.78) | 8.03× (7.97–8.09) | 7.66× (7.52–7.79) | 2.33× (2.29–2.36) | 2.66× (2.63–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.81× (2.77–2.85) | 6.99× (6.94–7.03) | 7.34× (7.26–7.42) | 8.54× (8.47–8.61) | 7.31× (7.22–7.39) | 7.40× (7.35–7.45) | 7.05× (6.94–7.17) | 2.14× (2.11–2.18) | 2.45× (2.43–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.77× (2.67–2.88) | 5.25× (5.16–5.33) | 5.70× (5.58–5.83) | 6.43× (6.32–6.54) | 5.60× (5.46–5.75) | 4.80× (4.68–4.92) | 4.58× (4.46–4.70) | 1.39× (1.35–1.43) | 1.59× (1.55–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.19× (3.11–3.27) | 3.97× (3.87–4.06) | 3.55× (3.48–3.62) | 6.22× (6.12–6.33) | 2.73× (2.68–2.79) | 2.69× (2.64–2.74) | 1.00× (0.99–1.02) | 1.91× (1.87–1.96) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.56) | 2.67× (2.60–2.74) | 4.65× (4.51–4.79) | 3.21× (3.13–3.29) | 5.47× (5.39–5.54) | 2.41× (2.36–2.47) | 2.38× (2.33–2.42) | 0.88× (0.87–0.90) | 1.69× (1.66–1.72) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.54) | 2.86× (2.80–2.92) | 4.76× (4.64–4.88) | 3.24× (3.16–3.33) | 5.52× (5.36–5.68) | 2.43× (2.36–2.50) | 2.39× (2.32–2.46) | 0.89× (0.86–0.92) | 1.70× (1.64–1.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.83× (2.77–2.89) | 3.31× (3.13–3.49) | 3.84× (3.77–3.90) | 4.42× (4.31–4.53) | 5.17× (5.05–5.29) | 3.00× (2.89–3.12) | 2.52× (2.47–2.57) | 2.13× (2.09–2.17) | 2.86× (2.81–2.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.29–2.47) | 2.73× (2.59–2.88) | 3.31× (3.16–3.47) | 3.71× (3.54–3.90) | 4.24× (4.00–4.48) | 2.37× (2.25–2.49) | 1.99× (1.90–2.08) | 1.68× (1.61–1.75) | 2.26× (2.16–2.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.78–2.27) | 1.85× (1.63–2.09) | 2.21× (1.97–2.48) | 2.18× (1.94–2.46) | 2.42× (2.15–2.72) | 1.20× (1.05–1.38) | 1.01× (0.89–1.14) | 0.85× (0.75–0.96) | 1.14× (1.01–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.43–1.46) | 3.18× (3.16–3.20) | 2.86× (2.84–2.88) | 2.87× (2.83–2.92) | 3.25× (3.19–3.31) | 3.63× (3.57–3.69) | 3.68× (3.63–3.73) | 1.41× (1.39–1.43) | 1.53× (1.50–1.57) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.37× (1.31–1.43) | 2.96× (2.94–2.99) | 2.55× (2.51–2.60) | 3.35× (3.29–3.42) | 2.90× (2.84–2.96) | 3.22× (3.17–3.28) | 3.27× (3.21–3.33) | 1.25× (1.23–1.27) | 1.36× (1.33–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.40–1.49) | 2.70× (2.66–2.73) | 2.25× (2.21–2.29) | 2.93× (2.89–2.98) | 2.77× (2.74–2.80) | 2.39× (2.34–2.44) | 2.43× (2.39–2.47) | 0.93× (0.91–0.95) | 1.01× (0.99–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.49–1.52) | 3.72× (3.68–3.75) | 3.04× (3.01–3.06) | 3.14× (3.11–3.17) | 3.53× (3.48–3.57) | 4.51× (4.47–4.56) | 4.86× (4.81–4.91) | 1.63× (1.61–1.66) | 1.80× (1.77–1.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.37–1.40) | 3.36× (3.32–3.40) | 2.70× (2.66–2.74) | 3.78× (3.73–3.83) | 3.12× (3.08–3.17) | 3.96× (3.91–4.02) | 4.27× (4.23–4.31) | 1.43× (1.42–1.45) | 1.58× (1.57–1.60) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.63) | 3.04× (2.99–3.09) | 2.27× (2.23–2.31) | 3.28× (3.23–3.33) | 2.92× (2.87–2.97) | 2.82× (2.77–2.87) | 3.04× (2.98–3.10) | 1.02× (1.01–1.04) | 1.13× (1.10–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.70) | 2.66× (2.60–2.72) | 3.47× (3.41–3.54) | 2.96× (2.88–3.05) | 6.59× (6.44–6.75) | 2.30× (2.27–2.33) | 2.28× (2.23–2.34) | 1.00× (0.97–1.03) | 1.58× (1.55–1.61) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.53–0.54) | 2.23× (2.15–2.30) | 3.85× (3.52–4.21) | 2.58× (2.53–2.63) | 5.60× (5.52–5.69) | 1.93× (1.91–1.95) | 1.92× (1.90–1.94) | 0.84× (0.83–0.85) | 1.33× (1.32–1.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.51–0.53) | 2.31× (2.25–2.38) | 3.69× (3.59–3.80) | 2.63× (2.58–2.68) | 5.60× (5.48–5.72) | 1.92× (1.88–1.96) | 1.90× (1.85–1.95) | 0.83× (0.81–0.86) | 1.31× (1.28–1.35) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 1 | 1.99× (1.95–2.02) | 4.38× (4.30–4.46) | 4.74× (4.66–4.83) | 5.11× (5.00–5.23) | 5.22× (5.11–5.33) | 5.26× (5.14–5.38) | 5.08× (4.99–5.18) | 1.70× (1.68–1.73) | 1.89× (1.86–1.91) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 2 | 1.61× (1.57–1.64) | 3.54× (3.47–3.62) | 3.85× (3.77–3.93) | 4.50× (4.41–4.59) | 4.12× (4.04–4.20) | 4.07× (3.97–4.17) | 3.94× (3.83–4.04) | 1.32× (1.29–1.35) | 1.46× (1.43–1.49) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 4 | 1.63× (1.59–1.67) | 2.79× (2.73–2.86) | 3.03× (2.99–3.08) | 3.46× (3.42–3.50) | 3.26× (3.21–3.30) | 2.61× (2.54–2.68) | 2.53× (2.46–2.59) | 0.85× (0.82–0.87) | 0.94× (0.92–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.80) | 10.97× (10.95–10.99) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.50× (0.44–0.57) | 0.72× (0.70–0.74) | 0.61× (0.59–0.62) | 5.76× (5.61–5.91) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.38) | 0.55× (0.42–0.72) | 0.64× (0.61–0.67) | 0.56× (0.53–0.58) | 3.72× (3.55–3.89) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.86× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.95× (7.93–7.97) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.71) | 0.62× (0.58–0.67) | 1.61× (1.56–1.67) | 0.81× (0.78–0.84) | 5.84× (5.64–6.05) | 0.48× (0.46–0.49) | 0.49× (0.48–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.68× (0.64–0.72) | 0.76× (0.70–0.83) | 1.18× (1.11–1.27) | 0.72× (0.68–0.77) | 3.44× (3.23–3.67) | 0.27× (0.25–0.29) | 0.28× (0.26–0.30) | 0.31× (0.29–0.33) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.15–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.13) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.41× (0.40–0.43) | 0.89× (0.87–0.91) | 0.70× (0.67–0.74) | 3.08× (3.00–3.16) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.45–0.47) | 0.89× (0.87–0.90) | 0.82× (0.79–0.86) | 2.59× (2.54–2.63) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.36× (0.34–0.39) | 0.83× (0.79–0.86) | 0.69× (0.65–0.72) | 2.84× (2.72–2.97) | 0.18× (0.17–0.19) | 0.19× (0.18–0.19) | 0.28× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.61× (0.59–0.63) | 0.45× (0.43–0.48) | 0.80× (0.79–0.81) | 0.81× (0.79–0.84) | 2.30× (2.27–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.31× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.24–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.89× (0.89–0.90) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.37× (1.37–1.37) | 0.67× (0.67–0.68) | 6.45× (6.44–6.47) | 0.38× (0.38–0.39) | 0.40× (0.40–0.41) | 0.62× (0.61–0.62) | 0.30× (0.30–0.30) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.44× (0.43–0.45) | 1.03× (1.00–1.07) | 0.68× (0.64–0.71) | 3.43× (3.33–3.53) | 0.21× (0.20–0.21) | 0.21× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.15–0.17) |
| `text` (2.75 MB → 17.9 MB, 6.5×) | 4 | 57 MB | 0.50× (0.48–0.51) | 0.47× (0.43–0.52) | 0.89× (0.87–0.91) | 0.74× (0.73–0.76) | 2.34× (2.29–2.39) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 724 | 1.39 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,288 | 1.51 | 1.09× (1.07–1.10) | 87% | 1.03× (1.02–1.04) | 1.02× (1.00–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,530 | 2.32 | 1.67× (1.63–1.72) | 53% | 1.19× (1.17–1.22) | 1.56× (1.50–1.61) |

**Best:** mibz2 at 4 threads (1,530 MB/s); lbzip2 at 4 threads (321 MB/s); pbz2 at 4 threads (522 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.06, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 13%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.04 | 0.37 | 0.01 | 0.06 | 0.137 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4183 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.75 | 11.2% |
| MTF + RLE2 | 1.52 | 22.8% |
| T-vector build | 1.29 | 19.3% |
| walk (the one each build uses) | 2.72 | 40.7% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.7% |
| **total** | 6.69 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.299 | 0.716 | (in its total) | 2.40× (2.35–2.47) |  |
| walk | 0.617 | 1.591 | (in its total) | 2.59× (2.52–2.65) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.030 | (in its total) | 1.17× (1.14–1.21) |  |
| inverse BWT, total | 0.942 | 2.337 | 1.327 | 2.49× (2.43–2.55) | 1.41× (1.40–1.43) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 41m49s ± 5m08s |
| 4 threads | 19m53s ± 2m51s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m19s ± 1m54s |
| the scanner alone (its serial part) | 5m49s ± 0.00s |
| the workers alone, over 4 | 13m19s ± 1m54s |

At 4 threads the workers bound it. Per byte: the scanner 3.4 ns a byte of bz2, the workers 1.75 ns a byte of output at 4 threads (CPU, summed over them), 1.18 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,752,714 | 17,924,005 | \`193efe32dda0ced5…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4182.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
