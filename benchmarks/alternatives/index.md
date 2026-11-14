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

The latest record: [`f5c7809`](<https://github.com/lmmx/mibz2/commit/f5c7809fdb06e317e5d13a6c38ca953fb6396d9b>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43615219266>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.419 s | 500 | 7.8 | 3.22× (3.19–3.25) | 7.33× (7.26–7.39) | 7.83× (7.76–7.90) | 8.64× (8.57–8.70) | 7.84× (7.78–7.91) | 8.19× (8.13–8.27) | 7.98× (7.91–8.06) | 2.51× (2.49–2.53) | 2.23× (2.21–2.25) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.245 s | 856 | 8.6 | 3.08× (3.03–3.13) | 6.63× (6.48–6.77) | 7.97× (7.83–8.12) | 7.93× (7.80–8.07) | 7.19× (7.07–7.31) | 14.12× (13.85–14.40) | 13.76× (13.52–14.01) | 4.33× (4.25–4.41) | 3.84× (3.77–3.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.206 s | 1,019 | 13.9 | 3.09× (3.05–3.14) | 5.61× (5.53–5.71) | 6.66× (6.55–6.78) | 6.57× (6.46–6.69) | 6.42× (6.31–6.53) | 16.75× (16.46–17.05) | 16.32× (16.03–16.62) | 5.13× (5.04–5.23) | 4.56× (4.48–4.64) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,235 | 3.5 | 0.64× (0.63–0.66) | 3.12× (3.03–3.20) | 4.44× (4.32–4.56) | 3.73× (3.65–3.81) | 6.35× (6.21–6.50) | 2.83× (2.76–2.90) | 2.86× (2.80–2.93) | 1.13× (1.09–1.16) | 1.26× (1.23–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,198 | 3.6 | 0.56× (0.55–0.57) | 3.06× (3.01–3.12) | 6.17× (6.07–6.29) | 3.71× (3.64–3.78) | 6.30× (6.18–6.42) | 2.77× (2.73–2.82) | 2.81× (2.77–2.85) | 1.11× (1.08–1.13) | 1.24× (1.22–1.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,180 | 3.6 | 0.55× (0.54–0.56) | 3.00× (2.94–3.06) | 5.96× (5.71–6.22) | 3.62× (3.55–3.69) | 6.16× (6.04–6.27) | 2.71× (2.65–2.77) | 2.74× (2.69–2.80) | 1.08× (1.05–1.11) | 1.21× (1.18–1.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 637 | 6.4 | 2.87× (2.83–2.91) | 3.29× (3.25–3.33) | 3.99× (3.95–4.03) | 4.30× (4.25–4.35) | 4.71× (4.66–4.76) | 3.17× (3.13–3.20) | 2.93× (2.90–2.97) | 1.97× (1.93–2.00) | 2.13× (2.00–2.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 956 | 7.8 | 2.20× (1.97–2.44) | 2.80× (2.49–3.15) | 3.79× (3.44–4.16) | 3.46× (3.10–3.85) | 4.63× (4.16–5.15) | 4.41× (3.97–4.89) | 4.08× (3.68–4.53) | 2.73× (2.47–3.02) | 2.97× (2.77–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 966 | 14.0 | 1.93× (1.83–2.03) | 2.07× (1.97–2.17) | 2.97× (2.71–3.26) | 2.45× (2.35–2.55) | 4.17× (3.97–4.39) | 4.74× (4.50–4.98) | 4.39× (4.18–4.61) | 2.94× (2.82–3.06) | 3.19× (3.01–3.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 s | 70 | 55.7 | 1.37× (1.37–1.38) | 3.02× (3.00–3.04) | 2.77× (2.72–2.83) | 2.79× (2.79–2.80) | 3.11× (3.09–3.12) | 3.41× (3.39–3.42) | 3.54× (3.53–3.55) | 1.26× (1.26–1.26) | 1.25× (1.24–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 61.4 | 1.32× (1.28–1.35) | 2.99× (2.73–3.28) | 2.80× (2.72–2.87) | 3.41× (3.32–3.49) | 3.21× (3.15–3.28) | 5.64× (5.52–5.77) | 5.86× (5.73–6.00) | 2.09× (2.04–2.14) | 2.06× (2.02–2.11) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 83.9 | 1.41× (1.37–1.45) | 2.73× (2.65–2.81) | 2.65× (2.55–2.75) | 3.21× (3.13–3.29) | 3.26× (3.19–3.34) | 6.89× (6.72–7.07) | 7.17× (6.99–7.35) | 2.55× (2.49–2.62) | 2.52× (2.47–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 s | 76 | 51.4 | 1.42× (1.41–1.43) | 3.39× (3.36–3.41) | 2.88× (2.85–2.91) | 2.94× (2.92–2.97) | 3.34× (3.29–3.39) | 4.05× (4.01–4.09) | 4.60× (4.56–4.64) | 1.47× (1.45–1.48) | 1.53× (1.51–1.54) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.076 s | 126 | 56.6 | 1.29× (1.23–1.34) | 3.03× (2.91–3.15) | 2.79× (2.68–2.91) | 3.70× (3.54–3.86) | 3.27× (3.14–3.40) | 6.62× (6.35–6.89) | 7.52× (7.22–7.84) | 2.40× (2.30–2.50) | 2.50× (2.40–2.60) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 78.1 | 1.45× (1.40–1.49) | 2.99× (2.89–3.10) | 2.70× (2.59–2.82) | 3.57× (3.46–3.69) | 3.25× (3.18–3.31) | 8.10× (7.91–8.28) | 9.21× (8.99–9.42) | 2.93× (2.86–3.00) | 3.05× (2.99–3.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 922 | 4.9 | 0.70× (0.69–0.72) | 2.82× (2.76–2.88) | 3.98× (3.78–4.18) | 3.19× (3.12–3.26) | 6.92× (6.79–7.05) | 2.44× (2.40–2.48) | 2.48× (2.44–2.53) | 1.04× (1.01–1.06) | 0.96× (0.94–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 862 | 5.3 | 0.53× (0.52–0.54) | 2.66× (2.62–2.70) | 5.40× (5.33–5.48) | 3.03× (2.98–3.08) | 6.52× (6.40–6.64) | 2.29× (2.26–2.33) | 2.33× (2.30–2.36) | 0.97× (0.95–1.00) | 0.90× (0.89–0.92) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 881 | 5.0 | 0.55× (0.54–0.55) | 2.72× (2.69–2.75) | 5.41× (5.21–5.61) | 3.12× (3.07–3.16) | 6.66× (6.55–6.77) | 2.34× (2.30–2.38) | 2.38× (2.35–2.41) | 0.99× (0.97–1.02) | 0.92× (0.91–0.94) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 0.057 s | 213 | 18.7 | 1.82× (1.77–1.87) | 4.18× (4.08–4.29) | 4.61× (4.48–4.74) | 4.69× (4.58–4.81) | 4.86× (4.74–4.98) | 5.05× (4.93–5.17) | 4.97× (4.84–5.10) | 1.47× (1.44–1.51) | 1.41× (1.37–1.45) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 0.037 s | 328 | 22.6 | 1.49× (1.42–1.58) | 3.51× (3.32–3.73) | 4.75× (4.45–5.06) | 4.69× (4.45–4.95) | 4.45× (4.21–4.70) | 7.64× (7.25–8.04) | 7.52× (7.15–7.91) | 2.23× (2.12–2.34) | 2.13× (2.02–2.24) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 0.033 s | 372 | 34.3 | 1.55× (1.51–1.59) | 2.87× (2.82–2.91) | 4.12× (4.04–4.19) | 4.16× (4.09–4.22) | 4.10× (4.02–4.19) | 8.86× (8.71–9.01) | 8.72× (8.56–8.87) | 2.58× (2.54–2.63) | 2.47× (2.43–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.21× (3.18–3.24) | 7.29× (7.23–7.35) | 7.79× (7.73–7.85) | 8.60× (8.54–8.67) | 7.81× (7.75–7.88) | 8.16× (8.09–8.23) | 7.95× (7.88–8.03) | 2.50× (2.48–2.53) | 2.21× (2.19–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.00× (2.96–3.03) | 6.85× (6.74–6.96) | 7.54× (7.46–7.62) | 8.23× (8.14–8.31) | 7.12× (7.04–7.20) | 7.36× (7.27–7.44) | 7.17× (7.09–7.25) | 2.26× (2.23–2.28) | 1.99× (1.97–2.02) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.92–3.02) | 6.04× (5.97–6.11) | 6.48× (6.41–6.56) | 7.07× (7.00–7.15) | 6.29× (6.23–6.36) | 4.59× (4.55–4.63) | 4.47× (4.42–4.53) | 1.41× (1.39–1.42) | 1.24× (1.23–1.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.66–0.69) | 2.85× (2.79–2.92) | 3.88× (3.81–3.96) | 3.46× (3.39–3.53) | 5.83× (5.70–5.95) | 2.65× (2.59–2.71) | 2.68× (2.62–2.74) | 1.11× (1.08–1.14) | 0.81× (0.79–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.62) | 2.82× (2.77–2.86) | 5.42× (5.34–5.50) | 3.41× (3.34–3.48) | 5.72× (5.61–5.83) | 2.56× (2.53–2.60) | 2.59× (2.56–2.63) | 1.08× (1.05–1.10) | 0.78× (0.77–0.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.11× (3.03–3.20) | 5.36× (5.23–5.49) | 3.41× (3.36–3.47) | 5.71× (5.62–5.81) | 2.57× (2.52–2.61) | 2.60× (2.56–2.64) | 1.08× (1.06–1.10) | 0.79× (0.77–0.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.77× (2.74–2.81) | 3.06× (3.03–3.09) | 3.61× (3.56–3.65) | 4.12× (4.07–4.17) | 4.51× (4.46–4.57) | 3.05× (3.02–3.09) | 2.83× (2.80–2.87) | 1.91× (1.88–1.95) | 1.90× (1.85–1.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.39–2.49) | 2.68× (2.62–2.74) | 3.27× (3.18–3.36) | 3.49× (3.41–3.56) | 3.84× (3.77–3.92) | 2.51× (2.46–2.56) | 2.33× (2.28–2.37) | 1.57× (1.53–1.62) | 1.56× (1.51–1.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.03–2.17) | 1.84× (1.76–1.92) | 2.28× (2.20–2.35) | 2.23× (2.16–2.31) | 2.36× (2.27–2.45) | 1.40× (1.35–1.45) | 1.30× (1.25–1.34) | 0.88× (0.84–0.91) | 0.87× (0.83–0.91) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.93× (2.91–2.94) | 2.70× (2.64–2.75) | 2.77× (2.76–2.78) | 3.08× (3.06–3.09) | 3.37× (3.36–3.38) | 3.50× (3.49–3.52) | 1.26× (1.25–1.26) | 1.23× (1.22–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.80× (2.75–2.85) | 2.55× (2.51–2.60) | 3.21× (3.16–3.27) | 2.86× (2.81–2.91) | 3.04× (2.98–3.09) | 3.16× (3.10–3.21) | 1.13× (1.11–1.15) | 1.11× (1.09–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.45–1.60) | 2.90× (2.83–2.96) | 2.50× (2.46–2.55) | 3.15× (3.07–3.23) | 2.88× (2.81–2.94) | 2.26× (2.21–2.31) | 2.35× (2.29–2.40) | 0.84× (0.82–0.86) | 0.82× (0.80–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.40–1.43) | 3.30× (3.28–3.33) | 2.82× (2.80–2.85) | 2.92× (2.89–2.94) | 3.31× (3.26–3.35) | 4.00× (3.97–4.04) | 4.55× (4.51–4.59) | 1.46× (1.45–1.47) | 1.46× (1.44–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.29–1.37) | 3.09× (3.01–3.18) | 2.67× (2.60–2.74) | 3.56× (3.45–3.66) | 3.03× (2.95–3.12) | 3.59× (3.49–3.70) | 4.09× (3.97–4.20) | 1.31× (1.27–1.35) | 1.31× (1.27–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.60–1.72) | 3.25× (3.12–3.38) | 2.50× (2.39–2.62) | 3.39× (3.29–3.50) | 3.04× (2.92–3.15) | 2.64× (2.55–2.73) | 3.00× (2.90–3.11) | 0.96× (0.93–1.00) | 0.96× (0.93–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.75) | 2.44× (2.40–2.48) | 3.42× (3.36–3.48) | 2.89× (2.82–2.95) | 6.12× (6.01–6.22) | 2.25× (2.21–2.28) | 2.28× (2.25–2.32) | 1.03× (1.01–1.05) | 0.76× (0.74–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.24× (2.19–2.28) | 4.33× (4.27–4.39) | 2.75× (2.71–2.79) | 5.68× (5.59–5.78) | 2.07× (2.04–2.10) | 2.10× (2.07–2.13) | 0.95× (0.93–0.97) | 0.70× (0.68–0.71) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.61) | 2.68× (2.57–2.79) | 4.52× (4.44–4.60) | 2.92× (2.88–2.97) | 5.99× (5.90–6.09) | 2.18× (2.15–2.22) | 2.22× (2.19–2.25) | 1.00× (0.98–1.03) | 0.74× (0.72–0.75) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 1.80× (1.75–1.84) | 3.98× (3.88–4.08) | 4.47× (4.35–4.59) | 4.59× (4.48–4.70) | 4.75× (4.63–4.87) | 4.93× (4.82–5.05) | 4.85× (4.73–4.98) | 1.46× (1.42–1.50) | 1.35× (1.31–1.38) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 1.60× (1.58–1.61) | 3.55× (3.50–3.60) | 4.14× (4.08–4.19) | 4.33× (4.29–4.38) | 4.08× (4.03–4.13) | 4.13× (4.10–4.17) | 4.07× (4.03–4.10) | 1.22× (1.21–1.23) | 1.13× (1.12–1.14) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 1.76× (1.70–1.81) | 3.19× (3.13–3.26) | 3.46× (3.40–3.52) | 3.73× (3.66–3.80) | 3.55× (3.48–3.63) | 2.71× (2.65–2.76) | 2.66× (2.61–2.72) | 0.80× (0.79–0.82) | 0.74× (0.73–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.23× (0.23–0.24) | 0.81× (0.81–0.81) | 0.64× (0.64–0.65) | 9.11× (9.09–9.14) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 79 MB | 0.24× (0.23–0.25) | 0.38× (0.33–0.43) | 0.50× (0.47–0.52) | 0.42× (0.40–0.44) | 4.00× (3.82–4.19) | 0.09× (0.09–0.10) | 0.10× (0.10–0.11) | 0.15× (0.15–0.16) | 0.08× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 125 MB | 0.24× (0.22–0.26) | 0.34× (0.28–0.42) | 0.42× (0.39–0.46) | 0.38× (0.35–0.41) | 2.52× (2.32–2.73) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.10× (0.09–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.17) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.30× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.01× (7.99–8.04) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.68) | 0.60× (0.55–0.66) | 1.59× (1.55–1.64) | 0.79× (0.77–0.81) | 5.76× (5.60–5.92) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.63–0.65) | 0.66× (0.58–0.75) | 1.15× (1.12–1.17) | 0.70× (0.68–0.73) | 3.43× (3.37–3.49) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.19–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.39× (0.37–0.42) | 0.88× (0.85–0.92) | 0.69× (0.65–0.73) | 3.04× (2.92–3.16) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.85× (0.83–0.88) | 0.79× (0.75–0.83) | 2.48× (2.40–2.56) | 0.15× (0.14–0.15) | 0.16× (0.15–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.36× (0.34–0.39) | 0.85× (0.82–0.88) | 0.70× (0.67–0.74) | 2.90× (2.79–3.00) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.61× (0.60–0.62) | 0.48× (0.46–0.51) | 0.82× (0.81–0.84) | 0.83× (0.82–0.85) | 2.37× (2.34–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.23) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.35–5.40) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.76) | 5.28× (5.26–5.30) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.65–0.66) | 6.40× (6.39–6.41) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 39 MB | 0.43× (0.42–0.44) | 0.39× (0.35–0.43) | 1.02× (0.97–1.06) | 0.64× (0.62–0.67) | 3.17× (3.06–3.27) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.15× (0.15–0.16) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 59 MB | 0.47× (0.46–0.49) | 0.48× (0.45–0.52) | 0.87× (0.85–0.89) | 0.68× (0.65–0.72) | 2.23× (2.18–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 500 | 2.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 856 | 2.24 | 1.11× (1.10–1.12) | 86% | 1.04× (1.03–1.06) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,019 | 3.60 | 1.78× (1.76–1.80) | 51% | 1.47× (1.46–1.49) | 1.65× (1.63–1.66) |

**Best:** mibz2 at 4 threads (1,019 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (329 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.13, lbzip2 1t 0.04, lbzip2 2t 0.23, lbzip2 4t 0.29, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.58 | 0.02 | 0.08 | 0.206 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3864 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 10.7% |
| MTF + RLE2 | 1.63 | 15.4% |
| T-vector build | 1.55 | 14.6% |
| walk (the one each build uses) | 5.90 | 55.7% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.59 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.369 | 0.934 | (in its total) | 2.53× (2.53–2.54) |  |
| walk | 1.441 | 2.754 | (in its total) | 1.91× (1.91–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.072 | (in its total) | 0.89× (0.89–0.90) |  |
| inverse BWT, total | 1.891 | 3.761 | 1.976 | 1.99× (1.99–1.99) | 1.05× (1.04–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m02s |
| 4 threads | 29m59s ± 4m14s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m59s ± 2m58s |
| the scanner alone (its serial part) | 11m12s ± 30.2s |
| the workers alone, over 4 | 20m59s ± 2m58s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 2.75 ns a byte of output at 4 threads (CPU, summed over them), 1.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,417,543 | 12,200,394 | \`a6b24102e87d445d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3864.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
