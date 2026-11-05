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

The latest record: [`eb5363c`](<https://github.com/lmmx/mibz2/commit/eb5363c24032ba3b59eca87a21a3527f8ed19cdf>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/41979594724>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.690 s | 304 | 10.7 | 2.19× (2.18–2.21) | 4.97× (4.93–5.01) | 5.09× (5.05–5.13) | 5.62× (5.57–5.67) | 5.00× (4.96–5.05) | 5.37× (5.33–5.41) | 5.12× (5.08–5.16) | 1.61× (1.60–1.62) | 1.86× (1.85–1.88) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.373 s | 562 | 11.4 | 2.28× (2.21–2.36) | 4.75× (4.63–4.87) | 5.46× (5.29–5.63) | 5.50× (5.33–5.67) | 4.88× (4.77–5.00) | 9.77× (9.52–10.03) | 9.32× (9.09–9.55) | 2.93× (2.85–3.00) | 3.39× (3.30–3.49) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.299 s | 700 | 17.4 | 2.33× (2.30–2.35) | 3.56× (3.49–3.62) | 4.44× (4.30–4.58) | 4.09× (3.92–4.26) | 4.12× (3.92–4.33) | 12.34× (12.22–12.46) | 11.76× (11.65–11.88) | 3.70× (3.66–3.74) | 4.28× (4.23–4.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.6 | 0.60× (0.60–0.60) | 3.65× (3.47–3.83) | 4.50× (4.46–4.55) | 3.59× (3.56–3.63) | 6.26× (6.19–6.34) | 2.87× (2.84–2.90) | 2.69× (2.66–2.72) | 1.01× (1.00–1.03) | 1.53× (1.51–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 964 | 3.7 | 0.56× (0.55–0.57) | 2.97× (2.93–3.02) | 5.92× (5.79–6.06) | 3.53× (3.48–3.57) | 6.10× (5.99–6.21) | 2.79× (2.75–2.83) | 2.62× (2.58–2.65) | 0.99× (0.97–1.00) | 1.49× (1.47–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 964 | 3.7 | 0.56× (0.55–0.57) | 2.97× (2.92–3.02) | 5.94× (5.82–6.06) | 3.51× (3.45–3.57) | 6.13× (6.03–6.23) | 2.78× (2.73–2.82) | 2.60× (2.56–2.64) | 0.98× (0.96–1.00) | 1.48× (1.46–1.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 537 | 6.3 | 2.41× (2.40–2.43) | 2.79× (2.77–2.81) | 3.53× (3.40–3.65) | 3.60× (3.58–3.63) | 3.82× (3.80–3.84) | 2.30× (2.29–2.32) | 1.93× (1.92–1.94) | 1.84× (1.83–1.85) | 2.78× (2.76–2.80) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 834 | 7.7 | 2.23× (2.04–2.44) | 2.62× (2.56–2.68) | 3.70× (3.52–3.89) | 3.28× (3.21–3.36) | 4.55× (4.45–4.66) | 3.57× (3.49–3.65) | 2.99× (2.93–3.06) | 2.85× (2.80–2.91) | 4.31× (4.22–4.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 757 | 15.1 | 1.83× (1.77–1.89) | 2.05× (2.01–2.10) | 2.93× (2.83–3.02) | 2.29× (2.07–2.54) | 3.90× (3.80–3.99) | 3.30× (3.23–3.37) | 2.76× (2.71–2.82) | 2.64× (2.58–2.69) | 3.98× (3.90–4.07) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.150 s | 53 | 61.8 | 1.17× (1.17–1.18) | 2.33× (2.31–2.36) | 2.09× (2.09–2.09) | 2.15× (2.15–2.16) | 2.47× (2.46–2.48) | 2.69× (2.68–2.69) | 2.83× (2.83–2.84) | 1.07× (1.07–1.07) | 1.22× (1.21–1.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.092 s | 87 | 68.5 | 1.09× (1.04–1.15) | 2.25× (2.17–2.34) | 2.03× (1.92–2.15) | 2.66× (2.52–2.80) | 2.50× (2.38–2.63) | 4.29× (4.08–4.51) | 4.52× (4.30–4.75) | 1.71× (1.63–1.80) | 1.95× (1.85–2.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 111 | 88.7 | 1.22× (1.21–1.24) | 2.01× (1.98–2.05) | 1.92× (1.87–1.98) | 2.44× (2.36–2.52) | 2.67× (2.63–2.71) | 5.60× (5.53–5.68) | 5.90× (5.84–5.97) | 2.23× (2.21–2.26) | 2.55× (2.51–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.198 s | 48 | 67.7 | 1.05× (1.02–1.08) | 2.19× (2.13–2.25) | 1.83× (1.77–1.88) | 1.92× (1.86–1.98) | 2.20× (2.14–2.26) | 2.70× (2.63–2.78) | 2.89× (2.81–2.98) | 1.02× (0.99–1.04) | 1.22× (1.18–1.25) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.114 s | 84 | 74.0 | 1.03× (1.02–1.05) | 2.15× (2.08–2.23) | 1.88× (1.87–1.90) | 2.72× (2.66–2.79) | 2.38× (2.37–2.40) | 4.78× (4.74–4.82) | 5.11× (5.07–5.15) | 1.80× (1.79–1.81) | 2.15× (2.13–2.16) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.091 s | 105 | 97.0 | 1.17× (1.14–1.19) | 1.91× (1.85–1.97) | 1.82× (1.80–1.85) | 2.49× (2.43–2.55) | 2.36× (2.33–2.40) | 5.98× (5.89–6.07) | 6.40× (6.30–6.49) | 2.25× (2.22–2.29) | 2.69× (2.65–2.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 733 | 5.0 | 0.66× (0.65–0.66) | 3.02× (2.99–3.04) | 4.43× (4.39–4.47) | 3.08× (3.05–3.11) | 6.79× (6.73–6.86) | 2.44× (2.42–2.47) | 2.32× (2.29–2.34) | 0.97× (0.96–0.98) | 1.52× (1.51–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 685 | 5.5 | 0.54× (0.54–0.55) | 2.83× (2.79–2.86) | 4.87× (4.81–4.93) | 2.92× (2.88–2.96) | 6.39× (6.31–6.47) | 2.29× (2.26–2.32) | 2.17× (2.14–2.19) | 0.91× (0.89–0.93) | 1.42× (1.40–1.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 5.3 | 0.56× (0.56–0.57) | 2.86× (2.83–2.88) | 4.91× (4.87–4.95) | 2.95× (2.92–2.99) | 6.44× (6.39–6.50) | 2.31× (2.29–2.33) | 2.19× (2.16–2.21) | 0.91× (0.90–0.93) | 1.43× (1.42–1.45) | yes |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 1 | 0.080 s | 152 | 21.9 | 1.53× (1.52–1.54) | 3.08× (3.04–3.12) | 3.42× (3.38–3.45) | 3.33× (3.31–3.35) | 3.58× (3.56–3.60) | 3.63× (3.61–3.65) | 3.65× (3.62–3.68) | 1.16× (1.15–1.16) | 1.39× (1.39–1.40) | yes |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 2 | 0.050 s | 244 | 25.4 | 1.31× (1.27–1.35) | 2.87× (2.50–3.28) | 3.20× (3.11–3.29) | 3.65× (3.32–4.01) | 3.54× (3.45–3.63) | 5.77× (5.62–5.92) | 5.80× (5.67–5.93) | 1.84× (1.79–1.89) | 2.21× (2.16–2.27) | yes |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 4 | 0.040 s | 301 | 36.0 | 1.38× (1.35–1.42) | 2.38× (2.26–2.52) | 2.80× (2.73–2.87) | 3.25× (3.09–3.41) | 3.43× (3.39–3.46) | 7.21× (7.14–7.28) | 7.25× (7.15–7.34) | 2.30× (2.27–2.32) | 2.77× (2.74–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.19× (2.18–2.21) | 4.95× (4.92–4.99) | 5.07× (5.03–5.11) | 5.61× (5.56–5.65) | 4.99× (4.95–5.04) | 5.36× (5.31–5.40) | 5.11× (5.06–5.15) | 1.61× (1.59–1.62) | 1.85× (1.84–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.14× (2.13–2.14) | 4.78× (4.76–4.81) | 5.01× (4.98–5.03) | 5.57× (5.55–5.59) | 4.75× (4.73–4.76) | 5.05× (5.02–5.07) | 4.81× (4.79–4.83) | 1.51× (1.51–1.52) | 1.75× (1.74–1.75) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.20× (2.17–2.23) | 3.71× (3.67–3.75) | 3.97× (3.92–4.01) | 4.21× (4.16–4.26) | 3.73× (3.68–3.78) | 3.31× (3.27–3.35) | 3.16× (3.12–3.19) | 0.99× (0.98–1.00) | 1.14× (1.13–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.33× (3.27–3.39) | 4.13× (4.08–4.18) | 3.40× (3.35–3.44) | 5.85× (5.77–5.94) | 2.73× (2.69–2.76) | 2.56× (2.52–2.59) | 1.02× (1.00–1.03) | 1.11× (1.06–1.17) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.05× (3.01–3.10) | 5.36× (5.28–5.43) | 3.28× (3.25–3.32) | 5.58× (5.49–5.67) | 2.60× (2.56–2.63) | 2.44× (2.40–2.47) | 0.97× (0.95–0.98) | 1.06× (1.02–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.41× (3.33–3.48) | 5.42× (5.34–5.51) | 3.33× (3.28–3.38) | 5.68× (5.59–5.78) | 2.63× (2.58–2.67) | 2.46× (2.42–2.50) | 0.98× (0.96–1.00) | 1.07× (1.03–1.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.35× (2.33–2.36) | 2.58× (2.56–2.60) | 3.26× (3.15–3.38) | 3.48× (3.46–3.51) | 3.69× (3.66–3.71) | 2.24× (2.23–2.26) | 1.89× (1.88–1.90) | 1.80× (1.79–1.81) | 2.51× (2.49–2.53) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (2.07–2.14) | 2.35× (2.30–2.40) | 2.85× (2.82–2.88) | 3.03× (3.00–3.06) | 3.17× (3.14–3.19) | 1.85× (1.83–1.87) | 1.55× (1.54–1.57) | 1.48× (1.47–1.50) | 2.06× (2.04–2.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.88× (1.85–1.92) | 1.63× (1.59–1.68) | 2.05× (2.00–2.10) | 1.96× (1.90–2.02) | 1.99× (1.95–2.03) | 0.95× (0.93–0.97) | 0.80× (0.78–0.81) | 0.76× (0.75–0.78) | 1.06× (1.04–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.17× (1.17–1.17) | 2.29× (2.28–2.30) | 2.05× (2.04–2.05) | 2.14× (2.14–2.15) | 2.45× (2.45–2.46) | 2.67× (2.66–2.67) | 2.81× (2.80–2.81) | 1.07× (1.07–1.07) | 1.19× (1.18–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.11× (1.11–1.12) | 2.18× (2.16–2.20) | 1.92× (1.91–1.94) | 2.59× (2.57–2.61) | 2.27× (2.25–2.28) | 2.41× (2.39–2.42) | 2.54× (2.53–2.55) | 0.97× (0.96–0.97) | 1.07× (1.06–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.30–1.38) | 2.10× (2.07–2.12) | 1.72× (1.69–1.75) | 2.35× (2.30–2.40) | 2.25× (2.21–2.28) | 1.86× (1.83–1.89) | 1.96× (1.93–1.99) | 0.75× (0.74–0.76) | 0.83× (0.81–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.05× (1.02–1.08) | 2.15× (2.09–2.21) | 1.80× (1.75–1.85) | 1.91× (1.86–1.97) | 2.19× (2.13–2.24) | 2.69× (2.61–2.76) | 2.87× (2.79–2.96) | 1.02× (0.99–1.04) | 1.18× (1.15–1.22) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.01× (1.01–1.02) | 2.08× (2.06–2.09) | 1.73× (1.73–1.74) | 2.46× (2.43–2.48) | 2.08× (2.07–2.08) | 2.48× (2.48–2.49) | 2.66× (2.64–2.67) | 0.94× (0.94–0.95) | 1.09× (1.09–1.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.26× (1.22–1.31) | 2.05× (2.00–2.10) | 1.57× (1.54–1.59) | 2.28× (2.22–2.34) | 2.06× (2.03–2.10) | 1.90× (1.87–1.93) | 2.03× (2.00–2.06) | 0.72× (0.71–0.73) | 0.84× (0.82–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.72× (2.68–2.76) | 3.56× (3.51–3.60) | 2.84× (2.80–2.87) | 6.11× (6.03–6.20) | 2.28× (2.26–2.31) | 2.17× (2.14–2.19) | 0.97× (0.96–0.99) | 1.04× (0.99–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.60–0.63) | 2.47× (2.43–2.51) | 4.21× (4.16–4.27) | 2.72× (2.68–2.76) | 5.70× (5.62–5.77) | 2.11× (2.08–2.14) | 2.00× (1.97–2.03) | 0.90× (0.88–0.92) | 0.96× (0.90–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.61) | 2.74× (2.71–2.77) | 4.25× (4.19–4.32) | 2.81× (2.77–2.85) | 5.85× (5.77–5.92) | 2.16× (2.13–2.19) | 2.05× (2.02–2.07) | 0.92× (0.91–0.94) | 0.98× (0.93–1.03) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 1 | 1.52× (1.51–1.53) | 3.01× (2.99–3.03) | 3.27× (3.23–3.30) | 3.28× (3.26–3.30) | 3.52× (3.50–3.55) | 3.58× (3.55–3.60) | 3.59× (3.56–3.62) | 1.16× (1.15–1.16) | 1.35× (1.34–1.35) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 2 | 1.39× (1.38–1.40) | 2.70× (2.67–2.73) | 2.96× (2.94–2.98) | 3.19× (3.16–3.21) | 3.10× (3.06–3.13) | 3.08× (3.06–3.11) | 3.10× (3.06–3.13) | 1.00× (0.99–1.00) | 1.16× (1.15–1.17) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 4 | 1.55× (1.49–1.62) | 2.31× (2.25–2.37) | 2.56× (2.51–2.60) | 2.71× (2.66–2.76) | 2.67× (2.62–2.71) | 2.19× (2.15–2.23) | 2.20× (2.16–2.24) | 0.71× (0.70–0.72) | 0.83× (0.81–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.23× (12.21–12.25) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.38–0.38) | 0.62× (0.55–0.71) | 0.81× (0.80–0.82) | 0.66× (0.66–0.67) | 6.44× (6.42–6.46) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.76× (0.67–0.86) | 0.71× (0.71–0.72) | 0.62× (0.60–0.64) | 4.11× (4.09–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.19) | 0.83× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.33–4.34) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.75) | 0.66× (0.65–0.67) | 8.19× (8.16–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.68) | 0.73× (0.67–0.80) | 1.63× (1.58–1.68) | 0.80× (0.78–0.82) | 5.88× (5.72–6.04) | 0.46× (0.45–0.48) | 0.49× (0.48–0.50) | 0.54× (0.52–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.66× (0.64–0.68) | 0.77× (0.73–0.81) | 1.21× (1.19–1.23) | 0.78× (0.75–0.81) | 3.52× (3.47–3.58) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.73–0.74) | 5.28× (5.27–5.29) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.44) | 0.92× (0.90–0.95) | 0.71× (0.68–0.73) | 3.18× (3.08–3.28) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.64) | 0.45× (0.44–0.45) | 0.86× (0.85–0.88) | 0.78× (0.76–0.81) | 2.49× (2.46–2.53) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.20× (1.20–1.21) | 0.84× (0.83–0.84) | 5.30× (5.29–5.31) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.47× (0.46–0.49) | 0.38× (0.35–0.41) | 0.86× (0.84–0.88) | 0.72× (0.70–0.74) | 2.94× (2.88–3.01) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.47–0.52) | 0.81× (0.80–0.82) | 0.80× (0.78–0.83) | 2.34× (2.32–2.36) | 0.14× (0.14–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.10× (1.10–1.10) | 0.77× (0.77–0.78) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.78) | 5.43× (5.41–5.45) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.66× (1.65–1.66) | 0.78× (0.77–0.78) | 5.46× (5.44–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.40× (0.40–0.41) | 1.51× (1.51–1.51) | 0.65× (0.64–0.65) | 6.78× (6.76–6.79) | 0.40× (0.40–0.40) | 0.43× (0.42–0.44) | 0.66× (0.66–0.67) | 0.33× (0.33–0.33) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 2 | 33 MB | 0.48× (0.48–0.49) | 0.43× (0.39–0.47) | 1.22× (1.20–1.23) | 0.74× (0.73–0.76) | 3.76× (3.71–3.80) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.37× (0.36–0.37) | 0.18× (0.18–0.19) |
| `text` (1.66 MB → 12.2 MB, 7.3×) | 4 | 49 MB | 0.55× (0.53–0.57) | 0.55× (0.51–0.59) | 1.05× (1.02–1.08) | 0.82× (0.79–0.86) | 2.68× (2.60–2.75) | 0.15× (0.15–0.15) | 0.16× (0.15–0.17) | 0.25× (0.24–0.26) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 304 | 3.30 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 562 | 3.51 | 1.06× (1.05–1.07) | 91% | 1.02× (1.02–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 700 | 5.36 | 1.62× (1.59–1.64) | 57% | 1.21× (1.20–1.22) | 1.62× (1.59–1.65) |

**Best:** mibz2 at 4 threads (700 MB/s); lbzip2 at 4 threads (197 MB/s); pbz2 at 4 threads (300 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.10, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 0.90 | 0.02 | 0.09 | 0.299 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3246 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 12.2% |
| MTF + RLE2 | 1.99 | 19.7% |
| T-vector build | 2.12 | 21.0% |
| walk (the one each build uses) | 4.21 | 41.6% |
| of which: segment assembly | 0.15 | 1.5% |
| RLE1 | 0.29 | 2.9% |
| CRC | 0.26 | 2.5% |
| **total** | 10.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.677 | 1.340 | (in its total) | 1.98× (1.98–1.98) |  |
| walk | 1.307 | 2.623 | (in its total) | 2.01× (2.00–2.02) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.054 | (in its total) | 1.18× (1.18–1.19) |  |
| inverse BWT, total | 2.030 | 4.018 | 3.228 | 1.98× (1.97–1.99) | 1.59× (1.58–1.60) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h40m ± 11m45s |
| 4 threads | 43m46s ± 6m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m25s ± 4m35s |
| the scanner alone (its serial part) | 16m53s ± 32.2s |
| the workers alone, over 4 | 32m25s ± 4m35s |

At 4 threads the workers bound it. Per byte: the scanner 9.8 ns a byte of bz2, the workers 4.25 ns a byte of output at 4 threads (CPU, summed over them), 2.74 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,664,166 | 12,160,259 | \`a03172cba7400dd7…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3246.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
