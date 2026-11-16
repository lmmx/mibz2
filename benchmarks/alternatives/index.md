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

The latest record: [`ee38d2a`](<https://github.com/lmmx/mibz2/commit/ee38d2ae71d70889aad4f777efb884f80a96e5c5>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43799547084>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.415 s | 505 | 7.3 | 2.93× (2.92–2.94) | 6.72× (6.69–6.74) | 6.96× (6.91–7.00) | 7.93× (7.88–7.99) | 6.92× (6.90–6.95) | 7.27× (7.25–7.29) | 6.90× (6.87–6.92) | 2.17× (2.15–2.18) | 2.43× (2.42–2.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.241 s | 868 | 8.0 | 2.96× (2.80–3.14) | 6.02× (5.77–6.28) | 6.95× (6.71–7.19) | 7.15× (6.94–7.38) | 6.32× (6.12–6.53) | 12.43× (12.04–12.83) | 11.79× (11.43–12.16) | 3.71× (3.59–3.83) | 4.15× (4.01–4.31) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.200 s | 1,048 | 12.3 | 2.94× (2.88–3.00) | 4.41× (4.32–4.51) | 5.44× (5.30–5.58) | 5.27× (5.16–5.38) | 5.19× (5.09–5.29) | 15.12× (14.95–15.30) | 14.35× (14.16–14.54) | 4.51× (4.45–4.57) | 5.06× (4.99–5.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,117 | 3.6 | 0.57× (0.56–0.58) | 3.41× (3.34–3.48) | 4.65× (4.54–4.76) | 3.56× (3.49–3.62) | 6.07× (5.93–6.22) | 2.65× (2.60–2.71) | 2.65× (2.59–2.70) | 0.97× (0.94–1.01) | 1.71× (1.67–1.74) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,099 | 3.8 | 0.53× (0.52–0.54) | 2.84× (2.80–2.88) | 5.69× (5.60–5.77) | 3.56× (3.51–3.61) | 6.00× (5.87–6.13) | 2.64× (2.61–2.68) | 2.64× (2.60–2.68) | 0.97× (0.94–1.00) | 1.70× (1.68–1.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,090 | 3.7 | 0.53× (0.52–0.54) | 2.80× (2.76–2.84) | 5.59× (5.51–5.67) | 3.50× (3.45–3.55) | 5.94× (5.83–6.04) | 2.59× (2.55–2.63) | 2.59× (2.55–2.63) | 0.95× (0.92–0.99) | 1.67× (1.65–1.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 740 | 5.3 | 2.68× (2.66–2.70) | 3.07× (3.05–3.09) | 3.84× (3.81–3.86) | 4.09× (4.06–4.11) | 4.47× (4.42–4.53) | 2.76× (2.75–2.78) | 2.26× (2.25–2.28) | 2.09× (2.06–2.12) | 3.06× (3.04–3.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,083 | 6.7 | 2.16× (1.95–2.40) | 2.80× (2.53–3.11) | 4.32× (3.90–4.79) | 3.46× (3.13–3.82) | 4.90× (4.43–5.41) | 3.94× (3.60–4.31) | 3.23× (2.95–3.53) | 2.98× (2.74–3.23) | 4.36× (3.98–4.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 947 | 13.0 | 1.81× (1.76–1.86) | 1.86× (1.70–2.03) | 2.98× (2.79–3.18) | 2.14× (2.03–2.26) | 4.00× (3.90–4.11) | 3.58× (3.49–3.67) | 2.93× (2.86–3.00) | 2.70× (2.62–2.79) | 3.96× (3.86–4.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 s | 73 | 51.3 | 1.39× (1.38–1.39) | 2.69× (2.67–2.71) | 2.43× (2.40–2.46) | 2.52× (2.51–2.52) | 2.83× (2.80–2.86) | 3.01× (3.00–3.02) | 3.14× (3.12–3.16) | 1.26× (1.25–1.26) | 1.42× (1.38–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 58.0 | 1.28× (1.24–1.32) | 2.46× (2.41–2.51) | 2.38× (2.29–2.47) | 2.99× (2.94–3.04) | 2.86× (2.80–2.92) | 4.82× (4.71–4.94) | 5.03× (4.91–5.16) | 2.02× (1.97–2.07) | 2.27× (2.20–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 78.0 | 1.32× (1.29–1.35) | 2.06× (1.97–2.16) | 2.05× (1.98–2.14) | 2.52× (2.44–2.60) | 2.84× (2.77–2.91) | 5.76× (5.62–5.91) | 6.01× (5.86–6.17) | 2.41× (2.35–2.47) | 2.72× (2.62–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.119 s | 81 | 46.4 | 1.52× (1.49–1.54) | 3.06× (3.02–3.10) | 2.66× (2.60–2.72) | 2.76× (2.72–2.80) | 3.14× (3.03–3.26) | 3.68× (3.62–3.73) | 3.98× (3.93–4.04) | 1.51× (1.48–1.53) | 1.79× (1.76–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 52.9 | 1.37× (1.35–1.39) | 2.93× (2.76–3.10) | 2.59× (2.52–2.67) | 3.43× (3.37–3.50) | 3.10× (3.07–3.13) | 6.04× (5.97–6.11) | 6.55× (6.47–6.63) | 2.48× (2.44–2.52) | 2.94× (2.89–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 151 | 74.7 | 1.40× (1.36–1.43) | 2.24× (2.14–2.36) | 2.23× (2.17–2.29) | 3.00× (2.85–3.15) | 2.84× (2.77–2.92) | 6.90× (6.70–7.10) | 7.48× (7.24–7.72) | 2.83× (2.73–2.94) | 3.35× (3.24–3.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 848 | 5.0 | 0.63× (0.62–0.64) | 3.47× (3.45–3.49) | 4.37× (4.34–4.39) | 3.11× (3.09–3.13) | 6.75× (6.58–6.92) | 2.29× (2.28–2.30) | 2.33× (2.32–2.34) | 0.97× (0.96–0.99) | 1.74× (1.74–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 816 | 5.3 | 0.53× (0.52–0.53) | 2.51× (2.48–2.55) | 5.05× (4.98–5.11) | 3.02× (2.98–3.06) | 6.45× (6.28–6.63) | 2.20× (2.16–2.23) | 2.23× (2.20–2.27) | 0.93× (0.91–0.96) | 1.68× (1.65–1.70) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 790 | 5.3 | 0.52× (0.51–0.54) | 2.46× (2.42–2.50) | 4.93× (4.85–5.01) | 2.96× (2.91–3.01) | 6.26× (6.19–6.33) | 2.15× (2.11–2.19) | 2.18× (2.15–2.22) | 0.91× (0.89–0.94) | 1.64× (1.61–1.66) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.058 s | 232 | 16.4 | 1.84× (1.83–1.85) | 4.06× (4.03–4.09) | 4.42× (4.40–4.43) | 4.63× (4.62–4.64) | 4.80× (4.79–4.82) | 4.80× (4.78–4.81) | 4.79× (4.78–4.81) | 1.52× (1.51–1.53) | 1.77× (1.76–1.77) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.039 s | 344 | 20.4 | 1.51× (1.46–1.56) | 3.25× (3.13–3.37) | 3.90× (3.76–4.06) | 4.37× (4.12–4.63) | 4.38× (4.19–4.58) | 6.90× (6.64–7.16) | 6.89× (6.63–7.16) | 2.18× (2.09–2.27) | 2.54× (2.44–2.64) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.036 s | 370 | 31.6 | 1.49× (1.44–1.54) | 2.51× (2.40–2.61) | 3.17× (3.06–3.28) | 3.30× (3.17–3.43) | 3.93× (3.77–4.09) | 7.62× (7.40–7.85) | 7.62× (7.40–7.84) | 2.41× (2.34–2.49) | 2.81× (2.73–2.89) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.92× (2.91–2.93) | 6.69× (6.66–6.71) | 6.92× (6.87–6.97) | 7.91× (7.85–7.96) | 6.90× (6.87–6.92) | 7.24× (7.22–7.27) | 6.87× (6.85–6.90) | 2.16× (2.15–2.18) | 2.41× (2.40–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.76× (2.73–2.79) | 6.23× (6.14–6.31) | 6.50× (6.46–6.54) | 7.50× (7.46–7.55) | 6.33× (6.30–6.36) | 6.61× (6.57–6.65) | 6.27× (6.24–6.30) | 1.97× (1.96–1.99) | 2.20× (2.19–2.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.89× (2.85–2.94) | 4.92× (4.80–5.04) | 5.28× (5.15–5.42) | 5.86× (5.77–5.96) | 5.10× (5.02–5.18) | 4.32× (4.27–4.37) | 4.10× (4.06–4.14) | 1.29× (1.28–1.31) | 1.44× (1.42–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.91× (2.85–2.96) | 3.95× (3.85–4.06) | 3.33× (3.27–3.39) | 5.62× (5.50–5.75) | 2.50× (2.46–2.55) | 2.50× (2.46–2.55) | 0.97× (0.94–1.00) | 1.62× (1.59–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.73× (2.69–2.77) | 4.96× (4.90–5.02) | 3.30× (3.26–3.35) | 5.48× (5.36–5.59) | 2.45× (2.42–2.49) | 2.45× (2.42–2.49) | 0.95× (0.93–0.98) | 1.59× (1.56–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.05× (2.98–3.12) | 4.95× (4.85–5.05) | 3.32× (3.28–3.37) | 5.53× (5.44–5.62) | 2.47× (2.43–2.50) | 2.47× (2.43–2.50) | 0.96× (0.92–1.00) | 1.60× (1.58–1.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.58× (2.56–2.59) | 2.84× (2.82–2.86) | 3.40× (3.34–3.46) | 3.90× (3.88–3.93) | 4.27× (4.22–4.32) | 2.66× (2.64–2.67) | 2.19× (2.18–2.20) | 2.02× (2.00–2.05) | 2.66× (2.63–2.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.23× (2.18–2.29) | 2.49× (2.41–2.57) | 3.31× (3.22–3.39) | 3.30× (3.23–3.38) | 3.58× (3.51–3.66) | 2.13× (2.09–2.18) | 1.76× (1.72–1.79) | 1.63× (1.59–1.66) | 2.14× (2.08–2.19) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.01× (1.98–2.04) | 1.68× (1.64–1.73) | 2.22× (2.16–2.28) | 2.06× (2.03–2.10) | 2.10× (2.06–2.15) | 1.08× (1.07–1.10) | 0.89× (0.88–0.91) | 0.83× (0.81–0.84) | 1.09× (1.06–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.59× (2.59–2.60) | 2.39× (2.36–2.42) | 2.49× (2.49–2.50) | 2.81× (2.78–2.84) | 2.98× (2.97–2.99) | 3.10× (3.09–3.12) | 1.25× (1.25–1.26) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.40× (2.37–2.43) | 2.13× (2.10–2.16) | 2.82× (2.77–2.86) | 2.50× (2.47–2.53) | 2.61× (2.57–2.65) | 2.72× (2.67–2.76) | 1.10× (1.08–1.12) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.39–1.51) | 2.26× (2.22–2.31) | 1.91× (1.87–1.95) | 2.53× (2.49–2.58) | 2.47× (2.41–2.53) | 1.95× (1.92–1.99) | 2.04× (2.01–2.07) | 0.82× (0.81–0.84) | 0.91× (0.89–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.49–1.53) | 2.97× (2.92–3.01) | 2.60× (2.54–2.66) | 2.74× (2.70–2.78) | 3.11× (3.00–3.23) | 3.64× (3.58–3.69) | 3.94× (3.89–3.99) | 1.50× (1.47–1.52) | 1.73× (1.70–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.40) | 2.77× (2.75–2.79) | 2.37× (2.33–2.41) | 3.20× (3.18–3.23) | 2.76× (2.75–2.78) | 3.20× (3.19–3.22) | 3.47× (3.45–3.50) | 1.32× (1.30–1.34) | 1.53× (1.51–1.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.65) | 2.48× (2.41–2.55) | 2.00× (1.97–2.04) | 2.82× (2.75–2.90) | 2.61× (2.54–2.67) | 2.29× (2.25–2.33) | 2.48× (2.44–2.53) | 0.94× (0.92–0.97) | 1.09× (1.07–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.45× (2.43–2.47) | 3.40× (3.39–3.42) | 2.84× (2.82–2.86) | 6.01× (5.88–6.15) | 2.12× (2.11–2.13) | 2.16× (2.15–2.17) | 0.98× (0.96–0.99) | 1.37× (1.35–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.27× (2.25–2.30) | 4.01× (3.98–4.05) | 2.77× (2.73–2.80) | 5.64× (5.54–5.75) | 2.00× (1.98–2.03) | 2.03× (2.01–2.06) | 0.92× (0.90–0.94) | 1.29× (1.27–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.52× (2.48–2.55) | 4.03× (3.98–4.09) | 2.80× (2.76–2.85) | 5.69× (5.63–5.75) | 2.02× (1.99–2.05) | 2.06× (2.03–2.09) | 0.93× (0.91–0.95) | 1.31× (1.28–1.33) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.82× (1.81–1.83) | 3.83× (3.82–3.84) | 4.23× (4.22–4.25) | 4.52× (4.51–4.53) | 4.69× (4.67–4.71) | 4.68× (4.66–4.70) | 4.68× (4.67–4.69) | 1.50× (1.49–1.51) | 1.64× (1.64–1.65) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.53× (1.52–1.55) | 3.21× (3.16–3.26) | 3.56× (3.50–3.61) | 4.11× (4.05–4.17) | 3.83× (3.76–3.90) | 3.74× (3.70–3.79) | 3.74× (3.69–3.79) | 1.20× (1.18–1.22) | 1.32× (1.30–1.33) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.68× (1.64–1.71) | 2.56× (2.49–2.62) | 2.88× (2.81–2.95) | 3.23× (3.17–3.29) | 3.09× (3.01–3.18) | 2.42× (2.36–2.47) | 2.42× (2.36–2.47) | 0.77× (0.76–0.79) | 0.85× (0.83–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.34× (0.32–0.35) | 0.98× (0.97–0.98) | 0.79× (0.78–0.79) | 11.04× (11.01–11.06) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.33) | 0.44× (0.36–0.52) | 0.67× (0.65–0.68) | 0.57× (0.55–0.58) | 5.39× (5.27–5.51) | 0.13× (0.12–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.53× (0.45–0.62) | 0.59× (0.57–0.62) | 0.52× (0.50–0.55) | 3.47× (3.34–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.00–8.07) | 0.65× (0.64–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.69) | 0.67× (0.60–0.76) | 1.56× (1.49–1.64) | 0.79× (0.75–0.82) | 5.67× (5.42–5.94) | 0.46× (0.44–0.48) | 0.48× (0.46–0.51) | 0.53× (0.50–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.69× (0.65–0.73) | 1.16× (1.12–1.20) | 0.71× (0.69–0.73) | 3.38× (3.29–3.48) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.52) | 0.39× (0.37–0.42) | 0.90× (0.87–0.93) | 0.71× (0.70–0.74) | 3.11× (3.00–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.46) | 0.87× (0.85–0.88) | 0.82× (0.80–0.83) | 2.53× (2.49–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.21–5.25) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.37–0.40) | 0.84× (0.81–0.87) | 0.69× (0.66–0.73) | 2.90× (2.80–3.01) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.51) | 0.79× (0.78–0.81) | 0.79× (0.76–0.82) | 2.30× (2.26–2.33) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.26) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.76× (0.76–0.77) | 5.31× (5.29–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.68× (0.68–0.68) | 6.42× (6.40–6.44) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.44× (0.44–0.45) | 0.40× (0.36–0.43) | 1.03× (1.01–1.05) | 0.66× (0.65–0.68) | 3.28× (3.24–3.31) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.47× (0.42–0.51) | 0.90× (0.88–0.92) | 0.75× (0.73–0.78) | 2.31× (2.25–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 505 | 1.99 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 868 | 2.17 | 1.10× (1.09–1.10) | 85% | 1.02× (1.01–1.04) | 1.04× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,048 | 3.32 | 1.68× (1.66–1.69) | 52% | 1.23× (1.21–1.26) | 1.66× (1.64–1.69) |

**Best:** mibz2 at 4 threads (1,048 MB/s); lbzip2 at 4 threads (239 MB/s); pbz2 at 4 threads (358 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.53 | 0.02 | 0.07 | 0.200 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.8% |
| MTF + RLE2 | 1.58 | 18.1% |
| T-vector build | 1.48 | 17.0% |
| walk (the one each build uses) | 4.09 | 47.0% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 2.9% |
| **total** | 8.70 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.397 | 1.061 | (in its total) | 2.68× (2.67–2.68) |  |
| walk | 1.056 | 2.051 | (in its total) | 1.94× (1.91–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.040 | (in its total) | 0.84× (0.78–0.91) |  |
| inverse BWT, total | 1.499 | 3.153 | 1.890 | 2.10× (2.07–2.12) | 1.26× (1.24–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m54s |
| 4 threads | 29m05s ± 4m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m23s ± 2m44s |
| the scanner alone (its serial part) | 11m12s ± 30.2s |
| the workers alone, over 4 | 19m23s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 6.5 ns a byte of bz2, the workers 2.54 ns a byte of output at 4 threads (CPU, summed over them), 1.61 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,453 | 13,353,938 | \`724a67aca091a7ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
