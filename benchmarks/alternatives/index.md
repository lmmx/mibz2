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

The latest record: [`ed2d368`](<https://github.com/lmmx/mibz2/commit/ed2d3683d988499da604a0f652471c8711fa429c>) (2026-11-07).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42362233063>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.660 s | 318 | 10.2 | 2.33× (2.32–2.34) | 5.31× (5.29–5.34) | 5.48× (5.45–5.52) | 5.92× (5.89–5.95) | 5.35× (5.32–5.37) | 5.74× (5.72–5.77) | 5.36× (5.34–5.39) | 1.72× (1.71–1.73) | 1.98× (1.97–1.99) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.369 s | 568 | 10.9 | 2.34× (2.28–2.41) | 5.05× (4.91–5.20) | 5.65× (5.49–5.82) | 5.61× (5.48–5.74) | 5.07× (4.96–5.18) | 10.23× (9.99–10.48) | 9.55× (9.33–9.77) | 3.06× (2.99–3.13) | 3.53× (3.44–3.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.303 s | 691 | 17.4 | 2.31× (2.25–2.36) | 3.71× (3.44–4.01) | 4.40× (4.29–4.51) | 4.07× (3.97–4.17) | 4.11× (4.01–4.21) | 12.40× (12.09–12.72) | 11.57× (11.30–11.85) | 3.71× (3.62–3.80) | 4.27× (4.16–4.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 995 | 3.6 | 0.60× (0.60–0.61) | 3.74× (3.55–3.93) | 4.52× (4.40–4.65) | 3.61× (3.56–3.65) | 6.23× (6.16–6.31) | 2.87× (2.83–2.90) | 2.69× (2.65–2.72) | 1.02× (1.00–1.03) | 1.53× (1.51–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 943 | 3.8 | 0.55× (0.54–0.56) | 2.92× (2.87–2.96) | 5.76× (5.67–5.84) | 3.47× (3.41–3.53) | 5.92× (5.83–6.01) | 2.74× (2.70–2.78) | 2.57× (2.53–2.61) | 0.97× (0.95–0.99) | 1.46× (1.44–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 959 | 3.7 | 0.57× (0.55–0.58) | 2.95× (2.89–3.01) | 5.84× (5.69–6.01) | 3.50× (3.43–3.56) | 5.99× (5.84–6.15) | 2.76× (2.71–2.82) | 2.59× (2.54–2.64) | 0.98× (0.95–1.01) | 1.47× (1.44–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 542 | 6.3 | 2.47× (2.46–2.48) | 2.82× (2.81–2.83) | 3.62× (3.60–3.64) | 3.55× (3.54–3.56) | 3.84× (3.83–3.86) | 2.33× (2.31–2.34) | 1.97× (1.96–1.97) | 1.88× (1.87–1.89) | 2.81× (2.79–2.82) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 820 | 7.8 | 2.08× (1.96–2.20) | 2.68× (2.54–2.82) | 3.74× (3.45–4.04) | 3.15× (2.99–3.33) | 4.44× (4.20–4.69) | 3.48× (3.32–3.66) | 2.94× (2.80–3.10) | 2.82× (2.69–2.96) | 4.20× (4.00–4.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 750 | 14.9 | 1.81× (1.71–1.92) | 1.91× (1.78–2.04) | 2.77× (2.71–2.84) | 2.10× (2.06–2.15) | 3.79× (3.73–3.85) | 3.20× (3.14–3.27) | 2.71× (2.65–2.77) | 2.59× (2.53–2.66) | 3.87× (3.78–3.96) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 54.3 | 1.34× (1.33–1.34) | 2.70× (2.69–2.70) | 2.46× (2.43–2.49) | 2.46× (2.45–2.47) | 2.84× (2.83–2.85) | 3.05× (3.03–3.07) | 3.09× (3.07–3.10) | 1.22× (1.21–1.23) | 1.38× (1.37–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 60.7 | 1.25× (1.22–1.29) | 2.51× (2.45–2.56) | 2.38× (2.33–2.43) | 3.00× (2.92–3.08) | 2.89× (2.82–2.96) | 4.92× (4.78–5.06) | 4.98× (4.84–5.13) | 1.97× (1.92–2.03) | 2.22× (2.16–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 84.1 | 1.28× (1.25–1.32) | 2.08× (2.04–2.12) | 2.05× (2.00–2.10) | 2.51× (2.46–2.56) | 2.77× (2.72–2.81) | 5.76× (5.68–5.85) | 5.83× (5.75–5.92) | 2.31× (2.28–2.34) | 2.61× (2.57–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 49.8 | 1.45× (1.44–1.46) | 3.04× (3.00–3.07) | 2.57× (2.56–2.58) | 2.63× (2.62–2.65) | 3.06× (3.04–3.07) | 3.71× (3.68–3.74) | 3.79× (3.77–3.81) | 1.43× (1.37–1.50) | 1.67× (1.66–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.3 | 1.33× (1.30–1.37) | 2.80× (2.73–2.88) | 2.51× (2.43–2.59) | 3.45× (3.38–3.52) | 3.06× (2.99–3.13) | 6.13× (6.01–6.25) | 6.26× (6.12–6.41) | 2.37× (2.23–2.51) | 2.75× (2.69–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 81.9 | 1.39× (1.32–1.45) | 2.25× (2.18–2.32) | 2.14× (2.11–2.17) | 2.92× (2.84–3.00) | 2.76× (2.68–2.84) | 6.92× (6.76–7.10) | 7.07× (6.91–7.24) | 2.67× (2.55–2.80) | 3.11× (3.04–3.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.0 | 0.67× (0.66–0.68) | 3.11× (2.95–3.28) | 4.45× (4.41–4.49) | 3.13× (3.10–3.16) | 6.81× (6.74–6.88) | 2.49× (2.46–2.51) | 2.34× (2.32–2.36) | 0.99× (0.97–1.00) | 1.53× (1.52–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 687 | 5.5 | 0.54× (0.52–0.57) | 2.78× (2.64–2.94) | 4.78× (4.53–5.04) | 2.89× (2.75–3.05) | 6.23× (5.90–6.59) | 2.27× (2.16–2.39) | 2.13× (2.02–2.25) | 0.90× (0.85–0.95) | 1.40× (1.32–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 690 | 5.3 | 0.56× (0.55–0.57) | 2.86× (2.82–2.89) | 4.89× (4.83–4.95) | 2.95× (2.92–2.99) | 6.38× (6.28–6.48) | 2.32× (2.29–2.36) | 2.18× (2.15–2.21) | 0.92× (0.90–0.94) | 1.43× (1.41–1.45) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 0.081 s | 148 | 22.4 | 1.51× (1.50–1.53) | 3.11× (3.08–3.15) | 3.45× (3.39–3.50) | 3.36× (3.32–3.40) | 3.59× (3.56–3.63) | 3.80× (3.76–3.84) | 3.59× (3.55–3.62) | 1.15× (1.14–1.17) | 1.38× (1.34–1.41) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 0.053 s | 228 | 26.0 | 1.27× (1.18–1.37) | 2.71× (2.46–2.99) | 3.40× (3.18–3.64) | 3.34× (3.09–3.62) | 3.33× (3.12–3.56) | 5.73× (5.37–6.12) | 5.41× (5.06–5.78) | 1.74× (1.63–1.86) | 2.08× (1.94–2.23) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 0.044 s | 272 | 38.0 | 1.31× (1.29–1.34) | 2.10× (2.03–2.17) | 2.95× (2.87–3.04) | 2.91× (2.84–2.99) | 3.15× (3.09–3.22) | 6.96× (6.82–7.11) | 6.57× (6.44–6.70) | 2.11× (2.06–2.16) | 2.53× (2.47–2.59) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.31–2.34) | 5.29× (5.27–5.32) | 5.46× (5.43–5.50) | 5.91× (5.88–5.93) | 5.33× (5.31–5.36) | 5.73× (5.70–5.75) | 5.35× (5.32–5.37) | 1.72× (1.71–1.72) | 1.97× (1.96–1.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.25× (2.25–2.26) | 5.04× (5.02–5.06) | 5.32× (5.31–5.33) | 5.81× (5.79–5.83) | 5.00× (4.98–5.02) | 5.35× (5.32–5.38) | 5.00× (4.98–5.01) | 1.60× (1.60–1.61) | 1.84× (1.83–1.85) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.19–2.24) | 3.76× (3.69–3.84) | 4.01× (3.95–4.07) | 4.25× (4.20–4.31) | 3.80× (3.74–3.86) | 3.33× (3.29–3.38) | 3.11× (3.07–3.15) | 1.00× (0.99–1.01) | 1.15× (1.13–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.31× (3.27–3.35) | 4.16× (4.05–4.27) | 3.38× (3.34–3.42) | 5.77× (5.70–5.84) | 2.70× (2.67–2.73) | 2.54× (2.51–2.57) | 1.02× (1.00–1.03) | 1.09× (1.06–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.99× (2.94–3.05) | 5.23× (5.15–5.32) | 3.23× (3.17–3.28) | 5.42× (5.33–5.51) | 2.55× (2.51–2.59) | 2.40× (2.35–2.44) | 0.96× (0.94–0.98) | 1.03× (1.01–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.33× (3.25–3.41) | 5.37× (5.24–5.50) | 3.30× (3.24–3.36) | 5.54× (5.40–5.68) | 2.61× (2.56–2.66) | 2.45× (2.41–2.50) | 0.98× (0.95–1.01) | 1.05× (1.02–1.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.38–2.40) | 2.61× (2.59–2.62) | 3.27× (3.25–3.29) | 3.42× (3.40–3.43) | 3.70× (3.68–3.71) | 2.26× (2.24–2.27) | 1.92× (1.90–1.93) | 1.84× (1.83–1.84) | 2.52× (2.49–2.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.09–2.19) | 2.42× (2.35–2.49) | 2.88× (2.83–2.94) | 2.97× (2.92–3.03) | 3.18× (3.12–3.25) | 1.85× (1.82–1.89) | 1.57× (1.54–1.61) | 1.51× (1.48–1.54) | 2.07× (2.02–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.86–1.96) | 1.66× (1.61–1.71) | 2.08× (2.04–2.13) | 2.01× (1.97–2.05) | 1.98× (1.94–2.03) | 0.95× (0.93–0.98) | 0.81× (0.79–0.83) | 0.78× (0.76–0.79) | 1.07× (1.04–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.33–1.34) | 2.63× (2.62–2.63) | 2.42× (2.38–2.45) | 2.44× (2.43–2.45) | 2.82× (2.80–2.83) | 3.02× (3.00–3.03) | 3.05× (3.04–3.07) | 1.22× (1.21–1.23) | 1.35× (1.34–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.24–1.26) | 2.45× (2.43–2.48) | 2.18× (2.16–2.21) | 2.88× (2.86–2.91) | 2.54× (2.52–2.57) | 2.68× (2.66–2.71) | 2.71× (2.68–2.74) | 1.08× (1.07–1.10) | 1.20× (1.19–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.41–1.47) | 2.27× (2.25–2.30) | 1.88× (1.85–1.90) | 2.53× (2.49–2.56) | 2.41× (2.37–2.44) | 1.96× (1.93–1.98) | 1.98× (1.96–2.00) | 0.79× (0.78–0.80) | 0.88× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.44–1.45) | 2.98× (2.97–2.99) | 2.51× (2.49–2.52) | 2.61× (2.60–2.62) | 3.03× (3.01–3.05) | 3.68× (3.64–3.71) | 3.75× (3.73–3.77) | 1.43× (1.36–1.50) | 1.62× (1.62–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.35) | 2.76× (2.72–2.79) | 2.29× (2.27–2.32) | 3.25× (3.22–3.27) | 2.73× (2.70–2.76) | 3.24× (3.20–3.29) | 3.31× (3.27–3.35) | 1.26× (1.20–1.33) | 1.43× (1.42–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.49–1.56) | 2.50× (2.45–2.55) | 1.89× (1.85–1.92) | 2.72× (2.66–2.77) | 2.48× (2.42–2.54) | 2.24× (2.20–2.29) | 2.29× (2.25–2.33) | 0.87× (0.83–0.91) | 0.99× (0.97–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.71–0.72) | 2.75× (2.71–2.79) | 3.57× (3.55–3.59) | 2.86× (2.84–2.89) | 6.08× (6.04–6.12) | 2.30× (2.28–2.32) | 2.17× (2.15–2.19) | 0.99× (0.98–1.00) | 1.02× (0.97–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.47× (2.43–2.51) | 4.17× (4.11–4.23) | 2.71× (2.66–2.76) | 5.59× (5.50–5.69) | 2.10× (2.07–2.13) | 1.99× (1.96–2.02) | 0.91× (0.89–0.92) | 0.93× (0.89–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.62) | 2.76× (2.72–2.80) | 4.29× (4.22–4.36) | 2.81× (2.78–2.84) | 5.78× (5.70–5.86) | 2.17× (2.14–2.20) | 2.05× (2.02–2.08) | 0.93× (0.92–0.95) | 0.96× (0.92–1.01) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 1.50× (1.49–1.52) | 3.02× (2.98–3.05) | 3.30× (3.26–3.35) | 3.31× (3.27–3.35) | 3.53× (3.50–3.57) | 3.73× (3.69–3.78) | 3.52× (3.49–3.56) | 1.15× (1.13–1.16) | 1.34× (1.33–1.36) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 1.39× (1.37–1.41) | 2.70× (2.68–2.73) | 2.99× (2.96–3.02) | 3.20× (3.17–3.24) | 3.09× (3.06–3.12) | 3.23× (3.20–3.26) | 3.05× (3.02–3.08) | 0.99× (0.98–1.00) | 1.16× (1.15–1.17) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 1.55× (1.49–1.60) | 2.30× (2.24–2.35) | 2.50× (2.45–2.56) | 2.61× (2.56–2.67) | 2.59× (2.54–2.64) | 2.21× (2.17–2.26) | 2.09× (2.05–2.13) | 0.68× (0.67–0.70) | 0.80× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.23× (12.21–12.24) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.47× (0.47–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.61× (0.55–0.67) | 0.81× (0.80–0.81) | 0.67× (0.66–0.68) | 6.38× (6.35–6.41) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.74× (0.66–0.83) | 0.71× (0.70–0.71) | 0.61× (0.61–0.62) | 4.10× (4.09–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.83× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.33–4.34) | 0.26× (0.25–0.26) | 0.26× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.83× (0.83–0.84) | 4.44× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.45× (0.45–0.46) | 1.74× (1.74–1.74) | 0.66× (0.66–0.67) | 8.20× (8.18–8.22) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.75× (0.75–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.67× (0.66–0.69) | 0.77× (0.71–0.85) | 1.67× (1.63–1.71) | 0.81× (0.79–0.83) | 6.03× (5.89–6.17) | 0.48× (0.47–0.49) | 0.50× (0.49–0.52) | 0.55× (0.54–0.56) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.62–0.67) | 0.74× (0.71–0.78) | 1.16× (1.13–1.20) | 0.75× (0.73–0.77) | 3.41× (3.31–3.52) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.41× (0.37–0.44) | 0.93× (0.90–0.96) | 0.69× (0.66–0.72) | 3.21× (3.12–3.31) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.47) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 2.52× (2.46–2.58) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.29× (5.29–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.48) | 0.36× (0.33–0.39) | 0.86× (0.84–0.88) | 0.72× (0.70–0.75) | 2.93× (2.86–3.01) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.61) | 0.51× (0.48–0.54) | 0.81× (0.80–0.83) | 0.80× (0.78–0.83) | 2.34× (2.30–2.39) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.22) | 1.10× (1.10–1.10) | 0.77× (0.77–0.77) | 5.48× (5.46–5.49) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.42× (5.40–5.44) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.20–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.77) | 5.44× (5.42–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.51× (1.50–1.51) | 0.65× (0.65–0.65) | 6.78× (6.76–6.80) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 31 MB | 0.51× (0.50–0.51) | 0.45× (0.42–0.49) | 1.25× (1.23–1.28) | 0.75× (0.72–0.77) | 3.90× (3.86–3.95) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.38× (0.37–0.39) | 0.19× (0.19–0.19) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 48 MB | 0.56× (0.54–0.57) | 0.57× (0.53–0.61) | 1.04× (1.01–1.08) | 0.81× (0.77–0.86) | 2.67× (2.60–2.75) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 318 | 3.16 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 568 | 3.38 | 1.07× (1.07–1.07) | 89% | 1.02× (1.01–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 691 | 5.40 | 1.72× (1.70–1.74) | 54% | 1.22× (1.21–1.23) | 1.64× (1.62–1.65) |

**Best:** mibz2 at 4 threads (691 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.07, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.92 | 0.02 | 0.09 | 0.303 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3231 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.0% |
| MTF + RLE2 | 1.51 | 15.9% |
| T-vector build | 2.12 | 22.4% |
| walk (the one each build uses) | 4.17 | 43.9% |
| of which: segment assembly | 0.16 | 1.7% |
| RLE1 | 0.29 | 3.1% |
| CRC | 0.26 | 2.7% |
| **total** | 9.49 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.300 | (in its total) | 1.93× (1.93–1.94) |  |
| walk | 1.266 | 2.651 | (in its total) | 2.10× (2.08–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.053 | (in its total) | 1.07× (1.06–1.08) |  |
| inverse BWT, total | 1.988 | 4.005 | 3.310 | 2.02× (2.01–2.03) | 1.67× (1.66–1.67) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h35m ± 11m13s |
| 4 threads | 44m28s ± 6m20s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m26s ± 4m42s |
| the scanner alone (its serial part) | 15m52s ± 19.7s |
| the workers alone, over 4 | 33m26s ± 4m42s |

At 4 threads the workers bound it. Per byte: the scanner 9.2 ns a byte of bz2, the workers 4.38 ns a byte of output at 4 threads (CPU, summed over them), 2.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,758,792 | 12,027,975 | \`3c64b8b3acbf9c14…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3230.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
