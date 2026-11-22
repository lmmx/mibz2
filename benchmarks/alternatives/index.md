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

The latest record: [`23c1e28`](<https://github.com/lmmx/mibz2/commit/23c1e283483ca465f92e060b1c0028f1d0404378>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44985820388>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.317 s | 662 | 6.6 | 2.79× (2.76–2.82) | 7.12× (6.78–7.47) | 7.40× (7.28–7.52) | 8.47× (8.23–8.72) | 7.63× (7.23–8.06) | 7.66× (7.54–7.78) | 7.38× (7.07–7.69) | 2.20× (2.18–2.22) | 2.54× (2.47–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.204 s | 1,030 | 7.8 | 2.74× (2.53–2.97) | 6.19× (5.57–6.88) | 7.28× (6.84–7.76) | 7.27× (6.81–7.76) | 6.55× (6.09–7.06) | 11.87× (11.16–12.63) | 11.44× (10.55–12.41) | 3.41× (3.24–3.60) | 3.94× (3.70–4.20) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.166 s | 1,260 | 12.0 | 3.06× (2.92–3.21) | 4.50× (4.36–4.65) | 6.36× (5.96–6.79) | 5.60× (5.35–5.87) | 5.68× (5.38–6.00) | 14.56× (14.25–14.87) | 14.03× (13.52–14.56) | 4.19× (4.08–4.30) | 4.83× (4.65–5.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,320 | 3.7 | 0.56× (0.55–0.57) | 3.55× (3.36–3.76) | 4.77× (4.65–4.89) | 3.65× (3.57–3.73) | 6.39× (6.24–6.55) | 2.73× (2.67–2.79) | 2.74× (2.68–2.80) | 1.01× (0.99–1.03) | 2.02× (1.97–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,329 | 3.7 | 0.51× (0.50–0.53) | 3.36× (3.29–3.44) | 5.86× (5.59–6.15) | 3.64× (3.56–3.71) | 6.31× (6.16–6.47) | 2.73× (2.68–2.78) | 2.73× (2.69–2.78) | 1.01× (0.98–1.03) | 2.02× (1.97–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,258 | 3.8 | 0.50× (0.49–0.51) | 3.23× (3.18–3.29) | 5.98× (5.51–6.49) | 3.57× (3.43–3.71) | 6.11× (5.97–6.26) | 2.61× (2.56–2.67) | 2.62× (2.56–2.67) | 0.97× (0.94–0.99) | 1.93× (1.90–1.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,008 | 4.7 | 2.82× (2.75–2.88) | 3.62× (3.53–3.70) | 4.23× (4.12–4.33) | 4.53× (4.47–4.59) | 5.39× (5.22–5.56) | 2.97× (2.90–3.05) | 2.63× (2.57–2.68) | 2.14× (2.10–2.18) | 3.23× (3.07–3.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,225 | 6.7 | 1.85× (1.60–2.14) | 2.44× (2.15–2.77) | 3.78× (3.14–4.56) | 3.06× (2.63–3.57) | 4.60× (3.96–5.34) | 3.33× (2.90–3.81) | 2.94× (2.54–3.41) | 2.40× (2.08–2.77) | 3.62× (3.08–4.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,071 | 13.4 | 1.73× (1.64–1.83) | 1.70× (1.61–1.80) | 2.74× (2.53–2.96) | 1.98× (1.86–2.11) | 3.97× (3.82–4.12) | 3.11× (2.95–3.29) | 2.75× (2.59–2.93) | 2.25× (2.12–2.37) | 3.39× (3.29–3.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.086 s | 93 | 47.8 | 1.36× (1.35–1.38) | 3.05× (2.97–3.14) | 2.73× (2.69–2.78) | 2.66× (2.63–2.69) | 3.01× (2.95–3.07) | 3.37× (3.30–3.44) | 3.42× (3.32–3.52) | 1.33× (1.30–1.35) | 1.53× (1.52–1.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.055 s | 144 | 56.5 | 1.26× (1.17–1.36) | 2.86× (2.62–3.13) | 2.56× (2.43–2.70) | 3.24× (2.96–3.54) | 2.96× (2.81–3.12) | 5.15× (4.99–5.32) | 5.22× (5.06–5.39) | 2.03× (1.94–2.12) | 2.34× (2.23–2.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.046 s | 172 | 74.7 | 1.26× (1.22–1.31) | 2.46× (2.32–2.61) | 2.51× (2.24–2.82) | 2.86× (2.60–3.14) | 3.05× (2.92–3.18) | 6.22× (5.97–6.48) | 6.30× (6.02–6.59) | 2.45× (2.35–2.55) | 2.82× (2.71–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.093 s | 103 | 42.9 | 1.44× (1.42–1.47) | 3.56× (3.48–3.64) | 2.97× (2.87–3.08) | 2.91× (2.83–3.00) | 3.37× (3.30–3.45) | 4.18× (4.08–4.29) | 4.50× (4.39–4.61) | 1.55× (1.51–1.59) | 1.78× (1.71–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.062 s | 156 | 52.0 | 1.27× (1.19–1.35) | 3.02× (2.85–3.21) | 2.65× (2.47–2.84) | 3.54× (3.31–3.78) | 3.10× (2.97–3.24) | 6.32× (6.02–6.63) | 6.80× (6.47–7.15) | 2.34× (2.24–2.44) | 2.69× (2.62–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.051 s | 187 | 70.9 | 1.34× (1.31–1.37) | 2.69× (2.51–2.89) | 2.48× (2.43–2.54) | 3.33× (3.16–3.51) | 3.21× (3.01–3.43) | 7.59× (7.44–7.75) | 8.17× (8.00–8.36) | 2.81× (2.74–2.88) | 3.23× (3.11–3.35) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 995 | 5.2 | 0.61× (0.61–0.62) | 3.15× (2.97–3.35) | 4.20× (4.11–4.28) | 3.16× (3.10–3.21) | 7.05× (6.93–7.17) | 2.33× (2.29–2.37) | 2.37× (2.28–2.45) | 0.98× (0.96–1.00) | 2.05× (2.01–2.09) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 943 | 5.5 | 0.49× (0.47–0.51) | 2.86× (2.74–2.98) | 5.15× (4.81–5.52) | 2.95× (2.85–3.06) | 6.73× (6.35–7.13) | 2.17× (2.08–2.26) | 2.20× (2.09–2.33) | 0.91× (0.87–0.96) | 1.91× (1.83–1.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 922 | 5.4 | 0.49× (0.49–0.50) | 2.84× (2.80–2.89) | 4.84× (4.76–4.92) | 2.94× (2.89–2.99) | 6.55× (6.47–6.63) | 2.15× (2.10–2.20) | 2.18× (2.11–2.26) | 0.91× (0.89–0.93) | 1.89× (1.86–1.92) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 0.048 s | 280 | 16.1 | 1.65× (1.51–1.80) | 3.96× (3.61–4.34) | 4.34× (3.98–4.72) | 4.44× (4.08–4.83) | 4.60× (4.20–5.03) | 4.73× (4.35–5.15) | 4.57× (4.21–4.97) | 1.47× (1.39–1.57) | 1.65× (1.51–1.80) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 0.036 s | 369 | 21.8 | 1.32× (1.14–1.53) | 3.06× (2.76–3.39) | 3.81× (3.35–4.34) | 3.90× (3.33–4.57) | 3.91× (3.40–4.50) | 6.10× (5.37–6.93) | 5.89× (5.13–6.76) | 1.90× (1.65–2.19) | 2.12× (1.86–2.43) | yes |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 0.033 s | 414 | 33.4 | 1.38× (1.32–1.43) | 2.77× (2.50–3.06) | 3.61× (3.26–4.00) | 3.54× (3.14–3.98) | 3.97× (3.80–4.15) | 7.28× (7.18–7.38) | 7.03× (6.86–7.22) | 2.27× (2.21–2.33) | 2.54× (2.47–2.60) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.78× (2.75–2.81) | 7.07× (6.74–7.42) | 7.35× (7.23–7.47) | 8.43× (8.20–8.68) | 7.59× (7.19–8.02) | 7.62× (7.50–7.74) | 7.34× (7.04–7.65) | 2.19× (2.17–2.21) | 2.52× (2.45–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.59× (2.42–2.77) | 6.34× (5.74–6.99) | 6.71× (6.35–7.10) | 7.68× (7.20–8.19) | 6.56× (6.08–7.08) | 6.32× (5.99–6.68) | 6.09× (5.68–6.54) | 1.82× (1.74–1.91) | 2.09× (1.99–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.83–3.12) | 5.04× (4.88–5.19) | 6.07× (5.70–6.46) | 6.28× (5.94–6.63) | 5.59× (5.29–5.90) | 4.19× (4.10–4.28) | 4.04× (3.90–4.18) | 1.21× (1.18–1.24) | 1.38× (1.34–1.43) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.06× (3.00–3.12) | 3.88× (3.80–3.96) | 3.38× (3.31–3.44) | 5.83× (5.70–5.96) | 2.55× (2.50–2.60) | 2.56× (2.51–2.61) | 1.01× (0.99–1.03) | 1.80× (1.77–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.81× (2.75–2.87) | 5.01× (4.88–5.13) | 3.33× (3.26–3.41) | 5.68× (5.53–5.82) | 2.50× (2.46–2.55) | 2.51× (2.46–2.55) | 0.99× (0.97–1.01) | 1.76× (1.72–1.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 2.96× (2.90–3.02) | 5.03× (4.63–5.46) | 3.36× (3.23–3.49) | 5.63× (5.50–5.76) | 2.47× (2.42–2.52) | 2.47× (2.43–2.52) | 0.98× (0.95–1.00) | 1.74× (1.70–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.67× (2.61–2.73) | 3.12× (3.08–3.16) | 3.78× (3.72–3.84) | 4.24× (4.19–4.30) | 5.04× (4.88–5.19) | 2.81× (2.74–2.89) | 2.50× (2.45–2.55) | 2.05× (2.01–2.09) | 2.80× (2.67–2.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.02–2.16) | 2.41× (2.29–2.54) | 3.07× (2.73–3.46) | 3.18× (3.04–3.32) | 3.63× (3.48–3.79) | 1.98× (1.91–2.05) | 1.76× (1.69–1.82) | 1.44× (1.39–1.49) | 1.97× (1.81–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.78–1.95) | 1.62× (1.54–1.70) | 1.98× (1.91–2.06) | 2.00× (1.92–2.08) | 2.16× (2.09–2.24) | 1.01× (0.95–1.07) | 0.89× (0.85–0.94) | 0.73× (0.70–0.77) | 1.00× (0.94–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.37) | 2.96× (2.89–3.04) | 2.65× (2.60–2.70) | 2.63× (2.60–2.65) | 2.97× (2.91–3.03) | 3.33× (3.26–3.40) | 3.37× (3.27–3.47) | 1.32× (1.29–1.35) | 1.44× (1.42–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.24) | 2.68× (2.58–2.79) | 2.31× (2.26–2.36) | 3.06× (2.86–3.27) | 2.60× (2.55–2.64) | 2.82× (2.75–2.89) | 2.86× (2.78–2.93) | 1.12× (1.10–1.14) | 1.22× (1.19–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.36–1.48) | 2.60× (2.47–2.73) | 2.24× (1.97–2.56) | 2.84× (2.61–3.10) | 2.68× (2.55–2.82) | 2.12× (2.04–2.20) | 2.15× (2.06–2.24) | 0.84× (0.81–0.88) | 0.92× (0.88–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.41–1.46) | 3.48× (3.39–3.56) | 2.88× (2.78–2.98) | 2.88× (2.80–2.96) | 3.33× (3.25–3.41) | 4.12× (4.02–4.23) | 4.44× (4.33–4.55) | 1.54× (1.50–1.58) | 1.71× (1.64–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.25× (1.18–1.33) | 3.01× (2.83–3.20) | 2.44× (2.27–2.62) | 3.32× (3.10–3.55) | 2.77× (2.63–2.92) | 3.37× (3.20–3.55) | 3.63× (3.43–3.84) | 1.26× (1.20–1.32) | 1.39× (1.36–1.43) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.49× (1.46–1.52) | 3.05× (2.84–3.26) | 2.22× (2.13–2.32) | 3.19× (2.91–3.49) | 2.94× (2.73–3.17) | 2.50× (2.43–2.56) | 2.69× (2.63–2.75) | 0.93× (0.91–0.95) | 1.03× (1.00–1.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.67) | 2.55× (2.49–2.61) | 3.39× (3.31–3.46) | 2.84× (2.79–2.88) | 6.15× (6.05–6.25) | 2.13× (2.09–2.17) | 2.18× (2.08–2.28) | 0.98× (0.96–1.00) | 1.48× (1.45–1.50) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.55–0.59) | 2.26× (2.19–2.33) | 4.16× (3.85–4.50) | 2.68× (2.59–2.78) | 5.82× (5.48–6.19) | 1.95× (1.87–2.04) | 2.00× (1.88–2.12) | 0.90× (0.86–0.94) | 1.35× (1.30–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.55–0.56) | 2.47× (2.40–2.54) | 3.98× (3.92–4.04) | 2.75× (2.71–2.80) | 5.83× (5.77–5.90) | 2.00× (1.96–2.05) | 2.04× (1.96–2.14) | 0.92× (0.90–0.94) | 1.39× (1.36–1.41) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 1.63× (1.48–1.78) | 3.82× (3.48–4.20) | 4.11× (3.77–4.48) | 4.31× (3.95–4.71) | 4.47× (4.07–4.90) | 4.59× (4.21–5.01) | 4.44× (4.07–4.84) | 1.45× (1.36–1.55) | 1.55× (1.42–1.69) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 1.42× (1.33–1.53) | 3.15× (3.04–3.27) | 3.62× (3.39–3.87) | 3.95× (3.75–4.16) | 3.63× (3.46–3.82) | 3.50× (3.41–3.59) | 3.38× (3.27–3.50) | 1.11× (1.07–1.15) | 1.18× (1.15–1.21) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 1.52× (1.45–1.61) | 2.82× (2.49–3.20) | 3.27× (2.81–3.80) | 3.46× (3.00–3.98) | 3.21× (3.04–3.40) | 2.31× (2.26–2.37) | 2.23× (2.16–2.31) | 0.73× (0.71–0.76) | 0.78× (0.76–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.77–0.80) | 10.96× (10.92–10.99) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.33) | 0.47× (0.39–0.56) | 0.67× (0.64–0.69) | 0.56× (0.54–0.58) | 5.34× (5.14–5.54) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.33–0.34) | 0.55× (0.44–0.68) | 0.60× (0.59–0.62) | 0.53× (0.52–0.54) | 3.45× (3.38–3.53) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.76) | 0.83× (0.83–0.84) | 4.36× (4.36–4.37) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.46) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.93–7.99) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.65× (0.62–0.68) | 0.64× (0.58–0.70) | 1.55× (1.48–1.62) | 0.77× (0.74–0.81) | 5.61× (5.38–5.85) | 0.46× (0.44–0.48) | 0.48× (0.46–0.50) | 0.52× (0.50–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.64× (0.61–0.67) | 0.69× (0.65–0.75) | 1.11× (1.06–1.17) | 0.69× (0.66–0.73) | 3.27× (3.13–3.41) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.13–5.17) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.53) | 0.42× (0.39–0.45) | 0.93× (0.90–0.96) | 0.75× (0.72–0.77) | 3.22× (3.12–3.32) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.83× (0.80–0.85) | 2.56× (2.51–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.16× (1.16–1.17) | 0.82× (0.82–0.82) | 5.16× (5.15–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.46× (0.45–0.48) | 0.35× (0.32–0.38) | 0.81× (0.79–0.84) | 0.69× (0.66–0.72) | 2.80× (2.72–2.87) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.47× (0.44–0.49) | 0.80× (0.79–0.81) | 0.80× (0.78–0.83) | 2.30× (2.27–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.76× (0.76–0.77) | 5.26× (5.23–5.28) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.68× (0.67–0.68) | 6.42× (6.40–6.45) | 0.40× (0.40–0.40) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.43) | 0.35× (0.32–0.39) | 0.97× (0.95–0.98) | 0.63× (0.61–0.65) | 3.12× (3.07–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.65 MB → 13.5 MB, 5.1×) | 4 | 62 MB | 0.46× (0.45–0.47) | 0.42× (0.38–0.46) | 0.83× (0.79–0.87) | 0.67× (0.64–0.71) | 2.12× (2.04–2.20) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.09–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 662 | 1.52 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,030 | 1.79 | 1.20× (1.15–1.26) | 78% | 1.08× (1.00–1.16) | 1.12× (1.08–1.16) |
| 4: all 4 CPUs, sharing SMT siblings | 1,260 | 2.77 | 1.82× (1.77–1.86) | 48% | 1.29× (1.24–1.36) | 1.95× (1.82–2.08) |

**Best:** mibz2 at 4 threads (1,260 MB/s); lbzip2 at 4 threads (277 MB/s); pbz2 at 4 threads (417 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.22, lbzip2 1t 0.08, lbzip2 2t 0.32, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.43 | 0.01 | 0.07 | 0.166 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4349 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 14.0% |
| MTF + RLE2 | 1.46 | 19.5% |
| T-vector build | 1.36 | 18.2% |
| walk (the one each build uses) | 3.18 | 42.5% |
| of which: segment assembly | 0.14 | 1.9% |
| RLE1 | 0.16 | 2.2% |
| CRC | 0.26 | 3.5% |
| **total** | 7.48 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.300 | 0.721 | (in its total) | 2.40× (2.39–2.42) |  |
| walk | 0.672 | 1.612 | (in its total) | 2.41× (2.38–2.45) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.031 | 0.034 | (in its total) | 1.10× (1.06–1.14) |  |
| inverse BWT, total | 1.004 | 2.365 | 1.338 | 2.37× (2.34–2.40) | 1.33× (1.33–1.34) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 46m25s ± 5m17s |
| 4 threads | 24m25s ± 3m28s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m54s ± 2m16s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 15m54s ± 2m16s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 2.08 ns a byte of output at 4 threads (CPU, summed over them), 1.23 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,649,511 | 13,461,581 | \`9982d156bb828917…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4348.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
