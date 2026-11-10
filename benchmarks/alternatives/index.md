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

The latest record: [`39ed2ca`](<https://github.com/lmmx/mibz2/commit/39ed2ca27eb7de46303d4e9b776b56174c78d5a5>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42799620105>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.614 s | 341 | 7.2 | 2.47× (2.46–2.49) | 5.70× (5.67–5.74) | 5.85× (5.82–5.88) | 6.34× (6.29–6.39) | 5.71× (5.68–5.74) | 6.22× (6.18–6.27) | 5.72× (5.68–5.77) | 1.81× (1.80–1.83) | 2.11× (2.10–2.13) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.351 s | 597 | 7.7 | 2.53× (2.35–2.71) | 5.37× (5.15–5.60) | 6.04× (5.81–6.29) | 5.96× (5.75–6.19) | 5.43× (5.19–5.68) | 10.99× (10.62–11.37) | 10.11× (9.76–10.48) | 3.20× (3.09–3.32) | 3.73× (3.60–3.87) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.292 s | 718 | 12.8 | 2.40× (2.37–2.43) | 3.70× (3.60–3.79) | 4.59× (4.49–4.70) | 4.20× (4.13–4.27) | 4.17× (4.11–4.23) | 13.03× (12.86–13.20) | 11.98× (11.80–12.17) | 3.80× (3.74–3.86) | 4.42× (4.36–4.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 2.7 | 0.58× (0.55–0.62) | 3.57× (3.28–3.88) | 4.47× (4.13–4.83) | 3.53× (3.30–3.77) | 6.04× (5.67–6.45) | 2.92× (2.73–3.12) | 2.61× (2.44–2.78) | 0.99× (0.94–1.05) | 1.48× (1.38–1.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 2.8 | 0.56× (0.56–0.57) | 2.95× (2.90–3.00) | 5.92× (5.83–6.00) | 3.52× (3.46–3.58) | 5.99× (5.92–6.07) | 2.92× (2.87–2.96) | 2.61× (2.57–2.65) | 0.99× (0.98–1.01) | 1.48× (1.45–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 952 | 2.8 | 0.56× (0.55–0.58) | 2.95× (2.89–3.00) | 5.88× (5.79–5.98) | 3.50× (3.44–3.56) | 6.05× (5.91–6.20) | 2.90× (2.85–2.95) | 2.59× (2.55–2.64) | 0.99× (0.97–1.00) | 1.47× (1.44–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 553 | 4.7 | 2.52× (2.50–2.54) | 2.89× (2.87–2.91) | 3.71× (3.68–3.73) | 3.63× (3.61–3.66) | 3.96× (3.93–4.00) | 2.44× (2.42–2.45) | 2.02× (2.01–2.04) | 1.94× (1.92–1.96) | 2.88× (2.86–2.90) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 843 | 5.8 | 2.22× (2.09–2.36) | 2.74× (2.57–2.92) | 3.98× (3.77–4.20) | 3.29× (3.19–3.40) | 4.62× (4.50–4.75) | 3.70× (3.60–3.81) | 3.08× (2.99–3.17) | 2.95× (2.86–3.04) | 4.37× (4.26–4.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 745 | 11.6 | 1.76× (1.70–1.82) | 1.83× (1.68–1.99) | 2.81× (2.72–2.91) | 2.11× (2.05–2.18) | 3.71× (3.60–3.82) | 3.25× (3.15–3.36) | 2.70× (2.62–2.79) | 2.59× (2.51–2.67) | 3.84× (3.73–3.96) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 62 | 39.8 | 1.39× (1.38–1.40) | 2.78× (2.75–2.80) | 2.51× (2.48–2.53) | 2.52× (2.50–2.54) | 2.94× (2.90–2.99) | 3.21× (3.19–3.24) | 3.21× (3.19–3.23) | 1.25× (1.24–1.26) | 1.42× (1.40–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 100 | 45.5 | 1.30× (1.27–1.34) | 2.56× (2.51–2.61) | 2.44× (2.37–2.51) | 3.08× (3.02–3.13) | 2.98× (2.93–3.02) | 5.17× (5.07–5.27) | 5.17× (5.08–5.25) | 2.01× (1.97–2.04) | 2.28× (2.24–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 62.2 | 1.32× (1.29–1.35) | 2.09× (2.03–2.15) | 2.03× (1.98–2.09) | 2.52× (2.45–2.60) | 2.79× (2.71–2.86) | 5.94× (5.81–6.08) | 5.94× (5.82–6.07) | 2.31× (2.25–2.36) | 2.62× (2.57–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 36.9 | 1.50× (1.49–1.51) | 3.10× (3.07–3.13) | 2.63× (2.58–2.68) | 2.68× (2.65–2.71) | 3.15× (3.06–3.25) | 3.68× (3.63–3.72) | 3.92× (3.88–3.96) | 1.43× (1.42–1.44) | 1.71× (1.69–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 111 | 42.1 | 1.37× (1.33–1.41) | 2.86× (2.76–2.96) | 2.49× (2.42–2.57) | 3.45× (3.36–3.54) | 3.09× (3.01–3.17) | 6.07× (5.93–6.22) | 6.47× (6.35–6.60) | 2.36× (2.32–2.41) | 2.83× (2.78–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 60.6 | 1.40× (1.36–1.43) | 2.26× (2.19–2.33) | 2.16× (2.10–2.22) | 2.96× (2.91–3.01) | 2.75× (2.70–2.80) | 6.81× (6.66–6.96) | 7.25× (7.09–7.42) | 2.65× (2.57–2.73) | 3.17× (3.08–3.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 3.8 | 0.66× (0.65–0.67) | 3.02× (2.99–3.05) | 4.42× (4.38–4.47) | 3.13× (3.10–3.16) | 6.80× (6.74–6.87) | 2.54× (2.52–2.57) | 2.33× (2.31–2.36) | 0.98× (0.97–1.00) | 1.52× (1.51–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 692 | 4.1 | 0.55× (0.54–0.56) | 2.85× (2.79–2.90) | 4.88× (4.79–4.98) | 2.96× (2.91–3.02) | 6.36× (6.24–6.48) | 2.39× (2.35–2.43) | 2.19× (2.15–2.23) | 0.92× (0.90–0.95) | 1.43× (1.40–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 694 | 4.0 | 0.56× (0.55–0.57) | 2.87× (2.83–2.91) | 4.91× (4.85–4.98) | 2.98× (2.94–3.02) | 6.42× (6.32–6.53) | 2.40× (2.37–2.43) | 2.20× (2.16–2.24) | 0.93× (0.91–0.94) | 1.44× (1.42–1.46) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.088 s | 140 | 17.9 | 1.50× (1.47–1.52) | 3.11× (3.05–3.17) | 3.42× (3.36–3.48) | 3.41× (3.35–3.46) | 3.59× (3.53–3.65) | 4.01× (3.93–4.08) | 3.63× (3.56–3.70) | 1.14× (1.12–1.16) | 1.42× (1.39–1.45) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.059 s | 208 | 21.8 | 1.24× (1.13–1.37) | 2.59× (2.46–2.74) | 2.97× (2.86–3.07) | 3.38× (3.23–3.54) | 3.28× (3.12–3.44) | 5.91× (5.64–6.20) | 5.36× (5.11–5.63) | 1.69× (1.61–1.76) | 2.10× (2.01–2.19) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.049 s | 255 | 31.4 | 1.28× (1.24–1.32) | 2.01× (1.93–2.09) | 2.59× (2.51–2.68) | 2.97× (2.86–3.08) | 3.13× (3.03–3.23) | 7.26× (7.04–7.50) | 6.58× (6.35–6.83) | 2.07× (2.01–2.14) | 2.57× (2.46–2.69) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.47× (2.46–2.48) | 5.68× (5.65–5.71) | 5.83× (5.79–5.86) | 6.32× (6.27–6.37) | 5.70× (5.67–5.73) | 6.20× (6.16–6.25) | 5.71× (5.66–5.76) | 1.81× (1.80–1.82) | 2.10× (2.08–2.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.43× (2.33–2.53) | 5.35× (5.29–5.41) | 5.64× (5.61–5.68) | 6.17× (6.12–6.22) | 5.35× (5.24–5.47) | 5.75× (5.68–5.82) | 5.29× (5.22–5.37) | 1.68× (1.66–1.70) | 1.95× (1.93–1.97) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.31× (2.27–2.35) | 3.83× (3.77–3.89) | 4.12× (4.05–4.20) | 4.38× (4.32–4.44) | 3.84× (3.80–3.89) | 3.50× (3.44–3.57) | 3.22× (3.16–3.28) | 1.02× (1.01–1.04) | 1.19× (1.16–1.21) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.57–0.66) | 3.20× (2.98–3.43) | 4.04× (3.74–4.37) | 3.29× (3.06–3.53) | 5.57× (5.21–5.96) | 2.73× (2.55–2.93) | 2.45× (2.29–2.63) | 0.99× (0.93–1.05) | 1.04× (0.98–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.04× (2.99–3.08) | 5.32× (5.26–5.38) | 3.27× (3.22–3.33) | 5.49× (5.42–5.55) | 2.71× (2.67–2.74) | 2.43× (2.39–2.46) | 0.98× (0.97–0.99) | 1.03× (1.01–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.37× (3.33–3.40) | 5.37× (5.28–5.46) | 3.31× (3.25–3.37) | 5.60× (5.46–5.74) | 2.73× (2.69–2.78) | 2.45× (2.41–2.49) | 0.99× (0.97–1.00) | 1.04× (1.01–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.44× (2.42–2.46) | 2.67× (2.65–2.68) | 3.35× (3.33–3.38) | 3.49× (3.47–3.52) | 3.80× (3.77–3.84) | 2.36× (2.34–2.38) | 1.97× (1.95–1.98) | 1.89× (1.87–1.91) | 2.60× (2.58–2.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.08–2.30) | 2.37× (2.32–2.43) | 2.91× (2.85–2.98) | 2.99× (2.93–3.05) | 3.20× (3.13–3.28) | 1.90× (1.86–1.95) | 1.59× (1.55–1.63) | 1.52× (1.49–1.56) | 2.10× (2.06–2.15) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.88× (1.82–1.94) | 1.62× (1.56–1.69) | 2.05× (1.98–2.13) | 1.95× (1.91–2.00) | 1.93× (1.88–1.98) | 0.95× (0.92–0.99) | 0.80× (0.77–0.82) | 0.76× (0.74–0.79) | 1.05× (1.02–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.71× (2.69–2.73) | 2.43× (2.41–2.45) | 2.50× (2.48–2.52) | 2.91× (2.87–2.96) | 3.18× (3.15–3.20) | 3.18× (3.16–3.20) | 1.24× (1.23–1.25) | 1.39× (1.38–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.49× (2.45–2.52) | 2.20× (2.18–2.23) | 2.93× (2.91–2.96) | 2.60× (2.57–2.62) | 2.80× (2.76–2.83) | 2.80× (2.77–2.82) | 1.09× (1.08–1.11) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.45–1.53) | 2.32× (2.25–2.38) | 1.89× (1.85–1.94) | 2.60× (2.55–2.66) | 2.47× (2.41–2.53) | 2.06× (2.02–2.11) | 2.06× (2.02–2.11) | 0.81× (0.79–0.82) | 0.90× (0.88–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.48–1.51) | 3.04× (3.01–3.06) | 2.56× (2.51–2.62) | 2.66× (2.62–2.69) | 3.12× (3.03–3.22) | 3.64× (3.60–3.68) | 3.88× (3.84–3.92) | 1.42× (1.41–1.44) | 1.67× (1.66–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.39) | 2.80× (2.77–2.84) | 2.32× (2.30–2.35) | 3.26× (3.21–3.30) | 2.75× (2.72–2.78) | 3.21× (3.18–3.25) | 3.42× (3.39–3.46) | 1.26× (1.24–1.27) | 1.48× (1.46–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.55–1.60) | 2.51× (2.45–2.57) | 1.92× (1.88–1.96) | 2.77× (2.71–2.84) | 2.49× (2.45–2.53) | 2.23× (2.19–2.28) | 2.38× (2.33–2.43) | 0.87× (0.85–0.90) | 1.03× (1.00–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.72× (2.69–2.75) | 3.57× (3.53–3.60) | 2.86× (2.83–2.89) | 6.06× (5.99–6.12) | 2.34× (2.32–2.37) | 2.16× (2.13–2.19) | 0.99× (0.98–1.00) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.49× (2.45–2.53) | 4.22× (4.17–4.28) | 2.74× (2.68–2.79) | 5.61× (5.52–5.70) | 2.17× (2.13–2.20) | 2.00× (1.97–2.04) | 0.91× (0.89–0.93) | 0.92× (0.88–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.61–0.62) | 2.77× (2.72–2.81) | 4.28× (4.21–4.34) | 2.81× (2.78–2.85) | 5.80× (5.71–5.89) | 2.23× (2.20–2.26) | 2.06× (2.02–2.09) | 0.94× (0.93–0.95) | 0.95× (0.91–0.99) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.48× (1.46–1.51) | 3.04× (2.98–3.09) | 3.29× (3.24–3.34) | 3.35× (3.30–3.41) | 3.53× (3.48–3.59) | 3.94× (3.87–4.01) | 3.57× (3.51–3.64) | 1.14× (1.12–1.16) | 1.36× (1.35–1.38) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.29× (1.25–1.33) | 2.59× (2.49–2.70) | 2.84× (2.72–2.95) | 3.14× (3.01–3.26) | 2.94× (2.82–3.06) | 3.20× (3.07–3.34) | 2.91× (2.78–3.03) | 0.93× (0.89–0.96) | 1.11× (1.07–1.15) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.42× (1.34–1.51) | 2.20× (2.12–2.27) | 2.45× (2.39–2.51) | 2.63× (2.55–2.72) | 2.55× (2.48–2.62) | 2.24× (2.17–2.32) | 2.04× (1.97–2.11) | 0.65× (0.63–0.67) | 0.78× (0.75–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.31–0.33) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.09× (12.07–12.12) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.47× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.58× (0.52–0.64) | 0.80× (0.79–0.81) | 0.66× (0.66–0.67) | 6.37× (6.31–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.76× (0.65–0.90) | 0.71× (0.70–0.72) | 0.62× (0.61–0.63) | 4.10× (4.07–4.12) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.79× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.72) | 0.67× (0.66–0.67) | 8.07× (8.04–8.10) | 0.66× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.70× (0.66–0.74) | 1.62× (1.58–1.65) | 0.80× (0.78–0.82) | 5.84× (5.70–5.98) | 0.47× (0.46–0.49) | 0.49× (0.48–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.68× (0.64–0.71) | 1.15× (1.11–1.18) | 0.70× (0.67–0.72) | 3.38× (3.27–3.50) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.22× (5.21–5.22) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.50–0.52) | 0.39× (0.35–0.43) | 0.93× (0.91–0.95) | 0.74× (0.72–0.76) | 3.21× (3.14–3.28) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.85–0.88) | 0.80× (0.77–0.83) | 2.51× (2.45–2.56) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.21–5.24) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.49) | 0.40× (0.39–0.41) | 0.86× (0.84–0.88) | 0.69× (0.66–0.72) | 2.95× (2.87–3.03) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.60–0.63) | 0.50× (0.48–0.53) | 0.82× (0.80–0.83) | 0.82× (0.80–0.84) | 2.34× (2.32–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.08× (1.08–1.09) | 0.77× (0.77–0.78) | 5.40× (5.39–5.41) | 0.35× (0.35–0.36) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.63× (1.62–1.63) | 0.77× (0.77–0.78) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.41) | 1.48× (1.47–1.48) | 0.69× (0.69–0.69) | 6.65× (6.63–6.67) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.65) | 0.32× (0.32–0.32) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.41× (0.38–0.44) | 1.06× (1.02–1.10) | 0.68× (0.66–0.71) | 3.34× (3.23–3.45) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.48× (0.45–0.52) | 0.94× (0.91–0.97) | 0.73× (0.69–0.77) | 2.38× (2.30–2.46) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 341 | 2.94 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 597 | 3.16 | 1.08× (1.07–1.09) | 88% | 1.02× (1.01–1.02) | 1.06× (1.01–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 718 | 5.22 | 1.77× (1.75–1.80) | 52% | 1.19× (1.18–1.20) | 1.66× (1.63–1.68) |

**Best:** mibz2 at 4 threads (718 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.09, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.88 | 0.02 | 0.09 | 0.292 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.84 | 12.6% |
| MTF + RLE2 | 1.20 | 17.8% |
| T-vector build | 1.14 | 17.0% |
| walk (the one each build uses) | 3.13 | 46.5% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.22 | 3.3% |
| CRC | 0.19 | 2.9% |
| **total** | 6.72 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.302 | (in its total) | 2.66× (2.63–2.69) |  |
| walk | 1.238 | 2.632 | (in its total) | 2.14× (2.10–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.053 | (in its total) | 1.15× (1.12–1.17) |  |
| inverse BWT, total | 1.778 | 3.987 | 3.250 | 2.25× (2.22–2.29) | 1.82× (1.81–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h29m ± 10m24s |
| 4 threads | 42m50s ± 6m02s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m05s ± 4m35s |
| the scanner alone (its serial part) | 15m34s ± 30.2s |
| the workers alone, over 4 | 32m05s ± 4m35s |

At 4 threads the workers bound it. Per byte: the scanner 9.0 ns a byte of bz2, the workers 4.21 ns a byte of output at 4 threads (CPU, summed over them), 2.43 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
