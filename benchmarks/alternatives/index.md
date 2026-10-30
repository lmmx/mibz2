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

The latest record: [`ae942bd`](<https://github.com/lmmx/mibz2/commit/ae942bdb135569224473116ffaa706f1c9e389ed>) (2026-10-30).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40988680203>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.754 s | 278 | 11.7 | 1.99× (1.97–2.01) | 4.57× (4.53–4.62) | 4.77× (4.72–4.81) | 5.13× (5.09–5.17) | 4.67× (4.63–4.72) | 4.98× (4.95–5.01) | 4.63× (4.59–4.67) | 1.48× (1.47–1.49) | 1.72× (1.71–1.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.495 s | 424 | 12.8 | 1.72× (1.65–1.80) | 3.66× (3.52–3.80) | 4.23× (4.08–4.39) | 4.19× (4.03–4.36) | 3.81× (3.67–3.95) | 7.61× (7.34–7.90) | 7.08× (6.82–7.36) | 2.26× (2.18–2.35) | 2.63× (2.54–2.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.394 s | 533 | 20.3 | 1.75× (1.73–1.78) | 2.71× (2.68–2.74) | 3.39× (3.33–3.46) | 3.05× (3.02–3.07) | 3.16× (3.13–3.19) | 9.55× (9.48–9.63) | 8.89× (8.81–8.97) | 2.84× (2.81–2.87) | 3.30× (3.26–3.34) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 952 | 3.7 | 0.57× (0.54–0.60) | 3.81× (3.64–4.00) | 4.71× (4.50–4.94) | 3.39× (3.24–3.55) | 6.44× (6.12–6.78) | 2.70× (2.58–2.83) | 2.53× (2.41–2.64) | 1.00× (0.95–1.06) | 1.44× (1.37–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 920 | 3.9 | 0.55× (0.54–0.57) | 2.84× (2.78–2.90) | 6.10× (5.98–6.24) | 3.40× (3.33–3.48) | 6.41× (6.30–6.51) | 2.68× (2.62–2.73) | 2.50× (2.45–2.56) | 0.99× (0.97–1.02) | 1.43× (1.40–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 905 | 3.9 | 0.55× (0.54–0.57) | 2.79× (2.75–2.83) | 5.96× (5.88–6.05) | 3.33× (3.29–3.37) | 6.30× (6.23–6.36) | 2.62× (2.58–2.65) | 2.45× (2.41–2.48) | 0.97× (0.95–0.99) | 1.39× (1.37–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.044 s | 448 | 7.5 | 2.16× (2.09–2.22) | 2.35× (2.31–2.38) | 2.79× (2.75–2.83) | 2.96× (2.91–3.01) | 3.49× (3.44–3.54) | 1.92× (1.90–1.95) | 1.63× (1.60–1.65) | 1.59× (1.57–1.61) | 2.34× (2.30–2.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 s | 710 | 9.1 | 1.81× (1.60–2.05) | 2.25× (2.05–2.45) | 3.29× (2.98–3.63) | 2.57× (2.24–2.94) | 4.14× (3.61–4.74) | 2.88× (2.52–3.29) | 2.43× (2.13–2.78) | 2.38× (2.08–2.72) | 3.50× (3.06–4.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.030 s | 644 | 17.4 | 1.53× (1.48–1.58) | 1.72× (1.66–1.78) | 2.48× (2.35–2.62) | 1.91× (1.80–2.04) | 3.66× (3.56–3.76) | 2.74× (2.66–2.83) | 2.32× (2.25–2.39) | 2.26× (2.20–2.33) | 3.33× (3.24–3.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 63.0 | 1.15× (1.14–1.16) | 2.34× (2.33–2.35) | 2.11× (2.10–2.13) | 2.14× (2.12–2.16) | 2.53× (2.51–2.54) | 2.66× (2.64–2.67) | 2.66× (2.64–2.68) | 1.05× (1.05–1.06) | 1.20× (1.18–1.21) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.095 s | 84 | 69.5 | 1.08× (1.04–1.12) | 2.24× (2.07–2.43) | 2.03× (1.95–2.11) | 2.60× (2.49–2.72) | 2.62× (2.52–2.72) | 4.27× (4.10–4.44) | 4.28× (4.11–4.45) | 1.69× (1.63–1.76) | 1.93× (1.84–2.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 93.0 | 1.16× (1.14–1.19) | 1.94× (1.84–2.04) | 1.92× (1.81–2.03) | 2.32× (2.25–2.39) | 2.70× (2.64–2.76) | 5.39× (5.27–5.51) | 5.40× (5.28–5.52) | 2.14× (2.09–2.19) | 2.43× (2.40–2.47) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.202 s | 48 | 68.8 | 1.04× (1.03–1.05) | 2.28× (2.26–2.30) | 1.86× (1.84–1.87) | 1.90× (1.89–1.92) | 2.27× (2.26–2.28) | 2.71× (2.68–2.73) | 2.75× (2.72–2.79) | 1.01× (1.00–1.01) | 1.16× (1.15–1.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.120 s | 80 | 75.3 | 0.95× (0.90–0.99) | 2.08× (2.01–2.16) | 1.79× (1.73–1.86) | 2.57× (2.45–2.69) | 2.32× (2.22–2.42) | 4.47× (4.28–4.68) | 4.55× (4.34–4.77) | 1.66× (1.59–1.74) | 1.91× (1.83–2.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.093 s | 103 | 101.9 | 1.14× (1.11–1.17) | 1.93× (1.88–1.97) | 1.82× (1.79–1.85) | 2.50× (2.44–2.56) | 2.44× (2.41–2.48) | 5.87× (5.80–5.95) | 5.97× (5.88–6.06) | 2.18× (2.16–2.20) | 2.51× (2.48–2.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 5.1 | 0.64× (0.63–0.65) | 3.32× (3.06–3.61) | 4.30× (4.23–4.37) | 2.99× (2.94–3.03) | 7.38× (7.27–7.49) | 2.37× (2.34–2.41) | 2.22× (2.19–2.26) | 0.98× (0.96–1.00) | 1.47× (1.44–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 660 | 5.7 | 0.54× (0.53–0.55) | 2.72× (2.67–2.77) | 4.72× (4.63–4.81) | 2.84× (2.78–2.90) | 6.94× (6.80–7.08) | 2.22× (2.18–2.26) | 2.08× (2.04–2.12) | 0.92× (0.90–0.93) | 1.37× (1.35–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 654 | 5.6 | 0.54× (0.54–0.55) | 2.71× (2.67–2.74) | 4.67× (4.61–4.73) | 2.83× (2.79–2.87) | 6.84× (6.74–6.94) | 2.20× (2.17–2.23) | 2.06× (2.03–2.08) | 0.91× (0.89–0.92) | 1.36× (1.34–1.38) | yes |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 1 | 0.088 s | 139 | 23.9 | 1.39× (1.38–1.40) | 2.91× (2.88–2.93) | 3.21× (3.17–3.24) | 3.07× (3.04–3.09) | 3.44× (3.40–3.48) | 3.33× (3.30–3.36) | 3.16× (3.13–3.19) | 1.05× (1.04–1.06) | 1.27× (1.26–1.29) | yes |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 2 | 0.055 s | 221 | 28.0 | 1.22× (1.17–1.27) | 2.61× (2.27–2.99) | 3.01× (2.88–3.15) | 3.32× (2.97–3.71) | 3.50× (3.39–3.61) | 5.27× (5.12–5.43) | 5.00× (4.85–5.15) | 1.66× (1.61–1.71) | 2.02× (1.96–2.08) | yes |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 4 | 0.047 s | 261 | 41.2 | 1.21× (1.16–1.25) | 1.92× (1.81–2.04) | 2.46× (2.35–2.57) | 2.69× (2.62–2.76) | 3.23× (3.13–3.33) | 6.17× (6.00–6.35) | 5.85× (5.68–6.04) | 1.95× (1.89–2.00) | 2.36× (2.30–2.43) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.99× (1.97–2.00) | 4.56× (4.52–4.60) | 4.75× (4.71–4.79) | 5.12× (5.08–5.16) | 4.67× (4.63–4.71) | 4.97× (4.94–5.00) | 4.62× (4.58–4.66) | 1.48× (1.46–1.49) | 1.72× (1.70–1.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.88× (1.87–1.90) | 4.26× (4.23–4.29) | 4.53× (4.50–4.56) | 4.95× (4.92–4.99) | 4.29× (4.27–4.32) | 4.58× (4.55–4.60) | 4.26× (4.24–4.28) | 1.36× (1.35–1.37) | 1.58× (1.57–1.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.87× (1.85–1.90) | 3.20× (3.17–3.24) | 3.40× (3.36–3.44) | 3.60× (3.55–3.64) | 3.23× (3.19–3.27) | 2.88× (2.85–2.91) | 2.68× (2.65–2.71) | 0.86× (0.85–0.86) | 0.99× (0.98–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.58–0.64) | 3.18× (3.03–3.34) | 4.07× (3.88–4.28) | 3.21× (3.06–3.37) | 6.02× (5.71–6.35) | 2.57× (2.45–2.70) | 2.41× (2.29–2.52) | 1.00× (0.95–1.06) | 1.06× (1.00–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.58–0.61) | 2.98× (2.92–3.04) | 5.29× (5.20–5.39) | 3.19× (3.12–3.25) | 5.90× (5.81–5.99) | 2.50× (2.45–2.56) | 2.34× (2.29–2.39) | 0.98× (0.95–1.00) | 1.04× (0.99–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.57–0.60) | 3.28× (3.21–3.35) | 5.27× (5.19–5.35) | 3.18× (3.15–3.21) | 5.87× (5.81–5.94) | 2.49× (2.46–2.52) | 2.33× (2.30–2.36) | 0.97× (0.95–0.99) | 1.03× (0.99–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.11× (2.05–2.18) | 2.25× (2.22–2.29) | 2.68× (2.64–2.72) | 2.89× (2.84–2.93) | 3.39× (3.34–3.44) | 1.89× (1.86–1.92) | 1.60× (1.58–1.63) | 1.57× (1.55–1.59) | 2.14× (2.11–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.83× (1.78–1.88) | 2.05× (2.02–2.07) | 2.51× (2.48–2.55) | 2.53× (2.50–2.56) | 2.93× (2.90–2.97) | 1.57× (1.55–1.59) | 1.33× (1.31–1.35) | 1.30× (1.29–1.32) | 1.77× (1.75–1.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.68× (1.61–1.75) | 1.57× (1.50–1.64) | 1.84× (1.76–1.91) | 1.74× (1.66–1.83) | 1.88× (1.82–1.95) | 0.83× (0.80–0.86) | 0.70× (0.68–0.73) | 0.69× (0.66–0.71) | 0.94× (0.91–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.15× (1.14–1.16) | 2.31× (2.30–2.32) | 2.06× (2.05–2.07) | 2.13× (2.11–2.15) | 2.51× (2.50–2.53) | 2.64× (2.62–2.65) | 2.64× (2.62–2.66) | 1.05× (1.05–1.06) | 1.14× (1.12–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.09× (1.09–1.09) | 2.19× (2.17–2.22) | 1.94× (1.93–1.95) | 2.56× (2.54–2.57) | 2.33× (2.32–2.34) | 2.39× (2.38–2.40) | 2.39× (2.38–2.40) | 0.95× (0.95–0.96) | 1.04× (1.02–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.26× (1.21–1.31) | 1.98× (1.94–2.03) | 1.66× (1.63–1.68) | 2.27× (2.25–2.29) | 2.20× (2.19–2.21) | 1.79× (1.78–1.80) | 1.79× (1.78–1.80) | 0.71× (0.71–0.72) | 0.78× (0.76–0.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.04× (1.03–1.05) | 2.25× (2.24–2.27) | 1.83× (1.82–1.84) | 1.89× (1.88–1.91) | 2.26× (2.25–2.27) | 2.69× (2.67–2.71) | 2.74× (2.70–2.77) | 1.01× (1.00–1.01) | 1.13× (1.12–1.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.00× (0.99–1.01) | 2.15× (2.14–2.16) | 1.74× (1.74–1.75) | 2.51× (2.48–2.54) | 2.11× (2.11–2.12) | 2.46× (2.45–2.48) | 2.50× (2.48–2.54) | 0.92× (0.92–0.92) | 1.03× (1.03–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.23× (1.21–1.25) | 2.00× (1.95–2.05) | 1.52× (1.50–1.54) | 2.24× (2.21–2.27) | 2.02× (2.00–2.04) | 1.82× (1.81–1.84) | 1.85× (1.83–1.88) | 0.68× (0.68–0.69) | 0.77× (0.76–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.70× (2.65–2.75) | 3.60× (3.54–3.66) | 2.77× (2.72–2.82) | 6.68× (6.57–6.80) | 2.23× (2.20–2.27) | 2.09× (2.05–2.13) | 0.98× (0.96–1.00) | 0.98× (0.93–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.41× (2.38–2.44) | 4.17× (4.12–4.22) | 2.64× (2.59–2.69) | 6.16× (6.05–6.27) | 2.04× (2.01–2.07) | 1.91× (1.88–1.94) | 0.90× (0.88–0.91) | 0.90× (0.85–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.68× (2.64–2.73) | 4.20× (4.13–4.26) | 2.71× (2.67–2.76) | 6.23× (6.13–6.33) | 2.07× (2.04–2.10) | 1.94× (1.91–1.97) | 0.91× (0.89–0.93) | 0.91× (0.87–0.95) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 1 | 1.38× (1.37–1.40) | 2.78× (2.76–2.80) | 3.08× (3.06–3.11) | 3.03× (3.00–3.06) | 3.40× (3.35–3.44) | 3.29× (3.26–3.32) | 3.12× (3.09–3.15) | 1.05× (1.04–1.06) | 1.25× (1.23–1.26) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 2 | 1.25× (1.23–1.27) | 2.46× (2.42–2.50) | 2.77× (2.74–2.81) | 2.87× (2.85–2.90) | 2.97× (2.93–3.00) | 2.80× (2.78–2.83) | 2.66× (2.62–2.70) | 0.90× (0.89–0.91) | 1.06× (1.05–1.08) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 4 | 1.39× (1.35–1.42) | 2.02× (1.97–2.07) | 2.28× (2.24–2.32) | 2.35× (2.29–2.40) | 2.42× (2.38–2.46) | 1.90× (1.87–1.93) | 1.80× (1.77–1.83) | 0.61× (0.60–0.62) | 0.72× (0.71–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.30–0.31) | 1.17× (1.17–1.17) | 0.84× (0.84–0.84) | 11.87× (11.85–11.89) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.51× (0.45–0.58) | 0.85× (0.84–0.87) | 0.67× (0.66–0.67) | 6.30× (6.22–6.39) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.76× (0.68–0.84) | 0.79× (0.78–0.80) | 0.64× (0.63–0.65) | 4.28× (4.26–4.30) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.77× (1.77–1.78) | 0.83× (0.83–0.83) | 4.40× (4.39–4.41) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.92× (0.92–0.93) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.77–1.78) | 0.83× (0.83–0.84) | 4.41× (4.40–4.41) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.48× (0.48–0.49) | 0.42× (0.42–0.43) | 1.64× (1.64–1.64) | 0.63× (0.63–0.64) | 7.79× (7.77–7.80) | 0.62× (0.61–0.62) | 0.65× (0.65–0.65) | 0.70× (0.70–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.64× (0.62–0.66) | 0.66× (0.62–0.69) | 1.55× (1.53–1.58) | 0.88× (0.87–0.90) | 5.63× (5.55–5.71) | 0.44× (0.44–0.45) | 0.47× (0.46–0.48) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.59–0.61) | 0.66× (0.63–0.70) | 1.11× (1.09–1.13) | 0.86× (0.84–0.87) | 3.23× (3.17–3.28) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.28) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.16) | 0.72× (0.72–0.72) | 5.10× (5.09–5.11) | 0.31× (0.31–0.31) | 0.33× (0.33–0.34) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.41× (0.39–0.45) | 0.93× (0.92–0.94) | 0.74× (0.73–0.76) | 3.24× (3.21–3.27) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 52 MB | 0.59× (0.57–0.61) | 0.43× (0.42–0.44) | 0.82× (0.81–0.83) | 0.76× (0.74–0.79) | 2.40× (2.36–2.43) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.15–1.16) | 0.81× (0.80–0.81) | 5.10× (5.09–5.11) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.50) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.41) | 0.87× (0.87–0.88) | 0.72× (0.69–0.75) | 3.01× (2.99–3.04) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.54× (0.53–0.55) | 0.47× (0.44–0.50) | 0.74× (0.72–0.75) | 0.74× (0.71–0.78) | 2.14× (2.11–2.18) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.20–0.20) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.20–0.22) | 1.09× (1.08–1.09) | 0.77× (0.76–0.77) | 5.43× (5.42–5.43) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.19× (0.19–0.20) | 1.62× (1.61–1.62) | 0.77× (0.76–0.77) | 5.37× (5.36–5.39) | 0.34× (0.34–0.34) | 0.36× (0.36–0.36) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.19–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.40× (5.39–5.42) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.12–0.13) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.39) | 1.45× (1.45–1.46) | 0.63× (0.63–0.64) | 6.54× (6.52–6.56) | 0.39× (0.38–0.39) | 0.41× (0.41–0.41) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 2 | 33 MB | 0.48× (0.46–0.50) | 0.42× (0.38–0.47) | 1.19× (1.14–1.24) | 0.73× (0.70–0.75) | 3.69× (3.57–3.82) | 0.22× (0.21–0.23) | 0.23× (0.22–0.24) | 0.36× (0.35–0.37) | 0.17× (0.17–0.18) |
| `text` (1.63 MB → 12.2 MB, 7.5×) | 4 | 54 MB | 0.49× (0.48–0.51) | 0.49× (0.46–0.53) | 0.96× (0.93–0.98) | 0.74× (0.72–0.76) | 2.40× (2.36–2.44) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 278 | 3.60 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 424 | 3.93 | 1.09× (1.08–1.10) | 76% | 1.01× (1.01–1.02) | 1.03× (1.03–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 533 | 6.25 | 1.73× (1.71–1.75) | 48% | 1.21× (1.21–1.22) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (533 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (303 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.72, mibz2 4t 0.94, lbzip2 1t 0.09, lbzip2 2t 0.30, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.35, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 5%, lbzip2 2t 4%, lbzip2 4t 6%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.10 | 0.02 | 0.07 | 0.394 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3246 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.22 | 11.0% |
| MTF + RLE2 | 2.27 | 20.5% |
| T-vector build | 2.11 | 19.1% |
| walk (the one each build uses) | 4.93 | 44.6% |
| of which: segment assembly | 0.18 | 1.6% |
| RLE1 | 0.28 | 2.5% |
| CRC | 0.26 | 2.3% |
| **total** | 11.07 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.676 | 1.347 | (in its total) | 1.99× (1.99–2.00) |  |
| walk | 1.525 | 2.624 | (in its total) | 1.72× (1.71–1.73) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.053 | 0.052 | (in its total) | 0.97× (0.96–0.98) |  |
| inverse BWT, total | 2.253 | 4.023 | 3.339 | 1.78× (1.77–1.79) | 1.48× (1.47–1.49) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h50m ± 13m11s |
| 4 threads | 57m26s ± 8m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 40m05s ± 5m38s |
| the scanner alone (its serial part) | 16m18s ± 26.3s |
| the workers alone, over 4 | 40m05s ± 5m38s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 5.25 ns a byte of output at 4 threads (CPU, summed over them), 3.08 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,626,321 | 12,192,200 | \`853b8fbf38a1ba8b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3246.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
