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

The latest record: [`53f7ecf`](<https://github.com/lmmx/mibz2/commit/53f7ecf935d62ac7896eafaf55c5d249789614ca>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44977451821>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.508 s | 412 | 7.9 | 2.99× (2.96–3.01) | 6.91× (6.82–6.99) | 7.14× (7.01–7.27) | 7.74× (7.62–7.86) | 6.97× (6.88–7.06) | 7.60× (7.47–7.74) | 6.95× (6.86–7.04) | 2.23× (2.19–2.27) | 2.58× (2.53–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.288 s | 727 | 8.6 | 2.94× (2.79–3.10) | 6.37× (5.94–6.84) | 7.25× (6.84–7.68) | 7.15× (6.77–7.55) | 6.38× (6.07–6.70) | 13.07× (12.40–13.78) | 11.95× (11.36–12.57) | 3.83× (3.63–4.03) | 4.44× (4.22–4.67) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.240 s | 874 | 13.4 | 3.06× (2.85–3.28) | 4.61× (4.42–4.82) | 5.75× (5.55–5.97) | 5.25× (4.98–5.52) | 5.36× (5.01–5.75) | 15.98× (15.64–16.32) | 14.61× (14.33–14.90) | 4.68× (4.57–4.79) | 5.43× (5.29–5.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 966 | 3.7 | 0.59× (0.58–0.60) | 3.64× (3.47–3.82) | 4.45× (4.34–4.56) | 3.57× (3.50–3.65) | 6.21× (6.09–6.34) | 2.94× (2.88–2.99) | 2.63× (2.58–2.68) | 1.01× (0.99–1.02) | 1.49× (1.46–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 955 | 3.8 | 0.55× (0.54–0.57) | 2.92× (2.86–2.98) | 5.85× (5.73–5.98) | 3.48× (3.41–3.55) | 5.99× (5.89–6.08) | 2.89× (2.83–2.95) | 2.58× (2.53–2.64) | 0.99× (0.97–1.02) | 1.46× (1.43–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 924 | 3.8 | 0.55× (0.54–0.56) | 2.86× (2.80–2.91) | 5.85× (5.64–6.08) | 3.40× (3.33–3.47) | 5.94× (5.83–6.06) | 2.82× (2.76–2.87) | 2.52× (2.47–2.57) | 0.97× (0.94–0.99) | 1.43× (1.40–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 582 | 5.9 | 2.68× (2.60–2.76) | 3.02× (2.98–3.06) | 3.93× (3.79–4.07) | 3.81× (3.78–3.84) | 4.24× (4.21–4.27) | 2.54× (2.51–2.57) | 2.11× (2.09–2.13) | 2.03× (2.01–2.05) | 3.01× (2.97–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 877 | 7.3 | 2.26× (2.10–2.44) | 2.89× (2.70–3.09) | 4.15× (3.98–4.34) | 3.41× (3.17–3.67) | 4.75× (4.53–4.97) | 3.78× (3.60–3.96) | 3.14× (2.99–3.29) | 3.02× (2.88–3.17) | 4.47× (4.26–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 760 | 14.2 | 1.82× (1.75–1.89) | 1.94× (1.84–2.04) | 2.95× (2.80–3.10) | 2.18× (2.03–2.34) | 3.85× (3.69–4.01) | 3.28× (3.15–3.42) | 2.73× (2.62–2.84) | 2.62× (2.51–2.74) | 3.88× (3.72–4.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 53.3 | 1.35× (1.34–1.37) | 2.75× (2.72–2.77) | 2.50× (2.45–2.55) | 2.51× (2.49–2.53) | 2.94× (2.90–2.98) | 3.20× (3.17–3.23) | 3.17× (3.14–3.21) | 1.24× (1.23–1.25) | 1.43× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.085 s | 95 | 61.4 | 1.18× (1.11–1.26) | 2.37× (2.20–2.56) | 2.25× (2.11–2.40) | 2.88× (2.71–3.06) | 2.77× (2.60–2.95) | 4.83× (4.52–5.16) | 4.79× (4.50–5.10) | 1.86× (1.74–1.99) | 2.15× (2.01–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 82.7 | 1.28× (1.25–1.32) | 2.12× (2.06–2.19) | 2.06× (1.97–2.15) | 2.59× (2.50–2.69) | 2.87× (2.77–2.97) | 6.02× (5.86–6.19) | 5.97× (5.79–6.16) | 2.33× (2.26–2.39) | 2.68× (2.61–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.3 | 1.45× (1.44–1.46) | 3.10× (3.05–3.14) | 2.62× (2.58–2.66) | 2.67× (2.65–2.68) | 3.17× (3.03–3.32) | 3.67× (3.64–3.69) | 3.87× (3.84–3.90) | 1.45× (1.42–1.47) | 1.72× (1.69–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.8 | 1.32× (1.28–1.37) | 2.80× (2.63–2.97) | 2.44× (2.38–2.50) | 3.40× (3.35–3.44) | 3.05× (3.01–3.08) | 5.88× (5.78–5.98) | 6.21× (6.11–6.32) | 2.32× (2.27–2.38) | 2.75× (2.69–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 81.3 | 1.39× (1.29–1.49) | 2.23× (2.11–2.36) | 2.17× (2.02–2.33) | 2.94× (2.76–3.14) | 2.80× (2.63–2.98) | 6.68× (6.29–7.09) | 7.05× (6.63–7.50) | 2.64× (2.47–2.82) | 3.13× (2.93–3.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 5.1 | 0.66× (0.65–0.67) | 2.98× (2.92–3.03) | 4.35× (4.27–4.43) | 3.07× (3.02–3.12) | 6.91× (6.79–7.03) | 2.50× (2.45–2.54) | 2.29× (2.25–2.33) | 0.98× (0.97–0.99) | 1.50× (1.47–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 690 | 5.5 | 0.55× (0.54–0.56) | 2.82× (2.77–2.87) | 4.90× (4.81–5.00) | 2.94× (2.90–2.98) | 6.47× (6.36–6.59) | 2.36× (2.32–2.40) | 2.16× (2.12–2.20) | 0.93× (0.91–0.94) | 1.42× (1.39–1.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 5.4 | 0.56× (0.54–0.58) | 2.81× (2.74–2.88) | 4.93× (4.77–5.10) | 2.93× (2.85–3.01) | 6.49× (6.36–6.63) | 2.34× (2.28–2.40) | 2.14× (2.09–2.20) | 0.92× (0.90–0.94) | 1.41× (1.37–1.44) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.072 s | 186 | 17.9 | 1.97× (1.95–2.00) | 4.12× (4.05–4.19) | 4.50× (4.47–4.54) | 4.51× (4.49–4.54) | 4.79× (4.67–4.92) | 5.30× (5.24–5.36) | 4.79× (4.75–4.83) | 1.53× (1.50–1.56) | 1.86× (1.83–1.90) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.051 s | 264 | 22.9 | 1.60× (1.55–1.65) | 3.20× (3.06–3.35) | 4.03× (3.69–4.41) | 4.23× (3.92–4.56) | 4.14× (3.94–4.35) | 7.46× (7.07–7.87) | 6.74× (6.41–7.10) | 2.15× (2.04–2.28) | 2.62× (2.48–2.78) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.045 s | 297 | 35.1 | 1.50× (1.45–1.55) | 2.46× (2.37–2.55) | 3.27× (3.15–3.39) | 3.23× (3.09–3.39) | 3.72× (3.57–3.88) | 8.33× (7.95–8.72) | 7.53× (7.24–7.83) | 2.41× (2.31–2.51) | 2.93× (2.80–3.06) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.98× (2.96–3.00) | 6.87× (6.79–6.96) | 7.11× (6.98–7.24) | 7.71× (7.60–7.83) | 6.95× (6.86–7.03) | 7.58× (7.44–7.71) | 6.93× (6.84–7.02) | 2.22× (2.18–2.26) | 2.57× (2.51–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.86× (2.81–2.91) | 6.48× (6.41–6.56) | 6.80× (6.67–6.93) | 7.45× (7.33–7.58) | 6.38× (6.26–6.50) | 6.93× (6.80–7.05) | 6.33× (6.26–6.41) | 2.03× (2.00–2.07) | 2.35× (2.30–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.96× (2.90–3.01) | 5.00× (4.80–5.20) | 5.41× (5.25–5.57) | 5.62× (5.50–5.74) | 5.05× (4.84–5.26) | 4.47× (4.39–4.56) | 4.09× (4.03–4.15) | 1.31× (1.29–1.33) | 1.52× (1.49–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.23× (3.16–3.29) | 4.03× (3.98–4.08) | 3.34× (3.27–3.41) | 5.74× (5.62–5.86) | 2.76× (2.71–2.81) | 2.48× (2.43–2.53) | 1.00× (0.99–1.02) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 3.00× (2.94–3.05) | 5.27× (5.18–5.35) | 3.23× (3.18–3.29) | 5.47× (5.39–5.54) | 2.67× (2.63–2.72) | 2.40× (2.36–2.45) | 0.97× (0.95–1.00) | 1.02× (0.98–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.29× (3.22–3.35) | 5.30× (5.19–5.42) | 3.23× (3.17–3.29) | 5.52× (5.42–5.62) | 2.67× (2.61–2.72) | 2.39× (2.35–2.44) | 0.97× (0.95–1.00) | 1.01× (0.98–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.58× (2.51–2.65) | 2.79× (2.76–2.82) | 3.55× (3.43–3.66) | 3.65× (3.62–3.68) | 4.06× (4.02–4.09) | 2.45× (2.42–2.48) | 2.05× (2.02–2.07) | 1.97× (1.96–1.99) | 2.71× (2.67–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.21–2.29) | 2.49× (2.45–2.53) | 3.04× (2.98–3.10) | 3.10× (3.06–3.15) | 3.34× (3.28–3.40) | 1.97× (1.93–2.01) | 1.64× (1.61–1.68) | 1.58× (1.55–1.61) | 2.18× (2.13–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.99–2.16) | 1.75× (1.69–1.82) | 2.22× (2.12–2.32) | 2.05× (1.95–2.17) | 2.15× (2.06–2.25) | 1.03× (0.98–1.07) | 0.86× (0.82–0.89) | 0.83× (0.79–0.86) | 1.13× (1.09–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.68× (2.66–2.70) | 2.43× (2.38–2.47) | 2.49× (2.47–2.50) | 2.91× (2.87–2.95) | 3.17× (3.14–3.20) | 3.14× (3.10–3.18) | 1.23× (1.22–1.24) | 1.39× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.24–1.26) | 2.48× (2.43–2.54) | 2.18× (2.15–2.20) | 2.92× (2.89–2.96) | 2.58× (2.54–2.61) | 2.78× (2.73–2.82) | 2.75× (2.72–2.78) | 1.08× (1.06–1.10) | 1.21× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.42–1.49) | 2.34× (2.30–2.38) | 1.89× (1.83–1.94) | 2.60× (2.55–2.66) | 2.54× (2.46–2.62) | 2.05× (2.03–2.08) | 2.04× (1.99–2.08) | 0.80× (0.79–0.81) | 0.90× (0.88–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.45) | 3.04× (3.00–3.08) | 2.56× (2.52–2.59) | 2.64× (2.63–2.66) | 3.14× (3.00–3.29) | 3.63× (3.61–3.65) | 3.83× (3.80–3.86) | 1.44× (1.42–1.47) | 1.67× (1.66–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.31–1.36) | 2.76× (2.72–2.81) | 2.32× (2.25–2.40) | 3.21× (3.15–3.26) | 2.75× (2.70–2.80) | 3.15× (3.11–3.19) | 3.33× (3.28–3.38) | 1.25× (1.22–1.28) | 1.45× (1.43–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.51–1.57) | 2.48× (2.39–2.58) | 1.90× (1.83–1.96) | 2.74× (2.67–2.82) | 2.54× (2.47–2.62) | 2.19× (2.14–2.23) | 2.31× (2.26–2.36) | 0.87× (0.84–0.89) | 1.01× (0.99–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.66× (2.61–2.72) | 3.54× (3.46–3.62) | 2.81× (2.76–2.86) | 6.16× (6.05–6.26) | 2.30× (2.27–2.34) | 2.12× (2.08–2.16) | 0.98× (0.97–1.00) | 0.96× (0.94–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.63) | 2.47× (2.43–2.51) | 4.28× (4.22–4.34) | 2.72× (2.68–2.76) | 5.72× (5.62–5.82) | 2.15× (2.11–2.19) | 1.98× (1.94–2.02) | 0.92× (0.90–0.93) | 0.90× (0.88–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.62) | 2.72× (2.67–2.77) | 4.25× (4.18–4.33) | 2.77× (2.70–2.85) | 5.83× (5.73–5.93) | 2.17× (2.12–2.22) | 2.00× (1.95–2.04) | 0.92× (0.91–0.94) | 0.91× (0.88–0.93) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.95× (1.92–1.97) | 4.00× (3.94–4.06) | 4.34× (4.31–4.38) | 4.42× (4.39–4.45) | 4.69× (4.57–4.82) | 5.18× (5.13–5.24) | 4.69× (4.65–4.73) | 1.52× (1.49–1.54) | 1.80× (1.77–1.82) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.62× (1.58–1.66) | 3.29× (3.18–3.41) | 3.70× (3.45–3.97) | 3.98× (3.90–4.05) | 3.68× (3.61–3.75) | 4.06× (3.97–4.15) | 3.67× (3.60–3.75) | 1.19× (1.16–1.22) | 1.41× (1.38–1.44) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.68× (1.63–1.73) | 2.60× (2.50–2.71) | 2.90× (2.78–3.03) | 3.17× (3.05–3.29) | 2.96× (2.87–3.06) | 2.63× (2.57–2.70) | 2.38× (2.34–2.43) | 0.77× (0.75–0.79) | 0.91× (0.89–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.28–0.31) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.95× (10.94–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.34) | 0.55× (0.48–0.63) | 0.69× (0.67–0.71) | 0.58× (0.56–0.59) | 5.50× (5.36–5.65) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.32–0.34) | 0.62× (0.55–0.71) | 0.59× (0.57–0.61) | 0.51× (0.49–0.54) | 3.43× (3.30–3.57) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.83) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.94× (7.91–7.98) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.72× (0.67–0.78) | 1.59× (1.57–1.61) | 0.80× (0.78–0.82) | 5.75× (5.67–5.84) | 0.47× (0.46–0.47) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.67) | 0.71× (0.67–0.74) | 1.15× (1.11–1.20) | 0.70× (0.68–0.73) | 3.40× (3.30–3.51) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.37–0.45) | 0.91× (0.87–0.95) | 0.73× (0.71–0.76) | 3.15× (3.03–3.28) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.45× (0.44–0.46) | 0.87× (0.85–0.88) | 0.80× (0.77–0.83) | 2.51× (2.47–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.15× (5.14–5.16) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.37× (0.34–0.40) | 0.84× (0.81–0.87) | 0.71× (0.68–0.75) | 2.88× (2.79–2.97) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.48× (0.45–0.52) | 0.80× (0.78–0.82) | 0.80× (0.77–0.83) | 2.30× (2.26–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.33× (5.32–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.76× (0.76–0.76) | 5.24× (5.23–5.26) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.30) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.67× (0.66–0.67) | 6.41× (6.39–6.43) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.42) | 0.39× (0.35–0.44) | 0.96× (0.94–0.98) | 0.62× (0.61–0.63) | 3.10× (3.06–3.13) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 62 MB | 0.45× (0.44–0.46) | 0.41× (0.37–0.45) | 0.81× (0.80–0.83) | 0.66× (0.63–0.69) | 2.08× (2.05–2.12) | 0.12× (0.12–0.12) | 0.13× (0.13–0.13) | 0.20× (0.19–0.20) | 0.10× (0.09–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 412 | 2.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 727 | 2.66 | 1.09× (1.09–1.10) | 86% | 1.03× (1.02–1.04) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 874 | 4.14 | 1.69× (1.67–1.72) | 53% | 1.23× (1.19–1.27) | 1.68× (1.66–1.71) |

**Best:** mibz2 at 4 threads (874 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (293 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.67 | 0.02 | 0.08 | 0.240 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.5% |
| MTF + RLE2 | 1.52 | 17.0% |
| T-vector build | 1.51 | 17.0% |
| walk (the one each build uses) | 4.24 | 47.4% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.93 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.300 | (in its total) | 2.64× (2.62–2.66) |  |
| walk | 1.300 | 2.679 | (in its total) | 2.08× (2.05–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.051 | 0.057 | (in its total) | 1.13× (1.10–1.15) |  |
| inverse BWT, total | 1.848 | 4.036 | 3.303 | 2.20× (2.17–2.23) | 1.80× (1.79–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 8m31s |
| 4 threads | 35m16s ± 4m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m24s ± 3m28s |
| the scanner alone (its serial part) | 13m23s ± 26.3s |
| the workers alone, over 4 | 24m24s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.20 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
