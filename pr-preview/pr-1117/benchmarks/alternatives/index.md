# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1117/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`ebb2dd9`](<https://github.com/lmmx/mibz2/commit/ebb2dd9f1a38d3a8e28d26ccc01b72739db2f8a0>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45958746863>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.316 s | 665 | 6.0 | 2.87× (2.76–2.98) | 7.18× (6.93–7.45) | 7.62× (7.31–7.94) | 8.63× (8.29–8.98) | 7.80× (7.45–8.17) | 7.84× (7.51–8.19) | 7.49× (7.17–7.82) | 2.26× (2.18–2.35) | 2.59× (2.49–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.179 s | 1,174 | 6.5 | 3.16× (3.02–3.31) | 7.07× (6.72–7.44) | 8.25× (8.08–8.43) | 8.24× (7.95–8.54) | 7.50× (7.27–7.74) | 13.92× (13.62–14.23) | 13.30× (13.02–13.59) | 4.02× (3.91–4.13) | 4.59× (4.46–4.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.154 s | 1,364 | 10.5 | 3.09× (2.99–3.19) | 4.92× (4.69–5.15) | 6.37× (6.00–6.77) | 6.09× (5.72–6.49) | 5.86× (5.61–6.13) | 16.07× (15.51–16.65) | 15.35× (14.81–15.91) | 4.63× (4.48–4.80) | 5.30× (5.12–5.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,290 | 3.4 | 0.56× (0.55–0.57) | 3.44× (3.29–3.60) | 4.87× (4.43–5.36) | 3.65× (3.57–3.73) | 6.40× (6.26–6.54) | 2.73× (2.67–2.79) | 2.73× (2.67–2.79) | 0.99× (0.97–1.01) | 2.05× (1.93–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,223 | 3.8 | 0.49× (0.47–0.51) | 3.14× (3.06–3.23) | 5.66× (5.48–5.84) | 3.48× (3.39–3.56) | 6.22× (5.75–6.72) | 2.58× (2.52–2.64) | 2.58× (2.52–2.64) | 0.94× (0.91–0.96) | 1.94× (1.82–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,201 | 3.8 | 0.47× (0.44–0.50) | 3.00× (2.80–3.22) | 5.46× (5.10–5.85) | 3.30× (3.08–3.54) | 5.82× (5.45–6.22) | 2.46× (2.29–2.64) | 2.46× (2.28–2.64) | 0.89× (0.83–0.96) | 1.85× (1.67–2.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,026 | 4.2 | 2.88× (2.81–2.94) | 3.75× (3.58–3.92) | 4.37× (4.19–4.57) | 4.67× (4.58–4.76) | 5.61× (5.50–5.72) | 3.12× (3.04–3.19) | 2.63× (2.57–2.68) | 2.25× (2.17–2.32) | 3.16× (3.08–3.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,338 | 5.7 | 1.93× (1.64–2.26) | 2.81× (2.45–3.23) | 3.74× (3.21–4.37) | 3.31× (2.82–3.89) | 5.12× (4.35–6.02) | 3.69× (3.15–4.32) | 3.11× (2.65–3.65) | 2.66× (2.26–3.12) | 3.74× (3.17–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,260 | 10.9 | 1.95× (1.86–2.05) | 2.20× (2.04–2.37) | 3.37× (3.17–3.58) | 2.46× (2.34–2.60) | 4.75× (4.58–4.94) | 3.86× (3.74–3.99) | 3.25× (3.11–3.40) | 2.78× (2.64–2.93) | 3.91× (3.75–4.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.084 s | 96 | 42.6 | 1.43× (1.40–1.46) | 3.20× (3.13–3.26) | 2.92× (2.87–2.97) | 2.84× (2.78–2.89) | 3.18× (3.12–3.25) | 3.65× (3.56–3.74) | 3.65× (3.58–3.71) | 1.39× (1.37–1.41) | 1.57× (1.54–1.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 156 | 48.5 | 1.38× (1.34–1.42) | 3.12× (3.01–3.24) | 2.91× (2.81–3.00) | 3.53× (3.36–3.70) | 3.37× (3.26–3.49) | 5.91× (5.72–6.11) | 5.91× (5.72–6.10) | 2.25× (2.18–2.32) | 2.55× (2.46–2.64) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 189 | 64.4 | 1.43× (1.36–1.50) | 2.88× (2.66–3.11) | 2.84× (2.51–3.22) | 3.13× (2.93–3.35) | 3.49× (3.24–3.75) | 7.21× (7.00–7.44) | 7.21× (7.05–7.37) | 2.75× (2.68–2.81) | 3.11× (3.06–3.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.088 s | 109 | 37.3 | 1.56× (1.54–1.57) | 3.89× (3.81–3.96) | 3.22× (3.14–3.29) | 3.18× (3.13–3.23) | 3.61× (3.54–3.69) | 4.61× (4.53–4.69) | 4.93× (4.88–4.99) | 1.67× (1.65–1.70) | 1.88× (1.84–1.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 175 | 43.6 | 1.46× (1.39–1.54) | 3.50× (3.29–3.73) | 3.02× (2.92–3.12) | 4.06× (3.89–4.24) | 3.57× (3.48–3.67) | 7.37× (7.14–7.60) | 7.88× (7.59–8.19) | 2.67× (2.56–2.79) | 3.00× (2.87–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.048 s | 202 | 59.5 | 1.45× (1.34–1.56) | 2.73× (2.59–2.89) | 2.75× (2.49–3.04) | 3.55× (3.29–3.83) | 3.25× (3.07–3.45) | 8.46× (7.94–9.01) | 9.05× (8.49–9.65) | 3.07× (2.87–3.28) | 3.45× (3.24–3.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 985 | 4.7 | 0.62× (0.61–0.63) | 3.04× (2.98–3.11) | 4.30× (4.00–4.61) | 3.16× (3.11–3.22) | 7.13× (7.02–7.24) | 2.32× (2.29–2.36) | 2.35× (2.30–2.39) | 0.98× (0.96–1.00) | 2.03× (1.99–2.08) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 913 | 5.4 | 0.50× (0.49–0.51) | 2.84× (2.78–2.89) | 4.93× (4.75–5.13) | 2.96× (2.90–3.02) | 6.59× (6.47–6.72) | 2.16× (2.12–2.21) | 2.18× (2.14–2.23) | 0.91× (0.87–0.95) | 1.89× (1.86–1.93) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 897 | 5.3 | 0.50× (0.49–0.50) | 2.78× (2.71–2.85) | 4.74× (4.62–4.86) | 2.94× (2.81–3.09) | 6.53× (6.43–6.63) | 2.11× (2.08–2.15) | 2.14× (2.10–2.17) | 0.89× (0.86–0.92) | 1.85× (1.81–1.89) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 0.051 s | 352 | 11.7 | 1.99× (1.96–2.03) | 4.49× (4.45–4.54) | 4.97× (4.79–5.17) | 5.14× (5.05–5.23) | 5.21× (5.11–5.31) | 5.19× (5.11–5.27) | 4.96× (4.86–5.06) | 1.72× (1.69–1.75) | 1.98× (1.93–2.03) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 0.036 s | 498 | 15.1 | 1.71× (1.42–2.05) | 4.05× (3.43–4.79) | 4.38× (4.10–4.68) | 4.61× (4.25–5.01) | 4.59× (4.42–4.78) | 7.18× (6.77–7.61) | 6.86× (6.41–7.34) | 2.38× (2.29–2.48) | 2.74× (2.56–2.92) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 0.032 s | 564 | 24.2 | 1.56× (1.51–1.62) | 2.72× (2.51–2.94) | 4.11× (3.51–4.81) | 3.73× (3.56–3.92) | 4.09× (3.92–4.26) | 8.27× (7.93–8.62) | 7.90× (7.62–8.19) | 2.74× (2.62–2.87) | 3.15× (3.03–3.28) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.86× (2.75–2.97) | 7.13× (6.87–7.40) | 7.56× (7.26–7.88) | 8.58× (8.25–8.93) | 7.76× (7.41–8.13) | 7.80× (7.47–8.14) | 7.45× (7.13–7.78) | 2.25× (2.17–2.34) | 2.56× (2.47–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.80× (2.77–2.84) | 7.06× (6.91–7.20) | 7.48× (7.38–7.59) | 8.48× (8.28–8.68) | 7.33× (7.18–7.49) | 7.26× (7.15–7.37) | 6.93× (6.85–7.02) | 2.10× (2.06–2.14) | 2.39× (2.35–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.89× (2.77–3.01) | 5.25× (5.02–5.49) | 5.91× (5.66–6.17) | 6.57× (6.22–6.94) | 5.55× (5.30–5.81) | 4.47× (4.32–4.62) | 4.27× (4.12–4.42) | 1.29× (1.24–1.34) | 1.47× (1.42–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.62) | 3.03× (2.99–3.08) | 4.12× (3.74–4.54) | 3.38× (3.32–3.44) | 5.84× (5.74–5.95) | 2.55× (2.51–2.60) | 2.55× (2.50–2.60) | 0.99× (0.98–1.01) | 1.83× (1.79–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.52–0.55) | 2.63× (2.57–2.68) | 4.65× (4.56–4.75) | 3.13× (3.06–3.20) | 5.49× (5.09–5.92) | 2.33× (2.27–2.38) | 2.32× (2.27–2.37) | 0.90× (0.88–0.92) | 1.67× (1.63–1.70) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.50× (0.46–0.53) | 2.71× (2.53–2.91) | 4.51× (4.20–4.84) | 3.02× (2.80–3.26) | 5.21× (4.86–5.59) | 2.26× (2.09–2.44) | 2.26× (2.09–2.44) | 0.88× (0.81–0.95) | 1.62× (1.49–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.66–2.78) | 3.24× (3.12–3.38) | 3.89× (3.77–4.01) | 4.37× (4.29–4.45) | 5.23× (5.13–5.33) | 2.94× (2.88–3.01) | 2.49× (2.44–2.54) | 2.15× (2.07–2.22) | 2.83× (2.75–2.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.22× (2.13–2.30) | 2.66× (2.58–2.76) | 3.19× (3.06–3.33) | 3.47× (3.33–3.62) | 4.05× (3.88–4.23) | 2.19× (2.11–2.27) | 1.86× (1.76–1.96) | 1.60× (1.49–1.71) | 2.10× (2.00–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (2.00–2.15) | 1.83× (1.75–1.92) | 2.27× (2.12–2.43) | 2.18× (2.09–2.27) | 2.38× (2.31–2.47) | 1.14× (1.11–1.18) | 0.97× (0.93–1.00) | 0.83× (0.79–0.88) | 1.10× (1.05–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.39–1.45) | 3.08× (3.03–3.13) | 2.81× (2.78–2.85) | 2.80× (2.75–2.85) | 3.14× (3.08–3.20) | 3.59× (3.50–3.69) | 3.59× (3.53–3.66) | 1.38× (1.36–1.40) | 1.50× (1.48–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.35× (1.33–1.37) | 2.92× (2.86–2.97) | 2.58× (2.47–2.69) | 3.23× (3.17–3.29) | 2.92× (2.82–3.02) | 3.16× (3.07–3.25) | 3.16× (3.11–3.21) | 1.21× (1.19–1.24) | 1.31× (1.29–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.47–1.60) | 3.05× (2.77–3.36) | 2.54× (2.25–2.87) | 3.10× (2.87–3.36) | 2.97× (2.71–3.26) | 2.39× (2.34–2.45) | 2.39× (2.36–2.43) | 0.92× (0.90–0.94) | 1.00× (0.98–1.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.53–1.56) | 3.77× (3.70–3.83) | 3.13× (3.04–3.22) | 3.14× (3.09–3.19) | 3.56× (3.49–3.63) | 4.54× (4.46–4.62) | 4.86× (4.80–4.91) | 1.66× (1.63–1.69) | 1.82× (1.78–1.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.43× (1.39–1.47) | 3.40× (3.32–3.48) | 2.70× (2.66–2.75) | 3.77× (3.62–3.93) | 3.16× (3.11–3.20) | 3.89× (3.81–3.97) | 4.16× (4.05–4.27) | 1.42× (1.39–1.46) | 1.55× (1.52–1.59) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.50–1.72) | 3.06× (2.90–3.23) | 2.47× (2.15–2.83) | 3.37× (3.11–3.65) | 2.91× (2.77–3.06) | 2.78× (2.63–2.92) | 2.97× (2.83–3.12) | 1.01× (0.97–1.07) | 1.11× (1.05–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.53× (2.50–2.56) | 3.52× (3.25–3.81) | 2.85× (2.81–2.89) | 6.24× (6.16–6.31) | 2.13× (2.11–2.15) | 2.15× (2.12–2.18) | 0.98× (0.97–1.00) | 1.50× (1.48–1.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.20× (2.16–2.23) | 3.83× (3.74–3.92) | 2.59× (2.54–2.64) | 5.49× (5.39–5.59) | 1.87× (1.84–1.91) | 1.89× (1.86–1.93) | 0.86× (0.83–0.90) | 1.32× (1.30–1.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.54) | 2.37× (2.31–2.43) | 3.81× (3.76–3.85) | 2.69× (2.56–2.82) | 5.66× (5.58–5.74) | 1.92× (1.88–1.95) | 1.94× (1.90–1.97) | 0.88× (0.86–0.91) | 1.35× (1.32–1.37) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 1.96× (1.93–2.00) | 4.28× (4.23–4.34) | 4.74× (4.54–4.95) | 5.00× (4.92–5.09) | 5.07× (4.98–5.16) | 5.05× (4.97–5.13) | 4.83× (4.74–4.93) | 1.70× (1.67–1.73) | 1.86× (1.83–1.88) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 1.66× (1.60–1.72) | 3.65× (3.42–3.89) | 3.97× (3.78–4.16) | 4.33× (4.18–4.49) | 4.08× (3.95–4.21) | 3.85× (3.73–3.97) | 3.68× (3.55–3.82) | 1.29× (1.27–1.31) | 1.41× (1.37–1.46) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 1.66× (1.61–1.72) | 2.85× (2.66–3.05) | 3.42× (3.00–3.89) | 3.54× (3.37–3.73) | 3.16× (3.08–3.24) | 2.48× (2.41–2.55) | 2.37× (2.31–2.43) | 0.83× (0.81–0.86) | 0.91× (0.89–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.81) | 11.07× (11.04–11.10) | 0.26× (0.26–0.27) | 0.29× (0.29–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.52× (0.44–0.61) | 0.72× (0.70–0.74) | 0.61× (0.59–0.62) | 5.76× (5.63–5.90) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.36× (0.34–0.37) | 0.53× (0.45–0.61) | 0.64× (0.61–0.67) | 0.56× (0.53–0.58) | 3.69× (3.53–3.84) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.27–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.83× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.97) | 0.93× (0.92–0.93) | 4.86× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.43–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.00× (7.96–8.04) | 0.66× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.66–0.72) | 0.67× (0.62–0.73) | 1.61× (1.55–1.67) | 0.81× (0.78–0.84) | 5.83× (5.63–6.04) | 0.48× (0.46–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.68× (0.65–0.70) | 0.77× (0.73–0.82) | 1.22× (1.18–1.26) | 0.75× (0.72–0.78) | 3.58× (3.46–3.70) | 0.28× (0.27–0.29) | 0.29× (0.28–0.30) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.18× (5.16–5.20) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.42× (0.41–0.43) | 0.91× (0.88–0.94) | 0.73× (0.71–0.75) | 3.13× (3.03–3.23) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.45× (0.45–0.46) | 0.87× (0.85–0.88) | 0.82× (0.81–0.83) | 2.52× (2.48–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.38× (0.35–0.40) | 0.83× (0.81–0.86) | 0.71× (0.68–0.74) | 2.86× (2.77–2.95) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.47× (0.43–0.51) | 0.82× (0.80–0.83) | 0.84× (0.81–0.86) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.06× (1.06–1.07) | 0.76× (0.75–0.77) | 5.33× (5.31–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.17–6.20) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.17–6.22) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.69× (0.68–0.69) | 6.56× (6.53–6.58) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.30–0.31) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 36 MB | 0.46× (0.44–0.49) | 0.44× (0.39–0.49) | 1.07× (1.01–1.13) | 0.69× (0.66–0.73) | 3.51× (3.34–3.68) | 0.21× (0.20–0.22) | 0.22× (0.21–0.23) | 0.34× (0.32–0.36) | 0.16× (0.16–0.17) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.46× (0.42–0.50) | 0.90× (0.87–0.93) | 0.74× (0.71–0.77) | 2.33× (2.26–2.39) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 665 | 1.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,174 | 1.63 | 1.07× (1.03–1.12) | 89% | 1.06× (1.02–1.11) | 1.05× (1.02–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,364 | 2.64 | 1.75× (1.66–1.83) | 51% | 1.28× (1.22–1.36) | 1.76× (1.66–1.88) |

