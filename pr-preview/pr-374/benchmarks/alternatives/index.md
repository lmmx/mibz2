# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-374/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`d504898`](<https://github.com/lmmx/mibz2/commit/d50489823f3c719afe8ea001b7d966275467a6c6>) (2026-11-12).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43268798106>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.432 s | 485 | 7.6 | 2.79× (2.75–2.83) | 6.38× (6.30–6.47) | 6.60× (6.50–6.69) | 7.52× (7.42–7.62) | 6.61× (6.52–6.71) | 6.95× (6.85–7.05) | 6.55× (6.47–6.63) | 2.06× (2.04–2.09) | 2.31× (2.28–2.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.252 s | 831 | 8.4 | 2.86× (2.73–3.00) | 5.80× (5.64–5.96) | 6.78× (6.50–7.07) | 6.96× (6.76–7.17) | 6.16× (5.98–6.34) | 12.02× (11.65–12.40) | 11.32× (10.97–11.69) | 3.57× (3.46–3.69) | 3.99× (3.86–4.13) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.222 s | 946 | 14.1 | 2.60× (2.54–2.65) | 3.84× (3.77–3.91) | 4.78× (4.65–4.91) | 4.69× (4.57–4.81) | 4.60× (4.54–4.65) | 13.58× (13.38–13.77) | 12.79× (12.61–12.98) | 4.03× (3.99–4.08) | 4.51× (4.45–4.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,136 | 3.6 | 0.58× (0.58–0.59) | 3.49× (3.45–3.52) | 4.66× (4.61–4.71) | 3.63× (3.59–3.66) | 6.18× (6.09–6.27) | 2.71× (2.67–2.75) | 2.70× (2.67–2.73) | 0.99× (0.98–1.01) | 1.80× (1.68–1.92) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,143 | 3.6 | 0.54× (0.53–0.55) | 2.86× (2.78–2.95) | 5.74× (5.58–5.91) | 3.58× (3.48–3.67) | 6.04× (5.90–6.18) | 2.67× (2.60–2.73) | 2.66× (2.58–2.73) | 0.98× (0.94–1.01) | 1.77× (1.65–1.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,117 | 3.6 | 0.54× (0.53–0.56) | 2.86× (2.80–2.92) | 5.72× (5.61–5.83) | 3.57× (3.50–3.63) | 6.13× (6.01–6.25) | 2.65× (2.61–2.70) | 2.64× (2.59–2.70) | 0.97× (0.95–0.99) | 1.76× (1.64–1.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 748 | 5.2 | 2.75× (2.73–2.77) | 3.12× (3.10–3.14) | 3.97× (3.82–4.14) | 4.17× (4.15–4.19) | 4.58× (4.49–4.68) | 2.82× (2.80–2.85) | 2.30× (2.29–2.31) | 2.11× (2.09–2.12) | 3.11× (3.09–3.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,140 | 6.5 | 2.27× (2.18–2.36) | 2.98× (2.80–3.17) | 4.48× (4.18–4.80) | 3.61× (3.47–3.76) | 5.21× (5.01–5.42) | 4.21× (4.04–4.38) | 3.43× (3.30–3.57) | 3.14× (3.02–3.27) | 4.64× (4.45–4.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 973 | 12.7 | 1.87× (1.81–1.94) | 1.71× (1.59–1.85) | 2.93× (2.76–3.12) | 2.23× (2.08–2.39) | 4.12× (4.01–4.24) | 3.69× (3.59–3.79) | 3.01× (2.93–3.09) | 2.75× (2.68–2.83) | 4.06× (3.96–4.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 s | 73 | 51.1 | 1.39× (1.38–1.40) | 2.65× (2.61–2.68) | 2.45× (2.41–2.49) | 2.52× (2.50–2.54) | 2.84× (2.81–2.88) | 3.01× (2.99–3.03) | 3.01× (2.99–3.03) | 1.26× (1.24–1.29) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 117 | 57.6 | 1.25× (1.19–1.31) | 2.43× (2.36–2.51) | 2.31× (2.21–2.40) | 3.05× (2.92–3.18) | 2.83× (2.70–2.97) | 4.75× (4.54–4.97) | 4.75× (4.54–4.97) | 2.00× (1.90–2.10) | 2.22× (2.11–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.054 s | 147 | 76.2 | 1.37× (1.33–1.41) | 2.14× (2.06–2.23) | 2.18× (2.06–2.30) | 2.72× (2.62–2.81) | 2.94× (2.89–2.99) | 6.03× (5.90–6.17) | 6.03× (5.91–6.16) | 2.53× (2.47–2.60) | 2.82× (2.74–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 46.0 | 1.52× (1.48–1.56) | 3.03× (2.95–3.12) | 2.64× (2.58–2.70) | 2.75× (2.68–2.81) | 3.09× (3.02–3.17) | 3.67× (3.59–3.76) | 3.87× (3.77–3.96) | 1.49× (1.46–1.53) | 1.78× (1.74–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 52.7 | 1.37× (1.31–1.44) | 2.71× (2.60–2.83) | 2.50× (2.42–2.58) | 3.54× (3.38–3.71) | 3.05× (2.95–3.16) | 5.94× (5.75–6.13) | 6.25× (6.04–6.46) | 2.42× (2.35–2.49) | 2.88× (2.79–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 72.5 | 1.43× (1.40–1.45) | 2.27× (2.20–2.35) | 2.29× (2.23–2.36) | 3.12× (3.04–3.20) | 2.89× (2.81–2.97) | 7.01× (6.86–7.17) | 7.38× (7.22–7.56) | 2.85× (2.80–2.91) | 3.41× (3.33–3.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 4.9 | 0.63× (0.63–0.64) | 3.43× (3.20–3.67) | 4.43× (4.39–4.48) | 3.15× (3.11–3.18) | 6.77× (6.69–6.85) | 2.32× (2.29–2.34) | 2.35× (2.33–2.37) | 0.96× (0.95–0.98) | 1.78× (1.75–1.80) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 833 | 5.2 | 0.54× (0.53–0.55) | 2.58× (2.54–2.62) | 5.17× (5.09–5.25) | 3.08× (3.03–3.12) | 6.57× (6.40–6.75) | 2.25× (2.22–2.28) | 2.28× (2.25–2.32) | 0.94× (0.92–0.96) | 1.73× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 823 | 5.1 | 0.53× (0.52–0.54) | 2.53× (2.47–2.59) | 5.07× (4.95–5.20) | 3.02× (2.94–3.10) | 6.38× (6.21–6.56) | 2.20× (2.14–2.26) | 2.24× (2.18–2.29) | 0.92× (0.89–0.94) | 1.69× (1.65–1.73) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.057 s | 229 | 16.6 | 1.80× (1.78–1.82) | 3.91× (3.88–3.95) | 4.32× (4.21–4.44) | 4.57× (4.53–4.61) | 4.74× (4.69–4.78) | 4.73× (4.69–4.78) | 4.59× (4.54–4.65) | 1.49× (1.48–1.51) | 1.77× (1.76–1.79) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.041 s | 322 | 21.6 | 1.39× (1.31–1.47) | 3.22× (2.86–3.62) | 3.66× (3.43–3.91) | 4.02× (3.83–4.23) | 4.15× (3.96–4.36) | 6.46× (6.10–6.84) | 6.28× (5.93–6.64) | 2.04× (1.93–2.16) | 2.42× (2.29–2.56) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.036 s | 359 | 32.4 | 1.45× (1.42–1.49) | 2.40× (2.27–2.53) | 3.09× (2.95–3.23) | 3.28× (3.23–3.33) | 3.81× (3.73–3.89) | 7.53× (7.39–7.67) | 7.31× (7.18–7.45) | 2.38× (2.34–2.42) | 2.82× (2.77–2.88) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.78× (2.74–2.82) | 6.36× (6.27–6.44) | 6.56× (6.47–6.66) | 7.50× (7.40–7.60) | 6.59× (6.50–6.68) | 6.92× (6.82–7.03) | 6.53× (6.45–6.60) | 2.06× (2.03–2.09) | 2.29× (2.26–2.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.61× (2.55–2.66) | 5.90× (5.78–6.01) | 6.16× (6.04–6.28) | 7.17× (7.04–7.30) | 6.05× (5.95–6.15) | 6.26× (6.13–6.39) | 5.90× (5.78–6.02) | 1.86× (1.82–1.90) | 2.07× (2.02–2.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.50× (2.45–2.55) | 4.15× (4.11–4.19) | 4.44× (4.38–4.51) | 5.05× (4.99–5.11) | 4.38× (4.33–4.42) | 3.78× (3.74–3.82) | 3.56× (3.52–3.60) | 1.12× (1.11–1.14) | 1.25× (1.23–1.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 2.96× (2.94–2.99) | 3.92× (3.87–3.96) | 3.39× (3.36–3.42) | 5.72× (5.63–5.80) | 2.56× (2.53–2.59) | 2.55× (2.52–2.58) | 0.99× (0.98–1.01) | 1.66× (1.64–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.76× (2.69–2.83) | 4.99× (4.86–5.12) | 3.33× (3.25–3.41) | 5.53× (5.42–5.64) | 2.49× (2.44–2.54) | 2.48× (2.42–2.54) | 0.96× (0.94–0.99) | 1.61× (1.57–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.60) | 3.07× (3.01–3.13) | 5.01× (4.91–5.11) | 3.39× (3.33–3.45) | 5.71× (5.60–5.82) | 2.53× (2.49–2.56) | 2.52× (2.47–2.57) | 0.98× (0.96–1.00) | 1.64× (1.61–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.63–2.66) | 2.88× (2.86–2.89) | 3.49× (3.33–3.65) | 3.98× (3.96–3.99) | 4.37× (4.28–4.46) | 2.71× (2.69–2.74) | 2.23× (2.21–2.24) | 2.04× (2.03–2.05) | 2.68× (2.66–2.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.23–2.32) | 2.50× (2.42–2.57) | 3.34× (3.26–3.41) | 3.34× (3.27–3.41) | 3.62× (3.52–3.73) | 2.17× (2.13–2.20) | 1.78× (1.74–1.81) | 1.63× (1.60–1.66) | 2.14× (2.09–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.99–2.06) | 1.70× (1.66–1.73) | 2.26× (2.19–2.32) | 2.09× (2.01–2.18) | 2.16× (2.11–2.20) | 1.11× (1.09–1.13) | 0.91× (0.89–0.93) | 0.83× (0.82–0.85) | 1.10× (1.07–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.59× (2.57–2.60) | 2.40× (2.36–2.44) | 2.50× (2.48–2.52) | 2.82× (2.78–2.86) | 2.98× (2.96–3.00) | 2.98× (2.96–3.00) | 1.26× (1.24–1.28) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.24–1.31) | 2.43× (2.37–2.48) | 2.13× (2.08–2.18) | 2.90× (2.84–2.97) | 2.52× (2.46–2.58) | 2.62× (2.56–2.68) | 2.62× (2.55–2.68) | 1.11× (1.07–1.14) | 1.21× (1.18–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.38–1.51) | 2.29× (2.25–2.32) | 1.95× (1.92–1.99) | 2.67× (2.63–2.70) | 2.51× (2.48–2.55) | 2.01× (1.98–2.04) | 2.01× (1.98–2.04) | 0.85× (0.83–0.87) | 0.93× (0.92–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.47–1.55) | 2.97× (2.90–3.04) | 2.57× (2.50–2.63) | 2.72× (2.66–2.79) | 3.06× (2.99–3.14) | 3.63× (3.55–3.72) | 3.82× (3.73–3.92) | 1.49× (1.45–1.53) | 1.73× (1.69–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.35–1.49) | 2.75× (2.72–2.79) | 2.35× (2.31–2.39) | 3.30× (3.25–3.36) | 2.76× (2.72–2.81) | 3.20× (3.16–3.25) | 3.37× (3.32–3.42) | 1.31× (1.29–1.33) | 1.53× (1.50–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.56–1.64) | 2.53× (2.46–2.61) | 2.04× (1.99–2.09) | 2.87× (2.77–2.98) | 2.64× (2.59–2.70) | 2.32× (2.27–2.37) | 2.44× (2.39–2.50) | 0.95× (0.93–0.97) | 1.10× (1.08–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.50× (2.48–2.52) | 3.44× (3.38–3.49) | 2.87× (2.85–2.89) | 6.03× (5.95–6.12) | 2.15× (2.13–2.16) | 2.18× (2.16–2.19) | 0.97× (0.95–0.98) | 1.38× (1.37–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.60–0.61) | 2.32× (2.29–2.35) | 4.10× (4.03–4.17) | 2.82× (2.78–2.86) | 5.76× (5.63–5.90) | 2.04× (2.02–2.07) | 2.08× (2.05–2.10) | 0.92× (0.90–0.94) | 1.32× (1.30–1.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.60) | 2.54× (2.49–2.59) | 4.10× (4.05–4.15) | 2.86× (2.80–2.92) | 5.80× (5.67–5.93) | 2.07× (2.03–2.12) | 2.10× (2.06–2.15) | 0.93× (0.91–0.96) | 1.33× (1.30–1.36) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.78× (1.76–1.79) | 3.78× (3.75–3.80) | 4.17× (4.08–4.27) | 4.46× (4.42–4.50) | 4.63× (4.58–4.67) | 4.62× (4.58–4.66) | 4.49× (4.43–4.55) | 1.48× (1.46–1.49) | 1.63× (1.61–1.66) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.43× (1.38–1.48) | 3.07× (2.87–3.28) | 3.30× (3.18–3.43) | 3.90× (3.78–4.03) | 3.63× (3.48–3.78) | 3.51× (3.39–3.64) | 3.41× (3.29–3.54) | 1.12× (1.09–1.16) | 1.24× (1.19–1.29) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.63× (1.56–1.69) | 2.51× (2.43–2.59) | 2.78× (2.69–2.88) | 3.23× (3.14–3.32) | 3.05× (2.96–3.14) | 2.41× (2.35–2.48) | 2.35× (2.29–2.41) | 0.77× (0.75–0.79) | 0.85× (0.83–0.88) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.35× (0.34–0.36) | 1.05× (1.05–1.06) | 0.84× (0.84–0.85) | 11.92× (11.90–11.94) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.35–0.37) | 0.54× (0.45–0.65) | 0.73× (0.72–0.75) | 0.63× (0.62–0.63) | 5.93× (5.82–6.04) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.39) | 0.50× (0.43–0.58) | 0.67× (0.66–0.69) | 0.59× (0.58–0.61) | 3.96× (3.90–4.03) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.01× (7.99–8.03) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.63–0.68) | 0.62× (0.57–0.67) | 1.57× (1.51–1.63) | 0.79× (0.76–0.82) | 5.70× (5.48–5.92) | 0.46× (0.44–0.48) | 0.48× (0.46–0.50) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.70× (0.67–0.74) | 1.18× (1.16–1.19) | 0.72× (0.71–0.73) | 3.44× (3.39–3.49) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.72–0.73) | 5.20× (5.20–5.21) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.41× (0.38–0.44) | 0.90× (0.86–0.93) | 0.72× (0.70–0.75) | 3.11× (3.00–3.23) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.65) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.81× (0.79–0.84) | 2.49× (2.43–2.55) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.20–5.24) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.39× (0.36–0.42) | 0.86× (0.83–0.89) | 0.74× (0.70–0.77) | 2.95× (2.84–3.07) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.49× (0.46–0.51) | 0.81× (0.80–0.82) | 0.79× (0.75–0.84) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.36× (5.35–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.31× (5.30–5.33) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.41× (1.41–1.41) | 0.66× (0.65–0.66) | 6.39× (6.38–6.40) | 0.38× (0.38–0.38) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 41 MB | 0.41× (0.40–0.42) | 0.36× (0.33–0.39) | 0.95× (0.93–0.98) | 0.62× (0.61–0.64) | 3.04× (2.96–3.12) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 59 MB | 0.48× (0.46–0.49) | 0.42× (0.38–0.46) | 0.86× (0.85–0.88) | 0.70× (0.67–0.73) | 2.21× (2.15–2.27) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 485 | 2.07 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 831 | 2.28 | 1.11× (1.09–1.13) | 86% | 1.03× (1.02–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 946 | 3.83 | 1.83× (1.80–1.87) | 49% | 1.20× (1.19–1.21) | 1.65× (1.62–1.67) |

**Best:** mibz2 at 4 threads (946 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (365 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.20, lbzip2 1t 0.06, lbzip2 2t 0.27, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.61 | 0.02 | 0.08 | 0.222 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.8% |
| MTF + RLE2 | 1.58 | 18.3% |
| T-vector build | 1.48 | 17.2% |
| walk (the one each build uses) | 4.02 | 46.6% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.64 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.062 | (in its total) | 2.72× (2.68–2.75) |  |
| walk | 1.029 | 2.040 | (in its total) | 1.98× (1.96–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.040 | (in its total) | 1.11× (1.09–1.12) |  |
| inverse BWT, total | 1.455 | 3.141 | 1.860 | 2.16× (2.13–2.18) | 1.27× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m16s |
| 4 threads | 32m23s ± 4m33s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m20s ± 3m10s |
| the scanner alone (its serial part) | 11m47s ± 19.7s |
| the workers alone, over 4 | 22m20s ± 3m10s |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 2.93 ns a byte of output at 4 threads (CPU, summed over them), 1.69 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,597,062 | 13,035,595 | \`b6341d3af4a2a231…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
