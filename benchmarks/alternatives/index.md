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

The latest record: [`9116c6c`](<https://github.com/lmmx/mibz2/commit/9116c6cfe1be5397f874b9b82ed5e49aeb24280b>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42864127553>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.528 s | 397 | 6.2 | 2.88× (2.86–2.90) | 6.66× (6.57–6.75) | 6.87× (6.81–6.93) | 7.39× (7.34–7.45) | 6.67× (6.59–6.75) | 7.22× (7.16–7.29) | 6.72× (6.64–6.81) | 2.16× (2.15–2.18) | 2.50× (2.48–2.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.329 s | 638 | 7.0 | 2.69× (2.59–2.80) | 5.73× (5.39–6.09) | 6.64× (6.17–7.15) | 6.54× (6.09–7.04) | 5.85× (5.48–6.24) | 11.70× (10.96–12.48) | 10.89× (10.23–11.60) | 3.51× (3.29–3.73) | 4.06× (3.81–4.32) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.272 s | 771 | 11.5 | 2.70× (2.64–2.76) | 4.24× (4.13–4.35) | 5.20× (5.08–5.32) | 4.71× (4.63–4.80) | 4.82× (4.74–4.91) | 14.14× (13.93–14.34) | 13.17× (12.98–13.36) | 4.24× (4.18–4.30) | 4.90× (4.84–4.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 983 | 2.7 | 0.60× (0.59–0.61) | 3.80× (3.62–3.98) | 4.43× (4.35–4.51) | 3.58× (3.53–3.63) | 6.17× (6.08–6.26) | 2.99× (2.94–3.03) | 2.67× (2.62–2.71) | 1.02× (0.99–1.06) | 1.51× (1.49–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 2.8 | 0.57× (0.56–0.58) | 2.98× (2.93–3.02) | 5.96× (5.79–6.15) | 3.55× (3.50–3.59) | 6.05× (5.98–6.12) | 2.94× (2.90–2.99) | 2.63× (2.59–2.67) | 1.01× (0.99–1.03) | 1.49× (1.47–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 957 | 2.8 | 0.57× (0.56–0.58) | 2.94× (2.88–2.99) | 6.00× (5.84–6.17) | 3.48× (3.42–3.55) | 6.02× (5.89–6.14) | 2.89× (2.84–2.95) | 2.58× (2.53–2.63) | 0.99× (0.96–1.02) | 1.46× (1.43–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 584 | 4.4 | 2.64× (2.61–2.67) | 3.03× (3.00–3.06) | 3.97× (3.82–4.12) | 3.82× (3.79–3.86) | 4.15× (4.12–4.19) | 2.55× (2.53–2.56) | 2.12× (2.11–2.14) | 2.09× (2.07–2.12) | 3.01× (2.98–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 834 | 5.8 | 2.19× (2.09–2.30) | 2.61× (2.22–3.07) | 3.87× (3.42–4.37) | 3.08× (2.72–3.49) | 4.37× (3.84–4.97) | 3.48× (3.06–3.96) | 2.90× (2.55–3.30) | 2.86× (2.53–3.23) | 4.11× (3.61–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 767 | 10.8 | 1.80× (1.71–1.90) | 1.83× (1.66–2.02) | 2.93× (2.77–3.10) | 2.08× (1.99–2.17) | 3.83× (3.63–4.04) | 3.31× (3.15–3.47) | 2.76× (2.63–2.89) | 2.72× (2.60–2.85) | 3.91× (3.73–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 41.4 | 1.34× (1.33–1.35) | 2.69× (2.66–2.71) | 2.46× (2.38–2.55) | 2.44× (2.42–2.46) | 2.83× (2.80–2.85) | 3.12× (3.09–3.16) | 3.08× (3.05–3.10) | 1.23× (1.22–1.24) | 1.42× (1.40–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 46.6 | 1.26× (1.22–1.30) | 2.46× (2.40–2.51) | 2.32× (2.27–2.37) | 3.09× (2.91–3.29) | 2.87× (2.80–2.93) | 4.97× (4.84–5.09) | 4.89× (4.78–5.01) | 1.96× (1.91–2.01) | 2.26× (2.18–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 62.0 | 1.31× (1.27–1.36) | 2.17× (2.07–2.27) | 2.10× (2.04–2.17) | 2.60× (2.54–2.67) | 2.84× (2.76–2.92) | 6.03× (5.85–6.22) | 5.94× (5.78–6.11) | 2.38× (2.31–2.45) | 2.75× (2.65–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 37.6 | 1.43× (1.41–1.45) | 3.00× (2.95–3.06) | 2.55× (2.49–2.62) | 2.61× (2.57–2.65) | 3.04× (2.99–3.09) | 3.61× (3.54–3.68) | 3.80× (3.75–3.86) | 1.42× (1.40–1.43) | 1.72× (1.70–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 43.2 | 1.33× (1.28–1.38) | 2.78× (2.70–2.85) | 2.50× (2.43–2.57) | 3.36× (3.27–3.46) | 3.04× (2.94–3.13) | 5.93× (5.75–6.12) | 6.25× (6.07–6.44) | 2.33× (2.28–2.38) | 2.82× (2.75–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 122 | 60.6 | 1.36× (1.32–1.39) | 2.24× (2.17–2.31) | 2.22× (2.15–2.29) | 2.92× (2.84–2.99) | 2.75× (2.68–2.83) | 6.74× (6.52–6.95) | 7.10× (6.88–7.33) | 2.65× (2.53–2.76) | 3.21× (3.10–3.32) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 730 | 3.8 | 0.67× (0.66–0.68) | 3.16× (2.95–3.37) | 4.42× (4.36–4.48) | 3.12× (3.08–3.17) | 6.74× (6.65–6.83) | 2.53× (2.50–2.57) | 2.32× (2.29–2.35) | 0.98× (0.96–1.00) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 694 | 4.1 | 0.56× (0.55–0.57) | 2.86× (2.82–2.90) | 4.90× (4.84–4.96) | 2.99× (2.95–3.02) | 6.39× (6.29–6.48) | 2.39× (2.36–2.42) | 2.19× (2.16–2.23) | 0.93× (0.90–0.95) | 1.44× (1.42–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 4.0 | 0.57× (0.56–0.58) | 2.86× (2.82–2.90) | 4.88× (4.81–4.96) | 2.98× (2.93–3.04) | 6.38× (6.28–6.48) | 2.38× (2.35–2.41) | 2.18× (2.15–2.21) | 0.92× (0.91–0.93) | 1.43× (1.41–1.45) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.072 s | 179 | 14.0 | 1.93× (1.89–1.96) | 4.03× (3.96–4.11) | 4.39× (4.30–4.48) | 4.41× (4.32–4.51) | 4.60× (4.51–4.68) | 5.12× (5.02–5.22) | 4.63× (4.56–4.70) | 1.48× (1.46–1.51) | 1.85× (1.81–1.89) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.050 s | 257 | 17.4 | 1.58× (1.46–1.71) | 3.13× (2.95–3.32) | 3.85× (3.65–4.06) | 4.05× (3.82–4.31) | 4.08× (3.88–4.29) | 7.15× (6.81–7.50) | 6.46× (6.16–6.78) | 2.07× (1.97–2.18) | 2.59× (2.46–2.72) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.045 s | 282 | 26.6 | 1.44× (1.40–1.49) | 2.41× (2.32–2.50) | 3.00× (2.92–3.09) | 3.17× (3.11–3.22) | 3.58× (3.52–3.64) | 7.96× (7.81–8.12) | 7.20× (7.08–7.32) | 2.31× (2.26–2.35) | 2.88× (2.82–2.94) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.85–2.90) | 6.63× (6.54–6.72) | 6.84× (6.78–6.90) | 7.37× (7.31–7.43) | 6.64× (6.57–6.72) | 7.20× (7.13–7.26) | 6.70× (6.62–6.79) | 2.16× (2.14–2.18) | 2.49× (2.46–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.67× (2.62–2.71) | 6.03× (5.88–6.19) | 6.41× (6.21–6.63) | 6.98× (6.83–7.13) | 5.99× (5.83–6.16) | 6.35× (6.19–6.51) | 5.91× (5.76–6.07) | 1.91× (1.86–1.95) | 2.19× (2.15–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.58× (2.54–2.62) | 4.46× (4.36–4.56) | 4.86× (4.75–4.97) | 5.03× (4.95–5.11) | 4.56× (4.47–4.65) | 3.87× (3.81–3.93) | 3.60× (3.55–3.66) | 1.16× (1.14–1.18) | 1.34× (1.32–1.36) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.62–0.65) | 3.28× (3.23–3.33) | 4.09× (4.02–4.16) | 3.35× (3.29–3.40) | 5.70× (5.62–5.78) | 2.81× (2.76–2.85) | 2.52× (2.48–2.56) | 1.02× (0.99–1.05) | 1.07× (1.03–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.07× (3.03–3.11) | 5.38× (5.30–5.45) | 3.30× (3.26–3.35) | 5.53× (5.47–5.60) | 2.73× (2.69–2.77) | 2.45× (2.42–2.48) | 0.99× (0.97–1.02) | 1.04× (1.00–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.59–0.62) | 3.38× (3.30–3.46) | 5.39× (5.27–5.51) | 3.30× (3.24–3.36) | 5.58× (5.46–5.69) | 2.73× (2.68–2.78) | 2.45× (2.40–2.50) | 0.99× (0.96–1.03) | 1.04× (1.02–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.55× (2.52–2.58) | 2.79× (2.77–2.81) | 3.60× (3.48–3.72) | 3.66× (3.63–3.70) | 3.97× (3.94–4.01) | 2.46× (2.44–2.48) | 2.06× (2.04–2.08) | 2.03× (2.00–2.05) | 2.70× (2.66–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.17× (2.10–2.25) | 2.39× (2.30–2.48) | 2.96× (2.88–3.05) | 3.01× (2.93–3.09) | 3.22× (3.13–3.31) | 1.90× (1.84–1.96) | 1.59× (1.54–1.64) | 1.57× (1.52–1.61) | 2.08× (2.02–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.95–2.07) | 1.71× (1.67–1.76) | 2.20× (2.14–2.27) | 2.05× (1.99–2.12) | 2.10× (2.05–2.15) | 1.01× (0.98–1.04) | 0.84× (0.82–0.87) | 0.83× (0.81–0.85) | 1.10× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.33–1.35) | 2.62× (2.60–2.65) | 2.39× (2.31–2.47) | 2.42× (2.40–2.44) | 2.80× (2.78–2.82) | 3.09× (3.06–3.12) | 3.05× (3.02–3.07) | 1.23× (1.22–1.24) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.47× (2.43–2.50) | 2.17× (2.16–2.19) | 2.90× (2.86–2.94) | 2.55× (2.53–2.58) | 2.74× (2.70–2.79) | 2.70× (2.68–2.73) | 1.09× (1.07–1.10) | 1.22× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.46–1.53) | 2.34× (2.30–2.38) | 1.92× (1.87–1.98) | 2.62× (2.55–2.70) | 2.49× (2.42–2.55) | 2.05× (2.02–2.09) | 2.02× (1.99–2.06) | 0.82× (0.80–0.83) | 0.91× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.42× (1.40–1.45) | 2.96× (2.91–3.01) | 2.50× (2.45–2.56) | 2.59× (2.55–2.63) | 3.01× (2.97–3.06) | 3.57× (3.50–3.64) | 3.76× (3.71–3.82) | 1.41× (1.40–1.43) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.32–1.36) | 2.75× (2.72–2.79) | 2.30× (2.26–2.34) | 3.17× (3.14–3.21) | 2.72× (2.68–2.76) | 3.14× (3.09–3.20) | 3.31× (3.27–3.36) | 1.24× (1.22–1.26) | 1.46× (1.44–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.54–1.63) | 2.59× (2.49–2.69) | 1.99× (1.94–2.04) | 2.81× (2.72–2.90) | 2.57× (2.49–2.65) | 2.26× (2.21–2.31) | 2.38× (2.34–2.43) | 0.89× (0.87–0.91) | 1.05× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.73× (2.69–2.78) | 3.59× (3.53–3.65) | 2.85× (2.81–2.89) | 6.01× (5.92–6.09) | 2.33× (2.30–2.37) | 2.15× (2.11–2.18) | 0.98× (0.96–1.00) | 0.99× (0.95–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.51× (2.48–2.54) | 4.25× (4.18–4.32) | 2.75× (2.72–2.78) | 5.62× (5.55–5.70) | 2.17× (2.14–2.19) | 2.00× (1.97–2.02) | 0.91× (0.89–0.94) | 0.92× (0.88–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.77× (2.73–2.81) | 4.30× (4.24–4.36) | 2.81× (2.77–2.85) | 5.76× (5.67–5.86) | 2.21× (2.17–2.24) | 2.03× (2.01–2.06) | 0.93× (0.92–0.94) | 0.94× (0.90–0.97) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.90× (1.86–1.94) | 3.89× (3.83–3.95) | 4.24× (4.16–4.32) | 4.32× (4.23–4.41) | 4.50× (4.42–4.58) | 5.01× (4.91–5.10) | 4.53× (4.46–4.59) | 1.47× (1.45–1.49) | 1.77× (1.75–1.80) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.65× (1.61–1.70) | 3.26× (3.21–3.30) | 3.64× (3.59–3.69) | 3.99× (3.94–4.03) | 3.72× (3.66–3.77) | 3.99× (3.94–4.03) | 3.60× (3.56–3.64) | 1.17× (1.15–1.19) | 1.41× (1.40–1.43) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.69× (1.63–1.75) | 2.72× (2.60–2.85) | 2.93× (2.84–3.01) | 3.18× (3.13–3.24) | 3.03× (2.97–3.09) | 2.61× (2.56–2.66) | 2.36× (2.32–2.40) | 0.77× (0.75–0.78) | 0.93× (0.91–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.34) | 1.05× (1.05–1.06) | 0.84× (0.84–0.84) | 11.91× (11.90–11.92) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.53× (0.43–0.64) | 0.76× (0.75–0.76) | 0.63× (0.63–0.64) | 6.00× (5.97–6.04) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.38–0.38) | 0.67× (0.60–0.75) | 0.67× (0.67–0.69) | 0.59× (0.57–0.60) | 3.91× (3.89–3.94) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.67) | 8.03× (8.01–8.05) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.68× (0.65–0.71) | 1.61× (1.60–1.63) | 0.81× (0.80–0.81) | 5.85× (5.79–5.90) | 0.47× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.71× (0.65–0.77) | 1.14× (1.09–1.20) | 0.70× (0.68–0.73) | 3.38× (3.23–3.53) | 0.26× (0.25–0.28) | 0.28× (0.26–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.73) | 5.20× (5.20–5.21) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.42× (0.39–0.45) | 0.92× (0.89–0.95) | 0.72× (0.68–0.77) | 3.18× (3.07–3.28) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.62–0.67) | 0.45× (0.44–0.46) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.21–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.35–0.41) | 0.87× (0.84–0.90) | 0.70× (0.67–0.73) | 2.97× (2.87–3.07) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.60–0.63) | 0.49× (0.47–0.52) | 0.81× (0.80–0.83) | 0.81× (0.79–0.84) | 2.34× (2.29–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.08× (1.07–1.08) | 0.76× (0.76–0.76) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.30–5.33) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.65–0.67) | 6.42× (6.41–6.43) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.64× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.42× (0.39–0.45) | 1.14× (1.11–1.18) | 0.75× (0.73–0.76) | 3.60× (3.55–3.65) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.36× (0.35–0.36) | 0.18× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 53 MB | 0.52× (0.50–0.53) | 0.52× (0.48–0.56) | 0.94× (0.92–0.96) | 0.76× (0.73–0.78) | 2.40× (2.36–2.45) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 397 | 2.53 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 638 | 2.84 | 1.13× (1.11–1.16) | 81% | 1.03× (1.02–1.05) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 771 | 4.71 | 1.86× (1.83–1.89) | 49% | 1.25× (1.24–1.26) | 1.67× (1.65–1.69) |

**Best:** mibz2 at 4 threads (771 MB/s); lbzip2 at 4 threads (184 MB/s); pbz2 at 4 threads (291 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.18, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.78 | 0.02 | 0.09 | 0.272 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.86 | 12.7% |
| MTF + RLE2 | 1.14 | 16.9% |
| T-vector build | 1.17 | 17.4% |
| walk (the one each build uses) | 3.17 | 47.0% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.19 | 2.9% |
| **total** | 6.75 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.299 | (in its total) | 2.63× (2.63–2.64) |  |
| walk | 1.239 | 2.633 | (in its total) | 2.12× (2.12–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.056 | (in its total) | 1.21× (1.20–1.23) |  |
| inverse BWT, total | 1.778 | 3.988 | 3.271 | 2.24× (2.24–2.25) | 1.84× (1.83–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m43s |
| 4 threads | 39m16s ± 5m31s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m11s ± 3m59s |
| the scanner alone (its serial part) | 14m50s ± 26.3s |
| the workers alone, over 4 | 28m11s ± 3m59s |

At 4 threads the workers bound it. Per byte: the scanner 8.6 ns a byte of bz2, the workers 3.70 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,559,615 | 12,840,880 | \`c564b1e6883ea19b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
