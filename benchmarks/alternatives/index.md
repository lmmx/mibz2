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

The latest record: [`b180d61`](<https://github.com/lmmx/mibz2/commit/b180d616153a97a0153a4a61c1a89224fefff67a>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43634388698>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.535 s | 392 | 7.9 | 2.94× (2.92–2.97) | 6.59× (6.53–6.66) | 6.98× (6.94–7.03) | 7.89× (7.84–7.94) | 7.03× (6.98–7.09) | 7.51× (7.46–7.56) | 7.21× (7.15–7.27) | 2.27× (2.26–2.28) | 2.11× (2.09–2.13) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.310 s | 675 | 8.8 | 2.80× (2.73–2.88) | 6.08× (5.91–6.26) | 7.18× (7.01–7.34) | 7.31× (7.06–7.57) | 6.49× (6.30–6.68) | 12.82× (12.43–13.22) | 12.31× (11.94–12.68) | 3.87× (3.75–4.00) | 3.60× (3.49–3.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.258 s | 813 | 14.1 | 3.07× (3.03–3.12) | 5.30× (5.19–5.42) | 6.14× (5.94–6.34) | 6.09× (5.94–6.25) | 5.92× (5.80–6.04) | 15.60× (15.43–15.77) | 14.98× (14.81–15.15) | 4.71× (4.66–4.76) | 4.38× (4.33–4.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 971 | 3.5 | 0.59× (0.58–0.59) | 3.44× (3.37–3.51) | 4.37× (4.28–4.46) | 3.79× (3.72–3.85) | 6.29× (6.19–6.38) | 2.83× (2.77–2.88) | 2.81× (2.76–2.85) | 1.15× (1.13–1.16) | 1.00× (0.98–1.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 943 | 3.6 | 0.54× (0.53–0.56) | 2.89× (2.83–2.96) | 5.84× (5.65–6.05) | 3.73× (3.65–3.81) | 6.12× (5.99–6.25) | 2.76× (2.70–2.82) | 2.74× (2.68–2.80) | 1.12× (1.08–1.16) | 0.98× (0.96–1.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 941 | 3.5 | 0.52× (0.47–0.58) | 2.81× (2.50–3.16) | 5.61× (4.91–6.40) | 3.54× (3.17–3.95) | 5.85× (5.21–6.57) | 2.63× (2.36–2.93) | 2.61× (2.34–2.91) | 1.07× (0.96–1.19) | 0.93× (0.83–1.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 528 | 6.2 | 2.87× (2.83–2.90) | 3.53× (3.49–3.57) | 4.05× (4.01–4.10) | 4.49× (4.43–4.55) | 4.93× (4.87–5.00) | 3.71× (3.67–3.75) | 3.03× (2.97–3.10) | 2.14× (2.12–2.16) | 2.18× (2.15–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 822 | 7.5 | 2.51× (2.41–2.61) | 3.16× (2.93–3.42) | 4.29× (4.08–4.52) | 3.95× (3.86–4.05) | 5.39× (5.27–5.51) | 5.80× (5.67–5.93) | 4.75× (4.63–4.86) | 3.34× (3.27–3.41) | 3.40× (3.33–3.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 800 | 14.1 | 2.09× (2.04–2.14) | 2.10× (2.02–2.19) | 3.00× (2.87–3.13) | 2.53× (2.38–2.70) | 4.29× (4.21–4.37) | 5.62× (5.50–5.73) | 4.59× (4.51–4.68) | 3.24× (3.16–3.32) | 3.29× (3.22–3.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.156 s | 51 | 60.9 | 1.28× (1.25–1.31) | 2.55× (2.47–2.63) | 2.23× (2.16–2.30) | 2.29× (2.23–2.35) | 2.63× (2.55–2.71) | 2.93× (2.85–3.01) | 3.08× (3.00–3.17) | 1.26× (1.24–1.28) | 1.29× (1.25–1.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.092 s | 87 | 65.5 | 1.24× (1.21–1.27) | 2.49× (2.41–2.56) | 2.34× (2.28–2.41) | 2.96× (2.82–3.11) | 2.77× (2.71–2.83) | 4.96× (4.86–5.06) | 5.21× (5.07–5.36) | 2.13× (2.07–2.19) | 2.18× (2.12–2.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 106 | 88.1 | 1.34× (1.31–1.38) | 2.34× (2.29–2.40) | 2.35× (2.19–2.51) | 2.81× (2.66–2.96) | 2.84× (2.78–2.91) | 6.06× (5.92–6.19) | 6.36× (6.19–6.54) | 2.60× (2.51–2.70) | 2.66× (2.59–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.170 s | 57 | 55.4 | 1.36× (1.33–1.39) | 2.98× (2.91–3.04) | 2.48× (2.41–2.55) | 2.55× (2.49–2.61) | 2.89× (2.81–2.97) | 3.64× (3.55–3.73) | 4.12× (4.02–4.22) | 1.49× (1.46–1.52) | 1.54× (1.52–1.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.102 s | 94 | 61.2 | 1.25× (1.20–1.31) | 2.80× (2.59–3.03) | 2.40× (2.29–2.52) | 3.31× (3.17–3.46) | 2.94× (2.79–3.10) | 6.05× (5.78–6.34) | 6.85× (6.54–7.18) | 2.48× (2.37–2.59) | 2.56× (2.44–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 119 | 82.6 | 1.46× (1.43–1.49) | 2.68× (2.60–2.76) | 2.44× (2.39–2.50) | 3.24× (3.15–3.35) | 2.91× (2.83–2.99) | 7.53× (7.34–7.72) | 8.52× (8.33–8.72) | 3.08× (3.03–3.14) | 3.18× (3.11–3.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 4.8 | 0.65× (0.61–0.68) | 2.98× (2.94–3.02) | 4.39× (4.34–4.45) | 3.24× (3.20–3.29) | 6.77× (6.71–6.83) | 2.42× (2.39–2.44) | 2.44× (2.41–2.48) | 1.05× (1.03–1.06) | 1.50× (1.48–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 692 | 5.1 | 0.55× (0.51–0.60) | 2.83× (2.78–2.89) | 4.89× (4.79–5.00) | 3.10× (3.05–3.16) | 6.48× (6.38–6.59) | 2.29× (2.25–2.34) | 2.32× (2.27–2.36) | 0.99× (0.97–1.02) | 1.43× (1.40–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 694 | 5.0 | 0.53× (0.52–0.54) | 2.85× (2.78–2.91) | 4.89× (4.77–5.01) | 3.12× (3.05–3.18) | 6.46× (6.31–6.62) | 2.30× (2.25–2.34) | 2.32× (2.27–2.38) | 0.99× (0.98–1.01) | 1.43× (1.40–1.46) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.071 s | 174 | 18.2 | 1.78× (1.75–1.81) | 3.90× (3.81–4.00) | 4.39× (4.28–4.52) | 4.52× (4.43–4.61) | 4.62× (4.51–4.73) | 4.89× (4.76–5.03) | 4.81× (4.69–4.94) | 1.47× (1.44–1.50) | 1.42× (1.38–1.45) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.046 s | 269 | 21.5 | 1.52× (1.47–1.57) | 3.36× (3.23–3.50) | 4.13× (3.98–4.29) | 4.85× (4.53–5.18) | 4.53× (4.35–4.71) | 7.59× (7.35–7.85) | 7.48× (7.27–7.68) | 2.28× (2.22–2.35) | 2.20× (2.14–2.26) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.043 s | 283 | 32.8 | 1.48× (1.42–1.55) | 2.64× (2.42–2.88) | 3.21× (3.03–3.41) | 3.74× (3.52–3.97) | 3.74× (3.53–3.96) | 7.91× (7.48–8.36) | 7.79× (7.39–8.21) | 2.38× (2.26–2.51) | 2.29× (2.18–2.41) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.92–2.96) | 6.57× (6.50–6.63) | 6.96× (6.92–7.00) | 7.87× (7.82–7.91) | 7.01× (6.96–7.07) | 7.49× (7.44–7.54) | 7.19× (7.13–7.25) | 2.26× (2.25–2.27) | 2.09× (2.08–2.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.72× (2.67–2.78) | 6.14× (6.03–6.25) | 6.73× (6.61–6.86) | 7.53× (7.37–7.69) | 6.38× (6.23–6.53) | 6.65× (6.49–6.82) | 6.39× (6.24–6.53) | 2.01× (1.96–2.06) | 1.86× (1.81–1.91) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.90–3.05) | 5.67× (5.56–5.79) | 5.94× (5.85–6.04) | 6.51× (6.40–6.61) | 5.73× (5.61–5.84) | 4.26× (4.16–4.36) | 4.09× (4.01–4.17) | 1.29× (1.26–1.32) | 1.19× (1.17–1.22) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.62) | 2.88× (2.83–2.93) | 3.96× (3.89–4.03) | 3.58× (3.51–3.64) | 5.89× (5.80–5.98) | 2.69× (2.64–2.74) | 2.67× (2.62–2.72) | 1.14× (1.12–1.16) | 0.92× (0.90–0.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.60) | 2.79× (2.73–2.85) | 5.18× (5.03–5.33) | 3.49× (3.42–3.56) | 5.66× (5.55–5.77) | 2.58× (2.54–2.63) | 2.57× (2.52–2.62) | 1.09× (1.06–1.13) | 0.88× (0.87–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.50–0.62) | 3.14× (2.81–3.50) | 5.09× (4.47–5.79) | 3.39× (3.05–3.76) | 5.50× (4.93–6.15) | 2.51× (2.27–2.78) | 2.50× (2.25–2.78) | 1.06× (0.96–1.18) | 0.86× (0.77–0.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.78× (2.75–2.81) | 3.20× (3.16–3.24) | 3.75× (3.70–3.79) | 4.34× (4.29–4.39) | 4.76× (4.70–4.82) | 3.59× (3.56–3.62) | 2.95× (2.89–3.01) | 2.09× (2.07–2.10) | 1.98× (1.96–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.46× (2.41–2.50) | 2.80× (2.75–2.86) | 3.31× (3.26–3.36) | 3.65× (3.60–3.71) | 3.99× (3.92–4.07) | 2.94× (2.89–3.00) | 2.42× (2.36–2.47) | 1.71× (1.68–1.74) | 1.62× (1.59–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.17× (2.11–2.23) | 1.82× (1.78–1.86) | 2.25× (2.19–2.30) | 2.22× (2.17–2.28) | 2.33× (2.27–2.39) | 1.58× (1.54–1.62) | 1.29× (1.26–1.32) | 0.92× (0.90–0.94) | 0.87× (0.85–0.89) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.28× (1.25–1.31) | 2.50× (2.41–2.58) | 2.19× (2.13–2.26) | 2.28× (2.21–2.34) | 2.61× (2.53–2.69) | 2.91× (2.84–2.99) | 3.06× (2.97–3.14) | 1.26× (1.23–1.28) | 1.25× (1.23–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.21–1.27) | 2.46× (2.41–2.51) | 2.13× (2.09–2.18) | 2.78× (2.71–2.84) | 2.48× (2.43–2.52) | 2.70× (2.64–2.75) | 2.83× (2.77–2.90) | 1.16× (1.14–1.19) | 1.16× (1.13–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.40–1.54) | 2.53× (2.43–2.63) | 2.10× (2.05–2.16) | 2.73× (2.65–2.81) | 2.50× (2.42–2.59) | 1.99× (1.93–2.06) | 2.09× (2.03–2.16) | 0.86× (0.84–0.89) | 0.86× (0.83–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.36× (1.32–1.39) | 2.93× (2.87–3.00) | 2.44× (2.38–2.50) | 2.53× (2.47–2.59) | 2.87× (2.79–2.95) | 3.61× (3.52–3.71) | 4.09× (3.99–4.19) | 1.49× (1.46–1.52) | 1.50× (1.48–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.27× (1.21–1.33) | 2.84× (2.65–3.04) | 2.26× (2.16–2.37) | 3.11× (2.97–3.25) | 2.65× (2.53–2.77) | 3.22× (3.07–3.37) | 3.64× (3.47–3.82) | 1.32× (1.25–1.40) | 1.34× (1.27–1.41) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.58–1.64) | 2.95× (2.90–3.00) | 2.19× (2.16–2.22) | 3.02× (2.96–3.08) | 2.65× (2.60–2.70) | 2.38× (2.33–2.44) | 2.70× (2.64–2.75) | 0.98× (0.96–1.00) | 0.99× (0.96–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.65–0.73) | 2.41× (2.38–2.43) | 3.45× (3.41–3.49) | 2.99× (2.95–3.03) | 6.12× (6.08–6.17) | 2.26× (2.23–2.28) | 2.28× (2.25–2.31) | 1.04× (1.03–1.06) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.56–0.67) | 2.26× (2.22–2.30) | 4.21× (4.14–4.27) | 2.89× (2.84–2.94) | 5.82× (5.72–5.91) | 2.12× (2.08–2.15) | 2.14× (2.10–2.18) | 0.98× (0.96–1.00) | 0.78× (0.76–0.79) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.65× (2.59–2.72) | 4.22× (4.17–4.27) | 2.98× (2.92–3.04) | 5.94× (5.83–6.06) | 2.17× (2.14–2.20) | 2.20× (2.16–2.23) | 1.00× (0.99–1.02) | 0.80× (0.78–0.81) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.76× (1.73–1.79) | 3.75× (3.66–3.85) | 4.23× (4.13–4.33) | 4.44× (4.35–4.52) | 4.54× (4.43–4.64) | 4.80× (4.67–4.93) | 4.72× (4.60–4.84) | 1.46× (1.43–1.49) | 1.37× (1.34–1.41) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.60× (1.58–1.63) | 3.43× (3.32–3.53) | 3.95× (3.89–4.02) | 4.34× (4.24–4.44) | 4.09× (3.94–4.25) | 4.09× (3.99–4.19) | 4.02× (3.95–4.10) | 1.24× (1.22–1.27) | 1.17× (1.15–1.20) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 1.78× (1.62–1.97) | 3.11× (2.88–3.36) | 3.47× (3.23–3.73) | 3.66× (3.41–3.93) | 3.52× (3.27–3.78) | 2.68× (2.49–2.88) | 2.63× (2.46–2.82) | 0.81× (0.76–0.87) | 0.77× (0.71–0.82) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 33 MB | 0.26× (0.26–0.26) | 0.22× (0.22–0.22) | 0.84× (0.84–0.85) | 0.67× (0.67–0.68) | 9.54× (9.52–9.57) | 0.23× (0.23–0.23) | 0.25× (0.25–0.25) | 0.37× (0.37–0.37) | 0.19× (0.18–0.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 71 MB | 0.27× (0.25–0.28) | 0.35× (0.30–0.41) | 0.54× (0.51–0.58) | 0.46× (0.44–0.49) | 4.38× (4.11–4.66) | 0.10× (0.10–0.11) | 0.11× (0.11–0.12) | 0.17× (0.16–0.18) | 0.08× (0.08–0.09) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 118 MB | 0.26× (0.25–0.28) | 0.34× (0.29–0.41) | 0.47× (0.44–0.50) | 0.41× (0.39–0.44) | 2.74× (2.59–2.90) | 0.06× (0.06–0.07) | 0.07× (0.07–0.07) | 0.10× (0.10–0.11) | 0.05× (0.05–0.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.01–8.06) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.68) | 0.64× (0.58–0.70) | 1.58× (1.54–1.63) | 0.80× (0.78–0.82) | 5.80× (5.68–5.93) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.71× (0.64–0.79) | 1.16× (1.14–1.18) | 0.72× (0.71–0.73) | 3.48× (3.42–3.54) | 0.27× (0.27–0.28) | 0.29× (0.28–0.29) | 0.31× (0.31–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.39–0.44) | 0.92× (0.89–0.95) | 0.72× (0.69–0.76) | 3.19× (3.09–3.29) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.62–0.67) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.80× (0.75–0.84) | 2.54× (2.48–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.23× (5.21–5.24) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.37× (0.34–0.40) | 0.84× (0.82–0.86) | 0.68× (0.66–0.70) | 2.87× (2.80–2.95) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.49× (0.47–0.52) | 0.82× (0.81–0.82) | 0.85× (0.83–0.88) | 2.35× (2.32–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.32× (5.30–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.43) | 0.66× (0.66–0.66) | 6.42× (6.39–6.44) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.43× (0.39–0.47) | 1.15× (1.12–1.18) | 0.74× (0.72–0.76) | 3.61× (3.54–3.67) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.18× (0.17–0.18) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 54 MB | 0.52× (0.49–0.54) | 0.50× (0.44–0.57) | 0.95× (0.92–0.97) | 0.76× (0.71–0.80) | 2.43× (2.35–2.51) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 392 | 2.56 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 675 | 2.85 | 1.13× (1.10–1.15) | 85% | 1.05× (1.04–1.07) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 813 | 4.54 | 1.76× (1.72–1.80) | 52% | 1.52× (1.49–1.55) | 1.78× (1.73–1.83) |

**Best:** mibz2 at 4 threads (813 MB/s); lbzip2 at 4 threads (155 MB/s); pbz2 at 4 threads (263 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.12, lbzip2 1t 0.05, lbzip2 2t 0.25, lbzip2 4t 0.25, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.75 | 0.02 | 0.09 | 0.258 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3097 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 11.2% |
| MTF + RLE2 | 1.63 | 16.0% |
| T-vector build | 1.58 | 15.5% |
| walk (the one each build uses) | 5.39 | 53.0% |
| of which: segment assembly | 0.28 | 2.8% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.498 | 1.301 | (in its total) | 2.61× (2.59–2.63) |  |
| walk | 1.605 | 3.066 | (in its total) | 1.90× (1.89–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.091 | 0.080 | (in its total) | 0.88× (0.86–0.90) |  |
| inverse BWT, total | 2.195 | 4.446 | 2.348 | 2.02× (2.00–2.04) | 1.07× (1.06–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m58s |
| 4 threads | 37m32s ± 5m16s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m55s ± 3m50s |
| the scanner alone (its serial part) | 13m58s ± 32.2s |
| the workers alone, over 4 | 26m55s ± 3m50s |

At 4 threads the workers bound it. Per byte: the scanner 8.1 ns a byte of bz2, the workers 3.53 ns a byte of output at 4 threads (CPU, summed over them), 2.10 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,443,515 | 12,305,874 | \`da24cf0eff3da816…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3097.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
