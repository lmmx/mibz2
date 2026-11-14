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

The latest record: [`058660c`](<https://github.com/lmmx/mibz2/commit/058660c2690219084b1bcf9e27ac6d66109e4971>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43604254326>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.413 s | 508 | 7.7 | 3.28× (3.25–3.31) | 7.45× (7.36–7.54) | 7.93× (7.84–8.03) | 8.78× (8.66–8.90) | 7.94× (7.85–8.02) | 8.31× (8.21–8.42) | 8.10× (8.01–8.19) | 2.56× (2.53–2.59) | 2.27× (2.24–2.30) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.235 s | 894 | 8.5 | 3.13× (2.93–3.34) | 6.69× (6.28–7.13) | 8.11× (7.55–8.72) | 8.00× (7.47–8.55) | 7.29× (6.81–7.80) | 14.20× (13.26–15.20) | 13.84× (12.92–14.81) | 4.37× (4.08–4.67) | 3.87× (3.62–4.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.199 s | 1,055 | 13.2 | 3.24× (3.18–3.30) | 5.94× (5.82–6.06) | 7.01× (6.87–7.14) | 6.90× (6.76–7.03) | 6.76× (6.65–6.88) | 17.53× (17.19–17.87) | 17.08× (16.78–17.38) | 5.39× (5.29–5.49) | 4.78× (4.70–4.86) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,294 | 3.3 | 0.67× (0.63–0.72) | 3.50× (3.20–3.83) | 5.02× (4.59–5.50) | 4.04× (3.72–4.39) | 6.94× (6.37–7.58) | 3.11× (2.84–3.41) | 3.14× (2.87–3.43) | 1.12× (1.03–1.21) | 1.41× (1.29–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,370 | 3.1 | 0.62× (0.58–0.67) | 3.48× (3.24–3.73) | 6.84× (6.48–7.22) | 4.05× (3.80–4.32) | 6.80× (6.40–7.22) | 3.09× (2.88–3.31) | 3.11× (2.90–3.34) | 1.11× (1.03–1.20) | 1.40× (1.31–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,235 | 3.4 | 0.59× (0.56–0.63) | 3.26× (3.08–3.45) | 6.33× (5.91–6.77) | 3.84× (3.65–4.04) | 6.49× (6.18–6.81) | 2.89× (2.73–3.07) | 2.92× (2.75–3.09) | 1.04× (0.95–1.14) | 1.31× (1.24–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 677 | 6.0 | 3.01× (2.90–3.13) | 3.49× (3.35–3.63) | 4.00× (3.90–4.10) | 4.46× (4.29–4.64) | 4.89× (4.73–5.05) | 3.30× (3.18–3.42) | 3.05× (2.94–3.17) | 2.04× (1.96–2.11) | 2.09× (2.01–2.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,057 | 7.2 | 2.63× (2.46–2.81) | 3.34× (3.13–3.55) | 4.49× (4.19–4.81) | 4.06× (3.78–4.37) | 5.44× (5.17–5.72) | 5.23× (4.94–5.55) | 4.84× (4.56–5.14) | 3.23× (3.03–3.45) | 3.32× (3.12–3.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,104 | 12.4 | 2.18× (2.02–2.36) | 2.33× (2.15–2.53) | 3.31× (3.09–3.54) | 2.75× (2.52–2.99) | 4.56× (4.25–4.90) | 5.35× (4.97–5.77) | 4.95× (4.60–5.34) | 3.31× (3.05–3.59) | 3.40× (3.14–3.68) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 55.2 | 1.38× (1.37–1.39) | 3.04× (3.01–3.07) | 2.75× (2.72–2.78) | 2.84× (2.82–2.86) | 3.14× (3.12–3.16) | 3.45× (3.43–3.48) | 3.60× (3.56–3.63) | 1.25× (1.25–1.26) | 1.26× (1.26–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 60.1 | 1.28× (1.19–1.38) | 2.80× (2.63–2.98) | 2.78× (2.59–2.98) | 3.34× (3.10–3.60) | 3.14× (2.93–3.37) | 5.59× (5.20–6.01) | 5.82× (5.42–6.26) | 2.03× (1.89–2.18) | 2.05× (1.90–2.21) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.054 s | 147 | 81.5 | 1.38× (1.33–1.43) | 2.83× (2.70–2.96) | 2.70× (2.58–2.82) | 3.21× (3.11–3.32) | 3.27× (3.17–3.37) | 7.02× (6.80–7.25) | 7.31× (7.07–7.57) | 2.55× (2.47–2.63) | 2.57× (2.49–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 51.1 | 1.40× (1.36–1.45) | 3.33× (3.21–3.46) | 2.84× (2.74–2.94) | 2.96× (2.86–3.07) | 3.31× (3.19–3.43) | 4.03× (3.90–4.16) | 4.57× (4.43–4.71) | 1.44× (1.39–1.49) | 1.50× (1.44–1.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 129 | 55.6 | 1.26× (1.18–1.34) | 3.05× (2.84–3.26) | 2.77× (2.60–2.95) | 3.70× (3.48–3.93) | 3.20× (3.00–3.41) | 6.58× (6.18–7.00) | 7.47× (7.01–7.95) | 2.36× (2.22–2.51) | 2.46× (2.32–2.60) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.061 s | 158 | 78.4 | 1.47× (1.40–1.53) | 3.01× (2.88–3.14) | 2.76× (2.61–2.92) | 3.61× (3.47–3.76) | 3.27× (3.15–3.40) | 8.34× (8.06–8.63) | 9.46× (9.15–9.79) | 2.99× (2.89–3.09) | 3.11× (2.99–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,105 | 4.0 | 0.71× (0.65–0.77) | 3.31× (3.06–3.57) | 4.63× (4.28–5.01) | 3.58× (3.33–3.86) | 7.73× (7.21–8.28) | 2.76× (2.56–2.97) | 2.82× (2.63–3.02) | 1.11× (1.02–1.20) | 1.13× (1.05–1.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 971 | 4.6 | 0.58× (0.56–0.61) | 2.99× (2.84–3.16) | 5.72× (5.43–6.03) | 3.29× (3.14–3.46) | 7.09× (6.82–7.38) | 2.49× (2.35–2.64) | 2.54× (2.40–2.69) | 1.00× (0.91–1.09) | 1.02× (0.97–1.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 939 | 4.6 | 0.57× (0.55–0.59) | 2.91× (2.82–2.99) | 5.36× (4.99–5.75) | 3.22× (3.12–3.33) | 6.84× (6.60–7.10) | 2.41× (2.33–2.49) | 2.46× (2.39–2.54) | 0.97× (0.91–1.03) | 0.99× (0.96–1.02) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 0.056 s | 218 | 18.3 | 1.86× (1.84–1.89) | 4.26× (4.16–4.37) | 4.78× (4.70–4.86) | 4.82× (4.76–4.88) | 5.01× (4.94–5.08) | 5.21× (5.14–5.28) | 5.11× (5.05–5.18) | 1.50× (1.48–1.52) | 1.46× (1.44–1.48) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 0.037 s | 333 | 21.2 | 1.46× (1.29–1.65) | 3.36× (2.91–3.88) | 4.46× (3.90–5.09) | 4.46× (3.92–5.07) | 4.26× (3.74–4.85) | 7.32× (6.38–8.41) | 7.19× (6.26–8.26) | 2.11× (1.84–2.42) | 2.05× (1.78–2.36) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 0.030 s | 403 | 32.0 | 1.66× (1.60–1.73) | 3.10× (2.98–3.23) | 4.32× (4.16–4.49) | 4.44× (4.31–4.58) | 4.38× (4.28–4.48) | 9.48× (9.16–9.81) | 9.31× (8.99–9.64) | 2.73× (2.64–2.81) | 2.65× (2.56–2.75) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.27× (3.24–3.31) | 7.41× (7.32–7.50) | 7.90× (7.80–7.99) | 8.75× (8.63–8.87) | 7.91× (7.82–8.00) | 8.29× (8.18–8.39) | 8.07× (7.98–8.17) | 2.55× (2.52–2.58) | 2.25× (2.22–2.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.02–3.13) | 7.02× (6.90–7.15) | 7.74× (7.58–7.91) | 8.41× (8.24–8.59) | 7.35× (7.19–7.50) | 7.50× (7.33–7.67) | 7.31× (7.15–7.47) | 2.31× (2.26–2.36) | 2.04× (1.99–2.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.20× (3.15–3.25) | 6.50× (6.35–6.66) | 6.92× (6.77–7.07) | 7.54× (7.41–7.69) | 6.78× (6.65–6.91) | 4.88× (4.79–4.97) | 4.75× (4.67–4.83) | 1.50× (1.47–1.53) | 1.32× (1.30–1.35) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.70× (0.67–0.74) | 3.18× (2.93–3.46) | 4.24× (3.91–4.60) | 3.78× (3.49–4.10) | 6.44× (5.92–7.01) | 2.93× (2.69–3.20) | 2.95× (2.71–3.22) | 1.10× (1.02–1.19) | 0.89× (0.82–0.96) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.67× (0.63–0.71) | 3.17× (2.96–3.39) | 5.98× (5.61–6.38) | 3.75× (3.52–3.99) | 6.22× (5.86–6.60) | 2.87× (2.68–3.08) | 2.89× (2.69–3.10) | 1.08× (1.00–1.16) | 0.87× (0.82–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.63× (0.60–0.66) | 3.32× (3.14–3.52) | 5.68× (5.36–6.02) | 3.62× (3.45–3.81) | 6.03× (5.76–6.33) | 2.74× (2.58–2.91) | 2.76× (2.60–2.93) | 1.03× (0.94–1.12) | 0.83× (0.78–0.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.92× (2.81–3.04) | 3.22× (3.09–3.35) | 3.76× (3.62–3.90) | 4.31× (4.14–4.49) | 4.71× (4.55–4.89) | 3.19× (3.07–3.32) | 2.96× (2.85–3.08) | 1.99× (1.92–2.07) | 1.94× (1.88–2.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.65× (2.52–2.78) | 2.89× (2.76–3.04) | 3.52× (3.35–3.69) | 3.78× (3.60–3.96) | 4.13× (3.96–4.31) | 2.71× (2.59–2.83) | 2.51× (2.40–2.63) | 1.69× (1.61–1.77) | 1.65× (1.59–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.31× (2.19–2.44) | 1.97× (1.89–2.06) | 2.51× (2.40–2.61) | 2.45× (2.32–2.58) | 2.55× (2.44–2.67) | 1.54× (1.47–1.62) | 1.43× (1.36–1.50) | 0.96× (0.91–1.01) | 0.94× (0.89–0.98) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.96× (2.94–2.99) | 2.70× (2.67–2.74) | 2.81× (2.79–2.83) | 3.11× (3.09–3.13) | 3.42× (3.40–3.45) | 3.57× (3.53–3.60) | 1.25× (1.25–1.26) | 1.24× (1.23–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.30–1.33) | 2.89× (2.86–2.92) | 2.62× (2.59–2.65) | 3.31× (3.27–3.35) | 2.94× (2.90–2.98) | 3.13× (3.08–3.17) | 3.26× (3.20–3.32) | 1.14× (1.13–1.16) | 1.13× (1.11–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.48–1.57) | 2.94× (2.82–3.06) | 2.51× (2.42–2.60) | 3.22× (3.14–3.30) | 2.94× (2.85–3.04) | 2.31× (2.25–2.38) | 2.41× (2.33–2.48) | 0.85× (0.82–0.87) | 0.83× (0.81–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.35–1.44) | 3.26× (3.16–3.37) | 2.79× (2.69–2.90) | 2.94× (2.84–3.04) | 3.28× (3.17–3.40) | 3.99× (3.87–4.12) | 4.53× (4.39–4.67) | 1.44× (1.39–1.49) | 1.44× (1.39–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.31–1.39) | 3.19× (3.09–3.29) | 2.73× (2.66–2.80) | 3.70× (3.60–3.81) | 3.13× (3.04–3.22) | 3.69× (3.59–3.80) | 4.19× (4.07–4.31) | 1.33× (1.29–1.37) | 1.33× (1.30–1.37) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.57–1.67) | 3.21× (3.11–3.32) | 2.47× (2.38–2.57) | 3.35× (3.28–3.43) | 3.01× (2.92–3.10) | 2.63× (2.57–2.70) | 2.98× (2.91–3.06) | 0.95× (0.92–0.97) | 0.95× (0.93–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.74× (0.69–0.80) | 2.78× (2.59–2.99) | 3.82× (3.60–4.05) | 3.28× (3.05–3.53) | 6.95× (6.49–7.45) | 2.57× (2.38–2.77) | 2.61× (2.44–2.80) | 1.10× (1.02–1.19) | 0.86× (0.80–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.61–0.67) | 2.51× (2.37–2.66) | 4.74× (4.53–4.97) | 3.04× (2.88–3.21) | 6.33× (6.06–6.61) | 2.29× (2.14–2.45) | 2.33× (2.18–2.49) | 0.98× (0.89–1.08) | 0.77× (0.72–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.60–0.63) | 2.76× (2.64–2.89) | 4.62× (4.45–4.79) | 3.04× (2.97–3.11) | 6.24× (6.02–6.46) | 2.26× (2.18–2.35) | 2.30× (2.23–2.38) | 0.97× (0.90–1.04) | 0.76× (0.74–0.79) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 1.84× (1.82–1.87) | 4.12× (4.05–4.18) | 4.60× (4.54–4.67) | 4.73× (4.67–4.79) | 4.91× (4.85–4.98) | 5.11× (5.04–5.17) | 5.01× (4.95–5.07) | 1.49× (1.47–1.51) | 1.39× (1.38–1.40) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 1.65× (1.58–1.71) | 3.65× (3.48–3.83) | 4.27× (4.11–4.44) | 4.52× (4.35–4.70) | 4.25× (4.09–4.43) | 4.30× (4.09–4.51) | 4.22× (4.02–4.43) | 1.25× (1.19–1.31) | 1.17× (1.11–1.23) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 1.81× (1.70–1.92) | 3.39× (3.28–3.51) | 3.69× (3.59–3.80) | 4.00× (3.89–4.12) | 3.83× (3.72–3.93) | 2.90× (2.80–3.00) | 2.85× (2.75–2.95) | 0.84× (0.82–0.87) | 0.79× (0.76–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.24× (0.24–0.24) | 0.81× (0.81–0.81) | 0.64× (0.64–0.65) | 9.11× (9.10–9.13) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 80 MB | 0.24× (0.23–0.26) | 0.34× (0.29–0.41) | 0.50× (0.47–0.53) | 0.42× (0.39–0.44) | 3.97× (3.75–4.21) | 0.09× (0.09–0.10) | 0.10× (0.10–0.11) | 0.15× (0.15–0.16) | 0.08× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 132 MB | 0.23× (0.22–0.25) | 0.34× (0.27–0.43) | 0.41× (0.38–0.45) | 0.37× (0.34–0.40) | 2.43× (2.26–2.62) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.09× (0.09–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.03× (8.00–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.70) | 0.61× (0.56–0.67) | 1.64× (1.60–1.68) | 0.82× (0.80–0.83) | 5.94× (5.81–6.06) | 0.48× (0.47–0.49) | 0.51× (0.49–0.52) | 0.55× (0.53–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.61–0.64) | 0.68× (0.63–0.74) | 1.12× (1.09–1.16) | 0.70× (0.68–0.73) | 3.38× (3.29–3.46) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.74× (0.73–0.74) | 5.23× (5.22–5.25) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.53) | 0.40× (0.37–0.43) | 0.90× (0.87–0.94) | 0.72× (0.69–0.75) | 3.12× (3.01–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.66) | 0.46× (0.44–0.47) | 0.87× (0.84–0.89) | 0.79× (0.76–0.82) | 2.52× (2.44–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.23× (5.21–5.25) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.41) | 0.85× (0.82–0.89) | 0.72× (0.68–0.76) | 2.92× (2.81–3.04) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.48× (0.45–0.52) | 0.81× (0.80–0.82) | 0.83× (0.81–0.86) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.37) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.30–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.42) | 0.66× (0.66–0.66) | 6.41× (6.40–6.42) | 0.39× (0.38–0.39) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 38 MB | 0.43× (0.42–0.44) | 0.37× (0.33–0.41) | 1.01× (0.99–1.04) | 0.64× (0.62–0.67) | 3.18× (3.11–3.26) | 0.19× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.15× (0.15–0.16) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 57 MB | 0.48× (0.46–0.50) | 0.45× (0.40–0.49) | 0.89× (0.86–0.91) | 0.71× (0.68–0.74) | 2.25× (2.20–2.31) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 508 | 1.97 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 894 | 2.18 | 1.10× (1.08–1.13) | 85% | 1.05× (1.04–1.06) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,055 | 3.39 | 1.70× (1.67–1.73) | 53% | 1.49× (1.48–1.50) | 1.66× (1.65–1.67) |

**Best:** mibz2 at 4 threads (1,055 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (330 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.16, lbzip2 1t 0.04, lbzip2 2t 0.18, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.55 | 0.02 | 0.07 | 0.199 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3882 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 10.5% |
| MTF + RLE2 | 1.61 | 15.1% |
| T-vector build | 1.55 | 14.5% |
| walk (the one each build uses) | 5.99 | 56.2% |
| of which: segment assembly | 0.30 | 2.8% |
| RLE1 | 0.25 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.370 | 0.927 | (in its total) | 2.51× (2.50–2.51) |  |
| walk | 1.464 | 2.781 | (in its total) | 1.90× (1.90–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.072 | (in its total) | 0.91× (0.91–0.92) |  |
| inverse BWT, total | 1.914 | 3.781 | 1.974 | 1.98× (1.97–1.98) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 7m01s |
| 4 threads | 28m38s ± 4m03s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m49s ± 2m50s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 19m49s ± 2m50s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 2.60 ns a byte of output at 4 threads (CPU, summed over them), 1.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,417,543 | 12,200,394 | \`a6b24102e87d445d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3881.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
