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

The latest record: [`66d5072`](<https://github.com/lmmx/mibz2/commit/66d5072516af92d320067c26318ec0244be31090>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44608252735>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.292 s | 719 | 6.3 | 2.77× (2.76–2.79) | 7.00× (6.94–7.05) | 7.29× (7.22–7.35) | 8.33× (8.27–8.39) | 7.39× (7.33–7.45) | 7.66× (7.61–7.71) | 7.29× (7.23–7.35) | 2.19× (2.18–2.20) | 2.53× (2.50–2.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.173 s | 1,215 | 7.0 | 2.79× (2.64–2.94) | 6.08× (5.80–6.38) | 7.40× (7.02–7.79) | 7.44× (7.05–7.85) | 6.62× (6.32–6.94) | 12.66× (12.08–13.26) | 12.04× (11.47–12.65) | 3.62× (3.45–3.79) | 4.17× (3.97–4.39) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.147 s | 1,428 | 10.6 | 2.75× (2.72–2.78) | 4.34× (4.26–4.42) | 5.62× (5.45–5.79) | 5.29× (5.18–5.40) | 5.32× (5.17–5.47) | 15.30× (15.04–15.56) | 14.56× (14.30–14.83) | 4.37× (4.29–4.46) | 5.04× (4.95–5.14) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,439 | 3.5 | 0.57× (0.56–0.57) | 3.66× (3.61–3.71) | 4.54× (4.47–4.60) | 3.70× (3.66–3.75) | 6.55× (6.47–6.62) | 2.78× (2.74–2.82) | 2.77× (2.74–2.81) | 1.00× (0.99–1.01) | 2.20× (2.17–2.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,399 | 3.6 | 0.50× (0.49–0.51) | 2.84× (2.79–2.90) | 5.82× (5.71–5.94) | 3.61× (3.51–3.71) | 6.24× (6.14–6.35) | 2.69× (2.63–2.74) | 2.68× (2.63–2.73) | 0.97× (0.95–0.99) | 2.13× (2.09–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,365 | 3.6 | 0.50× (0.49–0.51) | 2.81× (2.76–2.87) | 5.74× (5.63–5.85) | 3.55× (3.49–3.61) | 6.21× (6.08–6.34) | 2.65× (2.60–2.70) | 2.65× (2.60–2.69) | 0.96× (0.95–0.97) | 2.10× (2.06–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,092 | 4.5 | 2.84× (2.78–2.90) | 3.40× (3.34–3.46) | 4.08× (4.00–4.15) | 4.57× (4.48–4.66) | 5.43× (5.32–5.54) | 3.03× (2.95–3.12) | 2.58× (2.53–2.63) | 2.13× (2.10–2.17) | 3.39× (3.33–3.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,446 | 5.8 | 2.10× (2.00–2.22) | 3.01× (2.82–3.20) | 4.01× (3.70–4.35) | 3.62× (3.36–3.90) | 5.63× (5.26–6.02) | 4.00× (3.71–4.32) | 3.40× (3.19–3.63) | 2.82× (2.64–3.01) | 4.47× (4.20–4.76) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,221 | 11.4 | 1.61× (1.54–1.70) | 1.94× (1.86–2.03) | 2.75× (2.62–2.88) | 2.02× (1.90–2.14) | 4.19× (4.02–4.36) | 3.35× (3.16–3.55) | 2.85× (2.72–2.99) | 2.36× (2.25–2.47) | 3.74× (3.58–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.076 s | 106 | 43.6 | 1.41× (1.40–1.42) | 3.22× (3.20–3.24) | 2.85× (2.82–2.88) | 2.83× (2.82–2.85) | 3.19× (3.17–3.22) | 3.59× (3.57–3.61) | 3.63× (3.61–3.65) | 1.36× (1.36–1.37) | 1.61× (1.60–1.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.048 s | 165 | 50.0 | 1.28× (1.24–1.31) | 2.84× (2.74–2.95) | 2.64× (2.53–2.75) | 3.36× (3.22–3.50) | 3.15× (3.07–3.24) | 5.58× (5.43–5.73) | 5.64× (5.48–5.80) | 2.11× (2.06–2.17) | 2.49× (2.43–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.040 s | 199 | 67.9 | 1.27× (1.23–1.31) | 2.45× (2.37–2.53) | 2.36× (2.28–2.45) | 2.83× (2.74–2.93) | 3.13× (3.05–3.22) | 6.66× (6.47–6.87) | 6.74× (6.54–6.94) | 2.53× (2.46–2.60) | 2.98× (2.89–3.07) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.080 s | 120 | 38.4 | 1.51× (1.50–1.51) | 3.80× (3.79–3.82) | 3.12× (3.08–3.16) | 3.16× (3.14–3.18) | 3.58× (3.57–3.59) | 4.51× (4.49–4.53) | 4.84× (4.83–4.86) | 1.63× (1.62–1.63) | 1.90× (1.89–1.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.051 s | 187 | 45.4 | 1.31× (1.27–1.36) | 3.22× (3.12–3.32) | 2.81× (2.73–2.90) | 3.94× (3.77–4.11) | 3.37× (3.29–3.45) | 6.97× (6.80–7.14) | 7.48× (7.30–7.66) | 2.51× (2.45–2.58) | 2.93× (2.86–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.045 s | 214 | 62.5 | 1.35× (1.31–1.40) | 2.63× (2.53–2.73) | 2.52× (2.40–2.64) | 3.34× (3.24–3.45) | 3.21× (3.10–3.34) | 8.17× (7.96–8.39) | 8.77× (8.55–9.00) | 2.95× (2.87–3.03) | 3.43× (3.34–3.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,087 | 4.8 | 0.62× (0.61–0.63) | 3.32× (3.26–3.38) | 4.96× (4.32–5.70) | 3.14× (3.08–3.20) | 7.20× (7.10–7.30) | 2.33× (2.29–2.38) | 2.36× (2.31–2.40) | 0.99× (0.97–1.00) | 2.22× (2.17–2.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,015 | 5.2 | 0.50× (0.49–0.50) | 3.13× (3.09–3.18) | 5.00× (4.68–5.36) | 3.05× (2.90–3.22) | 6.69× (6.57–6.82) | 2.20× (2.17–2.23) | 2.22× (2.19–2.25) | 0.93× (0.92–0.94) | 2.09× (2.06–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 995 | 5.2 | 0.49× (0.48–0.50) | 3.05× (2.98–3.13) | 5.07× (4.81–5.34) | 2.94× (2.87–3.00) | 6.65× (6.45–6.85) | 2.14× (2.09–2.19) | 2.16× (2.11–2.21) | 0.90× (0.88–0.93) | 2.03× (1.98–2.08) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 0.043 s | 309 | 15.2 | 1.71× (1.68–1.73) | 4.23× (4.16–4.29) | 4.50× (4.43–4.57) | 4.68× (4.62–4.75) | 4.95× (4.87–5.02) | 5.02× (4.94–5.10) | 4.83× (4.75–4.91) | 1.50× (1.47–1.52) | 1.66× (1.60–1.73) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 0.032 s | 419 | 19.1 | 1.36× (1.29–1.44) | 3.33× (3.15–3.52) | 3.89× (3.72–4.07) | 4.16× (4.00–4.33) | 4.34× (4.16–4.52) | 6.88× (6.61–7.15) | 6.62× (6.37–6.89) | 2.05× (1.98–2.13) | 2.28× (2.18–2.39) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 0.030 s | 446 | 29.2 | 1.24× (1.21–1.27) | 2.42× (2.34–2.50) | 3.02× (2.90–3.14) | 3.07× (3.01–3.13) | 3.70× (3.64–3.77) | 7.26× (7.13–7.39) | 6.99× (6.87–7.12) | 2.17× (2.13–2.21) | 2.41× (2.31–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.76× (2.75–2.78) | 6.95× (6.89–7.00) | 7.24× (7.17–7.30) | 8.29× (8.23–8.35) | 7.35× (7.29–7.41) | 7.62× (7.57–7.68) | 7.25× (7.19–7.32) | 2.18× (2.17–2.20) | 2.50× (2.48–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.57× (2.44–2.70) | 6.36× (6.08–6.64) | 6.72× (6.43–7.03) | 7.80× (7.46–8.14) | 6.62× (6.32–6.93) | 6.76× (6.46–7.07) | 6.43× (6.15–6.73) | 1.94× (1.85–2.02) | 2.22× (2.12–2.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.75× (2.70–2.80) | 5.03× (4.91–5.15) | 5.49× (5.38–5.59) | 6.13× (6.01–6.25) | 5.32× (5.21–5.43) | 4.57× (4.47–4.67) | 4.35× (4.25–4.45) | 1.31× (1.28–1.34) | 1.50× (1.47–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.06× (3.00–3.11) | 3.86× (3.80–3.92) | 3.45× (3.39–3.51) | 6.03× (5.91–6.15) | 2.61× (2.56–2.67) | 2.60× (2.56–2.65) | 1.00× (1.00–1.01) | 1.83× (1.79–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.77× (2.72–2.82) | 4.91× (4.83–4.99) | 3.35× (3.26–3.43) | 5.69× (5.59–5.79) | 2.50× (2.44–2.55) | 2.49× (2.44–2.53) | 0.96× (0.94–0.98) | 1.75× (1.71–1.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 2.98× (2.93–3.02) | 4.95× (4.89–5.01) | 3.35× (3.30–3.39) | 5.74× (5.64–5.85) | 2.51× (2.47–2.56) | 2.50× (2.47–2.54) | 0.97× (0.96–0.98) | 1.76× (1.73–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.66–2.77) | 3.17× (3.12–3.24) | 3.77× (3.71–3.82) | 4.33× (4.24–4.41) | 5.12× (5.02–5.22) | 2.90× (2.81–2.99) | 2.47× (2.42–2.52) | 2.06× (2.02–2.09) | 2.78× (2.73–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.19–2.32) | 2.63× (2.56–2.70) | 3.17× (3.09–3.26) | 3.53× (3.43–3.63) | 4.14× (4.04–4.23) | 2.23× (2.15–2.31) | 1.90× (1.85–1.95) | 1.58× (1.55–1.62) | 2.14× (2.09–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.89× (1.80–1.99) | 1.71× (1.61–1.83) | 2.06× (1.97–2.16) | 2.06× (1.96–2.17) | 2.29× (2.18–2.40) | 1.11× (1.05–1.17) | 0.94× (0.90–0.99) | 0.79× (0.75–0.83) | 1.06× (1.01–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.39–1.41) | 3.09× (3.08–3.11) | 2.79× (2.77–2.81) | 2.80× (2.78–2.82) | 3.16× (3.13–3.18) | 3.55× (3.53–3.57) | 3.58× (3.56–3.61) | 1.36× (1.35–1.36) | 1.50× (1.49–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.81× (2.79–2.84) | 2.45× (2.43–2.47) | 3.22× (3.18–3.25) | 2.80× (2.78–2.82) | 3.09× (3.06–3.12) | 3.12× (3.09–3.15) | 1.18× (1.17–1.19) | 1.31× (1.29–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.35–1.44) | 2.59× (2.55–2.63) | 2.16× (2.12–2.19) | 2.83× (2.78–2.87) | 2.65× (2.60–2.70) | 2.29× (2.25–2.32) | 2.31× (2.28–2.34) | 0.87× (0.86–0.89) | 0.97× (0.95–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.49–1.51) | 3.68× (3.66–3.70) | 3.03× (3.01–3.04) | 3.12× (3.10–3.14) | 3.54× (3.52–3.56) | 4.45× (4.43–4.48) | 4.78× (4.75–4.80) | 1.62× (1.61–1.62) | 1.81× (1.79–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.31–1.37) | 3.24× (3.18–3.29) | 2.60× (2.56–2.64) | 3.62× (3.56–3.68) | 3.02× (2.98–3.07) | 3.76× (3.70–3.83) | 4.04× (3.97–4.11) | 1.37× (1.35–1.39) | 1.53× (1.50–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.54–1.60) | 2.98× (2.91–3.05) | 2.25× (2.21–2.29) | 3.22× (3.14–3.31) | 2.91× (2.85–2.98) | 2.76× (2.71–2.82) | 2.96× (2.90–3.02) | 1.00× (0.98–1.02) | 1.12× (1.10–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.54× (2.48–2.61) | 3.63× (3.07–4.29) | 2.87× (2.79–2.95) | 6.40× (6.24–6.56) | 2.18× (2.12–2.24) | 2.19× (2.14–2.24) | 0.99× (0.97–1.01) | 1.51× (1.47–1.55) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.57–0.58) | 2.30× (2.26–2.34) | 3.94× (3.88–4.00) | 2.80× (2.64–2.96) | 5.85× (5.73–5.97) | 2.01× (1.96–2.05) | 2.01× (1.98–2.05) | 0.91× (0.89–0.93) | 1.39× (1.36–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.56) | 2.43× (2.38–2.48) | 3.92× (3.86–3.99) | 2.77× (2.72–2.81) | 5.97× (5.82–6.14) | 2.01× (1.97–2.06) | 2.02× (1.98–2.06) | 0.92× (0.90–0.93) | 1.40× (1.36–1.43) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 1.68× (1.66–1.71) | 3.99× (3.93–4.05) | 4.30× (4.23–4.37) | 4.57× (4.51–4.64) | 4.83× (4.75–4.90) | 4.90× (4.82–4.98) | 4.71× (4.64–4.79) | 1.48× (1.46–1.51) | 1.62× (1.59–1.64) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 1.42× (1.39–1.44) | 3.30× (3.25–3.35) | 3.63× (3.59–3.68) | 4.18× (4.12–4.24) | 3.89× (3.83–3.95) | 3.87× (3.81–3.93) | 3.73× (3.67–3.78) | 1.17× (1.16–1.19) | 1.28× (1.26–1.30) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 1.44× (1.38–1.51) | 2.60× (2.54–2.66) | 2.86× (2.79–2.93) | 3.29× (3.21–3.38) | 3.09× (3.01–3.17) | 2.53× (2.46–2.60) | 2.44× (2.37–2.50) | 0.77× (0.75–0.79) | 0.83× (0.81–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.79× (0.78–0.81) | 10.95× (10.94–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.45× (0.37–0.56) | 0.66× (0.63–0.68) | 0.56× (0.54–0.59) | 5.25× (5.07–5.44) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.31–0.34) | 0.50× (0.42–0.59) | 0.58× (0.55–0.60) | 0.51× (0.49–0.54) | 3.38× (3.24–3.53) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.22) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.35–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.76) | 0.83× (0.83–0.84) | 4.36× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.45× (0.43–0.46) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.92–8.00) | 0.65× (0.64–0.65) | 0.68× (0.68–0.68) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.63–0.70) | 0.59× (0.55–0.64) | 1.56× (1.48–1.64) | 0.78× (0.74–0.82) | 5.66× (5.37–5.96) | 0.46× (0.44–0.48) | 0.48× (0.46–0.51) | 0.52× (0.50–0.55) | 0.26× (0.24–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.58–0.65) | 0.63× (0.56–0.70) | 1.08× (1.04–1.13) | 0.67× (0.64–0.71) | 3.17× (3.01–3.34) | 0.25× (0.24–0.26) | 0.26× (0.25–0.27) | 0.28× (0.27–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.72–0.73) | 5.16× (5.14–5.17) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.40× (0.36–0.44) | 0.92× (0.88–0.95) | 0.73× (0.70–0.76) | 3.17× (3.04–3.31) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.83× (0.81–0.85) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.40) | 0.85× (0.82–0.88) | 0.71× (0.68–0.75) | 2.91× (2.82–3.00) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.46× (0.44–0.48) | 0.80× (0.79–0.81) | 0.82× (0.79–0.84) | 2.32× (2.29–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.22× (0.21–0.23) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.24–5.28) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.66–0.67) | 6.32× (6.31–6.34) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.38× (0.34–0.42) | 1.02× (1.00–1.04) | 0.66× (0.63–0.68) | 3.30× (3.25–3.35) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 56 MB | 0.51× (0.50–0.52) | 0.44× (0.41–0.47) | 0.92× (0.90–0.94) | 0.73× (0.69–0.77) | 2.32× (2.28–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 719 | 1.40 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,215 | 1.55 | 1.13× (1.08–1.18) | 83% | 1.03× (1.02–1.04) | 1.05× (1.02–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,428 | 2.33 | 1.67× (1.63–1.70) | 50% | 1.21× (1.20–1.22) | 1.66× (1.62–1.71) |

**Best:** mibz2 at 4 threads (1,428 MB/s); lbzip2 at 4 threads (331 MB/s); pbz2 at 4 threads (524 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.91, mibz2 4t 1.28, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.48, pbz2 4t 0.54. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 16%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.35 | 0.01 | 0.07 | 0.147 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4528 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.02 | 14.5% |
| MTF + RLE2 | 1.46 | 20.7% |
| T-vector build | 1.32 | 18.9% |
| walk (the one each build uses) | 2.81 | 40.0% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.6% |
| **total** | 7.02 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.287 | 0.680 | (in its total) | 2.37× (2.35–2.38) |  |
| walk | 0.584 | 1.514 | (in its total) | 2.59× (2.58–2.60) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.024 | 0.028 | (in its total) | 1.14× (1.13–1.15) |  |
| inverse BWT, total | 0.895 | 2.221 | 1.248 | 2.48× (2.47–2.49) | 1.39× (1.38–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 42m28s ± 4m53s |
| 4 threads | 21m16s ± 3m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 12m53s ± 1m52s |
| the scanner alone (its serial part) | 7m43s ± 30.2s |
| the workers alone, over 4 | 12m53s ± 1m51s |

At 4 threads the workers bound it. Per byte: the scanner 4.5 ns a byte of bz2, the workers 1.69 ns a byte of output at 4 threads (CPU, summed over them), 1.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,615,664 | 13,288,779 | \`0b9f69f4e27fcf7e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4528.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
