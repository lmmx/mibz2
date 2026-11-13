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

The latest record: [`e674070`](<https://github.com/lmmx/mibz2/commit/e674070e1351460d27d745390b1db80ef5112b60>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43402546418>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.433 s | 484 | 7.6 | 2.83× (2.79–2.87) | 6.51× (6.47–6.54) | 6.73× (6.68–6.79) | 7.61× (7.54–7.68) | 6.69× (6.62–6.75) | 7.12× (6.92–7.32) | 6.75× (6.65–6.85) | 2.08× (2.07–2.09) | 2.33× (2.31–2.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.278 s | 754 | 8.7 | 2.61× (2.40–2.84) | 5.21× (4.86–5.60) | 6.07× (5.62–6.55) | 6.34× (5.91–6.81) | 5.60× (5.19–6.04) | 11.03× (10.30–11.81) | 10.46× (9.71–11.26) | 3.22× (2.98–3.49) | 3.60× (3.34–3.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.230 s | 911 | 14.7 | 2.54× (2.47–2.61) | 3.79× (3.70–3.88) | 4.71× (4.57–4.85) | 4.55× (4.44–4.66) | 4.54× (4.42–4.67) | 13.35× (12.91–13.80) | 12.66× (12.25–13.09) | 3.90× (3.76–4.04) | 4.36× (4.21–4.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,099 | 3.7 | 0.56× (0.55–0.57) | 3.35× (3.31–3.40) | 4.48× (4.41–4.55) | 3.50× (3.44–3.56) | 6.04× (5.91–6.18) | 2.60× (2.56–2.65) | 2.61× (2.57–2.64) | 0.95× (0.93–0.97) | 1.83× (1.66–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,102 | 3.7 | 0.53× (0.51–0.54) | 2.81× (2.74–2.90) | 5.62× (5.48–5.76) | 3.53× (3.44–3.63) | 5.94× (5.79–6.08) | 2.61× (2.54–2.69) | 2.61× (2.54–2.69) | 0.95× (0.92–0.99) | 1.84× (1.67–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,090 | 3.7 | 0.53× (0.52–0.54) | 2.86× (2.75–2.98) | 5.58× (5.46–5.71) | 3.52× (3.46–3.59) | 6.02× (5.89–6.14) | 2.60× (2.55–2.64) | 2.60× (2.55–2.65) | 0.95× (0.92–0.97) | 1.82× (1.64–2.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 741 | 5.3 | 2.71× (2.68–2.74) | 3.08× (3.05–3.12) | 3.93× (3.78–4.08) | 4.16× (4.07–4.25) | 4.52× (4.46–4.59) | 2.79× (2.76–2.82) | 2.28× (2.26–2.30) | 2.10× (2.08–2.13) | 3.07× (3.04–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,092 | 6.6 | 2.24× (2.05–2.44) | 2.94× (2.58–3.35) | 4.18× (3.84–4.56) | 3.58× (3.26–3.93) | 4.90× (4.56–5.26) | 3.94× (3.67–4.22) | 3.21× (2.98–3.46) | 2.96× (2.75–3.19) | 4.33× (4.02–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 961 | 13.1 | 1.87× (1.83–1.92) | 1.85× (1.67–2.05) | 3.09× (2.99–3.20) | 2.21× (2.03–2.40) | 4.05× (3.95–4.15) | 3.58× (3.49–3.68) | 2.93× (2.84–3.01) | 2.70× (2.63–2.77) | 3.94× (3.83–4.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 51.7 | 1.39× (1.37–1.40) | 2.68× (2.66–2.71) | 2.42× (2.38–2.47) | 2.49× (2.47–2.51) | 2.82× (2.78–2.85) | 3.00× (2.97–3.03) | 3.15× (3.12–3.17) | 1.26× (1.24–1.27) | 1.42× (1.39–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 114 | 58.8 | 1.26× (1.20–1.32) | 2.43× (2.31–2.55) | 2.30× (2.21–2.40) | 2.95× (2.80–3.11) | 2.78× (2.69–2.87) | 4.69× (4.54–4.84) | 4.91× (4.74–5.08) | 1.96× (1.90–2.02) | 2.22× (2.12–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 77.1 | 1.30× (1.28–1.31) | 2.09× (2.00–2.18) | 2.21× (2.12–2.30) | 2.61× (2.47–2.75) | 2.84× (2.77–2.90) | 5.79× (5.64–5.95) | 6.07× (5.90–6.24) | 2.42× (2.36–2.49) | 2.74× (2.65–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 46.6 | 1.52× (1.51–1.53) | 3.05× (3.01–3.09) | 2.64× (2.62–2.66) | 2.75× (2.71–2.78) | 3.08× (3.04–3.12) | 3.67× (3.62–3.72) | 4.01× (3.96–4.07) | 1.49× (1.46–1.51) | 1.79× (1.77–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 129 | 53.5 | 1.33× (1.27–1.38) | 2.81× (2.63–2.99) | 2.48× (2.39–2.58) | 3.32× (3.14–3.52) | 3.05× (2.89–3.22) | 5.83× (5.58–6.09) | 6.38× (6.10–6.68) | 2.36× (2.27–2.46) | 2.84× (2.72–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 151 | 75.0 | 1.39× (1.36–1.42) | 2.30× (2.21–2.40) | 2.25× (2.14–2.35) | 2.96× (2.89–3.04) | 2.87× (2.79–2.96) | 6.91× (6.78–7.04) | 7.56× (7.42–7.70) | 2.80× (2.73–2.87) | 3.37× (3.32–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 4.9 | 0.63× (0.62–0.64) | 3.47× (3.42–3.52) | 4.36× (4.30–4.43) | 3.11× (3.07–3.15) | 6.79× (6.67–6.91) | 2.29× (2.26–2.33) | 2.33× (2.30–2.37) | 0.96× (0.94–0.98) | 1.75× (1.72–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 813 | 5.3 | 0.54× (0.51–0.57) | 2.52× (2.47–2.57) | 5.13× (4.90–5.38) | 3.04× (2.96–3.12) | 6.49× (6.37–6.61) | 2.21× (2.17–2.25) | 2.25× (2.20–2.29) | 0.93× (0.90–0.95) | 1.68× (1.65–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.1 | 0.53× (0.52–0.54) | 2.51× (2.48–2.54) | 5.02× (4.95–5.09) | 3.01× (2.97–3.06) | 6.45× (6.30–6.61) | 2.19× (2.16–2.23) | 2.23× (2.20–2.27) | 0.92× (0.90–0.93) | 1.67× (1.65–1.69) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.058 s | 226 | 16.8 | 1.82× (1.80–1.83) | 3.88× (3.85–3.92) | 4.35× (4.30–4.39) | 4.53× (4.48–4.58) | 4.77× (4.74–4.79) | 4.72× (4.67–4.77) | 4.78× (4.73–4.83) | 1.49× (1.48–1.50) | 1.76× (1.74–1.78) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.043 s | 301 | 20.9 | 1.37× (1.27–1.47) | 2.89× (2.68–3.12) | 3.56× (3.34–3.80) | 3.84× (3.54–4.15) | 4.00× (3.74–4.29) | 6.25× (5.83–6.70) | 6.34× (5.91–6.79) | 1.97× (1.84–2.12) | 2.33× (2.17–2.50) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.038 s | 339 | 32.1 | 1.37× (1.31–1.44) | 2.26× (2.12–2.42) | 2.98× (2.86–3.12) | 3.12× (2.99–3.25) | 3.71× (3.49–3.95) | 7.07× (6.86–7.29) | 7.16× (6.97–7.36) | 2.23× (2.17–2.30) | 2.64× (2.57–2.71) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.82× (2.78–2.86) | 6.47× (6.43–6.51) | 6.70× (6.65–6.75) | 7.58× (7.52–7.65) | 6.66× (6.60–6.73) | 7.09× (6.89–7.30) | 6.73× (6.63–6.83) | 2.08× (2.06–2.09) | 2.31× (2.29–2.32) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.59× (2.51–2.67) | 5.78× (5.64–5.92) | 6.08× (5.93–6.23) | 7.04× (6.85–7.24) | 5.98× (5.81–6.15) | 6.25× (5.99–6.52) | 5.93× (5.75–6.11) | 1.83× (1.78–1.87) | 2.03× (1.99–2.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.47× (2.38–2.57) | 4.09× (3.94–4.25) | 4.45× (4.30–4.61) | 4.93× (4.75–5.11) | 4.31× (4.17–4.45) | 3.71× (3.49–3.94) | 3.52× (3.33–3.72) | 1.08× (1.03–1.14) | 1.21× (1.15–1.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.88× (2.83–2.93) | 3.89× (3.80–3.97) | 3.29× (3.23–3.34) | 5.61× (5.49–5.73) | 2.47× (2.42–2.52) | 2.47× (2.43–2.50) | 0.95× (0.94–0.97) | 1.61× (1.58–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.72× (2.65–2.79) | 4.92× (4.80–5.04) | 3.29× (3.21–3.37) | 5.43× (5.31–5.56) | 2.44× (2.37–2.51) | 2.44× (2.37–2.50) | 0.94× (0.91–0.97) | 1.59× (1.55–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.09× (2.94–3.24) | 5.01× (4.93–5.08) | 3.35× (3.29–3.41) | 5.60× (5.49–5.72) | 2.48× (2.44–2.52) | 2.48× (2.43–2.52) | 0.96× (0.94–0.98) | 1.61× (1.58–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.58–2.63) | 2.85× (2.82–2.87) | 3.50× (3.35–3.65) | 3.96× (3.88–4.04) | 4.31× (4.25–4.37) | 2.68× (2.65–2.71) | 2.20× (2.18–2.22) | 2.04× (2.01–2.06) | 2.66× (2.64–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.19–2.32) | 2.47× (2.38–2.55) | 3.34× (3.26–3.43) | 3.30× (3.23–3.38) | 3.59× (3.51–3.66) | 2.14× (2.09–2.19) | 1.75× (1.71–1.80) | 1.62× (1.59–1.66) | 2.12× (2.07–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.97–2.04) | 1.69× (1.61–1.77) | 2.18× (2.12–2.24) | 1.97× (1.91–2.04) | 2.08× (2.02–2.14) | 1.07× (1.04–1.10) | 0.88× (0.85–0.91) | 0.81× (0.79–0.84) | 1.06× (1.03–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.40) | 2.60× (2.56–2.64) | 2.37× (2.33–2.42) | 2.47× (2.44–2.49) | 2.79× (2.75–2.82) | 2.97× (2.94–3.00) | 3.12× (3.09–3.14) | 1.25× (1.23–1.27) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.22–1.33) | 2.39× (2.29–2.49) | 2.10× (2.02–2.18) | 2.75× (2.66–2.85) | 2.46× (2.37–2.55) | 2.57× (2.49–2.66) | 2.70× (2.60–2.80) | 1.08× (1.05–1.12) | 1.19× (1.15–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.40–1.49) | 2.26× (2.19–2.33) | 1.99× (1.90–2.08) | 2.54× (2.46–2.63) | 2.46× (2.39–2.54) | 1.98× (1.94–2.03) | 2.08× (2.03–2.13) | 0.84× (0.82–0.85) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.50–1.52) | 2.96× (2.92–3.00) | 2.58× (2.55–2.62) | 2.72× (2.68–2.76) | 3.05× (3.01–3.09) | 3.63× (3.57–3.68) | 3.97× (3.91–4.02) | 1.48× (1.46–1.50) | 1.74× (1.72–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.33–1.39) | 2.74× (2.65–2.84) | 2.31× (2.25–2.37) | 3.16× (3.04–3.28) | 2.74× (2.63–2.85) | 3.14× (3.05–3.22) | 3.43× (3.33–3.53) | 1.28× (1.24–1.32) | 1.50× (1.45–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.55–1.61) | 2.47× (2.41–2.54) | 1.98× (1.95–2.02) | 2.78× (2.71–2.85) | 2.63× (2.54–2.71) | 2.27× (2.22–2.31) | 2.48× (2.43–2.53) | 0.92× (0.90–0.95) | 1.09× (1.05–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.47× (2.43–2.50) | 3.45× (3.37–3.52) | 2.84× (2.79–2.88) | 6.05× (5.94–6.16) | 2.13× (2.10–2.16) | 2.16× (2.13–2.20) | 0.97× (0.95–0.99) | 1.37× (1.34–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.57–0.66) | 2.28× (2.24–2.32) | 4.13× (3.95–4.31) | 2.79× (2.73–2.87) | 5.71× (5.62–5.80) | 2.01× (1.98–2.05) | 2.05× (2.01–2.08) | 0.92× (0.89–0.94) | 1.29× (1.27–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.58–0.59) | 2.52× (2.49–2.55) | 4.11× (4.04–4.19) | 2.85× (2.82–2.89) | 5.87× (5.73–6.01) | 2.07× (2.04–2.10) | 2.10× (2.07–2.13) | 0.94× (0.92–0.96) | 1.33× (1.31–1.34) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.79× (1.77–1.81) | 3.75× (3.72–3.79) | 4.14× (4.10–4.17) | 4.43× (4.38–4.47) | 4.66× (4.63–4.68) | 4.60× (4.56–4.65) | 4.66× (4.62–4.71) | 1.48× (1.46–1.49) | 1.62× (1.61–1.63) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.50× (1.47–1.53) | 3.13× (3.07–3.20) | 3.48× (3.40–3.57) | 4.03× (3.95–4.11) | 3.78× (3.70–3.87) | 3.67× (3.59–3.75) | 3.72× (3.64–3.81) | 1.18× (1.15–1.20) | 1.29× (1.26–1.32) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.64× (1.60–1.69) | 2.55× (2.46–2.64) | 2.86× (2.75–2.97) | 3.30× (3.16–3.44) | 3.23× (3.02–3.46) | 2.44× (2.36–2.52) | 2.47× (2.39–2.55) | 0.78× (0.76–0.81) | 0.86× (0.83–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.35× (0.34–0.37) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.92× (11.91–11.94) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.36) | 0.48× (0.40–0.57) | 0.74× (0.72–0.76) | 0.62× (0.60–0.64) | 5.88× (5.76–6.01) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.38–0.38) | 0.62× (0.49–0.78) | 0.68× (0.66–0.69) | 0.60× (0.59–0.61) | 3.95× (3.91–3.99) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.38–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.03× (8.01–8.04) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.65× (0.63–0.68) | 0.63× (0.56–0.71) | 1.56× (1.51–1.62) | 0.78× (0.75–0.81) | 5.67× (5.48–5.88) | 0.46× (0.44–0.47) | 0.48× (0.47–0.50) | 0.52× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.67) | 0.72× (0.66–0.79) | 1.18× (1.16–1.20) | 0.72× (0.71–0.74) | 3.43× (3.37–3.49) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.48–0.53) | 0.42× (0.39–0.45) | 0.91× (0.87–0.96) | 0.73× (0.69–0.77) | 3.17× (3.02–3.33) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.64–0.65) | 0.45× (0.45–0.46) | 0.86× (0.85–0.87) | 0.81× (0.79–0.84) | 2.51× (2.48–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.20–5.24) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.40) | 0.83× (0.80–0.86) | 0.70× (0.67–0.74) | 2.87× (2.76–2.98) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.61) | 0.51× (0.48–0.53) | 0.80× (0.79–0.82) | 0.80× (0.77–0.83) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.23× (0.21–0.24) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.31× (5.30–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.66) | 6.41× (6.39–6.43) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.41× (0.36–0.45) | 1.13× (1.08–1.17) | 0.74× (0.72–0.76) | 3.58× (3.49–3.68) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 54 MB | 0.53× (0.51–0.55) | 0.49× (0.45–0.53) | 0.95× (0.92–0.99) | 0.77× (0.74–0.80) | 2.44× (2.35–2.53) | 0.14× (0.13–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 484 | 2.07 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 754 | 2.37 | 1.14× (1.11–1.16) | 77% | 1.01× (1.01–1.02) | 1.04× (1.02–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 911 | 3.98 | 1.91× (1.81–2.02) | 47% | 1.21× (1.19–1.23) | 1.68× (1.63–1.73) |

**Best:** mibz2 at 4 threads (911 MB/s); lbzip2 at 4 threads (240 MB/s); pbz2 at 4 threads (362 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.16, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.65 | 0.02 | 0.08 | 0.230 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3682 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.10 | 12.8% |
| MTF + RLE2 | 1.57 | 18.2% |
| T-vector build | 1.49 | 17.2% |
| walk (the one each build uses) | 4.04 | 46.7% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.65 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.063 | (in its total) | 2.71× (2.69–2.74) |  |
| walk | 1.045 | 2.072 | (in its total) | 1.98× (1.96–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.042 | (in its total) | 0.97× (0.94–1.00) |  |
| inverse BWT, total | 1.479 | 3.177 | 1.891 | 2.15× (2.13–2.16) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m05s |
| 4 threads | 33m43s ± 4m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m30s ± 3m37s |
| the scanner alone (its serial part) | 12m39s ± 30.2s |
| the workers alone, over 4 | 23m30s ± 3m37s |

At 4 threads the workers bound it. Per byte: the scanner 7.3 ns a byte of bz2, the workers 3.08 ns a byte of output at 4 threads (CPU, summed over them), 1.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3682.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
