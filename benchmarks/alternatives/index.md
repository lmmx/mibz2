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

The latest record: [`cf107b8`](<https://github.com/lmmx/mibz2/commit/cf107b86568ec4b35717a3017be107ffbe4bb06a>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42582342006>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.653 s | 321 | 10.1 | 2.32× (2.31–2.34) | 5.38× (5.34–5.42) | 5.58× (5.51–5.66) | 6.02× (5.97–6.06) | 5.36× (5.32–5.41) | 5.82× (5.80–5.85) | 5.41× (5.39–5.44) | 1.74× (1.73–1.75) | 1.99× (1.97–2.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.375 s | 559 | 10.8 | 2.29× (2.11–2.48) | 4.79× (4.50–5.10) | 5.65× (5.27–6.07) | 5.55× (5.24–5.88) | 4.89× (4.61–5.18) | 9.92× (9.38–10.49) | 9.23× (8.71–9.78) | 2.96× (2.80–3.14) | 3.39× (3.20–3.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.295 s | 711 | 17.1 | 2.40× (2.36–2.44) | 3.73× (3.67–3.79) | 4.61× (4.50–4.73) | 4.20× (4.11–4.29) | 4.22× (4.15–4.29) | 12.78× (12.55–13.01) | 11.88× (11.67–12.09) | 3.82× (3.75–3.88) | 4.37× (4.28–4.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.6 | 0.60× (0.59–0.60) | 3.65× (3.49–3.83) | 4.49× (4.36–4.61) | 3.59× (3.55–3.64) | 6.21× (6.14–6.28) | 2.84× (2.80–2.88) | 2.67× (2.64–2.70) | 1.01× (0.99–1.03) | 1.51× (1.50–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 957 | 3.8 | 0.57× (0.55–0.58) | 2.98× (2.93–3.03) | 5.88× (5.78–5.98) | 3.55× (3.48–3.62) | 6.03× (5.92–6.15) | 2.80× (2.75–2.85) | 2.63× (2.59–2.68) | 0.99× (0.97–1.01) | 1.49× (1.47–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 976 | 3.6 | 0.57× (0.56–0.59) | 3.00× (2.95–3.04) | 5.88× (5.80–5.97) | 3.58× (3.48–3.67) | 6.05× (5.94–6.17) | 2.80× (2.76–2.84) | 2.63× (2.59–2.67) | 0.99× (0.97–1.02) | 1.49× (1.47–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 544 | 6.3 | 2.43× (2.41–2.46) | 2.83× (2.81–2.85) | 3.62× (3.60–3.65) | 3.56× (3.53–3.59) | 3.88× (3.85–3.91) | 2.34× (2.32–2.36) | 1.97× (1.96–1.99) | 1.90× (1.88–1.92) | 2.81× (2.79–2.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 845 | 7.7 | 2.10× (1.99–2.22) | 2.68× (2.46–2.92) | 3.79× (3.53–4.08) | 3.13× (2.90–3.38) | 4.47× (4.17–4.80) | 3.50× (3.26–3.77) | 2.96× (2.75–3.18) | 2.85× (2.64–3.07) | 4.21× (3.92–4.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 764 | 14.6 | 1.86× (1.81–1.91) | 1.88× (1.76–2.02) | 2.91× (2.77–3.06) | 2.14× (2.06–2.22) | 3.81× (3.71–3.92) | 3.27× (3.19–3.36) | 2.77× (2.70–2.83) | 2.66× (2.59–2.73) | 3.94× (3.84–4.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 53.1 | 1.36× (1.35–1.37) | 2.75× (2.72–2.78) | 2.47× (2.42–2.52) | 2.50× (2.47–2.54) | 2.88× (2.83–2.93) | 3.14× (3.09–3.18) | 3.16× (3.10–3.22) | 1.29× (1.26–1.32) | 1.44× (1.41–1.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 99 | 60.6 | 1.26× (1.22–1.30) | 2.53× (2.48–2.59) | 2.38× (2.31–2.45) | 3.07× (2.98–3.16) | 2.92× (2.84–3.00) | 5.02× (4.88–5.16) | 5.05× (4.89–5.22) | 2.07× (2.01–2.13) | 2.31× (2.25–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 82.4 | 1.30× (1.29–1.31) | 2.16× (2.10–2.21) | 2.08× (2.02–2.15) | 2.56× (2.53–2.59) | 2.82× (2.78–2.86) | 5.93× (5.85–6.02) | 5.97× (5.84–6.10) | 2.45× (2.37–2.52) | 2.73× (2.67–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 49.0 | 1.45× (1.44–1.46) | 3.08× (3.04–3.13) | 2.59× (2.56–2.63) | 2.67× (2.63–2.71) | 3.09× (3.05–3.13) | 3.78× (3.75–3.82) | 3.87× (3.83–3.92) | 1.44× (1.43–1.46) | 1.70× (1.68–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 112 | 55.7 | 1.36× (1.33–1.39) | 2.83× (2.78–2.89) | 2.51× (2.43–2.59) | 3.50× (3.41–3.60) | 3.12× (3.04–3.20) | 6.27× (6.10–6.44) | 6.42× (6.24–6.59) | 2.39× (2.33–2.46) | 2.82× (2.74–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 125 | 80.0 | 1.38× (1.34–1.41) | 2.28× (2.22–2.35) | 2.18× (2.11–2.25) | 2.98× (2.92–3.04) | 2.78× (2.70–2.86) | 7.03× (6.89–7.19) | 7.20× (7.04–7.37) | 2.68× (2.62–2.75) | 3.16× (3.09–3.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 730 | 5.1 | 0.66× (0.66–0.67) | 3.08× (2.94–3.24) | 4.39× (4.35–4.44) | 3.12× (3.08–3.16) | 6.81× (6.74–6.89) | 2.46× (2.44–2.49) | 2.32× (2.29–2.34) | 0.98× (0.97–0.98) | 1.52× (1.50–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 687 | 5.5 | 0.55× (0.54–0.56) | 2.85× (2.79–2.90) | 4.88× (4.78–4.97) | 2.98× (2.92–3.04) | 6.38× (6.29–6.47) | 2.32× (2.28–2.36) | 2.18× (2.14–2.22) | 0.92× (0.90–0.94) | 1.43× (1.40–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 699 | 5.2 | 0.57× (0.56–0.58) | 2.88× (2.82–2.94) | 4.92× (4.82–5.02) | 2.97× (2.90–3.03) | 6.41× (6.28–6.54) | 2.34× (2.29–2.38) | 2.20× (2.15–2.25) | 0.93× (0.90–0.95) | 1.44× (1.41–1.47) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 0.090 s | 137 | 24.2 | 1.43× (1.42–1.43) | 3.05× (3.04–3.07) | 3.28× (3.25–3.32) | 3.29× (3.26–3.31) | 3.49× (3.47–3.52) | 3.75× (3.73–3.77) | 3.48× (3.46–3.49) | 1.11× (1.10–1.11) | 1.35× (1.35–1.36) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 0.057 s | 218 | 27.6 | 1.22× (1.17–1.27) | 2.73× (2.44–3.04) | 3.07× (2.90–3.25) | 3.44× (3.22–3.67) | 3.36× (3.23–3.49) | 5.82× (5.63–6.02) | 5.40× (5.22–5.59) | 1.72× (1.66–1.78) | 2.10× (2.03–2.18) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 0.046 s | 268 | 38.7 | 1.27× (1.23–1.30) | 2.00× (1.93–2.07) | 2.70× (2.58–2.83) | 2.97× (2.87–3.07) | 3.19× (3.07–3.32) | 7.14× (6.88–7.41) | 6.63× (6.38–6.88) | 2.11× (2.03–2.19) | 2.58× (2.49–2.68) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.31–2.33) | 5.36× (5.33–5.40) | 5.56× (5.49–5.64) | 6.00× (5.96–6.04) | 5.35× (5.31–5.39) | 5.81× (5.78–5.83) | 5.40× (5.37–5.43) | 1.74× (1.73–1.75) | 1.98× (1.96–2.00) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.28× (2.22–2.35) | 5.10× (5.06–5.13) | 5.50× (5.39–5.62) | 5.97× (5.85–6.09) | 5.05× (5.02–5.08) | 5.43× (5.40–5.45) | 5.05× (5.01–5.08) | 1.62× (1.61–1.63) | 1.85× (1.84–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.27× (2.24–2.30) | 3.86× (3.79–3.93) | 4.14× (4.09–4.19) | 4.36× (4.30–4.43) | 3.87× (3.81–3.92) | 3.41× (3.35–3.46) | 3.17× (3.12–3.22) | 1.02× (1.00–1.04) | 1.16× (1.14–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.27× (3.23–3.31) | 4.11× (4.02–4.21) | 3.36× (3.31–3.40) | 5.74× (5.67–5.80) | 2.67× (2.64–2.71) | 2.52× (2.49–2.55) | 1.01× (0.99–1.02) | 1.08× (1.04–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.07× (3.02–3.11) | 5.33× (5.25–5.41) | 3.30× (3.25–3.35) | 5.51× (5.42–5.59) | 2.60× (2.56–2.63) | 2.45× (2.42–2.48) | 0.98× (0.96–0.99) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.44× (3.37–3.50) | 5.42× (5.32–5.51) | 3.38× (3.30–3.46) | 5.60× (5.50–5.71) | 2.64× (2.61–2.68) | 2.49× (2.46–2.53) | 0.99× (0.97–1.02) | 1.07× (1.03–1.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.36× (2.34–2.38) | 2.62× (2.60–2.63) | 3.27× (3.25–3.29) | 3.42× (3.40–3.44) | 3.73× (3.70–3.75) | 2.27× (2.25–2.28) | 1.92× (1.91–1.94) | 1.85× (1.83–1.88) | 2.53× (2.51–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.10× (2.04–2.15) | 2.36× (2.28–2.45) | 2.87× (2.79–2.95) | 2.95× (2.87–3.03) | 3.17× (3.07–3.27) | 1.85× (1.79–1.90) | 1.57× (1.52–1.61) | 1.51× (1.46–1.56) | 2.06× (2.00–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.90–2.03) | 1.69× (1.65–1.73) | 2.15× (2.10–2.19) | 2.06× (2.00–2.11) | 2.03× (1.99–2.06) | 0.98× (0.96–0.99) | 0.83× (0.82–0.84) | 0.80× (0.78–0.81) | 1.09× (1.07–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.68× (2.66–2.71) | 2.41× (2.37–2.45) | 2.48× (2.44–2.52) | 2.85× (2.80–2.90) | 3.10× (3.06–3.15) | 3.12× (3.07–3.18) | 1.29× (1.26–1.32) | 1.40× (1.38–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.28) | 2.47× (2.44–2.51) | 2.18× (2.15–2.21) | 2.93× (2.88–2.97) | 2.57× (2.53–2.60) | 2.73× (2.70–2.77) | 2.75× (2.71–2.79) | 1.14× (1.10–1.17) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.39–1.50) | 2.31× (2.25–2.38) | 1.89× (1.85–1.93) | 2.60× (2.54–2.67) | 2.46× (2.39–2.52) | 2.02× (1.98–2.07) | 2.04× (1.98–2.09) | 0.84× (0.81–0.88) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.46) | 3.03× (2.99–3.08) | 2.53× (2.50–2.55) | 2.65× (2.61–2.68) | 3.06× (3.02–3.10) | 3.75× (3.71–3.78) | 3.83× (3.79–3.88) | 1.44× (1.42–1.45) | 1.67× (1.66–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.34–1.35) | 2.81× (2.77–2.86) | 2.34× (2.32–2.36) | 3.28× (3.26–3.31) | 2.78× (2.76–2.79) | 3.32× (3.29–3.35) | 3.40× (3.37–3.42) | 1.27× (1.26–1.29) | 1.48× (1.47–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.55–1.62) | 2.53× (2.49–2.57) | 1.94× (1.89–1.98) | 2.83× (2.75–2.92) | 2.55× (2.49–2.60) | 2.33× (2.28–2.39) | 2.39× (2.33–2.44) | 0.90× (0.87–0.92) | 1.04× (1.02–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.72× (2.70–2.75) | 3.55× (3.50–3.60) | 2.84× (2.81–2.87) | 6.05× (5.99–6.11) | 2.26× (2.24–2.28) | 2.14× (2.12–2.16) | 0.97× (0.96–0.99) | 0.99× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.60–0.63) | 2.49× (2.44–2.54) | 4.23× (4.13–4.32) | 2.74× (2.69–2.80) | 5.63× (5.55–5.71) | 2.11× (2.07–2.15) | 2.00× (1.96–2.04) | 0.91× (0.89–0.93) | 0.92× (0.88–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.77× (2.72–2.81) | 4.32× (4.24–4.40) | 2.82× (2.77–2.88) | 5.80× (5.69–5.90) | 2.18× (2.14–2.22) | 2.06× (2.03–2.10) | 0.94× (0.92–0.96) | 0.95× (0.93–0.98) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 1.41× (1.41–1.42) | 2.93× (2.91–2.94) | 3.20× (3.17–3.22) | 3.24× (3.21–3.26) | 3.44× (3.41–3.46) | 3.69× (3.67–3.70) | 3.42× (3.41–3.43) | 1.10× (1.10–1.11) | 1.29× (1.28–1.30) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 1.32× (1.31–1.33) | 2.67× (2.64–2.70) | 2.96× (2.89–3.04) | 3.21× (3.19–3.24) | 3.05× (3.03–3.08) | 3.22× (3.19–3.24) | 2.99× (2.96–3.01) | 0.96× (0.96–0.97) | 1.13× (1.12–1.13) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 1.50× (1.44–1.56) | 2.32× (2.28–2.35) | 2.56× (2.52–2.61) | 2.72× (2.68–2.76) | 2.67× (2.64–2.69) | 2.29× (2.27–2.32) | 2.13× (2.10–2.15) | 0.69× (0.68–0.69) | 0.80× (0.79–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.34× (0.31–0.37) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.18× (12.16–12.20) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.38–0.38) | 0.54× (0.46–0.64) | 0.82× (0.81–0.82) | 0.68× (0.67–0.68) | 6.41× (6.37–6.44) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.79× (0.70–0.90) | 0.70× (0.69–0.71) | 0.62× (0.61–0.63) | 4.08× (4.06–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.83–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.16× (8.14–8.19) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.76× (0.71–0.81) | 1.65× (1.63–1.68) | 0.81× (0.79–0.82) | 5.96× (5.87–6.06) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.67) | 0.71× (0.67–0.75) | 1.16× (1.13–1.19) | 0.75× (0.73–0.77) | 3.40× (3.32–3.48) | 0.26× (0.25–0.26) | 0.28× (0.27–0.28) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.26× (5.25–5.28) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.44) | 0.91× (0.88–0.94) | 0.68× (0.66–0.71) | 3.15× (3.04–3.26) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.81× (0.78–0.84) | 2.55× (2.50–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.83× (0.83–0.84) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.42) | 0.87× (0.84–0.90) | 0.73× (0.69–0.77) | 2.98× (2.88–3.08) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.51× (0.49–0.54) | 0.82× (0.80–0.83) | 0.82× (0.80–0.85) | 2.35× (2.32–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.41× (5.39–5.42) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.22) | 1.64× (1.64–1.64) | 0.77× (0.77–0.77) | 5.41× (5.40–5.43) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.41–0.41) | 1.50× (1.50–1.50) | 0.67× (0.67–0.67) | 6.75× (6.74–6.76) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 31 MB | 0.52× (0.51–0.53) | 0.49× (0.45–0.52) | 1.26× (1.23–1.28) | 0.78× (0.76–0.80) | 3.93× (3.87–4.00) | 0.23× (0.23–0.24) | 0.25× (0.24–0.25) | 0.38× (0.38–0.39) | 0.19× (0.19–0.20) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 49 MB | 0.55× (0.54–0.56) | 0.51× (0.47–0.55) | 1.01× (1.00–1.03) | 0.78× (0.73–0.83) | 2.60× (2.55–2.65) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 321 | 3.12 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 559 | 3.33 | 1.07× (1.06–1.08) | 85% | 1.02× (1.01–1.02) | 1.05× (1.02–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 711 | 5.29 | 1.70× (1.67–1.74) | 55% | 1.23× (1.20–1.25) | 1.67× (1.63–1.71) |

**Best:** mibz2 at 4 threads (711 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (294 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.06, lbzip2 1t 0.09, lbzip2 2t 0.30, lbzip2 4t 0.59, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.90 | 0.02 | 0.08 | 0.295 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3242 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 11.9% |
| MTF + RLE2 | 1.58 | 16.7% |
| T-vector build | 2.11 | 22.4% |
| walk (the one each build uses) | 4.08 | 43.2% |
| of which: segment assembly | 0.15 | 1.6% |
| RLE1 | 0.29 | 3.1% |
| CRC | 0.26 | 2.7% |
| **total** | 9.45 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.321 | (in its total) | 1.97× (1.96–1.98) |  |
| walk | 1.206 | 2.650 | (in its total) | 2.20× (2.18–2.21) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.17× (1.11–1.23) |  |
| inverse BWT, total | 1.926 | 4.028 | 3.291 | 2.09× (2.08–2.10) | 1.71× (1.70–1.72) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h34m ± 11m13s |
| 4 threads | 43m16s ± 6m06s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m02s ± 4m41s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 33m02s ± 4m41s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 4.33 ns a byte of output at 4 threads (CPU, summed over them), 2.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,148,587 | 12,367,175 | \`be4cf6e3036d6c66…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
