# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-316/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`755f961`](<https://github.com/lmmx/mibz2/commit/755f9612278c0ae455570ab613f946362f3ad8b0>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42796085183>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.366 s | 572 | 4.6 | 2.40× (2.37–2.44) | 6.10× (5.99–6.21) | 6.40× (6.28–6.51) | 7.22× (7.05–7.38) | 6.36× (6.28–6.44) | 6.60× (6.50–6.70) | 6.26× (6.19–6.34) | 1.89× (1.86–1.91) | 2.14× (2.09–2.19) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.224 s | 938 | 5.1 | 2.29× (2.13–2.46) | 5.19× (4.87–5.53) | 6.24× (5.83–6.67) | 6.34× (5.98–6.72) | 5.64× (5.31–5.98) | 10.85× (10.19–11.56) | 10.31× (9.68–10.97) | 3.11× (2.90–3.33) | 3.52× (3.29–3.77) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.198 s | 1,061 | 8.4 | 2.11× (1.95–2.29) | 3.39× (3.13–3.67) | 4.34× (4.00–4.70) | 4.09× (3.75–4.47) | 4.11× (3.78–4.48) | 12.01× (10.95–13.18) | 11.41× (10.43–12.47) | 3.44× (3.14–3.77) | 3.89× (3.57–4.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,394 | 2.1 | 0.57× (0.54–0.60) | 3.49× (3.35–3.64) | 4.43× (4.18–4.71) | 3.80× (3.37–4.27) | 6.44× (6.19–6.70) | 2.75× (2.64–2.87) | 2.87× (2.62–3.14) | 0.96× (0.92–1.01) | 2.10× (2.01–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,399 | 2.1 | 0.52× (0.51–0.54) | 3.17× (2.94–3.42) | 5.86× (5.71–6.03) | 3.74× (3.63–3.85) | 6.52× (6.31–6.74) | 2.79× (2.70–2.87) | 2.91× (2.65–3.19) | 0.97× (0.96–0.99) | 2.13× (2.07–2.18) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,324 | 2.2 | 0.51× (0.49–0.53) | 2.99× (2.77–3.23) | 5.83× (5.65–6.01) | 3.60× (3.52–3.69) | 6.27× (6.06–6.47) | 2.68× (2.58–2.79) | 2.80× (2.54–3.08) | 0.94× (0.90–0.97) | 2.04× (1.96–2.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,005 | 2.8 | 2.64× (2.37–2.94) | 3.25× (2.90–3.65) | 3.83× (3.42–4.28) | 4.20× (3.76–4.70) | 4.81× (4.32–5.36) | 2.81× (2.49–3.16) | 2.36× (2.11–2.64) | 1.97× (1.77–2.19) | 2.95× (2.62–3.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,385 | 3.5 | 2.04× (1.73–2.41) | 2.92× (2.31–3.70) | 3.81× (3.17–4.59) | 3.47× (2.97–4.05) | 5.01× (4.27–5.89) | 3.84× (3.27–4.52) | 3.23× (2.77–3.77) | 2.69× (2.32–3.12) | 4.05× (3.45–4.76) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,174 | 7.2 | 1.63× (1.59–1.66) | 1.87× (1.78–1.96) | 2.76× (2.56–2.97) | 2.14× (1.90–2.41) | 4.15× (3.99–4.32) | 3.44× (3.30–3.59) | 2.89× (2.79–3.00) | 2.41× (2.33–2.50) | 3.62× (3.47–3.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.082 s | 98 | 27.0 | 1.47× (1.36–1.58) | 3.25× (3.13–3.39) | 2.87× (2.79–2.97) | 2.82× (2.74–2.90) | 3.19× (3.09–3.29) | 3.60× (3.55–3.65) | 3.59× (3.53–3.65) | 1.39× (1.36–1.41) | 1.60× (1.51–1.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 30.7 | 1.33× (1.26–1.42) | 2.97× (2.85–3.09) | 2.71× (2.61–2.81) | 3.41× (3.33–3.49) | 3.22× (3.08–3.37) | 5.64× (5.53–5.75) | 5.62× (5.54–5.71) | 2.17× (2.13–2.21) | 2.51× (2.36–2.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 191 | 40.8 | 1.29× (1.25–1.33) | 2.48× (2.43–2.54) | 2.50× (2.39–2.61) | 2.92× (2.84–3.01) | 3.18× (3.06–3.29) | 6.83× (6.55–7.12) | 6.81× (6.59–7.04) | 2.63× (2.53–2.74) | 3.03× (2.91–3.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.086 s | 112 | 23.7 | 1.50× (1.46–1.54) | 3.73× (3.61–3.86) | 3.12× (2.97–3.27) | 3.17× (3.11–3.24) | 3.57× (3.48–3.66) | 4.48× (4.32–4.64) | 4.78× (4.70–4.87) | 1.63× (1.55–1.71) | 1.85× (1.81–1.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 26.8 | 1.46× (1.26–1.68) | 3.21× (3.03–3.40) | 2.79× (2.66–2.92) | 3.89× (3.75–4.04) | 3.34× (3.14–3.56) | 7.03× (6.72–7.36) | 7.51× (7.16–7.88) | 2.56× (2.40–2.72) | 2.90× (2.77–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 204 | 38.3 | 1.30× (1.16–1.46) | 2.53× (2.30–2.78) | 2.43× (2.19–2.70) | 3.26× (2.90–3.66) | 3.01× (2.70–3.35) | 7.97× (7.16–8.87) | 8.52× (7.63–9.51) | 2.90× (2.57–3.28) | 3.29× (2.99–3.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 2.9 | 0.60× (0.59–0.62) | 3.15× (3.07–3.24) | 4.36× (4.25–4.48) | 3.15× (3.05–3.27) | 7.06× (6.85–7.28) | 2.32× (2.26–2.38) | 2.41× (2.27–2.56) | 0.97× (0.94–0.99) | 2.11× (2.05–2.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,005 | 3.1 | 0.56× (0.46–0.70) | 3.12× (3.02–3.22) | 5.37× (5.20–5.55) | 3.12× (3.03–3.21) | 6.87× (6.64–7.11) | 2.29× (2.23–2.35) | 2.38× (2.28–2.48) | 0.95× (0.93–0.98) | 2.08× (2.02–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 952 | 3.1 | 0.50× (0.49–0.51) | 2.95× (2.91–3.00) | 5.06× (4.98–5.14) | 2.98× (2.92–3.04) | 6.80× (6.30–7.35) | 2.16× (2.12–2.21) | 2.25× (2.12–2.38) | 0.90× (0.88–0.92) | 1.97× (1.94–2.00) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.058 s | 212 | 12.6 | 1.28× (1.25–1.31) | 3.21× (3.09–3.34) | 3.42× (3.31–3.54) | 3.52× (3.42–3.62) | 3.75× (3.66–3.84) | 3.82× (3.73–3.91) | 3.64× (3.55–3.72) | 1.11× (1.08–1.14) | 1.31× (1.25–1.36) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.038 s | 328 | 15.0 | 1.13× (1.04–1.23) | 2.77× (2.61–2.95) | 3.12× (3.05–3.20) | 3.92× (3.72–4.12) | 3.64× (3.47–3.82) | 5.87× (5.63–6.12) | 5.59× (5.36–5.83) | 1.71× (1.64–1.78) | 2.01× (1.92–2.10) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.032 s | 381 | 22.7 | 1.04× (0.90–1.19) | 2.00× (1.71–2.36) | 2.59× (2.18–3.07) | 2.93× (2.55–3.36) | 3.21× (2.78–3.70) | 6.52× (5.71–7.46) | 6.21× (5.41–7.12) | 1.90× (1.66–2.17) | 2.23× (1.91–2.60) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.40× (2.36–2.44) | 6.06× (5.95–6.17) | 6.36× (6.24–6.48) | 7.19× (7.02–7.36) | 6.33× (6.25–6.42) | 6.57× (6.47–6.67) | 6.24× (6.16–6.32) | 1.88× (1.86–1.91) | 2.12× (2.08–2.17) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.25× (2.17–2.33) | 5.56× (5.42–5.70) | 5.87× (5.70–6.05) | 6.88× (6.74–7.04) | 5.83× (5.68–5.99) | 5.99× (5.81–6.16) | 5.69× (5.55–5.82) | 1.72× (1.66–1.78) | 1.93× (1.89–1.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.01× (1.85–2.19) | 3.71× (3.43–4.00) | 4.02× (3.73–4.34) | 4.48× (4.13–4.86) | 3.90× (3.60–4.24) | 3.37× (3.09–3.67) | 3.20× (2.94–3.48) | 0.97× (0.89–1.05) | 1.09× (1.00–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.58–0.64) | 3.02× (2.90–3.15) | 3.85× (3.70–4.00) | 3.54× (3.14–3.98) | 5.92× (5.71–6.13) | 2.59× (2.49–2.69) | 2.70× (2.47–2.95) | 0.97× (0.92–1.01) | 1.82× (1.74–1.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.87× (2.81–2.94) | 5.01× (4.94–5.08) | 3.42× (3.33–3.52) | 5.87× (5.67–6.07) | 2.56× (2.49–2.63) | 2.67× (2.44–2.92) | 0.96× (0.94–0.97) | 1.80× (1.76–1.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.58) | 3.02× (2.93–3.12) | 5.09× (4.97–5.22) | 3.39× (3.32–3.46) | 5.77× (5.59–5.94) | 2.53× (2.44–2.63) | 2.64× (2.41–2.90) | 0.95× (0.92–0.98) | 1.78× (1.72–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.27–2.78) | 2.89× (2.60–3.20) | 3.40× (3.07–3.77) | 3.97× (3.57–4.41) | 4.53× (4.10–5.01) | 2.67× (2.39–2.99) | 2.26× (2.03–2.51) | 1.89× (1.71–2.09) | 2.55× (2.30–2.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.23× (2.13–2.34) | 2.75× (2.30–3.29) | 3.08× (2.95–3.23) | 3.44× (3.30–3.59) | 3.84× (3.67–4.02) | 2.21× (2.09–2.34) | 1.86× (1.78–1.95) | 1.56× (1.51–1.62) | 2.11× (2.00–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.84× (1.74–1.96) | 1.67× (1.58–1.76) | 2.03× (1.92–2.15) | 2.03× (1.92–2.14) | 2.23× (2.13–2.33) | 1.10× (1.04–1.17) | 0.93× (0.89–0.97) | 0.78× (0.75–0.81) | 1.05× (1.00–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.46× (1.36–1.57) | 3.14× (3.02–3.27) | 2.79× (2.71–2.88) | 2.79× (2.70–2.87) | 3.15× (3.05–3.25) | 3.55× (3.50–3.60) | 3.54× (3.48–3.59) | 1.38× (1.36–1.40) | 1.52× (1.43–1.61) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.32) | 2.87× (2.76–2.99) | 2.45× (2.38–2.52) | 3.27× (3.20–3.34) | 2.82× (2.70–2.94) | 3.08× (3.00–3.17) | 3.08× (3.02–3.13) | 1.20× (1.17–1.22) | 1.32× (1.25–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.39–1.48) | 2.59× (2.53–2.65) | 2.14× (2.07–2.20) | 2.88× (2.79–2.97) | 2.67× (2.57–2.78) | 2.29× (2.20–2.39) | 2.29× (2.21–2.37) | 0.89× (0.86–0.93) | 0.98× (0.95–1.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.45–1.53) | 3.63× (3.51–3.75) | 3.02× (2.90–3.15) | 3.14× (3.07–3.20) | 3.53× (3.43–3.62) | 4.41× (4.26–4.58) | 4.72× (4.63–4.80) | 1.62× (1.54–1.70) | 1.78× (1.74–1.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.27–1.67) | 3.32× (3.22–3.42) | 2.62× (2.53–2.72) | 3.71× (3.58–3.84) | 3.06× (2.91–3.23) | 3.87× (3.73–4.01) | 4.13× (4.00–4.28) | 1.42× (1.35–1.49) | 1.56× (1.52–1.60) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.48× (1.32–1.66) | 2.83× (2.55–3.13) | 2.14× (1.92–2.38) | 3.03× (2.71–3.40) | 2.71× (2.42–3.03) | 2.64× (2.37–2.95) | 2.83× (2.52–3.18) | 0.97× (0.86–1.10) | 1.07× (0.96–1.19) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.58× (2.50–2.68) | 3.31× (3.20–3.43) | 2.86× (2.77–2.96) | 6.23× (6.08–6.39) | 2.14× (2.10–2.18) | 2.24× (2.12–2.37) | 0.97× (0.94–1.00) | 1.50× (1.44–1.57) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.65× (0.52–0.82) | 2.39× (2.34–2.43) | 4.09× (3.94–4.24) | 2.81× (2.74–2.88) | 5.93× (5.74–6.13) | 2.04× (2.01–2.08) | 2.14× (2.05–2.23) | 0.93× (0.91–0.95) | 1.43× (1.38–1.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.50× (2.46–2.56) | 3.96× (3.84–4.08) | 2.80× (2.74–2.86) | 6.11× (5.66–6.59) | 2.03× (1.98–2.07) | 2.12× (2.01–2.24) | 0.92× (0.90–0.94) | 1.42× (1.35–1.49) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.27× (1.24–1.30) | 3.06× (2.94–3.17) | 3.26× (3.16–3.36) | 3.44× (3.33–3.55) | 3.66× (3.55–3.77) | 3.73× (3.63–3.84) | 3.55× (3.46–3.65) | 1.10× (1.07–1.14) | 1.20× (1.17–1.23) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.17× (1.09–1.25) | 2.67× (2.60–2.74) | 2.91× (2.87–2.96) | 3.38× (3.33–3.42) | 3.16× (3.08–3.24) | 3.13× (3.05–3.20) | 2.98× (2.91–3.05) | 0.93× (0.91–0.94) | 1.01× (0.99–1.03) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.16× (1.03–1.31) | 2.06× (1.83–2.33) | 2.36× (2.03–2.75) | 2.54× (2.24–2.89) | 2.49× (2.18–2.83) | 2.00× (1.76–2.28) | 1.91× (1.68–2.17) | 0.59× (0.52–0.67) | 0.65× (0.57–0.74) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.38) | 1.07× (1.07–1.07) | 0.87× (0.85–0.89) | 12.10× (12.07–12.12) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.39) | 0.52× (0.44–0.61) | 0.79× (0.77–0.81) | 0.65× (0.64–0.67) | 6.24× (6.11–6.38) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.51× (0.44–0.58) | 0.69× (0.69–0.70) | 0.61× (0.59–0.62) | 4.07× (4.04–4.10) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.17) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.67× (0.66–0.67) | 8.05× (8.02–8.08) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.70) | 0.63× (0.58–0.69) | 1.57× (1.53–1.62) | 0.78× (0.76–0.81) | 5.71× (5.54–5.88) | 0.46× (0.45–0.47) | 0.49× (0.47–0.50) | 0.52× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.59–0.64) | 0.65× (0.59–0.71) | 1.12× (1.07–1.16) | 0.68× (0.65–0.72) | 3.25× (3.12–3.38) | 0.25× (0.24–0.26) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.47–0.53) | 0.42× (0.40–0.44) | 0.90× (0.85–0.94) | 0.72× (0.68–0.75) | 3.10× (2.95–3.26) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.29–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.62–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.81–0.85) | 2.56× (2.50–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.84× (0.83–0.84) | 5.22× (5.22–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.51) | 0.38× (0.37–0.41) | 0.85× (0.82–0.90) | 0.70× (0.68–0.72) | 2.94× (2.81–3.08) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.62) | 0.46× (0.44–0.48) | 0.81× (0.79–0.82) | 0.82× (0.79–0.86) | 2.33× (2.28–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.41× (5.40–5.42) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.62× (1.61–1.63) | 0.77× (0.77–0.78) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.62) | 0.77× (0.77–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.46–1.47) | 0.69× (0.68–0.69) | 6.64× (6.63–6.65) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 0.65× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.38× (0.35–0.41) | 1.07× (1.03–1.10) | 0.68× (0.67–0.71) | 3.35× (3.26–3.46) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.17× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.47× (0.43–0.51) | 0.92× (0.89–0.94) | 0.74× (0.70–0.77) | 2.36× (2.29–2.43) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 572 | 1.75 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 938 | 1.95 | 1.10× (1.06–1.13) | 82% | 1.01× (0.98–1.03) | 1.03× (0.99–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,061 | 3.23 | 1.95× (1.79–2.12) | 46% | 1.19× (1.17–1.21) | 1.63× (1.59–1.68) |

