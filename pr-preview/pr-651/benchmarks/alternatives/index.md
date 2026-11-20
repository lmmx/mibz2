# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-651/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`1763b70`](<https://github.com/lmmx/mibz2/commit/1763b7050664c4189f5810d780c023df79a02548>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44613722588>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.297 s | 706 | 3.7 | 2.80× (2.77–2.82) | 7.07× (6.98–7.17) | 7.38× (7.31–7.44) | 8.39× (8.29–8.50) | 7.45× (7.37–7.53) | 7.74× (7.65–7.84) | 7.48× (7.36–7.60) | 2.21× (2.17–2.25) | 2.56× (2.52–2.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.184 s | 1,137 | 4.0 | 2.70× (2.51–2.90) | 6.13× (5.81–6.47) | 7.34× (6.94–7.77) | 7.50× (7.03–8.00) | 6.58× (6.26–6.93) | 12.68× (12.03–13.36) | 12.24× (11.57–12.95) | 3.62× (3.43–3.81) | 4.18× (3.96–4.42) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.154 s | 1,360 | 6.2 | 2.70× (2.65–2.76) | 4.37× (4.28–4.46) | 5.61× (5.49–5.74) | 5.29× (5.19–5.40) | 5.20× (5.09–5.30) | 15.23× (14.92–15.54) | 14.70× (14.27–15.15) | 4.34× (4.23–4.46) | 5.02× (4.90–5.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,429 | 2.0 | 0.57× (0.56–0.57) | 3.65× (3.58–3.72) | 4.58× (4.36–4.80) | 3.73× (3.66–3.79) | 6.52× (6.43–6.62) | 2.80× (2.75–2.84) | 2.78× (2.72–2.83) | 1.01× (0.99–1.03) | 2.19× (2.15–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,418 | 2.1 | 0.51× (0.50–0.52) | 2.95× (2.86–3.04) | 5.96× (5.85–6.08) | 3.68× (3.60–3.77) | 6.41× (6.27–6.54) | 2.75× (2.70–2.80) | 2.73× (2.67–2.79) | 0.99× (0.97–1.02) | 2.16× (2.09–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,370 | 2.1 | 0.51× (0.50–0.52) | 2.87× (2.72–3.04) | 5.98× (5.49–6.51) | 3.61× (3.55–3.67) | 6.26× (6.10–6.43) | 2.67× (2.62–2.73) | 2.66× (2.58–2.73) | 0.96× (0.94–0.99) | 2.10× (2.05–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,095 | 2.6 | 2.86× (2.82–2.90) | 3.52× (3.37–3.67) | 4.23× (4.04–4.42) | 4.62× (4.53–4.71) | 5.43× (5.28–5.57) | 3.17× (2.98–3.37) | 2.59× (2.54–2.63) | 2.15× (2.11–2.19) | 3.35× (3.26–3.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,537 | 3.4 | 2.13× (1.85–2.45) | 3.23× (2.86–3.65) | 4.22× (3.63–4.90) | 3.77× (3.21–4.43) | 5.69× (4.99–6.49) | 4.25× (3.70–4.89) | 3.47× (3.02–3.98) | 2.88× (2.51–3.31) | 4.49× (3.91–5.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,150 | 6.9 | 1.63× (1.50–1.76) | 1.86× (1.74–1.99) | 2.71× (2.48–2.97) | 1.97× (1.85–2.09) | 4.13× (3.85–4.42) | 3.43× (3.08–3.82) | 2.80× (2.58–3.03) | 2.32× (2.15–2.51) | 3.62× (3.39–3.88) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.079 s | 102 | 26.1 | 1.39× (1.37–1.41) | 3.18× (3.11–3.26) | 2.88× (2.82–2.93) | 2.82× (2.76–2.87) | 3.14× (3.08–3.19) | 3.56× (3.49–3.62) | 3.62× (3.56–3.68) | 1.35× (1.32–1.38) | 1.57× (1.52–1.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 155 | 30.0 | 1.25× (1.20–1.31) | 2.84× (2.75–2.94) | 2.62× (2.53–2.73) | 3.25× (3.11–3.41) | 3.08× (2.96–3.20) | 5.42× (5.21–5.64) | 5.52× (5.30–5.75) | 2.05× (1.98–2.13) | 2.39× (2.29–2.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 193 | 40.0 | 1.28× (1.24–1.31) | 2.48× (2.40–2.57) | 2.44× (2.38–2.52) | 2.82× (2.75–2.89) | 3.15× (3.07–3.24) | 6.69× (6.49–6.90) | 6.81× (6.60–7.03) | 2.54× (2.47–2.61) | 2.95× (2.87–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.083 s | 116 | 22.9 | 1.49× (1.48–1.51) | 3.76× (3.73–3.80) | 3.15× (3.12–3.19) | 3.16× (3.13–3.19) | 3.57× (3.53–3.61) | 4.50× (4.46–4.55) | 4.83× (4.77–4.89) | 1.62× (1.59–1.64) | 1.93× (1.85–2.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.052 s | 183 | 27.2 | 1.24× (1.08–1.42) | 3.18× (2.81–3.60) | 2.66× (2.32–3.04) | 3.62× (3.18–4.12) | 3.23× (2.88–3.63) | 6.66× (5.83–7.60) | 7.14× (6.27–8.13) | 2.39× (2.10–2.71) | 2.85× (2.46–3.31) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 213 | 37.9 | 1.40× (1.35–1.46) | 2.72× (2.62–2.83) | 2.51× (2.42–2.61) | 3.39× (3.28–3.51) | 3.18× (3.09–3.27) | 8.25× (7.97–8.53) | 8.85× (8.54–9.16) | 2.96× (2.87–3.05) | 3.53× (3.32–3.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,058 | 2.9 | 0.62× (0.61–0.63) | 3.23× (3.13–3.34) | 4.51× (4.33–4.68) | 3.17× (3.09–3.26) | 7.11× (7.00–7.21) | 2.32× (2.28–2.37) | 2.40× (2.31–2.50) | 0.97× (0.96–0.99) | 2.16× (2.09–2.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,005 | 3.1 | 0.51× (0.49–0.52) | 3.09× (2.97–3.20) | 5.23× (4.91–5.56) | 3.03× (2.95–3.11) | 6.74× (6.55–6.94) | 2.21× (2.17–2.26) | 2.29× (2.20–2.39) | 0.93× (0.90–0.95) | 2.06× (1.99–2.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 980 | 3.0 | 0.50× (0.49–0.51) | 3.02× (2.93–3.11) | 5.21× (5.04–5.38) | 2.98× (2.90–3.05) | 6.58× (6.40–6.75) | 2.16× (2.11–2.22) | 2.24× (2.17–2.31) | 0.91× (0.88–0.93) | 2.01× (1.95–2.08) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 0.043 s | 307 | 8.8 | 1.71× (1.70–1.72) | 4.25× (4.18–4.33) | 4.66× (4.56–4.77) | 4.78× (4.71–4.86) | 5.01× (4.94–5.08) | 5.13× (5.04–5.23) | 4.92× (4.87–4.98) | 1.50× (1.50–1.51) | 1.72× (1.66–1.78) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 0.033 s | 402 | 11.6 | 1.25× (1.03–1.52) | 3.09× (2.58–3.70) | 3.67× (3.15–4.28) | 3.92× (3.24–4.74) | 4.08× (3.35–4.97) | 6.49× (5.35–7.88) | 6.22× (5.17–7.49) | 1.90× (1.58–2.29) | 2.17× (1.77–2.67) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 0.031 s | 429 | 16.8 | 1.24× (1.20–1.28) | 2.38× (2.27–2.50) | 2.99× (2.86–3.13) | 3.14× (3.03–3.26) | 3.90× (3.48–4.37) | 7.36× (7.19–7.53) | 7.06× (6.90–7.22) | 2.16× (2.11–2.20) | 2.46× (2.35–2.58) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.79× (2.76–2.81) | 7.02× (6.93–7.12) | 7.33× (7.27–7.40) | 8.35× (8.25–8.46) | 7.41× (7.34–7.49) | 7.70× (7.61–7.80) | 7.44× (7.32–7.56) | 2.20× (2.16–2.24) | 2.53× (2.50–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.62× (2.58–2.65) | 6.57× (6.48–6.67) | 6.95× (6.85–7.04) | 8.08× (7.98–8.18) | 6.81× (6.71–6.90) | 7.00× (6.93–7.07) | 6.76× (6.61–6.91) | 2.00× (1.97–2.04) | 2.30× (2.26–2.34) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.74× (2.66–2.83) | 5.12× (4.89–5.36) | 5.56× (5.34–5.79) | 6.28× (6.00–6.58) | 5.34× (5.15–5.53) | 4.62× (4.47–4.79) | 4.47× (4.28–4.66) | 1.32× (1.27–1.38) | 1.52× (1.46–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 3.07× (3.02–3.13) | 4.00× (3.78–4.23) | 3.44× (3.39–3.49) | 5.95× (5.87–6.04) | 2.61× (2.58–2.64) | 2.59× (2.54–2.64) | 1.01× (0.99–1.03) | 1.83× (1.80–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.57–0.58) | 2.83× (2.79–2.87) | 5.00× (4.95–5.05) | 3.38× (3.31–3.45) | 5.77× (5.67–5.88) | 2.53× (2.49–2.57) | 2.51× (2.46–2.56) | 0.98× (0.95–1.00) | 1.78× (1.74–1.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 3.01× (2.94–3.08) | 5.19× (4.73–5.70) | 3.40× (3.34–3.46) | 5.78× (5.64–5.92) | 2.53× (2.48–2.57) | 2.51× (2.44–2.58) | 0.98× (0.95–1.00) | 1.78× (1.74–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.67–2.74) | 3.16× (3.10–3.22) | 3.79× (3.72–3.86) | 4.32× (4.24–4.40) | 5.06× (4.94–5.19) | 2.99× (2.82–3.17) | 2.46× (2.41–2.50) | 2.05× (2.02–2.09) | 2.77× (2.72–2.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.24–2.33) | 2.76× (2.66–2.86) | 3.23× (3.17–3.30) | 3.60× (3.50–3.70) | 4.13× (4.02–4.24) | 2.34× (2.22–2.47) | 1.92× (1.88–1.96) | 1.60× (1.57–1.64) | 2.16× (2.12–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.87× (1.76–1.98) | 1.72× (1.64–1.81) | 2.06× (1.94–2.18) | 2.03× (1.92–2.15) | 2.28× (2.16–2.40) | 1.15× (1.06–1.23) | 0.94× (0.89–0.99) | 0.79× (0.75–0.83) | 1.06× (1.01–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.40) | 3.08× (3.01–3.14) | 2.77× (2.71–2.82) | 2.78× (2.73–2.83) | 3.09× (3.04–3.15) | 3.50× (3.44–3.56) | 3.57× (3.51–3.63) | 1.34× (1.31–1.36) | 1.47× (1.44–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.82× (2.78–2.86) | 2.45× (2.41–2.49) | 3.19× (3.13–3.25) | 2.78× (2.73–2.82) | 3.05× (3.00–3.10) | 3.11× (3.05–3.16) | 1.17× (1.15–1.18) | 1.28× (1.26–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.43–1.48) | 2.63× (2.58–2.69) | 2.19× (2.15–2.23) | 2.88× (2.84–2.91) | 2.72× (2.68–2.76) | 2.31× (2.27–2.35) | 2.35× (2.31–2.39) | 0.88× (0.87–0.90) | 0.97× (0.96–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.47–1.50) | 3.67× (3.64–3.70) | 3.05× (3.02–3.09) | 3.12× (3.09–3.14) | 3.52× (3.48–3.56) | 4.43× (4.39–4.48) | 4.76× (4.70–4.81) | 1.60× (1.58–1.62) | 1.85× (1.76–1.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.26× (1.10–1.45) | 3.10× (2.73–3.51) | 2.48× (2.17–2.85) | 3.46× (3.05–3.93) | 2.89× (2.57–3.24) | 3.56× (3.11–4.07) | 3.82× (3.34–4.36) | 1.29× (1.13–1.47) | 1.49× (1.27–1.73) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.51–1.63) | 2.97× (2.87–3.07) | 2.20× (2.14–2.26) | 3.14× (3.06–3.22) | 2.85× (2.78–2.93) | 2.72× (2.64–2.80) | 2.92× (2.83–3.01) | 0.98× (0.95–1.01) | 1.13× (1.07–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.50× (2.46–2.55) | 3.31× (3.25–3.37) | 2.86× (2.79–2.93) | 6.21× (6.13–6.30) | 2.13× (2.09–2.16) | 2.21× (2.10–2.33) | 0.98× (0.96–0.99) | 1.50× (1.47–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.33× (2.27–2.38) | 3.94× (3.85–4.03) | 2.75× (2.69–2.82) | 5.83× (5.69–5.97) | 2.00× (1.96–2.03) | 2.08× (1.97–2.19) | 0.92× (0.90–0.94) | 1.41× (1.37–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.48× (2.42–2.53) | 3.94× (3.85–4.04) | 2.80× (2.74–2.86) | 5.91× (5.76–6.05) | 2.03× (1.99–2.08) | 2.11× (2.03–2.20) | 0.93× (0.91–0.96) | 1.44× (1.40–1.47) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 1.68× (1.67–1.70) | 4.05× (4.00–4.10) | 4.41× (4.31–4.51) | 4.65× (4.58–4.72) | 4.86× (4.80–4.93) | 4.99× (4.90–5.07) | 4.78× (4.73–4.83) | 1.49× (1.48–1.49) | 1.62× (1.61–1.64) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 1.30× (1.09–1.56) | 3.10× (2.57–3.72) | 3.36× (2.80–4.01) | 3.88× (3.24–4.65) | 3.59× (2.99–4.31) | 3.64× (3.02–4.38) | 3.49× (2.92–4.16) | 1.08× (0.91–1.30) | 1.18× (0.99–1.42) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 1.46× (1.39–1.54) | 2.67× (2.56–2.79) | 2.92× (2.86–2.99) | 3.38× (3.27–3.50) | 3.21× (3.00–3.43) | 2.59× (2.54–2.64) | 2.48× (2.45–2.52) | 0.77× (0.77–0.78) | 0.84× (0.83–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.80× (0.79–0.81) | 10.94× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.33× (0.32–0.34) | 0.46× (0.41–0.52) | 0.66× (0.64–0.68) | 0.56× (0.54–0.58) | 5.33× (5.17–5.49) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 97 MB | 0.32× (0.31–0.34) | 0.46× (0.39–0.54) | 0.57× (0.55–0.60) | 0.50× (0.48–0.53) | 3.35× (3.21–3.50) | 0.08× (0.08–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.21–0.22) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.76) | 0.83× (0.83–0.84) | 4.36× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.67) | 7.95× (7.91–7.98) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.69) | 0.66× (0.60–0.73) | 1.55× (1.48–1.62) | 0.77× (0.74–0.80) | 5.60× (5.37–5.84) | 0.45× (0.43–0.47) | 0.48× (0.46–0.50) | 0.52× (0.50–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.62× (0.59–0.66) | 0.66× (0.61–0.71) | 1.09× (1.04–1.16) | 0.69× (0.66–0.72) | 3.22× (3.06–3.39) | 0.25× (0.24–0.26) | 0.27× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.41× (0.40–0.43) | 0.89× (0.85–0.93) | 0.70× (0.66–0.74) | 3.08× (2.95–3.21) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.83× (0.81–0.85) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.46–0.49) | 0.38× (0.35–0.40) | 0.84× (0.81–0.87) | 0.71× (0.68–0.74) | 2.88× (2.79–2.98) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.47× (0.45–0.50) | 0.79× (0.78–0.80) | 0.80× (0.78–0.83) | 2.28× (2.25–2.31) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.25–5.28) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.28–5.29) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.33× (6.30–6.36) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.38× (0.35–0.42) | 1.04× (1.02–1.06) | 0.68× (0.67–0.70) | 3.34× (3.28–3.39) | 0.21× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.45× (0.41–0.50) | 0.93× (0.91–0.95) | 0.77× (0.75–0.79) | 2.37× (2.32–2.41) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 706 | 1.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,137 | 1.55 | 1.10× (1.09–1.12) | 82% | 1.03× (1.02–1.04) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,360 | 2.39 | 1.67× (1.61–1.73) | 49% | 1.21× (1.19–1.24) | 1.64× (1.61–1.66) |

