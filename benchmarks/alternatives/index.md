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

The latest record: [`10351bd`](<https://github.com/lmmx/mibz2/commit/10351bd9a094259f016ee9f17fcc945a37d99b73>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45561681468>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.510 s | 411 | 6.0 | 3.44× (3.25–3.64) | 9.14× (7.89–10.59) | 8.86× (8.13–9.65) | 8.92× (8.30–9.59) | 8.44× (7.59–9.38) | 8.97× (7.93–10.14) | 8.36× (7.30–9.57) | 2.45× (2.33–2.58) | 2.86× (2.73–3.00) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.306 s | 686 | 6.5 | 3.27× (3.05–3.51) | 8.24× (7.07–9.61) | 8.69× (7.79–9.70) | 8.25× (7.60–8.95) | 8.49× (7.03–10.25) | 15.19× (13.54–17.04) | 14.16× (12.51–16.04) | 4.15× (3.83–4.51) | 4.85× (4.48–5.25) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.252 s | 833 | 10.4 | 3.57× (3.21–3.96) | 6.80× (5.55–8.34) | 7.83× (6.75–9.09) | 6.83× (5.87–7.96) | 7.54× (6.15–9.24) | 18.45× (16.47–20.67) | 17.20× (15.08–19.62) | 5.04× (4.79–5.32) | 5.89× (5.63–6.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 905 | 3.0 | 0.59× (0.58–0.60) | 3.71× (3.64–3.79) | 4.52× (4.41–4.63) | 3.39× (3.32–3.46) | 6.01× (5.89–6.14) | 2.82× (2.77–2.88) | 2.51× (2.46–2.57) | 1.05× (1.00–1.11) | 1.40× (1.37–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.024 s | 848 | 3.3 | 0.53× (0.51–0.56) | 2.58× (2.46–2.71) | 5.50× (5.31–5.69) | 3.13× (3.00–3.27) | 5.48× (5.25–5.72) | 2.60× (2.49–2.71) | 2.31× (2.21–2.42) | 0.97× (0.93–1.01) | 1.29× (1.23–1.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.025 s | 815 | 3.4 | 0.53× (0.51–0.55) | 2.55× (2.47–2.62) | 5.42× (5.30–5.54) | 3.06× (2.97–3.15) | 5.44× (5.22–5.66) | 2.54× (2.46–2.61) | 2.26× (2.19–2.33) | 0.95× (0.89–1.01) | 1.26× (1.22–1.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 521 | 5.0 | 2.51× (2.39–2.63) | 2.70× (2.51–2.90) | 3.59× (3.39–3.79) | 3.50× (3.28–3.73) | 3.91× (3.64–4.19) | 2.30× (2.14–2.48) | 1.99× (1.85–2.14) | 2.15× (2.02–2.28) | 2.86× (2.74–2.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 798 | 6.0 | 2.28× (2.05–2.54) | 2.62× (2.34–2.93) | 4.11× (3.72–4.54) | 3.26× (2.76–3.85) | 4.64× (4.22–5.09) | 3.47× (3.14–3.82) | 2.99× (2.70–3.32) | 3.24× (2.96–3.54) | 4.31× (3.99–4.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 750 | 12.5 | 2.01× (1.91–2.11) | 2.03× (1.90–2.17) | 3.19× (2.99–3.41) | 2.19× (2.05–2.34) | 4.10× (3.86–4.35) | 3.35× (3.15–3.57) | 2.90× (2.70–3.11) | 3.13× (2.81–3.49) | 4.17× (3.93–4.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 41.2 | 1.41× (1.39–1.43) | 3.41× (2.83–4.12) | 3.17× (2.58–3.90) | 2.87× (2.56–3.23) | 3.34× (2.93–3.82) | 3.73× (3.20–4.36) | 3.36× (3.25–3.46) | 1.31× (1.28–1.34) | 1.46× (1.45–1.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 46.1 | 1.36× (1.31–1.42) | 3.52× (2.72–4.57) | 3.38× (2.67–4.27) | 3.73× (3.23–4.30) | 3.39× (3.07–3.74) | 6.08× (5.19–7.12) | 5.47× (5.23–5.71) | 2.13× (2.04–2.22) | 2.38× (2.28–2.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.065 s | 124 | 61.8 | 1.47× (1.40–1.55) | 3.14× (2.57–3.83) | 3.42× (2.69–4.36) | 3.85× (3.20–4.62) | 3.94× (3.37–4.60) | 7.56× (6.43–8.90) | 6.80× (6.57–7.04) | 2.65× (2.57–2.73) | 2.96× (2.92–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 37.9 | 1.53× (1.51–1.55) | 3.76× (3.02–4.69) | 3.32× (2.76–4.00) | 3.23× (2.75–3.80) | 3.55× (3.17–3.97) | 4.72× (3.85–5.78) | 4.74× (3.96–5.68) | 1.55× (1.50–1.60) | 1.82× (1.80–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 43.1 | 1.42× (1.33–1.51) | 3.82× (3.03–4.83) | 3.35× (2.82–3.99) | 4.02× (3.60–4.50) | 4.11× (3.27–5.16) | 7.69× (6.27–9.42) | 7.73× (6.48–9.21) | 2.52× (2.42–2.62) | 2.97× (2.88–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 129 | 62.4 | 1.55× (1.47–1.62) | 3.56× (2.89–4.39) | 3.80× (3.03–4.77) | 4.38× (3.59–5.33) | 4.57× (3.66–5.71) | 9.25× (7.63–11.22) | 9.30× (7.77–11.14) | 3.03× (2.92–3.15) | 3.57× (3.49–3.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 664 | 4.2 | 0.64× (0.62–0.66) | 3.44× (3.35–3.54) | 4.22× (4.04–4.40) | 2.94× (2.86–3.02) | 6.69× (6.49–6.90) | 2.49× (2.41–2.58) | 2.27× (2.21–2.34) | 1.01× (0.98–1.04) | 1.39× (1.36–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.017 s | 601 | 4.8 | 0.53× (0.51–0.54) | 2.46× (2.36–2.56) | 4.97× (4.66–5.31) | 2.64× (2.53–2.75) | 5.95× (5.76–6.14) | 2.21× (2.11–2.32) | 2.02× (1.95–2.09) | 0.90× (0.88–0.91) | 1.23× (1.19–1.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.017 s | 583 | 4.9 | 0.51× (0.49–0.53) | 2.54× (2.37–2.71) | 4.83× (4.64–5.04) | 2.59× (2.54–2.64) | 5.88× (5.68–6.09) | 2.17× (2.12–2.23) | 1.98× (1.91–2.06) | 0.88× (0.84–0.93) | 1.21× (1.19–1.24) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 0.068 s | 177 | 14.3 | 2.09× (2.02–2.16) | 4.88× (4.46–5.33) | 5.29× (4.74–5.92) | 4.88× (4.64–5.13) | 5.84× (5.05–6.75) | 6.14× (5.54–6.80) | 5.83× (5.01–6.78) | 1.75× (1.63–1.88) | 1.97× (1.89–2.04) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 0.048 s | 249 | 18.8 | 1.63× (1.55–1.72) | 4.21× (3.62–4.90) | 5.51× (4.60–6.58) | 5.10× (4.49–5.79) | 5.79× (4.62–7.26) | 8.43× (7.58–9.38) | 8.01× (6.86–9.35) | 2.41× (2.19–2.65) | 2.70× (2.56–2.85) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 0.043 s | 277 | 29.1 | 1.69× (1.53–1.88) | 3.45× (2.85–4.17) | 4.39× (3.75–5.14) | 3.92× (3.53–4.36) | 4.49× (3.97–5.08) | 9.57× (8.61–10.63) | 9.09× (7.80–10.58) | 2.73× (2.52–2.96) | 3.07× (2.93–3.21) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.43× (3.24–3.62) | 9.09× (7.85–10.53) | 8.81× (8.09–9.60) | 8.89× (8.27–9.55) | 8.41× (7.56–9.34) | 8.93× (7.90–10.10) | 8.33× (7.27–9.53) | 2.45× (2.32–2.58) | 2.84× (2.71–2.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.21× (3.06–3.38) | 8.58× (7.34–10.03) | 8.52× (7.52–9.66) | 8.80× (8.08–9.58) | 8.71× (7.09–10.68) | 8.18× (7.27–9.19) | 7.62× (6.71–8.66) | 2.24× (2.12–2.36) | 2.60× (2.45–2.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.46× (3.14–3.82) | 7.37× (5.92–9.16) | 7.48× (6.26–8.93) | 7.38× (6.28–8.69) | 7.41× (5.84–9.40) | 5.13× (4.58–5.76) | 4.79× (4.23–5.41) | 1.41× (1.33–1.49) | 1.63× (1.54–1.73) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.62–0.63) | 3.09× (3.05–3.14) | 3.87× (3.81–3.93) | 3.16× (3.11–3.22) | 5.53× (5.42–5.65) | 2.64× (2.60–2.69) | 2.37× (2.33–2.41) | 1.04× (0.99–1.10) | 1.03× (1.00–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.68× (2.57–2.80) | 4.85× (4.68–5.03) | 2.87× (2.76–2.99) | 4.93× (4.74–5.13) | 2.37× (2.28–2.47) | 2.12× (2.03–2.22) | 0.94× (0.90–0.98) | 0.93× (0.88–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.58) | 2.98× (2.91–3.06) | 4.82× (4.71–4.94) | 2.85× (2.77–2.93) | 4.94× (4.76–5.14) | 2.36× (2.29–2.42) | 2.11× (2.05–2.17) | 0.93× (0.88–0.99) | 0.92× (0.88–0.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.42× (2.31–2.53) | 2.56× (2.39–2.74) | 3.31× (3.10–3.52) | 3.35× (3.15–3.57) | 3.74× (3.50–4.00) | 2.23× (2.08–2.39) | 1.93× (1.80–2.07) | 2.08× (1.96–2.21) | 2.63× (2.51–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.10–2.47) | 2.38× (2.17–2.61) | 3.11× (2.81–3.44) | 2.95× (2.70–3.23) | 3.27× (2.99–3.59) | 1.84× (1.67–2.02) | 1.59× (1.44–1.76) | 1.72× (1.55–1.91) | 2.17× (1.99–2.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.88–2.04) | 1.55× (1.47–1.63) | 2.08× (1.99–2.18) | 1.82× (1.70–1.96) | 1.93× (1.83–2.04) | 0.91× (0.86–0.97) | 0.79× (0.74–0.85) | 0.85× (0.77–0.95) | 1.07× (1.02–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.38–1.42) | 3.34× (2.76–4.04) | 3.10× (2.53–3.81) | 2.84× (2.53–3.19) | 3.30× (2.89–3.77) | 3.69× (3.17–4.30) | 3.32× (3.21–3.42) | 1.30× (1.27–1.33) | 1.42× (1.41–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.34× (1.30–1.38) | 3.40× (2.62–4.40) | 3.06× (2.38–3.94) | 3.42× (3.00–3.89) | 2.99× (2.69–3.32) | 3.24× (2.78–3.79) | 2.92× (2.82–3.02) | 1.15× (1.11–1.18) | 1.25× (1.22–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.45–1.59) | 3.37× (2.73–4.15) | 3.15× (2.36–4.19) | 3.67× (2.97–4.53) | 3.52× (2.92–4.25) | 2.43× (2.07–2.86) | 2.19× (2.09–2.29) | 0.86× (0.83–0.89) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.54) | 3.69× (2.96–4.60) | 3.25× (2.70–3.92) | 3.20× (2.72–3.75) | 3.51× (3.14–3.93) | 4.66× (3.81–5.70) | 4.69× (3.91–5.61) | 1.54× (1.49–1.59) | 1.77× (1.76–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.39–1.45) | 3.86× (3.05–4.89) | 3.08× (2.55–3.72) | 3.69× (3.39–4.01) | 3.80× (2.97–4.86) | 4.07× (3.31–4.99) | 4.09× (3.45–4.85) | 1.34× (1.30–1.39) | 1.54× (1.51–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.60–1.74) | 3.81× (3.10–4.68) | 3.32× (2.55–4.32) | 4.02× (3.28–4.91) | 4.16× (3.22–5.38) | 2.82× (2.32–3.44) | 2.84× (2.38–3.39) | 0.93× (0.90–0.96) | 1.07× (1.06–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.70) | 2.69× (2.61–2.77) | 3.56× (3.47–3.65) | 2.69× (2.62–2.75) | 5.95× (5.78–6.12) | 2.30× (2.23–2.37) | 2.11× (2.06–2.15) | 1.01× (0.98–1.05) | 0.95× (0.91–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.59) | 2.27× (2.17–2.38) | 4.06× (3.86–4.27) | 2.39× (2.30–2.48) | 5.12× (4.97–5.27) | 1.96× (1.88–2.04) | 1.80× (1.74–1.86) | 0.86× (0.85–0.88) | 0.81× (0.78–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.53–0.57) | 2.57× (2.44–2.71) | 4.06× (3.92–4.20) | 2.42× (2.37–2.47) | 5.19× (5.01–5.38) | 1.98× (1.93–2.03) | 1.82× (1.76–1.88) | 0.87× (0.83–0.91) | 0.82× (0.78–0.86) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 2.05× (1.99–2.12) | 4.69× (4.29–5.12) | 5.09× (4.55–5.69) | 4.75× (4.52–5.00) | 5.68× (4.92–6.57) | 5.97× (5.39–6.62) | 5.67× (4.88–6.60) | 1.73× (1.61–1.86) | 1.87× (1.82–1.91) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 1.69× (1.64–1.75) | 4.25× (3.67–4.91) | 4.94× (4.08–5.98) | 4.69× (4.06–5.40) | 5.25× (4.16–6.62) | 4.53× (4.11–5.01) | 4.31× (3.73–4.98) | 1.31× (1.22–1.41) | 1.42× (1.37–1.46) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 1.87× (1.70–2.07) | 3.62× (3.01–4.34) | 3.72× (3.11–4.45) | 3.52× (3.16–3.93) | 3.76× (3.21–4.40) | 2.90× (2.63–3.19) | 2.75× (2.38–3.18) | 0.84× (0.79–0.90) | 0.91× (0.88–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.25–0.27) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.97× (10.96–10.99) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.37) | 0.52× (0.43–0.62) | 0.74× (0.71–0.78) | 0.61× (0.58–0.64) | 5.81× (5.56–6.08) | 0.14× (0.13–0.15) | 0.15× (0.14–0.16) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.35–0.38) | 0.60× (0.49–0.73) | 0.66× (0.63–0.69) | 0.56× (0.53–0.59) | 3.75× (3.58–3.94) | 0.09× (0.08–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.83–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.01–1.02) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.92–7.96) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.74) | 0.72× (0.69–0.75) | 1.72× (1.68–1.75) | 0.86× (0.84–0.88) | 6.21× (6.08–6.35) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.57× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.74× (0.72–0.75) | 0.81× (0.72–0.91) | 1.29× (1.27–1.31) | 0.78× (0.77–0.80) | 3.80× (3.77–3.83) | 0.30× (0.30–0.30) | 0.31× (0.31–0.32) | 0.34× (0.34–0.34) | 0.17× (0.17–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.72–0.73) | 5.14× (5.12–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.36× (0.33–0.40) | 0.90× (0.87–0.93) | 0.68× (0.64–0.72) | 3.07× (2.97–3.16) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.87–0.89) | 0.82× (0.79–0.84) | 2.55× (2.52–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.16× (5.16–5.17) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.33× (0.31–0.35) | 0.85× (0.82–0.88) | 0.68× (0.66–0.70) | 2.86× (2.75–2.97) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.52) | 0.82× (0.81–0.84) | 0.81× (0.79–0.83) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.30× (5.28–5.32) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.19× (6.18–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.91) | 6.20× (6.18–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.40–1.41) | 0.65× (0.65–0.66) | 6.34× (6.33–6.36) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.64) | 0.31× (0.31–0.31) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 36 MB | 0.46× (0.44–0.49) | 0.40× (0.36–0.45) | 1.08× (1.04–1.12) | 0.68× (0.65–0.71) | 3.37× (3.22–3.53) | 0.21× (0.20–0.22) | 0.22× (0.21–0.24) | 0.34× (0.32–0.35) | 0.17× (0.16–0.17) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.47× (0.44–0.50) | 0.90× (0.88–0.92) | 0.71× (0.68–0.74) | 2.29× (2.23–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 411 | 2.44 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 686 | 2.65 | 1.09× (1.07–1.12) | 85% | 1.03× (0.86–1.24) | 1.02× (0.94–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 833 | 4.26 | 1.74× (1.70–1.78) | 51% | 1.41× (1.10–1.80) | 1.76× (1.55–1.98) |

**Best:** mibz2 at 4 threads (833 MB/s); lbzip2 at 4 threads (131 MB/s); pbz2 at 4 threads (236 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.02, lbzip2 1t 0.07, lbzip2 2t 0.34, lbzip2 4t 0.47, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.71 | 0.03 | 0.08 | 0.252 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m15s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.72 | 9.8% |
| MTF + RLE2 | 1.20 | 16.2% |
| T-vector build | 1.23 | 16.6% |
| walk (the one each build uses) | 3.77 | 51.2% |
| of which: segment assembly | 0.22 | 3.0% |
| RLE1 | 0.24 | 3.3% |
| CRC | 0.21 | 2.8% |
| **total** | 7.37 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.307 | (in its total) | 2.67× (2.66–2.68) |  |
| walk | 1.319 | 2.744 | (in its total) | 2.09× (2.07–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.066 | 0.084 | (in its total) | 1.26× (1.20–1.33) |  |
| inverse BWT, total | 1.875 | 4.139 | 3.412 | 2.21× (2.19–2.23) | 1.82× (1.81–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 9m10s |
| 4 threads | 36m21s ± 5m18s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m00s ± 3m50s |
| the scanner alone (its serial part) | 10m55s ± 32.9s |
| the workers alone, over 4 | 26m00s ± 3m50s |

At 4 threads the workers bound it. Per byte: the scanner 6.3 ns a byte of bz2, the workers 3.41 ns a byte of output at 4 threads (CPU, summed over them), 2.09 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,369,642 | 12,022,894 | \`502528b59d9a8643…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
