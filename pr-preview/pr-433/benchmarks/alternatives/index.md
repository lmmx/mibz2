# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-433/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`df93e35`](<https://github.com/lmmx/mibz2/commit/df93e350dcb55f7b833ed7f10ae32c485797b56c>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43744597797>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.417 s | 503 | 7.5 | 2.94× (2.88–3.00) | 6.66× (6.55–6.76) | 6.93× (6.82–7.05) | 7.87× (7.80–7.95) | 6.96× (6.88–7.04) | 7.17× (7.09–7.25) | 6.91× (6.84–6.98) | 2.21× (2.15–2.27) | 2.42× (2.38–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.259 s | 810 | 8.5 | 2.90× (2.74–3.06) | 5.80× (5.55–6.06) | 6.75× (6.36–7.16) | 6.89× (6.55–7.24) | 6.07× (5.79–6.35) | 11.45× (10.93–11.99) | 11.03× (10.54–11.55) | 3.52× (3.38–3.68) | 3.86× (3.68–4.06) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.205 s | 1,021 | 13.2 | 3.07× (2.92–3.22) | 4.54× (4.35–4.75) | 5.63× (5.40–5.87) | 5.32× (5.08–5.57) | 5.33× (5.11–5.55) | 14.59× (14.19–15.01) | 14.06× (13.68–14.46) | 4.49× (4.30–4.69) | 4.92× (4.77–5.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,096 | 3.8 | 0.57× (0.56–0.57) | 3.37× (3.33–3.42) | 4.50× (4.44–4.57) | 3.53× (3.47–3.58) | 6.08× (5.99–6.18) | 2.62× (2.59–2.66) | 2.62× (2.58–2.66) | 1.00× (0.98–1.01) | 2.01× (1.81–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,105 | 3.8 | 0.54× (0.53–0.55) | 2.87× (2.81–2.93) | 5.74× (5.62–5.87) | 3.60× (3.53–3.67) | 6.10× (5.91–6.30) | 2.67× (2.62–2.72) | 2.66× (2.61–2.72) | 1.01× (0.99–1.04) | 2.04× (1.82–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,081 | 3.8 | 0.53× (0.52–0.54) | 2.79× (2.73–2.86) | 5.58× (5.44–5.71) | 3.49× (3.41–3.58) | 5.93× (5.77–6.10) | 2.59× (2.53–2.64) | 2.58× (2.53–2.64) | 0.98× (0.96–1.00) | 1.98× (1.80–2.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 737 | 5.4 | 2.69× (2.66–2.72) | 3.07× (3.04–3.10) | 3.84× (3.80–3.88) | 4.11× (4.07–4.15) | 4.52× (4.46–4.58) | 2.77× (2.74–2.80) | 2.27× (2.25–2.29) | 2.11× (2.09–2.13) | 3.06× (3.02–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,048 | 6.8 | 2.18× (2.05–2.32) | 2.83× (2.63–3.04) | 4.35× (4.05–4.67) | 3.52× (3.29–3.76) | 4.92× (4.62–5.25) | 3.98× (3.74–4.23) | 3.26× (3.06–3.47) | 3.03× (2.85–3.22) | 4.39× (4.12–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 938 | 14.1 | 1.84× (1.79–1.90) | 1.77× (1.64–1.91) | 3.06× (2.80–3.35) | 2.13× (2.03–2.22) | 3.89× (3.71–4.07) | 3.46× (3.34–3.59) | 2.84× (2.74–2.94) | 2.64× (2.54–2.73) | 3.82× (3.69–3.96) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 54.1 | 1.35× (1.29–1.41) | 2.57× (2.45–2.70) | 2.34× (2.24–2.45) | 2.39× (2.28–2.50) | 2.72× (2.62–2.82) | 2.88× (2.74–3.02) | 3.03× (2.91–3.15) | 1.23× (1.17–1.29) | 1.38× (1.32–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 113 | 62.5 | 1.27× (1.24–1.31) | 2.48× (2.36–2.61) | 2.36× (2.27–2.45) | 2.95× (2.80–3.11) | 2.82× (2.72–2.92) | 4.71× (4.55–4.88) | 4.96× (4.78–5.13) | 2.01× (1.91–2.11) | 2.26× (2.16–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 79.0 | 1.33× (1.29–1.37) | 2.23× (2.04–2.44) | 2.19× (2.06–2.33) | 2.66× (2.54–2.79) | 2.94× (2.82–3.06) | 5.89× (5.74–6.04) | 6.20× (6.04–6.36) | 2.51× (2.45–2.58) | 2.82× (2.73–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 47.8 | 1.51× (1.48–1.54) | 3.02× (2.94–3.10) | 2.64× (2.56–2.71) | 2.72× (2.66–2.78) | 3.12× (2.98–3.26) | 3.65× (3.58–3.73) | 4.02× (3.92–4.13) | 1.55× (1.51–1.58) | 1.80× (1.75–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 54.6 | 1.37× (1.34–1.41) | 2.76× (2.68–2.84) | 2.55× (2.49–2.62) | 3.38× (3.31–3.46) | 3.08× (3.01–3.14) | 6.02× (5.90–6.15) | 6.63× (6.49–6.77) | 2.55× (2.45–2.65) | 2.96× (2.90–3.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 74.8 | 1.39× (1.35–1.43) | 2.31× (2.23–2.38) | 2.35× (2.26–2.44) | 2.94× (2.84–3.06) | 2.85× (2.75–2.95) | 6.93× (6.76–7.11) | 7.63× (7.43–7.83) | 2.93× (2.82–3.05) | 3.41× (3.31–3.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 5.2 | 0.61× (0.60–0.63) | 3.34× (3.20–3.49) | 4.28× (4.20–4.37) | 3.07× (3.02–3.12) | 6.81× (6.38–7.27) | 2.26× (2.22–2.30) | 2.30× (2.26–2.35) | 0.96× (0.93–0.99) | 1.72× (1.69–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 784 | 5.6 | 0.51× (0.49–0.53) | 2.43× (2.36–2.51) | 4.87× (4.72–5.03) | 2.92× (2.83–3.02) | 6.27× (6.07–6.47) | 2.13× (2.07–2.20) | 2.18× (2.11–2.25) | 0.91× (0.88–0.94) | 1.62× (1.57–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 810 | 5.2 | 0.53× (0.52–0.54) | 2.51× (2.47–2.55) | 5.10× (4.91–5.30) | 3.02× (2.98–3.06) | 6.46× (6.24–6.68) | 2.19× (2.16–2.23) | 2.24× (2.21–2.27) | 0.93× (0.92–0.95) | 1.67× (1.64–1.69) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.057 s | 224 | 17.3 | 1.77× (1.69–1.84) | 3.84× (3.69–4.00) | 4.26× (4.08–4.45) | 4.42× (4.24–4.61) | 4.63× (4.43–4.84) | 4.54× (4.35–4.73) | 4.62× (4.42–4.83) | 1.47× (1.41–1.55) | 1.72× (1.65–1.80) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.041 s | 309 | 22.2 | 1.43× (1.30–1.57) | 3.07× (2.70–3.48) | 3.63× (3.29–4.01) | 4.16× (3.75–4.62) | 4.03× (3.71–4.38) | 6.28× (5.79–6.81) | 6.39× (5.85–6.98) | 2.04× (1.87–2.22) | 2.38× (2.21–2.56) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.035 s | 359 | 33.1 | 1.43× (1.37–1.49) | 2.40× (2.23–2.59) | 3.17× (3.00–3.35) | 3.53× (3.35–3.72) | 3.90× (3.65–4.18) | 7.41× (7.14–7.70) | 7.55× (7.26–7.85) | 2.41× (2.31–2.51) | 2.81× (2.66–2.98) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.87–2.99) | 6.62× (6.52–6.73) | 6.89× (6.78–7.01) | 7.85× (7.77–7.92) | 6.93× (6.86–7.01) | 7.15× (7.06–7.23) | 6.89× (6.82–6.95) | 2.20× (2.14–2.26) | 2.41× (2.37–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.82× (2.71–2.93) | 6.14× (5.97–6.31) | 6.56× (6.37–6.76) | 7.39× (7.21–7.57) | 6.27× (6.07–6.47) | 6.27× (6.08–6.47) | 6.04× (5.86–6.23) | 1.93× (1.86–2.01) | 2.11× (2.03–2.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.98× (2.87–3.10) | 4.90× (4.70–5.11) | 5.36× (5.05–5.68) | 5.80× (5.50–6.12) | 5.15× (4.92–5.39) | 4.08× (3.97–4.19) | 3.93× (3.82–4.05) | 1.26× (1.20–1.32) | 1.37× (1.33–1.42) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.92× (2.88–2.95) | 3.89× (3.81–3.97) | 3.31× (3.26–3.36) | 5.65× (5.56–5.74) | 2.48× (2.45–2.52) | 2.48× (2.45–2.52) | 1.00× (0.99–1.02) | 1.63× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.77× (2.71–2.82) | 5.03× (4.96–5.11) | 3.34× (3.28–3.41) | 5.58× (5.40–5.77) | 2.48× (2.43–2.52) | 2.48× (2.43–2.53) | 1.00× (0.98–1.02) | 1.63× (1.59–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 2.99× (2.93–3.06) | 5.05× (4.93–5.17) | 3.32× (3.25–3.39) | 5.52× (5.37–5.66) | 2.46× (2.41–2.51) | 2.46× (2.41–2.51) | 0.99× (0.98–1.01) | 1.62× (1.58–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.59× (2.56–2.61) | 2.85× (2.82–2.87) | 3.42× (3.38–3.45) | 3.92× (3.88–3.96) | 4.30× (4.25–4.36) | 2.66× (2.64–2.69) | 2.19× (2.17–2.22) | 2.04× (2.02–2.06) | 2.67× (2.64–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.23–2.32) | 2.50× (2.43–2.58) | 3.35× (3.27–3.44) | 3.34× (3.23–3.45) | 3.56× (3.47–3.65) | 2.13× (2.07–2.18) | 1.75× (1.71–1.80) | 1.63× (1.59–1.67) | 2.14× (2.08–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.91–2.06) | 1.62× (1.54–1.72) | 2.19× (2.06–2.32) | 1.94× (1.85–2.04) | 2.02× (1.92–2.13) | 1.04× (0.99–1.09) | 0.85× (0.82–0.89) | 0.79× (0.76–0.83) | 1.04× (1.00–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.29–1.41) | 2.49× (2.39–2.60) | 2.28× (2.18–2.38) | 2.37× (2.26–2.47) | 2.69× (2.60–2.79) | 2.85× (2.72–2.99) | 3.00× (2.88–3.12) | 1.22× (1.16–1.29) | 1.33× (1.27–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.22–1.27) | 2.36× (2.31–2.41) | 2.10× (2.05–2.15) | 2.72× (2.65–2.79) | 2.44× (2.37–2.50) | 2.53× (2.47–2.59) | 2.66× (2.60–2.72) | 1.09× (1.04–1.13) | 1.18× (1.15–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.44–1.53) | 2.35× (2.25–2.46) | 1.97× (1.90–2.03) | 2.58× (2.50–2.67) | 2.55× (2.47–2.63) | 1.98× (1.95–2.02) | 2.09× (2.05–2.13) | 0.85× (0.82–0.89) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.46–1.53) | 2.94× (2.87–3.02) | 2.58× (2.49–2.66) | 2.69× (2.62–2.76) | 3.08× (2.95–3.23) | 3.61× (3.52–3.70) | 3.97× (3.86–4.09) | 1.53× (1.50–1.57) | 1.74× (1.70–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.37–1.40) | 2.75× (2.73–2.78) | 2.36× (2.33–2.39) | 3.20× (3.16–3.23) | 2.76× (2.73–2.79) | 3.20× (3.17–3.24) | 3.53× (3.49–3.56) | 1.36× (1.32–1.41) | 1.55× (1.53–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.57–1.63) | 2.56× (2.47–2.66) | 2.09× (2.01–2.17) | 2.83× (2.74–2.92) | 2.67× (2.60–2.74) | 2.31× (2.27–2.35) | 2.54× (2.48–2.60) | 0.98× (0.94–1.02) | 1.12× (1.09–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.46× (2.43–2.49) | 3.43× (3.38–3.49) | 2.81× (2.77–2.85) | 6.09× (5.72–6.48) | 2.10× (2.07–2.13) | 2.14× (2.11–2.18) | 0.97× (0.95–0.99) | 1.36× (1.34–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.55–0.60) | 2.21× (2.14–2.28) | 4.00× (3.87–4.13) | 2.68× (2.59–2.77) | 5.50× (5.33–5.68) | 1.94× (1.87–2.00) | 1.98× (1.91–2.05) | 0.89× (0.86–0.92) | 1.25× (1.21–1.29) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.57× (2.54–2.60) | 4.17× (4.07–4.27) | 2.86× (2.82–2.90) | 5.86× (5.68–6.06) | 2.06× (2.03–2.09) | 2.10× (2.08–2.13) | 0.95× (0.93–0.96) | 1.33× (1.31–1.35) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.74× (1.67–1.82) | 3.68× (3.53–3.84) | 4.07× (3.89–4.25) | 4.32× (4.15–4.50) | 4.53× (4.33–4.73) | 4.43× (4.26–4.62) | 4.52× (4.33–4.71) | 1.46× (1.39–1.53) | 1.58× (1.52–1.65) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.54× (1.48–1.60) | 3.19× (3.04–3.35) | 3.52× (3.40–3.64) | 4.02× (3.90–4.13) | 3.74× (3.68–3.81) | 3.55× (3.47–3.63) | 3.62× (3.52–3.72) | 1.17× (1.15–1.19) | 1.27× (1.25–1.29) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.64× (1.58–1.70) | 2.67× (2.47–2.89) | 3.00× (2.81–3.20) | 3.31× (3.15–3.48) | 3.25× (3.02–3.49) | 2.36× (2.32–2.40) | 2.40× (2.35–2.46) | 0.78× (0.76–0.79) | 0.84× (0.83–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.34× (0.32–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.07× (11.04–11.11) | 0.26× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.30–0.33) | 0.42× (0.37–0.47) | 0.66× (0.63–0.69) | 0.56× (0.53–0.59) | 5.26× (5.02–5.52) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 96 MB | 0.32× (0.31–0.34) | 0.45× (0.40–0.50) | 0.57× (0.54–0.60) | 0.50× (0.47–0.53) | 3.34× (3.17–3.52) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.05× (8.02–8.09) | 0.65× (0.64–0.66) | 0.69× (0.69–0.70) | 0.75× (0.75–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.69× (0.62–0.76) | 1.63× (1.59–1.67) | 0.81× (0.79–0.84) | 5.92× (5.78–6.07) | 0.48× (0.46–0.49) | 0.51× (0.49–0.52) | 0.55× (0.54–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.65× (0.62–0.68) | 0.65× (0.62–0.68) | 1.15× (1.10–1.21) | 0.70× (0.67–0.73) | 3.36× (3.23–3.51) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.23× (5.20–5.25) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.41× (0.39–0.43) | 0.89× (0.85–0.92) | 0.71× (0.69–0.74) | 3.07× (2.96–3.19) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.45–0.46) | 0.85× (0.84–0.87) | 0.78× (0.75–0.81) | 2.48× (2.44–2.52) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.35× (0.32–0.38) | 0.83× (0.80–0.86) | 0.70× (0.68–0.72) | 2.86× (2.77–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.49× (0.47–0.51) | 0.80× (0.78–0.82) | 0.82× (0.79–0.86) | 2.32× (2.28–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.67) | 6.39× (6.36–6.42) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.40× (0.36–0.43) | 1.05× (1.02–1.08) | 0.67× (0.66–0.69) | 3.33× (3.27–3.40) | 0.21× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 56 MB | 0.51× (0.50–0.52) | 0.45× (0.42–0.49) | 0.91× (0.90–0.92) | 0.74× (0.71–0.76) | 2.32× (2.30–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 503 | 2.00 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 810 | 2.27 | 1.14× (1.11–1.17) | 80% | 1.06× (1.03–1.09) | 1.09× (1.07–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 1,021 | 3.50 | 1.75× (1.70–1.81) | 51% | 1.29× (1.24–1.35) | 1.78× (1.68–1.89) |

