# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-455/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`caad2bd`](<https://github.com/lmmx/mibz2/commit/caad2bd97e236905a15a970b3a3a945877b4316e>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43774654165>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.534 s | 393 | 7.3 | 2.96× (2.94–2.98) | 6.80× (6.75–6.85) | 7.04× (6.97–7.10) | 7.97× (7.93–8.02) | 7.03× (6.97–7.09) | 7.32× (7.28–7.36) | 6.94× (6.89–6.99) | 2.19× (2.17–2.21) | 2.42× (2.41–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.314 s | 667 | 8.0 | 2.84× (2.71–2.97) | 6.10× (5.88–6.33) | 6.85× (6.70–7.00) | 7.11× (6.88–7.35) | 6.28× (6.09–6.48) | 12.29× (11.86–12.74) | 11.65× (11.26–12.06) | 3.68× (3.57–3.79) | 4.07× (3.92–4.23) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.251 s | 835 | 12.3 | 3.04× (2.94–3.13) | 4.52× (4.43–4.61) | 5.55× (5.38–5.72) | 5.53× (5.36–5.70) | 5.37× (5.27–5.48) | 15.60× (15.44–15.77) | 14.79× (14.65–14.93) | 4.67× (4.60–4.74) | 5.17× (5.11–5.23) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 909 | 3.5 | 0.60× (0.59–0.61) | 3.43× (3.28–3.59) | 4.57× (4.44–4.71) | 3.69× (3.66–3.73) | 6.29× (6.15–6.42) | 2.78× (2.75–2.81) | 2.78× (2.75–2.81) | 1.00× (0.97–1.02) | 1.86× (1.84–1.88) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 897 | 3.5 | 0.57× (0.54–0.59) | 2.75× (2.69–2.82) | 5.80× (5.60–6.00) | 3.66× (3.58–3.74) | 6.03× (5.90–6.15) | 2.73× (2.67–2.79) | 2.73× (2.67–2.80) | 0.98× (0.95–1.01) | 1.83× (1.79–1.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 889 | 3.5 | 0.56× (0.54–0.57) | 2.76× (2.64–2.89) | 5.72× (5.56–5.87) | 3.60× (3.52–3.68) | 6.06× (5.95–6.18) | 2.69× (2.63–2.75) | 2.69× (2.63–2.75) | 0.96× (0.93–1.00) | 1.80× (1.76–1.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 588 | 5.2 | 2.76× (2.74–2.78) | 3.05× (3.02–3.08) | 3.75× (3.56–3.96) | 4.18× (4.15–4.21) | 4.51× (4.43–4.59) | 2.82× (2.80–2.85) | 2.31× (2.30–2.33) | 2.14× (2.12–2.16) | 3.04× (3.01–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 850 | 6.6 | 2.25× (2.01–2.51) | 2.68× (2.23–3.21) | 4.04× (3.40–4.79) | 3.91× (3.43–4.45) | 4.58× (3.93–5.34) | 3.79× (3.28–4.39) | 3.11× (2.69–3.60) | 2.88× (2.48–3.34) | 4.08× (3.52–4.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 769 | 12.3 | 1.91× (1.81–2.01) | 1.80× (1.69–1.93) | 2.86× (2.73–3.01) | 2.20× (2.10–2.29) | 4.08× (3.92–4.25) | 3.70× (3.54–3.86) | 3.03× (2.90–3.16) | 2.80× (2.68–2.93) | 3.98× (3.81–4.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.143 s | 56 | 51.9 | 1.38× (1.37–1.39) | 2.62× (2.60–2.64) | 2.34× (2.31–2.38) | 2.42× (2.41–2.43) | 2.75× (2.73–2.76) | 3.00× (2.97–3.02) | 3.02× (3.01–3.03) | 1.26× (1.24–1.29) | 1.42× (1.42–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.086 s | 92 | 58.5 | 1.26× (1.16–1.38) | 2.40× (2.20–2.61) | 2.25× (2.06–2.45) | 2.84× (2.60–3.09) | 2.75× (2.52–3.01) | 4.75× (4.37–5.17) | 4.79× (4.40–5.22) | 2.01× (1.83–2.19) | 2.26× (2.07–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 76.5 | 1.35× (1.32–1.38) | 2.13× (2.08–2.19) | 2.12× (2.02–2.23) | 2.59× (2.54–2.65) | 2.91× (2.85–2.97) | 6.05× (5.96–6.14) | 6.10× (5.98–6.22) | 2.55× (2.48–2.62) | 2.87× (2.81–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.152 s | 63 | 46.2 | 1.53× (1.50–1.55) | 3.03× (2.97–3.09) | 2.59× (2.53–2.65) | 2.75× (2.70–2.81) | 3.15× (3.04–3.26) | 3.68× (3.61–3.74) | 3.92× (3.85–3.99) | 1.54× (1.50–1.58) | 1.79× (1.76–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.092 s | 104 | 52.0 | 1.39× (1.34–1.44) | 2.77× (2.68–2.85) | 2.50× (2.42–2.59) | 3.42× (3.30–3.56) | 3.10× (3.02–3.19) | 6.05× (5.85–6.26) | 6.45× (6.24–6.67) | 2.53× (2.47–2.60) | 2.94× (2.84–3.04) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 72.6 | 1.45× (1.41–1.49) | 2.28× (2.22–2.34) | 2.25× (2.19–2.30) | 3.03× (2.97–3.10) | 2.94× (2.85–3.04) | 7.17× (7.01–7.32) | 7.64× (7.48–7.81) | 3.00× (2.93–3.07) | 3.48× (3.40–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 690 | 4.8 | 0.65× (0.64–0.66) | 2.84× (2.80–2.87) | 4.16× (4.00–4.32) | 3.22× (3.18–3.26) | 7.05× (6.88–7.22) | 2.39× (2.36–2.42) | 2.43× (2.40–2.46) | 0.99× (0.97–1.00) | 2.13× (2.10–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 649 | 5.1 | 0.54× (0.53–0.55) | 2.67× (2.63–2.72) | 4.91× (4.67–5.17) | 3.06× (3.00–3.12) | 6.42× (6.27–6.58) | 2.25× (2.21–2.28) | 2.28× (2.24–2.32) | 0.93× (0.91–0.94) | 2.00× (1.97–2.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 641 | 5.0 | 0.54× (0.53–0.55) | 2.66× (2.64–2.69) | 4.75× (4.53–4.98) | 3.04× (3.01–3.07) | 6.36× (6.21–6.52) | 2.23× (2.20–2.25) | 2.26× (2.24–2.29) | 0.92× (0.91–0.93) | 1.98× (1.96–2.01) | yes |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 1 | 0.074 s | 179 | 16.5 | 1.85× (1.84–1.86) | 3.97× (3.93–4.00) | 4.38× (4.34–4.42) | 4.62× (4.58–4.66) | 4.77× (4.71–4.84) | 4.76× (4.73–4.79) | 4.67× (4.63–4.72) | 1.53× (1.50–1.56) | 1.77× (1.76–1.79) | yes |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 2 | 0.052 s | 254 | 20.6 | 1.52× (1.47–1.57) | 3.39× (3.04–3.79) | 3.83× (3.57–4.11) | 4.38× (4.00–4.80) | 4.28× (4.14–4.43) | 6.78× (6.55–7.01) | 6.66× (6.43–6.88) | 2.18× (2.10–2.26) | 2.53× (2.43–2.62) | yes |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 4 | 0.046 s | 289 | 31.3 | 1.56× (1.51–1.61) | 2.59× (2.52–2.65) | 3.26× (3.13–3.39) | 3.50× (3.35–3.65) | 4.02× (3.89–4.16) | 7.84× (7.62–8.06) | 7.70× (7.49–7.91) | 2.52× (2.42–2.62) | 2.92× (2.84–3.01) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.95× (2.94–2.97) | 6.77× (6.72–6.82) | 7.00× (6.93–7.07) | 7.95× (7.90–7.99) | 7.00× (6.94–7.06) | 7.30× (7.26–7.34) | 6.92× (6.87–6.97) | 2.19× (2.17–2.21) | 2.41× (2.40–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.82× (2.80–2.84) | 6.38× (6.30–6.45) | 6.73× (6.60–6.85) | 7.67× (7.61–7.73) | 6.47× (6.42–6.53) | 6.69× (6.65–6.73) | 6.35× (6.30–6.39) | 2.00× (1.99–2.02) | 2.21× (2.20–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.97× (2.93–3.02) | 4.92× (4.86–4.99) | 5.40× (5.30–5.50) | 5.99× (5.89–6.09) | 5.19× (5.11–5.27) | 4.37× (4.32–4.41) | 4.14× (4.09–4.19) | 1.31× (1.29–1.32) | 1.44× (1.42–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.05× (3.02–3.07) | 4.08× (4.01–4.16) | 3.46× (3.44–3.48) | 5.82× (5.72–5.93) | 2.62× (2.60–2.64) | 2.62× (2.60–2.64) | 0.99× (0.97–1.01) | 1.69× (1.68–1.70) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.64) | 2.84× (2.78–2.90) | 5.07× (4.95–5.19) | 3.41× (3.33–3.48) | 5.52× (5.41–5.64) | 2.54× (2.49–2.60) | 2.55× (2.49–2.60) | 0.96× (0.94–0.99) | 1.65× (1.61–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.61) | 3.12× (3.05–3.19) | 5.05× (4.95–5.15) | 3.42× (3.35–3.49) | 5.64× (5.54–5.75) | 2.56× (2.50–2.61) | 2.56× (2.50–2.61) | 0.97× (0.94–1.00) | 1.65× (1.62–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.64–2.67) | 2.90× (2.88–2.92) | 3.51× (3.34–3.68) | 4.00× (3.97–4.02) | 4.31× (4.24–4.38) | 2.72× (2.69–2.74) | 2.24× (2.22–2.25) | 2.07× (2.06–2.09) | 2.69× (2.66–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.23–2.34) | 2.54× (2.44–2.65) | 3.31× (3.23–3.40) | 3.37× (3.23–3.51) | 3.53× (3.44–3.62) | 2.15× (2.09–2.21) | 1.77× (1.73–1.82) | 1.64× (1.60–1.68) | 2.13× (2.07–2.20) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.01–2.20) | 1.77× (1.74–1.80) | 2.28× (2.23–2.34) | 2.16× (2.10–2.22) | 2.19× (2.15–2.23) | 1.14× (1.11–1.16) | 0.94× (0.91–0.96) | 0.87× (0.85–0.89) | 1.13× (1.10–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.55× (2.55–2.56) | 2.28× (2.25–2.31) | 2.40× (2.39–2.41) | 2.72× (2.71–2.74) | 2.97× (2.94–3.00) | 2.99× (2.98–3.00) | 1.26× (1.23–1.29) | 1.37× (1.36–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.24–1.30) | 2.38× (2.33–2.43) | 2.07× (2.02–2.12) | 2.74× (2.68–2.79) | 2.45× (2.40–2.51) | 2.61× (2.55–2.67) | 2.63× (2.57–2.69) | 1.11× (1.07–1.15) | 1.21× (1.18–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.46–1.50) | 2.29× (2.26–2.33) | 1.90× (1.88–1.93) | 2.54× (2.50–2.58) | 2.49× (2.45–2.53) | 2.02× (1.99–2.05) | 2.04× (2.01–2.06) | 0.86× (0.84–0.87) | 0.93× (0.92–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.49–1.54) | 2.97× (2.91–3.03) | 2.54× (2.48–2.59) | 2.73× (2.68–2.78) | 3.12× (3.01–3.23) | 3.64× (3.58–3.71) | 3.88× (3.81–3.95) | 1.53× (1.49–1.57) | 1.74× (1.71–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.41× (1.40–1.42) | 2.79× (2.77–2.82) | 2.35× (2.33–2.36) | 3.25× (3.21–3.28) | 2.81× (2.79–2.83) | 3.25× (3.22–3.28) | 3.46× (3.43–3.50) | 1.37× (1.34–1.39) | 1.55× (1.54–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.64× (1.61–1.67) | 2.56× (2.53–2.60) | 2.01× (1.99–2.04) | 2.84× (2.79–2.89) | 2.69× (2.65–2.72) | 2.34× (2.30–2.38) | 2.49× (2.46–2.53) | 0.98× (0.97–1.00) | 1.12× (1.10–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.55× (2.53–2.58) | 3.57× (3.50–3.65) | 2.94× (2.91–2.97) | 6.29× (6.15–6.43) | 2.21× (2.19–2.23) | 2.25× (2.23–2.27) | 0.99× (0.97–1.00) | 1.41× (1.40–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.33× (2.30–2.37) | 4.10× (4.03–4.18) | 2.82× (2.77–2.87) | 5.69× (5.55–5.83) | 2.06× (2.02–2.09) | 2.09× (2.05–2.12) | 0.92× (0.90–0.93) | 1.31× (1.29–1.33) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.59× (2.54–2.64) | 4.13× (4.08–4.18) | 2.88× (2.86–2.91) | 5.80× (5.67–5.94) | 2.10× (2.08–2.12) | 2.13× (2.11–2.15) | 0.94× (0.92–0.95) | 1.34× (1.32–1.35) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 1 | 1.83× (1.82–1.84) | 3.82× (3.79–3.85) | 4.22× (4.18–4.26) | 4.52× (4.48–4.56) | 4.67× (4.61–4.74) | 4.65× (4.62–4.68) | 4.57× (4.53–4.61) | 1.51× (1.49–1.54) | 1.64× (1.62–1.65) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 2 | 1.56× (1.52–1.60) | 3.21× (3.17–3.24) | 3.55× (3.48–3.62) | 4.11× (4.03–4.19) | 3.79× (3.71–3.86) | 3.70× (3.62–3.79) | 3.64× (3.55–3.72) | 1.20× (1.17–1.24) | 1.30× (1.28–1.33) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 4 | 1.67× (1.60–1.74) | 2.71× (2.61–2.81) | 3.01× (2.90–3.12) | 3.35× (3.28–3.41) | 3.17× (3.10–3.24) | 2.46× (2.41–2.51) | 2.42× (2.37–2.47) | 0.80× (0.78–0.83) | 0.87× (0.85–0.89) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.31) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.05× (11.02–11.08) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.34) | 0.41× (0.35–0.49) | 0.66× (0.63–0.69) | 0.56× (0.53–0.59) | 5.29× (5.05–5.54) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.31–0.35) | 0.48× (0.41–0.55) | 0.59× (0.56–0.62) | 0.52× (0.49–0.55) | 3.46× (3.30–3.62) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.16× (0.15–0.16) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.00–8.08) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.69× (0.63–0.74) | 1.61× (1.59–1.63) | 0.80× (0.79–0.81) | 5.83× (5.76–5.90) | 0.47× (0.46–0.48) | 0.49× (0.49–0.50) | 0.54× (0.53–0.54) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.68) | 0.71× (0.66–0.76) | 1.15× (1.09–1.21) | 0.71× (0.67–0.75) | 3.37× (3.21–3.53) | 0.26× (0.25–0.27) | 0.27× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.40× (0.38–0.43) | 0.90× (0.88–0.92) | 0.71× (0.69–0.73) | 3.12× (3.06–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.78× (0.76–0.81) | 2.49× (2.44–2.54) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.19× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.22–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.50× (0.48–0.52) | 0.40× (0.38–0.42) | 0.88× (0.86–0.91) | 0.72× (0.69–0.75) | 3.03× (2.95–3.11) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.81× (0.77–0.85) | 2.33× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.35× (5.34–5.36) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.62) | 0.77× (0.77–0.77) | 5.33× (5.31–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.42) | 0.66× (0.66–0.67) | 6.43× (6.41–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.45) | 0.39× (0.36–0.42) | 1.04× (1.00–1.08) | 0.66× (0.64–0.68) | 3.28× (3.21–3.35) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.61 MB → 13.3 MB, 5.1×) | 4 | 58 MB | 0.49× (0.47–0.51) | 0.47× (0.45–0.50) | 0.89× (0.86–0.92) | 0.73× (0.69–0.77) | 2.28× (2.20–2.36) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 393 | 2.56 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 667 | 2.79 | 1.09× (1.08–1.10) | 84% | 1.03× (1.01–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 835 | 4.28 | 1.67× (1.65–1.69) | 53% | 1.22× (1.20–1.23) | 1.68× (1.65–1.71) |

**Best:** mibz2 at 4 threads (835 MB/s); lbzip2 at 4 threads (185 MB/s); pbz2 at 4 threads (278 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.20, lbzip2 1t 0.06, lbzip2 2t 0.25, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.70 | 0.02 | 0.08 | 0.251 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2869 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.9% |
| MTF + RLE2 | 1.52 | 17.7% |
| T-vector build | 1.49 | 17.3% |
| walk (the one each build uses) | 4.01 | 46.7% |
| of which: segment assembly | 0.13 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.58 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.504 | 1.369 | (in its total) | 2.72× (2.72–2.72) |  |
| walk | 1.348 | 2.640 | (in its total) | 1.96× (1.96–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.050 | (in its total) | 1.09× (1.06–1.12) |  |
| inverse BWT, total | 1.897 | 4.058 | 2.466 | 2.14× (2.14–2.15) | 1.30× (1.30–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m47s |
| 4 threads | 36m28s ± 5m08s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m08s ± 3m32s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 25m08s ± 3m32s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 2.05 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,613,130 | 13,279,610 | \`c66d0e12350335b4…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2868.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
