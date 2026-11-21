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

The latest record: [`70b18ca`](<https://github.com/lmmx/mibz2/commit/70b18ca4411fa10c9f29b80821c835358dde2f9e>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44847138797>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.288 s | 728 | 6.3 | 2.94× (2.91–2.98) | 7.20× (7.16–7.25) | 7.50× (7.44–7.57) | 8.72× (8.48–8.97) | 7.62× (7.47–7.78) | 7.87× (7.80–7.95) | 7.51× (7.42–7.60) | 2.22× (2.20–2.24) | 2.57× (2.56–2.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.172 s | 1,223 | 6.9 | 2.88× (2.72–3.06) | 6.17× (5.88–6.47) | 7.45× (7.10–7.83) | 7.53× (7.15–7.94) | 6.76× (6.48–7.06) | 12.97× (12.31–13.66) | 12.37× (11.75–13.02) | 3.66× (3.47–3.85) | 4.24× (4.03–4.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.144 s | 1,452 | 10.4 | 2.74× (2.70–2.78) | 4.45× (4.33–4.58) | 5.69× (5.58–5.80) | 5.40× (5.34–5.47) | 5.43× (5.33–5.54) | 15.75× (15.45–16.05) | 15.03× (14.73–15.33) | 4.44× (4.35–4.54) | 5.15× (5.02–5.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,465 | 3.4 | 0.58× (0.57–0.58) | 3.74× (3.67–3.81) | 4.64× (4.55–4.73) | 3.76× (3.70–3.83) | 6.63× (6.53–6.73) | 2.84× (2.79–2.88) | 2.83× (2.79–2.88) | 1.01× (0.99–1.03) | 2.25× (2.21–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,434 | 3.6 | 0.52× (0.52–0.53) | 2.92× (2.84–3.01) | 5.98× (5.81–6.17) | 3.67× (3.59–3.75) | 6.38× (6.20–6.56) | 2.76× (2.69–2.83) | 2.76× (2.68–2.83) | 0.98× (0.95–1.02) | 2.19× (2.12–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,418 | 3.5 | 0.52× (0.50–0.53) | 2.87× (2.81–2.93) | 5.87× (5.76–5.99) | 3.63× (3.53–3.73) | 6.35× (6.23–6.47) | 2.71× (2.65–2.77) | 2.70× (2.65–2.76) | 0.97× (0.93–1.00) | 2.14× (2.10–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,120 | 4.3 | 3.06× (3.04–3.08) | 3.64× (3.47–3.83) | 4.28× (4.14–4.42) | 4.76× (4.67–4.85) | 5.48× (5.43–5.53) | 3.13× (3.07–3.19) | 2.63× (2.62–2.65) | 2.26× (2.10–2.44) | 3.47× (3.43–3.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.012 s | 1,613 | 5.6 | 2.32× (2.08–2.58) | 3.20× (2.88–3.55) | 4.35× (3.91–4.83) | 3.93× (3.55–4.35) | 5.80× (5.23–6.42) | 4.29× (3.90–4.72) | 3.61× (3.25–4.02) | 3.11× (2.73–3.53) | 4.76× (4.29–5.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,260 | 11.8 | 1.71× (1.67–1.74) | 2.04× (1.99–2.09) | 2.85× (2.79–2.92) | 2.14× (2.06–2.22) | 4.30× (4.20–4.41) | 3.53× (3.45–3.62) | 2.98× (2.92–3.04) | 2.56× (2.39–2.74) | 3.92× (3.83–4.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.078 s | 103 | 45.1 | 1.40× (1.36–1.44) | 3.15× (3.12–3.19) | 2.82× (2.76–2.87) | 2.79× (2.77–2.82) | 3.16× (3.08–3.24) | 3.56× (3.48–3.65) | 3.59× (3.56–3.62) | 1.34× (1.32–1.36) | 1.55× (1.54–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.050 s | 161 | 52.3 | 1.26× (1.19–1.34) | 2.81× (2.64–2.98) | 2.59× (2.46–2.74) | 3.18× (2.97–3.41) | 3.10× (2.86–3.36) | 5.49× (5.12–5.88) | 5.53× (5.21–5.87) | 2.06× (1.96–2.16) | 2.39× (2.26–2.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 193 | 69.8 | 1.24× (1.19–1.29) | 2.40× (2.33–2.47) | 2.37× (2.23–2.51) | 2.82× (2.70–2.96) | 3.08× (3.01–3.15) | 6.69× (6.42–6.98) | 6.74× (6.58–6.91) | 2.51× (2.46–2.57) | 2.92× (2.84–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.081 s | 118 | 39.1 | 1.53× (1.50–1.56) | 3.73× (3.56–3.91) | 3.09× (2.96–3.22) | 3.12× (2.99–3.25) | 3.54× (3.38–3.70) | 4.41× (4.24–4.59) | 4.76× (4.57–4.96) | 1.60× (1.53–1.67) | 1.84× (1.77–1.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.052 s | 184 | 45.5 | 1.30× (1.25–1.36) | 3.27× (3.17–3.38) | 2.78× (2.67–2.88) | 3.78× (3.66–3.91) | 3.29× (3.22–3.36) | 6.92× (6.73–7.11) | 7.47× (7.28–7.66) | 2.50× (2.43–2.57) | 2.89× (2.80–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 217 | 64.0 | 1.36× (1.32–1.41) | 2.66× (2.58–2.73) | 2.60× (2.51–2.68) | 3.39× (3.31–3.46) | 3.20× (3.15–3.25) | 8.34× (8.22–8.47) | 9.00× (8.88–9.13) | 3.02× (2.98–3.06) | 3.48× (3.43–3.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,124 | 4.6 | 0.63× (0.62–0.65) | 3.42× (3.37–3.48) | 4.93× (4.63–5.26) | 3.21× (3.15–3.26) | 7.28× (7.12–7.45) | 2.40× (2.36–2.45) | 2.43× (2.40–2.46) | 1.05× (0.93–1.19) | 2.29× (2.25–2.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,026 | 5.3 | 0.50× (0.48–0.51) | 3.15× (3.05–3.25) | 4.94× (4.61–5.30) | 2.97× (2.88–3.06) | 6.66× (6.46–6.86) | 2.21× (2.14–2.27) | 2.23× (2.16–2.31) | 0.97× (0.86–1.09) | 2.10× (2.03–2.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,031 | 5.1 | 0.51× (0.50–0.52) | 3.17× (3.13–3.22) | 4.95× (4.59–5.34) | 3.20× (2.82–3.63) | 6.83× (6.67–6.99) | 2.22× (2.18–2.27) | 2.25× (2.21–2.29) | 0.97× (0.86–1.11) | 2.11× (2.08–2.14) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 0.042 s | 315 | 14.9 | 1.77× (1.75–1.80) | 4.40× (4.28–4.52) | 4.60× (4.51–4.70) | 4.88× (4.79–4.97) | 5.11× (5.01–5.21) | 5.19× (5.10–5.29) | 5.06× (4.93–5.18) | 1.52× (1.49–1.54) | 1.83× (1.74–1.91) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 0.030 s | 448 | 19.0 | 1.47× (1.40–1.55) | 3.40× (3.23–3.56) | 4.14× (3.88–4.42) | 4.66× (4.38–4.96) | 4.54× (4.30–4.81) | 7.31× (6.99–7.64) | 7.12× (6.72–7.54) | 2.14× (2.04–2.24) | 2.57× (2.45–2.70) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 0.028 s | 483 | 29.1 | 1.32× (1.30–1.34) | 2.62× (2.55–2.68) | 3.23× (3.10–3.36) | 3.37× (3.29–3.46) | 4.04× (3.97–4.11) | 7.99× (7.71–8.29) | 7.79× (7.55–8.03) | 2.34× (2.28–2.39) | 2.81× (2.70–2.93) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.90–2.97) | 7.16× (7.12–7.20) | 7.46× (7.40–7.52) | 8.69× (8.45–8.93) | 7.59× (7.44–7.75) | 7.84× (7.76–7.92) | 7.48× (7.39–7.57) | 2.22× (2.19–2.24) | 2.55× (2.53–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.66× (2.54–2.79) | 6.52× (6.26–6.78) | 6.90× (6.67–7.14) | 7.97× (7.65–8.29) | 6.83× (6.65–7.02) | 6.99× (6.70–7.30) | 6.67× (6.37–6.98) | 1.98× (1.89–2.06) | 2.27× (2.17–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.71× (2.65–2.77) | 5.17× (5.02–5.32) | 5.62× (5.50–5.75) | 6.32× (6.16–6.49) | 5.53× (5.36–5.70) | 4.72× (4.58–4.87) | 4.51× (4.37–4.65) | 1.33× (1.29–1.38) | 1.54× (1.48–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.11× (3.05–3.17) | 3.96× (3.87–4.07) | 3.54× (3.46–3.61) | 6.17× (6.05–6.28) | 2.70× (2.64–2.76) | 2.69× (2.63–2.74) | 1.02× (0.99–1.04) | 1.90× (1.86–1.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.85× (2.81–2.90) | 5.11× (4.99–5.23) | 3.38× (3.32–3.45) | 5.78× (5.63–5.94) | 2.55× (2.49–2.61) | 2.54× (2.48–2.60) | 0.96× (0.93–0.99) | 1.79× (1.75–1.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.05× (2.98–3.11) | 5.03× (4.94–5.12) | 3.42× (3.33–3.51) | 5.87× (5.78–5.97) | 2.56× (2.52–2.61) | 2.55× (2.50–2.60) | 0.96× (0.93–1.00) | 1.80× (1.76–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.91× (2.89–2.94) | 3.26× (3.21–3.32) | 3.87× (3.78–3.96) | 4.50× (4.40–4.61) | 5.16× (5.12–5.21) | 2.99× (2.92–3.06) | 2.52× (2.50–2.55) | 2.18× (2.02–2.36) | 2.86× (2.81–2.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.42× (2.35–2.48) | 2.74× (2.64–2.85) | 3.25× (3.17–3.34) | 3.65× (3.57–3.74) | 4.16× (4.06–4.26) | 2.31× (2.25–2.38) | 1.95× (1.91–2.00) | 1.69× (1.59–1.80) | 2.21× (2.16–2.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.89× (1.82–1.96) | 1.74× (1.68–1.80) | 2.03× (1.96–2.11) | 2.11× (2.03–2.19) | 2.25× (2.17–2.35) | 1.11× (1.07–1.16) | 0.94× (0.90–0.97) | 0.81× (0.76–0.87) | 1.06× (1.02–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.36–1.44) | 3.06× (3.02–3.10) | 2.75× (2.70–2.79) | 2.77× (2.75–2.79) | 3.13× (3.05–3.20) | 3.53× (3.45–3.60) | 3.55× (3.52–3.58) | 1.33× (1.31–1.36) | 1.45× (1.43–1.48) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.16–1.31) | 2.70× (2.54–2.86) | 2.33× (2.20–2.47) | 3.02× (2.85–3.21) | 2.72× (2.51–2.94) | 2.99× (2.80–3.18) | 3.00× (2.84–3.18) | 1.13× (1.08–1.18) | 1.23× (1.16–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.39× (1.36–1.43) | 2.54× (2.46–2.62) | 2.15× (2.02–2.29) | 2.77× (2.65–2.89) | 2.61× (2.52–2.70) | 2.28× (2.20–2.37) | 2.30× (2.24–2.36) | 0.86× (0.84–0.89) | 0.94× (0.91–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.49–1.56) | 3.64× (3.47–3.81) | 2.99× (2.85–3.13) | 3.08× (2.95–3.23) | 3.49× (3.33–3.67) | 4.36× (4.17–4.55) | 4.70× (4.50–4.91) | 1.59× (1.52–1.66) | 1.75× (1.67–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.34–1.37) | 3.29× (3.27–3.32) | 2.63× (2.59–2.66) | 3.64× (3.59–3.69) | 3.03× (2.97–3.09) | 3.81× (3.76–3.87) | 4.11× (4.05–4.17) | 1.39× (1.37–1.41) | 1.53× (1.51–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.50× (1.44–1.57) | 2.98× (2.88–3.08) | 2.22× (2.17–2.28) | 3.13× (3.07–3.20) | 2.85× (2.76–2.94) | 2.73× (2.65–2.81) | 2.95× (2.86–3.03) | 0.99× (0.97–1.02) | 1.10× (1.07–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.71) | 2.63× (2.57–2.70) | 3.58× (3.22–3.98) | 2.93× (2.87–3.00) | 6.51× (6.35–6.69) | 2.26× (2.20–2.31) | 2.25× (2.22–2.29) | 1.05× (0.93–1.19) | 1.55× (1.52–1.57) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.30× (2.24–2.35) | 3.92× (3.81–4.04) | 2.71× (2.64–2.79) | 5.81× (5.65–5.98) | 2.01× (1.96–2.07) | 2.01× (1.95–2.07) | 0.94× (0.83–1.06) | 1.38× (1.34–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.58) | 2.53× (2.48–2.57) | 4.01× (3.93–4.10) | 3.02× (2.65–3.44) | 6.14× (6.00–6.29) | 2.09× (2.05–2.13) | 2.09× (2.04–2.13) | 0.97× (0.85–1.11) | 1.43× (1.40–1.46) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 1.75× (1.73–1.78) | 4.16× (4.08–4.25) | 4.43× (4.36–4.49) | 4.77× (4.68–4.86) | 5.00× (4.90–5.10) | 5.07× (4.98–5.17) | 4.94× (4.82–5.06) | 1.50× (1.48–1.53) | 1.67× (1.64–1.69) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 1.47× (1.44–1.51) | 3.37× (3.30–3.44) | 3.70× (3.60–3.81) | 4.26× (4.12–4.42) | 3.98× (3.88–4.09) | 4.00× (3.88–4.13) | 3.90× (3.77–4.03) | 1.19× (1.16–1.22) | 1.31× (1.29–1.34) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 1.48× (1.46–1.51) | 2.68× (2.62–2.75) | 2.91× (2.84–2.98) | 3.33× (3.24–3.42) | 3.14× (3.06–3.21) | 2.59× (2.50–2.68) | 2.52× (2.42–2.62) | 0.77× (0.75–0.79) | 0.85× (0.82–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.34× (0.33–0.35) | 0.97× (0.97–0.97) | 0.78× (0.76–0.80) | 10.96× (10.93–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.32–0.35) | 0.45× (0.38–0.54) | 0.69× (0.66–0.72) | 0.58× (0.56–0.60) | 5.48× (5.29–5.66) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.32–0.35) | 0.54× (0.45–0.66) | 0.60× (0.57–0.63) | 0.53× (0.51–0.55) | 3.48× (3.36–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.35–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.43–0.46) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.94–7.99) | 0.65× (0.64–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.71) | 0.62× (0.59–0.65) | 1.58× (1.53–1.64) | 0.79× (0.76–0.83) | 5.73× (5.54–5.94) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.62× (0.59–0.65) | 0.69× (0.65–0.74) | 1.12× (1.08–1.15) | 0.68× (0.66–0.70) | 3.25× (3.15–3.35) | 0.26× (0.25–0.26) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.14–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.40× (0.36–0.44) | 0.89× (0.85–0.94) | 0.70× (0.66–0.74) | 3.09× (2.94–3.24) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.81–0.85) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.17–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.40× (0.39–0.42) | 0.87× (0.84–0.90) | 0.73× (0.69–0.77) | 2.99× (2.89–3.10) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.48× (0.45–0.51) | 0.80× (0.79–0.80) | 0.81× (0.78–0.84) | 2.31× (2.29–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.06–1.07) | 0.77× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.24–5.27) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.27× (5.26–5.28) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.38–1.40) | 0.67× (0.66–0.67) | 6.32× (6.30–6.35) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.62× (0.61–0.62) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.36× (0.34–0.39) | 1.04× (1.00–1.08) | 0.68× (0.65–0.70) | 3.31× (3.24–3.39) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.44× (0.40–0.49) | 0.91× (0.89–0.93) | 0.74× (0.72–0.76) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 728 | 1.38 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,223 | 1.52 | 1.12× (1.07–1.17) | 82% | 1.02× (1.01–1.03) | 1.02× (1.00–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,452 | 2.28 | 1.66× (1.61–1.71) | 50% | 1.20× (1.18–1.22) | 1.53× (1.49–1.58) |

**Best:** mibz2 at 4 threads (1,452 MB/s); lbzip2 at 4 threads (328 MB/s); pbz2 at 4 threads (525 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.93, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.50, pbz2 4t 0.62. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.35 | 0.01 | 0.07 | 0.144 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4556 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 36.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.6% |
| MTF + RLE2 | 1.46 | 20.6% |
| T-vector build | 1.31 | 18.6% |
| walk (the one each build uses) | 2.86 | 40.4% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.26 | 3.6% |
| **total** | 7.07 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.281 | 0.681 | (in its total) | 2.42× (2.41–2.44) |  |
| walk | 0.603 | 1.534 | (in its total) | 2.55× (2.53–2.57) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.028 | (in its total) | 1.14× (1.13–1.15) |  |
| inverse BWT, total | 0.909 | 2.243 | 1.231 | 2.47× (2.46–2.49) | 1.35× (1.35–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 42m07s ± 4m51s |
| 4 threads | 21m03s ± 2m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 12m48s ± 1m52s |
| the scanner alone (its serial part) | 7m34s ± 26.3s |
| the workers alone, over 4 | 12m48s ± 1m52s |

At 4 threads the workers bound it. Per byte: the scanner 4.4 ns a byte of bz2, the workers 1.68 ns a byte of output at 4 threads (CPU, summed over them), 1.13 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,246 | 13,345,437 | \`3e431388ee58319e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4555.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
