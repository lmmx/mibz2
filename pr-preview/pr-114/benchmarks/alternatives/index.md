# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-114/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`0711dc3`](<https://github.com/lmmx/mibz2/commit/0711dc3a28b827d1101dbf8353afc75ce674be0a>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40929726603>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.621 s | 338 | 7.7 | 1.92× (1.84–2.00) | 4.40× (4.22–4.59) | 4.61× (4.44–4.79) | 5.18× (4.99–5.39) | 4.57× (4.38–4.75) | 5.11× (4.91–5.31) | 4.60× (4.43–4.77) | 1.43× (1.39–1.47) | 1.61× (1.54–1.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.435 s | 482 | 8.9 | 1.66× (1.59–1.74) | 3.46× (3.31–3.60) | 4.03× (3.86–4.20) | 4.12× (3.95–4.30) | 3.67× (3.52–3.84) | 7.58× (7.26–7.91) | 6.82× (6.55–7.11) | 2.12× (2.03–2.21) | 2.39× (2.28–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.351 s | 597 | 14.6 | 1.68× (1.62–1.75) | 2.50× (2.43–2.58) | 3.14× (3.03–3.25) | 2.98× (2.91–3.05) | 3.02× (2.94–3.09) | 9.19× (8.96–9.42) | 8.27× (8.08–8.47) | 2.57× (2.50–2.64) | 2.90× (2.82–2.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,050 | 2.7 | 0.55× (0.54–0.55) | 3.22× (3.18–3.26) | 4.32× (4.27–4.37) | 3.38× (3.35–3.42) | 6.31× (6.22–6.41) | 2.57× (2.45–2.70) | 2.50× (2.47–2.53) | 1.01× (0.99–1.04) | 2.15× (2.12–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,061 | 2.7 | 0.52× (0.51–0.53) | 2.73× (2.69–2.77) | 6.02× (5.93–6.11) | 3.50× (3.39–3.61) | 6.42× (6.19–6.67) | 2.60× (2.47–2.75) | 2.53× (2.50–2.57) | 1.03× (1.00–1.05) | 2.17× (2.14–2.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,061 | 2.6 | 0.53× (0.51–0.54) | 2.72× (2.67–2.76) | 6.12× (5.76–6.49) | 3.42× (3.36–3.48) | 6.31× (6.19–6.43) | 2.58× (2.45–2.73) | 2.51× (2.47–2.55) | 1.02× (0.99–1.04) | 2.16× (2.12–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 585 | 4.7 | 2.14× (2.12–2.16) | 2.44× (2.42–2.46) | 3.06× (3.04–3.08) | 3.26× (3.23–3.28) | 3.93× (3.88–3.98) | 2.72× (2.70–2.74) | 1.80× (1.79–1.82) | 1.67× (1.66–1.68) | 2.43× (2.42–2.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 903 | 5.8 | 1.87× (1.84–1.91) | 2.35× (2.30–2.41) | 3.61× (3.43–3.81) | 2.98× (2.91–3.05) | 4.89× (4.62–5.18) | 4.17× (4.07–4.27) | 2.76× (2.70–2.83) | 2.56× (2.50–2.62) | 3.73× (3.65–3.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 737 | 12.1 | 1.42× (1.32–1.52) | 1.57× (1.48–1.66) | 2.30× (2.10–2.53) | 1.73× (1.61–1.87) | 3.53× (3.31–3.75) | 3.39× (3.18–3.61) | 2.25× (2.12–2.39) | 2.08× (1.96–2.21) | 3.03× (2.86–3.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 63 | 42.0 | 1.20× (1.18–1.22) | 2.31× (2.28–2.35) | 2.10× (2.07–2.13) | 2.20× (2.15–2.26) | 2.49× (2.46–2.53) | 2.73× (2.69–2.77) | 2.73× (2.69–2.77) | 1.09× (1.07–1.11) | 1.20× (1.18–1.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.076 s | 105 | 45.7 | 1.18× (1.13–1.22) | 2.20× (2.12–2.29) | 2.16× (2.12–2.21) | 2.83× (2.74–2.93) | 2.74× (2.70–2.78) | 4.59× (4.50–4.68) | 4.59× (4.50–4.67) | 1.84× (1.81–1.86) | 2.02× (1.98–2.05) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 s | 135 | 58.4 | 1.24× (1.22–1.27) | 2.01× (1.97–2.06) | 2.10× (2.05–2.15) | 2.50× (2.45–2.54) | 2.95× (2.90–3.00) | 5.88× (5.76–6.00) | 5.87× (5.77–5.98) | 2.35× (2.32–2.38) | 2.58× (2.54–2.62) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.163 s | 59 | 44.4 | 1.10× (1.05–1.15) | 2.23× (2.13–2.34) | 1.92× (1.83–2.02) | 1.99× (1.91–2.08) | 2.30× (2.19–2.40) | 2.87× (2.74–3.01) | 2.94× (2.81–3.08) | 1.11× (1.06–1.18) | 1.25× (1.19–1.32) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.095 s | 102 | 48.0 | 1.06× (1.00–1.13) | 2.15× (2.05–2.25) | 2.01× (1.91–2.11) | 2.77× (2.67–2.88) | 2.46× (2.36–2.56) | 4.96× (4.75–5.18) | 5.08× (4.87–5.31) | 1.92× (1.84–2.01) | 2.16× (2.08–2.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 135 | 61.3 | 1.26× (1.24–1.28) | 2.08× (2.04–2.12) | 2.02× (1.97–2.08) | 2.78× (2.71–2.85) | 2.74× (2.69–2.80) | 6.74× (6.64–6.83) | 6.90× (6.83–6.97) | 2.61× (2.56–2.67) | 2.94× (2.88–3.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 784 | 3.7 | 0.59× (0.58–0.60) | 3.21× (3.17–3.25) | 4.06× (4.01–4.11) | 2.90× (2.86–2.93) | 7.06× (7.00–7.13) | 2.18× (2.15–2.20) | 2.16× (2.14–2.19) | 0.97× (0.96–0.99) | 1.61× (1.60–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 781 | 3.8 | 0.51× (0.50–0.51) | 2.41× (2.37–2.44) | 4.97× (4.69–5.27) | 2.90× (2.87–2.94) | 7.02× (6.88–7.15) | 2.17× (2.14–2.19) | 2.15× (2.12–2.18) | 0.97× (0.95–0.99) | 1.61× (1.59–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 3.8 | 0.51× (0.50–0.52) | 2.38× (2.34–2.42) | 4.86× (4.75–4.98) | 2.89× (2.84–2.93) | 6.91× (6.78–7.05) | 2.14× (2.10–2.17) | 2.12× (2.09–2.15) | 0.96× (0.94–0.97) | 1.59× (1.56–1.61) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 0.072 s | 173 | 15.4 | 1.31× (1.29–1.33) | 2.77× (2.68–2.86) | 3.12× (3.06–3.19) | 3.16× (3.10–3.23) | 3.55× (3.47–3.64) | 3.30× (3.24–3.35) | 3.26× (3.20–3.32) | 1.07× (1.05–1.08) | 1.28× (1.25–1.30) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 0.043 s | 286 | 17.6 | 1.16× (1.12–1.20) | 2.55× (2.39–2.73) | 3.17× (3.01–3.35) | 3.37× (3.24–3.50) | 3.72× (3.59–3.86) | 5.33× (5.14–5.53) | 5.26× (5.06–5.48) | 1.72× (1.67–1.78) | 2.06× (1.98–2.15) | yes |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 0.037 s | 339 | 26.3 | 1.23× (1.21–1.26) | 2.05× (1.95–2.15) | 2.73× (2.62–2.84) | 2.96× (2.89–3.04) | 3.60× (3.50–3.69) | 6.39× (6.27–6.51) | 6.31× (6.17–6.45) | 2.06× (2.03–2.10) | 2.47× (2.42–2.53) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.92× (1.84–2.00) | 4.39× (4.20–4.58) | 4.60× (4.42–4.78) | 5.17× (4.98–5.38) | 4.56× (4.38–4.74) | 5.09× (4.90–5.30) | 4.59× (4.42–4.76) | 1.43× (1.39–1.47) | 1.60× (1.53–1.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.75× (1.71–1.79) | 3.97× (3.87–4.07) | 4.20× (4.11–4.30) | 4.80× (4.68–4.93) | 4.04× (3.92–4.16) | 4.46× (4.33–4.60) | 4.02× (3.91–4.13) | 1.25× (1.22–1.28) | 1.40× (1.36–1.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.78× (1.73–1.83) | 2.94× (2.85–3.03) | 3.23× (3.12–3.35) | 3.51× (3.41–3.60) | 3.06× (2.97–3.15) | 2.76× (2.68–2.84) | 2.48× (2.41–2.55) | 0.77× (0.75–0.79) | 0.87× (0.84–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.59) | 2.77× (2.73–2.80) | 3.71× (3.64–3.78) | 3.19× (3.16–3.22) | 5.88× (5.80–5.96) | 2.45× (2.33–2.57) | 2.38× (2.36–2.40) | 1.01× (0.99–1.03) | 1.61× (1.58–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.66× (2.63–2.69) | 5.31× (5.23–5.40) | 3.26× (3.16–3.37) | 5.89× (5.67–6.11) | 2.44× (2.30–2.58) | 2.37× (2.34–2.40) | 1.01× (0.99–1.03) | 1.60× (1.56–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.95× (2.91–3.00) | 5.47× (5.20–5.76) | 3.27× (3.21–3.32) | 5.91× (5.80–6.02) | 2.47× (2.34–2.62) | 2.40× (2.37–2.44) | 1.02× (1.00–1.05) | 1.63× (1.58–1.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.09× (2.07–2.10) | 2.34× (2.32–2.36) | 2.81× (2.76–2.85) | 3.15× (3.12–3.17) | 3.79× (3.74–3.84) | 2.64× (2.61–2.66) | 1.76× (1.75–1.78) | 1.64× (1.62–1.65) | 2.16× (2.14–2.18) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.84× (1.77–1.91) | 2.08× (2.03–2.14) | 2.68× (2.62–2.74) | 2.68× (2.63–2.73) | 3.29× (3.09–3.50) | 2.13× (2.09–2.18) | 1.43× (1.39–1.46) | 1.32× (1.30–1.35) | 1.74× (1.71–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.50× (1.37–1.64) | 1.36× (1.26–1.46) | 1.69× (1.56–1.84) | 1.60× (1.47–1.73) | 1.71× (1.58–1.85) | 1.00× (0.92–1.08) | 0.67× (0.62–0.72) | 0.62× (0.57–0.67) | 0.82× (0.76–0.88) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.20× (1.18–1.21) | 2.25× (2.21–2.28) | 2.06× (2.03–2.08) | 2.18× (2.13–2.24) | 2.47× (2.44–2.51) | 2.71× (2.67–2.75) | 2.71× (2.67–2.75) | 1.09× (1.07–1.11) | 1.16× (1.14–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.16× (1.15–1.17) | 2.18× (2.16–2.20) | 1.99× (1.98–2.00) | 2.64× (2.62–2.66) | 2.35× (2.33–2.36) | 2.49× (2.47–2.51) | 2.49× (2.47–2.51) | 1.00× (0.99–1.01) | 1.07× (1.06–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.37× (1.35–1.40) | 2.14× (2.11–2.17) | 1.88× (1.84–1.91) | 2.45× (2.42–2.49) | 2.43× (2.38–2.49) | 1.95× (1.92–1.98) | 1.95× (1.92–1.97) | 0.79× (0.77–0.80) | 0.83× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.10× (1.05–1.15) | 2.20× (2.10–2.30) | 1.88× (1.80–1.97) | 1.98× (1.90–2.07) | 2.28× (2.18–2.39) | 2.85× (2.72–2.99) | 2.92× (2.79–3.06) | 1.11× (1.05–1.17) | 1.22× (1.16–1.28) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.08× (1.07–1.09) | 2.18× (2.17–2.19) | 1.87× (1.86–1.88) | 2.64× (2.61–2.68) | 2.21× (2.19–2.22) | 2.70× (2.68–2.71) | 2.76× (2.73–2.79) | 1.05× (1.03–1.07) | 1.15× (1.14–1.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.37× (1.35–1.38) | 2.18× (2.13–2.22) | 1.73× (1.69–1.77) | 2.49× (2.44–2.54) | 2.30× (2.24–2.36) | 2.10× (2.07–2.13) | 2.15× (2.12–2.18) | 0.82× (0.80–0.84) | 0.90× (0.88–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.63–0.64) | 2.32× (2.29–2.34) | 3.24× (3.20–3.28) | 2.68× (2.65–2.71) | 6.36× (6.30–6.41) | 2.04× (2.01–2.06) | 2.03× (2.01–2.05) | 0.98× (0.97–0.99) | 1.31× (1.29–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.58) | 2.22× (2.18–2.25) | 4.32× (4.09–4.55) | 2.69× (2.66–2.72) | 6.22× (6.11–6.33) | 1.99× (1.97–2.01) | 1.98× (1.96–2.00) | 0.95× (0.94–0.97) | 1.28× (1.25–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.54–0.60) | 2.46× (2.42–2.50) | 4.28× (4.20–4.36) | 2.76× (2.72–2.80) | 6.32× (6.20–6.43) | 2.02× (1.99–2.06) | 2.01× (1.98–2.04) | 0.97× (0.95–0.99) | 1.30× (1.28–1.32) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 1.30× (1.28–1.32) | 2.67× (2.60–2.75) | 3.00× (2.95–3.05) | 3.11× (3.05–3.17) | 3.49× (3.41–3.58) | 3.24× (3.19–3.30) | 3.20× (3.14–3.26) | 1.06× (1.05–1.08) | 1.15× (1.13–1.17) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 1.20× (1.18–1.21) | 2.42× (2.37–2.46) | 2.84× (2.77–2.92) | 3.06× (3.02–3.11) | 3.08× (3.04–3.13) | 2.81× (2.78–2.84) | 2.78× (2.74–2.81) | 0.92× (0.91–0.93) | 1.00× (0.99–1.01) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 1.36× (1.33–1.40) | 2.07× (1.96–2.19) | 2.35× (2.30–2.41) | 2.51× (2.47–2.55) | 2.52× (2.45–2.59) | 1.88× (1.85–1.91) | 1.86× (1.82–1.89) | 0.62× (0.61–0.62) | 0.67× (0.66–0.68) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.35× (0.33–0.36) | 1.16× (1.16–1.17) | 0.84× (0.83–0.84) | 11.82× (11.79–11.84) | 0.28× (0.28–0.28) | 0.30× (0.30–0.30) | 0.45× (0.45–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.49× (0.45–0.55) | 0.86× (0.84–0.87) | 0.69× (0.67–0.70) | 6.34× (6.28–6.40) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.42) | 0.57× (0.52–0.63) | 0.78× (0.77–0.78) | 0.64× (0.63–0.65) | 4.30× (4.23–4.36) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.27× (4.26–4.27) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.49–0.49) | 0.43× (0.43–0.43) | 1.63× (1.62–1.63) | 0.63× (0.63–0.64) | 7.74× (7.72–7.76) | 0.62× (0.62–0.62) | 0.66× (0.65–0.66) | 0.71× (0.70–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.63× (0.62–0.64) | 0.62× (0.58–0.65) | 1.53× (1.52–1.54) | 0.88× (0.86–0.89) | 5.55× (5.50–5.59) | 0.44× (0.44–0.44) | 0.47× (0.46–0.47) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.58–0.62) | 0.63× (0.60–0.66) | 1.10× (1.09–1.11) | 0.85× (0.82–0.87) | 3.20× (3.16–3.24) | 0.25× (0.24–0.25) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.71× (0.71–0.72) | 5.08× (5.07–5.09) | 0.32× (0.31–0.32) | 0.34× (0.34–0.34) | 0.51× (0.51–0.51) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.52× (0.51–0.52) | 0.42× (0.39–0.45) | 0.94× (0.93–0.94) | 0.75× (0.73–0.76) | 3.27× (3.25–3.30) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.57× (0.56–0.59) | 0.42× (0.42–0.43) | 0.80× (0.79–0.81) | 0.75× (0.74–0.77) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.32) | 1.15× (1.15–1.15) | 0.81× (0.80–0.81) | 5.08× (5.07–5.09) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.50) | 0.25× (0.25–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.49) | 0.39× (0.37–0.42) | 0.87× (0.86–0.88) | 0.73× (0.70–0.75) | 3.00× (2.98–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.55× (0.54–0.56) | 0.45× (0.42–0.47) | 0.74× (0.72–0.75) | 0.75× (0.72–0.77) | 2.14× (2.10–2.19) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.39× (5.38–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.61) | 0.77× (0.76–0.77) | 5.35× (5.34–5.36) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.44× (1.43–1.44) | 0.62× (0.62–0.62) | 6.50× (6.48–6.52) | 0.39× (0.39–0.39) | 0.41× (0.40–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 2 | 32 MB | 0.51× (0.51–0.52) | 0.43× (0.39–0.47) | 1.32× (1.29–1.34) | 0.72× (0.69–0.75) | 3.84× (3.80–3.88) | 0.23× (0.23–0.23) | 0.24× (0.24–0.25) | 0.37× (0.37–0.38) | 0.19× (0.18–0.19) |
| `text` (1.66 MB → 12.4 MB, 7.5×) | 4 | 51 MB | 0.52× (0.51–0.53) | 0.51× (0.47–0.55) | 1.05× (1.03–1.07) | 0.78× (0.76–0.80) | 2.51× (2.47–2.55) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 338 | 2.97 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 482 | 3.44 | 1.14× (1.11–1.17) | 74% | 1.03× (1.02–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 597 | 5.63 | 1.85× (1.77–1.93) | 45% | 1.24× (1.22–1.26) | 1.72× (1.66–1.77) |

**Best:** mibz2 at 4 threads (597 MB/s); lbzip2 at 4 threads (237 MB/s); pbz2 at 4 threads (354 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.70, mibz2 4t 0.98, lbzip2 1t 0.07, lbzip2 2t 0.20, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 7%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.99 | 0.02 | 0.09 | 0.351 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 11.1% |
| MTF + RLE2 | 1.47 | 20.1% |
| T-vector build | 1.22 | 16.7% |
| walk (the one each build uses) | 3.50 | 47.9% |
| of which: segment assembly | 0.11 | 1.5% |
| RLE1 | 0.13 | 1.8% |
| CRC | 0.18 | 2.5% |
| **total** | 7.31 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.467 | 1.064 | (in its total) | 2.28× (2.27–2.28) |  |
| walk | 1.239 | 2.062 | (in its total) | 1.65× (1.63–1.68) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.042 | (in its total) | 0.87× (0.84–0.90) |  |
| inverse BWT, total | 1.754 | 3.168 | 1.904 | 1.80× (1.78–1.82) | 1.08× (1.07–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h32m ± 12m06s |
| 4 threads | 51m33s ± 7m21s |
| 4 threads, the sample's overlap gap as a fixed cost | 36m09s ± 5m12s |
| the scanner alone (its serial part) | 12m05s ± 30.2s |
| the workers alone, over 4 | 36m09s ± 5m12s |

At 4 threads the workers bound it. Per byte: the scanner 7.0 ns a byte of bz2, the workers 4.74 ns a byte of output at 4 threads (CPU, summed over them), 2.65 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,657,661 | 12,411,049 | \`dd91f3870de58914…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
