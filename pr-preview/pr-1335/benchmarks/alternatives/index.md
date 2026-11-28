# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1335/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`469bd81`](<https://github.com/lmmx/mibz2/commit/469bd810452454aa630a7e6bf0c5315e28f3ba5e>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46125062914>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.473 s | 443 | 7.4 | 3.16× (3.13–3.19) | 7.23× (7.16–7.29) | 7.36× (7.30–7.43) | 8.17× (8.10–8.24) | 7.20× (7.09–7.32) | 7.91× (7.81–8.01) | 7.41× (7.35–7.47) | 2.32× (2.30–2.35) | 2.71× (2.69–2.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.285 s | 736 | 7.9 | 3.01× (2.81–3.22) | 6.36× (6.08–6.64) | 7.12× (6.77–7.49) | 7.26× (6.86–7.68) | 6.41× (6.09–6.75) | 13.09× (12.45–13.77) | 12.27× (11.63–12.95) | 3.85× (3.65–4.05) | 4.49× (4.26–4.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.216 s | 972 | 12.4 | 3.21× (3.17–3.25) | 4.95× (4.87–5.03) | 6.04× (5.99–6.10) | 5.63× (5.56–5.69) | 5.59× (5.50–5.68) | 17.44× (17.33–17.56) | 16.35× (16.19–16.51) | 5.13× (5.09–5.16) | 5.98× (5.90–6.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,026 | 3.5 | 0.62× (0.61–0.64) | 3.76× (3.61–3.91) | 4.63× (4.58–4.68) | 3.68× (3.63–3.72) | 6.34× (6.27–6.40) | 3.10× (3.06–3.14) | 2.76× (2.73–2.80) | 1.02× (1.01–1.03) | 1.57× (1.56–1.59) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 955 | 3.9 | 0.55× (0.54–0.56) | 2.92× (2.88–2.96) | 5.77× (5.69–5.86) | 3.45× (3.40–3.50) | 5.87× (5.79–5.95) | 2.88× (2.83–2.92) | 2.56× (2.53–2.60) | 0.95× (0.93–0.96) | 1.46× (1.44–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 937 | 3.9 | 0.55× (0.54–0.55) | 2.89× (2.87–2.92) | 5.70× (5.66–5.74) | 3.40× (3.38–3.43) | 5.82× (5.76–5.89) | 2.84× (2.82–2.86) | 2.53× (2.51–2.55) | 0.93× (0.92–0.95) | 1.44× (1.43–1.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 619 | 5.6 | 2.76× (2.74–2.78) | 3.22× (3.20–3.23) | 3.94× (3.79–4.10) | 4.16× (4.14–4.18) | 4.35× (4.33–4.37) | 2.70× (2.69–2.71) | 2.22× (2.21–2.22) | 2.11× (2.10–2.12) | 3.20× (3.19–3.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 916 | 6.7 | 2.27× (2.08–2.47) | 2.85× (2.63–3.09) | 4.18× (3.86–4.51) | 3.58× (3.27–3.91) | 4.80× (4.45–5.19) | 3.90× (3.61–4.21) | 3.20× (2.96–3.46) | 3.05× (2.82–3.29) | 4.62× (4.27–5.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 875 | 13.5 | 2.04× (1.94–2.14) | 2.13× (1.98–2.29) | 3.21× (3.10–3.32) | 2.40× (2.33–2.48) | 4.32× (4.12–4.53) | 3.81× (3.67–3.96) | 3.13× (3.01–3.26) | 2.98× (2.87–3.11) | 4.52× (4.35–4.70) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.118 s | 68 | 49.1 | 1.49× (1.49–1.50) | 2.93× (2.92–2.94) | 2.65× (2.64–2.66) | 2.73× (2.72–2.73) | 3.13× (3.12–3.15) | 3.45× (3.43–3.47) | 3.52× (3.48–3.56) | 1.35× (1.34–1.35) | 1.54× (1.54–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 113 | 54.3 | 1.44× (1.42–1.46) | 2.93× (2.74–3.13) | 2.70× (2.63–2.77) | 3.50× (3.43–3.57) | 3.32× (3.27–3.36) | 5.78× (5.69–5.86) | 5.89× (5.78–6.01) | 2.26× (2.23–2.29) | 2.58× (2.55–2.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 133 | 75.5 | 1.48× (1.45–1.51) | 2.36× (2.28–2.44) | 2.32× (2.24–2.39) | 2.93× (2.85–3.02) | 3.18× (3.11–3.24) | 6.82× (6.70–6.94) | 6.96× (6.81–7.12) | 2.67× (2.62–2.72) | 3.05× (2.99–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.133 s | 72 | 45.8 | 1.61× (1.55–1.67) | 3.32× (3.28–3.36) | 2.75× (2.73–2.78) | 2.89× (2.87–2.92) | 3.36× (3.27–3.44) | 3.94× (3.92–3.97) | 4.25× (4.23–4.28) | 1.51× (1.50–1.53) | 1.83× (1.82–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.080 s | 121 | 51.2 | 1.51× (1.36–1.67) | 2.99× (2.82–3.17) | 2.68× (2.58–2.77) | 3.73× (3.53–3.95) | 3.27× (3.09–3.45) | 6.42× (6.09–6.77) | 6.93× (6.57–7.31) | 2.47× (2.34–2.60) | 2.99× (2.84–3.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 140 | 72.6 | 1.54× (1.51–1.56) | 2.53× (2.40–2.66) | 2.41× (2.35–2.47) | 3.33× (3.23–3.43) | 3.06× (2.99–3.13) | 7.62× (7.43–7.82) | 8.22× (8.03–8.42) | 2.93× (2.86–3.00) | 3.54× (3.46–3.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 775 | 4.8 | 0.68× (0.67–0.69) | 3.18× (3.15–3.21) | 4.67× (4.63–4.72) | 3.21× (3.18–3.24) | 7.01× (6.94–7.08) | 2.64× (2.62–2.67) | 2.43× (2.41–2.45) | 1.00× (0.99–1.02) | 1.60× (1.59–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 692 | 5.6 | 0.54× (0.54–0.55) | 2.86× (2.81–2.90) | 4.93× (4.85–5.00) | 2.93× (2.87–2.99) | 6.31× (6.18–6.43) | 2.37× (2.33–2.41) | 2.18× (2.14–2.21) | 0.90× (0.89–0.91) | 1.44× (1.41–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.6 | 0.54× (0.51–0.56) | 2.76× (2.62–2.91) | 4.74× (4.50–5.00) | 2.83× (2.68–2.98) | 6.09× (5.81–6.38) | 2.28× (2.17–2.41) | 2.10× (1.99–2.21) | 0.87× (0.82–0.91) | 1.38× (1.32–1.46) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.062 s | 244 | 13.8 | 2.30× (2.28–2.32) | 4.45× (4.42–4.47) | 4.90× (4.87–4.93) | 4.97× (4.93–5.00) | 5.07× (5.02–5.12) | 5.55× (5.52–5.58) | 5.23× (5.18–5.27) | 1.71× (1.70–1.72) | 2.13× (2.12–2.15) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.043 s | 346 | 18.0 | 1.80× (1.66–1.94) | 3.64× (3.42–3.88) | 4.35× (4.14–4.58) | 4.79× (4.56–5.04) | 4.46× (4.33–4.60) | 7.85× (7.63–8.09) | 7.39× (7.18–7.62) | 2.42× (2.35–2.50) | 3.02× (2.93–3.11) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.039 s | 385 | 27.6 | 1.70× (1.64–1.76) | 2.71× (2.62–2.80) | 3.65× (3.44–3.89) | 3.78× (3.67–3.90) | 4.00× (3.87–4.14) | 8.85× (8.57–9.14) | 8.33× (8.05–8.62) | 2.73× (2.65–2.82) | 3.40× (3.30–3.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.15× (3.13–3.18) | 7.19× (7.13–7.26) | 7.33× (7.26–7.40) | 8.14× (8.07–8.21) | 7.18× (7.07–7.29) | 7.88× (7.78–7.98) | 7.39× (7.33–7.44) | 2.32× (2.30–2.34) | 2.69× (2.67–2.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.02× (2.90–3.15) | 6.78× (6.51–7.06) | 7.05× (6.77–7.35) | 7.92× (7.60–8.26) | 6.70× (6.42–6.98) | 7.26× (6.96–7.57) | 6.80× (6.52–7.10) | 2.13× (2.05–2.23) | 2.48× (2.38–2.58) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.12× (3.05–3.19) | 5.19× (5.10–5.28) | 5.56× (5.47–5.65) | 5.94× (5.85–6.04) | 5.19× (5.10–5.27) | 4.75× (4.66–4.83) | 4.45× (4.39–4.51) | 1.40× (1.37–1.42) | 1.62× (1.59–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.64–0.68) | 3.41× (3.36–3.45) | 4.21× (4.16–4.27) | 3.46× (3.42–3.50) | 5.90× (5.85–5.96) | 2.93× (2.90–2.97) | 2.62× (2.58–2.65) | 1.02× (1.01–1.03) | 1.11× (1.08–1.14) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.58–0.59) | 2.93× (2.89–2.97) | 5.10× (5.04–5.17) | 3.15× (3.11–3.20) | 5.28× (5.22–5.35) | 2.62× (2.59–2.66) | 2.34× (2.32–2.37) | 0.91× (0.90–0.92) | 0.99× (0.97–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.19× (3.17–3.21) | 5.13× (5.06–5.21) | 3.14× (3.12–3.17) | 5.27× (5.21–5.33) | 2.62× (2.60–2.64) | 2.34× (2.32–2.35) | 0.91× (0.90–0.92) | 0.99× (0.97–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.64–2.69) | 2.94× (2.93–2.96) | 3.72× (3.62–3.82) | 3.99× (3.97–4.01) | 4.17× (4.15–4.20) | 2.61× (2.60–2.62) | 2.15× (2.14–2.16) | 2.05× (2.04–2.06) | 2.83× (2.82–2.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.40× (2.37–2.43) | 2.69× (2.63–2.75) | 3.25× (3.22–3.29) | 3.48× (3.44–3.52) | 3.60× (3.55–3.64) | 2.16× (2.14–2.18) | 1.78× (1.76–1.79) | 1.69× (1.68–1.71) | 2.34× (2.32–2.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.06× (1.96–2.16) | 1.83× (1.74–1.92) | 2.27× (2.18–2.37) | 2.15× (2.06–2.25) | 2.19× (2.10–2.27) | 1.08× (1.04–1.12) | 0.89× (0.86–0.92) | 0.85× (0.81–0.88) | 1.17× (1.13–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.48–1.49) | 2.89× (2.88–2.90) | 2.58× (2.57–2.59) | 2.70× (2.69–2.71) | 3.10× (3.08–3.11) | 3.41× (3.39–3.43) | 3.47× (3.45–3.48) | 1.34× (1.34–1.35) | 1.50× (1.49–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.39–1.42) | 2.73× (2.69–2.77) | 2.41× (2.38–2.43) | 3.26× (3.21–3.31) | 2.85× (2.81–2.88) | 3.07× (3.04–3.10) | 3.12× (3.09–3.15) | 1.21× (1.19–1.22) | 1.35× (1.33–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.59× (1.55–1.64) | 2.48× (2.45–2.50) | 2.01× (1.98–2.03) | 2.78× (2.74–2.82) | 2.63× (2.60–2.66) | 2.22× (2.19–2.24) | 2.26× (2.23–2.28) | 0.87× (0.86–0.88) | 0.97× (0.96–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.60× (1.55–1.66) | 3.24× (3.20–3.29) | 2.70× (2.67–2.73) | 2.87× (2.84–2.89) | 3.33× (3.24–3.41) | 3.91× (3.89–3.93) | 4.21× (4.19–4.24) | 1.51× (1.50–1.52) | 1.78× (1.76–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.47–1.48) | 3.03× (2.99–3.07) | 2.51× (2.50–2.52) | 3.54× (3.50–3.58) | 2.98× (2.97–3.00) | 3.49× (3.47–3.51) | 3.76× (3.75–3.78) | 1.35× (1.34–1.35) | 1.59× (1.58–1.60) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.75× (1.70–1.80) | 2.75× (2.70–2.79) | 2.12× (2.08–2.17) | 3.11× (3.04–3.18) | 2.76× (2.70–2.82) | 2.49× (2.44–2.54) | 2.68× (2.62–2.74) | 0.96× (0.94–0.98) | 1.13× (1.11–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.74) | 2.85× (2.81–2.90) | 3.70× (3.66–3.75) | 2.96× (2.93–2.99) | 6.33× (6.27–6.38) | 2.47× (2.44–2.50) | 2.27× (2.24–2.30) | 1.01× (0.99–1.03) | 1.00× (0.97–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.41× (2.37–2.45) | 4.08× (4.02–4.13) | 2.64× (2.60–2.68) | 5.45× (5.35–5.55) | 2.11× (2.08–2.14) | 1.94× (1.91–1.97) | 0.86× (0.85–0.88) | 0.85× (0.84–0.86) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.59) | 2.57× (2.46–2.69) | 4.00× (3.82–4.19) | 2.61× (2.49–2.73) | 5.35× (5.14–5.58) | 2.07× (1.98–2.17) | 1.90× (1.81–1.99) | 0.85× (0.81–0.89) | 0.83× (0.80–0.87) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.26× (2.24–2.29) | 4.32× (4.30–4.35) | 4.71× (4.68–4.75) | 4.87× (4.83–4.90) | 4.97× (4.92–5.01) | 5.43× (5.40–5.47) | 5.12× (5.07–5.16) | 1.70× (1.68–1.71) | 2.06× (2.05–2.08) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.82× (1.78–1.86) | 3.46× (3.40–3.51) | 3.80× (3.72–3.89) | 4.25× (4.17–4.32) | 3.92× (3.85–3.99) | 4.16× (4.09–4.24) | 3.92× (3.86–3.98) | 1.30× (1.28–1.32) | 1.58× (1.55–1.61) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.85× (1.79–1.92) | 2.74× (2.67–2.81) | 3.04× (2.95–3.12) | 3.28× (3.21–3.36) | 3.09× (3.02–3.17) | 2.72× (2.65–2.79) | 2.56× (2.50–2.62) | 0.85× (0.83–0.87) | 1.03× (1.00–1.06) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.31× (0.29–0.32) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.02× (11.00–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.58× (0.50–0.66) | 0.72× (0.69–0.76) | 0.60× (0.58–0.62) | 5.75× (5.53–5.98) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.36× (0.34–0.38) | 0.66× (0.57–0.76) | 0.64× (0.61–0.66) | 0.56× (0.53–0.59) | 3.72× (3.56–3.89) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.81–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.66–0.66) | 7.95× (7.93–7.97) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.70× (0.67–0.73) | 0.75× (0.71–0.79) | 1.67× (1.61–1.73) | 0.83× (0.80–0.86) | 6.04× (5.82–6.27) | 0.49× (0.48–0.51) | 0.51× (0.49–0.54) | 0.56× (0.54–0.59) | 0.28× (0.26–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.65–0.73) | 0.80× (0.75–0.84) | 1.23× (1.17–1.30) | 0.76× (0.72–0.80) | 3.64× (3.47–3.81) | 0.29× (0.27–0.30) | 0.30× (0.28–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.17× (5.15–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.38× (0.35–0.42) | 0.91× (0.89–0.93) | 0.72× (0.70–0.74) | 3.14× (3.06–3.21) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.68) | 0.46× (0.45–0.47) | 0.88× (0.86–0.91) | 0.83× (0.81–0.85) | 2.57× (2.50–2.64) | 0.15× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.24–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.21× (5.21–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.49) | 0.37× (0.34–0.40) | 0.86× (0.84–0.88) | 0.70× (0.67–0.73) | 2.95× (2.88–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.47–0.52) | 0.80× (0.79–0.81) | 0.80× (0.77–0.84) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.06–1.07) | 0.76× (0.75–0.77) | 5.32× (5.29–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.89× (0.89–0.90) | 6.19× (6.16–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.29) | 0.23× (0.23–0.24) | 1.86× (1.85–1.87) | 0.89× (0.89–0.90) | 6.17× (6.14–6.20) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.38–1.40) | 0.66× (0.65–0.66) | 6.37× (6.34–6.40) | 0.38× (0.38–0.39) | 0.41× (0.40–0.41) | 0.63× (0.63–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 36 MB | 0.46× (0.44–0.47) | 0.41× (0.39–0.44) | 1.05× (1.02–1.08) | 0.69× (0.66–0.71) | 3.42× (3.33–3.52) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.34× (0.33–0.35) | 0.16× (0.16–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.45× (0.42–0.47) | 0.90× (0.88–0.91) | 0.75× (0.72–0.77) | 2.31× (2.27–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 443 | 2.27 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 736 | 2.43 | 1.09× (1.04–1.14) | 83% | 1.02× (1.02–1.03) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 972 | 3.79 | 1.66× (1.63–1.69) | 55% | 1.20× (1.19–1.20) | 1.64× (1.63–1.65) |

**Best:** mibz2 at 4 threads (972 MB/s); lbzip2 at 4 threads (198 MB/s); pbz2 at 4 threads (304 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.05, lbzip2 1t 0.09, lbzip2 2t 0.41, lbzip2 4t 0.60, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.62 | 0.02 | 0.08 | 0.216 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3273 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 59.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.5% |
| MTF + RLE2 | 1.55 | 18.1% |
| T-vector build | 1.51 | 17.7% |
| walk (the one each build uses) | 4.12 | 48.3% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.509 | 1.301 | (in its total) | 2.56× (2.55–2.56) |  |
| walk | 1.186 | 2.599 | (in its total) | 2.18× (2.16–2.20) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.053 | (in its total) | 1.18× (1.18–1.19) |  |
| inverse BWT, total | 1.741 | 3.953 | 3.203 | 2.26× (2.25–2.28) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m27s |
| 4 threads | 31m19s ± 4m24s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m46s ± 3m13s |
| the scanner alone (its serial part) | 8m53s ± 19.7s |
| the workers alone, over 4 | 22m46s ± 3m13s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.98 ns a byte of output at 4 threads (CPU, summed over them), 1.97 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3273.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
