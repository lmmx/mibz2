# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-793/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`5d4a34e`](<https://github.com/lmmx/mibz2/commit/5d4a34e58da47e731c09f5d73dcb4027ade85999>) (2026-11-23).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45144727869>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.392 s | 535 | 7.0 | 3.15× (3.12–3.19) | 7.17× (7.13–7.21) | 7.49× (7.43–7.55) | 8.52× (8.44–8.59) | 7.46× (7.41–7.51) | 7.84× (7.76–7.93) | 7.37× (7.31–7.44) | 2.34× (2.31–2.37) | 2.59× (2.57–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.222 s | 946 | 7.5 | 3.28× (3.17–3.39) | 6.69× (6.45–6.95) | 7.62× (7.43–7.81) | 7.91× (7.69–8.13) | 6.99× (6.84–7.16) | 13.79× (13.42–14.17) | 12.97× (12.62–13.32) | 4.11× (3.99–4.24) | 4.56× (4.44–4.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.183 s | 1,145 | 11.9 | 3.27× (3.20–3.34) | 4.76× (4.67–4.86) | 5.98× (5.84–6.12) | 5.88× (5.76–6.00) | 5.74× (5.67–5.81) | 16.71× (16.39–17.04) | 15.71× (15.42–16.01) | 4.99× (4.88–5.09) | 5.52× (5.43–5.61) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,108 | 3.7 | 0.57× (0.56–0.59) | 3.41× (3.31–3.51) | 4.55× (4.42–4.69) | 3.56× (3.47–3.65) | 6.22× (6.04–6.40) | 2.65× (2.58–2.72) | 2.66× (2.58–2.73) | 0.97× (0.95–1.00) | 1.86× (1.68–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,058 | 4.0 | 0.51× (0.50–0.51) | 2.71× (2.68–2.73) | 5.42× (5.37–5.47) | 3.40× (3.36–3.44) | 5.70× (5.62–5.78) | 2.52× (2.49–2.54) | 2.52× (2.50–2.54) | 0.92× (0.90–0.95) | 1.76× (1.60–1.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,034 | 4.0 | 0.50× (0.49–0.51) | 2.67× (2.62–2.73) | 5.48× (5.14–5.84) | 3.35× (3.29–3.42) | 5.70× (5.58–5.83) | 2.47× (2.42–2.53) | 2.48× (2.43–2.53) | 0.91× (0.88–0.94) | 1.74× (1.58–1.90) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 740 | 5.3 | 2.74× (2.71–2.77) | 3.13× (3.04–3.21) | 3.86× (3.84–3.88) | 4.12× (4.09–4.15) | 4.52× (4.47–4.58) | 2.78× (2.76–2.80) | 2.29× (2.26–2.31) | 2.12× (2.10–2.14) | 3.08× (3.06–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,177 | 6.4 | 2.37× (2.31–2.45) | 3.04× (2.96–3.13) | 4.76× (4.36–5.21) | 3.88× (3.78–4.00) | 5.46× (5.32–5.60) | 4.36× (4.23–4.48) | 3.58× (3.47–3.69) | 3.32× (3.24–3.41) | 4.82× (4.68–4.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,032 | 12.9 | 2.02× (1.96–2.08) | 1.96× (1.78–2.17) | 3.29× (3.12–3.47) | 2.37× (2.26–2.49) | 4.35× (4.19–4.51) | 3.87× (3.74–4.00) | 3.18× (3.07–3.30) | 2.95× (2.84–3.07) | 4.28× (4.13–4.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.108 s | 74 | 51.3 | 1.44× (1.42–1.46) | 2.72× (2.69–2.74) | 2.46× (2.41–2.51) | 2.56× (2.54–2.58) | 2.90× (2.86–2.94) | 3.06× (3.04–3.09) | 3.06× (3.03–3.08) | 1.29× (1.28–1.30) | 1.48× (1.46–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.065 s | 123 | 57.4 | 1.32× (1.22–1.43) | 2.51× (2.33–2.70) | 2.38× (2.20–2.58) | 3.05× (2.84–3.29) | 2.93× (2.71–3.17) | 4.89× (4.52–5.28) | 4.88× (4.51–5.27) | 2.06× (1.90–2.23) | 2.37× (2.20–2.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.051 s | 156 | 74.9 | 1.47× (1.38–1.56) | 2.34× (2.27–2.42) | 2.33× (2.25–2.42) | 2.86× (2.78–2.93) | 3.18× (3.11–3.24) | 6.37× (6.23–6.52) | 6.35× (6.22–6.49) | 2.68× (2.62–2.74) | 3.08× (3.02–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 45.9 | 1.53× (1.47–1.60) | 3.09× (2.98–3.21) | 2.65× (2.54–2.76) | 2.79× (2.69–2.90) | 3.14× (3.03–3.26) | 3.74× (3.58–3.91) | 3.90× (3.76–4.04) | 1.57× (1.53–1.62) | 1.84× (1.77–1.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 138 | 52.1 | 1.44× (1.39–1.49) | 2.90× (2.78–3.03) | 2.66× (2.55–2.78) | 3.65× (3.52–3.79) | 3.24× (3.12–3.36) | 6.33× (6.12–6.54) | 6.59× (6.34–6.85) | 2.66× (2.57–2.76) | 3.11× (2.99–3.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.060 s | 159 | 74.3 | 1.52× (1.47–1.59) | 2.42× (2.27–2.58) | 2.41× (2.35–2.48) | 3.26× (3.14–3.39) | 3.15× (2.99–3.33) | 7.42× (7.23–7.61) | 7.73× (7.56–7.90) | 3.12× (2.99–3.26) | 3.64× (3.53–3.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 5.1 | 0.62× (0.61–0.62) | 3.40× (3.36–3.45) | 4.28× (4.22–4.34) | 3.13× (2.95–3.32) | 6.71× (6.61–6.82) | 2.25× (2.21–2.28) | 2.28× (2.25–2.32) | 0.94× (0.92–0.96) | 1.71× (1.69–1.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 772 | 5.8 | 0.50× (0.49–0.51) | 2.40× (2.36–2.44) | 4.82× (4.74–4.89) | 2.90× (2.85–2.95) | 6.19× (6.07–6.32) | 2.10× (2.07–2.13) | 2.14× (2.10–2.17) | 0.88× (0.86–0.89) | 1.60× (1.57–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 772 | 5.6 | 0.51× (0.48–0.53) | 2.45× (2.28–2.64) | 4.76× (4.63–4.90) | 2.88× (2.84–2.93) | 6.07× (5.89–6.25) | 2.08× (2.02–2.14) | 2.11× (2.05–2.17) | 0.87× (0.84–0.90) | 1.58× (1.53–1.63) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.058 s | 238 | 16.1 | 1.91× (1.88–1.93) | 4.06× (4.01–4.10) | 4.58× (4.50–4.67) | 4.78× (4.75–4.81) | 4.96× (4.83–5.09) | 4.96× (4.90–5.02) | 4.79× (4.75–4.82) | 1.57× (1.56–1.59) | 1.81× (1.76–1.87) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.040 s | 349 | 20.1 | 1.50× (1.34–1.67) | 3.33× (3.10–3.58) | 3.95× (3.63–4.30) | 4.59× (4.24–4.97) | 4.27× (3.86–4.72) | 6.91× (6.31–7.56) | 6.66× (6.11–7.27) | 2.19× (2.00–2.39) | 2.52× (2.29–2.78) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.037 s | 378 | 31.1 | 1.56× (1.44–1.70) | 2.45× (2.32–2.59) | 3.47× (3.37–3.58) | 3.60× (3.49–3.70) | 3.97× (3.87–4.08) | 7.88× (7.65–8.12) | 7.60× (7.40–7.80) | 2.50× (2.45–2.55) | 2.88× (2.77–2.99) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.11–3.17) | 7.14× (7.10–7.18) | 7.45× (7.38–7.51) | 8.48× (8.41–8.56) | 7.43× (7.38–7.48) | 7.81× (7.73–7.89) | 7.34× (7.28–7.41) | 2.33× (2.30–2.36) | 2.57× (2.55–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.01–3.13) | 6.81× (6.73–6.89) | 7.08× (7.01–7.15) | 8.24× (8.16–8.33) | 6.93× (6.88–6.99) | 7.26× (7.16–7.37) | 6.83× (6.74–6.93) | 2.17× (2.13–2.21) | 2.39× (2.36–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.15× (3.07–3.23) | 5.16× (5.01–5.32) | 5.63× (5.49–5.77) | 6.31× (6.15–6.48) | 5.45× (5.27–5.63) | 4.64× (4.54–4.74) | 4.36× (4.27–4.46) | 1.39× (1.35–1.42) | 1.53× (1.49–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.63) | 2.92× (2.87–2.97) | 3.93× (3.81–4.05) | 3.34× (3.27–3.42) | 5.77× (5.61–5.93) | 2.51× (2.45–2.57) | 2.51× (2.45–2.58) | 0.98× (0.95–1.00) | 1.63× (1.59–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.55) | 2.57× (2.55–2.60) | 4.66× (4.61–4.71) | 3.11× (3.07–3.15) | 5.12× (5.06–5.19) | 2.30× (2.28–2.32) | 2.30× (2.28–2.32) | 0.89× (0.87–0.92) | 1.49× (1.48–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 2.84× (2.77–2.91) | 4.76× (4.50–5.04) | 3.12× (3.06–3.18) | 5.20× (5.09–5.31) | 2.31× (2.26–2.36) | 2.31× (2.26–2.36) | 0.90× (0.87–0.93) | 1.50× (1.47–1.53) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.60–2.66) | 2.88× (2.81–2.95) | 3.42× (3.39–3.45) | 3.92× (3.90–3.95) | 4.31× (4.25–4.36) | 2.67× (2.66–2.69) | 2.21× (2.19–2.23) | 2.05× (2.03–2.07) | 2.66× (2.65–2.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.31–2.38) | 2.61× (2.56–2.66) | 3.54× (3.37–3.71) | 3.45× (3.40–3.51) | 3.76× (3.70–3.81) | 2.22× (2.19–2.25) | 1.84× (1.81–1.87) | 1.71× (1.69–1.72) | 2.22× (2.19–2.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (2.02–2.13) | 1.71× (1.66–1.76) | 2.27× (2.20–2.34) | 2.10× (2.03–2.17) | 2.13× (2.05–2.21) | 1.09× (1.06–1.13) | 0.91× (0.88–0.94) | 0.84× (0.81–0.87) | 1.09× (1.06–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.43× (1.41–1.45) | 2.64× (2.62–2.67) | 2.41× (2.36–2.45) | 2.54× (2.52–2.56) | 2.87× (2.83–2.91) | 3.03× (3.01–3.06) | 3.03× (3.00–3.05) | 1.29× (1.28–1.30) | 1.41× (1.40–1.42) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.20–1.39) | 2.44× (2.26–2.63) | 2.15× (2.00–2.32) | 2.90× (2.70–3.11) | 2.55× (2.37–2.74) | 2.64× (2.45–2.84) | 2.63× (2.44–2.83) | 1.12× (1.04–1.20) | 1.22× (1.14–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.50–1.57) | 2.37× (2.34–2.41) | 2.02× (1.99–2.04) | 2.74× (2.69–2.79) | 2.63× (2.60–2.67) | 2.06× (2.02–2.09) | 2.05× (2.02–2.08) | 0.87× (0.86–0.88) | 0.95× (0.94–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.47–1.59) | 3.02× (2.91–3.13) | 2.59× (2.49–2.69) | 2.76× (2.66–2.87) | 3.11× (3.00–3.22) | 3.70× (3.54–3.86) | 3.85× (3.72–3.99) | 1.56× (1.52–1.61) | 1.77× (1.71–1.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.43× (1.40–1.47) | 2.84× (2.77–2.91) | 2.42× (2.36–2.49) | 3.38× (3.30–3.47) | 2.86× (2.78–2.93) | 3.31× (3.23–3.40) | 3.45× (3.36–3.55) | 1.40× (1.36–1.44) | 1.59× (1.55–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.59–1.66) | 2.60× (2.45–2.76) | 2.03× (1.96–2.09) | 2.91× (2.82–3.00) | 2.73× (2.66–2.81) | 2.32× (2.24–2.40) | 2.42× (2.37–2.47) | 0.98× (0.96–1.01) | 1.11× (1.09–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.43× (2.40–2.46) | 3.40× (3.34–3.45) | 2.87× (2.70–3.05) | 5.99× (5.90–6.08) | 2.09× (2.06–2.11) | 2.12× (2.10–2.15) | 0.95× (0.94–0.96) | 1.34× (1.33–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.12× (2.09–2.14) | 3.78× (3.74–3.81) | 2.60× (2.56–2.64) | 5.31× (5.22–5.41) | 1.87× (1.84–1.89) | 1.90× (1.87–1.92) | 0.85× (0.84–0.86) | 1.20× (1.19–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.52–0.57) | 2.46× (2.25–2.68) | 3.80× (3.67–3.93) | 2.67× (2.64–2.70) | 5.37× (5.22–5.52) | 1.90× (1.85–1.96) | 1.94× (1.88–1.99) | 0.86× (0.83–0.90) | 1.23× (1.19–1.26) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.88× (1.86–1.91) | 3.93× (3.91–3.96) | 4.39× (4.30–4.48) | 4.67× (4.65–4.70) | 4.85× (4.73–4.97) | 4.85× (4.79–4.91) | 4.68× (4.65–4.71) | 1.56× (1.54–1.57) | 1.71× (1.69–1.73) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.61× (1.58–1.63) | 3.31× (3.24–3.39) | 3.65× (3.58–3.71) | 4.33× (4.24–4.42) | 3.97× (3.83–4.12) | 3.88× (3.78–3.97) | 3.74× (3.66–3.82) | 1.24× (1.22–1.27) | 1.37× (1.34–1.40) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.79× (1.66–1.93) | 2.68× (2.59–2.79) | 3.03× (2.92–3.15) | 3.42× (3.31–3.53) | 3.22× (3.14–3.30) | 2.51× (2.46–2.57) | 2.42× (2.37–2.48) | 0.81× (0.79–0.82) | 0.89× (0.87–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.33) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 11.00× (10.98–11.02) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.48× (0.41–0.56) | 0.71× (0.70–0.73) | 0.61× (0.59–0.62) | 5.70× (5.56–5.84) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.37) | 0.50× (0.43–0.58) | 0.63× (0.61–0.66) | 0.56× (0.53–0.59) | 3.72× (3.57–3.87) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.85× (4.85–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.98× (7.94–8.02) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.71× (0.68–0.73) | 0.69× (0.66–0.73) | 1.68× (1.64–1.73) | 0.85× (0.83–0.87) | 6.10× (5.93–6.27) | 0.49× (0.48–0.51) | 0.52× (0.51–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.72) | 0.74× (0.72–0.77) | 1.24× (1.21–1.28) | 0.77× (0.75–0.79) | 3.67× (3.57–3.76) | 0.29× (0.28–0.29) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.15–1.16) | 0.73× (0.72–0.73) | 5.14× (5.12–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.40× (0.39–0.41) | 0.87× (0.85–0.89) | 0.69× (0.65–0.72) | 3.01× (2.93–3.08) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.47× (0.47–0.48) | 0.90× (0.88–0.91) | 0.84× (0.83–0.86) | 2.62× (2.58–2.66) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.40) | 0.83× (0.81–0.87) | 0.72× (0.69–0.74) | 2.87× (2.77–2.98) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.48× (0.45–0.51) | 0.81× (0.79–0.82) | 0.81× (0.78–0.84) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.32× (5.30–5.35) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.90) | 6.21× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.19–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.67× (0.66–0.67) | 6.41× (6.38–6.43) | 0.39× (0.39–0.40) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 33 MB | 0.51× (0.50–0.53) | 0.46× (0.41–0.51) | 1.19× (1.16–1.22) | 0.76× (0.74–0.79) | 3.79× (3.69–3.88) | 0.23× (0.22–0.24) | 0.25× (0.24–0.25) | 0.37× (0.36–0.38) | 0.18× (0.18–0.19) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 51 MB | 0.55× (0.54–0.56) | 0.49× (0.44–0.54) | 1.00× (0.97–1.02) | 0.81× (0.77–0.85) | 2.55× (2.50–2.60) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 535 | 1.88 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 946 | 2.02 | 1.08× (1.07–1.08) | 88% | 1.03× (1.01–1.04) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,145 | 3.19 | 1.68× (1.64–1.72) | 53% | 1.22× (1.19–1.24) | 1.69× (1.64–1.74) |

**Best:** mibz2 at 4 threads (1,145 MB/s); lbzip2 at 4 threads (238 MB/s); pbz2 at 4 threads (350 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.53 | 0.01 | 0.07 | 0.183 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3720 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.89 | 10.5% |
| MTF + RLE2 | 1.56 | 18.3% |
| T-vector build | 1.51 | 17.7% |
| walk (the one each build uses) | 4.10 | 48.2% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.52 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.069 | (in its total) | 2.73× (2.71–2.75) |  |
| walk | 1.062 | 2.092 | (in its total) | 1.98× (1.95–2.01) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.044 | (in its total) | 1.17× (1.10–1.24) |  |
| inverse BWT, total | 1.492 | 3.207 | 1.902 | 2.16× (2.13–2.19) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m03s ± 6m46s |
| 4 threads | 26m46s ± 3m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m53s ± 2m44s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 18m53s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.48 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3719.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
