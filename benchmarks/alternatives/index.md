# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`d6d4cac`](<https://github.com/lmmx/mibz2/commit/d6d4cac5ad49babee5a8f8dab7d163b8217fdc7c>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40927252903>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.408 s | 514 | 8.4 | 2.10× (2.01–2.21) | 5.49× (5.12–5.88) | 5.73× (5.27–6.22) | 6.59× (6.05–7.19) | 5.93× (5.50–6.39) | 6.25× (5.67–6.88) | 5.48× (5.32–5.64) | 1.64× (1.58–1.70) | 1.88× (1.81–1.95) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.309 s | 679 | 10.7 | 1.61× (1.48–1.75) | 3.75× (3.43–4.11) | 4.36× (4.03–4.71) | 4.40× (4.07–4.76) | 4.04× (3.75–4.35) | 8.29× (7.52–9.14) | 7.27× (6.74–7.84) | 2.17× (2.01–2.35) | 2.50× (2.31–2.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.276 s | 759 | 18.5 | 1.51× (1.47–1.55) | 2.64× (2.42–2.88) | 3.37× (2.95–3.86) | 3.00× (2.79–3.22) | 3.04× (2.90–3.19) | 9.33× (8.48–10.27) | 8.19× (7.89–8.49) | 2.45× (2.38–2.52) | 2.81× (2.72–2.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,329 | 3.6 | 0.55× (0.54–0.56) | 3.44× (3.34–3.55) | 4.58× (4.16–5.05) | 3.55× (3.50–3.59) | 6.67× (6.53–6.81) | 2.67× (2.62–2.72) | 2.67× (2.58–2.78) | 1.03× (1.00–1.05) | 2.03× (1.98–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,320 | 3.6 | 0.50× (0.49–0.52) | 2.77× (2.67–2.88) | 5.76× (5.59–5.94) | 3.51× (3.43–3.60) | 6.57× (6.40–6.74) | 2.67× (2.60–2.75) | 2.68× (2.55–2.81) | 1.03× (1.00–1.06) | 2.03× (1.96–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,303 | 3.6 | 0.50× (0.47–0.53) | 2.90× (2.62–3.20) | 5.74× (5.33–6.19) | 3.71× (3.20–4.31) | 6.42× (6.03–6.84) | 2.60× (2.46–2.75) | 2.60× (2.43–2.79) | 1.00× (0.95–1.05) | 1.98× (1.88–2.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.024 s | 817 | 5.6 | 2.28× (2.24–2.33) | 3.00× (2.93–3.07) | 3.51× (3.43–3.60) | 3.65× (3.57–3.72) | 4.91× (4.62–5.23) | 2.22× (2.17–2.27) | 2.02× (1.98–2.06) | 1.72× (1.69–1.75) | 2.57× (2.51–2.63) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,206 | 7.2 | 1.88× (1.74–2.02) | 2.50× (2.35–2.66) | 3.69× (3.38–4.03) | 3.38× (3.21–3.57) | 5.20× (4.94–5.48) | 3.21× (3.05–3.38) | 2.92× (2.75–3.11) | 2.48× (2.32–2.66) | 3.72× (3.49–3.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 970 | 13.9 | 1.35× (1.27–1.43) | 1.54× (1.44–1.65) | 2.21× (2.01–2.43) | 1.84× (1.71–1.98) | 3.75× (3.49–4.04) | 2.55× (2.40–2.72) | 2.32× (2.16–2.49) | 1.97× (1.83–2.13) | 2.95× (2.75–3.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.090 s | 89 | 49.1 | 1.11× (0.98–1.27) | 2.55× (2.20–2.96) | 2.28× (1.94–2.67) | 2.19× (1.88–2.54) | 2.58× (2.20–3.02) | 2.95× (2.52–3.46) | 2.90× (2.45–3.44) | 1.07× (0.93–1.22) | 1.19× (1.02–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.056 s | 143 | 54.9 | 1.18× (1.14–1.23) | 2.77× (2.62–2.93) | 2.53× (2.40–2.68) | 3.28× (2.73–3.94) | 3.07× (2.91–3.24) | 5.31× (5.11–5.52) | 5.21× (4.96–5.48) | 1.92× (1.87–1.97) | 2.14× (2.04–2.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.046 s | 173 | 75.7 | 1.14× (1.07–1.22) | 2.24× (2.11–2.38) | 2.42× (2.00–2.91) | 2.65× (2.39–2.95) | 3.05× (2.88–3.23) | 6.36× (5.93–6.83) | 6.25× (5.75–6.79) | 2.30× (2.15–2.45) | 2.57× (2.39–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 53.0 | 1.14× (1.03–1.27) | 2.91× (2.77–3.06) | 2.34× (2.22–2.46) | 2.33× (2.26–2.40) | 2.65× (2.57–2.74) | 3.45× (3.37–3.52) | 3.55× (3.47–3.63) | 1.17× (1.14–1.20) | 1.37× (1.28–1.47) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 137 | 59.9 | 0.95× (0.81–1.12) | 2.43× (2.08–2.85) | 2.11× (1.83–2.43) | 2.86× (2.48–3.31) | 2.61× (2.38–2.87) | 5.29× (4.33–6.46) | 5.45× (4.51–6.57) | 1.80× (1.47–2.21) | 2.10× (1.70–2.60) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 175 | 78.2 | 1.26× (1.09–1.45) | 2.39× (2.02–2.81) | 2.13× (2.06–2.19) | 2.83× (2.71–2.95) | 2.74× (2.63–2.86) | 7.24× (7.02–7.47) | 7.45× (7.25–7.66) | 2.47× (2.39–2.55) | 2.87× (2.71–3.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 962 | 5.1 | 0.57× (0.56–0.59) | 2.93× (2.83–3.04) | 4.36× (4.04–4.69) | 2.94× (2.82–3.07) | 7.05× (6.82–7.28) | 2.21× (2.12–2.30) | 2.17× (2.09–2.26) | 0.96× (0.94–0.98) | 1.96× (1.89–2.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 5.1 | 0.50× (0.49–0.51) | 3.01× (2.94–3.08) | 5.21× (5.08–5.34) | 2.98× (2.90–3.07) | 7.18× (6.98–7.40) | 2.26× (2.18–2.35) | 2.23× (2.16–2.30) | 0.98× (0.95–1.01) | 2.01× (1.97–2.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 926 | 5.3 | 0.48× (0.47–0.50) | 2.94× (2.77–3.13) | 5.05× (4.87–5.24) | 2.91× (2.70–3.14) | 6.80× (6.57–7.04) | 2.15× (2.08–2.22) | 2.11× (2.06–2.17) | 0.93× (0.90–0.97) | 1.91× (1.85–1.96) | yes |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 1 | 0.052 s | 235 | 18.9 | 1.34× (1.23–1.47) | 3.18× (3.05–3.30) | 3.27× (3.12–3.42) | 3.38× (3.24–3.52) | 3.73× (3.52–3.95) | 3.46× (3.26–3.67) | 3.29× (3.15–3.43) | 1.11× (1.09–1.14) | 1.39× (1.35–1.43) | yes |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 2 | 0.036 s | 342 | 23.0 | 1.04× (0.96–1.12) | 2.54× (2.32–2.78) | 2.92× (2.69–3.16) | 3.38× (2.97–3.85) | 3.90× (3.51–4.34) | 4.94× (4.48–5.45) | 4.70× (4.31–5.12) | 1.59× (1.48–1.72) | 1.99× (1.84–2.15) | yes |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 4 | 0.032 s | 389 | 36.2 | 1.04× (0.98–1.11) | 2.09× (1.81–2.41) | 2.84× (2.38–3.40) | 3.08× (2.70–3.51) | 3.72× (3.30–4.20) | 5.62× (5.13–6.15) | 5.34× (4.98–5.73) | 1.81× (1.71–1.92) | 2.26× (2.13–2.40) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.10× (2.00–2.20) | 5.46× (5.09–5.85) | 5.70× (5.25–6.19) | 6.57× (6.03–7.17) | 5.91× (5.48–6.37) | 6.23× (5.65–6.85) | 5.46× (5.30–5.62) | 1.63× (1.57–1.70) | 1.87× (1.80–1.94) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.71× (1.59–1.85) | 4.38× (4.03–4.76) | 4.49× (4.15–4.85) | 5.22× (4.81–5.67) | 4.52× (4.18–4.87) | 4.97× (4.53–5.46) | 4.36× (4.04–4.71) | 1.31× (1.20–1.42) | 1.49× (1.37–1.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.59× (1.54–1.63) | 3.18× (2.91–3.47) | 3.47× (2.98–4.04) | 3.62× (3.37–3.88) | 3.15× (3.03–3.28) | 2.88× (2.62–3.16) | 2.52× (2.43–2.62) | 0.76× (0.73–0.78) | 0.86× (0.83–0.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.97× (2.90–3.03) | 4.00× (3.71–4.30) | 3.31× (3.27–3.35) | 6.14× (6.03–6.24) | 2.51× (2.47–2.56) | 2.52× (2.43–2.61) | 1.03× (1.00–1.05) | 1.76× (1.72–1.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.54–0.58) | 2.74× (2.66–2.83) | 4.98× (4.83–5.14) | 3.24× (3.17–3.32) | 5.95× (5.81–6.10) | 2.47× (2.40–2.54) | 2.47× (2.36–2.59) | 1.01× (0.98–1.04) | 1.73× (1.67–1.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.52–0.58) | 3.02× (2.84–3.21) | 5.06× (4.77–5.37) | 3.53× (3.05–4.07) | 5.97× (5.64–6.32) | 2.47× (2.35–2.60) | 2.48× (2.32–2.64) | 1.01× (0.97–1.05) | 1.74× (1.65–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.20× (2.16–2.24) | 2.58× (2.54–2.63) | 3.09× (3.03–3.16) | 3.47× (3.40–3.54) | 4.66× (4.38–4.96) | 2.14× (2.09–2.19) | 1.95× (1.91–1.99) | 1.67× (1.64–1.70) | 2.25× (2.20–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.85× (1.74–1.97) | 2.20× (2.07–2.33) | 2.66× (2.53–2.80) | 2.98× (2.81–3.17) | 3.48× (3.30–3.66) | 1.64× (1.56–1.73) | 1.50× (1.41–1.59) | 1.28× (1.20–1.37) | 1.73× (1.64–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.55× (1.46–1.64) | 1.48× (1.37–1.59) | 1.72× (1.61–1.84) | 1.74× (1.62–1.88) | 1.97× (1.82–2.12) | 0.82× (0.76–0.89) | 0.75× (0.69–0.82) | 0.64× (0.59–0.70) | 0.87× (0.79–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.11× (0.98–1.26) | 2.47× (2.14–2.86) | 2.22× (1.89–2.59) | 2.17× (1.87–2.52) | 2.55× (2.18–2.99) | 2.93× (2.50–3.42) | 2.87× (2.43–3.40) | 1.07× (0.94–1.21) | 1.13× (0.98–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.17× (1.13–1.21) | 2.67× (2.49–2.86) | 2.31× (2.19–2.43) | 3.11× (2.61–3.70) | 2.65× (2.51–2.79) | 2.89× (2.78–3.00) | 2.84× (2.68–3.00) | 1.05× (1.03–1.08) | 1.12× (1.09–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.25× (1.20–1.31) | 2.32× (2.21–2.44) | 2.10× (1.73–2.54) | 2.57× (2.36–2.80) | 2.46× (2.33–2.60) | 2.12× (2.02–2.22) | 2.08× (1.95–2.22) | 0.77× (0.73–0.81) | 0.82× (0.78–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.14× (1.03–1.27) | 2.86× (2.73–2.99) | 2.27× (2.16–2.39) | 2.31× (2.25–2.38) | 2.63× (2.55–2.72) | 3.41× (3.34–3.49) | 3.51× (3.44–3.59) | 1.17× (1.14–1.20) | 1.32× (1.23–1.41) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.96× (0.83–1.10) | 2.41× (2.14–2.72) | 1.92× (1.70–2.16) | 2.68× (2.36–3.03) | 2.31× (2.17–2.45) | 2.81× (2.35–3.36) | 2.89× (2.45–3.41) | 0.96× (0.80–1.16) | 1.08× (0.90–1.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.37× (1.19–1.57) | 2.56× (2.19–3.00) | 1.77× (1.71–1.83) | 2.56× (2.47–2.66) | 2.29× (2.19–2.39) | 2.28× (2.19–2.38) | 2.35× (2.28–2.42) | 0.78× (0.75–0.82) | 0.88× (0.82–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.63× (0.61–0.64) | 2.38× (2.31–2.46) | 3.42× (3.17–3.70) | 2.70× (2.58–2.83) | 6.29× (6.09–6.49) | 2.06× (1.98–2.14) | 2.03× (1.95–2.11) | 0.97× (0.95–0.98) | 1.41× (1.38–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.31× (2.23–2.39) | 4.09× (3.97–4.22) | 2.73× (2.66–2.81) | 6.26× (6.10–6.41) | 2.05× (1.98–2.12) | 2.02× (1.97–2.08) | 0.96× (0.94–0.99) | 1.41× (1.37–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.52–0.56) | 2.50× (2.35–2.66) | 4.00× (3.82–4.17) | 2.77× (2.56–2.99) | 6.14× (5.95–6.34) | 2.02× (1.96–2.08) | 1.99× (1.94–2.04) | 0.95× (0.91–0.99) | 1.39× (1.34–1.43) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 1 | 1.33× (1.22–1.45) | 3.03× (2.91–3.15) | 3.12× (2.97–3.28) | 3.31× (3.17–3.45) | 3.65× (3.45–3.86) | 3.38× (3.19–3.59) | 3.22× (3.09–3.36) | 1.11× (1.08–1.14) | 1.27× (1.23–1.31) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 2 | 1.10× (1.02–1.18) | 2.43× (2.21–2.68) | 2.70× (2.53–2.89) | 3.16× (2.81–3.54) | 3.36× (2.99–3.77) | 2.69× (2.44–2.95) | 2.56× (2.37–2.77) | 0.88× (0.82–0.94) | 1.01× (0.93–1.09) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 4 | 1.17× (1.11–1.24) | 2.18× (1.90–2.50) | 2.55× (2.12–3.07) | 2.78× (2.38–3.23) | 2.82× (2.35–3.39) | 1.72× (1.56–1.90) | 1.64× (1.51–1.78) | 0.57× (0.53–0.61) | 0.64× (0.60–0.69) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.36× (0.35–0.37) | 1.16× (1.16–1.17) | 0.84× (0.83–0.86) | 11.83× (11.82–11.84) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.54× (0.47–0.62) | 0.85× (0.83–0.87) | 0.67× (0.65–0.69) | 6.30× (6.18–6.43) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 75 MB | 0.41× (0.40–0.41) | 0.70× (0.61–0.80) | 0.77× (0.76–0.79) | 0.63× (0.62–0.64) | 4.25× (4.23–4.28) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.27–4.28) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.15–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.43× (0.43–0.43) | 1.63× (1.62–1.63) | 0.64× (0.63–0.64) | 7.74× (7.73–7.76) | 0.62× (0.62–0.62) | 0.65× (0.65–0.66) | 0.71× (0.70–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.64× (0.62–0.66) | 0.63× (0.61–0.66) | 1.53× (1.52–1.54) | 0.87× (0.86–0.88) | 5.52× (5.51–5.54) | 0.44× (0.44–0.44) | 0.47× (0.46–0.47) | 0.50× (0.50–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.59× (0.58–0.60) | 0.63× (0.60–0.66) | 1.09× (1.07–1.11) | 0.84× (0.83–0.86) | 3.16× (3.11–3.21) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.27–0.28) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.14–1.15) | 0.71× (0.71–0.72) | 5.07× (5.06–5.08) | 0.32× (0.31–0.32) | 0.33× (0.33–0.33) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.43× (0.40–0.45) | 0.93× (0.92–0.94) | 0.75× (0.73–0.76) | 3.24× (3.21–3.27) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.58–0.60) | 0.43× (0.42–0.44) | 0.81× (0.80–0.83) | 0.76× (0.74–0.78) | 2.39× (2.34–2.43) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.81× (0.80–0.81) | 5.08× (5.07–5.09) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.50× (0.50–0.51) | 0.25× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.48–0.49) | 0.40× (0.37–0.42) | 0.87× (0.86–0.88) | 0.74× (0.72–0.77) | 3.01× (2.99–3.03) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.54× (0.53–0.56) | 0.44× (0.40–0.47) | 0.73× (0.71–0.74) | 0.71× (0.69–0.73) | 2.11× (2.07–2.15) | 0.12× (0.12–0.13) | 0.13× (0.13–0.13) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.41× (5.40–5.41) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.76–0.77) | 5.36× (5.35–5.37) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.62× (1.62–1.62) | 0.77× (0.77–0.78) | 5.38× (5.37–5.39) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.44× (1.43–1.44) | 0.63× (0.62–0.64) | 6.50× (6.48–6.51) | 0.39× (0.39–0.39) | 0.41× (0.41–0.41) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 2 | 32 MB | 0.52× (0.51–0.52) | 0.43× (0.39–0.47) | 1.23× (1.21–1.24) | 0.73× (0.71–0.76) | 3.84× (3.80–3.87) | 0.23× (0.23–0.23) | 0.24× (0.24–0.25) | 0.38× (0.37–0.38) | 0.19× (0.18–0.19) |
| `text` (1.64 MB → 12.3 MB, 7.5×) | 4 | 51 MB | 0.53× (0.52–0.54) | 0.52× (0.48–0.57) | 0.99× (0.97–1.02) | 0.79× (0.77–0.80) | 2.53× (2.48–2.58) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 514 | 1.95 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 679 | 2.48 | 1.25× (1.16–1.35) | 66% | 1.00× (0.97–1.04) | 1.02× (1.00–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 759 | 4.31 | 2.16× (2.07–2.26) | 37% | 1.26× (1.20–1.32) | 1.63× (1.58–1.69) |

**Best:** mibz2 at 4 threads (759 MB/s); lbzip2 at 4 threads (304 MB/s); pbz2 at 4 threads (510 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.03, lbzip2 1t 0.08, lbzip2 2t 0.26, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.75 | 0.01 | 0.09 | 0.276 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4299 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 12.9% |
| MTF + RLE2 | 1.84 | 23.4% |
| T-vector build | 1.37 | 17.5% |
| walk (the one each build uses) | 3.24 | 41.3% |
| of which: segment assembly | 0.11 | 1.4% |
| RLE1 | 0.15 | 1.9% |
| CRC | 0.25 | 3.2% |
| **total** | 7.84 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.348 | 0.715 | (in its total) | 2.07× (2.01–2.12) |  |
| walk | 0.767 | 1.623 | (in its total) | 2.08× (1.99–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.030 | (in its total) | 1.15× (1.09–1.20) |  |
| inverse BWT, total | 1.139 | 2.367 | 1.384 | 2.05× (1.98–2.13) | 1.19× (1.16–1.23) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 7m33s |
| 4 threads | 40m24s ± 5m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m02s ± 4m04s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 27m02s ± 4m04s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 3.54 ns a byte of output at 4 threads (CPU, summed over them), 1.67 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,641,171 | 12,347,817 | \`8d8fd1d325820edb…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4299.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
