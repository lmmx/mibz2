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

The latest record: [`1a5f96f`](<https://github.com/lmmx/mibz2/commit/1a5f96f801227503498799a95c5b69a725336fcb>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44111305942>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.418 s | 502 | 5.2 | 2.95× (2.92–2.98) | 6.86× (6.73–7.00) | 7.06× (6.98–7.13) | 8.03× (7.93–8.13) | 7.07× (6.94–7.21) | 7.41× (7.31–7.52) | 7.10× (7.07–7.14) | 2.19× (2.16–2.22) | 2.42× (2.41–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.259 s | 811 | 5.7 | 2.86× (2.73–3.01) | 5.79× (5.50–6.11) | 6.74× (6.42–7.07) | 6.97× (6.70–7.24) | 6.12× (5.84–6.42) | 12.07× (11.60–12.55) | 11.56× (11.13–12.01) | 3.56× (3.40–3.74) | 3.94× (3.80–4.09) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.200 s | 1,051 | 8.7 | 3.05× (2.98–3.13) | 4.49× (4.43–4.56) | 5.63× (5.43–5.85) | 5.48× (5.30–5.66) | 5.30× (5.19–5.42) | 15.52× (15.13–15.92) | 14.87× (14.67–15.07) | 4.58× (4.46–4.71) | 5.07× (4.99–5.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,108 | 2.6 | 0.57× (0.56–0.58) | 3.39× (3.33–3.45) | 4.52× (4.44–4.60) | 3.54× (3.49–3.59) | 6.02× (5.87–6.18) | 2.64× (2.59–2.69) | 2.63× (2.59–2.68) | 0.97× (0.95–1.00) | 1.91× (1.73–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 2.6 | 0.53× (0.51–0.54) | 2.81× (2.74–2.89) | 5.62× (5.47–5.77) | 3.54× (3.45–3.63) | 5.96× (5.85–6.07) | 2.61× (2.55–2.68) | 2.61× (2.54–2.68) | 0.97× (0.94–0.99) | 1.89× (1.71–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,078 | 2.6 | 0.53× (0.52–0.53) | 2.77× (2.72–2.82) | 5.53× (5.43–5.63) | 3.46× (3.39–3.53) | 5.87× (5.73–6.01) | 2.56× (2.52–2.61) | 2.56× (2.52–2.61) | 0.95× (0.93–0.97) | 1.85× (1.66–2.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 742 | 3.7 | 2.72× (2.71–2.73) | 3.09× (3.07–3.10) | 3.86× (3.84–3.88) | 4.13× (4.10–4.15) | 4.56× (4.50–4.61) | 2.80× (2.77–2.83) | 2.29× (2.27–2.31) | 2.12× (2.09–2.15) | 3.07× (3.06–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,074 | 4.7 | 2.14× (1.91–2.40) | 2.69× (2.42–3.00) | 4.17× (3.70–4.70) | 3.32× (2.97–3.70) | 4.83× (4.34–5.38) | 3.88× (3.47–4.33) | 3.17× (2.84–3.55) | 2.93× (2.64–3.26) | 4.26× (3.82–4.74) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 936 | 9.2 | 1.87× (1.78–1.95) | 1.85× (1.70–2.02) | 3.00× (2.85–3.16) | 2.13× (2.02–2.24) | 3.92× (3.80–4.05) | 3.51× (3.38–3.65) | 2.87× (2.75–3.00) | 2.66× (2.53–2.79) | 3.86× (3.70–4.02) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 37.2 | 1.36× (1.35–1.38) | 2.60× (2.56–2.63) | 2.36× (2.31–2.41) | 2.45× (2.42–2.47) | 2.73× (2.71–2.75) | 2.93× (2.90–2.97) | 3.09× (3.08–3.11) | 1.23× (1.21–1.25) | 1.40× (1.36–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 41.1 | 1.28× (1.23–1.32) | 2.46× (2.39–2.53) | 2.35× (2.27–2.43) | 3.08× (2.95–3.22) | 2.85× (2.75–2.95) | 4.79× (4.64–4.95) | 5.05× (4.89–5.22) | 2.01× (1.93–2.08) | 2.28× (2.18–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 141 | 54.5 | 1.34× (1.30–1.39) | 2.14× (2.07–2.22) | 2.14× (2.08–2.21) | 2.67× (2.56–2.78) | 2.88× (2.83–2.94) | 5.84× (5.72–5.96) | 6.16× (6.02–6.30) | 2.45× (2.38–2.51) | 2.78× (2.69–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.120 s | 80 | 32.9 | 1.52× (1.51–1.53) | 3.05× (3.02–3.08) | 2.62× (2.60–2.64) | 2.78× (2.75–2.81) | 3.14× (3.03–3.26) | 3.68× (3.65–3.72) | 4.09× (4.06–4.12) | 1.53× (1.50–1.56) | 1.80× (1.77–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 37.9 | 1.37× (1.32–1.42) | 2.85× (2.70–3.01) | 2.53× (2.46–2.60) | 3.48× (3.38–3.58) | 3.05× (2.95–3.14) | 5.93× (5.76–6.12) | 6.59× (6.39–6.80) | 2.46× (2.39–2.54) | 2.90× (2.80–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.065 s | 148 | 52.0 | 1.41× (1.38–1.43) | 2.25× (2.18–2.32) | 2.22× (2.15–2.29) | 2.98× (2.92–3.03) | 2.81× (2.73–2.89) | 6.80× (6.68–6.92) | 7.55× (7.43–7.68) | 2.82× (2.75–2.90) | 3.32× (3.26–3.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 3.5 | 0.62× (0.62–0.63) | 3.45× (3.42–3.48) | 4.33× (4.29–4.38) | 3.10× (3.07–3.13) | 6.67× (6.59–6.76) | 2.28× (2.26–2.30) | 2.32× (2.30–2.34) | 0.96× (0.94–0.98) | 1.73× (1.72–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 790 | 3.8 | 0.51× (0.51–0.52) | 2.45× (2.43–2.48) | 4.92× (4.87–4.97) | 2.95× (2.91–2.99) | 6.28× (6.09–6.46) | 2.15× (2.13–2.17) | 2.19× (2.17–2.21) | 0.91× (0.89–0.92) | 1.64× (1.62–1.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 787 | 3.7 | 0.52× (0.51–0.52) | 2.44× (2.40–2.47) | 4.88× (4.80–4.96) | 2.93× (2.89–2.97) | 6.36× (6.24–6.49) | 2.13× (2.10–2.16) | 2.17× (2.13–2.20) | 0.90× (0.87–0.92) | 1.62× (1.59–1.64) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 0.057 s | 225 | 11.9 | 1.84× (1.79–1.88) | 3.93× (3.88–3.99) | 4.31× (4.25–4.37) | 4.57× (4.49–4.65) | 4.73× (4.66–4.81) | 4.68× (4.63–4.72) | 4.79× (4.73–4.85) | 1.49× (1.48–1.50) | 1.78× (1.76–1.81) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 0.040 s | 324 | 14.8 | 1.56× (1.33–1.83) | 3.31× (3.06–3.57) | 3.85× (3.74–3.97) | 4.33× (4.14–4.54) | 4.28× (4.15–4.41) | 6.76× (6.54–6.98) | 6.92× (6.71–7.13) | 2.15× (2.09–2.21) | 2.58× (2.50–2.65) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 0.035 s | 367 | 22.8 | 1.49× (1.43–1.54) | 2.52× (2.36–2.68) | 3.17× (3.02–3.34) | 3.46× (3.37–3.56) | 3.91× (3.80–4.02) | 7.61× (7.47–7.76) | 7.80× (7.64–7.95) | 2.42× (2.37–2.48) | 2.90× (2.84–2.97) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.92–2.97) | 6.82× (6.69–6.96) | 7.02× (6.95–7.09) | 8.00× (7.90–8.10) | 7.05× (6.91–7.19) | 7.38× (7.28–7.49) | 7.08× (7.04–7.11) | 2.18× (2.15–2.22) | 2.41× (2.39–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.81× (2.75–2.87) | 6.25× (6.16–6.35) | 6.57× (6.45–6.68) | 7.54× (7.40–7.68) | 6.36× (6.24–6.49) | 6.65× (6.48–6.81) | 6.37× (6.27–6.47) | 1.97× (1.91–2.02) | 2.17× (2.13–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.00× (2.92–3.08) | 4.95× (4.87–5.03) | 5.44× (5.28–5.60) | 6.02× (5.91–6.14) | 5.15× (5.08–5.23) | 4.39× (4.29–4.48) | 4.20× (4.14–4.26) | 1.30× (1.27–1.32) | 1.43× (1.41–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.93× (2.85–3.01) | 3.89× (3.79–3.99) | 3.31× (3.26–3.37) | 5.57× (5.43–5.70) | 2.49× (2.44–2.54) | 2.49× (2.44–2.54) | 0.98× (0.95–1.00) | 1.63× (1.60–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.71× (2.65–2.78) | 4.96× (4.87–5.06) | 3.29× (3.21–3.37) | 5.45× (5.36–5.55) | 2.43× (2.37–2.49) | 2.43× (2.37–2.49) | 0.95× (0.93–0.98) | 1.59× (1.55–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.99× (2.94–3.05) | 4.91× (4.81–5.01) | 3.29× (3.23–3.34) | 5.46× (5.35–5.58) | 2.44× (2.40–2.48) | 2.44× (2.40–2.48) | 0.96× (0.94–0.98) | 1.60× (1.57–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.60–2.63) | 2.86× (2.84–2.87) | 3.43× (3.40–3.46) | 3.93× (3.91–3.95) | 4.34× (4.29–4.39) | 2.69× (2.66–2.72) | 2.21× (2.19–2.23) | 2.05× (2.02–2.08) | 2.67× (2.64–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.19–2.31) | 2.50× (2.39–2.60) | 3.29× (3.21–3.37) | 3.27× (3.18–3.36) | 3.57× (3.48–3.67) | 2.12× (2.06–2.19) | 1.75× (1.69–1.80) | 1.62× (1.58–1.66) | 2.10× (2.04–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.02× (1.92–2.11) | 1.69× (1.63–1.76) | 2.21× (2.14–2.27) | 2.03× (1.97–2.10) | 2.08× (2.01–2.15) | 1.08× (1.05–1.11) | 0.89× (0.86–0.92) | 0.82× (0.79–0.85) | 1.07× (1.03–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.37) | 2.52× (2.49–2.54) | 2.30× (2.26–2.35) | 2.42× (2.40–2.45) | 2.71× (2.69–2.73) | 2.90× (2.87–2.93) | 3.06× (3.05–3.08) | 1.23× (1.21–1.24) | 1.34× (1.33–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.24–1.30) | 2.40× (2.36–2.45) | 2.13× (2.08–2.18) | 2.89× (2.83–2.95) | 2.49× (2.44–2.54) | 2.60× (2.55–2.66) | 2.74× (2.69–2.80) | 1.10× (1.07–1.13) | 1.20× (1.17–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.37–1.50) | 2.27× (2.21–2.33) | 1.94× (1.90–1.98) | 2.60× (2.54–2.65) | 2.48× (2.43–2.53) | 1.98× (1.94–2.02) | 2.09× (2.05–2.13) | 0.84× (0.81–0.86) | 0.92× (0.89–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.50–1.52) | 2.98× (2.96–2.99) | 2.57× (2.55–2.59) | 2.75× (2.72–2.79) | 3.11× (3.01–3.23) | 3.65× (3.61–3.68) | 4.05× (4.01–4.08) | 1.52× (1.49–1.55) | 1.75× (1.73–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.40) | 2.75× (2.71–2.78) | 2.34× (2.32–2.37) | 3.30× (3.26–3.35) | 2.74× (2.72–2.77) | 3.18× (3.15–3.22) | 3.53× (3.49–3.57) | 1.33× (1.30–1.36) | 1.53× (1.50–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.52–1.66) | 2.58× (2.52–2.64) | 2.04× (2.00–2.08) | 2.92× (2.85–2.99) | 2.63× (2.56–2.70) | 2.32× (2.26–2.38) | 2.57× (2.50–2.65) | 0.97× (0.95–0.99) | 1.11× (1.08–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.47× (2.45–2.50) | 3.44× (3.37–3.51) | 2.84× (2.81–2.86) | 5.95× (5.88–6.03) | 2.12× (2.10–2.14) | 2.16× (2.14–2.17) | 0.97× (0.95–0.99) | 1.37× (1.35–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.58–0.59) | 2.24× (2.21–2.26) | 4.05× (3.98–4.12) | 2.72× (2.68–2.76) | 5.54× (5.38–5.70) | 1.96× (1.94–1.99) | 2.00× (1.98–2.02) | 0.90× (0.89–0.91) | 1.27× (1.25–1.29) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.48× (2.45–2.51) | 4.05× (3.96–4.14) | 2.78× (2.74–2.81) | 5.77× (5.65–5.89) | 2.00× (1.97–2.03) | 2.04× (2.00–2.07) | 0.92× (0.89–0.94) | 1.29× (1.27–1.31) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 1.81× (1.76–1.86) | 3.73× (3.69–3.78) | 4.11× (4.06–4.17) | 4.46× (4.39–4.54) | 4.62× (4.55–4.69) | 4.56× (4.52–4.61) | 4.67× (4.62–4.73) | 1.48× (1.46–1.49) | 1.61× (1.59–1.64) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 1.53× (1.50–1.56) | 3.21× (3.11–3.32) | 3.52× (3.48–3.57) | 4.09× (4.03–4.16) | 3.78× (3.72–3.83) | 3.67× (3.62–3.73) | 3.76× (3.71–3.81) | 1.19× (1.17–1.20) | 1.30× (1.28–1.32) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 1.63× (1.54–1.72) | 2.65× (2.56–2.73) | 2.90× (2.81–2.99) | 3.30× (3.22–3.37) | 3.13× (3.05–3.22) | 2.39× (2.34–2.45) | 2.45× (2.39–2.51) | 0.77× (0.76–0.79) | 0.85× (0.82–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.97× (10.96–10.99) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.31–0.34) | 0.40× (0.34–0.48) | 0.68× (0.65–0.71) | 0.57× (0.55–0.60) | 5.41× (5.20–5.64) | 0.13× (0.12–0.14) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.32–0.35) | 0.47× (0.42–0.52) | 0.60× (0.58–0.62) | 0.53× (0.51–0.55) | 3.50× (3.39–3.61) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.63–0.69) | 0.68× (0.61–0.76) | 1.58× (1.52–1.65) | 0.80× (0.76–0.83) | 5.75× (5.53–5.99) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.65× (0.62–0.67) | 0.68× (0.63–0.72) | 1.14× (1.09–1.18) | 0.70× (0.68–0.73) | 3.33× (3.21–3.46) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.17) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.47–0.52) | 0.41× (0.38–0.44) | 0.90× (0.86–0.94) | 0.73× (0.70–0.76) | 3.12× (2.99–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.80× (0.77–0.83) | 2.52× (2.46–2.57) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.50) | 0.37× (0.34–0.40) | 0.86× (0.84–0.88) | 0.73× (0.70–0.77) | 2.96× (2.89–3.04) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.62) | 0.50× (0.47–0.53) | 0.81× (0.79–0.83) | 0.81× (0.78–0.83) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.36× (5.34–5.37) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.65–0.66) | 6.37× (6.36–6.39) | 0.39× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 35 MB | 0.47× (0.46–0.49) | 0.43× (0.40–0.46) | 1.10× (1.06–1.14) | 0.73× (0.72–0.75) | 3.51× (3.43–3.60) | 0.22× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.44× (0.40–0.49) | 0.91× (0.88–0.93) | 0.74× (0.71–0.77) | 2.31× (2.26–2.36) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 502 | 2.00 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 811 | 2.21 | 1.11× (1.09–1.13) | 81% | 1.02× (0.99–1.04) | 1.06× (1.03–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 1,051 | 3.36 | 1.68× (1.66–1.71) | 52% | 1.22× (1.19–1.25) | 1.71× (1.66–1.77) |

**Best:** mibz2 at 4 threads (1,051 MB/s); lbzip2 at 4 threads (232 MB/s); pbz2 at 4 threads (337 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.19, lbzip2 1t 0.07, lbzip2 2t 0.21, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.53 | 0.02 | 0.08 | 0.200 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.79 | 12.6% |
| MTF + RLE2 | 1.09 | 17.4% |
| T-vector build | 1.07 | 17.0% |
| walk (the one each build uses) | 3.00 | 47.9% |
| of which: segment assembly | 0.12 | 2.0% |
| RLE1 | 0.13 | 2.1% |
| CRC | 0.18 | 2.9% |
| **total** | 6.26 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.393 | 1.067 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.078 | 2.071 | (in its total) | 1.93× (1.91–1.95) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.049 | (in its total) | 1.12× (1.08–1.17) |  |
| inverse BWT, total | 1.514 | 3.187 | 1.899 | 2.11× (2.09–2.13) | 1.25× (1.25–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m57s |
| 4 threads | 29m08s ± 4m07s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m30s ± 2m48s |
| the scanner alone (its serial part) | 11m30s ± 19.7s |
| the workers alone, over 4 | 19m30s ± 2m48s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.56 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,519,058 | 12,810,756 | \`6ae766e8f83bdb21…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
