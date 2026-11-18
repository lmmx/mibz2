# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-573/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`70a980e`](<https://github.com/lmmx/mibz2/commit/70a980e52628b4c61b2ef4679a3db1d1b95a713d>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44303452673>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.316 s | 663 | 3.9 | 2.67× (2.64–2.70) | 6.70× (6.62–6.77) | 7.05× (6.96–7.15) | 8.02× (7.92–8.12) | 7.10× (7.00–7.20) | 7.39× (7.30–7.49) | 7.08× (6.98–7.18) | 2.11× (2.07–2.15) | 2.44× (2.40–2.49) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.191 s | 1,098 | 4.4 | 2.56× (2.36–2.78) | 5.73× (5.32–6.17) | 6.78× (6.35–7.25) | 6.93× (6.49–7.40) | 6.21× (5.78–6.68) | 11.81× (11.04–12.63) | 11.31× (10.58–12.10) | 3.37× (3.14–3.61) | 3.90× (3.65–4.17) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.156 s | 1,347 | 6.6 | 2.79× (2.71–2.87) | 4.26× (4.14–4.38) | 5.58× (5.36–5.82) | 5.21× (4.99–5.44) | 5.15× (4.99–5.32) | 14.83× (14.27–15.41) | 14.20× (13.69–14.74) | 4.23× (4.05–4.40) | 4.90× (4.69–5.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,394 | 2.1 | 0.57× (0.55–0.58) | 3.59× (3.52–3.65) | 4.42× (4.34–4.51) | 3.74× (3.68–3.81) | 6.55× (6.39–6.71) | 2.97× (2.68–3.28) | 2.81× (2.76–2.87) | 1.10× (0.90–1.33) | 2.16× (2.12–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,365 | 2.1 | 0.52× (0.50–0.53) | 3.02× (2.78–3.28) | 5.74× (5.54–5.96) | 3.65× (3.55–3.75) | 6.26× (6.04–6.49) | 2.90× (2.60–3.22) | 2.75× (2.66–2.84) | 1.07× (0.88–1.30) | 2.11× (2.04–2.18) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,399 | 2.1 | 0.52× (0.51–0.53) | 3.02× (2.84–3.21) | 5.99× (5.46–6.56) | 3.69× (3.59–3.80) | 6.44× (6.30–6.59) | 2.90× (2.64–3.18) | 2.75× (2.68–2.81) | 1.07× (0.88–1.31) | 2.11× (2.05–2.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,037 | 2.7 | 2.84× (2.78–2.91) | 3.69× (3.53–3.86) | 4.26× (4.12–4.41) | 4.59× (4.47–4.71) | 5.28× (5.16–5.41) | 3.04× (2.94–3.15) | 2.60× (2.52–2.68) | 2.13× (2.11–2.16) | 3.35× (3.14–3.57) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,446 | 3.5 | 2.23× (1.95–2.56) | 3.08× (2.72–3.48) | 4.19× (3.73–4.72) | 3.70× (3.36–4.06) | 5.57× (5.08–6.11) | 4.11× (3.75–4.50) | 3.51× (3.26–3.77) | 2.88× (2.65–3.14) | 4.52× (4.01–5.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,150 | 7.0 | 1.64× (1.59–1.70) | 1.83× (1.74–1.92) | 2.62× (2.49–2.75) | 2.00× (1.87–2.14) | 4.01× (3.78–4.25) | 3.29× (3.06–3.54) | 2.81× (2.64–2.99) | 2.31× (2.18–2.44) | 3.62× (3.33–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.080 s | 100 | 26.4 | 1.36× (1.25–1.49) | 3.02× (2.76–3.30) | 2.79× (2.54–3.05) | 2.76× (2.48–3.06) | 3.04× (2.77–3.33) | 3.44× (3.15–3.76) | 3.48× (3.18–3.80) | 1.34× (1.23–1.46) | 1.56× (1.42–1.72) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.055 s | 147 | 31.2 | 1.25× (1.18–1.32) | 2.81× (2.63–3.01) | 2.53× (2.46–2.61) | 3.18× (3.03–3.34) | 2.97× (2.86–3.10) | 5.29× (5.09–5.51) | 5.35× (5.14–5.57) | 2.06× (1.99–2.14) | 2.40× (2.29–2.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 189 | 42.2 | 1.27× (1.16–1.39) | 2.39× (2.19–2.61) | 2.38× (2.18–2.59) | 2.82× (2.58–3.09) | 3.04× (2.77–3.32) | 6.51× (6.03–7.03) | 6.58× (6.02–7.18) | 2.54× (2.34–2.75) | 2.95× (2.69–3.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.085 s | 112 | 23.5 | 1.48× (1.42–1.54) | 3.67× (3.51–3.83) | 3.05× (2.93–3.18) | 3.02× (2.89–3.16) | 3.50× (3.28–3.74) | 4.46× (4.28–4.65) | 4.81× (4.61–5.03) | 1.60× (1.52–1.68) | 1.82× (1.73–1.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 27.5 | 1.39× (1.26–1.54) | 3.21× (3.05–3.37) | 2.73× (2.63–2.83) | 3.84× (3.59–4.12) | 3.35× (3.17–3.53) | 6.94× (6.76–7.13) | 7.50× (7.28–7.72) | 2.49× (2.43–2.56) | 2.84× (2.75–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 206 | 37.6 | 1.36× (1.26–1.47) | 2.55× (2.36–2.76) | 2.44× (2.29–2.61) | 3.40× (3.27–3.54) | 3.09× (2.88–3.32) | 8.10× (7.57–8.66) | 8.75× (8.22–9.31) | 2.91× (2.73–3.10) | 3.31× (3.07–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,036 | 2.9 | 0.61× (0.60–0.62) | 3.16× (3.07–3.25) | 4.37× (4.25–4.49) | 3.09× (3.00–3.18) | 6.96× (6.79–7.13) | 2.31× (2.23–2.39) | 2.37× (2.28–2.48) | 0.96× (0.93–0.99) | 2.12× (2.06–2.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,026 | 3.0 | 0.52× (0.50–0.53) | 3.21× (2.95–3.49) | 5.37× (5.21–5.53) | 3.10× (2.97–3.23) | 6.78× (6.57–6.99) | 2.28× (2.19–2.37) | 2.34× (2.28–2.41) | 0.95× (0.91–0.98) | 2.09× (2.02–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 966 | 3.1 | 0.50× (0.49–0.51) | 2.99× (2.94–3.05) | 5.15× (5.05–5.24) | 3.06× (2.87–3.26) | 6.79× (6.26–7.37) | 2.18× (2.12–2.24) | 2.24× (2.17–2.32) | 0.91× (0.89–0.93) | 2.00× (1.95–2.05) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 0.043 s | 298 | 9.0 | 1.72× (1.68–1.76) | 4.22× (4.14–4.30) | 4.61× (4.45–4.77) | 4.69× (4.55–4.84) | 4.96× (4.86–5.06) | 5.11× (4.94–5.28) | 4.86× (4.79–4.94) | 1.61× (1.43–1.81) | 1.85× (1.80–1.90) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 0.032 s | 399 | 11.9 | 1.42× (1.25–1.62) | 3.17× (2.88–3.49) | 3.85× (3.57–4.15) | 4.48× (4.06–4.95) | 4.11× (3.75–4.51) | 6.68× (6.07–7.36) | 6.36× (5.80–6.97) | 2.10× (1.91–2.31) | 2.42× (2.23–2.62) | yes |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 0.029 s | 449 | 17.8 | 1.30× (1.24–1.35) | 2.45× (2.35–2.55) | 3.16× (2.91–3.44) | 3.39× (3.23–3.54) | 4.02× (3.69–4.38) | 7.67× (7.41–7.93) | 7.30× (7.04–7.57) | 2.41× (2.14–2.72) | 2.78× (2.64–2.92) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.66× (2.64–2.68) | 6.65× (6.57–6.73) | 7.01× (6.92–7.10) | 7.98× (7.89–8.08) | 7.07× (6.97–7.17) | 7.36× (7.26–7.45) | 7.05× (6.95–7.15) | 2.10× (2.06–2.14) | 2.42× (2.37–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.41× (2.28–2.54) | 6.03× (5.70–6.38) | 6.36× (5.99–6.75) | 7.41× (6.95–7.90) | 6.29× (5.89–6.71) | 6.41× (6.04–6.81) | 6.14× (5.81–6.50) | 1.83× (1.73–1.94) | 2.11× (1.99–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.73× (2.64–2.82) | 4.85× (4.71–5.00) | 5.40× (5.20–5.60) | 5.97× (5.71–6.24) | 5.12× (4.93–5.31) | 4.35× (4.19–4.52) | 4.17× (4.03–4.32) | 1.24× (1.19–1.30) | 1.43× (1.37–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.57–0.64) | 3.06× (2.91–3.21) | 3.85× (3.62–4.08) | 3.40× (3.25–3.56) | 5.88× (5.58–6.19) | 2.74× (2.40–3.11) | 2.58× (2.44–2.73) | 1.08× (0.88–1.33) | 1.81× (1.71–1.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.59) | 2.86× (2.76–2.97) | 4.95× (4.81–5.10) | 3.35× (3.27–3.43) | 5.64× (5.45–5.83) | 2.68× (2.39–3.00) | 2.53× (2.46–2.60) | 1.06× (0.86–1.30) | 1.77× (1.72–1.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.58) | 3.10× (2.95–3.25) | 5.22× (4.75–5.72) | 3.43× (3.31–3.56) | 5.85× (5.72–5.97) | 2.71× (2.43–3.03) | 2.56× (2.50–2.62) | 1.07× (0.87–1.32) | 1.79× (1.75–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.63–2.75) | 3.16× (3.09–3.23) | 3.70× (3.63–3.78) | 4.29× (4.19–4.40) | 4.93× (4.81–5.05) | 2.87× (2.78–2.97) | 2.47× (2.38–2.57) | 2.04× (2.01–2.06) | 2.83× (2.66–3.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.08–2.50) | 2.69× (2.41–3.01) | 3.14× (2.99–3.30) | 3.41× (3.25–3.58) | 3.90× (3.71–4.10) | 2.17× (2.04–2.30) | 1.87× (1.78–1.96) | 1.54× (1.47–1.61) | 2.14× (1.96–2.33) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.90× (1.82–1.98) | 1.70× (1.63–1.77) | 2.03× (1.94–2.12) | 2.00× (1.90–2.11) | 2.22× (2.13–2.33) | 1.09× (1.02–1.17) | 0.94× (0.88–1.01) | 0.78× (0.74–0.81) | 1.08× (0.97–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.23–1.48) | 2.93× (2.67–3.21) | 2.69× (2.45–2.94) | 2.72× (2.44–3.03) | 2.99× (2.72–3.29) | 3.39× (3.09–3.71) | 3.42× (3.12–3.76) | 1.33× (1.22–1.46) | 1.48× (1.33–1.65) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.20–1.30) | 2.76× (2.67–2.86) | 2.37× (2.31–2.43) | 3.12× (3.04–3.20) | 2.67× (2.60–2.75) | 2.98× (2.89–3.06) | 3.01× (2.92–3.10) | 1.17× (1.14–1.20) | 1.31× (1.24–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.38× (1.25–1.51) | 2.47× (2.27–2.68) | 2.03× (1.86–2.21) | 2.77× (2.50–3.06) | 2.52× (2.27–2.80) | 2.16× (1.97–2.35) | 2.18× (1.97–2.41) | 0.85× (0.77–0.93) | 0.95× (0.86–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.47× (1.41–1.53) | 3.54× (3.37–3.72) | 2.94× (2.82–3.08) | 2.98× (2.84–3.13) | 3.45× (3.22–3.69) | 4.38× (4.18–4.59) | 4.74× (4.51–4.97) | 1.58× (1.50–1.67) | 1.74× (1.65–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.31–1.38) | 3.24× (3.14–3.35) | 2.58× (2.48–2.68) | 3.71× (3.44–4.00) | 3.06× (2.90–3.24) | 3.81× (3.68–3.94) | 4.12× (3.96–4.28) | 1.38× (1.33–1.42) | 1.51× (1.46–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.42–1.65) | 2.86× (2.66–3.08) | 2.17× (2.00–2.34) | 3.18× (3.04–3.33) | 2.78× (2.58–3.00) | 2.69× (2.51–2.90) | 2.91× (2.73–3.11) | 0.97× (0.91–1.04) | 1.07× (1.00–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.55× (2.44–2.67) | 3.33× (3.24–3.43) | 2.81× (2.73–2.89) | 6.15× (6.00–6.30) | 2.14× (2.06–2.22) | 2.21× (2.10–2.32) | 0.97× (0.94–1.00) | 1.54× (1.41–1.69) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.62) | 2.45× (2.19–2.75) | 4.06× (3.92–4.20) | 2.81× (2.69–2.93) | 5.85× (5.69–6.03) | 2.05× (1.98–2.13) | 2.12× (2.03–2.21) | 0.93× (0.90–0.96) | 1.48× (1.35–1.63) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.50× (2.43–2.57) | 3.99× (3.81–4.18) | 2.88× (2.71–3.05) | 6.09× (5.62–6.60) | 2.04× (1.99–2.10) | 2.11× (2.02–2.21) | 0.93× (0.90–0.95) | 1.48× (1.35–1.62) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 1.69× (1.65–1.73) | 4.00× (3.95–4.05) | 4.36× (4.19–4.53) | 4.56× (4.43–4.70) | 4.82× (4.73–4.91) | 4.96× (4.80–5.13) | 4.73× (4.65–4.80) | 1.59× (1.41–1.80) | 1.65× (1.62–1.68) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 1.49× (1.27–1.75) | 3.18× (2.99–3.39) | 3.48× (3.28–3.69) | 4.09× (3.80–4.41) | 3.66× (3.43–3.91) | 3.73× (3.45–4.03) | 3.55× (3.30–3.82) | 1.20× (1.11–1.30) | 1.24× (1.16–1.33) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 1.51× (1.47–1.55) | 2.57× (2.48–2.66) | 2.92× (2.80–3.04) | 3.24× (3.15–3.34) | 3.25× (2.90–3.65) | 2.50× (2.42–2.59) | 2.39× (2.33–2.44) | 0.80× (0.71–0.92) | 0.83× (0.82–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.77–0.80) | 10.99× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.48× (0.41–0.57) | 0.67× (0.64–0.69) | 0.57× (0.55–0.59) | 5.37× (5.20–5.55) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 88 MB | 0.34× (0.33–0.36) | 0.53× (0.45–0.62) | 0.61× (0.59–0.63) | 0.53× (0.51–0.56) | 3.55× (3.43–3.68) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.17) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.67× (0.66–0.67) | 8.04× (8.03–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.66–0.73) | 0.69× (0.65–0.72) | 1.62× (1.57–1.66) | 0.81× (0.79–0.84) | 5.86× (5.70–6.03) | 0.47× (0.46–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.57–0.63) | 0.69× (0.64–0.74) | 1.09× (1.03–1.15) | 0.66× (0.63–0.69) | 3.17× (3.02–3.33) | 0.25× (0.24–0.26) | 0.26× (0.25–0.27) | 0.28× (0.27–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.17× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.48–0.53) | 0.42× (0.40–0.44) | 0.90× (0.86–0.95) | 0.70× (0.66–0.74) | 3.13× (2.99–3.29) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.66) | 0.45× (0.44–0.46) | 0.86× (0.83–0.88) | 0.80× (0.77–0.83) | 2.50× (2.43–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.46–0.51) | 0.39× (0.38–0.41) | 0.85× (0.81–0.89) | 0.73× (0.69–0.76) | 2.93× (2.80–3.07) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.48× (0.45–0.50) | 0.80× (0.79–0.82) | 0.83× (0.80–0.86) | 2.32× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.76× (0.76–0.77) | 5.28× (5.25–5.30) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.43) | 0.67× (0.66–0.67) | 6.45× (6.43–6.46) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 2 | 36 MB | 0.46× (0.45–0.47) | 0.41× (0.38–0.44) | 1.08× (1.05–1.12) | 0.69× (0.67–0.71) | 3.37× (3.29–3.45) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.33–0.34) | 0.16× (0.16–0.17) |
| `text` (2.53 MB → 12.9 MB, 5.1×) | 4 | 56 MB | 0.49× (0.49–0.50) | 0.45× (0.41–0.49) | 0.90× (0.89–0.92) | 0.74× (0.73–0.75) | 2.31× (2.28–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 663 | 1.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,098 | 1.71 | 1.15× (1.08–1.22) | 80% | 1.04× (1.02–1.06) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,347 | 2.54 | 1.69× (1.63–1.76) | 50% | 1.23× (1.21–1.26) | 1.73× (1.63–1.85) |

**Best:** mibz2 at 4 threads (1,347 MB/s); lbzip2 at 4 threads (313 MB/s); pbz2 at 4 threads (478 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.23, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.40 | 0.01 | 0.07 | 0.156 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.62 | 14.3% |
| MTF + RLE2 | 0.88 | 20.2% |
| T-vector build | 0.79 | 18.3% |
| walk (the one each build uses) | 1.79 | 41.4% |
| of which: segment assembly | 0.08 | 1.7% |
| RLE1 | 0.09 | 2.1% |
| CRC | 0.15 | 3.6% |
| **total** | 4.33 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.302 | 0.717 | (in its total) | 2.32× (2.21–2.43) |  |
| walk | 0.647 | 1.591 | (in its total) | 2.38× (2.25–2.52) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.032 | 0.030 | (in its total) | 0.91× (0.83–0.99) |  |
| inverse BWT, total | 0.980 | 2.340 | 1.320 | 2.31× (2.19–2.44) | 1.31× (1.25–1.38) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m59s ± 5m17s |
| 4 threads | 22m57s ± 3m20s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m29s ± 2m08s |
| the scanner alone (its serial part) | 8m26s ± 26.3s |
| the workers alone, over 4 | 14m29s ± 2m08s |

At 4 threads the workers bound it. Per byte: the scanner 4.9 ns a byte of bz2, the workers 1.90 ns a byte of output at 4 threads (CPU, summed over them), 1.23 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,527,905 | 12,873,504 | \`d928dbd828b8a8a4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
