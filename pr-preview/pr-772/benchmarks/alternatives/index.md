# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-772/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`7e0045a`](<https://github.com/lmmx/mibz2/commit/7e0045a9db6a286c288968f21c3aaf5d2208101b>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45066087218>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.511 s | 411 | 7.6 | 3.20× (3.17–3.23) | 7.12× (7.04–7.19) | 7.59× (7.51–7.67) | 8.53× (8.44–8.62) | 7.62× (7.54–7.70) | 8.19× (8.11–8.27) | 7.79× (7.70–7.87) | 2.45× (2.43–2.48) | 2.28× (2.25–2.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.278 s | 754 | 8.2 | 3.27× (3.18–3.35) | 6.92× (6.71–7.13) | 8.37× (8.11–8.64) | 8.39× (8.17–8.62) | 7.39× (7.23–7.56) | 14.86× (14.51–15.22) | 14.12× (13.80–14.45) | 4.45× (4.35–4.55) | 4.14× (4.05–4.23) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.231 s | 908 | 12.7 | 3.51× (3.46–3.56) | 6.11× (6.02–6.19) | 6.93× (6.80–7.07) | 6.94× (6.81–7.07) | 6.67× (6.57–6.78) | 17.99× (17.70–18.30) | 17.10× (16.79–17.42) | 5.39× (5.29–5.49) | 5.01× (4.92–5.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 950 | 3.5 | 0.58× (0.57–0.59) | 3.39× (3.35–3.43) | 4.42× (4.22–4.62) | 3.83× (3.77–3.88) | 6.30× (6.21–6.38) | 2.85× (2.81–2.89) | 2.84× (2.79–2.89) | 1.14× (1.12–1.15) | 0.98× (0.97–0.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 909 | 3.8 | 0.52× (0.51–0.54) | 2.78× (2.72–2.84) | 5.57× (5.39–5.75) | 3.66× (3.58–3.75) | 5.93× (5.81–6.06) | 2.71× (2.66–2.77) | 2.71× (2.65–2.76) | 1.08× (1.06–1.11) | 0.93× (0.91–0.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 883 | 3.9 | 0.53× (0.52–0.54) | 2.72× (2.67–2.78) | 5.45× (5.27–5.63) | 3.56× (3.51–3.62) | 5.81× (5.71–5.90) | 2.65× (2.61–2.70) | 2.65× (2.60–2.69) | 1.06× (1.04–1.08) | 0.91× (0.90–0.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.039 s | 507 | 6.4 | 2.88× (2.85–2.91) | 3.43× (3.37–3.48) | 3.92× (3.86–3.98) | 4.50× (4.45–4.56) | 4.85× (4.80–4.90) | 3.71× (3.66–3.76) | 3.00× (2.97–3.03) | 2.14× (2.11–2.17) | 2.11× (2.08–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 841 | 7.4 | 2.49× (2.33–2.66) | 2.98× (2.78–3.19) | 4.29× (4.00–4.59) | 4.04× (3.79–4.29) | 5.39× (5.06–5.74) | 5.91× (5.54–6.31) | 4.78× (4.49–5.10) | 3.42× (3.21–3.63) | 3.37× (3.14–3.61) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 824 | 13.6 | 2.19× (2.13–2.24) | 2.19× (2.13–2.26) | 3.21× (3.03–3.39) | 2.66× (2.52–2.81) | 4.48× (4.35–4.60) | 5.92× (5.74–6.11) | 4.79× (4.65–4.93) | 3.42× (3.31–3.54) | 3.37× (3.27–3.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.155 s | 52 | 61.0 | 1.33× (1.31–1.34) | 2.62× (2.57–2.68) | 2.29× (2.25–2.33) | 2.38× (2.34–2.42) | 2.66× (2.61–2.70) | 3.03× (2.98–3.08) | 3.07× (3.02–3.13) | 1.28× (1.26–1.30) | 1.30× (1.29–1.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.087 s | 92 | 64.2 | 1.34× (1.32–1.36) | 2.83× (2.50–3.20) | 2.49× (2.43–2.54) | 3.35× (3.17–3.53) | 2.98× (2.90–3.06) | 5.36× (5.28–5.43) | 5.43× (5.36–5.51) | 2.27× (2.24–2.30) | 2.29× (2.26–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 85.4 | 1.52× (1.47–1.57) | 2.55× (2.47–2.63) | 2.49× (2.39–2.59) | 3.07× (2.98–3.17) | 3.09× (3.00–3.18) | 6.69× (6.47–6.91) | 6.79× (6.56–7.02) | 2.83× (2.76–2.90) | 2.86× (2.77–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.170 s | 56 | 55.8 | 1.39× (1.38–1.41) | 3.05× (2.99–3.10) | 2.50× (2.47–2.53) | 2.59× (2.55–2.64) | 2.93× (2.88–2.97) | 3.74× (3.68–3.79) | 3.90× (3.86–3.95) | 1.52× (1.51–1.53) | 1.56× (1.54–1.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.095 s | 101 | 59.2 | 1.39× (1.34–1.43) | 2.96× (2.83–3.10) | 2.57× (2.47–2.67) | 3.64× (3.49–3.80) | 3.07× (2.96–3.19) | 6.52× (6.28–6.76) | 6.81× (6.55–7.07) | 2.65× (2.55–2.76) | 2.72× (2.59–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 120 | 85.1 | 1.54× (1.49–1.58) | 2.87× (2.81–2.92) | 2.48× (2.39–2.58) | 3.44× (3.34–3.56) | 3.04× (2.97–3.11) | 7.90× (7.72–8.09) | 8.25× (8.07–8.44) | 3.21× (3.15–3.29) | 3.30× (3.23–3.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 722 | 4.8 | 0.63× (0.62–0.65) | 2.95× (2.91–2.98) | 4.32× (4.26–4.38) | 3.30× (3.26–3.34) | 6.84× (6.75–6.93) | 2.44× (2.41–2.48) | 2.46× (2.43–2.49) | 1.05× (1.03–1.06) | 1.49× (1.47–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.3 | 0.52× (0.51–0.53) | 2.75× (2.72–2.79) | 4.72× (4.64–4.79) | 3.08× (3.02–3.14) | 6.32× (6.19–6.46) | 2.27× (2.23–2.32) | 2.29× (2.24–2.33) | 0.97× (0.96–0.99) | 1.38× (1.36–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 664 | 5.3 | 0.53× (0.52–0.53) | 2.73× (2.71–2.76) | 4.68× (4.64–4.72) | 3.05× (3.02–3.08) | 6.23× (6.15–6.31) | 2.25× (2.23–2.27) | 2.27× (2.25–2.29) | 0.96× (0.95–0.98) | 1.37× (1.36–1.38) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.078 s | 177 | 18.0 | 1.90× (1.89–1.91) | 4.01× (3.97–4.06) | 4.55× (4.51–4.59) | 4.72× (4.68–4.77) | 4.78× (4.66–4.91) | 5.11× (5.06–5.15) | 4.98× (4.93–5.04) | 1.59× (1.49–1.69) | 1.54× (1.52–1.56) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.048 s | 287 | 20.6 | 1.59× (1.45–1.74) | 3.44× (3.18–3.72) | 4.44× (4.09–4.83) | 4.82× (4.35–5.33) | 4.41× (4.05–4.80) | 7.83× (7.18–8.54) | 7.64× (7.01–8.33) | 2.44× (2.17–2.74) | 2.36× (2.13–2.62) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.043 s | 322 | 32.0 | 1.67× (1.63–1.71) | 2.99× (2.87–3.10) | 3.93× (3.83–4.02) | 4.27× (4.01–4.56) | 4.27× (4.17–4.38) | 9.07× (8.86–9.28) | 8.85× (8.63–9.08) | 2.82× (2.65–3.00) | 2.73× (2.65–2.82) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.20× (3.17–3.23) | 7.08× (7.01–7.16) | 7.56× (7.48–7.64) | 8.50× (8.42–8.59) | 7.60× (7.52–7.68) | 8.17× (8.09–8.25) | 7.76× (7.68–7.85) | 2.45× (2.42–2.48) | 2.27× (2.24–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.10× (3.08–3.12) | 6.96× (6.84–7.08) | 7.66× (7.60–7.72) | 8.55× (8.44–8.66) | 7.21× (7.14–7.27) | 7.63× (7.55–7.72) | 7.26× (7.16–7.36) | 2.29× (2.26–2.32) | 2.12× (2.09–2.15) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.36× (3.28–3.44) | 6.46× (6.27–6.66) | 6.65× (6.51–6.80) | 7.36× (7.20–7.51) | 6.43× (6.33–6.53) | 4.87× (4.77–4.96) | 4.63× (4.52–4.73) | 1.46× (1.43–1.49) | 1.35× (1.32–1.38) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 2.90× (2.86–2.94) | 4.03× (3.91–4.15) | 3.61× (3.56–3.66) | 5.89× (5.81–5.98) | 2.71× (2.67–2.75) | 2.70× (2.66–2.74) | 1.12× (1.11–1.14) | 0.91× (0.90–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.72× (2.67–2.77) | 4.95× (4.87–5.04) | 3.40× (3.34–3.47) | 5.45× (5.35–5.55) | 2.53× (2.48–2.57) | 2.52× (2.47–2.56) | 1.05× (1.02–1.07) | 0.85× (0.84–0.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 3.08× (3.03–3.14) | 4.88× (4.79–4.96) | 3.36× (3.31–3.41) | 5.39× (5.30–5.48) | 2.51× (2.47–2.54) | 2.50× (2.46–2.54) | 1.04× (1.02–1.06) | 0.84× (0.83–0.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.80× (2.77–2.83) | 3.17× (3.14–3.20) | 3.72× (3.69–3.75) | 4.35× (4.31–4.40) | 4.68× (4.65–4.72) | 3.59× (3.54–3.64) | 2.91× (2.88–2.94) | 2.09× (2.07–2.12) | 1.97× (1.95–1.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.55× (2.50–2.60) | 2.93× (2.86–3.00) | 3.46× (3.40–3.52) | 3.83× (3.75–3.90) | 4.14× (4.06–4.21) | 3.10× (3.03–3.17) | 2.51× (2.47–2.56) | 1.81× (1.78–1.84) | 1.70× (1.66–1.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.33× (2.26–2.41) | 1.97× (1.88–2.06) | 2.47× (2.37–2.57) | 2.44× (2.33–2.56) | 2.50× (2.40–2.60) | 1.71× (1.65–1.77) | 1.39× (1.33–1.44) | 0.99× (0.95–1.04) | 0.94× (0.90–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.31–1.34) | 2.57× (2.52–2.63) | 2.24× (2.20–2.28) | 2.36× (2.32–2.40) | 2.64× (2.59–2.68) | 3.01× (2.96–3.06) | 3.05× (3.00–3.11) | 1.28× (1.26–1.30) | 1.26× (1.25–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.29–1.32) | 2.60× (2.54–2.66) | 2.24× (2.22–2.26) | 3.07× (2.93–3.21) | 2.59× (2.53–2.65) | 2.85× (2.82–2.89) | 2.89× (2.86–2.93) | 1.21× (1.19–1.23) | 1.20× (1.18–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.56–1.66) | 2.69× (2.64–2.74) | 2.22× (2.15–2.29) | 2.95× (2.88–3.03) | 2.63× (2.58–2.68) | 2.13× (2.08–2.18) | 2.16× (2.11–2.21) | 0.90× (0.89–0.92) | 0.89× (0.87–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.39× (1.37–1.40) | 3.00× (2.94–3.05) | 2.45× (2.42–2.49) | 2.58× (2.53–2.62) | 2.91× (2.86–2.95) | 3.71× (3.65–3.76) | 3.87× (3.83–3.92) | 1.51× (1.50–1.53) | 1.52× (1.51–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.32–1.38) | 2.96× (2.91–3.00) | 2.39× (2.34–2.44) | 3.35× (3.26–3.44) | 2.76× (2.71–2.82) | 3.42× (3.36–3.49) | 3.58× (3.50–3.65) | 1.40× (1.37–1.43) | 1.41× (1.37–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.51–1.63) | 2.91× (2.83–2.99) | 2.18× (2.14–2.23) | 3.05× (3.00–3.11) | 2.62× (2.56–2.68) | 2.38× (2.33–2.44) | 2.49× (2.43–2.55) | 0.97× (0.95–0.99) | 0.98× (0.96–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.44× (2.40–2.47) | 3.53× (3.47–3.58) | 3.06× (3.02–3.10) | 6.24× (6.16–6.32) | 2.29× (2.26–2.33) | 2.31× (2.28–2.34) | 1.05× (1.03–1.06) | 0.84× (0.83–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.20× (2.16–2.23) | 4.04× (3.97–4.10) | 2.83× (2.78–2.87) | 5.59× (5.50–5.68) | 2.06× (2.03–2.10) | 2.07× (2.04–2.11) | 0.94× (0.92–0.96) | 0.76× (0.74–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.56) | 2.51× (2.47–2.56) | 4.03× (3.98–4.07) | 2.85× (2.82–2.89) | 5.60× (5.53–5.67) | 2.08× (2.06–2.10) | 2.09× (2.06–2.12) | 0.95× (0.93–0.96) | 0.76× (0.75–0.77) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.88× (1.87–1.89) | 3.91× (3.86–3.95) | 4.41× (4.38–4.44) | 4.64× (4.60–4.68) | 4.70× (4.58–4.83) | 5.02× (4.97–5.06) | 4.90× (4.85–4.95) | 1.58× (1.48–1.68) | 1.45× (1.44–1.46) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.73× (1.71–1.75) | 3.60× (3.54–3.67) | 4.17× (4.12–4.21) | 4.65× (4.58–4.71) | 4.13× (4.09–4.17) | 4.35× (4.32–4.38) | 4.25× (4.21–4.28) | 1.37× (1.28–1.46) | 1.26× (1.25–1.27) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.87× (1.84–1.90) | 3.19× (3.14–3.24) | 3.56× (3.52–3.61) | 4.00× (3.88–4.13) | 3.52× (3.49–3.56) | 2.80× (2.78–2.82) | 2.74× (2.72–2.76) | 0.88× (0.83–0.94) | 0.81× (0.81–0.82) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.26× (0.26–0.26) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.98–11.03) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.37) | 0.48× (0.41–0.57) | 0.75× (0.73–0.76) | 0.63× (0.61–0.65) | 5.94× (5.80–6.09) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.35–0.39) | 0.56× (0.48–0.66) | 0.65× (0.62–0.69) | 0.58× (0.54–0.62) | 3.84× (3.64–4.06) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.18–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.86× (4.85–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.94) | 4.87× (4.86–4.88) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.41× (0.39–0.43) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.97× (7.94–8.00) | 0.66× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.73× (0.72–0.74) | 0.74× (0.68–0.80) | 1.73× (1.71–1.75) | 0.87× (0.87–0.88) | 6.30× (6.24–6.37) | 0.52× (0.51–0.52) | 0.53× (0.53–0.54) | 0.58× (0.57–0.59) | 0.29× (0.29–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.72) | 0.71× (0.65–0.77) | 1.23× (1.18–1.28) | 0.77× (0.73–0.80) | 3.67× (3.50–3.85) | 0.29× (0.28–0.30) | 0.30× (0.28–0.32) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.14× (5.12–5.16) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.50) | 0.37× (0.35–0.40) | 0.88× (0.86–0.89) | 0.69× (0.67–0.72) | 3.04× (2.97–3.10) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.66× (0.64–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.80–0.85) | 2.56× (2.52–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.39) | 0.83× (0.80–0.86) | 0.70× (0.66–0.74) | 2.87× (2.77–2.97) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.46× (0.44–0.49) | 0.81× (0.80–0.82) | 0.84× (0.81–0.87) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.31× (5.30–5.32) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.23–0.24) | 1.87× (1.87–1.87) | 0.91× (0.90–0.91) | 6.21× (6.20–6.22) | 0.41× (0.41–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.20× (6.18–6.22) | 0.41× (0.41–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.40× (1.40–1.41) | 0.67× (0.66–0.67) | 6.41× (6.39–6.42) | 0.39× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 32 MB | 0.52× (0.51–0.54) | 0.45× (0.40–0.51) | 1.19× (1.15–1.23) | 0.79× (0.76–0.83) | 3.87× (3.73–4.00) | 0.24× (0.23–0.25) | 0.25× (0.24–0.26) | 0.38× (0.36–0.39) | 0.19× (0.18–0.19) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 51 MB | 0.55× (0.53–0.57) | 0.54× (0.49–0.60) | 1.00× (0.98–1.02) | 0.81× (0.77–0.85) | 2.57× (2.53–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 411 | 2.44 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 754 | 2.62 | 1.07× (1.06–1.08) | 91% | 1.05× (1.03–1.07) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 908 | 4.08 | 1.68× (1.65–1.70) | 55% | 1.53× (1.50–1.57) | 1.76× (1.72–1.82) |

