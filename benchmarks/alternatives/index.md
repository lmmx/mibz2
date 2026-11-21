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

The latest record: [`23e40f7`](<https://github.com/lmmx/mibz2/commit/23e40f7708c86e4a421b1f0b9a6413ad3a6e70db>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44903808691>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.298 s | 704 | 6.2 | 2.80× (2.76–2.83) | 6.94× (6.83–7.05) | 7.29× (7.19–7.39) | 8.28× (8.18–8.38) | 7.39× (7.30–7.49) | 7.59× (7.50–7.68) | 7.19× (7.10–7.28) | 2.20× (2.17–2.23) | 2.54× (2.50–2.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.174 s | 1,206 | 6.8 | 2.78× (2.66–2.91) | 6.25× (5.92–6.60) | 7.41× (7.02–7.82) | 7.57× (7.22–7.93) | 6.75× (6.43–7.09) | 12.70× (12.10–13.33) | 12.02× (11.46–12.61) | 3.67× (3.49–3.87) | 4.25× (4.06–4.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.146 s | 1,435 | 10.4 | 2.67× (2.53–2.82) | 4.34× (4.10–4.59) | 5.57× (5.26–5.90) | 5.24× (4.96–5.52) | 5.34× (5.10–5.60) | 15.03× (14.27–15.82) | 14.23× (13.53–14.97) | 4.35× (4.13–4.58) | 5.03× (4.78–5.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,454 | 3.2 | 0.58× (0.56–0.60) | 3.71× (3.61–3.81) | 4.60× (4.47–4.73) | 3.75× (3.64–3.86) | 6.58× (6.33–6.83) | 2.85× (2.77–2.92) | 2.83× (2.76–2.91) | 0.99× (0.97–1.01) | 2.23× (2.17–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,424 | 3.4 | 0.52× (0.51–0.53) | 2.91× (2.82–3.00) | 5.95× (5.77–6.14) | 3.70× (3.62–3.78) | 6.40× (6.17–6.63) | 2.78× (2.71–2.86) | 2.77× (2.66–2.88) | 0.97× (0.93–1.01) | 2.18× (2.11–2.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,413 | 3.4 | 0.52× (0.51–0.53) | 2.94× (2.77–3.11) | 5.86× (5.69–6.05) | 3.62× (3.54–3.71) | 6.35× (6.13–6.57) | 2.74× (2.65–2.83) | 2.73× (2.63–2.83) | 0.96× (0.92–0.99) | 2.15× (2.08–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,089 | 4.3 | 2.90× (2.85–2.94) | 3.45× (3.33–3.57) | 4.27× (3.92–4.64) | 4.66× (4.54–4.78) | 5.38× (5.28–5.48) | 3.14× (3.06–3.22) | 2.61× (2.58–2.64) | 2.17× (2.15–2.19) | 3.38× (3.32–3.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.012 s | 1,574 | 5.5 | 2.31× (2.22–2.40) | 3.41× (3.16–3.68) | 4.49× (4.14–4.87) | 4.00× (3.80–4.21) | 6.05× (5.87–6.24) | 4.50× (4.34–4.67) | 3.74× (3.62–3.86) | 3.11× (3.00–3.22) | 4.84× (4.69–5.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,229 | 11.3 | 1.66× (1.59–1.74) | 1.97× (1.89–2.06) | 2.76× (2.64–2.88) | 2.12× (2.01–2.24) | 4.34× (4.14–4.54) | 3.51× (3.37–3.66) | 2.91× (2.80–3.04) | 2.42× (2.33–2.52) | 3.78× (3.63–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 103 | 42.4 | 1.39× (1.37–1.42) | 3.14× (3.07–3.21) | 2.82× (2.74–2.89) | 2.79× (2.73–2.85) | 3.15× (3.07–3.23) | 3.56× (3.50–3.62) | 3.52× (3.47–3.58) | 1.37× (1.34–1.40) | 1.57× (1.54–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 50.2 | 1.23× (1.14–1.32) | 2.78× (2.62–2.94) | 2.61× (2.35–2.90) | 3.23× (3.04–3.43) | 3.01× (2.85–3.18) | 5.26× (5.02–5.52) | 5.21× (4.95–5.48) | 2.03× (1.92–2.14) | 2.32× (2.21–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 191 | 67.8 | 1.25× (1.21–1.30) | 2.44× (2.34–2.54) | 2.39× (2.28–2.49) | 2.85× (2.79–2.91) | 3.07× (2.99–3.15) | 6.55× (6.34–6.77) | 6.49× (6.31–6.67) | 2.52× (2.44–2.61) | 2.88× (2.81–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.082 s | 118 | 37.4 | 1.49× (1.47–1.52) | 3.73× (3.66–3.81) | 3.09× (2.97–3.21) | 3.07× (3.04–3.09) | 3.52× (3.46–3.58) | 4.40× (4.35–4.46) | 4.73× (4.67–4.78) | 1.63× (1.60–1.66) | 1.88× (1.83–1.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.054 s | 177 | 44.7 | 1.25× (1.20–1.31) | 3.23× (2.92–3.56) | 2.73× (2.61–2.85) | 3.71× (3.47–3.97) | 3.23× (3.05–3.41) | 6.60× (6.23–7.00) | 7.09× (6.69–7.51) | 2.45× (2.30–2.60) | 2.82× (2.68–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 214 | 61.4 | 1.35× (1.33–1.38) | 2.67× (2.57–2.77) | 2.58× (2.51–2.64) | 3.41× (3.28–3.55) | 3.16× (3.08–3.24) | 8.08× (7.87–8.30) | 8.68× (8.50–8.86) | 3.00× (2.92–3.07) | 3.45× (3.33–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,087 | 4.6 | 0.63× (0.61–0.64) | 3.33× (3.25–3.42) | 4.93× (4.34–5.59) | 3.18× (3.13–3.23) | 7.19× (7.03–7.36) | 2.38× (2.33–2.43) | 2.41× (2.36–2.46) | 0.98× (0.95–1.01) | 2.23× (2.17–2.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,020 | 5.0 | 0.50× (0.49–0.51) | 3.13× (3.06–3.20) | 5.21× (4.94–5.49) | 3.05× (2.98–3.12) | 6.76× (6.63–6.89) | 2.23× (2.17–2.29) | 2.26× (2.23–2.29) | 0.92× (0.90–0.94) | 2.09× (2.04–2.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,015 | 4.8 | 0.50× (0.49–0.52) | 3.09× (3.01–3.17) | 5.01× (4.70–5.35) | 2.98× (2.93–3.03) | 6.68× (6.54–6.81) | 2.19× (2.14–2.24) | 2.22× (2.17–2.27) | 0.90× (0.88–0.93) | 2.05× (2.00–2.11) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.043 s | 313 | 14.3 | 1.77× (1.69–1.86) | 4.25× (4.19–4.32) | 4.59× (4.47–4.71) | 4.75× (4.69–4.80) | 5.04× (4.99–5.08) | 5.09× (5.04–5.13) | 4.86× (4.82–4.91) | 1.54× (1.50–1.58) | 1.80× (1.73–1.88) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.032 s | 418 | 18.3 | 1.38× (1.30–1.47) | 3.28× (3.17–3.40) | 4.05× (3.84–4.28) | 4.30× (4.11–4.50) | 4.50× (4.17–4.87) | 6.87× (6.62–7.14) | 6.57× (6.34–6.81) | 2.08× (2.00–2.18) | 2.44× (2.36–2.52) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.028 s | 475 | 28.6 | 1.36× (1.32–1.39) | 2.59× (2.48–2.70) | 3.31× (3.22–3.40) | 3.37× (3.33–3.42) | 4.11× (3.92–4.31) | 7.76× (7.60–7.91) | 7.41× (7.28–7.55) | 2.35× (2.30–2.41) | 2.75× (2.63–2.87) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.79× (2.75–2.82) | 6.90× (6.79–7.01) | 7.24× (7.14–7.35) | 8.24× (8.14–8.34) | 7.36× (7.27–7.45) | 7.56× (7.46–7.65) | 7.16× (7.07–7.25) | 2.19× (2.16–2.22) | 2.51× (2.47–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.62× (2.59–2.65) | 6.53× (6.43–6.64) | 6.91× (6.75–7.06) | 8.08× (7.94–8.22) | 6.83× (6.71–6.95) | 6.87× (6.77–6.96) | 6.50× (6.41–6.60) | 1.99× (1.95–2.03) | 2.28× (2.23–2.34) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.66× (2.52–2.81) | 4.94× (4.68–5.21) | 5.36× (5.09–5.65) | 6.04× (5.73–6.38) | 5.35× (5.09–5.62) | 4.44× (4.20–4.69) | 4.20× (3.98–4.43) | 1.29× (1.22–1.36) | 1.48× (1.40–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.65) | 3.10× (3.04–3.17) | 3.93× (3.84–4.03) | 3.50× (3.40–3.61) | 6.08× (5.85–6.33) | 2.69× (2.62–2.77) | 2.67× (2.60–2.75) | 1.00× (0.98–1.02) | 1.87× (1.82–1.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.88× (2.76–3.00) | 5.04× (4.89–5.19) | 3.41× (3.34–3.49) | 5.78× (5.59–5.98) | 2.57× (2.50–2.64) | 2.55× (2.45–2.66) | 0.95× (0.92–0.99) | 1.78× (1.71–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.05× (2.99–3.12) | 5.03× (4.86–5.20) | 3.40× (3.33–3.47) | 5.84× (5.66–6.03) | 2.58× (2.50–2.67) | 2.57× (2.47–2.66) | 0.96× (0.92–0.99) | 1.79× (1.73–1.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.76× (2.70–2.81) | 3.16× (3.08–3.24) | 3.86× (3.55–4.19) | 4.39× (4.27–4.51) | 5.06× (4.95–5.17) | 2.98× (2.90–3.07) | 2.49× (2.46–2.51) | 2.07× (2.05–2.10) | 2.77× (2.71–2.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.22–2.36) | 2.74× (2.65–2.82) | 3.19× (3.09–3.30) | 3.55× (3.45–3.64) | 4.08× (4.00–4.16) | 2.28× (2.22–2.35) | 1.90× (1.86–1.94) | 1.59× (1.55–1.63) | 2.12× (2.07–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.85–1.98) | 1.71× (1.63–1.78) | 2.04× (1.96–2.13) | 2.04× (1.97–2.12) | 2.31× (2.23–2.39) | 1.12× (1.08–1.16) | 0.93× (0.90–0.97) | 0.78× (0.75–0.81) | 1.04× (1.00–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.36–1.42) | 3.04× (2.98–3.09) | 2.72× (2.65–2.80) | 2.76× (2.70–2.82) | 3.11× (3.04–3.19) | 3.51× (3.45–3.58) | 3.48× (3.42–3.54) | 1.37× (1.34–1.39) | 1.48× (1.46–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.18–1.32) | 2.72× (2.61–2.84) | 2.45× (2.20–2.72) | 3.11× (2.94–3.29) | 2.70× (2.58–2.82) | 2.94× (2.82–3.07) | 2.91× (2.78–3.05) | 1.14× (1.09–1.20) | 1.24× (1.18–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.38× (1.36–1.41) | 2.57× (2.50–2.64) | 2.08× (2.03–2.13) | 2.78× (2.72–2.85) | 2.59× (2.53–2.64) | 2.21× (2.15–2.29) | 2.19× (2.13–2.26) | 0.86× (0.83–0.89) | 0.93× (0.91–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.46–1.51) | 3.61× (3.55–3.68) | 3.01× (2.91–3.12) | 3.03× (3.00–3.06) | 3.47× (3.41–3.54) | 4.34× (4.28–4.41) | 4.66× (4.60–4.72) | 1.62× (1.59–1.65) | 1.79× (1.76–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.28× (1.21–1.36) | 3.18× (3.02–3.35) | 2.51× (2.39–2.65) | 3.53× (3.31–3.76) | 2.94× (2.76–3.13) | 3.60× (3.37–3.84) | 3.86× (3.62–4.12) | 1.34× (1.25–1.44) | 1.49× (1.40–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.50× (1.44–1.56) | 2.87× (2.79–2.95) | 2.19× (2.15–2.24) | 3.11× (3.03–3.19) | 2.80× (2.75–2.86) | 2.64× (2.58–2.71) | 2.83× (2.79–2.88) | 0.98× (0.96–1.01) | 1.09× (1.06–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.70) | 2.58× (2.50–2.66) | 3.60× (3.07–4.22) | 2.90× (2.83–2.97) | 6.39× (6.19–6.59) | 2.21× (2.15–2.27) | 2.23× (2.17–2.29) | 0.98× (0.95–1.02) | 1.51× (1.47–1.55) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.30× (2.24–2.37) | 3.95× (3.85–4.05) | 2.78× (2.72–2.84) | 5.87× (5.75–5.99) | 2.02× (1.97–2.08) | 2.03× (2.00–2.06) | 0.90× (0.87–0.92) | 1.38× (1.36–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.47× (2.39–2.54) | 3.93× (3.84–4.03) | 2.79× (2.75–2.84) | 5.99× (5.86–6.11) | 2.05× (2.01–2.10) | 2.07× (2.02–2.12) | 0.91× (0.89–0.94) | 1.40× (1.37–1.44) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.75× (1.66–1.86) | 4.06× (3.99–4.13) | 4.39× (4.30–4.48) | 4.63× (4.57–4.69) | 4.91× (4.86–4.96) | 4.96× (4.91–5.01) | 4.74× (4.69–4.79) | 1.52× (1.48–1.57) | 1.66× (1.64–1.69) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.43× (1.41–1.45) | 3.35× (3.27–3.42) | 3.68× (3.62–3.73) | 4.22× (4.15–4.29) | 4.08× (3.85–4.33) | 3.89× (3.83–3.95) | 3.71× (3.65–3.78) | 1.19× (1.17–1.22) | 1.30× (1.29–1.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.46× (1.38–1.55) | 2.58× (2.49–2.66) | 2.85× (2.77–2.93) | 3.23× (3.13–3.33) | 3.23× (3.05–3.43) | 2.49× (2.42–2.56) | 2.38× (2.31–2.45) | 0.76× (0.75–0.78) | 0.83× (0.82–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.79× (0.77–0.80) | 10.96× (10.93–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.46× (0.39–0.54) | 0.66× (0.63–0.68) | 0.56× (0.53–0.58) | 5.25× (5.05–5.46) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.32–0.34) | 0.49× (0.39–0.61) | 0.58× (0.56–0.61) | 0.51× (0.48–0.53) | 3.38× (3.28–3.49) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.15–0.17) | 1.76× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.45× (0.44–0.46) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.93–7.98) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.67–0.72) | 0.68× (0.64–0.72) | 1.61× (1.59–1.64) | 0.81× (0.80–0.82) | 5.85× (5.77–5.93) | 0.47× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.70× (0.67–0.73) | 1.13× (1.09–1.17) | 0.69× (0.66–0.72) | 3.30× (3.18–3.43) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.14× (5.14–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.39–0.44) | 0.91× (0.87–0.94) | 0.72× (0.68–0.75) | 3.15× (3.03–3.27) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.66–0.68) | 0.47× (0.46–0.48) | 0.90× (0.89–0.90) | 0.84× (0.83–0.86) | 2.61× (2.59–2.64) | 0.16× (0.16–0.16) | 0.17× (0.17–0.17) | 0.25× (0.25–0.26) | 0.13× (0.13–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.17× (5.15–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.39× (0.37–0.41) | 0.84× (0.80–0.87) | 0.72× (0.68–0.76) | 2.89× (2.76–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.51) | 0.79× (0.79–0.80) | 0.79× (0.77–0.81) | 2.29× (2.27–2.31) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.07) | 0.77× (0.76–0.77) | 5.33× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.77× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.67× (0.67–0.67) | 6.33× (6.32–6.34) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.45) | 0.38× (0.34–0.41) | 1.03× (1.01–1.05) | 0.66× (0.65–0.68) | 3.29× (3.27–3.31) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.50× (0.48–0.51) | 0.42× (0.38–0.47) | 0.90× (0.87–0.92) | 0.74× (0.72–0.76) | 2.28× (2.23–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 704 | 1.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,206 | 1.56 | 1.10× (1.08–1.12) | 84% | 1.04× (1.03–1.05) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,435 | 2.40 | 1.70× (1.63–1.78) | 49% | 1.22× (1.20–1.24) | 1.62× (1.59–1.66) |

**Best:** mibz2 at 4 threads (1,435 MB/s); lbzip2 at 4 threads (327 MB/s); pbz2 at 4 threads (530 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.90, mibz2 4t 1.26, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.36 | 0.01 | 0.07 | 0.146 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4321 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.7% |
| MTF + RLE2 | 1.45 | 20.8% |
| T-vector build | 1.32 | 19.0% |
| walk (the one each build uses) | 2.78 | 39.8% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.26 | 3.7% |
| **total** | 6.98 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.298 | 0.719 | (in its total) | 2.43× (2.39–2.48) |  |
| walk | 0.616 | 1.600 | (in its total) | 2.62× (2.57–2.67) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.030 | (in its total) | 1.17× (1.15–1.20) |  |
| inverse BWT, total | 0.940 | 2.343 | 1.316 | 2.52× (2.48–2.57) | 1.41× (1.39–1.42) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m19s ± 4m57s |
| 4 threads | 21m56s ± 3m18s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m28s ± 2m05s |
| the scanner alone (its serial part) | 8m26s ± 57.4s |
| the workers alone, over 4 | 13m27s ± 2m05s |

At 4 threads the workers bound it. Per byte: the scanner 4.9 ns a byte of bz2, the workers 1.76 ns a byte of output at 4 threads (CPU, summed over them), 1.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 6 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,147 | 13,382,859 | \`f7d4e4f1aa08d971…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4321.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
