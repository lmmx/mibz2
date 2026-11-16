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

The latest record: [`311078e`](<https://github.com/lmmx/mibz2/commit/311078e929765bd573ae9ca2e1f576e9f2165e3b>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43950399710>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.524 s | 400 | 8.2 | 2.89× (2.87–2.91) | 6.64× (6.60–6.68) | 6.82× (6.74–6.89) | 7.41× (7.35–7.48) | 6.67× (6.63–6.71) | 7.24× (7.22–7.26) | 6.78× (6.76–6.80) | 2.15× (2.13–2.16) | 2.50× (2.49–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.311 s | 674 | 8.9 | 2.80× (2.71–2.90) | 6.24× (6.01–6.47) | 6.85× (6.61–7.10) | 6.73× (6.53–6.94) | 6.13× (5.96–6.31) | 12.34× (11.97–12.72) | 11.56× (11.21–11.92) | 3.66× (3.53–3.78) | 4.26× (4.12–4.41) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.245 s | 856 | 13.8 | 2.93× (2.86–3.01) | 4.45× (4.35–4.55) | 5.51× (5.37–5.66) | 5.06× (4.95–5.17) | 5.05× (4.97–5.13) | 15.45× (15.28–15.63) | 14.47× (14.30–14.65) | 4.58× (4.51–4.66) | 5.34× (5.28–5.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,002 | 3.6 | 0.62× (0.60–0.65) | 3.79× (3.58–4.00) | 4.53× (4.37–4.69) | 3.62× (3.56–3.68) | 6.21× (6.12–6.30) | 3.02× (2.97–3.07) | 2.70× (2.65–2.75) | 1.01× (0.99–1.03) | 1.53× (1.50–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 957 | 3.8 | 0.57× (0.57–0.58) | 2.95× (2.89–3.00) | 6.03× (5.89–6.17) | 3.52× (3.46–3.58) | 6.00× (5.92–6.08) | 2.91× (2.85–2.96) | 2.60× (2.56–2.64) | 0.98× (0.96–1.00) | 1.47× (1.44–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 966 | 3.7 | 0.58× (0.57–0.59) | 2.98× (2.92–3.03) | 5.94× (5.77–6.11) | 3.52× (3.46–3.58) | 6.06× (5.92–6.20) | 2.93× (2.87–2.98) | 2.62× (2.57–2.66) | 0.98× (0.96–1.00) | 1.48× (1.45–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 580 | 5.9 | 2.61× (2.59–2.64) | 3.03× (3.00–3.06) | 3.92× (3.84–4.00) | 3.81× (3.78–3.84) | 4.17× (4.13–4.21) | 2.56× (2.54–2.59) | 2.12× (2.09–2.14) | 2.02× (2.00–2.04) | 3.02× (2.99–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 873 | 7.4 | 2.14× (1.97–2.32) | 2.74× (2.54–2.97) | 3.94× (3.68–4.22) | 3.30× (3.05–3.57) | 4.67× (4.36–5.00) | 3.72× (3.47–3.99) | 3.07× (2.86–3.29) | 2.93× (2.73–3.14) | 4.38× (4.09–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 752 | 15.0 | 1.80× (1.75–1.86) | 1.97× (1.84–2.10) | 2.90× (2.74–3.07) | 2.13× (2.08–2.17) | 3.81× (3.75–3.88) | 3.32× (3.24–3.41) | 2.74× (2.68–2.80) | 2.62× (2.57–2.67) | 3.91× (3.83–4.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 59 | 55.5 | 1.31× (1.30–1.32) | 2.65× (2.63–2.68) | 2.36× (2.34–2.39) | 2.42× (2.41–2.44) | 2.80× (2.78–2.82) | 3.06× (3.03–3.08) | 3.14× (3.12–3.17) | 1.20× (1.19–1.21) | 1.37× (1.35–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 63.1 | 1.25× (1.23–1.28) | 2.49× (2.44–2.55) | 2.33× (2.29–2.36) | 2.95× (2.90–3.01) | 2.86× (2.81–2.91) | 4.94× (4.87–5.02) | 5.08× (5.00–5.16) | 1.94× (1.91–1.98) | 2.21× (2.16–2.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 111 | 85.1 | 1.27× (1.24–1.31) | 2.07× (2.03–2.12) | 2.07× (1.95–2.19) | 2.52× (2.43–2.61) | 2.74× (2.68–2.80) | 5.78× (5.67–5.90) | 5.94× (5.81–6.08) | 2.27× (2.23–2.32) | 2.58× (2.53–2.64) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.149 s | 64 | 51.0 | 1.42× (1.40–1.43) | 2.96× (2.92–3.01) | 2.50× (2.46–2.55) | 2.57× (2.54–2.60) | 3.06× (2.97–3.16) | 3.55× (3.51–3.59) | 3.87× (3.83–3.91) | 1.38× (1.35–1.41) | 1.67× (1.66–1.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 57.5 | 1.30× (1.23–1.37) | 2.71× (2.62–2.80) | 2.39× (2.27–2.51) | 3.36× (3.18–3.56) | 2.98× (2.85–3.12) | 5.73× (5.48–5.99) | 6.25× (5.99–6.53) | 2.23× (2.13–2.33) | 2.71× (2.62–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 81.9 | 1.37× (1.33–1.41) | 2.21× (2.13–2.29) | 2.15× (2.11–2.20) | 2.88× (2.78–2.97) | 2.70× (2.63–2.78) | 6.67× (6.48–6.86) | 7.27× (7.06–7.50) | 2.59× (2.54–2.65) | 3.15× (3.07–3.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 5.0 | 0.67× (0.66–0.68) | 3.11× (2.98–3.26) | 4.42× (4.28–4.57) | 3.14× (3.08–3.19) | 6.82× (6.72–6.93) | 2.58× (2.51–2.66) | 2.37× (2.33–2.42) | 0.98× (0.96–0.99) | 1.60× (1.45–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 692 | 5.5 | 0.56× (0.55–0.57) | 2.85× (2.80–2.89) | 4.89× (4.81–4.96) | 2.98× (2.94–3.02) | 6.38× (6.27–6.50) | 2.41× (2.33–2.49) | 2.21× (2.16–2.27) | 0.91× (0.90–0.93) | 1.49× (1.35–1.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.3 | 0.57× (0.56–0.58) | 2.85× (2.80–2.91) | 4.88× (4.78–4.98) | 2.96× (2.91–3.01) | 6.40× (6.28–6.51) | 2.40× (2.33–2.48) | 2.21× (2.16–2.26) | 0.91× (0.89–0.93) | 1.49× (1.36–1.63) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 0.066 s | 186 | 18.0 | 1.92× (1.88–1.96) | 4.03× (3.94–4.13) | 4.45× (4.32–4.59) | 4.42× (4.31–4.54) | 4.72× (4.61–4.84) | 5.16× (5.04–5.30) | 4.89× (4.76–5.02) | 1.49× (1.47–1.51) | 1.85× (1.80–1.90) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 0.048 s | 253 | 23.8 | 1.48× (1.41–1.56) | 3.17× (3.06–3.27) | 4.04× (3.92–4.16) | 4.18× (4.08–4.28) | 4.04× (3.98–4.11) | 7.17× (7.05–7.29) | 6.79× (6.66–6.93) | 2.07× (2.01–2.12) | 2.57× (2.49–2.65) | yes |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 0.042 s | 288 | 36.5 | 1.51× (1.46–1.57) | 2.35× (2.20–2.50) | 3.24× (3.13–3.36) | 3.45× (3.28–3.62) | 3.58× (3.48–3.69) | 8.14× (7.94–8.34) | 7.71× (7.49–7.93) | 2.35× (2.27–2.42) | 2.91× (2.82–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.86–2.90) | 6.60× (6.57–6.64) | 6.78× (6.71–6.86) | 7.39× (7.32–7.45) | 6.65× (6.61–6.68) | 7.21× (7.19–7.23) | 6.76× (6.74–6.77) | 2.14× (2.12–2.16) | 2.49× (2.47–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.70× (2.63–2.78) | 6.24× (6.00–6.49) | 6.41× (6.24–6.58) | 7.01× (6.85–7.17) | 6.08× (5.95–6.21) | 6.48× (6.34–6.64) | 6.07× (5.93–6.22) | 1.92× (1.87–1.98) | 2.24× (2.18–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.83× (2.78–2.89) | 4.79× (4.69–4.90) | 5.15× (5.02–5.28) | 5.44× (5.31–5.57) | 4.79× (4.71–4.87) | 4.27× (4.23–4.30) | 4.00× (3.96–4.03) | 1.27× (1.25–1.28) | 1.47× (1.45–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.63–0.68) | 3.31× (3.24–3.38) | 4.14× (4.03–4.25) | 3.38× (3.33–3.45) | 5.75× (5.65–5.84) | 2.84× (2.79–2.89) | 2.54× (2.50–2.59) | 1.01× (0.99–1.03) | 1.09× (1.05–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.04× (2.99–3.09) | 5.34× (5.25–5.44) | 3.27× (3.21–3.33) | 5.48× (5.39–5.56) | 2.69× (2.63–2.75) | 2.41× (2.37–2.45) | 0.96× (0.94–0.98) | 1.03× (0.99–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.42× (3.34–3.49) | 5.42× (5.30–5.54) | 3.33× (3.28–3.38) | 5.60× (5.48–5.72) | 2.76× (2.71–2.81) | 2.47× (2.43–2.51) | 0.98× (0.97–1.00) | 1.06× (1.02–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.50–2.55) | 2.80× (2.78–2.82) | 3.56× (3.45–3.66) | 3.66× (3.63–3.69) | 3.99× (3.96–4.03) | 2.48× (2.46–2.50) | 2.05× (2.03–2.08) | 1.96× (1.95–1.98) | 2.71× (2.68–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.21× (2.16–2.25) | 2.51× (2.45–2.57) | 3.04× (2.97–3.11) | 3.11× (3.05–3.17) | 3.36× (3.29–3.44) | 1.99× (1.95–2.03) | 1.65× (1.62–1.68) | 1.57× (1.54–1.61) | 2.17× (2.12–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.78–1.95) | 1.66× (1.60–1.72) | 2.08× (2.01–2.16) | 2.00× (1.92–2.08) | 2.01× (1.95–2.08) | 0.98× (0.94–1.01) | 0.81× (0.78–0.84) | 0.77× (0.75–0.80) | 1.07× (1.04–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.31× (1.30–1.32) | 2.58× (2.56–2.60) | 2.32× (2.30–2.33) | 2.40× (2.39–2.42) | 2.77× (2.76–2.79) | 3.03× (3.00–3.05) | 3.11× (3.08–3.13) | 1.20× (1.19–1.21) | 1.34× (1.33–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.25) | 2.41× (2.39–2.44) | 2.11× (2.09–2.13) | 2.80× (2.78–2.82) | 2.50× (2.47–2.53) | 2.66× (2.64–2.68) | 2.74× (2.71–2.76) | 1.06× (1.04–1.07) | 1.18× (1.16–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.35–1.50) | 2.26× (2.21–2.31) | 1.85× (1.81–1.89) | 2.53× (2.48–2.59) | 2.39× (2.35–2.44) | 1.99× (1.96–2.02) | 2.04× (2.00–2.08) | 0.79× (0.77–0.80) | 0.88× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.39–1.43) | 2.92× (2.88–2.96) | 2.45× (2.40–2.50) | 2.55× (2.52–2.58) | 3.04× (2.94–3.13) | 3.51× (3.47–3.55) | 3.83× (3.79–3.87) | 1.37× (1.35–1.40) | 1.63× (1.61–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.30–1.36) | 2.72× (2.69–2.75) | 2.25× (2.22–2.28) | 3.13× (3.07–3.19) | 2.72× (2.69–2.74) | 3.10× (3.05–3.14) | 3.38× (3.34–3.41) | 1.21× (1.19–1.24) | 1.43× (1.42–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.47–1.61) | 2.46× (2.42–2.51) | 1.88× (1.83–1.94) | 2.72× (2.66–2.78) | 2.46× (2.42–2.51) | 2.19× (2.15–2.22) | 2.38× (2.35–2.42) | 0.85× (0.83–0.88) | 1.01× (0.99–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.73× (2.68–2.78) | 3.57× (3.52–3.62) | 2.86× (2.82–2.91) | 6.05× (5.93–6.16) | 2.35× (2.31–2.39) | 2.18× (2.14–2.21) | 0.98× (0.97–0.99) | 1.08× (0.93–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.64) | 2.50× (2.47–2.53) | 4.24× (4.18–4.31) | 2.75× (2.72–2.79) | 5.63× (5.53–5.74) | 2.16× (2.13–2.19) | 2.00× (1.98–2.03) | 0.90× (0.89–0.92) | 0.99× (0.86–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.78× (2.72–2.84) | 4.29× (4.18–4.39) | 2.81× (2.75–2.86) | 5.77× (5.65–5.90) | 2.20× (2.15–2.26) | 2.04× (1.99–2.09) | 0.92× (0.90–0.94) | 1.01× (0.87–1.16) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 1.89× (1.85–1.93) | 3.91× (3.81–4.00) | 4.26× (4.14–4.39) | 4.32× (4.21–4.44) | 4.62× (4.51–4.73) | 5.05× (4.92–5.17) | 4.78× (4.66–4.91) | 1.48× (1.46–1.50) | 1.77× (1.72–1.82) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 1.57× (1.54–1.60) | 3.17× (3.12–3.23) | 3.47× (3.41–3.54) | 3.82× (3.77–3.88) | 3.64× (3.59–3.69) | 3.88× (3.83–3.93) | 3.68× (3.62–3.74) | 1.14× (1.11–1.16) | 1.36× (1.32–1.40) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 1.56× (1.46–1.66) | 2.51× (2.42–2.60) | 2.70× (2.61–2.79) | 2.97× (2.91–3.04) | 2.93× (2.84–3.02) | 2.51× (2.45–2.58) | 2.38× (2.31–2.45) | 0.73× (0.71–0.76) | 0.88× (0.86–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.31× (0.30–0.32) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.06× (11.04–11.08) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.53× (0.49–0.58) | 0.71× (0.69–0.72) | 0.59× (0.58–0.60) | 5.59× (5.46–5.73) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.31–0.34) | 0.56× (0.48–0.64) | 0.58× (0.56–0.61) | 0.51× (0.49–0.53) | 3.40× (3.27–3.53) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.15–0.17) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (8.00–8.05) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.67–0.68) | 0.73× (0.67–0.80) | 1.62× (1.60–1.64) | 0.81× (0.80–0.82) | 5.87× (5.80–5.94) | 0.47× (0.46–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.67) | 0.74× (0.70–0.78) | 1.17× (1.14–1.20) | 0.72× (0.70–0.74) | 3.43× (3.35–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.28) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.50–0.51) | 0.41× (0.39–0.43) | 0.90× (0.89–0.92) | 0.71× (0.69–0.74) | 3.13× (3.08–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.61–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.78–0.83) | 2.52× (2.46–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.47–0.50) | 0.35× (0.32–0.39) | 0.85× (0.83–0.88) | 0.70× (0.67–0.73) | 2.93× (2.84–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.49× (0.47–0.51) | 0.80× (0.79–0.82) | 0.81× (0.78–0.84) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.07–1.08) | 0.76× (0.75–0.76) | 5.36× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.30–5.33) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.43× (1.42–1.43) | 0.66× (0.66–0.67) | 6.42× (6.39–6.44) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 2 | 40 MB | 0.42× (0.40–0.43) | 0.37× (0.34–0.40) | 0.98× (0.95–1.01) | 0.62× (0.59–0.65) | 3.06× (2.98–3.15) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `text` (2.39 MB → 12.2 MB, 5.1×) | 4 | 58 MB | 0.48× (0.46–0.51) | 0.48× (0.43–0.52) | 0.87× (0.85–0.89) | 0.68× (0.66–0.70) | 2.21× (2.15–2.28) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 674 | 2.75 | 1.11× (1.08–1.14) | 85% | 1.05× (1.03–1.07) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 856 | 4.26 | 1.69× (1.67–1.71) | 53% | 1.23× (1.20–1.25) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (856 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (293 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.47, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.69 | 0.02 | 0.09 | 0.245 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3246 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.6% |
| MTF + RLE2 | 1.59 | 17.9% |
| T-vector build | 1.52 | 17.1% |
| walk (the one each build uses) | 4.10 | 46.3% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.87 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.306 | (in its total) | 2.65× (2.64–2.66) |  |
| walk | 1.236 | 2.651 | (in its total) | 2.14× (2.12–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.24× (1.21–1.27) |  |
| inverse BWT, total | 1.773 | 4.016 | 3.288 | 2.26× (2.24–2.28) | 1.85× (1.84–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m52s |
| 4 threads | 35m45s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m04s ± 3m33s |
| the scanner alone (its serial part) | 13m06s ± 0.00s |
| the workers alone, over 4 | 25m04s ± 3m33s |

At 4 threads the workers bound it. Per byte: the scanner 7.6 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 2.07 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,391,318 | 12,161,387 | \`09626dbf7dffd3f2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3246.5 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
