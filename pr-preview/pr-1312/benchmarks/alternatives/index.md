# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1312/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`e5b7980`](<https://github.com/lmmx/mibz2/commit/e5b7980e34969ed0ae263e1cc96d0d52245e16c0>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46109950073>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.369 s | 568 | 6.9 | 3.66× (3.65–3.67) | 8.40× (8.36–8.44) | 8.94× (8.90–8.97) | 9.88× (9.87–9.90) | 8.95× (8.90–9.01) | 9.39× (9.33–9.44) | 9.18× (9.15–9.22) | 2.86× (2.85–2.87) | 2.55× (2.53–2.56) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.225 s | 934 | 8.0 | 3.40× (3.35–3.46) | 7.28× (7.11–7.45) | 8.82× (8.65–9.00) | 8.78× (8.62–8.94) | 7.94× (7.81–8.08) | 15.51× (15.29–15.72) | 15.17× (14.93–15.41) | 4.73× (4.65–4.81) | 4.21× (4.14–4.28) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,159 | 12.7 | 3.54× (3.50–3.58) | 6.52× (6.41–6.63) | 7.66× (7.52–7.80) | 7.56× (7.46–7.65) | 7.37× (7.28–7.46) | 19.10× (18.91–19.29) | 18.69× (18.52–18.85) | 5.82× (5.76–5.88) | 5.19× (5.14–5.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,242 | 3.4 | 0.64× (0.63–0.65) | 3.18× (3.14–3.22) | 4.54× (4.47–4.60) | 3.74× (3.70–3.79) | 6.40× (6.30–6.50) | 2.85× (2.81–2.89) | 2.88× (2.84–2.92) | 1.12× (1.09–1.14) | 1.29× (1.27–1.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,170 | 3.8 | 0.54× (0.53–0.56) | 2.98× (2.89–3.09) | 6.03× (5.82–6.24) | 3.56× (3.45–3.68) | 6.03× (5.87–6.21) | 2.68× (2.59–2.76) | 2.70× (2.62–2.79) | 1.05× (1.02–1.08) | 1.21× (1.16–1.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,153 | 3.8 | 0.54× (0.53–0.56) | 2.96× (2.89–3.04) | 5.85× (5.69–6.00) | 3.55× (3.46–3.64) | 5.99× (5.86–6.12) | 2.65× (2.59–2.71) | 2.68× (2.61–2.74) | 1.04× (1.01–1.07) | 1.19× (1.17–1.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 653 | 6.3 | 2.89× (2.85–2.92) | 3.39× (3.36–3.42) | 4.08× (3.99–4.17) | 4.37× (4.33–4.41) | 4.80× (4.75–4.85) | 3.23× (3.21–3.26) | 2.99× (2.96–3.02) | 2.04× (1.99–2.08) | 2.20× (2.07–2.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,037 | 7.4 | 2.58× (2.40–2.77) | 3.16× (2.93–3.40) | 4.34× (4.01–4.70) | 3.90× (3.60–4.22) | 5.23× (4.85–5.63) | 5.00× (4.64–5.38) | 4.63× (4.30–4.98) | 3.15× (2.92–3.39) | 3.40× (3.10–3.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,104 | 13.4 | 2.17× (2.08–2.27) | 2.30× (2.19–2.41) | 3.29× (3.07–3.51) | 2.66× (2.52–2.80) | 4.56× (4.36–4.76) | 5.33× (5.11–5.56) | 4.93× (4.73–5.15) | 3.36× (3.22–3.49) | 3.62× (3.41–3.85) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.104 s | 77 | 51.3 | 1.50× (1.49–1.50) | 3.31× (3.30–3.33) | 3.01× (2.97–3.06) | 3.08× (3.06–3.10) | 3.36× (3.35–3.37) | 3.75× (3.73–3.77) | 3.94× (3.92–3.96) | 1.37× (1.37–1.38) | 1.36× (1.35–1.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 55.3 | 1.48× (1.42–1.54) | 3.27× (3.18–3.36) | 3.21× (3.08–3.35) | 3.90× (3.76–4.05) | 3.58× (3.44–3.73) | 6.41× (6.16–6.67) | 6.74× (6.47–7.01) | 2.35× (2.26–2.45) | 2.33× (2.24–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.048 s | 166 | 74.3 | 1.57× (1.50–1.64) | 3.18× (3.09–3.27) | 3.01× (2.90–3.12) | 3.73× (3.61–3.85) | 3.66× (3.55–3.78) | 7.92× (7.67–8.18) | 8.33× (8.07–8.58) | 2.90× (2.82–2.99) | 2.88× (2.80–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 47.8 | 1.54× (1.52–1.55) | 3.66× (3.63–3.69) | 3.08× (3.05–3.11) | 3.24× (3.22–3.26) | 3.56× (3.54–3.59) | 4.41× (4.39–4.44) | 4.77× (4.74–4.81) | 1.59× (1.58–1.60) | 1.65× (1.64–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.066 s | 146 | 52.0 | 1.46× (1.40–1.52) | 3.52× (3.36–3.68) | 3.25× (3.09–3.42) | 4.31× (4.15–4.47) | 3.66× (3.52–3.80) | 7.61× (7.34–7.90) | 8.23× (7.94–8.54) | 2.74× (2.65–2.85) | 2.85× (2.75–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 170 | 73.0 | 1.62× (1.56–1.68) | 3.34× (3.21–3.48) | 3.08× (2.98–3.18) | 4.07× (3.98–4.17) | 3.61× (3.54–3.68) | 9.18× (9.02–9.35) | 9.93× (9.74–10.12) | 3.31× (3.25–3.37) | 3.44× (3.38–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 935 | 4.8 | 0.70× (0.68–0.71) | 2.88× (2.82–2.95) | 3.97× (3.89–4.06) | 3.22× (3.15–3.29) | 6.88× (6.73–7.02) | 2.46× (2.42–2.51) | 2.50× (2.45–2.56) | 1.04× (1.00–1.07) | 0.98× (0.96–1.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 858 | 5.5 | 0.52× (0.51–0.54) | 2.63× (2.57–2.69) | 5.35× (5.23–5.48) | 2.98× (2.92–3.05) | 6.40× (6.27–6.53) | 2.24× (2.20–2.29) | 2.28× (2.23–2.33) | 0.94× (0.92–0.97) | 0.90× (0.88–0.92) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 840 | 5.4 | 0.52× (0.51–0.53) | 2.60× (2.57–2.63) | 5.09× (4.84–5.36) | 2.94× (2.92–2.97) | 6.28× (6.17–6.40) | 2.21× (2.18–2.24) | 2.25× (2.22–2.28) | 0.93× (0.91–0.95) | 0.88× (0.87–0.89) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.055 s | 271 | 14.8 | 2.06× (2.05–2.07) | 4.39× (4.35–4.42) | 4.95× (4.92–4.99) | 5.02× (4.99–5.05) | 5.13× (5.09–5.16) | 5.22× (5.18–5.25) | 5.20× (5.17–5.24) | 1.65× (1.64–1.66) | 1.64× (1.63–1.66) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.036 s | 418 | 17.8 | 1.73× (1.69–1.77) | 3.76× (3.60–3.94) | 4.84× (4.77–4.92) | 5.01× (4.91–5.11) | 4.81× (4.75–4.88) | 8.01× (7.89–8.13) | 7.99× (7.88–8.11) | 2.54× (2.50–2.57) | 2.52× (2.49–2.56) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.031 s | 483 | 26.9 | 1.75× (1.69–1.82) | 3.14× (3.01–3.28) | 4.32× (4.12–4.52) | 4.53× (4.40–4.68) | 4.58× (4.43–4.73) | 9.25× (9.01–9.50) | 9.23× (8.98–9.48) | 2.93× (2.85–3.01) | 2.91× (2.84–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.65× (3.64–3.66) | 8.34× (8.30–8.38) | 8.89× (8.85–8.92) | 9.84× (9.83–9.85) | 8.92× (8.86–8.97) | 9.34× (9.29–9.40) | 9.14× (9.10–9.18) | 2.85× (2.84–2.86) | 2.53× (2.51–2.54) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.27× (3.26–3.29) | 7.46× (7.39–7.54) | 8.27× (8.22–8.31) | 9.02× (8.97–9.08) | 7.82× (7.77–7.87) | 8.01× (7.96–8.07) | 7.84× (7.79–7.89) | 2.44× (2.43–2.46) | 2.17× (2.15–2.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.34× (3.26–3.42) | 6.83× (6.70–6.97) | 7.25× (7.11–7.40) | 7.95× (7.82–8.08) | 7.09× (6.97–7.22) | 5.11× (5.01–5.22) | 5.00× (4.90–5.10) | 1.56× (1.53–1.59) | 1.38× (1.36–1.41) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.67–0.68) | 2.90× (2.86–2.94) | 3.89× (3.83–3.95) | 3.48× (3.43–3.53) | 5.88× (5.79–5.98) | 2.67× (2.63–2.71) | 2.70× (2.66–2.74) | 1.11× (1.09–1.13) | 0.83× (0.81–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.69× (2.61–2.78) | 5.15× (5.00–5.31) | 3.24× (3.14–3.35) | 5.41× (5.26–5.57) | 2.45× (2.37–2.52) | 2.47× (2.39–2.55) | 1.01× (0.98–1.04) | 0.76× (0.74–0.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.05× (2.97–3.14) | 5.15× (5.06–5.24) | 3.28× (3.21–3.36) | 5.46× (5.36–5.56) | 2.47× (2.42–2.52) | 2.49× (2.44–2.54) | 1.02× (0.99–1.05) | 0.76× (0.75–0.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.76–2.83) | 3.14× (3.11–3.17) | 3.70× (3.65–3.75) | 4.20× (4.15–4.24) | 4.61× (4.55–4.66) | 3.12× (3.09–3.15) | 2.89× (2.86–2.93) | 1.99× (1.95–2.03) | 1.98× (1.95–2.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.57× (2.49–2.64) | 2.88× (2.79–2.98) | 3.48× (3.40–3.56) | 3.69× (3.60–3.78) | 4.05× (3.94–4.16) | 2.65× (2.59–2.71) | 2.46× (2.41–2.52) | 1.69× (1.63–1.75) | 1.68× (1.62–1.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.23× (2.17–2.29) | 1.95× (1.90–2.00) | 2.39× (2.31–2.47) | 2.35× (2.28–2.43) | 2.45× (2.38–2.52) | 1.48× (1.44–1.52) | 1.37× (1.33–1.41) | 0.94× (0.91–0.97) | 0.94× (0.91–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 3.21× (3.19–3.23) | 2.94× (2.90–2.97) | 3.05× (3.03–3.07) | 3.32× (3.31–3.34) | 3.71× (3.69–3.72) | 3.89× (3.87–3.91) | 1.37× (1.36–1.37) | 1.34× (1.33–1.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.46× (1.45–1.47) | 3.19× (3.16–3.22) | 2.91× (2.88–2.94) | 3.69× (3.66–3.72) | 3.21× (3.18–3.24) | 3.44× (3.42–3.47) | 3.62× (3.59–3.65) | 1.27× (1.26–1.28) | 1.24× (1.23–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.69× (1.63–1.76) | 3.20× (3.06–3.35) | 2.77× (2.71–2.83) | 3.61× (3.51–3.70) | 3.21× (3.14–3.29) | 2.54× (2.48–2.61) | 2.67× (2.61–2.74) | 0.94× (0.92–0.96) | 0.92× (0.90–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.54) | 3.57× (3.54–3.60) | 3.03× (3.00–3.05) | 3.21× (3.18–3.23) | 3.52× (3.50–3.55) | 4.36× (4.33–4.39) | 4.72× (4.68–4.75) | 1.58× (1.57–1.59) | 1.57× (1.56–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.42–1.50) | 3.44× (3.34–3.54) | 2.94× (2.85–3.03) | 3.97× (3.85–4.08) | 3.32× (3.22–3.41) | 3.96× (3.84–4.08) | 4.28× (4.16–4.40) | 1.44× (1.40–1.48) | 1.43× (1.39–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.70–1.83) | 3.46× (3.32–3.61) | 2.69× (2.61–2.77) | 3.76× (3.67–3.85) | 3.25× (3.18–3.32) | 2.85× (2.78–2.92) | 3.08× (3.01–3.16) | 1.03× (1.01–1.06) | 1.03× (1.00–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.74× (0.72–0.75) | 2.50× (2.44–2.56) | 3.49× (3.40–3.58) | 2.92× (2.86–2.98) | 6.09× (5.94–6.23) | 2.27× (2.22–2.32) | 2.30× (2.25–2.35) | 1.03× (1.00–1.06) | 0.77× (0.75–0.79) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.17× (2.12–2.21) | 4.17× (4.07–4.27) | 2.64× (2.59–2.70) | 5.47× (5.36–5.58) | 1.98× (1.96–2.01) | 2.01× (1.98–2.05) | 0.90× (0.88–0.92) | 0.67× (0.66–0.69) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.50× (2.42–2.59) | 4.14× (4.05–4.22) | 2.70× (2.67–2.73) | 5.54× (5.45–5.64) | 2.02× (1.99–2.04) | 2.05× (2.02–2.07) | 0.92× (0.89–0.94) | 0.68× (0.68–0.69) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.03× (2.02–2.04) | 4.18× (4.14–4.22) | 4.77× (4.73–4.80) | 4.90× (4.87–4.93) | 5.00× (4.97–5.04) | 5.09× (5.05–5.13) | 5.08× (5.04–5.11) | 1.63× (1.62–1.64) | 1.51× (1.50–1.52) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.79× (1.78–1.81) | 3.66× (3.60–3.72) | 4.37× (4.34–4.39) | 4.60× (4.56–4.65) | 4.29× (4.26–4.32) | 4.23× (4.20–4.27) | 4.22× (4.19–4.26) | 1.36× (1.35–1.37) | 1.26× (1.25–1.27) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.91× (1.88–1.95) | 3.26× (3.23–3.29) | 3.64× (3.60–3.69) | 4.00× (3.97–4.03) | 3.71× (3.66–3.75) | 2.80× (2.77–2.82) | 2.79× (2.76–2.82) | 0.90× (0.89–0.91) | 0.83× (0.82–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.26–0.29) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 11.00× (10.96–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.37× (0.36–0.37) | 0.47× (0.38–0.60) | 0.75× (0.73–0.77) | 0.63× (0.61–0.65) | 6.00× (5.82–6.18) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.35–0.38) | 0.58× (0.50–0.68) | 0.65× (0.63–0.67) | 0.57× (0.54–0.60) | 3.80× (3.66–3.95) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.92) | 4.83× (4.83–4.84) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.92–0.92) | 4.83× (4.82–4.83) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.69) | 0.66× (0.66–0.66) | 7.95× (7.93–7.97) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.70–0.72) | 0.70× (0.62–0.78) | 1.72× (1.68–1.75) | 0.86× (0.84–0.87) | 6.21× (6.09–6.33) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.68–0.71) | 0.75× (0.68–0.83) | 1.24× (1.21–1.26) | 0.77× (0.75–0.79) | 3.67× (3.59–3.75) | 0.29× (0.28–0.30) | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.14× (5.14–5.15) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.43) | 0.90× (0.88–0.93) | 0.71× (0.68–0.74) | 3.11× (3.02–3.21) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.67) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.82× (0.79–0.84) | 2.53× (2.48–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.21× (5.20–5.22) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.36× (0.33–0.39) | 0.84× (0.81–0.87) | 0.69× (0.65–0.73) | 2.86× (2.76–2.97) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.59× (0.58–0.60) | 0.46× (0.43–0.48) | 0.79× (0.78–0.80) | 0.80× (0.77–0.82) | 2.28× (2.26–2.30) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.06× (1.06–1.06) | 0.75× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.86× (1.86–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.23) | 1.86× (1.85–1.86) | 0.89× (0.89–0.90) | 6.16× (6.13–6.18) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.39) | 0.66× (0.66–0.66) | 6.40× (6.39–6.42) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.41× (0.37–0.46) | 1.13× (1.10–1.16) | 0.73× (0.72–0.75) | 3.69× (3.62–3.76) | 0.22× (0.22–0.23) | 0.24× (0.23–0.25) | 0.37× (0.36–0.37) | 0.18× (0.17–0.18) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 55 MB | 0.51× (0.50–0.52) | 0.47× (0.43–0.51) | 0.93× (0.91–0.95) | 0.76× (0.74–0.78) | 2.38× (2.35–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 568 | 1.77 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 934 | 2.06 | 1.17× (1.16–1.17) | 83% | 1.04× (1.03–1.06) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,159 | 3.27 | 1.83× (1.79–1.86) | 51% | 1.50× (1.49–1.51) | 1.67× (1.64–1.70) |

