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

The latest record: [`0813a3d`](<https://github.com/lmmx/mibz2/commit/0813a3d3e7cf48e402af10822055365fe16f3e06>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44601992797>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.521 s | 403 | 7.2 | 3.11× (3.04–3.17) | 7.27× (6.99–7.55) | 7.36× (7.15–7.58) | 8.56× (8.27–8.86) | 7.55× (7.35–7.75) | 7.78× (7.57–8.00) | 7.46× (7.16–7.77) | 2.30× (2.26–2.34) | 2.51× (2.46–2.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.297 s | 706 | 7.9 | 3.23× (3.13–3.34) | 7.05× (6.79–7.32) | 7.80× (7.50–8.11) | 8.25× (8.02–8.49) | 7.38× (7.09–7.68) | 13.84× (13.34–14.36) | 13.26× (12.72–13.83) | 4.09× (3.96–4.21) | 4.47× (4.39–4.56) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.250 s | 840 | 12.2 | 3.24× (3.16–3.32) | 5.19× (4.89–5.50) | 6.29× (5.94–6.65) | 6.07× (5.75–6.41) | 5.97× (5.67–6.29) | 16.42× (15.90–16.95) | 15.73× (15.16–16.33) | 4.85× (4.72–4.98) | 5.30× (5.21–5.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 889 | 3.5 | 0.58× (0.58–0.59) | 3.32× (3.18–3.47) | 4.43× (4.40–4.47) | 3.63× (3.61–3.65) | 6.13× (6.02–6.24) | 2.72× (2.70–2.74) | 2.72× (2.70–2.74) | 1.00× (0.97–1.03) | 1.82× (1.81–1.84) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 875 | 3.6 | 0.55× (0.53–0.58) | 2.72× (2.68–2.76) | 5.78× (5.68–5.87) | 3.60× (3.55–3.66) | 5.97× (5.89–6.05) | 2.69× (2.65–2.73) | 2.69× (2.65–2.73) | 0.99× (0.95–1.02) | 1.80× (1.77–1.83) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 885 | 3.5 | 0.55× (0.54–0.56) | 2.75× (2.70–2.80) | 5.77× (5.67–5.88) | 3.61× (3.55–3.68) | 6.05× (5.90–6.21) | 2.70× (2.65–2.75) | 2.70× (2.64–2.75) | 0.99× (0.96–1.02) | 1.81× (1.77–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 571 | 5.3 | 2.71× (2.68–2.74) | 2.97× (2.93–3.01) | 3.78× (3.73–3.83) | 4.08× (4.02–4.13) | 4.44× (4.37–4.52) | 2.76× (2.72–2.81) | 2.25× (2.21–2.28) | 2.15× (2.08–2.22) | 2.96× (2.92–3.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.026 s | 758 | 6.8 | 2.02× (1.81–2.25) | 2.42× (2.23–2.64) | 3.70× (3.42–4.00) | 3.19× (2.90–3.52) | 4.34× (3.95–4.76) | 3.60× (3.26–3.98) | 2.93× (2.66–3.23) | 2.80× (2.52–3.11) | 3.85× (3.50–4.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 778 | 12.9 | 1.97× (1.91–2.04) | 1.81× (1.70–1.92) | 2.98× (2.83–3.13) | 2.26× (2.11–2.42) | 4.09× (3.91–4.28) | 3.72× (3.58–3.88) | 3.03× (2.90–3.16) | 2.89× (2.79–3.00) | 3.98× (3.82–4.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.144 s | 55 | 52.5 | 1.38× (1.36–1.40) | 2.61× (2.53–2.70) | 2.37× (2.25–2.49) | 2.41× (2.33–2.49) | 2.77× (2.64–2.92) | 3.01× (2.94–3.08) | 3.04× (2.95–3.14) | 1.26× (1.23–1.28) | 1.40× (1.36–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.091 s | 88 | 59.0 | 1.31× (1.20–1.43) | 2.48× (2.28–2.71) | 2.39× (2.20–2.60) | 2.96× (2.81–3.12) | 2.78× (2.65–2.92) | 4.79× (4.65–4.93) | 4.85× (4.68–5.02) | 2.00× (1.95–2.06) | 2.23× (2.16–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 110 | 78.0 | 1.34× (1.30–1.37) | 2.33× (2.22–2.44) | 2.30× (2.12–2.49) | 2.81× (2.63–3.01) | 2.97× (2.84–3.10) | 6.03× (5.91–6.15) | 6.10× (6.05–6.15) | 2.52× (2.48–2.56) | 2.80× (2.76–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.156 s | 62 | 47.3 | 1.56× (1.52–1.61) | 3.10× (3.02–3.18) | 2.62× (2.55–2.69) | 2.81× (2.73–2.89) | 3.09× (3.06–3.13) | 3.67× (3.63–3.72) | 3.95× (3.88–4.01) | 1.57× (1.54–1.61) | 1.80× (1.77–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.095 s | 102 | 53.4 | 1.40× (1.34–1.45) | 2.82× (2.72–2.94) | 2.56× (2.46–2.67) | 3.64× (3.48–3.80) | 3.13× (3.02–3.23) | 6.05× (5.92–6.19) | 6.51× (6.40–6.61) | 2.59× (2.57–2.62) | 2.97× (2.90–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.081 s | 118 | 75.2 | 1.40× (1.37–1.44) | 2.44× (2.17–2.74) | 2.37× (2.26–2.48) | 3.18× (3.04–3.32) | 2.97× (2.80–3.14) | 7.05× (6.95–7.16) | 7.58× (7.44–7.72) | 3.02× (2.94–3.11) | 3.46× (3.39–3.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 680 | 4.8 | 0.65× (0.63–0.66) | 2.96× (2.66–3.30) | 3.99× (3.92–4.07) | 3.17× (3.11–3.23) | 6.76× (6.54–6.99) | 2.35× (2.30–2.40) | 2.38× (2.34–2.43) | 0.97× (0.95–1.00) | 2.08× (2.04–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 629 | 5.3 | 0.53× (0.52–0.54) | 2.67× (2.54–2.81) | 4.99× (4.78–5.20) | 3.01× (2.96–3.06) | 6.28× (6.18–6.39) | 2.21× (2.17–2.24) | 2.24× (2.20–2.28) | 0.91× (0.90–0.93) | 1.95× (1.92–1.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 629 | 5.1 | 0.53× (0.52–0.54) | 2.61× (2.58–2.65) | 5.05× (4.96–5.15) | 3.00× (2.96–3.04) | 6.40× (6.24–6.56) | 2.20× (2.17–2.23) | 2.23× (2.20–2.26) | 0.91× (0.89–0.92) | 1.95× (1.92–1.97) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 0.080 s | 166 | 17.8 | 1.80× (1.75–1.85) | 3.95× (3.76–4.16) | 4.26× (4.12–4.40) | 4.49× (4.40–4.59) | 4.59× (4.32–4.88) | 4.73× (4.38–5.10) | 4.68× (4.44–4.92) | 1.48× (1.45–1.52) | 1.65× (1.61–1.70) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 0.052 s | 254 | 20.9 | 1.51× (1.45–1.57) | 3.31× (3.10–3.54) | 4.05× (3.76–4.37) | 4.37× (4.20–4.55) | 4.36× (4.17–4.55) | 7.15× (6.63–7.71) | 7.07× (6.54–7.65) | 2.24× (2.14–2.35) | 2.50× (2.37–2.64) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 0.048 s | 275 | 31.9 | 1.54× (1.42–1.66) | 2.72× (2.46–3.01) | 3.24× (3.04–3.45) | 3.57× (3.36–3.78) | 3.92× (3.71–4.14) | 7.82× (7.26–8.42) | 7.74× (7.33–8.17) | 2.45× (2.40–2.51) | 2.73× (2.66–2.81) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.10× (3.04–3.16) | 7.23× (6.96–7.52) | 7.33× (7.11–7.55) | 8.53× (8.24–8.83) | 7.52× (7.32–7.72) | 7.75× (7.54–7.97) | 7.43× (7.13–7.74) | 2.29× (2.25–2.33) | 2.50× (2.44–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (2.99–3.12) | 7.02× (6.79–7.25) | 7.22× (6.95–7.50) | 8.31× (8.09–8.54) | 7.20× (6.94–7.47) | 7.12× (6.88–7.36) | 6.82× (6.55–7.10) | 2.10× (2.05–2.16) | 2.29× (2.25–2.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.08–3.31) | 5.67× (5.38–5.98) | 6.14× (5.87–6.43) | 6.62× (6.31–6.94) | 5.86× (5.61–6.13) | 4.62× (4.48–4.76) | 4.43× (4.25–4.60) | 1.36× (1.34–1.39) | 1.49× (1.46–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.62) | 2.99× (2.97–3.02) | 3.99× (3.97–4.01) | 3.40× (3.38–3.42) | 5.69× (5.59–5.79) | 2.58× (2.56–2.59) | 2.57× (2.56–2.59) | 1.00× (0.97–1.03) | 1.67× (1.66–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.58–0.63) | 2.81× (2.77–2.85) | 5.02× (4.93–5.11) | 3.35× (3.30–3.40) | 5.46× (5.39–5.52) | 2.50× (2.47–2.54) | 2.50× (2.46–2.54) | 0.97× (0.94–1.00) | 1.63× (1.60–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.14× (3.08–3.21) | 5.09× (5.01–5.17) | 3.42× (3.36–3.49) | 5.62× (5.49–5.77) | 2.56× (2.51–2.61) | 2.56× (2.51–2.60) | 0.99× (0.97–1.02) | 1.66× (1.63–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.58–2.64) | 2.83× (2.79–2.87) | 3.40× (3.37–3.44) | 3.90× (3.85–3.95) | 4.24× (4.18–4.31) | 2.66× (2.62–2.70) | 2.17× (2.15–2.20) | 2.08× (2.02–2.15) | 2.64× (2.60–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.22× (2.12–2.31) | 2.40× (2.27–2.55) | 3.15× (2.99–3.32) | 3.18× (3.00–3.36) | 3.34× (3.12–3.56) | 2.04× (1.92–2.16) | 1.66× (1.57–1.77) | 1.59× (1.54–1.65) | 2.02× (1.92–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.07–2.18) | 1.69× (1.62–1.77) | 2.25× (2.20–2.30) | 2.07× (1.99–2.16) | 2.11× (2.05–2.18) | 1.10× (1.07–1.12) | 0.90× (0.88–0.92) | 0.86× (0.82–0.90) | 1.09× (1.06–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.40) | 2.55× (2.48–2.64) | 2.31× (2.19–2.43) | 2.39× (2.31–2.47) | 2.75× (2.62–2.89) | 2.98× (2.91–3.05) | 3.01× (2.92–3.11) | 1.25× (1.23–1.28) | 1.35× (1.32–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.46× (2.37–2.55) | 2.24× (2.09–2.41) | 2.87× (2.77–2.97) | 2.51× (2.37–2.66) | 2.65× (2.61–2.70) | 2.68× (2.62–2.75) | 1.12× (1.10–1.13) | 1.21× (1.19–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.53) | 2.48× (2.40–2.55) | 2.08× (1.89–2.29) | 2.84× (2.64–3.06) | 2.67× (2.53–2.82) | 2.04× (1.98–2.10) | 2.06× (2.02–2.10) | 0.86× (0.84–0.88) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.55× (1.51–1.60) | 3.03× (2.95–3.11) | 2.57× (2.51–2.64) | 2.78× (2.70–2.87) | 3.06× (3.03–3.10) | 3.63× (3.59–3.68) | 3.91× (3.85–3.97) | 1.57× (1.53–1.60) | 1.76× (1.74–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.37–1.43) | 2.83× (2.71–2.97) | 2.36× (2.29–2.44) | 3.39× (3.24–3.56) | 2.82× (2.72–2.92) | 3.21× (3.15–3.28) | 3.45× (3.39–3.51) | 1.38× (1.37–1.40) | 1.55× (1.52–1.59) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.54–1.63) | 2.77× (2.41–3.18) | 2.17× (1.99–2.36) | 3.06× (2.85–3.29) | 2.78× (2.54–3.04) | 2.31× (2.26–2.37) | 2.48× (2.43–2.54) | 1.00× (0.97–1.03) | 1.12× (1.09–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.71) | 2.60× (2.41–2.80) | 3.53× (3.47–3.59) | 2.89× (2.84–2.94) | 6.03× (5.84–6.22) | 2.18× (2.13–2.22) | 2.20× (2.16–2.25) | 0.97× (0.95–0.99) | 1.39× (1.37–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.35× (2.27–2.43) | 4.10× (4.03–4.18) | 2.78× (2.74–2.82) | 5.58× (5.48–5.67) | 2.02× (1.99–2.05) | 2.05× (2.02–2.08) | 0.90× (0.89–0.91) | 1.29× (1.27–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.57× (2.54–2.60) | 4.24× (4.14–4.34) | 2.86× (2.83–2.89) | 5.84× (5.71–5.97) | 2.07× (2.05–2.10) | 2.10× (2.07–2.13) | 0.93× (0.91–0.94) | 1.33× (1.31–1.34) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 1.78× (1.73–1.83) | 3.81× (3.62–4.01) | 4.12× (3.98–4.26) | 4.40× (4.31–4.50) | 4.50× (4.24–4.78) | 4.63× (4.29–5.00) | 4.58× (4.36–4.82) | 1.47× (1.44–1.51) | 1.56× (1.51–1.60) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 1.56× (1.53–1.59) | 3.36× (3.22–3.52) | 3.81× (3.49–4.16) | 4.24× (4.18–4.29) | 3.88× (3.74–4.02) | 3.91× (3.64–4.20) | 3.88× (3.61–4.16) | 1.24× (1.21–1.28) | 1.32× (1.29–1.34) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 1.71× (1.57–1.86) | 3.02× (2.65–3.43) | 3.13× (2.95–3.33) | 3.55× (3.29–3.84) | 3.33× (3.08–3.60) | 2.57× (2.35–2.82) | 2.55× (2.38–2.73) | 0.82× (0.79–0.84) | 0.87× (0.84–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.28–0.32) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.96× (10.93–10.99) | 0.26× (0.26–0.26) | 0.28× (0.27–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.31–0.34) | 0.43× (0.38–0.49) | 0.68× (0.65–0.71) | 0.56× (0.54–0.59) | 5.39× (5.16–5.62) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.32–0.35) | 0.53× (0.44–0.64) | 0.60× (0.57–0.64) | 0.53× (0.50–0.56) | 3.50× (3.32–3.68) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.16× (0.15–0.16) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.28–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.65× (0.65–0.66) | 7.98× (7.97–7.99) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.68) | 0.66× (0.64–0.69) | 1.58× (1.54–1.63) | 0.80× (0.77–0.82) | 5.73× (5.57–5.90) | 0.46× (0.45–0.48) | 0.49× (0.47–0.50) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.65–0.68) | 0.73× (0.66–0.80) | 1.18× (1.16–1.20) | 0.73× (0.71–0.75) | 3.48× (3.43–3.54) | 0.27× (0.27–0.28) | 0.29× (0.28–0.29) | 0.31× (0.31–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.16× (5.15–5.17) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.47–0.52) | 0.40× (0.37–0.43) | 0.89× (0.85–0.93) | 0.71× (0.68–0.73) | 3.09× (2.95–3.23) | 0.19× (0.18–0.20) | 0.21× (0.19–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.87× (0.85–0.88) | 0.80× (0.78–0.82) | 2.52× (2.47–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.81–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.35× (0.32–0.39) | 0.84× (0.82–0.87) | 0.69× (0.65–0.72) | 2.87× (2.80–2.95) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.46× (0.43–0.48) | 0.80× (0.80–0.81) | 0.82× (0.80–0.84) | 2.30× (2.28–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.76× (0.75–0.76) | 5.35× (5.34–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.78) | 5.27× (5.26–5.29) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.66) | 6.34× (6.33–6.35) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 37 MB | 0.45× (0.45–0.46) | 0.39× (0.35–0.44) | 1.05× (1.02–1.08) | 0.68× (0.66–0.70) | 3.33× (3.27–3.38) | 0.20× (0.20–0.21) | 0.22× (0.22–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 54 MB | 0.52× (0.50–0.53) | 0.47× (0.44–0.51) | 0.94× (0.91–0.97) | 0.77× (0.74–0.81) | 2.39× (2.33–2.45) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 403 | 2.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 706 | 2.76 | 1.09× (1.06–1.12) | 89% | 1.06× (1.03–1.08) | 1.07× (1.06–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 840 | 4.26 | 1.68× (1.65–1.71) | 53% | 1.32× (1.26–1.38) | 1.73× (1.67–1.79) |

**Best:** mibz2 at 4 threads (840 MB/s); lbzip2 at 4 threads (160 MB/s); pbz2 at 4 threads (261 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.22, lbzip2 1t 0.06, lbzip2 2t 0.23, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.69 | 0.02 | 0.08 | 0.250 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2872 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.6% |
| MTF + RLE2 | 1.53 | 17.3% |
| T-vector build | 1.52 | 17.3% |
| walk (the one each build uses) | 4.21 | 47.7% |
| of which: segment assembly | 0.16 | 1.9% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.83 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.504 | 1.371 | (in its total) | 2.72× (2.72–2.73) |  |
| walk | 1.360 | 2.667 | (in its total) | 1.97× (1.96–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.051 | 0.055 | (in its total) | 1.09× (1.06–1.11) |  |
| inverse BWT, total | 1.915 | 4.094 | 2.528 | 2.14× (2.13–2.15) | 1.32× (1.32–1.32) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m46s |
| 4 threads | 36m28s ± 5m08s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m55s ± 3m32s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 24m55s ± 3m32s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.27 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,615,664 | 13,288,779 | \`0b9f69f4e27fcf7e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2872.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
