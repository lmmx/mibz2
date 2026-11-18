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

The latest record: [`7df6b69`](<https://github.com/lmmx/mibz2/commit/7df6b69fb3144554a0c0697d090d2b9412177693>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44292725058>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.420 s | 500 | 7.4 | 2.98× (2.93–3.03) | 6.87× (6.76–6.98) | 7.06× (6.98–7.14) | 8.06× (7.94–8.17) | 7.01× (6.92–7.10) | 7.39× (7.26–7.53) | 7.04× (6.99–7.09) | 2.19× (2.16–2.22) | 2.44× (2.41–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.238 s | 882 | 8.1 | 3.04× (2.84–3.25) | 6.20× (5.94–6.48) | 7.11× (6.78–7.47) | 7.47× (7.15–7.80) | 6.67× (6.30–7.07) | 12.82× (12.25–13.41) | 12.22× (11.72–12.73) | 3.80× (3.64–3.96) | 4.22× (4.05–4.40) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.199 s | 1,052 | 12.4 | 3.08× (2.97–3.20) | 4.62× (4.34–4.92) | 5.73× (5.52–5.94) | 5.48× (5.35–5.62) | 5.30× (5.13–5.47) | 15.47× (15.20–15.75) | 14.74× (14.46–15.04) | 4.59× (4.50–4.67) | 5.10× (5.00–5.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,111 | 3.6 | 0.57× (0.56–0.58) | 3.41× (3.34–3.47) | 4.55× (4.47–4.64) | 3.54× (3.49–3.60) | 6.05× (5.94–6.16) | 2.64× (2.59–2.69) | 2.65× (2.60–2.70) | 0.95× (0.92–0.98) | 2.09× (1.90–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,127 | 3.7 | 0.54× (0.53–0.56) | 2.88× (2.81–2.95) | 5.77× (5.63–5.91) | 3.60× (3.52–3.69) | 6.00× (5.87–6.14) | 2.67× (2.61–2.74) | 2.67× (2.61–2.74) | 0.96× (0.94–0.99) | 2.11× (1.91–2.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,072 | 3.8 | 0.52× (0.43–0.62) | 2.62× (2.26–3.03) | 5.22× (4.51–6.04) | 3.26× (2.81–3.78) | 5.58× (4.80–6.47) | 2.42× (2.09–2.80) | 2.42× (2.09–2.80) | 0.87× (0.76–1.00) | 1.91× (1.54–2.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 738 | 5.3 | 2.71× (2.69–2.73) | 3.08× (3.06–3.10) | 3.82× (3.74–3.90) | 4.12× (4.09–4.14) | 4.50× (4.46–4.55) | 2.78× (2.76–2.81) | 2.27× (2.25–2.29) | 2.12× (2.09–2.16) | 3.07× (3.05–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,133 | 6.6 | 2.30× (2.23–2.39) | 2.94× (2.84–3.04) | 4.38× (4.12–4.66) | 3.66× (3.52–3.81) | 5.22× (5.06–5.38) | 4.22× (4.07–4.37) | 3.44× (3.33–3.56) | 3.22× (3.14–3.30) | 4.65× (4.50–4.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 945 | 13.0 | 1.92× (1.86–1.97) | 1.85× (1.65–2.07) | 2.98× (2.80–3.17) | 2.10× (2.00–2.22) | 3.90× (3.78–4.03) | 3.55× (3.44–3.67) | 2.90× (2.81–2.99) | 2.71× (2.64–2.79) | 3.92× (3.79–4.05) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 52.3 | 1.38× (1.36–1.39) | 2.64× (2.62–2.66) | 2.38× (2.33–2.43) | 2.49× (2.47–2.51) | 2.76× (2.73–2.78) | 2.97× (2.94–2.99) | 3.12× (3.09–3.14) | 1.24× (1.23–1.26) | 1.41× (1.39–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 111 | 60.4 | 1.25× (1.21–1.29) | 2.43× (2.33–2.53) | 2.33× (2.26–2.41) | 3.06× (2.91–3.23) | 2.77× (2.66–2.89) | 4.66× (4.50–4.83) | 4.90× (4.72–5.09) | 1.95× (1.89–2.02) | 2.22× (2.15–2.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 142 | 76.9 | 1.33× (1.31–1.36) | 2.16× (2.09–2.23) | 2.21× (2.13–2.30) | 2.69× (2.59–2.80) | 2.89× (2.84–2.94) | 5.91× (5.79–6.02) | 6.20× (6.08–6.33) | 2.47× (2.41–2.53) | 2.81× (2.75–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 47.0 | 1.50× (1.45–1.55) | 3.01× (2.92–3.10) | 2.63× (2.56–2.69) | 2.75× (2.66–2.84) | 3.05× (2.96–3.14) | 3.64× (3.53–3.75) | 3.99× (3.86–4.12) | 1.52× (1.46–1.58) | 1.80× (1.72–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 53.6 | 1.36× (1.23–1.50) | 2.80× (2.48–3.16) | 2.46× (2.26–2.67) | 3.42× (3.16–3.71) | 2.98× (2.76–3.22) | 5.83× (5.40–6.30) | 6.39× (5.93–6.89) | 2.43× (2.28–2.60) | 2.88× (2.66–3.12) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 153 | 74.4 | 1.42× (1.37–1.47) | 2.26× (2.19–2.32) | 2.31× (2.23–2.39) | 3.04× (2.95–3.13) | 2.88× (2.81–2.95) | 7.03× (6.87–7.19) | 7.71× (7.50–7.91) | 2.93× (2.81–3.06) | 3.47× (3.35–3.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 5.0 | 0.63× (0.61–0.64) | 3.45× (3.41–3.50) | 4.34× (4.28–4.40) | 3.11× (3.07–3.16) | 6.62× (6.48–6.77) | 2.28× (2.25–2.32) | 2.33× (2.30–2.36) | 0.95× (0.93–0.97) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 806 | 5.4 | 0.52× (0.50–0.53) | 2.47× (2.42–2.52) | 4.96× (4.85–5.07) | 2.96× (2.90–3.01) | 6.30× (6.19–6.41) | 2.16× (2.12–2.21) | 2.21× (2.17–2.25) | 0.90× (0.87–0.93) | 1.65× (1.61–1.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 784 | 5.3 | 0.51× (0.50–0.52) | 2.44× (2.39–2.49) | 4.88× (4.79–4.98) | 2.92× (2.86–2.99) | 6.39× (5.96–6.85) | 2.13× (2.08–2.18) | 2.17× (2.13–2.22) | 0.89× (0.86–0.91) | 1.62× (1.58–1.66) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 0.057 s | 225 | 16.9 | 1.81× (1.79–1.82) | 3.91× (3.88–3.94) | 4.29× (4.23–4.36) | 4.54× (4.50–4.57) | 4.68× (4.63–4.72) | 4.65× (4.62–4.69) | 4.80× (4.75–4.84) | 1.51× (1.45–1.58) | 1.77× (1.76–1.79) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 0.039 s | 332 | 21.0 | 1.52× (1.46–1.58) | 3.31× (3.16–3.47) | 3.99× (3.64–4.37) | 4.35× (4.14–4.58) | 4.30× (4.16–4.45) | 6.77× (6.51–7.04) | 6.97× (6.69–7.27) | 2.20× (2.06–2.35) | 2.58× (2.48–2.68) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 0.035 s | 369 | 31.7 | 1.52× (1.47–1.57) | 2.52× (2.43–2.61) | 3.29× (3.08–3.52) | 3.43× (3.35–3.50) | 3.94× (3.85–4.03) | 7.64× (7.47–7.82) | 7.87× (7.71–8.04) | 2.49× (2.36–2.62) | 2.91× (2.84–2.99) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.97× (2.92–3.02) | 6.84× (6.73–6.94) | 7.03× (6.94–7.11) | 8.03× (7.91–8.14) | 6.99× (6.90–7.08) | 7.36× (7.23–7.50) | 7.02× (6.97–7.07) | 2.19× (2.15–2.22) | 2.42× (2.40–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.81× (2.75–2.88) | 6.32× (6.24–6.40) | 6.65× (6.52–6.78) | 7.74× (7.63–7.85) | 6.64× (6.36–6.94) | 6.76× (6.65–6.87) | 6.44× (6.36–6.52) | 2.01× (1.98–2.04) | 2.22× (2.19–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (2.96–3.16) | 5.18× (4.82–5.56) | 5.58× (5.24–5.94) | 6.06× (5.83–6.30) | 5.21× (5.01–5.42) | 4.43× (4.30–4.58) | 4.23× (4.13–4.33) | 1.32× (1.28–1.35) | 1.46× (1.42–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.92× (2.87–2.97) | 3.88× (3.82–3.93) | 3.32× (3.27–3.37) | 5.61× (5.52–5.71) | 2.50× (2.45–2.55) | 2.50× (2.46–2.55) | 0.96× (0.93–0.98) | 1.65× (1.62–1.69) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.77× (2.71–2.83) | 5.02× (4.92–5.11) | 3.34× (3.27–3.41) | 5.48× (5.37–5.59) | 2.48× (2.43–2.53) | 2.48× (2.43–2.54) | 0.95× (0.93–0.97) | 1.64× (1.60–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.46–0.65) | 2.84× (2.44–3.30) | 4.62× (3.98–5.36) | 3.09× (2.67–3.58) | 5.18× (4.46–6.01) | 2.29× (1.98–2.66) | 2.30× (1.98–2.66) | 0.88× (0.77–1.01) | 1.52× (1.30–1.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.59–2.63) | 2.85× (2.83–2.86) | 3.41× (3.39–3.43) | 3.93× (3.91–3.95) | 4.30× (4.25–4.34) | 2.68× (2.66–2.70) | 2.20× (2.18–2.21) | 2.06× (2.03–2.09) | 2.67× (2.64–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.25–2.32) | 2.51× (2.46–2.56) | 3.36× (3.30–3.43) | 3.34× (3.28–3.41) | 3.61× (3.55–3.68) | 2.17× (2.12–2.22) | 1.78× (1.74–1.81) | 1.67× (1.63–1.70) | 2.16× (2.11–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.85–2.08) | 1.73× (1.67–1.78) | 2.26× (2.20–2.32) | 2.04× (1.98–2.10) | 2.11× (2.05–2.17) | 1.10× (1.07–1.13) | 0.90× (0.88–0.93) | 0.85× (0.82–0.87) | 1.10× (1.07–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.56× (2.55–2.57) | 2.34× (2.31–2.37) | 2.47× (2.45–2.49) | 2.73× (2.70–2.76) | 2.94× (2.91–2.96) | 3.09× (3.06–3.11) | 1.24× (1.23–1.25) | 1.35× (1.35–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.22–1.29) | 2.37× (2.31–2.44) | 2.12× (2.08–2.16) | 2.85× (2.73–2.98) | 2.45× (2.37–2.53) | 2.55× (2.47–2.63) | 2.68× (2.59–2.77) | 1.08× (1.04–1.11) | 1.18× (1.14–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.52) | 2.30× (2.25–2.35) | 1.95× (1.91–2.00) | 2.66× (2.60–2.73) | 2.50× (2.46–2.54) | 1.99× (1.95–2.03) | 2.09× (2.05–2.14) | 0.84× (0.82–0.86) | 0.92× (0.90–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.44–1.54) | 2.94× (2.85–3.03) | 2.58× (2.52–2.64) | 2.72× (2.64–2.81) | 3.02× (2.93–3.10) | 3.60× (3.50–3.71) | 3.95× (3.82–4.07) | 1.51× (1.45–1.57) | 1.73× (1.67–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.34–1.42) | 2.72× (2.66–2.78) | 2.32× (2.27–2.37) | 3.23× (3.16–3.31) | 2.72× (2.66–2.78) | 3.16× (3.10–3.23) | 3.47× (3.40–3.54) | 1.33× (1.30–1.36) | 1.52× (1.49–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.52–1.62) | 2.51× (2.46–2.56) | 2.00× (1.97–2.04) | 2.85× (2.78–2.93) | 2.62× (2.58–2.65) | 2.29× (2.25–2.33) | 2.51× (2.47–2.55) | 0.96× (0.93–0.99) | 1.10× (1.08–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.47× (2.45–2.50) | 3.43× (3.40–3.46) | 2.85× (2.81–2.89) | 5.92× (5.81–6.04) | 2.12× (2.09–2.15) | 2.17× (2.14–2.19) | 0.96× (0.94–0.98) | 1.37× (1.35–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.24× (2.21–2.28) | 4.01× (3.95–4.06) | 2.73× (2.68–2.77) | 5.56× (5.47–5.65) | 1.97× (1.94–2.01) | 2.02× (1.98–2.05) | 0.89× (0.87–0.92) | 1.27× (1.25–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.49× (2.43–2.55) | 4.06× (3.94–4.19) | 2.78× (2.72–2.83) | 5.80× (5.41–6.22) | 2.00× (1.96–2.04) | 2.04× (2.01–2.08) | 0.91× (0.88–0.93) | 1.29× (1.27–1.32) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 1.78× (1.77–1.80) | 3.75× (3.72–3.77) | 4.16× (4.12–4.20) | 4.44× (4.41–4.47) | 4.58× (4.54–4.62) | 4.55× (4.52–4.59) | 4.69× (4.65–4.73) | 1.50× (1.44–1.57) | 1.61× (1.60–1.62) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 1.53× (1.51–1.55) | 3.24× (3.10–3.38) | 3.68× (3.34–4.06) | 4.10× (4.05–4.16) | 3.77× (3.72–3.82) | 3.65× (3.57–3.73) | 3.76× (3.67–3.85) | 1.20× (1.14–1.27) | 1.29× (1.26–1.32) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 1.68× (1.62–1.74) | 2.65× (2.56–2.74) | 3.03× (2.85–3.21) | 3.29× (3.19–3.40) | 3.18× (3.04–3.34) | 2.42× (2.36–2.48) | 2.49× (2.43–2.55) | 0.80× (0.76–0.84) | 0.85× (0.83–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.31–0.34) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.00× (10.98–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.31–0.34) | 0.44× (0.39–0.50) | 0.68× (0.64–0.71) | 0.57× (0.55–0.60) | 5.37× (5.12–5.63) | 0.13× (0.12–0.13) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.36) | 0.51× (0.42–0.63) | 0.61× (0.58–0.63) | 0.53× (0.51–0.55) | 3.52× (3.38–3.67) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.02–8.05) | 0.65× (0.64–0.65) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.66–0.71) | 0.68× (0.62–0.73) | 1.64× (1.58–1.69) | 0.82× (0.79–0.85) | 5.93× (5.74–6.13) | 0.48× (0.46–0.49) | 0.51× (0.49–0.53) | 0.55× (0.53–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.66× (0.63–0.69) | 0.71× (0.68–0.74) | 1.16× (1.13–1.18) | 0.71× (0.69–0.72) | 3.39× (3.31–3.46) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.42× (0.41–0.43) | 0.90× (0.87–0.93) | 0.73× (0.70–0.75) | 3.12× (3.02–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.87× (0.86–0.89) | 0.81× (0.79–0.84) | 2.54× (2.49–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.19× (5.18–5.20) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.48) | 0.36× (0.34–0.39) | 0.82× (0.80–0.84) | 0.69× (0.67–0.72) | 2.82× (2.75–2.89) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.44–0.50) | 0.79× (0.78–0.80) | 0.80× (0.77–0.84) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.25) | 1.07× (1.07–1.07) | 0.77× (0.76–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.31× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.43) | 0.67× (0.67–0.67) | 6.45× (6.44–6.46) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 0.63× (0.62–0.64) | 0.31× (0.31–0.32) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 36 MB | 0.46× (0.46–0.47) | 0.40× (0.37–0.43) | 1.10× (1.07–1.12) | 0.69× (0.66–0.71) | 3.42× (3.36–3.48) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.34) | 0.17× (0.16–0.17) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 54 MB | 0.51× (0.49–0.52) | 0.47× (0.43–0.51) | 0.93× (0.90–0.95) | 0.74× (0.71–0.77) | 2.37× (2.32–2.42) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 500 | 2.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 882 | 2.19 | 1.09× (1.08–1.10) | 87% | 1.01× (0.99–1.03) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,052 | 3.36 | 1.66× (1.62–1.70) | 52% | 1.26× (1.17–1.35) | 1.71× (1.66–1.77) |

**Best:** mibz2 at 4 threads (1,052 MB/s); lbzip2 at 4 threads (237 MB/s); pbz2 at 4 threads (336 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.24, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.54 | 0.02 | 0.08 | 0.199 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.8% |
| MTF + RLE2 | 1.53 | 17.4% |
| T-vector build | 1.49 | 17.0% |
| walk (the one each build uses) | 4.18 | 47.8% |
| of which: segment assembly | 0.17 | 1.9% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.75 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.061 | (in its total) | 2.71× (2.70–2.72) |  |
| walk | 1.077 | 2.108 | (in its total) | 1.96× (1.95–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.042 | (in its total) | 0.91× (0.88–0.94) |  |
| inverse BWT, total | 1.515 | 3.212 | 1.898 | 2.12× (2.11–2.13) | 1.25× (1.24–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 6m59s |
| 4 threads | 29m17s ± 4m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m15s ± 2m46s |
| the scanner alone (its serial part) | 11m38s ± 0.00s |
| the workers alone, over 4 | 19m14s ± 2m46s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.52 ns a byte of output at 4 threads (CPU, summed over them), 1.63 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,527,905 | 12,873,504 | \`d928dbd828b8a8a4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
