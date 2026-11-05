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

The latest record: [`e29bd97`](<https://github.com/lmmx/mibz2/commit/e29bd97569dd2248827b24f2d99d58819cb58592>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42045760043>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.690 s | 304 | 10.7 | 2.21× (2.18–2.23) | 5.14× (5.01–5.27) | 5.18× (5.07–5.29) | 5.68× (5.59–5.77) | 5.09× (4.98–5.20) | 5.42× (5.30–5.55) | 5.15× (5.07–5.23) | 1.65× (1.62–1.68) | 1.89× (1.87–1.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.378 s | 555 | 11.4 | 2.24× (2.15–2.34) | 4.92× (4.63–5.22) | 5.42× (5.17–5.67) | 5.47× (5.22–5.74) | 4.83× (4.64–5.03) | 9.74× (9.35–10.14) | 9.26× (8.84–9.69) | 2.96× (2.81–3.13) | 3.40× (3.22–3.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.304 s | 690 | 17.5 | 2.33× (2.28–2.40) | 3.67× (3.48–3.88) | 4.41× (4.27–4.55) | 4.08× (3.91–4.26) | 4.08× (3.91–4.25) | 12.37× (12.11–12.64) | 11.76× (11.59–11.93) | 3.77× (3.71–3.83) | 4.33× (4.26–4.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,000 | 3.6 | 0.61× (0.59–0.62) | 3.66× (3.56–3.75) | 4.64× (4.46–4.83) | 3.61× (3.55–3.68) | 6.22× (6.10–6.34) | 2.87× (2.81–2.93) | 2.69× (2.64–2.75) | 1.02× (0.99–1.05) | 1.53× (1.50–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 976 | 3.7 | 0.57× (0.56–0.58) | 3.01× (2.95–3.06) | 5.99× (5.89–6.09) | 3.57× (3.50–3.63) | 6.09× (6.00–6.19) | 2.82× (2.77–2.87) | 2.65× (2.60–2.70) | 1.00× (0.96–1.04) | 1.51× (1.48–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 957 | 3.7 | 0.56× (0.56–0.57) | 2.96× (2.92–3.00) | 5.87× (5.79–5.96) | 3.50× (3.46–3.55) | 5.99× (5.92–6.05) | 2.76× (2.73–2.79) | 2.59× (2.56–2.63) | 0.98× (0.94–1.02) | 1.47× (1.46–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 545 | 6.3 | 2.69× (2.61–2.77) | 2.83× (2.77–2.88) | 3.59× (3.50–3.69) | 3.66× (3.59–3.72) | 3.85× (3.78–3.91) | 2.33× (2.29–2.37) | 1.96× (1.92–1.99) | 1.89× (1.88–1.90) | 2.81× (2.75–2.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 854 | 7.7 | 2.10× (1.90–2.32) | 2.70× (2.29–3.18) | 3.81× (3.35–4.33) | 3.13× (2.85–3.44) | 4.36× (3.97–4.79) | 3.45× (3.14–3.79) | 2.90× (2.64–3.18) | 2.80× (2.54–3.08) | 4.16× (3.79–4.57) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 751 | 14.9 | 1.80× (1.76–1.85) | 1.81× (1.68–1.96) | 2.85× (2.78–2.92) | 2.17× (2.02–2.32) | 3.75× (3.63–3.88) | 3.23× (3.11–3.35) | 2.71× (2.62–2.81) | 2.62× (2.55–2.69) | 3.90× (3.75–4.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.151 s | 53 | 62.2 | 1.17× (1.16–1.18) | 2.36× (2.35–2.37) | 2.11× (2.07–2.16) | 2.17× (2.15–2.19) | 2.48× (2.42–2.55) | 2.70× (2.68–2.71) | 2.77× (2.74–2.79) | 1.07× (1.06–1.08) | 1.21× (1.20–1.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.091 s | 88 | 69.0 | 1.13× (1.11–1.16) | 2.33× (2.11–2.57) | 2.18× (2.03–2.33) | 2.75× (2.68–2.82) | 2.59× (2.55–2.64) | 4.50× (4.43–4.57) | 4.61× (4.53–4.70) | 1.79× (1.75–1.83) | 2.02× (1.98–2.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.074 s | 108 | 88.7 | 1.21× (1.14–1.29) | 1.97× (1.86–2.09) | 1.87× (1.74–2.01) | 2.38× (2.20–2.57) | 2.58× (2.45–2.73) | 5.46× (5.17–5.77) | 5.60× (5.30–5.92) | 2.17× (2.05–2.29) | 2.45× (2.32–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.200 s | 48 | 68.4 | 1.07× (1.07–1.08) | 2.22× (2.19–2.25) | 1.87× (1.83–1.91) | 1.94× (1.92–1.96) | 2.26× (2.20–2.31) | 2.76× (2.72–2.79) | 2.88× (2.85–2.90) | 1.04× (1.02–1.05) | 1.23× (1.22–1.24) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.116 s | 83 | 75.2 | 1.03× (1.01–1.05) | 2.12× (2.00–2.24) | 1.92× (1.82–2.02) | 2.62× (2.55–2.69) | 2.33× (2.28–2.38) | 4.76× (4.66–4.86) | 4.97× (4.85–5.09) | 1.79× (1.74–1.84) | 2.12× (2.07–2.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.093 s | 103 | 98.9 | 1.18× (1.15–1.21) | 1.91× (1.83–2.00) | 1.90× (1.72–2.10) | 2.52× (2.45–2.58) | 2.35× (2.24–2.47) | 5.96× (5.84–6.08) | 6.21× (6.08–6.35) | 2.24× (2.18–2.30) | 2.65× (2.61–2.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.1 | 0.66× (0.65–0.67) | 3.14× (2.97–3.32) | 4.40× (4.31–4.49) | 3.10× (3.05–3.15) | 6.69× (6.57–6.82) | 2.44× (2.40–2.48) | 2.32× (2.29–2.35) | 0.97× (0.95–0.99) | 1.51× (1.48–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 697 | 5.5 | 0.55× (0.55–0.56) | 2.84× (2.78–2.91) | 5.01× (4.92–5.10) | 2.95× (2.91–3.00) | 6.33× (6.19–6.47) | 2.31× (2.26–2.35) | 2.19× (2.16–2.22) | 0.92× (0.90–0.93) | 1.43× (1.40–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 683 | 5.4 | 0.56× (0.55–0.57) | 2.83× (2.77–2.89) | 5.05× (4.87–5.24) | 2.93× (2.88–2.98) | 6.28× (6.22–6.35) | 2.29× (2.25–2.33) | 2.17× (2.14–2.21) | 0.91× (0.89–0.93) | 1.42× (1.39–1.45) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 0.080 s | 152 | 22.0 | 1.49× (1.44–1.55) | 3.20× (3.04–3.37) | 3.40× (3.25–3.57) | 3.31× (3.17–3.45) | 3.53× (3.38–3.68) | 3.58× (3.43–3.73) | 3.64× (3.47–3.82) | 1.16× (1.11–1.21) | 1.40× (1.33–1.47) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 0.050 s | 242 | 25.6 | 1.42× (1.22–1.66) | 2.85× (2.67–3.05) | 3.35× (3.10–3.63) | 3.66× (3.55–3.77) | 3.65× (3.51–3.80) | 5.87× (5.72–6.02) | 5.97× (5.71–6.23) | 1.90× (1.83–1.97) | 2.29× (2.21–2.38) | yes |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 0.041 s | 293 | 36.8 | 1.35× (1.31–1.40) | 2.39× (2.18–2.61) | 2.88× (2.49–3.34) | 3.21× (3.02–3.41) | 3.46× (3.24–3.70) | 7.03× (6.79–7.28) | 7.15× (6.83–7.49) | 2.27× (2.19–2.37) | 2.75× (2.68–2.82) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.20× (2.18–2.23) | 5.12× (4.99–5.24) | 5.16× (5.05–5.27) | 5.66× (5.57–5.75) | 5.08× (4.97–5.18) | 5.41× (5.28–5.53) | 5.14× (5.06–5.22) | 1.65× (1.62–1.68) | 1.88× (1.86–1.91) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.14× (2.12–2.17) | 4.91× (4.78–5.06) | 5.05× (5.00–5.10) | 5.63× (5.55–5.71) | 4.75× (4.70–4.80) | 5.07× (5.01–5.13) | 4.82× (4.74–4.90) | 1.55× (1.50–1.59) | 1.77× (1.71–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.18–2.23) | 3.76× (3.57–3.96) | 4.02× (3.92–4.12) | 4.29× (4.13–4.46) | 3.79× (3.64–3.94) | 3.32× (3.27–3.38) | 3.16× (3.12–3.20) | 1.01× (0.98–1.05) | 1.16× (1.13–1.19) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.30× (3.25–3.36) | 4.17× (4.04–4.31) | 3.37× (3.31–3.44) | 5.74× (5.64–5.84) | 2.69× (2.64–2.75) | 2.54× (2.49–2.59) | 1.02× (0.99–1.04) | 1.07× (1.05–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.60–0.63) | 3.09× (3.04–3.14) | 5.37× (5.29–5.45) | 3.30× (3.24–3.36) | 5.56× (5.48–5.64) | 2.61× (2.57–2.65) | 2.46× (2.42–2.50) | 0.98× (0.95–1.02) | 1.04× (1.02–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.40× (3.34–3.46) | 5.36× (5.30–5.41) | 3.33× (3.28–3.38) | 5.56× (5.50–5.62) | 2.61× (2.58–2.65) | 2.46× (2.43–2.49) | 0.98× (0.95–1.02) | 1.04× (1.02–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.60× (2.52–2.68) | 2.61× (2.58–2.64) | 3.27× (3.23–3.30) | 3.52× (3.46–3.57) | 3.70× (3.64–3.76) | 2.26× (2.22–2.30) | 1.91× (1.88–1.93) | 1.84× (1.83–1.85) | 2.52× (2.48–2.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.13× (2.07–2.19) | 2.33× (2.27–2.40) | 2.85× (2.80–2.91) | 3.02× (2.96–3.09) | 3.14× (3.07–3.21) | 1.84× (1.80–1.88) | 1.55× (1.52–1.59) | 1.50× (1.47–1.54) | 2.06× (2.01–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.88× (1.81–1.96) | 1.67× (1.61–1.72) | 2.08× (2.02–2.14) | 1.97× (1.89–2.04) | 1.97× (1.92–2.02) | 0.95× (0.93–0.98) | 0.80× (0.78–0.82) | 0.78× (0.76–0.80) | 1.06× (1.03–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.17× (1.16–1.18) | 2.29× (2.28–2.31) | 2.07× (2.04–2.10) | 2.15× (2.13–2.17) | 2.46× (2.40–2.53) | 2.68× (2.66–2.69) | 2.74× (2.72–2.76) | 1.07× (1.06–1.08) | 1.18× (1.17–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.12× (1.11–1.13) | 2.18× (2.14–2.22) | 1.94× (1.88–2.00) | 2.59× (2.55–2.62) | 2.26× (2.24–2.27) | 2.42× (2.40–2.43) | 2.48× (2.46–2.50) | 0.97× (0.96–0.98) | 1.07× (1.06–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.37× (1.32–1.43) | 2.16× (2.08–2.25) | 1.75× (1.70–1.80) | 2.39× (2.35–2.43) | 2.28× (2.22–2.33) | 1.88× (1.85–1.91) | 1.93× (1.90–1.96) | 0.75× (0.74–0.77) | 0.83× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.07× (1.07–1.08) | 2.19× (2.17–2.20) | 1.84× (1.80–1.87) | 1.93× (1.91–1.95) | 2.24× (2.19–2.30) | 2.74× (2.70–2.77) | 2.86× (2.83–2.88) | 1.03× (1.02–1.05) | 1.20× (1.19–1.21) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.04) | 2.10× (2.02–2.20) | 1.73× (1.70–1.77) | 2.43× (2.41–2.45) | 2.07× (2.03–2.11) | 2.50× (2.47–2.52) | 2.60× (2.56–2.65) | 0.94× (0.93–0.96) | 1.09× (1.08–1.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.29× (1.25–1.34) | 2.09× (1.99–2.19) | 1.65× (1.50–1.80) | 2.29× (2.22–2.36) | 2.10× (1.98–2.24) | 1.90× (1.87–1.93) | 1.98× (1.95–2.01) | 0.72× (0.71–0.73) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.71× (2.67–2.75) | 3.54× (3.49–3.59) | 2.82× (2.78–2.87) | 5.95× (5.84–6.06) | 2.25× (2.21–2.29) | 2.14× (2.11–2.17) | 0.98× (0.96–1.00) | 0.99× (0.96–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.62) | 2.48× (2.44–2.51) | 4.22× (4.17–4.26) | 2.72× (2.69–2.76) | 5.58× (5.49–5.68) | 2.10× (2.07–2.13) | 2.00× (1.98–2.02) | 0.91× (0.89–0.93) | 0.92× (0.90–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.70–2.80) | 4.27× (4.20–4.33) | 2.78× (2.74–2.83) | 5.69× (5.64–5.75) | 2.14× (2.11–2.18) | 2.04× (2.02–2.06) | 0.93× (0.91–0.95) | 0.94× (0.91–0.96) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 1.48× (1.43–1.54) | 3.04× (2.88–3.21) | 3.26× (3.11–3.41) | 3.25× (3.12–3.38) | 3.47× (3.32–3.62) | 3.51× (3.37–3.66) | 3.57× (3.41–3.75) | 1.15× (1.11–1.20) | 1.34× (1.29–1.40) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 1.39× (1.36–1.41) | 2.81× (2.62–3.02) | 3.06× (2.85–3.29) | 3.22× (3.12–3.32) | 3.15× (3.05–3.26) | 3.10× (3.04–3.15) | 3.15× (3.03–3.27) | 1.02× (0.98–1.05) | 1.18× (1.15–1.21) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 1.49× (1.43–1.56) | 2.46× (2.23–2.70) | 2.78× (2.39–3.23) | 2.76× (2.60–2.93) | 2.74× (2.57–2.92) | 2.14× (2.09–2.19) | 2.17× (2.10–2.25) | 0.70× (0.68–0.72) | 0.82× (0.79–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.34× (0.33–0.35) | 1.08× (1.08–1.09) | 0.86× (0.85–0.86) | 12.23× (12.21–12.25) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.37–0.38) | 0.61× (0.55–0.68) | 0.81× (0.80–0.82) | 0.67× (0.66–0.68) | 6.40× (6.35–6.45) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.74× (0.63–0.86) | 0.70× (0.69–0.71) | 0.61× (0.59–0.63) | 4.10× (4.09–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.81× (1.80–1.81) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.74–1.75) | 0.66× (0.65–0.67) | 8.19× (8.16–8.22) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.67) | 0.71× (0.66–0.77) | 1.64× (1.60–1.68) | 0.80× (0.79–0.82) | 5.92× (5.79–6.05) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.64× (0.62–0.65) | 0.71× (0.67–0.76) | 1.16× (1.12–1.19) | 0.74× (0.72–0.77) | 3.42× (3.32–3.52) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.73–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.41× (0.37–0.44) | 0.91× (0.88–0.94) | 0.69× (0.66–0.73) | 3.13× (3.03–3.24) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.86× (0.85–0.87) | 0.78× (0.74–0.82) | 2.49× (2.46–2.52) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.21× (1.20–1.21) | 0.83× (0.83–0.84) | 5.30× (5.29–5.31) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.41) | 0.87× (0.83–0.91) | 0.72× (0.69–0.75) | 2.97× (2.86–3.10) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.47× (0.44–0.50) | 0.80× (0.79–0.82) | 0.79× (0.76–0.83) | 2.32× (2.28–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.10× (1.10–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.77× (0.77–0.78) | 5.45× (5.44–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.78× (0.77–0.78) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.40× (0.40–0.41) | 1.51× (1.50–1.52) | 0.65× (0.64–0.65) | 6.78× (6.76–6.80) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.34) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 2 | 32 MB | 0.48× (0.46–0.50) | 0.39× (0.36–0.43) | 1.18× (1.14–1.23) | 0.72× (0.69–0.75) | 3.73× (3.59–3.86) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.35–0.38) | 0.18× (0.18–0.19) |
| `text` (1.64 MB → 12.1 MB, 7.4×) | 4 | 52 MB | 0.51× (0.49–0.52) | 0.52× (0.48–0.57) | 0.97× (0.94–0.99) | 0.75× (0.73–0.77) | 2.47× (2.41–2.53) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 304 | 3.30 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 555 | 3.52 | 1.07× (1.04–1.10) | 90% | 1.02× (0.97–1.09) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 690 | 5.41 | 1.63× (1.59–1.67) | 57% | 1.19× (1.12–1.27) | 1.63× (1.59–1.68) |

**Best:** mibz2 at 4 threads (690 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.66, mibz2 4t 1.12, lbzip2 1t 0.09, lbzip2 2t 0.40, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 0.91 | 0.02 | 0.09 | 0.304 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 11.9% |
| MTF + RLE2 | 1.98 | 19.2% |
| T-vector build | 2.13 | 20.7% |
| walk (the one each build uses) | 4.42 | 42.9% |
| of which: segment assembly | 0.17 | 1.6% |
| RLE1 | 0.29 | 2.8% |
| CRC | 0.26 | 2.5% |
| **total** | 10.31 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.376 | (in its total) | 2.05× (2.04–2.06) |  |
| walk | 1.319 | 2.665 | (in its total) | 2.03× (2.01–2.04) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.057 | (in its total) | 1.16× (1.14–1.17) |  |
| inverse BWT, total | 2.041 | 4.097 | 3.271 | 2.01× (2.00–2.03) | 1.61× (1.60–1.63) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h41m ± 12m06s |
| 4 threads | 44m38s ± 6m22s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m15s ± 4m46s |
| the scanner alone (its serial part) | 17m01s ± 30.2s |
| the workers alone, over 4 | 33m15s ± 4m46s |

At 4 threads the workers bound it. Per byte: the scanner 9.9 ns a byte of bz2, the workers 4.36 ns a byte of output at 4 threads (CPU, summed over them), 2.78 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,643,758 | 12,097,676 | \`8bd1649fd2f4ff1c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
