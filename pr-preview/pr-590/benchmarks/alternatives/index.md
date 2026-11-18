# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-590/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`3502dd9`](<https://github.com/lmmx/mibz2/commit/3502dd9bbc75cee4feadb5cdebd487a5b937c822>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44379988999>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.312 s | 673 | 6.6 | 2.85× (2.82–2.89) | 6.96× (6.85–7.07) | 7.47× (7.37–7.57) | 8.36× (8.20–8.51) | 7.40× (7.25–7.55) | 7.72× (7.58–7.85) | 7.29× (7.20–7.38) | 2.20× (2.15–2.24) | 2.52× (2.50–2.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.189 s | 1,109 | 7.6 | 2.85× (2.68–3.02) | 6.26× (5.92–6.62) | 7.61× (7.17–8.08) | 7.40× (6.97–7.86) | 6.57× (6.21–6.94) | 12.51× (11.79–13.26) | 11.82× (11.15–12.52) | 3.56× (3.40–3.73) | 4.09× (3.86–4.33) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.160 s | 1,315 | 11.8 | 2.87× (2.76–3.00) | 4.43× (4.26–4.61) | 5.95× (5.66–6.24) | 5.26× (5.13–5.39) | 5.33× (5.17–5.50) | 15.05× (14.74–15.38) | 14.22× (13.92–14.54) | 4.29× (4.14–4.44) | 4.92× (4.80–5.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,360 | 3.6 | 0.57× (0.56–0.58) | 3.47× (3.40–3.55) | 4.46× (4.26–4.67) | 3.71× (3.62–3.81) | 6.44× (6.30–6.60) | 2.79× (2.73–2.85) | 2.94× (2.64–3.27) | 1.00× (0.98–1.02) | 2.09× (2.04–2.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,333 | 3.8 | 0.52× (0.51–0.53) | 3.07× (2.83–3.33) | 5.78× (5.54–6.03) | 3.72× (3.57–3.87) | 6.32× (6.17–6.48) | 2.75× (2.68–2.82) | 2.90× (2.61–3.22) | 0.98× (0.96–1.00) | 2.06× (2.01–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,278 | 3.8 | 0.46× (0.38–0.56) | 2.84× (2.33–3.47) | 5.43× (4.31–6.84) | 3.26× (2.67–3.97) | 5.67× (4.64–6.93) | 2.43× (1.98–2.97) | 2.56× (2.09–3.13) | 0.87× (0.71–1.05) | 1.81× (1.48–2.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,016 | 4.8 | 2.94× (2.87–3.01) | 3.65× (3.54–3.77) | 4.25× (4.13–4.37) | 4.53× (4.44–4.61) | 5.34× (5.26–5.42) | 3.07× (2.98–3.16) | 2.57× (2.51–2.63) | 2.16× (2.12–2.20) | 3.17× (3.07–3.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,366 | 6.6 | 2.19× (1.86–2.58) | 2.88× (2.53–3.29) | 4.11× (3.63–4.67) | 3.79× (3.05–4.72) | 5.70× (4.87–6.67) | 3.96× (3.48–4.52) | 3.31× (2.90–3.78) | 2.79× (2.46–3.16) | 4.09× (3.56–4.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,037 | 13.4 | 1.51× (1.36–1.68) | 1.67× (1.54–1.81) | 2.60× (2.30–2.93) | 1.87× (1.72–2.02) | 3.67× (3.36–4.00) | 3.04× (2.79–3.32) | 2.54× (2.36–2.74) | 2.14× (1.97–2.32) | 3.14× (2.88–3.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.086 s | 93 | 49.0 | 1.37× (1.34–1.40) | 3.08× (2.91–3.26) | 2.77× (2.70–2.84) | 2.67× (2.59–2.74) | 3.01× (2.93–3.10) | 3.44× (3.36–3.52) | 3.44× (3.35–3.54) | 1.34× (1.29–1.40) | 1.48× (1.44–1.51) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 148 | 55.0 | 1.25× (1.22–1.28) | 2.86× (2.77–2.96) | 2.74× (2.53–2.96) | 3.31× (3.16–3.47) | 3.04× (2.95–3.14) | 5.48× (5.35–5.60) | 5.47× (5.32–5.63) | 2.14× (2.08–2.20) | 2.35× (2.27–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 179 | 72.6 | 1.29× (1.26–1.33) | 2.50× (2.33–2.67) | 2.49× (2.35–2.63) | 2.80× (2.71–2.89) | 3.18× (3.07–3.28) | 6.68× (6.47–6.89) | 6.67× (6.50–6.85) | 2.61× (2.46–2.76) | 2.86× (2.78–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 111 | 40.9 | 1.47× (1.37–1.57) | 3.68× (3.42–3.97) | 3.04× (2.92–3.17) | 3.03× (2.80–3.27) | 3.51× (3.23–3.80) | 4.34× (4.05–4.66) | 4.71× (4.38–5.06) | 1.56× (1.46–1.67) | 1.82× (1.68–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 168 | 48.5 | 1.34× (1.26–1.43) | 3.51× (3.05–4.05) | 2.78× (2.64–2.93) | 3.69× (3.52–3.88) | 3.26× (3.12–3.40) | 6.83× (6.57–7.10) | 7.40× (7.07–7.75) | 2.46× (2.36–2.56) | 2.86× (2.69–3.05) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.048 s | 202 | 68.1 | 1.39× (1.34–1.44) | 2.76× (2.56–2.98) | 2.56× (2.45–2.67) | 3.46× (3.21–3.72) | 3.27× (3.13–3.42) | 8.23× (8.00–8.47) | 8.93× (8.65–9.21) | 2.96× (2.87–3.06) | 3.45× (3.32–3.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 5.1 | 0.62× (0.61–0.64) | 3.12× (3.04–3.20) | 4.39× (4.22–4.56) | 3.16× (3.09–3.23) | 7.25× (6.79–7.74) | 2.34× (2.30–2.39) | 2.36× (2.31–2.42) | 0.98× (0.96–1.00) | 2.09× (2.03–2.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 980 | 5.4 | 0.51× (0.50–0.53) | 3.00× (2.93–3.07) | 5.23× (4.95–5.53) | 3.08× (2.98–3.19) | 6.86× (6.66–7.06) | 2.25× (2.20–2.29) | 2.27× (2.20–2.33) | 0.94× (0.91–0.96) | 2.00× (1.95–2.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 948 | 5.4 | 0.50× (0.49–0.51) | 2.95× (2.86–3.03) | 5.22× (4.90–5.57) | 3.21× (2.77–3.71) | 6.67× (6.57–6.76) | 2.20× (2.15–2.25) | 2.22× (2.19–2.26) | 0.92× (0.90–0.94) | 1.96× (1.90–2.02) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.045 s | 289 | 16.0 | 1.77× (1.71–1.83) | 4.22× (4.09–4.36) | 4.75× (4.53–4.98) | 4.71× (4.59–4.84) | 5.01× (4.81–5.23) | 5.09× (4.96–5.23) | 4.89× (4.78–5.00) | 1.52× (1.47–1.56) | 1.77× (1.71–1.83) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.034 s | 382 | 20.7 | 1.46× (1.37–1.56) | 3.20× (2.99–3.43) | 3.81× (3.69–3.94) | 4.33× (4.04–4.63) | 4.21× (4.08–4.35) | 6.75× (6.55–6.96) | 6.49× (6.26–6.72) | 2.01× (1.93–2.09) | 2.35× (2.25–2.46) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.032 s | 416 | 30.9 | 1.28× (1.23–1.33) | 2.41× (2.29–2.54) | 3.28× (3.02–3.55) | 3.27× (3.08–3.47) | 3.77× (3.68–3.85) | 7.34× (7.18–7.49) | 7.05× (6.87–7.23) | 2.19× (2.13–2.24) | 2.55× (2.44–2.67) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.84× (2.80–2.88) | 6.91× (6.81–7.02) | 7.42× (7.33–7.51) | 8.32× (8.16–8.47) | 7.37× (7.22–7.52) | 7.68× (7.55–7.81) | 7.26× (7.17–7.35) | 2.19× (2.15–2.24) | 2.50× (2.48–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.63× (2.48–2.80) | 6.36× (5.97–6.77) | 6.89× (6.58–7.22) | 7.74× (7.31–8.19) | 6.52× (6.20–6.86) | 6.62× (6.28–6.97) | 6.25× (5.93–6.59) | 1.89× (1.82–1.96) | 2.15× (2.05–2.27) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.83× (2.71–2.94) | 5.02× (4.83–5.21) | 5.65× (5.44–5.86) | 5.97× (5.84–6.10) | 5.26× (5.08–5.44) | 4.38× (4.28–4.48) | 4.14× (4.03–4.25) | 1.25× (1.20–1.30) | 1.43× (1.39–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.05× (2.95–3.14) | 3.92× (3.81–4.05) | 3.43× (3.34–3.53) | 5.88× (5.73–6.04) | 2.60× (2.53–2.67) | 2.75× (2.47–3.07) | 1.00× (0.98–1.02) | 1.82× (1.78–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.54–0.60) | 2.77× (2.63–2.91) | 4.91× (4.64–5.21) | 3.34× (3.13–3.57) | 5.57× (5.27–5.89) | 2.47× (2.33–2.61) | 2.61× (2.35–2.90) | 0.95× (0.90–1.00) | 1.73× (1.64–1.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.42–0.61) | 2.76× (2.29–3.34) | 4.76× (3.80–5.95) | 3.08× (2.56–3.71) | 5.24× (4.35–6.33) | 2.30× (1.90–2.78) | 2.44× (2.03–2.93) | 0.88× (0.73–1.06) | 1.61× (1.33–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.78× (2.72–2.84) | 3.15× (3.08–3.22) | 3.77× (3.69–3.85) | 4.24× (4.17–4.32) | 4.99× (4.93–5.06) | 2.91× (2.83–2.99) | 2.44× (2.39–2.50) | 2.07× (2.03–2.11) | 2.77× (2.71–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.12× (1.91–2.36) | 2.44× (2.19–2.71) | 2.93× (2.64–3.25) | 3.22× (2.89–3.59) | 3.86× (3.28–4.53) | 2.04× (1.83–2.27) | 1.71× (1.54–1.91) | 1.45× (1.31–1.60) | 1.94× (1.75–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.82× (1.67–1.99) | 1.60× (1.49–1.72) | 1.94× (1.80–2.08) | 1.87× (1.73–2.03) | 2.06× (1.93–2.19) | 1.02× (0.96–1.08) | 0.86× (0.81–0.91) | 0.72× (0.68–0.77) | 0.97× (0.91–1.04) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.32–1.39) | 3.00× (2.82–3.18) | 2.67× (2.60–2.73) | 2.62× (2.54–2.71) | 2.96× (2.88–3.05) | 3.39× (3.31–3.47) | 3.38× (3.28–3.49) | 1.33× (1.29–1.38) | 1.41× (1.38–1.46) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.29) | 2.76× (2.72–2.80) | 2.43× (2.26–2.62) | 3.05× (2.98–3.13) | 2.68× (2.61–2.75) | 2.99× (2.93–3.05) | 2.99× (2.90–3.07) | 1.18× (1.14–1.22) | 1.25× (1.22–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.38–1.46) | 2.68× (2.54–2.83) | 2.19× (2.07–2.31) | 2.80× (2.71–2.89) | 2.69× (2.58–2.81) | 2.25× (2.19–2.31) | 2.25× (2.19–2.30) | 0.89× (0.84–0.94) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.35–1.56) | 3.57× (3.32–3.84) | 2.95× (2.82–3.08) | 2.99× (2.76–3.23) | 3.46× (3.19–3.75) | 4.27× (3.98–4.59) | 4.63× (4.30–4.99) | 1.55× (1.45–1.66) | 1.74× (1.61–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.31–1.45) | 3.46× (3.10–3.86) | 2.55× (2.47–2.63) | 3.53× (3.42–3.64) | 2.96× (2.87–3.05) | 3.69× (3.58–3.81) | 4.00× (3.86–4.14) | 1.34× (1.30–1.38) | 1.50× (1.43–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.51–1.62) | 3.11× (2.87–3.36) | 2.25× (2.14–2.37) | 3.26× (3.02–3.53) | 2.92× (2.86–2.98) | 2.70× (2.63–2.78) | 2.93× (2.83–3.02) | 0.98× (0.95–1.01) | 1.10× (1.06–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.59× (2.54–2.64) | 3.41× (3.34–3.48) | 2.84× (2.77–2.90) | 6.32× (5.93–6.75) | 2.14× (2.10–2.19) | 2.16× (2.12–2.20) | 0.98× (0.96–1.00) | 1.48× (1.44–1.51) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.61) | 2.34× (2.30–2.39) | 4.13× (3.90–4.36) | 2.80× (2.72–2.88) | 5.93× (5.76–6.10) | 2.03× (1.99–2.06) | 2.04× (1.99–2.09) | 0.93× (0.91–0.95) | 1.40× (1.37–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.52× (2.48–2.56) | 4.19× (3.91–4.49) | 3.02× (2.60–3.51) | 5.95× (5.87–6.03) | 2.06× (2.02–2.10) | 2.07× (2.04–2.10) | 0.94× (0.92–0.96) | 1.42× (1.40–1.44) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.75× (1.68–1.81) | 4.01× (3.91–4.12) | 4.49× (4.28–4.72) | 4.58× (4.46–4.70) | 4.87× (4.67–5.07) | 4.94× (4.81–5.07) | 4.75× (4.64–4.86) | 1.50× (1.45–1.54) | 1.63× (1.58–1.70) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.47× (1.42–1.52) | 3.36× (3.21–3.52) | 3.62× (3.54–3.70) | 4.26× (3.92–4.63) | 3.80× (3.73–3.88) | 3.82× (3.74–3.91) | 3.67× (3.60–3.75) | 1.16× (1.13–1.18) | 1.26× (1.22–1.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.49× (1.45–1.53) | 2.75× (2.63–2.87) | 3.14× (2.83–3.48) | 3.40× (3.22–3.59) | 3.17× (3.08–3.28) | 2.54× (2.49–2.59) | 2.44× (2.40–2.48) | 0.77× (0.75–0.79) | 0.84× (0.82–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.79× (0.77–0.80) | 10.99× (10.98–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.32–0.35) | 0.47× (0.41–0.53) | 0.69× (0.66–0.72) | 0.58× (0.56–0.61) | 5.52× (5.27–5.79) | 0.13× (0.13–0.14) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.45× (0.40–0.50) | 0.60× (0.57–0.63) | 0.52× (0.50–0.55) | 3.45× (3.28–3.63) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.67× (0.66–0.68) | 8.03× (8.01–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.67–0.73) | 0.68× (0.62–0.73) | 1.62× (1.58–1.67) | 0.81× (0.79–0.84) | 5.89× (5.72–6.06) | 0.48× (0.46–0.49) | 0.50× (0.49–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.61× (0.57–0.64) | 0.71× (0.67–0.76) | 1.10× (1.04–1.16) | 0.68× (0.66–0.71) | 3.22× (3.07–3.39) | 0.25× (0.24–0.26) | 0.26× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.74× (0.73–0.74) | 5.19× (5.17–5.21) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.53) | 0.41× (0.37–0.44) | 0.93× (0.90–0.96) | 0.74× (0.73–0.76) | 3.22× (3.13–3.31) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.81× (0.78–0.84) | 2.51× (2.44–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.82–0.83) | 5.19× (5.18–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.40) | 0.82× (0.80–0.85) | 0.71× (0.68–0.74) | 2.84× (2.74–2.94) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.51× (0.49–0.53) | 0.81× (0.79–0.82) | 0.82× (0.79–0.85) | 2.32× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.07× (1.07–1.07) | 0.77× (0.77–0.77) | 5.37× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.28× (5.27–5.30) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.41× (1.40–1.41) | 0.66× (0.66–0.67) | 6.38× (6.36–6.39) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.62× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.42× (0.39–0.46) | 1.14× (1.10–1.17) | 0.74× (0.72–0.75) | 3.63× (3.57–3.70) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.18× (0.17–0.18) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 54 MB | 0.52× (0.51–0.54) | 0.48× (0.44–0.53) | 0.94× (0.91–0.97) | 0.78× (0.75–0.81) | 2.41× (2.34–2.49) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 673 | 1.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,109 | 1.71 | 1.16× (1.10–1.22) | 81% | 1.07× (1.04–1.09) | 1.08× (1.05–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 1,315 | 2.66 | 1.75× (1.71–1.80) | 49% | 1.27× (1.22–1.33) | 1.74× (1.66–1.83) |

