# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-519/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`9cef5a9`](<https://github.com/lmmx/mibz2/commit/9cef5a99469d6a8f2caed4651a1e2754ffa4532b>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43981603574>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.311 s | 674 | 6.8 | 2.67× (2.64–2.70) | 6.73× (6.61–6.86) | 7.00× (6.88–7.11) | 8.01× (7.90–8.13) | 7.14× (7.03–7.26) | 7.31× (7.19–7.43) | 6.98× (6.86–7.10) | 2.14× (2.09–2.19) | 2.42× (2.39–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.181 s | 1,158 | 7.4 | 2.74× (2.62–2.87) | 6.07× (5.77–6.38) | 7.32× (7.05–7.61) | 7.50× (7.09–7.94) | 6.63× (6.32–6.96) | 12.35× (11.93–12.79) | 11.79× (11.37–12.23) | 3.61× (3.44–3.80) | 4.09× (3.96–4.21) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.153 s | 1,370 | 11.2 | 2.74× (2.65–2.83) | 4.26× (4.15–4.37) | 5.67× (5.42–5.94) | 5.21× (4.94–5.49) | 5.25× (5.09–5.41) | 14.80× (14.42–15.19) | 14.13× (13.68–14.59) | 4.33× (4.19–4.46) | 4.89× (4.77–5.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,454 | 3.5 | 0.58× (0.56–0.59) | 3.70× (3.60–3.80) | 4.56× (4.44–4.70) | 3.77× (3.69–3.86) | 6.77× (6.48–7.07) | 2.87× (2.78–2.96) | 2.82× (2.76–2.89) | 1.01× (0.99–1.03) | 2.22× (2.16–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,413 | 3.6 | 0.51× (0.49–0.53) | 2.87× (2.76–2.99) | 5.88× (5.65–6.13) | 3.65× (3.49–3.81) | 6.39× (6.14–6.66) | 2.78× (2.69–2.88) | 2.74× (2.65–2.83) | 0.98× (0.94–1.01) | 2.15× (2.07–2.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,424 | 3.5 | 0.53× (0.51–0.55) | 3.02× (2.70–3.37) | 5.89× (5.72–6.07) | 3.69× (3.57–3.80) | 6.42× (6.18–6.67) | 2.79× (2.66–2.93) | 2.74× (2.65–2.84) | 0.98× (0.95–1.01) | 2.16× (2.10–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,086 | 4.5 | 2.89× (2.79–3.00) | 3.61× (3.41–3.81) | 4.32× (4.16–4.49) | 4.75× (4.59–4.93) | 5.43× (5.31–5.56) | 3.09× (3.01–3.16) | 2.60× (2.54–2.66) | 2.17× (2.13–2.21) | 3.37× (3.32–3.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,562 | 5.9 | 2.05× (1.67–2.50) | 2.86× (2.33–3.50) | 3.94× (3.31–4.68) | 3.60× (2.87–4.52) | 5.44× (4.49–6.60) | 3.89× (3.13–4.82) | 3.27× (2.66–4.03) | 2.73× (2.23–3.34) | 4.25× (3.47–5.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,217 | 12.0 | 1.68× (1.58–1.78) | 1.94× (1.82–2.06) | 2.78× (2.64–2.92) | 1.99× (1.89–2.10) | 4.13× (3.92–4.35) | 3.39× (3.22–3.58) | 2.86× (2.73–3.00) | 2.38× (2.26–2.51) | 3.71× (3.51–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.078 s | 102 | 45.2 | 1.39× (1.36–1.42) | 3.11× (3.04–3.19) | 2.82× (2.75–2.88) | 2.75× (2.69–2.82) | 3.08× (3.01–3.15) | 3.51× (3.42–3.60) | 3.54× (3.46–3.61) | 1.37× (1.34–1.40) | 1.54× (1.50–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.050 s | 162 | 51.8 | 1.30× (1.24–1.35) | 2.88× (2.77–3.00) | 2.70× (2.60–2.82) | 3.40× (3.24–3.57) | 3.08× (2.99–3.17) | 5.52× (5.35–5.70) | 5.57× (5.37–5.77) | 2.15× (2.08–2.23) | 2.42× (2.34–2.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 194 | 69.1 | 1.30× (1.25–1.36) | 2.49× (2.44–2.54) | 2.45× (2.31–2.60) | 2.91× (2.76–3.08) | 3.13× (3.02–3.24) | 6.72× (6.51–6.94) | 6.78× (6.54–7.02) | 2.62× (2.53–2.71) | 2.95× (2.86–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.082 s | 117 | 39.6 | 1.52× (1.46–1.59) | 3.75× (3.69–3.81) | 3.10× (3.05–3.15) | 3.21× (3.05–3.37) | 3.53× (3.46–3.60) | 4.48× (4.39–4.57) | 4.81× (4.71–4.91) | 1.68× (1.65–1.71) | 1.94× (1.83–2.05) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.052 s | 183 | 46.7 | 1.28× (1.18–1.40) | 3.15× (2.87–3.45) | 2.73× (2.51–2.97) | 3.72× (3.41–4.07) | 3.24× (2.90–3.62) | 6.71× (6.14–7.33) | 7.21× (6.63–7.84) | 2.52× (2.30–2.76) | 2.91× (2.61–3.24) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.046 s | 209 | 65.0 | 1.37× (1.32–1.41) | 2.66× (2.55–2.78) | 2.54× (2.48–2.61) | 3.33× (3.18–3.49) | 3.13× (3.01–3.25) | 8.10× (7.87–8.34) | 8.70× (8.45–8.96) | 3.04× (2.94–3.15) | 3.51× (3.30–3.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,093 | 4.8 | 0.62× (0.61–0.64) | 3.33× (3.22–3.44) | 4.64× (4.48–4.80) | 3.24× (3.13–3.35) | 7.17× (6.97–7.37) | 2.36× (2.29–2.43) | 2.48× (2.32–2.66) | 0.99× (0.97–1.01) | 2.23× (2.15–2.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,036 | 5.2 | 0.51× (0.49–0.53) | 3.14× (3.02–3.27) | 5.34× (5.06–5.63) | 3.03× (2.94–3.13) | 6.76× (6.53–7.00) | 2.22× (2.15–2.30) | 2.34× (2.17–2.53) | 0.93× (0.90–0.96) | 2.10× (2.01–2.19) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,000 | 5.1 | 0.49× (0.44–0.54) | 2.97× (2.71–3.26) | 4.92× (4.45–5.44) | 2.89× (2.66–3.14) | 6.54× (5.88–7.28) | 2.10× (1.91–2.30) | 2.21× (1.97–2.47) | 0.88× (0.80–0.96) | 1.98× (1.80–2.17) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 0.041 s | 297 | 15.8 | 1.72× (1.66–1.78) | 4.19× (4.09–4.29) | 4.52× (4.41–4.63) | 4.66× (4.57–4.76) | 4.94× (4.86–5.03) | 5.01× (4.93–5.08) | 4.89× (4.69–5.09) | 1.49× (1.44–1.53) | 1.73× (1.70–1.77) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 0.031 s | 400 | 20.2 | 1.29× (1.21–1.39) | 3.18× (2.98–3.41) | 3.62× (3.45–3.80) | 4.29× (4.07–4.52) | 4.15× (3.91–4.40) | 6.68× (6.36–7.02) | 6.52× (6.13–6.94) | 1.98× (1.88–2.09) | 2.31× (2.21–2.42) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 0.028 s | 444 | 30.5 | 1.30× (1.24–1.36) | 2.33× (2.24–2.43) | 2.85× (2.75–2.96) | 3.33× (3.22–3.43) | 3.78× (3.66–3.91) | 7.50× (7.34–7.67) | 7.33× (7.05–7.61) | 2.23× (2.17–2.29) | 2.60× (2.54–2.66) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.66× (2.63–2.69) | 6.68× (6.56–6.81) | 6.94× (6.82–7.06) | 7.97× (7.85–8.09) | 7.10× (6.99–7.22) | 7.27× (7.16–7.39) | 6.94× (6.82–7.06) | 2.13× (2.08–2.18) | 2.39× (2.37–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.57× (2.48–2.66) | 6.33× (6.15–6.51) | 6.71× (6.56–6.87) | 7.87× (7.54–8.21) | 6.67× (6.47–6.88) | 6.65× (6.51–6.80) | 6.35× (6.18–6.53) | 1.95× (1.88–2.02) | 2.19× (2.15–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.70× (2.62–2.79) | 4.85× (4.69–5.02) | 5.46× (5.12–5.83) | 5.96× (5.66–6.28) | 5.22× (4.99–5.46) | 4.35× (4.19–4.51) | 4.15× (3.99–4.32) | 1.27× (1.23–1.33) | 1.43× (1.39–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.65) | 3.14× (3.04–3.24) | 3.95× (3.85–4.06) | 3.51× (3.43–3.60) | 6.22× (5.94–6.52) | 2.71× (2.60–2.83) | 2.65× (2.59–2.71) | 1.01× (0.99–1.03) | 1.88× (1.82–1.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.82× (2.73–2.91) | 4.91× (4.78–5.05) | 3.36× (3.23–3.49) | 5.78× (5.57–6.00) | 2.58× (2.47–2.69) | 2.52× (2.45–2.59) | 0.96× (0.93–0.99) | 1.79× (1.72–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.55–0.59) | 3.29× (2.84–3.81) | 5.14× (4.97–5.33) | 3.46× (3.36–3.57) | 5.91× (5.68–6.15) | 2.64× (2.50–2.80) | 2.58× (2.49–2.68) | 0.98× (0.95–1.01) | 1.83× (1.77–1.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.76× (2.66–2.86) | 3.24× (3.14–3.33) | 3.83× (3.76–3.90) | 4.48× (4.32–4.64) | 5.10× (4.99–5.22) | 2.93× (2.86–3.01) | 2.48× (2.42–2.55) | 2.08× (2.04–2.12) | 2.81× (2.75–2.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.12× (1.77–2.55) | 2.47× (2.05–2.98) | 2.97× (2.47–3.57) | 3.27× (2.74–3.91) | 3.83× (3.20–4.60) | 2.07× (1.71–2.50) | 1.75× (1.46–2.10) | 1.47× (1.22–1.76) | 1.98× (1.65–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.85× (1.72–1.98) | 1.72× (1.59–1.85) | 2.05× (1.91–2.20) | 2.03× (1.88–2.18) | 2.23× (2.08–2.40) | 1.10× (1.02–1.18) | 0.93× (0.87–1.00) | 0.78× (0.73–0.83) | 1.05× (0.98–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.41) | 3.01× (2.94–3.08) | 2.70× (2.66–2.75) | 2.72× (2.66–2.78) | 3.04× (2.97–3.11) | 3.46× (3.37–3.56) | 3.49× (3.42–3.56) | 1.36× (1.33–1.39) | 1.47× (1.43–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.22–1.31) | 2.76× (2.72–2.81) | 2.38× (2.29–2.48) | 3.17× (3.04–3.31) | 2.67× (2.59–2.76) | 2.99× (2.90–3.08) | 3.01× (2.90–3.13) | 1.17× (1.13–1.22) | 1.27× (1.21–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.39–1.49) | 2.59× (2.52–2.67) | 2.15× (2.08–2.23) | 2.79× (2.74–2.85) | 2.62× (2.57–2.68) | 2.26× (2.21–2.31) | 2.28× (2.23–2.33) | 0.89× (0.87–0.91) | 0.96× (0.94–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.45–1.58) | 3.63× (3.58–3.68) | 3.00× (2.96–3.04) | 3.16× (3.01–3.33) | 3.48× (3.41–3.55) | 4.41× (4.32–4.49) | 4.73× (4.63–4.84) | 1.67× (1.64–1.70) | 1.84× (1.74–1.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.28× (1.19–1.38) | 3.11× (2.88–3.36) | 2.53× (2.37–2.70) | 3.50× (3.24–3.78) | 2.92× (2.64–3.23) | 3.62× (3.35–3.91) | 3.89× (3.60–4.19) | 1.37× (1.26–1.48) | 1.51× (1.38–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.50× (1.44–1.57) | 2.97× (2.86–3.09) | 2.20× (2.15–2.25) | 3.10× (2.95–3.26) | 2.80× (2.73–2.87) | 2.70× (2.61–2.78) | 2.90× (2.80–3.00) | 1.02× (0.98–1.06) | 1.13× (1.06–1.20) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.66–0.71) | 2.57× (2.49–2.65) | 3.41× (3.30–3.52) | 2.92× (2.82–3.03) | 6.29× (6.08–6.51) | 2.17× (2.10–2.24) | 2.27× (2.13–2.43) | 0.99× (0.96–1.01) | 1.52× (1.46–1.58) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.61) | 2.34× (2.27–2.41) | 3.99× (3.86–4.13) | 2.74× (2.66–2.81) | 5.84× (5.67–6.02) | 2.00× (1.95–2.06) | 2.10× (1.96–2.25) | 0.91× (0.88–0.94) | 1.40× (1.34–1.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.48–0.62) | 2.38× (2.12–2.68) | 3.81× (3.44–4.22) | 2.69× (2.40–3.02) | 5.83× (5.08–6.68) | 1.95× (1.72–2.22) | 2.05× (1.78–2.35) | 0.89× (0.79–1.00) | 1.36× (1.20–1.54) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 1.69× (1.64–1.75) | 3.95× (3.85–4.05) | 4.26× (4.19–4.34) | 4.52× (4.43–4.61) | 4.79× (4.71–4.87) | 4.85× (4.78–4.93) | 4.74× (4.55–4.94) | 1.47× (1.42–1.51) | 1.61× (1.58–1.63) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 1.40× (1.36–1.44) | 3.25× (3.17–3.33) | 3.56× (3.47–3.66) | 4.08× (4.00–4.17) | 3.82× (3.69–3.97) | 3.79× (3.69–3.89) | 3.70× (3.54–3.86) | 1.15× (1.11–1.18) | 1.25× (1.22–1.29) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 1.45× (1.38–1.52) | 2.58× (2.51–2.66) | 2.87× (2.76–2.98) | 3.20× (3.08–3.32) | 3.20× (3.11–3.29) | 2.51× (2.43–2.59) | 2.45× (2.34–2.56) | 0.76× (0.73–0.79) | 0.83× (0.80–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.79× (0.77–0.81) | 11.07× (11.05–11.08) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.33) | 0.45× (0.40–0.52) | 0.66× (0.63–0.69) | 0.56× (0.54–0.58) | 5.29× (5.09–5.50) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.32–0.34) | 0.51× (0.40–0.64) | 0.59× (0.57–0.61) | 0.52× (0.50–0.54) | 3.43× (3.32–3.55) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.47) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.07) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.63–0.72) | 0.65× (0.58–0.72) | 1.58× (1.51–1.66) | 0.79× (0.76–0.83) | 5.75× (5.48–6.02) | 0.46× (0.44–0.48) | 0.49× (0.46–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.59–0.66) | 0.71× (0.65–0.76) | 1.12× (1.07–1.17) | 0.69× (0.67–0.72) | 3.28× (3.13–3.44) | 0.25× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.38–0.43) | 0.90× (0.88–0.91) | 0.73× (0.71–0.74) | 3.11× (3.06–3.16) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.62–0.67) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.83× (0.81–0.85) | 2.55× (2.49–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.50× (0.48–0.51) | 0.39× (0.36–0.41) | 0.88× (0.86–0.90) | 0.76× (0.73–0.78) | 3.03× (2.95–3.11) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.49× (0.46–0.52) | 0.81× (0.80–0.83) | 0.84× (0.81–0.86) | 2.35× (2.32–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.34× (5.32–5.35) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.41× (6.39–6.42) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.38× (0.36–0.41) | 1.03× (1.00–1.05) | 0.67× (0.66–0.69) | 3.27× (3.20–3.33) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 58 MB | 0.48× (0.47–0.50) | 0.43× (0.39–0.48) | 0.88× (0.86–0.91) | 0.72× (0.70–0.74) | 2.25× (2.20–2.31) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 674 | 1.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,158 | 1.63 | 1.09× (1.07–1.12) | 84% | 1.04× (1.02–1.05) | 1.05× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,370 | 2.48 | 1.67× (1.62–1.73) | 51% | 1.21× (1.18–1.25) | 1.70× (1.61–1.79) |

