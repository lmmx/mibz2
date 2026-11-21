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

The latest record: [`2956c18`](<https://github.com/lmmx/mibz2/commit/2956c184a8f5f3fa085ddc7baca76d51a7a2a7ed>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44856109015>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.498 s | 421 | 7.4 | 3.18× (3.12–3.25) | 7.07× (6.92–7.22) | 7.53× (7.37–7.69) | 8.48× (8.31–8.65) | 7.57× (7.42–7.73) | 8.08× (7.91–8.25) | 7.83× (7.69–7.97) | 2.46× (2.41–2.51) | 2.27× (2.21–2.32) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.293 s | 715 | 8.2 | 3.08× (2.96–3.20) | 6.61× (6.40–6.83) | 7.84× (7.59–8.11) | 7.92× (7.71–8.14) | 6.98× (6.81–7.17) | 13.97× (13.66–14.29) | 13.54× (13.23–13.85) | 4.25× (4.15–4.36) | 3.92× (3.83–4.01) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.239 s | 878 | 12.9 | 3.30× (3.25–3.36) | 5.80× (5.61–6.01) | 6.54× (6.39–6.70) | 6.55× (6.44–6.65) | 6.37× (6.27–6.47) | 17.10× (16.89–17.32) | 16.57× (16.39–16.75) | 5.21× (5.15–5.26) | 4.80× (4.73–4.87) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 968 | 3.5 | 0.58× (0.55–0.62) | 3.38× (3.24–3.53) | 4.35× (4.25–4.45) | 3.79× (3.59–4.00) | 6.16× (5.92–6.40) | 2.77× (2.65–2.89) | 2.77× (2.65–2.89) | 1.13× (1.08–1.19) | 0.98× (0.94–1.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 950 | 3.6 | 0.55× (0.54–0.57) | 2.92× (2.89–2.96) | 5.78× (5.70–5.86) | 3.79× (3.72–3.87) | 6.17× (6.10–6.24) | 2.77× (2.74–2.81) | 2.77× (2.74–2.80) | 1.13× (1.10–1.17) | 0.98× (0.97–0.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 939 | 3.6 | 0.50× (0.41–0.60) | 2.66× (2.20–3.22) | 5.25× (4.34–6.36) | 3.42× (2.83–4.13) | 5.64× (4.68–6.80) | 2.52× (2.08–3.05) | 2.52× (2.08–3.05) | 1.03× (0.84–1.26) | 0.89× (0.74–1.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 522 | 6.2 | 2.86× (2.83–2.88) | 3.51× (3.48–3.54) | 4.01× (3.98–4.04) | 4.47× (4.43–4.52) | 4.85× (4.81–4.89) | 3.68× (3.63–3.72) | 2.99× (2.96–3.02) | 2.13× (2.09–2.17) | 2.16× (2.14–2.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 827 | 7.5 | 2.62× (2.35–2.92) | 3.07× (2.97–3.18) | 4.42× (4.26–4.58) | 3.99× (3.90–4.08) | 5.45× (5.35–5.55) | 5.84× (5.75–5.94) | 4.76× (4.68–4.83) | 3.39× (3.32–3.46) | 3.43× (3.37–3.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.024 s | 802 | 13.3 | 2.10× (2.01–2.19) | 2.11× (2.00–2.22) | 3.10× (2.91–3.30) | 2.69× (2.46–2.95) | 4.29× (4.12–4.46) | 5.61× (5.42–5.81) | 4.57× (4.40–4.74) | 3.26× (3.15–3.36) | 3.30× (3.17–3.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.150 s | 54 | 58.7 | 1.35× (1.31–1.39) | 2.63× (2.60–2.66) | 2.33× (2.28–2.37) | 2.37× (2.34–2.40) | 2.73× (2.68–2.78) | 3.05× (3.01–3.10) | 3.21× (3.16–3.26) | 1.28× (1.26–1.29) | 1.30× (1.26–1.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.089 s | 90 | 64.7 | 1.25× (1.17–1.34) | 2.49× (2.31–2.68) | 2.32× (2.17–2.48) | 3.06× (2.89–3.25) | 2.83× (2.62–3.06) | 5.03× (4.70–5.38) | 5.28× (4.93–5.66) | 2.10× (1.97–2.24) | 2.14× (2.04–2.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.073 s | 110 | 86.0 | 1.39× (1.36–1.43) | 2.40× (2.35–2.46) | 2.32× (2.25–2.40) | 2.92× (2.87–2.97) | 2.92× (2.88–2.97) | 6.31× (6.19–6.42) | 6.63× (6.53–6.73) | 2.64× (2.58–2.69) | 2.68× (2.59–2.77) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.166 s | 58 | 54.2 | 1.38× (1.36–1.39) | 3.06× (3.03–3.10) | 2.52× (2.48–2.57) | 2.62× (2.58–2.66) | 2.97× (2.92–3.01) | 3.71× (3.67–3.76) | 4.02× (3.96–4.08) | 1.49× (1.48–1.50) | 1.53× (1.52–1.54) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.096 s | 100 | 59.7 | 1.33× (1.27–1.40) | 2.82× (2.73–2.92) | 2.50× (2.41–2.59) | 3.48× (3.36–3.61) | 3.02× (2.91–3.14) | 6.31× (6.09–6.53) | 6.83× (6.59–7.08) | 2.52× (2.43–2.62) | 2.59× (2.50–2.69) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 120 | 83.1 | 1.46× (1.43–1.50) | 2.71× (2.61–2.81) | 2.43× (2.36–2.50) | 3.28× (3.19–3.38) | 2.96× (2.88–3.03) | 7.59× (7.38–7.80) | 8.21× (7.98–8.46) | 3.04× (2.95–3.12) | 3.12× (3.04–3.19) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 714 | 4.9 | 0.62× (0.61–0.63) | 3.00× (2.84–3.16) | 4.31× (4.25–4.37) | 3.23× (3.15–3.32) | 6.73× (6.63–6.84) | 2.39× (2.35–2.44) | 2.40× (2.37–2.43) | 1.04× (1.03–1.06) | 1.48× (1.46–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 5.1 | 0.53× (0.53–0.53) | 2.87× (2.84–2.90) | 4.94× (4.88–5.00) | 3.19× (3.14–3.24) | 6.51× (6.43–6.59) | 2.34× (2.29–2.38) | 2.35× (2.32–2.37) | 1.02× (1.01–1.04) | 1.44× (1.43–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 690 | 5.0 | 0.53× (0.53–0.54) | 2.84× (2.81–2.86) | 4.88× (4.84–4.92) | 3.12× (3.08–3.15) | 6.45× (6.37–6.53) | 2.31× (2.27–2.34) | 2.32× (2.30–2.33) | 1.01× (0.99–1.03) | 1.42× (1.41–1.44) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 0.077 s | 174 | 18.3 | 1.81× (1.80–1.83) | 3.97× (3.94–4.00) | 4.41× (4.36–4.46) | 4.58× (4.50–4.65) | 4.64× (4.60–4.69) | 4.92× (4.86–4.97) | 4.97× (4.94–5.01) | 1.50× (1.48–1.51) | 1.45× (1.44–1.46) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 0.050 s | 269 | 21.5 | 1.50× (1.35–1.67) | 3.45× (3.12–3.81) | 4.00× (3.58–4.47) | 4.21× (3.82–4.64) | 4.17× (3.77–4.62) | 7.12× (6.48–7.83) | 7.20× (6.51–7.97) | 2.17× (1.96–2.39) | 2.10× (1.89–2.33) | yes |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 0.043 s | 314 | 32.2 | 1.62× (1.50–1.75) | 2.91× (2.75–3.08) | 3.63× (3.44–3.85) | 3.87× (3.64–4.12) | 4.19× (3.95–4.44) | 8.58× (8.07–9.12) | 8.68× (8.15–9.23) | 2.61× (2.45–2.78) | 2.53× (2.37–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.17× (3.11–3.24) | 7.04× (6.89–7.19) | 7.49× (7.33–7.65) | 8.45× (8.29–8.62) | 7.55× (7.40–7.71) | 8.06× (7.89–8.23) | 7.80× (7.67–7.94) | 2.46× (2.41–2.50) | 2.26× (2.20–2.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.01× (2.99–3.03) | 6.82× (6.71–6.93) | 7.44× (7.38–7.50) | 8.28× (8.23–8.34) | 7.01× (6.98–7.04) | 7.40× (7.35–7.44) | 7.17× (7.14–7.20) | 2.25× (2.24–2.27) | 2.07× (2.06–2.09) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.24× (3.20–3.28) | 6.19× (6.06–6.33) | 6.38× (6.29–6.48) | 7.04× (6.93–7.14) | 6.21× (6.15–6.27) | 4.69× (4.62–4.75) | 4.54× (4.50–4.58) | 1.43× (1.41–1.45) | 1.31× (1.30–1.33) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.58–0.65) | 2.86× (2.73–2.99) | 3.96× (3.86–4.07) | 3.59× (3.40–3.79) | 5.78× (5.55–6.02) | 2.64× (2.52–2.76) | 2.63× (2.52–2.76) | 1.12× (1.07–1.18) | 0.90× (0.86–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.61) | 2.82× (2.78–2.86) | 5.14× (5.10–5.19) | 3.56× (3.50–3.62) | 5.72× (5.65–5.78) | 2.61× (2.58–2.64) | 2.60× (2.57–2.63) | 1.11× (1.07–1.14) | 0.89× (0.88–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.44–0.64) | 3.00× (2.50–3.59) | 4.78× (3.98–5.75) | 3.27× (2.72–3.93) | 5.31× (4.42–6.37) | 2.41× (2.00–2.90) | 2.41× (2.00–2.90) | 1.02× (0.84–1.24) | 0.82× (0.69–0.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.78× (2.76–2.80) | 3.18× (3.13–3.23) | 3.69× (3.66–3.72) | 4.33× (4.28–4.37) | 4.69× (4.65–4.73) | 3.56× (3.52–3.60) | 2.91× (2.88–2.94) | 2.08× (2.05–2.12) | 1.99× (1.97–2.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.50× (2.47–2.53) | 2.86× (2.82–2.89) | 3.38× (3.35–3.40) | 3.70× (3.67–3.74) | 4.05× (4.01–4.09) | 2.98× (2.95–3.01) | 2.43× (2.41–2.45) | 1.74× (1.72–1.77) | 1.67× (1.64–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.25× (2.19–2.30) | 1.92× (1.86–1.99) | 2.34× (2.28–2.39) | 2.30× (2.24–2.36) | 2.43× (2.37–2.49) | 1.65× (1.60–1.69) | 1.34× (1.31–1.37) | 0.96× (0.94–0.99) | 0.92× (0.90–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.30–1.38) | 2.57× (2.55–2.60) | 2.28× (2.24–2.31) | 2.36× (2.33–2.38) | 2.71× (2.66–2.76) | 3.03× (2.98–3.07) | 3.18× (3.13–3.23) | 1.27× (1.26–1.29) | 1.26× (1.23–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.29) | 2.49× (2.44–2.54) | 2.16× (2.12–2.20) | 2.89× (2.83–2.95) | 2.55× (2.47–2.63) | 2.76× (2.70–2.82) | 2.90× (2.84–2.96) | 1.16× (1.13–1.19) | 1.15× (1.12–1.18) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.51–1.59) | 2.65× (2.56–2.74) | 2.18× (2.13–2.24) | 2.88× (2.82–2.94) | 2.58× (2.53–2.63) | 2.08× (2.04–2.12) | 2.19× (2.14–2.24) | 0.87× (0.85–0.90) | 0.87× (0.84–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.37× (1.36–1.39) | 3.01× (2.97–3.04) | 2.48× (2.44–2.52) | 2.60× (2.56–2.64) | 2.95× (2.90–2.99) | 3.69× (3.65–3.73) | 3.99× (3.93–4.05) | 1.48× (1.47–1.49) | 1.49× (1.48–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.27–1.34) | 2.82× (2.73–2.92) | 2.33× (2.25–2.40) | 3.24× (3.14–3.35) | 2.72× (2.64–2.81) | 3.30× (3.20–3.41) | 3.58× (3.47–3.69) | 1.33× (1.29–1.37) | 1.33× (1.29–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.48–1.62) | 2.91× (2.83–2.99) | 2.16× (2.10–2.23) | 2.99× (2.90–3.08) | 2.62× (2.54–2.70) | 2.36× (2.28–2.45) | 2.56× (2.47–2.65) | 0.95× (0.92–0.98) | 0.95× (0.92–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.40× (2.31–2.50) | 3.45× (3.39–3.50) | 2.99× (2.91–3.07) | 6.11× (6.01–6.22) | 2.24× (2.20–2.28) | 2.25× (2.22–2.28) | 1.04× (1.02–1.05) | 0.82× (0.80–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.28× (2.25–2.30) | 4.20× (4.16–4.25) | 2.96× (2.93–3.00) | 5.83× (5.76–5.90) | 2.15× (2.11–2.20) | 2.16× (2.13–2.18) | 1.00× (0.98–1.01) | 0.79× (0.77–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.58) | 2.64× (2.58–2.70) | 4.22× (4.17–4.26) | 2.96× (2.93–3.00) | 5.90× (5.84–5.97) | 2.17× (2.14–2.20) | 2.18× (2.16–2.20) | 1.00× (0.99–1.02) | 0.80× (0.78–0.82) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 1.79× (1.78–1.81) | 3.82× (3.78–3.86) | 4.28× (4.25–4.31) | 4.50× (4.43–4.57) | 4.57× (4.52–4.61) | 4.83× (4.78–4.89) | 4.89× (4.86–4.92) | 1.49× (1.47–1.50) | 1.39× (1.38–1.40) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 1.61× (1.59–1.62) | 3.44× (3.35–3.52) | 3.97× (3.90–4.04) | 4.35× (4.31–4.39) | 3.95× (3.91–3.99) | 4.08× (4.05–4.12) | 4.13× (4.08–4.17) | 1.26× (1.25–1.26) | 1.18× (1.16–1.19) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 1.79× (1.71–1.88) | 3.17× (3.07–3.27) | 3.54× (3.42–3.66) | 3.87× (3.74–4.01) | 3.52× (3.43–3.61) | 2.75× (2.67–2.83) | 2.78× (2.73–2.84) | 0.85× (0.83–0.87) | 0.79× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.26–0.26) | 0.97× (0.96–0.97) | 0.77× (0.77–0.78) | 10.94× (10.90–10.98) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.46× (0.37–0.58) | 0.70× (0.67–0.72) | 0.59× (0.57–0.61) | 5.61× (5.41–5.83) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.35× (0.34–0.37) | 0.49× (0.41–0.58) | 0.62× (0.61–0.64) | 0.54× (0.52–0.57) | 3.67× (3.53–3.81) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.16× (0.15–0.17) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.42× (0.40–0.44) | 1.69× (1.69–1.70) | 0.66× (0.65–0.67) | 8.00× (7.96–8.04) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.68× (0.62–0.74) | 1.58× (1.54–1.63) | 0.79× (0.77–0.81) | 5.78× (5.63–5.93) | 0.47× (0.46–0.48) | 0.49× (0.48–0.51) | 0.54× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.69) | 0.72× (0.67–0.76) | 1.16× (1.15–1.18) | 0.71× (0.70–0.73) | 3.48× (3.44–3.52) | 0.27× (0.27–0.28) | 0.29× (0.28–0.29) | 0.31× (0.31–0.32) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.16× (5.14–5.18) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.39× (0.37–0.42) | 0.91× (0.89–0.94) | 0.70× (0.66–0.74) | 3.17× (3.08–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.79× (0.76–0.82) | 2.51× (2.45–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.83× (0.82–0.83) | 5.17× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.51) | 0.35× (0.31–0.40) | 0.85× (0.82–0.88) | 0.69× (0.67–0.71) | 2.92× (2.82–3.03) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.61) | 0.46× (0.44–0.49) | 0.80× (0.79–0.81) | 0.80× (0.78–0.83) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.76× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.39) | 0.67× (0.66–0.67) | 6.33× (6.31–6.34) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 2 | 38 MB | 0.46× (0.44–0.47) | 0.39× (0.36–0.43) | 1.04× (1.00–1.07) | 0.68× (0.64–0.72) | 3.34× (3.26–3.43) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.63 MB → 13.3 MB, 5.1×) | 4 | 57 MB | 0.50× (0.49–0.51) | 0.48× (0.44–0.53) | 0.91× (0.88–0.94) | 0.71× (0.67–0.75) | 2.33× (2.25–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 421 | 2.38 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 715 | 2.63 | 1.09× (1.07–1.11) | 86% | 1.05× (1.04–1.07) | 1.03× (1.02–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 878 | 4.15 | 1.72× (1.68–1.76) | 53% | 1.51× (1.48–1.55) | 1.75× (1.73–1.78) |

**Best:** mibz2 at 4 threads (878 MB/s); lbzip2 at 4 threads (153 MB/s); pbz2 at 4 threads (267 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.19, lbzip2 1t 0.05, lbzip2 2t 0.28, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.36. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.66 | 0.02 | 0.09 | 0.239 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3106 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.16 | 11.3% |
| MTF + RLE2 | 1.64 | 15.9% |
| T-vector build | 1.60 | 15.6% |
| walk (the one each build uses) | 5.49 | 53.4% |
| of which: segment assembly | 0.29 | 2.8% |
| RLE1 | 0.26 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.29 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.500 | 1.308 | (in its total) | 2.61× (2.61–2.62) |  |
| walk | 1.656 | 3.184 | (in its total) | 1.92× (1.92–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.092 | 0.085 | (in its total) | 0.92× (0.91–0.93) |  |
| inverse BWT, total | 2.248 | 4.577 | 2.396 | 2.03× (2.03–2.04) | 1.07× (1.06–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h13m ± 8m29s |
| 4 threads | 34m39s ± 4m53s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m14s ± 3m26s |
| the scanner alone (its serial part) | 14m07s ± 30.2s |
| the workers alone, over 4 | 24m13s ± 3m26s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 3.18 ns a byte of output at 4 threads (CPU, summed over them), 1.94 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,627,246 | 13,345,437 | \`3e431388ee58319e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3105.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
