# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-841/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`9621b7a`](<https://github.com/lmmx/mibz2/commit/9621b7ad1bad30857d1216432e0117aa8d2c5a32>) (2026-11-24).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45498344108>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.389 s | 539 | 4.8 | 3.13× (3.11–3.14) | 7.16× (7.13–7.19) | 7.40× (7.35–7.45) | 8.43× (8.39–8.46) | 7.44× (7.40–7.48) | 7.76× (7.72–7.80) | 7.32× (7.28–7.36) | 2.32× (2.31–2.33) | 2.59× (2.58–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 959 | 5.2 | 3.24× (3.15–3.33) | 6.66× (6.51–6.82) | 7.64× (7.46–7.83) | 8.01× (7.82–8.20) | 7.04× (6.89–7.20) | 13.78× (13.50–14.07) | 13.01× (12.76–13.27) | 4.12× (4.04–4.21) | 4.61× (4.51–4.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.179 s | 1,172 | 8.1 | 3.15× (3.05–3.25) | 4.71× (4.57–4.86) | 5.78× (5.65–5.91) | 5.73× (5.56–5.91) | 5.65× (5.50–5.81) | 16.75× (16.33–17.18) | 15.81× (15.40–16.23) | 5.01× (4.88–5.14) | 5.60× (5.45–5.75) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,117 | 2.5 | 0.57× (0.57–0.58) | 3.43× (3.40–3.46) | 4.58× (4.55–4.62) | 3.57× (3.55–3.60) | 6.24× (6.10–6.37) | 2.66× (2.64–2.68) | 2.66× (2.64–2.68) | 0.95× (0.94–0.97) | 1.93× (1.73–2.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,044 | 2.8 | 0.51× (0.49–0.52) | 2.70× (2.63–2.77) | 5.41× (5.28–5.54) | 3.38× (3.30–3.46) | 5.71× (5.57–5.86) | 2.51× (2.45–2.57) | 2.50× (2.44–2.57) | 0.90× (0.88–0.92) | 1.81× (1.63–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,039 | 2.8 | 0.50× (0.49–0.51) | 2.66× (2.62–2.71) | 5.33× (5.24–5.41) | 3.33× (3.28–3.38) | 5.74× (5.62–5.85) | 2.46× (2.43–2.50) | 2.46× (2.42–2.50) | 0.88× (0.87–0.90) | 1.78× (1.60–1.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 751 | 3.7 | 2.75× (2.73–2.76) | 3.12× (3.11–3.14) | 3.91× (3.79–4.03) | 4.17× (4.15–4.19) | 4.63× (4.59–4.67) | 2.82× (2.80–2.84) | 2.31× (2.30–2.32) | 2.11× (2.11–2.12) | 3.11× (3.10–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,233 | 4.3 | 2.50× (2.45–2.55) | 3.26× (3.11–3.42) | 4.90× (4.66–5.16) | 4.15× (3.84–4.50) | 5.73× (5.62–5.84) | 4.59× (4.50–4.68) | 3.76× (3.69–3.83) | 3.44× (3.38–3.50) | 5.07× (4.97–5.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,130 | 8.5 | 2.08× (1.95–2.21) | 1.99× (1.83–2.18) | 3.37× (3.10–3.66) | 2.54× (2.30–2.80) | 4.63× (4.31–4.98) | 4.08× (3.83–4.34) | 3.34× (3.14–3.55) | 3.06× (2.87–3.26) | 4.50× (4.23–4.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 76 | 34.9 | 1.43× (1.41–1.44) | 2.74× (2.70–2.77) | 2.50× (2.48–2.52) | 2.58× (2.56–2.60) | 2.94× (2.92–2.97) | 3.10× (3.07–3.12) | 3.10× (3.07–3.13) | 1.31× (1.27–1.34) | 1.43× (1.42–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.064 s | 126 | 38.4 | 1.37× (1.33–1.42) | 2.69× (2.52–2.87) | 2.56× (2.49–2.64) | 3.31× (3.19–3.44) | 3.10× (3.02–3.19) | 5.17× (5.02–5.31) | 5.17× (5.04–5.31) | 2.18× (2.09–2.28) | 2.39× (2.32–2.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.051 s | 156 | 51.5 | 1.43× (1.40–1.46) | 2.26× (2.18–2.33) | 2.28× (2.21–2.36) | 2.92× (2.80–3.05) | 3.15× (3.07–3.22) | 6.40× (6.27–6.53) | 6.41× (6.27–6.54) | 2.70× (2.61–2.79) | 2.96× (2.90–3.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.114 s | 84 | 31.4 | 1.57× (1.57–1.58) | 3.20× (3.17–3.22) | 2.77× (2.74–2.80) | 2.87× (2.87–2.88) | 3.26× (3.24–3.28) | 3.84× (3.82–3.85) | 4.04× (4.02–4.05) | 1.56× (1.55–1.56) | 1.86× (1.86–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.068 s | 142 | 35.5 | 1.47× (1.42–1.52) | 2.98× (2.77–3.20) | 2.69× (2.63–2.76) | 3.73× (3.64–3.83) | 3.32× (3.23–3.40) | 6.41× (6.24–6.57) | 6.74× (6.56–6.92) | 2.60× (2.53–2.67) | 3.11× (3.03–3.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.059 s | 163 | 50.2 | 1.51× (1.47–1.55) | 2.40× (2.31–2.48) | 2.34× (2.30–2.38) | 3.30× (3.21–3.39) | 3.09× (3.03–3.14) | 7.47× (7.37–7.57) | 7.85× (7.74–7.97) | 3.03× (2.99–3.07) | 3.62× (3.57–3.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 3.5 | 0.63× (0.62–0.64) | 3.48× (3.41–3.55) | 4.39× (4.30–4.48) | 3.10× (3.04–3.16) | 6.81× (6.74–6.88) | 2.29× (2.25–2.34) | 2.32× (2.28–2.37) | 0.95× (0.93–0.97) | 1.75× (1.72–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 4.0 | 0.49× (0.48–0.50) | 2.37× (2.33–2.43) | 4.78× (4.68–4.87) | 2.83× (2.77–2.89) | 6.43× (6.04–6.85) | 2.07× (2.03–2.12) | 2.10× (2.06–2.14) | 0.86× (0.84–0.88) | 1.59× (1.55–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 763 | 4.0 | 0.49× (0.49–0.50) | 2.42× (2.27–2.58) | 4.71× (4.63–4.79) | 2.81× (2.76–2.86) | 6.11× (6.00–6.22) | 2.04× (2.01–2.08) | 2.07× (2.04–2.10) | 0.85× (0.83–0.87) | 1.56× (1.54–1.59) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.058 s | 242 | 11.0 | 1.94× (1.92–1.95) | 4.19× (4.13–4.26) | 4.59× (4.57–4.60) | 4.83× (4.82–4.85) | 5.03× (5.00–5.06) | 5.01× (4.97–5.05) | 4.86× (4.85–4.87) | 1.59× (1.58–1.59) | 1.78× (1.74–1.82) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.039 s | 360 | 13.6 | 1.52× (1.45–1.59) | 3.46× (3.27–3.66) | 4.19× (4.01–4.37) | 4.85× (4.56–5.17) | 4.49× (4.31–4.67) | 7.36× (7.01–7.72) | 7.14× (6.83–7.45) | 2.33× (2.24–2.42) | 2.62× (2.49–2.74) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.032 s | 430 | 20.5 | 1.68× (1.62–1.75) | 2.71× (2.49–2.96) | 3.71× (3.56–3.86) | 3.96× (3.83–4.10) | 4.33× (4.19–4.48) | 8.79× (8.49–9.09) | 8.52× (8.25–8.81) | 2.78× (2.69–2.87) | 3.12× (3.01–3.25) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.12× (3.10–3.13) | 7.12× (7.09–7.15) | 7.36× (7.31–7.41) | 8.39× (8.35–8.43) | 7.41× (7.37–7.45) | 7.73× (7.68–7.77) | 7.29× (7.26–7.33) | 2.31× (2.30–2.32) | 2.57× (2.56–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.00× (2.98–3.01) | 6.81× (6.76–6.85) | 7.06× (7.03–7.09) | 8.26× (8.23–8.29) | 6.97× (6.94–7.00) | 7.25× (7.22–7.27) | 6.84× (6.82–6.86) | 2.17× (2.16–2.18) | 2.41× (2.40–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.92–3.12) | 5.15× (4.97–5.33) | 5.47× (5.28–5.66) | 6.27× (6.09–6.46) | 5.41× (5.24–5.59) | 4.70× (4.54–4.87) | 4.44× (4.29–4.59) | 1.41× (1.36–1.46) | 1.57× (1.51–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.91× (2.89–2.93) | 3.89× (3.83–3.96) | 3.35× (3.32–3.37) | 5.78× (5.65–5.90) | 2.51× (2.49–2.53) | 2.51× (2.50–2.53) | 0.96× (0.95–0.97) | 1.64× (1.63–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.53–0.56) | 2.57× (2.52–2.63) | 4.62× (4.54–4.70) | 3.10× (3.03–3.17) | 5.17× (5.05–5.29) | 2.31× (2.25–2.36) | 2.30× (2.25–2.36) | 0.88× (0.86–0.90) | 1.51× (1.47–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 2.79× (2.75–2.84) | 4.59× (4.52–4.65) | 3.11× (3.06–3.15) | 5.24× (5.14–5.35) | 2.30× (2.27–2.34) | 2.30× (2.27–2.34) | 0.88× (0.86–0.89) | 1.51× (1.48–1.53) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.63–2.65) | 2.88× (2.86–2.89) | 3.46× (3.34–3.59) | 3.97× (3.95–3.99) | 4.41× (4.37–4.45) | 2.71× (2.69–2.73) | 2.23× (2.22–2.24) | 2.05× (2.04–2.06) | 2.68× (2.67–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.46× (2.43–2.49) | 2.72× (2.67–2.77) | 3.58× (3.54–3.63) | 3.60× (3.57–3.63) | 3.93× (3.86–4.00) | 2.34× (2.31–2.36) | 1.92× (1.90–1.94) | 1.77× (1.75–1.79) | 2.31× (2.29–2.34) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.18× (2.10–2.25) | 1.85× (1.79–1.91) | 2.42× (2.35–2.49) | 2.18× (2.12–2.24) | 2.31× (2.26–2.37) | 1.18× (1.15–1.21) | 0.97× (0.95–1.00) | 0.89× (0.87–0.91) | 1.17× (1.14–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.41–1.43) | 2.66× (2.64–2.68) | 2.44× (2.42–2.47) | 2.56× (2.54–2.58) | 2.91× (2.89–2.94) | 3.06× (3.04–3.08) | 3.07× (3.04–3.09) | 1.30× (1.26–1.34) | 1.41× (1.40–1.42) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.35× (1.34–1.36) | 2.55× (2.53–2.58) | 2.27× (2.25–2.28) | 3.08× (3.05–3.11) | 2.70× (2.68–2.73) | 2.78× (2.76–2.80) | 2.79× (2.77–2.80) | 1.18× (1.15–1.22) | 1.28× (1.28–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.48–1.57) | 2.37× (2.33–2.42) | 2.00× (1.96–2.04) | 2.73× (2.65–2.81) | 2.62× (2.56–2.68) | 2.08× (2.04–2.12) | 2.08× (2.04–2.13) | 0.88× (0.85–0.92) | 0.96× (0.94–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.56× (1.56–1.57) | 3.10× (3.09–3.11) | 2.69× (2.66–2.71) | 2.85× (2.84–2.85) | 3.23× (3.21–3.24) | 3.79× (3.78–3.81) | 3.99× (3.98–4.00) | 1.55× (1.54–1.55) | 1.79× (1.78–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.42–1.47) | 2.89× (2.83–2.96) | 2.46× (2.42–2.50) | 3.47× (3.41–3.53) | 2.93× (2.88–2.98) | 3.36× (3.31–3.42) | 3.54× (3.47–3.60) | 1.37× (1.35–1.40) | 1.58× (1.56–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.56–1.68) | 2.54× (2.47–2.60) | 2.05× (2.01–2.09) | 2.98× (2.92–3.03) | 2.70× (2.65–2.74) | 2.38× (2.34–2.42) | 2.50× (2.46–2.55) | 0.97× (0.96–0.99) | 1.12× (1.10–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.69) | 2.46× (2.42–2.51) | 3.38× (3.32–3.45) | 2.84× (2.79–2.89) | 6.08× (6.02–6.13) | 2.13× (2.09–2.17) | 2.15× (2.12–2.19) | 0.96× (0.94–0.97) | 1.38× (1.35–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.11× (2.08–2.15) | 3.71× (3.64–3.78) | 2.56× (2.51–2.61) | 5.56× (5.23–5.91) | 1.86× (1.83–1.89) | 1.88× (1.85–1.92) | 0.84× (0.82–0.85) | 1.21× (1.19–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.45× (2.14–2.80) | 3.73× (3.68–3.78) | 2.60× (2.56–2.65) | 5.43× (5.34–5.53) | 1.88× (1.85–1.92) | 1.91× (1.88–1.94) | 0.85× (0.83–0.86) | 1.23× (1.20–1.25) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.91× (1.89–1.93) | 3.99× (3.98–4.00) | 4.38× (4.36–4.40) | 4.72× (4.71–4.74) | 4.92× (4.89–4.95) | 4.89× (4.86–4.93) | 4.75× (4.74–4.76) | 1.57× (1.56–1.58) | 1.72× (1.71–1.73) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.62× (1.59–1.64) | 3.39× (3.32–3.46) | 3.71× (3.66–3.76) | 4.43× (4.37–4.50) | 4.05× (4.00–4.11) | 3.98× (3.91–4.06) | 3.87× (3.81–3.92) | 1.28× (1.26–1.29) | 1.40× (1.37–1.42) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.72× (1.65–1.80) | 2.71× (2.64–2.78) | 2.97× (2.92–3.02) | 3.51× (3.45–3.57) | 3.29× (3.23–3.35) | 2.63× (2.59–2.66) | 2.55× (2.51–2.58) | 0.84× (0.83–0.85) | 0.92× (0.91–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.96–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.36) | 0.51× (0.42–0.62) | 0.71× (0.68–0.74) | 0.60× (0.58–0.62) | 5.69× (5.49–5.89) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.37× (0.36–0.39) | 0.54× (0.50–0.59) | 0.66× (0.63–0.68) | 0.58× (0.56–0.61) | 3.89× (3.72–4.07) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.90× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.91–1.98) | 0.92× (0.90–0.94) | 4.82× (4.74–4.90) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 1.01× (1.00–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.65× (0.65–0.66) | 7.96× (7.93–7.99) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.68–0.72) | 0.71× (0.66–0.78) | 1.68× (1.63–1.72) | 0.84× (0.81–0.87) | 6.10× (5.93–6.27) | 0.50× (0.48–0.51) | 0.52× (0.50–0.54) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.68× (0.65–0.71) | 0.76× (0.70–0.82) | 1.24× (1.19–1.29) | 0.77× (0.74–0.80) | 3.63× (3.49–3.77) | 0.28× (0.27–0.30) | 0.30× (0.29–0.31) | 0.32× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.14× (5.12–5.16) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.39× (0.35–0.43) | 0.90× (0.87–0.93) | 0.72× (0.69–0.75) | 3.11× (3.02–3.21) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.65) | 0.46× (0.45–0.46) | 0.87× (0.86–0.88) | 0.81× (0.78–0.83) | 2.53× (2.50–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.44–0.49) | 0.35× (0.32–0.39) | 0.82× (0.79–0.86) | 0.70× (0.66–0.74) | 2.82× (2.70–2.95) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.46× (0.44–0.48) | 0.81× (0.79–0.83) | 0.82× (0.78–0.86) | 2.34× (2.28–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.32× (5.30–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.90) | 6.21× (6.20–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.40) | 0.67× (0.67–0.67) | 6.40× (6.39–6.41) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.47× (0.46–0.49) | 0.41× (0.36–0.47) | 1.09× (1.05–1.12) | 0.72× (0.70–0.74) | 3.53× (3.43–3.63) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.17× (0.16–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 54 MB | 0.52× (0.51–0.54) | 0.46× (0.42–0.51) | 0.95× (0.94–0.97) | 0.78× (0.76–0.81) | 2.42× (2.37–2.48) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 539 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 959 | 1.99 | 1.07× (1.06–1.07) | 89% | 1.02× (1.01–1.03) | 1.03× (1.02–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,172 | 3.10 | 1.64× (1.59–1.70) | 54% | 1.19× (1.18–1.19) | 1.59× (1.57–1.61) |

**Best:** mibz2 at 4 threads (1,172 MB/s); lbzip2 at 4 threads (247 MB/s); pbz2 at 4 threads (369 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.08, lbzip2 1t 0.06, lbzip2 2t 0.27, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.49 | 0.02 | 0.08 | 0.179 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.62 | 10.5% |
| MTF + RLE2 | 1.09 | 18.4% |
| T-vector build | 1.04 | 17.6% |
| walk (the one each build uses) | 2.86 | 48.3% |
| of which: segment assembly | 0.10 | 1.6% |
| RLE1 | 0.13 | 2.2% |
| CRC | 0.18 | 3.0% |
| **total** | 5.93 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.063 | (in its total) | 2.72× (2.70–2.74) |  |
| walk | 1.049 | 2.082 | (in its total) | 1.99× (1.97–2.01) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.041 | (in its total) | 1.07× (1.05–1.08) |  |
| inverse BWT, total | 1.478 | 3.186 | 1.900 | 2.16× (2.14–2.18) | 1.29× (1.28–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m42s ± 6m45s |
| 4 threads | 26m17s ± 3m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m52s ± 2m35s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 17m52s ± 2m35s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 2.34 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
