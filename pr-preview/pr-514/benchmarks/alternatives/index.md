# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-514/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`3ae9bbe`](<https://github.com/lmmx/mibz2/commit/3ae9bbe0d8e2fe7106a46117ce770e7eee7cd2c0>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43971246240>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.563 s | 373 | 7.5 | 3.00× (2.93–3.08) | 6.50× (6.41–6.59) | 7.53× (7.35–7.71) | 7.71× (7.56–7.86) | 6.98× (6.80–7.17) | 7.76× (7.62–7.89) | 7.09× (6.91–7.27) | 2.16× (2.12–2.20) | 2.11× (2.07–2.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.321 s | 654 | 8.1 | 3.14× (3.01–3.27) | 6.13× (5.93–6.33) | 7.88× (7.59–8.17) | 7.29× (7.03–7.57) | 6.72× (6.52–6.93) | 13.72× (13.25–14.21) | 12.54× (12.17–12.93) | 3.82× (3.68–3.97) | 3.73× (3.59–3.87) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.271 s | 773 | 13.6 | 3.37× (3.32–3.41) | 4.95× (4.74–5.18) | 6.56× (6.33–6.79) | 5.77× (5.62–5.92) | 5.89× (5.63–6.16) | 16.14× (15.86–16.42) | 14.75× (14.47–15.04) | 4.50× (4.43–4.57) | 4.39× (4.33–4.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,015 | 3.0 | 0.64× (0.60–0.68) | 3.12× (2.87–3.39) | 5.52× (5.17–5.91) | 3.51× (3.31–3.72) | 6.41× (6.05–6.78) | 2.74× (2.57–2.92) | 2.68× (2.55–2.82) | 1.15× (1.07–1.23) | 1.03× (0.97–1.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 950 | 3.2 | 0.60× (0.56–0.63) | 2.92× (2.78–3.07) | 6.75× (6.48–7.04) | 3.47× (3.25–3.69) | 6.15× (5.90–6.41) | 2.63× (2.49–2.77) | 2.57× (2.48–2.67) | 1.10× (1.04–1.17) | 0.98× (0.94–1.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 937 | 3.2 | 0.61× (0.58–0.63) | 2.91× (2.83–2.99) | 6.73× (6.49–6.97) | 3.41× (3.28–3.54) | 6.22× (6.04–6.40) | 2.62× (2.51–2.73) | 2.56× (2.48–2.64) | 1.10× (1.06–1.14) | 0.98× (0.96–1.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 530 | 5.5 | 3.10× (2.96–3.26) | 3.23× (3.12–3.34) | 4.36× (4.17–4.56) | 4.11× (3.93–4.30) | 4.66× (4.46–4.88) | 3.75× (3.60–3.90) | 2.88× (2.77–2.99) | 2.13× (2.00–2.27) | 2.20× (2.08–2.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.026 s | 764 | 6.8 | 2.39× (2.18–2.62) | 2.81× (2.38–3.33) | 4.32× (3.85–4.84) | 3.47× (3.01–3.99) | 4.84× (4.35–5.38) | 5.30× (4.78–5.87) | 4.07× (3.66–4.52) | 3.01× (2.66–3.41) | 3.12× (2.80–3.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 725 | 13.8 | 2.37× (2.28–2.48) | 1.88× (1.76–2.01) | 3.25× (3.08–3.42) | 2.22× (2.10–2.35) | 4.21× (4.07–4.35) | 5.20× (5.03–5.38) | 4.00× (3.84–4.16) | 2.96× (2.78–3.15) | 3.06× (2.89–3.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.166 s | 48 | 58.7 | 1.25× (1.22–1.29) | 2.32× (2.25–2.39) | 2.17× (2.09–2.25) | 2.21× (2.12–2.31) | 2.51× (2.42–2.59) | 2.84× (2.74–2.95) | 2.87× (2.77–2.96) | 1.22× (1.18–1.26) | 1.23× (1.20–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.097 s | 82 | 63.4 | 1.23× (1.21–1.25) | 2.33× (2.28–2.38) | 2.31× (2.22–2.41) | 2.85× (2.71–3.00) | 2.69× (2.62–2.76) | 4.83× (4.72–4.95) | 4.87× (4.73–5.02) | 2.07× (2.01–2.12) | 2.09× (2.02–2.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.080 s | 100 | 85.5 | 1.41× (1.37–1.46) | 2.21× (2.11–2.32) | 2.33× (2.15–2.52) | 2.66× (2.53–2.80) | 2.88× (2.75–3.03) | 5.85× (5.65–6.05) | 5.89× (5.75–6.05) | 2.50× (2.42–2.59) | 2.52× (2.46–2.60) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.190 s | 51 | 55.8 | 1.30× (1.27–1.33) | 2.65× (2.52–2.78) | 2.31× (2.26–2.37) | 2.29× (2.24–2.34) | 2.57× (2.51–2.63) | 3.37× (3.31–3.44) | 3.47× (3.42–3.51) | 1.35× (1.33–1.38) | 1.40× (1.37–1.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.108 s | 89 | 59.8 | 1.31× (1.26–1.37) | 2.52× (2.48–2.55) | 2.38× (2.26–2.51) | 3.33× (3.14–3.54) | 2.80× (2.75–2.85) | 6.00× (5.91–6.08) | 6.16× (6.05–6.28) | 2.40× (2.36–2.45) | 2.49× (2.43–2.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.091 s | 106 | 86.6 | 1.55× (1.52–1.58) | 2.29× (2.20–2.38) | 2.33× (2.24–2.43) | 3.02× (2.87–3.17) | 2.92× (2.75–3.10) | 7.09× (6.94–7.24) | 7.29× (7.11–7.47) | 2.84× (2.78–2.91) | 2.95× (2.89–3.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 766 | 4.1 | 0.75× (0.67–0.84) | 3.13× (3.06–3.21) | 5.43× (5.27–5.59) | 3.19× (3.14–3.25) | 7.53× (7.34–7.73) | 2.45× (2.38–2.52) | 2.52× (2.42–2.62) | 1.07× (1.03–1.12) | 1.58× (1.54–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 667 | 4.8 | 0.60× (0.57–0.64) | 2.78× (2.71–2.85) | 5.77× (5.50–6.05) | 2.90× (2.81–2.98) | 6.66× (6.47–6.85) | 2.17× (2.10–2.24) | 2.23× (2.13–2.33) | 0.95× (0.90–0.99) | 1.40× (1.36–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 697 | 4.5 | 0.63× (0.57–0.68) | 2.88× (2.79–2.96) | 6.02× (5.75–6.29) | 3.00× (2.91–3.09) | 6.99× (6.83–7.16) | 2.24× (2.16–2.32) | 2.30× (2.19–2.41) | 0.98× (0.92–1.04) | 1.44× (1.40–1.49) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 0.078 s | 158 | 18.1 | 1.80× (1.75–1.85) | 3.62× (3.51–3.73) | 4.33× (4.15–4.52) | 4.34× (4.10–4.60) | 4.50× (4.34–4.66) | 4.78× (4.63–4.93) | 4.40× (4.19–4.63) | 1.39× (1.34–1.45) | 1.41× (1.37–1.45) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 0.050 s | 243 | 20.6 | 1.55× (1.49–1.62) | 3.32× (3.11–3.54) | 3.95× (3.81–4.10) | 4.57× (4.34–4.81) | 4.23× (4.10–4.36) | 7.32× (7.10–7.54) | 6.74× (6.47–7.02) | 2.13× (2.07–2.20) | 2.16× (2.06–2.26) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 0.045 s | 271 | 32.1 | 1.65× (1.60–1.71) | 2.71× (2.59–2.84) | 3.44× (3.24–3.66) | 3.94× (3.56–4.37) | 4.14× (3.94–4.36) | 8.12× (7.91–8.33) | 7.48× (7.26–7.71) | 2.37× (2.31–2.42) | 2.39× (2.32–2.47) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.00× (2.93–3.07) | 6.47× (6.38–6.56) | 7.50× (7.32–7.68) | 7.69× (7.54–7.84) | 6.96× (6.79–7.15) | 7.73× (7.60–7.87) | 7.07× (6.89–7.25) | 2.16× (2.12–2.20) | 2.10× (2.06–2.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.01× (2.94–3.07) | 6.28× (6.09–6.47) | 7.53× (7.33–7.73) | 7.59× (7.43–7.75) | 6.69× (6.55–6.83) | 7.18× (7.05–7.32) | 6.57× (6.43–6.70) | 2.00× (1.96–2.05) | 1.95× (1.91–1.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.22× (3.17–3.28) | 5.09× (4.90–5.29) | 6.21× (5.96–6.47) | 5.95× (5.79–6.11) | 5.53× (5.27–5.80) | 4.26× (4.18–4.34) | 3.90× (3.81–3.98) | 1.19× (1.17–1.21) | 1.16× (1.14–1.17) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.63–0.71) | 2.81× (2.63–3.01) | 5.00× (4.72–5.31) | 3.31× (3.13–3.50) | 5.99× (5.68–6.32) | 2.60× (2.45–2.77) | 2.55× (2.43–2.68) | 1.14× (1.06–1.21) | 0.95× (0.89–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.60–0.67) | 2.67× (2.55–2.80) | 6.15× (5.93–6.38) | 3.27× (3.09–3.47) | 5.73× (5.51–5.95) | 2.48× (2.36–2.61) | 2.43× (2.34–2.52) | 1.08× (1.03–1.14) | 0.90× (0.86–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.64× (0.61–0.66) | 3.05× (2.95–3.15) | 6.16× (5.97–6.35) | 3.25× (3.13–3.38) | 5.82× (5.66–5.99) | 2.50× (2.39–2.60) | 2.44× (2.37–2.52) | 1.09× (1.05–1.13) | 0.91× (0.89–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.01× (2.87–3.15) | 2.92× (2.83–3.01) | 4.13× (3.99–4.27) | 3.97× (3.81–4.14) | 4.50× (4.31–4.70) | 3.62× (3.49–3.76) | 2.79× (2.69–2.90) | 2.08× (1.96–2.21) | 2.04× (1.94–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.58× (2.50–2.67) | 2.59× (2.49–2.69) | 3.70× (3.58–3.83) | 3.31× (3.21–3.42) | 3.80× (3.63–3.98) | 2.94× (2.84–3.05) | 2.27× (2.18–2.36) | 1.69× (1.58–1.81) | 1.66× (1.58–1.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.46× (2.39–2.53) | 1.71× (1.65–1.77) | 2.51× (2.42–2.60) | 2.05× (1.99–2.12) | 2.30× (2.21–2.39) | 1.49× (1.43–1.54) | 1.15× (1.11–1.19) | 0.85× (0.81–0.90) | 0.84× (0.80–0.88) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.25× (1.22–1.28) | 2.29× (2.22–2.36) | 2.14× (2.06–2.22) | 2.20× (2.11–2.29) | 2.49× (2.41–2.58) | 2.83× (2.73–2.93) | 2.85× (2.76–2.94) | 1.22× (1.17–1.26) | 1.20× (1.17–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.21–1.23) | 2.32× (2.26–2.38) | 2.13× (2.06–2.21) | 2.69× (2.60–2.77) | 2.39× (2.33–2.44) | 2.62× (2.57–2.67) | 2.64× (2.59–2.69) | 1.13× (1.10–1.15) | 1.11× (1.10–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.59× (1.53–1.65) | 2.42× (2.28–2.58) | 2.18× (1.97–2.42) | 2.68× (2.52–2.85) | 2.57× (2.41–2.74) | 1.93× (1.87–1.99) | 1.95× (1.89–2.00) | 0.83× (0.81–0.85) | 0.82× (0.80–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.30× (1.27–1.33) | 2.60× (2.48–2.73) | 2.28× (2.23–2.33) | 2.28× (2.23–2.33) | 2.56× (2.50–2.61) | 3.35× (3.29–3.42) | 3.44× (3.40–3.49) | 1.35× (1.32–1.37) | 1.37× (1.34–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.29× (1.27–1.31) | 2.51× (2.46–2.57) | 2.23× (2.14–2.32) | 3.04× (2.96–3.12) | 2.51× (2.47–2.55) | 3.16× (3.12–3.20) | 3.25× (3.21–3.28) | 1.27× (1.25–1.30) | 1.29× (1.26–1.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.70× (1.65–1.74) | 2.49× (2.38–2.60) | 2.06× (1.97–2.16) | 2.78× (2.63–2.95) | 2.59× (2.47–2.71) | 2.21× (2.15–2.28) | 2.27× (2.22–2.33) | 0.89× (0.87–0.91) | 0.90× (0.87–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.77× (0.70–0.86) | 2.57× (2.48–2.68) | 4.42× (4.32–4.53) | 2.93× (2.86–3.01) | 6.74× (6.58–6.92) | 2.27× (2.20–2.34) | 2.33× (2.24–2.43) | 1.06× (1.01–1.10) | 0.89× (0.86–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.65× (0.62–0.69) | 2.27× (2.20–2.34) | 4.92× (4.73–5.10) | 2.71× (2.63–2.79) | 5.98× (5.80–6.16) | 2.00× (1.94–2.07) | 2.05× (1.97–2.15) | 0.93× (0.88–0.98) | 0.79× (0.75–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.66× (0.62–0.71) | 2.71× (2.62–2.80) | 5.11× (5.02–5.20) | 2.86× (2.78–2.94) | 6.39× (6.25–6.53) | 2.10× (2.04–2.17) | 2.16× (2.07–2.25) | 0.98× (0.92–1.04) | 0.83× (0.79–0.87) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 1.78× (1.73–1.83) | 3.51× (3.40–3.62) | 4.17× (3.99–4.35) | 4.26× (4.02–4.51) | 4.42× (4.26–4.58) | 4.69× (4.55–4.84) | 4.32× (4.11–4.54) | 1.38× (1.33–1.44) | 1.33× (1.30–1.36) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 1.68× (1.64–1.73) | 3.40× (3.22–3.60) | 3.99× (3.86–4.11) | 4.26× (4.15–4.38) | 3.92× (3.86–3.99) | 4.11× (4.04–4.17) | 3.79× (3.68–3.89) | 1.21× (1.17–1.25) | 1.16× (1.13–1.19) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 1.98× (1.92–2.04) | 3.10× (2.89–3.32) | 3.72× (3.51–3.94) | 3.66× (3.32–4.03) | 3.70× (3.52–3.90) | 2.63× (2.57–2.70) | 2.43× (2.32–2.54) | 0.78× (0.75–0.80) | 0.75× (0.73–0.76) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.08× (11.05–11.10) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.33–0.35) | 0.44× (0.40–0.49) | 0.70× (0.67–0.72) | 0.59× (0.57–0.61) | 5.56× (5.40–5.73) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.35× (0.33–0.36) | 0.51× (0.46–0.56) | 0.61× (0.59–0.63) | 0.54× (0.52–0.56) | 3.58× (3.45–3.72) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.40–0.44) | 1.71× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.00–8.09) | 0.65× (0.64–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.70) | 0.67× (0.62–0.73) | 1.63× (1.60–1.66) | 0.82× (0.80–0.83) | 5.93× (5.82–6.04) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.63–0.67) | 0.71× (0.63–0.80) | 1.15× (1.11–1.19) | 0.71× (0.69–0.74) | 3.49× (3.39–3.59) | 0.27× (0.26–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.41× (0.38–0.43) | 0.91× (0.88–0.95) | 0.72× (0.68–0.75) | 3.14× (3.02–3.26) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.66) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.80× (0.78–0.82) | 2.51× (2.45–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.48) | 0.33× (0.31–0.35) | 0.82× (0.80–0.84) | 0.70× (0.68–0.73) | 2.81× (2.74–2.89) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.59× (0.57–0.60) | 0.49× (0.46–0.52) | 0.79× (0.78–0.81) | 0.78× (0.75–0.82) | 2.28× (2.24–2.33) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.34) | 0.36× (0.35–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.65–0.66) | 6.40× (6.39–6.42) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.38× (0.34–0.42) | 1.04× (1.00–1.08) | 0.68× (0.66–0.70) | 3.29× (3.21–3.38) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 58 MB | 0.47× (0.47–0.48) | 0.45× (0.41–0.49) | 0.87× (0.86–0.89) | 0.70× (0.68–0.72) | 2.24× (2.21–2.27) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 373 | 2.69 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 654 | 2.90 | 1.08× (1.04–1.11) | 88% | 1.04× (1.02–1.07) | 1.08× (1.06–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 773 | 4.88 | 1.81× (1.77–1.86) | 52% | 1.43× (1.37–1.49) | 1.95× (1.91–2.00) |

