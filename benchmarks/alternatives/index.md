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

The latest record: [`39877bf`](<https://github.com/lmmx/mibz2/commit/39877bfd3a977fac7ed530f196f57768e5a22b7c>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42652992472>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.765 s | 274 | 11.0 | 2.07× (2.05–2.08) | 4.63× (4.60–4.66) | 4.93× (4.89–4.98) | 5.55× (5.51–5.59) | 4.95× (4.91–4.99) | 5.31× (5.27–5.35) | 5.06× (5.01–5.10) | 1.59× (1.58–1.60) | 1.48× (1.47–1.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.406 s | 517 | 11.4 | 2.28× (2.22–2.35) | 4.74× (4.62–4.86) | 5.63× (5.50–5.76) | 5.70× (5.59–5.82) | 5.02× (4.92–5.12) | 9.94× (9.76–10.13) | 9.47× (9.31–9.63) | 2.97× (2.92–3.03) | 2.78× (2.73–2.83) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.331 s | 634 | 18.3 | 2.41× (2.36–2.46) | 4.11× (4.06–4.16) | 4.78× (4.71–4.84) | 4.80× (4.72–4.87) | 4.66× (4.57–4.75) | 12.27× (12.08–12.47) | 11.69× (11.53–11.86) | 3.67× (3.62–3.72) | 3.43× (3.38–3.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 926 | 3.5 | 0.57× (0.57–0.58) | 3.32× (3.27–3.36) | 4.25× (4.13–4.37) | 3.76× (3.71–3.81) | 6.22× (6.16–6.29) | 2.80× (2.76–2.83) | 2.80× (2.76–2.83) | 1.15× (1.13–1.16) | 0.96× (0.95–0.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 930 | 3.5 | 0.54× (0.53–0.56) | 2.85× (2.79–2.91) | 5.82× (5.66–5.98) | 3.77× (3.70–3.85) | 6.18× (6.07–6.30) | 2.80× (2.74–2.85) | 2.80× (2.75–2.85) | 1.15× (1.13–1.17) | 0.96× (0.94–0.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 907 | 3.6 | 0.54× (0.53–0.55) | 2.79× (2.74–2.84) | 5.59× (5.45–5.73) | 3.69× (3.64–3.75) | 6.05× (5.95–6.16) | 2.73× (2.68–2.78) | 2.73× (2.69–2.77) | 1.12× (1.11–1.14) | 0.94× (0.93–0.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.040 s | 485 | 6.5 | 2.66× (2.63–2.69) | 3.25× (3.22–3.28) | 3.72× (3.69–3.76) | 4.28× (4.23–4.32) | 4.63× (4.58–4.68) | 3.13× (3.10–3.16) | 2.86× (2.83–2.89) | 1.99× (1.97–2.02) | 2.00× (1.98–2.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 784 | 7.6 | 2.34× (2.21–2.48) | 3.34× (2.83–3.93) | 4.21× (3.90–4.53) | 3.85× (3.68–4.03) | 5.19× (4.96–5.44) | 4.97× (4.76–5.20) | 4.54× (4.35–4.75) | 3.17× (3.02–3.34) | 3.19× (3.05–3.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 745 | 14.4 | 2.05× (2.00–2.10) | 2.01× (1.95–2.08) | 2.97× (2.77–3.19) | 2.47× (2.32–2.63) | 4.20× (4.11–4.30) | 4.84× (4.70–4.98) | 4.42× (4.30–4.54) | 3.09× (2.99–3.18) | 3.10× (3.02–3.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.156 s | 51 | 59.1 | 1.31× (1.27–1.34) | 2.53× (2.47–2.59) | 2.22× (2.15–2.28) | 2.29× (2.24–2.35) | 2.68× (2.49–2.88) | 2.99× (2.93–3.06) | 2.97× (2.90–3.05) | 1.27× (1.24–1.30) | 1.29× (1.26–1.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.094 s | 85 | 64.2 | 1.23× (1.18–1.27) | 2.41× (2.32–2.50) | 2.26× (2.21–2.31) | 2.96× (2.83–3.09) | 2.69× (2.60–2.79) | 4.92× (4.75–5.10) | 4.89× (4.74–5.04) | 2.09× (2.01–2.17) | 2.12× (2.04–2.21) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 85.2 | 1.40× (1.37–1.44) | 2.35× (2.30–2.39) | 2.31× (2.23–2.40) | 2.91× (2.75–3.09) | 2.88× (2.82–2.94) | 6.26× (6.16–6.37) | 6.22× (6.10–6.34) | 2.66× (2.59–2.73) | 2.70× (2.64–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.171 s | 56 | 54.1 | 1.37× (1.35–1.38) | 2.93× (2.89–2.97) | 2.43× (2.39–2.48) | 2.52× (2.48–2.55) | 2.85× (2.82–2.89) | 3.69× (3.64–3.74) | 3.80× (3.77–3.84) | 1.50× (1.48–1.52) | 1.55× (1.52–1.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.102 s | 95 | 59.5 | 1.27× (1.25–1.29) | 2.97× (2.68–3.30) | 2.41× (2.38–2.44) | 3.42× (3.36–3.48) | 2.93× (2.88–2.98) | 6.21× (6.10–6.32) | 6.40× (6.28–6.52) | 2.52× (2.48–2.57) | 2.61× (2.55–2.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.084 s | 114 | 84.9 | 1.43× (1.39–1.47) | 2.63× (2.55–2.71) | 2.38× (2.31–2.44) | 3.22× (3.13–3.32) | 2.89× (2.81–2.99) | 7.44× (7.24–7.65) | 7.67× (7.45–7.90) | 3.02× (2.95–3.10) | 3.12× (3.05–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 702 | 4.8 | 0.62× (0.61–0.63) | 2.85× (2.79–2.92) | 4.18× (4.07–4.29) | 3.20× (3.13–3.27) | 6.66× (6.51–6.81) | 2.39× (2.34–2.44) | 2.39× (2.34–2.44) | 1.04× (1.02–1.06) | 1.44× (1.40–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 5.1 | 0.52× (0.51–0.53) | 2.75× (2.70–2.80) | 4.84× (4.65–5.03) | 3.10× (3.05–3.14) | 6.47× (6.32–6.61) | 2.29× (2.25–2.33) | 2.30× (2.26–2.34) | 1.00× (0.98–1.01) | 1.38× (1.36–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 662 | 5.0 | 0.53× (0.52–0.54) | 2.72× (2.65–2.79) | 4.78× (4.67–4.90) | 3.07× (2.99–3.15) | 6.40× (6.19–6.61) | 2.27× (2.21–2.32) | 2.27× (2.21–2.33) | 0.98× (0.96–1.01) | 1.36× (1.33–1.40) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.097 s | 126 | 24.2 | 1.31× (1.30–1.33) | 2.85× (2.82–2.89) | 3.21× (3.18–3.23) | 3.32× (3.28–3.36) | 3.41× (3.38–3.44) | 3.65× (3.62–3.68) | 3.46× (3.43–3.50) | 1.07× (1.06–1.08) | 1.04× (1.03–1.05) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.059 s | 208 | 27.3 | 1.20× (1.05–1.36) | 2.46× (2.19–2.77) | 3.00× (2.64–3.41) | 3.48× (3.02–4.02) | 3.24× (2.84–3.69) | 5.57× (4.88–6.36) | 5.29× (4.64–6.02) | 1.64× (1.44–1.86) | 1.59× (1.40–1.81) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.047 s | 261 | 37.9 | 1.41× (1.37–1.44) | 2.55× (2.42–2.68) | 3.24× (3.00–3.50) | 3.57× (3.53–3.62) | 3.61× (3.55–3.67) | 7.61× (7.48–7.74) | 7.22× (7.09–7.36) | 2.23× (2.19–2.28) | 2.17× (2.13–2.21) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.06× (2.05–2.08) | 4.61× (4.58–4.65) | 4.92× (4.87–4.97) | 5.54× (5.50–5.58) | 4.94× (4.90–4.98) | 5.30× (5.26–5.34) | 5.05× (5.00–5.09) | 1.59× (1.57–1.60) | 1.48× (1.46–1.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.11× (2.09–2.12) | 4.75× (4.67–4.83) | 5.18× (5.13–5.22) | 5.79× (5.76–5.82) | 4.88× (4.86–4.91) | 5.10× (5.06–5.13) | 4.86× (4.83–4.88) | 1.53× (1.52–1.54) | 1.42× (1.41–1.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.23× (2.18–2.27) | 4.20× (4.16–4.24) | 4.41× (4.35–4.48) | 4.89× (4.83–4.95) | 4.29× (4.24–4.34) | 3.19× (3.15–3.23) | 3.04× (3.01–3.07) | 0.96× (0.95–0.96) | 0.89× (0.88–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.87× (2.82–2.91) | 3.95× (3.90–4.01) | 3.56× (3.51–3.61) | 5.85× (5.78–5.92) | 2.67× (2.63–2.71) | 2.67× (2.63–2.71) | 1.14× (1.12–1.16) | 0.91× (0.90–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.85× (2.79–2.91) | 5.19× (5.08–5.30) | 3.54× (3.47–3.61) | 5.73× (5.63–5.84) | 2.63× (2.58–2.68) | 2.63× (2.58–2.67) | 1.12× (1.10–1.14) | 0.90× (0.88–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.23× (3.17–3.31) | 5.10× (5.00–5.21) | 3.53× (3.47–3.58) | 5.69× (5.57–5.80) | 2.61× (2.56–2.66) | 2.61× (2.57–2.65) | 1.11× (1.10–1.13) | 0.89× (0.87–0.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.59× (2.57–2.62) | 3.02× (3.00–3.05) | 3.54× (3.50–3.58) | 4.14× (4.10–4.19) | 4.48× (4.44–4.53) | 3.04× (3.01–3.07) | 2.78× (2.75–2.81) | 1.95× (1.93–1.98) | 1.86× (1.85–1.88) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.35–2.42) | 2.78× (2.74–2.83) | 3.28× (3.23–3.34) | 3.62× (3.57–3.67) | 3.94× (3.88–4.00) | 2.59× (2.56–2.62) | 2.37× (2.34–2.40) | 1.66× (1.63–1.70) | 1.58× (1.57–1.60) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.03–2.17) | 1.77× (1.70–1.85) | 2.21× (2.14–2.28) | 2.19× (2.12–2.26) | 2.29× (2.20–2.38) | 1.37× (1.33–1.42) | 1.25× (1.21–1.29) | 0.88× (0.85–0.91) | 0.84× (0.81–0.87) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.30× (1.27–1.34) | 2.47× (2.42–2.53) | 2.18× (2.12–2.25) | 2.28× (2.23–2.33) | 2.66× (2.47–2.86) | 2.97× (2.91–3.04) | 2.95× (2.88–3.02) | 1.27× (1.24–1.30) | 1.25× (1.22–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.24–1.27) | 2.46× (2.43–2.49) | 2.13× (2.10–2.16) | 2.87× (2.82–2.92) | 2.47× (2.44–2.50) | 2.74× (2.70–2.79) | 2.72× (2.70–2.75) | 1.17× (1.15–1.19) | 1.15× (1.14–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.47–1.59) | 2.59× (2.55–2.64) | 2.17× (2.13–2.21) | 2.90× (2.84–2.95) | 2.55× (2.51–2.59) | 2.07× (2.04–2.10) | 2.06× (2.03–2.09) | 0.88× (0.87–0.90) | 0.87× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.36× (1.35–1.38) | 2.88× (2.85–2.92) | 2.38× (2.34–2.43) | 2.50× (2.47–2.53) | 2.84× (2.80–2.87) | 3.66× (3.62–3.71) | 3.77× (3.74–3.81) | 1.49× (1.48–1.51) | 1.52× (1.49–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.29–1.31) | 2.83× (2.79–2.87) | 2.30× (2.28–2.32) | 3.24× (3.20–3.29) | 2.69× (2.67–2.71) | 3.31× (3.28–3.35) | 3.41× (3.38–3.45) | 1.35× (1.34–1.37) | 1.37× (1.35–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.55–1.63) | 2.89× (2.83–2.95) | 2.16× (2.11–2.22) | 3.04× (2.97–3.12) | 2.61× (2.55–2.68) | 2.36× (2.30–2.43) | 2.43× (2.37–2.50) | 0.96× (0.94–0.99) | 0.98× (0.96–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.38× (2.34–2.42) | 3.44× (3.39–3.50) | 2.98× (2.93–3.02) | 6.08× (5.99–6.17) | 2.25× (2.22–2.28) | 2.25× (2.22–2.28) | 1.04× (1.03–1.05) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.25× (2.20–2.30) | 4.19× (4.11–4.27) | 2.88× (2.83–2.93) | 5.81× (5.66–5.95) | 2.12× (2.07–2.16) | 2.12× (2.08–2.16) | 0.98× (0.96–1.00) | 0.78× (0.77–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.59) | 2.65× (2.60–2.71) | 4.19× (4.11–4.26) | 2.94× (2.87–3.01) | 5.90× (5.72–6.08) | 2.15× (2.10–2.20) | 2.15× (2.09–2.20) | 0.99× (0.97–1.02) | 0.79× (0.77–0.81) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.31× (1.29–1.32) | 2.78× (2.76–2.81) | 3.12× (3.09–3.15) | 3.28× (3.24–3.32) | 3.37× (3.34–3.39) | 3.60× (3.57–3.64) | 3.42× (3.39–3.45) | 1.07× (1.06–1.08) | 1.01× (1.00–1.02) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.26× (1.25–1.28) | 2.63× (2.60–2.66) | 3.08× (3.06–3.09) | 3.41× (3.37–3.45) | 3.12× (3.10–3.14) | 3.20× (3.17–3.22) | 3.04× (3.02–3.05) | 0.95× (0.94–0.96) | 0.90× (0.89–0.91) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.55× (1.46–1.64) | 2.67× (2.63–2.71) | 3.05× (2.94–3.15) | 3.17× (3.12–3.22) | 3.04× (3.00–3.08) | 2.31× (2.29–2.34) | 2.19× (2.17–2.22) | 0.69× (0.68–0.70) | 0.65× (0.64–0.66) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.28× (0.28–0.29) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.13× (12.11–12.14) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.39× (0.38–0.39) | 0.51× (0.43–0.61) | 0.80× (0.79–0.82) | 0.68× (0.67–0.69) | 6.43× (6.37–6.49) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.59× (0.48–0.71) | 0.72× (0.71–0.73) | 0.65× (0.64–0.66) | 4.28× (4.26–4.29) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.30–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.42× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.66) | 8.06× (8.04–8.07) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.65× (0.62–0.68) | 1.61× (1.58–1.64) | 0.80× (0.78–0.81) | 5.85× (5.73–5.97) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.63–0.65) | 0.70× (0.63–0.77) | 1.15× (1.12–1.18) | 0.72× (0.70–0.73) | 3.46× (3.40–3.53) | 0.27× (0.27–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.24× (5.22–5.25) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.41× (0.37–0.44) | 0.91× (0.88–0.95) | 0.72× (0.69–0.75) | 3.16× (3.04–3.29) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.47) | 0.87× (0.85–0.89) | 0.80× (0.79–0.82) | 2.54× (2.48–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.84× (0.83–0.84) | 5.24× (5.23–5.25) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.39) | 0.86× (0.83–0.89) | 0.71× (0.68–0.74) | 2.95× (2.85–3.04) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.59× (0.59–0.60) | 0.50× (0.47–0.52) | 0.81× (0.80–0.82) | 0.81× (0.79–0.84) | 2.34× (2.32–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.61–1.63) | 0.77× (0.77–0.78) | 5.37× (5.34–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.39× (5.37–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.47–1.47) | 0.68× (0.68–0.69) | 6.65× (6.64–6.66) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.32) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.40× (0.36–0.44) | 1.12× (1.08–1.15) | 0.70× (0.69–0.72) | 3.51× (3.42–3.60) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.35) | 0.17× (0.17–0.17) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 53 MB | 0.52× (0.50–0.54) | 0.48× (0.44–0.52) | 0.96× (0.93–0.99) | 0.77× (0.75–0.80) | 2.45× (2.39–2.52) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 274 | 3.66 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 517 | 3.79 | 1.04× (1.03–1.05) | 94% | 1.07× (1.05–1.09) | 1.06× (1.05–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 634 | 6.10 | 1.66× (1.65–1.68) | 58% | 1.51× (1.50–1.53) | 1.79× (1.76–1.82) |

**Best:** mibz2 at 4 threads (634 MB/s); lbzip2 at 4 threads (154 MB/s); pbz2 at 4 threads (264 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.66, mibz2 4t 1.04, lbzip2 1t 0.05, lbzip2 2t 0.22, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 7%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.04 | 0.03 | 0.09 | 0.331 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3002 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.19 | 11.5% |
| MTF + RLE2 | 1.68 | 16.3% |
| T-vector build | 1.75 | 16.9% |
| walk (the one each build uses) | 5.26 | 50.9% |
| of which: segment assembly | 0.28 | 2.8% |
| RLE1 | 0.28 | 2.7% |
| CRC | 0.14 | 1.3% |
| **total** | 10.33 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.563 | 1.334 | (in its total) | 2.37× (2.36–2.37) |  |
| walk | 1.650 | 2.960 | (in its total) | 1.79× (1.79–1.80) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.094 | 0.088 | (in its total) | 0.93× (0.90–0.96) |  |
| inverse BWT, total | 2.308 | 4.385 | 2.376 | 1.90× (1.89–1.91) | 1.03× (1.02–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h51m ± 13m13s |
| 4 threads | 48m10s ± 6m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m45s ± 5m19s |
| the scanner alone (its serial part) | 17m10s ± 26.3s |
| the workers alone, over 4 | 37m45s ± 5m19s |

At 4 threads the workers bound it. Per byte: the scanner 10.0 ns a byte of bz2, the workers 4.95 ns a byte of output at 4 threads (CPU, summed over them), 3.09 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3002.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
