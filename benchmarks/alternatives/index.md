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

The latest record: [`381fbf1`](<https://github.com/lmmx/mibz2/commit/381fbf1d5f76a7f1689e7546d254b7e2bfd664b6>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43714422113>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.441 s | 476 | 7.8 | 2.95× (2.91–2.99) | 6.78× (6.62–6.95) | 7.05× (6.95–7.16) | 7.95× (7.85–8.05) | 7.01× (6.87–7.15) | 7.38× (7.27–7.48) | 7.00× (6.86–7.14) | 2.21× (2.15–2.26) | 2.45× (2.43–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.261 s | 805 | 8.7 | 2.78× (2.61–2.95) | 5.72× (5.46–5.99) | 6.67× (6.34–7.02) | 7.06× (6.66–7.49) | 6.10× (5.77–6.45) | 12.11× (11.49–12.75) | 11.49× (10.90–12.11) | 3.62× (3.43–3.82) | 4.02× (3.79–4.27) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.221 s | 950 | 13.9 | 2.83× (2.74–2.93) | 4.12× (4.05–4.18) | 5.08× (4.97–5.19) | 5.05× (4.90–5.19) | 4.92× (4.85–4.98) | 14.62× (14.30–14.94) | 13.87× (13.60–14.14) | 4.37× (4.25–4.49) | 4.86× (4.77–4.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,020 | 3.9 | 0.55× (0.54–0.57) | 3.19× (3.07–3.31) | 4.25× (4.10–4.42) | 3.48× (3.39–3.57) | 5.98× (5.86–6.09) | 2.55× (2.48–2.63) | 2.55× (2.48–2.62) | 0.96× (0.92–1.00) | 1.97× (1.83–2.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,018 | 4.0 | 0.52× (0.50–0.54) | 2.80× (2.60–3.02) | 5.49× (5.29–5.70) | 3.48× (3.36–3.61) | 5.95× (5.75–6.16) | 2.53× (2.47–2.60) | 2.53× (2.44–2.63) | 0.95× (0.93–0.97) | 1.96× (1.82–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,010 | 4.0 | 0.52× (0.50–0.54) | 2.77× (2.55–3.01) | 5.52× (5.31–5.74) | 3.44× (3.30–3.58) | 5.81× (5.62–6.00) | 2.49× (2.41–2.59) | 2.49× (2.40–2.58) | 0.94× (0.90–0.98) | 1.93× (1.76–2.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 711 | 5.5 | 2.75× (2.69–2.81) | 3.18× (3.04–3.32) | 3.77× (3.68–3.87) | 4.20× (4.11–4.30) | 4.66× (4.56–4.76) | 2.84× (2.77–2.92) | 2.30× (2.24–2.37) | 2.12× (2.06–2.17) | 2.94× (2.84–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,060 | 6.9 | 2.29× (2.08–2.53) | 2.97× (2.67–3.30) | 4.39× (4.01–4.80) | 3.57× (3.27–3.90) | 5.17× (4.85–5.51) | 4.15× (3.82–4.50) | 3.36× (3.09–3.66) | 3.09× (2.84–3.36) | 4.30× (3.95–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 875 | 13.8 | 1.84× (1.78–1.90) | 1.83× (1.74–1.92) | 2.84× (2.62–3.08) | 2.21× (2.12–2.31) | 3.99× (3.84–4.15) | 3.57× (3.37–3.77) | 2.89× (2.75–3.03) | 2.66× (2.53–2.78) | 3.69× (3.49–3.91) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.115 s | 70 | 53.7 | 1.39× (1.35–1.43) | 2.66× (2.60–2.73) | 2.45× (2.40–2.50) | 2.51× (2.47–2.56) | 2.82× (2.76–2.87) | 2.97× (2.90–3.05) | 3.09× (3.02–3.16) | 1.25× (1.21–1.29) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 112 | 61.8 | 1.29× (1.25–1.33) | 2.44× (2.38–2.51) | 2.35× (2.30–2.40) | 3.00× (2.93–3.08) | 2.90× (2.83–2.98) | 4.78× (4.67–4.90) | 4.97× (4.86–5.10) | 2.01× (1.94–2.09) | 2.27× (2.20–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 132 | 84.7 | 1.31× (1.27–1.36) | 2.10× (2.03–2.17) | 2.09× (2.01–2.16) | 2.55× (2.47–2.63) | 2.87× (2.80–2.94) | 5.69× (5.45–5.93) | 5.91× (5.70–6.14) | 2.39× (2.33–2.45) | 2.70× (2.61–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.128 s | 75 | 49.9 | 1.53× (1.46–1.59) | 3.02× (2.92–3.12) | 2.66× (2.57–2.74) | 2.77× (2.68–2.87) | 3.14× (3.06–3.22) | 3.66× (3.56–3.77) | 3.99× (3.87–4.11) | 1.51× (1.48–1.54) | 1.75× (1.70–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.078 s | 123 | 56.9 | 1.35× (1.29–1.41) | 2.75× (2.62–2.89) | 2.48× (2.38–2.58) | 3.37× (3.23–3.52) | 3.08× (2.96–3.20) | 5.94× (5.71–6.18) | 6.46× (6.21–6.72) | 2.45× (2.35–2.54) | 2.84× (2.73–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.067 s | 143 | 79.3 | 1.41× (1.36–1.45) | 2.23× (2.12–2.33) | 2.21× (2.15–2.27) | 2.97× (2.87–3.07) | 2.84× (2.72–2.97) | 6.94× (6.69–7.20) | 7.55× (7.32–7.79) | 2.86× (2.74–2.98) | 3.32× (3.17–3.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 787 | 5.3 | 0.60× (0.57–0.62) | 3.21× (3.12–3.31) | 4.00× (3.88–4.12) | 2.98× (2.88–3.09) | 6.64× (6.38–6.92) | 2.20× (2.16–2.25) | 2.27× (2.19–2.36) | 0.94× (0.90–0.98) | 1.62× (1.57–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 752 | 5.7 | 0.52× (0.51–0.54) | 2.35× (2.30–2.40) | 4.76× (4.55–4.98) | 2.98× (2.86–3.09) | 6.46× (6.23–6.69) | 2.14× (2.08–2.20) | 2.21× (2.13–2.28) | 0.91× (0.88–0.94) | 1.57× (1.54–1.61) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 763 | 5.5 | 0.52× (0.50–0.53) | 2.46× (2.29–2.63) | 4.89× (4.62–5.17) | 2.94× (2.84–3.04) | 6.23× (5.98–6.48) | 2.10× (2.03–2.18) | 2.17× (2.11–2.23) | 0.89× (0.87–0.92) | 1.54× (1.49–1.60) | yes |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 1 | 0.060 s | 211 | 18.0 | 1.80× (1.75–1.85) | 3.87× (3.77–3.98) | 4.30× (4.22–4.37) | 4.51× (4.39–4.63) | 4.73× (4.62–4.83) | 4.62× (4.53–4.71) | 4.69× (4.61–4.78) | 1.48× (1.44–1.53) | 1.71× (1.67–1.75) | yes |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 2 | 0.041 s | 307 | 22.0 | 1.44× (1.37–1.52) | 3.08× (2.91–3.26) | 3.66× (3.47–3.85) | 4.25× (4.10–4.41) | 4.21× (4.06–4.37) | 6.63× (6.34–6.92) | 6.73× (6.43–7.05) | 2.13× (2.04–2.22) | 2.45× (2.34–2.57) | yes |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 4 | 0.037 s | 338 | 34.2 | 1.42× (1.34–1.51) | 2.19× (2.07–2.31) | 2.94× (2.72–3.18) | 3.24× (3.10–3.40) | 3.62× (3.49–3.75) | 7.23× (6.98–7.50) | 7.35× (7.10–7.60) | 2.32× (2.23–2.41) | 2.68× (2.56–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.90–2.98) | 6.74× (6.58–6.91) | 7.02× (6.92–7.12) | 7.92× (7.82–8.02) | 6.98× (6.84–7.13) | 7.35× (7.25–7.46) | 6.98× (6.84–7.12) | 2.20× (2.15–2.25) | 2.44× (2.41–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.73× (2.68–2.79) | 6.09× (5.93–6.26) | 6.46× (6.28–6.65) | 7.50× (7.26–7.74) | 6.22× (6.08–6.38) | 6.56× (6.37–6.77) | 6.23× (6.01–6.46) | 1.97× (1.90–2.03) | 2.18× (2.10–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.76× (2.71–2.82) | 4.51× (4.40–4.62) | 4.88× (4.77–4.99) | 5.50× (5.35–5.66) | 4.73× (4.62–4.84) | 4.10× (4.01–4.19) | 3.89× (3.82–3.97) | 1.23× (1.19–1.27) | 1.36× (1.33–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.84× (2.76–2.91) | 3.77× (3.69–3.86) | 3.26× (3.18–3.34) | 5.55× (5.45–5.65) | 2.42× (2.35–2.49) | 2.42× (2.35–2.49) | 0.96× (0.93–1.00) | 1.58× (1.55–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.60) | 2.70× (2.58–2.81) | 4.86× (4.70–5.03) | 3.26× (3.15–3.37) | 5.47× (5.30–5.65) | 2.37× (2.32–2.43) | 2.37× (2.29–2.46) | 0.94× (0.92–0.97) | 1.55× (1.50–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.58) | 2.96× (2.83–3.09) | 5.00× (4.82–5.18) | 3.27× (3.15–3.40) | 5.42× (5.26–5.59) | 2.38× (2.30–2.47) | 2.38× (2.30–2.47) | 0.95× (0.91–0.99) | 1.55× (1.48–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.59–2.70) | 2.90× (2.82–2.98) | 3.45× (3.37–3.52) | 4.00× (3.92–4.09) | 4.43× (4.34–4.52) | 2.73× (2.66–2.80) | 2.22× (2.17–2.28) | 2.05× (2.00–2.10) | 2.68× (2.61–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.33× (2.26–2.41) | 2.54× (2.47–2.61) | 3.40× (3.28–3.52) | 3.38× (3.27–3.50) | 3.70× (3.60–3.80) | 2.21× (2.14–2.27) | 1.80× (1.75–1.84) | 1.66× (1.61–1.71) | 2.16× (2.08–2.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.95–2.12) | 1.70× (1.65–1.74) | 2.21× (2.12–2.29) | 2.06× (1.98–2.14) | 2.10× (2.04–2.17) | 1.09× (1.04–1.14) | 0.89× (0.85–0.92) | 0.82× (0.78–0.85) | 1.07× (1.01–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.34–1.42) | 2.59× (2.55–2.64) | 2.38× (2.33–2.43) | 2.49× (2.44–2.54) | 2.79× (2.73–2.85) | 2.94× (2.87–3.02) | 3.06× (2.99–3.13) | 1.25× (1.21–1.29) | 1.37× (1.34–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.25–1.31) | 2.42× (2.39–2.45) | 2.13× (2.11–2.16) | 2.81× (2.77–2.85) | 2.51× (2.48–2.55) | 2.58× (2.52–2.64) | 2.69× (2.64–2.74) | 1.09× (1.07–1.12) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.36–1.46) | 2.19× (2.14–2.25) | 1.85× (1.81–1.90) | 2.46× (2.40–2.51) | 2.38× (2.32–2.43) | 1.86× (1.79–1.93) | 1.94× (1.87–2.00) | 0.79× (0.77–0.81) | 0.87× (0.84–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.46–1.59) | 2.95× (2.85–3.05) | 2.59× (2.50–2.68) | 2.75× (2.65–2.84) | 3.11× (3.03–3.18) | 3.62× (3.52–3.73) | 3.94× (3.83–4.06) | 1.50× (1.48–1.53) | 1.71× (1.65–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.34–1.40) | 2.71× (2.65–2.77) | 2.32× (2.27–2.37) | 3.19× (3.11–3.27) | 2.76× (2.70–2.81) | 3.17× (3.12–3.22) | 3.45× (3.37–3.53) | 1.31× (1.27–1.35) | 1.49× (1.44–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.63) | 2.51× (2.44–2.58) | 1.98× (1.93–2.03) | 2.76× (2.68–2.85) | 2.56× (2.48–2.64) | 2.28× (2.24–2.33) | 2.48× (2.43–2.54) | 0.95× (0.92–0.97) | 1.07× (1.05–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.63–0.67) | 2.39× (2.30–2.49) | 3.30× (3.19–3.42) | 2.74× (2.65–2.84) | 5.96× (5.73–6.20) | 2.06× (2.02–2.10) | 2.12× (2.05–2.20) | 0.95× (0.91–0.99) | 1.34× (1.30–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.61) | 2.21× (2.14–2.28) | 4.00× (3.87–4.13) | 2.74× (2.63–2.84) | 5.69× (5.49–5.89) | 1.95× (1.90–2.01) | 2.01× (1.95–2.08) | 0.90× (0.87–0.92) | 1.27× (1.23–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.56–0.59) | 2.50× (2.45–2.56) | 4.10× (3.93–4.27) | 2.79× (2.69–2.88) | 5.67× (5.45–5.90) | 1.98× (1.92–2.05) | 2.05× (1.99–2.10) | 0.91× (0.89–0.94) | 1.29× (1.24–1.34) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 1 | 1.77× (1.73–1.82) | 3.70× (3.62–3.79) | 4.11× (4.03–4.19) | 4.40× (4.28–4.52) | 4.62× (4.51–4.72) | 4.51× (4.42–4.60) | 4.58× (4.50–4.66) | 1.47× (1.43–1.51) | 1.59× (1.56–1.62) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 2 | 1.52× (1.47–1.58) | 3.12× (3.04–3.20) | 3.44× (3.35–3.52) | 4.02× (3.89–4.16) | 3.78× (3.68–3.88) | 3.64× (3.53–3.75) | 3.70× (3.59–3.81) | 1.19× (1.15–1.22) | 1.28× (1.24–1.33) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 4 | 1.59× (1.50–1.70) | 2.50× (2.41–2.59) | 2.87× (2.74–3.00) | 3.17× (3.02–3.32) | 3.06× (2.93–3.19) | 2.37× (2.28–2.46) | 2.40× (2.31–2.50) | 0.77× (0.74–0.80) | 0.83× (0.80–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 30 MB | 0.29× (0.29–0.29) | 0.31× (0.30–0.33) | 0.93× (0.93–0.93) | 0.75× (0.74–0.75) | 10.55× (10.54–10.55) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.41× (0.41–0.41) | 0.21× (0.20–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 64 MB | 0.30× (0.29–0.31) | 0.36× (0.32–0.40) | 0.62× (0.60–0.65) | 0.52× (0.50–0.54) | 4.94× (4.73–5.15) | 0.12× (0.11–0.12) | 0.13× (0.12–0.13) | 0.19× (0.18–0.20) | 0.10× (0.09–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 100 MB | 0.31× (0.30–0.32) | 0.47× (0.41–0.55) | 0.54× (0.52–0.56) | 0.48× (0.47–0.49) | 3.18× (3.08–3.28) | 0.07× (0.07–0.08) | 0.08× (0.08–0.08) | 0.12× (0.12–0.13) | 0.06× (0.06–0.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.03–8.05) | 0.65× (0.65–0.65) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.64× (0.60–0.69) | 1.60× (1.57–1.64) | 0.80× (0.78–0.81) | 5.81× (5.69–5.94) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.61–0.66) | 0.70× (0.67–0.73) | 1.16× (1.10–1.21) | 0.71× (0.68–0.74) | 3.36× (3.20–3.52) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.41× (0.38–0.44) | 0.90× (0.87–0.94) | 0.73× (0.70–0.75) | 3.13× (3.02–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.86× (0.83–0.88) | 0.80× (0.77–0.83) | 2.50× (2.43–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.36× (0.33–0.40) | 0.84× (0.81–0.87) | 0.71× (0.69–0.75) | 2.88× (2.79–2.98) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.60–0.63) | 0.48× (0.45–0.52) | 0.81× (0.80–0.82) | 0.80× (0.77–0.83) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.37× (5.37–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.76× (0.76–0.77) | 5.32× (5.30–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.67) | 6.41× (6.38–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.40× (0.37–0.44) | 1.05× (1.02–1.08) | 0.68× (0.67–0.70) | 3.31× (3.24–3.38) | 0.20× (0.20–0.21) | 0.22× (0.22–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.49 MB → 12.6 MB, 5.0×) | 4 | 57 MB | 0.49× (0.47–0.51) | 0.44× (0.40–0.48) | 0.90× (0.87–0.93) | 0.71× (0.68–0.73) | 2.28× (2.22–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 476 | 2.11 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 805 | 2.36 | 1.12× (1.08–1.16) | 82% | 1.01× (0.97–1.05) | 1.04× (1.00–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 950 | 3.78 | 1.79× (1.75–1.83) | 50% | 1.20× (1.16–1.24) | 1.68× (1.63–1.74) |

**Best:** mibz2 at 4 threads (950 MB/s); lbzip2 at 4 threads (232 MB/s); pbz2 at 4 threads (338 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.20, lbzip2 1t 0.07, lbzip2 2t 0.20, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.61 | 0.02 | 0.07 | 0.221 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3684 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.17 | 12.6% |
| MTF + RLE2 | 1.63 | 17.6% |
| T-vector build | 1.57 | 17.0% |
| walk (the one each build uses) | 4.39 | 47.6% |
| of which: segment assembly | 0.16 | 1.7% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.27 | 2.9% |
| **total** | 9.23 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.398 | 1.065 | (in its total) | 2.67× (2.64–2.71) |  |
| walk | 1.059 | 2.074 | (in its total) | 1.97× (1.94–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.043 | (in its total) | 1.10× (1.05–1.15) |  |
| inverse BWT, total | 1.494 | 3.180 | 1.893 | 2.13× (2.11–2.16) | 1.26× (1.25–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m20s |
| 4 threads | 32m15s ± 4m35s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m11s ± 3m10s |
| the scanner alone (its serial part) | 12m22s ± 44.2s |
| the workers alone, over 4 | 22m11s ± 3m10s |

At 4 threads the workers bound it. Per byte: the scanner 7.2 ns a byte of bz2, the workers 2.91 ns a byte of output at 4 threads (CPU, summed over them), 1.69 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,491,906 | 12,553,386 | \`5e21e3ad886d2197…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3683.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
