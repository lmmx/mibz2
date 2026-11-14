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

The latest record: [`1388759`](<https://github.com/lmmx/mibz2/commit/1388759acf3ba98236170c35988c921d9cedc7cf>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43604215197>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.436 s | 481 | 7.7 | 2.87× (2.81–2.93) | 6.54× (6.45–6.62) | 6.80× (6.69–6.92) | 7.71× (7.61–7.81) | 6.77× (6.65–6.89) | 7.07× (6.98–7.16) | 6.69× (6.59–6.79) | 2.10× (2.07–2.13) | 2.36× (2.32–2.39) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.260 s | 806 | 8.8 | 2.90× (2.77–3.03) | 5.86× (5.51–6.23) | 6.68× (6.41–6.96) | 6.85× (6.61–7.10) | 6.04× (5.81–6.29) | 11.77× (11.36–12.20) | 11.14× (10.74–11.55) | 3.50× (3.37–3.63) | 3.93× (3.79–4.07) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.224 s | 938 | 14.3 | 2.78× (2.69–2.87) | 4.16× (3.99–4.33) | 5.18× (4.99–5.37) | 4.88× (4.78–4.98) | 4.89× (4.68–5.11) | 13.89× (13.71–14.08) | 13.14× (12.87–13.41) | 4.12× (4.02–4.23) | 4.63× (4.53–4.74) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,108 | 3.7 | 0.58× (0.55–0.61) | 3.39× (3.34–3.44) | 4.56× (4.43–4.70) | 3.54× (3.49–3.59) | 6.05× (5.91–6.18) | 2.63× (2.59–2.67) | 2.63× (2.59–2.67) | 0.99× (0.96–1.03) | 2.02× (1.82–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,090 | 3.8 | 0.53× (0.52–0.54) | 2.80× (2.74–2.87) | 5.56× (5.44–5.69) | 3.49× (3.42–3.57) | 5.84× (5.70–5.98) | 2.58× (2.52–2.64) | 2.59× (2.53–2.65) | 0.98× (0.95–1.01) | 1.98× (1.80–2.18) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,078 | 3.7 | 0.53× (0.52–0.54) | 2.79× (2.73–2.85) | 5.55× (5.42–5.69) | 3.50× (3.43–3.57) | 5.92× (5.76–6.07) | 2.58× (2.52–2.64) | 2.58× (2.52–2.64) | 0.97× (0.94–1.01) | 1.98× (1.79–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 744 | 5.3 | 2.72× (2.69–2.75) | 3.07× (3.03–3.12) | 3.83× (3.77–3.89) | 4.10× (4.05–4.16) | 4.51× (4.43–4.59) | 2.77× (2.73–2.80) | 2.27× (2.24–2.31) | 2.14× (2.10–2.18) | 3.06× (3.01–3.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,095 | 6.8 | 2.21× (2.02–2.42) | 2.75× (2.52–3.00) | 4.35× (3.98–4.76) | 3.44× (3.19–3.72) | 4.88× (4.48–5.32) | 3.93× (3.61–4.29) | 3.23× (2.95–3.54) | 3.05× (2.78–3.34) | 4.35× (3.98–4.75) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 922 | 13.8 | 1.84× (1.74–1.93) | 1.82× (1.65–2.00) | 3.05× (2.94–3.17) | 2.08× (1.99–2.18) | 3.80× (3.65–3.95) | 3.43× (3.31–3.55) | 2.82× (2.71–2.93) | 2.65× (2.54–2.77) | 3.79× (3.65–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 52.5 | 1.40× (1.37–1.42) | 2.67× (2.62–2.71) | 2.44× (2.38–2.49) | 2.52× (2.47–2.58) | 2.80× (2.77–2.83) | 3.06× (2.98–3.14) | 3.11× (3.08–3.15) | 1.26× (1.24–1.28) | 1.44× (1.42–1.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 112 | 60.3 | 1.27× (1.21–1.33) | 2.51× (2.23–2.84) | 2.35× (2.21–2.50) | 3.02× (2.84–3.22) | 2.82× (2.70–2.94) | 4.81× (4.61–5.03) | 4.90× (4.71–5.10) | 1.98× (1.89–2.07) | 2.27× (2.16–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 142 | 77.4 | 1.35× (1.29–1.42) | 2.19× (2.11–2.27) | 2.24× (2.15–2.33) | 2.63× (2.56–2.70) | 2.91× (2.83–2.98) | 6.05× (5.86–6.23) | 6.15× (5.99–6.32) | 2.49× (2.43–2.55) | 2.85× (2.79–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 46.8 | 1.54× (1.53–1.55) | 3.07× (3.04–3.11) | 2.71× (2.61–2.83) | 2.79× (2.76–2.82) | 3.11× (3.08–3.13) | 3.74× (3.72–3.77) | 4.01× (3.98–4.05) | 1.55× (1.52–1.58) | 1.84× (1.81–1.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 127 | 54.5 | 1.36× (1.32–1.41) | 2.83× (2.66–3.02) | 2.65× (2.45–2.87) | 3.41× (3.30–3.53) | 3.05× (2.97–3.13) | 5.98× (5.84–6.13) | 6.42× (6.25–6.59) | 2.47× (2.39–2.56) | 2.93× (2.85–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 75.1 | 1.46× (1.41–1.51) | 2.36× (2.27–2.45) | 2.40× (2.34–2.46) | 3.12× (3.03–3.21) | 2.94× (2.89–2.99) | 7.16× (7.04–7.27) | 7.67× (7.58–7.77) | 2.96× (2.90–3.02) | 3.51× (3.46–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.64) | 3.43× (3.37–3.49) | 4.28× (4.20–4.35) | 3.08× (3.03–3.13) | 6.63× (6.49–6.76) | 2.26× (2.22–2.31) | 2.32× (2.28–2.36) | 0.94× (0.92–0.97) | 1.72× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 797 | 5.4 | 0.52× (0.50–0.53) | 2.54× (2.41–2.67) | 4.93× (4.76–5.10) | 2.95× (2.85–3.04) | 6.34× (6.19–6.50) | 2.16× (2.09–2.23) | 2.21× (2.15–2.28) | 0.90× (0.87–0.93) | 1.65× (1.59–1.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 797 | 5.2 | 0.52× (0.51–0.53) | 2.48× (2.43–2.53) | 4.94× (4.84–5.05) | 2.99× (2.94–3.04) | 6.36× (6.19–6.53) | 2.16× (2.12–2.21) | 2.22× (2.18–2.26) | 0.90× (0.89–0.92) | 1.65× (1.61–1.68) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 0.056 s | 220 | 17.3 | 1.79× (1.73–1.86) | 3.86× (3.70–4.03) | 4.21× (4.07–4.36) | 4.43× (4.29–4.58) | 4.60× (4.43–4.78) | 4.55× (4.38–4.72) | 4.48× (4.33–4.63) | 1.47× (1.41–1.54) | 1.65× (1.59–1.72) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 0.039 s | 316 | 22.2 | 1.45× (1.37–1.54) | 3.13× (3.03–3.23) | 3.98× (3.84–4.12) | 4.26× (4.11–4.42) | 4.14× (3.97–4.31) | 6.60× (6.37–6.83) | 6.50× (6.27–6.73) | 2.14× (2.05–2.23) | 2.40× (2.32–2.48) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 0.035 s | 353 | 33.6 | 1.46× (1.40–1.53) | 2.53× (2.28–2.81) | 3.30× (3.12–3.49) | 3.47× (3.31–3.64) | 3.85× (3.65–4.05) | 7.42× (7.12–7.72) | 7.30× (7.04–7.58) | 2.40× (2.28–2.53) | 2.70× (2.58–2.82) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.86× (2.81–2.92) | 6.50× (6.42–6.58) | 6.77× (6.65–6.88) | 7.68× (7.58–7.78) | 6.75× (6.63–6.87) | 7.04× (6.96–7.13) | 6.66× (6.56–6.76) | 2.09× (2.06–2.12) | 2.34× (2.30–2.37) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.66–2.84) | 6.02× (5.89–6.16) | 6.23× (6.15–6.31) | 7.12× (7.02–7.22) | 6.02× (5.91–6.12) | 6.22× (6.13–6.30) | 5.88× (5.79–5.96) | 1.85× (1.81–1.89) | 2.06× (2.03–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.67× (2.59–2.76) | 4.45× (4.27–4.63) | 4.83× (4.67–5.00) | 5.24× (5.17–5.32) | 4.61× (4.44–4.79) | 3.83× (3.77–3.89) | 3.62× (3.56–3.68) | 1.14× (1.11–1.16) | 1.27× (1.24–1.30) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.59–0.65) | 2.92× (2.86–2.97) | 3.91× (3.81–4.01) | 3.31× (3.27–3.36) | 5.59× (5.49–5.70) | 2.48× (2.45–2.51) | 2.49× (2.45–2.52) | 0.99× (0.96–1.03) | 1.63× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.59) | 2.70× (2.65–2.76) | 4.90× (4.80–4.99) | 3.24× (3.17–3.32) | 5.34× (5.21–5.47) | 2.40× (2.35–2.46) | 2.40× (2.35–2.46) | 0.96× (0.94–0.99) | 1.58× (1.55–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.05× (2.99–3.10) | 5.02× (4.90–5.15) | 3.33× (3.28–3.38) | 5.51× (5.39–5.64) | 2.46× (2.41–2.50) | 2.46× (2.42–2.51) | 0.98× (0.95–1.01) | 1.62× (1.59–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.62× (2.59–2.65) | 2.85× (2.81–2.89) | 3.41× (3.38–3.45) | 3.92× (3.86–3.97) | 4.30× (4.22–4.38) | 2.66× (2.62–2.70) | 2.20× (2.16–2.23) | 2.07× (2.04–2.11) | 2.66× (2.62–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.18–2.31) | 2.40× (2.31–2.51) | 3.26× (3.17–3.35) | 3.21× (3.11–3.32) | 3.47× (3.36–3.57) | 2.07× (2.00–2.15) | 1.71× (1.65–1.77) | 1.61× (1.55–1.68) | 2.07× (2.01–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.84–1.98) | 1.59× (1.53–1.65) | 2.11× (2.06–2.16) | 1.94× (1.87–2.01) | 1.98× (1.92–2.05) | 1.03× (0.99–1.06) | 0.85× (0.82–0.88) | 0.80× (0.78–0.83) | 1.03× (0.99–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.37–1.42) | 2.58× (2.55–2.62) | 2.38× (2.34–2.42) | 2.50× (2.44–2.56) | 2.77× (2.75–2.80) | 3.03× (2.95–3.11) | 3.08× (3.04–3.12) | 1.26× (1.24–1.27) | 1.37× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.21–1.31) | 2.39× (2.28–2.50) | 2.13× (2.03–2.23) | 2.80× (2.67–2.95) | 2.52× (2.43–2.60) | 2.60× (2.52–2.70) | 2.65× (2.57–2.73) | 1.08× (1.04–1.12) | 1.18× (1.14–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.46–1.59) | 2.44× (2.36–2.52) | 2.03× (1.95–2.12) | 2.62× (2.54–2.70) | 2.54× (2.45–2.62) | 2.04× (2.00–2.08) | 2.08× (2.04–2.11) | 0.85× (0.83–0.86) | 0.93× (0.90–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.52–1.55) | 3.02× (2.98–3.05) | 2.65× (2.56–2.75) | 2.76× (2.73–2.79) | 3.08× (3.05–3.10) | 3.70× (3.68–3.73) | 3.97× (3.93–4.01) | 1.54× (1.51–1.57) | 1.77× (1.75–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.34–1.43) | 2.75× (2.65–2.86) | 2.41× (2.25–2.59) | 3.17× (3.09–3.26) | 2.73× (2.64–2.82) | 3.16× (3.08–3.25) | 3.39× (3.30–3.48) | 1.32× (1.28–1.36) | 1.52× (1.47–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.54–1.66) | 2.59× (2.48–2.70) | 2.05× (1.96–2.15) | 2.84× (2.67–3.02) | 2.65× (2.55–2.75) | 2.29× (2.22–2.36) | 2.46× (2.40–2.52) | 0.95× (0.93–0.98) | 1.10× (1.07–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.69) | 2.50× (2.42–2.60) | 3.41× (3.35–3.48) | 2.82× (2.77–2.87) | 5.92× (5.79–6.04) | 2.10× (2.06–2.15) | 2.15× (2.11–2.19) | 0.96× (0.93–0.98) | 1.36× (1.33–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.31× (2.25–2.37) | 4.06× (3.98–4.14) | 2.71× (2.63–2.78) | 5.58× (5.46–5.70) | 1.97× (1.91–2.03) | 2.02× (1.96–2.07) | 0.89× (0.87–0.92) | 1.27× (1.24–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.59) | 2.57× (2.50–2.64) | 4.15× (4.07–4.22) | 2.83× (2.79–2.87) | 5.77× (5.63–5.91) | 2.03× (1.99–2.08) | 2.08× (2.05–2.11) | 0.92× (0.91–0.94) | 1.31× (1.29–1.34) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 1.76× (1.69–1.84) | 3.70× (3.55–3.86) | 4.03× (3.90–4.17) | 4.32× (4.19–4.47) | 4.49× (4.33–4.67) | 4.44× (4.28–4.61) | 4.37× (4.23–4.52) | 1.46× (1.39–1.53) | 1.57× (1.52–1.62) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 1.51× (1.47–1.56) | 3.07× (3.01–3.12) | 3.39× (3.31–3.48) | 3.89× (3.77–4.02) | 3.65× (3.52–3.79) | 3.52× (3.41–3.64) | 3.47× (3.36–3.59) | 1.16× (1.11–1.21) | 1.25× (1.21–1.29) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 1.62× (1.54–1.69) | 2.70× (2.53–2.87) | 2.90× (2.79–3.02) | 3.19× (3.14–3.23) | 3.22× (3.07–3.38) | 2.35× (2.30–2.41) | 2.32× (2.27–2.37) | 0.77× (0.75–0.80) | 0.83× (0.82–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 34 MB | 0.25× (0.25–0.25) | 0.27× (0.26–0.28) | 0.81× (0.81–0.81) | 0.65× (0.64–0.65) | 9.12× (9.11–9.13) | 0.22× (0.21–0.22) | 0.24× (0.24–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 79 MB | 0.24× (0.23–0.25) | 0.32× (0.26–0.38) | 0.49× (0.47–0.51) | 0.41× (0.39–0.43) | 3.91× (3.72–4.11) | 0.09× (0.09–0.10) | 0.10× (0.10–0.11) | 0.15× (0.14–0.16) | 0.08× (0.07–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 138 MB | 0.22× (0.21–0.24) | 0.32× (0.27–0.39) | 0.40× (0.38–0.43) | 0.35× (0.33–0.38) | 2.33× (2.19–2.49) | 0.05× (0.05–0.06) | 0.06× (0.06–0.06) | 0.09× (0.08–0.09) | 0.04× (0.04–0.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.03× (8.01–8.06) | 0.65× (0.65–0.65) | 0.69× (0.69–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.62× (0.56–0.69) | 1.59× (1.54–1.64) | 0.79× (0.76–0.81) | 5.76× (5.59–5.94) | 0.46× (0.45–0.48) | 0.49× (0.48–0.51) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.66× (0.64–0.69) | 0.65× (0.61–0.69) | 1.17× (1.14–1.19) | 0.71× (0.70–0.73) | 3.42× (3.37–3.48) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.73) | 5.21× (5.19–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.48–0.53) | 0.38× (0.35–0.42) | 0.91× (0.88–0.95) | 0.73× (0.70–0.76) | 3.15× (3.01–3.28) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.44× (0.43–0.45) | 0.84× (0.83–0.86) | 0.80× (0.78–0.82) | 2.46× (2.41–2.51) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.23× (5.22–5.25) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.37× (0.35–0.40) | 0.85× (0.83–0.87) | 0.70× (0.67–0.74) | 2.90× (2.83–2.98) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.63) | 0.46× (0.44–0.49) | 0.81× (0.79–0.82) | 0.81× (0.78–0.84) | 2.32× (2.28–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.23–0.25) | 1.08× (1.07–1.08) | 0.77× (0.76–0.77) | 5.39× (5.37–5.41) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.77–0.77) | 5.31× (5.29–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.12–0.13) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.41× (6.38–6.43) | 0.39× (0.38–0.39) | 0.42× (0.42–0.43) | 0.64× (0.63–0.65) | 0.31× (0.31–0.32) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 38 MB | 0.43× (0.42–0.45) | 0.39× (0.35–0.42) | 1.01× (0.98–1.04) | 0.65× (0.63–0.68) | 3.20× (3.10–3.31) | 0.19× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 55 MB | 0.49× (0.48–0.51) | 0.43× (0.38–0.48) | 0.91× (0.88–0.93) | 0.72× (0.68–0.76) | 2.31× (2.24–2.38) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 481 | 2.09 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 806 | 2.38 | 1.13× (1.11–1.16) | 83% | 1.05× (1.02–1.08) | 1.09× (1.07–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 938 | 3.86 | 1.84× (1.80–1.88) | 49% | 1.26× (1.21–1.31) | 1.72× (1.67–1.76) |

**Best:** mibz2 at 4 threads (938 MB/s); lbzip2 at 4 threads (230 MB/s); pbz2 at 4 threads (343 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.11, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.36, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.64 | 0.02 | 0.08 | 0.224 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.09 | 12.5% |
| MTF + RLE2 | 1.55 | 17.7% |
| T-vector build | 1.47 | 16.8% |
| walk (the one each build uses) | 4.18 | 47.7% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.76 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.062 | (in its total) | 2.71× (2.70–2.72) |  |
| walk | 1.066 | 2.063 | (in its total) | 1.93× (1.92–1.94) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.041 | (in its total) | 1.08× (1.06–1.11) |  |
| inverse BWT, total | 1.495 | 3.166 | 1.900 | 2.11× (2.10–2.12) | 1.28× (1.25–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m24s |
| 4 threads | 32m22s ± 4m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m15s ± 3m18s |
| the scanner alone (its serial part) | 11m12s ± 30.2s |
| the workers alone, over 4 | 23m15s ± 3m18s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 3.05 ns a byte of output at 4 threads (CPU, summed over them), 1.72 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,417,543 | 12,200,394 | \`a6b24102e87d445d…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
