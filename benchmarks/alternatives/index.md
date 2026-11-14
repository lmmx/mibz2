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

The latest record: [`28a30e6`](<https://github.com/lmmx/mibz2/commit/28a30e6d11e716ca0c6a03ad17ae7ff34ed8976b>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43637482861>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.592 s | 354 | 9.9 | 3.12× (3.07–3.17) | 6.89× (6.70–7.08) | 7.91× (7.71–8.11) | 8.08× (7.91–8.26) | 7.43× (7.26–7.62) | 8.22× (8.04–8.41) | 7.52× (7.42–7.63) | 2.22× (2.19–2.25) | 2.10× (2.08–2.11) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.343 s | 611 | 11.2 | 2.97× (2.73–3.24) | 6.22× (5.82–6.65) | 7.95× (7.27–8.69) | 7.22× (6.69–7.78) | 6.48× (6.02–6.98) | 13.43× (12.32–14.64) | 12.29× (11.39–13.25) | 3.62× (3.35–3.92) | 3.42× (3.16–3.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.303 s | 691 | 19.2 | 3.34× (3.26–3.42) | 5.30× (5.06–5.54) | 7.02× (6.61–7.47) | 5.97× (5.73–6.21) | 6.12× (5.95–6.28) | 15.94× (15.47–16.42) | 14.58× (14.31–14.87) | 4.30× (4.25–4.35) | 4.06× (4.00–4.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 939 | 4.1 | 0.66× (0.65–0.68) | 3.02× (2.84–3.22) | 5.17× (5.05–5.30) | 3.39× (3.31–3.47) | 6.21× (6.06–6.35) | 2.63× (2.58–2.69) | 2.58× (2.53–2.64) | 1.23× (1.20–1.27) | 1.06× (0.94–1.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 913 | 4.2 | 0.61× (0.60–0.63) | 2.81× (2.76–2.85) | 6.73× (6.54–6.92) | 3.33× (3.28–3.39) | 5.99× (5.89–6.08) | 2.55× (2.51–2.59) | 2.50× (2.46–2.55) | 1.19× (1.15–1.24) | 1.03× (0.92–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 897 | 4.2 | 0.61× (0.60–0.63) | 2.74× (2.70–2.79) | 6.63× (6.45–6.82) | 3.24× (3.18–3.29) | 5.99× (5.87–6.11) | 2.49× (2.45–2.54) | 2.45× (2.41–2.49) | 1.17× (1.13–1.21) | 1.00× (0.89–1.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.039 s | 507 | 7.2 | 3.15× (3.07–3.24) | 3.15× (3.09–3.21) | 4.39× (4.31–4.48) | 4.03× (3.95–4.11) | 4.58× (4.48–4.67) | 3.63× (3.56–3.70) | 2.83× (2.77–2.90) | 2.33× (2.25–2.41) | 2.25× (2.13–2.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.027 s | 719 | 9.2 | 2.31× (2.02–2.64) | 2.56× (2.24–2.92) | 4.03× (3.50–4.64) | 3.13× (2.74–3.58) | 4.55× (4.00–5.18) | 4.95× (4.35–5.62) | 3.86× (3.41–4.38) | 3.17× (2.82–3.56) | 3.06× (2.69–3.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 s | 681 | 18.1 | 2.34× (2.27–2.41) | 1.85× (1.78–1.92) | 3.08× (2.92–3.24) | 2.24× (2.06–2.44) | 4.00× (3.90–4.11) | 4.87× (4.72–5.02) | 3.80× (3.68–3.92) | 3.12× (2.96–3.28) | 3.01× (2.88–3.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.171 s | 47 | 75.4 | 1.25× (1.24–1.26) | 2.55× (2.37–2.74) | 2.39× (2.26–2.52) | 2.33× (2.26–2.40) | 2.68× (2.59–2.77) | 2.97× (2.93–3.02) | 2.94× (2.85–3.03) | 1.22× (1.21–1.23) | 1.24× (1.24–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.103 s | 78 | 81.4 | 1.24× (1.17–1.32) | 2.44× (2.33–2.54) | 2.52× (2.38–2.67) | 2.94× (2.88–2.99) | 2.87× (2.75–2.99) | 4.99× (4.93–5.06) | 4.93× (4.79–5.06) | 2.05× (2.02–2.09) | 2.09× (2.05–2.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.083 s | 97 | 110.0 | 1.43× (1.38–1.48) | 2.70× (2.50–2.91) | 2.71× (2.59–2.83) | 2.91× (2.77–3.07) | 3.32× (3.11–3.55) | 6.21× (6.06–6.38) | 6.13× (5.94–6.33) | 2.56× (2.50–2.61) | 2.60× (2.54–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.195 s | 49 | 71.7 | 1.31× (1.30–1.32) | 2.77× (2.67–2.87) | 2.53× (2.42–2.64) | 2.47× (2.42–2.52) | 2.83× (2.75–2.91) | 3.53× (3.39–3.67) | 3.62× (3.54–3.71) | 1.37× (1.35–1.38) | 1.40× (1.39–1.41) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.112 s | 86 | 77.6 | 1.23× (1.16–1.29) | 2.61× (2.41–2.82) | 2.51× (2.34–2.68) | 3.13× (2.93–3.34) | 2.87× (2.66–3.09) | 6.00× (5.87–6.13) | 6.17× (5.96–6.38) | 2.32× (2.23–2.43) | 2.38× (2.27–2.50) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.092 s | 104 | 110.0 | 1.57× (1.51–1.63) | 2.67× (2.51–2.85) | 2.59× (2.44–2.76) | 3.04× (2.91–3.17) | 3.05× (2.91–3.20) | 7.35× (6.93–7.80) | 7.56× (7.21–7.92) | 2.85× (2.75–2.95) | 2.92× (2.83–3.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 699 | 5.6 | 0.74× (0.71–0.77) | 2.87× (2.79–2.95) | 4.86× (4.74–4.99) | 2.99× (2.92–3.07) | 6.97× (6.79–7.15) | 2.34× (2.29–2.39) | 2.31× (2.25–2.38) | 1.08× (1.05–1.11) | 1.45× (1.41–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 637 | 6.3 | 0.61× (0.60–0.62) | 2.62× (2.57–2.68) | 5.73× (5.61–5.85) | 2.78× (2.72–2.84) | 6.44× (6.25–6.64) | 2.13× (2.09–2.17) | 2.11× (2.06–2.15) | 0.99× (0.96–1.02) | 1.32× (1.29–1.35) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 639 | 6.2 | 0.62× (0.60–0.63) | 2.64× (2.61–2.67) | 5.69× (5.56–5.83) | 2.78× (2.73–2.82) | 6.46× (6.39–6.53) | 2.14× (2.11–2.17) | 2.11× (2.08–2.15) | 0.99× (0.97–1.01) | 1.32× (1.31–1.34) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.082 s | 149 | 23.9 | 1.77× (1.73–1.82) | 3.72× (3.59–3.86) | 4.53× (4.33–4.73) | 4.40× (4.24–4.58) | 4.59× (4.41–4.78) | 4.77× (4.61–4.94) | 4.62× (4.43–4.82) | 1.41× (1.35–1.48) | 1.40× (1.35–1.45) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.052 s | 238 | 27.3 | 1.47× (1.31–1.65) | 3.29× (2.88–3.77) | 4.08× (3.67–4.55) | 4.65× (4.05–5.33) | 4.26× (3.79–4.80) | 7.13× (6.41–7.93) | 6.91× (6.12–7.80) | 2.11× (1.87–2.39) | 2.10× (1.91–2.30) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.047 s | 264 | 42.2 | 1.70× (1.63–1.76) | 3.10× (2.90–3.31) | 3.71× (3.48–3.95) | 4.15× (4.00–4.31) | 4.45× (4.27–4.63) | 8.31× (8.05–8.59) | 8.06× (7.72–8.41) | 2.47× (2.38–2.55) | 2.44× (2.38–2.51) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (3.06–3.16) | 6.86× (6.68–7.05) | 7.87× (7.67–8.08) | 8.06× (7.89–8.23) | 7.41× (7.24–7.59) | 8.20× (8.02–8.39) | 7.50× (7.40–7.61) | 2.22× (2.18–2.25) | 2.08× (2.07–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.94× (2.90–2.99) | 6.52× (6.28–6.76) | 7.80× (7.57–8.05) | 7.68× (7.48–7.89) | 6.63× (6.47–6.80) | 7.20× (7.01–7.39) | 6.59× (6.48–6.70) | 1.95× (1.91–1.99) | 1.83× (1.80–1.86) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.17× (3.08–3.25) | 5.45× (5.21–5.70) | 6.68× (6.31–7.06) | 6.14× (5.91–6.38) | 5.82× (5.64–6.00) | 4.20× (4.08–4.32) | 3.84× (3.77–3.92) | 1.14× (1.12–1.15) | 1.07× (1.05–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.69× (0.68–0.71) | 2.70× (2.63–2.78) | 4.76× (4.66–4.86) | 3.20× (3.13–3.26) | 5.77× (5.65–5.90) | 2.50× (2.45–2.54) | 2.45× (2.41–2.50) | 1.21× (1.18–1.25) | 0.96× (0.93–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.65× (0.64–0.67) | 2.57× (2.53–2.62) | 5.92× (5.82–6.03) | 3.12× (3.07–3.17) | 5.52× (5.43–5.61) | 2.39× (2.35–2.43) | 2.35× (2.30–2.39) | 1.16× (1.12–1.20) | 0.92× (0.90–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.64× (0.63–0.65) | 2.91× (2.88–2.95) | 5.88× (5.79–5.98) | 3.09× (3.04–3.14) | 5.60× (5.50–5.70) | 2.38× (2.35–2.41) | 2.34× (2.30–2.37) | 1.16× (1.12–1.19) | 0.91× (0.89–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.05× (2.98–3.13) | 2.89× (2.83–2.94) | 4.07× (4.00–4.15) | 3.89× (3.82–3.96) | 4.41× (4.33–4.50) | 3.51× (3.45–3.57) | 2.75× (2.69–2.81) | 2.27× (2.19–2.34) | 2.05× (1.99–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.52× (2.40–2.64) | 2.43× (2.33–2.54) | 3.50× (3.36–3.65) | 3.11× (2.99–3.23) | 3.55× (3.41–3.69) | 2.73× (2.63–2.84) | 2.14× (2.06–2.22) | 1.76× (1.71–1.82) | 1.59× (1.52–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.46× (2.38–2.55) | 1.64× (1.59–1.69) | 2.40× (2.33–2.48) | 1.94× (1.85–2.03) | 2.18× (2.11–2.26) | 1.40× (1.36–1.45) | 1.10× (1.06–1.13) | 0.90× (0.86–0.95) | 0.82× (0.79–0.84) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.25× (1.24–1.25) | 2.50× (2.32–2.68) | 2.34× (2.22–2.47) | 2.32× (2.25–2.39) | 2.66× (2.57–2.75) | 2.95× (2.91–3.00) | 2.91× (2.83–3.00) | 1.22× (1.21–1.23) | 1.20× (1.20–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.21× (1.19–1.23) | 2.47× (2.35–2.59) | 2.37× (2.22–2.54) | 2.78× (2.68–2.88) | 2.59× (2.47–2.72) | 2.72× (2.68–2.77) | 2.69× (2.61–2.77) | 1.13× (1.11–1.14) | 1.11× (1.09–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.48–1.65) | 2.98× (2.72–3.25) | 2.64× (2.51–2.77) | 2.92× (2.74–3.11) | 3.06× (2.83–3.32) | 2.03× (1.97–2.08) | 2.00× (1.94–2.06) | 0.84× (0.82–0.86) | 0.83× (0.81–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.31× (1.29–1.32) | 2.72× (2.64–2.81) | 2.50× (2.39–2.60) | 2.45× (2.40–2.50) | 2.81× (2.73–2.89) | 3.50× (3.36–3.64) | 3.60× (3.51–3.69) | 1.36× (1.35–1.38) | 1.37× (1.37–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.26× (1.24–1.28) | 2.66× (2.56–2.76) | 2.41× (2.30–2.53) | 3.00× (2.92–3.08) | 2.67× (2.54–2.82) | 3.24× (3.11–3.38) | 3.33× (3.24–3.43) | 1.26× (1.24–1.28) | 1.27× (1.25–1.29) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.68× (1.62–1.75) | 2.92× (2.74–3.11) | 2.42× (2.26–2.60) | 2.84× (2.73–2.95) | 2.79× (2.65–2.95) | 2.30× (2.18–2.42) | 2.36× (2.28–2.45) | 0.89× (0.87–0.92) | 0.90× (0.88–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.77× (0.75–0.79) | 2.48× (2.40–2.56) | 4.03× (3.95–4.11) | 2.75× (2.69–2.82) | 6.27× (6.11–6.42) | 2.18× (2.14–2.23) | 2.16× (2.10–2.22) | 1.07× (1.04–1.10) | 0.88× (0.86–0.90) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.66× (0.64–0.67) | 2.22× (2.18–2.26) | 4.69× (4.57–4.81) | 2.59× (2.54–2.64) | 5.76× (5.61–5.92) | 1.97× (1.93–2.01) | 1.94× (1.90–1.99) | 0.97× (0.94–1.00) | 0.79× (0.77–0.81) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.65× (0.64–0.66) | 2.60× (2.52–2.68) | 4.72× (4.67–4.76) | 2.64× (2.60–2.67) | 5.88× (5.81–5.94) | 2.01× (1.99–2.03) | 1.98× (1.95–2.02) | 0.99× (0.97–1.00) | 0.81× (0.80–0.82) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.76× (1.71–1.80) | 3.60× (3.48–3.73) | 4.39× (4.23–4.57) | 4.33× (4.16–4.50) | 4.51× (4.33–4.70) | 4.68× (4.52–4.85) | 4.54× (4.35–4.73) | 1.41× (1.34–1.47) | 1.32× (1.29–1.35) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.68× (1.64–1.71) | 3.57× (3.38–3.77) | 4.28× (4.07–4.51) | 4.49× (4.30–4.69) | 4.17× (4.09–4.26) | 4.14× (4.01–4.26) | 4.01× (3.84–4.19) | 1.24× (1.18–1.30) | 1.16× (1.13–1.19) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 2.07× (2.00–2.15) | 3.68× (3.47–3.90) | 4.09× (3.90–4.29) | 3.95× (3.80–4.09) | 4.13× (3.91–4.37) | 2.69× (2.60–2.79) | 2.61× (2.51–2.71) | 0.81× (0.78–0.84) | 0.76× (0.74–0.78) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 33 MB | 0.26× (0.26–0.26) | 0.22× (0.22–0.23) | 0.85× (0.84–0.85) | 0.68× (0.67–0.68) | 9.55× (9.53–9.58) | 0.23× (0.23–0.23) | 0.25× (0.24–0.25) | 0.37× (0.37–0.37) | 0.19× (0.18–0.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 73 MB | 0.26× (0.25–0.27) | 0.34× (0.28–0.40) | 0.55× (0.52–0.57) | 0.45× (0.43–0.47) | 4.33× (4.16–4.50) | 0.10× (0.10–0.11) | 0.11× (0.11–0.12) | 0.17× (0.16–0.17) | 0.08× (0.08–0.09) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 120 MB | 0.26× (0.24–0.28) | 0.36× (0.31–0.41) | 0.47× (0.44–0.50) | 0.40× (0.37–0.43) | 2.70× (2.51–2.91) | 0.06× (0.06–0.07) | 0.07× (0.06–0.07) | 0.10× (0.10–0.11) | 0.05× (0.05–0.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.00–8.09) | 0.65× (0.64–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.66× (0.62–0.69) | 1.63× (1.60–1.66) | 0.82× (0.80–0.83) | 5.93× (5.83–6.03) | 0.48× (0.47–0.49) | 0.50× (0.50–0.51) | 0.55× (0.54–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.63–0.67) | 0.73× (0.68–0.79) | 1.16× (1.13–1.19) | 0.72× (0.70–0.74) | 3.46× (3.36–3.57) | 0.27× (0.26–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.39× (0.36–0.42) | 0.89× (0.86–0.92) | 0.70× (0.66–0.74) | 3.07× (2.98–3.17) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.44–0.47) | 0.86× (0.84–0.88) | 0.81× (0.79–0.84) | 2.50× (2.43–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.38× (0.36–0.40) | 0.85× (0.83–0.87) | 0.71× (0.69–0.74) | 2.89× (2.83–2.96) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.49× (0.46–0.52) | 0.81× (0.80–0.82) | 0.81× (0.78–0.86) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.38× (5.36–5.41) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.30× (5.27–5.32) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.42) | 0.66× (0.66–0.66) | 6.42× (6.41–6.44) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.40× (0.37–0.43) | 1.14× (1.10–1.18) | 0.73× (0.72–0.74) | 3.58× (3.54–3.63) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 54 MB | 0.52× (0.50–0.53) | 0.48× (0.43–0.53) | 0.94× (0.91–0.97) | 0.76× (0.72–0.80) | 2.42× (2.35–2.50) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 354 | 2.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 611 | 3.19 | 1.14× (1.12–1.16) | 82% | 1.08× (1.04–1.12) | 1.08× (1.06–1.09) |
| 4: all 4 CPUs, sharing SMT siblings | 691 | 5.50 | 1.95× (1.92–1.98) | 48% | 1.55× (1.48–1.62) | 1.98× (1.93–2.04) |

**Best:** mibz2 at 4 threads (691 MB/s); lbzip2 at 4 threads (130 MB/s); pbz2 at 4 threads (207 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.04, lbzip2 1t 0.05, lbzip2 2t 0.23, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.25, pbz2 4t 0.32. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 7%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.94 | 0.03 | 0.08 | 0.303 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3492 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m20s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.27 | 9.6% |
| MTF + RLE2 | 1.82 | 13.8% |
| T-vector build | 1.81 | 13.7% |
| walk (the one each build uses) | 7.72 | 58.7% |
| of which: segment assembly | 0.48 | 3.7% |
| RLE1 | 0.37 | 2.8% |
| CRC | 0.16 | 1.2% |
| **total** | 13.15 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.498 | 1.279 | (in its total) | 2.57× (2.56–2.58) |  |
| walk | 2.003 | 3.087 | (in its total) | 1.55× (1.54–1.56) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.141 | 0.142 | (in its total) | 1.02× (0.98–1.05) |  |
| inverse BWT, total | 2.641 | 4.508 | 2.647 | 1.71× (1.70–1.73) | 1.00× (1.00–1.01) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h25m ± 9m55s |
| 4 threads | 44m21s ± 6m15s |
| 4 threads, the sample's overlap gap as a fixed cost | 34m18s ± 4m51s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 34m18s ± 4m51s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.50 ns a byte of output at 4 threads (CPU, summed over them), 2.32 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,443,515 | 12,305,874 | \`da24cf0eff3da816…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 3492.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
