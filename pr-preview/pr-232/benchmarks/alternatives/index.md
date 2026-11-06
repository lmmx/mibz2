# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-232/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`49fc4a6`](<https://github.com/lmmx/mibz2/commit/49fc4a6591847776770f88cf442fcc98c400ec41>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42204537003>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.374 s | 560 | 8.1 | 2.35× (2.31–2.38) | 5.96× (5.86–6.06) | 6.18× (6.07–6.30) | 7.02× (6.87–7.17) | 6.33× (6.14–6.53) | 6.62× (6.51–6.73) | 6.15× (6.03–6.27) | 1.84× (1.80–1.88) | 2.10× (2.05–2.16) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 956 | 9.1 | 2.30× (2.17–2.44) | 5.16× (4.85–5.48) | 6.18× (5.81–6.57) | 6.12× (5.74–6.51) | 5.49× (5.12–5.88) | 10.95× (10.33–11.62) | 10.17× (9.50–10.89) | 3.04× (2.87–3.22) | 3.48× (3.27–3.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.201 s | 1,044 | 15.9 | 2.18× (2.11–2.24) | 3.47× (3.31–3.64) | 4.51× (4.32–4.70) | 4.13× (3.95–4.31) | 4.13× (3.93–4.34) | 12.37× (11.81–12.96) | 11.49× (10.90–12.11) | 3.44× (3.28–3.60) | 3.93× (3.77–4.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,389 | 3.6 | 0.55× (0.50–0.60) | 3.44× (3.14–3.77) | 4.30× (3.89–4.74) | 3.81× (3.34–4.34) | 6.42× (5.78–7.12) | 2.69× (2.44–2.97) | 2.68× (2.43–2.95) | 1.04× (0.86–1.25) | 2.06× (1.88–2.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,347 | 3.8 | 0.50× (0.48–0.53) | 2.99× (2.73–3.27) | 5.70× (5.34–6.08) | 3.58× (3.39–3.78) | 6.16× (5.89–6.45) | 2.66× (2.53–2.81) | 2.65× (2.52–2.79) | 1.03× (0.89–1.18) | 2.04× (1.95–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,342 | 3.7 | 0.51× (0.51–0.52) | 3.04× (2.85–3.24) | 5.73× (5.54–5.94) | 3.72× (3.60–3.85) | 6.38× (6.28–6.48) | 2.71× (2.65–2.76) | 2.69× (2.65–2.75) | 1.04× (0.90–1.20) | 2.08× (2.02–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 947 | 5.2 | 2.66× (2.61–2.71) | 3.36× (3.23–3.50) | 3.89× (3.77–4.00) | 4.23× (4.08–4.39) | 4.86× (4.76–4.96) | 2.60× (2.56–2.64) | 2.50× (2.30–2.72) | 1.97× (1.92–2.02) | 2.96× (2.90–3.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,285 | 6.8 | 1.92× (1.69–2.17) | 2.64× (2.22–3.15) | 3.65× (3.16–4.22) | 3.50× (2.84–4.31) | 4.81× (4.20–5.50) | 3.31× (2.87–3.81) | 3.18× (2.65–3.83) | 2.51× (2.17–2.89) | 3.77× (3.30–4.31) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,184 | 13.6 | 1.70× (1.65–1.75) | 1.93× (1.85–2.01) | 2.72× (2.62–2.83) | 2.12× (1.98–2.27) | 4.25× (4.10–4.40) | 3.18× (3.03–3.35) | 3.07× (2.77–3.39) | 2.41× (2.30–2.53) | 3.63× (3.45–3.82) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 97 | 48.1 | 1.39× (1.36–1.42) | 3.13× (3.06–3.20) | 2.81× (2.73–2.88) | 2.81× (2.75–2.86) | 3.13× (3.04–3.22) | 3.70× (3.59–3.81) | 3.51× (3.36–3.67) | 1.32× (1.27–1.36) | 1.59× (1.50–1.68) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 150 | 53.4 | 1.30× (1.25–1.36) | 2.86× (2.70–3.03) | 2.61× (2.50–2.71) | 3.26× (3.13–3.40) | 3.07× (2.94–3.21) | 5.79× (5.60–5.99) | 5.50× (5.28–5.73) | 2.06× (1.98–2.15) | 2.49× (2.36–2.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 190 | 74.2 | 1.33× (1.30–1.37) | 2.47× (2.41–2.54) | 2.44× (2.36–2.53) | 2.80× (2.72–2.88) | 3.23× (3.08–3.38) | 7.09× (6.87–7.32) | 6.73× (6.39–7.09) | 2.53× (2.41–2.64) | 3.04× (2.88–3.22) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.086 s | 112 | 41.3 | 1.49× (1.47–1.52) | 3.77× (3.70–3.83) | 3.12× (3.04–3.19) | 3.15× (3.08–3.22) | 3.54× (3.46–3.61) | 4.74× (4.67–4.82) | 4.74× (4.65–4.84) | 1.61× (1.55–1.68) | 1.87× (1.82–1.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 49.1 | 1.34× (1.27–1.42) | 3.19× (3.03–3.37) | 2.86× (2.61–3.14) | 3.74× (3.59–3.90) | 3.31× (3.17–3.46) | 7.29× (6.93–7.67) | 7.29× (6.87–7.73) | 2.48× (2.36–2.60) | 2.87× (2.70–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 203 | 69.0 | 1.37× (1.34–1.41) | 2.65× (2.50–2.81) | 2.52× (2.38–2.67) | 3.23× (3.11–3.35) | 3.38× (2.91–3.94) | 8.47× (8.14–8.82) | 8.47× (8.13–8.81) | 2.88× (2.77–2.99) | 3.33× (3.18–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 5.1 | 0.63× (0.60–0.65) | 3.17× (3.11–3.24) | 4.38× (4.27–4.50) | 3.16× (3.10–3.22) | 7.43× (6.83–8.09) | 2.36× (2.30–2.42) | 2.34× (2.30–2.39) | 0.98× (0.96–1.00) | 2.22× (2.02–2.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 971 | 5.5 | 0.50× (0.49–0.52) | 3.03× (2.94–3.12) | 5.23× (5.07–5.39) | 3.02× (2.93–3.11) | 7.30× (6.51–8.18) | 2.25× (2.18–2.31) | 2.23× (2.18–2.29) | 0.93× (0.91–0.96) | 2.12× (1.92–2.34) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 962 | 5.4 | 0.50× (0.49–0.51) | 2.99× (2.90–3.08) | 5.13× (4.98–5.29) | 2.97× (2.91–3.04) | 6.66× (6.52–6.80) | 2.21× (2.14–2.29) | 2.20× (2.13–2.27) | 0.92× (0.89–0.95) | 2.09× (1.91–2.27) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 0.052 s | 240 | 19.5 | 1.39× (1.38–1.41) | 3.38× (3.18–3.58) | 3.52× (3.45–3.60) | 3.71× (3.53–3.90) | 3.99× (3.89–4.09) | 4.12× (4.05–4.19) | 3.67× (3.59–3.76) | 1.18× (1.14–1.22) | 1.40× (1.34–1.46) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 0.033 s | 375 | 23.0 | 1.23× (1.14–1.33) | 2.78× (2.65–2.92) | 3.38× (3.17–3.61) | 3.79× (3.43–4.18) | 3.78× (3.63–3.95) | 6.28× (6.04–6.53) | 5.60× (5.36–5.86) | 1.80× (1.71–1.88) | 2.14× (2.02–2.26) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 0.030 s | 410 | 34.8 | 1.19× (1.16–1.22) | 2.15× (2.06–2.25) | 2.82× (2.55–3.13) | 3.15× (2.97–3.35) | 3.62× (3.31–3.95) | 7.13× (6.86–7.40) | 6.35× (6.11–6.61) | 2.04× (1.96–2.12) | 2.42× (2.28–2.58) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.34× (2.30–2.38) | 5.92× (5.83–6.02) | 6.15× (6.03–6.26) | 6.99× (6.84–7.14) | 6.31× (6.12–6.51) | 6.59× (6.48–6.71) | 6.12× (6.00–6.24) | 1.83× (1.80–1.87) | 2.08× (2.03–2.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.12× (2.00–2.24) | 5.21× (4.94–5.48) | 5.57× (5.28–5.88) | 6.33× (5.97–6.71) | 5.39× (5.05–5.76) | 5.74× (5.42–6.07) | 5.33× (4.99–5.69) | 1.60× (1.51–1.69) | 1.81× (1.70–1.93) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.04× (1.98–2.10) | 3.71× (3.57–3.84) | 4.08× (3.93–4.23) | 4.43× (4.30–4.58) | 3.85× (3.69–4.03) | 3.39× (3.27–3.53) | 3.15× (3.02–3.29) | 0.94× (0.91–0.98) | 1.07× (1.03–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.54–0.65) | 2.95× (2.68–3.24) | 3.76× (3.42–4.12) | 3.58× (3.12–4.11) | 5.88× (5.34–6.48) | 2.53× (2.31–2.76) | 2.52× (2.30–2.75) | 1.05× (0.86–1.28) | 1.77× (1.62–1.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.54–0.59) | 2.79× (2.68–2.89) | 4.90× (4.67–5.15) | 3.31× (3.16–3.47) | 5.60× (5.40–5.80) | 2.47× (2.36–2.58) | 2.46× (2.36–2.57) | 1.03× (0.88–1.20) | 1.73× (1.66–1.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.56–0.57) | 3.06× (3.01–3.11) | 5.04× (4.99–5.10) | 3.50× (3.39–3.62) | 5.87× (5.78–5.96) | 2.56× (2.51–2.60) | 2.55× (2.50–2.59) | 1.06× (0.91–1.25) | 1.79× (1.76–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.54× (2.49–2.58) | 2.91× (2.85–2.98) | 3.45× (3.39–3.51) | 3.99× (3.85–4.14) | 4.57× (4.48–4.66) | 2.48× (2.44–2.52) | 2.39× (2.21–2.59) | 1.90× (1.85–1.94) | 2.55× (2.51–2.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.06× (1.96–2.16) | 2.36× (2.23–2.49) | 2.84× (2.70–2.99) | 3.21× (3.03–3.40) | 3.56× (3.36–3.78) | 1.84× (1.73–1.97) | 1.78× (1.58–2.00) | 1.41× (1.32–1.51) | 1.89× (1.78–2.02) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.80× (1.74–1.87) | 1.60× (1.53–1.67) | 1.92× (1.86–1.99) | 1.90× (1.83–1.98) | 2.08× (2.01–2.15) | 0.93× (0.89–0.98) | 0.90× (0.82–0.99) | 0.72× (0.68–0.75) | 0.96× (0.92–1.00) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.35–1.41) | 3.01× (2.93–3.09) | 2.72× (2.66–2.79) | 2.77× (2.71–2.83) | 3.08× (2.99–3.17) | 3.64× (3.54–3.74) | 3.46× (3.31–3.61) | 1.31× (1.26–1.35) | 1.49× (1.39–1.60) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.28–1.35) | 2.87× (2.72–3.04) | 2.44× (2.39–2.49) | 3.18× (3.07–3.30) | 2.77× (2.70–2.84) | 3.23× (3.14–3.33) | 3.07× (2.94–3.20) | 1.16× (1.13–1.19) | 1.32× (1.26–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.42–1.48) | 2.56× (2.50–2.61) | 2.12× (2.06–2.19) | 2.73× (2.63–2.84) | 2.67× (2.54–2.80) | 2.32× (2.25–2.40) | 2.20× (2.10–2.31) | 0.84× (0.80–0.87) | 0.95× (0.91–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.46–1.51) | 3.67× (3.61–3.74) | 3.02× (2.95–3.10) | 3.11× (3.04–3.18) | 3.49× (3.42–3.56) | 4.67× (4.60–4.75) | 4.67× (4.58–4.77) | 1.60× (1.54–1.67) | 1.79× (1.75–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.32–1.43) | 3.18× (3.07–3.29) | 2.63× (2.43–2.85) | 3.55× (3.46–3.65) | 3.01× (2.95–3.07) | 3.93× (3.81–4.06) | 3.93× (3.78–4.09) | 1.35× (1.30–1.40) | 1.51× (1.45–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.62) | 3.03× (2.89–3.17) | 2.28× (2.13–2.44) | 3.09× (3.00–3.18) | 3.05× (2.62–3.56) | 2.82× (2.70–2.94) | 2.81× (2.70–2.93) | 0.97× (0.91–1.02) | 1.08× (1.03–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.70) | 2.56× (2.49–2.62) | 3.38× (3.32–3.44) | 2.85× (2.79–2.92) | 6.53× (5.99–7.10) | 2.17× (2.10–2.24) | 2.16× (2.10–2.21) | 0.98× (0.96–1.01) | 1.58× (1.36–1.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.31× (2.25–2.37) | 3.98× (3.88–4.07) | 2.74× (2.67–2.82) | 6.32× (5.64–7.09) | 2.03× (1.97–2.08) | 2.02× (1.97–2.06) | 0.92× (0.90–0.94) | 1.48× (1.27–1.72) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.48× (2.43–2.54) | 4.03× (3.93–4.13) | 2.80× (2.75–2.86) | 5.96× (5.86–6.07) | 2.07× (2.00–2.13) | 2.06× (2.00–2.12) | 0.94× (0.91–0.96) | 1.51× (1.31–1.73) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 1.38× (1.36–1.39) | 3.17× (2.99–3.36) | 3.38× (3.32–3.44) | 3.63× (3.45–3.82) | 3.89× (3.80–4.00) | 4.02× (3.95–4.09) | 3.59× (3.51–3.67) | 1.17× (1.14–1.21) | 1.29× (1.26–1.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 1.24× (1.22–1.26) | 2.71× (2.62–2.80) | 3.07× (2.98–3.17) | 3.44× (3.19–3.70) | 3.28× (3.20–3.36) | 3.37× (3.28–3.45) | 3.00× (2.92–3.09) | 0.98× (0.95–1.02) | 1.08× (1.05–1.11) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 1.38× (1.33–1.42) | 2.30× (2.22–2.38) | 2.69× (2.40–3.01) | 2.79× (2.55–3.04) | 2.85× (2.56–3.18) | 2.22× (2.15–2.31) | 1.99× (1.91–2.06) | 0.65× (0.62–0.67) | 0.72× (0.69–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.36–0.39) | 1.08× (1.08–1.08) | 0.87× (0.86–0.88) | 12.23× (12.21–12.25) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.56× (0.47–0.67) | 0.80× (0.79–0.80) | 0.66× (0.65–0.67) | 6.39× (6.36–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.69× (0.57–0.84) | 0.71× (0.70–0.72) | 0.61× (0.60–0.62) | 4.10× (4.07–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.46) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.83–0.84) | 4.45× (4.44–4.46) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.46) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.18× (8.15–8.20) | 0.65× (0.65–0.65) | 0.69× (0.68–0.70) | 0.75× (0.75–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.64× (0.61–0.67) | 0.60× (0.53–0.68) | 1.57× (1.49–1.66) | 0.77× (0.73–0.82) | 5.68× (5.39–5.99) | 0.45× (0.43–0.47) | 0.48× (0.45–0.50) | 0.52× (0.49–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.64× (0.63–0.65) | 0.73× (0.70–0.76) | 1.20× (1.18–1.22) | 0.77× (0.75–0.80) | 3.49× (3.44–3.54) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.28–5.29) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.48–0.52) | 0.44× (0.42–0.45) | 0.94× (0.90–0.97) | 0.74× (0.71–0.77) | 3.22× (3.09–3.36) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.61–0.65) | 0.45× (0.43–0.47) | 0.86× (0.83–0.88) | 0.79× (0.77–0.81) | 2.48× (2.41–2.55) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.84× (0.83–0.84) | 5.28× (5.27–5.29) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.48) | 0.39× (0.37–0.41) | 0.86× (0.83–0.88) | 0.72× (0.69–0.74) | 2.93× (2.85–3.01) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.62) | 0.49× (0.47–0.52) | 0.81× (0.80–0.83) | 0.81× (0.78–0.83) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.25) | 1.09× (1.09–1.10) | 0.77× (0.76–0.77) | 5.46× (5.45–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.77× (0.77–0.78) | 5.46× (5.44–5.47) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.77× (0.77–0.78) | 5.46× (5.44–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.50× (1.50–1.51) | 0.66× (0.66–0.67) | 6.80× (6.79–6.81) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.66–0.67) | 0.33× (0.33–0.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 33 MB | 0.49× (0.47–0.50) | 0.42× (0.38–0.47) | 1.19× (1.16–1.22) | 0.73× (0.71–0.75) | 3.75× (3.69–3.80) | 0.22× (0.22–0.23) | 0.24× (0.24–0.24) | 0.37× (0.36–0.37) | 0.18× (0.18–0.19) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 50 MB | 0.54× (0.53–0.55) | 0.53× (0.50–0.56) | 1.01× (0.98–1.03) | 0.79× (0.77–0.82) | 2.57× (2.52–2.63) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 560 | 1.79 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 956 | 2.00 | 1.15× (1.09–1.21) | 83% | 1.01× (0.99–1.03) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,044 | 3.50 | 1.94× (1.88–2.01) | 47% | 1.22× (1.17–1.26) | 1.69× (1.61–1.77) |

