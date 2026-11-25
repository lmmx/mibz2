# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-889/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`84dcd63`](<https://github.com/lmmx/mibz2/commit/84dcd6313d055e7fd5eb1f93c4831e4caea20be6>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45596186138>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.495 s | 424 | 7.7 | 3.08× (3.06–3.09) | 7.11× (7.04–7.17) | 7.30× (7.25–7.34) | 7.92× (7.86–7.98) | 7.16× (7.10–7.22) | 7.76× (7.71–7.81) | 7.11× (7.06–7.16) | 2.29× (2.27–2.31) | 2.63× (2.61–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.271 s | 775 | 8.2 | 3.13× (3.04–3.21) | 6.79× (6.57–7.01) | 7.78× (7.58–7.97) | 7.69× (7.47–7.92) | 7.03× (6.79–7.28) | 13.99× (13.70–14.28) | 12.82× (12.52–13.12) | 4.12× (4.03–4.22) | 4.74× (4.62–4.85) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.227 s | 924 | 12.9 | 3.17× (3.12–3.24) | 4.97× (4.82–5.12) | 6.13× (5.96–6.30) | 5.54× (5.45–5.63) | 5.67× (5.49–5.86) | 16.89× (16.56–17.23) | 15.48× (15.16–15.81) | 4.98× (4.88–5.08) | 5.72× (5.59–5.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 957 | 3.7 | 0.59× (0.58–0.60) | 3.74× (3.58–3.91) | 4.39× (4.31–4.47) | 3.51× (3.45–3.56) | 6.10× (6.01–6.20) | 2.90× (2.84–2.96) | 2.60× (2.55–2.65) | 1.01× (0.99–1.03) | 1.47× (1.44–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 922 | 4.0 | 0.55× (0.50–0.61) | 2.79× (2.70–2.89) | 5.68× (5.47–5.90) | 3.34× (3.24–3.45) | 5.73× (5.55–5.91) | 2.76× (2.67–2.85) | 2.47× (2.40–2.56) | 0.96× (0.93–0.99) | 1.40× (1.35–1.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 893 | 4.0 | 0.54× (0.53–0.55) | 2.78× (2.72–2.83) | 5.66× (5.51–5.82) | 3.32× (3.26–3.37) | 5.74× (5.63–5.84) | 2.73× (2.68–2.77) | 2.45× (2.41–2.49) | 0.95× (0.92–0.98) | 1.38× (1.36–1.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 582 | 5.9 | 2.61× (2.59–2.63) | 3.01× (2.97–3.05) | 3.85× (3.80–3.90) | 3.81× (3.78–3.85) | 4.17× (4.12–4.22) | 2.54× (2.51–2.58) | 2.10× (2.08–2.13) | 2.03× (1.99–2.07) | 2.99× (2.96–3.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 907 | 7.0 | 2.17× (1.92–2.45) | 2.91× (2.45–3.46) | 4.03× (3.54–4.60) | 3.36× (2.95–3.83) | 4.77× (4.21–5.40) | 3.77× (3.31–4.30) | 3.12× (2.74–3.55) | 3.01× (2.63–3.44) | 4.44× (3.90–5.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 845 | 14.0 | 2.00× (1.93–2.07) | 2.11× (1.96–2.28) | 3.11× (3.01–3.22) | 2.30× (2.20–2.41) | 4.29× (4.18–4.41) | 3.67× (3.55–3.80) | 3.04× (2.94–3.14) | 2.93× (2.86–3.00) | 4.32× (4.17–4.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.126 s | 63 | 52.0 | 1.43× (1.36–1.50) | 2.82× (2.80–2.85) | 2.62× (2.49–2.75) | 2.61× (2.60–2.63) | 2.97× (2.95–3.00) | 3.27× (3.24–3.29) | 3.24× (3.20–3.27) | 1.27× (1.25–1.29) | 1.45× (1.44–1.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.076 s | 106 | 58.3 | 1.36× (1.32–1.40) | 2.74× (2.62–2.86) | 2.61× (2.49–2.74) | 3.31× (3.19–3.43) | 3.18× (3.07–3.29) | 5.44× (5.30–5.58) | 5.39× (5.24–5.54) | 2.11× (2.05–2.18) | 2.42× (2.35–2.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.063 s | 128 | 78.9 | 1.45× (1.41–1.49) | 2.36× (2.32–2.39) | 2.41× (2.06–2.82) | 2.92× (2.75–3.10) | 3.16× (3.11–3.21) | 6.61× (6.52–6.70) | 6.55× (6.51–6.59) | 2.57× (2.51–2.63) | 2.94× (2.89–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.140 s | 68 | 48.1 | 1.49× (1.47–1.50) | 3.20× (3.14–3.26) | 2.67× (2.63–2.70) | 2.76× (2.74–2.77) | 3.21× (3.12–3.30) | 3.76× (3.73–3.78) | 3.97× (3.94–4.00) | 1.46× (1.43–1.49) | 1.75× (1.73–1.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.085 s | 113 | 54.3 | 1.34× (1.27–1.43) | 2.90× (2.68–3.14) | 2.54× (2.36–2.73) | 3.50× (3.32–3.68) | 3.11× (2.95–3.28) | 6.04× (5.69–6.41) | 6.38× (5.99–6.78) | 2.34× (2.20–2.50) | 2.82× (2.65–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 77.5 | 1.53× (1.50–1.56) | 2.52× (2.45–2.59) | 2.43× (2.38–2.47) | 3.28× (3.20–3.36) | 3.12× (3.06–3.18) | 7.51× (7.40–7.61) | 7.93× (7.79–8.08) | 2.91× (2.82–3.01) | 3.51× (3.42–3.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 733 | 5.1 | 0.67× (0.66–0.68) | 3.08× (2.96–3.21) | 4.40× (4.34–4.46) | 3.12× (3.09–3.16) | 7.00× (6.64–7.37) | 2.54× (2.51–2.57) | 2.32× (2.29–2.35) | 1.04× (0.95–1.13) | 1.52× (1.50–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 5.8 | 0.54× (0.53–0.56) | 2.74× (2.69–2.79) | 4.95× (4.71–5.20) | 2.87× (2.82–2.92) | 6.19× (6.08–6.30) | 2.30× (2.27–2.34) | 2.11× (2.07–2.15) | 0.94× (0.86–1.03) | 1.38× (1.35–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 5.8 | 0.54× (0.54–0.54) | 2.71× (2.69–2.73) | 4.70× (4.55–4.85) | 2.83× (2.80–2.86) | 6.11× (6.03–6.18) | 2.27× (2.25–2.29) | 2.08× (2.06–2.09) | 0.93× (0.85–1.01) | 1.36× (1.35–1.37) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 0.062 s | 194 | 17.2 | 2.05× (2.02–2.08) | 4.27× (4.20–4.34) | 4.72× (4.65–4.80) | 4.73× (4.68–4.79) | 5.02× (4.96–5.07) | 5.50× (5.44–5.55) | 4.97× (4.91–5.02) | 1.60× (1.57–1.62) | 1.97× (1.93–2.02) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 0.043 s | 276 | 22.5 | 1.53× (1.31–1.79) | 3.25× (2.95–3.58) | 4.02× (3.54–4.56) | 4.22× (3.73–4.78) | 4.08× (3.62–4.60) | 7.30× (6.43–8.29) | 6.60× (5.78–7.53) | 2.12× (1.87–2.41) | 2.62× (2.31–2.97) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 0.038 s | 316 | 34.7 | 1.64× (1.53–1.75) | 2.58× (2.44–2.73) | 3.61× (3.50–3.73) | 3.75× (3.62–3.88) | 3.92× (3.80–4.05) | 8.87× (8.64–9.10) | 8.01× (7.84–8.19) | 2.58× (2.52–2.63) | 3.18× (3.08–3.29) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.07× (3.05–3.08) | 7.08× (7.02–7.14) | 7.26× (7.21–7.31) | 7.89× (7.83–7.95) | 7.14× (7.08–7.20) | 7.73× (7.68–7.78) | 7.09× (7.04–7.14) | 2.28× (2.27–2.30) | 2.61× (2.59–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.98× (2.96–3.01) | 6.80× (6.68–6.93) | 7.17× (7.09–7.26) | 7.89× (7.66–8.13) | 6.86× (6.60–7.14) | 7.24× (7.19–7.29) | 6.64× (6.56–6.72) | 2.14× (2.12–2.16) | 2.45× (2.42–2.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.08× (2.98–3.17) | 5.25× (5.00–5.50) | 5.68× (5.43–5.93) | 5.86× (5.70–6.03) | 5.30× (5.04–5.58) | 4.61× (4.55–4.66) | 4.22× (4.17–4.27) | 1.36× (1.34–1.38) | 1.56× (1.54–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.64) | 3.20× (3.14–3.26) | 3.98× (3.93–4.04) | 3.28× (3.23–3.34) | 5.65× (5.57–5.74) | 2.73× (2.68–2.79) | 2.46× (2.41–2.51) | 1.01× (0.99–1.03) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.54–0.64) | 2.83× (2.74–2.92) | 4.97× (4.84–5.11) | 3.05× (2.96–3.15) | 5.15× (5.00–5.29) | 2.51× (2.44–2.59) | 2.26× (2.20–2.33) | 0.93× (0.90–0.96) | 0.95× (0.92–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.56–0.57) | 3.12× (3.07–3.16) | 5.04× (4.96–5.11) | 3.08× (3.04–3.13) | 5.22× (5.13–5.31) | 2.53× (2.49–2.57) | 2.28× (2.24–2.31) | 0.94× (0.91–0.96) | 0.96× (0.93–0.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.50–2.54) | 2.77× (2.74–2.81) | 3.50× (3.47–3.53) | 3.65× (3.62–3.69) | 3.99× (3.95–4.04) | 2.46× (2.42–2.49) | 2.04× (2.02–2.06) | 1.97× (1.93–2.00) | 2.71× (2.68–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.30× (2.25–2.35) | 2.58× (2.52–2.65) | 3.18× (3.12–3.24) | 3.24× (3.16–3.32) | 3.49× (3.41–3.58) | 2.05× (2.00–2.11) | 1.70× (1.66–1.74) | 1.64× (1.60–1.68) | 2.27× (2.23–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.99–2.15) | 1.79× (1.72–1.86) | 2.25× (2.18–2.33) | 2.10× (2.03–2.17) | 2.19× (2.11–2.27) | 1.05× (1.01–1.09) | 0.87× (0.84–0.91) | 0.84× (0.81–0.87) | 1.16× (1.11–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.35–1.49) | 2.75× (2.73–2.77) | 2.56× (2.43–2.69) | 2.59× (2.57–2.60) | 2.94× (2.92–2.97) | 3.23× (3.21–3.25) | 3.20× (3.17–3.24) | 1.26× (1.25–1.28) | 1.42× (1.41–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.29–1.34) | 2.58× (2.53–2.64) | 2.32× (2.24–2.40) | 3.06× (2.99–3.12) | 2.71× (2.64–2.79) | 2.88× (2.83–2.93) | 2.85× (2.80–2.91) | 1.13× (1.10–1.16) | 1.27× (1.24–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.54× (1.50–1.59) | 2.45× (2.40–2.51) | 2.12× (1.79–2.52) | 2.72× (2.67–2.77) | 2.62× (2.53–2.71) | 2.14× (2.12–2.17) | 2.13× (2.10–2.15) | 0.84× (0.82–0.86) | 0.94× (0.93–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.47–1.49) | 3.12× (3.06–3.19) | 2.61× (2.57–2.64) | 2.73× (2.72–2.75) | 3.18× (3.09–3.27) | 3.72× (3.69–3.74) | 3.93× (3.90–3.95) | 1.45× (1.42–1.48) | 1.72× (1.70–1.73) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.41) | 2.87× (2.82–2.91) | 2.41× (2.37–2.45) | 3.34× (3.30–3.39) | 2.85× (2.82–2.87) | 3.29× (3.26–3.32) | 3.48× (3.43–3.52) | 1.29× (1.26–1.31) | 1.52× (1.50–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.53–1.66) | 2.62× (2.58–2.66) | 2.00× (1.96–2.04) | 2.87× (2.78–2.96) | 2.65× (2.56–2.73) | 2.31× (2.26–2.35) | 2.44× (2.38–2.49) | 0.90× (0.88–0.92) | 1.07× (1.04–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.72× (2.69–2.74) | 3.58× (3.54–3.62) | 2.85× (2.82–2.88) | 6.23× (5.93–6.54) | 2.34× (2.32–2.36) | 2.16× (2.13–2.18) | 1.04× (0.95–1.13) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.34× (2.31–2.38) | 4.05× (3.99–4.12) | 2.58× (2.55–2.61) | 5.33× (5.25–5.41) | 2.04× (2.01–2.07) | 1.88× (1.85–1.91) | 0.91× (0.83–0.99) | 0.87× (0.84–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.58× (2.55–2.62) | 4.02× (3.93–4.12) | 2.62× (2.59–2.66) | 5.39× (5.30–5.48) | 2.06× (2.04–2.09) | 1.90× (1.89–1.92) | 0.92× (0.84–1.00) | 0.88× (0.85–0.91) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 2.01× (1.98–2.04) | 4.13× (4.09–4.18) | 4.50× (4.45–4.56) | 4.61× (4.56–4.66) | 4.89× (4.84–4.94) | 5.35× (5.30–5.40) | 4.84× (4.79–4.88) | 1.58× (1.56–1.60) | 1.86× (1.83–1.89) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 1.62× (1.51–1.74) | 3.26× (3.09–3.44) | 3.57× (3.33–3.83) | 3.96× (3.71–4.23) | 3.70× (3.48–3.93) | 4.00× (3.73–4.29) | 3.61× (3.35–3.90) | 1.18× (1.10–1.26) | 1.39× (1.29–1.49) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 1.68× (1.57–1.79) | 2.66× (2.62–2.70) | 2.92× (2.86–2.97) | 3.27× (3.16–3.39) | 3.11× (3.04–3.18) | 2.68× (2.66–2.70) | 2.42× (2.40–2.44) | 0.79× (0.78–0.80) | 0.93× (0.91–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.30) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.95–11.07) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.35) | 0.47× (0.40–0.56) | 0.71× (0.70–0.73) | 0.60× (0.58–0.61) | 5.67× (5.57–5.77) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.38) | 0.63× (0.55–0.73) | 0.64× (0.61–0.68) | 0.56× (0.54–0.59) | 3.73× (3.56–3.90) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.94× (7.92–7.96) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.73) | 0.75× (0.71–0.78) | 1.72× (1.70–1.75) | 0.86× (0.85–0.87) | 6.24× (6.15–6.33) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.69–0.71) | 0.79× (0.74–0.85) | 1.27× (1.24–1.30) | 0.77× (0.74–0.80) | 3.71× (3.61–3.81) | 0.29× (0.28–0.30) | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.13) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.51) | 0.40× (0.36–0.43) | 0.89× (0.87–0.92) | 0.71× (0.68–0.74) | 3.08× (2.99–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.65–0.68) | 0.47× (0.46–0.47) | 0.90× (0.88–0.91) | 0.83× (0.81–0.86) | 2.60× (2.56–2.63) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.16× (5.16–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.34× (0.31–0.37) | 0.83× (0.80–0.87) | 0.71× (0.68–0.74) | 2.85× (2.74–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.64) | 0.51× (0.48–0.54) | 0.82× (0.80–0.83) | 0.85× (0.81–0.89) | 2.36× (2.30–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.31× (5.28–5.33) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.17–6.19) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.89–0.90) | 6.22× (6.19–6.25) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.41) | 0.65× (0.65–0.66) | 6.35× (6.33–6.36) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 35 MB | 0.48× (0.46–0.51) | 0.41× (0.38–0.45) | 1.12× (1.06–1.19) | 0.72× (0.68–0.76) | 3.52× (3.34–3.71) | 0.22× (0.21–0.23) | 0.23× (0.22–0.25) | 0.35× (0.33–0.37) | 0.17× (0.16–0.18) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 56 MB | 0.51× (0.49–0.52) | 0.50× (0.46–0.55) | 0.90× (0.89–0.92) | 0.71× (0.69–0.74) | 2.33× (2.30–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 424 | 2.37 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 775 | 2.52 | 1.07× (1.06–1.08) | 90% | 1.03× (1.01–1.05) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 924 | 3.98 | 1.68× (1.67–1.69) | 54% | 1.25× (1.19–1.30) | 1.68× (1.64–1.73) |

**Best:** mibz2 at 4 threads (924 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.09 | 0.227 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.3% |
| MTF + RLE2 | 1.52 | 17.1% |
| T-vector build | 1.53 | 17.2% |
| walk (the one each build uses) | 4.39 | 49.5% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.88 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.486 | 1.302 | (in its total) | 2.68× (2.67–2.68) |  |
| walk | 1.269 | 2.655 | (in its total) | 2.10× (2.07–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.20× (1.18–1.21) |  |
| inverse BWT, total | 1.804 | 4.014 | 3.254 | 2.23× (2.21–2.25) | 1.81× (1.79–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m40s |
| 4 threads | 33m06s ± 4m41s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m39s ± 3m20s |
| the scanner alone (its serial part) | 10m11s ± 0.00s |
| the workers alone, over 4 | 23m39s ± 3m20s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.10 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,369,642 | 12,022,894 | \`502528b59d9a8643…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
