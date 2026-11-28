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

The latest record: [`4e76c0a`](<https://github.com/lmmx/mibz2/commit/4e76c0ad774486f34ad97720dc2edfc86098877f>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46142638782>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.381 s | 550 | 6.8 | 3.19× (3.18–3.21) | 7.32× (7.28–7.35) | 7.60× (7.56–7.65) | 8.68× (8.61–8.74) | 7.61× (7.56–7.65) | 7.96× (7.92–8.00) | 7.50× (7.48–7.53) | 2.38× (2.36–2.39) | 2.65× (2.64–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 957 | 7.3 | 3.30× (3.15–3.44) | 6.79× (6.54–7.05) | 7.72× (7.54–7.90) | 8.05× (7.86–8.24) | 7.09× (6.89–7.29) | 13.85× (13.52–14.20) | 13.05× (12.76–13.36) | 4.14× (4.04–4.24) | 4.61× (4.49–4.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,166 | 11.4 | 3.28× (3.18–3.39) | 4.84× (4.71–4.97) | 5.98× (5.75–6.21) | 5.87× (5.75–6.00) | 5.79× (5.72–5.87) | 16.82× (16.57–17.08) | 15.85× (15.62–16.09) | 5.03× (4.94–5.11) | 5.60× (5.50–5.71) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,096 | 3.7 | 0.58× (0.54–0.62) | 3.37× (3.33–3.41) | 4.50× (4.45–4.55) | 3.51× (3.47–3.55) | 5.99× (5.93–6.05) | 2.62× (2.59–2.65) | 2.61× (2.59–2.64) | 0.96× (0.92–1.01) | 1.84× (1.67–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,044 | 4.0 | 0.50× (0.49–0.51) | 2.69× (2.64–2.74) | 5.37× (5.28–5.47) | 3.36× (3.30–3.41) | 5.63× (5.54–5.72) | 2.49× (2.45–2.54) | 2.49× (2.45–2.54) | 0.92× (0.86–0.97) | 1.75× (1.57–1.95) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,031 | 4.0 | 0.50× (0.50–0.51) | 2.66× (2.62–2.70) | 5.31× (5.23–5.38) | 3.31× (3.26–3.36) | 5.65× (5.56–5.73) | 2.46× (2.42–2.49) | 2.46× (2.42–2.49) | 0.90× (0.86–0.95) | 1.73× (1.56–1.91) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 770 | 5.1 | 2.83× (2.80–2.85) | 3.20× (3.17–3.22) | 4.00× (3.97–4.03) | 4.31× (4.22–4.41) | 4.68× (4.63–4.73) | 2.89× (2.86–2.91) | 2.36× (2.34–2.38) | 2.19× (2.17–2.21) | 3.19× (3.16–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,146 | 6.3 | 2.14× (1.88–2.45) | 2.88× (2.38–3.47) | 4.29× (3.63–5.08) | 3.67× (3.05–4.42) | 4.91× (4.26–5.65) | 3.93× (3.44–4.49) | 3.22× (2.82–3.67) | 2.98× (2.61–3.41) | 4.34× (3.80–4.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,095 | 12.1 | 2.19× (2.09–2.30) | 2.08× (1.91–2.27) | 3.26× (3.05–3.47) | 2.44× (2.34–2.55) | 4.59× (4.46–4.73) | 4.10× (3.99–4.21) | 3.35× (3.27–3.45) | 3.11× (3.03–3.19) | 4.53× (4.41–4.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.101 s | 79 | 47.6 | 1.49× (1.44–1.54) | 2.86× (2.74–2.97) | 2.56× (2.48–2.65) | 2.70× (2.61–2.78) | 3.03× (2.93–3.14) | 3.22× (3.11–3.33) | 3.23× (3.12–3.33) | 1.35× (1.31–1.39) | 1.52× (1.45–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 131 | 53.0 | 1.40× (1.32–1.49) | 2.78× (2.54–3.05) | 2.58× (2.44–2.72) | 3.34× (3.13–3.55) | 3.12× (2.96–3.29) | 5.26× (4.99–5.55) | 5.28× (5.00–5.57) | 2.21× (2.10–2.32) | 2.48× (2.37–2.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 161 | 70.2 | 1.50× (1.46–1.53) | 2.37× (2.30–2.45) | 2.41× (2.35–2.47) | 2.94× (2.88–3.01) | 3.27× (3.19–3.35) | 6.66× (6.52–6.80) | 6.68× (6.53–6.83) | 2.79× (2.73–2.85) | 3.14× (3.05–3.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 87 | 43.4 | 1.64× (1.62–1.66) | 3.28× (3.26–3.31) | 2.84× (2.82–2.86) | 2.99× (2.96–3.02) | 3.38× (3.27–3.50) | 3.95× (3.94–3.97) | 4.16× (4.13–4.19) | 1.63× (1.62–1.65) | 1.93× (1.91–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 48.9 | 1.51× (1.46–1.56) | 3.16× (2.94–3.40) | 2.77× (2.71–2.82) | 3.90× (3.72–4.09) | 3.38× (3.27–3.51) | 6.56× (6.41–6.70) | 6.90× (6.73–7.06) | 2.71× (2.64–2.77) | 3.20× (3.11–3.29) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 170 | 67.8 | 1.59× (1.51–1.68) | 2.49× (2.38–2.61) | 2.47× (2.39–2.55) | 3.40× (3.31–3.50) | 3.22× (3.15–3.29) | 7.80× (7.65–7.95) | 8.21× (8.06–8.35) | 3.22× (3.15–3.29) | 3.81× (3.71–3.91) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.62–0.63) | 3.45× (3.42–3.47) | 4.34× (4.31–4.37) | 3.10× (3.08–3.11) | 6.69× (6.63–6.75) | 2.27× (2.26–2.29) | 2.31× (2.29–2.34) | 0.94× (0.94–0.95) | 1.73× (1.72–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 5.7 | 0.50× (0.49–0.50) | 2.39× (2.36–2.43) | 4.82× (4.75–4.88) | 2.88× (2.82–2.94) | 6.11× (5.99–6.22) | 2.09× (2.06–2.12) | 2.13× (2.10–2.16) | 0.87× (0.85–0.89) | 1.59× (1.57–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 769 | 5.6 | 0.50× (0.49–0.51) | 2.38× (2.34–2.42) | 4.76× (4.68–4.84) | 2.84× (2.80–2.88) | 6.08× (6.00–6.16) | 2.07× (2.04–2.11) | 2.11× (2.07–2.15) | 0.86× (0.85–0.87) | 1.58× (1.55–1.61) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 0.052 s | 291 | 13.1 | 2.06× (2.04–2.08) | 4.13× (4.06–4.20) | 4.70× (4.63–4.78) | 4.94× (4.90–4.99) | 5.05× (4.99–5.11) | 4.81× (4.76–4.87) | 4.71× (4.66–4.76) | 1.69× (1.67–1.71) | 1.96× (1.92–2.01) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 0.039 s | 386 | 16.9 | 1.50× (1.38–1.63) | 3.24× (2.93–3.58) | 3.83× (3.45–4.25) | 4.46× (4.03–4.93) | 4.14× (3.77–4.54) | 6.26× (5.70–6.87) | 6.12× (5.57–6.73) | 2.20× (2.00–2.42) | 2.55× (2.32–2.81) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 0.032 s | 473 | 26.0 | 1.64× (1.58–1.70) | 2.61× (2.54–2.69) | 3.64× (3.52–3.76) | 3.98× (3.81–4.15) | 4.11× (4.01–4.22) | 7.83× (7.60–8.07) | 7.66× (7.43–7.90) | 2.75× (2.69–2.81) | 3.19× (3.09–3.29) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.18× (3.17–3.20) | 7.28× (7.24–7.32) | 7.56× (7.51–7.61) | 8.64× (8.58–8.71) | 7.58× (7.54–7.62) | 7.93× (7.89–7.97) | 7.47× (7.45–7.50) | 2.37× (2.36–2.39) | 2.63× (2.62–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.03–3.11) | 6.94× (6.88–6.99) | 7.29× (7.22–7.36) | 8.43× (8.37–8.50) | 7.06× (7.00–7.13) | 7.35× (7.30–7.39) | 6.92× (6.89–6.96) | 2.20× (2.18–2.21) | 2.44× (2.42–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.09–3.28) | 5.29× (5.22–5.37) | 5.73× (5.59–5.88) | 6.44× (6.32–6.56) | 5.54× (5.41–5.67) | 4.73× (4.62–4.83) | 4.45× (4.36–4.55) | 1.41× (1.39–1.44) | 1.57× (1.54–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.57–0.67) | 2.89× (2.84–2.93) | 3.88× (3.84–3.92) | 3.29× (3.26–3.32) | 5.56× (5.50–5.61) | 2.48× (2.45–2.50) | 2.48× (2.45–2.50) | 0.97× (0.92–1.01) | 1.62× (1.60–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.55) | 2.54× (2.50–2.58) | 4.57× (4.50–4.63) | 3.07× (3.02–3.11) | 5.06× (4.99–5.14) | 2.28× (2.25–2.31) | 2.28× (2.24–2.32) | 0.89× (0.84–0.94) | 1.49× (1.46–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 2.81× (2.77–2.84) | 4.62× (4.55–4.70) | 3.08× (3.03–3.13) | 5.15× (5.07–5.23) | 2.29× (2.26–2.33) | 2.29× (2.26–2.33) | 0.90× (0.85–0.94) | 1.50× (1.48–1.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.69–2.73) | 2.94× (2.92–2.97) | 3.51× (3.48–3.55) | 4.11× (4.02–4.19) | 4.45× (4.40–4.50) | 2.77× (2.74–2.79) | 2.28× (2.26–2.29) | 2.11× (2.10–2.13) | 2.75× (2.71–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.36–2.41) | 2.61× (2.56–2.65) | 3.50× (3.43–3.57) | 3.49× (3.43–3.54) | 3.77× (3.71–3.84) | 2.25× (2.22–2.28) | 1.85× (1.82–1.88) | 1.72× (1.69–1.75) | 2.23× (2.20–2.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (1.97–2.23) | 1.80× (1.72–1.87) | 2.36× (2.27–2.46) | 2.19× (2.12–2.27) | 2.28× (2.20–2.35) | 1.17× (1.13–1.21) | 0.96× (0.93–1.00) | 0.89× (0.86–0.93) | 1.16× (1.12–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.43–1.53) | 2.75× (2.66–2.85) | 2.51× (2.43–2.59) | 2.67× (2.59–2.75) | 3.00× (2.89–3.10) | 3.18× (3.08–3.29) | 3.19× (3.09–3.29) | 1.34× (1.30–1.38) | 1.47× (1.42–1.52) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.39–1.42) | 2.67× (2.64–2.70) | 2.37× (2.35–2.39) | 3.20× (3.17–3.24) | 2.78× (2.75–2.81) | 2.90× (2.88–2.92) | 2.91× (2.88–2.93) | 1.22× (1.22–1.23) | 1.34× (1.32–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.58–1.64) | 2.51× (2.45–2.56) | 2.13× (2.08–2.19) | 2.87× (2.81–2.94) | 2.74× (2.68–2.81) | 2.19× (2.15–2.23) | 2.20× (2.16–2.23) | 0.93× (0.91–0.94) | 1.01× (0.99–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.61–1.64) | 3.18× (3.16–3.21) | 2.77× (2.75–2.79) | 2.96× (2.93–2.98) | 3.35× (3.24–3.46) | 3.91× (3.89–3.92) | 4.11× (4.08–4.14) | 1.62× (1.61–1.64) | 1.87× (1.85–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.51× (1.50–1.52) | 3.00× (2.97–3.02) | 2.57× (2.54–2.59) | 3.58× (3.55–3.62) | 3.04× (2.95–3.13) | 3.49× (3.46–3.52) | 3.67× (3.64–3.70) | 1.45× (1.43–1.47) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.67–1.78) | 2.70× (2.65–2.76) | 2.17× (2.13–2.21) | 3.08× (3.02–3.15) | 2.86× (2.79–2.92) | 2.50× (2.47–2.54) | 2.63× (2.59–2.67) | 1.04× (1.02–1.06) | 1.20× (1.17–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.46× (2.43–2.49) | 3.39× (3.36–3.41) | 2.83× (2.81–2.85) | 5.98× (5.93–6.02) | 2.12× (2.10–2.13) | 2.15× (2.13–2.17) | 0.95× (0.95–0.96) | 1.36× (1.35–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.12× (2.09–2.15) | 3.75× (3.70–3.80) | 2.59× (2.54–2.63) | 5.26× (5.17–5.35) | 1.87× (1.84–1.90) | 1.90× (1.87–1.93) | 0.84× (0.83–0.86) | 1.21× (1.18–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.34× (2.30–2.38) | 3.79× (3.74–3.84) | 2.62× (2.59–2.66) | 5.38× (5.33–5.44) | 1.90× (1.87–1.93) | 1.93× (1.90–1.97) | 0.86× (0.85–0.87) | 1.23× (1.20–1.25) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 2.03× (2.00–2.05) | 3.98× (3.94–4.03) | 4.47× (4.40–4.54) | 4.81× (4.77–4.86) | 4.92× (4.87–4.98) | 4.69× (4.64–4.74) | 4.59× (4.54–4.64) | 1.67× (1.65–1.69) | 1.84× (1.82–1.87) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 1.65× (1.61–1.69) | 3.26× (3.13–3.41) | 3.59× (3.47–3.71) | 4.23× (4.10–4.36) | 3.88× (3.76–4.01) | 3.60× (3.48–3.73) | 3.52× (3.40–3.65) | 1.28× (1.24–1.32) | 1.42× (1.38–1.45) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 1.73× (1.67–1.79) | 2.63× (2.57–2.68) | 2.98× (2.92–3.04) | 3.38× (3.29–3.47) | 3.15× (3.10–3.20) | 2.39× (2.33–2.45) | 2.34× (2.28–2.40) | 0.85× (0.84–0.86) | 0.94× (0.92–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.79× (0.78–0.79) | 10.99× (10.97–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.37) | 0.43× (0.37–0.50) | 0.72× (0.69–0.74) | 0.61× (0.58–0.64) | 5.75× (5.52–5.99) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.36–0.39) | 0.60× (0.50–0.72) | 0.65× (0.63–0.68) | 0.58× (0.55–0.61) | 3.84× (3.69–3.99) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.23) | 0.17× (0.17–0.18) | 1.91× (1.85–1.97) | 0.90× (0.87–0.93) | 4.73× (4.58–4.89) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 1.00× (0.97–1.04) | 0.12× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.95× (1.94–1.95) | 0.92× (0.92–0.92) | 4.82× (4.81–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.68) | 0.66× (0.65–0.67) | 7.93× (7.91–7.94) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.71) | 0.66× (0.61–0.73) | 1.63× (1.58–1.69) | 0.82× (0.79–0.85) | 5.93× (5.74–6.12) | 0.48× (0.47–0.50) | 0.50× (0.49–0.52) | 0.55× (0.54–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.72× (0.68–0.76) | 0.75× (0.70–0.80) | 1.25× (1.21–1.29) | 0.77× (0.75–0.80) | 3.69× (3.58–3.81) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.34–0.41) | 0.88× (0.86–0.91) | 0.70× (0.67–0.73) | 3.06× (2.96–3.15) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.64–0.66) | 0.45× (0.45–0.46) | 0.86× (0.85–0.87) | 0.79× (0.77–0.82) | 2.51× (2.48–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.84) | 5.22× (5.20–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.41) | 0.86× (0.82–0.89) | 0.72× (0.68–0.76) | 2.93× (2.81–3.06) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.58–0.62) | 0.47× (0.45–0.50) | 0.79× (0.78–0.80) | 0.82× (0.79–0.85) | 2.28× (2.25–2.31) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.05–1.06) | 0.76× (0.76–0.76) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.23–0.24) | 1.86× (1.85–1.86) | 0.89× (0.89–0.90) | 6.17× (6.14–6.19) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.85× (1.84–1.86) | 0.89× (0.88–0.89) | 6.14× (6.12–6.16) | 0.40× (0.39–0.40) | 0.43× (0.42–0.43) | 1.05× (1.04–1.05) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.38) | 0.66× (0.65–0.66) | 6.37× (6.36–6.38) | 0.39× (0.38–0.39) | 0.41× (0.40–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.43× (0.38–0.48) | 1.07× (1.04–1.09) | 0.71× (0.69–0.73) | 3.51× (3.44–3.59) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.35) | 0.17× (0.16–0.17) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 54 MB | 0.51× (0.50–0.52) | 0.46× (0.42–0.50) | 0.93× (0.91–0.95) | 0.76× (0.73–0.79) | 2.39× (2.34–2.44) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 550 | 1.82 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 957 | 1.98 | 1.08× (1.07–1.09) | 87% | 1.03× (1.02–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,166 | 3.08 | 1.68× (1.64–1.72) | 53% | 1.22× (1.20–1.24) | 1.68× (1.62–1.75) |

**Best:** mibz2 at 4 threads (1,166 MB/s); lbzip2 at 4 threads (244 MB/s); pbz2 at 4 threads (354 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.24, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.39, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 14%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.49 | 0.01 | 0.09 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3698 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.6% |
| MTF + RLE2 | 1.49 | 17.9% |
| T-vector build | 1.49 | 17.8% |
| walk (the one each build uses) | 4.13 | 49.4% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.36 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.058 | (in its total) | 2.70× (2.69–2.72) |  |
| walk | 1.048 | 2.062 | (in its total) | 1.96× (1.94–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.042 | (in its total) | 1.12× (1.09–1.15) |  |
| inverse BWT, total | 1.476 | 3.162 | 1.893 | 2.14× (2.12–2.16) | 1.28× (1.27–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m29s ± 6m42s |
| 4 threads | 26m16s ± 3m42s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m45s ± 2m31s |
| the scanner alone (its serial part) | 7m43s ± 30.2s |
| the workers alone, over 4 | 17m45s ± 2m31s |

At 4 threads the workers bound it. Per byte: the scanner 4.5 ns a byte of bz2, the workers 2.33 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,269,316 | 15,087,510 | \`c5dc29887a3b9d42…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3697.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
