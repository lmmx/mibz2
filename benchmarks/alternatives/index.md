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

The latest record: [`3dce61e`](<https://github.com/lmmx/mibz2/commit/3dce61e409e529197e7235315b8fcc09da9c334a>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44905142909>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.405 s | 518 | 7.2 | 2.96× (2.86–3.06) | 6.75× (6.53–6.97) | 7.01× (6.78–7.24) | 7.99× (7.74–8.25) | 7.04× (6.84–7.25) | 7.36× (7.12–7.60) | 6.95× (6.71–7.19) | 2.20× (2.13–2.28) | 2.46× (2.38–2.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.231 s | 907 | 7.8 | 3.08× (2.90–3.28) | 6.08× (5.84–6.34) | 7.03× (6.73–7.35) | 7.50× (7.20–7.81) | 6.48× (6.22–6.76) | 12.73× (12.23–13.25) | 12.02× (11.56–12.49) | 3.81× (3.66–3.96) | 4.25× (4.07–4.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,069 | 12.1 | 3.02× (2.96–3.08) | 4.40× (4.34–4.45) | 5.42× (5.32–5.52) | 5.34× (5.24–5.44) | 5.29× (5.18–5.40) | 15.47× (15.30–15.65) | 14.61× (14.45–14.76) | 4.63× (4.58–4.68) | 5.17× (5.12–5.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 3.6 | 0.57× (0.57–0.58) | 3.51× (3.38–3.65) | 4.70× (4.56–4.84) | 3.60× (3.56–3.64) | 6.16× (6.04–6.28) | 2.69× (2.65–2.72) | 2.68× (2.65–2.72) | 0.99× (0.96–1.01) | 1.83× (1.68–2.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,120 | 3.7 | 0.54× (0.52–0.55) | 2.86× (2.80–2.91) | 5.73× (5.62–5.83) | 3.63× (3.48–3.79) | 6.04× (5.88–6.20) | 2.65× (2.60–2.71) | 2.65× (2.60–2.70) | 0.97× (0.95–1.00) | 1.81× (1.67–1.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,105 | 3.6 | 0.53× (0.52–0.54) | 2.83× (2.77–2.88) | 5.64× (5.53–5.75) | 3.53× (3.46–3.60) | 6.02× (5.86–6.19) | 2.61× (2.56–2.66) | 2.61× (2.56–2.66) | 0.96× (0.94–0.98) | 1.78× (1.64–1.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 744 | 5.3 | 2.63× (2.37–2.92) | 2.96× (2.69–3.25) | 3.70× (3.37–4.07) | 3.93× (3.58–4.33) | 4.32× (3.95–4.72) | 2.67× (2.43–2.95) | 2.18× (1.98–2.40) | 2.00× (1.83–2.19) | 2.91× (2.65–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,104 | 6.5 | 2.19× (2.04–2.36) | 2.92× (2.75–3.10) | 4.41× (4.20–4.64) | 3.54× (3.26–3.85) | 5.03× (4.64–5.45) | 4.04× (3.74–4.37) | 3.29× (3.06–3.55) | 3.03× (2.81–3.25) | 4.41× (4.07–4.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 929 | 13.2 | 1.82× (1.78–1.87) | 1.82× (1.64–2.02) | 2.84× (2.67–3.02) | 2.06× (1.98–2.14) | 3.94× (3.83–4.05) | 3.53× (3.41–3.65) | 2.88× (2.79–2.96) | 2.64× (2.57–2.72) | 3.85× (3.74–3.96) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 51.8 | 1.36× (1.34–1.38) | 2.60× (2.53–2.66) | 2.39× (2.32–2.45) | 2.48× (2.43–2.53) | 2.80× (2.72–2.89) | 2.95× (2.89–3.01) | 3.04× (2.97–3.12) | 1.23× (1.21–1.25) | 1.36× (1.34–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 115 | 58.5 | 1.24× (1.16–1.32) | 2.33× (2.19–2.47) | 2.23× (2.12–2.35) | 2.98× (2.83–3.14) | 2.76× (2.61–2.91) | 4.62× (4.38–4.88) | 4.77× (4.51–5.05) | 1.93× (1.83–2.04) | 2.14× (2.02–2.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 78.4 | 1.35× (1.31–1.38) | 2.11× (2.02–2.19) | 2.17× (2.08–2.26) | 2.65× (2.58–2.73) | 2.93× (2.85–3.01) | 5.90× (5.77–6.04) | 6.09× (5.95–6.24) | 2.47× (2.41–2.52) | 2.73× (2.67–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.2 | 1.51× (1.45–1.57) | 3.02× (2.90–3.15) | 2.65× (2.58–2.72) | 2.76× (2.66–2.86) | 3.12× (3.09–3.15) | 3.65× (3.51–3.78) | 3.94× (3.80–4.09) | 1.49× (1.43–1.56) | 1.78× (1.71–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 129 | 52.6 | 1.34× (1.28–1.40) | 2.73× (2.57–2.91) | 2.47× (2.36–2.58) | 3.43× (3.28–3.59) | 3.03× (2.90–3.15) | 5.86× (5.62–6.12) | 6.34× (6.06–6.63) | 2.40× (2.28–2.53) | 2.86× (2.73–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 73.4 | 1.43× (1.34–1.51) | 2.19× (2.14–2.24) | 2.24× (2.16–2.32) | 3.02× (2.92–3.12) | 2.83× (2.74–2.92) | 6.91× (6.73–7.11) | 7.48× (7.26–7.70) | 2.83× (2.76–2.90) | 3.37× (3.29–3.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 5.0 | 0.63× (0.63–0.64) | 3.49× (3.45–3.53) | 4.39× (4.34–4.44) | 3.13× (3.10–3.16) | 6.66× (6.58–6.74) | 2.30× (2.28–2.33) | 2.34× (2.30–2.38) | 0.96× (0.94–0.98) | 1.76× (1.74–1.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 806 | 5.3 | 0.53× (0.51–0.55) | 2.50× (2.47–2.53) | 5.01× (4.95–5.07) | 2.98× (2.94–3.01) | 6.42× (6.34–6.50) | 2.19× (2.16–2.21) | 2.22× (2.17–2.28) | 0.91× (0.89–0.94) | 1.67× (1.64–1.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 797 | 5.2 | 0.52× (0.51–0.53) | 2.46× (2.43–2.49) | 4.92× (4.86–4.99) | 2.95× (2.91–2.99) | 6.41× (6.02–6.83) | 2.15× (2.12–2.18) | 2.18× (2.13–2.24) | 0.90× (0.88–0.92) | 1.64× (1.61–1.66) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.059 s | 228 | 16.7 | 1.84× (1.75–1.94) | 3.86× (3.80–3.91) | 4.34× (4.29–4.39) | 4.54× (4.51–4.58) | 4.68× (4.63–4.73) | 4.67× (4.62–4.72) | 4.58× (4.53–4.63) | 1.49× (1.47–1.51) | 1.72× (1.71–1.74) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.040 s | 338 | 20.3 | 1.54× (1.42–1.67) | 3.20× (2.98–3.42) | 3.82× (3.63–4.02) | 4.30× (4.04–4.58) | 4.31× (4.10–4.53) | 6.83× (6.48–7.19) | 6.70× (6.35–7.07) | 2.18× (2.07–2.30) | 2.52× (2.39–2.65) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.037 s | 366 | 31.9 | 1.47× (1.44–1.50) | 2.40× (2.29–2.52) | 3.12× (3.01–3.22) | 3.30× (3.23–3.37) | 3.83× (3.76–3.91) | 7.56× (7.44–7.68) | 7.42× (7.25–7.60) | 2.42× (2.36–2.48) | 2.79× (2.74–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.95× (2.85–3.05) | 6.71× (6.50–6.93) | 6.97× (6.74–7.20) | 7.96× (7.71–8.21) | 7.01× (6.81–7.22) | 7.33× (7.10–7.57) | 6.92× (6.69–7.16) | 2.20× (2.12–2.27) | 2.44× (2.36–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.79–2.89) | 6.32× (6.27–6.37) | 6.66× (6.58–6.74) | 7.78× (7.69–7.87) | 6.50× (6.44–6.56) | 6.79× (6.73–6.84) | 6.41× (6.36–6.46) | 2.03× (2.02–2.05) | 2.26× (2.23–2.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.90–3.04) | 4.90× (4.83–4.97) | 5.30× (5.24–5.36) | 5.97× (5.90–6.04) | 5.12× (5.06–5.18) | 4.43× (4.37–4.50) | 4.19× (4.12–4.26) | 1.33× (1.31–1.35) | 1.47× (1.45–1.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.98× (2.88–3.08) | 4.07× (3.88–4.27) | 3.37× (3.34–3.41) | 5.71× (5.60–5.82) | 2.54× (2.51–2.57) | 2.54× (2.50–2.57) | 0.99× (0.97–1.01) | 1.66× (1.64–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.75× (2.71–2.80) | 4.99× (4.92–5.06) | 3.38× (3.24–3.53) | 5.52× (5.39–5.65) | 2.47× (2.43–2.51) | 2.47× (2.42–2.51) | 0.96× (0.94–0.99) | 1.61× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.02× (2.95–3.08) | 4.96× (4.88–5.04) | 3.35× (3.29–3.42) | 5.61× (5.47–5.75) | 2.49× (2.44–2.53) | 2.48× (2.44–2.53) | 0.97× (0.95–0.99) | 1.62× (1.60–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.27–2.80) | 2.72× (2.46–2.99) | 3.23× (2.92–3.56) | 3.74× (3.40–4.13) | 4.11× (3.75–4.50) | 2.56× (2.32–2.83) | 2.10× (1.91–2.31) | 1.94× (1.76–2.13) | 2.53× (2.30–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.23–2.33) | 2.51× (2.43–2.60) | 3.34× (3.28–3.41) | 3.35× (3.28–3.42) | 3.63× (3.56–3.71) | 2.17× (2.12–2.22) | 1.78× (1.74–1.81) | 1.64× (1.61–1.67) | 2.14× (2.11–2.18) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.95–2.05) | 1.69× (1.62–1.75) | 2.22× (2.16–2.27) | 2.04× (1.98–2.11) | 2.10× (2.04–2.15) | 1.08× (1.06–1.11) | 0.89× (0.87–0.91) | 0.82× (0.80–0.84) | 1.07× (1.05–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.33–1.38) | 2.54× (2.49–2.59) | 2.33× (2.27–2.39) | 2.46× (2.41–2.51) | 2.78× (2.70–2.87) | 2.92× (2.87–2.98) | 3.02× (2.94–3.09) | 1.23× (1.21–1.25) | 1.35× (1.33–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.20–1.29) | 2.34× (2.26–2.42) | 2.09× (2.02–2.16) | 2.84× (2.75–2.93) | 2.45× (2.37–2.53) | 2.55× (2.47–2.63) | 2.63× (2.55–2.73) | 1.07× (1.04–1.11) | 1.18× (1.14–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.36–1.50) | 2.24× (2.19–2.29) | 1.91× (1.86–1.97) | 2.57× (2.52–2.62) | 2.45× (2.40–2.50) | 1.96× (1.92–1.99) | 2.02× (1.98–2.06) | 0.82× (0.81–0.84) | 0.90× (0.89–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.44–1.56) | 2.95× (2.84–3.06) | 2.59× (2.52–2.66) | 2.73× (2.63–2.83) | 3.09× (3.07–3.12) | 3.61× (3.48–3.74) | 3.90× (3.76–4.05) | 1.49× (1.43–1.55) | 1.71× (1.64–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.31–1.44) | 2.71× (2.59–2.84) | 2.31× (2.21–2.42) | 3.26× (3.12–3.41) | 2.73× (2.61–2.85) | 3.16× (3.02–3.30) | 3.42× (3.26–3.58) | 1.30× (1.24–1.37) | 1.50× (1.43–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.59–1.67) | 2.53× (2.47–2.59) | 2.02× (1.98–2.06) | 2.86× (2.79–2.93) | 2.61× (2.56–2.67) | 2.31× (2.26–2.36) | 2.49× (2.44–2.55) | 0.95× (0.93–0.97) | 1.09× (1.07–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.47× (2.44–2.49) | 3.44× (3.38–3.50) | 2.86× (2.83–2.89) | 5.94× (5.86–6.03) | 2.14× (2.12–2.16) | 2.17× (2.13–2.21) | 0.97× (0.95–0.99) | 1.38× (1.36–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.62) | 2.25× (2.23–2.28) | 3.98× (3.94–4.01) | 2.73× (2.70–2.76) | 5.64× (5.57–5.71) | 1.99× (1.96–2.01) | 2.02× (1.98–2.06) | 0.90× (0.88–0.92) | 1.28× (1.26–1.29) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.48× (2.45–2.52) | 4.07× (4.00–4.15) | 2.81× (2.77–2.84) | 5.84× (5.50–6.21) | 2.02× (2.00–2.05) | 2.06× (2.01–2.11) | 0.92× (0.90–0.94) | 1.30× (1.29–1.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.82× (1.73–1.92) | 3.75× (3.71–3.78) | 4.16× (4.10–4.23) | 4.45× (4.42–4.48) | 4.58× (4.54–4.62) | 4.57× (4.52–4.62) | 4.49× (4.44–4.53) | 1.48× (1.46–1.50) | 1.62× (1.60–1.63) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.55× (1.53–1.56) | 3.24× (3.13–3.35) | 3.55× (3.51–3.60) | 4.23× (4.14–4.33) | 3.83× (3.79–3.88) | 3.75× (3.70–3.80) | 3.68× (3.62–3.74) | 1.21× (1.20–1.24) | 1.33× (1.31–1.34) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.65× (1.62–1.69) | 2.52× (2.50–2.55) | 2.81× (2.77–2.86) | 3.25× (3.23–3.27) | 3.02× (2.99–3.06) | 2.40× (2.37–2.42) | 2.35× (2.31–2.40) | 0.78× (0.76–0.79) | 0.85× (0.84–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.95× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.32–0.33) | 0.38× (0.34–0.42) | 0.66× (0.64–0.67) | 0.56× (0.55–0.57) | 5.27× (5.18–5.37) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 88 MB | 0.35× (0.34–0.36) | 0.54× (0.47–0.62) | 0.61× (0.60–0.63) | 0.54× (0.52–0.55) | 3.58× (3.50–3.66) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.28–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.66–0.66) | 7.96× (7.93–7.99) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.69) | 0.65× (0.60–0.70) | 1.59× (1.54–1.65) | 0.80× (0.77–0.83) | 5.77× (5.57–5.97) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.68× (0.62–0.74) | 1.16× (1.12–1.19) | 0.71× (0.69–0.73) | 3.38× (3.28–3.47) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.15× (5.14–5.15) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.50× (0.48–0.52) | 0.38× (0.35–0.41) | 0.89× (0.86–0.93) | 0.71× (0.69–0.74) | 3.07× (2.95–3.19) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.85× (0.83–0.88) | 0.79× (0.77–0.82) | 2.49× (2.42–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.82× (0.82–0.83) | 5.15× (5.14–5.16) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.45–0.50) | 0.36× (0.33–0.39) | 0.85× (0.81–0.88) | 0.72× (0.69–0.75) | 2.91× (2.81–3.02) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.51× (0.47–0.54) | 0.81× (0.80–0.82) | 0.82× (0.80–0.85) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.25–5.28) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.40× (1.39–1.40) | 0.67× (0.67–0.68) | 6.35× (6.33–6.37) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.37× (0.34–0.41) | 1.05× (1.02–1.08) | 0.67× (0.66–0.69) | 3.34× (3.28–3.39) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.43× (0.40–0.47) | 0.89× (0.87–0.91) | 0.74× (0.71–0.76) | 2.28× (2.23–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 518 | 1.94 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 907 | 2.12 | 1.08× (1.04–1.12) | 86% | 1.02× (1.01–1.02) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,069 | 3.27 | 1.65× (1.60–1.71) | 53% | 1.21× (1.19–1.22) | 1.67× (1.62–1.71) |

**Best:** mibz2 at 4 threads (1,069 MB/s); lbzip2 at 4 threads (245 MB/s); pbz2 at 4 threads (353 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.52 | 0.01 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.9% |
| MTF + RLE2 | 1.53 | 17.8% |
| T-vector build | 1.50 | 17.4% |
| walk (the one each build uses) | 4.01 | 46.7% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.59 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.062 | (in its total) | 2.71× (2.69–2.72) |  |
| walk | 1.079 | 2.067 | (in its total) | 1.92× (1.91–1.94) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.042 | (in its total) | 0.93× (0.89–0.97) |  |
| inverse BWT, total | 1.516 | 3.171 | 1.916 | 2.10× (2.08–2.11) | 1.26× (1.25–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m51s ± 7m08s |
| 4 threads | 28m26s ± 4m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m35s ± 2m40s |
| the scanner alone (its serial part) | 11m12s ± 30.2s |
| the workers alone, over 4 | 18m35s ± 2m40s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 2.44 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,147 | 13,382,859 | \`f7d4e4f1aa08d971…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