**Best:** mibz2 at 4 threads (1,021 MB/s); lbzip2 at 4 threads (228 MB/s); pbz2 at 4 threads (332 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.19, lbzip2 1t 0.07, lbzip2 2t 0.25, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.56 | 0.02 | 0.07 | 0.205 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3761 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.7% |
| MTF + RLE2 | 1.56 | 17.7% |
| T-vector build | 1.52 | 17.2% |
| walk (the one each build uses) | 4.15 | 47.1% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.396 | 1.066 | (in its total) | 2.68× (2.64–2.71) |  |
| walk | 1.042 | 2.038 | (in its total) | 1.95× (1.93–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.040 | (in its total) | 1.09× (1.06–1.12) |  |
| inverse BWT, total | 1.474 | 3.145 | 1.880 | 2.12× (2.10–2.15) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m09s |
| 4 threads | 30m13s ± 4m19s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m35s ± 2m59s |
| the scanner alone (its serial part) | 11m03s ± 32.2s |
| the workers alone, over 4 | 20m35s ± 2m59s |

At 4 threads the workers bound it. Per byte: the scanner 6.4 ns a byte of bz2, the workers 2.70 ns a byte of output at 4 threads (CPU, summed over them), 1.65 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,509,359 | 12,625,378 | \`78d0692041033556…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3761.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
