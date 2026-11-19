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

The latest record: [`98d66f5`](<https://github.com/lmmx/mibz2/commit/98d66f5a937d2898de91650ab716a4cca937853a>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44590411769>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.544 s | 385 | 7.8 | 3.12× (3.08–3.17) | 6.95× (6.82–7.08) | 7.42× (7.32–7.52) | 8.26× (8.16–8.35) | 7.44× (7.31–7.58) | 7.94× (7.83–8.06) | 7.56× (7.45–7.67) | 2.35× (2.31–2.39) | 2.16× (2.12–2.19) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.306 s | 684 | 8.5 | 3.13× (2.91–3.36) | 6.31× (5.95–6.70) | 7.92× (7.44–8.43) | 7.88× (7.43–8.35) | 6.89× (6.47–7.34) | 13.69× (12.95–14.47) | 13.03× (12.30–13.79) | 4.05× (3.79–4.32) | 3.71× (3.50–3.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.257 s | 815 | 13.3 | 3.49× (3.36–3.63) | 6.06× (6.01–6.11) | 7.04× (6.93–7.15) | 7.02× (6.89–7.16) | 6.73× (6.60–6.87) | 16.93× (16.66–17.21) | 16.12× (15.81–16.43) | 5.01× (4.90–5.12) | 4.60× (4.49–4.70) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 920 | 3.5 | 0.58× (0.56–0.59) | 3.30× (3.22–3.37) | 4.44× (4.31–4.57) | 3.78× (3.66–3.90) | 6.15× (6.03–6.28) | 2.80× (2.73–2.86) | 2.78× (2.72–2.83) | 1.23× (1.17–1.29) | 1.03× (0.92–1.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 870 | 3.8 | 0.54× (0.51–0.57) | 2.78× (2.61–2.95) | 5.56× (5.31–5.81) | 3.55× (3.44–3.66) | 5.81× (5.63–6.00) | 2.64× (2.54–2.75) | 2.62× (2.52–2.73) | 1.16× (1.10–1.23) | 0.98× (0.88–1.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 858 | 3.9 | 0.53× (0.49–0.57) | 2.85× (2.61–3.10) | 5.51× (5.28–5.75) | 3.50× (3.34–3.66) | 5.74× (5.52–5.96) | 2.59× (2.49–2.69) | 2.57× (2.46–2.68) | 1.14× (1.06–1.22) | 0.96× (0.85–1.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.041 s | 478 | 6.6 | 2.81× (2.77–2.85) | 3.21× (3.14–3.27) | 3.80× (3.70–3.90) | 4.28× (4.17–4.39) | 4.59× (4.46–4.71) | 3.52× (3.44–3.60) | 2.79× (2.73–2.86) | 2.31× (2.22–2.41) | 2.05× (1.97–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 s | 699 | 8.3 | 2.18× (2.00–2.37) | 2.66× (2.39–2.96) | 3.77× (3.49–4.08) | 3.44× (3.19–3.71) | 4.71× (4.29–5.16) | 5.05× (4.70–5.43) | 4.01× (3.73–4.30) | 3.31× (3.05–3.60) | 2.94× (2.68–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 723 | 14.6 | 2.14× (2.09–2.20) | 1.94× (1.87–2.01) | 2.91× (2.77–3.05) | 2.51× (2.35–2.67) | 4.06× (3.95–4.18) | 5.30× (5.06–5.55) | 4.20× (4.05–4.36) | 3.47× (3.29–3.67) | 3.08× (2.91–3.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.168 s | 48 | 63.8 | 1.26× (1.24–1.28) | 2.44× (2.40–2.48) | 2.20× (2.17–2.22) | 2.19× (2.17–2.22) | 2.50× (2.47–2.54) | 2.84× (2.81–2.88) | 2.91× (2.87–2.95) | 1.29× (1.27–1.30) | 1.29× (1.27–1.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.102 s | 79 | 69.7 | 1.19× (1.14–1.24) | 2.35× (2.26–2.44) | 2.26× (2.18–2.35) | 2.86× (2.69–3.05) | 2.61× (2.53–2.70) | 4.68× (4.54–4.83) | 4.79× (4.62–4.97) | 2.12× (2.03–2.21) | 2.12× (2.03–2.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.081 s | 99 | 89.9 | 1.36× (1.32–1.40) | 2.36× (2.28–2.43) | 2.44× (2.35–2.53) | 2.88× (2.80–2.98) | 2.85× (2.77–2.93) | 5.96× (5.78–6.14) | 6.10× (5.97–6.23) | 2.70× (2.65–2.74) | 2.70× (2.65–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.190 s | 51 | 60.0 | 1.30× (1.27–1.34) | 2.84× (2.77–2.91) | 2.41× (2.34–2.47) | 2.43× (2.37–2.48) | 2.68× (2.66–2.70) | 3.43× (3.41–3.46) | 3.65× (3.59–3.71) | 1.49× (1.47–1.50) | 1.48× (1.46–1.50) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.109 s | 88 | 65.3 | 1.24× (1.22–1.27) | 2.71× (2.49–2.94) | 2.44× (2.38–2.50) | 3.35× (3.23–3.48) | 2.81× (2.74–2.89) | 5.90× (5.72–6.09) | 6.27× (6.11–6.44) | 2.55× (2.48–2.63) | 2.55× (2.49–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.090 s | 107 | 88.7 | 1.43× (1.39–1.46) | 2.65× (2.55–2.75) | 2.60× (2.35–2.87) | 3.18× (3.11–3.26) | 2.94× (2.83–3.04) | 7.19× (7.02–7.37) | 7.65× (7.47–7.83) | 3.11× (3.05–3.18) | 3.10× (3.04–3.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 678 | 5.0 | 0.64× (0.60–0.68) | 2.86× (2.62–3.13) | 3.99× (3.81–4.16) | 3.14× (2.98–3.32) | 6.53× (6.22–6.85) | 2.40× (2.28–2.52) | 2.46× (2.33–2.60) | 1.05× (0.99–1.12) | 1.39× (1.33–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.017 s | 588 | 5.8 | 0.48× (0.44–0.52) | 2.39× (2.19–2.61) | 4.54× (4.19–4.93) | 2.79× (2.56–3.03) | 5.72× (5.24–6.24) | 2.07× (1.90–2.24) | 2.12× (1.90–2.37) | 0.91× (0.83–0.99) | 1.20× (1.10–1.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 621 | 5.4 | 0.53× (0.49–0.57) | 2.55× (2.45–2.65) | 4.92× (4.74–5.10) | 3.03× (2.91–3.16) | 6.01× (5.85–6.18) | 2.21× (2.11–2.31) | 2.27× (2.10–2.45) | 0.97× (0.92–1.02) | 1.28× (1.23–1.33) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 0.082 s | 162 | 19.0 | 1.82× (1.80–1.85) | 3.84× (3.77–3.92) | 4.44× (4.33–4.54) | 4.50× (4.42–4.59) | 4.54× (4.43–4.67) | 4.84× (4.74–4.95) | 4.69× (4.63–4.75) | 1.49× (1.47–1.51) | 1.45× (1.40–1.51) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 0.054 s | 246 | 22.7 | 1.59× (1.49–1.71) | 3.33× (3.15–3.51) | 4.25× (3.93–4.58) | 4.59× (4.32–4.87) | 4.39× (4.16–4.63) | 7.24× (6.90–7.60) | 7.02× (6.62–7.44) | 2.22× (2.12–2.33) | 2.17× (2.05–2.30) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 0.047 s | 279 | 34.2 | 1.71× (1.64–1.79) | 3.07× (2.96–3.19) | 3.99× (3.83–4.15) | 4.11× (3.99–4.24) | 4.47× (4.26–4.68) | 8.48× (8.20–8.78) | 8.22× (8.02–8.43) | 2.61× (2.53–2.69) | 2.55× (2.45–2.65) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (3.07–3.16) | 6.92× (6.79–7.05) | 7.39× (7.28–7.49) | 8.23× (8.14–8.32) | 7.42× (7.29–7.56) | 7.92× (7.81–8.03) | 7.54× (7.43–7.65) | 2.34× (2.30–2.38) | 2.14× (2.11–2.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (3.03–3.09) | 6.61× (6.56–6.66) | 7.58× (7.49–7.68) | 8.25× (8.20–8.29) | 6.94× (6.84–7.04) | 7.25× (7.20–7.30) | 6.90× (6.82–6.98) | 2.15× (2.11–2.18) | 1.96× (1.93–2.00) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.42× (3.34–3.49) | 6.54× (6.38–6.70) | 6.94× (6.68–7.21) | 7.59× (7.31–7.89) | 6.66× (6.45–6.89) | 4.67× (4.57–4.77) | 4.45× (4.34–4.56) | 1.38× (1.35–1.41) | 1.27× (1.24–1.29) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.63) | 2.87× (2.84–2.90) | 3.93× (3.86–4.00) | 3.58× (3.47–3.68) | 5.78× (5.68–5.88) | 2.68× (2.62–2.73) | 2.66× (2.61–2.70) | 1.21× (1.16–1.27) | 0.92× (0.89–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.55–0.60) | 2.73× (2.64–2.83) | 4.92× (4.75–5.10) | 3.31× (3.22–3.40) | 5.35× (5.20–5.50) | 2.47× (2.38–2.56) | 2.45× (2.37–2.54) | 1.12× (1.07–1.18) | 0.85× (0.82–0.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.52–0.59) | 3.06× (2.90–3.21) | 4.91× (4.71–5.11) | 3.30× (3.17–3.44) | 5.33× (5.14–5.53) | 2.45× (2.36–2.55) | 2.43× (2.33–2.54) | 1.11× (1.04–1.19) | 0.84× (0.80–0.89) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.73× (2.68–2.78) | 3.01× (2.93–3.08) | 3.54× (3.46–3.62) | 4.12× (4.01–4.25) | 4.43× (4.30–4.55) | 3.40× (3.32–3.49) | 2.71× (2.65–2.78) | 2.25× (2.16–2.34) | 1.90× (1.85–1.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.20–2.40) | 2.56× (2.46–2.67) | 3.08× (3.01–3.16) | 3.38× (3.30–3.46) | 3.64× (3.53–3.75) | 2.70× (2.64–2.76) | 2.15× (2.10–2.21) | 1.78× (1.70–1.86) | 1.51× (1.47–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.23× (2.16–2.31) | 1.81× (1.76–1.87) | 2.34× (2.26–2.44) | 2.27× (2.20–2.35) | 2.29× (2.22–2.35) | 1.54× (1.49–1.60) | 1.23× (1.20–1.27) | 1.02× (0.98–1.06) | 0.86× (0.83–0.90) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.26× (1.23–1.28) | 2.40× (2.37–2.44) | 2.15× (2.12–2.19) | 2.18× (2.16–2.20) | 2.49× (2.45–2.52) | 2.82× (2.78–2.86) | 2.89× (2.85–2.93) | 1.28× (1.27–1.30) | 1.25× (1.24–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.20× (1.16–1.23) | 2.37× (2.30–2.45) | 2.11× (2.06–2.17) | 2.73× (2.65–2.81) | 2.37× (2.31–2.43) | 2.57× (2.51–2.63) | 2.63× (2.56–2.70) | 1.17× (1.13–1.21) | 1.14× (1.11–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.54× (1.50–1.58) | 2.64× (2.58–2.71) | 2.27× (2.20–2.34) | 2.89× (2.81–2.96) | 2.59× (2.52–2.65) | 2.00× (1.94–2.06) | 2.04× (2.00–2.09) | 0.91× (0.89–0.92) | 0.89× (0.87–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.30× (1.26–1.34) | 2.79× (2.72–2.86) | 2.35× (2.29–2.42) | 2.41× (2.36–2.46) | 2.67× (2.65–2.69) | 3.41× (3.38–3.43) | 3.62× (3.57–3.69) | 1.48× (1.47–1.50) | 1.45× (1.43–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.27× (1.23–1.30) | 2.68× (2.60–2.77) | 2.27× (2.24–2.30) | 3.09× (3.01–3.17) | 2.56× (2.50–2.63) | 3.12× (3.02–3.21) | 3.31× (3.22–3.41) | 1.35× (1.32–1.39) | 1.33× (1.29–1.37) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.52–1.63) | 2.94× (2.83–3.05) | 2.35× (2.15–2.56) | 3.07× (2.98–3.15) | 2.70× (2.62–2.79) | 2.31× (2.25–2.37) | 2.45× (2.39–2.52) | 1.00× (0.98–1.03) | 0.98× (0.96–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.64–0.71) | 2.37× (2.25–2.50) | 3.31× (3.16–3.47) | 2.88× (2.76–3.00) | 5.86× (5.58–6.15) | 2.23× (2.15–2.31) | 2.28× (2.15–2.41) | 1.03× (0.98–1.09) | 0.79× (0.77–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.50–0.59) | 2.09× (1.92–2.27) | 3.91× (3.63–4.22) | 2.63× (2.44–2.83) | 5.17× (4.82–5.56) | 1.93× (1.79–2.08) | 1.98× (1.80–2.17) | 0.89× (0.82–0.97) | 0.69× (0.64–0.74) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.53–0.61) | 2.59× (2.48–2.71) | 4.20× (4.08–4.32) | 2.93× (2.79–3.07) | 5.56× (5.42–5.70) | 2.10× (2.01–2.19) | 2.15× (2.00–2.31) | 0.97× (0.92–1.02) | 0.75× (0.71–0.79) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 1.81× (1.78–1.83) | 3.72× (3.65–3.79) | 4.30× (4.21–4.40) | 4.43× (4.34–4.51) | 4.47× (4.36–4.58) | 4.76× (4.65–4.87) | 4.61× (4.56–4.66) | 1.48× (1.46–1.50) | 1.37× (1.34–1.39) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 1.61× (1.58–1.64) | 3.36× (3.27–3.45) | 3.95× (3.81–4.10) | 4.35× (4.16–4.54) | 3.90× (3.76–4.03) | 3.93× (3.83–4.03) | 3.81× (3.68–3.94) | 1.22× (1.18–1.26) | 1.13× (1.09–1.17) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 1.87× (1.83–1.92) | 3.37× (3.27–3.47) | 3.92× (3.77–4.07) | 4.06× (3.94–4.17) | 3.76× (3.63–3.90) | 2.67× (2.58–2.77) | 2.59× (2.52–2.67) | 0.83× (0.81–0.85) | 0.77× (0.75–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.95× (10.93–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.33–0.35) | 0.45× (0.37–0.55) | 0.69× (0.68–0.71) | 0.59× (0.57–0.60) | 5.53× (5.40–5.66) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.34) | 0.44× (0.39–0.48) | 0.59× (0.57–0.62) | 0.52× (0.50–0.54) | 3.44× (3.31–3.57) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.16× (0.16–0.17) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.17× (0.15–0.18) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.17× (0.16–0.18) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.53) | 0.44× (0.43–0.45) | 1.70× (1.69–1.71) | 0.65× (0.65–0.66) | 8.01× (7.97–8.06) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.65× (0.58–0.73) | 1.62× (1.58–1.66) | 0.81× (0.79–0.84) | 5.87× (5.73–6.03) | 0.47× (0.46–0.49) | 0.50× (0.49–0.51) | 0.55× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.72× (0.69–0.75) | 1.16× (1.14–1.19) | 0.72× (0.70–0.74) | 3.47× (3.42–3.52) | 0.27× (0.27–0.27) | 0.28× (0.28–0.29) | 0.31× (0.31–0.32) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.17× (5.14–5.19) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.38× (0.35–0.41) | 0.90× (0.87–0.93) | 0.72× (0.69–0.74) | 3.12× (3.04–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.46) | 0.87× (0.85–0.88) | 0.82× (0.80–0.83) | 2.52× (2.48–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.46–0.48) | 0.34× (0.31–0.37) | 0.83× (0.81–0.85) | 0.69× (0.67–0.71) | 2.83× (2.76–2.89) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.47× (0.44–0.50) | 0.82× (0.79–0.84) | 0.83× (0.80–0.86) | 2.35× (2.30–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.35× (5.34–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.78) | 5.28× (5.27–5.30) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.31× (5.29–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.41) | 0.66× (0.65–0.66) | 6.40× (6.39–6.40) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 38 MB | 0.43× (0.41–0.44) | 0.35× (0.32–0.39) | 1.00× (0.96–1.03) | 0.65× (0.61–0.69) | 3.18× (3.09–3.28) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.32) | 0.15× (0.15–0.16) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 58 MB | 0.49× (0.47–0.51) | 0.48× (0.44–0.53) | 0.88× (0.84–0.91) | 0.74× (0.71–0.77) | 2.27× (2.19–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 385 | 2.61 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 684 | 2.83 | 1.09× (1.08–1.11) | 86% | 1.04× (1.03–1.06) | 1.07× (1.06–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 815 | 4.43 | 1.69× (1.67–1.72) | 53% | 1.60× (1.58–1.62) | 1.86× (1.83–1.90) |

**Best:** mibz2 at 4 threads (815 MB/s); lbzip2 at 4 threads (135 MB/s); pbz2 at 4 threads (238 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.21, lbzip2 1t 0.05, lbzip2 2t 0.19, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.26, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.72 | 0.02 | 0.08 | 0.257 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3003 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m15s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.20 | 10.7% |
| MTF + RLE2 | 1.71 | 15.3% |
| T-vector build | 1.68 | 15.0% |
| walk (the one each build uses) | 6.13 | 54.8% |
| of which: segment assembly | 0.35 | 3.1% |
| RLE1 | 0.31 | 2.8% |
| CRC | 0.14 | 1.3% |
| **total** | 11.20 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.530 | 1.385 | (in its total) | 2.62× (2.60–2.63) |  |
| walk | 1.875 | 3.365 | (in its total) | 1.80× (1.78–1.82) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.117 | 0.126 | (in its total) | 1.08× (1.06–1.10) |  |
| inverse BWT, total | 2.522 | 4.878 | 2.531 | 1.94× (1.92–1.95) | 1.00× (1.00–1.01) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h19m ± 9m01s |
| 4 threads | 37m14s ± 5m15s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m07s ± 3m42s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 26m07s ± 3m42s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.42 ns a byte of output at 4 threads (CPU, summed over them), 2.10 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,599,481 | 13,234,711 | \`71fd43c7474b3515…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3003.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
