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

The latest record: [`fc44897`](<https://github.com/lmmx/mibz2/commit/fc44897d75e428a0975ca02c72ae3a0b9d824ae2>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43408049737>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.300 s | 700 | 6.5 | 2.87× (2.70–3.05) | 7.09× (6.78–7.42) | 7.34× (6.96–7.75) | 8.40× (8.01–8.82) | 7.56× (7.24–7.91) | 7.67× (7.25–8.11) | 7.24× (6.84–7.66) | 2.25× (2.11–2.40) | 2.55× (2.41–2.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.192 s | 1,091 | 7.3 | 2.72× (2.50–2.96) | 6.04× (5.61–6.50) | 7.13× (6.55–7.75) | 7.17× (6.64–7.73) | 6.38× (5.93–6.88) | 12.28× (11.43–13.19) | 11.59× (10.80–12.45) | 3.60× (3.29–3.94) | 4.09× (3.80–4.40) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.178 s | 1,178 | 13.0 | 2.40× (2.32–2.47) | 3.83× (3.69–3.97) | 4.87× (4.71–5.04) | 4.65× (4.51–4.80) | 4.69× (4.54–4.85) | 13.44× (12.89–14.02) | 12.69× (12.13–13.28) | 3.94× (3.70–4.20) | 4.48× (4.29–4.67) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,434 | 3.5 | 0.58× (0.57–0.59) | 3.64× (3.53–3.75) | 4.76× (4.47–5.07) | 4.12× (3.63–4.67) | 6.58× (6.46–6.69) | 2.87× (2.83–2.92) | 2.87× (2.81–2.94) | 1.02× (0.99–1.04) | 2.19× (2.12–2.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,370 | 3.8 | 0.55× (0.46–0.65) | 3.06× (2.84–3.29) | 5.83× (5.64–6.03) | 3.70× (3.57–3.84) | 6.26× (6.06–6.46) | 2.75× (2.67–2.84) | 2.75× (2.67–2.84) | 0.97× (0.93–1.02) | 2.09× (2.04–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,404 | 3.5 | 0.52× (0.51–0.54) | 3.03× (2.82–3.27) | 5.82× (5.67–5.98) | 3.71× (3.58–3.85) | 6.37× (6.22–6.52) | 2.79× (2.70–2.88) | 2.79× (2.72–2.85) | 0.98× (0.94–1.03) | 2.12× (2.05–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,062 | 4.6 | 2.94× (2.89–2.99) | 3.74× (3.63–3.86) | 4.32× (4.14–4.50) | 4.71× (4.66–4.75) | 5.55× (5.44–5.67) | 3.11× (3.02–3.21) | 2.59× (2.55–2.63) | 2.16× (2.12–2.19) | 3.30× (3.21–3.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,426 | 6.1 | 2.16× (1.97–2.36) | 3.05× (2.77–3.36) | 4.08× (3.74–4.44) | 3.64× (3.35–3.96) | 5.65× (5.21–6.13) | 4.10× (3.80–4.42) | 3.41× (3.15–3.69) | 2.84× (2.62–3.08) | 4.34× (3.99–4.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,260 | 11.9 | 1.74× (1.65–1.82) | 2.01× (1.86–2.16) | 3.01× (2.86–3.17) | 2.18× (2.10–2.27) | 4.47× (4.23–4.73) | 3.64× (3.40–3.91) | 3.03× (2.86–3.22) | 2.52× (2.37–2.69) | 3.86× (3.61–4.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.080 s | 99 | 46.2 | 1.39× (1.36–1.41) | 3.17× (3.03–3.31) | 2.82× (2.72–2.92) | 2.74× (2.65–2.84) | 3.16× (3.01–3.31) | 3.51× (3.37–3.66) | 3.55× (3.45–3.66) | 1.33× (1.29–1.36) | 1.56× (1.49–1.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 151 | 54.0 | 1.22× (1.16–1.28) | 2.74× (2.60–2.88) | 2.49× (2.36–2.63) | 3.16× (2.95–3.38) | 2.99× (2.82–3.17) | 5.33× (5.01–5.67) | 5.39× (5.14–5.66) | 2.02× (1.92–2.12) | 2.36× (2.28–2.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 194 | 70.6 | 1.28× (1.23–1.33) | 2.45× (2.30–2.60) | 2.42× (2.22–2.63) | 2.76× (2.58–2.95) | 3.09× (2.94–3.24) | 6.67× (6.23–7.14) | 6.74× (6.34–7.18) | 2.52× (2.39–2.66) | 2.95× (2.76–3.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.086 s | 112 | 41.1 | 1.51× (1.46–1.57) | 3.68× (3.56–3.81) | 3.09× (2.90–3.29) | 2.95× (2.81–3.10) | 3.49× (3.28–3.70) | 4.33× (4.15–4.53) | 4.68× (4.52–4.84) | 1.68× (1.51–1.86) | 1.85× (1.73–1.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 48.1 | 1.28× (1.23–1.33) | 3.10× (3.01–3.19) | 2.75× (2.69–2.82) | 3.67× (3.54–3.81) | 3.27× (3.17–3.39) | 6.72× (6.47–6.98) | 7.25× (6.94–7.56) | 2.60× (2.35–2.87) | 2.87× (2.64–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.048 s | 202 | 68.6 | 1.37× (1.30–1.45) | 2.56× (2.51–2.61) | 2.49× (2.39–2.59) | 3.23× (3.14–3.33) | 3.08× (3.00–3.17) | 7.98× (7.79–8.18) | 8.61× (8.35–8.89) | 3.09× (2.79–3.42) | 3.42× (3.18–3.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,064 | 4.9 | 0.70× (0.61–0.80) | 3.27× (3.17–3.38) | 4.54× (4.39–4.70) | 3.22× (3.14–3.30) | 7.20× (7.04–7.36) | 2.41× (2.35–2.46) | 2.44× (2.38–2.50) | 0.99× (0.96–1.02) | 2.19× (2.13–2.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 5.3 | 0.47× (0.39–0.57) | 2.83× (2.35–3.41) | 4.87× (4.04–5.88) | 2.83× (2.37–3.39) | 6.16× (5.12–7.41) | 2.08× (1.73–2.49) | 2.11× (1.75–2.54) | 0.85× (0.71–1.03) | 1.89× (1.57–2.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 5.2 | 0.52× (0.51–0.53) | 3.10× (3.00–3.20) | 5.42× (5.20–5.64) | 3.07× (2.99–3.15) | 6.76× (6.56–6.97) | 2.27× (2.21–2.33) | 2.31× (2.25–2.36) | 0.94× (0.90–0.97) | 2.07× (2.01–2.14) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.045 s | 292 | 16.0 | 1.78× (1.67–1.90) | 4.36× (3.92–4.85) | 4.50× (4.33–4.67) | 4.68× (4.54–4.82) | 4.96× (4.78–5.15) | 5.13× (4.83–5.44) | 4.81× (4.63–5.01) | 1.50× (1.46–1.53) | 1.80× (1.74–1.86) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.033 s | 393 | 20.2 | 1.36× (1.32–1.41) | 3.31× (3.13–3.50) | 3.84× (3.67–4.02) | 4.15× (3.97–4.35) | 4.31× (4.07–4.56) | 7.03× (6.69–7.39) | 6.60× (6.35–6.87) | 2.06× (1.96–2.16) | 2.47× (2.35–2.59) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.031 s | 416 | 30.3 | 1.27× (1.13–1.42) | 2.42× (2.30–2.55) | 3.05× (2.92–3.19) | 3.11× (2.92–3.31) | 3.65× (3.52–3.78) | 7.44× (7.09–7.82) | 6.99× (6.66–7.33) | 2.18× (2.07–2.29) | 2.61× (2.49–2.74) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.86× (2.69–3.04) | 7.05× (6.74–7.38) | 7.30× (6.92–7.70) | 8.37× (7.98–8.77) | 7.53× (7.21–7.87) | 7.63× (7.22–8.07) | 7.21× (6.81–7.63) | 2.24× (2.10–2.39) | 2.53× (2.39–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.59× (2.46–2.73) | 6.35× (5.97–6.75) | 6.72× (6.31–7.16) | 7.78× (7.27–8.32) | 6.59× (6.19–7.02) | 6.76× (6.36–7.19) | 6.38× (6.00–6.79) | 1.99× (1.84–2.14) | 2.24× (2.09–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.37× (2.28–2.47) | 4.31× (4.17–4.45) | 4.71× (4.55–4.87) | 5.30× (5.11–5.50) | 4.65× (4.47–4.84) | 3.92× (3.73–4.12) | 3.70× (3.50–3.91) | 1.15× (1.08–1.23) | 1.30× (1.23–1.37) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.64) | 3.13× (3.02–3.25) | 4.19× (3.87–4.54) | 3.82× (3.38–4.31) | 6.04× (5.89–6.20) | 2.70× (2.64–2.76) | 2.69× (2.61–2.77) | 1.01× (0.99–1.04) | 1.91× (1.85–1.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.49–0.73) | 2.83× (2.69–2.96) | 4.87× (4.65–5.09) | 3.34× (3.17–3.51) | 5.55× (5.29–5.83) | 2.49× (2.36–2.62) | 2.48× (2.36–2.60) | 0.93× (0.88–1.00) | 1.76× (1.68–1.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.55–0.58) | 3.09× (2.99–3.20) | 5.08× (4.95–5.21) | 3.50× (3.37–3.62) | 5.88× (5.75–6.02) | 2.64× (2.56–2.72) | 2.62× (2.57–2.68) | 0.99× (0.95–1.03) | 1.86× (1.80–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.80× (2.74–2.86) | 3.25× (3.20–3.31) | 3.82× (3.75–3.90) | 4.44× (4.38–4.51) | 5.22× (5.12–5.31) | 2.96× (2.86–3.07) | 2.47× (2.42–2.53) | 2.07× (2.03–2.12) | 2.81× (2.72–2.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.17–2.37) | 2.63× (2.53–2.74) | 3.15× (3.07–3.24) | 3.51× (3.38–3.64) | 4.12× (3.79–4.47) | 2.24× (2.12–2.36) | 1.86× (1.78–1.95) | 1.56× (1.49–1.64) | 2.12× (2.03–2.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.92× (1.85–2.00) | 1.75× (1.65–1.86) | 2.14× (2.06–2.23) | 2.10× (2.01–2.20) | 2.39× (2.28–2.51) | 1.14× (1.07–1.22) | 0.95× (0.90–1.01) | 0.80× (0.75–0.85) | 1.08× (1.02–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.35–1.41) | 3.07× (2.94–3.20) | 2.73× (2.64–2.82) | 2.71× (2.62–2.81) | 3.12× (2.98–3.27) | 3.47× (3.33–3.61) | 3.50× (3.41–3.61) | 1.32× (1.29–1.36) | 1.48× (1.43–1.53) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.19–1.26) | 2.70× (2.60–2.80) | 2.28× (2.18–2.38) | 2.97× (2.83–3.12) | 2.62× (2.50–2.74) | 2.91× (2.77–3.05) | 2.94× (2.83–3.06) | 1.11× (1.07–1.15) | 1.24× (1.20–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.39–1.47) | 2.59× (2.52–2.67) | 2.18× (2.06–2.29) | 2.78× (2.73–2.84) | 2.64× (2.59–2.70) | 2.28× (2.19–2.36) | 2.30× (2.23–2.38) | 0.87× (0.85–0.89) | 0.97× (0.94–1.01) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.46–1.56) | 3.58× (3.46–3.71) | 2.99× (2.81–3.19) | 2.93× (2.78–3.08) | 3.45× (3.25–3.66) | 4.28× (4.09–4.48) | 4.62× (4.45–4.79) | 1.67× (1.50–1.86) | 1.78× (1.67–1.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.37) | 3.19× (3.11–3.27) | 2.60× (2.53–2.68) | 3.55× (3.44–3.66) | 3.03× (2.95–3.12) | 3.72× (3.60–3.84) | 4.01× (3.89–4.14) | 1.45× (1.31–1.61) | 1.55× (1.45–1.66) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.48–1.61) | 2.82× (2.74–2.90) | 2.16× (2.11–2.21) | 2.98× (2.90–3.07) | 2.75× (2.69–2.81) | 2.61× (2.54–2.67) | 2.81× (2.73–2.90) | 1.02× (0.90–1.14) | 1.09× (1.02–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.75× (0.65–0.87) | 2.58× (2.52–2.64) | 3.38× (3.33–3.43) | 2.90× (2.83–2.97) | 6.32× (6.18–6.46) | 2.21× (2.16–2.26) | 2.24× (2.19–2.30) | 0.99× (0.97–1.02) | 1.55× (1.51–1.59) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.45–0.65) | 2.16× (1.81–2.58) | 3.70× (3.10–4.41) | 2.58× (2.15–3.10) | 5.35× (4.44–6.46) | 1.88× (1.56–2.26) | 1.91× (1.58–2.30) | 0.84× (0.70–1.01) | 1.32× (1.09–1.59) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.59× (2.51–2.68) | 4.15× (4.03–4.29) | 2.88× (2.82–2.95) | 6.07× (5.92–6.23) | 2.13× (2.08–2.18) | 2.16× (2.12–2.19) | 0.95× (0.92–0.99) | 1.49× (1.45–1.53) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.76× (1.65–1.87) | 4.16× (3.73–4.64) | 4.27× (4.13–4.41) | 4.56× (4.43–4.69) | 4.83× (4.66–5.01) | 4.99× (4.71–5.29) | 4.70× (4.51–4.89) | 1.48× (1.45–1.52) | 1.62× (1.56–1.67) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.44× (1.42–1.47) | 3.35× (3.22–3.49) | 3.64× (3.51–3.76) | 4.18× (4.07–4.30) | 3.91× (3.72–4.11) | 3.98× (3.85–4.12) | 3.75× (3.65–3.85) | 1.18× (1.14–1.23) | 1.29× (1.25–1.33) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.55× (1.39–1.74) | 2.71× (2.53–2.89) | 2.93× (2.82–3.04) | 3.26× (3.13–3.40) | 3.15× (3.05–3.25) | 2.62× (2.53–2.72) | 2.47× (2.37–2.56) | 0.78× (0.74–0.81) | 0.85× (0.81–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.34–0.38) | 1.05× (1.05–1.05) | 0.85× (0.83–0.88) | 11.91× (11.90–11.93) | 0.28× (0.28–0.28) | 0.31× (0.31–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.35–0.37) | 0.53× (0.44–0.63) | 0.75× (0.73–0.76) | 0.63× (0.62–0.64) | 5.97× (5.86–6.08) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.37× (0.37–0.38) | 0.57× (0.48–0.68) | 0.66× (0.66–0.67) | 0.59× (0.58–0.60) | 3.90× (3.87–3.92) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.46) | 1.71× (1.70–1.71) | 0.67× (0.66–0.68) | 8.05× (8.01–8.09) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.62–0.67) | 0.61× (0.55–0.68) | 1.54× (1.47–1.61) | 0.77× (0.73–0.81) | 5.58× (5.33–5.85) | 0.45× (0.43–0.47) | 0.48× (0.45–0.50) | 0.52× (0.49–0.54) | 0.26× (0.24–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.60–0.65) | 0.69× (0.66–0.71) | 1.13× (1.08–1.17) | 0.70× (0.67–0.73) | 3.31× (3.19–3.43) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.20× (5.19–5.21) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.40× (0.37–0.43) | 0.90× (0.86–0.94) | 0.72× (0.69–0.76) | 3.11× (2.99–3.25) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.87× (0.84–0.89) | 0.83× (0.81–0.85) | 2.54× (2.49–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.41) | 0.86× (0.83–0.90) | 0.74× (0.70–0.78) | 2.97× (2.85–3.10) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.51) | 0.80× (0.78–0.82) | 0.80× (0.77–0.84) | 2.32× (2.26–2.37) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.77× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.30–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.65–0.67) | 6.40× (6.39–6.41) | 0.38× (0.38–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 35 MB | 0.48× (0.46–0.49) | 0.45× (0.42–0.49) | 1.11× (1.08–1.14) | 0.72× (0.70–0.75) | 3.54× (3.43–3.65) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.52) | 0.45× (0.40–0.51) | 0.92× (0.89–0.96) | 0.76× (0.74–0.78) | 2.36× (2.28–2.44) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 700 | 1.44 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,091 | 1.62 | 1.13× (1.03–1.24) | 80% | 1.02× (0.99–1.04) | 1.02× (0.98–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,178 | 2.89 | 1.95× (1.83–2.08) | 44% | 1.19× (1.15–1.23) | 1.61× (1.56–1.67) |

**Best:** mibz2 at 4 threads (1,178 MB/s); lbzip2 at 4 threads (320 MB/s); pbz2 at 4 threads (506 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.91, mibz2 4t 1.28, lbzip2 1t 0.07, lbzip2 2t 0.34, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.56. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 15%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.44 | 0.01 | 0.08 | 0.178 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4509 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.05 | 14.5% |
| MTF + RLE2 | 1.53 | 21.1% |
| T-vector build | 1.33 | 18.3% |
| walk (the one each build uses) | 2.90 | 40.0% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.26 | 3.6% |
| **total** | 7.26 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.302 | 0.714 | (in its total) | 2.32× (2.21–2.44) |  |
| walk | 0.668 | 1.620 | (in its total) | 2.40× (2.28–2.53) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.030 | 0.031 | (in its total) | 1.05× (0.99–1.11) |  |
| inverse BWT, total | 1.000 | 2.365 | 1.335 | 2.34× (2.22–2.46) | 1.33× (1.26–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m33s ± 5m34s |
| 4 threads | 25m23s ± 3m43s |
| 4 threads, the sample's overlap gap as a fixed cost | 16m00s ± 2m27s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 16m00s ± 2m27s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 2.10 ns a byte of output at 4 threads (CPU, summed over them), 1.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4509.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
