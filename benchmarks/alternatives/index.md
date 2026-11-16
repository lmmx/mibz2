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

The latest record: [`ae740d3`](<https://github.com/lmmx/mibz2/commit/ae740d3d1ed9a7879d94f776125b484c21f0113a>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43795712836>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.415 s | 505 | 7.3 | 2.87× (2.77–2.98) | 6.62× (6.39–6.86) | 6.87× (6.61–7.14) | 7.77× (7.49–8.05) | 6.80× (6.57–7.03) | 7.18× (6.97–7.39) | 6.85× (6.65–7.06) | 2.13× (2.07–2.19) | 2.38× (2.30–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.244 s | 858 | 8.0 | 2.83× (2.68–3.00) | 5.85× (5.48–6.25) | 6.74× (6.30–7.21) | 6.98× (6.53–7.47) | 6.14× (5.77–6.54) | 12.12× (11.39–12.89) | 11.57× (10.88–12.31) | 3.59× (3.38–3.82) | 4.01× (3.76–4.28) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,068 | 12.3 | 2.96× (2.92–3.00) | 4.38× (4.31–4.45) | 5.43× (5.33–5.54) | 5.33× (5.21–5.45) | 5.18× (5.15–5.22) | 15.45× (15.30–15.59) | 14.75× (14.62–14.88) | 4.58× (4.54–4.62) | 5.12× (5.08–5.16) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,127 | 3.6 | 0.58× (0.57–0.58) | 3.43× (3.38–3.49) | 4.58× (4.50–4.66) | 3.58× (3.53–3.63) | 6.10× (6.00–6.20) | 2.66× (2.62–2.70) | 2.67× (2.62–2.72) | 0.97× (0.96–0.99) | 1.77× (1.65–1.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,127 | 3.7 | 0.54× (0.53–0.55) | 2.88× (2.83–2.92) | 5.76× (5.66–5.85) | 3.59× (3.54–3.65) | 6.07× (5.95–6.20) | 2.67× (2.62–2.71) | 2.68× (2.64–2.71) | 0.97× (0.94–1.01) | 1.77× (1.67–1.88) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,114 | 3.6 | 0.53× (0.53–0.54) | 2.84× (2.80–2.88) | 5.67× (5.59–5.76) | 3.55× (3.51–3.60) | 6.04× (5.90–6.19) | 2.62× (2.58–2.66) | 2.63× (2.60–2.67) | 0.96× (0.93–0.99) | 1.74× (1.64–1.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 741 | 5.3 | 2.72× (2.69–2.74) | 3.06× (3.03–3.10) | 3.90× (3.73–4.08) | 4.09× (4.03–4.15) | 4.49× (4.42–4.56) | 2.77× (2.74–2.80) | 2.26× (2.23–2.29) | 2.08× (2.05–2.11) | 3.05× (3.01–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,071 | 6.5 | 2.20× (1.98–2.44) | 2.89× (2.54–3.29) | 4.19× (3.73–4.70) | 3.60× (3.11–4.18) | 4.85× (4.46–5.28) | 3.94× (3.62–4.28) | 3.21× (2.96–3.49) | 2.96× (2.72–3.21) | 4.34× (3.99–4.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 973 | 12.8 | 1.89× (1.81–1.98) | 1.89× (1.75–2.05) | 2.98× (2.82–3.15) | 2.22× (2.09–2.36) | 4.12× (3.97–4.26) | 3.66× (3.56–3.76) | 2.99× (2.91–3.06) | 2.75× (2.68–2.82) | 4.04× (3.93–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 s | 73 | 51.3 | 1.38× (1.36–1.40) | 2.62× (2.53–2.71) | 2.43× (2.34–2.53) | 2.49× (2.42–2.56) | 2.78× (2.71–2.86) | 2.98× (2.90–3.06) | 3.10× (3.02–3.19) | 1.25× (1.21–1.29) | 1.38× (1.34–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 117 | 57.9 | 1.24× (1.16–1.32) | 2.45× (2.22–2.70) | 2.30× (2.17–2.44) | 2.88× (2.71–3.07) | 2.77× (2.60–2.94) | 4.68× (4.41–4.97) | 4.88× (4.59–5.19) | 1.96× (1.84–2.09) | 2.18× (2.04–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 77.9 | 1.32× (1.26–1.37) | 2.06× (1.98–2.15) | 2.08× (1.99–2.17) | 2.63× (2.54–2.71) | 2.89× (2.78–3.00) | 5.84× (5.64–6.04) | 6.08× (5.87–6.30) | 2.45× (2.38–2.52) | 2.71× (2.61–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.3 | 1.52× (1.52–1.53) | 3.07× (3.03–3.10) | 2.68× (2.65–2.71) | 2.77× (2.76–2.78) | 3.10× (3.08–3.12) | 3.70× (3.68–3.71) | 4.03× (4.01–4.05) | 1.51× (1.49–1.52) | 1.80× (1.79–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 52.9 | 1.35× (1.29–1.41) | 2.84× (2.59–3.12) | 2.49× (2.40–2.59) | 3.38× (3.26–3.51) | 3.03× (2.92–3.15) | 5.92× (5.71–6.13) | 6.45× (6.22–6.70) | 2.41× (2.32–2.51) | 2.89× (2.78–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 75.1 | 1.42× (1.39–1.45) | 2.26× (2.21–2.31) | 2.29× (2.18–2.41) | 2.96× (2.89–3.03) | 2.84× (2.79–2.89) | 6.91× (6.81–7.02) | 7.54× (7.42–7.66) | 2.82× (2.78–2.86) | 3.37× (3.33–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 848 | 5.0 | 0.63× (0.62–0.63) | 3.31× (3.07–3.58) | 4.38× (4.36–4.41) | 3.12× (3.11–3.14) | 6.74× (6.64–6.84) | 2.35× (2.24–2.48) | 2.34× (2.32–2.36) | 0.97× (0.96–0.98) | 1.75× (1.74–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 820 | 5.3 | 0.53× (0.52–0.54) | 2.52× (2.48–2.56) | 5.06× (4.98–5.14) | 3.02× (2.96–3.07) | 6.42× (6.29–6.55) | 2.26× (2.14–2.38) | 2.24× (2.20–2.28) | 0.93× (0.92–0.94) | 1.68× (1.65–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.52× (0.51–0.54) | 2.47× (2.44–2.50) | 4.96× (4.90–5.02) | 2.97× (2.93–3.01) | 6.40× (6.19–6.61) | 2.21× (2.08–2.34) | 2.20× (2.17–2.23) | 0.91× (0.89–0.93) | 1.65× (1.63–1.66) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.058 s | 231 | 16.5 | 1.83× (1.82–1.83) | 4.02× (3.98–4.06) | 4.38× (4.36–4.39) | 4.60× (4.58–4.61) | 4.78× (4.74–4.82) | 4.75× (4.73–4.76) | 4.77× (4.75–4.79) | 1.51× (1.51–1.52) | 1.75× (1.75–1.76) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.041 s | 328 | 20.9 | 1.44× (1.31–1.58) | 3.11× (2.84–3.41) | 3.78× (3.45–4.14) | 4.02× (3.68–4.39) | 4.28× (3.81–4.82) | 6.54× (6.01–7.10) | 6.57× (6.04–7.15) | 2.09× (1.92–2.27) | 2.41× (2.22–2.62) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.037 s | 362 | 31.7 | 1.45× (1.39–1.51) | 2.46× (2.37–2.56) | 3.09× (3.02–3.17) | 3.17× (3.09–3.25) | 3.77× (3.67–3.86) | 7.37× (7.19–7.55) | 7.41× (7.23–7.60) | 2.35× (2.29–2.42) | 2.72× (2.66–2.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.76–2.97) | 6.58× (6.35–6.83) | 6.84× (6.58–7.10) | 7.74× (7.47–8.02) | 6.77× (6.55–7.01) | 7.15× (6.94–7.36) | 6.83× (6.63–7.04) | 2.12× (2.06–2.19) | 2.36× (2.29–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.76× (2.74–2.77) | 6.25× (6.21–6.29) | 6.60× (6.53–6.66) | 7.57× (7.53–7.61) | 6.36× (6.33–6.40) | 6.67× (6.62–6.72) | 6.37× (6.32–6.42) | 1.98× (1.96–2.00) | 2.20× (2.19–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.86–2.95) | 4.77× (4.72–4.83) | 5.16× (5.12–5.20) | 5.78× (5.71–5.85) | 4.98× (4.92–5.05) | 4.33× (4.29–4.38) | 4.14× (4.10–4.18) | 1.29× (1.27–1.30) | 1.43× (1.42–1.44) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 2.92× (2.87–2.96) | 3.96× (3.86–4.07) | 3.36× (3.31–3.40) | 5.66× (5.57–5.75) | 2.52× (2.48–2.56) | 2.53× (2.49–2.57) | 0.98× (0.96–0.99) | 1.64× (1.62–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.76× (2.73–2.80) | 4.99× (4.91–5.07) | 3.33× (3.29–3.38) | 5.54× (5.44–5.65) | 2.47× (2.44–2.51) | 2.48× (2.45–2.51) | 0.96× (0.93–0.99) | 1.61× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.58) | 3.04× (3.00–3.08) | 5.06× (5.01–5.11) | 3.38× (3.34–3.41) | 5.63× (5.52–5.75) | 2.50× (2.47–2.53) | 2.51× (2.48–2.54) | 0.97× (0.94–1.00) | 1.63× (1.61–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.59–2.64) | 2.82× (2.79–2.85) | 3.43× (3.26–3.60) | 3.90× (3.85–3.95) | 4.28× (4.22–4.35) | 2.66× (2.63–2.69) | 2.19× (2.16–2.21) | 2.01× (1.99–2.04) | 2.64× (2.62–2.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.26–2.36) | 2.52× (2.42–2.63) | 3.38× (3.28–3.49) | 3.36× (3.28–3.44) | 3.63× (3.56–3.71) | 2.18× (2.14–2.22) | 1.79× (1.75–1.83) | 1.65× (1.61–1.69) | 2.17× (2.12–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.89–2.08) | 1.71× (1.67–1.75) | 2.26× (2.20–2.32) | 2.04× (2.00–2.09) | 2.13× (2.07–2.18) | 1.10× (1.08–1.12) | 0.90× (0.89–0.92) | 0.83× (0.82–0.85) | 1.09× (1.07–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.35–1.39) | 2.56× (2.49–2.63) | 2.38× (2.29–2.47) | 2.46× (2.40–2.53) | 2.76× (2.68–2.83) | 2.95× (2.87–3.03) | 3.07× (2.99–3.16) | 1.25× (1.21–1.28) | 1.36× (1.33–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.25–1.30) | 2.42× (2.38–2.46) | 2.15× (2.11–2.18) | 2.83× (2.79–2.88) | 2.51× (2.47–2.55) | 2.63× (2.59–2.67) | 2.74× (2.70–2.78) | 1.11× (1.09–1.13) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.36–1.46) | 2.23× (2.18–2.29) | 1.88× (1.83–1.92) | 2.49× (2.40–2.58) | 2.45× (2.39–2.51) | 1.95× (1.90–1.99) | 2.03× (1.98–2.08) | 0.82× (0.81–0.84) | 0.90× (0.88–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.52) | 2.99× (2.97–3.02) | 2.61× (2.58–2.64) | 2.74× (2.73–2.76) | 3.07× (3.05–3.09) | 3.66× (3.64–3.68) | 3.99× (3.97–4.01) | 1.50× (1.49–1.51) | 1.73× (1.72–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.39) | 2.74× (2.70–2.79) | 2.34× (2.30–2.37) | 3.19× (3.13–3.25) | 2.74× (2.70–2.78) | 3.18× (3.14–3.23) | 3.47× (3.42–3.53) | 1.31× (1.28–1.33) | 1.51× (1.48–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.50–1.64) | 2.49× (2.42–2.56) | 2.04× (1.95–2.15) | 2.76× (2.66–2.86) | 2.58× (2.50–2.65) | 2.27× (2.21–2.33) | 2.47× (2.40–2.54) | 0.93× (0.90–0.96) | 1.07× (1.04–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.46× (2.43–2.49) | 3.43× (3.38–3.47) | 2.85× (2.83–2.87) | 6.01× (5.92–6.10) | 2.18× (2.08–2.29) | 2.17× (2.15–2.18) | 0.97× (0.96–0.98) | 1.36× (1.35–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.59–0.60) | 2.27× (2.24–2.31) | 4.02× (3.96–4.09) | 2.76× (2.73–2.80) | 5.64× (5.55–5.74) | 2.05× (1.95–2.16) | 2.04× (2.01–2.07) | 0.91× (0.91–0.92) | 1.28× (1.27–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.49× (2.47–2.52) | 4.04× (3.97–4.11) | 2.80× (2.76–2.84) | 5.82× (5.64–6.00) | 2.08× (1.96–2.20) | 2.06× (2.04–2.09) | 0.93× (0.91–0.94) | 1.30× (1.28–1.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.80× (1.79–1.81) | 3.80× (3.79–3.81) | 4.18× (4.16–4.19) | 4.49× (4.48–4.51) | 4.67× (4.63–4.71) | 4.64× (4.62–4.65) | 4.66× (4.64–4.68) | 1.50× (1.49–1.51) | 1.63× (1.62–1.64) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.51× (1.49–1.52) | 3.14× (3.10–3.18) | 3.51× (3.45–3.56) | 4.04× (3.99–4.09) | 3.78× (3.72–3.84) | 3.68× (3.63–3.72) | 3.70× (3.65–3.75) | 1.19× (1.17–1.20) | 1.29× (1.28–1.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.64× (1.55–1.72) | 2.56× (2.51–2.61) | 2.84× (2.80–2.88) | 3.22× (3.16–3.28) | 3.07× (3.01–3.14) | 2.42× (2.38–2.47) | 2.44× (2.39–2.48) | 0.78× (0.77–0.80) | 0.85× (0.84–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.06× (11.03–11.08) | 0.26× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.33× (0.32–0.35) | 0.45× (0.40–0.51) | 0.68× (0.66–0.71) | 0.58× (0.56–0.60) | 5.46× (5.28–5.65) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.32× (0.31–0.33) | 0.50× (0.40–0.64) | 0.57× (0.55–0.59) | 0.51× (0.49–0.53) | 3.36× (3.25–3.46) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.03× (8.01–8.05) | 0.65× (0.64–0.66) | 0.69× (0.69–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.67× (0.64–0.69) | 0.62× (0.54–0.71) | 1.57× (1.51–1.64) | 0.78× (0.76–0.81) | 5.71× (5.48–5.95) | 0.46× (0.44–0.48) | 0.49× (0.47–0.51) | 0.53× (0.50–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.71× (0.66–0.75) | 1.16× (1.13–1.20) | 0.71× (0.68–0.73) | 3.39× (3.29–3.50) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.44) | 0.90× (0.87–0.94) | 0.73× (0.70–0.76) | 3.13× (3.00–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.80× (0.76–0.85) | 2.51× (2.44–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.21–5.25) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.35× (0.32–0.38) | 0.84× (0.82–0.87) | 0.69× (0.66–0.73) | 2.90× (2.81–2.99) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.50× (0.48–0.53) | 0.80× (0.79–0.81) | 0.81× (0.78–0.83) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.31× (5.29–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.33× (6.32–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 39 MB | 0.43× (0.42–0.45) | 0.36× (0.33–0.38) | 1.00× (0.96–1.04) | 0.66× (0.64–0.68) | 3.18× (3.09–3.29) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 58 MB | 0.47× (0.46–0.49) | 0.44× (0.41–0.48) | 0.87× (0.84–0.91) | 0.73× (0.71–0.76) | 2.23× (2.16–2.31) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 505 | 1.99 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 858 | 2.17 | 1.07× (1.04–1.11) | 84% | 1.02× (1.01–1.02) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,068 | 3.34 | 1.65× (1.59–1.71) | 54% | 1.20× (1.18–1.21) | 1.67× (1.65–1.70) |

**Best:** mibz2 at 4 threads (1,068 MB/s); lbzip2 at 4 threads (245 MB/s); pbz2 at 4 threads (360 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.21, lbzip2 1t 0.07, lbzip2 2t 0.25, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.53 | 0.01 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.8% |
| MTF + RLE2 | 1.58 | 18.1% |
| T-vector build | 1.48 | 16.9% |
| walk (the one each build uses) | 4.09 | 46.9% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.72 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.396 | 1.066 | (in its total) | 2.69× (2.68–2.70) |  |
| walk | 1.039 | 2.038 | (in its total) | 1.96× (1.96–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.040 | (in its total) | 1.08× (1.07–1.09) |  |
| inverse BWT, total | 1.471 | 3.145 | 1.866 | 2.14× (2.13–2.14) | 1.27× (1.26–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m25s |
| 4 threads | 28m34s ± 4m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m25s ± 2m44s |
| the scanner alone (its serial part) | 11m03s ± 32.2s |
| the workers alone, over 4 | 19m25s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 6.4 ns a byte of bz2, the workers 2.55 ns a byte of output at 4 threads (CPU, summed over them), 1.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,189 | 13,350,290 | \`2669bf5db8b933d6…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