**Best:** mibz2 at 4 threads (908 MB/s); lbzip2 at 4 threads (150 MB/s); pbz2 at 4 threads (259 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.05, lbzip2 1t 0.05, lbzip2 2t 0.21, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.68 | 0.02 | 0.08 | 0.231 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3109 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.95 | 9.5% |
| MTF + RLE2 | 1.64 | 16.4% |
| T-vector build | 1.60 | 16.0% |
| walk (the one each build uses) | 5.40 | 54.1% |
| of which: segment assembly | 0.28 | 2.8% |
| RLE1 | 0.27 | 2.7% |
| CRC | 0.13 | 1.3% |
| **total** | 9.99 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.501 | 1.306 | (in its total) | 2.60× (2.57–2.62) |  |
| walk | 1.629 | 3.093 | (in its total) | 1.89× (1.87–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.091 | 0.080 | (in its total) | 0.88× (0.87–0.89) |  |
| inverse BWT, total | 2.220 | 4.479 | 2.401 | 2.01× (1.99–2.03) | 1.08× (1.07–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m49s |
| 4 threads | 33m27s ± 4m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m27s ± 3m31s |
| the scanner alone (its serial part) | 11m21s ± 26.3s |
| the workers alone, over 4 | 24m27s ± 3m31s |

At 4 threads the workers bound it. Per byte: the scanner 6.6 ns a byte of bz2, the workers 3.21 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3108.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
