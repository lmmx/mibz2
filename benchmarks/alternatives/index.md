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

The latest record: [`26cc7cc`](<https://github.com/lmmx/mibz2/commit/26cc7cc6c6bea128896aff5c9a2de8711a8ff9da>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45955533039>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.385 s | 545 | 6.8 | 3.13× (3.03–3.23) | 7.15× (6.92–7.39) | 7.43× (7.20–7.66) | 8.45× (8.19–8.71) | 7.44× (7.20–7.68) | 7.77× (7.52–8.03) | 7.54× (7.29–7.80) | 2.33× (2.27–2.40) | 2.59× (2.51–2.67) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.215 s | 974 | 7.3 | 3.44× (3.32–3.56) | 6.83× (6.68–6.99) | 8.06× (7.86–8.26) | 8.24× (8.08–8.42) | 7.30× (7.14–7.46) | 14.06× (13.81–14.32) | 13.65× (13.41–13.89) | 4.22× (4.12–4.31) | 4.68× (4.58–4.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.182 s | 1,154 | 11.6 | 3.27× (3.19–3.35) | 4.80× (4.68–4.91) | 6.01× (5.81–6.21) | 5.80× (5.69–5.91) | 5.83× (5.67–5.98) | 16.64× (16.33–16.97) | 16.15× (15.87–16.44) | 4.99× (4.88–5.10) | 5.54× (5.42–5.66) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,099 | 3.7 | 0.56× (0.56–0.57) | 3.37× (3.33–3.40) | 4.52× (4.43–4.61) | 3.51× (3.47–3.55) | 6.06× (5.99–6.13) | 2.62× (2.58–2.65) | 2.61× (2.58–2.64) | 0.95× (0.94–0.97) | 1.94× (1.75–2.16) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,067 | 3.9 | 0.52× (0.49–0.56) | 2.71× (2.62–2.80) | 5.52× (5.23–5.83) | 3.44× (3.36–3.51) | 5.74× (5.58–5.91) | 2.52× (2.43–2.61) | 2.51× (2.43–2.60) | 0.91× (0.88–0.95) | 1.87× (1.68–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,053 | 3.9 | 0.51× (0.50–0.52) | 2.70× (2.65–2.75) | 5.38× (5.29–5.47) | 3.38× (3.32–3.44) | 5.77× (5.62–5.92) | 2.50× (2.46–2.55) | 2.50× (2.45–2.54) | 0.91× (0.89–0.92) | 1.86× (1.66–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 761 | 5.2 | 2.80× (2.74–2.86) | 3.17× (3.12–3.22) | 4.00× (3.90–4.10) | 4.23× (4.16–4.29) | 4.64× (4.56–4.72) | 2.88× (2.84–2.93) | 2.39× (2.27–2.52) | 2.16× (2.12–2.19) | 3.16× (3.11–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,225 | 6.1 | 2.25× (1.89–2.68) | 2.86× (2.39–3.44) | 4.50× (3.66–5.54) | 3.61× (3.07–4.25) | 5.04× (4.25–5.98) | 4.06× (3.41–4.85) | 3.37× (2.94–3.87) | 3.04× (2.56–3.61) | 4.46× (3.76–5.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,045 | 12.2 | 2.04× (1.98–2.11) | 2.13× (1.95–2.31) | 3.37× (3.17–3.58) | 2.34× (2.26–2.41) | 4.51× (4.41–4.60) | 4.01× (3.90–4.12) | 3.33× (3.17–3.50) | 3.00× (2.92–3.08) | 4.40× (4.28–4.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 79 | 47.6 | 1.50× (1.49–1.51) | 2.87× (2.83–2.91) | 2.59× (2.57–2.62) | 2.71× (2.68–2.73) | 3.04× (3.02–3.05) | 3.23× (3.21–3.26) | 3.42× (3.40–3.45) | 1.36× (1.35–1.37) | 1.52× (1.49–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 129 | 53.7 | 1.39× (1.30–1.49) | 2.69× (2.48–2.92) | 2.55× (2.41–2.70) | 3.31× (3.10–3.52) | 3.08× (2.90–3.28) | 5.19× (4.90–5.50) | 5.50× (5.18–5.84) | 2.18× (2.05–2.32) | 2.44× (2.32–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 164 | 68.8 | 1.51× (1.47–1.56) | 2.39× (2.29–2.49) | 2.39× (2.32–2.46) | 3.01× (2.92–3.11) | 3.31× (3.20–3.41) | 6.66× (6.49–6.83) | 7.05× (6.87–7.24) | 2.80× (2.73–2.87) | 3.14× (3.06–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 87 | 43.2 | 1.62× (1.61–1.64) | 3.28× (3.23–3.32) | 2.84× (2.80–2.89) | 2.97× (2.93–3.00) | 3.35× (3.29–3.41) | 3.95× (3.90–3.99) | 4.38× (4.33–4.43) | 1.62× (1.59–1.65) | 1.92× (1.90–1.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 144 | 48.8 | 1.53× (1.32–1.77) | 2.98× (2.82–3.14) | 2.70× (2.59–2.81) | 3.75× (3.58–3.94) | 3.30× (3.16–3.45) | 6.38× (6.13–6.65) | 7.08× (6.79–7.38) | 2.62× (2.50–2.74) | 3.10× (2.97–3.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 174 | 65.8 | 1.61× (1.54–1.68) | 2.58× (2.50–2.67) | 2.51× (2.42–2.60) | 3.47× (3.35–3.59) | 3.22× (3.12–3.33) | 7.87× (7.63–8.12) | 8.73× (8.45–9.01) | 3.23× (3.12–3.34) | 3.82× (3.70–3.94) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 5.0 | 0.63× (0.62–0.64) | 3.47× (3.43–3.51) | 4.34× (4.30–4.39) | 3.10× (3.06–3.13) | 6.87× (6.63–7.12) | 2.28× (2.26–2.30) | 2.33× (2.31–2.35) | 0.95× (0.94–0.95) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 784 | 5.6 | 0.50× (0.49–0.51) | 2.42× (2.39–2.45) | 4.86× (4.80–4.92) | 2.89× (2.85–2.93) | 6.36× (6.05–6.69) | 2.12× (2.09–2.14) | 2.16× (2.14–2.19) | 0.88× (0.87–0.89) | 1.62× (1.60–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 752 | 5.6 | 0.47× (0.43–0.52) | 2.32× (2.05–2.63) | 4.52× (4.11–4.97) | 2.70× (2.45–2.98) | 5.99× (5.33–6.74) | 1.97× (1.79–2.16) | 2.01× (1.83–2.22) | 0.82× (0.74–0.90) | 1.50× (1.37–1.65) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 0.062 s | 292 | 13.0 | 2.09× (2.06–2.11) | 4.28× (4.24–4.33) | 4.63× (4.56–4.70) | 5.04× (4.96–5.11) | 5.06× (4.99–5.12) | 4.96× (4.92–5.01) | 5.11× (5.06–5.16) | 1.69× (1.67–1.70) | 1.97× (1.95–1.99) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 0.042 s | 432 | 16.5 | 1.61× (1.47–1.77) | 3.29× (3.00–3.61) | 4.10× (3.73–4.50) | 4.49× (4.11–4.90) | 4.40× (4.04–4.79) | 7.00× (6.40–7.66) | 7.21× (6.59–7.88) | 2.38× (2.17–2.60) | 2.78× (2.54–3.04) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 0.037 s | 487 | 25.9 | 1.68× (1.61–1.75) | 2.64× (2.45–2.84) | 3.51× (3.38–3.65) | 3.70× (3.56–3.84) | 4.03× (3.97–4.08) | 8.29× (8.16–8.42) | 8.53× (8.40–8.65) | 2.81× (2.77–2.86) | 3.29× (3.24–3.34) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.12× (3.02–3.22) | 7.11× (6.88–7.34) | 7.39× (7.16–7.62) | 8.41× (8.16–8.67) | 7.41× (7.17–7.65) | 7.74× (7.49–8.00) | 7.51× (7.27–7.76) | 2.32× (2.26–2.39) | 2.57× (2.49–2.65) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.11× (3.04–3.18) | 6.87× (6.83–6.92) | 7.29× (7.20–7.39) | 8.41× (8.35–8.46) | 7.07× (7.00–7.13) | 7.25× (7.18–7.32) | 7.03× (6.98–7.09) | 2.18× (2.15–2.20) | 2.40× (2.38–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.13× (3.05–3.21) | 5.09× (5.04–5.14) | 5.65× (5.42–5.88) | 6.20× (6.13–6.27) | 5.51× (5.33–5.68) | 4.58× (4.51–4.65) | 4.44× (4.39–4.49) | 1.37× (1.35–1.39) | 1.52× (1.49–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.88× (2.84–2.92) | 3.86× (3.77–3.96) | 3.29× (3.25–3.33) | 5.61× (5.54–5.68) | 2.47× (2.44–2.51) | 2.47× (2.43–2.50) | 0.95× (0.94–0.97) | 1.62× (1.60–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.53–0.59) | 2.57× (2.49–2.66) | 4.72× (4.47–4.97) | 3.14× (3.08–3.20) | 5.16× (5.03–5.30) | 2.30× (2.23–2.38) | 2.30× (2.23–2.38) | 0.89× (0.86–0.92) | 1.51× (1.46–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 2.86× (2.79–2.92) | 4.72× (4.66–4.78) | 3.15× (3.09–3.20) | 5.26× (5.13–5.39) | 2.33× (2.29–2.37) | 2.33× (2.29–2.37) | 0.90× (0.88–0.92) | 1.53× (1.50–1.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.63–2.74) | 2.92× (2.88–2.96) | 3.54× (3.41–3.68) | 4.02× (3.97–4.08) | 4.41× (4.33–4.48) | 2.76× (2.72–2.80) | 2.30× (2.19–2.42) | 2.08× (2.05–2.12) | 2.72× (2.69–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (1.97–2.61) | 2.49× (2.14–2.90) | 3.45× (2.93–4.05) | 3.32× (2.88–3.82) | 3.62× (3.14–4.17) | 2.16× (1.86–2.50) | 1.80× (1.63–1.99) | 1.63× (1.41–1.88) | 2.12× (1.84–2.45) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.08× (1.97–2.20) | 1.86× (1.76–1.97) | 2.36× (2.29–2.44) | 2.18× (2.12–2.24) | 2.25× (2.18–2.33) | 1.16× (1.11–1.21) | 0.97× (0.92–1.03) | 0.88× (0.85–0.90) | 1.14× (1.11–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 2.79× (2.77–2.81) | 2.54× (2.52–2.55) | 2.68× (2.66–2.70) | 3.00× (2.99–3.02) | 3.20× (3.17–3.22) | 3.38× (3.36–3.41) | 1.35× (1.34–1.37) | 1.49× (1.48–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.36–1.41) | 2.64× (2.57–2.70) | 2.33× (2.28–2.37) | 3.15× (3.08–3.21) | 2.71× (2.67–2.77) | 2.84× (2.79–2.88) | 3.00× (2.95–3.06) | 1.20× (1.18–1.23) | 1.32× (1.29–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.66× (1.63–1.68) | 2.55× (2.50–2.60) | 2.17× (2.13–2.21) | 2.94× (2.90–2.98) | 2.79× (2.75–2.82) | 2.22× (2.19–2.25) | 2.35× (2.33–2.37) | 0.94× (0.93–0.95) | 1.03× (1.02–1.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.61× (1.60–1.63) | 3.21× (3.17–3.25) | 2.77× (2.73–2.82) | 2.94× (2.90–2.97) | 3.31× (3.26–3.37) | 3.90× (3.86–3.95) | 4.33× (4.28–4.38) | 1.61× (1.58–1.65) | 1.85× (1.84–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.38–1.53) | 2.90× (2.76–3.05) | 2.47× (2.35–2.60) | 3.47× (3.30–3.64) | 2.92× (2.78–3.07) | 3.37× (3.21–3.55) | 3.74× (3.55–3.94) | 1.39× (1.32–1.47) | 1.60× (1.52–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.82× (1.78–1.85) | 2.83× (2.77–2.89) | 2.24× (2.19–2.28) | 3.23× (3.18–3.27) | 2.90× (2.83–2.96) | 2.57× (2.53–2.61) | 2.85× (2.80–2.89) | 1.06× (1.04–1.08) | 1.22× (1.20–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.46× (2.44–2.48) | 3.40× (3.37–3.42) | 2.83× (2.80–2.85) | 6.13× (5.90–6.37) | 2.11× (2.10–2.13) | 2.16× (2.14–2.18) | 0.96× (0.95–0.96) | 1.36× (1.35–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.56) | 2.13× (2.11–2.16) | 3.77× (3.72–3.81) | 2.60× (2.57–2.62) | 5.46× (5.20–5.73) | 1.88× (1.86–1.90) | 1.92× (1.90–1.94) | 0.85× (0.84–0.86) | 1.21× (1.20–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.51× (0.47–0.55) | 2.26× (2.04–2.49) | 3.58× (3.29–3.89) | 2.51× (2.30–2.74) | 5.32× (4.78–5.92) | 1.81× (1.66–1.97) | 1.85× (1.69–2.02) | 0.82× (0.75–0.89) | 1.17× (1.07–1.28) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 2.05× (2.03–2.07) | 4.07× (4.03–4.11) | 4.49× (4.45–4.54) | 4.92× (4.85–4.99) | 4.94× (4.88–5.01) | 4.85× (4.81–4.90) | 4.99× (4.94–5.04) | 1.67× (1.65–1.68) | 1.86× (1.84–1.88) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 1.70× (1.67–1.73) | 3.38× (3.31–3.45) | 3.74× (3.67–3.80) | 4.41× (4.34–4.47) | 3.99× (3.93–4.05) | 3.83× (3.77–3.89) | 3.94× (3.87–4.01) | 1.31× (1.29–1.34) | 1.47× (1.45–1.49) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 1.76× (1.69–1.83) | 2.68× (2.59–2.77) | 3.00× (2.89–3.11) | 3.40× (3.32–3.48) | 3.15× (3.06–3.24) | 2.46× (2.41–2.51) | 2.53× (2.47–2.59) | 0.84× (0.83–0.86) | 0.94× (0.92–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.33× (0.32–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.06× (11.04–11.08) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.45× (0.39–0.53) | 0.72× (0.70–0.75) | 0.61× (0.59–0.62) | 5.74× (5.58–5.91) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.35–0.39) | 0.60× (0.51–0.70) | 0.65× (0.62–0.70) | 0.57× (0.53–0.61) | 3.81× (3.61–4.02) | 0.09× (0.08–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.65–0.66) | 8.00× (7.98–8.02) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.68–0.74) | 0.69× (0.63–0.76) | 1.68× (1.62–1.74) | 0.84× (0.81–0.88) | 6.08× (5.87–6.30) | 0.50× (0.48–0.51) | 0.52× (0.50–0.54) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.69× (0.66–0.71) | 0.79× (0.75–0.83) | 1.25× (1.22–1.28) | 0.77× (0.74–0.79) | 3.65× (3.56–3.75) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.74) | 5.18× (5.16–5.20) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.51) | 0.41× (0.39–0.43) | 0.89× (0.87–0.92) | 0.71× (0.67–0.75) | 3.09× (2.99–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.80–0.84) | 2.55× (2.51–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.20× (5.19–5.21) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.44–0.50) | 0.35× (0.32–0.38) | 0.83× (0.79–0.87) | 0.70× (0.66–0.74) | 2.86× (2.72–3.00) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.82× (0.81–0.84) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.23–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.30) | 0.24× (0.24–0.24) | 1.87× (1.86–1.88) | 0.90× (0.89–0.91) | 6.19× (6.16–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.69× (0.68–0.69) | 6.56× (6.53–6.58) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.64) | 0.31× (0.30–0.31) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 38 MB | 0.44× (0.43–0.46) | 0.45× (0.39–0.51) | 1.02× (0.97–1.07) | 0.66× (0.64–0.68) | 3.35× (3.23–3.47) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.16× (0.15–0.16) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 57 MB | 0.48× (0.47–0.49) | 0.44× (0.41–0.47) | 0.88× (0.86–0.89) | 0.74× (0.73–0.76) | 2.32× (2.28–2.37) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 545 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 974 | 1.99 | 1.07× (1.03–1.11) | 90% | 1.03× (1.02–1.04) | 1.06× (1.03–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 1,154 | 3.15 | 1.69× (1.63–1.76) | 54% | 1.21× (1.19–1.24) | 1.70× (1.64–1.75) |

**Best:** mibz2 at 4 threads (1,154 MB/s); lbzip2 at 4 threads (243 MB/s); pbz2 at 4 threads (356 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.06, lbzip2 1t 0.07, lbzip2 2t 0.18, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.52 | 0.01 | 0.08 | 0.182 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3686 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 54.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.84 | 9.8% |
| MTF + RLE2 | 1.47 | 17.0% |
| T-vector build | 1.53 | 17.8% |
| walk (the one each build uses) | 4.32 | 50.1% |
| of which: segment assembly | 0.18 | 2.1% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.61 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.394 | 1.066 | (in its total) | 2.70× (2.69–2.71) |  |
| walk | 1.084 | 2.065 | (in its total) | 1.92× (1.90–1.94) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.047 | (in its total) | 1.07× (1.02–1.12) |  |
| inverse BWT, total | 1.525 | 3.178 | 1.905 | 2.10× (2.08–2.12) | 1.26× (1.25–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m45s ± 7m00s |
| 4 threads | 26m30s ± 3m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m42s ± 2m39s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 18m42s ± 2m39s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 2.45 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,762,177 | 17,991,040 | \`04b0571f08562a96…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3685.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
