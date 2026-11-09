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

The latest record: [`c5b7a3c`](<https://github.com/lmmx/mibz2/commit/c5b7a3cd4c3eb80063b4e3abe854fecfccf12481>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42539755762>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.649 s | 323 | 10.1 | 2.33× (2.31–2.35) | 5.38× (5.35–5.41) | 5.50× (5.46–5.54) | 5.99× (5.96–6.02) | 5.39× (5.33–5.44) | 5.81× (5.77–5.84) | 5.43× (5.40–5.46) | 1.74× (1.73–1.75) | 1.98× (1.96–2.00) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.356 s | 590 | 10.9 | 2.34× (2.26–2.43) | 5.24× (5.07–5.42) | 5.78× (5.56–6.01) | 5.74× (5.51–5.98) | 5.17× (5.00–5.36) | 10.39× (10.05–10.74) | 9.71× (9.37–10.06) | 3.11× (3.01–3.22) | 3.54× (3.41–3.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.298 s | 704 | 17.3 | 2.34× (2.29–2.39) | 3.62× (3.53–3.72) | 4.50× (4.40–4.60) | 4.08× (4.01–4.16) | 4.07× (4.00–4.15) | 12.54× (12.30–12.79) | 11.72× (11.48–11.97) | 3.75× (3.68–3.83) | 4.27× (4.17–4.38) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,000 | 3.5 | 0.61× (0.60–0.62) | 3.57× (3.51–3.62) | 4.51× (4.44–4.58) | 3.62× (3.57–3.67) | 6.24× (6.15–6.32) | 2.88× (2.84–2.92) | 2.70× (2.66–2.74) | 1.01× (0.99–1.03) | 1.54× (1.51–1.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 985 | 3.7 | 0.58× (0.57–0.59) | 3.00× (2.93–3.06) | 5.97× (5.85–6.10) | 3.58× (3.51–3.65) | 6.06× (5.94–6.17) | 2.81× (2.75–2.88) | 2.64× (2.58–2.70) | 0.98× (0.97–1.00) | 1.50× (1.47–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 962 | 3.7 | 0.58× (0.57–0.58) | 2.97× (2.95–3.00) | 5.85× (5.79–5.90) | 3.51× (3.48–3.55) | 6.03× (5.94–6.12) | 2.78× (2.76–2.80) | 2.61× (2.58–2.63) | 0.97× (0.96–0.99) | 1.48× (1.47–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 548 | 6.2 | 2.43× (2.36–2.50) | 2.82× (2.74–2.90) | 3.64× (3.48–3.82) | 3.56× (3.46–3.67) | 3.84× (3.73–3.96) | 2.33× (2.26–2.39) | 1.96× (1.91–2.02) | 1.88× (1.82–1.93) | 2.81× (2.72–2.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 860 | 7.5 | 2.09× (2.01–2.17) | 2.64× (2.52–2.76) | 3.81× (3.60–4.05) | 3.22× (3.08–3.37) | 4.55× (4.34–4.76) | 3.60× (3.44–3.76) | 3.04× (2.90–3.18) | 2.90× (2.77–3.04) | 4.34× (4.14–4.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 770 | 14.9 | 1.83× (1.78–1.89) | 1.95× (1.80–2.10) | 2.85× (2.73–2.98) | 2.19× (2.09–2.30) | 3.81× (3.68–3.95) | 3.27× (3.19–3.35) | 2.76× (2.69–2.83) | 2.64× (2.57–2.70) | 3.95× (3.85–4.05) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 63 | 52.5 | 1.36× (1.36–1.37) | 2.79× (2.79–2.79) | 2.51× (2.47–2.54) | 2.61× (2.57–2.66) | 2.96× (2.89–3.03) | 3.20× (3.19–3.21) | 3.21× (3.19–3.22) | 1.27× (1.26–1.28) | 1.43× (1.43–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 102 | 59.4 | 1.25× (1.19–1.30) | 2.53× (2.41–2.65) | 2.39× (2.29–2.49) | 3.11× (2.91–3.33) | 2.92× (2.80–3.04) | 5.03× (4.80–5.26) | 5.04× (4.83–5.25) | 1.99× (1.90–2.08) | 2.24× (2.15–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 81.9 | 1.29× (1.26–1.32) | 2.12× (2.07–2.17) | 2.09× (2.02–2.16) | 2.59× (2.53–2.64) | 2.83× (2.78–2.88) | 6.00× (5.90–6.10) | 6.01× (5.89–6.14) | 2.37× (2.34–2.41) | 2.68× (2.63–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 67 | 48.8 | 1.45× (1.44–1.46) | 3.09× (3.07–3.11) | 2.61× (2.58–2.65) | 2.72× (2.71–2.74) | 3.19× (3.05–3.34) | 3.83× (3.82–3.84) | 3.92× (3.87–3.97) | 1.46× (1.43–1.48) | 1.72× (1.70–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 110 | 55.7 | 1.31× (1.26–1.36) | 2.77× (2.68–2.86) | 2.48× (2.37–2.60) | 3.44× (3.32–3.55) | 3.05× (2.95–3.17) | 6.15× (5.94–6.37) | 6.29× (6.07–6.52) | 2.34× (2.29–2.39) | 2.76× (2.68–2.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 127 | 79.5 | 1.37× (1.33–1.41) | 2.24× (2.17–2.31) | 2.19× (2.12–2.27) | 2.99× (2.92–3.06) | 2.79× (2.71–2.87) | 7.11× (6.93–7.29) | 7.27× (7.14–7.40) | 2.70× (2.61–2.79) | 3.19× (3.09–3.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 749 | 4.9 | 0.67× (0.66–0.67) | 3.13× (2.96–3.31) | 4.49× (4.44–4.54) | 3.13× (3.10–3.17) | 6.79× (6.72–6.87) | 2.50× (2.47–2.53) | 2.34× (2.32–2.37) | 0.98× (0.97–1.00) | 1.54× (1.53–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 697 | 5.4 | 0.58× (0.54–0.61) | 2.88× (2.83–2.92) | 4.94× (4.87–5.02) | 2.98× (2.93–3.03) | 6.42× (6.28–6.56) | 2.34× (2.30–2.39) | 2.20× (2.16–2.23) | 0.92× (0.91–0.94) | 1.45× (1.42–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 694 | 5.3 | 0.57× (0.56–0.58) | 2.87× (2.84–2.91) | 4.92× (4.86–4.98) | 2.97× (2.94–3.00) | 6.39× (6.29–6.49) | 2.33× (2.29–2.36) | 2.18× (2.15–2.21) | 0.92× (0.90–0.93) | 1.44× (1.42–1.46) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 0.089 s | 141 | 23.5 | 1.45× (1.44–1.45) | 3.10× (3.10–3.11) | 3.31× (3.30–3.32) | 3.34× (3.33–3.35) | 3.51× (3.49–3.53) | 3.79× (3.77–3.80) | 3.54× (3.53–3.55) | 1.12× (1.12–1.13) | 1.38× (1.37–1.38) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 0.057 s | 221 | 28.0 | 1.22× (1.10–1.35) | 2.62× (2.36–2.90) | 3.02× (2.76–3.32) | 3.32× (2.95–3.74) | 3.32× (3.02–3.65) | 5.72× (5.19–6.31) | 5.35× (4.85–5.89) | 1.70× (1.54–1.87) | 2.08× (1.89–2.29) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 0.046 s | 272 | 38.3 | 1.32× (1.28–1.37) | 2.26× (2.17–2.36) | 2.78× (2.64–2.93) | 2.97× (2.89–3.05) | 3.25× (3.17–3.34) | 7.22× (7.03–7.41) | 6.74× (6.56–6.93) | 2.14× (2.09–2.19) | 2.62× (2.55–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.31–2.34) | 5.35× (5.32–5.39) | 5.48× (5.44–5.52) | 5.97× (5.94–6.00) | 5.37× (5.32–5.43) | 5.79× (5.76–5.82) | 5.41× (5.38–5.44) | 1.74× (1.73–1.75) | 1.97× (1.94–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.22–2.26) | 5.12× (5.09–5.15) | 5.32× (5.28–5.35) | 5.85× (5.81–5.89) | 5.06× (5.02–5.09) | 5.38× (5.35–5.41) | 5.03× (4.99–5.07) | 1.61× (1.60–1.62) | 1.83× (1.80–1.85) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.23× (2.19–2.27) | 3.82× (3.77–3.87) | 4.10× (4.04–4.16) | 4.30× (4.21–4.38) | 3.80× (3.74–3.87) | 3.40× (3.35–3.45) | 3.18× (3.13–3.23) | 1.02× (1.00–1.04) | 1.16× (1.14–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.65) | 3.31× (3.26–3.36) | 4.11× (4.06–4.17) | 3.39× (3.34–3.45) | 5.79× (5.71–5.88) | 2.72× (2.68–2.76) | 2.56× (2.52–2.60) | 1.01× (0.99–1.03) | 1.07× (1.05–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.62–0.63) | 3.07× (3.01–3.13) | 5.40× (5.31–5.49) | 3.32× (3.26–3.38) | 5.53× (5.44–5.62) | 2.61× (2.56–2.66) | 2.45× (2.40–2.50) | 0.97× (0.95–0.98) | 1.03× (1.00–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.61–0.62) | 3.41× (3.38–3.44) | 5.41× (5.34–5.48) | 3.32× (3.29–3.35) | 5.59× (5.51–5.67) | 2.63× (2.61–2.65) | 2.47× (2.45–2.49) | 0.97× (0.96–0.99) | 1.04× (1.01–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.35× (2.27–2.43) | 2.60× (2.51–2.69) | 3.30× (3.15–3.46) | 3.42× (3.31–3.54) | 3.69× (3.56–3.82) | 2.25× (2.18–2.33) | 1.91× (1.84–1.98) | 1.83× (1.77–1.89) | 2.50× (2.42–2.59) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.13× (2.11–2.16) | 2.35× (2.30–2.41) | 2.92× (2.88–2.95) | 3.03× (2.99–3.07) | 3.21× (3.17–3.25) | 1.88× (1.86–1.90) | 1.60× (1.58–1.61) | 1.53× (1.51–1.54) | 2.09× (2.07–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.85–2.03) | 1.66× (1.59–1.74) | 2.10× (2.03–2.16) | 1.97× (1.88–2.05) | 1.99× (1.93–2.06) | 0.96× (0.93–0.99) | 0.81× (0.79–0.84) | 0.78× (0.75–0.80) | 1.07× (1.04–1.10) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.35–1.36) | 2.72× (2.71–2.73) | 2.45× (2.43–2.47) | 2.59× (2.54–2.64) | 2.93× (2.87–3.00) | 3.17× (3.16–3.18) | 3.18× (3.16–3.19) | 1.26× (1.25–1.27) | 1.39× (1.39–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.28) | 2.52× (2.47–2.58) | 2.23× (2.16–2.29) | 3.02× (2.91–3.13) | 2.61× (2.57–2.65) | 2.78× (2.74–2.83) | 2.79× (2.75–2.83) | 1.11× (1.09–1.13) | 1.23× (1.20–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.40–1.51) | 2.33× (2.27–2.40) | 1.89× (1.84–1.93) | 2.60× (2.54–2.67) | 2.48× (2.43–2.52) | 2.05× (2.01–2.09) | 2.06× (2.02–2.10) | 0.82× (0.80–0.84) | 0.90× (0.88–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.44–1.45) | 3.04× (3.04–3.05) | 2.55× (2.53–2.57) | 2.70× (2.69–2.71) | 3.16× (3.02–3.31) | 3.79× (3.79–3.80) | 3.88× (3.83–3.93) | 1.45× (1.43–1.47) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.32–1.35) | 2.81× (2.77–2.85) | 2.33× (2.32–2.35) | 3.29× (3.25–3.32) | 2.78× (2.75–2.82) | 3.33× (3.30–3.35) | 3.40× (3.36–3.45) | 1.27× (1.25–1.30) | 1.46× (1.44–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.53–1.61) | 2.54× (2.48–2.61) | 1.95× (1.90–1.99) | 2.82× (2.74–2.90) | 2.55× (2.48–2.62) | 2.34× (2.28–2.41) | 2.40× (2.32–2.48) | 0.90× (0.87–0.92) | 1.03× (1.00–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.76× (2.72–2.81) | 3.62× (3.50–3.74) | 2.87× (2.83–2.90) | 6.07× (5.99–6.15) | 2.32× (2.29–2.34) | 2.17× (2.14–2.20) | 0.99× (0.97–1.01) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.64× (0.61–0.67) | 2.51× (2.47–2.54) | 4.23× (4.17–4.30) | 2.75× (2.70–2.80) | 5.66× (5.55–5.78) | 2.14× (2.10–2.18) | 2.01× (1.97–2.04) | 0.91× (0.89–0.93) | 0.92× (0.89–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.77× (2.73–2.80) | 4.27× (4.23–4.31) | 2.82× (2.80–2.84) | 5.80× (5.72–5.88) | 2.18× (2.15–2.21) | 2.05× (2.02–2.07) | 0.93× (0.91–0.94) | 0.94× (0.91–0.98) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 1.44× (1.43–1.44) | 2.99× (2.97–3.00) | 3.23× (3.22–3.24) | 3.29× (3.28–3.30) | 3.45× (3.44–3.47) | 3.72× (3.71–3.73) | 3.48× (3.47–3.49) | 1.12× (1.12–1.12) | 1.30× (1.29–1.30) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 1.29× (1.28–1.31) | 2.68× (2.61–2.75) | 2.87× (2.84–2.91) | 3.16× (3.12–3.19) | 2.99× (2.95–3.03) | 3.14× (3.11–3.17) | 2.94× (2.91–2.97) | 0.94× (0.94–0.95) | 1.10× (1.08–1.11) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 1.54× (1.49–1.59) | 2.27× (2.21–2.34) | 2.57× (2.50–2.64) | 2.74× (2.68–2.80) | 2.63× (2.59–2.67) | 2.28× (2.24–2.32) | 2.13× (2.10–2.16) | 0.69× (0.68–0.69) | 0.80× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.33× (0.32–0.35) | 1.08× (1.07–1.08) | 0.86× (0.86–0.86) | 12.17× (12.15–12.19) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.62× (0.53–0.73) | 0.81× (0.80–0.82) | 0.67× (0.66–0.68) | 6.40× (6.36–6.45) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.38–0.39) | 0.72× (0.66–0.80) | 0.70× (0.70–0.71) | 0.61× (0.59–0.62) | 4.09× (4.07–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.43× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.73–1.74) | 0.67× (0.66–0.67) | 8.16× (8.14–8.19) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.68× (0.63–0.73) | 1.65× (1.63–1.68) | 0.81× (0.80–0.82) | 5.97× (5.87–6.06) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.62–0.67) | 0.74× (0.70–0.79) | 1.19× (1.15–1.24) | 0.77× (0.74–0.79) | 3.46× (3.35–3.59) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.73–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.49–0.54) | 0.41× (0.38–0.45) | 0.94× (0.90–0.98) | 0.73× (0.69–0.77) | 3.25× (3.12–3.38) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.62–0.65) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.81–0.85) | 2.56× (2.51–2.62) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.83× (0.83–0.84) | 5.26× (5.26–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.41) | 0.85× (0.82–0.88) | 0.69× (0.66–0.73) | 2.91× (2.81–3.02) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.27–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.49× (0.46–0.53) | 0.81× (0.79–0.83) | 0.81× (0.77–0.84) | 2.32× (2.27–2.36) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.40× (5.38–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.40–5.43) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.41–0.41) | 1.50× (1.50–1.51) | 0.67× (0.67–0.67) | 6.76× (6.75–6.78) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 33 MB | 0.47× (0.46–0.49) | 0.45× (0.41–0.49) | 1.16× (1.11–1.21) | 0.73× (0.70–0.76) | 3.60× (3.50–3.71) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 50 MB | 0.54× (0.53–0.55) | 0.48× (0.45–0.51) | 1.01× (0.98–1.04) | 0.80× (0.77–0.82) | 2.57× (2.51–2.62) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 323 | 3.11 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 590 | 3.35 | 1.08× (1.07–1.09) | 89% | 1.03× (1.03–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 704 | 5.32 | 1.70× (1.67–1.73) | 54% | 1.21× (1.21–1.22) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (704 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.09, lbzip2 1t 0.09, lbzip2 2t 0.40, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.91 | 0.02 | 0.08 | 0.298 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 11.7% |
| MTF + RLE2 | 1.58 | 16.6% |
| T-vector build | 2.11 | 22.1% |
| walk (the one each build uses) | 4.18 | 43.8% |
| of which: segment assembly | 0.16 | 1.7% |
| RLE1 | 0.29 | 3.0% |
| CRC | 0.26 | 2.7% |
| **total** | 9.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.673 | 1.303 | (in its total) | 1.94× (1.93–1.94) |  |
| walk | 1.250 | 2.639 | (in its total) | 2.10× (2.10–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.055 | (in its total) | 1.16× (1.16–1.17) |  |
| inverse BWT, total | 1.970 | 3.997 | 3.249 | 2.02× (2.02–2.03) | 1.65× (1.64–1.66) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h34m ± 11m05s |
| 4 threads | 43m48s ± 6m12s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m42s ± 4m38s |
| the scanner alone (its serial part) | 15m34s ± 30.2s |
| the workers alone, over 4 | 32m42s ± 4m38s |

At 4 threads the workers bound it. Per byte: the scanner 9.0 ns a byte of bz2, the workers 4.29 ns a byte of output at 4 threads (CPU, summed over them), 2.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,094,880 | 12,489,382 | \`0cafc9c0ea86fcad…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
