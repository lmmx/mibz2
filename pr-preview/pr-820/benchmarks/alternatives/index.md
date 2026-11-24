# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-820/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`a284cdc`](<https://github.com/lmmx/mibz2/commit/a284cdc84e1d7cafe4dbf642b85d969ca781a526>) (2026-11-24).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45364914539>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.390 s | 538 | 6.9 | 3.13× (3.11–3.15) | 7.17× (7.15–7.20) | 7.44× (7.39–7.48) | 8.44× (8.40–8.48) | 7.43× (7.40–7.47) | 7.77× (7.73–7.80) | 7.34× (7.31–7.38) | 2.31× (2.30–2.32) | 2.59× (2.57–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.231 s | 910 | 7.4 | 3.09× (2.93–3.26) | 6.31× (6.00–6.65) | 7.31× (6.93–7.71) | 7.61× (7.21–8.04) | 6.73× (6.35–7.12) | 13.09× (12.39–13.83) | 12.38× (11.70–13.09) | 3.89× (3.68–4.11) | 4.36× (4.12–4.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,155 | 11.6 | 3.20× (3.13–3.28) | 4.80× (4.61–4.98) | 5.90× (5.71–6.09) | 5.79× (5.66–5.93) | 5.70× (5.59–5.81) | 16.67× (16.38–16.97) | 15.76× (15.51–16.02) | 4.96× (4.87–5.04) | 5.55× (5.45–5.65) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,078 | 3.8 | 0.55× (0.55–0.56) | 3.31× (3.27–3.35) | 4.42× (4.37–4.48) | 3.48× (3.44–3.52) | 6.15× (6.04–6.26) | 2.57× (2.54–2.61) | 2.57× (2.54–2.61) | 0.94× (0.93–0.96) | 2.02× (1.82–2.25) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 995 | 4.2 | 0.48× (0.47–0.49) | 2.58× (2.53–2.63) | 5.16× (5.06–5.26) | 3.25× (3.19–3.32) | 5.54× (5.41–5.69) | 2.39× (2.35–2.44) | 2.39× (2.35–2.44) | 0.88× (0.86–0.90) | 1.88× (1.71–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 1,018 | 4.0 | 0.49× (0.48–0.51) | 2.61× (2.56–2.67) | 5.22× (5.10–5.33) | 3.28× (3.21–3.36) | 5.71× (5.58–5.85) | 2.42× (2.37–2.47) | 2.42× (2.37–2.47) | 0.89× (0.87–0.91) | 1.90× (1.73–2.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 742 | 5.3 | 2.72× (2.70–2.74) | 3.08× (3.06–3.11) | 3.86× (3.82–3.89) | 4.11× (4.08–4.14) | 4.61× (4.54–4.68) | 2.78× (2.76–2.80) | 2.27× (2.26–2.29) | 2.09× (2.08–2.11) | 3.07× (3.05–3.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,195 | 6.2 | 2.35× (2.22–2.49) | 3.04× (2.86–3.23) | 4.52× (4.17–4.90) | 3.80× (3.59–4.02) | 5.51× (5.22–5.83) | 4.35× (4.09–4.63) | 3.56× (3.35–3.78) | 3.28× (3.08–3.48) | 4.81× (4.52–5.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,098 | 12.2 | 2.08× (2.00–2.17) | 2.05× (1.86–2.25) | 3.44× (3.20–3.69) | 2.47× (2.30–2.65) | 4.70× (4.41–5.00) | 4.05× (3.91–4.20) | 3.31× (3.19–3.43) | 3.05× (2.94–3.16) | 4.47× (4.31–4.64) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 75 | 49.8 | 1.44× (1.42–1.45) | 2.77× (2.75–2.79) | 2.50× (2.48–2.52) | 2.59× (2.58–2.60) | 2.98× (2.95–3.02) | 3.11× (3.10–3.13) | 3.10× (3.09–3.11) | 1.30× (1.29–1.31) | 1.44× (1.42–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.064 s | 124 | 56.2 | 1.35× (1.25–1.45) | 2.55× (2.44–2.68) | 2.50× (2.40–2.60) | 3.20× (3.07–3.34) | 3.05× (2.94–3.18) | 5.08× (4.90–5.27) | 5.06× (4.86–5.26) | 2.12× (2.04–2.20) | 2.35× (2.24–2.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.053 s | 150 | 74.3 | 1.44× (1.36–1.51) | 2.28× (2.20–2.36) | 2.23× (2.16–2.30) | 2.79× (2.66–2.92) | 3.13× (3.04–3.23) | 6.25× (6.11–6.40) | 6.23× (6.09–6.37) | 2.61× (2.54–2.67) | 2.89× (2.81–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.115 s | 83 | 45.0 | 1.56× (1.56–1.57) | 3.18× (3.17–3.19) | 2.76× (2.72–2.79) | 2.86× (2.85–2.87) | 3.26× (3.24–3.28) | 3.81× (3.79–3.83) | 4.01× (3.99–4.03) | 1.55× (1.54–1.56) | 1.85× (1.84–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.069 s | 140 | 50.6 | 1.48× (1.35–1.63) | 2.91× (2.76–3.07) | 2.67× (2.53–2.82) | 3.68× (3.50–3.86) | 3.25× (3.11–3.40) | 6.26× (6.00–6.53) | 6.59× (6.32–6.87) | 2.55× (2.45–2.65) | 3.03× (2.91–3.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 165 | 72.8 | 1.52× (1.48–1.55) | 2.43× (2.36–2.51) | 2.41× (2.33–2.48) | 3.25× (3.16–3.35) | 3.16× (3.10–3.22) | 7.51× (7.36–7.67) | 7.91× (7.75–8.07) | 3.06× (2.99–3.12) | 3.64× (3.56–3.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 830 | 5.1 | 0.62× (0.61–0.62) | 3.40× (3.38–3.43) | 4.27× (4.24–4.31) | 3.05× (3.04–3.07) | 6.78× (6.64–6.92) | 2.25× (2.23–2.27) | 2.28× (2.26–2.30) | 0.94× (0.94–0.95) | 1.71× (1.69–1.73) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 744 | 5.9 | 0.48× (0.47–0.50) | 2.31× (2.26–2.36) | 4.64× (4.54–4.74) | 2.78× (2.73–2.84) | 6.08× (5.89–6.27) | 2.02× (1.98–2.07) | 2.05× (2.01–2.10) | 0.85× (0.83–0.87) | 1.54× (1.51–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 744 | 5.8 | 0.49× (0.48–0.50) | 2.32× (2.28–2.36) | 4.64× (4.56–4.72) | 2.80× (2.74–2.86) | 6.18× (5.99–6.37) | 2.03× (1.99–2.06) | 2.05× (2.02–2.09) | 0.85× (0.83–0.87) | 1.54× (1.51–1.57) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.058 s | 242 | 15.7 | 1.87× (1.78–1.95) | 4.09× (3.92–4.28) | 4.50× (4.29–4.71) | 4.73× (4.52–4.95) | 4.94× (4.73–5.17) | 4.87× (4.66–5.09) | 4.76× (4.55–4.98) | 1.55× (1.48–1.62) | 1.74× (1.65–1.83) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.038 s | 362 | 19.8 | 1.56× (1.45–1.67) | 3.50× (3.20–3.84) | 4.24× (4.01–4.48) | 4.76× (4.51–5.02) | 4.53× (4.34–4.73) | 7.33× (7.00–7.67) | 7.16× (6.84–7.49) | 2.33× (2.23–2.44) | 2.62× (2.47–2.77) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.035 s | 400 | 30.4 | 1.59× (1.54–1.65) | 2.64× (2.50–2.79) | 3.49× (3.36–3.62) | 3.71× (3.64–3.78) | 4.14× (4.01–4.27) | 8.20× (8.04–8.36) | 8.01× (7.85–8.17) | 2.60× (2.56–2.65) | 2.93× (2.86–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.12× (3.10–3.14) | 7.14× (7.11–7.16) | 7.40× (7.37–7.43) | 8.42× (8.38–8.46) | 7.41× (7.38–7.44) | 7.74× (7.71–7.77) | 7.32× (7.30–7.34) | 2.30× (2.30–2.31) | 2.57× (2.56–2.58) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.92× (2.81–3.03) | 6.60× (6.36–6.86) | 6.92× (6.65–7.21) | 8.10× (7.79–8.42) | 6.80× (6.54–7.08) | 7.04× (6.77–7.31) | 6.65× (6.39–6.92) | 2.09× (2.01–2.18) | 2.33× (2.24–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.07× (3.01–3.13) | 5.19× (5.06–5.32) | 5.57× (5.48–5.67) | 6.28× (6.17–6.39) | 5.43× (5.32–5.54) | 4.64× (4.56–4.72) | 4.39× (4.31–4.47) | 1.38× (1.36–1.41) | 1.54× (1.51–1.57) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.60) | 2.83× (2.80–2.87) | 3.77× (3.73–3.81) | 3.26× (3.23–3.29) | 5.70× (5.60–5.79) | 2.43× (2.41–2.46) | 2.43× (2.41–2.46) | 0.95× (0.94–0.96) | 1.60× (1.57–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.52× (0.51–0.53) | 2.46× (2.41–2.50) | 4.46× (4.40–4.52) | 2.98× (2.93–3.04) | 5.00× (4.88–5.12) | 2.20× (2.16–2.24) | 2.20× (2.16–2.24) | 0.86× (0.84–0.87) | 1.44× (1.42–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.51–0.53) | 2.75× (2.69–2.82) | 4.57× (4.46–4.67) | 3.06× (2.99–3.13) | 5.22× (5.09–5.35) | 2.26× (2.21–2.31) | 2.26× (2.21–2.31) | 0.88× (0.86–0.90) | 1.48× (1.45–1.51) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.60–2.63) | 2.84× (2.82–2.87) | 3.37× (3.35–3.39) | 3.92× (3.89–3.94) | 4.39× (4.32–4.46) | 2.67× (2.65–2.69) | 2.19× (2.18–2.21) | 2.02× (2.01–2.04) | 2.65× (2.62–2.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.39× (2.34–2.44) | 2.59× (2.55–2.63) | 3.51× (3.44–3.58) | 3.52× (3.44–3.60) | 3.87× (3.78–3.97) | 2.27× (2.22–2.33) | 1.87× (1.82–1.91) | 1.72× (1.69–1.76) | 2.25× (2.21–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.05–2.21) | 1.81× (1.72–1.92) | 2.38× (2.31–2.45) | 2.19× (2.11–2.29) | 2.31× (2.23–2.41) | 1.16× (1.13–1.20) | 0.96× (0.93–0.99) | 0.88× (0.85–0.91) | 1.15× (1.12–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.43× (1.41–1.44) | 2.67× (2.65–2.68) | 2.45× (2.44–2.45) | 2.56× (2.55–2.58) | 2.95× (2.91–2.99) | 3.08× (3.06–3.10) | 3.07× (3.05–3.08) | 1.29× (1.29–1.30) | 1.41× (1.41–1.42) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.33× (1.27–1.40) | 2.49× (2.44–2.54) | 2.22× (2.17–2.26) | 3.00× (2.94–3.06) | 2.64× (2.59–2.69) | 2.72× (2.68–2.77) | 2.71× (2.66–2.77) | 1.15× (1.12–1.17) | 1.25× (1.23–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.51× (1.45–1.57) | 2.38× (2.34–2.42) | 1.99× (1.94–2.04) | 2.72× (2.68–2.76) | 2.63× (2.59–2.66) | 2.07× (2.04–2.10) | 2.06× (2.03–2.09) | 0.87× (0.85–0.88) | 0.95× (0.93–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.55× (1.55–1.56) | 3.09× (3.07–3.10) | 2.70× (2.67–2.73) | 2.83× (2.82–2.84) | 3.22× (3.20–3.24) | 3.77× (3.75–3.78) | 3.96× (3.94–3.99) | 1.54× (1.53–1.55) | 1.79× (1.78–1.80) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.43–1.46) | 2.88× (2.84–2.91) | 2.46× (2.43–2.48) | 3.46× (3.40–3.52) | 2.92× (2.89–2.96) | 3.34× (3.30–3.38) | 3.51× (3.47–3.56) | 1.37× (1.36–1.38) | 1.58× (1.56–1.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.52–1.64) | 2.58× (2.52–2.63) | 2.05× (1.98–2.11) | 2.89× (2.82–2.96) | 2.72× (2.67–2.76) | 2.34× (2.29–2.38) | 2.46× (2.41–2.50) | 0.96× (0.94–0.97) | 1.11× (1.09–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.42× (2.40–2.44) | 3.32× (3.30–3.35) | 2.79× (2.77–2.80) | 6.04× (5.92–6.16) | 2.09× (2.07–2.10) | 2.11× (2.09–2.13) | 0.95× (0.94–0.96) | 1.35× (1.33–1.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.52–0.54) | 2.05× (2.01–2.08) | 3.67× (3.60–3.75) | 2.51× (2.46–2.56) | 5.24× (5.08–5.39) | 1.81× (1.77–1.85) | 1.83× (1.80–1.87) | 0.83× (0.81–0.84) | 1.17× (1.15–1.20) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.54) | 2.28× (2.23–2.33) | 3.71× (3.64–3.78) | 2.59× (2.54–2.64) | 5.48× (5.30–5.65) | 1.86× (1.83–1.90) | 1.89× (1.85–1.92) | 0.85× (0.83–0.87) | 1.20× (1.18–1.23) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.84× (1.76–1.92) | 3.90× (3.74–4.07) | 4.31× (4.13–4.50) | 4.62× (4.42–4.83) | 4.83× (4.62–5.04) | 4.76× (4.56–4.97) | 4.65× (4.45–4.86) | 1.53× (1.47–1.60) | 1.68× (1.61–1.76) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.62× (1.55–1.68) | 3.34× (3.26–3.41) | 3.66× (3.58–3.74) | 4.31× (4.22–4.41) | 3.98× (3.89–4.08) | 3.86× (3.77–3.95) | 3.77× (3.68–3.87) | 1.24× (1.21–1.27) | 1.36× (1.33–1.40) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.69× (1.62–1.76) | 2.69× (2.60–2.78) | 2.92× (2.88–2.96) | 3.38× (3.34–3.42) | 3.23× (3.17–3.29) | 2.52× (2.49–2.56) | 2.47× (2.43–2.50) | 0.81× (0.80–0.82) | 0.89× (0.88–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.32× (0.30–0.33) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.98× (10.97–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.33–0.37) | 0.43× (0.36–0.52) | 0.72× (0.68–0.76) | 0.61× (0.57–0.64) | 5.71× (5.42–6.02) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.35–0.40) | 0.49× (0.40–0.60) | 0.65× (0.62–0.69) | 0.58× (0.54–0.62) | 3.84× (3.60–4.10) | 0.09× (0.08–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.66× (0.66–0.66) | 7.96× (7.93–7.99) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.68–0.72) | 0.63× (0.57–0.70) | 1.67× (1.62–1.71) | 0.84× (0.81–0.86) | 6.06× (5.90–6.22) | 0.49× (0.48–0.51) | 0.51× (0.50–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.73) | 0.73× (0.67–0.80) | 1.25× (1.21–1.31) | 0.77× (0.74–0.80) | 3.68× (3.53–3.84) | 0.29× (0.28–0.30) | 0.30× (0.29–0.32) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.72× (0.72–0.73) | 5.15× (5.13–5.17) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.38× (0.35–0.41) | 0.88× (0.86–0.91) | 0.70× (0.68–0.72) | 3.05× (2.97–3.13) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.64–0.67) | 0.46× (0.45–0.47) | 0.87× (0.86–0.88) | 0.82× (0.80–0.84) | 2.54× (2.50–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.49) | 0.37× (0.35–0.39) | 0.82× (0.79–0.85) | 0.68× (0.64–0.73) | 2.83× (2.72–2.94) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.49× (0.45–0.53) | 0.81× (0.79–0.82) | 0.82× (0.79–0.86) | 2.34× (2.29–2.38) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.31× (5.30–5.31) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.91) | 6.21× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.40) | 0.67× (0.66–0.67) | 6.39× (6.38–6.40) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.47× (0.44–0.50) | 0.42× (0.39–0.46) | 1.10× (1.03–1.17) | 0.71× (0.66–0.76) | 3.47× (3.29–3.67) | 0.21× (0.20–0.22) | 0.22× (0.21–0.24) | 0.34× (0.32–0.36) | 0.17× (0.16–0.18) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 55 MB | 0.50× (0.49–0.51) | 0.47× (0.43–0.53) | 0.92× (0.90–0.93) | 0.75× (0.72–0.78) | 2.34× (2.30–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 538 | 1.87 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 910 | 2.01 | 1.10× (1.06–1.14) | 84% | 1.02× (1.01–1.03) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,155 | 3.14 | 1.67× (1.64–1.70) | 54% | 1.21× (1.18–1.24) | 1.64× (1.60–1.68) |

**Best:** mibz2 at 4 threads (1,155 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (360 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.06, lbzip2 1t 0.06, lbzip2 2t 0.23, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.01 | 0.07 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3692 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.88 | 10.5% |
| MTF + RLE2 | 1.54 | 18.4% |
| T-vector build | 1.48 | 17.7% |
| walk (the one each build uses) | 4.03 | 48.0% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.39 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.058 | (in its total) | 2.70× (2.69–2.71) |  |
| walk | 1.036 | 2.059 | (in its total) | 1.98× (1.98–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.041 | (in its total) | 1.13× (1.12–1.15) |  |
| inverse BWT, total | 1.464 | 3.158 | 1.887 | 2.15× (2.15–2.16) | 1.29× (1.28–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m49s ± 6m45s |
| 4 threads | 26m29s ± 3m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m29s ± 2m39s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 18m29s ± 2m39s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.42 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3692.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
