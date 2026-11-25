# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-845/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`1bfdf17`](<https://github.com/lmmx/mibz2/commit/1bfdf172cb9dfc51a1efae9fd2314f53689a156a>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45520156150>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.296 s | 708 | 3.7 | 2.89× (2.87–2.91) | 7.13× (7.03–7.24) | 7.56× (7.43–7.69) | 8.48× (8.36–8.60) | 7.62× (7.50–7.74) | 7.76× (7.66–7.87) | 7.40× (7.31–7.49) | 2.27× (2.21–2.32) | 2.61× (2.57–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.179 s | 1,174 | 4.2 | 3.22× (3.08–3.36) | 6.50× (6.27–6.73) | 8.18× (7.89–8.48) | 8.06× (7.81–8.33) | 7.01× (6.73–7.31) | 13.04× (12.51–13.58) | 12.42× (11.94–12.91) | 3.80× (3.67–3.94) | 4.38× (4.23–4.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.153 s | 1,368 | 6.8 | 3.20× (3.05–3.35) | 4.91× (4.71–5.12) | 6.72× (6.40–7.07) | 5.98× (5.80–6.16) | 5.90× (5.76–6.05) | 15.36× (14.81–15.92) | 14.63× (14.18–15.09) | 4.48× (4.33–4.64) | 5.16× (5.05–5.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,389 | 2.1 | 0.56× (0.55–0.57) | 3.57× (3.48–3.66) | 4.40× (4.29–4.51) | 3.65× (3.56–3.75) | 6.51× (6.32–6.71) | 2.76× (2.68–2.85) | 2.76× (2.68–2.84) | 1.02× (1.00–1.04) | 2.15× (2.10–2.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,312 | 2.3 | 0.48× (0.47–0.49) | 2.74× (2.64–2.84) | 5.57× (5.29–5.86) | 3.55× (3.26–3.87) | 6.00× (5.85–6.16) | 2.58× (2.53–2.63) | 2.58× (2.54–2.62) | 0.95× (0.92–0.98) | 2.00× (1.96–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,274 | 2.3 | 0.48× (0.46–0.49) | 2.81× (2.52–3.12) | 5.48× (5.23–5.75) | 3.41× (3.31–3.52) | 5.96× (5.78–6.14) | 2.53× (2.46–2.60) | 2.53× (2.46–2.60) | 0.93× (0.90–0.96) | 1.96× (1.92–2.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,040 | 2.7 | 2.82× (2.64–3.01) | 3.64× (3.41–3.89) | 4.41× (4.06–4.79) | 4.44× (4.25–4.64) | 5.27× (5.00–5.55) | 2.94× (2.75–3.13) | 2.56× (2.43–2.70) | 2.12× (2.02–2.24) | 3.20× (3.05–3.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,352 | 3.8 | 2.35× (1.98–2.78) | 2.87× (2.69–3.07) | 4.43× (4.00–4.90) | 3.69× (3.46–3.92) | 5.46× (5.09–5.85) | 3.90× (3.59–4.23) | 3.40× (3.17–3.65) | 2.82× (2.63–3.03) | 4.25× (3.98–4.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,167 | 7.2 | 1.85× (1.74–1.98) | 2.17× (1.79–2.63) | 3.13× (2.84–3.45) | 2.18× (2.04–2.33) | 4.61× (4.10–5.19) | 3.45× (3.15–3.79) | 3.01× (2.76–3.28) | 2.50× (2.30–2.71) | 3.77× (3.46–4.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 96 | 27.5 | 1.37× (1.35–1.39) | 3.08× (2.92–3.25) | 2.82× (2.69–2.95) | 2.83× (2.66–3.00) | 3.07× (2.98–3.16) | 3.45× (3.35–3.56) | 3.43× (3.32–3.55) | 1.32× (1.29–1.36) | 1.53× (1.48–1.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 32.4 | 1.38× (1.27–1.51) | 2.96× (2.81–3.13) | 2.75× (2.63–2.88) | 3.38× (3.22–3.56) | 3.17× (3.00–3.35) | 5.52× (5.25–5.81) | 5.49× (5.19–5.80) | 2.12× (2.03–2.21) | 2.45× (2.33–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 193 | 43.4 | 1.37× (1.34–1.39) | 2.65× (2.49–2.82) | 2.72× (2.60–2.85) | 3.04× (2.88–3.22) | 3.32× (3.21–3.43) | 6.81× (6.55–7.08) | 6.77× (6.49–7.07) | 2.61× (2.53–2.70) | 3.02× (2.90–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 24.1 | 1.50× (1.45–1.54) | 3.64× (3.49–3.80) | 3.00× (2.86–3.15) | 3.00× (2.88–3.13) | 3.40× (3.26–3.55) | 4.45× (4.35–4.56) | 4.60× (4.36–4.85) | 1.57× (1.52–1.63) | 1.86× (1.75–1.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 172 | 29.2 | 1.29× (1.12–1.49) | 3.07× (2.72–3.47) | 2.73× (2.34–3.19) | 3.61× (3.15–4.14) | 3.16× (2.79–3.58) | 6.63× (5.82–7.57) | 6.85× (6.11–7.67) | 2.34× (2.08–2.64) | 2.76× (2.41–3.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 205 | 40.2 | 1.47× (1.41–1.52) | 2.76× (2.67–2.86) | 2.69× (2.59–2.78) | 3.48× (3.39–3.58) | 3.54× (3.18–3.94) | 8.44× (8.10–8.79) | 8.71× (8.31–9.12) | 2.98× (2.91–3.06) | 3.51× (3.33–3.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,053 | 2.9 | 0.62× (0.61–0.63) | 3.23× (3.16–3.31) | 4.48× (4.37–4.59) | 3.17× (3.08–3.28) | 7.22× (7.12–7.33) | 2.31× (2.27–2.35) | 2.37× (2.30–2.45) | 0.98× (0.95–1.01) | 2.16× (2.12–2.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 952 | 3.4 | 0.46× (0.42–0.50) | 2.83× (2.60–3.08) | 4.90× (4.50–5.33) | 2.75× (2.52–3.01) | 6.11× (5.68–6.58) | 2.02× (1.85–2.20) | 2.07× (1.90–2.26) | 0.86× (0.78–0.94) | 1.89× (1.73–2.06) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 952 | 3.2 | 0.48× (0.47–0.49) | 2.97× (2.92–3.02) | 5.11× (5.03–5.20) | 2.93× (2.86–2.99) | 6.35× (6.24–6.46) | 2.11× (2.07–2.15) | 2.16× (2.10–2.23) | 0.90× (0.87–0.92) | 1.98× (1.94–2.01) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.046 s | 300 | 9.0 | 1.73× (1.62–1.84) | 4.14× (3.92–4.37) | 4.53× (4.27–4.81) | 4.60× (4.36–4.86) | 4.81× (4.57–5.07) | 4.92× (4.65–5.20) | 4.67× (4.46–4.88) | 1.49× (1.40–1.58) | 1.72× (1.62–1.83) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.033 s | 422 | 12.1 | 1.41× (1.22–1.63) | 3.41× (2.96–3.93) | 4.24× (3.79–4.75) | 4.51× (3.96–5.13) | 4.25× (3.77–4.81) | 6.87× (6.06–7.78) | 6.52× (5.75–7.40) | 2.08× (1.83–2.35) | 2.41× (2.13–2.71) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.030 s | 460 | 18.6 | 1.46× (1.43–1.50) | 2.95× (2.73–3.18) | 3.88× (3.65–4.13) | 3.69× (3.56–3.82) | 4.15× (3.99–4.32) | 7.89× (7.42–8.38) | 7.49× (7.02–7.98) | 2.39× (2.28–2.50) | 2.76× (2.60–2.93) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.86–2.90) | 7.08× (6.97–7.19) | 7.51× (7.38–7.63) | 8.44× (8.32–8.56) | 7.58× (7.46–7.70) | 7.72× (7.62–7.83) | 7.36× (7.27–7.45) | 2.26× (2.20–2.31) | 2.59× (2.54–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.78–2.90) | 6.69× (6.55–6.83) | 7.40× (7.27–7.53) | 8.24× (8.07–8.42) | 6.93× (6.74–7.13) | 6.87× (6.67–7.07) | 6.55× (6.39–6.71) | 2.01× (1.95–2.07) | 2.30× (2.26–2.35) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.00× (2.91–3.09) | 5.32× (5.17–5.47) | 6.21× (5.98–6.46) | 6.53× (6.39–6.69) | 5.67× (5.53–5.81) | 4.32× (4.12–4.52) | 4.11× (3.95–4.28) | 1.26× (1.21–1.32) | 1.45× (1.40–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.99× (2.91–3.08) | 3.89× (3.81–3.98) | 3.37× (3.29–3.46) | 5.93× (5.77–6.10) | 2.58× (2.50–2.65) | 2.57× (2.51–2.64) | 1.02× (1.00–1.03) | 1.83× (1.78–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.52–0.53) | 2.60× (2.56–2.63) | 4.74× (4.54–4.94) | 3.18× (2.93–3.45) | 5.28× (5.16–5.41) | 2.32× (2.28–2.36) | 2.31× (2.28–2.35) | 0.91× (0.89–0.94) | 1.65× (1.60–1.69) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.53) | 2.91× (2.65–3.19) | 4.62× (4.46–4.79) | 3.13× (3.03–3.23) | 5.34× (5.19–5.50) | 2.33× (2.27–2.39) | 2.32× (2.26–2.38) | 0.92× (0.89–0.94) | 1.65× (1.61–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.48–2.85) | 3.08× (2.92–3.24) | 3.78× (3.47–4.11) | 4.14× (3.94–4.35) | 4.90× (4.62–5.20) | 2.76× (2.58–2.96) | 2.42× (2.28–2.57) | 2.02× (1.92–2.14) | 2.70× (2.54–2.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.07–2.66) | 2.54× (2.41–2.67) | 3.08× (2.96–3.20) | 3.25× (3.12–3.38) | 3.79× (3.60–3.99) | 2.04× (1.92–2.18) | 1.79× (1.70–1.89) | 1.50× (1.42–1.58) | 1.99× (1.89–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.96–2.11) | 1.89× (1.52–2.35) | 2.20× (2.07–2.34) | 2.15× (2.05–2.26) | 2.37× (2.17–2.58) | 1.07× (0.98–1.15) | 0.93× (0.87–1.00) | 0.78× (0.73–0.84) | 1.04× (0.97–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.38) | 3.00× (2.84–3.16) | 2.72× (2.60–2.85) | 2.79× (2.62–2.96) | 3.03× (2.94–3.11) | 3.40× (3.30–3.50) | 3.38× (3.27–3.50) | 1.32× (1.28–1.36) | 1.45× (1.42–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.28–1.37) | 2.79× (2.74–2.85) | 2.41× (2.36–2.46) | 3.11× (3.02–3.20) | 2.73× (2.65–2.82) | 2.96× (2.85–3.08) | 2.94× (2.82–3.07) | 1.15× (1.11–1.19) | 1.26× (1.23–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.40–1.46) | 2.65× (2.57–2.74) | 2.24× (2.17–2.32) | 2.84× (2.73–2.95) | 2.72× (2.66–2.78) | 2.18× (2.11–2.25) | 2.17× (2.09–2.24) | 0.84× (0.82–0.87) | 0.93× (0.91–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.44–1.53) | 3.54× (3.39–3.70) | 2.92× (2.79–3.06) | 2.96× (2.84–3.09) | 3.36× (3.22–3.50) | 4.38× (4.28–4.49) | 4.53× (4.29–4.77) | 1.56× (1.51–1.62) | 1.77× (1.68–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.29× (1.13–1.48) | 3.03× (2.66–3.46) | 2.43× (2.13–2.78) | 3.35× (2.92–3.84) | 2.80× (2.46–3.19) | 3.50× (3.05–4.03) | 3.62× (3.19–4.11) | 1.25× (1.10–1.42) | 1.42× (1.23–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.54–1.64) | 2.93× (2.86–3.02) | 2.28× (2.17–2.39) | 3.15× (3.11–3.20) | 3.03× (2.70–3.40) | 2.64× (2.55–2.74) | 2.73× (2.63–2.83) | 0.94× (0.92–0.96) | 1.07× (1.02–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.69) | 2.55× (2.50–2.60) | 3.39× (3.30–3.49) | 2.84× (2.76–2.93) | 6.28× (6.19–6.38) | 2.11× (2.07–2.14) | 2.16× (2.10–2.23) | 0.98× (0.96–1.01) | 1.50× (1.46–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.51× (0.47–0.57) | 2.02× (1.82–2.23) | 3.55× (3.17–3.98) | 2.40× (2.16–2.66) | 5.06× (4.63–5.53) | 1.74× (1.57–1.93) | 1.79× (1.61–1.98) | 0.81× (0.73–0.91) | 1.24× (1.12–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.51–0.53) | 2.33× (2.30–2.36) | 3.83× (3.74–3.91) | 2.65× (2.60–2.71) | 5.47× (5.38–5.57) | 1.90× (1.87–1.93) | 1.95× (1.90–2.01) | 0.89× (0.87–0.91) | 1.35× (1.32–1.39) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.70× (1.59–1.80) | 3.92× (3.71–4.14) | 4.28× (4.04–4.54) | 4.46× (4.23–4.70) | 4.67× (4.44–4.90) | 4.76× (4.52–5.02) | 4.52× (4.33–4.72) | 1.46× (1.38–1.56) | 1.60× (1.50–1.71) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.41× (1.23–1.62) | 3.25× (2.78–3.79) | 3.48× (3.09–3.93) | 3.94× (3.46–4.48) | 3.64× (3.21–4.14) | 3.55× (3.12–4.03) | 3.37× (2.96–3.83) | 1.09× (0.96–1.24) | 1.19× (1.05–1.35) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.59× (1.54–1.64) | 2.91× (2.75–3.08) | 3.19× (3.07–3.31) | 3.44× (3.33–3.55) | 3.27× (3.17–3.37) | 2.42× (2.29–2.57) | 2.30× (2.16–2.45) | 0.74× (0.72–0.77) | 0.81× (0.78–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.34× (0.33–0.35) | 0.97× (0.97–0.98) | 0.80× (0.79–0.82) | 11.01× (10.98–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.45× (0.40–0.50) | 0.71× (0.68–0.74) | 0.59× (0.58–0.61) | 5.66× (5.47–5.85) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.36–0.39) | 0.60× (0.49–0.73) | 0.67× (0.65–0.70) | 0.59× (0.57–0.62) | 3.88× (3.74–4.03) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.43–0.46) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.93–7.96) | 0.65× (0.65–0.65) | 0.67× (0.66–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.72× (0.68–0.75) | 0.70× (0.67–0.75) | 1.66× (1.62–1.69) | 0.84× (0.82–0.86) | 6.00× (5.88–6.13) | 0.49× (0.48–0.50) | 0.51× (0.50–0.51) | 0.56× (0.54–0.57) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.66× (0.62–0.70) | 0.69× (0.64–0.74) | 1.19× (1.14–1.25) | 0.74× (0.70–0.78) | 3.51× (3.35–3.68) | 0.28× (0.26–0.29) | 0.29× (0.27–0.30) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.33) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.39–0.43) | 0.90× (0.88–0.92) | 0.70× (0.68–0.73) | 3.12× (3.04–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.46–0.47) | 0.89× (0.87–0.90) | 0.84× (0.81–0.86) | 2.58× (2.54–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.44–0.48) | 0.36× (0.32–0.39) | 0.83× (0.79–0.86) | 0.69× (0.65–0.74) | 2.84× (2.73–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.46× (0.44–0.48) | 0.80× (0.78–0.82) | 0.81× (0.79–0.84) | 2.31× (2.25–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.31× (5.29–5.33) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.25) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.18–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.27× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.90) | 6.20× (6.18–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.67× (0.67–0.67) | 6.41× (6.40–6.42) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.46× (0.45–0.48) | 0.42× (0.39–0.44) | 1.06× (1.03–1.10) | 0.71× (0.68–0.74) | 3.44× (3.32–3.56) | 0.21× (0.20–0.22) | 0.22× (0.21–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 56 MB | 0.51× (0.49–0.52) | 0.46× (0.42–0.50) | 0.94× (0.90–0.98) | 0.76× (0.73–0.79) | 2.33× (2.28–2.39) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 708 | 1.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,174 | 1.63 | 1.12× (1.10–1.15) | 84% | 1.06× (1.03–1.09) | 1.11× (1.08–1.14) |
| 4: all 4 CPUs, sharing SMT siblings | 1,368 | 2.62 | 1.79× (1.72–1.86) | 49% | 1.34× (1.28–1.41) | 1.86× (1.77–1.97) |

