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

The latest record: [`d54632f`](<https://github.com/lmmx/mibz2/commit/d54632f6954b5b5f654967eb0d7c43929108cecd>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42687219959>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.659 s | 318 | 9.5 | 2.33× (2.28–2.38) | 5.28× (5.12–5.45) | 5.61× (5.33–5.90) | 5.85× (5.65–6.06) | 5.26× (5.09–5.44) | 5.64× (5.43–5.86) | 5.25× (5.08–5.44) | 1.70× (1.63–1.78) | 1.98× (1.93–2.02) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.379 s | 554 | 10.3 | 2.38× (2.28–2.47) | 4.79× (4.51–5.08) | 5.65× (5.36–5.95) | 5.39× (5.10–5.70) | 5.11× (4.94–5.28) | 9.58× (8.84–10.38) | 8.92× (8.27–9.63) | 2.89× (2.66–3.14) | 3.35× (3.15–3.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.323 s | 649 | 17.1 | 2.34× (2.21–2.48) | 3.54× (3.33–3.76) | 4.45× (4.18–4.74) | 4.17× (3.87–4.49) | 4.22× (3.91–4.55) | 11.69× (11.03–12.38) | 10.88× (10.29–11.51) | 3.53× (3.33–3.75) | 4.09× (3.87–4.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 932 | 3.6 | 0.59× (0.56–0.62) | 3.56× (3.37–3.76) | 4.44× (4.24–4.65) | 3.39× (3.21–3.58) | 5.86× (5.54–6.20) | 2.70× (2.59–2.82) | 2.53× (2.41–2.66) | 1.03× (0.99–1.08) | 1.42× (1.33–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 913 | 3.7 | 0.57× (0.54–0.60) | 2.86× (2.74–2.99) | 5.97× (5.83–6.11) | 3.44× (3.35–3.54) | 5.80× (5.65–5.95) | 2.69× (2.61–2.78) | 2.52× (2.45–2.60) | 1.03× (0.97–1.09) | 1.41× (1.37–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 911 | 3.6 | 0.60× (0.55–0.65) | 2.79× (2.66–2.92) | 5.80× (5.58–6.03) | 3.36× (3.22–3.50) | 5.81× (5.60–6.04) | 2.65× (2.54–2.77) | 2.49× (2.38–2.59) | 1.01× (0.96–1.07) | 1.39× (1.33–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 521 | 6.1 | 2.59× (2.44–2.75) | 2.69× (2.58–2.80) | 3.47× (3.27–3.69) | 3.42× (3.28–3.56) | 3.80× (3.58–4.04) | 2.24× (2.15–2.34) | 1.91× (1.84–1.98) | 1.88× (1.83–1.92) | 2.67× (2.56–2.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 807 | 7.6 | 2.23× (2.04–2.44) | 2.61× (2.43–2.79) | 4.05× (3.78–4.33) | 3.23× (3.10–3.37) | 4.52× (4.39–4.65) | 3.49× (3.36–3.62) | 2.97× (2.87–3.07) | 2.92× (2.84–3.00) | 4.16× (4.00–4.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 715 | 14.3 | 1.77× (1.71–1.84) | 1.93× (1.81–2.05) | 2.83× (2.66–3.02) | 2.15× (1.99–2.32) | 3.68× (3.52–3.85) | 3.08× (2.94–3.23) | 2.63× (2.50–2.75) | 2.58× (2.48–2.68) | 3.68× (3.48–3.89) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 61 | 50.2 | 1.38× (1.36–1.41) | 2.81× (2.61–3.03) | 2.44× (2.38–2.50) | 2.47× (2.40–2.54) | 2.88× (2.82–2.94) | 3.19× (2.98–3.42) | 3.24× (2.95–3.57) | 1.26× (1.25–1.28) | 1.41× (1.38–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 56.6 | 1.24× (1.16–1.32) | 2.46× (2.31–2.63) | 2.46× (2.18–2.78) | 3.20× (2.77–3.69) | 2.84× (2.66–3.04) | 4.97× (4.59–5.38) | 5.05× (4.57–5.59) | 1.97× (1.85–2.09) | 2.20× (2.06–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 112 | 77.6 | 1.31× (1.25–1.38) | 2.08× (1.99–2.18) | 2.11× (2.00–2.21) | 2.67× (2.47–2.88) | 2.79× (2.65–2.94) | 5.89× (5.41–6.41) | 5.99× (5.36–6.69) | 2.33× (2.23–2.44) | 2.61× (2.49–2.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 47.2 | 1.57× (1.38–1.79) | 3.01× (2.89–3.12) | 2.53× (2.45–2.62) | 2.59× (2.53–2.66) | 3.04× (2.97–3.12) | 3.69× (3.58–3.80) | 3.80× (3.69–3.91) | 1.44× (1.41–1.48) | 1.68× (1.64–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.092 s | 104 | 54.8 | 1.37× (1.33–1.40) | 2.69× (2.59–2.78) | 2.51× (2.40–2.63) | 3.70× (3.46–3.96) | 3.06× (2.98–3.13) | 6.00× (5.83–6.18) | 6.19× (5.97–6.41) | 2.34× (2.28–2.41) | 2.74× (2.67–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 77.3 | 1.45× (1.38–1.52) | 2.38× (2.16–2.63) | 2.26× (2.11–2.41) | 3.06× (2.96–3.17) | 2.94× (2.70–3.20) | 6.97× (6.77–7.18) | 7.19× (7.00–7.38) | 2.73× (2.62–2.84) | 3.18× (3.07–3.29) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 694 | 5.0 | 0.64× (0.62–0.67) | 3.10× (2.87–3.36) | 4.20× (3.92–4.50) | 2.96× (2.82–3.12) | 6.39× (6.05–6.75) | 2.37× (2.23–2.52) | 2.23× (2.09–2.37) | 0.96× (0.92–1.01) | 1.43× (1.35–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.3 | 0.55× (0.54–0.57) | 2.73× (2.64–2.83) | 5.09× (4.94–5.24) | 2.95× (2.90–3.01) | 6.33× (6.02–6.65) | 2.28× (2.22–2.33) | 2.14× (2.07–2.20) | 0.92× (0.89–0.96) | 1.37× (1.32–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 627 | 5.4 | 0.54× (0.52–0.57) | 2.62× (2.48–2.76) | 4.79× (4.63–4.96) | 2.80× (2.69–2.91) | 5.83× (5.54–6.13) | 2.17× (2.08–2.27) | 2.04× (1.96–2.12) | 0.88× (0.85–0.91) | 1.31× (1.24–1.38) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.095 s | 130 | 23.8 | 1.41× (1.39–1.44) | 2.90× (2.82–2.98) | 3.14× (3.08–3.20) | 3.13× (3.08–3.18) | 3.32× (3.28–3.37) | 3.55× (3.49–3.62) | 3.30× (3.23–3.36) | 1.09× (1.07–1.12) | 1.35× (1.31–1.39) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.060 s | 203 | 28.0 | 1.25× (1.20–1.31) | 2.50× (2.42–2.58) | 2.98× (2.83–3.14) | 3.33× (3.18–3.48) | 3.24× (3.08–3.41) | 5.44× (5.15–5.74) | 5.05× (4.78–5.34) | 1.67× (1.57–1.77) | 2.06× (1.96–2.17) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.049 s | 250 | 38.3 | 1.33× (1.28–1.38) | 2.08× (1.95–2.23) | 2.60× (2.42–2.79) | 2.96× (2.82–3.10) | 3.17× (3.04–3.31) | 6.79× (6.52–7.07) | 6.30× (6.03–6.59) | 2.09× (2.01–2.16) | 2.57× (2.48–2.67) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.27–2.37) | 5.26× (5.10–5.43) | 5.58× (5.31–5.87) | 5.84× (5.64–6.04) | 5.25× (5.07–5.43) | 5.63× (5.41–5.85) | 5.24× (5.06–5.42) | 1.70× (1.63–1.78) | 1.97× (1.92–2.01) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.30× (2.24–2.35) | 4.98× (4.79–5.18) | 5.31× (5.14–5.49) | 5.65× (5.41–5.90) | 5.13× (5.01–5.26) | 5.09× (4.80–5.40) | 4.74× (4.48–5.02) | 1.54× (1.45–1.64) | 1.78× (1.70–1.86) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.21× (2.12–2.31) | 3.72× (3.54–3.92) | 4.01× (3.82–4.22) | 4.36× (4.10–4.64) | 3.95× (3.64–4.29) | 3.17× (2.98–3.37) | 2.95× (2.78–3.13) | 0.96× (0.90–1.02) | 1.11× (1.04–1.17) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.60–0.65) | 3.13× (3.00–3.27) | 3.87× (3.67–4.07) | 3.18× (3.03–3.35) | 5.44× (5.17–5.73) | 2.55× (2.45–2.66) | 2.40× (2.29–2.51) | 1.03× (0.99–1.07) | 1.11× (1.06–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.59–0.65) | 3.03× (2.91–3.16) | 5.28× (5.11–5.44) | 3.20× (3.13–3.28) | 5.32× (5.19–5.46) | 2.51× (2.43–2.59) | 2.36× (2.29–2.43) | 1.01× (0.95–1.07) | 1.09× (1.02–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.63× (0.58–0.68) | 3.28× (3.18–3.38) | 5.26× (5.05–5.48) | 3.19× (3.08–3.31) | 5.40× (5.23–5.59) | 2.52× (2.42–2.61) | 2.36× (2.27–2.45) | 1.01× (0.97–1.06) | 1.09× (1.04–1.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.37–2.65) | 2.50× (2.41–2.60) | 3.22× (3.06–3.39) | 3.29× (3.17–3.43) | 3.66× (3.45–3.89) | 2.18× (2.10–2.27) | 1.86× (1.80–1.93) | 1.83× (1.79–1.88) | 2.46× (2.38–2.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (2.03–2.20) | 2.29× (2.20–2.38) | 2.90× (2.82–2.98) | 2.94× (2.83–3.05) | 3.08× (2.99–3.17) | 1.77× (1.71–1.83) | 1.51× (1.46–1.56) | 1.49× (1.45–1.52) | 1.99× (1.93–2.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.90× (1.82–1.99) | 1.60× (1.52–1.69) | 2.03× (1.94–2.13) | 1.85× (1.74–1.98) | 1.91× (1.79–2.04) | 0.91× (0.86–0.96) | 0.78× (0.73–0.82) | 0.76× (0.73–0.80) | 1.03× (0.97–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.35–1.40) | 2.75× (2.55–2.96) | 2.37× (2.31–2.43) | 2.44× (2.38–2.52) | 2.85× (2.79–2.92) | 3.16× (2.95–3.38) | 3.21× (2.91–3.53) | 1.26× (1.25–1.27) | 1.36× (1.33–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.28–1.32) | 2.51× (2.47–2.54) | 2.36× (2.09–2.65) | 3.19× (2.70–3.77) | 2.62× (2.58–2.66) | 2.82× (2.62–3.04) | 2.87× (2.59–3.18) | 1.13× (1.11–1.14) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.54× (1.48–1.60) | 2.35× (2.23–2.48) | 1.94× (1.86–2.02) | 2.64× (2.54–2.73) | 2.48× (2.38–2.57) | 2.07× (1.90–2.25) | 2.10× (1.88–2.35) | 0.82× (0.80–0.85) | 0.89× (0.86–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.56× (1.38–1.78) | 2.95× (2.85–3.06) | 2.47× (2.39–2.55) | 2.57× (2.51–2.63) | 3.02× (2.94–3.09) | 3.65× (3.55–3.76) | 3.76× (3.66–3.87) | 1.44× (1.40–1.47) | 1.65× (1.62–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.40) | 2.69× (2.63–2.76) | 2.29× (2.23–2.35) | 3.21× (3.10–3.33) | 2.72× (2.67–2.78) | 3.16× (3.10–3.23) | 3.26× (3.17–3.36) | 1.24× (1.21–1.28) | 1.43× (1.40–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.57–1.69) | 2.62× (2.41–2.84) | 1.99× (1.86–2.14) | 2.83× (2.72–2.95) | 2.63× (2.44–2.83) | 2.25× (2.19–2.31) | 2.32× (2.25–2.39) | 0.88× (0.85–0.92) | 1.02× (0.98–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.66–0.71) | 2.67× (2.53–2.81) | 3.44× (3.24–3.66) | 2.74× (2.64–2.85) | 5.72× (5.47–5.99) | 2.20× (2.09–2.32) | 2.08× (1.97–2.20) | 0.97× (0.94–1.00) | 1.01× (0.96–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.63) | 2.54× (2.45–2.63) | 4.25× (4.10–4.40) | 2.72× (2.67–2.77) | 5.58× (5.33–5.84) | 2.07× (2.02–2.11) | 1.96× (1.90–2.01) | 0.91× (0.88–0.94) | 0.94× (0.88–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.57–0.62) | 2.62× (2.54–2.70) | 4.18× (4.01–4.35) | 2.68× (2.58–2.78) | 5.31× (5.07–5.57) | 2.04× (1.96–2.13) | 1.93× (1.86–2.00) | 0.90× (0.87–0.93) | 0.93× (0.87–0.99) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.40× (1.38–1.43) | 2.84× (2.77–2.91) | 3.05× (3.00–3.10) | 3.09× (3.04–3.14) | 3.27× (3.23–3.32) | 3.50× (3.44–3.56) | 3.25× (3.18–3.31) | 1.09× (1.06–1.11) | 1.28× (1.25–1.30) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.31× (1.27–1.34) | 2.46× (2.41–2.51) | 2.82× (2.72–2.92) | 3.03× (2.97–3.08) | 2.87× (2.80–2.95) | 2.92× (2.83–3.02) | 2.71× (2.62–2.81) | 0.91× (0.88–0.94) | 1.07× (1.03–1.10) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.55× (1.48–1.61) | 2.30× (2.16–2.45) | 2.54× (2.39–2.69) | 2.67× (2.52–2.82) | 2.64× (2.54–2.75) | 2.13× (2.05–2.22) | 1.98× (1.90–2.07) | 0.66× (0.64–0.69) | 0.78× (0.76–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.12× (12.09–12.14) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.47× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.62× (0.54–0.72) | 0.80× (0.79–0.81) | 0.66× (0.65–0.66) | 6.33× (6.29–6.38) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.69× (0.59–0.81) | 0.72× (0.71–0.73) | 0.62× (0.61–0.63) | 4.11× (4.07–4.16) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.78–1.79) | 0.84× (0.84–0.84) | 4.42× (4.41–4.42) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.17) | 1.79× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.72) | 0.67× (0.66–0.67) | 8.07× (8.06–8.09) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.67–0.68) | 0.73× (0.68–0.78) | 1.64× (1.62–1.66) | 0.82× (0.80–0.83) | 5.96× (5.88–6.03) | 0.48× (0.47–0.49) | 0.51× (0.50–0.51) | 0.55× (0.54–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.66) | 0.75× (0.71–0.79) | 1.17× (1.15–1.19) | 0.71× (0.70–0.72) | 3.43× (3.38–3.47) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.41× (0.38–0.44) | 0.92× (0.90–0.94) | 0.74× (0.72–0.75) | 3.18× (3.12–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.47) | 0.87× (0.84–0.89) | 0.80× (0.77–0.84) | 2.51× (2.45–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.84) | 5.24× (5.23–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.41) | 0.86× (0.84–0.89) | 0.70× (0.67–0.74) | 2.95× (2.86–3.03) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.44–0.51) | 0.80× (0.79–0.82) | 0.83× (0.80–0.86) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.08× (1.08–1.09) | 0.77× (0.77–0.78) | 5.41× (5.41–5.42) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.78) | 5.39× (5.38–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.41× (0.41–0.41) | 1.48× (1.47–1.48) | 0.68× (0.68–0.69) | 6.65× (6.64–6.66) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.32) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.40× (0.36–0.43) | 1.13× (1.08–1.18) | 0.71× (0.68–0.75) | 3.53× (3.44–3.62) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 52 MB | 0.54× (0.52–0.56) | 0.55× (0.50–0.61) | 0.99× (0.94–1.04) | 0.79× (0.75–0.83) | 2.53× (2.42–2.63) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 318 | 3.15 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 554 | 3.43 | 1.10× (1.08–1.13) | 85% | 1.04× (1.01–1.09) | 1.09× (1.05–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 649 | 5.67 | 1.78× (1.66–1.90) | 52% | 1.26× (1.18–1.34) | 1.70× (1.67–1.72) |

**Best:** mibz2 at 4 threads (649 MB/s); lbzip2 at 4 threads (186 MB/s); pbz2 at 4 threads (281 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.72, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.50, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.98 | 0.02 | 0.09 | 0.323 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3013 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.06 | 11.9% |
| MTF + RLE2 | 1.40 | 15.8% |
| T-vector build | 1.98 | 22.3% |
| walk (the one each build uses) | 3.91 | 44.1% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.27 | 3.0% |
| CRC | 0.24 | 2.7% |
| **total** | 8.88 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.673 | 1.302 | (in its total) | 1.93× (1.93–1.94) |  |
| walk | 1.225 | 2.619 | (in its total) | 2.13× (2.12–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.056 | (in its total) | 1.22× (1.20–1.24) |  |
| inverse BWT, total | 1.944 | 3.978 | 3.270 | 2.04× (2.03–2.05) | 1.68× (1.67–1.69) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h37m ± 12m04s |
| 4 threads | 47m20s ± 7m15s |
| 4 threads, the sample's overlap gap as a fixed cost | 35m41s ± 5m40s |
| the scanner alone (its serial part) | 15m34s ± 30.2s |
| the workers alone, over 4 | 35m41s ± 5m40s |

At 4 threads the workers bound it. Per byte: the scanner 9.0 ns a byte of bz2, the workers 4.68 ns a byte of output at 4 threads (CPU, summed over them), 2.70 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3012.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
