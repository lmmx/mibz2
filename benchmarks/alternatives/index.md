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

The latest record: [`29861b2`](<https://github.com/lmmx/mibz2/commit/29861b2bdc0a3beb9e3f9db89603b31be4e52463>) (2026-11-24).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45403274135>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.498 s | 421 | 6.9 | 3.05× (3.01–3.09) | 6.94× (6.87–7.01) | 7.18× (7.12–7.24) | 7.81× (7.74–7.87) | 7.03× (6.97–7.08) | 7.64× (7.57–7.71) | 7.01× (6.94–7.08) | 2.26× (2.23–2.29) | 2.63× (2.60–2.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.281 s | 745 | 7.4 | 3.12× (2.79–3.49) | 6.44× (5.88–7.06) | 7.32× (6.72–7.96) | 7.26× (6.72–7.84) | 6.58× (6.07–7.13) | 13.07× (12.19–14.02) | 11.99× (11.18–12.87) | 3.87× (3.61–4.16) | 4.50× (4.19–4.82) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.227 s | 925 | 11.5 | 3.20× (3.13–3.28) | 4.91× (4.82–5.00) | 6.15× (6.04–6.27) | 5.54× (5.44–5.64) | 5.60× (5.50–5.69) | 16.82× (16.68–16.97) | 15.44× (15.33–15.55) | 4.98× (4.94–5.03) | 5.79× (5.73–5.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 939 | 3.4 | 0.58× (0.57–0.58) | 3.58× (3.41–3.76) | 4.36× (4.20–4.53) | 3.46× (3.41–3.51) | 6.12× (6.02–6.21) | 2.85× (2.81–2.89) | 2.55× (2.51–2.59) | 1.00× (0.98–1.02) | 1.44× (1.42–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 897 | 3.7 | 0.53× (0.52–0.55) | 2.77× (2.71–2.83) | 5.92× (5.79–6.05) | 3.33× (3.26–3.40) | 5.74× (5.59–5.89) | 2.74× (2.68–2.80) | 2.45× (2.39–2.50) | 0.96× (0.94–0.98) | 1.38× (1.35–1.42) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 881 | 3.7 | 0.52× (0.51–0.54) | 2.71× (2.65–2.78) | 5.67× (5.53–5.81) | 3.24× (3.16–3.32) | 5.70× (5.61–5.79) | 2.66× (2.60–2.73) | 2.38× (2.33–2.44) | 0.93× (0.91–0.96) | 1.35× (1.31–1.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 568 | 5.4 | 2.76× (2.66–2.86) | 2.96× (2.93–2.99) | 3.85× (3.73–3.97) | 3.76× (3.72–3.80) | 4.19× (4.15–4.24) | 2.50× (2.47–2.53) | 2.07× (2.06–2.09) | 2.03× (1.96–2.10) | 2.94× (2.91–2.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 875 | 6.5 | 2.27× (2.19–2.34) | 2.93× (2.69–3.19) | 4.21× (4.01–4.42) | 3.41× (3.29–3.55) | 4.96× (4.80–5.13) | 3.85× (3.75–3.96) | 3.19× (3.10–3.29) | 3.13× (3.00–3.27) | 4.53× (4.40–4.67) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 829 | 13.0 | 2.07× (2.01–2.13) | 2.06× (1.92–2.21) | 3.25× (3.10–3.42) | 2.40× (2.28–2.52) | 4.40× (4.25–4.55) | 3.66× (3.57–3.75) | 3.04× (2.95–3.12) | 2.97× (2.87–3.08) | 4.31× (4.20–4.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 49.1 | 1.34× (1.32–1.35) | 2.68× (2.64–2.72) | 2.42× (2.37–2.48) | 2.44× (2.41–2.46) | 2.87× (2.84–2.90) | 3.11× (3.08–3.15) | 3.06× (3.03–3.09) | 1.24× (1.22–1.27) | 1.41× (1.39–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 101 | 53.9 | 1.32× (1.29–1.35) | 2.59× (2.54–2.64) | 2.44× (2.39–2.49) | 3.18× (3.08–3.28) | 3.05× (2.98–3.12) | 5.23× (5.13–5.34) | 5.15× (5.04–5.25) | 2.09× (2.05–2.14) | 2.37× (2.31–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.066 s | 121 | 73.8 | 1.37× (1.34–1.39) | 2.22× (2.16–2.28) | 2.24× (2.17–2.31) | 2.69× (2.63–2.76) | 3.01× (2.95–3.08) | 6.26× (6.12–6.41) | 6.16× (6.02–6.30) | 2.50× (2.44–2.57) | 2.83× (2.77–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 44.6 | 1.45× (1.44–1.46) | 3.02× (3.00–3.04) | 2.56× (2.53–2.60) | 2.62× (2.60–2.64) | 3.13× (3.08–3.17) | 3.65× (3.62–3.68) | 3.84× (3.82–3.86) | 1.45× (1.42–1.47) | 1.72× (1.71–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 111 | 50.0 | 1.38× (1.36–1.41) | 2.89× (2.85–2.93) | 2.59× (2.54–2.63) | 3.53× (3.44–3.62) | 3.20× (3.15–3.24) | 6.17× (6.05–6.29) | 6.50× (6.38–6.61) | 2.44× (2.38–2.52) | 2.92× (2.88–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 129 | 72.7 | 1.47× (1.43–1.50) | 2.37× (2.29–2.44) | 2.42× (2.34–2.50) | 3.12× (3.08–3.16) | 3.03× (2.99–3.08) | 7.20× (7.12–7.28) | 7.58× (7.47–7.69) | 2.85× (2.78–2.92) | 3.40× (3.34–3.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 4.7 | 0.64× (0.63–0.66) | 2.99× (2.83–3.15) | 4.27× (4.20–4.33) | 3.04× (2.99–3.08) | 6.84× (6.69–7.00) | 2.45× (2.41–2.49) | 2.24× (2.21–2.28) | 0.97× (0.95–0.99) | 1.47× (1.45–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 649 | 5.3 | 0.53× (0.52–0.54) | 2.68× (2.65–2.72) | 4.92× (4.71–5.13) | 2.81× (2.76–2.85) | 6.28× (6.15–6.42) | 2.25× (2.21–2.28) | 2.05× (2.02–2.09) | 0.89× (0.87–0.91) | 1.35× (1.33–1.37) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 645 | 5.2 | 0.52× (0.51–0.54) | 2.66× (2.60–2.71) | 4.72× (4.52–4.94) | 2.77× (2.71–2.83) | 6.16× (6.00–6.32) | 2.22× (2.18–2.26) | 2.03× (1.98–2.07) | 0.88× (0.87–0.89) | 1.33× (1.30–1.36) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.073 s | 192 | 15.5 | 2.02× (2.00–2.05) | 4.23× (4.17–4.29) | 4.60× (4.54–4.66) | 4.60× (4.56–4.64) | 4.87× (4.83–4.92) | 5.40× (5.33–5.46) | 4.89× (4.81–4.98) | 1.58× (1.55–1.61) | 1.97× (1.93–2.00) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.049 s | 287 | 19.4 | 1.67× (1.62–1.73) | 3.73× (3.51–3.96) | 4.49× (4.17–4.83) | 4.62× (4.42–4.83) | 4.42× (4.30–4.54) | 7.98× (7.76–8.21) | 7.23× (7.01–7.46) | 2.33× (2.27–2.40) | 2.91× (2.80–3.03) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.044 s | 319 | 29.4 | 1.59× (1.52–1.67) | 2.68× (2.59–2.77) | 3.81× (3.69–3.94) | 3.57× (3.47–3.69) | 4.03× (3.91–4.15) | 8.95× (8.69–9.21) | 8.11× (7.92–8.31) | 2.62× (2.53–2.71) | 3.26× (3.16–3.36) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.04× (3.00–3.08) | 6.91× (6.84–6.98) | 7.15× (7.09–7.20) | 7.78× (7.72–7.84) | 7.00× (6.95–7.06) | 7.61× (7.54–7.68) | 6.99× (6.92–7.05) | 2.26× (2.23–2.29) | 2.61× (2.58–2.64) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.06× (2.89–3.25) | 6.71× (6.52–6.91) | 7.03× (6.88–7.18) | 7.72× (7.55–7.89) | 6.71× (6.54–6.88) | 7.07× (6.91–7.23) | 6.49× (6.35–6.63) | 2.10× (2.05–2.15) | 2.42× (2.38–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.08× (3.03–3.13) | 5.20× (5.13–5.28) | 5.61× (5.48–5.74) | 5.87× (5.77–5.97) | 5.23× (5.14–5.32) | 4.59× (4.54–4.65) | 4.22× (4.17–4.27) | 1.36× (1.35–1.38) | 1.58× (1.56–1.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 3.15× (3.11–3.19) | 3.94× (3.88–4.00) | 3.24× (3.19–3.30) | 5.67× (5.58–5.76) | 2.69× (2.65–2.73) | 2.41× (2.37–2.45) | 1.00× (0.98–1.02) | 1.02× (0.99–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.58) | 2.80× (2.75–2.86) | 4.98× (4.89–5.07) | 3.04× (2.98–3.10) | 5.14× (5.02–5.27) | 2.49× (2.44–2.54) | 2.23× (2.19–2.28) | 0.93× (0.91–0.95) | 0.95× (0.92–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.56) | 3.02× (2.93–3.12) | 4.89× (4.78–5.00) | 3.00× (2.93–3.08) | 5.16× (5.07–5.25) | 2.46× (2.40–2.52) | 2.21× (2.15–2.26) | 0.92× (0.89–0.94) | 0.94× (0.91–0.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.66× (2.56–2.76) | 2.74× (2.71–2.76) | 3.52× (3.43–3.62) | 3.61× (3.57–3.65) | 4.01× (3.98–4.05) | 2.42× (2.39–2.45) | 2.02× (2.00–2.03) | 1.97× (1.91–2.04) | 2.66× (2.63–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.29× (2.24–2.34) | 2.57× (2.48–2.66) | 3.10× (3.04–3.16) | 3.15× (3.11–3.20) | 3.44× (3.37–3.51) | 1.99× (1.96–2.02) | 1.66× (1.64–1.68) | 1.62× (1.58–1.67) | 2.19× (2.16–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.02–2.18) | 1.72× (1.66–1.78) | 2.19× (2.11–2.28) | 2.02× (1.94–2.11) | 2.15× (2.09–2.21) | 1.02× (0.99–1.06) | 0.85× (0.82–0.88) | 0.83× (0.80–0.87) | 1.12× (1.09–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.35) | 2.60× (2.57–2.63) | 2.36× (2.32–2.41) | 2.41× (2.39–2.44) | 2.84× (2.82–2.87) | 3.08× (3.05–3.11) | 3.03× (3.00–3.06) | 1.24× (1.22–1.26) | 1.37× (1.35–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.49× (2.45–2.54) | 2.21× (2.17–2.25) | 2.93× (2.88–2.98) | 2.62× (2.56–2.69) | 2.78× (2.73–2.83) | 2.73× (2.69–2.78) | 1.12× (1.09–1.15) | 1.24× (1.21–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.43–1.51) | 2.33× (2.28–2.38) | 1.90× (1.85–1.96) | 2.61× (2.55–2.67) | 2.52× (2.47–2.57) | 2.04× (1.99–2.08) | 2.00× (1.96–2.05) | 0.82× (0.80–0.84) | 0.91× (0.89–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.46) | 2.97× (2.95–2.99) | 2.50× (2.47–2.53) | 2.59× (2.58–2.61) | 3.10× (3.05–3.14) | 3.61× (3.58–3.65) | 3.80× (3.78–3.82) | 1.44× (1.41–1.47) | 1.68× (1.67–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.35–1.37) | 2.80× (2.77–2.84) | 2.33× (2.30–2.37) | 3.22× (3.17–3.27) | 2.80× (2.76–2.83) | 3.21× (3.16–3.25) | 3.38× (3.33–3.42) | 1.28× (1.25–1.31) | 1.49× (1.48–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.53–1.57) | 2.49× (2.43–2.54) | 1.93× (1.90–1.96) | 2.77× (2.72–2.81) | 2.57× (2.51–2.63) | 2.21× (2.19–2.23) | 2.33× (2.30–2.35) | 0.88× (0.86–0.90) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.65× (2.60–2.69) | 3.51× (3.43–3.59) | 2.77× (2.73–2.81) | 6.10× (5.97–6.23) | 2.26× (2.23–2.29) | 2.08× (2.05–2.11) | 0.97× (0.95–0.99) | 0.94× (0.91–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.29× (2.26–2.32) | 4.00× (3.93–4.07) | 2.53× (2.49–2.57) | 5.40× (5.30–5.51) | 1.99× (1.97–2.02) | 1.83× (1.81–1.86) | 0.86× (0.84–0.87) | 0.83× (0.81–0.86) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.51× (2.47–2.55) | 4.00× (3.94–4.06) | 2.57× (2.51–2.62) | 5.43× (5.29–5.58) | 2.01× (1.97–2.06) | 1.85× (1.81–1.89) | 0.87× (0.85–0.88) | 0.84× (0.81–0.87) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 2.00× (1.98–2.02) | 4.08× (4.03–4.13) | 4.43× (4.38–4.48) | 4.50× (4.46–4.54) | 4.77× (4.72–4.81) | 5.28× (5.22–5.34) | 4.79× (4.70–4.87) | 1.56× (1.54–1.59) | 1.86× (1.84–1.89) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.74× (1.71–1.76) | 3.46× (3.38–3.54) | 3.78× (3.73–3.82) | 4.18× (4.14–4.23) | 3.90× (3.85–3.95) | 4.22× (4.15–4.28) | 3.82× (3.77–3.88) | 1.25× (1.22–1.27) | 1.49× (1.46–1.52) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.80× (1.75–1.84) | 2.87× (2.80–2.93) | 3.11× (3.01–3.22) | 3.35× (3.29–3.40) | 3.26× (3.17–3.34) | 2.80× (2.77–2.83) | 2.54× (2.52–2.56) | 0.83× (0.81–0.84) | 0.99× (0.97–1.01) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.30× (0.28–0.32) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.98–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.35× (0.34–0.37) | 0.60× (0.55–0.65) | 0.73× (0.70–0.76) | 0.61× (0.59–0.64) | 5.78× (5.56–6.02) | 0.14× (0.13–0.14) | 0.15× (0.14–0.16) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.37× (0.35–0.38) | 0.74× (0.64–0.84) | 0.67× (0.64–0.70) | 0.56× (0.53–0.59) | 3.83× (3.67–3.99) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.81) | 4.28× (4.27–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.97) | 0.92× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.97× (1.96–1.97) | 0.93× (0.92–0.93) | 4.86× (4.85–4.88) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.66) | 7.96× (7.93–7.99) | 0.65× (0.65–0.66) | 0.68× (0.67–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.71–0.74) | 0.76× (0.69–0.83) | 1.73× (1.69–1.77) | 0.86× (0.84–0.88) | 6.26× (6.14–6.38) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.71× (0.69–0.73) | 0.79× (0.77–0.82) | 1.27× (1.25–1.29) | 0.77× (0.75–0.79) | 3.75× (3.69–3.81) | 0.30× (0.29–0.30) | 0.31× (0.30–0.31) | 0.33× (0.33–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.13× (5.11–5.16) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.49–0.52) | 0.40× (0.37–0.44) | 0.90× (0.87–0.92) | 0.72× (0.70–0.74) | 3.10× (3.02–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.46–0.47) | 0.88× (0.87–0.89) | 0.82× (0.80–0.84) | 2.56× (2.53–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.17× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.38× (0.36–0.40) | 0.84× (0.81–0.86) | 0.69× (0.66–0.72) | 2.87× (2.79–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.49× (0.47–0.52) | 0.80× (0.78–0.81) | 0.79× (0.75–0.82) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.31× (5.30–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.88× (1.87–1.88) | 0.90× (0.90–0.90) | 6.21× (6.20–6.21) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.21× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.66–0.67) | 6.40× (6.39–6.41) | 0.38× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 35 MB | 0.48× (0.46–0.49) | 0.43× (0.39–0.47) | 1.11× (1.08–1.15) | 0.72× (0.69–0.75) | 3.53× (3.41–3.66) | 0.21× (0.20–0.22) | 0.23× (0.22–0.24) | 0.35× (0.33–0.36) | 0.17× (0.16–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 55 MB | 0.50× (0.49–0.51) | 0.50× (0.46–0.55) | 0.91× (0.88–0.94) | 0.75× (0.73–0.77) | 2.33× (2.27–2.38) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 421 | 2.39 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 745 | 2.55 | 1.08× (1.05–1.10) | 86% | 1.05× (1.03–1.07) | 1.09× (1.03–1.15) |
| 4: all 4 CPUs, sharing SMT siblings | 925 | 3.96 | 1.66× (1.64–1.68) | 55% | 1.25× (1.23–1.27) | 1.68× (1.66–1.70) |

**Best:** mibz2 at 4 threads (925 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (291 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.03, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.65 | 0.02 | 0.09 | 0.227 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2894 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.82 | 10.6% |
| MTF + RLE2 | 1.38 | 17.8% |
| T-vector build | 1.36 | 17.5% |
| walk (the one each build uses) | 3.71 | 47.9% |
| of which: segment assembly | 0.14 | 1.8% |
| RLE1 | 0.25 | 3.2% |
| CRC | 0.23 | 3.0% |
| **total** | 7.74 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.485 | 1.302 | (in its total) | 2.69× (2.68–2.70) |  |
| walk | 1.222 | 2.651 | (in its total) | 2.18× (2.16–2.20) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.056 | (in its total) | 1.19× (1.17–1.21) |  |
| inverse BWT, total | 1.757 | 4.009 | 3.238 | 2.29× (2.28–2.31) | 1.85× (1.84–1.86) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h12m ± 8m44s |
| 4 threads | 32m59s ± 4m38s |
| 4 threads, the sample's overlap gap as a fixed cost | 23m43s ± 3m21s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 23m43s ± 3m21s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 3.11 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2894.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
