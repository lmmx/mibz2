# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-860/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`37fbd77`](<https://github.com/lmmx/mibz2/commit/37fbd77b06edbfc0e01d1355bebdfd218861f099>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45533501530>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.495 s | 423 | 7.7 | 3.08× (3.07–3.10) | 7.01× (6.99–7.04) | 7.26× (7.20–7.31) | 7.88× (7.82–7.93) | 7.11× (7.06–7.16) | 7.69× (7.65–7.73) | 7.09× (7.04–7.14) | 2.27× (2.26–2.28) | 2.64× (2.62–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.273 s | 768 | 8.2 | 3.13× (3.03–3.22) | 6.70× (6.41–7.01) | 7.73× (7.44–8.02) | 7.51× (7.21–7.82) | 6.81× (6.57–7.06) | 13.66× (13.24–14.09) | 12.59× (12.20–12.99) | 4.04× (3.90–4.18) | 4.69× (4.54–4.85) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 935 | 12.8 | 3.24× (3.19–3.29) | 4.98× (4.88–5.09) | 6.22× (6.08–6.37) | 5.64× (5.57–5.70) | 5.73× (5.65–5.81) | 17.04× (16.85–17.23) | 15.70× (15.50–15.91) | 5.03× (4.99–5.08) | 5.85× (5.80–5.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 978 | 3.6 | 0.60× (0.59–0.60) | 3.70× (3.54–3.87) | 4.49× (4.37–4.61) | 3.59× (3.56–3.61) | 6.18× (6.14–6.22) | 2.99× (2.96–3.01) | 2.67× (2.65–2.70) | 1.02× (1.00–1.04) | 1.45× (1.33–1.58) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 935 | 4.0 | 0.55× (0.54–0.57) | 2.85× (2.80–2.90) | 5.76× (5.67–5.85) | 3.40× (3.35–3.45) | 5.83× (5.74–5.93) | 2.81× (2.77–2.86) | 2.52× (2.48–2.56) | 0.96× (0.94–0.98) | 1.37× (1.25–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 897 | 4.1 | 0.54× (0.54–0.55) | 2.78× (2.74–2.83) | 5.65× (5.52–5.78) | 3.30× (3.26–3.34) | 5.72× (5.63–5.81) | 2.73× (2.69–2.78) | 2.45× (2.41–2.48) | 0.93× (0.92–0.95) | 1.33× (1.23–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 570 | 6.0 | 2.59× (2.56–2.62) | 2.97× (2.94–3.01) | 3.88× (3.74–4.01) | 3.77× (3.74–3.81) | 4.11× (4.08–4.14) | 2.51× (2.48–2.54) | 2.09× (2.07–2.11) | 2.01× (1.99–2.02) | 2.96× (2.93–2.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 891 | 7.3 | 2.21× (2.01–2.44) | 2.81× (2.49–3.18) | 4.08× (3.63–4.58) | 3.41× (3.06–3.79) | 4.72× (4.32–5.16) | 3.76× (3.44–4.10) | 3.13× (2.86–3.42) | 3.00× (2.75–3.28) | 4.43× (4.05–4.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 848 | 13.7 | 2.09× (2.00–2.18) | 2.17× (1.99–2.36) | 3.22× (3.02–3.42) | 2.37× (2.24–2.51) | 4.22× (4.07–4.38) | 3.65× (3.52–3.78) | 3.03× (2.93–3.15) | 2.92× (2.82–3.02) | 4.30× (4.15–4.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 55.1 | 1.35× (1.34–1.36) | 2.67× (2.65–2.70) | 2.40× (2.37–2.43) | 2.43× (2.41–2.46) | 2.81× (2.79–2.84) | 3.12× (3.08–3.15) | 3.07× (3.03–3.10) | 1.23× (1.22–1.24) | 1.43× (1.42–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 101 | 60.5 | 1.31× (1.27–1.36) | 2.52× (2.43–2.62) | 2.41× (2.32–2.50) | 3.10× (2.98–3.23) | 2.96× (2.86–3.06) | 5.16× (4.97–5.35) | 5.07× (4.90–5.25) | 2.04× (1.97–2.11) | 2.37× (2.28–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.065 s | 122 | 82.3 | 1.38× (1.35–1.42) | 2.23× (2.15–2.31) | 2.17× (2.10–2.24) | 2.73× (2.63–2.82) | 3.00× (2.91–3.08) | 6.30× (6.15–6.45) | 6.19× (6.05–6.34) | 2.49× (2.43–2.55) | 2.90× (2.81–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 50.6 | 1.45× (1.42–1.47) | 3.00× (2.97–3.04) | 2.53× (2.49–2.57) | 2.60× (2.57–2.62) | 3.04× (3.01–3.08) | 3.61× (3.58–3.64) | 3.81× (3.79–3.83) | 1.43× (1.42–1.44) | 1.71× (1.70–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.5 | 1.34× (1.29–1.39) | 2.81× (2.66–2.96) | 2.48× (2.41–2.56) | 3.45× (3.29–3.63) | 3.07× (2.98–3.17) | 5.97× (5.78–6.16) | 6.31× (6.12–6.49) | 2.37× (2.30–2.45) | 2.82× (2.74–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 130 | 80.7 | 1.47× (1.44–1.50) | 2.38× (2.31–2.46) | 2.36× (2.28–2.44) | 3.23× (3.10–3.37) | 2.97× (2.93–3.02) | 7.15× (7.07–7.24) | 7.56× (7.44–7.68) | 2.84× (2.78–2.91) | 3.39× (3.33–3.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 717 | 5.2 | 0.66× (0.65–0.67) | 3.04× (2.89–3.21) | 4.35× (4.29–4.41) | 3.08× (3.05–3.11) | 6.71× (6.62–6.79) | 2.50× (2.47–2.53) | 2.28× (2.25–2.31) | 0.98× (0.96–0.99) | 1.50× (1.48–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 5.8 | 0.54× (0.53–0.55) | 2.76× (2.72–2.80) | 4.80× (4.63–4.98) | 2.88× (2.85–2.91) | 6.20× (6.12–6.28) | 2.31× (2.28–2.35) | 2.12× (2.09–2.14) | 0.91× (0.89–0.92) | 1.39× (1.37–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 652 | 5.8 | 0.53× (0.53–0.54) | 2.71× (2.67–2.74) | 4.64× (4.58–4.70) | 2.81× (2.78–2.84) | 6.07× (5.98–6.15) | 2.26× (2.24–2.29) | 2.07× (2.04–2.09) | 0.88× (0.87–0.89) | 1.36× (1.34–1.37) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.075 s | 186 | 17.9 | 2.00× (1.97–2.03) | 4.14× (4.04–4.24) | 4.55× (4.40–4.72) | 4.53× (4.43–4.63) | 4.69× (4.60–4.79) | 5.30× (5.19–5.42) | 4.78× (4.68–4.88) | 1.57× (1.54–1.60) | 1.91× (1.87–1.96) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.049 s | 284 | 21.9 | 1.63× (1.56–1.70) | 3.45× (3.24–3.67) | 4.24× (4.12–4.36) | 4.52× (4.32–4.73) | 4.22× (4.07–4.37) | 7.80× (7.53–8.09) | 7.03× (6.78–7.30) | 2.31× (2.22–2.39) | 2.81× (2.72–2.91) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.044 s | 316 | 33.4 | 1.64× (1.55–1.74) | 2.62× (2.52–2.73) | 3.65× (3.56–3.75) | 3.61× (3.47–3.75) | 3.90× (3.81–3.99) | 8.90× (8.66–9.14) | 8.02× (7.84–8.20) | 2.63× (2.56–2.70) | 3.21× (3.12–3.30) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.08× (3.06–3.09) | 6.98× (6.95–7.00) | 7.22× (7.17–7.28) | 7.85× (7.80–7.90) | 7.09× (7.04–7.13) | 7.66× (7.62–7.70) | 7.06× (7.01–7.11) | 2.27× (2.26–2.28) | 2.62× (2.60–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.02× (2.99–3.05) | 6.77× (6.69–6.84) | 7.12× (7.04–7.21) | 7.81× (7.66–7.96) | 6.73× (6.67–6.80) | 7.15× (7.12–7.19) | 6.59× (6.55–6.63) | 2.12× (2.10–2.14) | 2.45× (2.43–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.10× (3.03–3.16) | 5.21× (5.12–5.31) | 5.64× (5.54–5.75) | 5.91× (5.84–5.97) | 5.33× (5.20–5.45) | 4.60× (4.56–4.63) | 4.24× (4.20–4.27) | 1.36× (1.34–1.38) | 1.57× (1.56–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.28× (3.25–3.30) | 4.11× (4.04–4.19) | 3.36× (3.34–3.38) | 5.72× (5.68–5.75) | 2.81× (2.79–2.83) | 2.52× (2.50–2.54) | 1.01× (0.99–1.04) | 1.04× (1.02–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.89× (2.84–2.94) | 5.07× (5.01–5.12) | 3.10× (3.06–3.14) | 5.24× (5.16–5.31) | 2.56× (2.53–2.60) | 2.30× (2.27–2.33) | 0.93× (0.91–0.94) | 0.95× (0.93–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.57) | 3.13× (3.06–3.20) | 5.00× (4.90–5.11) | 3.06× (3.02–3.10) | 5.19× (5.10–5.27) | 2.53× (2.49–2.57) | 2.27× (2.23–2.30) | 0.91× (0.90–0.93) | 0.94× (0.92–0.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.48–2.53) | 2.74× (2.71–2.78) | 3.51× (3.40–3.63) | 3.62× (3.59–3.65) | 3.94× (3.91–3.97) | 2.43× (2.40–2.45) | 2.03× (2.01–2.05) | 1.95× (1.93–1.97) | 2.66× (2.63–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.22–2.32) | 2.52× (2.44–2.60) | 3.10× (3.05–3.16) | 3.17× (3.11–3.24) | 3.40× (3.33–3.47) | 2.00× (1.96–2.05) | 1.67× (1.64–1.71) | 1.61× (1.58–1.64) | 2.20× (2.17–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.01–2.20) | 1.82× (1.76–1.89) | 2.30× (2.23–2.36) | 2.13× (2.05–2.22) | 2.22× (2.15–2.29) | 1.07× (1.03–1.10) | 0.89× (0.86–0.92) | 0.86× (0.83–0.89) | 1.17× (1.13–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.33–1.35) | 2.60× (2.58–2.63) | 2.34× (2.32–2.37) | 2.41× (2.39–2.44) | 2.79× (2.76–2.81) | 3.09× (3.05–3.12) | 3.03× (3.00–3.07) | 1.23× (1.22–1.24) | 1.38× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.31) | 2.49× (2.46–2.52) | 2.20× (2.18–2.22) | 2.92× (2.89–2.95) | 2.58× (2.55–2.60) | 2.79× (2.76–2.81) | 2.74× (2.71–2.76) | 1.11× (1.09–1.13) | 1.24× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.46–1.51) | 2.32× (2.26–2.39) | 1.90× (1.87–1.94) | 2.60× (2.55–2.65) | 2.49× (2.43–2.55) | 2.04× (2.00–2.07) | 2.00× (1.96–2.04) | 0.81× (0.80–0.83) | 0.91× (0.89–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.47) | 2.95× (2.92–2.98) | 2.48× (2.44–2.52) | 2.57× (2.55–2.60) | 3.02× (2.98–3.05) | 3.57× (3.54–3.60) | 3.77× (3.75–3.80) | 1.43× (1.42–1.44) | 1.67× (1.67–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.30–1.38) | 2.76× (2.65–2.88) | 2.28× (2.21–2.36) | 3.17× (3.05–3.29) | 2.72× (2.63–2.81) | 3.15× (3.03–3.26) | 3.32× (3.22–3.43) | 1.26× (1.22–1.30) | 1.48× (1.43–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.47–1.60) | 2.53× (2.48–2.58) | 1.94× (1.92–1.97) | 2.81× (2.75–2.88) | 2.57× (2.54–2.59) | 2.23× (2.21–2.24) | 2.35× (2.33–2.37) | 0.89× (0.88–0.90) | 1.04× (1.03–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.68× (2.63–2.73) | 3.54× (3.50–3.58) | 2.82× (2.78–2.85) | 5.98× (5.90–6.06) | 2.30× (2.27–2.33) | 2.12× (2.09–2.15) | 0.98× (0.96–0.99) | 0.96× (0.92–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.36× (2.33–2.39) | 4.05× (3.98–4.12) | 2.60× (2.57–2.63) | 5.35× (5.28–5.42) | 2.05× (2.02–2.08) | 1.89× (1.86–1.91) | 0.87× (0.86–0.88) | 0.86× (0.82–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.56–0.57) | 2.55× (2.52–2.58) | 4.00× (3.97–4.02) | 2.59× (2.57–2.61) | 5.33× (5.27–5.39) | 2.04× (2.02–2.06) | 1.88× (1.86–1.90) | 0.87× (0.86–0.88) | 0.86× (0.82–0.90) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.97× (1.95–2.00) | 4.01× (3.93–4.08) | 4.41× (4.27–4.55) | 4.43× (4.34–4.53) | 4.60× (4.50–4.69) | 5.19× (5.08–5.30) | 4.68× (4.59–4.77) | 1.55× (1.52–1.58) | 1.85× (1.81–1.88) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.73× (1.70–1.76) | 3.40× (3.31–3.49) | 3.73× (3.69–3.78) | 4.14× (4.08–4.19) | 3.80× (3.75–3.85) | 4.18× (4.13–4.24) | 3.77× (3.73–3.82) | 1.25× (1.23–1.27) | 1.49× (1.47–1.51) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.75× (1.67–1.82) | 2.81× (2.74–2.87) | 3.09× (3.02–3.17) | 3.35× (3.29–3.40) | 3.16× (3.11–3.20) | 2.78× (2.75–2.82) | 2.51× (2.48–2.54) | 0.83× (0.82–0.84) | 0.99× (0.98–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.78) | 11.02× (10.99–11.05) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.50× (0.42–0.58) | 0.72× (0.70–0.74) | 0.60× (0.58–0.62) | 5.70× (5.54–5.88) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.36× (0.34–0.38) | 0.62× (0.51–0.76) | 0.64× (0.59–0.69) | 0.57× (0.53–0.61) | 3.71× (3.47–3.96) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.85× (4.85–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.64–0.66) | 7.93× (7.91–7.96) | 0.65× (0.64–0.65) | 0.68× (0.67–0.68) | 0.73× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.72× (0.67–0.78) | 1.70× (1.66–1.74) | 0.85× (0.83–0.88) | 6.15× (6.00–6.30) | 0.50× (0.49–0.51) | 0.52× (0.51–0.53) | 0.57× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.68× (0.64–0.72) | 0.78× (0.72–0.85) | 1.23× (1.16–1.30) | 0.76× (0.72–0.79) | 3.61× (3.42–3.80) | 0.28× (0.27–0.30) | 0.30× (0.28–0.31) | 0.32× (0.30–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.43) | 0.89× (0.87–0.92) | 0.70× (0.68–0.73) | 3.09× (3.00–3.19) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.66× (0.65–0.68) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.82–0.85) | 2.56× (2.51–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.17–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.38× (0.36–0.40) | 0.85× (0.83–0.87) | 0.69× (0.66–0.72) | 2.92× (2.85–2.98) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.50× (0.47–0.53) | 0.80× (0.78–0.82) | 0.80× (0.76–0.83) | 2.32× (2.26–2.37) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.29–5.32) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.90) | 6.20× (6.19–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.24) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.66–0.67) | 6.41× (6.39–6.44) | 0.39× (0.38–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.41× (0.37–0.46) | 1.10× (1.07–1.14) | 0.71× (0.69–0.73) | 3.47× (3.38–3.56) | 0.21× (0.20–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.51× (0.47–0.56) | 0.92× (0.90–0.93) | 0.76× (0.75–0.78) | 2.33× (2.28–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 423 | 2.37 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 768 | 2.54 | 1.07× (1.06–1.08) | 89% | 1.04× (1.03–1.05) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 935 | 3.96 | 1.67× (1.65–1.69) | 55% | 1.25× (1.23–1.27) | 1.68× (1.65–1.71) |

**Best:** mibz2 at 4 threads (935 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (291 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.05, lbzip2 1t 0.09, lbzip2 2t 0.29, lbzip2 4t 0.45, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.09 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.93 | 10.5% |
| MTF + RLE2 | 1.55 | 17.6% |
| T-vector build | 1.54 | 17.4% |
| walk (the one each build uses) | 4.24 | 48.1% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.302 | (in its total) | 2.68× (2.67–2.70) |  |
| walk | 1.241 | 2.653 | (in its total) | 2.14× (2.13–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.056 | (in its total) | 1.24× (1.24–1.25) |  |
| inverse BWT, total | 1.771 | 4.012 | 3.325 | 2.26× (2.25–2.27) | 1.88× (1.87–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h12m ± 8m39s |
| 4 threads | 32m33s ± 4m35s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m39s ± 3m19s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 23m39s ± 3m19s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 3.10 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3242.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
