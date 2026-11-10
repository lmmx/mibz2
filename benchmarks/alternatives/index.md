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

The latest record: [`c87d3c8`](<https://github.com/lmmx/mibz2/commit/c87d3c8f6e88c207c98e0454c0014fee99737cbc>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42807664412>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.511 s | 410 | 9.0 | 2.39× (2.37–2.40) | 5.45× (5.41–5.48) | 5.64× (5.60–5.68) | 6.42× (6.39–6.45) | 5.64× (5.62–5.67) | 5.89× (5.86–5.92) | 5.55× (5.53–5.57) | 1.76× (1.75–1.77) | 1.96× (1.94–1.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.286 s | 732 | 9.7 | 2.48× (2.39–2.56) | 5.08× (4.94–5.22) | 5.86× (5.67–6.05) | 6.06× (5.84–6.29) | 5.37× (5.20–5.54) | 10.48× (10.15–10.82) | 9.87× (9.55–10.20) | 3.13× (3.03–3.24) | 3.48× (3.37–3.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.245 s | 857 | 16.1 | 2.33× (2.29–2.37) | 3.48× (3.43–3.53) | 4.39× (4.26–4.51) | 4.24× (4.16–4.32) | 4.21× (4.12–4.29) | 12.25× (12.08–12.43) | 11.54× (11.36–11.72) | 3.66× (3.60–3.72) | 4.07× (4.01–4.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,114 | 3.6 | 0.57× (0.56–0.58) | 3.42× (3.36–3.48) | 4.58× (4.49–4.66) | 3.57× (3.50–3.64) | 6.06× (5.94–6.19) | 2.66× (2.61–2.70) | 2.65× (2.61–2.70) | 0.96× (0.93–0.98) | 1.92× (1.73–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.53× (0.52–0.55) | 2.84× (2.76–2.92) | 5.69× (5.54–5.85) | 3.55× (3.45–3.64) | 5.95× (5.76–6.15) | 2.64× (2.56–2.71) | 2.63× (2.56–2.71) | 0.95× (0.92–0.98) | 1.91× (1.73–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,114 | 3.6 | 0.54× (0.53–0.55) | 2.86× (2.80–2.92) | 5.72× (5.60–5.84) | 3.58× (3.50–3.65) | 6.10× (5.97–6.24) | 2.65× (2.59–2.70) | 2.64× (2.59–2.70) | 0.95× (0.93–0.98) | 1.91× (1.72–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 709 | 5.5 | 2.60× (2.57–2.63) | 2.95× (2.92–2.99) | 3.66× (3.56–3.76) | 3.93× (3.89–3.98) | 4.30× (4.22–4.37) | 2.68× (2.64–2.72) | 2.17× (2.15–2.20) | 2.00× (1.99–2.02) | 2.94× (2.91–2.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,045 | 6.8 | 2.13× (1.93–2.36) | 2.75× (2.47–3.07) | 3.98× (3.69–4.29) | 3.41× (3.11–3.75) | 4.77× (4.33–5.25) | 3.90× (3.57–4.27) | 3.17× (2.89–3.47) | 2.92× (2.68–3.18) | 4.28× (3.91–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 958 | 13.4 | 1.84× (1.79–1.89) | 1.87× (1.71–2.05) | 2.96× (2.80–3.12) | 2.16× (2.07–2.26) | 4.03× (3.93–4.14) | 3.63× (3.57–3.69) | 2.94× (2.90–2.99) | 2.71× (2.68–2.74) | 3.98× (3.92–4.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.6 | 1.38× (1.38–1.39) | 2.65× (2.63–2.68) | 2.43× (2.38–2.48) | 2.50× (2.49–2.52) | 2.83× (2.80–2.86) | 3.00× (2.98–3.02) | 2.99× (2.97–3.01) | 1.25× (1.24–1.25) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 118 | 58.2 | 1.27× (1.24–1.30) | 2.52× (2.40–2.64) | 2.30× (2.22–2.39) | 3.04× (2.97–3.12) | 2.85× (2.77–2.93) | 4.82× (4.69–4.95) | 4.80× (4.67–4.93) | 2.00× (1.95–2.06) | 2.26× (2.17–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 77.4 | 1.31× (1.26–1.37) | 2.14× (2.03–2.27) | 2.12× (2.00–2.24) | 2.63× (2.54–2.73) | 2.89× (2.77–3.01) | 5.89× (5.65–6.14) | 5.87× (5.64–6.11) | 2.45× (2.35–2.55) | 2.77× (2.63–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 46.0 | 1.53× (1.53–1.54) | 3.09× (3.06–3.12) | 2.67× (2.65–2.68) | 2.79× (2.78–2.81) | 3.18× (3.09–3.28) | 3.72× (3.70–3.73) | 3.92× (3.90–3.94) | 1.52× (1.50–1.55) | 1.80× (1.79–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 134 | 52.2 | 1.37× (1.35–1.40) | 2.81× (2.72–2.91) | 2.55× (2.50–2.61) | 3.56× (3.53–3.59) | 3.12× (3.05–3.20) | 6.07× (5.96–6.18) | 6.40× (6.26–6.54) | 2.49× (2.44–2.55) | 2.94× (2.88–3.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 151 | 71.7 | 1.42× (1.39–1.46) | 2.22× (2.18–2.27) | 2.22× (2.13–2.33) | 3.05× (2.99–3.10) | 2.88× (2.82–2.95) | 6.98× (6.86–7.11) | 7.37× (7.24–7.50) | 2.87× (2.82–2.92) | 3.39× (3.33–3.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 5.0 | 0.62× (0.61–0.63) | 3.46× (3.42–3.49) | 4.35× (4.29–4.40) | 3.08× (3.05–3.12) | 6.62× (6.46–6.78) | 2.28× (2.25–2.31) | 2.30× (2.27–2.33) | 0.95× (0.93–0.96) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 800 | 5.3 | 0.51× (0.50–0.53) | 2.46× (2.39–2.53) | 4.95× (4.81–5.09) | 2.93× (2.85–3.02) | 6.26× (6.04–6.49) | 2.16× (2.10–2.21) | 2.18× (2.12–2.24) | 0.90× (0.87–0.93) | 1.64× (1.60–1.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.53× (0.51–0.54) | 2.48× (2.42–2.54) | 4.98× (4.87–5.10) | 2.96× (2.89–3.03) | 6.40× (6.23–6.58) | 2.17× (2.12–2.21) | 2.19× (2.14–2.24) | 0.90× (0.87–0.93) | 1.65× (1.61–1.69) | yes |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 1 | 0.073 s | 170 | 22.2 | 1.35× (1.34–1.36) | 2.93× (2.91–2.95) | 3.20× (3.18–3.22) | 3.41× (3.39–3.43) | 3.59× (3.55–3.62) | 3.52× (3.50–3.55) | 3.42× (3.40–3.44) | 1.12× (1.11–1.14) | 1.27× (1.24–1.30) | yes |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 2 | 0.046 s | 272 | 26.0 | 1.17× (1.12–1.22) | 2.65× (2.49–2.82) | 3.00× (2.88–3.12) | 3.65× (3.54–3.76) | 3.52× (3.40–3.64) | 5.56× (5.41–5.72) | 5.40× (5.26–5.55) | 1.78× (1.72–1.83) | 2.00× (1.92–2.08) | yes |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 4 | 0.040 s | 312 | 38.3 | 1.23× (1.18–1.28) | 2.08× (1.89–2.30) | 2.46× (2.32–2.61) | 2.95× (2.87–3.03) | 3.20× (3.09–3.31) | 6.44× (6.25–6.63) | 6.25× (6.07–6.43) | 2.05× (1.99–2.12) | 2.31× (2.25–2.38) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.38× (2.37–2.40) | 5.42× (5.39–5.46) | 5.61× (5.58–5.65) | 6.41× (6.38–6.44) | 5.63× (5.60–5.65) | 5.88× (5.85–5.90) | 5.53× (5.51–5.56) | 1.76× (1.75–1.77) | 1.94× (1.93–1.96) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.26× (2.24–2.28) | 5.13× (5.10–5.16) | 5.39× (5.32–5.45) | 6.24× (6.19–6.28) | 5.26× (5.21–5.32) | 5.45× (5.41–5.49) | 5.14× (5.10–5.18) | 1.63× (1.62–1.65) | 1.80× (1.79–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.19× (2.15–2.23) | 3.64× (3.56–3.72) | 3.93× (3.82–4.03) | 4.44× (4.35–4.53) | 3.87× (3.79–3.96) | 3.30× (3.24–3.37) | 3.11× (3.04–3.17) | 0.99× (0.97–1.01) | 1.09× (1.07–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.92× (2.88–2.97) | 3.90× (3.82–3.97) | 3.35× (3.29–3.41) | 5.62× (5.50–5.73) | 2.51× (2.47–2.55) | 2.51× (2.47–2.55) | 0.96× (0.94–0.98) | 1.64× (1.61–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.60) | 2.74× (2.67–2.81) | 4.94× (4.82–5.06) | 3.30× (3.22–3.39) | 5.45× (5.28–5.63) | 2.46× (2.39–2.52) | 2.46× (2.39–2.52) | 0.94× (0.91–0.97) | 1.61× (1.56–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.59) | 3.07× (3.01–3.13) | 5.03× (4.92–5.14) | 3.40× (3.33–3.46) | 5.68× (5.56–5.80) | 2.52× (2.47–2.56) | 2.51× (2.47–2.56) | 0.96× (0.94–0.98) | 1.64× (1.61–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.48–2.53) | 2.73× (2.71–2.76) | 3.24× (3.21–3.27) | 3.76× (3.72–3.80) | 4.11× (4.04–4.18) | 2.59× (2.54–2.63) | 2.11× (2.09–2.13) | 1.95× (1.94–1.96) | 2.53× (2.51–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.13–2.24) | 2.43× (2.38–2.49) | 3.22× (3.15–3.29) | 3.22× (3.14–3.30) | 3.46× (3.36–3.56) | 2.10× (2.05–2.15) | 1.71× (1.67–1.75) | 1.58× (1.54–1.61) | 2.06× (2.00–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.87–2.01) | 1.69× (1.65–1.72) | 2.17× (2.11–2.22) | 2.00× (1.94–2.06) | 2.07× (2.03–2.11) | 1.08× (1.05–1.10) | 0.88× (0.86–0.89) | 0.81× (0.79–0.83) | 1.05× (1.03–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.38) | 2.58× (2.57–2.59) | 2.37× (2.33–2.41) | 2.48× (2.47–2.49) | 2.80× (2.77–2.84) | 2.97× (2.95–2.99) | 2.96× (2.94–2.98) | 1.24× (1.23–1.25) | 1.37× (1.35–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.28) | 2.44× (2.38–2.51) | 2.14× (2.12–2.16) | 2.91× (2.88–2.95) | 2.52× (2.50–2.55) | 2.63× (2.60–2.67) | 2.63× (2.60–2.65) | 1.10× (1.09–1.12) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.42–1.50) | 2.27× (2.22–2.32) | 1.93× (1.89–1.97) | 2.64× (2.59–2.68) | 2.50× (2.45–2.55) | 1.98× (1.95–2.02) | 1.98× (1.95–2.01) | 0.83× (0.81–0.85) | 0.92× (0.90–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.52–1.53) | 3.00× (2.99–3.01) | 2.60× (2.59–2.62) | 2.77× (2.75–2.78) | 3.15× (3.06–3.25) | 3.67× (3.66–3.69) | 3.88× (3.85–3.90) | 1.52× (1.49–1.54) | 1.74× (1.73–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.37–1.41) | 2.77× (2.73–2.82) | 2.36× (2.32–2.40) | 3.32× (3.25–3.39) | 2.79× (2.73–2.84) | 3.22× (3.17–3.27) | 3.39× (3.33–3.46) | 1.33× (1.30–1.35) | 1.52× (1.49–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.54–1.71) | 2.58× (2.50–2.65) | 2.04× (1.98–2.10) | 2.91× (2.84–2.99) | 2.69× (2.62–2.75) | 2.35× (2.29–2.42) | 2.48× (2.41–2.56) | 0.97× (0.94–1.01) | 1.11× (1.09–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.46× (2.43–2.48) | 3.42× (3.36–3.49) | 2.83× (2.80–2.86) | 5.92× (5.79–6.05) | 2.12× (2.09–2.14) | 2.14× (2.12–2.16) | 0.95× (0.94–0.97) | 1.36× (1.34–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.56–0.60) | 2.24× (2.19–2.29) | 3.97× (3.89–4.05) | 2.71× (2.64–2.78) | 5.54× (5.36–5.73) | 1.97× (1.93–2.02) | 1.99× (1.95–2.04) | 0.89× (0.86–0.91) | 1.27× (1.23–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.56–0.59) | 2.51× (2.46–2.57) | 4.05× (3.97–4.13) | 2.81× (2.76–2.87) | 5.84× (5.68–5.99) | 2.04× (2.00–2.09) | 2.07× (2.02–2.11) | 0.92× (0.89–0.95) | 1.31× (1.28–1.35) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 1 | 1.34× (1.33–1.35) | 2.84× (2.82–2.87) | 3.12× (3.10–3.14) | 3.36× (3.34–3.37) | 3.53× (3.50–3.56) | 3.47× (3.44–3.49) | 3.37× (3.35–3.39) | 1.12× (1.11–1.13) | 1.23× (1.22–1.24) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 2 | 1.21× (1.20–1.23) | 2.54× (2.51–2.56) | 2.80× (2.77–2.83) | 3.30× (3.27–3.34) | 3.07× (3.04–3.11) | 2.96× (2.92–3.00) | 2.87× (2.84–2.91) | 0.96× (0.94–0.97) | 1.05× (1.03–1.06) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 4 | 1.39× (1.35–1.43) | 2.14× (2.09–2.19) | 2.37× (2.32–2.42) | 2.72× (2.68–2.76) | 2.64× (2.60–2.68) | 2.02× (1.99–2.06) | 1.97× (1.93–2.00) | 0.66× (0.64–0.67) | 0.72× (0.70–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.34) | 0.36× (0.35–0.37) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.07× (12.05–12.09) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.47× (0.47–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.39) | 0.51× (0.44–0.60) | 0.78× (0.77–0.80) | 0.65× (0.64–0.67) | 6.25× (6.12–6.39) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.61× (0.53–0.70) | 0.71× (0.69–0.72) | 0.62× (0.61–0.64) | 4.12× (4.06–4.18) | 0.10× (0.09–0.10) | 0.10× (0.10–0.11) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.38–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.65× (0.64–0.66) | 8.01× (8.00–8.02) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.68) | 0.67× (0.62–0.71) | 1.58× (1.54–1.62) | 0.80× (0.78–0.82) | 5.75× (5.60–5.90) | 0.46× (0.45–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.73× (0.68–0.78) | 1.15× (1.11–1.19) | 0.71× (0.68–0.74) | 3.40× (3.29–3.51) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.72–0.74) | 5.21× (5.19–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.42× (0.39–0.45) | 0.93× (0.90–0.97) | 0.75× (0.73–0.78) | 3.24× (3.13–3.35) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.90) | 0.83× (0.80–0.85) | 2.54× (2.48–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.19–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.49) | 0.38× (0.37–0.40) | 0.86× (0.84–0.88) | 0.73× (0.71–0.75) | 2.96× (2.89–3.02) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.53) | 0.81× (0.80–0.82) | 0.81× (0.78–0.84) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.38–0.38) | 0.24× (0.23–0.25) | 1.08× (1.07–1.08) | 0.77× (0.77–0.77) | 5.40× (5.38–5.41) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.33× (5.30–5.35) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.78× (0.77–0.78) | 5.36× (5.35–5.37) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 1 | 18 MB | 0.48× (0.48–0.49) | 0.40× (0.40–0.40) | 1.47× (1.47–1.48) | 0.70× (0.69–0.70) | 6.66× (6.65–6.68) | 0.40× (0.40–0.40) | 0.43× (0.42–0.44) | 0.66× (0.66–0.66) | 0.32× (0.32–0.33) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 2 | 35 MB | 0.47× (0.46–0.49) | 0.41× (0.37–0.45) | 1.10× (1.07–1.13) | 0.71× (0.68–0.75) | 3.47× (3.38–3.57) | 0.21× (0.20–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.47 MB → 12.4 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.52) | 0.45× (0.42–0.48) | 0.91× (0.89–0.94) | 0.73× (0.70–0.75) | 2.34× (2.28–2.40) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 410 | 2.44 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 732 | 2.64 | 1.08× (1.07–1.09) | 89% | 1.02× (1.01–1.03) | 1.02× (1.01–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 857 | 4.37 | 1.78× (1.74–1.82) | 52% | 1.19× (1.18–1.21) | 1.64× (1.60–1.67) |

**Best:** mibz2 at 4 threads (857 MB/s); lbzip2 at 4 threads (247 MB/s); pbz2 at 4 threads (368 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.12, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.73 | 0.02 | 0.09 | 0.245 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.9% |
| MTF + RLE2 | 1.58 | 18.4% |
| T-vector build | 1.48 | 17.3% |
| walk (the one each build uses) | 3.95 | 46.0% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.25 | 3.0% |
| **total** | 8.58 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.060 | (in its total) | 2.72× (2.71–2.72) |  |
| walk | 1.036 | 2.018 | (in its total) | 1.94× (1.93–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.039 | (in its total) | 1.07× (1.05–1.09) |  |
| inverse BWT, total | 1.463 | 3.118 | 1.864 | 2.13× (2.11–2.14) | 1.27× (1.26–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 8m42s |
| 4 threads | 35m51s ± 5m03s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m22s ± 3m47s |
| the scanner alone (its serial part) | 12m31s ± 32.2s |
| the workers alone, over 4 | 26m22s ± 3m47s |

At 4 threads the workers bound it. Per byte: the scanner 7.3 ns a byte of bz2, the workers 3.46 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,470,860 | 12,412,186 | \`65b0d5d32a2bdc38…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.0 (bench/report.py), 10 rounds, threads 1, 2, 4
