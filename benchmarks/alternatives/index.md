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

The latest record: [`3dff14b`](<https://github.com/lmmx/mibz2/commit/3dff14b8bb2249556bc4f9b75ad4edb92284b9b8>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43802244514>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.423 s | 496 | 7.6 | 3.02× (2.97–3.08) | 6.99× (6.87–7.12) | 7.13× (7.02–7.23) | 8.09× (7.96–8.22) | 7.09× (6.94–7.24) | 7.45× (7.33–7.57) | 7.19× (7.04–7.34) | 2.25× (2.18–2.32) | 2.43× (2.40–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.247 s | 849 | 8.4 | 3.00× (2.88–3.13) | 6.24× (6.01–6.48) | 7.35× (6.98–7.74) | 7.32× (7.02–7.64) | 6.55× (6.25–6.87) | 12.63× (12.22–13.06) | 12.19× (11.64–12.76) | 3.82× (3.63–4.01) | 4.12× (3.97–4.27) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.205 s | 1,021 | 13.1 | 3.11× (3.02–3.20) | 4.66× (4.47–4.85) | 6.15× (5.77–6.55) | 5.56× (5.39–5.74) | 5.36× (5.23–5.51) | 15.27× (14.89–15.66) | 14.73× (14.27–15.21) | 4.61× (4.42–4.81) | 4.98× (4.86–5.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,090 | 3.8 | 0.56× (0.55–0.57) | 3.35× (3.28–3.42) | 4.45× (4.36–4.55) | 3.53× (3.46–3.59) | 6.06× (5.87–6.27) | 2.61× (2.55–2.67) | 2.60× (2.55–2.66) | 0.98× (0.95–1.00) | 1.88× (1.70–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,090 | 3.9 | 0.52× (0.51–0.53) | 2.78× (2.73–2.83) | 5.61× (5.42–5.80) | 3.50× (3.45–3.56) | 5.92× (5.77–6.07) | 2.59× (2.55–2.63) | 2.58× (2.54–2.63) | 0.97× (0.94–1.00) | 1.86× (1.68–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,064 | 3.9 | 0.54× (0.49–0.59) | 2.72× (2.63–2.81) | 5.42× (5.23–5.61) | 3.41× (3.30–3.53) | 5.76× (5.59–5.95) | 2.53× (2.43–2.62) | 2.52× (2.43–2.61) | 0.94× (0.90–0.99) | 1.82× (1.65–2.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 721 | 5.6 | 2.63× (2.61–2.65) | 3.01× (2.98–3.04) | 3.76× (3.72–3.79) | 4.04× (3.99–4.08) | 4.41× (4.34–4.48) | 2.71× (2.68–2.74) | 2.23× (2.21–2.25) | 2.10× (2.07–2.13) | 3.00× (2.97–3.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,074 | 7.0 | 2.23× (2.03–2.45) | 2.89× (2.63–3.17) | 4.28× (3.87–4.72) | 3.44× (3.24–3.66) | 5.03× (4.70–5.38) | 3.91× (3.68–4.15) | 3.21× (3.02–3.41) | 3.03× (2.86–3.21) | 4.32× (4.08–4.58) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 912 | 14.1 | 1.85× (1.80–1.91) | 1.85× (1.70–2.01) | 3.03× (2.93–3.15) | 2.11× (1.95–2.28) | 3.83× (3.70–3.97) | 3.42× (3.34–3.51) | 2.81× (2.74–2.88) | 2.66× (2.58–2.73) | 3.78× (3.69–3.88) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 54.1 | 1.39× (1.37–1.42) | 2.63× (2.60–2.67) | 2.43× (2.38–2.49) | 2.45× (2.43–2.48) | 2.75× (2.72–2.79) | 2.96× (2.92–3.00) | 3.07× (3.05–3.10) | 1.26× (1.25–1.28) | 1.43× (1.41–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 115 | 60.9 | 1.27× (1.23–1.31) | 2.44× (2.32–2.57) | 2.34× (2.22–2.48) | 2.92× (2.79–3.05) | 2.81× (2.67–2.96) | 4.71× (4.49–4.95) | 4.90× (4.67–5.14) | 2.02× (1.92–2.11) | 2.28× (2.16–2.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 80.1 | 1.30× (1.29–1.32) | 2.16× (2.09–2.24) | 2.22× (2.12–2.33) | 2.56× (2.53–2.59) | 2.86× (2.80–2.93) | 5.77× (5.65–5.89) | 5.99× (5.86–6.13) | 2.47× (2.41–2.52) | 2.78× (2.73–2.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.122 s | 79 | 48.5 | 1.53× (1.52–1.53) | 3.03× (3.00–3.05) | 2.64× (2.59–2.69) | 2.75× (2.72–2.77) | 3.07× (3.04–3.09) | 3.66× (3.63–3.69) | 4.00× (3.96–4.03) | 1.57× (1.54–1.60) | 1.82× (1.80–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 129 | 55.4 | 1.34× (1.27–1.43) | 2.69× (2.44–2.96) | 2.43× (2.25–2.63) | 3.21× (2.94–3.51) | 2.90× (2.68–3.14) | 5.67× (5.21–6.17) | 6.20× (5.71–6.74) | 2.43× (2.26–2.62) | 2.82× (2.60–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 150 | 77.6 | 1.42× (1.35–1.48) | 2.31× (2.19–2.45) | 2.31× (2.24–2.38) | 2.92× (2.81–3.03) | 2.86× (2.76–2.96) | 6.87× (6.64–7.11) | 7.51× (7.27–7.75) | 2.95× (2.85–3.05) | 3.42× (3.31–3.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 826 | 5.2 | 0.62× (0.61–0.63) | 3.39× (3.36–3.43) | 4.25× (4.21–4.30) | 3.07× (3.04–3.09) | 6.51× (6.40–6.62) | 2.25× (2.23–2.27) | 2.30× (2.26–2.33) | 0.96× (0.94–0.98) | 1.71× (1.69–1.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 787 | 5.6 | 0.51× (0.50–0.52) | 2.44× (2.41–2.48) | 4.89× (4.82–4.96) | 2.94× (2.90–2.98) | 6.33× (6.19–6.46) | 2.15× (2.11–2.18) | 2.19× (2.16–2.23) | 0.92× (0.90–0.93) | 1.63× (1.61–1.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 797 | 5.4 | 0.52× (0.51–0.53) | 2.45× (2.40–2.50) | 4.88× (4.78–4.99) | 2.95× (2.90–3.00) | 6.44× (6.22–6.67) | 2.14× (2.10–2.18) | 2.19× (2.15–2.23) | 0.91× (0.89–0.93) | 1.63× (1.60–1.66) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.060 s | 223 | 17.4 | 1.83× (1.81–1.85) | 4.02× (3.91–4.13) | 4.34× (4.29–4.38) | 4.51× (4.46–4.56) | 4.73× (4.70–4.77) | 4.76× (4.65–4.88) | 4.74× (4.69–4.79) | 1.55× (1.52–1.58) | 1.72× (1.68–1.75) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.040 s | 332 | 21.8 | 1.55× (1.50–1.61) | 3.43× (3.26–3.62) | 4.17× (3.93–4.42) | 4.37× (4.20–4.55) | 4.52× (4.21–4.85) | 7.02× (6.83–7.21) | 6.99× (6.84–7.14) | 2.28× (2.23–2.34) | 2.53× (2.48–2.58) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.037 s | 362 | 33.6 | 1.52× (1.46–1.58) | 2.68× (2.49–2.88) | 3.33× (3.18–3.49) | 3.60× (3.39–3.82) | 4.25× (4.01–4.51) | 7.71× (7.35–8.09) | 7.68× (7.40–7.96) | 2.51× (2.41–2.62) | 2.78× (2.66–2.91) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.02× (2.96–3.07) | 6.96× (6.83–7.08) | 7.09× (6.99–7.20) | 8.06× (7.93–8.19) | 7.06× (6.92–7.21) | 7.42× (7.30–7.54) | 7.16× (7.01–7.31) | 2.24× (2.17–2.32) | 2.41× (2.38–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.91× (2.81–3.02) | 6.48× (6.31–6.65) | 6.89× (6.69–7.10) | 7.71× (7.56–7.87) | 6.58× (6.39–6.78) | 6.73× (6.61–6.84) | 6.49× (6.35–6.63) | 2.03× (1.98–2.09) | 2.18× (2.16–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.07× (2.99–3.15) | 5.11× (4.95–5.28) | 5.95× (5.64–6.26) | 6.11× (5.93–6.30) | 5.24× (5.08–5.40) | 4.33× (4.24–4.42) | 4.18× (4.08–4.27) | 1.31× (1.26–1.36) | 1.40× (1.38–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.88× (2.82–2.94) | 3.84× (3.78–3.91) | 3.29× (3.22–3.35) | 5.61× (5.43–5.80) | 2.47× (2.41–2.53) | 2.46× (2.41–2.51) | 0.98× (0.95–1.00) | 1.61× (1.58–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.59) | 2.69× (2.64–2.74) | 4.98× (4.78–5.18) | 3.26× (3.21–3.31) | 5.42× (5.29–5.55) | 2.41× (2.38–2.45) | 2.41× (2.37–2.45) | 0.96× (0.93–0.98) | 1.57× (1.55–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.52–0.64) | 2.94× (2.85–3.02) | 4.88× (4.73–5.04) | 3.24× (3.14–3.34) | 5.36× (5.20–5.52) | 2.40× (2.32–2.49) | 2.40× (2.32–2.48) | 0.95× (0.91–1.00) | 1.57× (1.52–1.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.51–2.55) | 2.79× (2.77–2.81) | 3.37× (3.34–3.40) | 3.85× (3.81–3.88) | 4.20× (4.13–4.26) | 2.61× (2.58–2.63) | 2.15× (2.13–2.17) | 2.03× (2.01–2.06) | 2.63× (2.59–2.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.21× (2.16–2.25) | 2.45× (2.37–2.53) | 3.26× (3.21–3.32) | 3.21× (3.15–3.27) | 3.52× (3.43–3.61) | 2.06× (2.01–2.11) | 1.70× (1.66–1.74) | 1.61× (1.58–1.64) | 2.08× (2.05–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.87–2.05) | 1.64× (1.57–1.70) | 2.17× (2.11–2.24) | 1.95× (1.88–2.02) | 2.01× (1.94–2.09) | 1.04× (1.00–1.07) | 0.85× (0.83–0.88) | 0.81× (0.78–0.84) | 1.04× (1.01–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.37–1.41) | 2.57× (2.54–2.60) | 2.36× (2.32–2.40) | 2.43× (2.41–2.46) | 2.73× (2.69–2.76) | 2.93× (2.89–2.97) | 3.04× (3.01–3.07) | 1.26× (1.25–1.28) | 1.37× (1.35–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.40× (2.37–2.43) | 2.14× (2.12–2.18) | 2.78× (2.74–2.81) | 2.50× (2.46–2.54) | 2.59× (2.56–2.63) | 2.69× (2.66–2.73) | 1.12× (1.10–1.13) | 1.21× (1.20–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.35–1.48) | 2.34× (2.26–2.44) | 2.02× (1.90–2.15) | 2.58× (2.52–2.65) | 2.52× (2.46–2.59) | 1.96× (1.94–1.99) | 2.04× (2.01–2.07) | 0.84× (0.83–0.86) | 0.92× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.52) | 2.96× (2.94–2.98) | 2.57× (2.54–2.61) | 2.72× (2.70–2.75) | 3.04× (3.01–3.06) | 3.62× (3.59–3.65) | 3.95× (3.92–3.99) | 1.56× (1.53–1.59) | 1.77× (1.76–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.41) | 2.75× (2.70–2.80) | 2.32× (2.28–2.36) | 3.13× (3.06–3.21) | 2.70× (2.65–2.76) | 3.14× (3.07–3.21) | 3.43× (3.35–3.51) | 1.35× (1.33–1.38) | 1.54× (1.51–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.53–1.64) | 2.57× (2.50–2.64) | 2.06× (1.99–2.13) | 2.80× (2.73–2.87) | 2.64× (2.55–2.73) | 2.27× (2.22–2.31) | 2.48× (2.43–2.52) | 0.98× (0.95–1.00) | 1.11× (1.09–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.67) | 2.44× (2.42–2.46) | 3.41× (3.35–3.47) | 2.80× (2.77–2.83) | 5.81× (5.71–5.91) | 2.09× (2.07–2.11) | 2.13× (2.10–2.17) | 0.96× (0.95–0.98) | 1.36× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.22× (2.20–2.24) | 4.07× (4.02–4.12) | 2.71× (2.68–2.75) | 5.58× (5.47–5.69) | 1.96× (1.94–1.99) | 2.00× (1.98–2.03) | 0.91× (0.89–0.92) | 1.28× (1.26–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.51× (2.46–2.57) | 4.21× (4.11–4.31) | 2.80× (2.75–2.85) | 5.85× (5.64–6.06) | 2.02× (1.98–2.06) | 2.06× (2.02–2.10) | 0.93× (0.91–0.95) | 1.31× (1.28–1.34) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.81× (1.79–1.82) | 3.82× (3.73–3.92) | 4.16× (4.12–4.20) | 4.41× (4.37–4.46) | 4.63× (4.60–4.66) | 4.65× (4.54–4.77) | 4.63× (4.59–4.68) | 1.53× (1.51–1.56) | 1.63× (1.62–1.64) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.54× (1.51–1.57) | 3.29× (3.17–3.40) | 3.71× (3.55–3.87) | 4.07× (4.00–4.13) | 3.88× (3.59–4.20) | 3.70× (3.63–3.78) | 3.69× (3.64–3.73) | 1.22× (1.21–1.24) | 1.30× (1.28–1.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.64× (1.58–1.71) | 2.78× (2.62–2.96) | 3.12× (2.91–3.35) | 3.42× (3.31–3.54) | 3.45× (3.16–3.76) | 2.40× (2.30–2.49) | 2.39× (2.32–2.45) | 0.79× (0.77–0.81) | 0.84× (0.82–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.04× (11.02–11.06) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.39× (0.33–0.46) | 0.68× (0.66–0.71) | 0.57× (0.54–0.59) | 5.44× (5.23–5.65) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.33–0.36) | 0.55× (0.46–0.66) | 0.63× (0.61–0.66) | 0.54× (0.52–0.57) | 3.61× (3.47–3.76) | 0.08× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.41) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.07) | 0.65× (0.65–0.66) | 0.69× (0.68–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.70) | 0.72× (0.63–0.82) | 1.61× (1.55–1.67) | 0.80× (0.78–0.83) | 5.84× (5.64–6.06) | 0.47× (0.46–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.69) | 0.69× (0.65–0.73) | 1.16× (1.12–1.19) | 0.71× (0.69–0.73) | 3.38× (3.30–3.47) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.47–0.53) | 0.41× (0.38–0.44) | 0.90× (0.85–0.94) | 0.71× (0.67–0.75) | 3.10× (2.95–3.26) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.45× (0.44–0.47) | 0.87× (0.85–0.89) | 0.81× (0.78–0.84) | 2.53× (2.48–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.21–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.84× (0.81–0.87) | 0.69× (0.66–0.73) | 2.87× (2.77–2.98) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.51× (0.48–0.54) | 0.81× (0.80–0.82) | 0.81× (0.79–0.83) | 2.33× (2.31–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.22–0.25) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.26× (0.25–0.26) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.76× (0.76–0.77) | 5.32× (5.29–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.68× (0.67–0.68) | 6.43× (6.41–6.45) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.40× (0.36–0.44) | 1.02× (1.00–1.05) | 0.66× (0.64–0.68) | 3.26× (3.20–3.33) | 0.20× (0.20–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 58 MB | 0.49× (0.47–0.50) | 0.46× (0.40–0.52) | 0.88× (0.85–0.90) | 0.73× (0.72–0.75) | 2.25× (2.20–2.29) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 496 | 2.02 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 849 | 2.23 | 1.10× (1.09–1.11) | 85% | 1.03× (1.01–1.05) | 1.07× (1.04–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 1,021 | 3.46 | 1.71× (1.69–1.74) | 51% | 1.26× (1.22–1.30) | 1.75× (1.70–1.80) |

**Best:** mibz2 at 4 threads (1,021 MB/s); lbzip2 at 4 threads (221 MB/s); pbz2 at 4 threads (326 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.21, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.56 | 0.02 | 0.07 | 0.205 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3765 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.5% |
| MTF + RLE2 | 1.62 | 17.7% |
| T-vector build | 1.54 | 16.9% |
| walk (the one each build uses) | 4.38 | 47.9% |
| of which: segment assembly | 0.20 | 2.1% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 9.14 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.396 | 1.067 | (in its total) | 2.69× (2.68–2.71) |  |
| walk | 1.048 | 2.042 | (in its total) | 1.95× (1.93–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.041 | (in its total) | 1.09× (1.06–1.12) |  |
| inverse BWT, total | 1.482 | 3.150 | 1.878 | 2.13× (2.11–2.14) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m01s |
| 4 threads | 30m01s ± 4m17s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m11s ± 2m53s |
| the scanner alone (its serial part) | 11m30s ± 19.7s |
| the workers alone, over 4 | 20m11s ± 2m53s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.65 ns a byte of output at 4 threads (CPU, summed over them), 1.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,453 | 13,353,938 | \`724a67aca091a7ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3765.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
