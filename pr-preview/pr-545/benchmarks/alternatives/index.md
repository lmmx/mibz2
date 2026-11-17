# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-545/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c7c2d78`](<https://github.com/lmmx/mibz2/commit/c7c2d787e04b3606f8249ded687fa0cb248712aa>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44095812630>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.523 s | 401 | 6.1 | 2.91× (2.90–2.92) | 6.68× (6.67–6.69) | 6.81× (6.79–6.83) | 7.43× (7.40–7.45) | 6.68× (6.67–6.70) | 7.30× (7.25–7.36) | 6.72× (6.70–6.74) | 2.14× (2.12–2.15) | 2.50× (2.49–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.298 s | 704 | 6.7 | 2.89× (2.76–3.04) | 6.10× (5.94–6.25) | 6.92× (6.76–7.09) | 6.92× (6.78–7.06) | 6.22× (6.07–6.37) | 12.70× (12.37–13.03) | 11.68× (11.41–11.96) | 3.72× (3.62–3.81) | 4.35× (4.24–4.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 859 | 10.2 | 2.83× (2.80–2.87) | 4.39× (4.31–4.47) | 5.41× (5.32–5.51) | 4.96× (4.89–5.03) | 5.09× (4.88–5.30) | 15.54× (15.32–15.78) | 14.30× (14.11–14.50) | 4.55× (4.49–4.61) | 5.32× (5.24–5.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,002 | 2.7 | 0.61× (0.60–0.62) | 3.72× (3.54–3.90) | 4.52× (4.46–4.58) | 3.61× (3.56–3.65) | 6.17× (6.08–6.25) | 3.03× (2.99–3.07) | 2.71× (2.67–2.74) | 1.00× (0.99–1.01) | 1.54× (1.52–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 978 | 2.8 | 0.58× (0.55–0.61) | 3.02× (2.96–3.08) | 5.96× (5.85–6.07) | 3.58× (3.52–3.65) | 6.11× (5.97–6.27) | 2.97× (2.92–3.03) | 2.66× (2.60–2.71) | 0.98× (0.97–1.00) | 1.51× (1.48–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 930 | 2.9 | 0.55× (0.54–0.55) | 2.87× (2.84–2.90) | 5.64× (5.58–5.70) | 3.38× (3.34–3.43) | 5.78× (5.71–5.86) | 2.81× (2.78–2.84) | 2.51× (2.49–2.54) | 0.93× (0.92–0.94) | 1.43× (1.41–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 589 | 4.4 | 2.67× (2.64–2.69) | 3.06× (3.04–3.07) | 3.83× (3.71–3.95) | 3.86× (3.84–3.88) | 4.14× (4.12–4.16) | 2.57× (2.55–2.58) | 2.12× (2.11–2.14) | 2.04× (2.02–2.05) | 3.04× (3.03–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 891 | 5.4 | 2.27× (2.16–2.38) | 2.93× (2.76–3.11) | 4.10× (3.99–4.21) | 3.45× (3.34–3.56) | 4.81× (4.68–4.93) | 3.88× (3.79–3.98) | 3.21× (3.14–3.28) | 3.08× (3.00–3.16) | 4.60× (4.50–4.71) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 747 | 10.8 | 1.76× (1.70–1.82) | 1.86× (1.71–2.01) | 2.80× (2.70–2.90) | 2.06× (2.00–2.12) | 3.69× (3.57–3.82) | 3.27× (3.16–3.37) | 2.70× (2.61–2.79) | 2.59× (2.50–2.68) | 3.87× (3.75–3.99) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 41.1 | 1.33× (1.32–1.33) | 2.69× (2.69–2.69) | 2.39× (2.35–2.43) | 2.48× (2.47–2.48) | 2.83× (2.80–2.87) | 3.09× (3.09–3.10) | 3.07× (3.06–3.07) | 1.21× (1.20–1.22) | 1.38× (1.37–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 46.5 | 1.24× (1.21–1.26) | 2.53× (2.43–2.63) | 2.36× (2.31–2.42) | 3.07× (2.99–3.15) | 2.89× (2.82–2.96) | 5.01× (4.89–5.14) | 4.97× (4.85–5.09) | 1.96× (1.91–2.01) | 2.23× (2.18–2.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 63.8 | 1.26× (1.24–1.29) | 2.13× (2.07–2.18) | 2.02× (1.95–2.09) | 2.55× (2.46–2.64) | 2.76× (2.72–2.80) | 5.89× (5.79–5.98) | 5.83× (5.75–5.92) | 2.30× (2.26–2.35) | 2.62× (2.58–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 37.8 | 1.41× (1.38–1.44) | 3.01× (2.94–3.07) | 2.55× (2.50–2.60) | 2.62× (2.57–2.67) | 3.01× (2.95–3.06) | 3.56× (3.50–3.62) | 3.73× (3.67–3.80) | 1.36× (1.33–1.38) | 1.65× (1.62–1.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 42.6 | 1.31× (1.22–1.40) | 2.77× (2.57–2.99) | 2.40× (2.30–2.51) | 3.36× (3.24–3.49) | 3.00× (2.89–3.11) | 5.85× (5.63–6.07) | 6.13× (5.89–6.39) | 2.23× (2.14–2.31) | 2.71× (2.61–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 122 | 61.5 | 1.36× (1.33–1.39) | 2.24× (2.18–2.31) | 2.19× (2.09–2.30) | 2.95× (2.85–3.06) | 2.71× (2.68–2.74) | 6.68× (6.61–6.76) | 7.01× (6.91–7.11) | 2.54× (2.52–2.57) | 3.10× (3.06–3.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 760 | 3.7 | 0.68× (0.66–0.69) | 3.10× (3.04–3.17) | 4.54× (4.45–4.63) | 3.17× (3.11–3.23) | 6.86× (6.71–7.02) | 2.60× (2.55–2.65) | 2.37× (2.33–2.42) | 0.99× (0.98–1.01) | 1.56× (1.53–1.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 717 | 4.0 | 0.56× (0.55–0.58) | 2.93× (2.88–2.99) | 5.03× (4.93–5.13) | 3.03× (2.98–3.09) | 6.47× (6.32–6.62) | 2.45× (2.40–2.50) | 2.24× (2.19–2.28) | 0.94× (0.92–0.96) | 1.47× (1.44–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 667 | 4.2 | 0.54× (0.53–0.54) | 2.75× (2.73–2.77) | 4.71× (4.67–4.74) | 2.84× (2.82–2.86) | 6.04× (5.98–6.10) | 2.29× (2.27–2.31) | 2.09× (2.07–2.11) | 0.88× (0.86–0.89) | 1.38× (1.37–1.39) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 0.068 s | 188 | 13.4 | 1.98× (1.95–2.01) | 4.17× (4.07–4.28) | 4.44× (4.36–4.52) | 4.52× (4.44–4.61) | 4.70× (4.62–4.79) | 5.27× (5.17–5.37) | 4.76× (4.68–4.85) | 1.50× (1.48–1.53) | 1.94× (1.90–1.97) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 0.046 s | 274 | 16.6 | 1.54× (1.39–1.71) | 3.21× (2.88–3.58) | 3.75× (3.43–4.11) | 4.49× (3.87–5.20) | 4.09× (3.70–4.51) | 7.32× (6.64–8.08) | 6.62× (6.01–7.29) | 2.09× (1.89–2.30) | 2.69× (2.44–2.96) | yes |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 0.043 s | 296 | 26.6 | 1.46× (1.42–1.50) | 2.34× (2.26–2.44) | 3.08× (2.97–3.18) | 3.33× (3.25–3.41) | 3.64× (3.57–3.71) | 8.35× (8.23–8.48) | 7.55× (7.41–7.69) | 2.38× (2.34–2.42) | 3.07× (3.02–3.12) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.90× (2.89–2.91) | 6.65× (6.63–6.66) | 6.78× (6.76–6.80) | 7.40× (7.38–7.43) | 6.66× (6.65–6.68) | 7.28× (7.22–7.33) | 6.70× (6.68–6.71) | 2.13× (2.12–2.15) | 2.49× (2.47–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.76–2.78) | 6.24× (6.20–6.28) | 6.51× (6.49–6.54) | 7.19× (7.16–7.22) | 6.18× (6.16–6.21) | 6.69× (6.64–6.74) | 6.16× (6.13–6.18) | 1.96× (1.95–1.97) | 2.28× (2.28–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.84× (2.80–2.88) | 4.79× (4.71–4.87) | 5.15× (5.08–5.22) | 5.44× (5.35–5.52) | 4.84× (4.78–4.91) | 4.38× (4.31–4.45) | 4.03× (3.97–4.08) | 1.28× (1.26–1.30) | 1.49× (1.48–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.34× (3.29–3.38) | 4.12× (4.07–4.17) | 3.39× (3.34–3.44) | 5.74× (5.65–5.82) | 2.86× (2.82–2.90) | 2.56× (2.53–2.60) | 1.00× (0.99–1.01) | 1.08× (1.06–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.60–0.65) | 3.09× (3.04–3.14) | 5.37× (5.28–5.47) | 3.32× (3.27–3.37) | 5.57× (5.45–5.70) | 2.74× (2.70–2.79) | 2.46× (2.42–2.50) | 0.96× (0.94–0.98) | 1.03× (1.01–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.58–0.59) | 3.28× (3.22–3.33) | 5.18× (5.15–5.22) | 3.21× (3.18–3.24) | 5.37× (5.32–5.43) | 2.66× (2.64–2.68) | 2.38× (2.36–2.40) | 0.93× (0.92–0.94) | 1.00× (0.98–1.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.57× (2.55–2.60) | 2.81× (2.79–2.83) | 3.49× (3.46–3.52) | 3.71× (3.68–3.73) | 3.97× (3.94–3.99) | 2.48× (2.47–2.50) | 2.06× (2.04–2.08) | 1.98× (1.96–2.00) | 2.72× (2.70–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.22–2.28) | 2.55× (2.48–2.62) | 3.04× (3.00–3.09) | 3.17× (3.14–3.20) | 3.36× (3.31–3.40) | 2.01× (1.98–2.03) | 1.67× (1.64–1.69) | 1.60× (1.57–1.63) | 2.20× (2.18–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.86–2.06) | 1.69× (1.61–1.78) | 2.14× (2.05–2.23) | 2.01× (1.93–2.09) | 2.06× (1.97–2.15) | 1.00× (0.96–1.05) | 0.83× (0.80–0.87) | 0.80× (0.77–0.84) | 1.10× (1.05–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.32–1.33) | 2.62× (2.61–2.62) | 2.35× (2.33–2.38) | 2.45× (2.45–2.46) | 2.81× (2.78–2.84) | 3.06× (3.06–3.07) | 3.04× (3.03–3.04) | 1.21× (1.20–1.22) | 1.35× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.24) | 2.44× (2.40–2.48) | 2.15× (2.12–2.18) | 2.88× (2.84–2.91) | 2.52× (2.49–2.55) | 2.71× (2.67–2.74) | 2.68× (2.65–2.72) | 1.07× (1.05–1.08) | 1.19× (1.18–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.39–1.45) | 2.22× (2.16–2.27) | 1.82× (1.79–1.85) | 2.49× (2.46–2.52) | 2.38× (2.34–2.41) | 1.98× (1.95–2.01) | 1.96× (1.93–1.99) | 0.78× (0.77–0.79) | 0.87× (0.86–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.38–1.43) | 2.96× (2.90–3.02) | 2.49× (2.44–2.53) | 2.60× (2.55–2.65) | 2.98× (2.93–3.03) | 3.53× (3.47–3.59) | 3.70× (3.63–3.76) | 1.35× (1.33–1.38) | 1.61× (1.58–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.27–1.40) | 2.74× (2.71–2.77) | 2.28× (2.26–2.30) | 3.21× (3.18–3.23) | 2.70× (2.68–2.72) | 3.14× (3.11–3.16) | 3.29× (3.25–3.33) | 1.20× (1.19–1.21) | 1.43× (1.42–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.50–1.57) | 2.48× (2.42–2.54) | 1.91× (1.87–1.96) | 2.72× (2.66–2.78) | 2.47× (2.41–2.54) | 2.21× (2.16–2.25) | 2.31× (2.27–2.36) | 0.85× (0.83–0.86) | 1.01× (0.99–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.81× (2.76–2.86) | 3.68× (3.59–3.77) | 2.92× (2.88–2.96) | 6.17× (6.05–6.29) | 2.42× (2.39–2.45) | 2.21× (2.18–2.24) | 1.00× (0.98–1.02) | 1.00× (0.95–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.55× (2.52–2.58) | 4.31× (4.27–4.35) | 2.80× (2.77–2.83) | 5.71× (5.64–5.79) | 2.23× (2.20–2.25) | 2.04× (2.02–2.05) | 0.92× (0.90–0.94) | 0.92× (0.88–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.58–0.59) | 2.66× (2.63–2.69) | 4.09× (4.05–4.13) | 2.70× (2.68–2.72) | 5.48× (5.44–5.53) | 2.14× (2.13–2.16) | 1.96× (1.94–1.97) | 0.89× (0.87–0.90) | 0.88× (0.85–0.92) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 1.96× (1.95–1.97) | 4.01× (3.99–4.03) | 4.33× (4.31–4.34) | 4.46× (4.43–4.48) | 4.63× (4.60–4.67) | 5.18× (5.16–5.21) | 4.69× (4.67–4.72) | 1.50× (1.49–1.51) | 1.79× (1.78–1.80) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 1.67× (1.65–1.69) | 3.39× (3.35–3.43) | 3.69× (3.65–3.74) | 4.13× (4.08–4.18) | 3.83× (3.79–3.87) | 4.19× (4.14–4.23) | 3.79× (3.74–3.84) | 1.21× (1.20–1.22) | 1.45× (1.43–1.47) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 1.68× (1.62–1.74) | 2.57× (2.49–2.64) | 2.81× (2.74–2.88) | 3.11× (3.05–3.17) | 2.95× (2.88–3.03) | 2.63× (2.57–2.69) | 2.38× (2.32–2.44) | 0.76× (0.74–0.78) | 0.91× (0.89–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.30× (0.28–0.31) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.99–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.50× (0.43–0.59) | 0.69× (0.67–0.71) | 0.57× (0.56–0.59) | 5.43× (5.27–5.59) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.34) | 0.66× (0.58–0.75) | 0.60× (0.57–0.62) | 0.51× (0.49–0.53) | 3.44× (3.30–3.58) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.70) | 0.73× (0.66–0.79) | 1.61× (1.56–1.66) | 0.80× (0.78–0.83) | 5.84× (5.66–6.03) | 0.47× (0.46–0.49) | 0.50× (0.48–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.74× (0.70–0.79) | 1.16× (1.14–1.18) | 0.71× (0.69–0.73) | 3.42× (3.35–3.48) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.17× (5.17–5.18) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.44) | 0.90× (0.87–0.94) | 0.71× (0.67–0.74) | 3.11× (2.99–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.64) | 0.45× (0.44–0.46) | 0.86× (0.84–0.87) | 0.78× (0.75–0.81) | 2.49× (2.44–2.55) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.18× (5.17–5.20) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.39× (0.38–0.40) | 0.84× (0.82–0.87) | 0.69× (0.66–0.73) | 2.88× (2.80–2.97) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.58–0.61) | 0.49× (0.46–0.51) | 0.80× (0.78–0.81) | 0.80× (0.77–0.82) | 2.29× (2.26–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.08–1.08) | 0.77× (0.76–0.77) | 5.38× (5.37–5.40) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.43) | 0.66× (0.66–0.67) | 6.40× (6.38–6.42) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.32× (0.31–0.32) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 2 | 40 MB | 0.43× (0.42–0.45) | 0.39× (0.36–0.43) | 1.02× (0.98–1.05) | 0.65× (0.63–0.68) | 3.18× (3.08–3.29) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.5 MB → 12.7 MB, 5.1×) | 4 | 58 MB | 0.48× (0.47–0.50) | 0.44× (0.41–0.48) | 0.88× (0.86–0.90) | 0.71× (0.69–0.74) | 2.25× (2.20–2.30) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 401 | 2.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 704 | 2.73 | 1.09× (1.08–1.09) | 87% | 1.02× (1.02–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 859 | 4.19 | 1.66× (1.64–1.68) | 53% | 1.20× (1.19–1.21) | 1.63× (1.61–1.64) |

**Best:** mibz2 at 4 threads (859 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (301 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.32, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.68 | 0.02 | 0.08 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.6% |
| MTF + RLE2 | 1.17 | 17.2% |
| T-vector build | 1.14 | 16.9% |
| walk (the one each build uses) | 3.20 | 47.3% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.9% |
| **total** | 6.77 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.491 | 1.304 | (in its total) | 2.66× (2.64–2.67) |  |
| walk | 1.262 | 2.658 | (in its total) | 2.11× (2.09–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.055 | (in its total) | 1.18× (1.16–1.21) |  |
| inverse BWT, total | 1.801 | 4.017 | 3.277 | 2.23× (2.22–2.25) | 1.82× (1.81–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m41s |
| 4 threads | 35m46s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m31s ± 3m28s |
| the scanner alone (its serial part) | 14m07s ± 30.2s |
| the workers alone, over 4 | 24m31s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 3.21 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,501,823 | 12,718,405 | \`5e8127c5c1057fcd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
