# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-288/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c0a3b0d`](<https://github.com/lmmx/mibz2/commit/c0a3b0d34210873b6389f8991dea2a4e94e99389>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42585739207>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.654 s | 320 | 7.7 | 2.31× (2.30–2.32) | 5.31× (5.30–5.33) | 5.47× (5.45–5.49) | 5.93× (5.91–5.95) | 5.32× (5.28–5.36) | 5.75× (5.72–5.77) | 5.34× (5.30–5.37) | 1.72× (1.72–1.73) | 1.99× (1.98–2.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.359 s | 584 | 8.2 | 2.36× (2.29–2.43) | 5.21× (5.05–5.38) | 5.81× (5.67–5.96) | 5.83× (5.74–5.92) | 5.21× (5.10–5.32) | 10.44× (10.21–10.68) | 9.70× (9.48–9.92) | 3.13× (3.07–3.20) | 3.62× (3.55–3.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.299 s | 701 | 13.1 | 2.37× (2.33–2.41) | 3.66× (3.60–3.72) | 4.50× (4.42–4.59) | 4.13× (4.09–4.18) | 4.13× (4.09–4.17) | 12.59× (12.45–12.75) | 11.70× (11.52–11.88) | 3.78× (3.73–3.82) | 4.37× (4.32–4.42) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 990 | 2.7 | 0.61× (0.59–0.63) | 3.68× (3.49–3.87) | 4.41× (4.27–4.56) | 3.55× (3.43–3.66) | 6.09× (5.91–6.27) | 2.82× (2.72–2.91) | 2.66× (2.57–2.75) | 1.00× (0.96–1.04) | 1.50× (1.45–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 976 | 2.8 | 0.57× (0.55–0.58) | 2.97× (2.92–3.03) | 5.86× (5.76–5.97) | 3.55× (3.49–3.62) | 6.03× (5.94–6.12) | 2.79× (2.74–2.84) | 2.64× (2.58–2.70) | 0.99× (0.97–1.01) | 1.49× (1.46–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 950 | 2.8 | 0.57× (0.56–0.57) | 2.96× (2.91–3.01) | 5.91× (5.80–6.03) | 3.51× (3.46–3.56) | 6.01× (5.92–6.11) | 2.77× (2.72–2.81) | 2.61× (2.57–2.66) | 0.98× (0.96–1.00) | 1.47× (1.45–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 545 | 4.7 | 2.45× (2.43–2.46) | 2.84× (2.82–2.85) | 3.64× (3.62–3.66) | 3.58× (3.55–3.60) | 3.87× (3.84–3.90) | 2.35× (2.33–2.36) | 1.98× (1.97–1.99) | 1.89× (1.88–1.90) | 2.82× (2.81–2.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 836 | 5.9 | 2.09× (2.04–2.14) | 2.63× (2.54–2.71) | 3.94× (3.77–4.11) | 3.24× (3.09–3.38) | 4.55× (4.41–4.70) | 3.59× (3.47–3.71) | 3.03× (2.93–3.13) | 2.89× (2.79–2.99) | 4.32× (4.18–4.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 745 | 11.2 | 1.79× (1.72–1.86) | 1.84× (1.70–1.99) | 2.90× (2.75–3.06) | 2.21× (2.11–2.32) | 3.72× (3.60–3.85) | 3.20× (3.09–3.31) | 2.70× (2.61–2.79) | 2.58× (2.49–2.66) | 3.85× (3.73–3.98) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 62 | 39.8 | 1.37× (1.36–1.37) | 2.78× (2.76–2.81) | 2.49× (2.46–2.53) | 2.52× (2.51–2.54) | 2.90× (2.88–2.92) | 3.16× (3.14–3.18) | 3.17× (3.15–3.18) | 1.27× (1.27–1.28) | 1.42× (1.41–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 100 | 45.3 | 1.30× (1.25–1.36) | 2.60× (2.52–2.69) | 2.42× (2.38–2.47) | 3.15× (3.03–3.27) | 2.99× (2.92–3.05) | 5.09× (5.00–5.17) | 5.10× (5.00–5.19) | 2.05× (2.02–2.08) | 2.29× (2.25–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 62.2 | 1.30× (1.26–1.34) | 2.12× (2.06–2.18) | 2.06× (1.96–2.16) | 2.58× (2.49–2.69) | 2.80× (2.72–2.87) | 5.88× (5.74–6.03) | 5.89× (5.74–6.06) | 2.37× (2.31–2.44) | 2.65× (2.58–2.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 37.2 | 1.44× (1.43–1.45) | 3.07× (3.04–3.10) | 2.60× (2.58–2.62) | 2.67× (2.64–2.69) | 3.14× (3.03–3.25) | 3.77× (3.75–3.80) | 3.83× (3.80–3.85) | 1.44× (1.42–1.45) | 1.70× (1.67–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 42.2 | 1.41× (1.27–1.57) | 2.81× (2.75–2.87) | 2.54× (2.44–2.65) | 3.52× (3.43–3.62) | 3.12× (3.07–3.17) | 6.23× (6.14–6.31) | 6.32× (6.23–6.41) | 2.37× (2.33–2.41) | 2.80× (2.75–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 60.5 | 1.36× (1.32–1.40) | 2.23× (2.15–2.32) | 2.19× (2.12–2.27) | 2.92× (2.86–2.97) | 2.76× (2.71–2.81) | 6.99× (6.85–7.14) | 7.09× (6.95–7.24) | 2.66× (2.61–2.72) | 3.14× (3.07–3.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 3.8 | 0.67× (0.66–0.67) | 3.04× (3.03–3.06) | 4.46× (4.43–4.49) | 3.13× (3.11–3.16) | 6.79× (6.73–6.85) | 2.48× (2.47–2.50) | 2.34× (2.31–2.36) | 0.97× (0.97–0.98) | 1.53× (1.52–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 697 | 4.1 | 0.56× (0.55–0.57) | 2.87× (2.84–2.91) | 4.94× (4.87–5.00) | 2.98× (2.94–3.02) | 6.40× (6.31–6.50) | 2.34× (2.30–2.37) | 2.20× (2.17–2.23) | 0.92× (0.90–0.93) | 1.44× (1.42–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 697 | 4.0 | 0.57× (0.56–0.57) | 2.88× (2.85–2.92) | 4.94× (4.87–5.01) | 2.99× (2.94–3.03) | 6.37× (6.27–6.46) | 2.34× (2.30–2.37) | 2.20× (2.17–2.23) | 0.92× (0.90–0.93) | 1.44× (1.42–1.46) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 0.090 s | 137 | 18.2 | 1.42× (1.41–1.43) | 3.05× (3.02–3.07) | 3.26× (3.21–3.30) | 3.26× (3.23–3.29) | 3.47× (3.43–3.51) | 3.72× (3.69–3.75) | 3.47× (3.41–3.53) | 1.11× (1.09–1.12) | 1.35× (1.34–1.36) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 0.057 s | 216 | 21.1 | 1.24× (1.18–1.30) | 2.62× (2.49–2.76) | 3.05× (2.93–3.18) | 3.37× (3.20–3.55) | 3.34× (3.21–3.48) | 5.77× (5.55–6.01) | 5.39× (5.19–5.61) | 1.72× (1.65–1.80) | 2.10× (2.02–2.18) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 0.047 s | 261 | 30.3 | 1.27× (1.23–1.31) | 2.03× (1.98–2.07) | 2.60× (2.51–2.70) | 2.91× (2.81–3.01) | 3.15× (3.06–3.24) | 7.03× (6.81–7.26) | 6.57× (6.34–6.80) | 2.10× (2.02–2.18) | 2.55× (2.47–2.64) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.30× (2.29–2.31) | 5.29× (5.27–5.31) | 5.45× (5.43–5.47) | 5.91× (5.89–5.93) | 5.31× (5.27–5.35) | 5.73× (5.71–5.76) | 5.32× (5.28–5.36) | 1.72× (1.71–1.73) | 1.98× (1.97–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.22–2.25) | 5.10× (5.08–5.12) | 5.32× (5.28–5.36) | 5.90× (5.81–6.00) | 5.03× (5.00–5.05) | 5.34× (5.30–5.38) | 4.96× (4.91–5.01) | 1.60× (1.59–1.61) | 1.85× (1.83–1.86) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.25× (2.22–2.28) | 3.79× (3.74–3.85) | 4.08× (4.02–4.14) | 4.31× (4.25–4.37) | 3.80× (3.75–3.85) | 3.37× (3.34–3.41) | 3.13× (3.10–3.17) | 1.01× (1.00–1.02) | 1.17× (1.15–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.63–0.67) | 3.25× (3.15–3.35) | 4.04× (3.92–4.16) | 3.32× (3.22–3.42) | 5.63× (5.49–5.79) | 2.65× (2.57–2.74) | 2.51× (2.43–2.59) | 1.00× (0.96–1.03) | 1.09× (1.03–1.15) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.06× (3.01–3.12) | 5.36× (5.27–5.44) | 3.30× (3.25–3.36) | 5.52× (5.44–5.61) | 2.59× (2.55–2.64) | 2.46× (2.40–2.51) | 0.97× (0.95–0.99) | 1.06× (1.01–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.60–0.61) | 3.39× (3.33–3.46) | 5.41× (5.34–5.49) | 3.33× (3.28–3.38) | 5.58× (5.49–5.67) | 2.62× (2.58–2.66) | 2.48× (2.44–2.53) | 0.98× (0.97–1.00) | 1.07× (1.03–1.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.37× (2.36–2.38) | 2.62× (2.61–2.63) | 3.31× (3.28–3.34) | 3.44× (3.42–3.46) | 3.72× (3.69–3.75) | 2.28× (2.26–2.29) | 1.93× (1.92–1.94) | 1.84× (1.83–1.85) | 2.53× (2.52–2.54) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.07× (2.04–2.11) | 2.35× (2.28–2.41) | 2.85× (2.79–2.91) | 2.94× (2.86–3.02) | 3.13× (3.06–3.20) | 1.83× (1.79–1.87) | 1.55× (1.52–1.59) | 1.48× (1.45–1.52) | 2.04× (1.99–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.91–2.03) | 1.66× (1.58–1.75) | 2.10× (2.03–2.18) | 1.96× (1.89–2.03) | 1.99× (1.93–2.05) | 0.96× (0.93–0.99) | 0.81× (0.79–0.84) | 0.78× (0.75–0.80) | 1.07× (1.03–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.35–1.37) | 2.70× (2.68–2.73) | 2.44× (2.41–2.46) | 2.50× (2.48–2.52) | 2.87× (2.85–2.90) | 3.13× (3.10–3.15) | 3.13× (3.12–3.15) | 1.27× (1.26–1.28) | 1.39× (1.38–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.51× (2.49–2.53) | 2.21× (2.19–2.24) | 2.95× (2.92–2.97) | 2.61× (2.57–2.65) | 2.75× (2.72–2.78) | 2.76× (2.73–2.79) | 1.12× (1.11–1.12) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.43–1.51) | 2.30× (2.24–2.36) | 1.88× (1.83–1.93) | 2.56× (2.50–2.62) | 2.42× (2.36–2.48) | 2.00× (1.95–2.05) | 2.00× (1.96–2.05) | 0.81× (0.79–0.83) | 0.89× (0.87–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.42–1.44) | 3.01× (2.99–3.03) | 2.53× (2.52–2.54) | 2.64× (2.62–2.67) | 3.11× (3.00–3.22) | 3.73× (3.71–3.76) | 3.79× (3.76–3.81) | 1.43× (1.41–1.45) | 1.67× (1.64–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.36) | 2.78× (2.75–2.81) | 2.35× (2.29–2.40) | 3.27× (3.22–3.32) | 2.78× (2.75–2.80) | 3.29× (3.26–3.32) | 3.34× (3.31–3.37) | 1.26× (1.24–1.28) | 1.47× (1.45–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.52–1.59) | 2.53× (2.48–2.58) | 1.95× (1.91–1.99) | 2.78× (2.73–2.84) | 2.51× (2.46–2.55) | 2.30× (2.27–2.34) | 2.33× (2.30–2.37) | 0.88× (0.86–0.90) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.73× (2.70–2.76) | 3.58× (3.55–3.62) | 2.86× (2.83–2.89) | 6.06× (5.98–6.14) | 2.30× (2.27–2.32) | 2.17× (2.14–2.20) | 0.98× (0.97–0.99) | 0.99× (0.96–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.50× (2.47–2.53) | 4.23× (4.18–4.28) | 2.75× (2.71–2.78) | 5.65× (5.57–5.73) | 2.13× (2.09–2.16) | 2.01× (1.98–2.03) | 0.91× (0.89–0.92) | 0.92× (0.88–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.78× (2.74–2.83) | 4.34× (4.29–4.40) | 2.83× (2.80–2.87) | 5.76× (5.68–5.84) | 2.18× (2.15–2.21) | 2.06× (2.03–2.09) | 0.93× (0.92–0.94) | 0.94× (0.92–0.97) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 1.41× (1.40–1.42) | 2.91× (2.89–2.94) | 3.18× (3.14–3.23) | 3.21× (3.19–3.24) | 3.42× (3.38–3.45) | 3.66× (3.63–3.69) | 3.42× (3.36–3.48) | 1.11× (1.09–1.12) | 1.30× (1.29–1.31) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 1.33× (1.29–1.38) | 2.65× (2.62–2.68) | 2.90× (2.87–2.93) | 3.18× (3.14–3.21) | 3.02× (2.99–3.05) | 3.17× (3.13–3.20) | 2.96× (2.91–3.01) | 0.96× (0.95–0.97) | 1.12× (1.11–1.14) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 1.47× (1.43–1.51) | 2.27× (2.23–2.30) | 2.50× (2.46–2.55) | 2.66× (2.63–2.70) | 2.57× (2.52–2.62) | 2.22× (2.19–2.25) | 2.08× (2.03–2.12) | 0.67× (0.66–0.68) | 0.79× (0.77–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.33× (0.31–0.35) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.19× (12.16–12.22) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.59× (0.54–0.64) | 0.81× (0.80–0.82) | 0.66× (0.65–0.67) | 6.38× (6.34–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.74× (0.63–0.87) | 0.70× (0.69–0.71) | 0.61× (0.60–0.62) | 4.10× (4.07–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.44× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.67) | 8.13× (8.11–8.15) | 0.64× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.71× (0.67–0.76) | 1.64× (1.61–1.66) | 0.81× (0.80–0.82) | 5.92× (5.83–6.01) | 0.47× (0.46–0.47) | 0.49× (0.48–0.50) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.65) | 0.70× (0.65–0.75) | 1.15× (1.12–1.19) | 0.74× (0.71–0.77) | 3.42× (3.31–3.53) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.74× (0.73–0.74) | 5.26× (5.25–5.26) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.42× (0.39–0.45) | 0.94× (0.92–0.97) | 0.71× (0.67–0.74) | 3.24× (3.15–3.33) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.45–0.46) | 0.87× (0.85–0.88) | 0.80× (0.78–0.83) | 2.51× (2.46–2.56) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.83× (0.83–0.84) | 5.26× (5.26–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.41) | 0.84× (0.81–0.88) | 0.68× (0.65–0.72) | 2.89× (2.78–3.01) | 0.17× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.58–0.61) | 0.49× (0.46–0.53) | 0.80× (0.79–0.82) | 0.79× (0.75–0.82) | 2.32× (2.28–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.20–0.22) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.44× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.40× (5.39–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.64–1.64) | 0.77× (0.77–0.78) | 5.41× (5.40–5.43) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.41–0.41) | 1.50× (1.50–1.50) | 0.67× (0.66–0.67) | 6.75× (6.74–6.77) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 31 MB | 0.52× (0.51–0.53) | 0.49× (0.46–0.53) | 1.25× (1.22–1.28) | 0.77× (0.72–0.81) | 3.93× (3.85–4.01) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.38× (0.37–0.39) | 0.19× (0.19–0.20) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 49 MB | 0.55× (0.54–0.56) | 0.56× (0.52–0.60) | 1.02× (1.00–1.04) | 0.81× (0.77–0.85) | 2.62× (2.56–2.67) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 320 | 3.13 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 584 | 3.37 | 1.07× (1.07–1.08) | 91% | 1.03× (1.03–1.04) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 701 | 5.36 | 1.70× (1.68–1.72) | 55% | 1.22× (1.21–1.23) | 1.66× (1.65–1.67) |

