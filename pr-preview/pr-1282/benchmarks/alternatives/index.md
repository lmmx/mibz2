# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`7932a3e`](<https://github.com/lmmx/mibz2/commit/7932a3ef5f16731d87cbe18e5afd98672f096936>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46150594673>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.477 s | 439 | 7.4 | 3.18× (3.16–3.20) | 7.20× (7.17–7.23) | 7.40× (7.37–7.44) | 8.05× (8.01–8.10) | 7.24× (7.20–7.28) | 7.85× (7.80–7.91) | 7.25× (7.22–7.28) | 2.33× (2.32–2.34) | 2.75× (2.73–2.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.265 s | 792 | 8.0 | 3.19× (2.85–3.57) | 6.45× (5.81–7.15) | 7.60× (6.85–8.42) | 7.48× (6.81–8.21) | 6.69× (6.02–7.44) | 13.49× (12.09–15.06) | 12.46× (11.22–13.83) | 4.01× (3.60–4.45) | 4.73× (4.26–5.25) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.218 s | 964 | 12.6 | 3.23× (3.18–3.28) | 4.87× (4.81–4.94) | 6.09× (6.00–6.18) | 5.57× (5.45–5.69) | 5.55× (5.47–5.63) | 17.12× (16.86–17.39) | 15.81× (15.59–16.03) | 5.08× (5.02–5.15) | 6.00× (5.90–6.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 968 | 3.7 | 0.59× (0.58–0.59) | 3.55× (3.42–3.70) | 4.39× (4.33–4.44) | 3.53× (3.51–3.56) | 6.11× (6.06–6.16) | 2.94× (2.92–2.97) | 2.63× (2.61–2.66) | 1.00× (0.99–1.01) | 1.49× (1.48–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 917 | 4.0 | 0.53× (0.53–0.54) | 2.81× (2.77–2.86) | 5.59× (5.49–5.69) | 3.35× (3.30–3.41) | 5.73× (5.66–5.80) | 2.77× (2.73–2.82) | 2.48× (2.44–2.52) | 0.95× (0.93–0.96) | 1.41× (1.38–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 891 | 4.0 | 0.53× (0.52–0.54) | 2.76× (2.72–2.80) | 5.47× (5.34–5.60) | 3.28× (3.23–3.32) | 5.66× (5.60–5.73) | 2.71× (2.67–2.75) | 2.42× (2.39–2.46) | 0.92× (0.91–0.94) | 1.37× (1.35–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 606 | 5.7 | 2.86× (2.76–2.97) | 3.13× (3.08–3.18) | 4.01× (3.95–4.07) | 3.96× (3.90–4.01) | 4.31× (4.25–4.37) | 2.63× (2.59–2.68) | 2.18× (2.14–2.21) | 2.07× (2.05–2.10) | 3.11× (3.06–3.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 933 | 6.8 | 2.22× (1.95–2.53) | 2.93× (2.49–3.44) | 4.01× (3.52–4.56) | 3.34× (2.93–3.80) | 4.72× (4.14–5.38) | 3.74× (3.29–4.25) | 3.09× (2.71–3.53) | 2.95× (2.59–3.36) | 4.42× (3.88–5.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 885 | 13.4 | 2.08× (1.99–2.18) | 2.16× (2.01–2.33) | 3.24× (3.03–3.47) | 2.34× (2.21–2.47) | 4.39× (4.16–4.63) | 3.79× (3.58–4.01) | 3.13× (2.97–3.31) | 2.99× (2.83–3.14) | 4.48× (4.24–4.73) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.122 s | 65 | 50.5 | 1.44× (1.43–1.45) | 2.85× (2.83–2.88) | 2.59× (2.54–2.65) | 2.65× (2.63–2.68) | 3.06× (3.00–3.12) | 3.34× (3.30–3.37) | 3.31× (3.28–3.34) | 1.31× (1.30–1.32) | 1.51× (1.49–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 s | 110 | 55.8 | 1.43× (1.30–1.57) | 2.83× (2.61–3.08) | 2.53× (2.40–2.67) | 3.36× (3.13–3.61) | 3.19× (3.00–3.39) | 5.49× (5.19–5.80) | 5.45× (5.15–5.77) | 2.15× (2.03–2.28) | 2.48× (2.34–2.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 129 | 77.9 | 1.44× (1.41–1.47) | 2.35× (2.31–2.39) | 2.23× (2.15–2.31) | 2.79× (2.76–2.82) | 3.13× (3.11–3.15) | 6.60× (6.55–6.65) | 6.56× (6.49–6.62) | 2.59× (2.55–2.62) | 2.99× (2.94–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.135 s | 71 | 46.4 | 1.56× (1.55–1.58) | 3.24× (3.22–3.26) | 2.70× (2.67–2.72) | 2.83× (2.81–2.84) | 3.28× (3.27–3.30) | 3.86× (3.85–3.87) | 4.10× (4.07–4.13) | 1.49× (1.48–1.49) | 1.79× (1.78–1.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.081 s | 118 | 52.5 | 1.44× (1.38–1.49) | 2.96× (2.79–3.15) | 2.63× (2.53–2.73) | 3.80× (3.59–4.02) | 3.27× (3.17–3.38) | 6.35× (6.15–6.55) | 6.74× (6.52–6.96) | 2.45× (2.37–2.52) | 2.95× (2.85–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 74.7 | 1.53× (1.49–1.58) | 2.49× (2.43–2.55) | 2.47× (2.39–2.56) | 3.26× (3.17–3.35) | 3.07× (3.03–3.11) | 7.48× (7.37–7.59) | 7.94× (7.80–8.08) | 2.88× (2.84–2.93) | 3.47× (3.42–3.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.1 | 0.66× (0.66–0.67) | 3.09× (2.94–3.24) | 4.42× (4.36–4.48) | 3.11× (3.07–3.16) | 6.81× (6.73–6.90) | 2.52× (2.49–2.56) | 2.31× (2.28–2.34) | 1.00× (0.98–1.01) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 667 | 5.8 | 0.53× (0.52–0.54) | 2.76× (2.72–2.80) | 4.78× (4.67–4.90) | 2.95× (2.73–3.18) | 6.22× (6.08–6.37) | 2.30× (2.26–2.33) | 2.10× (2.07–2.13) | 0.91× (0.89–0.93) | 1.38× (1.36–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 5.8 | 0.53× (0.53–0.54) | 2.72× (2.68–2.76) | 4.66× (4.59–4.73) | 2.81× (2.77–2.85) | 6.16× (6.06–6.27) | 2.26× (2.23–2.29) | 2.07× (2.04–2.10) | 0.89× (0.88–0.91) | 1.36× (1.34–1.38) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 0.063 s | 240 | 13.9 | 2.26× (2.24–2.28) | 4.42× (4.34–4.50) | 4.81× (4.76–4.85) | 4.90× (4.85–4.95) | 5.07× (5.03–5.11) | 5.47× (5.40–5.54) | 4.97× (4.90–5.04) | 1.70× (1.68–1.72) | 2.15× (2.09–2.21) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 0.045 s | 334 | 18.3 | 1.74× (1.69–1.79) | 3.53× (3.36–3.71) | 4.19× (4.03–4.35) | 4.77× (4.42–5.16) | 4.41× (4.34–4.49) | 7.66× (7.57–7.76) | 6.96× (6.90–7.03) | 2.38× (2.34–2.41) | 3.01× (2.92–3.11) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 0.040 s | 382 | 28.0 | 1.72× (1.66–1.79) | 2.76× (2.60–2.93) | 3.67× (3.56–3.78) | 3.79× (3.69–3.89) | 4.02× (3.91–4.13) | 8.79× (8.59–8.99) | 7.98× (7.79–8.18) | 2.73× (2.67–2.79) | 3.45× (3.33–3.58) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.17× (3.15–3.19) | 7.16× (7.14–7.19) | 7.36× (7.33–7.40) | 8.02× (7.98–8.07) | 7.21× (7.17–7.25) | 7.82× (7.77–7.88) | 7.23× (7.20–7.25) | 2.33× (2.31–2.34) | 2.73× (2.71–2.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.04–3.09) | 6.83× (6.79–6.88) | 7.15× (7.10–7.21) | 7.93× (7.82–8.04) | 6.80× (6.70–6.89) | 7.27× (7.20–7.35) | 6.72× (6.67–6.77) | 2.16× (2.15–2.18) | 2.54× (2.51–2.57) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.05× (3.01–3.09) | 5.10× (5.03–5.16) | 5.47× (5.41–5.53) | 5.79× (5.72–5.85) | 5.10× (5.05–5.15) | 4.61× (4.56–4.67) | 4.26× (4.22–4.30) | 1.37× (1.36–1.39) | 1.61× (1.59–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.62–0.63) | 3.23× (3.20–3.26) | 4.02× (3.98–4.06) | 3.31× (3.29–3.33) | 5.66× (5.61–5.71) | 2.77× (2.75–2.79) | 2.49× (2.47–2.51) | 1.00× (0.99–1.01) | 1.07× (1.02–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.84× (2.80–2.88) | 4.96× (4.91–5.01) | 3.06× (3.02–3.11) | 5.15× (5.09–5.21) | 2.53× (2.49–2.56) | 2.27× (2.24–2.30) | 0.92× (0.90–0.93) | 0.98× (0.93–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.08× (3.03–3.13) | 4.94× (4.87–5.02) | 3.05× (3.01–3.09) | 5.16× (5.10–5.21) | 2.51× (2.48–2.55) | 2.26× (2.23–2.29) | 0.91× (0.89–0.93) | 0.97× (0.92–1.02) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.75× (2.65–2.86) | 2.87× (2.83–2.92) | 3.60× (3.55–3.64) | 3.79× (3.74–3.84) | 4.12× (4.06–4.17) | 2.54× (2.50–2.58) | 2.11× (2.08–2.14) | 2.01× (1.99–2.04) | 2.79× (2.75–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.39× (2.33–2.46) | 2.67× (2.60–2.73) | 3.22× (3.15–3.28) | 3.34× (3.28–3.41) | 3.59× (3.52–3.67) | 2.12× (2.08–2.16) | 1.76× (1.72–1.80) | 1.68× (1.64–1.71) | 2.32× (2.28–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.05–2.22) | 1.90× (1.85–1.96) | 2.38× (2.29–2.47) | 2.24× (2.16–2.32) | 2.27× (2.20–2.35) | 1.11× (1.07–1.15) | 0.92× (0.89–0.96) | 0.88× (0.85–0.91) | 1.22× (1.18–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.43× (1.42–1.45) | 2.79× (2.77–2.82) | 2.53× (2.48–2.57) | 2.63× (2.60–2.65) | 3.03× (2.97–3.09) | 3.30× (3.27–3.33) | 3.28× (3.25–3.30) | 1.30× (1.29–1.32) | 1.46× (1.45–1.47) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.42× (1.32–1.52) | 2.72× (2.59–2.87) | 2.35× (2.34–2.37) | 3.15× (3.12–3.19) | 2.78× (2.75–2.81) | 2.98× (2.95–3.01) | 2.96× (2.93–2.99) | 1.18× (1.16–1.19) | 1.32× (1.30–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.52–1.57) | 2.42× (2.39–2.45) | 1.94× (1.92–1.96) | 2.70× (2.66–2.75) | 2.57× (2.53–2.61) | 2.14× (2.12–2.16) | 2.12× (2.11–2.14) | 0.85× (0.83–0.86) | 0.95× (0.94–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.55× (1.54–1.57) | 3.17× (3.15–3.18) | 2.65× (2.63–2.67) | 2.80× (2.79–2.82) | 3.25× (3.24–3.26) | 3.82× (3.81–3.83) | 4.05× (4.03–4.08) | 1.48× (1.48–1.49) | 1.76× (1.75–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.42–1.46) | 2.94× (2.91–2.98) | 2.44× (2.41–2.47) | 3.47× (3.38–3.55) | 2.93× (2.88–2.98) | 3.37× (3.34–3.40) | 3.58× (3.55–3.62) | 1.31× (1.29–1.32) | 1.55× (1.53–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.63–1.69) | 2.66× (2.60–2.72) | 2.06× (2.02–2.10) | 2.95× (2.89–3.01) | 2.67× (2.64–2.71) | 2.37× (2.33–2.41) | 2.51× (2.47–2.56) | 0.92× (0.90–0.93) | 1.09× (1.07–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.70–0.71) | 2.70× (2.67–2.73) | 3.56× (3.53–3.58) | 2.83× (2.81–2.86) | 6.05× (6.00–6.11) | 2.32× (2.31–2.34) | 2.13× (2.11–2.16) | 1.00× (0.98–1.01) | 1.00× (0.95–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.36× (2.31–2.42) | 4.00× (3.95–4.06) | 2.66× (2.47–2.88) | 5.37× (5.25–5.50) | 2.04× (2.01–2.07) | 1.88× (1.85–1.91) | 0.88× (0.86–0.90) | 0.88× (0.83–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.53× (2.48–2.58) | 3.98× (3.91–4.05) | 2.58× (2.54–2.63) | 5.41× (5.30–5.53) | 2.05× (2.01–2.08) | 1.88× (1.85–1.92) | 0.88× (0.86–0.90) | 0.88× (0.83–0.93) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 2.23× (2.21–2.25) | 4.25× (4.21–4.30) | 4.64× (4.60–4.68) | 4.78× (4.73–4.83) | 4.95× (4.91–4.98) | 5.34× (5.27–5.40) | 4.85× (4.79–4.91) | 1.68× (1.66–1.70) | 2.06× (2.03–2.09) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 1.80× (1.79–1.82) | 3.39× (3.35–3.43) | 3.74× (3.69–3.78) | 4.18× (4.09–4.28) | 3.87× (3.83–3.92) | 4.07× (4.04–4.11) | 3.70× (3.66–3.74) | 1.28× (1.27–1.30) | 1.57× (1.56–1.58) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 1.84× (1.78–1.90) | 2.75× (2.68–2.81) | 3.02× (2.96–3.08) | 3.24× (3.19–3.30) | 3.07× (3.02–3.13) | 2.66× (2.62–2.71) | 2.42× (2.38–2.46) | 0.84× (0.83–0.85) | 1.03× (1.01–1.05) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.99–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.55× (0.47–0.65) | 0.73× (0.71–0.76) | 0.61× (0.59–0.63) | 5.79× (5.61–5.98) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.35–0.39) | 0.70× (0.62–0.79) | 0.66× (0.62–0.70) | 0.58× (0.55–0.61) | 3.80× (3.62–4.00) | 0.09× (0.08–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.17) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.93–0.93) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.94× (7.92–7.96) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.79× (0.73–0.86) | 1.69× (1.65–1.74) | 0.85× (0.83–0.87) | 6.13× (5.96–6.30) | 0.50× (0.48–0.51) | 0.52× (0.51–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.72) | 0.80× (0.74–0.86) | 1.23× (1.17–1.30) | 0.77× (0.74–0.81) | 3.64× (3.47–3.81) | 0.29× (0.27–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.39× (0.36–0.42) | 0.90× (0.87–0.93) | 0.71× (0.67–0.74) | 3.10× (3.01–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.90) | 0.82× (0.80–0.84) | 2.58× (2.54–2.62) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.20× (5.19–5.21) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.37× (0.34–0.39) | 0.83× (0.80–0.86) | 0.70× (0.68–0.73) | 2.85× (2.76–2.94) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.50) | 0.80× (0.79–0.81) | 0.82× (0.80–0.84) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.30× (5.29–5.32) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.19× (6.17–6.20) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.91) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.65× (0.65–0.66) | 6.36× (6.34–6.38) | 0.39× (0.39–0.39) | 0.40× (0.40–0.41) | 0.62× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 35 MB | 0.47× (0.47–0.48) | 0.43× (0.39–0.48) | 1.07× (1.05–1.09) | 0.72× (0.70–0.73) | 3.54× (3.48–3.60) | 0.22× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.35) | 0.17× (0.16–0.17) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.47× (0.44–0.51) | 0.91× (0.88–0.95) | 0.75× (0.72–0.78) | 2.35× (2.29–2.41) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 439 | 2.29 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 792 | 2.46 | 1.08× (1.07–1.08) | 86% | 1.03× (1.02–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 964 | 3.89 | 1.70× (1.68–1.71) | 55% | 1.21× (1.20–1.21) | 1.63× (1.60–1.66) |

**Best:** mibz2 at 4 threads (964 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.01, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.35, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.09 | 0.218 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3246 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.4% |
| MTF + RLE2 | 1.53 | 17.9% |
| T-vector build | 1.52 | 17.7% |
| walk (the one each build uses) | 4.16 | 48.7% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.56 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.508 | 1.319 | (in its total) | 2.60× (2.59–2.61) |  |
| walk | 1.193 | 2.626 | (in its total) | 2.21× (2.19–2.23) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.23× (1.22–1.24) |  |
| inverse BWT, total | 1.745 | 3.998 | 3.278 | 2.30× (2.28–2.31) | 1.88× (1.87–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m29s |
| 4 threads | 31m54s ± 4m29s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m19s ± 3m18s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 23m19s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 3.06 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,269,316 | 15,087,510 | \`c5dc29887a3b9d42…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3245.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
