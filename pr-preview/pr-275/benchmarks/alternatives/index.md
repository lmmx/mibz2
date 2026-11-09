# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-275/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`44b0b1a`](<https://github.com/lmmx/mibz2/commit/44b0b1a30e0117176a0e4e6acae76e2a8fd323ba>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42547818983>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.651 s | 322 | 9.1 | 2.35× (2.32–2.37) | 5.32× (5.26–5.39) | 5.51× (5.46–5.57) | 5.96× (5.90–6.02) | 5.35× (5.29–5.40) | 5.74× (5.67–5.80) | 5.36× (5.32–5.40) | 1.74× (1.73–1.76) | 2.00× (1.98–2.03) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.371 s | 565 | 9.8 | 2.45× (2.28–2.64) | 5.03× (4.90–5.16) | 5.79× (5.55–6.05) | 5.72× (5.49–5.95) | 5.09× (4.92–5.27) | 10.09× (9.75–10.44) | 9.42× (9.13–9.73) | 3.07× (2.97–3.17) | 3.52× (3.41–3.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.306 s | 685 | 15.6 | 2.38× (2.27–2.50) | 3.58× (3.45–3.71) | 4.51× (4.33–4.70) | 4.04× (3.92–4.17) | 4.07× (3.93–4.21) | 12.08× (11.56–12.62) | 11.28× (10.83–11.75) | 3.67× (3.52–3.83) | 4.22× (4.05–4.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 964 | 3.3 | 0.59× (0.58–0.60) | 3.65× (3.47–3.84) | 4.46× (4.33–4.60) | 3.54× (3.48–3.60) | 6.10× (5.98–6.22) | 2.81× (2.76–2.86) | 2.63× (2.59–2.68) | 1.01× (1.00–1.03) | 1.49× (1.47–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 952 | 3.4 | 0.56× (0.55–0.58) | 2.94× (2.89–2.99) | 6.13× (5.93–6.33) | 3.52× (3.44–3.60) | 5.99× (5.90–6.08) | 2.77× (2.72–2.81) | 2.59× (2.55–2.64) | 1.00× (0.98–1.02) | 1.47× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 939 | 3.4 | 0.57× (0.56–0.58) | 2.91× (2.87–2.95) | 5.89× (5.74–6.05) | 3.45× (3.41–3.50) | 5.95× (5.84–6.07) | 2.73× (2.69–2.76) | 2.56× (2.52–2.59) | 0.98× (0.96–1.01) | 1.45× (1.43–1.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 538 | 5.7 | 2.42× (2.40–2.44) | 2.79× (2.75–2.84) | 3.58× (3.52–3.64) | 3.56× (3.51–3.61) | 3.85× (3.80–3.91) | 2.31× (2.28–2.35) | 1.96× (1.93–1.99) | 1.89× (1.87–1.91) | 2.78× (2.74–2.82) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 797 | 7.2 | 1.86× (1.51–2.28) | 2.34× (1.93–2.83) | 3.21× (2.66–3.87) | 2.78× (2.23–3.45) | 3.86× (3.18–4.68) | 3.01× (2.47–3.66) | 2.55× (2.10–3.09) | 2.46× (2.02–2.98) | 3.62× (2.97–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 731 | 13.8 | 1.82× (1.77–1.86) | 1.88× (1.78–1.97) | 2.79× (2.67–2.92) | 2.02× (1.96–2.08) | 3.70× (3.61–3.80) | 3.14× (3.06–3.23) | 2.66× (2.60–2.73) | 2.56× (2.51–2.63) | 3.77× (3.68–3.88) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.131 s | 61 | 48.4 | 1.35× (1.34–1.37) | 2.71× (2.69–2.73) | 2.45× (2.42–2.49) | 2.52× (2.51–2.54) | 2.87× (2.86–2.89) | 3.15× (3.12–3.17) | 3.14× (3.12–3.15) | 1.28× (1.27–1.29) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 55.6 | 1.26× (1.24–1.29) | 2.51× (2.46–2.56) | 2.36× (2.30–2.42) | 3.04× (2.98–3.10) | 2.92× (2.87–2.97) | 5.03× (4.93–5.12) | 5.01× (4.91–5.11) | 2.04× (2.00–2.08) | 2.27× (2.20–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 118 | 73.5 | 1.31× (1.28–1.34) | 2.20× (2.14–2.27) | 2.14× (2.06–2.23) | 2.65× (2.57–2.73) | 2.92× (2.82–3.02) | 6.06× (5.94–6.19) | 6.04× (5.93–6.15) | 2.46× (2.41–2.52) | 2.74× (2.68–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 66 | 45.1 | 1.44× (1.43–1.45) | 3.05× (3.01–3.09) | 2.58× (2.53–2.63) | 2.66× (2.64–2.68) | 3.10× (3.00–3.21) | 3.75× (3.72–3.78) | 3.88× (3.86–3.91) | 1.46× (1.44–1.48) | 1.73× (1.70–1.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 107 | 51.0 | 1.36× (1.27–1.47) | 2.72× (2.62–2.83) | 2.46× (2.37–2.55) | 3.49× (3.36–3.63) | 3.06× (2.95–3.17) | 6.09× (5.91–6.27) | 6.30× (6.11–6.50) | 2.37× (2.27–2.48) | 2.81× (2.68–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 73.2 | 1.38× (1.34–1.41) | 2.38× (2.23–2.54) | 2.23× (2.14–2.32) | 2.92× (2.84–3.01) | 2.81× (2.74–2.88) | 7.02× (6.89–7.16) | 7.27× (7.13–7.42) | 2.74× (2.65–2.83) | 3.24× (3.15–3.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 4.6 | 0.65× (0.64–0.66) | 3.09× (2.92–3.28) | 4.38× (4.23–4.54) | 3.08× (3.02–3.13) | 6.69× (6.55–6.83) | 2.43× (2.38–2.48) | 2.28× (2.24–2.33) | 0.97× (0.95–0.99) | 1.49× (1.46–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.0 | 0.55× (0.54–0.56) | 2.81× (2.76–2.85) | 5.06× (4.85–5.29) | 2.94× (2.89–2.98) | 6.36× (6.24–6.48) | 2.30× (2.27–2.33) | 2.16× (2.13–2.19) | 0.92× (0.90–0.93) | 1.41× (1.39–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 4.9 | 0.55× (0.54–0.56) | 2.79× (2.72–2.86) | 5.17× (4.98–5.37) | 2.92× (2.85–2.99) | 6.23× (6.09–6.38) | 2.28× (2.23–2.33) | 2.14× (2.10–2.19) | 0.91× (0.89–0.93) | 1.40× (1.36–1.43) | yes |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 1 | 0.094 s | 134 | 22.1 | 1.41× (1.39–1.42) | 2.97× (2.93–3.02) | 3.26× (3.22–3.30) | 3.21× (3.17–3.25) | 3.35× (3.32–3.39) | 3.63× (3.60–3.66) | 3.40× (3.37–3.43) | 1.13× (1.11–1.15) | 1.37× (1.34–1.40) | yes |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 2 | 0.063 s | 200 | 25.4 | 1.16× (1.03–1.30) | 2.35× (2.08–2.64) | 2.80× (2.52–3.11) | 2.99× (2.69–3.32) | 3.08× (2.77–3.42) | 5.19× (4.63–5.82) | 4.86× (4.34–5.44) | 1.62× (1.44–1.81) | 1.95× (1.75–2.18) | yes |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 4 | 0.048 s | 262 | 35.1 | 1.39× (1.27–1.52) | 2.10× (1.93–2.28) | 2.80× (2.63–2.99) | 2.90× (2.72–3.09) | 3.30× (3.11–3.51) | 6.93× (6.68–7.20) | 6.49× (6.26–6.74) | 2.16× (2.07–2.25) | 2.61× (2.50–2.72) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.34× (2.32–2.37) | 5.30× (5.23–5.37) | 5.49× (5.43–5.55) | 5.94× (5.88–6.01) | 5.33× (5.27–5.39) | 5.72× (5.66–5.78) | 5.34× (5.30–5.39) | 1.74× (1.73–1.76) | 1.99× (1.97–2.01) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.31× (2.23–2.39) | 5.06× (5.02–5.10) | 5.41× (5.34–5.48) | 5.91× (5.78–6.04) | 5.04× (4.97–5.12) | 5.30× (5.24–5.36) | 4.95× (4.90–4.99) | 1.61× (1.59–1.63) | 1.84× (1.82–1.86) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.33× (2.26–2.41) | 3.88× (3.75–4.00) | 4.21× (4.07–4.35) | 4.37× (4.20–4.54) | 3.89× (3.76–4.04) | 3.36× (3.25–3.49) | 3.14× (3.03–3.26) | 1.02× (0.99–1.06) | 1.17× (1.13–1.22) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.23× (3.17–3.29) | 4.03× (3.98–4.09) | 3.31× (3.25–3.37) | 5.64× (5.54–5.74) | 2.64× (2.60–2.68) | 2.48× (2.44–2.52) | 1.01× (1.00–1.02) | 1.08× (1.04–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.02× (2.97–3.08) | 5.34× (5.25–5.43) | 3.27× (3.19–3.34) | 5.46× (5.38–5.54) | 2.56× (2.52–2.60) | 2.41× (2.37–2.45) | 0.98× (0.96–1.00) | 1.05× (1.01–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.32× (3.25–3.39) | 5.31× (5.20–5.43) | 3.27× (3.22–3.32) | 5.50× (5.39–5.62) | 2.57× (2.53–2.61) | 2.42× (2.38–2.46) | 0.98× (0.96–1.01) | 1.05× (1.03–1.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.34× (2.33–2.36) | 2.59× (2.55–2.63) | 3.27× (3.23–3.31) | 3.42× (3.38–3.47) | 3.70× (3.65–3.75) | 2.24× (2.21–2.28) | 1.91× (1.88–1.93) | 1.84× (1.82–1.86) | 2.53× (2.49–2.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.10× (2.03–2.18) | 2.31× (2.22–2.41) | 2.79× (2.73–2.86) | 2.86× (2.80–2.93) | 3.06× (2.99–3.13) | 1.77× (1.73–1.82) | 1.51× (1.47–1.55) | 1.46× (1.42–1.49) | 2.00× (1.96–2.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.88–2.01) | 1.63× (1.57–1.69) | 2.05× (2.00–2.11) | 1.94× (1.88–2.01) | 1.95× (1.89–2.01) | 0.93× (0.91–0.96) | 0.79× (0.77–0.81) | 0.76× (0.74–0.78) | 1.05× (1.03–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.65× (2.63–2.66) | 2.40× (2.37–2.43) | 2.50× (2.49–2.51) | 2.84× (2.83–2.86) | 3.11× (3.09–3.14) | 3.10× (3.09–3.12) | 1.28× (1.26–1.29) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.25–1.27) | 2.48× (2.44–2.51) | 2.18× (2.15–2.21) | 2.91× (2.87–2.95) | 2.56× (2.53–2.58) | 2.73× (2.70–2.77) | 2.72× (2.69–2.76) | 1.12× (1.10–1.14) | 1.21× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.43–1.51) | 2.36× (2.33–2.40) | 1.92× (1.89–1.96) | 2.65× (2.58–2.72) | 2.54× (2.45–2.63) | 2.04× (2.01–2.07) | 2.03× (2.01–2.06) | 0.84× (0.83–0.85) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.43–1.44) | 2.99× (2.96–3.02) | 2.53× (2.48–2.57) | 2.64× (2.61–2.66) | 3.07× (2.97–3.18) | 3.71× (3.68–3.74) | 3.84× (3.82–3.87) | 1.46× (1.43–1.48) | 1.69× (1.67–1.71) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.29–1.45) | 2.79× (2.74–2.84) | 2.32× (2.28–2.36) | 3.24× (3.21–3.28) | 2.77× (2.70–2.84) | 3.27× (3.23–3.31) | 3.38× (3.35–3.42) | 1.28× (1.25–1.31) | 1.49× (1.46–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.51× (1.45–1.56) | 2.57× (2.49–2.65) | 1.94× (1.89–2.00) | 2.74× (2.67–2.81) | 2.53× (2.48–2.58) | 2.28× (2.23–2.33) | 2.36× (2.32–2.39) | 0.89× (0.87–0.91) | 1.04× (1.02–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.67× (2.63–2.72) | 3.52× (3.45–3.59) | 2.82× (2.77–2.86) | 5.94× (5.85–6.04) | 2.24× (2.21–2.27) | 2.11× (2.08–2.15) | 0.98× (0.96–0.99) | 1.00× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.60–0.63) | 2.48× (2.45–2.51) | 4.25× (4.16–4.34) | 2.72× (2.68–2.76) | 5.62× (5.52–5.73) | 2.10× (2.08–2.12) | 1.98× (1.96–2.00) | 0.91× (0.90–0.92) | 0.93× (0.90–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.62) | 2.74× (2.68–2.80) | 4.30× (4.21–4.40) | 2.78× (2.71–2.85) | 5.64× (5.51–5.78) | 2.13× (2.09–2.17) | 2.01× (1.97–2.05) | 0.93× (0.91–0.94) | 0.95× (0.91–0.99) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 1 | 1.40× (1.39–1.41) | 2.88× (2.84–2.91) | 3.14× (3.11–3.18) | 3.16× (3.13–3.20) | 3.30× (3.27–3.34) | 3.58× (3.55–3.60) | 3.35× (3.32–3.38) | 1.13× (1.11–1.15) | 1.30× (1.29–1.32) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 2 | 1.31× (1.29–1.33) | 2.64× (2.57–2.72) | 2.91× (2.86–2.96) | 3.14× (3.09–3.20) | 3.01× (2.93–3.09) | 3.10× (3.06–3.15) | 2.91× (2.86–2.96) | 0.98× (0.96–1.00) | 1.13× (1.11–1.15) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 4 | 1.46× (1.39–1.52) | 2.40× (2.28–2.52) | 2.65× (2.54–2.77) | 2.79× (2.66–2.94) | 2.78× (2.63–2.94) | 2.25× (2.21–2.28) | 2.10× (2.06–2.14) | 0.71× (0.69–0.72) | 0.82× (0.80–0.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.33) | 0.33× (0.32–0.34) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.18× (12.15–12.22) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.61× (0.54–0.69) | 0.80× (0.79–0.81) | 0.67× (0.66–0.67) | 6.39× (6.35–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.70× (0.58–0.84) | 0.71× (0.70–0.72) | 0.62× (0.61–0.63) | 4.08× (4.06–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.17× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.32× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.83–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.67) | 8.13× (8.12–8.15) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.63–0.66) | 0.72× (0.68–0.77) | 1.58× (1.54–1.63) | 0.78× (0.76–0.80) | 5.73× (5.59–5.88) | 0.45× (0.44–0.47) | 0.48× (0.47–0.49) | 0.52× (0.51–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.67) | 0.75× (0.70–0.80) | 1.16× (1.10–1.21) | 0.74× (0.71–0.78) | 3.38× (3.23–3.55) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.74× (0.73–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.41) | 0.89× (0.87–0.92) | 0.68× (0.66–0.70) | 3.09× (3.01–3.17) | 0.19× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.66) | 0.46× (0.46–0.47) | 0.89× (0.87–0.90) | 0.83× (0.80–0.85) | 2.57× (2.53–2.62) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.50) | 0.40× (0.37–0.43) | 0.88× (0.86–0.90) | 0.73× (0.71–0.76) | 3.01× (2.94–3.09) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.50× (0.46–0.53) | 0.81× (0.80–0.83) | 0.80× (0.77–0.83) | 2.34× (2.30–2.37) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.76–0.77) | 5.44× (5.43–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.20–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.77) | 5.40× (5.38–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.20–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.78) | 5.43× (5.41–5.44) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.66× (0.66–0.67) | 6.72× (6.71–6.73) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 2 | 33 MB | 0.49× (0.49–0.50) | 0.43× (0.39–0.47) | 1.21× (1.18–1.24) | 0.74× (0.73–0.76) | 3.75× (3.72–3.79) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.37× (0.36–0.37) | 0.18× (0.18–0.19) |
| `text` (2.13 MB → 12.6 MB, 5.9×) | 4 | 49 MB | 0.57× (0.55–0.59) | 0.51× (0.47–0.55) | 1.04× (1.01–1.06) | 0.80× (0.77–0.84) | 2.62× (2.57–2.67) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 322 | 3.11 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 565 | 3.38 | 1.08× (1.07–1.09) | 88% | 1.03× (1.01–1.05) | 1.06× (1.02–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 685 | 5.34 | 1.70× (1.64–1.77) | 53% | 1.24× (1.21–1.28) | 1.69× (1.65–1.74) |

**Best:** mibz2 at 4 threads (685 MB/s); lbzip2 at 4 threads (189 MB/s); pbz2 at 4 threads (286 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.69, mibz2 4t 1.06, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.49, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.90 | 0.02 | 0.09 | 0.306 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2910 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 11.6% |
| MTF + RLE2 | 1.43 | 16.5% |
| T-vector build | 1.90 | 21.9% |
| walk (the one each build uses) | 3.84 | 44.3% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.26 | 3.0% |
| CRC | 0.23 | 2.7% |
| **total** | 8.67 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.302 | (in its total) | 1.93× (1.92–1.95) |  |
| walk | 1.253 | 2.647 | (in its total) | 2.11× (2.08–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.059 | (in its total) | 1.22× (1.17–1.27) |  |
| inverse BWT, total | 1.972 | 4.009 | 3.327 | 2.03× (2.01–2.05) | 1.68× (1.67–1.70) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h35m ± 11m20s |
| 4 threads | 45m24s ± 6m42s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m53s ± 4m58s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 32m53s ± 4m58s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 4.31 ns a byte of output at 4 threads (CPU, summed over them), 2.63 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,128,398 | 12,600,596 | \`cf6e7dd1a28cfd1d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2909.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
