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

The latest record: [`f0fdd8b`](<https://github.com/lmmx/mibz2/commit/f0fdd8b9e2d8832a8dddfc13aa5c700bd1787313>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45967852401>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.383 s | 548 | 6.8 | 3.20× (3.13–3.26) | 7.30× (7.17–7.44) | 7.56× (7.43–7.70) | 8.59× (8.45–8.73) | 7.53× (7.44–7.62) | 7.99× (7.81–8.17) | 7.59× (7.38–7.81) | 2.44× (2.37–2.50) | 2.63× (2.57–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 958 | 7.4 | 3.36× (3.24–3.48) | 6.56× (6.33–6.79) | 7.76× (7.57–7.95) | 8.21× (7.80–8.65) | 7.06× (6.75–7.39) | 13.84× (13.36–14.34) | 13.16× (12.66–13.68) | 4.22× (4.05–4.40) | 4.56× (4.38–4.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.185 s | 1,134 | 11.9 | 3.28× (3.12–3.44) | 4.86× (4.67–5.06) | 5.99× (5.60–6.40) | 5.91× (5.46–6.40) | 5.83× (5.39–6.31) | 16.67× (15.85–17.53) | 15.84× (15.03–16.70) | 5.08× (4.80–5.39) | 5.49× (5.26–5.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,099 | 3.7 | 0.56× (0.56–0.57) | 3.38× (3.33–3.43) | 4.56× (4.44–4.68) | 3.54× (3.49–3.59) | 6.02× (5.95–6.09) | 2.62× (2.59–2.66) | 2.62× (2.59–2.66) | 0.95× (0.93–0.97) | 1.74× (1.63–1.86) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,064 | 4.0 | 0.51× (0.50–0.52) | 2.74× (2.69–2.79) | 5.48× (5.38–5.58) | 3.42× (3.36–3.48) | 5.72× (5.62–5.82) | 2.54× (2.50–2.59) | 2.54× (2.50–2.59) | 0.92× (0.91–0.94) | 1.69× (1.59–1.80) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,050 | 4.0 | 0.51× (0.50–0.52) | 2.70× (2.63–2.76) | 5.38× (5.25–5.51) | 3.36× (3.28–3.44) | 5.73× (5.58–5.88) | 2.49× (2.43–2.55) | 2.49× (2.43–2.55) | 0.91× (0.88–0.93) | 1.66× (1.55–1.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 776 | 5.1 | 2.83× (2.80–2.85) | 3.21× (3.17–3.25) | 4.02× (3.97–4.07) | 4.28× (4.23–4.33) | 4.71× (4.66–4.77) | 2.89× (2.86–2.92) | 2.40× (2.32–2.49) | 2.19× (2.18–2.20) | 3.20× (3.16–3.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,170 | 6.4 | 2.25× (1.95–2.61) | 2.90× (2.49–3.38) | 4.43× (3.81–5.14) | 3.63× (3.11–4.24) | 5.18× (4.46–6.02) | 4.15× (3.56–4.84) | 3.45× (2.94–4.06) | 3.14× (2.71–3.64) | 4.59× (3.93–5.36) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,026 | 12.2 | 2.06× (1.98–2.14) | 2.02× (1.82–2.25) | 3.17× (2.92–3.45) | 2.31× (2.20–2.42) | 4.38× (4.19–4.59) | 3.90× (3.74–4.06) | 3.24× (3.08–3.41) | 2.95× (2.83–3.08) | 4.31× (4.14–4.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.103 s | 78 | 48.6 | 1.50× (1.49–1.51) | 2.87× (2.83–2.91) | 2.59× (2.56–2.63) | 2.70× (2.67–2.73) | 3.01× (2.96–3.06) | 3.22× (3.18–3.25) | 3.32× (3.28–3.37) | 1.38× (1.35–1.41) | 1.54× (1.49–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 128 | 54.5 | 1.40× (1.35–1.46) | 2.77× (2.65–2.88) | 2.60× (2.50–2.70) | 3.39× (3.24–3.55) | 3.14× (3.05–3.23) | 5.27× (5.13–5.41) | 5.45× (5.29–5.61) | 2.26× (2.20–2.33) | 2.52× (2.42–2.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 159 | 70.0 | 1.52× (1.48–1.56) | 2.41× (2.34–2.48) | 2.44× (2.36–2.53) | 3.06× (2.98–3.14) | 3.27× (3.22–3.32) | 6.64× (6.54–6.75) | 6.87× (6.77–6.96) | 2.86× (2.78–2.94) | 3.18× (3.10–3.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 86 | 43.9 | 1.64× (1.61–1.66) | 3.27× (3.23–3.31) | 2.85× (2.79–2.91) | 2.99× (2.95–3.04) | 3.34× (3.29–3.39) | 3.97× (3.91–4.02) | 4.27× (4.19–4.35) | 1.65× (1.62–1.68) | 1.94× (1.90–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 49.5 | 1.45× (1.37–1.54) | 2.96× (2.80–3.13) | 2.68× (2.54–2.84) | 3.73× (3.50–3.98) | 3.22× (3.04–3.41) | 6.34× (6.01–6.69) | 6.83× (6.46–7.21) | 2.64× (2.51–2.78) | 3.10× (2.94–3.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 165 | 68.6 | 1.50× (1.41–1.60) | 2.44× (2.30–2.58) | 2.43× (2.24–2.62) | 3.27× (3.06–3.49) | 3.06× (2.87–3.26) | 7.43× (6.99–7.91) | 8.00× (7.52–8.52) | 3.10× (2.91–3.30) | 3.63× (3.41–3.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 5.1 | 0.62× (0.61–0.63) | 3.43× (3.37–3.49) | 4.32× (4.24–4.39) | 3.08× (3.02–3.13) | 6.60× (6.47–6.73) | 2.27× (2.23–2.31) | 2.29× (2.25–2.33) | 0.94× (0.92–0.96) | 1.72× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 787 | 5.7 | 0.52× (0.49–0.55) | 2.42× (2.38–2.47) | 4.87× (4.79–4.95) | 2.90× (2.86–2.95) | 6.16× (6.06–6.26) | 2.13× (2.09–2.16) | 2.15× (2.12–2.19) | 0.88× (0.86–0.90) | 1.62× (1.59–1.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 5.6 | 0.51× (0.50–0.51) | 2.41× (2.38–2.44) | 4.84× (4.78–4.90) | 2.89× (2.86–2.92) | 6.12× (6.04–6.21) | 2.11× (2.08–2.13) | 2.13× (2.11–2.16) | 0.87× (0.86–0.89) | 1.60× (1.58–1.63) | yes |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 1 | 0.044 s | 277 | 14.0 | 2.00× (1.98–2.02) | 4.20× (4.15–4.25) | 4.63× (4.57–4.69) | 4.82× (4.76–4.87) | 5.14× (5.09–5.18) | 4.74× (4.70–4.78) | 4.67× (4.63–4.71) | 1.67× (1.63–1.70) | 1.90× (1.85–1.95) | yes |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 2 | 0.031 s | 396 | 17.8 | 1.66× (1.57–1.75) | 3.35× (3.23–3.48) | 4.10× (3.93–4.27) | 4.53× (4.36–4.71) | 4.76× (4.60–4.92) | 6.75× (6.48–7.02) | 6.65× (6.38–6.93) | 2.38× (2.28–2.48) | 2.70× (2.58–2.82) | yes |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 4 | 0.027 s | 449 | 27.0 | 1.61× (1.56–1.65) | 2.80× (2.58–3.04) | 3.52× (3.32–3.74) | 4.13× (4.01–4.25) | 4.30× (4.20–4.40) | 7.65× (7.44–7.86) | 7.54× (7.33–7.75) | 2.69× (2.65–2.74) | 3.06× (3.00–3.12) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.19× (3.12–3.25) | 7.26× (7.13–7.40) | 7.52× (7.38–7.65) | 8.56× (8.42–8.70) | 7.50× (7.41–7.58) | 7.95× (7.78–8.13) | 7.56× (7.35–7.78) | 2.43× (2.37–2.49) | 2.61× (2.55–2.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.15× (3.07–3.23) | 6.90× (6.84–6.96) | 7.30× (7.16–7.45) | 8.57× (8.33–8.83) | 7.12× (6.99–7.25) | 7.40× (7.24–7.56) | 7.03× (6.91–7.16) | 2.26× (2.20–2.32) | 2.43× (2.39–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.06–3.32) | 5.33× (5.14–5.53) | 5.76× (5.47–6.07) | 6.46× (6.10–6.85) | 5.57× (5.22–5.95) | 4.68× (4.55–4.82) | 4.45× (4.36–4.55) | 1.43× (1.40–1.47) | 1.54× (1.50–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.60–0.61) | 2.88× (2.84–2.93) | 3.87× (3.78–3.96) | 3.32× (3.27–3.36) | 5.58× (5.52–5.63) | 2.48× (2.45–2.51) | 2.48× (2.45–2.51) | 0.96× (0.94–0.99) | 1.62× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.56) | 2.60× (2.56–2.64) | 4.67× (4.61–4.72) | 3.13× (3.07–3.18) | 5.15× (5.06–5.24) | 2.33× (2.29–2.36) | 2.33× (2.29–2.36) | 0.90× (0.88–0.92) | 1.51× (1.49–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.52–0.55) | 2.81× (2.74–2.89) | 4.61× (4.50–4.73) | 3.12× (3.05–3.19) | 5.22× (5.09–5.35) | 2.32× (2.26–2.38) | 2.32× (2.27–2.37) | 0.90× (0.87–0.93) | 1.51× (1.47–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.69–2.73) | 2.95× (2.92–2.98) | 3.52× (3.50–3.54) | 4.07× (4.02–4.12) | 4.47× (4.42–4.53) | 2.77× (2.73–2.80) | 2.31× (2.23–2.39) | 2.11× (2.10–2.12) | 2.76× (2.74–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.32–2.43) | 2.63× (2.54–2.72) | 3.56× (3.43–3.70) | 3.50× (3.40–3.60) | 3.79× (3.70–3.88) | 2.25× (2.19–2.32) | 1.88× (1.79–1.98) | 1.72× (1.68–1.76) | 2.25× (2.19–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.00–2.25) | 1.81× (1.68–1.94) | 2.37× (2.25–2.50) | 2.17× (2.05–2.30) | 2.24× (2.12–2.37) | 1.15× (1.09–1.22) | 0.96× (0.90–1.03) | 0.88× (0.84–0.93) | 1.15× (1.09–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 2.78× (2.75–2.80) | 2.54× (2.50–2.58) | 2.67× (2.64–2.70) | 2.98× (2.93–3.02) | 3.18× (3.14–3.21) | 3.29× (3.25–3.33) | 1.38× (1.35–1.41) | 1.49× (1.46–1.52) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.38× (1.34–1.43) | 2.65× (2.54–2.76) | 2.32× (2.25–2.39) | 3.14× (3.04–3.23) | 2.71× (2.64–2.78) | 2.81× (2.74–2.88) | 2.91× (2.83–2.99) | 1.22× (1.19–1.25) | 1.32× (1.28–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.68× (1.63–1.73) | 2.58× (2.50–2.66) | 2.19× (2.13–2.26) | 3.00× (2.91–3.09) | 2.79× (2.72–2.87) | 2.22× (2.18–2.25) | 2.29× (2.25–2.33) | 0.96× (0.93–0.99) | 1.04× (1.01–1.07) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.60–1.65) | 3.20× (3.15–3.24) | 2.79× (2.74–2.84) | 2.96× (2.92–3.01) | 3.30× (3.26–3.35) | 3.92× (3.86–3.98) | 4.22× (4.14–4.30) | 1.64× (1.61–1.67) | 1.89× (1.85–1.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.42–1.56) | 2.95× (2.80–3.11) | 2.52× (2.41–2.64) | 3.55× (3.38–3.73) | 2.94× (2.79–3.09) | 3.43× (3.28–3.59) | 3.69× (3.51–3.89) | 1.44× (1.38–1.50) | 1.65× (1.58–1.73) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.71× (1.61–1.80) | 2.71× (2.56–2.86) | 2.15× (2.01–2.29) | 3.09× (2.91–3.28) | 2.80× (2.64–2.97) | 2.45× (2.31–2.61) | 2.64× (2.49–2.80) | 1.03× (0.97–1.09) | 1.18× (1.11–1.25) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.67) | 2.46× (2.41–2.50) | 3.38× (3.32–3.45) | 2.82× (2.77–2.87) | 5.90× (5.80–6.01) | 2.11× (2.07–2.14) | 2.13× (2.10–2.17) | 0.95× (0.93–0.97) | 1.35× (1.33–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.54–0.59) | 2.14× (2.11–2.18) | 3.79× (3.72–3.85) | 2.60× (2.56–2.64) | 5.28× (5.22–5.35) | 1.89× (1.86–1.92) | 1.91× (1.88–1.94) | 0.85× (0.83–0.87) | 1.21× (1.19–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.55) | 2.37× (2.33–2.40) | 3.84× (3.78–3.89) | 2.67× (2.64–2.70) | 5.41× (5.33–5.50) | 1.93× (1.91–1.96) | 1.95× (1.93–1.98) | 0.87× (0.85–0.89) | 1.24× (1.22–1.25) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 1 | 1.96× (1.94–1.99) | 3.92× (3.89–3.94) | 4.34× (4.28–4.40) | 4.67× (4.62–4.73) | 4.98× (4.94–5.02) | 4.60× (4.56–4.63) | 4.53× (4.49–4.57) | 1.64× (1.61–1.68) | 1.79× (1.78–1.81) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 2 | 1.69× (1.61–1.78) | 3.25× (3.17–3.34) | 3.61× (3.54–3.68) | 4.21× (4.12–4.29) | 4.00× (3.90–4.10) | 3.61× (3.53–3.68) | 3.56× (3.48–3.63) | 1.29× (1.26–1.32) | 1.41× (1.38–1.43) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 4 | 1.73× (1.69–1.78) | 2.73× (2.62–2.84) | 3.19× (2.97–3.44) | 3.46× (3.29–3.63) | 3.25× (3.21–3.29) | 2.36× (2.32–2.40) | 2.33× (2.29–2.37) | 0.84× (0.83–0.86) | 0.92× (0.91–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.33× (0.31–0.35) | 0.98× (0.97–0.98) | 0.79× (0.78–0.79) | 11.06× (11.03–11.10) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.43× (0.43–0.44) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.49× (0.41–0.59) | 0.72× (0.70–0.75) | 0.61× (0.59–0.63) | 5.79× (5.62–5.96) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.36–0.38) | 0.53× (0.44–0.64) | 0.66× (0.63–0.68) | 0.58× (0.56–0.60) | 3.85× (3.74–3.98) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.82–4.86) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.18–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.04) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.66–0.66) | 8.00× (7.98–8.01) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.71) | 0.65× (0.61–0.68) | 1.63× (1.58–1.69) | 0.82× (0.79–0.84) | 5.92× (5.73–6.11) | 0.48× (0.46–0.50) | 0.51× (0.49–0.52) | 0.55× (0.53–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.68× (0.66–0.71) | 0.75× (0.69–0.81) | 1.20× (1.14–1.26) | 0.74× (0.71–0.78) | 3.53× (3.37–3.71) | 0.28× (0.26–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.51) | 0.41× (0.39–0.43) | 0.89× (0.87–0.92) | 0.71× (0.68–0.74) | 3.09× (3.01–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.46× (0.45–0.46) | 0.87× (0.85–0.88) | 0.82× (0.80–0.84) | 2.52× (2.49–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.35× (0.32–0.39) | 0.85× (0.83–0.87) | 0.72× (0.69–0.75) | 2.92× (2.84–3.00) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.83× (0.79–0.86) | 2.34× (2.32–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.23–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.91) | 6.20× (6.18–6.23) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.19× (6.17–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 1 | 19 MB | 0.47× (0.46–0.47) | 0.38× (0.38–0.38) | 1.41× (1.40–1.41) | 0.65× (0.64–0.66) | 6.35× (6.32–6.39) | 0.39× (0.39–0.40) | 0.41× (0.41–0.42) | 0.64× (0.64–0.65) | 0.31× (0.31–0.31) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.39× (0.36–0.44) | 1.11× (1.08–1.14) | 0.71× (0.69–0.72) | 3.60× (3.54–3.66) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.36× (0.36–0.37) | 0.17× (0.17–0.18) |
| `text` (1.86 MB → 12.1 MB, 6.5×) | 4 | 49 MB | 0.57× (0.56–0.58) | 0.48× (0.45–0.52) | 1.03× (1.01–1.05) | 0.81× (0.79–0.83) | 2.59× (2.55–2.64) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.26) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 548 | 1.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 958 | 1.99 | 1.07× (1.05–1.10) | 87% | 1.02× (1.01–1.04) | 1.06× (1.04–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 1,134 | 3.18 | 1.70× (1.65–1.74) | 52% | 1.25× (1.21–1.28) | 1.70× (1.64–1.76) |

**Best:** mibz2 at 4 threads (1,134 MB/s); lbzip2 at 4 threads (234 MB/s); pbz2 at 4 threads (339 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.08, lbzip2 1t 0.07, lbzip2 2t 0.24, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.02 | 0.08 | 0.185 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3733 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 9.9% |
| MTF + RLE2 | 1.48 | 17.3% |
| T-vector build | 1.51 | 17.7% |
| walk (the one each build uses) | 4.25 | 49.8% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.064 | (in its total) | 2.71× (2.70–2.73) |  |
| walk | 1.055 | 2.097 | (in its total) | 1.98× (1.97–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.040 | 0.042 | (in its total) | 1.04× (1.01–1.08) |  |
| inverse BWT, total | 1.489 | 3.204 | 1.906 | 2.15× (2.13–2.16) | 1.28× (1.27–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m37s ± 6m54s |
| 4 threads | 27m09s ± 3m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m27s ± 2m40s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 18m26s ± 2m40s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.42 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,855,556 | 12,135,318 | \`8d290aeb4f9ab0c9…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3733.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
