# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-172/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`e7c30a8`](<https://github.com/lmmx/mibz2/commit/e7c30a86f4e4cc51a41c25451d11d9e050a001dc>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41954943178>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.846 s | 248 | 13.1 | 1.98× (1.93–2.02) | 4.53× (4.42–4.65) | 4.68× (4.57–4.80) | 5.06× (4.92–5.21) | 4.55× (4.42–4.68) | 4.72× (4.54–4.91) | 4.58× (4.43–4.73) | 1.50× (1.46–1.54) | 1.69× (1.63–1.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.538 s | 390 | 14.7 | 1.70× (1.64–1.76) | 3.68× (3.50–3.86) | 4.13× (3.97–4.30) | 4.10× (3.89–4.32) | 3.67× (3.54–3.80) | 7.14× (6.86–7.44) | 6.93× (6.70–7.16) | 2.27× (2.21–2.34) | 2.55× (2.45–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.419 s | 500 | 21.4 | 1.82× (1.77–1.86) | 2.99× (2.80–3.19) | 3.50× (3.38–3.63) | 3.17× (3.05–3.30) | 3.40× (3.12–3.70) | 9.25× (8.95–9.57) | 8.97× (8.65–9.31) | 2.95× (2.82–3.08) | 3.31× (3.18–3.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 920 | 3.9 | 0.58× (0.58–0.59) | 3.62× (3.43–3.81) | 4.53× (4.39–4.67) | 3.43× (3.38–3.48) | 6.00× (5.89–6.11) | 2.71× (2.66–2.76) | 2.54× (2.50–2.58) | 1.04× (1.02–1.06) | 1.44× (1.42–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 893 | 4.0 | 0.55× (0.53–0.56) | 2.74× (2.65–2.82) | 5.83× (5.65–6.02) | 3.29× (3.20–3.38) | 5.72× (5.57–5.87) | 2.58× (2.50–2.67) | 2.42× (2.35–2.50) | 0.99× (0.96–1.03) | 1.37× (1.33–1.42) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 901 | 3.9 | 0.56× (0.54–0.57) | 2.80× (2.74–2.86) | 5.94× (5.82–6.07) | 3.35× (3.28–3.42) | 5.89× (5.77–6.02) | 2.63× (2.57–2.69) | 2.46× (2.41–2.52) | 1.01× (0.99–1.03) | 1.40× (1.37–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.044 s | 448 | 7.6 | 2.04× (1.98–2.10) | 2.33× (2.25–2.42) | 2.98× (2.88–3.09) | 3.05× (2.95–3.16) | 3.28× (3.15–3.41) | 1.93× (1.86–2.00) | 1.64× (1.58–1.69) | 1.59× (1.53–1.65) | 2.32× (2.24–2.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.029 s | 674 | 9.0 | 1.78× (1.63–1.93) | 2.24× (2.02–2.48) | 3.23× (2.99–3.49) | 2.66× (2.50–2.84) | 3.83× (3.56–4.12) | 2.89× (2.71–3.09) | 2.45× (2.30–2.62) | 2.38× (2.24–2.54) | 3.48× (3.26–3.71) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 s | 678 | 17.2 | 1.67× (1.62–1.71) | 1.76× (1.69–1.83) | 2.65× (2.51–2.80) | 1.92× (1.87–1.99) | 3.54× (3.45–3.64) | 2.95× (2.87–3.03) | 2.50× (2.44–2.57) | 2.43× (2.38–2.48) | 3.55× (3.45–3.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.160 s | 50 | 65.8 | 1.14× (1.13–1.15) | 2.28× (2.24–2.31) | 2.05× (2.02–2.09) | 2.08× (2.05–2.11) | 2.41× (2.36–2.46) | 2.58× (2.55–2.61) | 2.66× (2.63–2.69) | 1.06× (1.05–1.07) | 1.21× (1.19–1.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.097 s | 83 | 72.2 | 1.09× (1.05–1.14) | 2.14× (2.06–2.22) | 2.05× (1.98–2.13) | 2.61× (2.52–2.71) | 2.52× (2.44–2.61) | 4.21× (4.07–4.36) | 4.34× (4.18–4.51) | 1.73× (1.67–1.78) | 1.97× (1.90–2.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.077 s | 103 | 93.9 | 1.21× (1.16–1.27) | 1.97× (1.92–2.02) | 1.95× (1.84–2.07) | 2.32× (2.27–2.36) | 2.60× (2.53–2.67) | 5.27× (5.16–5.38) | 5.43× (5.32–5.55) | 2.16× (2.12–2.21) | 2.46× (2.40–2.53) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.211 s | 46 | 72.0 | 1.03× (0.99–1.06) | 2.13× (2.05–2.21) | 1.79× (1.73–1.86) | 1.86× (1.80–1.93) | 2.20× (2.11–2.28) | 2.60× (2.51–2.68) | 2.78× (2.69–2.87) | 1.04× (1.01–1.08) | 1.21× (1.17–1.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.121 s | 79 | 78.3 | 1.00× (0.98–1.01) | 2.07× (2.01–2.13) | 1.87× (1.83–1.91) | 2.63× (2.55–2.72) | 2.41× (2.23–2.60) | 4.55× (4.47–4.62) | 4.87× (4.81–4.93) | 1.83× (1.80–1.86) | 2.13× (2.10–2.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.097 s | 99 | 104.5 | 1.13× (1.10–1.15) | 2.02× (1.80–2.26) | 2.01× (1.73–2.33) | 2.56× (2.38–2.77) | 2.46× (2.32–2.60) | 5.70× (5.55–5.86) | 6.11× (5.97–6.25) | 2.29× (2.24–2.35) | 2.67× (2.60–2.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 685 | 5.5 | 0.64× (0.63–0.66) | 3.31× (3.08–3.55) | 4.13× (4.07–4.18) | 2.98× (2.92–3.03) | 6.64× (6.52–6.77) | 2.33× (2.31–2.36) | 2.21× (2.18–2.23) | 0.96× (0.95–0.98) | 1.43× (1.41–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 631 | 6.0 | 0.53× (0.52–0.54) | 2.61× (2.56–2.67) | 5.03× (4.87–5.18) | 2.76× (2.71–2.80) | 6.15× (6.04–6.27) | 2.14× (2.10–2.19) | 2.03× (1.99–2.06) | 0.88× (0.86–0.91) | 1.31× (1.29–1.34) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 633 | 5.8 | 0.54× (0.53–0.55) | 2.65× (2.62–2.68) | 5.14× (5.07–5.20) | 2.80× (2.75–2.85) | 6.16× (6.03–6.29) | 2.16× (2.14–2.18) | 2.04× (2.02–2.07) | 0.89× (0.87–0.91) | 1.32× (1.31–1.34) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 0.092 s | 134 | 24.8 | 1.44× (1.43–1.46) | 2.91× (2.84–2.97) | 3.19× (3.12–3.27) | 3.12× (3.07–3.17) | 3.33× (3.27–3.39) | 3.42× (3.30–3.55) | 3.35× (3.31–3.38) | 1.12× (1.10–1.15) | 1.32× (1.30–1.35) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 0.059 s | 211 | 28.7 | 1.28× (1.22–1.33) | 2.69× (2.50–2.90) | 3.20× (3.05–3.35) | 3.32× (3.14–3.51) | 3.42× (3.26–3.58) | 5.40× (5.17–5.65) | 5.28× (5.10–5.47) | 1.77× (1.72–1.82) | 2.09× (2.02–2.16) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 0.047 s | 262 | 39.9 | 1.32× (1.28–1.37) | 2.45× (2.20–2.72) | 2.96× (2.76–3.18) | 3.07× (2.90–3.25) | 3.56× (3.45–3.66) | 6.64× (6.37–6.93) | 6.49× (6.28–6.70) | 2.18× (2.11–2.25) | 2.57× (2.50–2.64) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.97× (1.93–2.02) | 4.52× (4.41–4.64) | 4.67× (4.55–4.78) | 5.05× (4.91–5.20) | 4.54× (4.41–4.67) | 4.71× (4.53–4.90) | 4.57× (4.42–4.72) | 1.50× (1.46–1.54) | 1.68× (1.62–1.74) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.84× (1.78–1.90) | 4.18× (3.97–4.39) | 4.34× (4.19–4.50) | 4.75× (4.54–4.97) | 4.09× (3.96–4.23) | 4.21× (4.01–4.42) | 4.08× (3.91–4.27) | 1.34× (1.29–1.40) | 1.50× (1.43–1.57) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.96× (1.94–1.98) | 3.48× (3.36–3.61) | 3.65× (3.56–3.75) | 3.76× (3.65–3.87) | 3.60× (3.29–3.93) | 2.81× (2.73–2.90) | 2.72× (2.63–2.82) | 0.90× (0.85–0.94) | 1.00× (0.95–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.15× (3.10–3.20) | 3.97× (3.88–4.07) | 3.22× (3.17–3.26) | 5.56× (5.47–5.66) | 2.56× (2.52–2.60) | 2.41× (2.37–2.44) | 1.04× (1.01–1.06) | 1.04× (1.01–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.58–0.61) | 2.85× (2.77–2.94) | 5.07× (4.94–5.21) | 3.07× (2.99–3.14) | 5.24× (5.12–5.36) | 2.41× (2.34–2.48) | 2.26× (2.20–2.33) | 0.98× (0.94–1.01) | 0.98× (0.95–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.22× (3.16–3.29) | 5.23× (5.09–5.37) | 3.18× (3.12–3.25) | 5.47× (5.36–5.59) | 2.49× (2.44–2.54) | 2.34× (2.29–2.39) | 1.01× (0.99–1.03) | 1.02× (0.97–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.00× (1.94–2.06) | 2.19× (2.12–2.26) | 2.80× (2.72–2.89) | 2.96× (2.87–3.06) | 3.18× (3.07–3.31) | 1.89× (1.83–1.96) | 1.61× (1.56–1.66) | 1.57× (1.51–1.62) | 2.14× (2.07–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.88× (1.83–1.93) | 2.10× (2.04–2.16) | 2.58× (2.53–2.63) | 2.66× (2.60–2.71) | 2.81× (2.76–2.87) | 1.61× (1.57–1.65) | 1.37× (1.34–1.40) | 1.33× (1.30–1.36) | 1.81× (1.78–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.76× (1.70–1.81) | 1.50× (1.44–1.56) | 1.90× (1.83–1.98) | 1.79× (1.72–1.86) | 1.81× (1.74–1.88) | 0.86× (0.82–0.89) | 0.73× (0.70–0.76) | 0.71× (0.68–0.74) | 0.97× (0.93–1.01) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.14× (1.13–1.15) | 2.23× (2.19–2.26) | 2.00× (1.96–2.03) | 2.07× (2.03–2.10) | 2.39× (2.35–2.44) | 2.56× (2.53–2.58) | 2.64× (2.60–2.67) | 1.06× (1.05–1.07) | 1.16× (1.16–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.10× (1.09–1.11) | 2.14× (2.12–2.16) | 1.90× (1.88–1.93) | 2.52× (2.50–2.54) | 2.25× (2.22–2.29) | 2.33× (2.31–2.34) | 2.40× (2.37–2.42) | 0.96× (0.95–0.97) | 1.06× (1.05–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.31× (1.27–1.36) | 2.12× (2.09–2.15) | 1.75× (1.65–1.86) | 2.31× (2.26–2.35) | 2.26× (2.21–2.30) | 1.77× (1.75–1.79) | 1.82× (1.80–1.85) | 0.73× (0.72–0.75) | 0.81× (0.79–0.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.03× (0.99–1.06) | 2.10× (2.02–2.18) | 1.75× (1.69–1.81) | 1.85× (1.79–1.92) | 2.19× (2.10–2.27) | 2.58× (2.50–2.66) | 2.76× (2.68–2.85) | 1.04× (1.01–1.08) | 1.19× (1.15–1.23) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.01× (1.00–1.01) | 2.06× (2.03–2.10) | 1.72× (1.70–1.74) | 2.41× (2.39–2.44) | 2.15× (2.00–2.31) | 2.40× (2.39–2.41) | 2.57× (2.55–2.58) | 0.97× (0.96–0.98) | 1.10× (1.10–1.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.24× (1.23–1.26) | 2.21× (1.96–2.48) | 1.75× (1.48–2.07) | 2.33× (2.12–2.55) | 2.19× (2.05–2.34) | 1.80× (1.77–1.82) | 1.92× (1.90–1.95) | 0.73× (0.72–0.74) | 0.83× (0.82–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.70) | 2.64× (2.61–2.67) | 3.46× (3.41–3.52) | 2.72× (2.67–2.78) | 5.92× (5.81–6.04) | 2.16× (2.13–2.20) | 2.06× (2.02–2.09) | 0.97× (0.96–0.98) | 0.97× (0.95–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.61) | 2.36× (2.30–2.42) | 4.15× (4.08–4.21) | 2.56× (2.53–2.60) | 5.46× (5.36–5.56) | 1.97× (1.93–2.01) | 1.87× (1.84–1.90) | 0.88× (0.86–0.90) | 0.88× (0.86–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.61) | 2.68× (2.61–2.74) | 4.26× (4.20–4.33) | 2.67× (2.62–2.72) | 5.58× (5.46–5.71) | 2.03× (2.01–2.05) | 1.93× (1.90–1.95) | 0.91× (0.89–0.93) | 0.91× (0.89–0.93) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 1.43× (1.42–1.44) | 2.83× (2.76–2.90) | 3.11× (3.04–3.18) | 3.07× (3.02–3.12) | 3.28× (3.22–3.33) | 3.37× (3.25–3.49) | 3.29× (3.25–3.33) | 1.12× (1.10–1.14) | 1.28× (1.26–1.30) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 1.34× (1.32–1.36) | 2.70× (2.60–2.81) | 3.05× (2.90–3.21) | 3.07× (3.04–3.09) | 3.03× (2.97–3.09) | 2.92× (2.84–3.01) | 2.86× (2.83–2.89) | 0.97× (0.96–0.98) | 1.11× (1.10–1.12) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 1.54× (1.51–1.58) | 2.62× (2.43–2.82) | 2.90× (2.69–3.13) | 2.85× (2.73–2.98) | 2.96× (2.82–3.12) | 2.08× (2.00–2.17) | 2.03× (2.00–2.06) | 0.69× (0.68–0.71) | 0.79× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.31× (0.29–0.33) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.25× (12.24–12.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 46 MB | 0.40× (0.40–0.41) | 0.60× (0.52–0.69) | 0.86× (0.84–0.87) | 0.70× (0.69–0.72) | 6.78× (6.70–6.85) | 0.16× (0.16–0.16) | 0.17× (0.17–0.17) | 0.26× (0.25–0.26) | 0.13× (0.13–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 69 MB | 0.43× (0.43–0.44) | 0.75× (0.66–0.85) | 0.79× (0.78–0.80) | 0.68× (0.67–0.69) | 4.56× (4.54–4.59) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) | 0.17× (0.17–0.17) | 0.09× (0.09–0.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.83) | 0.81× (0.81–0.81) | 4.32× (4.32–4.33) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.81× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.46) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.81× (1.80–1.81) | 0.84× (0.83–0.84) | 4.46× (4.44–4.47) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.75) | 0.66× (0.65–0.67) | 8.19× (8.17–8.22) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.71× (0.67–0.76) | 1.66× (1.63–1.70) | 0.81× (0.80–0.83) | 6.00× (5.88–6.12) | 0.47× (0.47–0.48) | 0.50× (0.49–0.51) | 0.55× (0.54–0.56) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.63–0.67) | 0.75× (0.70–0.80) | 1.20× (1.17–1.23) | 0.77× (0.75–0.80) | 3.51× (3.45–3.58) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 35 MB | 0.53× (0.53–0.54) | 0.45× (0.43–0.48) | 0.99× (0.98–1.00) | 0.78× (0.76–0.80) | 3.41× (3.38–3.44) | 0.21× (0.21–0.21) | 0.22× (0.22–0.22) | 0.34× (0.33–0.34) | 0.17× (0.17–0.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.46) | 0.87× (0.85–0.89) | 0.81× (0.79–0.83) | 2.52× (2.47–2.57) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.21) | 0.84× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 38 MB | 0.50× (0.50–0.51) | 0.40× (0.37–0.44) | 0.93× (0.92–0.94) | 0.77× (0.75–0.80) | 3.19× (3.16–3.21) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.16–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 56 MB | 0.58× (0.57–0.59) | 0.49× (0.46–0.53) | 0.78× (0.77–0.80) | 0.77× (0.75–0.80) | 2.25× (2.22–2.27) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.10× (1.10–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.49) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.78× (0.77–0.78) | 5.44× (5.42–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.66× (1.65–1.66) | 0.78× (0.77–0.78) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.41× (0.41–0.41) | 1.52× (1.51–1.52) | 0.64× (0.64–0.65) | 6.84× (6.82–6.85) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 0.67× (0.66–0.67) | 0.33× (0.33–0.34) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 30 MB | 0.52× (0.51–0.54) | 0.47× (0.42–0.53) | 1.31× (1.27–1.36) | 0.79× (0.77–0.82) | 4.05× (3.93–4.17) | 0.24× (0.23–0.25) | 0.26× (0.25–0.27) | 0.40× (0.38–0.41) | 0.20× (0.19–0.20) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 49 MB | 0.55× (0.54–0.56) | 0.55× (0.51–0.59) | 1.04× (1.03–1.06) | 0.81× (0.79–0.83) | 2.67× (2.63–2.71) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 248 | 4.04 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 390 | 4.54 | 1.12× (1.08–1.16) | 76% | 1.03× (1.00–1.07) | 1.04× (1.02–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 500 | 6.61 | 1.68× (1.60–1.76) | 49% | 1.29× (1.21–1.37) | 1.66× (1.60–1.73) |

