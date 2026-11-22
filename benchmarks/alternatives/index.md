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

The latest record: [`3296195`](<https://github.com/lmmx/mibz2/commit/3296195e3104967d800e105d44c4c43d7fc01f14>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45005333087>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.505 s | 415 | 7.5 | 3.10× (3.09–3.12) | 6.94× (6.90–6.99) | 7.43× (7.40–7.45) | 8.35× (8.31–8.38) | 7.41× (7.36–7.47) | 8.01× (7.97–8.04) | 7.69× (7.64–7.74) | 2.40× (2.39–2.41) | 2.24× (2.22–2.25) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.290 s | 724 | 8.0 | 3.05× (2.94–3.17) | 6.46× (6.06–6.88) | 7.57× (7.08–8.09) | 7.75× (7.28–8.24) | 6.86× (6.44–7.30) | 13.63× (12.80–14.52) | 13.08× (12.29–13.93) | 4.08× (3.85–4.34) | 3.81× (3.57–4.05) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.232 s | 904 | 12.4 | 3.36× (3.30–3.42) | 5.82× (5.72–5.92) | 6.57× (6.46–6.69) | 6.64× (6.54–6.74) | 6.50× (6.38–6.62) | 17.33× (17.06–17.61) | 16.63× (16.38–16.89) | 5.19× (5.12–5.27) | 4.84× (4.75–4.92) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.4 | 0.59× (0.58–0.60) | 3.47× (3.38–3.57) | 4.43× (4.29–4.57) | 3.86× (3.76–3.96) | 6.32× (6.16–6.48) | 2.87× (2.79–2.95) | 2.86× (2.79–2.93) | 1.13× (1.10–1.16) | 1.01× (0.98–1.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 913 | 3.8 | 0.52× (0.52–0.53) | 2.80× (2.75–2.85) | 5.54× (5.44–5.64) | 3.63× (3.56–3.70) | 5.91× (5.82–6.01) | 2.69× (2.64–2.75) | 2.68× (2.64–2.73) | 1.06× (1.04–1.09) | 0.94× (0.93–0.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 905 | 3.8 | 0.52× (0.51–0.53) | 2.80× (2.70–2.90) | 5.43× (5.32–5.54) | 3.57× (3.51–3.63) | 5.87× (5.74–6.00) | 2.64× (2.59–2.69) | 2.63× (2.58–2.68) | 1.04× (1.02–1.06) | 0.92× (0.91–0.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 520 | 6.2 | 2.85× (2.81–2.89) | 3.50× (3.46–3.54) | 4.00× (3.95–4.05) | 4.49× (4.44–4.54) | 4.88× (4.82–4.93) | 3.68× (3.63–3.73) | 3.01× (2.97–3.04) | 2.10× (2.08–2.12) | 2.15× (2.13–2.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 854 | 7.2 | 2.60× (2.54–2.66) | 3.31× (3.11–3.52) | 4.49× (4.40–4.57) | 4.20× (4.13–4.26) | 5.66× (5.55–5.77) | 6.07× (5.97–6.18) | 4.97× (4.88–5.05) | 3.47× (3.42–3.52) | 3.56× (3.49–3.62) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 848 | 13.3 | 2.23× (2.13–2.33) | 2.28× (2.19–2.38) | 3.27× (3.08–3.48) | 2.79× (2.65–2.93) | 4.57× (4.39–4.76) | 5.98× (5.75–6.22) | 4.89× (4.69–5.09) | 3.42× (3.28–3.56) | 3.50× (3.36–3.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.147 s | 54 | 57.6 | 1.34× (1.32–1.36) | 2.64× (2.61–2.68) | 2.30× (2.27–2.34) | 2.38× (2.35–2.41) | 2.68× (2.65–2.71) | 3.06× (3.02–3.10) | 3.22× (3.18–3.26) | 1.27× (1.25–1.28) | 1.30× (1.29–1.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.084 s | 95 | 61.9 | 1.38× (1.31–1.45) | 2.75× (2.63–2.87) | 2.51× (2.45–2.56) | 3.26× (3.21–3.31) | 2.97× (2.91–3.04) | 5.34× (5.24–5.45) | 5.62× (5.51–5.73) | 2.21× (2.17–2.25) | 2.27× (2.23–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 84.9 | 1.42× (1.38–1.47) | 2.51× (2.36–2.66) | 2.34× (2.27–2.41) | 2.92× (2.85–2.99) | 2.91× (2.83–3.00) | 6.29× (6.10–6.48) | 6.61× (6.42–6.81) | 2.60× (2.52–2.69) | 2.67× (2.59–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.165 s | 58 | 53.8 | 1.38× (1.37–1.39) | 3.02× (2.99–3.06) | 2.46× (2.43–2.49) | 2.58× (2.55–2.62) | 2.91× (2.88–2.94) | 3.71× (3.67–3.75) | 4.00× (3.96–4.05) | 1.48× (1.46–1.50) | 1.53× (1.51–1.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.094 s | 103 | 59.0 | 1.32× (1.25–1.40) | 2.98× (2.75–3.23) | 2.56× (2.41–2.71) | 3.63× (3.42–3.87) | 3.04× (2.88–3.22) | 6.38× (6.05–6.73) | 6.89× (6.52–7.28) | 2.55× (2.41–2.69) | 2.64× (2.49–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 120 | 83.6 | 1.46× (1.43–1.50) | 2.74× (2.66–2.83) | 2.50× (2.34–2.68) | 3.33× (3.23–3.42) | 2.95× (2.88–3.01) | 7.58× (7.41–7.76) | 8.18× (8.00–8.36) | 3.02× (2.94–3.11) | 3.13× (3.07–3.19) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 4.8 | 0.63× (0.62–0.63) | 3.00× (2.95–3.04) | 4.41× (4.34–4.47) | 3.29× (3.23–3.34) | 6.78× (6.68–6.88) | 2.45× (2.41–2.50) | 2.47× (2.42–2.51) | 1.04× (1.03–1.05) | 1.51× (1.49–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.3 | 0.51× (0.50–0.52) | 2.75× (2.70–2.80) | 4.72× (4.64–4.80) | 3.03× (2.98–3.08) | 6.28× (6.17–6.40) | 2.25× (2.20–2.29) | 2.26× (2.22–2.30) | 0.95× (0.94–0.97) | 1.38× (1.36–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 662 | 5.3 | 0.53× (0.49–0.56) | 2.72× (2.68–2.75) | 4.67× (4.60–4.74) | 2.99× (2.95–3.03) | 6.17× (6.06–6.28) | 2.22× (2.18–2.25) | 2.23× (2.20–2.26) | 0.94× (0.92–0.96) | 1.36× (1.34–1.38) | yes |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 1 | 0.078 s | 176 | 18.0 | 1.79× (1.75–1.83) | 3.86× (3.78–3.95) | 4.35× (4.22–4.48) | 4.50× (4.39–4.60) | 4.57× (4.46–4.69) | 4.89× (4.77–5.02) | 4.92× (4.80–5.04) | 1.47× (1.43–1.51) | 1.42× (1.39–1.45) | yes |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 2 | 0.049 s | 284 | 20.9 | 1.63× (1.54–1.72) | 3.52× (3.29–3.78) | 4.47× (4.28–4.68) | 4.78× (4.52–5.06) | 4.47× (4.33–4.63) | 7.79× (7.56–8.04) | 7.83× (7.58–8.09) | 2.34× (2.27–2.42) | 2.26× (2.19–2.33) | yes |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 4 | 0.043 s | 320 | 32.3 | 1.65× (1.63–1.69) | 3.00× (2.83–3.19) | 3.82× (3.74–3.91) | 3.97× (3.91–4.03) | 4.28× (4.21–4.36) | 8.88× (8.73–9.03) | 8.92× (8.80–9.05) | 2.67× (2.63–2.71) | 2.57× (2.53–2.61) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.10× (3.09–3.11) | 6.91× (6.86–6.96) | 7.40× (7.37–7.43) | 8.32× (8.28–8.36) | 7.39× (7.34–7.44) | 7.98× (7.95–8.02) | 7.66× (7.61–7.71) | 2.40× (2.39–2.40) | 2.22× (2.21–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.03× (3.02–3.04) | 6.86× (6.76–6.95) | 7.46× (7.43–7.49) | 8.36× (8.32–8.39) | 7.10× (7.07–7.13) | 7.43× (7.38–7.49) | 7.14× (7.10–7.18) | 2.23× (2.22–2.24) | 2.07× (2.06–2.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.38× (3.35–3.41) | 6.37× (6.29–6.45) | 6.56× (6.50–6.62) | 7.27× (7.20–7.34) | 6.42× (6.32–6.53) | 4.85× (4.78–4.91) | 4.65× (4.60–4.71) | 1.45× (1.44–1.47) | 1.35× (1.33–1.37) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 2.93× (2.85–3.00) | 4.00× (3.88–4.12) | 3.63× (3.52–3.73) | 5.89× (5.73–6.06) | 2.72× (2.63–2.80) | 2.71× (2.64–2.78) | 1.12× (1.08–1.15) | 0.92× (0.89–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.72× (2.67–2.76) | 4.91× (4.82–5.00) | 3.38× (3.32–3.44) | 5.45× (5.38–5.52) | 2.51× (2.47–2.56) | 2.50× (2.46–2.55) | 1.03× (1.01–1.05) | 0.85× (0.83–0.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.55) | 3.07× (3.02–3.13) | 4.89× (4.81–4.96) | 3.35× (3.30–3.40) | 5.43× (5.32–5.55) | 2.48× (2.44–2.52) | 2.48× (2.44–2.52) | 1.02× (1.00–1.04) | 0.84× (0.82–0.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.77× (2.73–2.81) | 3.19× (3.15–3.23) | 3.71× (3.66–3.76) | 4.34× (4.29–4.40) | 4.71× (4.66–4.77) | 3.57× (3.52–3.61) | 2.92× (2.89–2.95) | 2.06× (2.03–2.08) | 1.98× (1.95–2.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.58× (2.55–2.62) | 3.03× (2.92–3.15) | 3.50× (3.46–3.54) | 3.87× (3.82–3.92) | 4.22× (4.17–4.28) | 3.10× (3.06–3.14) | 2.54× (2.50–2.57) | 1.79× (1.76–1.82) | 1.72× (1.70–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.32× (2.24–2.40) | 1.98× (1.90–2.07) | 2.40× (2.33–2.47) | 2.37× (2.30–2.45) | 2.49× (2.40–2.59) | 1.68× (1.63–1.74) | 1.38× (1.33–1.43) | 0.97× (0.94–1.00) | 0.94× (0.90–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.35) | 2.60× (2.57–2.63) | 2.25× (2.21–2.28) | 2.37× (2.33–2.40) | 2.66× (2.63–2.70) | 3.04× (3.00–3.08) | 3.19× (3.15–3.24) | 1.26× (1.24–1.28) | 1.26× (1.24–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.30–1.33) | 2.65× (2.55–2.75) | 2.25× (2.23–2.27) | 3.01× (2.98–3.04) | 2.60× (2.57–2.63) | 2.83× (2.81–2.86) | 2.98× (2.95–3.01) | 1.18× (1.17–1.19) | 1.17× (1.16–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.54–1.62) | 2.68× (2.61–2.75) | 2.22× (2.17–2.27) | 2.94× (2.87–3.00) | 2.59× (2.53–2.65) | 2.08× (2.04–2.13) | 2.19× (2.14–2.25) | 0.87× (0.85–0.88) | 0.86× (0.84–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.38× (1.37–1.39) | 2.97× (2.93–3.00) | 2.41× (2.39–2.44) | 2.57× (2.53–2.60) | 2.89× (2.86–2.92) | 3.68× (3.64–3.72) | 3.97× (3.92–4.02) | 1.47× (1.46–1.49) | 1.49× (1.47–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.31× (1.24–1.38) | 2.88× (2.75–3.02) | 2.35× (2.23–2.48) | 3.31× (3.14–3.49) | 2.73× (2.59–2.87) | 3.32× (3.16–3.49) | 3.58× (3.40–3.77) | 1.33× (1.27–1.40) | 1.35× (1.28–1.41) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.57–1.65) | 2.96× (2.90–3.03) | 2.25× (2.18–2.33) | 3.08× (3.00–3.15) | 2.66× (2.60–2.72) | 2.39× (2.34–2.44) | 2.58× (2.53–2.63) | 0.96× (0.94–0.98) | 0.97× (0.95–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.42× (2.38–2.46) | 3.51× (3.44–3.58) | 3.03× (2.98–3.08) | 6.14× (6.04–6.23) | 2.29× (2.25–2.33) | 2.30× (2.26–2.35) | 1.03× (1.03–1.04) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.17× (2.14–2.20) | 3.99× (3.94–4.04) | 2.78× (2.74–2.82) | 5.55× (5.47–5.63) | 2.05× (2.01–2.08) | 2.05× (2.03–2.08) | 0.92× (0.91–0.94) | 0.74× (0.73–0.75) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.53–0.60) | 2.51× (2.45–2.57) | 3.99× (3.93–4.04) | 2.81× (2.77–2.84) | 5.57× (5.48–5.67) | 2.06× (2.03–2.10) | 2.07× (2.04–2.09) | 0.93× (0.91–0.95) | 0.75× (0.74–0.76) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 1 | 1.77× (1.73–1.82) | 3.74× (3.65–3.84) | 4.21× (4.09–4.32) | 4.43× (4.33–4.53) | 4.50× (4.39–4.61) | 4.81× (4.70–4.93) | 4.83× (4.72–4.95) | 1.46× (1.43–1.50) | 1.39× (1.36–1.42) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 2 | 1.65× (1.63–1.66) | 3.49× (3.42–3.55) | 4.02× (3.97–4.08) | 4.45× (4.39–4.52) | 4.02× (3.99–4.06) | 4.16× (4.11–4.21) | 4.18× (4.13–4.22) | 1.26× (1.25–1.27) | 1.20× (1.19–1.21) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 4 | 1.83× (1.80–1.87) | 3.05× (2.96–3.15) | 3.40× (3.33–3.47) | 3.78× (3.74–3.83) | 3.47× (3.43–3.52) | 2.72× (2.69–2.75) | 2.73× (2.70–2.76) | 0.82× (0.82–0.83) | 0.79× (0.78–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.97× (10.93–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.46× (0.40–0.54) | 0.71× (0.69–0.73) | 0.60× (0.59–0.62) | 5.73× (5.61–5.84) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.36× (0.34–0.37) | 0.59× (0.52–0.68) | 0.62× (0.60–0.64) | 0.56× (0.54–0.58) | 3.71× (3.57–3.85) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.67) | 7.96× (7.92–8.00) | 0.64× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.74) | 0.71× (0.65–0.79) | 1.69× (1.65–1.74) | 0.86× (0.84–0.88) | 6.21× (6.05–6.37) | 0.50× (0.49–0.51) | 0.53× (0.51–0.54) | 0.57× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.69–0.72) | 0.74× (0.70–0.79) | 1.26× (1.24–1.28) | 0.77× (0.75–0.80) | 3.77× (3.71–3.83) | 0.29× (0.29–0.30) | 0.31× (0.31–0.31) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.17× (5.15–5.20) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.39× (0.36–0.43) | 0.91× (0.88–0.93) | 0.71× (0.68–0.74) | 3.14× (3.05–3.23) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.47) | 0.86× (0.84–0.88) | 0.81× (0.79–0.84) | 2.52× (2.46–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.83× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.44–0.48) | 0.35× (0.33–0.38) | 0.82× (0.79–0.85) | 0.69× (0.68–0.70) | 2.81× (2.72–2.90) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.61) | 0.46× (0.44–0.49) | 0.80× (0.79–0.80) | 0.81× (0.78–0.83) | 2.30× (2.28–2.32) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.76–0.77) | 5.33× (5.33–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.30) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.89–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.04–1.05) | 0.15× (0.15–0.15) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.36× (6.33–6.38) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 2 | 33 MB | 0.51× (0.50–0.52) | 0.47× (0.42–0.52) | 1.15× (1.12–1.18) | 0.75× (0.72–0.78) | 3.75× (3.68–3.83) | 0.23× (0.23–0.24) | 0.24× (0.24–0.25) | 0.37× (0.36–0.38) | 0.18× (0.18–0.18) |
| `text` (2.71 MB → 13.7 MB, 5.1×) | 4 | 50 MB | 0.54× (0.54–0.55) | 0.50× (0.45–0.55) | 1.00× (0.99–1.02) | 0.82× (0.80–0.85) | 2.57× (2.54–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 415 | 2.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 724 | 2.59 | 1.07× (1.07–1.08) | 85% | 1.07× (1.05–1.08) | 1.05× (1.05–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 904 | 4.00 | 1.65× (1.63–1.67) | 54% | 1.52× (1.50–1.53) | 1.80× (1.78–1.81) |

**Best:** mibz2 at 4 threads (904 MB/s); lbzip2 at 4 threads (155 MB/s); pbz2 at 4 threads (268 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.12, lbzip2 1t 0.04, lbzip2 2t 0.22, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.37. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.64 | 0.02 | 0.08 | 0.232 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3104 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.18 | 11.6% |
| MTF + RLE2 | 1.69 | 16.5% |
| T-vector build | 1.61 | 15.8% |
| walk (the one each build uses) | 5.29 | 51.9% |
| of which: segment assembly | 0.28 | 2.7% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.19 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.502 | 1.309 | (in its total) | 2.60× (2.59–2.62) |  |
| walk | 1.604 | 3.025 | (in its total) | 1.88× (1.87–1.89) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.090 | 0.078 | (in its total) | 0.87× (0.86–0.88) |  |
| inverse BWT, total | 2.196 | 4.412 | 2.354 | 2.00× (1.99–2.02) | 1.07× (1.06–1.08) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m20s |
| 4 threads | 33m57s ± 4m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m19s ± 3m18s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 23m19s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.06 ns a byte of output at 4 threads (CPU, summed over them), 1.95 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,711,422 | 13,739,425 | \`33e2ae9392d04f47…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3103.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
