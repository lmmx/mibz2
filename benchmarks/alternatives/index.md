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

The latest record: [`7315dce`](<https://github.com/lmmx/mibz2/commit/7315dce4bf1070f86015017f86e6ac06cc3568f4>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43599347430>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.526 s | 399 | 6.7 | 2.90× (2.87–2.93) | 6.67× (6.62–6.73) | 6.84× (6.78–6.90) | 7.43× (7.37–7.49) | 6.73× (6.68–6.78) | 7.29× (7.21–7.37) | 6.84× (6.79–6.88) | 2.13× (2.11–2.15) | 2.46× (2.44–2.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.308 s | 681 | 7.3 | 2.74× (2.61–2.87) | 5.84× (5.55–6.15) | 6.72× (6.32–7.14) | 6.60× (6.25–6.97) | 5.99× (5.70–6.29) | 12.25× (11.64–12.88) | 11.48× (10.94–12.05) | 3.58× (3.42–3.75) | 4.13× (3.93–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.262 s | 801 | 12.1 | 2.69× (2.64–2.73) | 4.14× (4.07–4.22) | 5.05× (4.98–5.13) | 4.66× (4.56–4.76) | 4.68× (4.60–4.76) | 14.53× (14.32–14.75) | 13.63× (13.40–13.86) | 4.25× (4.18–4.32) | 4.90× (4.82–4.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 993 | 3.0 | 0.60× (0.59–0.61) | 3.64× (3.51–3.78) | 4.49× (4.42–4.56) | 3.60× (3.55–3.65) | 6.16× (6.07–6.25) | 3.01× (2.96–3.05) | 2.69× (2.65–2.73) | 1.01× (1.00–1.02) | 1.53× (1.50–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 983 | 3.0 | 0.57× (0.57–0.58) | 3.03× (2.98–3.07) | 5.98× (5.89–6.07) | 3.59× (3.54–3.63) | 6.09× (6.01–6.16) | 2.98× (2.94–3.03) | 2.67× (2.63–2.71) | 1.00× (0.98–1.03) | 1.51× (1.49–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 966 | 3.0 | 0.57× (0.56–0.58) | 2.99× (2.96–3.03) | 5.89× (5.81–5.96) | 3.52× (3.48–3.57) | 6.00× (5.92–6.09) | 2.94× (2.90–2.97) | 2.63× (2.59–2.66) | 0.99× (0.97–1.00) | 1.49× (1.47–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 592 | 4.7 | 2.66× (2.63–2.69) | 3.08× (3.04–3.11) | 3.98× (3.86–4.10) | 3.85× (3.81–3.89) | 4.15× (4.12–4.18) | 2.59× (2.56–2.61) | 2.14× (2.12–2.16) | 2.05× (2.02–2.07) | 3.06× (3.03–3.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 889 | 5.9 | 2.26× (2.10–2.44) | 2.97× (2.66–3.31) | 4.13× (3.88–4.40) | 3.54× (3.27–3.83) | 4.76× (4.45–5.08) | 3.83× (3.61–4.06) | 3.17× (2.99–3.36) | 3.03× (2.85–3.21) | 4.53× (4.28–4.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 803 | 11.4 | 1.89× (1.84–1.93) | 1.98× (1.82–2.15) | 3.02× (2.87–3.18) | 2.25× (2.18–2.32) | 3.94× (3.84–4.04) | 3.48× (3.40–3.56) | 2.88× (2.81–2.95) | 2.75× (2.68–2.82) | 4.12× (4.03–4.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 45.0 | 1.34× (1.33–1.35) | 2.67× (2.66–2.68) | 2.39× (2.35–2.44) | 2.42× (2.41–2.43) | 2.82× (2.78–2.85) | 3.10× (3.09–3.12) | 3.13× (3.12–3.15) | 1.20× (1.20–1.21) | 1.38× (1.36–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 50.5 | 1.31× (1.11–1.53) | 2.44× (2.27–2.62) | 2.28× (2.12–2.45) | 2.78× (2.58–2.99) | 2.81× (2.60–3.04) | 4.88× (4.53–5.25) | 4.93× (4.58–5.30) | 1.89× (1.76–2.04) | 2.16× (2.03–2.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 69.7 | 1.30× (1.27–1.34) | 2.08× (2.03–2.13) | 2.01× (1.94–2.08) | 2.42× (2.38–2.46) | 2.75× (2.69–2.81) | 5.96× (5.82–6.09) | 6.02× (5.89–6.14) | 2.31× (2.26–2.36) | 2.64× (2.59–2.70) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 41.1 | 1.46× (1.45–1.48) | 3.01× (2.99–3.04) | 2.54× (2.49–2.58) | 2.60× (2.57–2.62) | 3.08× (3.00–3.16) | 3.60× (3.57–3.63) | 3.87× (3.84–3.91) | 1.37× (1.36–1.39) | 1.66× (1.65–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 46.8 | 1.33× (1.25–1.41) | 2.67× (2.47–2.88) | 2.37× (2.23–2.51) | 3.18× (3.00–3.36) | 2.96× (2.78–3.16) | 5.81× (5.48–6.16) | 6.25× (5.88–6.63) | 2.22× (2.09–2.35) | 2.68× (2.52–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 125 | 66.4 | 1.43× (1.41–1.46) | 2.24× (2.17–2.30) | 2.19× (2.12–2.27) | 2.86× (2.79–2.92) | 2.78× (2.74–2.82) | 6.86× (6.75–6.98) | 7.38× (7.25–7.50) | 2.62× (2.57–2.66) | 3.16× (3.11–3.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 755 | 4.0 | 0.68× (0.67–0.68) | 3.11× (3.08–3.13) | 4.55× (4.51–4.60) | 3.18× (3.15–3.20) | 6.87× (6.80–6.94) | 2.59× (2.57–2.61) | 2.38× (2.36–2.40) | 0.99× (0.98–0.99) | 1.56× (1.55–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 719 | 4.3 | 0.61× (0.56–0.67) | 2.96× (2.94–2.99) | 5.16× (5.01–5.33) | 3.06× (3.04–3.08) | 6.63× (6.56–6.70) | 2.47× (2.45–2.49) | 2.27× (2.25–2.28) | 0.94× (0.93–0.95) | 1.49× (1.48–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 704 | 4.3 | 0.57× (0.56–0.58) | 2.91× (2.88–2.95) | 5.01× (4.95–5.06) | 3.00× (2.98–3.03) | 6.43× (6.34–6.52) | 2.42× (2.40–2.45) | 2.23× (2.20–2.25) | 0.92× (0.91–0.93) | 1.46× (1.45–1.48) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.071 s | 184 | 14.9 | 1.97× (1.94–2.01) | 4.18× (4.15–4.21) | 4.44× (4.39–4.49) | 4.48× (4.45–4.50) | 4.68× (4.64–4.72) | 5.26× (5.23–5.30) | 4.90× (4.85–4.95) | 1.50× (1.49–1.52) | 1.87× (1.85–1.88) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.050 s | 260 | 18.6 | 1.57× (1.48–1.67) | 3.24× (3.05–3.45) | 3.82× (3.58–4.08) | 3.94× (3.74–4.14) | 4.10× (3.86–4.35) | 7.32× (6.90–7.77) | 6.82× (6.41–7.25) | 2.09× (1.97–2.22) | 2.60× (2.45–2.75) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.045 s | 287 | 28.3 | 1.50× (1.45–1.55) | 2.37× (2.30–2.44) | 3.09× (2.97–3.21) | 3.09× (3.02–3.16) | 3.66× (3.59–3.73) | 8.20× (8.04–8.35) | 7.63× (7.45–7.81) | 2.34× (2.29–2.39) | 2.91× (2.85–2.96) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.89× (2.87–2.92) | 6.64× (6.59–6.70) | 6.80× (6.74–6.87) | 7.40× (7.34–7.47) | 6.70× (6.65–6.75) | 7.27× (7.18–7.35) | 6.81× (6.76–6.86) | 2.13× (2.11–2.15) | 2.44× (2.42–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.73× (2.70–2.76) | 6.14× (6.07–6.21) | 6.46× (6.39–6.54) | 7.03× (6.94–7.12) | 6.11× (6.03–6.18) | 6.62× (6.51–6.73) | 6.21× (6.13–6.28) | 1.94× (1.92–1.96) | 2.22× (2.20–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.63× (2.59–2.67) | 4.42× (4.35–4.49) | 4.71× (4.63–4.78) | 4.98× (4.91–5.05) | 4.44× (4.40–4.47) | 4.01× (3.96–4.05) | 3.76× (3.72–3.80) | 1.17× (1.16–1.19) | 1.35× (1.33–1.36) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.29× (3.24–3.34) | 4.09× (4.04–4.14) | 3.37× (3.33–3.41) | 5.70× (5.63–5.77) | 2.83× (2.79–2.87) | 2.54× (2.50–2.57) | 1.01× (1.00–1.02) | 1.05× (1.04–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.61–0.62) | 3.09× (3.05–3.13) | 5.37× (5.31–5.42) | 3.31× (3.27–3.36) | 5.53× (5.47–5.60) | 2.75× (2.72–2.79) | 2.47× (2.43–2.50) | 0.98× (0.96–1.01) | 1.02× (1.00–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.43× (3.36–3.50) | 5.43× (5.33–5.53) | 3.33× (3.29–3.37) | 5.55× (5.48–5.63) | 2.77× (2.74–2.80) | 2.48× (2.45–2.51) | 0.99× (0.97–1.01) | 1.03× (1.00–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.57× (2.54–2.60) | 2.83× (2.80–2.87) | 3.62× (3.48–3.77) | 3.70× (3.65–3.74) | 3.98× (3.94–4.02) | 2.50× (2.48–2.53) | 2.08× (2.06–2.10) | 1.99× (1.96–2.02) | 2.75× (2.72–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.23–2.29) | 2.50× (2.46–2.53) | 3.07× (3.03–3.11) | 3.30× (3.26–3.34) | 3.37× (3.31–3.43) | 2.02× (1.99–2.04) | 1.68× (1.65–1.70) | 1.60× (1.58–1.63) | 2.22× (2.20–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.02× (1.93–2.11) | 1.75× (1.70–1.80) | 2.21× (2.15–2.28) | 2.15× (2.08–2.22) | 2.13× (2.07–2.20) | 1.04× (1.01–1.07) | 0.86× (0.84–0.89) | 0.83× (0.80–0.85) | 1.14× (1.10–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.35) | 2.61× (2.59–2.62) | 2.35× (2.32–2.38) | 2.40× (2.39–2.41) | 2.79× (2.76–2.82) | 3.07× (3.06–3.09) | 3.10× (3.09–3.12) | 1.20× (1.19–1.21) | 1.34× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.24–1.27) | 2.43× (2.40–2.47) | 2.15× (2.12–2.17) | 2.74× (2.71–2.78) | 2.54× (2.49–2.60) | 2.73× (2.70–2.76) | 2.76× (2.73–2.78) | 1.07× (1.05–1.08) | 1.19× (1.18–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.40–1.49) | 2.22× (2.17–2.28) | 1.83× (1.79–1.87) | 2.42× (2.37–2.47) | 2.35× (2.31–2.40) | 2.00× (1.96–2.05) | 2.02× (1.98–2.06) | 0.78× (0.77–0.80) | 0.87× (0.85–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.44–1.47) | 2.96× (2.94–2.99) | 2.48× (2.45–2.51) | 2.58× (2.55–2.60) | 3.05× (2.98–3.13) | 3.57× (3.54–3.60) | 3.83× (3.80–3.87) | 1.37× (1.36–1.38) | 1.62× (1.60–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.33–1.37) | 2.70× (2.68–2.73) | 2.26× (2.23–2.28) | 3.03× (2.99–3.07) | 2.69× (2.65–2.74) | 3.14× (3.10–3.17) | 3.37× (3.33–3.41) | 1.20× (1.19–1.22) | 1.42× (1.40–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.52–1.66) | 2.50× (2.44–2.56) | 1.90× (1.86–1.94) | 2.67× (2.59–2.75) | 2.49× (2.44–2.54) | 2.23× (2.19–2.27) | 2.40× (2.35–2.45) | 0.86× (0.84–0.88) | 1.01× (0.99–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.80× (2.77–2.82) | 3.65× (3.61–3.69) | 2.92× (2.89–2.95) | 6.16× (6.09–6.22) | 2.41× (2.39–2.43) | 2.22× (2.20–2.24) | 1.00× (0.99–1.01) | 1.00× (0.97–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.67× (0.63–0.72) | 2.66× (2.53–2.80) | 4.39× (4.28–4.49) | 2.81× (2.79–2.83) | 5.84× (5.78–5.90) | 2.24× (2.22–2.26) | 2.06× (2.05–2.08) | 0.93× (0.92–0.94) | 0.93× (0.90–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.82× (2.78–2.86) | 4.32× (4.27–4.37) | 2.84× (2.81–2.86) | 5.82× (5.75–5.89) | 2.26× (2.24–2.28) | 2.08× (2.06–2.10) | 0.94× (0.92–0.95) | 0.94× (0.91–0.98) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.95× (1.91–1.98) | 3.98× (3.96–4.01) | 4.29× (4.26–4.33) | 4.38× (4.36–4.41) | 4.59× (4.55–4.62) | 5.15× (5.11–5.18) | 4.79× (4.74–4.84) | 1.49× (1.48–1.50) | 1.77× (1.75–1.78) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.66× (1.64–1.68) | 3.33× (3.26–3.39) | 3.60× (3.55–3.66) | 3.95× (3.89–4.02) | 3.74× (3.68–3.80) | 4.12× (4.06–4.19) | 3.84× (3.78–3.90) | 1.19× (1.17–1.22) | 1.41× (1.39–1.44) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.72× (1.66–1.77) | 2.65× (2.60–2.70) | 2.87× (2.81–2.93) | 3.17× (3.12–3.21) | 3.04× (3.00–3.09) | 2.70× (2.66–2.73) | 2.51× (2.47–2.55) | 0.78× (0.77–0.79) | 0.93× (0.91–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.35) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.93× (11.90–11.95) | 0.28× (0.28–0.28) | 0.31× (0.31–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.54× (0.46–0.62) | 0.76× (0.75–0.77) | 0.63× (0.62–0.64) | 6.00× (5.96–6.04) | 0.14× (0.14–0.14) | 0.16× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.38–0.38) | 0.72× (0.63–0.83) | 0.68× (0.67–0.69) | 0.60× (0.59–0.61) | 3.93× (3.90–3.97) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.38× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (8.01–8.04) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.69× (0.65–0.74) | 1.61× (1.57–1.65) | 0.81× (0.79–0.83) | 5.83× (5.69–5.97) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.68) | 0.72× (0.67–0.78) | 1.16× (1.12–1.21) | 0.72× (0.69–0.75) | 3.41× (3.29–3.54) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.40× (0.38–0.43) | 0.90× (0.88–0.92) | 0.72× (0.69–0.74) | 3.13× (3.06–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.46) | 0.87× (0.85–0.89) | 0.81× (0.79–0.83) | 2.53× (2.47–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.50) | 0.37× (0.34–0.41) | 0.86× (0.84–0.88) | 0.69× (0.66–0.73) | 2.94× (2.87–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.47–0.51) | 0.80× (0.79–0.81) | 0.79× (0.75–0.83) | 2.31× (2.27–2.34) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.36× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.31–5.34) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.65× (0.65–0.66) | 6.38× (6.36–6.40) | 0.38× (0.38–0.38) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 35 MB | 0.48× (0.47–0.49) | 0.45× (0.41–0.50) | 1.12× (1.11–1.14) | 0.72× (0.69–0.76) | 3.55× (3.48–3.61) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.35) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.51× (0.47–0.56) | 0.93× (0.90–0.96) | 0.75× (0.72–0.78) | 2.37× (2.30–2.44) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 399 | 2.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 681 | 2.75 | 1.10× (1.09–1.11) | 84% | 1.01× (1.01–1.02) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 801 | 4.54 | 1.81× (1.79–1.83) | 50% | 1.21× (1.19–1.22) | 1.65× (1.62–1.67) |

**Best:** mibz2 at 4 threads (801 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (297 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.20, lbzip2 1t 0.09, lbzip2 2t 0.30, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.74 | 0.02 | 0.09 | 0.262 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2661 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.93 | 12.7% |
| MTF + RLE2 | 1.30 | 17.8% |
| T-vector build | 1.24 | 17.0% |
| walk (the one each build uses) | 3.40 | 46.5% |
| of which: segment assembly | 0.13 | 1.8% |
| RLE1 | 0.23 | 3.2% |
| CRC | 0.21 | 2.9% |
| **total** | 7.30 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.483 | 1.303 | (in its total) | 2.70× (2.69–2.71) |  |
| walk | 1.214 | 2.672 | (in its total) | 2.20× (2.18–2.22) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.19× (1.18–1.20) |  |
| inverse BWT, total | 1.744 | 4.030 | 3.250 | 2.31× (2.30–2.33) | 1.87× (1.86–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m40s |
| 4 threads | 38m19s ± 5m24s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m08s ± 3m51s |
| the scanner alone (its serial part) | 14m42s ± 19.7s |
| the workers alone, over 4 | 27m08s ± 3m51s |

At 4 threads the workers bound it. Per byte: the scanner 8.5 ns a byte of bz2, the workers 3.56 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2660.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
