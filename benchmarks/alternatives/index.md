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

The latest record: [`dab66ed`](<https://github.com/lmmx/mibz2/commit/dab66ed212a6eb587165dfb334d147c902cbc22c>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43790252802>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.327 s | 642 | 6.4 | 2.74× (2.66–2.82) | 6.96× (6.72–7.20) | 7.46× (7.16–7.79) | 8.22× (7.91–8.55) | 7.35× (7.07–7.65) | 7.58× (7.37–7.80) | 7.21× (6.99–7.43) | 2.17× (2.10–2.24) | 2.48× (2.41–2.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.205 s | 1,021 | 7.6 | 3.04× (2.84–3.26) | 6.08× (5.73–6.46) | 8.00× (7.28–8.80) | 7.10× (6.81–7.41) | 6.31× (6.00–6.65) | 12.06× (11.62–12.52) | 11.46× (11.11–11.83) | 3.45× (3.32–3.59) | 3.95× (3.80–4.10) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.173 s | 1,210 | 11.8 | 2.85× (2.63–3.09) | 4.32× (3.92–4.77) | 6.02× (5.23–6.92) | 5.15× (4.70–5.64) | 4.97× (4.44–5.56) | 14.03× (12.80–15.37) | 13.33× (12.31–14.45) | 4.02× (3.73–4.33) | 4.59× (4.27–4.93) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,351 | 3.4 | 0.58× (0.55–0.60) | 3.46× (3.31–3.60) | 4.93× (4.08–5.96) | 3.70× (3.53–3.88) | 6.45× (6.26–6.65) | 2.75× (2.64–2.86) | 2.75× (2.64–2.86) | 1.05× (0.99–1.11) | 2.08× (1.99–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,270 | 3.6 | 0.52× (0.49–0.55) | 2.88× (2.64–3.15) | 5.51× (5.21–5.84) | 3.42× (3.26–3.59) | 5.90× (5.60–6.22) | 2.56× (2.41–2.71) | 2.56× (2.42–2.70) | 0.98× (0.92–1.04) | 1.93× (1.82–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,258 | 3.6 | 0.52× (0.47–0.59) | 2.81× (2.54–3.10) | 5.61× (5.13–6.13) | 3.40× (3.16–3.65) | 5.93× (5.52–6.38) | 2.52× (2.34–2.72) | 2.52× (2.35–2.71) | 0.97× (0.86–1.09) | 1.91× (1.78–2.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 982 | 4.5 | 2.69× (2.54–2.85) | 3.41× (3.15–3.69) | 4.13× (3.74–4.55) | 4.28× (4.02–4.56) | 5.10× (4.58–5.67) | 2.88× (2.63–3.15) | 2.51× (2.33–2.71) | 2.07× (1.94–2.21) | 3.00× (2.76–3.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,202 | 6.9 | 2.05× (1.74–2.42) | 2.53× (2.08–3.09) | 3.94× (3.37–4.61) | 3.20× (2.67–3.84) | 4.84× (4.08–5.73) | 3.50× (2.89–4.25) | 3.06× (2.58–3.64) | 2.52× (2.10–3.03) | 3.65× (3.03–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 982 | 13.6 | 1.64× (1.52–1.76) | 1.69× (1.55–1.83) | 2.73× (2.50–2.99) | 1.95× (1.76–2.15) | 4.05× (3.37–4.85) | 3.08× (2.78–3.42) | 2.70× (2.49–2.92) | 2.22× (2.04–2.42) | 3.21× (2.91–3.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.087 s | 92 | 45.4 | 1.38× (1.34–1.42) | 3.15× (2.97–3.33) | 2.80× (2.71–2.90) | 2.66× (2.54–2.79) | 3.07× (2.94–3.22) | 3.43× (3.34–3.52) | 3.50× (3.39–3.62) | 1.34× (1.29–1.39) | 1.50× (1.46–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.056 s | 143 | 54.2 | 1.24× (1.17–1.32) | 2.72× (2.56–2.89) | 2.60× (2.43–2.78) | 3.19× (2.98–3.41) | 3.16× (2.89–3.45) | 5.21× (5.02–5.42) | 5.32× (4.99–5.67) | 2.03× (1.91–2.17) | 2.27× (2.16–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.046 s | 173 | 70.8 | 1.24× (1.17–1.31) | 2.50× (2.28–2.73) | 2.53× (2.31–2.76) | 2.77× (2.60–2.96) | 3.11× (2.95–3.26) | 6.31× (5.95–6.70) | 6.44× (6.03–6.88) | 2.46× (2.29–2.64) | 2.75× (2.57–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.092 s | 104 | 39.5 | 1.48× (1.42–1.55) | 3.59× (3.41–3.78) | 3.05× (2.90–3.20) | 2.97× (2.86–3.09) | 3.37× (3.22–3.52) | 4.35× (4.18–4.53) | 4.69× (4.53–4.85) | 1.62× (1.48–1.78) | 1.78× (1.71–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.059 s | 161 | 48.3 | 1.28× (1.20–1.37) | 3.11× (2.94–3.28) | 2.87× (2.66–3.10) | 3.74× (3.51–3.98) | 3.24× (3.06–3.44) | 6.59× (6.24–6.97) | 7.10× (6.73–7.50) | 2.46× (2.20–2.75) | 2.70× (2.53–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.050 s | 190 | 67.2 | 1.34× (1.29–1.38) | 2.66× (2.52–2.81) | 2.88× (2.51–3.30) | 3.33× (3.22–3.44) | 3.21× (3.03–3.40) | 7.90× (7.66–8.16) | 8.52× (8.20–8.85) | 2.94× (2.64–3.29) | 3.24× (3.12–3.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 976 | 4.9 | 0.62× (0.59–0.64) | 3.14× (2.93–3.37) | 4.33× (4.01–4.68) | 3.05× (2.93–3.18) | 6.78× (6.54–7.04) | 2.28× (2.21–2.36) | 2.32× (2.25–2.39) | 0.95× (0.92–0.97) | 2.03× (1.94–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 939 | 5.4 | 0.53× (0.47–0.60) | 2.83× (2.65–3.04) | 5.33× (5.04–5.65) | 3.16× (2.76–3.62) | 6.44× (6.08–6.83) | 2.11× (1.99–2.24) | 2.15× (2.00–2.30) | 0.87× (0.82–0.93) | 1.88× (1.74–2.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 913 | 5.1 | 0.50× (0.48–0.53) | 2.86× (2.70–3.04) | 5.15× (4.93–5.37) | 2.96× (2.83–3.10) | 6.77× (6.06–7.56) | 2.13× (2.03–2.22) | 2.16× (2.05–2.28) | 0.88× (0.85–0.91) | 1.89× (1.78–2.00) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.048 s | 281 | 15.0 | 1.74× (1.68–1.81) | 4.24× (4.00–4.49) | 4.79× (4.42–5.20) | 4.68× (4.53–4.84) | 5.03× (4.78–5.30) | 5.11× (4.86–5.37) | 4.81× (4.62–5.01) | 1.58× (1.43–1.74) | 1.81× (1.72–1.91) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.037 s | 362 | 20.7 | 1.28× (1.16–1.42) | 3.15× (2.83–3.52) | 4.05× (3.52–4.66) | 4.11× (3.65–4.63) | 4.04× (3.66–4.46) | 6.47× (5.78–7.23) | 6.09× (5.57–6.67) | 2.00× (1.71–2.33) | 2.29× (2.08–2.52) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.032 s | 413 | 31.8 | 1.35× (1.29–1.40) | 2.90× (2.58–3.25) | 3.65× (3.23–4.11) | 3.71× (3.45–3.98) | 4.02× (3.91–4.14) | 7.64× (7.31–7.99) | 7.20× (6.83–7.58) | 2.36× (2.08–2.67) | 2.70× (2.61–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.73× (2.65–2.81) | 6.90× (6.67–7.15) | 7.41× (7.10–7.73) | 8.19× (7.87–8.51) | 7.32× (7.04–7.61) | 7.54× (7.33–7.77) | 7.17× (6.95–7.40) | 2.16× (2.10–2.23) | 2.46× (2.39–2.53) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.81× (2.64–2.99) | 6.23× (5.95–6.51) | 7.35× (6.68–8.09) | 7.53× (7.20–7.88) | 6.33× (6.05–6.62) | 6.46× (6.25–6.66) | 6.14× (5.98–6.30) | 1.85× (1.80–1.91) | 2.10× (2.05–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.75× (2.57–2.95) | 4.85× (4.40–5.35) | 5.74× (5.01–6.57) | 5.80× (5.29–6.36) | 4.91× (4.34–5.55) | 4.06× (3.68–4.47) | 3.86× (3.53–4.22) | 1.16× (1.07–1.26) | 1.32× (1.22–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.59–0.65) | 3.05× (2.92–3.18) | 4.17× (3.43–5.06) | 3.40× (3.23–3.59) | 5.84× (5.63–6.07) | 2.55× (2.44–2.67) | 2.55× (2.44–2.67) | 1.04× (0.98–1.11) | 1.82× (1.72–1.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.54–0.68) | 2.73× (2.60–2.87) | 4.75× (4.53–4.98) | 3.16× (3.01–3.32) | 5.35× (5.08–5.63) | 2.36× (2.23–2.50) | 2.36× (2.24–2.49) | 0.97× (0.91–1.02) | 1.68× (1.59–1.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.50–0.63) | 2.90× (2.64–3.19) | 4.78× (4.34–5.26) | 3.15× (2.89–3.44) | 5.38× (4.93–5.87) | 2.35× (2.15–2.57) | 2.35× (2.16–2.56) | 0.96× (0.84–1.10) | 1.68× (1.53–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.56× (2.41–2.72) | 2.93× (2.72–3.15) | 3.64× (3.35–3.97) | 4.03× (3.77–4.30) | 4.80× (4.28–5.39) | 2.73× (2.49–2.99) | 2.40× (2.22–2.60) | 1.99× (1.86–2.12) | 2.60× (2.43–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.07× (1.94–2.20) | 2.31× (2.11–2.53) | 2.96× (2.83–3.10) | 3.05× (2.82–3.29) | 3.53× (3.28–3.79) | 1.92× (1.71–2.16) | 1.69× (1.56–1.84) | 1.40× (1.28–1.53) | 1.83× (1.67–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.83× (1.75–1.92) | 1.57× (1.45–1.69) | 1.93× (1.84–2.02) | 1.83× (1.70–1.97) | 2.13× (1.80–2.51) | 0.97× (0.89–1.06) | 0.86× (0.80–0.92) | 0.71× (0.66–0.76) | 0.93× (0.86–0.99) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.33–1.41) | 3.05× (2.88–3.23) | 2.70× (2.62–2.79) | 2.63× (2.51–2.75) | 3.03× (2.90–3.17) | 3.38× (3.29–3.48) | 3.45× (3.34–3.56) | 1.33× (1.28–1.38) | 1.42× (1.39–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.22× (1.17–1.26) | 2.65× (2.52–2.79) | 2.35× (2.23–2.49) | 2.95× (2.79–3.12) | 2.71× (2.56–2.88) | 2.82× (2.71–2.93) | 2.88× (2.71–3.06) | 1.11× (1.05–1.18) | 1.19× (1.13–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.35–1.47) | 2.78× (2.51–3.07) | 2.31× (2.10–2.53) | 2.72× (2.60–2.85) | 2.77× (2.64–2.90) | 2.15× (2.04–2.26) | 2.19× (2.08–2.31) | 0.85× (0.80–0.89) | 0.90× (0.86–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.42–1.55) | 3.51× (3.32–3.70) | 2.95× (2.81–3.09) | 2.94× (2.83–3.06) | 3.33× (3.19–3.48) | 4.30× (4.13–4.47) | 4.63× (4.48–4.79) | 1.62× (1.47–1.78) | 1.72× (1.65–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.29–1.37) | 3.19× (3.07–3.31) | 2.69× (2.53–2.86) | 3.56× (3.39–3.75) | 2.99× (2.85–3.14) | 3.57× (3.46–3.69) | 3.85× (3.71–4.00) | 1.34× (1.22–1.47) | 1.43× (1.38–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.50× (1.44–1.56) | 2.98× (2.80–3.17) | 2.60× (2.17–3.11) | 3.16× (2.99–3.34) | 2.95× (2.75–3.18) | 2.58× (2.51–2.65) | 2.77× (2.69–2.86) | 0.97× (0.88–1.07) | 1.03× (1.00–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.65–0.70) | 2.49× (2.39–2.59) | 3.32× (3.16–3.49) | 2.75× (2.65–2.85) | 5.90× (5.70–6.11) | 2.08× (2.02–2.15) | 2.12× (2.06–2.18) | 0.96× (0.92–1.00) | 1.45× (1.38–1.51) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.54–0.66) | 2.30× (2.15–2.45) | 4.05× (3.82–4.29) | 2.83× (2.48–3.25) | 5.50× (5.15–5.87) | 1.88× (1.76–2.00) | 1.91× (1.78–2.06) | 0.86× (0.81–0.91) | 1.30× (1.21–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.54–0.59) | 2.52× (2.39–2.65) | 4.08× (3.90–4.26) | 2.79× (2.66–2.93) | 6.04× (5.42–6.74) | 1.99× (1.91–2.07) | 2.02× (1.92–2.13) | 0.91× (0.87–0.95) | 1.38× (1.30–1.46) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.72× (1.66–1.78) | 4.04× (3.82–4.26) | 4.52× (4.17–4.89) | 4.56× (4.42–4.70) | 4.89× (4.65–5.14) | 4.97× (4.72–5.23) | 4.68× (4.50–4.86) | 1.56× (1.41–1.72) | 1.65× (1.57–1.74) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.37× (1.31–1.42) | 3.25× (3.07–3.43) | 3.88× (3.61–4.17) | 4.08× (3.86–4.32) | 3.71× (3.50–3.93) | 3.67× (3.46–3.89) | 3.46× (3.30–3.62) | 1.15× (1.02–1.30) | 1.22× (1.15–1.29) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.50× (1.42–1.58) | 3.01× (2.72–3.34) | 3.37× (2.94–3.87) | 3.55× (3.33–3.78) | 3.26× (3.08–3.46) | 2.44× (2.32–2.56) | 2.29× (2.19–2.41) | 0.76× (0.68–0.86) | 0.81× (0.77–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.98× (0.97–0.98) | 0.79× (0.77–0.80) | 11.04× (11.01–11.07) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.31–0.34) | 0.46× (0.41–0.51) | 0.67× (0.64–0.70) | 0.57× (0.54–0.59) | 5.35× (5.16–5.55) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.32–0.35) | 0.54× (0.44–0.65) | 0.61× (0.59–0.64) | 0.52× (0.49–0.55) | 3.50× (3.33–3.67) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.37–4.40) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.05× (8.03–8.08) | 0.66× (0.65–0.66) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.68× (0.65–0.71) | 0.63× (0.57–0.70) | 1.56× (1.52–1.60) | 0.78× (0.76–0.80) | 5.66× (5.53–5.80) | 0.46× (0.45–0.47) | 0.49× (0.47–0.50) | 0.52× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.62–0.67) | 0.71× (0.67–0.76) | 1.14× (1.11–1.18) | 0.70× (0.68–0.73) | 3.35× (3.24–3.46) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.50–0.54) | 0.40× (0.36–0.43) | 0.93× (0.90–0.97) | 0.74× (0.71–0.78) | 3.22× (3.10–3.34) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.81× (0.77–0.84) | 2.53× (2.47–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.37× (0.34–0.40) | 0.85× (0.83–0.88) | 0.73× (0.69–0.76) | 2.91× (2.83–3.00) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.48× (0.45–0.51) | 0.80× (0.79–0.82) | 0.82× (0.78–0.86) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.21) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.30–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.39–0.39) | 1.41× (1.41–1.42) | 0.68× (0.68–0.68) | 6.43× (6.41–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.43–0.47) | 0.41× (0.37–0.44) | 1.06× (1.02–1.10) | 0.69× (0.67–0.71) | 3.35× (3.25–3.44) | 0.21× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.46× (0.42–0.51) | 0.90× (0.87–0.93) | 0.74× (0.73–0.76) | 2.27× (2.21–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 642 | 1.57 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,021 | 1.87 | 1.17× (1.13–1.21) | 80% | 1.05× (1.02–1.09) | 1.20× (1.13–1.29) |
| 4: all 4 CPUs, sharing SMT siblings | 1,210 | 2.89 | 1.86× (1.68–2.06) | 46% | 1.31× (1.23–1.39) | 1.88× (1.73–2.03) |

**Best:** mibz2 at 4 threads (1,210 MB/s); lbzip2 at 4 threads (281 MB/s); pbz2 at 4 threads (421 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.21, lbzip2 1t 0.08, lbzip2 2t 0.28, lbzip2 4t 0.42, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.46 | 0.01 | 0.07 | 0.173 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4073 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.00 | 14.0% |
| MTF + RLE2 | 1.43 | 20.1% |
| T-vector build | 1.28 | 18.0% |
| walk (the one each build uses) | 2.98 | 42.0% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.25 | 3.5% |
| **total** | 7.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.308 | 0.734 | (in its total) | 2.40× (2.34–2.45) |  |
| walk | 0.682 | 1.647 | (in its total) | 2.44× (2.38–2.50) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.030 | 0.035 | (in its total) | 1.16× (1.09–1.22) |  |
| inverse BWT, total | 1.020 | 2.415 | 1.351 | 2.39× (2.33–2.45) | 1.33× (1.30–1.35) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 47m45s ± 5m34s |
| 4 threads | 25m59s ± 4m29s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m04s ± 3m21s |
| the scanner alone (its serial part) | 9m27s ± 1m01s |
| the workers alone, over 4 | 17m04s ± 3m21s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 2.24 ns a byte of output at 4 threads (CPU, summed over them), 1.25 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,079 | 13,357,122 | \`cc146a11acfb9941…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4072.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