**Best:** mibz2 at 4 threads (1,364 MB/s); lbzip2 at 4 threads (281 MB/s); pbz2 at 4 threads (445 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.07, lbzip2 1t 0.08, lbzip2 2t 0.32, lbzip2 4t 0.42, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.43 | 0.01 | 0.07 | 0.154 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3991 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.74 | 11.2% |
| MTF + RLE2 | 1.39 | 21.0% |
| T-vector build | 1.28 | 19.3% |
| walk (the one each build uses) | 2.85 | 42.8% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.15 | 2.3% |
| CRC | 0.25 | 3.7% |
| **total** | 6.65 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.324 | 0.777 | (in its total) | 2.39× (2.37–2.41) |  |
| walk | 0.695 | 1.724 | (in its total) | 2.48× (2.42–2.53) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.033 | 0.034 | (in its total) | 1.02× (0.96–1.09) |  |
| inverse BWT, total | 1.051 | 2.535 | 1.435 | 2.40× (2.36–2.45) | 1.35× (1.33–1.38) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 46m16s ± 5m48s |
| 4 threads | 22m34s ± 3m14s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m36s ± 2m19s |
| the scanner alone (its serial part) | 6m50s ± 30.2s |
| the workers alone, over 4 | 15m36s ± 2m19s |

At 4 threads the workers bound it. Per byte: the scanner 4.0 ns a byte of bz2, the workers 2.05 ns a byte of output at 4 threads (CPU, summed over them), 1.29 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,762,177 | 17,991,040 | \`04b0571f08562a96…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 3990.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
