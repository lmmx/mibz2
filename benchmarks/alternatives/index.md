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

The latest record: [`0ad0e9a`](<https://github.com/lmmx/mibz2/commit/0ad0e9a5fa9f97dc0563bac771b92bf44c80f5e6>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45534815847>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.490 s | 428 | 7.6 | 3.07× (3.05–3.08) | 6.99× (6.97–7.01) | 7.20× (7.16–7.24) | 7.83× (7.79–7.86) | 7.02× (6.99–7.05) | 7.67× (7.64–7.70) | 7.06× (7.02–7.09) | 2.26× (2.25–2.28) | 2.66× (2.64–2.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.285 s | 736 | 8.1 | 3.07× (2.99–3.14) | 6.30× (6.01–6.61) | 7.26× (6.81–7.75) | 7.09× (6.66–7.55) | 6.41× (6.01–6.84) | 13.09× (12.29–13.94) | 12.04× (11.31–12.82) | 3.86× (3.61–4.12) | 4.53× (4.25–4.83) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.221 s | 950 | 12.6 | 3.12× (3.07–3.16) | 4.82× (4.70–4.94) | 5.98× (5.86–6.09) | 5.49× (5.40–5.59) | 5.48× (5.41–5.56) | 16.99× (16.73–17.25) | 15.63× (15.41–15.86) | 5.01× (4.92–5.10) | 5.88× (5.81–5.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 3.6 | 0.59× (0.56–0.62) | 3.47× (3.25–3.71) | 4.43× (4.14–4.73) | 3.49× (3.31–3.67) | 6.05× (5.74–6.38) | 2.91× (2.76–3.07) | 2.60× (2.47–2.74) | 0.99× (0.93–1.04) | 1.47× (1.40–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 939 | 3.9 | 0.55× (0.54–0.55) | 2.88× (2.84–2.92) | 5.69× (5.62–5.77) | 3.43× (3.38–3.48) | 5.89× (5.79–6.00) | 2.85× (2.81–2.89) | 2.54× (2.51–2.58) | 0.96× (0.95–0.98) | 1.44× (1.42–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 917 | 4.0 | 0.54× (0.53–0.55) | 2.83× (2.80–2.86) | 5.60× (5.52–5.69) | 3.34× (3.30–3.37) | 5.77× (5.71–5.83) | 2.78× (2.75–2.80) | 2.48× (2.45–2.51) | 0.94× (0.92–0.96) | 1.41× (1.39–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 590 | 5.8 | 2.73× (2.67–2.79) | 3.07× (3.06–3.08) | 3.94× (3.92–3.96) | 3.85× (3.83–3.87) | 4.21× (4.19–4.24) | 2.59× (2.57–2.61) | 2.13× (2.12–2.14) | 2.03× (2.02–2.04) | 3.11× (2.98–3.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 918 | 7.0 | 2.36× (2.26–2.46) | 3.02× (2.81–3.25) | 4.33× (4.09–4.58) | 3.74× (3.33–4.20) | 5.02× (4.95–5.10) | 4.04× (3.96–4.12) | 3.33× (3.26–3.39) | 3.16× (3.10–3.22) | 4.85× (4.61–5.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 865 | 13.7 | 2.05× (1.97–2.13) | 2.11× (1.93–2.31) | 3.24× (3.05–3.43) | 2.44× (2.34–2.53) | 4.36× (4.18–4.55) | 3.76× (3.62–3.91) | 3.09× (2.98–3.21) | 2.94× (2.84–3.06) | 4.51× (4.33–4.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 53.4 | 1.35× (1.34–1.36) | 2.69× (2.65–2.72) | 2.44× (2.39–2.49) | 2.47× (2.45–2.49) | 2.86× (2.83–2.89) | 3.18× (3.14–3.22) | 3.12× (3.08–3.15) | 1.23× (1.22–1.25) | 1.41× (1.39–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 101 | 59.5 | 1.28× (1.21–1.36) | 2.59× (2.51–2.66) | 2.36× (2.25–2.46) | 3.03× (2.89–3.18) | 2.96× (2.80–3.12) | 5.11× (4.85–5.38) | 5.01× (4.77–5.26) | 1.98× (1.89–2.07) | 2.26× (2.15–2.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.065 s | 124 | 81.4 | 1.36× (1.34–1.39) | 2.23× (2.18–2.27) | 2.10× (2.06–2.13) | 2.74× (2.62–2.86) | 2.97× (2.93–3.01) | 6.35× (6.22–6.50) | 6.23× (6.15–6.31) | 2.47× (2.43–2.51) | 2.81× (2.77–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 68 | 48.8 | 1.47× (1.43–1.51) | 3.09× (3.07–3.11) | 2.57× (2.55–2.58) | 2.68× (2.65–2.72) | 3.12× (3.04–3.21) | 3.69× (3.68–3.70) | 3.84× (3.83–3.85) | 1.42× (1.41–1.43) | 1.71× (1.71–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 54.7 | 1.39× (1.16–1.66) | 2.76× (2.46–3.10) | 2.36× (2.16–2.59) | 3.33× (3.01–3.68) | 2.96× (2.69–3.25) | 5.79× (5.28–6.36) | 6.03× (5.50–6.61) | 2.23× (2.03–2.45) | 2.69× (2.45–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.073 s | 132 | 79.7 | 1.45× (1.42–1.49) | 2.38× (2.29–2.47) | 2.28× (2.23–2.34) | 3.13× (3.08–3.18) | 2.95× (2.92–2.97) | 7.22× (7.13–7.30) | 7.51× (7.42–7.60) | 2.77× (2.75–2.80) | 3.35× (3.31–3.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 752 | 5.0 | 0.67× (0.66–0.68) | 3.07× (3.05–3.10) | 4.50× (4.47–4.54) | 3.15× (3.12–3.17) | 6.91× (6.84–6.98) | 2.56× (2.54–2.58) | 2.35× (2.33–2.36) | 0.99× (0.98–1.00) | 1.55× (1.53–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 683 | 5.7 | 0.54× (0.53–0.55) | 2.81× (2.74–2.88) | 4.80× (4.73–4.86) | 2.89× (2.85–2.93) | 6.29× (6.19–6.39) | 2.32× (2.29–2.35) | 2.13× (2.10–2.16) | 0.90× (0.89–0.91) | 1.40× (1.38–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 664 | 5.7 | 0.53× (0.50–0.55) | 2.68× (2.56–2.81) | 4.60× (4.38–4.82) | 2.78× (2.65–2.93) | 6.06× (5.79–6.34) | 2.22× (2.12–2.33) | 2.04× (1.94–2.14) | 0.86× (0.82–0.90) | 1.34× (1.28–1.41) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.071 s | 198 | 16.9 | 2.02× (1.96–2.09) | 4.26× (4.12–4.41) | 4.62× (4.44–4.81) | 4.64× (4.49–4.81) | 4.81× (4.65–4.98) | 5.45× (5.27–5.64) | 4.95× (4.77–5.13) | 1.56× (1.51–1.61) | 1.98× (1.91–2.05) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.047 s | 297 | 21.0 | 1.63× (1.56–1.71) | 3.60× (3.42–3.78) | 4.32× (4.19–4.46) | 4.80× (4.53–5.08) | 4.41× (4.18–4.65) | 8.14× (7.83–8.46) | 7.39× (7.13–7.65) | 2.33× (2.24–2.42) | 2.96× (2.85–3.07) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.043 s | 324 | 32.4 | 1.56× (1.52–1.60) | 2.56× (2.47–2.66) | 3.60× (3.50–3.71) | 3.58× (3.50–3.67) | 3.87× (3.79–3.96) | 8.99× (8.77–9.22) | 8.16× (7.95–8.37) | 2.57× (2.50–2.64) | 3.27× (3.19–3.35) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.06× (3.04–3.07) | 6.96× (6.94–6.98) | 7.16× (7.12–7.20) | 7.80× (7.76–7.83) | 6.99× (6.97–7.02) | 7.64× (7.61–7.67) | 7.03× (7.00–7.07) | 2.26× (2.24–2.27) | 2.64× (2.63–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.99× (2.97–3.00) | 6.70× (6.67–6.73) | 7.02× (7.00–7.05) | 7.74× (7.72–7.77) | 6.64× (6.62–6.67) | 7.19× (7.16–7.23) | 6.62× (6.59–6.65) | 2.12× (2.11–2.14) | 2.49× (2.48–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.04× (2.99–3.08) | 5.09× (5.02–5.16) | 5.46× (5.39–5.53) | 5.82× (5.75–5.88) | 5.14× (5.07–5.21) | 4.65× (4.60–4.71) | 4.28× (4.23–4.33) | 1.37× (1.36–1.39) | 1.61× (1.59–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.59–0.65) | 3.20× (3.05–3.36) | 4.03× (3.82–4.26) | 3.28× (3.13–3.44) | 5.63× (5.38–5.90) | 2.75× (2.62–2.89) | 2.47× (2.36–2.59) | 0.99× (0.94–1.04) | 1.09× (1.02–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.90× (2.85–2.94) | 5.06× (4.99–5.13) | 3.13× (3.09–3.16) | 5.28× (5.19–5.37) | 2.58× (2.55–2.61) | 2.32× (2.29–2.35) | 0.93× (0.91–0.94) | 1.02× (0.96–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.56–0.57) | 3.14× (3.09–3.19) | 5.01× (4.95–5.06) | 3.09× (3.06–3.12) | 5.23× (5.19–5.28) | 2.56× (2.54–2.59) | 2.30× (2.28–2.33) | 0.92× (0.91–0.94) | 1.01× (0.96–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.57–2.69) | 2.82× (2.80–2.83) | 3.52× (3.49–3.54) | 3.69× (3.67–3.72) | 4.03× (4.00–4.06) | 2.50× (2.48–2.52) | 2.07× (2.06–2.08) | 1.97× (1.96–1.98) | 2.81× (2.68–2.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.29–2.40) | 2.60× (2.54–2.66) | 3.16× (3.12–3.21) | 3.28× (3.24–3.33) | 3.51× (3.45–3.57) | 2.09× (2.06–2.11) | 1.73× (1.71–1.75) | 1.64× (1.62–1.67) | 2.35× (2.23–2.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.06–2.19) | 1.81× (1.76–1.86) | 2.26× (2.18–2.34) | 2.12× (2.03–2.22) | 2.21× (2.13–2.29) | 1.06× (1.03–1.10) | 0.88× (0.85–0.91) | 0.84× (0.81–0.86) | 1.20× (1.15–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.33–1.36) | 2.64× (2.62–2.67) | 2.39× (2.35–2.43) | 2.45× (2.43–2.47) | 2.83× (2.81–2.86) | 3.15× (3.10–3.19) | 3.08× (3.05–3.12) | 1.23× (1.22–1.24) | 1.37× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.50× (2.46–2.53) | 2.20× (2.17–2.23) | 2.94× (2.90–2.99) | 2.63× (2.57–2.68) | 2.82× (2.77–2.88) | 2.77× (2.73–2.81) | 1.10× (1.09–1.12) | 1.23× (1.22–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.44–1.49) | 2.29× (2.25–2.32) | 1.88× (1.86–1.89) | 2.55× (2.51–2.59) | 2.45× (2.42–2.47) | 2.06× (2.03–2.09) | 2.02× (2.00–2.05) | 0.81× (0.80–0.82) | 0.90× (0.89–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.43–1.50) | 3.02× (3.00–3.04) | 2.52× (2.50–2.54) | 2.66× (2.63–2.69) | 3.09× (3.01–3.17) | 3.65× (3.64–3.66) | 3.80× (3.79–3.81) | 1.41× (1.41–1.42) | 1.67× (1.66–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.29–1.38) | 2.79× (2.69–2.89) | 2.31× (2.23–2.38) | 3.22× (3.10–3.34) | 2.74× (2.65–2.83) | 3.20× (3.09–3.30) | 3.33× (3.22–3.44) | 1.24× (1.20–1.28) | 1.46× (1.42–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.53–1.61) | 2.50× (2.43–2.57) | 1.92× (1.88–1.96) | 2.80× (2.73–2.87) | 2.53× (2.48–2.58) | 2.25× (2.21–2.30) | 2.34× (2.29–2.39) | 0.87× (0.86–0.89) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.75× (2.73–2.78) | 3.67× (3.56–3.77) | 2.89× (2.87–2.92) | 6.20× (6.13–6.27) | 2.38× (2.35–2.40) | 2.19× (2.17–2.21) | 1.01× (1.00–1.02) | 1.04× (1.00–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.44× (2.31–2.57) | 4.00× (3.96–4.03) | 2.60× (2.58–2.63) | 5.41× (5.32–5.50) | 2.06× (2.03–2.08) | 1.90× (1.87–1.92) | 0.87× (0.86–0.88) | 0.90× (0.88–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.59) | 2.54× (2.44–2.64) | 3.93× (3.80–4.08) | 2.59× (2.48–2.71) | 5.38× (5.18–5.60) | 2.04× (1.95–2.12) | 1.88× (1.80–1.96) | 0.86× (0.83–0.90) | 0.89× (0.86–0.93) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.99× (1.93–2.06) | 4.10× (3.97–4.24) | 4.42× (4.27–4.58) | 4.54× (4.39–4.70) | 4.71× (4.55–4.87) | 5.33× (5.15–5.51) | 4.83× (4.66–5.00) | 1.54× (1.49–1.60) | 1.86× (1.80–1.92) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.73× (1.69–1.76) | 3.49× (3.43–3.56) | 3.82× (3.76–3.88) | 4.28× (4.21–4.35) | 3.91× (3.85–3.97) | 4.35× (4.28–4.42) | 3.94× (3.88–4.01) | 1.26× (1.24–1.28) | 1.52× (1.49–1.54) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.76× (1.70–1.81) | 2.76× (2.70–2.82) | 3.00× (2.93–3.06) | 3.34× (3.28–3.41) | 3.11× (3.04–3.17) | 2.83× (2.78–2.89) | 2.57× (2.52–2.62) | 0.82× (0.80–0.84) | 0.99× (0.97–1.01) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 10.99× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.34–0.38) | 0.55× (0.48–0.63) | 0.74× (0.71–0.78) | 0.62× (0.59–0.65) | 5.90× (5.64–6.16) | 0.14× (0.13–0.15) | 0.15× (0.14–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.35–0.37) | 0.62× (0.54–0.71) | 0.65× (0.64–0.67) | 0.57× (0.55–0.59) | 3.79× (3.67–3.91) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.85–4.86) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.97× (7.94–8.01) | 0.65× (0.64–0.66) | 0.68× (0.67–0.68) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.74) | 0.74× (0.67–0.83) | 1.71× (1.66–1.76) | 0.86× (0.84–0.88) | 6.19× (6.02–6.36) | 0.50× (0.49–0.52) | 0.52× (0.51–0.54) | 0.57× (0.56–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.71) | 0.82× (0.78–0.86) | 1.25× (1.20–1.29) | 0.78× (0.75–0.80) | 3.68× (3.55–3.81) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.41) | 0.87× (0.85–0.90) | 0.69× (0.65–0.72) | 3.02× (2.94–3.11) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.66× (0.64–0.68) | 0.47× (0.46–0.47) | 0.89× (0.88–0.91) | 0.84× (0.82–0.86) | 2.59× (2.55–2.64) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.40) | 0.83× (0.80–0.86) | 0.68× (0.65–0.72) | 2.85× (2.75–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.48× (0.46–0.50) | 0.81× (0.79–0.82) | 0.81× (0.77–0.85) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.29–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.24–0.24) | 1.88× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.20–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.67–0.68) | 6.41× (6.38–6.44) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 35 MB | 0.48× (0.46–0.50) | 0.43× (0.39–0.48) | 1.12× (1.07–1.17) | 0.71× (0.68–0.74) | 3.53× (3.39–3.67) | 0.21× (0.20–0.22) | 0.23× (0.22–0.24) | 0.35× (0.33–0.36) | 0.17× (0.16–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 55 MB | 0.50× (0.49–0.52) | 0.52× (0.49–0.56) | 0.92× (0.89–0.96) | 0.77× (0.74–0.80) | 2.35× (2.28–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 428 | 2.35 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 736 | 2.50 | 1.06× (1.06–1.07) | 85% | 1.02× (1.02–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 950 | 3.88 | 1.64× (1.62–1.66) | 55% | 1.20× (1.19–1.21) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (950 MB/s); lbzip2 at 4 threads (197 MB/s); pbz2 at 4 threads (304 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.52, pbz2 1t 0.00, pbz2 2t 0.35, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.64 | 0.02 | 0.08 | 0.221 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.6% |
| MTF + RLE2 | 1.55 | 17.9% |
| T-vector build | 1.51 | 17.5% |
| walk (the one each build uses) | 4.12 | 47.7% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.63 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.484 | 1.301 | (in its total) | 2.69× (2.69–2.69) |  |
| walk | 1.211 | 2.609 | (in its total) | 2.15× (2.14–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.24× (1.23–1.26) |  |
| inverse BWT, total | 1.739 | 3.962 | 3.210 | 2.28× (2.27–2.29) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m35s |
| 4 threads | 32m14s ± 4m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m06s ± 3m15s |
| the scanner alone (its serial part) | 10m11s ± 0.00s |
| the workers alone, over 4 | 23m06s ± 3m15s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.03 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.5 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
