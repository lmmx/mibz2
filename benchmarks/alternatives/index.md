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

The latest record: [`bb45f80`](<https://github.com/lmmx/mibz2/commit/bb45f80ff8365aabf0c91bf4d7f36474443d4b9a>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44958406203>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.304 s | 689 | 3.8 | 2.85× (2.79–2.90) | 7.17× (6.98–7.38) | 7.68× (7.36–8.01) | 8.69× (8.41–8.97) | 7.94× (7.63–8.27) | 8.13× (7.83–8.45) | 7.67× (7.40–7.96) | 2.25× (2.17–2.33) | 2.61× (2.51–2.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.186 s | 1,129 | 4.3 | 2.87× (2.69–3.07) | 6.08× (5.69–6.50) | 7.77× (7.21–8.37) | 7.71× (6.97–8.52) | 6.78× (6.44–7.13) | 13.33× (12.72–13.96) | 12.57× (11.94–13.25) | 3.68× (3.46–3.92) | 4.28× (4.01–4.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.160 s | 1,311 | 6.5 | 2.94× (2.76–3.13) | 4.85× (4.39–5.36) | 6.05× (5.68–6.44) | 5.52× (5.06–6.03) | 5.46× (4.97–6.00) | 15.51× (14.64–16.44) | 14.64× (13.64–15.70) | 4.28× (4.04–4.55) | 4.99× (4.70–5.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,347 | 2.1 | 0.59× (0.53–0.66) | 3.60× (3.40–3.82) | 4.32× (4.09–4.57) | 3.69× (3.55–3.84) | 6.50× (6.18–6.84) | 2.81× (2.64–3.00) | 2.73× (2.63–2.84) | 1.03× (0.96–1.10) | 2.11× (2.03–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,329 | 2.2 | 0.50× (0.45–0.56) | 2.82× (2.50–3.19) | 5.61× (5.11–6.15) | 3.47× (3.12–3.86) | 5.87× (5.36–6.42) | 2.63× (2.39–2.89) | 2.55× (2.36–2.75) | 0.96× (0.87–1.06) | 1.97× (1.80–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,235 | 2.3 | 0.47× (0.42–0.52) | 2.57× (2.27–2.91) | 5.15× (4.34–6.12) | 3.16× (2.74–3.64) | 5.45× (4.69–6.32) | 2.36× (1.98–2.81) | 2.29× (1.95–2.69) | 0.86× (0.73–1.02) | 1.77× (1.52–2.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,037 | 2.7 | 2.87× (2.67–3.08) | 3.67× (3.51–3.85) | 4.33× (4.15–4.51) | 4.59× (4.42–4.76) | 5.50× (5.30–5.71) | 2.97× (2.86–3.08) | 2.51× (2.39–2.64) | 2.07× (1.99–2.16) | 3.17× (3.04–3.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,361 | 3.6 | 1.94× (1.50–2.50) | 2.73× (2.06–3.62) | 3.98× (3.17–4.99) | 3.39× (2.87–3.99) | 4.74× (3.74–6.01) | 3.36× (2.63–4.30) | 2.85× (2.24–3.61) | 2.35× (1.85–2.99) | 3.60× (2.84–4.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,062 | 7.8 | 1.57× (1.29–1.92) | 1.73× (1.47–2.03) | 2.63× (2.15–3.21) | 1.96× (1.63–2.35) | 3.86× (3.24–4.60) | 2.98× (2.49–3.56) | 2.52× (2.15–2.95) | 2.08× (1.79–2.43) | 3.19× (2.72–3.74) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.087 s | 92 | 28.8 | 1.37× (1.29–1.45) | 3.08× (2.91–3.26) | 2.76× (2.60–2.94) | 2.67× (2.49–2.86) | 3.08× (2.78–3.42) | 3.42× (3.15–3.73) | 3.45× (3.22–3.71) | 1.31× (1.25–1.38) | 1.45× (1.40–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 149 | 31.6 | 1.24× (1.20–1.27) | 2.90× (2.71–3.11) | 2.71× (2.50–2.94) | 3.25× (3.05–3.46) | 3.21× (2.91–3.54) | 5.56× (5.20–5.95) | 5.61× (5.26–5.99) | 2.13× (2.04–2.22) | 2.36× (2.29–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 184 | 41.8 | 1.28× (1.20–1.36) | 2.53× (2.43–2.63) | 2.42× (2.30–2.54) | 2.79× (2.63–2.97) | 3.12× (2.96–3.28) | 6.72× (6.25–7.23) | 6.78× (6.33–7.27) | 2.57× (2.46–2.70) | 2.85× (2.74–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.088 s | 110 | 24.1 | 1.42× (1.33–1.51) | 3.81× (3.30–4.41) | 2.93× (2.71–3.16) | 3.03× (2.79–3.29) | 3.48× (3.16–3.84) | 4.32× (4.02–4.65) | 4.54× (4.18–4.93) | 1.54× (1.41–1.68) | 1.79× (1.65–1.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.058 s | 167 | 28.2 | 1.32× (1.15–1.53) | 3.17× (2.77–3.63) | 2.71× (2.39–3.07) | 3.60× (3.24–3.99) | 3.42× (3.07–3.82) | 6.67× (6.06–7.33) | 7.01× (6.40–7.68) | 2.37× (2.17–2.60) | 2.76× (2.57–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.048 s | 200 | 39.4 | 1.35× (1.28–1.43) | 2.80× (2.58–3.03) | 2.58× (2.48–2.69) | 3.34× (3.15–3.54) | 3.19× (3.00–3.40) | 8.26× (7.71–8.85) | 8.69× (8.29–9.11) | 2.94× (2.76–3.13) | 3.41× (3.23–3.61) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,053 | 2.9 | 0.60× (0.57–0.64) | 3.17× (2.95–3.40) | 4.34× (4.07–4.63) | 3.40× (3.08–3.75) | 7.19× (6.61–7.83) | 2.45× (2.22–2.71) | 2.40× (2.19–2.62) | 0.99× (0.91–1.08) | 2.09× (1.96–2.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 3.1 | 0.52× (0.47–0.59) | 2.99× (2.73–3.28) | 5.13× (4.67–5.64) | 2.93× (2.70–3.17) | 6.58× (6.04–7.17) | 2.29× (2.09–2.52) | 2.24× (2.02–2.49) | 0.93× (0.84–1.03) | 1.96× (1.81–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 966 | 3.1 | 0.54× (0.45–0.63) | 3.14× (2.91–3.39) | 5.33× (5.05–5.62) | 2.97× (2.81–3.15) | 6.65× (6.30–7.02) | 2.30× (2.20–2.42) | 2.25× (2.10–2.41) | 0.93× (0.87–1.00) | 1.97× (1.88–2.05) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 0.045 s | 295 | 9.2 | 1.75× (1.68–1.83) | 4.34× (3.97–4.74) | 4.69× (4.43–4.96) | 5.03× (4.60–5.51) | 5.14× (4.91–5.38) | 5.22× (4.94–5.51) | 4.95× (4.80–5.11) | 1.52× (1.46–1.59) | 1.82× (1.73–1.92) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 0.034 s | 395 | 11.9 | 1.44× (1.31–1.58) | 3.32× (3.05–3.61) | 4.13× (3.67–4.65) | 4.49× (4.12–4.90) | 4.51× (4.14–4.92) | 6.80× (6.35–7.29) | 6.45× (5.99–6.95) | 1.99× (1.85–2.13) | 2.37× (2.19–2.57) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 0.032 s | 417 | 18.4 | 1.29× (1.22–1.36) | 2.50× (2.26–2.76) | 3.47× (3.06–3.94) | 3.31× (3.09–3.55) | 4.29× (3.97–4.63) | 7.46× (6.93–8.03) | 7.08× (6.72–7.46) | 2.18× (2.06–2.31) | 2.60× (2.53–2.68) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.84× (2.78–2.89) | 7.12× (6.93–7.32) | 7.63× (7.32–7.95) | 8.64× (8.37–8.92) | 7.90× (7.59–8.23) | 8.09× (7.79–8.41) | 7.64× (7.37–7.92) | 2.24× (2.16–2.32) | 2.59× (2.49–2.70) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.68× (2.54–2.84) | 6.32× (6.01–6.64) | 7.13× (6.69–7.61) | 8.08× (7.47–8.75) | 6.76× (6.58–6.94) | 7.11× (6.83–7.40) | 6.71× (6.37–7.06) | 1.97× (1.88–2.07) | 2.28× (2.15–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.91× (2.73–3.10) | 5.61× (5.09–6.19) | 5.99× (5.49–6.53) | 6.39× (5.77–7.08) | 5.53× (5.03–6.09) | 4.62× (4.32–4.95) | 4.36× (3.99–4.77) | 1.28× (1.19–1.37) | 1.48× (1.38–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.57–0.75) | 3.08× (2.95–3.21) | 3.83× (3.68–3.99) | 3.45× (3.30–3.62) | 5.95× (5.66–6.25) | 2.64× (2.46–2.84) | 2.56× (2.45–2.68) | 1.03× (0.96–1.10) | 1.80× (1.72–1.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.50–0.65) | 2.76× (2.48–3.06) | 4.78× (4.39–5.20) | 3.18× (2.88–3.50) | 5.30× (4.86–5.78) | 2.43× (2.21–2.66) | 2.35× (2.19–2.53) | 0.94× (0.85–1.04) | 1.65× (1.52–1.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.46–0.56) | 2.74× (2.37–3.16) | 4.45× (3.80–5.21) | 2.99× (2.63–3.40) | 5.02× (4.36–5.78) | 2.24× (1.89–2.66) | 2.17× (1.86–2.54) | 0.87× (0.74–1.02) | 1.52× (1.31–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.51–2.94) | 3.16× (3.03–3.30) | 3.82× (3.63–4.01) | 4.28× (4.12–4.46) | 5.13× (4.91–5.36) | 2.79× (2.69–2.90) | 2.38× (2.26–2.51) | 1.98× (1.89–2.07) | 2.73× (2.58–2.89) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.17× (1.85–2.55) | 2.44× (2.19–2.72) | 3.01× (2.80–3.24) | 3.08× (2.67–3.55) | 3.64× (3.16–4.19) | 1.94× (1.65–2.28) | 1.65× (1.42–1.91) | 1.37× (1.19–1.59) | 1.90× (1.61–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.81× (1.52–2.14) | 1.52× (1.29–1.80) | 1.92× (1.64–2.24) | 1.92× (1.64–2.24) | 2.15× (1.81–2.54) | 0.97× (0.83–1.14) | 0.83× (0.72–0.96) | 0.69× (0.60–0.79) | 0.95× (0.82–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.28–1.44) | 2.96× (2.78–3.15) | 2.66× (2.49–2.84) | 2.62× (2.43–2.83) | 3.03× (2.72–3.37) | 3.36× (3.08–3.68) | 3.39× (3.15–3.64) | 1.30× (1.23–1.37) | 1.37× (1.32–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.22–1.30) | 2.82× (2.71–2.94) | 2.44× (2.30–2.60) | 3.13× (3.00–3.27) | 2.86× (2.59–3.16) | 3.08× (2.90–3.26) | 3.10× (2.95–3.26) | 1.19× (1.15–1.23) | 1.25× (1.22–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.36–1.50) | 2.73× (2.56–2.92) | 2.20× (2.11–2.28) | 2.80× (2.64–2.97) | 2.73× (2.50–2.99) | 2.31× (2.15–2.48) | 2.32× (2.18–2.47) | 0.89× (0.84–0.94) | 0.94× (0.90–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.32–1.51) | 3.70× (3.18–4.30) | 2.83× (2.63–3.05) | 2.98× (2.74–3.25) | 3.43× (3.10–3.80) | 4.24× (3.95–4.57) | 4.47× (4.10–4.86) | 1.52× (1.39–1.67) | 1.71× (1.58–1.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.27× (1.20–1.35) | 3.15× (2.76–3.59) | 2.47× (2.19–2.79) | 3.32× (3.02–3.66) | 3.07× (2.77–3.40) | 3.57× (3.24–3.93) | 3.76× (3.44–4.10) | 1.28× (1.17–1.40) | 1.44× (1.33–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.53× (1.45–1.61) | 3.06× (2.75–3.42) | 2.24× (2.10–2.39) | 3.11× (2.93–3.30) | 2.83× (2.69–2.97) | 2.71× (2.54–2.90) | 2.85× (2.67–3.05) | 0.97× (0.90–1.05) | 1.09× (1.02–1.17) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.63–0.69) | 2.58× (2.43–2.74) | 3.33× (3.18–3.49) | 3.05× (2.78–3.35) | 6.29× (5.80–6.82) | 2.30× (2.03–2.60) | 2.20× (2.02–2.41) | 1.00× (0.92–1.08) | 1.47× (1.39–1.55) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.57–0.70) | 2.41× (2.21–2.63) | 4.01× (3.76–4.28) | 2.69× (2.57–2.82) | 5.77× (5.47–6.07) | 2.14× (1.93–2.38) | 2.05× (1.91–2.21) | 0.93× (0.87–0.99) | 1.37× (1.31–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.52–0.69) | 2.76× (2.49–3.07) | 4.18× (3.96–4.41) | 2.82× (2.64–3.02) | 5.98× (5.67–6.31) | 2.21× (2.03–2.39) | 2.11× (1.98–2.26) | 0.96× (0.90–1.02) | 1.41× (1.36–1.46) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 1.73× (1.66–1.80) | 4.15× (3.79–4.54) | 4.47× (4.24–4.71) | 4.89× (4.48–5.35) | 4.99× (4.76–5.23) | 5.07× (4.81–5.35) | 4.81× (4.66–4.96) | 1.52× (1.43–1.61) | 1.67× (1.59–1.74) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 1.46× (1.41–1.50) | 3.44× (3.25–3.63) | 3.87× (3.53–4.23) | 4.57× (4.12–5.06) | 4.23× (3.87–4.62) | 3.89× (3.67–4.14) | 3.69× (3.51–3.88) | 1.16× (1.07–1.27) | 1.28× (1.19–1.38) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 1.46× (1.39–1.53) | 2.75× (2.46–3.08) | 3.24× (2.78–3.77) | 3.43× (3.18–3.69) | 3.68× (3.32–4.07) | 2.49× (2.32–2.68) | 2.36× (2.22–2.52) | 0.75× (0.69–0.81) | 0.82× (0.79–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.34× (0.32–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.79) | 10.94× (10.93–10.96) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.31–0.34) | 0.48× (0.41–0.55) | 0.68× (0.65–0.71) | 0.57× (0.54–0.61) | 5.36× (5.12–5.62) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.31–0.34) | 0.48× (0.37–0.61) | 0.59× (0.56–0.62) | 0.51× (0.48–0.54) | 3.38× (3.22–3.55) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.17× (0.15–0.20) | 1.76× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.76× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.98× (7.95–8.01) | 0.64× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.70) | 0.65× (0.59–0.72) | 1.58× (1.52–1.64) | 0.80× (0.77–0.84) | 5.67× (5.47–5.88) | 0.45× (0.44–0.47) | 0.48× (0.46–0.50) | 0.53× (0.51–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.59–0.65) | 0.67× (0.59–0.76) | 1.11× (1.05–1.17) | 0.69× (0.66–0.73) | 3.27× (3.10–3.45) | 0.25× (0.24–0.27) | 0.27× (0.25–0.28) | 0.29× (0.28–0.31) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.46–0.51) | 0.39× (0.36–0.43) | 0.88× (0.84–0.92) | 0.70× (0.68–0.73) | 3.04× (2.90–3.19) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.78–0.84) | 2.49× (2.43–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.82× (0.81–0.82) | 5.15× (5.15–5.16) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.84× (0.80–0.88) | 0.71× (0.67–0.76) | 2.89× (2.76–3.01) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.58–0.63) | 0.45× (0.44–0.47) | 0.80× (0.78–0.81) | 0.81× (0.78–0.83) | 2.30× (2.25–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.76) | 5.34× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.77× (0.76–0.77) | 5.28× (5.26–5.29) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.30) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.67× (0.66–0.67) | 6.33× (6.32–6.35) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 34 MB | 0.48× (0.48–0.49) | 0.44× (0.40–0.48) | 1.14× (1.11–1.17) | 0.73× (0.72–0.75) | 3.58× (3.54–3.62) | 0.22× (0.22–0.22) | 0.23× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.17) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 55 MB | 0.50× (0.48–0.53) | 0.43× (0.38–0.49) | 0.93× (0.89–0.98) | 0.78× (0.75–0.81) | 2.37× (2.27–2.47) | 0.14× (0.13–0.15) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 689 | 1.46 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,129 | 1.66 | 1.14× (1.09–1.18) | 82% | 1.01× (0.98–1.04) | 1.08× (1.03–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 1,311 | 2.52 | 1.75× (1.64–1.87) | 48% | 1.38× (1.25–1.52) | 1.80× (1.66–1.95) |

**Best:** mibz2 at 4 threads (1,311 MB/s); lbzip2 at 4 threads (279 MB/s); pbz2 at 4 threads (453 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.88, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.44, pbz2 4t 0.60. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 4%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.39 | 0.01 | 0.07 | 0.160 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.60 | 14.1% |
| MTF + RLE2 | 0.84 | 19.5% |
| T-vector build | 0.78 | 18.1% |
| walk (the one each build uses) | 1.78 | 41.6% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.09 | 2.2% |
| CRC | 0.15 | 3.5% |
| **total** | 4.29 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.298 | 0.715 | (in its total) | 2.43× (2.39–2.47) |  |
| walk | 0.646 | 1.627 | (in its total) | 2.53× (2.48–2.59) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.027 | 0.030 | (in its total) | 1.14× (1.08–1.21) |  |
| inverse BWT, total | 0.970 | 2.373 | 1.312 | 2.46× (2.41–2.52) | 1.37× (1.32–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m52s ± 5m13s |
| 4 threads | 23m32s ± 3m26s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m31s ± 2m19s |
| the scanner alone (its serial part) | 8m26s ± 26.3s |
| the workers alone, over 4 | 14m31s ± 2m19s |

At 4 threads the workers bound it. Per byte: the scanner 4.9 ns a byte of bz2, the workers 1.90 ns a byte of output at 4 threads (CPU, summed over them), 1.19 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,640,714 | 13,421,331 | \`8da9d679141c517c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
