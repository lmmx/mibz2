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

The latest record: [`a44531c`](<https://github.com/lmmx/mibz2/commit/a44531c810be79114feed9a19ab554613ae756cf>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40921330569>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.800 s | 262 | 10.0 | 1.89× (1.86–1.92) | 4.36× (4.28–4.44) | 4.60× (4.52–4.68) | 4.88× (4.79–4.97) | 4.45× (4.36–4.53) | 4.71× (4.63–4.79) | 4.42× (4.35–4.50) | 1.39× (1.37–1.41) | 1.65× (1.62–1.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.519 s | 404 | 11.1 | 1.66× (1.62–1.70) | 3.54× (3.47–3.62) | 4.12× (4.01–4.23) | 4.03× (3.93–4.13) | 3.67× (3.58–3.75) | 7.26× (7.09–7.42) | 6.82× (6.66–6.97) | 2.14× (2.09–2.19) | 2.54× (2.49–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.412 s | 509 | 17.1 | 1.70× (1.67–1.72) | 2.66× (2.64–2.69) | 3.28× (3.23–3.33) | 2.94× (2.89–2.99) | 3.00× (2.96–3.04) | 9.09× (8.92–9.27) | 8.54× (8.37–8.72) | 2.68× (2.63–2.73) | 3.19× (3.13–3.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 893 | 3.2 | 0.56× (0.56–0.56) | 3.66× (3.62–3.70) | 4.51× (4.46–4.56) | 3.31× (3.27–3.35) | 6.37× (6.29–6.44) | 2.60× (2.57–2.63) | 2.45× (2.43–2.47) | 1.00× (0.99–1.02) | 1.38× (1.36–1.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 905 | 3.2 | 0.56× (0.55–0.56) | 2.79× (2.76–2.81) | 5.97× (5.91–6.03) | 3.37× (3.33–3.41) | 6.37× (6.29–6.45) | 2.63× (2.60–2.66) | 2.48× (2.45–2.51) | 1.02× (1.00–1.03) | 1.40× (1.38–1.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 875 | 3.2 | 0.55× (0.53–0.56) | 2.73× (2.66–2.79) | 5.82× (5.69–5.96) | 3.28× (3.20–3.35) | 6.29× (6.15–6.45) | 2.56× (2.50–2.62) | 2.41× (2.36–2.47) | 0.99× (0.96–1.02) | 1.36× (1.33–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.046 s | 429 | 6.4 | 1.99× (1.94–2.04) | 2.28× (2.22–2.34) | 2.88× (2.77–2.99) | 2.89× (2.81–2.96) | 3.47× (3.37–3.58) | 1.89× (1.83–1.94) | 1.60× (1.55–1.66) | 1.53× (1.49–1.57) | 2.27× (2.21–2.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 s | 691 | 7.7 | 1.72× (1.56–1.89) | 2.18× (1.98–2.39) | 3.20× (2.94–3.47) | 2.61× (2.37–2.87) | 4.12× (3.81–4.45) | 2.85× (2.64–3.08) | 2.43× (2.25–2.62) | 2.31× (2.14–2.49) | 3.43× (3.18–3.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.032 s | 621 | 14.5 | 1.46× (1.42–1.49) | 1.69× (1.63–1.75) | 2.41× (2.29–2.53) | 1.79× (1.73–1.84) | 3.61× (3.53–3.69) | 2.66× (2.59–2.72) | 2.26× (2.21–2.31) | 2.15× (2.10–2.20) | 3.19× (3.12–3.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.155 s | 52 | 51.5 | 1.14× (1.11–1.17) | 2.34× (2.27–2.41) | 2.09× (2.03–2.14) | 2.12× (2.06–2.18) | 2.52× (2.45–2.58) | 2.61× (2.55–2.68) | 2.65× (2.58–2.72) | 1.03× (1.00–1.06) | 1.17× (1.14–1.20) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.094 s | 85 | 57.0 | 1.11× (1.09–1.13) | 2.20× (2.13–2.28) | 2.12× (2.06–2.17) | 2.65× (2.59–2.71) | 2.71× (2.66–2.77) | 4.33× (4.23–4.43) | 4.39× (4.29–4.48) | 1.70× (1.66–1.74) | 1.94× (1.89–1.98) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 105 | 76.4 | 1.19× (1.17–1.22) | 1.91× (1.86–1.96) | 1.87× (1.82–1.92) | 2.30× (2.25–2.36) | 2.75× (2.71–2.78) | 5.33× (5.22–5.44) | 5.40× (5.29–5.51) | 2.10× (2.06–2.14) | 2.38× (2.34–2.44) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.204 s | 47 | 56.3 | 1.05× (1.05–1.06) | 2.28× (2.26–2.30) | 1.86× (1.84–1.88) | 1.91× (1.89–1.93) | 2.31× (2.24–2.38) | 2.69× (2.67–2.72) | 2.77× (2.76–2.79) | 0.99× (0.98–1.00) | 1.15× (1.14–1.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.120 s | 80 | 62.1 | 1.01× (0.97–1.05) | 2.09× (2.00–2.18) | 1.83× (1.77–1.89) | 2.57× (2.47–2.66) | 2.34× (2.25–2.43) | 4.49× (4.34–4.65) | 4.62× (4.46–4.79) | 1.65× (1.59–1.72) | 1.92× (1.85–2.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.095 s | 102 | 82.5 | 1.17× (1.15–1.19) | 1.89× (1.85–1.94) | 1.84× (1.80–1.88) | 2.49× (2.44–2.54) | 2.45× (2.41–2.49) | 5.81× (5.72–5.90) | 5.99× (5.90–6.08) | 2.14× (2.11–2.18) | 2.49× (2.44–2.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 652 | 4.5 | 0.61× (0.60–0.61) | 3.14× (2.90–3.39) | 3.94× (3.86–4.02) | 2.82× (2.78–2.87) | 7.12× (7.03–7.21) | 2.21× (2.18–2.25) | 2.09× (2.07–2.12) | 0.97× (0.96–0.97) | 1.35× (1.33–1.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 639 | 4.8 | 0.55× (0.53–0.56) | 2.64× (2.57–2.71) | 5.12× (4.89–5.36) | 2.80× (2.73–2.87) | 6.97× (6.82–7.14) | 2.17× (2.12–2.22) | 2.05× (2.00–2.11) | 0.95× (0.93–0.97) | 1.33× (1.29–1.36) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 633 | 4.7 | 0.54× (0.53–0.56) | 2.62× (2.58–2.66) | 5.02× (4.78–5.27) | 2.77× (2.72–2.83) | 6.86× (6.72–6.99) | 2.15× (2.12–2.18) | 2.04× (2.01–2.06) | 0.94× (0.92–0.96) | 1.31× (1.29–1.34) | yes |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 1 | 0.091 s | 136 | 19.7 | 1.32× (1.30–1.35) | 2.79× (2.74–2.84) | 3.13× (3.05–3.20) | 3.03× (2.98–3.08) | 3.40× (3.34–3.46) | 3.25× (3.20–3.31) | 3.11× (3.06–3.16) | 0.99× (0.98–1.01) | 1.25× (1.22–1.27) | yes |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 2 | 0.056 s | 221 | 22.6 | 1.17× (1.13–1.22) | 2.51× (2.35–2.68) | 3.04× (2.88–3.20) | 3.12× (3.02–3.23) | 3.49× (3.36–3.62) | 5.23× (5.03–5.43) | 4.99× (4.82–5.18) | 1.60× (1.54–1.66) | 2.00× (1.91–2.09) | yes |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 4 | 0.049 s | 253 | 33.7 | 1.22× (1.16–1.28) | 1.95× (1.88–2.03) | 2.51× (2.45–2.57) | 2.61× (2.56–2.66) | 3.19× (3.14–3.23) | 6.05× (5.94–6.16) | 5.78× (5.67–5.89) | 1.85× (1.80–1.89) | 2.31× (2.25–2.38) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.89× (1.86–1.92) | 4.35× (4.27–4.43) | 4.58× (4.50–4.66) | 4.87× (4.78–4.96) | 4.44× (4.36–4.53) | 4.70× (4.62–4.78) | 4.41× (4.34–4.49) | 1.39× (1.37–1.40) | 1.64× (1.61–1.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.78× (1.76–1.81) | 4.03× (3.97–4.09) | 4.32× (4.24–4.39) | 4.66× (4.59–4.74) | 4.05× (3.99–4.12) | 4.27× (4.21–4.34) | 4.02× (3.95–4.08) | 1.26× (1.24–1.28) | 1.49× (1.47–1.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.84× (1.82–1.86) | 3.11× (3.08–3.15) | 3.33× (3.29–3.38) | 3.47× (3.42–3.52) | 3.07× (3.04–3.11) | 2.74× (2.69–2.79) | 2.58× (2.53–2.63) | 0.81× (0.80–0.82) | 0.96× (0.94–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.60) | 3.07× (3.04–3.10) | 3.96× (3.91–4.00) | 3.12× (3.09–3.15) | 5.92× (5.85–5.99) | 2.46× (2.44–2.49) | 2.33× (2.31–2.35) | 1.00× (0.99–1.02) | 1.02× (0.98–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.60–0.61) | 2.93× (2.91–2.96) | 5.28× (5.22–5.34) | 3.15× (3.12–3.19) | 5.84× (5.77–5.91) | 2.45× (2.43–2.47) | 2.32× (2.30–2.34) | 1.00× (0.99–1.01) | 1.01× (0.98–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.60) | 3.23× (3.15–3.32) | 5.26× (5.13–5.39) | 3.13× (3.07–3.19) | 5.87× (5.74–6.00) | 2.44× (2.39–2.48) | 2.31× (2.26–2.35) | 0.99× (0.97–1.02) | 1.01× (0.97–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.95× (1.90–2.00) | 2.20× (2.14–2.26) | 2.69× (2.57–2.80) | 2.81× (2.74–2.88) | 3.37× (3.27–3.48) | 1.85× (1.80–1.90) | 1.58× (1.53–1.63) | 1.51× (1.47–1.54) | 2.09× (2.04–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.71× (1.67–1.75) | 1.98× (1.93–2.04) | 2.44× (2.38–2.50) | 2.42× (2.36–2.48) | 2.83× (2.76–2.91) | 1.50× (1.47–1.54) | 1.28× (1.25–1.32) | 1.22× (1.20–1.25) | 1.70× (1.66–1.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.59× (1.53–1.65) | 1.48× (1.44–1.53) | 1.78× (1.72–1.84) | 1.67× (1.61–1.72) | 1.79× (1.73–1.86) | 0.79× (0.76–0.81) | 0.67× (0.65–0.70) | 0.64× (0.62–0.66) | 0.89× (0.86–0.92) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.14× (1.11–1.17) | 2.28× (2.22–2.34) | 2.05× (1.99–2.10) | 2.11× (2.05–2.16) | 2.50× (2.44–2.56) | 2.59× (2.53–2.66) | 2.63× (2.56–2.70) | 1.03× (1.00–1.06) | 1.12× (1.09–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.10× (1.09–1.11) | 2.19× (2.18–2.21) | 1.94× (1.93–1.96) | 2.54× (2.52–2.57) | 2.33× (2.32–2.35) | 2.36× (2.33–2.39) | 2.39× (2.36–2.42) | 0.93× (0.92–0.95) | 1.02× (1.00–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.33× (1.30–1.36) | 2.02× (2.00–2.05) | 1.66× (1.65–1.68) | 2.26× (2.24–2.29) | 2.22× (2.19–2.25) | 1.76× (1.75–1.78) | 1.79× (1.77–1.80) | 0.70× (0.69–0.70) | 0.76× (0.75–0.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.05× (1.05–1.06) | 2.25× (2.23–2.26) | 1.84× (1.82–1.85) | 1.90× (1.88–1.92) | 2.30× (2.23–2.37) | 2.68× (2.65–2.70) | 2.76× (2.74–2.77) | 0.99× (0.98–1.00) | 1.13× (1.12–1.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (0.99–1.08) | 2.12× (2.08–2.16) | 1.72× (1.70–1.75) | 2.47× (2.43–2.51) | 2.09× (2.06–2.12) | 2.42× (2.39–2.46) | 2.49× (2.46–2.53) | 0.90× (0.88–0.91) | 1.02× (1.01–1.04) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.28× (1.26–1.29) | 2.04× (2.00–2.08) | 1.56× (1.52–1.60) | 2.27× (2.20–2.35) | 2.04× (2.01–2.07) | 1.82× (1.78–1.85) | 1.87× (1.84–1.90) | 0.67× (0.66–0.68) | 0.77× (0.75–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.65–0.66) | 2.51× (2.46–2.55) | 3.42× (3.36–3.47) | 2.61× (2.57–2.65) | 6.42× (6.33–6.51) | 2.07× (2.05–2.10) | 1.97× (1.95–2.00) | 0.97× (0.97–0.98) | 0.96× (0.90–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.59–0.62) | 2.37× (2.32–2.43) | 4.15× (4.05–4.24) | 2.60× (2.55–2.65) | 6.17× (6.05–6.30) | 1.99× (1.95–2.03) | 1.89× (1.85–1.93) | 0.93× (0.91–0.95) | 0.93× (0.87–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.64× (2.60–2.68) | 4.16× (4.08–4.24) | 2.67× (2.62–2.72) | 6.26× (6.13–6.39) | 2.03× (2.00–2.06) | 1.93× (1.90–1.96) | 0.95× (0.94–0.97) | 0.94× (0.89–1.01) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 1 | 1.31× (1.29–1.34) | 2.72× (2.67–2.77) | 3.05× (2.99–3.11) | 2.98× (2.94–3.03) | 3.34× (3.29–3.40) | 3.20× (3.15–3.26) | 3.06× (3.02–3.11) | 0.99× (0.98–1.01) | 1.20× (1.19–1.22) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 2 | 1.24× (1.23–1.25) | 2.48× (2.44–2.52) | 2.80× (2.76–2.83) | 2.89× (2.86–2.93) | 2.96× (2.93–3.00) | 2.80× (2.77–2.84) | 2.68× (2.65–2.71) | 0.87× (0.86–0.88) | 1.05× (1.04–1.07) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 4 | 1.37× (1.31–1.43) | 2.03× (1.99–2.07) | 2.30× (2.27–2.33) | 2.36× (2.32–2.39) | 2.36× (2.34–2.38) | 1.89× (1.86–1.91) | 1.80× (1.78–1.82) | 0.58× (0.58–0.59) | 0.71× (0.70–0.72) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.29–0.31) | 1.17× (1.17–1.17) | 0.84× (0.84–0.84) | 11.88× (11.87–11.90) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.37× (0.37–0.38) | 0.59× (0.53–0.67) | 0.86× (0.84–0.88) | 0.67× (0.65–0.68) | 6.30× (6.20–6.41) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.41–0.41) | 0.81× (0.71–0.92) | 0.79× (0.78–0.81) | 0.64× (0.63–0.65) | 4.29× (4.28–4.30) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.80× (0.80–0.80) | 4.29× (4.29–4.30) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.91× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.18–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.93–0.93) | 0.10× (0.10–0.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.48–0.49) | 0.43× (0.42–0.43) | 1.64× (1.64–1.64) | 0.63× (0.62–0.64) | 7.79× (7.79–7.80) | 0.62× (0.61–0.62) | 0.65× (0.64–0.65) | 0.71× (0.70–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.62× (0.61–0.63) | 0.67× (0.64–0.70) | 1.53× (1.51–1.55) | 0.86× (0.85–0.88) | 5.55× (5.46–5.63) | 0.44× (0.43–0.44) | 0.46× (0.45–0.47) | 0.50× (0.49–0.51) | 0.25× (0.24–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.59× (0.58–0.60) | 0.65× (0.62–0.69) | 1.12× (1.10–1.13) | 0.85× (0.84–0.86) | 3.22× (3.17–3.26) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) | 0.28× (0.28–0.28) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.38× (0.38–0.38) | 0.31× (0.31–0.31) | 1.15× (1.14–1.15) | 0.71× (0.70–0.71) | 5.07× (5.06–5.08) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.45) | 0.93× (0.93–0.94) | 0.73× (0.70–0.76) | 3.26× (3.23–3.29) | 0.20× (0.19–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.59–0.60) | 0.43× (0.42–0.43) | 0.81× (0.79–0.83) | 0.76× (0.74–0.78) | 2.37× (2.33–2.42) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.81× (0.80–0.81) | 5.11× (5.11–5.12) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.51) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.38× (0.35–0.41) | 0.88× (0.87–0.88) | 0.71× (0.68–0.73) | 3.01× (2.99–3.04) | 0.18× (0.18–0.18) | 0.19× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.55× (0.54–0.56) | 0.45× (0.43–0.48) | 0.74× (0.72–0.76) | 0.74× (0.71–0.77) | 2.15× (2.11–2.19) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.09× (1.09–1.09) | 0.76× (0.76–0.76) | 5.43× (5.43–5.44) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.40× (5.39–5.40) | 0.34× (0.34–0.34) | 0.36× (0.35–0.36) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.76–0.77) | 5.41× (5.40–5.42) | 0.34× (0.34–0.35) | 0.36× (0.35–0.36) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.46× (1.46–1.46) | 0.62× (0.62–0.62) | 6.60× (6.59–6.61) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 2 | 29 MB | 0.55× (0.55–0.56) | 0.50× (0.45–0.55) | 1.35× (1.31–1.39) | 0.82× (0.79–0.84) | 4.24× (4.20–4.29) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.41× (0.41–0.42) | 0.20× (0.20–0.20) |
| `text` (1.58 MB → 12.4 MB, 7.8×) | 4 | 48 MB | 0.56× (0.55–0.57) | 0.55× (0.52–0.58) | 1.06× (1.04–1.07) | 0.82× (0.79–0.85) | 2.68× (2.65–2.71) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 262 | 3.82 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 404 | 4.22 | 1.10× (1.08–1.12) | 77% | 1.02× (1.01–1.02) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 509 | 6.52 | 1.71× (1.67–1.76) | 48% | 1.23× (1.20–1.25) | 1.67× (1.65–1.69) |

**Best:** mibz2 at 4 threads (509 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 0.94, lbzip2 1t 0.10, lbzip2 2t 0.37, lbzip2 4t 0.59, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.37. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 5%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.16 | 0.02 | 0.08 | 0.412 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2625 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.99 | 10.6% |
| MTF + RLE2 | 1.81 | 19.3% |
| T-vector build | 1.84 | 19.5% |
| walk (the one each build uses) | 4.33 | 46.0% |
| of which: segment assembly | 0.15 | 1.6% |
| RLE1 | 0.22 | 2.4% |
| CRC | 0.21 | 2.2% |
| **total** | 9.40 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.697 | 1.304 | (in its total) | 1.87× (1.87–1.87) |  |
| walk | 1.615 | 2.660 | (in its total) | 1.65× (1.62–1.67) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.054 | 0.057 | (in its total) | 1.06× (1.03–1.09) |  |
| inverse BWT, total | 2.364 | 4.021 | 3.395 | 1.70× (1.68–1.72) | 1.44× (1.43–1.45) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h56m ± 14m15s |
| 4 threads | 1h00m ± 8m33s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m23s ± 6m01s |
| the scanner alone (its serial part) | 16m09s ± 35.4s |
| the workers alone, over 4 | 42m22s ± 6m01s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 5.56 ns a byte of output at 4 threads (CPU, summed over them), 3.30 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,582,006 | 12,389,993 | \`a116e21d349bc8d4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2624.6 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