**Best:** mibz2 at 4 threads (773 MB/s); lbzip2 at 4 threads (157 MB/s); pbz2 at 4 threads (230 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.13, lbzip2 1t 0.05, lbzip2 2t 0.21, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.31. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.81 | 0.03 | 0.08 | 0.271 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2795 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m12s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 10.2% |
| MTF + RLE2 | 1.41 | 14.2% |
| T-vector build | 1.41 | 14.2% |
| walk (the one each build uses) | 5.63 | 56.8% |
| of which: segment assembly | 0.29 | 2.9% |
| RLE1 | 0.23 | 2.3% |
| CRC | 0.13 | 1.3% |
| **total** | 9.91 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.494 | 1.275 | (in its total) | 2.58× (2.57–2.59) |  |
| walk | 1.992 | 3.064 | (in its total) | 1.54× (1.53–1.56) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.107 | 0.118 | (in its total) | 1.11× (1.04–1.18) |  |
| inverse BWT, total | 2.594 | 4.454 | 2.620 | 1.72× (1.71–1.74) | 1.01× (1.01–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h22m ± 9m31s |
| 4 threads | 39m34s ± 5m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m45s ± 4m13s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 29m45s ± 4m13s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.90 ns a byte of output at 4 threads (CPU, summed over them), 2.20 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,408,661 | 12,232,367 | \`2d4782182c8b0988…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 2794.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
