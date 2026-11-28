# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1311/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`0e9b59f`](<https://github.com/lmmx/mibz2/commit/0e9b59fff879456941b72de3cb6c0681dd583266>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46106787841>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.381 s | 550 | 6.7 | 3.20× (3.18–3.21) | 7.29× (7.27–7.31) | 7.54× (7.52–7.56) | 8.60× (8.58–8.63) | 7.55× (7.53–7.57) | 7.93× (7.89–7.96) | 7.65× (7.63–7.67) | 2.36× (2.36–2.37) | 2.62× (2.61–2.63) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.212 s | 990 | 7.2 | 3.35× (3.21–3.50) | 6.74× (6.46–7.03) | 7.74× (7.46–8.03) | 8.15× (7.84–8.48) | 7.15× (6.88–7.43) | 14.09× (13.54–14.66) | 13.60× (13.08–14.13) | 4.20× (4.04–4.37) | 4.65× (4.47–4.85) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.174 s | 1,208 | 11.3 | 3.24× (3.19–3.29) | 4.87× (4.79–4.95) | 5.92× (5.84–5.99) | 5.93× (5.83–6.04) | 5.81× (5.74–5.88) | 17.33× (17.19–17.47) | 16.72× (16.59–16.85) | 5.17× (5.12–5.22) | 5.72× (5.68–5.77) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,140 | 3.6 | 0.57× (0.56–0.59) | 3.44× (3.35–3.52) | 4.60× (4.48–4.71) | 3.57× (3.49–3.66) | 6.08× (5.92–6.23) | 2.67× (2.60–2.74) | 2.67× (2.60–2.73) | 0.95× (0.92–0.98) | 1.82× (1.66–2.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,090 | 3.9 | 0.52× (0.51–0.53) | 2.78× (2.72–2.83) | 5.56× (5.46–5.67) | 3.45× (3.39–3.51) | 5.77× (5.66–5.88) | 2.58× (2.53–2.62) | 2.57× (2.53–2.62) | 0.91× (0.89–0.94) | 1.76× (1.63–1.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,064 | 3.8 | 0.52× (0.51–0.53) | 2.74× (2.70–2.78) | 5.48× (5.40–5.55) | 3.41× (3.36–3.45) | 5.80× (5.70–5.90) | 2.53× (2.50–2.57) | 2.53× (2.50–2.57) | 0.90× (0.88–0.92) | 1.73× (1.59–1.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 772 | 5.1 | 2.80× (2.78–2.81) | 3.21× (3.18–3.23) | 3.93× (3.75–4.12) | 4.27× (4.24–4.31) | 4.63× (4.57–4.70) | 2.89× (2.87–2.91) | 2.37× (2.35–2.38) | 2.17× (2.15–2.18) | 3.20× (3.18–3.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,202 | 6.1 | 2.32× (2.08–2.58) | 2.98× (2.69–3.31) | 4.39× (3.96–4.88) | 3.75× (3.32–4.24) | 5.22× (4.70–5.81) | 4.28× (3.85–4.75) | 3.50× (3.15–3.88) | 3.20× (2.89–3.55) | 4.73× (4.26–5.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,143 | 11.9 | 2.14× (2.07–2.22) | 2.29× (2.12–2.48) | 3.33× (3.10–3.57) | 2.59× (2.47–2.70) | 4.72× (4.58–4.87) | 4.25× (4.13–4.37) | 3.47× (3.38–3.57) | 3.18× (3.09–3.27) | 4.69× (4.57–4.83) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.100 s | 80 | 46.9 | 1.51× (1.51–1.52) | 2.94× (2.92–2.97) | 2.65× (2.63–2.68) | 2.78× (2.77–2.79) | 3.07× (3.05–3.10) | 3.30× (3.28–3.31) | 3.45× (3.43–3.46) | 1.37× (1.37–1.38) | 1.52× (1.52–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 52.2 | 1.48× (1.32–1.66) | 2.78× (2.67–2.90) | 2.68× (2.60–2.75) | 3.47× (3.35–3.60) | 3.20× (3.11–3.29) | 5.41× (5.26–5.56) | 5.66× (5.49–5.83) | 2.25× (2.19–2.32) | 2.50× (2.42–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 162 | 68.4 | 1.50× (1.45–1.55) | 2.38× (2.29–2.47) | 2.45× (2.33–2.58) | 3.00× (2.93–3.07) | 3.25× (3.16–3.34) | 6.73× (6.56–6.90) | 7.04× (6.86–7.22) | 2.80× (2.73–2.88) | 3.11× (3.03–3.18) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.109 s | 88 | 42.7 | 1.66× (1.64–1.67) | 3.34× (3.30–3.38) | 2.90× (2.87–2.94) | 3.04× (3.03–3.06) | 3.36× (3.35–3.38) | 4.01× (3.99–4.02) | 4.41× (4.39–4.43) | 1.63× (1.62–1.63) | 1.95× (1.94–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.065 s | 147 | 48.2 | 1.53× (1.48–1.59) | 3.05× (2.98–3.13) | 2.85× (2.79–2.90) | 3.98× (3.92–4.05) | 3.42× (3.38–3.46) | 6.72× (6.65–6.79) | 7.40× (7.32–7.48) | 2.73× (2.70–2.76) | 3.26× (3.23–3.30) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 174 | 64.9 | 1.61× (1.56–1.66) | 2.59× (2.48–2.71) | 2.55× (2.47–2.64) | 3.44× (3.37–3.52) | 3.23× (3.16–3.30) | 7.96× (7.78–8.15) | 8.76× (8.57–8.96) | 3.23× (3.16–3.30) | 3.87× (3.78–3.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 4.9 | 0.64× (0.63–0.64) | 3.45× (3.22–3.70) | 4.46× (4.41–4.50) | 3.14× (3.11–3.17) | 6.78× (6.69–6.86) | 2.33× (2.31–2.35) | 2.38× (2.35–2.40) | 0.96× (0.94–0.98) | 1.78× (1.76–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 803 | 5.5 | 0.51× (0.51–0.52) | 2.47× (2.42–2.51) | 4.96× (4.87–5.05) | 2.94× (2.89–3.00) | 6.27× (6.19–6.35) | 2.15× (2.12–2.19) | 2.20× (2.16–2.24) | 0.89× (0.87–0.91) | 1.65× (1.62–1.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 772 | 5.6 | 0.50× (0.50–0.51) | 2.40× (2.37–2.43) | 4.81× (4.75–4.87) | 2.85× (2.82–2.89) | 6.07× (6.00–6.14) | 2.09× (2.06–2.12) | 2.13× (2.10–2.17) | 0.86× (0.84–0.88) | 1.60× (1.58–1.62) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.051 s | 293 | 13.1 | 2.06× (2.05–2.06) | 4.16× (4.15–4.17) | 4.67× (4.59–4.74) | 4.96× (4.94–4.98) | 5.08× (5.07–5.10) | 4.87× (4.85–4.89) | 5.01× (4.99–5.04) | 1.68× (1.68–1.69) | 1.97× (1.97–1.98) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.035 s | 424 | 16.4 | 1.62× (1.50–1.75) | 3.34× (3.17–3.51) | 4.08× (3.82–4.37) | 4.72× (4.44–5.02) | 4.44× (4.22–4.67) | 6.87× (6.54–7.22) | 7.07× (6.71–7.45) | 2.37× (2.25–2.50) | 2.78× (2.64–2.94) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.031 s | 484 | 25.4 | 1.67× (1.61–1.73) | 2.70× (2.64–2.75) | 3.64× (3.50–3.77) | 3.98× (3.84–4.12) | 4.20× (4.11–4.30) | 8.10× (7.93–8.27) | 8.34× (8.14–8.55) | 2.80× (2.74–2.86) | 3.28× (3.21–3.36) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.19× (3.17–3.20) | 7.25× (7.23–7.28) | 7.50× (7.48–7.52) | 8.57× (8.54–8.59) | 7.52× (7.50–7.54) | 7.89× (7.86–7.93) | 7.62× (7.59–7.64) | 2.36× (2.35–2.36) | 2.60× (2.59–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (3.05–3.07) | 6.91× (6.85–6.98) | 7.18× (7.16–7.21) | 8.42× (8.38–8.45) | 7.06× (7.04–7.09) | 7.38× (7.34–7.43) | 7.13× (7.10–7.15) | 2.21× (2.20–2.22) | 2.43× (2.42–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.99–3.06) | 5.12× (5.08–5.16) | 5.45× (5.41–5.49) | 6.25× (6.20–6.30) | 5.37× (5.33–5.42) | 4.70× (4.66–4.74) | 4.53× (4.49–4.57) | 1.40× (1.39–1.41) | 1.55× (1.53–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.62) | 2.93× (2.86–3.01) | 3.89× (3.80–3.99) | 3.35× (3.27–3.43) | 5.64× (5.50–5.78) | 2.52× (2.46–2.59) | 2.52× (2.47–2.58) | 0.95× (0.92–0.98) | 1.64× (1.60–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.63× (2.59–2.67) | 4.69× (4.61–4.77) | 3.15× (3.11–3.19) | 5.18× (5.10–5.27) | 2.35× (2.32–2.39) | 2.35× (2.31–2.39) | 0.89× (0.87–0.91) | 1.53× (1.51–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.55) | 2.89× (2.86–2.93) | 4.70× (4.61–4.78) | 3.17× (3.13–3.21) | 5.30× (5.21–5.39) | 2.36× (2.33–2.40) | 2.36× (2.33–2.40) | 0.89× (0.87–0.91) | 1.54× (1.52–1.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.67–2.70) | 2.95× (2.93–2.97) | 3.52× (3.40–3.64) | 4.07× (4.04–4.10) | 4.41× (4.35–4.46) | 2.78× (2.75–2.80) | 2.28× (2.26–2.30) | 2.09× (2.08–2.11) | 2.74× (2.72–2.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.43× (2.38–2.48) | 2.72× (2.65–2.79) | 3.59× (3.51–3.68) | 3.59× (3.52–3.67) | 3.86× (3.76–3.96) | 2.33× (2.28–2.38) | 1.92× (1.88–1.95) | 1.76× (1.72–1.79) | 2.30× (2.25–2.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.14× (2.04–2.23) | 1.84× (1.79–1.90) | 2.42× (2.38–2.45) | 2.22× (2.16–2.28) | 2.29× (2.26–2.33) | 1.19× (1.17–1.21) | 0.98× (0.97–0.99) | 0.90× (0.89–0.91) | 1.18× (1.16–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.51× (1.50–1.51) | 2.83× (2.82–2.85) | 2.60× (2.58–2.62) | 2.75× (2.74–2.76) | 3.04× (3.01–3.06) | 3.26× (3.24–3.27) | 3.41× (3.39–3.42) | 1.37× (1.36–1.37) | 1.50× (1.50–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.38–1.43) | 2.67× (2.61–2.73) | 2.39× (2.34–2.43) | 3.23× (3.17–3.30) | 2.77× (2.72–2.82) | 2.91× (2.85–2.97) | 3.04× (2.98–3.10) | 1.22× (1.20–1.24) | 1.34× (1.31–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.64× (1.62–1.67) | 2.55× (2.50–2.60) | 2.14× (2.10–2.18) | 2.94× (2.87–3.01) | 2.76× (2.70–2.82) | 2.24× (2.19–2.29) | 2.34× (2.29–2.39) | 0.94× (0.92–0.96) | 1.03× (1.01–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.65× (1.63–1.66) | 3.25× (3.22–3.28) | 2.85× (2.81–2.88) | 3.02× (3.00–3.03) | 3.33× (3.31–3.34) | 3.96× (3.95–3.98) | 4.36× (4.34–4.38) | 1.62× (1.61–1.62) | 1.87× (1.86–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.53× (1.48–1.59) | 3.02× (2.99–3.05) | 2.60× (2.58–2.63) | 3.65× (3.62–3.69) | 3.01× (2.98–3.04) | 3.51× (3.48–3.55) | 3.87× (3.83–3.91) | 1.43× (1.42–1.45) | 1.66× (1.64–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.82× (1.79–1.86) | 2.81× (2.74–2.89) | 2.26× (2.21–2.30) | 3.22× (3.16–3.28) | 2.93× (2.89–2.97) | 2.61× (2.56–2.65) | 2.87× (2.82–2.92) | 1.06× (1.05–1.08) | 1.23× (1.21–1.25) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.50× (2.47–2.53) | 3.43× (3.41–3.46) | 2.86× (2.85–2.88) | 6.04× (5.97–6.10) | 2.16× (2.14–2.17) | 2.20× (2.18–2.22) | 0.96× (0.95–0.98) | 1.39× (1.38–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.17× (2.14–2.20) | 3.80× (3.74–3.86) | 2.64× (2.59–2.69) | 5.38× (5.32–5.44) | 1.91× (1.88–1.94) | 1.95× (1.92–1.99) | 0.85× (0.84–0.87) | 1.23× (1.21–1.25) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.36× (2.33–2.40) | 3.75× (3.69–3.82) | 2.63× (2.60–2.67) | 5.38× (5.32–5.44) | 1.92× (1.89–1.94) | 1.96× (1.93–1.99) | 0.86× (0.84–0.87) | 1.23× (1.22–1.25) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.02× (2.02–2.03) | 4.01× (4.00–4.02) | 4.44× (4.43–4.46) | 4.83× (4.81–4.84) | 4.95× (4.94–4.96) | 4.75× (4.72–4.77) | 4.88× (4.86–4.90) | 1.66× (1.66–1.67) | 1.85× (1.84–1.85) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.72× (1.68–1.76) | 3.33× (3.30–3.37) | 3.72× (3.69–3.75) | 4.41× (4.38–4.45) | 4.03× (4.01–4.06) | 3.79× (3.75–3.82) | 3.90× (3.87–3.92) | 1.33× (1.32–1.33) | 1.47× (1.47–1.48) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.75× (1.71–1.79) | 2.65× (2.58–2.72) | 2.97× (2.90–3.04) | 3.37× (3.32–3.43) | 3.17× (3.10–3.24) | 2.45× (2.40–2.50) | 2.52× (2.47–2.58) | 0.86× (0.84–0.88) | 0.95× (0.93–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.98) | 0.79× (0.78–0.79) | 11.02× (10.99–11.04) | 0.26× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.33–0.37) | 0.44× (0.37–0.53) | 0.71× (0.68–0.75) | 0.61× (0.58–0.64) | 5.76× (5.47–6.05) | 0.14× (0.13–0.14) | 0.15× (0.14–0.16) | 0.22× (0.21–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.37–0.39) | 0.61× (0.49–0.74) | 0.67× (0.65–0.70) | 0.59× (0.56–0.61) | 3.94× (3.78–4.09) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.14–0.16) | 0.08× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.92) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.92–0.92) | 4.82× (4.81–4.83) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.66–0.67) | 7.97× (7.93–8.01) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.66–0.73) | 0.70× (0.64–0.76) | 1.64× (1.57–1.72) | 0.83× (0.79–0.87) | 5.96× (5.71–6.23) | 0.48× (0.46–0.50) | 0.51× (0.49–0.53) | 0.56× (0.53–0.58) | 0.27× (0.26–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.65–0.72) | 0.76× (0.72–0.81) | 1.25× (1.21–1.29) | 0.76× (0.74–0.79) | 3.66× (3.55–3.78) | 0.29× (0.28–0.29) | 0.30× (0.29–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.15× (5.14–5.16) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.50) | 0.40× (0.38–0.43) | 0.88× (0.86–0.91) | 0.71× (0.69–0.73) | 3.06× (2.98–3.15) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.46–0.47) | 0.88× (0.86–0.89) | 0.82× (0.79–0.85) | 2.57× (2.52–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.37× (0.34–0.40) | 0.84× (0.81–0.87) | 0.71× (0.68–0.74) | 2.89× (2.79–2.99) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.60–0.61) | 0.47× (0.45–0.50) | 0.80× (0.78–0.81) | 0.79× (0.76–0.82) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.31× (5.29–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.23× (0.23–0.24) | 1.86× (1.85–1.87) | 0.89× (0.89–0.90) | 6.18× (6.16–6.20) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.85× (1.85–1.86) | 0.89× (0.88–0.89) | 6.16× (6.14–6.18) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.39) | 0.66× (0.66–0.67) | 6.40× (6.39–6.41) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.64) | 0.31× (0.31–0.31) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 35 MB | 0.47× (0.46–0.48) | 0.41× (0.37–0.44) | 1.09× (1.06–1.11) | 0.72× (0.70–0.73) | 3.55× (3.49–3.61) | 0.22× (0.21–0.22) | 0.23× (0.23–0.23) | 0.35× (0.34–0.36) | 0.17× (0.17–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.44× (0.41–0.48) | 0.93× (0.90–0.96) | 0.78× (0.75–0.80) | 2.39× (2.32–2.46) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 550 | 1.82 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 990 | 1.95 | 1.07× (1.07–1.07) | 89% | 1.02× (1.01–1.03) | 1.03× (1.02–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,208 | 3.07 | 1.68× (1.67–1.69) | 55% | 1.19× (1.18–1.20) | 1.59× (1.57–1.62) |

**Best:** mibz2 at 4 threads (1,208 MB/s); lbzip2 at 4 threads (249 MB/s); pbz2 at 4 threads (371 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.03, lbzip2 1t 0.06, lbzip2 2t 0.19, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.50 | 0.01 | 0.08 | 0.174 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.7% |
| MTF + RLE2 | 1.48 | 18.0% |
| T-vector build | 1.48 | 17.9% |
| walk (the one each build uses) | 4.06 | 49.1% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.25 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.065 | (in its total) | 2.73× (2.71–2.74) |  |
| walk | 1.042 | 2.075 | (in its total) | 1.99× (1.98–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.041 | (in its total) | 1.14× (1.14–1.14) |  |
| inverse BWT, total | 1.468 | 3.181 | 1.883 | 2.16× (2.15–2.17) | 1.28× (1.27–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m26s ± 6m42s |
| 4 threads | 25m21s ± 3m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m11s ± 2m33s |
| the scanner alone (its serial part) | 7m34s ± 26.3s |
| the workers alone, over 4 | 18m11s ± 2m33s |

At 4 threads the workers bound it. Per byte: the scanner 4.4 ns a byte of bz2, the workers 2.38 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,260,864 | 15,030,132 | \`de55d3765b0096e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
