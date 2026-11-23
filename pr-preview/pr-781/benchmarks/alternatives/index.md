# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-781/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`affe873`](<https://github.com/lmmx/mibz2/commit/affe8737e0a317c8d0cd96105f426fa7c279192e>) (2026-11-23).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45091102467>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.391 s | 536 | 8.5 | 3.11× (3.09–3.13) | 7.14× (7.10–7.17) | 7.40× (7.36–7.44) | 8.45× (8.37–8.54) | 7.40× (7.36–7.43) | 7.78× (7.73–7.83) | 7.30× (7.27–7.33) | 2.30× (2.29–2.31) | 2.60× (2.58–2.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.227 s | 924 | 9.2 | 3.18× (3.01–3.36) | 6.40× (6.04–6.79) | 7.55× (7.14–7.98) | 7.66× (7.25–8.10) | 6.79× (6.42–7.19) | 13.27× (12.51–14.08) | 12.45× (11.74–13.20) | 3.92× (3.70–4.15) | 4.44× (4.17–4.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.185 s | 1,133 | 14.4 | 3.25× (3.13–3.37) | 4.86× (4.66–5.06) | 5.97× (5.79–6.15) | 5.78× (5.58–5.98) | 5.69× (5.50–5.89) | 16.32× (15.90–16.75) | 15.31× (14.91–15.72) | 4.82× (4.69–4.95) | 5.46× (5.30–5.62) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,075 | 4.6 | 0.56× (0.56–0.57) | 3.32× (3.28–3.36) | 4.47× (4.33–4.62) | 3.50× (3.44–3.57) | 5.97× (5.85–6.10) | 2.58× (2.55–2.62) | 2.58× (2.55–2.61) | 0.95× (0.93–0.98) | 2.03× (1.84–2.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,050 | 4.9 | 0.51× (0.50–0.52) | 2.69× (2.64–2.74) | 5.38× (5.28–5.48) | 3.38× (3.32–3.45) | 5.71× (5.58–5.84) | 2.50× (2.46–2.55) | 2.50× (2.45–2.54) | 0.92× (0.91–0.93) | 1.97× (1.78–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,044 | 4.8 | 0.51× (0.49–0.52) | 2.67× (2.60–2.74) | 5.33× (5.20–5.46) | 3.34× (3.26–3.42) | 5.71× (5.49–5.93) | 2.48× (2.43–2.53) | 2.47× (2.41–2.53) | 0.91× (0.88–0.95) | 1.95× (1.77–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 745 | 6.5 | 2.74× (2.70–2.79) | 3.10× (3.08–3.12) | 3.88× (3.86–3.91) | 4.16× (4.13–4.18) | 4.57× (4.53–4.60) | 2.81× (2.79–2.82) | 2.30× (2.29–2.31) | 2.14× (2.10–2.17) | 3.09× (3.07–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,150 | 7.9 | 2.37× (2.29–2.44) | 3.00× (2.92–3.09) | 4.88× (4.65–5.13) | 3.74× (3.60–3.88) | 5.42× (5.25–5.60) | 4.31× (4.19–4.43) | 3.53× (3.43–3.63) | 3.28× (3.17–3.39) | 4.75× (4.61–4.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,043 | 15.2 | 2.06× (1.96–2.16) | 2.06× (1.83–2.30) | 3.43× (3.31–3.55) | 2.42× (2.30–2.54) | 4.41× (4.26–4.56) | 3.95× (3.79–4.12) | 3.24× (3.11–3.37) | 3.01× (2.89–3.13) | 4.36× (4.18–4.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 75 | 61.3 | 1.43× (1.42–1.43) | 2.74× (2.70–2.77) | 2.50× (2.44–2.55) | 2.57× (2.55–2.60) | 2.92× (2.90–2.94) | 3.10× (3.06–3.13) | 3.10× (3.07–3.12) | 1.30× (1.29–1.31) | 1.47× (1.44–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 122 | 68.5 | 1.31× (1.26–1.37) | 2.56× (2.41–2.71) | 2.41× (2.31–2.51) | 3.23× (3.01–3.47) | 2.98× (2.85–3.11) | 4.99× (4.77–5.21) | 4.99× (4.77–5.21) | 2.09× (2.00–2.18) | 2.37× (2.28–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 152 | 90.8 | 1.41× (1.38–1.44) | 2.34× (2.25–2.43) | 2.30× (2.20–2.41) | 2.78× (2.72–2.85) | 3.13× (3.06–3.21) | 6.31× (6.20–6.42) | 6.31× (6.19–6.43) | 2.65× (2.59–2.71) | 3.00× (2.91–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 55.6 | 1.52× (1.44–1.61) | 3.04× (2.87–3.22) | 2.65× (2.52–2.79) | 2.75× (2.61–2.90) | 3.15× (3.02–3.28) | 3.67× (3.48–3.88) | 3.86× (3.66–4.08) | 1.52× (1.42–1.61) | 1.79× (1.69–1.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 138 | 62.7 | 1.44× (1.39–1.49) | 2.85× (2.75–2.96) | 2.64× (2.57–2.71) | 3.68× (3.58–3.78) | 3.27× (3.16–3.38) | 6.30× (6.12–6.48) | 6.62× (6.43–6.82) | 2.60× (2.48–2.72) | 3.07× (2.97–3.18) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.059 s | 162 | 90.1 | 1.50× (1.46–1.54) | 2.47× (2.30–2.65) | 2.35× (2.25–2.46) | 3.18× (3.11–3.26) | 3.05× (2.96–3.14) | 7.36× (7.15–7.58) | 7.74× (7.53–7.95) | 3.03× (2.90–3.18) | 3.59× (3.46–3.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 6.2 | 0.63× (0.62–0.64) | 3.46× (3.44–3.48) | 4.34× (4.31–4.38) | 3.11× (3.10–3.13) | 6.81× (6.70–6.93) | 2.28× (2.27–2.30) | 2.32× (2.31–2.34) | 0.95× (0.95–0.96) | 1.76× (1.72–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 7.0 | 0.50× (0.49–0.52) | 2.40× (2.34–2.46) | 4.82× (4.70–4.95) | 2.89× (2.81–2.96) | 6.22× (6.05–6.39) | 2.10× (2.05–2.16) | 2.14× (2.09–2.19) | 0.88× (0.86–0.90) | 1.62× (1.56–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 769 | 6.9 | 0.50× (0.49–0.51) | 2.46× (2.29–2.65) | 4.78× (4.73–4.84) | 2.89× (2.85–2.94) | 6.21× (6.11–6.31) | 2.09× (2.06–2.11) | 2.12× (2.10–2.15) | 0.87× (0.86–0.89) | 1.60× (1.57–1.64) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.058 s | 241 | 19.4 | 1.90× (1.89–1.92) | 4.05× (4.00–4.10) | 4.55× (4.48–4.61) | 4.77× (4.71–4.83) | 4.93× (4.90–4.96) | 4.93× (4.87–4.99) | 4.81× (4.76–4.86) | 1.57× (1.55–1.58) | 1.75× (1.73–1.78) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.042 s | 333 | 24.5 | 1.42× (1.34–1.50) | 3.15× (3.01–3.31) | 3.94× (3.73–4.16) | 4.40× (4.22–4.60) | 4.16× (3.97–4.35) | 6.77× (6.46–7.09) | 6.60× (6.28–6.93) | 2.15× (2.05–2.26) | 2.41× (2.30–2.53) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.037 s | 376 | 38.4 | 1.50× (1.45–1.55) | 2.35× (2.25–2.46) | 3.38× (3.24–3.53) | 3.48× (3.37–3.60) | 3.83× (3.76–3.90) | 7.71× (7.58–7.85) | 7.51× (7.40–7.62) | 2.45× (2.42–2.48) | 2.74× (2.70–2.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.10× (3.08–3.12) | 7.10× (7.06–7.13) | 7.36× (7.32–7.40) | 8.42× (8.33–8.51) | 7.36× (7.33–7.40) | 7.75× (7.69–7.80) | 7.27× (7.24–7.30) | 2.29× (2.29–2.30) | 2.57× (2.55–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.01× (2.97–3.05) | 6.75× (6.69–6.80) | 7.13× (7.05–7.21) | 8.21× (8.15–8.28) | 6.93× (6.88–6.97) | 7.19× (7.14–7.24) | 6.74× (6.69–6.79) | 2.13× (2.11–2.14) | 2.39× (2.36–2.41) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.13× (3.06–3.21) | 5.23× (5.10–5.37) | 5.66× (5.51–5.82) | 6.18× (6.11–6.25) | 5.43× (5.27–5.60) | 4.54× (4.48–4.61) | 4.26× (4.21–4.32) | 1.34× (1.33–1.36) | 1.51× (1.49–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.85× (2.80–2.89) | 3.83× (3.73–3.93) | 3.29× (3.21–3.37) | 5.54× (5.43–5.65) | 2.44× (2.41–2.47) | 2.44× (2.41–2.47) | 0.96× (0.93–0.98) | 1.60× (1.58–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.56) | 2.55× (2.51–2.59) | 4.64× (4.56–4.71) | 3.09× (3.04–3.14) | 5.13× (5.03–5.24) | 2.29× (2.25–2.32) | 2.28× (2.25–2.32) | 0.90× (0.88–0.91) | 1.50× (1.48–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.54) | 2.84× (2.79–2.90) | 4.65× (4.55–4.75) | 3.12× (3.06–3.17) | 5.22× (5.04–5.40) | 2.31× (2.27–2.36) | 2.31× (2.27–2.36) | 0.91× (0.88–0.94) | 1.52× (1.49–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.59–2.68) | 2.87× (2.85–2.89) | 3.43× (3.40–3.45) | 3.96× (3.94–3.98) | 4.34× (4.31–4.38) | 2.69× (2.68–2.71) | 2.22× (2.21–2.23) | 2.06× (2.03–2.10) | 2.67× (2.65–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.25–2.38) | 2.55× (2.46–2.64) | 3.43× (3.33–3.54) | 3.38× (3.30–3.47) | 3.71× (3.60–3.82) | 2.19× (2.13–2.25) | 1.80× (1.75–1.85) | 1.68× (1.63–1.72) | 2.17× (2.11–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.09× (1.99–2.20) | 1.78× (1.73–1.84) | 2.35× (2.26–2.45) | 2.15× (2.07–2.24) | 2.22× (2.14–2.31) | 1.15× (1.10–1.19) | 0.94× (0.91–0.98) | 0.88× (0.85–0.91) | 1.14× (1.09–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.42–1.43) | 2.66× (2.64–2.67) | 2.44× (2.39–2.49) | 2.55× (2.52–2.57) | 2.89× (2.87–2.91) | 3.06× (3.03–3.10) | 3.06× (3.04–3.08) | 1.29× (1.28–1.30) | 1.42× (1.41–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.29–1.34) | 2.49× (2.43–2.55) | 2.20× (2.15–2.25) | 2.99× (2.93–3.06) | 2.60× (2.53–2.68) | 2.70× (2.63–2.78) | 2.70× (2.64–2.77) | 1.14× (1.12–1.17) | 1.25× (1.22–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.48–1.58) | 2.40× (2.30–2.51) | 2.01× (1.97–2.05) | 2.74× (2.69–2.79) | 2.64× (2.57–2.72) | 2.08× (2.05–2.11) | 2.08× (2.04–2.11) | 0.88× (0.86–0.89) | 0.96× (0.95–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.43–1.60) | 2.96× (2.80–3.12) | 2.59× (2.46–2.73) | 2.73× (2.59–2.87) | 3.11× (2.99–3.24) | 3.63× (3.44–3.83) | 3.82× (3.62–4.03) | 1.51× (1.42–1.61) | 1.73× (1.64–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.43× (1.38–1.48) | 2.83× (2.73–2.92) | 2.41× (2.33–2.49) | 3.39× (3.29–3.49) | 2.86× (2.76–2.97) | 3.29× (3.19–3.40) | 3.46× (3.35–3.58) | 1.37× (1.30–1.43) | 1.57× (1.51–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.66) | 2.54× (2.48–2.60) | 2.02× (1.97–2.08) | 2.88× (2.81–2.96) | 2.66× (2.60–2.71) | 2.32× (2.29–2.36) | 2.44× (2.40–2.48) | 0.96× (0.94–0.99) | 1.11× (1.09–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.46× (2.44–2.49) | 3.40× (3.38–3.42) | 2.84× (2.81–2.86) | 6.05× (5.95–6.15) | 2.12× (2.10–2.13) | 2.15× (2.13–2.17) | 0.96× (0.95–0.97) | 1.36× (1.36–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.12× (2.07–2.17) | 3.78× (3.69–3.88) | 2.59× (2.53–2.65) | 5.33× (5.20–5.47) | 1.87× (1.83–1.92) | 1.90× (1.86–1.94) | 0.85× (0.83–0.87) | 1.21× (1.18–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.49× (2.19–2.82) | 3.82× (3.76–3.88) | 2.67× (2.63–2.71) | 5.48× (5.39–5.58) | 1.91× (1.89–1.94) | 1.94× (1.92–1.97) | 0.87× (0.85–0.88) | 1.23× (1.22–1.25) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.88× (1.86–1.89) | 3.94× (3.90–3.98) | 4.33× (4.28–4.38) | 4.66× (4.60–4.72) | 4.82× (4.79–4.85) | 4.82× (4.76–4.88) | 4.70× (4.65–4.75) | 1.55× (1.54–1.56) | 1.70× (1.68–1.72) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.58× (1.57–1.60) | 3.28× (3.23–3.32) | 3.64× (3.60–3.68) | 4.27× (4.23–4.31) | 3.90× (3.86–3.95) | 3.83× (3.79–3.88) | 3.74× (3.70–3.77) | 1.23× (1.22–1.24) | 1.35× (1.34–1.37) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.65× (1.60–1.71) | 2.59× (2.48–2.71) | 2.90× (2.81–2.98) | 3.27× (3.22–3.32) | 3.08× (3.05–3.11) | 2.45× (2.42–2.48) | 2.38× (2.36–2.41) | 0.79× (0.78–0.80) | 0.86× (0.85–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.97–11.05) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.37) | 0.45× (0.41–0.50) | 0.73× (0.70–0.75) | 0.61× (0.59–0.63) | 5.83× (5.64–6.02) | 0.14× (0.13–0.14) | 0.15× (0.14–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.37× (0.35–0.39) | 0.53× (0.44–0.63) | 0.65× (0.61–0.68) | 0.57× (0.53–0.61) | 3.80× (3.60–4.01) | 0.09× (0.08–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.95× (7.93–7.97) | 0.65× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.67–0.72) | 0.69× (0.63–0.77) | 1.66× (1.61–1.72) | 0.83× (0.80–0.85) | 6.03× (5.83–6.24) | 0.49× (0.47–0.51) | 0.51× (0.49–0.53) | 0.56× (0.54–0.58) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.68× (0.66–0.71) | 0.76× (0.68–0.84) | 1.23× (1.18–1.28) | 0.76× (0.73–0.79) | 3.62× (3.48–3.77) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.32× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.39× (0.37–0.42) | 0.88× (0.86–0.90) | 0.70× (0.68–0.72) | 3.04× (2.96–3.12) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.83× (0.81–0.84) | 2.54× (2.49–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.19× (5.16–5.21) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.41) | 0.84× (0.80–0.87) | 0.72× (0.69–0.75) | 2.88× (2.76–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.53) | 0.81× (0.80–0.82) | 0.80× (0.77–0.83) | 2.34× (2.29–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.90) | 6.20× (6.20–6.21) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.91) | 6.20× (6.18–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.40× (6.38–6.43) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 34 MB | 0.50× (0.49–0.50) | 0.44× (0.40–0.47) | 1.17× (1.14–1.19) | 0.76× (0.74–0.77) | 3.67× (3.63–3.72) | 0.22× (0.22–0.23) | 0.24× (0.24–0.24) | 0.36× (0.36–0.37) | 0.18× (0.17–0.18) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 51 MB | 0.54× (0.53–0.55) | 0.47× (0.44–0.49) | 1.00× (0.97–1.02) | 0.80× (0.77–0.83) | 2.52× (2.49–2.56) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 536 | 1.87 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 924 | 2.02 | 1.08× (1.07–1.09) | 85% | 1.03× (1.01–1.04) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,133 | 3.18 | 1.71× (1.68–1.73) | 52% | 1.26× (1.22–1.30) | 1.72× (1.66–1.79) |

**Best:** mibz2 at 4 threads (1,133 MB/s); lbzip2 at 4 threads (232 MB/s); pbz2 at 4 threads (351 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.53 | 0.02 | 0.07 | 0.185 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4532 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.09 | 10.3% |
| MTF + RLE2 | 1.90 | 18.0% |
| T-vector build | 1.85 | 17.6% |
| walk (the one each build uses) | 5.15 | 49.0% |
| of which: segment assembly | 0.20 | 1.9% |
| RLE1 | 0.23 | 2.2% |
| CRC | 0.32 | 3.0% |
| **total** | 10.51 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.064 | (in its total) | 2.71× (2.70–2.73) |  |
| walk | 1.060 | 2.038 | (in its total) | 1.92× (1.91–1.94) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.041 | (in its total) | 1.11× (1.09–1.13) |  |
| inverse BWT, total | 1.488 | 3.143 | 1.868 | 2.11× (2.10–2.12) | 1.26× (1.25–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m54s ± 6m43s |
| 4 threads | 27m09s ± 3m53s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m04s ± 2m43s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 19m04s ± 2m43s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 2.50 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 4532.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