**Best:** mibz2 at 4 threads (1,368 MB/s); lbzip2 at 4 threads (276 MB/s); pbz2 at 4 threads (430 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.24, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.41 | 0.01 | 0.07 | 0.153 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.50 | 11.4% |
| MTF + RLE2 | 0.89 | 20.2% |
| T-vector build | 0.80 | 18.3% |
| walk (the one each build uses) | 1.94 | 44.3% |
| of which: segment assembly | 0.09 | 2.0% |
| RLE1 | 0.09 | 2.2% |
| CRC | 0.15 | 3.5% |
| **total** | 4.38 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.297 | 0.702 | (in its total) | 2.38× (2.34–2.42) |  |
| walk | 0.664 | 1.558 | (in its total) | 2.38× (2.33–2.43) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.029 | 0.035 | (in its total) | 1.21× (1.15–1.27) |  |
| inverse BWT, total | 0.991 | 2.294 | 1.297 | 2.34× (2.30–2.39) | 1.32× (1.29–1.35) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m09s ± 5m06s |
| 4 threads | 21m50s ± 3m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m50s ± 2m12s |
| the scanner alone (its serial part) | 6m59s ± 26.3s |
| the workers alone, over 4 | 14m50s ± 2m12s |

At 4 threads the workers bound it. Per byte: the scanner 4.0 ns a byte of bz2, the workers 1.95 ns a byte of output at 4 threads (CPU, summed over them), 1.19 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
