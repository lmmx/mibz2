# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-425/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`28c4dc1`](<https://github.com/lmmx/mibz2/commit/28c4dc135283a2ba4a9263a85d099dad15af15cf>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43727674360>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.542 s | 387 | 2.1 | 2.94× (2.88–2.99) | 6.45× (6.33–6.57) | 7.46× (7.32–7.60) | 7.63× (7.50–7.77) | 7.00× (6.86–7.15) | 7.81× (7.65–7.97) | 7.09× (6.98–7.21) | 2.21× (2.16–2.25) | 2.22× (2.17–2.26) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.305 s | 686 | 2.2 | 2.95× (2.82–3.07) | 6.05× (5.86–6.24) | 7.69× (7.45–7.94) | 7.15× (6.93–7.38) | 6.60× (6.41–6.81) | 13.83× (13.41–14.26) | 12.56× (12.17–12.97) | 3.91× (3.79–4.03) | 3.92× (3.80–4.05) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.266 s | 788 | 3.8 | 3.23× (3.18–3.29) | 4.50× (4.43–4.57) | 5.98× (5.82–6.15) | 5.29× (5.17–5.41) | 5.39× (5.28–5.50) | 15.97× (15.74–16.21) | 14.51× (14.27–14.76) | 4.52× (4.46–4.58) | 4.53× (4.47–4.60) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,020 | 0.9 | 0.66× (0.65–0.68) | 3.13× (3.07–3.19) | 5.65× (5.54–5.76) | 3.62× (3.55–3.69) | 6.64× (6.51–6.78) | 2.80× (2.74–2.85) | 2.76× (2.70–2.82) | 1.17× (1.15–1.19) | 1.06× (1.04–1.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 998 | 0.9 | 0.63× (0.60–0.66) | 2.99× (2.84–3.15) | 6.87× (6.53–7.23) | 3.51× (3.33–3.69) | 6.27× (5.96–6.59) | 2.66× (2.53–2.81) | 2.63× (2.50–2.76) | 1.11× (1.05–1.18) | 1.01× (0.96–1.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 971 | 0.9 | 0.63× (0.62–0.65) | 3.01× (2.94–3.07) | 6.92× (6.76–7.09) | 3.49× (3.42–3.57) | 6.39× (6.23–6.55) | 2.67× (2.62–2.73) | 2.64× (2.58–2.70) | 1.11× (1.08–1.15) | 1.01× (0.99–1.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 541 | 1.6 | 3.05× (3.04–3.07) | 3.37× (3.34–3.39) | 4.45× (4.42–4.48) | 4.19× (4.16–4.23) | 4.81× (4.77–4.84) | 3.86× (3.83–3.89) | 3.00× (2.98–3.03) | 2.15× (2.13–2.17) | 2.24× (2.23–2.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 877 | 1.8 | 2.62× (2.52–2.72) | 3.08× (2.93–3.23) | 4.97× (4.71–5.24) | 3.73× (3.57–3.90) | 5.55× (5.31–5.80) | 6.08× (5.82–6.36) | 4.74× (4.55–4.93) | 3.39× (3.25–3.54) | 3.54× (3.39–3.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 778 | 3.7 | 2.38× (2.31–2.45) | 2.13× (2.06–2.21) | 3.30× (3.18–3.42) | 2.41× (2.33–2.49) | 4.51× (4.33–4.69) | 5.59× (5.39–5.80) | 4.35× (4.21–4.50) | 3.12× (3.01–3.23) | 3.25× (3.14–3.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.146 s | 55 | 14.8 | 1.35× (1.33–1.38) | 2.55× (2.50–2.61) | 2.34× (2.29–2.40) | 2.37× (2.32–2.42) | 2.75× (2.70–2.81) | 3.12× (3.05–3.18) | 3.04× (2.98–3.11) | 1.21× (1.19–1.23) | 1.26× (1.24–1.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 91 | 16.3 | 1.28× (1.22–1.34) | 2.44× (2.31–2.57) | 2.30× (2.21–2.38) | 2.91× (2.82–3.00) | 2.86× (2.75–2.97) | 5.14× (4.96–5.32) | 5.02× (4.84–5.20) | 1.99× (1.91–2.07) | 2.08× (1.99–2.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 105 | 23.6 | 1.45× (1.42–1.48) | 2.14× (2.08–2.19) | 2.09× (2.02–2.15) | 2.57× (2.46–2.68) | 2.82× (2.78–2.86) | 5.99× (5.88–6.11) | 5.85× (5.72–5.98) | 2.32× (2.26–2.38) | 2.43× (2.35–2.51) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.164 s | 59 | 13.8 | 1.42× (1.39–1.46) | 2.86× (2.80–2.93) | 2.49× (2.43–2.56) | 2.55× (2.49–2.61) | 3.04× (2.83–3.25) | 3.76× (3.68–3.84) | 3.91× (3.84–3.99) | 1.39× (1.36–1.43) | 1.46× (1.42–1.49) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.096 s | 100 | 15.2 | 1.31× (1.19–1.43) | 2.64× (2.50–2.79) | 2.42× (2.23–2.63) | 3.32× (3.17–3.47) | 2.92× (2.70–3.15) | 6.17× (5.74–6.65) | 6.43× (5.94–6.95) | 2.29× (2.11–2.48) | 2.39× (2.22–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.089 s | 108 | 23.2 | 1.59× (1.55–1.64) | 2.21× (2.12–2.31) | 2.18× (2.09–2.28) | 2.87× (2.79–2.97) | 2.76× (2.68–2.85) | 7.11× (6.90–7.33) | 7.40× (7.17–7.64) | 2.63× (2.55–2.72) | 2.76× (2.68–2.83) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 749 | 1.2 | 0.72× (0.70–0.74) | 3.09× (3.03–3.14) | 4.88× (4.59–5.19) | 3.15× (3.09–3.20) | 7.34× (7.22–7.46) | 2.41× (2.37–2.45) | 2.39× (2.35–2.43) | 1.06× (1.04–1.09) | 1.56× (1.53–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 690 | 1.3 | 0.65× (0.63–0.66) | 2.85× (2.78–2.92) | 5.56× (5.43–5.69) | 2.96× (2.89–3.03) | 6.78× (6.63–6.93) | 2.22× (2.16–2.27) | 2.20× (2.15–2.26) | 0.98× (0.94–1.01) | 1.43× (1.40–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 690 | 1.3 | 0.64× (0.63–0.65) | 2.85× (2.79–2.90) | 5.57× (5.46–5.67) | 2.93× (2.89–2.98) | 6.78× (6.66–6.91) | 2.21× (2.17–2.25) | 2.20× (2.15–2.25) | 0.98× (0.94–1.01) | 1.43× (1.40–1.46) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.075 s | 169 | 4.9 | 1.80× (1.78–1.81) | 3.66× (3.64–3.68) | 4.27× (4.21–4.34) | 4.29× (4.26–4.32) | 4.56× (4.52–4.60) | 4.82× (4.79–4.85) | 4.48× (4.45–4.51) | 1.41× (1.39–1.42) | 1.49× (1.48–1.50) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.050 s | 255 | 5.8 | 1.51× (1.47–1.54) | 3.13× (2.98–3.30) | 3.81× (3.66–3.97) | 4.24× (4.13–4.36) | 4.32× (4.21–4.44) | 7.29× (7.08–7.51) | 6.78× (6.57–7.00) | 2.13× (2.06–2.19) | 2.26× (2.19–2.32) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.044 s | 286 | 8.8 | 1.67× (1.62–1.72) | 2.41× (2.30–2.52) | 3.07× (2.95–3.20) | 3.53× (3.46–3.62) | 3.90× (3.81–4.00) | 8.17× (7.98–8.36) | 7.60× (7.41–7.78) | 2.38× (2.33–2.44) | 2.53× (2.47–2.59) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.88–2.99) | 6.42× (6.31–6.54) | 7.43× (7.29–7.57) | 7.61× (7.48–7.75) | 6.98× (6.84–7.13) | 7.78× (7.63–7.94) | 7.07× (6.96–7.18) | 2.20× (2.16–2.25) | 2.20× (2.16–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.84× (2.82–2.86) | 6.19× (6.13–6.25) | 7.32× (7.24–7.41) | 7.46× (7.43–7.49) | 6.60× (6.54–6.65) | 7.28× (7.24–7.32) | 6.61× (6.56–6.67) | 2.06× (2.05–2.07) | 2.06× (2.05–2.07) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.15× (3.10–3.19) | 4.78× (4.73–4.83) | 5.68× (5.58–5.78) | 5.60× (5.55–5.66) | 5.13× (5.07–5.18) | 4.33× (4.29–4.36) | 3.93× (3.91–3.96) | 1.23× (1.22–1.23) | 1.22× (1.21–1.23) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.69× (0.68–0.71) | 2.86× (2.79–2.93) | 5.05× (4.95–5.15) | 3.39× (3.32–3.45) | 6.14× (6.02–6.27) | 2.63× (2.58–2.69) | 2.60× (2.55–2.65) | 1.15× (1.13–1.17) | 0.99× (0.97–1.02) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.67× (0.64–0.70) | 2.70× (2.58–2.82) | 6.14× (5.90–6.39) | 3.27× (3.14–3.42) | 5.76× (5.52–6.01) | 2.49× (2.38–2.60) | 2.46× (2.35–2.56) | 1.09× (1.03–1.14) | 0.94× (0.90–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.66× (0.65–0.68) | 3.14× (3.07–3.21) | 6.26× (6.13–6.39) | 3.32× (3.25–3.40) | 5.96× (5.81–6.12) | 2.54× (2.49–2.60) | 2.51× (2.46–2.57) | 1.11× (1.08–1.15) | 0.96× (0.94–0.98) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.96× (2.94–2.97) | 3.05× (3.02–3.07) | 4.20× (4.18–4.23) | 4.04× (4.01–4.07) | 4.63× (4.59–4.66) | 3.72× (3.69–3.75) | 2.91× (2.89–2.93) | 2.10× (2.08–2.12) | 2.08× (2.06–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.65× (2.58–2.71) | 2.74× (2.67–2.81) | 3.87× (3.77–3.97) | 3.48× (3.39–3.57) | 4.03× (3.94–4.13) | 3.12× (3.04–3.21) | 2.44× (2.38–2.50) | 1.76× (1.71–1.81) | 1.75× (1.70–1.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.53× (2.44–2.62) | 1.86× (1.79–1.92) | 2.63× (2.53–2.73) | 2.19× (2.10–2.27) | 2.43× (2.34–2.53) | 1.58× (1.52–1.65) | 1.24× (1.19–1.28) | 0.89× (0.86–0.93) | 0.88× (0.85–0.92) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.32–1.37) | 2.48× (2.44–2.53) | 2.29× (2.24–2.33) | 2.35× (2.30–2.40) | 2.73× (2.68–2.78) | 3.09× (3.03–3.16) | 3.02× (2.95–3.08) | 1.20× (1.18–1.22) | 1.22× (1.19–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.26–1.32) | 2.40× (2.35–2.44) | 2.16× (2.12–2.20) | 2.78× (2.73–2.83) | 2.55× (2.51–2.60) | 2.83× (2.78–2.88) | 2.76× (2.71–2.81) | 1.10× (1.08–1.13) | 1.11× (1.09–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.60× (1.57–1.63) | 2.24× (2.18–2.30) | 1.92× (1.90–1.95) | 2.48× (2.43–2.53) | 2.39× (2.36–2.43) | 1.96× (1.92–2.00) | 1.91× (1.88–1.95) | 0.76× (0.74–0.79) | 0.77× (0.75–0.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.42× (1.39–1.45) | 2.80× (2.74–2.87) | 2.45× (2.39–2.52) | 2.53× (2.48–2.59) | 3.01× (2.81–3.23) | 3.73× (3.65–3.81) | 3.88× (3.81–3.96) | 1.39× (1.36–1.42) | 1.41× (1.38–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.30–1.37) | 2.65× (2.55–2.75) | 2.28× (2.22–2.35) | 3.12× (3.00–3.24) | 2.69× (2.62–2.77) | 3.36× (3.26–3.46) | 3.50× (3.38–3.61) | 1.25× (1.20–1.30) | 1.27× (1.23–1.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.73× (1.68–1.77) | 2.41× (2.37–2.46) | 1.90× (1.86–1.93) | 2.66× (2.59–2.73) | 2.44× (2.40–2.48) | 2.25× (2.22–2.28) | 2.34× (2.30–2.38) | 0.84× (0.81–0.86) | 0.85× (0.84–0.86) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.74–0.77) | 2.45× (2.41–2.49) | 4.11× (4.05–4.17) | 2.89× (2.85–2.94) | 6.59× (6.49–6.70) | 2.24× (2.21–2.27) | 2.23× (2.20–2.27) | 1.06× (1.04–1.08) | 0.91× (0.90–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.69× (0.68–0.71) | 2.26× (2.22–2.29) | 4.68× (4.58–4.77) | 2.75× (2.70–2.79) | 6.04× (5.94–6.16) | 2.04× (2.00–2.08) | 2.03× (2.00–2.07) | 0.96× (0.94–0.99) | 0.83× (0.81–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.67× (0.66–0.68) | 2.59× (2.53–2.65) | 4.82× (4.72–4.92) | 2.80× (2.76–2.84) | 6.20× (6.09–6.31) | 2.09× (2.05–2.12) | 2.08× (2.03–2.12) | 0.99× (0.95–1.02) | 0.85× (0.84–0.86) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.78× (1.76–1.79) | 3.54× (3.52–3.57) | 4.13× (4.08–4.19) | 4.21× (4.19–4.24) | 4.48× (4.44–4.51) | 4.73× (4.70–4.76) | 4.40× (4.37–4.43) | 1.40× (1.39–1.41) | 1.39× (1.38–1.40) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.61× (1.59–1.64) | 3.18× (3.10–3.25) | 3.69× (3.65–3.72) | 4.03× (4.00–4.05) | 3.87× (3.83–3.91) | 3.98× (3.95–4.01) | 3.70× (3.67–3.73) | 1.17× (1.16–1.18) | 1.17× (1.16–1.18) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.98× (1.93–2.03) | 2.76× (2.71–2.82) | 3.12× (3.04–3.19) | 3.39× (3.29–3.49) | 3.27× (3.22–3.33) | 2.60× (2.55–2.64) | 2.42× (2.38–2.46) | 0.77× (0.75–0.78) | 0.76× (0.75–0.78) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.08× (11.06–11.10) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.33× (0.32–0.35) | 0.45× (0.38–0.53) | 0.68× (0.65–0.71) | 0.57× (0.55–0.59) | 5.45× (5.24–5.67) | 0.13× (0.12–0.13) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.35) | 0.52× (0.45–0.61) | 0.58× (0.56–0.61) | 0.52× (0.49–0.55) | 3.44× (3.29–3.60) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.28× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.44) | 1.71× (1.70–1.71) | 0.66× (0.65–0.66) | 8.05× (8.00–8.09) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.64–0.68) | 0.64× (0.58–0.71) | 1.59× (1.54–1.63) | 0.79× (0.77–0.82) | 5.77× (5.62–5.92) | 0.46× (0.45–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.66) | 0.72× (0.69–0.75) | 1.14× (1.11–1.17) | 0.72× (0.70–0.73) | 3.44× (3.37–3.51) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.22× (5.21–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.40× (0.36–0.44) | 0.91× (0.88–0.94) | 0.72× (0.69–0.75) | 3.15× (3.06–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.47) | 0.86× (0.84–0.88) | 0.80× (0.77–0.83) | 2.51× (2.47–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.50) | 0.36× (0.32–0.39) | 0.86× (0.83–0.89) | 0.75× (0.72–0.77) | 2.97× (2.87–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.47× (0.45–0.50) | 0.81× (0.80–0.83) | 0.84× (0.81–0.87) | 2.36× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.38× (0.38–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.26–5.32) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.40× (6.38–6.43) | 0.39× (0.39–0.40) | 0.43× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 40 MB | 0.42× (0.42–0.43) | 0.36× (0.33–0.40) | 0.97× (0.95–0.99) | 0.65× (0.64–0.66) | 3.10× (3.06–3.14) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.49× (0.44–0.55) | 0.90× (0.88–0.93) | 0.73× (0.71–0.76) | 2.32× (2.26–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 387 | 2.60 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 686 | 2.80 | 1.07× (1.05–1.09) | 89% | 1.03× (1.02–1.04) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 788 | 4.70 | 1.80× (1.77–1.83) | 51% | 1.34× (1.33–1.34) | 1.93× (1.90–1.96) |

