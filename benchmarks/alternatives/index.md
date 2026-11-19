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

The latest record: [`1740582`](<https://github.com/lmmx/mibz2/commit/1740582674ac8e6dcd90329a62a6c46a4b92fd29>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44588740255>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.404 s | 519 | 7.2 | 2.96× (2.89–3.03) | 6.76× (6.62–6.91) | 7.05× (6.91–7.20) | 8.02× (7.86–8.19) | 7.05× (6.89–7.22) | 7.36× (7.19–7.53) | 6.95× (6.79–7.11) | 2.20× (2.14–2.25) | 2.46× (2.41–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.232 s | 904 | 7.8 | 3.10× (2.93–3.27) | 6.22× (5.93–6.53) | 7.33× (6.95–7.73) | 7.44× (7.09–7.79) | 6.71× (6.31–7.15) | 12.78× (12.25–13.35) | 12.07× (11.58–12.58) | 3.82× (3.66–3.98) | 4.27× (4.10–4.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,068 | 12.2 | 2.91× (2.78–3.05) | 4.33× (4.08–4.59) | 5.35× (5.05–5.65) | 5.23× (4.96–5.52) | 5.18× (4.90–5.48) | 14.92× (14.12–15.76) | 14.09× (13.34–14.88) | 4.45× (4.21–4.71) | 4.98× (4.72–5.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,117 | 3.6 | 0.58× (0.57–0.59) | 3.44× (3.37–3.50) | 4.64× (4.51–4.77) | 3.59× (3.54–3.64) | 6.06× (5.98–6.14) | 2.67× (2.62–2.72) | 2.67× (2.63–2.72) | 0.99× (0.97–1.01) | 1.98× (1.77–2.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.53× (0.51–0.55) | 2.81× (2.75–2.88) | 5.64× (5.50–5.77) | 3.53× (3.44–3.62) | 5.92× (5.78–6.07) | 2.62× (2.55–2.68) | 2.62× (2.55–2.69) | 0.97× (0.94–1.01) | 1.94× (1.74–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,084 | 3.7 | 0.53× (0.52–0.54) | 2.79× (2.74–2.83) | 5.57× (5.48–5.66) | 3.49× (3.44–3.55) | 5.94× (5.85–6.04) | 2.58× (2.54–2.62) | 2.58× (2.54–2.63) | 0.96× (0.93–0.98) | 1.92× (1.72–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 745 | 5.3 | 2.72× (2.70–2.74) | 3.09× (3.08–3.11) | 3.94× (3.77–4.11) | 4.13× (4.11–4.16) | 4.48× (4.41–4.56) | 2.79× (2.77–2.81) | 2.29× (2.27–2.32) | 2.11× (2.09–2.13) | 3.12× (3.04–3.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,120 | 6.5 | 2.21× (1.87–2.61) | 2.73× (2.37–3.15) | 4.11× (3.53–4.80) | 3.54× (3.34–3.75) | 4.85× (4.18–5.61) | 3.91× (3.38–4.51) | 3.21× (2.78–3.71) | 2.96× (2.57–3.41) | 4.37× (3.76–5.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 963 | 13.0 | 1.71× (1.47–2.00) | 1.70× (1.40–2.06) | 2.90× (2.50–3.37) | 2.02× (1.70–2.39) | 3.73× (3.22–4.32) | 3.34× (2.87–3.89) | 2.75× (2.36–3.21) | 2.53× (2.17–2.96) | 3.75× (3.30–4.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 52.5 | 1.35× (1.31–1.40) | 2.57× (2.48–2.66) | 2.35× (2.27–2.43) | 2.45× (2.37–2.53) | 2.74× (2.65–2.82) | 2.94× (2.84–3.05) | 3.01× (2.92–3.12) | 1.23× (1.19–1.27) | 1.35× (1.30–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 58.5 | 1.27× (1.24–1.30) | 2.45× (2.42–2.48) | 2.33× (2.26–2.40) | 3.03× (2.96–3.10) | 2.82× (2.79–2.86) | 4.81× (4.68–4.93) | 4.92× (4.81–5.03) | 2.00× (1.96–2.04) | 2.21× (2.17–2.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 142 | 77.2 | 1.32× (1.29–1.35) | 2.10× (2.02–2.17) | 2.13× (2.03–2.24) | 2.64× (2.56–2.71) | 2.87× (2.82–2.93) | 5.89× (5.74–6.04) | 6.03× (5.91–6.15) | 2.45× (2.40–2.51) | 2.71× (2.64–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.3 | 1.52× (1.48–1.56) | 3.03× (2.95–3.11) | 2.67× (2.59–2.74) | 2.77× (2.73–2.81) | 3.14× (3.01–3.27) | 3.67× (3.59–3.75) | 3.96× (3.87–4.06) | 1.51× (1.46–1.56) | 1.79× (1.75–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 53.1 | 1.34× (1.30–1.38) | 2.76× (2.66–2.86) | 2.50× (2.41–2.59) | 3.46× (3.35–3.58) | 3.04× (2.93–3.15) | 5.91× (5.72–6.11) | 6.39× (6.17–6.61) | 2.44× (2.34–2.54) | 2.88× (2.79–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 74.3 | 1.41× (1.36–1.45) | 2.27× (2.17–2.37) | 2.20× (2.11–2.30) | 3.04× (2.99–3.11) | 2.86× (2.78–2.93) | 6.95× (6.79–7.12) | 7.51× (7.31–7.71) | 2.86× (2.78–2.96) | 3.39× (3.30–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 848 | 5.0 | 0.63× (0.62–0.64) | 3.46× (3.41–3.50) | 4.34× (4.29–4.39) | 3.10× (3.06–3.13) | 6.73× (6.59–6.87) | 2.29× (2.26–2.31) | 2.38× (2.23–2.53) | 0.97× (0.94–0.99) | 1.74× (1.72–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 790 | 5.4 | 0.51× (0.50–0.52) | 2.45× (2.41–2.50) | 4.93× (4.83–5.03) | 2.94× (2.88–2.99) | 6.34× (6.20–6.49) | 2.15× (2.11–2.19) | 2.24× (2.09–2.39) | 0.91× (0.89–0.94) | 1.64× (1.61–1.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 790 | 5.3 | 0.51× (0.49–0.53) | 2.43× (2.32–2.54) | 4.86× (4.64–5.08) | 2.91× (2.78–3.04) | 6.16× (5.91–6.41) | 2.12× (2.03–2.21) | 2.20× (2.04–2.38) | 0.90× (0.85–0.95) | 1.61× (1.54–1.68) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 0.059 s | 226 | 16.8 | 1.79× (1.76–1.82) | 3.84× (3.75–3.92) | 4.27× (4.18–4.37) | 4.51× (4.41–4.60) | 4.64× (4.54–4.73) | 4.61× (4.52–4.71) | 4.54× (4.44–4.65) | 1.47× (1.45–1.50) | 1.72× (1.69–1.76) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 0.039 s | 338 | 20.9 | 1.50× (1.46–1.55) | 3.20× (3.04–3.38) | 3.86× (3.72–4.01) | 4.34× (4.11–4.58) | 4.36× (4.20–4.53) | 6.81× (6.57–7.07) | 6.71× (6.46–6.98) | 2.18× (2.10–2.26) | 2.55× (2.45–2.64) | yes |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 0.036 s | 367 | 31.5 | 1.49× (1.44–1.55) | 2.53× (2.35–2.72) | 3.06× (2.94–3.19) | 3.35× (3.23–3.48) | 3.87× (3.71–4.04) | 7.50× (7.26–7.75) | 7.39× (7.13–7.65) | 2.39× (2.31–2.48) | 2.80× (2.71–2.90) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.95× (2.88–3.02) | 6.73× (6.58–6.87) | 7.02× (6.87–7.16) | 7.99× (7.83–8.16) | 7.03× (6.86–7.19) | 7.33× (7.16–7.50) | 6.92× (6.76–7.08) | 2.19× (2.14–2.24) | 2.44× (2.39–2.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.81–2.88) | 6.37× (6.33–6.42) | 6.73× (6.66–6.80) | 7.75× (7.70–7.80) | 6.68× (6.37–7.02) | 6.76× (6.71–6.81) | 6.38× (6.34–6.43) | 2.02× (2.01–2.03) | 2.25× (2.24–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.92× (2.85–2.99) | 4.85× (4.71–4.98) | 5.27× (5.14–5.41) | 5.90× (5.73–6.07) | 5.15× (5.00–5.30) | 4.32× (4.20–4.44) | 4.08× (3.96–4.20) | 1.29× (1.26–1.33) | 1.44× (1.40–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 2.95× (2.89–3.01) | 3.92× (3.85–4.01) | 3.35× (3.30–3.39) | 5.61× (5.54–5.68) | 2.52× (2.48–2.56) | 2.52× (2.49–2.56) | 0.99× (0.98–1.01) | 1.65× (1.62–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.72× (2.66–2.79) | 4.96× (4.84–5.09) | 3.28× (3.21–3.36) | 5.42× (5.30–5.54) | 2.44× (2.38–2.49) | 2.44× (2.38–2.49) | 0.96× (0.93–0.99) | 1.60× (1.56–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.56–0.57) | 3.00× (2.95–3.04) | 4.93× (4.86–5.00) | 3.31× (3.26–3.36) | 5.53× (5.44–5.62) | 2.46× (2.42–2.49) | 2.46× (2.41–2.50) | 0.97× (0.94–0.99) | 1.61× (1.58–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.62× (2.60–2.64) | 2.86× (2.84–2.87) | 3.46× (3.30–3.64) | 3.94× (3.92–3.97) | 4.28× (4.21–4.35) | 2.68× (2.66–2.70) | 2.21× (2.19–2.24) | 2.05× (2.03–2.06) | 2.70× (2.62–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.26–2.37) | 2.52× (2.44–2.60) | 3.38× (3.29–3.47) | 3.39× (3.32–3.46) | 3.66× (3.58–3.75) | 2.19× (2.14–2.23) | 1.81× (1.77–1.85) | 1.67× (1.64–1.70) | 2.20× (2.12–2.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.88× (1.59–2.24) | 1.61× (1.35–1.93) | 2.13× (1.78–2.55) | 1.93× (1.63–2.28) | 2.01× (1.70–2.37) | 1.03× (0.87–1.22) | 0.85× (0.72–1.01) | 0.79× (0.67–0.93) | 1.04× (0.91–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.30–1.39) | 2.50× (2.43–2.58) | 2.29× (2.22–2.37) | 2.43× (2.35–2.50) | 2.71× (2.63–2.79) | 2.91× (2.82–3.01) | 2.98× (2.89–3.08) | 1.22× (1.18–1.26) | 1.33× (1.29–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.29) | 2.39× (2.35–2.43) | 2.12× (2.08–2.15) | 2.87× (2.82–2.93) | 2.48× (2.44–2.52) | 2.61× (2.54–2.68) | 2.67× (2.62–2.73) | 1.10× (1.08–1.11) | 1.20× (1.17–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.44–1.52) | 2.26× (2.21–2.31) | 1.92× (1.87–1.98) | 2.60× (2.54–2.67) | 2.46× (2.40–2.52) | 1.99× (1.94–2.04) | 2.04× (2.00–2.08) | 0.83× (0.81–0.86) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.48–1.55) | 2.97× (2.90–3.03) | 2.60× (2.52–2.68) | 2.74× (2.71–2.78) | 3.11× (2.98–3.23) | 3.63× (3.56–3.71) | 3.92× (3.83–4.01) | 1.51× (1.46–1.56) | 1.73× (1.69–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.40) | 2.73× (2.67–2.80) | 2.34× (2.29–2.40) | 3.28× (3.20–3.37) | 2.74× (2.68–2.81) | 3.18× (3.10–3.25) | 3.43× (3.35–3.51) | 1.32× (1.27–1.36) | 1.51× (1.47–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.57–1.65) | 2.50× (2.45–2.57) | 1.98× (1.93–2.03) | 2.86× (2.77–2.95) | 2.59× (2.51–2.66) | 2.28× (2.22–2.35) | 2.46× (2.39–2.54) | 0.95× (0.92–0.97) | 1.09× (1.06–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.69) | 2.46× (2.43–2.50) | 3.44× (3.36–3.52) | 2.83× (2.80–2.86) | 6.00× (5.88–6.13) | 2.12× (2.09–2.14) | 2.21× (2.06–2.37) | 0.97× (0.95–1.00) | 1.37× (1.35–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.23× (2.19–2.26) | 4.00× (3.94–4.06) | 2.71× (2.67–2.75) | 5.60× (5.49–5.71) | 1.97× (1.94–2.00) | 2.05× (1.91–2.21) | 0.90× (0.88–0.92) | 1.27× (1.25–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.52–0.59) | 2.44× (2.29–2.61) | 3.94× (3.68–4.22) | 2.72× (2.54–2.91) | 5.53× (5.17–5.90) | 1.97× (1.84–2.11) | 2.06× (1.86–2.27) | 0.90× (0.84–0.98) | 1.28× (1.19–1.37) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 1.77× (1.74–1.80) | 3.73× (3.64–3.81) | 4.07× (4.00–4.15) | 4.41× (4.32–4.50) | 4.54× (4.45–4.63) | 4.51× (4.42–4.61) | 4.45× (4.35–4.54) | 1.46× (1.43–1.49) | 1.60× (1.56–1.63) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 1.52× (1.50–1.54) | 3.15× (3.09–3.22) | 3.50× (3.43–3.57) | 4.14× (4.06–4.22) | 3.77× (3.70–3.84) | 3.67× (3.60–3.74) | 3.61× (3.54–3.68) | 1.19× (1.17–1.21) | 1.30× (1.27–1.33) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 1.66× (1.60–1.72) | 2.61× (2.45–2.78) | 2.89× (2.79–2.99) | 3.29× (3.19–3.40) | 3.09× (2.98–3.20) | 2.41× (2.37–2.45) | 2.37× (2.33–2.42) | 0.78× (0.77–0.79) | 0.85× (0.84–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.32× (0.31–0.34) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.95× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.34) | 0.43× (0.37–0.50) | 0.67× (0.64–0.71) | 0.57× (0.54–0.60) | 5.35× (5.13–5.58) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.31–0.34) | 0.46× (0.39–0.55) | 0.58× (0.55–0.60) | 0.51× (0.49–0.53) | 3.38× (3.24–3.54) | 0.08× (0.08–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.28–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.65–0.66) | 8.00× (7.98–8.02) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.60× (0.55–0.66) | 1.58× (1.54–1.63) | 0.80× (0.77–0.82) | 5.76× (5.58–5.94) | 0.46× (0.45–0.48) | 0.49× (0.47–0.50) | 0.54× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.62–0.66) | 0.68× (0.63–0.73) | 1.14× (1.10–1.19) | 0.70× (0.66–0.73) | 3.34× (3.21–3.48) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.16× (5.14–5.17) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.41× (0.40–0.42) | 0.89× (0.86–0.91) | 0.72× (0.70–0.74) | 3.07× (2.99–3.16) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.67) | 0.45× (0.44–0.47) | 0.86× (0.83–0.89) | 0.81× (0.78–0.85) | 2.52× (2.44–2.60) | 0.15× (0.15–0.15) | 0.16× (0.15–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.36× (0.33–0.39) | 0.85× (0.82–0.88) | 0.71× (0.68–0.75) | 2.91× (2.80–3.02) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.49× (0.46–0.51) | 0.80× (0.79–0.81) | 0.82× (0.80–0.85) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.24× (0.23–0.25) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.35× (5.35–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.77× (0.76–0.77) | 5.27× (5.26–5.29) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.31) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.66) | 6.41× (6.39–6.43) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.36× (0.33–0.40) | 1.01× (0.97–1.04) | 0.66× (0.64–0.69) | 3.22× (3.14–3.30) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.32) | 0.15× (0.15–0.16) |
| `text` (2.6 MB → 13.2 MB, 5.1×) | 4 | 59 MB | 0.48× (0.47–0.49) | 0.43× (0.39–0.47) | 0.87× (0.84–0.90) | 0.73× (0.71–0.76) | 2.23× (2.17–2.30) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 519 | 1.94 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 904 | 2.12 | 1.08× (1.06–1.11) | 87% | 1.03× (1.02–1.03) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,068 | 3.31 | 1.70× (1.64–1.75) | 51% | 1.22× (1.21–1.24) | 1.68× (1.63–1.73) |

**Best:** mibz2 at 4 threads (1,068 MB/s); lbzip2 at 4 threads (241 MB/s); pbz2 at 4 threads (359 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.23, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.53 | 0.02 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3697 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.9% |
| MTF + RLE2 | 1.52 | 17.7% |
| T-vector build | 1.49 | 17.3% |
| walk (the one each build uses) | 4.04 | 47.0% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.61 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.061 | (in its total) | 2.72× (2.70–2.74) |  |
| walk | 1.044 | 2.017 | (in its total) | 1.94× (1.93–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.040 | (in its total) | 1.08× (1.07–1.09) |  |
| inverse BWT, total | 1.472 | 3.118 | 1.870 | 2.13× (2.11–2.14) | 1.27× (1.27–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m32s ± 6m53s |
| 4 threads | 29m25s ± 4m27s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m08s ± 2m47s |
| the scanner alone (its serial part) | 11m30s ± 19.7s |
| the workers alone, over 4 | 19m08s ± 2m47s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.51 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,599,481 | 13,234,711 | \`71fd43c7474b3515…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3697.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
