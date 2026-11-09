# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-305/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`5133bdd`](<https://github.com/lmmx/mibz2/commit/5133bdd2cd259539e525ae584a03498f90d56b69>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42621762153>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.662 s | 317 | 10.4 | 2.31× (2.29–2.33) | 5.28× (5.24–5.32) | 5.44× (5.39–5.50) | 5.88× (5.83–5.92) | 5.29× (5.23–5.35) | 5.66× (5.59–5.74) | 5.40× (5.33–5.46) | 1.71× (1.70–1.72) | 1.96× (1.93–1.98) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.367 s | 571 | 11.2 | 2.30× (2.16–2.44) | 4.93× (4.65–5.24) | 5.56× (5.21–5.94) | 5.56× (5.26–5.87) | 5.00× (4.71–5.31) | 9.92× (9.36–10.52) | 9.45× (8.93–10.01) | 3.00× (2.82–3.18) | 3.43× (3.22–3.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.300 s | 700 | 17.8 | 2.37× (2.34–2.40) | 3.65× (3.60–3.70) | 4.53× (4.42–4.64) | 4.15× (4.06–4.23) | 4.17× (4.11–4.24) | 12.47× (12.32–12.61) | 11.87× (11.73–12.02) | 3.76× (3.73–3.80) | 4.30× (4.25–4.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.7 | 0.60× (0.59–0.61) | 3.67× (3.52–3.82) | 4.56× (4.40–4.73) | 3.59× (3.55–3.63) | 6.19× (6.08–6.30) | 2.85× (2.81–2.89) | 2.68× (2.64–2.71) | 1.03× (0.98–1.09) | 1.52× (1.50–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 968 | 3.8 | 0.56× (0.55–0.57) | 2.94× (2.87–3.01) | 5.89× (5.78–6.00) | 3.50× (3.43–3.57) | 5.97× (5.86–6.08) | 2.76× (2.70–2.82) | 2.59× (2.53–2.65) | 1.00× (0.94–1.05) | 1.47× (1.43–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 950 | 3.8 | 0.57× (0.55–0.58) | 2.96× (2.88–3.04) | 5.81× (5.66–5.97) | 3.49× (3.42–3.57) | 6.03× (5.91–6.15) | 2.76× (2.69–2.84) | 2.59× (2.53–2.66) | 1.00× (0.94–1.06) | 1.47× (1.43–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 541 | 6.4 | 2.72× (2.62–2.82) | 2.83× (2.80–2.85) | 3.60× (3.55–3.65) | 3.55× (3.52–3.58) | 3.86× (3.82–3.90) | 2.33× (2.31–2.35) | 1.96× (1.94–1.98) | 1.88× (1.87–1.90) | 2.81× (2.79–2.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.026 s | 760 | 7.9 | 1.84× (1.53–2.21) | 2.35× (2.06–2.68) | 3.47× (3.01–4.01) | 2.74× (2.31–3.25) | 3.87× (3.26–4.58) | 3.03× (2.55–3.60) | 2.55× (2.15–3.03) | 2.45× (2.06–2.90) | 3.66× (3.08–4.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 731 | 15.1 | 1.73× (1.66–1.80) | 1.87× (1.76–1.99) | 2.79× (2.66–2.92) | 2.13× (2.06–2.21) | 3.79× (3.59–4.01) | 3.13× (3.01–3.26) | 2.64× (2.54–2.74) | 2.53× (2.43–2.63) | 3.78× (3.64–3.94) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.131 s | 61 | 54.4 | 1.38× (1.36–1.39) | 2.73× (2.71–2.76) | 2.46× (2.42–2.50) | 2.50× (2.48–2.52) | 2.88× (2.86–2.91) | 3.11× (3.07–3.14) | 3.21× (3.18–3.23) | 1.25× (1.24–1.27) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 98 | 61.3 | 1.28× (1.20–1.37) | 2.58× (2.34–2.84) | 2.36× (2.25–2.47) | 3.00× (2.87–3.14) | 2.89× (2.75–3.03) | 4.91× (4.68–5.14) | 5.06× (4.83–5.30) | 1.98× (1.89–2.07) | 2.25× (2.14–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 83.8 | 1.30× (1.27–1.34) | 2.15× (2.08–2.24) | 2.03× (1.97–2.08) | 2.56× (2.49–2.64) | 2.80× (2.74–2.86) | 5.84× (5.70–5.99) | 6.03× (5.88–6.18) | 2.36× (2.31–2.41) | 2.67× (2.59–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 66 | 50.2 | 1.49× (1.48–1.49) | 3.10× (3.04–3.15) | 2.59× (2.55–2.62) | 2.66× (2.63–2.70) | 3.11× (3.08–3.13) | 3.78× (3.75–3.81) | 3.96× (3.93–3.99) | 1.44× (1.43–1.45) | 1.71× (1.69–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 56.5 | 1.35× (1.31–1.39) | 2.79× (2.71–2.87) | 2.49× (2.41–2.58) | 3.44× (3.34–3.54) | 3.07× (2.99–3.15) | 6.14× (5.98–6.30) | 6.44× (6.27–6.61) | 2.34× (2.28–2.41) | 2.78× (2.70–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 127 | 81.0 | 1.45× (1.41–1.50) | 2.27× (2.20–2.35) | 2.26× (2.21–2.30) | 3.02× (2.95–3.09) | 2.83× (2.77–2.88) | 7.19× (7.03–7.35) | 7.54× (7.39–7.69) | 2.75× (2.68–2.81) | 3.25× (3.17–3.33) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.1 | 0.66× (0.64–0.67) | 3.02× (2.95–3.09) | 4.42× (4.32–4.53) | 3.10× (3.04–3.16) | 6.72× (6.57–6.88) | 2.45× (2.40–2.50) | 2.32× (2.27–2.37) | 0.97× (0.95–0.99) | 1.52× (1.48–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 690 | 5.5 | 0.55× (0.54–0.56) | 2.83× (2.78–2.89) | 4.87× (4.77–4.97) | 2.95× (2.89–3.00) | 6.34× (6.24–6.44) | 2.30× (2.25–2.34) | 2.18× (2.14–2.22) | 0.91× (0.89–0.92) | 1.42× (1.39–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 685 | 5.4 | 0.56× (0.55–0.57) | 2.84× (2.80–2.88) | 4.86× (4.79–4.93) | 2.93× (2.89–2.98) | 6.31× (6.23–6.39) | 2.29× (2.26–2.32) | 2.17× (2.15–2.20) | 0.91× (0.89–0.92) | 1.42× (1.40–1.44) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.091 s | 135 | 24.9 | 1.40× (1.39–1.42) | 2.95× (2.91–2.99) | 3.18× (3.13–3.23) | 3.18× (3.14–3.22) | 3.38× (3.33–3.43) | 3.60× (3.55–3.65) | 3.49× (3.41–3.58) | 1.08× (1.07–1.10) | 1.32× (1.30–1.34) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.058 s | 212 | 29.2 | 1.26× (1.06–1.49) | 2.62× (2.24–3.06) | 2.85× (2.60–3.13) | 3.23× (2.98–3.50) | 3.15× (2.86–3.47) | 5.40× (4.90–5.95) | 5.24× (4.75–5.79) | 1.63× (1.48–1.79) | 1.98× (1.80–2.19) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.047 s | 259 | 40.4 | 1.28× (1.23–1.32) | 2.10× (2.01–2.20) | 2.54× (2.46–2.63) | 3.00× (2.89–3.11) | 3.19× (3.09–3.30) | 6.95× (6.72–7.19) | 6.74× (6.47–7.03) | 2.09× (2.03–2.16) | 2.55× (2.47–2.64) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.31× (2.29–2.33) | 5.26× (5.22–5.30) | 5.42× (5.37–5.47) | 5.86× (5.82–5.91) | 5.28× (5.22–5.34) | 5.65× (5.58–5.72) | 5.38× (5.32–5.45) | 1.71× (1.70–1.72) | 1.95× (1.92–1.97) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.22–2.25) | 5.02× (4.99–5.06) | 5.27× (5.24–5.29) | 5.77× (5.74–5.80) | 4.99× (4.97–5.01) | 5.25× (5.18–5.31) | 5.00× (4.95–5.04) | 1.59× (1.57–1.60) | 1.81× (1.79–1.83) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.23× (2.19–2.28) | 3.78× (3.74–3.82) | 4.06× (4.01–4.12) | 4.26× (4.21–4.32) | 3.82× (3.76–3.87) | 3.32× (3.26–3.38) | 3.16× (3.11–3.21) | 1.00× (0.99–1.02) | 1.14× (1.13–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.30× (3.26–3.35) | 4.17× (4.05–4.29) | 3.37× (3.33–3.42) | 5.76× (5.64–5.87) | 2.70× (2.66–2.74) | 2.53× (2.50–2.57) | 1.03× (0.99–1.08) | 1.08× (1.04–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.04× (2.98–3.10) | 5.31× (5.23–5.40) | 3.27× (3.21–3.33) | 5.50× (5.40–5.59) | 2.58× (2.53–2.63) | 2.42× (2.37–2.47) | 0.99× (0.94–1.04) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.62) | 3.42× (3.35–3.48) | 5.38× (5.25–5.51) | 3.31× (3.24–3.38) | 5.59× (5.50–5.68) | 2.61× (2.55–2.68) | 2.45× (2.40–2.51) | 1.00× (0.95–1.05) | 1.04× (1.01–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.63× (2.54–2.73) | 2.61× (2.59–2.64) | 3.28× (3.26–3.31) | 3.42× (3.39–3.45) | 3.71× (3.67–3.76) | 2.26× (2.24–2.29) | 1.92× (1.90–1.93) | 1.84× (1.82–1.85) | 2.54× (2.52–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (2.06–2.16) | 2.32× (2.27–2.37) | 2.84× (2.78–2.91) | 2.91× (2.85–2.98) | 3.13× (3.05–3.20) | 1.82× (1.77–1.87) | 1.54× (1.50–1.57) | 1.47× (1.44–1.51) | 2.04× (1.99–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.87–2.01) | 1.67× (1.60–1.74) | 2.10× (2.02–2.17) | 1.99× (1.90–2.09) | 2.04× (1.96–2.13) | 0.96× (0.92–0.99) | 0.81× (0.78–0.84) | 0.78× (0.75–0.81) | 1.07× (1.04–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.39) | 2.67× (2.64–2.69) | 2.40× (2.37–2.43) | 2.48× (2.46–2.50) | 2.86× (2.83–2.88) | 3.08× (3.04–3.11) | 3.18× (3.15–3.20) | 1.25× (1.24–1.26) | 1.38× (1.36–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.19–1.33) | 2.44× (2.32–2.57) | 2.16× (2.05–2.27) | 2.87× (2.73–3.03) | 2.54× (2.41–2.67) | 2.68× (2.55–2.81) | 2.76× (2.63–2.90) | 1.09× (1.03–1.14) | 1.20× (1.14–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.45–1.55) | 2.31× (2.23–2.39) | 1.89× (1.84–1.95) | 2.58× (2.50–2.66) | 2.45× (2.38–2.53) | 2.01× (1.96–2.06) | 2.07× (2.02–2.12) | 0.82× (0.79–0.84) | 0.90× (0.87–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.48× (1.47–1.49) | 3.05× (3.00–3.10) | 2.54× (2.50–2.57) | 2.64× (2.61–2.68) | 3.08× (3.05–3.10) | 3.74× (3.71–3.77) | 3.92× (3.89–3.95) | 1.44× (1.42–1.45) | 1.66× (1.65–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.40) | 2.80× (2.76–2.83) | 2.33× (2.32–2.35) | 3.26× (3.21–3.31) | 2.78× (2.75–2.81) | 3.30× (3.27–3.34) | 3.47× (3.44–3.49) | 1.27× (1.25–1.29) | 1.47× (1.45–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.53–1.66) | 2.55× (2.51–2.60) | 1.93× (1.90–1.97) | 2.78× (2.74–2.82) | 2.53× (2.50–2.57) | 2.32× (2.28–2.36) | 2.43× (2.40–2.47) | 0.89× (0.88–0.90) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.71× (2.65–2.78) | 3.57× (3.49–3.64) | 2.84× (2.79–2.90) | 6.02× (5.88–6.17) | 2.28× (2.23–2.33) | 2.16× (2.12–2.21) | 0.98× (0.96–0.99) | 1.01× (0.97–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.49× (2.44–2.53) | 4.21× (4.15–4.28) | 2.73× (2.68–2.78) | 5.61× (5.52–5.70) | 2.10× (2.06–2.14) | 1.99× (1.96–2.03) | 0.90× (0.88–0.92) | 0.93× (0.89–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.70–2.79) | 4.29× (4.23–4.34) | 2.79× (2.75–2.83) | 5.72× (5.65–5.80) | 2.15× (2.12–2.17) | 2.04× (2.01–2.06) | 0.92× (0.91–0.93) | 0.95× (0.91–1.00) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.40× (1.38–1.41) | 2.85× (2.80–2.89) | 3.09× (3.05–3.13) | 3.14× (3.10–3.17) | 3.33× (3.28–3.39) | 3.54× (3.49–3.60) | 3.44× (3.36–3.53) | 1.08× (1.07–1.10) | 1.27× (1.26–1.29) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.29× (1.28–1.30) | 2.58× (2.53–2.62) | 2.81× (2.78–2.85) | 3.08× (3.04–3.13) | 2.95× (2.91–2.99) | 3.05× (2.99–3.10) | 2.96× (2.87–3.04) | 0.93× (0.92–0.95) | 1.10× (1.08–1.11) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.49× (1.45–1.54) | 2.25× (2.18–2.33) | 2.53× (2.48–2.58) | 2.66× (2.61–2.71) | 2.67× (2.60–2.74) | 2.20× (2.16–2.25) | 2.14× (2.08–2.19) | 0.67× (0.66–0.68) | 0.79× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.10× (12.08–12.11) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.58× (0.49–0.68) | 0.80× (0.79–0.81) | 0.67× (0.66–0.68) | 6.38× (6.33–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.77× (0.66–0.90) | 0.71× (0.70–0.72) | 0.62× (0.60–0.63) | 4.11× (4.10–4.13) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.30–4.30) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.78–1.79) | 0.84× (0.84–0.84) | 4.42× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.72) | 0.67× (0.66–0.67) | 8.08× (8.05–8.11) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.68) | 0.73× (0.68–0.79) | 1.62× (1.58–1.65) | 0.80× (0.78–0.82) | 5.85× (5.73–5.96) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.66) | 0.73× (0.70–0.77) | 1.15× (1.09–1.21) | 0.71× (0.68–0.73) | 3.37× (3.23–3.51) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.37–0.44) | 0.91× (0.88–0.94) | 0.72× (0.70–0.74) | 3.15× (3.05–3.26) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.80–0.84) | 2.55× (2.50–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.84× (0.84–0.84) | 5.25× (5.24–5.26) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.39× (0.37–0.41) | 0.86× (0.83–0.90) | 0.71× (0.67–0.75) | 2.94× (2.82–3.07) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.48–0.52) | 0.80× (0.79–0.82) | 0.81× (0.78–0.84) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.21) | 1.09× (1.09–1.09) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.77–0.77) | 5.35× (5.33–5.37) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.63× (1.62–1.64) | 0.77× (0.77–0.78) | 5.39× (5.37–5.41) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.41× (0.40–0.41) | 1.48× (1.48–1.48) | 0.68× (0.68–0.69) | 6.66× (6.65–6.67) | 0.40× (0.40–0.40) | 0.44× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.32) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.46× (0.42–0.50) | 1.13× (1.11–1.16) | 0.72× (0.70–0.73) | 3.51× (3.43–3.60) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.35) | 0.17× (0.17–0.18) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 52 MB | 0.53× (0.52–0.54) | 0.52× (0.48–0.56) | 0.97× (0.94–0.99) | 0.77× (0.75–0.79) | 2.48× (2.42–2.55) | 0.14× (0.14–0.15) | 0.16× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 317 | 3.17 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 571 | 3.40 | 1.08× (1.07–1.09) | 88% | 1.03× (1.02–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 700 | 5.40 | 1.70× (1.68–1.73) | 55% | 1.22× (1.21–1.24) | 1.65× (1.61–1.69) |

