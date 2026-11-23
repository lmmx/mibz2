# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-799/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`fd96fc0`](<https://github.com/lmmx/mibz2/commit/fd96fc08a293de71bdd9aafff1441a1d889fdf58>) (2026-11-23).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45160976727>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.496 s | 423 | 6.6 | 3.14× (3.11–3.17) | 7.18× (7.14–7.21) | 7.45× (7.36–7.54) | 8.01× (7.96–8.07) | 7.33× (7.26–7.39) | 7.84× (7.75–7.93) | 7.29× (7.15–7.43) | 2.34× (2.32–2.37) | 2.66× (2.64–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.284 s | 737 | 7.2 | 3.02× (2.85–3.20) | 6.56× (6.10–7.05) | 7.48× (7.05–7.94) | 7.30× (6.85–7.77) | 6.77× (6.37–7.21) | 13.12× (12.33–13.95) | 12.19× (11.45–12.98) | 3.92× (3.72–4.14) | 4.46× (4.20–4.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.232 s | 904 | 11.2 | 3.28× (3.18–3.39) | 5.31× (4.99–5.66) | 6.30× (6.12–6.48) | 5.70× (5.60–5.80) | 5.92× (5.80–6.04) | 16.72× (16.42–17.02) | 15.54× (15.19–15.91) | 5.00× (4.92–5.07) | 5.68× (5.58–5.78) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 924 | 3.3 | 0.57× (0.55–0.60) | 3.71× (3.54–3.88) | 4.51× (4.31–4.73) | 3.40× (3.26–3.53) | 5.96× (5.73–6.20) | 2.81× (2.70–2.92) | 2.54× (2.47–2.61) | 1.00× (0.97–1.04) | 1.42× (1.37–1.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 893 | 3.5 | 0.54× (0.53–0.56) | 2.76× (2.72–2.80) | 5.89× (5.81–5.97) | 3.33× (3.28–3.38) | 5.76× (5.68–5.83) | 2.73× (2.70–2.77) | 2.47× (2.43–2.51) | 0.97× (0.95–1.00) | 1.38× (1.36–1.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 877 | 3.5 | 0.54× (0.52–0.56) | 2.72× (2.68–2.76) | 5.77× (5.68–5.86) | 3.26× (3.20–3.32) | 5.67× (5.54–5.79) | 2.68× (2.64–2.72) | 2.42× (2.38–2.46) | 0.95× (0.93–0.98) | 1.35× (1.33–1.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 558 | 5.2 | 2.65× (2.57–2.72) | 2.92× (2.88–2.95) | 3.84× (3.68–4.00) | 3.72× (3.67–3.77) | 4.13× (4.08–4.18) | 2.47× (2.44–2.50) | 2.06× (2.03–2.09) | 2.01× (1.96–2.06) | 2.94× (2.85–3.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 865 | 6.4 | 2.29× (2.20–2.39) | 2.85× (2.68–3.04) | 4.20× (3.94–4.47) | 3.49× (3.32–3.67) | 4.92× (4.80–5.04) | 3.81× (3.71–3.91) | 3.18× (3.09–3.26) | 3.10× (3.00–3.21) | 4.53× (4.37–4.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 803 | 12.6 | 2.00× (1.95–2.05) | 2.07× (1.95–2.20) | 3.22× (3.04–3.42) | 2.24× (2.16–2.32) | 4.21× (4.10–4.33) | 3.51× (3.43–3.60) | 2.93× (2.86–3.00) | 2.86× (2.79–2.94) | 4.18× (4.10–4.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 59 | 47.4 | 1.34× (1.32–1.35) | 2.69× (2.63–2.75) | 2.42× (2.38–2.46) | 2.48× (2.44–2.52) | 2.90× (2.88–2.93) | 3.12× (3.09–3.14) | 3.06× (3.02–3.10) | 1.24× (1.23–1.26) | 1.43× (1.41–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 102 | 52.3 | 1.31× (1.26–1.36) | 2.59× (2.52–2.66) | 2.46× (2.35–2.57) | 3.13× (3.02–3.25) | 3.03× (2.90–3.16) | 5.23× (5.04–5.42) | 5.14× (4.97–5.31) | 2.09× (2.00–2.17) | 2.39× (2.32–2.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 120 | 71.6 | 1.35× (1.28–1.42) | 2.25× (2.13–2.37) | 2.19× (2.10–2.29) | 2.75× (2.56–2.96) | 2.99× (2.84–3.15) | 6.22× (5.93–6.52) | 6.11× (5.79–6.43) | 2.48× (2.36–2.60) | 2.85× (2.71–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 65 | 43.0 | 1.46× (1.44–1.47) | 3.06× (3.01–3.12) | 2.59× (2.54–2.63) | 2.65× (2.63–2.67) | 3.18× (3.08–3.28) | 3.65× (3.61–3.69) | 3.88× (3.85–3.91) | 1.48× (1.47–1.50) | 1.75× (1.72–1.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 108 | 48.4 | 1.35× (1.32–1.39) | 2.82× (2.76–2.88) | 2.57× (2.49–2.65) | 3.45× (3.37–3.53) | 3.12× (3.07–3.17) | 6.04× (5.94–6.15) | 6.42× (6.30–6.55) | 2.46× (2.40–2.51) | 2.89× (2.81–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 130 | 70.4 | 1.45× (1.42–1.48) | 2.49× (2.38–2.60) | 2.41× (2.33–2.49) | 3.17× (3.10–3.25) | 3.03× (2.96–3.09) | 7.21× (7.07–7.35) | 7.66× (7.52–7.80) | 2.93× (2.85–3.01) | 3.45× (3.35–3.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 712 | 4.5 | 0.66× (0.64–0.67) | 3.18× (2.96–3.41) | 4.22× (4.13–4.32) | 3.03× (2.97–3.08) | 6.82× (6.65–6.99) | 2.46× (2.41–2.51) | 2.24× (2.19–2.29) | 0.99× (0.97–1.01) | 1.46× (1.43–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 647 | 5.1 | 0.53× (0.51–0.55) | 2.66× (2.59–2.72) | 4.92× (4.70–5.16) | 2.80× (2.73–2.87) | 6.30× (6.00–6.61) | 2.24× (2.19–2.30) | 2.04× (2.00–2.09) | 0.90× (0.88–0.92) | 1.34× (1.30–1.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 633 | 5.1 | 0.53× (0.52–0.54) | 2.63× (2.60–2.65) | 4.79× (4.55–5.04) | 2.75× (2.71–2.78) | 6.12× (6.03–6.21) | 2.21× (2.19–2.23) | 2.01× (1.99–2.04) | 0.89× (0.87–0.91) | 1.32× (1.30–1.33) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.075 s | 186 | 15.3 | 2.02× (1.98–2.06) | 4.15× (4.07–4.24) | 4.59× (4.50–4.68) | 4.57× (4.47–4.67) | 4.82× (4.69–4.96) | 5.35× (5.22–5.48) | 4.82× (4.71–4.94) | 1.57× (1.53–1.60) | 1.92× (1.88–1.97) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.051 s | 272 | 18.9 | 1.65× (1.56–1.74) | 3.47× (3.23–3.71) | 4.24× (4.01–4.47) | 4.52× (4.25–4.81) | 4.31× (4.09–4.54) | 7.78× (7.40–8.18) | 7.02× (6.72–7.32) | 2.28× (2.18–2.39) | 2.80× (2.66–2.94) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.047 s | 296 | 29.6 | 1.55× (1.49–1.62) | 2.69× (2.52–2.88) | 3.54× (3.38–3.70) | 3.47× (3.39–3.55) | 3.89× (3.74–4.04) | 8.50× (8.26–8.74) | 7.66× (7.48–7.85) | 2.49× (2.44–2.55) | 3.06× (2.99–3.12) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.13× (3.10–3.16) | 7.14× (7.10–7.17) | 7.42× (7.33–7.50) | 7.98× (7.93–8.04) | 7.30× (7.24–7.36) | 7.81× (7.72–7.90) | 7.26× (7.12–7.41) | 2.34× (2.31–2.36) | 2.65× (2.62–2.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.01–3.13) | 6.92× (6.72–7.12) | 7.24× (7.15–7.33) | 7.91× (7.73–8.09) | 7.01× (6.85–7.18) | 7.19× (7.10–7.28) | 6.68× (6.56–6.82) | 2.15× (2.13–2.17) | 2.44× (2.41–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.16× (3.05–3.27) | 5.52× (5.36–5.68) | 5.79× (5.57–6.01) | 6.04× (5.89–6.18) | 5.57× (5.44–5.71) | 4.56× (4.44–4.69) | 4.24× (4.11–4.38) | 1.36× (1.33–1.40) | 1.55× (1.51–1.58) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.63) | 3.10× (2.99–3.22) | 3.89× (3.74–4.05) | 3.18× (3.05–3.30) | 5.51× (5.30–5.73) | 2.64× (2.54–2.74) | 2.40× (2.33–2.47) | 1.00× (0.96–1.03) | 1.03× (0.98–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.81× (2.77–2.85) | 4.98× (4.92–5.06) | 3.04× (3.00–3.09) | 5.17× (5.11–5.24) | 2.49× (2.46–2.52) | 2.26× (2.22–2.30) | 0.94× (0.92–0.96) | 0.97× (0.94–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.08× (3.02–3.14) | 4.95× (4.86–5.04) | 3.03× (2.98–3.09) | 5.14× (5.04–5.26) | 2.48× (2.44–2.51) | 2.25× (2.21–2.29) | 0.94× (0.91–0.96) | 0.96× (0.92–1.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.56× (2.49–2.63) | 2.72× (2.68–2.77) | 3.54× (3.40–3.69) | 3.57× (3.52–3.62) | 3.96× (3.92–4.01) | 2.39× (2.36–2.42) | 2.00× (1.97–2.03) | 1.95× (1.91–2.00) | 2.66× (2.59–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.21–2.27) | 2.49× (2.43–2.54) | 3.08× (3.05–3.11) | 3.16× (3.08–3.25) | 3.38× (3.35–3.42) | 1.96× (1.94–1.98) | 1.64× (1.62–1.66) | 1.60× (1.58–1.62) | 2.18× (2.14–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.09× (2.00–2.18) | 1.73× (1.63–1.83) | 2.18× (2.10–2.27) | 2.04× (1.94–2.14) | 2.14× (2.06–2.22) | 1.01× (0.96–1.05) | 0.84× (0.81–0.88) | 0.82× (0.79–0.85) | 1.12× (1.07–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.32–1.35) | 2.63× (2.57–2.69) | 2.37× (2.34–2.40) | 2.46× (2.42–2.50) | 2.88× (2.85–2.91) | 3.09× (3.07–3.11) | 3.04× (3.00–3.08) | 1.24× (1.23–1.25) | 1.37× (1.35–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.31) | 2.51× (2.48–2.55) | 2.21× (2.17–2.25) | 2.93× (2.89–2.97) | 2.66× (2.60–2.73) | 2.79× (2.75–2.83) | 2.74× (2.71–2.78) | 1.12× (1.10–1.14) | 1.24× (1.22–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.45–1.53) | 2.42× (2.36–2.48) | 1.94× (1.90–1.97) | 2.66× (2.61–2.72) | 2.61× (2.53–2.68) | 2.06× (2.03–2.08) | 2.02× (1.98–2.06) | 0.83× (0.81–0.84) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.47) | 3.01× (2.95–3.07) | 2.52× (2.49–2.56) | 2.62× (2.61–2.64) | 3.15× (3.06–3.25) | 3.61× (3.57–3.65) | 3.84× (3.81–3.87) | 1.48× (1.46–1.49) | 1.70× (1.68–1.72) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.34–1.39) | 2.81× (2.76–2.86) | 2.36× (2.34–2.39) | 3.27× (3.21–3.33) | 2.83× (2.80–2.86) | 3.21× (3.18–3.25) | 3.41× (3.37–3.46) | 1.31× (1.30–1.33) | 1.51× (1.50–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.48–1.60) | 2.61× (2.52–2.69) | 1.99× (1.95–2.04) | 2.85× (2.74–2.96) | 2.65× (2.58–2.72) | 2.22× (2.17–2.27) | 2.36× (2.30–2.42) | 0.91× (0.89–0.93) | 1.05× (1.02–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.72) | 2.66× (2.61–2.71) | 3.51× (3.42–3.60) | 2.76× (2.71–2.81) | 6.07× (5.91–6.23) | 2.27× (2.22–2.32) | 2.08× (2.04–2.13) | 0.99× (0.96–1.01) | 0.97× (0.93–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.56–0.60) | 2.29× (2.24–2.36) | 4.01× (3.91–4.11) | 2.52× (2.46–2.59) | 5.42× (5.16–5.69) | 1.99× (1.94–2.04) | 1.83× (1.78–1.87) | 0.87× (0.85–0.89) | 0.85× (0.81–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.57) | 2.50× (2.46–2.55) | 3.95× (3.88–4.02) | 2.54× (2.50–2.58) | 5.37× (5.29–5.45) | 2.00× (1.98–2.03) | 1.83× (1.81–1.86) | 0.87× (0.86–0.88) | 0.85× (0.82–0.89) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.99× (1.95–2.03) | 4.01× (3.93–4.09) | 4.42× (4.33–4.50) | 4.47× (4.37–4.57) | 4.72× (4.59–4.85) | 5.23× (5.10–5.35) | 4.71× (4.60–4.83) | 1.55× (1.52–1.59) | 1.84× (1.81–1.88) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.75× (1.71–1.80) | 3.48× (3.42–3.55) | 3.84× (3.71–3.97) | 4.20× (4.12–4.29) | 3.94× (3.84–4.04) | 4.23× (4.16–4.31) | 3.82× (3.73–3.91) | 1.26× (1.22–1.29) | 1.49× (1.46–1.52) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.75× (1.70–1.80) | 2.95× (2.77–3.14) | 3.15× (2.97–3.34) | 3.41× (3.24–3.58) | 3.30× (3.13–3.48) | 2.74× (2.63–2.85) | 2.47× (2.40–2.54) | 0.81× (0.79–0.84) | 0.97× (0.93–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.98× (10.96–11.00) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.37) | 0.49× (0.42–0.56) | 0.74× (0.72–0.76) | 0.62× (0.60–0.63) | 5.87× (5.72–6.02) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.35× (0.34–0.37) | 0.65× (0.58–0.74) | 0.63× (0.61–0.66) | 0.55× (0.53–0.57) | 3.66× (3.54–3.79) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.18–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.65–0.66) | 7.97× (7.95–7.99) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.69–0.72) | 0.73× (0.68–0.79) | 1.68× (1.64–1.72) | 0.84× (0.82–0.86) | 6.09× (5.93–6.24) | 0.49× (0.48–0.51) | 0.51× (0.50–0.53) | 0.56× (0.55–0.57) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.71) | 0.81× (0.77–0.85) | 1.25× (1.20–1.29) | 0.77× (0.74–0.80) | 3.68× (3.55–3.81) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.41× (0.39–0.43) | 0.91× (0.89–0.93) | 0.71× (0.68–0.74) | 3.13× (3.05–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.80× (0.76–0.84) | 2.53× (2.50–2.57) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.81–0.82) | 5.17× (5.16–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.36× (0.33–0.39) | 0.83× (0.80–0.85) | 0.68× (0.65–0.72) | 2.83× (2.74–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.49× (0.46–0.52) | 0.82× (0.80–0.83) | 0.82× (0.80–0.85) | 2.35× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.31× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.67) | 6.39× (6.38–6.40) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 33 MB | 0.51× (0.50–0.53) | 0.49× (0.46–0.53) | 1.19× (1.16–1.22) | 0.78× (0.76–0.81) | 3.80× (3.70–3.91) | 0.23× (0.23–0.24) | 0.25× (0.24–0.26) | 0.37× (0.36–0.38) | 0.18× (0.18–0.19) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 51 MB | 0.54× (0.52–0.55) | 0.49× (0.45–0.53) | 1.00× (0.98–1.02) | 0.83× (0.82–0.85) | 2.53× (2.50–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 423 | 2.38 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 737 | 2.58 | 1.09× (1.08–1.10) | 84% | 1.05× (1.02–1.08) | 1.07× (1.05–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 904 | 4.06 | 1.71× (1.67–1.75) | 53% | 1.32× (1.28–1.37) | 1.73× (1.69–1.77) |

**Best:** mibz2 at 4 threads (904 MB/s); lbzip2 at 4 threads (172 MB/s); pbz2 at 4 threads (276 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.67 | 0.02 | 0.09 | 0.232 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2768 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.79 | 10.3% |
| MTF + RLE2 | 1.33 | 17.2% |
| T-vector build | 1.32 | 17.2% |
| walk (the one each build uses) | 3.80 | 49.4% |
| of which: segment assembly | 0.17 | 2.2% |
| RLE1 | 0.24 | 3.2% |
| CRC | 0.22 | 2.9% |
| **total** | 7.70 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.487 | 1.298 | (in its total) | 2.68× (2.66–2.70) |  |
| walk | 1.254 | 2.675 | (in its total) | 2.15× (2.12–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.20× (1.15–1.24) |  |
| inverse BWT, total | 1.788 | 4.029 | 3.327 | 2.27× (2.24–2.29) | 1.86× (1.85–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h12m ± 8m39s |
| 4 threads | 33m52s ± 4m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m24s ± 3m28s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 24m24s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 3.20 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2768.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
