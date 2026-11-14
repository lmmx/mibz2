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

The latest record: [`befd9ba`](<https://github.com/lmmx/mibz2/commit/befd9badf1700ca0b11a75d4a5510223223aab8f>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43598245341>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.524 s | 400 | 6.1 | 2.91× (2.87–2.95) | 6.65× (6.56–6.74) | 6.78× (6.69–6.87) | 7.39× (7.31–7.47) | 6.74× (6.67–6.82) | 7.26× (7.18–7.33) | 6.76× (6.68–6.83) | 2.14× (2.12–2.16) | 2.51× (2.48–2.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.312 s | 672 | 6.8 | 2.67× (2.50–2.85) | 5.69× (5.33–6.07) | 6.45× (6.01–6.93) | 6.37× (5.98–6.79) | 5.85× (5.53–6.18) | 11.72× (10.93–12.57) | 10.92× (10.15–11.74) | 3.46× (3.22–3.72) | 4.05× (3.76–4.37) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.266 s | 790 | 11.4 | 2.67× (2.64–2.70) | 4.04× (3.93–4.16) | 5.17× (4.96–5.40) | 4.56× (4.49–4.64) | 4.67× (4.61–4.73) | 14.11× (13.79–14.44) | 13.14× (12.81–13.47) | 4.17× (4.08–4.26) | 4.88× (4.76–5.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 2.8 | 0.60× (0.59–0.61) | 3.85× (3.66–4.04) | 4.57× (4.43–4.72) | 3.58× (3.53–3.63) | 6.19× (6.03–6.35) | 2.99× (2.94–3.04) | 2.67× (2.63–2.71) | 1.03× (1.00–1.06) | 1.51× (1.48–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 957 | 2.9 | 0.57× (0.57–0.58) | 2.95× (2.90–3.00) | 6.00× (5.88–6.13) | 3.55× (3.46–3.65) | 6.00× (5.94–6.07) | 2.93× (2.88–2.97) | 2.61× (2.57–2.66) | 1.01× (0.97–1.04) | 1.48× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 941 | 2.9 | 0.57× (0.56–0.58) | 2.94× (2.87–3.01) | 5.91× (5.75–6.07) | 3.49× (3.39–3.59) | 5.96× (5.87–6.04) | 2.90× (2.83–2.96) | 2.59× (2.53–2.64) | 1.00× (0.96–1.03) | 1.46× (1.43–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 587 | 4.4 | 2.63× (2.59–2.68) | 3.01× (2.95–3.07) | 3.92× (3.80–4.03) | 3.79× (3.71–3.87) | 4.12× (4.06–4.19) | 2.54× (2.49–2.59) | 2.10× (2.07–2.14) | 2.01× (1.98–2.05) | 2.99× (2.93–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 839 | 5.6 | 2.13× (1.83–2.48) | 2.71× (2.25–3.25) | 3.74× (3.14–4.46) | 3.28× (2.77–3.89) | 4.40× (3.75–5.16) | 3.54× (3.01–4.16) | 2.93× (2.50–3.44) | 2.81× (2.40–3.28) | 4.17× (3.55–4.90) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 778 | 10.9 | 1.86× (1.81–1.91) | 1.88× (1.74–2.02) | 2.86× (2.78–2.95) | 2.24× (2.13–2.35) | 3.89× (3.77–4.00) | 3.40× (3.28–3.51) | 2.82× (2.72–2.91) | 2.70× (2.61–2.78) | 4.00× (3.87–4.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 41.6 | 1.33× (1.31–1.34) | 2.65× (2.60–2.69) | 2.39× (2.34–2.43) | 2.39× (2.37–2.42) | 2.80× (2.75–2.84) | 3.07× (3.04–3.10) | 3.11× (3.09–3.14) | 1.21× (1.18–1.24) | 1.38× (1.36–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.084 s | 96 | 47.4 | 1.24× (1.20–1.28) | 2.43× (2.37–2.50) | 2.29× (2.24–2.35) | 2.85× (2.79–2.92) | 2.85× (2.77–2.94) | 4.94× (4.81–5.08) | 5.00× (4.87–5.14) | 1.95× (1.88–2.01) | 2.21× (2.15–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 63.6 | 1.27× (1.24–1.30) | 2.02× (1.93–2.11) | 1.99× (1.92–2.06) | 2.37× (2.27–2.47) | 2.68× (2.62–2.74) | 5.76× (5.64–5.87) | 5.83× (5.72–5.94) | 2.27× (2.20–2.34) | 2.58× (2.51–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.149 s | 65 | 38.4 | 1.46× (1.44–1.47) | 2.96× (2.93–2.99) | 2.47× (2.44–2.50) | 2.55× (2.52–2.59) | 3.01× (2.96–3.06) | 3.53× (3.49–3.58) | 3.85× (3.80–3.90) | 1.37× (1.34–1.41) | 1.65× (1.63–1.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 107 | 43.2 | 1.30× (1.23–1.38) | 2.75× (2.45–3.08) | 2.36× (2.22–2.50) | 3.14× (2.91–3.37) | 2.97× (2.77–3.19) | 5.66× (5.32–6.02) | 6.18× (5.82–6.55) | 2.20× (2.05–2.36) | 2.65× (2.49–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 124 | 61.4 | 1.42× (1.39–1.46) | 2.25× (2.16–2.34) | 2.21× (2.10–2.33) | 2.81× (2.74–2.88) | 2.82× (2.69–2.95) | 6.75× (6.63–6.87) | 7.36× (7.22–7.50) | 2.62× (2.54–2.71) | 3.16× (3.08–3.24) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 3.8 | 0.67× (0.66–0.69) | 3.18× (3.01–3.35) | 4.45× (4.35–4.56) | 3.13× (3.06–3.20) | 6.76× (6.65–6.88) | 2.55× (2.50–2.60) | 2.35× (2.30–2.39) | 0.98× (0.96–1.00) | 1.53× (1.50–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 4.1 | 0.55× (0.49–0.61) | 2.74× (2.50–3.01) | 4.83× (4.38–5.32) | 2.90× (2.74–3.06) | 6.10× (5.54–6.71) | 2.30× (2.09–2.53) | 2.11× (1.92–2.32) | 0.88× (0.81–0.96) | 1.38× (1.26–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 697 | 4.0 | 0.56× (0.54–0.58) | 2.82× (2.72–2.92) | 4.89× (4.67–5.12) | 2.93× (2.84–3.02) | 6.25× (6.06–6.45) | 2.36× (2.28–2.43) | 2.17× (2.09–2.24) | 0.91× (0.87–0.94) | 1.41× (1.37–1.46) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.072 s | 182 | 13.8 | 2.01× (1.93–2.08) | 4.06× (4.01–4.10) | 4.42× (4.33–4.50) | 4.43× (4.36–4.49) | 4.68× (4.61–4.75) | 5.20× (5.14–5.27) | 4.87× (4.81–4.93) | 1.49× (1.47–1.52) | 1.87× (1.82–1.91) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.052 s | 252 | 17.2 | 1.53× (1.43–1.64) | 3.26× (2.92–3.65) | 3.73× (3.49–3.99) | 3.92× (3.62–4.25) | 4.09× (3.97–4.20) | 7.10× (6.69–7.53) | 6.64× (6.26–7.04) | 2.03× (1.91–2.16) | 2.54× (2.39–2.71) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.047 s | 280 | 26.8 | 1.48× (1.45–1.51) | 2.34× (2.28–2.40) | 3.06× (2.96–3.17) | 3.04× (2.97–3.12) | 3.56× (3.52–3.61) | 7.90× (7.75–8.05) | 7.39× (7.23–7.55) | 2.26× (2.22–2.31) | 2.83× (2.77–2.90) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.86–2.94) | 6.62× (6.53–6.71) | 6.75× (6.67–6.84) | 7.36× (7.29–7.44) | 6.72× (6.64–6.80) | 7.23× (7.16–7.30) | 6.73× (6.66–6.81) | 2.14× (2.12–2.16) | 2.49× (2.46–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.69× (2.65–2.74) | 6.02× (5.87–6.17) | 6.31× (6.19–6.43) | 6.87× (6.75–6.99) | 6.03× (5.92–6.14) | 6.41× (6.27–6.56) | 5.97× (5.81–6.14) | 1.90× (1.85–1.94) | 2.21× (2.15–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.56× (2.49–2.63) | 4.28× (4.17–4.41) | 4.63× (4.53–4.74) | 4.82× (4.71–4.94) | 4.38× (4.31–4.46) | 3.84× (3.73–3.96) | 3.58× (3.46–3.71) | 1.14× (1.10–1.17) | 1.32× (1.28–1.37) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.29× (3.22–3.35) | 4.11× (4.05–4.17) | 3.34× (3.28–3.40) | 5.71× (5.56–5.86) | 2.81× (2.76–2.86) | 2.52× (2.47–2.56) | 1.02× (0.99–1.05) | 1.09× (1.05–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.04× (2.99–3.09) | 5.33× (5.25–5.41) | 3.30× (3.21–3.39) | 5.48× (5.41–5.56) | 2.71× (2.66–2.76) | 2.43× (2.38–2.48) | 0.99× (0.96–1.02) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.59–0.62) | 3.37× (3.31–3.43) | 5.36× (5.25–5.47) | 3.29× (3.21–3.37) | 5.50× (5.43–5.57) | 2.73× (2.67–2.78) | 2.44× (2.39–2.49) | 0.99× (0.96–1.02) | 1.05× (1.01–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.55× (2.50–2.59) | 2.78× (2.73–2.84) | 3.55× (3.44–3.66) | 3.64× (3.56–3.72) | 3.95× (3.89–4.01) | 2.45× (2.41–2.50) | 2.04× (2.01–2.08) | 1.96× (1.93–1.99) | 2.71× (2.63–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.18–2.36) | 2.45× (2.35–2.56) | 3.02× (2.94–3.10) | 3.21× (3.11–3.32) | 3.31× (3.19–3.42) | 1.97× (1.90–2.04) | 1.64× (1.58–1.69) | 1.57× (1.52–1.62) | 2.17× (2.08–2.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.91–2.07) | 1.68× (1.60–1.77) | 2.15× (2.05–2.26) | 2.05× (1.95–2.15) | 2.06× (1.97–2.16) | 1.00× (0.95–1.05) | 0.83× (0.79–0.88) | 0.80× (0.76–0.84) | 1.10× (1.04–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.31–1.34) | 2.58× (2.55–2.62) | 2.33× (2.30–2.37) | 2.37× (2.35–2.40) | 2.77× (2.73–2.82) | 3.04× (3.01–3.07) | 3.08× (3.05–3.10) | 1.21× (1.18–1.24) | 1.34× (1.32–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.22–1.26) | 2.39× (2.34–2.44) | 2.12× (2.09–2.15) | 2.71× (2.66–2.76) | 2.51× (2.47–2.56) | 2.69× (2.64–2.75) | 2.73× (2.67–2.78) | 1.07× (1.04–1.09) | 1.19× (1.16–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.44–1.49) | 2.26× (2.18–2.35) | 1.83× (1.79–1.87) | 2.40× (2.35–2.44) | 2.34× (2.31–2.37) | 1.98× (1.96–2.01) | 2.01× (1.98–2.03) | 0.79× (0.76–0.81) | 0.87× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.46) | 2.91× (2.88–2.94) | 2.42× (2.39–2.44) | 2.53× (2.50–2.56) | 2.98× (2.93–3.03) | 3.50× (3.46–3.54) | 3.82× (3.77–3.86) | 1.37× (1.34–1.40) | 1.62× (1.60–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.29–1.37) | 2.67× (2.59–2.76) | 2.22× (2.16–2.29) | 2.99× (2.85–3.13) | 2.66× (2.59–2.73) | 3.05× (2.95–3.16) | 3.33× (3.22–3.45) | 1.20× (1.14–1.25) | 1.41× (1.36–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.53–1.65) | 2.51× (2.43–2.58) | 1.95× (1.81–2.10) | 2.64× (2.59–2.69) | 2.47× (2.43–2.52) | 2.20× (2.17–2.24) | 2.40× (2.37–2.44) | 0.86× (0.84–0.88) | 1.02× (1.00–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.73) | 2.73× (2.70–2.77) | 3.59× (3.51–3.66) | 2.86× (2.80–2.92) | 6.02× (5.92–6.13) | 2.36× (2.31–2.41) | 2.17× (2.13–2.21) | 0.98× (0.96–1.01) | 1.02× (0.99–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.55–0.69) | 2.41× (2.20–2.64) | 4.07× (3.72–4.46) | 2.67× (2.52–2.83) | 5.38× (4.88–5.92) | 2.09× (1.90–2.30) | 1.92× (1.75–2.11) | 0.87× (0.80–0.96) | 0.91× (0.81–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.59–0.63) | 2.72× (2.64–2.79) | 4.19× (4.05–4.34) | 2.77× (2.69–2.85) | 5.63× (5.47–5.79) | 2.19× (2.13–2.26) | 2.02× (1.96–2.08) | 0.91× (0.88–0.94) | 0.95× (0.91–1.00) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.98× (1.90–2.06) | 3.92× (3.88–3.96) | 4.25× (4.18–4.33) | 4.33× (4.27–4.39) | 4.58× (4.51–4.65) | 5.09× (5.02–5.15) | 4.76× (4.70–4.82) | 1.48× (1.45–1.50) | 1.79× (1.76–1.81) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.66× (1.64–1.68) | 3.27× (3.21–3.34) | 3.56× (3.48–3.63) | 3.92× (3.84–4.00) | 3.69× (3.61–3.77) | 4.05× (3.96–4.13) | 3.78× (3.70–3.87) | 1.18× (1.15–1.20) | 1.42× (1.39–1.46) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.75× (1.69–1.81) | 2.58× (2.52–2.64) | 2.87× (2.77–2.99) | 3.10× (3.02–3.19) | 3.00× (2.94–3.07) | 2.62× (2.58–2.67) | 2.45× (2.40–2.50) | 0.76× (0.75–0.77) | 0.92× (0.90–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.30–0.33) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.93× (11.90–11.96) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.56× (0.49–0.63) | 0.77× (0.76–0.78) | 0.63× (0.62–0.64) | 6.02× (5.97–6.07) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.38) | 0.65× (0.58–0.73) | 0.68× (0.67–0.69) | 0.59× (0.57–0.61) | 3.93× (3.90–3.96) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.02× (8.01–8.04) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.73× (0.69–0.77) | 1.61× (1.57–1.65) | 0.81× (0.79–0.82) | 5.83× (5.70–5.96) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.66) | 0.73× (0.67–0.79) | 1.17× (1.13–1.21) | 0.71× (0.69–0.74) | 3.42× (3.33–3.52) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.52× (0.50–0.54) | 0.41× (0.37–0.45) | 0.93× (0.90–0.97) | 0.75× (0.73–0.78) | 3.23× (3.12–3.34) | 0.20× (0.19–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.85× (0.84–0.87) | 0.78× (0.75–0.81) | 2.49× (2.44–2.53) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.49) | 0.38× (0.35–0.41) | 0.86× (0.83–0.88) | 0.69× (0.67–0.72) | 2.93× (2.83–3.03) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.50) | 0.80× (0.78–0.82) | 0.81× (0.78–0.83) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.36× (5.34–5.38) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.77) | 5.30× (5.27–5.32) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.90× (0.89–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.31× (5.30–5.33) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.65–0.66) | 6.39× (6.38–6.41) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 35 MB | 0.48× (0.47–0.49) | 0.39× (0.36–0.43) | 1.14× (1.11–1.17) | 0.74× (0.71–0.76) | 3.56× (3.50–3.62) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 54 MB | 0.52× (0.51–0.53) | 0.48× (0.44–0.52) | 0.94× (0.92–0.96) | 0.76× (0.74–0.78) | 2.40× (2.36–2.43) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 672 | 2.79 | 1.13× (1.10–1.16) | 81% | 1.03× (1.01–1.04) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 790 | 4.65 | 1.88× (1.81–1.95) | 49% | 1.22× (1.18–1.26) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (790 MB/s); lbzip2 at 4 threads (197 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.77 | 0.02 | 0.08 | 0.266 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.7% |
| MTF + RLE2 | 1.20 | 17.8% |
| T-vector build | 1.14 | 17.0% |
| walk (the one each build uses) | 3.11 | 46.4% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.9% |
| **total** | 6.70 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.488 | 1.306 | (in its total) | 2.67× (2.65–2.69) |  |
| walk | 1.230 | 2.634 | (in its total) | 2.14× (2.12–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.054 | (in its total) | 1.19× (1.18–1.21) |  |
| inverse BWT, total | 1.764 | 3.994 | 3.263 | 2.26× (2.24–2.28) | 1.84× (1.83–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m38s |
| 4 threads | 39m17s ± 5m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m29s ± 4m14s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 28m29s ± 4m14s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 3.73 ns a byte of output at 4 threads (CPU, summed over them), 2.01 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
