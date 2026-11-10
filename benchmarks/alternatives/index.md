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

The latest record: [`8f755d0`](<https://github.com/lmmx/mibz2/commit/8f755d070a14696cf84ea65449ca66e9921a645e>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42786994906>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.341 s | 615 | 4.2 | 2.40× (2.36–2.44) | 6.03× (5.96–6.11) | 6.29× (6.20–6.38) | 7.21× (7.10–7.31) | 6.36× (6.26–6.46) | 6.59× (6.46–6.72) | 6.22× (6.11–6.33) | 1.89× (1.86–1.91) | 2.16× (2.12–2.20) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.200 s | 1,048 | 4.6 | 2.39× (2.19–2.61) | 5.27× (4.84–5.73) | 6.34× (5.79–6.94) | 6.35× (5.79–6.96) | 5.64× (5.19–6.14) | 10.89× (9.92–11.95) | 10.28× (9.34–11.32) | 3.12× (2.83–3.43) | 3.57× (3.25–3.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.177 s | 1,185 | 7.9 | 2.13× (2.01–2.26) | 3.50× (3.28–3.73) | 4.51× (4.25–4.79) | 4.25× (4.00–4.51) | 4.30× (4.08–4.53) | 12.40× (11.69–13.16) | 11.70× (11.01–12.45) | 3.55× (3.34–3.78) | 4.06× (3.82–4.32) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,471 | 1.9 | 0.62× (0.51–0.76) | 3.73× (3.64–3.82) | 4.62× (4.52–4.73) | 3.71× (3.62–3.81) | 6.59× (6.42–6.78) | 2.84× (2.76–2.92) | 2.91× (2.76–3.06) | 1.01× (0.98–1.05) | 2.24× (2.19–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,408 | 2.1 | 0.51× (0.49–0.53) | 2.83× (2.75–2.92) | 5.79× (5.61–5.98) | 3.54× (3.43–3.66) | 6.22× (6.05–6.39) | 2.68× (2.60–2.77) | 2.75× (2.57–2.95) | 0.96× (0.93–0.99) | 2.12× (2.05–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,342 | 2.1 | 0.46× (0.37–0.56) | 2.54× (2.07–3.12) | 5.16× (4.21–6.33) | 3.18× (2.59–3.90) | 5.63× (4.57–6.92) | 2.40× (1.97–2.91) | 2.46× (1.98–3.05) | 0.86× (0.71–1.04) | 1.89× (1.54–2.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,040 | 2.7 | 2.79× (2.71–2.87) | 3.39× (3.24–3.55) | 4.03× (3.87–4.20) | 4.40× (4.31–4.48) | 5.11× (5.01–5.22) | 2.87× (2.79–2.94) | 2.46× (2.42–2.50) | 2.11× (2.01–2.22) | 3.23× (3.17–3.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,395 | 3.6 | 2.05× (1.90–2.20) | 2.90× (2.70–3.11) | 3.92× (3.56–4.32) | 3.51× (3.26–3.79) | 5.23× (4.84–5.65) | 3.83× (3.55–4.13) | 3.28× (3.06–3.52) | 2.82× (2.58–3.08) | 4.31× (4.02–4.63) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,248 | 6.9 | 1.64× (1.53–1.76) | 1.95× (1.82–2.09) | 2.77× (2.62–2.93) | 2.02× (1.86–2.20) | 4.26× (3.91–4.64) | 3.35× (3.15–3.57) | 2.87× (2.71–3.05) | 2.47× (2.29–2.67) | 3.77× (3.53–4.03) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 104 | 25.3 | 1.39× (1.32–1.45) | 3.05× (2.90–3.21) | 2.78× (2.61–2.95) | 2.74× (2.59–2.91) | 3.04× (2.88–3.21) | 3.42× (3.26–3.59) | 3.45× (3.27–3.65) | 1.32× (1.26–1.39) | 1.53× (1.46–1.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.048 s | 166 | 29.0 | 1.31× (1.27–1.35) | 2.89× (2.78–3.00) | 2.77× (2.68–2.86) | 3.51× (3.37–3.66) | 3.22× (3.14–3.30) | 5.60× (5.48–5.73) | 5.65× (5.54–5.77) | 2.17× (2.12–2.21) | 2.51× (2.46–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 196 | 39.2 | 1.27× (1.23–1.30) | 2.48× (2.37–2.58) | 2.34× (2.26–2.43) | 2.89× (2.75–3.04) | 3.12× (3.02–3.22) | 6.58× (6.40–6.75) | 6.63× (6.47–6.80) | 2.54× (2.48–2.61) | 2.95× (2.86–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.081 s | 118 | 22.3 | 1.48× (1.41–1.55) | 3.70× (3.51–3.89) | 3.06× (2.95–3.18) | 3.15× (2.97–3.34) | 3.47× (3.31–3.64) | 4.41× (4.18–4.65) | 4.69× (4.45–4.93) | 1.59× (1.51–1.68) | 1.85× (1.74–1.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.051 s | 187 | 26.1 | 1.33× (1.28–1.38) | 3.40× (3.27–3.53) | 2.83× (2.71–2.97) | 3.88× (3.75–4.02) | 3.37× (3.26–3.48) | 7.04× (6.80–7.28) | 7.48× (7.22–7.75) | 2.54× (2.45–2.64) | 2.95× (2.82–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 219 | 36.6 | 1.36× (1.27–1.45) | 2.67× (2.46–2.89) | 2.53× (2.39–2.67) | 3.36× (3.17–3.55) | 3.14× (2.96–3.33) | 8.20× (7.65–8.78) | 8.71× (8.17–9.30) | 2.96× (2.78–3.16) | 3.44× (3.21–3.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,105 | 2.7 | 0.62× (0.61–0.63) | 3.39× (3.33–3.44) | 4.73× (4.66–4.81) | 3.25× (3.10–3.40) | 7.15× (7.02–7.29) | 2.38× (2.35–2.40) | 2.38× (2.34–2.43) | 0.97× (0.95–1.00) | 2.26× (2.23–2.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,026 | 3.0 | 0.51× (0.49–0.52) | 3.17× (3.09–3.25) | 5.26× (4.91–5.64) | 3.02× (2.93–3.12) | 6.76× (6.58–6.94) | 2.22× (2.18–2.27) | 2.23× (2.17–2.29) | 0.91× (0.89–0.94) | 2.12× (2.07–2.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,000 | 3.0 | 0.49× (0.48–0.50) | 3.07× (2.98–3.15) | 5.23× (5.03–5.43) | 2.96× (2.88–3.03) | 6.52× (6.31–6.73) | 2.14× (2.08–2.21) | 2.15× (2.09–2.21) | 0.88× (0.85–0.91) | 2.04× (1.99–2.10) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.054 s | 231 | 11.5 | 1.30× (1.29–1.31) | 3.27× (3.21–3.33) | 3.45× (3.37–3.52) | 3.57× (3.51–3.64) | 3.77× (3.70–3.84) | 3.81× (3.76–3.87) | 3.65× (3.58–3.72) | 1.14× (1.12–1.17) | 1.31× (1.29–1.33) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.036 s | 341 | 14.3 | 1.10× (1.03–1.17) | 2.77× (2.59–2.95) | 3.17× (3.04–3.30) | 3.74× (3.49–4.01) | 3.56× (3.46–3.67) | 5.66× (5.50–5.82) | 5.42× (5.22–5.63) | 1.70× (1.64–1.75) | 1.95× (1.90–2.00) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.031 s | 402 | 20.4 | 1.10× (1.04–1.15) | 2.23× (1.97–2.52) | 2.64× (2.49–2.80) | 3.05× (2.89–3.20) | 3.42× (3.19–3.66) | 6.74× (6.56–6.93) | 6.46× (6.25–6.67) | 2.02× (1.95–2.09) | 2.32× (2.26–2.39) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.40× (2.36–2.44) | 6.00× (5.92–6.07) | 6.25× (6.16–6.34) | 7.18× (7.08–7.29) | 6.34× (6.24–6.44) | 6.56× (6.43–6.70) | 6.20× (6.09–6.30) | 1.88× (1.86–1.91) | 2.14× (2.11–2.17) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.29× (2.26–2.31) | 5.67× (5.61–5.72) | 6.01× (5.94–6.09) | 6.94× (6.86–7.02) | 5.88× (5.74–6.01) | 6.05× (5.97–6.14) | 5.71× (5.64–5.78) | 1.74× (1.71–1.76) | 1.97× (1.95–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.99× (1.90–2.08) | 3.77× (3.57–3.99) | 4.10× (3.91–4.30) | 4.61× (4.37–4.87) | 4.03× (3.84–4.23) | 3.44× (3.28–3.62) | 3.25× (3.09–3.42) | 0.99× (0.94–1.04) | 1.12× (1.07–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.57–0.80) | 3.13× (3.05–3.21) | 3.93× (3.83–4.02) | 3.49× (3.41–3.57) | 6.13× (5.98–6.29) | 2.70× (2.63–2.77) | 2.77× (2.60–2.94) | 1.02× (0.98–1.05) | 1.88× (1.84–1.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.76× (2.68–2.85) | 4.84× (4.72–4.96) | 3.28× (3.19–3.37) | 5.65× (5.52–5.79) | 2.49× (2.42–2.56) | 2.55× (2.36–2.75) | 0.94× (0.91–0.97) | 1.74× (1.69–1.79) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.50× (0.41–0.61) | 2.74× (2.25–3.32) | 4.52× (3.72–5.50) | 3.03× (2.50–3.67) | 5.25× (4.32–6.39) | 2.30× (1.91–2.76) | 2.35× (1.91–2.90) | 0.86× (0.72–1.03) | 1.60× (1.33–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.62–2.75) | 3.07× (3.02–3.11) | 3.62× (3.57–3.67) | 4.20× (4.13–4.27) | 4.86× (4.77–4.96) | 2.77× (2.70–2.84) | 2.38× (2.34–2.41) | 2.06× (1.94–2.17) | 2.69× (2.64–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.08–2.20) | 2.51× (2.42–2.61) | 2.95× (2.84–3.06) | 3.30× (3.18–3.43) | 3.74× (3.60–3.88) | 2.06× (2.03–2.10) | 1.77× (1.72–1.82) | 1.53× (1.44–1.63) | 2.01× (1.95–2.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.81× (1.71–1.92) | 1.66× (1.59–1.74) | 1.97× (1.88–2.07) | 1.95× (1.86–2.05) | 2.25× (2.06–2.47) | 1.06× (1.00–1.11) | 0.91× (0.86–0.95) | 0.79× (0.73–0.84) | 1.03× (0.98–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.31–1.45) | 2.96× (2.81–3.12) | 2.66× (2.51–2.82) | 2.71× (2.55–2.88) | 3.01× (2.84–3.18) | 3.38× (3.21–3.56) | 3.41× (3.22–3.61) | 1.32× (1.25–1.39) | 1.43× (1.35–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.80× (2.75–2.84) | 2.44× (2.40–2.47) | 3.25× (3.22–3.28) | 2.81× (2.74–2.88) | 3.04× (2.99–3.09) | 3.06× (3.03–3.09) | 1.18× (1.17–1.20) | 1.29× (1.28–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.38–1.43) | 2.58× (2.55–2.62) | 2.14× (2.11–2.18) | 2.84× (2.79–2.89) | 2.68× (2.65–2.70) | 2.25× (2.22–2.27) | 2.27× (2.23–2.30) | 0.88× (0.86–0.89) | 0.95× (0.94–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.47× (1.40–1.55) | 3.59× (3.40–3.79) | 2.96× (2.85–3.08) | 3.11× (2.93–3.31) | 3.43× (3.27–3.61) | 4.35× (4.12–4.60) | 4.63× (4.38–4.88) | 1.58× (1.50–1.67) | 1.78× (1.67–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.38) | 3.29× (3.25–3.33) | 2.63× (2.60–2.67) | 3.70× (3.65–3.74) | 3.04× (3.00–3.08) | 3.81× (3.72–3.90) | 4.05× (3.98–4.12) | 1.38× (1.36–1.41) | 1.55× (1.51–1.60) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.50–1.56) | 2.93× (2.83–3.02) | 2.20× (2.14–2.25) | 3.15× (3.09–3.22) | 2.81× (2.75–2.88) | 2.71× (2.62–2.80) | 2.88× (2.81–2.96) | 0.99× (0.96–1.01) | 1.11× (1.07–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.69) | 2.60× (2.54–2.66) | 3.42× (3.36–3.48) | 2.97× (2.82–3.13) | 6.40× (6.27–6.53) | 2.23× (2.21–2.26) | 2.22× (2.18–2.27) | 0.98× (0.96–1.01) | 1.54× (1.51–1.57) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.59) | 2.30× (2.25–2.35) | 3.96× (3.86–4.06) | 2.75× (2.68–2.83) | 5.86× (5.72–5.99) | 2.02× (1.98–2.05) | 2.01× (1.96–2.05) | 0.89× (0.86–0.91) | 1.39× (1.37–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.57) | 2.47× (2.40–2.55) | 3.96× (3.87–4.05) | 2.80× (2.73–2.87) | 5.89× (5.71–6.08) | 2.03× (1.97–2.09) | 2.02× (1.96–2.08) | 0.89× (0.86–0.92) | 1.40× (1.37–1.43) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.30× (1.29–1.31) | 3.11× (3.06–3.15) | 3.30× (3.24–3.37) | 3.52× (3.45–3.58) | 3.70× (3.63–3.77) | 3.75× (3.69–3.81) | 3.59× (3.52–3.67) | 1.14× (1.11–1.17) | 1.24× (1.22–1.26) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.13× (1.12–1.14) | 2.70× (2.54–2.87) | 2.88× (2.84–2.91) | 3.33× (3.23–3.45) | 3.15× (3.08–3.22) | 3.06× (3.02–3.10) | 2.93× (2.87–2.99) | 0.93× (0.91–0.94) | 1.01× (1.00–1.02) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.24× (1.20–1.27) | 2.33× (2.12–2.56) | 2.45× (2.35–2.56) | 2.75× (2.67–2.83) | 2.72× (2.59–2.85) | 2.14× (2.08–2.20) | 2.05× (1.98–2.13) | 0.65× (0.62–0.68) | 0.71× (0.68–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.37× (0.36–0.38) | 1.07× (1.07–1.07) | 0.89× (0.88–0.89) | 12.09× (12.07–12.10) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.49× (0.40–0.60) | 0.79× (0.78–0.81) | 0.67× (0.65–0.68) | 6.32× (6.23–6.42) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.57× (0.48–0.68) | 0.69× (0.69–0.70) | 0.60× (0.59–0.62) | 4.06× (4.04–4.08) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.67× (0.66–0.68) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.63–0.69) | 0.68× (0.63–0.74) | 1.57× (1.51–1.63) | 0.78× (0.75–0.81) | 5.66× (5.46–5.87) | 0.46× (0.44–0.48) | 0.48× (0.46–0.50) | 0.52× (0.50–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.71× (0.65–0.78) | 1.14× (1.09–1.19) | 0.69× (0.66–0.72) | 3.31× (3.16–3.46) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.22× (5.21–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.53) | 0.40× (0.37–0.43) | 0.90× (0.86–0.94) | 0.72× (0.69–0.75) | 3.12× (2.99–3.26) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.46× (0.45–0.47) | 0.89× (0.87–0.90) | 0.84× (0.82–0.86) | 2.59× (2.54–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.23–5.24) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.39× (0.37–0.42) | 0.87× (0.83–0.91) | 0.72× (0.69–0.76) | 2.98× (2.85–3.11) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.45–0.50) | 0.80× (0.79–0.81) | 0.82× (0.80–0.85) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.41× (5.40–5.42) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.62) | 0.77× (0.77–0.78) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.47–1.47) | 0.70× (0.69–0.70) | 6.64× (6.63–6.66) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.65) | 0.32× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.39× (0.36–0.43) | 1.07× (1.03–1.12) | 0.67× (0.65–0.68) | 3.34× (3.24–3.46) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.50–0.53) | 0.45× (0.41–0.49) | 0.92× (0.90–0.95) | 0.75× (0.73–0.77) | 2.37× (2.30–2.43) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 615 | 1.63 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,048 | 1.78 | 1.08× (1.06–1.11) | 83% | 1.03× (1.01–1.04) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,185 | 3.04 | 1.91× (1.81–2.01) | 47% | 1.20× (1.18–1.22) | 1.58× (1.56–1.60) |

**Best:** mibz2 at 4 threads (1,185 MB/s); lbzip2 at 4 threads (330 MB/s); pbz2 at 4 threads (539 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.88, mibz2 4t 1.17, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.48, pbz2 4t 0.59. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.49 | 0.01 | 0.08 | 0.177 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.58 | 14.2% |
| MTF + RLE2 | 0.90 | 22.0% |
| T-vector build | 0.76 | 18.5% |
| walk (the one each build uses) | 1.61 | 39.5% |
| of which: segment assembly | 0.06 | 1.6% |
| RLE1 | 0.09 | 2.3% |
| CRC | 0.14 | 3.6% |
| **total** | 4.09 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.283 | 0.680 | (in its total) | 2.39× (2.37–2.41) |  |
| walk | 0.609 | 1.538 | (in its total) | 2.53× (2.52–2.54) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.028 | (in its total) | 1.13× (1.12–1.14) |  |
| inverse BWT, total | 0.917 | 2.249 | 1.298 | 2.45× (2.43–2.46) | 1.40× (1.38–1.43) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 49m59s ± 5m48s |
| 4 threads | 26m38s ± 4m06s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m24s ± 2m48s |
| the scanner alone (its serial part) | 9m01s ± 39.5s |
| the workers alone, over 4 | 18m24s ± 2m48s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 2.41 ns a byte of output at 4 threads (CPU, summed over them), 1.34 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
