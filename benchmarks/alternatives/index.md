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

The latest record: [`6f98a11`](<https://github.com/lmmx/mibz2/commit/6f98a115841deb6bd72aed083dea7622551af68a>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44958798291>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.393 s | 533 | 7.3 | 3.50× (3.48–3.51) | 8.01× (7.96–8.06) | 8.51× (8.46–8.57) | 9.43× (9.39–9.48) | 8.55× (8.51–8.59) | 8.92× (8.87–8.97) | 8.73× (8.69–8.77) | 2.76× (2.74–2.77) | 2.43× (2.42–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.223 s | 939 | 7.9 | 3.30× (3.14–3.47) | 7.18× (6.85–7.52) | 8.69× (8.23–9.18) | 8.58× (8.15–9.04) | 7.78× (7.39–8.20) | 15.51× (14.74–16.32) | 15.18× (14.40–16.01) | 4.80× (4.55–5.05) | 4.23× (4.02–4.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.189 s | 1,112 | 12.4 | 3.40× (3.35–3.45) | 6.20× (6.14–6.26) | 7.37× (7.28–7.45) | 7.21× (7.14–7.28) | 7.05× (6.99–7.11) | 18.68× (18.53–18.83) | 18.29× (18.15–18.43) | 5.78× (5.73–5.83) | 5.09× (5.04–5.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,235 | 3.5 | 0.63× (0.61–0.65) | 3.15× (3.10–3.21) | 4.49× (4.41–4.57) | 3.75× (3.68–3.82) | 6.35× (6.25–6.46) | 2.82× (2.77–2.87) | 2.86× (2.81–2.91) | 1.12× (1.10–1.14) | 1.27× (1.25–1.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,223 | 3.6 | 0.56× (0.54–0.57) | 3.11× (3.02–3.20) | 6.27× (6.09–6.45) | 3.72× (3.63–3.80) | 6.26× (6.14–6.37) | 2.79× (2.72–2.86) | 2.82× (2.76–2.89) | 1.10× (1.07–1.14) | 1.26× (1.22–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,208 | 3.5 | 0.55× (0.54–0.57) | 3.08× (2.99–3.17) | 6.02× (5.73–6.33) | 3.70× (3.59–3.81) | 6.22× (6.07–6.36) | 2.75× (2.67–2.84) | 2.79× (2.71–2.88) | 1.09× (1.05–1.13) | 1.24× (1.20–1.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 624 | 6.6 | 2.58× (2.52–2.64) | 3.26× (3.21–3.31) | 3.95× (3.88–4.02) | 4.22× (4.17–4.27) | 4.63× (4.56–4.70) | 3.12× (3.08–3.16) | 2.89× (2.85–2.93) | 1.98× (1.94–2.03) | 2.11× (2.00–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,008 | 7.7 | 2.23× (2.05–2.43) | 3.09× (2.81–3.39) | 4.22× (4.01–4.44) | 3.97× (3.70–4.26) | 5.01× (4.62–5.44) | 4.83× (4.47–5.22) | 4.48× (4.14–4.84) | 3.08× (2.86–3.31) | 3.27× (2.93–3.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 987 | 13.8 | 1.95× (1.85–2.05) | 2.08× (1.98–2.19) | 2.89× (2.71–3.09) | 2.41× (2.29–2.54) | 4.12× (3.97–4.27) | 4.86× (4.65–5.08) | 4.50× (4.32–4.70) | 3.09× (2.95–3.24) | 3.29× (3.06–3.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.115 s | 70 | 56.6 | 1.37× (1.36–1.38) | 3.01× (3.00–3.02) | 2.77× (2.75–2.79) | 2.84× (2.81–2.86) | 3.11× (3.08–3.14) | 3.45× (3.42–3.48) | 3.56× (3.54–3.58) | 1.25× (1.24–1.26) | 1.24× (1.22–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 120 | 61.5 | 1.32× (1.28–1.36) | 2.95× (2.84–3.05) | 2.88× (2.83–2.93) | 3.48× (3.43–3.53) | 3.23× (3.17–3.28) | 5.87× (5.77–5.97) | 6.06× (5.95–6.16) | 2.13× (2.09–2.16) | 2.12× (2.07–2.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 82.9 | 1.40× (1.36–1.44) | 2.74× (2.68–2.79) | 2.59× (2.52–2.66) | 3.26× (3.16–3.36) | 3.19× (3.12–3.26) | 7.03× (6.87–7.19) | 7.25× (7.09–7.41) | 2.55× (2.49–2.60) | 2.53× (2.48–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.127 s | 76 | 52.1 | 1.42× (1.42–1.43) | 3.40× (3.37–3.43) | 2.89× (2.87–2.90) | 3.01× (2.99–3.02) | 3.34× (3.33–3.36) | 4.09× (4.07–4.10) | 4.40× (4.39–4.42) | 1.46× (1.46–1.47) | 1.51× (1.51–1.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 127 | 57.3 | 1.29× (1.27–1.31) | 3.06× (2.97–3.15) | 2.82× (2.76–2.89) | 3.76× (3.68–3.85) | 3.24× (3.16–3.32) | 6.83× (6.68–6.99) | 7.36× (7.19–7.54) | 2.45× (2.39–2.50) | 2.53× (2.47–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 150 | 80.9 | 1.41× (1.37–1.45) | 2.95× (2.88–3.02) | 2.62× (2.56–2.67) | 3.53× (3.48–3.58) | 3.12× (3.07–3.16) | 8.10× (8.00–8.21) | 8.73× (8.63–8.83) | 2.90× (2.87–2.94) | 3.00× (2.96–3.04) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 926 | 4.9 | 0.68× (0.66–0.70) | 2.83× (2.77–2.89) | 3.99× (3.85–4.14) | 3.17× (3.12–3.22) | 6.85× (6.74–6.96) | 2.43× (2.39–2.47) | 2.47× (2.43–2.51) | 1.04× (1.02–1.05) | 0.96× (0.94–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 866 | 5.3 | 0.52× (0.49–0.54) | 2.63× (2.48–2.79) | 5.36× (5.04–5.70) | 2.97× (2.81–3.14) | 6.40× (6.11–6.70) | 2.25× (2.13–2.39) | 2.29× (2.17–2.42) | 0.96× (0.92–1.02) | 0.90× (0.84–0.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 873 | 5.1 | 0.53× (0.52–0.54) | 2.70× (2.65–2.76) | 5.38× (5.10–5.68) | 3.06× (3.00–3.13) | 6.53× (6.42–6.65) | 2.31× (2.26–2.36) | 2.35× (2.31–2.39) | 0.99× (0.97–1.01) | 0.92× (0.90–0.93) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 0.063 s | 214 | 18.7 | 1.84× (1.83–1.85) | 4.21× (4.19–4.23) | 4.71× (4.69–4.74) | 4.77× (4.74–4.80) | 4.86× (4.84–4.89) | 5.09× (5.07–5.12) | 5.08× (5.05–5.10) | 1.49× (1.49–1.50) | 1.46× (1.45–1.47) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 0.040 s | 335 | 21.5 | 1.58× (1.41–1.78) | 3.75× (3.16–4.45) | 4.59× (4.10–5.13) | 4.75× (4.21–5.36) | 4.46× (4.15–4.80) | 7.68× (7.13–8.28) | 7.66× (7.13–8.23) | 2.25× (2.09–2.42) | 2.20× (2.05–2.36) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 0.035 s | 381 | 32.2 | 1.60× (1.56–1.63) | 3.00× (2.88–3.13) | 3.92× (3.82–4.03) | 4.10× (3.84–4.37) | 4.33× (4.23–4.43) | 9.01× (8.81–9.22) | 8.98× (8.79–9.17) | 2.64× (2.58–2.69) | 2.58× (2.52–2.63) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.49× (3.47–3.50) | 7.97× (7.91–8.02) | 8.47× (8.41–8.52) | 9.40× (9.35–9.44) | 8.52× (8.47–8.56) | 8.88× (8.83–8.93) | 8.70× (8.66–8.73) | 2.75× (2.74–2.76) | 2.41× (2.40–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.25× (3.21–3.30) | 7.51× (7.38–7.63) | 8.27× (8.15–8.38) | 9.01× (8.88–9.14) | 7.82× (7.71–7.93) | 8.18× (8.06–8.30) | 8.01× (7.90–8.12) | 2.53× (2.50–2.57) | 2.22× (2.19–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.34× (3.29–3.40) | 6.84× (6.76–6.91) | 7.28× (7.21–7.34) | 7.95× (7.87–8.04) | 7.10× (7.01–7.18) | 5.24× (5.16–5.32) | 5.13× (5.07–5.19) | 1.62× (1.60–1.64) | 1.42× (1.41–1.44) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.64–0.68) | 2.88× (2.84–2.92) | 3.89× (3.83–3.95) | 3.48× (3.43–3.53) | 5.84× (5.76–5.92) | 2.65× (2.61–2.68) | 2.68× (2.64–2.72) | 1.11× (1.09–1.13) | 0.82× (0.81–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.62) | 2.85× (2.78–2.92) | 5.47× (5.35–5.60) | 3.42× (3.34–3.50) | 5.68× (5.58–5.78) | 2.58× (2.51–2.64) | 2.61× (2.55–2.67) | 1.08× (1.05–1.11) | 0.80× (0.78–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.57–0.61) | 3.25× (3.11–3.39) | 5.46× (5.30–5.64) | 3.48× (3.39–3.57) | 5.76× (5.65–5.88) | 2.61× (2.54–2.68) | 2.64× (2.57–2.72) | 1.09× (1.05–1.13) | 0.81× (0.79–0.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.44–2.57) | 3.02× (2.97–3.07) | 3.54× (3.47–3.60) | 4.06× (3.99–4.12) | 4.45× (4.37–4.53) | 3.01× (2.97–3.06) | 2.79× (2.75–2.84) | 1.94× (1.89–1.98) | 1.90× (1.87–1.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.24–2.41) | 2.73× (2.66–2.81) | 3.34× (3.23–3.46) | 3.55× (3.44–3.67) | 3.89× (3.76–4.02) | 2.57× (2.49–2.65) | 2.38× (2.31–2.45) | 1.65× (1.60–1.70) | 1.62× (1.57–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.96–2.11) | 1.86× (1.78–1.95) | 2.32× (2.24–2.40) | 2.26× (2.18–2.33) | 2.35× (2.28–2.43) | 1.43× (1.38–1.48) | 1.32× (1.28–1.37) | 0.92× (0.89–0.95) | 0.90× (0.87–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.96× (2.94–2.98) | 2.71× (2.69–2.72) | 2.81× (2.78–2.83) | 3.08× (3.05–3.11) | 3.41× (3.38–3.45) | 3.52× (3.50–3.54) | 1.25× (1.24–1.26) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.29–1.31) | 2.85× (2.83–2.88) | 2.60× (2.58–2.61) | 3.29× (3.27–3.32) | 2.88× (2.86–2.90) | 3.15× (3.13–3.17) | 3.25× (3.23–3.27) | 1.15× (1.14–1.16) | 1.13× (1.12–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.53–1.61) | 2.90× (2.81–3.00) | 2.51× (2.46–2.55) | 3.23× (3.16–3.30) | 2.88× (2.83–2.94) | 2.34× (2.29–2.38) | 2.41× (2.37–2.45) | 0.85× (0.84–0.87) | 0.83× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.42× (1.41–1.42) | 3.32× (3.31–3.34) | 2.83× (2.81–2.84) | 2.98× (2.97–3.00) | 3.31× (3.30–3.32) | 4.04× (4.03–4.06) | 4.36× (4.34–4.37) | 1.46× (1.45–1.46) | 1.45× (1.45–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.29–1.35) | 3.11× (3.03–3.18) | 2.66× (2.60–2.71) | 3.57× (3.50–3.65) | 3.01× (2.94–3.07) | 3.64× (3.57–3.72) | 3.93× (3.84–4.02) | 1.31× (1.28–1.34) | 1.31× (1.28–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.52–1.62) | 3.17× (3.04–3.30) | 2.41× (2.34–2.49) | 3.39× (3.33–3.45) | 2.91× (2.84–2.97) | 2.62× (2.56–2.68) | 2.82× (2.75–2.88) | 0.94× (0.92–0.96) | 0.94× (0.92–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 2.47× (2.42–2.52) | 3.44× (3.40–3.48) | 2.89× (2.85–2.93) | 6.08× (6.00–6.17) | 2.23× (2.20–2.26) | 2.28× (2.25–2.31) | 1.03× (1.02–1.05) | 0.75× (0.74–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.56–0.61) | 2.22× (2.13–2.31) | 4.28× (4.08–4.50) | 2.72× (2.60–2.85) | 5.62× (5.41–5.84) | 2.04× (1.94–2.14) | 2.08× (1.99–2.18) | 0.95× (0.90–0.99) | 0.69× (0.65–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.61× (2.51–2.71) | 4.41× (4.29–4.54) | 2.87× (2.81–2.92) | 5.88× (5.78–5.98) | 2.14× (2.10–2.18) | 2.18× (2.15–2.22) | 0.99× (0.97–1.01) | 0.72× (0.71–0.74) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 1.81× (1.80–1.82) | 4.04× (4.02–4.07) | 4.52× (4.50–4.54) | 4.67× (4.64–4.70) | 4.76× (4.73–4.79) | 4.98× (4.95–5.01) | 4.97× (4.94–4.99) | 1.48× (1.47–1.49) | 1.37× (1.36–1.39) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 1.64× (1.63–1.66) | 3.67× (3.60–3.74) | 4.28× (4.23–4.33) | 4.54× (4.47–4.61) | 4.19× (4.15–4.23) | 4.32× (4.28–4.36) | 4.31× (4.26–4.35) | 1.28× (1.27–1.29) | 1.19× (1.18–1.20) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 1.79× (1.71–1.88) | 3.28× (3.22–3.34) | 3.70× (3.62–3.79) | 4.02× (3.92–4.11) | 3.70× (3.63–3.76) | 2.89× (2.84–2.94) | 2.88× (2.83–2.94) | 0.86× (0.84–0.87) | 0.80× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.29× (0.29–0.29) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.96× (10.92–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.35) | 0.56× (0.47–0.66) | 0.71× (0.68–0.73) | 0.60× (0.57–0.63) | 5.62× (5.43–5.83) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.34) | 0.51× (0.43–0.59) | 0.59× (0.57–0.61) | 0.53× (0.51–0.55) | 3.48× (3.36–3.60) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.42× (0.40–0.44) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.96× (7.92–7.99) | 0.64× (0.64–0.64) | 0.68× (0.67–0.68) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.69) | 0.58× (0.53–0.63) | 1.59× (1.53–1.66) | 0.81× (0.78–0.84) | 5.80× (5.58–6.02) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.61–0.65) | 0.68× (0.62–0.75) | 1.12× (1.08–1.16) | 0.71× (0.69–0.74) | 3.35× (3.22–3.48) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.16× (5.14–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.53) | 0.43× (0.41–0.45) | 0.92× (0.87–0.97) | 0.72× (0.68–0.77) | 3.17× (3.01–3.33) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.86× (0.83–0.88) | 0.80× (0.77–0.83) | 2.49× (2.42–2.57) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.17× (5.15–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.51) | 0.39× (0.37–0.41) | 0.86× (0.82–0.90) | 0.72× (0.67–0.77) | 2.94× (2.82–3.08) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.60–0.63) | 0.49× (0.46–0.51) | 0.82× (0.81–0.83) | 0.84× (0.82–0.86) | 2.36× (2.33–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.28× (5.27–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.33× (6.31–6.35) | 0.39× (0.39–0.39) | 0.41× (0.40–0.41) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.45× (0.40–0.50) | 1.14× (1.11–1.17) | 0.75× (0.73–0.76) | 3.61× (3.54–3.68) | 0.22× (0.22–0.23) | 0.23× (0.23–0.24) | 0.36× (0.35–0.37) | 0.17× (0.17–0.18) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 53 MB | 0.53× (0.51–0.54) | 0.46× (0.41–0.50) | 0.97× (0.94–0.99) | 0.79× (0.74–0.83) | 2.46× (2.40–2.53) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 533 | 1.88 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 939 | 2.04 | 1.09× (1.07–1.10) | 87% | 1.02× (1.01–1.03) | 1.01× (1.01–1.01) |
| 4: all 4 CPUs, sharing SMT siblings | 1,112 | 3.20 | 1.70× (1.68–1.71) | 52% | 1.45× (1.44–1.47) | 1.63× (1.60–1.65) |

**Best:** mibz2 at 4 threads (1,112 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (328 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.88, mibz2 4t 1.21, lbzip2 1t 0.04, lbzip2 2t 0.21, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.51 | 0.02 | 0.07 | 0.189 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3889 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 10.5% |
| MTF + RLE2 | 1.62 | 14.8% |
| T-vector build | 1.53 | 14.1% |
| walk (the one each build uses) | 6.21 | 57.0% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.90 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.372 | 0.934 | (in its total) | 2.51× (2.51–2.51) |  |
| walk | 1.512 | 2.884 | (in its total) | 1.91× (1.91–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.073 | (in its total) | 0.91× (0.90–0.91) |  |
| inverse BWT, total | 1.965 | 3.891 | 2.025 | 1.98× (1.98–1.98) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m14s ± 6m35s |
| 4 threads | 27m19s ± 3m50s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m24s ± 2m38s |
| the scanner alone (its serial part) | 10m20s ± 19.7s |
| the workers alone, over 4 | 18m24s ± 2m38s |

At 4 threads the workers bound it. Per byte: the scanner 6.0 ns a byte of bz2, the workers 2.41 ns a byte of output at 4 threads (CPU, summed over them), 1.54 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,640,714 | 13,421,331 | \`8da9d679141c517c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3888.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
