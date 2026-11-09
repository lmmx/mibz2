# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-271/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`cbe46e4`](<https://github.com/lmmx/mibz2/commit/cbe46e4423b4c38dba1596fe828849609e682f49>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42543641882>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.662 s | 317 | 10.3 | 2.30× (2.29–2.32) | 5.30× (5.27–5.34) | 5.47× (5.42–5.53) | 5.90× (5.85–5.95) | 5.30× (5.25–5.36) | 5.69× (5.63–5.75) | 5.34× (5.30–5.38) | 1.73× (1.71–1.74) | 1.99× (1.97–2.00) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.373 s | 562 | 11.1 | 2.29× (2.23–2.36) | 4.98× (4.87–5.11) | 5.68× (5.45–5.91) | 5.52× (5.36–5.69) | 5.01× (4.86–5.16) | 10.05× (9.75–10.35) | 9.43× (9.18–9.68) | 3.05× (2.98–3.13) | 3.51× (3.41–3.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.299 s | 701 | 17.4 | 2.34× (2.28–2.39) | 3.63× (3.58–3.69) | 4.52× (4.41–4.64) | 4.18× (3.92–4.46) | 4.18× (4.07–4.30) | 12.41× (12.01–12.83) | 11.65× (11.27–12.05) | 3.77× (3.67–3.87) | 4.33× (4.22–4.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 978 | 3.6 | 0.60× (0.59–0.60) | 3.73× (3.53–3.93) | 4.45× (4.33–4.57) | 3.56× (3.53–3.60) | 6.15× (6.09–6.22) | 2.84× (2.79–2.88) | 2.65× (2.62–2.68) | 1.02× (0.99–1.04) | 1.50× (1.49–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 973 | 3.7 | 0.56× (0.52–0.59) | 2.90× (2.72–3.08) | 5.99× (5.68–6.32) | 3.46× (3.26–3.67) | 5.92× (5.57–6.29) | 2.73× (2.58–2.90) | 2.56× (2.41–2.72) | 0.98× (0.92–1.04) | 1.45× (1.36–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 952 | 3.7 | 0.57× (0.56–0.58) | 2.96× (2.90–3.02) | 6.13× (5.96–6.31) | 3.51× (3.44–3.57) | 6.05× (5.93–6.18) | 2.78× (2.71–2.84) | 2.60× (2.55–2.65) | 0.99× (0.97–1.02) | 1.47× (1.44–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 546 | 6.3 | 2.43× (2.41–2.46) | 2.82× (2.79–2.85) | 3.62× (3.57–3.66) | 3.57× (3.53–3.62) | 3.86× (3.82–3.91) | 2.33× (2.31–2.36) | 1.97× (1.95–1.99) | 1.89× (1.88–1.91) | 2.81× (2.78–2.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 790 | 7.7 | 1.96× (1.83–2.08) | 2.68× (2.39–3.02) | 3.75× (3.37–4.17) | 3.05× (2.79–3.32) | 4.33× (4.03–4.65) | 3.35× (3.12–3.60) | 2.84× (2.64–3.05) | 2.72× (2.54–2.91) | 4.04× (3.76–4.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 727 | 15.4 | 1.81× (1.75–1.87) | 1.89× (1.76–2.04) | 2.83× (2.72–2.95) | 2.06× (1.98–2.14) | 3.72× (3.59–3.85) | 3.16× (3.08–3.25) | 2.68× (2.61–2.75) | 2.57× (2.50–2.64) | 3.81× (3.71–3.91) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 62 | 52.8 | 1.36× (1.35–1.37) | 2.76× (2.73–2.79) | 2.51× (2.47–2.54) | 2.55× (2.52–2.58) | 2.93× (2.88–2.99) | 3.17× (3.15–3.20) | 3.21× (3.18–3.23) | 1.26× (1.25–1.27) | 1.44× (1.42–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 61.3 | 1.27× (1.22–1.32) | 2.47× (2.41–2.54) | 2.43× (2.35–2.52) | 3.13× (3.02–3.24) | 2.91× (2.86–2.97) | 5.01× (4.92–5.10) | 5.05× (4.96–5.15) | 1.99× (1.96–2.03) | 2.26× (2.22–2.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 83.9 | 1.30× (1.27–1.33) | 2.15× (2.10–2.20) | 2.08× (2.02–2.15) | 2.55× (2.49–2.62) | 2.84× (2.80–2.88) | 5.94× (5.83–6.05) | 6.00× (5.88–6.12) | 2.36× (2.32–2.41) | 2.69× (2.64–2.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.2 | 1.45× (1.43–1.47) | 3.08× (3.04–3.13) | 2.58× (2.56–2.61) | 2.71× (2.67–2.74) | 3.08× (3.04–3.12) | 3.78× (3.74–3.82) | 3.91× (3.86–3.95) | 1.44× (1.43–1.45) | 1.70× (1.69–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 56.1 | 1.42× (1.25–1.62) | 2.78× (2.63–2.95) | 2.49× (2.43–2.56) | 3.47× (3.32–3.62) | 3.07× (2.96–3.19) | 6.17× (5.96–6.38) | 6.38× (6.15–6.62) | 2.35× (2.27–2.43) | 2.78× (2.68–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 81.8 | 1.37× (1.33–1.40) | 2.19× (2.14–2.24) | 2.17× (2.12–2.22) | 2.92× (2.86–2.98) | 2.74× (2.69–2.79) | 6.93× (6.80–7.06) | 7.17× (7.04–7.30) | 2.64× (2.60–2.68) | 3.12× (3.07–3.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 5.1 | 0.66× (0.65–0.67) | 3.06× (2.90–3.23) | 4.37× (4.30–4.45) | 3.10× (3.06–3.14) | 6.78× (6.68–6.88) | 2.45× (2.41–2.49) | 2.30× (2.27–2.34) | 0.98× (0.97–0.98) | 1.51× (1.48–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.6 | 0.54× (0.53–0.56) | 2.76× (2.66–2.86) | 4.79× (4.57–5.03) | 2.88× (2.77–2.99) | 6.21× (5.98–6.44) | 2.25× (2.17–2.33) | 2.12× (2.04–2.20) | 0.90× (0.86–0.93) | 1.39× (1.34–1.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 683 | 5.4 | 0.56× (0.55–0.57) | 2.82× (2.77–2.87) | 4.82× (4.74–4.91) | 2.93× (2.90–2.97) | 6.34× (6.25–6.43) | 2.29× (2.25–2.33) | 2.16× (2.12–2.19) | 0.91× (0.89–0.93) | 1.41× (1.39–1.43) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 0.089 s | 140 | 23.7 | 1.43× (1.42–1.45) | 3.04× (2.99–3.09) | 3.29× (3.24–3.34) | 3.28× (3.24–3.32) | 3.47× (3.43–3.51) | 3.71× (3.66–3.76) | 3.47× (3.43–3.51) | 1.13× (1.11–1.15) | 1.37× (1.35–1.39) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 0.058 s | 215 | 28.4 | 1.27× (1.20–1.34) | 2.57× (2.49–2.66) | 3.19× (3.02–3.36) | 3.25× (3.18–3.31) | 3.37× (3.30–3.45) | 5.78× (5.65–5.92) | 5.41× (5.29–5.53) | 1.76× (1.71–1.81) | 2.13× (2.07–2.20) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 0.047 s | 263 | 39.3 | 1.30× (1.25–1.35) | 2.18× (2.08–2.29) | 2.80× (2.70–2.90) | 2.91× (2.85–2.97) | 3.24× (3.15–3.32) | 7.05× (6.90–7.21) | 6.59× (6.47–6.72) | 2.14× (2.10–2.19) | 2.60× (2.53–2.68) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.30× (2.28–2.31) | 5.28× (5.24–5.32) | 5.45× (5.40–5.51) | 5.88× (5.84–5.93) | 5.29× (5.24–5.34) | 5.67× (5.61–5.74) | 5.33× (5.29–5.36) | 1.72× (1.71–1.74) | 1.98× (1.96–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.22–2.25) | 5.02× (4.97–5.07) | 5.29× (5.25–5.33) | 5.75× (5.72–5.79) | 4.97× (4.93–5.00) | 5.28× (5.23–5.33) | 4.96× (4.91–5.00) | 1.61× (1.58–1.63) | 1.84× (1.82–1.85) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.27× (2.22–2.32) | 3.84× (3.72–3.96) | 4.13× (3.98–4.28) | 4.33× (4.17–4.50) | 3.85× (3.70–4.00) | 3.37× (3.32–3.42) | 3.16× (3.12–3.21) | 1.02× (1.00–1.05) | 1.17× (1.15–1.20) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.26× (3.21–3.30) | 4.06× (4.01–4.11) | 3.33× (3.29–3.37) | 5.69× (5.62–5.76) | 2.67× (2.63–2.71) | 2.50× (2.48–2.53) | 1.01× (1.00–1.03) | 1.08× (1.04–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.57–0.64) | 2.99× (2.83–3.17) | 5.27× (5.00–5.55) | 3.22× (3.05–3.40) | 5.42× (5.14–5.72) | 2.54× (2.41–2.68) | 2.39× (2.26–2.52) | 0.97× (0.91–1.02) | 1.03× (0.95–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.39× (3.30–3.47) | 5.48× (5.34–5.63) | 3.32× (3.26–3.39) | 5.61× (5.49–5.73) | 2.63× (2.57–2.69) | 2.47× (2.42–2.52) | 1.00× (0.97–1.02) | 1.06× (1.03–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.36× (2.33–2.38) | 2.61× (2.58–2.64) | 3.28× (3.25–3.31) | 3.44× (3.40–3.48) | 3.71× (3.67–3.76) | 2.26× (2.24–2.29) | 1.92× (1.90–1.94) | 1.85× (1.83–1.86) | 2.55× (2.51–2.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.06–2.12) | 2.39× (2.35–2.44) | 2.91× (2.84–2.98) | 2.96× (2.91–3.00) | 3.19× (3.12–3.26) | 1.84× (1.81–1.87) | 1.56× (1.53–1.59) | 1.50× (1.48–1.52) | 2.07× (2.04–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.89× (1.81–1.97) | 1.63× (1.56–1.70) | 2.05× (1.97–2.13) | 1.94× (1.86–2.02) | 1.95× (1.89–2.01) | 0.93× (0.91–0.96) | 0.79× (0.77–0.82) | 0.76× (0.74–0.79) | 1.05× (1.02–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.69× (2.67–2.72) | 2.43× (2.39–2.47) | 2.53× (2.50–2.56) | 2.91× (2.85–2.96) | 3.14× (3.11–3.17) | 3.17× (3.15–3.20) | 1.26× (1.25–1.27) | 1.40× (1.38–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.23–1.33) | 2.47× (2.45–2.50) | 2.24× (2.17–2.31) | 2.96× (2.89–3.03) | 2.58× (2.54–2.61) | 2.74× (2.71–2.78) | 2.77× (2.73–2.81) | 1.10× (1.08–1.11) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.42–1.49) | 2.28× (2.20–2.35) | 1.84× (1.79–1.89) | 2.54× (2.47–2.61) | 2.43× (2.36–2.49) | 1.99× (1.95–2.03) | 2.01× (1.97–2.05) | 0.80× (0.78–0.81) | 0.89× (0.87–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.47) | 3.01× (2.97–3.06) | 2.53× (2.50–2.56) | 2.68× (2.65–2.72) | 3.05× (3.02–3.09) | 3.74× (3.70–3.78) | 3.87× (3.82–3.91) | 1.43× (1.42–1.44) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.36) | 2.77× (2.74–2.80) | 2.32× (2.29–2.34) | 3.28× (3.20–3.36) | 2.75× (2.71–2.79) | 3.29× (3.26–3.33) | 3.41× (3.37–3.44) | 1.26× (1.25–1.27) | 1.46× (1.45–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.48–1.57) | 2.48× (2.43–2.52) | 1.90× (1.87–1.94) | 2.72× (2.68–2.76) | 2.47× (2.44–2.50) | 2.26× (2.23–2.29) | 2.34× (2.31–2.36) | 0.87× (0.86–0.88) | 1.00× (0.99–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.70× (2.66–2.73) | 3.53× (3.49–3.57) | 2.83× (2.79–2.86) | 6.04× (5.96–6.12) | 2.26× (2.22–2.29) | 2.13× (2.11–2.16) | 0.98× (0.97–0.98) | 0.99× (0.96–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.59–0.62) | 2.43× (2.35–2.50) | 4.14× (4.02–4.27) | 2.67× (2.58–2.76) | 5.51× (5.34–5.68) | 2.06× (1.99–2.12) | 1.95× (1.89–2.01) | 0.89× (0.86–0.92) | 0.90× (0.87–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.59–0.62) | 2.73× (2.69–2.76) | 4.25× (4.21–4.30) | 2.79× (2.76–2.81) | 5.72× (5.66–5.78) | 2.13× (2.10–2.16) | 2.02× (1.99–2.04) | 0.92× (0.91–0.94) | 0.93× (0.91–0.96) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 1.42× (1.41–1.44) | 2.92× (2.89–2.96) | 3.19× (3.15–3.22) | 3.23× (3.19–3.27) | 3.42× (3.38–3.45) | 3.65× (3.60–3.70) | 3.41× (3.37–3.46) | 1.12× (1.11–1.15) | 1.31× (1.30–1.32) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 1.29× (1.27–1.32) | 2.58× (2.51–2.65) | 2.85× (2.81–2.89) | 3.08× (3.04–3.11) | 2.95× (2.91–2.98) | 3.08× (3.05–3.12) | 2.88× (2.85–2.92) | 0.95× (0.93–0.97) | 1.10× (1.09–1.12) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 1.50× (1.46–1.54) | 2.30× (2.23–2.36) | 2.62× (2.54–2.70) | 2.72× (2.67–2.77) | 2.64× (2.59–2.69) | 2.23× (2.20–2.26) | 2.09× (2.06–2.12) | 0.69× (0.67–0.71) | 0.80× (0.79–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.33) | 0.32× (0.32–0.33) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.17× (12.15–12.20) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.55× (0.47–0.64) | 0.81× (0.79–0.82) | 0.67× (0.66–0.68) | 6.39× (6.33–6.44) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.75× (0.66–0.86) | 0.70× (0.69–0.71) | 0.61× (0.59–0.63) | 4.10× (4.08–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.32× (4.31–4.32) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.83–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.66) | 8.14× (8.12–8.16) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.72× (0.69–0.75) | 1.63× (1.60–1.66) | 0.80× (0.78–0.82) | 5.89× (5.78–6.00) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.63–0.66) | 0.74× (0.68–0.80) | 1.18× (1.16–1.21) | 0.77× (0.75–0.78) | 3.46× (3.41–3.51) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.74× (0.74–0.74) | 5.26× (5.25–5.26) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.40× (0.37–0.43) | 0.90× (0.87–0.92) | 0.69× (0.66–0.71) | 3.08× (3.01–3.16) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.83× (0.81–0.85) | 2.56× (2.51–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.39) | 0.85× (0.83–0.87) | 0.70× (0.67–0.73) | 2.92× (2.84–3.00) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.61) | 0.50× (0.47–0.54) | 0.80× (0.79–0.82) | 0.80× (0.76–0.83) | 2.31× (2.27–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.36× (0.36–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.44× (5.43–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.39–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.42× (5.40–5.43) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.67× (0.67–0.68) | 6.75× (6.74–6.77) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 33 MB | 0.48× (0.47–0.50) | 0.41× (0.38–0.46) | 1.20× (1.14–1.26) | 0.72× (0.69–0.76) | 3.67× (3.55–3.78) | 0.22× (0.21–0.23) | 0.23× (0.22–0.24) | 0.36× (0.35–0.37) | 0.18× (0.17–0.18) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 50 MB | 0.54× (0.52–0.55) | 0.49× (0.44–0.54) | 1.01× (0.98–1.04) | 0.79× (0.75–0.83) | 2.58× (2.51–2.65) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 317 | 3.17 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 562 | 3.42 | 1.07× (1.06–1.08) | 88% | 1.02× (1.01–1.03) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 701 | 5.36 | 1.68× (1.65–1.72) | 55% | 1.22× (1.20–1.25) | 1.66× (1.64–1.69) |

