# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-533/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`521ef93`](<https://github.com/lmmx/mibz2/commit/521ef934edb4c5f763aaf542ee70d043a1735dfc>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44065592931>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.405 s | 518 | 5.9 | 3.34× (3.31–3.37) | 7.67× (7.57–7.77) | 8.14× (8.07–8.22) | 8.98× (8.91–9.04) | 8.15× (8.06–8.24) | 8.55× (8.45–8.64) | 8.30× (8.21–8.38) | 2.63× (2.61–2.65) | 2.33× (2.31–2.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.228 s | 921 | 6.4 | 3.38× (3.31–3.44) | 7.24× (7.13–7.34) | 8.76× (8.57–8.96) | 8.65× (8.52–8.78) | 7.82× (7.71–7.93) | 15.31× (15.07–15.56) | 14.86× (14.62–15.11) | 4.71× (4.64–4.78) | 4.18× (4.12–4.24) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.190 s | 1,102 | 10.0 | 3.37× (3.32–3.43) | 6.22× (6.12–6.31) | 7.28× (7.22–7.34) | 7.23× (7.15–7.30) | 7.05× (6.96–7.14) | 18.18× (17.89–18.48) | 17.65× (17.38–17.92) | 5.59× (5.52–5.67) | 4.97× (4.90–5.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 2.7 | 0.64× (0.62–0.66) | 3.18× (3.12–3.25) | 4.54× (4.45–4.64) | 3.74× (3.68–3.81) | 6.36× (6.26–6.47) | 2.85× (2.80–2.90) | 2.89× (2.84–2.94) | 1.13× (1.10–1.16) | 1.29× (1.26–1.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,220 | 2.8 | 0.55× (0.54–0.57) | 3.12× (3.02–3.22) | 6.24× (5.97–6.52) | 3.69× (3.60–3.78) | 6.20× (6.04–6.37) | 2.79× (2.71–2.88) | 2.83× (2.76–2.91) | 1.11× (1.08–1.13) | 1.26× (1.22–1.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,216 | 2.8 | 0.55× (0.54–0.56) | 3.11× (3.05–3.16) | 6.07× (5.86–6.28) | 3.69× (3.62–3.76) | 6.19× (6.07–6.31) | 2.77× (2.72–2.82) | 2.81× (2.75–2.87) | 1.10× (1.06–1.14) | 1.25× (1.23–1.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 630 | 5.1 | 2.61× (2.54–2.67) | 3.32× (3.23–3.40) | 3.92× (3.82–4.03) | 4.27× (4.17–4.37) | 4.69× (4.58–4.80) | 3.16× (3.08–3.23) | 2.93× (2.86–3.00) | 2.01× (1.95–2.07) | 2.05× (1.95–2.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 947 | 6.0 | 2.17× (1.97–2.38) | 2.90× (2.66–3.17) | 4.13× (3.85–4.43) | 3.56× (3.23–3.92) | 4.79× (4.38–5.25) | 4.59× (4.20–5.02) | 4.26× (3.89–4.66) | 2.92× (2.64–3.23) | 2.98× (2.69–3.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,024 | 10.8 | 2.02× (1.96–2.08) | 2.21× (2.15–2.28) | 3.13× (2.91–3.36) | 2.59× (2.44–2.75) | 4.43× (4.32–4.53) | 5.10× (5.00–5.20) | 4.73× (4.63–4.83) | 3.25× (3.18–3.32) | 3.32× (3.19–3.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 43.9 | 1.38× (1.37–1.39) | 3.04× (3.02–3.07) | 2.78× (2.76–2.80) | 2.83× (2.81–2.85) | 3.12× (3.09–3.14) | 3.45× (3.43–3.48) | 3.51× (3.48–3.54) | 1.25× (1.24–1.26) | 1.25× (1.24–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 118 | 48.0 | 1.29× (1.18–1.40) | 3.03× (2.84–3.22) | 2.77× (2.56–3.00) | 3.40× (3.12–3.69) | 3.13× (2.89–3.39) | 5.58× (5.15–6.05) | 5.68× (5.24–6.15) | 2.02× (1.87–2.19) | 2.02× (1.87–2.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 141 | 66.5 | 1.40× (1.36–1.45) | 2.72× (2.64–2.81) | 2.69× (2.59–2.80) | 3.26× (3.16–3.35) | 3.22× (3.13–3.31) | 6.93× (6.76–7.12) | 7.05× (6.87–7.24) | 2.51× (2.45–2.58) | 2.51× (2.45–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 40.4 | 1.44× (1.43–1.44) | 3.41× (3.38–3.44) | 2.91× (2.87–2.95) | 3.03× (3.01–3.04) | 3.36× (3.33–3.40) | 4.12× (4.10–4.13) | 4.35× (4.33–4.37) | 1.47× (1.46–1.48) | 1.54× (1.53–1.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 128 | 44.7 | 1.28× (1.19–1.38) | 3.19× (2.88–3.54) | 2.80× (2.59–3.03) | 3.76× (3.48–4.07) | 3.23× (2.99–3.48) | 6.66× (6.18–7.18) | 7.04× (6.52–7.60) | 2.38× (2.21–2.56) | 2.49× (2.31–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 62.3 | 1.44× (1.41–1.47) | 3.05× (2.95–3.15) | 2.85× (2.76–2.93) | 3.69× (3.64–3.74) | 3.29× (3.22–3.36) | 8.32× (8.21–8.43) | 8.79× (8.66–8.92) | 2.97× (2.93–3.01) | 3.11× (3.07–3.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 957 | 3.7 | 0.70× (0.69–0.71) | 3.06× (2.87–3.26) | 4.20× (3.97–4.43) | 3.31× (3.17–3.45) | 7.09× (6.81–7.39) | 2.54× (2.44–2.64) | 2.56× (2.47–2.66) | 1.04× (1.01–1.07) | 1.01× (0.96–1.06) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 897 | 4.1 | 0.54× (0.52–0.55) | 2.77× (2.69–2.86) | 5.64× (5.47–5.82) | 3.13× (3.05–3.22) | 6.71× (6.54–6.89) | 2.36× (2.31–2.42) | 2.39× (2.34–2.43) | 0.97× (0.95–0.98) | 0.94× (0.91–0.97) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 877 | 4.0 | 0.53× (0.52–0.54) | 2.71× (2.67–2.76) | 5.41× (5.19–5.63) | 3.08× (3.02–3.14) | 6.52× (6.41–6.62) | 2.31× (2.27–2.34) | 2.33× (2.30–2.37) | 0.95× (0.92–0.98) | 0.92× (0.90–0.93) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 0.058 s | 214 | 14.7 | 1.82× (1.81–1.83) | 4.19× (4.16–4.22) | 4.64× (4.57–4.72) | 4.74× (4.70–4.77) | 4.86× (4.83–4.89) | 5.11× (5.07–5.15) | 4.95× (4.92–4.99) | 1.48× (1.47–1.49) | 1.40× (1.39–1.41) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 0.038 s | 331 | 17.4 | 1.52× (1.44–1.60) | 3.76× (3.21–4.42) | 4.27× (4.07–4.47) | 4.65× (4.47–4.83) | 4.47× (4.29–4.66) | 7.67× (7.35–8.00) | 7.44× (7.12–7.77) | 2.22× (2.13–2.32) | 2.10× (2.01–2.19) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 0.033 s | 376 | 25.7 | 1.53× (1.49–1.58) | 2.91× (2.76–3.06) | 3.59× (3.48–3.70) | 4.25× (4.06–4.46) | 4.16× (4.08–4.26) | 8.92× (8.70–9.15) | 8.65× (8.45–8.84) | 2.58× (2.53–2.64) | 2.44× (2.38–2.50) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.33× (3.30–3.36) | 7.63× (7.53–7.73) | 8.11× (8.03–8.18) | 8.94× (8.88–9.01) | 8.12× (8.04–8.21) | 8.51× (8.42–8.61) | 8.26× (8.18–8.35) | 2.62× (2.60–2.65) | 2.32× (2.30–2.34) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.22× (3.19–3.24) | 7.38× (7.27–7.50) | 8.16× (8.10–8.23) | 8.86× (8.78–8.94) | 7.67× (7.61–7.73) | 7.88× (7.79–7.97) | 7.65× (7.57–7.73) | 2.43× (2.40–2.45) | 2.15× (2.13–2.17) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.34× (3.29–3.39) | 6.80× (6.70–6.91) | 7.25× (7.16–7.34) | 7.90× (7.79–8.01) | 7.06× (6.95–7.16) | 5.06× (4.99–5.14) | 4.92× (4.84–4.99) | 1.56× (1.54–1.58) | 1.38× (1.36–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.65–0.69) | 2.91× (2.86–2.96) | 3.91× (3.85–3.97) | 3.49× (3.44–3.55) | 5.87× (5.79–5.96) | 2.68× (2.64–2.72) | 2.72× (2.68–2.76) | 1.11× (1.09–1.14) | 0.83× (0.82–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.87× (2.78–2.96) | 5.45× (5.28–5.63) | 3.42× (3.33–3.51) | 5.67× (5.51–5.84) | 2.60× (2.51–2.68) | 2.63× (2.56–2.71) | 1.08× (1.06–1.11) | 0.81× (0.79–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.58–0.59) | 3.18× (3.11–3.26) | 5.44× (5.31–5.56) | 3.47× (3.42–3.53) | 5.73× (5.63–5.82) | 2.62× (2.57–2.67) | 2.66× (2.60–2.71) | 1.09× (1.05–1.13) | 0.81× (0.80–0.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.46–2.60) | 3.07× (3.00–3.14) | 3.58× (3.49–3.66) | 4.11× (4.01–4.21) | 4.51× (4.41–4.61) | 3.05× (2.98–3.13) | 2.83× (2.77–2.90) | 1.96× (1.90–2.02) | 1.91× (1.85–1.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.34–2.43) | 2.81× (2.75–2.88) | 3.41× (3.33–3.50) | 3.61× (3.54–3.68) | 3.95× (3.87–4.02) | 2.59× (2.53–2.64) | 2.40× (2.35–2.45) | 1.66× (1.63–1.70) | 1.62× (1.58–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.02–2.21) | 1.91× (1.82–1.99) | 2.35× (2.25–2.45) | 2.31× (2.23–2.39) | 2.43× (2.35–2.52) | 1.45× (1.40–1.51) | 1.35× (1.30–1.40) | 0.93× (0.89–0.98) | 0.91× (0.87–0.95) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.39) | 2.96× (2.93–2.99) | 2.70× (2.68–2.72) | 2.80× (2.78–2.83) | 3.09× (3.06–3.11) | 3.41× (3.39–3.44) | 3.47× (3.45–3.50) | 1.25× (1.24–1.26) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.31–1.33) | 2.92× (2.89–2.95) | 2.65× (2.63–2.67) | 3.35× (3.33–3.37) | 2.94× (2.91–2.96) | 3.13× (3.11–3.16) | 3.19× (3.16–3.22) | 1.15× (1.14–1.16) | 1.13× (1.12–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.52–1.60) | 2.93× (2.86–3.01) | 2.47× (2.39–2.56) | 3.23× (3.16–3.30) | 2.91× (2.85–2.97) | 2.28× (2.24–2.33) | 2.32× (2.27–2.37) | 0.83× (0.82–0.85) | 0.82× (0.80–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.43–1.44) | 3.35× (3.32–3.37) | 2.83× (2.81–2.86) | 3.00× (2.99–3.02) | 3.33× (3.30–3.37) | 4.08× (4.06–4.09) | 4.31× (4.29–4.33) | 1.46× (1.46–1.47) | 1.47× (1.47–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.32–1.37) | 3.18× (3.10–3.25) | 2.72× (2.67–2.78) | 3.66× (3.58–3.74) | 3.09× (3.03–3.16) | 3.66× (3.58–3.73) | 3.87× (3.78–3.95) | 1.31× (1.29–1.34) | 1.32× (1.29–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.58–1.65) | 3.25× (3.16–3.34) | 2.51× (2.45–2.58) | 3.50× (3.44–3.57) | 3.03× (2.98–3.08) | 2.65× (2.61–2.68) | 2.80× (2.76–2.83) | 0.95× (0.94–0.96) | 0.95× (0.94–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.74× (0.73–0.76) | 2.61× (2.52–2.70) | 3.61× (3.45–3.78) | 3.02× (2.89–3.15) | 6.32× (6.06–6.60) | 2.35× (2.26–2.45) | 2.37× (2.29–2.46) | 1.04× (1.01–1.07) | 0.79× (0.76–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.30× (2.25–2.35) | 4.39× (4.29–4.50) | 2.82× (2.76–2.89) | 5.82× (5.68–5.96) | 2.12× (2.07–2.17) | 2.14× (2.11–2.17) | 0.94× (0.92–0.96) | 0.71× (0.70–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.57–0.60) | 2.61× (2.51–2.72) | 4.41× (4.29–4.52) | 2.88× (2.82–2.94) | 5.87× (5.77–5.98) | 2.15× (2.11–2.20) | 2.18× (2.13–2.22) | 0.96× (0.93–0.99) | 0.72× (0.71–0.74) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 1.79× (1.78–1.80) | 4.03× (3.99–4.06) | 4.47× (4.44–4.50) | 4.63× (4.59–4.67) | 4.75× (4.72–4.79) | 4.99× (4.95–5.04) | 4.84× (4.80–4.88) | 1.47× (1.46–1.48) | 1.37× (1.36–1.38) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 1.62× (1.60–1.63) | 3.61× (3.56–3.65) | 4.21× (4.18–4.25) | 4.45× (4.40–4.49) | 4.17× (4.12–4.22) | 4.21× (4.16–4.26) | 4.08× (4.03–4.13) | 1.24× (1.22–1.25) | 1.15× (1.14–1.17) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 1.76× (1.66–1.86) | 3.35× (3.26–3.45) | 3.75× (3.67–3.84) | 3.93× (3.77–4.11) | 3.77× (3.68–3.87) | 2.85× (2.77–2.93) | 2.76× (2.69–2.84) | 0.84× (0.81–0.86) | 0.78× (0.76–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.28× (0.27–0.29) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.96–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.44× (0.36–0.53) | 0.70× (0.68–0.72) | 0.59× (0.57–0.61) | 5.58× (5.43–5.73) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.53× (0.45–0.64) | 0.60× (0.57–0.62) | 0.53× (0.51–0.55) | 3.49× (3.35–3.64) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.69–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.05) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.73× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.65× (0.62–0.69) | 1.62× (1.58–1.66) | 0.82× (0.81–0.84) | 5.93× (5.82–6.04) | 0.48× (0.47–0.49) | 0.50× (0.49–0.52) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.65) | 0.71× (0.66–0.77) | 1.16× (1.15–1.18) | 0.72× (0.72–0.73) | 3.47× (3.45–3.50) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.18× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.44) | 0.91× (0.89–0.94) | 0.72× (0.70–0.74) | 3.14× (3.05–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.48) | 0.88× (0.86–0.90) | 0.83× (0.81–0.84) | 2.55× (2.49–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.18–5.19) | 0.32× (0.32–0.33) | 0.34× (0.33–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 39 MB | 0.50× (0.47–0.52) | 0.40× (0.37–0.42) | 0.89× (0.85–0.93) | 0.71× (0.67–0.74) | 3.03× (2.90–3.16) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.50) | 0.82× (0.80–0.83) | 0.84× (0.81–0.87) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.31–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.33× (5.31–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.67) | 6.37× (6.36–6.38) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 37 MB | 0.45× (0.45–0.46) | 0.36× (0.34–0.39) | 1.06× (1.03–1.09) | 0.69× (0.67–0.70) | 3.33× (3.26–3.40) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.51) | 0.47× (0.42–0.53) | 0.91× (0.88–0.94) | 0.71× (0.68–0.74) | 2.32× (2.27–2.37) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 518 | 1.94 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 921 | 2.10 | 1.08× (1.08–1.08) | 90% | 1.05× (1.03–1.06) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,102 | 3.26 | 1.68× (1.66–1.70) | 53% | 1.50× (1.49–1.51) | 1.69× (1.67–1.71) |

**Best:** mibz2 at 4 threads (1,102 MB/s); lbzip2 at 4 threads (176 MB/s); pbz2 at 4 threads (325 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.22, lbzip2 1t 0.04, lbzip2 2t 0.18, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.51 | 0.02 | 0.08 | 0.190 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3062 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.7% |
| MTF + RLE2 | 1.29 | 15.1% |
| T-vector build | 1.24 | 14.5% |
| walk (the one each build uses) | 4.80 | 56.2% |
| of which: segment assembly | 0.25 | 2.9% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.11 | 1.3% |
| **total** | 8.55 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.942 | (in its total) | 2.52× (2.52–2.52) |  |
| walk | 1.480 | 2.839 | (in its total) | 1.92× (1.92–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.072 | (in its total) | 0.90× (0.89–0.90) |  |
| inverse BWT, total | 1.935 | 3.854 | 2.007 | 1.99× (1.99–1.99) | 1.04× (1.04–1.04) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m11s ± 6m50s |
| 4 threads | 27m49s ± 3m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m33s ± 2m41s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 18m33s ± 2m41s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.43 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,457,111 | 12,465,313 | \`f5a13800cacccde1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3062.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
