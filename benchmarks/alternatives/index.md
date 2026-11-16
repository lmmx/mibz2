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

The latest record: [`8b0b0a5`](<https://github.com/lmmx/mibz2/commit/8b0b0a5f3c5ab4025cd3a88725f8c5643fd4752c>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43968899931>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.419 s | 500 | 7.4 | 2.98× (2.86–3.11) | 6.74× (6.51–6.98) | 7.01× (6.74–7.29) | 7.98× (7.64–8.33) | 7.07× (6.75–7.41) | 7.27× (7.00–7.56) | 7.04× (6.74–7.35) | 2.19× (2.10–2.29) | 2.44× (2.33–2.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.240 s | 874 | 8.2 | 3.07× (2.96–3.18) | 6.33× (6.00–6.67) | 7.29× (7.01–7.59) | 7.68× (7.45–7.92) | 6.66× (6.48–6.84) | 12.90× (12.70–13.09) | 12.48× (12.20–12.76) | 3.89× (3.80–3.98) | 4.33× (4.21–4.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.203 s | 1,034 | 12.6 | 3.07× (3.00–3.15) | 4.63× (4.45–4.82) | 6.08× (5.70–6.50) | 5.65× (5.40–5.91) | 5.55× (5.29–5.82) | 15.31× (15.11–15.52) | 14.82× (14.56–15.08) | 4.62× (4.54–4.70) | 5.15× (5.03–5.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,114 | 3.6 | 0.57× (0.57–0.58) | 3.43× (3.39–3.46) | 4.63× (4.50–4.76) | 3.59× (3.55–3.62) | 6.19× (6.09–6.29) | 2.71× (2.61–2.82) | 2.66× (2.63–2.69) | 1.00× (0.98–1.02) | 1.87× (1.69–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,099 | 3.8 | 0.52× (0.51–0.53) | 2.80× (2.75–2.85) | 5.59× (5.49–5.70) | 3.53× (3.46–3.59) | 5.94× (5.83–6.06) | 2.64× (2.52–2.76) | 2.59× (2.55–2.64) | 0.98× (0.96–0.99) | 1.82× (1.66–2.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,064 | 3.8 | 0.51× (0.48–0.54) | 2.70× (2.55–2.86) | 5.38× (5.08–5.70) | 3.37× (3.18–3.57) | 5.74× (5.41–6.10) | 2.54× (2.48–2.60) | 2.49× (2.35–2.64) | 0.94× (0.88–0.99) | 1.75× (1.55–1.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 730 | 5.4 | 2.73× (2.60–2.86) | 3.03× (2.99–3.07) | 3.79× (3.74–3.84) | 4.06× (4.01–4.11) | 4.45× (4.37–4.52) | 2.75× (2.71–2.78) | 2.25× (2.22–2.28) | 2.08× (2.06–2.09) | 3.01× (2.98–3.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,074 | 6.7 | 2.21× (2.13–2.29) | 2.82× (2.67–2.99) | 4.40× (4.22–4.60) | 3.53× (3.35–3.71) | 4.99× (4.74–5.25) | 3.99× (3.85–4.14) | 3.27× (3.16–3.39) | 3.02× (2.91–3.14) | 4.38× (4.22–4.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 942 | 13.4 | 1.91× (1.86–1.96) | 2.05× (1.99–2.12) | 3.05× (2.88–3.23) | 2.18× (2.10–2.27) | 4.05× (3.85–4.25) | 3.60× (3.49–3.72) | 2.95× (2.86–3.05) | 2.73× (2.64–2.82) | 3.95× (3.83–4.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 52.2 | 1.39× (1.37–1.42) | 2.65× (2.62–2.68) | 2.41× (2.38–2.44) | 2.51× (2.48–2.53) | 2.77× (2.75–2.79) | 3.01× (2.97–3.05) | 3.14× (3.10–3.17) | 1.25× (1.24–1.26) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 113 | 58.2 | 1.25× (1.20–1.31) | 2.44× (2.26–2.62) | 2.37× (2.27–2.47) | 3.00× (2.86–3.15) | 2.78× (2.66–2.90) | 4.74× (4.53–4.97) | 4.94× (4.71–5.18) | 1.97× (1.88–2.07) | 2.24× (2.12–2.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 142 | 77.2 | 1.31× (1.26–1.35) | 2.16× (2.08–2.23) | 2.14× (2.09–2.20) | 2.69× (2.59–2.78) | 2.90× (2.80–2.99) | 5.91× (5.71–6.11) | 6.15× (5.98–6.33) | 2.45× (2.38–2.52) | 2.79× (2.73–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 47.0 | 1.53× (1.52–1.54) | 3.04× (3.02–3.07) | 2.65× (2.61–2.69) | 2.78× (2.74–2.82) | 3.08× (3.06–3.11) | 3.68× (3.66–3.71) | 4.04× (4.01–4.07) | 1.54× (1.51–1.56) | 1.83× (1.80–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 127 | 53.9 | 1.36× (1.31–1.40) | 2.72× (2.62–2.84) | 2.49× (2.41–2.57) | 3.49× (3.37–3.63) | 3.01× (2.91–3.11) | 5.88× (5.70–6.06) | 6.45× (6.24–6.67) | 2.45× (2.37–2.54) | 2.92× (2.85–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.065 s | 147 | 78.4 | 1.41× (1.37–1.44) | 2.23× (2.16–2.30) | 2.26× (2.12–2.41) | 3.04× (2.99–3.09) | 2.87× (2.71–3.05) | 6.82× (6.72–6.92) | 7.48× (7.37–7.59) | 2.84× (2.78–2.91) | 3.38× (3.30–3.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 837 | 5.0 | 0.62× (0.61–0.63) | 3.43× (3.37–3.49) | 4.30× (4.23–4.38) | 3.08× (3.03–3.13) | 6.61× (6.51–6.71) | 2.26× (2.23–2.30) | 2.31× (2.27–2.35) | 0.96× (0.94–0.97) | 1.72× (1.69–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 800 | 5.4 | 0.52× (0.51–0.53) | 2.47× (2.42–2.51) | 4.95× (4.86–5.03) | 2.96× (2.90–3.01) | 6.32× (6.22–6.42) | 2.16× (2.12–2.20) | 2.21× (2.17–2.25) | 0.91× (0.90–0.93) | 1.64× (1.62–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 790 | 5.3 | 0.52× (0.50–0.53) | 2.44× (2.39–2.50) | 4.88× (4.76–4.99) | 2.94× (2.87–3.01) | 6.24× (6.08–6.40) | 2.13× (2.09–2.18) | 2.18× (2.13–2.23) | 0.90× (0.88–0.93) | 1.62× (1.58–1.66) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 0.054 s | 225 | 17.0 | 1.83× (1.81–1.84) | 3.99× (3.87–4.12) | 4.31× (4.27–4.36) | 4.54× (4.51–4.57) | 4.73× (4.70–4.75) | 4.67× (4.63–4.71) | 4.77× (4.74–4.81) | 1.52× (1.48–1.55) | 1.69× (1.66–1.73) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 0.039 s | 316 | 21.5 | 1.43× (1.38–1.48) | 3.06× (2.90–3.23) | 3.60× (3.35–3.88) | 4.43× (4.14–4.74) | 4.09× (3.97–4.22) | 6.46× (6.24–6.69) | 6.60× (6.38–6.83) | 2.10× (2.01–2.19) | 2.34× (2.25–2.44) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 0.035 s | 354 | 32.5 | 1.43× (1.37–1.48) | 2.44× (2.29–2.60) | 3.01× (2.81–3.22) | 3.50× (3.41–3.58) | 3.80× (3.64–3.96) | 7.34× (7.20–7.50) | 7.51× (7.35–7.66) | 2.38× (2.35–2.42) | 2.66× (2.62–2.71) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.97× (2.85–3.10) | 6.71× (6.48–6.94) | 6.97× (6.70–7.25) | 7.95× (7.62–8.30) | 7.04× (6.73–7.38) | 7.25× (6.97–7.53) | 7.01× (6.72–7.32) | 2.19× (2.09–2.29) | 2.43× (2.31–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.85× (2.76–2.94) | 6.41× (6.18–6.64) | 6.73× (6.53–6.95) | 7.78× (7.60–7.96) | 6.51× (6.37–6.65) | 6.69× (6.63–6.74) | 6.47× (6.36–6.58) | 2.02× (1.99–2.05) | 2.24× (2.19–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.01× (2.93–3.09) | 5.06× (4.91–5.22) | 5.81× (5.51–6.14) | 6.18× (5.93–6.43) | 5.35× (5.09–5.63) | 4.29× (4.26–4.33) | 4.16× (4.09–4.22) | 1.30× (1.28–1.31) | 1.44× (1.41–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.93× (2.88–2.98) | 3.99× (3.83–4.15) | 3.35× (3.31–3.39) | 5.72× (5.62–5.81) | 2.57× (2.45–2.70) | 2.51× (2.48–2.54) | 1.00× (0.98–1.02) | 1.65× (1.62–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.70× (2.65–2.74) | 4.94× (4.87–5.01) | 3.27× (3.22–3.33) | 5.43× (5.33–5.53) | 2.47× (2.34–2.61) | 2.41× (2.37–2.45) | 0.96× (0.95–0.98) | 1.58× (1.56–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.51–0.58) | 2.88× (2.68–3.09) | 4.82× (4.54–5.11) | 3.19× (2.98–3.41) | 5.32× (4.97–5.70) | 2.42× (2.36–2.48) | 2.36× (2.21–2.53) | 0.94× (0.88–1.01) | 1.55× (1.45–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.51–2.75) | 2.82× (2.80–2.84) | 3.40× (3.37–3.44) | 3.88× (3.84–3.92) | 4.25× (4.18–4.31) | 2.65× (2.62–2.68) | 2.17× (2.15–2.19) | 2.02× (2.01–2.03) | 2.63× (2.61–2.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.22–2.30) | 2.48× (2.42–2.54) | 3.34× (3.27–3.40) | 3.29× (3.23–3.35) | 3.54× (3.47–3.62) | 2.12× (2.08–2.17) | 1.74× (1.71–1.78) | 1.62× (1.60–1.64) | 2.11× (2.07–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.95–2.05) | 1.66× (1.61–1.72) | 2.19× (2.13–2.25) | 2.01× (1.94–2.10) | 2.07× (2.01–2.14) | 1.07× (1.04–1.10) | 0.88× (0.85–0.90) | 0.82× (0.80–0.84) | 1.07× (1.04–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.36–1.41) | 2.57× (2.55–2.59) | 2.34× (2.33–2.36) | 2.48× (2.46–2.51) | 2.74× (2.72–2.77) | 2.96× (2.94–2.98) | 3.10× (3.07–3.14) | 1.25× (1.23–1.26) | 1.37× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.42× (2.37–2.48) | 2.19× (2.11–2.28) | 2.91× (2.86–2.97) | 2.50× (2.47–2.54) | 2.63× (2.59–2.68) | 2.76× (2.70–2.82) | 1.11× (1.08–1.13) | 1.22× (1.19–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.41–1.50) | 2.33× (2.28–2.38) | 1.99× (1.95–2.03) | 2.67× (2.61–2.74) | 2.53× (2.46–2.59) | 1.99× (1.96–2.03) | 2.09× (2.05–2.13) | 0.84× (0.82–0.86) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.53) | 2.98× (2.96–3.01) | 2.59× (2.55–2.62) | 2.75× (2.72–2.79) | 3.05× (3.03–3.08) | 3.65× (3.62–3.67) | 4.00× (3.97–4.03) | 1.53× (1.51–1.55) | 1.77× (1.74–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.41) | 2.73× (2.66–2.79) | 2.33× (2.28–2.39) | 3.31× (3.21–3.42) | 2.73× (2.67–2.79) | 3.16× (3.10–3.22) | 3.46× (3.39–3.53) | 1.32× (1.30–1.35) | 1.53× (1.51–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.48–1.61) | 2.49× (2.42–2.57) | 2.04× (1.88–2.22) | 2.81× (2.73–2.89) | 2.62× (2.42–2.83) | 2.22× (2.17–2.29) | 2.44× (2.37–2.51) | 0.93× (0.90–0.96) | 1.08× (1.05–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.46× (2.40–2.52) | 3.43× (3.38–3.48) | 2.82× (2.78–2.86) | 5.91× (5.83–6.00) | 2.11× (2.07–2.14) | 2.15× (2.12–2.19) | 0.96× (0.95–0.98) | 1.35× (1.33–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.24× (2.20–2.28) | 4.04× (3.94–4.14) | 2.72× (2.68–2.76) | 5.57× (5.49–5.64) | 1.97× (1.94–2.00) | 2.02× (1.98–2.05) | 0.90× (0.89–0.91) | 1.27× (1.24–1.29) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.58) | 2.49× (2.44–2.53) | 4.06× (3.98–4.15) | 2.79× (2.73–2.85) | 5.67× (5.54–5.81) | 2.01× (1.97–2.05) | 2.05× (2.01–2.10) | 0.92× (0.90–0.94) | 1.29× (1.26–1.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 1.80× (1.78–1.82) | 3.81× (3.71–3.91) | 4.15× (4.12–4.18) | 4.43× (4.40–4.46) | 4.61× (4.58–4.63) | 4.55× (4.52–4.59) | 4.65× (4.62–4.69) | 1.50× (1.47–1.53) | 1.61× (1.60–1.63) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 1.51× (1.48–1.54) | 3.12× (3.06–3.18) | 3.51× (3.41–3.62) | 4.05× (3.97–4.13) | 3.74× (3.68–3.81) | 3.57× (3.49–3.65) | 3.65× (3.57–3.73) | 1.18× (1.15–1.21) | 1.27× (1.24–1.29) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 1.62× (1.53–1.71) | 2.71× (2.53–2.90) | 3.08× (2.90–3.28) | 3.33× (3.23–3.43) | 3.32× (3.12–3.53) | 2.40× (2.35–2.44) | 2.45× (2.40–2.50) | 0.79× (0.77–0.81) | 0.85× (0.83–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.08× (11.06–11.10) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.41× (0.35–0.47) | 0.66× (0.63–0.68) | 0.55× (0.54–0.57) | 5.25× (5.07–5.44) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.35) | 0.49× (0.43–0.56) | 0.61× (0.59–0.63) | 0.52× (0.50–0.54) | 3.47× (3.35–3.60) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.39–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.69–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.05) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.64× (0.60–0.69) | 1.61× (1.57–1.64) | 0.81× (0.79–0.82) | 5.83× (5.72–5.94) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.63–0.68) | 0.68× (0.64–0.73) | 1.18× (1.14–1.21) | 0.72× (0.70–0.75) | 3.43× (3.35–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.73) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.41× (0.38–0.44) | 0.91× (0.89–0.94) | 0.73× (0.72–0.75) | 3.16× (3.09–3.24) | 0.20× (0.19–0.20) | 0.21× (0.21–0.21) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.83× (0.81–0.84) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.24× (5.22–5.25) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.38) | 0.84× (0.82–0.87) | 0.71× (0.68–0.74) | 2.91× (2.81–3.00) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.44–0.49) | 0.80× (0.78–0.82) | 0.82× (0.80–0.85) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.31–5.34) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.42) | 0.66× (0.66–0.66) | 6.42× (6.40–6.44) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 36 MB | 0.45× (0.44–0.47) | 0.40× (0.38–0.42) | 1.05× (1.02–1.09) | 0.68× (0.67–0.70) | 3.34× (3.26–3.43) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.47× (0.43–0.53) | 0.90× (0.88–0.91) | 0.72× (0.70–0.75) | 2.31× (2.27–2.34) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 500 | 2.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 874 | 2.21 | 1.08× (1.04–1.13) | 89% | 1.04× (1.01–1.07) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,034 | 3.41 | 1.69× (1.62–1.76) | 53% | 1.27× (1.23–1.32) | 1.71× (1.67–1.75) |

**Best:** mibz2 at 4 threads (1,034 MB/s); lbzip2 at 4 threads (228 MB/s); pbz2 at 4 threads (338 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.22, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.54 | 0.02 | 0.08 | 0.203 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3696 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.6% |
| MTF + RLE2 | 1.52 | 17.2% |
| T-vector build | 1.52 | 17.1% |
| walk (the one each build uses) | 4.26 | 48.1% |
| of which: segment assembly | 0.20 | 2.3% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.86 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.394 | 1.065 | (in its total) | 2.70× (2.70–2.71) |  |
| walk | 1.093 | 2.077 | (in its total) | 1.89× (1.88–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.042 | (in its total) | 0.89× (0.85–0.92) |  |
| inverse BWT, total | 1.535 | 3.183 | 1.907 | 2.07× (2.06–2.08) | 1.24× (1.24–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h02m ± 7m30s |
| 4 threads | 29m30s ± 4m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m03s ± 2m52s |
| the scanner alone (its serial part) | 11m12s ± 42.1s |
| the workers alone, over 4 | 20m03s ± 2m52s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 2.63 ns a byte of output at 4 threads (CPU, summed over them), 1.67 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,408,661 | 12,232,367 | \`2d4782182c8b0988…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
