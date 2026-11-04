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

The latest record: [`5f53787`](<https://github.com/lmmx/mibz2/commit/5f537871505fb50f1c84529418b88b7c60c3d7ec>) (2026-11-04).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41928375258>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.770 s | 272 | 12.0 | 1.96× (1.92–2.01) | 4.52× (4.41–4.64) | 4.70× (4.59–4.81) | 5.02× (4.88–5.15) | 4.49× (4.32–4.67) | 4.84× (4.71–4.98) | 4.54× (4.42–4.66) | 1.47× (1.43–1.51) | 1.70× (1.66–1.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.508 s | 413 | 13.4 | 1.68× (1.59–1.77) | 3.65× (3.43–3.87) | 4.12× (3.92–4.32) | 4.06× (3.89–4.24) | 3.71× (3.58–3.86) | 7.27× (6.99–7.57) | 6.81× (6.54–7.09) | 2.21× (2.12–2.30) | 2.55× (2.45–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.409 s | 513 | 20.9 | 1.75× (1.72–1.77) | 2.73× (2.66–2.79) | 3.40× (3.33–3.47) | 3.08× (2.96–3.21) | 3.11× (3.02–3.21) | 9.10× (8.90–9.30) | 8.52× (8.36–8.69) | 2.76× (2.71–2.81) | 3.19× (3.14–3.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 946 | 3.8 | 0.58× (0.57–0.59) | 3.52× (3.36–3.69) | 4.36× (4.24–4.49) | 3.47× (3.42–3.52) | 6.12× (5.99–6.26) | 2.74× (2.70–2.78) | 2.57× (2.54–2.61) | 1.00× (0.99–1.01) | 1.46× (1.44–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 935 | 3.9 | 0.55× (0.55–0.56) | 2.88× (2.84–2.92) | 6.09× (5.97–6.22) | 3.45× (3.40–3.50) | 5.96× (5.89–6.02) | 2.71× (2.67–2.75) | 2.55× (2.51–2.58) | 0.99× (0.98–1.00) | 1.44× (1.42–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 924 | 3.9 | 0.56× (0.55–0.56) | 2.86× (2.82–2.89) | 6.03× (5.93–6.14) | 3.41× (3.37–3.45) | 5.97× (5.86–6.08) | 2.68× (2.64–2.71) | 2.52× (2.48–2.55) | 0.98× (0.95–1.00) | 1.43× (1.41–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.043 s | 452 | 7.5 | 2.04× (2.01–2.08) | 2.36× (2.31–2.40) | 3.02× (2.96–3.08) | 2.99× (2.93–3.04) | 3.35× (3.18–3.53) | 1.95× (1.91–1.98) | 1.64× (1.61–1.68) | 1.61× (1.59–1.64) | 2.34× (2.30–2.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.027 s | 723 | 9.1 | 1.84× (1.74–1.95) | 2.33× (2.16–2.50) | 3.29× (3.12–3.48) | 2.79× (2.63–2.95) | 3.96× (3.77–4.15) | 3.04× (2.92–3.18) | 2.57× (2.46–2.68) | 2.52× (2.42–2.62) | 3.66× (3.51–3.82) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.030 s | 647 | 17.3 | 1.57× (1.53–1.61) | 1.60× (1.50–1.71) | 2.57× (2.45–2.71) | 1.83× (1.80–1.87) | 3.41× (3.34–3.47) | 2.82× (2.77–2.87) | 2.38× (2.34–2.42) | 2.33× (2.29–2.38) | 3.40× (3.34–3.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.157 s | 51 | 64.7 | 1.09× (1.02–1.17) | 2.20× (2.03–2.39) | 1.99× (1.84–2.16) | 2.04× (1.90–2.19) | 2.40× (2.19–2.62) | 2.51× (2.33–2.71) | 2.57× (2.38–2.76) | 1.02× (0.95–1.10) | 1.16× (1.07–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.096 s | 84 | 71.8 | 1.04× (0.96–1.13) | 2.19× (1.97–2.44) | 1.98× (1.82–2.15) | 2.55× (2.36–2.76) | 2.45× (2.25–2.68) | 4.10× (3.78–4.45) | 4.19× (3.86–4.55) | 1.67× (1.53–1.81) | 1.89× (1.75–2.03) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 105 | 94.0 | 1.20× (1.17–1.22) | 1.93× (1.90–1.97) | 1.93× (1.87–1.99) | 2.36× (2.26–2.46) | 2.60× (2.55–2.65) | 5.36× (5.27–5.45) | 5.47× (5.38–5.57) | 2.18× (2.15–2.22) | 2.46× (2.42–2.51) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.207 s | 46 | 71.0 | 1.01× (1.01–1.02) | 2.17× (2.13–2.21) | 1.80× (1.77–1.82) | 1.87× (1.83–1.91) | 2.22× (2.14–2.30) | 2.64× (2.59–2.69) | 2.75× (2.71–2.79) | 1.01× (1.00–1.03) | 1.21× (1.19–1.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.123 s | 78 | 77.3 | 0.95× (0.92–0.99) | 2.03× (1.95–2.11) | 1.81× (1.72–1.89) | 2.53× (2.39–2.67) | 2.25× (2.18–2.33) | 4.45× (4.31–4.58) | 4.63× (4.48–4.79) | 1.71× (1.64–1.78) | 2.04× (1.97–2.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.096 s | 100 | 104.7 | 1.11× (1.08–1.14) | 1.85× (1.79–1.91) | 1.82× (1.78–1.86) | 2.38× (2.32–2.45) | 2.31× (2.24–2.37) | 5.70× (5.59–5.82) | 5.94× (5.82–6.06) | 2.19× (2.15–2.23) | 2.61× (2.56–2.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 712 | 5.3 | 0.65× (0.64–0.66) | 3.11× (2.86–3.37) | 4.25× (4.18–4.32) | 3.01× (2.97–3.05) | 6.61× (6.53–6.68) | 2.38× (2.35–2.41) | 2.24× (2.21–2.27) | 0.96× (0.96–0.97) | 1.47× (1.45–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 5.7 | 0.54× (0.53–0.55) | 2.75× (2.70–2.81) | 5.11× (4.85–5.39) | 2.89× (2.84–2.95) | 6.36× (6.25–6.47) | 2.25× (2.21–2.30) | 2.12× (2.08–2.16) | 0.91× (0.89–0.93) | 1.39× (1.37–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 5.6 | 0.54× (0.53–0.55) | 2.72× (2.69–2.75) | 4.79× (4.61–4.98) | 2.85× (2.82–2.89) | 6.24× (6.15–6.32) | 2.22× (2.19–2.24) | 2.09× (2.07–2.11) | 0.90× (0.89–0.91) | 1.37× (1.35–1.39) | yes |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 1 | 0.088 s | 140 | 23.8 | 1.40× (1.38–1.43) | 2.91× (2.88–2.95) | 3.17× (3.12–3.23) | 3.10× (3.05–3.15) | 3.38× (3.31–3.46) | 3.32× (3.29–3.36) | 3.23× (3.17–3.29) | 1.08× (1.07–1.09) | 1.33× (1.29–1.38) | yes |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 2 | 0.057 s | 218 | 28.8 | 1.19× (1.12–1.25) | 2.48× (2.32–2.65) | 2.92× (2.82–3.04) | 3.06× (2.87–3.26) | 3.20× (3.02–3.38) | 5.04× (4.77–5.34) | 4.90× (4.71–5.11) | 1.63× (1.54–1.74) | 2.02× (1.90–2.16) | yes |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 4 | 0.047 s | 262 | 41.7 | 1.24× (1.21–1.26) | 2.06× (2.00–2.12) | 2.54× (2.47–2.62) | 2.80× (2.69–2.91) | 3.11× (3.08–3.13) | 6.20× (6.15–6.25) | 6.03× (5.91–6.15) | 2.01× (1.99–2.02) | 2.49× (2.41–2.57) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.96× (1.91–2.01) | 4.51× (4.40–4.62) | 4.69× (4.58–4.80) | 5.01× (4.87–5.14) | 4.48× (4.31–4.66) | 4.83× (4.70–4.97) | 4.53× (4.41–4.65) | 1.47× (1.43–1.51) | 1.69× (1.65–1.74) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.82× (1.76–1.89) | 4.12× (3.97–4.28) | 4.35× (4.20–4.51) | 4.71× (4.56–4.87) | 4.12× (4.01–4.23) | 4.27× (4.14–4.42) | 4.00× (3.87–4.14) | 1.30× (1.25–1.34) | 1.50× (1.44–1.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.90× (1.88–1.92) | 3.24× (3.16–3.32) | 3.53× (3.41–3.64) | 3.65× (3.59–3.71) | 3.29× (3.21–3.38) | 2.79× (2.76–2.82) | 2.62× (2.59–2.64) | 0.85× (0.84–0.85) | 0.98× (0.97–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.16× (3.11–3.21) | 3.95× (3.90–4.00) | 3.25× (3.20–3.29) | 5.66× (5.53–5.79) | 2.58× (2.54–2.62) | 2.43× (2.40–2.46) | 1.00× (0.98–1.01) | 1.07× (1.01–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.98× (2.93–3.02) | 5.27× (5.20–5.33) | 3.21× (3.15–3.26) | 5.44× (5.38–5.51) | 2.52× (2.48–2.56) | 2.37× (2.34–2.41) | 0.97× (0.96–0.99) | 1.05× (0.98–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.59–0.60) | 3.32× (3.25–3.40) | 5.35× (5.24–5.47) | 3.24× (3.19–3.28) | 5.55× (5.45–5.64) | 2.54× (2.50–2.58) | 2.39× (2.36–2.43) | 0.98× (0.96–1.01) | 1.05× (1.00–1.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.00× (1.96–2.03) | 2.20× (2.16–2.24) | 2.77× (2.72–2.82) | 2.90× (2.85–2.95) | 3.25× (3.08–3.42) | 1.91× (1.87–1.94) | 1.62× (1.59–1.65) | 1.58× (1.56–1.61) | 2.15× (2.11–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.82× (1.79–1.84) | 2.03× (2.00–2.06) | 2.49× (2.45–2.54) | 2.55× (2.49–2.60) | 2.75× (2.69–2.81) | 1.58× (1.55–1.62) | 1.34× (1.31–1.37) | 1.31× (1.29–1.34) | 1.78× (1.74–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.70× (1.66–1.73) | 1.46× (1.42–1.49) | 1.85× (1.81–1.90) | 1.74× (1.68–1.80) | 1.80× (1.76–1.84) | 0.84× (0.82–0.85) | 0.71× (0.69–0.72) | 0.69× (0.68–0.71) | 0.94× (0.92–0.96) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.09× (1.01–1.17) | 2.15× (2.00–2.32) | 1.95× (1.81–2.10) | 2.03× (1.89–2.17) | 2.38× (2.18–2.60) | 2.49× (2.31–2.68) | 2.55× (2.37–2.74) | 1.02× (0.95–1.10) | 1.12× (1.04–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.09× (1.08–1.10) | 2.17× (2.11–2.23) | 1.89× (1.87–1.91) | 2.53× (2.49–2.58) | 2.25× (2.19–2.30) | 2.34× (2.31–2.36) | 2.39× (2.36–2.41) | 0.96× (0.95–0.97) | 1.05× (1.04–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.32× (1.30–1.33) | 2.05× (2.02–2.09) | 1.69× (1.67–1.71) | 2.26× (2.24–2.29) | 2.19× (2.15–2.23) | 1.76× (1.75–1.78) | 1.80× (1.78–1.82) | 0.72× (0.71–0.73) | 0.79× (0.78–0.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.01× (1.01–1.02) | 2.14× (2.09–2.18) | 1.77× (1.75–1.80) | 1.86× (1.82–1.90) | 2.20× (2.13–2.28) | 2.62× (2.58–2.67) | 2.73× (2.69–2.77) | 1.01× (1.00–1.03) | 1.18× (1.16–1.20) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.99× (0.98–0.99) | 2.07× (2.02–2.12) | 1.72× (1.69–1.75) | 2.40× (2.36–2.43) | 2.06× (2.04–2.08) | 2.42× (2.40–2.44) | 2.52× (2.51–2.53) | 0.93× (0.92–0.94) | 1.09× (1.08–1.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.22× (1.20–1.24) | 2.03× (1.98–2.09) | 1.55× (1.51–1.59) | 2.17× (2.12–2.23) | 2.02× (1.96–2.08) | 1.80× (1.77–1.83) | 1.87× (1.85–1.90) | 0.69× (0.68–0.71) | 0.81× (0.79–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.63× (2.58–2.68) | 3.44× (3.40–3.48) | 2.75× (2.71–2.79) | 5.89× (5.81–5.96) | 2.19× (2.16–2.22) | 2.08× (2.04–2.11) | 0.96× (0.96–0.97) | 0.97× (0.93–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.60–0.61) | 2.43× (2.39–2.46) | 4.27× (4.11–4.43) | 2.67× (2.63–2.72) | 5.62× (5.54–5.71) | 2.05× (2.02–2.09) | 1.95× (1.91–1.98) | 0.90× (0.88–0.92) | 0.91× (0.88–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.67× (2.62–2.71) | 4.21× (4.12–4.30) | 2.72× (2.68–2.76) | 5.65× (5.56–5.74) | 2.07× (2.04–2.10) | 1.96× (1.94–1.99) | 0.91× (0.90–0.92) | 0.91× (0.88–0.95) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 1 | 1.40× (1.37–1.42) | 2.80× (2.77–2.83) | 3.07× (3.03–3.11) | 3.06× (3.00–3.11) | 3.33× (3.26–3.41) | 3.27× (3.24–3.31) | 3.18× (3.12–3.24) | 1.08× (1.06–1.09) | 1.27× (1.25–1.29) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 2 | 1.26× (1.23–1.28) | 2.43× (2.38–2.47) | 2.72× (2.68–2.77) | 2.83× (2.78–2.88) | 2.80× (2.76–2.84) | 2.72× (2.67–2.77) | 2.64× (2.60–2.69) | 0.89× (0.88–0.91) | 1.05× (1.03–1.08) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 4 | 1.39× (1.35–1.43) | 2.08× (2.03–2.13) | 2.28× (2.23–2.33) | 2.34× (2.31–2.37) | 2.37× (2.34–2.41) | 1.86× (1.85–1.88) | 1.81× (1.78–1.85) | 0.61× (0.61–0.62) | 0.72× (0.71–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.32× (0.31–0.34) | 1.05× (1.04–1.05) | 0.83× (0.83–0.83) | 11.82× (11.80–11.84) | 0.27× (0.27–0.28) | 0.30× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.53× (0.45–0.62) | 0.80× (0.79–0.81) | 0.66× (0.65–0.67) | 6.36× (6.31–6.40) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.82× (0.75–0.90) | 0.74× (0.73–0.74) | 0.64× (0.62–0.67) | 4.28× (4.27–4.29) | 0.10× (0.10–0.10) | 0.11× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.80) | 4.28× (4.27–4.28) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.90× (0.90–0.90) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.83× (0.83–0.83) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.83× (0.83–0.83) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.43× (0.43–0.43) | 1.65× (1.65–1.66) | 0.63× (0.63–0.63) | 7.78× (7.75–7.80) | 0.62× (0.62–0.62) | 0.65× (0.64–0.66) | 0.71× (0.71–0.72) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.63× (0.61–0.64) | 0.65× (0.62–0.68) | 1.55× (1.53–1.57) | 0.76× (0.75–0.77) | 5.59× (5.53–5.65) | 0.44× (0.44–0.45) | 0.47× (0.46–0.48) | 0.51× (0.50–0.52) | 0.26× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.59× (0.58–0.61) | 0.70× (0.66–0.74) | 1.10× (1.07–1.12) | 0.70× (0.67–0.73) | 3.23× (3.17–3.28) | 0.25× (0.24–0.25) | 0.26× (0.25–0.27) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.16) | 0.71× (0.71–0.71) | 5.09× (5.08–5.09) | 0.31× (0.31–0.31) | 0.33× (0.32–0.34) | 0.51× (0.50–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.42× (0.40–0.45) | 0.95× (0.94–0.95) | 0.72× (0.69–0.75) | 3.27× (3.25–3.29) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.32) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.58–0.61) | 0.42× (0.42–0.43) | 0.81× (0.80–0.82) | 0.74× (0.72–0.77) | 2.35× (2.31–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.12× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.15–1.16) | 0.80× (0.80–0.81) | 5.09× (5.08–5.10) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.51) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 39 MB | 0.48× (0.48–0.49) | 0.39× (0.37–0.42) | 0.89× (0.89–0.90) | 0.75× (0.74–0.77) | 3.05× (3.02–3.08) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.30–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.54–0.57) | 0.46× (0.43–0.49) | 0.74× (0.73–0.76) | 0.74× (0.71–0.78) | 2.14× (2.10–2.19) | 0.13× (0.12–0.13) | 0.13× (0.13–0.13) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.09) | 0.76× (0.76–0.76) | 5.42× (5.41–5.43) | 0.35× (0.35–0.35) | 0.36× (0.35–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.20–0.21) | 1.63× (1.63–1.64) | 0.77× (0.76–0.77) | 5.38× (5.36–5.40) | 0.34× (0.34–0.35) | 0.36× (0.35–0.36) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.40× (5.39–5.41) | 0.34× (0.34–0.35) | 0.36× (0.35–0.36) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.39× (0.39–0.39) | 1.45× (1.45–1.46) | 0.62× (0.61–0.62) | 6.53× (6.50–6.55) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.64× (0.63–0.64) | 0.32× (0.32–0.32) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 2 | 34 MB | 0.46× (0.45–0.47) | 0.39× (0.35–0.43) | 1.13× (1.10–1.16) | 0.67× (0.65–0.70) | 3.51× (3.44–3.57) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.34× (0.34–0.35) | 0.17× (0.17–0.17) |
| `text` (1.65 MB → 12.3 MB, 7.5×) | 4 | 55 MB | 0.49× (0.49–0.50) | 0.50× (0.47–0.54) | 0.92× (0.91–0.93) | 0.72× (0.70–0.75) | 2.35× (2.33–2.38) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 272 | 3.68 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 413 | 4.12 | 1.13× (1.10–1.16) | 75% | 1.03× (1.02–1.05) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 513 | 6.41 | 1.73× (1.69–1.77) | 47% | 1.24× (1.20–1.28) | 1.68× (1.65–1.70) |

**Best:** mibz2 at 4 threads (513 MB/s); lbzip2 at 4 threads (189 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 0.96, lbzip2 1t 0.09, lbzip2 2t 0.31, lbzip2 4t 0.60, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.13 | 0.02 | 0.09 | 0.409 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3260 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 11.2% |
| MTF + RLE2 | 2.23 | 20.4% |
| T-vector build | 2.15 | 19.6% |
| walk (the one each build uses) | 4.82 | 43.9% |
| of which: segment assembly | 0.16 | 1.5% |
| RLE1 | 0.28 | 2.6% |
| CRC | 0.26 | 2.3% |
| **total** | 10.98 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.677 | 1.345 | (in its total) | 1.99× (1.98–1.99) |  |
| walk | 1.441 | 2.626 | (in its total) | 1.82× (1.82–1.83) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.055 | (in its total) | 1.11× (1.09–1.12) |  |
| inverse BWT, total | 2.168 | 4.026 | 3.253 | 1.86× (1.85–1.86) | 1.50× (1.50–1.50) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h53m ± 14m06s |
| 4 threads | 1h00m ± 8m31s |
| 4 threads, the sample's overlap gap as a fixed cost | 41m04s ± 5m46s |
| the scanner alone (its serial part) | 16m18s ± 39.5s |
| the workers alone, over 4 | 41m04s ± 5m46s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 5.38 ns a byte of output at 4 threads (CPU, summed over them), 3.17 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,648,377 | 12,328,175 | \`1c54305d57f7a42b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3260.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
