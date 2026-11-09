# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-302/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`066deb2`](<https://github.com/lmmx/mibz2/commit/066deb2d69cbc87fbe08ba31c6fe0c68580cf81d>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42654086763>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.664 s | 316 | 10.3 | 2.37× (2.33–2.40) | 5.30× (5.23–5.38) | 5.50× (5.40–5.60) | 5.94× (5.85–6.03) | 5.32× (5.25–5.40) | 5.67× (5.59–5.76) | 5.30× (5.22–5.39) | 1.74× (1.71–1.76) | 1.98× (1.96–2.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.371 s | 565 | 11.3 | 2.35× (2.23–2.47) | 4.96× (4.79–5.14) | 5.58× (5.27–5.92) | 5.57× (5.30–5.85) | 4.98× (4.71–5.26) | 9.73× (9.18–10.31) | 9.10× (8.57–9.65) | 2.98× (2.82–3.15) | 3.40× (3.21–3.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.318 s | 659 | 18.5 | 2.39× (2.33–2.45) | 3.65× (3.56–3.73) | 4.58× (4.46–4.70) | 4.10× (4.03–4.17) | 4.16× (4.09–4.23) | 11.93× (11.70–12.15) | 11.15× (10.92–11.38) | 3.65× (3.57–3.73) | 4.17× (4.08–4.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 939 | 3.8 | 0.58× (0.57–0.60) | 3.79× (3.64–3.95) | 4.48× (4.31–4.65) | 3.45× (3.38–3.52) | 6.00× (5.88–6.11) | 2.77× (2.65–2.89) | 2.62× (2.47–2.78) | 1.01× (0.98–1.04) | 1.45× (1.42–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 946 | 3.8 | 0.56× (0.54–0.58) | 2.87× (2.78–2.96) | 6.03× (5.75–6.33) | 3.44× (3.33–3.55) | 5.88× (5.67–6.09) | 2.75× (2.60–2.90) | 2.59× (2.43–2.77) | 1.00× (0.97–1.03) | 1.43× (1.39–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 911 | 3.9 | 0.56× (0.54–0.57) | 2.82× (2.76–2.87) | 5.98× (5.87–6.09) | 3.34× (3.27–3.41) | 5.84× (5.72–5.96) | 2.68× (2.56–2.81) | 2.53× (2.43–2.65) | 0.98× (0.94–1.02) | 1.40× (1.38–1.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 530 | 6.5 | 2.60× (2.51–2.69) | 2.75× (2.73–2.78) | 3.56× (3.44–3.69) | 3.50× (3.47–3.53) | 3.80× (3.76–3.83) | 2.28× (2.25–2.30) | 1.93× (1.91–1.96) | 1.88× (1.85–1.90) | 2.74× (2.71–2.77) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 794 | 8.2 | 2.11× (2.04–2.17) | 2.71× (2.44–3.00) | 3.71× (3.53–3.89) | 3.11× (2.96–3.26) | 4.38× (4.22–4.54) | 3.38× (3.27–3.50) | 2.87× (2.77–2.98) | 2.79× (2.70–2.88) | 4.07× (3.93–4.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 729 | 14.9 | 1.83× (1.73–1.94) | 1.85× (1.74–1.96) | 2.96× (2.77–3.15) | 2.06× (1.94–2.20) | 3.65× (3.54–3.77) | 3.10× (2.99–3.20) | 2.63× (2.54–2.73) | 2.55× (2.46–2.65) | 3.72× (3.60–3.86) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 55.0 | 1.37× (1.34–1.41) | 2.68× (2.60–2.76) | 2.49× (2.42–2.56) | 2.44× (2.38–2.50) | 2.81× (2.74–2.88) | 3.05× (2.96–3.14) | 3.04× (2.97–3.11) | 1.26× (1.23–1.29) | 1.42× (1.38–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 62.5 | 1.30× (1.27–1.33) | 2.51× (2.44–2.57) | 2.42× (2.37–2.47) | 3.03× (2.97–3.08) | 2.92× (2.87–2.97) | 4.94× (4.84–5.05) | 4.93× (4.85–5.01) | 2.04× (2.00–2.09) | 2.31× (2.27–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 115 | 84.2 | 1.35× (1.32–1.38) | 2.16× (2.06–2.27) | 2.18× (2.08–2.28) | 2.60× (2.55–2.66) | 2.86× (2.77–2.95) | 5.86× (5.70–6.02) | 5.84× (5.70–5.99) | 2.42× (2.37–2.48) | 2.73× (2.66–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 50.8 | 1.50× (1.48–1.52) | 3.02× (2.96–3.08) | 2.53× (2.48–2.57) | 2.59× (2.55–2.64) | 3.02× (2.96–3.07) | 3.67× (3.61–3.72) | 3.82× (3.75–3.89) | 1.45× (1.43–1.48) | 1.72× (1.69–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 106 | 58.3 | 1.39× (1.32–1.47) | 2.70× (2.56–2.84) | 2.45× (2.36–2.54) | 3.32× (3.18–3.47) | 3.02× (2.90–3.15) | 5.95× (5.75–6.16) | 6.20× (5.97–6.44) | 2.36× (2.29–2.43) | 2.80× (2.70–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 80.7 | 1.45× (1.42–1.48) | 2.34× (2.16–2.54) | 2.27× (2.19–2.36) | 3.02× (2.93–3.11) | 2.92× (2.74–3.12) | 6.99× (6.88–7.10) | 7.29× (7.17–7.40) | 2.77× (2.72–2.83) | 3.29× (3.21–3.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 717 | 5.2 | 0.66× (0.65–0.67) | 3.15× (2.90–3.43) | 4.30× (4.23–4.38) | 3.07× (3.02–3.12) | 6.81× (6.54–7.10) | 2.41× (2.37–2.45) | 2.27× (2.24–2.31) | 0.97× (0.95–0.99) | 1.49× (1.46–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 678 | 5.6 | 0.56× (0.54–0.58) | 2.79× (2.74–2.84) | 5.03× (4.80–5.27) | 3.01× (2.88–3.14) | 6.37× (6.28–6.45) | 2.27× (2.24–2.30) | 2.14× (2.11–2.17) | 0.92× (0.90–0.94) | 1.40× (1.38–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 664 | 5.5 | 0.56× (0.55–0.57) | 2.77× (2.72–2.81) | 4.98× (4.79–5.18) | 2.88× (2.83–2.93) | 6.22× (6.11–6.33) | 2.24× (2.20–2.29) | 2.12× (2.08–2.15) | 0.90× (0.88–0.93) | 1.38× (1.36–1.41) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.095 s | 129 | 25.7 | 1.40× (1.39–1.41) | 2.88× (2.82–2.93) | 3.14× (3.10–3.19) | 3.10× (3.06–3.14) | 3.30× (3.23–3.37) | 3.51× (3.44–3.58) | 3.27× (3.19–3.34) | 1.10× (1.08–1.12) | 1.33× (1.31–1.36) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.062 s | 198 | 29.9 | 1.08× (0.95–1.23) | 2.34× (2.03–2.70) | 2.58× (2.25–2.96) | 2.96× (2.58–3.39) | 2.85× (2.49–3.27) | 4.90× (4.27–5.62) | 4.56× (3.96–5.24) | 1.53× (1.33–1.76) | 1.86× (1.63–2.13) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.049 s | 252 | 41.8 | 1.32× (1.28–1.36) | 2.22× (2.10–2.35) | 2.76× (2.66–2.88) | 3.06× (2.87–3.25) | 3.25× (3.13–3.38) | 6.86× (6.67–7.04) | 6.38× (6.19–6.57) | 2.14× (2.05–2.24) | 2.61× (2.48–2.74) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.36× (2.33–2.40) | 5.28× (5.21–5.36) | 5.48× (5.38–5.58) | 5.92× (5.84–6.01) | 5.31× (5.24–5.39) | 5.66× (5.57–5.74) | 5.29× (5.20–5.38) | 1.73× (1.71–1.76) | 1.97× (1.95–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.26× (2.23–2.30) | 5.05× (4.99–5.10) | 5.31× (5.22–5.40) | 5.77× (5.65–5.89) | 4.98× (4.90–5.06) | 5.16× (5.06–5.26) | 4.82× (4.71–4.93) | 1.58× (1.56–1.60) | 1.80× (1.77–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.26× (2.20–2.32) | 3.77× (3.68–3.86) | 4.09× (3.99–4.20) | 4.23× (4.14–4.32) | 3.82× (3.74–3.90) | 3.17× (3.10–3.24) | 2.96× (2.89–3.04) | 0.97× (0.95–0.99) | 1.10× (1.08–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.17× (3.11–3.24) | 3.97× (3.90–4.04) | 3.23× (3.18–3.29) | 5.56× (5.47–5.66) | 2.62× (2.51–2.74) | 2.48× (2.35–2.62) | 1.01× (0.98–1.04) | 1.04× (1.01–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.63) | 2.96× (2.88–3.05) | 5.25× (5.10–5.41) | 3.20× (3.12–3.29) | 5.38× (5.21–5.55) | 2.56× (2.42–2.70) | 2.42× (2.27–2.57) | 0.99× (0.96–1.02) | 1.02× (0.98–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.61) | 3.25× (3.18–3.33) | 5.26× (5.16–5.36) | 3.17× (3.11–3.24) | 5.42× (5.32–5.53) | 2.55× (2.42–2.68) | 2.41× (2.31–2.51) | 0.98× (0.95–1.02) | 1.01× (0.98–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.43–2.60) | 2.57× (2.54–2.59) | 3.31× (3.20–3.42) | 3.37× (3.34–3.39) | 3.65× (3.62–3.68) | 2.21× (2.18–2.23) | 1.88× (1.86–1.90) | 1.83× (1.81–1.85) | 2.49× (2.46–2.53) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.09× (2.06–2.13) | 2.29× (2.18–2.41) | 2.78× (2.74–2.82) | 2.81× (2.77–2.85) | 3.02× (2.98–3.07) | 1.73× (1.71–1.76) | 1.48× (1.46–1.50) | 1.44× (1.41–1.46) | 1.96× (1.92–1.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.84–2.05) | 1.68× (1.62–1.74) | 2.15× (2.07–2.23) | 1.97× (1.90–2.04) | 2.01× (1.96–2.07) | 0.96× (0.93–0.99) | 0.82× (0.79–0.85) | 0.80× (0.77–0.82) | 1.08× (1.05–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.34–1.40) | 2.61× (2.55–2.68) | 2.43× (2.37–2.49) | 2.42× (2.36–2.48) | 2.78× (2.71–2.85) | 3.02× (2.94–3.10) | 3.01× (2.95–3.08) | 1.26× (1.23–1.29) | 1.37× (1.33–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.44× (2.41–2.47) | 2.22× (2.19–2.25) | 2.86× (2.83–2.88) | 2.54× (2.51–2.57) | 2.67× (2.63–2.71) | 2.66× (2.63–2.69) | 1.11× (1.09–1.13) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.41–1.56) | 2.34× (2.23–2.45) | 1.94× (1.91–1.96) | 2.61× (2.57–2.64) | 2.51× (2.44–2.59) | 1.99× (1.97–2.01) | 1.98× (1.96–2.00) | 0.83× (0.82–0.84) | 0.90× (0.89–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.48–1.51) | 2.96× (2.91–3.02) | 2.47× (2.43–2.52) | 2.57× (2.53–2.62) | 2.99× (2.94–3.04) | 3.63× (3.58–3.68) | 3.78× (3.72–3.85) | 1.45× (1.42–1.48) | 1.67× (1.64–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.33–1.46) | 2.70× (2.60–2.80) | 2.25× (2.18–2.32) | 3.10× (3.00–3.20) | 2.70× (2.61–2.80) | 3.16× (3.07–3.26) | 3.30× (3.19–3.41) | 1.26× (1.23–1.30) | 1.46× (1.41–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.56–1.66) | 2.57× (2.44–2.71) | 2.01× (1.90–2.13) | 2.84× (2.73–2.96) | 2.69× (2.45–2.95) | 2.28× (2.24–2.33) | 2.38× (2.33–2.44) | 0.91× (0.89–0.93) | 1.05× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.72) | 2.71× (2.64–2.77) | 3.55× (3.51–3.59) | 2.80× (2.75–2.85) | 6.05× (5.81–6.30) | 2.23× (2.19–2.27) | 2.11× (2.08–2.15) | 0.97× (0.96–1.00) | 0.99× (0.95–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.65) | 2.46× (2.43–2.48) | 4.27× (4.20–4.34) | 2.78× (2.67–2.89) | 5.61× (5.56–5.66) | 2.07× (2.04–2.10) | 1.96× (1.93–1.99) | 0.90× (0.89–0.92) | 0.92× (0.88–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.71× (2.65–2.77) | 4.26× (4.19–4.32) | 2.74× (2.70–2.79) | 5.63× (5.53–5.73) | 2.10× (2.06–2.14) | 1.99× (1.95–2.02) | 0.92× (0.90–0.94) | 0.93× (0.90–0.96) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.39× (1.38–1.41) | 2.78× (2.74–2.83) | 3.06× (3.02–3.10) | 3.05× (3.02–3.09) | 3.25× (3.18–3.32) | 3.46× (3.39–3.52) | 3.22× (3.14–3.29) | 1.10× (1.07–1.12) | 1.27× (1.25–1.28) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.31× (1.29–1.33) | 2.54× (2.50–2.57) | 2.83× (2.76–2.89) | 3.05× (3.01–3.09) | 2.88× (2.85–2.91) | 2.98× (2.95–3.02) | 2.78× (2.74–2.82) | 0.95× (0.93–0.96) | 1.09× (1.08–1.11) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.56× (1.51–1.61) | 2.33× (2.24–2.43) | 2.68× (2.56–2.80) | 2.67× (2.58–2.77) | 2.71× (2.61–2.80) | 2.14× (2.09–2.18) | 1.99× (1.94–2.04) | 0.68× (0.65–0.70) | 0.78× (0.76–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.32–0.33) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.12× (12.10–12.14) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.64× (0.59–0.70) | 0.80× (0.80–0.81) | 0.67× (0.65–0.68) | 6.34× (6.30–6.38) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.79× (0.71–0.88) | 0.70× (0.69–0.71) | 0.61× (0.60–0.63) | 4.09× (4.06–4.12) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.45× (0.44–0.45) | 1.71× (1.71–1.72) | 0.67× (0.66–0.67) | 8.08× (8.05–8.10) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.67) | 0.71× (0.67–0.76) | 1.62× (1.61–1.64) | 0.80× (0.79–0.81) | 5.88× (5.82–5.93) | 0.47× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.54) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.62–0.65) | 0.74× (0.68–0.80) | 1.14× (1.09–1.18) | 0.69× (0.66–0.71) | 3.33× (3.22–3.45) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.23–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.51× (0.49–0.52) | 0.40× (0.36–0.44) | 0.91× (0.88–0.94) | 0.72× (0.68–0.75) | 3.14× (3.04–3.24) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.46× (0.44–0.48) | 0.86× (0.84–0.89) | 0.82× (0.80–0.84) | 2.53× (2.47–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.84× (0.83–0.84) | 5.24× (5.23–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.41) | 0.85× (0.81–0.89) | 0.69× (0.65–0.73) | 2.91× (2.79–3.04) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.27–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.47–0.52) | 0.80× (0.79–0.81) | 0.82× (0.78–0.85) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.21) | 1.09× (1.08–1.09) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.21) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.35–5.39) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.20–0.21) | 1.63× (1.63–1.64) | 0.78× (0.77–0.78) | 5.40× (5.39–5.42) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.41) | 1.48× (1.47–1.48) | 0.68× (0.67–0.68) | 6.64× (6.63–6.65) | 0.40× (0.40–0.41) | 0.44× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.32) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.43× (0.40–0.47) | 1.13× (1.09–1.17) | 0.70× (0.67–0.72) | 3.57× (3.47–3.67) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 53 MB | 0.53× (0.51–0.54) | 0.46× (0.44–0.49) | 0.98× (0.95–1.01) | 0.76× (0.71–0.82) | 2.48× (2.42–2.54) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 316 | 3.17 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 565 | 3.50 | 1.10× (1.08–1.12) | 86% | 1.05× (1.03–1.06) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 659 | 5.71 | 1.78× (1.75–1.82) | 53% | 1.27× (1.24–1.31) | 1.71× (1.68–1.74) |

