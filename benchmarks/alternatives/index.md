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

The latest record: [`9ece704`](<https://github.com/lmmx/mibz2/commit/9ece704953d815d1d0e8dd184c99defce273dd4d>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42306802033>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.529 s | 396 | 9.3 | 2.30× (2.29–2.31) | 5.31× (5.28–5.33) | 5.47× (5.45–5.49) | 6.23× (6.19–6.26) | 5.44× (5.42–5.46) | 6.15× (6.12–6.18) | 5.38× (5.34–5.43) | 1.71× (1.70–1.71) | 1.90× (1.89–1.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.299 s | 701 | 9.9 | 2.35× (2.27–2.44) | 4.87× (4.70–5.04) | 5.58× (5.37–5.80) | 5.81× (5.59–6.05) | 5.13× (4.92–5.34) | 10.82× (10.40–11.25) | 9.47× (9.11–9.84) | 3.00× (2.90–3.12) | 3.35× (3.22–3.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.243 s | 862 | 16.3 | 2.24× (2.13–2.36) | 3.45× (3.27–3.64) | 4.19× (3.95–4.44) | 4.16× (3.94–4.38) | 4.07× (3.87–4.28) | 13.08× (12.41–13.79) | 11.45× (10.86–12.07) | 3.63× (3.45–3.83) | 4.05× (3.84–4.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,133 | 3.6 | 0.58× (0.57–0.59) | 3.47× (3.41–3.53) | 4.64× (4.56–4.72) | 3.60× (3.54–3.66) | 6.19× (6.12–6.27) | 2.71× (2.67–2.75) | 2.68× (2.64–2.73) | 0.98× (0.95–1.01) | 1.74× (1.71–1.77) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,124 | 3.6 | 0.54× (0.53–0.55) | 2.87× (2.82–2.92) | 5.76× (5.66–5.85) | 3.58× (3.52–3.64) | 6.04× (5.94–6.14) | 2.68× (2.64–2.73) | 2.66× (2.62–2.70) | 0.97× (0.93–1.01) | 1.72× (1.70–1.75) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,105 | 3.6 | 0.54× (0.53–0.54) | 2.85× (2.80–2.89) | 5.76× (5.53–6.01) | 3.55× (3.49–3.62) | 6.06× (5.93–6.19) | 2.65× (2.61–2.69) | 2.63× (2.59–2.67) | 0.96× (0.92–1.00) | 1.70× (1.67–1.73) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 690 | 5.7 | 2.53× (2.51–2.54) | 2.88× (2.86–2.89) | 3.60× (3.58–3.62) | 3.82× (3.80–3.84) | 4.19× (4.14–4.23) | 3.21× (3.18–3.24) | 2.11× (2.10–2.12) | 1.94× (1.92–1.95) | 2.87× (2.85–2.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,051 | 6.7 | 2.10× (1.94–2.27) | 2.74× (2.59–2.89) | 4.12× (3.75–4.53) | 3.36× (3.17–3.56) | 4.73× (4.40–5.09) | 4.77× (4.48–5.09) | 3.14× (2.94–3.35) | 2.88× (2.70–3.08) | 4.26× (3.99–4.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 954 | 13.3 | 1.78× (1.72–1.85) | 1.71× (1.57–1.87) | 2.85× (2.66–3.06) | 2.15× (2.03–2.28) | 3.95× (3.84–4.05) | 4.43× (4.30–4.57) | 2.92× (2.83–3.01) | 2.67× (2.59–2.76) | 3.96× (3.83–4.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.108 s | 74 | 50.6 | 1.39× (1.35–1.42) | 2.68× (2.60–2.76) | 2.44× (2.37–2.51) | 2.55× (2.49–2.60) | 2.84× (2.77–2.91) | 3.19× (3.12–3.26) | 3.01× (2.95–3.08) | 1.26× (1.23–1.29) | 1.40× (1.39–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 56.3 | 1.29× (1.26–1.32) | 2.69× (2.46–2.94) | 2.43× (2.38–2.48) | 3.19× (3.07–3.31) | 2.93× (2.87–2.99) | 5.19× (5.10–5.28) | 4.90× (4.83–4.99) | 2.05× (2.01–2.08) | 2.28× (2.25–2.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 145 | 74.7 | 1.34× (1.30–1.38) | 2.12× (2.08–2.16) | 2.12× (2.04–2.21) | 2.65× (2.61–2.70) | 2.92× (2.86–2.98) | 6.27× (6.13–6.40) | 5.92× (5.79–6.05) | 2.47× (2.43–2.52) | 2.76× (2.68–2.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 45.5 | 1.54× (1.54–1.55) | 3.08× (3.05–3.11) | 2.69× (2.69–2.70) | 2.85× (2.84–2.86) | 3.17× (3.15–3.19) | 4.05× (4.04–4.07) | 3.94× (3.93–3.94) | 1.52× (1.52–1.52) | 1.82× (1.82–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 s | 136 | 51.3 | 1.37× (1.30–1.44) | 2.76× (2.65–2.89) | 2.56× (2.44–2.69) | 3.65× (3.60–3.70) | 3.14× (2.99–3.30) | 6.57× (6.26–6.91) | 6.38× (6.08–6.70) | 2.47× (2.35–2.59) | 2.96× (2.82–3.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.061 s | 156 | 71.8 | 1.44× (1.42–1.46) | 2.30× (2.23–2.37) | 2.32× (2.27–2.38) | 3.11× (3.05–3.18) | 2.95× (2.88–3.02) | 7.78× (7.62–7.94) | 7.55× (7.41–7.70) | 2.92× (2.86–2.97) | 3.50× (3.43–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 4.9 | 0.62× (0.60–0.64) | 3.28× (2.96–3.63) | 4.23× (3.88–4.62) | 3.00× (2.78–3.24) | 6.49× (5.94–7.08) | 2.27× (2.09–2.47) | 2.24× (2.06–2.44) | 0.93× (0.87–0.99) | 1.70× (1.56–1.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 826 | 5.2 | 0.53× (0.52–0.54) | 2.54× (2.50–2.59) | 5.11× (5.02–5.20) | 3.02× (2.97–3.08) | 6.55× (6.38–6.73) | 2.28× (2.24–2.32) | 2.25× (2.21–2.29) | 0.93× (0.90–0.96) | 1.70× (1.67–1.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 813 | 5.1 | 0.53× (0.52–0.54) | 2.52× (2.48–2.56) | 5.04× (4.96–5.12) | 3.00× (2.96–3.04) | 6.47× (6.34–6.61) | 2.25× (2.21–2.28) | 2.22× (2.19–2.25) | 0.92× (0.89–0.95) | 1.68× (1.65–1.70) | yes |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 1 | 0.066 s | 189 | 19.9 | 1.44× (1.44–1.45) | 3.10× (3.09–3.11) | 3.41× (3.40–3.42) | 3.55× (3.55–3.56) | 3.77× (3.73–3.81) | 3.86× (3.85–3.88) | 3.56× (3.55–3.58) | 1.18× (1.17–1.18) | 1.39× (1.39–1.40) | yes |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 2 | 0.040 s | 309 | 23.0 | 1.30× (1.26–1.34) | 2.75× (2.65–2.86) | 3.30× (3.25–3.36) | 3.76× (3.69–3.84) | 3.82× (3.75–3.89) | 6.28× (6.18–6.37) | 5.79× (5.70–5.88) | 1.91× (1.88–1.94) | 2.26× (2.23–2.30) | yes |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 4 | 0.034 s | 365 | 33.3 | 1.34× (1.31–1.37) | 2.17× (2.11–2.24) | 2.77× (2.64–2.91) | 3.21× (3.14–3.28) | 3.53× (3.44–3.62) | 7.42× (7.27–7.58) | 6.84× (6.70–6.99) | 2.26× (2.22–2.29) | 2.68× (2.62–2.73) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.29× (2.28–2.31) | 5.28× (5.26–5.30) | 5.45× (5.42–5.47) | 6.21× (6.18–6.24) | 5.43× (5.41–5.45) | 6.13× (6.10–6.16) | 5.37× (5.33–5.41) | 1.71× (1.70–1.71) | 1.89× (1.88–1.90) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.21× (2.18–2.25) | 5.04× (5.01–5.08) | 5.23× (5.17–5.28) | 6.09× (6.02–6.15) | 5.12× (5.06–5.19) | 5.74× (5.68–5.79) | 5.02× (5.00–5.04) | 1.60× (1.58–1.61) | 1.77× (1.75–1.79) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.13× (2.09–2.17) | 3.60× (3.54–3.66) | 3.85× (3.77–3.92) | 4.38× (4.30–4.45) | 3.77× (3.71–3.83) | 3.52× (3.46–3.58) | 3.08× (3.04–3.13) | 0.98× (0.96–1.00) | 1.09× (1.07–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.96× (2.90–3.01) | 3.91× (3.85–3.97) | 3.37× (3.32–3.42) | 5.72× (5.67–5.78) | 2.55× (2.52–2.59) | 2.53× (2.49–2.57) | 0.98× (0.95–1.01) | 1.65× (1.62–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.77× (2.72–2.82) | 4.95× (4.86–5.04) | 3.33× (3.28–3.39) | 5.53× (5.44–5.63) | 2.50× (2.46–2.54) | 2.48× (2.44–2.52) | 0.96× (0.93–1.00) | 1.62× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.58) | 3.05× (3.00–3.10) | 5.07× (4.84–5.31) | 3.38× (3.32–3.44) | 5.65× (5.55–5.76) | 2.53× (2.49–2.56) | 2.50× (2.47–2.54) | 0.97× (0.94–1.01) | 1.63× (1.61–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.44× (2.43–2.45) | 2.66× (2.65–2.67) | 3.14× (3.13–3.15) | 3.66× (3.64–3.67) | 4.00× (3.96–4.04) | 3.08× (3.06–3.11) | 2.05× (2.04–2.06) | 1.88× (1.87–1.90) | 2.47× (2.45–2.48) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.21× (2.17–2.25) | 2.42× (2.34–2.50) | 3.25× (3.18–3.33) | 3.24× (3.17–3.31) | 3.49× (3.41–3.56) | 2.60× (2.55–2.65) | 1.73× (1.69–1.76) | 1.59× (1.56–1.62) | 2.08× (2.04–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.91–1.97) | 1.64× (1.59–1.68) | 2.15× (2.12–2.18) | 2.02× (1.96–2.08) | 2.04× (2.01–2.07) | 1.30× (1.28–1.33) | 0.87× (0.85–0.88) | 0.80× (0.78–0.81) | 1.04× (1.02–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.35–1.41) | 2.59× (2.54–2.65) | 2.39× (2.32–2.46) | 2.52× (2.47–2.58) | 2.81× (2.74–2.88) | 3.16× (3.09–3.23) | 2.98× (2.92–3.05) | 1.25× (1.23–1.28) | 1.38× (1.35–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.29–1.31) | 2.49× (2.47–2.52) | 2.20× (2.19–2.22) | 3.03× (3.00–3.06) | 2.60× (2.58–2.62) | 2.86× (2.84–2.88) | 2.70× (2.69–2.72) | 1.14× (1.13–1.15) | 1.25× (1.24–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.41–1.52) | 2.32× (2.27–2.37) | 1.95× (1.92–1.99) | 2.70× (2.64–2.76) | 2.55× (2.48–2.62) | 2.15× (2.11–2.20) | 2.04× (1.99–2.08) | 0.86× (0.84–0.88) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.53–1.54) | 3.03× (3.02–3.04) | 2.62× (2.62–2.63) | 2.83× (2.82–2.83) | 3.14× (3.11–3.16) | 4.01× (3.99–4.02) | 3.89× (3.89–3.90) | 1.51× (1.51–1.52) | 1.75× (1.75–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.40–1.41) | 2.81× (2.79–2.82) | 2.40× (2.39–2.41) | 3.42× (3.39–3.44) | 2.85× (2.83–2.88) | 3.55× (3.53–3.57) | 3.45× (3.43–3.46) | 1.34× (1.33–1.35) | 1.55× (1.54–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.57–1.68) | 2.55× (2.50–2.60) | 2.04× (2.00–2.08) | 2.95× (2.87–3.03) | 2.67× (2.61–2.74) | 2.54× (2.48–2.60) | 2.47× (2.41–2.53) | 0.96× (0.94–0.98) | 1.11× (1.08–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.45× (2.24–2.69) | 3.31× (3.09–3.54) | 2.75× (2.56–2.96) | 5.81× (5.34–6.32) | 2.12× (1.95–2.29) | 2.09× (1.93–2.26) | 0.94× (0.88–1.00) | 1.33× (1.23–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.60) | 2.30× (2.27–2.33) | 4.03× (3.98–4.07) | 2.78× (2.74–2.81) | 5.77× (5.63–5.91) | 2.08× (2.05–2.10) | 2.05× (2.03–2.07) | 0.92× (0.90–0.94) | 1.31× (1.29–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.53× (2.51–2.56) | 4.11× (4.01–4.21) | 2.84× (2.81–2.88) | 5.88× (5.76–6.00) | 2.11× (2.08–2.15) | 2.09× (2.06–2.12) | 0.94× (0.91–0.96) | 1.33× (1.31–1.35) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 1 | 1.43× (1.43–1.44) | 2.95× (2.94–2.96) | 3.26× (3.25–3.27) | 3.49× (3.48–3.50) | 3.69× (3.66–3.73) | 3.79× (3.77–3.80) | 3.50× (3.48–3.51) | 1.17× (1.17–1.17) | 1.27× (1.27–1.28) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 2 | 1.31× (1.30–1.31) | 2.65× (2.62–2.67) | 2.96× (2.94–2.98) | 3.40× (3.38–3.41) | 3.25× (3.23–3.27) | 3.28× (3.27–3.30) | 3.03× (3.01–3.04) | 1.01× (1.01–1.02) | 1.10× (1.10–1.11) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 4 | 1.52× (1.49–1.56) | 2.29× (2.21–2.36) | 2.52× (2.46–2.58) | 2.84× (2.81–2.88) | 2.74× (2.69–2.80) | 2.27× (2.22–2.32) | 2.10× (2.05–2.14) | 0.70× (0.69–0.72) | 0.76× (0.75–0.78) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.35–0.38) | 1.08× (1.08–1.08) | 0.86× (0.86–0.87) | 12.23× (12.21–12.25) | 0.28× (0.28–0.29) | 0.31× (0.31–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.55× (0.49–0.61) | 0.79× (0.78–0.80) | 0.66× (0.66–0.67) | 6.37× (6.31–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.59× (0.50–0.71) | 0.69× (0.68–0.71) | 0.61× (0.59–0.63) | 4.10× (4.07–4.13) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.92× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.44× (4.44–4.45) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.19× (8.17–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.38× (0.38–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.64× (0.59–0.69) | 1.67× (1.65–1.69) | 0.82× (0.81–0.83) | 6.03× (5.96–6.09) | 0.48× (0.47–0.48) | 0.50× (0.50–0.51) | 0.55× (0.54–0.55) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.66) | 0.71× (0.66–0.75) | 1.17× (1.13–1.21) | 0.76× (0.74–0.78) | 3.45× (3.35–3.54) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.47–0.52) | 0.41× (0.38–0.45) | 0.93× (0.89–0.97) | 0.73× (0.69–0.77) | 3.20× (3.07–3.34) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.46) | 0.87× (0.86–0.88) | 0.82× (0.80–0.83) | 2.53× (2.50–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.47× (0.46–0.49) | 0.38× (0.35–0.41) | 0.87× (0.84–0.89) | 0.72× (0.70–0.75) | 2.96× (2.87–3.05) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.81× (0.78–0.85) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.25) | 1.09× (1.09–1.10) | 0.77× (0.76–0.78) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.43× (5.41–5.44) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.77) | 5.45× (5.43–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.40–0.41) | 1.50× (1.50–1.51) | 0.65× (0.65–0.65) | 6.79× (6.77–6.82) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.66–0.66) | 0.33× (0.33–0.34) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 2 | 33 MB | 0.48× (0.47–0.49) | 0.39× (0.36–0.41) | 1.17× (1.14–1.21) | 0.73× (0.71–0.74) | 3.68× (3.61–3.75) | 0.22× (0.22–0.22) | 0.23× (0.23–0.24) | 0.36× (0.35–0.37) | 0.18× (0.18–0.18) |
| `text` (1.81 MB → 12.4 MB, 6.8×) | 4 | 50 MB | 0.54× (0.52–0.56) | 0.53× (0.48–0.59) | 1.00× (0.97–1.03) | 0.80× (0.76–0.83) | 2.58× (2.49–2.67) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 396 | 2.53 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 701 | 2.70 | 1.07× (1.06–1.08) | 88% | 1.02× (1.01–1.03) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 862 | 4.43 | 1.74× (1.71–1.77) | 53% | 1.19× (1.18–1.19) | 1.61× (1.59–1.64) |

**Best:** mibz2 at 4 threads (862 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (376 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.05, lbzip2 1t 0.06, lbzip2 2t 0.26, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.74 | 0.02 | 0.08 | 0.243 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3677 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.10 | 12.4% |
| MTF + RLE2 | 1.51 | 17.0% |
| T-vector build | 1.76 | 19.9% |
| walk (the one each build uses) | 4.03 | 45.5% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.25 | 2.9% |
| **total** | 8.85 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.453 | 1.065 | (in its total) | 2.35× (2.35–2.35) |  |
| walk | 1.045 | 2.079 | (in its total) | 1.99× (1.99–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.040 | 0.040 | (in its total) | 1.02× (1.01–1.03) |  |
| inverse BWT, total | 1.538 | 3.185 | 1.916 | 2.07× (2.07–2.07) | 1.25× (1.24–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m06s |
| 4 threads | 36m20s ± 5m29s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m10s ± 3m52s |
| the scanner alone (its serial part) | 12m13s ± 32.2s |
| the workers alone, over 4 | 27m10s ± 3m52s |

At 4 threads the workers bound it. Per byte: the scanner 7.1 ns a byte of bz2, the workers 3.56 ns a byte of output at 4 threads (CPU, summed over them), 2.13 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,814,441 | 12,421,916 | \`77c41b5d3b41ba96…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3677.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.2 (bench/report.py), 10 rounds, threads 1, 2, 4
