# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-75/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c24437f`](<https://github.com/lmmx/mibz2/commit/c24437f9857db7d3258ad3744e7a0f8a9771e80c>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40829735109>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.1.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.436 s | 481 | 8.6 | 2.07× (1.99–2.16) | 5.19× (5.00–5.40) | 5.36× (5.20–5.53) | 6.08× (5.85–6.32) | 5.46× (5.31–5.61) | 5.69× (5.47–5.93) | 5.25× (5.07–5.45) | 1.61× (1.55–1.67) | 1.87× (1.79–1.95) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.292 s | 717 | 9.8 | 1.70× (1.55–1.86) | 3.82× (3.50–4.17) | 4.52× (4.16–4.91) | 4.50× (4.11–4.94) | 4.12× (3.75–4.53) | 7.92× (7.19–8.73) | 7.31× (6.66–8.03) | 2.24× (2.03–2.47) | 2.60× (2.35–2.87) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.297 s | 706 | 19.0 | 1.49× (1.41–1.57) | 2.43× (2.30–2.56) | 3.09× (2.91–3.29) | 2.88× (2.73–3.04) | 3.01× (2.88–3.14) | 8.34× (7.91–8.79) | 7.70× (7.30–8.12) | 2.36× (2.22–2.50) | 2.74× (2.59–2.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,266 | 3.6 | 0.54× (0.53–0.55) | 3.33× (3.17–3.50) | 4.40× (4.19–4.62) | 3.48× (3.38–3.58) | 6.56× (6.39–6.73) | 2.60× (2.53–2.66) | 2.58× (2.52–2.65) | 1.02× (1.00–1.04) | 2.04× (1.89–2.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,262 | 3.6 | 0.49× (0.47–0.51) | 2.81× (2.58–3.07) | 5.83× (5.55–6.11) | 3.80× (3.36–4.29) | 6.93× (6.23–7.70) | 2.59× (2.49–2.70) | 2.58× (2.46–2.70) | 1.02× (0.98–1.06) | 2.04× (1.88–2.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,250 | 3.6 | 0.54× (0.46–0.63) | 2.89× (2.65–3.14) | 5.84× (5.72–5.96) | 3.51× (3.46–3.57) | 6.54× (6.43–6.65) | 2.60× (2.55–2.64) | 2.58× (2.53–2.63) | 1.02× (1.00–1.04) | 2.04× (1.89–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 790 | 5.5 | 2.28× (2.19–2.38) | 2.87× (2.77–2.98) | 3.39× (3.23–3.56) | 3.53× (3.41–3.66) | 4.48× (4.30–4.66) | 2.17× (2.09–2.25) | 2.18× (1.91–2.49) | 1.66× (1.60–1.73) | 2.46× (2.37–2.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,170 | 7.0 | 1.69× (1.39–2.07) | 2.26× (1.82–2.79) | 3.57× (2.88–4.43) | 2.92× (2.34–3.62) | 4.66× (3.86–5.63) | 2.84× (2.35–3.44) | 2.85× (2.26–3.61) | 2.18× (1.83–2.60) | 3.21× (2.66–3.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 914 | 14.3 | 1.37× (1.32–1.42) | 1.58× (1.47–1.71) | 2.41× (2.21–2.64) | 1.81× (1.72–1.90) | 3.86× (3.37–4.41) | 2.50× (2.37–2.64) | 2.51× (2.22–2.83) | 1.91× (1.81–2.02) | 2.83× (2.69–2.97) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.094 s | 86 | 48.9 | 1.24× (1.22–1.27) | 2.86× (2.80–2.92) | 2.49× (2.38–2.59) | 2.54× (2.43–2.65) | 2.87× (2.79–2.95) | 3.32× (3.25–3.40) | 3.09× (3.03–3.15) | 1.25× (1.22–1.27) | 1.34× (1.31–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.058 s | 139 | 55.5 | 1.19× (1.13–1.25) | 2.68× (2.56–2.80) | 2.47× (2.37–2.56) | 3.06× (2.93–3.21) | 2.99× (2.87–3.11) | 5.33× (5.19–5.48) | 4.96× (4.84–5.09) | 2.00× (1.94–2.07) | 2.15× (2.08–2.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 164 | 75.5 | 1.16× (1.12–1.20) | 2.22× (2.14–2.31) | 2.26× (2.14–2.38) | 2.60× (2.51–2.69) | 3.02× (2.93–3.11) | 6.40× (6.25–6.55) | 5.95× (5.83–6.08) | 2.40× (2.34–2.47) | 2.58× (2.52–2.64) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.121 s | 80 | 52.2 | 1.05× (1.00–1.10) | 2.64× (2.49–2.79) | 2.10× (1.98–2.22) | 2.15× (2.04–2.27) | 2.49× (2.36–2.62) | 3.31× (3.20–3.43) | 3.21× (3.04–3.39) | 1.15× (1.09–1.21) | 1.26× (1.19–1.34) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 59.4 | 1.01× (0.94–1.10) | 2.57× (2.24–2.95) | 2.12× (1.99–2.26) | 2.97× (2.79–3.16) | 2.72× (2.42–3.05) | 5.54× (5.21–5.89) | 5.37× (5.08–5.68) | 1.93× (1.82–2.04) | 2.11× (1.98–2.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 166 | 78.1 | 1.12× (1.10–1.14) | 2.24× (2.12–2.36) | 2.12× (2.05–2.19) | 2.86× (2.69–3.04) | 2.73× (2.67–2.80) | 7.14× (6.97–7.32) | 6.92× (6.70–7.15) | 2.48× (2.41–2.56) | 2.73× (2.66–2.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 930 | 5.1 | 0.62× (0.55–0.71) | 2.86× (2.82–2.91) | 3.96× (3.89–4.03) | 2.92× (2.86–2.98) | 7.20× (7.06–7.34) | 2.16× (2.13–2.20) | 2.15× (2.11–2.19) | 1.09× (0.84–1.41) | 1.91× (1.88–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 926 | 5.2 | 0.49× (0.48–0.50) | 2.86× (2.79–2.93) | 4.91× (4.80–5.03) | 2.93× (2.85–3.00) | 7.26× (7.04–7.48) | 2.16× (2.11–2.20) | 2.14× (2.09–2.20) | 1.08× (0.83–1.41) | 1.91× (1.86–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 897 | 5.2 | 0.49× (0.48–0.50) | 2.78× (2.75–2.82) | 4.80× (4.74–4.86) | 2.86× (2.82–2.90) | 6.91× (6.76–7.06) | 2.10× (2.06–2.13) | 2.08× (2.04–2.13) | 1.05× (0.81–1.37) | 1.85× (1.83–1.87) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 0.052 s | 234 | 18.1 | 1.31× (1.28–1.34) | 3.10× (3.00–3.20) | 3.38× (3.30–3.45) | 3.49× (3.38–3.60) | 3.93× (3.83–4.04) | 3.57× (3.48–3.66) | 3.25× (3.17–3.33) | 1.10× (1.08–1.12) | 1.38× (1.33–1.44) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 0.035 s | 354 | 22.0 | 1.14× (0.99–1.32) | 2.71× (2.42–3.03) | 3.04× (2.89–3.20) | 3.52× (3.26–3.79) | 3.76× (3.59–3.92) | 5.29× (5.04–5.54) | 4.81× (4.60–5.03) | 1.62× (1.54–1.71) | 2.05× (1.93–2.18) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 0.031 s | 395 | 34.5 | 1.07× (1.02–1.12) | 2.11× (2.00–2.24) | 2.62× (2.46–2.80) | 2.84× (2.76–2.93) | 3.55× (3.44–3.66) | 6.04× (5.84–6.25) | 5.50× (5.30–5.70) | 1.86× (1.78–1.93) | 2.34× (2.24–2.44) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.07× (1.99–2.15) | 5.17× (4.98–5.37) | 5.33× (5.17–5.50) | 6.06× (5.83–6.30) | 5.44× (5.29–5.59) | 5.67× (5.45–5.91) | 5.24× (5.05–5.43) | 1.61× (1.55–1.67) | 1.85× (1.77–1.93) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.80× (1.68–1.93) | 4.46× (4.18–4.77) | 4.74× (4.49–5.01) | 5.37× (5.00–5.77) | 4.62× (4.28–4.99) | 4.77× (4.41–5.17) | 4.41× (4.09–4.75) | 1.35× (1.25–1.47) | 1.56× (1.44–1.69) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.55× (1.46–1.66) | 2.88× (2.73–3.05) | 3.11× (2.93–3.31) | 3.42× (3.24–3.62) | 3.07× (2.94–3.20) | 2.53× (2.39–2.68) | 2.34× (2.21–2.47) | 0.72× (0.67–0.76) | 0.82× (0.78–0.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.59) | 2.93× (2.85–3.01) | 3.79× (3.72–3.86) | 3.26× (3.15–3.37) | 6.05× (5.87–6.23) | 2.46× (2.38–2.53) | 2.44× (2.37–2.51) | 1.02× (1.00–1.05) | 1.78× (1.72–1.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.57) | 2.69× (2.58–2.80) | 4.90× (4.71–5.09) | 3.55× (3.11–4.06) | 6.29× (5.66–7.00) | 2.41× (2.32–2.50) | 2.39× (2.29–2.50) | 1.00× (0.97–1.03) | 1.75× (1.68–1.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.50–0.69) | 2.94× (2.89–2.99) | 5.06× (4.98–5.13) | 3.34× (3.29–3.39) | 6.08× (5.98–6.17) | 2.47× (2.43–2.52) | 2.46× (2.41–2.51) | 1.03× (1.01–1.05) | 1.80× (1.76–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.19× (2.06–2.34) | 2.54× (2.38–2.70) | 3.10× (2.93–3.28) | 3.35× (3.18–3.53) | 4.23× (3.98–4.49) | 2.09× (1.97–2.20) | 2.09× (1.82–2.41) | 1.61× (1.52–1.70) | 2.16× (2.05–2.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.83× (1.77–1.89) | 2.17× (2.04–2.30) | 2.78× (2.44–3.17) | 2.81× (2.68–2.94) | 3.45× (3.31–3.59) | 1.63× (1.56–1.71) | 1.63× (1.46–1.83) | 1.26× (1.21–1.30) | 1.69× (1.62–1.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.50× (1.42–1.57) | 1.46× (1.37–1.56) | 1.70× (1.58–1.84) | 1.71× (1.60–1.83) | 1.95× (1.70–2.25) | 0.79× (0.73–0.85) | 0.79× (0.71–0.88) | 0.61× (0.56–0.65) | 0.81× (0.75–0.88) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.24× (1.21–1.26) | 2.77× (2.71–2.84) | 2.39× (2.30–2.49) | 2.52× (2.41–2.63) | 2.84× (2.76–2.92) | 3.28× (3.21–3.36) | 3.06× (3.00–3.12) | 1.24× (1.22–1.27) | 1.28× (1.26–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.16× (1.14–1.19) | 2.59× (2.54–2.65) | 2.22× (2.18–2.26) | 2.84× (2.79–2.89) | 2.58× (2.49–2.67) | 2.90× (2.85–2.94) | 2.70× (2.65–2.75) | 1.10× (1.07–1.13) | 1.13× (1.11–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.26× (1.23–1.29) | 2.28× (2.22–2.35) | 2.00× (1.91–2.08) | 2.46× (2.40–2.53) | 2.44× (2.37–2.51) | 2.10× (2.07–2.14) | 1.96× (1.93–1.99) | 0.80× (0.78–0.81) | 0.82× (0.81–0.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.05× (1.00–1.10) | 2.57× (2.43–2.72) | 2.03× (1.92–2.15) | 2.13× (2.02–2.25) | 2.46× (2.33–2.60) | 3.28× (3.15–3.40) | 3.18× (3.00–3.36) | 1.15× (1.09–1.21) | 1.21× (1.15–1.28) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.02× (0.99–1.05) | 2.58× (2.40–2.78) | 1.98× (1.92–2.03) | 2.78× (2.71–2.86) | 2.40× (2.20–2.63) | 2.99× (2.93–3.06) | 2.90× (2.83–2.97) | 1.05× (1.03–1.07) | 1.11× (1.08–1.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.22× (1.20–1.25) | 2.37× (2.27–2.47) | 1.77× (1.73–1.81) | 2.59× (2.43–2.76) | 2.27× (2.21–2.34) | 2.24× (2.19–2.30) | 2.17× (2.10–2.25) | 0.79× (0.76–0.81) | 0.83× (0.81–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.60–0.78) | 2.41× (2.37–2.45) | 3.27× (3.20–3.34) | 2.66× (2.62–2.71) | 6.36× (6.25–6.48) | 2.00× (1.97–2.03) | 2.00× (1.96–2.03) | 1.08× (0.85–1.38) | 1.42× (1.39–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.37× (2.19–2.55) | 4.03× (3.93–4.13) | 2.67× (2.61–2.73) | 6.30× (6.12–6.47) | 1.95× (1.92–1.99) | 1.94× (1.91–1.98) | 1.05× (0.83–1.34) | 1.38× (1.36–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.56) | 2.44× (2.33–2.55) | 4.01× (3.94–4.08) | 2.72× (2.69–2.76) | 6.26× (6.11–6.42) | 1.98× (1.95–2.01) | 1.97× (1.93–2.01) | 1.07× (0.84–1.36) | 1.40× (1.37–1.43) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 1.30× (1.27–1.33) | 2.92× (2.85–3.00) | 3.19× (3.11–3.28) | 3.41× (3.31–3.52) | 3.84× (3.74–3.94) | 3.49× (3.41–3.58) | 3.18× (3.10–3.25) | 1.09× (1.07–1.12) | 1.25× (1.18–1.33) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 1.17× (1.12–1.23) | 2.62× (2.33–2.94) | 2.81× (2.73–2.89) | 3.18× (3.01–3.36) | 3.21× (3.12–3.30) | 2.88× (2.78–2.98) | 2.62× (2.52–2.72) | 0.90× (0.87–0.94) | 1.03× (0.98–1.09) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 1.16× (1.13–1.19) | 2.07× (1.96–2.18) | 2.32× (2.18–2.47) | 2.44× (2.38–2.49) | 2.53× (2.47–2.58) | 1.82× (1.76–1.89) | 1.66× (1.59–1.73) | 0.57× (0.55–0.59) | 0.66× (0.61–0.70) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.36× (0.35–0.37) | 1.17× (1.16–1.17) | 0.84× (0.83–0.86) | 11.84× (11.82–11.85) | 0.28× (0.28–0.28) | 0.30× (0.29–0.30) | 0.45× (0.45–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.58× (0.50–0.68) | 0.86× (0.84–0.88) | 0.68× (0.66–0.69) | 6.37× (6.25–6.49) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.67× (0.58–0.77) | 0.78× (0.77–0.79) | 0.65× (0.64–0.66) | 4.27× (4.25–4.29) | 0.10× (0.10–0.10) | 0.11× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.28–4.28) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.15–0.17) | 1.77× (1.77–1.78) | 0.83× (0.83–0.83) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.43× (0.43–0.43) | 1.63× (1.62–1.64) | 0.63× (0.63–0.64) | 7.75× (7.73–7.78) | 0.62× (0.62–0.62) | 0.65× (0.65–0.66) | 0.71× (0.71–0.72) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.63× (0.61–0.66) | 0.60× (0.55–0.65) | 1.54× (1.52–1.57) | 0.88× (0.86–0.90) | 5.56× (5.48–5.65) | 0.44× (0.44–0.45) | 0.47× (0.46–0.47) | 0.51× (0.50–0.52) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.60–0.63) | 0.70× (0.66–0.73) | 1.10× (1.08–1.13) | 0.86× (0.84–0.87) | 3.21× (3.16–3.27) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.14× (1.14–1.15) | 0.71× (0.71–0.71) | 5.05× (5.04–5.06) | 0.31× (0.31–0.31) | 0.33× (0.33–0.34) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.49–0.52) | 0.43× (0.43–0.44) | 0.92× (0.90–0.94) | 0.74× (0.73–0.75) | 3.22× (3.17–3.26) | 0.20× (0.19–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.58–0.61) | 0.42× (0.42–0.43) | 0.81× (0.80–0.82) | 0.76× (0.74–0.78) | 2.37× (2.34–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.11× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.32) | 1.15× (1.15–1.15) | 0.81× (0.81–0.81) | 5.09× (5.09–5.10) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.26× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.48–0.49) | 0.38× (0.36–0.41) | 0.87× (0.85–0.88) | 0.74× (0.71–0.76) | 2.98× (2.94–3.02) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.55× (0.54–0.57) | 0.44× (0.41–0.46) | 0.74× (0.73–0.76) | 0.76× (0.73–0.79) | 2.15× (2.09–2.20) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.22) | 1.08× (1.08–1.08) | 0.77× (0.76–0.77) | 5.42× (5.40–5.43) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.76–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.40) | 1.45× (1.45–1.46) | 0.63× (0.63–0.64) | 6.56× (6.54–6.57) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.64× (0.64–0.65) | 0.32× (0.31–0.32) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 31 MB | 0.52× (0.50–0.54) | 0.43× (0.39–0.48) | 1.27× (1.23–1.31) | 0.78× (0.74–0.81) | 3.99× (3.86–4.13) | 0.24× (0.23–0.25) | 0.26× (0.25–0.26) | 0.39× (0.38–0.41) | 0.19× (0.19–0.20) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 50 MB | 0.53× (0.52–0.55) | 0.52× (0.48–0.56) | 1.02× (1.00–1.05) | 0.79× (0.77–0.81) | 2.58× (2.53–2.62) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 481 | 2.08 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 717 | 2.38 | 1.19× (1.13–1.25) | 70% | 1.03× (1.01–1.04) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 706 | 4.63 | 2.24× (2.12–2.37) | 37% | 1.25× (1.21–1.30) | 1.68× (1.58–1.80) |