**Best:** mibz2 at 4 threads (701 MB/s); lbzip2 at 4 threads (192 MB/s); pbz2 at 4 threads (295 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.12, lbzip2 1t 0.09, lbzip2 2t 0.43, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.91 | 0.02 | 0.09 | 0.299 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 11.6% |
| MTF + RLE2 | 1.19 | 16.3% |
| T-vector build | 1.59 | 21.8% |
| walk (the one each build uses) | 3.26 | 44.5% |
| of which: segment assembly | 0.13 | 1.7% |
| RLE1 | 0.22 | 3.0% |
| CRC | 0.19 | 2.6% |
| **total** | 7.31 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.673 | 1.303 | (in its total) | 1.94× (1.94–1.94) |  |
| walk | 1.257 | 2.676 | (in its total) | 2.13× (2.12–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.056 | (in its total) | 1.14× (1.12–1.15) |  |
| inverse BWT, total | 1.978 | 4.036 | 3.314 | 2.04× (2.03–2.05) | 1.67× (1.67–1.68) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h35m ± 11m12s |
| 4 threads | 43m28s ± 6m07s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m51s ± 4m38s |
| the scanner alone (its serial part) | 15m25s ± 32.2s |
| the workers alone, over 4 | 32m51s ± 4m38s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.31 ns a byte of output at 4 threads (CPU, summed over them), 2.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,148,587 | 12,367,175 | \`be4cf6e3036d6c66…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
