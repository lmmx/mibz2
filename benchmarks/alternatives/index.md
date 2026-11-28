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

The latest record: [`0762c7e`](<https://github.com/lmmx/mibz2/commit/0762c7eabe72b7acc2cc7f4003ff4885ad0c176f>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46113921437>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.308 s | 680 | 6.7 | 2.96× (2.87–3.04) | 7.48× (7.23–7.73) | 7.82× (7.53–8.11) | 8.92× (8.61–9.24) | 7.84× (7.58–8.11) | 8.14× (7.89–8.40) | 7.82× (7.58–8.08) | 2.33× (2.27–2.39) | 2.67× (2.59–2.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.178 s | 1,178 | 7.3 | 3.00× (2.87–3.15) | 6.64× (6.32–6.96) | 7.98× (7.56–8.42) | 8.02× (7.63–8.43) | 7.21× (6.88–7.57) | 13.84× (13.17–14.54) | 13.30× (12.74–13.88) | 3.95× (3.79–4.12) | 4.53× (4.33–4.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.150 s | 1,394 | 11.6 | 3.05× (2.96–3.15) | 4.87× (4.77–4.97) | 6.33× (6.15–6.51) | 5.87× (5.68–6.07) | 5.85× (5.67–6.03) | 16.87× (16.49–17.25) | 16.21× (15.95–16.47) | 4.82× (4.72–4.92) | 5.53× (5.40–5.66) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,307 | 3.9 | 0.56× (0.55–0.58) | 3.79× (3.65–3.94) | 4.61× (4.40–4.82) | 3.70× (3.59–3.81) | 6.47× (6.30–6.65) | 2.79× (2.72–2.86) | 2.75× (2.67–2.84) | 0.99× (0.97–1.01) | 2.19× (2.03–2.36) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,235 | 4.2 | 0.49× (0.49–0.50) | 2.99× (2.84–3.14) | 5.56× (5.38–5.75) | 3.51× (3.41–3.61) | 6.05× (5.89–6.22) | 2.66× (2.59–2.73) | 2.62× (2.56–2.69) | 0.94× (0.93–0.96) | 2.09× (1.94–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,205 | 4.3 | 0.48× (0.46–0.49) | 2.88× (2.76–3.01) | 5.59× (5.12–6.09) | 3.39× (3.27–3.51) | 5.94× (5.75–6.13) | 2.56× (2.48–2.63) | 2.52× (2.46–2.58) | 0.91× (0.88–0.94) | 2.01× (1.87–2.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 998 | 4.9 | 2.97× (2.90–3.03) | 3.58× (3.45–3.71) | 4.27× (4.09–4.47) | 4.73× (4.60–4.86) | 5.54× (5.36–5.73) | 3.19× (3.03–3.34) | 2.79× (2.52–3.09) | 2.42× (1.94–3.03) | 3.31× (3.06–3.59) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,452 | 6.3 | 2.36× (2.28–2.43) | 3.30× (3.06–3.55) | 4.59× (4.35–4.84) | 4.11× (3.94–4.29) | 6.25× (6.01–6.49) | 4.56× (4.32–4.81) | 3.99× (3.62–4.39) | 3.47× (2.80–4.30) | 4.74× (4.40–5.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,248 | 12.2 | 1.93× (1.80–2.06) | 2.12× (1.86–2.43) | 3.13× (2.86–3.43) | 2.39× (2.17–2.63) | 4.69× (4.35–5.07) | 3.83× (3.49–4.20) | 3.35× (3.02–3.72) | 2.92× (2.37–3.58) | 3.98× (3.63–4.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.081 s | 99 | 46.6 | 1.50× (1.48–1.53) | 3.38× (3.30–3.47) | 3.10× (3.02–3.19) | 3.06× (2.97–3.15) | 3.40× (3.33–3.47) | 3.85× (3.75–3.95) | 3.86× (3.75–3.98) | 1.42× (1.40–1.45) | 1.65× (1.61–1.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.050 s | 162 | 53.0 | 1.38× (1.31–1.45) | 3.17× (3.05–3.30) | 2.79× (2.68–2.90) | 3.56× (3.41–3.72) | 3.39× (3.27–3.50) | 6.13× (5.86–6.40) | 6.15× (5.86–6.46) | 2.27× (2.17–2.37) | 2.63× (2.53–2.74) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 198 | 69.5 | 1.44× (1.37–1.52) | 2.82× (2.63–3.03) | 2.65× (2.52–2.78) | 3.15× (3.04–3.26) | 3.47× (3.32–3.63) | 7.60× (7.23–8.00) | 7.64× (7.22–8.08) | 2.81× (2.68–2.95) | 3.27× (3.12–3.41) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.086 s | 111 | 41.4 | 1.60× (1.58–1.62) | 4.07× (3.99–4.15) | 3.32× (3.27–3.37) | 3.39× (3.33–3.44) | 3.80× (3.73–3.87) | 4.75× (4.69–4.81) | 5.08× (5.04–5.12) | 1.69× (1.66–1.72) | 1.95× (1.93–1.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 174 | 49.1 | 1.41× (1.35–1.48) | 3.57× (3.23–3.94) | 2.94× (2.79–3.09) | 4.01× (3.85–4.19) | 3.48× (3.35–3.62) | 7.29× (6.97–7.61) | 7.80× (7.44–8.17) | 2.60× (2.50–2.70) | 3.00× (2.86–3.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 218 | 66.7 | 1.57× (1.51–1.63) | 2.93× (2.79–3.08) | 2.85× (2.77–2.94) | 3.73× (3.55–3.92) | 3.54× (3.38–3.70) | 9.09× (8.80–9.39) | 9.72× (9.38–10.08) | 3.24× (3.14–3.34) | 3.74× (3.61–3.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 980 | 5.4 | 0.62× (0.61–0.63) | 3.04× (2.94–3.15) | 4.17× (4.01–4.35) | 3.18× (3.13–3.22) | 7.15× (7.00–7.31) | 2.33× (2.29–2.38) | 2.36× (2.32–2.42) | 0.99× (0.97–1.00) | 2.03× (1.97–2.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 905 | 6.2 | 0.49× (0.49–0.50) | 2.80× (2.69–2.92) | 4.69× (4.51–4.89) | 2.93× (2.87–3.00) | 6.48× (6.35–6.62) | 2.15× (2.10–2.19) | 2.17× (2.12–2.23) | 0.91× (0.88–0.94) | 1.87× (1.79–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 885 | 6.1 | 0.49× (0.47–0.50) | 2.77× (2.68–2.86) | 4.80× (4.53–5.09) | 2.92× (2.85–2.99) | 6.47× (6.37–6.57) | 2.11× (2.07–2.15) | 2.14× (2.10–2.19) | 0.89× (0.87–0.92) | 1.84× (1.78–1.90) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.043 s | 348 | 13.6 | 1.96× (1.91–2.02) | 4.47× (4.29–4.66) | 5.04× (4.81–5.28) | 5.15× (5.02–5.29) | 5.37× (5.27–5.47) | 5.16× (5.03–5.30) | 4.95× (4.84–5.07) | 1.68× (1.63–1.73) | 2.08× (2.01–2.14) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.031 s | 484 | 17.4 | 1.49× (1.38–1.62) | 3.39× (3.12–3.68) | 4.18× (3.90–4.48) | 4.60× (4.25–4.98) | 4.43× (4.16–4.72) | 6.88× (6.41–7.39) | 6.60× (6.16–7.07) | 2.24× (2.11–2.38) | 2.77× (2.60–2.95) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.028 s | 539 | 27.7 | 1.48× (1.41–1.55) | 2.63× (2.45–2.82) | 3.51× (3.31–3.73) | 3.73× (3.54–3.92) | 4.19× (3.99–4.40) | 7.78× (7.49–8.08) | 7.46× (7.19–7.74) | 2.54× (2.43–2.65) | 3.13× (3.01–3.26) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.94× (2.86–3.03) | 7.43× (7.19–7.68) | 7.77× (7.49–8.06) | 8.88× (8.57–9.19) | 7.80× (7.55–8.07) | 8.10× (7.85–8.35) | 7.78× (7.54–8.03) | 2.32× (2.26–2.38) | 2.64× (2.56–2.72) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.79× (2.73–2.84) | 6.86× (6.65–7.07) | 7.31× (7.07–7.55) | 8.35× (8.10–8.62) | 7.19× (6.98–7.41) | 7.36× (7.22–7.51) | 7.08× (6.94–7.22) | 2.11× (2.05–2.16) | 2.40× (2.33–2.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.89× (2.81–2.97) | 5.26× (5.14–5.38) | 5.88× (5.76–6.00) | 6.42× (6.26–6.58) | 5.54× (5.43–5.65) | 4.73× (4.58–4.88) | 4.55× (4.43–4.67) | 1.35× (1.31–1.40) | 1.54× (1.50–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.06× (2.98–3.15) | 3.99× (3.82–4.16) | 3.44× (3.34–3.53) | 5.94× (5.79–6.08) | 2.61× (2.56–2.67) | 2.58× (2.51–2.65) | 0.99× (0.97–1.01) | 1.85× (1.81–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.53–0.55) | 2.62× (2.56–2.68) | 4.67× (4.55–4.79) | 3.15× (3.06–3.24) | 5.34× (5.19–5.49) | 2.39× (2.33–2.46) | 2.36× (2.30–2.42) | 0.91× (0.89–0.92) | 1.69× (1.67–1.72) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.50× (0.49–0.52) | 2.76× (2.69–2.83) | 4.74× (4.33–5.20) | 3.11× (3.01–3.22) | 5.34× (5.18–5.50) | 2.36× (2.29–2.43) | 2.32× (2.27–2.38) | 0.89× (0.86–0.92) | 1.67× (1.62–1.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.81× (2.74–2.87) | 3.25× (3.16–3.35) | 3.88× (3.77–3.99) | 4.43× (4.30–4.56) | 5.17× (5.00–5.35) | 3.01× (2.87–3.16) | 2.66× (2.38–2.96) | 2.33× (1.86–2.92) | 2.97× (2.74–3.21) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.28–2.41) | 2.77× (2.66–2.89) | 3.28× (3.16–3.40) | 3.66× (3.54–3.77) | 4.21× (4.07–4.36) | 2.32× (2.22–2.42) | 2.05× (1.85–2.26) | 1.79× (1.44–2.23) | 2.29× (2.12–2.46) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.03–2.23) | 1.85× (1.75–1.96) | 2.21× (2.07–2.35) | 2.17× (2.05–2.29) | 2.40× (2.22–2.60) | 1.18× (1.09–1.27) | 1.04× (0.91–1.18) | 0.91× (0.71–1.16) | 1.16× (1.04–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.47–1.51) | 3.29× (3.20–3.37) | 3.01× (2.92–3.11) | 3.01× (2.93–3.10) | 3.35× (3.28–3.42) | 3.79× (3.69–3.89) | 3.80× (3.69–3.92) | 1.41× (1.39–1.44) | 1.57× (1.53–1.61) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.38× (1.35–1.41) | 3.05× (2.90–3.21) | 2.56× (2.49–2.62) | 3.37× (3.20–3.56) | 2.95× (2.87–3.04) | 3.32× (3.25–3.40) | 3.34× (3.24–3.44) | 1.24× (1.22–1.27) | 1.38× (1.34–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.55–1.61) | 2.91× (2.75–3.07) | 2.36× (2.30–2.43) | 3.12× (3.03–3.21) | 2.91× (2.81–3.02) | 2.56× (2.46–2.66) | 2.57× (2.45–2.69) | 0.95× (0.92–0.99) | 1.06× (1.02–1.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.59× (1.57–1.61) | 3.95× (3.87–4.02) | 3.24× (3.19–3.29) | 3.34× (3.29–3.39) | 3.74× (3.67–3.81) | 4.67× (4.61–4.73) | 4.99× (4.95–5.04) | 1.68× (1.65–1.71) | 1.87× (1.85–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.42× (1.38–1.46) | 3.42× (3.29–3.54) | 2.73× (2.64–2.83) | 3.79× (3.67–3.91) | 3.12× (3.05–3.19) | 3.91× (3.75–4.07) | 4.18× (4.02–4.34) | 1.40× (1.36–1.45) | 1.56× (1.50–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.57–1.78) | 3.19× (3.05–3.34) | 2.43× (2.36–2.50) | 3.37× (3.24–3.51) | 3.03× (2.93–3.14) | 2.88× (2.73–3.04) | 3.08× (2.93–3.23) | 1.03× (0.99–1.08) | 1.15× (1.10–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.53× (2.50–2.56) | 3.35× (3.30–3.41) | 2.85× (2.81–2.90) | 6.25× (6.13–6.37) | 2.14× (2.10–2.18) | 2.16× (2.12–2.21) | 0.99× (0.98–1.00) | 1.50× (1.48–1.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.18× (2.13–2.22) | 3.75× (3.70–3.81) | 2.59× (2.53–2.65) | 5.43× (5.32–5.55) | 1.88× (1.84–1.92) | 1.90× (1.85–1.95) | 0.87× (0.84–0.89) | 1.32× (1.29–1.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.35× (2.30–2.40) | 3.75× (3.64–3.87) | 2.64× (2.58–2.71) | 5.60× (5.53–5.67) | 1.91× (1.88–1.94) | 1.93× (1.89–1.97) | 0.88× (0.86–0.90) | 1.34× (1.33–1.36) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 1.93× (1.89–1.97) | 4.25× (4.14–4.36) | 4.76× (4.57–4.96) | 5.00× (4.91–5.09) | 5.21× (5.14–5.28) | 5.01× (4.91–5.11) | 4.81× (4.73–4.89) | 1.66× (1.63–1.70) | 1.89× (1.86–1.93) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.57× (1.52–1.62) | 3.42× (3.30–3.56) | 3.77× (3.63–3.93) | 4.35× (4.15–4.56) | 3.99× (3.83–4.17) | 3.83× (3.67–4.01) | 3.68× (3.55–3.82) | 1.27× (1.22–1.33) | 1.45× (1.39–1.51) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.57× (1.52–1.63) | 2.73× (2.57–2.89) | 2.99× (2.86–3.12) | 3.33× (3.11–3.56) | 3.18× (3.08–3.29) | 2.43× (2.34–2.52) | 2.33× (2.27–2.39) | 0.80× (0.78–0.83) | 0.92× (0.89–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.35) | 0.97× (0.97–0.97) | 0.79× (0.78–0.80) | 10.98× (10.96–11.00) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.53× (0.45–0.62) | 0.71× (0.69–0.74) | 0.61× (0.59–0.63) | 5.76× (5.55–5.97) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.36× (0.34–0.39) | 0.59× (0.48–0.74) | 0.65× (0.62–0.68) | 0.57× (0.54–0.61) | 3.78× (3.56–4.01) | 0.09× (0.08–0.10) | 0.10× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.66–0.66) | 7.94× (7.93–7.96) | 0.65× (0.65–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.75) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.67–0.72) | 0.66× (0.60–0.73) | 1.63× (1.61–1.65) | 0.82× (0.81–0.84) | 5.92× (5.84–6.00) | 0.48× (0.48–0.49) | 0.50× (0.49–0.51) | 0.55× (0.54–0.56) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.63–0.70) | 0.73× (0.67–0.81) | 1.19× (1.13–1.26) | 0.74× (0.69–0.78) | 3.51× (3.34–3.69) | 0.28× (0.26–0.29) | 0.29× (0.27–0.30) | 0.32× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.15× (5.14–5.16) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.50) | 0.41× (0.40–0.43) | 0.89× (0.86–0.92) | 0.72× (0.69–0.74) | 3.08× (2.99–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.62–0.67) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.82× (0.81–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.21× (5.21–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.36× (0.34–0.39) | 0.82× (0.80–0.84) | 0.71× (0.68–0.73) | 2.82× (2.74–2.91) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.52) | 0.79× (0.78–0.81) | 0.80× (0.78–0.82) | 2.29× (2.26–2.32) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.20–0.22) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.31× (5.30–5.32) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.17–6.21) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.27× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.85–1.86) | 0.89× (0.88–0.90) | 6.16× (6.14–6.19) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.38) | 0.66× (0.66–0.66) | 6.36× (6.35–6.37) | 0.39× (0.39–0.39) | 0.41× (0.40–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 38 MB | 0.44× (0.42–0.46) | 0.43× (0.41–0.45) | 1.00× (0.96–1.04) | 0.66× (0.64–0.69) | 3.28× (3.16–3.40) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.16× (0.15–0.16) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 57 MB | 0.50× (0.48–0.51) | 0.45× (0.41–0.50) | 0.89× (0.88–0.91) | 0.74× (0.71–0.77) | 2.30× (2.25–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 680 | 1.48 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,178 | 1.62 | 1.10× (1.07–1.13) | 85% | 1.01× (0.99–1.04) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,394 | 2.56 | 1.71× (1.64–1.79) | 52% | 1.21× (1.18–1.24) | 1.68× (1.62–1.75) |

**Best:** mibz2 at 4 threads (1,394 MB/s); lbzip2 at 4 threads (288 MB/s); pbz2 at 4 threads (450 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.32, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.04 | 0.41 | 0.01 | 0.07 | 0.150 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4523 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.73 | 10.7% |
| MTF + RLE2 | 1.47 | 21.6% |
| T-vector build | 1.29 | 19.0% |
| walk (the one each build uses) | 2.91 | 42.9% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.26 | 3.7% |
| **total** | 6.79 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.288 | 0.689 | (in its total) | 2.20× (1.92–2.51) |  |
| walk | 0.631 | 1.554 | (in its total) | 2.26× (1.96–2.61) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.028 | 0.029 | (in its total) | 0.95× (0.80–1.12) |  |
| inverse BWT, total | 0.948 | 2.272 | 1.276 | 2.20× (1.91–2.53) | 1.30× (1.12–1.51) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m38s ± 5m51s |
| 4 threads | 21m31s ± 3m07s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m44s ± 2m11s |
| the scanner alone (its serial part) | 6m15s ± 30.2s |
| the workers alone, over 4 | 14m44s ± 2m11s |

At 4 threads the workers bound it. Per byte: the scanner 3.6 ns a byte of bz2, the workers 1.93 ns a byte of output at 4 threads (CPU, summed over them), 1.26 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4522.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
