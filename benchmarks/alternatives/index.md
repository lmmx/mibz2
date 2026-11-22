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

The latest record: [`96a24d3`](<https://github.com/lmmx/mibz2/commit/96a24d3e4f764b3918089b0d5bcd5fe4b9bc1fea>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44988145922>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.502 s | 418 | 5.9 | 3.02× (3.01–3.04) | 6.90× (6.87–6.93) | 7.07× (7.05–7.10) | 7.71× (7.68–7.73) | 6.95× (6.92–6.98) | 7.56× (7.53–7.59) | 6.95× (6.93–6.98) | 2.25× (2.24–2.26) | 2.62× (2.61–2.63) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.292 s | 718 | 6.5 | 2.93× (2.81–3.06) | 6.42× (6.18–6.67) | 7.12× (6.88–7.37) | 7.08× (6.82–7.35) | 6.41× (6.19–6.64) | 12.88× (12.42–13.36) | 11.84× (11.42–12.28) | 3.84× (3.70–3.98) | 4.46× (4.30–4.63) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.240 s | 872 | 10.0 | 2.95× (2.90–3.00) | 4.59× (4.50–4.68) | 5.62× (5.49–5.76) | 5.17× (5.10–5.25) | 5.22× (5.15–5.28) | 15.79× (15.58–16.00) | 14.52× (14.35–14.69) | 4.71× (4.64–4.78) | 5.47× (5.40–5.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 978 | 2.7 | 0.60× (0.59–0.60) | 3.65× (3.49–3.82) | 4.43× (4.37–4.49) | 3.57× (3.53–3.62) | 6.17× (6.05–6.28) | 2.98× (2.94–3.02) | 2.66× (2.63–2.70) | 1.01× (0.99–1.03) | 1.51× (1.49–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 966 | 2.8 | 0.56× (0.55–0.57) | 2.96× (2.92–3.01) | 5.94× (5.79–6.09) | 3.55× (3.52–3.59) | 6.05× (5.98–6.11) | 2.92× (2.88–2.97) | 2.61× (2.58–2.65) | 0.99× (0.97–1.01) | 1.48× (1.46–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 943 | 2.8 | 0.56× (0.55–0.57) | 2.93× (2.89–2.97) | 5.94× (5.75–6.15) | 3.47× (3.41–3.52) | 5.98× (5.89–6.07) | 2.88× (2.84–2.92) | 2.57× (2.54–2.61) | 0.97× (0.96–0.99) | 1.46× (1.44–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 588 | 4.4 | 2.81× (2.72–2.90) | 3.05× (3.01–3.08) | 4.06× (3.96–4.16) | 3.84× (3.81–3.88) | 4.18× (4.13–4.23) | 2.61× (2.53–2.70) | 2.12× (2.10–2.15) | 2.08× (2.05–2.10) | 3.03× (3.00–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 867 | 5.4 | 2.15× (1.96–2.35) | 2.67× (2.37–3.02) | 3.67× (3.26–4.13) | 3.24× (2.88–3.65) | 4.54× (4.13–4.98) | 3.68× (3.28–4.13) | 2.99× (2.71–3.30) | 2.93× (2.65–3.23) | 4.27× (3.87–4.71) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 784 | 10.5 | 1.87× (1.76–1.99) | 1.92× (1.75–2.12) | 2.84× (2.71–2.99) | 2.16× (2.08–2.24) | 3.97× (3.80–4.14) | 3.43× (3.22–3.65) | 2.78× (2.65–2.92) | 2.72× (2.59–2.86) | 3.97× (3.78–4.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 40.0 | 1.38× (1.37–1.40) | 2.76× (2.73–2.78) | 2.44× (2.42–2.47) | 2.49× (2.47–2.51) | 2.89× (2.86–2.92) | 3.17× (3.14–3.20) | 3.15× (3.11–3.19) | 1.24× (1.23–1.25) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 45.6 | 1.26× (1.19–1.32) | 2.50× (2.36–2.63) | 2.34× (2.23–2.46) | 3.01× (2.90–3.12) | 2.86× (2.72–3.00) | 4.94× (4.71–5.18) | 4.92× (4.68–5.16) | 1.93× (1.84–2.02) | 2.22× (2.11–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 62.7 | 1.28× (1.24–1.31) | 2.06× (2.02–2.10) | 1.98× (1.90–2.05) | 2.49× (2.43–2.55) | 2.76× (2.69–2.82) | 5.81× (5.69–5.94) | 5.78× (5.64–5.93) | 2.27× (2.22–2.32) | 2.62× (2.54–2.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 37.1 | 1.49× (1.49–1.50) | 3.05× (3.03–3.06) | 2.57× (2.53–2.60) | 2.64× (2.63–2.66) | 3.08× (3.06–3.10) | 3.64× (3.62–3.67) | 3.84× (3.81–3.86) | 1.42× (1.41–1.43) | 1.69× (1.68–1.70) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 42.7 | 1.33× (1.28–1.39) | 2.71× (2.61–2.80) | 2.46× (2.37–2.56) | 3.36× (3.20–3.52) | 3.01× (2.91–3.12) | 5.82× (5.61–6.03) | 6.13× (5.91–6.35) | 2.26× (2.18–2.35) | 2.70× (2.61–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 125 | 60.5 | 1.43× (1.38–1.47) | 2.27× (2.20–2.35) | 2.17× (2.12–2.22) | 2.97× (2.90–3.04) | 2.80× (2.73–2.86) | 6.79× (6.65–6.93) | 7.15× (7.01–7.29) | 2.64× (2.58–2.70) | 3.15× (3.09–3.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 3.8 | 0.67× (0.65–0.68) | 3.02× (2.97–3.08) | 4.43× (4.35–4.51) | 3.12× (3.07–3.17) | 6.75× (6.65–6.87) | 2.53× (2.48–2.57) | 2.32× (2.29–2.35) | 0.99× (0.98–1.00) | 1.52× (1.50–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 685 | 4.1 | 0.55× (0.54–0.56) | 2.84× (2.80–2.88) | 4.87× (4.80–4.94) | 2.96× (2.91–3.02) | 6.41× (6.29–6.54) | 2.37× (2.33–2.40) | 2.18× (2.14–2.21) | 0.93× (0.91–0.94) | 1.43× (1.41–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 683 | 4.1 | 0.56× (0.55–0.57) | 2.83× (2.79–2.87) | 4.84× (4.77–4.91) | 2.92× (2.87–2.97) | 6.38× (6.27–6.50) | 2.35× (2.31–2.38) | 2.16× (2.12–2.19) | 0.92× (0.90–0.94) | 1.42× (1.39–1.44) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.071 s | 189 | 13.3 | 1.99× (1.98–2.00) | 4.14× (4.11–4.17) | 4.53× (4.50–4.56) | 4.54× (4.50–4.57) | 4.72× (4.68–4.77) | 5.32× (5.27–5.36) | 5.00× (4.52–5.52) | 1.53× (1.51–1.54) | 1.85× (1.84–1.86) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.051 s | 264 | 16.8 | 1.55× (1.43–1.68) | 3.28× (3.01–3.57) | 3.80× (3.52–4.11) | 4.14× (3.79–4.53) | 4.05× (3.78–4.33) | 7.23× (6.75–7.75) | 6.80× (5.98–7.73) | 2.08× (1.94–2.22) | 2.52× (2.35–2.69) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.046 s | 293 | 25.6 | 1.51× (1.44–1.57) | 2.42× (2.34–2.50) | 3.25× (3.12–3.40) | 3.29× (3.10–3.49) | 3.72× (3.64–3.81) | 8.29× (8.08–8.51) | 7.80× (7.07–8.60) | 2.38× (2.33–2.44) | 2.89× (2.81–2.96) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.01× (3.00–3.03) | 6.86× (6.84–6.89) | 7.04× (7.02–7.06) | 7.68× (7.66–7.70) | 6.92× (6.89–6.95) | 7.53× (7.50–7.56) | 6.93× (6.90–6.95) | 2.25× (2.24–2.26) | 2.60× (2.59–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.87× (2.85–2.89) | 6.48× (6.43–6.53) | 6.76× (6.72–6.79) | 7.46× (7.41–7.50) | 6.44× (6.41–6.47) | 6.86× (6.82–6.90) | 6.31× (6.28–6.34) | 2.05× (2.03–2.06) | 2.37× (2.35–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.86–2.95) | 4.98× (4.92–5.04) | 5.30× (5.22–5.38) | 5.64× (5.57–5.72) | 5.04× (4.97–5.11) | 4.44× (4.38–4.51) | 4.09× (4.02–4.15) | 1.33× (1.31–1.35) | 1.53× (1.51–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.28× (3.24–3.32) | 4.07× (4.02–4.11) | 3.35× (3.31–3.38) | 5.72× (5.62–5.82) | 2.81× (2.78–2.84) | 2.52× (2.49–2.55) | 1.01× (0.99–1.03) | 1.07× (1.05–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.04× (3.00–3.08) | 5.32× (5.25–5.38) | 3.28× (3.24–3.33) | 5.52× (5.46–5.58) | 2.71× (2.67–2.74) | 2.43× (2.40–2.46) | 0.97× (0.96–0.99) | 1.03× (1.01–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.36× (3.30–3.43) | 5.32× (5.24–5.40) | 3.28× (3.23–3.32) | 5.53× (5.46–5.59) | 2.71× (2.68–2.75) | 2.44× (2.41–2.46) | 0.97× (0.96–0.99) | 1.04× (1.01–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.63–2.80) | 2.81× (2.78–2.83) | 3.73× (3.57–3.90) | 3.68× (3.64–3.72) | 4.00× (3.95–4.05) | 2.52× (2.44–2.61) | 2.06× (2.04–2.08) | 2.02× (2.00–2.04) | 2.72× (2.68–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.22–2.32) | 2.53× (2.45–2.61) | 3.07× (3.00–3.15) | 3.18× (3.10–3.26) | 3.42× (3.33–3.51) | 2.05× (1.97–2.14) | 1.67× (1.63–1.72) | 1.64× (1.60–1.69) | 2.22× (2.14–2.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.03–2.22) | 1.81× (1.76–1.87) | 2.26× (2.19–2.32) | 2.15× (2.09–2.21) | 2.22× (2.16–2.29) | 1.08× (1.04–1.12) | 0.88× (0.86–0.91) | 0.86× (0.84–0.89) | 1.17× (1.13–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.39) | 2.67× (2.65–2.69) | 2.39× (2.37–2.41) | 2.47× (2.45–2.49) | 2.86× (2.83–2.89) | 3.13× (3.10–3.17) | 3.12× (3.07–3.16) | 1.23× (1.22–1.25) | 1.39× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.49× (2.46–2.52) | 2.19× (2.17–2.22) | 2.92× (2.88–2.95) | 2.58× (2.55–2.61) | 2.76× (2.73–2.79) | 2.74× (2.71–2.77) | 1.09× (1.07–1.10) | 1.22× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.43–1.50) | 2.27× (2.22–2.33) | 1.86× (1.82–1.90) | 2.56× (2.52–2.61) | 2.43× (2.38–2.49) | 2.01× (1.97–2.05) | 2.00× (1.96–2.03) | 0.79× (0.77–0.81) | 0.89× (0.87–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.48–1.49) | 3.00× (2.98–3.01) | 2.52× (2.50–2.54) | 2.62× (2.61–2.64) | 3.05× (3.03–3.07) | 3.61× (3.59–3.63) | 3.80× (3.77–3.82) | 1.41× (1.40–1.42) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.39) | 2.78× (2.72–2.84) | 2.30× (2.27–2.33) | 3.19× (3.14–3.25) | 2.73× (2.70–2.77) | 3.14× (3.11–3.17) | 3.31× (3.26–3.35) | 1.23× (1.22–1.24) | 1.45× (1.43–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.57–1.64) | 2.52× (2.46–2.58) | 1.94× (1.91–1.97) | 2.76× (2.72–2.79) | 2.54× (2.50–2.57) | 2.22× (2.19–2.25) | 2.34× (2.30–2.37) | 0.87× (0.86–0.88) | 1.02× (1.01–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.71× (2.65–2.77) | 3.57× (3.48–3.67) | 2.86× (2.82–2.90) | 6.03× (5.93–6.13) | 2.33× (2.29–2.37) | 2.16× (2.13–2.19) | 1.00× (0.98–1.01) | 0.98× (0.95–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.49× (2.46–2.52) | 4.22× (4.17–4.27) | 2.74× (2.70–2.77) | 5.66× (5.58–5.74) | 2.15× (2.13–2.18) | 1.99× (1.96–2.02) | 0.92× (0.91–0.93) | 0.91× (0.89–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.60–0.61) | 2.73× (2.69–2.77) | 4.23× (4.19–4.28) | 2.77× (2.72–2.81) | 5.76× (5.64–5.88) | 2.18× (2.15–2.21) | 2.02× (1.98–2.05) | 0.93× (0.92–0.95) | 0.92× (0.90–0.94) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.96× (1.95–1.98) | 3.99× (3.97–4.02) | 4.35× (4.32–4.37) | 4.44× (4.41–4.47) | 4.62× (4.58–4.67) | 5.19× (5.15–5.24) | 4.88× (4.43–5.39) | 1.51× (1.50–1.52) | 1.80× (1.79–1.82) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.66× (1.63–1.69) | 3.35× (3.29–3.42) | 3.66× (3.59–3.74) | 4.10× (4.03–4.17) | 3.79× (3.72–3.85) | 4.12× (4.04–4.20) | 3.88× (3.52–4.27) | 1.20× (1.18–1.22) | 1.43× (1.41–1.46) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.70× (1.62–1.78) | 2.67× (2.59–2.74) | 2.94× (2.88–3.00) | 3.20× (3.09–3.32) | 3.04× (2.95–3.13) | 2.69× (2.62–2.76) | 2.53× (2.32–2.76) | 0.78× (0.76–0.80) | 0.94× (0.91–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.94× (10.92–10.95) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.52× (0.45–0.60) | 0.70× (0.68–0.72) | 0.58× (0.57–0.60) | 5.55× (5.39–5.71) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.33× (0.32–0.34) | 0.59× (0.50–0.69) | 0.58× (0.56–0.60) | 0.51× (0.49–0.52) | 3.36× (3.25–3.48) | 0.08× (0.08–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.15–0.17) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.36× (4.35–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.68) | 0.66× (0.65–0.66) | 7.93× (7.92–7.95) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.73× (0.73–0.74) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.69) | 0.67× (0.62–0.72) | 1.59× (1.53–1.64) | 0.79× (0.76–0.82) | 5.75× (5.56–5.95) | 0.47× (0.45–0.49) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.58–0.66) | 0.70× (0.64–0.77) | 1.11× (1.06–1.17) | 0.68× (0.65–0.72) | 3.28× (3.12–3.44) | 0.26× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.53) | 0.42× (0.40–0.44) | 0.91× (0.88–0.94) | 0.73× (0.70–0.75) | 3.16× (3.07–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.45–0.46) | 0.87× (0.85–0.89) | 0.80× (0.78–0.83) | 2.52× (2.48–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.38× (0.36–0.41) | 0.86× (0.82–0.89) | 0.70× (0.67–0.73) | 2.94× (2.83–3.05) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.51) | 0.80× (0.80–0.81) | 0.80× (0.76–0.84) | 2.33× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.34× (5.33–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.76) | 5.26× (5.24–5.27) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.67× (0.66–0.67) | 6.42× (6.40–6.43) | 0.40× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.37× (0.34–0.41) | 0.98× (0.96–1.00) | 0.63× (0.59–0.66) | 3.08× (3.03–3.13) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 60 MB | 0.46× (0.45–0.48) | 0.46× (0.41–0.52) | 0.85× (0.83–0.88) | 0.68× (0.65–0.72) | 2.16× (2.09–2.24) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 418 | 2.40 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 718 | 2.64 | 1.10× (1.09–1.10) | 85% | 1.04× (1.03–1.04) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 872 | 4.11 | 1.70× (1.67–1.72) | 52% | 1.23× (1.22–1.24) | 1.63× (1.61–1.66) |

**Best:** mibz2 at 4 threads (872 MB/s); lbzip2 at 4 threads (191 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.66 | 0.02 | 0.09 | 0.240 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.86 | 12.8% |
| MTF + RLE2 | 1.15 | 17.1% |
| T-vector build | 1.15 | 17.2% |
| walk (the one each build uses) | 3.12 | 46.6% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.21 | 3.2% |
| CRC | 0.19 | 2.9% |
| **total** | 6.69 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.300 | (in its total) | 2.64× (2.63–2.64) |  |
| walk | 1.245 | 2.663 | (in its total) | 2.13× (2.12–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.20× (1.17–1.23) |  |
| inverse BWT, total | 1.784 | 4.018 | 3.276 | 2.25× (2.23–2.26) | 1.83× (1.82–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m18s |
| 4 threads | 35m00s ± 4m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m36s ± 3m21s |
| the scanner alone (its serial part) | 13m49s ± 32.9s |
| the workers alone, over 4 | 23m36s ± 3m21s |

At 4 threads the workers bound it. Per byte: the scanner 8.0 ns a byte of bz2, the workers 3.10 ns a byte of output at 4 threads (CPU, summed over them), 1.94 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
