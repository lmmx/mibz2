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

The latest record: [`3d8fc66`](<https://github.com/lmmx/mibz2/commit/3d8fc666e201534e0c08e92ca92cb4d2bffe1d22>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46083192412>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.512 s | 410 | 8.7 | 3.18× (3.10–3.27) | 6.88× (6.67–7.10) | 7.94× (7.75–8.14) | 8.07× (7.95–8.19) | 7.31× (7.18–7.44) | 8.16× (8.05–8.28) | 7.39× (7.22–7.56) | 2.24× (2.20–2.29) | 2.26× (2.22–2.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.287 s | 730 | 9.2 | 3.41× (3.14–3.69) | 6.58× (6.21–6.97) | 8.75× (8.13–9.42) | 7.97× (7.42–8.55) | 7.32× (6.71–7.99) | 14.67× (14.01–15.37) | 13.28× (12.62–13.98) | 4.03× (3.88–4.18) | 4.07× (3.92–4.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 858 | 15.7 | 3.76× (3.52–4.00) | 5.40× (4.83–6.05) | 7.25× (6.67–7.87) | 6.35× (5.88–6.85) | 6.38× (5.88–6.94) | 17.24× (16.95–17.53) | 15.61× (15.27–15.95) | 4.73× (4.64–4.82) | 4.78× (4.67–4.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,000 | 3.9 | 0.66× (0.65–0.67) | 3.05× (2.99–3.12) | 5.49× (5.36–5.62) | 3.54× (3.47–3.60) | 6.40× (6.31–6.49) | 2.75× (2.70–2.80) | 2.70× (2.65–2.75) | 1.19× (1.15–1.24) | 1.08× (1.00–1.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 932 | 4.2 | 0.61× (0.59–0.63) | 2.85× (2.79–2.91) | 6.58× (6.41–6.77) | 3.36× (3.29–3.42) | 6.03× (5.90–6.15) | 2.57× (2.52–2.62) | 2.52× (2.46–2.57) | 1.11× (1.05–1.18) | 1.01× (0.92–1.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 915 | 4.2 | 0.60× (0.59–0.62) | 2.82× (2.76–2.87) | 6.58× (6.44–6.72) | 3.30× (3.22–3.38) | 6.01× (5.87–6.14) | 2.53× (2.49–2.57) | 2.48× (2.43–2.53) | 1.09× (1.04–1.15) | 0.99× (0.90–1.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 548 | 6.7 | 3.07× (3.01–3.13) | 3.31× (3.13–3.50) | 4.42× (4.25–4.60) | 4.16× (3.97–4.36) | 4.74× (4.51–4.98) | 3.81× (3.60–4.02) | 2.96× (2.81–3.12) | 2.19× (2.09–2.29) | 2.28× (2.23–2.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 871 | 7.9 | 2.57× (2.21–2.98) | 2.97× (2.63–3.36) | 4.76× (4.33–5.23) | 3.67× (3.16–4.26) | 5.27× (4.53–6.13) | 5.82× (5.00–6.76) | 4.52× (3.89–5.25) | 3.35× (2.84–3.94) | 3.48× (2.99–4.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 829 | 15.0 | 2.70× (2.62–2.79) | 2.14× (1.98–2.31) | 3.65× (3.45–3.86) | 2.66× (2.41–2.93) | 4.76× (4.48–5.05) | 5.89× (5.54–6.25) | 4.57× (4.33–4.84) | 3.39× (3.17–3.62) | 3.53× (3.38–3.68) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 60 | 59.2 | 1.56× (1.55–1.57) | 2.89× (2.78–2.99) | 2.67× (2.57–2.77) | 2.69× (2.59–2.79) | 3.10× (3.00–3.21) | 3.51× (3.44–3.58) | 3.46× (3.34–3.59) | 1.50× (1.47–1.53) | 1.52× (1.48–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.077 s | 104 | 63.4 | 1.57× (1.53–1.61) | 2.97× (2.75–3.22) | 2.90× (2.60–3.23) | 3.64× (3.39–3.90) | 3.38× (3.24–3.54) | 6.02× (5.83–6.22) | 5.93× (5.67–6.21) | 2.57× (2.50–2.65) | 2.60× (2.51–2.70) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.064 s | 125 | 88.5 | 1.78× (1.70–1.86) | 2.75× (2.56–2.95) | 2.84× (2.60–3.10) | 3.36× (3.11–3.63) | 3.57× (3.36–3.80) | 7.11× (6.85–7.39) | 7.01× (6.66–7.38) | 3.04× (2.94–3.15) | 3.08× (2.96–3.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.151 s | 63 | 56.4 | 1.62× (1.61–1.63) | 3.21× (3.01–3.43) | 2.94× (2.68–3.24) | 2.89× (2.71–3.07) | 3.21× (3.08–3.34) | 4.15× (4.08–4.21) | 4.23× (4.10–4.37) | 1.67× (1.65–1.69) | 1.73× (1.70–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 61.3 | 1.54× (1.48–1.60) | 3.08× (2.91–3.26) | 2.92× (2.75–3.10) | 4.03× (3.74–4.33) | 3.35× (3.24–3.47) | 7.07× (6.83–7.32) | 7.21× (6.94–7.49) | 2.85× (2.76–2.94) | 2.94× (2.83–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 129 | 90.5 | 1.88× (1.83–1.93) | 2.82× (2.69–2.94) | 2.87× (2.70–3.05) | 3.62× (3.48–3.76) | 3.49× (3.29–3.70) | 8.43× (8.15–8.71) | 8.59× (8.23–8.98) | 3.39× (3.29–3.50) | 3.51× (3.38–3.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 758 | 5.3 | 0.74× (0.72–0.77) | 3.06× (2.99–3.13) | 5.14× (4.95–5.33) | 3.13× (3.06–3.21) | 7.24× (7.10–7.38) | 2.43× (2.40–2.47) | 2.40× (2.37–2.44) | 1.09× (1.06–1.12) | 1.55× (1.51–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 656 | 6.2 | 0.62× (0.61–0.63) | 2.73× (2.66–2.80) | 5.56× (5.31–5.83) | 2.83× (2.77–2.90) | 6.62× (6.28–6.98) | 2.16× (2.10–2.23) | 2.13× (2.07–2.20) | 0.97× (0.94–0.99) | 1.37× (1.34–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 645 | 6.2 | 0.62× (0.61–0.63) | 2.67× (2.64–2.71) | 5.40× (5.22–5.59) | 2.78× (2.76–2.81) | 6.52× (6.10–6.96) | 2.12× (2.08–2.15) | 2.09× (2.06–2.11) | 0.95× (0.92–0.98) | 1.34× (1.32–1.36) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.070 s | 212 | 17.1 | 2.10× (2.06–2.13) | 4.14× (3.85–4.46) | 4.89× (4.67–5.12) | 4.82× (4.65–5.00) | 4.91× (4.79–5.02) | 5.13× (5.01–5.25) | 4.73× (4.62–4.85) | 1.63× (1.60–1.66) | 1.64× (1.60–1.68) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.046 s | 321 | 20.5 | 1.67× (1.51–1.85) | 3.23× (2.93–3.57) | 4.43× (4.07–4.82) | 4.56× (4.16–5.01) | 4.39× (4.04–4.76) | 7.27× (6.56–8.05) | 6.71× (6.05–7.44) | 2.31× (2.10–2.53) | 2.33× (2.09–2.59) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.042 s | 353 | 33.3 | 1.88× (1.83–1.94) | 2.99× (2.68–3.33) | 3.94× (3.70–4.20) | 4.02× (3.86–4.18) | 4.58× (4.36–4.81) | 8.49× (8.23–8.76) | 7.84× (7.54–8.15) | 2.70× (2.61–2.79) | 2.72× (2.64–2.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.17× (3.09–3.26) | 6.85× (6.65–7.07) | 7.91× (7.71–8.11) | 8.04× (7.93–8.16) | 7.29× (7.16–7.42) | 8.14× (8.02–8.25) | 7.37× (7.20–7.54) | 2.24× (2.19–2.28) | 2.25× (2.21–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.20× (3.12–3.28) | 6.79× (6.44–7.17) | 8.30× (7.79–8.85) | 8.27× (7.82–8.74) | 7.33× (6.78–7.93) | 7.73× (7.55–7.90) | 6.99× (6.80–7.19) | 2.12× (2.10–2.15) | 2.14× (2.11–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.57× (3.39–3.77) | 5.60× (5.00–6.28) | 6.79× (6.18–7.46) | 6.55× (6.08–7.06) | 5.99× (5.48–6.55) | 4.56× (4.45–4.68) | 4.13× (4.02–4.24) | 1.25× (1.22–1.29) | 1.26× (1.22–1.30) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.69× (0.67–0.70) | 2.80× (2.75–2.86) | 4.97× (4.88–5.06) | 3.32× (3.25–3.39) | 5.93× (5.85–6.02) | 2.60× (2.56–2.65) | 2.55× (2.50–2.61) | 1.17× (1.13–1.21) | 0.99× (0.98–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.63–0.66) | 2.56× (2.51–2.62) | 5.87× (5.76–5.99) | 3.10× (3.04–3.16) | 5.48× (5.38–5.59) | 2.38× (2.33–2.42) | 2.33× (2.28–2.38) | 1.07× (1.01–1.13) | 0.91× (0.89–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.61–0.63) | 2.95× (2.90–3.00) | 5.86× (5.76–5.96) | 3.09× (3.02–3.17) | 5.53× (5.42–5.65) | 2.38× (2.34–2.41) | 2.33× (2.29–2.38) | 1.07× (1.01–1.13) | 0.91× (0.89–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.96× (2.89–3.04) | 3.01× (2.86–3.16) | 4.19× (4.00–4.38) | 4.00× (3.82–4.19) | 4.55× (4.33–4.78) | 3.66× (3.46–3.87) | 2.86× (2.71–3.01) | 2.13× (2.03–2.23) | 2.11× (2.05–2.18) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.71× (2.63–2.79) | 2.76× (2.64–2.90) | 3.91× (3.76–4.06) | 3.58× (3.40–3.76) | 4.02× (3.83–4.23) | 3.13× (2.97–3.30) | 2.44× (2.32–2.57) | 1.82× (1.73–1.91) | 1.80× (1.74–1.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.77× (2.66–2.88) | 1.94× (1.87–2.02) | 2.80× (2.71–2.89) | 2.30× (2.19–2.41) | 2.59× (2.49–2.68) | 1.68× (1.61–1.74) | 1.31× (1.26–1.36) | 0.97× (0.91–1.04) | 0.97× (0.93–1.00) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.55× (1.54–1.56) | 2.82× (2.72–2.92) | 2.61× (2.51–2.71) | 2.66× (2.57–2.76) | 3.07× (2.97–3.18) | 3.48× (3.41–3.55) | 3.43× (3.30–3.55) | 1.49× (1.46–1.53) | 1.46× (1.41–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.51× (1.49–1.53) | 2.90× (2.66–3.15) | 2.61× (2.41–2.82) | 3.31× (3.15–3.48) | 2.96× (2.82–3.11) | 3.22× (3.15–3.31) | 3.18× (3.06–3.30) | 1.39× (1.36–1.41) | 1.35× (1.31–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.94× (1.86–2.01) | 2.94× (2.68–3.21) | 2.68× (2.38–3.01) | 3.35× (3.04–3.69) | 3.13× (2.84–3.46) | 2.31× (2.25–2.38) | 2.28× (2.18–2.38) | 0.99× (0.96–1.02) | 0.97× (0.93–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.61× (1.60–1.62) | 3.15× (2.95–3.37) | 2.90× (2.64–3.18) | 2.86× (2.69–3.05) | 3.18× (3.05–3.31) | 4.11× (4.05–4.17) | 4.19× (4.06–4.32) | 1.66× (1.64–1.68) | 1.69× (1.67–1.72) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.55× (1.50–1.59) | 3.04× (2.92–3.17) | 2.73× (2.59–2.89) | 3.70× (3.49–3.94) | 3.00× (2.91–3.09) | 3.73× (3.63–3.83) | 3.80× (3.67–3.93) | 1.51× (1.47–1.55) | 1.53× (1.49–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 2.00× (1.96–2.03) | 2.92× (2.75–3.11) | 2.53× (2.31–2.78) | 3.27× (3.11–3.45) | 3.04× (2.79–3.31) | 2.57× (2.50–2.63) | 2.62× (2.52–2.72) | 1.04× (1.02–1.06) | 1.06× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.78× (0.75–0.80) | 2.49× (2.43–2.55) | 4.18× (4.12–4.24) | 2.89× (2.83–2.94) | 6.51× (6.38–6.64) | 2.27× (2.23–2.30) | 2.24× (2.21–2.27) | 1.08× (1.05–1.11) | 0.91× (0.90–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.66× (0.65–0.67) | 2.16× (2.10–2.23) | 4.61× (4.45–4.79) | 2.60× (2.52–2.67) | 5.81× (5.53–6.11) | 1.96× (1.89–2.03) | 1.94× (1.87–2.01) | 0.93× (0.91–0.96) | 0.79× (0.76–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.64× (0.63–0.65) | 2.49× (2.47–2.52) | 4.60× (4.53–4.67) | 2.60× (2.57–2.64) | 5.86× (5.48–6.26) | 1.96× (1.93–1.99) | 1.94× (1.91–1.96) | 0.93× (0.91–0.96) | 0.79× (0.78–0.80) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 2.07× (2.03–2.10) | 3.99× (3.70–4.30) | 4.72× (4.50–4.96) | 4.72× (4.56–4.89) | 4.80× (4.69–4.91) | 5.02× (4.91–5.13) | 4.63× (4.52–4.75) | 1.61× (1.58–1.64) | 1.55× (1.52–1.59) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.83× (1.79–1.86) | 3.43× (3.33–3.54) | 4.30× (4.11–4.49) | 4.50× (4.38–4.63) | 4.15× (4.04–4.25) | 4.14× (4.04–4.24) | 3.82× (3.74–3.91) | 1.33× (1.31–1.35) | 1.28× (1.25–1.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 2.07× (2.02–2.13) | 3.10× (2.84–3.38) | 3.57× (3.30–3.86) | 3.63× (3.46–3.82) | 3.68× (3.42–3.96) | 2.59× (2.54–2.64) | 2.39× (2.31–2.47) | 0.83× (0.81–0.86) | 0.80× (0.78–0.82) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.09× (11.05–11.13) | 0.27× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.44) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.35× (0.34–0.36) | 0.43× (0.37–0.50) | 0.73× (0.70–0.76) | 0.61× (0.59–0.64) | 5.79× (5.61–5.97) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.37× (0.35–0.39) | 0.54× (0.46–0.64) | 0.66× (0.63–0.70) | 0.57× (0.53–0.61) | 3.82× (3.61–4.04) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.66–0.66) | 8.01× (7.99–8.03) | 0.66× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.73× (0.72–0.73) | 0.71× (0.67–0.75) | 1.74× (1.72–1.75) | 0.87× (0.86–0.88) | 6.31× (6.25–6.38) | 0.51× (0.51–0.52) | 0.54× (0.53–0.55) | 0.59× (0.58–0.59) | 0.29× (0.29–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.69–0.71) | 0.75× (0.70–0.81) | 1.24× (1.20–1.28) | 0.77× (0.75–0.80) | 3.71× (3.60–3.82) | 0.29× (0.28–0.30) | 0.31× (0.30–0.32) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.72× (0.72–0.73) | 5.18× (5.16–5.19) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.50) | 0.37× (0.34–0.41) | 0.88× (0.86–0.90) | 0.69× (0.66–0.72) | 3.02× (2.94–3.11) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.47× (0.46–0.48) | 0.88× (0.86–0.89) | 0.81× (0.79–0.84) | 2.57× (2.52–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.21–5.25) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.45× (0.43–0.47) | 0.34× (0.31–0.37) | 0.80× (0.78–0.83) | 0.69× (0.66–0.72) | 2.76× (2.67–2.85) | 0.17× (0.17–0.18) | 0.18× (0.17–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.48× (0.45–0.50) | 0.80× (0.78–0.82) | 0.82× (0.79–0.85) | 2.31× (2.26–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.90) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.19–6.22) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.47× (0.46–0.47) | 0.39× (0.39–0.39) | 1.40× (1.40–1.40) | 0.67× (0.66–0.67) | 6.45× (6.43–6.46) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 32 MB | 0.53× (0.51–0.55) | 0.49× (0.45–0.52) | 1.20× (1.16–1.23) | 0.79× (0.77–0.82) | 3.95× (3.82–4.08) | 0.24× (0.23–0.25) | 0.26× (0.25–0.27) | 0.39× (0.38–0.40) | 0.19× (0.18–0.20) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 52 MB | 0.54× (0.53–0.55) | 0.50× (0.47–0.54) | 0.99× (0.96–1.01) | 0.82× (0.80–0.84) | 2.54× (2.49–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 410 | 2.45 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 730 | 2.60 | 1.05× (1.03–1.07) | 90% | 1.04× (0.98–1.11) | 1.06× (1.02–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 858 | 4.44 | 1.78× (1.74–1.83) | 53% | 1.46× (1.29–1.65) | 2.01× (1.88–2.15) |

**Best:** mibz2 at 4 threads (858 MB/s); lbzip2 at 4 threads (169 MB/s); pbz2 at 4 threads (235 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 0.94, lbzip2 1t 0.05, lbzip2 2t 0.16, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.32. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.76 | 0.02 | 0.09 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3529 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.89 | 7.8% |
| MTF + RLE2 | 1.78 | 15.6% |
| T-vector build | 1.76 | 15.4% |
| walk (the one each build uses) | 6.54 | 57.3% |
| of which: segment assembly | 0.32 | 2.8% |
| RLE1 | 0.28 | 2.4% |
| CRC | 0.15 | 1.4% |
| **total** | 11.42 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.490 | 1.264 | (in its total) | 2.58× (2.58–2.59) |  |
| walk | 1.816 | 2.777 | (in its total) | 1.54× (1.52–1.55) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.117 | 0.124 | (in its total) | 1.05× (1.01–1.10) |  |
| inverse BWT, total | 2.423 | 4.169 | 2.515 | 1.73× (1.71–1.74) | 1.04× (1.03–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 9m23s |
| 4 threads | 35m41s ± 5m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m10s ± 4m00s |
| the scanner alone (its serial part) | 9m45s ± 30.2s |
| the workers alone, over 4 | 27m10s ± 4m00s |

At 4 threads the workers bound it. Per byte: the scanner 5.7 ns a byte of bz2, the workers 3.56 ns a byte of output at 4 threads (CPU, summed over them), 2.15 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,227,455 | 14,888,808 | \`c0a0f7a4c597a907…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 3528.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
