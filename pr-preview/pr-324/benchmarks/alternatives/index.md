# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-324/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`829ff7d`](<https://github.com/lmmx/mibz2/commit/829ff7d38b93849945044b7663529f50b35b2985>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42788764097>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 546 | 7.5 | 2.28× (2.17–2.40) | 5.72× (5.45–5.99) | 6.07× (5.68–6.48) | 6.76× (6.40–7.15) | 6.02× (5.70–6.37) | 6.17× (5.81–6.55) | 5.91× (5.56–6.29) | 1.78× (1.70–1.88) | 2.03× (1.96–2.11) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.228 s | 922 | 8.2 | 2.28× (2.12–2.46) | 5.26× (4.88–5.67) | 6.43× (5.93–6.97) | 6.28× (5.90–6.69) | 5.49× (5.14–5.86) | 10.30× (9.68–10.96) | 9.87× (9.35–10.43) | 2.98× (2.78–3.20) | 3.39× (3.13–3.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.210 s | 997 | 14.9 | 2.25× (2.11–2.40) | 3.33× (3.25–3.41) | 4.68× (4.35–5.03) | 4.01× (3.84–4.20) | 4.29× (4.07–4.52) | 11.22× (10.71–11.75) | 10.76× (10.40–11.12) | 3.25× (3.11–3.39) | 3.70× (3.49–3.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,365 | 3.3 | 0.56× (0.55–0.57) | 3.70× (3.25–4.21) | 4.56× (4.32–4.82) | 3.65× (3.56–3.73) | 6.72× (6.08–7.43) | 2.77× (2.71–2.82) | 2.82× (2.67–2.99) | 0.99× (0.96–1.03) | 2.09× (2.05–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,333 | 3.5 | 0.51× (0.50–0.52) | 3.02× (2.82–3.23) | 5.75× (5.53–5.98) | 3.63× (3.52–3.74) | 6.29× (6.15–6.42) | 2.73× (2.68–2.78) | 2.79× (2.68–2.89) | 0.98× (0.95–1.01) | 2.07× (2.01–2.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,307 | 3.4 | 0.50× (0.48–0.52) | 2.86× (2.60–3.14) | 5.71× (5.52–5.90) | 3.50× (3.38–3.64) | 6.18× (5.91–6.46) | 2.63× (2.55–2.72) | 2.69× (2.54–2.84) | 0.95× (0.90–0.99) | 1.99× (1.92–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 945 | 4.6 | 2.63× (2.51–2.76) | 3.33× (3.10–3.58) | 4.04× (3.90–4.18) | 4.22× (3.99–4.47) | 4.82× (4.60–5.05) | 2.77× (2.63–2.90) | 2.34× (2.23–2.46) | 1.99× (1.89–2.10) | 2.93× (2.78–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,202 | 6.6 | 2.01× (1.81–2.24) | 2.77× (2.40–3.20) | 3.53× (3.16–3.95) | 3.19× (2.96–3.45) | 4.78× (4.39–5.19) | 3.43× (3.12–3.78) | 2.91× (2.70–3.12) | 2.47× (2.32–2.63) | 3.64× (3.37–3.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,051 | 12.6 | 1.64× (1.56–1.72) | 1.85× (1.64–2.09) | 2.73× (2.49–2.99) | 2.00× (1.90–2.11) | 3.93× (3.78–4.09) | 3.17× (3.02–3.32) | 2.68× (2.57–2.80) | 2.28× (2.18–2.38) | 3.36× (3.23–3.49) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.085 s | 94 | 43.9 | 1.36× (1.23–1.49) | 2.93× (2.69–3.20) | 2.68× (2.49–2.90) | 2.59× (2.38–2.81) | 2.89× (2.65–3.15) | 3.39× (3.08–3.74) | 3.33× (3.07–3.62) | 1.29× (1.19–1.40) | 1.44× (1.32–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 147 | 48.9 | 1.26× (1.23–1.30) | 2.78× (2.63–2.93) | 2.57× (2.45–2.69) | 3.29× (3.09–3.50) | 2.98× (2.87–3.09) | 5.47× (5.11–5.85) | 5.37× (5.18–5.57) | 2.08× (2.00–2.16) | 2.33× (2.24–2.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 179 | 65.8 | 1.30× (1.26–1.33) | 2.38× (2.32–2.45) | 2.43× (2.27–2.60) | 2.95× (2.77–3.14) | 3.13× (3.05–3.21) | 6.70× (6.38–7.03) | 6.58× (6.41–6.76) | 2.54× (2.47–2.62) | 2.85× (2.76–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.090 s | 107 | 38.7 | 1.49× (1.41–1.58) | 3.81× (3.43–4.24) | 3.04× (2.92–3.16) | 3.00× (2.89–3.11) | 3.40× (3.28–3.53) | 4.35× (4.19–4.52) | 4.59× (4.45–4.74) | 1.58× (1.52–1.63) | 1.84× (1.75–1.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 170 | 44.9 | 1.43× (1.30–1.56) | 3.17× (3.05–3.31) | 2.73× (2.66–2.81) | 3.81× (3.69–3.94) | 3.28× (3.18–3.38) | 6.88× (6.71–7.06) | 7.26× (7.05–7.47) | 2.50× (2.42–2.58) | 2.91× (2.74–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.049 s | 194 | 62.4 | 1.38× (1.32–1.45) | 2.70× (2.52–2.89) | 2.56× (2.45–2.68) | 3.29× (3.18–3.41) | 3.22× (3.00–3.46) | 8.02× (7.73–8.32) | 8.46× (8.17–8.76) | 2.91× (2.80–3.03) | 3.39× (3.21–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,042 | 4.6 | 0.65× (0.58–0.73) | 3.15× (3.04–3.26) | 4.35× (4.20–4.51) | 3.16× (3.06–3.27) | 7.11× (6.91–7.31) | 2.32× (2.23–2.42) | 2.40× (2.35–2.46) | 0.97× (0.94–1.00) | 2.11× (2.04–2.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 976 | 5.0 | 0.47× (0.41–0.55) | 2.81× (2.43–3.25) | 5.22× (4.18–6.51) | 2.80× (2.44–3.23) | 6.23× (5.38–7.22) | 2.06× (1.77–2.40) | 2.14× (1.94–2.35) | 0.86× (0.75–1.00) | 1.87× (1.62–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 948 | 4.9 | 0.55× (0.44–0.67) | 2.94× (2.88–3.01) | 5.02× (4.73–5.32) | 2.97× (2.90–3.04) | 6.80× (6.18–7.48) | 2.15× (2.10–2.20) | 2.23× (2.11–2.35) | 0.90× (0.87–0.92) | 1.95× (1.91–2.00) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.059 s | 209 | 20.0 | 1.27× (1.23–1.30) | 3.08× (3.02–3.14) | 3.36× (3.23–3.49) | 3.41× (3.30–3.51) | 3.57× (3.47–3.67) | 3.76× (3.59–3.95) | 3.52× (3.39–3.66) | 1.13× (1.06–1.21) | 1.27× (1.22–1.33) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.042 s | 296 | 24.6 | 1.01× (0.90–1.13) | 2.45× (2.21–2.72) | 2.97× (2.67–3.31) | 3.42× (3.07–3.81) | 3.16× (2.88–3.48) | 5.25× (4.66–5.91) | 4.91× (4.51–5.36) | 1.58× (1.40–1.78) | 1.77× (1.59–1.98) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.034 s | 369 | 37.0 | 1.15× (1.11–1.18) | 2.22× (2.06–2.39) | 2.95× (2.64–3.29) | 3.21× (3.00–3.44) | 3.47× (3.33–3.62) | 6.67× (6.26–7.11) | 6.25× (5.95–6.57) | 2.01× (1.85–2.19) | 2.26× (2.14–2.38) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.28× (2.17–2.39) | 5.68× (5.42–5.95) | 6.03× (5.64–6.44) | 6.74× (6.37–7.12) | 6.00× (5.68–6.34) | 6.14× (5.79–6.52) | 5.89× (5.54–6.27) | 1.78× (1.70–1.87) | 2.01× (1.94–2.09) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.17× (2.08–2.27) | 5.38× (5.10–5.68) | 5.95× (5.58–6.35) | 6.67× (6.42–6.93) | 5.53× (5.32–5.74) | 5.54× (5.25–5.84) | 5.31× (5.11–5.51) | 1.61× (1.52–1.69) | 1.81× (1.70–1.93) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.10× (1.97–2.23) | 3.60× (3.44–3.76) | 4.19× (3.98–4.42) | 4.33× (4.08–4.60) | 4.08× (3.83–4.34) | 3.11× (2.89–3.35) | 2.98× (2.80–3.17) | 0.90× (0.84–0.97) | 1.02× (0.93–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.62) | 3.21× (2.79–3.68) | 3.93× (3.81–4.07) | 3.37× (3.29–3.45) | 6.12× (5.53–6.78) | 2.58× (2.53–2.63) | 2.63× (2.49–2.78) | 1.00× (0.96–1.03) | 1.82× (1.79–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.57) | 2.80× (2.75–2.84) | 4.94× (4.86–5.02) | 3.33× (3.25–3.42) | 5.65× (5.55–5.76) | 2.50× (2.46–2.55) | 2.56× (2.47–2.65) | 0.97× (0.94–1.00) | 1.76× (1.73–1.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.57) | 2.98× (2.89–3.08) | 4.99× (4.86–5.11) | 3.32× (3.21–3.43) | 5.71× (5.49–5.95) | 2.50× (2.43–2.56) | 2.55× (2.43–2.67) | 0.96× (0.92–1.01) | 1.76× (1.71–1.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.40–2.63) | 2.93× (2.75–3.12) | 3.58× (3.46–3.70) | 3.99× (3.78–4.21) | 4.54× (4.35–4.75) | 2.64× (2.51–2.78) | 2.25× (2.14–2.36) | 1.92× (1.83–2.02) | 2.57× (2.46–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.13× (1.92–2.37) | 2.36× (2.23–2.50) | 2.81× (2.68–2.95) | 3.07× (2.91–3.25) | 3.51× (3.31–3.71) | 1.90× (1.78–2.03) | 1.62× (1.54–1.70) | 1.39× (1.33–1.44) | 1.85× (1.79–1.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.76× (1.69–1.83) | 1.67× (1.43–1.95) | 1.89× (1.81–1.99) | 1.86× (1.78–1.95) | 2.03× (1.92–2.15) | 0.98× (0.91–1.05) | 0.83× (0.79–0.88) | 0.71× (0.68–0.75) | 0.95× (0.91–1.00) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.23–1.49) | 2.85× (2.61–3.10) | 2.60× (2.41–2.80) | 2.56× (2.35–2.78) | 2.86× (2.62–3.11) | 3.35× (3.04–3.68) | 3.29× (3.03–3.57) | 1.28× (1.18–1.39) | 1.39× (1.27–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.31) | 2.82× (2.66–2.98) | 2.40× (2.31–2.49) | 3.17× (3.01–3.34) | 2.68× (2.61–2.75) | 3.06× (2.90–3.24) | 3.01× (2.90–3.14) | 1.17× (1.14–1.21) | 1.27× (1.23–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.33–1.48) | 2.58× (2.50–2.67) | 2.17× (1.99–2.37) | 2.94× (2.72–3.18) | 2.71× (2.59–2.83) | 2.29× (2.19–2.38) | 2.25× (2.19–2.31) | 0.88× (0.85–0.90) | 0.95× (0.92–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.40–1.57) | 3.71× (3.34–4.11) | 2.93× (2.82–3.05) | 2.96× (2.85–3.07) | 3.36× (3.23–3.48) | 4.29× (4.13–4.45) | 4.52× (4.38–4.66) | 1.57× (1.51–1.62) | 1.76× (1.68–1.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.33–1.42) | 3.19× (3.09–3.29) | 2.52× (2.48–2.56) | 3.58× (3.55–3.62) | 2.92× (2.87–2.97) | 3.68× (3.61–3.75) | 3.88× (3.82–3.94) | 1.34× (1.31–1.38) | 1.51× (1.45–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.50–1.63) | 3.02× (2.80–3.27) | 2.25× (2.15–2.34) | 3.10× (2.98–3.21) | 2.95× (2.70–3.22) | 2.64× (2.56–2.71) | 2.78× (2.68–2.88) | 0.96× (0.93–1.00) | 1.08× (1.03–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.62–0.84) | 2.53× (2.43–2.65) | 3.33× (3.18–3.50) | 2.82× (2.70–2.94) | 6.15× (5.93–6.38) | 2.11× (2.01–2.21) | 2.18× (2.15–2.21) | 0.96× (0.93–1.00) | 1.48× (1.42–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.50–0.63) | 2.18× (1.93–2.47) | 4.07× (3.33–4.98) | 2.56× (2.26–2.90) | 5.41× (4.75–6.16) | 1.87× (1.63–2.14) | 1.93× (1.78–2.10) | 0.86× (0.75–0.97) | 1.31× (1.15–1.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.49–0.77) | 2.56× (2.38–2.76) | 4.11× (3.99–4.24) | 2.81× (2.75–2.87) | 6.12× (5.57–6.71) | 2.02× (1.99–2.06) | 2.09× (1.99–2.20) | 0.93× (0.90–0.95) | 1.42× (1.38–1.45) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.26× (1.23–1.29) | 2.94× (2.89–3.00) | 3.22× (3.10–3.35) | 3.34× (3.24–3.45) | 3.50× (3.40–3.60) | 3.69× (3.52–3.87) | 3.46× (3.33–3.59) | 1.14× (1.06–1.22) | 1.19× (1.16–1.22) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.14× (1.09–1.21) | 2.57× (2.51–2.63) | 2.94× (2.77–3.13) | 3.37× (3.09–3.68) | 2.99× (2.86–3.13) | 3.05× (2.84–3.27) | 2.86× (2.80–2.92) | 0.94× (0.86–1.02) | 0.98× (0.94–1.02) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.29× (1.24–1.34) | 2.28× (2.14–2.43) | 2.68× (2.40–3.00) | 2.86× (2.63–3.11) | 2.75× (2.57–2.94) | 2.03× (1.91–2.15) | 1.90× (1.81–1.99) | 0.62× (0.57–0.68) | 0.65× (0.63–0.68) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.38) | 1.07× (1.07–1.07) | 0.86× (0.85–0.88) | 12.09× (12.07–12.10) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.37× (0.37–0.38) | 0.49× (0.43–0.56) | 0.79× (0.77–0.80) | 0.65× (0.64–0.67) | 6.23× (6.12–6.34) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.58× (0.47–0.71) | 0.71× (0.69–0.72) | 0.61× (0.60–0.62) | 4.08× (4.06–4.11) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.21) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.83–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.45× (0.44–0.46) | 1.71× (1.70–1.71) | 0.66× (0.65–0.67) | 8.06× (8.04–8.09) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.62–0.68) | 0.65× (0.58–0.74) | 1.57× (1.52–1.63) | 0.78× (0.75–0.81) | 5.69× (5.48–5.91) | 0.46× (0.44–0.48) | 0.48× (0.46–0.50) | 0.52× (0.50–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.70× (0.63–0.77) | 1.15× (1.09–1.21) | 0.71× (0.69–0.74) | 3.35× (3.20–3.50) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.22× (5.21–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.40× (0.37–0.44) | 0.90× (0.86–0.95) | 0.72× (0.69–0.76) | 3.11× (2.96–3.27) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.46× (0.45–0.47) | 0.88× (0.85–0.90) | 0.82× (0.79–0.86) | 2.57× (2.52–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.22–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.49) | 0.38× (0.37–0.40) | 0.84× (0.80–0.87) | 0.70× (0.67–0.74) | 2.86× (2.75–2.98) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.45× (0.43–0.48) | 0.80× (0.79–0.81) | 0.82× (0.80–0.85) | 2.31× (2.28–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.08× (1.08–1.08) | 0.77× (0.77–0.77) | 5.41× (5.39–5.42) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.61× (1.61–1.62) | 0.77× (0.77–0.77) | 5.35× (5.33–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.62) | 0.77× (0.77–0.78) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.46–1.47) | 0.69× (0.69–0.70) | 6.65× (6.63–6.67) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.64–0.66) | 0.32× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.40× (0.38–0.43) | 1.03× (1.00–1.07) | 0.67× (0.65–0.69) | 3.28× (3.20–3.35) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.46× (0.42–0.50) | 0.93× (0.90–0.97) | 0.74× (0.71–0.79) | 2.35× (2.26–2.44) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 546 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 922 | 2.03 | 1.11× (1.03–1.20) | 83% | 1.05× (1.00–1.11) | 1.06× (1.01–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 997 | 3.67 | 1.98× (1.77–2.20) | 45% | 1.25× (1.19–1.32) | 1.82× (1.69–1.96) |

