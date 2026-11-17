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

The latest record: [`9ebd31f`](<https://github.com/lmmx/mibz2/commit/9ebd31fbee68a674927cf375aae55df6d10450e9>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44064278310>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.523 s | 401 | 8.1 | 2.91× (2.87–2.95) | 6.72× (6.63–6.81) | 6.87× (6.78–6.95) | 7.45× (7.34–7.55) | 6.73× (6.64–6.82) | 7.30× (7.21–7.39) | 6.84× (6.75–6.94) | 2.16× (2.12–2.19) | 2.47× (2.45–2.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.298 s | 702 | 8.9 | 2.81× (2.65–2.98) | 6.11× (5.73–6.53) | 6.93× (6.46–7.43) | 6.83× (6.42–7.26) | 6.15× (5.83–6.50) | 12.48× (11.81–13.19) | 11.71× (11.10–12.35) | 3.69× (3.49–3.90) | 4.23× (4.00–4.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 861 | 13.8 | 2.96× (2.94–2.99) | 4.56× (4.48–4.65) | 5.67× (5.55–5.79) | 5.13× (5.03–5.23) | 5.18× (5.06–5.30) | 15.75× (15.58–15.92) | 14.77× (14.63–14.92) | 4.66× (4.58–4.73) | 5.34× (5.27–5.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 983 | 3.6 | 0.60× (0.59–0.61) | 3.64× (3.50–3.79) | 4.46× (4.41–4.52) | 3.58× (3.52–3.63) | 6.18× (6.01–6.35) | 2.98× (2.93–3.03) | 2.66× (2.62–2.71) | 1.03× (1.00–1.06) | 1.53× (1.48–1.58) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 966 | 3.8 | 0.57× (0.56–0.58) | 3.00× (2.96–3.03) | 6.10× (5.89–6.31) | 3.56× (3.52–3.60) | 6.05× (5.99–6.12) | 2.96× (2.92–3.00) | 2.65× (2.61–2.68) | 1.02× (1.01–1.04) | 1.52× (1.47–1.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 946 | 3.8 | 0.58× (0.55–0.61) | 2.94× (2.89–2.98) | 5.91× (5.76–6.07) | 3.48× (3.42–3.54) | 5.98× (5.86–6.10) | 2.89× (2.85–2.94) | 2.58× (2.55–2.62) | 1.00× (0.97–1.03) | 1.48× (1.44–1.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 579 | 5.9 | 2.62× (2.59–2.64) | 3.02× (2.99–3.05) | 3.93× (3.81–4.04) | 3.79× (3.76–3.83) | 4.12× (4.07–4.16) | 2.54× (2.53–2.56) | 2.11× (2.09–2.14) | 2.07× (2.05–2.09) | 3.00× (2.97–3.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 773 | 7.6 | 2.04× (1.79–2.32) | 2.64× (2.35–2.97) | 3.55× (3.25–3.87) | 2.96× (2.69–3.25) | 4.18× (3.83–4.56) | 3.38× (3.08–3.69) | 2.80× (2.56–3.07) | 2.74× (2.51–2.99) | 3.98× (3.64–4.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 757 | 14.8 | 1.86× (1.82–1.89) | 2.02× (1.92–2.12) | 2.98× (2.82–3.16) | 2.15× (2.09–2.22) | 3.80× (3.73–3.87) | 3.33× (3.27–3.40) | 2.77× (2.70–2.83) | 2.71× (2.66–2.75) | 3.93× (3.86–4.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 61 | 54.2 | 1.36× (1.35–1.38) | 2.71× (2.69–2.74) | 2.47× (2.43–2.51) | 2.47× (2.45–2.50) | 2.86× (2.83–2.89) | 3.16× (3.13–3.19) | 3.20× (3.17–3.23) | 1.23× (1.22–1.24) | 1.39× (1.38–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 61.8 | 1.28× (1.26–1.30) | 2.50× (2.41–2.58) | 2.36× (2.33–2.40) | 2.99× (2.95–3.03) | 2.92× (2.86–2.97) | 5.08× (5.00–5.17) | 5.15× (5.06–5.24) | 1.98× (1.94–2.02) | 2.25× (2.20–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 81.7 | 1.33× (1.30–1.36) | 2.15× (2.09–2.22) | 2.11× (2.08–2.14) | 2.59× (2.55–2.63) | 2.84× (2.80–2.89) | 6.05× (5.96–6.14) | 6.13× (6.04–6.23) | 2.36× (2.32–2.39) | 2.67× (2.61–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 50.0 | 1.48× (1.47–1.50) | 3.05× (3.02–3.09) | 2.56× (2.54–2.58) | 2.63× (2.61–2.65) | 3.08× (3.00–3.17) | 3.62× (3.59–3.66) | 3.94× (3.90–3.99) | 1.41× (1.38–1.43) | 1.69× (1.67–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 110 | 57.0 | 1.39× (1.35–1.42) | 2.82× (2.75–2.89) | 2.50× (2.45–2.56) | 3.46× (3.35–3.57) | 3.06× (3.01–3.12) | 6.03× (5.92–6.15) | 6.57× (6.41–6.74) | 2.34× (2.26–2.42) | 2.82× (2.75–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 125 | 78.9 | 1.44× (1.41–1.46) | 2.30× (2.22–2.39) | 2.26× (2.19–2.34) | 2.97× (2.91–3.04) | 2.80× (2.75–2.85) | 6.95× (6.86–7.04) | 7.56× (7.41–7.72) | 2.70× (2.62–2.77) | 3.25× (3.18–3.32) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 5.0 | 0.67× (0.66–0.68) | 3.06× (3.02–3.10) | 4.47× (4.41–4.53) | 3.20× (3.10–3.29) | 6.82× (6.72–6.92) | 2.57× (2.54–2.60) | 2.37× (2.34–2.39) | 0.99× (0.97–1.01) | 1.54× (1.52–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 687 | 5.5 | 0.56× (0.54–0.57) | 2.89× (2.82–2.96) | 4.92× (4.80–5.05) | 2.98× (2.92–3.06) | 6.41× (6.24–6.59) | 2.41× (2.35–2.47) | 2.22× (2.16–2.27) | 0.93× (0.90–0.95) | 1.44× (1.41–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 702 | 5.3 | 0.56× (0.56–0.57) | 2.86× (2.80–2.93) | 4.89× (4.77–5.01) | 2.97× (2.91–3.02) | 6.37× (6.25–6.49) | 2.39× (2.33–2.45) | 2.20× (2.14–2.26) | 0.92× (0.89–0.95) | 1.43× (1.40–1.47) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 0.068 s | 183 | 18.3 | 1.96× (1.93–1.99) | 4.08× (4.02–4.15) | 4.43× (4.38–4.48) | 4.45× (4.38–4.51) | 4.71× (4.64–4.78) | 5.18× (5.10–5.26) | 4.90× (4.81–5.00) | 1.50× (1.48–1.53) | 1.82× (1.78–1.85) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 0.050 s | 248 | 23.1 | 1.44× (1.30–1.60) | 2.98× (2.71–3.28) | 3.39× (3.13–3.67) | 3.90× (3.61–4.22) | 3.74× (3.42–4.08) | 6.72× (6.15–7.34) | 6.36× (5.82–6.95) | 1.95× (1.77–2.14) | 2.35× (2.14–2.59) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 0.043 s | 291 | 36.1 | 1.50× (1.45–1.55) | 2.32× (2.24–2.42) | 3.09× (2.90–3.29) | 3.37× (3.28–3.47) | 3.58× (3.51–3.65) | 8.23× (8.00–8.47) | 7.79× (7.56–8.03) | 2.39× (2.31–2.46) | 2.88× (2.77–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.86–2.95) | 6.68× (6.59–6.78) | 6.83× (6.74–6.92) | 7.42× (7.32–7.53) | 6.70× (6.61–6.79) | 7.27× (7.18–7.36) | 6.82× (6.73–6.91) | 2.15× (2.12–2.19) | 2.46× (2.43–2.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.74–2.80) | 6.29× (6.21–6.37) | 6.60× (6.51–6.69) | 7.18× (7.11–7.25) | 6.21× (6.16–6.27) | 6.68× (6.62–6.73) | 6.27× (6.21–6.32) | 1.98× (1.95–2.01) | 2.26× (2.24–2.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.85× (2.80–2.91) | 4.80× (4.74–4.87) | 5.23× (5.14–5.32) | 5.45× (5.36–5.53) | 4.85× (4.74–4.95) | 4.30× (4.27–4.33) | 4.04× (3.99–4.08) | 1.27× (1.25–1.29) | 1.46× (1.44–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.27× (3.20–3.33) | 4.07× (4.01–4.13) | 3.34× (3.30–3.40) | 5.72× (5.56–5.88) | 2.80× (2.76–2.85) | 2.51× (2.47–2.56) | 1.03× (1.00–1.06) | 1.05× (1.02–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.09× (3.05–3.12) | 5.37× (5.30–5.45) | 3.30× (3.26–3.34) | 5.51× (5.46–5.57) | 2.74× (2.71–2.77) | 2.45× (2.42–2.49) | 1.01× (0.99–1.03) | 1.03× (1.00–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.58–0.64) | 3.37× (3.30–3.44) | 5.36× (5.27–5.45) | 3.28× (3.22–3.34) | 5.51× (5.41–5.62) | 2.72× (2.68–2.76) | 2.44× (2.40–2.47) | 1.00× (0.97–1.03) | 1.02× (0.99–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.50–2.54) | 2.78× (2.76–2.81) | 3.53× (3.43–3.64) | 3.63× (3.60–3.66) | 3.94× (3.91–3.98) | 2.46× (2.44–2.47) | 2.05× (2.03–2.07) | 2.00× (1.98–2.02) | 2.69× (2.65–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.09–2.20) | 2.41× (2.34–2.49) | 2.93× (2.87–2.99) | 3.00× (2.93–3.06) | 3.19× (3.13–3.26) | 1.91× (1.86–1.96) | 1.59× (1.56–1.62) | 1.56× (1.52–1.59) | 2.09× (2.04–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.98–2.09) | 1.68× (1.63–1.72) | 2.13× (2.07–2.19) | 2.05× (1.99–2.12) | 2.01× (1.94–2.07) | 0.99× (0.96–1.02) | 0.82× (0.80–0.85) | 0.81× (0.78–0.83) | 1.08× (1.05–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.37) | 2.65× (2.63–2.68) | 2.40× (2.36–2.45) | 2.45× (2.43–2.48) | 2.83× (2.80–2.86) | 3.12× (3.10–3.15) | 3.17× (3.13–3.20) | 1.23× (1.22–1.24) | 1.37× (1.35–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.45× (2.43–2.48) | 2.17× (2.15–2.19) | 2.87× (2.84–2.90) | 2.55× (2.52–2.58) | 2.76× (2.73–2.79) | 2.79× (2.77–2.82) | 1.08× (1.07–1.09) | 1.20× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.51× (1.48–1.55) | 2.31× (2.26–2.36) | 1.88× (1.84–1.92) | 2.57× (2.51–2.63) | 2.47× (2.41–2.53) | 2.05× (2.01–2.09) | 2.08× (2.04–2.11) | 0.80× (0.79–0.82) | 0.90× (0.88–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.46–1.49) | 2.99× (2.95–3.03) | 2.49× (2.47–2.51) | 2.61× (2.59–2.62) | 3.05× (2.97–3.14) | 3.58× (3.55–3.62) | 3.90× (3.86–3.95) | 1.40× (1.37–1.43) | 1.65× (1.63–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.39) | 2.76× (2.73–2.78) | 2.31× (2.29–2.33) | 3.17× (3.13–3.21) | 2.72× (2.69–2.75) | 3.18× (3.15–3.21) | 3.46× (3.40–3.52) | 1.24× (1.21–1.27) | 1.46× (1.44–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.58–1.69) | 2.54× (2.47–2.61) | 1.97× (1.94–2.00) | 2.81× (2.75–2.88) | 2.55× (2.50–2.59) | 2.29× (2.25–2.32) | 2.49× (2.45–2.53) | 0.89× (0.87–0.91) | 1.05× (1.03–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.73) | 2.74× (2.70–2.79) | 3.64× (3.57–3.70) | 2.91× (2.83–2.99) | 6.07× (5.99–6.15) | 2.37× (2.33–2.40) | 2.19× (2.16–2.22) | 0.99× (0.97–1.00) | 0.98× (0.95–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.64) | 2.51× (2.46–2.57) | 4.25× (4.17–4.33) | 2.75× (2.70–2.81) | 5.65× (5.52–5.78) | 2.19× (2.14–2.23) | 2.02× (1.98–2.06) | 0.91× (0.89–0.94) | 0.91× (0.89–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.77× (2.69–2.86) | 4.29× (4.19–4.38) | 2.80× (2.75–2.84) | 5.72× (5.61–5.83) | 2.21× (2.16–2.27) | 2.04× (1.99–2.10) | 0.92× (0.90–0.95) | 0.92× (0.88–0.95) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 1.93× (1.90–1.96) | 3.93× (3.88–3.98) | 4.28× (4.22–4.34) | 4.35× (4.28–4.41) | 4.59× (4.52–4.66) | 5.06× (4.98–5.14) | 4.79× (4.70–4.88) | 1.49× (1.46–1.51) | 1.75× (1.73–1.77) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 1.65× (1.61–1.69) | 3.26× (3.18–3.34) | 3.55× (3.46–3.65) | 3.91× (3.83–3.99) | 3.68× (3.60–3.77) | 3.99× (3.90–4.07) | 3.77× (3.68–3.87) | 1.17× (1.15–1.19) | 1.38× (1.35–1.40) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 1.71× (1.64–1.77) | 2.60× (2.50–2.71) | 2.90× (2.77–3.03) | 3.06× (2.98–3.15) | 2.99× (2.90–3.07) | 2.58× (2.52–2.65) | 2.44× (2.37–2.52) | 0.76× (0.74–0.78) | 0.89× (0.86–0.92) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.29× (0.28–0.31) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.98–11.04) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.35) | 0.50× (0.44–0.56) | 0.71× (0.69–0.73) | 0.59× (0.58–0.61) | 5.65× (5.51–5.80) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.33–0.35) | 0.71× (0.59–0.86) | 0.61× (0.58–0.63) | 0.52× (0.50–0.54) | 3.52× (3.39–3.66) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.07) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.65–0.67) | 0.69× (0.64–0.73) | 1.59× (1.56–1.62) | 0.80× (0.78–0.81) | 5.75× (5.67–5.84) | 0.46× (0.46–0.47) | 0.48× (0.47–0.49) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.65) | 0.76× (0.71–0.82) | 1.16× (1.13–1.19) | 0.72× (0.69–0.74) | 3.41× (3.33–3.50) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.72–0.73) | 5.16× (5.15–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.41× (0.37–0.45) | 0.93× (0.91–0.96) | 0.74× (0.71–0.77) | 3.21× (3.14–3.29) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.61–0.63) | 0.45× (0.44–0.46) | 0.85× (0.83–0.88) | 0.80× (0.78–0.83) | 2.48× (2.42–2.55) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.82) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.50) | 0.38× (0.36–0.41) | 0.87× (0.86–0.89) | 0.73× (0.70–0.75) | 2.98× (2.93–3.04) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.50× (0.47–0.53) | 0.81× (0.79–0.82) | 0.81× (0.78–0.85) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.36× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.30) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.38× (6.37–6.39) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.40× (0.39–0.41) | 1.00× (0.98–1.03) | 0.64× (0.62–0.66) | 3.11× (3.03–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 58 MB | 0.47× (0.46–0.48) | 0.45× (0.42–0.48) | 0.87× (0.85–0.89) | 0.69× (0.67–0.72) | 2.22× (2.18–2.26) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 401 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 702 | 2.73 | 1.09× (1.07–1.10) | 86% | 1.02× (1.00–1.04) | 1.04× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 861 | 4.25 | 1.69× (1.67–1.71) | 54% | 1.21× (1.19–1.23) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (861 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.18, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.69 | 0.02 | 0.08 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 12.6% |
| MTF + RLE2 | 1.55 | 17.3% |
| T-vector build | 1.51 | 16.8% |
| walk (the one each build uses) | 4.23 | 47.3% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.95 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.496 | 1.303 | (in its total) | 2.63× (2.61–2.66) |  |
| walk | 1.273 | 2.643 | (in its total) | 2.09× (2.07–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.055 | (in its total) | 1.17× (1.14–1.19) |  |
| inverse BWT, total | 1.820 | 4.002 | 3.265 | 2.21× (2.19–2.24) | 1.81× (1.78–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m49s |
| 4 threads | 35m27s ± 4m58s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m08s ± 3m32s |
| the scanner alone (its serial part) | 13m49s ± 32.9s |
| the workers alone, over 4 | 25m08s ± 3m32s |

At 4 threads the workers bound it. Per byte: the scanner 8.0 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 2.06 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,434,784 | 12,361,720 | \`c83a65c1574133bd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
