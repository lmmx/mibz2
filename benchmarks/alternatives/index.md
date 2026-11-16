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

The latest record: [`c2ad538`](<https://github.com/lmmx/mibz2/commit/c2ad53877b4f5a40b1e4816f17e83c20e55f7457>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43953415134>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.530 s | 396 | 2.0 | 3.05× (3.03–3.08) | 6.65× (6.59–6.72) | 7.64× (7.58–7.69) | 7.85× (7.78–7.92) | 7.21× (7.14–7.28) | 8.03× (7.97–8.09) | 7.27× (7.14–7.40) | 2.28× (2.21–2.36) | 2.25× (2.24–2.26) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.301 s | 697 | 2.2 | 2.93× (2.85–3.02) | 6.03× (5.85–6.21) | 7.78× (7.50–8.08) | 7.29× (7.03–7.55) | 6.74× (6.41–7.10) | 13.99× (13.47–14.53) | 12.66× (12.15–13.20) | 3.98× (3.78–4.19) | 3.92× (3.77–4.07) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.258 s | 813 | 3.7 | 3.16× (3.11–3.22) | 4.60× (4.51–4.69) | 5.93× (5.82–6.04) | 5.37× (5.30–5.44) | 5.41× (5.32–5.49) | 16.34× (15.94–16.76) | 14.79× (14.32–15.28) | 4.65× (4.45–4.85) | 4.58× (4.48–4.68) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,072 | 0.8 | 0.68× (0.67–0.68) | 3.26× (3.20–3.33) | 5.89× (5.78–6.01) | 3.74× (3.68–3.81) | 6.77× (6.66–6.88) | 2.90× (2.86–2.95) | 2.84× (2.79–2.90) | 1.13× (1.11–1.16) | 1.10× (1.09–1.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,023 | 0.9 | 0.61× (0.50–0.75) | 2.90× (2.37–3.55) | 6.67× (5.46–8.15) | 3.38× (2.77–4.13) | 5.98× (4.88–7.32) | 2.57× (2.11–3.13) | 2.52× (2.07–3.08) | 1.01× (0.83–1.23) | 0.98× (0.80–1.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,026 | 0.8 | 0.65× (0.64–0.67) | 3.14× (3.08–3.20) | 7.20× (7.07–7.34) | 3.64× (3.56–3.71) | 6.53× (6.40–6.66) | 2.77× (2.72–2.83) | 2.72× (2.67–2.77) | 1.09× (1.06–1.12) | 1.06× (1.04–1.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 549 | 1.5 | 3.12× (3.09–3.16) | 3.43× (3.39–3.46) | 4.53× (4.48–4.58) | 4.25× (4.21–4.30) | 4.84× (4.80–4.88) | 3.90× (3.85–3.94) | 3.03× (3.00–3.07) | 2.18× (2.13–2.23) | 2.28× (2.26–2.31) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 895 | 1.8 | 2.56× (2.38–2.75) | 3.05× (2.78–3.35) | 4.59× (4.15–5.07) | 3.71× (3.43–4.01) | 5.36× (4.96–5.79) | 5.99× (5.54–6.47) | 4.66× (4.31–5.04) | 3.35× (3.08–3.65) | 3.51× (3.24–3.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 843 | 3.3 | 2.40× (2.28–2.52) | 2.17× (2.01–2.34) | 3.52× (3.34–3.70) | 2.55× (2.40–2.71) | 4.93× (4.27–5.70) | 5.89× (5.63–6.16) | 4.58× (4.38–4.79) | 3.30× (3.12–3.48) | 3.45× (3.30–3.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.142 s | 56 | 14.4 | 1.40× (1.38–1.42) | 2.62× (2.59–2.65) | 2.42× (2.38–2.46) | 2.46× (2.42–2.50) | 2.83× (2.80–2.86) | 3.25× (3.18–3.32) | 3.15× (3.12–3.17) | 1.21× (1.21–1.22) | 1.27× (1.26–1.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.087 s | 92 | 15.9 | 1.29× (1.25–1.34) | 2.44× (2.36–2.53) | 2.31× (2.24–2.39) | 2.98× (2.88–3.09) | 2.94× (2.82–3.07) | 5.28× (5.06–5.51) | 5.10× (4.94–5.28) | 1.97× (1.92–2.03) | 2.07× (2.00–2.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 22.7 | 1.46× (1.43–1.49) | 2.16× (2.09–2.23) | 2.09× (2.02–2.15) | 2.65× (2.56–2.75) | 2.84× (2.76–2.93) | 6.16× (5.93–6.39) | 5.96× (5.80–6.12) | 2.30× (2.24–2.36) | 2.41× (2.35–2.48) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.162 s | 59 | 13.6 | 1.45× (1.43–1.46) | 2.89× (2.86–2.93) | 2.56× (2.53–2.60) | 2.62× (2.59–2.65) | 3.02× (2.97–3.08) | 3.89× (3.81–3.97) | 3.89× (3.85–3.93) | 1.39× (1.37–1.42) | 1.44× (1.43–1.45) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.096 s | 100 | 15.0 | 1.33× (1.28–1.38) | 2.70× (2.59–2.82) | 2.45× (2.33–2.58) | 3.42× (3.26–3.58) | 3.02× (2.91–3.13) | 6.49× (6.25–6.73) | 6.49× (6.21–6.79) | 2.33× (2.23–2.42) | 2.40× (2.30–2.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.086 s | 112 | 22.2 | 1.58× (1.54–1.62) | 2.28× (2.18–2.38) | 2.19× (2.10–2.29) | 3.03× (2.93–3.12) | 2.80× (2.70–2.90) | 7.36× (7.14–7.59) | 7.37× (7.20–7.54) | 2.64× (2.56–2.73) | 2.73× (2.65–2.80) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 790 | 1.2 | 0.72× (0.71–0.74) | 3.18× (3.11–3.25) | 4.96× (4.73–5.21) | 3.22× (3.14–3.29) | 7.42× (7.29–7.55) | 2.44× (2.39–2.49) | 2.42× (2.38–2.47) | 1.05× (1.03–1.07) | 1.60× (1.56–1.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 727 | 1.3 | 0.65× (0.63–0.67) | 3.00× (2.92–3.07) | 5.86× (5.73–5.99) | 3.09× (3.03–3.17) | 6.97× (6.82–7.12) | 2.30× (2.24–2.35) | 2.28× (2.23–2.33) | 0.99× (0.96–1.01) | 1.51× (1.47–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 714 | 1.2 | 0.65× (0.62–0.68) | 2.95× (2.90–3.01) | 5.88× (5.61–6.16) | 3.03× (2.99–3.07) | 6.92× (6.81–7.03) | 2.26× (2.21–2.30) | 2.24× (2.20–2.27) | 0.97× (0.95–0.99) | 1.48× (1.45–1.51) | yes |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 1 | 0.070 s | 174 | 4.7 | 1.85× (1.84–1.87) | 3.80× (3.74–3.86) | 4.36× (4.30–4.43) | 4.43× (4.37–4.48) | 4.71× (4.66–4.76) | 4.94× (4.89–5.00) | 4.59× (4.54–4.63) | 1.45× (1.43–1.47) | 1.44× (1.43–1.45) | yes |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 2 | 0.046 s | 267 | 5.6 | 1.55× (1.46–1.65) | 3.18× (3.05–3.31) | 3.78× (3.63–3.93) | 4.55× (4.33–4.78) | 4.40× (4.28–4.52) | 7.52× (7.31–7.73) | 6.98× (6.76–7.20) | 2.21× (2.14–2.27) | 2.19× (2.13–2.25) | yes |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 4 | 0.041 s | 300 | 8.8 | 1.70× (1.67–1.74) | 2.55× (2.51–2.59) | 3.12× (2.91–3.35) | 3.78× (3.61–3.95) | 4.05× (3.99–4.11) | 8.53× (8.38–8.68) | 7.91× (7.77–8.06) | 2.50× (2.46–2.55) | 2.48× (2.45–2.52) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.04× (3.02–3.07) | 6.62× (6.56–6.69) | 7.61× (7.55–7.66) | 7.83× (7.76–7.90) | 7.19× (7.12–7.26) | 8.01× (7.95–8.06) | 7.25× (7.12–7.38) | 2.28× (2.21–2.36) | 2.24× (2.23–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.88× (2.84–2.93) | 6.30× (6.18–6.42) | 7.46× (7.26–7.66) | 7.66× (7.50–7.82) | 6.81× (6.59–7.04) | 7.43× (7.33–7.53) | 6.73× (6.59–6.86) | 2.12× (2.05–2.19) | 2.08× (2.05–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.04× (2.97–3.10) | 4.75× (4.69–4.81) | 5.65× (5.51–5.80) | 5.63× (5.57–5.68) | 5.10× (5.07–5.14) | 4.37× (4.30–4.44) | 3.96× (3.86–4.05) | 1.25× (1.20–1.29) | 1.22× (1.20–1.24) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.70× (0.69–0.71) | 2.97× (2.92–3.01) | 5.23× (5.16–5.30) | 3.51× (3.46–3.56) | 6.29× (6.20–6.37) | 2.74× (2.70–2.77) | 2.69× (2.65–2.73) | 1.12× (1.10–1.15) | 1.03× (1.01–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.53–0.79) | 2.60× (2.14–3.16) | 5.80× (4.77–7.05) | 3.15× (2.60–3.82) | 5.49× (4.51–6.68) | 2.40× (1.98–2.90) | 2.35× (1.94–2.85) | 0.98× (0.81–1.19) | 0.90× (0.74–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.68× (0.67–0.70) | 3.22× (3.17–3.28) | 6.35× (6.25–6.46) | 3.44× (3.38–3.51) | 6.07× (5.96–6.18) | 2.63× (2.58–2.68) | 2.58× (2.54–2.62) | 1.08× (1.05–1.11) | 0.99× (0.97–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.02× (2.99–3.06) | 3.09× (3.05–3.13) | 4.21× (4.16–4.26) | 4.09× (4.05–4.14) | 4.66× (4.61–4.71) | 3.76× (3.71–3.80) | 2.94× (2.90–2.97) | 2.13× (2.08–2.17) | 2.10× (2.07–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.73× (2.71–2.76) | 2.85× (2.82–2.89) | 3.93× (3.88–3.97) | 3.60× (3.57–3.64) | 4.11× (4.07–4.16) | 3.23× (3.20–3.26) | 2.53× (2.50–2.55) | 1.83× (1.78–1.88) | 1.81× (1.78–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.62× (2.52–2.73) | 1.99× (1.89–2.10) | 2.86× (2.75–2.98) | 2.41× (2.29–2.53) | 2.73× (2.50–2.99) | 1.74× (1.67–1.81) | 1.36× (1.30–1.41) | 0.98× (0.94–1.03) | 0.97× (0.94–1.01) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.37–1.41) | 2.56× (2.54–2.59) | 2.36× (2.33–2.40) | 2.44× (2.40–2.48) | 2.81× (2.78–2.84) | 3.22× (3.16–3.29) | 3.12× (3.09–3.15) | 1.21× (1.21–1.22) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.28–1.33) | 2.43× (2.39–2.48) | 2.19× (2.15–2.23) | 2.88× (2.83–2.93) | 2.63× (2.56–2.69) | 2.92× (2.84–3.01) | 2.83× (2.77–2.88) | 1.10× (1.08–1.11) | 1.10× (1.08–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.62× (1.60–1.65) | 2.25× (2.18–2.31) | 1.93× (1.89–1.96) | 2.59× (2.53–2.66) | 2.39× (2.33–2.45) | 2.02× (1.96–2.09) | 1.96× (1.91–2.01) | 0.76× (0.74–0.78) | 0.76× (0.75–0.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.46) | 2.86× (2.83–2.89) | 2.51× (2.48–2.55) | 2.60× (2.57–2.63) | 3.00× (2.95–3.06) | 3.86× (3.77–3.94) | 3.86× (3.82–3.90) | 1.39× (1.37–1.41) | 1.41× (1.40–1.42) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.29–1.38) | 2.68× (2.60–2.77) | 2.28× (2.20–2.38) | 3.18× (3.03–3.33) | 2.71× (2.61–2.82) | 3.42× (3.29–3.56) | 3.42× (3.27–3.58) | 1.23× (1.18–1.29) | 1.25× (1.20–1.32) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.73× (1.62–1.84) | 2.48× (2.41–2.55) | 1.94× (1.87–2.00) | 2.88× (2.79–2.97) | 2.51× (2.43–2.59) | 2.35× (2.29–2.40) | 2.35× (2.29–2.41) | 0.85× (0.82–0.87) | 0.86× (0.84–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.75–0.78) | 2.52× (2.47–2.57) | 4.20× (4.14–4.26) | 2.95× (2.89–3.02) | 6.66× (6.55–6.77) | 2.27× (2.23–2.32) | 2.25× (2.21–2.29) | 1.04× (1.02–1.06) | 0.92× (0.90–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.70× (0.68–0.72) | 2.33× (2.27–2.40) | 4.79× (4.69–4.88) | 2.86× (2.79–2.92) | 6.19× (6.05–6.33) | 2.11× (2.06–2.16) | 2.09× (2.04–2.14) | 0.97× (0.95–0.99) | 0.85× (0.83–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.69× (0.66–0.71) | 2.64× (2.59–2.69) | 4.94× (4.65–5.24) | 2.88× (2.84–2.92) | 6.30× (6.21–6.39) | 2.12× (2.08–2.16) | 2.10× (2.07–2.14) | 0.97× (0.96–0.99) | 0.86× (0.84–0.88) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 1 | 1.83× (1.81–1.85) | 3.66× (3.63–3.70) | 4.20× (4.16–4.25) | 4.34× (4.28–4.39) | 4.62× (4.56–4.67) | 4.84× (4.79–4.90) | 4.49× (4.45–4.54) | 1.44× (1.42–1.45) | 1.40× (1.39–1.41) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 2 | 1.64× (1.62–1.65) | 3.21× (3.16–3.26) | 3.78× (3.75–3.82) | 4.16× (4.12–4.20) | 3.98× (3.94–4.03) | 4.11× (4.06–4.16) | 3.81× (3.75–3.88) | 1.22× (1.21–1.23) | 1.19× (1.17–1.20) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 4 | 1.95× (1.88–2.02) | 2.73× (2.67–2.81) | 3.16× (3.09–3.22) | 3.32× (3.26–3.38) | 3.34× (3.28–3.40) | 2.62× (2.58–2.67) | 2.43× (2.39–2.48) | 0.78× (0.76–0.79) | 0.76× (0.75–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.07× (11.05–11.09) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.32–0.35) | 0.43× (0.35–0.53) | 0.69× (0.66–0.72) | 0.58× (0.55–0.61) | 5.54× (5.30–5.78) | 0.13× (0.13–0.14) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.34–0.36) | 0.52× (0.43–0.64) | 0.63× (0.61–0.65) | 0.55× (0.53–0.57) | 3.68× (3.58–3.78) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.45) | 1.71× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.02–8.08) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.69× (0.64–0.73) | 1.60× (1.57–1.64) | 0.80× (0.79–0.82) | 5.83× (5.69–5.97) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.62–0.67) | 0.70× (0.66–0.75) | 1.15× (1.11–1.19) | 0.71× (0.68–0.74) | 3.44× (3.32–3.55) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.39× (0.35–0.42) | 0.91× (0.88–0.93) | 0.70× (0.67–0.73) | 3.14× (3.05–3.23) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.81× (0.79–0.83) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.36× (0.32–0.39) | 0.85× (0.83–0.88) | 0.72× (0.70–0.75) | 2.91× (2.84–2.99) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.49× (0.46–0.53) | 0.81× (0.79–0.82) | 0.82× (0.80–0.84) | 2.33× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.32–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.65–0.66) | 6.37× (6.35–6.38) | 0.39× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.36× (0.33–0.40) | 1.04× (1.00–1.07) | 0.67× (0.65–0.69) | 3.30× (3.22–3.38) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.4 MB → 12.2 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.46× (0.42–0.51) | 0.91× (0.88–0.94) | 0.73× (0.70–0.75) | 2.33× (2.28–2.39) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 396 | 2.54 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 697 | 2.71 | 1.08× (1.06–1.09) | 87% | 1.02× (1.01–1.04) | 1.02× (1.01–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 813 | 4.62 | 1.83× (1.81–1.85) | 51% | 1.31× (1.28–1.35) | 1.83× (1.77–1.89) |

**Best:** mibz2 at 4 threads (813 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (258 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.16, lbzip2 1t 0.04, lbzip2 2t 0.23, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.33. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.76 | 0.03 | 0.08 | 0.258 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.29 | 10.5% |
| MTF + RLE2 | 0.42 | 15.2% |
| T-vector build | 0.40 | 14.4% |
| walk (the one each build uses) | 1.54 | 56.1% |
| of which: segment assembly | 0.07 | 2.5% |
| RLE1 | 0.06 | 2.3% |
| CRC | 0.04 | 1.3% |
| **total** | 2.75 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.268 | (in its total) | 2.59× (2.58–2.60) |  |
| walk | 1.830 | 2.993 | (in its total) | 1.63× (1.62–1.64) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.085 | 0.082 | (in its total) | 0.96× (0.95–0.97) |  |
| inverse BWT, total | 2.404 | 4.343 | 2.547 | 1.80× (1.79–1.81) | 1.06× (1.05–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m45s |
| 4 threads | 37m53s ± 5m23s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m52s ± 3m56s |
| the scanner alone (its serial part) | 14m42s ± 19.7s |
| the workers alone, over 4 | 27m52s ± 3m56s |

At 4 threads the workers bound it. Per byte: the scanner 8.5 ns a byte of bz2, the workers 3.65 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,399,539 | 12,203,289 | \`c01f680597fba9e8…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
