# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1183/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`7cbbb78`](<https://github.com/lmmx/mibz2/commit/7cbbb78d68584e2005af204a977d765218eb9b9c>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46023786825>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.388 s | 541 | 6.9 | 3.29× (3.19–3.38) | 7.58× (7.49–7.66) | 7.83× (7.71–7.95) | 8.92× (8.74–9.10) | 7.64× (7.56–7.71) | 8.14× (7.93–8.36) | 7.87× (7.71–8.02) | 2.42× (2.40–2.45) | 2.67× (2.65–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.225 s | 932 | 7.6 | 3.42× (3.15–3.70) | 6.89× (6.42–7.40) | 8.06× (7.52–8.64) | 8.13× (7.68–8.60) | 7.09× (6.75–7.45) | 13.91× (13.01–14.87) | 13.44× (12.73–14.18) | 4.14× (3.89–4.41) | 4.57× (4.29–4.86) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.187 s | 1,120 | 12.1 | 3.38× (3.28–3.48) | 5.04× (4.87–5.22) | 6.60× (6.37–6.84) | 6.21× (5.98–6.44) | 6.10× (5.55–6.70) | 16.87× (16.48–17.28) | 16.30× (16.08–16.52) | 5.02× (4.94–5.11) | 5.54× (5.46–5.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,070 | 3.8 | 0.55× (0.55–0.56) | 3.29× (3.25–3.34) | 4.39× (4.33–4.45) | 3.46× (3.41–3.50) | 5.89× (5.74–6.04) | 2.56× (2.53–2.59) | 2.60× (2.51–2.69) | 0.96× (0.94–0.99) | 2.01× (1.82–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,039 | 4.0 | 0.50× (0.48–0.52) | 2.64× (2.58–2.69) | 5.27× (5.15–5.39) | 3.31× (3.24–3.38) | 5.59× (5.51–5.68) | 2.45× (2.39–2.50) | 2.48× (2.37–2.60) | 0.92× (0.89–0.96) | 1.93× (1.74–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,036 | 4.0 | 0.50× (0.49–0.51) | 2.66× (2.59–2.72) | 5.29× (5.18–5.42) | 3.33× (3.25–3.41) | 5.64× (5.53–5.76) | 2.46× (2.40–2.51) | 2.49× (2.39–2.60) | 0.93× (0.91–0.95) | 1.93× (1.77–2.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 760 | 5.2 | 2.80× (2.78–2.82) | 3.22× (3.14–3.31) | 3.97× (3.94–4.01) | 4.26× (4.23–4.29) | 4.67× (4.62–4.72) | 2.87× (2.85–2.89) | 2.35× (2.33–2.37) | 2.21× (2.17–2.26) | 3.24× (3.14–3.36) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,136 | 6.5 | 2.04× (1.69–2.47) | 2.73× (2.45–3.05) | 3.95× (3.21–4.86) | 3.44× (2.88–4.11) | 4.73× (4.08–5.50) | 3.66× (3.03–4.42) | 3.00× (2.49–3.62) | 2.83× (2.35–3.40) | 4.14× (3.38–5.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,034 | 12.6 | 2.04× (1.97–2.11) | 1.93× (1.75–2.13) | 3.44× (3.32–3.56) | 2.42× (2.28–2.57) | 4.51× (4.37–4.65) | 3.93× (3.83–4.04) | 3.22× (3.13–3.31) | 3.04× (2.92–3.16) | 4.45× (4.21–4.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.103 s | 78 | 48.2 | 1.50× (1.46–1.54) | 2.87× (2.77–2.98) | 2.61× (2.50–2.74) | 2.68× (2.59–2.76) | 2.99× (2.89–3.10) | 3.19× (3.09–3.29) | 3.38× (3.28–3.49) | 1.35× (1.32–1.39) | 1.54× (1.50–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.063 s | 126 | 54.0 | 1.41× (1.37–1.44) | 2.74× (2.66–2.82) | 2.57× (2.49–2.65) | 3.42× (3.31–3.54) | 3.13× (3.05–3.22) | 5.23× (5.11–5.35) | 5.55× (5.44–5.66) | 2.22× (2.16–2.29) | 2.53× (2.47–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.051 s | 157 | 70.7 | 1.48× (1.44–1.52) | 2.43× (2.34–2.52) | 2.40× (2.30–2.50) | 3.01× (2.86–3.16) | 3.25× (3.14–3.36) | 6.52× (6.37–6.68) | 6.92× (6.76–7.09) | 2.77× (2.71–2.84) | 3.16× (3.08–3.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.114 s | 84 | 44.5 | 1.62× (1.59–1.66) | 3.23× (3.13–3.32) | 2.82× (2.71–2.93) | 2.94× (2.87–3.00) | 3.26× (3.18–3.34) | 3.87× (3.79–3.96) | 4.32× (4.21–4.43) | 1.66× (1.62–1.70) | 1.93× (1.89–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.068 s | 140 | 49.9 | 1.52× (1.36–1.69) | 2.96× (2.83–3.08) | 2.72× (2.55–2.91) | 3.75× (3.52–3.99) | 3.32× (3.10–3.55) | 6.37× (5.98–6.79) | 7.10× (6.65–7.59) | 2.73× (2.55–2.92) | 3.17× (2.97–3.39) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 171 | 67.5 | 1.59× (1.54–1.64) | 2.64× (2.56–2.73) | 2.66× (2.52–2.81) | 3.55× (3.40–3.71) | 3.27× (3.19–3.35) | 7.84× (7.65–8.03) | 8.74× (8.55–8.93) | 3.36× (3.25–3.47) | 3.90× (3.81–4.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 826 | 5.1 | 0.61× (0.59–0.63) | 3.34× (3.24–3.45) | 4.18× (4.06–4.31) | 3.01× (2.92–3.10) | 6.60× (6.40–6.81) | 2.26× (2.15–2.37) | 2.26× (2.19–2.33) | 0.95× (0.91–1.00) | 1.68× (1.63–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 5.8 | 0.49× (0.48–0.50) | 2.36× (2.32–2.41) | 4.78× (4.64–4.93) | 2.88× (2.80–2.96) | 6.10× (5.95–6.26) | 2.11× (2.09–2.14) | 2.12× (2.07–2.16) | 0.89× (0.85–0.94) | 1.58× (1.55–1.61) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 749 | 5.7 | 0.49× (0.48–0.50) | 2.32× (2.29–2.36) | 4.62× (4.52–4.72) | 2.80× (2.76–2.85) | 6.09× (5.96–6.23) | 2.07× (2.02–2.13) | 2.08× (2.05–2.11) | 0.87× (0.84–0.91) | 1.55× (1.52–1.57) | yes |
| `text` (2.07 MB → 14 MB, 6.8×) | 1 | 0.050 s | 278 | 13.7 | 2.01× (1.97–2.05) | 4.06× (3.99–4.12) | 4.53× (4.47–4.60) | 4.79× (4.72–4.86) | 5.03× (4.96–5.10) | 4.72× (4.67–4.78) | 4.82× (4.75–4.89) | 1.64× (1.62–1.66) | 1.89× (1.81–1.98) | yes |
| `text` (2.07 MB → 14 MB, 6.8×) | 2 | 0.037 s | 382 | 18.1 | 1.53× (1.43–1.63) | 3.14× (2.80–3.52) | 4.02× (3.67–4.41) | 4.07× (3.67–4.50) | 4.33× (3.93–4.77) | 6.19× (5.65–6.78) | 6.31× (5.73–6.95) | 2.15× (1.95–2.37) | 2.48× (2.26–2.72) | yes |
| `text` (2.07 MB → 14 MB, 6.8×) | 4 | 0.034 s | 409 | 27.7 | 1.58× (1.42–1.75) | 2.61× (2.32–2.94) | 3.95× (3.47–4.50) | 3.63× (3.32–3.97) | 4.04× (3.88–4.20) | 6.80× (6.54–7.07) | 6.93× (6.61–7.27) | 2.37× (2.27–2.47) | 2.72× (2.52–2.95) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.28× (3.18–3.37) | 7.53× (7.45–7.62) | 7.80× (7.68–7.92) | 8.88× (8.70–9.06) | 7.61× (7.53–7.68) | 8.11× (7.90–8.33) | 7.83× (7.68–7.99) | 2.42× (2.39–2.44) | 2.65× (2.62–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.22× (3.12–3.32) | 7.10× (6.94–7.25) | 7.51× (7.26–7.76) | 8.48× (8.33–8.62) | 7.06× (6.92–7.20) | 7.35× (7.19–7.51) | 7.10× (7.02–7.17) | 2.19× (2.15–2.23) | 2.40× (2.37–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.18× (3.10–3.27) | 5.34× (5.19–5.50) | 6.16× (5.96–6.37) | 6.58× (6.39–6.78) | 5.75× (5.18–6.39) | 4.61× (4.45–4.77) | 4.45× (4.35–4.55) | 1.37× (1.34–1.41) | 1.51× (1.47–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.84× (2.79–2.88) | 3.81× (3.73–3.89) | 3.24× (3.20–3.28) | 5.46× (5.32–5.60) | 2.42× (2.39–2.45) | 2.45× (2.37–2.54) | 0.97× (0.94–0.99) | 1.60× (1.57–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.52–0.55) | 2.50× (2.45–2.55) | 4.56× (4.48–4.65) | 3.02× (2.96–3.08) | 5.02× (4.96–5.08) | 2.23× (2.19–2.28) | 2.27× (2.17–2.37) | 0.89× (0.86–0.93) | 1.48× (1.44–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 2.81× (2.74–2.89) | 4.65× (4.57–4.73) | 3.10× (3.03–3.17) | 5.14× (5.04–5.25) | 2.29× (2.24–2.34) | 2.32× (2.22–2.42) | 0.91× (0.89–0.93) | 1.51× (1.49–1.54) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.66–2.70) | 2.97× (2.90–3.05) | 3.56× (3.53–3.58) | 4.05× (4.02–4.08) | 4.44× (4.38–4.49) | 2.75× (2.73–2.77) | 2.27× (2.25–2.28) | 2.14× (2.10–2.18) | 2.85× (2.73–2.98) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.23–2.39) | 2.51× (2.38–2.64) | 3.41× (3.28–3.54) | 3.36× (3.21–3.52) | 3.65× (3.48–3.83) | 2.16× (2.06–2.27) | 1.78× (1.70–1.87) | 1.68× (1.61–1.75) | 2.24× (2.10–2.40) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.14× (2.06–2.22) | 1.73× (1.65–1.81) | 2.33× (2.24–2.41) | 2.11× (2.04–2.19) | 2.20× (2.13–2.28) | 1.12× (1.09–1.16) | 0.93× (0.90–0.96) | 0.87× (0.84–0.91) | 1.17× (1.10–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.45–1.53) | 2.79× (2.69–2.88) | 2.54× (2.44–2.65) | 2.65× (2.57–2.74) | 2.96× (2.86–3.06) | 3.15× (3.05–3.25) | 3.34× (3.24–3.45) | 1.35× (1.31–1.39) | 1.48× (1.44–1.52) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.38–1.44) | 2.66× (2.61–2.72) | 2.36× (2.32–2.40) | 3.23× (3.13–3.34) | 2.76× (2.70–2.82) | 2.85× (2.80–2.90) | 3.02× (2.97–3.07) | 1.22× (1.20–1.24) | 1.34× (1.32–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.57–1.66) | 2.59× (2.46–2.73) | 2.20× (2.12–2.28) | 2.98× (2.81–3.17) | 2.83× (2.71–2.95) | 2.20× (2.14–2.26) | 2.33× (2.27–2.40) | 0.94× (0.92–0.97) | 1.03× (1.01–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.61× (1.58–1.65) | 3.14× (3.05–3.24) | 2.75× (2.66–2.85) | 2.91× (2.84–2.97) | 3.22× (3.15–3.30) | 3.83× (3.75–3.91) | 4.27× (4.16–4.38) | 1.65× (1.61–1.69) | 1.88× (1.83–1.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.43–1.55) | 2.94× (2.82–3.06) | 2.51× (2.41–2.62) | 3.52× (3.41–3.64) | 2.96× (2.83–3.09) | 3.38× (3.27–3.50) | 3.77× (3.62–3.92) | 1.46× (1.40–1.51) | 1.66× (1.59–1.72) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.75× (1.70–1.81) | 2.91× (2.75–3.09) | 2.33× (2.16–2.51) | 3.18× (3.07–3.29) | 2.93× (2.86–3.00) | 2.53× (2.48–2.57) | 2.81× (2.76–2.87) | 1.09× (1.07–1.11) | 1.24× (1.21–1.26) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.64–0.67) | 2.41× (2.34–2.48) | 3.34× (3.24–3.43) | 2.75× (2.68–2.83) | 5.89× (5.74–6.05) | 2.10× (2.01–2.20) | 2.10× (2.04–2.16) | 0.97× (0.91–1.02) | 1.33× (1.29–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.53–0.56) | 2.10× (2.06–2.14) | 3.88× (3.71–4.05) | 2.59× (2.52–2.65) | 5.24× (5.12–5.36) | 1.88× (1.85–1.91) | 1.88× (1.85–1.91) | 0.87× (0.82–0.92) | 1.19× (1.16–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.31× (2.26–2.37) | 3.82× (3.74–3.90) | 2.59× (2.54–2.63) | 5.38× (5.27–5.49) | 1.90× (1.85–1.96) | 1.90× (1.87–1.93) | 0.87× (0.83–0.92) | 1.20× (1.17–1.23) |
| `text` (2.07 MB → 14 MB, 6.8×) | 1 | 1.97× (1.94–2.01) | 3.86× (3.83–3.90) | 4.36× (4.29–4.43) | 4.67× (4.60–4.73) | 4.90× (4.83–4.96) | 4.60× (4.55–4.65) | 4.69× (4.63–4.75) | 1.62× (1.60–1.64) | 1.79× (1.76–1.82) |
| `text` (2.07 MB → 14 MB, 6.8×) | 2 | 1.60× (1.57–1.64) | 3.16× (3.07–3.24) | 3.61× (3.53–3.69) | 4.10× (4.03–4.18) | 3.85× (3.75–3.96) | 3.45× (3.38–3.52) | 3.52× (3.43–3.62) | 1.22× (1.19–1.25) | 1.34× (1.30–1.38) |
| `text` (2.07 MB → 14 MB, 6.8×) | 4 | 1.79× (1.70–1.89) | 2.90× (2.51–3.35) | 3.43× (2.92–4.02) | 3.53× (3.13–3.99) | 3.36× (3.16–3.57) | 2.25× (2.18–2.33) | 2.30× (2.21–2.39) | 0.79× (0.76–0.83) | 0.88× (0.84–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.32–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.80) | 11.06× (11.05–11.08) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.37) | 0.46× (0.38–0.56) | 0.73× (0.71–0.75) | 0.62× (0.60–0.63) | 5.85× (5.72–5.99) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.38) | 0.51× (0.43–0.59) | 0.65× (0.62–0.68) | 0.56× (0.53–0.59) | 3.75× (3.56–3.95) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.83× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.66–0.67) | 8.00× (7.99–8.01) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.66–0.71) | 0.63× (0.57–0.68) | 1.62× (1.57–1.69) | 0.82× (0.79–0.84) | 5.90× (5.69–6.13) | 0.48× (0.46–0.50) | 0.50× (0.48–0.52) | 0.55× (0.53–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.68× (0.67–0.69) | 0.70× (0.66–0.75) | 1.25× (1.22–1.28) | 0.77× (0.75–0.79) | 3.65× (3.58–3.73) | 0.29× (0.28–0.29) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.17× (5.15–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.42× (0.41–0.43) | 0.90× (0.87–0.92) | 0.72× (0.69–0.75) | 3.10× (3.01–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.80× (0.78–0.83) | 2.51× (2.44–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.20× (5.20–5.21) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.37× (0.35–0.40) | 0.82× (0.79–0.86) | 0.68× (0.64–0.72) | 2.84× (2.73–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.60) | 0.51× (0.47–0.54) | 0.80× (0.79–0.81) | 0.81× (0.79–0.84) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.32× (5.32–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.30× (0.29–0.30) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.18–6.21) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.91) | 6.22× (6.20–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.07 MB → 14 MB, 6.8×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.65× (0.64–0.66) | 6.39× (6.36–6.42) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.07 MB → 14 MB, 6.8×) | 2 | 33 MB | 0.51× (0.50–0.52) | 0.47× (0.42–0.52) | 1.18× (1.14–1.21) | 0.76× (0.75–0.78) | 3.80× (3.73–3.87) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.38× (0.37–0.38) | 0.18× (0.18–0.19) |
| `text` (2.07 MB → 14 MB, 6.8×) | 4 | 50 MB | 0.56× (0.54–0.58) | 0.49× (0.45–0.54) | 1.01× (1.00–1.03) | 0.84× (0.81–0.86) | 2.58× (2.53–2.64) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 541 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 932 | 2.05 | 1.10× (1.09–1.12) | 85% | 1.04× (1.02–1.06) | 1.08× (1.04–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 1,120 | 3.27 | 1.76× (1.71–1.81) | 52% | 1.25× (1.20–1.29) | 1.71× (1.65–1.77) |

**Best:** mibz2 at 4 threads (1,120 MB/s); lbzip2 at 4 threads (224 MB/s); pbz2 at 4 threads (334 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.03, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.54 | 0.01 | 0.08 | 0.187 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3692 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.84 | 9.8% |
| MTF + RLE2 | 1.58 | 18.4% |
| T-vector build | 1.50 | 17.5% |
| walk (the one each build uses) | 4.21 | 49.2% |
| of which: segment assembly | 0.17 | 2.0% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.56 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.063 | (in its total) | 2.70× (2.68–2.73) |  |
| walk | 1.064 | 2.068 | (in its total) | 1.95× (1.92–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.042 | (in its total) | 0.96× (0.90–1.04) |  |
| inverse BWT, total | 1.502 | 3.173 | 1.901 | 2.12× (2.09–2.14) | 1.26× (1.25–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m27s ± 6m44s |
| 4 threads | 27m15s ± 3m51s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m28s ± 2m47s |
| the scanner alone (its serial part) | 8m26s ± 26.3s |
| the workers alone, over 4 | 19m28s ± 2m47s |

At 4 threads the workers bound it. Per byte: the scanner 4.9 ns a byte of bz2, the workers 2.55 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,073,579 | 14,017,486 | \`f53aa2d497223deb…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3691.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
