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

The latest record: [`9e5db34`](<https://github.com/lmmx/mibz2/commit/9e5db348d75a56c6600eb3ea7fce861ce5c9d51f>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45963183720>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.309 s | 679 | 6.0 | 2.95× (2.91–2.99) | 7.45× (7.29–7.62) | 7.81× (7.58–8.05) | 8.79× (8.56–9.03) | 7.89× (7.67–8.12) | 8.11× (7.92–8.30) | 7.75× (7.61–7.89) | 2.32× (2.28–2.36) | 2.66× (2.62–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.179 s | 1,170 | 6.7 | 3.02× (2.87–3.18) | 6.78× (6.55–7.01) | 8.03× (7.75–8.32) | 8.21× (7.89–8.54) | 7.23× (6.98–7.50) | 13.84× (13.37–14.33) | 13.22× (12.80–13.66) | 3.96× (3.84–4.08) | 4.54× (4.41–4.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.149 s | 1,404 | 10.2 | 2.97× (2.90–3.05) | 4.82× (4.65–5.00) | 6.12× (5.93–6.32) | 5.85× (5.78–5.92) | 5.90× (5.86–5.95) | 16.84× (16.58–17.11) | 16.09× (15.89–16.30) | 4.82× (4.75–4.89) | 5.53× (5.48–5.58) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,316 | 3.5 | 0.57× (0.56–0.58) | 3.65× (3.46–3.86) | 4.84× (4.54–5.16) | 3.72× (3.65–3.79) | 6.53× (6.37–6.70) | 2.82× (2.75–2.88) | 2.81× (2.76–2.86) | 1.00× (0.98–1.01) | 2.07× (1.95–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,258 | 3.8 | 0.50× (0.49–0.51) | 3.18× (3.09–3.26) | 5.59× (5.36–5.83) | 3.53× (3.46–3.61) | 6.10× (5.97–6.24) | 2.67× (2.61–2.74) | 2.66× (2.60–2.73) | 0.95× (0.92–0.97) | 1.96× (1.86–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,205 | 3.8 | 0.49× (0.48–0.50) | 3.10× (3.01–3.20) | 5.56× (5.42–5.71) | 3.46× (3.38–3.55) | 6.08× (5.88–6.28) | 2.60× (2.53–2.68) | 2.60× (2.54–2.66) | 0.92× (0.90–0.94) | 1.91× (1.78–2.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,018 | 4.4 | 2.96× (2.92–2.99) | 3.69× (3.63–3.76) | 4.32× (4.22–4.42) | 4.76× (4.71–4.81) | 5.72× (5.62–5.83) | 3.20× (3.15–3.26) | 2.68× (2.65–2.72) | 2.27× (2.24–2.29) | 3.26× (3.14–3.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,441 | 5.6 | 2.30× (2.12–2.48) | 3.29× (2.74–3.94) | 4.77× (4.20–5.42) | 4.18× (3.50–5.00) | 6.11× (5.62–6.64) | 4.48× (4.12–4.88) | 3.75× (3.48–4.05) | 3.17× (2.95–3.41) | 4.56× (4.15–5.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,214 | 11.0 | 1.82× (1.71–1.94) | 1.94× (1.80–2.10) | 3.18× (2.94–3.45) | 2.23× (2.08–2.39) | 4.61× (4.33–4.89) | 3.76× (3.48–4.06) | 3.15× (2.94–3.37) | 2.66× (2.49–2.84) | 3.82× (3.52–4.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.084 s | 95 | 43.8 | 1.45× (1.44–1.46) | 3.28× (3.24–3.31) | 3.00× (2.94–3.07) | 2.91× (2.89–2.93) | 3.27× (3.23–3.31) | 3.70× (3.65–3.75) | 3.71× (3.67–3.75) | 1.40× (1.39–1.41) | 1.59× (1.56–1.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 49.4 | 1.31× (1.16–1.47) | 3.05× (2.82–3.30) | 2.70× (2.39–3.04) | 3.32× (2.97–3.73) | 3.13× (2.82–3.47) | 5.63× (5.05–6.28) | 5.65× (5.08–6.28) | 2.14× (1.92–2.37) | 2.41× (2.17–2.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 192 | 63.7 | 1.42× (1.38–1.46) | 2.68× (2.61–2.75) | 2.61× (2.51–2.71) | 3.10× (2.99–3.21) | 3.41× (3.31–3.51) | 7.44× (7.21–7.67) | 7.46× (7.27–7.65) | 2.82× (2.75–2.89) | 3.19× (3.09–3.28) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 37.7 | 1.57× (1.55–1.58) | 3.96× (3.91–4.01) | 3.25× (3.22–3.28) | 3.30× (3.26–3.35) | 3.76× (3.70–3.81) | 4.71× (4.67–4.76) | 5.05× (4.98–5.12) | 1.70× (1.67–1.74) | 1.93× (1.90–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 175 | 44.0 | 1.43× (1.21–1.68) | 3.38× (3.08–3.71) | 2.86× (2.66–3.08) | 3.86× (3.57–4.17) | 3.39× (3.18–3.62) | 7.14× (6.68–7.63) | 7.65× (7.15–8.18) | 2.58× (2.43–2.75) | 2.92× (2.74–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 213 | 59.1 | 1.51× (1.48–1.55) | 2.88× (2.78–2.99) | 2.79× (2.70–2.87) | 3.70× (3.56–3.84) | 3.45× (3.39–3.52) | 9.10× (8.87–9.33) | 9.75× (9.46–10.05) | 3.29× (3.22–3.37) | 3.72× (3.64–3.80) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 995 | 4.8 | 0.63× (0.62–0.64) | 3.06× (2.99–3.13) | 4.19× (4.08–4.30) | 3.22× (3.17–3.27) | 7.32× (7.04–7.61) | 2.38× (2.33–2.42) | 2.41× (2.36–2.47) | 1.00× (0.98–1.03) | 2.05× (2.00–2.09) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 918 | 5.5 | 0.46× (0.40–0.54) | 2.63× (2.26–3.07) | 4.58× (3.91–5.37) | 2.78× (2.35–3.28) | 6.35× (5.28–7.63) | 2.04× (1.74–2.41) | 2.08× (1.76–2.45) | 0.86× (0.73–1.02) | 1.76× (1.51–2.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 897 | 5.4 | 0.49× (0.48–0.50) | 2.77× (2.71–2.83) | 4.72× (4.62–4.84) | 2.91× (2.87–2.95) | 6.44× (6.37–6.50) | 2.15× (2.12–2.18) | 2.18× (2.14–2.23) | 0.91× (0.89–0.92) | 1.85× (1.81–1.89) | yes |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 1 | 0.036 s | 336 | 12.8 | 1.92× (1.90–1.94) | 4.45× (4.39–4.51) | 4.88× (4.79–4.98) | 4.99× (4.93–5.05) | 5.43× (5.36–5.50) | 5.07× (4.97–5.17) | 4.89× (4.83–4.96) | 1.65× (1.63–1.67) | 1.98× (1.95–2.02) | yes |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 2 | 0.028 s | 431 | 16.7 | 1.51× (1.40–1.63) | 3.27× (2.96–3.61) | 3.88× (3.49–4.31) | 4.09× (3.76–4.44) | 4.55× (4.15–4.99) | 6.48× (5.93–7.09) | 6.26× (5.69–6.88) | 2.11× (1.91–2.32) | 2.53× (2.30–2.79) | yes |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 4 | 0.023 s | 524 | 25.5 | 1.49× (1.44–1.55) | 2.70× (2.60–2.82) | 3.41× (3.27–3.55) | 3.99× (3.87–4.11) | 4.45× (4.32–4.58) | 7.87× (7.59–8.16) | 7.60× (7.38–7.83) | 2.56× (2.49–2.63) | 3.08× (2.97–3.19) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.90–2.98) | 7.41× (7.25–7.57) | 7.75× (7.53–7.99) | 8.75× (8.52–8.99) | 7.85× (7.64–8.08) | 8.07× (7.88–8.26) | 7.71× (7.57–7.85) | 2.31× (2.27–2.36) | 2.63× (2.59–2.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.74–2.81) | 6.91× (6.82–7.00) | 7.43× (7.25–7.62) | 8.47× (8.36–8.58) | 7.20× (7.10–7.31) | 7.35× (7.25–7.45) | 7.02× (6.94–7.10) | 2.11× (2.08–2.13) | 2.40× (2.37–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.82× (2.73–2.91) | 5.30× (5.16–5.44) | 5.70× (5.61–5.79) | 6.43× (6.35–6.51) | 5.65× (5.59–5.71) | 4.75× (4.66–4.84) | 4.54× (4.48–4.60) | 1.36× (1.34–1.38) | 1.55× (1.53–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.10× (3.05–3.15) | 4.10× (3.80–4.43) | 3.47× (3.40–3.55) | 6.02× (5.87–6.18) | 2.66× (2.59–2.72) | 2.64× (2.59–2.70) | 1.00× (0.99–1.01) | 1.87× (1.84–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.55) | 2.68× (2.63–2.73) | 4.68× (4.59–4.76) | 3.19× (3.12–3.25) | 5.40× (5.29–5.52) | 2.41× (2.36–2.47) | 2.40× (2.35–2.45) | 0.91× (0.88–0.93) | 1.70× (1.65–1.74) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.54) | 2.87× (2.79–2.94) | 4.67× (4.58–4.77) | 3.18× (3.10–3.27) | 5.47× (5.29–5.66) | 2.40× (2.34–2.47) | 2.39× (2.33–2.45) | 0.90× (0.88–0.93) | 1.69× (1.66–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.80× (2.76–2.85) | 3.28× (3.23–3.33) | 3.92× (3.84–4.00) | 4.47× (4.40–4.54) | 5.35× (5.25–5.46) | 3.04× (2.98–3.09) | 2.55× (2.50–2.60) | 2.17× (2.14–2.20) | 2.92× (2.89–2.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.29–2.44) | 2.81× (2.73–2.89) | 3.32× (3.24–3.41) | 3.67× (3.55–3.79) | 4.31× (4.18–4.45) | 2.37× (2.28–2.46) | 1.99× (1.94–2.04) | 1.69× (1.65–1.73) | 2.28× (2.21–2.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.07× (1.98–2.17) | 1.85× (1.75–1.96) | 2.25× (2.17–2.33) | 2.31× (2.22–2.40) | 2.53× (2.42–2.65) | 1.22× (1.15–1.29) | 1.03× (0.99–1.07) | 0.87× (0.84–0.91) | 1.17× (1.12–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.43–1.45) | 3.18× (3.16–3.20) | 2.91× (2.85–2.97) | 2.88× (2.86–2.89) | 3.23× (3.19–3.27) | 3.65× (3.60–3.70) | 3.66× (3.62–3.70) | 1.40× (1.39–1.40) | 1.52× (1.51–1.54) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.34× (1.32–1.35) | 2.91× (2.88–2.94) | 2.54× (2.50–2.57) | 3.32× (3.21–3.43) | 2.87× (2.84–2.91) | 3.22× (3.18–3.26) | 3.23× (3.17–3.29) | 1.23× (1.22–1.25) | 1.34× (1.34–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.42–1.53) | 2.78× (2.72–2.85) | 2.32× (2.28–2.37) | 3.01× (2.94–3.09) | 2.84× (2.78–2.91) | 2.47× (2.41–2.52) | 2.47× (2.41–2.54) | 0.94× (0.92–0.97) | 1.03× (1.00–1.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.56× (1.54–1.57) | 3.83× (3.79–3.88) | 3.16× (3.12–3.19) | 3.26× (3.21–3.31) | 3.70× (3.65–3.76) | 4.64× (4.59–4.69) | 4.97× (4.90–5.05) | 1.69× (1.66–1.73) | 1.85× (1.83–1.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.41× (1.37–1.44) | 3.40× (3.31–3.50) | 2.73× (2.64–2.82) | 3.75× (3.64–3.86) | 3.16× (3.07–3.25) | 3.96× (3.82–4.10) | 4.24× (4.11–4.38) | 1.45× (1.40–1.50) | 1.58× (1.52–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.59–1.72) | 3.21× (3.16–3.26) | 2.41× (2.38–2.45) | 3.46× (3.39–3.53) | 3.07× (3.01–3.13) | 2.97× (2.90–3.03) | 3.18× (3.11–3.25) | 1.08× (1.06–1.11) | 1.18× (1.16–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.70) | 2.60× (2.54–2.66) | 3.44× (3.38–3.49) | 2.91× (2.85–2.98) | 6.45× (6.18–6.73) | 2.20× (2.15–2.25) | 2.22× (2.16–2.29) | 1.00× (0.97–1.03) | 1.53× (1.50–1.56) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.52× (0.44–0.61) | 2.06× (1.74–2.44) | 3.52× (2.96–4.20) | 2.45× (2.06–2.91) | 5.33× (4.39–6.46) | 1.79× (1.51–2.12) | 1.81× (1.52–2.15) | 0.81× (0.68–0.97) | 1.25× (1.06–1.48) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.36× (2.33–2.40) | 3.75× (3.71–3.79) | 2.65× (2.61–2.69) | 5.59× (5.54–5.65) | 1.95× (1.92–1.98) | 1.97× (1.92–2.02) | 0.89× (0.86–0.91) | 1.36× (1.34–1.38) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 1 | 1.88× (1.85–1.90) | 4.17× (4.12–4.23) | 4.53× (4.47–4.59) | 4.81× (4.75–4.88) | 5.24× (5.17–5.31) | 4.89× (4.79–4.99) | 4.72× (4.66–4.79) | 1.62× (1.60–1.64) | 1.80× (1.78–1.83) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 2 | 1.51× (1.50–1.53) | 3.21× (3.11–3.30) | 3.56× (3.47–3.65) | 4.08× (3.95–4.21) | 3.99× (3.86–4.12) | 3.67× (3.56–3.78) | 3.54× (3.46–3.63) | 1.21× (1.18–1.25) | 1.35× (1.32–1.39) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 4 | 1.61× (1.56–1.65) | 2.68× (2.63–2.74) | 2.97× (2.94–3.01) | 3.27× (3.16–3.38) | 3.23× (3.14–3.32) | 2.44× (2.35–2.54) | 2.36× (2.27–2.44) | 0.81× (0.78–0.83) | 0.90× (0.87–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.32× (0.31–0.34) | 0.98× (0.98–0.98) | 0.80× (0.79–0.81) | 11.06× (11.04–11.07) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.50× (0.42–0.60) | 0.72× (0.70–0.74) | 0.60× (0.59–0.62) | 5.73× (5.56–5.92) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 87 MB | 0.35× (0.34–0.36) | 0.54× (0.46–0.65) | 0.62× (0.60–0.64) | 0.55× (0.52–0.57) | 3.63× (3.51–3.77) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.84–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.04) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.69–1.70) | 0.66× (0.66–0.67) | 7.99× (7.96–8.01) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.71) | 0.64× (0.59–0.70) | 1.62× (1.57–1.67) | 0.81× (0.79–0.83) | 5.87× (5.70–6.04) | 0.48× (0.46–0.49) | 0.50× (0.48–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.67× (0.63–0.71) | 0.72× (0.67–0.78) | 1.20× (1.15–1.26) | 0.74× (0.70–0.77) | 3.48× (3.32–3.64) | 0.27× (0.26–0.29) | 0.28× (0.27–0.30) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.74) | 5.17× (5.16–5.19) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.41× (0.38–0.44) | 0.89× (0.86–0.92) | 0.70× (0.67–0.73) | 3.08× (2.98–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.46× (0.44–0.48) | 0.87× (0.86–0.89) | 0.82× (0.80–0.84) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.45–0.50) | 0.37× (0.34–0.39) | 0.83× (0.80–0.86) | 0.71× (0.67–0.74) | 2.86× (2.76–2.97) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.46× (0.44–0.49) | 0.80× (0.79–0.81) | 0.83× (0.80–0.85) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.23) | 1.07× (1.06–1.07) | 0.76× (0.75–0.77) | 5.34× (5.31–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.91) | 6.21× (6.17–6.24) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.38) | 1.40× (1.40–1.40) | 0.65× (0.65–0.66) | 6.33× (6.32–6.35) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 2 | 36 MB | 0.48× (0.47–0.49) | 0.38× (0.35–0.41) | 1.09× (1.05–1.13) | 0.69× (0.67–0.71) | 3.50× (3.39–3.60) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (1.85 MB → 12.1 MB, 6.5×) | 4 | 52 MB | 0.55× (0.54–0.56) | 0.46× (0.43–0.51) | 0.98× (0.96–1.00) | 0.77× (0.75–0.79) | 2.45× (2.40–2.50) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 679 | 1.48 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,170 | 1.63 | 1.10× (1.08–1.12) | 85% | 1.02× (1.01–1.04) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,404 | 2.50 | 1.70× (1.68–1.72) | 52% | 1.22× (1.17–1.26) | 1.63× (1.57–1.69) |

**Best:** mibz2 at 4 threads (1,404 MB/s); lbzip2 at 4 threads (295 MB/s); pbz2 at 4 threads (477 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.42, pbz2 1t 0.00, pbz2 2t 0.48, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.39 | 0.01 | 0.07 | 0.149 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4091 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.76 | 11.3% |
| MTF + RLE2 | 1.44 | 21.2% |
| T-vector build | 1.32 | 19.4% |
| walk (the one each build uses) | 2.84 | 41.9% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.15 | 2.3% |
| CRC | 0.26 | 3.8% |
| **total** | 6.77 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.314 | 0.762 | (in its total) | 2.42× (2.40–2.44) |  |
| walk | 0.666 | 1.712 | (in its total) | 2.57× (2.55–2.60) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.032 | (in its total) | 1.20× (1.19–1.22) |  |
| inverse BWT, total | 1.006 | 2.505 | 1.400 | 2.49× (2.47–2.51) | 1.38× (1.37–1.39) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m12s ± 5m28s |
| 4 threads | 21m45s ± 3m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m35s ± 2m07s |
| the scanner alone (its serial part) | 6m59s ± 26.3s |
| the workers alone, over 4 | 14m35s ± 2m07s |

At 4 threads the workers bound it. Per byte: the scanner 4.0 ns a byte of bz2, the workers 1.91 ns a byte of output at 4 threads (CPU, summed over them), 1.25 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,854,772 | 12,126,476 | \`093b980ffe8caf47…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4090.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