**Best:** mibz2 at 4 threads (1,370 MB/s); lbzip2 at 4 threads (321 MB/s); pbz2 at 4 threads (508 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.91, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.54. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.37 | 0.01 | 0.07 | 0.153 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4526 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.07 | 14.8% |
| MTF + RLE2 | 1.51 | 20.8% |
| T-vector build | 1.36 | 18.8% |
| walk (the one each build uses) | 2.88 | 39.7% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.27 | 3.7% |
| **total** | 7.26 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.296 | 0.704 | (in its total) | 2.41× (2.37–2.44) |  |
| walk | 0.614 | 1.574 | (in its total) | 2.58× (2.55–2.62) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.029 | (in its total) | 1.15× (1.13–1.16) |  |
| inverse BWT, total | 0.936 | 2.307 | 1.285 | 2.49× (2.45–2.53) | 1.38× (1.35–1.42) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m23s ± 5m16s |
| 4 threads | 22m26s ± 3m14s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m52s ± 2m03s |
| the scanner alone (its serial part) | 8m09s ± 32.2s |
| the workers alone, over 4 | 13m51s ± 2m02s |

At 4 threads the workers bound it. Per byte: the scanner 4.7 ns a byte of bz2, the workers 1.82 ns a byte of output at 4 threads (CPU, summed over them), 1.22 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,408,661 | 12,232,367 | \`2d4782182c8b0988…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4525.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
