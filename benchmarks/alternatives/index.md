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

The latest record: [`4ca8523`](<https://github.com/lmmx/mibz2/commit/4ca8523522b87af654ef55a7c84d80c4b9b196bb>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43527993957>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.293 s | 716 | 6.3 | 2.89× (2.82–2.97) | 7.20× (7.03–7.37) | 7.45× (7.32–7.59) | 8.54× (8.31–8.78) | 7.57× (7.36–7.79) | 7.83× (7.59–8.08) | 7.52× (7.36–7.69) | 2.23× (2.17–2.29) | 2.58× (2.52–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.179 s | 1,171 | 7.1 | 2.81× (2.65–2.97) | 6.07× (5.76–6.39) | 7.29× (6.77–7.85) | 7.36× (6.81–7.96) | 6.71× (6.28–7.17) | 12.66× (11.84–13.53) | 12.16× (11.34–13.04) | 3.60× (3.37–3.84) | 4.17× (3.89–4.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.164 s | 1,279 | 11.8 | 2.45× (2.38–2.53) | 3.99× (3.89–4.08) | 5.09× (4.89–5.30) | 4.75× (4.66–4.85) | 4.83× (4.74–4.93) | 13.86× (13.42–14.32) | 13.32× (12.99–13.65) | 3.94× (3.83–4.06) | 4.57× (4.42–4.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,465 | 3.4 | 0.57× (0.56–0.58) | 3.68× (3.56–3.81) | 4.62× (4.51–4.74) | 3.75× (3.67–3.84) | 6.62× (6.40–6.86) | 2.85× (2.79–2.91) | 2.84× (2.78–2.90) | 1.04× (0.92–1.17) | 2.22× (2.14–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,434 | 3.5 | 0.54× (0.51–0.56) | 3.10× (2.84–3.40) | 6.01× (5.92–6.11) | 3.70× (3.60–3.81) | 6.42× (6.24–6.61) | 2.80× (2.74–2.87) | 2.79× (2.74–2.85) | 1.02× (0.90–1.17) | 2.18× (2.12–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,418 | 3.5 | 0.51× (0.49–0.54) | 2.97× (2.76–3.21) | 5.88× (5.59–6.19) | 3.63× (3.47–3.81) | 6.38× (5.98–6.80) | 2.73× (2.62–2.85) | 2.73× (2.62–2.84) | 1.00× (0.87–1.14) | 2.13× (2.02–2.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,107 | 4.4 | 3.02× (2.94–3.10) | 3.52× (3.37–3.68) | 4.31× (4.16–4.47) | 4.76× (4.58–4.95) | 6.13× (4.92–7.65) | 3.39× (2.87–4.00) | 2.70× (2.58–2.81) | 2.37× (1.95–2.89) | 3.41× (3.34–3.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,479 | 6.0 | 2.13× (1.93–2.36) | 2.85× (2.47–3.28) | 3.75× (2.86–4.91) | 3.44× (2.65–4.46) | 5.03× (3.92–6.44) | 3.97× (3.49–4.52) | 3.16× (2.46–4.05) | 2.78× (1.99–3.88) | 3.99× (3.11–5.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,192 | 11.5 | 1.63× (1.50–1.77) | 1.86× (1.72–2.01) | 2.77× (2.54–3.02) | 2.04× (1.85–2.24) | 4.19× (3.92–4.48) | 3.60× (3.13–4.15) | 2.87× (2.66–3.09) | 2.53× (2.08–3.06) | 3.62× (3.35–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.080 s | 100 | 45.9 | 1.38× (1.33–1.43) | 3.15× (2.98–3.32) | 2.87× (2.78–2.96) | 2.72× (2.64–2.79) | 3.13× (3.02–3.24) | 3.49× (3.40–3.59) | 3.57× (3.45–3.69) | 1.33× (1.29–1.37) | 1.57× (1.50–1.64) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 52.0 | 1.24× (1.15–1.33) | 2.86× (2.64–3.09) | 2.64× (2.44–2.87) | 3.20× (2.97–3.44) | 3.06× (2.93–3.20) | 5.33× (5.00–5.69) | 5.45× (5.09–5.83) | 2.03× (1.90–2.18) | 2.39× (2.20–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 195 | 68.2 | 1.28× (1.22–1.35) | 2.50× (2.42–2.59) | 2.34× (2.24–2.45) | 2.80× (2.67–2.95) | 3.17× (2.96–3.40) | 6.67× (6.40–6.95) | 6.81× (6.54–7.09) | 2.54× (2.43–2.66) | 3.00× (2.82–3.18) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.082 s | 118 | 39.0 | 1.53× (1.50–1.56) | 3.78× (3.69–3.88) | 3.15× (3.03–3.26) | 3.14× (3.07–3.21) | 3.65× (3.53–3.78) | 4.63× (4.47–4.79) | 4.88× (4.76–5.01) | 1.67× (1.58–1.76) | 1.87× (1.82–1.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 176 | 47.1 | 1.26× (1.17–1.36) | 3.19× (2.99–3.40) | 2.75× (2.57–2.94) | 3.60× (3.39–3.83) | 3.26× (3.05–3.48) | 6.79× (6.27–7.34) | 7.16× (6.69–7.67) | 2.45× (2.27–2.64) | 2.74× (2.57–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 217 | 64.7 | 1.38× (1.36–1.40) | 2.69× (2.60–2.79) | 2.56× (2.49–2.63) | 3.41× (3.31–3.51) | 3.17× (3.12–3.23) | 8.44× (8.15–8.75) | 8.91× (8.67–9.15) | 3.05× (2.89–3.20) | 3.41× (3.31–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,111 | 4.6 | 0.64× (0.63–0.66) | 3.36× (3.24–3.48) | 4.93× (4.47–5.44) | 3.19× (3.11–3.26) | 7.50× (6.72–8.37) | 2.40× (2.34–2.46) | 2.43× (2.38–2.49) | 0.99× (0.96–1.01) | 2.24× (2.17–2.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,026 | 5.3 | 0.50× (0.47–0.53) | 3.15× (2.85–3.48) | 4.86× (4.38–5.39) | 3.03× (2.87–3.19) | 6.57× (6.21–6.95) | 2.18× (2.06–2.30) | 2.21× (2.10–2.32) | 0.90× (0.86–0.93) | 2.04× (1.93–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,010 | 5.1 | 0.50× (0.48–0.52) | 3.06× (2.96–3.17) | 5.29× (4.62–6.05) | 3.11× (2.92–3.32) | 6.70× (6.58–6.83) | 2.18× (2.11–2.25) | 2.21× (2.14–2.27) | 0.90× (0.86–0.93) | 2.03× (1.97–2.10) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.043 s | 304 | 15.4 | 1.77× (1.71–1.83) | 4.30× (4.20–4.41) | 4.64× (4.48–4.81) | 5.01× (4.80–5.22) | 5.37× (5.09–5.66) | 5.16× (5.03–5.29) | 5.04× (4.84–5.24) | 1.55× (1.50–1.61) | 1.78× (1.69–1.88) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.034 s | 389 | 20.1 | 1.26× (1.12–1.40) | 3.05× (2.72–3.41) | 3.55× (3.16–3.98) | 3.89× (3.49–4.33) | 4.06× (3.63–4.54) | 6.39× (5.72–7.15) | 6.24× (5.55–7.03) | 1.92× (1.74–2.13) | 2.21× (1.96–2.48) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.029 s | 441 | 30.0 | 1.24× (1.20–1.28) | 2.33× (2.21–2.45) | 3.03× (2.93–3.13) | 3.11× (2.99–3.24) | 3.76× (3.66–3.85) | 7.35× (7.01–7.70) | 7.18× (6.82–7.55) | 2.21× (2.09–2.34) | 2.54× (2.39–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.81–2.96) | 7.15× (6.99–7.32) | 7.40× (7.26–7.54) | 8.50× (8.28–8.74) | 7.54× (7.32–7.76) | 7.79× (7.55–8.05) | 7.49× (7.33–7.65) | 2.22× (2.16–2.28) | 2.55× (2.49–2.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.65× (2.58–2.72) | 6.47× (6.33–6.62) | 6.81× (6.51–7.12) | 7.86× (7.57–8.16) | 6.83× (6.62–7.05) | 6.86× (6.59–7.15) | 6.59× (6.35–6.85) | 1.96× (1.89–2.03) | 2.25× (2.16–2.34) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.47× (2.39–2.54) | 4.63× (4.45–4.82) | 5.01× (4.86–5.16) | 5.54× (5.29–5.81) | 4.91× (4.71–5.13) | 4.15× (3.91–4.40) | 3.99× (3.80–4.19) | 1.18× (1.12–1.25) | 1.36× (1.29–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.63) | 3.12× (3.03–3.21) | 3.96× (3.83–4.10) | 3.51× (3.41–3.62) | 6.13× (5.89–6.38) | 2.70× (2.62–2.77) | 2.68× (2.61–2.76) | 1.05× (0.93–1.18) | 1.88× (1.82–1.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.58–0.63) | 3.01× (2.77–3.27) | 5.09× (4.96–5.23) | 3.43× (3.35–3.51) | 5.84× (5.70–5.99) | 2.60× (2.54–2.66) | 2.59× (2.53–2.65) | 1.01× (0.89–1.15) | 1.81× (1.77–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.53–0.58) | 3.08× (2.94–3.24) | 5.02× (4.75–5.30) | 3.42× (3.27–3.57) | 5.87× (5.52–6.24) | 2.58× (2.49–2.68) | 2.57× (2.47–2.66) | 1.00× (0.88–1.14) | 1.80× (1.70–1.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.87× (2.79–2.96) | 3.20× (3.14–3.27) | 3.83× (3.76–3.91) | 4.49× (4.29–4.69) | 5.76× (4.60–7.21) | 3.23× (2.72–3.83) | 2.57× (2.46–2.69) | 2.28× (1.86–2.79) | 2.83× (2.77–2.89) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.23–2.29) | 2.55× (2.31–2.81) | 2.86× (2.30–3.54) | 3.20× (2.62–3.92) | 3.68× (3.01–4.51) | 2.17× (2.05–2.30) | 1.73× (1.41–2.12) | 1.54× (1.14–2.06) | 1.90× (1.54–2.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.93× (1.84–2.02) | 1.76× (1.67–1.85) | 2.08× (1.97–2.20) | 2.10× (2.01–2.20) | 2.38× (2.25–2.51) | 1.22× (1.03–1.44) | 0.97× (0.91–1.04) | 0.86× (0.69–1.07) | 1.07× (1.01–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.32–1.43) | 3.05× (2.89–3.21) | 2.77× (2.69–2.86) | 2.68× (2.60–2.76) | 3.08× (2.97–3.20) | 3.44× (3.33–3.55) | 3.52× (3.39–3.65) | 1.32× (1.28–1.37) | 1.48× (1.42–1.55) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.16–1.32) | 2.74× (2.55–2.95) | 2.35× (2.21–2.50) | 3.07× (2.85–3.30) | 2.71× (2.58–2.84) | 2.94× (2.75–3.13) | 3.00× (2.81–3.21) | 1.13× (1.05–1.21) | 1.27× (1.17–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.39× (1.33–1.46) | 2.57× (2.51–2.64) | 2.10× (2.05–2.16) | 2.79× (2.69–2.90) | 2.68× (2.49–2.88) | 2.23× (2.15–2.32) | 2.28× (2.20–2.37) | 0.86× (0.82–0.89) | 0.96× (0.92–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.49–1.56) | 3.68× (3.60–3.77) | 3.04× (2.94–3.14) | 3.11× (3.04–3.18) | 3.61× (3.49–3.74) | 4.57× (4.41–4.73) | 4.82× (4.69–4.95) | 1.66× (1.57–1.76) | 1.80× (1.76–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.30–1.37) | 3.28× (3.22–3.34) | 2.62× (2.55–2.70) | 3.53× (3.44–3.62) | 3.04× (2.95–3.13) | 3.78× (3.62–3.95) | 3.99× (3.86–4.13) | 1.38× (1.30–1.46) | 1.49× (1.45–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.49–1.58) | 2.94× (2.86–3.03) | 2.25× (2.16–2.34) | 3.12× (3.02–3.23) | 2.83× (2.77–2.89) | 2.76× (2.62–2.90) | 2.91× (2.81–3.02) | 1.00× (0.95–1.06) | 1.09× (1.05–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.72) | 2.61× (2.52–2.71) | 3.63× (3.27–4.03) | 2.88× (2.77–3.00) | 6.62× (5.86–7.47) | 2.22× (2.13–2.32) | 2.25× (2.15–2.35) | 0.98× (0.95–1.02) | 1.57× (1.47–1.67) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.54–0.61) | 2.42× (2.05–2.85) | 3.85× (3.64–4.08) | 2.75× (2.61–2.89) | 5.69× (5.39–6.00) | 1.97× (1.86–2.08) | 1.99× (1.89–2.10) | 0.87× (0.83–0.91) | 1.39× (1.29–1.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.59) | 2.52× (2.40–2.65) | 4.25× (3.72–4.85) | 2.91× (2.74–3.10) | 6.01× (5.89–6.14) | 2.04× (1.97–2.11) | 2.06× (1.98–2.15) | 0.90× (0.86–0.94) | 1.44× (1.35–1.53) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.75× (1.69–1.80) | 4.08× (4.00–4.16) | 4.46× (4.32–4.61) | 4.88× (4.69–5.09) | 5.24× (4.97–5.52) | 5.03× (4.90–5.16) | 4.91× (4.71–5.13) | 1.54× (1.48–1.60) | 1.66× (1.61–1.71) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.38× (1.32–1.45) | 3.30× (3.21–3.39) | 3.53× (3.38–3.70) | 4.08× (3.87–4.30) | 3.83× (3.67–4.00) | 3.78× (3.57–4.01) | 3.70× (3.43–3.98) | 1.16× (1.08–1.24) | 1.25× (1.16–1.35) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.44× (1.41–1.48) | 2.63× (2.57–2.69) | 2.87× (2.79–2.95) | 3.29× (3.16–3.43) | 3.17× (3.11–3.23) | 2.53× (2.41–2.65) | 2.47× (2.34–2.61) | 0.78× (0.73–0.82) | 0.83× (0.79–0.89) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.38) | 1.06× (1.05–1.06) | 0.85× (0.84–0.86) | 11.92× (11.90–11.95) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.50× (0.44–0.57) | 0.74× (0.72–0.76) | 0.63× (0.62–0.64) | 5.97× (5.86–6.08) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.38) | 0.65× (0.52–0.81) | 0.66× (0.66–0.67) | 0.58× (0.57–0.60) | 3.90× (3.86–3.93) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.46) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.01–8.08) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.62–0.70) | 0.63× (0.56–0.71) | 1.57× (1.50–1.65) | 0.79× (0.75–0.83) | 5.71× (5.44–5.99) | 0.46× (0.44–0.48) | 0.48× (0.46–0.51) | 0.53× (0.50–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.59× (0.56–0.63) | 0.64× (0.60–0.68) | 1.07× (1.01–1.14) | 0.66× (0.63–0.70) | 3.13× (2.96–3.31) | 0.24× (0.23–0.26) | 0.26× (0.24–0.27) | 0.28× (0.26–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.53) | 0.41× (0.37–0.45) | 0.94× (0.90–0.98) | 0.76× (0.73–0.79) | 3.24× (3.12–3.37) | 0.20× (0.19–0.21) | 0.22× (0.21–0.22) | 0.33× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.89× (0.87–0.90) | 0.84× (0.82–0.86) | 2.58× (2.53–2.64) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.40) | 0.84× (0.81–0.87) | 0.72× (0.70–0.75) | 2.90× (2.80–3.00) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.46× (0.44–0.49) | 0.81× (0.80–0.82) | 0.81× (0.80–0.83) | 2.33× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.78) | 5.30× (5.28–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.78) | 5.33× (5.31–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.65–0.66) | 6.40× (6.38–6.41) | 0.39× (0.38–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.44× (0.40–0.48) | 1.13× (1.10–1.16) | 0.73× (0.69–0.76) | 3.59× (3.50–3.69) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.47× (0.43–0.52) | 0.94× (0.91–0.97) | 0.78× (0.75–0.80) | 2.40× (2.33–2.46) | 0.14× (0.13–0.14) | 0.15× (0.14–0.16) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 716 | 1.40 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,171 | 1.57 | 1.14× (1.11–1.16) | 81% | 1.03× (1.00–1.05) | 1.04× (1.00–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,279 | 2.61 | 1.88× (1.81–1.95) | 44% | 1.22× (1.18–1.26) | 1.61× (1.53–1.68) |

**Best:** mibz2 at 4 threads (1,279 MB/s); lbzip2 at 4 threads (319 MB/s); pbz2 at 4 threads (526 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.93, mibz2 4t 1.20, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.47, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.58. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 16%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.39 | 0.01 | 0.09 | 0.164 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4518 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 36.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.6% |
| MTF + RLE2 | 1.50 | 21.3% |
| T-vector build | 1.30 | 18.4% |
| walk (the one each build uses) | 2.84 | 40.2% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.26 | 3.6% |
| **total** | 7.06 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.285 | 0.686 | (in its total) | 2.37× (2.26–2.49) |  |
| walk | 0.589 | 1.542 | (in its total) | 2.58× (2.47–2.70) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.024 | 0.028 | (in its total) | 1.14× (1.09–1.20) |  |
| inverse BWT, total | 0.898 | 2.257 | 1.267 | 2.48× (2.36–2.59) | 1.41× (1.33–1.50) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m06s ± 5m05s |
| 4 threads | 24m21s ± 3m31s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m42s ± 2m23s |
| the scanner alone (its serial part) | 8m44s ± 41.6s |
| the workers alone, over 4 | 14m42s ± 2m23s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 1.93 ns a byte of output at 4 threads (CPU, summed over them), 1.13 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4518.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
