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

The latest record: [`73d4a3c`](<https://github.com/lmmx/mibz2/commit/73d4a3c20d2d1d8e00794e8659da86d36bc9b4b8>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42796420107>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.629 s | 334 | 9.8 | 2.48× (2.44–2.52) | 5.63× (5.54–5.72) | 5.82× (5.75–5.89) | 6.25× (6.16–6.33) | 5.61× (5.54–5.69) | 6.12× (6.02–6.23) | 5.63× (5.54–5.71) | 1.83× (1.80–1.86) | 2.11× (2.07–2.16) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.359 s | 584 | 10.7 | 2.49× (2.38–2.61) | 5.24× (5.02–5.47) | 6.05× (5.87–6.23) | 5.89× (5.67–6.12) | 5.32× (5.10–5.56) | 10.61× (10.23–10.99) | 9.75× (9.41–10.10) | 3.17× (3.04–3.30) | 3.66× (3.54–3.79) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.299 s | 702 | 17.4 | 2.45× (2.41–2.48) | 3.88× (3.77–3.99) | 4.80× (4.64–4.97) | 4.37× (4.28–4.46) | 4.30× (4.22–4.38) | 12.80× (12.56–13.04) | 11.76× (11.54–11.98) | 3.83× (3.75–3.90) | 4.42× (4.32–4.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 964 | 3.7 | 0.60× (0.58–0.61) | 3.76× (3.57–3.96) | 4.47× (4.28–4.66) | 3.55× (3.48–3.63) | 6.13× (6.01–6.25) | 2.95× (2.89–3.01) | 2.64× (2.59–2.69) | 1.02× (1.00–1.05) | 1.50× (1.46–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 946 | 3.8 | 0.56× (0.55–0.57) | 2.91× (2.83–2.99) | 5.92× (5.80–6.04) | 3.47× (3.38–3.57) | 5.91× (5.78–6.05) | 2.88× (2.80–2.96) | 2.58× (2.51–2.65) | 1.00× (0.97–1.03) | 1.46× (1.42–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 932 | 3.8 | 0.56× (0.55–0.58) | 2.90× (2.86–2.94) | 6.02× (5.91–6.12) | 3.44× (3.40–3.48) | 5.90× (5.83–5.96) | 2.85× (2.82–2.89) | 2.55× (2.52–2.58) | 0.99× (0.96–1.01) | 1.44× (1.43–1.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 544 | 6.3 | 2.49× (2.44–2.53) | 2.84× (2.80–2.89) | 3.65× (3.59–3.71) | 3.60× (3.54–3.65) | 3.92× (3.86–3.99) | 2.41× (2.36–2.45) | 1.99× (1.96–2.03) | 1.93× (1.89–1.97) | 2.83× (2.78–2.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 824 | 7.8 | 2.24× (1.90–2.63) | 2.86× (2.56–3.20) | 3.92× (3.70–4.14) | 3.28× (2.98–3.62) | 4.38× (4.13–4.65) | 3.54× (3.35–3.73) | 2.92× (2.76–3.10) | 2.83× (2.67–3.00) | 4.15× (3.93–4.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 764 | 15.1 | 1.83× (1.78–1.87) | 1.99× (1.89–2.09) | 2.92× (2.77–3.08) | 2.17× (2.08–2.25) | 3.83× (3.71–3.94) | 3.33× (3.23–3.42) | 2.75× (2.68–2.82) | 2.67× (2.61–2.73) | 3.91× (3.81–4.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 54.6 | 1.38× (1.37–1.39) | 2.74× (2.70–2.79) | 2.46× (2.42–2.49) | 2.47× (2.44–2.49) | 2.87× (2.84–2.90) | 3.14× (3.12–3.17) | 3.12× (3.09–3.15) | 1.24× (1.23–1.26) | 1.44× (1.42–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 61.8 | 1.28× (1.24–1.32) | 2.54× (2.46–2.61) | 2.36× (2.29–2.43) | 3.27× (2.95–3.63) | 2.89× (2.81–2.97) | 5.01× (4.87–5.16) | 4.97× (4.80–5.13) | 1.98× (1.92–2.05) | 2.29× (2.22–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 83.4 | 1.31× (1.27–1.35) | 2.13× (2.06–2.21) | 2.12× (2.03–2.20) | 2.64× (2.50–2.78) | 2.85× (2.77–2.94) | 5.91× (5.75–6.09) | 5.86× (5.69–6.04) | 2.34× (2.26–2.42) | 2.70× (2.63–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 50.7 | 1.48× (1.47–1.49) | 3.01× (2.97–3.06) | 2.53× (2.49–2.57) | 2.59× (2.58–2.61) | 3.04× (3.01–3.07) | 3.59× (3.55–3.63) | 3.86× (3.82–3.90) | 1.45× (1.43–1.47) | 1.72× (1.69–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 110 | 56.7 | 1.41× (1.38–1.43) | 2.82× (2.75–2.89) | 2.52× (2.46–2.58) | 3.51× (3.45–3.56) | 3.12× (3.06–3.17) | 6.06× (5.94–6.18) | 6.52× (6.41–6.63) | 2.45× (2.40–2.50) | 2.90× (2.85–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 82.3 | 1.41× (1.36–1.46) | 2.25× (2.18–2.32) | 2.19× (2.12–2.26) | 2.94× (2.87–3.00) | 2.81× (2.72–2.92) | 6.74× (6.56–6.92) | 7.25× (7.06–7.45) | 2.72× (2.64–2.80) | 3.22× (3.13–3.32) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 5.2 | 0.66× (0.64–0.67) | 3.08× (2.88–3.29) | 4.31× (4.19–4.42) | 3.06× (2.99–3.13) | 6.63× (6.49–6.76) | 2.48× (2.43–2.53) | 2.26× (2.21–2.31) | 0.96× (0.95–0.98) | 1.48× (1.45–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 694 | 5.5 | 0.56× (0.54–0.57) | 2.85× (2.79–2.91) | 5.08× (4.84–5.33) | 2.99× (2.92–3.06) | 6.38× (6.23–6.53) | 2.39× (2.33–2.44) | 2.18× (2.14–2.23) | 0.93× (0.90–0.95) | 1.43× (1.40–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 5.4 | 0.55× (0.54–0.56) | 2.79× (2.73–2.85) | 4.90× (4.73–5.08) | 2.91× (2.86–2.96) | 6.23× (6.11–6.35) | 2.33× (2.29–2.38) | 2.13× (2.09–2.17) | 0.91× (0.89–0.92) | 1.40× (1.37–1.42) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.091 s | 136 | 24.3 | 1.48× (1.45–1.51) | 3.08× (3.02–3.15) | 3.34× (3.29–3.40) | 3.30× (3.26–3.34) | 3.49× (3.43–3.55) | 3.92× (3.87–3.98) | 3.56× (3.51–3.62) | 1.15× (1.13–1.18) | 1.43× (1.40–1.46) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.059 s | 209 | 29.2 | 1.24× (1.11–1.39) | 2.63× (2.33–2.96) | 2.93× (2.65–3.22) | 3.30× (3.00–3.63) | 3.22× (2.93–3.54) | 5.69× (5.18–6.25) | 5.17× (4.70–5.69) | 1.67× (1.51–1.85) | 2.07× (1.86–2.30) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.049 s | 252 | 41.9 | 1.37× (1.29–1.45) | 2.28× (2.16–2.39) | 2.76× (2.62–2.92) | 3.06× (2.98–3.14) | 3.25× (3.16–3.34) | 7.29× (7.14–7.44) | 6.62× (6.47–6.78) | 2.14× (2.11–2.17) | 2.66× (2.60–2.72) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.47× (2.43–2.51) | 5.60× (5.52–5.69) | 5.80× (5.73–5.87) | 6.23× (6.14–6.32) | 5.60× (5.53–5.67) | 6.10× (6.00–6.21) | 5.61× (5.53–5.69) | 1.83× (1.80–1.86) | 2.10× (2.06–2.14) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.39× (2.36–2.41) | 5.29× (5.23–5.36) | 5.69× (5.58–5.79) | 6.11× (6.00–6.22) | 5.27× (5.14–5.40) | 5.56× (5.48–5.64) | 5.11× (5.03–5.19) | 1.67× (1.63–1.70) | 1.91× (1.88–1.95) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.32× (2.27–2.37) | 3.99× (3.84–4.16) | 4.34× (4.15–4.53) | 4.50× (4.38–4.63) | 3.96× (3.83–4.10) | 3.42× (3.36–3.48) | 3.14× (3.08–3.20) | 1.02× (1.01–1.04) | 1.18× (1.16–1.20) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.26× (3.20–3.32) | 4.06× (3.98–4.15) | 3.32× (3.26–3.39) | 5.66× (5.57–5.76) | 2.77× (2.72–2.83) | 2.50× (2.45–2.54) | 1.02× (0.99–1.05) | 1.10× (1.06–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.00× (2.92–3.08) | 5.26× (5.17–5.36) | 3.22× (3.14–3.31) | 5.40× (5.28–5.51) | 2.66× (2.59–2.73) | 2.40× (2.33–2.46) | 0.98× (0.95–1.01) | 1.05× (1.01–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.62) | 3.37× (3.32–3.43) | 5.34× (5.27–5.41) | 3.27× (3.23–3.30) | 5.47× (5.40–5.53) | 2.69× (2.66–2.73) | 2.42× (2.40–2.45) | 0.99× (0.97–1.02) | 1.06× (1.03–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.41× (2.37–2.45) | 2.64× (2.60–2.68) | 3.35× (3.30–3.40) | 3.46× (3.41–3.51) | 3.77× (3.71–3.83) | 2.33× (2.29–2.38) | 1.94× (1.91–1.97) | 1.88× (1.84–1.92) | 2.57× (2.53–2.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.15× (2.12–2.19) | 2.31× (2.28–2.35) | 2.94× (2.85–3.04) | 2.96× (2.92–3.00) | 3.14× (3.09–3.19) | 1.87× (1.84–1.91) | 1.56× (1.53–1.58) | 1.51× (1.48–1.54) | 2.07× (2.04–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.92× (1.89–1.95) | 1.65× (1.62–1.69) | 2.09× (2.04–2.15) | 1.97× (1.91–2.02) | 1.97× (1.94–2.01) | 0.96× (0.94–0.99) | 0.80× (0.79–0.82) | 0.78× (0.76–0.80) | 1.06× (1.05–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.39) | 2.68× (2.63–2.72) | 2.40× (2.36–2.43) | 2.44× (2.42–2.47) | 2.84× (2.81–2.87) | 3.11× (3.08–3.14) | 3.09× (3.06–3.12) | 1.24× (1.22–1.26) | 1.39× (1.38–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.30) | 2.49× (2.45–2.53) | 2.21× (2.17–2.24) | 2.94× (2.86–3.01) | 2.56× (2.52–2.59) | 2.75× (2.70–2.79) | 2.72× (2.68–2.77) | 1.09× (1.07–1.11) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.45–1.55) | 2.35× (2.28–2.43) | 1.95× (1.90–2.00) | 2.60× (2.54–2.67) | 2.53× (2.43–2.63) | 2.03× (1.98–2.07) | 2.01× (1.96–2.06) | 0.81× (0.79–0.83) | 0.90× (0.88–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.46–1.49) | 2.95× (2.91–2.99) | 2.48× (2.45–2.51) | 2.58× (2.56–2.59) | 3.02× (2.99–3.05) | 3.56× (3.53–3.59) | 3.83× (3.79–3.87) | 1.45× (1.43–1.47) | 1.68× (1.67–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.39–1.41) | 2.78× (2.75–2.81) | 2.33× (2.30–2.36) | 3.20× (3.14–3.25) | 2.76× (2.73–2.80) | 3.18× (3.15–3.22) | 3.42× (3.39–3.46) | 1.29× (1.28–1.31) | 1.50× (1.49–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.52–1.64) | 2.52× (2.42–2.62) | 1.92× (1.87–1.96) | 2.77× (2.69–2.86) | 2.58× (2.46–2.70) | 2.19× (2.15–2.23) | 2.36× (2.31–2.41) | 0.89× (0.87–0.91) | 1.03× (1.01–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.72) | 2.67× (2.62–2.73) | 3.53× (3.46–3.59) | 2.79× (2.72–2.86) | 5.91× (5.78–6.04) | 2.28× (2.23–2.34) | 2.10× (2.06–2.16) | 0.97× (0.94–0.99) | 0.96× (0.92–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.51× (2.46–2.56) | 4.33× (4.26–4.41) | 2.76× (2.71–2.82) | 5.62× (5.51–5.73) | 2.17× (2.13–2.21) | 2.00× (1.96–2.03) | 0.92× (0.89–0.94) | 0.91× (0.88–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.71× (2.66–2.77) | 4.27× (4.20–4.35) | 2.76× (2.72–2.81) | 5.64× (5.54–5.74) | 2.18× (2.14–2.21) | 2.00× (1.97–2.04) | 0.92× (0.91–0.93) | 0.91× (0.87–0.95) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.47× (1.44–1.50) | 2.99× (2.94–3.04) | 3.24× (3.19–3.30) | 3.25× (3.22–3.29) | 3.44× (3.38–3.50) | 3.86× (3.81–3.91) | 3.51× (3.45–3.56) | 1.15× (1.12–1.17) | 1.35× (1.33–1.37) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.34× (1.31–1.37) | 2.66× (2.62–2.70) | 2.92× (2.83–3.01) | 3.18× (3.12–3.24) | 3.01× (2.94–3.08) | 3.20× (3.16–3.25) | 2.91× (2.86–2.97) | 0.95× (0.93–0.98) | 1.12× (1.10–1.14) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.45× (1.38–1.53) | 2.39× (2.27–2.52) | 2.61× (2.47–2.76) | 2.69× (2.61–2.78) | 2.69× (2.60–2.78) | 2.25× (2.21–2.30) | 2.05× (1.99–2.11) | 0.67× (0.66–0.69) | 0.79× (0.77–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.09× (12.07–12.11) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.51× (0.45–0.57) | 0.80× (0.79–0.81) | 0.67× (0.66–0.68) | 6.36× (6.34–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.84× (0.74–0.95) | 0.71× (0.70–0.71) | 0.61× (0.60–0.63) | 4.10× (4.08–4.12) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.03–8.05) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.67) | 0.67× (0.59–0.75) | 1.60× (1.56–1.63) | 0.80× (0.79–0.81) | 5.79× (5.68–5.89) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.75× (0.70–0.81) | 1.18× (1.15–1.21) | 0.72× (0.70–0.75) | 3.47× (3.41–3.52) | 0.27× (0.27–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.21× (5.21–5.22) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.40× (0.37–0.43) | 0.92× (0.89–0.94) | 0.71× (0.68–0.74) | 3.17× (3.09–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.64) | 0.45× (0.44–0.46) | 0.86× (0.85–0.87) | 0.81× (0.79–0.82) | 2.50× (2.46–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.37× (0.34–0.41) | 0.86× (0.83–0.89) | 0.73× (0.70–0.77) | 2.95× (2.85–3.05) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.50× (0.47–0.53) | 0.79× (0.78–0.81) | 0.82× (0.80–0.84) | 2.29× (2.26–2.33) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.21) | 1.08× (1.08–1.09) | 0.77× (0.77–0.78) | 5.40× (5.40–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.47–1.48) | 0.70× (0.69–0.70) | 6.64× (6.63–6.65) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 0.65× (0.65–0.65) | 0.33× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.46) | 0.40× (0.37–0.45) | 1.05× (1.01–1.09) | 0.67× (0.64–0.70) | 3.30× (3.19–3.41) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.51× (0.47–0.54) | 0.89× (0.88–0.91) | 0.71× (0.68–0.74) | 2.29× (2.24–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 334 | 3.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 584 | 3.28 | 1.10× (1.07–1.12) | 87% | 1.04× (1.02–1.06) | 1.06× (1.04–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 702 | 5.36 | 1.79× (1.74–1.83) | 52% | 1.27× (1.23–1.31) | 1.68× (1.65–1.70) |

**Best:** mibz2 at 4 threads (702 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (286 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.71, mibz2 4t 1.07, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.91 | 0.02 | 0.08 | 0.299 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.4% |
| MTF + RLE2 | 1.59 | 17.6% |
| T-vector build | 1.53 | 17.0% |
| walk (the one each build uses) | 4.26 | 47.2% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.29 | 3.2% |
| CRC | 0.26 | 2.8% |
| **total** | 9.04 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.491 | 1.303 | (in its total) | 2.66× (2.65–2.67) |  |
| walk | 1.262 | 2.669 | (in its total) | 2.12× (2.11–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.056 | (in its total) | 1.14× (1.10–1.18) |  |
| inverse BWT, total | 1.802 | 4.028 | 3.308 | 2.24× (2.23–2.24) | 1.84× (1.82–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h31m ± 10m46s |
| 4 threads | 43m36s ± 6m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m13s ± 4m42s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 33m13s ± 4m42s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.35 ns a byte of output at 4 threads (CPU, summed over them), 2.49 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.5 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
