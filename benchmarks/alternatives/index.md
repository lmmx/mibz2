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

The latest record: [`39155e7`](<https://github.com/lmmx/mibz2/commit/39155e70ebada0560dfe86b7d8474d9fd1d03d88>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44098508386>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.527 s | 398 | 6.2 | 2.94× (2.86–3.02) | 6.67× (6.49–6.85) | 6.85× (6.71–6.99) | 7.36× (7.21–7.50) | 6.68× (6.56–6.80) | 7.20× (7.07–7.34) | 6.62× (6.49–6.75) | 2.12× (2.08–2.16) | 2.47× (2.42–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.298 s | 702 | 6.8 | 2.89× (2.68–3.11) | 6.19× (5.81–6.60) | 6.98× (6.53–7.46) | 6.88× (6.48–7.29) | 6.31× (5.90–6.75) | 12.36× (11.66–13.09) | 11.35× (10.70–12.05) | 3.64× (3.43–3.86) | 4.23× (4.01–4.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.251 s | 836 | 10.5 | 3.00× (2.94–3.06) | 4.65× (4.53–4.78) | 5.66× (5.50–5.83) | 5.06× (4.93–5.20) | 5.16× (5.03–5.28) | 15.34× (15.07–15.61) | 14.09× (13.84–14.35) | 4.52× (4.44–4.60) | 5.25× (5.17–5.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 971 | 2.8 | 0.60× (0.58–0.62) | 3.63× (3.46–3.81) | 4.53× (4.34–4.73) | 3.52× (3.43–3.61) | 6.08× (5.90–6.26) | 2.95× (2.87–3.03) | 2.62× (2.56–2.69) | 1.00× (0.97–1.02) | 1.48× (1.44–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 957 | 2.8 | 0.59× (0.57–0.60) | 2.96× (2.91–3.02) | 6.19× (6.05–6.34) | 3.55× (3.50–3.59) | 6.05× (5.96–6.13) | 2.95× (2.90–2.99) | 2.62× (2.58–2.67) | 1.00× (0.98–1.02) | 1.48× (1.46–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 893 | 3.0 | 0.55× (0.54–0.56) | 2.78× (2.73–2.83) | 5.72× (5.58–5.87) | 3.30× (3.25–3.35) | 5.68× (5.60–5.76) | 2.75× (2.71–2.79) | 2.45× (2.41–2.49) | 0.93× (0.92–0.94) | 1.38× (1.36–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 569 | 4.5 | 2.59× (2.54–2.63) | 2.95× (2.90–3.00) | 3.82× (3.71–3.93) | 3.75× (3.69–3.81) | 4.11× (4.04–4.17) | 2.50× (2.46–2.54) | 2.09× (2.05–2.12) | 1.99× (1.95–2.02) | 2.94× (2.89–2.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 858 | 5.7 | 2.30× (2.17–2.43) | 2.89× (2.68–3.11) | 4.19× (4.02–4.38) | 3.61× (3.21–4.06) | 4.79× (4.63–4.96) | 3.80× (3.67–3.94) | 3.18× (3.05–3.31) | 3.02× (2.90–3.14) | 4.47× (4.29–4.66) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 729 | 11.1 | 1.77× (1.70–1.84) | 1.84× (1.73–1.95) | 2.88× (2.76–3.00) | 2.05× (1.96–2.15) | 3.71× (3.58–3.84) | 3.19× (3.11–3.28) | 2.67× (2.58–2.75) | 2.54× (2.45–2.63) | 3.76× (3.63–3.89) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.136 s | 59 | 42.3 | 1.31× (1.30–1.33) | 2.64× (2.61–2.67) | 2.36× (2.34–2.39) | 2.43× (2.40–2.46) | 2.81× (2.75–2.87) | 3.06× (3.01–3.11) | 3.02× (2.99–3.05) | 1.21× (1.19–1.22) | 1.39× (1.37–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 91 | 49.4 | 1.23× (1.19–1.27) | 2.49× (2.34–2.64) | 2.27× (2.20–2.35) | 3.01× (2.71–3.34) | 2.76× (2.69–2.83) | 4.84× (4.72–4.97) | 4.77× (4.65–4.90) | 1.91× (1.85–1.97) | 2.19× (2.13–2.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 64.9 | 1.26× (1.23–1.30) | 2.14× (1.96–2.33) | 2.11× (1.91–2.32) | 2.49× (2.41–2.57) | 2.74× (2.67–2.80) | 5.78× (5.63–5.93) | 5.70× (5.57–5.83) | 2.28× (2.23–2.34) | 2.62× (2.56–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.151 s | 64 | 38.9 | 1.41× (1.40–1.43) | 2.97× (2.93–3.01) | 2.50× (2.47–2.52) | 2.60× (2.56–2.63) | 3.04× (2.86–3.23) | 3.56× (3.48–3.64) | 3.73× (3.67–3.79) | 1.40× (1.36–1.43) | 1.68× (1.65–1.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 106 | 44.1 | 1.30× (1.28–1.32) | 2.72× (2.65–2.78) | 2.45× (2.40–2.51) | 3.48× (3.27–3.70) | 2.99× (2.94–3.04) | 5.88× (5.72–6.04) | 6.17× (6.07–6.26) | 2.31× (2.26–2.36) | 2.78× (2.71–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.081 s | 119 | 62.2 | 1.33× (1.31–1.36) | 2.17× (2.11–2.23) | 2.17× (2.10–2.24) | 2.82× (2.78–2.86) | 2.69× (2.63–2.75) | 6.62× (6.51–6.74) | 6.95× (6.81–7.09) | 2.60× (2.54–2.67) | 3.13× (3.05–3.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 3.8 | 0.67× (0.66–0.68) | 3.18× (2.98–3.38) | 4.44× (4.38–4.50) | 3.15× (3.12–3.19) | 6.84× (6.70–6.98) | 2.55× (2.52–2.58) | 2.34× (2.31–2.36) | 0.99× (0.98–1.00) | 1.53× (1.51–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 697 | 4.1 | 0.57× (0.56–0.58) | 2.87× (2.82–2.91) | 4.98× (4.85–5.12) | 2.99× (2.94–3.04) | 6.45× (6.32–6.58) | 2.40× (2.37–2.44) | 2.20× (2.17–2.22) | 0.93× (0.91–0.95) | 1.44× (1.42–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 645 | 4.2 | 0.54× (0.53–0.55) | 2.67× (2.64–2.71) | 4.63× (4.50–4.77) | 2.78× (2.74–2.81) | 5.97× (5.85–6.10) | 2.23× (2.20–2.26) | 2.04× (2.02–2.07) | 0.86× (0.85–0.88) | 1.34× (1.32–1.36) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 0.070 s | 183 | 13.8 | 1.94× (1.92–1.96) | 4.09× (4.03–4.16) | 4.39× (4.33–4.45) | 4.47× (4.39–4.55) | 4.64× (4.58–4.70) | 5.18× (5.10–5.26) | 4.69× (4.62–4.75) | 1.50× (1.47–1.52) | 1.89× (1.87–1.92) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 0.050 s | 256 | 17.8 | 1.54× (1.47–1.62) | 3.24× (2.99–3.51) | 3.70× (3.51–3.91) | 4.10× (3.81–4.42) | 4.03× (3.82–4.26) | 7.14× (6.76–7.54) | 6.46× (6.14–6.80) | 2.06× (1.95–2.18) | 2.61× (2.47–2.76) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 0.044 s | 286 | 27.2 | 1.46× (1.42–1.51) | 2.45× (2.34–2.57) | 3.10× (2.93–3.28) | 3.25× (3.18–3.32) | 3.63× (3.54–3.73) | 8.10× (8.00–8.21) | 7.33× (7.25–7.42) | 2.34× (2.30–2.38) | 2.96× (2.92–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.86–3.01) | 6.64× (6.46–6.82) | 6.82× (6.68–6.95) | 7.33× (7.19–7.48) | 6.66× (6.54–6.78) | 7.18× (7.05–7.31) | 6.60× (6.47–6.72) | 2.12× (2.08–2.16) | 2.45× (2.41–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.81× (2.75–2.86) | 6.34× (6.20–6.49) | 6.58× (6.51–6.64) | 7.23× (7.12–7.34) | 6.37× (6.18–6.56) | 6.60× (6.55–6.65) | 6.07× (6.01–6.12) | 1.95× (1.93–1.97) | 2.26× (2.24–2.27) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.84–2.95) | 5.02× (4.83–5.22) | 5.30× (5.14–5.47) | 5.43× (5.26–5.59) | 4.91× (4.79–5.04) | 4.24× (4.18–4.30) | 3.90× (3.84–3.96) | 1.25× (1.23–1.27) | 1.45× (1.43–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.62–0.65) | 3.23× (3.15–3.31) | 4.05× (3.94–4.17) | 3.30× (3.22–3.38) | 5.64× (5.49–5.79) | 2.78× (2.71–2.85) | 2.48× (2.43–2.54) | 1.00× (0.98–1.02) | 1.05× (1.01–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.62–0.64) | 3.06× (3.01–3.10) | 5.39× (5.32–5.47) | 3.29× (3.25–3.33) | 5.51× (5.43–5.60) | 2.73× (2.69–2.77) | 2.44× (2.39–2.48) | 0.98× (0.96–1.00) | 1.03× (1.00–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.23× (3.19–3.27) | 5.18× (5.09–5.27) | 3.14× (3.10–3.18) | 5.28× (5.21–5.35) | 2.61× (2.57–2.65) | 2.33× (2.30–2.36) | 0.94× (0.92–0.95) | 0.99× (0.95–1.02) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.46–2.54) | 2.74× (2.70–2.78) | 3.50× (3.39–3.61) | 3.60× (3.55–3.66) | 3.94× (3.89–4.00) | 2.42× (2.38–2.46) | 2.03× (2.00–2.06) | 1.93× (1.90–1.97) | 2.67× (2.63–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.22–2.30) | 2.47× (2.41–2.54) | 3.07× (3.00–3.14) | 3.09× (3.02–3.17) | 3.32× (3.24–3.41) | 1.96× (1.92–2.01) | 1.65× (1.61–1.68) | 1.57× (1.54–1.60) | 2.16× (2.11–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.02× (1.93–2.11) | 1.70× (1.63–1.79) | 2.15× (2.07–2.25) | 1.95× (1.87–2.04) | 2.08× (1.99–2.18) | 1.00× (0.95–1.05) | 0.84× (0.80–0.88) | 0.80× (0.76–0.84) | 1.10× (1.06–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.31× (1.29–1.33) | 2.57× (2.55–2.60) | 2.31× (2.29–2.33) | 2.41× (2.38–2.44) | 2.79× (2.73–2.85) | 3.03× (2.98–3.08) | 2.99× (2.96–3.02) | 1.21× (1.19–1.22) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.20–1.25) | 2.38× (2.31–2.45) | 2.11× (2.05–2.17) | 2.78× (2.70–2.85) | 2.43× (2.37–2.49) | 2.64× (2.57–2.70) | 2.60× (2.54–2.66) | 1.05× (1.02–1.08) | 1.17× (1.14–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.35–1.45) | 2.34× (2.10–2.60) | 1.94× (1.73–2.18) | 2.52× (2.44–2.60) | 2.39× (2.33–2.45) | 1.97× (1.94–2.00) | 1.94× (1.90–1.99) | 0.78× (0.76–0.81) | 0.87× (0.85–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.39–1.42) | 2.92× (2.89–2.95) | 2.43× (2.40–2.46) | 2.58× (2.54–2.61) | 3.01× (2.84–3.20) | 3.52× (3.44–3.60) | 3.69× (3.64–3.75) | 1.39× (1.36–1.43) | 1.64× (1.61–1.66) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.29–1.32) | 2.71× (2.65–2.76) | 2.24× (2.20–2.27) | 3.12× (3.07–3.17) | 2.67× (2.64–2.70) | 3.10× (3.03–3.16) | 3.25× (3.20–3.30) | 1.22× (1.20–1.25) | 1.44× (1.41–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.52–1.58) | 2.49× (2.45–2.53) | 1.93× (1.83–2.02) | 2.71× (2.65–2.77) | 2.50× (2.42–2.59) | 2.21× (2.13–2.28) | 2.32× (2.26–2.37) | 0.87× (0.84–0.90) | 1.03× (1.00–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.73) | 2.76× (2.71–2.81) | 3.66× (3.55–3.78) | 2.88× (2.84–2.92) | 6.09× (5.97–6.22) | 2.36× (2.32–2.39) | 2.17× (2.15–2.19) | 0.99× (0.98–1.01) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.50× (2.46–2.54) | 4.29× (4.22–4.37) | 2.75× (2.71–2.79) | 5.66× (5.56–5.77) | 2.17× (2.15–2.20) | 2.00× (1.98–2.02) | 0.92× (0.90–0.93) | 0.92× (0.89–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.64× (2.60–2.68) | 4.09× (4.03–4.14) | 2.65× (2.62–2.68) | 5.42× (5.32–5.52) | 2.09× (2.06–2.11) | 1.92× (1.90–1.94) | 0.88× (0.86–0.90) | 0.88× (0.86–0.91) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 1.91× (1.90–1.93) | 3.91× (3.86–3.96) | 4.25× (4.20–4.30) | 4.37× (4.30–4.45) | 4.54× (4.48–4.59) | 5.06× (4.98–5.13) | 4.58× (4.52–4.64) | 1.48× (1.46–1.50) | 1.77× (1.75–1.79) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 1.65× (1.61–1.68) | 3.26× (3.19–3.34) | 3.54× (3.46–3.61) | 3.98× (3.88–4.07) | 3.68× (3.60–3.77) | 3.97× (3.87–4.06) | 3.59× (3.52–3.66) | 1.16× (1.14–1.19) | 1.39× (1.36–1.42) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 1.69× (1.61–1.77) | 2.71× (2.57–2.86) | 2.94× (2.78–3.11) | 3.14× (3.06–3.22) | 3.02× (2.90–3.14) | 2.58× (2.53–2.62) | 2.33× (2.30–2.37) | 0.76× (0.74–0.77) | 0.90× (0.89–0.92) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.31× (0.30–0.32) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.98–11.02) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.32–0.35) | 0.52× (0.46–0.58) | 0.71× (0.68–0.74) | 0.59× (0.56–0.61) | 5.57× (5.37–5.78) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.32–0.35) | 0.61× (0.56–0.66) | 0.60× (0.57–0.63) | 0.52× (0.50–0.55) | 3.50× (3.33–3.66) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.07) | 0.65× (0.65–0.66) | 0.69× (0.68–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.73× (0.68–0.80) | 1.63× (1.61–1.66) | 0.82× (0.80–0.84) | 5.91× (5.82–6.00) | 0.48× (0.47–0.49) | 0.50× (0.50–0.51) | 0.55× (0.54–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.67) | 0.73× (0.69–0.77) | 1.17× (1.13–1.21) | 0.71× (0.67–0.74) | 3.42× (3.30–3.54) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.20× (5.17–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.40× (0.36–0.44) | 0.91× (0.88–0.94) | 0.73× (0.71–0.76) | 3.15× (3.04–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.82× (0.80–0.84) | 2.51× (2.45–2.58) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.19× (5.18–5.20) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.39× (0.38–0.40) | 0.85× (0.82–0.87) | 0.70× (0.66–0.73) | 2.89× (2.82–2.97) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.49) | 0.80× (0.79–0.82) | 0.81× (0.78–0.83) | 2.31× (2.26–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.30–5.33) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.42) | 0.67× (0.67–0.67) | 6.40× (6.38–6.41) | 0.40× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 38 MB | 0.44× (0.42–0.45) | 0.38× (0.35–0.42) | 1.03× (1.00–1.06) | 0.66× (0.64–0.68) | 3.22× (3.13–3.32) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.51) | 0.48× (0.44–0.54) | 0.90× (0.88–0.93) | 0.72× (0.69–0.75) | 2.30× (2.23–2.38) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 398 | 2.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 702 | 2.76 | 1.09× (1.06–1.11) | 86% | 1.04× (1.02–1.05) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 836 | 4.28 | 1.69× (1.65–1.74) | 53% | 1.28× (1.23–1.33) | 1.67× (1.63–1.71) |

**Best:** mibz2 at 4 threads (836 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (283 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.16, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.51, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.70 | 0.02 | 0.09 | 0.251 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.86 | 12.5% |
| MTF + RLE2 | 1.18 | 16.9% |
| T-vector build | 1.15 | 16.5% |
| walk (the one each build uses) | 3.35 | 48.3% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.8% |
| **total** | 6.94 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.495 | 1.301 | (in its total) | 2.63× (2.63–2.63) |  |
| walk | 1.317 | 2.670 | (in its total) | 2.02× (2.01–2.04) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.057 | (in its total) | 1.15× (1.13–1.16) |  |
| inverse BWT, total | 1.860 | 4.029 | 3.299 | 2.16× (2.15–2.17) | 1.77× (1.76–1.78) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m03s |
| 4 threads | 36m18s ± 5m07s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m26s ± 3m37s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 25m25s ± 3m37s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.33 ns a byte of output at 4 threads (CPU, summed over them), 2.09 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,501,823 | 12,718,405 | \`5e8127c5c1057fcd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
