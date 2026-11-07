# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-249/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`3035c20`](<https://github.com/lmmx/mibz2/commit/3035c2086b3549343dc734c1bb2a075b7ce0c0ba>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42314998827>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.355 s | 590 | 7.7 | 2.31× (2.23–2.40) | 5.73× (5.53–5.93) | 6.00× (5.74–6.28) | 6.81× (6.54–7.09) | 6.04× (5.84–6.24) | 6.48× (6.25–6.73) | 5.97× (5.74–6.21) | 1.77× (1.71–1.84) | 2.04× (1.97–2.12) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.205 s | 1,022 | 8.2 | 2.41× (2.33–2.49) | 5.33× (5.08–5.60) | 6.41× (6.16–6.66) | 6.51× (6.21–6.83) | 5.79× (5.50–6.10) | 11.50× (10.95–12.07) | 10.58× (10.05–11.14) | 3.14× (2.99–3.30) | 3.62× (3.45–3.81) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,162 | 14.0 | 2.18× (2.11–2.25) | 3.52× (3.41–3.64) | 4.61× (4.45–4.78) | 4.30× (4.17–4.42) | 4.29× (4.16–4.43) | 12.94× (12.52–13.38) | 11.91× (11.50–12.34) | 3.54× (3.42–3.67) | 4.08× (3.93–4.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,492 | 3.3 | 0.59× (0.58–0.60) | 3.97× (3.60–4.38) | 4.83× (4.70–4.96) | 3.85× (3.80–3.91) | 6.67× (6.57–6.78) | 2.94× (2.89–2.98) | 2.90× (2.85–2.96) | 1.01× (1.00–1.03) | 2.28× (2.21–2.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,429 | 3.5 | 0.52× (0.51–0.53) | 3.03× (2.87–3.20) | 5.93× (5.76–6.10) | 3.69× (3.60–3.77) | 6.35× (6.24–6.47) | 2.80× (2.72–2.88) | 2.77× (2.71–2.83) | 0.96× (0.94–0.99) | 2.17× (2.11–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,418 | 3.5 | 0.52× (0.51–0.54) | 3.00× (2.89–3.13) | 5.92× (5.72–6.13) | 3.68× (3.59–3.77) | 6.36× (6.25–6.47) | 2.77× (2.70–2.83) | 2.74× (2.69–2.79) | 0.95× (0.94–0.97) | 2.15× (2.07–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,043 | 4.6 | 2.81× (2.74–2.88) | 3.26× (3.17–3.36) | 3.94× (3.79–4.10) | 4.36× (4.25–4.47) | 5.03× (4.93–5.13) | 2.77× (2.73–2.81) | 2.53× (2.48–2.58) | 2.02× (1.98–2.06) | 3.15× (3.05–3.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,302 | 6.2 | 2.09× (1.78–2.46) | 2.78× (2.51–3.08) | 3.71× (3.28–4.19) | 3.36× (2.99–3.78) | 5.21× (4.33–6.26) | 3.55× (3.18–3.96) | 3.24× (2.91–3.61) | 2.59× (2.33–2.88) | 4.04× (3.63–4.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,174 | 12.6 | 1.56× (1.48–1.65) | 1.95× (1.78–2.15) | 2.70× (2.54–2.88) | 2.08× (1.92–2.25) | 3.99× (3.73–4.27) | 3.17× (2.97–3.37) | 2.89× (2.70–3.10) | 2.31× (2.16–2.47) | 3.61× (3.36–3.87) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 104 | 43.8 | 1.45× (1.38–1.52) | 3.17× (3.13–3.22) | 2.88× (2.83–2.93) | 2.82× (2.77–2.88) | 3.16× (3.10–3.22) | 3.72× (3.65–3.78) | 3.57× (3.48–3.67) | 1.36× (1.34–1.37) | 1.57× (1.54–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.050 s | 159 | 50.5 | 1.25× (1.22–1.28) | 2.85× (2.76–2.95) | 2.63× (2.55–2.72) | 3.29× (3.20–3.39) | 3.11× (3.02–3.20) | 5.77× (5.64–5.89) | 5.54× (5.36–5.73) | 2.10× (2.06–2.15) | 2.43× (2.38–2.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.040 s | 200 | 67.6 | 1.29× (1.24–1.35) | 2.45× (2.32–2.59) | 2.49× (2.35–2.64) | 2.85× (2.74–2.95) | 3.14× (3.04–3.25) | 7.09× (6.86–7.32) | 6.81× (6.56–7.07) | 2.58× (2.51–2.66) | 2.99× (2.90–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.082 s | 117 | 39.1 | 1.51× (1.50–1.52) | 3.78× (3.74–3.82) | 3.11× (3.08–3.14) | 3.24× (3.13–3.36) | 3.51× (3.47–3.54) | 4.75× (4.71–4.79) | 4.93× (4.61–5.27) | 1.64× (1.56–1.74) | 1.89× (1.81–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.053 s | 182 | 45.6 | 1.32× (1.27–1.37) | 3.26× (3.13–3.39) | 2.84× (2.76–2.93) | 3.83× (3.72–3.95) | 3.35× (3.27–3.43) | 7.37× (7.22–7.53) | 7.65× (7.13–8.21) | 2.55× (2.45–2.66) | 2.93× (2.78–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 214 | 63.1 | 1.37× (1.33–1.42) | 2.66× (2.58–2.74) | 2.63× (2.50–2.76) | 3.33× (3.21–3.46) | 3.14× (3.05–3.25) | 8.72× (8.48–8.95) | 9.05× (8.34–9.82) | 3.02× (2.82–3.23) | 3.46× (3.29–3.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,099 | 4.7 | 0.62× (0.61–0.63) | 3.37× (3.28–3.46) | 4.71× (4.57–4.86) | 3.31× (3.15–3.49) | 7.25× (7.16–7.35) | 2.42× (2.36–2.48) | 2.41× (2.35–2.46) | 0.98× (0.97–0.99) | 2.25× (2.19–2.31) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,031 | 5.1 | 0.52× (0.49–0.55) | 3.16× (3.09–3.23) | 5.07× (4.32–5.95) | 3.02× (2.96–3.09) | 6.68× (6.58–6.77) | 2.27× (2.22–2.31) | 2.25× (2.22–2.29) | 0.92× (0.90–0.94) | 2.11× (2.06–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,015 | 5.1 | 0.51× (0.50–0.51) | 3.12× (3.05–3.20) | 4.96× (4.65–5.30) | 3.03× (2.95–3.10) | 6.62× (6.46–6.78) | 2.24× (2.18–2.29) | 2.22× (2.17–2.27) | 0.91× (0.89–0.92) | 2.08× (2.03–2.13) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 0.047 s | 254 | 18.2 | 1.40× (1.39–1.41) | 3.28× (3.21–3.34) | 3.52× (3.47–3.58) | 3.68× (3.66–3.70) | 4.02× (3.96–4.08) | 4.14× (4.11–4.16) | 3.73× (3.71–3.75) | 1.18× (1.17–1.19) | 1.42× (1.35–1.50) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 0.031 s | 384 | 21.4 | 1.17× (1.09–1.26) | 2.79× (2.65–2.94) | 3.53× (3.34–3.73) | 3.69× (3.51–3.88) | 3.69× (3.52–3.87) | 6.14× (5.87–6.43) | 5.54× (5.31–5.78) | 1.75× (1.67–1.83) | 2.11× (1.97–2.26) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 0.029 s | 419 | 32.0 | 1.11× (1.07–1.15) | 2.00× (1.89–2.13) | 2.93× (2.83–3.04) | 2.91× (2.85–2.98) | 3.27× (3.20–3.35) | 6.88× (6.77–7.00) | 6.20× (6.11–6.30) | 1.96× (1.93–1.99) | 2.36× (2.24–2.49) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.31× (2.22–2.39) | 5.70× (5.51–5.90) | 5.98× (5.72–6.25) | 6.79× (6.52–7.07) | 6.02× (5.82–6.23) | 6.46× (6.23–6.71) | 5.95× (5.72–6.19) | 1.77× (1.70–1.84) | 2.03× (1.95–2.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.23× (2.19–2.28) | 5.48× (5.33–5.64) | 5.79× (5.66–5.91) | 6.72× (6.56–6.89) | 5.72× (5.56–5.89) | 6.06× (5.88–6.24) | 5.58× (5.39–5.77) | 1.66× (1.60–1.71) | 1.90× (1.85–1.95) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.03× (1.99–2.07) | 3.76× (3.63–3.89) | 4.12× (4.03–4.21) | 4.61× (4.47–4.76) | 4.01× (3.88–4.15) | 3.55× (3.41–3.70) | 3.27× (3.14–3.41) | 0.97× (0.93–1.01) | 1.11× (1.07–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.36× (3.04–3.72) | 4.07× (4.01–4.12) | 3.60× (3.53–3.67) | 6.17× (6.02–6.31) | 2.77× (2.71–2.84) | 2.73× (2.66–2.80) | 1.01× (1.00–1.03) | 1.92× (1.87–1.96) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.86× (2.79–2.93) | 5.00× (4.91–5.10) | 3.40× (3.33–3.47) | 5.77× (5.65–5.89) | 2.59× (2.52–2.65) | 2.55× (2.50–2.61) | 0.94× (0.92–0.97) | 1.79× (1.75–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.09× (3.03–3.14) | 5.07× (4.94–5.21) | 3.46× (3.38–3.54) | 5.87× (5.76–5.97) | 2.62× (2.56–2.68) | 2.58× (2.54–2.63) | 0.96× (0.94–0.97) | 1.81× (1.78–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.62–2.77) | 3.02× (2.96–3.08) | 3.59× (3.51–3.68) | 4.15× (4.03–4.27) | 4.77× (4.66–4.88) | 2.66× (2.61–2.72) | 2.43× (2.37–2.49) | 1.96× (1.91–2.00) | 2.66× (2.62–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.07–2.33) | 2.47× (2.35–2.60) | 2.92× (2.83–3.01) | 3.25× (3.11–3.40) | 3.87× (3.36–4.45) | 1.98× (1.89–2.06) | 1.80× (1.73–1.89) | 1.45× (1.39–1.52) | 1.97× (1.91–2.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.80× (1.72–1.88) | 1.61× (1.56–1.67) | 1.96× (1.89–2.03) | 2.05× (1.98–2.11) | 2.14× (2.07–2.21) | 1.01× (0.98–1.04) | 0.92× (0.90–0.95) | 0.74× (0.72–0.77) | 1.01× (0.97–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.37–1.51) | 3.08× (3.05–3.12) | 2.77× (2.72–2.82) | 2.79× (2.74–2.85) | 3.12× (3.07–3.19) | 3.68× (3.61–3.74) | 3.53× (3.44–3.63) | 1.35× (1.34–1.36) | 1.48× (1.46–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.78× (2.72–2.84) | 2.43× (2.38–2.48) | 3.15× (3.06–3.24) | 2.76× (2.70–2.82) | 3.19× (3.13–3.25) | 3.06× (2.99–3.13) | 1.17× (1.15–1.19) | 1.29× (1.26–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.38–1.47) | 2.59× (2.55–2.63) | 2.14× (2.10–2.17) | 2.82× (2.78–2.86) | 2.64× (2.59–2.70) | 2.40× (2.35–2.44) | 2.30× (2.25–2.35) | 0.88× (0.86–0.90) | 0.97× (0.95–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.49–1.52) | 3.67× (3.64–3.70) | 3.02× (3.00–3.05) | 3.21× (3.10–3.32) | 3.47× (3.43–3.51) | 4.69× (4.65–4.73) | 4.87× (4.55–5.21) | 1.63× (1.55–1.73) | 1.80× (1.73–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.32–1.40) | 3.27× (3.21–3.33) | 2.64× (2.58–2.70) | 3.67× (3.57–3.78) | 3.03× (2.96–3.10) | 4.00× (3.93–4.06) | 4.15× (3.86–4.46) | 1.39× (1.32–1.47) | 1.54× (1.47–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.53–1.57) | 2.95× (2.88–3.04) | 2.28× (2.15–2.42) | 3.15× (3.04–3.26) | 2.85× (2.76–2.94) | 2.90× (2.82–2.97) | 3.01× (2.80–3.23) | 1.01× (0.95–1.08) | 1.11× (1.06–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.60× (2.52–2.67) | 3.45× (3.35–3.55) | 3.02× (2.88–3.16) | 6.45× (6.33–6.57) | 2.25× (2.18–2.33) | 2.23× (2.16–2.29) | 0.99× (0.97–1.00) | 1.52× (1.49–1.56) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.62) | 2.35× (2.31–2.38) | 4.20× (3.69–4.77) | 2.77× (2.72–2.82) | 5.84× (5.77–5.91) | 2.07× (2.03–2.10) | 2.04× (2.01–2.08) | 0.91× (0.88–0.94) | 1.40× (1.37–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.53× (2.47–2.58) | 4.02× (3.94–4.11) | 2.85× (2.78–2.92) | 5.96× (5.82–6.10) | 2.10× (2.04–2.16) | 2.08× (2.03–2.13) | 0.92× (0.90–0.94) | 1.42× (1.39–1.45) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 1.39× (1.38–1.40) | 3.15× (3.12–3.17) | 3.39× (3.37–3.42) | 3.61× (3.58–3.63) | 3.94× (3.89–4.00) | 4.06× (4.03–4.09) | 3.66× (3.63–3.69) | 1.17× (1.16–1.18) | 1.28× (1.27–1.29) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 1.24× (1.22–1.26) | 2.74× (2.71–2.78) | 3.01× (2.96–3.07) | 3.41× (3.35–3.46) | 3.33× (3.27–3.40) | 3.41× (3.35–3.47) | 3.08× (3.02–3.13) | 0.99× (0.97–1.00) | 1.08× (1.06–1.09) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 1.32× (1.28–1.37) | 2.25× (2.17–2.33) | 2.50× (2.41–2.61) | 2.75× (2.67–2.82) | 2.73× (2.65–2.80) | 2.31× (2.26–2.37) | 2.08× (2.03–2.14) | 0.67× (0.65–0.69) | 0.73× (0.71–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.36–0.38) | 1.08× (1.08–1.08) | 0.87× (0.85–0.89) | 12.23× (12.22–12.24) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.57× (0.48–0.67) | 0.80× (0.79–0.81) | 0.67× (0.65–0.68) | 6.35× (6.27–6.43) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.60× (0.49–0.73) | 0.71× (0.70–0.72) | 0.61× (0.60–0.62) | 4.10× (4.08–4.13) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.83–0.84) | 4.44× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.46× (0.45–0.47) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.19× (8.17–8.22) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.62–0.69) | 0.62× (0.59–0.65) | 1.56× (1.49–1.64) | 0.77× (0.74–0.80) | 5.66× (5.41–5.93) | 0.45× (0.43–0.47) | 0.47× (0.45–0.50) | 0.51× (0.49–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.60× (0.57–0.63) | 0.68× (0.63–0.74) | 1.13× (1.09–1.17) | 0.73× (0.70–0.76) | 3.29× (3.16–3.42) | 0.25× (0.24–0.26) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.73–0.74) | 5.28× (5.26–5.29) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.44) | 0.93× (0.89–0.97) | 0.73× (0.69–0.76) | 3.20× (3.06–3.36) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.82× (0.79–0.85) | 2.55× (2.49–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.39× (0.36–0.41) | 0.86× (0.83–0.89) | 0.72× (0.68–0.76) | 2.94× (2.84–3.05) | 0.18× (0.17–0.19) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.49) | 0.82× (0.81–0.83) | 0.81× (0.78–0.85) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.43× (5.41–5.45) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.77) | 5.44× (5.43–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 18 MB | 0.47× (0.46–0.47) | 0.41× (0.40–0.41) | 1.50× (1.50–1.50) | 0.65× (0.64–0.65) | 6.76× (6.75–6.77) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.66–0.66) | 0.33× (0.33–0.33) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 31 MB | 0.51× (0.50–0.52) | 0.44× (0.40–0.50) | 1.24× (1.22–1.26) | 0.78× (0.76–0.79) | 3.93× (3.88–3.97) | 0.23× (0.23–0.24) | 0.25× (0.25–0.25) | 0.39× (0.38–0.39) | 0.19× (0.19–0.19) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 49 MB | 0.55× (0.54–0.57) | 0.53× (0.47–0.59) | 1.06× (1.03–1.09) | 0.82× (0.78–0.86) | 2.69× (2.60–2.78) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.26) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 590 | 1.70 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,022 | 1.83 | 1.07× (1.04–1.10) | 89% | 1.03× (1.01–1.04) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,162 | 3.11 | 1.82× (1.76–1.88) | 50% | 1.20× (1.18–1.22) | 1.60× (1.55–1.66) |

**Best:** mibz2 at 4 threads (1,162 MB/s); lbzip2 at 4 threads (328 MB/s); pbz2 at 4 threads (538 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.15, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.49 | 0.01 | 0.09 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4506 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 14.3% |
| MTF + RLE2 | 1.43 | 19.9% |
| T-vector build | 1.48 | 20.7% |
| walk (the one each build uses) | 2.81 | 39.3% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.25 | 3.5% |
| **total** | 7.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.311 | 0.688 | (in its total) | 2.21× (2.21–2.22) |  |
| walk | 0.614 | 1.543 | (in its total) | 2.52× (2.51–2.53) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.029 | (in its total) | 1.12× (1.12–1.13) |  |
| inverse BWT, total | 0.951 | 2.260 | 1.258 | 2.38× (2.37–2.39) | 1.33× (1.32–1.33) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 53m36s ± 6m40s |
| 4 threads | 26m51s ± 3m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m31s ± 2m52s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 18m31s ± 2m52s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 2.43 ns a byte of output at 4 threads (CPU, summed over them), 1.48 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,758,792 | 12,027,975 | \`3c64b8b3acbf9c14…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4505.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
