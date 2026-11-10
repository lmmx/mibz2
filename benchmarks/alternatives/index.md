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

The latest record: [`dfe0405`](<https://github.com/lmmx/mibz2/commit/dfe04059c938c5aa3ed9458f766d49f4cb5c4e61>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42866572991>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.540 s | 388 | 8.1 | 2.92× (2.87–2.96) | 6.74× (6.62–6.86) | 6.95× (6.87–7.03) | 7.39× (7.31–7.47) | 6.75× (6.55–6.97) | 7.22× (7.11–7.32) | 6.61× (6.53–6.70) | 2.15× (2.12–2.18) | 2.45× (2.42–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.313 s | 671 | 8.9 | 2.81× (2.74–2.89) | 6.17× (5.93–6.42) | 7.13× (6.87–7.39) | 6.88× (6.69–7.09) | 6.22× (6.00–6.44) | 12.35× (12.01–12.69) | 11.32× (11.01–11.63) | 3.68× (3.56–3.80) | 4.19× (4.08–4.30) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.275 s | 763 | 15.2 | 2.70× (2.62–2.78) | 4.28× (4.16–4.40) | 5.18× (5.10–5.26) | 4.70× (4.59–4.81) | 4.95× (4.76–5.14) | 14.02× (13.69–14.35) | 12.85× (12.58–13.12) | 4.18× (4.07–4.29) | 4.75× (4.65–4.86) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 973 | 3.6 | 0.60× (0.59–0.61) | 3.70× (3.51–3.90) | 4.56× (4.37–4.76) | 3.55× (3.50–3.60) | 6.15× (6.07–6.24) | 2.95× (2.91–2.99) | 2.64× (2.60–2.68) | 1.04× (1.01–1.07) | 1.49× (1.47–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 959 | 3.7 | 0.57× (0.55–0.58) | 2.95× (2.89–3.00) | 6.08× (5.90–6.27) | 3.52× (3.45–3.60) | 5.99× (5.87–6.11) | 2.90× (2.85–2.96) | 2.60× (2.55–2.66) | 1.02× (0.98–1.07) | 1.47× (1.44–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 937 | 3.7 | 0.57× (0.56–0.58) | 2.93× (2.90–2.97) | 6.17× (6.01–6.32) | 3.48× (3.44–3.52) | 5.94× (5.82–6.07) | 2.88× (2.84–2.92) | 2.58× (2.55–2.61) | 1.02× (0.99–1.04) | 1.46× (1.44–1.47) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 560 | 6.0 | 2.62× (2.58–2.66) | 2.93× (2.90–2.95) | 3.73× (3.70–3.77) | 3.74× (3.71–3.78) | 4.06× (4.02–4.09) | 2.47× (2.45–2.49) | 2.06× (2.04–2.07) | 2.07× (2.05–2.10) | 2.91× (2.89–2.94) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 822 | 7.6 | 2.06× (1.94–2.20) | 2.55× (2.36–2.75) | 3.78× (3.55–4.03) | 3.07× (2.87–3.27) | 4.42× (4.16–4.69) | 3.50× (3.29–3.72) | 2.91× (2.74–3.10) | 2.94× (2.76–3.13) | 4.13× (3.87–4.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 757 | 14.3 | 1.85× (1.77–1.93) | 1.95× (1.83–2.07) | 3.06× (2.90–3.23) | 2.13× (2.04–2.22) | 3.83× (3.71–3.96) | 3.29× (3.19–3.39) | 2.74× (2.66–2.82) | 2.76× (2.69–2.83) | 3.88× (3.77–4.00) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 s | 60 | 53.4 | 1.35× (1.33–1.37) | 2.74× (2.67–2.81) | 2.46× (2.42–2.50) | 2.49× (2.46–2.52) | 2.91× (2.81–3.01) | 3.15× (3.13–3.17) | 3.11× (3.09–3.14) | 1.25× (1.24–1.27) | 1.41× (1.38–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 60.4 | 1.35× (1.24–1.48) | 2.54× (2.46–2.61) | 2.37× (2.32–2.42) | 3.05× (2.98–3.12) | 2.92× (2.87–2.97) | 5.07× (4.99–5.14) | 5.01× (4.92–5.10) | 2.02× (1.97–2.06) | 2.27× (2.20–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 81.1 | 1.34× (1.30–1.37) | 2.18× (2.13–2.22) | 2.13× (2.08–2.17) | 2.68× (2.61–2.76) | 2.99× (2.89–3.09) | 6.13× (6.00–6.25) | 6.06× (5.93–6.19) | 2.44× (2.38–2.50) | 2.75× (2.68–2.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 49.3 | 1.44× (1.43–1.45) | 3.03× (2.98–3.09) | 2.58× (2.53–2.64) | 2.63× (2.61–2.65) | 3.06× (3.00–3.12) | 3.62× (3.59–3.65) | 3.85× (3.83–3.88) | 1.45× (1.43–1.47) | 1.75× (1.72–1.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 55.4 | 1.29× (1.25–1.33) | 2.75× (2.63–2.87) | 2.48× (2.38–2.58) | 3.49× (3.31–3.68) | 3.03× (2.96–3.11) | 5.85× (5.63–6.07) | 6.23× (6.00–6.46) | 2.34× (2.25–2.44) | 2.83× (2.73–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 78.7 | 1.39× (1.37–1.42) | 2.34× (2.25–2.43) | 2.30× (2.22–2.38) | 3.07× (2.93–3.21) | 2.90× (2.84–2.97) | 6.88× (6.73–7.04) | 7.33× (7.15–7.51) | 2.76× (2.67–2.85) | 3.33× (3.23–3.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 727 | 5.0 | 0.66× (0.65–0.67) | 3.33× (3.07–3.62) | 4.40× (4.19–4.62) | 3.11× (3.05–3.16) | 6.77× (6.62–6.92) | 2.53× (2.48–2.57) | 2.34× (2.29–2.39) | 0.98× (0.96–1.00) | 1.50× (1.48–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 680 | 5.4 | 0.55× (0.54–0.56) | 2.79× (2.75–2.84) | 5.09× (4.81–5.39) | 2.92× (2.87–2.98) | 6.31× (6.20–6.42) | 2.36× (2.33–2.38) | 2.18× (2.16–2.21) | 0.92× (0.90–0.93) | 1.40× (1.38–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 673 | 5.3 | 0.56× (0.55–0.57) | 2.81× (2.77–2.85) | 5.03× (4.83–5.23) | 2.94× (2.89–2.98) | 6.32× (6.23–6.42) | 2.36× (2.33–2.40) | 2.19× (2.14–2.23) | 0.92× (0.90–0.94) | 1.41× (1.39–1.43) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.071 s | 180 | 17.9 | 1.98× (1.96–2.00) | 4.22× (4.09–4.35) | 4.52× (4.39–4.64) | 4.53× (4.44–4.62) | 4.75× (4.67–4.83) | 5.26× (5.19–5.34) | 4.90× (4.74–5.07) | 1.55× (1.52–1.59) | 1.87× (1.84–1.91) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.050 s | 258 | 22.8 | 1.50× (1.35–1.66) | 3.47× (3.15–3.82) | 3.75× (3.35–4.19) | 4.12× (3.60–4.71) | 3.97× (3.57–4.40) | 7.06× (6.30–7.91) | 6.57× (5.86–7.38) | 2.09× (1.85–2.35) | 2.51× (2.25–2.81) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.046 s | 277 | 34.1 | 1.45× (1.42–1.49) | 2.53× (2.34–2.73) | 3.19× (3.09–3.29) | 3.30× (3.19–3.42) | 3.67× (3.56–3.77) | 8.08× (7.91–8.25) | 7.52× (7.26–7.79) | 2.39× (2.31–2.46) | 2.87× (2.81–2.94) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.86–2.95) | 6.71× (6.59–6.83) | 6.92× (6.83–7.00) | 7.37× (7.29–7.44) | 6.73× (6.52–6.95) | 7.19× (7.09–7.30) | 6.59× (6.51–6.67) | 2.15× (2.11–2.18) | 2.43× (2.40–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.76× (2.74–2.79) | 6.32× (6.22–6.43) | 6.76× (6.66–6.85) | 7.21× (7.10–7.33) | 6.26× (6.13–6.38) | 6.57× (6.49–6.66) | 6.03× (5.98–6.08) | 1.96× (1.93–1.99) | 2.22× (2.20–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.61× (2.57–2.65) | 4.56× (4.46–4.66) | 4.84× (4.73–4.94) | 5.00× (4.84–5.17) | 4.66× (4.51–4.81) | 3.84× (3.75–3.93) | 3.52× (3.46–3.58) | 1.14× (1.11–1.18) | 1.30× (1.27–1.32) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.24× (3.19–3.29) | 4.06× (4.01–4.11) | 3.32× (3.28–3.36) | 5.69× (5.61–5.76) | 2.77× (2.74–2.81) | 2.49× (2.46–2.52) | 1.04× (1.01–1.06) | 1.05× (1.04–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.03× (2.97–3.09) | 5.34× (5.23–5.45) | 3.26× (3.19–3.34) | 5.46× (5.35–5.58) | 2.69× (2.63–2.74) | 2.41× (2.36–2.46) | 1.00× (0.97–1.04) | 1.02× (0.99–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.38× (3.32–3.44) | 5.45× (5.35–5.56) | 3.29× (3.25–3.33) | 5.51× (5.40–5.62) | 2.72× (2.69–2.75) | 2.44× (2.42–2.47) | 1.02× (0.99–1.04) | 1.03× (1.02–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.53× (2.49–2.57) | 2.71× (2.69–2.73) | 3.46× (3.43–3.50) | 3.59× (3.56–3.63) | 3.89× (3.86–3.92) | 2.39× (2.37–2.41) | 2.00× (1.99–2.01) | 2.02× (1.99–2.04) | 2.65× (2.63–2.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.11–2.18) | 2.34× (2.28–2.41) | 2.94× (2.88–2.99) | 2.96× (2.89–3.02) | 3.18× (3.11–3.25) | 1.87× (1.83–1.91) | 1.56× (1.53–1.60) | 1.58× (1.54–1.61) | 2.07× (2.02–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.92–2.04) | 1.73× (1.65–1.80) | 2.19× (2.12–2.28) | 2.04× (1.95–2.13) | 2.09× (2.02–2.15) | 1.00× (0.97–1.03) | 0.84× (0.81–0.86) | 0.84× (0.81–0.88) | 1.11× (1.08–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.33–1.37) | 2.67× (2.60–2.74) | 2.40× (2.35–2.44) | 2.47× (2.44–2.50) | 2.88× (2.78–2.98) | 3.12× (3.10–3.14) | 3.08× (3.06–3.11) | 1.25× (1.23–1.26) | 1.37× (1.35–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.28) | 2.49× (2.43–2.55) | 2.18× (2.15–2.22) | 2.89× (2.85–2.93) | 2.58× (2.53–2.62) | 2.74× (2.71–2.77) | 2.71× (2.68–2.74) | 1.10× (1.09–1.11) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.42–1.50) | 2.34× (2.28–2.40) | 1.91× (1.87–1.96) | 2.66× (2.58–2.74) | 2.60× (2.48–2.73) | 2.03× (1.99–2.08) | 2.01× (1.97–2.06) | 0.82× (0.80–0.84) | 0.90× (0.88–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.42–1.45) | 2.98× (2.93–3.02) | 2.53× (2.48–2.58) | 2.61× (2.59–2.63) | 3.03× (2.97–3.09) | 3.58× (3.55–3.61) | 3.81× (3.79–3.84) | 1.44× (1.42–1.47) | 1.70× (1.68–1.72) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.33–1.35) | 2.82× (2.77–2.87) | 2.37× (2.30–2.43) | 3.31× (3.19–3.44) | 2.79× (2.72–2.86) | 3.18× (3.15–3.22) | 3.39× (3.36–3.41) | 1.28× (1.27–1.30) | 1.51× (1.50–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.53–1.58) | 2.61× (2.52–2.69) | 2.00× (1.95–2.06) | 2.89× (2.78–3.02) | 2.61× (2.57–2.65) | 2.23× (2.19–2.28) | 2.38× (2.33–2.42) | 0.90× (0.88–0.92) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.74× (2.69–2.78) | 3.68× (3.53–3.83) | 2.84× (2.79–2.89) | 6.03× (5.90–6.16) | 2.33× (2.29–2.37) | 2.17× (2.13–2.21) | 0.99× (0.97–1.01) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.47× (2.45–2.49) | 4.28× (4.16–4.40) | 2.70× (2.65–2.75) | 5.56× (5.48–5.64) | 2.14× (2.12–2.16) | 1.99× (1.97–2.02) | 0.91× (0.89–0.92) | 0.91× (0.88–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.70–2.81) | 4.34× (4.26–4.42) | 2.79× (2.75–2.83) | 5.71× (5.64–5.79) | 2.20× (2.16–2.23) | 2.05× (2.01–2.09) | 0.93× (0.91–0.95) | 0.94× (0.91–0.98) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.95× (1.93–1.97) | 4.06× (3.93–4.20) | 4.37× (4.28–4.46) | 4.43× (4.34–4.52) | 4.65× (4.57–4.73) | 5.15× (5.08–5.22) | 4.79× (4.63–4.96) | 1.54× (1.51–1.57) | 1.79× (1.78–1.81) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.66× (1.65–1.68) | 3.48× (3.36–3.60) | 3.68× (3.59–3.77) | 4.15× (3.98–4.34) | 3.75× (3.69–3.81) | 4.07× (3.99–4.16) | 3.79× (3.64–3.95) | 1.22× (1.18–1.26) | 1.42× (1.40–1.44) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.71× (1.67–1.76) | 2.95× (2.80–3.12) | 3.02× (2.89–3.15) | 3.32× (3.22–3.44) | 3.19× (3.05–3.32) | 2.69× (2.61–2.77) | 2.50× (2.39–2.61) | 0.80× (0.78–0.83) | 0.94× (0.91–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.34) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.92× (11.89–11.94) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.57× (0.49–0.67) | 0.75× (0.74–0.76) | 0.63× (0.62–0.64) | 6.00× (5.96–6.03) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.37–0.38) | 0.72× (0.61–0.84) | 0.68× (0.67–0.69) | 0.60× (0.58–0.61) | 3.94× (3.92–3.95) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.81) | 4.28× (4.28–4.29) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.00× (7.98–8.01) | 0.64× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.74–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.65–0.68) | 0.68× (0.63–0.73) | 1.60× (1.56–1.64) | 0.80× (0.78–0.82) | 5.79× (5.64–5.94) | 0.46× (0.45–0.47) | 0.49× (0.47–0.50) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.69× (0.65–0.74) | 1.13× (1.06–1.20) | 0.69× (0.66–0.73) | 3.33× (3.16–3.51) | 0.26× (0.24–0.27) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.72–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.39× (0.35–0.43) | 0.91× (0.89–0.94) | 0.71× (0.68–0.75) | 3.16× (3.06–3.27) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.64× (0.62–0.67) | 0.45× (0.44–0.47) | 0.87× (0.85–0.90) | 0.80× (0.77–0.84) | 2.53× (2.46–2.61) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.21× (5.21–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.40× (0.39–0.42) | 0.88× (0.85–0.91) | 0.75× (0.72–0.78) | 3.00× (2.91–3.10) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.50× (0.47–0.53) | 0.79× (0.78–0.81) | 0.79× (0.77–0.81) | 2.28× (2.25–2.31) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.36× (5.35–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.12× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.30–5.33) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.91) | 0.12× (0.12–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.66–0.66) | 6.42× (6.41–6.43) | 0.39× (0.38–0.39) | 0.42× (0.41–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 34 MB | 0.49× (0.47–0.50) | 0.44× (0.40–0.48) | 1.13× (1.10–1.16) | 0.73× (0.70–0.75) | 3.58× (3.50–3.66) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 55 MB | 0.51× (0.50–0.52) | 0.49× (0.45–0.53) | 0.92× (0.90–0.94) | 0.75× (0.72–0.77) | 2.36× (2.32–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 388 | 2.58 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 671 | 2.83 | 1.09× (1.08–1.10) | 86% | 1.03× (1.01–1.05) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 763 | 4.84 | 1.87× (1.84–1.91) | 49% | 1.27× (1.24–1.31) | 1.68× (1.65–1.72) |

**Best:** mibz2 at 4 threads (763 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (280 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.14, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.81 | 0.02 | 0.09 | 0.275 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3148 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.4% |
| MTF + RLE2 | 1.48 | 16.5% |
| T-vector build | 1.53 | 17.1% |
| walk (the one each build uses) | 4.33 | 48.3% |
| of which: segment assembly | 0.18 | 2.0% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.25 | 2.8% |
| **total** | 8.96 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.301 | (in its total) | 2.66× (2.64–2.67) |  |
| walk | 1.308 | 2.696 | (in its total) | 2.07× (2.04–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.051 | 0.064 | (in its total) | 1.21× (1.12–1.30) |  |
| inverse BWT, total | 1.847 | 4.062 | 3.329 | 2.20× (2.17–2.23) | 1.81× (1.79–1.83) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h18m ± 8m55s |
| 4 threads | 40m26s ± 5m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m15s ± 4m08s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 29m15s ± 4m08s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 3.83 ns a byte of output at 4 threads (CPU, summed over them), 2.08 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,559,615 | 12,840,880 | \`c564b1e6883ea19b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3147.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
