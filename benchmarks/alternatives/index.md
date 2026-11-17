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

The latest record: [`120a1cc`](<https://github.com/lmmx/mibz2/commit/120a1cc9ebba1b9de96d9fe073be60dac27fc66b>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44070924420>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.421 s | 498 | 7.5 | 2.92× (2.90–2.95) | 6.70× (6.63–6.77) | 7.01× (6.91–7.10) | 7.92× (7.81–8.04) | 6.92× (6.87–6.98) | 7.25× (7.17–7.34) | 6.90× (6.75–7.05) | 2.18× (2.15–2.22) | 2.42× (2.40–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.242 s | 865 | 8.2 | 2.97× (2.85–3.10) | 6.07× (5.92–6.22) | 7.09× (6.91–7.28) | 7.32× (7.15–7.50) | 6.42× (6.29–6.56) | 12.53× (12.22–12.85) | 11.93× (11.55–12.31) | 3.77× (3.67–3.88) | 4.18× (4.08–4.28) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.198 s | 1,058 | 12.5 | 2.97× (2.86–3.09) | 4.36× (4.18–4.56) | 5.44× (5.21–5.69) | 5.24× (5.04–5.45) | 5.18× (4.96–5.42) | 15.13× (14.63–15.64) | 14.39× (14.10–14.70) | 4.55× (4.43–4.68) | 5.04× (4.87–5.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 3.6 | 0.59× (0.55–0.63) | 3.44× (3.38–3.50) | 4.59× (4.51–4.67) | 3.58× (3.53–3.63) | 6.08× (5.95–6.21) | 2.66× (2.62–2.71) | 2.66× (2.62–2.71) | 0.97× (0.95–0.99) | 1.93× (1.74–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.53× (0.52–0.54) | 2.83× (2.75–2.91) | 5.72× (5.53–5.91) | 3.53× (3.44–3.62) | 6.06× (5.70–6.44) | 2.62× (2.55–2.69) | 2.62× (2.55–2.69) | 0.95× (0.93–0.97) | 1.90× (1.73–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,108 | 3.7 | 0.54× (0.53–0.54) | 2.84× (2.80–2.88) | 5.67× (5.58–5.75) | 3.54× (3.49–3.59) | 5.98× (5.91–6.05) | 2.62× (2.58–2.66) | 2.62× (2.58–2.66) | 0.95× (0.94–0.97) | 1.90× (1.71–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 744 | 5.3 | 2.77× (2.67–2.88) | 3.10× (3.08–3.13) | 3.88× (3.85–3.91) | 4.14× (4.11–4.17) | 4.57× (4.37–4.77) | 2.79× (2.77–2.82) | 2.29× (2.27–2.31) | 2.11× (2.10–2.13) | 3.09× (3.06–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,120 | 6.7 | 2.19× (1.95–2.45) | 2.83× (2.49–3.21) | 4.12× (3.63–4.67) | 3.44× (3.06–3.86) | 4.81× (4.26–5.43) | 3.91× (3.45–4.43) | 3.20× (2.82–3.62) | 2.95× (2.62–3.33) | 4.32× (3.81–4.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 938 | 13.3 | 1.85× (1.79–1.93) | 1.80× (1.68–1.93) | 2.92× (2.65–3.23) | 2.15× (2.04–2.27) | 3.97× (3.76–4.18) | 3.56× (3.42–3.71) | 2.91× (2.80–3.04) | 2.69× (2.59–2.79) | 3.93× (3.78–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.6 | 1.39× (1.38–1.40) | 2.66× (2.63–2.69) | 2.39× (2.37–2.41) | 2.51× (2.49–2.53) | 2.80× (2.78–2.82) | 3.00× (2.98–3.02) | 3.10× (3.06–3.14) | 1.27× (1.24–1.29) | 1.40× (1.38–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 58.9 | 1.30× (1.26–1.34) | 2.50× (2.39–2.61) | 2.34× (2.27–2.42) | 3.17× (3.03–3.31) | 2.88× (2.84–2.92) | 4.85× (4.76–4.95) | 5.01× (4.89–5.14) | 2.05× (1.99–2.11) | 2.27× (2.24–2.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 145 | 76.9 | 1.34× (1.30–1.38) | 2.19× (2.08–2.30) | 2.19× (2.09–2.30) | 2.67× (2.59–2.75) | 2.91× (2.84–2.97) | 5.94× (5.77–6.11) | 6.14× (5.99–6.28) | 2.51× (2.42–2.60) | 2.78× (2.68–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.119 s | 80 | 46.7 | 1.51× (1.45–1.57) | 3.04× (2.93–3.17) | 2.61× (2.50–2.72) | 2.73× (2.63–2.83) | 3.04× (2.93–3.15) | 3.62× (3.49–3.76) | 3.91× (3.75–4.08) | 1.51× (1.44–1.58) | 1.77× (1.70–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 133 | 52.9 | 1.33× (1.26–1.40) | 2.73× (2.59–2.87) | 2.47× (2.35–2.60) | 3.44× (3.26–3.62) | 2.99× (2.84–3.14) | 5.87× (5.58–6.17) | 6.34× (6.05–6.65) | 2.45× (2.32–2.59) | 2.88× (2.71–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 73.5 | 1.42× (1.39–1.46) | 2.28× (2.21–2.35) | 2.30× (2.22–2.38) | 3.09× (3.00–3.18) | 2.87× (2.80–2.94) | 7.06× (6.89–7.24) | 7.63× (7.40–7.86) | 2.95× (2.88–3.02) | 3.46× (3.35–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 4.9 | 0.63× (0.63–0.64) | 3.51× (3.48–3.54) | 4.41× (4.37–4.45) | 3.14× (3.11–3.17) | 6.70× (6.63–6.76) | 2.31× (2.29–2.32) | 2.35× (2.32–2.38) | 0.97× (0.95–0.98) | 1.76× (1.75–1.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 803 | 5.3 | 0.52× (0.50–0.53) | 2.46× (2.42–2.51) | 4.95× (4.85–5.05) | 2.95× (2.89–3.01) | 6.29× (6.18–6.41) | 2.15× (2.11–2.19) | 2.19× (2.16–2.22) | 0.90× (0.88–0.93) | 1.64× (1.61–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 797 | 5.3 | 0.52× (0.52–0.53) | 2.48× (2.45–2.50) | 4.96× (4.90–5.02) | 2.97× (2.94–3.00) | 6.31× (6.14–6.49) | 2.15× (2.13–2.18) | 2.19× (2.17–2.22) | 0.90× (0.89–0.92) | 1.65× (1.63–1.66) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 0.056 s | 224 | 17.0 | 1.75× (1.69–1.82) | 3.82× (3.67–3.98) | 4.19× (4.02–4.37) | 4.45× (4.26–4.65) | 4.61× (4.44–4.78) | 4.54× (4.38–4.71) | 4.48× (4.32–4.65) | 1.47× (1.41–1.53) | 1.64× (1.56–1.71) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 0.038 s | 326 | 20.9 | 1.43× (1.33–1.54) | 3.19× (2.95–3.46) | 3.69× (3.48–3.92) | 4.35× (4.16–4.54) | 4.15× (3.87–4.45) | 6.49× (6.07–6.94) | 6.40× (6.00–6.83) | 2.10× (1.97–2.23) | 2.34× (2.17–2.52) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 0.036 s | 350 | 32.4 | 1.41× (1.36–1.46) | 2.31× (2.10–2.54) | 3.01× (2.83–3.20) | 3.38× (3.27–3.50) | 3.68× (3.56–3.81) | 7.32× (7.12–7.52) | 7.22× (7.03–7.41) | 2.36× (2.29–2.43) | 2.63× (2.55–2.72) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.89–2.94) | 6.67× (6.60–6.74) | 6.97× (6.87–7.07) | 7.89× (7.78–8.01) | 6.90× (6.84–6.95) | 7.23× (7.14–7.31) | 6.88× (6.73–7.02) | 2.18× (2.14–2.21) | 2.40× (2.38–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.78× (2.69–2.88) | 6.23× (6.15–6.30) | 6.57× (6.41–6.72) | 7.60× (7.48–7.72) | 6.34× (6.28–6.41) | 6.57× (6.49–6.65) | 6.25× (6.11–6.40) | 1.98× (1.95–2.02) | 2.18× (2.16–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.93× (2.85–3.01) | 4.78× (4.64–4.92) | 5.18× (5.03–5.34) | 5.76× (5.62–5.91) | 5.02× (4.85–5.19) | 4.27× (4.17–4.37) | 4.06× (3.99–4.12) | 1.29× (1.26–1.31) | 1.42× (1.39–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.59–0.66) | 2.96× (2.91–3.00) | 3.92× (3.87–3.97) | 3.35× (3.31–3.39) | 5.63× (5.52–5.75) | 2.52× (2.48–2.56) | 2.52× (2.48–2.56) | 0.97× (0.96–0.99) | 1.65× (1.62–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.73× (2.67–2.79) | 4.98× (4.82–5.15) | 3.28× (3.21–3.35) | 5.54× (5.22–5.88) | 2.44× (2.38–2.50) | 2.44× (2.38–2.49) | 0.94× (0.93–0.96) | 1.60× (1.56–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.58) | 3.06× (3.02–3.10) | 5.04× (4.97–5.11) | 3.35× (3.30–3.39) | 5.55× (5.49–5.61) | 2.49× (2.45–2.52) | 2.49× (2.45–2.52) | 0.96× (0.95–0.98) | 1.63× (1.61–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.56–2.78) | 2.86× (2.84–2.88) | 3.42× (3.40–3.43) | 3.95× (3.92–3.97) | 4.35× (4.17–4.54) | 2.69× (2.67–2.71) | 2.21× (2.20–2.23) | 2.05× (2.03–2.06) | 2.67× (2.65–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.17–2.40) | 2.44× (2.34–2.55) | 3.30× (3.19–3.42) | 3.28× (3.18–3.39) | 3.53× (3.42–3.64) | 2.12× (2.05–2.19) | 1.75× (1.69–1.80) | 1.62× (1.57–1.66) | 2.11× (2.04–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.94–2.12) | 1.69× (1.60–1.79) | 2.22× (2.12–2.32) | 2.03× (1.96–2.11) | 2.07× (1.98–2.17) | 1.08× (1.03–1.13) | 0.89× (0.85–0.93) | 0.82× (0.79–0.86) | 1.07× (1.02–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.58× (2.56–2.60) | 2.34× (2.33–2.35) | 2.49× (2.46–2.51) | 2.77× (2.75–2.79) | 2.97× (2.95–2.99) | 3.06× (3.03–3.10) | 1.26× (1.23–1.29) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.24–1.35) | 2.40× (2.37–2.43) | 2.13× (2.10–2.17) | 2.92× (2.84–3.01) | 2.49× (2.46–2.52) | 2.61× (2.56–2.65) | 2.69× (2.64–2.74) | 1.11× (1.08–1.14) | 1.20× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.46–1.53) | 2.35× (2.31–2.39) | 1.98× (1.95–2.01) | 2.68× (2.63–2.72) | 2.53× (2.50–2.57) | 2.02× (1.99–2.05) | 2.09× (2.05–2.13) | 0.86× (0.84–0.88) | 0.93× (0.92–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.44–1.56) | 2.95× (2.84–3.07) | 2.55× (2.44–2.66) | 2.70× (2.60–2.81) | 3.01× (2.90–3.12) | 3.58× (3.45–3.72) | 3.87× (3.72–4.03) | 1.50× (1.44–1.57) | 1.71× (1.64–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.36–1.41) | 2.75× (2.70–2.80) | 2.36× (2.32–2.40) | 3.31× (3.24–3.38) | 2.74× (2.69–2.79) | 3.20× (3.14–3.26) | 3.45× (3.38–3.53) | 1.34× (1.32–1.36) | 1.53× (1.48–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.56–1.67) | 2.53× (2.45–2.61) | 2.01× (1.95–2.08) | 2.89× (2.80–2.99) | 2.60× (2.53–2.69) | 2.32× (2.24–2.39) | 2.50× (2.42–2.59) | 0.97× (0.94–1.00) | 1.11× (1.06–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.50× (2.47–2.53) | 3.48× (3.41–3.54) | 2.87× (2.84–2.90) | 5.98× (5.91–6.04) | 2.14× (2.13–2.16) | 2.18× (2.15–2.21) | 0.97× (0.96–0.99) | 1.39× (1.37–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.25× (2.20–2.30) | 4.01× (3.94–4.07) | 2.73× (2.67–2.79) | 5.58× (5.48–5.69) | 1.97× (1.93–2.02) | 2.01× (1.98–2.04) | 0.90× (0.88–0.92) | 1.28× (1.25–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.52× (2.49–2.55) | 4.06× (4.00–4.12) | 2.81× (2.78–2.83) | 5.72× (5.58–5.88) | 2.02× (2.00–2.05) | 2.06× (2.04–2.09) | 0.92× (0.90–0.94) | 1.31× (1.29–1.33) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 1.73× (1.67–1.79) | 3.69× (3.54–3.84) | 4.03× (3.89–4.18) | 4.35× (4.17–4.55) | 4.50× (4.34–4.67) | 4.44× (4.28–4.61) | 4.38× (4.22–4.54) | 1.45× (1.40–1.51) | 1.58× (1.52–1.64) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 1.51× (1.50–1.53) | 3.22× (3.10–3.36) | 3.51× (3.46–3.55) | 4.11× (4.07–4.15) | 3.82× (3.76–3.89) | 3.64× (3.61–3.68) | 3.59× (3.56–3.63) | 1.19× (1.18–1.21) | 1.30× (1.28–1.31) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 1.61× (1.55–1.66) | 2.62× (2.42–2.83) | 2.88× (2.75–3.01) | 3.20× (3.11–3.30) | 3.12× (3.04–3.20) | 2.36× (2.28–2.44) | 2.33× (2.25–2.40) | 0.77× (0.75–0.79) | 0.84× (0.81–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.97–11.04) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.47× (0.38–0.58) | 0.69× (0.67–0.70) | 0.57× (0.55–0.58) | 5.42× (5.27–5.57) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.31–0.35) | 0.52× (0.44–0.62) | 0.59× (0.56–0.62) | 0.51× (0.49–0.54) | 3.43× (3.27–3.60) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.02–8.06) | 0.65× (0.65–0.65) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.69) | 0.62× (0.56–0.68) | 1.59× (1.52–1.66) | 0.79× (0.76–0.83) | 5.77× (5.53–6.03) | 0.47× (0.45–0.49) | 0.50× (0.47–0.52) | 0.53× (0.51–0.56) | 0.26× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.71× (0.65–0.76) | 1.15× (1.10–1.21) | 0.71× (0.68–0.75) | 3.36× (3.20–3.53) | 0.26× (0.25–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.74) | 5.18× (5.17–5.19) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.50) | 0.42× (0.40–0.43) | 0.89× (0.86–0.92) | 0.72× (0.69–0.74) | 3.09× (2.97–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.29–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.87) | 0.81× (0.79–0.83) | 2.51× (2.46–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.19× (5.18–5.21) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.37× (0.34–0.40) | 0.86× (0.83–0.88) | 0.73× (0.70–0.75) | 2.94× (2.85–3.03) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.52× (0.49–0.55) | 0.80× (0.79–0.82) | 0.81× (0.79–0.84) | 2.33× (2.28–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.07× (1.07–1.08) | 0.77× (0.77–0.78) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.66–0.67) | 6.38× (6.37–6.39) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 37 MB | 0.46× (0.44–0.47) | 0.40× (0.37–0.44) | 1.07× (1.03–1.11) | 0.68× (0.66–0.71) | 3.37× (3.27–3.48) | 0.21× (0.20–0.22) | 0.22× (0.22–0.23) | 0.34× (0.32–0.35) | 0.16× (0.16–0.17) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 55 MB | 0.50× (0.49–0.52) | 0.46× (0.42–0.50) | 0.92× (0.89–0.96) | 0.74× (0.71–0.77) | 2.34× (2.27–2.42) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 498 | 2.02 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 865 | 2.21 | 1.10× (1.09–1.10) | 86% | 1.03× (1.02–1.04) | 1.05× (1.02–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,058 | 3.38 | 1.69× (1.64–1.75) | 52% | 1.21× (1.19–1.24) | 1.70× (1.66–1.74) |

**Best:** mibz2 at 4 threads (1,058 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (355 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.22, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.54 | 0.02 | 0.08 | 0.198 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3703 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.9% |
| MTF + RLE2 | 1.55 | 17.8% |
| T-vector build | 1.49 | 17.2% |
| walk (the one each build uses) | 4.08 | 46.9% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.69 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.060 | (in its total) | 2.69× (2.65–2.73) |  |
| walk | 1.057 | 2.076 | (in its total) | 1.95× (1.92–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.041 | 0.041 | (in its total) | 1.02× (0.95–1.08) |  |
| inverse BWT, total | 1.489 | 3.178 | 1.907 | 2.12× (2.09–2.15) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 6m57s |
| 4 threads | 29m23s ± 4m19s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m41s ± 2m52s |
| the scanner alone (its serial part) | 11m38s ± 29.4s |
| the workers alone, over 4 | 19m41s ± 2m52s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.58 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,457,111 | 12,465,313 | \`f5a13800cacccde1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3703.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