**Best:** mibz2 at 4 threads (500 MB/s); lbzip2 at 4 threads (172 MB/s); pbz2 at 4 threads (276 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.64, mibz2 4t 0.96, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.17 | 0.02 | 0.08 | 0.419 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 10.7% |
| MTF + RLE2 | 2.02 | 17.6% |
| T-vector build | 2.16 | 18.8% |
| walk (the one each build uses) | 5.50 | 48.0% |
| of which: segment assembly | 0.17 | 1.5% |
| RLE1 | 0.29 | 2.6% |
| CRC | 0.26 | 2.3% |
| **total** | 11.46 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.677 | 1.344 | (in its total) | 1.99× (1.98–1.99) |  |
| walk | 1.414 | 2.633 | (in its total) | 1.86× (1.85–1.87) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.056 | (in its total) | 1.15× (1.14–1.17) |  |
| inverse BWT, total | 2.139 | 4.032 | 3.300 | 1.89× (1.88–1.89) | 1.54× (1.53–1.55) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h59m ± 15m43s |
| 4 threads | 1h00m ± 8m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m18s ± 6m09s |
| the scanner alone (its serial part) | 16m09s ± 35.4s |
| the workers alone, over 4 | 42m18s ± 6m09s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 5.55 ns a byte of output at 4 threads (CPU, summed over them), 3.38 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,692,167 | 12,398,015 | \`ffe3c447bc03294e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
