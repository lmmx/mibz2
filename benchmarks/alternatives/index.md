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

The latest record: [`8be686b`](<https://github.com/lmmx/mibz2/commit/8be686b5d6282f4cecb08b561077849ab8700dc8>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42553207242>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.529 s | 396 | 9.4 | 2.29× (2.28–2.30) | 5.26× (5.24–5.28) | 5.45× (5.43–5.47) | 6.19× (6.17–6.21) | 5.42× (5.39–5.44) | 6.13× (6.11–6.15) | 5.38× (5.36–5.41) | 1.70× (1.69–1.70) | 1.89× (1.88–1.91) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.290 s | 724 | 10.1 | 2.37× (2.30–2.46) | 4.86× (4.70–5.02) | 5.65× (5.39–5.91) | 5.92× (5.71–6.13) | 5.15× (4.96–5.35) | 10.93× (10.53–11.35) | 9.61× (9.26–9.96) | 3.03× (2.91–3.15) | 3.38× (3.25–3.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.246 s | 852 | 16.4 | 2.33× (2.28–2.38) | 3.47× (3.38–3.56) | 4.29× (4.18–4.39) | 4.18× (4.10–4.27) | 4.11× (4.05–4.17) | 13.10× (12.90–13.30) | 11.51× (11.32–11.70) | 3.63× (3.57–3.68) | 4.05× (3.99–4.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,120 | 3.6 | 0.57× (0.57–0.58) | 3.42× (3.38–3.47) | 4.57× (4.51–4.63) | 3.56× (3.52–3.61) | 6.10× (6.03–6.16) | 2.68× (2.64–2.73) | 2.65× (2.62–2.69) | 0.97× (0.95–1.00) | 1.87× (1.68–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,087 | 3.8 | 0.52× (0.51–0.53) | 2.78× (2.73–2.82) | 5.57× (5.48–5.66) | 3.48× (3.42–3.53) | 5.90× (5.81–5.99) | 2.61× (2.55–2.66) | 2.58× (2.53–2.62) | 0.95× (0.92–0.97) | 1.81× (1.66–1.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,078 | 3.7 | 0.52× (0.51–0.53) | 2.77× (2.70–2.83) | 5.53× (5.40–5.66) | 3.45× (3.39–3.52) | 5.97× (5.80–6.15) | 2.59× (2.51–2.66) | 2.56× (2.50–2.62) | 0.94× (0.92–0.96) | 1.80× (1.65–1.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 685 | 5.8 | 2.50× (2.49–2.52) | 2.85× (2.84–2.85) | 3.63× (3.49–3.78) | 3.80× (3.77–3.84) | 4.22× (4.14–4.30) | 3.19× (3.17–3.21) | 2.15× (2.04–2.26) | 1.92× (1.92–1.93) | 2.84× (2.83–2.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,080 | 6.8 | 2.10× (1.89–2.33) | 2.63× (2.38–2.91) | 3.92× (3.63–4.23) | 3.37× (3.10–3.67) | 4.68× (4.21–5.19) | 4.68× (4.22–5.19) | 3.15× (2.81–3.54) | 2.83× (2.55–3.13) | 4.17× (3.76–4.61) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 922 | 13.8 | 1.79× (1.74–1.84) | 1.84× (1.68–2.01) | 2.85× (2.67–3.04) | 2.15× (2.05–2.25) | 3.89× (3.80–3.99) | 4.32× (4.23–4.41) | 2.91× (2.76–3.06) | 2.61× (2.57–2.65) | 3.84× (3.79–3.90) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.5 | 1.38× (1.38–1.38) | 2.63× (2.60–2.66) | 2.41× (2.39–2.42) | 2.53× (2.51–2.54) | 2.80× (2.78–2.83) | 3.17× (3.15–3.19) | 3.08× (3.06–3.10) | 1.25× (1.24–1.26) | 1.39× (1.37–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 121 | 57.7 | 1.29× (1.26–1.33) | 2.60× (2.48–2.72) | 2.42× (2.35–2.50) | 3.22× (3.13–3.32) | 3.00× (2.91–3.08) | 5.21× (5.09–5.34) | 5.06× (4.94–5.18) | 2.06× (2.01–2.11) | 2.29× (2.22–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 76.3 | 1.35× (1.30–1.39) | 2.20× (2.08–2.32) | 2.10× (2.06–2.14) | 2.71× (2.60–2.83) | 2.94× (2.88–2.99) | 6.33× (6.21–6.46) | 6.15× (6.02–6.27) | 2.50× (2.45–2.55) | 2.78× (2.73–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 46.1 | 1.52× (1.52–1.53) | 3.05× (3.02–3.08) | 2.70× (2.66–2.73) | 2.81× (2.80–2.82) | 3.12× (3.11–3.14) | 4.02× (3.99–4.06) | 4.02× (3.98–4.06) | 1.51× (1.50–1.52) | 1.81× (1.80–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 s | 135 | 52.7 | 1.39× (1.35–1.43) | 2.84× (2.76–2.92) | 2.59× (2.56–2.62) | 3.62× (3.54–3.70) | 3.14× (3.09–3.19) | 6.61× (6.49–6.74) | 6.61× (6.54–6.68) | 2.48× (2.44–2.53) | 2.97× (2.92–3.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 74.6 | 1.43× (1.41–1.46) | 2.25× (2.19–2.32) | 2.26× (2.21–2.31) | 3.05× (3.00–3.10) | 2.90× (2.85–2.95) | 7.63× (7.55–7.72) | 7.63× (7.50–7.75) | 2.87× (2.83–2.90) | 3.43× (3.38–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.63) | 3.45× (3.40–3.50) | 4.42× (4.23–4.63) | 3.08× (3.04–3.13) | 6.65× (6.54–6.77) | 2.35× (2.31–2.39) | 2.30× (2.27–2.34) | 0.96× (0.94–0.98) | 1.73× (1.71–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 806 | 5.3 | 0.52× (0.52–0.53) | 2.50× (2.47–2.53) | 5.03× (4.97–5.09) | 3.03× (2.95–3.11) | 6.37× (6.29–6.45) | 2.26× (2.23–2.29) | 2.21× (2.19–2.24) | 0.92× (0.91–0.93) | 1.67× (1.65–1.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.52× (0.51–0.53) | 2.48× (2.43–2.53) | 4.97× (4.89–5.06) | 2.98× (2.92–3.04) | 6.37× (6.23–6.51) | 2.23× (2.19–2.28) | 2.19× (2.15–2.23) | 0.91× (0.89–0.94) | 1.65× (1.62–1.68) | yes |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 1 | 0.072 s | 176 | 21.6 | 1.36× (1.36–1.37) | 2.96× (2.95–2.98) | 3.24× (3.22–3.26) | 3.47× (3.42–3.52) | 3.61× (3.57–3.66) | 3.78× (3.76–3.80) | 3.47× (3.45–3.50) | 1.12× (1.11–1.12) | 1.27× (1.26–1.27) | yes |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 2 | 0.045 s | 282 | 25.1 | 1.15× (1.02–1.30) | 2.44× (2.16–2.76) | 2.92× (2.60–3.27) | 3.40× (3.05–3.80) | 3.42× (3.00–3.88) | 5.57× (4.98–6.23) | 5.12× (4.57–5.73) | 1.65× (1.47–1.85) | 1.87× (1.67–2.09) | yes |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 4 | 0.037 s | 344 | 36.0 | 1.29× (1.25–1.33) | 2.09× (1.98–2.22) | 2.78× (2.72–2.85) | 3.07× (3.02–3.12) | 3.49× (3.44–3.54) | 7.37× (7.25–7.48) | 6.77× (6.62–6.92) | 2.18× (2.15–2.21) | 2.47× (2.44–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.28× (2.27–2.29) | 5.24× (5.22–5.26) | 5.43× (5.41–5.45) | 6.18× (6.16–6.20) | 5.40× (5.38–5.42) | 6.11× (6.09–6.13) | 5.37× (5.35–5.39) | 1.69× (1.69–1.70) | 1.88× (1.87–1.89) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.20× (2.18–2.22) | 4.98× (4.95–5.02) | 5.21× (5.17–5.25) | 6.05× (6.02–6.08) | 5.07× (5.05–5.08) | 5.71× (5.69–5.73) | 5.01× (4.99–5.04) | 1.58× (1.58–1.59) | 1.76× (1.75–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.16× (2.12–2.20) | 3.57× (3.51–3.62) | 3.84× (3.76–3.93) | 4.31× (4.24–4.39) | 3.73× (3.67–3.79) | 3.48× (3.42–3.54) | 3.05× (3.00–3.11) | 0.96× (0.95–0.98) | 1.07× (1.05–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.92× (2.89–2.95) | 3.91× (3.85–3.97) | 3.34× (3.30–3.38) | 5.65× (5.59–5.71) | 2.54× (2.50–2.58) | 2.51× (2.48–2.54) | 0.98× (0.96–1.00) | 1.64× (1.62–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.69× (2.65–2.73) | 4.84× (4.76–4.92) | 3.24× (3.19–3.28) | 5.40× (5.33–5.48) | 2.43× (2.38–2.48) | 2.40× (2.37–2.44) | 0.94× (0.92–0.95) | 1.57× (1.56–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.96× (2.90–3.02) | 4.88× (4.79–4.98) | 3.29× (3.23–3.35) | 5.58× (5.42–5.74) | 2.47× (2.40–2.54) | 2.44× (2.39–2.50) | 0.95× (0.93–0.97) | 1.60× (1.57–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.42× (2.41–2.43) | 2.63× (2.63–2.64) | 3.23× (3.05–3.41) | 3.64× (3.61–3.68) | 4.04× (3.96–4.12) | 3.06× (3.04–3.08) | 2.09× (1.98–2.20) | 1.87× (1.87–1.88) | 2.46× (2.45–2.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.19× (2.16–2.23) | 2.38× (2.32–2.45) | 3.20× (3.15–3.25) | 3.21× (3.16–3.26) | 3.48× (3.41–3.54) | 2.56× (2.53–2.60) | 1.75× (1.65–1.85) | 1.57× (1.54–1.59) | 2.06× (2.02–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.89× (1.81–1.98) | 1.60× (1.54–1.65) | 2.11× (2.06–2.17) | 1.98× (1.93–2.03) | 2.01× (1.95–2.07) | 1.28× (1.25–1.31) | 0.87× (0.83–0.91) | 0.78× (0.76–0.80) | 1.03× (1.00–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.57× (2.56–2.58) | 2.36× (2.34–2.37) | 2.50× (2.49–2.52) | 2.78× (2.75–2.80) | 3.14× (3.12–3.16) | 3.05× (3.03–3.07) | 1.25× (1.24–1.26) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.45× (2.44–2.47) | 2.20× (2.14–2.26) | 2.97× (2.95–2.99) | 2.60× (2.52–2.68) | 2.81× (2.80–2.83) | 2.73× (2.72–2.75) | 1.12× (1.11–1.13) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.44–1.50) | 2.28× (2.22–2.35) | 1.94× (1.91–1.97) | 2.61× (2.56–2.67) | 2.49× (2.45–2.54) | 2.12× (2.08–2.15) | 2.05× (2.02–2.09) | 0.84× (0.83–0.86) | 0.92× (0.91–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.51–1.52) | 3.00× (2.99–3.01) | 2.63× (2.59–2.67) | 2.79× (2.78–2.79) | 3.09× (3.08–3.11) | 3.98× (3.94–4.02) | 3.98× (3.94–4.02) | 1.50× (1.50–1.51) | 1.75× (1.73–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.39) | 2.77× (2.74–2.79) | 2.36× (2.35–2.38) | 3.32× (3.29–3.34) | 2.77× (2.75–2.79) | 3.48× (3.44–3.52) | 3.47× (3.44–3.51) | 1.31× (1.30–1.33) | 1.52× (1.52–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.57–1.63) | 2.46× (2.39–2.53) | 1.97× (1.94–2.00) | 2.81× (2.76–2.86) | 2.56× (2.54–2.59) | 2.46× (2.42–2.49) | 2.45× (2.41–2.50) | 0.93× (0.92–0.94) | 1.08× (1.06–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.44× (2.41–2.47) | 3.45× (3.31–3.60) | 2.83× (2.79–2.86) | 5.96× (5.86–6.05) | 2.18× (2.15–2.21) | 2.14× (2.12–2.17) | 0.97× (0.95–0.99) | 1.37× (1.34–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.26× (2.24–2.29) | 4.04× (3.99–4.10) | 2.79× (2.72–2.85) | 5.62× (5.55–5.70) | 2.06× (2.03–2.09) | 2.02× (2.00–2.04) | 0.91× (0.90–0.92) | 1.29× (1.27–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.58) | 2.51× (2.47–2.55) | 4.03× (3.95–4.10) | 2.83× (2.77–2.88) | 5.80× (5.68–5.92) | 2.10× (2.07–2.13) | 2.06× (2.03–2.10) | 0.93× (0.91–0.95) | 1.31× (1.29–1.34) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 1 | 1.35× (1.35–1.36) | 2.85× (2.83–2.87) | 3.16× (3.14–3.18) | 3.41× (3.36–3.46) | 3.55× (3.51–3.59) | 3.71× (3.69–3.73) | 3.41× (3.39–3.43) | 1.11× (1.11–1.12) | 1.21× (1.20–1.22) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 2 | 1.23× (1.22–1.24) | 2.59× (2.57–2.61) | 2.86× (2.84–2.89) | 3.37× (3.27–3.46) | 3.18× (3.04–3.32) | 3.20× (3.18–3.23) | 2.95× (2.92–2.97) | 0.96× (0.95–0.97) | 1.04× (1.04–1.05) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 4 | 1.43× (1.39–1.46) | 2.18× (2.16–2.21) | 2.46× (2.42–2.50) | 2.76× (2.72–2.80) | 2.66× (2.63–2.69) | 2.23× (2.21–2.26) | 2.05× (2.01–2.09) | 0.67× (0.66–0.68) | 0.73× (0.72–0.74) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.37× (0.35–0.38) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.18× (12.17–12.19) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.54× (0.47–0.62) | 0.79× (0.78–0.81) | 0.66× (0.65–0.67) | 6.34× (6.24–6.45) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.59× (0.48–0.73) | 0.70× (0.70–0.71) | 0.62× (0.61–0.62) | 4.10× (4.08–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.44× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.74) | 0.66× (0.66–0.67) | 8.14× (8.12–8.16) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.70) | 0.63× (0.57–0.69) | 1.64× (1.60–1.68) | 0.81× (0.79–0.83) | 5.93× (5.78–6.09) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.62–0.67) | 0.70× (0.63–0.77) | 1.17× (1.13–1.21) | 0.76× (0.72–0.79) | 3.42× (3.31–3.53) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.25× (5.25–5.26) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.52) | 0.42× (0.39–0.45) | 0.91× (0.88–0.95) | 0.71× (0.68–0.75) | 3.16× (3.04–3.29) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.90) | 0.82× (0.80–0.85) | 2.57× (2.53–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.45× (0.45–0.46) | 0.37× (0.34–0.39) | 0.83× (0.81–0.84) | 0.70× (0.67–0.73) | 2.84× (2.79–2.89) | 0.17× (0.17–0.17) | 0.18× (0.18–0.19) | 0.28× (0.27–0.28) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.46× (0.43–0.49) | 0.80× (0.79–0.81) | 0.78× (0.73–0.82) | 2.30× (2.27–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.23× (0.22–0.25) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.45× (5.44–5.46) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.77) | 5.40× (5.38–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.42× (5.40–5.43) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.40–0.41) | 1.50× (1.49–1.50) | 0.67× (0.67–0.67) | 6.75× (6.74–6.76) | 0.40× (0.40–0.40) | 0.42× (0.42–0.43) | 0.66× (0.66–0.66) | 0.33× (0.33–0.33) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 2 | 34 MB | 0.47× (0.45–0.49) | 0.46× (0.43–0.49) | 1.14× (1.08–1.19) | 0.71× (0.67–0.76) | 3.60× (3.45–3.74) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.35× (0.34–0.37) | 0.18× (0.17–0.18) |
| `text` (2.14 MB → 12.6 MB, 5.9×) | 4 | 51 MB | 0.53× (0.52–0.55) | 0.47× (0.45–0.49) | 1.01× (0.99–1.03) | 0.80× (0.76–0.83) | 2.55× (2.50–2.61) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 396 | 2.53 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 724 | 2.71 | 1.07× (1.06–1.08) | 89% | 1.02× (1.01–1.02) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 852 | 4.43 | 1.76× (1.72–1.79) | 53% | 1.20× (1.18–1.21) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (852 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (366 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.08, lbzip2 1t 0.06, lbzip2 2t 0.24, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.75 | 0.01 | 0.09 | 0.246 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3705 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.3% |
| MTF + RLE2 | 1.58 | 17.6% |
| T-vector build | 1.77 | 19.6% |
| walk (the one each build uses) | 4.08 | 45.3% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.8% |
| **total** | 9.00 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.453 | 1.057 | (in its total) | 2.33× (2.32–2.34) |  |
| walk | 1.047 | 2.041 | (in its total) | 1.95× (1.94–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.041 | 0.040 | (in its total) | 0.98× (0.95–1.01) |  |
| inverse BWT, total | 1.542 | 3.138 | 1.887 | 2.03× (2.02–2.04) | 1.22× (1.22–1.23) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m09s |
| 4 threads | 36m04s ± 5m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m28s ± 3m54s |
| the scanner alone (its serial part) | 11m47s ± 19.7s |
| the workers alone, over 4 | 27m28s ± 3m54s |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 3.60 ns a byte of output at 4 threads (CPU, summed over them), 2.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,141,897 | 12,643,470 | \`de764ea51208f145…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3705.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
