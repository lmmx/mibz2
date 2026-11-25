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

The latest record: [`49a6729`](<https://github.com/lmmx/mibz2/commit/49a6729f5679532f6f825aebb62bedf9ddc958c2>) (2026-11-25).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45555944914>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.307 s | 684 | 6.0 | 2.88× (2.80–2.96) | 7.15× (6.90–7.40) | 7.57× (7.32–7.83) | 8.58× (8.34–8.83) | 7.61× (7.37–7.85) | 7.88× (7.61–8.16) | 7.41× (7.15–7.69) | 2.23× (2.16–2.30) | 2.60× (2.51–2.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.180 s | 1,165 | 6.8 | 3.09× (2.88–3.31) | 6.70× (6.45–6.96) | 8.15× (7.72–8.60) | 8.05× (7.76–8.35) | 7.24× (7.00–7.49) | 13.73× (13.22–14.25) | 12.91× (12.44–13.40) | 3.89× (3.75–4.03) | 4.52× (4.35–4.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.154 s | 1,357 | 10.7 | 3.00× (2.90–3.10) | 4.83× (4.61–5.05) | 6.56× (6.03–7.14) | 5.85× (5.65–6.05) | 6.02× (5.59–6.47) | 16.01× (15.47–16.57) | 15.06× (14.60–15.54) | 4.54× (4.40–4.67) | 5.28× (5.13–5.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,316 | 3.5 | 0.56× (0.54–0.59) | 3.36× (3.15–3.57) | 4.68× (4.51–4.86) | 3.74× (3.40–4.11) | 6.39× (6.17–6.61) | 2.74× (2.64–2.85) | 2.73× (2.64–2.83) | 0.99× (0.96–1.02) | 1.98× (1.91–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,282 | 3.7 | 0.51× (0.49–0.53) | 3.28× (3.20–3.36) | 5.79× (5.64–5.94) | 3.56× (3.50–3.63) | 6.27× (6.09–6.45) | 2.73× (2.65–2.80) | 2.71× (2.66–2.77) | 0.98× (0.95–1.01) | 1.97× (1.92–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,231 | 3.8 | 0.49× (0.48–0.51) | 3.12× (2.99–3.25) | 5.54× (5.33–5.76) | 3.47× (3.38–3.56) | 6.10× (5.95–6.25) | 2.64× (2.57–2.71) | 2.63× (2.58–2.68) | 0.95× (0.92–0.99) | 1.91× (1.86–1.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,010 | 4.4 | 2.90× (2.81–3.00) | 3.84× (3.59–4.11) | 4.28× (4.23–4.32) | 4.66× (4.62–4.70) | 5.49× (5.41–5.57) | 3.07× (3.02–3.12) | 2.62× (2.59–2.65) | 2.17× (2.13–2.21) | 3.15× (3.12–3.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,457 | 5.7 | 2.18× (1.89–2.50) | 2.98× (2.52–3.53) | 4.31× (3.58–5.19) | 3.79× (3.26–4.42) | 5.65× (4.92–6.49) | 4.12× (3.59–4.73) | 3.52× (3.06–4.04) | 2.91× (2.55–3.33) | 4.23× (3.68–4.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.016 s | 1,237 | 10.8 | 1.85× (1.77–1.93) | 2.00× (1.89–2.11) | 3.36× (2.99–3.78) | 2.38× (2.22–2.55) | 4.70× (4.35–5.08) | 3.71× (3.53–3.91) | 3.17× (3.01–3.34) | 2.63× (2.50–2.76) | 3.81× (3.62–4.02) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.084 s | 95 | 44.1 | 1.39× (1.37–1.40) | 3.10× (3.03–3.16) | 2.84× (2.77–2.91) | 2.72× (2.64–2.80) | 3.11× (3.05–3.16) | 3.45× (3.36–3.54) | 3.45× (3.36–3.55) | 1.34× (1.32–1.37) | 1.54× (1.49–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 154 | 50.3 | 1.38× (1.23–1.54) | 3.10× (2.77–3.46) | 2.76× (2.63–2.89) | 3.46× (3.16–3.79) | 3.21× (2.98–3.45) | 5.46× (5.12–5.82) | 5.46× (5.12–5.83) | 2.13× (1.99–2.28) | 2.43× (2.27–2.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 184 | 66.2 | 1.35× (1.31–1.39) | 2.57× (2.47–2.68) | 2.55× (2.45–2.65) | 3.01× (2.88–3.15) | 3.38× (3.21–3.56) | 6.85× (6.71–6.99) | 6.85× (6.73–6.98) | 2.67× (2.61–2.73) | 3.05× (2.99–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 38.2 | 1.51× (1.48–1.55) | 3.80× (3.64–3.97) | 3.07× (3.00–3.14) | 3.11× (3.04–3.18) | 3.53× (3.45–3.61) | 4.45× (4.31–4.60) | 4.80× (4.64–4.95) | 1.63× (1.60–1.67) | 1.85× (1.83–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.054 s | 179 | 44.0 | 1.49× (1.29–1.72) | 3.39× (3.27–3.51) | 2.97× (2.80–3.14) | 3.96× (3.84–4.08) | 3.56× (3.35–3.80) | 7.23× (6.95–7.53) | 7.79× (7.40–8.20) | 2.65× (2.56–2.74) | 3.01× (2.90–3.11) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 204 | 63.4 | 1.47× (1.42–1.52) | 2.78× (2.66–2.90) | 2.73× (2.56–2.91) | 3.53× (3.40–3.67) | 3.34× (3.25–3.43) | 8.52× (8.21–8.84) | 9.17× (8.78–9.59) | 3.12× (3.02–3.23) | 3.54× (3.43–3.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 980 | 4.9 | 0.61× (0.60–0.62) | 3.02× (2.94–3.10) | 4.15× (4.03–4.26) | 3.12× (3.01–3.23) | 7.12× (6.96–7.28) | 2.33× (2.29–2.38) | 2.43× (2.18–2.72) | 0.97× (0.95–0.99) | 2.02× (1.97–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 935 | 5.4 | 0.49× (0.48–0.51) | 2.85× (2.79–2.91) | 4.90× (4.79–5.01) | 2.93× (2.85–3.01) | 6.50× (6.36–6.64) | 2.20× (2.16–2.24) | 2.30× (2.06–2.56) | 0.92× (0.89–0.94) | 1.90× (1.86–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 885 | 5.5 | 0.52× (0.42–0.65) | 2.73× (2.68–2.77) | 4.66× (4.58–4.73) | 2.82× (2.75–2.89) | 6.49× (6.09–6.91) | 2.10× (2.06–2.14) | 2.19× (1.97–2.43) | 0.87× (0.85–0.89) | 1.82× (1.79–1.84) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 0.041 s | 294 | 14.6 | 1.88× (1.83–1.93) | 4.51× (4.35–4.68) | 5.09× (4.71–5.49) | 4.93× (4.84–5.02) | 5.43× (5.24–5.63) | 5.33× (5.22–5.44) | 5.09× (5.00–5.17) | 1.63× (1.58–1.69) | 1.87× (1.77–1.97) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 0.030 s | 400 | 19.2 | 1.46× (1.31–1.63) | 3.52× (3.29–3.77) | 4.28× (4.08–4.48) | 4.48× (4.17–4.82) | 4.46× (4.15–4.79) | 7.10× (6.66–7.58) | 6.78× (6.40–7.18) | 2.18× (2.04–2.32) | 2.49× (2.36–2.63) | yes |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 0.028 s | 425 | 29.1 | 1.30× (1.26–1.33) | 2.40× (2.24–2.56) | 3.36× (3.18–3.56) | 3.53× (3.40–3.66) | 3.86× (3.72–4.01) | 7.64× (7.41–7.88) | 7.30× (7.11–7.49) | 2.34× (2.26–2.43) | 2.68× (2.56–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.79–2.95) | 7.10× (6.86–7.35) | 7.53× (7.28–7.78) | 8.54× (8.30–8.79) | 7.57× (7.34–7.82) | 7.84× (7.58–8.12) | 7.38× (7.11–7.65) | 2.23× (2.16–2.30) | 2.57× (2.48–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.74–2.95) | 6.89× (6.69–7.09) | 7.49× (7.21–7.79) | 8.40× (8.27–8.52) | 7.14× (6.98–7.32) | 7.23× (7.07–7.40) | 6.80× (6.67–6.94) | 2.05× (2.01–2.09) | 2.37× (2.32–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.88× (2.76–3.01) | 5.30× (5.10–5.50) | 6.18× (5.58–6.86) | 6.48× (6.23–6.75) | 5.83× (5.31–6.39) | 4.55× (4.46–4.64) | 4.28× (4.20–4.36) | 1.29× (1.26–1.32) | 1.49× (1.46–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.58–0.63) | 2.99× (2.87–3.11) | 3.84× (3.74–3.95) | 3.46× (3.14–3.82) | 5.85× (5.64–6.07) | 2.57× (2.47–2.67) | 2.56× (2.47–2.65) | 0.99× (0.95–1.02) | 1.79× (1.74–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.54–0.57) | 2.71× (2.65–2.78) | 4.78× (4.68–4.89) | 3.20× (3.14–3.27) | 5.53× (5.37–5.69) | 2.45× (2.39–2.52) | 2.44× (2.39–2.49) | 0.94× (0.92–0.97) | 1.71× (1.67–1.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.53) | 2.88× (2.82–2.95) | 4.70× (4.60–4.80) | 3.18× (3.10–3.26) | 5.47× (5.34–5.60) | 2.43× (2.37–2.49) | 2.41× (2.37–2.46) | 0.93× (0.90–0.96) | 1.69× (1.66–1.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.75× (2.66–2.84) | 3.38× (3.06–3.72) | 3.79× (3.73–3.85) | 4.37× (4.33–4.41) | 5.13× (5.06–5.21) | 2.90× (2.86–2.95) | 2.49× (2.46–2.52) | 2.08× (2.04–2.12) | 2.80× (2.76–2.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.14–2.36) | 2.71× (2.53–2.89) | 3.17× (2.99–3.35) | 3.60× (3.34–3.88) | 4.00× (3.77–4.25) | 2.20× (2.09–2.32) | 1.89× (1.79–1.99) | 1.57× (1.50–1.65) | 2.12× (2.01–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.09× (2.01–2.17) | 1.85× (1.76–1.95) | 2.31× (2.06–2.59) | 2.14× (2.03–2.26) | 2.43× (2.30–2.57) | 1.16× (1.11–1.21) | 0.99× (0.95–1.04) | 0.83× (0.80–0.85) | 1.12× (1.07–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.98× (2.93–3.04) | 2.76× (2.68–2.84) | 2.69× (2.61–2.77) | 3.07× (3.01–3.12) | 3.41× (3.32–3.50) | 3.41× (3.32–3.50) | 1.34× (1.31–1.37) | 1.45× (1.41–1.49) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.28–1.32) | 2.82× (2.77–2.87) | 2.47× (2.42–2.51) | 3.17× (3.07–3.28) | 2.80× (2.68–2.93) | 2.96× (2.87–3.05) | 2.97× (2.88–3.05) | 1.16× (1.13–1.20) | 1.26× (1.22–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.43–1.51) | 2.73× (2.62–2.85) | 2.26× (2.19–2.34) | 2.89× (2.81–2.96) | 2.78× (2.67–2.89) | 2.28× (2.21–2.35) | 2.28× (2.22–2.34) | 0.89× (0.87–0.92) | 0.97× (0.94–1.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.47–1.54) | 3.70× (3.55–3.86) | 2.98× (2.92–3.05) | 3.07× (3.00–3.15) | 3.49× (3.41–3.57) | 4.39× (4.25–4.54) | 4.73× (4.58–4.88) | 1.62× (1.59–1.66) | 1.78× (1.74–1.81) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.37–1.43) | 3.32× (3.19–3.45) | 2.69× (2.52–2.87) | 3.67× (3.57–3.78) | 3.12× (2.94–3.32) | 3.77× (3.62–3.93) | 4.06× (3.86–4.27) | 1.39× (1.35–1.44) | 1.53× (1.48–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.52–1.62) | 2.97× (2.89–3.05) | 2.30× (2.13–2.48) | 3.10× (3.05–3.16) | 2.83× (2.73–2.93) | 2.66× (2.59–2.73) | 2.86× (2.76–2.97) | 0.98× (0.96–1.00) | 1.08× (1.05–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.68) | 2.55× (2.47–2.63) | 3.37× (3.26–3.47) | 2.84× (2.73–2.95) | 6.28× (6.14–6.42) | 2.16× (2.11–2.21) | 2.25× (2.01–2.53) | 0.98× (0.96–1.00) | 1.49× (1.45–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.55–0.56) | 2.19× (2.14–2.24) | 3.72× (3.65–3.79) | 2.58× (2.51–2.65) | 5.44× (5.33–5.55) | 1.92× (1.89–1.95) | 2.01× (1.79–2.24) | 0.87× (0.85–0.89) | 1.33× (1.30–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.46–0.71) | 2.29× (2.24–2.34) | 3.68× (3.61–3.75) | 2.56× (2.49–2.63) | 5.63× (5.27–6.02) | 1.90× (1.87–1.93) | 1.98× (1.78–2.21) | 0.86× (0.84–0.88) | 1.31× (1.29–1.34) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 1.84× (1.79–1.89) | 4.25× (4.12–4.38) | 4.79× (4.44–5.16) | 4.77× (4.68–4.86) | 5.25× (5.07–5.44) | 5.16× (5.05–5.26) | 4.92× (4.84–5.00) | 1.61× (1.55–1.66) | 1.74× (1.67–1.81) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 1.50× (1.46–1.54) | 3.45× (3.31–3.61) | 3.77× (3.67–3.86) | 4.24× (4.15–4.33) | 4.02× (3.93–4.12) | 3.94× (3.85–4.03) | 3.76× (3.66–3.87) | 1.23× (1.17–1.29) | 1.33× (1.27–1.39) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 1.55× (1.51–1.58) | 2.76× (2.55–2.98) | 3.03× (2.87–3.19) | 3.38× (3.29–3.48) | 3.28× (3.19–3.38) | 2.56× (2.48–2.64) | 2.44× (2.37–2.51) | 0.80× (0.76–0.83) | 0.86× (0.84–0.89) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.34× (0.33–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.80) | 11.00× (10.96–11.05) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.37) | 0.48× (0.40–0.58) | 0.72× (0.69–0.75) | 0.61× (0.58–0.64) | 5.73× (5.49–5.97) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.36–0.39) | 0.60× (0.49–0.73) | 0.67× (0.64–0.71) | 0.59× (0.56–0.61) | 3.90× (3.74–4.06) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.23–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.99× (7.93–8.04) | 0.65× (0.64–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.71× (0.68–0.74) | 0.73× (0.69–0.78) | 1.66× (1.60–1.72) | 0.83× (0.80–0.86) | 6.00× (5.78–6.22) | 0.49× (0.47–0.50) | 0.51× (0.49–0.53) | 0.56× (0.54–0.58) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.66× (0.62–0.69) | 0.76× (0.72–0.81) | 1.19× (1.14–1.25) | 0.74× (0.70–0.77) | 3.48× (3.32–3.64) | 0.27× (0.26–0.29) | 0.29× (0.27–0.30) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.14× (5.12–5.15) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.40× (0.37–0.44) | 0.90× (0.88–0.93) | 0.72× (0.69–0.75) | 3.12× (3.03–3.21) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.65) | 0.46× (0.45–0.46) | 0.88× (0.86–0.89) | 0.82× (0.81–0.83) | 2.54× (2.50–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.19) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.84× (0.81–0.87) | 0.71× (0.68–0.74) | 2.88× (2.79–2.98) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.60–0.61) | 0.45× (0.43–0.48) | 0.79× (0.78–0.81) | 0.80× (0.77–0.82) | 2.29× (2.26–2.31) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.20–0.23) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.30× (5.30–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.90) | 6.19× (6.16–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.89–0.91) | 6.20× (6.18–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.07) | 0.15× (0.15–0.16) |
| `text` (2.37 MB → 12 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.65× (0.65–0.66) | 6.35× (6.32–6.38) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.37 MB → 12 MB, 5.1×) | 2 | 33 MB | 0.52× (0.51–0.53) | 0.45× (0.42–0.48) | 1.18× (1.15–1.21) | 0.76× (0.74–0.78) | 3.76× (3.70–3.83) | 0.23× (0.23–0.23) | 0.25× (0.24–0.26) | 0.38× (0.37–0.38) | 0.19× (0.18–0.19) |
| `text` (2.37 MB → 12 MB, 5.1×) | 4 | 51 MB | 0.55× (0.54–0.56) | 0.48× (0.44–0.52) | 0.99× (0.98–1.01) | 0.80× (0.77–0.82) | 2.54× (2.49–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 684 | 1.47 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,165 | 1.64 | 1.08× (1.06–1.11) | 87% | 1.05× (1.02–1.08) | 1.08× (1.03–1.13) |
| 4: all 4 CPUs, sharing SMT siblings | 1,357 | 2.60 | 1.72× (1.66–1.79) | 51% | 1.29× (1.22–1.35) | 1.73× (1.65–1.82) |

**Best:** mibz2 at 4 threads (1,357 MB/s); lbzip2 at 4 threads (292 MB/s); pbz2 at 4 threads (456 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.47, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.52. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.41 | 0.01 | 0.08 | 0.154 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4114 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 41.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 11.4% |
| MTF + RLE2 | 1.46 | 20.0% |
| T-vector build | 1.34 | 18.3% |
| walk (the one each build uses) | 3.26 | 44.5% |
| of which: segment assembly | 0.15 | 2.0% |
| RLE1 | 0.16 | 2.1% |
| CRC | 0.26 | 3.5% |
| **total** | 7.32 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.312 | 0.749 | (in its total) | 2.37× (2.32–2.43) |  |
| walk | 0.693 | 1.668 | (in its total) | 2.40× (2.31–2.49) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.034 | 0.033 | (in its total) | 0.99× (0.92–1.07) |  |
| inverse BWT, total | 1.041 | 2.450 | 1.383 | 2.35× (2.27–2.42) | 1.32× (1.28–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m42s ± 5m40s |
| 4 threads | 22m30s ± 3m15s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m53s ± 2m09s |
| the scanner alone (its serial part) | 7m08s ± 19.7s |
| the workers alone, over 4 | 14m53s ± 2m09s |

At 4 threads the workers bound it. Per byte: the scanner 4.1 ns a byte of bz2, the workers 1.95 ns a byte of output at 4 threads (CPU, summed over them), 1.26 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,367,452 | 12,009,010 | \`eb0f47ca286822c7…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4113.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
