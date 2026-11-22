# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-719/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`811c889`](<https://github.com/lmmx/mibz2/commit/811c889d498f70aab87f5055c8dff322052fb2c3>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44961520599>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.504 s | 416 | 7.8 | 3.01× (2.99–3.03) | 6.92× (6.87–6.96) | 7.04× (7.00–7.09) | 7.82× (7.78–7.87) | 7.01× (6.95–7.08) | 7.58× (7.54–7.62) | 7.08× (7.02–7.14) | 2.22× (2.21–2.24) | 2.58× (2.56–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.290 s | 724 | 8.6 | 2.93× (2.77–3.09) | 6.16× (5.89–6.45) | 6.95× (6.61–7.31) | 7.03× (6.67–7.40) | 6.32× (6.04–6.62) | 12.84× (12.27–13.44) | 11.99× (11.48–12.53) | 3.77× (3.60–3.95) | 4.37× (4.16–4.59) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.237 s | 885 | 13.3 | 2.95× (2.92–2.97) | 4.48× (4.43–4.53) | 5.55× (5.52–5.58) | 5.12× (5.07–5.17) | 5.19× (5.14–5.23) | 16.11× (16.00–16.23) | 15.05× (14.90–15.20) | 4.73× (4.69–4.77) | 5.48× (5.42–5.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 3.6 | 0.60× (0.59–0.61) | 3.55× (3.41–3.70) | 4.37× (4.29–4.44) | 3.51× (3.46–3.56) | 6.16× (6.09–6.23) | 2.94× (2.90–2.99) | 2.62× (2.58–2.66) | 1.00× (0.99–1.01) | 1.49× (1.46–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 3.7 | 0.58× (0.57–0.59) | 2.94× (2.89–3.00) | 5.84× (5.70–6.00) | 3.51× (3.45–3.57) | 6.05× (5.95–6.15) | 2.91× (2.86–2.97) | 2.60× (2.55–2.64) | 0.99× (0.97–1.01) | 1.47× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 935 | 3.8 | 0.57× (0.56–0.58) | 2.91× (2.86–2.95) | 5.80× (5.65–5.96) | 3.44× (3.39–3.49) | 6.03× (5.93–6.13) | 2.87× (2.82–2.91) | 2.55× (2.52–2.59) | 0.97× (0.95–1.00) | 1.45× (1.43–1.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 586 | 5.8 | 2.74× (2.69–2.78) | 3.06× (3.04–3.07) | 3.92× (3.89–3.95) | 3.96× (3.93–3.98) | 4.24× (4.20–4.27) | 2.57× (2.56–2.59) | 2.12× (2.11–2.13) | 2.02× (2.01–2.03) | 3.04× (3.02–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 912 | 7.1 | 2.42× (2.22–2.64) | 3.08× (2.86–3.32) | 4.28× (4.10–4.47) | 3.82× (3.36–4.35) | 5.09× (4.92–5.26) | 3.98× (3.88–4.08) | 3.28× (3.20–3.37) | 3.12× (3.05–3.20) | 4.71× (4.59–4.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 766 | 14.5 | 1.85× (1.77–1.93) | 1.90× (1.77–2.05) | 2.93× (2.75–3.13) | 2.17× (2.06–2.28) | 3.89× (3.75–4.04) | 3.35× (3.24–3.47) | 2.76× (2.67–2.86) | 2.63× (2.54–2.72) | 3.96× (3.83–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 54.4 | 1.35× (1.34–1.36) | 2.69× (2.68–2.70) | 2.42× (2.37–2.46) | 2.48× (2.47–2.49) | 2.86× (2.84–2.88) | 3.11× (3.09–3.13) | 3.18× (3.17–3.19) | 1.20× (1.19–1.20) | 1.38× (1.37–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.085 s | 94 | 62.1 | 1.30× (1.13–1.49) | 2.41× (2.34–2.48) | 2.30× (2.23–2.37) | 3.09× (2.82–3.39) | 2.85× (2.76–2.93) | 4.91× (4.77–5.05) | 5.02× (4.86–5.18) | 1.89× (1.83–1.95) | 2.17× (2.11–2.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 84.2 | 1.30× (1.27–1.34) | 2.10× (2.06–2.14) | 1.97× (1.90–2.04) | 2.48× (2.43–2.54) | 2.78× (2.72–2.84) | 5.93× (5.81–6.05) | 6.06× (5.95–6.18) | 2.28× (2.24–2.32) | 2.62× (2.57–2.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.145 s | 66 | 49.5 | 1.49× (1.48–1.49) | 3.05× (3.02–3.08) | 2.55× (2.53–2.57) | 2.68× (2.67–2.69) | 3.09× (3.07–3.11) | 3.62× (3.61–3.63) | 3.93× (3.91–3.95) | 1.39× (1.38–1.39) | 1.67× (1.67–1.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 56.4 | 1.35× (1.31–1.38) | 2.75× (2.67–2.84) | 2.44× (2.36–2.53) | 3.43× (3.33–3.54) | 3.04× (2.97–3.12) | 5.90× (5.76–6.05) | 6.41× (6.24–6.58) | 2.26× (2.21–2.32) | 2.72× (2.66–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 79.7 | 1.42× (1.38–1.47) | 2.21× (2.17–2.25) | 2.18× (2.13–2.23) | 2.95× (2.89–3.00) | 2.75× (2.71–2.80) | 6.78× (6.69–6.88) | 7.37× (7.26–7.47) | 2.60× (2.55–2.65) | 3.13× (3.08–3.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 744 | 4.9 | 0.67× (0.66–0.68) | 3.16× (2.97–3.37) | 4.43× (4.32–4.53) | 3.11× (3.04–3.17) | 6.85× (6.68–7.02) | 2.54× (2.50–2.59) | 2.33× (2.29–2.37) | 0.98× (0.97–0.99) | 1.52× (1.49–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 678 | 5.5 | 0.57× (0.56–0.59) | 2.82× (2.77–2.87) | 4.94× (4.77–5.12) | 2.94× (2.90–2.98) | 6.47× (6.40–6.54) | 2.37× (2.33–2.40) | 2.17× (2.14–2.20) | 0.91× (0.89–0.93) | 1.42× (1.39–1.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 676 | 5.4 | 0.57× (0.56–0.58) | 2.79× (2.76–2.82) | 4.77× (4.72–4.83) | 2.90× (2.87–2.93) | 6.32× (6.24–6.39) | 2.33× (2.31–2.36) | 2.14× (2.12–2.17) | 0.90× (0.88–0.92) | 1.40× (1.38–1.41) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 0.071 s | 190 | 17.5 | 2.02× (2.00–2.03) | 4.16× (4.12–4.20) | 4.55× (4.51–4.59) | 4.61× (4.56–4.65) | 4.80× (4.75–4.86) | 5.34× (5.29–5.39) | 5.06× (5.01–5.12) | 1.54× (1.52–1.55) | 1.86× (1.84–1.87) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 0.051 s | 266 | 22.2 | 1.64× (1.55–1.73) | 3.30× (3.16–3.45) | 3.95× (3.81–4.10) | 4.21× (4.09–4.34) | 4.27× (4.15–4.41) | 7.60× (7.38–7.83) | 7.21× (7.00–7.43) | 2.19× (2.13–2.25) | 2.64× (2.57–2.72) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 0.047 s | 288 | 34.1 | 1.49× (1.44–1.55) | 2.42× (2.27–2.57) | 3.14× (3.07–3.22) | 3.15× (3.08–3.21) | 3.64× (3.58–3.71) | 8.16× (8.03–8.29) | 7.74× (7.64–7.85) | 2.35× (2.31–2.39) | 2.84× (2.80–2.88) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.00× (2.98–3.02) | 6.88× (6.84–6.93) | 7.01× (6.96–7.06) | 7.80× (7.75–7.84) | 6.99× (6.93–7.05) | 7.55× (7.51–7.59) | 7.06× (7.00–7.11) | 2.22× (2.21–2.23) | 2.56× (2.54–2.58) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.83× (2.80–2.86) | 6.39× (6.33–6.45) | 6.66× (6.61–6.72) | 7.43× (7.35–7.51) | 6.38× (6.33–6.44) | 6.88× (6.81–6.94) | 6.42× (6.35–6.50) | 2.02× (2.00–2.05) | 2.33× (2.30–2.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.89× (2.85–2.92) | 4.81× (4.77–4.85) | 5.16× (5.11–5.21) | 5.49× (5.43–5.54) | 4.88× (4.83–4.92) | 4.43× (4.40–4.47) | 4.14× (4.10–4.19) | 1.30× (1.29–1.31) | 1.50× (1.49–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.62–0.65) | 3.25× (3.19–3.31) | 4.02× (3.96–4.09) | 3.30× (3.25–3.35) | 5.74× (5.67–5.81) | 2.79× (2.74–2.83) | 2.49× (2.45–2.53) | 1.00× (0.99–1.01) | 1.06× (1.02–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.62–0.64) | 3.05× (2.98–3.11) | 5.31× (5.20–5.42) | 3.27× (3.22–3.33) | 5.55× (5.47–5.64) | 2.71× (2.66–2.77) | 2.42× (2.38–2.47) | 0.97× (0.95–1.00) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.60–0.61) | 3.31× (3.25–3.37) | 5.31× (5.22–5.40) | 3.26× (3.21–3.30) | 5.59× (5.50–5.68) | 2.71× (2.67–2.75) | 2.42× (2.38–2.45) | 0.97× (0.94–1.00) | 1.03× (1.00–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.65× (2.60–2.69) | 2.81× (2.79–2.83) | 3.51× (3.48–3.54) | 3.80× (3.77–3.83) | 4.06× (4.02–4.11) | 2.49× (2.47–2.51) | 2.06× (2.05–2.07) | 1.96× (1.95–1.98) | 2.73× (2.70–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.26–2.33) | 2.54× (2.49–2.58) | 3.11× (3.08–3.14) | 3.30× (3.25–3.34) | 3.49× (3.44–3.54) | 2.03× (2.00–2.07) | 1.68× (1.66–1.70) | 1.60× (1.58–1.63) | 2.23× (2.19–2.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.95–2.14) | 1.73× (1.65–1.82) | 2.17× (2.07–2.27) | 2.04× (1.94–2.15) | 2.13× (2.04–2.22) | 1.02× (0.98–1.07) | 0.85× (0.81–0.89) | 0.81× (0.77–0.84) | 1.12× (1.07–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.63× (2.61–2.64) | 2.36× (2.33–2.39) | 2.46× (2.45–2.47) | 2.84× (2.82–2.85) | 3.08× (3.06–3.10) | 3.15× (3.14–3.16) | 1.19× (1.19–1.20) | 1.35× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.22–1.25) | 2.40× (2.37–2.42) | 2.12× (2.10–2.14) | 2.86× (2.81–2.91) | 2.52× (2.50–2.54) | 2.70× (2.67–2.72) | 2.76× (2.73–2.78) | 1.04× (1.03–1.05) | 1.18× (1.17–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.39–1.48) | 2.23× (2.19–2.27) | 1.81× (1.78–1.85) | 2.51× (2.47–2.55) | 2.35× (2.31–2.39) | 1.99× (1.96–2.03) | 2.04× (2.01–2.07) | 0.77× (0.76–0.79) | 0.87× (0.85–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.48–1.49) | 2.99× (2.98–3.01) | 2.50× (2.48–2.51) | 2.66× (2.65–2.67) | 3.07× (3.05–3.08) | 3.59× (3.57–3.60) | 3.89× (3.88–3.91) | 1.38× (1.38–1.39) | 1.63× (1.62–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.35–1.38) | 2.72× (2.69–2.75) | 2.26× (2.24–2.29) | 3.21× (3.16–3.25) | 2.72× (2.70–2.74) | 3.14× (3.11–3.17) | 3.41× (3.37–3.44) | 1.21× (1.20–1.22) | 1.42× (1.41–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.67) | 2.49× (2.43–2.56) | 1.90× (1.86–1.94) | 2.76× (2.66–2.87) | 2.48× (2.41–2.54) | 2.22× (2.16–2.27) | 2.41× (2.35–2.47) | 0.85× (0.83–0.88) | 1.01× (0.98–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.73) | 2.75× (2.68–2.81) | 3.60× (3.49–3.71) | 2.86× (2.79–2.93) | 6.16× (5.98–6.35) | 2.37× (2.31–2.43) | 2.18× (2.13–2.22) | 0.98× (0.97–1.00) | 0.97× (0.94–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.65) | 2.47× (2.42–2.51) | 4.24× (4.17–4.31) | 2.72× (2.68–2.75) | 5.70× (5.63–5.77) | 2.15× (2.12–2.19) | 1.98× (1.95–2.01) | 0.89× (0.88–0.91) | 0.88× (0.85–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.72× (2.68–2.77) | 4.19× (4.15–4.22) | 2.76× (2.72–2.79) | 5.72× (5.65–5.79) | 2.18× (2.15–2.21) | 2.00× (1.98–2.03) | 0.90× (0.88–0.93) | 0.89× (0.86–0.92) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 1.99× (1.98–2.01) | 4.05× (4.02–4.08) | 4.36× (4.33–4.40) | 4.52× (4.48–4.56) | 4.71× (4.66–4.75) | 5.23× (5.18–5.27) | 4.96× (4.91–5.00) | 1.52× (1.51–1.54) | 1.80× (1.78–1.81) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 1.67× (1.64–1.71) | 3.34× (3.27–3.42) | 3.65× (3.57–3.72) | 4.11× (4.02–4.21) | 3.80× (3.71–3.88) | 4.15× (4.06–4.25) | 3.94× (3.84–4.04) | 1.21× (1.18–1.24) | 1.43× (1.39–1.46) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 1.72× (1.64–1.81) | 2.63× (2.56–2.71) | 2.87× (2.79–2.94) | 3.19× (3.11–3.26) | 2.99× (2.91–3.08) | 2.70× (2.63–2.76) | 2.56× (2.50–2.62) | 0.79× (0.77–0.80) | 0.93× (0.90–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.28–0.30) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.95× (10.93–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.32–0.35) | 0.53× (0.48–0.58) | 0.69× (0.67–0.71) | 0.58× (0.56–0.61) | 5.50× (5.31–5.69) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.32–0.34) | 0.67× (0.60–0.76) | 0.59× (0.57–0.60) | 0.51× (0.49–0.53) | 3.42× (3.32–3.52) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.97× (7.93–8.00) | 0.64× (0.64–0.64) | 0.68× (0.68–0.68) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.73× (0.66–0.82) | 1.60× (1.57–1.64) | 0.81× (0.79–0.82) | 5.82× (5.70–5.94) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.67) | 0.73× (0.69–0.78) | 1.12× (1.08–1.17) | 0.70× (0.66–0.74) | 3.33× (3.18–3.49) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.17–1.17) | 0.72× (0.72–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.50) | 0.37× (0.34–0.41) | 0.89× (0.87–0.91) | 0.71× (0.68–0.73) | 3.07× (3.01–3.13) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.31× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.79–0.84) | 2.55× (2.50–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.33× (0.30–0.36) | 0.84× (0.81–0.87) | 0.69× (0.65–0.72) | 2.88× (2.78–2.99) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.48–0.51) | 0.81× (0.79–0.82) | 0.82× (0.79–0.85) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.06–1.07) | 0.76× (0.76–0.76) | 5.33× (5.32–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.27× (5.25–5.28) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.66–0.66) | 6.33× (6.32–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.45× (0.41–0.48) | 1.14× (1.12–1.17) | 0.74× (0.72–0.76) | 3.63× (3.55–3.70) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 54 MB | 0.52× (0.51–0.54) | 0.50× (0.46–0.55) | 0.95× (0.92–0.98) | 0.76× (0.72–0.80) | 2.41× (2.34–2.48) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 416 | 2.41 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 724 | 2.65 | 1.10× (1.08–1.11) | 85% | 1.02× (1.01–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 885 | 4.12 | 1.70× (1.69–1.72) | 53% | 1.19× (1.18–1.20) | 1.64× (1.62–1.66) |

**Best:** mibz2 at 4 threads (885 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.66 | 0.02 | 0.09 | 0.237 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3231 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.6% |
| MTF + RLE2 | 1.51 | 17.1% |
| T-vector build | 1.50 | 17.0% |
| walk (the one each build uses) | 4.16 | 47.0% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.85 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.299 | (in its total) | 2.63× (2.62–2.64) |  |
| walk | 1.261 | 2.670 | (in its total) | 2.11× (2.09–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.19× (1.16–1.21) |  |
| inverse BWT, total | 1.801 | 4.027 | 3.244 | 2.23× (2.21–2.24) | 1.82× (1.79–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m25s |
| 4 threads | 34m35s ± 4m51s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m00s ± 3m23s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 24m00s ± 3m23s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.15 ns a byte of output at 4 threads (CPU, summed over them), 1.97 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,640,714 | 13,421,331 | \`8da9d679141c517c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3231.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
