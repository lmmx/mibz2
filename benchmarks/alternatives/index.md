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

The latest record: [`d207f65`](<https://github.com/lmmx/mibz2/commit/d207f65433dfc07e9e87b5f3a4a8817d200eaa4b>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44612099070>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.392 s | 535 | 5.1 | 3.47× (3.45–3.48) | 7.84× (7.81–7.88) | 8.38× (8.36–8.41) | 9.27× (9.23–9.30) | 8.41× (8.37–8.45) | 8.80× (8.75–8.86) | 8.61× (8.57–8.65) | 2.69× (2.68–2.70) | 2.39× (2.38–2.41) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.220 s | 953 | 5.5 | 3.43× (3.39–3.47) | 7.32× (7.21–7.42) | 8.83× (8.74–8.91) | 8.79× (8.72–8.87) | 7.96× (7.89–8.02) | 15.62× (15.48–15.76) | 15.28× (15.15–15.41) | 4.78× (4.73–4.82) | 4.25× (4.21–4.29) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.189 s | 1,113 | 8.6 | 3.37× (3.32–3.42) | 6.13× (6.03–6.24) | 7.23× (7.10–7.35) | 7.14× (7.03–7.25) | 6.97× (6.86–7.09) | 18.26× (17.92–18.59) | 17.86× (17.55–18.17) | 5.58× (5.50–5.67) | 4.97× (4.88–5.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,266 | 2.4 | 0.65× (0.63–0.66) | 3.20× (3.12–3.27) | 4.55× (4.46–4.65) | 3.79× (3.71–3.87) | 6.45× (6.33–6.58) | 2.87× (2.81–2.93) | 2.91× (2.85–2.96) | 1.14× (1.12–1.16) | 1.29× (1.26–1.32) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,227 | 2.5 | 0.57× (0.56–0.58) | 3.13× (3.08–3.18) | 6.32× (6.22–6.42) | 3.74× (3.67–3.80) | 6.31× (6.23–6.39) | 2.81× (2.77–2.85) | 2.85× (2.81–2.89) | 1.12× (1.10–1.14) | 1.27× (1.24–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,205 | 2.5 | 0.56× (0.56–0.57) | 3.09× (3.05–3.14) | 6.05× (5.80–6.30) | 3.71× (3.66–3.76) | 6.26× (6.16–6.36) | 2.78× (2.74–2.81) | 2.81× (2.77–2.85) | 1.10× (1.08–1.13) | 1.25× (1.23–1.27) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 642 | 4.5 | 2.88× (2.84–2.93) | 3.33× (3.30–3.36) | 4.03× (3.99–4.08) | 4.32× (4.27–4.36) | 4.75× (4.71–4.80) | 3.18× (3.14–3.21) | 2.95× (2.92–2.99) | 1.98× (1.94–2.02) | 2.12× (2.00–2.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,016 | 5.4 | 2.52× (2.32–2.73) | 3.15× (3.02–3.28) | 4.37× (4.11–4.64) | 3.97× (3.72–4.24) | 5.23× (5.01–5.46) | 4.98× (4.79–5.18) | 4.63× (4.43–4.84) | 3.11× (2.97–3.25) | 3.33× (3.06–3.63) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,021 | 9.4 | 2.07× (1.99–2.15) | 2.13× (2.04–2.21) | 3.18× (2.99–3.38) | 2.53× (2.43–2.65) | 4.33× (4.20–4.48) | 4.99× (4.85–5.14) | 4.65× (4.50–4.80) | 3.12× (2.99–3.25) | 3.34× (3.21–3.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 38.9 | 1.38× (1.37–1.39) | 3.02× (2.99–3.05) | 2.75× (2.67–2.83) | 2.81× (2.80–2.83) | 3.06× (3.04–3.08) | 3.43× (3.41–3.45) | 3.52× (3.49–3.55) | 1.25× (1.24–1.25) | 1.25× (1.24–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 117 | 42.7 | 1.31× (1.27–1.34) | 2.87× (2.78–2.96) | 2.78× (2.68–2.90) | 3.46× (3.36–3.55) | 3.15× (3.07–3.23) | 5.66× (5.52–5.81) | 5.82× (5.67–5.96) | 2.06× (2.01–2.11) | 2.07× (2.02–2.12) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 142 | 59.3 | 1.40× (1.37–1.43) | 2.71× (2.65–2.78) | 2.57× (2.54–2.61) | 3.21× (3.13–3.30) | 3.20× (3.14–3.26) | 6.90× (6.78–7.02) | 7.08× (6.96–7.20) | 2.51× (2.47–2.55) | 2.52× (2.47–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 35.8 | 1.40× (1.33–1.48) | 3.31× (3.12–3.51) | 2.84× (2.69–3.00) | 2.89× (2.74–3.05) | 3.22× (3.04–3.41) | 3.98× (3.76–4.21) | 4.28× (4.06–4.52) | 1.42× (1.35–1.50) | 1.50× (1.42–1.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 132 | 39.2 | 1.46× (1.29–1.65) | 3.16× (3.08–3.24) | 2.91× (2.86–2.97) | 3.91× (3.83–4.00) | 3.35× (3.28–3.41) | 6.97× (6.84–7.11) | 7.51× (7.37–7.65) | 2.50× (2.45–2.54) | 2.63× (2.58–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 55.0 | 1.47× (1.43–1.51) | 2.97× (2.90–3.05) | 2.72× (2.66–2.79) | 3.67× (3.62–3.73) | 3.24× (3.19–3.28) | 8.27× (8.13–8.42) | 8.91× (8.77–9.04) | 2.96× (2.91–3.01) | 3.12× (3.07–3.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 918 | 3.4 | 0.70× (0.69–0.71) | 2.86× (2.80–2.93) | 3.94× (3.86–4.03) | 3.20× (3.13–3.27) | 6.89× (6.77–7.01) | 2.45× (2.41–2.50) | 2.50× (2.46–2.54) | 1.04× (1.03–1.06) | 0.97× (0.95–1.00) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 889 | 3.6 | 0.54× (0.53–0.55) | 2.73× (2.70–2.77) | 5.57× (5.49–5.64) | 3.09× (3.04–3.15) | 6.62× (6.47–6.77) | 2.34× (2.31–2.37) | 2.39× (2.36–2.42) | 1.00× (0.97–1.02) | 0.93× (0.92–0.94) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 885 | 3.5 | 0.55× (0.53–0.56) | 2.73× (2.67–2.78) | 5.42× (5.20–5.66) | 3.10× (3.04–3.17) | 6.55× (6.43–6.67) | 2.33× (2.30–2.36) | 2.38× (2.33–2.43) | 0.99× (0.97–1.02) | 0.93× (0.91–0.95) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 0.062 s | 215 | 13.0 | 1.83× (1.83–1.84) | 4.11× (4.07–4.15) | 4.62× (4.59–4.65) | 4.71× (4.68–4.73) | 4.83× (4.81–4.86) | 5.09× (5.06–5.11) | 5.05× (5.03–5.07) | 1.47× (1.47–1.48) | 1.47× (1.47–1.48) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 0.040 s | 335 | 15.1 | 1.62× (1.57–1.68) | 3.63× (3.54–3.73) | 4.59× (4.49–4.69) | 4.66× (4.54–4.78) | 4.71× (4.58–4.84) | 7.93× (7.72–8.15) | 7.88× (7.65–8.11) | 2.30× (2.23–2.36) | 2.30× (2.23–2.36) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 0.035 s | 376 | 22.7 | 1.57× (1.54–1.61) | 3.06× (2.92–3.21) | 3.79× (3.72–3.86) | 3.99× (3.91–4.08) | 4.34× (4.25–4.44) | 8.91× (8.77–9.06) | 8.85× (8.70–9.00) | 2.58× (2.54–2.62) | 2.58× (2.54–2.63) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.45× (3.44–3.47) | 7.80× (7.77–7.83) | 8.33× (8.30–8.36) | 9.23× (9.19–9.27) | 8.38× (8.33–8.42) | 8.76× (8.71–8.82) | 8.57× (8.53–8.62) | 2.68× (2.67–2.69) | 2.38× (2.36–2.39) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.28× (3.26–3.30) | 7.48× (7.40–7.56) | 8.26× (8.21–8.30) | 9.00× (8.94–9.07) | 7.80× (7.75–7.85) | 8.05× (7.99–8.10) | 7.87× (7.82–7.93) | 2.46× (2.45–2.48) | 2.18× (2.16–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.41× (3.36–3.46) | 6.86× (6.73–7.00) | 7.33× (7.18–7.48) | 7.99× (7.84–8.15) | 7.13× (6.99–7.26) | 5.21× (5.11–5.30) | 5.09× (5.00–5.19) | 1.59× (1.56–1.63) | 1.41× (1.38–1.44) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.67–0.70) | 2.91× (2.85–2.97) | 3.94× (3.87–4.02) | 3.52× (3.45–3.60) | 5.94× (5.82–6.05) | 2.69× (2.63–2.75) | 2.73× (2.68–2.78) | 1.13× (1.11–1.15) | 0.83× (0.82–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 2.85× (2.80–2.90) | 5.49× (5.37–5.61) | 3.43× (3.37–3.49) | 5.70× (5.61–5.79) | 2.59× (2.54–2.63) | 2.62× (2.58–2.66) | 1.09× (1.07–1.10) | 0.80× (0.78–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.24× (3.13–3.34) | 5.43× (5.28–5.59) | 3.48× (3.42–3.53) | 5.77× (5.68–5.86) | 2.61× (2.58–2.65) | 2.65× (2.61–2.69) | 1.10× (1.07–1.12) | 0.81× (0.79–0.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.78× (2.74–2.83) | 3.08× (3.05–3.12) | 3.62× (3.58–3.66) | 4.14× (4.10–4.19) | 4.55× (4.51–4.60) | 3.06× (3.03–3.09) | 2.85× (2.82–2.88) | 1.93× (1.89–1.97) | 1.92× (1.88–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.46× (2.41–2.50) | 2.75× (2.68–2.82) | 3.34× (3.26–3.41) | 3.55× (3.49–3.62) | 3.89× (3.82–3.96) | 2.54× (2.49–2.59) | 2.37× (2.32–2.41) | 1.60× (1.56–1.65) | 1.60× (1.55–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.14× (2.06–2.22) | 1.87× (1.83–1.90) | 2.36× (2.33–2.39) | 2.31× (2.28–2.33) | 2.41× (2.38–2.43) | 1.44× (1.42–1.46) | 1.34× (1.32–1.36) | 0.91× (0.89–0.92) | 0.90× (0.89–0.92) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.39) | 2.93× (2.91–2.95) | 2.69× (2.63–2.75) | 2.79× (2.77–2.81) | 3.03× (3.01–3.05) | 3.39× (3.37–3.42) | 3.48× (3.46–3.51) | 1.24× (1.23–1.25) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.32) | 2.80× (2.72–2.87) | 2.54× (2.48–2.61) | 3.26× (3.17–3.34) | 2.80× (2.74–2.87) | 3.05× (2.98–3.12) | 3.13× (3.06–3.21) | 1.12× (1.09–1.15) | 1.10× (1.08–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.51–1.56) | 2.83× (2.78–2.88) | 2.43× (2.38–2.48) | 3.17× (3.12–3.21) | 2.81× (2.76–2.85) | 2.24× (2.21–2.27) | 2.30× (2.26–2.34) | 0.82× (0.81–0.83) | 0.81× (0.80–0.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.32–1.47) | 3.22× (3.05–3.41) | 2.77× (2.63–2.91) | 2.87× (2.72–3.02) | 3.19× (3.02–3.38) | 3.94× (3.73–4.16) | 4.24× (4.02–4.47) | 1.42× (1.34–1.49) | 1.43× (1.35–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.36× (1.35–1.37) | 3.17× (3.15–3.18) | 2.73× (2.71–2.75) | 3.67× (3.65–3.70) | 3.06× (3.04–3.08) | 3.69× (3.67–3.71) | 3.98× (3.95–4.00) | 1.33× (1.32–1.34) | 1.34× (1.33–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.57–1.67) | 3.26× (3.20–3.32) | 2.48× (2.43–2.54) | 3.48× (3.42–3.55) | 2.98× (2.93–3.03) | 2.64× (2.59–2.69) | 2.84× (2.79–2.89) | 0.95× (0.93–0.97) | 0.96× (0.94–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.74× (0.73–0.76) | 2.47× (2.41–2.53) | 3.43× (3.37–3.50) | 2.89× (2.83–2.95) | 6.07× (5.98–6.17) | 2.25× (2.21–2.29) | 2.29× (2.25–2.34) | 1.04× (1.02–1.05) | 0.76× (0.75–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.62) | 2.28× (2.25–2.31) | 4.41× (4.35–4.47) | 2.81× (2.76–2.85) | 5.77× (5.63–5.92) | 2.11× (2.09–2.14) | 2.16× (2.13–2.18) | 0.98× (0.95–1.00) | 0.72× (0.71–0.72) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.62) | 2.66× (2.58–2.75) | 4.45× (4.34–4.56) | 2.91× (2.85–2.97) | 5.90× (5.80–6.01) | 2.17× (2.14–2.21) | 2.22× (2.18–2.26) | 1.00× (0.98–1.03) | 0.74× (0.72–0.75) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 1.81× (1.80–1.82) | 3.98× (3.96–4.01) | 4.48× (4.46–4.50) | 4.60× (4.57–4.64) | 4.73× (4.70–4.76) | 4.97× (4.95–5.00) | 4.94× (4.92–4.96) | 1.46× (1.45–1.47) | 1.36× (1.36–1.37) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 1.63× (1.61–1.64) | 3.60× (3.55–3.66) | 4.22× (4.19–4.25) | 4.49× (4.44–4.54) | 4.15× (4.11–4.19) | 4.25× (4.20–4.29) | 4.22× (4.17–4.27) | 1.25× (1.23–1.26) | 1.16× (1.15–1.18) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 1.80× (1.74–1.87) | 3.28× (3.20–3.35) | 3.68× (3.59–3.78) | 4.08× (3.98–4.17) | 3.69× (3.61–3.77) | 2.88× (2.81–2.94) | 2.86× (2.79–2.92) | 0.84× (0.83–0.86) | 0.79× (0.77–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.27–0.30) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.95× (10.92–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.34× (0.32–0.35) | 0.50× (0.43–0.58) | 0.68× (0.65–0.71) | 0.58× (0.56–0.61) | 5.50× (5.28–5.73) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.35) | 0.56× (0.47–0.68) | 0.59× (0.57–0.62) | 0.52× (0.50–0.55) | 3.47× (3.33–3.62) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.36× (4.35–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.42× (0.41–0.44) | 1.68× (1.68–1.68) | 0.65× (0.65–0.66) | 7.92× (7.90–7.94) | 0.64× (0.64–0.64) | 0.68× (0.68–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.70) | 0.63× (0.57–0.68) | 1.62× (1.59–1.66) | 0.81× (0.79–0.82) | 5.86× (5.74–5.97) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.65× (0.63–0.67) | 0.65× (0.59–0.72) | 1.15× (1.12–1.19) | 0.71× (0.68–0.73) | 3.43× (3.33–3.54) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.48–0.53) | 0.41× (0.38–0.44) | 0.91× (0.87–0.95) | 0.71× (0.67–0.75) | 3.13× (3.00–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.86× (0.83–0.89) | 0.80× (0.77–0.83) | 2.50× (2.42–2.59) | 0.15× (0.15–0.16) | 0.16× (0.15–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.52) | 0.37× (0.36–0.39) | 0.87× (0.83–0.92) | 0.72× (0.68–0.75) | 2.99× (2.85–3.14) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.49× (0.46–0.52) | 0.82× (0.81–0.83) | 0.84× (0.81–0.86) | 2.36× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.07× (1.06–1.07) | 0.77× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.59) | 0.76× (0.76–0.77) | 5.26× (5.25–5.28) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.25× (0.25–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.77× (0.76–0.77) | 5.27× (5.25–5.29) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.65–0.67) | 6.33× (6.31–6.36) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 38 MB | 0.45× (0.45–0.46) | 0.39× (0.35–0.43) | 1.05× (1.02–1.08) | 0.68× (0.64–0.71) | 3.32× (3.24–3.40) | 0.21× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 54 MB | 0.52× (0.49–0.54) | 0.48× (0.43–0.52) | 0.95× (0.92–0.99) | 0.79× (0.75–0.83) | 2.43× (2.34–2.53) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 535 | 1.88 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 953 | 2.04 | 1.09× (1.08–1.10) | 89% | 1.04× (1.04–1.05) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,113 | 3.18 | 1.68× (1.65–1.72) | 52% | 1.48× (1.47–1.49) | 1.66× (1.64–1.69) |

**Best:** mibz2 at 4 threads (1,113 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (330 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.24, lbzip2 1t 0.04, lbzip2 2t 0.22, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.49 | 0.02 | 0.08 | 0.189 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2708 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.81 | 10.8% |
| MTF + RLE2 | 1.14 | 15.3% |
| T-vector build | 1.07 | 14.4% |
| walk (the one each build uses) | 4.14 | 55.8% |
| of which: segment assembly | 0.22 | 2.9% |
| RLE1 | 0.17 | 2.3% |
| CRC | 0.10 | 1.3% |
| **total** | 7.42 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.370 | 0.935 | (in its total) | 2.52× (2.52–2.53) |  |
| walk | 1.445 | 2.742 | (in its total) | 1.90× (1.90–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.91× (0.91–0.91) |  |
| inverse BWT, total | 1.895 | 3.749 | 1.971 | 1.98× (1.98–1.98) | 1.04× (1.04–1.04) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m57s ± 6m30s |
| 4 threads | 27m28s ± 3m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m47s ± 2m34s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 17m47s ± 2m34s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.33 ns a byte of output at 4 threads (CPU, summed over them), 1.52 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,615,664 | 13,288,779 | \`0b9f69f4e27fcf7e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 2707.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
