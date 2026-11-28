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

The latest record: [`9dd08e0`](<https://github.com/lmmx/mibz2/commit/9dd08e0a778d2f7900803bab5125674b76016011>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46118595866>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.377 s | 556 | 6.7 | 3.24× (3.17–3.31) | 7.38× (7.28–7.47) | 7.67× (7.59–7.75) | 8.69× (8.57–8.81) | 7.61× (7.50–7.72) | 7.97× (7.86–8.09) | 7.69× (7.58–7.81) | 2.37× (2.34–2.41) | 2.66× (2.62–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.217 s | 965 | 7.3 | 3.32× (3.16–3.49) | 6.69× (6.34–7.06) | 7.65× (7.28–8.03) | 8.05× (7.67–8.45) | 7.06× (6.72–7.41) | 13.79× (13.15–14.45) | 13.30× (12.67–13.96) | 4.10× (3.91–4.30) | 4.59× (4.40–4.79) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.176 s | 1,190 | 11.4 | 3.29× (3.25–3.33) | 4.89× (4.81–4.96) | 6.00× (5.91–6.08) | 5.94× (5.83–6.06) | 5.99× (5.72–6.28) | 17.12× (16.96–17.28) | 16.52× (16.34–16.70) | 5.09× (5.04–5.15) | 5.70× (5.62–5.79) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,055 | 3.8 | 0.55× (0.54–0.56) | 3.26× (3.20–3.32) | 4.34× (4.25–4.42) | 3.43× (3.37–3.49) | 5.98× (5.89–6.08) | 2.53× (2.48–2.57) | 2.53× (2.48–2.57) | 0.94× (0.92–0.97) | 1.93× (1.74–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,015 | 4.1 | 0.48× (0.47–0.50) | 2.58× (2.51–2.66) | 5.22× (5.05–5.40) | 3.27× (3.17–3.38) | 5.53× (5.40–5.66) | 2.40× (2.33–2.47) | 2.40× (2.33–2.48) | 0.90× (0.86–0.94) | 1.84× (1.66–2.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 985 | 4.2 | 0.48× (0.47–0.50) | 2.57× (2.48–2.66) | 5.10× (4.95–5.26) | 3.21× (3.12–3.31) | 5.54× (5.39–5.70) | 2.36× (2.29–2.44) | 2.37× (2.29–2.44) | 0.88× (0.85–0.93) | 1.81× (1.60–2.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 767 | 5.1 | 2.81× (2.79–2.83) | 3.19× (3.18–3.21) | 4.11× (3.97–4.25) | 4.25× (4.24–4.27) | 4.76× (4.69–4.83) | 2.90× (2.85–2.95) | 2.36× (2.35–2.37) | 2.16× (2.15–2.17) | 3.18× (3.16–3.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,210 | 6.2 | 2.33× (2.04–2.65) | 2.97× (2.62–3.37) | 4.43× (3.81–5.14) | 3.72× (3.25–4.26) | 5.43× (4.77–6.19) | 4.29× (3.76–4.89) | 3.49× (3.07–3.96) | 3.20× (2.82–3.64) | 4.71× (4.14–5.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,077 | 12.1 | 2.07× (1.95–2.20) | 2.08× (1.88–2.30) | 3.29× (3.13–3.45) | 2.39× (2.21–2.59) | 4.58× (4.35–4.82) | 4.02× (3.82–4.23) | 3.27× (3.13–3.43) | 3.00× (2.87–3.14) | 4.42× (4.22–4.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 78 | 47.8 | 1.49× (1.48–1.50) | 2.83× (2.79–2.88) | 2.60× (2.57–2.64) | 2.72× (2.70–2.74) | 3.04× (3.01–3.06) | 3.22× (3.20–3.24) | 3.38× (3.35–3.40) | 1.36× (1.34–1.37) | 1.51× (1.48–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 129 | 53.6 | 1.42× (1.39–1.45) | 2.77× (2.69–2.85) | 2.65× (2.59–2.70) | 3.48× (3.39–3.58) | 3.20× (3.16–3.25) | 5.34× (5.26–5.42) | 5.60× (5.51–5.69) | 2.25× (2.21–2.28) | 2.50× (2.44–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 68.8 | 1.51× (1.49–1.54) | 2.45× (2.38–2.53) | 2.45× (2.37–2.52) | 3.00× (2.95–3.06) | 3.31× (3.25–3.37) | 6.70× (6.60–6.81) | 7.03× (6.91–7.15) | 2.82× (2.78–2.86) | 3.14× (3.07–3.22) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 86 | 43.6 | 1.59× (1.52–1.66) | 3.21× (3.06–3.36) | 2.80× (2.66–2.94) | 2.94× (2.80–3.08) | 3.33× (3.18–3.50) | 3.88× (3.70–4.07) | 4.24× (4.04–4.46) | 1.58× (1.51–1.65) | 1.87× (1.79–1.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 48.8 | 1.50× (1.45–1.56) | 3.02× (2.94–3.10) | 2.77× (2.69–2.84) | 3.88× (3.77–4.00) | 3.44× (3.25–3.63) | 6.54× (6.40–6.69) | 7.16× (7.00–7.31) | 2.66× (2.62–2.71) | 3.16× (3.09–3.23) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 170 | 65.8 | 1.59× (1.55–1.62) | 2.49× (2.42–2.56) | 2.52× (2.44–2.59) | 3.41× (3.35–3.48) | 3.23× (3.19–3.27) | 7.83× (7.73–7.92) | 8.56× (8.46–8.67) | 3.19× (3.15–3.23) | 3.78× (3.74–3.82) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 826 | 5.1 | 0.61× (0.60–0.62) | 3.38× (3.33–3.42) | 4.25× (4.20–4.30) | 3.04× (3.00–3.07) | 6.65× (6.48–6.82) | 2.22× (2.19–2.25) | 2.27× (2.25–2.30) | 0.94× (0.93–0.95) | 1.70× (1.68–1.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 744 | 5.9 | 0.48× (0.47–0.49) | 2.32× (2.24–2.39) | 4.60× (4.50–4.71) | 2.77× (2.70–2.84) | 6.12× (5.90–6.35) | 2.00× (1.95–2.04) | 2.04× (2.00–2.09) | 0.85× (0.83–0.87) | 1.53× (1.49–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 722 | 6.0 | 0.47× (0.47–0.48) | 2.24× (2.20–2.28) | 4.48× (4.40–4.56) | 2.69× (2.66–2.73) | 5.97× (5.81–6.14) | 1.95× (1.91–1.98) | 1.99× (1.95–2.03) | 0.82× (0.81–0.84) | 1.49× (1.46–1.51) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.052 s | 289 | 13.2 | 2.03× (2.02–2.05) | 4.12× (4.05–4.18) | 4.73× (4.59–4.88) | 4.94× (4.83–5.06) | 5.16× (5.11–5.21) | 4.80× (4.76–4.85) | 4.92× (4.86–4.98) | 1.67× (1.65–1.68) | 1.94× (1.93–1.96) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.037 s | 406 | 17.0 | 1.54× (1.48–1.60) | 3.34× (3.20–3.50) | 3.99× (3.81–4.18) | 4.65× (4.43–4.89) | 4.49× (4.33–4.65) | 6.72× (6.46–6.99) | 6.89× (6.62–7.16) | 2.33× (2.24–2.43) | 2.72× (2.62–2.83) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.032 s | 475 | 26.4 | 1.66× (1.61–1.70) | 2.60× (2.53–2.67) | 3.61× (3.51–3.71) | 3.82× (3.71–3.92) | 4.26× (4.12–4.40) | 7.83× (7.63–8.04) | 8.03× (7.80–8.26) | 2.72× (2.64–2.80) | 3.17× (3.09–3.25) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.23× (3.16–3.30) | 7.33× (7.23–7.44) | 7.62× (7.54–7.71) | 8.65× (8.53–8.77) | 7.58× (7.47–7.69) | 7.94× (7.83–8.05) | 7.66× (7.54–7.78) | 2.37× (2.33–2.40) | 2.63× (2.60–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.08× (3.06–3.09) | 6.95× (6.87–7.04) | 7.24× (7.17–7.31) | 8.47× (8.39–8.55) | 7.08× (7.01–7.15) | 7.35× (7.31–7.40) | 7.10× (7.05–7.14) | 2.19× (2.17–2.21) | 2.44× (2.42–2.46) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.12× (3.06–3.18) | 5.18× (5.12–5.25) | 5.60× (5.53–5.67) | 6.30× (6.23–6.36) | 5.60× (5.35–5.86) | 4.68× (4.64–4.72) | 4.52× (4.47–4.56) | 1.39× (1.38–1.41) | 1.55× (1.54–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.57–0.59) | 2.79× (2.74–2.84) | 3.76× (3.70–3.82) | 3.23× (3.17–3.28) | 5.56× (5.46–5.65) | 2.40× (2.35–2.44) | 2.40× (2.35–2.44) | 0.95× (0.92–0.98) | 1.57× (1.54–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.52× (0.51–0.54) | 2.47× (2.41–2.54) | 4.52× (4.39–4.66) | 3.00× (2.92–3.08) | 4.99× (4.89–5.10) | 2.21× (2.15–2.26) | 2.21× (2.15–2.26) | 0.87× (0.84–0.91) | 1.45× (1.41–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.52) | 2.71× (2.62–2.81) | 4.49× (4.38–4.60) | 3.01× (2.93–3.09) | 5.08× (4.95–5.22) | 2.22× (2.16–2.28) | 2.22× (2.15–2.28) | 0.88× (0.84–0.92) | 1.45× (1.41–1.50) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.68–2.71) | 2.95× (2.92–2.98) | 3.64× (3.47–3.82) | 4.05× (4.03–4.06) | 4.52× (4.46–4.58) | 2.77× (2.73–2.82) | 2.27× (2.26–2.28) | 2.09× (2.08–2.10) | 2.74× (2.72–2.75) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.42× (2.37–2.46) | 2.65× (2.59–2.73) | 3.54× (3.47–3.61) | 3.55× (3.49–3.61) | 3.92× (3.84–4.01) | 2.31× (2.25–2.37) | 1.89× (1.85–1.92) | 1.73× (1.70–1.77) | 2.27× (2.23–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.18× (2.11–2.25) | 1.86× (1.80–1.91) | 2.39× (2.32–2.46) | 2.22× (2.16–2.27) | 2.30× (2.24–2.36) | 1.18× (1.16–1.20) | 0.96× (0.94–0.98) | 0.89× (0.87–0.90) | 1.16× (1.14–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.47–1.49) | 2.77× (2.75–2.79) | 2.54× (2.50–2.57) | 2.69× (2.67–2.71) | 3.00× (2.98–3.03) | 3.18× (3.16–3.20) | 3.34× (3.31–3.36) | 1.35× (1.34–1.36) | 1.47× (1.46–1.48) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.37–1.40) | 2.63× (2.60–2.66) | 2.33× (2.31–2.35) | 3.20× (3.16–3.23) | 2.76× (2.74–2.79) | 2.85× (2.82–2.87) | 2.98× (2.96–3.01) | 1.21× (1.19–1.22) | 1.32× (1.31–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.65× (1.63–1.68) | 2.57× (2.54–2.61) | 2.15× (2.11–2.20) | 2.96× (2.89–3.03) | 2.80× (2.75–2.84) | 2.22× (2.18–2.25) | 2.33× (2.29–2.36) | 0.94× (0.93–0.96) | 1.03× (1.01–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.58× (1.51–1.65) | 3.12× (2.98–3.26) | 2.73× (2.59–2.87) | 2.91× (2.78–3.04) | 3.30× (3.15–3.46) | 3.83× (3.66–4.02) | 4.19× (4.00–4.40) | 1.57× (1.50–1.64) | 1.81× (1.73–1.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.50× (1.47–1.54) | 2.99× (2.93–3.06) | 2.54× (2.49–2.58) | 3.61× (3.56–3.65) | 3.07× (2.95–3.19) | 3.47× (3.41–3.52) | 3.79× (3.74–3.85) | 1.42× (1.40–1.44) | 1.64× (1.61–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.70–1.83) | 2.78× (2.71–2.85) | 2.24× (2.22–2.26) | 3.21× (3.16–3.26) | 2.93× (2.88–2.99) | 2.57× (2.53–2.62) | 2.82× (2.77–2.86) | 1.05× (1.04–1.07) | 1.22× (1.20–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.42× (2.39–2.45) | 3.33× (3.29–3.39) | 2.79× (2.76–2.82) | 5.96× (5.81–6.11) | 2.07× (2.05–2.10) | 2.12× (2.10–2.15) | 0.95× (0.94–0.96) | 1.34× (1.33–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.52–0.54) | 2.06× (2.02–2.09) | 3.68× (3.61–3.76) | 2.52× (2.47–2.57) | 5.32× (5.18–5.47) | 1.80× (1.77–1.83) | 1.84× (1.81–1.87) | 0.83× (0.81–0.84) | 1.17× (1.14–1.19) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.51× (0.50–0.51) | 2.22× (2.17–2.28) | 3.62× (3.54–3.70) | 2.49× (2.46–2.52) | 5.28× (5.14–5.42) | 1.78× (1.75–1.81) | 1.83× (1.79–1.86) | 0.82× (0.80–0.83) | 1.16× (1.13–1.18) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.00× (1.98–2.02) | 3.96× (3.92–4.00) | 4.46× (4.31–4.61) | 4.81× (4.71–4.93) | 5.03× (4.98–5.07) | 4.68× (4.64–4.72) | 4.79× (4.74–4.85) | 1.65× (1.63–1.66) | 1.82× (1.81–1.84) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.63× (1.61–1.66) | 3.23× (3.17–3.28) | 3.59× (3.54–3.64) | 4.25× (4.18–4.31) | 3.96× (3.90–4.02) | 3.62× (3.57–3.68) | 3.71× (3.67–3.75) | 1.27× (1.26–1.29) | 1.41× (1.39–1.43) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.73× (1.69–1.77) | 2.54× (2.49–2.60) | 2.94× (2.90–2.98) | 3.26× (3.19–3.32) | 3.15× (3.07–3.22) | 2.34× (2.30–2.38) | 2.40× (2.36–2.44) | 0.82× (0.81–0.84) | 0.91× (0.89–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.97–11.05) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.34× (0.33–0.36) | 0.44× (0.37–0.52) | 0.70× (0.67–0.74) | 0.60× (0.57–0.62) | 5.65× (5.41–5.89) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.36–0.38) | 0.56× (0.46–0.68) | 0.65× (0.63–0.67) | 0.56× (0.54–0.58) | 3.78× (3.66–3.89) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.27–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.93) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.94× (7.93–7.95) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.67–0.71) | 0.66× (0.59–0.74) | 1.65× (1.60–1.71) | 0.83× (0.80–0.87) | 5.99× (5.79–6.19) | 0.49× (0.47–0.50) | 0.51× (0.49–0.53) | 0.56× (0.54–0.58) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.69× (0.65–0.72) | 0.75× (0.69–0.81) | 1.22× (1.17–1.27) | 0.75× (0.72–0.78) | 3.58× (3.44–3.73) | 0.28× (0.27–0.29) | 0.29× (0.28–0.31) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.40× (0.36–0.43) | 0.89× (0.86–0.92) | 0.71× (0.69–0.74) | 3.08× (2.98–3.17) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.81× (0.79–0.84) | 2.54× (2.50–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.84) | 5.22× (5.19–5.24) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.38× (0.36–0.40) | 0.86× (0.83–0.89) | 0.73× (0.71–0.75) | 2.95× (2.86–3.05) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.48× (0.44–0.52) | 0.80× (0.79–0.81) | 0.81× (0.79–0.84) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.30× (5.29–5.32) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.86) | 0.89× (0.89–0.89) | 6.17× (6.15–6.19) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.39) | 0.66× (0.65–0.66) | 6.37× (6.34–6.41) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 38 MB | 0.45× (0.44–0.47) | 0.41× (0.38–0.44) | 1.02× (1.00–1.06) | 0.67× (0.65–0.70) | 3.37× (3.27–3.48) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.35) | 0.16× (0.16–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.42× (0.40–0.45) | 0.88× (0.87–0.90) | 0.73× (0.70–0.75) | 2.29× (2.26–2.32) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 556 | 1.81 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 965 | 1.97 | 1.08× (1.07–1.09) | 86% | 1.02× (1.01–1.03) | 1.03× (1.01–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,190 | 3.09 | 1.70× (1.68–1.72) | 54% | 1.20× (1.19–1.21) | 1.64× (1.60–1.68) |

**Best:** mibz2 at 4 threads (1,190 MB/s); lbzip2 at 4 threads (244 MB/s); pbz2 at 4 threads (361 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.50 | 0.02 | 0.08 | 0.176 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3693 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.6% |
| MTF + RLE2 | 1.48 | 17.8% |
| T-vector build | 1.49 | 17.9% |
| walk (the one each build uses) | 4.08 | 49.2% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.30 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.063 | (in its total) | 2.71× (2.69–2.72) |  |
| walk | 1.052 | 2.055 | (in its total) | 1.95× (1.94–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.041 | (in its total) | 0.97× (0.93–1.01) |  |
| inverse BWT, total | 1.487 | 3.160 | 1.897 | 2.12× (2.11–2.14) | 1.28× (1.26–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m17s ± 6m42s |
| 4 threads | 25m45s ± 3m37s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m18s ± 2m36s |
| the scanner alone (its serial part) | 7m51s ± 32.2s |
| the workers alone, over 4 | 18m18s ± 2m36s |

At 4 threads the workers bound it. Per byte: the scanner 4.6 ns a byte of bz2, the workers 2.40 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3693.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
