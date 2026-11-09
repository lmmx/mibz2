# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-295/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`637ad34`](<https://github.com/lmmx/mibz2/commit/637ad34b75169587f25a5666a05d0f85479b9688>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42592662593>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.3.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.644 s | 325 | 9.0 | 2.35× (2.34–2.36) | 5.34× (5.30–5.38) | 5.49× (5.40–5.58) | 6.16× (5.88–6.44) | 5.49× (5.22–5.77) | 5.88× (5.66–6.11) | 5.63× (5.33–5.94) | 1.77× (1.72–1.83) | 2.03× (2.01–2.05) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.358 s | 585 | 9.8 | 2.38× (2.26–2.51) | 5.10× (4.88–5.33) | 5.70× (5.56–5.84) | 5.77× (5.54–6.01) | 5.13× (4.94–5.33) | 10.20× (9.84–10.58) | 9.76× (9.39–10.13) | 3.08× (2.97–3.18) | 3.52× (3.33–3.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.294 s | 713 | 15.5 | 2.41× (2.38–2.44) | 3.71× (3.54–3.90) | 4.63× (4.43–4.84) | 4.16× (4.08–4.25) | 4.19× (4.09–4.30) | 12.74× (12.55–12.94) | 12.19× (11.82–12.57) | 3.84× (3.81–3.88) | 4.40× (4.29–4.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,010 | 3.2 | 0.61× (0.60–0.61) | 3.68× (3.57–3.80) | 4.53× (4.48–4.59) | 3.63× (3.60–3.67) | 6.26× (6.19–6.33) | 2.90× (2.86–2.93) | 2.71× (2.68–2.74) | 1.01× (1.00–1.03) | 1.54× (1.52–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 962 | 3.4 | 0.57× (0.56–0.58) | 2.99× (2.94–3.04) | 5.91× (5.81–6.01) | 3.56× (3.51–3.62) | 6.07× (5.98–6.16) | 2.81× (2.76–2.86) | 2.63× (2.59–2.68) | 0.98× (0.97–1.00) | 1.50× (1.47–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 971 | 3.3 | 0.57× (0.56–0.58) | 3.00× (2.97–3.03) | 5.94× (5.81–6.07) | 3.54× (3.50–3.58) | 6.09× (6.01–6.18) | 2.80× (2.77–2.83) | 2.62× (2.59–2.65) | 0.98× (0.96–1.00) | 1.49× (1.47–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 544 | 5.7 | 2.81× (2.75–2.87) | 2.84× (2.82–2.85) | 3.64× (3.62–3.66) | 3.67× (3.65–3.69) | 3.87× (3.84–3.90) | 2.33× (2.33–2.34) | 1.98× (1.96–1.99) | 1.88× (1.87–1.89) | 2.83× (2.81–2.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 838 | 6.9 | 2.03× (1.76–2.35) | 2.69× (2.50–2.90) | 3.76× (3.28–4.32) | 3.10× (2.69–3.57) | 4.31× (3.76–4.94) | 3.37× (2.94–3.87) | 2.86× (2.49–3.27) | 2.72× (2.37–3.12) | 4.09× (3.56–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 758 | 13.4 | 1.74× (1.65–1.83) | 1.88× (1.73–2.04) | 2.76× (2.65–2.88) | 2.06× (1.95–2.17) | 3.72× (3.55–3.91) | 3.17× (3.02–3.32) | 2.68× (2.56–2.80) | 2.55× (2.43–2.67) | 3.83× (3.66–4.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 48.0 | 1.38× (1.37–1.38) | 2.71× (2.69–2.73) | 2.44× (2.41–2.46) | 2.52× (2.50–2.53) | 2.87× (2.85–2.88) | 3.10× (3.09–3.12) | 3.24× (3.23–3.25) | 1.25× (1.24–1.25) | 1.42× (1.40–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 54.5 | 1.28× (1.25–1.30) | 2.57× (2.41–2.75) | 2.40× (2.35–2.45) | 3.12× (3.05–3.20) | 2.92× (2.87–2.97) | 4.96× (4.86–5.07) | 5.18× (5.09–5.28) | 1.99× (1.96–2.03) | 2.27× (2.22–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 120 | 73.9 | 1.35× (1.33–1.37) | 2.17× (2.11–2.23) | 2.05× (1.99–2.12) | 2.62× (2.57–2.68) | 2.88× (2.83–2.93) | 5.97× (5.89–6.06) | 6.24× (6.15–6.33) | 2.40× (2.36–2.44) | 2.74× (2.69–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 44.1 | 1.54× (1.46–1.62) | 3.08× (3.04–3.12) | 2.55× (2.53–2.57) | 2.69× (2.68–2.71) | 3.08× (3.06–3.10) | 3.77× (3.75–3.79) | 3.97× (3.96–3.99) | 1.44× (1.42–1.47) | 1.72× (1.68–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 107 | 50.4 | 1.29× (1.23–1.37) | 2.73× (2.52–2.96) | 2.38× (2.24–2.52) | 3.44× (3.26–3.63) | 2.95× (2.80–3.11) | 5.93× (5.62–6.26) | 6.25× (5.93–6.60) | 2.27× (2.15–2.40) | 2.70× (2.59–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.075 s | 128 | 70.8 | 1.44× (1.41–1.47) | 2.27× (2.22–2.33) | 2.25× (2.19–2.32) | 3.05× (2.97–3.12) | 2.82× (2.79–2.84) | 7.12× (7.02–7.23) | 7.51× (7.42–7.61) | 2.73× (2.69–2.77) | 3.24× (3.17–3.32) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 746 | 4.5 | 0.67× (0.66–0.68) | 3.06× (3.03–3.08) | 4.48× (4.44–4.52) | 3.12× (3.09–3.15) | 6.84× (6.77–6.91) | 2.49× (2.47–2.52) | 2.35× (2.33–2.37) | 0.99× (0.97–1.00) | 1.54× (1.53–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 704 | 4.8 | 0.56× (0.56–0.57) | 2.90× (2.87–2.94) | 4.99× (4.94–5.05) | 3.02× (2.99–3.05) | 6.44× (6.34–6.54) | 2.36× (2.34–2.39) | 2.23× (2.20–2.25) | 0.93× (0.92–0.95) | 1.46× (1.44–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 699 | 4.8 | 0.57× (0.56–0.58) | 2.90× (2.86–2.94) | 4.98× (4.91–5.04) | 3.00× (2.96–3.04) | 6.42× (6.32–6.53) | 2.35× (2.32–2.38) | 2.22× (2.19–2.25) | 0.93× (0.91–0.95) | 1.45× (1.43–1.47) | yes |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 1 | 0.090 s | 137 | 21.9 | 1.43× (1.42–1.44) | 2.96× (2.93–3.00) | 3.27× (3.25–3.29) | 3.26× (3.24–3.28) | 3.42× (3.41–3.44) | 3.69× (3.67–3.72) | 3.55× (3.52–3.58) | 1.11× (1.11–1.12) | 1.36× (1.35–1.37) | yes |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 2 | 0.057 s | 215 | 25.5 | 1.15× (1.03–1.27) | 2.45× (2.20–2.74) | 2.83× (2.56–3.12) | 3.25× (2.92–3.62) | 3.13× (2.82–3.46) | 5.44× (4.92–6.02) | 5.23× (4.73–5.78) | 1.64× (1.48–1.81) | 2.00× (1.81–2.21) | yes |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 4 | 0.045 s | 273 | 35.7 | 1.38× (1.27–1.51) | 2.11× (2.06–2.16) | 2.65× (2.53–2.77) | 3.03× (2.92–3.15) | 3.25× (3.14–3.36) | 7.23× (6.95–7.53) | 6.95× (6.68–7.23) | 2.18× (2.10–2.27) | 2.66× (2.56–2.76) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.35× (2.33–2.36) | 5.32× (5.28–5.36) | 5.47× (5.38–5.56) | 6.14× (5.87–6.43) | 5.47× (5.21–5.75) | 5.86× (5.65–6.09) | 5.61× (5.31–5.92) | 1.77× (1.72–1.83) | 2.02× (2.00–2.04) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.31× (2.23–2.39) | 5.15× (5.00–5.29) | 5.35× (5.24–5.47) | 5.98× (5.83–6.13) | 5.10× (4.92–5.30) | 5.38× (5.31–5.46) | 5.15× (5.00–5.30) | 1.62× (1.61–1.64) | 1.85× (1.80–1.90) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.27× (2.24–2.30) | 3.82× (3.62–4.03) | 4.12× (3.90–4.36) | 4.31× (4.21–4.42) | 3.83× (3.71–3.96) | 3.39× (3.33–3.45) | 3.24× (3.13–3.36) | 1.02× (1.01–1.04) | 1.17× (1.14–1.19) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.64–0.65) | 3.33× (3.31–3.35) | 4.14× (4.10–4.18) | 3.40× (3.37–3.43) | 5.78× (5.72–5.84) | 2.72× (2.70–2.75) | 2.56× (2.53–2.58) | 1.01× (1.00–1.03) | 1.10× (1.06–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.64) | 3.08× (3.03–3.13) | 5.36× (5.29–5.44) | 3.31× (3.27–3.36) | 5.55× (5.48–5.62) | 2.61× (2.57–2.65) | 2.45× (2.41–2.49) | 0.97× (0.95–0.99) | 1.05× (1.03–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.61–0.62) | 3.41× (3.36–3.46) | 5.42× (5.35–5.49) | 3.35× (3.32–3.39) | 5.64× (5.58–5.71) | 2.65× (2.62–2.67) | 2.48× (2.46–2.51) | 0.98× (0.97–1.00) | 1.07× (1.04–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.66–2.77) | 2.62× (2.61–2.63) | 3.28× (3.26–3.29) | 3.53× (3.52–3.55) | 3.72× (3.70–3.74) | 2.26× (2.25–2.27) | 1.92× (1.91–1.94) | 1.83× (1.83–1.84) | 2.54× (2.53–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.12× (2.08–2.16) | 2.40× (2.31–2.50) | 2.87× (2.82–2.93) | 3.05× (2.99–3.11) | 3.18× (3.12–3.25) | 1.85× (1.81–1.88) | 1.57× (1.54–1.60) | 1.50× (1.47–1.52) | 2.08× (2.04–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.88–2.03) | 1.69× (1.62–1.76) | 2.14× (2.07–2.22) | 2.06× (1.99–2.13) | 2.06× (1.99–2.13) | 0.97× (0.94–1.01) | 0.83× (0.80–0.86) | 0.79× (0.76–0.82) | 1.09× (1.06–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.66× (2.65–2.67) | 2.39× (2.38–2.40) | 2.49× (2.48–2.50) | 2.84× (2.82–2.85) | 3.07× (3.06–3.09) | 3.21× (3.19–3.22) | 1.24× (1.24–1.25) | 1.38× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.28) | 2.47× (2.44–2.50) | 2.17× (2.15–2.19) | 2.93× (2.90–2.96) | 2.55× (2.52–2.58) | 2.69× (2.66–2.73) | 2.81× (2.78–2.84) | 1.09× (1.08–1.10) | 1.21× (1.19–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.48–1.53) | 2.29× (2.27–2.31) | 1.87× (1.84–1.89) | 2.59× (2.57–2.62) | 2.44× (2.41–2.47) | 1.99× (1.97–2.01) | 2.08× (2.06–2.10) | 0.81× (0.80–0.81) | 0.90× (0.89–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.46–1.61) | 3.02× (2.98–3.05) | 2.51× (2.49–2.54) | 2.67× (2.65–2.69) | 3.05× (3.03–3.08) | 3.73× (3.71–3.75) | 3.94× (3.92–3.96) | 1.44× (1.42–1.46) | 1.68× (1.65–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.35–1.38) | 2.80× (2.73–2.87) | 2.30× (2.27–2.33) | 3.27× (3.23–3.31) | 2.72× (2.69–2.75) | 3.27× (3.22–3.31) | 3.44× (3.40–3.49) | 1.26× (1.24–1.28) | 1.47× (1.44–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.57–1.68) | 2.54× (2.49–2.60) | 1.95× (1.90–2.00) | 2.80× (2.72–2.89) | 2.54× (2.49–2.59) | 2.33× (2.28–2.38) | 2.45× (2.40–2.50) | 0.90× (0.88–0.92) | 1.04× (1.02–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.74× (2.71–2.76) | 3.59× (3.55–3.62) | 2.86× (2.83–2.89) | 6.12× (6.05–6.19) | 2.31× (2.29–2.34) | 2.18× (2.16–2.21) | 0.99× (0.98–1.01) | 1.02× (0.98–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.52× (2.49–2.55) | 4.28× (4.25–4.32) | 2.79× (2.77–2.82) | 5.69× (5.63–5.76) | 2.15× (2.13–2.18) | 2.04× (2.01–2.06) | 0.93× (0.92–0.94) | 0.95× (0.91–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.64) | 2.79× (2.74–2.84) | 4.31× (4.23–4.38) | 2.85× (2.81–2.90) | 5.82× (5.72–5.92) | 2.20× (2.17–2.23) | 2.08× (2.05–2.11) | 0.95× (0.93–0.96) | 0.97× (0.93–1.01) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 1 | 1.42× (1.41–1.43) | 2.90× (2.87–2.93) | 3.14× (3.11–3.16) | 3.21× (3.19–3.23) | 3.37× (3.36–3.39) | 3.64× (3.61–3.66) | 3.49× (3.47–3.52) | 1.11× (1.10–1.12) | 1.29× (1.29–1.30) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 2 | 1.27× (1.24–1.31) | 2.55× (2.48–2.62) | 2.78× (2.70–2.86) | 3.08× (3.00–3.17) | 2.91× (2.82–2.99) | 3.05× (2.97–3.14) | 2.94× (2.85–3.03) | 0.93× (0.91–0.96) | 1.09× (1.06–1.12) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 4 | 1.51× (1.46–1.58) | 2.26× (2.22–2.30) | 2.51× (2.45–2.56) | 2.65× (2.59–2.70) | 2.59× (2.52–2.66) | 2.22× (2.19–2.26) | 2.14× (2.09–2.18) | 0.68× (0.67–0.69) | 0.79× (0.78–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.08× (1.07–1.08) | 0.86× (0.86–0.86) | 12.17× (12.15–12.19) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.39) | 0.59× (0.53–0.64) | 0.80× (0.79–0.81) | 0.67× (0.66–0.68) | 6.38× (6.33–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.73× (0.63–0.84) | 0.71× (0.70–0.72) | 0.61× (0.60–0.62) | 4.09× (4.06–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.83–0.84) | 4.42× (4.41–4.43) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.42× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.67) | 8.12× (8.11–8.14) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.72× (0.65–0.79) | 1.62× (1.58–1.65) | 0.79× (0.78–0.81) | 5.84× (5.70–5.97) | 0.46× (0.45–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.74× (0.70–0.79) | 1.14× (1.08–1.20) | 0.75× (0.71–0.80) | 3.35× (3.18–3.53) | 0.26× (0.24–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.74× (0.73–0.74) | 5.25× (5.24–5.27) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.38–0.44) | 0.92× (0.89–0.94) | 0.70× (0.67–0.74) | 3.17× (3.08–3.26) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.87× (0.85–0.89) | 0.81× (0.79–0.83) | 2.52× (2.46–2.58) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.83× (0.83–0.84) | 5.26× (5.25–5.28) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.49) | 0.39× (0.36–0.41) | 0.87× (0.84–0.90) | 0.73× (0.69–0.77) | 2.97× (2.86–3.09) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.49× (0.47–0.52) | 0.81× (0.79–0.82) | 0.80× (0.78–0.83) | 2.32× (2.28–2.36) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.78) | 5.45× (5.43–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.38× (5.36–5.40) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.39–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.49× (1.49–1.50) | 0.68× (0.67–0.68) | 6.72× (6.71–6.73) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 2 | 34 MB | 0.47× (0.47–0.48) | 0.45× (0.41–0.50) | 1.15× (1.13–1.17) | 0.71× (0.66–0.75) | 3.58× (3.51–3.64) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.19 MB → 12.3 MB, 5.6×) | 4 | 51 MB | 0.55× (0.53–0.56) | 0.50× (0.46–0.53) | 0.99× (0.98–1.00) | 0.78× (0.75–0.81) | 2.54× (2.51–2.56) | 0.14× (0.14–0.15) | 0.15× (0.15–0.15) | 0.23× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 325 | 3.08 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 585 | 3.33 | 1.09× (1.06–1.12) | 87% | 1.05× (1.00–1.11) | 1.07× (1.03–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 713 | 5.30 | 1.73× (1.69–1.77) | 54% | 1.24× (1.16–1.33) | 1.68× (1.63–1.72) |

**Best:** mibz2 at 4 threads (713 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.06, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.90 | 0.02 | 0.08 | 0.294 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2926 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m13s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 11.0% |
| MTF + RLE2 | 1.46 | 15.7% |
| T-vector build | 1.98 | 21.4% |
| walk (the one each build uses) | 4.27 | 46.2% |
| of which: segment assembly | 0.21 | 2.3% |
| RLE1 | 0.27 | 3.0% |
| CRC | 0.24 | 2.6% |
| **total** | 9.26 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.304 | (in its total) | 1.94× (1.92–1.95) |  |
| walk | 1.283 | 2.700 | (in its total) | 2.11× (2.08–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.060 | 0.065 | (in its total) | 1.10× (1.02–1.19) |  |
| inverse BWT, total | 2.016 | 4.069 | 3.405 | 2.02× (1.99–2.05) | 1.67× (1.65–1.70) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h33m ± 11m01s |
| 4 threads | 43m20s ± 6m11s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m08s ± 4m46s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 33m08s ± 4m46s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.34 ns a byte of output at 4 threads (CPU, summed over them), 2.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,189,059 | 12,250,340 | \`cc0185b2dce29802…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2926.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.3.0 (bench/report.py), 10 rounds, threads 1, 2, 4
