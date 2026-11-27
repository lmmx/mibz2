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

The latest record: [`cb4afa1`](<https://github.com/lmmx/mibz2/commit/cb4afa18333a3e82eb2a500d8ae155f4d3e996d5>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45955456600>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.381 s | 550 | 7.0 | 3.64× (3.58–3.71) | 8.27× (8.11–8.45) | 8.82× (8.66–8.98) | 9.72× (9.52–9.92) | 8.86× (8.71–9.01) | 9.22× (9.04–9.39) | 9.05× (8.88–9.22) | 2.85× (2.79–2.91) | 2.51× (2.46–2.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.223 s | 938 | 8.0 | 3.32× (3.16–3.48) | 7.13× (6.81–7.47) | 8.55× (8.12–9.00) | 8.54× (8.14–8.96) | 7.76× (7.39–8.15) | 15.45× (14.74–16.20) | 15.17× (14.47–15.92) | 4.78× (4.55–5.02) | 4.22× (4.01–4.43) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.185 s | 1,135 | 12.6 | 3.44× (3.39–3.50) | 6.36× (6.23–6.49) | 7.49× (7.33–7.65) | 7.41× (7.29–7.54) | 7.24× (7.11–7.37) | 19.17× (18.78–19.56) | 18.82× (18.43–19.21) | 5.93× (5.81–6.05) | 5.23× (5.13–5.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,223 | 3.5 | 0.64× (0.61–0.68) | 3.22× (2.97–3.49) | 4.59× (4.23–4.98) | 3.81× (3.57–4.07) | 6.52× (6.10–6.98) | 2.90× (2.69–3.13) | 2.93× (2.72–3.15) | 1.13× (1.08–1.19) | 1.30× (1.20–1.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,156 | 3.7 | 0.55× (0.53–0.56) | 3.00× (2.88–3.13) | 6.06× (5.81–6.33) | 3.56× (3.45–3.68) | 6.08× (5.87–6.29) | 2.70× (2.61–2.80) | 2.73× (2.63–2.83) | 1.05× (1.03–1.08) | 1.21× (1.16–1.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,156 | 3.7 | 0.54× (0.53–0.56) | 2.95× (2.86–3.04) | 5.71× (5.46–5.98) | 3.52× (3.43–3.62) | 5.97× (5.78–6.16) | 2.65× (2.58–2.72) | 2.68× (2.61–2.75) | 1.04× (1.00–1.07) | 1.19× (1.15–1.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 633 | 6.3 | 2.96× (2.92–3.01) | 3.31× (3.27–3.35) | 4.01× (3.96–4.06) | 4.29× (4.25–4.33) | 4.73× (4.69–4.77) | 3.17× (3.15–3.20) | 2.93× (2.90–2.95) | 2.06× (2.03–2.09) | 2.08× (1.96–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,037 | 7.3 | 2.57× (2.38–2.77) | 3.19× (2.96–3.44) | 4.54× (4.25–4.85) | 3.91× (3.66–4.18) | 5.33× (4.96–5.73) | 5.09× (4.74–5.46) | 4.69× (4.36–5.04) | 3.30× (3.04–3.57) | 3.33× (2.98–3.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,080 | 12.7 | 2.19× (2.09–2.30) | 2.32× (2.18–2.46) | 3.29× (3.01–3.60) | 2.75× (2.54–2.98) | 4.62× (4.38–4.86) | 5.36× (5.12–5.62) | 4.94× (4.71–5.18) | 3.47× (3.32–3.63) | 3.51× (3.27–3.76) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.104 s | 77 | 50.5 | 1.52× (1.51–1.53) | 3.37× (3.34–3.40) | 3.08× (3.04–3.11) | 3.15× (3.14–3.17) | 3.47× (3.46–3.49) | 3.83× (3.81–3.85) | 3.94× (3.92–3.95) | 1.38× (1.37–1.38) | 1.37× (1.35–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 54.1 | 1.49× (1.45–1.52) | 3.28× (3.12–3.44) | 3.24× (3.16–3.31) | 3.93× (3.83–4.03) | 3.62× (3.53–3.71) | 6.61× (6.46–6.75) | 6.79× (6.64–6.94) | 2.37× (2.32–2.43) | 2.37× (2.31–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 162 | 73.6 | 1.61× (1.56–1.66) | 3.13× (2.99–3.27) | 2.99× (2.88–3.11) | 3.68× (3.59–3.78) | 3.65× (3.53–3.77) | 8.04× (7.81–8.28) | 8.27× (8.04–8.51) | 2.89× (2.81–2.97) | 2.88× (2.79–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 47.4 | 1.54× (1.53–1.55) | 3.71× (3.69–3.73) | 3.15× (3.11–3.19) | 3.32× (3.27–3.37) | 3.64× (3.62–3.67) | 4.46× (4.43–4.49) | 4.79× (4.76–4.82) | 1.58× (1.57–1.59) | 1.64× (1.63–1.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 51.1 | 1.47× (1.41–1.53) | 3.46× (3.32–3.60) | 3.24× (3.11–3.37) | 4.31× (4.13–4.50) | 3.68× (3.53–3.84) | 7.81× (7.49–8.14) | 8.39× (8.03–8.75) | 2.77× (2.66–2.89) | 2.87× (2.74–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 174 | 70.7 | 1.61× (1.56–1.67) | 3.36× (3.28–3.44) | 3.13× (3.05–3.21) | 4.17× (4.06–4.29) | 3.64× (3.54–3.74) | 9.52× (9.27–9.79) | 10.22× (9.96–10.49) | 3.38× (3.29–3.46) | 3.49× (3.40–3.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 889 | 4.9 | 0.68× (0.65–0.71) | 2.78× (2.68–2.89) | 3.83× (3.68–3.99) | 3.11× (3.02–3.21) | 6.75× (6.55–6.94) | 2.39× (2.32–2.46) | 2.43× (2.37–2.50) | 1.02× (1.00–1.04) | 0.95× (0.91–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 826 | 5.5 | 0.52× (0.50–0.53) | 2.61× (2.51–2.72) | 5.32× (5.10–5.55) | 2.96× (2.86–3.05) | 6.35× (6.13–6.57) | 2.24× (2.16–2.32) | 2.28× (2.21–2.36) | 0.96× (0.93–0.99) | 0.89× (0.85–0.92) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 833 | 5.3 | 0.52× (0.50–0.54) | 2.59× (2.50–2.69) | 5.08× (4.88–5.30) | 2.95× (2.83–3.06) | 6.27× (6.10–6.44) | 2.22× (2.16–2.29) | 2.26× (2.20–2.32) | 0.95× (0.92–0.98) | 0.88× (0.85–0.91) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 0.067 s | 269 | 14.6 | 2.13× (2.12–2.14) | 4.57× (4.54–4.60) | 5.14× (5.10–5.18) | 5.17× (5.14–5.21) | 5.19× (5.16–5.22) | 5.42× (5.39–5.45) | 5.41× (5.38–5.44) | 1.69× (1.68–1.70) | 1.67× (1.66–1.68) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 0.041 s | 438 | 17.1 | 1.78× (1.59–2.00) | 3.86× (3.43–4.33) | 5.04× (4.48–5.68) | 4.96× (4.45–5.52) | 4.77× (4.30–5.30) | 8.42× (7.54–9.41) | 8.40× (7.52–9.38) | 2.62× (2.35–2.92) | 2.60× (2.33–2.90) | yes |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 0.035 s | 509 | 26.1 | 1.88× (1.82–1.94) | 3.32× (3.23–3.41) | 4.46× (4.34–4.58) | 4.52× (4.43–4.61) | 4.66× (4.58–4.74) | 10.30× (10.06–10.55) | 10.28× (10.05–10.51) | 3.20× (3.13–3.28) | 3.18× (3.11–3.26) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.63× (3.56–3.70) | 8.23× (8.07–8.40) | 8.77× (8.61–8.93) | 9.68× (9.48–9.88) | 8.82× (8.68–8.97) | 9.18× (9.01–9.35) | 9.01× (8.84–9.18) | 2.84× (2.79–2.90) | 2.50× (2.44–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.22× (3.19–3.24) | 7.38× (7.27–7.50) | 8.11× (8.05–8.17) | 8.87× (8.81–8.93) | 7.71× (7.62–7.80) | 8.06× (7.97–8.15) | 7.91× (7.83–8.00) | 2.50× (2.47–2.52) | 2.19× (2.17–2.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.29× (3.23–3.35) | 6.73× (6.56–6.91) | 7.19× (7.02–7.37) | 7.85× (7.64–8.06) | 7.00× (6.82–7.17) | 5.16× (5.03–5.30) | 5.07× (4.94–5.19) | 1.60× (1.56–1.64) | 1.40× (1.37–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.65–0.71) | 2.95× (2.73–3.18) | 3.97× (3.71–4.25) | 3.55× (3.32–3.79) | 6.01× (5.61–6.44) | 2.73× (2.53–2.94) | 2.75× (2.56–2.96) | 1.12× (1.07–1.18) | 0.83× (0.78–0.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.60) | 2.71× (2.60–2.82) | 5.21× (5.02–5.41) | 3.25× (3.14–3.35) | 5.45× (5.26–5.65) | 2.47× (2.38–2.56) | 2.49× (2.40–2.59) | 1.02× (0.99–1.04) | 0.75× (0.73–0.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.55–0.59) | 3.09× (2.95–3.24) | 5.11× (4.94–5.28) | 3.26× (3.17–3.35) | 5.44× (5.27–5.62) | 2.47× (2.40–2.54) | 2.50× (2.43–2.57) | 1.02× (0.99–1.05) | 0.75× (0.73–0.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.86× (2.81–2.91) | 3.09× (3.05–3.13) | 3.59× (3.55–3.64) | 4.12× (4.08–4.17) | 4.54× (4.49–4.58) | 3.06× (3.03–3.09) | 2.83× (2.79–2.86) | 2.00× (1.97–2.04) | 1.91× (1.88–1.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.60× (2.50–2.71) | 2.89× (2.77–3.01) | 3.53× (3.40–3.65) | 3.70× (3.56–3.84) | 4.08× (3.93–4.24) | 2.67× (2.58–2.76) | 2.46× (2.37–2.56) | 1.75× (1.67–1.82) | 1.66× (1.59–1.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.33× (2.22–2.44) | 2.05× (1.95–2.16) | 2.50× (2.38–2.61) | 2.46× (2.35–2.58) | 2.59× (2.48–2.71) | 1.56× (1.49–1.63) | 1.44× (1.37–1.51) | 1.02× (0.97–1.07) | 0.97× (0.92–1.02) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.51× (1.50–1.52) | 3.28× (3.27–3.30) | 3.00× (2.99–3.01) | 3.12× (3.10–3.13) | 3.43× (3.42–3.45) | 3.79× (3.76–3.81) | 3.89× (3.87–3.91) | 1.37× (1.36–1.38) | 1.35× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.44× (1.41–1.47) | 3.16× (3.09–3.22) | 2.87× (2.81–2.93) | 3.64× (3.57–3.72) | 3.19× (3.13–3.26) | 3.50× (3.43–3.57) | 3.59× (3.52–3.66) | 1.27× (1.24–1.29) | 1.24× (1.22–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.75× (1.70–1.79) | 3.23× (3.09–3.37) | 2.81× (2.74–2.89) | 3.61× (3.52–3.71) | 3.22× (3.15–3.30) | 2.60× (2.53–2.66) | 2.67× (2.61–2.74) | 0.94× (0.92–0.96) | 0.92× (0.90–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.52–1.55) | 3.61× (3.59–3.64) | 3.08× (3.05–3.11) | 3.28× (3.23–3.33) | 3.60× (3.58–3.63) | 4.41× (4.38–4.44) | 4.73× (4.70–4.77) | 1.57× (1.56–1.58) | 1.57× (1.56–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.47× (1.43–1.50) | 3.45× (3.38–3.52) | 2.98× (2.91–3.05) | 3.99× (3.87–4.10) | 3.35× (3.25–3.44) | 4.07× (3.96–4.18) | 4.37× (4.25–4.50) | 1.45× (1.41–1.49) | 1.45× (1.41–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.72–1.83) | 3.59× (3.52–3.65) | 2.74× (2.67–2.82) | 3.82× (3.72–3.92) | 3.28× (3.22–3.34) | 2.97× (2.91–3.02) | 3.19× (3.13–3.24) | 1.06× (1.04–1.08) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.69–0.74) | 2.45× (2.37–2.53) | 3.37× (3.28–3.45) | 2.84× (2.75–2.93) | 5.99× (5.80–6.18) | 2.21× (2.14–2.28) | 2.24× (2.18–2.31) | 1.02× (1.01–1.04) | 0.75× (0.72–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.56–0.57) | 2.15× (2.09–2.22) | 4.17× (4.06–4.28) | 2.65× (2.57–2.73) | 5.48× (5.30–5.67) | 2.01× (1.94–2.08) | 2.04× (1.97–2.10) | 0.93× (0.90–0.96) | 0.68× (0.65–0.70) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.58) | 2.57× (2.44–2.70) | 4.22× (4.10–4.33) | 2.73× (2.62–2.83) | 5.59× (5.42–5.77) | 2.05× (2.00–2.11) | 2.08× (2.03–2.14) | 0.95× (0.92–0.98) | 0.69× (0.67–0.72) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 2.10× (2.08–2.12) | 4.37× (4.34–4.41) | 4.97× (4.93–5.00) | 5.07× (5.03–5.11) | 5.09× (5.05–5.12) | 5.31× (5.27–5.35) | 5.30× (5.26–5.34) | 1.67× (1.65–1.68) | 1.54× (1.54–1.55) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 1.84× (1.81–1.87) | 3.86× (3.79–3.93) | 4.58× (4.49–4.66) | 4.79× (4.71–4.86) | 4.35× (4.28–4.43) | 4.51× (4.42–4.59) | 4.50× (4.41–4.58) | 1.41× (1.39–1.44) | 1.31× (1.28–1.33) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 1.96× (1.88–2.04) | 3.42× (3.34–3.49) | 3.80× (3.72–3.89) | 4.19× (4.10–4.29) | 3.81× (3.72–3.90) | 2.97× (2.90–3.04) | 2.96× (2.89–3.03) | 0.93× (0.91–0.95) | 0.86× (0.85–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.27–0.30) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.07× (11.04–11.11) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.37) | 0.51× (0.41–0.62) | 0.74× (0.72–0.77) | 0.62× (0.60–0.64) | 5.91× (5.74–6.08) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.36× (0.35–0.38) | 0.53× (0.44–0.64) | 0.65× (0.62–0.68) | 0.58× (0.55–0.60) | 3.81× (3.64–3.98) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.42–0.45) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 7.99× (7.97–8.02) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.73) | 0.69× (0.62–0.75) | 1.71× (1.68–1.74) | 0.85× (0.84–0.87) | 6.20× (6.10–6.30) | 0.51× (0.50–0.51) | 0.53× (0.52–0.54) | 0.57× (0.56–0.58) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.69× (0.67–0.71) | 0.75× (0.70–0.81) | 1.23× (1.19–1.27) | 0.76× (0.74–0.79) | 3.66× (3.55–3.77) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.18) | 0.73× (0.73–0.74) | 5.18× (5.16–5.21) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.43) | 0.90× (0.87–0.92) | 0.69× (0.67–0.72) | 3.09× (3.01–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.81–0.84) | 2.56× (2.51–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.84) | 5.22× (5.20–5.25) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.46× (0.44–0.48) | 0.36× (0.33–0.39) | 0.81× (0.78–0.85) | 0.68× (0.64–0.71) | 2.79× (2.67–2.92) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.61) | 0.47× (0.44–0.49) | 0.80× (0.78–0.82) | 0.82× (0.79–0.85) | 2.31× (2.26–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.23) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.18× (6.17–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.19× (6.17–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.76 MB → 18 MB, 6.5×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.69× (0.68–0.69) | 6.54× (6.53–6.56) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.64) | 0.31× (0.30–0.31) |
| `text` (2.76 MB → 18 MB, 6.5×) | 2 | 39 MB | 0.44× (0.42–0.46) | 0.43× (0.40–0.48) | 1.04× (0.99–1.08) | 0.66× (0.64–0.69) | 3.31× (3.17–3.46) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.76 MB → 18 MB, 6.5×) | 4 | 57 MB | 0.50× (0.49–0.51) | 0.46× (0.42–0.50) | 0.91× (0.87–0.94) | 0.75× (0.72–0.78) | 2.38× (2.30–2.46) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 550 | 1.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 938 | 2.09 | 1.14× (1.11–1.16) | 84% | 1.02× (1.01–1.04) | 1.01× (1.01–1.01) |
| 4: all 4 CPUs, sharing SMT siblings | 1,135 | 3.29 | 1.78× (1.71–1.85) | 52% | 1.45× (1.44–1.47) | 1.61× (1.60–1.63) |

**Best:** mibz2 at 4 threads (1,135 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (331 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.01, lbzip2 1t 0.04, lbzip2 2t 0.20, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.53 | 0.02 | 0.08 | 0.185 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3815 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 7.7% |
| MTF + RLE2 | 1.59 | 15.2% |
| T-vector build | 1.54 | 14.6% |
| walk (the one each build uses) | 6.18 | 58.8% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.24 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.51 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.375 | 0.944 | (in its total) | 2.52× (2.51–2.52) |  |
| walk | 1.533 | 2.922 | (in its total) | 1.91× (1.90–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.082 | 0.074 | (in its total) | 0.91× (0.90–0.92) |  |
| inverse BWT, total | 1.990 | 3.941 | 2.043 | 1.98× (1.98–1.98) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m50s ± 6m53s |
| 4 threads | 26m51s ± 3m48s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m19s ± 2m45s |
| the scanner alone (its serial part) | 7m25s ± 19.7s |
| the workers alone, over 4 | 19m19s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 4.3 ns a byte of bz2, the workers 2.53 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,762,177 | 17,991,040 | \`04b0571f08562a96…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3815.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
