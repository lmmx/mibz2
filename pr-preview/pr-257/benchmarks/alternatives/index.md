# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-257/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`dcc22ba`](<https://github.com/lmmx/mibz2/commit/dcc22ba1b0b5e7c3758c25e5f7c1bb4c23e87edb>) (2026-11-08).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42474322785>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.563 s | 372 | 9.4 | 2.27× (2.25–2.30) | 5.20× (5.15–5.25) | 5.38× (5.32–5.44) | 6.12× (6.06–6.19) | 5.37× (5.32–5.42) | 6.07× (6.01–6.13) | 5.35× (5.30–5.40) | 1.69× (1.67–1.70) | 1.89× (1.87–1.90) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.317 s | 662 | 10.3 | 2.37× (2.27–2.48) | 4.83× (4.69–4.98) | 5.63× (5.41–5.84) | 5.85× (5.63–6.08) | 5.13× (4.96–5.31) | 10.73× (10.32–11.17) | 9.45× (9.08–9.83) | 2.98× (2.87–3.10) | 3.34× (3.20–3.48) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.271 s | 775 | 17.1 | 2.31× (2.23–2.40) | 3.41× (3.31–3.52) | 4.21× (4.07–4.34) | 4.11× (4.02–4.21) | 4.06× (3.96–4.17) | 12.64× (12.33–12.95) | 11.12× (10.86–11.40) | 3.51× (3.44–3.59) | 3.93× (3.83–4.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,047 | 3.6 | 0.56× (0.55–0.58) | 3.19× (3.13–3.25) | 4.38× (4.15–4.62) | 3.52× (3.46–3.59) | 6.09× (5.98–6.20) | 2.63× (2.58–2.68) | 2.61× (2.55–2.66) | 0.98× (0.96–1.01) | 2.12× (2.09–2.16) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,023 | 3.8 | 0.52× (0.51–0.53) | 2.74× (2.58–2.90) | 5.61× (5.42–5.81) | 3.50× (3.46–3.53) | 5.94× (5.83–6.05) | 2.60× (2.57–2.64) | 2.58× (2.54–2.62) | 0.97× (0.95–1.00) | 2.10× (2.07–2.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,000 | 3.8 | 0.51× (0.49–0.54) | 2.62× (2.48–2.76) | 5.40× (5.15–5.67) | 3.41× (3.29–3.52) | 5.85× (5.63–6.07) | 2.53× (2.43–2.63) | 2.50× (2.42–2.59) | 0.94× (0.91–0.97) | 2.04× (1.98–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 652 | 5.7 | 2.51× (2.45–2.58) | 2.94× (2.78–3.10) | 3.43× (3.31–3.54) | 3.80× (3.73–3.86) | 4.21× (4.12–4.29) | 3.18× (3.10–3.25) | 2.11× (2.06–2.15) | 1.95× (1.92–1.98) | 2.72× (2.64–2.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 985 | 7.0 | 2.09× (1.87–2.33) | 2.47× (2.25–2.71) | 3.88× (3.45–4.35) | 3.25× (2.90–3.64) | 4.62× (4.13–5.18) | 4.53× (4.05–5.06) | 3.00× (2.68–3.36) | 2.77× (2.50–3.08) | 3.88× (3.46–4.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 877 | 13.7 | 1.78× (1.72–1.83) | 1.75× (1.62–1.88) | 2.80× (2.70–2.90) | 2.10× (2.00–2.20) | 3.89× (3.77–4.01) | 4.24× (4.09–4.40) | 2.81× (2.72–2.91) | 2.60× (2.52–2.68) | 3.63× (3.47–3.80) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.117 s | 68 | 51.6 | 1.38× (1.37–1.40) | 2.64× (2.60–2.69) | 2.44× (2.37–2.51) | 2.56× (2.49–2.62) | 2.85× (2.77–2.93) | 3.18× (3.12–3.24) | 3.10× (3.04–3.16) | 1.26× (1.24–1.28) | 1.43× (1.41–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 110 | 59.3 | 1.33× (1.21–1.45) | 2.49× (2.41–2.57) | 2.34× (2.28–2.41) | 3.14× (3.06–3.22) | 2.89× (2.82–2.97) | 5.09× (5.00–5.19) | 4.96× (4.84–5.09) | 2.02× (1.96–2.07) | 2.29× (2.21–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 76.8 | 1.40× (1.37–1.43) | 2.23× (2.15–2.31) | 2.19× (2.11–2.28) | 2.72× (2.64–2.81) | 3.02× (2.96–3.07) | 6.39× (6.27–6.51) | 6.23× (6.09–6.36) | 2.53× (2.47–2.59) | 2.88× (2.78–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 46.1 | 1.51× (1.45–1.56) | 3.02× (2.92–3.13) | 2.62× (2.52–2.72) | 2.75× (2.65–2.86) | 3.10× (2.98–3.22) | 3.93× (3.78–4.08) | 3.94× (3.77–4.12) | 1.50× (1.43–1.57) | 1.76× (1.68–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.078 s | 123 | 53.4 | 1.37× (1.33–1.41) | 2.79× (2.67–2.91) | 2.56× (2.50–2.63) | 3.51× (3.39–3.64) | 3.07× (2.99–3.17) | 6.38× (6.20–6.56) | 6.41× (6.28–6.53) | 2.43× (2.38–2.49) | 2.86× (2.79–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 142 | 76.0 | 1.42× (1.38–1.46) | 2.25× (2.22–2.28) | 2.25× (2.17–2.34) | 3.04× (2.92–3.17) | 2.88× (2.78–2.99) | 7.40× (7.19–7.61) | 7.43× (7.24–7.62) | 2.82× (2.76–2.89) | 3.32× (3.22–3.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 784 | 5.1 | 0.62× (0.61–0.63) | 3.24× (3.17–3.31) | 4.02× (3.93–4.11) | 3.07× (2.99–3.16) | 6.82× (6.59–7.05) | 2.33× (2.28–2.38) | 2.31× (2.24–2.38) | 0.96× (0.94–0.99) | 1.63× (1.60–1.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 760 | 5.4 | 0.52× (0.51–0.53) | 2.34× (2.28–2.39) | 4.86× (4.67–5.05) | 2.95× (2.88–3.02) | 6.41× (6.24–6.58) | 2.22× (2.17–2.28) | 2.20× (2.13–2.28) | 0.92× (0.89–0.95) | 1.56× (1.52–1.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 749 | 5.2 | 0.51× (0.50–0.52) | 2.32× (2.27–2.38) | 4.67× (4.46–4.90) | 2.95× (2.89–3.01) | 6.33× (6.19–6.48) | 2.20× (2.14–2.27) | 2.18× (2.11–2.26) | 0.91× (0.88–0.94) | 1.54× (1.50–1.58) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 0.076 s | 167 | 21.4 | 1.37× (1.35–1.39) | 2.94× (2.88–3.00) | 3.30× (3.23–3.37) | 3.44× (3.38–3.50) | 3.63× (3.57–3.69) | 3.79× (3.72–3.85) | 3.48× (3.42–3.54) | 1.12× (1.10–1.15) | 1.30× (1.25–1.34) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 0.049 s | 258 | 25.2 | 1.18× (1.07–1.30) | 2.51× (2.28–2.77) | 3.02× (2.77–3.29) | 3.33× (3.05–3.63) | 3.48× (3.17–3.82) | 5.74× (5.25–6.27) | 5.27× (4.82–5.77) | 1.70× (1.55–1.87) | 1.96× (1.81–2.13) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 0.039 s | 323 | 36.1 | 1.35× (1.26–1.45) | 2.12× (2.08–2.17) | 2.86× (2.77–2.95) | 3.07× (3.03–3.11) | 3.49× (3.45–3.54) | 7.29× (7.15–7.43) | 6.70× (6.58–6.83) | 2.16× (2.14–2.19) | 2.50× (2.40–2.60) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.27× (2.25–2.29) | 5.18× (5.13–5.23) | 5.36× (5.30–5.41) | 6.11× (6.04–6.18) | 5.35× (5.30–5.40) | 6.05× (5.99–6.12) | 5.33× (5.28–5.38) | 1.69× (1.67–1.70) | 1.87× (1.86–1.89) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.19× (2.16–2.22) | 4.90× (4.83–4.96) | 5.15× (5.07–5.23) | 5.95× (5.86–6.04) | 5.01× (4.96–5.06) | 5.57× (5.50–5.65) | 4.91× (4.84–4.97) | 1.55× (1.53–1.57) | 1.72× (1.70–1.75) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.13× (2.09–2.17) | 3.52× (3.45–3.60) | 3.82× (3.73–3.91) | 4.25× (4.18–4.32) | 3.67× (3.62–3.72) | 3.34× (3.27–3.42) | 2.94× (2.88–3.01) | 0.93× (0.91–0.95) | 1.03× (1.01–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.89× (2.85–2.92) | 3.89× (3.78–3.99) | 3.31× (3.26–3.37) | 5.66× (5.56–5.76) | 2.49× (2.45–2.54) | 2.47× (2.43–2.52) | 0.99× (0.96–1.01) | 1.62× (1.59–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.70× (2.64–2.75) | 4.91× (4.83–5.00) | 3.26× (3.22–3.29) | 5.44× (5.35–5.54) | 2.43× (2.39–2.46) | 2.41× (2.37–2.44) | 0.96× (0.94–0.98) | 1.58× (1.54–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.58) | 2.95× (2.86–3.03) | 4.86× (4.70–5.03) | 3.25× (3.15–3.35) | 5.46× (5.28–5.65) | 2.42× (2.33–2.50) | 2.39× (2.32–2.47) | 0.96× (0.93–0.98) | 1.57× (1.51–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.43× (2.36–2.49) | 2.64× (2.56–2.72) | 3.17× (3.10–3.24) | 3.64× (3.58–3.69) | 4.02× (3.95–4.10) | 3.05× (2.98–3.12) | 2.04× (2.00–2.09) | 1.89× (1.86–1.92) | 2.49× (2.42–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.17× (2.05–2.30) | 2.32× (2.22–2.44) | 3.07× (2.92–3.22) | 3.10× (2.95–3.26) | 3.35× (3.19–3.52) | 2.43× (2.31–2.56) | 1.63× (1.55–1.71) | 1.51× (1.45–1.57) | 1.98× (1.89–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.86–2.05) | 1.59× (1.54–1.65) | 2.17× (2.10–2.24) | 2.02× (1.93–2.12) | 2.05× (1.97–2.13) | 1.28× (1.25–1.32) | 0.86× (0.83–0.89) | 0.80× (0.77–0.83) | 1.05× (1.00–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.39) | 2.58× (2.54–2.62) | 2.38× (2.32–2.43) | 2.53× (2.47–2.60) | 2.82× (2.75–2.90) | 3.14× (3.08–3.21) | 3.07× (3.01–3.13) | 1.26× (1.24–1.28) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.30) | 2.38× (2.35–2.42) | 2.13× (2.10–2.17) | 2.91× (2.87–2.95) | 2.51× (2.47–2.55) | 2.74× (2.70–2.78) | 2.67× (2.62–2.72) | 1.09× (1.08–1.11) | 1.19× (1.17–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.40–1.54) | 2.31× (2.24–2.38) | 1.97× (1.92–2.02) | 2.68× (2.62–2.74) | 2.54× (2.49–2.60) | 2.12× (2.08–2.16) | 2.07× (2.03–2.10) | 0.85× (0.83–0.87) | 0.92× (0.90–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.44–1.56) | 2.96× (2.86–3.07) | 2.55× (2.46–2.65) | 2.73× (2.63–2.83) | 3.07× (2.95–3.19) | 3.88× (3.74–4.03) | 3.90× (3.74–4.07) | 1.49× (1.42–1.56) | 1.71× (1.64–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.37–1.42) | 2.76× (2.69–2.83) | 2.37× (2.32–2.42) | 3.31× (3.25–3.38) | 2.76× (2.69–2.82) | 3.40× (3.35–3.46) | 3.42× (3.37–3.46) | 1.31× (1.28–1.33) | 1.50× (1.47–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.62) | 2.47× (2.43–2.52) | 1.98× (1.92–2.03) | 2.79× (2.73–2.85) | 2.57× (2.50–2.63) | 2.39× (2.35–2.43) | 2.40× (2.36–2.44) | 0.92× (0.91–0.93) | 1.06× (1.05–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.44× (2.38–2.50) | 3.38× (3.30–3.47) | 2.81× (2.75–2.88) | 6.09× (5.90–6.28) | 2.16× (2.12–2.20) | 2.15× (2.08–2.21) | 0.97× (0.95–0.99) | 1.36× (1.33–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.24× (2.19–2.30) | 4.06× (3.99–4.13) | 2.72× (2.66–2.78) | 5.65× (5.52–5.80) | 2.03× (1.98–2.08) | 2.01× (1.96–2.08) | 0.91× (0.88–0.93) | 1.28× (1.25–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.56–0.57) | 2.50× (2.45–2.56) | 4.06× (3.95–4.17) | 2.79× (2.75–2.84) | 5.76× (5.64–5.88) | 2.07× (2.02–2.13) | 2.06× (2.00–2.12) | 0.93× (0.90–0.96) | 1.31× (1.27–1.34) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 1.36× (1.34–1.38) | 2.84× (2.79–2.89) | 3.19× (3.13–3.24) | 3.38× (3.32–3.44) | 3.57× (3.51–3.62) | 3.72× (3.65–3.79) | 3.42× (3.36–3.48) | 1.12× (1.10–1.14) | 1.22× (1.20–1.24) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 1.23× (1.21–1.26) | 2.53× (2.47–2.58) | 2.82× (2.75–2.89) | 3.28× (3.23–3.33) | 3.07× (3.01–3.14) | 3.14× (3.09–3.20) | 2.89× (2.84–2.94) | 0.95× (0.93–0.96) | 1.03× (1.01–1.05) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 1.42× (1.37–1.46) | 2.16× (2.13–2.19) | 2.45× (2.34–2.56) | 2.74× (2.68–2.81) | 2.64× (2.59–2.70) | 2.19× (2.15–2.24) | 2.02× (1.97–2.07) | 0.66× (0.65–0.67) | 0.72× (0.70–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.36× (0.35–0.38) | 1.08× (1.08–1.08) | 0.86× (0.86–0.87) | 12.18× (12.16–12.20) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.49× (0.46–0.52) | 0.80× (0.79–0.81) | 0.66× (0.65–0.67) | 6.34× (6.26–6.41) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.62× (0.52–0.74) | 0.70× (0.69–0.70) | 0.62× (0.61–0.62) | 4.10× (4.09–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.17× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.32× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.17) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.43× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.50× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.67) | 8.12× (8.11–8.14) | 0.64× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.65× (0.63–0.68) | 0.65× (0.59–0.71) | 1.60× (1.55–1.65) | 0.79× (0.76–0.81) | 5.79× (5.60–5.99) | 0.46× (0.44–0.47) | 0.49× (0.47–0.50) | 0.53× (0.51–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.62× (0.59–0.66) | 0.66× (0.59–0.73) | 1.15× (1.09–1.21) | 0.74× (0.71–0.78) | 3.36× (3.19–3.55) | 0.26× (0.24–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.48–0.53) | 0.40× (0.36–0.44) | 0.93× (0.89–0.97) | 0.72× (0.69–0.76) | 3.21× (3.09–3.34) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.46) | 0.87× (0.85–0.89) | 0.81× (0.79–0.83) | 2.52× (2.47–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.48) | 0.39× (0.37–0.41) | 0.86× (0.84–0.89) | 0.72× (0.70–0.75) | 2.96× (2.89–3.04) | 0.18× (0.18–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.49× (0.46–0.52) | 0.80× (0.79–0.81) | 0.80× (0.77–0.83) | 2.29× (2.26–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.76–0.77) | 5.40× (5.39–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.40–0.41) | 1.50× (1.50–1.50) | 0.66× (0.66–0.67) | 6.77× (6.76–6.79) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 35 MB | 0.46× (0.44–0.47) | 0.39× (0.36–0.42) | 1.11× (1.06–1.16) | 0.69× (0.67–0.72) | 3.48× (3.36–3.60) | 0.21× (0.20–0.22) | 0.22× (0.21–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.18) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 51 MB | 0.53× (0.51–0.55) | 0.48× (0.46–0.51) | 0.99× (0.95–1.04) | 0.79× (0.75–0.83) | 2.53× (2.43–2.64) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 372 | 2.69 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 662 | 2.95 | 1.09× (1.07–1.10) | 88% | 1.03× (1.01–1.04) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 775 | 4.89 | 1.81× (1.78–1.85) | 52% | 1.23× (1.21–1.26) | 1.70× (1.66–1.74) |