**Best:** mibz2 at 4 threads (659 MB/s); lbzip2 at 4 threads (182 MB/s); pbz2 at 4 threads (277 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.52, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.99 | 0.02 | 0.08 | 0.318 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3242 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 11.5% |
| MTF + RLE2 | 1.52 | 15.4% |
| T-vector build | 2.16 | 21.9% |
| walk (the one each build uses) | 4.49 | 45.4% |
| of which: segment assembly | 0.19 | 1.9% |
| RLE1 | 0.30 | 3.0% |
| CRC | 0.26 | 2.6% |
| **total** | 9.90 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.674 | 1.298 | (in its total) | 1.93× (1.92–1.94) |  |
| walk | 1.262 | 2.669 | (in its total) | 2.12× (2.11–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.055 | 0.061 | (in its total) | 1.08× (1.04–1.12) |  |
| inverse BWT, total | 1.992 | 4.028 | 3.414 | 2.03× (2.02–2.04) | 1.72× (1.70–1.74) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h36m ± 11m31s |
| 4 threads | 46m09s ± 6m32s |
| 4 threads, the sample's overlap gap as a fixed cost | 35m43s ± 5m07s |
| the scanner alone (its serial part) | 15m25s ± 32.2s |
| the workers alone, over 4 | 35m43s ± 5m07s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.68 ns a byte of output at 4 threads (CPU, summed over them), 2.67 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3242.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