**Best:** mibz2 at 4 threads (701 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.07, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.91 | 0.02 | 0.09 | 0.299 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 11.5% |
| MTF + RLE2 | 1.58 | 16.2% |
| T-vector build | 2.12 | 21.7% |
| walk (the one each build uses) | 4.40 | 44.9% |
| of which: segment assembly | 0.18 | 1.9% |
| RLE1 | 0.29 | 3.0% |
| CRC | 0.26 | 2.6% |
| **total** | 9.80 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.674 | 1.299 | (in its total) | 1.93× (1.92–1.93) |  |
| walk | 1.300 | 2.679 | (in its total) | 2.06× (2.04–2.07) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.052 | 0.059 | (in its total) | 1.11× (1.07–1.14) |  |
| inverse BWT, total | 2.027 | 4.036 | 3.387 | 1.99× (1.98–2.00) | 1.66× (1.65–1.68) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h36m ± 11m25s |
| 4 threads | 44m18s ± 6m24s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m55s ± 4m39s |
| the scanner alone (its serial part) | 15m25s ± 32.2s |
| the workers alone, over 4 | 32m55s ± 4m39s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.32 ns a byte of output at 4 threads (CPU, summed over them), 2.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,094,880 | 12,489,382 | \`0cafc9c0ea86fcad…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
