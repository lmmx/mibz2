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

The latest record: [`838890a`](<https://github.com/lmmx/mibz2/commit/838890abc53a2e88b65934b731450b28f5415c16>) (2026-11-07).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42376896350>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.697 s | 301 | 9.6 | 2.32× (2.29–2.36) | 5.52× (5.31–5.75) | 5.62× (5.51–5.74) | 6.41× (6.24–6.59) | 5.54× (5.45–5.63) | 6.31× (6.23–6.40) | 5.54× (5.46–5.63) | 1.71× (1.69–1.73) | 1.91× (1.89–1.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.396 s | 530 | 10.5 | 2.28× (2.05–2.53) | 4.77× (4.31–5.29) | 5.35× (4.81–5.95) | 5.71× (5.16–6.33) | 5.02× (4.52–5.57) | 10.52× (9.42–11.76) | 9.24× (8.31–10.27) | 2.86× (2.56–3.19) | 3.19× (2.87–3.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.339 s | 619 | 17.6 | 2.34× (2.27–2.42) | 3.72× (3.50–3.95) | 4.42× (4.28–4.56) | 4.35× (4.16–4.55) | 4.27× (4.09–4.44) | 12.97× (12.70–13.25) | 11.38× (11.19–11.59) | 3.52× (3.45–3.59) | 3.93× (3.85–4.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 893 | 3.5 | 0.59× (0.59–0.60) | 3.34× (3.19–3.50) | 4.47× (4.42–4.53) | 3.66× (3.62–3.70) | 6.17× (6.08–6.26) | 2.76× (2.73–2.79) | 2.74× (2.71–2.78) | 0.99× (0.98–1.01) | 1.88× (1.79–1.97) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 868 | 3.7 | 0.54× (0.53–0.55) | 2.68× (2.63–2.72) | 5.72× (5.57–5.87) | 3.56× (3.50–3.61) | 5.93× (5.72–6.14) | 2.67× (2.62–2.72) | 2.65× (2.61–2.70) | 0.96× (0.95–0.97) | 1.82× (1.74–1.90) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 862 | 3.6 | 0.54× (0.53–0.55) | 2.75× (2.60–2.91) | 5.64× (5.53–5.76) | 3.53× (3.45–3.61) | 5.93× (5.76–6.10) | 2.65× (2.60–2.71) | 2.64× (2.58–2.69) | 0.95× (0.93–0.98) | 1.80× (1.71–1.91) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 530 | 5.7 | 2.51× (2.48–2.55) | 2.76× (2.73–2.80) | 3.46× (3.39–3.53) | 3.79× (3.74–3.85) | 4.14× (4.07–4.22) | 3.17× (3.12–3.23) | 2.10× (2.08–2.12) | 1.94× (1.91–1.97) | 2.75× (2.71–2.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 822 | 7.0 | 2.19× (2.09–2.29) | 2.61× (2.53–2.68) | 4.06× (3.91–4.21) | 3.37× (3.29–3.45) | 4.73× (4.61–4.86) | 4.88× (4.76–4.99) | 3.22× (3.15–3.30) | 2.98× (2.92–3.05) | 4.22× (4.13–4.31) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 731 | 13.6 | 1.84× (1.79–1.89) | 1.61× (1.57–1.65) | 2.75× (2.65–2.84) | 2.13× (2.04–2.22) | 3.89× (3.75–4.03) | 4.38× (4.24–4.52) | 2.89× (2.81–2.98) | 2.68× (2.60–2.75) | 3.79× (3.68–3.89) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.143 s | 56 | 52.1 | 1.39× (1.38–1.39) | 2.62× (2.60–2.64) | 2.38× (2.31–2.44) | 2.43× (2.41–2.45) | 2.78× (2.69–2.87) | 3.17× (3.15–3.20) | 3.08× (3.07–3.10) | 1.27× (1.26–1.28) | 1.42× (1.41–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 90 | 59.0 | 1.27× (1.23–1.31) | 2.44× (2.34–2.54) | 2.29× (2.19–2.39) | 3.01× (2.91–3.11) | 2.87× (2.74–3.02) | 5.09× (4.93–5.27) | 4.94× (4.80–5.08) | 2.03× (1.97–2.09) | 2.28× (2.21–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 110 | 76.6 | 1.33× (1.31–1.35) | 2.17× (2.11–2.23) | 2.15× (2.07–2.24) | 2.66× (2.61–2.72) | 2.91× (2.84–2.99) | 6.29× (6.22–6.36) | 6.10× (6.04–6.17) | 2.51× (2.48–2.54) | 2.81× (2.77–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.152 s | 63 | 46.1 | 1.52× (1.46–1.58) | 3.02× (2.89–3.16) | 2.56× (2.45–2.68) | 2.74× (2.63–2.86) | 3.11× (3.00–3.22) | 3.96× (3.80–4.12) | 3.95× (3.80–4.11) | 1.55× (1.50–1.61) | 1.80× (1.74–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.093 s | 103 | 52.6 | 1.40× (1.37–1.42) | 2.78× (2.73–2.84) | 2.54× (2.49–2.59) | 3.65× (3.56–3.74) | 3.14× (3.08–3.20) | 6.64× (6.54–6.75) | 6.63× (6.54–6.73) | 2.61× (2.56–2.66) | 3.03× (2.97–3.09) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 74.0 | 1.39× (1.30–1.49) | 2.21× (2.05–2.39) | 2.21× (2.06–2.37) | 3.04× (2.82–3.28) | 2.87× (2.66–3.10) | 7.42× (6.92–7.96) | 7.41× (6.90–7.95) | 2.91× (2.72–3.12) | 3.38× (3.16–3.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 671 | 4.9 | 0.64× (0.64–0.65) | 2.83× (2.70–2.97) | 3.99× (3.96–4.03) | 3.17× (3.13–3.20) | 6.77× (6.61–6.94) | 2.41× (2.39–2.43) | 2.38× (2.36–2.40) | 0.96× (0.95–0.97) | 2.07× (2.06–2.09) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 633 | 5.2 | 0.52× (0.51–0.53) | 2.60× (2.54–2.65) | 4.97× (4.80–5.15) | 3.03× (2.90–3.17) | 6.34× (6.17–6.52) | 2.25× (2.21–2.30) | 2.23× (2.18–2.27) | 0.90× (0.88–0.92) | 1.94× (1.90–1.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 623 | 5.2 | 0.53× (0.52–0.54) | 2.59× (2.54–2.64) | 4.93× (4.76–5.12) | 2.97× (2.91–3.03) | 6.32× (6.20–6.45) | 2.24× (2.19–2.29) | 2.21× (2.16–2.26) | 0.89× (0.87–0.92) | 1.92× (1.89–1.96) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 0.092 s | 138 | 21.3 | 1.41× (1.39–1.44) | 2.97× (2.92–3.03) | 3.30× (3.26–3.35) | 3.50× (3.47–3.53) | 3.67× (3.58–3.75) | 3.84× (3.79–3.89) | 3.53× (3.49–3.58) | 1.15× (1.12–1.18) | 1.32× (1.30–1.34) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 0.058 s | 219 | 24.9 | 1.20× (1.14–1.27) | 2.79× (2.49–3.13) | 3.20× (2.99–3.44) | 3.44× (3.26–3.63) | 3.52× (3.32–3.73) | 5.91× (5.56–6.29) | 5.44× (5.15–5.75) | 1.77× (1.68–1.87) | 2.03× (1.91–2.15) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 0.050 s | 255 | 36.1 | 1.31× (1.26–1.36) | 2.21× (2.03–2.41) | 2.96× (2.70–3.23) | 3.12× (2.96–3.29) | 3.53× (3.32–3.74) | 7.13× (6.95–7.31) | 6.56× (6.37–6.75) | 2.14× (2.06–2.22) | 2.45× (2.40–2.49) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.28–2.36) | 5.50× (5.29–5.73) | 5.60× (5.49–5.71) | 6.40× (6.23–6.57) | 5.53× (5.44–5.61) | 6.30× (6.21–6.39) | 5.53× (5.44–5.61) | 1.71× (1.69–1.73) | 1.90× (1.88–1.92) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.18–2.31) | 5.03× (4.88–5.19) | 5.20× (5.05–5.35) | 6.06× (5.93–6.20) | 5.11× (4.96–5.27) | 5.66× (5.48–5.85) | 4.97× (4.83–5.12) | 1.54× (1.49–1.59) | 1.71× (1.66–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.19× (2.14–2.24) | 3.86× (3.69–4.03) | 4.08× (4.00–4.17) | 4.50× (4.37–4.63) | 3.95× (3.83–4.09) | 3.46× (3.37–3.55) | 3.03× (2.96–3.11) | 0.94× (0.92–0.96) | 1.04× (1.02–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.01× (2.98–3.05) | 4.03× (3.99–4.07) | 3.43× (3.39–3.46) | 5.72× (5.64–5.80) | 2.61× (2.58–2.64) | 2.59× (2.56–2.62) | 0.99× (0.98–1.01) | 1.69× (1.66–1.71) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.77× (2.72–2.81) | 5.01× (4.83–5.19) | 3.32× (3.27–3.37) | 5.44× (5.26–5.62) | 2.49× (2.45–2.53) | 2.47× (2.43–2.52) | 0.95× (0.93–0.96) | 1.61× (1.58–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.59) | 3.06× (2.98–3.14) | 5.03× (4.90–5.16) | 3.36× (3.28–3.43) | 5.54× (5.39–5.69) | 2.53× (2.48–2.58) | 2.51× (2.46–2.57) | 0.96× (0.94–0.99) | 1.63× (1.59–1.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.43× (2.40–2.46) | 2.64× (2.60–2.67) | 3.14× (3.11–3.18) | 3.64× (3.59–3.69) | 3.98× (3.90–4.05) | 3.06× (3.00–3.11) | 2.04× (2.02–2.06) | 1.89× (1.86–1.92) | 2.45× (2.42–2.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.17× (2.14–2.20) | 2.40× (2.33–2.46) | 3.15× (3.08–3.22) | 3.15× (3.09–3.21) | 3.34× (3.28–3.40) | 2.52× (2.47–2.58) | 1.68× (1.65–1.72) | 1.56× (1.53–1.59) | 2.02× (1.98–2.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.97–2.04) | 1.62× (1.60–1.64) | 2.12× (2.09–2.16) | 2.00× (1.96–2.04) | 2.00× (1.97–2.04) | 1.29× (1.27–1.32) | 0.86× (0.85–0.88) | 0.80× (0.79–0.81) | 1.03× (1.01–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.39) | 2.56× (2.54–2.57) | 2.32× (2.25–2.39) | 2.41× (2.39–2.43) | 2.76× (2.67–2.84) | 3.14× (3.12–3.17) | 3.05× (3.04–3.06) | 1.26× (1.25–1.27) | 1.37× (1.37–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.24–1.30) | 2.39× (2.33–2.45) | 2.10× (2.03–2.16) | 2.85× (2.79–2.92) | 2.54× (2.39–2.69) | 2.77× (2.70–2.83) | 2.69× (2.63–2.74) | 1.11× (1.09–1.14) | 1.21× (1.18–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.44–1.50) | 2.37× (2.30–2.44) | 2.00× (1.93–2.06) | 2.64× (2.57–2.70) | 2.53× (2.45–2.60) | 2.12× (2.07–2.17) | 2.06× (2.02–2.10) | 0.85× (0.83–0.87) | 0.93× (0.90–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.45–1.57) | 2.97× (2.84–3.11) | 2.51× (2.41–2.62) | 2.72× (2.61–2.84) | 3.08× (2.97–3.19) | 3.92× (3.76–4.08) | 3.91× (3.76–4.06) | 1.55× (1.50–1.60) | 1.76× (1.69–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.38–1.42) | 2.75× (2.71–2.80) | 2.33× (2.31–2.36) | 3.35× (3.30–3.41) | 2.80× (2.76–2.85) | 3.49× (3.42–3.55) | 3.48× (3.42–3.54) | 1.38× (1.35–1.40) | 1.56× (1.53–1.59) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.49–1.66) | 2.50× (2.36–2.65) | 1.99× (1.91–2.09) | 2.87× (2.68–3.08) | 2.64× (2.47–2.81) | 2.43× (2.30–2.56) | 2.42× (2.29–2.56) | 0.96× (0.91–1.01) | 1.09× (1.03–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.69) | 2.52× (2.49–2.55) | 3.50× (3.48–3.52) | 2.89× (2.87–2.92) | 6.05× (5.91–6.20) | 2.23× (2.22–2.25) | 2.21× (2.20–2.23) | 0.97× (0.96–0.97) | 1.39× (1.38–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.29× (2.26–2.33) | 4.07× (4.01–4.13) | 2.81× (2.69–2.93) | 5.63× (5.47–5.80) | 2.06× (2.03–2.09) | 2.04× (2.01–2.08) | 0.89× (0.88–0.91) | 1.28× (1.26–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.53× (2.48–2.58) | 4.09× (4.00–4.19) | 2.83× (2.77–2.88) | 5.76× (5.65–5.88) | 2.11× (2.06–2.15) | 2.08× (2.04–2.12) | 0.91× (0.89–0.93) | 1.31× (1.28–1.34) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 1.40× (1.38–1.43) | 2.88× (2.84–2.93) | 3.20× (3.17–3.24) | 3.44× (3.41–3.47) | 3.60× (3.52–3.69) | 3.77× (3.72–3.82) | 3.48× (3.43–3.52) | 1.15× (1.12–1.18) | 1.22× (1.21–1.23) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 1.27× (1.25–1.28) | 2.74× (2.58–2.91) | 3.00× (2.87–3.14) | 3.35× (3.31–3.40) | 3.14× (3.09–3.18) | 3.23× (3.18–3.29) | 2.98× (2.94–3.01) | 0.98× (0.97–1.00) | 1.04× (1.03–1.06) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 1.44× (1.38–1.50) | 2.47× (2.27–2.70) | 2.77× (2.50–3.06) | 2.93× (2.79–3.08) | 2.88× (2.69–3.09) | 2.25× (2.22–2.28) | 2.07× (2.04–2.11) | 0.68× (0.67–0.70) | 0.73× (0.72–0.74) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.33× (0.31–0.34) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.19× (12.18–12.21) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.51× (0.46–0.57) | 0.80× (0.79–0.82) | 0.66× (0.65–0.67) | 6.31× (6.20–6.43) | 0.15× (0.14–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.56× (0.49–0.64) | 0.71× (0.69–0.72) | 0.60× (0.59–0.62) | 4.09× (4.07–4.11) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.16× (0.16–0.16) | 0.82× (0.82–0.83) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.83–0.84) | 4.42× (4.42–4.42) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.73–1.74) | 0.66× (0.66–0.67) | 8.14× (8.12–8.16) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.67) | 0.70× (0.65–0.75) | 1.65× (1.64–1.66) | 0.81× (0.80–0.82) | 5.96× (5.92–6.01) | 0.47× (0.47–0.47) | 0.50× (0.49–0.50) | 0.54× (0.54–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.65) | 0.71× (0.68–0.74) | 1.17× (1.14–1.19) | 0.75× (0.73–0.78) | 3.42× (3.34–3.51) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.38–0.44) | 0.92× (0.90–0.95) | 0.70× (0.66–0.74) | 3.17× (3.08–3.28) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.86× (0.84–0.89) | 0.80× (0.78–0.83) | 2.51× (2.43–2.58) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.37× (0.34–0.40) | 0.84× (0.82–0.87) | 0.69× (0.66–0.72) | 2.89× (2.80–2.98) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.49× (0.46–0.53) | 0.80× (0.80–0.81) | 0.82× (0.80–0.84) | 2.32× (2.30–2.34) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.45× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.78) | 5.41× (5.39–5.43) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.42× (5.41–5.43) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.41× (0.40–0.41) | 1.50× (1.50–1.51) | 0.66× (0.66–0.67) | 6.76× (6.75–6.77) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.66–0.66) | 0.33× (0.33–0.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 31 MB | 0.51× (0.51–0.52) | 0.47× (0.43–0.51) | 1.26× (1.23–1.29) | 0.79× (0.77–0.80) | 3.94× (3.89–4.00) | 0.23× (0.23–0.24) | 0.25× (0.25–0.25) | 0.39× (0.38–0.39) | 0.19× (0.19–0.20) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 49 MB | 0.55× (0.54–0.57) | 0.55× (0.51–0.58) | 1.04× (1.00–1.07) | 0.82× (0.79–0.86) | 2.64× (2.56–2.72) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 301 | 3.33 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 530 | 3.65 | 1.11× (1.08–1.14) | 83% | 1.02× (0.99–1.04) | 1.07× (1.06–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 619 | 6.15 | 1.82× (1.77–1.88) | 51% | 1.28× (1.23–1.32) | 1.72× (1.69–1.75) |

**Best:** mibz2 at 4 threads (619 MB/s); lbzip2 at 4 threads (170 MB/s); pbz2 at 4 threads (265 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.70, mibz2 4t 1.05, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 6%, mibz2 4t 6%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.07 | 0.02 | 0.07 | 0.339 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2870 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.2% |
| MTF + RLE2 | 1.59 | 17.4% |
| T-vector build | 1.78 | 19.5% |
| walk (the one each build uses) | 4.18 | 45.8% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.26 | 2.8% |
| **total** | 9.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.585 | 1.369 | (in its total) | 2.34× (2.33–2.36) |  |
| walk | 1.354 | 2.681 | (in its total) | 1.98× (1.97–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.056 | 0.057 | (in its total) | 0.99× (0.95–1.03) |  |
| inverse BWT, total | 1.994 | 4.109 | 2.480 | 2.06× (2.05–2.07) | 1.24× (1.23–1.26) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h41m ± 11m51s |
| 4 threads | 49m17s ± 6m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 38m25s ± 5m34s |
| the scanner alone (its serial part) | 16m44s ± 32.9s |
| the workers alone, over 4 | 38m25s ± 5m34s |

At 4 threads the workers bound it. Per byte: the scanner 9.7 ns a byte of bz2, the workers 5.04 ns a byte of output at 4 threads (CPU, summed over them), 2.77 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,065,205 | 12,657,539 | \`96a905131eb63a3a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2870.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
