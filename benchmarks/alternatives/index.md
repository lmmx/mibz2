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

The latest record: [`f9f6887`](<https://github.com/lmmx/mibz2/commit/f9f688730c01977fc756a8e51dde1aa8958b39ea>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40838356303>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.390 s | 538 | 4.8 | 2.13× (2.04–2.23) | 5.32× (5.10–5.56) | 5.51× (5.22–5.83) | 6.30× (6.06–6.55) | 5.59× (5.32–5.88) | 5.88× (5.61–6.18) | 5.48× (5.26–5.71) | 1.66× (1.59–1.73) | 1.93× (1.85–2.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.274 s | 765 | 5.5 | 1.77× (1.67–1.89) | 3.96× (3.71–4.24) | 4.75× (4.46–5.06) | 4.77× (4.50–5.05) | 4.27× (3.99–4.58) | 8.23× (7.69–8.81) | 7.67× (7.19–8.18) | 2.32× (2.19–2.46) | 2.70× (2.53–2.87) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.255 s | 824 | 10.2 | 1.53× (1.50–1.55) | 2.58× (2.49–2.67) | 3.23× (3.14–3.31) | 3.12× (2.97–3.28) | 3.16× (3.04–3.29) | 8.87× (8.43–9.34) | 8.27× (7.90–8.66) | 2.50× (2.41–2.60) | 2.91× (2.78–3.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,370 | 2.0 | 0.55× (0.53–0.57) | 3.46× (3.32–3.60) | 4.56× (4.25–4.89) | 3.53× (3.44–3.63) | 7.02× (6.17–7.98) | 2.66× (2.56–2.77) | 2.66× (2.57–2.74) | 1.00× (0.98–1.03) | 2.08× (1.99–2.16) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,404 | 2.1 | 0.52× (0.50–0.53) | 2.88× (2.74–3.04) | 5.87× (5.66–6.09) | 3.63× (3.50–3.77) | 6.65× (6.50–6.80) | 2.71× (2.62–2.80) | 2.70× (2.60–2.81) | 1.02× (0.98–1.06) | 2.11× (2.04–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,370 | 2.1 | 0.56× (0.49–0.64) | 2.85× (2.70–3.02) | 5.79× (5.67–5.91) | 3.60× (3.48–3.71) | 6.57× (6.42–6.73) | 2.67× (2.63–2.72) | 2.67× (2.60–2.73) | 1.01× (0.99–1.02) | 2.08× (2.02–2.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.022 s | 875 | 3.1 | 2.38× (2.36–2.40) | 3.11× (3.01–3.21) | 3.38× (3.29–3.48) | 3.69× (3.67–3.71) | 4.65× (4.58–4.72) | 2.28× (2.26–2.31) | 2.22× (2.09–2.37) | 1.72× (1.69–1.76) | 2.70× (2.65–2.75) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,315 | 4.0 | 1.84× (1.62–2.09) | 2.55× (2.24–2.90) | 3.59× (3.10–4.16) | 3.13× (2.76–3.54) | 5.34× (4.52–6.32) | 3.21× (2.87–3.59) | 3.12× (2.70–3.61) | 2.42× (2.14–2.74) | 3.79× (3.33–4.31) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,034 | 8.3 | 1.46× (1.29–1.65) | 1.63× (1.55–1.72) | 2.43× (2.33–2.53) | 1.90× (1.82–1.98) | 3.87× (3.73–4.02) | 2.66× (2.56–2.78) | 2.59× (2.42–2.77) | 2.01× (1.92–2.10) | 3.15× (2.97–3.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.086 s | 92 | 28.5 | 1.20× (1.10–1.31) | 2.74× (2.52–2.99) | 2.45× (2.23–2.69) | 2.41× (2.21–2.62) | 2.78× (2.55–3.03) | 3.27× (2.99–3.57) | 3.00× (2.75–3.26) | 1.20× (1.11–1.31) | 1.28× (1.16–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 151 | 31.4 | 1.16× (1.13–1.20) | 2.70× (2.63–2.78) | 2.54× (2.40–2.69) | 3.17× (2.98–3.36) | 3.11× (2.91–3.33) | 5.52× (5.33–5.71) | 5.06× (4.91–5.21) | 2.03× (1.98–2.09) | 2.16× (2.09–2.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 186 | 43.0 | 1.18× (1.16–1.21) | 2.28× (2.20–2.37) | 2.29× (2.19–2.40) | 2.71× (2.60–2.82) | 3.08× (3.02–3.13) | 6.71× (6.48–6.94) | 6.15× (6.01–6.29) | 2.47× (2.43–2.52) | 2.62× (2.56–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 86 | 30.4 | 1.10× (1.07–1.12) | 2.81× (2.68–2.94) | 2.22× (2.17–2.28) | 2.29× (2.23–2.36) | 2.64× (2.57–2.72) | 3.52× (3.44–3.60) | 3.45× (3.36–3.53) | 1.24× (1.20–1.27) | 1.36× (1.33–1.40) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.068 s | 140 | 34.1 | 1.04× (0.95–1.14) | 2.53× (2.42–2.64) | 2.16× (2.08–2.23) | 3.07× (2.86–3.29) | 2.72× (2.58–2.88) | 5.69× (5.49–5.89) | 5.58× (5.38–5.78) | 2.00× (1.94–2.06) | 2.21× (2.15–2.26) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.053 s | 181 | 46.4 | 1.11× (1.04–1.18) | 2.20× (2.08–2.32) | 2.11× (1.99–2.25) | 2.82× (2.62–3.05) | 2.87× (2.66–3.08) | 7.15× (6.70–7.63) | 7.01× (6.55–7.49) | 2.51× (2.34–2.69) | 2.77× (2.60–2.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 2.9 | 0.59× (0.58–0.60) | 3.06× (2.94–3.19) | 4.31× (4.12–4.51) | 2.98× (2.90–3.06) | 7.34× (7.11–7.58) | 2.29× (2.10–2.51) | 2.21× (2.13–2.30) | 0.99× (0.97–1.00) | 2.05× (1.97–2.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,010 | 3.0 | 0.46× (0.36–0.58) | 2.80× (2.20–3.56) | 4.89× (3.83–6.23) | 2.69× (2.14–3.40) | 6.56× (5.19–8.29) | 2.09× (1.62–2.70) | 2.02× (1.60–2.54) | 0.90× (0.72–1.13) | 1.87× (1.47–2.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 3.0 | 0.50× (0.48–0.51) | 3.01× (2.90–3.14) | 5.25× (5.03–5.47) | 2.92× (2.82–3.02) | 7.18× (6.91–7.46) | 2.25× (2.08–2.43) | 2.17× (2.10–2.24) | 0.97× (0.95–0.98) | 2.01× (1.93–2.09) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 0.047 s | 260 | 10.3 | 1.34× (1.32–1.36) | 3.22× (3.15–3.30) | 3.48× (3.42–3.54) | 3.55× (3.49–3.61) | 4.06× (3.99–4.12) | 3.75× (3.63–3.87) | 3.35× (3.29–3.41) | 1.13× (1.11–1.15) | 1.31× (1.28–1.35) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 0.030 s | 403 | 12.1 | 1.14× (1.10–1.17) | 2.72× (2.58–2.86) | 3.20× (2.99–3.41) | 3.83× (3.43–4.28) | 4.02× (3.86–4.20) | 5.77× (5.51–6.04) | 5.16× (4.94–5.39) | 1.74× (1.68–1.81) | 2.02× (1.96–2.08) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 0.028 s | 431 | 19.6 | 1.04× (1.01–1.07) | 2.02× (1.84–2.21) | 2.61× (2.51–2.72) | 2.89× (2.82–2.96) | 3.56× (3.43–3.69) | 6.28× (5.94–6.64) | 5.62× (5.39–5.85) | 1.90× (1.84–1.96) | 2.20× (2.10–2.30) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.13× (2.03–2.23) | 5.30× (5.07–5.53) | 5.49× (5.20–5.80) | 6.28× (6.04–6.53) | 5.58× (5.31–5.86) | 5.87× (5.59–6.16) | 5.47× (5.25–5.70) | 1.66× (1.59–1.73) | 1.91× (1.84–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.91× (1.78–2.04) | 4.74× (4.43–5.07) | 4.96× (4.67–5.27) | 5.76× (5.43–6.11) | 4.87× (4.53–5.23) | 5.04× (4.68–5.44) | 4.70× (4.38–5.05) | 1.42× (1.34–1.52) | 1.64× (1.53–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.60× (1.57–1.64) | 3.09× (2.99–3.19) | 3.32× (3.21–3.45) | 3.78× (3.59–3.97) | 3.29× (3.14–3.45) | 2.74× (2.59–2.89) | 2.55× (2.42–2.68) | 0.77× (0.74–0.81) | 0.89× (0.85–0.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.58–0.62) | 2.98× (2.88–3.09) | 3.96× (3.71–4.24) | 3.35× (3.26–3.43) | 6.59× (5.81–7.48) | 2.55× (2.46–2.64) | 2.54× (2.45–2.62) | 1.01× (0.98–1.04) | 1.81× (1.75–1.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.81× (2.72–2.89) | 5.06× (4.91–5.23) | 3.37× (3.25–3.48) | 6.05× (5.93–6.18) | 2.52× (2.44–2.59) | 2.50× (2.42–2.59) | 0.99× (0.96–1.03) | 1.79× (1.72–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.53–0.69) | 3.04× (2.96–3.11) | 5.12× (5.00–5.24) | 3.42× (3.31–3.52) | 6.12× (5.97–6.26) | 2.55× (2.50–2.59) | 2.53× (2.48–2.59) | 1.01× (1.00–1.02) | 1.81× (1.77–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.30× (2.27–2.32) | 2.69× (2.66–2.72) | 3.16× (3.12–3.20) | 3.54× (3.50–3.57) | 4.44× (4.38–4.51) | 2.22× (2.20–2.24) | 2.16× (2.02–2.31) | 1.68× (1.64–1.72) | 2.28× (2.25–2.32) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.89× (1.81–1.97) | 2.24× (2.14–2.34) | 2.67× (2.59–2.76) | 2.91× (2.79–3.03) | 3.70× (3.25–4.21) | 1.70× (1.64–1.78) | 1.66× (1.53–1.80) | 1.29× (1.22–1.36) | 1.75× (1.67–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.65× (1.44–1.88) | 1.52× (1.45–1.60) | 1.81× (1.73–1.91) | 1.82× (1.73–1.90) | 1.99× (1.90–2.08) | 0.85× (0.81–0.90) | 0.83× (0.76–0.90) | 0.64× (0.62–0.68) | 0.88× (0.83–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.20× (1.10–1.31) | 2.67× (2.46–2.91) | 2.35× (2.14–2.58) | 2.39× (2.20–2.60) | 2.75× (2.53–3.00) | 3.24× (2.96–3.54) | 2.97× (2.73–3.22) | 1.20× (1.10–1.31) | 1.24× (1.14–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.17× (1.14–1.19) | 2.65× (2.62–2.69) | 2.26× (2.20–2.31) | 2.95× (2.82–3.09) | 2.67× (2.53–2.80) | 3.01× (2.94–3.08) | 2.76× (2.71–2.81) | 1.12× (1.10–1.13) | 1.15× (1.13–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.28× (1.24–1.31) | 2.36× (2.33–2.40) | 2.02× (1.94–2.09) | 2.62× (2.53–2.71) | 2.48× (2.43–2.54) | 2.22× (2.17–2.27) | 2.04× (2.02–2.06) | 0.82× (0.82–0.83) | 0.85× (0.84–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.10× (1.07–1.12) | 2.77× (2.65–2.89) | 2.17× (2.11–2.23) | 2.28× (2.22–2.35) | 2.62× (2.55–2.70) | 3.49× (3.41–3.57) | 3.42× (3.34–3.50) | 1.23× (1.20–1.27) | 1.30× (1.26–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.04× (1.01–1.06) | 2.53× (2.47–2.60) | 2.01× (1.97–2.05) | 2.87× (2.77–2.98) | 2.43× (2.35–2.51) | 3.08× (3.03–3.13) | 3.02× (2.95–3.09) | 1.09× (1.07–1.11) | 1.15× (1.11–1.20) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.18× (1.12–1.24) | 2.31× (2.23–2.40) | 1.74× (1.66–1.83) | 2.53× (2.36–2.72) | 2.32× (2.22–2.42) | 2.23× (2.13–2.33) | 2.18× (2.08–2.29) | 0.79× (0.75–0.83) | 0.83× (0.79–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.63–0.65) | 2.44× (2.38–2.50) | 3.30× (3.21–3.40) | 2.74× (2.65–2.82) | 6.58× (6.34–6.83) | 2.15× (1.96–2.36) | 2.06× (1.97–2.15) | 0.98× (0.96–1.00) | 1.44× (1.40–1.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.52× (0.41–0.65) | 2.11× (1.65–2.68) | 3.64× (2.88–4.61) | 2.47× (1.96–3.11) | 5.73× (4.55–7.23) | 1.90× (1.48–2.44) | 1.82× (1.45–2.28) | 0.87× (0.70–1.09) | 1.28× (1.02–1.61) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.57) | 2.47× (2.42–2.53) | 4.01× (3.90–4.11) | 2.78× (2.69–2.86) | 6.51× (6.28–6.75) | 2.12× (1.97–2.28) | 2.03× (1.97–2.09) | 0.97× (0.95–0.99) | 1.43× (1.40–1.46) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 1.33× (1.31–1.35) | 3.04× (2.99–3.10) | 3.28× (3.23–3.32) | 3.49× (3.43–3.55) | 3.98× (3.92–4.04) | 3.68× (3.56–3.80) | 3.29× (3.23–3.35) | 1.13× (1.11–1.15) | 1.25× (1.23–1.27) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 1.19× (1.16–1.23) | 2.63× (2.56–2.70) | 2.91× (2.81–3.00) | 3.28× (3.20–3.37) | 3.38× (3.29–3.47) | 3.08× (2.96–3.20) | 2.75× (2.67–2.84) | 0.94× (0.93–0.96) | 1.04× (1.02–1.07) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 1.18× (1.16–1.21) | 2.15× (2.00–2.32) | 2.32× (2.25–2.38) | 2.50× (2.46–2.55) | 2.58× (2.48–2.68) | 1.92× (1.81–2.03) | 1.72× (1.64–1.80) | 0.59× (0.57–0.61) | 0.65× (0.62–0.68) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.35× (0.34–0.36) | 1.16× (1.16–1.17) | 0.86× (0.84–0.88) | 11.83× (11.82–11.84) | 0.28× (0.28–0.28) | 0.30× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.55× (0.49–0.63) | 0.86× (0.85–0.87) | 0.68× (0.67–0.69) | 6.35× (6.26–6.44) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.75× (0.63–0.89) | 0.76× (0.76–0.77) | 0.64× (0.63–0.65) | 4.26× (4.24–4.28) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.28–4.28) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.45× (0.43–0.47) | 1.63× (1.62–1.63) | 0.64× (0.64–0.64) | 7.76× (7.74–7.78) | 0.62× (0.62–0.62) | 0.65× (0.65–0.66) | 0.71× (0.71–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.63× (0.61–0.65) | 0.63× (0.57–0.69) | 1.51× (1.49–1.53) | 0.87× (0.85–0.89) | 5.48× (5.42–5.55) | 0.44× (0.43–0.44) | 0.46× (0.45–0.47) | 0.50× (0.49–0.51) | 0.25× (0.24–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.59–0.62) | 0.65× (0.62–0.68) | 1.10× (1.08–1.12) | 0.85× (0.83–0.86) | 3.18× (3.13–3.23) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.28) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.14× (1.14–1.14) | 0.71× (0.71–0.71) | 5.05× (5.05–5.06) | 0.31× (0.31–0.31) | 0.33× (0.33–0.34) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.44× (0.43–0.44) | 0.93× (0.92–0.94) | 0.75× (0.74–0.76) | 3.25× (3.22–3.28) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 52 MB | 0.60× (0.59–0.61) | 0.44× (0.42–0.45) | 0.82× (0.80–0.84) | 0.76× (0.73–0.79) | 2.41× (2.37–2.45) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.32× (0.31–0.32) | 1.15× (1.15–1.15) | 0.81× (0.81–0.81) | 5.09× (5.08–5.10) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.50× (0.50–0.51) | 0.26× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.39× (0.36–0.41) | 0.87× (0.85–0.88) | 0.74× (0.72–0.76) | 3.01× (2.95–3.06) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.54× (0.53–0.55) | 0.44× (0.41–0.47) | 0.72× (0.71–0.73) | 0.74× (0.71–0.76) | 2.10× (2.08–2.12) | 0.12× (0.12–0.13) | 0.13× (0.13–0.13) | 0.20× (0.20–0.20) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.40× (5.39–5.41) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.38× (5.37–5.38) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.76–0.77) | 5.39× (5.38–5.41) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.45× (1.45–1.45) | 0.63× (0.63–0.64) | 6.55× (6.55–6.56) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.65× (0.64–0.65) | 0.32× (0.32–0.32) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 31 MB | 0.51× (0.50–0.53) | 0.41× (0.38–0.45) | 1.25× (1.22–1.28) | 0.77× (0.75–0.79) | 3.95× (3.87–4.03) | 0.24× (0.23–0.24) | 0.25× (0.25–0.26) | 0.39× (0.38–0.40) | 0.19× (0.19–0.20) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 50 MB | 0.53× (0.52–0.55) | 0.50× (0.46–0.54) | 1.01× (0.99–1.04) | 0.81× (0.79–0.83) | 2.59× (2.52–2.65) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 538 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 765 | 2.13 | 1.16× (1.09–1.24) | 70% | 1.04× (1.03–1.05) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 824 | 3.91 | 2.14× (2.06–2.23) | 38% | 1.25× (1.22–1.28) | 1.61× (1.52–1.71) |

**Best:** mibz2 at 4 threads (824 MB/s); lbzip2 at 4 threads (308 MB/s); pbz2 at 4 threads (540 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.04, lbzip2 1t 0.08, lbzip2 2t 0.28, lbzip2 4t 0.46, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.68 | 0.01 | 0.08 | 0.255 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 44.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.64 | 11.6% |
| MTF + RLE2 | 1.16 | 20.9% |
| T-vector build | 0.90 | 16.2% |
| walk (the one each build uses) | 2.60 | 47.0% |
| of which: segment assembly | 0.09 | 1.6% |
| RLE1 | 0.09 | 1.7% |
| CRC | 0.16 | 2.9% |
| **total** | 5.53 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.344 | 0.717 | (in its total) | 2.13× (2.04–2.23) |  |
| walk | 0.799 | 1.623 | (in its total) | 2.04× (1.92–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.031 | (in its total) | 1.20× (1.12–1.29) |  |
| inverse BWT, total | 1.170 | 2.372 | 1.351 | 2.05× (1.95–2.15) | 1.16× (1.12–1.20) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 58m13s ± 7m47s |
| 4 threads | 38m38s ± 6m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m39s ± 4m05s |
| the scanner alone (its serial part) | 8m53s ± 46.1s |
| the workers alone, over 4 | 25m39s ± 4m05s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 3.36 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,478,819 | 12,273,030 | \`80f51ade7d6ce928…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
