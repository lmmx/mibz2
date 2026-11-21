# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-689/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`10207ff`](<https://github.com/lmmx/mibz2/commit/10207ffa2c231141f46b593bccd1a4e6005a699f>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44892597123>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.504 s | 416 | 7.8 | 2.98× (2.96–3.00) | 6.82× (6.77–6.88) | 6.98× (6.92–7.04) | 7.62× (7.57–7.68) | 6.84× (6.80–6.89) | 7.47× (7.42–7.52) | 6.90× (6.84–6.95) | 2.21× (2.19–2.22) | 2.59× (2.56–2.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.299 s | 702 | 8.6 | 2.91× (2.80–3.02) | 6.21× (5.98–6.45) | 7.01× (6.77–7.25) | 6.97× (6.76–7.19) | 6.26× (6.06–6.46) | 12.61× (12.21–13.03) | 11.64× (11.28–12.00) | 3.72× (3.62–3.84) | 4.37× (4.22–4.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.236 s | 886 | 13.2 | 3.00× (2.95–3.06) | 4.50× (4.43–4.56) | 5.66× (5.53–5.80) | 5.18× (5.10–5.25) | 5.22× (5.12–5.32) | 15.89× (15.66–16.13) | 14.67× (14.45–14.89) | 4.69× (4.62–4.77) | 5.50× (5.40–5.61) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 966 | 3.7 | 0.59× (0.59–0.60) | 3.65× (3.49–3.83) | 4.38× (4.35–4.42) | 3.56× (3.53–3.59) | 6.17× (6.12–6.22) | 2.95× (2.92–2.97) | 2.63× (2.61–2.66) | 1.00× (0.98–1.02) | 1.49× (1.48–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 3.8 | 0.56× (0.55–0.57) | 2.96× (2.93–2.99) | 5.92× (5.78–6.07) | 3.53× (3.49–3.57) | 6.04× (5.98–6.10) | 2.92× (2.89–2.95) | 2.61× (2.58–2.64) | 0.99× (0.97–1.01) | 1.48× (1.46–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 943 | 3.8 | 0.56× (0.55–0.57) | 2.92× (2.88–2.97) | 5.80× (5.73–5.86) | 3.48× (3.43–3.53) | 6.03× (5.95–6.11) | 2.88× (2.84–2.92) | 2.57× (2.53–2.61) | 0.98× (0.95–1.00) | 1.46× (1.43–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 582 | 5.9 | 2.61× (2.58–2.64) | 3.02× (2.98–3.06) | 3.91× (3.82–4.01) | 3.81× (3.77–3.85) | 4.20× (4.15–4.26) | 2.55× (2.51–2.60) | 2.11× (2.09–2.13) | 2.01× (1.99–2.04) | 3.01× (2.97–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 800 | 7.3 | 2.00× (1.74–2.31) | 2.50× (2.14–2.91) | 3.49× (2.86–4.26) | 2.94× (2.43–3.57) | 4.18× (3.50–4.99) | 3.30× (2.78–3.92) | 2.73× (2.28–3.26) | 2.60× (2.18–3.11) | 3.89× (3.25–4.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 775 | 13.6 | 1.88× (1.79–1.98) | 2.00× (1.85–2.16) | 2.91× (2.80–3.02) | 2.24× (2.11–2.39) | 3.97× (3.83–4.11) | 3.39× (3.28–3.51) | 2.81× (2.71–2.90) | 2.68× (2.59–2.76) | 4.00× (3.87–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 53.4 | 1.36× (1.34–1.38) | 2.70× (2.65–2.75) | 2.42× (2.38–2.46) | 2.46× (2.43–2.49) | 2.89× (2.83–2.94) | 3.15× (3.12–3.19) | 3.17× (3.13–3.20) | 1.22× (1.21–1.23) | 1.40× (1.38–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 60.0 | 1.28× (1.25–1.31) | 2.55× (2.49–2.61) | 2.39× (2.35–2.42) | 3.08× (3.01–3.15) | 2.93× (2.88–2.98) | 5.07× (4.99–5.16) | 5.10× (5.01–5.18) | 1.97× (1.93–2.00) | 2.25× (2.22–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 80.7 | 1.32× (1.27–1.37) | 2.10× (2.01–2.20) | 2.02× (1.96–2.07) | 2.62× (2.45–2.80) | 2.84× (2.75–2.93) | 5.98× (5.82–6.15) | 6.01× (5.84–6.18) | 2.32× (2.26–2.38) | 2.66× (2.58–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.4 | 1.49× (1.48–1.49) | 3.07× (3.01–3.13) | 2.53× (2.51–2.56) | 2.63× (2.60–2.65) | 3.08× (3.06–3.11) | 3.64× (3.61–3.67) | 3.87× (3.83–3.90) | 1.41× (1.39–1.42) | 1.70× (1.68–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 56.0 | 1.42× (1.27–1.58) | 2.76× (2.61–2.93) | 2.47× (2.37–2.56) | 3.37× (3.27–3.48) | 3.05× (2.96–3.15) | 5.91× (5.73–6.10) | 6.28× (6.07–6.49) | 2.28× (2.21–2.36) | 2.76× (2.67–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 127 | 77.7 | 1.46× (1.42–1.50) | 2.33× (2.26–2.40) | 2.24× (2.19–2.30) | 3.04× (2.95–3.15) | 2.85× (2.79–2.92) | 6.91× (6.77–7.05) | 7.34× (7.22–7.47) | 2.67× (2.63–2.71) | 3.23× (3.16–3.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 5.1 | 0.66× (0.66–0.67) | 3.01× (2.97–3.05) | 4.41× (4.35–4.48) | 3.09× (3.04–3.14) | 6.80× (6.68–6.93) | 2.52× (2.48–2.56) | 2.31× (2.28–2.35) | 0.98× (0.97–0.99) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 694 | 5.4 | 0.55× (0.54–0.56) | 2.84× (2.79–2.89) | 4.89× (4.80–4.98) | 2.95× (2.90–3.00) | 6.55× (6.41–6.69) | 2.37× (2.34–2.41) | 2.18× (2.14–2.22) | 0.92× (0.90–0.94) | 1.43× (1.41–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 5.4 | 0.55× (0.54–0.56) | 2.80× (2.76–2.84) | 4.80× (4.74–4.87) | 2.90× (2.86–2.95) | 6.43× (6.33–6.53) | 2.33× (2.30–2.37) | 2.14× (2.11–2.18) | 0.90× (0.89–0.92) | 1.41× (1.39–1.42) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.071 s | 187 | 17.8 | 1.97× (1.96–1.99) | 4.12× (4.09–4.16) | 4.43× (4.40–4.46) | 4.48× (4.44–4.52) | 4.69× (4.66–4.73) | 5.23× (5.18–5.28) | 4.79× (4.75–4.83) | 1.51× (1.50–1.52) | 1.84× (1.83–1.86) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.049 s | 276 | 22.2 | 1.64× (1.58–1.70) | 3.52× (3.21–3.85) | 4.04× (3.93–4.15) | 4.34× (4.19–4.49) | 4.39× (4.24–4.55) | 7.66× (7.46–7.86) | 7.01× (6.84–7.19) | 2.22× (2.15–2.28) | 2.70× (2.64–2.77) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.045 s | 297 | 34.0 | 1.52× (1.44–1.59) | 2.41× (2.28–2.55) | 3.19× (3.08–3.32) | 3.24× (3.06–3.43) | 3.72× (3.60–3.85) | 8.23× (7.97–8.50) | 7.53× (7.28–7.79) | 2.38× (2.30–2.47) | 2.90× (2.81–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.97× (2.95–2.99) | 6.79× (6.74–6.85) | 6.95× (6.90–7.01) | 7.60× (7.54–7.65) | 6.82× (6.77–6.86) | 7.45× (7.40–7.50) | 6.87× (6.82–6.92) | 2.20× (2.19–2.22) | 2.57× (2.54–2.60) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.85× (2.82–2.87) | 6.38× (6.34–6.43) | 6.69× (6.64–6.74) | 7.41× (7.36–7.46) | 6.34× (6.26–6.43) | 6.78× (6.73–6.82) | 6.25× (6.21–6.30) | 2.00× (1.99–2.02) | 2.34× (2.31–2.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.93× (2.87–3.00) | 4.90× (4.86–4.94) | 5.28× (5.23–5.33) | 5.61× (5.55–5.67) | 5.01× (4.93–5.08) | 4.44× (4.41–4.48) | 4.10× (4.07–4.14) | 1.31× (1.31–1.32) | 1.54× (1.52–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.63) | 3.24× (3.21–3.26) | 4.03× (4.00–4.06) | 3.33× (3.30–3.35) | 5.71× (5.66–5.76) | 2.77× (2.75–2.80) | 2.49× (2.47–2.50) | 1.00× (0.99–1.02) | 1.04× (1.02–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.05× (3.02–3.07) | 5.33× (5.28–5.38) | 3.28× (3.25–3.32) | 5.53× (5.48–5.59) | 2.71× (2.68–2.74) | 2.43× (2.41–2.45) | 0.98× (0.96–0.99) | 1.02× (1.00–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.31× (3.27–3.35) | 5.31× (5.25–5.38) | 3.29× (3.24–3.34) | 5.58× (5.52–5.65) | 2.71× (2.68–2.75) | 2.43× (2.40–2.47) | 0.98× (0.96–1.00) | 1.02× (0.99–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.49–2.55) | 2.78× (2.74–2.81) | 3.52× (3.42–3.63) | 3.66× (3.62–3.70) | 4.03× (3.98–4.08) | 2.47× (2.43–2.50) | 2.05× (2.03–2.07) | 1.96× (1.93–1.98) | 2.70× (2.66–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.20× (2.16–2.25) | 2.46× (2.40–2.52) | 3.00× (2.92–3.08) | 3.09× (3.03–3.16) | 3.37× (3.29–3.45) | 1.98× (1.93–2.02) | 1.64× (1.61–1.68) | 1.57× (1.53–1.61) | 2.16× (2.11–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.92–2.17) | 1.76× (1.70–1.83) | 2.24× (2.17–2.31) | 2.07× (1.99–2.15) | 2.19× (2.13–2.26) | 1.05× (1.02–1.08) | 0.87× (0.85–0.90) | 0.83× (0.81–0.86) | 1.15× (1.12–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.37) | 2.63× (2.60–2.67) | 2.38× (2.33–2.42) | 2.44× (2.41–2.47) | 2.86× (2.81–2.91) | 3.12× (3.09–3.16) | 3.13× (3.10–3.17) | 1.22× (1.21–1.23) | 1.37× (1.35–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.48× (2.45–2.51) | 2.19× (2.16–2.21) | 2.93× (2.89–2.97) | 2.58× (2.56–2.61) | 2.77× (2.74–2.80) | 2.78× (2.75–2.82) | 1.08× (1.07–1.10) | 1.21× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.45–1.55) | 2.33× (2.29–2.36) | 1.91× (1.87–1.95) | 2.63× (2.58–2.69) | 2.53× (2.48–2.59) | 2.08× (2.04–2.13) | 2.09× (2.05–2.14) | 0.81× (0.80–0.83) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.47–1.49) | 3.01× (2.95–3.06) | 2.50× (2.48–2.52) | 2.60× (2.58–2.63) | 3.06× (3.03–3.08) | 3.60× (3.57–3.63) | 3.82× (3.79–3.86) | 1.40× (1.39–1.41) | 1.66× (1.65–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.40) | 2.77× (2.74–2.79) | 2.31× (2.29–2.32) | 3.22× (3.20–3.24) | 2.76× (2.74–2.78) | 3.18× (3.16–3.20) | 3.37× (3.34–3.41) | 1.24× (1.22–1.25) | 1.46× (1.45–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.53–1.73) | 2.54× (2.48–2.60) | 1.96× (1.93–2.00) | 2.81× (2.73–2.89) | 2.62× (2.56–2.68) | 2.29× (2.25–2.34) | 2.44× (2.39–2.48) | 0.89× (0.87–0.91) | 1.05× (1.03–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.70× (2.66–2.74) | 3.54× (3.48–3.61) | 2.83× (2.78–2.87) | 6.07× (5.97–6.18) | 2.33× (2.29–2.36) | 2.15× (2.11–2.18) | 0.98× (0.97–1.00) | 0.97× (0.93–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.50× (2.46–2.54) | 4.26× (4.22–4.31) | 2.74× (2.70–2.78) | 5.80× (5.69–5.92) | 2.17× (2.14–2.20) | 2.00× (1.97–2.04) | 0.92× (0.90–0.93) | 0.90× (0.87–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.69× (2.66–2.72) | 4.25× (4.18–4.32) | 2.75× (2.71–2.78) | 5.81× (5.73–5.89) | 2.17× (2.15–2.20) | 2.00× (1.98–2.03) | 0.92× (0.91–0.93) | 0.90× (0.87–0.93) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.95× (1.93–1.96) | 3.95× (3.92–3.98) | 4.29× (4.26–4.31) | 4.38× (4.35–4.42) | 4.60× (4.56–4.63) | 5.12× (5.07–5.17) | 4.69× (4.65–4.72) | 1.50× (1.49–1.51) | 1.78× (1.77–1.79) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.67× (1.64–1.69) | 3.34× (3.28–3.39) | 3.66× (3.60–3.72) | 4.08× (4.00–4.15) | 3.83× (3.72–3.96) | 4.10× (4.04–4.17) | 3.76× (3.70–3.82) | 1.20× (1.18–1.23) | 1.43× (1.40–1.45) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.75× (1.69–1.80) | 2.64× (2.58–2.70) | 2.91× (2.85–2.97) | 3.20× (3.13–3.28) | 3.03× (2.99–3.08) | 2.68× (2.64–2.73) | 2.46× (2.42–2.50) | 0.79× (0.77–0.80) | 0.93× (0.92–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.28–0.31) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.95× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.51× (0.43–0.60) | 0.67× (0.65–0.69) | 0.57× (0.55–0.59) | 5.37× (5.20–5.54) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.32–0.35) | 0.61× (0.52–0.72) | 0.59× (0.56–0.61) | 0.52× (0.50–0.54) | 3.41× (3.26–3.57) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.36× (4.35–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.66–0.67) | 7.98× (7.95–8.00) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.71× (0.66–0.75) | 1.60× (1.56–1.63) | 0.80× (0.78–0.82) | 5.78× (5.65–5.90) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.63× (0.60–0.67) | 0.70× (0.63–0.79) | 1.13× (1.09–1.16) | 0.69× (0.66–0.71) | 3.32× (3.21–3.43) | 0.26× (0.25–0.27) | 0.27× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.74) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.38–0.44) | 0.91× (0.88–0.94) | 0.73× (0.71–0.76) | 3.16× (3.06–3.26) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.87× (0.84–0.89) | 0.81× (0.79–0.83) | 2.51× (2.45–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.18× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.38× (0.35–0.41) | 0.85× (0.82–0.88) | 0.69× (0.66–0.72) | 2.91× (2.82–3.01) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.83) | 0.80× (0.78–0.82) | 2.32× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.77× (0.76–0.77) | 5.34× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.67–0.68) | 6.38× (6.36–6.40) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.43–0.46) | 0.41× (0.38–0.43) | 1.05× (1.01–1.08) | 0.68× (0.65–0.71) | 3.28× (3.17–3.39) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.34) | 0.16× (0.15–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 58 MB | 0.49× (0.47–0.50) | 0.48× (0.45–0.52) | 0.90× (0.87–0.93) | 0.73× (0.68–0.77) | 2.27× (2.20–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 416 | 2.41 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 702 | 2.65 | 1.10× (1.09–1.11) | 84% | 1.03× (1.03–1.04) | 1.05× (1.05–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 886 | 4.06 | 1.68× (1.66–1.69) | 53% | 1.21× (1.20–1.21) | 1.65× (1.62–1.68) |

**Best:** mibz2 at 4 threads (886 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.21, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.65 | 0.02 | 0.09 | 0.236 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.9% |
| MTF + RLE2 | 1.52 | 17.2% |
| T-vector build | 1.52 | 17.3% |
| walk (the one each build uses) | 4.10 | 46.6% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.302 | (in its total) | 2.64× (2.63–2.65) |  |
| walk | 1.238 | 2.631 | (in its total) | 2.12× (2.11–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.21× (1.20–1.23) |  |
| inverse BWT, total | 1.777 | 3.987 | 3.318 | 2.24× (2.23–2.26) | 1.87× (1.86–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m20s |
| 4 threads | 34m35s ± 4m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m19s ± 3m17s |
| the scanner alone (its serial part) | 14m07s ± 30.2s |
| the workers alone, over 4 | 23m19s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 3.06 ns a byte of output at 4 threads (CPU, summed over them), 1.95 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,633,100 | 13,390,258 | \`b7ebdafdab81f6ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