**Best:** mibz2 at 4 threads (1,315 MB/s); lbzip2 at 4 threads (299 MB/s); pbz2 at 4 threads (458 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.24, lbzip2 1t 0.08, lbzip2 2t 0.28, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.55. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 15%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.41 | 0.01 | 0.08 | 0.160 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4438 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.05 | 14.4% |
| MTF + RLE2 | 1.46 | 19.9% |
| T-vector build | 1.36 | 18.5% |
| walk (the one each build uses) | 3.05 | 41.5% |
| of which: segment assembly | 0.13 | 1.7% |
| RLE1 | 0.16 | 2.1% |
| CRC | 0.26 | 3.5% |
| **total** | 7.34 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.297 | 0.729 | (in its total) | 2.44× (2.28–2.60) |  |
| walk | 0.639 | 1.619 | (in its total) | 2.52× (2.34–2.71) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.029 | 0.032 | (in its total) | 1.06× (0.95–1.18) |  |
| inverse BWT, total | 0.965 | 2.380 | 1.342 | 2.45× (2.28–2.63) | 1.35× (1.27–1.45) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m14s ± 5m10s |
| 4 threads | 23m12s ± 3m18s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m35s ± 2m09s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 14m35s ± 2m09s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 1.91 ns a byte of output at 4 threads (CPU, summed over them), 1.20 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,747 | 13,092,730 | \`9b1907f0616f895e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4438.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