**Best:** mibz2 at 4 threads (1,360 MB/s); lbzip2 at 4 threads (314 MB/s); pbz2 at 4 threads (505 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.93, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.58. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.37 | 0.01 | 0.07 | 0.154 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.63 | 14.6% |
| MTF + RLE2 | 0.89 | 20.6% |
| T-vector build | 0.80 | 18.6% |
| walk (the one each build uses) | 1.74 | 40.4% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.09 | 2.1% |
| CRC | 0.15 | 3.6% |
| **total** | 4.30 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.304 | 0.734 | (in its total) | 2.41× (2.40–2.42) |  |
| walk | 0.651 | 1.656 | (in its total) | 2.54× (2.52–2.57) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.031 | (in its total) | 1.16× (1.14–1.18) |  |
| inverse BWT, total | 0.981 | 2.420 | 1.339 | 2.47× (2.45–2.48) | 1.38× (1.36–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m42s ± 5m07s |
| 4 threads | 22m13s ± 3m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m23s ± 1m58s |
| the scanner alone (its serial part) | 8m00s ± 32.9s |
| the workers alone, over 4 | 13m23s ± 1m58s |

At 4 threads the workers bound it. Per byte: the scanner 4.6 ns a byte of bz2, the workers 1.75 ns a byte of output at 4 threads (CPU, summed over them), 1.17 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,615,664 | 13,288,779 | \`0b9f69f4e27fcf7e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