**Best:** mibz2 at 4 threads (788 MB/s); lbzip2 at 4 threads (175 MB/s); pbz2 at 4 threads (244 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.16, lbzip2 1t 0.04, lbzip2 2t 0.27, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.32. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.77 | 0.03 | 0.09 | 0.266 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.29 | 10.2% |
| MTF + RLE2 | 0.41 | 14.4% |
| T-vector build | 0.40 | 14.1% |
| walk (the one each build uses) | 1.63 | 57.6% |
| of which: segment assembly | 0.08 | 2.7% |
| RLE1 | 0.07 | 2.4% |
| CRC | 0.04 | 1.3% |
| **total** | 2.83 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.495 | 1.272 | (in its total) | 2.57× (2.56–2.58) |  |
| walk | 1.932 | 3.037 | (in its total) | 1.57× (1.57–1.58) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.094 | 0.089 | (in its total) | 0.95× (0.94–0.95) |  |
| inverse BWT, total | 2.520 | 4.399 | 2.592 | 1.75× (1.74–1.75) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h19m ± 9m10s |
| 4 threads | 38m54s ± 5m29s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m56s ± 3m58s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 27m56s ± 3m58s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.66 ns a byte of output at 4 threads (CPU, summed over them), 2.11 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,506,264 | 12,614,346 | \`520a957f38391906…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
