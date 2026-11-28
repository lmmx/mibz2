# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1348/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`60db333`](<https://github.com/lmmx/mibz2/commit/60db33374e26c826573294f03c9bcd409a935f90>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46132994113>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.464 s | 452 | 1.8 | 3.37× (3.28–3.45) | 7.55× (7.36–7.73) | 8.02× (7.85–8.19) | 9.03× (8.81–9.26) | 8.04× (7.85–8.23) | 8.58× (8.38–8.79) | 8.32× (8.11–8.54) | 2.61× (2.55–2.68) | 2.42× (2.35–2.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.267 s | 786 | 1.9 | 3.29× (3.14–3.45) | 7.05× (6.78–7.32) | 8.41× (8.07–8.77) | 8.60× (8.26–8.94) | 7.61× (7.32–7.90) | 15.13× (14.55–15.73) | 14.67× (14.09–15.28) | 4.60× (4.44–4.78) | 4.26× (4.09–4.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.213 s | 986 | 3.0 | 3.64× (3.60–3.69) | 6.36× (6.23–6.50) | 7.27× (7.10–7.44) | 7.23× (7.12–7.33) | 7.09× (6.99–7.20) | 18.88× (18.57–19.21) | 18.31× (18.00–18.62) | 5.75× (5.65–5.85) | 5.32× (5.20–5.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 968 | 0.9 | 0.58× (0.58–0.59) | 3.45× (3.40–3.50) | 4.38× (4.32–4.44) | 3.78× (3.72–3.84) | 6.21× (6.11–6.31) | 2.80× (2.76–2.85) | 2.80× (2.76–2.84) | 1.14× (1.11–1.16) | 1.00× (0.98–1.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 911 | 1.0 | 0.52× (0.51–0.53) | 2.80× (2.74–2.86) | 5.54× (5.42–5.67) | 3.58× (3.50–3.67) | 5.88× (5.77–5.98) | 2.65× (2.59–2.71) | 2.64× (2.59–2.70) | 1.07× (1.06–1.09) | 0.94× (0.92–0.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 903 | 1.0 | 0.51× (0.49–0.53) | 2.73× (2.63–2.84) | 5.39× (5.18–5.61) | 3.49× (3.36–3.63) | 5.77× (5.55–6.01) | 2.58× (2.48–2.67) | 2.57× (2.48–2.67) | 1.04× (1.01–1.08) | 0.92× (0.88–0.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 536 | 1.6 | 2.91× (2.89–2.93) | 3.60× (3.57–3.62) | 4.12× (4.09–4.15) | 4.56× (4.53–4.59) | 4.99× (4.93–5.04) | 3.74× (3.69–3.80) | 3.09× (3.00–3.18) | 2.15× (2.13–2.17) | 2.21× (2.20–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 858 | 1.8 | 2.46× (2.32–2.61) | 3.11× (2.91–3.32) | 4.55× (4.23–4.90) | 4.03× (3.77–4.30) | 5.47× (5.16–5.79) | 5.87× (5.54–6.23) | 4.85× (4.55–5.18) | 3.37× (3.16–3.59) | 3.48× (3.28–3.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 893 | 3.3 | 2.28× (2.21–2.35) | 2.34× (2.25–2.44) | 3.37× (3.24–3.50) | 2.68× (2.57–2.79) | 4.69× (4.51–4.89) | 6.14× (5.90–6.38) | 5.07× (4.89–5.25) | 3.52× (3.39–3.66) | 3.63× (3.50–3.76) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 61 | 13.2 | 1.50× (1.49–1.51) | 3.00× (2.98–3.02) | 2.60× (2.57–2.63) | 2.72× (2.71–2.73) | 3.06× (3.05–3.07) | 3.47× (3.46–3.49) | 3.64× (3.62–3.66) | 1.43× (1.41–1.46) | 1.46× (1.44–1.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.076 s | 105 | 14.3 | 1.46× (1.44–1.48) | 2.97× (2.92–3.03) | 2.76× (2.72–2.81) | 3.58× (3.50–3.65) | 3.26× (3.19–3.33) | 5.90× (5.79–6.01) | 6.19× (6.08–6.31) | 2.44× (2.38–2.50) | 2.48× (2.42–2.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.063 s | 128 | 19.2 | 1.61× (1.56–1.65) | 2.81× (2.74–2.88) | 2.68× (2.60–2.76) | 3.36× (3.27–3.46) | 3.34× (3.25–3.42) | 7.23× (7.04–7.42) | 7.59× (7.41–7.77) | 2.98× (2.90–3.07) | 3.04× (2.94–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 12.3 | 1.51× (1.44–1.59) | 3.35× (3.17–3.53) | 2.73× (2.60–2.88) | 2.90× (2.76–3.06) | 3.25× (3.08–3.44) | 4.12× (3.92–4.34) | 4.44× (4.20–4.69) | 1.63× (1.56–1.71) | 1.67× (1.58–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 13.3 | 1.50× (1.39–1.62) | 3.26× (3.10–3.43) | 2.88× (2.74–3.02) | 4.15× (4.03–4.27) | 3.42× (3.27–3.59) | 7.24× (6.90–7.60) | 7.80× (7.44–8.17) | 2.86× (2.72–3.01) | 2.92× (2.79–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 140 | 18.7 | 1.70× (1.64–1.77) | 3.14× (3.08–3.20) | 2.81× (2.72–2.90) | 3.81× (3.71–3.91) | 3.41× (3.33–3.48) | 8.86× (8.67–9.05) | 9.54× (9.33–9.75) | 3.50× (3.43–3.58) | 3.58× (3.50–3.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 1.2 | 0.63× (0.62–0.65) | 3.02× (2.96–3.07) | 4.44× (4.36–4.53) | 3.28× (3.22–3.34) | 6.79× (6.66–6.93) | 2.42× (2.38–2.47) | 2.45× (2.41–2.49) | 1.03× (1.01–1.06) | 1.52× (1.49–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 1.4 | 0.51× (0.50–0.52) | 2.76× (2.71–2.81) | 4.76× (4.68–4.84) | 3.02× (2.97–3.07) | 6.26× (6.14–6.38) | 2.21× (2.16–2.25) | 2.23× (2.20–2.26) | 0.94× (0.91–0.97) | 1.38× (1.36–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 669 | 1.4 | 0.52× (0.51–0.52) | 2.75× (2.71–2.79) | 4.72× (4.67–4.78) | 3.01× (2.97–3.06) | 6.19× (6.11–6.28) | 2.20× (2.17–2.23) | 2.22× (2.19–2.25) | 0.94× (0.92–0.95) | 1.38× (1.36–1.40) | yes |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 1 | 0.067 s | 226 | 3.6 | 2.08× (2.06–2.09) | 4.25× (4.22–4.28) | 4.86× (4.80–4.91) | 5.02× (4.99–5.06) | 5.10× (5.05–5.15) | 5.16× (5.13–5.20) | 5.22× (5.17–5.27) | 1.71× (1.69–1.72) | 1.67× (1.66–1.68) | yes |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 2 | 0.043 s | 347 | 4.4 | 1.72× (1.68–1.77) | 3.61× (3.45–3.78) | 4.74× (4.59–4.91) | 5.05× (4.88–5.24) | 4.71× (4.58–4.85) | 7.78× (7.57–8.00) | 7.87× (7.65–8.10) | 2.57× (2.50–2.65) | 2.52× (2.45–2.59) | yes |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 4 | 0.038 s | 393 | 6.8 | 1.82× (1.75–1.90) | 3.08× (2.95–3.22) | 4.15× (4.01–4.29) | 4.40× (4.24–4.56) | 4.49× (4.38–4.61) | 8.98× (8.74–9.23) | 9.09× (8.84–9.34) | 2.97× (2.90–3.04) | 2.91× (2.83–2.98) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.36× (3.27–3.44) | 7.51× (7.33–7.69) | 7.99× (7.82–8.16) | 9.00× (8.78–9.23) | 8.01× (7.83–8.20) | 8.55× (8.36–8.76) | 8.29× (8.08–8.51) | 2.61× (2.54–2.67) | 2.40× (2.34–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.24× (3.19–3.30) | 7.29× (7.19–7.39) | 8.03× (7.89–8.17) | 8.98× (8.84–9.12) | 7.63× (7.51–7.75) | 7.99× (7.87–8.12) | 7.75× (7.62–7.88) | 2.43× (2.40–2.47) | 2.24× (2.20–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.46× (3.42–3.51) | 6.62× (6.51–6.74) | 6.85× (6.74–6.96) | 7.52× (7.44–7.61) | 6.68× (6.61–6.76) | 5.02× (4.93–5.10) | 4.87× (4.79–4.94) | 1.53× (1.50–1.55) | 1.41× (1.38–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 2.88× (2.84–2.92) | 3.94× (3.88–4.00) | 3.57× (3.52–3.62) | 5.82× (5.73–5.92) | 2.67× (2.63–2.71) | 2.67× (2.62–2.71) | 1.13× (1.10–1.15) | 0.91× (0.89–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.68× (2.62–2.73) | 4.86× (4.77–4.95) | 3.33× (3.26–3.40) | 5.40× (5.30–5.50) | 2.47× (2.41–2.52) | 2.46× (2.41–2.51) | 1.04× (1.03–1.05) | 0.84× (0.82–0.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.52–0.56) | 3.05× (2.95–3.15) | 4.81× (4.69–4.94) | 3.29× (3.18–3.40) | 5.36× (5.16–5.56) | 2.43× (2.35–2.51) | 2.43× (2.35–2.51) | 1.02× (0.99–1.06) | 0.82× (0.80–0.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.82× (2.81–2.84) | 3.23× (3.21–3.25) | 3.75× (3.73–3.77) | 4.40× (4.37–4.43) | 4.81× (4.75–4.86) | 3.62× (3.57–3.68) | 3.00× (2.91–3.09) | 2.09× (2.07–2.12) | 2.01× (1.99–2.02) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.60× (2.55–2.65) | 2.97× (2.92–3.03) | 3.53× (3.45–3.61) | 3.90× (3.82–3.97) | 4.26× (4.18–4.35) | 3.13× (3.05–3.21) | 2.59× (2.52–2.66) | 1.81× (1.76–1.86) | 1.73× (1.70–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.33× (2.29–2.37) | 2.03× (1.96–2.11) | 2.48× (2.38–2.59) | 2.45× (2.36–2.53) | 2.58× (2.48–2.67) | 1.74× (1.67–1.81) | 1.44× (1.38–1.51) | 1.01× (0.97–1.04) | 0.96× (0.93–1.00) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 2.94× (2.92–2.96) | 2.56× (2.54–2.58) | 2.70× (2.69–2.71) | 3.03× (3.02–3.05) | 3.44× (3.42–3.45) | 3.61× (3.59–3.63) | 1.43× (1.41–1.45) | 1.40× (1.39–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.45× (1.44–1.46) | 2.90× (2.87–2.92) | 2.50× (2.49–2.52) | 3.36× (3.34–3.39) | 2.89× (2.86–2.91) | 3.18× (3.16–3.20) | 3.33× (3.30–3.37) | 1.32× (1.30–1.34) | 1.29× (1.28–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.77× (1.72–1.82) | 2.99× (2.88–3.10) | 2.47× (2.37–2.57) | 3.31× (3.19–3.43) | 2.91× (2.83–2.99) | 2.37× (2.30–2.44) | 2.48× (2.42–2.55) | 0.98× (0.95–1.02) | 0.96× (0.93–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.43–1.59) | 3.28× (3.11–3.46) | 2.69× (2.56–2.84) | 2.88× (2.74–3.03) | 3.23× (3.06–3.41) | 4.09× (3.88–4.30) | 4.40× (4.17–4.64) | 1.62× (1.55–1.70) | 1.63× (1.56–1.71) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.51× (1.43–1.59) | 3.26× (3.22–3.31) | 2.67× (2.64–2.70) | 3.77× (3.74–3.80) | 3.10× (3.07–3.13) | 3.82× (3.78–3.86) | 4.11× (4.08–4.15) | 1.52× (1.50–1.53) | 1.53× (1.51–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.80× (1.75–1.85) | 3.34× (3.29–3.39) | 2.49× (2.43–2.54) | 3.52× (3.43–3.61) | 3.01× (2.96–3.06) | 2.74× (2.70–2.78) | 2.95× (2.91–2.99) | 1.09× (1.07–1.10) | 1.09× (1.08–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.69) | 2.43× (2.39–2.48) | 3.48× (3.41–3.55) | 3.04× (2.99–3.09) | 6.17× (6.06–6.30) | 2.27× (2.24–2.31) | 2.30× (2.26–2.33) | 1.03× (1.01–1.05) | 0.84× (0.82–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.15× (2.12–2.18) | 3.98× (3.93–4.03) | 2.76× (2.72–2.80) | 5.52× (5.42–5.61) | 2.00× (1.97–2.03) | 2.02× (2.00–2.05) | 0.91× (0.89–0.93) | 0.74× (0.72–0.75) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.55) | 2.51× (2.44–2.57) | 3.96× (3.90–4.03) | 2.81× (2.77–2.86) | 5.57× (5.47–5.66) | 2.03× (2.00–2.06) | 2.05× (2.02–2.08) | 0.92× (0.91–0.93) | 0.75× (0.73–0.76) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 1 | 2.05× (2.03–2.07) | 4.10× (4.07–4.14) | 4.66× (4.62–4.71) | 4.92× (4.89–4.96) | 5.00× (4.95–5.05) | 5.06× (5.03–5.09) | 5.12× (5.07–5.17) | 1.69× (1.68–1.70) | 1.58× (1.57–1.59) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 2 | 1.79× (1.76–1.82) | 3.59× (3.48–3.70) | 4.21× (4.15–4.26) | 4.63× (4.56–4.70) | 4.22× (4.16–4.27) | 4.17× (4.11–4.23) | 4.22× (4.15–4.28) | 1.39× (1.37–1.41) | 1.30× (1.28–1.32) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 4 | 1.90× (1.83–1.98) | 3.11× (3.03–3.18) | 3.50× (3.43–3.57) | 3.83× (3.74–3.91) | 3.52× (3.46–3.59) | 2.70× (2.65–2.74) | 2.73× (2.68–2.78) | 0.90× (0.89–0.91) | 0.84× (0.83–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.98–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.44× (0.38–0.50) | 0.72× (0.69–0.74) | 0.61× (0.60–0.63) | 5.76× (5.58–5.95) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.37× (0.35–0.39) | 0.52× (0.45–0.60) | 0.65× (0.62–0.68) | 0.57× (0.54–0.61) | 3.86× (3.68–4.05) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.95× (1.95–1.95) | 0.92× (0.92–0.92) | 4.83× (4.83–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.91–0.93) | 4.83× (4.81–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.41–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.95× (7.94–7.96) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.63× (0.57–0.68) | 1.68× (1.63–1.74) | 0.84× (0.82–0.87) | 6.13× (5.95–6.32) | 0.50× (0.48–0.51) | 0.52× (0.51–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.71) | 0.75× (0.69–0.81) | 1.23× (1.20–1.27) | 0.76× (0.74–0.79) | 3.69× (3.56–3.82) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.74) | 5.16× (5.13–5.18) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.48× (0.47–0.50) | 0.40× (0.37–0.43) | 0.88× (0.85–0.91) | 0.68× (0.65–0.71) | 3.05× (2.95–3.15) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.81× (0.78–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.45–0.48) | 0.37× (0.35–0.39) | 0.82× (0.79–0.85) | 0.68× (0.66–0.71) | 2.81× (2.73–2.90) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.59× (0.57–0.61) | 0.48× (0.44–0.52) | 0.78× (0.77–0.80) | 0.79× (0.76–0.82) | 2.27× (2.22–2.31) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.19× (6.17–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.85× (1.85–1.85) | 0.89× (0.88–0.89) | 6.15× (6.13–6.16) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.39) | 0.66× (0.65–0.66) | 6.37× (6.35–6.38) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 2 | 34 MB | 0.48× (0.47–0.50) | 0.41× (0.38–0.45) | 1.10× (1.06–1.14) | 0.73× (0.70–0.76) | 3.64× (3.53–3.75) | 0.22× (0.21–0.23) | 0.24× (0.23–0.24) | 0.36× (0.35–0.37) | 0.17× (0.17–0.18) |
| `text` (2.27 MB → 15.1 MB, 6.7×) | 4 | 54 MB | 0.52× (0.49–0.55) | 0.45× (0.41–0.49) | 0.94× (0.91–0.97) | 0.76× (0.73–0.79) | 2.41× (2.34–2.49) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 452 | 2.22 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 786 | 2.38 | 1.07× (1.06–1.08) | 88% | 1.04× (1.03–1.05) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 986 | 3.79 | 1.70× (1.66–1.75) | 55% | 1.50× (1.47–1.54) | 1.76× (1.73–1.79) |

