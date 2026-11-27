# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1199/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`b4005e8`](<https://github.com/lmmx/mibz2/commit/b4005e84d5aab60ce9c0720e82702d0dcf476230>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46037550432>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 546 | 6.8 | 3.15× (3.12–3.19) | 7.23× (7.15–7.32) | 7.52× (7.44–7.60) | 8.52× (8.42–8.62) | 7.47× (7.41–7.54) | 7.83× (7.76–7.89) | 7.53× (7.46–7.61) | 2.33× (2.30–2.35) | 2.62× (2.59–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.227 s | 924 | 7.3 | 3.22× (3.05–3.39) | 6.67× (6.37–6.99) | 7.53× (7.27–7.79) | 7.86× (7.54–8.19) | 6.87× (6.65–7.09) | 13.49× (13.05–13.95) | 12.99× (12.53–13.46) | 4.01× (3.87–4.15) | 4.52× (4.36–4.69) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,165 | 11.5 | 3.22× (3.12–3.32) | 4.77× (4.65–4.88) | 5.99× (5.80–6.19) | 5.86× (5.68–6.05) | 5.60× (5.48–5.73) | 16.59× (16.24–16.95) | 15.97× (15.56–16.39) | 4.93× (4.82–5.04) | 5.56× (5.43–5.69) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,105 | 3.7 | 0.57× (0.55–0.58) | 3.38× (3.31–3.44) | 4.51× (4.42–4.60) | 3.53× (3.47–3.60) | 6.05× (5.92–6.17) | 2.62× (2.57–2.68) | 2.62× (2.57–2.68) | 0.96× (0.93–0.99) | 1.90× (1.71–2.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,061 | 4.0 | 0.51× (0.50–0.52) | 2.72× (2.68–2.76) | 5.44× (5.35–5.53) | 3.41× (3.36–3.45) | 5.70× (5.62–5.79) | 2.52× (2.48–2.56) | 2.52× (2.48–2.56) | 0.92× (0.90–0.94) | 1.82× (1.64–2.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,044 | 3.9 | 0.46× (0.40–0.54) | 2.45× (2.12–2.84) | 4.90× (4.24–5.67) | 3.07× (2.66–3.55) | 5.21× (4.55–5.95) | 2.27× (1.96–2.62) | 2.27× (1.96–2.62) | 0.83× (0.72–0.96) | 1.64× (1.34–2.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 775 | 5.1 | 2.81× (2.74–2.87) | 3.18× (3.11–3.24) | 4.12× (3.86–4.40) | 4.25× (4.17–4.33) | 4.69× (4.61–4.77) | 2.87× (2.81–2.93) | 2.38× (2.30–2.47) | 2.17× (2.13–2.20) | 3.16× (3.10–3.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,163 | 6.2 | 2.29× (2.04–2.57) | 2.94× (2.63–3.29) | 4.42× (3.98–4.90) | 3.59× (3.25–3.96) | 5.15× (4.69–5.65) | 4.13× (3.76–4.54) | 3.44× (3.11–3.81) | 3.13× (2.83–3.45) | 4.56× (4.15–5.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,062 | 11.9 | 2.00× (1.91–2.09) | 2.10× (1.95–2.26) | 3.24× (3.02–3.47) | 2.42× (2.28–2.56) | 4.35× (4.17–4.54) | 3.92× (3.79–4.06) | 3.26× (3.11–3.43) | 2.97× (2.87–3.06) | 4.33× (4.18–4.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.102 s | 78 | 47.9 | 1.53× (1.49–1.57) | 2.87× (2.84–2.90) | 2.63× (2.56–2.69) | 2.71× (2.70–2.72) | 3.01× (3.00–3.02) | 3.23× (3.22–3.24) | 3.39× (3.37–3.41) | 1.36× (1.35–1.37) | 1.53× (1.50–1.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.063 s | 127 | 53.6 | 1.38× (1.31–1.46) | 2.70× (2.47–2.95) | 2.51× (2.38–2.64) | 3.31× (3.09–3.55) | 3.03× (2.83–3.24) | 5.09× (4.76–5.44) | 5.35× (5.00–5.73) | 2.14× (2.00–2.29) | 2.42× (2.26–2.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.051 s | 156 | 70.5 | 1.46× (1.40–1.53) | 2.36× (2.32–2.40) | 2.36× (2.28–2.44) | 2.92× (2.82–3.04) | 3.17× (3.09–3.26) | 6.39× (6.22–6.56) | 6.72× (6.56–6.89) | 2.69× (2.63–2.75) | 3.03× (2.93–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.111 s | 86 | 43.4 | 1.63× (1.62–1.64) | 3.26× (3.24–3.29) | 2.84× (2.81–2.88) | 2.98× (2.96–3.00) | 3.30× (3.28–3.32) | 3.97× (3.93–4.02) | 4.35× (4.31–4.39) | 1.65× (1.62–1.67) | 1.93× (1.91–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 144 | 49.3 | 1.51× (1.47–1.55) | 3.07× (2.96–3.19) | 2.80× (2.71–2.90) | 3.86× (3.78–3.95) | 3.39× (3.34–3.44) | 6.61× (6.45–6.78) | 7.24× (7.06–7.43) | 2.74× (2.65–2.84) | 3.21× (3.11–3.32) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.057 s | 170 | 67.5 | 1.51× (1.43–1.61) | 2.47× (2.25–2.70) | 2.44× (2.27–2.62) | 3.32× (3.09–3.56) | 3.07× (2.87–3.29) | 7.55× (7.01–8.12) | 8.27× (7.71–8.88) | 3.13× (2.93–3.34) | 3.67× (3.44–3.91) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.63) | 3.42× (3.35–3.49) | 4.30× (4.21–4.39) | 3.07× (3.00–3.14) | 6.63× (6.45–6.81) | 2.25× (2.20–2.30) | 2.31× (2.25–2.36) | 0.94× (0.92–0.97) | 1.72× (1.68–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 5.7 | 0.50× (0.49–0.51) | 2.40× (2.36–2.45) | 4.82× (4.73–4.92) | 2.88× (2.82–2.94) | 6.18× (6.04–6.32) | 2.10× (2.06–2.14) | 2.15× (2.10–2.20) | 0.88× (0.85–0.90) | 1.60× (1.57–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 766 | 5.6 | 0.49× (0.48–0.51) | 2.34× (2.27–2.41) | 4.73× (4.56–4.92) | 2.84× (2.74–2.95) | 6.03× (5.89–6.18) | 2.03× (1.98–2.10) | 2.08× (2.02–2.15) | 0.85× (0.82–0.88) | 1.55× (1.51–1.60) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 0.050 s | 284 | 13.5 | 2.02× (1.99–2.04) | 4.10× (4.03–4.17) | 4.49× (4.41–4.57) | 4.81× (4.73–4.88) | 5.01× (4.94–5.07) | 4.71× (4.67–4.76) | 4.84× (4.79–4.90) | 1.67× (1.59–1.74) | 1.90× (1.83–1.97) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 0.035 s | 402 | 17.7 | 1.56× (1.42–1.71) | 3.34× (2.99–3.73) | 3.84× (3.63–4.07) | 4.18× (3.82–4.57) | 4.35× (4.03–4.69) | 6.41× (5.92–6.93) | 6.58× (6.07–7.14) | 2.26× (2.05–2.49) | 2.58× (2.32–2.86) | yes |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 0.032 s | 443 | 27.5 | 1.58× (1.52–1.65) | 2.53× (2.38–2.68) | 3.30× (3.13–3.48) | 3.60× (3.44–3.76) | 4.09× (3.96–4.23) | 7.33× (7.09–7.57) | 7.53× (7.27–7.79) | 2.59× (2.43–2.76) | 2.95× (2.77–3.14) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.11–3.18) | 7.19× (7.11–7.28) | 7.48× (7.40–7.56) | 8.49× (8.39–8.59) | 7.44× (7.38–7.51) | 7.79× (7.73–7.86) | 7.50× (7.43–7.58) | 2.32× (2.30–2.34) | 2.60× (2.57–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (3.01–3.12) | 6.87× (6.82–6.92) | 7.14× (7.10–7.18) | 8.29× (8.24–8.33) | 6.95× (6.92–6.99) | 7.25× (7.22–7.29) | 6.98× (6.93–7.04) | 2.16× (2.14–2.17) | 2.42× (2.39–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (2.96–3.16) | 5.11× (4.96–5.25) | 5.53× (5.35–5.72) | 6.24× (6.04–6.45) | 5.30× (5.16–5.44) | 4.58× (4.47–4.70) | 4.41× (4.29–4.53) | 1.36× (1.33–1.40) | 1.53× (1.49–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.87× (2.83–2.92) | 3.87× (3.78–3.97) | 3.30× (3.25–3.36) | 5.59× (5.50–5.70) | 2.48× (2.43–2.52) | 2.48× (2.43–2.52) | 0.96× (0.93–0.99) | 1.62× (1.59–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.56) | 2.58× (2.54–2.61) | 4.65× (4.59–4.70) | 3.11× (3.07–3.15) | 5.12× (5.05–5.19) | 2.30× (2.27–2.34) | 2.30× (2.27–2.34) | 0.89× (0.88–0.91) | 1.51× (1.48–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.49× (0.42–0.57) | 2.61× (2.24–3.04) | 4.28× (3.64–5.02) | 2.85× (2.45–3.31) | 4.73× (4.11–5.45) | 2.11× (1.81–2.45) | 2.11× (1.81–2.45) | 0.82× (0.71–0.95) | 1.38× (1.18–1.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.64–2.75) | 2.93× (2.88–2.99) | 3.65× (3.39–3.93) | 4.05× (3.98–4.12) | 4.46× (4.39–4.53) | 2.75× (2.70–2.80) | 2.31× (2.21–2.40) | 2.09× (2.06–2.12) | 2.72× (2.68–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.32–2.57) | 2.68× (2.56–2.81) | 3.54× (3.46–3.63) | 3.51× (3.43–3.59) | 3.79× (3.70–3.89) | 2.26× (2.21–2.32) | 1.90× (1.81–2.00) | 1.73× (1.68–1.77) | 2.24× (2.18–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.20× (2.13–2.28) | 1.85× (1.76–1.94) | 2.45× (2.35–2.55) | 2.21× (2.10–2.32) | 2.27× (2.18–2.37) | 1.18× (1.13–1.23) | 0.99× (0.93–1.05) | 0.90× (0.86–0.93) | 1.17× (1.12–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.52× (1.48–1.56) | 2.79× (2.77–2.80) | 2.57× (2.52–2.62) | 2.68× (2.67–2.69) | 2.98× (2.97–2.99) | 3.19× (3.18–3.20) | 3.36× (3.33–3.38) | 1.35× (1.34–1.36) | 1.49× (1.47–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.33–1.44) | 2.61× (2.50–2.72) | 2.30× (2.22–2.39) | 3.14× (3.01–3.27) | 2.69× (2.58–2.81) | 2.81× (2.69–2.93) | 2.95× (2.82–3.09) | 1.19× (1.14–1.24) | 1.31× (1.25–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.54–1.72) | 2.52× (2.46–2.59) | 2.12× (2.08–2.17) | 2.94× (2.85–3.03) | 2.76× (2.70–2.81) | 2.17× (2.13–2.22) | 2.28× (2.23–2.34) | 0.92× (0.90–0.94) | 1.01× (0.99–1.03) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.62× (1.62–1.63) | 3.18× (3.17–3.20) | 2.77× (2.74–2.80) | 2.95× (2.93–2.97) | 3.27× (3.25–3.29) | 3.93× (3.88–3.97) | 4.30× (4.26–4.34) | 1.64× (1.61–1.66) | 1.87× (1.85–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.49× (1.47–1.51) | 2.97× (2.93–3.02) | 2.52× (2.48–2.56) | 3.58× (3.53–3.64) | 2.97× (2.94–3.00) | 3.45× (3.38–3.52) | 3.78× (3.72–3.85) | 1.44× (1.40–1.48) | 1.64× (1.61–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.68× (1.53–1.84) | 2.71× (2.51–2.92) | 2.13× (1.99–2.28) | 3.05× (2.85–3.26) | 2.76× (2.59–2.96) | 2.45× (2.28–2.63) | 2.68× (2.50–2.88) | 1.02× (0.96–1.09) | 1.17× (1.09–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.44× (2.39–2.49) | 3.37× (3.30–3.44) | 2.81× (2.76–2.86) | 5.92× (5.78–6.07) | 2.09× (2.06–2.13) | 2.14× (2.10–2.18) | 0.95× (0.93–0.97) | 1.36× (1.33–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.13× (2.09–2.17) | 3.81× (3.73–3.89) | 2.59× (2.54–2.64) | 5.31× (5.20–5.42) | 1.87× (1.83–1.90) | 1.91× (1.87–1.95) | 0.85× (0.83–0.86) | 1.21× (1.19–1.24) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.51–0.54) | 2.32× (2.25–2.39) | 3.78× (3.66–3.91) | 2.63× (2.53–2.74) | 5.34× (5.22–5.47) | 1.87× (1.82–1.92) | 1.91× (1.85–1.97) | 0.85× (0.82–0.87) | 1.21× (1.18–1.25) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 1.98× (1.96–2.00) | 3.90× (3.86–3.95) | 4.33× (4.29–4.39) | 4.68× (4.61–4.75) | 4.87× (4.81–4.93) | 4.59× (4.54–4.63) | 4.71× (4.66–4.76) | 1.64× (1.57–1.72) | 1.79× (1.77–1.81) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 1.63× (1.56–1.69) | 3.14× (3.08–3.21) | 3.52× (3.46–3.57) | 4.08× (4.03–4.13) | 3.80× (3.75–3.85) | 3.51× (3.46–3.56) | 3.61× (3.56–3.66) | 1.26× (1.20–1.32) | 1.37× (1.34–1.40) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 1.68× (1.63–1.74) | 2.64× (2.48–2.82) | 2.94× (2.83–3.05) | 3.22× (3.15–3.30) | 3.06× (2.99–3.14) | 2.26× (2.21–2.32) | 2.33× (2.26–2.39) | 0.81× (0.78–0.84) | 0.88× (0.86–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.99) | 0.79× (0.78–0.80) | 11.09× (11.05–11.14) | 0.27× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.35–0.37) | 0.52× (0.45–0.60) | 0.73× (0.70–0.76) | 0.63× (0.61–0.65) | 5.86× (5.66–6.07) | 0.14× (0.14–0.14) | 0.15× (0.14–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.36–0.38) | 0.62× (0.52–0.73) | 0.67× (0.65–0.68) | 0.58× (0.57–0.60) | 3.88× (3.79–3.98) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.53) | 0.44× (0.44–0.45) | 1.71× (1.69–1.72) | 0.66× (0.65–0.67) | 8.05× (7.99–8.10) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.70× (0.67–0.73) | 0.71× (0.63–0.81) | 1.66× (1.60–1.71) | 0.83× (0.80–0.86) | 6.02× (5.82–6.23) | 0.49× (0.47–0.51) | 0.51× (0.50–0.53) | 0.56× (0.54–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 37 MB | 0.67× (0.64–0.71) | 0.72× (0.65–0.80) | 1.21× (1.14–1.27) | 0.74× (0.70–0.78) | 3.57× (3.39–3.75) | 0.28× (0.27–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.51) | 0.40× (0.37–0.44) | 0.91× (0.88–0.93) | 0.74× (0.72–0.75) | 3.14× (3.06–3.22) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.64–0.66) | 0.46× (0.45–0.46) | 0.87× (0.86–0.88) | 0.82× (0.81–0.84) | 2.53× (2.49–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.19) | 0.83× (0.83–0.84) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.83× (0.81–0.86) | 0.71× (0.69–0.74) | 2.87× (2.77–2.97) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.49× (0.47–0.51) | 0.80× (0.78–0.81) | 0.82× (0.78–0.85) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.24–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.32–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.88) | 0.91× (0.90–0.91) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.91) | 6.19× (6.18–6.20) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.39× (0.38–0.39) | 1.40× (1.40–1.41) | 0.66× (0.66–0.66) | 6.41× (6.38–6.43) | 0.39× (0.39–0.39) | 0.43× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.30–0.31) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.37× (0.35–0.40) | 1.00× (0.97–1.03) | 0.67× (0.65–0.68) | 3.31× (3.24–3.39) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.1 MB → 14.2 MB, 6.7×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.46× (0.42–0.50) | 0.93× (0.90–0.96) | 0.76× (0.73–0.79) | 2.34× (2.28–2.40) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 546 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 924 | 1.99 | 1.07× (1.07–1.08) | 86% | 1.03× (1.01–1.04) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,165 | 3.12 | 1.70× (1.66–1.75) | 53% | 1.21× (1.19–1.22) | 1.65× (1.62–1.69) |

**Best:** mibz2 at 4 threads (1,165 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (362 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.34, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.49. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.01 | 0.07 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3695 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 9.9% |
| MTF + RLE2 | 1.58 | 18.9% |
| T-vector build | 1.49 | 17.9% |
| walk (the one each build uses) | 3.99 | 47.8% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.36 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.068 | (in its total) | 2.74× (2.72–2.75) |  |
| walk | 1.032 | 2.043 | (in its total) | 1.99× (1.97–2.01) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.040 | (in its total) | 1.07× (1.06–1.08) |  |
| inverse BWT, total | 1.461 | 3.151 | 1.872 | 2.17× (2.15–2.19) | 1.28× (1.28–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m15s ± 6m42s |
| 4 threads | 26m33s ± 3m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m42s ± 2m41s |
| the scanner alone (its serial part) | 8m35s ± 35.4s |
| the workers alone, over 4 | 18m42s ± 2m41s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 2.45 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,101,202 | 14,161,338 | \`702f0be27457325c…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
