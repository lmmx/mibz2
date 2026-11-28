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

The latest record: [`9875c71`](<https://github.com/lmmx/mibz2/commit/9875c7129e2dfd75ffd09234050956e6e7fbc753>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46095457797>)

Intel(R) Xeon(R) 6973P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.374 s | 560 | 1.4 | 3.55× (3.49–3.62) | 8.42× (8.30–8.55) | 8.67× (8.57–8.76) | 9.48× (9.33–9.62) | 9.19× (9.05–9.33) | 9.16× (9.06–9.26) | 9.07× (8.87–9.28) | 2.71× (2.65–2.77) | 2.47× (2.42–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.206 s | 1,018 | 1.5 | 3.45× (3.28–3.64) | 7.68× (7.33–8.04) | 9.11× (8.81–9.43) | 9.41× (8.89–9.96) | 8.62× (8.17–9.10) | 16.28× (15.42–17.18) | 16.12× (15.28–17.00) | 4.82× (4.57–5.08) | 4.40× (4.19–4.62) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.175 s | 1,199 | 2.5 | 3.62× (3.56–3.69) | 6.78× (6.56–7.00) | 7.69× (7.37–8.04) | 7.59× (7.26–7.93) | 7.68× (7.40–7.98) | 19.61× (18.62–20.66) | 19.42× (18.49–20.40) | 5.80× (5.56–6.06) | 5.30× (5.14–5.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 0.7 | 0.61× (0.56–0.68) | 3.49× (3.14–3.88) | 4.71× (4.35–5.09) | 3.94× (3.61–4.29) | 6.97× (6.43–7.55) | 2.98× (2.73–3.25) | 2.94× (2.69–3.22) | 1.21× (1.13–1.29) | 1.36× (1.24–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,093 | 0.8 | 0.50× (0.47–0.52) | 2.86× (2.69–3.05) | 5.96× (5.72–6.22) | 3.39× (3.22–3.56) | 6.04× (5.76–6.33) | 2.52× (2.37–2.68) | 2.49× (2.34–2.65) | 1.02× (0.96–1.09) | 1.15× (1.09–1.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,114 | 0.8 | 0.50× (0.47–0.52) | 2.89× (2.70–3.09) | 5.59× (5.32–5.88) | 3.26× (3.07–3.47) | 5.69× (5.40–6.00) | 2.50× (2.35–2.66) | 2.47× (2.35–2.60) | 1.01× (0.96–1.07) | 1.15× (1.09–1.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 637 | 1.3 | 2.55× (2.44–2.67) | 3.05× (2.92–3.17) | 3.74× (3.60–3.89) | 4.00× (3.85–4.15) | 4.62× (4.45–4.80) | 2.94× (2.83–3.05) | 2.75× (2.62–2.88) | 2.02× (1.96–2.08) | 2.04× (1.95–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,032 | 1.6 | 2.36× (2.18–2.54) | 3.13× (2.79–3.51) | 4.38× (3.96–4.86) | 3.80× (3.52–4.10) | 5.51× (5.12–5.94) | 4.86× (4.53–5.22) | 4.55× (4.16–4.97) | 3.34× (3.08–3.61) | 3.38× (3.09–3.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,051 | 2.6 | 2.01× (1.87–2.17) | 2.14× (1.92–2.38) | 3.28× (2.98–3.60) | 2.49× (2.32–2.67) | 4.40× (4.17–4.65) | 4.66× (4.42–4.92) | 4.36× (4.20–4.53) | 3.20× (3.05–3.35) | 3.24× (3.12–3.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.099 s | 81 | 10.0 | 1.53× (1.51–1.54) | 3.43× (3.33–3.53) | 3.17× (3.07–3.28) | 3.30× (3.21–3.39) | 3.63× (3.52–3.73) | 3.97× (3.85–4.10) | 4.16× (4.05–4.27) | 1.41× (1.35–1.46) | 1.36× (1.33–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.057 s | 140 | 10.9 | 1.50× (1.46–1.55) | 3.61× (3.19–4.08) | 3.26× (3.11–3.41) | 4.05× (3.94–4.16) | 3.70× (3.55–3.84) | 6.81× (6.50–7.13) | 7.13× (6.74–7.54) | 2.41× (2.25–2.58) | 2.33× (2.18–2.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.048 s | 168 | 15.0 | 1.56× (1.48–1.64) | 3.12× (2.95–3.30) | 3.08× (2.96–3.20) | 3.80× (3.62–4.00) | 3.78× (3.67–3.90) | 8.36× (7.92–8.83) | 8.76× (8.35–9.19) | 2.96× (2.77–3.16) | 2.87× (2.69–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.112 s | 86 | 9.4 | 1.54× (1.50–1.59) | 3.78× (3.68–3.88) | 3.24× (3.16–3.33) | 3.37× (3.26–3.48) | 3.74× (3.60–3.88) | 4.49× (4.33–4.67) | 4.78× (4.65–4.92) | 1.57× (1.55–1.59) | 1.62× (1.60–1.64) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.065 s | 149 | 10.3 | 1.44× (1.35–1.53) | 3.50× (3.29–3.72) | 3.26× (3.04–3.49) | 4.26× (4.01–4.52) | 3.73× (3.53–3.94) | 7.73× (7.13–8.37) | 8.22× (7.69–8.79) | 2.70× (2.50–2.91) | 2.78× (2.59–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.054 s | 178 | 14.5 | 1.57× (1.54–1.59) | 3.42× (3.28–3.58) | 3.06× (2.92–3.20) | 4.04× (3.92–4.16) | 3.67× (3.55–3.79) | 9.35× (8.84–9.88) | 9.95× (9.47–10.45) | 3.27× (3.04–3.51) | 3.37× (3.16–3.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,000 | 0.9 | 0.66× (0.61–0.72) | 3.13× (2.82–3.47) | 4.44× (3.99–4.93) | 3.30× (3.00–3.63) | 7.59× (6.99–8.23) | 2.53× (2.31–2.77) | 2.55× (2.34–2.79) | 1.10× (1.04–1.16) | 1.07× (0.97–1.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 837 | 1.1 | 0.52× (0.46–0.58) | 2.66× (2.48–2.86) | 5.58× (5.19–6.00) | 2.87× (2.66–3.10) | 6.51× (6.09–6.96) | 2.15× (2.02–2.28) | 2.17× (2.05–2.30) | 0.93× (0.88–0.99) | 0.91× (0.85–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 844 | 1.1 | 0.50× (0.47–0.53) | 2.63× (2.49–2.77) | 5.15× (4.82–5.51) | 2.94× (2.73–3.17) | 6.40× (6.09–6.72) | 2.12× (2.00–2.25) | 2.14× (2.01–2.28) | 0.92× (0.87–0.98) | 0.90× (0.85–0.94) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.054 s | 278 | 3.0 | 2.08× (2.03–2.14) | 4.44× (4.30–4.58) | 5.04× (4.89–5.21) | 4.99× (4.87–5.11) | 5.33× (5.24–5.43) | 5.26× (5.09–5.43) | 5.22× (5.09–5.35) | 1.66× (1.64–1.69) | 1.69× (1.65–1.73) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.036 s | 416 | 3.7 | 1.66× (1.59–1.73) | 3.82× (3.38–4.31) | 4.61× (4.40–4.83) | 4.92× (4.67–5.18) | 4.94× (4.75–5.12) | 7.88× (7.47–8.32) | 7.82× (7.41–8.25) | 2.49× (2.39–2.60) | 2.53× (2.41–2.65) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.032 s | 473 | 5.7 | 1.69× (1.58–1.82) | 3.00× (2.80–3.22) | 3.98× (3.69–4.28) | 4.20× (3.90–4.53) | 4.65× (4.40–4.91) | 9.06× (8.49–9.68) | 9.00× (8.29–9.76) | 2.87× (2.67–3.08) | 2.91× (2.71–3.12) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.54× (3.48–3.61) | 8.39× (8.26–8.51) | 8.62× (8.53–8.72) | 9.45× (9.30–9.59) | 9.16× (9.02–9.30) | 9.13× (9.02–9.23) | 9.04× (8.84–9.25) | 2.70× (2.64–2.77) | 2.46× (2.40–2.51) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.38× (3.33–3.43) | 8.03× (7.95–8.10) | 8.61× (8.47–8.76) | 9.84× (9.50–10.18) | 8.72× (8.57–8.88) | 8.64× (8.42–8.87) | 8.56× (8.35–8.78) | 2.56× (2.51–2.61) | 2.33× (2.29–2.36) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.44× (3.35–3.53) | 7.14× (6.90–7.38) | 7.33× (7.00–7.66) | 8.03× (7.66–8.41) | 7.40× (7.14–7.68) | 5.29× (5.04–5.55) | 5.24× (5.01–5.47) | 1.57× (1.50–1.63) | 1.42× (1.38–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.58–0.71) | 3.00× (2.74–3.29) | 4.13× (3.81–4.47) | 3.73× (3.44–4.05) | 6.55× (6.07–7.07) | 2.85× (2.62–3.09) | 2.81× (2.58–3.06) | 1.20× (1.13–1.28) | 0.89× (0.82–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.50–0.55) | 2.55× (2.44–2.67) | 4.96× (4.73–5.21) | 3.15× (3.00–3.31) | 5.54× (5.28–5.82) | 2.35× (2.21–2.50) | 2.32× (2.18–2.47) | 0.99× (0.93–1.06) | 0.74× (0.69–0.80) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.50–0.55) | 2.95× (2.77–3.15) | 4.75× (4.50–5.02) | 3.07× (2.89–3.26) | 5.26× (4.99–5.54) | 2.36× (2.22–2.50) | 2.33× (2.21–2.45) | 0.99× (0.94–1.06) | 0.74× (0.69–0.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.49× (2.38–2.61) | 2.91× (2.81–3.02) | 3.44× (3.32–3.57) | 3.88× (3.73–4.04) | 4.48× (4.30–4.67) | 2.87× (2.76–2.98) | 2.68× (2.55–2.81) | 1.98× (1.92–2.04) | 1.88× (1.82–1.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.30× (2.15–2.46) | 2.66× (2.48–2.84) | 3.27× (3.03–3.52) | 3.46× (3.23–3.70) | 3.99× (3.73–4.27) | 2.46× (2.30–2.64) | 2.30× (2.11–2.52) | 1.70× (1.57–1.84) | 1.61× (1.50–1.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.15× (2.03–2.27) | 1.93× (1.79–2.08) | 2.50× (2.36–2.66) | 2.40× (2.30–2.50) | 2.54× (2.44–2.65) | 1.43× (1.38–1.47) | 1.33× (1.27–1.40) | 0.98× (0.94–1.03) | 0.93× (0.89–0.98) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.52× (1.51–1.53) | 3.35× (3.26–3.44) | 3.12× (3.02–3.23) | 3.27× (3.19–3.36) | 3.60× (3.50–3.70) | 3.94× (3.82–4.06) | 4.13× (4.02–4.24) | 1.40× (1.35–1.46) | 1.32× (1.32–1.33) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.47× (1.44–1.49) | 3.23× (3.16–3.31) | 2.94× (2.87–3.01) | 3.69× (3.58–3.80) | 3.26× (3.15–3.38) | 3.61× (3.48–3.74) | 3.78× (3.61–3.96) | 1.28× (1.21–1.36) | 1.21× (1.15–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.68× (1.58–1.78) | 3.19× (3.04–3.35) | 2.78× (2.67–2.89) | 3.60× (3.49–3.71) | 3.27× (3.16–3.38) | 2.67× (2.53–2.81) | 2.79× (2.66–2.93) | 0.95× (0.89–1.01) | 0.90× (0.85–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.49–1.58) | 3.70× (3.61–3.79) | 3.17× (3.09–3.26) | 3.35× (3.24–3.45) | 3.71× (3.57–3.85) | 4.46× (4.29–4.63) | 4.74× (4.61–4.87) | 1.57× (1.54–1.59) | 1.54× (1.53–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.46× (1.43–1.48) | 3.49× (3.39–3.59) | 3.06× (2.95–3.17) | 4.01× (3.91–4.11) | 3.44× (3.32–3.56) | 4.10× (3.84–4.37) | 4.36× (4.14–4.59) | 1.44× (1.36–1.52) | 1.42× (1.34–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.68× (1.61–1.76) | 3.46× (3.34–3.58) | 2.73× (2.63–2.84) | 3.76× (3.63–3.90) | 3.27× (3.12–3.42) | 2.91× (2.74–3.09) | 3.10× (2.93–3.28) | 1.02× (0.95–1.11) | 1.01× (0.94–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.65–0.76) | 2.61× (2.38–2.86) | 3.70× (3.37–4.07) | 3.07× (2.80–3.36) | 6.90× (6.36–7.48) | 2.38× (2.18–2.59) | 2.40× (2.20–2.60) | 1.09× (1.04–1.15) | 0.83× (0.77–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.51–0.63) | 2.18× (2.03–2.34) | 4.22× (3.93–4.52) | 2.65× (2.46–2.85) | 5.78× (5.39–6.19) | 1.96× (1.84–2.09) | 1.98× (1.87–2.09) | 0.90× (0.85–0.96) | 0.69× (0.65–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.51–0.55) | 2.55× (2.43–2.68) | 4.09× (3.93–4.26) | 2.73× (2.54–2.92) | 5.70× (5.44–5.97) | 1.94× (1.84–2.04) | 1.96× (1.84–2.08) | 0.89× (0.84–0.95) | 0.68× (0.65–0.72) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 2.06× (2.01–2.12) | 4.29× (4.15–4.44) | 4.86× (4.70–5.02) | 4.91× (4.79–5.03) | 5.24× (5.14–5.34) | 5.17× (5.01–5.33) | 5.13× (5.00–5.26) | 1.65× (1.63–1.67) | 1.53× (1.51–1.56) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.74× (1.68–1.81) | 3.62× (3.47–3.78) | 4.23× (4.04–4.42) | 4.51× (4.31–4.71) | 4.40× (4.23–4.57) | 4.21× (3.97–4.46) | 4.17× (3.94–4.42) | 1.34× (1.28–1.41) | 1.25× (1.19–1.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.83× (1.73–1.93) | 3.17× (3.03–3.32) | 3.50× (3.33–3.69) | 3.77× (3.56–3.99) | 3.70× (3.55–3.86) | 2.73× (2.58–2.90) | 2.71× (2.52–2.92) | 0.87× (0.82–0.93) | 0.81× (0.76–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.29× (0.28–0.29) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.07× (11.04–11.10) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.34–0.37) | 0.43× (0.37–0.50) | 0.74× (0.72–0.77) | 0.62× (0.60–0.64) | 5.86× (5.66–6.06) | 0.14× (0.14–0.15) | 0.15× (0.14–0.15) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.35–0.38) | 0.49× (0.43–0.56) | 0.65× (0.61–0.68) | 0.57× (0.55–0.60) | 3.81× (3.66–3.97) | 0.09× (0.09–0.09) | 0.09× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.69–1.71) | 0.66× (0.66–0.67) | 8.03× (7.99–8.07) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.76) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.68–0.73) | 0.64× (0.57–0.73) | 1.69× (1.63–1.75) | 0.84× (0.81–0.88) | 6.15× (5.94–6.37) | 0.50× (0.48–0.52) | 0.53× (0.51–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.67× (0.66–0.69) | 0.71× (0.66–0.78) | 1.19× (1.16–1.23) | 0.74× (0.71–0.77) | 3.56× (3.46–3.66) | 0.28× (0.27–0.29) | 0.29× (0.29–0.30) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.74) | 5.19× (5.17–5.20) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.37× (0.34–0.41) | 0.91× (0.89–0.94) | 0.72× (0.70–0.74) | 3.16× (3.08–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.46) | 0.87× (0.86–0.88) | 0.81× (0.79–0.83) | 2.54× (2.51–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.48) | 0.35× (0.33–0.38) | 0.84× (0.82–0.86) | 0.66× (0.64–0.69) | 2.88× (2.80–2.95) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.46× (0.43–0.50) | 0.80× (0.78–0.81) | 0.80× (0.78–0.83) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.18–6.24) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.23) | 1.86× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.16–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.66× (0.65–0.66) | 6.41× (6.39–6.42) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.64) | 0.30× (0.30–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.41× (0.37–0.45) | 1.10× (1.05–1.14) | 0.74× (0.70–0.78) | 3.61× (3.48–3.74) | 0.22× (0.21–0.23) | 0.24× (0.23–0.24) | 0.36× (0.34–0.37) | 0.17× (0.17–0.18) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 54 MB | 0.52× (0.51–0.53) | 0.46× (0.43–0.49) | 0.94× (0.93–0.96) | 0.76× (0.74–0.78) | 2.42× (2.38–2.47) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 560 | 1.79 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,018 | 1.91 | 1.06× (1.04–1.08) | 89% | 1.01× (1.00–1.02) | 1.01× (1.00–1.02) |
| 4: all 4 CPUs, sharing SMT siblings | 1,199 | 3.07 | 1.73× (1.65–1.80) | 54% | 1.47× (1.45–1.49) | 1.68× (1.61–1.74) |

**Best:** mibz2 at 4 threads (1,199 MB/s); lbzip2 at 4 threads (176 MB/s); pbz2 at 4 threads (332 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.05, lbzip2 1t 0.04, lbzip2 2t 0.18, lbzip2 4t 0.25, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 13%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.50 | 0.02 | 0.08 | 0.175 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.17 | 7.7% |
| MTF + RLE2 | 0.35 | 15.8% |
| T-vector build | 0.31 | 14.2% |
| walk (the one each build uses) | 1.29 | 58.5% |
| of which: segment assembly | 0.06 | 2.9% |
| RLE1 | 0.05 | 2.4% |
| CRC | 0.03 | 1.3% |
| **total** | 2.21 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.352 | 0.876 | (in its total) | 2.49× (2.46–2.52) |  |
| walk | 1.508 | 2.918 | (in its total) | 1.93× (1.91–1.95) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.074 | (in its total) | 0.92× (0.92–0.93) |  |
| inverse BWT, total | 1.938 | 3.868 | 1.940 | 1.99× (1.97–2.01) | 1.01× (1.00–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m51s ± 6m41s |
| 4 threads | 25m38s ± 3m43s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m18s ± 2m39s |
| the scanner alone (its serial part) | 7m34s ± 26.3s |
| the workers alone, over 4 | 18m18s ± 2m39s |

At 4 threads the workers bound it. Per byte: the scanner 4.4 ns a byte of bz2, the workers 2.40 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,228,219 | 14,890,034 | \`9ad9de226af45720…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6973P-C, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
