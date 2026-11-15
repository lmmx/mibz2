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

The latest record: [`e449ba4`](<https://github.com/lmmx/mibz2/commit/e449ba406ea601878a70bbaa2933bca5ab4baeaa>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43735652778>)

Intel(R) Xeon(R) 6973P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.415 s | 505 | 6.6 | 3.10× (3.01–3.19) | 7.33× (7.13–7.53) | 7.54× (7.25–7.83) | 8.30× (8.00–8.61) | 7.95× (7.71–8.19) | 8.02× (7.76–8.29) | 7.79× (7.64–7.93) | 2.34× (2.27–2.41) | 2.20× (2.12–2.27) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.237 s | 885 | 7.1 | 3.12× (2.93–3.32) | 6.77× (6.32–7.24) | 7.96× (7.55–8.40) | 7.82× (7.38–8.29) | 7.47× (7.01–7.96) | 13.86× (12.93–14.86) | 13.45× (12.66–14.29) | 4.04× (3.76–4.34) | 3.79× (3.53–4.08) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,070 | 11.2 | 3.11× (2.98–3.26) | 5.92× (5.59–6.26) | 6.94× (6.58–7.33) | 6.75× (6.33–7.20) | 6.79× (6.44–7.16) | 16.90× (15.86–18.01) | 16.40× (15.49–17.37) | 4.92× (4.62–5.24) | 4.63× (4.31–4.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,194 | 3.0 | 0.59× (0.56–0.62) | 3.08× (2.93–3.24) | 4.24× (4.03–4.45) | 3.35× (3.19–3.51) | 6.10× (5.77–6.46) | 2.52× (2.37–2.67) | 2.55× (2.40–2.71) | 1.14× (1.07–1.21) | 1.20× (1.14–1.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,120 | 3.3 | 0.49× (0.47–0.51) | 2.83× (2.68–2.98) | 5.76× (5.44–6.09) | 3.26× (3.02–3.51) | 5.78× (5.45–6.14) | 2.39× (2.25–2.53) | 2.42× (2.35–2.49) | 1.08× (1.01–1.16) | 1.14× (1.08–1.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,108 | 3.3 | 0.51× (0.49–0.52) | 2.85× (2.79–2.91) | 5.77× (5.47–6.08) | 3.25× (3.16–3.33) | 5.80× (5.60–6.01) | 2.40× (2.35–2.44) | 2.43× (2.35–2.52) | 1.09× (1.04–1.13) | 1.15× (1.12–1.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 602 | 5.8 | 2.38× (2.29–2.48) | 2.87× (2.77–2.98) | 3.49× (3.40–3.58) | 3.78× (3.70–3.86) | 4.38× (4.28–4.48) | 2.85× (2.75–2.95) | 2.66× (2.55–2.78) | 1.91× (1.87–1.95) | 1.94× (1.86–2.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 910 | 7.2 | 1.93× (1.64–2.27) | 2.50× (2.03–3.09) | 3.65× (3.26–4.08) | 3.47× (3.11–3.86) | 4.55× (3.87–5.35) | 4.05× (3.42–4.80) | 3.78× (3.20–4.47) | 2.71× (2.31–3.18) | 2.75× (2.30–3.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 889 | 12.6 | 1.71× (1.59–1.84) | 1.93× (1.84–2.03) | 2.96× (2.81–3.13) | 2.24× (2.15–2.33) | 3.97× (3.76–4.20) | 4.20× (4.01–4.40) | 3.93× (3.70–4.16) | 2.82× (2.69–2.95) | 2.86× (2.72–3.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.108 s | 74 | 45.7 | 1.39× (1.33–1.45) | 3.11× (3.06–3.15) | 2.92× (2.88–2.97) | 2.93× (2.88–2.98) | 3.26× (3.21–3.32) | 3.54× (3.47–3.61) | 3.64× (3.58–3.71) | 1.25× (1.23–1.28) | 1.29× (1.26–1.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 120 | 51.3 | 1.28× (1.21–1.34) | 2.87× (2.71–3.03) | 2.81× (2.66–2.98) | 3.57× (3.22–3.97) | 3.27× (3.09–3.45) | 5.61× (5.32–5.92) | 5.78× (5.48–6.09) | 1.99× (1.87–2.11) | 2.04× (1.96–2.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 70.3 | 1.38× (1.34–1.41) | 2.77× (2.68–2.86) | 2.74× (2.66–2.81) | 3.23× (3.12–3.34) | 3.31× (3.23–3.39) | 6.88× (6.76–7.00) | 7.08× (6.92–7.25) | 2.44× (2.37–2.51) | 2.50× (2.42–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.122 s | 79 | 42.8 | 1.36× (1.32–1.40) | 3.35× (3.24–3.46) | 3.00× (2.88–3.13) | 2.96× (2.84–3.08) | 3.31× (3.19–3.44) | 4.06× (3.92–4.20) | 4.32× (4.14–4.50) | 1.43× (1.36–1.51) | 1.47× (1.41–1.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 47.9 | 1.30× (1.27–1.34) | 3.13× (3.00–3.28) | 2.96× (2.85–3.07) | 3.96× (3.80–4.12) | 3.44× (3.36–3.53) | 6.90× (6.67–7.14) | 7.34× (7.10–7.58) | 2.44× (2.35–2.54) | 2.50× (2.39–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 153 | 68.8 | 1.37× (1.33–1.42) | 2.89× (2.77–3.02) | 2.71× (2.60–2.81) | 3.62× (3.37–3.88) | 3.20× (3.07–3.35) | 7.90× (7.51–8.31) | 8.41× (7.95–8.90) | 2.79× (2.63–2.96) | 2.86× (2.72–3.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 881 | 4.3 | 0.64× (0.61–0.67) | 2.74× (2.58–2.92) | 3.76× (3.59–3.93) | 2.87× (2.77–2.98) | 6.60× (6.42–6.79) | 2.18× (2.10–2.26) | 2.21× (2.13–2.30) | 1.06× (1.03–1.09) | 0.92× (0.88–0.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 837 | 4.6 | 0.49× (0.47–0.50) | 2.56× (2.50–2.62) | 5.20× (5.08–5.31) | 2.83× (2.71–2.94) | 6.38× (6.19–6.57) | 2.09× (2.02–2.15) | 2.12× (2.05–2.19) | 1.01× (0.99–1.03) | 0.88× (0.86–0.90) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 826 | 4.6 | 0.49× (0.47–0.51) | 2.56× (2.49–2.63) | 5.13× (4.77–5.50) | 2.77× (2.71–2.84) | 6.29× (6.16–6.43) | 2.08× (2.01–2.15) | 2.11× (2.04–2.18) | 1.01× (0.98–1.04) | 0.87× (0.85–0.90) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.057 s | 220 | 15.6 | 1.76× (1.71–1.81) | 4.18× (4.04–4.31) | 4.64× (4.45–4.83) | 4.66× (4.48–4.85) | 5.12× (4.96–5.28) | 4.94× (4.77–5.12) | 4.96× (4.75–5.19) | 1.48× (1.41–1.56) | 1.41× (1.35–1.48) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.041 s | 306 | 19.3 | 1.33× (1.19–1.47) | 3.17× (2.81–3.58) | 3.77× (3.42–4.16) | 4.23× (3.77–4.74) | 4.23× (3.81–4.69) | 6.65× (6.00–7.37) | 6.68× (6.06–7.36) | 1.99× (1.80–2.21) | 1.90× (1.71–2.11) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.035 s | 365 | 28.9 | 1.46× (1.43–1.49) | 2.84× (2.73–2.94) | 3.49× (3.33–3.67) | 3.89× (3.79–3.99) | 4.23× (4.12–4.35) | 8.27× (8.04–8.50) | 8.30× (7.97–8.64) | 2.48× (2.40–2.56) | 2.36× (2.26–2.46) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.09× (3.00–3.18) | 7.29× (7.10–7.49) | 7.50× (7.22–7.80) | 8.27× (7.97–8.58) | 7.92× (7.69–8.16) | 7.99× (7.74–8.26) | 7.76× (7.62–7.90) | 2.33× (2.26–2.40) | 2.18× (2.10–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.99× (2.86–3.13) | 7.10× (6.79–7.42) | 7.58× (7.38–7.79) | 8.31× (8.04–8.58) | 7.59× (7.31–7.89) | 7.41× (7.11–7.73) | 7.19× (6.95–7.44) | 2.16× (2.06–2.26) | 2.02× (1.92–2.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.14× (3.04–3.25) | 6.64× (6.36–6.94) | 6.95× (6.66–7.26) | 7.54× (7.23–7.87) | 6.94× (6.69–7.20) | 4.83× (4.60–5.07) | 4.69× (4.48–4.91) | 1.41× (1.34–1.47) | 1.32× (1.25–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.60–0.66) | 2.64× (2.52–2.77) | 3.55× (3.39–3.72) | 3.15× (3.01–3.30) | 5.68× (5.37–6.02) | 2.39× (2.25–2.54) | 2.43× (2.28–2.58) | 1.13× (1.06–1.20) | 0.81× (0.79–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.52–0.57) | 2.57× (2.41–2.74) | 4.80× (4.56–5.05) | 3.05× (2.84–3.28) | 5.33× (5.05–5.63) | 2.24× (2.12–2.37) | 2.28× (2.22–2.33) | 1.06× (0.99–1.13) | 0.76× (0.70–0.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 2.98× (2.89–3.07) | 4.99× (4.75–5.24) | 3.08× (3.00–3.16) | 5.40× (5.23–5.58) | 2.28× (2.23–2.33) | 2.32× (2.23–2.40) | 1.08× (1.03–1.12) | 0.77× (0.73–0.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.33× (2.24–2.42) | 2.75× (2.69–2.82) | 3.24× (3.16–3.32) | 3.66× (3.58–3.75) | 4.23× (4.13–4.34) | 2.77× (2.67–2.87) | 2.59× (2.48–2.71) | 1.87× (1.83–1.91) | 1.81× (1.74–1.88) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (2.00–2.15) | 2.38× (2.26–2.49) | 2.90× (2.79–3.01) | 3.09× (2.96–3.22) | 3.61× (3.46–3.76) | 2.24× (2.14–2.34) | 2.10× (1.99–2.21) | 1.51× (1.45–1.58) | 1.46× (1.38–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.83× (1.69–1.99) | 1.67× (1.56–1.79) | 2.14× (2.00–2.28) | 2.06× (1.91–2.23) | 2.18× (2.02–2.35) | 1.23× (1.13–1.34) | 1.15× (1.06–1.25) | 0.83× (0.78–0.89) | 0.80× (0.74–0.87) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.33–1.44) | 3.04× (3.01–3.07) | 2.85× (2.81–2.90) | 2.90× (2.85–2.95) | 3.23× (3.17–3.29) | 3.50× (3.44–3.57) | 3.60× (3.54–3.67) | 1.25× (1.23–1.27) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.22–1.31) | 2.84× (2.73–2.94) | 2.59× (2.49–2.70) | 3.24× (3.11–3.37) | 2.93× (2.81–3.06) | 3.06× (2.93–3.20) | 3.15× (3.02–3.29) | 1.09× (1.04–1.15) | 1.07× (1.02–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.43–1.62) | 2.95× (2.86–3.04) | 2.58× (2.50–2.67) | 3.23× (3.13–3.34) | 2.94× (2.86–3.02) | 2.29× (2.22–2.35) | 2.35× (2.27–2.44) | 0.82× (0.79–0.85) | 0.80× (0.77–0.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.36× (1.32–1.39) | 3.29× (3.18–3.41) | 2.92× (2.81–3.05) | 2.93× (2.82–3.05) | 3.28× (3.17–3.41) | 4.02× (3.88–4.16) | 4.27× (4.10–4.45) | 1.43× (1.35–1.51) | 1.42× (1.37–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.29× (1.26–1.33) | 3.14× (3.02–3.25) | 2.72× (2.63–2.82) | 3.61× (3.48–3.75) | 3.11× (3.03–3.19) | 3.63× (3.49–3.77) | 3.86× (3.72–4.00) | 1.29× (1.24–1.34) | 1.28× (1.22–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.50–1.61) | 3.15× (3.01–3.30) | 2.51× (2.41–2.62) | 3.43× (3.27–3.59) | 2.94× (2.82–3.06) | 2.56× (2.42–2.70) | 2.72× (2.57–2.88) | 0.91× (0.86–0.96) | 0.90× (0.85–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.65–0.72) | 2.22× (2.13–2.31) | 3.18× (3.07–3.30) | 2.66× (2.57–2.75) | 5.96× (5.79–6.12) | 2.05× (1.97–2.12) | 2.08× (2.01–2.16) | 1.06× (1.02–1.10) | 0.75× (0.71–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.10× (2.06–2.15) | 4.11× (4.02–4.19) | 2.64× (2.54–2.75) | 5.71× (5.54–5.88) | 1.93× (1.88–1.99) | 1.97× (1.91–2.02) | 1.00× (0.98–1.02) | 0.70× (0.68–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.56) | 2.52× (2.48–2.55) | 4.21× (4.00–4.44) | 2.61× (2.57–2.66) | 5.69× (5.60–5.78) | 1.95× (1.89–2.01) | 1.98× (1.92–2.04) | 1.01× (0.99–1.03) | 0.71× (0.69–0.73) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.73× (1.68–1.79) | 4.03× (3.91–4.15) | 4.43× (4.23–4.63) | 4.56× (4.38–4.75) | 5.01× (4.85–5.17) | 4.84× (4.67–5.01) | 4.85× (4.64–5.07) | 1.47× (1.39–1.55) | 1.34× (1.29–1.40) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.50× (1.45–1.55) | 3.43× (3.30–3.56) | 3.87× (3.76–4.00) | 4.21× (4.04–4.39) | 4.12× (3.97–4.27) | 3.87× (3.72–4.02) | 3.88× (3.67–4.10) | 1.17× (1.11–1.24) | 1.07× (1.02–1.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.73× (1.67–1.78) | 3.26× (3.18–3.35) | 3.39× (3.26–3.53) | 3.69× (3.63–3.76) | 3.65× (3.54–3.75) | 2.60× (2.52–2.68) | 2.61× (2.50–2.73) | 0.79× (0.76–0.82) | 0.72× (0.69–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.27–0.29) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.08× (11.05–11.10) | 0.26× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.40× (0.34–0.47) | 0.67× (0.65–0.70) | 0.57× (0.54–0.59) | 5.35× (5.19–5.51) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 96 MB | 0.32× (0.30–0.34) | 0.48× (0.41–0.55) | 0.57× (0.54–0.60) | 0.49× (0.47–0.52) | 3.32× (3.16–3.48) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.29× (4.29–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.36–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.02–8.08) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.75× (0.75–0.76) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.69) | 0.60× (0.54–0.66) | 1.61× (1.59–1.64) | 0.81× (0.80–0.82) | 5.85× (5.77–5.94) | 0.47× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.54–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.66× (0.61–0.71) | 1.13× (1.09–1.17) | 0.71× (0.68–0.73) | 3.40× (3.27–3.52) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.50–0.53) | 0.43× (0.41–0.45) | 0.92× (0.89–0.96) | 0.72× (0.69–0.75) | 3.18× (3.07–3.30) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.61–0.63) | 0.45× (0.44–0.46) | 0.85× (0.84–0.87) | 0.79× (0.77–0.81) | 2.49× (2.44–2.53) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.21–5.25) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.38× (0.35–0.42) | 0.87× (0.84–0.90) | 0.71× (0.67–0.76) | 2.99× (2.89–3.10) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.58–0.63) | 0.49× (0.46–0.52) | 0.81× (0.79–0.83) | 0.82× (0.78–0.85) | 2.34× (2.29–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.66) | 6.39× (6.37–6.40) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.39× (0.35–0.43) | 1.05× (1.01–1.09) | 0.68× (0.67–0.70) | 3.30× (3.23–3.36) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.48× (0.45–0.52) | 0.89× (0.87–0.92) | 0.72× (0.69–0.74) | 2.29× (2.24–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 505 | 1.99 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 885 | 2.14 | 1.08× (1.03–1.13) | 86% | 1.05× (1.04–1.06) | 1.04× (1.01–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,070 | 3.37 | 1.66× (1.57–1.75) | 53% | 1.51× (1.49–1.53) | 1.68× (1.61–1.76) |

**Best:** mibz2 at 4 threads (1,070 MB/s); lbzip2 at 4 threads (177 MB/s); pbz2 at 4 threads (339 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.21, lbzip2 1t 0.04, lbzip2 2t 0.17, lbzip2 4t 0.29, pbz2 1t 0.00, pbz2 2t 0.39, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.53 | 0.02 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3327 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.99 | 10.8% |
| MTF + RLE2 | 1.39 | 15.2% |
| T-vector build | 1.30 | 14.2% |
| walk (the one each build uses) | 5.13 | 56.0% |
| of which: segment assembly | 0.26 | 2.9% |
| RLE1 | 0.22 | 2.4% |
| CRC | 0.12 | 1.3% |
| **total** | 9.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.349 | 0.876 | (in its total) | 2.51× (2.42–2.60) |  |
| walk | 1.444 | 2.717 | (in its total) | 1.88× (1.84–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.072 | (in its total) | 0.91× (0.90–0.92) |  |
| inverse BWT, total | 1.873 | 3.668 | 1.907 | 1.96× (1.91–2.01) | 1.02× (0.99–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m21s |
| 4 threads | 29m13s ± 4m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m19s ± 2m52s |
| the scanner alone (its serial part) | 10m55s ± 44.2s |
| the workers alone, over 4 | 19m19s ± 2m52s |

At 4 threads the workers bound it. Per byte: the scanner 6.3 ns a byte of bz2, the workers 2.53 ns a byte of output at 4 threads (CPU, summed over them), 1.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,509,359 | 12,625,378 | \`78d0692041033556…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6973P-C, 4 cores, reported clock 3326.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
