# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-106/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`7805524`](<https://github.com/lmmx/mibz2/commit/7805524a705c0aab3af6c98a70132e0a74ff0272>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40923825220>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.800 s | 262 | 9.3 | 1.92× (1.90–1.94) | 4.36× (4.32–4.40) | 4.60× (4.56–4.65) | 4.89× (4.84–4.96) | 4.43× (4.39–4.48) | 4.73× (4.68–4.78) | 4.41× (4.36–4.46) | 1.43× (1.42–1.45) | 1.67× (1.66–1.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.510 s | 411 | 10.3 | 1.67× (1.63–1.72) | 3.57× (3.48–3.65) | 4.18× (4.07–4.29) | 4.07× (4.01–4.13) | 3.73× (3.67–3.79) | 7.35× (7.26–7.45) | 6.86× (6.77–6.96) | 2.23× (2.20–2.26) | 2.61× (2.58–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.404 s | 519 | 15.9 | 1.78× (1.75–1.81) | 2.79× (2.75–2.83) | 3.40× (3.34–3.47) | 3.06× (3.04–3.08) | 3.16× (3.10–3.22) | 9.31× (9.21–9.42) | 8.69× (8.60–8.78) | 2.83× (2.80–2.85) | 3.30× (3.26–3.34) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 930 | 2.9 | 0.58× (0.57–0.58) | 3.75× (3.64–3.87) | 4.69× (4.64–4.74) | 3.42× (3.38–3.46) | 6.45× (6.38–6.51) | 2.70× (2.67–2.73) | 2.53× (2.51–2.56) | 1.02× (1.00–1.04) | 1.43× (1.42–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 930 | 2.9 | 0.57× (0.56–0.57) | 2.89× (2.85–2.92) | 6.19× (6.12–6.26) | 3.47× (3.43–3.51) | 6.46× (6.39–6.53) | 2.72× (2.69–2.75) | 2.55× (2.53–2.58) | 1.03× (1.00–1.05) | 1.45× (1.43–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 915 | 2.9 | 0.57× (0.55–0.58) | 2.83× (2.78–2.88) | 6.04× (5.94–6.14) | 3.38× (3.33–3.43) | 6.32× (6.22–6.43) | 2.66× (2.61–2.70) | 2.49× (2.45–2.53) | 1.00× (0.98–1.03) | 1.41× (1.39–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 s | 437 | 5.8 | 1.99× (1.95–2.03) | 2.29× (2.24–2.33) | 2.85× (2.73–2.97) | 2.88× (2.83–2.93) | 3.41× (3.36–3.46) | 1.88× (1.85–1.92) | 1.59× (1.57–1.62) | 1.56× (1.53–1.59) | 2.28× (2.23–2.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.029 s | 682 | 7.1 | 1.74× (1.62–1.86) | 2.18× (2.01–2.35) | 3.18× (3.04–3.32) | 2.66× (2.50–2.84) | 4.13× (3.93–4.34) | 2.88× (2.74–3.03) | 2.44× (2.32–2.57) | 2.39× (2.27–2.52) | 3.48× (3.30–3.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.031 s | 625 | 13.9 | 1.46× (1.43–1.50) | 1.66× (1.59–1.74) | 2.47× (2.35–2.60) | 1.83× (1.74–1.93) | 3.52× (3.42–3.62) | 2.67× (2.58–2.75) | 2.26× (2.19–2.33) | 2.21× (2.15–2.28) | 3.22× (3.12–3.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.155 s | 52 | 48.0 | 1.12× (1.05–1.21) | 2.27× (2.10–2.45) | 2.04× (1.89–2.20) | 2.04× (1.89–2.19) | 2.44× (2.26–2.62) | 2.54× (2.35–2.73) | 2.56× (2.37–2.77) | 1.04× (0.96–1.12) | 1.14× (1.06–1.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.097 s | 83 | 53.4 | 1.09× (1.01–1.17) | 2.10× (1.98–2.21) | 2.02× (1.90–2.13) | 2.50× (2.37–2.64) | 2.54× (2.39–2.70) | 4.09× (3.86–4.33) | 4.13× (3.89–4.38) | 1.67× (1.58–1.77) | 1.84× (1.74–1.95) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 105 | 69.5 | 1.21× (1.19–1.24) | 1.95× (1.88–2.02) | 1.90× (1.86–1.93) | 2.34× (2.29–2.40) | 2.75× (2.68–2.82) | 5.33× (5.22–5.45) | 5.39× (5.27–5.51) | 2.18× (2.13–2.22) | 2.40× (2.37–2.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.203 s | 47 | 52.2 | 1.06× (1.05–1.08) | 2.27× (2.23–2.31) | 1.87× (1.85–1.90) | 1.87× (1.84–1.89) | 2.28× (2.20–2.37) | 2.65× (2.61–2.68) | 2.73× (2.68–2.78) | 1.01× (1.00–1.03) | 1.17× (1.14–1.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.119 s | 81 | 57.0 | 1.01× (0.99–1.03) | 2.14× (2.07–2.21) | 1.87× (1.82–1.92) | 2.68× (2.57–2.80) | 2.39× (2.33–2.45) | 4.51× (4.38–4.64) | 4.66× (4.53–4.79) | 1.73× (1.68–1.78) | 1.99× (1.94–2.05) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.094 s | 102 | 76.0 | 1.19× (1.16–1.21) | 1.88× (1.84–1.92) | 1.85× (1.80–1.89) | 2.47× (2.43–2.52) | 2.43× (2.39–2.46) | 5.71× (5.60–5.82) | 5.89× (5.77–6.02) | 2.19× (2.15–2.22) | 2.52× (2.47–2.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 683 | 4.1 | 0.63× (0.62–0.63) | 3.07× (2.84–3.33) | 4.14× (4.10–4.18) | 2.93× (2.90–2.96) | 7.17× (7.09–7.26) | 2.30× (2.28–2.32) | 2.16× (2.14–2.18) | 0.98× (0.97–0.98) | 1.42× (1.41–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 4.3 | 0.55× (0.54–0.56) | 2.74× (2.70–2.77) | 4.98× (4.77–5.20) | 2.86× (2.82–2.90) | 7.00× (6.91–7.08) | 2.24× (2.21–2.27) | 2.10× (2.07–2.13) | 0.95× (0.93–0.96) | 1.38× (1.36–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 4.2 | 0.55× (0.54–0.56) | 2.72× (2.68–2.76) | 4.87× (4.57–5.20) | 2.86× (2.83–2.89) | 6.91× (6.80–7.02) | 2.22× (2.18–2.25) | 2.08× (2.05–2.11) | 0.94× (0.92–0.95) | 1.36× (1.35–1.38) | yes |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 1 | 0.088 s | 138 | 18.2 | 1.38× (1.36–1.39) | 2.87× (2.83–2.91) | 3.21× (3.19–3.24) | 3.03× (3.01–3.06) | 3.40× (3.37–3.43) | 3.31× (3.29–3.34) | 3.14× (3.11–3.16) | 1.04× (1.03–1.05) | 1.27× (1.26–1.28) | yes |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 2 | 0.057 s | 214 | 21.1 | 1.18× (1.06–1.32) | 2.42× (2.28–2.56) | 2.82× (2.66–2.98) | 3.06× (2.86–3.28) | 3.30× (3.12–3.49) | 5.05× (4.76–5.35) | 4.78× (4.52–5.05) | 1.58× (1.50–1.68) | 1.93× (1.83–2.04) | yes |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 4 | 0.048 s | 252 | 30.9 | 1.20× (1.18–1.22) | 1.94× (1.85–2.03) | 2.40× (2.36–2.44) | 2.68× (2.61–2.76) | 3.17× (3.11–3.23) | 6.09× (5.96–6.22) | 5.77× (5.65–5.88) | 1.91× (1.88–1.94) | 2.33× (2.29–2.37) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.92× (1.90–1.94) | 4.35× (4.30–4.39) | 4.59× (4.54–4.63) | 4.89× (4.83–4.95) | 4.42× (4.38–4.47) | 4.72× (4.67–4.77) | 4.40× (4.35–4.45) | 1.43× (1.42–1.45) | 1.67× (1.65–1.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.80× (1.79–1.81) | 4.03× (3.99–4.06) | 4.31× (4.29–4.33) | 4.67× (4.63–4.70) | 4.06× (4.02–4.09) | 4.26× (4.21–4.30) | 3.98× (3.94–4.02) | 1.29× (1.28–1.30) | 1.50× (1.49–1.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.89× (1.87–1.90) | 3.22× (3.18–3.27) | 3.39× (3.37–3.41) | 3.55× (3.51–3.58) | 3.19× (3.13–3.25) | 2.75× (2.71–2.80) | 2.57× (2.53–2.61) | 0.84× (0.82–0.85) | 0.97× (0.96–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.15× (3.11–3.19) | 4.10× (4.06–4.15) | 3.22× (3.18–3.26) | 5.99× (5.93–6.04) | 2.55× (2.53–2.58) | 2.40× (2.38–2.43) | 1.02× (1.00–1.03) | 1.06× (1.02–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.02× (2.99–3.04) | 5.38× (5.32–5.43) | 3.23× (3.20–3.26) | 5.91× (5.86–5.96) | 2.53× (2.51–2.56) | 2.38× (2.36–2.41) | 1.01× (0.99–1.03) | 1.05× (1.01–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.32× (3.25–3.39) | 5.37× (5.26–5.48) | 3.22× (3.17–3.27) | 5.88× (5.78–5.99) | 2.52× (2.48–2.57) | 2.37× (2.33–2.42) | 1.00× (0.98–1.03) | 1.05× (1.01–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.95× (1.91–1.99) | 2.20× (2.17–2.24) | 2.68× (2.58–2.78) | 2.80× (2.75–2.85) | 3.31× (3.27–3.36) | 1.85× (1.81–1.88) | 1.57× (1.55–1.60) | 1.54× (1.51–1.57) | 2.10× (2.07–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.74× (1.71–1.76) | 1.98× (1.93–2.02) | 2.45× (2.42–2.48) | 2.44× (2.40–2.49) | 2.81× (2.77–2.86) | 1.52× (1.50–1.54) | 1.29× (1.27–1.31) | 1.27× (1.25–1.28) | 1.73× (1.70–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.59× (1.54–1.64) | 1.52× (1.46–1.58) | 1.80× (1.74–1.86) | 1.69× (1.62–1.77) | 1.77× (1.71–1.84) | 0.79× (0.77–0.82) | 0.67× (0.65–0.70) | 0.66× (0.64–0.68) | 0.90× (0.87–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.12× (1.04–1.21) | 2.22× (2.07–2.39) | 2.01× (1.87–2.16) | 2.02× (1.88–2.18) | 2.42× (2.25–2.60) | 2.52× (2.34–2.71) | 2.54× (2.35–2.74) | 1.03× (0.96–1.12) | 1.10× (1.03–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.10× (1.09–1.12) | 2.17× (2.14–2.20) | 1.93× (1.90–1.96) | 2.50× (2.48–2.53) | 2.30× (2.26–2.33) | 2.33× (2.30–2.36) | 2.35× (2.32–2.38) | 0.96× (0.94–0.97) | 1.02× (1.01–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.37× (1.34–1.41) | 2.06× (2.04–2.09) | 1.72× (1.71–1.74) | 2.35× (2.30–2.39) | 2.29× (2.23–2.35) | 1.80× (1.78–1.82) | 1.82× (1.79–1.84) | 0.74× (0.73–0.75) | 0.79× (0.78–0.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.06× (1.05–1.08) | 2.25× (2.21–2.28) | 1.84× (1.81–1.87) | 1.86× (1.83–1.88) | 2.27× (2.19–2.36) | 2.63× (2.59–2.67) | 2.72× (2.67–2.76) | 1.01× (1.00–1.03) | 1.14× (1.12–1.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.04) | 2.14× (2.13–2.16) | 1.75× (1.73–1.76) | 2.45× (2.41–2.50) | 2.12× (2.11–2.13) | 2.42× (2.40–2.44) | 2.50× (2.48–2.52) | 0.93× (0.93–0.94) | 1.05× (1.04–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.32× (1.29–1.35) | 2.05× (2.02–2.09) | 1.57× (1.54–1.60) | 2.28× (2.24–2.33) | 2.08× (2.04–2.12) | 1.82× (1.79–1.85) | 1.88× (1.85–1.91) | 0.70× (0.69–0.71) | 0.79× (0.78–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.59× (2.56–2.62) | 3.51× (3.47–3.54) | 2.70× (2.68–2.73) | 6.45× (6.39–6.50) | 2.15× (2.13–2.17) | 2.02× (2.00–2.04) | 0.98× (0.97–0.98) | 0.95× (0.92–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.62) | 2.44× (2.41–2.47) | 4.25× (4.21–4.30) | 2.65× (2.62–2.69) | 6.19× (6.12–6.26) | 2.05× (2.03–2.07) | 1.93× (1.90–1.95) | 0.93× (0.92–0.94) | 0.91× (0.87–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.68× (2.64–2.71) | 4.37× (4.17–4.57) | 2.75× (2.72–2.77) | 6.30× (6.21–6.39) | 2.09× (2.06–2.12) | 1.96× (1.94–1.99) | 0.95× (0.94–0.97) | 0.93× (0.89–0.96) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 1 | 1.37× (1.36–1.38) | 2.75× (2.73–2.77) | 3.09× (3.06–3.11) | 2.99× (2.96–3.01) | 3.35× (3.32–3.38) | 3.26× (3.24–3.29) | 3.09× (3.07–3.12) | 1.04× (1.03–1.05) | 1.23× (1.22–1.24) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 2 | 1.27× (1.25–1.29) | 2.46× (2.42–2.50) | 2.79× (2.76–2.83) | 2.88× (2.84–2.92) | 2.94× (2.90–2.99) | 2.81× (2.77–2.85) | 2.67× (2.62–2.71) | 0.90× (0.88–0.91) | 1.06× (1.05–1.07) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 4 | 1.41× (1.35–1.47) | 2.10× (2.04–2.17) | 2.37× (2.34–2.39) | 2.38× (2.35–2.42) | 2.44× (2.42–2.47) | 1.93× (1.92–1.95) | 1.83× (1.81–1.85) | 0.62× (0.61–0.62) | 0.73× (0.72–0.74) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.31× (0.29–0.32) | 1.17× (1.17–1.17) | 0.84× (0.83–0.84) | 11.88× (11.86–11.90) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.37× (0.37–0.37) | 0.53× (0.47–0.60) | 0.86× (0.84–0.87) | 0.66× (0.66–0.67) | 6.30× (6.24–6.35) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.75× (0.67–0.85) | 0.78× (0.77–0.79) | 0.64× (0.63–0.65) | 4.29× (4.27–4.31) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.29–4.29) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.18–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.10× (0.10–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.48–0.49) | 0.43× (0.42–0.43) | 1.64× (1.64–1.65) | 0.63× (0.63–0.64) | 7.80× (7.77–7.83) | 0.62× (0.61–0.62) | 0.65× (0.64–0.65) | 0.71× (0.71–0.72) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.62× (0.61–0.63) | 0.67× (0.61–0.73) | 1.54× (1.52–1.56) | 0.87× (0.86–0.89) | 5.56× (5.49–5.64) | 0.44× (0.43–0.44) | 0.46× (0.45–0.47) | 0.51× (0.50–0.51) | 0.25× (0.24–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.58–0.61) | 0.67× (0.63–0.70) | 1.12× (1.10–1.13) | 0.85× (0.84–0.86) | 3.22× (3.19–3.26) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.15–1.16) | 0.71× (0.71–0.72) | 5.10× (5.10–5.11) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.51–0.51) | 0.25× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.51× (0.50–0.51) | 0.44× (0.43–0.44) | 0.94× (0.93–0.95) | 0.72× (0.70–0.75) | 3.27× (3.24–3.30) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.60× (0.58–0.61) | 0.42× (0.42–0.43) | 0.81× (0.80–0.83) | 0.77× (0.75–0.78) | 2.38× (2.34–2.42) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.81× (0.80–0.81) | 5.11× (5.11–5.12) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.50× (0.50–0.51) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.35× (0.32–0.38) | 0.87× (0.87–0.88) | 0.71× (0.69–0.74) | 3.02× (2.99–3.05) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.54–0.57) | 0.46× (0.43–0.49) | 0.74× (0.73–0.76) | 0.76× (0.74–0.79) | 2.16× (2.12–2.20) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.09× (1.09–1.09) | 0.76× (0.76–0.76) | 5.43× (5.42–5.44) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.19–0.20) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.39× (5.38–5.41) | 0.34× (0.34–0.34) | 0.36× (0.36–0.36) | 0.91× (0.91–0.92) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.63× (1.63–1.63) | 0.77× (0.77–0.77) | 5.41× (5.40–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.12–0.13) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.39× (0.38–0.39) | 1.45× (1.45–1.45) | 0.62× (0.62–0.62) | 6.54× (6.52–6.55) | 0.39× (0.38–0.39) | 0.41× (0.40–0.41) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 2 | 29 MB | 0.55× (0.55–0.56) | 0.49× (0.45–0.54) | 1.36× (1.34–1.37) | 0.81× (0.77–0.84) | 4.22× (4.18–4.27) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.41× (0.40–0.42) | 0.20× (0.20–0.20) |
| `text` (1.58 MB → 12.1 MB, 7.7×) | 4 | 49 MB | 0.55× (0.54–0.56) | 0.50× (0.46–0.54) | 1.03× (1.02–1.05) | 0.81× (0.80–0.83) | 2.64× (2.60–2.68) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 262 | 3.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 411 | 4.20 | 1.11× (1.10–1.12) | 78% | 1.03× (1.02–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 519 | 6.49 | 1.71× (1.69–1.74) | 49% | 1.27× (1.25–1.30) | 1.68× (1.65–1.71) |

**Best:** mibz2 at 4 threads (519 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (292 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 0.94, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.15 | 0.02 | 0.07 | 0.404 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m13s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 9.5% |
| MTF + RLE2 | 1.70 | 17.5% |
| T-vector build | 1.72 | 17.7% |
| walk (the one each build uses) | 5.00 | 51.4% |
| of which: segment assembly | 0.15 | 1.5% |
| RLE1 | 0.21 | 2.2% |
| CRC | 0.19 | 2.0% |
| **total** | 9.73 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.696 | 1.310 | (in its total) | 1.88× (1.88–1.88) |  |
| walk | 1.610 | 2.669 | (in its total) | 1.65× (1.62–1.67) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.053 | 0.057 | (in its total) | 1.07× (1.06–1.09) |  |
| inverse BWT, total | 2.358 | 4.034 | 3.402 | 1.70× (1.68–1.72) | 1.44× (1.42–1.45) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h56m ± 14m02s |
| 4 threads | 58m58s ± 8m17s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m12s ± 5m57s |
| the scanner alone (its serial part) | 16m26s ± 30.2s |
| the workers alone, over 4 | 42m12s ± 5m57s |

At 4 threads the workers bound it. Per byte: the scanner 9.5 ns a byte of bz2, the workers 5.53 ns a byte of output at 4 threads (CPU, summed over them), 3.27 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,576,486 | 12,118,601 | \`73441f255ec7869b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
