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

The latest record: [`a2e859c`](<https://github.com/lmmx/mibz2/commit/a2e859c235c870577fb084baedf52830d8df41d3>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44084634620>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.519 s | 404 | 7.2 | 2.91× (2.90–2.92) | 6.66× (6.63–6.68) | 6.85× (6.82–6.88) | 7.44× (7.40–7.48) | 6.70× (6.63–6.76) | 7.30× (7.25–7.35) | 6.74× (6.71–6.77) | 2.14× (2.13–2.15) | 2.46× (2.45–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.292 s | 718 | 7.9 | 2.90× (2.74–3.06) | 5.97× (5.70–6.26) | 7.06× (6.81–7.32) | 6.86× (6.51–7.22) | 6.18× (5.91–6.46) | 12.55× (11.94–13.19) | 11.58× (11.05–12.15) | 3.68× (3.51–3.85) | 4.23× (4.03–4.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.242 s | 866 | 12.2 | 2.93× (2.88–2.97) | 4.46× (4.37–4.55) | 5.50× (5.41–5.60) | 5.06× (4.99–5.12) | 5.06× (4.99–5.12) | 15.64× (15.44–15.86) | 14.44× (14.26–14.62) | 4.59× (4.53–4.65) | 5.27× (5.21–5.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,000 | 3.2 | 0.63× (0.58–0.68) | 3.71× (3.54–3.90) | 4.56× (4.43–4.70) | 3.63× (3.59–3.67) | 6.21× (6.15–6.27) | 3.03× (3.00–3.06) | 2.71× (2.68–2.74) | 1.02× (1.00–1.04) | 1.54× (1.52–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 973 | 3.3 | 0.57× (0.56–0.58) | 3.00× (2.96–3.05) | 5.95× (5.85–6.05) | 3.60× (3.52–3.68) | 6.04× (5.97–6.11) | 2.96× (2.92–3.00) | 2.65× (2.61–2.69) | 0.99× (0.98–1.01) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 959 | 3.3 | 0.57× (0.56–0.58) | 3.00× (2.95–3.05) | 5.99× (5.82–6.17) | 3.54× (3.49–3.59) | 6.04× (5.95–6.12) | 2.94× (2.90–2.99) | 2.63× (2.59–2.67) | 0.99× (0.97–1.01) | 1.49× (1.47–1.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 590 | 5.2 | 2.64× (2.62–2.66) | 3.06× (3.04–3.09) | 4.00× (3.92–4.07) | 3.84× (3.80–3.89) | 4.16× (4.11–4.21) | 2.58× (2.56–2.61) | 2.14× (2.12–2.16) | 2.03× (2.01–2.06) | 3.02× (2.95–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 871 | 6.7 | 2.13× (1.91–2.38) | 2.69× (2.41–3.01) | 4.04× (3.59–4.56) | 3.22× (2.94–3.53) | 4.56× (4.14–5.03) | 3.68× (3.34–4.06) | 3.05× (2.76–3.37) | 2.90× (2.63–3.20) | 4.31× (3.82–4.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 789 | 12.2 | 1.89× (1.84–1.94) | 1.97× (1.85–2.10) | 2.94× (2.82–3.07) | 2.25× (2.14–2.37) | 3.85× (3.75–3.95) | 3.42× (3.33–3.51) | 2.83× (2.76–2.91) | 2.69× (2.63–2.76) | 4.00× (3.86–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 47.4 | 1.39× (1.37–1.40) | 2.76× (2.73–2.79) | 2.47× (2.42–2.51) | 2.50× (2.48–2.53) | 2.88× (2.85–2.92) | 3.20× (3.17–3.24) | 3.18× (3.14–3.21) | 1.24× (1.23–1.26) | 1.42× (1.40–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 102 | 53.6 | 1.31× (1.29–1.32) | 2.59× (2.54–2.63) | 2.47× (2.43–2.50) | 3.11× (3.06–3.16) | 2.99× (2.96–3.02) | 5.25× (5.20–5.30) | 5.20× (5.15–5.25) | 2.04× (2.01–2.07) | 2.33× (2.29–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 119 | 72.1 | 1.35× (1.31–1.38) | 2.16× (2.11–2.21) | 2.07× (1.99–2.15) | 2.58× (2.48–2.69) | 2.84× (2.77–2.91) | 6.12× (5.98–6.27) | 6.07× (5.92–6.22) | 2.38× (2.31–2.44) | 2.71× (2.66–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 68 | 43.5 | 1.52× (1.51–1.52) | 3.10× (3.07–3.13) | 2.63× (2.60–2.67) | 2.69× (2.66–2.71) | 3.15× (3.12–3.18) | 3.71× (3.70–3.73) | 3.93× (3.91–3.95) | 1.42× (1.41–1.43) | 1.71× (1.70–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 112 | 49.6 | 1.40× (1.29–1.51) | 2.77× (2.60–2.94) | 2.49× (2.34–2.64) | 3.46× (3.26–3.68) | 3.07× (2.92–3.23) | 6.02× (5.71–6.34) | 6.37× (6.05–6.71) | 2.30× (2.19–2.42) | 2.77× (2.64–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 s | 126 | 68.7 | 1.44× (1.40–1.47) | 2.32× (2.20–2.45) | 2.24× (2.16–2.33) | 2.95× (2.87–3.03) | 2.79× (2.73–2.86) | 6.90× (6.76–7.05) | 7.30× (7.12–7.49) | 2.64× (2.57–2.71) | 3.18× (3.10–3.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 758 | 4.4 | 0.68× (0.68–0.69) | 3.18× (3.02–3.34) | 4.56× (4.52–4.59) | 3.19× (3.17–3.21) | 6.92× (6.86–6.98) | 2.60× (2.58–2.62) | 2.41× (2.33–2.50) | 1.00× (0.99–1.00) | 1.56× (1.55–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 4.8 | 0.57× (0.56–0.57) | 2.89× (2.85–2.93) | 4.97× (4.90–5.04) | 2.98× (2.95–3.02) | 6.41× (6.32–6.50) | 2.41× (2.38–2.44) | 2.24× (2.15–2.33) | 0.92× (0.91–0.94) | 1.45× (1.43–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 709 | 4.6 | 0.58× (0.57–0.59) | 2.93× (2.89–2.97) | 5.03× (4.96–5.09) | 3.03× (3.00–3.07) | 6.41× (6.33–6.50) | 2.43× (2.40–2.47) | 2.26× (2.18–2.35) | 0.93× (0.92–0.95) | 1.47× (1.45–1.49) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 0.065 s | 191 | 15.7 | 2.03× (2.01–2.04) | 4.20× (4.18–4.22) | 4.60× (4.54–4.66) | 4.58× (4.55–4.61) | 4.83× (4.79–4.88) | 5.38× (5.34–5.41) | 4.87× (4.83–4.92) | 1.54× (1.53–1.55) | 1.88× (1.84–1.91) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 0.048 s | 258 | 20.1 | 1.53× (1.48–1.58) | 3.35× (3.02–3.72) | 3.74× (3.55–3.94) | 4.15× (4.01–4.30) | 4.16× (3.94–4.39) | 7.35× (7.11–7.60) | 6.66× (6.46–6.87) | 2.11× (2.04–2.18) | 2.57× (2.46–2.67) | yes |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 0.043 s | 293 | 31.0 | 1.46× (1.42–1.50) | 2.29× (2.19–2.40) | 2.97× (2.86–3.08) | 3.34× (3.24–3.45) | 3.57× (3.48–3.67) | 8.23× (8.02–8.44) | 7.45× (7.27–7.64) | 2.36× (2.30–2.42) | 2.87× (2.78–2.96) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.90–2.92) | 6.62× (6.60–6.65) | 6.82× (6.79–6.84) | 7.42× (7.38–7.45) | 6.68× (6.61–6.74) | 7.28× (7.23–7.32) | 6.72× (6.68–6.75) | 2.14× (2.13–2.14) | 2.44× (2.43–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.75–2.80) | 6.19× (6.14–6.24) | 6.58× (6.50–6.67) | 7.20× (7.10–7.30) | 6.17× (6.14–6.20) | 6.64× (6.59–6.70) | 6.13× (6.11–6.16) | 1.95× (1.94–1.96) | 2.23× (2.22–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.88× (2.84–2.91) | 4.80× (4.74–4.86) | 5.12× (5.05–5.18) | 5.45× (5.37–5.52) | 4.81× (4.75–4.87) | 4.34× (4.27–4.40) | 4.00× (3.95–4.05) | 1.27× (1.26–1.29) | 1.46× (1.43–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.61–0.71) | 3.32× (3.29–3.35) | 4.14× (4.06–4.23) | 3.39× (3.36–3.42) | 5.74× (5.70–5.78) | 2.84× (2.82–2.87) | 2.55× (2.53–2.57) | 1.01× (0.99–1.03) | 1.06× (1.02–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.61–0.62) | 3.08× (3.04–3.12) | 5.38× (5.31–5.45) | 3.34× (3.27–3.41) | 5.52× (5.46–5.58) | 2.74× (2.71–2.78) | 2.46× (2.43–2.49) | 0.98× (0.96–0.99) | 1.03× (0.98–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.61) | 3.39× (3.33–3.45) | 5.48× (5.35–5.61) | 3.36× (3.31–3.40) | 5.61× (5.52–5.69) | 2.78× (2.74–2.83) | 2.50× (2.46–2.54) | 0.99× (0.97–1.01) | 1.04× (1.00–1.09) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.55× (2.52–2.58) | 2.81× (2.79–2.84) | 3.61× (3.49–3.73) | 3.68× (3.64–3.72) | 3.98× (3.94–4.03) | 2.49× (2.47–2.52) | 2.08× (2.06–2.10) | 1.98× (1.95–2.00) | 2.71× (2.68–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.18× (2.14–2.22) | 2.47× (2.43–2.52) | 2.97× (2.92–3.02) | 3.06× (3.01–3.11) | 3.27× (3.21–3.33) | 1.96× (1.92–1.99) | 1.63× (1.60–1.66) | 1.55× (1.52–1.58) | 2.12× (2.09–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.11× (1.99–2.24) | 1.83× (1.76–1.90) | 2.26× (2.17–2.36) | 2.14× (2.04–2.24) | 2.15× (2.05–2.25) | 1.06× (1.02–1.11) | 0.89× (0.85–0.93) | 0.84× (0.81–0.88) | 1.15× (1.10–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.39) | 2.68× (2.65–2.71) | 2.41× (2.38–2.45) | 2.48× (2.45–2.51) | 2.85× (2.82–2.89) | 3.17× (3.14–3.21) | 3.14× (3.11–3.17) | 1.24× (1.23–1.25) | 1.38× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.50× (2.48–2.52) | 2.21× (2.19–2.23) | 2.93× (2.90–2.96) | 2.59× (2.58–2.61) | 2.82× (2.79–2.84) | 2.79× (2.76–2.82) | 1.10× (1.09–1.12) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.49–1.56) | 2.33× (2.31–2.36) | 1.88× (1.85–1.91) | 2.60× (2.56–2.64) | 2.47× (2.43–2.51) | 2.08× (2.05–2.11) | 2.06× (2.04–2.09) | 0.82× (0.80–0.83) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.50–1.52) | 3.05× (3.03–3.06) | 2.57× (2.53–2.60) | 2.67× (2.64–2.69) | 3.12× (3.09–3.15) | 3.67× (3.66–3.69) | 3.89× (3.87–3.91) | 1.41× (1.40–1.43) | 1.67× (1.66–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.41) | 2.79× (2.75–2.82) | 2.33× (2.30–2.36) | 3.22× (3.19–3.26) | 2.78× (2.75–2.80) | 3.23× (3.20–3.26) | 3.42× (3.38–3.45) | 1.24× (1.23–1.25) | 1.46× (1.45–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.59–1.74) | 2.58× (2.48–2.68) | 1.99× (1.95–2.04) | 2.88× (2.80–2.95) | 2.60× (2.55–2.65) | 2.32× (2.28–2.37) | 2.46× (2.42–2.50) | 0.89× (0.88–0.91) | 1.05× (1.03–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.72–0.73) | 2.78× (2.75–2.81) | 3.64× (3.62–3.66) | 2.92× (2.90–2.94) | 6.19× (6.14–6.23) | 2.40× (2.39–2.42) | 2.24× (2.17–2.32) | 1.00× (0.99–1.01) | 1.00× (0.97–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.52× (2.49–2.55) | 4.25× (4.20–4.30) | 2.75× (2.72–2.78) | 5.66× (5.58–5.73) | 2.19× (2.16–2.22) | 2.04× (1.96–2.13) | 0.91× (0.90–0.92) | 0.91× (0.89–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.63) | 2.81× (2.75–2.87) | 4.36× (4.30–4.42) | 2.87× (2.84–2.90) | 5.80× (5.74–5.87) | 2.27× (2.24–2.30) | 2.12× (2.04–2.20) | 0.94× (0.93–0.96) | 0.94× (0.92–0.97) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 1.99× (1.98–2.00) | 4.04× (4.02–4.07) | 4.39× (4.36–4.43) | 4.48× (4.45–4.51) | 4.72× (4.67–4.76) | 5.25× (5.21–5.28) | 4.76× (4.71–4.80) | 1.52× (1.52–1.53) | 1.81× (1.79–1.82) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 1.65× (1.63–1.67) | 3.34× (3.28–3.39) | 3.65× (3.58–3.73) | 4.03× (3.98–4.08) | 3.82× (3.73–3.90) | 4.11× (4.05–4.16) | 3.72× (3.67–3.78) | 1.19× (1.18–1.21) | 1.42× (1.40–1.43) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 1.78× (1.69–1.86) | 2.66× (2.54–2.79) | 2.93× (2.82–3.06) | 3.20× (3.08–3.31) | 3.08× (2.96–3.21) | 2.71× (2.60–2.82) | 2.45× (2.35–2.56) | 0.79× (0.75–0.82) | 0.93× (0.89–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.29× (0.29–0.30) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.00× (10.99–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.44× (0.37–0.53) | 0.68× (0.67–0.70) | 0.57× (0.56–0.58) | 5.42× (5.30–5.54) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.35) | 0.64× (0.56–0.74) | 0.61× (0.58–0.63) | 0.53× (0.52–0.55) | 3.52× (3.43–3.62) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.03× (8.02–8.05) | 0.65× (0.64–0.66) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.72× (0.67–0.78) | 1.60× (1.57–1.63) | 0.80× (0.78–0.82) | 5.79× (5.69–5.90) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.68) | 0.72× (0.68–0.75) | 1.17× (1.14–1.19) | 0.72× (0.70–0.74) | 3.42× (3.34–3.50) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.74) | 5.18× (5.17–5.18) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.41× (0.38–0.44) | 0.91× (0.89–0.94) | 0.73× (0.71–0.75) | 3.16× (3.09–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.80× (0.77–0.82) | 2.50× (2.43–2.57) | 0.15× (0.15–0.15) | 0.16× (0.15–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.35× (0.33–0.38) | 0.84× (0.82–0.86) | 0.70× (0.67–0.73) | 2.87× (2.80–2.94) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.49× (0.46–0.53) | 0.81× (0.80–0.83) | 0.81× (0.79–0.83) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.08–1.08) | 0.77× (0.76–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.28× (5.26–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.66) | 6.39× (6.37–6.41) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 2 | 36 MB | 0.47× (0.45–0.49) | 0.41× (0.38–0.45) | 1.09× (1.05–1.14) | 0.71× (0.69–0.74) | 3.43× (3.30–3.57) | 0.21× (0.20–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.46 MB → 12.5 MB, 5.1×) | 4 | 57 MB | 0.49× (0.46–0.51) | 0.48× (0.44–0.53) | 0.89× (0.85–0.93) | 0.71× (0.68–0.75) | 2.27× (2.19–2.36) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 404 | 2.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 718 | 2.72 | 1.09× (1.09–1.10) | 86% | 1.02× (1.02–1.03) | 1.04× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 866 | 4.20 | 1.68× (1.65–1.70) | 54% | 1.22× (1.21–1.22) | 1.66× (1.63–1.68) |

**Best:** mibz2 at 4 threads (866 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (297 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.76, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.29, lbzip2 4t 0.57, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.67 | 0.02 | 0.10 | 0.242 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2907 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 12.7% |
| MTF + RLE2 | 1.39 | 17.3% |
| T-vector build | 1.35 | 16.9% |
| walk (the one each build uses) | 3.78 | 47.2% |
| of which: segment assembly | 0.14 | 1.8% |
| RLE1 | 0.25 | 3.1% |
| CRC | 0.23 | 2.9% |
| **total** | 8.02 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.320 | (in its total) | 2.69× (2.67–2.70) |  |
| walk | 1.271 | 2.645 | (in its total) | 2.09× (2.08–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.054 | (in its total) | 1.12× (1.10–1.14) |  |
| inverse BWT, total | 1.812 | 4.019 | 3.279 | 2.22× (2.21–2.23) | 1.81× (1.81–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m41s |
| 4 threads | 35m17s ± 4m58s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m27s ± 3m28s |
| the scanner alone (its serial part) | 13m41s ± 32.2s |
| the workers alone, over 4 | 24m27s ± 3m28s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 3.21 ns a byte of output at 4 threads (CPU, summed over them), 2.03 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,457,111 | 12,465,313 | \`f5a13800cacccde1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2907.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
