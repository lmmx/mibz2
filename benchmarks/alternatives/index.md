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

The latest record: [`172ae3f`](<https://github.com/lmmx/mibz2/commit/172ae3f940c021f12210f3116006cccb64da082d>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46119003123>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.477 s | 439 | 7.4 | 3.16× (3.15–3.17) | 7.23× (7.20–7.27) | 7.42× (7.39–7.45) | 8.17× (8.13–8.21) | 7.24× (7.17–7.30) | 7.93× (7.89–7.97) | 7.42× (7.40–7.44) | 2.35× (2.34–2.35) | 2.71× (2.70–2.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.276 s | 760 | 8.1 | 3.02× (2.78–3.27) | 6.36× (5.94–6.81) | 7.10× (6.60–7.64) | 7.18× (6.66–7.74) | 6.37× (5.92–6.84) | 12.96× (12.03–13.95) | 12.12× (11.28–13.03) | 3.83× (3.57–4.11) | 4.43× (4.11–4.77) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.217 s | 967 | 12.6 | 3.26× (3.22–3.31) | 4.91× (4.86–4.96) | 6.18× (6.06–6.30) | 5.72× (5.60–5.85) | 5.63× (5.56–5.71) | 17.43× (17.22–17.65) | 16.31× (16.13–16.49) | 5.15× (5.10–5.21) | 5.96× (5.89–6.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 950 | 3.8 | 0.59× (0.58–0.59) | 3.58× (3.41–3.75) | 4.28× (4.24–4.32) | 3.48× (3.45–3.51) | 6.19× (6.04–6.33) | 2.89× (2.87–2.91) | 2.58× (2.56–2.60) | 1.00× (0.99–1.02) | 1.46× (1.45–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 917 | 4.0 | 0.54× (0.53–0.55) | 2.82× (2.79–2.86) | 5.70× (5.58–5.83) | 3.38× (3.34–3.42) | 5.82× (5.73–5.90) | 2.79× (2.76–2.82) | 2.49× (2.46–2.53) | 0.97× (0.95–0.98) | 1.41× (1.39–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 899 | 4.1 | 0.53× (0.52–0.54) | 2.78× (2.72–2.83) | 5.59× (5.44–5.74) | 3.31× (3.25–3.37) | 5.77× (5.69–5.85) | 2.74× (2.69–2.78) | 2.45× (2.40–2.49) | 0.95× (0.93–0.97) | 1.38× (1.36–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 600 | 5.8 | 2.78× (2.72–2.84) | 3.11× (3.08–3.14) | 3.99× (3.95–4.04) | 4.05× (4.01–4.09) | 4.36× (4.31–4.41) | 2.64× (2.63–2.66) | 2.17× (2.15–2.19) | 2.08× (2.06–2.09) | 3.10× (3.06–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 940 | 6.9 | 2.47× (2.30–2.66) | 2.96× (2.85–3.07) | 4.39× (4.22–4.56) | 3.83× (3.53–4.16) | 5.34× (5.13–5.54) | 4.15× (3.99–4.32) | 3.41× (3.29–3.53) | 3.26× (3.15–3.38) | 4.86× (4.68–5.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 877 | 13.8 | 2.05× (1.99–2.10) | 2.27× (2.13–2.41) | 3.39× (3.26–3.53) | 2.45× (2.37–2.53) | 4.47× (4.35–4.59) | 3.86× (3.76–3.96) | 3.17× (3.08–3.25) | 3.03× (2.95–3.11) | 4.52× (4.40–4.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.119 s | 67 | 49.2 | 1.49× (1.48–1.50) | 2.97× (2.94–2.99) | 2.66× (2.62–2.71) | 2.72× (2.70–2.74) | 3.16× (3.14–3.19) | 3.45× (3.43–3.47) | 3.50× (3.47–3.54) | 1.36× (1.34–1.37) | 1.53× (1.52–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 s | 110 | 55.5 | 1.43× (1.39–1.47) | 3.00× (2.75–3.28) | 2.66× (2.60–2.72) | 3.41× (3.32–3.50) | 3.27× (3.20–3.35) | 5.65× (5.51–5.80) | 5.74× (5.61–5.87) | 2.22× (2.17–2.28) | 2.51× (2.45–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 s | 135 | 76.3 | 1.49× (1.45–1.54) | 2.46× (2.36–2.56) | 2.38× (2.28–2.48) | 2.99× (2.84–3.14) | 3.26× (3.19–3.32) | 6.89× (6.75–7.04) | 6.99× (6.88–7.11) | 2.71× (2.66–2.76) | 3.06× (3.00–3.12) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.133 s | 72 | 45.8 | 1.59× (1.58–1.59) | 3.30× (3.27–3.34) | 2.78× (2.72–2.84) | 2.89× (2.87–2.91) | 3.35× (3.33–3.38) | 3.93× (3.91–3.96) | 4.27× (4.24–4.30) | 1.53× (1.51–1.55) | 1.83× (1.82–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 118 | 51.4 | 1.40× (1.33–1.48) | 2.87× (2.69–3.05) | 2.57× (2.45–2.70) | 3.67× (3.41–3.94) | 3.20× (3.04–3.37) | 6.21× (5.89–6.55) | 6.75× (6.39–7.12) | 2.42× (2.29–2.55) | 2.90× (2.77–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 141 | 73.0 | 1.57× (1.53–1.61) | 2.58× (2.46–2.70) | 2.51× (2.45–2.58) | 3.34× (3.28–3.39) | 3.19× (3.15–3.23) | 7.71× (7.61–7.82) | 8.38× (8.24–8.51) | 3.00× (2.96–3.05) | 3.60× (3.52–3.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 722 | 5.2 | 0.66× (0.65–0.67) | 2.97× (2.93–3.00) | 4.34× (4.29–4.40) | 3.06× (3.03–3.09) | 6.87× (6.75–6.99) | 2.49× (2.46–2.52) | 2.30× (2.27–2.33) | 0.99× (0.98–1.00) | 1.49× (1.47–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 5.8 | 0.53× (0.53–0.54) | 2.74× (2.69–2.79) | 4.83× (4.62–5.04) | 2.86× (2.81–2.91) | 6.30× (6.22–6.38) | 2.29× (2.26–2.33) | 2.12× (2.09–2.15) | 0.91× (0.89–0.93) | 1.37× (1.35–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 5.8 | 0.54× (0.52–0.55) | 2.71× (2.69–2.73) | 4.63× (4.60–4.66) | 2.81× (2.79–2.82) | 6.19× (6.14–6.24) | 2.26× (2.24–2.27) | 2.08× (2.07–2.10) | 0.90× (0.89–0.91) | 1.35× (1.34–1.36) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.062 s | 243 | 13.8 | 2.26× (2.23–2.30) | 4.38× (4.31–4.46) | 4.82× (4.74–4.91) | 4.91× (4.83–5.00) | 5.19× (5.07–5.33) | 5.49× (5.40–5.58) | 5.23× (5.09–5.38) | 1.71× (1.68–1.73) | 2.10× (2.07–2.14) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.047 s | 318 | 18.3 | 1.62× (1.47–1.79) | 3.22× (2.89–3.60) | 3.95× (3.58–4.35) | 4.22× (3.78–4.70) | 4.14× (3.75–4.57) | 7.14× (6.46–7.89) | 6.80× (6.16–7.52) | 2.22× (2.01–2.45) | 2.74× (2.47–3.02) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.040 s | 378 | 28.8 | 1.70× (1.63–1.77) | 2.65× (2.55–2.77) | 3.62× (3.53–3.70) | 3.75× (3.60–3.90) | 4.04× (3.91–4.17) | 8.60× (8.40–8.81) | 8.20× (7.92–8.49) | 2.68× (2.61–2.74) | 3.30× (3.22–3.38) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.15× (3.14–3.16) | 7.20× (7.16–7.24) | 7.38× (7.35–7.42) | 8.14× (8.10–8.18) | 7.21× (7.14–7.28) | 7.90× (7.86–7.94) | 7.39× (7.37–7.42) | 2.34× (2.33–2.35) | 2.69× (2.68–2.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.00–3.10) | 6.81× (6.73–6.89) | 7.12× (7.03–7.21) | 7.90× (7.78–8.02) | 6.70× (6.60–6.79) | 7.24× (7.13–7.36) | 6.78× (6.67–6.88) | 2.14× (2.12–2.17) | 2.47× (2.43–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.09× (3.06–3.12) | 5.11× (5.06–5.16) | 5.54× (5.48–5.60) | 5.88× (5.81–5.95) | 5.15× (5.10–5.20) | 4.66× (4.62–4.71) | 4.36× (4.32–4.41) | 1.38× (1.36–1.40) | 1.59× (1.57–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.18× (3.15–3.20) | 3.96× (3.92–4.00) | 3.26× (3.23–3.29) | 5.72× (5.59–5.86) | 2.72× (2.70–2.74) | 2.44× (2.42–2.46) | 1.00× (0.99–1.01) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.57–0.58) | 2.85× (2.82–2.88) | 5.01× (4.95–5.07) | 3.08× (3.05–3.12) | 5.22× (5.14–5.29) | 2.54× (2.51–2.57) | 2.28× (2.25–2.31) | 0.93× (0.92–0.95) | 0.96× (0.93–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.11× (3.06–3.15) | 4.99× (4.87–5.12) | 3.07× (3.02–3.12) | 5.23× (5.17–5.30) | 2.53× (2.49–2.57) | 2.27× (2.23–2.31) | 0.93× (0.91–0.95) | 0.96× (0.92–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.62–2.73) | 2.86× (2.83–2.89) | 3.59× (3.56–3.62) | 3.87× (3.84–3.90) | 4.17× (4.12–4.21) | 2.55× (2.53–2.56) | 2.10× (2.09–2.12) | 2.02× (2.00–2.03) | 2.79× (2.77–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.40× (2.35–2.45) | 2.67× (2.60–2.74) | 3.29× (3.21–3.36) | 3.43× (3.37–3.50) | 3.67× (3.59–3.75) | 2.14× (2.10–2.17) | 1.76× (1.73–1.80) | 1.69× (1.66–1.72) | 2.34× (2.29–2.39) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.05× (1.99–2.12) | 1.78× (1.72–1.84) | 2.27× (2.20–2.35) | 2.12× (2.04–2.20) | 2.17× (2.11–2.24) | 1.06× (1.02–1.10) | 0.88× (0.85–0.90) | 0.84× (0.81–0.87) | 1.16× (1.13–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.48–1.49) | 2.89× (2.87–2.91) | 2.60× (2.57–2.63) | 2.69× (2.67–2.72) | 3.13× (3.10–3.16) | 3.41× (3.39–3.43) | 3.46× (3.42–3.50) | 1.35× (1.34–1.37) | 1.50× (1.49–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.38–1.41) | 2.72× (2.68–2.76) | 2.39× (2.37–2.42) | 3.22× (3.19–3.25) | 2.83× (2.80–2.87) | 3.03× (2.99–3.07) | 3.08× (3.05–3.10) | 1.20× (1.19–1.22) | 1.33× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.52–1.63) | 2.49× (2.43–2.55) | 2.02× (1.99–2.06) | 2.79× (2.73–2.85) | 2.68× (2.63–2.72) | 2.22× (2.19–2.26) | 2.26× (2.22–2.29) | 0.88× (0.87–0.90) | 0.98× (0.96–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.58× (1.57–1.59) | 3.23× (3.20–3.26) | 2.72× (2.67–2.77) | 2.86× (2.85–2.88) | 3.32× (3.29–3.34) | 3.89× (3.87–3.91) | 4.22× (4.19–4.25) | 1.52× (1.51–1.54) | 1.79× (1.78–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.47–1.50) | 2.99× (2.97–3.02) | 2.51× (2.48–2.54) | 3.53× (3.49–3.58) | 3.00× (2.97–3.03) | 3.46× (3.43–3.49) | 3.75× (3.72–3.79) | 1.35× (1.33–1.37) | 1.59× (1.57–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.60–1.75) | 2.74× (2.70–2.78) | 2.10× (2.06–2.15) | 3.03× (2.98–3.08) | 2.79× (2.75–2.83) | 2.44× (2.41–2.47) | 2.65× (2.61–2.69) | 0.96× (0.94–0.97) | 1.12× (1.11–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.65× (2.61–2.70) | 3.53× (3.45–3.61) | 2.79× (2.77–2.82) | 6.11× (6.01–6.22) | 2.29× (2.27–2.31) | 2.12× (2.10–2.15) | 0.99× (0.98–1.00) | 1.00× (0.97–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.34× (2.31–2.37) | 4.03× (3.96–4.10) | 2.58× (2.54–2.62) | 5.43× (5.37–5.50) | 2.04× (2.01–2.07) | 1.89× (1.86–1.92) | 0.88× (0.86–0.90) | 0.89× (0.85–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.58) | 2.56× (2.52–2.60) | 3.99× (3.93–4.04) | 2.59× (2.56–2.62) | 5.46× (5.40–5.52) | 2.05× (2.02–2.07) | 1.90× (1.87–1.92) | 0.88× (0.87–0.89) | 0.89× (0.86–0.93) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.22× (2.19–2.26) | 4.26× (4.19–4.33) | 4.66× (4.59–4.74) | 4.79× (4.71–4.87) | 5.06× (4.93–5.19) | 5.35× (5.26–5.43) | 5.10× (4.96–5.24) | 1.68× (1.66–1.71) | 2.04× (2.00–2.08) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.80× (1.76–1.83) | 3.38× (3.30–3.46) | 3.74× (3.66–3.82) | 4.13× (4.02–4.25) | 3.87× (3.77–3.98) | 4.04× (3.94–4.15) | 3.85× (3.73–3.98) | 1.27× (1.25–1.30) | 1.54× (1.50–1.58) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.78× (1.74–1.83) | 2.67× (2.63–2.70) | 2.98× (2.91–3.05) | 3.16× (3.12–3.21) | 3.04× (2.99–3.09) | 2.59× (2.55–2.63) | 2.47× (2.41–2.53) | 0.82× (0.81–0.83) | 0.99× (0.97–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.28–0.30) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.96–11.01) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.48× (0.40–0.57) | 0.71× (0.68–0.73) | 0.58× (0.56–0.61) | 5.58× (5.36–5.81) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.38) | 0.69× (0.57–0.84) | 0.64× (0.62–0.67) | 0.55× (0.53–0.58) | 3.72× (3.56–3.88) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.91–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.67) | 7.96× (7.91–8.00) | 0.65× (0.64–0.66) | 0.68× (0.68–0.68) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.77× (0.73–0.82) | 1.70× (1.67–1.73) | 0.85× (0.83–0.87) | 6.16× (6.04–6.28) | 0.50× (0.49–0.51) | 0.52× (0.51–0.54) | 0.57× (0.56–0.58) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.68–0.72) | 0.81× (0.78–0.85) | 1.28× (1.24–1.31) | 0.78× (0.75–0.80) | 3.72× (3.61–3.84) | 0.29× (0.28–0.30) | 0.31× (0.30–0.32) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.74) | 5.15× (5.13–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.52) | 0.39× (0.35–0.42) | 0.89× (0.86–0.92) | 0.71× (0.69–0.74) | 3.07× (2.97–3.18) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.65–0.68) | 0.46× (0.46–0.47) | 0.89× (0.88–0.90) | 0.83× (0.81–0.85) | 2.59× (2.57–2.61) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.84) | 5.21× (5.19–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.35× (0.32–0.38) | 0.87× (0.84–0.89) | 0.70× (0.66–0.75) | 2.97× (2.88–3.06) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.51) | 0.80× (0.79–0.81) | 0.80× (0.76–0.84) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.30–5.31) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.18× (6.17–6.20) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.87) | 0.89× (0.89–0.90) | 6.17× (6.14–6.19) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.66× (0.65–0.66) | 6.37× (6.34–6.40) | 0.39× (0.38–0.39) | 0.41× (0.40–0.41) | 0.63× (0.63–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 36 MB | 0.46× (0.44–0.47) | 0.41× (0.37–0.46) | 1.04× (1.01–1.07) | 0.70× (0.67–0.72) | 3.42× (3.32–3.53) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.34× (0.33–0.35) | 0.16× (0.16–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.44× (0.43–0.46) | 0.90× (0.88–0.91) | 0.72× (0.69–0.74) | 2.29× (2.25–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 439 | 2.29 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 760 | 2.48 | 1.09× (1.07–1.11) | 82% | 1.03× (1.02–1.04) | 1.06× (1.04–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 967 | 3.87 | 1.69× (1.68–1.71) | 55% | 1.20× (1.19–1.22) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (967 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 0.99, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 12%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.64 | 0.02 | 0.09 | 0.217 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3251 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.4% |
| MTF + RLE2 | 1.54 | 17.9% |
| T-vector build | 1.51 | 17.6% |
| walk (the one each build uses) | 4.18 | 48.7% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.59 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.510 | 1.300 | (in its total) | 2.55× (2.54–2.55) |  |
| walk | 1.224 | 2.619 | (in its total) | 2.13× (2.12–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.053 | (in its total) | 1.18× (1.16–1.20) |  |
| inverse BWT, total | 1.778 | 3.974 | 3.268 | 2.23× (2.21–2.24) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m29s |
| 4 threads | 31m38s ± 4m27s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m12s ± 3m17s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 23m12s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 3.04 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3251.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