**Best:** mibz2 at 4 threads (997 MB/s); lbzip2 at 4 threads (306 MB/s); pbz2 at 4 threads (438 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.15, lbzip2 1t 0.08, lbzip2 2t 0.28, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.60 | 0.01 | 0.08 | 0.210 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4055 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 42.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.99 | 13.7% |
| MTF + RLE2 | 1.50 | 20.8% |
| T-vector build | 1.29 | 17.8% |
| walk (the one each build uses) | 3.06 | 42.2% |
| of which: segment assembly | 0.15 | 2.0% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.25 | 3.4% |
| **total** | 7.24 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.306 | 0.740 | (in its total) | 2.45× (2.38–2.51) |  |
| walk | 0.685 | 1.677 | (in its total) | 2.46× (2.39–2.53) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.032 | 0.037 | (in its total) | 1.20× (1.12–1.29) |  |
| inverse BWT, total | 1.022 | 2.460 | 1.381 | 2.42× (2.35–2.48) | 1.37× (1.32–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m56s ± 7m02s |
| 4 threads | 31m20s ± 4m42s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m22s ± 3m52s |
| the scanner alone (its serial part) | 10m02s ± 35.4s |
| the workers alone, over 4 | 22m22s ± 3m52s |

At 4 threads the workers bound it. Per byte: the scanner 5.8 ns a byte of bz2, the workers 2.93 ns a byte of output at 4 threads (CPU, summed over them), 1.54 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4055.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
