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

The latest record: [`238693a`](<https://github.com/lmmx/mibz2/commit/238693a4b6bd05faefc09766207f328c4849ff16>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46092740548>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.480 s | 437 | 7.5 | 3.15× (3.13–3.16) | 7.17× (7.15–7.19) | 7.30× (7.27–7.33) | 8.10× (8.07–8.13) | 7.20× (7.18–7.23) | 7.87× (7.84–7.90) | 7.38× (7.35–7.40) | 2.32× (2.31–2.32) | 2.81× (2.80–2.83) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.281 s | 746 | 8.0 | 2.99× (2.83–3.15) | 6.48× (6.18–6.80) | 7.32× (6.98–7.68) | 7.39× (7.07–7.73) | 6.58× (6.25–6.92) | 13.33× (12.66–14.03) | 12.49× (11.87–13.14) | 3.92× (3.73–4.12) | 4.76× (4.52–5.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.219 s | 959 | 12.6 | 3.21× (3.17–3.26) | 4.87× (4.81–4.93) | 6.10× (5.98–6.23) | 5.61× (5.55–5.68) | 5.59× (5.52–5.66) | 17.21× (17.04–17.38) | 16.12× (15.96–16.29) | 5.06× (5.02–5.11) | 6.15× (6.09–6.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 976 | 3.6 | 0.59× (0.59–0.60) | 3.54× (3.42–3.65) | 4.41× (4.36–4.47) | 3.55× (3.51–3.58) | 6.21× (6.12–6.30) | 2.96× (2.93–3.00) | 2.64× (2.62–2.67) | 1.00× (0.98–1.01) | 1.50× (1.49–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 920 | 4.0 | 0.54× (0.53–0.55) | 2.83× (2.79–2.86) | 5.62× (5.51–5.74) | 3.37× (3.33–3.42) | 5.79× (5.71–5.87) | 2.79× (2.75–2.83) | 2.49× (2.45–2.52) | 0.94× (0.92–0.95) | 1.41× (1.39–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 901 | 4.0 | 0.54× (0.53–0.55) | 2.79× (2.75–2.83) | 5.66× (5.50–5.83) | 3.31× (3.26–3.36) | 5.77× (5.69–5.86) | 2.74× (2.71–2.78) | 2.45× (2.41–2.48) | 0.92× (0.90–0.94) | 1.39× (1.37–1.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 605 | 5.7 | 2.71× (2.68–2.74) | 3.15× (3.13–3.18) | 4.08× (3.99–4.16) | 4.08× (4.05–4.11) | 4.37× (4.33–4.40) | 2.65× (2.62–2.68) | 2.18× (2.17–2.20) | 2.09× (2.07–2.11) | 3.14× (3.11–3.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 899 | 6.8 | 2.25× (2.09–2.41) | 2.88× (2.61–3.19) | 4.05× (3.80–4.32) | 3.53× (3.25–3.82) | 4.88× (4.57–5.22) | 3.89× (3.64–4.16) | 3.21× (3.00–3.42) | 3.07× (2.87–3.28) | 4.61× (4.31–4.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 867 | 13.0 | 2.14× (2.02–2.27) | 2.29× (2.14–2.45) | 3.22× (3.12–3.33) | 2.42× (2.34–2.51) | 4.38× (4.24–4.53) | 3.81× (3.69–3.94) | 3.14× (3.04–3.24) | 3.01× (2.93–3.09) | 4.52× (4.38–4.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.120 s | 67 | 49.5 | 1.47× (1.46–1.49) | 2.93× (2.87–2.98) | 2.61× (2.58–2.64) | 2.69× (2.66–2.72) | 3.12× (3.09–3.15) | 3.40× (3.36–3.44) | 3.47× (3.41–3.53) | 1.33× (1.31–1.36) | 1.55× (1.47–1.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 111 | 54.8 | 1.42× (1.39–1.46) | 2.88× (2.80–2.96) | 2.68× (2.63–2.74) | 3.48× (3.41–3.55) | 3.30× (3.24–3.37) | 5.66× (5.55–5.78) | 5.78× (5.62–5.95) | 2.23× (2.18–2.28) | 2.58× (2.45–2.72) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.060 s | 132 | 75.5 | 1.47× (1.44–1.50) | 2.38× (2.31–2.45) | 2.30× (2.24–2.37) | 2.90× (2.84–2.96) | 3.19× (3.13–3.25) | 6.74× (6.65–6.84) | 6.89× (6.75–7.02) | 2.65× (2.62–2.68) | 3.08× (2.96–3.19) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.133 s | 72 | 45.6 | 1.57× (1.56–1.59) | 3.29× (3.26–3.33) | 2.73× (2.70–2.76) | 2.87× (2.84–2.91) | 3.35× (3.31–3.38) | 3.91× (3.87–3.95) | 4.23× (4.19–4.27) | 1.51× (1.50–1.53) | 1.82× (1.81–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 118 | 51.5 | 1.43× (1.38–1.48) | 2.93× (2.82–3.04) | 2.66× (2.54–2.79) | 3.75× (3.54–3.98) | 3.28× (3.15–3.43) | 6.32× (6.05–6.60) | 6.84× (6.56–7.13) | 2.44× (2.35–2.54) | 2.95× (2.82–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.068 s | 142 | 72.3 | 1.57× (1.54–1.61) | 2.55× (2.49–2.62) | 2.46× (2.41–2.51) | 3.44× (3.28–3.60) | 3.19× (3.14–3.25) | 7.68× (7.58–7.79) | 8.32× (8.21–8.43) | 2.97× (2.94–3.01) | 3.58× (3.53–3.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.0 | 0.66× (0.66–0.67) | 3.04× (3.00–3.07) | 4.43× (4.39–4.48) | 3.11× (3.06–3.17) | 6.90× (6.76–7.03) | 2.53× (2.50–2.55) | 2.33× (2.30–2.36) | 0.99× (0.98–1.00) | 1.52× (1.51–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 5.7 | 0.54× (0.53–0.54) | 2.77× (2.74–2.80) | 4.76× (4.71–4.81) | 2.87× (2.83–2.90) | 6.28× (6.20–6.36) | 2.31× (2.29–2.33) | 2.13× (2.10–2.16) | 0.90× (0.89–0.92) | 1.39× (1.38–1.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 664 | 5.7 | 0.54× (0.53–0.55) | 2.73× (2.68–2.78) | 4.69× (4.59–4.78) | 2.83× (2.77–2.88) | 6.18× (6.07–6.30) | 2.27× (2.23–2.31) | 2.10× (2.06–2.14) | 0.89× (0.88–0.90) | 1.37× (1.34–1.40) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.062 s | 239 | 14.0 | 2.25× (2.24–2.26) | 4.41× (4.39–4.42) | 4.85× (4.84–4.86) | 4.87× (4.85–4.89) | 5.07× (5.03–5.11) | 5.46× (5.44–5.48) | 5.15× (5.14–5.16) | 1.69× (1.68–1.70) | 2.28× (2.27–2.28) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.043 s | 343 | 18.1 | 1.69× (1.60–1.79) | 3.59× (3.28–3.93) | 4.13× (3.90–4.38) | 4.74× (4.31–5.21) | 4.42× (4.18–4.67) | 7.64× (7.23–8.07) | 7.21× (6.82–7.62) | 2.37× (2.23–2.51) | 3.18× (3.01–3.37) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.038 s | 392 | 27.6 | 1.72× (1.66–1.78) | 2.75× (2.58–2.92) | 3.63× (3.50–3.77) | 3.84× (3.72–3.96) | 4.12× (4.00–4.26) | 8.93× (8.70–9.17) | 8.43× (8.22–8.65) | 2.77× (2.71–2.83) | 3.72× (3.63–3.82) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.12–3.15) | 7.13× (7.11–7.15) | 7.26× (7.24–7.29) | 8.07× (8.05–8.10) | 7.18× (7.15–7.20) | 7.84× (7.81–7.87) | 7.35× (7.32–7.38) | 2.31× (2.31–2.32) | 2.79× (2.78–2.80) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (3.04–3.07) | 6.88× (6.84–6.92) | 7.19× (7.16–7.22) | 8.03× (8.00–8.06) | 6.82× (6.79–6.84) | 7.33× (7.30–7.36) | 6.87× (6.84–6.90) | 2.16× (2.15–2.17) | 2.61× (2.60–2.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.05× (2.99–3.11) | 5.16× (5.06–5.27) | 5.54× (5.44–5.64) | 5.94× (5.84–6.04) | 5.19× (5.08–5.29) | 4.68× (4.60–4.77) | 4.39× (4.31–4.46) | 1.38× (1.36–1.40) | 1.67× (1.64–1.70) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.26× (3.22–3.30) | 4.06× (4.01–4.11) | 3.34× (3.30–3.38) | 5.78× (5.68–5.87) | 2.81× (2.77–2.85) | 2.51× (2.47–2.54) | 1.00× (0.98–1.01) | 1.04× (1.02–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.58) | 2.85× (2.81–2.89) | 5.01× (4.95–5.08) | 3.09× (3.05–3.13) | 5.21× (5.14–5.28) | 2.55× (2.52–2.58) | 2.27× (2.24–2.31) | 0.91× (0.89–0.92) | 0.95× (0.93–0.96) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.11× (3.05–3.17) | 5.02× (4.94–5.10) | 3.08× (3.03–3.13) | 5.26× (5.18–5.35) | 2.55× (2.52–2.59) | 2.28× (2.24–2.31) | 0.91× (0.89–0.93) | 0.95× (0.93–0.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.62× (2.59–2.65) | 2.90× (2.88–2.93) | 3.66× (3.54–3.78) | 3.92× (3.88–3.95) | 4.19× (4.15–4.22) | 2.57× (2.53–2.60) | 2.12× (2.10–2.14) | 2.03× (2.01–2.05) | 2.83× (2.81–2.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.34–2.42) | 2.71× (2.65–2.76) | 3.25× (3.20–3.29) | 3.45× (3.40–3.50) | 3.62× (3.56–3.68) | 2.14× (2.11–2.17) | 1.77× (1.75–1.80) | 1.70× (1.67–1.72) | 2.37× (2.33–2.40) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.17× (2.09–2.26) | 1.88× (1.81–1.95) | 2.36× (2.27–2.45) | 2.22× (2.12–2.33) | 2.26× (2.18–2.36) | 1.11× (1.07–1.15) | 0.92× (0.88–0.95) | 0.88× (0.85–0.91) | 1.23× (1.18–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.47× (1.45–1.48) | 2.86× (2.82–2.89) | 2.56× (2.53–2.59) | 2.66× (2.63–2.69) | 3.09× (3.06–3.12) | 3.36× (3.32–3.40) | 3.43× (3.37–3.49) | 1.33× (1.31–1.35) | 1.52× (1.45–1.59) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.38–1.40) | 2.72× (2.69–2.76) | 2.41× (2.38–2.44) | 3.25× (3.22–3.27) | 2.86× (2.83–2.89) | 3.03× (3.00–3.06) | 3.09× (3.05–3.14) | 1.20× (1.19–1.21) | 1.37× (1.31–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.50–1.62) | 2.46× (2.41–2.51) | 2.02× (1.99–2.05) | 2.79× (2.74–2.83) | 2.65× (2.62–2.68) | 2.20× (2.17–2.23) | 2.25× (2.20–2.29) | 0.87× (0.86–0.88) | 1.00× (0.95–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.57× (1.55–1.58) | 3.22× (3.19–3.26) | 2.68× (2.65–2.71) | 2.85× (2.82–2.88) | 3.31× (3.28–3.34) | 3.87× (3.83–3.90) | 4.18× (4.15–4.22) | 1.50× (1.49–1.52) | 1.77× (1.76–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.43–1.48) | 2.98× (2.93–3.04) | 2.49× (2.46–2.52) | 3.55× (3.48–3.62) | 2.99× (2.95–3.04) | 3.41× (3.35–3.47) | 3.69× (3.63–3.75) | 1.33× (1.30–1.35) | 1.57× (1.54–1.59) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.74× (1.72–1.75) | 2.77× (2.73–2.81) | 2.10× (2.06–2.13) | 3.09× (3.03–3.15) | 2.78× (2.75–2.82) | 2.44× (2.42–2.47) | 2.64× (2.61–2.68) | 0.95× (0.94–0.96) | 1.12× (1.11–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.72× (2.69–2.76) | 3.55× (3.52–3.59) | 2.86× (2.83–2.89) | 6.19× (6.09–6.30) | 2.35× (2.33–2.36) | 2.17× (2.15–2.20) | 1.00× (0.98–1.02) | 0.98× (0.96–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.58–0.59) | 2.35× (2.33–2.38) | 4.02× (3.99–4.05) | 2.59× (2.56–2.61) | 5.41× (5.35–5.47) | 2.05× (2.03–2.07) | 1.90× (1.87–1.92) | 0.87× (0.86–0.89) | 0.85× (0.83–0.87) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.56× (2.51–2.61) | 4.05× (3.96–4.15) | 2.62× (2.56–2.67) | 5.46× (5.36–5.56) | 2.07× (2.03–2.10) | 1.91× (1.88–1.94) | 0.88× (0.87–0.89) | 0.86× (0.83–0.88) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 2.21× (2.20–2.22) | 4.23× (4.21–4.24) | 4.60× (4.59–4.62) | 4.76× (4.74–4.78) | 4.95× (4.91–4.99) | 5.33× (5.31–5.35) | 5.02× (5.01–5.04) | 1.67× (1.66–1.68) | 2.15× (2.13–2.17) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.82× (1.80–1.84) | 3.47× (3.41–3.53) | 3.79× (3.73–3.84) | 4.24× (4.18–4.30) | 3.95× (3.89–4.02) | 4.14× (4.09–4.19) | 3.91× (3.85–3.96) | 1.30× (1.28–1.32) | 1.67× (1.64–1.70) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.83× (1.78–1.88) | 2.75× (2.71–2.78) | 3.05× (3.03–3.08) | 3.33× (3.29–3.37) | 3.13× (3.10–3.17) | 2.70× (2.67–2.73) | 2.55× (2.53–2.57) | 0.85× (0.84–0.86) | 1.09× (1.08–1.10) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.30× (0.29–0.31) | 0.98× (0.98–0.99) | 0.79× (0.78–0.79) | 11.09× (11.05–11.13) | 0.27× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.33–0.36) | 0.54× (0.49–0.60) | 0.72× (0.70–0.74) | 0.60× (0.58–0.62) | 5.69× (5.51–5.88) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.36× (0.34–0.38) | 0.66× (0.56–0.79) | 0.64× (0.61–0.67) | 0.56× (0.54–0.59) | 3.72× (3.56–3.89) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.87× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.93–0.93) | 4.87× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.01× (7.99–8.02) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.75× (0.75–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.73) | 0.76× (0.68–0.84) | 1.72× (1.69–1.75) | 0.86× (0.84–0.87) | 6.23× (6.13–6.33) | 0.51× (0.50–0.51) | 0.53× (0.52–0.54) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.72) | 0.83× (0.74–0.92) | 1.24× (1.18–1.30) | 0.76× (0.73–0.80) | 3.64× (3.48–3.81) | 0.29× (0.27–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.89× (0.86–0.92) | 0.69× (0.65–0.73) | 3.07× (2.98–3.17) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.67× (0.66–0.69) | 0.47× (0.46–0.47) | 0.90× (0.89–0.91) | 0.83× (0.80–0.86) | 2.60× (2.57–2.64) | 0.16× (0.15–0.16) | 0.17× (0.17–0.17) | 0.25× (0.25–0.26) | 0.13× (0.13–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.83–0.84) | 5.22× (5.20–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.37× (0.33–0.40) | 0.83× (0.80–0.86) | 0.68× (0.65–0.71) | 2.84× (2.74–2.95) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.51× (0.48–0.54) | 0.81× (0.80–0.82) | 0.81× (0.78–0.84) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.90) | 6.21× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.88× (1.87–1.89) | 0.90× (0.89–0.90) | 6.21× (6.18–6.24) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.40× (1.39–1.40) | 0.66× (0.66–0.66) | 6.41× (6.40–6.42) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.63) | 0.30× (0.30–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 34 MB | 0.49× (0.47–0.51) | 0.44× (0.39–0.49) | 1.09× (1.05–1.13) | 0.74× (0.72–0.76) | 3.62× (3.49–3.76) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.34–0.37) | 0.17× (0.17–0.18) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 55 MB | 0.52× (0.51–0.52) | 0.48× (0.46–0.50) | 0.93× (0.91–0.94) | 0.77× (0.75–0.79) | 2.39× (2.36–2.42) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 437 | 2.30 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 746 | 2.46 | 1.07× (1.07–1.07) | 85% | 1.03× (1.03–1.04) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 959 | 3.88 | 1.67× (1.65–1.70) | 55% | 1.21× (1.20–1.22) | 1.63× (1.60–1.65) |

**Best:** mibz2 at 4 threads (959 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.34, lbzip2 4t 0.49, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.64 | 0.02 | 0.08 | 0.219 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 10.0% |
| MTF + RLE2 | 1.58 | 18.6% |
| T-vector build | 1.51 | 17.7% |
| walk (the one each build uses) | 4.03 | 47.4% |
| of which: segment assembly | 0.15 | 1.8% |
| RLE1 | 0.28 | 3.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.50 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.512 | 1.303 | (in its total) | 2.54× (2.54–2.54) |  |
| walk | 1.213 | 2.615 | (in its total) | 2.15× (2.14–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.053 | (in its total) | 1.20× (1.19–1.21) |  |
| inverse BWT, total | 1.771 | 3.971 | 3.232 | 2.24× (2.23–2.25) | 1.82× (1.82–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h09m ± 8m21s |
| 4 threads | 31m55s ± 4m29s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m08s ± 3m17s |
| the scanner alone (its serial part) | 10m11s ± 0.00s |
| the workers alone, over 4 | 23m08s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.03 ns a byte of output at 4 threads (CPU, summed over them), 1.95 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,228,219 | 14,890,034 | \`9ad9de226af45720…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
