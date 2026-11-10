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

The latest record: [`c9329f0`](<https://github.com/lmmx/mibz2/commit/c9329f010c3cc77166c56b8be42eb1ccfebe79ba>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42815016494>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.563 s | 372 | 7.0 | 2.85× (2.82–2.89) | 6.58× (6.49–6.66) | 6.74× (6.65–6.83) | 7.65× (7.55–7.75) | 6.76× (6.64–6.88) | 7.02× (6.94–7.10) | 6.77× (6.67–6.87) | 2.11× (2.07–2.14) | 2.31× (2.29–2.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.330 s | 636 | 7.7 | 2.78× (2.72–2.85) | 5.89× (5.70–6.08) | 6.62× (6.39–6.86) | 7.04× (6.93–7.16) | 6.20× (5.99–6.41) | 12.03× (11.61–12.47) | 11.60× (11.25–11.97) | 3.61× (3.45–3.77) | 3.96× (3.81–4.11) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.295 s | 710 | 13.3 | 2.64× (2.58–2.70) | 3.97× (3.83–4.12) | 4.93× (4.78–5.07) | 4.85× (4.74–4.97) | 4.79× (4.60–4.98) | 13.60× (13.17–14.05) | 13.12× (12.73–13.52) | 4.08× (3.90–4.27) | 4.48× (4.33–4.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 909 | 3.1 | 0.60× (0.59–0.61) | 3.60× (3.44–3.78) | 4.51× (4.43–4.59) | 3.69× (3.63–3.75) | 6.18× (6.01–6.36) | 2.77× (2.72–2.82) | 2.77× (2.71–2.83) | 1.00× (0.97–1.03) | 1.85× (1.82–1.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 881 | 3.3 | 0.55× (0.54–0.57) | 2.72× (2.66–2.79) | 5.78× (5.65–5.92) | 3.62× (3.53–3.70) | 5.96× (5.83–6.10) | 2.71× (2.64–2.77) | 2.71× (2.65–2.77) | 0.97× (0.94–1.00) | 1.81× (1.77–1.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 879 | 3.2 | 0.56× (0.55–0.57) | 2.82× (2.68–2.96) | 5.99× (5.54–6.47) | 3.62× (3.56–3.68) | 6.08× (5.94–6.22) | 2.70× (2.66–2.75) | 2.71× (2.66–2.76) | 0.97× (0.94–1.00) | 1.81× (1.78–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 581 | 4.7 | 2.76× (2.73–2.80) | 3.03× (3.01–3.05) | 3.87× (3.68–4.07) | 4.17× (4.14–4.20) | 4.53× (4.44–4.62) | 2.80× (2.79–2.82) | 2.31× (2.29–2.33) | 2.14× (2.11–2.17) | 3.02× (3.00–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 830 | 5.8 | 2.09× (1.78–2.46) | 2.67× (2.21–3.22) | 3.99× (3.40–4.68) | 3.35× (2.83–3.95) | 4.65× (4.02–5.39) | 3.78× (3.23–4.42) | 3.11× (2.66–3.65) | 2.89× (2.50–3.33) | 4.07× (3.48–4.76) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 755 | 11.4 | 1.90× (1.80–2.01) | 1.70× (1.58–1.83) | 2.96× (2.75–3.19) | 2.18× (2.07–2.30) | 4.07× (3.76–4.41) | 3.58× (3.42–3.75) | 2.95× (2.81–3.09) | 2.74× (2.60–2.88) | 3.85× (3.68–4.03) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.144 s | 56 | 47.4 | 1.38× (1.37–1.38) | 2.59× (2.56–2.62) | 2.32× (2.28–2.36) | 2.41× (2.38–2.43) | 2.75× (2.72–2.77) | 2.97× (2.94–3.00) | 3.10× (3.07–3.12) | 1.26× (1.24–1.28) | 1.41× (1.39–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.090 s | 89 | 53.7 | 1.23× (1.17–1.29) | 2.33× (2.20–2.47) | 2.18× (2.06–2.30) | 2.88× (2.76–3.01) | 2.67× (2.53–2.82) | 4.61× (4.37–4.87) | 4.81× (4.57–5.06) | 1.96× (1.86–2.06) | 2.19× (2.07–2.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 69.6 | 1.33× (1.30–1.37) | 2.18× (2.13–2.23) | 2.12× (2.01–2.23) | 2.67× (2.62–2.73) | 2.94× (2.79–3.09) | 5.92× (5.76–6.08) | 6.17× (6.02–6.33) | 2.51× (2.43–2.59) | 2.81× (2.73–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.155 s | 62 | 42.4 | 1.54× (1.52–1.55) | 3.01× (2.98–3.04) | 2.57× (2.54–2.61) | 2.77× (2.73–2.81) | 3.15× (3.05–3.25) | 3.68× (3.66–3.71) | 4.03× (4.00–4.07) | 1.54× (1.50–1.58) | 1.80× (1.77–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.093 s | 103 | 48.0 | 1.37× (1.30–1.45) | 2.74× (2.56–2.93) | 2.47× (2.34–2.60) | 3.53× (3.36–3.69) | 3.03× (2.89–3.19) | 5.93× (5.65–6.22) | 6.49× (6.18–6.82) | 2.48× (2.34–2.62) | 2.89× (2.74–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.082 s | 118 | 67.4 | 1.37× (1.26–1.48) | 2.23× (2.04–2.43) | 2.14× (1.97–2.33) | 3.02× (2.77–3.29) | 2.79× (2.55–3.05) | 6.80× (6.28–7.37) | 7.45× (6.88–8.07) | 2.84× (2.61–3.10) | 3.32× (3.06–3.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 685 | 4.3 | 0.67× (0.64–0.71) | 2.83× (2.80–2.86) | 4.13× (4.04–4.23) | 3.23× (3.20–3.26) | 6.86× (6.71–7.01) | 2.40× (2.38–2.42) | 2.45× (2.43–2.46) | 0.98× (0.97–0.99) | 2.12× (2.10–2.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 647 | 4.7 | 0.54× (0.53–0.54) | 2.66× (2.61–2.71) | 5.08× (4.85–5.32) | 3.04× (2.98–3.10) | 6.47× (6.29–6.65) | 2.25× (2.21–2.29) | 2.29× (2.25–2.33) | 0.92× (0.89–0.94) | 1.99× (1.95–2.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 639 | 4.6 | 0.54× (0.53–0.55) | 2.65× (2.62–2.67) | 4.91× (4.71–5.12) | 3.03× (3.00–3.07) | 6.50× (6.35–6.65) | 2.23× (2.20–2.26) | 2.27× (2.25–2.30) | 0.91× (0.89–0.93) | 1.97× (1.95–1.99) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.074 s | 174 | 15.3 | 1.80× (1.74–1.87) | 3.86× (3.73–3.99) | 4.27× (4.13–4.41) | 4.52× (4.37–4.68) | 4.73× (4.62–4.84) | 4.65× (4.48–4.83) | 4.72× (4.54–4.92) | 1.49× (1.42–1.56) | 1.67× (1.61–1.74) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.050 s | 255 | 19.1 | 1.57× (1.42–1.73) | 3.15× (2.99–3.31) | 3.74× (3.55–3.95) | 4.44× (4.07–4.85) | 4.24× (4.02–4.48) | 6.75× (6.41–7.12) | 6.86× (6.50–7.23) | 2.16× (2.03–2.30) | 2.43× (2.29–2.58) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.047 s | 274 | 29.1 | 1.44× (1.38–1.50) | 2.41× (2.28–2.55) | 3.11× (2.92–3.30) | 3.46× (3.32–3.60) | 3.93× (3.67–4.21) | 7.45× (7.30–7.61) | 7.57× (7.41–7.72) | 2.39× (2.30–2.48) | 2.68× (2.58–2.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.85× (2.81–2.88) | 6.55× (6.46–6.63) | 6.71× (6.62–6.79) | 7.63× (7.53–7.73) | 6.74× (6.62–6.86) | 7.00× (6.92–7.08) | 6.75× (6.65–6.85) | 2.10× (2.07–2.14) | 2.30× (2.28–2.32) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.67× (2.64–2.71) | 6.02× (5.91–6.14) | 6.32× (6.19–6.45) | 7.32× (7.26–7.39) | 6.15× (6.03–6.29) | 6.32× (6.20–6.44) | 6.10× (5.99–6.20) | 1.90× (1.85–1.95) | 2.08× (2.03–2.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.59× (2.49–2.68) | 4.26× (4.09–4.44) | 4.73× (4.54–4.92) | 5.18× (5.01–5.35) | 4.53× (4.34–4.74) | 3.74× (3.62–3.87) | 3.61× (3.49–3.74) | 1.12× (1.08–1.17) | 1.23× (1.19–1.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.06× (2.99–3.14) | 4.05× (3.97–4.13) | 3.46× (3.39–3.52) | 5.73× (5.57–5.90) | 2.61× (2.56–2.66) | 2.62× (2.56–2.68) | 1.00× (0.97–1.02) | 1.68× (1.65–1.71) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.81× (2.75–2.87) | 5.03× (4.94–5.13) | 3.37× (3.29–3.44) | 5.47× (5.35–5.59) | 2.52× (2.46–2.57) | 2.52× (2.48–2.57) | 0.96× (0.93–0.99) | 1.62× (1.59–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.13× (3.09–3.17) | 5.26× (4.88–5.68) | 3.43× (3.39–3.48) | 5.66× (5.54–5.78) | 2.57× (2.53–2.60) | 2.57× (2.53–2.61) | 0.98× (0.95–1.01) | 1.65× (1.63–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.62–2.69) | 2.88× (2.87–2.90) | 3.50× (3.35–3.66) | 3.98× (3.96–4.01) | 4.32× (4.23–4.41) | 2.70× (2.68–2.71) | 2.23× (2.22–2.25) | 2.07× (2.04–2.10) | 2.69× (2.65–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.23–2.40) | 2.53× (2.46–2.60) | 3.36× (3.25–3.48) | 3.37× (3.26–3.48) | 3.60× (3.49–3.72) | 2.17× (2.10–2.24) | 1.79× (1.73–1.86) | 1.67× (1.63–1.70) | 2.16× (2.09–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.15× (2.08–2.22) | 1.74× (1.68–1.81) | 2.36× (2.19–2.54) | 2.08× (2.01–2.17) | 2.20× (2.10–2.31) | 1.13× (1.09–1.16) | 0.93× (0.90–0.96) | 0.87× (0.84–0.90) | 1.12× (1.08–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.53× (2.52–2.55) | 2.26× (2.23–2.29) | 2.39× (2.36–2.41) | 2.72× (2.69–2.75) | 2.94× (2.91–2.97) | 3.07× (3.04–3.09) | 1.26× (1.24–1.27) | 1.36× (1.35–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.19–1.32) | 2.34× (2.23–2.46) | 2.06× (1.94–2.17) | 2.79× (2.65–2.94) | 2.41× (2.29–2.53) | 2.56× (2.43–2.69) | 2.67× (2.55–2.80) | 1.09× (1.04–1.15) | 1.19× (1.13–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.45–1.54) | 2.32× (2.22–2.43) | 1.94× (1.88–2.01) | 2.63× (2.53–2.75) | 2.55× (2.44–2.67) | 2.01× (1.96–2.06) | 2.09× (2.04–2.15) | 0.86× (0.84–0.88) | 0.93× (0.91–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.54) | 2.95× (2.93–2.98) | 2.53× (2.49–2.56) | 2.75× (2.71–2.79) | 3.12× (3.02–3.22) | 3.65× (3.62–3.67) | 3.99× (3.96–4.02) | 1.53× (1.50–1.57) | 1.75× (1.73–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.36–1.47) | 2.78× (2.65–2.92) | 2.31× (2.25–2.38) | 3.28× (3.20–3.36) | 2.75× (2.69–2.82) | 3.20× (3.11–3.28) | 3.50× (3.41–3.59) | 1.34× (1.31–1.38) | 1.53× (1.50–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.43–1.68) | 2.48× (2.26–2.73) | 1.92× (1.75–2.12) | 2.77× (2.53–3.03) | 2.57× (2.33–2.84) | 2.22× (2.03–2.42) | 2.43× (2.23–2.65) | 0.93× (0.85–1.02) | 1.06× (0.97–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.68–0.74) | 2.56× (2.54–2.58) | 3.64× (3.52–3.76) | 2.96× (2.94–2.98) | 6.13× (6.00–6.27) | 2.22× (2.21–2.24) | 2.27× (2.26–2.28) | 0.99× (0.97–1.00) | 1.41× (1.39–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.58–0.61) | 2.35× (2.30–2.39) | 4.18× (4.06–4.29) | 2.81× (2.76–2.86) | 5.72× (5.58–5.87) | 2.05× (2.02–2.08) | 2.09× (2.06–2.12) | 0.91× (0.89–0.93) | 1.30× (1.27–1.33) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.57× (2.54–2.59) | 4.17× (4.12–4.22) | 2.89× (2.86–2.92) | 5.93× (5.81–6.06) | 2.10× (2.08–2.13) | 2.15× (2.12–2.17) | 0.93× (0.91–0.95) | 1.33× (1.31–1.35) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.79× (1.72–1.86) | 3.75× (3.63–3.88) | 4.15× (4.02–4.28) | 4.44× (4.30–4.59) | 4.64× (4.55–4.74) | 4.57× (4.41–4.73) | 4.64× (4.46–4.82) | 1.48× (1.41–1.56) | 1.61× (1.56–1.67) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.60× (1.49–1.73) | 3.25× (3.19–3.32) | 3.59× (3.51–3.68) | 4.18× (4.10–4.26) | 3.83× (3.75–3.91) | 3.73× (3.66–3.81) | 3.79× (3.70–3.88) | 1.21× (1.17–1.26) | 1.31× (1.29–1.35) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.65× (1.54–1.77) | 2.74× (2.60–2.88) | 3.05× (2.88–3.24) | 3.39× (3.29–3.50) | 3.41× (3.11–3.74) | 2.45× (2.42–2.48) | 2.49× (2.45–2.52) | 0.79× (0.77–0.82) | 0.86× (0.85–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.31–0.32) | 1.05× (1.05–1.06) | 0.84× (0.83–0.84) | 11.91× (11.90–11.92) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.46× (0.39–0.55) | 0.76× (0.75–0.77) | 0.64× (0.63–0.64) | 5.98× (5.94–6.03) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.38) | 0.58× (0.49–0.69) | 0.67× (0.66–0.68) | 0.59× (0.58–0.60) | 3.93× (3.90–3.95) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.16× (0.15–0.16) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.65× (0.64–0.66) | 8.01× (7.99–8.04) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.65× (0.61–0.70) | 1.63× (1.61–1.66) | 0.82× (0.80–0.83) | 5.92× (5.84–6.01) | 0.47× (0.47–0.48) | 0.50× (0.50–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.68× (0.63–0.74) | 1.14× (1.10–1.19) | 0.69× (0.65–0.72) | 3.31× (3.16–3.46) | 0.26× (0.24–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.21× (5.19–5.22) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.40× (0.36–0.43) | 0.89× (0.86–0.93) | 0.71× (0.68–0.75) | 3.09× (2.98–3.20) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.85–0.88) | 0.80× (0.79–0.81) | 2.51× (2.47–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.21× (5.21–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.37× (0.34–0.39) | 0.85× (0.82–0.87) | 0.71× (0.67–0.75) | 2.91× (2.83–3.00) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.51× (0.49–0.54) | 0.81× (0.80–0.82) | 0.80× (0.77–0.83) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.25–0.25) | 0.21× (0.20–0.21) | 1.61× (1.61–1.62) | 0.77× (0.77–0.78) | 5.33× (5.32–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.77–0.78) | 5.33× (5.32–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.65× (0.65–0.66) | 6.35× (6.33–6.37) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.43× (0.40–0.46) | 1.15× (1.14–1.17) | 0.74× (0.73–0.75) | 3.61× (3.57–3.65) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 53 MB | 0.52× (0.50–0.53) | 0.51× (0.47–0.55) | 0.95× (0.92–0.97) | 0.76× (0.73–0.80) | 2.43× (2.36–2.49) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 372 | 2.70 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 636 | 2.98 | 1.11× (1.09–1.13) | 86% | 1.02× (1.00–1.04) | 1.04× (1.01–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 710 | 5.13 | 1.87× (1.81–1.93) | 48% | 1.22× (1.19–1.24) | 1.70× (1.64–1.76) |

**Best:** mibz2 at 4 threads (710 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (273 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.19, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 0.86 | 0.02 | 0.09 | 0.295 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 12.8% |
| MTF + RLE2 | 1.44 | 18.3% |
| T-vector build | 1.35 | 17.2% |
| walk (the one each build uses) | 3.67 | 46.7% |
| of which: segment assembly | 0.13 | 1.6% |
| RLE1 | 0.17 | 2.2% |
| CRC | 0.23 | 3.0% |
| **total** | 7.86 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.504 | 1.373 | (in its total) | 2.74× (2.70–2.78) |  |
| walk | 1.380 | 2.658 | (in its total) | 1.95× (1.91–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.053 | (in its total) | 1.11× (1.04–1.19) |  |
| inverse BWT, total | 1.933 | 4.085 | 2.466 | 2.13× (2.09–2.17) | 1.28× (1.27–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h22m ± 9m17s |
| 4 threads | 42m39s ± 6m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m42s ± 4m28s |
| the scanner alone (its serial part) | 16m44s ± 32.9s |
| the workers alone, over 4 | 30m42s ± 4m28s |

At 4 threads the workers bound it. Per byte: the scanner 9.7 ns a byte of bz2, the workers 4.03 ns a byte of output at 4 threads (CPU, summed over them), 2.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,560,162 | 12,831,886 | \`440dcb959c00cf7f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.0 (bench/report.py), 10 rounds, threads 1, 2, 4
