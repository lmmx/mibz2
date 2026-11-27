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

The latest record: [`11433a5`](<https://github.com/lmmx/mibz2/commit/11433a5986741fc205db97cea0834beb7dea5f19>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46067899021>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.374 s | 560 | 4.8 | 3.71× (3.70–3.73) | 8.49× (8.45–8.53) | 9.03× (8.98–9.07) | 9.97× (9.93–10.01) | 9.07× (9.01–9.13) | 9.49× (9.42–9.56) | 9.23× (9.17–9.30) | 2.95× (2.93–2.96) | 2.59× (2.57–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.221 s | 950 | 5.5 | 3.26× (3.11–3.42) | 7.22× (6.87–7.60) | 8.61× (8.16–9.09) | 8.55× (8.13–8.99) | 7.71× (7.34–8.10) | 15.60× (14.85–16.40) | 15.19× (14.46–15.95) | 4.84× (4.60–5.10) | 4.26× (4.05–4.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.179 s | 1,170 | 8.8 | 3.53× (3.48–3.58) | 6.53× (6.44–6.62) | 7.63× (7.53–7.73) | 7.59× (7.50–7.67) | 7.38× (7.30–7.45) | 19.77× (19.52–20.01) | 19.24× (19.01–19.48) | 6.14× (6.07–6.20) | 5.39× (5.32–5.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,556 | 1.9 | 0.68× (0.64–0.74) | 3.74× (3.38–4.14) | 5.36× (4.83–5.96) | 4.32× (3.96–4.72) | 7.33× (6.70–8.03) | 3.32× (3.01–3.67) | 3.36× (3.05–3.71) | 1.20× (1.12–1.29) | 1.51× (1.36–1.67) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,282 | 2.4 | 0.57× (0.54–0.61) | 3.32× (3.11–3.55) | 6.40× (6.00–6.84) | 3.84× (3.63–4.06) | 6.45× (6.07–6.85) | 2.94× (2.76–3.14) | 2.98× (2.80–3.17) | 1.07× (1.01–1.12) | 1.34× (1.25–1.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,242 | 2.4 | 0.57× (0.54–0.59) | 3.24× (3.08–3.40) | 6.23× (5.84–6.64) | 3.79× (3.63–3.97) | 6.30× (6.02–6.58) | 2.87× (2.74–3.01) | 2.91× (2.77–3.04) | 1.04× (0.99–1.09) | 1.30× (1.24–1.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 668 | 4.2 | 2.72× (2.62–2.83) | 3.46× (3.33–3.59) | 4.03× (3.98–4.07) | 4.43× (4.28–4.59) | 4.84× (4.69–5.00) | 3.27× (3.16–3.38) | 3.01× (2.91–3.12) | 2.13× (2.08–2.19) | 2.14× (2.05–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,010 | 4.9 | 2.33× (2.12–2.57) | 3.34× (2.88–3.88) | 4.27× (3.91–4.67) | 4.03× (3.42–4.76) | 5.12× (4.72–5.54) | 5.00× (4.59–5.44) | 4.60× (4.23–5.01) | 3.26× (3.00–3.54) | 3.26× (2.96–3.59) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,167 | 8.2 | 2.12× (1.93–2.32) | 2.29× (2.12–2.48) | 3.20× (2.95–3.48) | 2.93× (2.65–3.24) | 4.68× (4.33–5.06) | 5.54× (5.10–6.02) | 5.10× (4.70–5.54) | 3.61× (3.35–3.89) | 3.62× (3.36–3.89) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.104 s | 77 | 35.7 | 1.50× (1.48–1.52) | 3.39× (3.34–3.44) | 3.08× (3.00–3.16) | 3.17× (3.12–3.22) | 3.47× (3.42–3.52) | 3.83× (3.77–3.89) | 3.91× (3.85–3.97) | 1.38× (1.36–1.39) | 1.36× (1.34–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 37.8 | 1.40× (1.29–1.52) | 3.15× (2.88–3.45) | 3.05× (2.84–3.28) | 3.72× (3.43–4.02) | 3.40× (3.13–3.69) | 6.31× (5.85–6.82) | 6.45× (5.97–6.97) | 2.27× (2.10–2.46) | 2.25× (2.08–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.048 s | 167 | 50.6 | 1.60× (1.54–1.66) | 3.13× (3.03–3.23) | 3.11× (2.96–3.26) | 3.78× (3.65–3.92) | 3.68× (3.58–3.78) | 8.33× (8.12–8.55) | 8.51× (8.29–8.73) | 3.00× (2.93–3.07) | 2.97× (2.89–3.05) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 33.1 | 1.55× (1.54–1.56) | 3.77× (3.74–3.79) | 3.19× (3.17–3.21) | 3.36× (3.33–3.38) | 3.69× (3.65–3.74) | 4.51× (4.47–4.55) | 4.77× (4.73–4.80) | 1.60× (1.59–1.60) | 1.66× (1.64–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.066 s | 145 | 35.6 | 1.46× (1.42–1.50) | 3.46× (3.36–3.56) | 3.19× (3.09–3.28) | 4.28× (4.19–4.37) | 3.65× (3.55–3.75) | 7.83× (7.64–8.03) | 8.27× (8.07–8.48) | 2.77× (2.71–2.84) | 2.88× (2.81–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.054 s | 179 | 49.4 | 1.65× (1.59–1.70) | 3.45× (3.34–3.57) | 3.15× (3.05–3.26) | 4.20× (4.09–4.31) | 3.68× (3.60–3.76) | 9.76× (9.53–9.99) | 10.30× (10.07–10.54) | 3.45× (3.38–3.52) | 3.59× (3.50–3.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,075 | 2.9 | 0.72× (0.69–0.75) | 3.30× (2.98–3.65) | 4.59× (4.11–5.12) | 3.61× (3.32–3.93) | 7.68× (7.07–8.35) | 2.77× (2.53–3.04) | 2.82× (2.58–3.07) | 1.08× (1.05–1.11) | 1.13× (1.02–1.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 957 | 3.3 | 0.56× (0.53–0.60) | 2.94× (2.74–3.16) | 5.53× (5.35–5.71) | 3.23× (3.06–3.41) | 6.76× (6.43–7.09) | 2.47× (2.31–2.63) | 2.50× (2.36–2.66) | 0.96× (0.93–1.00) | 1.00× (0.93–1.08) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 877 | 3.5 | 0.53× (0.50–0.55) | 2.73× (2.62–2.86) | 5.13× (4.94–5.32) | 3.03× (2.93–3.13) | 6.35× (6.06–6.65) | 2.29× (2.20–2.39) | 2.33× (2.24–2.41) | 0.89× (0.85–0.94) | 0.93× (0.89–0.97) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.055 s | 270 | 10.2 | 2.07× (2.05–2.10) | 4.44× (4.36–4.52) | 5.04× (4.95–5.13) | 5.06× (4.97–5.15) | 5.15× (5.09–5.21) | 5.28× (5.19–5.37) | 5.16× (5.06–5.25) | 1.68× (1.66–1.70) | 1.67× (1.64–1.70) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.035 s | 429 | 12.1 | 1.88× (1.67–2.11) | 3.78× (3.63–3.93) | 5.00× (4.84–5.16) | 5.14× (4.95–5.33) | 4.91× (4.79–5.03) | 8.49× (8.20–8.78) | 8.28× (7.99–8.59) | 2.70× (2.62–2.79) | 2.68× (2.58–2.77) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.030 s | 498 | 18.2 | 1.85× (1.74–1.96) | 3.29× (3.15–3.43) | 4.32× (4.14–4.51) | 4.55× (4.35–4.75) | 4.65× (4.55–4.77) | 9.79× (9.42–10.18) | 9.56× (9.20–9.94) | 3.12× (3.01–3.23) | 3.09× (2.97–3.21) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.70× (3.69–3.72) | 8.45× (8.41–8.49) | 8.99× (8.94–9.03) | 9.94× (9.90–9.98) | 9.04× (8.98–9.10) | 9.45× (9.38–9.53) | 9.20× (9.14–9.27) | 2.94× (2.92–2.95) | 2.57× (2.55–2.58) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.22× (3.21–3.24) | 7.54× (7.50–7.58) | 8.24× (8.20–8.28) | 9.00× (8.96–9.04) | 7.78× (7.75–7.81) | 8.25× (8.20–8.31) | 8.03× (7.98–8.09) | 2.57× (2.55–2.58) | 2.24× (2.23–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.27× (3.22–3.32) | 6.76× (6.70–6.82) | 7.21× (7.16–7.25) | 7.88× (7.83–7.93) | 7.04× (6.99–7.08) | 5.23× (5.20–5.27) | 5.10× (5.06–5.13) | 1.63× (1.61–1.64) | 1.42× (1.41–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.72× (0.67–0.77) | 3.39× (3.08–3.74) | 4.50× (4.11–4.92) | 4.04× (3.68–4.43) | 6.79× (6.19–7.45) | 3.13× (2.83–3.46) | 3.16× (2.86–3.50) | 1.19× (1.11–1.27) | 0.93× (0.86–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.57–0.64) | 2.98× (2.80–3.17) | 5.59× (5.28–5.91) | 3.51× (3.32–3.70) | 5.83× (5.48–6.19) | 2.70× (2.54–2.88) | 2.73× (2.57–2.91) | 1.03× (0.98–1.08) | 0.81× (0.76–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.56–0.62) | 3.32× (3.11–3.54) | 5.57× (5.33–5.83) | 3.53× (3.38–3.69) | 5.78× (5.53–6.04) | 2.69× (2.56–2.82) | 2.72× (2.60–2.85) | 1.02× (0.97–1.07) | 0.80× (0.77–0.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.54–2.75) | 3.20× (3.08–3.32) | 3.72× (3.60–3.85) | 4.29× (4.13–4.45) | 4.68× (4.52–4.84) | 3.18× (3.06–3.29) | 2.93× (2.82–3.04) | 2.09× (2.03–2.14) | 2.00× (1.94–2.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.55× (2.41–2.70) | 3.02× (2.86–3.19) | 3.62× (3.44–3.82) | 3.93× (3.70–4.16) | 4.25× (4.04–4.47) | 2.83× (2.66–3.00) | 2.61× (2.46–2.77) | 1.86× (1.77–1.95) | 1.78× (1.69–1.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.33× (2.23–2.44) | 2.11× (2.03–2.19) | 2.62× (2.52–2.72) | 2.58× (2.45–2.72) | 2.72× (2.62–2.83) | 1.65× (1.59–1.71) | 1.52× (1.47–1.58) | 1.08× (1.04–1.12) | 1.04× (1.00–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.50× (1.48–1.52) | 3.30× (3.24–3.35) | 3.01× (2.94–3.09) | 3.14× (3.09–3.20) | 3.44× (3.39–3.49) | 3.79× (3.73–3.85) | 3.87× (3.80–3.94) | 1.37× (1.35–1.39) | 1.34× (1.32–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.46× (1.45–1.47) | 3.19× (3.17–3.21) | 2.91× (2.89–2.93) | 3.71× (3.67–3.74) | 3.24× (3.21–3.26) | 3.57× (3.54–3.61) | 3.65× (3.61–3.69) | 1.29× (1.28–1.30) | 1.26× (1.25–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.72× (1.68–1.77) | 3.28× (3.22–3.35) | 2.80× (2.74–2.86) | 3.66× (3.60–3.73) | 3.24× (3.16–3.32) | 2.66× (2.60–2.72) | 2.72× (2.66–2.78) | 0.96× (0.94–0.98) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.53–1.55) | 3.69× (3.66–3.72) | 3.11× (3.09–3.13) | 3.33× (3.30–3.36) | 3.66× (3.61–3.71) | 4.47× (4.43–4.51) | 4.72× (4.68–4.77) | 1.59× (1.58–1.60) | 1.58× (1.58–1.59) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.47× (1.44–1.49) | 3.48× (3.41–3.56) | 2.98× (2.92–3.04) | 4.04× (3.96–4.12) | 3.38× (3.31–3.45) | 4.14× (4.04–4.23) | 4.37× (4.27–4.47) | 1.47× (1.44–1.50) | 1.47× (1.44–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.77× (1.72–1.83) | 3.58× (3.45–3.71) | 2.75× (2.69–2.82) | 3.88× (3.76–4.00) | 3.32× (3.25–3.38) | 3.00× (2.94–3.06) | 3.17× (3.10–3.23) | 1.07× (1.05–1.08) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.74–0.78) | 2.85× (2.62–3.10) | 3.85× (3.61–4.10) | 3.30× (3.03–3.59) | 6.87× (6.31–7.48) | 2.56× (2.34–2.81) | 2.59× (2.37–2.83) | 1.07× (1.05–1.10) | 0.85× (0.79–0.92) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.59–0.65) | 2.41× (2.25–2.57) | 4.55× (4.33–4.78) | 2.93× (2.76–3.11) | 5.92× (5.60–6.25) | 2.22× (2.08–2.38) | 2.25× (2.11–2.40) | 0.93× (0.90–0.97) | 0.74× (0.70–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.57) | 2.53× (2.45–2.60) | 4.27× (4.18–4.37) | 2.81× (2.72–2.90) | 5.67× (5.42–5.93) | 2.11× (2.03–2.19) | 2.13× (2.06–2.21) | 0.88× (0.84–0.93) | 0.70× (0.68–0.72) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 2.05× (2.01–2.08) | 4.26× (4.18–4.34) | 4.85× (4.77–4.92) | 4.95× (4.85–5.05) | 5.04× (4.97–5.12) | 5.17× (5.07–5.28) | 5.05× (4.94–5.16) | 1.66× (1.64–1.69) | 1.53× (1.51–1.55) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.81× (1.76–1.85) | 3.73× (3.62–3.85) | 4.44× (4.33–4.54) | 4.69× (4.56–4.82) | 4.33× (4.25–4.42) | 4.43× (4.29–4.57) | 4.32× (4.19–4.46) | 1.42× (1.39–1.46) | 1.31× (1.28–1.34) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.97× (1.88–2.06) | 3.35× (3.21–3.50) | 3.75× (3.61–3.90) | 4.09× (3.92–4.27) | 3.80× (3.65–3.95) | 2.94× (2.82–3.07) | 2.87× (2.76–3.00) | 0.95× (0.91–0.98) | 0.87× (0.84–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.27–0.30) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.06× (11.04–11.08) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.35–0.37) | 0.57× (0.51–0.63) | 0.75× (0.73–0.78) | 0.63× (0.61–0.65) | 5.96× (5.78–6.14) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.37) | 0.56× (0.47–0.66) | 0.64× (0.60–0.67) | 0.57× (0.54–0.61) | 3.74× (3.57–3.93) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.97) | 0.93× (0.93–0.94) | 4.85× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.70–1.70) | 0.66× (0.65–0.67) | 8.01× (7.99–8.02) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.64× (0.59–0.69) | 1.61× (1.54–1.69) | 0.81× (0.77–0.85) | 5.85× (5.59–6.12) | 0.47× (0.45–0.50) | 0.50× (0.48–0.52) | 0.54× (0.51–0.57) | 0.27× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.66× (0.63–0.70) | 0.73× (0.67–0.79) | 1.21× (1.16–1.26) | 0.75× (0.71–0.78) | 3.61× (3.47–3.76) | 0.28× (0.27–0.30) | 0.30× (0.29–0.31) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.38× (0.35–0.42) | 0.91× (0.88–0.93) | 0.72× (0.68–0.75) | 3.12× (3.04–3.22) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.82× (0.79–0.84) | 2.56× (2.52–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.21–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.44–0.49) | 0.37× (0.35–0.39) | 0.83× (0.79–0.86) | 0.70× (0.67–0.72) | 2.81× (2.70–2.93) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.62) | 0.49× (0.46–0.52) | 0.82× (0.80–0.83) | 0.81× (0.78–0.84) | 2.35× (2.31–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.41× (0.41–0.41) | 0.43× (0.43–0.44) | 1.06× (1.06–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.23) | 1.86× (1.85–1.87) | 0.90× (0.90–0.91) | 6.18× (6.15–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.39–1.40) | 0.65× (0.64–0.66) | 6.42× (6.40–6.45) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 35 MB | 0.49× (0.47–0.51) | 0.41× (0.36–0.46) | 1.11× (1.06–1.16) | 0.74× (0.71–0.77) | 3.60× (3.46–3.74) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.34–0.37) | 0.17× (0.17–0.18) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 52 MB | 0.54× (0.52–0.56) | 0.47× (0.43–0.52) | 0.96× (0.94–0.99) | 0.79× (0.76–0.83) | 2.49× (2.42–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 560 | 1.79 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 950 | 2.06 | 1.15× (1.14–1.15) | 82% | 1.02× (1.01–1.03) | 1.00× (1.00–1.00) |
| 4: all 4 CPUs, sharing SMT siblings | 1,170 | 3.24 | 1.81× (1.79–1.82) | 52% | 1.44× (1.43–1.46) | 1.59× (1.57–1.62) |

**Best:** mibz2 at 4 threads (1,170 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (330 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.04, lbzip2 1t 0.04, lbzip2 2t 0.26, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.26, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.53 | 0.02 | 0.09 | 0.179 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2700 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.58 | 7.7% |
| MTF + RLE2 | 1.17 | 15.6% |
| T-vector build | 1.07 | 14.3% |
| walk (the one each build uses) | 4.42 | 58.8% |
| of which: segment assembly | 0.22 | 2.9% |
| RLE1 | 0.17 | 2.3% |
| CRC | 0.10 | 1.3% |
| **total** | 7.50 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.940 | (in its total) | 2.52× (2.52–2.52) |  |
| walk | 1.540 | 2.970 | (in its total) | 1.93× (1.93–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.073 | (in its total) | 0.91× (0.90–0.91) |  |
| inverse BWT, total | 1.994 | 3.984 | 2.037 | 2.00× (2.00–2.00) | 1.02× (1.02–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m28s ± 6m37s |
| 4 threads | 26m09s ± 3m40s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m12s ± 2m43s |
| the scanner alone (its serial part) | 7m16s ± 0.00s |
| the workers alone, over 4 | 19m12s ± 2m43s |

At 4 threads the workers bound it. Per byte: the scanner 4.2 ns a byte of bz2, the workers 2.52 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,220,170 | 14,849,844 | \`3f9b2018b96fd2e0…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 2700.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1018-aws; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
