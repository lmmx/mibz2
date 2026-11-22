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

The latest record: [`6996139`](<https://github.com/lmmx/mibz2/commit/69961392739a455561cbdfe544a1529c53c57fbe>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44978556583>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.512 s | 410 | 6.0 | 3.06× (3.02–3.10) | 7.04× (6.94–7.15) | 7.23× (7.15–7.31) | 7.76× (7.70–7.82) | 7.08× (7.05–7.12) | 7.62× (7.59–7.65) | 7.08× (7.02–7.13) | 2.27× (2.25–2.29) | 2.63× (2.60–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.295 s | 710 | 6.6 | 2.96× (2.78–3.15) | 6.43× (5.90–7.00) | 7.46× (7.15–7.78) | 7.13× (6.69–7.61) | 6.53× (6.16–6.92) | 12.90× (12.16–13.68) | 11.97× (11.32–12.66) | 3.84× (3.62–4.08) | 4.45× (4.20–4.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 861 | 10.2 | 3.08× (3.02–3.14) | 4.93× (4.80–5.06) | 5.97× (5.82–6.13) | 5.40× (5.29–5.51) | 5.53× (5.35–5.72) | 16.02× (15.75–16.29) | 14.87× (14.67–15.07) | 4.77× (4.70–4.85) | 5.52× (5.44–5.61) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 930 | 2.9 | 0.58× (0.57–0.59) | 3.66× (3.51–3.81) | 4.52× (4.39–4.65) | 3.44× (3.38–3.50) | 6.06× (5.96–6.17) | 2.84× (2.79–2.90) | 2.54× (2.49–2.59) | 1.03× (1.00–1.06) | 1.43× (1.41–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 915 | 3.0 | 0.55× (0.53–0.57) | 2.80× (2.68–2.92) | 5.96× (5.72–6.21) | 3.36× (3.22–3.50) | 5.82× (5.56–6.09) | 2.77× (2.65–2.89) | 2.48× (2.38–2.58) | 1.00× (0.97–1.04) | 1.40× (1.34–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 911 | 2.9 | 0.56× (0.55–0.57) | 2.84× (2.79–2.88) | 5.98× (5.83–6.13) | 3.40× (3.34–3.45) | 5.92× (5.85–6.00) | 2.80× (2.75–2.84) | 2.50× (2.46–2.54) | 1.01× (0.98–1.05) | 1.41× (1.39–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 557 | 4.6 | 2.54× (2.49–2.60) | 2.90× (2.84–2.95) | 3.76× (3.62–3.91) | 3.69× (3.62–3.75) | 4.08× (4.02–4.14) | 2.45× (2.41–2.50) | 2.04× (2.00–2.07) | 1.99× (1.96–2.02) | 2.88× (2.83–2.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 830 | 5.9 | 2.12× (1.75–2.58) | 2.79× (2.27–3.43) | 3.85× (3.30–4.48) | 3.01× (2.60–3.47) | 4.40× (3.77–5.14) | 3.42× (2.95–3.97) | 2.84× (2.44–3.30) | 2.78× (2.39–3.22) | 4.02× (3.46–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 736 | 11.1 | 1.83× (1.77–1.88) | 1.87× (1.72–2.03) | 2.81× (2.65–2.99) | 2.06× (1.97–2.15) | 3.90× (3.74–4.06) | 3.23× (3.10–3.37) | 2.68× (2.58–2.79) | 2.62× (2.52–2.72) | 3.80× (3.65–3.96) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.133 s | 60 | 41.4 | 1.36× (1.35–1.36) | 2.73× (2.69–2.76) | 2.49× (2.45–2.53) | 2.49× (2.47–2.52) | 2.91× (2.88–2.95) | 3.16× (3.12–3.19) | 3.11× (3.07–3.14) | 1.25× (1.24–1.27) | 1.43× (1.41–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 47.6 | 1.24× (1.16–1.32) | 2.44× (2.29–2.60) | 2.33× (2.19–2.48) | 3.00× (2.88–3.12) | 2.88× (2.70–3.07) | 4.94× (4.63–5.27) | 4.86× (4.55–5.19) | 1.96× (1.84–2.09) | 2.24× (2.09–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 115 | 63.1 | 1.32× (1.28–1.37) | 2.20× (2.12–2.28) | 2.16× (2.08–2.24) | 2.62× (2.50–2.73) | 2.93× (2.82–3.05) | 6.05× (5.88–6.22) | 5.95× (5.77–6.14) | 2.40× (2.32–2.48) | 2.74× (2.65–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.149 s | 64 | 38.5 | 1.44× (1.42–1.46) | 3.02× (3.00–3.05) | 2.61× (2.49–2.74) | 2.63× (2.60–2.65) | 3.11× (3.04–3.18) | 3.62× (3.58–3.66) | 3.84× (3.79–3.89) | 1.44× (1.41–1.47) | 1.72× (1.70–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 106 | 44.1 | 1.33× (1.28–1.37) | 2.73× (2.67–2.79) | 2.48× (2.40–2.55) | 3.37× (3.31–3.43) | 3.04× (2.98–3.10) | 5.91× (5.80–6.02) | 6.27× (6.15–6.39) | 2.35× (2.28–2.41) | 2.81× (2.76–2.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 121 | 62.6 | 1.36× (1.32–1.40) | 2.29× (2.20–2.38) | 2.24× (2.18–2.31) | 2.97× (2.91–3.03) | 2.89× (2.82–2.96) | 6.78× (6.62–6.94) | 7.19× (7.01–7.37) | 2.69× (2.61–2.77) | 3.22× (3.15–3.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 702 | 4.0 | 0.65× (0.64–0.66) | 3.15× (2.92–3.40) | 4.26× (4.09–4.43) | 3.01× (2.98–3.05) | 6.74× (6.62–6.86) | 2.46× (2.39–2.53) | 2.23× (2.20–2.26) | 0.97× (0.96–0.99) | 1.45× (1.43–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 4.3 | 0.55× (0.54–0.56) | 2.70× (2.56–2.84) | 4.97× (4.74–5.22) | 2.90× (2.85–2.95) | 6.42× (6.28–6.56) | 2.34× (2.30–2.39) | 2.12× (2.09–2.16) | 0.93× (0.90–0.95) | 1.39× (1.36–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 667 | 4.2 | 0.54× (0.53–0.56) | 2.75× (2.69–2.80) | 5.07× (4.86–5.29) | 2.90× (2.84–2.97) | 6.41× (6.28–6.54) | 2.33× (2.28–2.37) | 2.11× (2.07–2.14) | 0.92× (0.89–0.95) | 1.38× (1.35–1.40) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.076 s | 178 | 14.1 | 1.93× (1.91–1.96) | 4.05× (3.92–4.19) | 4.45× (4.34–4.57) | 4.41× (4.32–4.50) | 4.62× (4.55–4.68) | 5.18× (5.08–5.28) | 4.62× (4.56–4.69) | 1.53× (1.49–1.57) | 1.87× (1.84–1.89) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.054 s | 251 | 17.8 | 1.57× (1.47–1.69) | 3.18× (2.90–3.48) | 3.83× (3.50–4.19) | 4.06× (3.74–4.41) | 4.07× (3.71–4.47) | 7.19× (6.67–7.77) | 6.42× (5.96–6.92) | 2.12× (1.95–2.31) | 2.59× (2.40–2.80) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.046 s | 291 | 27.2 | 1.53× (1.49–1.58) | 2.62× (2.52–2.72) | 3.43× (3.27–3.61) | 3.31× (3.18–3.43) | 3.84× (3.69–3.99) | 8.49× (8.27–8.72) | 7.58× (7.39–7.77) | 2.51× (2.45–2.56) | 3.06× (3.00–3.13) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.05× (3.01–3.09) | 7.01× (6.91–7.12) | 7.20× (7.11–7.28) | 7.73× (7.67–7.79) | 7.06× (7.02–7.09) | 7.59× (7.56–7.63) | 7.05× (6.99–7.11) | 2.27× (2.25–2.29) | 2.61× (2.58–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.92× (2.87–2.96) | 6.66× (6.40–6.92) | 7.03× (6.82–7.25) | 7.52× (7.34–7.71) | 6.59× (6.44–6.74) | 6.90× (6.83–6.96) | 6.40× (6.34–6.46) | 2.06× (2.03–2.08) | 2.37× (2.34–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.00× (2.96–3.05) | 5.31× (5.16–5.47) | 5.65× (5.48–5.82) | 5.83× (5.72–5.94) | 5.31× (5.15–5.47) | 4.45× (4.38–4.52) | 4.13× (4.05–4.21) | 1.33× (1.30–1.35) | 1.53× (1.50–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.15× (3.10–3.20) | 3.94× (3.88–4.00) | 3.23× (3.17–3.28) | 5.62× (5.53–5.72) | 2.68× (2.63–2.73) | 2.41× (2.36–2.45) | 1.03× (1.00–1.06) | 1.04× (1.00–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.61) | 2.90× (2.78–3.02) | 5.15× (4.94–5.36) | 3.13× (3.01–3.26) | 5.33× (5.10–5.56) | 2.57× (2.47–2.68) | 2.31× (2.22–2.40) | 0.99× (0.96–1.02) | 1.00× (0.96–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.27× (3.21–3.33) | 5.30× (5.21–5.40) | 3.24× (3.19–3.29) | 5.51× (5.44–5.59) | 2.65× (2.62–2.69) | 2.38× (2.35–2.42) | 1.02× (0.99–1.05) | 1.03× (0.99–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.46× (2.41–2.51) | 2.69× (2.64–2.74) | 3.48× (3.35–3.61) | 3.54× (3.48–3.60) | 3.91× (3.85–3.97) | 2.37× (2.33–2.42) | 1.98× (1.94–2.01) | 1.93× (1.90–1.96) | 2.64× (2.58–2.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.15× (2.09–2.21) | 2.37× (2.29–2.45) | 2.92× (2.85–3.00) | 2.95× (2.87–3.03) | 3.22× (3.13–3.33) | 1.87× (1.82–1.91) | 1.56× (1.52–1.60) | 1.52× (1.47–1.57) | 2.08× (2.02–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.99–2.15) | 1.67× (1.59–1.76) | 2.17× (2.09–2.25) | 2.04× (1.96–2.12) | 2.13× (2.05–2.21) | 1.00× (0.96–1.04) | 0.83× (0.80–0.87) | 0.81× (0.78–0.85) | 1.11× (1.06–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.66× (2.63–2.70) | 2.43× (2.40–2.47) | 2.47× (2.44–2.50) | 2.88× (2.85–2.92) | 3.12× (3.09–3.16) | 3.07× (3.04–3.10) | 1.25× (1.23–1.26) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.27) | 2.46× (2.42–2.50) | 2.20× (2.17–2.23) | 2.88× (2.83–2.93) | 2.60× (2.53–2.68) | 2.74× (2.67–2.80) | 2.69× (2.64–2.74) | 1.09× (1.07–1.11) | 1.21× (1.19–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.43–1.54) | 2.43× (2.33–2.53) | 1.94× (1.85–2.03) | 2.63× (2.53–2.74) | 2.59× (2.50–2.69) | 2.05× (2.00–2.10) | 2.02× (1.96–2.07) | 0.82× (0.79–0.84) | 0.91× (0.88–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.42–1.45) | 2.97× (2.95–3.00) | 2.57× (2.46–2.68) | 2.60× (2.58–2.63) | 3.08× (3.02–3.16) | 3.58× (3.55–3.62) | 3.80× (3.76–3.85) | 1.43× (1.41–1.46) | 1.67× (1.64–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.31–1.34) | 2.71× (2.68–2.74) | 2.28× (2.24–2.32) | 3.15× (3.11–3.19) | 2.72× (2.66–2.78) | 3.12× (3.07–3.16) | 3.31× (3.25–3.36) | 1.25× (1.21–1.28) | 1.45× (1.42–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.52–1.57) | 2.55× (2.45–2.64) | 2.00× (1.94–2.06) | 2.79× (2.71–2.88) | 2.63× (2.52–2.74) | 2.20× (2.15–2.25) | 2.33× (2.29–2.38) | 0.88× (0.86–0.90) | 1.02× (1.00–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.65× (2.59–2.70) | 3.55× (3.44–3.66) | 2.76× (2.73–2.78) | 6.02× (5.93–6.11) | 2.27× (2.22–2.33) | 2.07× (2.05–2.09) | 0.98× (0.96–0.99) | 0.98× (0.93–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.62) | 2.44× (2.41–2.46) | 4.25× (4.17–4.34) | 2.68× (2.64–2.72) | 5.67× (5.56–5.78) | 2.13× (2.09–2.18) | 1.94× (1.91–1.98) | 0.92× (0.90–0.94) | 0.92× (0.87–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.58–0.61) | 2.69× (2.66–2.73) | 4.31× (4.19–4.44) | 2.76× (2.69–2.82) | 5.79× (5.68–5.91) | 2.17× (2.12–2.21) | 1.98× (1.94–2.02) | 0.93× (0.91–0.96) | 0.94× (0.89–0.99) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.91× (1.88–1.94) | 3.92× (3.79–4.05) | 4.30× (4.19–4.41) | 4.32× (4.23–4.41) | 4.52× (4.46–4.58) | 5.07× (4.97–5.16) | 4.52× (4.46–4.58) | 1.51× (1.47–1.56) | 1.77× (1.75–1.79) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.64× (1.60–1.67) | 3.26× (3.16–3.36) | 3.63× (3.48–3.79) | 3.99× (3.83–4.15) | 3.73× (3.56–3.90) | 4.00× (3.89–4.11) | 3.57× (3.50–3.64) | 1.20× (1.16–1.24) | 1.40× (1.37–1.43) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.70× (1.63–1.77) | 2.76× (2.64–2.88) | 3.04× (2.93–3.15) | 3.20× (3.08–3.33) | 3.10× (2.98–3.23) | 2.65× (2.61–2.69) | 2.37× (2.33–2.41) | 0.79× (0.77–0.81) | 0.93× (0.91–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.32× (0.30–0.33) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.94× (10.92–10.95) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.45× (0.38–0.52) | 0.68× (0.66–0.70) | 0.57× (0.55–0.59) | 5.45× (5.32–5.58) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.32–0.35) | 0.64× (0.55–0.76) | 0.60× (0.57–0.63) | 0.52× (0.50–0.55) | 3.49× (3.33–3.65) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.36× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.68× (1.68–1.68) | 0.66× (0.65–0.66) | 7.93× (7.92–7.94) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.71× (0.65–0.78) | 1.59× (1.56–1.61) | 0.80× (0.78–0.82) | 5.75× (5.67–5.83) | 0.47× (0.46–0.47) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.69× (0.63–0.76) | 1.13× (1.08–1.18) | 0.70× (0.67–0.73) | 3.35× (3.21–3.49) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.14× (5.12–5.17) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.51) | 0.40× (0.37–0.44) | 0.89× (0.87–0.92) | 0.72× (0.70–0.74) | 3.10× (3.00–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.80× (0.77–0.83) | 2.51× (2.45–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.16× (5.14–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.47× (0.45–0.50) | 0.35× (0.32–0.38) | 0.83× (0.80–0.87) | 0.69× (0.67–0.72) | 2.85× (2.74–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.52) | 0.79× (0.78–0.80) | 0.81× (0.78–0.84) | 2.29× (2.26–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.75–0.76) | 5.33× (5.31–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.24–5.28) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.67× (0.67–0.67) | 6.42× (6.40–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 41 MB | 0.41× (0.41–0.42) | 0.39× (0.37–0.42) | 0.97× (0.95–0.99) | 0.63× (0.62–0.63) | 3.05× (3.03–3.08) | 0.19× (0.19–0.19) | 0.20× (0.20–0.20) | 0.30× (0.30–0.30) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 61 MB | 0.45× (0.43–0.47) | 0.46× (0.43–0.50) | 0.83× (0.80–0.87) | 0.70× (0.67–0.73) | 2.13× (2.05–2.21) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.09–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 410 | 2.45 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 710 | 2.70 | 1.10× (1.09–1.11) | 85% | 1.05× (1.01–1.08) | 1.05× (1.04–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 861 | 4.18 | 1.71× (1.68–1.73) | 53% | 1.29× (1.26–1.33) | 1.68× (1.64–1.72) |

**Best:** mibz2 at 4 threads (861 MB/s); lbzip2 at 4 threads (176 MB/s); pbz2 at 4 threads (280 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.30, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.67 | 0.02 | 0.09 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.5% |
| MTF + RLE2 | 1.14 | 16.8% |
| T-vector build | 1.16 | 17.0% |
| walk (the one each build uses) | 3.24 | 47.7% |
| of which: segment assembly | 0.12 | 1.8% |
| RLE1 | 0.21 | 3.1% |
| CRC | 0.20 | 2.9% |
| **total** | 6.79 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.490 | 1.302 | (in its total) | 2.65× (2.65–2.66) |  |
| walk | 1.234 | 2.683 | (in its total) | 2.18× (2.16–2.21) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.052 | 0.059 | (in its total) | 1.17× (1.09–1.26) |  |
| inverse BWT, total | 1.777 | 4.042 | 3.365 | 2.28× (2.26–2.31) | 1.90× (1.88–1.91) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 8m31s |
| 4 threads | 35m28s ± 5m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m31s ± 3m28s |
| the scanner alone (its serial part) | 13m49s ± 32.9s |
| the workers alone, over 4 | 24m31s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 8.0 ns a byte of bz2, the workers 3.21 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
