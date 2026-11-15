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

The latest record: [`06aa585`](<https://github.com/lmmx/mibz2/commit/06aa5858ba5f4ad05c5fd4adeaea79ea9ef34355>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43750727376>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.521 s | 402 | 8.1 | 2.90× (2.87–2.93) | 6.63× (6.58–6.68) | 6.79× (6.75–6.82) | 7.39× (7.34–7.44) | 6.66× (6.62–6.70) | 7.26× (7.21–7.31) | 6.70× (6.66–6.75) | 2.14× (2.13–2.16) | 2.49× (2.46–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.295 s | 712 | 8.8 | 2.79× (2.56–3.04) | 5.88× (5.43–6.38) | 6.65× (6.14–7.20) | 6.63× (6.16–7.13) | 5.97× (5.56–6.41) | 12.18× (11.35–13.07) | 11.24× (10.46–12.09) | 3.60× (3.34–3.87) | 4.18× (3.88–4.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.240 s | 873 | 13.6 | 2.88× (2.82–2.93) | 4.37× (4.28–4.47) | 5.47× (5.32–5.62) | 4.97× (4.87–5.07) | 5.03× (4.91–5.15) | 15.65× (15.33–15.97) | 14.45× (14.14–14.75) | 4.62× (4.52–4.72) | 5.37× (5.26–5.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 993 | 3.6 | 0.60× (0.59–0.61) | 3.65× (3.51–3.81) | 4.50× (4.43–4.56) | 3.62× (3.56–3.68) | 6.21× (6.12–6.30) | 3.02× (2.98–3.07) | 2.70× (2.66–2.74) | 1.03× (0.97–1.10) | 1.53× (1.51–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 978 | 3.7 | 0.57× (0.56–0.58) | 3.01× (2.98–3.04) | 5.95× (5.89–6.02) | 3.58× (3.54–3.62) | 6.06× (6.00–6.11) | 2.97× (2.94–3.00) | 2.66× (2.63–2.68) | 1.02× (0.95–1.08) | 1.51× (1.49–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 983 | 3.6 | 0.57× (0.56–0.58) | 3.02× (2.97–3.07) | 5.94× (5.85–6.04) | 3.54× (3.49–3.60) | 6.06× (5.98–6.15) | 2.97× (2.92–3.01) | 2.65× (2.60–2.69) | 1.01× (0.95–1.08) | 1.50× (1.48–1.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 593 | 5.8 | 2.70× (2.65–2.76) | 3.08× (3.05–3.10) | 3.98× (3.92–4.04) | 3.85× (3.82–3.88) | 4.18× (4.14–4.22) | 2.60× (2.57–2.62) | 2.13× (2.11–2.15) | 2.09× (2.07–2.11) | 3.06× (3.03–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 885 | 7.1 | 2.28× (2.19–2.38) | 2.83× (2.66–3.02) | 4.09× (3.87–4.33) | 3.66× (3.28–4.08) | 4.79× (4.58–5.00) | 3.89× (3.73–4.07) | 3.19× (3.05–3.33) | 3.13× (2.99–3.27) | 4.59× (4.39–4.80) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 781 | 13.5 | 1.83× (1.75–1.91) | 2.01× (1.86–2.17) | 2.95× (2.79–3.12) | 2.22× (2.12–2.33) | 3.87× (3.68–4.08) | 3.46× (3.30–3.62) | 2.83× (2.71–2.96) | 2.78× (2.66–2.91) | 4.08× (3.90–4.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 62 | 52.9 | 1.39× (1.38–1.40) | 2.72× (2.69–2.75) | 2.44× (2.40–2.49) | 2.49× (2.47–2.51) | 2.89× (2.86–2.91) | 3.19× (3.16–3.21) | 3.18× (3.15–3.20) | 1.23× (1.22–1.24) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 59.7 | 1.28× (1.23–1.33) | 2.57× (2.44–2.69) | 2.33× (2.22–2.43) | 2.92× (2.81–3.04) | 2.90× (2.80–3.01) | 5.05× (4.86–5.24) | 5.03× (4.86–5.21) | 1.95× (1.88–2.03) | 2.25× (2.16–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 118 | 81.5 | 1.33× (1.30–1.36) | 2.14× (2.06–2.23) | 2.05× (1.97–2.15) | 2.49× (2.43–2.55) | 2.83× (2.76–2.90) | 6.08× (5.94–6.22) | 6.06× (5.92–6.20) | 2.35× (2.30–2.40) | 2.71× (2.66–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 48.9 | 1.52× (1.51–1.53) | 3.10× (3.06–3.14) | 2.57× (2.54–2.60) | 2.65× (2.63–2.66) | 3.11× (3.09–3.13) | 3.67× (3.66–3.68) | 3.91× (3.88–3.95) | 1.43× (1.41–1.46) | 1.70× (1.70–1.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 55.5 | 1.36× (1.33–1.40) | 2.74× (2.66–2.81) | 2.46× (2.39–2.54) | 3.26× (3.17–3.36) | 3.05× (2.97–3.14) | 5.98× (5.82–6.13) | 6.38× (6.21–6.55) | 2.33× (2.26–2.41) | 2.78× (2.70–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 127 | 80.2 | 1.45× (1.42–1.48) | 2.20× (2.11–2.29) | 2.25× (2.19–2.31) | 2.86× (2.80–2.92) | 2.77× (2.72–2.82) | 6.86× (6.73–6.99) | 7.32× (7.15–7.50) | 2.68× (2.61–2.75) | 3.19× (3.13–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 755 | 4.9 | 0.68× (0.67–0.69) | 3.25× (3.04–3.48) | 4.56× (4.52–4.60) | 3.19× (3.16–3.22) | 6.89× (6.83–6.96) | 2.60× (2.58–2.62) | 2.39× (2.36–2.42) | 1.00× (0.99–1.01) | 1.57× (1.55–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 714 | 5.3 | 0.57× (0.56–0.58) | 2.94× (2.89–2.98) | 5.11× (5.01–5.22) | 3.08× (3.01–3.15) | 6.52× (6.43–6.61) | 2.45× (2.42–2.49) | 2.25× (2.21–2.29) | 0.95× (0.93–0.96) | 1.48× (1.45–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 712 | 5.2 | 0.57× (0.56–0.58) | 2.92× (2.89–2.96) | 5.01× (4.95–5.08) | 3.00× (2.96–3.04) | 6.43× (6.37–6.49) | 2.43× (2.40–2.47) | 2.23× (2.21–2.26) | 0.94× (0.93–0.94) | 1.46× (1.44–1.48) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 0.068 s | 188 | 17.8 | 2.01× (1.99–2.02) | 4.19× (4.14–4.25) | 4.48× (4.46–4.50) | 4.53× (4.51–4.56) | 4.75× (4.71–4.78) | 5.30× (5.27–5.34) | 4.87× (4.83–4.91) | 1.52× (1.51–1.53) | 1.95× (1.94–1.96) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 0.047 s | 272 | 22.2 | 1.61× (1.58–1.64) | 3.46× (3.05–3.94) | 3.95× (3.81–4.09) | 4.25× (4.16–4.35) | 4.31× (4.23–4.39) | 7.68× (7.54–7.82) | 7.05× (6.94–7.18) | 2.20× (2.16–2.25) | 2.83× (2.78–2.88) | yes |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 0.042 s | 300 | 34.3 | 1.50× (1.46–1.54) | 2.39× (2.29–2.50) | 3.11× (3.02–3.19) | 3.33× (3.24–3.42) | 3.71× (3.62–3.80) | 8.46× (8.26–8.67) | 7.77× (7.58–7.98) | 2.43× (2.37–2.49) | 3.12× (3.04–3.19) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.89× (2.86–2.92) | 6.60× (6.54–6.65) | 6.75× (6.72–6.79) | 7.37× (7.32–7.41) | 6.64× (6.59–6.68) | 7.24× (7.18–7.29) | 6.68× (6.64–6.73) | 2.14× (2.12–2.16) | 2.47× (2.44–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.76× (2.71–2.80) | 6.23× (6.13–6.34) | 6.48× (6.39–6.58) | 7.10× (7.00–7.21) | 6.15× (6.06–6.24) | 6.64× (6.55–6.74) | 6.13× (6.04–6.22) | 1.96× (1.94–1.99) | 2.27× (2.24–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.80× (2.75–2.85) | 4.74× (4.65–4.84) | 5.10× (4.99–5.20) | 5.37× (5.27–5.47) | 4.79× (4.70–4.88) | 4.35× (4.27–4.44) | 4.02× (3.95–4.09) | 1.29× (1.26–1.31) | 1.49× (1.46–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.31× (3.27–3.36) | 4.10× (4.03–4.16) | 3.38× (3.33–3.43) | 5.74× (5.66–5.82) | 2.83× (2.79–2.88) | 2.54× (2.51–2.58) | 1.03× (0.97–1.09) | 1.09× (1.06–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.61–0.62) | 3.08× (3.04–3.11) | 5.34× (5.29–5.40) | 3.31× (3.28–3.35) | 5.51× (5.47–5.56) | 2.74× (2.71–2.77) | 2.46× (2.44–2.48) | 1.00× (0.94–1.06) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.63) | 3.47× (3.43–3.52) | 5.45× (5.37–5.52) | 3.36× (3.31–3.41) | 5.62× (5.55–5.69) | 2.80× (2.76–2.84) | 2.51× (2.47–2.55) | 1.02× (0.96–1.08) | 1.07× (1.04–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.56–2.66) | 2.83× (2.80–2.86) | 3.56× (3.48–3.65) | 3.69× (3.66–3.72) | 4.00× (3.97–4.04) | 2.51× (2.48–2.53) | 2.06× (2.04–2.08) | 2.03× (2.01–2.05) | 2.74× (2.70–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.27–2.35) | 2.55× (2.52–2.59) | 3.08× (3.03–3.14) | 3.34× (3.28–3.39) | 3.40× (3.34–3.46) | 2.05× (2.01–2.08) | 1.68× (1.66–1.71) | 1.65× (1.63–1.68) | 2.24× (2.19–2.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.09× (2.02–2.16) | 1.84× (1.78–1.90) | 2.29× (2.23–2.37) | 2.23× (2.16–2.30) | 2.17× (2.11–2.23) | 1.08× (1.05–1.12) | 0.89× (0.87–0.92) | 0.88× (0.85–0.90) | 1.19× (1.15–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.38–1.39) | 2.66× (2.64–2.68) | 2.40× (2.36–2.44) | 2.47× (2.45–2.49) | 2.86× (2.83–2.88) | 3.15× (3.13–3.18) | 3.14× (3.12–3.17) | 1.23× (1.22–1.24) | 1.39× (1.37–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.29–1.30) | 2.50× (2.48–2.53) | 2.21× (2.20–2.21) | 2.83× (2.82–2.85) | 2.60× (2.59–2.61) | 2.80× (2.79–2.82) | 2.80× (2.78–2.82) | 1.09× (1.09–1.10) | 1.24× (1.22–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.51× (1.48–1.54) | 2.32× (2.26–2.38) | 1.89× (1.85–1.93) | 2.50× (2.45–2.56) | 2.44× (2.39–2.50) | 2.06× (2.02–2.11) | 2.06× (2.01–2.11) | 0.80× (0.79–0.82) | 0.91× (0.89–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.50–1.52) | 3.04× (3.00–3.07) | 2.52× (2.50–2.54) | 2.62× (2.61–2.64) | 3.08× (3.06–3.10) | 3.63× (3.62–3.64) | 3.87× (3.84–3.90) | 1.43× (1.40–1.45) | 1.67× (1.66–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.40) | 2.77× (2.75–2.79) | 2.30× (2.28–2.32) | 3.10× (3.06–3.14) | 2.74× (2.71–2.77) | 3.18× (3.16–3.21) | 3.40× (3.37–3.42) | 1.25× (1.23–1.27) | 1.46× (1.45–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.59–1.66) | 2.46× (2.40–2.52) | 1.91× (1.85–1.97) | 2.67× (2.59–2.75) | 2.46× (2.41–2.51) | 2.21× (2.17–2.25) | 2.36× (2.31–2.41) | 0.87× (0.85–0.89) | 1.01× (1.00–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.73) | 2.79× (2.75–2.84) | 3.68× (3.59–3.77) | 2.91× (2.88–2.94) | 6.14× (6.08–6.20) | 2.40× (2.38–2.42) | 2.21× (2.19–2.24) | 1.01× (1.00–1.02) | 1.04× (1.01–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.54× (2.51–2.58) | 4.31× (4.22–4.40) | 2.82× (2.75–2.88) | 5.73× (5.66–5.80) | 2.22× (2.19–2.25) | 2.05× (2.02–2.09) | 0.93× (0.92–0.95) | 0.97× (0.93–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.63× (0.61–0.64) | 2.83× (2.78–2.89) | 4.33× (4.27–4.38) | 2.84× (2.80–2.87) | 5.82× (5.77–5.87) | 2.27× (2.24–2.30) | 2.10× (2.07–2.12) | 0.95× (0.94–0.96) | 0.99× (0.96–1.02) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 1.98× (1.96–1.99) | 4.01× (3.98–4.04) | 4.32× (4.30–4.34) | 4.43× (4.40–4.45) | 4.64× (4.61–4.67) | 5.17× (5.14–5.20) | 4.75× (4.71–4.80) | 1.50× (1.49–1.52) | 1.80× (1.79–1.81) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 1.66× (1.63–1.69) | 3.33× (3.28–3.37) | 3.62× (3.56–3.68) | 3.98× (3.92–4.05) | 3.78× (3.72–3.84) | 4.11× (4.04–4.18) | 3.78× (3.73–3.83) | 1.20× (1.18–1.21) | 1.43× (1.40–1.45) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 1.76× (1.70–1.82) | 2.68× (2.64–2.72) | 2.88× (2.81–2.94) | 3.16× (3.09–3.23) | 3.04× (2.98–3.10) | 2.69× (2.65–2.74) | 2.48× (2.44–2.52) | 0.78× (0.77–0.80) | 0.94× (0.92–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.29–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.78) | 11.03× (11.00–11.05) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.33) | 0.48× (0.42–0.55) | 0.68× (0.66–0.70) | 0.57× (0.56–0.58) | 5.42× (5.33–5.51) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.61× (0.52–0.72) | 0.60× (0.57–0.64) | 0.52× (0.49–0.54) | 3.50× (3.33–3.68) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.03× (8.00–8.05) | 0.65× (0.64–0.65) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.73× (0.67–0.79) | 1.61× (1.58–1.64) | 0.80× (0.79–0.82) | 5.84× (5.73–5.95) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.71× (0.65–0.77) | 1.13× (1.07–1.19) | 0.69× (0.66–0.73) | 3.33× (3.17–3.49) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.41× (0.38–0.43) | 0.92× (0.89–0.94) | 0.73× (0.71–0.75) | 3.17× (3.08–3.26) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.87× (0.85–0.88) | 0.82× (0.81–0.84) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.45–0.48) | 0.38× (0.36–0.40) | 0.84× (0.81–0.87) | 0.68× (0.65–0.72) | 2.87× (2.78–2.98) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.52) | 0.80× (0.79–0.81) | 0.81× (0.77–0.85) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.08–1.08) | 0.77× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.67) | 6.39× (6.38–6.40) | 0.39× (0.38–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 2 | 37 MB | 0.45× (0.43–0.47) | 0.41× (0.39–0.43) | 1.05× (1.01–1.10) | 0.67× (0.64–0.70) | 3.28× (3.14–3.43) | 0.20× (0.19–0.21) | 0.22× (0.21–0.23) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `text` (2.52 MB → 12.7 MB, 5.0×) | 4 | 58 MB | 0.48× (0.47–0.49) | 0.50× (0.48–0.52) | 0.88× (0.86–0.91) | 0.71× (0.68–0.73) | 2.24× (2.19–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 402 | 2.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 712 | 2.71 | 1.09× (1.07–1.11) | 84% | 1.03× (1.02–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 873 | 4.20 | 1.66× (1.63–1.70) | 54% | 1.20× (1.18–1.21) | 1.61× (1.59–1.64) |

**Best:** mibz2 at 4 threads (873 MB/s); lbzip2 at 4 threads (198 MB/s); pbz2 at 4 threads (301 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.18, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.49, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 12%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.67 | 0.02 | 0.10 | 0.240 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.8% |
| MTF + RLE2 | 1.51 | 17.1% |
| T-vector build | 1.52 | 17.2% |
| walk (the one each build uses) | 4.14 | 46.8% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.85 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.488 | 1.332 | (in its total) | 2.73× (2.71–2.74) |  |
| walk | 1.223 | 2.615 | (in its total) | 2.14× (2.13–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.24× (1.22–1.26) |  |
| inverse BWT, total | 1.755 | 4.002 | 3.252 | 2.28× (2.27–2.30) | 1.85× (1.84–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m49s |
| 4 threads | 35m17s ± 5m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m07s ± 3m25s |
| the scanner alone (its serial part) | 13m14s ± 19.7s |
| the workers alone, over 4 | 24m07s ± 3m25s |

At 4 threads the workers bound it. Per byte: the scanner 7.7 ns a byte of bz2, the workers 3.16 ns a byte of output at 4 threads (CPU, summed over them), 2.06 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,524,062 | 12,693,642 | \`8a23ef84eb869287…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
