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

The latest record: [`2e30e97`](<https://github.com/lmmx/mibz2/commit/2e30e97e6c0c7ea559fe92b90869b1354f776324>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44906943776>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.353 s | 594 | 6.4 | 2.84× (2.81–2.87) | 7.00× (6.85–7.15) | 7.31× (7.17–7.44) | 8.34× (8.16–8.53) | 7.44× (7.18–7.71) | 7.66× (7.46–7.86) | 7.24× (7.09–7.41) | 2.17× (2.14–2.21) | 2.50× (2.47–2.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.211 s | 995 | 7.1 | 2.95× (2.87–3.03) | 6.30× (6.09–6.52) | 7.66× (7.16–8.19) | 7.75× (7.45–8.07) | 6.84× (6.60–7.08) | 12.91× (12.48–13.34) | 12.21× (11.84–12.59) | 3.67× (3.56–3.77) | 4.22× (4.10–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,161 | 10.8 | 2.81× (2.69–2.93) | 4.57× (4.36–4.79) | 5.92× (5.64–6.22) | 5.51× (5.29–5.74) | 5.62× (5.39–5.85) | 15.15× (14.63–15.69) | 14.34× (13.86–14.82) | 4.30× (4.16–4.45) | 4.95× (4.79–5.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,231 | 3.4 | 0.58× (0.57–0.59) | 3.79× (3.69–3.89) | 4.56× (4.31–4.83) | 3.79× (3.70–3.87) | 6.61× (6.47–6.75) | 2.84× (2.78–2.89) | 2.85× (2.80–2.90) | 1.02× (1.00–1.05) | 2.46× (2.33–2.59) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,176 | 3.6 | 0.51× (0.49–0.52) | 3.01× (2.90–3.11) | 5.78× (5.63–5.94) | 3.58× (3.48–3.69) | 6.20× (6.05–6.36) | 2.69× (2.60–2.78) | 2.70× (2.62–2.78) | 0.97× (0.93–1.01) | 2.33× (2.20–2.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,176 | 3.5 | 0.52× (0.51–0.54) | 3.04× (2.99–3.10) | 5.78× (5.57–5.99) | 3.62× (3.57–3.67) | 6.34× (6.24–6.44) | 2.71× (2.68–2.75) | 2.73× (2.69–2.76) | 0.98× (0.95–1.01) | 2.35× (2.22–2.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 936 | 4.4 | 3.02× (2.98–3.06) | 3.60× (3.52–3.69) | 4.35× (4.22–4.49) | 4.61× (4.54–4.69) | 5.39× (5.32–5.47) | 3.06× (2.99–3.13) | 2.71× (2.65–2.76) | 2.18× (2.14–2.23) | 3.21× (3.08–3.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,181 | 6.0 | 2.05× (1.82–2.30) | 2.84× (2.47–3.27) | 3.80× (3.33–4.33) | 3.55× (2.90–4.33) | 5.12× (4.58–5.73) | 3.75× (3.34–4.21) | 3.32× (2.95–3.75) | 2.68× (2.37–3.02) | 3.94× (3.57–4.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,032 | 12.1 | 1.75× (1.66–1.85) | 1.72× (1.65–1.78) | 2.98× (2.83–3.15) | 2.09× (2.02–2.17) | 4.22× (4.07–4.37) | 3.41× (3.32–3.51) | 3.03× (2.94–3.12) | 2.44× (2.38–2.50) | 3.58× (3.43–3.74) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.095 s | 84 | 45.7 | 1.44× (1.41–1.48) | 3.09× (3.03–3.15) | 2.76× (2.70–2.81) | 2.71× (2.66–2.75) | 3.06× (3.01–3.11) | 3.48× (3.41–3.55) | 3.46× (3.39–3.52) | 1.33× (1.31–1.35) | 1.50× (1.46–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 52.4 | 1.36× (1.27–1.46) | 2.79× (2.73–2.85) | 2.60× (2.55–2.65) | 3.24× (3.15–3.34) | 3.10× (3.03–3.17) | 5.45× (5.36–5.54) | 5.42× (5.31–5.53) | 2.08× (2.05–2.11) | 2.34× (2.29–2.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 161 | 71.7 | 1.32× (1.28–1.36) | 2.44× (2.35–2.53) | 2.36× (2.29–2.44) | 2.84× (2.75–2.94) | 3.18× (3.10–3.25) | 6.66× (6.51–6.82) | 6.62× (6.49–6.76) | 2.54× (2.50–2.58) | 2.86× (2.79–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.101 s | 95 | 40.5 | 1.53× (1.50–1.55) | 3.61× (3.54–3.68) | 2.98× (2.93–3.04) | 3.01× (2.96–3.07) | 3.43× (3.36–3.50) | 4.34× (4.27–4.42) | 4.59× (4.53–4.66) | 1.57× (1.54–1.59) | 1.79× (1.77–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.064 s | 149 | 47.9 | 1.32× (1.28–1.37) | 3.19× (3.04–3.35) | 2.73× (2.65–2.80) | 3.70× (3.58–3.83) | 3.25× (3.16–3.35) | 6.71× (6.54–6.87) | 7.09× (6.89–7.29) | 2.42× (2.34–2.50) | 2.77× (2.68–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.054 s | 177 | 66.2 | 1.37× (1.34–1.41) | 2.75× (2.58–2.93) | 2.51× (2.45–2.57) | 3.35× (3.28–3.44) | 3.15× (3.05–3.25) | 8.09× (7.89–8.29) | 8.55× (8.35–8.76) | 2.92× (2.85–2.98) | 3.34× (3.26–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 918 | 4.8 | 0.62× (0.61–0.63) | 3.53× (3.26–3.82) | 4.23× (3.92–4.57) | 3.15× (3.08–3.21) | 7.03× (6.87–7.20) | 2.32× (2.27–2.36) | 2.35× (2.31–2.39) | 0.96× (0.94–0.98) | 1.88× (1.84–1.92) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 870 | 5.2 | 0.52× (0.51–0.52) | 2.70× (2.65–2.76) | 5.25× (5.04–5.46) | 3.06× (3.01–3.10) | 6.79× (6.68–6.90) | 2.22× (2.18–2.26) | 2.25× (2.22–2.29) | 0.92× (0.91–0.94) | 1.80× (1.76–1.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 848 | 5.1 | 0.52× (0.50–0.54) | 2.66× (2.60–2.72) | 5.15× (4.95–5.36) | 3.02× (2.98–3.06) | 6.61× (6.51–6.71) | 2.18× (2.15–2.21) | 2.21× (2.18–2.24) | 0.90× (0.89–0.92) | 1.77× (1.72–1.81) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.052 s | 257 | 15.2 | 1.78× (1.74–1.81) | 4.26× (4.19–4.34) | 4.59× (4.47–4.71) | 4.73× (4.63–4.84) | 4.99× (4.89–5.09) | 5.05× (4.95–5.14) | 4.83× (4.74–4.93) | 1.53× (1.50–1.56) | 1.78× (1.74–1.83) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.038 s | 352 | 19.7 | 1.48× (1.32–1.66) | 3.46× (3.18–3.77) | 3.97× (3.79–4.17) | 4.36× (3.93–4.82) | 4.39× (4.04–4.78) | 6.83× (6.37–7.32) | 6.54× (6.05–7.06) | 2.07× (1.92–2.23) | 2.41× (2.26–2.57) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.034 s | 398 | 29.7 | 1.38× (1.33–1.42) | 2.67× (2.49–2.86) | 3.57× (3.28–3.89) | 3.59× (3.35–3.86) | 4.14× (3.92–4.36) | 7.74× (7.48–8.00) | 7.41× (7.22–7.60) | 2.35× (2.28–2.43) | 2.73× (2.65–2.81) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.83× (2.80–2.86) | 6.96× (6.82–7.10) | 7.26× (7.13–7.39) | 8.30× (8.12–8.49) | 7.41× (7.15–7.67) | 7.62× (7.42–7.82) | 7.21× (7.05–7.37) | 2.17× (2.13–2.20) | 2.48× (2.44–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.73× (2.69–2.77) | 6.56× (6.49–6.63) | 7.16× (6.80–7.55) | 8.06× (7.94–8.18) | 6.84× (6.74–6.95) | 6.89× (6.82–6.96) | 6.52× (6.45–6.59) | 1.96× (1.93–1.99) | 2.24× (2.22–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.86× (2.78–2.94) | 5.22× (5.05–5.40) | 5.86× (5.72–5.99) | 6.28× (6.12–6.44) | 5.61× (5.45–5.76) | 4.46× (4.38–4.55) | 4.22× (4.13–4.32) | 1.27× (1.24–1.30) | 1.45× (1.42–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.14× (3.09–3.19) | 4.07× (3.91–4.23) | 3.51× (3.44–3.58) | 6.05× (5.94–6.16) | 2.66× (2.61–2.70) | 2.67× (2.63–2.71) | 1.03× (1.01–1.05) | 1.87× (1.84–1.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.78× (2.72–2.84) | 4.91× (4.79–5.04) | 3.30× (3.21–3.39) | 5.61× (5.48–5.73) | 2.48× (2.41–2.55) | 2.49× (2.42–2.56) | 0.96× (0.93–0.99) | 1.75× (1.69–1.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.55–0.58) | 3.08× (3.03–3.13) | 5.06× (4.97–5.15) | 3.41× (3.36–3.45) | 5.84× (5.76–5.92) | 2.57× (2.54–2.59) | 2.58× (2.55–2.61) | 0.99× (0.96–1.02) | 1.81× (1.77–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.86× (2.82–2.90) | 3.21× (3.15–3.26) | 3.84× (3.79–3.90) | 4.32× (4.25–4.39) | 5.04× (4.97–5.11) | 2.89× (2.82–2.96) | 2.57× (2.52–2.62) | 2.09× (2.04–2.13) | 2.81× (2.77–2.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.23–2.33) | 2.61× (2.53–2.69) | 3.09× (3.03–3.15) | 3.38× (3.32–3.44) | 3.87× (3.79–3.95) | 2.13× (2.08–2.18) | 1.89× (1.85–1.94) | 1.54× (1.51–1.57) | 2.07× (2.03–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.87× (1.76–1.99) | 1.68× (1.59–1.78) | 2.05× (1.97–2.14) | 2.09× (2.00–2.18) | 2.23× (2.14–2.32) | 1.08× (1.04–1.13) | 0.96× (0.93–1.00) | 0.78× (0.75–0.81) | 1.05× (1.01–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.43× (1.40–1.47) | 3.00× (2.95–3.04) | 2.67× (2.63–2.71) | 2.67× (2.63–2.72) | 3.02× (2.97–3.07) | 3.43× (3.36–3.50) | 3.41× (3.35–3.47) | 1.32× (1.30–1.34) | 1.44× (1.41–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.29–1.33) | 2.74× (2.69–2.79) | 2.36× (2.33–2.40) | 3.08× (3.04–3.12) | 2.71× (2.66–2.77) | 2.97× (2.92–3.03) | 2.96× (2.90–3.01) | 1.14× (1.13–1.16) | 1.24× (1.22–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.38× (1.32–1.44) | 2.55× (2.50–2.60) | 2.11× (2.07–2.15) | 2.78× (2.71–2.86) | 2.62× (2.59–2.65) | 2.20× (2.15–2.24) | 2.18× (2.15–2.22) | 0.84× (0.83–0.85) | 0.92× (0.91–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.49–1.54) | 3.53× (3.47–3.59) | 2.92× (2.87–2.97) | 2.98× (2.92–3.03) | 3.38× (3.32–3.45) | 4.28× (4.21–4.36) | 4.53× (4.46–4.59) | 1.56× (1.53–1.58) | 1.73× (1.70–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.30–1.38) | 3.17× (3.09–3.26) | 2.51× (2.45–2.57) | 3.51× (3.39–3.62) | 2.91× (2.85–2.98) | 3.58× (3.49–3.67) | 3.78× (3.69–3.88) | 1.30× (1.27–1.34) | 1.44× (1.40–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.49–1.55) | 2.94× (2.83–3.06) | 2.19× (2.13–2.26) | 3.10× (3.03–3.17) | 2.78× (2.73–2.84) | 2.63× (2.56–2.71) | 2.78× (2.71–2.86) | 0.96× (0.93–0.98) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.56× (2.50–2.62) | 3.36× (3.29–3.43) | 2.83× (2.78–2.88) | 6.15× (6.01–6.29) | 2.13× (2.09–2.16) | 2.15× (2.12–2.18) | 0.97× (0.95–0.99) | 1.49× (1.46–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.61) | 2.35× (2.30–2.40) | 4.03× (3.95–4.11) | 2.80× (2.76–2.84) | 5.89× (5.79–5.99) | 2.01× (1.97–2.05) | 2.03× (2.00–2.07) | 0.92× (0.90–0.93) | 1.41× (1.39–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.60) | 2.50× (2.45–2.56) | 4.11× (3.99–4.23) | 2.84× (2.80–2.88) | 5.94× (5.85–6.03) | 2.05× (2.01–2.08) | 2.07× (2.04–2.10) | 0.93× (0.92–0.95) | 1.44× (1.41–1.47) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.75× (1.72–1.79) | 4.05× (3.99–4.11) | 4.37× (4.27–4.47) | 4.60× (4.50–4.70) | 4.85× (4.75–4.94) | 4.90× (4.81–5.00) | 4.70× (4.61–4.79) | 1.51× (1.48–1.54) | 1.64× (1.61–1.66) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.48× (1.45–1.51) | 3.41× (3.23–3.60) | 3.66× (3.58–3.74) | 4.19× (4.04–4.35) | 3.96× (3.74–4.19) | 3.80× (3.68–3.91) | 3.64× (3.52–3.75) | 1.17× (1.13–1.22) | 1.27× (1.22–1.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.52× (1.47–1.57) | 2.79× (2.62–2.97) | 3.28× (2.96–3.64) | 3.56× (3.37–3.77) | 3.34× (3.12–3.58) | 2.50× (2.44–2.56) | 2.39× (2.33–2.46) | 0.77× (0.75–0.79) | 0.83× (0.81–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.32× (0.31–0.33) | 0.97× (0.97–0.97) | 0.78× (0.77–0.79) | 10.95× (10.94–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.34) | 0.46× (0.39–0.54) | 0.67× (0.64–0.70) | 0.56× (0.54–0.58) | 5.32× (5.12–5.54) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.31–0.34) | 0.51× (0.40–0.64) | 0.58× (0.55–0.61) | 0.51× (0.49–0.53) | 3.39× (3.24–3.54) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.93–7.98) | 0.64× (0.63–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.62–0.68) | 0.66× (0.60–0.73) | 1.56× (1.50–1.64) | 0.78× (0.75–0.82) | 5.67× (5.43–5.93) | 0.45× (0.43–0.47) | 0.48× (0.46–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.63× (0.59–0.67) | 0.65× (0.61–0.69) | 1.12× (1.09–1.16) | 0.69× (0.66–0.71) | 3.27× (3.14–3.41) | 0.25× (0.24–0.26) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.51) | 0.41× (0.40–0.43) | 0.89× (0.86–0.92) | 0.71× (0.67–0.74) | 3.07× (2.96–3.19) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.67) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.79–0.83) | 2.51× (2.44–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.16× (5.15–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.39) | 0.84× (0.80–0.87) | 0.70× (0.66–0.74) | 2.87× (2.76–2.98) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.47–0.52) | 0.80× (0.79–0.81) | 0.81× (0.78–0.84) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.35× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.21) | 1.59× (1.58–1.59) | 0.76× (0.76–0.76) | 5.26× (5.24–5.28) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.66–0.67) | 6.35× (6.33–6.36) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.35–0.43) | 1.03× (1.00–1.05) | 0.65× (0.63–0.67) | 3.29× (3.27–3.32) | 0.20× (0.20–0.20) | 0.22× (0.22–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.45× (0.41–0.48) | 0.90× (0.88–0.92) | 0.74× (0.71–0.76) | 2.28× (2.23–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 594 | 1.69 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 995 | 1.87 | 1.11× (1.08–1.13) | 84% | 1.04× (1.02–1.07) | 1.07× (1.04–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 1,161 | 2.86 | 1.71× (1.67–1.75) | 49% | 1.28× (1.22–1.34) | 1.72× (1.65–1.80) |

**Best:** mibz2 at 4 threads (1,161 MB/s); lbzip2 at 4 threads (259 MB/s); pbz2 at 4 threads (420 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.23, lbzip2 1t 0.08, lbzip2 2t 0.28, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.47, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.45 | 0.01 | 0.08 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3781 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 43.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 14.2% |
| MTF + RLE2 | 1.44 | 20.2% |
| T-vector build | 1.30 | 18.3% |
| walk (the one each build uses) | 2.96 | 41.5% |
| of which: segment assembly | 0.13 | 1.8% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.25 | 3.6% |
| **total** | 7.13 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.343 | 0.824 | (in its total) | 2.41× (2.39–2.42) |  |
| walk | 0.742 | 1.831 | (in its total) | 2.46× (2.40–2.52) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.035 | 0.036 | (in its total) | 1.01× (0.96–1.07) |  |
| inverse BWT, total | 1.118 | 2.689 | 1.506 | 2.40× (2.35–2.44) | 1.36× (1.33–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 51m21s ± 5m54s |
| 4 threads | 25m58s ± 3m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 16m11s ± 2m21s |
| the scanner alone (its serial part) | 9m45s ± 42.1s |
| the workers alone, over 4 | 16m11s ± 2m21s |

At 4 threads the workers bound it. Per byte: the scanner 5.7 ns a byte of bz2, the workers 2.12 ns a byte of output at 4 threads (CPU, summed over them), 1.36 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,147 | 13,382,859 | \`f7d4e4f1aa08d971…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 3781.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