**Best:** mibz2 at 4 threads (1,044 MB/s); lbzip2 at 4 threads (305 MB/s); pbz2 at 4 threads (488 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.15, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.45, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.56. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.57 | 0.01 | 0.09 | 0.201 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4540 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 14.2% |
| MTF + RLE2 | 1.44 | 19.7% |
| T-vector build | 1.48 | 20.3% |
| walk (the one each build uses) | 2.91 | 39.8% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.26 | 3.5% |
| **total** | 7.32 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.308 | 0.682 | (in its total) | 2.22× (2.19–2.24) |  |
| walk | 0.606 | 1.547 | (in its total) | 2.56× (2.53–2.60) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.028 | (in its total) | 1.13× (1.11–1.16) |  |
| inverse BWT, total | 0.940 | 2.256 | 1.245 | 2.41× (2.38–2.44) | 1.34× (1.32–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m18s ± 6m38s |
| 4 threads | 29m05s ± 4m18s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m27s ± 3m05s |
| the scanner alone (its serial part) | 8m53s ± 19.7s |
| the workers alone, over 4 | 20m27s ± 3m05s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.68 ns a byte of output at 4 threads (CPU, summed over them), 1.49 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,811,638 | 12,416,792 | \`85be81a96aac1534…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4539.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.2 (bench/report.py), 10 rounds, threads 1, 2, 4
