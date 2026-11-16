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

The latest record: [`d70ec11`](<https://github.com/lmmx/mibz2/commit/d70ec11088367d5f4df8d9b353ab05c81df29de1>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43850926081>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.413 s | 508 | 5.1 | 2.94× (2.93–2.94) | 6.75× (6.73–6.76) | 6.97× (6.95–6.99) | 7.91× (7.90–7.93) | 6.98× (6.95–7.00) | 7.30× (7.29–7.31) | 6.99× (6.97–7.01) | 2.17× (2.17–2.17) | 2.43× (2.42–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.233 s | 899 | 5.6 | 3.02× (2.96–3.08) | 6.15× (6.03–6.28) | 7.13× (6.97–7.30) | 7.41× (7.29–7.54) | 6.58× (6.49–6.67) | 12.85× (12.71–13.00) | 12.30× (12.16–12.45) | 3.82× (3.78–3.86) | 4.28× (4.23–4.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.194 s | 1,079 | 8.6 | 2.88× (2.86–2.90) | 4.32× (4.26–4.38) | 5.33× (5.24–5.42) | 5.27× (5.20–5.34) | 5.19× (5.13–5.25) | 15.49× (15.31–15.67) | 14.83× (14.65–15.01) | 4.61× (4.55–4.66) | 5.16× (5.11–5.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,130 | 2.5 | 0.59× (0.55–0.63) | 3.43× (3.38–3.48) | 4.58× (4.51–4.65) | 3.56× (3.50–3.61) | 6.12× (5.99–6.25) | 2.66× (2.62–2.70) | 2.67× (2.63–2.71) | 0.97× (0.94–0.99) | 1.77× (1.65–1.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,120 | 2.6 | 0.53× (0.52–0.55) | 2.85× (2.78–2.91) | 5.75× (5.58–5.92) | 3.55× (3.47–3.64) | 6.04× (5.86–6.22) | 2.64× (2.58–2.70) | 2.65× (2.59–2.71) | 0.96× (0.93–0.99) | 1.76× (1.65–1.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,078 | 2.6 | 0.53× (0.52–0.53) | 2.78× (2.73–2.82) | 5.55× (5.46–5.64) | 3.46× (3.40–3.53) | 5.85× (5.69–6.02) | 2.56× (2.52–2.61) | 2.57× (2.53–2.61) | 0.93× (0.91–0.96) | 1.71× (1.61–1.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 754 | 3.7 | 2.77× (2.71–2.84) | 3.13× (3.10–3.16) | 3.95× (3.74–4.17) | 4.18× (4.14–4.22) | 4.54× (4.47–4.60) | 2.83× (2.81–2.85) | 2.31× (2.29–2.34) | 2.12× (2.10–2.13) | 3.12× (3.09–3.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,150 | 4.4 | 2.31× (2.20–2.42) | 2.97× (2.83–3.11) | 4.60× (4.26–4.96) | 3.75× (3.54–3.96) | 5.25× (4.99–5.53) | 4.26× (4.07–4.47) | 3.48× (3.33–3.64) | 3.19× (3.04–3.34) | 4.70× (4.49–4.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,005 | 8.6 | 1.85× (1.76–1.95) | 1.84× (1.66–2.04) | 3.07× (2.91–3.23) | 2.17× (2.04–2.31) | 4.07× (3.86–4.30) | 3.66× (3.49–3.84) | 2.99× (2.86–3.13) | 2.74× (2.61–2.87) | 4.04× (3.86–4.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 s | 74 | 35.9 | 1.39× (1.39–1.39) | 2.65× (2.61–2.68) | 2.44× (2.42–2.46) | 2.52× (2.51–2.53) | 2.81× (2.80–2.83) | 3.02× (3.01–3.04) | 3.17× (3.15–3.18) | 1.25× (1.25–1.25) | 1.41× (1.39–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 118 | 40.2 | 1.28× (1.25–1.32) | 2.46× (2.41–2.51) | 2.34× (2.26–2.43) | 3.02× (2.93–3.12) | 2.88× (2.80–2.96) | 4.85× (4.74–4.96) | 5.08× (4.96–5.20) | 2.01× (1.96–2.05) | 2.26× (2.19–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 53.6 | 1.33× (1.31–1.35) | 2.11× (2.04–2.18) | 2.09× (2.03–2.15) | 2.59× (2.54–2.64) | 2.89× (2.84–2.94) | 5.93× (5.83–6.03) | 6.21× (6.10–6.33) | 2.45× (2.41–2.50) | 2.76× (2.70–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 32.2 | 1.53× (1.52–1.53) | 3.11× (3.08–3.13) | 2.71× (2.68–2.74) | 2.80× (2.79–2.81) | 3.24× (3.09–3.39) | 3.73× (3.72–3.74) | 4.10× (4.09–4.11) | 1.49× (1.49–1.50) | 1.81× (1.81–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 s | 135 | 36.8 | 1.37× (1.34–1.40) | 2.79× (2.68–2.90) | 2.55× (2.45–2.66) | 3.45× (3.31–3.59) | 3.11× (2.99–3.24) | 6.07× (5.82–6.32) | 6.67× (6.41–6.95) | 2.43× (2.33–2.53) | 2.95× (2.83–3.07) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.061 s | 156 | 50.7 | 1.40× (1.37–1.44) | 2.22× (2.14–2.31) | 2.25× (2.18–2.32) | 2.98× (2.91–3.05) | 2.86× (2.80–2.93) | 7.01× (6.85–7.18) | 7.72× (7.54–7.90) | 2.81× (2.74–2.87) | 3.41× (3.33–3.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 3.5 | 0.63× (0.63–0.64) | 3.42× (3.20–3.66) | 4.43× (4.39–4.46) | 3.13× (3.11–3.15) | 6.70× (6.61–6.80) | 2.31× (2.29–2.33) | 2.36× (2.34–2.38) | 0.96× (0.94–0.97) | 1.77× (1.76–1.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 823 | 3.7 | 0.53× (0.52–0.54) | 2.54× (2.50–2.58) | 5.11× (5.03–5.19) | 3.01× (2.97–3.06) | 6.45× (6.28–6.62) | 2.22× (2.18–2.26) | 2.26× (2.23–2.30) | 0.92× (0.90–0.94) | 1.70× (1.67–1.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 794 | 3.7 | 0.52× (0.51–0.53) | 2.48× (2.45–2.52) | 4.97× (4.90–5.04) | 2.96× (2.92–2.99) | 6.30× (6.14–6.46) | 2.16× (2.12–2.19) | 2.20× (2.17–2.23) | 0.89× (0.87–0.91) | 1.65× (1.63–1.67) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.057 s | 233 | 11.5 | 1.84× (1.83–1.85) | 4.06× (4.02–4.09) | 4.42× (4.41–4.43) | 4.63× (4.61–4.65) | 4.80× (4.78–4.82) | 4.79× (4.78–4.81) | 4.86× (4.79–4.92) | 1.51× (1.50–1.52) | 1.77× (1.76–1.77) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.039 s | 343 | 14.1 | 1.49× (1.43–1.55) | 3.23× (3.02–3.46) | 3.94× (3.70–4.19) | 4.30× (4.04–4.58) | 4.42× (4.20–4.64) | 6.92× (6.60–7.24) | 7.01× (6.67–7.36) | 2.18× (2.08–2.29) | 2.55× (2.44–2.67) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.035 s | 378 | 21.7 | 1.50× (1.42–1.59) | 2.47× (2.34–2.60) | 3.19× (3.08–3.30) | 3.40× (3.21–3.61) | 3.94× (3.80–4.09) | 7.75× (7.46–8.05) | 7.85× (7.57–8.15) | 2.44× (2.35–2.54) | 2.86× (2.75–2.97) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.92–2.94) | 6.71× (6.69–6.73) | 6.93× (6.91–6.94) | 7.88× (7.87–7.90) | 6.95× (6.93–6.98) | 7.27× (7.26–7.28) | 6.96× (6.94–6.98) | 2.17× (2.16–2.17) | 2.41× (2.41–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.74–2.75) | 6.23× (6.19–6.26) | 6.52× (6.48–6.55) | 7.57× (7.53–7.62) | 6.41× (6.39–6.43) | 6.66× (6.64–6.68) | 6.37× (6.35–6.39) | 1.98× (1.98–1.99) | 2.21× (2.20–2.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.81× (2.78–2.83) | 4.73× (4.69–4.78) | 5.08× (5.04–5.12) | 5.76× (5.72–5.81) | 4.99× (4.95–5.03) | 4.35× (4.31–4.38) | 4.16× (4.13–4.19) | 1.29× (1.28–1.31) | 1.44× (1.43–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.59–0.67) | 2.93× (2.88–2.97) | 3.85× (3.80–3.90) | 3.33× (3.28–3.37) | 5.66× (5.55–5.77) | 2.51× (2.48–2.54) | 2.52× (2.49–2.55) | 0.97× (0.95–0.99) | 1.64× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.60) | 2.74× (2.69–2.79) | 4.95× (4.83–5.08) | 3.30× (3.23–3.38) | 5.52× (5.37–5.67) | 2.46× (2.41–2.51) | 2.46× (2.41–2.52) | 0.95× (0.93–0.98) | 1.60× (1.57–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.98× (2.94–3.03) | 4.88× (4.80–4.96) | 3.29× (3.24–3.35) | 5.46× (5.31–5.61) | 2.45× (2.41–2.48) | 2.45× (2.42–2.49) | 0.95× (0.93–0.97) | 1.59× (1.57–1.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.60–2.72) | 2.89× (2.85–2.92) | 3.47× (3.30–3.64) | 3.98× (3.94–4.01) | 4.32× (4.26–4.37) | 2.72× (2.69–2.74) | 2.23× (2.21–2.25) | 2.05× (2.03–2.06) | 2.69× (2.67–2.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.32–2.41) | 2.60× (2.54–2.67) | 3.46× (3.39–3.53) | 3.48× (3.42–3.54) | 3.77× (3.70–3.83) | 2.26× (2.23–2.29) | 1.86× (1.82–1.89) | 1.70× (1.68–1.73) | 2.24× (2.20–2.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.12× (2.06–2.18) | 1.82× (1.75–1.88) | 2.36× (2.31–2.41) | 2.19× (2.13–2.25) | 2.25× (2.19–2.31) | 1.17× (1.14–1.19) | 0.96× (0.93–0.98) | 0.88× (0.86–0.90) | 1.16× (1.13–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.38–1.39) | 2.60× (2.59–2.61) | 2.38× (2.35–2.40) | 2.50× (2.49–2.50) | 2.79× (2.77–2.80) | 2.99× (2.98–3.00) | 3.13× (3.12–3.14) | 1.25× (1.24–1.25) | 1.38× (1.38–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.25–1.31) | 2.44× (2.36–2.52) | 2.15× (2.10–2.20) | 2.84× (2.78–2.91) | 2.52× (2.46–2.59) | 2.64× (2.58–2.70) | 2.76× (2.70–2.83) | 1.10× (1.08–1.13) | 1.22× (1.19–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.51) | 2.25× (2.20–2.30) | 1.93× (1.89–1.97) | 2.53× (2.46–2.59) | 2.46× (2.41–2.52) | 1.99× (1.96–2.02) | 2.08× (2.04–2.12) | 0.83× (0.82–0.84) | 0.92× (0.90–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.52–1.52) | 3.01× (3.01–3.02) | 2.64× (2.60–2.67) | 2.77× (2.76–2.78) | 3.20× (3.06–3.35) | 3.69× (3.68–3.70) | 4.06× (4.05–4.07) | 1.48× (1.48–1.49) | 1.74× (1.74–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.40) | 2.76× (2.71–2.81) | 2.35× (2.31–2.40) | 3.24× (3.18–3.31) | 2.77× (2.72–2.82) | 3.22× (3.16–3.28) | 3.54× (3.47–3.60) | 1.29× (1.27–1.32) | 1.52× (1.49–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.54–1.67) | 2.55× (2.48–2.63) | 2.02× (1.96–2.09) | 2.82× (2.75–2.90) | 2.65× (2.57–2.73) | 2.34× (2.27–2.42) | 2.58× (2.50–2.66) | 0.94× (0.91–0.97) | 1.11× (1.07–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.49× (2.45–2.53) | 3.43× (3.39–3.47) | 2.86× (2.84–2.88) | 5.98× (5.90–6.06) | 2.14× (2.13–2.16) | 2.19× (2.18–2.20) | 0.96× (0.95–0.98) | 1.34× (1.22–1.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.29× (2.26–2.32) | 4.02× (3.96–4.07) | 2.76× (2.72–2.80) | 5.66× (5.53–5.81) | 2.02× (1.99–2.05) | 2.06× (2.03–2.09) | 0.91× (0.89–0.92) | 1.26× (1.14–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.51× (2.48–2.55) | 4.03× (3.96–4.11) | 2.79× (2.76–2.82) | 5.71× (5.57–5.86) | 2.03× (2.00–2.05) | 2.07× (2.04–2.09) | 0.91× (0.89–0.93) | 1.26× (1.16–1.37) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.82× (1.81–1.83) | 3.83× (3.82–3.85) | 4.20× (4.19–4.22) | 4.52× (4.50–4.53) | 4.69× (4.67–4.70) | 4.68× (4.66–4.69) | 4.74× (4.68–4.81) | 1.49× (1.49–1.50) | 1.65× (1.64–1.65) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.53× (1.48–1.58) | 3.20× (3.10–3.31) | 3.54× (3.42–3.66) | 4.11× (3.98–4.25) | 3.85× (3.72–3.98) | 3.75× (3.63–3.87) | 3.80× (3.66–3.95) | 1.20× (1.16–1.24) | 1.32× (1.28–1.36) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.67× (1.60–1.74) | 2.60× (2.52–2.68) | 2.90× (2.83–2.98) | 3.29× (3.23–3.35) | 3.13× (3.05–3.21) | 2.50× (2.44–2.56) | 2.53× (2.45–2.61) | 0.80× (0.78–0.82) | 0.88× (0.86–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.31–0.34) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.04× (11.01–11.06) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.34) | 0.41× (0.38–0.45) | 0.66× (0.63–0.69) | 0.57× (0.54–0.59) | 5.34× (5.09–5.60) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 88 MB | 0.34× (0.32–0.35) | 0.45× (0.38–0.53) | 0.60× (0.58–0.63) | 0.53× (0.51–0.55) | 3.52× (3.37–3.67) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.66× (0.62–0.70) | 1.63× (1.60–1.65) | 0.82× (0.80–0.83) | 5.93× (5.85–6.00) | 0.48× (0.47–0.48) | 0.51× (0.50–0.51) | 0.55× (0.54–0.56) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.58–0.66) | 0.70× (0.63–0.78) | 1.11× (1.05–1.17) | 0.68× (0.64–0.72) | 3.28× (3.10–3.46) | 0.26× (0.24–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.41× (0.37–0.45) | 0.93× (0.90–0.96) | 0.75× (0.73–0.78) | 3.22× (3.12–3.34) | 0.20× (0.19–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.67) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.23× (5.21–5.25) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.45–0.48) | 0.36× (0.34–0.39) | 0.83× (0.80–0.85) | 0.69× (0.67–0.71) | 2.84× (2.76–2.93) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.49× (0.46–0.53) | 0.82× (0.80–0.83) | 0.83× (0.80–0.85) | 2.36× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.26–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.41) | 0.67× (0.67–0.68) | 6.41× (6.40–6.42) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.36–0.41) | 1.03× (1.00–1.06) | 0.68× (0.66–0.70) | 3.27× (3.21–3.33) | 0.20× (0.20–0.20) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.46× (0.41–0.51) | 0.90× (0.88–0.93) | 0.74× (0.70–0.78) | 2.32× (2.25–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 508 | 1.98 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 899 | 2.16 | 1.09× (1.09–1.10) | 88% | 1.01× (1.01–1.02) | 1.02× (1.02–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,079 | 3.32 | 1.67× (1.66–1.69) | 53% | 1.18× (1.17–1.19) | 1.60× (1.59–1.62) |

**Best:** mibz2 at 4 threads (1,079 MB/s); lbzip2 at 4 threads (249 MB/s); pbz2 at 4 threads (376 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.22, lbzip2 1t 0.06, lbzip2 2t 0.23, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.53 | 0.02 | 0.08 | 0.194 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.9% |
| MTF + RLE2 | 1.11 | 18.4% |
| T-vector build | 1.03 | 17.1% |
| walk (the one each build uses) | 2.80 | 46.3% |
| of which: segment assembly | 0.10 | 1.6% |
| RLE1 | 0.13 | 2.2% |
| CRC | 0.18 | 3.0% |
| **total** | 6.04 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.062 | (in its total) | 2.72× (2.72–2.73) |  |
| walk | 1.040 | 2.055 | (in its total) | 1.98× (1.97–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.040 | 0.040 | (in its total) | 1.00× (1.00–1.01) |  |
| inverse BWT, total | 1.471 | 3.157 | 1.890 | 2.15× (2.14–2.16) | 1.29× (1.28–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m55s |
| 4 threads | 28m21s ± 3m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m08s ± 2m43s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 19m08s ± 2m43s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.51 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,453 | 13,353,938 | \`724a67aca091a7ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
