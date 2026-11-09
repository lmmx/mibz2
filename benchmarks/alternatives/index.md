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

The latest record: [`5995a67`](<https://github.com/lmmx/mibz2/commit/5995a67a221433b1be2d092cc30fb60637c29923>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42619150518>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.537 s | 390 | 6.7 | 2.30× (2.26–2.34) | 5.28× (5.23–5.33) | 5.47× (5.43–5.52) | 6.25× (6.18–6.32) | 5.50× (5.41–5.60) | 6.10× (6.05–6.16) | 5.59× (5.51–5.68) | 1.71× (1.68–1.73) | 1.90× (1.87–1.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.311 s | 674 | 7.3 | 2.46× (2.35–2.58) | 4.97× (4.74–5.20) | 5.63× (5.40–5.86) | 5.84× (5.65–6.03) | 5.15× (4.97–5.34) | 10.60× (10.31–10.91) | 9.72× (9.40–10.06) | 2.97× (2.87–3.07) | 3.29× (3.18–3.41) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.264 s | 794 | 12.1 | 2.31× (2.24–2.39) | 3.40× (3.28–3.53) | 4.16× (4.00–4.32) | 4.17× (4.00–4.35) | 4.05× (3.92–4.17) | 12.40× (12.01–12.80) | 11.37× (10.98–11.76) | 3.47× (3.38–3.57) | 3.85× (3.70–4.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,111 | 2.6 | 0.56× (0.53–0.58) | 3.34× (3.22–3.47) | 4.46× (4.30–4.63) | 3.49× (3.37–3.62) | 5.98× (5.70–6.26) | 2.61× (2.51–2.72) | 2.61× (2.51–2.72) | 0.99× (0.91–1.07) | 1.93× (1.75–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,093 | 2.6 | 0.52× (0.51–0.53) | 2.85× (2.71–3.00) | 5.60× (5.45–5.76) | 3.51× (3.42–3.60) | 5.94× (5.85–6.04) | 2.62× (2.55–2.69) | 2.61× (2.55–2.68) | 0.99× (0.91–1.07) | 1.93× (1.76–2.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,087 | 2.6 | 0.52× (0.50–0.54) | 2.76× (2.68–2.85) | 5.52× (5.35–5.70) | 3.48× (3.36–3.59) | 5.97× (5.80–6.14) | 2.57× (2.49–2.66) | 2.57× (2.49–2.66) | 0.97× (0.89–1.06) | 1.90× (1.70–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 681 | 4.0 | 2.47× (2.45–2.50) | 2.83× (2.80–2.86) | 3.54× (3.50–3.58) | 3.78× (3.73–3.82) | 4.17× (4.09–4.25) | 3.16× (3.12–3.19) | 2.09× (2.07–2.12) | 1.93× (1.90–1.95) | 2.82× (2.79–2.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,040 | 4.9 | 2.21× (1.91–2.56) | 2.68× (2.48–2.90) | 3.93× (3.60–4.28) | 3.29× (3.06–3.54) | 4.63× (4.34–4.95) | 4.68× (4.38–5.00) | 3.10× (2.89–3.32) | 2.85× (2.67–3.05) | 4.17× (3.89–4.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 918 | 9.9 | 1.78× (1.68–1.88) | 1.78× (1.63–1.96) | 2.89× (2.73–3.07) | 2.12× (1.96–2.30) | 3.86× (3.68–4.04) | 4.24× (4.10–4.39) | 2.81× (2.72–2.90) | 2.59× (2.51–2.67) | 3.78× (3.65–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 36.6 | 1.40× (1.36–1.45) | 2.66× (2.62–2.69) | 2.41× (2.37–2.45) | 2.50× (2.47–2.52) | 2.77× (2.74–2.79) | 3.15× (3.12–3.18) | 3.13× (3.10–3.17) | 1.25× (1.24–1.26) | 1.44× (1.40–1.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 118 | 41.0 | 1.29× (1.26–1.32) | 2.49× (2.42–2.55) | 2.39× (2.33–2.46) | 3.11× (3.00–3.22) | 2.89× (2.81–2.98) | 5.11× (4.97–5.25) | 5.08× (4.95–5.21) | 2.02× (1.98–2.07) | 2.33× (2.25–2.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 54.7 | 1.36× (1.32–1.40) | 2.16× (2.08–2.25) | 2.18× (2.09–2.28) | 2.69× (2.60–2.78) | 2.94× (2.87–3.01) | 6.32× (6.17–6.48) | 6.29× (6.15–6.43) | 2.50× (2.44–2.57) | 2.89× (2.81–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 32.5 | 1.53× (1.52–1.54) | 3.09× (3.07–3.11) | 2.68× (2.64–2.73) | 2.84× (2.77–2.91) | 3.14× (3.09–3.18) | 4.00× (3.98–4.02) | 4.08× (4.05–4.11) | 1.55× (1.51–1.59) | 1.81× (1.79–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 129 | 37.2 | 1.34× (1.31–1.37) | 2.79× (2.63–2.97) | 2.53× (2.44–2.62) | 3.49× (3.40–3.58) | 3.02× (2.94–3.09) | 6.37× (6.21–6.53) | 6.49× (6.33–6.66) | 2.47× (2.38–2.56) | 2.88× (2.82–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 52.3 | 1.40× (1.34–1.46) | 2.22× (2.13–2.32) | 2.24× (2.18–2.29) | 3.06× (2.96–3.16) | 2.85× (2.77–2.94) | 7.49× (7.28–7.70) | 7.64× (7.45–7.82) | 2.90× (2.78–3.03) | 3.39× (3.27–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 3.5 | 0.62× (0.61–0.62) | 3.43× (3.38–3.47) | 4.30× (4.24–4.36) | 3.10× (3.04–3.15) | 6.63× (6.48–6.79) | 2.32× (2.29–2.35) | 2.31× (2.27–2.34) | 0.97× (0.94–0.99) | 1.72× (1.70–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 784 | 3.8 | 0.50× (0.48–0.52) | 2.41× (2.34–2.47) | 4.83× (4.70–4.96) | 2.89× (2.81–2.97) | 6.07× (5.88–6.26) | 2.16× (2.10–2.22) | 2.15× (2.09–2.21) | 0.90× (0.87–0.93) | 1.60× (1.56–1.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 784 | 3.7 | 0.50× (0.40–0.63) | 2.27× (1.89–2.72) | 4.54× (3.79–5.45) | 2.72× (2.27–3.26) | 5.86× (4.85–7.09) | 2.03× (1.69–2.44) | 2.02× (1.68–2.42) | 0.85× (0.71–1.01) | 1.51× (1.26–1.81) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.074 s | 166 | 16.1 | 1.30× (1.27–1.33) | 2.86× (2.80–2.92) | 3.12× (3.06–3.19) | 3.26× (3.19–3.33) | 3.54× (3.40–3.68) | 3.61× (3.52–3.70) | 3.41× (3.34–3.48) | 1.07× (1.05–1.09) | 1.24× (1.20–1.28) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.046 s | 266 | 18.5 | 1.17× (1.09–1.26) | 2.62× (2.35–2.91) | 2.99× (2.79–3.21) | 3.60× (3.50–3.70) | 3.44× (3.18–3.72) | 5.68× (5.35–6.03) | 5.37× (5.09–5.66) | 1.68× (1.60–1.78) | 1.95× (1.83–2.07) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.039 s | 318 | 27.1 | 1.28× (1.21–1.36) | 2.10× (1.97–2.23) | 2.73× (2.46–3.03) | 3.10× (2.94–3.27) | 3.42× (3.24–3.60) | 6.92× (6.72–7.13) | 6.54× (6.37–6.71) | 2.05× (2.00–2.10) | 2.37× (2.29–2.46) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.29× (2.26–2.33) | 5.25× (5.20–5.31) | 5.45× (5.40–5.49) | 6.23× (6.16–6.30) | 5.49× (5.39–5.59) | 6.08× (6.03–6.14) | 5.58× (5.50–5.66) | 1.71× (1.68–1.73) | 1.88× (1.85–1.91) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.26× (2.20–2.32) | 5.00× (4.88–5.13) | 5.20× (5.14–5.26) | 5.99× (5.92–6.07) | 5.08× (4.97–5.20) | 5.54× (5.47–5.61) | 5.08× (4.99–5.16) | 1.55× (1.52–1.58) | 1.71× (1.67–1.75) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.14× (2.07–2.21) | 3.53× (3.40–3.66) | 3.77× (3.62–3.94) | 4.28× (4.17–4.40) | 3.72× (3.57–3.87) | 3.31× (3.20–3.42) | 3.03× (2.92–3.14) | 0.93× (0.90–0.96) | 1.02× (0.98–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.58–0.62) | 2.89× (2.82–2.97) | 3.86× (3.76–3.96) | 3.30× (3.22–3.38) | 5.59× (5.39–5.79) | 2.49× (2.43–2.56) | 2.49× (2.43–2.56) | 1.00× (0.92–1.07) | 1.62× (1.58–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.58) | 2.77× (2.62–2.91) | 4.88× (4.78–4.99) | 3.27× (3.19–3.35) | 5.45× (5.36–5.53) | 2.44× (2.38–2.50) | 2.44× (2.38–2.50) | 0.97× (0.90–1.05) | 1.58× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 2.97× (2.89–3.05) | 4.96× (4.80–5.12) | 3.31× (3.22–3.41) | 5.57× (5.43–5.72) | 2.46× (2.39–2.53) | 2.46× (2.39–2.53) | 0.98× (0.91–1.06) | 1.60× (1.55–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.37–2.41) | 2.62× (2.60–2.65) | 3.13× (3.09–3.16) | 3.62× (3.58–3.66) | 4.00× (3.92–4.08) | 3.04× (3.01–3.06) | 2.03× (2.01–2.05) | 1.88× (1.86–1.90) | 2.46× (2.43–2.49) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.13× (2.08–2.19) | 2.35× (2.28–2.43) | 3.14× (3.07–3.21) | 3.14× (3.07–3.21) | 3.36× (3.30–3.43) | 2.50× (2.45–2.55) | 1.67× (1.63–1.71) | 1.54× (1.51–1.58) | 2.02× (1.97–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.85× (1.77–1.93) | 1.60× (1.52–1.68) | 2.07× (1.99–2.15) | 1.89× (1.82–1.96) | 1.96× (1.88–2.04) | 1.24× (1.19–1.29) | 0.83× (0.80–0.86) | 0.77× (0.74–0.80) | 1.01× (0.97–1.04) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.36–1.44) | 2.58× (2.55–2.61) | 2.35× (2.32–2.37) | 2.47× (2.45–2.50) | 2.74× (2.72–2.76) | 3.12× (3.09–3.15) | 3.10× (3.07–3.13) | 1.24× (1.23–1.25) | 1.38× (1.36–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.44× (2.39–2.50) | 2.14× (2.12–2.17) | 2.92× (2.89–2.95) | 2.53× (2.50–2.57) | 2.77× (2.73–2.82) | 2.76× (2.72–2.80) | 1.11× (1.09–1.12) | 1.23× (1.21–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.46–1.52) | 2.30× (2.24–2.36) | 1.93× (1.89–1.96) | 2.64× (2.60–2.68) | 2.49× (2.46–2.53) | 2.10× (2.07–2.14) | 2.09× (2.07–2.12) | 0.84× (0.83–0.85) | 0.93× (0.92–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.54) | 3.00× (2.98–3.02) | 2.63× (2.58–2.67) | 2.81× (2.74–2.89) | 3.11× (3.06–3.15) | 3.95× (3.94–3.97) | 4.03× (4.00–4.06) | 1.54× (1.50–1.58) | 1.75× (1.73–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.40) | 2.76× (2.72–2.80) | 2.38× (2.32–2.43) | 3.33× (3.28–3.39) | 2.75× (2.72–2.79) | 3.46× (3.41–3.51) | 3.53× (3.48–3.58) | 1.35× (1.31–1.39) | 1.53× (1.51–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.53–1.63) | 2.52× (2.45–2.60) | 2.01× (1.93–2.10) | 2.87× (2.79–2.95) | 2.62× (2.56–2.69) | 2.47× (2.41–2.54) | 2.52× (2.46–2.59) | 0.96× (0.92–1.01) | 1.09× (1.06–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.46× (2.43–2.48) | 3.39× (3.34–3.45) | 2.84× (2.79–2.89) | 5.93× (5.80–6.05) | 2.15× (2.12–2.18) | 2.14× (2.12–2.17) | 0.97× (0.95–0.99) | 1.36× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.55–0.58) | 2.19× (2.15–2.23) | 3.95× (3.85–4.05) | 2.66× (2.60–2.73) | 5.36× (5.21–5.52) | 1.97× (1.92–2.02) | 1.96× (1.92–2.01) | 0.89× (0.86–0.92) | 1.24× (1.22–1.27) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.45–0.71) | 2.32× (1.93–2.78) | 3.73× (3.10–4.49) | 2.58× (2.14–3.10) | 5.33× (4.40–6.47) | 1.91× (1.59–2.30) | 1.90× (1.58–2.29) | 0.86× (0.72–1.04) | 1.20× (1.00–1.45) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.29× (1.27–1.32) | 2.72× (2.66–2.78) | 3.01× (2.96–3.07) | 3.21× (3.15–3.28) | 3.48× (3.35–3.61) | 3.55× (3.46–3.64) | 3.35× (3.29–3.42) | 1.07× (1.05–1.09) | 1.18× (1.15–1.21) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.21× (1.17–1.25) | 2.48× (2.41–2.55) | 2.87× (2.71–3.04) | 3.20× (3.11–3.29) | 3.06× (2.86–3.27) | 3.05× (2.91–3.20) | 2.88× (2.77–3.00) | 0.92× (0.88–0.96) | 1.01× (0.97–1.06) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.40× (1.31–1.51) | 2.18× (2.06–2.31) | 2.59× (2.35–2.85) | 2.70× (2.60–2.81) | 2.73× (2.65–2.82) | 2.13× (2.08–2.18) | 2.01× (1.97–2.05) | 0.64× (0.63–0.65) | 0.71× (0.69–0.72) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.34–0.38) | 1.07× (1.07–1.07) | 0.86× (0.86–0.87) | 12.12× (12.10–12.14) | 0.28× (0.28–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.56× (0.50–0.62) | 0.78× (0.76–0.80) | 0.66× (0.64–0.67) | 6.27× (6.14–6.41) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.54× (0.48–0.62) | 0.70× (0.69–0.71) | 0.61× (0.60–0.62) | 4.08× (4.05–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.30× (4.29–4.30) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.17) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.44–0.45) | 1.71× (1.71–1.71) | 0.67× (0.66–0.67) | 8.07× (8.05–8.09) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.67–0.67) | 0.65× (0.59–0.72) | 1.62× (1.59–1.65) | 0.80× (0.78–0.81) | 5.86× (5.76–5.96) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.53× (0.53–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.61–0.69) | 0.70× (0.65–0.76) | 1.16× (1.12–1.21) | 0.71× (0.67–0.75) | 3.43× (3.29–3.57) | 0.27× (0.25–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.49–0.52) | 0.42× (0.40–0.45) | 0.93× (0.90–0.96) | 0.74× (0.72–0.77) | 3.21× (3.10–3.33) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.64) | 0.45× (0.44–0.45) | 0.85× (0.84–0.87) | 0.80× (0.79–0.82) | 2.49× (2.44–2.53) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.84× (0.83–0.84) | 5.24× (5.24–5.25) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.51) | 0.39× (0.36–0.41) | 0.88× (0.84–0.92) | 0.73× (0.70–0.77) | 3.01× (2.88–3.15) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.28–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.63) | 0.49× (0.46–0.52) | 0.81× (0.79–0.82) | 0.82× (0.78–0.86) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.23–0.25) | 1.08× (1.08–1.08) | 0.77× (0.77–0.77) | 5.41× (5.40–5.42) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.21× (0.20–0.21) | 1.62× (1.61–1.63) | 0.77× (0.77–0.78) | 5.37× (5.35–5.39) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.63× (1.63–1.64) | 0.78× (0.77–0.78) | 5.41× (5.40–5.42) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.47–1.48) | 0.68× (0.68–0.69) | 6.65× (6.64–6.66) | 0.40× (0.40–0.40) | 0.44× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.33) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.42× (0.39–0.46) | 1.17× (1.14–1.19) | 0.73× (0.71–0.76) | 3.65× (3.61–3.70) | 0.22× (0.22–0.22) | 0.24× (0.24–0.24) | 0.36× (0.35–0.36) | 0.18× (0.18–0.18) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 52 MB | 0.53× (0.51–0.55) | 0.48× (0.45–0.51) | 0.98× (0.96–1.01) | 0.79× (0.76–0.82) | 2.50× (2.42–2.58) | 0.14× (0.14–0.15) | 0.16× (0.15–0.16) | 0.23× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 390 | 2.57 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 674 | 2.80 | 1.10× (1.09–1.11) | 87% | 1.05× (1.02–1.08) | 1.08× (1.03–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 794 | 4.67 | 1.84× (1.77–1.91) | 51% | 1.24× (1.19–1.28) | 1.71× (1.66–1.77) |

**Best:** mibz2 at 4 threads (794 MB/s); lbzip2 at 4 threads (240 MB/s); pbz2 at 4 threads (343 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.39, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.80 | 0.02 | 0.09 | 0.264 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.1% |
| MTF + RLE2 | 1.11 | 17.1% |
| T-vector build | 1.25 | 19.3% |
| walk (the one each build uses) | 3.01 | 46.5% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.14 | 2.2% |
| CRC | 0.18 | 2.8% |
| **total** | 6.48 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.453 | 1.058 | (in its total) | 2.34× (2.33–2.34) |  |
| walk | 1.043 | 2.056 | (in its total) | 1.97× (1.97–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.041 | (in its total) | 1.11× (1.09–1.13) |  |
| inverse BWT, total | 1.534 | 3.156 | 1.887 | 2.06× (2.05–2.06) | 1.24× (1.22–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h18m ± 9m16s |
| 4 threads | 38m33s ± 5m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m19s ± 4m23s |
| the scanner alone (its serial part) | 12m13s ± 32.2s |
| the workers alone, over 4 | 29m19s ± 4m23s |

At 4 threads the workers bound it. Per byte: the scanner 7.1 ns a byte of bz2, the workers 3.84 ns a byte of output at 4 threads (CPU, summed over them), 2.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
