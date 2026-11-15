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

The latest record: [`dc94c3c`](<https://github.com/lmmx/mibz2/commit/dc94c3c77f49787bc5c38b18d9abc14a10bc85db>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43735078441>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.445 s | 471 | 7.7 | 3.12× (3.10–3.13) | 6.95× (6.90–7.00) | 7.34× (7.28–7.39) | 8.31× (8.24–8.38) | 7.46× (7.41–7.52) | 7.93× (7.89–7.98) | 7.63× (7.58–7.68) | 2.42× (2.40–2.44) | 2.22× (2.22–2.23) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.249 s | 843 | 8.2 | 3.02× (2.81–3.25) | 6.43× (5.93–6.96) | 7.61× (7.01–8.25) | 7.71× (7.16–8.30) | 6.91× (6.42–7.44) | 13.69× (12.71–14.75) | 13.17× (12.24–14.18) | 4.18× (3.87–4.50) | 3.84× (3.56–4.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.203 s | 1,034 | 12.7 | 3.32× (3.27–3.38) | 5.77× (5.70–5.84) | 6.62× (6.50–6.73) | 6.58× (6.47–6.68) | 6.52× (6.42–6.62) | 17.39× (17.19–17.59) | 16.73× (16.50–16.97) | 5.30× (5.23–5.38) | 4.87× (4.81–4.94) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,399 | 2.8 | 0.66× (0.60–0.71) | 4.10× (3.70–4.53) | 5.24× (4.83–5.68) | 4.38× (3.98–4.82) | 7.35× (6.69–8.07) | 3.30× (2.99–3.65) | 3.30× (2.98–3.64) | 1.17× (1.11–1.23) | 1.38× (1.25–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,096 | 3.5 | 0.54× (0.52–0.57) | 2.85× (2.73–2.98) | 5.74× (5.49–6.00) | 3.68× (3.52–3.86) | 6.13× (5.86–6.42) | 2.75× (2.63–2.87) | 2.74× (2.63–2.87) | 0.98× (0.91–1.05) | 1.15× (1.10–1.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,140 | 3.4 | 0.56× (0.54–0.57) | 3.07× (2.88–3.28) | 5.84× (5.64–6.04) | 3.78× (3.66–3.91) | 6.24× (6.02–6.46) | 2.80× (2.71–2.89) | 2.79× (2.70–2.89) | 0.99× (0.93–1.06) | 1.17× (1.13–1.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 661 | 5.7 | 3.05× (2.95–3.16) | 3.70× (3.58–3.82) | 4.07× (3.93–4.21) | 4.75× (4.60–4.91) | 5.18× (5.02–5.34) | 3.88× (3.76–4.01) | 3.16× (3.06–3.25) | 2.21× (2.14–2.28) | 2.35× (2.28–2.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 987 | 6.9 | 2.45× (2.25–2.66) | 3.06× (2.80–3.36) | 4.33× (3.90–4.79) | 4.03× (3.66–4.45) | 5.35× (4.91–5.83) | 5.77× (5.26–6.33) | 4.69× (4.30–5.12) | 3.29× (3.01–3.58) | 3.50× (3.21–3.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,034 | 12.2 | 2.32× (2.24–2.41) | 2.21× (2.13–2.28) | 3.43× (3.32–3.56) | 2.72× (2.61–2.84) | 4.64× (4.49–4.80) | 6.19× (5.96–6.43) | 5.03× (4.87–5.21) | 3.53× (3.40–3.65) | 3.75× (3.62–3.90) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.125 s | 64 | 56.7 | 1.34× (1.33–1.35) | 2.76× (2.73–2.79) | 2.47× (2.45–2.50) | 2.58× (2.56–2.60) | 2.85× (2.83–2.88) | 3.21× (3.18–3.23) | 3.27× (3.24–3.31) | 1.27× (1.26–1.28) | 1.30× (1.28–1.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.076 s | 105 | 62.3 | 1.27× (1.22–1.32) | 2.73× (2.58–2.89) | 2.54× (2.38–2.72) | 3.17× (3.01–3.33) | 2.93× (2.82–3.05) | 5.30× (5.08–5.53) | 5.41× (5.18–5.66) | 2.10× (2.01–2.18) | 2.14× (2.05–2.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 134 | 83.4 | 1.44× (1.40–1.47) | 2.61× (2.53–2.70) | 2.45× (2.37–2.52) | 3.03× (2.98–3.08) | 3.06× (2.99–3.14) | 6.71× (6.56–6.85) | 6.85× (6.71–7.00) | 2.65× (2.59–2.71) | 2.71× (2.65–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.138 s | 70 | 52.1 | 1.40× (1.39–1.41) | 3.16× (3.14–3.18) | 2.64× (2.61–2.66) | 2.79× (2.77–2.81) | 3.12× (3.10–3.13) | 3.91× (3.87–3.95) | 4.14× (4.11–4.18) | 1.49× (1.48–1.50) | 1.54× (1.53–1.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 115 | 58.5 | 1.30× (1.21–1.40) | 2.91× (2.71–3.12) | 2.58× (2.42–2.74) | 3.52× (3.31–3.74) | 3.02× (2.85–3.20) | 6.39× (6.03–6.78) | 6.77× (6.40–7.16) | 2.44× (2.30–2.58) | 2.52× (2.37–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 140 | 78.7 | 1.46× (1.41–1.50) | 2.80× (2.73–2.88) | 2.48× (2.41–2.56) | 3.34× (3.24–3.44) | 3.03× (2.95–3.10) | 7.91× (7.74–8.09) | 8.38× (8.18–8.59) | 3.02× (2.95–3.09) | 3.12× (3.04–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 3.9 | 0.68× (0.64–0.73) | 3.23× (2.97–3.52) | 4.54× (4.25–4.85) | 3.65× (3.38–3.94) | 7.79× (7.18–8.46) | 2.74× (2.51–2.98) | 2.75× (2.53–2.99) | 1.10× (1.00–1.20) | 1.04× (0.95–1.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 870 | 4.6 | 0.56× (0.52–0.61) | 2.70× (2.50–2.91) | 5.45× (5.06–5.87) | 3.28× (3.04–3.55) | 6.95× (6.39–7.55) | 2.41× (2.25–2.59) | 2.43× (2.26–2.61) | 0.97× (0.90–1.04) | 0.92× (0.85–0.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 858 | 4.7 | 0.56× (0.54–0.59) | 2.88× (2.66–3.11) | 5.34× (5.05–5.64) | 3.28× (3.11–3.45) | 6.79× (6.48–7.10) | 2.36× (2.23–2.49) | 2.37× (2.25–2.50) | 0.94× (0.89–1.01) | 0.89× (0.85–0.94) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.061 s | 207 | 17.7 | 1.83× (1.81–1.85) | 4.10× (4.04–4.15) | 4.50× (4.45–4.56) | 4.73× (4.66–4.79) | 4.88× (4.80–4.95) | 5.11× (5.03–5.19) | 5.00× (4.94–5.07) | 1.52× (1.50–1.53) | 1.49× (1.47–1.51) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.042 s | 303 | 21.6 | 1.48× (1.40–1.56) | 3.30× (3.10–3.51) | 4.06× (3.86–4.28) | 4.77× (4.55–5.01) | 4.48× (4.30–4.67) | 7.61× (7.22–8.01) | 7.44× (7.09–7.81) | 2.26× (2.15–2.37) | 2.22× (2.11–2.33) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.034 s | 372 | 31.2 | 1.62× (1.57–1.67) | 2.81× (2.67–2.96) | 3.56× (3.47–3.66) | 4.06× (3.94–4.19) | 4.18× (4.06–4.29) | 9.06× (8.72–9.40) | 8.86× (8.56–9.18) | 2.69× (2.59–2.78) | 2.64× (2.55–2.74) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (3.10–3.13) | 6.92× (6.88–6.97) | 7.31× (7.25–7.37) | 8.29× (8.22–8.36) | 7.44× (7.39–7.50) | 7.91× (7.87–7.96) | 7.61× (7.56–7.67) | 2.42× (2.40–2.43) | 2.21× (2.20–2.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.98× (2.96–3.00) | 6.75× (6.65–6.86) | 7.32× (7.23–7.41) | 8.18× (8.07–8.28) | 7.04× (6.96–7.12) | 7.33× (7.25–7.42) | 7.06× (6.97–7.14) | 2.24× (2.21–2.27) | 2.05× (2.02–2.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.26× (3.20–3.32) | 6.25× (6.19–6.32) | 6.39× (6.32–6.47) | 7.10× (7.02–7.17) | 6.39× (6.33–6.46) | 4.80× (4.75–4.84) | 4.61× (4.58–4.65) | 1.46× (1.45–1.48) | 1.34× (1.32–1.35) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.64–0.73) | 3.43× (3.11–3.78) | 4.58× (4.18–5.01) | 4.13× (3.76–4.53) | 6.89× (6.29–7.54) | 3.14× (2.85–3.45) | 3.13× (2.84–3.45) | 1.16× (1.11–1.22) | 1.06× (0.97–1.15) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.60) | 2.85× (2.74–2.97) | 5.09× (4.89–5.30) | 3.48× (3.34–3.62) | 5.73× (5.50–5.96) | 2.60× (2.50–2.70) | 2.59× (2.49–2.70) | 0.96× (0.90–1.03) | 0.88× (0.84–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.57–0.61) | 3.45× (3.34–3.56) | 5.26× (5.09–5.44) | 3.62× (3.51–3.74) | 5.89× (5.69–6.09) | 2.68× (2.60–2.76) | 2.67× (2.59–2.76) | 0.99× (0.93–1.06) | 0.90× (0.88–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.98× (2.88–3.08) | 3.38× (3.27–3.49) | 3.90× (3.78–4.02) | 4.61× (4.46–4.77) | 5.03× (4.88–5.18) | 3.78× (3.66–3.90) | 3.08× (2.99–3.17) | 2.17× (2.10–2.24) | 2.09× (2.03–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.65× (2.53–2.79) | 2.99× (2.86–3.13) | 3.54× (3.35–3.74) | 4.02× (3.84–4.21) | 4.31× (4.11–4.53) | 3.17× (3.00–3.34) | 2.58× (2.46–2.71) | 1.82× (1.74–1.90) | 1.75× (1.67–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.46× (2.38–2.54) | 2.07× (1.98–2.17) | 2.52× (2.44–2.61) | 2.53× (2.44–2.62) | 2.64× (2.54–2.74) | 1.80× (1.72–1.87) | 1.46× (1.41–1.53) | 1.03× (1.00–1.07) | 0.99× (0.96–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.32–1.35) | 2.71× (2.68–2.73) | 2.42× (2.39–2.44) | 2.56× (2.54–2.59) | 2.84× (2.81–2.86) | 3.18× (3.16–3.21) | 3.25× (3.22–3.29) | 1.27× (1.25–1.28) | 1.25× (1.24–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.31) | 2.65× (2.59–2.70) | 2.36× (2.27–2.46) | 3.03× (2.97–3.10) | 2.66× (2.59–2.72) | 2.91× (2.84–2.98) | 2.97× (2.90–3.04) | 1.16× (1.13–1.18) | 1.14× (1.12–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.53–1.59) | 2.67× (2.58–2.76) | 2.24× (2.21–2.28) | 2.95× (2.88–3.02) | 2.64× (2.59–2.70) | 2.15× (2.11–2.20) | 2.20× (2.16–2.24) | 0.86× (0.84–0.87) | 0.85× (0.83–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.38–1.41) | 3.12× (3.10–3.14) | 2.58× (2.56–2.61) | 2.77× (2.75–2.80) | 3.10× (3.08–3.12) | 3.89× (3.85–3.93) | 4.12× (4.08–4.15) | 1.49× (1.48–1.50) | 1.51× (1.50–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.28–1.39) | 2.97× (2.87–3.07) | 2.49× (2.42–2.56) | 3.34× (3.22–3.47) | 2.84× (2.76–2.92) | 3.48× (3.38–3.59) | 3.69× (3.58–3.80) | 1.33× (1.29–1.38) | 1.35× (1.30–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.62–1.70) | 3.12× (3.06–3.18) | 2.32× (2.27–2.38) | 3.19× (3.12–3.27) | 2.85× (2.77–2.92) | 2.58× (2.53–2.63) | 2.73× (2.68–2.79) | 0.99× (0.97–1.00) | 1.00× (0.98–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.68–0.76) | 2.81× (2.60–3.04) | 3.87× (3.60–4.16) | 3.38× (3.13–3.64) | 7.09× (6.54–7.70) | 2.56× (2.35–2.79) | 2.58× (2.37–2.80) | 1.09× (0.99–1.19) | 0.95× (0.87–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.58–0.65) | 2.45× (2.30–2.61) | 4.49× (4.20–4.79) | 3.08× (2.88–3.29) | 6.30× (5.86–6.78) | 2.24× (2.11–2.38) | 2.26× (2.12–2.40) | 0.95× (0.90–1.01) | 0.83× (0.78–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.57–0.63) | 2.85× (2.73–2.98) | 4.38× (4.17–4.59) | 3.11× (2.96–3.26) | 6.22× (5.97–6.48) | 2.21× (2.10–2.33) | 2.22× (2.12–2.33) | 0.94× (0.89–1.00) | 0.82× (0.78–0.86) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.82× (1.80–1.84) | 3.93× (3.89–3.98) | 4.37× (4.31–4.42) | 4.66× (4.60–4.72) | 4.81× (4.74–4.88) | 5.04× (4.96–5.12) | 4.93× (4.87–4.99) | 1.51× (1.49–1.52) | 1.44× (1.42–1.45) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.58× (1.53–1.64) | 3.41× (3.29–3.54) | 3.94× (3.81–4.07) | 4.36× (4.21–4.50) | 4.11× (3.91–4.32) | 4.15× (4.01–4.30) | 4.07× (3.93–4.20) | 1.24× (1.20–1.28) | 1.18× (1.14–1.23) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.86× (1.77–1.95) | 3.17× (3.08–3.26) | 3.52× (3.44–3.61) | 3.83× (3.77–3.90) | 3.63× (3.57–3.70) | 2.83× (2.77–2.90) | 2.77× (2.72–2.84) | 0.85× (0.83–0.87) | 0.81× (0.79–0.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.26–0.27) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.06× (11.05–11.07) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.41× (0.34–0.49) | 0.65× (0.63–0.67) | 0.56× (0.54–0.58) | 5.27× (5.09–5.45) | 0.12× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.31–0.34) | 0.45× (0.40–0.52) | 0.58× (0.55–0.61) | 0.51× (0.48–0.54) | 3.39× (3.20–3.59) | 0.08× (0.07–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.00–8.07) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.62× (0.56–0.68) | 1.62× (1.58–1.65) | 0.81× (0.79–0.83) | 5.90× (5.76–6.03) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.62–0.65) | 0.70× (0.64–0.76) | 1.13× (1.10–1.16) | 0.71× (0.69–0.73) | 3.40× (3.33–3.48) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.43× (0.41–0.45) | 0.92× (0.89–0.95) | 0.72× (0.69–0.76) | 3.19× (3.08–3.30) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.61–0.66) | 0.45× (0.43–0.46) | 0.85× (0.82–0.87) | 0.78× (0.74–0.82) | 2.47× (2.39–2.55) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.49× (0.46–0.51) | 0.39× (0.36–0.42) | 0.86× (0.82–0.90) | 0.72× (0.68–0.76) | 2.95× (2.82–3.08) | 0.18× (0.17–0.19) | 0.20× (0.19–0.21) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.47× (0.45–0.49) | 0.81× (0.80–0.83) | 0.83× (0.81–0.86) | 2.36× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.33) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.30–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.66) | 6.39× (6.37–6.42) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.38× (0.35–0.41) | 1.03× (1.00–1.05) | 0.67× (0.65–0.69) | 3.23× (3.15–3.32) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 58 MB | 0.48× (0.47–0.49) | 0.42× (0.38–0.48) | 0.88× (0.86–0.90) | 0.72× (0.70–0.74) | 2.25× (2.19–2.31) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 471 | 2.13 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 843 | 2.29 | 1.08× (1.06–1.09) | 86% | 1.05× (1.04–1.07) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,034 | 3.52 | 1.65× (1.63–1.67) | 55% | 1.49× (1.48–1.51) | 1.73× (1.70–1.76) |

**Best:** mibz2 at 4 threads (1,034 MB/s); lbzip2 at 4 threads (179 MB/s); pbz2 at 4 threads (310 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.23, lbzip2 1t 0.04, lbzip2 2t 0.16, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.36, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.56 | 0.02 | 0.08 | 0.203 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3600 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 11.3% |
| MTF + RLE2 | 1.65 | 16.2% |
| T-vector build | 1.59 | 15.7% |
| walk (the one each build uses) | 5.38 | 52.9% |
| of which: segment assembly | 0.28 | 2.7% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.13 | 1.3% |
| **total** | 10.18 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.432 | 1.128 | (in its total) | 2.61× (2.60–2.62) |  |
| walk | 1.438 | 2.804 | (in its total) | 1.96× (1.95–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.078 | 0.072 | (in its total) | 0.93× (0.92–0.93) |  |
| inverse BWT, total | 1.948 | 4.005 | 2.081 | 2.06× (2.05–2.07) | 1.07× (1.07–1.07) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h04m ± 7m25s |
| 4 threads | 29m35s ± 4m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m24s ± 2m53s |
| the scanner alone (its serial part) | 11m56s ± 26.3s |
| the workers alone, over 4 | 20m24s ± 2m53s |

At 4 threads the workers bound it. Per byte: the scanner 6.9 ns a byte of bz2, the workers 2.67 ns a byte of output at 4 threads (CPU, summed over them), 1.73 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,508,059 | 12,616,482 | \`8d43d83ee25ebeaf…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3600.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
