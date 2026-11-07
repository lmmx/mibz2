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

The latest record: [`dd93951`](<https://github.com/lmmx/mibz2/commit/dd93951845f2543e035f9a44a77d330adc1d7a8c>) (2026-11-07).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42369687302>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.826 s | 254 | 3.2 | 2.19× (2.16–2.22) | 4.90× (4.76–5.05) | 5.69× (5.46–5.92) | 5.77× (5.56–5.99) | 5.26× (5.02–5.51) | 5.98× (5.70–6.27) | 5.40× (5.20–5.61) | 1.59× (1.55–1.63) | 1.49× (1.47–1.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.456 s | 460 | 3.3 | 2.29× (2.22–2.37) | 4.68× (4.46–4.90) | 6.21× (5.94–6.48) | 5.51× (5.27–5.76) | 5.04× (4.77–5.33) | 10.87× (10.21–11.59) | 9.82× (9.15–10.54) | 2.89× (2.69–3.10) | 2.70× (2.54–2.87) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.396 s | 530 | 5.8 | 2.49× (2.41–2.56) | 3.97× (3.72–4.24) | 5.10× (4.82–5.39) | 4.41× (4.23–4.59) | 4.48× (4.26–4.72) | 12.56× (11.77–13.40) | 11.34× (10.78–11.93) | 3.33× (3.18–3.49) | 3.12× (2.99–3.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 964 | 0.9 | 0.65× (0.61–0.70) | 2.96× (2.89–3.02) | 5.45× (5.09–5.83) | 3.46× (3.39–3.52) | 6.33× (6.21–6.46) | 2.67× (2.62–2.73) | 2.63× (2.58–2.68) | 1.25× (1.17–1.34) | 1.04× (0.95–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 932 | 0.9 | 0.61× (0.58–0.63) | 2.87× (2.80–2.95) | 6.83× (6.56–7.12) | 3.38× (3.29–3.47) | 6.10× (5.96–6.24) | 2.59× (2.52–2.65) | 2.55× (2.48–2.62) | 1.21× (1.13–1.31) | 1.01× (0.91–1.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 920 | 0.9 | 0.62× (0.59–0.65) | 2.86× (2.76–2.96) | 6.74× (6.52–6.98) | 3.36× (3.24–3.48) | 6.15× (5.97–6.33) | 2.57× (2.47–2.67) | 2.53× (2.45–2.62) | 1.20× (1.11–1.31) | 1.00× (0.91–1.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.042 s | 472 | 1.8 | 2.98× (2.94–3.02) | 2.96× (2.89–3.03) | 4.15× (4.05–4.25) | 3.80× (3.72–3.88) | 4.34× (4.22–4.45) | 3.54× (3.45–3.62) | 2.69× (2.63–2.76) | 2.20× (2.01–2.41) | 2.16× (2.06–2.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.029 s | 683 | 2.2 | 2.41× (2.25–2.58) | 2.56× (2.43–2.70) | 4.14× (3.83–4.48) | 3.17× (3.00–3.34) | 4.65× (4.44–4.87) | 5.20× (4.94–5.48) | 3.96× (3.78–4.15) | 3.24× (3.01–3.48) | 3.18× (2.99–3.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 s | 671 | 4.4 | 2.42× (2.35–2.50) | 1.84× (1.75–1.94) | 3.05× (2.93–3.18) | 2.20× (2.05–2.36) | 3.91× (3.81–4.00) | 5.00× (4.85–5.15) | 3.81× (3.70–3.92) | 3.11× (2.84–3.41) | 3.06× (2.88–3.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.167 s | 48 | 16.9 | 1.27× (1.26–1.28) | 2.68× (2.47–2.90) | 2.54× (2.29–2.82) | 2.53× (2.31–2.77) | 2.75× (2.57–2.96) | 3.13× (2.92–3.36) | 3.09× (2.87–3.34) | 1.25× (1.24–1.26) | 1.24× (1.22–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.097 s | 82 | 18.0 | 1.24× (1.22–1.26) | 2.46× (2.31–2.62) | 2.78× (2.52–3.06) | 3.16× (2.99–3.34) | 2.82× (2.75–2.90) | 5.37× (4.96–5.80) | 5.30× (4.87–5.76) | 2.14× (2.09–2.19) | 2.13× (2.06–2.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.079 s | 102 | 24.2 | 1.48× (1.46–1.51) | 2.81× (2.51–3.15) | 2.71× (2.48–2.96) | 3.34× (3.02–3.69) | 3.36× (3.15–3.58) | 6.64× (6.25–7.05) | 6.55× (6.08–7.06) | 2.65× (2.60–2.70) | 2.63× (2.55–2.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.191 s | 50 | 16.0 | 1.32× (1.31–1.33) | 2.86× (2.73–3.00) | 2.73× (2.55–2.93) | 2.63× (2.48–2.79) | 2.97× (2.83–3.12) | 3.55× (3.37–3.75) | 3.77× (3.60–3.96) | 1.39× (1.38–1.41) | 1.42× (1.40–1.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.108 s | 89 | 17.2 | 1.26× (1.22–1.29) | 2.73× (2.59–2.86) | 2.75× (2.59–2.93) | 3.48× (3.31–3.67) | 2.90× (2.78–3.03) | 6.25× (6.02–6.48) | 6.63× (6.38–6.90) | 2.45× (2.41–2.48) | 2.50× (2.43–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.092 s | 104 | 24.4 | 1.57× (1.53–1.61) | 2.58× (2.39–2.78) | 2.66× (2.40–2.95) | 3.35× (3.08–3.65) | 3.06× (2.79–3.36) | 7.36× (6.97–7.76) | 7.81× (7.52–8.12) | 2.88× (2.85–2.92) | 2.94× (2.88–3.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 717 | 1.3 | 0.69× (0.68–0.70) | 2.92× (2.87–2.97) | 4.98× (4.88–5.07) | 3.03× (2.97–3.09) | 7.10× (6.98–7.23) | 2.35× (2.31–2.39) | 2.35× (2.29–2.40) | 1.06× (1.04–1.08) | 1.48× (1.45–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 1.4 | 0.64× (0.62–0.65) | 2.78× (2.73–2.83) | 6.02× (5.87–6.17) | 2.90× (2.84–2.96) | 6.67× (6.54–6.81) | 2.22× (2.16–2.28) | 2.22× (2.16–2.28) | 1.00× (0.97–1.04) | 1.40× (1.37–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 652 | 1.4 | 0.61× (0.60–0.62) | 2.71× (2.65–2.77) | 5.80× (5.53–6.08) | 2.81× (2.74–2.89) | 6.58× (6.42–6.75) | 2.16× (2.10–2.23) | 2.16× (2.11–2.22) | 0.98× (0.94–1.02) | 1.36× (1.33–1.40) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 0.097 s | 130 | 6.2 | 1.48× (1.47–1.50) | 3.19× (3.08–3.31) | 3.87× (3.68–4.07) | 3.71× (3.56–3.86) | 3.90× (3.76–4.04) | 4.00× (3.82–4.19) | 3.79× (3.62–3.97) | 1.15× (1.12–1.19) | 1.15× (1.13–1.16) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 0.058 s | 219 | 6.9 | 1.40× (1.30–1.50) | 3.11× (2.76–3.50) | 3.81× (3.67–3.95) | 3.81× (3.64–3.99) | 3.87× (3.77–3.97) | 6.64× (6.47–6.80) | 6.29× (6.05–6.54) | 1.91× (1.85–1.97) | 1.90× (1.83–1.98) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 0.050 s | 255 | 10.1 | 1.56× (1.47–1.66) | 2.85× (2.64–3.08) | 3.78× (3.41–4.19) | 3.67× (3.42–3.93) | 4.24× (3.93–4.57) | 7.78× (7.39–8.20) | 7.38× (7.06–7.72) | 2.24× (2.16–2.33) | 2.23× (2.16–2.31) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.19× (2.15–2.22) | 4.89× (4.74–5.04) | 5.67× (5.45–5.90) | 5.76× (5.55–5.98) | 5.25× (5.01–5.50) | 5.97× (5.69–6.26) | 5.39× (5.19–5.60) | 1.59× (1.55–1.62) | 1.48× (1.46–1.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.21× (2.18–2.24) | 4.83× (4.68–5.00) | 5.93× (5.78–6.08) | 5.72× (5.55–5.91) | 5.03× (4.82–5.26) | 5.69× (5.43–5.97) | 5.14× (4.90–5.39) | 1.51× (1.45–1.58) | 1.41× (1.36–1.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.31× (2.23–2.40) | 4.02× (3.76–4.30) | 4.76× (4.49–5.04) | 4.45× (4.27–4.64) | 4.16× (3.93–4.41) | 3.25× (3.04–3.46) | 2.93× (2.79–3.09) | 0.86× (0.82–0.90) | 0.80× (0.77–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.65–0.71) | 2.74× (2.68–2.79) | 4.98× (4.73–5.24) | 3.26× (3.20–3.32) | 5.91× (5.80–6.01) | 2.53× (2.49–2.58) | 2.50× (2.46–2.55) | 1.24× (1.16–1.32) | 0.96× (0.93–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.65× (0.63–0.67) | 2.62× (2.56–2.69) | 6.15× (5.93–6.39) | 3.16× (3.09–3.24) | 5.62× (5.50–5.75) | 2.42× (2.36–2.48) | 2.39× (2.33–2.45) | 1.18× (1.10–1.27) | 0.92× (0.88–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.65× (0.62–0.69) | 3.03× (2.92–3.14) | 6.08× (5.89–6.28) | 3.20× (3.09–3.31) | 5.74× (5.58–5.91) | 2.44× (2.36–2.53) | 2.41× (2.34–2.49) | 1.19× (1.09–1.30) | 0.92× (0.89–0.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.90× (2.86–2.94) | 2.72× (2.64–2.79) | 3.87× (3.77–3.97) | 3.68× (3.61–3.76) | 4.20× (4.09–4.31) | 3.43× (3.35–3.52) | 2.62× (2.56–2.68) | 2.15× (1.97–2.35) | 2.00× (1.90–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.48× (2.39–2.57) | 2.34× (2.26–2.42) | 3.47× (3.33–3.61) | 3.03× (2.93–3.13) | 3.45× (3.34–3.56) | 2.76× (2.66–2.86) | 2.10× (2.04–2.18) | 1.73× (1.58–1.89) | 1.61× (1.53–1.70) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.45× (2.39–2.51) | 1.58× (1.50–1.65) | 2.28× (2.20–2.35) | 1.88× (1.82–1.95) | 2.07× (2.00–2.14) | 1.39× (1.34–1.44) | 1.06× (1.02–1.10) | 0.87× (0.80–0.95) | 0.81× (0.77–0.86) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.27× (1.26–1.28) | 2.62× (2.42–2.84) | 2.50× (2.25–2.78) | 2.52× (2.30–2.75) | 2.74× (2.55–2.94) | 3.11× (2.90–3.34) | 3.07× (2.85–3.31) | 1.25× (1.24–1.26) | 1.21× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.23–1.25) | 2.49× (2.31–2.69) | 2.57× (2.31–2.85) | 2.95× (2.79–3.12) | 2.59× (2.54–2.65) | 2.92× (2.73–3.13) | 2.89× (2.68–3.10) | 1.17× (1.16–1.18) | 1.14× (1.13–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.66× (1.62–1.69) | 3.13× (2.73–3.58) | 2.59× (2.31–2.90) | 3.22× (2.93–3.55) | 3.14× (2.90–3.40) | 2.17× (2.03–2.32) | 2.15× (1.99–2.32) | 0.87× (0.86–0.89) | 0.85× (0.83–0.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.31× (1.30–1.32) | 2.83× (2.70–2.96) | 2.69× (2.51–2.89) | 2.61× (2.46–2.77) | 2.96× (2.82–3.10) | 3.53× (3.34–3.73) | 3.75× (3.57–3.93) | 1.39× (1.37–1.40) | 1.39× (1.37–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.27× (1.26–1.29) | 2.78× (2.66–2.90) | 2.55× (2.38–2.74) | 3.26× (3.09–3.43) | 2.69× (2.56–2.81) | 3.30× (3.13–3.47) | 3.50× (3.35–3.65) | 1.30× (1.29–1.31) | 1.29× (1.28–1.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.68–1.76) | 2.83× (2.64–3.04) | 2.47× (2.20–2.77) | 3.09× (2.85–3.35) | 2.83× (2.57–3.12) | 2.32× (2.19–2.46) | 2.46× (2.34–2.58) | 0.91× (0.89–0.94) | 0.91× (0.89–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 2.42× (2.38–2.46) | 4.12× (4.03–4.20) | 2.80× (2.74–2.85) | 6.39× (6.28–6.51) | 2.19× (2.16–2.22) | 2.19× (2.14–2.23) | 1.05× (1.03–1.07) | 0.88× (0.86–0.89) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.68× (0.67–0.70) | 2.36× (2.21–2.52) | 4.95× (4.87–5.04) | 2.70× (2.65–2.75) | 5.96× (5.84–6.08) | 2.05× (2.00–2.10) | 2.04× (2.00–2.09) | 0.98× (0.94–1.01) | 0.82× (0.80–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.65× (0.64–0.67) | 2.60× (2.49–2.72) | 4.87× (4.76–4.98) | 2.69× (2.63–2.76) | 6.05× (5.90–6.19) | 2.05× (1.99–2.11) | 2.05× (2.00–2.09) | 0.98× (0.94–1.02) | 0.82× (0.79–0.85) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 1.47× (1.46–1.49) | 3.11× (3.00–3.22) | 3.77× (3.58–3.97) | 3.65× (3.51–3.81) | 3.84× (3.71–3.98) | 3.94× (3.77–4.13) | 3.74× (3.57–3.92) | 1.15× (1.12–1.18) | 1.07× (1.06–1.09) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 1.44× (1.36–1.52) | 3.04× (2.84–3.25) | 3.61× (3.49–3.74) | 3.68× (3.53–3.82) | 3.49× (3.38–3.60) | 3.57× (3.44–3.71) | 3.38× (3.23–3.54) | 1.04× (1.01–1.07) | 0.97× (0.95–0.99) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 1.74× (1.65–1.84) | 3.26× (3.01–3.53) | 3.79× (3.57–4.01) | 3.53× (3.32–3.76) | 3.77× (3.55–4.00) | 2.45× (2.36–2.54) | 2.32× (2.23–2.41) | 0.71× (0.69–0.73) | 0.67× (0.65–0.68) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.29× (0.28–0.29) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.19× (12.18–12.21) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.39× (0.38–0.39) | 0.46× (0.41–0.52) | 0.82× (0.81–0.83) | 0.67× (0.66–0.69) | 6.48× (6.40–6.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.41–0.42) | 0.54× (0.50–0.58) | 0.74× (0.74–0.75) | 0.64× (0.62–0.65) | 4.31× (4.28–4.34) | 0.10× (0.10–0.10) | 0.11× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.31) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.42× (4.42–4.43) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.44× (0.43–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.66) | 8.14× (8.12–8.16) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.67) | 0.62× (0.57–0.67) | 1.63× (1.62–1.65) | 0.81× (0.80–0.81) | 5.95× (5.93–5.96) | 0.47× (0.47–0.47) | 0.50× (0.49–0.50) | 0.54× (0.53–0.54) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.64–0.66) | 0.74× (0.69–0.79) | 1.18× (1.16–1.21) | 0.77× (0.75–0.79) | 3.54× (3.49–3.59) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.46–0.51) | 0.38× (0.35–0.43) | 0.89× (0.86–0.93) | 0.68× (0.64–0.73) | 3.05× (2.93–3.17) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.66) | 0.47× (0.45–0.48) | 0.87× (0.85–0.89) | 0.78× (0.73–0.82) | 2.53× (2.46–2.59) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.83× (0.83–0.83) | 5.26× (5.26–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.45× (0.44–0.47) | 0.33× (0.30–0.36) | 0.85× (0.82–0.88) | 0.70× (0.67–0.73) | 2.84× (2.75–2.94) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.53) | 0.84× (0.82–0.85) | 0.83× (0.79–0.86) | 2.38× (2.35–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.44× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.64–1.64) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.42× (5.40–5.43) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.40× (0.40–0.41) | 1.50× (1.49–1.50) | 0.66× (0.66–0.67) | 6.76× (6.75–6.77) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 31 MB | 0.52× (0.51–0.53) | 0.46× (0.41–0.51) | 1.25× (1.22–1.28) | 0.80× (0.79–0.81) | 4.01× (3.96–4.06) | 0.24× (0.23–0.24) | 0.25× (0.25–0.26) | 0.39× (0.38–0.40) | 0.19× (0.19–0.20) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 49 MB | 0.56× (0.53–0.58) | 0.52× (0.47–0.58) | 1.06× (1.02–1.09) | 0.83× (0.79–0.87) | 2.69× (2.59–2.79) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.26) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 254 | 3.95 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 460 | 4.11 | 1.05× (1.01–1.09) | 91% | 1.04× (0.99–1.09) | 1.06× (1.02–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 530 | 7.28 | 1.84× (1.77–1.91) | 52% | 1.51× (1.40–1.63) | 1.94× (1.85–2.05) |

**Best:** mibz2 at 4 threads (530 MB/s); lbzip2 at 4 threads (129 MB/s); pbz2 at 4 threads (212 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.64, mibz2 4t 0.92, lbzip2 1t 0.05, lbzip2 2t 0.15, lbzip2 4t 0.26, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.29. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 5%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.29 | 0.04 | 0.08 | 0.396 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m18s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.29 | 9.7% |
| MTF + RLE2 | 0.43 | 14.1% |
| T-vector build | 0.46 | 15.1% |
| walk (the one each build uses) | 1.73 | 57.1% |
| of which: segment assembly | 0.10 | 3.3% |
| RLE1 | 0.08 | 2.6% |
| CRC | 0.04 | 1.2% |
| **total** | 3.04 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.552 | 1.275 | (in its total) | 2.31× (2.30–2.32) |  |
| walk | 1.933 | 3.111 | (in its total) | 1.61× (1.60–1.62) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.101 | 0.106 | (in its total) | 1.09× (0.99–1.19) |  |
| inverse BWT, total | 2.587 | 4.494 | 2.577 | 1.74× (1.73–1.75) | 1.00× (0.99–1.01) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 2h01m ± 14m43s |
| 4 threads | 57m41s ± 8m17s |
| 4 threads, the sample's overlap gap as a fixed cost | 47m24s ± 6m52s |
| the scanner alone (its serial part) | 16m53s ± 43.7s |
| the workers alone, over 4 | 47m24s ± 6m52s |

At 4 threads the workers bound it. Per byte: the scanner 9.8 ns a byte of bz2, the workers 6.21 ns a byte of output at 4 threads (CPU, summed over them), 3.41 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,065,205 | 12,657,539 | \`96a905131eb63a3a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
