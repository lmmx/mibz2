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

The latest record: [`a3f26aa`](<https://github.com/lmmx/mibz2/commit/a3f26aa010a78811aad93e07a060c0d8a34ef16c>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45012911811>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.490 s | 428 | 7.6 | 3.07× (3.06–3.09) | 7.04× (7.01–7.06) | 7.19× (7.18–7.20) | 7.97× (7.94–8.00) | 7.11× (7.10–7.12) | 7.73× (7.71–7.75) | 7.24× (7.22–7.26) | 2.28× (2.27–2.29) | 2.64× (2.62–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.269 s | 778 | 8.1 | 3.07× (2.89–3.26) | 6.53× (6.14–6.95) | 7.50× (6.97–8.07) | 7.43× (6.98–7.90) | 6.66× (6.26–7.08) | 13.48× (12.67–14.34) | 12.63× (11.89–13.41) | 3.98× (3.75–4.23) | 4.61× (4.34–4.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 938 | 12.7 | 3.14× (3.10–3.17) | 4.82× (4.75–4.89) | 6.01× (5.88–6.15) | 5.50× (5.42–5.58) | 5.53× (5.43–5.64) | 17.00× (16.78–17.22) | 15.92× (15.74–16.11) | 5.02× (4.95–5.09) | 5.81× (5.72–5.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 983 | 3.6 | 0.60× (0.60–0.61) | 3.66× (3.50–3.82) | 4.44× (4.41–4.48) | 3.58× (3.55–3.61) | 6.23× (6.18–6.28) | 2.99× (2.97–3.02) | 2.67× (2.64–2.69) | 1.01× (1.00–1.02) | 1.52× (1.50–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 917 | 4.0 | 0.53× (0.53–0.54) | 2.81× (2.77–2.86) | 5.72× (5.54–5.92) | 3.35× (3.30–3.40) | 5.75× (5.67–5.83) | 2.78× (2.74–2.82) | 2.48× (2.44–2.51) | 0.94× (0.92–0.96) | 1.41× (1.39–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 917 | 3.9 | 0.54× (0.53–0.55) | 2.83× (2.79–2.87) | 5.74× (5.62–5.86) | 3.38× (3.33–3.44) | 5.84× (5.75–5.93) | 2.79× (2.75–2.82) | 2.48× (2.45–2.52) | 0.94× (0.93–0.95) | 1.41× (1.39–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 589 | 5.8 | 2.62× (2.52–2.72) | 3.02× (2.90–3.14) | 3.90× (3.72–4.08) | 3.92× (3.77–4.09) | 4.17× (4.04–4.31) | 2.54× (2.45–2.64) | 2.13× (2.02–2.23) | 1.99× (1.92–2.07) | 3.00× (2.89–3.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 905 | 7.0 | 2.26× (2.06–2.48) | 2.87× (2.56–3.22) | 4.00× (3.60–4.45) | 3.73× (3.15–4.41) | 4.80× (4.39–5.25) | 3.82× (3.49–4.18) | 3.19× (2.88–3.53) | 2.99× (2.74–3.27) | 4.51× (4.12–4.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 873 | 13.8 | 2.06× (2.01–2.11) | 2.24× (2.12–2.38) | 3.33× (3.12–3.55) | 2.40× (2.30–2.51) | 4.38× (4.20–4.55) | 3.77× (3.63–3.91) | 3.15× (2.98–3.32) | 2.95× (2.84–3.07) | 4.45× (4.28–4.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 53.0 | 1.35× (1.35–1.36) | 2.75× (2.73–2.78) | 2.43× (2.42–2.44) | 2.54× (2.51–2.57) | 2.91× (2.90–2.92) | 3.19× (3.18–3.21) | 3.25× (3.24–3.26) | 1.23× (1.23–1.24) | 1.43× (1.41–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.077 s | 103 | 59.3 | 1.33× (1.20–1.47) | 2.73× (2.53–2.96) | 2.45× (2.33–2.57) | 3.21× (3.05–3.37) | 3.05× (2.91–3.20) | 5.21× (5.03–5.39) | 5.30× (5.12–5.49) | 2.01× (1.94–2.07) | 2.34× (2.24–2.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.065 s | 124 | 81.4 | 1.37× (1.35–1.39) | 2.25× (2.19–2.30) | 2.21× (2.10–2.32) | 2.77× (2.69–2.85) | 3.02× (2.93–3.10) | 6.36× (6.27–6.45) | 6.48× (6.38–6.58) | 2.45× (2.41–2.49) | 2.86× (2.80–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.141 s | 68 | 48.4 | 1.46× (1.45–1.47) | 3.13× (3.10–3.15) | 2.61× (2.57–2.65) | 2.74× (2.72–2.76) | 3.19× (3.12–3.25) | 3.71× (3.69–3.73) | 4.03× (4.00–4.06) | 1.43× (1.42–1.43) | 1.73× (1.72–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.083 s | 116 | 54.3 | 1.38× (1.34–1.41) | 2.94× (2.84–3.03) | 2.61× (2.53–2.69) | 3.65× (3.54–3.77) | 3.22× (3.15–3.29) | 6.25× (6.11–6.38) | 6.79× (6.62–6.95) | 2.40× (2.34–2.46) | 2.91× (2.84–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 130 | 79.4 | 1.44× (1.40–1.48) | 2.40× (2.28–2.53) | 2.32× (2.27–2.36) | 3.11× (3.03–3.19) | 2.92× (2.87–2.97) | 7.15× (7.00–7.30) | 7.76× (7.58–7.95) | 2.75× (2.68–2.81) | 3.33× (3.25–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.0 | 0.67× (0.66–0.68) | 3.06× (3.03–3.08) | 4.47× (4.44–4.50) | 3.13× (3.11–3.15) | 6.87× (6.79–6.96) | 2.56× (2.54–2.58) | 2.35× (2.34–2.37) | 0.99× (0.98–1.00) | 1.54× (1.53–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.7 | 0.54× (0.53–0.55) | 2.78× (2.77–2.80) | 4.77× (4.74–4.80) | 2.89× (2.86–2.91) | 6.29× (6.22–6.35) | 2.33× (2.31–2.35) | 2.14× (2.12–2.15) | 0.90× (0.89–0.91) | 1.40× (1.39–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 660 | 5.7 | 0.54× (0.53–0.54) | 2.71× (2.66–2.76) | 4.64× (4.55–4.73) | 2.81× (2.77–2.85) | 6.12× (6.00–6.26) | 2.26× (2.21–2.31) | 2.07× (2.03–2.12) | 0.87× (0.85–0.89) | 1.36× (1.33–1.39) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 0.070 s | 197 | 16.9 | 2.05× (2.04–2.07) | 4.33× (4.27–4.39) | 4.68× (4.61–4.75) | 4.74× (4.69–4.80) | 4.90× (4.84–4.96) | 5.51× (5.44–5.58) | 5.23× (5.20–5.26) | 1.59× (1.57–1.61) | 2.01× (1.99–2.04) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 0.048 s | 287 | 21.2 | 1.60× (1.55–1.65) | 3.46× (3.34–3.59) | 4.30× (4.02–4.61) | 4.57× (4.35–4.81) | 4.34× (4.20–4.49) | 7.93× (7.67–8.21) | 7.53× (7.27–7.80) | 2.29× (2.22–2.37) | 2.90× (2.80–3.00) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 0.041 s | 334 | 32.9 | 1.65× (1.59–1.72) | 2.75× (2.66–2.85) | 3.73× (3.61–3.86) | 3.70× (3.58–3.83) | 4.07× (3.94–4.21) | 9.30× (9.01–9.60) | 8.83× (8.58–9.08) | 2.69× (2.61–2.77) | 3.40× (3.29–3.50) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.07× (3.05–3.08) | 7.00× (6.98–7.03) | 7.15× (7.14–7.17) | 7.94× (7.91–7.97) | 7.08× (7.07–7.09) | 7.70× (7.68–7.72) | 7.21× (7.19–7.23) | 2.28× (2.27–2.29) | 2.62× (2.61–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.00× (2.98–3.03) | 6.79× (6.74–6.84) | 7.18× (7.02–7.33) | 7.88× (7.84–7.92) | 6.72× (6.68–6.75) | 7.21× (7.17–7.26) | 6.76× (6.72–6.79) | 2.13× (2.12–2.15) | 2.46× (2.44–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.00× (2.96–3.04) | 5.09× (5.05–5.14) | 5.46× (5.40–5.52) | 5.81× (5.74–5.87) | 5.13× (5.09–5.18) | 4.62× (4.58–4.67) | 4.33× (4.29–4.37) | 1.37× (1.36–1.38) | 1.58× (1.56–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.28× (3.26–3.31) | 4.09× (4.05–4.12) | 3.37× (3.34–3.40) | 5.79× (5.74–5.84) | 2.83× (2.80–2.85) | 2.53× (2.51–2.55) | 1.02× (1.00–1.03) | 1.09× (1.05–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.85× (2.81–2.88) | 5.00× (4.92–5.08) | 3.07× (3.03–3.11) | 5.17× (5.11–5.24) | 2.54× (2.50–2.57) | 2.27× (2.24–2.30) | 0.91× (0.89–0.93) | 0.98× (0.94–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.17× (3.11–3.23) | 5.11× (5.04–5.17) | 3.14× (3.10–3.19) | 5.30× (5.23–5.38) | 2.58× (2.54–2.61) | 2.31× (2.27–2.34) | 0.93× (0.92–0.94) | 0.99× (0.96–1.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.44–2.63) | 2.78× (2.67–2.89) | 3.51× (3.34–3.69) | 3.77× (3.62–3.93) | 4.01× (3.88–4.15) | 2.47× (2.37–2.56) | 2.07× (1.97–2.17) | 1.94× (1.87–2.02) | 2.70× (2.59–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.27–2.38) | 2.63× (2.56–2.69) | 3.18× (3.12–3.24) | 3.35× (3.28–3.42) | 3.53× (3.46–3.59) | 2.08× (2.04–2.12) | 1.74× (1.68–1.81) | 1.64× (1.60–1.67) | 2.27× (2.22–2.32) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.95–2.12) | 1.82× (1.78–1.87) | 2.24× (2.21–2.28) | 2.11× (2.05–2.18) | 2.21× (2.16–2.25) | 1.06× (1.04–1.08) | 0.89× (0.86–0.92) | 0.83× (0.82–0.85) | 1.16× (1.13–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.35–1.35) | 2.67× (2.66–2.69) | 2.39× (2.38–2.40) | 2.52× (2.49–2.54) | 2.89× (2.88–2.90) | 3.16× (3.15–3.17) | 3.22× (3.21–3.23) | 1.23× (1.22–1.23) | 1.39× (1.38–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.32) | 2.53× (2.50–2.56) | 2.24× (2.21–2.26) | 3.01× (2.96–3.07) | 2.64× (2.61–2.67) | 2.83× (2.79–2.86) | 2.88× (2.84–2.91) | 1.10× (1.08–1.11) | 1.25× (1.22–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.46–1.51) | 2.32× (2.29–2.36) | 1.88× (1.86–1.91) | 2.59× (2.58–2.61) | 2.44× (2.41–2.47) | 2.06× (2.04–2.09) | 2.10× (2.08–2.12) | 0.80× (0.79–0.81) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.45–1.46) | 3.06× (3.03–3.09) | 2.56× (2.54–2.59) | 2.72× (2.70–2.74) | 3.16× (3.09–3.22) | 3.67× (3.66–3.69) | 3.99× (3.96–4.02) | 1.42× (1.41–1.43) | 1.69× (1.67–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.39) | 2.86× (2.82–2.90) | 2.38× (2.36–2.41) | 3.36× (3.32–3.40) | 2.84× (2.82–2.87) | 3.27× (3.25–3.30) | 3.56× (3.51–3.60) | 1.27× (1.25–1.28) | 1.50× (1.48–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.48–1.61) | 2.52× (2.43–2.61) | 1.92× (1.86–1.98) | 2.81× (2.73–2.91) | 2.52× (2.44–2.61) | 2.25× (2.18–2.31) | 2.44× (2.37–2.51) | 0.87× (0.84–0.89) | 1.03× (1.00–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.74× (2.70–2.78) | 3.60× (3.54–3.67) | 2.87× (2.84–2.90) | 6.14× (6.05–6.24) | 2.37× (2.35–2.40) | 2.18× (2.16–2.21) | 0.99× (0.98–1.01) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.39× (2.33–2.45) | 4.03× (3.98–4.09) | 2.61× (2.58–2.64) | 5.42× (5.35–5.50) | 2.07× (2.06–2.09) | 1.91× (1.89–1.93) | 0.87× (0.86–0.88) | 0.87× (0.84–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.56× (2.52–2.60) | 4.00× (3.94–4.06) | 2.60× (2.58–2.62) | 5.42× (5.33–5.50) | 2.06× (2.03–2.09) | 1.90× (1.87–1.92) | 0.86× (0.85–0.88) | 0.87× (0.83–0.91) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 2.03× (2.02–2.04) | 4.16× (4.14–4.18) | 4.51× (4.49–4.53) | 4.66× (4.64–4.68) | 4.81× (4.78–4.84) | 5.41× (5.37–5.44) | 5.14× (5.09–5.18) | 1.58× (1.57–1.59) | 1.87× (1.86–1.88) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 1.72× (1.69–1.74) | 3.51× (3.47–3.56) | 3.82× (3.76–3.87) | 4.31× (4.25–4.38) | 3.95× (3.90–4.02) | 4.34× (4.28–4.40) | 4.12× (4.07–4.17) | 1.27× (1.25–1.29) | 1.50× (1.48–1.52) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 1.70× (1.63–1.78) | 2.74× (2.69–2.80) | 3.01× (2.96–3.06) | 3.33× (3.26–3.40) | 3.13× (3.07–3.19) | 2.80× (2.76–2.86) | 2.66× (2.61–2.72) | 0.82× (0.81–0.84) | 0.97× (0.95–0.99) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.01× (10.98–11.04) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.36× (0.35–0.37) | 0.60× (0.53–0.67) | 0.74× (0.72–0.75) | 0.62× (0.60–0.63) | 5.83× (5.71–5.96) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.37× (0.36–0.39) | 0.68× (0.58–0.81) | 0.66× (0.63–0.69) | 0.58× (0.54–0.61) | 3.86× (3.67–4.05) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.94× (7.91–7.96) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.69–0.74) | 0.73× (0.68–0.78) | 1.69× (1.64–1.74) | 0.85× (0.83–0.87) | 6.12× (5.93–6.30) | 0.49× (0.48–0.51) | 0.52× (0.51–0.54) | 0.57× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.71× (0.68–0.74) | 0.82× (0.79–0.86) | 1.25× (1.21–1.29) | 0.77× (0.75–0.79) | 3.70× (3.59–3.80) | 0.29× (0.28–0.30) | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.15× (5.13–5.17) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.39× (0.36–0.42) | 0.87× (0.85–0.90) | 0.70× (0.68–0.72) | 3.01× (2.92–3.09) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.89) | 0.82× (0.81–0.83) | 2.57× (2.54–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.16× (5.15–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.49) | 0.38× (0.35–0.40) | 0.85× (0.83–0.87) | 0.71× (0.69–0.73) | 2.93× (2.87–2.99) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.80× (0.78–0.82) | 2.32× (2.28–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.91× (0.90–0.91) | 6.21× (6.18–6.23) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.90–0.91) | 6.22× (6.20–6.25) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.38× (6.37–6.40) | 0.39× (0.38–0.39) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 35 MB | 0.48× (0.47–0.49) | 0.46× (0.43–0.49) | 1.13× (1.10–1.15) | 0.72× (0.69–0.75) | 3.56× (3.48–3.63) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.17) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 54 MB | 0.52× (0.51–0.53) | 0.51× (0.48–0.55) | 0.95× (0.93–0.97) | 0.77× (0.73–0.81) | 2.43× (2.39–2.46) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 428 | 2.35 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 778 | 2.50 | 1.07× (1.06–1.07) | 87% | 1.04× (1.03–1.04) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 938 | 3.92 | 1.67× (1.65–1.68) | 55% | 1.21× (1.20–1.22) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (938 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (301 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.02, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.64 | 0.02 | 0.08 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.8% |
| MTF + RLE2 | 1.52 | 17.8% |
| T-vector build | 1.51 | 17.7% |
| walk (the one each build uses) | 4.06 | 47.5% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.486 | 1.301 | (in its total) | 2.68× (2.67–2.68) |  |
| walk | 1.194 | 2.615 | (in its total) | 2.19× (2.19–2.20) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.22× (1.21–1.23) |  |
| inverse BWT, total | 1.726 | 3.969 | 3.199 | 2.31× (2.30–2.31) | 1.86× (1.85–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m32s |
| 4 threads | 32m25s ± 4m33s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m17s ± 3m17s |
| the scanner alone (its serial part) | 10m20s ± 19.7s |
| the workers alone, over 4 | 23m17s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 6.0 ns a byte of bz2, the workers 3.05 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,732,203 | 13,845,587 | \`5d6d3769149d431f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
