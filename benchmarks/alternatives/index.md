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

The latest record: [`9b5be3b`](<https://github.com/lmmx/mibz2/commit/9b5be3bf9924e96f237eb880ebc349fca1fb51f1>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46141827981>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.478 s | 438 | 7.4 | 3.18× (3.17–3.19) | 7.33× (7.30–7.36) | 7.53× (7.51–7.54) | 8.17× (8.15–8.20) | 7.36× (7.34–7.38) | 8.02× (8.00–8.04) | 7.38× (7.35–7.40) | 2.35× (2.34–2.36) | 2.71× (2.70–2.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.271 s | 773 | 8.0 | 3.18× (2.96–3.42) | 6.73× (6.29–7.20) | 7.63× (7.30–7.97) | 7.59× (7.18–8.02) | 6.86× (6.61–7.12) | 13.77× (13.14–14.42) | 12.66× (12.08–13.28) | 4.03× (3.84–4.23) | 4.65× (4.44–4.88) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.222 s | 944 | 12.7 | 3.19× (3.14–3.25) | 4.99× (4.83–5.15) | 6.14× (6.02–6.25) | 5.51× (5.44–5.58) | 5.63× (5.49–5.77) | 17.23× (17.07–17.39) | 15.85× (15.70–16.00) | 5.04× (4.98–5.11) | 5.82× (5.76–5.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 3.7 | 0.60× (0.59–0.61) | 3.68× (3.50–3.86) | 4.49× (4.33–4.66) | 3.57× (3.53–3.60) | 6.14× (6.09–6.19) | 2.96× (2.93–2.99) | 2.66× (2.63–2.70) | 1.01× (0.99–1.03) | 1.50× (1.49–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 909 | 4.1 | 0.54× (0.54–0.55) | 2.81× (2.78–2.84) | 5.67× (5.54–5.81) | 3.36× (3.33–3.40) | 5.71× (5.65–5.78) | 2.77× (2.74–2.80) | 2.50× (2.47–2.52) | 0.94× (0.93–0.96) | 1.41× (1.39–1.42) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 905 | 4.0 | 0.54× (0.53–0.55) | 2.79× (2.75–2.83) | 5.66× (5.49–5.84) | 3.32× (3.26–3.37) | 5.71× (5.59–5.83) | 2.74× (2.71–2.78) | 2.47× (2.41–2.52) | 0.93× (0.92–0.95) | 1.39× (1.37–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 594 | 5.8 | 2.69× (2.65–2.73) | 3.09× (3.04–3.14) | 3.96× (3.89–4.03) | 3.91× (3.85–3.98) | 4.25× (4.19–4.32) | 2.62× (2.58–2.66) | 2.16× (2.13–2.20) | 2.06× (2.03–2.09) | 3.07× (3.02–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 929 | 7.0 | 2.33× (2.16–2.51) | 2.90× (2.68–3.15) | 4.33× (4.00–4.69) | 3.49× (3.24–3.75) | 4.99× (4.66–5.35) | 4.01× (3.74–4.30) | 3.31× (3.09–3.56) | 3.16× (2.95–3.39) | 4.70× (4.38–5.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 832 | 13.7 | 1.96× (1.87–2.05) | 2.14× (2.02–2.27) | 3.17× (2.96–3.39) | 2.28× (2.15–2.42) | 4.20× (4.01–4.41) | 3.65× (3.47–3.84) | 3.01× (2.87–3.17) | 2.87× (2.74–3.02) | 4.28× (4.06–4.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.124 s | 64 | 51.2 | 1.43× (1.41–1.45) | 2.86× (2.82–2.89) | 2.57× (2.54–2.61) | 2.64× (2.61–2.67) | 3.00× (2.96–3.04) | 3.30× (3.26–3.34) | 3.27× (3.23–3.31) | 1.29× (1.28–1.31) | 1.47× (1.44–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 s | 108 | 56.5 | 1.38× (1.32–1.44) | 2.78× (2.67–2.89) | 2.59× (2.48–2.70) | 3.36× (3.20–3.52) | 3.16× (3.04–3.29) | 5.51× (5.29–5.74) | 5.45× (5.22–5.69) | 2.16× (2.07–2.25) | 2.45× (2.35–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 128 | 77.1 | 1.43× (1.40–1.47) | 2.37× (2.29–2.45) | 2.25× (2.18–2.32) | 2.83× (2.77–2.89) | 3.12× (3.07–3.17) | 6.61× (6.51–6.71) | 6.54× (6.45–6.63) | 2.59× (2.56–2.62) | 2.94× (2.88–3.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.137 s | 70 | 46.9 | 1.54× (1.52–1.56) | 3.22× (3.17–3.26) | 2.73× (2.69–2.78) | 2.82× (2.79–2.85) | 3.28× (3.18–3.38) | 3.85× (3.81–3.90) | 4.05× (3.98–4.12) | 1.50× (1.47–1.53) | 1.79× (1.76–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 118 | 52.5 | 1.40× (1.31–1.50) | 2.88× (2.71–3.07) | 2.59× (2.41–2.78) | 3.58× (3.35–3.83) | 3.19× (2.99–3.41) | 6.26× (5.87–6.67) | 6.57× (6.15–7.02) | 2.43× (2.26–2.60) | 2.90× (2.71–3.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.069 s | 139 | 75.9 | 1.54× (1.50–1.59) | 2.51× (2.45–2.58) | 2.44× (2.36–2.53) | 3.27× (3.21–3.33) | 3.11× (3.06–3.15) | 7.62× (7.51–7.74) | 8.01× (7.90–8.11) | 2.96× (2.91–3.00) | 3.53× (3.48–3.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.0 | 0.67× (0.66–0.67) | 3.04× (3.00–3.08) | 4.43× (4.38–4.48) | 3.14× (3.10–3.17) | 6.77× (6.68–6.87) | 2.54× (2.52–2.57) | 2.33× (2.30–2.35) | 0.99× (0.97–1.00) | 1.53× (1.51–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 5.8 | 0.54× (0.53–0.55) | 2.76× (2.71–2.81) | 4.74× (4.66–4.82) | 2.88× (2.83–2.94) | 6.22× (6.12–6.33) | 2.31× (2.27–2.35) | 2.11× (2.08–2.15) | 0.90× (0.88–0.91) | 1.39× (1.36–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 660 | 5.8 | 0.54× (0.53–0.54) | 2.71× (2.67–2.74) | 4.69× (4.55–4.84) | 2.80× (2.76–2.85) | 6.07× (5.99–6.16) | 2.26× (2.23–2.29) | 2.07× (2.04–2.10) | 0.88× (0.86–0.89) | 1.36× (1.34–1.37) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 0.064 s | 237 | 14.1 | 2.24× (2.22–2.26) | 4.47× (4.42–4.52) | 4.87× (4.81–4.93) | 4.90× (4.84–4.97) | 5.09× (5.01–5.16) | 5.46× (5.39–5.52) | 4.96× (4.90–5.03) | 1.70× (1.68–1.72) | 2.08× (2.04–2.13) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 0.046 s | 328 | 18.9 | 1.70× (1.61–1.80) | 3.39× (3.20–3.60) | 4.07× (3.90–4.24) | 4.45× (4.23–4.68) | 4.26× (4.12–4.40) | 7.41× (7.18–7.64) | 6.74× (6.54–6.93) | 2.30× (2.22–2.39) | 2.83× (2.72–2.94) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 0.042 s | 360 | 28.4 | 1.60× (1.53–1.68) | 2.49× (2.37–2.62) | 3.45× (3.29–3.61) | 3.49× (3.35–3.63) | 3.76× (3.61–3.92) | 8.17× (7.84–8.51) | 7.43× (7.10–7.77) | 2.54× (2.44–2.65) | 3.12× (3.04–3.20) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.17× (3.16–3.18) | 7.29× (7.26–7.32) | 7.49× (7.47–7.51) | 8.14× (8.12–8.17) | 7.33× (7.31–7.35) | 7.99× (7.97–8.01) | 7.35× (7.33–7.37) | 2.34× (2.33–2.35) | 2.69× (2.68–2.70) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.07× (3.05–3.08) | 6.89× (6.84–6.93) | 7.30× (7.21–7.39) | 8.12× (7.93–8.31) | 6.97× (6.82–7.12) | 7.42× (7.40–7.44) | 6.83× (6.80–6.85) | 2.17× (2.16–2.19) | 2.50× (2.49–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.07× (3.03–3.11) | 5.20× (5.12–5.27) | 5.62× (5.54–5.69) | 5.83× (5.75–5.92) | 5.25× (5.17–5.32) | 4.70× (4.62–4.78) | 4.32× (4.25–4.40) | 1.38× (1.35–1.40) | 1.58× (1.56–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.27× (3.23–3.31) | 4.08× (4.03–4.14) | 3.34× (3.31–3.38) | 5.69× (5.63–5.75) | 2.79× (2.76–2.82) | 2.50× (2.48–2.53) | 1.01× (0.99–1.03) | 1.08× (1.05–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.84× (2.82–2.87) | 4.98× (4.93–5.02) | 3.07× (3.04–3.11) | 5.13× (5.08–5.19) | 2.53× (2.50–2.56) | 2.27× (2.24–2.29) | 0.91× (0.90–0.93) | 0.97× (0.95–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.12× (3.08–3.17) | 5.00× (4.93–5.07) | 3.07× (3.02–3.13) | 5.17× (5.05–5.29) | 2.53× (2.49–2.57) | 2.27× (2.24–2.30) | 0.92× (0.90–0.93) | 0.98× (0.94–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.59× (2.56–2.62) | 2.84× (2.81–2.87) | 3.57× (3.54–3.61) | 3.75× (3.70–3.79) | 4.07× (4.02–4.12) | 2.52× (2.50–2.55) | 2.10× (2.07–2.12) | 2.00× (1.98–2.03) | 2.77× (2.74–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.33× (2.30–2.37) | 2.64× (2.59–2.68) | 3.20× (3.14–3.25) | 3.28× (3.23–3.33) | 3.51× (3.47–3.56) | 2.09× (2.06–2.12) | 1.74× (1.71–1.76) | 1.66× (1.64–1.68) | 2.30× (2.26–2.34) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.03–2.18) | 1.85× (1.75–1.95) | 2.31× (2.23–2.40) | 2.17× (2.07–2.28) | 2.24× (2.14–2.34) | 1.09× (1.04–1.14) | 0.90× (0.86–0.94) | 0.86× (0.82–0.90) | 1.19× (1.14–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.40–1.44) | 2.78× (2.75–2.81) | 2.49× (2.47–2.52) | 2.61× (2.58–2.64) | 2.97× (2.93–3.01) | 3.27× (3.23–3.30) | 3.23× (3.19–3.27) | 1.29× (1.27–1.31) | 1.44× (1.42–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.37× (1.36–1.39) | 2.67× (2.64–2.70) | 2.36× (2.34–2.38) | 3.15× (3.13–3.18) | 2.77× (2.74–2.79) | 2.98× (2.96–3.00) | 2.95× (2.93–2.97) | 1.18× (1.17–1.19) | 1.32× (1.30–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.52–1.59) | 2.43× (2.39–2.48) | 1.99× (1.95–2.03) | 2.71× (2.66–2.77) | 2.61× (2.56–2.66) | 2.16× (2.13–2.19) | 2.14× (2.11–2.17) | 0.85× (0.84–0.86) | 0.95× (0.94–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.55) | 3.17× (3.13–3.21) | 2.67× (2.62–2.72) | 2.79× (2.76–2.83) | 3.24× (3.15–3.34) | 3.81× (3.77–3.85) | 4.00× (3.94–4.07) | 1.49× (1.46–1.52) | 1.74× (1.72–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.45× (1.44–1.45) | 2.94× (2.91–2.97) | 2.46× (2.44–2.48) | 3.46× (3.43–3.49) | 2.92× (2.89–2.94) | 3.40× (3.37–3.44) | 3.57× (3.53–3.61) | 1.33× (1.31–1.34) | 1.56× (1.55–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.64–1.71) | 2.65× (2.62–2.69) | 2.03× (2.01–2.06) | 2.92× (2.88–2.96) | 2.66× (2.65–2.68) | 2.37× (2.35–2.39) | 2.49× (2.46–2.51) | 0.92× (0.91–0.94) | 1.08× (1.08–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.74× (2.70–2.78) | 3.57× (3.54–3.60) | 2.86× (2.83–2.89) | 6.03× (5.95–6.10) | 2.35× (2.32–2.37) | 2.16× (2.14–2.18) | 0.99× (0.98–1.00) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.35× (2.30–2.39) | 3.99× (3.93–4.06) | 2.59× (2.53–2.65) | 5.34× (5.24–5.45) | 2.05× (2.00–2.09) | 1.88× (1.84–1.92) | 0.86× (0.85–0.88) | 0.87× (0.85–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.56–0.57) | 2.54× (2.51–2.58) | 3.96× (3.92–4.01) | 2.58× (2.54–2.62) | 5.32× (5.25–5.40) | 2.04× (2.02–2.07) | 1.88× (1.85–1.90) | 0.86× (0.85–0.87) | 0.87× (0.84–0.90) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 2.20× (2.18–2.22) | 4.26× (4.22–4.31) | 4.66× (4.62–4.71) | 4.79× (4.72–4.85) | 4.96× (4.89–5.04) | 5.32× (5.26–5.38) | 4.84× (4.78–4.90) | 1.67× (1.65–1.70) | 2.02× (2.00–2.04) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 1.79× (1.71–1.88) | 3.38× (3.24–3.52) | 3.65× (3.55–3.76) | 4.05× (3.94–4.17) | 3.77× (3.66–3.88) | 3.98× (3.86–4.10) | 3.62× (3.51–3.73) | 1.25× (1.21–1.29) | 1.51× (1.46–1.55) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 1.82× (1.74–1.91) | 2.74× (2.68–2.79) | 3.02× (2.97–3.06) | 3.21× (3.16–3.27) | 3.05× (2.98–3.13) | 2.64× (2.60–2.69) | 2.40× (2.36–2.45) | 0.83× (0.81–0.85) | 1.00× (0.98–1.02) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.99–11.04) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.51× (0.44–0.59) | 0.71× (0.69–0.73) | 0.60× (0.58–0.61) | 5.66× (5.52–5.82) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.37) | 0.74× (0.67–0.82) | 0.65× (0.62–0.67) | 0.56× (0.54–0.58) | 3.71× (3.57–3.86) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.83–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.96× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.67) | 7.95× (7.91–7.99) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.73) | 0.77× (0.72–0.83) | 1.71× (1.67–1.76) | 0.85× (0.83–0.88) | 6.21× (6.05–6.38) | 0.50× (0.49–0.52) | 0.53× (0.51–0.55) | 0.58× (0.56–0.60) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.68× (0.65–0.71) | 0.76× (0.70–0.83) | 1.20× (1.14–1.26) | 0.73× (0.70–0.77) | 3.55× (3.38–3.74) | 0.28× (0.26–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.34) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.13–5.16) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.39× (0.36–0.43) | 0.90× (0.88–0.92) | 0.72× (0.69–0.75) | 3.11× (3.04–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.79–0.84) | 2.56× (2.52–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.37× (0.35–0.40) | 0.83× (0.80–0.87) | 0.68× (0.65–0.71) | 2.85× (2.74–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.51) | 0.81× (0.79–0.83) | 0.81× (0.78–0.83) | 2.33× (2.28–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.29× (5.29–5.30) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.19) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.16× (6.14–6.19) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.05× (1.04–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.38–1.39) | 0.66× (0.66–0.67) | 6.37× (6.35–6.40) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 32 MB | 0.51× (0.50–0.53) | 0.47× (0.44–0.50) | 1.16× (1.13–1.19) | 0.77× (0.74–0.79) | 3.84× (3.76–3.92) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.38× (0.37–0.39) | 0.18× (0.18–0.19) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 52 MB | 0.54× (0.53–0.55) | 0.51× (0.47–0.56) | 0.97× (0.96–0.99) | 0.81× (0.79–0.83) | 2.50× (2.46–2.54) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 438 | 2.29 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 773 | 2.47 | 1.08× (1.07–1.08) | 86% | 1.02× (1.01–1.02) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 944 | 3.91 | 1.70× (1.67–1.73) | 54% | 1.21× (1.20–1.22) | 1.65× (1.62–1.68) |

**Best:** mibz2 at 4 threads (944 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.02, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.60, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.65 | 0.02 | 0.09 | 0.222 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 9.4% |
| MTF + RLE2 | 1.54 | 17.9% |
| T-vector build | 1.51 | 17.6% |
| walk (the one each build uses) | 4.22 | 49.0% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.61 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.509 | 1.308 | (in its total) | 2.57× (2.56–2.58) |  |
| walk | 1.225 | 2.630 | (in its total) | 2.14× (2.11–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.054 | (in its total) | 1.14× (1.14–1.15) |  |
| inverse BWT, total | 1.779 | 3.993 | 3.247 | 2.23× (2.21–2.25) | 1.82× (1.80–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m28s |
| 4 threads | 32m27s ± 4m33s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m23s ± 3m18s |
| the scanner alone (its serial part) | 9m19s ± 32.2s |
| the workers alone, over 4 | 23m23s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.4 ns a byte of bz2, the workers 3.07 ns a byte of output at 4 threads (CPU, summed over them), 1.98 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,268,900 | 15,084,590 | \`c53443aec4395158…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
