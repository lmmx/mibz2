# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-857/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`724b76c`](<https://github.com/lmmx/mibz2/commit/724b76c5f8575d19df8f71b8934ea7bd7b9dbb07>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45531510378>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.493 s | 426 | 7.7 | 3.07× (3.06–3.08) | 7.08× (7.03–7.13) | 7.24× (7.21–7.27) | 7.88× (7.85–7.91) | 7.12× (7.09–7.15) | 7.74× (7.73–7.76) | 7.12× (7.10–7.14) | 2.26× (2.26–2.27) | 2.66× (2.64–2.67) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.281 s | 747 | 8.2 | 3.10× (2.88–3.34) | 6.53× (6.31–6.75) | 7.39× (7.11–7.67) | 7.39× (7.18–7.61) | 6.67× (6.46–6.90) | 13.53× (13.11–13.96) | 12.43× (12.05–12.83) | 3.95× (3.83–4.08) | 4.64× (4.50–4.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 938 | 12.8 | 3.14× (3.10–3.18) | 4.87× (4.81–4.94) | 6.06× (5.83–6.30) | 5.50× (5.45–5.55) | 5.54× (5.50–5.58) | 17.08× (16.94–17.22) | 15.69× (15.56–15.83) | 4.99× (4.95–5.03) | 5.86× (5.80–5.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 968 | 3.7 | 0.59× (0.58–0.60) | 3.72× (3.53–3.92) | 4.34× (4.27–4.41) | 3.52× (3.47–3.57) | 6.22× (6.13–6.31) | 2.93× (2.88–2.97) | 2.62× (2.58–2.66) | 1.01× (1.00–1.03) | 1.48× (1.46–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 928 | 4.0 | 0.55× (0.54–0.56) | 2.87× (2.83–2.90) | 5.83× (5.65–6.02) | 3.43× (3.39–3.47) | 5.90× (5.83–5.97) | 2.83× (2.80–2.87) | 2.54× (2.50–2.57) | 0.98× (0.96–1.00) | 1.43× (1.41–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 893 | 4.0 | 0.53× (0.52–0.54) | 2.76× (2.71–2.80) | 5.64× (5.50–5.79) | 3.28× (3.23–3.33) | 5.74× (5.67–5.82) | 2.71× (2.67–2.76) | 2.43× (2.38–2.47) | 0.94× (0.91–0.97) | 1.37× (1.35–1.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 583 | 5.9 | 2.62× (2.59–2.65) | 3.03× (3.00–3.05) | 3.92× (3.84–3.99) | 3.81× (3.78–3.84) | 4.24× (4.21–4.28) | 2.56× (2.53–2.59) | 2.11× (2.09–2.13) | 2.02× (2.00–2.03) | 3.01× (2.99–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 903 | 7.0 | 2.37× (2.20–2.56) | 3.06× (2.77–3.39) | 4.03× (3.80–4.28) | 3.72× (3.26–4.25) | 5.08× (4.89–5.28) | 3.99× (3.82–4.17) | 3.29× (3.16–3.43) | 3.14× (3.01–3.27) | 4.69× (4.51–4.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 862 | 13.6 | 2.07× (1.98–2.16) | 2.26× (2.16–2.37) | 3.25× (3.05–3.47) | 2.48× (2.30–2.68) | 4.44× (4.27–4.61) | 3.78× (3.64–3.92) | 3.12× (3.01–3.23) | 2.97× (2.87–3.08) | 4.45× (4.28–4.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 53.4 | 1.35× (1.35–1.36) | 2.75× (2.73–2.76) | 2.50× (2.46–2.54) | 2.50× (2.48–2.51) | 2.92× (2.90–2.93) | 3.20× (3.18–3.22) | 3.14× (3.13–3.15) | 1.24× (1.23–1.25) | 1.41× (1.40–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.077 s | 104 | 59.0 | 1.31× (1.28–1.33) | 2.67× (2.57–2.77) | 2.48× (2.43–2.53) | 3.20× (3.13–3.28) | 3.08× (3.02–3.14) | 5.32× (5.21–5.43) | 5.23× (5.13–5.34) | 2.06× (2.03–2.10) | 2.35× (2.30–2.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.066 s | 122 | 81.8 | 1.36× (1.33–1.39) | 2.23× (2.19–2.27) | 2.20× (2.13–2.27) | 2.74× (2.64–2.84) | 2.99× (2.94–3.03) | 6.35× (6.28–6.42) | 6.24× (6.16–6.32) | 2.46× (2.44–2.48) | 2.81× (2.76–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 67 | 48.8 | 1.46× (1.45–1.46) | 3.09× (3.05–3.14) | 2.62× (2.59–2.65) | 2.67× (2.66–2.69) | 3.17× (3.08–3.26) | 3.69× (3.67–3.71) | 3.85× (3.83–3.88) | 1.43× (1.41–1.45) | 1.71× (1.69–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.084 s | 114 | 54.9 | 1.38× (1.34–1.41) | 2.89× (2.81–2.97) | 2.59× (2.51–2.68) | 3.59× (3.52–3.66) | 3.24× (3.18–3.30) | 6.24× (6.13–6.34) | 6.52× (6.40–6.63) | 2.41× (2.37–2.46) | 2.89× (2.84–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.072 s | 133 | 79.8 | 1.46× (1.44–1.48) | 2.41× (2.35–2.49) | 2.41× (2.32–2.51) | 3.16× (3.11–3.21) | 3.01× (2.97–3.06) | 7.29× (7.18–7.40) | 7.61× (7.50–7.73) | 2.82× (2.75–2.89) | 3.38× (3.31–3.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 5.1 | 0.66× (0.65–0.67) | 2.98× (2.92–3.04) | 4.36× (4.28–4.44) | 3.08× (3.03–3.13) | 6.83× (6.69–6.96) | 2.50× (2.45–2.55) | 2.29× (2.25–2.33) | 0.99× (0.97–1.01) | 1.50× (1.48–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 667 | 5.8 | 0.53× (0.52–0.54) | 2.72× (2.67–2.77) | 4.67× (4.59–4.75) | 2.84× (2.79–2.89) | 6.21× (6.09–6.33) | 2.28× (2.25–2.32) | 2.09× (2.05–2.13) | 0.90× (0.88–0.92) | 1.37× (1.35–1.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 660 | 5.8 | 0.54× (0.53–0.55) | 2.72× (2.68–2.76) | 4.66× (4.60–4.73) | 2.84× (2.79–2.88) | 6.25× (6.16–6.33) | 2.28× (2.24–2.31) | 2.08× (2.05–2.12) | 0.90× (0.89–0.91) | 1.37× (1.35–1.39) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.071 s | 196 | 17.0 | 2.05× (2.03–2.07) | 4.27× (4.21–4.34) | 4.66× (4.59–4.73) | 4.70× (4.63–4.77) | 4.93× (4.86–5.00) | 5.51× (5.43–5.60) | 5.02× (4.94–5.11) | 1.58× (1.57–1.59) | 1.99× (1.96–2.01) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.048 s | 291 | 21.2 | 1.63× (1.52–1.74) | 3.58× (3.39–3.77) | 4.30× (4.00–4.62) | 4.59× (4.39–4.80) | 4.34× (4.08–4.61) | 8.01× (7.55–8.50) | 7.30× (6.91–7.71) | 2.30× (2.16–2.44) | 2.88× (2.72–3.06) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.043 s | 328 | 32.6 | 1.59× (1.54–1.65) | 2.71× (2.60–2.82) | 3.66× (3.58–3.74) | 3.70× (3.63–3.78) | 4.03× (3.93–4.13) | 9.28× (9.09–9.49) | 8.46× (8.27–8.66) | 2.66× (2.59–2.73) | 3.34× (3.27–3.41) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.06× (3.05–3.07) | 7.04× (6.99–7.09) | 7.21× (7.18–7.24) | 7.85× (7.83–7.88) | 7.09× (7.06–7.12) | 7.72× (7.70–7.73) | 7.09× (7.07–7.11) | 2.26× (2.25–2.26) | 2.64× (2.62–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.99× (2.97–3.01) | 6.76× (6.70–6.82) | 7.07× (7.03–7.11) | 7.77× (7.73–7.80) | 6.72× (6.69–6.76) | 7.24× (7.21–7.27) | 6.65× (6.62–6.69) | 2.12× (2.11–2.13) | 2.48× (2.45–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.00× (2.95–3.05) | 5.15× (5.09–5.21) | 5.48× (5.40–5.56) | 5.83× (5.76–5.91) | 5.17× (5.06–5.27) | 4.66× (4.60–4.71) | 4.28× (4.23–4.33) | 1.36× (1.35–1.38) | 1.59× (1.57–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.24× (3.18–3.30) | 4.01× (3.95–4.07) | 3.31× (3.26–3.35) | 5.78× (5.69–5.87) | 2.77× (2.73–2.81) | 2.49× (2.45–2.53) | 1.02× (1.00–1.03) | 1.05× (1.01–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.59) | 2.89× (2.85–2.93) | 5.09× (5.03–5.16) | 3.13× (3.09–3.17) | 5.29× (5.23–5.36) | 2.58× (2.55–2.61) | 2.32× (2.28–2.35) | 0.95× (0.93–0.97) | 0.98× (0.94–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.11× (3.06–3.17) | 4.96× (4.89–5.03) | 3.05× (3.01–3.10) | 5.22× (5.16–5.29) | 2.52× (2.48–2.56) | 2.26× (2.22–2.30) | 0.92× (0.90–0.95) | 0.95× (0.92–0.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.50–2.55) | 2.78× (2.76–2.80) | 3.52× (3.42–3.63) | 3.66× (3.63–3.68) | 4.06× (4.02–4.10) | 2.47× (2.44–2.50) | 2.05× (2.03–2.07) | 1.96× (1.94–1.97) | 2.73× (2.70–2.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.28–2.39) | 2.64× (2.56–2.72) | 3.16× (3.09–3.23) | 3.27× (3.20–3.34) | 3.56× (3.48–3.64) | 2.08× (2.04–2.12) | 1.72× (1.69–1.76) | 1.65× (1.61–1.69) | 2.30× (2.24–2.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.16× (2.08–2.23) | 1.82× (1.75–1.88) | 2.31× (2.24–2.39) | 2.13× (2.05–2.22) | 2.26× (2.19–2.33) | 1.08× (1.05–1.11) | 0.90× (0.87–0.92) | 0.86× (0.83–0.88) | 1.19× (1.16–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.66× (2.65–2.67) | 2.42× (2.39–2.45) | 2.48× (2.46–2.49) | 2.89× (2.87–2.90) | 3.16× (3.14–3.18) | 3.11× (3.10–3.12) | 1.23× (1.23–1.24) | 1.38× (1.37–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.54× (2.51–2.57) | 2.24× (2.21–2.27) | 3.00× (2.96–3.05) | 2.65× (2.62–2.68) | 2.86× (2.82–2.89) | 2.81× (2.78–2.84) | 1.12× (1.10–1.13) | 1.24× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.40–1.48) | 2.30× (2.26–2.34) | 1.87× (1.86–1.89) | 2.56× (2.52–2.60) | 2.46× (2.42–2.49) | 2.06× (2.03–2.08) | 2.02× (2.00–2.04) | 0.80× (0.80–0.81) | 0.90× (0.89–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.46) | 3.04× (3.00–3.08) | 2.55× (2.51–2.58) | 2.65× (2.64–2.66) | 3.13× (3.05–3.23) | 3.65× (3.63–3.67) | 3.81× (3.79–3.83) | 1.42× (1.40–1.44) | 1.67× (1.66–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.37) | 2.81× (2.76–2.86) | 2.35× (2.31–2.39) | 3.27× (3.21–3.33) | 2.81× (2.76–2.85) | 3.23× (3.18–3.28) | 3.37× (3.32–3.43) | 1.26× (1.23–1.28) | 1.48× (1.46–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.56× (1.54–1.58) | 2.50× (2.47–2.54) | 1.93× (1.89–1.96) | 2.78× (2.73–2.83) | 2.54× (2.51–2.57) | 2.23× (2.21–2.26) | 2.33× (2.31–2.36) | 0.87× (0.85–0.89) | 1.02× (1.01–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.72) | 2.68× (2.63–2.74) | 3.55× (3.48–3.61) | 2.83× (2.78–2.88) | 6.12× (5.98–6.25) | 2.32× (2.28–2.37) | 2.13× (2.09–2.17) | 0.99× (0.97–1.02) | 0.98× (0.94–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.32× (2.28–2.36) | 3.98× (3.93–4.04) | 2.57× (2.53–2.62) | 5.37× (5.27–5.47) | 2.03× (2.01–2.06) | 1.86× (1.83–1.89) | 0.87× (0.85–0.89) | 0.85× (0.82–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.56× (2.52–2.59) | 4.00× (3.95–4.05) | 2.61× (2.57–2.65) | 5.49× (5.42–5.56) | 2.06× (2.03–2.09) | 1.89× (1.86–1.92) | 0.88× (0.87–0.89) | 0.87× (0.84–0.90) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 2.02× (2.00–2.04) | 4.15× (4.09–4.21) | 4.49× (4.44–4.55) | 4.60× (4.53–4.67) | 4.82× (4.75–4.89) | 5.39× (5.31–5.47) | 4.91× (4.84–4.99) | 1.57× (1.55–1.58) | 1.86× (1.85–1.87) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.72× (1.70–1.74) | 3.51× (3.46–3.57) | 3.82× (3.78–3.86) | 4.25× (4.21–4.29) | 3.92× (3.87–3.97) | 4.33× (4.28–4.38) | 3.95× (3.90–4.00) | 1.26× (1.24–1.28) | 1.50× (1.48–1.52) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.75× (1.70–1.81) | 2.78× (2.70–2.86) | 3.01× (2.96–3.07) | 3.33× (3.27–3.40) | 3.13× (3.08–3.19) | 2.82× (2.77–2.88) | 2.57× (2.52–2.63) | 0.82× (0.80–0.84) | 0.98× (0.96–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.29–0.32) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.99× (10.97–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.34–0.35) | 0.50× (0.43–0.58) | 0.71× (0.70–0.72) | 0.60× (0.58–0.61) | 5.64× (5.56–5.71) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.35–0.39) | 0.78× (0.68–0.89) | 0.65× (0.61–0.70) | 0.57× (0.54–0.60) | 3.80× (3.59–4.02) | 0.09× (0.08–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.28× (4.26–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.18–0.18) | 1.97× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.87× (4.86–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.96× (7.92–8.00) | 0.65× (0.64–0.65) | 0.67× (0.67–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.74) | 0.80× (0.72–0.89) | 1.73× (1.69–1.76) | 0.86× (0.85–0.88) | 6.25× (6.12–6.38) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.58× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.72) | 0.79× (0.73–0.85) | 1.25× (1.19–1.31) | 0.77× (0.73–0.81) | 3.68× (3.53–3.84) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.72× (0.72–0.73) | 5.15× (5.13–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.39× (0.36–0.42) | 0.88× (0.85–0.92) | 0.71× (0.69–0.73) | 3.05× (2.95–3.15) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.90) | 0.82× (0.80–0.84) | 2.57× (2.54–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.82) | 5.18× (5.17–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.37× (0.34–0.40) | 0.84× (0.82–0.86) | 0.71× (0.69–0.74) | 2.88× (2.82–2.95) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.58–0.63) | 0.51× (0.47–0.54) | 0.80× (0.78–0.82) | 0.80× (0.77–0.84) | 2.31× (2.26–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.30–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.88–1.88) | 0.90× (0.90–0.91) | 6.22× (6.20–6.24) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.67–0.67) | 6.39× (6.37–6.41) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.62× (0.61–0.62) | 0.31× (0.31–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 34 MB | 0.48× (0.46–0.50) | 0.46× (0.42–0.49) | 1.11× (1.07–1.14) | 0.73× (0.70–0.77) | 3.55× (3.41–3.70) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.34× (0.33–0.36) | 0.17× (0.17–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 55 MB | 0.50× (0.49–0.51) | 0.51× (0.48–0.55) | 0.91× (0.89–0.94) | 0.76× (0.74–0.79) | 2.34× (2.28–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 426 | 2.36 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 747 | 2.51 | 1.07× (1.06–1.07) | 87% | 1.02× (1.02–1.03) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 938 | 3.93 | 1.66× (1.64–1.68) | 55% | 1.21× (1.20–1.23) | 1.62× (1.60–1.65) |

**Best:** mibz2 at 4 threads (938 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (301 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.05, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.09 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.93 | 10.7% |
| MTF + RLE2 | 1.55 | 18.0% |
| T-vector build | 1.52 | 17.6% |
| walk (the one each build uses) | 4.12 | 47.6% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.304 | (in its total) | 2.68× (2.68–2.69) |  |
| walk | 1.259 | 2.647 | (in its total) | 2.11× (2.09–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.055 | (in its total) | 1.16× (1.15–1.17) |  |
| inverse BWT, total | 1.792 | 4.006 | 3.186 | 2.24× (2.23–2.25) | 1.78× (1.77–1.79) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m35s |
| 4 threads | 32m32s ± 4m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m30s ± 3m20s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 23m30s ± 3m20s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 3.08 ns a byte of output at 4 threads (CPU, summed over them), 2.01 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
