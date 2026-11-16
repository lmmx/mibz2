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

The latest record: [`1a37d1c`](<https://github.com/lmmx/mibz2/commit/1a37d1c77fa90a80e54dfa7482f8876334686cbd>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43786899170>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.419 s | 501 | 5.2 | 2.98× (2.89–3.08) | 7.01× (6.70–7.33) | 7.21× (7.00–7.42) | 8.19× (7.93–8.45) | 7.15× (6.96–7.33) | 7.38× (7.22–7.54) | 7.09× (6.89–7.30) | 2.18× (2.13–2.23) | 2.44× (2.39–2.49) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.244 s | 861 | 5.8 | 2.94× (2.68–3.21) | 5.97× (5.43–6.56) | 6.82× (6.29–7.40) | 7.05× (6.44–7.72) | 6.27× (5.71–6.89) | 12.07× (11.01–13.24) | 11.61× (10.64–12.66) | 3.57× (3.26–3.90) | 3.99× (3.65–4.38) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.201 s | 1,042 | 8.9 | 3.10× (2.98–3.23) | 4.71× (4.44–5.00) | 5.89× (5.46–6.35) | 5.58× (5.32–5.86) | 5.68× (5.27–6.12) | 15.55× (15.26–15.83) | 14.94× (14.68–15.21) | 4.59× (4.52–4.66) | 5.14× (5.06–5.22) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,102 | 2.6 | 0.54× (0.48–0.60) | 3.20× (2.88–3.56) | 4.27× (3.84–4.75) | 3.35× (3.01–3.73) | 5.77× (5.18–6.43) | 2.49× (2.24–2.77) | 2.49× (2.24–2.76) | 0.93× (0.83–1.03) | 1.91× (1.60–2.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,072 | 2.7 | 0.53× (0.51–0.55) | 2.76× (2.70–2.82) | 5.52× (5.40–5.64) | 3.46× (3.39–3.53) | 5.87× (5.71–6.03) | 2.56× (2.51–2.61) | 2.56× (2.51–2.61) | 0.95× (0.93–0.97) | 1.96× (1.75–2.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,055 | 2.7 | 0.51× (0.50–0.53) | 2.73× (2.64–2.82) | 5.44× (5.25–5.62) | 3.43× (3.31–3.54) | 5.89× (5.59–6.22) | 2.52× (2.44–2.61) | 2.52× (2.44–2.61) | 0.94× (0.90–0.98) | 1.93× (1.74–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.027 s | 731 | 3.8 | 2.68× (2.66–2.71) | 3.04× (3.01–3.07) | 3.98× (3.76–4.21) | 4.07× (4.04–4.10) | 4.43× (4.39–4.46) | 2.75× (2.73–2.77) | 2.24× (2.22–2.26) | 2.11× (2.05–2.18) | 3.03× (3.00–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,057 | 4.8 | 2.12× (1.84–2.44) | 2.74× (2.45–3.07) | 4.24× (3.68–4.88) | 3.37× (2.91–3.89) | 4.72× (4.12–5.41) | 3.80× (3.31–4.35) | 3.09× (2.70–3.54) | 2.91× (2.53–3.35) | 4.17× (3.65–4.78) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 931 | 9.7 | 1.88× (1.82–1.95) | 1.89× (1.68–2.12) | 2.92× (2.77–3.09) | 2.14× (1.98–2.32) | 3.91× (3.70–4.12) | 3.46× (3.29–3.63) | 2.82× (2.69–2.96) | 2.66× (2.50–2.82) | 3.80× (3.62–3.99) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 36.7 | 1.38× (1.34–1.43) | 2.64× (2.57–2.72) | 2.44× (2.36–2.53) | 2.47× (2.40–2.54) | 2.81× (2.72–2.91) | 2.99× (2.88–3.11) | 3.09× (3.00–3.18) | 1.26× (1.21–1.31) | 1.41× (1.37–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 117 | 41.6 | 1.31× (1.27–1.35) | 2.70× (2.46–2.97) | 2.42× (2.35–2.49) | 3.04× (2.97–3.10) | 2.92× (2.84–3.00) | 4.94× (4.77–5.11) | 5.10× (4.98–5.21) | 2.07× (2.00–2.15) | 2.32× (2.25–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 141 | 55.5 | 1.35× (1.31–1.38) | 2.19× (2.12–2.26) | 2.27× (2.18–2.36) | 2.66× (2.56–2.76) | 2.99× (2.90–3.09) | 5.99× (5.81–6.18) | 6.18× (6.07–6.30) | 2.52× (2.44–2.60) | 2.81× (2.74–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.121 s | 79 | 33.2 | 1.53× (1.52–1.55) | 3.06× (3.03–3.09) | 2.67× (2.62–2.72) | 2.78× (2.74–2.81) | 3.17× (3.06–3.29) | 3.71× (3.64–3.79) | 4.01× (3.98–4.03) | 1.54× (1.52–1.57) | 1.81× (1.79–1.83) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.078 s | 123 | 38.8 | 1.32× (1.29–1.35) | 2.74× (2.60–2.88) | 2.44× (2.36–2.51) | 3.28× (3.18–3.37) | 2.96× (2.88–3.04) | 5.77× (5.64–5.90) | 6.23× (6.06–6.40) | 2.40× (2.32–2.49) | 2.81× (2.74–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.066 s | 146 | 54.3 | 1.38× (1.33–1.43) | 2.34× (2.14–2.56) | 2.30× (2.17–2.45) | 2.91× (2.81–3.02) | 2.86× (2.74–3.00) | 6.86× (6.67–7.06) | 7.41× (7.20–7.62) | 2.85× (2.74–2.98) | 3.35× (3.25–3.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 826 | 3.6 | 0.62× (0.61–0.63) | 3.39× (3.33–3.45) | 4.25× (4.17–4.34) | 3.05× (3.00–3.11) | 6.67× (6.48–6.86) | 2.24× (2.20–2.28) | 2.29× (2.24–2.33) | 0.95× (0.92–0.98) | 1.71× (1.68–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 797 | 3.8 | 0.52× (0.50–0.55) | 2.45× (2.38–2.53) | 4.91× (4.76–5.06) | 2.95× (2.87–3.04) | 6.37× (6.28–6.47) | 2.15× (2.09–2.22) | 2.19× (2.13–2.26) | 0.91× (0.88–0.95) | 1.64× (1.59–1.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 790 | 3.7 | 0.51× (0.50–0.53) | 2.45× (2.39–2.51) | 4.89× (4.76–5.03) | 2.94× (2.86–3.02) | 6.43× (6.09–6.79) | 2.14× (2.08–2.20) | 2.18× (2.13–2.24) | 0.91× (0.87–0.94) | 1.63× (1.59–1.67) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.060 s | 221 | 12.1 | 1.82× (1.80–1.84) | 4.09× (3.93–4.27) | 4.34× (4.28–4.41) | 4.54× (4.47–4.62) | 4.71× (4.64–4.78) | 4.68× (4.57–4.80) | 4.75× (4.59–4.91) | 1.52× (1.47–1.58) | 1.71× (1.67–1.74) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.042 s | 318 | 15.0 | 1.58× (1.38–1.81) | 3.26× (3.06–3.47) | 3.94× (3.67–4.23) | 4.20× (3.99–4.43) | 4.30× (4.10–4.51) | 6.74× (6.40–7.11) | 6.84× (6.45–7.25) | 2.19× (2.07–2.32) | 2.46× (2.34–2.58) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.038 s | 353 | 22.7 | 1.48× (1.43–1.54) | 2.66× (2.33–3.03) | 3.38× (3.06–3.72) | 3.48× (3.18–3.81) | 3.94× (3.70–4.19) | 7.44× (7.17–7.72) | 7.54× (7.24–7.86) | 2.42× (2.32–2.52) | 2.71× (2.62–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.98× (2.88–3.07) | 6.97× (6.67–7.29) | 7.17× (6.97–7.39) | 8.16× (7.91–8.42) | 7.12× (6.93–7.30) | 7.35× (7.19–7.52) | 7.07× (6.87–7.27) | 2.17× (2.13–2.22) | 2.42× (2.38–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.91× (2.79–3.04) | 6.42× (6.29–6.55) | 6.68× (6.54–6.82) | 7.65× (7.52–7.78) | 6.57× (6.37–6.78) | 6.70× (6.57–6.84) | 6.44× (6.31–6.57) | 1.98× (1.95–2.02) | 2.21× (2.17–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.94–3.09) | 5.11× (4.85–5.39) | 5.61× (5.20–6.06) | 6.04× (5.80–6.28) | 5.47× (5.07–5.91) | 4.34× (4.28–4.39) | 4.17× (4.08–4.26) | 1.28× (1.27–1.30) | 1.43× (1.41–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.57× (0.51–0.64) | 2.74× (2.46–3.06) | 3.67× (3.28–4.10) | 3.13× (2.80–3.49) | 5.33× (4.77–5.95) | 2.35× (2.11–2.61) | 2.35× (2.11–2.61) | 0.92× (0.83–1.03) | 1.54× (1.38–1.72) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.54–0.64) | 2.67× (2.62–2.73) | 4.85× (4.77–4.92) | 3.22× (3.17–3.28) | 5.38× (5.24–5.53) | 2.39× (2.34–2.43) | 2.39× (2.34–2.43) | 0.94× (0.92–0.96) | 1.57× (1.53–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 2.93× (2.84–3.02) | 4.91× (4.77–5.05) | 3.26× (3.16–3.36) | 5.49× (5.21–5.78) | 2.40× (2.33–2.48) | 2.40× (2.33–2.48) | 0.95× (0.91–0.98) | 1.58× (1.53–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.58× (2.56–2.60) | 2.82× (2.80–2.84) | 3.56× (3.35–3.79) | 3.88× (3.86–3.91) | 4.22× (4.19–4.25) | 2.65× (2.62–2.67) | 2.17× (2.15–2.19) | 2.05× (1.99–2.10) | 2.65× (2.62–2.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (1.84–2.49) | 2.33× (2.01–2.70) | 3.14× (2.72–3.63) | 3.08× (2.67–3.57) | 3.34× (2.88–3.88) | 1.99× (1.72–2.31) | 1.63× (1.41–1.89) | 1.54× (1.32–1.79) | 1.99× (1.72–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.90–2.04) | 1.66× (1.63–1.68) | 2.14× (2.10–2.19) | 1.99× (1.94–2.04) | 2.03× (1.99–2.07) | 1.04× (1.02–1.06) | 0.85× (0.84–0.87) | 0.80× (0.79–0.82) | 1.04× (1.02–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.33–1.42) | 2.56× (2.49–2.64) | 2.39× (2.31–2.47) | 2.45× (2.38–2.52) | 2.79× (2.69–2.88) | 2.96× (2.85–3.08) | 3.05× (2.97–3.14) | 1.25× (1.21–1.30) | 1.35× (1.32–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.30) | 2.49× (2.37–2.62) | 2.17× (2.15–2.19) | 2.81× (2.79–2.83) | 2.54× (2.49–2.58) | 2.65× (2.59–2.71) | 2.73× (2.72–2.74) | 1.12× (1.10–1.15) | 1.21× (1.21–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.43–1.49) | 2.30× (2.24–2.37) | 1.96× (1.90–2.03) | 2.58× (2.50–2.66) | 2.61× (2.48–2.75) | 1.99× (1.94–2.05) | 2.05× (2.02–2.08) | 0.84× (0.82–0.87) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.54) | 3.00× (2.98–3.02) | 2.60× (2.56–2.63) | 2.75× (2.72–2.79) | 3.14× (3.03–3.25) | 3.67× (3.60–3.74) | 3.96× (3.94–3.99) | 1.54× (1.51–1.57) | 1.77× (1.75–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.31–1.39) | 2.70× (2.61–2.78) | 2.32× (2.24–2.39) | 3.12× (3.01–3.23) | 2.69× (2.61–2.78) | 3.12× (2.99–3.25) | 3.36× (3.26–3.47) | 1.30× (1.26–1.35) | 1.50× (1.46–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.54–1.62) | 2.66× (2.45–2.88) | 2.09× (1.99–2.19) | 2.75× (2.66–2.85) | 2.67× (2.56–2.78) | 2.27× (2.21–2.33) | 2.45× (2.39–2.51) | 0.95× (0.92–0.98) | 1.09× (1.07–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.44× (2.39–2.48) | 3.40× (3.33–3.48) | 2.79× (2.75–2.84) | 5.96× (5.80–6.12) | 2.09× (2.05–2.12) | 2.13× (2.09–2.16) | 0.96× (0.93–0.98) | 1.35× (1.33–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.56–0.61) | 2.23× (2.17–2.29) | 4.04× (3.91–4.17) | 2.70× (2.62–2.78) | 5.58× (5.50–5.66) | 1.95× (1.89–2.01) | 1.99× (1.93–2.05) | 0.90× (0.86–0.93) | 1.26× (1.22–1.30) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.51× (2.46–2.56) | 4.10× (4.00–4.20) | 2.78× (2.71–2.85) | 5.84× (5.52–6.18) | 2.02× (1.96–2.07) | 2.05× (2.01–2.10) | 0.93× (0.90–0.96) | 1.30× (1.27–1.33) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.80× (1.78–1.82) | 3.92× (3.75–4.09) | 4.17× (4.13–4.22) | 4.44× (4.37–4.52) | 4.60× (4.53–4.67) | 4.58× (4.47–4.69) | 4.64× (4.49–4.80) | 1.51× (1.46–1.56) | 1.61× (1.59–1.64) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.53× (1.48–1.58) | 3.21× (3.14–3.27) | 3.64× (3.44–3.85) | 4.04× (3.96–4.12) | 3.78× (3.71–3.85) | 3.65× (3.56–3.74) | 3.70× (3.58–3.82) | 1.20× (1.17–1.24) | 1.29× (1.26–1.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.63× (1.57–1.68) | 2.82× (2.59–3.07) | 3.23× (2.94–3.54) | 3.50× (3.22–3.79) | 3.29× (3.06–3.54) | 2.44× (2.35–2.53) | 2.47× (2.37–2.58) | 0.80× (0.77–0.84) | 0.86× (0.83–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.31–0.35) | 0.98× (0.97–0.98) | 0.79× (0.78–0.79) | 11.03× (11.01–11.04) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.33× (0.31–0.34) | 0.49× (0.40–0.58) | 0.67× (0.64–0.70) | 0.56× (0.53–0.59) | 5.35× (5.10–5.62) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.50× (0.42–0.58) | 0.59× (0.57–0.62) | 0.52× (0.49–0.55) | 3.47× (3.33–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.36–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.70) | 0.66× (0.66–0.66) | 8.02× (8.01–8.04) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.70) | 0.68× (0.62–0.74) | 1.61× (1.55–1.67) | 0.81× (0.78–0.83) | 5.85× (5.65–6.05) | 0.47× (0.46–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.61–0.69) | 0.70× (0.66–0.75) | 1.16× (1.12–1.20) | 0.72× (0.68–0.76) | 3.39× (3.26–3.52) | 0.26× (0.25–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.33–0.33) | 0.35× (0.34–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.42× (0.41–0.44) | 0.90× (0.87–0.93) | 0.73× (0.70–0.75) | 3.13× (3.03–3.23) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.66× (0.64–0.67) | 0.46× (0.45–0.47) | 0.87× (0.85–0.90) | 0.83× (0.80–0.85) | 2.53× (2.47–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.23× (5.22–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.39× (0.38–0.40) | 0.85× (0.82–0.88) | 0.72× (0.69–0.75) | 2.90× (2.81–3.00) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.62) | 0.49× (0.45–0.53) | 0.81× (0.80–0.82) | 0.83× (0.81–0.85) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.36× (5.35–5.37) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.25–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.31× (5.30–5.32) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.32× (5.30–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.41) | 0.67× (0.67–0.68) | 6.41× (6.39–6.43) | 0.39× (0.39–0.40) | 0.42× (0.42–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.39× (0.36–0.42) | 1.05× (1.03–1.08) | 0.68× (0.66–0.70) | 3.33× (3.25–3.41) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.52) | 0.44× (0.40–0.49) | 0.91× (0.88–0.93) | 0.74× (0.72–0.77) | 2.31× (2.25–2.37) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 501 | 2.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 861 | 2.22 | 1.10× (1.07–1.12) | 82% | 1.01× (0.97–1.05) | 1.07× (1.04–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 1,042 | 3.43 | 1.70× (1.66–1.73) | 53% | 1.24× (1.17–1.32) | 1.72× (1.66–1.78) |

**Best:** mibz2 at 4 threads (1,042 MB/s); lbzip2 at 4 threads (226 MB/s); pbz2 at 4 threads (338 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.20, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.54 | 0.01 | 0.08 | 0.201 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.8% |
| MTF + RLE2 | 1.11 | 18.2% |
| T-vector build | 1.04 | 17.0% |
| walk (the one each build uses) | 2.87 | 47.0% |
| of which: segment assembly | 0.11 | 1.8% |
| RLE1 | 0.13 | 2.2% |
| CRC | 0.18 | 2.9% |
| **total** | 6.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.398 | 1.064 | (in its total) | 2.68× (2.67–2.69) |  |
| walk | 1.060 | 2.063 | (in its total) | 1.95× (1.94–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.040 | 0.040 | (in its total) | 0.98× (0.96–1.01) |  |
| inverse BWT, total | 1.499 | 3.168 | 1.894 | 2.12× (2.11–2.13) | 1.26× (1.26–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m08s |
| 4 threads | 29m13s ± 4m07s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m52s ± 2m50s |
| the scanner alone (its serial part) | 11m03s ± 32.2s |
| the workers alone, over 4 | 19m52s ± 2m50s |

At 4 threads the workers bound it. Per byte: the scanner 6.4 ns a byte of bz2, the workers 2.60 ns a byte of output at 4 threads (CPU, summed over them), 1.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,079 | 13,357,122 | \`cc146a11acfb9941…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
