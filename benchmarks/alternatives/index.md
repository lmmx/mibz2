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

The latest record: [`938bcfe`](<https://github.com/lmmx/mibz2/commit/938bcfe83092d1b992ba5746ab9554ea26eb4a1d>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44975894539>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.288 s | 729 | 6.2 | 2.82× (2.79–2.86) | 7.07× (7.04–7.10) | 7.33× (7.28–7.37) | 8.42× (8.39–8.44) | 7.52× (7.49–7.54) | 7.74× (7.71–7.77) | 7.36× (7.35–7.38) | 2.22× (2.21–2.22) | 2.55× (2.53–2.56) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.169 s | 1,243 | 6.9 | 2.82× (2.72–2.92) | 6.19× (5.97–6.41) | 7.36× (7.05–7.70) | 7.58× (7.35–7.82) | 6.80× (6.55–7.05) | 12.95× (12.47–13.46) | 12.33× (11.88–12.80) | 3.71× (3.57–3.86) | 4.26× (4.10–4.43) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.141 s | 1,488 | 10.3 | 2.72× (2.67–2.78) | 4.52× (4.36–4.68) | 5.63× (5.52–5.74) | 5.35× (5.26–5.45) | 5.51× (5.39–5.63) | 15.77× (15.48–16.07) | 15.01× (14.76–15.27) | 4.52× (4.44–4.60) | 5.19× (5.09–5.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.013 s | 1,492 | 3.3 | 0.58× (0.57–0.59) | 3.77× (3.71–3.84) | 4.69× (4.61–4.77) | 3.75× (3.68–3.82) | 6.65× (6.52–6.78) | 2.85× (2.80–2.90) | 2.84× (2.79–2.90) | 1.01× (0.99–1.02) | 2.27× (2.23–2.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,449 | 3.5 | 0.47× (0.37–0.59) | 2.68× (2.10–3.41) | 5.50× (4.31–7.00) | 3.35× (2.63–4.26) | 5.81× (4.57–7.39) | 2.52× (1.98–3.21) | 2.51× (1.97–3.20) | 0.89× (0.70–1.14) | 2.00× (1.57–2.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,429 | 3.5 | 0.52× (0.51–0.53) | 2.93× (2.87–2.98) | 5.99× (5.87–6.10) | 3.65× (3.58–3.73) | 6.40× (6.26–6.54) | 2.75× (2.70–2.80) | 2.74× (2.69–2.79) | 0.97× (0.95–0.99) | 2.18× (2.14–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,120 | 4.3 | 2.90× (2.88–2.92) | 3.48× (3.46–3.50) | 4.18× (4.16–4.20) | 4.67× (4.64–4.69) | 5.59× (5.57–5.62) | 3.10× (2.99–3.22) | 2.63× (2.61–2.65) | 2.17× (2.15–2.18) | 3.47× (3.45–3.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,562 | 5.5 | 2.13× (1.93–2.36) | 3.10× (2.79–3.45) | 4.15× (3.67–4.68) | 3.76× (3.38–4.18) | 5.73× (5.17–6.36) | 4.12× (3.76–4.52) | 3.50× (3.15–3.89) | 2.88× (2.59–3.20) | 4.61× (4.15–5.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,277 | 11.4 | 1.67× (1.61–1.73) | 2.03× (1.94–2.13) | 2.89× (2.76–3.03) | 2.10× (1.99–2.21) | 4.37× (4.22–4.52) | 3.52× (3.31–3.75) | 2.99× (2.88–3.10) | 2.46× (2.37–2.55) | 3.94× (3.80–4.09) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.076 s | 105 | 43.6 | 1.39× (1.38–1.40) | 3.19× (3.18–3.21) | 2.83× (2.80–2.86) | 2.81× (2.80–2.83) | 3.18× (3.10–3.26) | 3.54× (3.53–3.56) | 3.59× (3.57–3.60) | 1.36× (1.36–1.37) | 1.59× (1.58–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.048 s | 166 | 50.1 | 1.25× (1.22–1.29) | 2.88× (2.76–3.00) | 2.61× (2.49–2.74) | 3.35× (3.22–3.48) | 3.14× (3.06–3.22) | 5.58× (5.45–5.71) | 5.64× (5.52–5.77) | 2.14× (2.10–2.19) | 2.50× (2.45–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.039 s | 203 | 66.7 | 1.29× (1.28–1.31) | 2.42× (2.36–2.47) | 2.50× (2.37–2.65) | 3.00× (2.86–3.15) | 3.17× (3.13–3.21) | 6.86× (6.80–6.92) | 6.94× (6.89–6.99) | 2.64× (2.61–2.66) | 3.08× (3.06–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.080 s | 120 | 38.2 | 1.49× (1.47–1.51) | 3.77× (3.71–3.83) | 3.06× (3.00–3.12) | 3.14× (3.09–3.18) | 3.53× (3.46–3.60) | 4.45× (4.39–4.52) | 4.80× (4.72–4.89) | 1.60× (1.58–1.62) | 1.91× (1.83–2.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.051 s | 188 | 44.5 | 1.29× (1.24–1.35) | 3.20× (3.12–3.28) | 2.82× (2.74–2.90) | 3.88× (3.73–4.03) | 3.33× (3.23–3.44) | 6.94× (6.72–7.17) | 7.49× (7.23–7.74) | 2.49× (2.41–2.58) | 2.98× (2.87–3.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 218 | 61.8 | 1.32× (1.25–1.40) | 2.53× (2.42–2.65) | 2.41× (2.30–2.51) | 3.29× (3.15–3.43) | 3.09× (2.93–3.26) | 8.01× (7.65–8.38) | 8.64× (8.25–9.04) | 2.88× (2.75–3.01) | 3.44× (3.22–3.67) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,124 | 4.5 | 0.63× (0.62–0.63) | 3.62× (3.20–4.09) | 5.13× (4.44–5.93) | 3.22× (3.18–3.25) | 7.24× (7.17–7.31) | 2.43× (2.36–2.49) | 2.42× (2.39–2.44) | 1.00× (0.98–1.02) | 2.30× (2.27–2.34) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.009 s | 1,053 | 5.0 | 0.51× (0.50–0.52) | 3.23× (3.18–3.28) | 4.65× (4.47–4.85) | 3.04× (2.98–3.10) | 7.01× (6.66–7.38) | 2.29× (2.22–2.36) | 2.28× (2.24–2.32) | 0.94× (0.93–0.96) | 2.17× (2.13–2.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 1,042 | 4.9 | 0.51× (0.50–0.52) | 3.19× (3.14–3.25) | 4.60× (4.34–4.87) | 3.03× (2.98–3.07) | 7.07× (6.69–7.48) | 2.26× (2.20–2.32) | 2.25× (2.21–2.29) | 0.93× (0.91–0.95) | 2.14× (2.11–2.18) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 0.042 s | 319 | 14.5 | 1.76× (1.75–1.78) | 4.34× (4.32–4.35) | 4.62× (4.61–4.63) | 4.82× (4.81–4.84) | 5.06× (5.03–5.09) | 5.16× (5.15–5.18) | 5.00× (4.91–5.09) | 1.55× (1.54–1.55) | 1.78× (1.69–1.87) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 0.031 s | 436 | 18.1 | 1.34× (1.25–1.45) | 3.34× (3.08–3.62) | 3.98× (3.75–4.22) | 4.40× (4.20–4.61) | 4.38× (4.08–4.69) | 7.00× (6.52–7.51) | 6.77× (6.30–7.28) | 2.09× (1.95–2.24) | 2.41× (2.25–2.58) | yes |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 0.029 s | 465 | 28.2 | 1.24× (1.22–1.27) | 2.50× (2.46–2.55) | 3.00× (2.91–3.09) | 3.17× (3.04–3.29) | 3.75× (3.68–3.82) | 7.51× (7.39–7.63) | 7.26× (7.08–7.46) | 2.25× (2.21–2.29) | 2.58× (2.45–2.72) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.81× (2.78–2.85) | 7.02× (7.00–7.05) | 7.28× (7.24–7.32) | 8.38× (8.36–8.41) | 7.49× (7.47–7.52) | 7.71× (7.68–7.74) | 7.34× (7.32–7.36) | 2.21× (2.20–2.22) | 2.52× (2.51–2.53) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.59× (2.57–2.62) | 6.47× (6.42–6.51) | 6.81× (6.76–6.85) | 7.96× (7.91–8.02) | 6.85× (6.80–6.90) | 6.97× (6.93–7.01) | 6.63× (6.61–6.65) | 2.00× (1.99–2.01) | 2.28× (2.27–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.69× (2.66–2.72) | 5.07× (5.02–5.12) | 5.43× (5.40–5.46) | 6.18× (6.14–6.23) | 5.50× (5.45–5.54) | 4.66× (4.61–4.70) | 4.43× (4.40–4.47) | 1.34× (1.33–1.35) | 1.52× (1.51–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.63) | 3.18× (3.11–3.24) | 3.98× (3.92–4.04) | 3.54× (3.48–3.60) | 6.22× (6.11–6.34) | 2.73× (2.69–2.77) | 2.70× (2.66–2.75) | 1.01× (0.99–1.03) | 1.90× (1.87–1.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.52× (0.41–0.66) | 2.59× (2.02–3.31) | 4.54× (3.55–5.81) | 3.10× (2.42–3.96) | 5.29× (4.14–6.77) | 2.34× (1.83–2.99) | 2.32× (1.81–2.96) | 0.86× (0.67–1.11) | 1.63× (1.27–2.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.07× (3.02–3.12) | 5.05× (4.94–5.16) | 3.44× (3.37–3.50) | 5.91× (5.79–6.03) | 2.60× (2.56–2.64) | 2.57× (2.54–2.61) | 0.96× (0.94–0.98) | 1.81× (1.78–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.76–2.81) | 3.26× (3.23–3.28) | 3.83× (3.81–3.85) | 4.45× (4.42–4.48) | 5.31× (5.28–5.34) | 2.99× (2.89–3.08) | 2.53× (2.51–2.55) | 2.09× (2.08–2.11) | 2.83× (2.81–2.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.23× (2.02–2.46) | 2.63× (2.36–2.93) | 3.10× (2.81–3.42) | 3.50× (3.17–3.86) | 4.11× (3.74–4.52) | 2.22× (1.99–2.47) | 1.88× (1.70–2.08) | 1.55× (1.41–1.72) | 2.10× (1.90–2.32) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.92× (1.88–1.96) | 1.74× (1.68–1.81) | 2.06× (2.01–2.11) | 2.06× (2.00–2.12) | 2.35× (2.29–2.42) | 1.12× (1.07–1.17) | 0.95× (0.93–0.97) | 0.79× (0.77–0.81) | 1.06× (1.04–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.38–1.39) | 3.05× (3.04–3.07) | 2.75× (2.72–2.78) | 2.79× (2.77–2.80) | 3.14× (3.07–3.22) | 3.51× (3.49–3.53) | 3.54× (3.53–3.56) | 1.36× (1.35–1.36) | 1.48× (1.47–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.26–1.27) | 2.77× (2.75–2.80) | 2.39× (2.38–2.41) | 3.14× (3.12–3.16) | 2.75× (2.72–2.77) | 3.05× (3.03–3.07) | 3.08× (3.07–3.10) | 1.18× (1.18–1.18) | 1.29× (1.28–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.39× (1.35–1.43) | 2.57× (2.55–2.60) | 2.16× (2.00–2.34) | 2.79× (2.74–2.84) | 2.63× (2.60–2.67) | 2.29× (2.26–2.31) | 2.31× (2.28–2.34) | 0.88× (0.87–0.89) | 0.96× (0.95–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.47–1.51) | 3.65× (3.59–3.70) | 2.99× (2.95–3.04) | 3.11× (3.06–3.16) | 3.49× (3.43–3.56) | 4.41× (4.34–4.48) | 4.75× (4.67–4.83) | 1.59× (1.57–1.61) | 1.82× (1.74–1.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.35) | 3.27× (3.24–3.30) | 2.63× (2.57–2.70) | 3.64× (3.60–3.68) | 3.06× (3.03–3.08) | 3.82× (3.79–3.85) | 4.12× (4.08–4.15) | 1.38× (1.37–1.39) | 1.57× (1.51–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.49–1.60) | 2.98× (2.92–3.04) | 2.21× (2.16–2.27) | 3.21× (3.16–3.27) | 2.88× (2.82–2.93) | 2.76× (2.71–2.82) | 2.98× (2.92–3.04) | 1.00× (0.98–1.02) | 1.14× (1.09–1.19) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.80× (2.42–3.24) | 3.71× (3.10–4.43) | 2.95× (2.90–3.00) | 6.49× (6.40–6.59) | 2.25× (2.22–2.29) | 2.25× (2.21–2.28) | 0.99× (0.97–1.02) | 1.54× (1.51–1.58) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.35× (2.31–2.38) | 3.98× (3.93–4.02) | 2.78× (2.74–2.82) | 6.12× (5.82–6.44) | 2.06× (2.03–2.09) | 2.05× (2.02–2.08) | 0.91× (0.90–0.92) | 1.41× (1.39–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.51× (2.47–2.54) | 3.99× (3.92–4.05) | 2.84× (2.79–2.88) | 6.34× (6.01–6.70) | 2.09× (2.05–2.12) | 2.08× (2.05–2.12) | 0.92× (0.90–0.94) | 1.43× (1.41–1.45) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 1.74× (1.73–1.76) | 4.11× (4.09–4.14) | 4.43× (4.41–4.45) | 4.72× (4.70–4.74) | 4.95× (4.91–4.98) | 5.06× (5.03–5.08) | 4.89× (4.80–4.97) | 1.53× (1.52–1.54) | 1.67× (1.66–1.67) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 1.48× (1.47–1.49) | 3.42× (3.40–3.45) | 3.76× (3.74–3.79) | 4.36× (4.33–4.40) | 4.05× (4.03–4.08) | 4.05× (4.02–4.08) | 3.92× (3.85–3.98) | 1.23× (1.22–1.24) | 1.33× (1.32–1.35) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 1.50× (1.48–1.53) | 2.66× (2.64–2.68) | 2.91× (2.86–2.96) | 3.30× (3.26–3.35) | 3.13× (3.10–3.17) | 2.59× (2.56–2.62) | 2.51× (2.46–2.56) | 0.79× (0.78–0.79) | 0.85× (0.85–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.32× (0.32–0.33) | 0.97× (0.97–0.97) | 0.78× (0.76–0.80) | 10.95× (10.91–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.34) | 0.47× (0.38–0.57) | 0.66× (0.63–0.69) | 0.57× (0.54–0.59) | 5.32× (5.07–5.58) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.32× (0.30–0.34) | 0.55× (0.47–0.64) | 0.57× (0.54–0.60) | 0.49× (0.47–0.52) | 3.32× (3.16–3.49) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.15–0.16) | 1.76× (1.76–1.76) | 0.84× (0.84–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.46× (0.44–0.48) | 1.68× (1.68–1.69) | 0.65× (0.65–0.66) | 7.95× (7.93–7.98) | 0.64× (0.64–0.64) | 0.68× (0.67–0.68) | 0.73× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.70) | 0.66× (0.60–0.72) | 1.57× (1.51–1.64) | 0.80× (0.76–0.83) | 5.70× (5.48–5.92) | 0.46× (0.44–0.48) | 0.48× (0.47–0.50) | 0.52× (0.50–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.61× (0.58–0.64) | 0.67× (0.63–0.71) | 1.10× (1.05–1.16) | 0.69× (0.66–0.73) | 3.23× (3.09–3.39) | 0.25× (0.24–0.26) | 0.27× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.51) | 0.41× (0.38–0.43) | 0.89× (0.86–0.92) | 0.71× (0.69–0.74) | 3.09× (2.98–3.20) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.64–0.66) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.84× (0.82–0.86) | 2.58× (2.53–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.17) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.51) | 0.40× (0.38–0.42) | 0.86× (0.82–0.90) | 0.74× (0.71–0.77) | 2.95× (2.81–3.09) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.53) | 0.80× (0.79–0.82) | 0.82× (0.80–0.85) | 2.32× (2.28–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.77× (0.76–0.77) | 5.26× (5.24–5.27) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.31× (5.29–5.33) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.67× (0.66–0.67) | 6.34× (6.32–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 2 | 34 MB | 0.50× (0.49–0.51) | 0.44× (0.41–0.48) | 1.17× (1.13–1.20) | 0.74× (0.72–0.76) | 3.71× (3.62–3.80) | 0.23× (0.22–0.23) | 0.24× (0.24–0.25) | 0.36× (0.35–0.38) | 0.18× (0.17–0.18) |
| `text` (2.64 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.50) | 0.44× (0.40–0.48) | 0.91× (0.89–0.94) | 0.76× (0.74–0.77) | 2.32× (2.28–2.37) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 729 | 1.38 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,243 | 1.53 | 1.11× (1.10–1.11) | 84% | 1.02× (1.01–1.02) | 1.02× (1.01–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,488 | 2.29 | 1.65× (1.64–1.67) | 51% | 1.19× (1.19–1.20) | 1.58× (1.55–1.61) |

**Best:** mibz2 at 4 threads (1,488 MB/s); lbzip2 at 4 threads (334 MB/s); pbz2 at 4 threads (547 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.92, mibz2 4t 1.26, lbzip2 1t 0.07, lbzip2 2t 0.27, lbzip2 4t 0.45, pbz2 1t 0.00, pbz2 2t 0.49, pbz2 4t 0.56. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.35 | 0.01 | 0.07 | 0.141 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4502 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 14.7% |
| MTF + RLE2 | 1.42 | 20.7% |
| T-vector build | 1.29 | 18.9% |
| walk (the one each build uses) | 2.71 | 39.7% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.7% |
| **total** | 6.83 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.285 | 0.682 | (in its total) | 2.39× (2.38–2.41) |  |
| walk | 0.590 | 1.510 | (in its total) | 2.57× (2.56–2.57) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.028 | (in its total) | 1.16× (1.15–1.17) |  |
| inverse BWT, total | 0.900 | 2.220 | 1.252 | 2.47× (2.46–2.48) | 1.39× (1.38–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 41m52s ± 4m47s |
| 4 threads | 20m32s ± 2m54s |
| 4 threads, the sample's overlap gap as a fixed cost | 12m35s ± 1m47s |
| the scanner alone (its serial part) | 7m43s ± 30.2s |
| the workers alone, over 4 | 12m35s ± 1m47s |

At 4 threads the workers bound it. Per byte: the scanner 4.5 ns a byte of bz2, the workers 1.65 ns a byte of output at 4 threads (CPU, summed over them), 1.12 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,640,714 | 13,421,331 | \`8da9d679141c517c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4501.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
