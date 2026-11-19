# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-623/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`5298b23`](<https://github.com/lmmx/mibz2/commit/5298b23c7c89048c8986979e19b67c840397f0a0>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44468220343>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.390 s | 538 | 7.3 | 3.42× (3.41–3.43) | 7.92× (7.89–7.95) | 8.42× (8.38–8.46) | 9.30× (9.28–9.33) | 8.46× (8.42–8.49) | 8.83× (8.80–8.85) | 8.57× (8.55–8.59) | 2.70× (2.69–2.70) | 2.41× (2.40–2.42) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 956 | 7.9 | 3.43× (3.22–3.65) | 7.56× (7.30–7.84) | 8.89× (8.76–9.01) | 8.80× (8.67–8.92) | 7.98× (7.88–8.07) | 15.65× (15.48–15.82) | 15.19× (15.02–15.36) | 4.78× (4.74–4.83) | 4.27× (4.23–4.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.185 s | 1,136 | 12.2 | 3.41× (3.39–3.44) | 6.29× (6.21–6.37) | 7.43× (7.30–7.57) | 7.30× (7.22–7.38) | 7.17× (7.09–7.24) | 18.70× (18.51–18.89) | 18.15× (17.96–18.34) | 5.72× (5.66–5.78) | 5.10× (5.07–5.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,246 | 3.5 | 0.63× (0.61–0.65) | 3.22× (3.04–3.42) | 4.60× (4.33–4.89) | 3.78× (3.59–3.97) | 6.52× (6.21–6.84) | 2.87× (2.71–3.04) | 2.91× (2.75–3.08) | 1.13× (1.08–1.18) | 1.30× (1.23–1.38) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,208 | 3.6 | 0.56× (0.54–0.59) | 3.19× (3.01–3.37) | 6.39× (6.08–6.70) | 3.76× (3.57–3.96) | 6.41× (6.05–6.79) | 2.84× (2.69–3.00) | 2.88× (2.73–3.04) | 1.12× (1.07–1.16) | 1.29× (1.21–1.36) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,201 | 3.5 | 0.54× (0.54–0.55) | 3.07× (3.02–3.12) | 5.94× (5.71–6.18) | 3.64× (3.58–3.70) | 6.18× (6.07–6.29) | 2.73× (2.68–2.78) | 2.76× (2.71–2.81) | 1.07× (1.04–1.11) | 1.23× (1.21–1.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 636 | 6.4 | 2.60× (2.52–2.68) | 3.34× (3.24–3.45) | 3.96× (3.85–4.07) | 4.29× (4.17–4.43) | 4.75× (4.63–4.87) | 3.16× (3.07–3.26) | 2.90× (2.82–2.98) | 2.00× (1.94–2.06) | 2.01× (1.95–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,018 | 7.7 | 2.16× (1.82–2.56) | 3.00× (2.52–3.58) | 4.06× (3.48–4.73) | 3.53× (3.02–4.13) | 4.82× (4.10–5.67) | 4.57× (3.90–5.36) | 4.20× (3.59–4.91) | 2.88× (2.45–3.39) | 2.90× (2.47–3.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,026 | 12.8 | 1.98× (1.88–2.09) | 2.25× (2.15–2.36) | 3.39× (3.22–3.57) | 2.73× (2.51–2.97) | 4.47× (4.26–4.69) | 5.12× (4.86–5.39) | 4.70× (4.47–4.94) | 3.23× (3.09–3.38) | 3.25× (3.08–3.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 s | 70 | 56.2 | 1.35× (1.35–1.36) | 2.98× (2.95–3.01) | 2.71× (2.68–2.74) | 2.80× (2.79–2.81) | 3.11× (3.10–3.12) | 3.41× (3.40–3.43) | 3.54× (3.53–3.55) | 1.25× (1.25–1.26) | 1.25× (1.25–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 61.5 | 1.28× (1.24–1.31) | 2.93× (2.62–3.27) | 2.77× (2.68–2.86) | 3.40× (3.28–3.52) | 3.18× (3.08–3.27) | 5.60× (5.44–5.77) | 5.80× (5.64–5.98) | 2.05× (2.00–2.12) | 2.05× (1.99–2.11) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 84.2 | 1.37× (1.33–1.42) | 2.69× (2.58–2.80) | 2.58× (2.48–2.68) | 3.19× (3.07–3.31) | 3.21× (3.10–3.32) | 6.82× (6.58–7.08) | 7.07× (6.81–7.33) | 2.50× (2.42–2.59) | 2.50× (2.41–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 s | 76 | 51.9 | 1.40× (1.39–1.41) | 3.37× (3.35–3.40) | 2.87× (2.84–2.90) | 2.95× (2.93–2.97) | 3.34× (3.32–3.35) | 4.04× (4.02–4.07) | 4.33× (4.31–4.35) | 1.46× (1.45–1.47) | 1.52× (1.50–1.54) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 129 | 56.9 | 1.27× (1.19–1.35) | 3.04× (2.84–3.25) | 2.79× (2.62–2.96) | 3.71× (3.49–3.95) | 3.23× (3.04–3.43) | 6.62× (6.22–7.04) | 7.08× (6.66–7.52) | 2.39× (2.25–2.55) | 2.48× (2.34–2.64) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 156 | 79.5 | 1.40× (1.36–1.44) | 2.95× (2.82–3.07) | 2.70× (2.59–2.82) | 3.63× (3.49–3.79) | 3.22× (3.11–3.34) | 8.15× (7.89–8.42) | 8.72× (8.45–9.01) | 2.95× (2.85–3.04) | 3.06× (2.96–3.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 943 | 4.7 | 0.67× (0.64–0.70) | 2.97× (2.86–3.08) | 4.12× (3.96–4.30) | 3.27× (3.18–3.36) | 7.14× (6.92–7.38) | 2.50× (2.42–2.57) | 2.53× (2.46–2.61) | 1.06× (1.03–1.08) | 1.01× (0.98–1.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 889 | 5.2 | 0.53× (0.51–0.54) | 2.73× (2.64–2.82) | 5.58× (5.39–5.78) | 3.04× (2.95–3.14) | 6.61× (6.44–6.79) | 2.30× (2.23–2.36) | 2.33× (2.26–2.39) | 0.97× (0.94–1.00) | 0.93× (0.90–0.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 877 | 5.0 | 0.54× (0.52–0.55) | 2.76× (2.68–2.84) | 5.41× (5.13–5.70) | 3.12× (3.03–3.21) | 6.72× (6.57–6.87) | 2.31× (2.25–2.37) | 2.34× (2.28–2.40) | 0.98× (0.94–1.01) | 0.93× (0.91–0.96) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.060 s | 218 | 18.4 | 1.84× (1.83–1.85) | 4.23× (4.20–4.25) | 4.72× (4.67–4.78) | 4.76× (4.72–4.79) | 4.91× (4.88–4.94) | 5.15× (5.13–5.17) | 5.02× (5.00–5.04) | 1.50× (1.49–1.50) | 1.52× (1.52–1.53) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.038 s | 341 | 21.4 | 1.59× (1.53–1.66) | 3.59× (3.42–3.78) | 4.55× (4.38–4.73) | 4.68× (4.55–4.82) | 4.73× (4.57–4.89) | 7.97× (7.72–8.23) | 7.77× (7.53–8.03) | 2.32× (2.25–2.40) | 2.36× (2.28–2.44) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.033 s | 397 | 31.5 | 1.64× (1.60–1.69) | 3.16× (3.04–3.29) | 4.12× (3.99–4.24) | 4.17× (4.08–4.27) | 4.57× (4.45–4.69) | 9.33× (9.13–9.54) | 9.10× (8.92–9.29) | 2.72× (2.66–2.78) | 2.76× (2.71–2.82) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.41× (3.40–3.42) | 7.88× (7.85–7.91) | 8.38× (8.34–8.42) | 9.27× (9.24–9.29) | 8.42× (8.39–8.46) | 8.79× (8.77–8.82) | 8.53× (8.51–8.55) | 2.69× (2.69–2.70) | 2.39× (2.38–2.41) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.26× (3.24–3.28) | 7.66× (7.59–7.73) | 8.33× (8.30–8.36) | 9.09× (9.04–9.14) | 7.89× (7.84–7.93) | 8.13× (8.09–8.17) | 7.89× (7.86–7.92) | 2.49× (2.48–2.50) | 2.21× (2.20–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.35× (3.31–3.40) | 6.89× (6.81–6.98) | 7.31× (7.20–7.41) | 8.00× (7.89–8.12) | 7.17× (7.07–7.28) | 5.21× (5.14–5.29) | 5.06× (4.99–5.14) | 1.60× (1.57–1.62) | 1.42× (1.40–1.44) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.65–0.69) | 2.94× (2.78–3.11) | 3.95× (3.75–4.15) | 3.51× (3.34–3.70) | 6.01× (5.71–6.31) | 2.70× (2.55–2.86) | 2.73× (2.58–2.89) | 1.12× (1.07–1.17) | 0.84× (0.80–0.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.57–0.63) | 2.89× (2.74–3.05) | 5.52× (5.26–5.80) | 3.45× (3.27–3.63) | 5.80× (5.47–6.15) | 2.62× (2.48–2.76) | 2.65× (2.51–2.79) | 1.09× (1.04–1.13) | 0.81× (0.78–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.59) | 3.20× (3.11–3.29) | 5.37× (5.25–5.50) | 3.45× (3.39–3.50) | 5.76× (5.66–5.87) | 2.60× (2.54–2.65) | 2.63× (2.57–2.68) | 1.08× (1.04–1.11) | 0.81× (0.79–0.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.44–2.60) | 3.09× (3.00–3.17) | 3.61× (3.50–3.71) | 4.13× (4.01–4.25) | 4.56× (4.45–4.68) | 3.05× (2.97–3.14) | 2.81× (2.73–2.88) | 1.94× (1.89–2.00) | 1.88× (1.83–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.24–2.44) | 2.80× (2.68–2.92) | 3.40× (3.26–3.54) | 3.63× (3.47–3.79) | 4.00× (3.85–4.16) | 2.60× (2.48–2.73) | 2.39× (2.28–2.51) | 1.66× (1.58–1.74) | 1.60× (1.53–1.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.92–2.23) | 1.97× (1.88–2.06) | 2.46× (2.36–2.57) | 2.37× (2.25–2.50) | 2.53× (2.42–2.65) | 1.51× (1.44–1.58) | 1.38× (1.32–1.45) | 0.96× (0.92–1.00) | 0.93× (0.89–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.92× (2.91–2.93) | 2.66× (2.65–2.68) | 2.77× (2.76–2.79) | 3.08× (3.07–3.09) | 3.38× (3.37–3.39) | 3.50× (3.49–3.52) | 1.25× (1.25–1.25) | 1.22× (1.22–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.80× (2.76–2.85) | 2.56× (2.52–2.60) | 3.26× (3.21–3.31) | 2.87× (2.83–2.91) | 3.06× (3.01–3.11) | 3.17× (3.12–3.22) | 1.13× (1.11–1.15) | 1.11× (1.09–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.49–1.57) | 2.84× (2.73–2.97) | 2.43× (2.36–2.50) | 3.18× (3.11–3.25) | 2.86× (2.79–2.93) | 2.26× (2.20–2.31) | 2.34× (2.28–2.40) | 0.83× (0.81–0.85) | 0.82× (0.80–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.39–1.40) | 3.29× (3.27–3.30) | 2.81× (2.79–2.82) | 2.93× (2.91–2.94) | 3.31× (3.29–3.32) | 4.00× (3.98–4.03) | 4.28× (4.26–4.30) | 1.46× (1.45–1.46) | 1.45× (1.44–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.27–1.34) | 3.09× (3.00–3.18) | 2.64× (2.58–2.71) | 3.55× (3.47–3.64) | 3.03× (2.95–3.10) | 3.59× (3.50–3.68) | 3.84× (3.75–3.93) | 1.30× (1.27–1.34) | 1.30× (1.26–1.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.55–1.63) | 3.21× (3.13–3.30) | 2.42× (2.34–2.52) | 3.39× (3.32–3.46) | 2.97× (2.90–3.04) | 2.61× (2.55–2.67) | 2.79× (2.73–2.86) | 0.95× (0.93–0.97) | 0.94× (0.92–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.69–0.74) | 2.53× (2.45–2.61) | 3.53× (3.42–3.66) | 2.98× (2.90–3.07) | 6.38× (6.16–6.61) | 2.32× (2.24–2.40) | 2.34× (2.27–2.42) | 1.05× (1.02–1.08) | 0.78× (0.77–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.27× (2.20–2.33) | 4.41× (4.26–4.55) | 2.79× (2.70–2.87) | 5.81× (5.65–5.97) | 2.08× (2.02–2.15) | 2.11× (2.05–2.17) | 0.94× (0.91–0.97) | 0.71× (0.68–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.60) | 2.68× (2.58–2.79) | 4.50× (4.39–4.61) | 2.93× (2.84–3.01) | 6.08× (5.94–6.22) | 2.16× (2.10–2.22) | 2.18× (2.13–2.24) | 0.98× (0.94–1.01) | 0.73× (0.71–0.75) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.82× (1.81–1.83) | 4.05× (4.02–4.07) | 4.53× (4.51–4.56) | 4.66× (4.63–4.70) | 4.81× (4.78–4.85) | 5.04× (5.01–5.07) | 4.91× (4.89–4.94) | 1.49× (1.48–1.49) | 1.38× (1.38–1.39) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.64× (1.61–1.66) | 3.62× (3.55–3.69) | 4.27× (4.18–4.37) | 4.54× (4.47–4.61) | 4.22× (4.15–4.29) | 4.31× (4.24–4.38) | 4.20× (4.14–4.27) | 1.27× (1.25–1.29) | 1.18× (1.17–1.20) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.81× (1.76–1.87) | 3.31× (3.25–3.38) | 3.75× (3.67–3.83) | 4.12× (4.05–4.19) | 3.78× (3.72–3.83) | 2.92× (2.88–2.96) | 2.85× (2.81–2.89) | 0.86× (0.85–0.87) | 0.80× (0.79–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.28× (0.27–0.30) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 10.99× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.27–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.57× (0.51–0.64) | 0.69× (0.66–0.71) | 0.58× (0.56–0.60) | 5.50× (5.33–5.68) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.53× (0.43–0.64) | 0.59× (0.57–0.61) | 0.53× (0.51–0.55) | 3.49× (3.39–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.02× (8.00–8.05) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.62× (0.56–0.69) | 1.62× (1.58–1.65) | 0.81× (0.79–0.83) | 5.84× (5.71–5.97) | 0.47× (0.46–0.49) | 0.50× (0.48–0.51) | 0.53× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.62–0.65) | 0.66× (0.62–0.71) | 1.13× (1.10–1.16) | 0.70× (0.68–0.72) | 3.39× (3.31–3.47) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.15–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.52) | 0.38× (0.35–0.40) | 0.89× (0.85–0.94) | 0.70× (0.65–0.75) | 3.07× (2.92–3.23) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.67) | 0.46× (0.46–0.47) | 0.88× (0.86–0.90) | 0.82× (0.79–0.85) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.42) | 0.87× (0.84–0.91) | 0.71× (0.68–0.74) | 2.99× (2.88–3.09) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.28–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.44–0.50) | 0.81× (0.80–0.82) | 0.83× (0.81–0.85) | 2.33× (2.31–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.19–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.31× (5.29–5.33) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.42–1.43) | 0.67× (0.66–0.67) | 6.46× (6.45–6.47) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.46) | 0.36× (0.33–0.40) | 1.04× (1.00–1.08) | 0.68× (0.66–0.70) | 3.28× (3.19–3.38) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.45× (0.41–0.49) | 0.92× (0.90–0.94) | 0.77× (0.75–0.78) | 2.35× (2.29–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 538 | 1.87 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 956 | 2.02 | 1.08× (1.08–1.09) | 89% | 1.05× (1.04–1.06) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,136 | 3.14 | 1.69× (1.66–1.71) | 53% | 1.48× (1.47–1.48) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (1,136 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (334 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.22, lbzip2 1t 0.04, lbzip2 2t 0.30, lbzip2 4t 0.29, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.49 | 0.02 | 0.08 | 0.185 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3900 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 10.7% |
| MTF + RLE2 | 1.67 | 15.6% |
| T-vector build | 1.52 | 14.2% |
| walk (the one each build uses) | 5.97 | 55.8% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.69 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.369 | 0.936 | (in its total) | 2.51× (2.46–2.57) |  |
| walk | 1.447 | 2.729 | (in its total) | 1.88× (1.87–1.89) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.072 | (in its total) | 0.91× (0.90–0.91) |  |
| inverse BWT, total | 1.896 | 3.737 | 1.966 | 1.96× (1.94–1.98) | 1.03× (1.02–1.04) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m39s ± 6m27s |
| 4 threads | 26m45s ± 3m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m43s ± 2m29s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 17m43s ± 2m29s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.32 ns a byte of output at 4 threads (CPU, summed over them), 1.51 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3900.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