**Best:** mibz2 at 4 threads (1,061 MB/s); lbzip2 at 4 threads (311 MB/s); pbz2 at 4 threads (499 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.22, lbzip2 1t 0.07, lbzip2 2t 0.24, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.54. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.52 | 0.01 | 0.09 | 0.198 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 40.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.63 | 14.1% |
| MTF + RLE2 | 0.96 | 21.5% |
| T-vector build | 0.80 | 17.9% |
| walk (the one each build uses) | 1.82 | 40.7% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.10 | 2.2% |
| CRC | 0.16 | 3.5% |
| **total** | 4.48 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.303 | 0.728 | (in its total) | 2.32× (2.20–2.45) |  |
| walk | 0.660 | 1.652 | (in its total) | 2.43× (2.31–2.55) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.029 | 0.030 | (in its total) | 1.04× (0.96–1.14) |  |
| inverse BWT, total | 0.992 | 2.413 | 1.329 | 2.36× (2.24–2.48) | 1.31× (1.24–1.39) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 53m09s ± 6m16s |
| 4 threads | 29m23s ± 4m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m07s ± 3m28s |
| the scanner alone (its serial part) | 9m27s ± 53.1s |
| the workers alone, over 4 | 20m07s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 2.64 ns a byte of output at 4 threads (CPU, summed over them), 1.43 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
