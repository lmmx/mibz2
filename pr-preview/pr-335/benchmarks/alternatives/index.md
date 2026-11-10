# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-335/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c47fbe5`](<https://github.com/lmmx/mibz2/commit/c47fbe50cf99fbff930526356b9723bfc453ddb9>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42753660237>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.609 s | 344 | 8.6 | 2.49× (2.47–2.52) | 5.63× (5.56–5.70) | 5.78× (5.72–5.85) | 6.30× (6.22–6.38) | 5.67× (5.60–5.74) | 6.19× (6.12–6.26) | 5.69× (5.62–5.76) | 1.83× (1.82–1.85) | 2.11× (2.08–2.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.352 s | 596 | 9.4 | 2.46× (2.31–2.61) | 5.24× (5.04–5.44) | 5.95× (5.66–6.25) | 5.84× (5.55–6.13) | 5.32× (5.04–5.61) | 10.69× (10.19–11.22) | 9.82× (9.36–10.31) | 3.17× (3.01–3.33) | 3.65× (3.47–3.83) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.292 s | 719 | 15.4 | 2.41× (2.37–2.45) | 3.67× (3.61–3.73) | 4.50× (4.43–4.58) | 4.18× (4.13–4.24) | 4.15× (4.09–4.21) | 12.89× (12.71–13.07) | 11.84× (11.67–12.02) | 3.82× (3.76–3.87) | 4.40× (4.33–4.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,005 | 3.3 | 0.63× (0.59–0.68) | 3.61× (3.56–3.66) | 4.57× (4.51–4.64) | 3.65× (3.60–3.70) | 6.24× (6.18–6.30) | 3.06× (3.02–3.10) | 2.74× (2.70–2.78) | 1.02× (1.01–1.03) | 1.56× (1.54–1.58) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 983 | 3.4 | 0.58× (0.57–0.59) | 3.04× (3.00–3.08) | 6.00× (5.93–6.08) | 3.61× (3.56–3.66) | 6.13× (6.06–6.21) | 2.99× (2.95–3.03) | 2.68× (2.64–2.71) | 0.99× (0.97–1.02) | 1.52× (1.49–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 973 | 3.3 | 0.59× (0.57–0.60) | 3.01× (2.96–3.06) | 5.92× (5.82–6.02) | 3.54× (3.49–3.60) | 6.04× (5.96–6.12) | 2.95× (2.91–2.99) | 2.64× (2.60–2.68) | 0.98× (0.97–1.00) | 1.50× (1.47–1.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 565 | 5.6 | 2.72× (2.62–2.83) | 2.93× (2.91–2.95) | 3.76× (3.67–3.86) | 3.69× (3.67–3.71) | 3.96× (3.93–3.98) | 2.47× (2.45–2.48) | 2.06× (2.01–2.12) | 1.94× (1.93–1.95) | 2.92× (2.90–2.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 856 | 6.8 | 2.20× (2.02–2.39) | 2.73× (2.50–2.97) | 3.92× (3.61–4.25) | 3.24× (3.00–3.49) | 4.59× (4.30–4.89) | 3.68× (3.44–3.94) | 3.08× (2.95–3.21) | 2.90× (2.72–3.09) | 4.35× (4.08–4.64) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 772 | 13.3 | 1.79× (1.69–1.88) | 1.95× (1.82–2.08) | 2.86× (2.73–3.00) | 2.21× (2.09–2.33) | 3.74× (3.55–3.93) | 3.31× (3.13–3.49) | 2.77× (2.60–2.94) | 2.60× (2.47–2.75) | 3.91× (3.70–4.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 48.6 | 1.37× (1.36–1.37) | 2.70× (2.67–2.73) | 2.46× (2.42–2.50) | 2.48× (2.48–2.49) | 2.86× (2.84–2.88) | 3.16× (3.14–3.17) | 3.12× (3.11–3.13) | 1.26× (1.25–1.27) | 1.41× (1.41–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 101 | 54.9 | 1.30× (1.26–1.34) | 2.56× (2.50–2.61) | 2.41× (2.34–2.49) | 3.10× (2.99–3.22) | 2.94× (2.85–3.03) | 5.09× (4.94–5.24) | 5.04× (4.88–5.19) | 2.03× (1.97–2.10) | 2.28× (2.21–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 75.4 | 1.30× (1.28–1.32) | 2.11× (2.07–2.16) | 2.05× (1.97–2.14) | 2.55× (2.49–2.61) | 2.80× (2.74–2.85) | 5.94× (5.85–6.03) | 5.88× (5.80–5.96) | 2.37× (2.34–2.41) | 2.66× (2.62–2.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 45.0 | 1.46× (1.45–1.47) | 3.05× (3.01–3.08) | 2.53× (2.52–2.54) | 2.65× (2.62–2.67) | 3.06× (3.03–3.09) | 3.62× (3.58–3.65) | 3.81× (3.78–3.83) | 1.43× (1.42–1.44) | 1.69× (1.68–1.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 51.5 | 1.44× (1.36–1.53) | 2.84× (2.71–2.97) | 2.53× (2.44–2.63) | 3.54× (3.39–3.69) | 3.12× (3.05–3.20) | 5.95× (5.83–6.08) | 6.26× (6.13–6.41) | 2.35× (2.30–2.39) | 2.79× (2.72–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.075 s | 128 | 71.8 | 1.43× (1.40–1.46) | 2.27× (2.21–2.34) | 2.23× (2.17–2.29) | 3.02× (2.95–3.09) | 2.82× (2.76–2.88) | 6.92× (6.80–7.05) | 7.29× (7.14–7.43) | 2.73× (2.67–2.78) | 3.24× (3.16–3.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 752 | 4.5 | 0.67× (0.67–0.68) | 3.16× (3.01–3.32) | 4.54× (4.50–4.58) | 3.16× (3.15–3.18) | 6.82× (6.79–6.85) | 2.58× (2.56–2.60) | 2.36× (2.35–2.38) | 0.99× (0.98–1.00) | 1.56× (1.54–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 709 | 4.9 | 0.57× (0.56–0.58) | 2.92× (2.89–2.95) | 5.04× (4.98–5.09) | 3.02× (2.99–3.05) | 6.47× (6.41–6.53) | 2.44× (2.41–2.46) | 2.23× (2.20–2.26) | 0.93× (0.92–0.95) | 1.47× (1.45–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 704 | 4.8 | 0.57× (0.56–0.58) | 2.91× (2.88–2.94) | 4.99× (4.93–5.05) | 3.00× (2.96–3.03) | 6.46× (6.40–6.53) | 2.42× (2.39–2.44) | 2.21× (2.19–2.24) | 0.93× (0.91–0.94) | 1.46× (1.44–1.47) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 0.086 s | 141 | 21.6 | 1.51× (1.50–1.52) | 3.09× (3.06–3.12) | 3.41× (3.36–3.45) | 3.39× (3.36–3.42) | 3.58× (3.55–3.62) | 3.96× (3.93–4.00) | 3.62× (3.58–3.65) | 1.16× (1.15–1.17) | 1.42× (1.41–1.43) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 0.056 s | 214 | 25.4 | 1.28× (1.23–1.33) | 2.83× (2.46–3.26) | 3.34× (3.24–3.45) | 3.64× (3.46–3.82) | 3.37× (3.27–3.46) | 6.04× (5.86–6.23) | 5.51× (5.34–5.68) | 1.77× (1.71–1.82) | 2.16× (2.10–2.23) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 0.049 s | 246 | 37.5 | 1.22× (1.16–1.29) | 1.97× (1.85–2.10) | 2.73× (2.59–2.88) | 2.80× (2.63–2.99) | 2.91× (2.77–3.06) | 6.77× (6.45–7.10) | 6.17× (5.89–6.47) | 1.98× (1.89–2.07) | 2.42× (2.31–2.55) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.49× (2.46–2.52) | 5.61× (5.54–5.68) | 5.76× (5.70–5.82) | 6.28× (6.20–6.37) | 5.65× (5.59–5.72) | 6.17× (6.10–6.24) | 5.67× (5.60–5.75) | 1.83× (1.81–1.85) | 2.10× (2.07–2.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.41× (2.40–2.42) | 5.40× (5.36–5.43) | 5.63× (5.61–5.65) | 6.20× (6.16–6.25) | 5.37× (5.27–5.47) | 5.72× (5.69–5.75) | 5.25× (5.22–5.29) | 1.70× (1.69–1.71) | 1.95× (1.93–1.96) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.35× (2.30–2.39) | 3.88× (3.80–3.96) | 4.16× (4.09–4.24) | 4.42× (4.34–4.50) | 3.88× (3.82–3.94) | 3.51× (3.44–3.58) | 3.22× (3.16–3.29) | 1.04× (1.02–1.06) | 1.19× (1.17–1.22) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.62–0.71) | 3.36× (3.30–3.41) | 4.15× (4.09–4.21) | 3.42× (3.37–3.47) | 5.78× (5.71–5.85) | 2.88× (2.83–2.92) | 2.58× (2.54–2.63) | 1.02× (1.00–1.03) | 1.09× (1.06–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.11× (3.07–3.15) | 5.43× (5.38–5.48) | 3.35× (3.31–3.39) | 5.60× (5.54–5.67) | 2.77× (2.73–2.80) | 2.48× (2.45–2.51) | 0.98× (0.96–1.00) | 1.05× (1.01–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.61–0.63) | 3.43× (3.37–3.49) | 5.45× (5.38–5.53) | 3.36× (3.31–3.41) | 5.60× (5.54–5.67) | 2.79× (2.75–2.82) | 2.50× (2.46–2.54) | 0.99× (0.97–1.00) | 1.06× (1.02–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.53–2.73) | 2.70× (2.68–2.71) | 3.42× (3.30–3.53) | 3.54× (3.52–3.56) | 3.79× (3.77–3.82) | 2.39× (2.37–2.40) | 2.01× (1.95–2.06) | 1.89× (1.88–1.91) | 2.61× (2.59–2.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.20× (2.17–2.23) | 2.46× (2.42–2.51) | 2.99× (2.87–3.11) | 3.04× (3.00–3.08) | 3.25× (3.20–3.30) | 1.93× (1.90–1.97) | 1.63× (1.59–1.66) | 1.54× (1.51–1.56) | 2.12× (2.09–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.94–2.07) | 1.72× (1.64–1.79) | 2.14× (2.07–2.21) | 1.99× (1.92–2.07) | 2.05× (1.99–2.11) | 1.01× (0.98–1.04) | 0.85× (0.82–0.87) | 0.80× (0.78–0.83) | 1.10× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.36–1.37) | 2.66× (2.64–2.67) | 2.41× (2.38–2.43) | 2.46× (2.45–2.47) | 2.84× (2.82–2.85) | 3.12× (3.11–3.13) | 3.09× (3.08–3.10) | 1.26× (1.25–1.26) | 1.38× (1.37–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.28–1.33) | 2.49× (2.47–2.52) | 2.20× (2.18–2.22) | 2.94× (2.91–2.97) | 2.59× (2.57–2.61) | 2.77× (2.74–2.79) | 2.74× (2.72–2.76) | 1.11× (1.10–1.12) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.44–1.52) | 2.29× (2.25–2.33) | 1.86× (1.84–1.89) | 2.58× (2.54–2.62) | 2.42× (2.38–2.46) | 2.02× (1.99–2.05) | 2.00× (1.97–2.03) | 0.81× (0.80–0.82) | 0.89× (0.88–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.47) | 2.99× (2.96–3.02) | 2.50× (2.49–2.51) | 2.62× (2.60–2.65) | 3.03× (3.00–3.06) | 3.58× (3.55–3.61) | 3.77× (3.75–3.79) | 1.42× (1.41–1.43) | 1.66× (1.64–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.35–1.49) | 2.82× (2.73–2.90) | 2.34× (2.27–2.40) | 3.24× (3.19–3.30) | 2.78× (2.72–2.84) | 3.15× (3.11–3.19) | 3.32× (3.28–3.36) | 1.25× (1.24–1.26) | 1.46× (1.44–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.55–1.67) | 2.54× (2.50–2.59) | 1.96× (1.92–1.99) | 2.84× (2.80–2.89) | 2.55× (2.51–2.59) | 2.26× (2.23–2.28) | 2.38× (2.35–2.41) | 0.89× (0.88–0.91) | 1.04× (1.03–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.72) | 2.76× (2.74–2.79) | 3.59× (3.57–3.62) | 2.89× (2.87–2.91) | 6.08× (6.03–6.13) | 2.38× (2.36–2.41) | 2.19× (2.17–2.22) | 0.99× (0.98–1.01) | 0.99× (0.97–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.62–0.65) | 2.55× (2.52–2.58) | 4.30× (4.25–4.35) | 2.79× (2.76–2.81) | 5.72× (5.67–5.77) | 2.22× (2.20–2.24) | 2.04× (2.02–2.06) | 0.93× (0.91–0.94) | 0.92× (0.91–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.82× (2.79–2.85) | 4.36× (4.30–4.41) | 2.84× (2.81–2.87) | 5.84× (5.79–5.89) | 2.25× (2.23–2.27) | 2.07× (2.05–2.09) | 0.94× (0.92–0.95) | 0.94× (0.92–0.96) |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 1.50× (1.49–1.51) | 3.01× (2.99–3.04) | 3.27× (3.24–3.30) | 3.34× (3.30–3.37) | 3.53× (3.49–3.56) | 3.90× (3.86–3.93) | 3.56× (3.53–3.59) | 1.16× (1.15–1.16) | 1.36× (1.34–1.37) |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 1.37× (1.35–1.39) | 2.71× (2.67–2.75) | 2.96× (2.92–2.99) | 3.30× (3.26–3.34) | 3.08× (3.05–3.11) | 3.32× (3.29–3.36) | 3.04× (3.00–3.07) | 0.99× (0.98–1.00) | 1.16× (1.14–1.17) |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 1.51× (1.44–1.58) | 2.26× (2.22–2.30) | 2.43× (2.41–2.45) | 2.66× (2.62–2.69) | 2.57× (2.54–2.60) | 2.26× (2.23–2.28) | 2.06× (2.03–2.09) | 0.67× (0.66–0.68) | 0.78× (0.77–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.34) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.10× (12.08–12.13) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.59× (0.50–0.70) | 0.80× (0.79–0.82) | 0.67× (0.66–0.67) | 6.35× (6.31–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.75× (0.66–0.87) | 0.70× (0.69–0.71) | 0.62× (0.61–0.63) | 4.09× (4.07–4.11) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.42× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.72) | 0.67× (0.66–0.67) | 8.08× (8.05–8.10) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.70) | 0.70× (0.66–0.74) | 1.64× (1.61–1.68) | 0.81× (0.80–0.83) | 5.94× (5.83–6.05) | 0.48× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.71× (0.66–0.76) | 1.14× (1.09–1.19) | 0.69× (0.66–0.72) | 3.35× (3.21–3.50) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.25) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.42× (0.39–0.45) | 0.94× (0.91–0.97) | 0.74× (0.71–0.78) | 3.25× (3.15–3.36) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.64) | 0.45× (0.44–0.46) | 0.86× (0.84–0.87) | 0.81× (0.79–0.82) | 2.49× (2.44–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.84× (0.83–0.84) | 5.24× (5.23–5.25) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.51) | 0.37× (0.34–0.41) | 0.86× (0.83–0.89) | 0.73× (0.69–0.76) | 2.95× (2.85–3.06) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.51) | 0.81× (0.79–0.82) | 0.80× (0.77–0.82) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.09× (1.08–1.09) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.36× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.20–0.22) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.35× (5.32–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.63× (1.63–1.63) | 0.77× (0.77–0.78) | 5.39× (5.37–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 18 MB | 0.48× (0.48–0.48) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.69× (0.68–0.70) | 6.73× (6.72–6.75) | 0.41× (0.41–0.41) | 0.44× (0.44–0.45) | 0.66× (0.66–0.67) | 0.33× (0.32–0.33) |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 32 MB | 0.52× (0.51–0.53) | 0.46× (0.42–0.50) | 1.24× (1.21–1.27) | 0.77× (0.73–0.81) | 3.85× (3.77–3.93) | 0.23× (0.23–0.24) | 0.25× (0.25–0.26) | 0.38× (0.37–0.39) | 0.19× (0.18–0.19) |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 49 MB | 0.56× (0.54–0.57) | 0.53× (0.49–0.58) | 1.03× (1.01–1.05) | 0.80× (0.77–0.84) | 2.62× (2.58–2.65) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 344 | 2.91 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 596 | 3.15 | 1.08× (1.07–1.09) | 86% | 1.04× (1.03–1.05) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 719 | 5.18 | 1.76× (1.72–1.80) | 52% | 1.22× (1.21–1.23) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (719 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.12, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.59, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.87 | 0.02 | 0.09 | 0.292 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2968 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 12.6% |
| MTF + RLE2 | 1.45 | 17.8% |
| T-vector build | 1.38 | 17.0% |
| walk (the one each build uses) | 3.76 | 46.4% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.27 | 3.3% |
| CRC | 0.23 | 2.9% |
| **total** | 8.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.494 | 1.301 | (in its total) | 2.64× (2.63–2.64) |  |
| walk | 1.224 | 2.621 | (in its total) | 2.14× (2.14–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.23× (1.22–1.24) |  |
| inverse BWT, total | 1.762 | 3.976 | 3.427 | 2.26× (2.25–2.26) | 1.95× (1.94–1.95) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h28m ± 10m23s |
| 4 threads | 42m43s ± 6m01s |
| 4 threads, the sample's overlap gap as a fixed cost | 31m34s ± 4m31s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 31m34s ± 4m31s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 4.14 ns a byte of output at 4 threads (CPU, summed over them), 2.42 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,384,894 | 12,023,507 | \`4c10ae1761b31e94…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2967.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
