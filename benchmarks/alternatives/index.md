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

The latest record: [`e0e6e4a`](<https://github.com/lmmx/mibz2/commit/e0e6e4a47d909e9e00393c77793732e680f82b8f>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40834102377>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 546 | 4.8 | 2.13× (2.03–2.23) | 5.35× (5.11–5.61) | 5.47× (5.22–5.74) | 6.26× (5.97–6.56) | 5.58× (5.33–5.85) | 5.93× (5.64–6.23) | 5.47× (5.22–5.73) | 1.64× (1.56–1.72) | 1.92× (1.82–2.02) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.282 s | 743 | 5.8 | 1.78× (1.73–1.83) | 3.91× (3.77–4.05) | 4.64× (4.51–4.77) | 4.70× (4.57–4.84) | 4.24× (4.10–4.38) | 8.35× (8.09–8.62) | 7.70× (7.44–7.98) | 2.31× (2.23–2.39) | 2.70× (2.61–2.80) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.266 s | 789 | 10.4 | 1.50× (1.44–1.55) | 2.47× (2.39–2.55) | 3.03× (2.90–3.17) | 2.92× (2.82–3.03) | 3.04× (2.99–3.10) | 8.78× (8.45–9.13) | 8.10× (7.83–8.39) | 2.43× (2.34–2.51) | 2.84× (2.74–2.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,360 | 2.1 | 0.54× (0.53–0.56) | 3.46× (3.36–3.57) | 4.57× (4.03–5.18) | 3.53× (3.41–3.66) | 6.63× (6.40–6.85) | 2.63× (2.55–2.71) | 2.62× (2.55–2.70) | 1.02× (0.98–1.06) | 2.08× (2.02–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,379 | 2.1 | 0.51× (0.50–0.52) | 2.79× (2.72–2.87) | 5.74× (5.58–5.91) | 3.58× (3.51–3.64) | 6.59× (6.47–6.71) | 2.64× (2.57–2.71) | 2.63× (2.56–2.71) | 1.02× (0.98–1.07) | 2.09× (2.04–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,347 | 2.1 | 0.50× (0.49–0.52) | 2.75× (2.68–2.82) | 5.64× (5.51–5.79) | 3.50× (3.40–3.60) | 6.49× (6.29–6.69) | 2.59× (2.53–2.66) | 2.59× (2.52–2.65) | 1.00× (0.99–1.02) | 2.05× (2.00–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.022 s | 871 | 3.2 | 2.35× (2.30–2.40) | 3.10× (2.98–3.23) | 3.39× (3.21–3.57) | 3.68× (3.57–3.80) | 4.57× (4.47–4.68) | 2.25× (2.19–2.31) | 2.14× (2.08–2.20) | 1.72× (1.67–1.76) | 2.69× (2.64–2.75) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,264 | 4.0 | 1.89× (1.76–2.04) | 2.59× (2.45–2.74) | 3.59× (3.30–3.92) | 3.16× (2.99–3.34) | 5.24× (4.94–5.57) | 3.21× (3.03–3.40) | 3.06× (2.89–3.25) | 2.46× (2.31–2.61) | 3.85× (3.64–4.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,037 | 8.5 | 1.35× (1.31–1.40) | 1.66× (1.61–1.72) | 2.37× (2.27–2.47) | 1.92× (1.83–2.01) | 3.89× (3.80–3.99) | 2.64× (2.55–2.74) | 2.52× (2.44–2.61) | 2.02× (1.96–2.08) | 3.17× (3.07–3.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.087 s | 92 | 28.8 | 1.24× (1.21–1.26) | 2.89× (2.81–2.98) | 2.49× (2.44–2.55) | 2.49× (2.44–2.55) | 2.88× (2.82–2.94) | 3.33× (3.26–3.40) | 3.16× (3.09–3.24) | 1.25× (1.22–1.27) | 1.35× (1.30–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.056 s | 144 | 31.8 | 1.11× (1.06–1.16) | 2.66× (2.47–2.87) | 2.39× (2.29–2.50) | 2.94× (2.77–3.13) | 2.90× (2.74–3.08) | 5.16× (4.98–5.35) | 4.91× (4.69–5.13) | 1.93× (1.85–2.02) | 2.09× (1.96–2.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 183 | 43.0 | 1.18× (1.16–1.21) | 2.30× (2.24–2.36) | 2.27× (2.18–2.36) | 2.70× (2.59–2.82) | 3.06× (3.00–3.11) | 6.60× (6.48–6.72) | 6.27× (6.14–6.40) | 2.47× (2.42–2.53) | 2.68× (2.58–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.112 s | 86 | 30.6 | 1.10× (1.08–1.11) | 2.82× (2.78–2.86) | 2.22× (2.20–2.25) | 2.27× (2.25–2.30) | 2.66× (2.60–2.71) | 3.46× (3.42–3.50) | 3.44× (3.40–3.49) | 1.20× (1.19–1.22) | 1.37× (1.34–1.40) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 33.9 | 1.04× (1.00–1.07) | 2.57× (2.52–2.63) | 2.22× (2.16–2.28) | 3.10× (2.99–3.21) | 2.73× (2.68–2.78) | 5.78× (5.67–5.89) | 5.75× (5.63–5.87) | 2.01× (1.97–2.05) | 2.28× (2.24–2.33) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.054 s | 179 | 46.3 | 1.11× (1.06–1.15) | 2.22× (2.08–2.38) | 2.11× (2.02–2.20) | 2.78× (2.71–2.87) | 2.74× (2.62–2.86) | 7.13× (6.90–7.36) | 7.09× (6.86–7.32) | 2.48× (2.41–2.55) | 2.82× (2.74–2.90) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 2.9 | 0.60× (0.59–0.61) | 3.14× (3.10–3.18) | 4.43× (4.38–4.49) | 3.00× (2.96–3.03) | 7.26× (7.16–7.36) | 2.25× (2.21–2.30) | 2.24× (2.20–2.28) | 1.09× (0.87–1.38) | 2.10× (2.07–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 3.1 | 0.49× (0.48–0.50) | 3.02× (2.94–3.09) | 5.28× (5.14–5.41) | 2.89× (2.81–2.96) | 7.22× (6.90–7.56) | 2.17× (2.10–2.23) | 2.15× (2.10–2.21) | 1.05× (0.84–1.32) | 2.02× (1.97–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 3.0 | 0.50× (0.49–0.51) | 3.04× (2.98–3.10) | 5.30× (5.19–5.40) | 2.94× (2.87–3.01) | 7.04× (6.86–7.22) | 2.17× (2.11–2.24) | 2.16× (2.11–2.21) | 1.06× (0.84–1.33) | 2.02× (1.98–2.06) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 0.047 s | 262 | 10.2 | 1.35× (1.32–1.39) | 3.21× (3.13–3.30) | 3.49× (3.44–3.53) | 3.57× (3.48–3.66) | 4.06× (3.96–4.17) | 3.73× (3.63–3.83) | 3.43× (3.33–3.52) | 1.13× (1.10–1.16) | 1.28× (1.25–1.32) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 0.031 s | 396 | 12.3 | 1.07× (1.00–1.15) | 2.67× (2.48–2.88) | 3.16× (2.95–3.39) | 3.59× (3.24–3.99) | 3.90× (3.64–4.18) | 5.51× (5.11–5.93) | 5.06× (4.69–5.45) | 1.66× (1.55–1.79) | 1.90× (1.76–2.04) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 0.029 s | 425 | 19.4 | 1.03× (0.99–1.07) | 2.01× (1.88–2.15) | 2.53× (2.40–2.67) | 2.84× (2.73–2.95) | 3.43× (3.34–3.52) | 6.11× (5.94–6.29) | 5.62× (5.47–5.77) | 1.85× (1.79–1.90) | 2.10× (2.04–2.16) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.12× (2.03–2.23) | 5.32× (5.08–5.58) | 5.44× (5.20–5.70) | 6.24× (5.96–6.54) | 5.57× (5.31–5.83) | 5.91× (5.62–6.21) | 5.45× (5.21–5.71) | 1.64× (1.56–1.71) | 1.90× (1.80–2.00) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.87× (1.81–1.94) | 4.60× (4.42–4.80) | 4.81× (4.65–4.99) | 5.62× (5.42–5.83) | 4.76× (4.56–4.96) | 5.04× (4.84–5.24) | 4.65× (4.45–4.85) | 1.39× (1.34–1.45) | 1.62× (1.55–1.69) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.60× (1.54–1.65) | 3.01× (2.91–3.12) | 3.16× (3.03–3.30) | 3.58× (3.43–3.73) | 3.19× (3.12–3.26) | 2.75× (2.64–2.86) | 2.54× (2.45–2.63) | 0.76× (0.73–0.79) | 0.88× (0.85–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.60) | 2.94× (2.83–3.05) | 4.04× (3.54–4.61) | 3.32× (3.20–3.44) | 6.13× (5.92–6.36) | 2.49× (2.41–2.57) | 2.48× (2.40–2.57) | 1.02× (0.98–1.06) | 1.76× (1.71–1.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.58) | 2.75× (2.68–2.82) | 4.99× (4.88–5.10) | 3.31× (3.26–3.37) | 5.99× (5.87–6.12) | 2.45× (2.39–2.52) | 2.45× (2.38–2.51) | 1.00× (0.97–1.04) | 1.74× (1.69–1.79) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.56) | 2.94× (2.87–3.01) | 5.03× (4.88–5.18) | 3.31× (3.23–3.40) | 6.02× (5.84–6.20) | 2.46× (2.41–2.52) | 2.46× (2.40–2.52) | 1.01× (0.99–1.03) | 1.75× (1.72–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.27× (2.23–2.32) | 2.66× (2.61–2.72) | 3.15× (3.06–3.24) | 3.53× (3.41–3.65) | 4.37× (4.27–4.47) | 2.18× (2.12–2.24) | 2.08× (2.02–2.14) | 1.68× (1.64–1.72) | 2.25× (2.22–2.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.88× (1.79–1.97) | 2.23× (2.11–2.37) | 2.66× (2.53–2.79) | 2.84× (2.69–2.99) | 3.49× (3.30–3.68) | 1.67× (1.58–1.76) | 1.60× (1.51–1.69) | 1.29× (1.22–1.36) | 1.73× (1.65–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.50× (1.44–1.57) | 1.49× (1.42–1.57) | 1.74× (1.66–1.83) | 1.77× (1.69–1.87) | 1.93× (1.83–2.04) | 0.82× (0.77–0.87) | 0.78× (0.74–0.83) | 0.63× (0.60–0.67) | 0.85× (0.81–0.89) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.23× (1.20–1.26) | 2.80× (2.73–2.86) | 2.41× (2.35–2.46) | 2.47× (2.41–2.52) | 2.85× (2.79–2.91) | 3.29× (3.22–3.36) | 3.13× (3.06–3.20) | 1.24× (1.22–1.27) | 1.29× (1.26–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.16× (1.15–1.18) | 2.62× (2.57–2.66) | 2.24× (2.20–2.28) | 2.89× (2.85–2.94) | 2.62× (2.57–2.67) | 2.95× (2.91–2.99) | 2.80× (2.75–2.85) | 1.11× (1.10–1.13) | 1.16× (1.13–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.30× (1.26–1.34) | 2.39× (2.33–2.45) | 2.00× (1.94–2.07) | 2.57× (2.50–2.64) | 2.48× (2.43–2.53) | 2.19× (2.15–2.23) | 2.08× (2.03–2.13) | 0.83× (0.81–0.84) | 0.86× (0.84–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.10× (1.08–1.11) | 2.75× (2.71–2.78) | 2.17× (2.15–2.19) | 2.26× (2.23–2.29) | 2.64× (2.58–2.69) | 3.43× (3.39–3.47) | 3.41× (3.37–3.46) | 1.20× (1.19–1.22) | 1.31× (1.28–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.05) | 2.56× (2.53–2.58) | 2.03× (2.01–2.06) | 2.88× (2.83–2.92) | 2.41× (2.37–2.44) | 3.08× (3.04–3.11) | 3.06× (3.02–3.10) | 1.08× (1.07–1.09) | 1.18× (1.15–1.21) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.20× (1.17–1.23) | 2.38× (2.22–2.56) | 1.73× (1.68–1.79) | 2.53× (2.43–2.63) | 2.29× (2.17–2.42) | 2.24× (2.17–2.32) | 2.23× (2.16–2.31) | 0.79× (0.76–0.81) | 0.86× (0.83–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.64–0.66) | 2.44× (2.38–2.49) | 3.28× (3.23–3.33) | 2.76× (2.71–2.80) | 6.49× (6.37–6.63) | 2.11× (2.06–2.17) | 2.09× (2.05–2.14) | 1.09× (0.88–1.35) | 1.44× (1.42–1.47) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.58) | 2.25× (2.21–2.30) | 3.95× (3.86–4.04) | 2.65× (2.59–2.72) | 6.35× (6.07–6.63) | 1.98× (1.92–2.05) | 1.96× (1.91–2.02) | 1.03× (0.83–1.26) | 1.36× (1.32–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.56) | 2.45× (2.39–2.51) | 4.02× (3.92–4.11) | 2.78× (2.72–2.85) | 6.36× (6.19–6.54) | 2.05× (1.99–2.11) | 2.03× (1.99–2.08) | 1.06× (0.86–1.31) | 1.40× (1.38–1.42) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 1.34× (1.31–1.38) | 3.08× (2.99–3.17) | 3.31× (3.25–3.37) | 3.50× (3.41–3.59) | 3.97× (3.87–4.08) | 3.65× (3.55–3.75) | 3.36× (3.26–3.45) | 1.12× (1.09–1.15) | 1.24× (1.21–1.27) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 1.18× (1.15–1.20) | 2.66× (2.59–2.74) | 2.92× (2.86–2.98) | 3.24× (3.17–3.32) | 3.40× (3.29–3.51) | 3.05× (2.98–3.12) | 2.81× (2.74–2.87) | 0.94× (0.92–0.96) | 1.04× (1.01–1.06) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 1.18× (1.14–1.21) | 2.08× (2.05–2.12) | 2.31× (2.25–2.37) | 2.45× (2.42–2.49) | 2.54× (2.49–2.59) | 1.93× (1.88–1.97) | 1.77× (1.72–1.82) | 0.59× (0.58–0.61) | 0.65× (0.64–0.67) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.36× (0.35–0.37) | 1.17× (1.16–1.17) | 0.85× (0.83–0.87) | 11.83× (11.82–11.84) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.53× (0.45–0.64) | 0.86× (0.85–0.87) | 0.67× (0.65–0.68) | 6.35× (6.28–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.65× (0.54–0.78) | 0.77× (0.76–0.78) | 0.64× (0.63–0.65) | 4.26× (4.24–4.28) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.27–4.28) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.44× (0.42–0.45) | 1.63× (1.62–1.63) | 0.64× (0.63–0.64) | 7.75× (7.74–7.76) | 0.62× (0.62–0.62) | 0.65× (0.65–0.66) | 0.71× (0.71–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.64× (0.62–0.66) | 0.64× (0.59–0.70) | 1.51× (1.49–1.53) | 0.87× (0.85–0.88) | 5.48× (5.42–5.55) | 0.44× (0.43–0.44) | 0.46× (0.45–0.47) | 0.50× (0.49–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.59× (0.58–0.61) | 0.65× (0.61–0.69) | 1.10× (1.08–1.13) | 0.85× (0.84–0.86) | 3.19× (3.14–3.25) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.14× (1.14–1.15) | 0.71× (0.71–0.71) | 5.06× (5.05–5.06) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.51) | 0.41× (0.39–0.44) | 0.93× (0.92–0.94) | 0.75× (0.74–0.75) | 3.23× (3.21–3.26) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 52 MB | 0.59× (0.56–0.61) | 0.43× (0.42–0.44) | 0.81× (0.79–0.84) | 0.77× (0.74–0.79) | 2.38× (2.31–2.44) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.81× (0.81–0.81) | 5.09× (5.08–5.10) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.50× (0.50–0.51) | 0.25× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.40× (0.39–0.41) | 0.87× (0.85–0.88) | 0.73× (0.71–0.76) | 2.99× (2.94–3.05) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.54× (0.53–0.56) | 0.44× (0.41–0.47) | 0.73× (0.71–0.74) | 0.73× (0.71–0.75) | 2.11× (2.07–2.15) | 0.12× (0.12–0.13) | 0.13× (0.13–0.13) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.41× (5.40–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.62) | 0.77× (0.77–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.39× (5.38–5.41) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.45× (1.44–1.45) | 0.63× (0.63–0.63) | 6.54× (6.53–6.56) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.64× (0.64–0.65) | 0.32× (0.31–0.32) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 31 MB | 0.52× (0.50–0.53) | 0.43× (0.40–0.47) | 1.26× (1.23–1.30) | 0.76× (0.73–0.79) | 3.94× (3.86–4.03) | 0.24× (0.23–0.24) | 0.25× (0.24–0.26) | 0.39× (0.38–0.40) | 0.19× (0.19–0.20) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 48 MB | 0.55× (0.54–0.57) | 0.54× (0.53–0.55) | 1.03× (1.01–1.05) | 0.81× (0.78–0.84) | 2.64× (2.59–2.68) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 546 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 743 | 2.24 | 1.17× (1.13–1.22) | 70% | 1.01× (1.00–1.03) | 1.03× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 789 | 4.01 | 2.15× (2.09–2.21) | 37% | 1.22× (1.20–1.23) | 1.62× (1.56–1.67) |

**Best:** mibz2 at 4 threads (789 MB/s); lbzip2 at 4 threads (318 MB/s); pbz2 at 4 threads (532 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.06, lbzip2 1t 0.07, lbzip2 2t 0.28, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.69 | 0.01 | 0.08 | 0.266 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.60 | 12.9% |
| MTF + RLE2 | 1.09 | 23.4% |
| T-vector build | 0.83 | 17.8% |
| walk (the one each build uses) | 1.91 | 41.2% |
| of which: segment assembly | 0.07 | 1.4% |
| RLE1 | 0.09 | 1.8% |
| CRC | 0.15 | 3.2% |
| **total** | 4.65 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.327 | 0.678 | (in its total) | 2.05× (1.99–2.11) |  |
| walk | 0.741 | 1.547 | (in its total) | 2.06× (1.96–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.024 | 0.028 | (in its total) | 1.13× (1.08–1.18) |  |
| inverse BWT, total | 1.092 | 2.253 | 1.309 | 2.04× (1.95–2.12) | 1.19× (1.15–1.24) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m32s ± 7m24s |
| 4 threads | 38m48s ± 5m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m32s ± 3m45s |
| the scanner alone (its serial part) | 9m01s ± 39.5s |
| the workers alone, over 4 | 25m32s ± 3m44s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 3.35 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,478,819 | 12,273,030 | \`80f51ade7d6ce928…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
