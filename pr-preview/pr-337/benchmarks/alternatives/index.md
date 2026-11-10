# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-337/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`8c40bde`](<https://github.com/lmmx/mibz2/commit/8c40bde6b22a1218762a1539b5f33253c5cd12c1>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42765250121>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.5.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.514 s | 408 | 9.2 | 2.36× (2.33–2.39) | 5.43× (5.37–5.48) | 5.59× (5.53–5.66) | 6.37× (6.30–6.45) | 5.59× (5.51–5.68) | 5.88× (5.82–5.95) | 5.53× (5.47–5.58) | 1.75× (1.73–1.76) | 1.95× (1.93–1.98) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.290 s | 724 | 9.9 | 2.47× (2.40–2.54) | 5.10× (4.91–5.31) | 5.82× (5.60–6.05) | 6.09× (5.86–6.33) | 5.31× (5.15–5.47) | 10.48× (10.14–10.82) | 9.84× (9.52–10.17) | 3.11× (3.01–3.21) | 3.48× (3.36–3.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.240 s | 872 | 16.2 | 2.37× (2.31–2.42) | 3.56× (3.44–3.69) | 4.36× (4.23–4.50) | 4.32× (4.23–4.41) | 4.18× (4.10–4.26) | 12.47× (12.19–12.76) | 11.72× (11.46–11.98) | 3.70× (3.62–3.79) | 4.14× (4.05–4.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,143 | 3.6 | 0.58× (0.57–0.59) | 3.47× (3.39–3.55) | 4.69× (4.54–4.84) | 3.61× (3.53–3.68) | 6.09× (5.97–6.21) | 2.70× (2.63–2.76) | 2.69× (2.63–2.75) | 0.97× (0.96–0.98) | 1.84× (1.67–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,136 | 3.7 | 0.54× (0.53–0.55) | 2.88× (2.81–2.95) | 5.77× (5.63–5.90) | 3.60× (3.51–3.68) | 5.99× (5.80–6.18) | 2.68× (2.61–2.74) | 2.67× (2.61–2.73) | 0.96× (0.95–0.98) | 1.83× (1.67–2.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,105 | 3.7 | 0.54× (0.53–0.54) | 2.84× (2.81–2.88) | 5.69× (5.62–5.75) | 3.55× (3.51–3.59) | 6.03× (5.91–6.15) | 2.64× (2.60–2.67) | 2.63× (2.60–2.66) | 0.95× (0.93–0.97) | 1.80× (1.64–1.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 714 | 5.6 | 2.60× (2.58–2.62) | 2.96× (2.94–2.98) | 3.67× (3.59–3.75) | 3.95× (3.93–3.98) | 4.35× (4.19–4.52) | 2.67× (2.66–2.69) | 2.19× (2.16–2.22) | 2.01× (1.99–2.02) | 2.95× (2.93–2.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,086 | 6.7 | 2.21× (2.12–2.31) | 2.83× (2.72–2.95) | 4.28× (4.03–4.56) | 3.60× (3.44–3.76) | 4.98× (4.75–5.22) | 4.07× (3.90–4.24) | 3.33× (3.19–3.47) | 3.05× (2.93–3.18) | 4.49× (4.31–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 947 | 13.4 | 1.78× (1.72–1.85) | 1.85× (1.68–2.03) | 2.89× (2.69–3.11) | 2.18× (2.11–2.25) | 3.92× (3.77–4.08) | 3.57× (3.45–3.70) | 2.92× (2.82–3.03) | 2.68× (2.59–2.78) | 3.94× (3.80–4.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 52.3 | 1.38× (1.36–1.39) | 2.62× (2.59–2.65) | 2.40× (2.38–2.41) | 2.51× (2.49–2.54) | 2.78× (2.76–2.80) | 2.99× (2.97–3.02) | 2.99× (2.97–3.01) | 1.25× (1.25–1.26) | 1.38× (1.37–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 58.7 | 1.34× (1.16–1.54) | 2.41× (2.31–2.51) | 2.32× (2.25–2.41) | 3.06× (2.92–3.21) | 2.81× (2.73–2.90) | 4.76× (4.61–4.92) | 4.75× (4.59–4.91) | 1.99× (1.93–2.04) | 2.20× (2.12–2.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 78.0 | 1.31× (1.29–1.34) | 2.14× (2.09–2.19) | 2.10× (2.03–2.17) | 2.72× (2.57–2.87) | 2.88× (2.82–2.94) | 5.93× (5.83–6.03) | 5.91× (5.82–6.01) | 2.48× (2.44–2.51) | 2.73× (2.69–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 46.6 | 1.51× (1.47–1.55) | 3.02× (2.93–3.11) | 2.64× (2.57–2.71) | 2.77× (2.70–2.85) | 3.07× (2.99–3.15) | 3.68× (3.58–3.79) | 3.87× (3.76–3.98) | 1.49× (1.45–1.53) | 1.79× (1.74–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 134 | 53.0 | 1.39× (1.35–1.43) | 2.77× (2.68–2.87) | 2.58× (2.51–2.65) | 3.58× (3.50–3.65) | 3.10× (3.03–3.18) | 6.10× (5.97–6.23) | 6.41× (6.27–6.55) | 2.47× (2.41–2.53) | 2.96× (2.89–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 153 | 72.3 | 1.42× (1.39–1.45) | 2.25× (2.21–2.30) | 2.23× (2.17–2.29) | 3.08× (3.01–3.14) | 2.84× (2.79–2.89) | 7.01× (6.89–7.14) | 7.36× (7.22–7.51) | 2.84× (2.78–2.89) | 3.40× (3.34–3.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 848 | 5.1 | 0.63× (0.62–0.64) | 3.47× (3.44–3.51) | 4.37× (4.33–4.41) | 3.09× (3.06–3.12) | 6.69× (6.56–6.83) | 2.29× (2.26–2.31) | 2.31× (2.29–2.33) | 0.96× (0.94–0.97) | 1.75× (1.73–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 820 | 5.3 | 0.53× (0.51–0.54) | 2.49× (2.43–2.56) | 5.00× (4.88–5.13) | 2.96× (2.88–3.05) | 6.30× (6.15–6.45) | 2.17× (2.12–2.23) | 2.20× (2.14–2.26) | 0.91× (0.88–0.93) | 1.66× (1.62–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 803 | 5.3 | 0.53× (0.52–0.53) | 2.50× (2.47–2.53) | 5.01× (4.95–5.07) | 2.98× (2.94–3.02) | 6.37× (6.20–6.55) | 2.18× (2.15–2.20) | 2.20× (2.18–2.23) | 0.91× (0.89–0.93) | 1.66× (1.64–1.68) | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 1 | 0.071 s | 169 | 22.7 | 1.35× (1.34–1.36) | 3.00× (2.98–3.02) | 3.28× (3.25–3.31) | 3.40× (3.37–3.42) | 3.58× (3.54–3.61) | 3.52× (3.50–3.55) | 3.43× (3.40–3.46) | 1.12× (1.10–1.14) | 1.28× (1.27–1.29) | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 2 | 0.045 s | 266 | 26.4 | 1.16× (1.10–1.22) | 2.61× (2.38–2.87) | 3.18× (3.02–3.36) | 3.56× (3.43–3.70) | 3.44× (3.24–3.66) | 5.41× (5.19–5.64) | 5.27× (5.05–5.50) | 1.72× (1.64–1.80) | 1.97× (1.89–2.06) | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 4 | 0.037 s | 324 | 37.9 | 1.30× (1.26–1.33) | 2.02× (1.93–2.12) | 2.86× (2.77–2.96) | 3.08× (3.01–3.16) | 3.30× (3.21–3.39) | 6.73× (6.56–6.92) | 6.55× (6.38–6.73) | 2.14× (2.10–2.19) | 2.45× (2.39–2.52) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.35× (2.32–2.38) | 5.40× (5.35–5.46) | 5.57× (5.51–5.63) | 6.35× (6.28–6.43) | 5.58× (5.50–5.66) | 5.87× (5.80–5.93) | 5.51× (5.45–5.57) | 1.74× (1.73–1.76) | 1.94× (1.91–1.97) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.27× (2.23–2.30) | 5.12× (5.04–5.20) | 5.33× (5.24–5.42) | 6.23× (6.13–6.33) | 5.22× (5.14–5.31) | 5.47× (5.38–5.57) | 5.14× (5.06–5.23) | 1.63× (1.60–1.65) | 1.81× (1.78–1.84) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.17× (2.11–2.23) | 3.67× (3.56–3.77) | 4.00× (3.87–4.15) | 4.51× (4.39–4.64) | 3.84× (3.74–3.94) | 3.34× (3.24–3.44) | 3.14× (3.05–3.23) | 0.99× (0.97–1.02) | 1.10× (1.07–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.96× (2.91–3.00) | 3.97× (3.87–4.08) | 3.38× (3.32–3.45) | 5.65× (5.55–5.75) | 2.55× (2.49–2.60) | 2.54× (2.49–2.60) | 0.98× (0.96–0.99) | 1.66× (1.62–1.69) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.77× (2.71–2.83) | 4.98× (4.88–5.09) | 3.35× (3.27–3.42) | 5.48× (5.32–5.65) | 2.49× (2.43–2.55) | 2.48× (2.43–2.54) | 0.95× (0.94–0.96) | 1.62× (1.58–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.58) | 3.06× (3.02–3.11) | 5.02× (4.95–5.10) | 3.38× (3.34–3.42) | 5.63× (5.51–5.74) | 2.51× (2.48–2.55) | 2.51× (2.48–2.54) | 0.96× (0.94–0.98) | 1.64× (1.62–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.49–2.53) | 2.73× (2.72–2.75) | 3.24× (3.21–3.26) | 3.78× (3.76–3.81) | 4.16× (4.01–4.31) | 2.58× (2.56–2.60) | 2.12× (2.10–2.15) | 1.95× (1.94–1.96) | 2.54× (2.52–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.23× (2.19–2.28) | 2.47× (2.41–2.54) | 3.29× (3.20–3.38) | 3.29× (3.22–3.36) | 3.54× (3.47–3.61) | 2.13× (2.08–2.19) | 1.76× (1.72–1.79) | 1.61× (1.58–1.65) | 2.10× (2.05–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.90–2.04) | 1.70× (1.62–1.77) | 2.17× (2.08–2.25) | 2.01× (1.95–2.09) | 2.06× (1.98–2.15) | 1.08× (1.04–1.12) | 0.89× (0.85–0.92) | 0.82× (0.79–0.85) | 1.06× (1.03–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.57× (2.55–2.59) | 2.34× (2.32–2.36) | 2.49× (2.46–2.52) | 2.75× (2.73–2.77) | 2.96× (2.94–2.98) | 2.96× (2.94–2.98) | 1.25× (1.24–1.25) | 1.36× (1.36–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.36× (1.19–1.54) | 2.43× (2.41–2.45) | 2.15× (2.14–2.17) | 2.94× (2.91–2.96) | 2.53× (2.51–2.54) | 2.65× (2.63–2.67) | 2.64× (2.62–2.67) | 1.11× (1.10–1.13) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.44–1.49) | 2.26× (2.22–2.29) | 1.92× (1.89–1.94) | 2.60× (2.57–2.63) | 2.46× (2.43–2.49) | 1.99× (1.97–2.01) | 1.98× (1.96–2.01) | 0.84× (0.83–0.84) | 0.92× (0.91–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.47–1.55) | 2.96× (2.88–3.04) | 2.57× (2.50–2.64) | 2.75× (2.67–2.82) | 3.04× (2.96–3.12) | 3.64× (3.55–3.74) | 3.83× (3.72–3.93) | 1.48× (1.44–1.53) | 1.71× (1.67–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.39–1.42) | 2.78× (2.76–2.81) | 2.38× (2.35–2.40) | 3.36× (3.32–3.40) | 2.79× (2.75–2.82) | 3.25× (3.23–3.28) | 3.42× (3.39–3.45) | 1.33× (1.31–1.34) | 1.53× (1.52–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.62–1.69) | 2.59× (2.54–2.64) | 2.06× (2.03–2.10) | 2.96× (2.92–3.00) | 2.67× (2.63–2.72) | 2.38× (2.35–2.42) | 2.50× (2.47–2.54) | 0.97× (0.95–0.99) | 1.12× (1.11–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.46× (2.43–2.50) | 3.38× (3.34–3.41) | 2.83× (2.81–2.86) | 5.99× (5.88–6.10) | 2.13× (2.11–2.14) | 2.15× (2.14–2.17) | 0.96× (0.96–0.97) | 1.37× (1.36–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.61) | 2.25× (2.20–2.31) | 3.97× (3.88–4.06) | 2.73× (2.66–2.80) | 5.56× (5.44–5.69) | 1.99× (1.94–2.03) | 2.01× (1.97–2.06) | 0.90× (0.88–0.92) | 1.28× (1.25–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.58) | 2.54× (2.49–2.59) | 4.05× (4.00–4.10) | 2.83× (2.79–2.86) | 5.80× (5.65–5.95) | 2.05× (2.02–2.08) | 2.08× (2.06–2.10) | 0.93× (0.91–0.95) | 1.32× (1.31–1.34) |
| `text` (2.39 MB → 12 MB, 5.0×) | 1 | 1.34× (1.33–1.35) | 2.84× (2.82–2.87) | 3.11× (3.08–3.13) | 3.34× (3.32–3.37) | 3.52× (3.48–3.55) | 3.47× (3.44–3.49) | 3.37× (3.35–3.40) | 1.12× (1.10–1.13) | 1.22× (1.21–1.23) |
| `text` (2.39 MB → 12 MB, 5.0×) | 2 | 1.23× (1.22–1.25) | 2.61× (2.55–2.67) | 2.83× (2.79–2.86) | 3.32× (3.28–3.35) | 3.13× (3.06–3.19) | 2.99× (2.95–3.02) | 2.91× (2.87–2.94) | 0.96× (0.95–0.98) | 1.05× (1.04–1.06) |
| `text` (2.39 MB → 12 MB, 5.0×) | 4 | 1.39× (1.33–1.45) | 2.17× (2.10–2.24) | 2.42× (2.36–2.50) | 2.77× (2.71–2.84) | 2.69× (2.62–2.75) | 2.09× (2.04–2.14) | 2.04× (1.99–2.08) | 0.68× (0.66–0.69) | 0.73× (0.72–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.37× (0.36–0.38) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.10× (12.09–12.12) | 0.29× (0.28–0.29) | 0.31× (0.31–0.31) | 0.47× (0.46–0.47) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.57× (0.50–0.64) | 0.79× (0.77–0.81) | 0.66× (0.65–0.67) | 6.32× (6.25–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.57× (0.48–0.66) | 0.70× (0.68–0.71) | 0.62× (0.60–0.64) | 4.13× (4.06–4.20) | 0.10× (0.09–0.10) | 0.10× (0.10–0.11) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.71) | 0.66× (0.66–0.67) | 8.06× (8.05–8.08) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.69) | 0.70× (0.64–0.76) | 1.60× (1.54–1.65) | 0.79× (0.77–0.82) | 5.79× (5.60–5.98) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.53× (0.51–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.72× (0.67–0.76) | 1.17× (1.14–1.21) | 0.71× (0.68–0.74) | 3.44× (3.36–3.53) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.25) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.47–0.53) | 0.42× (0.40–0.44) | 0.90× (0.86–0.94) | 0.72× (0.68–0.75) | 3.12× (2.98–3.27) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.61–0.64) | 0.46× (0.45–0.46) | 0.87× (0.85–0.88) | 0.79× (0.75–0.84) | 2.52× (2.48–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.84× (0.83–0.84) | 5.24× (5.22–5.25) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.39× (0.36–0.41) | 0.86× (0.82–0.89) | 0.71× (0.68–0.74) | 2.94× (2.83–3.05) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.48× (0.46–0.51) | 0.82× (0.81–0.83) | 0.83× (0.80–0.86) | 2.37× (2.34–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.23–0.25) | 1.08× (1.08–1.09) | 0.77× (0.77–0.78) | 5.42× (5.41–5.44) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.21) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.39 MB → 12 MB, 5.0×) | 1 | 18 MB | 0.48× (0.48–0.48) | 0.40× (0.40–0.41) | 1.49× (1.48–1.49) | 0.69× (0.68–0.69) | 6.71× (6.69–6.72) | 0.41× (0.41–0.41) | 0.44× (0.44–0.44) | 0.66× (0.66–0.66) | 0.33× (0.32–0.33) |
| `text` (2.39 MB → 12 MB, 5.0×) | 2 | 34 MB | 0.48× (0.47–0.50) | 0.43× (0.40–0.47) | 1.13× (1.09–1.17) | 0.73× (0.71–0.75) | 3.60× (3.51–3.69) | 0.22× (0.21–0.23) | 0.24× (0.23–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (2.39 MB → 12 MB, 5.0×) | 4 | 52 MB | 0.54× (0.52–0.56) | 0.45× (0.41–0.50) | 0.97× (0.95–0.99) | 0.77× (0.74–0.80) | 2.49× (2.43–2.55) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 408 | 2.46 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 724 | 2.63 | 1.07× (1.05–1.09) | 89% | 1.02× (1.01–1.02) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 872 | 4.32 | 1.76× (1.70–1.81) | 53% | 1.19× (1.17–1.21) | 1.62× (1.57–1.67) |

**Best:** mibz2 at 4 threads (872 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (368 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.09, lbzip2 1t 0.06, lbzip2 2t 0.31, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.71 | 0.02 | 0.09 | 0.240 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3755 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 12.7% |
| MTF + RLE2 | 1.61 | 18.1% |
| T-vector build | 1.51 | 17.0% |
| walk (the one each build uses) | 4.18 | 46.9% |
| of which: segment assembly | 0.17 | 1.9% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.9% |
| **total** | 8.90 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.060 | (in its total) | 2.71× (2.70–2.71) |  |
| walk | 1.049 | 2.059 | (in its total) | 1.96× (1.95–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.041 | (in its total) | 1.05× (1.01–1.09) |  |
| inverse BWT, total | 1.480 | 3.161 | 1.893 | 2.13× (2.12–2.15) | 1.29× (1.27–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m51s |
| 4 threads | 35m25s ± 5m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m14s ± 3m48s |
| the scanner alone (its serial part) | 12m13s ± 32.2s |
| the workers alone, over 4 | 26m14s ± 3m48s |

At 4 threads the workers bound it. Per byte: the scanner 7.1 ns a byte of bz2, the workers 3.44 ns a byte of output at 4 threads (CPU, summed over them), 2.06 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,388,533 | 12,031,475 | \`c2da850f07011c2a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3755.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.5.0 (bench/report.py), 10 rounds, threads 1, 2, 4
