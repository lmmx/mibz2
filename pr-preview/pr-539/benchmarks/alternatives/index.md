# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-539/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`68fa367`](<https://github.com/lmmx/mibz2/commit/68fa3679c08969f84a23395793b777c45faf2f53>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44076623121>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.525 s | 400 | 8.1 | 2.94× (2.90–2.97) | 6.80× (6.68–6.93) | 6.99× (6.86–7.12) | 7.61× (7.47–7.75) | 6.84× (6.69–6.99) | 7.38× (7.30–7.46) | 6.83× (6.73–6.93) | 2.17× (2.13–2.21) | 2.48× (2.44–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.302 s | 694 | 8.9 | 2.94× (2.75–3.15) | 6.16× (5.99–6.34) | 6.98× (6.68–7.28) | 6.95× (6.74–7.17) | 6.33× (6.09–6.58) | 12.67× (12.26–13.09) | 11.72× (11.34–12.12) | 3.73× (3.62–3.84) | 4.26× (4.14–4.39) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.247 s | 850 | 13.8 | 3.05× (2.88–3.23) | 4.56× (4.46–4.65) | 5.62× (5.43–5.81) | 5.18× (5.04–5.33) | 5.23× (5.07–5.39) | 15.69× (15.45–15.93) | 14.52× (14.24–14.81) | 4.62× (4.52–4.71) | 5.28× (5.17–5.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 980 | 3.6 | 0.61× (0.58–0.63) | 3.63× (3.49–3.78) | 4.53× (4.41–4.66) | 3.56× (3.52–3.59) | 6.15× (6.09–6.20) | 2.96× (2.93–3.00) | 2.65× (2.61–2.68) | 1.02× (0.98–1.06) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 950 | 3.8 | 0.56× (0.55–0.57) | 2.92× (2.87–2.98) | 6.04× (5.86–6.23) | 3.49× (3.43–3.55) | 5.97× (5.89–6.04) | 2.89× (2.83–2.94) | 2.58× (2.53–2.63) | 1.00× (0.94–1.05) | 1.46× (1.43–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 948 | 3.7 | 0.56× (0.54–0.58) | 2.91× (2.81–3.01) | 5.90× (5.77–6.04) | 3.44× (3.33–3.56) | 5.95× (5.81–6.09) | 2.86× (2.77–2.96) | 2.55× (2.47–2.64) | 0.99× (0.93–1.05) | 1.45× (1.40–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 584 | 5.9 | 2.63× (2.60–2.65) | 3.03× (2.99–3.07) | 3.89× (3.83–3.95) | 3.82× (3.79–3.85) | 4.14× (4.09–4.18) | 2.55× (2.52–2.58) | 2.13× (2.11–2.15) | 2.04× (2.02–2.06) | 3.02× (2.98–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 865 | 7.5 | 2.20× (2.03–2.39) | 2.78× (2.51–3.07) | 4.05× (3.70–4.44) | 3.23× (2.99–3.49) | 4.71× (4.35–5.09) | 3.69× (3.43–3.97) | 3.08× (2.87–3.31) | 2.96× (2.75–3.17) | 4.37× (4.06–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 773 | 14.7 | 1.89× (1.79–1.99) | 1.98× (1.77–2.20) | 3.02× (2.89–3.15) | 2.28× (2.10–2.48) | 3.90× (3.72–4.08) | 3.36× (3.20–3.53) | 2.81× (2.67–2.95) | 2.69× (2.58–2.81) | 3.98× (3.78–4.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 53.4 | 1.39× (1.38–1.41) | 2.76× (2.74–2.78) | 2.52× (2.46–2.58) | 2.51× (2.47–2.56) | 2.92× (2.89–2.96) | 3.19× (3.16–3.22) | 3.16× (3.12–3.19) | 1.27× (1.25–1.29) | 1.42× (1.41–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 98 | 61.2 | 1.26× (1.22–1.31) | 2.62× (2.40–2.85) | 2.36× (2.28–2.44) | 3.01× (2.88–3.14) | 2.91× (2.81–3.01) | 5.03× (4.87–5.19) | 4.97× (4.82–5.13) | 2.00× (1.94–2.05) | 2.24× (2.18–2.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 81.1 | 1.35× (1.31–1.39) | 2.26× (2.07–2.46) | 2.11× (1.98–2.25) | 2.63× (2.47–2.80) | 2.88× (2.76–3.02) | 6.07× (5.95–6.19) | 6.00× (5.87–6.14) | 2.41× (2.33–2.50) | 2.71× (2.62–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 50.1 | 1.50× (1.48–1.52) | 3.08× (3.02–3.13) | 2.66× (2.57–2.75) | 2.66× (2.62–2.70) | 3.18× (3.09–3.28) | 3.68× (3.64–3.73) | 3.88× (3.85–3.91) | 1.43× (1.40–1.45) | 1.70× (1.68–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.6 | 1.38× (1.32–1.44) | 2.87× (2.60–3.16) | 2.55× (2.46–2.65) | 3.39× (3.30–3.48) | 3.08× (3.01–3.15) | 6.01× (5.83–6.19) | 6.32× (6.12–6.53) | 2.32× (2.26–2.39) | 2.78× (2.70–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 79.4 | 1.45× (1.42–1.47) | 2.44× (2.25–2.65) | 2.27× (2.15–2.40) | 3.13× (2.93–3.34) | 2.89× (2.77–3.01) | 7.03× (6.79–7.27) | 7.40× (7.18–7.63) | 2.72× (2.62–2.82) | 3.25× (3.13–3.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 5.1 | 0.67× (0.67–0.68) | 3.29× (3.04–3.56) | 4.47× (4.33–4.61) | 3.12× (3.07–3.16) | 6.84× (6.76–6.91) | 2.53× (2.49–2.58) | 2.32× (2.29–2.36) | 0.98× (0.97–1.00) | 1.52× (1.49–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.6 | 0.55× (0.54–0.56) | 2.82× (2.73–2.90) | 5.08× (4.90–5.27) | 2.94× (2.87–3.01) | 6.34× (6.17–6.52) | 2.36× (2.30–2.42) | 2.16× (2.11–2.21) | 0.91× (0.90–0.93) | 1.41× (1.37–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 694 | 5.3 | 0.57× (0.56–0.58) | 2.93× (2.78–3.08) | 5.09× (4.88–5.30) | 2.96× (2.93–3.00) | 6.45× (6.30–6.61) | 2.39× (2.37–2.42) | 2.19× (2.18–2.20) | 0.93× (0.91–0.95) | 1.43× (1.42–1.45) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 0.069 s | 182 | 18.4 | 1.97× (1.91–2.02) | 4.14× (4.03–4.24) | 4.48× (4.40–4.57) | 4.40× (4.25–4.56) | 4.64× (4.49–4.80) | 5.17× (4.94–5.42) | 4.68× (4.48–4.88) | 1.51× (1.44–1.59) | 1.82× (1.76–1.88) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 0.047 s | 267 | 22.6 | 1.61× (1.52–1.71) | 3.30× (3.13–3.47) | 3.87× (3.68–4.07) | 4.34× (4.13–4.57) | 4.23× (3.99–4.48) | 7.57× (7.19–7.97) | 6.85× (6.52–7.19) | 2.21× (2.09–2.35) | 2.67× (2.56–2.78) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 0.043 s | 287 | 34.8 | 1.50× (1.37–1.64) | 2.32× (2.19–2.46) | 3.14× (2.89–3.42) | 3.48× (3.13–3.86) | 3.53× (3.43–3.63) | 8.24× (7.98–8.51) | 7.46× (7.13–7.79) | 2.41× (2.29–2.54) | 2.91× (2.81–3.01) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.90–2.97) | 6.77× (6.65–6.89) | 6.96× (6.83–7.08) | 7.59× (7.45–7.73) | 6.81× (6.67–6.96) | 7.35× (7.27–7.43) | 6.80× (6.71–6.90) | 2.17× (2.13–2.20) | 2.47× (2.43–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.82× (2.76–2.88) | 6.32× (6.25–6.40) | 6.64× (6.51–6.77) | 7.28× (7.20–7.36) | 6.33× (6.12–6.54) | 6.71× (6.64–6.78) | 6.21× (6.13–6.29) | 1.98× (1.94–2.01) | 2.25× (2.21–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.86–3.08) | 4.96× (4.82–5.09) | 5.30× (5.12–5.48) | 5.60× (5.42–5.79) | 5.04× (4.87–5.21) | 4.39× (4.31–4.46) | 4.06× (3.97–4.15) | 1.29× (1.26–1.33) | 1.47× (1.44–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.62–0.68) | 3.28× (3.21–3.36) | 4.11× (4.02–4.20) | 3.34× (3.30–3.38) | 5.71× (5.64–5.78) | 2.80× (2.76–2.83) | 2.51× (2.47–2.54) | 1.03× (0.99–1.06) | 1.03× (1.02–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.62) | 3.02× (2.97–3.07) | 5.32× (5.23–5.41) | 3.25× (3.20–3.30) | 5.47× (5.41–5.53) | 2.68× (2.63–2.73) | 2.40× (2.36–2.45) | 0.98× (0.93–1.04) | 0.99× (0.97–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.61) | 3.31× (3.19–3.43) | 5.30× (5.16–5.44) | 3.26× (3.16–3.37) | 5.51× (5.39–5.64) | 2.70× (2.61–2.79) | 2.42× (2.34–2.50) | 0.99× (0.93–1.05) | 1.00× (0.96–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.51–2.56) | 2.78× (2.75–2.82) | 3.51× (3.48–3.55) | 3.66× (3.63–3.69) | 3.96× (3.91–4.00) | 2.46× (2.43–2.49) | 2.07× (2.04–2.09) | 1.98× (1.97–2.00) | 2.71× (2.67–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.12–2.27) | 2.47× (2.36–2.57) | 3.03× (2.94–3.12) | 3.05× (2.96–3.15) | 3.33× (3.21–3.47) | 1.94× (1.88–2.00) | 1.63× (1.58–1.68) | 1.56× (1.52–1.61) | 2.13× (2.08–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.89–2.08) | 1.67× (1.57–1.78) | 2.13× (2.03–2.23) | 1.98× (1.88–2.08) | 2.02× (1.92–2.13) | 0.98× (0.93–1.04) | 0.82× (0.78–0.87) | 0.79× (0.76–0.83) | 1.08× (1.03–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.38–1.40) | 2.70× (2.68–2.72) | 2.45× (2.40–2.50) | 2.49× (2.45–2.53) | 2.90× (2.86–2.93) | 3.16× (3.13–3.19) | 3.13× (3.09–3.16) | 1.26× (1.25–1.28) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.45× (2.43–2.48) | 2.19× (2.16–2.21) | 2.92× (2.84–3.00) | 2.60× (2.55–2.64) | 2.78× (2.76–2.80) | 2.75× (2.72–2.77) | 1.11× (1.08–1.13) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.44–1.60) | 2.46× (2.25–2.70) | 1.97× (1.85–2.11) | 2.73× (2.54–2.93) | 2.58× (2.45–2.72) | 2.10× (2.06–2.15) | 2.08× (2.04–2.13) | 0.84× (0.81–0.87) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.47–1.51) | 3.01× (2.97–3.06) | 2.60× (2.51–2.69) | 2.64× (2.60–2.67) | 3.15× (3.06–3.25) | 3.65× (3.61–3.69) | 3.84× (3.81–3.87) | 1.42× (1.40–1.44) | 1.65× (1.64–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.41) | 2.75× (2.69–2.81) | 2.34× (2.30–2.37) | 3.19× (3.14–3.25) | 2.75× (2.72–2.79) | 3.19× (3.13–3.26) | 3.36× (3.29–3.43) | 1.24× (1.22–1.27) | 1.45× (1.42–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.63–1.71) | 2.71× (2.57–2.86) | 2.03× (1.93–2.14) | 2.95× (2.88–3.02) | 2.67× (2.55–2.79) | 2.32× (2.27–2.38) | 2.44× (2.38–2.50) | 0.90× (0.88–0.93) | 1.05× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.75× (2.69–2.81) | 3.62× (3.52–3.71) | 2.85× (2.80–2.90) | 6.10× (6.02–6.18) | 2.34× (2.29–2.39) | 2.15× (2.11–2.19) | 0.99× (0.97–1.00) | 0.97× (0.94–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.61–0.62) | 2.47× (2.41–2.53) | 4.23× (4.15–4.31) | 2.71× (2.65–2.77) | 5.59× (5.44–5.76) | 2.15× (2.09–2.20) | 1.97× (1.93–2.01) | 0.90× (0.89–0.92) | 0.89× (0.86–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.84× (2.63–3.06) | 4.34× (4.28–4.41) | 2.81× (2.77–2.85) | 5.83× (5.70–5.97) | 2.23× (2.20–2.26) | 2.05× (2.04–2.06) | 0.94× (0.92–0.95) | 0.92× (0.90–0.95) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 1.94× (1.88–1.99) | 3.98× (3.89–4.09) | 4.32× (4.24–4.40) | 4.30× (4.15–4.46) | 4.54× (4.39–4.69) | 5.05× (4.83–5.28) | 4.57× (4.38–4.77) | 1.50× (1.43–1.57) | 1.73× (1.67–1.79) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 1.67× (1.65–1.70) | 3.39× (3.33–3.45) | 3.72× (3.63–3.81) | 4.13× (3.95–4.32) | 3.85× (3.73–3.97) | 4.16× (4.07–4.25) | 3.76× (3.65–3.88) | 1.23× (1.19–1.27) | 1.42× (1.40–1.45) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 1.84× (1.67–2.04) | 2.70× (2.59–2.82) | 3.13× (2.85–3.43) | 3.18× (3.05–3.31) | 3.05× (2.94–3.16) | 2.69× (2.60–2.79) | 2.44× (2.32–2.57) | 0.80× (0.75–0.85) | 0.92× (0.89–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.30× (0.29–0.32) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.97–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.51× (0.44–0.58) | 0.70× (0.69–0.72) | 0.58× (0.57–0.60) | 5.57× (5.45–5.70) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 88 MB | 0.34× (0.33–0.36) | 0.66× (0.60–0.73) | 0.62× (0.59–0.64) | 0.53× (0.51–0.55) | 3.57× (3.44–3.71) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.15–0.17) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.05× (8.04–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.74× (0.66–0.82) | 1.61× (1.58–1.65) | 0.82× (0.79–0.84) | 5.85× (5.72–5.98) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.68) | 0.74× (0.69–0.79) | 1.18× (1.15–1.20) | 0.73× (0.70–0.75) | 3.46× (3.38–3.53) | 0.27× (0.26–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.18× (5.16–5.20) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.38× (0.35–0.42) | 0.91× (0.89–0.94) | 0.73× (0.70–0.75) | 3.16× (3.07–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.67) | 0.45× (0.44–0.46) | 0.86× (0.85–0.88) | 0.80× (0.78–0.81) | 2.51× (2.47–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.84) | 5.21× (5.18–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.37× (0.34–0.41) | 0.85× (0.81–0.88) | 0.70× (0.67–0.74) | 2.90× (2.79–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.58–0.61) | 0.49× (0.47–0.53) | 0.81× (0.79–0.82) | 0.79× (0.76–0.83) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.77–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.30× (5.28–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.66) | 6.38× (6.36–6.40) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 37 MB | 0.46× (0.44–0.47) | 0.44× (0.40–0.48) | 1.09× (1.06–1.13) | 0.69× (0.66–0.72) | 3.38× (3.26–3.49) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.32–0.35) | 0.16× (0.16–0.17) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.46× (0.42–0.50) | 0.90× (0.87–0.93) | 0.70× (0.66–0.74) | 2.28× (2.23–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 694 | 2.75 | 1.10× (1.09–1.10) | 86% | 1.02× (1.00–1.05) | 1.05× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 850 | 4.24 | 1.68× (1.64–1.71) | 53% | 1.23× (1.19–1.26) | 1.70× (1.61–1.78) |

**Best:** mibz2 at 4 threads (850 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (287 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.20, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.68 | 0.02 | 0.09 | 0.247 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 12.6% |
| MTF + RLE2 | 1.55 | 17.3% |
| T-vector build | 1.51 | 16.8% |
| walk (the one each build uses) | 4.26 | 47.4% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.99 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.305 | (in its total) | 2.65× (2.64–2.66) |  |
| walk | 1.272 | 2.640 | (in its total) | 2.07× (2.06–2.08) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.054 | (in its total) | 1.14× (1.11–1.17) |  |
| inverse BWT, total | 1.812 | 3.999 | 3.270 | 2.20× (2.19–2.21) | 1.80× (1.79–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m44s |
| 4 threads | 35m53s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m38s ± 3m29s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 24m37s ± 3m29s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.23 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,457,111 | 12,465,313 | \`f5a13800cacccde1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
