# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-765/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`dd6450e`](<https://github.com/lmmx/mibz2/commit/dd6450e6ab7dc80064f86ce2a737eb4b998cedf1>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45021378079>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.492 s | 427 | 7.6 | 3.05× (3.00–3.10) | 6.97× (6.88–7.07) | 7.15× (7.04–7.26) | 7.91× (7.78–8.04) | 7.02× (6.90–7.14) | 7.65× (7.52–7.78) | 7.17× (7.03–7.30) | 2.27× (2.23–2.30) | 2.65× (2.60–2.70) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.271 s | 774 | 8.1 | 3.23× (2.98–3.50) | 6.74× (6.60–6.88) | 7.50× (7.34–7.66) | 7.65× (7.44–7.85) | 6.77× (6.60–6.94) | 13.78× (13.46–14.12) | 12.92× (12.62–13.22) | 4.09× (3.99–4.18) | 4.77× (4.66–4.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.223 s | 942 | 12.6 | 3.16× (3.09–3.22) | 4.81× (4.71–4.91) | 5.94× (5.73–6.17) | 5.45× (5.38–5.52) | 5.49× (5.41–5.58) | 16.88× (16.53–17.24) | 15.82× (15.55–16.10) | 5.01× (4.92–5.09) | 5.84× (5.75–5.94) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 990 | 3.6 | 0.60× (0.59–0.61) | 3.62× (3.45–3.80) | 4.47× (4.41–4.53) | 3.59× (3.54–3.63) | 6.21× (6.14–6.29) | 3.00× (2.96–3.04) | 2.68× (2.64–2.71) | 1.02× (1.00–1.04) | 1.52× (1.50–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 909 | 4.0 | 0.53× (0.49–0.57) | 2.76× (2.62–2.90) | 5.62× (5.37–5.88) | 3.29× (3.12–3.47) | 5.67× (5.39–5.97) | 2.72× (2.58–2.86) | 2.43× (2.31–2.55) | 0.92× (0.88–0.97) | 1.38× (1.31–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 907 | 4.0 | 0.54× (0.53–0.55) | 2.81× (2.76–2.86) | 5.57× (5.47–5.67) | 3.32× (3.27–3.38) | 5.79× (5.70–5.88) | 2.76× (2.71–2.81) | 2.46× (2.42–2.50) | 0.94× (0.91–0.96) | 1.40× (1.38–1.42) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 589 | 5.8 | 2.66× (2.63–2.68) | 3.06× (3.03–3.09) | 3.89× (3.83–3.96) | 3.97× (3.93–4.02) | 4.20× (4.16–4.24) | 2.57× (2.55–2.60) | 2.13× (2.11–2.15) | 2.03× (2.01–2.05) | 3.05× (3.02–3.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 933 | 7.0 | 2.31× (2.15–2.48) | 2.95× (2.80–3.11) | 4.19× (3.92–4.47) | 3.62× (3.32–3.94) | 4.93× (4.64–5.23) | 3.92× (3.67–4.18) | 3.25× (3.04–3.47) | 3.10× (2.90–3.31) | 4.64× (4.35–4.96) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 858 | 13.1 | 2.01× (1.95–2.08) | 2.17× (2.01–2.35) | 3.26× (3.16–3.36) | 2.37× (2.29–2.46) | 4.37× (4.22–4.52) | 3.76× (3.63–3.89) | 3.12× (3.02–3.22) | 2.97× (2.88–3.07) | 4.45× (4.30–4.61) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 52.9 | 1.35× (1.34–1.36) | 2.70× (2.67–2.73) | 2.44× (2.40–2.48) | 2.51× (2.49–2.53) | 2.88× (2.87–2.90) | 3.18× (3.16–3.20) | 3.23× (3.20–3.25) | 1.23× (1.22–1.23) | 1.42× (1.40–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.078 s | 102 | 59.2 | 1.27× (1.19–1.35) | 2.56× (2.43–2.70) | 2.40× (2.26–2.56) | 3.14× (2.92–3.39) | 2.96× (2.79–3.14) | 5.15× (4.84–5.47) | 5.22× (4.93–5.54) | 1.98× (1.87–2.11) | 2.29× (2.15–2.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.063 s | 127 | 79.9 | 1.42× (1.36–1.48) | 2.28× (2.22–2.34) | 2.15× (2.06–2.25) | 2.74× (2.65–2.83) | 3.07× (3.00–3.15) | 6.46× (6.31–6.62) | 6.56× (6.41–6.72) | 2.49× (2.43–2.55) | 2.88× (2.79–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 68 | 48.4 | 1.46× (1.45–1.47) | 3.09× (3.07–3.12) | 2.58× (2.55–2.61) | 2.71× (2.69–2.73) | 3.13× (3.11–3.14) | 3.69× (3.66–3.71) | 3.99× (3.96–4.02) | 1.43× (1.41–1.45) | 1.73× (1.70–1.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.084 s | 115 | 54.7 | 1.37× (1.33–1.42) | 2.90× (2.77–3.03) | 2.56× (2.49–2.64) | 3.61× (3.51–3.71) | 3.20× (3.12–3.28) | 6.17× (6.01–6.33) | 6.67× (6.51–6.84) | 2.40× (2.33–2.47) | 2.89× (2.80–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.072 s | 134 | 79.5 | 1.46× (1.42–1.51) | 2.37× (2.28–2.46) | 2.35× (2.27–2.43) | 3.16× (3.08–3.24) | 2.94× (2.86–3.03) | 7.25× (7.03–7.47) | 7.84× (7.63–8.05) | 2.82× (2.72–2.92) | 3.39× (3.28–3.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.0 | 0.67× (0.66–0.67) | 3.11× (2.95–3.27) | 4.45× (4.40–4.50) | 3.12× (3.09–3.15) | 6.82× (6.76–6.88) | 2.54× (2.52–2.56) | 2.34× (2.33–2.36) | 0.99× (0.97–1.01) | 1.53× (1.52–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 667 | 5.7 | 0.54× (0.52–0.56) | 2.78× (2.73–2.82) | 4.77× (4.69–4.85) | 2.88× (2.83–2.92) | 6.21× (6.12–6.30) | 2.31× (2.27–2.36) | 2.14× (2.10–2.17) | 0.90× (0.88–0.92) | 1.40× (1.37–1.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 662 | 5.6 | 0.54× (0.53–0.55) | 2.75× (2.73–2.78) | 4.72× (4.67–4.77) | 2.85× (2.82–2.88) | 6.17× (6.11–6.23) | 2.29× (2.27–2.31) | 2.11× (2.09–2.13) | 0.89× (0.88–0.91) | 1.38× (1.37–1.39) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 0.071 s | 196 | 16.9 | 2.05× (2.04–2.06) | 4.33× (4.30–4.35) | 4.63× (4.59–4.68) | 4.72× (4.69–4.75) | 4.88× (4.85–4.92) | 5.51× (5.46–5.57) | 5.25× (5.15–5.35) | 1.61× (1.58–1.63) | 2.01× (2.00–2.02) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 0.047 s | 297 | 20.8 | 1.72× (1.56–1.89) | 3.51× (3.33–3.69) | 4.35× (4.08–4.65) | 4.64× (4.37–4.93) | 4.41× (4.17–4.66) | 8.13× (7.68–8.61) | 7.74× (7.38–8.11) | 2.37× (2.23–2.52) | 2.97× (2.80–3.15) | yes |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 0.042 s | 332 | 31.8 | 1.66× (1.62–1.69) | 2.66× (2.55–2.77) | 3.71× (3.64–3.79) | 3.73× (3.65–3.81) | 4.09× (4.02–4.16) | 9.35× (9.10–9.60) | 8.90× (8.70–9.10) | 2.72× (2.65–2.80) | 3.41× (3.33–3.50) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.04× (2.99–3.09) | 6.94× (6.84–7.03) | 7.12× (7.01–7.23) | 7.88× (7.75–8.01) | 7.00× (6.88–7.12) | 7.62× (7.49–7.75) | 7.14× (7.01–7.27) | 2.26× (2.23–2.30) | 2.63× (2.59–2.68) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.09× (2.94–3.25) | 6.74× (6.69–6.79) | 7.05× (7.01–7.09) | 7.88× (7.82–7.93) | 6.68× (6.64–6.72) | 7.22× (7.18–7.26) | 6.77× (6.73–6.81) | 2.14× (2.13–2.16) | 2.50× (2.48–2.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.04× (2.98–3.09) | 5.10× (5.00–5.21) | 5.50× (5.38–5.62) | 5.84× (5.72–5.96) | 5.19× (5.10–5.28) | 4.65× (4.58–4.73) | 4.36× (4.29–4.43) | 1.38× (1.36–1.40) | 1.61× (1.58–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.64) | 3.28× (3.23–3.33) | 4.08× (4.03–4.14) | 3.36× (3.33–3.40) | 5.76× (5.70–5.83) | 2.83× (2.80–2.85) | 2.53× (2.51–2.56) | 1.02× (1.00–1.03) | 1.07× (1.05–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.54–0.59) | 2.79× (2.67–2.92) | 4.95× (4.70–5.21) | 3.01× (2.87–3.17) | 5.11× (4.88–5.36) | 2.49× (2.37–2.61) | 2.23× (2.13–2.33) | 0.90× (0.86–0.94) | 0.94× (0.89–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.15× (3.11–3.19) | 5.01× (4.93–5.08) | 3.08× (3.04–3.13) | 5.26× (5.19–5.33) | 2.55× (2.51–2.59) | 2.29× (2.25–2.32) | 0.92× (0.90–0.94) | 0.96× (0.95–0.98) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.57× (2.54–2.60) | 2.82× (2.79–2.84) | 3.52× (3.47–3.56) | 3.82× (3.77–3.86) | 4.03× (3.99–4.07) | 2.49× (2.46–2.52) | 2.07× (2.05–2.09) | 1.98× (1.96–2.00) | 2.73× (2.70–2.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.29–2.36) | 2.62× (2.58–2.66) | 3.20× (3.10–3.30) | 3.37× (3.32–3.42) | 3.53× (3.47–3.58) | 2.08× (2.06–2.11) | 1.73× (1.71–1.75) | 1.66× (1.64–1.68) | 2.28× (2.25–2.32) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.19× (2.09–2.29) | 1.91× (1.82–2.00) | 2.39× (2.28–2.51) | 2.25× (2.14–2.37) | 2.33× (2.23–2.43) | 1.12× (1.07–1.17) | 0.93× (0.89–0.97) | 0.89× (0.85–0.93) | 1.23× (1.17–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.35) | 2.65× (2.63–2.67) | 2.39× (2.35–2.43) | 2.49× (2.47–2.51) | 2.86× (2.84–2.87) | 3.15× (3.13–3.17) | 3.20× (3.17–3.23) | 1.22× (1.22–1.23) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.50× (2.46–2.54) | 2.22× (2.16–2.27) | 2.97× (2.93–3.01) | 2.60× (2.57–2.64) | 2.81× (2.77–2.85) | 2.85× (2.81–2.89) | 1.09× (1.07–1.11) | 1.23× (1.21–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.50× (1.48–1.53) | 2.35× (2.30–2.40) | 1.91× (1.88–1.94) | 2.63× (2.58–2.68) | 2.49× (2.45–2.53) | 2.08× (2.05–2.12) | 2.12× (2.08–2.15) | 0.81× (0.79–0.83) | 0.91× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.46) | 3.04× (3.01–3.07) | 2.53× (2.51–2.55) | 2.69× (2.67–2.71) | 3.10× (3.08–3.12) | 3.65× (3.63–3.68) | 3.95× (3.92–3.97) | 1.43× (1.41–1.45) | 1.68× (1.67–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.38) | 2.83× (2.80–2.85) | 2.36× (2.34–2.38) | 3.33× (3.28–3.37) | 2.82× (2.80–2.85) | 3.25× (3.21–3.28) | 3.51× (3.48–3.54) | 1.27× (1.25–1.29) | 1.50× (1.48–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.54–1.61) | 2.54× (2.49–2.59) | 1.93× (1.89–1.98) | 2.81× (2.75–2.89) | 2.52× (2.47–2.57) | 2.25× (2.21–2.29) | 2.43× (2.39–2.47) | 0.88× (0.86–0.90) | 1.04× (1.02–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.74× (2.70–2.78) | 3.60× (3.53–3.66) | 2.86× (2.83–2.89) | 6.10× (6.05–6.16) | 2.35× (2.33–2.38) | 2.18× (2.16–2.20) | 0.99× (0.97–1.01) | 0.98× (0.95–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.35× (2.32–2.38) | 4.00× (3.95–4.05) | 2.59× (2.56–2.63) | 5.35× (5.28–5.43) | 2.06× (2.02–2.10) | 1.91× (1.88–1.94) | 0.87× (0.85–0.88) | 0.85× (0.83–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.57× (2.54–2.60) | 3.97× (3.94–4.00) | 2.63× (2.62–2.65) | 5.44× (5.39–5.49) | 2.08× (2.06–2.10) | 1.93× (1.91–1.95) | 0.88× (0.86–0.89) | 0.86× (0.84–0.89) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 2.02× (2.01–2.03) | 4.13× (4.10–4.17) | 4.48× (4.46–4.50) | 4.62× (4.58–4.65) | 4.78× (4.74–4.82) | 5.39× (5.33–5.45) | 5.13× (5.03–5.23) | 1.59× (1.57–1.61) | 1.88× (1.86–1.90) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 1.75× (1.72–1.77) | 3.53× (3.48–3.59) | 3.84× (3.80–3.88) | 4.34× (4.29–4.39) | 3.98× (3.93–4.03) | 4.40× (4.34–4.45) | 4.18× (4.12–4.25) | 1.30× (1.27–1.33) | 1.53× (1.50–1.56) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 1.81× (1.76–1.86) | 2.80× (2.73–2.88) | 3.07× (2.98–3.15) | 3.42× (3.33–3.52) | 3.21× (3.14–3.28) | 2.87× (2.82–2.92) | 2.73× (2.66–2.81) | 0.85× (0.84–0.86) | 1.00× (0.99–1.02) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.30× (0.29–0.32) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.98× (10.97–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.51× (0.44–0.59) | 0.71× (0.70–0.73) | 0.60× (0.58–0.61) | 5.66× (5.53–5.80) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.35× (0.33–0.37) | 0.69× (0.62–0.77) | 0.63× (0.60–0.66) | 0.55× (0.53–0.57) | 3.66× (3.49–3.83) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.93× (0.93–0.93) | 4.86× (4.86–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.97× (7.93–8.02) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.74) | 0.80× (0.74–0.87) | 1.71× (1.67–1.76) | 0.86× (0.83–0.88) | 6.20× (6.04–6.37) | 0.50× (0.49–0.52) | 0.52× (0.51–0.54) | 0.57× (0.55–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.73) | 0.80× (0.74–0.87) | 1.25× (1.20–1.30) | 0.77× (0.74–0.80) | 3.66× (3.51–3.82) | 0.29× (0.27–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.15× (5.12–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.40× (0.38–0.43) | 0.89× (0.86–0.91) | 0.71× (0.69–0.74) | 3.06× (2.97–3.15) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.47× (0.46–0.48) | 0.88× (0.87–0.89) | 0.82× (0.80–0.85) | 2.56× (2.54–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.17× (5.15–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.38× (0.36–0.41) | 0.85× (0.83–0.87) | 0.72× (0.69–0.74) | 2.91× (2.84–2.99) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.50× (0.47–0.52) | 0.81× (0.80–0.82) | 0.80× (0.76–0.84) | 2.33× (2.29–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.06–1.07) | 0.76× (0.76–0.76) | 5.32× (5.29–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.19× (6.17–6.21) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.88× (1.87–1.88) | 0.90× (0.89–0.90) | 6.21× (6.18–6.24) | 0.40× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.07) | 0.15× (0.15–0.15) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.38× (6.37–6.39) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.45× (0.41–0.50) | 1.13× (1.11–1.15) | 0.73× (0.71–0.76) | 3.60× (3.52–3.68) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.73 MB → 13.8 MB, 5.1×) | 4 | 54 MB | 0.52× (0.50–0.54) | 0.52× (0.47–0.58) | 0.93× (0.91–0.95) | 0.76× (0.74–0.79) | 2.38× (2.34–2.42) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 427 | 2.35 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 774 | 2.49 | 1.05× (1.04–1.07) | 90% | 1.02× (1.02–1.03) | 1.07× (1.02–1.12) |
| 4: all 4 CPUs, sharing SMT siblings | 942 | 3.90 | 1.64× (1.61–1.67) | 55% | 1.21× (1.19–1.22) | 1.63× (1.62–1.65) |