**Best:** mibz2 at 4 threads (1,159 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (326 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 0.97, lbzip2 1t 0.04, lbzip2 2t 0.16, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.54 | 0.02 | 0.07 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3888 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.79 | 7.6% |
| MTF + RLE2 | 1.61 | 15.6% |
| T-vector build | 1.56 | 15.1% |
| walk (the one each build uses) | 6.02 | 58.1% |
| of which: segment assembly | 0.31 | 3.0% |
| RLE1 | 0.24 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.37 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.368 | 0.929 | (in its total) | 2.52× (2.51–2.53) |  |
| walk | 1.458 | 2.761 | (in its total) | 1.89× (1.89–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.072 | (in its total) | 0.92× (0.91–0.92) |  |
| inverse BWT, total | 1.905 | 3.763 | 1.977 | 1.97× (1.97–1.98) | 1.04× (1.03–1.04) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 53m50s ± 6m33s |
| 4 threads | 26m27s ± 3m43s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m34s ± 2m47s |
| the scanner alone (its serial part) | 7m08s ± 19.7s |
| the workers alone, over 4 | 19m34s ± 2m47s |

At 4 threads the workers bound it. Per byte: the scanner 4.1 ns a byte of bz2, the workers 2.57 ns a byte of output at 4 threads (CPU, summed over them), 1.53 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,260,864 | 15,030,132 | \`de55d3765b0096e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3888.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1018-aws; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
