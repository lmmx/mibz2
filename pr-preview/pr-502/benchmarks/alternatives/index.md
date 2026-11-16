# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-502/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`c15615e`](<https://github.com/lmmx/mibz2/commit/c15615eb9a972d7e8852a994122be857e8553910>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43906768125>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.522 s | 402 | 8.2 | 2.91× (2.90–2.93) | 6.67× (6.64–6.70) | 6.87× (6.85–6.89) | 7.45× (7.42–7.47) | 6.76× (6.74–6.78) | 7.35× (7.33–7.37) | 6.83× (6.81–6.86) | 2.14× (2.13–2.15) | 2.50× (2.48–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.312 s | 672 | 9.1 | 2.69× (2.56–2.83) | 5.78× (5.46–6.13) | 6.66× (6.28–7.07) | 6.44× (6.12–6.77) | 5.88× (5.59–6.19) | 11.98× (11.36–12.62) | 11.14× (10.58–11.72) | 3.49× (3.32–3.67) | 4.07× (3.87–4.28) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 859 | 13.9 | 2.88× (2.83–2.92) | 4.42× (4.32–4.51) | 5.48× (5.38–5.58) | 4.96× (4.88–5.04) | 5.08× (5.00–5.17) | 15.64× (15.41–15.88) | 14.54× (14.34–14.75) | 4.56× (4.50–4.62) | 5.32× (5.24–5.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 998 | 3.6 | 0.61× (0.60–0.61) | 3.80× (3.60–4.00) | 4.49× (4.45–4.54) | 3.63× (3.59–3.67) | 6.25× (6.18–6.32) | 3.02× (2.99–3.05) | 2.70× (2.68–2.73) | 1.01× (1.01–1.02) | 1.53× (1.51–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 980 | 3.8 | 0.58× (0.57–0.59) | 3.02× (2.97–3.06) | 5.95× (5.87–6.03) | 3.60× (3.55–3.65) | 6.09× (6.01–6.17) | 2.98× (2.94–3.02) | 2.66× (2.62–2.70) | 1.00× (0.98–1.02) | 1.51× (1.49–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 952 | 3.8 | 0.59× (0.56–0.63) | 3.01× (2.90–3.13) | 6.01× (5.82–6.21) | 3.51× (3.46–3.56) | 6.11× (5.93–6.28) | 2.91× (2.87–2.96) | 2.60× (2.56–2.65) | 0.98× (0.96–1.00) | 1.47× (1.45–1.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 589 | 5.9 | 2.66× (2.59–2.72) | 3.04× (2.97–3.12) | 3.92× (3.89–3.96) | 3.84× (3.80–3.87) | 4.19× (4.14–4.23) | 2.56× (2.49–2.63) | 2.12× (2.06–2.18) | 2.02× (1.98–2.06) | 3.03× (2.95–3.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 897 | 7.3 | 2.25× (2.14–2.36) | 2.92× (2.70–3.16) | 4.08× (3.89–4.29) | 3.65× (3.44–3.87) | 4.88× (4.66–5.10) | 3.89× (3.72–4.06) | 3.22× (3.08–3.36) | 3.06× (2.92–3.21) | 4.59× (4.39–4.80) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 786 | 14.8 | 1.86× (1.82–1.90) | 1.96× (1.83–2.10) | 3.15× (2.98–3.33) | 2.29× (2.17–2.42) | 4.01× (3.92–4.10) | 3.45× (3.39–3.52) | 2.86× (2.80–2.92) | 2.72× (2.67–2.77) | 4.08× (4.01–4.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 56.4 | 1.32× (1.30–1.34) | 2.64× (2.60–2.67) | 2.38× (2.34–2.42) | 2.39× (2.36–2.41) | 2.78× (2.74–2.82) | 3.04× (3.00–3.08) | 3.09× (3.05–3.13) | 1.19× (1.18–1.20) | 1.36× (1.34–1.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 63.6 | 1.23× (1.21–1.26) | 2.47× (2.42–2.53) | 2.34× (2.29–2.40) | 2.89× (2.85–2.93) | 2.88× (2.81–2.95) | 4.96× (4.86–5.07) | 5.04× (4.93–5.14) | 1.94× (1.90–1.98) | 2.23× (2.16–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 112 | 87.2 | 1.27× (1.23–1.30) | 2.04× (1.97–2.10) | 1.95× (1.86–2.03) | 2.36× (2.29–2.43) | 2.69× (2.62–2.77) | 5.73× (5.60–5.87) | 5.82× (5.68–5.96) | 2.24× (2.18–2.30) | 2.57× (2.49–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 51.0 | 1.46× (1.46–1.47) | 3.01× (2.98–3.03) | 2.52× (2.49–2.55) | 2.60× (2.58–2.63) | 3.07× (3.05–3.09) | 3.61× (3.59–3.62) | 3.88× (3.85–3.91) | 1.37× (1.37–1.38) | 1.66× (1.66–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 57.8 | 1.37× (1.33–1.40) | 2.75× (2.68–2.81) | 2.48× (2.41–2.54) | 3.31× (3.18–3.45) | 3.07× (3.02–3.13) | 5.94× (5.83–6.06) | 6.40× (6.28–6.52) | 2.27× (2.22–2.31) | 2.74× (2.69–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 83.5 | 1.39× (1.36–1.42) | 2.25× (2.21–2.30) | 2.16× (2.07–2.25) | 2.78× (2.73–2.82) | 2.74× (2.67–2.80) | 6.68× (6.59–6.78) | 7.19× (7.10–7.29) | 2.55× (2.51–2.59) | 3.09× (3.04–3.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 5.1 | 0.66× (0.65–0.68) | 3.09× (2.92–3.27) | 4.42× (4.34–4.51) | 3.11× (3.04–3.19) | 6.74× (6.58–6.91) | 2.53× (2.48–2.58) | 2.33× (2.28–2.38) | 0.97× (0.95–0.99) | 1.52× (1.49–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 704 | 5.5 | 0.56× (0.56–0.57) | 2.90× (2.87–2.93) | 4.99× (4.94–5.04) | 3.01× (2.98–3.05) | 6.53× (6.44–6.62) | 2.42× (2.40–2.45) | 2.23× (2.21–2.25) | 0.93× (0.92–0.94) | 1.46× (1.44–1.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 697 | 5.4 | 0.57× (0.56–0.58) | 2.88× (2.84–2.92) | 4.93× (4.87–5.00) | 2.98× (2.93–3.03) | 6.41× (6.33–6.48) | 2.40× (2.37–2.43) | 2.21× (2.18–2.24) | 0.92× (0.91–0.93) | 1.44× (1.43–1.46) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 0.073 s | 187 | 18.1 | 2.02× (1.95–2.09) | 4.17× (4.13–4.21) | 4.51× (4.42–4.59) | 4.53× (4.47–4.58) | 4.72× (4.66–4.77) | 5.29× (5.23–5.35) | 4.95× (4.90–5.00) | 1.50× (1.49–1.52) | 1.90× (1.85–1.95) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 0.051 s | 271 | 22.4 | 1.59× (1.53–1.65) | 3.32× (3.20–3.43) | 4.02× (3.90–4.14) | 4.22× (4.08–4.35) | 4.22× (4.08–4.37) | 7.66× (7.45–7.88) | 7.16× (6.96–7.37) | 2.17× (2.11–2.24) | 2.75× (2.63–2.87) | yes |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 0.046 s | 300 | 34.3 | 1.56× (1.49–1.63) | 2.41× (2.31–2.51) | 3.31× (3.17–3.45) | 3.25× (3.13–3.39) | 3.72× (3.60–3.85) | 8.41× (8.22–8.60) | 7.86× (7.67–8.06) | 2.39× (2.32–2.45) | 3.01× (2.91–3.13) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.89–2.92) | 6.65× (6.61–6.68) | 6.83× (6.82–6.85) | 7.42× (7.40–7.45) | 6.73× (6.71–6.75) | 7.32× (7.30–7.34) | 6.81× (6.78–6.83) | 2.14× (2.13–2.14) | 2.48× (2.47–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.74–2.80) | 6.24× (6.19–6.29) | 6.59× (6.48–6.70) | 7.09× (7.06–7.12) | 6.19× (6.17–6.21) | 6.67× (6.64–6.70) | 6.21× (6.17–6.24) | 1.95× (1.94–1.96) | 2.26× (2.24–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.81× (2.78–2.85) | 4.78× (4.72–4.84) | 5.15× (5.07–5.22) | 5.37× (5.30–5.44) | 4.86× (4.78–4.93) | 4.35× (4.29–4.41) | 4.05× (3.99–4.11) | 1.27× (1.25–1.29) | 1.48× (1.46–1.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.64–0.65) | 3.33× (3.28–3.37) | 4.12× (4.08–4.16) | 3.39× (3.36–3.43) | 5.78× (5.71–5.86) | 2.84× (2.82–2.87) | 2.55× (2.53–2.58) | 1.02× (1.01–1.02) | 1.08× (1.05–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.09× (3.05–3.14) | 5.41× (5.35–5.47) | 3.33× (3.29–3.38) | 5.55× (5.48–5.63) | 2.75× (2.71–2.79) | 2.47× (2.44–2.51) | 0.98× (0.97–1.00) | 1.04× (1.01–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.59–0.66) | 3.42× (3.30–3.55) | 5.43× (5.32–5.54) | 3.32× (3.28–3.37) | 5.65× (5.50–5.81) | 2.75× (2.70–2.79) | 2.47× (2.43–2.51) | 0.98× (0.96–1.00) | 1.04× (1.00–1.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.56× (2.50–2.62) | 2.81× (2.74–2.87) | 3.49× (3.41–3.58) | 3.65× (3.56–3.74) | 3.98× (3.89–4.08) | 2.47× (2.41–2.53) | 2.06× (2.00–2.11) | 1.96× (1.92–2.00) | 2.71× (2.64–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.23–2.30) | 2.56× (2.50–2.61) | 3.09× (3.04–3.14) | 3.32× (3.27–3.37) | 3.42× (3.36–3.49) | 2.03× (2.00–2.06) | 1.69× (1.67–1.71) | 1.61× (1.59–1.63) | 2.23× (2.19–2.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.88–2.06) | 1.70× (1.67–1.74) | 2.17× (2.12–2.23) | 2.07× (2.02–2.12) | 2.09× (2.04–2.13) | 1.01× (0.99–1.03) | 0.84× (0.83–0.86) | 0.80× (0.78–0.82) | 1.11× (1.09–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.30–1.34) | 2.57× (2.55–2.60) | 2.32× (2.29–2.36) | 2.37× (2.35–2.39) | 2.76× (2.72–2.79) | 3.01× (2.98–3.05) | 3.06× (3.02–3.10) | 1.19× (1.17–1.20) | 1.33× (1.31–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.24) | 2.40× (2.38–2.43) | 2.13× (2.11–2.15) | 2.71× (2.67–2.74) | 2.51× (2.48–2.54) | 2.68× (2.64–2.72) | 2.72× (2.69–2.75) | 1.06× (1.04–1.07) | 1.18× (1.17–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.42–1.48) | 2.20× (2.14–2.26) | 1.81× (1.77–1.84) | 2.41× (2.36–2.45) | 2.34× (2.30–2.39) | 1.96× (1.93–1.99) | 1.99× (1.96–2.03) | 0.77× (0.76–0.79) | 0.87× (0.85–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.45–1.46) | 2.96× (2.94–2.99) | 2.47× (2.46–2.48) | 2.58× (2.56–2.60) | 3.04× (3.02–3.06) | 3.57× (3.55–3.59) | 3.84× (3.81–3.87) | 1.37× (1.36–1.37) | 1.63× (1.62–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.34–1.37) | 2.73× (2.70–2.76) | 2.28× (2.25–2.30) | 3.05× (3.01–3.10) | 2.74× (2.70–2.78) | 3.14× (3.12–3.17) | 3.38× (3.36–3.41) | 1.21× (1.19–1.22) | 1.43× (1.42–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.49–1.62) | 2.50× (2.43–2.58) | 1.89× (1.82–1.98) | 2.62× (2.54–2.70) | 2.48× (2.40–2.57) | 2.19× (2.14–2.24) | 2.36× (2.30–2.42) | 0.84× (0.82–0.86) | 1.00× (0.97–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.69–0.72) | 2.71× (2.66–2.76) | 3.62× (3.49–3.75) | 2.85× (2.78–2.91) | 6.02× (5.88–6.15) | 2.34× (2.29–2.38) | 2.16× (2.12–2.20) | 0.97× (0.96–0.99) | 0.98× (0.95–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.54× (2.51–2.56) | 4.31× (4.27–4.35) | 2.77× (2.75–2.80) | 5.76× (5.68–5.83) | 2.20× (2.18–2.22) | 2.04× (2.02–2.06) | 0.92× (0.91–0.93) | 0.93× (0.90–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.62) | 2.77× (2.74–2.80) | 4.31× (4.26–4.37) | 2.81× (2.78–2.85) | 5.77× (5.72–5.83) | 2.23× (2.20–2.25) | 2.06× (2.04–2.08) | 0.93× (0.92–0.94) | 0.94× (0.91–0.97) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 1.99× (1.93–2.05) | 4.00× (3.95–4.05) | 4.34× (4.27–4.41) | 4.43× (4.38–4.48) | 4.62× (4.56–4.67) | 5.18× (5.12–5.24) | 4.84× (4.80–4.89) | 1.49× (1.47–1.50) | 1.78× (1.76–1.80) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 1.66× (1.63–1.69) | 3.37× (3.32–3.41) | 3.66× (3.60–3.72) | 4.00× (3.94–4.07) | 3.77× (3.71–3.84) | 4.15× (4.08–4.23) | 3.89× (3.83–3.94) | 1.19× (1.18–1.21) | 1.43× (1.41–1.45) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 1.76× (1.71–1.81) | 2.68× (2.64–2.73) | 2.94× (2.90–2.98) | 3.18× (3.12–3.24) | 3.10× (3.04–3.16) | 2.74× (2.70–2.78) | 2.57× (2.53–2.60) | 0.79× (0.78–0.80) | 0.94× (0.93–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.30× (0.29–0.32) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.06× (11.03–11.08) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.33× (0.32–0.35) | 0.52× (0.48–0.57) | 0.70× (0.67–0.72) | 0.58× (0.56–0.60) | 5.52× (5.33–5.72) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.34× (0.32–0.35) | 0.65× (0.53–0.81) | 0.60× (0.58–0.62) | 0.52× (0.50–0.54) | 3.49× (3.36–3.62) | 0.08× (0.08–0.08) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.04–8.06) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.74× (0.67–0.81) | 1.63× (1.59–1.67) | 0.81× (0.79–0.83) | 5.90× (5.76–6.04) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.55× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.67) | 0.72× (0.68–0.77) | 1.17× (1.15–1.19) | 0.72× (0.70–0.74) | 3.45× (3.40–3.50) | 0.27× (0.27–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.51) | 0.39× (0.36–0.43) | 0.92× (0.89–0.95) | 0.71× (0.68–0.75) | 3.16× (3.08–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.81× (0.79–0.84) | 2.51× (2.46–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.82–0.83) | 5.23× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.48–0.51) | 0.40× (0.39–0.41) | 0.86× (0.84–0.88) | 0.74× (0.72–0.76) | 2.97× (2.90–3.04) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.51× (0.49–0.53) | 0.81× (0.79–0.82) | 0.79× (0.76–0.83) | 2.32× (2.30–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.30–5.34) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.66–0.67) | 6.40× (6.37–6.42) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.42× (0.38–0.47) | 1.05× (1.02–1.08) | 0.68× (0.66–0.71) | 3.31× (3.23–3.40) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.7 MB → 13.7 MB, 5.1×) | 4 | 57 MB | 0.49× (0.47–0.51) | 0.45× (0.41–0.50) | 0.91× (0.88–0.94) | 0.74× (0.71–0.77) | 2.28× (2.20–2.37) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 402 | 2.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 672 | 2.74 | 1.10× (1.09–1.10) | 82% | 1.03× (1.02–1.04) | 1.04× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 859 | 4.22 | 1.68× (1.66–1.71) | 53% | 1.21× (1.20–1.22) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (859 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.69 | 0.02 | 0.09 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3304 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 12.5% |
| MTF + RLE2 | 1.61 | 17.5% |
| T-vector build | 1.55 | 16.9% |
| walk (the one each build uses) | 4.31 | 46.9% |
| of which: segment assembly | 0.16 | 1.7% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.8% |
| **total** | 9.19 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.305 | (in its total) | 2.68× (2.65–2.70) |  |
| walk | 1.250 | 2.656 | (in its total) | 2.12× (2.10–2.14) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.055 | (in its total) | 1.15× (1.13–1.17) |  |
| inverse BWT, total | 1.784 | 4.017 | 3.298 | 2.24× (2.22–2.26) | 1.84× (1.83–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m44s |
| 4 threads | 35m38s ± 5m01s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m44s ± 3m31s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 24m44s ± 3m31s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.24 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,698,958 | 13,676,554 | \`b30556180238088f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3303.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
