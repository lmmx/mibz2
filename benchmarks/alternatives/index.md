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

The latest record: [`d20a71b`](<https://github.com/lmmx/mibz2/commit/d20a71bb28e06191cd579fa7b56e1ea638d3a994>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46114103441>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 547 | 6.8 | 3.20× (3.13–3.27) | 7.40× (7.25–7.56) | 7.84× (7.67–8.02) | 8.72× (8.54–8.90) | 7.72× (7.53–7.91) | 8.02× (7.85–8.20) | 7.70× (7.54–7.87) | 2.39× (2.32–2.46) | 2.64× (2.58–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.224 s | 936 | 7.5 | 3.21× (3.05–3.38) | 6.71× (6.33–7.13) | 7.79× (7.48–8.12) | 7.93× (7.58–8.30) | 7.03× (6.71–7.36) | 13.68× (13.02–14.39) | 13.14× (12.52–13.78) | 4.08× (3.87–4.29) | 4.51× (4.30–4.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.182 s | 1,152 | 11.9 | 3.37× (3.28–3.46) | 4.97× (4.83–5.12) | 6.26× (6.08–6.45) | 5.99× (5.82–6.17) | 5.96× (5.78–6.15) | 17.01× (16.73–17.30) | 16.33× (16.09–16.58) | 5.07× (4.93–5.20) | 5.60× (5.51–5.71) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,061 | 3.8 | 0.55× (0.54–0.56) | 3.25× (3.22–3.29) | 4.34× (4.30–4.38) | 3.42× (3.38–3.46) | 5.90× (5.83–5.97) | 2.53× (2.50–2.55) | 2.53× (2.51–2.55) | 0.94× (0.92–0.96) | 1.88× (1.68–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,034 | 4.0 | 0.50× (0.49–0.51) | 2.64× (2.58–2.70) | 5.28× (5.17–5.40) | 3.32× (3.24–3.39) | 5.66× (5.42–5.90) | 2.45× (2.40–2.51) | 2.45× (2.40–2.51) | 0.91× (0.88–0.94) | 1.82× (1.62–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,000 | 4.1 | 0.48× (0.48–0.49) | 2.57× (2.51–2.62) | 5.12× (5.01–5.22) | 3.22× (3.16–3.28) | 5.50× (5.36–5.65) | 2.37× (2.33–2.42) | 2.37× (2.33–2.43) | 0.88× (0.85–0.91) | 1.77× (1.60–1.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 764 | 5.2 | 2.82× (2.78–2.86) | 3.18× (3.16–3.20) | 3.97× (3.95–4.00) | 4.25× (4.22–4.29) | 4.71× (4.64–4.77) | 2.87× (2.85–2.90) | 2.35× (2.33–2.37) | 2.18× (2.16–2.19) | 3.17× (3.14–3.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,117 | 6.4 | 2.19× (1.99–2.41) | 2.88× (2.72–3.04) | 4.13× (3.72–4.59) | 3.48× (3.08–3.93) | 4.98× (4.36–5.69) | 3.94× (3.48–4.46) | 3.22× (2.84–3.65) | 2.98× (2.62–3.39) | 4.34× (3.83–4.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,065 | 12.2 | 2.10× (2.05–2.15) | 2.14× (1.94–2.37) | 3.27× (3.11–3.43) | 2.33× (2.21–2.45) | 4.49× (4.34–4.64) | 3.96× (3.83–4.09) | 3.24× (3.13–3.34) | 3.00× (2.89–3.11) | 4.36× (4.21–4.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.104 s | 77 | 48.8 | 1.49× (1.48–1.50) | 2.85× (2.83–2.87) | 2.68× (2.63–2.74) | 2.70× (2.68–2.72) | 3.04× (2.97–3.12) | 3.24× (3.19–3.28) | 3.35× (3.34–3.37) | 1.35× (1.34–1.36) | 1.56× (1.54–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 130 | 54.1 | 1.38× (1.28–1.48) | 2.80× (2.54–3.09) | 2.56× (2.39–2.75) | 3.33× (3.12–3.55) | 3.08× (2.86–3.31) | 5.23× (4.87–5.63) | 5.43× (5.05–5.83) | 2.19× (2.03–2.35) | 2.52× (2.35–2.70) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 69.9 | 1.52× (1.50–1.54) | 2.51× (2.46–2.56) | 2.54× (2.49–2.59) | 3.12× (3.03–3.21) | 3.33× (3.26–3.41) | 6.80× (6.64–6.97) | 7.05× (6.94–7.17) | 2.84× (2.79–2.90) | 3.27× (3.19–3.36) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.113 s | 85 | 43.9 | 1.64× (1.63–1.65) | 3.28× (3.25–3.32) | 2.94× (2.89–2.99) | 2.98× (2.97–3.00) | 3.38× (3.24–3.51) | 3.94× (3.92–3.96) | 4.33× (4.30–4.36) | 1.65× (1.63–1.67) | 1.96× (1.92–2.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 140 | 49.3 | 1.49× (1.41–1.56) | 3.01× (2.89–3.14) | 2.75× (2.64–2.86) | 3.85× (3.69–4.02) | 3.30× (3.19–3.42) | 6.45× (6.22–6.69) | 7.10× (6.85–7.36) | 2.71× (2.61–2.81) | 3.22× (3.08–3.36) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.057 s | 169 | 67.3 | 1.61× (1.56–1.65) | 2.58× (2.45–2.72) | 2.63× (2.46–2.80) | 3.42× (3.32–3.52) | 3.24× (3.16–3.33) | 7.82× (7.61–8.04) | 8.60× (8.40–8.82) | 3.29× (3.22–3.35) | 3.90× (3.78–4.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 823 | 5.1 | 0.61× (0.61–0.62) | 3.36× (3.33–3.39) | 4.19× (4.14–4.24) | 3.03× (3.00–3.05) | 6.65× (6.53–6.77) | 2.23× (2.21–2.26) | 2.27× (2.25–2.29) | 0.94× (0.93–0.96) | 1.69× (1.67–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 752 | 5.8 | 0.48× (0.47–0.49) | 2.31× (2.26–2.36) | 4.63× (4.54–4.72) | 2.77× (2.72–2.83) | 5.94× (5.87–6.01) | 2.03× (2.00–2.07) | 2.07× (2.03–2.11) | 0.86× (0.84–0.88) | 1.54× (1.51–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 744 | 5.8 | 0.49× (0.48–0.50) | 2.31× (2.26–2.35) | 4.61× (4.53–4.70) | 2.77× (2.72–2.83) | 5.99× (5.81–6.17) | 2.02× (1.98–2.07) | 2.06× (2.02–2.10) | 0.85× (0.83–0.88) | 1.53× (1.50–1.56) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.052 s | 287 | 13.3 | 2.06× (2.01–2.12) | 4.12× (4.05–4.19) | 4.92× (4.75–5.10) | 4.90× (4.87–4.93) | 5.06× (5.02–5.10) | 4.79× (4.75–4.84) | 4.94× (4.92–4.97) | 1.68× (1.67–1.69) | 1.93× (1.91–1.95) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.037 s | 410 | 17.3 | 1.53× (1.35–1.74) | 3.33× (2.88–3.85) | 4.03× (3.51–4.63) | 4.61× (3.90–5.44) | 4.18× (3.70–4.72) | 6.35× (5.64–7.15) | 6.55× (5.83–7.36) | 2.23× (1.98–2.50) | 2.56× (2.27–2.88) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.033 s | 458 | 27.4 | 1.68× (1.58–1.78) | 2.66× (2.55–2.78) | 3.74× (3.59–3.89) | 3.83× (3.76–3.90) | 4.21× (4.07–4.36) | 7.68× (7.53–7.84) | 7.92× (7.76–8.08) | 2.69× (2.63–2.75) | 3.09× (3.03–3.16) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.19× (3.12–3.26) | 7.36× (7.21–7.51) | 7.80× (7.63–7.97) | 8.68× (8.50–8.86) | 7.69× (7.50–7.88) | 7.99× (7.82–8.17) | 7.67× (7.51–7.84) | 2.38× (2.32–2.45) | 2.62× (2.56–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (2.95–3.17) | 6.88× (6.64–7.13) | 7.25× (7.04–7.46) | 8.25× (8.03–8.47) | 6.99× (6.82–7.18) | 7.22× (6.99–7.46) | 6.93× (6.72–7.15) | 2.15× (2.08–2.23) | 2.37× (2.30–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.19× (3.13–3.26) | 5.26× (5.17–5.36) | 5.86× (5.75–5.96) | 6.35× (6.23–6.48) | 5.58× (5.45–5.72) | 4.63× (4.58–4.67) | 4.44× (4.38–4.50) | 1.38× (1.36–1.40) | 1.52× (1.50–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.59) | 2.80× (2.75–2.85) | 3.79× (3.75–3.83) | 3.21× (3.18–3.25) | 5.48× (5.41–5.55) | 2.40× (2.37–2.42) | 2.40× (2.38–2.42) | 0.94× (0.92–0.96) | 1.58× (1.54–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.52–0.55) | 2.51× (2.46–2.57) | 4.54× (4.46–4.62) | 3.04× (2.97–3.10) | 5.10× (4.89–5.31) | 2.25× (2.20–2.29) | 2.25× (2.20–2.29) | 0.88× (0.86–0.91) | 1.48× (1.43–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.52) | 2.69× (2.63–2.75) | 4.51× (4.41–4.61) | 3.01× (2.97–3.05) | 5.04× (4.91–5.16) | 2.22× (2.18–2.26) | 2.22× (2.18–2.27) | 0.87× (0.84–0.90) | 1.46× (1.42–1.51) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.67–2.74) | 2.93× (2.91–2.94) | 3.52× (3.49–3.55) | 4.04× (4.02–4.07) | 4.47× (4.41–4.52) | 2.75× (2.73–2.77) | 2.26× (2.25–2.28) | 2.10× (2.08–2.11) | 2.74× (2.71–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.30–2.41) | 2.59× (2.50–2.69) | 3.46× (3.38–3.54) | 3.43× (3.36–3.51) | 3.75× (3.68–3.83) | 2.22× (2.17–2.27) | 1.82× (1.79–1.86) | 1.69× (1.66–1.72) | 2.21× (2.16–2.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.17× (2.09–2.24) | 1.84× (1.75–1.93) | 2.37× (2.27–2.47) | 2.16× (2.08–2.24) | 2.25× (2.16–2.34) | 1.15× (1.10–1.20) | 0.95× (0.91–0.99) | 0.88× (0.85–0.91) | 1.15× (1.10–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.47–1.49) | 2.79× (2.77–2.80) | 2.60× (2.54–2.66) | 2.67× (2.66–2.69) | 3.01× (2.94–3.08) | 3.20× (3.16–3.24) | 3.32× (3.30–3.33) | 1.35× (1.34–1.36) | 1.47× (1.46–1.48) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.38–1.40) | 2.65× (2.63–2.68) | 2.34× (2.32–2.36) | 3.16× (3.14–3.19) | 2.73× (2.70–2.75) | 2.87× (2.82–2.91) | 2.97× (2.94–3.00) | 1.21× (1.19–1.22) | 1.32× (1.31–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.65× (1.61–1.69) | 2.60× (2.54–2.66) | 2.22× (2.17–2.27) | 2.98× (2.93–3.02) | 2.80× (2.74–2.86) | 2.24× (2.18–2.29) | 2.32× (2.27–2.37) | 0.94× (0.92–0.96) | 1.03× (1.01–1.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.62–1.64) | 3.21× (3.19–3.23) | 2.86× (2.82–2.91) | 2.95× (2.94–2.97) | 3.34× (3.21–3.48) | 3.89× (3.87–3.91) | 4.28× (4.25–4.31) | 1.64× (1.63–1.66) | 1.89× (1.85–1.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.51× (1.49–1.52) | 3.00× (2.97–3.04) | 2.56× (2.55–2.58) | 3.61× (3.53–3.68) | 3.00× (2.97–3.03) | 3.46× (3.43–3.50) | 3.81× (3.77–3.84) | 1.46× (1.44–1.48) | 1.68× (1.64–1.73) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.78× (1.74–1.82) | 2.80× (2.75–2.86) | 2.31× (2.16–2.47) | 3.17× (3.09–3.26) | 2.92× (2.86–2.98) | 2.55× (2.50–2.59) | 2.80× (2.75–2.85) | 1.08× (1.05–1.10) | 1.24× (1.21–1.27) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.40× (2.37–2.44) | 3.39× (3.32–3.45) | 2.77× (2.75–2.80) | 5.95× (5.85–6.05) | 2.08× (2.06–2.10) | 2.11× (2.09–2.14) | 0.95× (0.94–0.96) | 1.34× (1.32–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.52–0.54) | 2.06× (2.02–2.10) | 3.68× (3.60–3.75) | 2.50× (2.46–2.55) | 5.13× (5.06–5.19) | 1.82× (1.79–1.85) | 1.85× (1.82–1.88) | 0.83× (0.81–0.85) | 1.17× (1.15–1.20) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.51–0.53) | 2.27× (2.23–2.32) | 3.71× (3.62–3.80) | 2.55× (2.50–2.60) | 5.28× (5.13–5.45) | 1.85× (1.82–1.89) | 1.88× (1.85–1.92) | 0.85× (0.82–0.87) | 1.19× (1.18–1.21) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.03× (1.98–2.09) | 3.96× (3.94–3.99) | 4.69× (4.55–4.84) | 4.78× (4.75–4.80) | 4.93× (4.89–4.97) | 4.67× (4.63–4.71) | 4.82× (4.80–4.84) | 1.66× (1.65–1.67) | 1.83× (1.82–1.84) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.63× (1.59–1.68) | 3.21× (3.06–3.36) | 3.66× (3.49–3.83) | 4.21× (4.03–4.39) | 3.84× (3.73–3.95) | 3.53× (3.44–3.63) | 3.64× (3.55–3.74) | 1.25× (1.22–1.29) | 1.39× (1.35–1.43) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.75× (1.65–1.84) | 2.67× (2.57–2.77) | 3.05× (2.97–3.13) | 3.29× (3.20–3.39) | 3.15× (3.04–3.26) | 2.28× (2.25–2.32) | 2.35× (2.31–2.39) | 0.81× (0.80–0.83) | 0.89× (0.88–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.32× (0.32–0.33) | 0.97× (0.97–0.98) | 0.79× (0.78–0.79) | 11.00× (10.95–11.04) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.47× (0.39–0.56) | 0.73× (0.71–0.75) | 0.61× (0.59–0.63) | 5.75× (5.58–5.93) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.38) | 0.50× (0.43–0.58) | 0.64× (0.61–0.67) | 0.57× (0.54–0.60) | 3.75× (3.58–3.93) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.92) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.92–0.93) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.65–0.66) | 7.94× (7.92–7.96) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.71) | 0.68× (0.61–0.75) | 1.61× (1.54–1.68) | 0.81× (0.77–0.84) | 5.84× (5.60–6.09) | 0.47× (0.46–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.57) | 0.27× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.71) | 0.78× (0.71–0.84) | 1.23× (1.16–1.30) | 0.76× (0.72–0.81) | 3.63× (3.44–3.84) | 0.29× (0.27–0.30) | 0.30× (0.28–0.32) | 0.33× (0.31–0.35) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.38× (0.35–0.42) | 0.88× (0.85–0.91) | 0.69× (0.66–0.72) | 3.04× (2.94–3.14) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.82× (0.79–0.85) | 2.55× (2.50–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.37× (0.33–0.40) | 0.85× (0.81–0.88) | 0.70× (0.67–0.74) | 2.91× (2.80–3.03) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.50× (0.48–0.52) | 0.79× (0.78–0.80) | 0.83× (0.81–0.85) | 2.29× (2.26–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.30) | 0.24× (0.24–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.17–6.20) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.86) | 0.89× (0.88–0.89) | 6.16× (6.14–6.18) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.39) | 0.66× (0.66–0.66) | 6.37× (6.35–6.39) | 0.39× (0.38–0.39) | 0.41× (0.40–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 36 MB | 0.45× (0.44–0.47) | 0.39× (0.36–0.42) | 1.05× (1.01–1.10) | 0.68× (0.66–0.71) | 3.39× (3.26–3.53) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.35) | 0.16× (0.15–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 56 MB | 0.49× (0.48–0.51) | 0.48× (0.45–0.51) | 0.90× (0.88–0.92) | 0.75× (0.73–0.76) | 2.31× (2.27–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 547 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 936 | 2.02 | 1.11× (1.08–1.13) | 85% | 1.03× (1.02–1.05) | 1.06× (1.04–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,152 | 3.21 | 1.73× (1.69–1.76) | 53% | 1.23× (1.21–1.26) | 1.73× (1.71–1.75) |

**Best:** mibz2 at 4 threads (1,152 MB/s); lbzip2 at 4 threads (233 MB/s); pbz2 at 4 threads (343 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.01, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.53 | 0.01 | 0.08 | 0.182 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3693 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.4% |
| MTF + RLE2 | 1.49 | 17.3% |
| T-vector build | 1.50 | 17.5% |
| walk (the one each build uses) | 4.33 | 50.4% |
| of which: segment assembly | 0.21 | 2.4% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.60 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.058 | (in its total) | 2.68× (2.64–2.72) |  |
| walk | 1.071 | 2.075 | (in its total) | 1.93× (1.89–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.042 | (in its total) | 0.96× (0.91–1.01) |  |
| inverse BWT, total | 1.510 | 3.177 | 1.893 | 2.10× (2.06–2.13) | 1.25× (1.23–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m25s ± 6m55s |
| 4 threads | 26m36s ± 3m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m06s ± 2m44s |
| the scanner alone (its serial part) | 7m43s ± 30.2s |
| the workers alone, over 4 | 19m06s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 4.5 ns a byte of bz2, the workers 2.50 ns a byte of output at 4 threads (CPU, summed over them), 1.60 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3693.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