**Best:** mibz2 at 4 threads (775 MB/s); lbzip2 at 4 threads (226 MB/s); pbz2 at 4 threads (337 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.72, mibz2 4t 1.06, lbzip2 1t 0.06, lbzip2 2t 0.21, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.83 | 0.02 | 0.08 | 0.271 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3489 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.09 | 12.4% |
| MTF + RLE2 | 1.55 | 17.5% |
| T-vector build | 1.73 | 19.6% |
| walk (the one each build uses) | 4.01 | 45.3% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.25 | 2.8% |
| **total** | 8.84 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.476 | 1.108 | (in its total) | 2.32× (2.29–2.36) |  |
| walk | 1.120 | 2.161 | (in its total) | 1.93× (1.90–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.045 | (in its total) | 0.95× (0.91–0.99) |  |
| inverse BWT, total | 1.641 | 3.315 | 2.001 | 2.01× (1.98–2.05) | 1.21× (1.19–1.23) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h22m ± 9m44s |
| 4 threads | 39m42s ± 5m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m40s ± 4m22s |
| the scanner alone (its serial part) | 13m23s ± 26.3s |
| the workers alone, over 4 | 30m40s ± 4m22s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 4.02 ns a byte of output at 4 threads (CPU, summed over them), 2.27 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,073,075 | 12,677,537 | \`e4ea56361d34c699…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3489.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
