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

The latest record: [`2268743`](<https://github.com/lmmx/mibz2/commit/2268743d0a694f22e24e0a185f617dec68e28c4f>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43964253307>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.331 s | 633 | 6.8 | 2.71× (2.64–2.78) | 6.94× (6.76–7.13) | 7.20× (7.10–7.30) | 8.14× (8.03–8.24) | 7.22× (7.05–7.40) | 7.53× (7.33–7.74) | 7.23× (7.10–7.36) | 2.12× (2.07–2.17) | 2.44× (2.42–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.202 s | 1,040 | 7.5 | 2.78× (2.71–2.86) | 6.06× (5.85–6.27) | 7.32× (7.08–7.56) | 7.34× (7.12–7.58) | 6.47× (6.31–6.63) | 12.59× (12.36–12.83) | 12.08× (11.71–12.47) | 3.54× (3.45–3.63) | 4.09× (3.98–4.20) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.167 s | 1,256 | 11.6 | 2.92× (2.86–2.98) | 4.64× (4.45–4.84) | 6.18× (5.68–6.72) | 5.40× (5.21–5.60) | 5.63× (5.15–6.15) | 15.15× (14.71–15.59) | 14.53× (14.08–15.00) | 4.26× (4.14–4.38) | 4.91× (4.78–5.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,324 | 3.6 | 0.54× (0.47–0.60) | 3.49× (2.93–4.16) | 4.38× (3.83–5.01) | 3.58× (3.21–3.99) | 6.25× (5.53–7.07) | 2.77× (2.36–3.25) | 2.64× (2.37–2.94) | 0.96× (0.87–1.06) | 1.97× (1.76–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,320 | 3.6 | 0.48× (0.43–0.54) | 2.83× (2.52–3.18) | 5.52× (4.90–6.21) | 3.41× (3.03–3.83) | 6.04× (5.23–6.98) | 2.67× (2.27–3.15) | 2.55× (2.27–2.86) | 0.93× (0.83–1.04) | 1.90× (1.67–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,282 | 3.6 | 0.51× (0.49–0.52) | 3.09× (2.86–3.33) | 5.73× (5.55–5.91) | 3.57× (3.48–3.66) | 6.41× (5.87–7.00) | 2.77× (2.49–3.09) | 2.65× (2.60–2.69) | 0.96× (0.94–0.98) | 1.97× (1.92–2.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,008 | 4.6 | 2.83× (2.77–2.88) | 3.64× (3.53–3.76) | 4.32× (4.21–4.43) | 4.74× (4.39–5.12) | 5.28× (5.20–5.36) | 2.99× (2.89–3.08) | 2.55× (2.50–2.61) | 2.14× (2.11–2.17) | 3.11× (3.02–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,347 | 6.3 | 2.03× (1.78–2.32) | 2.87× (2.49–3.30) | 4.12× (3.65–4.65) | 3.52× (3.07–4.03) | 5.23× (4.58–5.98) | 3.82× (3.32–4.39) | 3.27× (2.82–3.78) | 2.74× (2.36–3.17) | 3.98× (3.43–4.61) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,120 | 12.6 | 1.82× (1.59–2.08) | 1.87× (1.71–2.05) | 2.73× (2.55–2.93) | 2.00× (1.86–2.14) | 4.05× (3.91–4.19) | 3.23× (3.09–3.39) | 2.76× (2.65–2.88) | 2.32× (2.22–2.42) | 3.37× (3.19–3.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.085 s | 94 | 46.3 | 1.37× (1.33–1.40) | 3.07× (3.00–3.13) | 2.78× (2.71–2.86) | 2.69× (2.63–2.75) | 3.03× (2.96–3.10) | 3.46× (3.34–3.60) | 3.48× (3.37–3.60) | 1.34× (1.31–1.38) | 1.50× (1.45–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.056 s | 143 | 55.0 | 1.27× (1.25–1.29) | 2.82× (2.63–3.03) | 2.62× (2.53–2.70) | 3.33× (3.20–3.47) | 2.99× (2.92–3.06) | 5.43× (5.19–5.68) | 5.46× (5.29–5.62) | 2.10× (2.05–2.16) | 2.35× (2.28–2.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.047 s | 172 | 71.1 | 1.28× (1.23–1.34) | 2.50× (2.35–2.66) | 2.44× (2.32–2.58) | 2.82× (2.74–2.90) | 3.11× (2.97–3.27) | 6.58× (6.32–6.85) | 6.62× (6.47–6.76) | 2.55× (2.47–2.63) | 2.85× (2.73–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.090 s | 107 | 40.6 | 1.51× (1.48–1.54) | 3.79× (3.69–3.88) | 3.09× (3.03–3.15) | 3.19× (3.05–3.34) | 3.57× (3.48–3.67) | 4.49× (4.39–4.58) | 4.79× (4.70–4.89) | 1.64× (1.60–1.69) | 1.86× (1.79–1.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.060 s | 160 | 48.9 | 1.32× (1.24–1.40) | 3.30× (3.20–3.40) | 2.74× (2.67–2.81) | 3.80× (3.62–3.98) | 3.33× (3.08–3.61) | 6.79× (6.59–7.00) | 7.25× (7.02–7.49) | 2.49× (2.39–2.59) | 2.82× (2.70–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.051 s | 189 | 69.5 | 1.33× (1.30–1.37) | 2.60× (2.48–2.73) | 2.53× (2.31–2.77) | 3.22× (3.13–3.32) | 3.23× (2.99–3.49) | 7.86× (7.70–8.04) | 8.40× (8.17–8.64) | 2.88× (2.78–2.98) | 3.27× (3.13–3.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,005 | 4.9 | 0.62× (0.61–0.63) | 3.09× (3.00–3.19) | 4.24× (4.10–4.39) | 3.37× (2.90–3.91) | 7.32× (6.74–7.94) | 2.33× (2.28–2.38) | 2.35× (2.29–2.40) | 0.99× (0.96–1.02) | 2.06× (2.00–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 952 | 5.3 | 0.50× (0.49–0.51) | 2.95× (2.86–3.03) | 5.14× (4.95–5.34) | 2.98× (2.92–3.05) | 6.64× (6.46–6.83) | 2.22× (2.15–2.28) | 2.23× (2.18–2.29) | 0.94× (0.90–0.98) | 1.96× (1.91–2.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 957 | 5.1 | 0.45× (0.34–0.59) | 2.64× (2.04–3.43) | 4.59× (3.52–5.99) | 2.74× (2.08–3.60) | 6.07× (4.64–7.94) | 1.98× (1.52–2.59) | 2.00× (1.53–2.60) | 0.84× (0.66–1.08) | 1.76× (1.36–2.28) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 0.044 s | 278 | 15.9 | 1.71× (1.61–1.82) | 4.17× (4.07–4.28) | 4.49× (4.27–4.72) | 4.63× (4.44–4.82) | 4.88× (4.66–5.11) | 4.93× (4.67–5.21) | 4.73× (4.55–4.92) | 1.51× (1.44–1.59) | 1.81× (1.63–2.01) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 0.035 s | 355 | 21.4 | 1.34× (1.11–1.62) | 3.36× (2.68–4.21) | 3.61× (3.35–3.89) | 4.22× (3.93–4.52) | 4.24× (3.67–4.89) | 6.34× (5.90–6.81) | 6.08× (5.70–6.48) | 1.94× (1.83–2.06) | 2.33× (2.13–2.53) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 0.030 s | 402 | 31.9 | 1.20× (1.04–1.37) | 2.56× (2.26–2.90) | 2.99× (2.76–3.24) | 3.36× (2.96–3.81) | 3.52× (3.13–3.96) | 6.81× (6.04–7.68) | 6.53× (5.76–7.41) | 2.09× (1.85–2.36) | 2.50× (2.11–2.95) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.70× (2.63–2.77) | 6.89× (6.71–7.08) | 7.15× (7.05–7.25) | 8.10× (7.99–8.20) | 7.19× (7.02–7.36) | 7.50× (7.30–7.70) | 7.19× (7.06–7.32) | 2.11× (2.06–2.16) | 2.42× (2.39–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.55× (2.49–2.62) | 6.23× (6.12–6.34) | 6.78× (6.63–6.93) | 7.71× (7.59–7.83) | 6.46× (6.33–6.58) | 6.69× (6.54–6.85) | 6.42× (6.23–6.62) | 1.89× (1.85–1.93) | 2.16× (2.11–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.88× (2.77–3.00) | 5.25× (5.03–5.48) | 5.94× (5.51–6.40) | 6.12× (5.92–6.32) | 5.60× (5.11–6.13) | 4.40× (4.26–4.55) | 4.22× (4.07–4.38) | 1.24× (1.20–1.28) | 1.42× (1.38–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.51–0.65) | 3.05× (2.56–3.65) | 3.74× (3.32–4.22) | 3.30× (2.94–3.71) | 5.68× (4.99–6.47) | 2.58× (2.18–3.06) | 2.46× (2.19–2.76) | 0.96× (0.86–1.06) | 1.73× (1.54–1.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.47–0.60) | 2.61× (2.30–2.98) | 4.63× (4.08–5.26) | 3.11× (2.74–3.54) | 5.41× (4.64–6.32) | 2.45× (2.05–2.93) | 2.33× (2.05–2.65) | 0.91× (0.80–1.03) | 1.65× (1.45–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.57) | 3.09× (2.92–3.27) | 5.03× (4.89–5.16) | 3.37× (3.28–3.45) | 5.91× (5.41–6.45) | 2.63× (2.35–2.94) | 2.50× (2.46–2.54) | 0.97× (0.95–1.00) | 1.77× (1.74–1.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.63–2.73) | 3.14× (3.08–3.21) | 3.77× (3.71–3.83) | 4.43× (4.11–4.78) | 4.93× (4.85–5.00) | 2.82× (2.73–2.91) | 2.42× (2.37–2.48) | 2.05× (2.02–2.07) | 2.74× (2.70–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (1.89–2.29) | 2.48× (2.21–2.77) | 2.96× (2.69–3.26) | 3.28× (2.95–3.65) | 3.67× (3.31–4.07) | 2.02× (1.83–2.23) | 1.73× (1.55–1.95) | 1.46× (1.30–1.65) | 1.96× (1.74–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.74–2.38) | 1.70× (1.58–1.83) | 1.99× (1.89–2.09) | 1.99× (1.88–2.10) | 2.16× (2.08–2.25) | 1.03× (0.98–1.09) | 0.89× (0.85–0.93) | 0.75× (0.72–0.78) | 1.00× (0.96–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.33–1.39) | 2.96× (2.89–3.03) | 2.69× (2.61–2.76) | 2.66× (2.60–2.72) | 2.99× (2.93–3.05) | 3.42× (3.29–3.54) | 3.43× (3.33–3.55) | 1.33× (1.30–1.37) | 1.43× (1.39–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.21–1.29) | 2.66× (2.61–2.72) | 2.32× (2.26–2.38) | 3.01× (2.94–3.08) | 2.58× (2.53–2.63) | 2.92× (2.79–3.05) | 2.93× (2.85–3.02) | 1.14× (1.12–1.16) | 1.22× (1.19–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.38–1.48) | 2.70× (2.48–2.94) | 2.22× (2.13–2.31) | 2.84× (2.80–2.89) | 2.72× (2.56–2.90) | 2.24× (2.17–2.33) | 2.26× (2.20–2.32) | 0.88× (0.86–0.90) | 0.94× (0.91–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.47–1.53) | 3.68× (3.60–3.76) | 3.00× (2.94–3.06) | 3.15× (3.01–3.30) | 3.53× (3.43–3.62) | 4.42× (4.33–4.51) | 4.72× (4.63–4.81) | 1.63× (1.59–1.68) | 1.79× (1.72–1.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.28–1.43) | 3.22× (3.15–3.30) | 2.54× (2.49–2.58) | 3.53× (3.48–3.57) | 3.02× (2.79–3.26) | 3.66× (3.60–3.73) | 3.91× (3.85–3.98) | 1.35× (1.33–1.38) | 1.48× (1.44–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.48–1.59) | 2.94× (2.81–3.08) | 2.28× (2.06–2.52) | 3.08× (2.98–3.19) | 2.98× (2.69–3.29) | 2.59× (2.53–2.65) | 2.77× (2.71–2.83) | 0.96× (0.93–0.98) | 1.05× (1.01–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.54× (2.50–2.57) | 3.36× (3.30–3.42) | 3.02× (2.62–3.49) | 6.40× (5.90–6.93) | 2.14× (2.10–2.18) | 2.15× (2.10–2.19) | 0.99× (0.97–1.01) | 1.48× (1.44–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.29× (2.24–2.34) | 3.99× (3.85–4.13) | 2.71× (2.66–2.77) | 5.75× (5.59–5.91) | 2.00× (1.95–2.06) | 2.01× (1.97–2.06) | 0.93× (0.90–0.95) | 1.39× (1.35–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.51× (0.39–0.65) | 2.26× (1.75–2.92) | 3.67× (2.86–4.72) | 2.58× (1.99–3.34) | 5.44× (4.21–7.05) | 1.86× (1.44–2.40) | 1.87× (1.46–2.41) | 0.86× (0.68–1.10) | 1.29× (1.00–1.67) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 1.68× (1.58–1.79) | 3.94× (3.86–4.02) | 4.28× (4.10–4.48) | 4.49× (4.32–4.67) | 4.73× (4.53–4.95) | 4.78× (4.53–5.05) | 4.59× (4.42–4.76) | 1.49× (1.42–1.56) | 1.65× (1.50–1.83) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 1.42× (1.36–1.48) | 3.35× (2.97–3.77) | 3.51× (3.42–3.60) | 4.01× (3.93–4.09) | 3.95× (3.47–4.50) | 3.63× (3.49–3.77) | 3.48× (3.39–3.57) | 1.13× (1.10–1.17) | 1.25× (1.15–1.36) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 1.47× (1.28–1.68) | 2.86× (2.57–3.17) | 2.95× (2.82–3.09) | 3.23× (2.89–3.61) | 3.07× (2.77–3.39) | 2.29× (2.06–2.56) | 2.20× (1.96–2.46) | 0.72× (0.64–0.80) | 0.79× (0.68–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.98× (0.98–0.98) | 0.78× (0.77–0.80) | 11.07× (11.05–11.09) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.33) | 0.46× (0.40–0.52) | 0.67× (0.65–0.69) | 0.57× (0.55–0.59) | 5.32× (5.17–5.48) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.35) | 0.55× (0.46–0.65) | 0.61× (0.59–0.62) | 0.53× (0.51–0.55) | 3.51× (3.44–3.58) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.01–8.07) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.63–0.72) | 0.68× (0.64–0.72) | 1.59× (1.51–1.67) | 0.80× (0.76–0.84) | 5.76× (5.49–6.05) | 0.46× (0.44–0.49) | 0.49× (0.47–0.52) | 0.53× (0.51–0.56) | 0.26× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.59–0.66) | 0.67× (0.63–0.71) | 1.13× (1.09–1.17) | 0.69× (0.66–0.72) | 3.26× (3.14–3.38) | 0.25× (0.24–0.26) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.39–0.43) | 0.90× (0.87–0.93) | 0.72× (0.69–0.75) | 3.13× (3.02–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.62–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.90) | 0.84× (0.83–0.85) | 2.58× (2.54–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.82× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.40× (0.38–0.41) | 0.85× (0.82–0.89) | 0.70× (0.67–0.74) | 2.94× (2.82–3.07) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.47–0.51) | 0.80× (0.79–0.82) | 0.81× (0.78–0.84) | 2.32× (2.27–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.28× (5.26–5.30) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.42) | 0.66× (0.66–0.66) | 6.41× (6.40–6.43) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.39× (0.35–0.43) | 1.04× (1.00–1.08) | 0.67× (0.64–0.70) | 3.30× (3.19–3.41) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.31–0.34) | 0.16× (0.16–0.17) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 58 MB | 0.48× (0.47–0.50) | 0.45× (0.40–0.49) | 0.90× (0.87–0.92) | 0.72× (0.69–0.75) | 2.28× (2.22–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 633 | 1.59 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,040 | 1.77 | 1.12× (1.09–1.15) | 84% | 1.01× (0.98–1.05) | 1.06× (1.04–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,256 | 2.72 | 1.70× (1.64–1.76) | 50% | 1.30× (1.22–1.38) | 1.82× (1.75–1.89) |

**Best:** mibz2 at 4 threads (1,256 MB/s); lbzip2 at 4 threads (280 MB/s); pbz2 at 4 threads (431 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.25, lbzip2 1t 0.08, lbzip2 2t 0.27, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.52. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.42 | 0.01 | 0.07 | 0.167 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4260 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.3% |
| MTF + RLE2 | 1.44 | 19.9% |
| T-vector build | 1.33 | 18.4% |
| walk (the one each build uses) | 3.02 | 41.7% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.26 | 3.5% |
| **total** | 7.22 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.306 | 0.728 | (in its total) | 2.37× (2.36–2.39) |  |
| walk | 0.694 | 1.642 | (in its total) | 2.36× (2.32–2.39) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.032 | 0.032 | (in its total) | 0.99× (0.95–1.03) |  |
| inverse BWT, total | 1.032 | 2.402 | 1.354 | 2.32× (2.29–2.35) | 1.33× (1.28–1.39) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 48m35s ± 5m42s |
| 4 threads | 24m09s ± 3m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m30s ± 2m14s |
| the scanner alone (its serial part) | 8m53s ± 19.7s |
| the workers alone, over 4 | 15m30s ± 2m14s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.03 ns a byte of output at 4 threads (CPU, summed over them), 1.30 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,408,661 | 12,232,367 | \`2d4782182c8b0988…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4260.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
