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

The latest record: [`2a12e39`](<https://github.com/lmmx/mibz2/commit/2a12e39f69ef7c0f17be860138701b14df82b713>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43618070360>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.527 s | 398 | 8.2 | 2.87× (2.85–2.89) | 6.62× (6.58–6.67) | 6.80× (6.77–6.84) | 7.38× (7.35–7.41) | 6.71× (6.66–6.76) | 7.23× (7.19–7.27) | 6.66× (6.59–6.73) | 2.13× (2.11–2.14) | 2.45× (2.44–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.332 s | 631 | 9.3 | 2.70× (2.53–2.89) | 5.73× (5.34–6.16) | 6.37× (6.06–6.68) | 6.24× (5.90–6.61) | 5.73× (5.42–6.05) | 11.54× (10.92–12.19) | 10.62× (10.06–11.22) | 3.39× (3.21–3.58) | 3.90× (3.70–4.11) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.266 s | 788 | 15.0 | 2.70× (2.66–2.74) | 4.12× (4.06–4.19) | 5.17× (5.06–5.28) | 4.60× (4.53–4.67) | 4.74× (4.67–4.80) | 14.45× (14.22–14.68) | 13.30× (13.10–13.51) | 4.25× (4.19–4.30) | 4.89× (4.80–4.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 966 | 3.7 | 0.60× (0.59–0.61) | 3.87× (3.75–4.01) | 4.49× (4.36–4.63) | 3.56× (3.49–3.63) | 6.19× (6.07–6.31) | 2.97× (2.91–3.03) | 2.65× (2.60–2.70) | 1.02× (0.99–1.06) | 1.50× (1.47–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 955 | 3.8 | 0.58× (0.56–0.59) | 2.95× (2.91–3.00) | 6.10× (5.94–6.27) | 3.54× (3.49–3.59) | 6.07× (5.96–6.19) | 2.92× (2.88–2.97) | 2.61× (2.57–2.64) | 1.01× (0.98–1.03) | 1.48× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 943 | 3.8 | 0.57× (0.56–0.58) | 2.93× (2.88–2.98) | 5.99× (5.79–6.20) | 3.48× (3.42–3.55) | 6.00× (5.92–6.09) | 2.89× (2.84–2.94) | 2.58× (2.54–2.62) | 1.00× (0.97–1.02) | 1.46× (1.44–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 584 | 5.9 | 2.79× (2.67–2.91) | 3.03× (3.00–3.06) | 3.98× (3.84–4.12) | 3.84× (3.80–3.88) | 4.16× (4.10–4.22) | 2.56× (2.54–2.59) | 2.11× (2.09–2.13) | 2.03× (2.01–2.05) | 3.02× (2.99–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 822 | 7.6 | 2.07× (1.93–2.23) | 2.64× (2.31–3.01) | 3.77× (3.36–4.24) | 3.25× (3.06–3.45) | 4.26× (3.71–4.88) | 3.41× (2.97–3.91) | 2.81× (2.44–3.24) | 2.70× (2.34–3.11) | 4.02× (3.48–4.63) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 776 | 14.8 | 1.84× (1.79–1.89) | 1.96× (1.84–2.09) | 2.91× (2.82–3.00) | 2.15× (2.09–2.21) | 3.92× (3.80–4.04) | 3.42× (3.33–3.53) | 2.82× (2.74–2.91) | 2.71× (2.64–2.78) | 4.03× (3.92–4.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 61 | 54.3 | 1.36× (1.34–1.38) | 2.70× (2.66–2.74) | 2.42× (2.39–2.46) | 2.48× (2.44–2.51) | 2.87× (2.82–2.92) | 3.13× (3.08–3.19) | 3.15× (3.10–3.21) | 1.22× (1.20–1.24) | 1.40× (1.36–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 60.9 | 1.29× (1.28–1.31) | 2.70× (2.41–3.02) | 2.39× (2.35–2.43) | 2.95× (2.90–3.01) | 2.94× (2.90–2.99) | 5.11× (5.04–5.19) | 5.14× (5.04–5.25) | 2.00× (1.96–2.03) | 2.28× (2.23–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 81.8 | 1.34× (1.30–1.39) | 2.11× (2.03–2.20) | 2.09× (2.01–2.16) | 2.45× (2.39–2.52) | 2.83× (2.77–2.90) | 6.03× (5.86–6.20) | 6.07× (5.93–6.20) | 2.35× (2.30–2.41) | 2.69× (2.60–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.145 s | 66 | 49.9 | 1.48× (1.47–1.50) | 3.09× (3.04–3.14) | 2.58× (2.55–2.61) | 2.65× (2.62–2.68) | 3.13× (3.04–3.21) | 3.64× (3.61–3.68) | 3.88× (3.84–3.92) | 1.40× (1.39–1.42) | 1.70× (1.67–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.3 | 1.37× (1.31–1.43) | 2.73× (2.60–2.87) | 2.41× (2.32–2.51) | 3.24× (3.10–3.38) | 3.02× (2.92–3.13) | 5.88× (5.67–6.10) | 6.27× (6.05–6.50) | 2.26× (2.19–2.34) | 2.74× (2.63–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.075 s | 128 | 79.3 | 1.47× (1.43–1.51) | 2.32× (2.26–2.37) | 2.30× (2.25–2.35) | 2.95× (2.88–3.02) | 2.87× (2.80–2.93) | 7.05× (6.93–7.17) | 7.51× (7.38–7.65) | 2.71× (2.68–2.75) | 3.28× (3.21–3.36) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 5.1 | 0.67× (0.66–0.68) | 3.08× (2.93–3.23) | 4.40× (4.31–4.48) | 3.11× (3.06–3.16) | 6.78× (6.65–6.91) | 2.53× (2.48–2.58) | 2.31× (2.28–2.35) | 0.99× (0.96–1.02) | 1.52× (1.49–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 690 | 5.5 | 0.56× (0.55–0.57) | 2.85× (2.79–2.91) | 4.89× (4.79–4.99) | 2.98× (2.93–3.03) | 6.54× (6.30–6.78) | 2.39× (2.34–2.44) | 2.19× (2.14–2.23) | 0.94× (0.91–0.96) | 1.43× (1.41–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.4 | 0.56× (0.54–0.57) | 2.81× (2.74–2.88) | 4.93× (4.69–5.19) | 2.92× (2.84–3.00) | 6.41× (6.19–6.65) | 2.35× (2.29–2.41) | 2.15× (2.09–2.21) | 0.92× (0.89–0.95) | 1.41× (1.37–1.44) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.068 s | 180 | 18.6 | 1.90× (1.85–1.96) | 4.02× (3.90–4.15) | 4.42× (4.29–4.55) | 4.42× (4.29–4.56) | 4.66× (4.51–4.80) | 5.12× (4.97–5.28) | 4.69× (4.54–4.84) | 1.47× (1.43–1.52) | 1.78× (1.73–1.84) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.048 s | 258 | 23.1 | 1.46× (1.40–1.53) | 3.27× (3.10–3.45) | 3.59× (3.42–3.77) | 4.34× (3.85–4.89) | 4.03× (3.84–4.22) | 7.22× (6.91–7.55) | 6.61× (6.31–6.93) | 2.08× (1.99–2.17) | 2.52× (2.41–2.63) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.043 s | 287 | 35.0 | 1.45× (1.40–1.50) | 2.30× (2.19–2.43) | 2.89× (2.72–3.07) | 3.25× (3.12–3.38) | 3.52× (3.42–3.63) | 8.09× (7.81–8.37) | 7.40× (7.14–7.67) | 2.33× (2.24–2.41) | 2.82× (2.72–2.92) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.85–2.89) | 6.59× (6.55–6.63) | 6.77× (6.73–6.81) | 7.36× (7.32–7.39) | 6.69× (6.64–6.74) | 7.21× (7.17–7.25) | 6.64× (6.57–6.71) | 2.12× (2.11–2.14) | 2.43× (2.42–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.71× (2.60–2.83) | 5.97× (5.85–6.08) | 6.22× (6.17–6.26) | 6.76× (6.70–6.82) | 5.94× (5.85–6.03) | 6.34× (6.27–6.41) | 5.84× (5.77–5.91) | 1.87× (1.85–1.88) | 2.14× (2.12–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.61× (2.57–2.65) | 4.39× (4.34–4.45) | 4.71× (4.65–4.78) | 4.89× (4.83–4.95) | 4.45× (4.41–4.49) | 3.95× (3.90–4.00) | 3.64× (3.59–3.69) | 1.16× (1.15–1.18) | 1.33× (1.31–1.35) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.28× (3.21–3.35) | 4.08× (4.00–4.16) | 3.34× (3.27–3.41) | 5.74× (5.62–5.86) | 2.80× (2.74–2.86) | 2.51× (2.46–2.56) | 1.02× (0.99–1.05) | 1.06× (1.04–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.61–0.64) | 3.04× (3.01–3.08) | 5.35× (5.27–5.43) | 3.27× (3.24–3.31) | 5.53× (5.45–5.61) | 2.70× (2.66–2.73) | 2.42× (2.40–2.44) | 0.99× (0.96–1.01) | 1.03× (1.00–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.39× (3.34–3.44) | 5.40× (5.28–5.51) | 3.30× (3.24–3.36) | 5.56× (5.49–5.64) | 2.73× (2.69–2.77) | 2.45× (2.42–2.49) | 1.00× (0.98–1.02) | 1.04× (1.03–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.58–2.81) | 2.80× (2.77–2.83) | 3.60× (3.45–3.75) | 3.68× (3.64–3.72) | 3.98× (3.93–4.04) | 2.48× (2.45–2.50) | 2.05× (2.03–2.07) | 1.97× (1.95–1.99) | 2.73× (2.70–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.20× (2.16–2.24) | 2.43× (2.37–2.49) | 3.00× (2.95–3.04) | 3.13× (3.05–3.22) | 3.27× (3.21–3.33) | 1.94× (1.91–1.98) | 1.61× (1.58–1.64) | 1.55× (1.52–1.58) | 2.15× (2.10–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.90–1.99) | 1.71× (1.66–1.76) | 2.12× (2.07–2.18) | 2.02× (1.96–2.07) | 2.02× (1.96–2.08) | 0.99× (0.96–1.02) | 0.82× (0.80–0.84) | 0.79× (0.77–0.81) | 1.09× (1.06–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.33–1.37) | 2.64× (2.60–2.68) | 2.36× (2.33–2.39) | 2.45× (2.42–2.49) | 2.84× (2.79–2.89) | 3.10× (3.05–3.15) | 3.12× (3.06–3.17) | 1.22× (1.20–1.24) | 1.37× (1.34–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.48× (2.46–2.50) | 2.18× (2.16–2.21) | 2.78× (2.75–2.81) | 2.57× (2.54–2.61) | 2.77× (2.73–2.81) | 2.79× (2.74–2.83) | 1.09× (1.08–1.10) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.50–1.57) | 2.32× (2.26–2.38) | 1.88× (1.84–1.92) | 2.50× (2.44–2.56) | 2.46× (2.41–2.52) | 2.07× (2.01–2.12) | 2.08× (2.02–2.14) | 0.81× (0.79–0.83) | 0.91× (0.89–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.46–1.49) | 3.02× (2.97–3.06) | 2.51× (2.48–2.53) | 2.63× (2.60–2.66) | 3.10× (3.01–3.18) | 3.60× (3.57–3.64) | 3.84× (3.80–3.88) | 1.40× (1.38–1.41) | 1.65× (1.63–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.39) | 2.75× (2.72–2.78) | 2.30× (2.26–2.34) | 3.07× (3.02–3.12) | 2.75× (2.71–2.79) | 3.18× (3.13–3.23) | 3.39× (3.33–3.45) | 1.23× (1.21–1.25) | 1.46× (1.44–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.64× (1.61–1.67) | 2.54× (2.49–2.60) | 1.93× (1.88–1.98) | 2.66× (2.58–2.74) | 2.53× (2.50–2.57) | 2.26× (2.22–2.29) | 2.41× (2.37–2.45) | 0.88× (0.86–0.89) | 1.04× (1.01–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.70× (2.65–2.74) | 3.53× (3.48–3.58) | 2.83× (2.79–2.88) | 6.02× (5.90–6.14) | 2.33× (2.29–2.37) | 2.14× (2.10–2.18) | 0.99× (0.96–1.02) | 1.00× (0.96–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.47× (2.43–2.51) | 4.25× (4.20–4.30) | 2.74× (2.70–2.78) | 5.75× (5.54–5.97) | 2.17× (2.13–2.20) | 1.99× (1.96–2.02) | 0.92× (0.90–0.95) | 0.93× (0.89–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.59–0.62) | 2.72× (2.66–2.78) | 4.31× (4.20–4.42) | 2.76× (2.71–2.82) | 5.78× (5.61–5.96) | 2.18× (2.14–2.23) | 2.01× (1.96–2.05) | 0.93× (0.90–0.95) | 0.94× (0.91–0.97) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.88× (1.82–1.93) | 3.86× (3.75–3.98) | 4.22× (4.09–4.34) | 4.32× (4.20–4.45) | 4.55× (4.41–4.69) | 5.00× (4.85–5.14) | 4.58× (4.44–4.72) | 1.46× (1.42–1.50) | 1.73× (1.68–1.78) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.60× (1.57–1.64) | 3.24× (3.16–3.33) | 3.52× (3.44–3.60) | 3.88× (3.81–3.94) | 3.68× (3.60–3.77) | 3.98× (3.89–4.08) | 3.65× (3.56–3.74) | 1.16× (1.14–1.19) | 1.38× (1.34–1.41) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 1.72× (1.65–1.79) | 2.61× (2.51–2.71) | 2.85× (2.77–2.94) | 3.05× (2.98–3.14) | 3.03× (2.98–3.08) | 2.62× (2.55–2.70) | 2.40× (2.33–2.48) | 0.77× (0.74–0.79) | 0.91× (0.88–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.25× (0.24–0.26) | 0.81× (0.81–0.81) | 0.64× (0.64–0.65) | 9.11× (9.10–9.12) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.35) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 83 MB | 0.23× (0.22–0.24) | 0.34× (0.28–0.41) | 0.47× (0.46–0.49) | 0.39× (0.38–0.41) | 3.78× (3.66–3.91) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 133 MB | 0.23× (0.21–0.25) | 0.44× (0.37–0.52) | 0.41× (0.38–0.45) | 0.36× (0.33–0.39) | 2.39× (2.19–2.60) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.09× (0.08–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.29) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.71) | 0.66× (0.65–0.66) | 8.02× (8.00–8.05) | 0.65× (0.64–0.65) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.68) | 0.68× (0.62–0.74) | 1.57× (1.53–1.61) | 0.78× (0.76–0.80) | 5.69× (5.55–5.83) | 0.46× (0.44–0.47) | 0.49× (0.47–0.50) | 0.53× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.72× (0.67–0.76) | 1.18× (1.16–1.20) | 0.72× (0.71–0.74) | 3.47× (3.43–3.50) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.31× (0.31–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.38–0.44) | 0.91× (0.88–0.94) | 0.70× (0.67–0.74) | 3.14× (3.04–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.45–0.46) | 0.87× (0.85–0.88) | 0.82× (0.80–0.83) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.23–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.35× (0.33–0.38) | 0.86× (0.83–0.88) | 0.71× (0.68–0.74) | 2.94× (2.84–3.03) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.46–0.52) | 0.80× (0.79–0.82) | 0.80× (0.76–0.83) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.38× (5.37–5.38) | 0.35× (0.35–0.35) | 0.38× (0.38–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.76) | 5.29× (5.27–5.30) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.90× (0.89–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.76× (0.76–0.77) | 5.32× (5.30–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.65–0.66) | 6.41× (6.40–6.42) | 0.39× (0.39–0.40) | 0.43× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 37 MB | 0.46× (0.44–0.47) | 0.43× (0.41–0.45) | 1.08× (1.05–1.11) | 0.68× (0.65–0.71) | 3.35× (3.25–3.46) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.48× (0.45–0.51) | 0.91× (0.89–0.92) | 0.73× (0.71–0.75) | 2.30× (2.26–2.34) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 398 | 2.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 631 | 2.88 | 1.14× (1.13–1.15) | 80% | 1.03× (1.02–1.04) | 1.08× (1.03–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 788 | 4.61 | 1.82× (1.80–1.85) | 50% | 1.22× (1.20–1.23) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (788 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (292 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.08, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.77 | 0.02 | 0.08 | 0.266 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.5% |
| MTF + RLE2 | 1.55 | 17.3% |
| T-vector build | 1.49 | 16.7% |
| walk (the one each build uses) | 4.25 | 47.5% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.94 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.484 | 1.318 | (in its total) | 2.72× (2.71–2.74) |  |
| walk | 1.265 | 2.663 | (in its total) | 2.11× (2.09–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.059 | (in its total) | 1.23× (1.20–1.26) |  |
| inverse BWT, total | 1.797 | 4.040 | 3.280 | 2.25× (2.23–2.27) | 1.83× (1.82–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m52s |
| 4 threads | 38m30s ± 5m26s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m05s ± 3m59s |
| the scanner alone (its serial part) | 13m41s ± 32.2s |
| the workers alone, over 4 | 28m05s ± 3m59s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 3.68 ns a byte of output at 4 threads (CPU, summed over them), 2.07 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,439,877 | 12,292,642 | \`3897e199bbd7462e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
