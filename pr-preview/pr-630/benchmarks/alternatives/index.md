# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-630/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`f6cfcb0`](<https://github.com/lmmx/mibz2/commit/f6cfcb04a53c91d521ddb973427b4d53ad7548da>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44487697898>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.506 s | 415 | 7.9 | 3.01× (2.99–3.03) | 6.88× (6.84–6.93) | 7.09× (7.05–7.12) | 7.69× (7.65–7.73) | 6.92× (6.87–6.97) | 7.57× (7.51–7.63) | 6.98× (6.93–7.03) | 2.22× (2.21–2.23) | 2.60× (2.58–2.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.297 s | 706 | 8.6 | 2.90× (2.76–3.04) | 6.17× (5.86–6.49) | 7.06× (6.68–7.46) | 6.94× (6.62–7.28) | 6.25× (5.94–6.57) | 12.79× (12.15–13.46) | 11.79× (11.18–12.43) | 3.74× (3.56–3.94) | 4.39× (4.15–4.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.238 s | 881 | 13.3 | 3.00× (2.96–3.04) | 4.61× (4.55–4.66) | 5.68× (5.56–5.80) | 5.26× (5.04–5.50) | 5.19× (5.14–5.24) | 16.08× (15.90–16.27) | 14.82× (14.67–14.98) | 4.71× (4.66–4.76) | 5.52× (5.46–5.58) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 998 | 3.6 | 0.61× (0.59–0.62) | 3.80× (3.61–4.00) | 4.55× (4.40–4.69) | 3.62× (3.57–3.67) | 6.23× (6.13–6.33) | 3.02× (2.98–3.07) | 2.70× (2.66–2.74) | 1.02× (1.01–1.03) | 1.48× (1.38–1.59) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 973 | 3.7 | 0.58× (0.56–0.60) | 3.00× (2.95–3.06) | 5.97× (5.85–6.10) | 3.58× (3.52–3.64) | 6.05× (5.96–6.15) | 2.96× (2.91–3.01) | 2.65× (2.60–2.69) | 1.00× (0.98–1.02) | 1.45× (1.36–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 964 | 3.7 | 0.57× (0.56–0.58) | 2.97× (2.93–3.01) | 5.84× (5.77–5.92) | 3.51× (3.47–3.56) | 6.06× (5.99–6.13) | 2.92× (2.88–2.96) | 2.61× (2.58–2.64) | 0.98× (0.96–1.01) | 1.43× (1.33–1.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 587 | 5.9 | 2.79× (2.72–2.85) | 3.06× (3.05–3.07) | 3.95× (3.88–4.03) | 3.87× (3.85–3.89) | 4.15× (4.13–4.18) | 2.57× (2.56–2.59) | 2.15× (2.11–2.20) | 2.04× (2.03–2.05) | 3.04× (3.03–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 885 | 7.2 | 2.18× (1.99–2.38) | 2.73× (2.49–3.00) | 4.01× (3.73–4.31) | 3.28× (3.02–3.56) | 4.56× (4.19–4.96) | 3.68× (3.38–4.01) | 3.08× (2.82–3.37) | 2.92× (2.69–3.17) | 4.35× (4.00–4.74) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 773 | 14.1 | 1.83× (1.77–1.90) | 1.98× (1.83–2.14) | 2.94× (2.81–3.06) | 2.16× (2.07–2.25) | 3.82× (3.69–3.96) | 3.39× (3.29–3.49) | 2.84× (2.73–2.96) | 2.69× (2.61–2.78) | 4.01× (3.89–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 54.9 | 1.34× (1.34–1.35) | 2.67× (2.66–2.68) | 2.41× (2.37–2.44) | 2.47× (2.46–2.48) | 2.81× (2.80–2.82) | 3.11× (3.09–3.12) | 3.09× (3.07–3.10) | 1.21× (1.20–1.21) | 1.37× (1.36–1.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 98 | 62.5 | 1.27× (1.24–1.30) | 2.53× (2.48–2.59) | 2.36× (2.33–2.39) | 3.04× (2.99–3.10) | 2.91× (2.87–2.95) | 5.07× (4.99–5.14) | 5.03× (4.97–5.09) | 1.97× (1.94–1.99) | 2.23× (2.20–2.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 83.7 | 1.29× (1.24–1.35) | 2.06× (1.97–2.16) | 2.00× (1.91–2.11) | 2.48× (2.36–2.60) | 2.75× (2.60–2.92) | 5.85× (5.59–6.13) | 5.81× (5.55–6.09) | 2.27× (2.18–2.37) | 2.57× (2.46–2.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 65 | 50.4 | 1.47× (1.45–1.49) | 3.01× (2.97–3.05) | 2.55× (2.52–2.57) | 2.64× (2.61–2.66) | 3.08× (3.01–3.15) | 3.60× (3.57–3.63) | 3.81× (3.78–3.84) | 1.39× (1.37–1.41) | 1.66× (1.64–1.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 57.0 | 1.37× (1.35–1.38) | 2.76× (2.71–2.81) | 2.47× (2.42–2.52) | 3.45× (3.36–3.54) | 3.08× (3.03–3.12) | 6.00× (5.91–6.09) | 6.35× (6.27–6.42) | 2.32× (2.28–2.35) | 2.76× (2.71–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 80.8 | 1.40× (1.37–1.42) | 2.19× (2.15–2.23) | 2.16× (2.10–2.21) | 2.93× (2.85–3.00) | 2.69× (2.65–2.73) | 6.71× (6.63–6.79) | 7.10× (6.99–7.20) | 2.59× (2.55–2.63) | 3.09× (3.04–3.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 749 | 5.0 | 0.67× (0.66–0.68) | 3.07× (3.03–3.12) | 4.50× (4.43–4.57) | 3.17× (3.13–3.21) | 6.83× (6.74–6.91) | 2.57× (2.54–2.61) | 2.35× (2.32–2.38) | 0.99× (0.98–1.01) | 1.55× (1.53–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 709 | 5.3 | 0.57× (0.56–0.57) | 2.91× (2.89–2.94) | 5.01× (4.97–5.05) | 3.03× (3.00–3.06) | 6.46× (6.39–6.52) | 2.43× (2.41–2.46) | 2.22× (2.20–2.25) | 0.94× (0.92–0.95) | 1.46× (1.45–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 5.3 | 0.56× (0.55–0.57) | 2.84× (2.79–2.89) | 4.87× (4.78–4.95) | 2.94× (2.90–2.98) | 6.26× (6.15–6.37) | 2.36× (2.32–2.41) | 2.16× (2.12–2.20) | 0.91× (0.89–0.93) | 1.42× (1.40–1.45) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.070 s | 186 | 17.9 | 1.95× (1.89–2.02) | 4.13× (3.99–4.26) | 4.45× (4.29–4.61) | 4.50× (4.35–4.65) | 4.66× (4.48–4.84) | 5.20× (5.03–5.38) | 4.74× (4.58–4.90) | 1.49× (1.44–1.53) | 1.86× (1.80–1.92) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.049 s | 266 | 22.2 | 1.58× (1.48–1.70) | 3.20× (2.98–3.44) | 3.85× (3.60–4.12) | 4.17× (3.84–4.52) | 4.17× (3.89–4.48) | 7.36× (6.87–7.88) | 6.70× (6.26–7.18) | 2.10× (1.97–2.25) | 2.63× (2.46–2.81) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.042 s | 313 | 34.5 | 1.63× (1.59–1.67) | 2.57× (2.45–2.69) | 3.34× (3.24–3.44) | 3.38× (3.28–3.48) | 3.92× (3.83–4.02) | 8.71× (8.52–8.91) | 7.93× (7.76–8.11) | 2.49× (2.44–2.55) | 3.11× (3.04–3.18) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.00× (2.98–3.02) | 6.85× (6.81–6.89) | 7.05× (7.01–7.09) | 7.66× (7.62–7.70) | 6.89× (6.84–6.94) | 7.54× (7.48–7.60) | 6.95× (6.90–7.00) | 2.21× (2.20–2.22) | 2.58× (2.56–2.60) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.89× (2.85–2.92) | 6.47× (6.42–6.52) | 6.76× (6.72–6.81) | 7.43× (7.40–7.46) | 6.37× (6.34–6.41) | 6.92× (6.88–6.95) | 6.37× (6.35–6.40) | 2.03× (2.01–2.04) | 2.37× (2.34–2.39) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.94× (2.88–3.00) | 4.97× (4.92–5.03) | 5.36× (5.28–5.43) | 5.62× (5.57–5.68) | 4.96× (4.92–5.00) | 4.48× (4.44–4.52) | 4.13× (4.10–4.16) | 1.31× (1.30–1.33) | 1.53× (1.51–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.35× (3.27–3.43) | 4.15× (4.06–4.25) | 3.38× (3.33–3.43) | 5.76× (5.67–5.86) | 2.84× (2.80–2.89) | 2.55× (2.51–2.59) | 1.02× (1.00–1.03) | 1.08× (1.05–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.61–0.64) | 3.08× (3.04–3.12) | 5.39× (5.32–5.47) | 3.32× (3.27–3.37) | 5.53× (5.45–5.60) | 2.74× (2.70–2.78) | 2.46× (2.42–2.50) | 0.98× (0.96–1.00) | 1.04× (1.01–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.37× (3.34–3.40) | 5.39× (5.28–5.50) | 3.32× (3.28–3.36) | 5.60× (5.54–5.66) | 2.75× (2.71–2.79) | 2.47× (2.44–2.50) | 0.99× (0.96–1.01) | 1.05× (1.02–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.62–2.75) | 2.81× (2.80–2.82) | 3.58× (3.48–3.67) | 3.71× (3.69–3.73) | 3.98× (3.95–4.00) | 2.48× (2.47–2.50) | 2.09× (2.05–2.13) | 1.98× (1.98–1.99) | 2.75× (2.70–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.25–2.33) | 2.53× (2.47–2.60) | 3.07× (3.03–3.11) | 3.17× (3.12–3.22) | 3.36× (3.31–3.40) | 2.01× (1.97–2.04) | 1.69× (1.64–1.74) | 1.60× (1.58–1.63) | 2.22× (2.17–2.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.97–2.12) | 1.76× (1.71–1.82) | 2.22× (2.15–2.29) | 2.04× (1.99–2.09) | 2.09× (2.03–2.15) | 1.03× (1.00–1.06) | 0.87× (0.85–0.89) | 0.82× (0.80–0.85) | 1.14× (1.11–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.33–1.35) | 2.61× (2.60–2.62) | 2.35× (2.33–2.38) | 2.45× (2.44–2.46) | 2.78× (2.77–2.80) | 3.08× (3.06–3.09) | 3.05× (3.04–3.07) | 1.20× (1.20–1.21) | 1.34× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.23–1.26) | 2.43× (2.40–2.45) | 2.14× (2.11–2.16) | 2.85× (2.83–2.88) | 2.52× (2.48–2.56) | 2.71× (2.68–2.74) | 2.69× (2.66–2.71) | 1.06× (1.05–1.07) | 1.18× (1.17–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.47–1.51) | 2.26× (2.23–2.29) | 1.85× (1.84–1.87) | 2.53× (2.49–2.57) | 2.40× (2.37–2.43) | 2.02× (1.99–2.05) | 2.00× (1.98–2.03) | 0.79× (0.78–0.80) | 0.88× (0.87–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.44–1.48) | 2.96× (2.93–2.99) | 2.49× (2.45–2.52) | 2.61× (2.59–2.64) | 3.05× (2.99–3.12) | 3.57× (3.54–3.60) | 3.77× (3.74–3.80) | 1.38× (1.37–1.40) | 1.63× (1.61–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.34–1.37) | 2.72× (2.68–2.76) | 2.28× (2.25–2.31) | 3.17× (3.13–3.22) | 2.71× (2.68–2.75) | 3.15× (3.10–3.20) | 3.33× (3.29–3.37) | 1.22× (1.20–1.24) | 1.44× (1.42–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.57–1.65) | 2.51× (2.47–2.55) | 1.91× (1.87–1.95) | 2.76× (2.68–2.85) | 2.49× (2.46–2.52) | 2.23× (2.20–2.27) | 2.36× (2.33–2.40) | 0.87× (0.85–0.88) | 1.02× (1.00–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.71–0.72) | 2.74× (2.71–2.78) | 3.65× (3.54–3.76) | 2.89× (2.86–2.93) | 6.08× (6.00–6.16) | 2.37× (2.35–2.39) | 2.17× (2.15–2.20) | 0.99× (0.98–1.01) | 1.04× (0.98–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.63–0.64) | 2.53× (2.51–2.55) | 4.32× (4.27–4.37) | 2.79× (2.77–2.82) | 5.68× (5.63–5.74) | 2.21× (2.19–2.23) | 2.03× (2.01–2.04) | 0.93× (0.91–0.94) | 0.97× (0.91–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.71–2.80) | 4.24× (4.16–4.32) | 2.78× (2.74–2.82) | 5.64× (5.54–5.75) | 2.20× (2.16–2.24) | 2.02× (1.98–2.05) | 0.92× (0.90–0.94) | 0.96× (0.91–1.02) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.93× (1.87–1.99) | 3.94× (3.82–4.06) | 4.29× (4.16–4.42) | 4.40× (4.26–4.54) | 4.56× (4.39–4.73) | 5.08× (4.93–5.24) | 4.63× (4.49–4.79) | 1.47× (1.43–1.52) | 1.76× (1.71–1.82) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.70× (1.67–1.74) | 3.39× (3.32–3.45) | 3.71× (3.66–3.77) | 4.15× (4.08–4.23) | 3.84× (3.79–3.90) | 4.20× (4.13–4.26) | 3.82× (3.77–3.88) | 1.22× (1.20–1.24) | 1.45× (1.43–1.47) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.74× (1.69–1.79) | 2.64× (2.55–2.72) | 2.90× (2.85–2.95) | 3.22× (3.15–3.28) | 3.01× (2.95–3.07) | 2.67× (2.62–2.72) | 2.43× (2.39–2.48) | 0.77× (0.76–0.79) | 0.92× (0.91–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.29× (0.28–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.01× (10.97–11.05) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.33) | 0.45× (0.38–0.55) | 0.66× (0.63–0.69) | 0.55× (0.53–0.57) | 5.26× (5.06–5.47) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.31–0.34) | 0.66× (0.58–0.74) | 0.59× (0.56–0.62) | 0.51× (0.49–0.54) | 3.40× (3.25–3.55) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.02–8.05) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.72× (0.66–0.79) | 1.61× (1.57–1.65) | 0.82× (0.79–0.84) | 5.84× (5.70–5.98) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.62–0.68) | 0.71× (0.68–0.75) | 1.14× (1.12–1.17) | 0.71× (0.68–0.74) | 3.38× (3.28–3.49) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.18× (5.16–5.20) | 0.33× (0.33–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.51) | 0.40× (0.37–0.43) | 0.91× (0.89–0.92) | 0.73× (0.70–0.75) | 3.15× (3.09–3.21) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.78–0.83) | 2.50× (2.45–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.19× (5.17–5.21) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.40) | 0.85× (0.82–0.88) | 0.71× (0.69–0.74) | 2.93× (2.84–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.49× (0.46–0.53) | 0.81× (0.79–0.82) | 0.77× (0.74–0.80) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.77) | 5.30× (5.28–5.32) | 0.35× (0.34–0.35) | 0.36× (0.35–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.29–5.32) | 0.35× (0.34–0.35) | 0.36× (0.35–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.43–1.44) | 0.67× (0.67–0.67) | 6.47× (6.45–6.49) | 0.40× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.39× (0.36–0.43) | 1.06× (1.02–1.09) | 0.69× (0.67–0.70) | 3.33× (3.24–3.42) | 0.20× (0.20–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 57 MB | 0.49× (0.47–0.50) | 0.49× (0.46–0.52) | 0.90× (0.88–0.91) | 0.74× (0.72–0.76) | 2.29× (2.25–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 415 | 2.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 706 | 2.64 | 1.09× (1.08–1.10) | 84% | 1.03× (1.02–1.04) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 881 | 4.09 | 1.68× (1.67–1.70) | 53% | 1.22× (1.21–1.23) | 1.65× (1.62–1.69) |

**Best:** mibz2 at 4 threads (881 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (293 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.20, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.66 | 0.02 | 0.09 | 0.238 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3245 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.5% |
| MTF + RLE2 | 1.59 | 17.7% |
| T-vector build | 1.52 | 16.9% |
| walk (the one each build uses) | 4.21 | 47.0% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.97 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.301 | (in its total) | 2.64× (2.64–2.65) |  |
| walk | 1.224 | 2.654 | (in its total) | 2.15× (2.14–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.057 | (in its total) | 1.19× (1.16–1.21) |  |
| inverse BWT, total | 1.764 | 4.010 | 3.301 | 2.26× (2.25–2.28) | 1.87× (1.86–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m25s |
| 4 threads | 34m45s ± 4m53s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m54s ± 3m23s |
| the scanner alone (its serial part) | 13m49s ± 32.9s |
| the workers alone, over 4 | 23m54s ± 3m23s |

At 4 threads the workers bound it. Per byte: the scanner 8.0 ns a byte of bz2, the workers 3.13 ns a byte of output at 4 threads (CPU, summed over them), 1.97 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3245.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
