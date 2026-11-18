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

The latest record: [`37d3d98`](<https://github.com/lmmx/mibz2/commit/37d3d98b595dd31e99780099345816ed9849566a>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44281638053>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.407 s | 516 | 5.3 | 3.31× (3.27–3.34) | 7.57× (7.50–7.64) | 8.04× (7.96–8.13) | 8.97× (8.89–9.05) | 8.10× (8.03–8.18) | 8.48× (8.40–8.57) | 8.29× (8.22–8.35) | 2.57× (2.55–2.59) | 2.30× (2.27–2.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.225 s | 931 | 5.7 | 3.29× (3.25–3.33) | 7.18× (7.00–7.36) | 8.65× (8.53–8.77) | 8.58× (8.44–8.71) | 7.74× (7.63–7.85) | 15.21× (14.96–15.47) | 14.86× (14.64–15.07) | 4.61× (4.54–4.68) | 4.12× (4.05–4.19) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.190 s | 1,104 | 8.7 | 3.29× (3.25–3.34) | 6.12× (6.04–6.20) | 7.16× (7.09–7.23) | 7.08× (7.01–7.15) | 6.92× (6.85–6.99) | 18.20× (18.03–18.37) | 17.78× (17.64–17.91) | 5.51× (5.46–5.57) | 4.93× (4.87–4.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 2.4 | 0.65× (0.63–0.68) | 3.27× (3.06–3.49) | 4.68× (4.37–5.01) | 3.94× (3.72–4.17) | 6.62× (6.24–7.02) | 2.94× (2.76–3.12) | 2.97× (2.80–3.16) | 1.13× (1.10–1.16) | 1.32× (1.24–1.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,231 | 2.5 | 0.57× (0.56–0.59) | 3.16× (3.10–3.21) | 6.26× (6.01–6.52) | 3.84× (3.77–3.91) | 6.35× (6.22–6.48) | 2.83× (2.79–2.88) | 2.87× (2.82–2.92) | 1.09× (1.03–1.15) | 1.28× (1.25–1.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,227 | 2.4 | 0.57× (0.55–0.60) | 3.15× (3.00–3.30) | 6.35× (6.05–6.66) | 3.85× (3.68–4.02) | 6.38× (6.07–6.71) | 2.82× (2.69–2.95) | 2.85× (2.72–2.99) | 1.08× (1.01–1.15) | 1.27× (1.21–1.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 640 | 4.5 | 2.88× (2.84–2.92) | 3.34× (3.29–3.39) | 4.02× (3.95–4.09) | 4.37× (4.31–4.44) | 4.75× (4.69–4.82) | 3.19× (3.15–3.24) | 2.95× (2.92–2.98) | 1.98× (1.96–2.01) | 2.10× (1.99–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 998 | 5.3 | 2.30× (2.06–2.56) | 3.11× (2.56–3.79) | 4.22× (3.72–4.80) | 3.70× (3.29–4.16) | 4.85× (4.32–5.44) | 4.66× (4.15–5.22) | 4.30× (3.83–4.84) | 2.89× (2.59–3.23) | 3.06× (2.68–3.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,013 | 9.2 | 2.02× (1.90–2.14) | 2.10× (1.94–2.27) | 3.12× (2.90–3.36) | 2.65× (2.44–2.88) | 4.40× (4.23–4.59) | 5.07× (4.89–5.26) | 4.69× (4.51–4.87) | 3.15× (3.03–3.27) | 3.33× (3.11–3.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 70 | 38.8 | 1.37× (1.37–1.38) | 2.98× (2.96–3.01) | 2.71× (2.69–2.73) | 2.79× (2.78–2.80) | 3.09× (3.08–3.11) | 3.42× (3.41–3.43) | 3.53× (3.51–3.55) | 1.24× (1.24–1.25) | 1.25× (1.25–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 115 | 42.5 | 1.22× (1.12–1.34) | 2.66× (2.42–2.92) | 2.61× (2.38–2.87) | 3.19× (2.91–3.50) | 3.00× (2.73–3.30) | 5.32× (4.85–5.82) | 5.49× (5.01–6.01) | 1.93× (1.76–2.12) | 1.95× (1.78–2.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 57.6 | 1.42× (1.37–1.47) | 2.82× (2.71–2.93) | 2.64× (2.57–2.71) | 3.28× (3.17–3.39) | 3.28× (3.19–3.37) | 7.02× (6.84–7.21) | 7.25× (7.05–7.46) | 2.55× (2.48–2.62) | 2.57× (2.50–2.64) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.127 s | 76 | 36.2 | 1.40× (1.39–1.41) | 3.31× (3.27–3.34) | 2.85× (2.82–2.88) | 2.93× (2.90–2.95) | 3.26× (3.24–3.28) | 4.03× (4.00–4.06) | 4.31× (4.28–4.34) | 1.43× (1.42–1.44) | 1.51× (1.50–1.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 129 | 39.5 | 1.29× (1.18–1.40) | 3.04× (2.80–3.29) | 2.81× (2.62–3.01) | 3.72× (3.46–4.00) | 3.21× (2.98–3.46) | 6.67× (6.17–7.22) | 7.13× (6.57–7.74) | 2.37× (2.20–2.57) | 2.50× (2.31–2.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 55.2 | 1.43× (1.39–1.47) | 2.94× (2.85–3.02) | 2.61× (2.54–2.70) | 3.64× (3.50–3.79) | 3.14× (3.07–3.21) | 8.08× (7.90–8.27) | 8.64× (8.43–8.86) | 2.88× (2.82–2.94) | 3.03× (2.96–3.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 939 | 3.3 | 0.68× (0.66–0.70) | 2.87× (2.81–2.94) | 4.06× (3.88–4.24) | 3.26× (3.20–3.33) | 6.94× (6.81–7.07) | 2.45× (2.40–2.49) | 2.47× (2.42–2.52) | 1.05× (1.03–1.06) | 0.98× (0.96–1.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 889 | 3.6 | 0.55× (0.53–0.58) | 2.79× (2.67–2.91) | 5.59× (5.41–5.79) | 3.21× (3.09–3.33) | 6.75× (6.51–7.00) | 2.38× (2.28–2.47) | 2.40× (2.30–2.50) | 1.02× (0.99–1.04) | 0.95× (0.91–0.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 885 | 3.5 | 0.55× (0.53–0.57) | 2.75× (2.67–2.84) | 5.59× (5.41–5.78) | 3.18× (3.09–3.27) | 6.72× (6.51–6.94) | 2.34× (2.27–2.41) | 2.36× (2.28–2.44) | 1.00× (0.97–1.03) | 0.94× (0.91–0.97) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 0.059 s | 216 | 12.8 | 1.83× (1.82–1.83) | 4.11× (4.09–4.13) | 4.64× (4.61–4.67) | 4.71× (4.67–4.74) | 4.85× (4.82–4.89) | 5.09× (5.07–5.12) | 5.04× (5.02–5.06) | 1.47× (1.46–1.48) | 1.52× (1.49–1.56) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 0.038 s | 341 | 14.9 | 1.59× (1.56–1.62) | 3.69× (3.53–3.84) | 4.51× (4.44–4.58) | 4.72× (4.62–4.83) | 4.69× (4.58–4.80) | 8.00× (7.80–8.21) | 7.92× (7.73–8.11) | 2.31× (2.25–2.36) | 2.39× (2.32–2.47) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 0.033 s | 391 | 22.5 | 1.57× (1.53–1.61) | 3.04× (2.90–3.18) | 3.83× (3.73–3.93) | 4.21× (4.11–4.31) | 4.40× (4.29–4.52) | 9.21× (8.99–9.42) | 9.11× (8.90–9.32) | 2.65× (2.59–2.71) | 2.75× (2.66–2.85) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.30× (3.26–3.33) | 7.53× (7.46–7.60) | 8.01× (7.92–8.09) | 8.93× (8.85–9.02) | 8.07× (7.99–8.15) | 8.45× (8.36–8.54) | 8.25× (8.19–8.32) | 2.56× (2.54–2.59) | 2.28× (2.25–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.16× (3.13–3.19) | 7.35× (7.21–7.49) | 8.01× (7.94–8.07) | 8.79× (8.70–8.87) | 7.59× (7.51–7.67) | 7.84× (7.75–7.93) | 7.66× (7.58–7.73) | 2.38× (2.35–2.40) | 2.12× (2.09–2.15) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.28× (3.24–3.32) | 6.72× (6.64–6.81) | 7.14× (7.05–7.23) | 7.81× (7.72–7.90) | 6.98× (6.88–7.07) | 5.11× (5.05–5.18) | 4.99× (4.92–5.06) | 1.55× (1.53–1.57) | 1.38× (1.36–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.66–0.71) | 2.99× (2.80–3.19) | 4.03× (3.80–4.26) | 3.66× (3.45–3.89) | 6.10× (5.74–6.49) | 2.77× (2.59–2.95) | 2.80× (2.62–2.98) | 1.12× (1.09–1.15) | 0.84× (0.80–0.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 2.89× (2.83–2.94) | 5.50× (5.37–5.64) | 3.53× (3.45–3.60) | 5.76× (5.65–5.87) | 2.62× (2.57–2.67) | 2.65× (2.60–2.69) | 1.06× (1.01–1.12) | 0.80× (0.77–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.59–0.64) | 3.32× (3.14–3.52) | 5.61× (5.37–5.85) | 3.63× (3.49–3.78) | 5.93× (5.67–6.21) | 2.68× (2.57–2.79) | 2.71× (2.59–2.83) | 1.08× (1.02–1.15) | 0.81× (0.78–0.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.74–2.84) | 3.10× (3.04–3.16) | 3.63× (3.56–3.71) | 4.21× (4.13–4.29) | 4.57× (4.49–4.65) | 3.08× (3.03–3.14) | 2.86× (2.82–2.90) | 1.94× (1.91–1.96) | 1.93× (1.89–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.56× (2.46–2.65) | 2.87× (2.74–3.01) | 3.44× (3.32–3.57) | 3.71× (3.56–3.86) | 4.01× (3.87–4.17) | 2.64× (2.54–2.74) | 2.44× (2.35–2.54) | 1.66× (1.60–1.72) | 1.65× (1.57–1.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.02–2.19) | 1.91× (1.85–1.97) | 2.39× (2.31–2.46) | 2.36× (2.28–2.43) | 2.48× (2.42–2.55) | 1.49× (1.44–1.53) | 1.38× (1.34–1.41) | 0.93× (0.91–0.96) | 0.93× (0.91–0.95) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.37) | 2.91× (2.90–2.92) | 2.66× (2.64–2.67) | 2.76× (2.75–2.77) | 3.06× (3.05–3.08) | 3.38× (3.37–3.40) | 3.49× (3.47–3.51) | 1.24× (1.23–1.24) | 1.23× (1.22–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.29–1.31) | 2.82× (2.78–2.85) | 2.58× (2.54–2.61) | 3.27× (3.23–3.31) | 2.88× (2.84–2.91) | 3.09× (3.05–3.13) | 3.19× (3.15–3.23) | 1.13× (1.12–1.14) | 1.12× (1.11–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.52–1.59) | 2.82× (2.71–2.94) | 2.46× (2.40–2.51) | 3.22× (3.16–3.28) | 2.87× (2.82–2.92) | 2.28× (2.25–2.32) | 2.36× (2.31–2.40) | 0.84× (0.82–0.85) | 0.83× (0.81–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.39–1.41) | 3.24× (3.22–3.26) | 2.79× (2.77–2.82) | 2.90× (2.88–2.92) | 3.23× (3.21–3.25) | 3.99× (3.96–4.02) | 4.26× (4.23–4.29) | 1.43× (1.42–1.44) | 1.44× (1.43–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.36) | 3.15× (3.12–3.18) | 2.70× (2.68–2.73) | 3.62× (3.56–3.68) | 3.04× (3.01–3.08) | 3.67× (3.63–3.71) | 3.92× (3.87–3.97) | 1.31× (1.30–1.32) | 1.32× (1.31–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.50–1.65) | 3.22× (3.14–3.31) | 2.44× (2.38–2.49) | 3.38× (3.28–3.48) | 2.92× (2.84–3.01) | 2.62× (2.55–2.70) | 2.81× (2.72–2.89) | 0.94× (0.91–0.97) | 0.95× (0.92–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.71–0.74) | 2.49× (2.44–2.54) | 3.48× (3.42–3.54) | 2.96× (2.90–3.03) | 6.17× (6.03–6.31) | 2.26× (2.20–2.31) | 2.28× (2.23–2.34) | 1.05× (1.03–1.06) | 0.77× (0.75–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.63) | 2.31× (2.24–2.39) | 4.46× (4.34–4.58) | 2.90× (2.80–3.01) | 5.89× (5.69–6.10) | 2.14× (2.05–2.23) | 2.16× (2.08–2.25) | 0.99× (0.97–1.02) | 0.73× (0.70–0.76) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.64× (2.54–2.74) | 4.46× (4.33–4.60) | 2.96× (2.88–3.04) | 6.03× (5.84–6.22) | 2.17× (2.10–2.24) | 2.19× (2.12–2.27) | 1.00× (0.97–1.04) | 0.74× (0.72–0.76) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 1.80× (1.80–1.81) | 3.96× (3.94–3.98) | 4.47× (4.46–4.49) | 4.61× (4.58–4.64) | 4.75× (4.72–4.78) | 4.98× (4.96–5.01) | 4.93× (4.91–4.95) | 1.46× (1.45–1.47) | 1.37× (1.37–1.37) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 1.64× (1.62–1.66) | 3.65× (3.59–3.72) | 4.25× (4.20–4.29) | 4.51× (4.46–4.56) | 4.22× (4.17–4.27) | 4.31× (4.25–4.36) | 4.26× (4.21–4.32) | 1.26× (1.25–1.27) | 1.18× (1.17–1.20) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 1.80× (1.76–1.83) | 3.21× (3.15–3.26) | 3.64× (3.55–3.73) | 3.99× (3.91–4.07) | 3.70× (3.62–3.78) | 2.84× (2.78–2.90) | 2.81× (2.75–2.87) | 0.83× (0.81–0.85) | 0.78× (0.77–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.28× (0.26–0.29) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.97× (10.96–10.98) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.35) | 0.56× (0.47–0.68) | 0.69× (0.66–0.72) | 0.59× (0.56–0.62) | 5.49× (5.27–5.71) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.34–0.36) | 0.55× (0.47–0.65) | 0.62× (0.60–0.63) | 0.54× (0.52–0.56) | 3.61× (3.51–3.70) | 0.09× (0.08–0.09) | 0.09× (0.09–0.09) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.40–0.44) | 1.71× (1.70–1.71) | 0.66× (0.65–0.66) | 8.03× (8.00–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.65× (0.62–0.68) | 0.64× (0.58–0.70) | 1.57× (1.50–1.64) | 0.79× (0.75–0.83) | 5.68× (5.42–5.96) | 0.46× (0.44–0.48) | 0.49× (0.46–0.51) | 0.52× (0.50–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.67) | 0.65× (0.59–0.71) | 1.14× (1.09–1.19) | 0.70× (0.67–0.73) | 3.38× (3.25–3.52) | 0.26× (0.25–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.18× (5.15–5.20) | 0.33× (0.33–0.33) | 0.35× (0.34–0.36) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.48–0.53) | 0.40× (0.36–0.43) | 0.91× (0.86–0.96) | 0.70× (0.65–0.75) | 3.14× (2.98–3.31) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.60–0.66) | 0.45× (0.43–0.46) | 0.86× (0.83–0.88) | 0.79× (0.76–0.82) | 2.49× (2.41–2.57) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.48× (0.45–0.51) | 0.38× (0.35–0.41) | 0.86× (0.81–0.90) | 0.72× (0.68–0.77) | 2.92× (2.77–3.07) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.50× (0.46–0.53) | 0.83× (0.82–0.84) | 0.84× (0.81–0.87) | 2.38× (2.35–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.38) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.77) | 5.32× (5.30–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.66–0.66) | 6.37× (6.36–6.38) | 0.39× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 34 MB | 0.48× (0.47–0.50) | 0.43× (0.39–0.48) | 1.14× (1.11–1.18) | 0.74× (0.72–0.76) | 3.58× (3.49–3.67) | 0.22× (0.22–0.23) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 56 MB | 0.49× (0.47–0.51) | 0.43× (0.39–0.46) | 0.92× (0.89–0.94) | 0.75× (0.72–0.77) | 2.34× (2.29–2.39) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 516 | 1.95 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 931 | 2.10 | 1.08× (1.07–1.09) | 90% | 1.05× (1.04–1.07) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,104 | 3.21 | 1.65× (1.62–1.68) | 54% | 1.48× (1.47–1.48) | 1.64× (1.62–1.66) |

**Best:** mibz2 at 4 threads (1,104 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (337 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.21, lbzip2 1t 0.04, lbzip2 2t 0.30, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.51 | 0.02 | 0.08 | 0.190 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2700 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 10.9% |
| MTF + RLE2 | 1.13 | 15.3% |
| T-vector build | 1.08 | 14.6% |
| walk (the one each build uses) | 4.10 | 55.6% |
| of which: segment assembly | 0.21 | 2.9% |
| RLE1 | 0.17 | 2.3% |
| CRC | 0.10 | 1.3% |
| **total** | 7.38 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.368 | 0.932 | (in its total) | 2.53× (2.53–2.53) |  |
| walk | 1.441 | 2.714 | (in its total) | 1.89× (1.88–1.89) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.072 | (in its total) | 0.91× (0.91–0.92) |  |
| inverse BWT, total | 1.889 | 3.718 | 2.002 | 1.97× (1.97–1.97) | 1.06× (1.06–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m09s ± 6m49s |
| 4 threads | 27m34s ± 3m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m24s ± 2m36s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 18m24s ± 2m36s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.41 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,519,058 | 12,810,756 | \`6ae766e8f83bdb21…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 2700.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
