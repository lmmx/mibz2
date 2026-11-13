# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-379/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`3e7e294`](<https://github.com/lmmx/mibz2/commit/3e7e294cc10f29495e9828181345d74233c549d4>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43409387482>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.302 s | 693 | 6.1 | 2.87× (2.82–2.91) | 7.24× (7.11–7.36) | 7.61× (7.43–7.79) | 8.53× (8.35–8.70) | 7.68× (7.52–7.84) | 7.81× (7.69–7.94) | 7.43× (7.25–7.62) | 2.22× (2.19–2.25) | 2.57× (2.53–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.183 s | 1,147 | 6.8 | 2.71× (2.66–2.77) | 6.14× (6.00–6.28) | 7.41× (7.16–7.67) | 7.59× (7.34–7.84) | 6.95× (6.43–7.52) | 12.74× (12.40–13.09) | 12.12× (11.75–12.51) | 3.61× (3.52–3.71) | 4.19× (4.07–4.32) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.168 s | 1,246 | 11.6 | 2.49× (2.44–2.54) | 3.97× (3.90–4.04) | 5.24× (5.07–5.41) | 4.74× (4.60–4.88) | 4.85× (4.74–4.96) | 13.74× (13.32–14.18) | 13.08× (12.57–13.61) | 3.90× (3.77–4.03) | 4.52× (4.36–4.70) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,439 | 3.3 | 0.58× (0.57–0.58) | 3.67× (3.61–3.74) | 4.59× (4.42–4.78) | 3.98× (3.60–4.40) | 6.62× (6.48–6.77) | 2.88× (2.81–2.95) | 2.88× (2.80–2.95) | 1.01× (0.99–1.03) | 2.21× (2.17–2.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,379 | 3.5 | 0.50× (0.44–0.56) | 2.90× (2.62–3.21) | 5.56× (4.91–6.30) | 3.50× (3.12–3.93) | 5.97× (5.30–6.73) | 2.65× (2.35–2.99) | 2.65× (2.36–2.98) | 0.93× (0.82–1.05) | 2.03× (1.80–2.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,379 | 3.4 | 0.52× (0.51–0.54) | 3.10× (2.75–3.50) | 5.79× (5.61–5.98) | 3.66× (3.56–3.77) | 6.34× (6.14–6.54) | 2.77× (2.65–2.89) | 2.77× (2.63–2.91) | 0.97× (0.94–1.01) | 2.12× (2.05–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,071 | 4.3 | 2.90× (2.86–2.94) | 3.62× (3.39–3.86) | 4.85× (4.08–5.77) | 4.88× (4.53–5.26) | 5.79× (5.16–6.49) | 3.10× (3.03–3.17) | 2.65× (2.60–2.70) | 2.19× (2.15–2.22) | 3.34× (3.29–3.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,385 | 5.9 | 2.22× (1.75–2.82) | 2.74× (2.38–3.16) | 4.00× (3.28–4.88) | 3.79× (3.48–4.14) | 5.13× (4.46–5.90) | 3.78× (3.29–4.34) | 3.22× (2.81–3.70) | 2.67× (2.31–3.07) | 4.07× (3.53–4.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,188 | 11.8 | 1.50× (1.24–1.83) | 1.73× (1.41–2.12) | 2.57× (2.03–3.25) | 1.84× (1.53–2.21) | 3.82× (3.13–4.66) | 3.13× (2.56–3.82) | 2.67× (2.17–3.29) | 2.21× (1.79–2.72) | 3.37× (2.76–4.12) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.081 s | 99 | 43.5 | 1.38× (1.35–1.40) | 3.10× (3.04–3.17) | 2.86× (2.80–2.92) | 2.76× (2.73–2.80) | 3.16× (3.12–3.21) | 3.53× (3.49–3.58) | 3.58× (3.54–3.63) | 1.34× (1.32–1.36) | 1.53× (1.49–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 158 | 49.9 | 1.34× (1.18–1.53) | 3.00× (2.63–3.42) | 2.67× (2.49–2.86) | 3.20× (3.02–3.39) | 3.10× (2.94–3.27) | 5.53× (5.21–5.86) | 5.61× (5.31–5.93) | 2.10× (2.01–2.20) | 2.39× (2.25–2.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 191 | 65.2 | 1.28× (1.24–1.33) | 2.44× (2.34–2.54) | 2.37× (2.28–2.46) | 2.93× (2.74–3.14) | 3.15× (3.06–3.24) | 6.72× (6.56–6.89) | 6.82× (6.63–7.01) | 2.56× (2.49–2.62) | 2.91× (2.85–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.085 s | 113 | 38.1 | 1.50× (1.49–1.51) | 3.76× (3.70–3.83) | 3.15× (3.06–3.24) | 3.24× (2.91–3.62) | 3.78× (3.33–4.28) | 4.48× (4.41–4.55) | 4.87× (4.75–4.99) | 1.64× (1.56–1.72) | 1.88× (1.79–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 44.3 | 1.41× (1.13–1.74) | 3.25× (3.13–3.38) | 2.81× (2.67–2.94) | 3.80× (3.62–3.98) | 3.39× (3.22–3.56) | 6.91× (6.72–7.11) | 7.52× (7.23–7.82) | 2.53× (2.37–2.70) | 2.90× (2.73–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 205 | 61.5 | 1.32× (1.12–1.55) | 2.45× (2.15–2.80) | 2.47× (2.15–2.84) | 3.15× (2.76–3.60) | 2.99× (2.60–3.45) | 7.71× (6.70–8.88) | 8.39× (7.40–9.50) | 2.82× (2.55–3.12) | 3.23× (2.92–3.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,064 | 4.6 | 0.63× (0.62–0.65) | 3.29× (3.20–3.38) | 4.54× (4.45–4.63) | 3.20× (3.14–3.27) | 7.13× (7.00–7.26) | 2.40× (2.38–2.43) | 2.39× (2.33–2.45) | 0.97× (0.94–1.00) | 2.18× (2.14–2.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,042 | 4.8 | 0.51× (0.50–0.52) | 3.16× (3.07–3.26) | 5.36× (5.07–5.67) | 3.09× (3.03–3.16) | 6.73× (6.58–6.88) | 2.33× (2.25–2.41) | 2.31× (2.25–2.38) | 0.94× (0.92–0.97) | 2.11× (2.05–2.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,000 | 4.8 | 0.51× (0.49–0.52) | 3.08× (3.00–3.16) | 5.29× (5.14–5.45) | 3.03× (2.95–3.12) | 6.65× (6.45–6.87) | 2.26× (2.18–2.34) | 2.24× (2.19–2.30) | 0.91× (0.89–0.94) | 2.05× (2.00–2.10) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.043 s | 300 | 14.6 | 1.69× (1.55–1.85) | 4.08× (3.65–4.56) | 4.28× (3.65–5.02) | 4.46× (3.82–5.22) | 4.73× (4.03–5.55) | 4.88× (4.14–5.75) | 4.57× (3.91–5.35) | 1.43× (1.23–1.66) | 1.67× (1.43–1.95) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.034 s | 387 | 18.6 | 1.28× (1.21–1.37) | 3.11× (2.90–3.34) | 3.76× (3.52–4.02) | 4.10× (3.84–4.37) | 4.15× (3.86–4.46) | 6.62× (6.20–7.07) | 6.21× (5.83–6.61) | 1.94× (1.82–2.08) | 2.27× (2.10–2.45) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.030 s | 430 | 28.4 | 1.20× (1.09–1.32) | 2.26× (1.95–2.63) | 2.97× (2.54–3.48) | 3.10× (2.53–3.80) | 3.71× (3.57–3.85) | 6.99× (5.86–8.33) | 6.55× (5.48–7.83) | 2.05× (1.71–2.46) | 2.39× (2.02–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.86× (2.81–2.90) | 7.18× (7.06–7.31) | 7.56× (7.38–7.74) | 8.49× (8.31–8.66) | 7.64× (7.48–7.80) | 7.77× (7.65–7.90) | 7.40× (7.22–7.58) | 2.21× (2.18–2.24) | 2.55× (2.51–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.60× (2.51–2.68) | 6.41× (6.23–6.59) | 6.89× (6.66–7.12) | 7.95× (7.71–8.21) | 7.04× (6.59–7.52) | 6.85× (6.64–7.07) | 6.52× (6.29–6.77) | 1.95× (1.88–2.02) | 2.25× (2.17–2.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.46× (2.40–2.51) | 4.56× (4.46–4.67) | 5.04× (4.88–5.20) | 5.46× (5.28–5.65) | 4.87× (4.75–5.00) | 4.05× (3.90–4.22) | 3.86× (3.68–4.05) | 1.15× (1.11–1.20) | 1.33× (1.27–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.13× (3.07–3.20) | 3.93× (3.82–4.03) | 3.69× (3.34–4.08) | 6.06× (5.92–6.20) | 2.69× (2.62–2.76) | 2.69× (2.63–2.75) | 1.01× (0.99–1.04) | 1.88× (1.83–1.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.50–0.62) | 2.73× (2.47–3.03) | 4.77× (4.27–5.34) | 3.24× (2.92–3.60) | 5.41× (4.85–6.04) | 2.45× (2.19–2.73) | 2.45× (2.20–2.72) | 0.92× (0.82–1.04) | 1.71× (1.55–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.55–0.58) | 3.23× (2.87–3.63) | 5.04× (4.88–5.21) | 3.45× (3.35–3.54) | 5.84× (5.67–6.03) | 2.61× (2.51–2.72) | 2.61× (2.49–2.74) | 0.98× (0.95–1.02) | 1.83× (1.75–1.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.75× (2.71–2.78) | 3.19× (3.12–3.25) | 4.26× (3.54–5.12) | 4.57× (4.25–4.92) | 5.41× (4.82–6.07) | 2.93× (2.86–3.01) | 2.51× (2.47–2.56) | 2.09× (2.06–2.13) | 2.83× (2.75–2.91) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.41× (1.97–2.95) | 2.51× (2.42–2.60) | 3.20× (2.82–3.64) | 3.54× (3.21–3.90) | 3.88× (3.74–4.03) | 2.14× (2.08–2.20) | 1.83× (1.77–1.90) | 1.52× (1.47–1.58) | 2.06× (1.94–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.70× (1.38–2.09) | 1.51× (1.21–1.89) | 1.87× (1.48–2.36) | 1.84× (1.49–2.26) | 2.05× (1.67–2.52) | 0.99× (0.80–1.22) | 0.85× (0.68–1.05) | 0.71× (0.57–0.88) | 0.96× (0.77–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.35–1.39) | 3.04× (2.98–3.10) | 2.77× (2.72–2.82) | 2.73× (2.69–2.76) | 3.12× (3.08–3.17) | 3.48× (3.44–3.53) | 3.53× (3.49–3.58) | 1.34× (1.32–1.36) | 1.47× (1.45–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.75× (2.70–2.81) | 2.42× (2.38–2.47) | 3.09× (3.02–3.16) | 2.75× (2.69–2.82) | 3.05× (2.96–3.15) | 3.10× (3.03–3.16) | 1.17× (1.15–1.20) | 1.29× (1.26–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.41–1.48) | 2.63× (2.57–2.69) | 2.17× (2.11–2.23) | 2.83× (2.74–2.93) | 2.71× (2.65–2.78) | 2.30× (2.24–2.36) | 2.34× (2.30–2.37) | 0.88× (0.87–0.89) | 0.97× (0.96–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.48–1.50) | 3.65× (3.59–3.70) | 3.06× (2.97–3.15) | 3.20× (2.87–3.57) | 3.73× (3.29–4.22) | 4.41× (4.34–4.48) | 4.80× (4.68–4.92) | 1.63× (1.55–1.71) | 1.81× (1.72–1.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.19–1.80) | 3.32× (3.21–3.43) | 2.62× (2.55–2.69) | 3.66× (3.50–3.82) | 3.11× (2.93–3.29) | 3.78× (3.70–3.86) | 4.11× (3.98–4.24) | 1.39× (1.32–1.47) | 1.55× (1.47–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.27–1.81) | 2.77× (2.41–3.18) | 2.12× (1.85–2.43) | 2.97× (2.58–3.42) | 2.70× (2.35–3.11) | 2.56× (2.22–2.96) | 2.78× (2.45–3.17) | 0.94× (0.85–1.05) | 1.05× (0.95–1.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.71) | 2.57× (2.52–2.62) | 3.40× (3.33–3.47) | 2.90× (2.85–2.94) | 6.27× (6.15–6.39) | 2.21× (2.18–2.24) | 2.20× (2.14–2.25) | 0.98× (0.95–1.01) | 1.51× (1.48–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.60) | 2.37× (2.34–2.40) | 3.99× (3.92–4.06) | 2.80× (2.75–2.85) | 5.81× (5.69–5.94) | 2.09× (2.03–2.16) | 2.08× (2.03–2.13) | 0.93× (0.90–0.95) | 1.43× (1.40–1.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.55× (2.51–2.58) | 4.03× (3.96–4.09) | 2.85× (2.78–2.93) | 5.96× (5.79–6.14) | 2.11× (2.05–2.18) | 2.10× (2.06–2.15) | 0.94× (0.91–0.96) | 1.44× (1.41–1.47) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.66× (1.52–1.82) | 3.88× (3.48–4.32) | 4.07× (3.48–4.77) | 4.33× (3.70–5.07) | 4.59× (3.90–5.40) | 4.73× (4.00–5.58) | 4.44× (3.79–5.19) | 1.41× (1.21–1.64) | 1.53× (1.32–1.78) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.42× (1.39–1.44) | 3.32× (3.26–3.39) | 3.66× (3.57–3.75) | 4.23× (4.14–4.33) | 3.93× (3.81–4.07) | 3.92× (3.80–4.04) | 3.68× (3.58–3.78) | 1.17× (1.13–1.22) | 1.27× (1.24–1.30) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.41× (1.28–1.55) | 2.49× (2.13–2.90) | 2.68× (2.25–3.20) | 3.18× (2.60–3.89) | 3.09× (2.93–3.26) | 2.39× (1.99–2.88) | 2.24× (1.86–2.70) | 0.71× (0.59–0.86) | 0.78× (0.65–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.37) | 1.05× (1.05–1.06) | 0.85× (0.84–0.87) | 11.91× (11.90–11.93) | 0.28× (0.28–0.28) | 0.31× (0.31–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.35–0.37) | 0.58× (0.50–0.66) | 0.74× (0.72–0.75) | 0.63× (0.62–0.64) | 5.92× (5.82–6.02) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.38) | 0.58× (0.48–0.70) | 0.67× (0.66–0.69) | 0.59× (0.58–0.60) | 3.91× (3.88–3.94) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.38–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.67× (0.66–0.68) | 8.04× (8.01–8.06) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 24 MB | 0.65× (0.63–0.67) | 0.61× (0.57–0.65) | 1.54× (1.49–1.59) | 0.78× (0.75–0.80) | 5.58× (5.42–5.75) | 0.45× (0.44–0.46) | 0.48× (0.46–0.49) | 0.51× (0.50–0.53) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.71× (0.65–0.77) | 1.13× (1.08–1.18) | 0.71× (0.69–0.73) | 3.34× (3.20–3.48) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.53) | 0.40× (0.37–0.43) | 0.92× (0.90–0.95) | 0.75× (0.73–0.77) | 3.21× (3.12–3.30) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.67) | 0.46× (0.44–0.47) | 0.88× (0.85–0.90) | 0.80× (0.77–0.84) | 2.55× (2.48–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.35× (0.35–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.51) | 0.36× (0.33–0.40) | 0.86× (0.82–0.89) | 0.73× (0.71–0.76) | 2.95× (2.83–3.07) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.46× (0.43–0.50) | 0.80× (0.79–0.81) | 0.82× (0.79–0.85) | 2.31× (2.27–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.12× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.34) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.66) | 6.39× (6.36–6.41) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.31–0.32) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.43× (0.39–0.47) | 1.12× (1.08–1.15) | 0.72× (0.70–0.74) | 3.56× (3.50–3.62) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.45× (0.41–0.50) | 0.91× (0.89–0.93) | 0.73× (0.70–0.77) | 2.33× (2.28–2.37) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 693 | 1.45 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,147 | 1.61 | 1.13× (1.10–1.17) | 82% | 1.01× (0.99–1.03) | 1.03× (1.00–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,246 | 2.75 | 1.92× (1.86–1.98) | 44% | 1.22× (1.19–1.24) | 1.65× (1.57–1.73) |

**Best:** mibz2 at 4 threads (1,246 MB/s); lbzip2 at 4 threads (310 MB/s); pbz2 at 4 threads (501 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.90, mibz2 4t 1.22, lbzip2 1t 0.07, lbzip2 2t 0.31, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 15%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.42 | 0.01 | 0.09 | 0.168 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4217 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 14.4% |
| MTF + RLE2 | 1.48 | 21.0% |
| T-vector build | 1.29 | 18.3% |
| walk (the one each build uses) | 2.87 | 40.6% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.25 | 3.6% |
| **total** | 7.06 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.304 | 0.734 | (in its total) | 2.40× (2.36–2.45) |  |
| walk | 0.656 | 1.661 | (in its total) | 2.53× (2.48–2.58) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.028 | 0.031 | (in its total) | 1.07× (1.01–1.14) |  |
| inverse BWT, total | 0.988 | 2.425 | 1.356 | 2.45× (2.40–2.50) | 1.38× (1.33–1.42) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m38s ± 4m59s |
| 4 threads | 24m48s ± 3m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m21s ± 2m15s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 15m21s ± 2m15s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.01 ns a byte of output at 4 threads (CPU, summed over them), 1.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4217.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