**Best:** mibz2 at 2 threads (717 MB/s); lbzip2 at 4 threads (294 MB/s); pbz2 at 4 threads (487 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 0.99, lbzip2 1t 0.08, lbzip2 2t 0.30, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.81 | 0.01 | 0.09 | 0.297 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4105 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 42.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.05 | 12.8% |
| MTF + RLE2 | 1.89 | 23.1% |
| T-vector build | 1.45 | 17.8% |
| walk (the one each build uses) | 3.39 | 41.5% |
| of which: segment assembly | 0.11 | 1.4% |
| RLE1 | 0.15 | 1.8% |
| CRC | 0.25 | 3.1% |
| **total** | 8.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build takes the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | bz2ngrs / mibz2 |
| --- | ---: | ---: | ---: |
| T-vector build | 0.333 | 0.741 | 2.22× (2.21–2.23) |
| walk | 0.755 | 1.651 | 2.15× (2.09–2.22) |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.030 | 1.21× (1.20–1.23) |
| inverse BWT, total | 1.112 | 2.422 | 2.15× (2.11–2.20) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h02m ± 7m55s |
| 4 threads | 42m58s ± 6m23s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m34s ± 4m32s |
| the scanner alone (its serial part) | 9m36s ± 43.7s |
| the workers alone, over 4 | 29m34s ± 4m32s |

At 4 threads the workers bound it. Per byte: the scanner 5.6 ns a byte of bz2, the workers 3.88 ns a byte of output at 4 threads (CPU, summed over them), 1.75 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,478,819 | 12,273,030 | \`80f51ade7d6ce928…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4104.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 3.1.0 (bench/report.py), 10 rounds, threads 1, 2, 4
