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

The latest record: [`18c50c5`](<https://github.com/lmmx/mibz2/commit/18c50c5131f9ce5b4c1e9d41bd13a487f33fec7b>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42134327564>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.533 s | 393 | 9.4 | 2.32× (2.27–2.37) | 5.35× (5.21–5.49) | 5.52× (5.44–5.60) | 6.23× (6.15–6.32) | 5.52× (5.35–5.68) | 6.12× (6.05–6.18) | 5.54× (5.44–5.63) | 1.71× (1.67–1.74) | 1.90× (1.88–1.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.302 s | 694 | 10.1 | 2.35× (2.25–2.45) | 4.91× (4.71–5.12) | 5.57× (5.32–5.82) | 5.85× (5.59–6.13) | 5.15× (4.93–5.38) | 10.69× (10.29–11.12) | 9.68× (9.28–10.09) | 2.99× (2.85–3.13) | 3.32× (3.19–3.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.247 s | 847 | 16.2 | 2.33× (2.26–2.40) | 3.51× (3.34–3.69) | 4.31× (4.20–4.42) | 4.17× (4.05–4.28) | 4.11× (4.04–4.19) | 12.98× (12.69–13.27) | 11.74× (11.40–12.10) | 3.63× (3.50–3.76) | 4.03× (3.94–4.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,120 | 3.6 | 0.57× (0.56–0.58) | 3.43× (3.38–3.48) | 4.62× (4.48–4.77) | 3.57× (3.52–3.63) | 6.03× (5.96–6.10) | 2.67× (2.63–2.71) | 2.65× (2.61–2.69) | 0.99× (0.95–1.02) | 1.87× (1.70–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,090 | 3.8 | 0.52× (0.50–0.54) | 2.81× (2.66–2.97) | 5.79× (5.20–6.44) | 3.46× (3.34–3.59) | 5.83× (5.62–6.06) | 2.57× (2.47–2.67) | 2.55× (2.45–2.65) | 0.95× (0.92–0.98) | 1.80× (1.64–1.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,093 | 3.7 | 0.53× (0.52–0.54) | 2.81× (2.77–2.86) | 5.61× (5.51–5.71) | 3.51× (3.46–3.56) | 6.09× (5.84–6.35) | 2.61× (2.57–2.66) | 2.59× (2.55–2.64) | 0.96× (0.93–1.00) | 1.83× (1.65–2.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 685 | 5.7 | 2.50× (2.49–2.52) | 2.86× (2.84–2.88) | 3.67× (3.44–3.92) | 3.81× (3.77–3.84) | 4.22× (4.08–4.36) | 3.18× (3.15–3.21) | 2.11× (2.09–2.12) | 1.94× (1.93–1.95) | 2.85× (2.83–2.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 973 | 6.9 | 2.00× (1.79–2.24) | 2.56× (2.22–2.95) | 3.84× (3.28–4.49) | 3.14× (2.72–3.62) | 4.46× (3.96–5.03) | 4.44× (3.90–5.05) | 2.94× (2.60–3.33) | 2.71× (2.38–3.08) | 3.98× (3.50–4.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 924 | 13.5 | 1.78× (1.72–1.85) | 1.84× (1.68–2.01) | 2.86× (2.69–3.03) | 2.10× (2.00–2.20) | 3.85× (3.70–4.00) | 4.25× (4.05–4.46) | 2.82× (2.70–2.94) | 2.59× (2.48–2.70) | 3.81× (3.64–3.98) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.5 | 1.38× (1.38–1.39) | 2.65× (2.62–2.68) | 2.43× (2.40–2.47) | 2.52× (2.51–2.53) | 2.81× (2.79–2.83) | 3.19× (3.16–3.21) | 3.18× (3.16–3.20) | 1.27× (1.24–1.30) | 1.40× (1.38–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 118 | 57.8 | 1.29× (1.27–1.31) | 2.51× (2.44–2.58) | 2.37× (2.29–2.44) | 3.13× (3.05–3.22) | 2.90× (2.86–2.95) | 5.18× (5.08–5.28) | 5.16× (5.08–5.25) | 2.07× (1.99–2.14) | 2.27× (2.21–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 77.8 | 1.35× (1.32–1.38) | 2.18× (2.10–2.27) | 2.19× (2.09–2.29) | 2.68× (2.64–2.73) | 2.97× (2.89–3.04) | 6.36× (6.23–6.49) | 6.34× (6.24–6.45) | 2.54× (2.47–2.61) | 2.78× (2.72–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.0 | 1.53× (1.52–1.54) | 3.07× (3.02–3.12) | 2.69× (2.64–2.75) | 2.93× (2.73–3.14) | 3.15× (3.13–3.18) | 4.02× (3.96–4.08) | 4.13× (4.09–4.17) | 1.52× (1.51–1.53) | 1.82× (1.80–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 52.0 | 1.38× (1.33–1.43) | 2.76× (2.67–2.85) | 2.64× (2.43–2.87) | 3.65× (3.54–3.76) | 3.13× (3.03–3.23) | 6.53× (6.34–6.72) | 6.71× (6.55–6.88) | 2.47× (2.40–2.54) | 2.96× (2.85–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 73.5 | 1.43× (1.38–1.49) | 2.29× (2.19–2.39) | 2.29× (2.21–2.36) | 3.11× (2.98–3.25) | 2.90× (2.82–3.00) | 7.60× (7.38–7.83) | 7.82× (7.67–7.97) | 2.88× (2.82–2.94) | 3.45× (3.35–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 830 | 5.1 | 0.62× (0.60–0.63) | 3.41× (3.34–3.48) | 4.28× (4.20–4.37) | 3.04× (2.98–3.10) | 6.56× (6.40–6.72) | 2.34× (2.24–2.46) | 2.28× (2.24–2.33) | 0.94× (0.92–0.96) | 1.72× (1.68–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 778 | 5.5 | 0.50× (0.49–0.52) | 2.42× (2.37–2.48) | 4.86× (4.75–4.98) | 2.88× (2.83–2.94) | 6.34× (6.21–6.47) | 2.21× (2.13–2.30) | 2.15× (2.10–2.20) | 0.89× (0.87–0.91) | 1.62× (1.58–1.66) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 781 | 5.3 | 0.51× (0.50–0.53) | 2.43× (2.36–2.51) | 4.87× (4.73–5.01) | 2.93× (2.84–3.02) | 6.43× (6.19–6.69) | 2.22× (2.11–2.33) | 2.16× (2.10–2.22) | 0.89× (0.86–0.92) | 1.62× (1.58–1.67) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 0.066 s | 189 | 20.1 | 1.43× (1.41–1.46) | 3.09× (3.07–3.12) | 3.41× (3.34–3.48) | 3.60× (3.42–3.80) | 3.70× (3.63–3.77) | 3.80× (3.74–3.87) | 3.72× (3.61–3.83) | 1.17× (1.16–1.19) | 1.38× (1.36–1.41) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 0.042 s | 294 | 23.3 | 1.21× (1.12–1.31) | 2.65× (2.50–2.81) | 3.12× (2.89–3.36) | 3.49× (3.24–3.75) | 3.57× (3.31–3.85) | 5.84× (5.47–6.23) | 5.71× (5.29–6.16) | 1.80× (1.67–1.94) | 2.12× (1.98–2.28) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 0.035 s | 359 | 34.5 | 1.32× (1.28–1.36) | 2.20× (2.08–2.32) | 2.90× (2.65–3.18) | 3.20× (3.09–3.32) | 3.51× (3.43–3.58) | 7.26× (7.09–7.43) | 7.09× (6.86–7.34) | 2.24× (2.19–2.29) | 2.64× (2.58–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.27–2.36) | 5.33× (5.19–5.46) | 5.49× (5.41–5.58) | 6.22× (6.13–6.30) | 5.50× (5.34–5.67) | 6.10× (6.03–6.16) | 5.52× (5.42–5.62) | 1.71× (1.67–1.74) | 1.89× (1.87–1.91) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.22× (2.17–2.27) | 5.00× (4.93–5.07) | 5.23× (5.16–5.31) | 6.08× (5.97–6.18) | 5.11× (5.04–5.18) | 5.63× (5.54–5.72) | 5.09× (5.04–5.15) | 1.58× (1.55–1.60) | 1.74× (1.71–1.78) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.11–2.31) | 3.68× (3.51–3.86) | 4.02× (3.89–4.16) | 4.41× (4.26–4.56) | 3.83× (3.71–3.95) | 3.52× (3.39–3.64) | 3.18× (3.06–3.30) | 0.98× (0.95–1.02) | 1.09× (1.05–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.90× (2.86–2.95) | 3.93× (3.83–4.04) | 3.34× (3.30–3.39) | 5.58× (5.51–5.65) | 2.52× (2.49–2.55) | 2.50× (2.47–2.53) | 0.99× (0.96–1.02) | 1.64× (1.62–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.74× (2.56–2.93) | 4.93× (4.64–5.23) | 3.22× (3.12–3.33) | 5.34× (5.15–5.54) | 2.39× (2.30–2.48) | 2.38× (2.29–2.47) | 0.94× (0.91–0.97) | 1.56× (1.51–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.01× (2.96–3.06) | 4.94× (4.87–5.01) | 3.33× (3.29–3.38) | 5.67× (5.44–5.92) | 2.49× (2.44–2.53) | 2.47× (2.43–2.51) | 0.97× (0.94–1.01) | 1.62× (1.59–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.41× (2.40–2.43) | 2.64× (2.62–2.65) | 3.30× (3.06–3.56) | 3.64× (3.61–3.68) | 4.03× (3.90–4.16) | 3.05× (3.02–3.08) | 2.04× (2.03–2.05) | 1.88× (1.87–1.89) | 2.45× (2.44–2.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.13× (2.09–2.18) | 2.34× (2.24–2.45) | 3.15× (3.05–3.25) | 3.13× (3.01–3.25) | 3.38× (3.26–3.51) | 2.48× (2.37–2.60) | 1.66× (1.60–1.73) | 1.53× (1.47–1.59) | 2.00× (1.92–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.91–2.02) | 1.63× (1.55–1.72) | 2.11× (2.02–2.19) | 1.99× (1.89–2.08) | 2.01× (1.93–2.09) | 1.27× (1.21–1.33) | 0.85× (0.82–0.89) | 0.78× (0.75–0.82) | 1.02× (0.98–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.59× (2.57–2.60) | 2.37× (2.34–2.40) | 2.49× (2.48–2.50) | 2.78× (2.77–2.80) | 3.16× (3.13–3.18) | 3.15× (3.12–3.17) | 1.27× (1.24–1.30) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.30) | 2.44× (2.40–2.47) | 2.17× (2.15–2.20) | 2.96× (2.92–3.00) | 2.54× (2.50–2.57) | 2.81× (2.77–2.85) | 2.80× (2.77–2.83) | 1.13× (1.10–1.16) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.47–1.51) | 2.30× (2.24–2.35) | 1.94× (1.91–1.98) | 2.62× (2.57–2.67) | 2.49× (2.44–2.54) | 2.10× (2.08–2.13) | 2.10× (2.07–2.13) | 0.85× (0.82–0.87) | 0.91× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.53) | 3.01× (2.98–3.04) | 2.62× (2.57–2.67) | 2.90× (2.70–3.11) | 3.12× (3.10–3.15) | 3.97× (3.91–4.03) | 4.09× (4.05–4.13) | 1.51× (1.50–1.52) | 1.75× (1.73–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.39–1.41) | 2.80× (2.78–2.82) | 2.45× (2.30–2.61) | 3.39× (3.31–3.47) | 2.83× (2.78–2.87) | 3.52× (3.47–3.57) | 3.62× (3.57–3.68) | 1.34× (1.33–1.36) | 1.55× (1.53–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.54–1.66) | 2.52× (2.46–2.58) | 2.03× (1.97–2.08) | 2.91× (2.80–3.02) | 2.63× (2.55–2.70) | 2.49× (2.42–2.55) | 2.56× (2.51–2.62) | 0.95× (0.93–0.97) | 1.10× (1.07–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.41× (2.35–2.47) | 3.36× (3.28–3.44) | 2.78× (2.73–2.83) | 5.86× (5.72–5.99) | 2.18× (2.08–2.28) | 2.12× (2.08–2.16) | 0.95× (0.93–0.97) | 1.35× (1.32–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.58) | 2.19× (2.15–2.24) | 3.93× (3.87–3.99) | 2.66× (2.61–2.71) | 5.60× (5.49–5.71) | 2.02× (1.94–2.10) | 1.97× (1.92–2.01) | 0.88× (0.87–0.90) | 1.25× (1.22–1.29) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.58) | 2.46× (2.38–2.53) | 3.98× (3.85–4.11) | 2.78× (2.71–2.86) | 5.86× (5.65–6.08) | 2.09× (2.00–2.19) | 2.04× (1.98–2.09) | 0.91× (0.89–0.94) | 1.30× (1.25–1.34) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 1.42× (1.40–1.44) | 2.93× (2.92–2.95) | 3.26× (3.18–3.34) | 3.54× (3.36–3.72) | 3.63× (3.57–3.70) | 3.73× (3.67–3.79) | 3.65× (3.54–3.76) | 1.17× (1.16–1.18) | 1.26× (1.24–1.28) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 1.30× (1.28–1.32) | 2.61× (2.57–2.66) | 2.95× (2.91–3.00) | 3.36× (3.31–3.42) | 3.22× (3.18–3.26) | 3.23× (3.17–3.29) | 3.16× (3.10–3.22) | 1.01× (0.98–1.04) | 1.09× (1.07–1.11) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 1.45× (1.40–1.50) | 2.22× (2.16–2.28) | 2.56× (2.36–2.77) | 2.72× (2.66–2.79) | 2.68× (2.62–2.74) | 2.18× (2.14–2.21) | 2.13× (2.06–2.19) | 0.68× (0.66–0.71) | 0.74× (0.72–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.35–0.38) | 1.08× (1.08–1.09) | 0.86× (0.86–0.87) | 12.25× (12.23–12.26) | 0.28× (0.28–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.37× (0.36–0.38) | 0.54× (0.45–0.64) | 0.79× (0.77–0.80) | 0.66× (0.65–0.67) | 6.28× (6.15–6.42) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.61× (0.53–0.70) | 0.70× (0.68–0.71) | 0.62× (0.61–0.63) | 4.11× (4.09–4.13) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.17) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.67× (0.66–0.67) | 8.18× (8.15–8.20) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.66× (0.64–0.69) | 1.65× (1.62–1.68) | 0.80× (0.79–0.82) | 5.97× (5.86–6.08) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.65) | 0.69× (0.64–0.75) | 1.18× (1.15–1.21) | 0.75× (0.72–0.77) | 3.42× (3.35–3.51) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.49× (0.48–0.51) | 0.43× (0.41–0.44) | 0.92× (0.89–0.95) | 0.72× (0.69–0.75) | 3.17× (3.07–3.29) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.46× (0.45–0.47) | 0.86× (0.84–0.88) | 0.80× (0.78–0.83) | 2.51× (2.45–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.84–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.41) | 0.87× (0.84–0.90) | 0.73× (0.69–0.77) | 2.98× (2.87–3.09) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.60) | 0.49× (0.46–0.53) | 0.81× (0.80–0.82) | 0.83× (0.79–0.87) | 2.33× (2.30–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 22 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.10× (1.09–1.10) | 0.77× (0.77–0.78) | 5.48× (5.47–5.49) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.78) | 5.44× (5.41–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.77× (0.77–0.78) | 5.46× (5.45–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.50× (1.50–1.51) | 0.65× (0.65–0.66) | 6.80× (6.79–6.81) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 33 MB | 0.48× (0.46–0.49) | 0.40× (0.36–0.44) | 1.18× (1.12–1.23) | 0.72× (0.69–0.75) | 3.66× (3.53–3.79) | 0.22× (0.21–0.23) | 0.23× (0.22–0.24) | 0.35× (0.34–0.37) | 0.18× (0.17–0.18) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 53 MB | 0.51× (0.50–0.52) | 0.52× (0.48–0.56) | 0.96× (0.94–0.99) | 0.77× (0.75–0.80) | 2.48× (2.42–2.54) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 393 | 2.55 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 694 | 2.72 | 1.08× (1.06–1.11) | 87% | 1.02× (1.00–1.04) | 1.04× (1.01–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 847 | 4.39 | 1.73× (1.68–1.79) | 53% | 1.20× (1.14–1.26) | 1.65× (1.58–1.73) |

**Best:** mibz2 at 4 threads (847 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (362 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.09, lbzip2 1t 0.07, lbzip2 2t 0.28, lbzip2 4t 0.42, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.73 | 0.02 | 0.09 | 0.247 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.5% |
| MTF + RLE2 | 1.52 | 17.1% |
| T-vector build | 1.77 | 19.9% |
| walk (the one each build uses) | 4.03 | 45.4% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.9% |
| **total** | 8.87 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.454 | 1.062 | (in its total) | 2.36× (2.33–2.38) |  |
| walk | 1.046 | 2.052 | (in its total) | 1.98× (1.95–2.01) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.040 | (in its total) | 1.03× (1.01–1.06) |  |
| inverse BWT, total | 1.539 | 3.154 | 1.902 | 2.07× (2.04–2.09) | 1.23× (1.23–1.24) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m15s |
| 4 threads | 36m31s ± 5m12s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m53s ± 3m55s |
| the scanner alone (its serial part) | 11m38s ± 0.00s |
| the workers alone, over 4 | 26m53s ± 3m55s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 3.52 ns a byte of output at 4 threads (CPU, summed over them), 2.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,807,836 | 12,402,701 | \`8331caec1322f098…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