**Best:** mibz2 at 4 threads (986 MB/s); lbzip2 at 4 threads (156 MB/s); pbz2 at 4 threads (271 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.02, lbzip2 1t 0.04, lbzip2 2t 0.21, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.63 | 0.02 | 0.09 | 0.213 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.20 | 8.0% |
| MTF + RLE2 | 0.42 | 16.6% |
| T-vector build | 0.41 | 16.0% |
| walk (the one each build uses) | 1.42 | 55.4% |
| of which: segment assembly | 0.07 | 2.9% |
| RLE1 | 0.07 | 2.7% |
| CRC | 0.04 | 1.4% |
| **total** | 2.56 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.501 | 1.307 | (in its total) | 2.61× (2.59–2.62) |  |
| walk | 1.656 | 3.119 | (in its total) | 1.88× (1.87–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.092 | 0.082 | (in its total) | 0.91× (0.88–0.94) |  |
| inverse BWT, total | 2.249 | 4.511 | 2.384 | 2.01× (1.99–2.02) | 1.06× (1.05–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h08m ± 8m28s |
| 4 threads | 31m04s ± 4m24s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m02s ± 3m16s |
| the scanner alone (its serial part) | 9m19s ± 32.2s |
| the workers alone, over 4 | 23m01s ± 3m16s |

At 4 threads the workers bound it. Per byte: the scanner 5.4 ns a byte of bz2, the workers 3.02 ns a byte of output at 4 threads (CPU, summed over them), 1.94 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,267,482 | 15,078,892 | \`e75b7b7b19e40933…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