**Best:** mibz2 at 4 threads (942 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (297 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.04, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.64 | 0.02 | 0.09 | 0.223 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3230 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m01s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.6% |
| MTF + RLE2 | 1.50 | 17.4% |
| T-vector build | 1.51 | 17.4% |
| walk (the one each build uses) | 4.17 | 48.2% |
| of which: segment assembly | 0.15 | 1.7% |
| RLE1 | 0.28 | 3.2% |
| CRC | 0.25 | 2.9% |
| **total** | 8.65 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.301 | (in its total) | 2.68× (2.67–2.69) |  |
| walk | 1.225 | 2.629 | (in its total) | 2.14× (2.13–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.056 | (in its total) | 1.23× (1.21–1.24) |  |
| inverse BWT, total | 1.756 | 3.986 | 3.282 | 2.27× (2.26–2.28) | 1.87× (1.85–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m43s |
| 4 threads | 32m34s ± 4m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m02s ± 3m17s |
| the scanner alone (its serial part) | 10m11s ± 0.00s |
| the workers alone, over 4 | 23m01s ± 3m17s |

At 4 threads the workers bound it. Per byte: the scanner 5.9 ns a byte of bz2, the workers 3.02 ns a byte of output at 4 threads (CPU, summed over them), 2.02 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,732,203 | 13,845,587 | \`5d6d3769149d431f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3230.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
