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

The latest record: [`a1869b5`](<https://github.com/lmmx/mibz2/commit/a1869b5a24cf4016075e41ea5d3316f947ac6d49>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45944633301>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.488 s | 430 | 7.6 | 3.12× (3.11–3.13) | 7.11× (7.08–7.13) | 7.22× (7.18–7.26) | 7.91× (7.88–7.93) | 7.14× (7.11–7.17) | 7.79× (7.76–7.83) | 7.17× (7.15–7.20) | 2.30× (2.28–2.31) | 2.64× (2.61–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.264 s | 794 | 8.0 | 3.19× (3.08–3.30) | 6.80× (6.56–7.04) | 7.63× (7.38–7.89) | 7.64× (7.43–7.86) | 6.93× (6.73–7.13) | 14.16× (13.75–14.58) | 13.04× (12.67–13.41) | 4.17× (4.05–4.30) | 4.79× (4.64–4.95) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.219 s | 956 | 12.7 | 3.16× (3.10–3.23) | 4.84× (4.78–4.91) | 5.96× (5.82–6.11) | 5.49× (5.41–5.57) | 5.56× (5.42–5.70) | 17.22× (17.00–17.45) | 15.86× (15.65–16.06) | 5.07× (5.00–5.15) | 5.83× (5.76–5.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,010 | 3.5 | 0.61× (0.60–0.62) | 3.76× (3.59–3.93) | 4.63× (4.51–4.74) | 3.63× (3.60–3.67) | 6.32× (6.25–6.39) | 3.06× (3.03–3.09) | 2.73× (2.70–2.76) | 1.01× (0.99–1.02) | 1.60× (1.51–1.70) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 948 | 3.9 | 0.55× (0.54–0.56) | 2.92× (2.88–2.95) | 5.77× (5.71–5.83) | 3.45× (3.42–3.48) | 5.89× (5.82–5.96) | 2.87× (2.84–2.90) | 2.56× (2.53–2.59) | 0.94× (0.93–0.96) | 1.50× (1.41–1.60) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 937 | 3.9 | 0.55× (0.54–0.56) | 2.89× (2.85–2.93) | 5.68× (5.61–5.76) | 3.40× (3.36–3.44) | 5.84× (5.76–5.92) | 2.83× (2.79–2.87) | 2.52× (2.49–2.56) | 0.93× (0.91–0.95) | 1.48× (1.38–1.59) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 598 | 5.7 | 2.69× (2.68–2.70) | 3.11× (3.10–3.12) | 3.81× (3.70–3.93) | 3.89× (3.88–3.90) | 4.22× (4.21–4.24) | 2.61× (2.60–2.63) | 2.16× (2.15–2.17) | 2.10× (2.09–2.11) | 3.10× (3.09–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 947 | 6.8 | 2.40× (2.31–2.50) | 3.17× (2.91–3.46) | 4.42× (4.14–4.73) | 4.01× (3.51–4.59) | 5.08× (4.92–5.26) | 4.11× (3.99–4.22) | 3.39× (3.28–3.50) | 3.30× (3.20–3.40) | 4.87× (4.71–5.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 871 | 13.5 | 2.05× (1.95–2.16) | 2.07× (1.93–2.24) | 3.25× (3.07–3.45) | 2.51× (2.43–2.59) | 4.29× (4.10–4.50) | 3.77× (3.62–3.93) | 3.11× (2.99–3.25) | 3.03× (2.91–3.17) | 4.47× (4.28–4.67) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 62 | 52.6 | 1.40× (1.39–1.40) | 2.79× (2.78–2.79) | 2.51× (2.46–2.56) | 2.51× (2.50–2.52) | 2.92× (2.91–2.93) | 3.21× (3.20–3.22) | 3.20× (3.18–3.22) | 1.25× (1.24–1.25) | 1.43× (1.42–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.077 s | 104 | 58.4 | 1.39× (1.32–1.46) | 2.68× (2.60–2.77) | 2.51× (2.46–2.56) | 3.22× (3.14–3.31) | 3.12× (3.03–3.21) | 5.35× (5.25–5.46) | 5.34× (5.23–5.46) | 2.08× (2.04–2.12) | 2.39× (2.32–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.064 s | 125 | 80.1 | 1.40× (1.36–1.44) | 2.28× (2.22–2.33) | 2.16× (2.10–2.22) | 2.71× (2.64–2.78) | 3.00× (2.94–3.07) | 6.39× (6.25–6.53) | 6.37× (6.22–6.53) | 2.48× (2.43–2.54) | 2.85× (2.81–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.140 s | 68 | 48.0 | 1.54× (1.53–1.55) | 3.14× (3.12–3.17) | 2.64× (2.60–2.69) | 2.72× (2.71–2.74) | 3.22× (3.14–3.30) | 3.76× (3.74–3.77) | 3.95× (3.93–3.97) | 1.42× (1.41–1.42) | 1.73× (1.73–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.084 s | 114 | 53.8 | 1.41× (1.36–1.45) | 3.01× (2.64–3.44) | 2.52× (2.44–2.61) | 3.51× (3.41–3.62) | 3.18× (3.09–3.27) | 6.19× (6.01–6.37) | 6.51× (6.33–6.69) | 2.34× (2.27–2.41) | 2.85× (2.77–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 134 | 78.6 | 1.54× (1.51–1.56) | 2.40× (2.31–2.49) | 2.37× (2.30–2.44) | 3.14× (3.05–3.25) | 2.98× (2.93–3.03) | 7.35× (7.23–7.48) | 7.74× (7.60–7.88) | 2.78× (2.73–2.83) | 3.39× (3.33–3.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 772 | 4.8 | 0.68× (0.67–0.69) | 3.24× (3.08–3.40) | 4.72× (4.55–4.90) | 3.21× (3.16–3.26) | 7.03× (6.90–7.17) | 2.64× (2.60–2.67) | 2.40× (2.37–2.44) | 1.00× (0.98–1.02) | 1.59× (1.57–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 683 | 5.6 | 0.55× (0.53–0.57) | 2.81× (2.78–2.85) | 4.84× (4.78–4.90) | 2.93× (2.88–2.97) | 6.38× (6.17–6.59) | 2.34× (2.31–2.37) | 2.13× (2.10–2.16) | 0.89× (0.88–0.90) | 1.41× (1.40–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.5 | 0.55× (0.55–0.56) | 2.83× (2.81–2.86) | 4.86× (4.82–4.90) | 2.91× (2.89–2.94) | 6.28× (6.18–6.38) | 2.35× (2.33–2.37) | 2.14× (2.12–2.16) | 0.89× (0.88–0.91) | 1.42× (1.41–1.43) | yes |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 1 | 0.072 s | 230 | 14.5 | 2.19× (2.14–2.24) | 4.35× (4.25–4.45) | 4.69× (4.58–4.80) | 4.72× (4.61–4.84) | 4.83× (4.69–4.97) | 5.40× (5.28–5.52) | 4.89× (4.78–5.00) | 1.62× (1.58–1.66) | 2.10× (2.05–2.14) | yes |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 2 | 0.049 s | 336 | 18.2 | 1.69× (1.49–1.92) | 3.57× (3.18–4.01) | 3.85× (3.49–4.26) | 4.42× (3.98–4.90) | 4.18× (3.77–4.64) | 7.53× (6.78–8.36) | 6.82× (6.14–7.57) | 2.26× (2.04–2.51) | 2.92× (2.63–3.24) | yes |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 4 | 0.042 s | 392 | 28.6 | 1.70× (1.65–1.75) | 2.72× (2.60–2.85) | 3.37× (3.23–3.52) | 3.79× (3.70–3.88) | 3.93× (3.83–4.04) | 9.20× (8.95–9.45) | 8.33× (8.10–8.56) | 2.77× (2.69–2.84) | 3.57× (3.48–3.67) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (3.10–3.12) | 7.08× (7.05–7.10) | 7.19× (7.15–7.22) | 7.88× (7.85–7.91) | 7.11× (7.08–7.15) | 7.77× (7.73–7.80) | 7.15× (7.12–7.18) | 2.29× (2.28–2.30) | 2.62× (2.60–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.03–3.07) | 6.84× (6.79–6.88) | 7.08× (7.05–7.12) | 7.84× (7.80–7.88) | 6.76× (6.73–6.79) | 7.33× (7.29–7.37) | 6.75× (6.71–6.79) | 2.16× (2.15–2.18) | 2.47× (2.45–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.98–3.07) | 5.15× (5.07–5.24) | 5.48× (5.38–5.58) | 5.83× (5.72–5.95) | 5.15× (5.05–5.25) | 4.71× (4.62–4.80) | 4.34× (4.25–4.42) | 1.39× (1.37–1.41) | 1.59× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.36× (3.32–3.41) | 4.19× (4.11–4.27) | 3.42× (3.38–3.46) | 5.89× (5.82–5.96) | 2.90× (2.87–2.94) | 2.59× (2.56–2.62) | 1.01× (0.99–1.02) | 1.12× (1.03–1.23) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.58–0.59) | 2.91× (2.89–2.94) | 5.12× (5.08–5.17) | 3.15× (3.13–3.17) | 5.29× (5.24–5.34) | 2.62× (2.60–2.64) | 2.34× (2.32–2.35) | 0.91× (0.90–0.92) | 1.01× (0.92–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.18× (3.12–3.25) | 5.07× (4.99–5.15) | 3.14× (3.09–3.18) | 5.28× (5.21–5.36) | 2.61× (2.57–2.65) | 2.33× (2.29–2.37) | 0.90× (0.89–0.92) | 1.01× (0.91–1.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.60× (2.59–2.62) | 2.86× (2.84–2.87) | 3.55× (3.53–3.56) | 3.75× (3.73–3.76) | 4.06× (4.04–4.08) | 2.53× (2.52–2.54) | 2.10× (2.08–2.11) | 2.04× (2.03–2.05) | 2.77× (2.74–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.33–2.41) | 2.63× (2.55–2.72) | 3.22× (3.15–3.29) | 3.33× (3.27–3.39) | 3.54× (3.46–3.62) | 2.12× (2.08–2.16) | 1.75× (1.72–1.79) | 1.71× (1.67–1.74) | 2.32× (2.27–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.05× (1.92–2.19) | 1.82× (1.73–1.90) | 2.28× (2.20–2.37) | 2.15× (2.04–2.26) | 2.17× (2.09–2.25) | 1.08× (1.04–1.11) | 0.89× (0.86–0.92) | 0.87× (0.84–0.89) | 1.18× (1.14–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.38–1.40) | 2.70× (2.69–2.71) | 2.45× (2.41–2.49) | 2.49× (2.48–2.50) | 2.89× (2.88–2.90) | 3.18× (3.17–3.19) | 3.17× (3.15–3.19) | 1.24× (1.24–1.25) | 1.39× (1.38–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.35× (1.29–1.42) | 2.57× (2.54–2.59) | 2.26× (2.24–2.27) | 3.00× (2.97–3.02) | 2.70× (2.63–2.77) | 2.87× (2.85–2.89) | 2.86× (2.83–2.89) | 1.12× (1.11–1.13) | 1.26× (1.24–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.38–1.56) | 2.34× (2.29–2.38) | 1.89× (1.85–1.93) | 2.58× (2.53–2.63) | 2.47× (2.43–2.51) | 2.08× (2.04–2.12) | 2.07× (2.03–2.11) | 0.81× (0.80–0.83) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.53–1.54) | 3.09× (3.08–3.10) | 2.59× (2.56–2.62) | 2.70× (2.69–2.72) | 3.19× (3.12–3.27) | 3.72× (3.71–3.74) | 3.91× (3.89–3.94) | 1.41× (1.41–1.42) | 1.69× (1.68–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.40–1.44) | 2.84× (2.80–2.89) | 2.38× (2.34–2.41) | 3.32× (3.26–3.39) | 2.84× (2.80–2.89) | 3.29× (3.24–3.35) | 3.46× (3.41–3.52) | 1.25× (1.23–1.27) | 1.49× (1.47–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.65× (1.61–1.69) | 2.55× (2.50–2.61) | 1.95× (1.91–1.99) | 2.79× (2.73–2.86) | 2.54× (2.49–2.60) | 2.28× (2.24–2.32) | 2.40× (2.35–2.45) | 0.87× (0.85–0.88) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 2.84× (2.80–2.89) | 3.79× (3.65–3.94) | 2.94× (2.89–3.00) | 6.30× (6.18–6.43) | 2.45× (2.41–2.48) | 2.23× (2.20–2.27) | 1.00× (0.98–1.03) | 1.02× (0.98–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.58–0.61) | 2.39× (2.37–2.41) | 4.04× (4.00–4.08) | 2.65× (2.61–2.69) | 5.52× (5.33–5.72) | 2.09× (2.07–2.10) | 1.91× (1.89–1.92) | 0.86× (0.84–0.87) | 0.87× (0.84–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.63× (2.60–2.67) | 4.07× (4.02–4.11) | 2.68× (2.66–2.71) | 5.53× (5.43–5.62) | 2.13× (2.11–2.15) | 1.94× (1.92–1.96) | 0.87× (0.86–0.89) | 0.89× (0.85–0.93) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 1 | 2.16× (2.11–2.22) | 4.17× (4.08–4.27) | 4.51× (4.41–4.62) | 4.64× (4.53–4.75) | 4.74× (4.61–4.88) | 5.30× (5.18–5.42) | 4.80× (4.69–4.90) | 1.61× (1.57–1.65) | 1.96× (1.91–2.00) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 2 | 1.85× (1.78–1.93) | 3.51× (3.46–3.55) | 3.79× (3.75–3.83) | 4.24× (4.19–4.29) | 3.91× (3.87–3.95) | 4.24× (4.19–4.29) | 3.84× (3.80–3.88) | 1.29× (1.28–1.30) | 1.57× (1.55–1.59) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 4 | 1.82× (1.77–1.88) | 2.72× (2.65–2.80) | 2.98× (2.92–3.04) | 3.22× (3.12–3.32) | 3.04× (2.96–3.13) | 2.72× (2.66–2.77) | 2.46× (2.41–2.51) | 0.82× (0.81–0.84) | 1.00× (0.98–1.02) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.28–0.30) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.98× (10.96–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.50× (0.45–0.56) | 0.70× (0.68–0.72) | 0.58× (0.57–0.59) | 5.57× (5.43–5.71) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.36× (0.34–0.38) | 0.73× (0.63–0.84) | 0.65× (0.61–0.68) | 0.57× (0.54–0.60) | 3.77× (3.57–3.98) | 0.09× (0.08–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.68) | 0.65× (0.65–0.66) | 7.93× (7.91–7.94) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.73× (0.66–0.80) | 1.69× (1.64–1.75) | 0.85× (0.82–0.87) | 6.15× (5.96–6.34) | 0.50× (0.49–0.52) | 0.52× (0.51–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.67–0.73) | 0.80× (0.74–0.86) | 1.27× (1.22–1.32) | 0.76× (0.73–0.80) | 3.69× (3.53–3.85) | 0.29× (0.28–0.30) | 0.30× (0.29–0.32) | 0.33× (0.32–0.35) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.15× (5.12–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.52) | 0.41× (0.38–0.44) | 0.89× (0.86–0.92) | 0.72× (0.69–0.74) | 3.08× (2.97–3.19) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.64–0.67) | 0.47× (0.46–0.48) | 0.89× (0.87–0.91) | 0.83× (0.81–0.86) | 2.59× (2.54–2.65) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.81–0.83) | 5.17× (5.15–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.36× (0.33–0.39) | 0.82× (0.80–0.85) | 0.69× (0.65–0.72) | 2.83× (2.74–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.51× (0.48–0.54) | 0.81× (0.79–0.82) | 0.81× (0.77–0.85) | 2.31× (2.26–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.29× (5.28–5.30) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.20× (6.18–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.89–0.90) | 6.22× (6.19–6.24) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.39× (1.38–1.40) | 0.67× (0.67–0.68) | 6.45× (6.41–6.48) | 0.39× (0.38–0.39) | 0.41× (0.40–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 2 | 38 MB | 0.45× (0.44–0.47) | 0.45× (0.42–0.48) | 1.02× (0.98–1.06) | 0.68× (0.65–0.70) | 3.40× (3.28–3.53) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.35) | 0.16× (0.16–0.17) |
| `text` (2.57 MB → 16.5 MB, 6.4×) | 4 | 57 MB | 0.49× (0.47–0.51) | 0.48× (0.43–0.52) | 0.89× (0.86–0.91) | 0.72× (0.68–0.75) | 2.31× (2.26–2.36) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 430 | 2.34 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 794 | 2.47 | 1.06× (1.05–1.06) | 91% | 1.02× (1.02–1.03) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 956 | 3.90 | 1.65× (1.62–1.68) | 55% | 1.20× (1.20–1.21) | 1.60× (1.58–1.62) |

**Best:** mibz2 at 4 threads (956 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.07, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.63 | 0.02 | 0.09 | 0.219 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.6% |
| MTF + RLE2 | 1.53 | 17.7% |
| T-vector build | 1.51 | 17.4% |
| walk (the one each build uses) | 4.19 | 48.3% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.69 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.484 | 1.338 | (in its total) | 2.76× (2.75–2.77) |  |
| walk | 1.246 | 2.636 | (in its total) | 2.12× (2.11–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.053 | (in its total) | 1.14× (1.14–1.15) |  |
| inverse BWT, total | 1.777 | 4.028 | 3.238 | 2.27× (2.26–2.28) | 1.83× (1.81–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m30s |
| 4 threads | 32m09s ± 4m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m55s ± 3m17s |
| the scanner alone (its serial part) | 10m20s ± 19.7s |
| the workers alone, over 4 | 22m55s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 6.0 ns a byte of bz2, the workers 3.00 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,572,713 | 16,485,805 | \`ed9c9ff3cfcf1f83…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
