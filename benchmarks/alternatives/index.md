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

The latest record: [`bc89940`](<https://github.com/lmmx/mibz2/commit/bc89940090777c48b30ee172200099943c14165f>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42124071330>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.669 s | 314 | 9.3 | 2.31× (2.28–2.34) | 5.31× (5.26–5.37) | 5.46× (5.40–5.53) | 5.86× (5.81–5.92) | 5.30× (5.25–5.36) | 5.64× (5.59–5.69) | 5.38× (5.33–5.42) | 1.71× (1.69–1.73) | 1.97× (1.95–2.00) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.376 s | 558 | 10.1 | 2.29× (2.16–2.43) | 4.97× (4.83–5.13) | 5.56× (5.33–5.81) | 5.56× (5.39–5.73) | 4.99× (4.75–5.25) | 9.75× (9.33–10.18) | 9.29× (8.90–9.70) | 2.96× (2.82–3.10) | 3.41× (3.25–3.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.309 s | 679 | 16.4 | 2.36× (2.31–2.41) | 3.71× (3.64–3.77) | 4.54× (4.46–4.63) | 4.05× (3.96–4.14) | 4.11× (4.07–4.14) | 12.12× (11.94–12.31) | 11.56× (11.36–11.75) | 3.68× (3.62–3.74) | 4.24× (4.17–4.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 950 | 3.4 | 0.59× (0.59–0.60) | 3.74× (3.58–3.91) | 4.45× (4.32–4.58) | 3.49× (3.44–3.53) | 6.09× (6.00–6.18) | 2.76× (2.73–2.80) | 2.59× (2.56–2.63) | 1.02× (1.00–1.04) | 1.47× (1.45–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 911 | 3.5 | 0.56× (0.55–0.57) | 2.84× (2.80–2.89) | 6.02× (5.93–6.12) | 3.41× (3.35–3.46) | 5.91× (5.82–6.01) | 2.68× (2.63–2.72) | 2.51× (2.47–2.55) | 0.98× (0.96–1.01) | 1.42× (1.40–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 917 | 3.5 | 0.56× (0.55–0.57) | 2.84× (2.80–2.88) | 6.05× (5.87–6.23) | 3.38× (3.33–3.43) | 5.89× (5.79–5.99) | 2.66× (2.62–2.70) | 2.50× (2.46–2.54) | 0.98× (0.95–1.01) | 1.42× (1.40–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 535 | 5.8 | 2.61× (2.53–2.70) | 2.80× (2.77–2.82) | 3.63× (3.51–3.76) | 3.54× (3.51–3.58) | 3.90× (3.85–3.94) | 2.30× (2.28–2.32) | 1.95× (1.93–1.97) | 1.87× (1.84–1.90) | 2.78× (2.75–2.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 781 | 7.1 | 1.99× (1.81–2.18) | 2.48× (2.23–2.76) | 3.60× (3.28–3.95) | 2.97× (2.71–3.27) | 4.27× (3.91–4.66) | 3.28× (3.01–3.58) | 2.78× (2.55–3.02) | 2.67× (2.44–2.91) | 3.96× (3.64–4.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 717 | 14.3 | 1.72× (1.68–1.76) | 1.85× (1.74–1.96) | 2.78× (2.66–2.91) | 2.04× (1.95–2.14) | 3.71× (3.60–3.83) | 3.09× (3.03–3.15) | 2.61× (2.56–2.67) | 2.51× (2.45–2.57) | 3.73× (3.66–3.80) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.136 s | 59 | 50.4 | 1.31× (1.29–1.33) | 2.63× (2.61–2.65) | 2.37× (2.35–2.40) | 2.43× (2.40–2.46) | 2.84× (2.74–2.94) | 3.00× (2.97–3.04) | 3.12× (3.08–3.16) | 1.21× (1.20–1.23) | 1.36× (1.33–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 97 | 56.7 | 1.23× (1.17–1.28) | 2.50× (2.40–2.60) | 2.31× (2.23–2.39) | 2.97× (2.82–3.12) | 2.87× (2.74–3.01) | 4.84× (4.61–5.08) | 5.03× (4.82–5.25) | 1.96× (1.86–2.06) | 2.20× (2.09–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 76.8 | 1.30× (1.28–1.33) | 2.16× (2.09–2.23) | 2.08× (2.05–2.12) | 2.68× (2.50–2.87) | 2.86× (2.81–2.91) | 5.88× (5.78–5.98) | 6.11× (6.04–6.19) | 2.38× (2.34–2.41) | 2.67× (2.60–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.149 s | 64 | 45.9 | 1.41× (1.39–1.43) | 3.00× (2.93–3.07) | 2.52× (2.47–2.56) | 2.59× (2.55–2.64) | 3.04× (2.92–3.16) | 3.71× (3.61–3.82) | 3.92× (3.86–3.98) | 1.43× (1.40–1.46) | 1.69× (1.66–1.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 51.6 | 1.28× (1.22–1.34) | 2.70× (2.55–2.85) | 2.43× (2.32–2.55) | 3.38× (3.18–3.58) | 2.98× (2.85–3.12) | 6.04× (5.70–6.39) | 6.37× (6.06–6.69) | 2.33× (2.21–2.45) | 2.74× (2.62–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 75.0 | 1.36× (1.33–1.39) | 2.27× (2.20–2.34) | 2.24× (2.18–2.30) | 2.92× (2.86–2.98) | 2.82× (2.76–2.89) | 7.05× (6.85–7.25) | 7.43× (7.33–7.55) | 2.72× (2.66–2.78) | 3.20× (3.16–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 707 | 4.7 | 0.65× (0.65–0.66) | 3.20× (2.95–3.47) | 4.28× (4.24–4.32) | 3.04× (3.02–3.07) | 6.70× (6.62–6.79) | 2.40× (2.38–2.41) | 2.27× (2.25–2.28) | 0.98× (0.97–0.99) | 1.48× (1.46–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 667 | 5.1 | 0.55× (0.54–0.56) | 2.76× (2.72–2.79) | 4.92× (4.72–5.13) | 2.89× (2.85–2.92) | 6.30× (6.20–6.40) | 2.25× (2.23–2.28) | 2.13× (2.10–2.16) | 0.92× (0.91–0.93) | 1.39× (1.37–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 669 | 5.0 | 0.55× (0.54–0.56) | 2.77× (2.73–2.81) | 4.86× (4.66–5.08) | 2.90× (2.85–2.94) | 6.35× (6.26–6.44) | 2.25× (2.22–2.29) | 2.13× (2.10–2.17) | 0.92× (0.90–0.94) | 1.39× (1.37–1.41) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 0.087 s | 143 | 20.9 | 1.48× (1.47–1.50) | 3.09× (3.04–3.14) | 3.32× (3.28–3.36) | 3.30× (3.26–3.33) | 3.51× (3.47–3.55) | 3.68× (3.64–3.71) | 3.67× (3.64–3.69) | 1.16× (1.14–1.18) | 1.42× (1.40–1.43) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 0.055 s | 225 | 24.9 | 1.30× (1.25–1.35) | 2.70× (2.48–2.94) | 3.19× (3.04–3.36) | 3.41× (3.25–3.58) | 3.47× (3.38–3.57) | 5.72× (5.54–5.90) | 5.71× (5.53–5.89) | 1.80× (1.73–1.88) | 2.21× (2.14–2.28) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 0.045 s | 275 | 35.1 | 1.37× (1.34–1.40) | 2.14× (2.07–2.21) | 2.86× (2.76–2.95) | 3.04× (2.97–3.12) | 3.33× (3.26–3.40) | 7.02× (6.85–7.20) | 7.01× (6.86–7.16) | 2.21× (2.14–2.29) | 2.71× (2.64–2.77) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.31× (2.28–2.33) | 5.29× (5.24–5.35) | 5.44× (5.38–5.51) | 5.85× (5.80–5.90) | 5.29× (5.23–5.35) | 5.62× (5.58–5.67) | 5.36× (5.32–5.40) | 1.71× (1.69–1.73) | 1.96× (1.94–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.22× (2.19–2.25) | 5.06× (4.98–5.14) | 5.27× (5.25–5.30) | 5.76× (5.69–5.84) | 4.98× (4.90–5.07) | 5.16× (5.13–5.20) | 4.92× (4.88–4.97) | 1.57× (1.55–1.59) | 1.80× (1.78–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.20× (2.17–2.24) | 3.82× (3.77–3.88) | 4.05× (3.99–4.11) | 4.17× (4.11–4.22) | 3.74× (3.69–3.79) | 3.21× (3.18–3.23) | 3.06× (3.02–3.09) | 0.97× (0.97–0.98) | 1.12× (1.11–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.63) | 3.20× (3.17–3.24) | 4.03× (3.93–4.14) | 3.26× (3.22–3.30) | 5.63× (5.54–5.72) | 2.60× (2.57–2.64) | 2.45× (2.42–2.48) | 1.02× (1.00–1.03) | 1.07× (1.03–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 2.95× (2.90–2.99) | 5.23× (5.15–5.31) | 3.17× (3.12–3.23) | 5.41× (5.33–5.49) | 2.49× (2.45–2.53) | 2.35× (2.31–2.38) | 0.97× (0.95–0.99) | 1.03× (0.98–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.27× (3.21–3.32) | 5.29× (5.16–5.43) | 3.20× (3.15–3.25) | 5.45× (5.36–5.54) | 2.52× (2.48–2.56) | 2.37× (2.33–2.40) | 0.98× (0.96–1.01) | 1.04× (0.99–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.45–2.61) | 2.60× (2.57–2.62) | 3.33× (3.21–3.45) | 3.41× (3.37–3.44) | 3.74× (3.70–3.79) | 2.23× (2.21–2.26) | 1.90× (1.88–1.92) | 1.83× (1.80–1.86) | 2.51× (2.48–2.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.05–2.13) | 2.32× (2.25–2.39) | 2.87× (2.82–2.92) | 2.89× (2.84–2.95) | 3.13× (3.08–3.19) | 1.80× (1.76–1.83) | 1.53× (1.50–1.56) | 1.47× (1.44–1.49) | 2.02× (1.98–2.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.80–1.93) | 1.60× (1.54–1.67) | 2.02× (1.95–2.09) | 1.88× (1.80–1.97) | 1.91× (1.86–1.97) | 0.91× (0.88–0.94) | 0.77× (0.74–0.80) | 0.74× (0.71–0.77) | 1.02× (0.99–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.30× (1.29–1.32) | 2.58× (2.56–2.60) | 2.30× (2.28–2.33) | 2.41× (2.38–2.44) | 2.81× (2.72–2.91) | 2.97× (2.94–3.01) | 3.09× (3.05–3.13) | 1.21× (1.20–1.22) | 1.33× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.25) | 2.42× (2.38–2.46) | 2.14× (2.11–2.16) | 2.85× (2.82–2.88) | 2.53× (2.46–2.60) | 2.64× (2.60–2.68) | 2.74× (2.71–2.77) | 1.07× (1.06–1.09) | 1.18× (1.16–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.40–1.48) | 2.29× (2.23–2.34) | 1.85× (1.82–1.88) | 2.54× (2.49–2.58) | 2.42× (2.36–2.48) | 1.95× (1.92–1.97) | 2.02× (2.00–2.05) | 0.79× (0.78–0.80) | 0.87× (0.86–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.38–1.43) | 2.93× (2.87–2.99) | 2.46× (2.41–2.50) | 2.57× (2.53–2.61) | 3.01× (2.90–3.13) | 3.68× (3.58–3.78) | 3.88× (3.82–3.94) | 1.43× (1.40–1.45) | 1.64× (1.61–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.31–1.33) | 2.75× (2.74–2.77) | 2.28× (2.26–2.30) | 3.20× (3.14–3.25) | 2.72× (2.70–2.74) | 3.27× (3.22–3.33) | 3.45× (3.42–3.48) | 1.27× (1.26–1.28) | 1.46× (1.45–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.50–1.56) | 2.51× (2.46–2.55) | 1.91× (1.88–1.94) | 2.73× (2.68–2.78) | 2.49× (2.45–2.54) | 2.27× (2.21–2.33) | 2.40× (2.37–2.42) | 0.88× (0.87–0.90) | 1.01× (1.00–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.70–0.71) | 2.68× (2.65–2.70) | 3.50× (3.48–3.52) | 2.80× (2.78–2.81) | 6.01× (5.94–6.08) | 2.23× (2.21–2.24) | 2.12× (2.11–2.13) | 0.99× (0.98–1.00) | 0.99× (0.95–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.61–0.62) | 2.44× (2.41–2.46) | 4.20× (4.15–4.25) | 2.67× (2.65–2.70) | 5.57× (5.49–5.65) | 2.06× (2.04–2.08) | 1.96× (1.94–1.98) | 0.91× (0.90–0.92) | 0.92× (0.88–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.73× (2.68–2.78) | 4.23× (4.14–4.31) | 2.75× (2.70–2.79) | 5.74× (5.65–5.83) | 2.10× (2.07–2.14) | 2.00× (1.97–2.04) | 0.93× (0.91–0.95) | 0.94× (0.90–0.98) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 1.47× (1.45–1.48) | 2.97× (2.93–3.01) | 3.22× (3.19–3.25) | 3.24× (3.21–3.28) | 3.45× (3.42–3.48) | 3.61× (3.58–3.64) | 3.60× (3.58–3.63) | 1.15× (1.14–1.17) | 1.34× (1.33–1.36) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 1.36× (1.34–1.37) | 2.61× (2.56–2.66) | 2.89× (2.86–2.93) | 3.11× (3.07–3.15) | 2.99× (2.93–3.05) | 3.03× (3.00–3.06) | 3.03× (3.00–3.06) | 0.97× (0.95–0.99) | 1.13× (1.11–1.14) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 1.56× (1.50–1.62) | 2.31× (2.23–2.39) | 2.55× (2.48–2.62) | 2.62× (2.54–2.70) | 2.58× (2.53–2.62) | 2.12× (2.09–2.16) | 2.12× (2.09–2.15) | 0.68× (0.66–0.70) | 0.79× (0.77–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.34) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.24× (12.23–12.26) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.38–0.39) | 0.57× (0.49–0.67) | 0.81× (0.80–0.82) | 0.67× (0.66–0.68) | 6.41× (6.36–6.46) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.85× (0.77–0.94) | 0.71× (0.70–0.72) | 0.61× (0.60–0.62) | 4.11× (4.09–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.83× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.81× (1.81–1.81) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.18× (8.16–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.72× (0.67–0.78) | 1.66× (1.63–1.69) | 0.81× (0.80–0.83) | 5.99× (5.87–6.11) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.64× (0.62–0.66) | 0.75× (0.70–0.81) | 1.18× (1.14–1.23) | 0.76× (0.72–0.79) | 3.42× (3.31–3.54) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.73–0.74) | 5.28× (5.28–5.29) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.48× (0.48–0.49) | 0.41× (0.39–0.43) | 0.90× (0.89–0.92) | 0.70× (0.67–0.73) | 3.11× (3.06–3.17) | 0.19× (0.19–0.19) | 0.20× (0.20–0.20) | 0.31× (0.30–0.31) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.62–0.66) | 0.47× (0.46–0.48) | 0.89× (0.87–0.90) | 0.82× (0.80–0.84) | 2.57× (2.52–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.28× (5.27–5.29) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.40× (0.39–0.42) | 0.87× (0.84–0.90) | 0.72× (0.68–0.76) | 2.97× (2.87–3.07) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.60) | 0.52× (0.50–0.53) | 0.80× (0.79–0.81) | 0.81× (0.78–0.84) | 2.30× (2.27–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.10× (1.10–1.10) | 0.77× (0.77–0.78) | 5.48× (5.47–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.66) | 0.77× (0.77–0.78) | 5.44× (5.42–5.45) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.66× (1.65–1.66) | 0.77× (0.77–0.77) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.51× (1.51–1.51) | 0.65× (0.64–0.65) | 6.79× (6.78–6.81) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 33 MB | 0.47× (0.45–0.49) | 0.39× (0.36–0.42) | 1.17× (1.12–1.22) | 0.70× (0.66–0.73) | 3.60× (3.49–3.72) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 53 MB | 0.51× (0.51–0.52) | 0.51× (0.47–0.55) | 0.96× (0.94–0.97) | 0.74× (0.70–0.78) | 2.45× (2.42–2.47) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 314 | 3.20 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 558 | 3.48 | 1.09× (1.08–1.10) | 86% | 1.04× (1.02–1.07) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 679 | 5.63 | 1.75× (1.73–1.78) | 54% | 1.27× (1.25–1.28) | 1.68× (1.65–1.71) |

**Best:** mibz2 at 4 threads (679 MB/s); lbzip2 at 4 threads (182 MB/s); pbz2 at 4 threads (285 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.95 | 0.02 | 0.10 | 0.309 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2911 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 11.9% |
| MTF + RLE2 | 1.36 | 15.8% |
| T-vector build | 1.92 | 22.3% |
| walk (the one each build uses) | 3.82 | 44.4% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.26 | 3.0% |
| CRC | 0.23 | 2.7% |
| **total** | 8.62 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.673 | 1.302 | (in its total) | 1.93× (1.92–1.95) |  |
| walk | 1.293 | 2.692 | (in its total) | 2.09× (2.07–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.058 | (in its total) | 1.19× (1.16–1.21) |  |
| inverse BWT, total | 2.016 | 4.051 | 3.352 | 2.01× (2.00–2.03) | 1.66× (1.65–1.67) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h37m ± 11m30s |
| 4 threads | 45m20s ± 6m23s |
| 4 threads, the sample's overlap gap as a fixed cost | 34m46s ± 4m53s |
| the scanner alone (its serial part) | 15m34s ± 30.2s |
| the workers alone, over 4 | 34m46s ± 4m53s |

At 4 threads the workers bound it. Per byte: the scanner 9.0 ns a byte of bz2, the workers 4.56 ns a byte of output at 4 threads (CPU, summed over them), 2.68 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,807,836 | 12,402,701 | \`8331caec1322f098…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2911.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
