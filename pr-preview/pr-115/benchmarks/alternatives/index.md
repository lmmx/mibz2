# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-115/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`6a341f6`](<https://github.com/lmmx/mibz2/commit/6a341f6a79c4926cbd27ce24470fa645ec49d44d>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40973966802>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.415 s | 506 | 8.6 | 2.01× (1.90–2.13) | 5.20× (4.95–5.47) | 5.33× (5.12–5.54) | 6.06× (5.76–6.37) | 5.44× (5.17–5.72) | 5.78× (5.45–6.12) | 5.52× (5.27–5.78) | 1.59× (1.50–1.68) | 1.86× (1.78–1.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.282 s | 744 | 9.4 | 1.70× (1.49–1.94) | 3.78× (3.23–4.44) | 4.42× (3.82–5.12) | 4.51× (3.86–5.27) | 4.26× (3.87–4.70) | 7.95× (6.78–9.32) | 7.59× (6.85–8.42) | 2.19× (1.87–2.56) | 2.56× (2.21–2.96) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.274 s | 765 | 17.8 | 1.58× (1.43–1.73) | 2.50× (2.40–2.61) | 3.08× (2.97–3.20) | 2.87× (2.75–3.00) | 2.97× (2.81–3.14) | 8.75× (8.29–9.24) | 8.36× (7.80–8.95) | 2.41× (2.28–2.54) | 2.82× (2.70–2.93) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,307 | 3.6 | 0.53× (0.52–0.55) | 3.34× (3.20–3.49) | 4.38× (3.99–4.80) | 3.47× (3.36–3.59) | 6.54× (6.34–6.74) | 2.62× (2.52–2.73) | 2.61× (2.53–2.68) | 1.02× (0.98–1.05) | 2.01× (1.92–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,294 | 3.7 | 0.51× (0.48–0.53) | 2.78× (2.57–3.01) | 5.71× (5.48–5.95) | 3.46× (3.34–3.58) | 6.40× (6.18–6.62) | 2.60× (2.49–2.72) | 2.59× (2.49–2.68) | 1.01× (0.98–1.03) | 1.99× (1.92–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,320 | 3.6 | 0.50× (0.48–0.51) | 2.90× (2.75–3.05) | 5.85× (5.28–6.48) | 3.53× (3.40–3.66) | 6.43× (6.19–6.68) | 2.59× (2.48–2.71) | 2.58× (2.49–2.67) | 1.01× (0.96–1.05) | 1.98× (1.89–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.024 s | 820 | 5.6 | 2.13× (1.95–2.32) | 2.93× (2.65–3.24) | 3.39× (3.07–3.75) | 3.47× (3.18–3.79) | 4.50× (4.12–4.91) | 2.18× (2.00–2.38) | 1.97× (1.82–2.14) | 1.64× (1.51–1.79) | 2.47× (2.27–2.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,221 | 7.1 | 1.76× (1.53–2.02) | 2.36× (2.04–2.72) | 3.44× (2.94–4.03) | 3.09× (2.60–3.67) | 4.93× (4.29–5.66) | 3.10× (2.68–3.59) | 2.80× (2.45–3.20) | 2.33× (2.06–2.64) | 3.51× (3.04–4.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,005 | 14.1 | 1.43× (1.37–1.49) | 1.66× (1.57–1.75) | 2.54× (2.34–2.75) | 1.86× (1.77–1.95) | 3.95× (3.79–4.11) | 2.71× (2.54–2.89) | 2.45× (2.32–2.58) | 2.04× (1.95–2.13) | 3.07× (2.89–3.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.088 s | 91 | 48.5 | 1.28× (1.24–1.33) | 2.91× (2.80–3.03) | 2.48× (2.39–2.58) | 2.51× (2.42–2.59) | 2.89× (2.79–3.00) | 3.31× (3.20–3.43) | 3.41× (3.06–3.78) | 1.22× (1.18–1.27) | 1.40× (1.32–1.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.057 s | 142 | 55.9 | 1.15× (1.09–1.22) | 2.54× (2.33–2.78) | 2.37× (2.23–2.52) | 3.03× (2.88–3.18) | 2.90× (2.70–3.11) | 5.04× (4.69–5.41) | 5.18× (4.80–5.59) | 1.86× (1.73–2.00) | 2.13× (1.97–2.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.046 s | 173 | 75.4 | 1.18× (1.15–1.21) | 2.26× (2.19–2.33) | 2.26× (2.13–2.40) | 2.62× (2.52–2.73) | 3.03× (2.96–3.11) | 6.33× (6.22–6.44) | 6.51× (5.77–7.35) | 2.34× (2.28–2.40) | 2.68× (2.58–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.115 s | 83 | 52.4 | 1.09× (1.06–1.12) | 2.78× (2.72–2.85) | 2.21× (2.13–2.29) | 2.28× (2.17–2.39) | 2.62× (2.51–2.73) | 3.56× (3.33–3.80) | 3.51× (3.41–3.61) | 1.18× (1.15–1.22) | 1.34× (1.29–1.39) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 137 | 59.2 | 1.03× (0.91–1.17) | 2.52× (2.38–2.68) | 2.16× (2.06–2.26) | 2.97× (2.86–3.08) | 2.70× (2.62–2.78) | 5.84× (5.36–6.35) | 5.75× (5.55–5.96) | 1.94× (1.87–2.02) | 2.19× (2.09–2.30) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 175 | 79.6 | 1.12× (1.10–1.14) | 2.23× (2.19–2.27) | 2.16× (2.11–2.22) | 2.86× (2.78–2.94) | 2.75× (2.70–2.79) | 7.52× (7.10–7.96) | 7.41× (7.24–7.59) | 2.50× (2.44–2.56) | 2.83× (2.73–2.93) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 939 | 5.2 | 0.54× (0.46–0.63) | 2.84× (2.35–3.43) | 3.89× (3.33–4.54) | 2.73× (2.33–3.19) | 6.73× (5.71–7.93) | 2.06× (1.75–2.43) | 2.07× (1.75–2.44) | 0.91× (0.77–1.09) | 1.85× (1.56–2.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 926 | 5.4 | 0.48× (0.46–0.50) | 2.88× (2.79–2.97) | 5.08× (4.88–5.29) | 2.88× (2.76–3.00) | 7.06× (6.84–7.29) | 2.14× (2.06–2.22) | 2.15× (2.09–2.22) | 0.95× (0.91–0.99) | 1.92× (1.86–1.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 935 | 5.2 | 0.48× (0.47–0.50) | 2.85× (2.77–2.94) | 5.03× (4.86–5.20) | 2.92× (2.77–3.08) | 6.94× (6.74–7.15) | 2.12× (2.05–2.19) | 2.13× (2.09–2.18) | 0.94× (0.90–0.98) | 1.90× (1.85–1.97) | yes |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 1 | 0.053 s | 228 | 19.4 | 1.22× (1.15–1.30) | 2.99× (2.81–3.19) | 3.12× (2.94–3.31) | 3.24× (3.12–3.36) | 3.63× (3.45–3.81) | 3.36× (3.19–3.53) | 3.13× (3.00–3.27) | 1.05× (1.00–1.11) | 1.32× (1.24–1.40) | yes |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 2 | 0.034 s | 355 | 23.8 | 1.07× (1.03–1.10) | 2.62× (2.55–2.69) | 3.02× (2.93–3.11) | 3.43× (3.30–3.57) | 3.81× (3.70–3.92) | 5.33× (5.16–5.51) | 4.97× (4.84–5.11) | 1.67× (1.61–1.74) | 2.09× (1.96–2.22) | yes |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 4 | 0.030 s | 408 | 36.3 | 1.05× (1.03–1.08) | 2.02× (1.93–2.13) | 2.59× (2.34–2.87) | 2.82× (2.73–2.91) | 3.48× (3.34–3.64) | 6.01× (5.75–6.28) | 5.61× (5.41–5.81) | 1.89× (1.81–1.97) | 2.35× (2.21–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.01× (1.90–2.13) | 5.18× (4.93–5.44) | 5.30× (5.10–5.51) | 6.04× (5.75–6.36) | 5.42× (5.16–5.71) | 5.76× (5.43–6.11) | 5.50× (5.25–5.76) | 1.59× (1.50–1.68) | 1.84× (1.77–1.92) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.81× (1.64–2.01) | 4.54× (3.95–5.21) | 4.64× (4.06–5.29) | 5.45× (4.70–6.31) | 4.92× (4.49–5.39) | 4.92× (4.25–5.70) | 4.70× (4.27–5.17) | 1.36× (1.17–1.57) | 1.57× (1.38–1.79) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.67× (1.53–1.81) | 3.03× (2.91–3.16) | 3.19× (3.07–3.33) | 3.49× (3.33–3.67) | 3.09× (2.91–3.29) | 2.71× (2.55–2.88) | 2.59× (2.40–2.79) | 0.75× (0.71–0.79) | 0.87× (0.83–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.56–0.60) | 2.90× (2.78–3.03) | 3.90× (3.56–4.27) | 3.27× (3.15–3.39) | 6.08× (5.87–6.31) | 2.50× (2.39–2.62) | 2.48× (2.39–2.56) | 1.02× (0.99–1.06) | 1.79× (1.72–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.53–0.59) | 2.75× (2.62–2.89) | 4.97× (4.75–5.19) | 3.21× (3.10–3.32) | 5.83× (5.64–6.03) | 2.42× (2.32–2.53) | 2.40× (2.32–2.48) | 0.99× (0.97–1.01) | 1.73× (1.67–1.79) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.52–0.56) | 2.97× (2.90–3.04) | 5.10× (4.69–5.54) | 3.36× (3.24–3.49) | 5.98× (5.76–6.22) | 2.47× (2.37–2.59) | 2.45× (2.37–2.53) | 1.01× (0.97–1.05) | 1.77× (1.71–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.05× (1.86–2.25) | 2.56× (2.26–2.89) | 3.04× (2.68–3.46) | 3.30× (3.01–3.63) | 4.27× (3.89–4.69) | 2.11× (1.92–2.31) | 1.91× (1.75–2.07) | 1.59× (1.45–1.75) | 2.10× (1.92–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.79× (1.55–2.07) | 2.13× (1.87–2.42) | 2.54× (2.24–2.88) | 2.83× (2.41–3.32) | 3.41× (2.99–3.89) | 1.64× (1.43–1.89) | 1.49× (1.32–1.68) | 1.24× (1.11–1.40) | 1.64× (1.42–1.89) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.57× (1.50–1.65) | 1.53× (1.46–1.59) | 1.77× (1.68–1.87) | 1.76× (1.66–1.86) | 2.01× (1.92–2.10) | 0.85× (0.79–0.91) | 0.77× (0.72–0.81) | 0.64× (0.61–0.67) | 0.84× (0.79–0.90) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.28× (1.23–1.32) | 2.84× (2.74–2.95) | 2.41× (2.33–2.51) | 2.48× (2.40–2.57) | 2.87× (2.77–2.97) | 3.28× (3.16–3.40) | 3.37× (3.04–3.74) | 1.22× (1.18–1.27) | 1.33× (1.25–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.16× (1.14–1.19) | 2.55× (2.46–2.65) | 2.20× (2.11–2.28) | 2.85× (2.76–2.95) | 2.57× (2.49–2.66) | 2.82× (2.72–2.94) | 2.90× (2.61–3.23) | 1.05× (1.01–1.09) | 1.14× (1.09–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.30× (1.27–1.33) | 2.39× (2.29–2.50) | 1.97× (1.90–2.05) | 2.58× (2.49–2.68) | 2.47× (2.40–2.55) | 2.10× (2.05–2.15) | 2.16× (1.93–2.42) | 0.78× (0.75–0.81) | 0.85× (0.80–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.09× (1.06–1.12) | 2.72× (2.65–2.79) | 2.15× (2.08–2.23) | 2.26× (2.16–2.38) | 2.60× (2.50–2.72) | 3.53× (3.30–3.78) | 3.48× (3.38–3.58) | 1.18× (1.15–1.22) | 1.30× (1.26–1.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.07× (0.96–1.19) | 2.55× (2.46–2.64) | 2.01× (1.95–2.07) | 2.86× (2.80–2.93) | 2.40× (2.32–2.48) | 3.16× (2.94–3.40) | 3.11× (3.03–3.20) | 1.06× (1.03–1.08) | 1.16× (1.12–1.20) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.22× (1.21–1.23) | 2.34× (2.31–2.37) | 1.78× (1.75–1.82) | 2.56× (2.50–2.62) | 2.28× (2.23–2.34) | 2.34× (2.20–2.49) | 2.31× (2.26–2.36) | 0.78× (0.77–0.80) | 0.86× (0.84–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.59× (0.49–0.70) | 2.33× (1.92–2.83) | 3.07× (2.70–3.49) | 2.52× (2.14–2.97) | 6.05× (5.09–7.18) | 1.94× (1.63–2.30) | 1.93× (1.63–2.30) | 0.92× (0.77–1.09) | 1.35× (1.13–1.61) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.53–0.57) | 2.24× (2.17–2.31) | 3.98× (3.81–4.15) | 2.65× (2.55–2.76) | 6.21× (6.01–6.42) | 1.96× (1.89–2.04) | 1.96× (1.91–2.01) | 0.93× (0.89–0.97) | 1.37× (1.31–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.45× (2.37–2.53) | 3.99× (3.85–4.13) | 2.78× (2.63–2.93) | 6.31× (6.11–6.51) | 2.01× (1.95–2.08) | 2.01× (1.97–2.05) | 0.95× (0.91–0.99) | 1.40× (1.32–1.48) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 1 | 1.22× (1.14–1.30) | 2.85× (2.67–3.04) | 3.00× (2.82–3.20) | 3.19× (3.07–3.32) | 3.56× (3.39–3.75) | 3.31× (3.14–3.48) | 3.08× (2.95–3.22) | 1.05× (1.00–1.11) | 1.19× (1.13–1.26) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 2 | 1.08× (1.05–1.12) | 2.44× (2.39–2.48) | 2.64× (2.59–2.69) | 3.01× (2.91–3.11) | 3.09× (3.02–3.15) | 2.76× (2.66–2.86) | 2.58× (2.50–2.65) | 0.88× (0.85–0.92) | 1.00× (0.92–1.07) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 4 | 1.16× (1.13–1.19) | 1.97× (1.90–2.05) | 2.26× (2.03–2.51) | 2.39× (2.31–2.48) | 2.51× (2.36–2.66) | 1.81× (1.72–1.90) | 1.69× (1.62–1.76) | 0.58× (0.55–0.61) | 0.65× (0.61–0.70) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.36× (0.35–0.37) | 1.17× (1.16–1.17) | 0.85× (0.83–0.87) | 11.83× (11.81–11.85) | 0.28× (0.28–0.28) | 0.30× (0.29–0.30) | 0.45× (0.45–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.56× (0.46–0.67) | 0.86× (0.85–0.87) | 0.67× (0.66–0.68) | 6.30× (6.21–6.38) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.42) | 0.83× (0.74–0.93) | 0.78× (0.76–0.79) | 0.64× (0.63–0.65) | 4.30× (4.23–4.36) | 0.10× (0.10–0.10) | 0.11× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.28× (4.27–4.28) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.27× (0.26–0.27) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.26–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.44× (0.43–0.45) | 1.63× (1.63–1.63) | 0.64× (0.63–0.64) | 7.75× (7.73–7.76) | 0.62× (0.62–0.62) | 0.66× (0.65–0.66) | 0.71× (0.71–0.72) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.65× (0.63–0.67) | 0.61× (0.57–0.66) | 1.54× (1.51–1.56) | 0.88× (0.86–0.90) | 5.56× (5.46–5.66) | 0.44× (0.43–0.45) | 0.47× (0.46–0.48) | 0.51× (0.50–0.52) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.59–0.63) | 0.65× (0.61–0.70) | 1.10× (1.08–1.12) | 0.83× (0.80–0.86) | 3.21× (3.17–3.26) | 0.25× (0.24–0.25) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.72× (0.72–0.72) | 5.08× (5.07–5.09) | 0.32× (0.31–0.32) | 0.34× (0.34–0.34) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.51–0.53) | 0.43× (0.41–0.45) | 0.94× (0.93–0.94) | 0.76× (0.75–0.76) | 3.26× (3.23–3.29) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.57–0.61) | 0.42× (0.42–0.43) | 0.81× (0.79–0.82) | 0.76× (0.74–0.78) | 2.37× (2.32–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.32× (0.31–0.32) | 1.15× (1.15–1.16) | 0.81× (0.81–0.81) | 5.09× (5.08–5.10) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.50× (0.50–0.51) | 0.26× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.49) | 0.40× (0.40–0.40) | 0.86× (0.86–0.87) | 0.73× (0.70–0.76) | 2.99× (2.98–3.01) | 0.18× (0.18–0.18) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.53× (0.52–0.54) | 0.44× (0.42–0.47) | 0.72× (0.71–0.72) | 0.72× (0.70–0.75) | 2.08× (2.05–2.10) | 0.12× (0.12–0.12) | 0.13× (0.13–0.13) | 0.20× (0.20–0.20) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.08× (1.08–1.08) | 0.77× (0.77–0.77) | 5.41× (5.40–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.76–0.77) | 5.36× (5.34–5.37) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.37× (5.36–5.39) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.44× (1.43–1.44) | 0.63× (0.63–0.63) | 6.47× (6.46–6.49) | 0.39× (0.39–0.39) | 0.41× (0.40–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 2 | 34 MB | 0.46× (0.45–0.48) | 0.44× (0.39–0.49) | 1.13× (1.10–1.16) | 0.69× (0.67–0.70) | 3.53× (3.44–3.63) | 0.21× (0.21–0.22) | 0.22× (0.22–0.23) | 0.35× (0.34–0.35) | 0.17× (0.17–0.18) |
| `text` (1.61 MB → 12.1 MB, 7.5×) | 4 | 55 MB | 0.49× (0.48–0.50) | 0.45× (0.41–0.49) | 0.92× (0.90–0.94) | 0.73× (0.72–0.75) | 2.35× (2.31–2.39) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 506 | 1.98 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 744 | 2.17 | 1.17× (1.07–1.28) | 69% | 1.03× (1.00–1.05) | 1.06× (1.00–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 765 | 4.12 | 2.12× (2.04–2.21) | 38% | 1.24× (1.19–1.30) | 1.76× (1.58–1.96) |

**Best:** mibz2 at 4 threads (765 MB/s); lbzip2 at 4 threads (304 MB/s); pbz2 at 4 threads (514 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.03, lbzip2 1t 0.08, lbzip2 2t 0.30, lbzip2 4t 0.45, pbz2 1t 0.00, pbz2 2t 0.49, pbz2 4t 0.52. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.70 | 0.01 | 0.09 | 0.274 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4329 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.06 | 13.5% |
| MTF + RLE2 | 1.95 | 24.9% |
| T-vector build | 1.39 | 17.8% |
| walk (the one each build uses) | 3.04 | 38.8% |
| of which: segment assembly | 0.11 | 1.4% |
| RLE1 | 0.15 | 1.9% |
| CRC | 0.26 | 3.3% |
| **total** | 7.84 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.315 | 0.702 | (in its total) | 2.22× (2.19–2.25) |  |
| walk | 0.698 | 1.605 | (in its total) | 2.29× (2.25–2.32) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.024 | 0.029 | (in its total) | 1.21× (1.19–1.22) |  |
| inverse BWT, total | 1.040 | 2.336 | 1.310 | 2.24× (2.21–2.27) | 1.25× (1.24–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 8m35s |
| 4 threads | 40m43s ± 6m15s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m51s ± 4m16s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 26m51s ± 4m16s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 3.52 ns a byte of output at 4 threads (CPU, summed over them), 1.72 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,608,759 | 12,123,200 | \`c8a87e6ee952a89d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4328.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
