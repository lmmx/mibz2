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

The latest record: [`5b8a3a8`](<https://github.com/lmmx/mibz2/commit/5b8a3a81810ccae9ff43538ea33e7c34ba3012ea>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44903080703>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.307 s | 682 | 6.2 | 2.78× (2.73–2.83) | 7.10× (6.92–7.28) | 7.46× (7.33–7.60) | 8.45× (8.23–8.68) | 7.44× (7.27–7.62) | 7.65× (7.55–7.74) | 7.26× (7.19–7.33) | 2.20× (2.16–2.25) | 2.55× (2.50–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.192 s | 1,094 | 7.1 | 2.69× (2.56–2.82) | 5.94× (5.67–6.22) | 7.35× (6.98–7.74) | 7.36× (7.06–7.67) | 6.39× (6.19–6.60) | 12.02× (11.47–12.60) | 11.41× (10.97–11.87) | 3.46× (3.30–3.63) | 4.01× (3.86–4.16) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.157 s | 1,339 | 11.0 | 2.95× (2.82–3.10) | 4.55× (4.36–4.74) | 6.14× (5.81–6.50) | 5.61× (5.36–5.88) | 5.69× (5.43–5.97) | 14.90× (14.16–15.68) | 14.14× (13.44–14.89) | 4.29× (4.07–4.52) | 4.97× (4.72–5.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,399 | 3.4 | 0.57× (0.55–0.58) | 3.54× (3.46–3.61) | 4.40× (4.30–4.50) | 3.68× (3.59–3.78) | 6.71× (6.26–7.19) | 2.79× (2.72–2.85) | 2.75× (2.69–2.82) | 1.01× (0.99–1.03) | 2.12× (2.08–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,329 | 3.6 | 0.50× (0.49–0.52) | 2.75× (2.68–2.83) | 5.62× (5.47–5.77) | 3.83× (3.31–4.44) | 6.20× (6.05–6.35) | 2.70× (2.66–2.74) | 2.67× (2.64–2.70) | 0.98× (0.96–0.99) | 2.06× (2.01–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,316 | 3.5 | 0.50× (0.49–0.51) | 2.82× (2.64–3.01) | 5.60× (5.36–5.84) | 3.52× (3.47–3.57) | 6.16× (6.06–6.26) | 2.64× (2.59–2.68) | 2.61× (2.56–2.66) | 0.96× (0.92–0.99) | 2.01× (1.99–2.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,065 | 4.3 | 2.84× (2.80–2.89) | 3.69× (3.49–3.90) | 4.63× (4.18–5.14) | 4.69× (4.56–4.81) | 5.48× (5.35–5.62) | 3.12× (3.05–3.19) | 2.70× (2.65–2.75) | 2.19× (2.15–2.23) | 3.31× (3.27–3.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,479 | 5.9 | 2.30× (2.13–2.48) | 3.30× (2.86–3.81) | 4.52× (4.21–4.87) | 3.73× (3.50–3.97) | 5.67× (5.30–6.06) | 4.23× (3.91–4.57) | 3.67× (3.46–3.88) | 2.96× (2.81–3.12) | 4.48× (4.21–4.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,150 | 12.4 | 1.71× (1.58–1.85) | 1.86× (1.73–2.00) | 2.65× (2.47–2.85) | 2.01× (1.88–2.15) | 4.09× (3.78–4.43) | 3.33× (3.05–3.64) | 2.89× (2.69–3.11) | 2.34× (2.17–2.52) | 3.53× (3.26–3.83) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.080 s | 100 | 43.3 | 1.40× (1.36–1.44) | 3.10× (3.02–3.17) | 2.77× (2.69–2.86) | 2.74× (2.65–2.83) | 3.10× (3.00–3.21) | 3.44× (3.32–3.56) | 3.41× (3.30–3.52) | 1.36× (1.31–1.40) | 1.56× (1.47–1.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 148 | 51.3 | 1.21× (1.13–1.30) | 2.80× (2.62–2.99) | 2.55× (2.37–2.75) | 3.10× (2.85–3.39) | 2.92× (2.74–3.12) | 5.07× (4.80–5.35) | 5.03× (4.76–5.30) | 2.00× (1.88–2.13) | 2.30× (2.20–2.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 183 | 70.1 | 1.23× (1.09–1.38) | 2.33× (2.10–2.59) | 2.31× (2.10–2.54) | 2.70× (2.43–3.01) | 2.98× (2.64–3.36) | 6.14× (5.42–6.96) | 6.09× (5.38–6.88) | 2.42× (2.14–2.74) | 2.78× (2.42–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 39.1 | 1.49× (1.44–1.53) | 3.65× (3.54–3.76) | 2.98× (2.88–3.09) | 3.08× (2.91–3.26) | 3.42× (3.30–3.55) | 4.32× (4.16–4.48) | 4.54× (4.43–4.65) | 1.58× (1.53–1.63) | 1.81× (1.77–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 172 | 47.4 | 1.37× (1.26–1.50) | 3.26× (3.00–3.55) | 2.78× (2.64–2.92) | 3.77× (3.64–3.90) | 3.31× (3.20–3.42) | 6.72× (6.44–7.00) | 7.05× (6.76–7.36) | 2.46× (2.37–2.55) | 2.82× (2.71–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 204 | 62.5 | 1.40× (1.34–1.47) | 2.61× (2.42–2.81) | 2.73× (2.36–3.17) | 3.30× (3.14–3.46) | 3.21× (2.95–3.50) | 7.88× (7.58–8.19) | 8.27× (7.99–8.56) | 2.88× (2.75–3.03) | 3.30× (3.13–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 4.8 | 0.61× (0.60–0.62) | 3.26× (3.05–3.49) | 4.39× (4.29–4.49) | 3.07× (3.00–3.14) | 7.09× (6.87–7.31) | 2.28× (2.24–2.33) | 2.31× (2.25–2.37) | 1.07× (0.86–1.33) | 2.12× (2.07–2.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,000 | 5.1 | 0.50× (0.49–0.51) | 3.07× (3.03–3.10) | 5.29× (5.22–5.37) | 3.10× (2.90–3.31) | 6.68× (6.56–6.80) | 2.21× (2.15–2.26) | 2.23× (2.17–2.29) | 1.03× (0.83–1.29) | 2.05× (2.02–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 962 | 5.1 | 0.49× (0.47–0.51) | 2.95× (2.86–3.04) | 5.07× (4.91–5.24) | 2.91× (2.81–3.02) | 6.69× (6.00–7.47) | 2.11× (2.04–2.19) | 2.14× (2.06–2.22) | 0.99× (0.79–1.24) | 1.96× (1.90–2.02) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.046 s | 293 | 15.0 | 1.84× (1.65–2.07) | 4.27× (4.11–4.43) | 4.58× (4.46–4.70) | 4.64× (4.55–4.74) | 4.98× (4.78–5.19) | 5.06× (4.91–5.22) | 4.69× (4.60–4.78) | 1.49× (1.45–1.52) | 1.77× (1.73–1.81) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.034 s | 394 | 20.0 | 1.41× (1.34–1.48) | 3.26× (3.13–3.40) | 3.93× (3.80–4.07) | 4.44× (4.02–4.90) | 4.27× (4.15–4.40) | 6.93× (6.66–7.22) | 6.42× (6.21–6.64) | 2.04× (1.97–2.10) | 2.42× (2.36–2.49) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.031 s | 435 | 29.8 | 1.34× (1.30–1.38) | 2.62× (2.32–2.95) | 3.36× (3.08–3.67) | 3.44× (3.17–3.73) | 4.08× (3.63–4.58) | 7.62× (7.30–7.96) | 7.06× (6.74–7.41) | 2.24× (2.16–2.32) | 2.67× (2.55–2.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.77× (2.73–2.82) | 7.05× (6.87–7.23) | 7.41× (7.28–7.55) | 8.41× (8.19–8.63) | 7.41× (7.24–7.58) | 7.61× (7.52–7.70) | 7.22× (7.16–7.29) | 2.20× (2.15–2.24) | 2.53× (2.49–2.57) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.60× (2.54–2.67) | 6.32× (6.23–6.41) | 6.99× (6.66–7.34) | 7.92× (7.74–8.10) | 6.59× (6.47–6.71) | 6.61× (6.44–6.79) | 6.28× (6.12–6.44) | 1.91× (1.85–1.97) | 2.20× (2.14–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.84× (2.74–2.95) | 5.12× (4.92–5.34) | 5.90× (5.50–6.32) | 6.31× (6.01–6.64) | 5.63× (5.35–5.92) | 4.32× (4.13–4.51) | 4.10× (3.92–4.28) | 1.25× (1.19–1.30) | 1.43× (1.38–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.04× (2.95–3.13) | 3.94× (3.81–4.08) | 3.40× (3.33–3.48) | 6.12× (5.72–6.55) | 2.60× (2.55–2.65) | 2.57× (2.52–2.63) | 1.01× (0.98–1.03) | 1.81× (1.77–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.58) | 2.77× (2.71–2.84) | 4.87× (4.76–4.99) | 3.55× (3.04–4.14) | 5.60× (5.46–5.74) | 2.49× (2.45–2.53) | 2.46× (2.43–2.49) | 0.96× (0.95–0.98) | 1.74× (1.71–1.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.56) | 2.98× (2.94–3.02) | 5.00× (4.73–5.29) | 3.32× (3.27–3.36) | 5.67× (5.58–5.77) | 2.49× (2.45–2.54) | 2.47× (2.42–2.52) | 0.97× (0.93–1.00) | 1.74× (1.70–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.66–2.74) | 3.19× (3.12–3.27) | 4.03× (3.62–4.48) | 4.39× (4.27–4.51) | 5.12× (5.00–5.26) | 2.95× (2.89–3.02) | 2.57× (2.53–2.61) | 2.10× (2.06–2.13) | 2.79× (2.75–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.18–2.31) | 2.73× (2.44–3.05) | 3.17× (3.04–3.30) | 3.44× (3.33–3.55) | 3.92× (3.79–4.05) | 2.20× (2.12–2.29) | 1.91× (1.86–1.97) | 1.56× (1.52–1.60) | 2.08× (2.02–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.79× (1.69–1.89) | 1.63× (1.56–1.70) | 1.93× (1.86–2.00) | 1.93× (1.85–2.01) | 2.11× (2.02–2.19) | 1.03× (0.97–1.09) | 0.90× (0.86–0.94) | 0.73× (0.70–0.76) | 0.97× (0.93–1.02) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.35–1.43) | 2.99× (2.93–3.06) | 2.67× (2.60–2.75) | 2.70× (2.62–2.79) | 3.06× (2.96–3.16) | 3.39× (3.28–3.51) | 3.36× (3.26–3.47) | 1.35× (1.31–1.40) | 1.48× (1.41–1.56) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.74× (2.66–2.81) | 2.38× (2.29–2.48) | 3.09× (2.95–3.24) | 2.69× (2.62–2.76) | 2.90× (2.81–3.00) | 2.88× (2.79–2.97) | 1.16× (1.12–1.20) | 1.27× (1.21–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.35× (1.22–1.48) | 2.50× (2.33–2.68) | 2.03× (1.87–2.21) | 2.72× (2.47–2.99) | 2.54× (2.30–2.80) | 2.07× (1.85–2.31) | 2.05× (1.84–2.29) | 0.82× (0.74–0.92) | 0.90× (0.80–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.43–1.52) | 3.55× (3.46–3.65) | 2.89× (2.81–2.98) | 3.04× (2.87–3.21) | 3.37× (3.26–3.50) | 4.26× (4.10–4.42) | 4.47× (4.36–4.58) | 1.57× (1.52–1.62) | 1.75× (1.72–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.29–1.37) | 3.11× (3.04–3.18) | 2.50× (2.42–2.59) | 3.49× (3.37–3.61) | 2.91× (2.83–3.00) | 3.56× (3.44–3.69) | 3.74× (3.61–3.87) | 1.31× (1.27–1.36) | 1.46× (1.42–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.50–1.68) | 3.02× (2.79–3.27) | 2.47× (2.05–2.98) | 3.18× (3.04–3.33) | 3.01× (2.69–3.36) | 2.66× (2.56–2.76) | 2.79× (2.69–2.90) | 0.98× (0.95–1.02) | 1.09× (1.05–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.55× (2.41–2.70) | 3.34× (3.29–3.40) | 2.77× (2.71–2.83) | 6.22× (6.03–6.42) | 2.10× (2.06–2.15) | 2.12× (2.07–2.18) | 1.08× (0.87–1.33) | 1.49× (1.46–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.30× (2.27–2.34) | 4.04× (3.95–4.13) | 2.82× (2.62–3.04) | 5.76× (5.66–5.87) | 1.99× (1.94–2.04) | 2.01× (1.96–2.06) | 1.02× (0.81–1.27) | 1.41× (1.39–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.53–0.57) | 2.45× (2.39–2.50) | 3.92× (3.80–4.04) | 2.74× (2.65–2.83) | 5.99× (5.35–6.70) | 1.97× (1.92–2.03) | 1.99× (1.93–2.06) | 1.01× (0.80–1.27) | 1.40× (1.36–1.45) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.82× (1.62–2.03) | 4.05× (3.91–4.21) | 4.36× (4.27–4.44) | 4.51× (4.42–4.61) | 4.84× (4.64–5.04) | 4.93× (4.77–5.10) | 4.56× (4.48–4.64) | 1.47× (1.44–1.50) | 1.63× (1.61–1.66) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.43× (1.41–1.44) | 3.23× (3.16–3.29) | 3.56× (3.42–3.71) | 4.07× (3.88–4.27) | 3.70× (3.57–3.82) | 3.75× (3.61–3.90) | 3.47× (3.37–3.57) | 1.12× (1.09–1.15) | 1.24× (1.22–1.27) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.51× (1.46–1.57) | 2.85× (2.55–3.18) | 3.13× (2.91–3.36) | 3.44× (3.24–3.66) | 3.40× (2.93–3.94) | 2.53× (2.39–2.69) | 2.34× (2.22–2.48) | 0.75× (0.72–0.79) | 0.84× (0.80–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.76–0.80) | 10.97× (10.94–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.33) | 0.49× (0.43–0.56) | 0.66× (0.64–0.68) | 0.56× (0.54–0.58) | 5.27× (5.14–5.41) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.33–0.35) | 0.53× (0.44–0.63) | 0.60× (0.58–0.62) | 0.53× (0.50–0.55) | 3.50× (3.39–3.61) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.21–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.43–0.46) | 1.69× (1.68–1.70) | 0.66× (0.66–0.67) | 7.97× (7.94–8.00) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.67–0.72) | 0.66× (0.61–0.71) | 1.62× (1.58–1.66) | 0.81× (0.79–0.84) | 5.86× (5.72–6.01) | 0.48× (0.47–0.49) | 0.50× (0.49–0.52) | 0.54× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.63–0.69) | 0.73× (0.70–0.76) | 1.15× (1.10–1.20) | 0.70× (0.67–0.74) | 3.36× (3.23–3.49) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.13–5.16) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.44) | 0.90× (0.87–0.93) | 0.71× (0.69–0.74) | 3.13× (3.03–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.60–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.81× (0.79–0.83) | 2.51× (2.45–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.49) | 0.39× (0.36–0.41) | 0.86× (0.83–0.88) | 0.72× (0.70–0.75) | 2.95× (2.87–3.03) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.46× (0.44–0.49) | 0.81× (0.79–0.83) | 0.84× (0.81–0.86) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.23× (0.22–0.24) | 1.06× (1.06–1.07) | 0.77× (0.76–0.77) | 5.33× (5.32–5.35) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.24–5.27) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.68× (0.67–0.68) | 6.38× (6.36–6.41) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.41× (0.37–0.45) | 1.04× (1.01–1.07) | 0.67× (0.65–0.70) | 3.33× (3.26–3.41) | 0.20× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.50× (0.48–0.52) | 0.44× (0.41–0.48) | 0.90× (0.86–0.94) | 0.74× (0.71–0.77) | 2.29× (2.20–2.39) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 682 | 1.47 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,094 | 1.66 | 1.15× (1.12–1.18) | 79% | 1.03× (1.01–1.05) | 1.08× (1.04–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 1,339 | 2.58 | 1.76× (1.68–1.85) | 49% | 1.28× (1.22–1.35) | 1.81× (1.71–1.91) |

**Best:** mibz2 at 4 threads (1,339 MB/s); lbzip2 at 4 threads (300 MB/s); pbz2 at 4 threads (469 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.24, lbzip2 1t 0.08, lbzip2 2t 0.33, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.40 | 0.01 | 0.07 | 0.157 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 42.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.0% |
| MTF + RLE2 | 1.45 | 19.7% |
| T-vector build | 1.34 | 18.2% |
| walk (the one each build uses) | 3.13 | 42.6% |
| of which: segment assembly | 0.15 | 2.1% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.25 | 3.5% |
| **total** | 7.34 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.303 | 0.719 | (in its total) | 2.36× (2.34–2.38) |  |
| walk | 0.685 | 1.614 | (in its total) | 2.34× (2.31–2.37) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.033 | 0.035 | (in its total) | 1.08× (1.00–1.15) |  |
| inverse BWT, total | 1.020 | 2.368 | 1.342 | 2.31× (2.28–2.34) | 1.31× (1.29–1.33) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m27s ± 5m04s |
| 4 threads | 22m51s ± 3m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m29s ± 2m12s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 14m29s ± 2m12s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 1.90 ns a byte of output at 4 threads (CPU, summed over them), 1.18 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,633,100 | 13,390,258 | \`b7ebdafdab81f6ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4243.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
