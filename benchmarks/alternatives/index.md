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

The latest record: [`fa800c9`](<https://github.com/lmmx/mibz2/commit/fa800c98d6a0c074be5ce8cea41eb797fb880284>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44850659464>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.518 s | 404 | 7.0 | 3.05× (3.03–3.06) | 6.57× (6.53–6.60) | 7.58× (7.55–7.62) | 7.80× (7.77–7.82) | 7.12× (7.08–7.17) | 7.96× (7.90–8.02) | 7.19× (7.13–7.24) | 2.23× (2.21–2.24) | 2.24× (2.22–2.26) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.302 s | 695 | 7.6 | 2.92× (2.73–3.12) | 5.85× (5.52–6.20) | 7.58× (7.17–8.01) | 7.15× (6.81–7.50) | 6.45× (6.11–6.81) | 13.44× (12.80–14.11) | 12.13× (11.54–12.75) | 3.76× (3.57–3.95) | 3.78× (3.60–3.98) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.259 s | 809 | 12.6 | 3.33× (3.19–3.47) | 4.47× (4.38–4.56) | 6.01× (5.84–6.18) | 5.37× (5.28–5.47) | 5.31× (5.22–5.41) | 15.86× (15.56–16.15) | 14.31× (14.09–14.54) | 4.43× (4.36–4.51) | 4.46× (4.38–4.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,028 | 3.0 | 0.65× (0.64–0.67) | 3.12× (3.05–3.20) | 5.63× (5.50–5.77) | 3.60× (3.52–3.69) | 6.55× (6.40–6.70) | 2.79× (2.72–2.86) | 2.75× (2.68–2.82) | 1.16× (1.11–1.21) | 1.06× (1.03–1.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 980 | 3.1 | 0.62× (0.61–0.64) | 3.01× (2.94–3.09) | 6.93× (6.75–7.11) | 3.52× (3.44–3.61) | 6.29× (6.14–6.45) | 2.68× (2.62–2.75) | 2.65× (2.58–2.71) | 1.12× (1.06–1.18) | 1.02× (0.99–1.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 973 | 3.1 | 0.63× (0.61–0.65) | 3.02× (2.95–3.10) | 6.94× (6.77–7.12) | 3.53× (3.43–3.62) | 6.42× (6.24–6.60) | 2.69× (2.62–2.75) | 2.65× (2.58–2.72) | 1.12× (1.09–1.15) | 1.02× (0.99–1.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 556 | 5.3 | 3.11× (3.09–3.13) | 3.46× (3.43–3.48) | 4.57× (4.54–4.61) | 4.31× (4.28–4.34) | 4.94× (4.90–4.98) | 3.96× (3.93–3.98) | 3.08× (3.05–3.10) | 2.17× (2.14–2.19) | 2.30× (2.29–2.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 848 | 6.4 | 2.55× (2.46–2.65) | 2.95× (2.71–3.20) | 4.72× (4.43–5.04) | 3.72× (3.50–3.96) | 5.31× (5.04–5.59) | 5.92× (5.60–6.25) | 4.60× (4.36–4.86) | 3.24× (3.07–3.42) | 3.45× (3.27–3.64) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 794 | 12.5 | 2.37× (2.28–2.47) | 2.03× (1.90–2.18) | 3.45× (3.30–3.60) | 2.40× (2.31–2.50) | 4.51× (4.34–4.68) | 5.64× (5.44–5.85) | 4.39× (4.24–4.54) | 3.09× (2.98–3.20) | 3.28× (3.17–3.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.150 s | 53 | 53.2 | 1.34× (1.31–1.38) | 2.40× (2.33–2.47) | 2.24× (2.17–2.31) | 2.23× (2.16–2.29) | 2.60× (2.53–2.67) | 2.98× (2.92–3.05) | 2.94× (2.86–3.02) | 1.22× (1.17–1.27) | 1.24× (1.19–1.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.093 s | 86 | 59.8 | 1.25× (1.21–1.29) | 2.26× (2.15–2.37) | 2.14× (2.04–2.24) | 2.76× (2.64–2.89) | 2.66× (2.57–2.75) | 4.84× (4.68–5.01) | 4.77× (4.59–4.96) | 1.98× (1.91–2.05) | 2.00× (1.93–2.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.077 s | 104 | 82.3 | 1.44× (1.40–1.48) | 2.12× (2.05–2.19) | 2.08× (2.01–2.16) | 2.59× (2.50–2.69) | 2.75× (2.69–2.80) | 5.88× (5.77–6.00) | 5.80× (5.66–5.94) | 2.41× (2.32–2.50) | 2.44× (2.36–2.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.176 s | 54 | 52.0 | 1.37× (1.33–1.41) | 2.65× (2.56–2.74) | 2.32× (2.23–2.40) | 2.36× (2.30–2.43) | 2.72× (2.64–2.80) | 3.52× (3.41–3.62) | 3.64× (3.54–3.75) | 1.39× (1.33–1.45) | 1.42× (1.36–1.47) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.100 s | 96 | 55.0 | 1.32× (1.27–1.38) | 2.58× (2.49–2.68) | 2.36× (2.26–2.48) | 3.33× (3.23–3.44) | 2.93× (2.79–3.08) | 6.13× (5.89–6.39) | 6.36× (6.11–6.62) | 2.42× (2.32–2.52) | 2.47× (2.38–2.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.088 s | 109 | 82.7 | 1.56× (1.54–1.59) | 2.21× (2.13–2.29) | 2.22× (2.11–2.33) | 2.88× (2.80–2.96) | 2.70× (2.64–2.75) | 6.96× (6.85–7.07) | 7.21× (7.11–7.32) | 2.75× (2.66–2.83) | 2.80× (2.74–2.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 758 | 4.2 | 0.71× (0.70–0.73) | 3.10× (3.06–3.15) | 5.08× (4.80–5.38) | 3.15× (3.11–3.20) | 7.33× (7.21–7.46) | 2.43× (2.40–2.46) | 2.40× (2.36–2.43) | 1.05× (1.02–1.07) | 1.57× (1.54–1.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 694 | 4.7 | 0.64× (0.62–0.66) | 2.86× (2.79–2.93) | 5.60× (5.47–5.74) | 2.95× (2.88–3.02) | 6.75× (6.58–6.92) | 2.23× (2.17–2.30) | 2.20× (2.14–2.26) | 0.96× (0.93–0.99) | 1.44× (1.41–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 685 | 4.5 | 0.64× (0.62–0.66) | 2.84× (2.77–2.92) | 5.56× (5.41–5.71) | 2.93× (2.84–3.01) | 6.69× (6.51–6.89) | 2.21× (2.15–2.28) | 2.19× (2.13–2.24) | 0.96× (0.92–0.99) | 1.43× (1.39–1.47) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 0.077 s | 174 | 16.4 | 1.79× (1.67–1.92) | 3.55× (3.29–3.82) | 4.20× (3.89–4.53) | 4.19× (3.91–4.50) | 4.38× (4.09–4.70) | 4.72× (4.41–5.06) | 4.35× (4.05–4.67) | 1.38× (1.29–1.47) | 1.41× (1.32–1.51) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 0.050 s | 265 | 19.4 | 1.62× (1.57–1.67) | 3.42× (3.23–3.62) | 4.04× (3.91–4.18) | 4.35× (4.17–4.54) | 4.41× (4.28–4.54) | 7.51× (7.31–7.72) | 6.91× (6.77–7.06) | 2.19× (2.13–2.25) | 2.24× (2.20–2.29) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 0.045 s | 294 | 30.3 | 1.77× (1.73–1.82) | 2.53× (2.49–2.56) | 3.36× (3.27–3.46) | 3.45× (3.37–3.53) | 4.05× (3.95–4.15) | 8.18× (8.02–8.34) | 7.53× (7.38–7.69) | 2.38× (2.34–2.43) | 2.44× (2.40–2.49) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.04× (3.03–3.06) | 6.54× (6.50–6.57) | 7.55× (7.52–7.59) | 7.77× (7.75–7.80) | 7.10× (7.06–7.14) | 7.93× (7.87–8.00) | 7.16× (7.11–7.21) | 2.22× (2.21–2.23) | 2.23× (2.21–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.93× (2.85–3.01) | 6.18× (6.10–6.27) | 7.40× (7.32–7.48) | 7.61× (7.52–7.69) | 6.61× (6.52–6.69) | 7.29× (7.23–7.36) | 6.58× (6.53–6.64) | 2.04× (2.02–2.06) | 2.05× (2.03–2.07) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.27× (3.21–3.34) | 4.85× (4.75–4.94) | 5.92× (5.82–6.02) | 5.82× (5.71–5.92) | 5.20× (5.10–5.29) | 4.40× (4.33–4.48) | 3.97× (3.90–4.05) | 1.23× (1.21–1.26) | 1.24× (1.21–1.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.67–0.70) | 2.86× (2.79–2.92) | 5.10× (4.99–5.21) | 3.39× (3.32–3.47) | 6.09× (5.96–6.23) | 2.64× (2.59–2.70) | 2.60× (2.55–2.66) | 1.15× (1.10–1.20) | 0.99× (0.97–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.67× (0.65–0.68) | 2.73× (2.67–2.79) | 6.19× (6.06–6.33) | 3.30× (3.23–3.37) | 5.81× (5.68–5.93) | 2.52× (2.46–2.58) | 2.48× (2.43–2.54) | 1.10× (1.04–1.15) | 0.95× (0.92–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.66× (0.64–0.68) | 3.14× (3.06–3.23) | 6.30× (6.15–6.46) | 3.35× (3.26–3.44) | 5.99× (5.82–6.16) | 2.56× (2.50–2.62) | 2.52× (2.46–2.59) | 1.11× (1.08–1.15) | 0.96× (0.94–0.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.00× (2.98–3.02) | 3.12× (3.09–3.14) | 4.34× (4.30–4.37) | 4.15× (4.12–4.18) | 4.75× (4.71–4.78) | 3.81× (3.79–3.84) | 2.98× (2.95–3.00) | 2.11× (2.08–2.14) | 2.14× (2.11–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.65× (2.59–2.72) | 2.77× (2.69–2.84) | 3.90× (3.80–4.00) | 3.54× (3.45–3.63) | 4.04× (3.95–4.14) | 3.17× (3.11–3.25) | 2.48× (2.42–2.53) | 1.76× (1.72–1.79) | 1.78× (1.74–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.45× (2.32–2.59) | 1.85× (1.76–1.95) | 2.67× (2.56–2.80) | 2.23× (2.13–2.33) | 2.46× (2.36–2.57) | 1.62× (1.55–1.69) | 1.26× (1.21–1.31) | 0.89× (0.86–0.93) | 0.91× (0.87–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.31–1.37) | 2.35× (2.29–2.42) | 2.19× (2.12–2.25) | 2.21× (2.15–2.28) | 2.58× (2.51–2.65) | 2.96× (2.90–3.02) | 2.92× (2.83–3.00) | 1.22× (1.17–1.27) | 1.20× (1.17–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.29) | 2.25× (2.17–2.32) | 2.01× (1.95–2.08) | 2.64× (2.57–2.71) | 2.37× (2.31–2.44) | 2.66× (2.60–2.72) | 2.62× (2.55–2.70) | 1.09× (1.07–1.13) | 1.08× (1.05–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.54–1.63) | 2.23× (2.15–2.31) | 1.88× (1.82–1.94) | 2.53× (2.47–2.59) | 2.33× (2.27–2.39) | 1.92× (1.88–1.97) | 1.89× (1.86–1.93) | 0.79× (0.77–0.82) | 0.78× (0.76–0.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.37× (1.33–1.41) | 2.61× (2.52–2.70) | 2.28× (2.20–2.36) | 2.35× (2.29–2.41) | 2.70× (2.62–2.78) | 3.49× (3.39–3.60) | 3.62× (3.51–3.72) | 1.38× (1.32–1.44) | 1.38× (1.33–1.43) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.32–1.37) | 2.58× (2.52–2.65) | 2.20× (2.15–2.24) | 3.09× (3.00–3.17) | 2.65× (2.54–2.76) | 3.27× (3.20–3.33) | 3.39× (3.32–3.45) | 1.29× (1.26–1.33) | 1.29× (1.26–1.32) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.68× (1.62–1.74) | 2.40× (2.34–2.46) | 1.97× (1.87–2.06) | 2.67× (2.61–2.74) | 2.35× (2.31–2.39) | 2.18× (2.16–2.21) | 2.26× (2.24–2.29) | 0.86× (0.84–0.89) | 0.86× (0.84–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.75× (0.74–0.76) | 2.45× (2.41–2.49) | 4.18× (4.13–4.23) | 2.91× (2.86–2.95) | 6.60× (6.50–6.71) | 2.27× (2.24–2.29) | 2.24× (2.21–2.26) | 1.05× (1.02–1.07) | 0.89× (0.87–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.68× (0.67–0.70) | 2.25× (2.19–2.30) | 4.69× (4.57–4.82) | 2.74× (2.67–2.80) | 6.00× (5.85–6.15) | 2.05× (1.99–2.11) | 2.02× (1.96–2.08) | 0.94× (0.91–0.98) | 0.81× (0.79–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.67× (0.65–0.69) | 2.55× (2.47–2.62) | 4.82× (4.65–5.00) | 2.79× (2.72–2.86) | 6.12× (5.96–6.27) | 2.08× (2.02–2.15) | 2.06× (2.00–2.11) | 0.96× (0.93–1.00) | 0.82× (0.79–0.85) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 1.77× (1.66–1.90) | 3.42× (3.19–3.67) | 4.06× (3.77–4.36) | 4.12× (3.84–4.42) | 4.30× (4.02–4.61) | 4.64× (4.33–4.97) | 4.27× (3.98–4.59) | 1.37× (1.28–1.46) | 1.36× (1.27–1.46) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 1.67× (1.64–1.69) | 3.27× (3.13–3.41) | 3.76× (3.71–3.81) | 4.15× (4.06–4.24) | 3.86× (3.80–3.91) | 4.06× (3.99–4.14) | 3.74× (3.69–3.80) | 1.20× (1.18–1.22) | 1.19× (1.18–1.21) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 1.93× (1.84–2.03) | 2.66× (2.60–2.73) | 3.17× (3.07–3.27) | 3.42× (3.34–3.50) | 3.23× (3.17–3.29) | 2.59× (2.56–2.61) | 2.38× (2.35–2.42) | 0.76× (0.75–0.78) | 0.76× (0.75–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.96× (10.94–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.48× (0.38–0.60) | 0.69× (0.66–0.72) | 0.59× (0.57–0.60) | 5.54× (5.36–5.74) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.36) | 0.49× (0.41–0.59) | 0.60× (0.58–0.63) | 0.54× (0.51–0.57) | 3.53× (3.39–3.67) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.40–0.43) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.97× (7.94–8.01) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.65× (0.59–0.70) | 1.60× (1.56–1.63) | 0.80× (0.78–0.82) | 5.80× (5.67–5.93) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.54× (0.53–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.71× (0.66–0.75) | 1.16× (1.15–1.17) | 0.72× (0.71–0.73) | 3.48× (3.44–3.51) | 0.27× (0.27–0.28) | 0.29× (0.28–0.29) | 0.31× (0.31–0.32) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.16× (5.14–5.17) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.52) | 0.37× (0.34–0.40) | 0.88× (0.85–0.92) | 0.70× (0.66–0.74) | 3.05× (2.92–3.18) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.62–0.64) | 0.44× (0.44–0.45) | 0.84× (0.83–0.85) | 0.79× (0.78–0.81) | 2.46× (2.42–2.49) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.83× (0.82–0.83) | 5.17× (5.15–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.39× (0.36–0.42) | 0.86× (0.82–0.90) | 0.72× (0.67–0.78) | 2.94× (2.81–3.08) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.62) | 0.49× (0.46–0.53) | 0.81× (0.80–0.82) | 0.82× (0.79–0.85) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.06–1.07) | 0.77× (0.76–0.77) | 5.35× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.76× (0.76–0.76) | 5.27× (5.26–5.28) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.67× (0.66–0.67) | 6.35× (6.32–6.37) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 37 MB | 0.46× (0.45–0.47) | 0.40× (0.36–0.44) | 1.04× (1.01–1.08) | 0.67× (0.64–0.70) | 3.37× (3.30–3.45) | 0.21× (0.20–0.21) | 0.22× (0.22–0.23) | 0.33× (0.33–0.34) | 0.16× (0.16–0.17) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.45× (0.40–0.51) | 0.89× (0.87–0.91) | 0.73× (0.71–0.75) | 2.26× (2.22–2.31) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 404 | 2.48 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 695 | 2.70 | 1.09× (1.08–1.10) | 84% | 1.03× (1.02–1.04) | 1.05× (1.02–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 809 | 4.51 | 1.80× (1.76–1.84) | 50% | 1.34× (1.32–1.35) | 1.94× (1.91–1.97) |

**Best:** mibz2 at 4 threads (809 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (245 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.18, lbzip2 1t 0.05, lbzip2 2t 0.24, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.33. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.73 | 0.03 | 0.08 | 0.259 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 10.8% |
| MTF + RLE2 | 1.40 | 15.0% |
| T-vector build | 1.40 | 14.9% |
| walk (the one each build uses) | 5.24 | 55.9% |
| of which: segment assembly | 0.25 | 2.7% |
| RLE1 | 0.21 | 2.3% |
| CRC | 0.12 | 1.3% |
| **total** | 9.38 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.264 | (in its total) | 2.60× (2.60–2.61) |  |
| walk | 1.757 | 2.834 | (in its total) | 1.61× (1.60–1.62) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.085 | 0.086 | (in its total) | 1.00× (0.96–1.05) |  |
| inverse BWT, total | 2.328 | 4.185 | 2.482 | 1.79× (1.78–1.80) | 1.07× (1.06–1.07) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m30s |
| 4 threads | 37m57s ± 5m22s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m31s ± 3m46s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 26m31s ± 3m46s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 3.48 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,246 | 13,345,437 | \`3e431388ee58319e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 2800.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
