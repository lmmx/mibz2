# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1238/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`a705435`](<https://github.com/lmmx/mibz2/commit/a7054355767226a112720106f014860a99317d4c>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46074636444>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.383 s | 547 | 6.8 | 3.17× (3.15–3.19) | 7.28× (7.24–7.31) | 7.52× (7.49–7.55) | 8.54× (8.51–8.58) | 7.52× (7.48–7.56) | 7.90× (7.86–7.94) | 7.60× (7.57–7.62) | 2.35× (2.34–2.36) | 2.62× (2.61–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.213 s | 986 | 7.3 | 3.20× (3.05–3.36) | 6.60× (6.20–7.03) | 7.65× (7.20–8.12) | 7.92× (7.51–8.34) | 6.99× (6.58–7.41) | 13.71× (12.94–14.52) | 13.18× (12.45–13.96) | 4.08× (3.85–4.32) | 4.55× (4.30–4.82) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,158 | 11.5 | 3.28× (3.21–3.35) | 4.79× (4.71–4.87) | 5.92× (5.82–6.01) | 5.82× (5.74–5.90) | 5.73× (5.66–5.81) | 16.76× (16.44–17.09) | 16.12× (15.80–16.45) | 4.99× (4.89–5.09) | 5.57× (5.46–5.68) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,105 | 3.6 | 0.56× (0.56–0.57) | 3.38× (3.33–3.42) | 4.51× (4.45–4.57) | 3.53× (3.49–3.57) | 6.04× (5.95–6.14) | 2.62× (2.59–2.66) | 2.62× (2.59–2.66) | 0.95× (0.93–0.97) | 1.79× (1.66–1.94) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,034 | 4.0 | 0.50× (0.49–0.52) | 2.68× (2.63–2.74) | 5.37× (5.25–5.49) | 3.35× (3.29–3.42) | 5.67× (5.57–5.78) | 2.49× (2.44–2.54) | 2.49× (2.43–2.54) | 0.90× (0.87–0.93) | 1.70× (1.56–1.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,031 | 4.0 | 0.50× (0.50–0.51) | 2.65× (2.62–2.68) | 5.29× (5.24–5.35) | 3.31× (3.28–3.34) | 5.61× (5.52–5.70) | 2.45× (2.43–2.48) | 2.45× (2.42–2.47) | 0.89× (0.88–0.90) | 1.67× (1.53–1.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 769 | 5.1 | 2.83× (2.80–2.87) | 3.20× (3.18–3.21) | 4.08× (3.92–4.25) | 4.26× (4.25–4.28) | 4.73× (4.68–4.78) | 2.88× (2.87–2.89) | 2.36× (2.35–2.37) | 2.18× (2.17–2.18) | 3.19× (3.17–3.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,206 | 6.1 | 2.37× (2.06–2.73) | 3.05× (2.62–3.55) | 4.38× (3.77–5.10) | 3.69× (3.21–4.23) | 5.26× (4.62–6.00) | 4.21× (3.70–4.79) | 3.45× (3.03–3.92) | 3.18× (2.79–3.61) | 4.65× (4.09–5.29) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,016 | 12.1 | 2.01× (1.93–2.08) | 2.11× (1.97–2.26) | 3.13× (2.93–3.34) | 2.31× (2.21–2.41) | 4.34× (4.19–4.49) | 3.86× (3.72–4.00) | 3.16× (3.04–3.28) | 2.91× (2.80–3.02) | 4.26× (4.11–4.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 78 | 47.8 | 1.49× (1.48–1.50) | 2.88× (2.84–2.91) | 2.64× (2.58–2.71) | 2.69× (2.67–2.71) | 3.00× (2.98–3.02) | 3.24× (3.21–3.26) | 3.40× (3.38–3.43) | 1.36× (1.35–1.37) | 1.52× (1.49–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 132 | 53.2 | 1.41× (1.36–1.47) | 2.74× (2.63–2.84) | 2.60× (2.50–2.70) | 3.42× (3.28–3.57) | 3.18× (3.05–3.31) | 5.37× (5.16–5.60) | 5.65× (5.43–5.88) | 2.25× (2.16–2.35) | 2.53× (2.41–2.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 165 | 69.3 | 1.54× (1.52–1.57) | 2.52× (2.40–2.65) | 2.52× (2.42–2.62) | 3.04× (2.98–3.11) | 3.32× (3.27–3.38) | 6.86× (6.74–6.99) | 7.22× (7.11–7.33) | 2.88× (2.83–2.93) | 3.23× (3.16–3.31) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.110 s | 87 | 43.0 | 1.63× (1.61–1.65) | 3.28× (3.22–3.35) | 2.86× (2.80–2.92) | 2.98× (2.97–3.00) | 3.32× (3.28–3.36) | 3.96× (3.91–4.02) | 4.39× (4.33–4.45) | 1.62× (1.59–1.65) | 1.93× (1.90–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 140 | 48.4 | 1.47× (1.41–1.52) | 2.97× (2.82–3.14) | 2.70× (2.60–2.81) | 3.80× (3.64–3.96) | 3.29× (3.17–3.42) | 6.43× (6.20–6.67) | 7.12× (6.85–7.40) | 2.62× (2.52–2.73) | 3.14× (3.01–3.27) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 173 | 66.5 | 1.62× (1.56–1.68) | 2.55× (2.45–2.65) | 2.58× (2.47–2.69) | 3.53× (3.33–3.74) | 3.24× (3.12–3.36) | 7.92× (7.68–8.17) | 8.77× (8.50–9.05) | 3.23× (3.11–3.36) | 3.87× (3.73–4.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 5.0 | 0.62× (0.62–0.63) | 3.46× (3.42–3.49) | 4.35× (4.31–4.40) | 3.09× (3.06–3.13) | 6.71× (6.65–6.76) | 2.28× (2.26–2.31) | 2.33× (2.30–2.35) | 0.95× (0.93–0.96) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 784 | 5.6 | 0.50× (0.49–0.51) | 2.40× (2.35–2.46) | 4.90× (4.70–5.12) | 2.88× (2.82–2.95) | 6.19× (6.02–6.38) | 2.11× (2.07–2.15) | 2.15× (2.10–2.19) | 0.87× (0.85–0.89) | 1.61× (1.57–1.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 766 | 5.6 | 0.53× (0.46–0.61) | 2.36× (2.32–2.40) | 4.93× (4.48–5.42) | 2.83× (2.78–2.88) | 6.01× (5.91–6.12) | 2.06× (2.02–2.10) | 2.10× (2.07–2.14) | 0.85× (0.84–0.87) | 1.57× (1.54–1.60) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.051 s | 289 | 13.2 | 2.02× (2.01–2.04) | 4.14× (4.10–4.17) | 4.60× (4.48–4.71) | 4.86× (4.82–4.90) | 5.06× (5.02–5.11) | 4.82× (4.76–4.88) | 4.93× (4.89–4.98) | 1.67× (1.66–1.68) | 1.97× (1.95–1.98) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.039 s | 382 | 16.8 | 1.46× (1.37–1.56) | 3.07× (2.84–3.31) | 3.69× (3.47–3.93) | 4.38× (4.11–4.66) | 4.16× (3.91–4.44) | 6.39× (6.00–6.81) | 6.54× (6.16–6.96) | 2.21× (2.08–2.35) | 2.61× (2.45–2.78) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.032 s | 466 | 25.6 | 1.57× (1.51–1.64) | 2.57× (2.42–2.73) | 3.45× (3.27–3.64) | 3.72× (3.60–3.84) | 4.09× (3.94–4.24) | 7.71× (7.49–7.94) | 7.89× (7.65–8.15) | 2.67× (2.59–2.75) | 3.15× (3.04–3.25) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.16× (3.14–3.18) | 7.23× (7.20–7.27) | 7.48× (7.45–7.51) | 8.51× (8.48–8.54) | 7.49× (7.45–7.52) | 7.87× (7.83–7.90) | 7.57× (7.54–7.59) | 2.35× (2.34–2.35) | 2.60× (2.59–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.03–3.08) | 6.86× (6.81–6.91) | 7.21× (7.16–7.27) | 8.37× (8.33–8.41) | 7.05× (7.00–7.10) | 7.34× (7.29–7.39) | 7.06× (7.02–7.11) | 2.19× (2.18–2.20) | 2.43× (2.41–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.10× (3.02–3.19) | 5.15× (5.06–5.24) | 5.51× (5.40–5.62) | 6.23× (6.16–6.31) | 5.44× (5.34–5.55) | 4.63× (4.59–4.68) | 4.46× (4.42–4.50) | 1.38× (1.37–1.40) | 1.53× (1.52–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.89× (2.85–2.93) | 3.86× (3.77–3.95) | 3.31× (3.27–3.35) | 5.60× (5.51–5.70) | 2.48× (2.45–2.52) | 2.48× (2.45–2.51) | 0.96× (0.94–0.97) | 1.62× (1.60–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.55) | 2.54× (2.50–2.59) | 4.60× (4.49–4.71) | 3.07× (3.01–3.12) | 5.10× (5.02–5.19) | 2.28× (2.24–2.33) | 2.28× (2.23–2.32) | 0.88× (0.85–0.90) | 1.49× (1.47–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.53) | 2.79× (2.75–2.82) | 4.54× (4.51–4.57) | 3.08× (3.06–3.11) | 5.12× (5.04–5.20) | 2.29× (2.27–2.31) | 2.28× (2.26–2.30) | 0.88× (0.87–0.89) | 1.50× (1.47–1.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.69–2.75) | 2.95× (2.93–2.96) | 3.60× (3.43–3.78) | 4.06× (4.04–4.08) | 4.49× (4.45–4.54) | 2.77× (2.76–2.78) | 2.28× (2.27–2.29) | 2.10× (2.09–2.11) | 2.75× (2.73–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.39–2.48) | 2.64× (2.59–2.70) | 3.56× (3.50–3.62) | 3.54× (3.49–3.58) | 3.85× (3.78–3.93) | 2.29× (2.25–2.32) | 1.88× (1.86–1.91) | 1.74× (1.71–1.76) | 2.28× (2.24–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.14× (2.08–2.20) | 1.84× (1.77–1.92) | 2.38× (2.28–2.48) | 2.24× (2.15–2.33) | 2.28× (2.18–2.38) | 1.17× (1.13–1.22) | 0.96× (0.93–1.00) | 0.89× (0.85–0.92) | 1.16× (1.12–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.47–1.49) | 2.78× (2.75–2.80) | 2.57× (2.52–2.63) | 2.66× (2.64–2.68) | 2.97× (2.95–2.99) | 3.20× (3.17–3.23) | 3.36× (3.34–3.39) | 1.35× (1.34–1.36) | 1.48× (1.46–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.36–1.42) | 2.62× (2.57–2.67) | 2.33× (2.28–2.37) | 3.16× (3.11–3.21) | 2.73× (2.68–2.79) | 2.86× (2.80–2.93) | 3.01× (2.95–3.07) | 1.21× (1.18–1.24) | 1.32× (1.29–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.61× (1.59–1.64) | 2.49× (2.45–2.53) | 2.10× (2.07–2.14) | 2.89× (2.87–2.91) | 2.72× (2.69–2.74) | 2.20× (2.17–2.22) | 2.31× (2.29–2.34) | 0.93× (0.92–0.94) | 1.01× (1.00–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.62× (1.60–1.64) | 3.20× (3.16–3.24) | 2.80× (2.75–2.85) | 2.96× (2.94–2.98) | 3.29× (3.26–3.32) | 3.92× (3.88–3.96) | 4.34× (4.30–4.38) | 1.61× (1.59–1.63) | 1.87× (1.85–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.43–1.53) | 2.94× (2.85–3.04) | 2.51× (2.43–2.59) | 3.56× (3.44–3.69) | 2.95× (2.86–3.04) | 3.43× (3.33–3.52) | 3.79× (3.68–3.91) | 1.41× (1.36–1.45) | 1.64× (1.58–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.71–1.83) | 2.80× (2.74–2.85) | 2.24× (2.18–2.29) | 3.22× (3.16–3.29) | 2.89× (2.84–2.94) | 2.55× (2.52–2.57) | 2.82× (2.79–2.85) | 1.04× (1.03–1.06) | 1.21× (1.20–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.45× (2.43–2.48) | 3.41× (3.37–3.44) | 2.83× (2.80–2.85) | 5.98× (5.94–6.02) | 2.12× (2.10–2.14) | 2.16× (2.14–2.18) | 0.96× (0.94–0.97) | 1.36× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.14× (2.09–2.19) | 3.86× (3.61–4.13) | 2.60× (2.55–2.66) | 5.36× (5.21–5.50) | 1.89× (1.86–1.92) | 1.92× (1.89–1.96) | 0.85× (0.83–0.87) | 1.21× (1.19–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.49–0.67) | 2.31× (2.27–2.36) | 3.86× (3.50–4.26) | 2.61× (2.56–2.67) | 5.32× (5.21–5.43) | 1.89× (1.85–1.93) | 1.93× (1.89–1.96) | 0.85× (0.83–0.87) | 1.21× (1.19–1.23) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 1.99× (1.98–2.01) | 3.93× (3.91–3.96) | 4.42× (4.34–4.51) | 4.74× (4.70–4.77) | 4.93× (4.89–4.98) | 4.70× (4.65–4.75) | 4.81× (4.77–4.85) | 1.65× (1.64–1.66) | 1.83× (1.82–1.85) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.65× (1.63–1.67) | 3.25× (3.21–3.29) | 3.61× (3.58–3.64) | 4.26× (4.22–4.30) | 3.95× (3.92–3.98) | 3.69× (3.66–3.72) | 3.77× (3.74–3.81) | 1.29× (1.28–1.31) | 1.44× (1.42–1.46) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.78× (1.74–1.82) | 2.65× (2.57–2.73) | 2.95× (2.90–3.00) | 3.37× (3.33–3.41) | 3.20× (3.15–3.26) | 2.43× (2.38–2.48) | 2.48× (2.44–2.53) | 0.85× (0.84–0.86) | 0.95× (0.93–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.33× (0.32–0.34) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.07× (11.06–11.08) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.45× (0.39–0.51) | 0.71× (0.69–0.73) | 0.60× (0.58–0.63) | 5.70× (5.50–5.90) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.36× (0.34–0.38) | 0.50× (0.40–0.62) | 0.63× (0.60–0.67) | 0.57× (0.53–0.60) | 3.71× (3.51–3.92) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.86× (4.85–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.53) | 0.44× (0.44–0.44) | 1.70× (1.69–1.71) | 0.66× (0.66–0.67) | 8.04× (7.99–8.09) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.72× (0.70–0.74) | 0.70× (0.63–0.77) | 1.68× (1.63–1.74) | 0.85× (0.82–0.88) | 6.10× (5.90–6.30) | 0.50× (0.48–0.51) | 0.52× (0.50–0.54) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.63–0.71) | 0.73× (0.68–0.78) | 1.19× (1.13–1.24) | 0.73× (0.69–0.77) | 3.49× (3.31–3.68) | 0.27× (0.26–0.29) | 0.29× (0.27–0.30) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.50) | 0.40× (0.37–0.43) | 0.90× (0.87–0.92) | 0.72× (0.71–0.74) | 3.11× (3.03–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.45–0.47) | 0.88× (0.87–0.89) | 0.83× (0.82–0.84) | 2.55× (2.52–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.84) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.39) | 0.84× (0.81–0.87) | 0.70× (0.68–0.73) | 2.89× (2.78–3.01) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.50× (0.47–0.52) | 0.80× (0.79–0.81) | 0.81× (0.78–0.84) | 2.31× (2.29–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.90) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.89–0.90) | 6.21× (6.19–6.24) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.39–1.40) | 0.66× (0.66–0.66) | 6.42× (6.39–6.45) | 0.39× (0.39–0.39) | 0.41× (0.40–0.42) | 0.63× (0.62–0.64) | 0.31× (0.31–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 34 MB | 0.49× (0.48–0.51) | 0.43× (0.38–0.48) | 1.10× (1.06–1.14) | 0.75× (0.73–0.78) | 3.66× (3.53–3.79) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.35–0.38) | 0.18× (0.17–0.18) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 53 MB | 0.52× (0.51–0.54) | 0.47× (0.43–0.51) | 0.95× (0.92–0.98) | 0.77× (0.75–0.80) | 2.43× (2.37–2.50) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 547 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 986 | 1.98 | 1.07× (1.07–1.08) | 87% | 1.02× (1.01–1.02) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,158 | 3.12 | 1.70× (1.68–1.71) | 53% | 1.21× (1.19–1.23) | 1.67× (1.63–1.71) |

**Best:** mibz2 at 4 threads (1,158 MB/s); lbzip2 at 4 threads (244 MB/s); pbz2 at 4 threads (355 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.21, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.50 | 0.02 | 0.08 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3681 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 9.9% |
| MTF + RLE2 | 1.57 | 18.6% |
| T-vector build | 1.49 | 17.7% |
| walk (the one each build uses) | 4.07 | 48.5% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.40 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.062 | (in its total) | 2.71× (2.71–2.72) |  |
| walk | 1.048 | 2.075 | (in its total) | 1.98× (1.97–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.041 | (in its total) | 1.06× (1.05–1.08) |  |
| inverse BWT, total | 1.479 | 3.179 | 1.895 | 2.15× (2.14–2.16) | 1.28× (1.28–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m49s ± 6m40s |
| 4 threads | 26m18s ± 3m43s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m16s ± 2m34s |
| the scanner alone (its serial part) | 8m18s ± 30.2s |
| the workers alone, over 4 | 18m16s ± 2m34s |

At 4 threads the workers bound it. Per byte: the scanner 4.8 ns a byte of bz2, the workers 2.39 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,220,170 | 14,849,844 | \`3f9b2018b96fd2e0…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3681.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
