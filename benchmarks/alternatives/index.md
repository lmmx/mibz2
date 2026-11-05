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

The latest record: [`e26e3d9`](<https://github.com/lmmx/mibz2/commit/e26e3d9cbac6bcd2106b7070c869dd91fcd7217d>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41942278817>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.710 s | 295 | 11.7 | 1.99× (1.98–2.01) | 4.43× (4.39–4.48) | 4.72× (4.68–4.77) | 5.31× (5.26–5.36) | 4.76× (4.71–4.82) | 5.06× (5.01–5.11) | 4.94× (4.89–4.99) | 1.55× (1.53–1.57) | 1.43× (1.41–1.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.450 s | 466 | 12.6 | 1.83× (1.73–1.94) | 3.91× (3.71–4.11) | 4.52× (4.30–4.74) | 4.54× (4.36–4.73) | 4.06× (3.88–4.24) | 7.94× (7.58–8.32) | 7.75× (7.40–8.11) | 2.43× (2.31–2.56) | 2.24× (2.14–2.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.340 s | 616 | 18.2 | 2.06× (2.02–2.10) | 3.60× (3.53–3.68) | 4.12× (4.05–4.19) | 4.09× (4.00–4.19) | 4.04× (3.96–4.12) | 10.60× (10.40–10.81) | 10.35× (10.14–10.56) | 3.24× (3.18–3.31) | 2.99× (2.93–3.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,018 | 3.7 | 0.55× (0.54–0.56) | 3.13× (3.05–3.20) | 4.17× (4.06–4.28) | 3.57× (3.44–3.71) | 6.14× (5.83–6.46) | 2.63× (2.57–2.70) | 2.65× (2.59–2.71) | 1.10× (1.07–1.13) | 1.05× (1.03–1.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,013 | 3.7 | 0.50× (0.47–0.53) | 2.81× (2.58–3.07) | 5.59× (5.28–5.93) | 3.45× (3.28–3.64) | 5.79× (5.52–6.08) | 2.54× (2.41–2.69) | 2.56× (2.38–2.74) | 1.06× (1.01–1.12) | 1.02× (0.97–1.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,015 | 3.7 | 0.57× (0.50–0.65) | 3.06× (2.93–3.20) | 5.75× (5.60–5.91) | 3.56× (3.47–3.66) | 6.16× (5.98–6.34) | 2.62× (2.52–2.71) | 2.63× (2.54–2.73) | 1.09× (1.06–1.13) | 1.05× (1.02–1.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.040 s | 492 | 7.3 | 2.37× (2.30–2.44) | 2.84× (2.72–2.97) | 3.32× (3.25–3.40) | 3.71× (3.62–3.79) | 4.12× (4.00–4.24) | 2.71× (2.63–2.79) | 2.49× (2.39–2.59) | 1.78× (1.73–1.83) | 1.80× (1.72–1.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.027 s | 738 | 8.7 | 1.94× (1.68–2.25) | 2.39× (1.92–2.96) | 3.38× (2.83–4.05) | 3.14× (2.56–3.86) | 4.14× (3.56–4.81) | 3.88× (3.32–4.53) | 3.57× (3.04–4.19) | 2.55× (2.19–2.96) | 2.58× (2.20–3.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 789 | 15.6 | 1.89× (1.80–1.99) | 1.83× (1.66–2.01) | 2.84× (2.62–3.08) | 2.18× (2.10–2.26) | 3.92× (3.75–4.10) | 4.40× (4.22–4.59) | 4.04× (3.85–4.24) | 2.89× (2.77–3.01) | 2.93× (2.79–3.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.136 s | 59 | 59.4 | 1.27× (1.24–1.30) | 2.60× (2.56–2.64) | 2.30× (2.26–2.34) | 2.42× (2.40–2.45) | 2.69× (2.64–2.73) | 3.03× (2.99–3.07) | 3.19× (3.13–3.26) | 1.22× (1.20–1.24) | 1.24× (1.21–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 100 | 63.5 | 1.22× (1.16–1.29) | 2.60× (2.36–2.87) | 2.44× (2.29–2.60) | 3.14× (3.02–3.28) | 2.93× (2.75–3.12) | 5.08× (4.82–5.36) | 5.35× (5.07–5.66) | 2.05× (1.94–2.17) | 2.08× (1.98–2.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.064 s | 124 | 84.0 | 1.37× (1.29–1.47) | 2.47× (2.33–2.62) | 2.39× (2.24–2.56) | 3.04× (2.84–3.25) | 2.93× (2.76–3.10) | 6.31× (5.96–6.67) | 6.65× (6.29–7.03) | 2.55× (2.41–2.68) | 2.58× (2.44–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.171 s | 56 | 62.1 | 1.14× (1.03–1.25) | 2.52× (2.33–2.74) | 2.09× (1.93–2.26) | 2.24× (2.06–2.43) | 2.50× (2.30–2.72) | 3.11× (2.86–3.37) | 3.35× (3.08–3.64) | 1.22× (1.12–1.32) | 1.24× (1.14–1.34) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.099 s | 97 | 66.3 | 1.12× (1.06–1.17) | 2.66× (2.37–2.99) | 2.29× (2.17–2.41) | 3.17× (3.05–3.29) | 2.71× (2.63–2.80) | 5.52× (5.27–5.78) | 5.95× (5.70–6.21) | 2.16× (2.06–2.27) | 2.20× (2.10–2.29) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 125 | 87.5 | 1.36× (1.32–1.41) | 2.55× (2.47–2.64) | 2.30× (2.20–2.40) | 3.21× (3.09–3.33) | 2.85× (2.77–2.92) | 7.18× (7.03–7.32) | 7.74× (7.53–7.95) | 2.81× (2.73–2.90) | 2.86× (2.80–2.91) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 781 | 4.9 | 0.62× (0.60–0.64) | 3.20× (3.15–3.24) | 4.01× (3.94–4.07) | 3.10× (3.03–3.17) | 6.66× (6.54–6.78) | 2.30× (2.23–2.38) | 2.38× (2.31–2.44) | 1.03× (1.00–1.06) | 0.88× (0.76–1.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 744 | 5.3 | 0.52× (0.48–0.57) | 2.30× (2.24–2.37) | 4.81× (4.56–5.06) | 3.02× (2.92–3.11) | 6.41× (6.18–6.64) | 2.19× (2.13–2.24) | 2.26× (2.16–2.37) | 0.98× (0.93–1.03) | 0.84× (0.73–0.97) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 707 | 5.4 | 0.52× (0.49–0.55) | 2.64× (2.33–2.98) | 4.63× (4.34–4.94) | 2.97× (2.84–3.10) | 6.10× (5.83–6.38) | 2.11× (2.02–2.20) | 2.18× (2.10–2.26) | 0.94× (0.90–0.99) | 0.81× (0.69–0.95) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 0.077 s | 162 | 21.8 | 1.39× (1.37–1.42) | 3.15× (3.09–3.22) | 3.41× (3.36–3.47) | 3.57× (3.51–3.64) | 3.76× (3.68–3.85) | 3.65× (3.60–3.69) | 3.75× (3.72–3.77) | 1.14× (1.12–1.16) | 1.17× (1.13–1.21) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 0.046 s | 269 | 24.7 | 1.33× (1.12–1.59) | 2.85× (2.64–3.08) | 3.65× (3.26–4.08) | 3.80× (3.58–4.04) | 3.86× (3.63–4.10) | 5.88× (5.51–6.28) | 6.04× (5.64–6.48) | 1.83× (1.71–1.97) | 1.89× (1.75–2.03) | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 0.039 s | 320 | 36.9 | 1.42× (1.38–1.46) | 2.57× (2.50–2.65) | 3.07× (2.92–3.24) | 3.57× (3.40–3.76) | 3.73× (3.59–3.87) | 7.20× (6.85–7.57) | 7.41× (7.09–7.73) | 2.25× (2.15–2.35) | 2.31× (2.18–2.45) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.99× (1.98–2.00) | 4.42× (4.37–4.46) | 4.71× (4.66–4.75) | 5.30× (5.25–5.35) | 4.76× (4.70–4.81) | 5.05× (5.00–5.10) | 4.93× (4.88–4.98) | 1.55× (1.53–1.56) | 1.42× (1.40–1.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.00× (1.94–2.06) | 4.42× (4.33–4.51) | 4.83× (4.72–4.94) | 5.38× (5.28–5.48) | 4.60× (4.50–4.71) | 4.74× (4.65–4.84) | 4.63× (4.54–4.72) | 1.45× (1.41–1.50) | 1.33× (1.30–1.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.25× (2.20–2.31) | 4.38× (4.31–4.45) | 4.53× (4.46–4.59) | 4.99× (4.90–5.08) | 4.46× (4.39–4.52) | 3.29× (3.23–3.34) | 3.21× (3.15–3.26) | 1.01× (0.99–1.02) | 0.92× (0.91–0.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.60) | 2.75× (2.69–2.82) | 3.70× (3.61–3.78) | 3.38× (3.24–3.52) | 5.75× (5.46–6.05) | 2.51× (2.45–2.57) | 2.52× (2.46–2.58) | 1.10× (1.06–1.13) | 0.85× (0.83–0.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.51–0.57) | 2.62× (2.46–2.78) | 4.87× (4.63–5.12) | 3.25× (3.08–3.43) | 5.38× (5.13–5.65) | 2.40× (2.27–2.53) | 2.41× (2.25–2.59) | 1.05× (0.99–1.10) | 0.81× (0.76–0.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.53–0.69) | 3.21× (3.09–3.34) | 5.06× (4.90–5.23) | 3.40× (3.31–3.49) | 5.78× (5.61–5.95) | 2.50× (2.42–2.59) | 2.52× (2.43–2.60) | 1.09× (1.06–1.13) | 0.84× (0.81–0.88) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.32× (2.25–2.38) | 2.69× (2.57–2.81) | 3.14× (3.07–3.22) | 3.60× (3.52–3.69) | 4.01× (3.89–4.12) | 2.64× (2.57–2.72) | 2.43× (2.34–2.53) | 1.75× (1.70–1.80) | 1.66× (1.60–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.04× (1.93–2.16) | 2.32× (2.15–2.50) | 2.79× (2.67–2.92) | 3.02× (2.86–3.19) | 3.36× (3.18–3.55) | 2.17× (2.06–2.29) | 2.00× (1.88–2.13) | 1.44× (1.35–1.52) | 1.36× (1.27–1.46) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.87–2.12) | 1.66× (1.57–1.75) | 2.03× (1.94–2.13) | 2.01× (1.91–2.11) | 2.14× (2.03–2.26) | 1.26× (1.20–1.31) | 1.16× (1.10–1.21) | 0.83× (0.80–0.87) | 0.79× (0.75–0.83) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.27× (1.24–1.30) | 2.55× (2.51–2.59) | 2.25× (2.22–2.28) | 2.40× (2.38–2.43) | 2.67× (2.62–2.71) | 3.01× (2.97–3.05) | 3.17× (3.11–3.23) | 1.22× (1.20–1.24) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.22–1.26) | 2.56× (2.51–2.62) | 2.29× (2.20–2.38) | 3.03× (2.97–3.08) | 2.68× (2.59–2.77) | 2.81× (2.77–2.85) | 2.96× (2.90–3.01) | 1.14× (1.12–1.16) | 1.12× (1.10–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.44–1.62) | 2.68× (2.57–2.79) | 2.24× (2.14–2.35) | 2.93× (2.78–3.10) | 2.61× (2.49–2.74) | 2.09× (2.00–2.20) | 2.21× (2.09–2.33) | 0.85× (0.81–0.89) | 0.83× (0.80–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.13× (1.03–1.24) | 2.47× (2.28–2.68) | 2.06× (1.90–2.23) | 2.23× (2.05–2.42) | 2.49× (2.29–2.70) | 3.09× (2.85–3.35) | 3.33× (3.07–3.62) | 1.22× (1.12–1.32) | 1.20× (1.11–1.30) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.15× (1.13–1.18) | 2.62× (2.53–2.71) | 2.17× (2.11–2.23) | 3.02× (2.97–3.07) | 2.51× (2.47–2.56) | 2.99× (2.93–3.05) | 3.22× (3.15–3.29) | 1.18× (1.15–1.20) | 1.16× (1.14–1.19) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.49× (1.42–1.57) | 2.79× (2.73–2.84) | 2.08× (2.04–2.13) | 2.98× (2.87–3.09) | 2.59× (2.54–2.63) | 2.27× (2.24–2.31) | 2.45× (2.41–2.50) | 0.90× (0.88–0.91) | 0.89× (0.87–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.69) | 2.36× (2.31–2.40) | 3.35× (3.26–3.44) | 2.88× (2.83–2.94) | 6.07× (5.98–6.17) | 2.17× (2.10–2.24) | 2.23× (2.17–2.30) | 1.03× (1.01–1.06) | 0.79× (0.77–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.54–0.62) | 2.15× (2.06–2.23) | 4.04× (3.93–4.16) | 2.82× (2.74–2.90) | 5.77× (5.57–5.97) | 2.03× (1.98–2.08) | 2.09× (2.00–2.18) | 0.96× (0.92–1.01) | 0.74× (0.72–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.54–0.59) | 2.50× (2.38–2.63) | 4.04× (3.79–4.30) | 2.83× (2.73–2.94) | 5.60× (5.36–5.84) | 1.99× (1.91–2.07) | 2.05× (1.99–2.12) | 0.95× (0.91–0.99) | 0.73× (0.69–0.77) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 1.39× (1.36–1.41) | 3.05× (3.00–3.10) | 3.32× (3.28–3.38) | 3.52× (3.46–3.58) | 3.71× (3.63–3.79) | 3.59× (3.55–3.64) | 3.69× (3.67–3.72) | 1.13× (1.12–1.15) | 1.08× (1.05–1.10) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 1.31× (1.24–1.39) | 2.84× (2.68–3.01) | 3.27× (3.10–3.44) | 3.53× (3.38–3.69) | 3.39× (3.21–3.58) | 3.13× (2.98–3.30) | 3.22× (3.06–3.39) | 0.99× (0.94–1.04) | 0.94× (0.90–0.99) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 1.57× (1.52–1.63) | 2.68× (2.62–2.74) | 2.97× (2.88–3.06) | 3.12× (3.02–3.22) | 3.02× (2.95–3.09) | 2.15× (2.07–2.22) | 2.21× (2.15–2.27) | 0.68× (0.66–0.70) | 0.64× (0.63–0.66) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.29× (0.29–0.29) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.24× (12.23–12.26) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 45 MB | 0.41× (0.40–0.42) | 0.63× (0.54–0.73) | 0.86× (0.84–0.88) | 0.71× (0.70–0.73) | 6.91× (6.78–7.03) | 0.16× (0.16–0.16) | 0.17× (0.17–0.18) | 0.26× (0.26–0.27) | 0.13× (0.13–0.14) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 66 MB | 0.44× (0.43–0.45) | 0.61× (0.54–0.67) | 0.80× (0.79–0.82) | 0.70× (0.68–0.73) | 4.74× (4.67–4.82) | 0.11× (0.11–0.11) | 0.12× (0.11–0.12) | 0.18× (0.17–0.18) | 0.09× (0.09–0.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.85) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.42–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.19× (8.17–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.38× (0.38–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.64× (0.59–0.70) | 1.66× (1.64–1.68) | 0.82× (0.81–0.83) | 6.04× (5.96–6.11) | 0.48× (0.47–0.48) | 0.50× (0.49–0.51) | 0.55× (0.54–0.56) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.64–0.66) | 0.68× (0.63–0.74) | 1.19× (1.16–1.21) | 0.78× (0.75–0.80) | 3.53× (3.47–3.60) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.73–0.74) | 5.29× (5.28–5.30) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 35 MB | 0.53× (0.51–0.55) | 0.41× (0.38–0.45) | 0.98× (0.95–1.02) | 0.77× (0.73–0.81) | 3.39× (3.28–3.51) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.34× (0.32–0.35) | 0.17× (0.17–0.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.61× (0.60–0.63) | 0.46× (0.45–0.47) | 0.87× (0.86–0.88) | 0.79× (0.76–0.81) | 2.52× (2.49–2.56) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.83× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 37 MB | 0.52× (0.51–0.53) | 0.41× (0.38–0.45) | 0.95× (0.95–0.96) | 0.80× (0.78–0.82) | 3.26× (3.25–3.28) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.32) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.51) | 0.82× (0.81–0.83) | 0.84× (0.81–0.87) | 2.36× (2.33–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.47× (5.47–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.78× (0.77–0.78) | 5.45× (5.43–5.46) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.78× (0.77–0.78) | 5.47× (5.45–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.40× (0.40–0.40) | 1.50× (1.50–1.51) | 0.64× (0.64–0.64) | 6.78× (6.76–6.81) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.34) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 30 MB | 0.53× (0.52–0.53) | 0.45× (0.40–0.50) | 1.29× (1.27–1.31) | 0.80× (0.79–0.81) | 4.07× (4.03–4.12) | 0.24× (0.24–0.24) | 0.26× (0.25–0.26) | 0.40× (0.39–0.40) | 0.20× (0.20–0.20) |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 49 MB | 0.55× (0.54–0.56) | 0.49× (0.45–0.54) | 1.04× (1.02–1.06) | 0.83× (0.82–0.85) | 2.66× (2.63–2.69) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 295 | 3.39 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 466 | 3.63 | 1.07× (1.04–1.09) | 78% | 1.07× (1.06–1.08) | 1.07× (1.05–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 616 | 5.25 | 1.54× (1.51–1.56) | 52% | 1.53× (1.52–1.53) | 1.74× (1.71–1.78) |

**Best:** mibz2 at 4 threads (616 MB/s); lbzip2 at 4 threads (172 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.00, lbzip2 1t 0.04, lbzip2 2t 0.29, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.35, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.89 | 0.02 | 0.09 | 0.340 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3460 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.25 | 11.1% |
| MTF + RLE2 | 2.19 | 19.3% |
| T-vector build | 1.71 | 15.2% |
| walk (the one each build uses) | 5.77 | 51.0% |
| of which: segment assembly | 0.29 | 2.5% |
| RLE1 | 0.27 | 2.4% |
| CRC | 0.14 | 1.2% |
| **total** | 11.31 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.504 | 1.172 | (in its total) | 2.31× (2.26–2.36) |  |
| walk | 1.605 | 3.053 | (in its total) | 1.89× (1.86–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.083 | 0.077 | (in its total) | 0.92× (0.90–0.94) |  |
| inverse BWT, total | 2.190 | 4.303 | 2.214 | 1.95× (1.92–1.98) | 1.00× (0.98–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h43m ± 12m38s |
| 4 threads | 49m37s ± 7m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m25s ± 4m34s |
| the scanner alone (its serial part) | 14m33s ± 29.4s |
| the workers alone, over 4 | 32m25s ± 4m34s |

At 4 threads the workers bound it. Per byte: the scanner 8.4 ns a byte of bz2, the workers 4.25 ns a byte of output at 4 threads (CPU, summed over them), 2.93 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,690,460 | 12,385,319 | \`42f88d7c5a17e825…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3459.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