**Best:** mibz2 at 4 threads (700 MB/s); lbzip2 at 4 threads (192 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.66, mibz2 4t 1.09, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.56, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.91 | 0.02 | 0.09 | 0.300 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3288 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 11.6% |
| MTF + RLE2 | 1.60 | 16.5% |
| T-vector build | 2.14 | 22.0% |
| walk (the one each build uses) | 4.30 | 44.1% |
| of which: segment assembly | 0.16 | 1.6% |
| RLE1 | 0.29 | 3.0% |
| CRC | 0.26 | 2.7% |
| **total** | 9.75 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.302 | (in its total) | 1.94× (1.94–1.94) |  |
| walk | 1.250 | 2.627 | (in its total) | 2.11× (2.10–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.21× (1.20–1.22) |  |
| inverse BWT, total | 1.968 | 3.987 | 3.197 | 2.03× (2.03–2.03) | 1.63× (1.62–1.64) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h36m ± 11m18s |
| 4 threads | 43m43s ± 6m08s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m08s ± 4m41s |
| the scanner alone (its serial part) | 15m43s ± 26.3s |
| the workers alone, over 4 | 33m08s ± 4m41s |

At 4 threads the workers bound it. Per byte: the scanner 9.1 ns a byte of bz2, the workers 4.34 ns a byte of output at 4 threads (CPU, summed over them), 2.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3288.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
