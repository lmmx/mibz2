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

The latest record: [`5ed20c1`](<https://github.com/lmmx/mibz2/commit/5ed20c1d51afc56c088ae244ba0bf40f37cb93b1>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45949114795>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.394 s | 533 | 7.0 | 3.29× (3.22–3.36) | 7.44× (7.33–7.56) | 7.77× (7.63–7.91) | 8.91× (8.71–9.11) | 7.80× (7.50–8.11) | 8.01× (7.88–8.15) | 7.74× (7.61–7.88) | 2.46× (2.38–2.54) | 2.63× (2.59–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.231 s | 910 | 7.6 | 3.32× (3.07–3.59) | 6.76× (6.34–7.22) | 7.65× (7.13–8.20) | 7.83× (7.38–8.32) | 7.11× (6.69–7.56) | 13.35× (12.65–14.09) | 12.90× (12.29–13.54) | 4.10× (3.92–4.30) | 4.38× (4.14–4.63) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.194 s | 1,079 | 12.3 | 3.34× (3.18–3.51) | 5.30× (4.78–5.88) | 6.56× (6.01–7.16) | 6.09× (5.74–6.47) | 5.95× (5.61–6.31) | 16.21× (15.75–16.70) | 15.67× (15.19–16.17) | 4.98× (4.77–5.21) | 5.32× (5.16–5.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,090 | 3.7 | 0.57× (0.56–0.58) | 3.36× (3.30–3.41) | 4.47× (4.40–4.54) | 3.52× (3.47–3.57) | 6.00× (5.92–6.08) | 2.61× (2.56–2.65) | 2.61× (2.57–2.65) | 0.97× (0.95–0.99) | 2.11× (1.92–2.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,053 | 4.0 | 0.50× (0.49–0.52) | 2.73× (2.60–2.87) | 5.35× (5.23–5.47) | 3.37× (3.29–3.46) | 5.70× (5.54–5.85) | 2.48× (2.43–2.54) | 2.49× (2.43–2.54) | 0.93× (0.90–0.95) | 2.01× (1.85–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,028 | 4.0 | 0.51× (0.47–0.54) | 2.62× (2.56–2.69) | 5.23× (5.11–5.35) | 3.29× (3.23–3.36) | 5.62× (5.44–5.80) | 2.42× (2.37–2.48) | 2.43× (2.37–2.48) | 0.90× (0.87–0.94) | 1.96× (1.79–2.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 721 | 5.5 | 2.64× (2.55–2.73) | 2.97× (2.85–3.08) | 3.69× (3.55–3.84) | 3.98× (3.84–4.13) | 4.38× (4.26–4.51) | 2.70× (2.59–2.82) | 2.19× (2.11–2.27) | 2.10× (2.00–2.20) | 2.95× (2.84–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,104 | 6.8 | 2.36× (2.13–2.63) | 2.89× (2.68–3.10) | 4.49× (4.16–4.85) | 3.52× (3.31–3.75) | 5.12× (4.80–5.47) | 4.10× (3.80–4.43) | 3.33× (3.10–3.56) | 3.19× (2.97–3.43) | 4.48× (4.18–4.80) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 995 | 13.3 | 2.02× (1.93–2.12) | 2.16× (1.89–2.45) | 3.11× (2.91–3.34) | 2.24× (2.12–2.37) | 4.19× (3.98–4.42) | 3.74× (3.59–3.89) | 3.03× (2.89–3.18) | 2.90× (2.79–3.02) | 4.08× (3.88–4.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 52.4 | 1.40× (1.39–1.42) | 2.69× (2.66–2.72) | 2.44× (2.39–2.48) | 2.56× (2.54–2.58) | 2.87× (2.84–2.90) | 3.05× (3.02–3.09) | 3.18× (3.15–3.21) | 1.28× (1.27–1.29) | 1.44× (1.42–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 120 | 57.9 | 1.34× (1.30–1.39) | 2.66× (2.53–2.80) | 2.54× (2.38–2.71) | 3.24× (3.13–3.36) | 3.06× (2.93–3.19) | 5.07× (4.90–5.25) | 5.28× (5.11–5.45) | 2.13× (2.06–2.20) | 2.40× (2.32–2.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 152 | 74.7 | 1.44× (1.39–1.48) | 2.67× (2.26–3.16) | 2.59× (2.27–2.95) | 3.19× (2.93–3.47) | 3.40× (3.13–3.69) | 6.43× (6.24–6.62) | 6.69× (6.50–6.89) | 2.70× (2.63–2.77) | 3.04× (2.93–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.122 s | 79 | 47.5 | 1.51× (1.46–1.56) | 3.03× (2.92–3.15) | 2.73× (2.54–2.93) | 2.78× (2.67–2.90) | 3.14× (3.01–3.28) | 3.66× (3.52–3.81) | 4.03× (3.89–4.18) | 1.58× (1.51–1.65) | 1.82× (1.75–1.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 54.7 | 1.34× (1.25–1.44) | 2.79× (2.65–2.93) | 2.65× (2.47–2.84) | 3.57× (3.31–3.85) | 3.14× (2.92–3.37) | 5.86× (5.45–6.31) | 6.46× (6.02–6.93) | 2.52× (2.37–2.69) | 2.91× (2.71–3.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.061 s | 158 | 75.4 | 1.52× (1.45–1.60) | 2.79× (2.47–3.16) | 2.88× (2.51–3.30) | 3.48× (3.26–3.71) | 3.69× (3.19–4.28) | 7.41× (7.26–7.55) | 8.16× (7.96–8.36) | 3.19× (3.14–3.24) | 3.68× (3.60–3.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 820 | 5.1 | 0.61× (0.61–0.62) | 3.38× (3.36–3.41) | 4.23× (4.20–4.27) | 3.05× (3.02–3.09) | 6.65× (6.54–6.75) | 2.31× (2.21–2.41) | 2.31× (2.27–2.35) | 0.94× (0.93–0.96) | 1.70× (1.69–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 758 | 5.8 | 0.49× (0.48–0.50) | 2.35× (2.29–2.40) | 4.79× (4.58–5.01) | 2.83× (2.77–2.90) | 6.13× (6.03–6.23) | 2.12× (2.05–2.20) | 2.12× (2.08–2.17) | 0.87× (0.84–0.89) | 1.56× (1.53–1.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 738 | 5.8 | 0.47× (0.44–0.50) | 2.23× (2.10–2.37) | 4.44× (4.19–4.71) | 2.69× (2.54–2.86) | 6.00× (5.56–6.49) | 2.01× (1.87–2.16) | 2.01× (1.89–2.14) | 0.82× (0.78–0.86) | 1.48× (1.39–1.57) | yes |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 1 | 0.066 s | 264 | 14.4 | 1.99× (1.93–2.05) | 4.01× (3.95–4.08) | 4.52× (4.42–4.63) | 4.79× (4.70–4.89) | 4.84× (4.74–4.94) | 4.74× (4.63–4.85) | 4.87× (4.78–4.96) | 1.70× (1.61–1.80) | 1.88× (1.85–1.92) | yes |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 2 | 0.048 s | 364 | 18.6 | 1.41× (1.26–1.58) | 3.15× (2.68–3.71) | 3.70× (3.18–4.29) | 4.20× (3.72–4.74) | 3.95× (3.45–4.51) | 6.20× (5.55–6.91) | 6.37× (5.70–7.12) | 2.22× (1.94–2.54) | 2.46× (2.22–2.73) | yes |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 4 | 0.041 s | 424 | 28.7 | 1.56× (1.50–1.62) | 2.79× (2.50–3.12) | 3.63× (3.29–4.00) | 3.72× (3.54–3.92) | 3.93× (3.69–4.19) | 7.68× (7.39–7.98) | 7.90× (7.61–8.19) | 2.75× (2.57–2.94) | 3.05× (2.93–3.17) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.28× (3.21–3.35) | 7.40× (7.29–7.51) | 7.72× (7.58–7.86) | 8.87× (8.67–9.08) | 7.77× (7.47–8.07) | 7.98× (7.84–8.12) | 7.71× (7.58–7.85) | 2.45× (2.37–2.54) | 2.61× (2.58–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.20× (3.08–3.32) | 7.23× (6.94–7.53) | 7.48× (7.19–7.77) | 8.48× (8.35–8.61) | 7.37× (7.09–7.67) | 7.33× (7.22–7.43) | 7.08× (6.97–7.19) | 2.25× (2.19–2.32) | 2.40× (2.37–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.22× (3.08–3.36) | 5.74× (5.28–6.23) | 6.36× (5.87–6.89) | 6.63× (6.32–6.95) | 5.78× (5.46–6.11) | 4.53× (4.32–4.76) | 4.38× (4.16–4.62) | 1.39× (1.32–1.47) | 1.48× (1.41–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.88× (2.83–2.94) | 3.85× (3.79–3.91) | 3.29× (3.24–3.35) | 5.55× (5.47–5.63) | 2.46× (2.42–2.50) | 2.47× (2.42–2.51) | 0.97× (0.95–0.99) | 1.62× (1.58–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.56) | 2.57× (2.49–2.66) | 4.66× (4.58–4.73) | 3.08× (3.02–3.14) | 5.12× (5.00–5.25) | 2.27× (2.23–2.32) | 2.28× (2.23–2.33) | 0.90× (0.88–0.92) | 1.49× (1.47–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.50–0.57) | 2.81× (2.76–2.86) | 4.63× (4.54–4.73) | 3.06× (3.00–3.12) | 5.12× (4.97–5.28) | 2.26× (2.21–2.31) | 2.26× (2.22–2.31) | 0.89× (0.86–0.92) | 1.48× (1.45–1.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.57× (2.53–2.61) | 2.79× (2.74–2.85) | 3.37× (3.32–3.42) | 3.84× (3.78–3.90) | 4.21× (4.12–4.29) | 2.62× (2.55–2.70) | 2.14× (2.10–2.18) | 2.06× (2.00–2.12) | 2.63× (2.59–2.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.22–2.53) | 2.47× (2.39–2.56) | 3.37× (3.26–3.48) | 3.30× (3.18–3.42) | 3.60× (3.50–3.69) | 2.14× (2.05–2.23) | 1.74× (1.68–1.81) | 1.68× (1.62–1.73) | 2.14× (2.08–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.00–2.21) | 1.77× (1.61–1.96) | 2.26× (2.15–2.37) | 2.06× (1.95–2.17) | 2.11× (2.03–2.20) | 1.09× (1.03–1.16) | 0.89× (0.85–0.93) | 0.85× (0.81–0.90) | 1.09× (1.05–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.39–1.41) | 2.62× (2.60–2.64) | 2.38× (2.34–2.43) | 2.54× (2.51–2.56) | 2.84× (2.81–2.87) | 3.02× (2.99–3.05) | 3.15× (3.11–3.18) | 1.28× (1.27–1.29) | 1.39× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.31–1.34) | 2.53× (2.49–2.58) | 2.30× (2.20–2.40) | 3.05× (3.01–3.09) | 2.68× (2.62–2.73) | 2.73× (2.69–2.77) | 2.84× (2.79–2.89) | 1.15× (1.14–1.17) | 1.25× (1.23–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.53–1.60) | 2.91× (2.47–3.43) | 2.36× (2.06–2.70) | 3.13× (2.84–3.46) | 3.04× (2.72–3.39) | 2.13× (2.09–2.17) | 2.22× (2.18–2.26) | 0.90× (0.88–0.92) | 0.98× (0.96–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.45–1.56) | 2.97× (2.85–3.08) | 2.67× (2.49–2.87) | 2.76× (2.65–2.87) | 3.11× (2.98–3.25) | 3.62× (3.48–3.77) | 3.99× (3.85–4.13) | 1.57× (1.50–1.64) | 1.77× (1.71–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.25–1.45) | 2.71× (2.51–2.91) | 2.38× (2.21–2.56) | 3.26× (3.04–3.50) | 2.79× (2.60–3.01) | 3.08× (2.85–3.33) | 3.39× (3.14–3.65) | 1.33× (1.25–1.42) | 1.50× (1.40–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.64× (1.56–1.73) | 3.01× (2.61–3.49) | 2.54× (2.08–3.10) | 3.23× (2.94–3.55) | 3.36× (2.80–4.03) | 2.31× (2.28–2.33) | 2.54× (2.49–2.59) | 1.00× (0.98–1.02) | 1.13× (1.11–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.49× (2.44–2.54) | 3.44× (3.38–3.50) | 2.80× (2.77–2.83) | 5.94× (5.83–6.05) | 2.15× (2.06–2.24) | 2.15× (2.11–2.18) | 0.95× (0.94–0.96) | 1.36× (1.34–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.53–0.55) | 2.11× (2.08–2.15) | 3.88× (3.78–3.97) | 2.54× (2.48–2.59) | 5.25× (5.17–5.34) | 1.88× (1.82–1.95) | 1.88× (1.85–1.92) | 0.83× (0.81–0.86) | 1.19× (1.15–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.50× (0.47–0.54) | 2.26× (2.11–2.41) | 3.69× (3.42–3.98) | 2.48× (2.31–2.65) | 5.28× (4.85–5.76) | 1.83× (1.70–1.98) | 1.83× (1.71–1.97) | 0.81× (0.76–0.87) | 1.16× (1.08–1.25) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 1 | 1.96× (1.91–2.02) | 3.87× (3.80–3.94) | 4.34× (4.24–4.44) | 4.69× (4.61–4.78) | 4.74× (4.65–4.84) | 4.64× (4.53–4.75) | 4.77× (4.68–4.86) | 1.68× (1.59–1.77) | 1.78× (1.75–1.80) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 2 | 1.63× (1.59–1.66) | 3.40× (3.18–3.64) | 3.78× (3.54–4.04) | 4.33× (4.20–4.46) | 3.85× (3.67–4.04) | 3.62× (3.50–3.76) | 3.73× (3.61–3.85) | 1.31× (1.22–1.41) | 1.39× (1.34–1.43) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 4 | 1.74× (1.70–1.78) | 2.97× (2.68–3.29) | 3.43× (3.10–3.80) | 3.54× (3.41–3.69) | 3.27× (3.09–3.47) | 2.32× (2.30–2.34) | 2.39× (2.35–2.42) | 0.84× (0.80–0.88) | 0.89× (0.87–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.79) | 10.97× (10.96–10.98) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.45× (0.38–0.53) | 0.72× (0.69–0.74) | 0.60× (0.58–0.63) | 5.71× (5.51–5.91) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.37× (0.36–0.38) | 0.58× (0.49–0.67) | 0.66× (0.64–0.69) | 0.57× (0.55–0.60) | 3.80× (3.68–3.92) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.83× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.67–1.69) | 0.66× (0.65–0.66) | 7.95× (7.90–8.00) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.67× (0.63–0.72) | 1.70× (1.65–1.75) | 0.84× (0.82–0.87) | 6.13× (5.97–6.30) | 0.50× (0.49–0.51) | 0.52× (0.51–0.54) | 0.57× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.69× (0.67–0.71) | 0.75× (0.70–0.80) | 1.22× (1.17–1.27) | 0.75× (0.72–0.79) | 3.60× (3.45–3.75) | 0.28× (0.27–0.29) | 0.29× (0.28–0.31) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.38× (0.35–0.42) | 0.89× (0.87–0.91) | 0.72× (0.70–0.74) | 3.09× (3.02–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.44–0.47) | 0.87× (0.85–0.90) | 0.82× (0.80–0.85) | 2.53× (2.46–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.44–0.48) | 0.33× (0.30–0.36) | 0.83× (0.80–0.86) | 0.68× (0.65–0.71) | 2.82× (2.72–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.47× (0.44–0.49) | 0.81× (0.80–0.82) | 0.82× (0.79–0.85) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.05–1.07) | 0.76× (0.75–0.76) | 5.31× (5.28–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.17–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.88) | 0.89× (0.89–0.90) | 6.19× (6.16–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 1 | 20 MB | 0.45× (0.44–0.45) | 0.38× (0.38–0.38) | 1.37× (1.37–1.37) | 0.67× (0.66–0.67) | 6.43× (6.41–6.44) | 0.38× (0.38–0.39) | 0.40× (0.40–0.41) | 0.62× (0.62–0.62) | 0.30× (0.30–0.30) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 2 | 36 MB | 0.46× (0.44–0.49) | 0.43× (0.39–0.47) | 1.05× (0.99–1.11) | 0.69× (0.65–0.74) | 3.51× (3.31–3.73) | 0.21× (0.20–0.22) | 0.22× (0.21–0.23) | 0.34× (0.32–0.36) | 0.16× (0.16–0.17) |
| `text` (2.69 MB → 17.4 MB, 6.4×) | 4 | 60 MB | 0.47× (0.46–0.48) | 0.44× (0.41–0.48) | 0.85× (0.83–0.88) | 0.71× (0.69–0.72) | 2.22× (2.17–2.28) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 533 | 1.89 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 910 | 2.06 | 1.09× (1.08–1.10) | 83% | 1.06× (1.03–1.10) | 1.06× (1.03–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 1,079 | 3.34 | 1.76× (1.67–1.85) | 51% | 1.37× (1.25–1.49) | 1.73× (1.70–1.75) |

**Best:** mibz2 at 4 threads (1,079 MB/s); lbzip2 at 4 threads (213 MB/s); pbz2 at 4 threads (327 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.01, lbzip2 1t 0.07, lbzip2 2t 0.20, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.55 | 0.02 | 0.07 | 0.194 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3696 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.84 | 9.7% |
| MTF + RLE2 | 1.48 | 17.1% |
| T-vector build | 1.52 | 17.6% |
| walk (the one each build uses) | 4.35 | 50.3% |
| of which: segment assembly | 0.17 | 2.0% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.064 | (in its total) | 2.71× (2.70–2.72) |  |
| walk | 1.095 | 2.063 | (in its total) | 1.89× (1.87–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.046 | (in its total) | 1.07× (1.03–1.12) |  |
| inverse BWT, total | 1.531 | 3.174 | 1.889 | 2.07× (2.06–2.09) | 1.24× (1.23–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m25s ± 6m54s |
| 4 threads | 28m23s ± 4m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m58s ± 3m00s |
| the scanner alone (its serial part) | 8m18s ± 42.1s |
| the workers alone, over 4 | 19m58s ± 3m00s |

At 4 threads the workers bound it. Per byte: the scanner 4.8 ns a byte of bz2, the workers 2.62 ns a byte of output at 4 threads (CPU, summed over them), 1.61 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,691,091 | 17,357,170 | \`a5bb505b8121f35b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
