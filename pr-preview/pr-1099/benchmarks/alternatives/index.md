# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1099/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`07cf47a`](<https://github.com/lmmx/mibz2/commit/07cf47a49f84f376ef83fe64fefa65c1eb8b0bc7>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45950379784>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.385 s | 545 | 6.8 | 3.15× (3.14–3.16) | 7.27× (7.24–7.30) | 7.51× (7.47–7.55) | 8.67× (8.43–8.92) | 7.52× (7.48–7.56) | 7.87× (7.83–7.90) | 7.58× (7.55–7.62) | 2.35× (2.34–2.36) | 2.62× (2.60–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.224 s | 937 | 7.4 | 3.22× (3.11–3.35) | 6.55× (6.31–6.81) | 7.60× (7.26–7.97) | 7.89× (7.61–8.18) | 6.90× (6.63–7.19) | 13.52× (12.98–14.09) | 13.04× (12.50–13.59) | 4.04× (3.88–4.21) | 4.50× (4.31–4.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,162 | 11.6 | 3.21× (3.10–3.32) | 4.78× (4.61–4.95) | 5.94× (5.78–6.09) | 5.83× (5.64–6.03) | 5.64× (5.49–5.80) | 16.70× (16.33–17.09) | 16.10× (15.75–16.45) | 4.99× (4.89–5.09) | 5.56× (5.44–5.68) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,096 | 3.7 | 0.56× (0.55–0.57) | 3.35× (3.26–3.43) | 4.47× (4.35–4.59) | 3.49× (3.41–3.58) | 5.96× (5.83–6.10) | 2.59× (2.53–2.66) | 2.60× (2.53–2.67) | 0.95× (0.92–0.97) | 1.88× (1.69–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,050 | 4.0 | 0.50× (0.50–0.51) | 2.67× (2.64–2.71) | 5.36× (5.29–5.43) | 3.37× (3.33–3.41) | 5.70× (5.64–5.77) | 2.48× (2.45–2.52) | 2.48× (2.45–2.52) | 0.90× (0.88–0.93) | 1.80× (1.62–1.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,013 | 4.1 | 0.49× (0.48–0.51) | 2.60× (2.54–2.67) | 5.20× (5.06–5.34) | 3.26× (3.18–3.34) | 5.62× (5.50–5.74) | 2.41× (2.34–2.47) | 2.41× (2.35–2.47) | 0.88× (0.84–0.91) | 1.74× (1.56–1.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 742 | 5.3 | 2.73× (2.69–2.77) | 3.08× (3.06–3.11) | 3.84× (3.79–3.89) | 4.11× (4.09–4.14) | 4.55× (4.51–4.60) | 2.77× (2.75–2.80) | 2.28× (2.27–2.29) | 2.10× (2.08–2.12) | 3.07× (3.05–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,098 | 6.3 | 2.24× (2.07–2.43) | 2.86× (2.53–3.23) | 4.29× (3.90–4.73) | 3.51× (3.19–3.87) | 5.06× (4.54–5.64) | 4.02× (3.63–4.44) | 3.30× (3.00–3.64) | 3.04× (2.76–3.36) | 4.45× (4.03–4.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,089 | 12.0 | 2.10× (2.00–2.20) | 2.14× (1.96–2.34) | 3.51× (3.17–3.89) | 2.44× (2.34–2.54) | 4.58× (4.33–4.86) | 4.04× (3.84–4.25) | 3.32× (3.15–3.50) | 3.06× (2.90–3.23) | 4.47× (4.25–4.71) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 75 | 50.1 | 1.43× (1.42–1.44) | 2.76× (2.74–2.78) | 2.51× (2.46–2.57) | 2.63× (2.61–2.65) | 2.90× (2.86–2.94) | 3.10× (3.09–3.12) | 3.24× (3.22–3.26) | 1.30× (1.29–1.30) | 1.44× (1.41–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.065 s | 124 | 55.9 | 1.35× (1.30–1.40) | 2.62× (2.52–2.72) | 2.45× (2.35–2.55) | 3.35× (3.19–3.51) | 2.97× (2.83–3.12) | 5.00× (4.79–5.21) | 5.22× (5.01–5.44) | 2.08× (2.00–2.17) | 2.31× (2.21–2.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 154 | 73.5 | 1.41× (1.36–1.46) | 2.27× (2.23–2.32) | 2.25× (2.18–2.32) | 2.83× (2.75–2.90) | 3.13× (3.00–3.27) | 6.29× (6.12–6.46) | 6.57× (6.40–6.74) | 2.62× (2.55–2.69) | 2.91× (2.84–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.115 s | 83 | 45.2 | 1.56× (1.54–1.57) | 3.16× (3.13–3.19) | 2.72× (2.70–2.74) | 2.88× (2.85–2.92) | 3.52× (2.85–4.36) | 3.79× (3.75–3.84) | 4.14× (4.11–4.18) | 1.53× (1.53–1.54) | 1.85× (1.83–1.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 138 | 51.0 | 1.42× (1.40–1.45) | 2.95× (2.81–3.09) | 2.62× (2.56–2.69) | 3.66× (3.53–3.80) | 3.20× (3.10–3.30) | 6.23× (6.02–6.44) | 6.81× (6.59–7.03) | 2.52× (2.45–2.59) | 3.04× (2.97–3.12) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.060 s | 161 | 71.8 | 1.47× (1.44–1.51) | 2.39× (2.25–2.53) | 2.36× (2.27–2.45) | 3.20× (3.13–3.27) | 3.01× (2.93–3.10) | 7.34× (7.12–7.57) | 8.02× (7.80–8.26) | 2.97× (2.89–3.05) | 3.59× (3.51–3.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 5.0 | 0.62× (0.61–0.64) | 3.44× (3.37–3.51) | 4.32× (4.23–4.42) | 3.07× (3.01–3.14) | 6.70× (6.55–6.85) | 2.26× (2.21–2.31) | 2.31× (2.26–2.36) | 0.95× (0.92–0.97) | 1.73× (1.69–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 5.7 | 0.50× (0.49–0.50) | 2.38× (2.34–2.43) | 4.77× (4.67–4.88) | 2.92× (2.81–3.03) | 6.15× (6.02–6.27) | 2.08× (2.05–2.12) | 2.13× (2.09–2.17) | 0.87× (0.86–0.88) | 1.59× (1.56–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 760 | 5.7 | 0.49× (0.48–0.51) | 2.34× (2.29–2.38) | 4.68× (4.59–4.77) | 2.79× (2.74–2.84) | 6.02× (5.92–6.11) | 2.03× (2.00–2.07) | 2.08× (2.04–2.12) | 0.85× (0.83–0.87) | 1.55× (1.52–1.58) | yes |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 1 | 0.061 s | 284 | 13.4 | 2.01× (2.00–2.03) | 4.15× (4.10–4.20) | 4.55× (4.49–4.61) | 4.89× (4.86–4.93) | 4.94× (4.90–4.99) | 4.83× (4.80–4.86) | 4.98× (4.94–5.02) | 1.65× (1.63–1.66) | 1.98× (1.96–1.99) | yes |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 2 | 0.041 s | 424 | 16.6 | 1.63× (1.48–1.79) | 3.49× (3.12–3.91) | 3.95× (3.70–4.22) | 4.76× (4.37–5.18) | 4.41× (4.07–4.78) | 7.07× (6.54–7.64) | 7.30× (6.74–7.89) | 2.41× (2.23–2.61) | 2.90× (2.68–3.13) | yes |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 4 | 0.039 s | 446 | 26.2 | 1.52× (1.48–1.57) | 2.42× (2.33–2.52) | 3.02× (2.90–3.15) | 3.44× (3.36–3.53) | 3.62× (3.51–3.73) | 7.56× (7.34–7.78) | 7.80× (7.57–8.03) | 2.58× (2.50–2.65) | 3.10× (3.01–3.19) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.13–3.15) | 7.23× (7.20–7.26) | 7.47× (7.43–7.51) | 8.64× (8.40–8.89) | 7.49× (7.45–7.53) | 7.83× (7.80–7.87) | 7.55× (7.51–7.59) | 2.34× (2.33–2.35) | 2.60× (2.58–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.03× (2.99–3.06) | 6.79× (6.71–6.86) | 7.18× (7.04–7.33) | 8.28× (8.20–8.36) | 6.95× (6.87–7.03) | 7.23× (7.14–7.32) | 6.97× (6.87–7.06) | 2.16× (2.13–2.19) | 2.40× (2.36–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.07× (3.01–3.13) | 5.11× (5.02–5.20) | 5.57× (5.49–5.65) | 6.24× (6.12–6.37) | 5.39× (5.28–5.50) | 4.65× (4.56–4.75) | 4.49× (4.39–4.58) | 1.39× (1.36–1.43) | 1.54× (1.50–1.58) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.87× (2.81–2.93) | 3.80× (3.70–3.91) | 3.27× (3.20–3.35) | 5.54× (5.42–5.66) | 2.46× (2.40–2.52) | 2.46× (2.40–2.52) | 0.95× (0.93–0.97) | 1.61× (1.56–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.54) | 2.55× (2.52–2.58) | 4.59× (4.55–4.64) | 3.08× (3.05–3.11) | 5.13× (5.08–5.19) | 2.27× (2.25–2.30) | 2.27× (2.25–2.30) | 0.88× (0.86–0.90) | 1.49× (1.47–1.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.50–0.54) | 2.75× (2.68–2.82) | 4.56× (4.43–4.69) | 3.04× (2.97–3.11) | 5.14× (5.04–5.24) | 2.25× (2.20–2.31) | 2.25× (2.20–2.31) | 0.87× (0.84–0.90) | 1.47× (1.44–1.51) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.62× (2.58–2.66) | 2.84× (2.82–2.86) | 3.37× (3.36–3.39) | 3.92× (3.89–3.94) | 4.33× (4.29–4.37) | 2.66× (2.64–2.68) | 2.20× (2.19–2.21) | 2.03× (2.01–2.05) | 2.66× (2.64–2.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.26–2.43) | 2.59× (2.47–2.72) | 3.42× (3.31–3.54) | 3.41× (3.28–3.55) | 3.72× (3.55–3.90) | 2.20× (2.11–2.30) | 1.82× (1.75–1.89) | 1.68× (1.61–1.75) | 2.19× (2.12–2.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.05–2.22) | 1.88× (1.80–1.96) | 2.53× (2.30–2.78) | 2.19× (2.11–2.27) | 2.30× (2.24–2.36) | 1.17× (1.14–1.21) | 0.97× (0.94–1.00) | 0.90× (0.87–0.92) | 1.17× (1.14–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.41–1.43) | 2.67× (2.65–2.68) | 2.47× (2.42–2.51) | 2.60× (2.58–2.62) | 2.87× (2.83–2.91) | 3.07× (3.06–3.09) | 3.21× (3.19–3.23) | 1.29× (1.28–1.30) | 1.42× (1.41–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.35× (1.33–1.36) | 2.53× (2.50–2.56) | 2.25× (2.22–2.27) | 3.06× (3.03–3.09) | 2.64× (2.61–2.66) | 2.75× (2.71–2.79) | 2.87× (2.83–2.92) | 1.16× (1.14–1.17) | 1.27× (1.25–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.54× (1.49–1.60) | 2.41× (2.37–2.46) | 2.03× (1.99–2.08) | 2.79× (2.74–2.85) | 2.69× (2.54–2.86) | 2.10× (2.05–2.16) | 2.20× (2.15–2.25) | 0.88× (0.87–0.90) | 0.97× (0.95–0.99) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.55× (1.54–1.56) | 3.07× (3.05–3.09) | 2.65× (2.62–2.68) | 2.85× (2.82–2.89) | 3.48× (2.82–4.31) | 3.75× (3.70–3.79) | 4.10× (4.06–4.13) | 1.53× (1.52–1.53) | 1.78× (1.75–1.81) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.43× (1.42–1.45) | 2.84× (2.77–2.91) | 2.42× (2.37–2.48) | 3.44× (3.34–3.54) | 2.85× (2.79–2.91) | 3.31× (3.22–3.40) | 3.62× (3.53–3.70) | 1.35× (1.32–1.37) | 1.57× (1.55–1.60) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.55–1.70) | 2.56× (2.47–2.65) | 2.04× (1.99–2.10) | 2.93× (2.86–3.00) | 2.67× (2.60–2.75) | 2.37× (2.30–2.44) | 2.59× (2.52–2.66) | 0.96× (0.94–0.99) | 1.13× (1.10–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.68) | 2.46× (2.38–2.54) | 3.37× (3.32–3.43) | 2.81× (2.75–2.87) | 5.98× (5.85–6.11) | 2.10× (2.06–2.15) | 2.15× (2.10–2.19) | 0.95× (0.93–0.97) | 1.35× (1.32–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.14× (2.08–2.20) | 3.78× (3.72–3.85) | 2.63× (2.54–2.73) | 5.32× (5.22–5.42) | 1.87× (1.84–1.89) | 1.90× (1.87–1.94) | 0.85× (0.83–0.86) | 1.19× (1.17–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.33× (2.28–2.38) | 3.69× (3.61–3.76) | 2.58× (2.53–2.63) | 5.33× (5.25–5.42) | 1.87× (1.84–1.91) | 1.91× (1.87–1.95) | 0.85× (0.83–0.86) | 1.20× (1.17–1.22) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 1 | 1.99× (1.98–2.00) | 3.99× (3.94–4.04) | 4.36× (4.33–4.39) | 4.79× (4.75–4.82) | 4.84× (4.79–4.88) | 4.72× (4.70–4.75) | 4.87× (4.84–4.91) | 1.63× (1.62–1.64) | 1.81× (1.80–1.82) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 2 | 1.72× (1.64–1.80) | 3.38× (3.32–3.43) | 3.71× (3.65–3.76) | 4.43× (4.38–4.48) | 3.99× (3.93–4.05) | 3.82× (3.76–3.89) | 3.94× (3.87–4.01) | 1.32× (1.30–1.33) | 1.46× (1.44–1.49) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 4 | 1.76× (1.70–1.82) | 2.68× (2.61–2.74) | 2.92× (2.86–3.00) | 3.35× (3.29–3.41) | 3.13× (3.07–3.20) | 2.43× (2.39–2.48) | 2.51× (2.47–2.55) | 0.84× (0.82–0.85) | 0.93× (0.91–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.78–0.79) | 10.97× (10.93–11.02) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.48× (0.42–0.56) | 0.72× (0.70–0.74) | 0.60× (0.59–0.62) | 5.74× (5.58–5.90) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.37× (0.34–0.39) | 0.52× (0.41–0.65) | 0.64× (0.60–0.69) | 0.57× (0.54–0.61) | 3.81× (3.58–4.07) | 0.09× (0.08–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.17–0.19) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.83–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.85× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.91–7.98) | 0.65× (0.65–0.66) | 0.68× (0.68–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.67–0.72) | 0.66× (0.60–0.72) | 1.65× (1.59–1.70) | 0.83× (0.80–0.87) | 5.97× (5.78–6.17) | 0.49× (0.47–0.50) | 0.51× (0.49–0.53) | 0.55× (0.54–0.57) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.70× (0.68–0.72) | 0.75× (0.70–0.81) | 1.25× (1.21–1.29) | 0.76× (0.74–0.78) | 3.65× (3.57–3.74) | 0.29× (0.28–0.29) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.15× (5.12–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.43) | 0.90× (0.87–0.93) | 0.72× (0.70–0.75) | 3.11× (3.01–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.66) | 0.46× (0.45–0.46) | 0.87× (0.85–0.88) | 0.81× (0.80–0.83) | 2.52× (2.49–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.17× (1.16–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.21) | 0.32× (0.32–0.33) | 0.34× (0.33–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.35× (0.32–0.39) | 0.84× (0.81–0.87) | 0.71× (0.68–0.75) | 2.88× (2.78–3.00) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.51) | 0.80× (0.79–0.81) | 0.80× (0.77–0.83) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.30× (5.29–5.31) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.19× (6.17–6.21) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.89) | 6.19× (6.17–6.20) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 1 | 20 MB | 0.45× (0.44–0.45) | 0.38× (0.38–0.38) | 1.38× (1.37–1.38) | 0.68× (0.68–0.68) | 6.44× (6.43–6.46) | 0.38× (0.38–0.39) | 0.41× (0.41–0.41) | 0.62× (0.62–0.62) | 0.30× (0.30–0.31) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.44× (0.40–0.48) | 1.06× (1.04–1.09) | 0.70× (0.68–0.72) | 3.56× (3.48–3.64) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.34× (0.34–0.35) | 0.17× (0.16–0.17) |
| `text` (2.69 MB → 17.4 MB, 6.5×) | 4 | 54 MB | 0.53× (0.51–0.54) | 0.50× (0.46–0.54) | 0.95× (0.93–0.98) | 0.79× (0.76–0.81) | 2.50× (2.43–2.57) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 545 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 937 | 1.99 | 1.08× (1.07–1.10) | 86% | 1.02× (1.01–1.02) | 1.04× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,162 | 3.13 | 1.68× (1.65–1.72) | 53% | 1.19× (1.18–1.20) | 1.64× (1.60–1.69) |

**Best:** mibz2 at 4 threads (1,162 MB/s); lbzip2 at 4 threads (245 MB/s); pbz2 at 4 threads (365 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.25, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.01 | 0.09 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3707 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 10.0% |
| MTF + RLE2 | 1.48 | 17.7% |
| T-vector build | 1.50 | 17.9% |
| walk (the one each build uses) | 4.08 | 49.0% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.33 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.066 | (in its total) | 2.71× (2.70–2.72) |  |
| walk | 1.088 | 2.088 | (in its total) | 1.94× (1.91–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.050 | (in its total) | 1.15× (1.07–1.22) |  |
| inverse BWT, total | 1.528 | 3.203 | 1.920 | 2.11× (2.09–2.13) | 1.25× (1.24–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m59s ± 6m42s |
| 4 threads | 26m22s ± 3m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m05s ± 2m37s |
| the scanner alone (its serial part) | 8m09s ± 32.2s |
| the workers alone, over 4 | 18m05s ± 2m37s |

At 4 threads the workers bound it. Per byte: the scanner 4.7 ns a byte of bz2, the workers 2.37 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,694,819 | 17,405,908 | \`b755d996bc984a75…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3707.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
