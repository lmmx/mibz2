# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-690/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`5f12912`](<https://github.com/lmmx/mibz2/commit/5f12912419a28f7b4bb9f341676e077b37dcc01d>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44908854602>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.508 s | 413 | 7.9 | 3.03× (3.00–3.05) | 6.94× (6.84–7.04) | 7.18× (7.04–7.32) | 7.71× (7.67–7.76) | 7.04× (6.96–7.12) | 7.54× (7.49–7.58) | 6.89× (6.82–6.95) | 2.21× (2.20–2.22) | 2.59× (2.57–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.293 s | 716 | 8.7 | 2.96× (2.83–3.10) | 6.37× (6.08–6.68) | 7.31× (6.97–7.67) | 7.13× (6.85–7.42) | 6.47× (6.21–6.74) | 12.82× (12.39–13.25) | 11.71× (11.26–12.17) | 3.76× (3.64–3.89) | 4.40× (4.24–4.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.243 s | 861 | 13.6 | 2.94× (2.88–3.01) | 4.53× (4.39–4.68) | 5.68× (5.47–5.89) | 5.09× (4.99–5.19) | 5.09× (4.97–5.20) | 15.60× (15.28–15.93) | 14.26× (13.98–14.53) | 4.58× (4.47–4.69) | 5.36× (5.24–5.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 971 | 3.7 | 0.61× (0.60–0.63) | 3.74× (3.58–3.92) | 4.41× (4.30–4.51) | 3.55× (3.49–3.61) | 6.16× (6.05–6.27) | 2.98× (2.92–3.04) | 2.66× (2.61–2.71) | 1.01× (0.98–1.03) | 1.49× (1.46–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 943 | 3.8 | 0.60× (0.57–0.64) | 2.88× (2.81–2.96) | 6.09× (5.92–6.27) | 3.48× (3.41–3.56) | 6.01× (5.90–6.12) | 2.88× (2.82–2.95) | 2.57× (2.52–2.62) | 0.98× (0.95–1.00) | 1.44× (1.40–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 917 | 3.9 | 0.57× (0.56–0.59) | 2.80× (2.71–2.90) | 5.85× (5.60–6.11) | 3.37× (3.27–3.46) | 5.83× (5.66–6.02) | 2.79× (2.70–2.88) | 2.49× (2.41–2.56) | 0.94× (0.92–0.97) | 1.39× (1.35–1.44) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 579 | 5.9 | 2.69× (2.65–2.73) | 3.02× (3.00–3.04) | 3.93× (3.81–4.05) | 3.83× (3.81–3.85) | 4.30× (4.21–4.40) | 2.58× (2.55–2.61) | 2.19× (2.15–2.23) | 2.02× (2.01–2.03) | 3.00× (2.98–3.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 838 | 7.6 | 2.11× (1.88–2.36) | 2.72× (2.46–3.01) | 3.88× (3.51–4.28) | 3.11× (2.84–3.41) | 4.56× (4.17–5.00) | 3.56× (3.26–3.90) | 3.02× (2.75–3.31) | 2.78× (2.54–3.05) | 4.14× (3.80–4.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 760 | 14.8 | 1.81× (1.77–1.86) | 2.00× (1.91–2.09) | 2.98× (2.85–3.11) | 2.19× (2.06–2.33) | 3.92× (3.84–4.01) | 3.38× (3.32–3.44) | 2.86× (2.78–2.94) | 2.64× (2.58–2.70) | 3.93× (3.85–4.00) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.131 s | 61 | 53.9 | 1.38× (1.36–1.40) | 2.73× (2.68–2.78) | 2.45× (2.40–2.49) | 2.47× (2.43–2.51) | 2.88× (2.84–2.93) | 3.16× (3.11–3.22) | 3.12× (3.06–3.18) | 1.22× (1.19–1.25) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 97 | 61.9 | 1.32× (1.23–1.41) | 2.60× (2.37–2.85) | 2.35× (2.28–2.43) | 3.04× (2.95–3.13) | 2.92× (2.85–2.99) | 5.02× (4.92–5.13) | 4.96× (4.82–5.09) | 1.93× (1.89–1.97) | 2.24× (2.17–2.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 82.5 | 1.32× (1.26–1.39) | 2.11× (2.03–2.19) | 2.04× (1.95–2.15) | 2.61× (2.51–2.72) | 2.83× (2.73–2.93) | 5.93× (5.65–6.23) | 5.85× (5.57–6.14) | 2.28× (2.19–2.38) | 2.64× (2.50–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.4 | 1.50× (1.49–1.51) | 3.11× (3.05–3.17) | 2.60× (2.55–2.64) | 2.64× (2.60–2.67) | 3.09× (3.05–3.13) | 3.65× (3.60–3.69) | 3.86× (3.82–3.89) | 1.43× (1.39–1.46) | 1.71× (1.68–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 57.3 | 1.34× (1.30–1.39) | 2.75× (2.70–2.80) | 2.47× (2.30–2.66) | 3.38× (3.26–3.51) | 3.06× (2.91–3.22) | 5.78× (5.60–5.97) | 6.11× (5.92–6.32) | 2.26× (2.17–2.35) | 2.72× (2.63–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 122 | 82.4 | 1.45× (1.41–1.48) | 2.38× (2.04–2.78) | 2.31× (2.05–2.61) | 3.03× (2.76–3.33) | 2.84× (2.68–3.01) | 6.70× (6.56–6.84) | 7.09× (6.96–7.22) | 2.62× (2.54–2.70) | 3.15× (3.05–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 5.2 | 0.66× (0.65–0.68) | 3.24× (3.04–3.47) | 4.30× (4.18–4.42) | 3.03× (2.97–3.10) | 6.69× (6.56–6.83) | 2.50× (2.45–2.54) | 2.27× (2.23–2.32) | 0.98× (0.96–1.01) | 1.46× (1.42–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 654 | 5.8 | 0.59× (0.57–0.61) | 2.71× (2.63–2.79) | 4.86× (4.70–5.03) | 2.86× (2.79–2.94) | 6.26× (6.14–6.37) | 2.32× (2.26–2.37) | 2.11× (2.06–2.16) | 0.91× (0.89–0.93) | 1.36× (1.32–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 5.6 | 0.57× (0.55–0.59) | 2.65× (2.53–2.78) | 4.75× (4.67–4.84) | 2.81× (2.70–2.92) | 6.05× (5.83–6.27) | 2.26× (2.19–2.33) | 2.06× (1.99–2.13) | 0.89× (0.87–0.91) | 1.32× (1.26–1.39) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.073 s | 183 | 18.2 | 1.98× (1.92–2.04) | 4.06× (3.91–4.21) | 4.42× (4.29–4.56) | 4.43× (4.26–4.60) | 4.67× (4.50–4.85) | 5.20× (4.99–5.43) | 4.73× (4.55–4.91) | 1.51× (1.46–1.56) | 1.84× (1.79–1.89) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.053 s | 254 | 22.9 | 1.56× (1.50–1.63) | 3.24× (3.09–3.41) | 3.81× (3.62–4.02) | 4.03× (3.86–4.21) | 4.10× (3.93–4.28) | 7.22× (6.93–7.52) | 6.56× (6.31–6.81) | 2.09× (2.01–2.19) | 2.55× (2.42–2.69) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.046 s | 290 | 36.2 | 1.56× (1.52–1.60) | 2.44× (2.33–2.55) | 3.23× (3.03–3.43) | 3.19× (3.15–3.24) | 3.67× (3.60–3.75) | 8.21× (7.95–8.48) | 7.46× (7.27–7.65) | 2.38× (2.31–2.46) | 2.90× (2.80–3.01) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.02× (2.99–3.04) | 6.91× (6.81–7.01) | 7.14× (7.00–7.29) | 7.68× (7.64–7.73) | 7.01× (6.93–7.08) | 7.51× (7.47–7.55) | 6.86× (6.79–6.93) | 2.21× (2.19–2.22) | 2.57× (2.54–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.87× (2.84–2.90) | 6.47× (6.31–6.64) | 6.80× (6.66–6.94) | 7.40× (7.27–7.54) | 6.43× (6.33–6.53) | 6.76× (6.67–6.84) | 6.17× (6.08–6.27) | 1.98× (1.96–2.01) | 2.31× (2.28–2.35) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.90× (2.83–2.98) | 4.91× (4.76–5.06) | 5.30× (5.11–5.48) | 5.53× (5.41–5.66) | 4.86× (4.75–4.98) | 4.36× (4.28–4.45) | 3.99× (3.90–4.07) | 1.28× (1.26–1.30) | 1.49× (1.47–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.67) | 3.26× (3.20–3.32) | 4.02× (3.95–4.09) | 3.32× (3.26–3.38) | 5.68× (5.58–5.79) | 2.80× (2.74–2.85) | 2.50× (2.46–2.55) | 1.00× (0.99–1.02) | 1.06× (1.03–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.65× (0.62–0.69) | 3.04× (2.98–3.10) | 5.32× (5.21–5.42) | 3.24× (3.17–3.30) | 5.48× (5.38–5.58) | 2.67× (2.61–2.73) | 2.39× (2.35–2.44) | 0.96× (0.94–0.98) | 1.01× (0.98–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.31× (3.22–3.40) | 5.23× (5.09–5.38) | 3.19× (3.11–3.28) | 5.41× (5.24–5.58) | 2.64× (2.56–2.72) | 2.36× (2.30–2.43) | 0.95× (0.92–0.97) | 1.00× (0.95–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.59× (2.55–2.63) | 2.81× (2.79–2.82) | 3.60× (3.48–3.72) | 3.67× (3.65–3.69) | 4.12× (4.03–4.21) | 2.50× (2.46–2.53) | 2.12× (2.08–2.16) | 1.96× (1.95–1.97) | 2.74× (2.71–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.22× (2.16–2.28) | 2.46× (2.39–2.54) | 3.07× (2.99–3.15) | 3.08× (3.00–3.16) | 3.43× (3.34–3.53) | 1.98× (1.91–2.04) | 1.68× (1.62–1.74) | 1.55× (1.52–1.59) | 2.17× (2.10–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.92–2.05) | 1.72× (1.68–1.76) | 2.19× (2.13–2.25) | 1.98× (1.86–2.12) | 2.10× (2.03–2.18) | 1.01× (0.98–1.04) | 0.86× (0.83–0.90) | 0.80× (0.77–0.82) | 1.11× (1.08–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.35–1.39) | 2.66× (2.62–2.71) | 2.38× (2.34–2.42) | 2.45× (2.41–2.49) | 2.85× (2.81–2.90) | 3.13× (3.07–3.19) | 3.09× (3.03–3.14) | 1.21× (1.19–1.24) | 1.37× (1.35–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.49× (2.46–2.53) | 2.17× (2.15–2.20) | 2.91× (2.87–2.96) | 2.58× (2.55–2.60) | 2.74× (2.70–2.78) | 2.70× (2.66–2.75) | 1.06× (1.05–1.07) | 1.20× (1.18–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.47–1.58) | 2.35× (2.31–2.38) | 1.91× (1.86–1.97) | 2.66× (2.58–2.75) | 2.49× (2.46–2.52) | 2.05× (2.00–2.11) | 2.02× (1.97–2.08) | 0.80× (0.78–0.81) | 0.90× (0.87–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.49–1.51) | 3.04× (2.99–3.09) | 2.54× (2.50–2.58) | 2.61× (2.58–2.65) | 3.06× (3.02–3.10) | 3.61× (3.56–3.65) | 3.82× (3.78–3.85) | 1.42× (1.39–1.45) | 1.67× (1.64–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.40) | 2.77× (2.71–2.84) | 2.34× (2.20–2.48) | 3.20× (3.11–3.30) | 2.78× (2.62–2.96) | 3.12× (3.07–3.17) | 3.30× (3.25–3.35) | 1.23× (1.19–1.26) | 1.44× (1.41–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.51–1.65) | 2.65× (2.29–3.08) | 2.04× (1.75–2.39) | 2.89× (2.57–3.25) | 2.57× (2.37–2.79) | 2.18× (2.14–2.22) | 2.31× (2.27–2.34) | 0.86× (0.84–0.88) | 1.01× (0.98–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.73) | 2.68× (2.63–2.74) | 3.53× (3.45–3.60) | 2.79× (2.73–2.85) | 5.99× (5.87–6.12) | 2.32× (2.27–2.37) | 2.12× (2.08–2.17) | 0.99× (0.97–1.02) | 0.98× (0.92–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.65× (0.63–0.67) | 2.45× (2.40–2.50) | 4.15× (4.07–4.23) | 2.66× (2.60–2.72) | 5.52× (5.43–5.61) | 2.11× (2.07–2.16) | 1.93× (1.89–1.97) | 0.90× (0.88–0.92) | 0.89× (0.84–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.60–0.64) | 2.66× (2.57–2.74) | 4.12× (4.02–4.23) | 2.69× (2.59–2.79) | 5.49× (5.30–5.68) | 2.11× (2.05–2.18) | 1.94× (1.87–2.00) | 0.90× (0.88–0.93) | 0.89× (0.83–0.96) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.95× (1.90–2.01) | 3.91× (3.77–4.06) | 4.26× (4.13–4.40) | 4.34× (4.18–4.50) | 4.57× (4.40–4.75) | 5.09× (4.88–5.30) | 4.62× (4.45–4.80) | 1.50× (1.45–1.54) | 1.77× (1.71–1.84) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.64× (1.59–1.69) | 3.28× (3.21–3.36) | 3.56× (3.43–3.70) | 3.95× (3.80–4.11) | 3.70× (3.54–3.86) | 4.00× (3.86–4.15) | 3.63× (3.52–3.75) | 1.18× (1.16–1.20) | 1.39× (1.36–1.43) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.72× (1.67–1.76) | 2.58× (2.47–2.69) | 2.80× (2.68–2.94) | 3.10× (3.05–3.14) | 2.85× (2.79–2.91) | 2.56× (2.50–2.63) | 2.33× (2.27–2.39) | 0.75× (0.73–0.79) | 0.89× (0.86–0.92) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.27× (0.25–0.28) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.95× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.33–0.34) | 0.48× (0.44–0.53) | 0.69× (0.67–0.71) | 0.57× (0.56–0.58) | 5.49× (5.37–5.61) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.31–0.34) | 0.55× (0.48–0.62) | 0.58× (0.56–0.61) | 0.51× (0.50–0.53) | 3.39× (3.24–3.54) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.92–7.96) | 0.64× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.73× (0.64–0.82) | 1.62× (1.59–1.64) | 0.81× (0.79–0.83) | 5.84× (5.77–5.92) | 0.47× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.63–0.65) | 0.71× (0.66–0.75) | 1.16× (1.14–1.18) | 0.71× (0.70–0.73) | 3.41× (3.38–3.45) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.16× (5.14–5.18) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.39× (0.35–0.43) | 0.91× (0.87–0.94) | 0.71× (0.68–0.75) | 3.13× (3.02–3.24) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.81× (0.79–0.83) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.16× (5.15–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.50) | 0.36× (0.32–0.40) | 0.86× (0.82–0.89) | 0.70× (0.67–0.73) | 2.93× (2.82–3.03) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.47× (0.44–0.49) | 0.80× (0.78–0.81) | 0.80× (0.76–0.83) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.27× (5.25–5.28) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.66) | 6.34× (6.32–6.37) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.42× (0.40–0.45) | 1.05× (1.03–1.07) | 0.68× (0.66–0.70) | 3.33× (3.30–3.37) | 0.20× (0.20–0.21) | 0.22× (0.22–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.51) | 0.48× (0.45–0.52) | 0.90× (0.88–0.93) | 0.73× (0.69–0.77) | 2.29× (2.24–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 413 | 2.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 716 | 2.69 | 1.11× (1.10–1.12) | 85% | 1.04× (1.02–1.07) | 1.06× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 861 | 4.20 | 1.72× (1.69–1.76) | 52% | 1.22× (1.19–1.26) | 1.65× (1.61–1.70) |

**Best:** mibz2 at 4 threads (861 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (290 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.68 | 0.02 | 0.09 | 0.243 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3245 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.4% |
| MTF + RLE2 | 1.53 | 16.7% |
| T-vector build | 1.55 | 16.9% |
| walk (the one each build uses) | 4.39 | 47.9% |
| of which: segment assembly | 0.17 | 1.9% |
| RLE1 | 0.29 | 3.1% |
| CRC | 0.26 | 2.8% |
| **total** | 9.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.303 | (in its total) | 2.65× (2.65–2.66) |  |
| walk | 1.280 | 2.668 | (in its total) | 2.08× (2.07–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.067 | (in its total) | 1.33× (1.25–1.42) |  |
| inverse BWT, total | 1.820 | 4.038 | 3.443 | 2.22× (2.20–2.23) | 1.88× (1.86–1.91) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m25s |
| 4 threads | 35m44s ± 5m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m29s ± 3m31s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 24m29s ± 3m31s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.21 ns a byte of output at 4 threads (CPU, summed over them), 1.97 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,147 | 13,382,859 | \`f7d4e4f1aa08d971…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3245.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
