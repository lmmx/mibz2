# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-325/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c4465d9`](<https://github.com/lmmx/mibz2/commit/c4465d91aa451fad0b3d4c0cdbb1ee4d1fcee05b>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42747017714>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.513 s | 409 | 9.1 | 2.36× (2.35–2.38) | 5.42× (5.40–5.45) | 5.58× (5.56–5.61) | 6.38× (6.36–6.40) | 5.60× (5.58–5.61) | 5.88× (5.86–5.90) | 5.53× (5.52–5.54) | 1.76× (1.75–1.77) | 1.96× (1.95–1.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.299 s | 701 | 9.7 | 2.37× (2.26–2.48) | 4.93× (4.72–5.16) | 5.55× (5.33–5.78) | 5.86× (5.64–6.08) | 5.14× (4.92–5.37) | 10.13× (9.72–10.55) | 9.53× (9.15–9.92) | 3.03× (2.91–3.16) | 3.38× (3.24–3.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.237 s | 883 | 15.7 | 2.36× (2.33–2.38) | 3.53× (3.50–3.57) | 4.31× (4.24–4.37) | 4.34× (4.27–4.42) | 4.23× (4.19–4.27) | 12.67× (12.54–12.80) | 11.92× (11.82–12.02) | 3.79× (3.75–3.83) | 4.23× (4.20–4.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,149 | 3.5 | 0.58× (0.57–0.59) | 3.53× (3.48–3.58) | 4.71× (4.65–4.78) | 3.65× (3.60–3.70) | 6.29× (6.18–6.40) | 2.74× (2.70–2.77) | 2.74× (2.70–2.78) | 0.99× (0.96–1.01) | 1.82× (1.70–1.94) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.51× (0.49–0.55) | 2.77× (2.60–2.95) | 5.55× (5.22–5.91) | 3.45× (3.23–3.68) | 5.84× (5.46–6.24) | 2.57× (2.42–2.73) | 2.58× (2.42–2.74) | 0.93× (0.87–0.99) | 1.71× (1.56–1.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,117 | 3.6 | 0.54× (0.53–0.54) | 2.86× (2.82–2.90) | 5.76× (5.62–5.91) | 3.55× (3.51–3.60) | 5.99× (5.91–6.06) | 2.64× (2.60–2.68) | 2.65× (2.61–2.69) | 0.95× (0.93–0.98) | 1.76× (1.64–1.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 719 | 5.5 | 2.65× (2.54–2.77) | 2.99× (2.98–3.01) | 3.78× (3.59–3.97) | 3.99× (3.97–4.01) | 4.32× (4.27–4.38) | 2.70× (2.68–2.72) | 2.21× (2.19–2.22) | 2.02× (2.01–2.03) | 2.98× (2.97–2.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,150 | 6.5 | 2.39× (2.22–2.57) | 3.07× (2.84–3.31) | 4.36× (4.07–4.66) | 3.70× (3.48–3.94) | 5.28× (4.99–5.58) | 4.24× (4.08–4.42) | 3.47× (3.34–3.60) | 3.18× (3.06–3.31) | 4.69× (4.50–4.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 963 | 12.8 | 1.85× (1.77–1.93) | 1.88× (1.72–2.06) | 2.95× (2.75–3.17) | 2.27× (2.16–2.39) | 4.08× (3.94–4.22) | 3.64× (3.54–3.75) | 2.98× (2.90–3.06) | 2.73× (2.65–2.81) | 4.02× (3.92–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.6 | 1.38× (1.37–1.38) | 2.65× (2.61–2.68) | 2.40× (2.39–2.40) | 2.52× (2.50–2.54) | 2.79× (2.77–2.80) | 2.99× (2.98–3.00) | 2.99× (2.98–3.00) | 1.25× (1.24–1.27) | 1.38× (1.38–1.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 121 | 57.2 | 1.30× (1.23–1.37) | 2.46× (2.33–2.59) | 2.30× (2.18–2.43) | 3.08× (2.97–3.20) | 2.86× (2.72–3.00) | 4.82× (4.60–5.05) | 4.83× (4.61–5.05) | 2.02× (1.92–2.13) | 2.23× (2.13–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 76.2 | 1.36× (1.33–1.40) | 2.14× (2.07–2.21) | 2.11× (2.06–2.17) | 2.67× (2.62–2.72) | 2.91× (2.86–2.96) | 6.01× (5.92–6.11) | 6.02× (5.92–6.12) | 2.52× (2.48–2.57) | 2.78× (2.74–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 45.9 | 1.53× (1.53–1.54) | 3.08× (3.05–3.12) | 2.68× (2.67–2.69) | 2.81× (2.78–2.83) | 3.12× (3.10–3.14) | 3.74× (3.70–3.78) | 3.94× (3.89–3.98) | 1.51× (1.50–1.52) | 1.81× (1.80–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 s | 135 | 51.9 | 1.43× (1.33–1.53) | 2.84× (2.66–3.02) | 2.56× (2.44–2.67) | 3.55× (3.40–3.70) | 3.09× (2.97–3.20) | 6.06× (5.82–6.31) | 6.38× (6.11–6.66) | 2.45× (2.35–2.55) | 2.93× (2.81–3.05) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 70.4 | 1.43× (1.41–1.46) | 2.23× (2.16–2.31) | 2.23× (2.16–2.31) | 3.10× (3.01–3.20) | 2.86× (2.80–2.91) | 7.07× (6.92–7.22) | 7.44× (7.30–7.58) | 2.86× (2.81–2.90) | 3.42× (3.36–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 4.9 | 0.63× (0.63–0.64) | 3.54× (3.51–3.57) | 4.54× (4.36–4.72) | 3.14× (3.12–3.16) | 6.78× (6.67–6.89) | 2.33× (2.31–2.35) | 2.35× (2.34–2.37) | 0.97× (0.95–0.99) | 1.78× (1.76–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 833 | 5.2 | 0.54× (0.53–0.55) | 2.57× (2.54–2.61) | 5.17× (5.10–5.24) | 3.06× (3.02–3.10) | 6.54× (6.37–6.71) | 2.25× (2.22–2.28) | 2.27× (2.24–2.30) | 0.93× (0.91–0.95) | 1.72× (1.69–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 813 | 5.1 | 0.53× (0.52–0.54) | 2.52× (2.49–2.54) | 5.06× (5.01–5.11) | 2.99× (2.97–3.02) | 6.44× (6.29–6.60) | 2.20× (2.18–2.22) | 2.22× (2.20–2.24) | 0.91× (0.89–0.93) | 1.67× (1.66–1.69) | yes |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 1 | 0.074 s | 171 | 22.1 | 1.35× (1.32–1.39) | 2.89× (2.88–2.91) | 3.17× (3.15–3.18) | 3.39× (3.38–3.41) | 3.56× (3.52–3.59) | 3.46× (3.44–3.48) | 3.38× (3.36–3.40) | 1.11× (1.10–1.12) | 1.24× (1.23–1.24) | yes |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 2 | 0.046 s | 272 | 25.3 | 1.13× (1.06–1.21) | 2.62× (2.50–2.74) | 3.01× (2.84–3.18) | 3.42× (3.24–3.62) | 3.40× (3.21–3.59) | 5.34× (5.05–5.65) | 5.21× (4.92–5.52) | 1.71× (1.62–1.81) | 1.91× (1.80–2.02) | yes |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 4 | 0.037 s | 337 | 36.7 | 1.29× (1.25–1.33) | 2.00× (1.92–2.08) | 2.74× (2.67–2.82) | 3.11× (2.99–3.25) | 3.44× (3.38–3.50) | 6.78× (6.68–6.89) | 6.62× (6.53–6.71) | 2.18× (2.15–2.21) | 2.43× (2.39–2.46) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.36× (2.35–2.37) | 5.40× (5.38–5.42) | 5.56× (5.54–5.59) | 6.36× (6.34–6.38) | 5.58× (5.56–5.60) | 5.86× (5.84–5.89) | 5.51× (5.50–5.53) | 1.76× (1.75–1.76) | 1.95× (1.94–1.95) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.27× (2.26–2.28) | 5.13× (5.10–5.17) | 5.33× (5.32–5.34) | 6.25× (6.23–6.27) | 5.25× (5.23–5.27) | 5.49× (5.47–5.51) | 5.16× (5.15–5.18) | 1.64× (1.64–1.65) | 1.82× (1.81–1.83) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.19× (2.17–2.21) | 3.70× (3.65–3.75) | 3.93× (3.89–3.97) | 4.53× (4.47–4.58) | 3.88× (3.84–3.92) | 3.40× (3.37–3.43) | 3.20× (3.16–3.23) | 1.02× (1.01–1.03) | 1.13× (1.11–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.00× (2.95–3.06) | 3.99× (3.90–4.10) | 3.41× (3.37–3.45) | 5.82× (5.72–5.92) | 2.58× (2.55–2.61) | 2.58× (2.55–2.61) | 0.99× (0.97–1.01) | 1.68× (1.65–1.70) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.53–0.60) | 2.68× (2.53–2.84) | 4.79× (4.50–5.09) | 3.22× (3.03–3.42) | 5.37× (5.03–5.72) | 2.40× (2.27–2.54) | 2.40× (2.27–2.55) | 0.92× (0.87–0.98) | 1.56× (1.47–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.58) | 3.06× (3.03–3.10) | 5.03× (4.87–5.21) | 3.38× (3.34–3.42) | 5.59× (5.52–5.65) | 2.52× (2.49–2.55) | 2.52× (2.49–2.55) | 0.96× (0.94–0.99) | 1.64× (1.61–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.56× (2.45–2.67) | 2.76× (2.75–2.78) | 3.34× (3.15–3.53) | 3.81× (3.80–3.83) | 4.13× (4.08–4.18) | 2.60× (2.58–2.62) | 2.13× (2.12–2.14) | 1.96× (1.95–1.97) | 2.56× (2.55–2.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.33× (2.22–2.45) | 2.52× (2.46–2.58) | 3.31× (3.25–3.37) | 3.35× (3.29–3.42) | 3.60× (3.54–3.66) | 2.18× (2.14–2.21) | 1.79× (1.75–1.82) | 1.64× (1.62–1.67) | 2.14× (2.11–2.18) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.95–2.04) | 1.74× (1.70–1.78) | 2.22× (2.18–2.27) | 2.02× (1.96–2.09) | 2.14× (2.09–2.18) | 1.10× (1.08–1.12) | 0.91× (0.89–0.92) | 0.83× (0.82–0.85) | 1.09× (1.06–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.37) | 2.57× (2.56–2.58) | 2.34× (2.33–2.35) | 2.50× (2.48–2.52) | 2.76× (2.74–2.77) | 2.96× (2.95–2.97) | 2.96× (2.95–2.97) | 1.25× (1.23–1.27) | 1.36× (1.36–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.33× (1.30–1.36) | 2.45× (2.43–2.46) | 2.17× (2.16–2.18) | 2.95× (2.93–2.96) | 2.55× (2.54–2.56) | 2.66× (2.65–2.68) | 2.67× (2.65–2.68) | 1.13× (1.11–1.14) | 1.23× (1.22–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.48–1.52) | 2.28× (2.25–2.30) | 1.92× (1.89–1.94) | 2.65× (2.61–2.68) | 2.47× (2.44–2.50) | 2.01× (1.99–2.02) | 2.01× (1.99–2.03) | 0.85× (0.83–0.86) | 0.93× (0.92–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.52–1.53) | 3.00× (2.99–3.01) | 2.61× (2.59–2.63) | 2.78× (2.76–2.81) | 3.09× (3.07–3.11) | 3.70× (3.66–3.74) | 3.89× (3.85–3.94) | 1.50× (1.50–1.51) | 1.74× (1.74–1.74) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.39–1.52) | 2.80× (2.78–2.83) | 2.39× (2.37–2.41) | 3.37× (3.33–3.42) | 2.81× (2.78–2.83) | 3.28× (3.24–3.32) | 3.45× (3.41–3.50) | 1.33× (1.32–1.34) | 1.54× (1.53–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.61–1.73) | 2.59× (2.52–2.67) | 2.06× (2.00–2.12) | 2.97× (2.87–3.06) | 2.68× (2.61–2.76) | 2.40× (2.32–2.47) | 2.52× (2.45–2.60) | 0.97× (0.94–1.00) | 1.13× (1.09–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.68–0.68) | 2.52× (2.51–2.53) | 3.50× (3.33–3.68) | 2.87× (2.86–2.88) | 6.06× (5.96–6.16) | 2.17× (2.15–2.18) | 2.19× (2.18–2.20) | 0.98× (0.96–0.99) | 1.39× (1.38–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.61) | 2.34× (2.30–2.37) | 4.08× (4.02–4.13) | 2.81× (2.78–2.84) | 5.76× (5.62–5.90) | 2.05× (2.03–2.07) | 2.07× (2.05–2.09) | 0.93× (0.91–0.94) | 1.31× (1.30–1.33) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.58–0.59) | 2.54× (2.51–2.57) | 4.08× (4.03–4.13) | 2.84× (2.82–2.86) | 5.87× (5.73–6.00) | 2.07× (2.05–2.09) | 2.09× (2.07–2.11) | 0.94× (0.92–0.95) | 1.32× (1.31–1.34) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 1 | 1.35× (1.31–1.38) | 2.81× (2.80–2.83) | 3.10× (3.09–3.12) | 3.34× (3.33–3.35) | 3.50× (3.46–3.53) | 3.40× (3.39–3.42) | 3.32× (3.31–3.34) | 1.11× (1.10–1.11) | 1.21× (1.20–1.21) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 2 | 1.23× (1.22–1.23) | 2.58× (2.56–2.60) | 2.84× (2.82–2.87) | 3.34× (3.32–3.35) | 3.10× (3.09–3.12) | 2.98× (2.96–2.99) | 2.91× (2.89–2.92) | 0.97× (0.96–0.97) | 1.06× (1.05–1.06) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 4 | 1.39× (1.36–1.43) | 2.16× (2.13–2.20) | 2.40× (2.37–2.43) | 2.73× (2.69–2.78) | 2.65× (2.63–2.67) | 2.06× (2.04–2.07) | 2.01× (2.00–2.02) | 0.67× (0.67–0.67) | 0.73× (0.73–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.35× (0.34–0.37) | 1.07× (1.07–1.07) | 0.86× (0.86–0.87) | 12.11× (12.08–12.15) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 50 MB | 0.37× (0.36–0.38) | 0.53× (0.45–0.62) | 0.77× (0.75–0.78) | 0.65× (0.63–0.67) | 6.16× (6.01–6.31) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.23× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.63× (0.53–0.74) | 0.69× (0.68–0.70) | 0.61× (0.60–0.62) | 4.08× (4.05–4.11) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.30–4.31) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.72) | 0.67× (0.67–0.67) | 8.08× (8.05–8.10) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.65× (0.59–0.71) | 1.63× (1.59–1.67) | 0.81× (0.79–0.83) | 5.92× (5.78–6.07) | 0.48× (0.46–0.49) | 0.50× (0.49–0.52) | 0.54× (0.53–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.67) | 0.71× (0.66–0.76) | 1.15× (1.10–1.20) | 0.70× (0.67–0.74) | 3.35× (3.21–3.50) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.75) | 5.23× (5.22–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.51) | 0.40× (0.38–0.43) | 0.90× (0.87–0.93) | 0.72× (0.69–0.74) | 3.11× (3.01–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.61–0.66) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.82× (0.80–0.84) | 2.51× (2.44–2.59) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.24× (5.23–5.25) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.40) | 0.84× (0.81–0.87) | 0.70× (0.66–0.73) | 2.89× (2.80–2.98) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.28× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.51× (0.49–0.54) | 0.82× (0.81–0.84) | 0.82× (0.80–0.83) | 2.38× (2.34–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.25) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.42× (5.40–5.43) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.61× (1.61–1.62) | 0.77× (0.77–0.77) | 5.35× (5.33–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 1 | 18 MB | 0.48× (0.48–0.48) | 0.40× (0.40–0.40) | 1.49× (1.48–1.49) | 0.69× (0.68–0.69) | 6.72× (6.70–6.73) | 0.40× (0.40–0.41) | 0.44× (0.44–0.45) | 0.66× (0.66–0.66) | 0.32× (0.32–0.33) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 2 | 32 MB | 0.51× (0.50–0.53) | 0.46× (0.43–0.50) | 1.23× (1.19–1.27) | 0.78× (0.75–0.81) | 3.82× (3.69–3.96) | 0.23× (0.22–0.24) | 0.25× (0.24–0.26) | 0.38× (0.36–0.39) | 0.19× (0.18–0.19) |
| `text` (2.36 MB → 12.6 MB, 5.3×) | 4 | 52 MB | 0.53× (0.51–0.55) | 0.49× (0.44–0.54) | 0.97× (0.95–1.00) | 0.77× (0.74–0.81) | 2.47× (2.42–2.52) | 0.14× (0.14–0.14) | 0.16× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 409 | 2.46 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 701 | 2.63 | 1.07× (1.07–1.07) | 86% | 1.02× (1.01–1.02) | 1.03× (1.02–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 883 | 4.24 | 1.72× (1.71–1.74) | 54% | 1.18× (1.18–1.19) | 1.60× (1.58–1.62) |

**Best:** mibz2 at 4 threads (883 MB/s); lbzip2 at 4 threads (250 MB/s); pbz2 at 4 threads (375 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.14, lbzip2 1t 0.06, lbzip2 2t 0.27, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.70 | 0.02 | 0.08 | 0.237 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.8% |
| MTF + RLE2 | 1.58 | 18.2% |
| T-vector build | 1.49 | 17.1% |
| walk (the one each build uses) | 4.05 | 46.6% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.9% |
| **total** | 8.68 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.062 | (in its total) | 2.70× (2.65–2.75) |  |
| walk | 1.052 | 2.082 | (in its total) | 1.97× (1.94–2.01) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.041 | (in its total) | 1.03× (1.00–1.06) |  |
| inverse BWT, total | 1.483 | 3.183 | 1.898 | 2.14× (2.10–2.18) | 1.27× (1.25–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 8m43s |
| 4 threads | 34m41s ± 4m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m36s ± 3m36s |
| the scanner alone (its serial part) | 12m31s ± 32.2s |
| the workers alone, over 4 | 25m36s ± 3m36s |

At 4 threads the workers bound it. Per byte: the scanner 7.3 ns a byte of bz2, the workers 3.36 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,364,635 | 12,602,243 | \`75bfc99e586bd1ab…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
