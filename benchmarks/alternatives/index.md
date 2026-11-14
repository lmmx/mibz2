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

The latest record: [`ca6f236`](<https://github.com/lmmx/mibz2/commit/ca6f2366aa26a21afce6bbf376ca9ce8a0da2a0a>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43605159961>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.432 s | 485 | 7.6 | 2.91× (2.80–3.03) | 6.66× (6.40–6.92) | 6.90× (6.71–7.09) | 7.77× (7.54–8.01) | 6.99× (6.66–7.34) | 7.20× (6.93–7.47) | 6.81× (6.58–7.04) | 2.13× (2.05–2.23) | 2.37× (2.30–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.267 s | 785 | 8.8 | 2.75× (2.61–2.89) | 5.63× (5.36–5.90) | 6.47× (6.11–6.86) | 6.69× (6.34–7.06) | 5.99× (5.70–6.29) | 11.71× (11.13–12.31) | 11.07× (10.52–11.65) | 3.47× (3.26–3.69) | 3.86× (3.65–4.07) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.222 s | 944 | 14.0 | 2.80× (2.74–2.87) | 4.28× (4.11–4.45) | 5.24× (5.06–5.43) | 4.98× (4.86–5.10) | 4.91× (4.77–5.04) | 14.11× (13.79–14.44) | 13.34× (13.00–13.68) | 4.18× (4.03–4.34) | 4.65× (4.50–4.80) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,105 | 3.7 | 0.56× (0.56–0.57) | 3.36× (3.31–3.42) | 4.48× (4.40–4.56) | 3.52× (3.46–3.57) | 6.03× (5.92–6.13) | 2.62× (2.58–2.67) | 2.61× (2.57–2.66) | 0.99× (0.96–1.02) | 1.74× (1.65–1.83) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,105 | 3.7 | 0.53× (0.52–0.54) | 2.82× (2.76–2.89) | 5.70× (5.51–5.89) | 3.54× (3.46–3.62) | 5.97× (5.84–6.11) | 2.62× (2.56–2.69) | 2.62× (2.55–2.68) | 0.99× (0.96–1.02) | 1.74× (1.62–1.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,081 | 3.7 | 0.53× (0.51–0.54) | 2.84× (2.70–2.99) | 5.56× (5.43–5.68) | 3.50× (3.41–3.58) | 5.98× (5.87–6.08) | 2.58× (2.53–2.64) | 2.58× (2.52–2.64) | 0.97× (0.94–1.00) | 1.71× (1.59–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 745 | 5.3 | 2.74× (2.72–2.77) | 3.09× (3.06–3.12) | 4.01× (3.80–4.23) | 4.13× (4.09–4.18) | 4.50× (4.46–4.55) | 2.79× (2.76–2.81) | 2.28× (2.26–2.30) | 2.13× (2.10–2.15) | 3.08× (3.05–3.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,029 | 6.6 | 2.18× (1.96–2.42) | 2.71× (2.51–2.93) | 4.14× (3.69–4.64) | 3.36× (3.13–3.61) | 4.88× (4.51–5.28) | 3.89× (3.60–4.20) | 3.18× (2.94–3.43) | 2.96× (2.76–3.19) | 4.29× (3.97–4.63) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 938 | 12.9 | 1.90× (1.81–1.99) | 1.81× (1.64–2.01) | 2.87× (2.66–3.09) | 2.09× (2.01–2.18) | 4.00× (3.81–4.20) | 3.50× (3.38–3.64) | 2.87× (2.75–2.98) | 2.67× (2.57–2.78) | 3.87× (3.72–4.02) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 52.4 | 1.39× (1.38–1.41) | 2.64× (2.61–2.67) | 2.40× (2.38–2.43) | 2.48× (2.44–2.51) | 2.79× (2.76–2.81) | 2.98× (2.95–3.01) | 3.07× (3.03–3.11) | 1.26× (1.25–1.28) | 1.41× (1.39–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 115 | 59.9 | 1.31× (1.22–1.40) | 2.48× (2.40–2.57) | 2.36× (2.30–2.42) | 2.99× (2.86–3.14) | 2.86× (2.77–2.95) | 4.79× (4.68–4.89) | 4.92× (4.81–5.04) | 2.02× (1.98–2.07) | 2.27× (2.20–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 78.4 | 1.33× (1.28–1.38) | 2.24× (2.14–2.34) | 2.24× (2.11–2.38) | 2.60× (2.48–2.72) | 2.99× (2.88–3.11) | 5.87× (5.71–6.03) | 6.04× (5.88–6.21) | 2.48× (2.42–2.55) | 2.78× (2.69–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.121 s | 79 | 47.4 | 1.50× (1.45–1.56) | 3.01× (2.90–3.14) | 2.58× (2.49–2.68) | 2.70× (2.61–2.80) | 3.07× (2.94–3.20) | 3.65× (3.50–3.80) | 3.91× (3.76–4.07) | 1.55× (1.50–1.61) | 1.79× (1.72–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.078 s | 124 | 53.6 | 1.30× (1.27–1.34) | 2.65× (2.55–2.74) | 2.46× (2.33–2.60) | 3.22× (3.10–3.35) | 2.96× (2.87–3.06) | 5.83× (5.65–6.01) | 6.25× (6.05–6.45) | 2.48× (2.41–2.56) | 2.86× (2.78–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 151 | 74.2 | 1.40× (1.35–1.45) | 2.26× (2.15–2.37) | 2.37× (2.15–2.62) | 2.93× (2.88–2.99) | 2.90× (2.78–3.04) | 6.97× (6.73–7.22) | 7.48× (7.24–7.72) | 2.97× (2.87–3.07) | 3.42× (3.32–3.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 830 | 5.1 | 0.62× (0.61–0.63) | 3.43× (3.39–3.48) | 4.31× (4.25–4.36) | 3.09× (3.06–3.13) | 6.93× (6.56–7.33) | 2.27× (2.24–2.29) | 2.31× (2.28–2.35) | 0.99× (0.92–1.06) | 1.73× (1.71–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 806 | 5.3 | 0.52× (0.50–0.54) | 2.49× (2.41–2.57) | 4.99× (4.84–5.15) | 3.01× (2.88–3.14) | 6.44× (6.19–6.70) | 2.18× (2.11–2.25) | 2.22× (2.15–2.30) | 0.95× (0.88–1.02) | 1.66× (1.61–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.53× (0.52–0.53) | 2.50× (2.48–2.53) | 5.01× (4.96–5.07) | 3.02× (2.99–3.06) | 6.39× (6.29–6.50) | 2.18× (2.16–2.21) | 2.23× (2.20–2.26) | 0.95× (0.89–1.02) | 1.67× (1.65–1.68) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 0.057 s | 215 | 17.7 | 1.80× (1.76–1.83) | 3.87× (3.78–3.96) | 4.21× (4.13–4.28) | 4.43× (4.36–4.49) | 4.59× (4.52–4.66) | 4.54× (4.47–4.61) | 4.50× (4.44–4.56) | 1.50× (1.47–1.54) | 1.64× (1.60–1.69) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 0.040 s | 305 | 22.3 | 1.37× (1.30–1.44) | 3.00× (2.73–3.30) | 3.79× (3.60–4.00) | 4.16× (3.90–4.43) | 3.89× (3.69–4.10) | 6.21× (5.84–6.59) | 6.16× (5.86–6.47) | 2.05× (1.94–2.17) | 2.24× (2.10–2.39) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 0.034 s | 358 | 33.2 | 1.44× (1.38–1.50) | 2.46× (2.34–2.59) | 3.51× (3.37–3.65) | 3.48× (3.38–3.58) | 3.75× (3.60–3.90) | 7.42× (7.18–7.68) | 7.36× (7.14–7.59) | 2.46× (2.35–2.56) | 2.69× (2.57–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.79–3.02) | 6.62× (6.37–6.88) | 6.86× (6.67–7.06) | 7.75× (7.52–7.98) | 6.96× (6.63–7.31) | 7.17× (6.91–7.44) | 6.78× (6.55–7.02) | 2.13× (2.04–2.22) | 2.35× (2.28–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.66× (2.58–2.75) | 5.86× (5.73–5.99) | 6.18× (5.97–6.39) | 7.07× (6.91–7.24) | 6.06× (5.89–6.23) | 6.28× (6.13–6.43) | 5.94× (5.77–6.11) | 1.86× (1.79–1.94) | 2.06× (1.98–2.14) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.75× (2.69–2.81) | 4.60× (4.43–4.76) | 5.02× (4.81–5.25) | 5.38× (5.25–5.52) | 4.71× (4.58–4.84) | 3.93× (3.82–4.04) | 3.72× (3.62–3.81) | 1.17× (1.13–1.21) | 1.29× (1.25–1.33) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.90× (2.84–2.95) | 3.90× (3.82–3.97) | 3.30× (3.25–3.34) | 5.59× (5.49–5.69) | 2.48× (2.44–2.52) | 2.47× (2.44–2.51) | 0.99× (0.96–1.01) | 1.61× (1.59–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.73× (2.67–2.78) | 5.02× (4.87–5.18) | 3.29× (3.22–3.37) | 5.47× (5.35–5.58) | 2.44× (2.39–2.50) | 2.44× (2.38–2.49) | 0.97× (0.94–1.00) | 1.59× (1.55–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.10× (2.89–3.33) | 4.98× (4.90–5.07) | 3.33× (3.25–3.40) | 5.57× (5.49–5.66) | 2.46× (2.41–2.51) | 2.45× (2.40–2.50) | 0.98× (0.95–1.01) | 1.60× (1.57–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.61–2.66) | 2.85× (2.83–2.87) | 3.57× (3.35–3.81) | 3.94× (3.90–3.98) | 4.28× (4.24–4.32) | 2.67× (2.65–2.69) | 2.20× (2.18–2.22) | 2.05× (2.03–2.08) | 2.68× (2.65–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.14–2.34) | 2.47× (2.36–2.58) | 3.30× (3.16–3.44) | 3.28× (3.15–3.43) | 3.57× (3.40–3.74) | 2.11× (2.02–2.21) | 1.74× (1.66–1.82) | 1.63× (1.56–1.70) | 2.12× (2.03–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (1.98–2.29) | 1.71× (1.65–1.78) | 2.26× (2.19–2.34) | 2.07× (2.00–2.14) | 2.16× (2.09–2.23) | 1.10× (1.06–1.14) | 0.91× (0.88–0.94) | 0.85× (0.82–0.87) | 1.10× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.37–1.40) | 2.56× (2.53–2.59) | 2.33× (2.32–2.35) | 2.45× (2.42–2.49) | 2.76× (2.74–2.79) | 2.95× (2.92–2.98) | 3.04× (3.00–3.07) | 1.26× (1.24–1.27) | 1.36× (1.35–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.23–1.39) | 2.43× (2.39–2.47) | 2.14× (2.11–2.17) | 2.79× (2.75–2.83) | 2.52× (2.48–2.56) | 2.61× (2.57–2.65) | 2.68× (2.65–2.72) | 1.11× (1.10–1.13) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.53) | 2.38× (2.26–2.49) | 1.98× (1.91–2.05) | 2.61× (2.50–2.72) | 2.60× (2.50–2.70) | 1.98× (1.93–2.02) | 2.03× (1.99–2.08) | 0.84× (0.82–0.86) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.44–1.55) | 2.93× (2.82–3.05) | 2.54× (2.44–2.63) | 2.67× (2.58–2.77) | 3.04× (2.92–3.17) | 3.61× (3.46–3.76) | 3.87× (3.72–4.02) | 1.55× (1.49–1.60) | 1.74× (1.68–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.34–1.40) | 2.72× (2.65–2.79) | 2.36× (2.26–2.46) | 3.13× (3.05–3.21) | 2.74× (2.68–2.80) | 3.20× (3.12–3.29) | 3.43× (3.37–3.50) | 1.37× (1.33–1.42) | 1.55× (1.51–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.58–1.66) | 2.60× (2.49–2.72) | 2.18× (1.89–2.52) | 2.80× (2.73–2.88) | 2.72× (2.61–2.83) | 2.33× (2.28–2.37) | 2.49× (2.45–2.54) | 1.00× (0.97–1.03) | 1.12× (1.10–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.46× (2.43–2.48) | 3.41× (3.38–3.45) | 2.83× (2.79–2.86) | 6.18× (5.85–6.54) | 2.10× (2.08–2.13) | 2.15× (2.11–2.18) | 0.99× (0.93–1.06) | 1.36× (1.34–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.61) | 2.26× (2.20–2.33) | 4.11× (3.97–4.25) | 2.75× (2.65–2.86) | 5.66× (5.44–5.88) | 1.98× (1.92–2.04) | 2.02× (1.96–2.09) | 0.94× (0.87–1.00) | 1.28× (1.25–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.58–0.59) | 2.54× (2.51–2.56) | 4.14× (4.09–4.20) | 2.86× (2.84–2.89) | 5.81× (5.72–5.89) | 2.06× (2.04–2.07) | 2.10× (2.08–2.12) | 0.97× (0.92–1.03) | 1.33× (1.32–1.35) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 1.77× (1.74–1.80) | 3.70× (3.62–3.78) | 4.05× (4.00–4.11) | 4.32× (4.26–4.38) | 4.48× (4.41–4.55) | 4.43× (4.36–4.50) | 4.39× (4.33–4.45) | 1.49× (1.45–1.52) | 1.57× (1.55–1.59) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 1.47× (1.45–1.49) | 3.04× (2.99–3.08) | 3.43× (3.36–3.49) | 3.89× (3.83–3.94) | 3.62× (3.58–3.66) | 3.49× (3.44–3.54) | 3.46× (3.43–3.49) | 1.17× (1.13–1.21) | 1.24× (1.22–1.25) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 1.64× (1.60–1.67) | 2.60× (2.46–2.74) | 2.94× (2.78–3.12) | 3.19× (3.08–3.30) | 3.15× (3.00–3.30) | 2.33× (2.30–2.36) | 2.31× (2.27–2.36) | 0.78× (0.75–0.81) | 0.83× (0.81–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.27× (0.26–0.28) | 0.81× (0.81–0.81) | 0.65× (0.64–0.65) | 9.12× (9.11–9.13) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 81 MB | 0.24× (0.22–0.25) | 0.32× (0.28–0.37) | 0.49× (0.46–0.52) | 0.41× (0.39–0.44) | 3.94× (3.71–4.19) | 0.09× (0.09–0.10) | 0.10× (0.10–0.11) | 0.15× (0.14–0.16) | 0.08× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 126 MB | 0.24× (0.23–0.25) | 0.33× (0.29–0.38) | 0.42× (0.40–0.44) | 0.37× (0.35–0.39) | 2.47× (2.35–2.60) | 0.06× (0.05–0.06) | 0.06× (0.06–0.07) | 0.09× (0.09–0.10) | 0.05× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.72) | 0.66× (0.65–0.67) | 8.06× (8.01–8.10) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.67× (0.64–0.70) | 1.59× (1.54–1.64) | 0.79× (0.76–0.82) | 5.76× (5.57–5.95) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.58–0.66) | 0.67× (0.61–0.74) | 1.12× (1.06–1.18) | 0.68× (0.65–0.72) | 3.26× (3.10–3.43) | 0.25× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.48–0.54) | 0.43× (0.41–0.45) | 0.93× (0.89–0.97) | 0.73× (0.70–0.78) | 3.22× (3.07–3.37) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.80× (0.77–0.82) | 2.48× (2.42–2.54) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.37× (0.34–0.40) | 0.86× (0.83–0.89) | 0.72× (0.68–0.75) | 2.96× (2.85–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.51) | 0.81× (0.79–0.82) | 0.82× (0.79–0.85) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.76× (0.76–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.76× (0.76–0.77) | 5.33× (5.31–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.41× (6.38–6.44) | 0.39× (0.39–0.39) | 0.43× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 39 MB | 0.43× (0.42–0.44) | 0.37× (0.35–0.40) | 0.99× (0.95–1.04) | 0.64× (0.61–0.67) | 3.13× (3.05–3.22) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 56 MB | 0.50× (0.48–0.52) | 0.46× (0.43–0.49) | 0.91× (0.86–0.95) | 0.71× (0.67–0.75) | 2.32× (2.23–2.41) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 485 | 2.07 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 785 | 2.38 | 1.14× (1.10–1.19) | 81% | 1.01× (0.98–1.05) | 1.05× (1.00–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 944 | 3.80 | 1.82× (1.76–1.89) | 49% | 1.27× (1.22–1.31) | 1.73× (1.68–1.78) |

**Best:** mibz2 at 4 threads (944 MB/s); lbzip2 at 4 threads (218 MB/s); pbz2 at 4 threads (338 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.15, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.63 | 0.02 | 0.07 | 0.222 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.09 | 12.7% |
| MTF + RLE2 | 1.55 | 18.0% |
| T-vector build | 1.47 | 17.0% |
| walk (the one each build uses) | 4.07 | 47.2% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.62 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.398 | 1.060 | (in its total) | 2.66× (2.66–2.67) |  |
| walk | 1.061 | 2.055 | (in its total) | 1.94× (1.93–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.042 | (in its total) | 1.11× (1.10–1.13) |  |
| inverse BWT, total | 1.497 | 3.157 | 1.880 | 2.11× (2.10–2.12) | 1.26× (1.25–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m31s |
| 4 threads | 32m27s ± 4m35s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m02s ± 3m19s |
| the scanner alone (its serial part) | 11m30s ± 19.7s |
| the workers alone, over 4 | 23m01s ± 3m19s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 3.02 ns a byte of output at 4 threads (CPU, summed over them), 1.71 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,417,543 | 12,200,394 | \`a6b24102e87d445d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
