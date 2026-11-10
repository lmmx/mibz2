# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`9cab9bb`](<https://github.com/lmmx/mibz2/commit/9cab9bb33fd5c24a11f2f74fe28035282d9bfffc>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42763321783>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.670 s | 313 | 9.2 | 2.36× (2.33–2.38) | 5.40× (5.35–5.44) | 5.60× (5.53–5.67) | 6.36× (6.28–6.44) | 5.56× (5.49–5.63) | 5.84× (5.78–5.91) | 5.50× (5.44–5.57) | 1.74× (1.72–1.76) | 1.93× (1.91–1.96) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.368 s | 569 | 9.8 | 2.41× (2.32–2.51) | 5.17× (5.02–5.33) | 5.89× (5.73–6.06) | 6.23× (6.06–6.41) | 5.42× (5.31–5.53) | 10.70× (10.47–10.93) | 10.08× (9.86–10.29) | 3.19× (3.12–3.26) | 3.54× (3.46–3.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.317 s | 662 | 16.5 | 2.39× (2.32–2.47) | 3.57× (3.43–3.70) | 4.28× (4.11–4.45) | 4.33× (4.13–4.53) | 4.26× (4.11–4.42) | 12.30× (11.88–12.74) | 11.59× (11.18–12.02) | 3.66× (3.53–3.80) | 4.07× (3.92–4.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 901 | 3.5 | 0.59× (0.58–0.60) | 3.39× (3.20–3.59) | 4.51× (4.37–4.65) | 3.65× (3.59–3.71) | 6.16× (6.05–6.26) | 2.74× (2.69–2.79) | 2.74× (2.69–2.79) | 0.98× (0.95–1.00) | 1.88× (1.78–1.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 868 | 3.6 | 0.54× (0.52–0.55) | 2.69× (2.55–2.83) | 5.62× (5.42–5.83) | 3.52× (3.40–3.65) | 5.82× (5.61–6.04) | 2.63× (2.53–2.73) | 2.63× (2.53–2.72) | 0.94× (0.91–0.97) | 1.80× (1.71–1.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 872 | 3.6 | 0.55× (0.54–0.56) | 2.69× (2.66–2.73) | 5.70× (5.62–5.77) | 3.57× (3.52–3.61) | 6.02× (5.92–6.13) | 2.66× (2.63–2.70) | 2.66× (2.63–2.70) | 0.95× (0.92–0.98) | 1.82× (1.72–1.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 552 | 5.5 | 2.56× (2.52–2.60) | 2.85× (2.81–2.90) | 3.57× (3.50–3.65) | 3.91× (3.85–3.97) | 4.25× (4.17–4.33) | 2.66× (2.61–2.71) | 2.17× (2.14–2.20) | 2.01× (1.98–2.03) | 2.84× (2.80–2.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 839 | 6.6 | 2.23× (2.11–2.35) | 2.79× (2.56–3.04) | 4.18× (3.82–4.57) | 3.47× (3.30–3.64) | 4.96× (4.76–5.15) | 4.05× (3.89–4.22) | 3.30× (3.19–3.42) | 3.05× (2.94–3.17) | 4.33× (4.17–4.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 747 | 13.5 | 1.85× (1.79–1.92) | 1.72× (1.62–1.84) | 2.79× (2.70–2.88) | 2.16× (2.11–2.22) | 3.95× (3.83–4.07) | 3.62× (3.53–3.71) | 2.95× (2.87–3.03) | 2.73× (2.66–2.80) | 3.86× (3.76–3.97) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.143 s | 56 | 51.9 | 1.38× (1.37–1.39) | 2.60× (2.57–2.62) | 2.33× (2.30–2.36) | 2.41× (2.39–2.42) | 2.70× (2.69–2.72) | 2.98× (2.96–3.00) | 2.97× (2.95–2.99) | 1.25× (1.24–1.26) | 1.41× (1.40–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 91 | 58.4 | 1.32× (1.28–1.36) | 2.55× (2.44–2.66) | 2.27× (2.22–2.31) | 3.02× (2.94–3.10) | 2.84× (2.74–2.95) | 4.88× (4.78–4.99) | 4.87× (4.76–4.98) | 2.05× (2.00–2.09) | 2.31× (2.25–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 75.6 | 1.37× (1.33–1.41) | 2.17× (2.10–2.25) | 2.13× (2.04–2.22) | 2.77× (2.59–2.96) | 2.90× (2.83–2.97) | 6.03× (5.91–6.15) | 6.02× (5.90–6.14) | 2.53× (2.47–2.59) | 2.86× (2.81–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.154 s | 62 | 46.5 | 1.53× (1.52–1.55) | 3.03× (2.98–3.07) | 2.58× (2.54–2.63) | 2.80× (2.76–2.84) | 3.18× (3.07–3.30) | 3.69× (3.65–3.73) | 3.90× (3.84–3.96) | 1.55× (1.53–1.57) | 1.79× (1.76–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.092 s | 104 | 52.5 | 1.53× (1.33–1.75) | 2.73× (2.63–2.83) | 2.47× (2.38–2.57) | 3.55× (3.41–3.69) | 3.06× (2.94–3.19) | 6.02× (5.80–6.24) | 6.37× (6.12–6.62) | 2.53× (2.44–2.62) | 2.92× (2.81–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.081 s | 119 | 71.1 | 1.44× (1.42–1.46) | 2.29× (2.23–2.35) | 2.31× (2.22–2.40) | 3.13× (3.02–3.24) | 2.86× (2.82–2.91) | 7.09× (6.97–7.20) | 7.49× (7.37–7.62) | 2.98× (2.92–3.05) | 3.44× (3.37–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 683 | 4.8 | 0.64× (0.63–0.66) | 2.86× (2.71–3.02) | 4.10× (3.96–4.24) | 3.20× (3.16–3.24) | 6.82× (6.67–6.97) | 2.36× (2.34–2.38) | 2.40× (2.38–2.43) | 0.97× (0.95–0.99) | 2.09× (2.07–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 637 | 5.2 | 0.53× (0.53–0.54) | 2.63× (2.59–2.66) | 4.89× (4.68–5.10) | 3.02× (2.95–3.10) | 6.30× (6.21–6.39) | 2.21× (2.18–2.24) | 2.25× (2.22–2.28) | 0.91× (0.89–0.92) | 1.96× (1.94–1.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 633 | 5.1 | 0.53× (0.53–0.54) | 2.62× (2.60–2.64) | 4.80× (4.58–5.04) | 3.01× (2.98–3.04) | 6.43× (6.26–6.61) | 2.20× (2.17–2.22) | 2.24× (2.21–2.26) | 0.90× (0.89–0.92) | 1.95× (1.93–1.97) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 0.093 s | 129 | 22.7 | 1.34× (1.32–1.36) | 2.91× (2.86–2.97) | 3.18× (3.10–3.26) | 3.36× (3.30–3.42) | 3.51× (3.43–3.59) | 3.46× (3.39–3.53) | 3.39× (3.32–3.47) | 1.11× (1.09–1.13) | 1.30× (1.26–1.33) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 0.059 s | 202 | 26.0 | 1.21× (1.10–1.32) | 2.63× (2.49–2.77) | 3.26× (3.19–3.32) | 3.66× (3.58–3.75) | 3.44× (3.37–3.51) | 5.48× (5.38–5.58) | 5.38× (5.29–5.47) | 1.76× (1.73–1.80) | 2.06× (2.00–2.11) | yes |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 0.049 s | 245 | 37.5 | 1.24× (1.21–1.27) | 2.00× (1.90–2.09) | 2.82× (2.72–2.92) | 3.09× (3.01–3.16) | 3.29× (3.17–3.41) | 6.58× (6.45–6.72) | 6.46× (6.33–6.58) | 2.11× (2.09–2.14) | 2.47× (2.39–2.55) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.35× (2.33–2.38) | 5.37× (5.32–5.42) | 5.57× (5.51–5.64) | 6.34× (6.27–6.42) | 5.54× (5.47–5.61) | 5.83× (5.76–5.89) | 5.49× (5.43–5.55) | 1.74× (1.72–1.76) | 1.92× (1.90–1.94) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.29× (2.25–2.33) | 5.20× (5.12–5.28) | 5.49× (5.38–5.60) | 6.28× (6.22–6.34) | 5.24× (5.21–5.27) | 5.47× (5.42–5.53) | 5.16× (5.10–5.21) | 1.63× (1.61–1.65) | 1.81× (1.79–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.14–2.28) | 3.64× (3.52–3.76) | 3.94× (3.76–4.13) | 4.42× (4.23–4.62) | 3.85× (3.70–4.02) | 3.24× (3.12–3.36) | 3.05× (2.94–3.17) | 0.97× (0.93–1.01) | 1.07× (1.03–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.64) | 3.02× (2.96–3.08) | 4.04× (3.92–4.16) | 3.42× (3.36–3.48) | 5.71× (5.62–5.81) | 2.59× (2.54–2.64) | 2.59× (2.55–2.64) | 0.98× (0.96–1.00) | 1.69× (1.64–1.73) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.60) | 2.74× (2.65–2.84) | 4.93× (4.76–5.11) | 3.27× (3.17–3.39) | 5.34× (5.16–5.52) | 2.45× (2.37–2.53) | 2.45× (2.37–2.54) | 0.93× (0.90–0.95) | 1.60× (1.54–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.59) | 3.11× (3.07–3.15) | 5.09× (5.03–5.16) | 3.39× (3.36–3.43) | 5.62× (5.53–5.71) | 2.54× (2.51–2.56) | 2.54× (2.51–2.57) | 0.96× (0.93–0.98) | 1.65× (1.61–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.48× (2.44–2.51) | 2.72× (2.69–2.75) | 3.25× (3.22–3.29) | 3.75× (3.70–3.80) | 4.07× (4.00–4.15) | 2.57× (2.52–2.61) | 2.10× (2.07–2.14) | 1.95× (1.93–1.97) | 2.53× (2.50–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.21–2.29) | 2.48× (2.41–2.55) | 3.28× (3.21–3.36) | 3.29× (3.23–3.34) | 3.54× (3.43–3.64) | 2.14× (2.09–2.18) | 1.75× (1.72–1.78) | 1.62× (1.60–1.65) | 2.11× (2.07–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.90–2.04) | 1.66× (1.62–1.71) | 2.16× (2.09–2.22) | 1.99× (1.93–2.05) | 2.02× (1.96–2.08) | 1.06× (1.03–1.10) | 0.87× (0.84–0.90) | 0.81× (0.78–0.84) | 1.05× (1.02–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.55× (2.53–2.57) | 2.27× (2.24–2.30) | 2.39× (2.37–2.40) | 2.68× (2.66–2.69) | 2.95× (2.93–2.97) | 2.94× (2.92–2.96) | 1.25× (1.24–1.25) | 1.36× (1.36–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.29–1.35) | 2.42× (2.37–2.47) | 2.09× (2.07–2.10) | 2.87× (2.84–2.89) | 2.50× (2.42–2.58) | 2.63× (2.61–2.65) | 2.63× (2.61–2.65) | 1.11× (1.10–1.12) | 1.22× (1.21–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.48–1.57) | 2.37× (2.31–2.43) | 1.96× (1.88–2.05) | 2.66× (2.60–2.71) | 2.54× (2.44–2.63) | 2.03× (2.00–2.06) | 2.02× (2.00–2.05) | 0.86× (0.85–0.87) | 0.94× (0.93–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.54) | 2.97× (2.93–3.01) | 2.53× (2.50–2.57) | 2.78× (2.73–2.82) | 3.15× (3.05–3.27) | 3.65× (3.61–3.69) | 3.86× (3.81–3.92) | 1.54× (1.52–1.57) | 1.75× (1.73–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.41–1.58) | 2.76× (2.73–2.78) | 2.34× (2.32–2.35) | 3.38× (3.34–3.42) | 2.79× (2.76–2.82) | 3.24× (3.22–3.27) | 3.43× (3.39–3.47) | 1.37× (1.36–1.39) | 1.56× (1.54–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.70× (1.65–1.75) | 2.65× (2.57–2.73) | 2.15× (2.02–2.29) | 3.02× (2.91–3.13) | 2.72× (2.65–2.78) | 2.40× (2.34–2.46) | 2.54× (2.47–2.60) | 1.01× (0.99–1.04) | 1.15× (1.12–1.18) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.53× (2.49–2.57) | 3.54× (3.48–3.59) | 2.93× (2.89–2.97) | 6.09× (5.96–6.23) | 2.19× (2.16–2.21) | 2.23× (2.20–2.25) | 0.97× (0.96–0.99) | 1.40× (1.38–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.59–0.60) | 2.30× (2.27–2.33) | 4.05× (4.00–4.10) | 2.79× (2.73–2.85) | 5.57× (5.50–5.65) | 2.02× (1.99–2.05) | 2.05× (2.03–2.08) | 0.90× (0.88–0.91) | 1.29× (1.28–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.58–0.59) | 2.57× (2.54–2.59) | 4.08× (4.03–4.13) | 2.87× (2.85–2.90) | 5.87× (5.72–6.03) | 2.07× (2.06–2.09) | 2.11× (2.09–2.13) | 0.92× (0.91–0.94) | 1.33× (1.31–1.34) |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 1.33× (1.32–1.35) | 2.80× (2.75–2.85) | 3.07× (3.01–3.13) | 3.31× (3.25–3.36) | 3.46× (3.38–3.53) | 3.40× (3.33–3.48) | 3.34× (3.27–3.41) | 1.11× (1.09–1.13) | 1.21× (1.20–1.22) |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 1.24× (1.23–1.26) | 2.64× (2.57–2.71) | 2.85× (2.83–2.88) | 3.36× (3.32–3.40) | 3.11× (3.08–3.14) | 2.99× (2.96–3.02) | 2.94× (2.90–2.97) | 0.97× (0.96–0.99) | 1.06× (1.04–1.08) |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 1.41× (1.37–1.45) | 2.23× (2.17–2.29) | 2.45× (2.40–2.50) | 2.82× (2.74–2.90) | 2.74× (2.66–2.81) | 2.07× (2.04–2.10) | 2.03× (2.00–2.06) | 0.67× (0.66–0.69) | 0.74× (0.72–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.31–0.33) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.11× (12.09–12.13) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.53× (0.46–0.63) | 0.79× (0.78–0.80) | 0.67× (0.65–0.68) | 6.34× (6.26–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.64× (0.55–0.74) | 0.70× (0.69–0.71) | 0.61× (0.60–0.62) | 4.10× (4.08–4.12) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.16× (0.16–0.16) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.30–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.44–0.45) | 1.72× (1.71–1.72) | 0.67× (0.67–0.68) | 8.08× (8.06–8.11) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.66× (0.59–0.73) | 1.63× (1.62–1.65) | 0.81× (0.80–0.82) | 5.91× (5.85–5.98) | 0.47× (0.47–0.48) | 0.50× (0.50–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.66) | 0.71× (0.66–0.76) | 1.16× (1.11–1.20) | 0.70× (0.68–0.72) | 3.44× (3.32–3.56) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.39× (0.36–0.43) | 0.92× (0.89–0.95) | 0.73× (0.71–0.76) | 3.16× (3.06–3.27) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.64) | 0.46× (0.45–0.46) | 0.87× (0.85–0.88) | 0.81× (0.79–0.82) | 2.52× (2.47–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.84× (0.83–0.84) | 5.25× (5.24–5.27) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.34× (0.31–0.36) | 0.84× (0.82–0.86) | 0.70× (0.67–0.73) | 2.88× (2.80–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.49× (0.47–0.52) | 0.82× (0.80–0.83) | 0.82× (0.79–0.84) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.21) | 1.09× (1.09–1.09) | 0.77× (0.77–0.78) | 5.43× (5.42–5.45) | 0.36× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.63× (1.63–1.63) | 0.77× (0.77–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.38 MB → 12 MB, 5.0×) | 1 | 18 MB | 0.48× (0.48–0.49) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.69× (0.69–0.70) | 6.74× (6.72–6.76) | 0.41× (0.41–0.41) | 0.44× (0.43–0.44) | 0.66× (0.66–0.67) | 0.33× (0.33–0.33) |
| `text` (2.38 MB → 12 MB, 5.0×) | 2 | 30 MB | 0.53× (0.52–0.55) | 0.50× (0.46–0.54) | 1.28× (1.25–1.31) | 0.80× (0.78–0.82) | 3.98× (3.88–4.07) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) | 0.39× (0.38–0.40) | 0.19× (0.19–0.20) |
| `text` (2.38 MB → 12 MB, 5.0×) | 4 | 49 MB | 0.56× (0.54–0.58) | 0.50× (0.45–0.54) | 1.02× (0.99–1.06) | 0.81× (0.78–0.84) | 2.62× (2.54–2.70) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 313 | 3.21 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 569 | 3.43 | 1.06× (1.05–1.08) | 92% | 1.03× (1.01–1.05) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 662 | 5.76 | 1.80× (1.73–1.87) | 53% | 1.22× (1.20–1.24) | 1.69× (1.65–1.73) |

**Best:** mibz2 at 4 threads (662 MB/s); lbzip2 at 4 threads (185 MB/s); pbz2 at 4 threads (274 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.23, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.98 | 0.02 | 0.08 | 0.317 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2858 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.6% |
| MTF + RLE2 | 1.58 | 18.1% |
| T-vector build | 1.49 | 17.0% |
| walk (the one each build uses) | 4.11 | 47.0% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.9% |
| **total** | 8.74 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.505 | 1.372 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.357 | 2.680 | (in its total) | 1.97× (1.97–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.050 | 0.052 | (in its total) | 1.06× (1.02–1.10) |  |
| inverse BWT, total | 1.911 | 4.104 | 2.498 | 2.15× (2.14–2.15) | 1.31× (1.30–1.32) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h38m ± 11m32s |
| 4 threads | 46m42s ± 6m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 36m20s ± 5m25s |
| the scanner alone (its serial part) | 16m18s ± 26.3s |
| the workers alone, over 4 | 36m20s ± 5m25s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 4.76 ns a byte of output at 4 threads (CPU, summed over them), 2.69 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,384,894 | 12,023,507 | \`4c10ae1761b31e94…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2858.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
