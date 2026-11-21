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

The latest record: [`90e6719`](<https://github.com/lmmx/mibz2/commit/90e6719663af0615fb417b038906ad2636193aa3>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44955815785>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.317 s | 663 | 6.5 | 2.85× (2.77–2.93) | 7.01× (6.82–7.21) | 7.33× (7.13–7.54) | 8.32× (8.13–8.52) | 7.38× (7.22–7.55) | 7.66× (7.59–7.74) | 7.28× (7.15–7.41) | 2.20× (2.15–2.25) | 2.52× (2.46–2.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.197 s | 1,065 | 7.2 | 2.68× (2.47–2.92) | 5.93× (5.49–6.40) | 7.01× (6.56–7.49) | 7.17× (6.65–7.74) | 6.33× (5.85–6.85) | 12.19× (11.32–13.13) | 11.57× (10.79–12.41) | 3.50× (3.27–3.74) | 4.01× (3.73–4.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.159 s | 1,320 | 11.0 | 2.84× (2.75–2.94) | 4.42× (4.26–4.59) | 5.88× (5.67–6.09) | 5.39× (5.24–5.54) | 5.38× (5.26–5.51) | 15.39× (14.99–15.80) | 14.61× (14.25–14.98) | 4.41× (4.31–4.52) | 5.06× (4.96–5.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,347 | 3.5 | 0.57× (0.56–0.59) | 3.42× (3.35–3.49) | 4.58× (4.35–4.83) | 3.79× (3.53–4.08) | 6.56× (6.36–6.75) | 2.78× (2.73–2.82) | 2.77× (2.73–2.82) | 1.05× (0.95–1.17) | 2.06× (2.02–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,333 | 3.6 | 0.52× (0.51–0.53) | 3.26× (3.08–3.45) | 5.89× (5.67–6.12) | 3.68× (3.62–3.74) | 6.40× (6.19–6.61) | 2.76× (2.72–2.81) | 2.76× (2.71–2.81) | 1.05× (0.94–1.17) | 2.05× (2.01–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,246 | 3.7 | 0.46× (0.37–0.56) | 2.78× (2.15–3.60) | 5.18× (4.12–6.52) | 3.22× (2.60–4.00) | 5.67× (4.58–7.01) | 2.41× (1.94–2.98) | 2.40× (1.94–2.97) | 0.91× (0.72–1.16) | 1.78× (1.44–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,021 | 4.5 | 2.97× (2.93–3.00) | 3.70× (3.65–3.75) | 4.39× (4.16–4.63) | 4.77× (4.53–5.03) | 5.51× (5.24–5.80) | 3.03× (2.94–3.11) | 2.59× (2.55–2.62) | 2.16× (2.13–2.19) | 3.21× (3.08–3.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,311 | 6.1 | 2.21× (1.95–2.50) | 2.76× (2.44–3.13) | 4.10× (3.61–4.65) | 3.50× (3.15–3.88) | 5.56× (5.00–6.18) | 3.84× (3.48–4.24) | 3.28× (2.95–3.64) | 2.74× (2.47–3.05) | 4.07× (3.62–4.58) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,117 | 12.3 | 1.60× (1.49–1.73) | 1.79× (1.61–1.99) | 2.64× (2.39–2.91) | 2.01× (1.83–2.22) | 3.96× (3.71–4.23) | 3.20× (3.01–3.39) | 2.73× (2.57–2.90) | 2.28× (2.15–2.43) | 3.39× (3.15–3.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 97 | 45.1 | 1.42× (1.39–1.45) | 3.21× (3.09–3.34) | 2.87× (2.79–2.94) | 2.83× (2.71–2.94) | 3.11× (3.05–3.18) | 3.56× (3.50–3.62) | 3.55× (3.50–3.60) | 1.36× (1.34–1.39) | 1.58× (1.55–1.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 153 | 52.2 | 1.30× (1.25–1.36) | 2.88× (2.70–3.06) | 2.66× (2.58–2.74) | 3.31× (3.18–3.44) | 3.11× (3.01–3.21) | 5.57× (5.36–5.78) | 5.56× (5.38–5.73) | 2.13× (2.06–2.21) | 2.47× (2.37–2.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 180 | 69.8 | 1.27× (1.22–1.32) | 2.45× (2.24–2.68) | 2.41× (2.29–2.55) | 2.80× (2.69–2.92) | 3.12× (3.00–3.24) | 6.66× (6.40–6.92) | 6.64× (6.43–6.86) | 2.55× (2.47–2.64) | 2.96× (2.83–3.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.088 s | 110 | 39.6 | 1.51× (1.48–1.54) | 3.75× (3.66–3.84) | 3.08× (3.02–3.14) | 3.09× (3.05–3.14) | 3.51× (3.45–3.58) | 4.52× (4.38–4.66) | 4.84× (4.73–4.94) | 1.60× (1.59–1.62) | 1.84× (1.81–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.058 s | 164 | 47.5 | 1.23× (1.14–1.33) | 3.12× (2.79–3.50) | 2.60× (2.38–2.84) | 3.59× (3.30–3.91) | 3.15× (2.88–3.45) | 6.60× (5.96–7.30) | 7.06× (6.37–7.83) | 2.34× (2.13–2.58) | 2.68× (2.46–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.049 s | 195 | 65.4 | 1.36× (1.31–1.40) | 2.60× (2.50–2.71) | 2.51× (2.33–2.71) | 3.25× (3.15–3.35) | 3.09× (3.00–3.18) | 8.14× (7.71–8.59) | 8.71× (8.34–9.09) | 2.89× (2.77–3.01) | 3.31× (3.21–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,005 | 4.9 | 0.62× (0.61–0.63) | 3.10× (3.04–3.16) | 4.48× (3.95–5.09) | 3.17× (3.10–3.24) | 7.45× (6.84–8.12) | 2.33× (2.28–2.39) | 2.35× (2.29–2.40) | 1.07× (0.89–1.27) | 2.07× (2.03–2.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 952 | 5.3 | 0.48× (0.41–0.55) | 2.87× (2.43–3.39) | 4.86× (4.18–5.66) | 3.02× (2.50–3.65) | 6.53× (5.98–7.12) | 2.09× (1.80–2.43) | 2.11× (1.82–2.44) | 0.96× (0.91–1.00) | 1.86× (1.61–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 935 | 5.2 | 0.51× (0.49–0.53) | 2.89× (2.84–2.94) | 5.21× (4.57–5.95) | 2.97× (2.92–3.02) | 6.62× (6.44–6.80) | 2.16× (2.13–2.20) | 2.18× (2.13–2.22) | 0.99× (0.83–1.19) | 1.92× (1.89–1.96) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 0.046 s | 291 | 15.1 | 1.74× (1.71–1.78) | 4.16× (4.04–4.29) | 4.54× (4.43–4.65) | 4.67× (4.56–4.78) | 5.20× (5.00–5.41) | 5.05× (4.94–5.16) | 4.89× (4.84–4.95) | 1.51× (1.47–1.55) | 1.76× (1.70–1.82) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 0.035 s | 388 | 19.6 | 1.39× (1.21–1.58) | 3.25× (3.07–3.45) | 3.83× (3.63–4.05) | 4.08× (3.75–4.45) | 4.27× (4.07–4.49) | 6.65× (6.15–7.19) | 6.45× (5.90–7.04) | 1.99× (1.83–2.16) | 2.31× (2.12–2.52) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 0.032 s | 414 | 30.1 | 1.21× (1.14–1.29) | 2.34× (2.08–2.64) | 3.01× (2.77–3.27) | 3.05× (2.84–3.27) | 3.72× (3.33–4.15) | 7.05× (6.64–7.48) | 6.83× (6.42–7.27) | 2.11× (1.97–2.25) | 2.45× (2.28–2.63) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.84× (2.76–2.92) | 6.97× (6.78–7.16) | 7.28× (7.08–7.49) | 8.29× (8.09–8.48) | 7.35× (7.19–7.52) | 7.63× (7.55–7.71) | 7.24× (7.12–7.37) | 2.19× (2.14–2.24) | 2.50× (2.44–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.56× (2.40–2.73) | 6.24× (5.88–6.63) | 6.59× (6.22–6.98) | 7.61× (7.16–8.10) | 6.44× (6.06–6.85) | 6.62× (6.24–7.03) | 6.29× (5.95–6.65) | 1.90× (1.81–2.01) | 2.17× (2.05–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.78× (2.70–2.87) | 5.02× (4.89–5.17) | 5.68× (5.48–5.87) | 6.18× (6.01–6.36) | 5.32× (5.24–5.41) | 4.52× (4.45–4.60) | 4.30× (4.24–4.36) | 1.30× (1.28–1.32) | 1.48× (1.45–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.63) | 3.05× (2.98–3.11) | 3.86× (3.77–3.95) | 3.52× (3.29–3.77) | 5.99× (5.82–6.16) | 2.60× (2.55–2.64) | 2.59× (2.56–2.63) | 1.06× (0.95–1.19) | 1.84× (1.81–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.83× (2.79–2.88) | 5.08× (4.91–5.26) | 3.37× (3.32–3.42) | 5.77× (5.57–5.98) | 2.54× (2.51–2.57) | 2.54× (2.49–2.58) | 1.04× (0.92–1.17) | 1.80× (1.77–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.50× (0.41–0.61) | 2.72× (2.21–3.35) | 4.50× (3.66–5.52) | 3.06× (2.49–3.76) | 5.26× (4.29–6.46) | 2.29× (1.86–2.82) | 2.29× (1.86–2.81) | 0.94× (0.74–1.19) | 1.63× (1.33–1.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.81× (2.77–2.84) | 3.17× (3.13–3.21) | 3.90× (3.65–4.16) | 4.46× (4.24–4.70) | 5.15× (4.89–5.43) | 2.86× (2.79–2.93) | 2.46× (2.43–2.49) | 2.07× (2.04–2.09) | 2.84× (2.73–2.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.17–2.41) | 2.56× (2.42–2.71) | 3.08× (2.95–3.22) | 3.38× (3.23–3.54) | 4.03× (3.66–4.44) | 2.10× (2.01–2.21) | 1.81× (1.72–1.90) | 1.52× (1.45–1.59) | 2.09× (1.98–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.85× (1.77–1.94) | 1.73× (1.60–1.87) | 2.05× (1.99–2.12) | 2.01× (1.92–2.10) | 2.21× (2.14–2.28) | 1.07× (1.03–1.11) | 0.92× (0.89–0.95) | 0.77× (0.75–0.80) | 1.06× (1.03–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.41× (1.38–1.44) | 3.10× (2.98–3.23) | 2.77× (2.72–2.83) | 2.79× (2.68–2.90) | 3.07× (3.00–3.13) | 3.50× (3.44–3.57) | 3.50× (3.45–3.55) | 1.36× (1.33–1.38) | 1.49× (1.47–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.32) | 2.78× (2.71–2.84) | 2.39× (2.35–2.43) | 3.12× (3.05–3.19) | 2.69× (2.64–2.75) | 3.02× (2.94–3.10) | 3.02× (2.95–3.08) | 1.17× (1.14–1.20) | 1.28× (1.25–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.38–1.45) | 2.72× (2.50–2.96) | 2.13× (2.07–2.20) | 2.78× (2.68–2.87) | 2.64× (2.58–2.71) | 2.26× (2.19–2.33) | 2.25× (2.21–2.31) | 0.87× (0.85–0.90) | 0.96× (0.93–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.47–1.53) | 3.65× (3.57–3.74) | 2.99× (2.94–3.03) | 3.05× (3.01–3.10) | 3.46× (3.40–3.53) | 4.45× (4.31–4.59) | 4.76× (4.66–4.87) | 1.59× (1.57–1.61) | 1.77× (1.74–1.81) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.28× (1.19–1.38) | 3.07× (2.82–3.34) | 2.45× (2.26–2.65) | 3.42× (3.16–3.71) | 2.88× (2.64–3.14) | 3.61× (3.28–3.97) | 3.86× (3.48–4.29) | 1.29× (1.18–1.42) | 1.44× (1.31–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.56× (1.52–1.59) | 2.95× (2.85–3.05) | 2.23× (2.12–2.35) | 3.09× (2.98–3.20) | 2.80× (2.74–2.87) | 2.71× (2.58–2.84) | 2.90× (2.74–3.06) | 0.97× (0.93–1.01) | 1.08× (1.04–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.69) | 2.57× (2.50–2.64) | 3.52× (3.13–3.96) | 2.85× (2.80–2.91) | 6.52× (6.00–7.09) | 2.13× (2.09–2.18) | 2.15× (2.10–2.20) | 1.07× (0.90–1.28) | 1.50× (1.47–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.48–0.65) | 2.29× (1.90–2.76) | 3.76× (3.24–4.36) | 2.75× (2.25–3.37) | 5.65× (5.22–6.12) | 1.88× (1.62–2.18) | 1.90× (1.64–2.20) | 0.95× (0.91–0.99) | 1.33× (1.14–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.60) | 2.48× (2.45–2.52) | 4.28× (3.73–4.90) | 2.79× (2.75–2.84) | 5.93× (5.77–6.10) | 2.02× (1.99–2.05) | 2.04× (2.00–2.08) | 1.02× (0.85–1.22) | 1.43× (1.40–1.45) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 1.72× (1.68–1.76) | 4.01× (3.91–4.10) | 4.32× (4.23–4.41) | 4.54× (4.44–4.64) | 5.06× (4.86–5.26) | 4.91× (4.80–5.01) | 4.76× (4.71–4.81) | 1.49× (1.46–1.53) | 1.65× (1.59–1.71) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 1.49× (1.28–1.73) | 3.26× (3.10–3.44) | 3.54× (3.35–3.74) | 4.00× (3.66–4.37) | 3.82× (3.65–3.98) | 3.72× (3.42–4.05) | 3.61× (3.29–3.96) | 1.13× (1.04–1.24) | 1.25× (1.14–1.37) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 1.48× (1.45–1.51) | 2.68× (2.45–2.92) | 2.94× (2.81–3.06) | 3.31× (3.19–3.43) | 3.21× (2.91–3.53) | 2.48× (2.42–2.54) | 2.41× (2.33–2.49) | 0.76× (0.73–0.78) | 0.83× (0.80–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.34× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.80) | 10.95× (10.94–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.30–0.33) | 0.46× (0.40–0.53) | 0.65× (0.61–0.68) | 0.55× (0.52–0.58) | 5.21× (4.95–5.49) | 0.12× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.32× (0.31–0.34) | 0.47× (0.38–0.59) | 0.58× (0.56–0.60) | 0.49× (0.47–0.52) | 3.34× (3.22–3.47) | 0.08× (0.07–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.95× (7.93–7.98) | 0.64× (0.64–0.64) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.62–0.71) | 0.64× (0.59–0.69) | 1.58× (1.49–1.67) | 0.80× (0.76–0.84) | 5.72× (5.41–6.05) | 0.46× (0.43–0.49) | 0.49× (0.46–0.52) | 0.53× (0.50–0.56) | 0.26× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.59–0.66) | 0.64× (0.60–0.69) | 1.11× (1.05–1.17) | 0.68× (0.65–0.72) | 3.23× (3.07–3.39) | 0.25× (0.24–0.26) | 0.27× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.73× (0.72–0.73) | 5.16× (5.14–5.18) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.50× (0.48–0.52) | 0.40× (0.38–0.43) | 0.89× (0.85–0.92) | 0.71× (0.69–0.74) | 3.07× (2.95–3.19) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.46–0.47) | 0.88× (0.87–0.90) | 0.83× (0.82–0.85) | 2.57× (2.53–2.62) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.19) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.50) | 0.38× (0.36–0.40) | 0.85× (0.81–0.88) | 0.71× (0.66–0.76) | 2.91× (2.79–3.04) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.46× (0.44–0.48) | 0.81× (0.79–0.82) | 0.81× (0.79–0.83) | 2.33× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.06–1.07) | 0.77× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.76× (0.76–0.77) | 5.28× (5.26–5.29) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.28–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.67× (0.66–0.67) | 6.33× (6.30–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.43× (0.39–0.47) | 1.11× (1.09–1.13) | 0.73× (0.71–0.75) | 3.61× (3.54–3.68) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 55 MB | 0.51× (0.48–0.53) | 0.44× (0.40–0.48) | 0.93× (0.91–0.95) | 0.75× (0.72–0.79) | 2.38× (2.32–2.44) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 663 | 1.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,065 | 1.71 | 1.15× (1.08–1.22) | 80% | 1.03× (1.00–1.06) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,320 | 2.59 | 1.69× (1.65–1.73) | 50% | 1.22× (1.19–1.25) | 1.65× (1.61–1.69) |

**Best:** mibz2 at 4 threads (1,320 MB/s); lbzip2 at 4 threads (301 MB/s); pbz2 at 4 threads (462 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.27, lbzip2 1t 0.07, lbzip2 2t 0.31, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.59. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.40 | 0.01 | 0.07 | 0.159 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4249 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 14.7% |
| MTF + RLE2 | 1.43 | 20.5% |
| T-vector build | 1.29 | 18.6% |
| walk (the one each build uses) | 2.82 | 40.4% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.6% |
| **total** | 6.97 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.302 | 0.723 | (in its total) | 2.42× (2.38–2.47) |  |
| walk | 0.625 | 1.609 | (in its total) | 2.61× (2.55–2.67) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.030 | (in its total) | 1.17× (1.13–1.20) |  |
| inverse BWT, total | 0.955 | 2.362 | 1.325 | 2.51× (2.46–2.56) | 1.40× (1.37–1.42) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 46m19s ± 5m22s |
| 4 threads | 23m04s ± 3m16s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m35s ± 2m04s |
| the scanner alone (its serial part) | 8m44s ± 29.4s |
| the workers alone, over 4 | 14m35s ± 2m04s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 1.91 ns a byte of output at 4 threads (CPU, summed over them), 1.23 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,640,714 | 13,421,331 | \`8da9d679141c517c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4249.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
