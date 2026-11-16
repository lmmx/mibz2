# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-503/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`4bca922`](<https://github.com/lmmx/mibz2/commit/4bca922442aacafd3b73500b3dacef80b7d13f48>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43926597379>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.525 s | 400 | 6.5 | 2.89× (2.87–2.90) | 6.64× (6.61–6.67) | 6.81× (6.77–6.85) | 7.40× (7.36–7.43) | 6.72× (6.66–6.78) | 7.27× (7.23–7.30) | 6.79× (6.76–6.82) | 2.12× (2.11–2.14) | 2.53× (2.52–2.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.303 s | 692 | 7.1 | 2.84× (2.72–2.98) | 6.05× (5.78–6.33) | 7.08× (6.88–7.29) | 6.75× (6.40–7.12) | 6.17× (5.89–6.46) | 12.32× (11.77–12.90) | 11.51× (10.99–12.05) | 3.60× (3.45–3.76) | 4.30× (4.10–4.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.246 s | 851 | 11.0 | 2.88× (2.83–2.93) | 4.38× (4.34–4.43) | 5.44× (5.34–5.54) | 4.94× (4.87–5.01) | 5.00× (4.94–5.06) | 15.47× (15.26–15.69) | 14.44× (14.24–14.65) | 4.52× (4.47–4.57) | 5.39× (5.33–5.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 988 | 2.9 | 0.62× (0.61–0.62) | 3.87× (3.69–4.06) | 4.51× (4.40–4.62) | 3.60× (3.56–3.64) | 6.19× (6.12–6.27) | 3.01× (2.97–3.05) | 2.69× (2.65–2.73) | 1.00× (0.98–1.03) | 1.52× (1.50–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 948 | 3.0 | 0.59× (0.58–0.61) | 2.94× (2.88–3.00) | 5.93× (5.78–6.08) | 3.51× (3.43–3.58) | 5.96× (5.85–6.08) | 2.91× (2.85–2.97) | 2.60× (2.54–2.66) | 0.97× (0.94–1.00) | 1.47× (1.44–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 962 | 3.0 | 0.59× (0.57–0.61) | 2.97× (2.93–3.01) | 6.01× (5.87–6.16) | 3.51× (3.46–3.56) | 6.05× (5.96–6.15) | 2.93× (2.88–2.97) | 2.61× (2.57–2.65) | 0.97× (0.95–1.00) | 1.48× (1.46–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 580 | 4.7 | 2.68× (2.58–2.79) | 3.03× (3.00–3.06) | 3.89× (3.85–3.92) | 3.81× (3.77–3.85) | 4.22× (4.18–4.26) | 2.56× (2.54–2.58) | 2.17× (2.14–2.21) | 2.02× (2.01–2.04) | 3.01× (2.99–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 889 | 5.8 | 2.25× (2.15–2.35) | 2.96× (2.80–3.14) | 4.16× (3.97–4.34) | 3.87× (3.39–4.41) | 4.88× (4.70–5.06) | 3.84× (3.68–3.99) | 3.26× (3.10–3.42) | 3.03× (2.90–3.17) | 4.52× (4.34–4.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 767 | 11.8 | 1.88× (1.68–2.12) | 1.88× (1.72–2.05) | 2.97× (2.78–3.17) | 2.22× (2.11–2.34) | 3.85× (3.68–4.02) | 3.34× (3.20–3.49) | 2.84× (2.71–2.98) | 2.64× (2.53–2.76) | 3.94× (3.77–4.12) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 43.6 | 1.34× (1.33–1.35) | 2.67× (2.65–2.69) | 2.38× (2.36–2.41) | 2.42× (2.41–2.44) | 2.82× (2.80–2.84) | 3.09× (3.07–3.12) | 3.15× (3.09–3.22) | 1.20× (1.19–1.21) | 1.37× (1.36–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.084 s | 95 | 49.9 | 1.30× (1.18–1.43) | 2.47× (2.43–2.51) | 2.32× (2.28–2.36) | 2.98× (2.76–3.21) | 2.85× (2.79–2.92) | 4.96× (4.86–5.06) | 5.06× (4.94–5.18) | 1.93× (1.89–1.97) | 2.19× (2.15–2.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 68.0 | 1.32× (1.29–1.35) | 2.14× (2.10–2.19) | 2.05× (1.99–2.11) | 2.48× (2.42–2.54) | 2.79× (2.76–2.83) | 6.01× (5.94–6.08) | 6.12× (5.99–6.26) | 2.34× (2.30–2.37) | 2.65× (2.62–2.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.147 s | 65 | 40.0 | 1.45× (1.42–1.47) | 2.97× (2.91–3.02) | 2.53× (2.50–2.57) | 2.56× (2.51–2.61) | 3.00× (2.93–3.06) | 3.55× (3.47–3.63) | 3.83× (3.75–3.91) | 1.36× (1.34–1.39) | 1.65× (1.61–1.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 106 | 45.4 | 1.34× (1.31–1.37) | 2.72× (2.66–2.79) | 2.42× (2.37–2.48) | 3.21× (3.12–3.29) | 3.01× (2.94–3.09) | 5.84× (5.72–5.95) | 6.29× (6.17–6.42) | 2.24× (2.19–2.30) | 2.72× (2.64–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 65.2 | 1.39× (1.37–1.41) | 2.21× (2.15–2.28) | 2.15× (2.07–2.23) | 2.78× (2.71–2.86) | 2.71× (2.66–2.76) | 6.66× (6.51–6.80) | 7.18× (7.04–7.32) | 2.56× (2.51–2.61) | 3.10× (3.03–3.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 4.0 | 0.68× (0.67–0.69) | 3.28× (3.03–3.56) | 4.48× (4.42–4.55) | 3.15× (3.11–3.18) | 6.83× (6.73–6.94) | 2.59× (2.55–2.64) | 2.37× (2.34–2.41) | 0.99× (0.97–1.01) | 1.54× (1.52–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 687 | 4.3 | 0.57× (0.55–0.58) | 2.82× (2.76–2.89) | 4.92× (4.74–5.10) | 2.95× (2.87–3.04) | 6.32× (6.17–6.47) | 2.38× (2.31–2.45) | 2.18× (2.12–2.24) | 0.91× (0.88–0.94) | 1.42× (1.38–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 4.2 | 0.58× (0.56–0.61) | 2.83× (2.78–2.88) | 4.84× (4.75–4.93) | 2.95× (2.89–3.01) | 6.34× (6.20–6.48) | 2.38× (2.34–2.43) | 2.18× (2.14–2.22) | 0.91× (0.89–0.93) | 1.42× (1.39–1.44) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 0.073 s | 188 | 14.0 | 1.99× (1.97–2.01) | 4.20× (4.17–4.22) | 4.49× (4.43–4.55) | 4.60× (4.56–4.63) | 4.74× (4.69–4.79) | 5.33× (5.30–5.35) | 4.98× (4.94–5.01) | 1.51× (1.50–1.52) | 1.95× (1.94–1.96) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 0.051 s | 266 | 18.1 | 1.55× (1.49–1.62) | 3.22× (3.11–3.33) | 3.97× (3.79–4.16) | 4.13× (3.97–4.30) | 4.22× (4.04–4.41) | 7.43× (7.21–7.67) | 6.95× (6.72–7.17) | 2.11× (2.04–2.18) | 2.72× (2.64–2.81) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 0.047 s | 292 | 26.9 | 1.52× (1.45–1.59) | 2.35× (2.29–2.41) | 3.30× (3.19–3.41) | 3.15× (3.08–3.23) | 3.68× (3.61–3.74) | 8.31× (8.14–8.48) | 7.76× (7.61–7.91) | 2.36× (2.31–2.40) | 3.04× (2.98–3.10) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.86–2.89) | 6.61× (6.57–6.64) | 6.78× (6.74–6.82) | 7.37× (7.34–7.40) | 6.69× (6.63–6.75) | 7.24× (7.21–7.28) | 6.76× (6.73–6.79) | 2.12× (2.11–2.13) | 2.52× (2.51–2.53) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.70× (2.64–2.76) | 6.17× (6.04–6.30) | 6.54× (6.35–6.73) | 7.02× (6.87–7.18) | 6.16× (6.02–6.29) | 6.51× (6.39–6.64) | 6.08× (5.95–6.21) | 1.91× (1.86–1.95) | 2.26× (2.21–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.78× (2.73–2.84) | 4.71× (4.62–4.80) | 5.05× (4.97–5.14) | 5.30× (5.23–5.37) | 4.73× (4.66–4.81) | 4.28× (4.22–4.34) | 3.99× (3.93–4.06) | 1.25× (1.23–1.27) | 1.49× (1.46–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.65–0.66) | 3.31× (3.26–3.37) | 4.09× (4.04–4.15) | 3.38× (3.34–3.42) | 5.74× (5.66–5.82) | 2.84× (2.79–2.88) | 2.54× (2.50–2.58) | 1.01× (0.98–1.03) | 1.04× (1.02–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.62–0.66) | 3.06× (3.00–3.13) | 5.30× (5.21–5.38) | 3.26× (3.20–3.32) | 5.45× (5.37–5.54) | 2.70× (2.65–2.75) | 2.42× (2.37–2.47) | 0.96× (0.93–0.99) | 0.99× (0.97–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.61–0.64) | 3.47× (3.37–3.58) | 5.39× (5.32–5.46) | 3.31× (3.26–3.35) | 5.58× (5.50–5.67) | 2.75× (2.71–2.79) | 2.47× (2.43–2.50) | 0.97× (0.95–1.00) | 1.01× (0.99–1.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.60× (2.49–2.70) | 2.81× (2.78–2.84) | 3.54× (3.51–3.58) | 3.66× (3.61–3.70) | 4.04× (4.00–4.09) | 2.47× (2.45–2.50) | 2.11× (2.07–2.15) | 1.97× (1.95–1.98) | 2.71× (2.67–2.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.23–2.29) | 2.54× (2.52–2.57) | 3.07× (3.02–3.12) | 3.31× (3.23–3.40) | 3.46× (3.41–3.52) | 2.00× (1.98–2.03) | 1.71× (1.67–1.75) | 1.59× (1.56–1.62) | 2.20× (2.15–2.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.83–2.00) | 1.68× (1.61–1.75) | 2.14× (2.07–2.21) | 2.03× (1.96–2.11) | 2.06× (2.00–2.13) | 0.99× (0.96–1.02) | 0.85× (0.82–0.88) | 0.79× (0.77–0.81) | 1.09× (1.05–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.35) | 2.60× (2.58–2.63) | 2.33× (2.31–2.34) | 2.40× (2.39–2.42) | 2.79× (2.77–2.82) | 3.06× (3.04–3.09) | 3.12× (3.06–3.19) | 1.20× (1.19–1.21) | 1.34× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.23–1.26) | 2.41× (2.39–2.44) | 2.12× (2.10–2.14) | 2.72× (2.67–2.77) | 2.50× (2.47–2.54) | 2.68× (2.65–2.72) | 2.74× (2.68–2.79) | 1.05× (1.04–1.06) | 1.17× (1.16–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.41–1.47) | 2.22× (2.17–2.27) | 1.81× (1.77–1.84) | 2.38× (2.36–2.40) | 2.33× (2.30–2.35) | 1.97× (1.95–1.99) | 2.01× (1.97–2.05) | 0.77× (0.76–0.78) | 0.86× (0.85–0.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.41–1.47) | 2.92× (2.86–2.98) | 2.47× (2.43–2.51) | 2.54× (2.49–2.59) | 2.97× (2.91–3.03) | 3.51× (3.44–3.59) | 3.79× (3.71–3.87) | 1.36× (1.34–1.38) | 1.62× (1.59–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.37) | 2.73× (2.70–2.76) | 2.26× (2.23–2.28) | 3.02× (2.97–3.07) | 2.71× (2.67–2.75) | 3.10× (3.07–3.14) | 3.35× (3.30–3.40) | 1.20× (1.18–1.23) | 1.43× (1.40–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.61) | 2.46× (2.40–2.51) | 1.88× (1.84–1.91) | 2.57× (2.52–2.61) | 2.45× (2.41–2.49) | 2.18× (2.15–2.22) | 2.35× (2.33–2.38) | 0.84× (0.83–0.86) | 1.00× (0.99–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.73) | 2.82× (2.69–2.95) | 3.57× (3.52–3.63) | 2.86× (2.83–2.90) | 6.08× (5.98–6.18) | 2.38× (2.34–2.43) | 2.19× (2.16–2.23) | 0.99× (0.97–1.00) | 1.01× (0.97–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.64) | 2.51× (2.44–2.58) | 4.21× (4.09–4.34) | 2.74× (2.68–2.80) | 5.60× (5.47–5.74) | 2.17× (2.13–2.22) | 2.00× (1.96–2.05) | 0.90× (0.88–0.92) | 0.92× (0.88–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.63× (0.61–0.65) | 2.77× (2.72–2.83) | 4.23× (4.14–4.32) | 2.79× (2.73–2.85) | 5.70× (5.59–5.82) | 2.21× (2.17–2.26) | 2.04× (2.00–2.08) | 0.92× (0.90–0.93) | 0.93× (0.90–0.97) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 1.96× (1.94–1.98) | 4.03× (4.01–4.06) | 4.36× (4.30–4.41) | 4.51× (4.47–4.54) | 4.64× (4.59–4.69) | 5.21× (5.19–5.24) | 4.87× (4.84–4.91) | 1.50× (1.49–1.51) | 1.86× (1.84–1.87) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 1.62× (1.56–1.68) | 3.29× (3.18–3.40) | 3.58× (3.46–3.70) | 3.93× (3.79–4.07) | 3.75× (3.58–3.93) | 4.04× (3.91–4.18) | 3.78× (3.64–3.92) | 1.16× (1.12–1.20) | 1.44× (1.39–1.49) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 1.78× (1.69–1.87) | 2.66× (2.56–2.77) | 2.91× (2.81–3.01) | 3.16× (3.05–3.28) | 3.05× (2.96–3.15) | 2.72× (2.63–2.80) | 2.54× (2.46–2.62) | 0.78× (0.75–0.81) | 0.97× (0.94–1.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.27–0.31) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.05× (11.02–11.09) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.33) | 0.57× (0.51–0.63) | 0.68× (0.67–0.70) | 0.57× (0.56–0.58) | 5.41× (5.33–5.50) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.32× (0.31–0.33) | 0.55× (0.47–0.65) | 0.58× (0.56–0.59) | 0.50× (0.49–0.51) | 3.34× (3.27–3.42) | 0.08× (0.08–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.70) | 0.65× (0.65–0.66) | 8.02× (8.00–8.04) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.73× (0.69–0.78) | 1.62× (1.61–1.63) | 0.81× (0.80–0.83) | 5.87× (5.85–5.90) | 0.48× (0.47–0.48) | 0.50× (0.50–0.50) | 0.54× (0.54–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.67) | 0.70× (0.65–0.75) | 1.17× (1.14–1.19) | 0.72× (0.70–0.74) | 3.42× (3.35–3.50) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.73) | 5.20× (5.19–5.21) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.51× (0.49–0.53) | 0.41× (0.37–0.44) | 0.91× (0.88–0.94) | 0.71× (0.66–0.75) | 3.15× (3.04–3.27) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.43–0.46) | 0.85× (0.83–0.87) | 0.78× (0.75–0.80) | 2.48× (2.42–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.22–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.51) | 0.39× (0.37–0.41) | 0.86× (0.84–0.88) | 0.70× (0.66–0.73) | 2.95× (2.87–3.04) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.52) | 0.80× (0.79–0.81) | 0.83× (0.81–0.85) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.35–5.40) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.29× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.77–0.78) | 5.33× (5.32–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.66–0.67) | 6.40× (6.39–6.41) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.42× (0.39–0.45) | 1.03× (1.00–1.07) | 0.67× (0.65–0.70) | 3.28× (3.21–3.36) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 58 MB | 0.49× (0.47–0.51) | 0.49× (0.44–0.54) | 0.90× (0.86–0.94) | 0.72× (0.68–0.76) | 2.28× (2.19–2.38) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 692 | 2.77 | 1.11× (1.09–1.14) | 85% | 1.04× (1.03–1.05) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 851 | 4.28 | 1.69× (1.67–1.72) | 53% | 1.21× (1.20–1.22) | 1.64× (1.62–1.65) |

**Best:** mibz2 at 4 threads (851 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.16, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.69 | 0.02 | 0.09 | 0.246 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2570 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.90 | 12.6% |
| MTF + RLE2 | 1.25 | 17.6% |
| T-vector build | 1.21 | 17.0% |
| walk (the one each build uses) | 3.32 | 46.6% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.22 | 3.1% |
| CRC | 0.20 | 2.9% |
| **total** | 7.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.488 | 1.304 | (in its total) | 2.67× (2.66–2.69) |  |
| walk | 1.228 | 2.653 | (in its total) | 2.16× (2.14–2.18) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.19× (1.17–1.21) |  |
| inverse BWT, total | 1.764 | 4.013 | 3.296 | 2.27× (2.25–2.29) | 1.86× (1.85–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m43s |
| 4 threads | 35m51s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m57s ± 3m31s |
| the scanner alone (its serial part) | 14m07s ± 30.2s |
| the workers alone, over 4 | 24m57s ± 3m31s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 3.27 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,698,958 | 13,676,554 | \`b30556180238088f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2570.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
