# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-682/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`bb98c99`](<https://github.com/lmmx/mibz2/commit/bb98c9958c5d5158e3d23d1da1939b747eef6391>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44858830640>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.508 s | 412 | 7.9 | 3.02× (3.00–3.03) | 6.94× (6.88–7.01) | 7.36× (7.27–7.46) | 7.71× (7.66–7.77) | 6.95× (6.89–7.01) | 7.54× (7.50–7.57) | 7.10× (7.06–7.14) | 2.25× (2.23–2.28) | 2.61× (2.58–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.299 s | 701 | 8.7 | 3.02× (2.93–3.12) | 6.26× (6.04–6.49) | 7.30× (6.99–7.62) | 7.04× (6.82–7.28) | 6.43× (6.23–6.63) | 12.77× (12.39–13.17) | 12.04× (11.66–12.44) | 3.82× (3.71–3.93) | 4.43× (4.32–4.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.247 s | 849 | 13.4 | 3.01× (2.96–3.06) | 4.71× (4.62–4.81) | 5.81× (5.68–5.93) | 5.36× (5.23–5.49) | 5.60× (5.37–5.85) | 15.58× (15.41–15.75) | 14.69× (14.52–14.86) | 4.66× (4.57–4.75) | 5.41× (5.30–5.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 948 | 3.8 | 0.59× (0.58–0.60) | 3.78× (3.62–3.94) | 4.60× (4.44–4.77) | 3.49× (3.46–3.53) | 6.11× (6.04–6.18) | 2.89× (2.86–2.92) | 2.59× (2.56–2.62) | 1.02× (1.00–1.04) | 1.46× (1.45–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 935 | 3.9 | 0.56× (0.55–0.57) | 2.89× (2.86–2.93) | 6.16× (6.09–6.24) | 3.47× (3.43–3.52) | 5.98× (5.88–6.07) | 2.86× (2.82–2.89) | 2.56× (2.53–2.59) | 1.01× (1.00–1.02) | 1.45× (1.43–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 909 | 3.9 | 0.55× (0.54–0.56) | 2.83× (2.78–2.88) | 6.01× (5.90–6.12) | 3.38× (3.33–3.43) | 5.92× (5.82–6.02) | 2.78× (2.73–2.84) | 2.49× (2.45–2.54) | 0.98× (0.97–1.00) | 1.41× (1.38–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 566 | 6.1 | 2.91× (2.81–3.01) | 2.96× (2.92–3.00) | 3.84× (3.74–3.95) | 3.75× (3.70–3.79) | 4.15× (4.10–4.20) | 2.51× (2.47–2.54) | 2.07× (2.05–2.09) | 2.02× (1.99–2.04) | 2.95× (2.91–2.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 834 | 7.7 | 1.97× (1.63–2.36) | 2.39× (2.03–2.81) | 3.43× (2.86–4.12) | 2.84× (2.38–3.38) | 4.14× (3.48–4.93) | 3.24× (2.73–3.86) | 2.68× (2.26–3.19) | 2.61× (2.20–3.10) | 3.82× (3.21–4.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 761 | 14.8 | 1.88× (1.77–1.99) | 2.00× (1.90–2.10) | 2.96× (2.78–3.15) | 2.20× (2.06–2.34) | 3.86× (3.70–4.03) | 3.30× (3.17–3.44) | 2.73× (2.63–2.84) | 2.66× (2.56–2.77) | 3.88× (3.73–4.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 60 | 55.4 | 1.33× (1.33–1.34) | 2.66× (2.64–2.68) | 2.42× (2.40–2.43) | 2.43× (2.41–2.45) | 2.84× (2.82–2.87) | 3.12× (3.09–3.15) | 3.14× (3.11–3.17) | 1.24× (1.23–1.26) | 1.43× (1.42–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 62.5 | 1.27× (1.24–1.29) | 2.56× (2.39–2.74) | 2.36× (2.30–2.44) | 3.05× (2.98–3.12) | 2.92× (2.84–3.00) | 5.04× (4.90–5.19) | 5.08× (4.96–5.19) | 2.01× (1.97–2.05) | 2.31× (2.24–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 84.1 | 1.31× (1.27–1.35) | 2.27× (2.18–2.36) | 2.16× (2.06–2.26) | 2.61× (2.58–2.65) | 2.93× (2.87–2.99) | 6.03× (5.92–6.16) | 6.08× (5.98–6.18) | 2.40× (2.38–2.43) | 2.76× (2.70–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 50.7 | 1.44× (1.43–1.46) | 3.06× (3.04–3.09) | 2.54× (2.52–2.57) | 2.60× (2.58–2.63) | 3.11× (3.04–3.18) | 3.65× (3.60–3.69) | 3.97× (3.92–4.02) | 1.46× (1.44–1.47) | 1.73× (1.71–1.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 107 | 57.4 | 1.32× (1.28–1.36) | 2.74× (2.67–2.81) | 2.47× (2.37–2.57) | 3.42× (3.29–3.55) | 3.07× (3.01–3.14) | 5.95× (5.76–6.14) | 6.47× (6.29–6.66) | 2.38× (2.31–2.44) | 2.82× (2.73–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 80.4 | 1.40× (1.37–1.44) | 2.28× (2.18–2.38) | 2.25× (2.13–2.37) | 2.96× (2.89–3.03) | 2.88× (2.80–2.96) | 6.91× (6.74–7.08) | 7.52× (7.36–7.68) | 2.76× (2.70–2.82) | 3.28× (3.21–3.35) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 712 | 5.2 | 0.65× (0.64–0.66) | 3.27× (3.02–3.53) | 4.27× (4.22–4.32) | 3.06× (3.02–3.10) | 6.77× (6.66–6.88) | 2.47× (2.44–2.50) | 2.28× (2.25–2.31) | 0.99× (0.97–1.01) | 1.48× (1.46–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 5.7 | 0.54× (0.53–0.56) | 2.76× (2.70–2.82) | 4.95× (4.67–5.24) | 2.88× (2.82–2.95) | 6.33× (6.21–6.45) | 2.31× (2.25–2.37) | 2.13× (2.08–2.19) | 0.93× (0.90–0.95) | 1.38× (1.35–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 667 | 5.5 | 0.54× (0.54–0.55) | 2.74× (2.70–2.79) | 5.01× (4.73–5.31) | 2.87× (2.83–2.91) | 6.28× (6.15–6.41) | 2.30× (2.26–2.34) | 2.13× (2.08–2.17) | 0.92× (0.90–0.94) | 1.38× (1.35–1.40) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.074 s | 182 | 18.3 | 1.95× (1.91–1.98) | 4.08× (4.02–4.14) | 4.43× (4.35–4.51) | 4.41× (4.38–4.45) | 4.61× (4.56–4.67) | 5.18× (5.11–5.25) | 4.92× (4.85–4.99) | 1.52× (1.50–1.54) | 1.89× (1.85–1.93) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.052 s | 258 | 22.8 | 1.59× (1.43–1.76) | 3.46× (3.23–3.71) | 3.74× (3.49–4.00) | 3.90× (3.60–4.22) | 4.00× (3.70–4.31) | 7.07× (6.52–7.65) | 6.71× (6.20–7.28) | 2.07× (1.92–2.24) | 2.58× (2.36–2.82) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.046 s | 290 | 34.9 | 1.52× (1.47–1.58) | 2.66× (2.45–2.89) | 3.34× (3.21–3.47) | 3.29× (3.19–3.38) | 3.91× (3.61–4.22) | 8.18× (7.93–8.44) | 7.77× (7.57–7.99) | 2.40× (2.34–2.46) | 2.98× (2.87–3.10) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.01× (2.99–3.03) | 6.91× (6.85–6.97) | 7.33× (7.23–7.42) | 7.69× (7.63–7.74) | 6.92× (6.87–6.98) | 7.51× (7.48–7.54) | 7.08× (7.04–7.12) | 2.25× (2.22–2.27) | 2.60× (2.57–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.92× (2.88–2.96) | 6.50× (6.44–6.57) | 6.93× (6.79–7.08) | 7.46× (7.42–7.49) | 6.49× (6.42–6.57) | 6.84× (6.79–6.89) | 6.45× (6.43–6.47) | 2.05× (2.02–2.08) | 2.37× (2.33–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.04× (2.98–3.10) | 5.22× (5.12–5.32) | 5.52× (5.42–5.62) | 5.86× (5.78–5.95) | 5.39× (5.25–5.53) | 4.42× (4.35–4.50) | 4.17× (4.10–4.24) | 1.32× (1.30–1.35) | 1.53× (1.51–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.63) | 3.20× (3.16–3.23) | 4.03× (3.98–4.09) | 3.28× (3.25–3.30) | 5.66× (5.60–5.72) | 2.72× (2.70–2.75) | 2.45× (2.43–2.47) | 1.02× (1.00–1.03) | 1.05× (1.02–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 2.97× (2.93–3.02) | 5.29× (5.21–5.36) | 3.21× (3.17–3.26) | 5.44× (5.36–5.52) | 2.64× (2.60–2.67) | 2.37× (2.34–2.40) | 0.99× (0.98–0.99) | 1.02× (0.98–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.25× (3.19–3.32) | 5.28× (5.20–5.37) | 3.21× (3.17–3.26) | 5.49× (5.39–5.58) | 2.63× (2.58–2.68) | 2.37× (2.32–2.41) | 0.98× (0.97–1.00) | 1.02× (0.98–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.80× (2.71–2.90) | 2.75× (2.71–2.79) | 3.53× (3.44–3.63) | 3.59× (3.55–3.64) | 3.97× (3.92–4.02) | 2.42× (2.39–2.45) | 2.01× (1.99–2.03) | 1.96× (1.94–1.98) | 2.67× (2.63–2.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.23× (2.16–2.31) | 2.41× (2.35–2.47) | 2.97× (2.90–3.05) | 2.99× (2.93–3.06) | 3.26× (3.19–3.33) | 1.90× (1.86–1.95) | 1.58× (1.55–1.62) | 1.54× (1.51–1.58) | 2.10× (2.04–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.91–2.09) | 1.71× (1.63–1.79) | 2.15× (2.06–2.25) | 2.01× (1.91–2.12) | 2.05× (1.96–2.13) | 0.99× (0.95–1.03) | 0.82× (0.79–0.86) | 0.80× (0.77–0.84) | 1.09× (1.04–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.33) | 2.60× (2.59–2.62) | 2.35× (2.33–2.38) | 2.41× (2.39–2.43) | 2.81× (2.79–2.84) | 3.08× (3.05–3.11) | 3.10× (3.08–3.13) | 1.24× (1.22–1.25) | 1.37× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.24–1.27) | 2.46× (2.43–2.50) | 2.18× (2.15–2.20) | 2.87× (2.84–2.90) | 2.55× (2.51–2.59) | 2.73× (2.68–2.77) | 2.74× (2.72–2.77) | 1.09× (1.08–1.11) | 1.21× (1.19–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.52) | 2.39× (2.33–2.45) | 1.94× (1.91–1.97) | 2.61× (2.53–2.68) | 2.56× (2.48–2.65) | 2.05× (2.00–2.09) | 2.06× (2.02–2.10) | 0.82× (0.80–0.84) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.45) | 2.99× (2.96–3.02) | 2.50× (2.48–2.52) | 2.58× (2.56–2.61) | 3.08× (3.01–3.15) | 3.61× (3.57–3.65) | 3.93× (3.88–3.98) | 1.45× (1.43–1.47) | 1.69× (1.67–1.71) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.30–1.37) | 2.73× (2.67–2.79) | 2.30× (2.23–2.36) | 3.20× (3.09–3.30) | 2.74× (2.67–2.81) | 3.16× (3.08–3.24) | 3.43× (3.35–3.52) | 1.27× (1.24–1.30) | 1.48× (1.44–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.57–1.63) | 2.58× (2.50–2.66) | 1.98× (1.93–2.02) | 2.83× (2.77–2.90) | 2.65× (2.58–2.72) | 2.27× (2.22–2.32) | 2.47× (2.41–2.53) | 0.91× (0.89–0.93) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.70) | 2.68× (2.66–2.71) | 3.54× (3.47–3.60) | 2.80× (2.76–2.84) | 6.04× (5.94–6.14) | 2.28× (2.26–2.31) | 2.12× (2.09–2.15) | 0.99× (0.98–1.01) | 0.98× (0.94–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.62) | 2.45× (2.35–2.55) | 4.18× (4.09–4.28) | 2.67× (2.62–2.72) | 5.59× (5.49–5.68) | 2.10× (2.05–2.15) | 1.95× (1.90–1.99) | 0.91× (0.89–0.93) | 0.90× (0.86–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.61) | 2.67× (2.60–2.74) | 4.27× (4.18–4.36) | 2.73× (2.69–2.78) | 5.69× (5.56–5.82) | 2.15× (2.10–2.19) | 1.99× (1.95–2.04) | 0.93× (0.91–0.96) | 0.92× (0.88–0.96) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.92× (1.89–1.96) | 3.94× (3.88–4.00) | 4.27× (4.21–4.34) | 4.32× (4.29–4.35) | 4.52× (4.47–4.57) | 5.06× (5.00–5.13) | 4.82× (4.75–4.88) | 1.51× (1.49–1.53) | 1.78× (1.77–1.79) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.70× (1.67–1.72) | 3.38× (3.30–3.47) | 3.66× (3.60–3.72) | 4.03× (3.93–4.14) | 3.74× (3.68–3.79) | 4.05× (3.99–4.12) | 3.85× (3.79–3.92) | 1.20× (1.18–1.23) | 1.43× (1.40–1.45) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.72× (1.66–1.79) | 2.78× (2.67–2.90) | 3.03× (2.90–3.17) | 3.27× (3.16–3.38) | 3.20× (3.02–3.40) | 2.66× (2.60–2.71) | 2.53× (2.47–2.59) | 0.79× (0.76–0.82) | 0.94× (0.91–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.30× (0.28–0.32) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.93× (10.92–10.95) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.54× (0.50–0.59) | 0.69× (0.66–0.72) | 0.57× (0.55–0.59) | 5.44× (5.26–5.64) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.31–0.35) | 0.57× (0.50–0.65) | 0.59× (0.56–0.62) | 0.52× (0.50–0.54) | 3.42× (3.25–3.59) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.83) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.93–7.99) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.70× (0.65–0.74) | 1.61× (1.58–1.64) | 0.81× (0.79–0.83) | 5.84× (5.74–5.95) | 0.47× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.73× (0.67–0.80) | 1.15× (1.11–1.20) | 0.71× (0.68–0.74) | 3.38× (3.25–3.52) | 0.26× (0.25–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.13–5.16) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.38–0.43) | 0.91× (0.89–0.93) | 0.72× (0.68–0.75) | 3.15× (3.08–3.23) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.65) | 0.46× (0.45–0.47) | 0.86× (0.84–0.88) | 0.82× (0.80–0.84) | 2.53× (2.47–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.83× (0.82–0.83) | 5.17× (5.17–5.18) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.39× (0.37–0.41) | 0.86× (0.82–0.89) | 0.72× (0.67–0.76) | 2.93× (2.81–3.06) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.47–0.52) | 0.81× (0.79–0.82) | 0.80× (0.78–0.82) | 2.31× (2.29–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.77× (0.76–0.77) | 5.34× (5.32–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.28× (5.25–5.30) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.31× (5.30–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.66–0.67) | 6.38× (6.37–6.40) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.40× (0.36–0.44) | 1.05× (1.03–1.07) | 0.68× (0.67–0.69) | 3.35× (3.29–3.40) | 0.21× (0.20–0.21) | 0.22× (0.22–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.49× (0.47–0.51) | 0.49× (0.44–0.54) | 0.90× (0.87–0.94) | 0.74× (0.72–0.77) | 2.29× (2.21–2.38) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 412 | 2.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 701 | 2.67 | 1.10× (1.09–1.11) | 85% | 1.03× (1.02–1.05) | 1.06× (1.05–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 849 | 4.13 | 1.70× (1.67–1.72) | 52% | 1.28× (1.26–1.31) | 1.71× (1.68–1.74) |

**Best:** mibz2 at 4 threads (849 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (283 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.21, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.67 | 0.02 | 0.08 | 0.247 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.5% |
| MTF + RLE2 | 1.52 | 16.7% |
| T-vector build | 1.54 | 16.9% |
| walk (the one each build uses) | 4.38 | 48.1% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.8% |
| **total** | 9.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.302 | (in its total) | 2.64× (2.63–2.65) |  |
| walk | 1.294 | 2.662 | (in its total) | 2.06× (2.04–2.07) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.057 | (in its total) | 1.18× (1.17–1.19) |  |
| inverse BWT, total | 1.836 | 4.021 | 3.297 | 2.19× (2.18–2.20) | 1.80× (1.79–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m24s |
| 4 threads | 35m47s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m14s ± 3m25s |
| the scanner alone (its serial part) | 14m07s ± 42.1s |
| the workers alone, over 4 | 24m13s ± 3m25s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 3.18 ns a byte of output at 4 threads (CPU, summed over them), 1.96 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,633,100 | 13,390,258 | \`b7ebdafdab81f6ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
