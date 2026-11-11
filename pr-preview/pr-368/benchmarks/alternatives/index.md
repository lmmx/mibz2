# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-368/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`57975de`](<https://github.com/lmmx/mibz2/commit/57975de5d16143a951e8cb5610d6442e0ac06430>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42870424701>)

Intel(R) Xeon(R) 6973P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.593 s | 354 | 10.4 | 2.44× (2.39–2.50) | 5.49× (5.39–5.60) | 5.86× (5.76–5.97) | 6.47× (6.37–6.57) | 5.88× (5.77–6.00) | 6.14× (6.03–6.26) | 5.98× (5.88–6.07) | 1.82× (1.79–1.85) | 1.71× (1.67–1.74) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.332 s | 631 | 11.1 | 2.45× (2.26–2.66) | 5.01× (4.64–5.40) | 6.25× (5.79–6.74) | 6.12× (5.71–6.56) | 5.56× (5.19–5.97) | 10.78× (10.06–11.56) | 10.49× (9.75–11.28) | 3.20× (2.98–3.43) | 3.00× (2.80–3.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.276 s | 759 | 18.4 | 2.47× (2.45–2.49) | 4.47× (4.38–4.57) | 5.41× (5.32–5.52) | 5.26× (5.21–5.31) | 5.13× (5.07–5.20) | 13.25× (13.06–13.44) | 12.89× (12.68–13.10) | 3.93× (3.88–3.99) | 3.69× (3.62–3.75) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,058 | 3.8 | 0.65× (0.58–0.72) | 3.23× (3.14–3.32) | 4.36× (4.19–4.53) | 3.58× (3.44–3.72) | 6.39× (6.15–6.63) | 2.63× (2.57–2.68) | 2.68× (2.58–2.78) | 1.21× (1.19–1.23) | 1.09× (1.06–1.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,020 | 4.0 | 0.52× (0.50–0.54) | 2.56× (2.45–2.68) | 5.70× (5.46–5.94) | 3.36× (3.19–3.53) | 5.95× (5.71–6.19) | 2.48× (2.37–2.60) | 2.53× (2.37–2.69) | 1.14× (1.09–1.20) | 1.03× (0.98–1.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 983 | 4.1 | 0.53× (0.51–0.54) | 2.83× (2.62–3.05) | 5.74× (5.53–5.96) | 3.35× (3.25–3.47) | 5.98× (5.80–6.16) | 2.50× (2.44–2.56) | 2.55× (2.45–2.65) | 1.15× (1.11–1.19) | 1.04× (1.01–1.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 539 | 7.2 | 2.55× (2.41–2.71) | 3.02× (2.83–3.22) | 3.54× (3.33–3.77) | 3.90× (3.65–4.17) | 4.41× (4.15–4.69) | 2.90× (2.71–3.10) | 2.67× (2.49–2.86) | 1.82× (1.69–1.95) | 1.98× (1.81–2.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 838 | 8.8 | 1.94× (1.60–2.37) | 2.61× (2.27–3.00) | 3.46× (2.90–4.13) | 3.06× (2.46–3.79) | 4.43× (3.60–5.47) | 3.98× (3.21–4.94) | 3.67× (2.99–4.49) | 2.50× (2.02–3.09) | 2.72× (2.18–3.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 751 | 16.4 | 1.59× (1.49–1.69) | 1.58× (1.47–1.69) | 2.70× (2.47–2.95) | 2.08× (1.94–2.22) | 3.70× (3.46–3.95) | 4.01× (3.72–4.32) | 3.69× (3.46–3.95) | 2.52× (2.34–2.71) | 2.74× (2.50–2.99) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.127 s | 63 | 59.0 | 1.36× (1.34–1.38) | 2.91× (2.83–3.00) | 2.69× (2.66–2.73) | 2.67× (2.60–2.74) | 2.96× (2.91–3.01) | 3.27× (3.22–3.32) | 3.31× (3.25–3.37) | 1.26× (1.24–1.29) | 1.28× (1.25–1.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.078 s | 102 | 65.3 | 1.33× (1.15–1.54) | 2.75× (2.42–3.11) | 2.63× (2.49–2.79) | 3.32× (3.11–3.54) | 3.03× (2.92–3.15) | 5.28× (5.02–5.54) | 5.34× (5.09–5.61) | 2.04× (1.95–2.14) | 2.06× (1.96–2.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.062 s | 128 | 89.1 | 1.34× (1.30–1.39) | 2.61× (2.48–2.76) | 2.62× (2.45–2.79) | 3.13× (3.01–3.26) | 3.13× (3.02–3.25) | 6.53× (6.29–6.78) | 6.61× (6.39–6.84) | 2.53× (2.42–2.64) | 2.55× (2.43–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.140 s | 68 | 54.5 | 1.38× (1.34–1.42) | 3.20× (3.08–3.33) | 2.79× (2.71–2.88) | 2.80× (2.72–2.89) | 3.17× (3.05–3.29) | 3.87× (3.74–3.99) | 4.06× (3.94–4.19) | 1.46× (1.40–1.53) | 1.49× (1.43–1.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.082 s | 117 | 60.0 | 1.31× (1.21–1.41) | 3.05× (2.90–3.22) | 2.76× (2.62–2.90) | 3.81× (3.46–4.18) | 3.14× (2.97–3.33) | 6.48× (6.12–6.87) | 6.81× (6.44–7.20) | 2.45× (2.31–2.61) | 2.50× (2.34–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.072 s | 133 | 87.9 | 1.37× (1.33–1.41) | 2.76× (2.67–2.85) | 2.63× (2.53–2.73) | 3.50× (3.37–3.65) | 3.04× (2.96–3.12) | 7.58× (7.40–7.77) | 7.97× (7.81–8.13) | 2.87× (2.78–2.96) | 2.92× (2.81–3.04) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 790 | 5.3 | 0.66× (0.65–0.67) | 3.11× (2.88–3.35) | 4.02× (3.86–4.18) | 2.98× (2.81–3.16) | 6.74× (6.45–7.06) | 2.24× (2.12–2.37) | 2.29× (2.19–2.39) | 1.08× (1.05–1.11) | 0.83× (0.80–0.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 6.0 | 0.48× (0.43–0.53) | 2.10× (1.93–2.29) | 4.87× (4.46–5.30) | 2.59× (2.37–2.83) | 5.87× (5.32–6.49) | 1.94× (1.78–2.12) | 1.99× (1.81–2.18) | 0.93× (0.84–1.03) | 0.72× (0.66–0.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 707 | 5.9 | 0.49× (0.47–0.52) | 2.70× (2.38–3.07) | 4.96× (4.73–5.21) | 2.73× (2.60–2.87) | 6.11× (5.87–6.36) | 2.04× (1.92–2.18) | 2.09× (1.98–2.20) | 0.98× (0.95–1.01) | 0.76× (0.72–0.80) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.067 s | 193 | 19.6 | 1.79× (1.73–1.85) | 4.02× (3.82–4.22) | 4.49× (4.35–4.62) | 4.44× (4.30–4.58) | 4.71× (4.51–4.91) | 4.83× (4.67–5.00) | 4.73× (4.58–4.88) | 1.50× (1.45–1.56) | 1.40× (1.35–1.45) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.043 s | 296 | 23.4 | 1.72× (1.47–2.02) | 3.47× (3.13–3.84) | 4.37× (4.15–4.59) | 4.72× (4.23–5.27) | 4.53× (4.39–4.68) | 7.42× (7.18–7.67) | 7.25× (7.02–7.50) | 2.31× (2.18–2.45) | 2.15× (2.04–2.26) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.041 s | 311 | 35.8 | 1.43× (1.37–1.49) | 2.62× (2.52–2.73) | 3.64× (3.34–3.97) | 3.69× (3.53–3.85) | 4.11× (3.98–4.23) | 7.93× (7.66–8.21) | 7.76× (7.52–8.00) | 2.47× (2.33–2.62) | 2.30× (2.18–2.42) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.44× (2.38–2.49) | 5.47× (5.37–5.58) | 5.84× (5.74–5.95) | 6.45× (6.35–6.55) | 5.87× (5.75–5.98) | 6.13× (6.01–6.24) | 5.96× (5.87–6.05) | 1.82× (1.79–1.85) | 1.70× (1.66–1.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.37× (2.31–2.43) | 5.26× (5.10–5.42) | 5.93× (5.77–6.09) | 6.44× (6.30–6.59) | 5.61× (5.47–5.75) | 5.74× (5.59–5.88) | 5.58× (5.43–5.73) | 1.70× (1.66–1.75) | 1.59× (1.55–1.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.32× (2.29–2.35) | 4.59× (4.51–4.67) | 5.03× (4.96–5.11) | 5.42× (5.39–5.46) | 4.77× (4.73–4.80) | 3.48× (3.44–3.52) | 3.39× (3.34–3.43) | 1.03× (1.02–1.05) | 0.97× (0.95–0.98) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.62–0.75) | 2.68× (2.62–2.75) | 3.82× (3.69–3.96) | 3.35× (3.23–3.47) | 5.91× (5.69–6.13) | 2.48× (2.43–2.53) | 2.53× (2.44–2.62) | 1.19× (1.17–1.21) | 0.82× (0.80–0.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.54–0.59) | 2.56× (2.44–2.69) | 4.90× (4.70–5.12) | 3.14× (2.98–3.30) | 5.47× (5.24–5.70) | 2.32× (2.21–2.44) | 2.36× (2.22–2.52) | 1.11× (1.06–1.17) | 0.77× (0.73–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.04× (2.93–3.14) | 4.98× (4.82–5.16) | 3.19× (3.08–3.29) | 5.55× (5.39–5.73) | 2.37× (2.32–2.43) | 2.42× (2.33–2.51) | 1.14× (1.10–1.17) | 0.79× (0.77–0.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.47× (2.33–2.62) | 2.77× (2.60–2.95) | 3.30× (3.11–3.51) | 3.76× (3.52–4.00) | 4.24× (3.99–4.50) | 2.80× (2.62–2.99) | 2.58× (2.41–2.76) | 1.77× (1.66–1.90) | 1.82× (1.69–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.16× (2.09–2.24) | 2.43× (2.34–2.51) | 2.95× (2.82–3.08) | 3.15× (3.06–3.25) | 3.62× (3.50–3.75) | 2.27× (2.17–2.38) | 2.10× (2.01–2.18) | 1.44× (1.38–1.51) | 1.48× (1.40–1.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.81× (1.71–1.91) | 1.64× (1.53–1.76) | 2.07× (1.92–2.25) | 2.04× (1.88–2.20) | 2.17× (2.04–2.30) | 1.25× (1.16–1.35) | 1.15× (1.08–1.23) | 0.79× (0.75–0.84) | 0.81× (0.75–0.88) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.34–1.37) | 2.84× (2.75–2.92) | 2.64× (2.59–2.68) | 2.65× (2.58–2.72) | 2.93× (2.88–2.99) | 3.24× (3.19–3.29) | 3.27× (3.22–3.33) | 1.26× (1.23–1.29) | 1.23× (1.22–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.24–1.31) | 2.67× (2.58–2.77) | 2.44× (2.37–2.52) | 3.08× (3.01–3.16) | 2.73× (2.65–2.81) | 2.90× (2.83–2.98) | 2.94× (2.87–3.01) | 1.13× (1.10–1.16) | 1.11× (1.08–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.36–1.52) | 2.66× (2.53–2.79) | 2.35× (2.29–2.41) | 3.05× (2.98–3.12) | 2.69× (2.63–2.75) | 2.13× (2.06–2.19) | 2.15× (2.10–2.21) | 0.83× (0.80–0.85) | 0.81× (0.78–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.37× (1.34–1.41) | 3.13× (3.01–3.26) | 2.74× (2.67–2.82) | 2.78× (2.69–2.87) | 3.14× (3.02–3.26) | 3.83× (3.71–3.96) | 4.02× (3.90–4.15) | 1.46× (1.40–1.52) | 1.45× (1.39–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.27–1.41) | 2.97× (2.91–3.04) | 2.60× (2.55–2.65) | 3.49× (3.36–3.63) | 2.89× (2.83–2.95) | 3.48× (3.41–3.57) | 3.66× (3.58–3.74) | 1.33× (1.30–1.35) | 1.32× (1.27–1.37) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.52× (1.48–1.56) | 3.00× (2.93–3.07) | 2.33× (2.28–2.38) | 3.22× (3.12–3.33) | 2.73× (2.67–2.80) | 2.42× (2.39–2.45) | 2.54× (2.51–2.57) | 0.92× (0.90–0.94) | 0.91× (0.89–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.32× (2.21–2.44) | 3.27× (3.13–3.41) | 2.73× (2.58–2.88) | 6.01× (5.77–6.26) | 2.08× (1.98–2.18) | 2.12× (2.04–2.21) | 1.06× (1.04–1.09) | 0.75× (0.72–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.54× (0.49–0.59) | 1.93× (1.79–2.08) | 3.82× (3.49–4.18) | 2.41× (2.22–2.62) | 5.23× (4.79–5.71) | 1.79× (1.66–1.93) | 1.83× (1.69–1.99) | 0.92× (0.84–1.01) | 0.65× (0.60–0.70) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.52–0.57) | 2.50× (2.37–2.65) | 4.03× (3.84–4.23) | 2.58× (2.47–2.70) | 5.54× (5.32–5.76) | 1.92× (1.81–2.03) | 1.96× (1.87–2.05) | 0.98× (0.95–1.01) | 0.69× (0.66–0.73) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.77× (1.72–1.82) | 3.87× (3.70–4.05) | 4.34× (4.23–4.46) | 4.35× (4.22–4.48) | 4.61× (4.43–4.80) | 4.73× (4.58–4.88) | 4.63× (4.48–4.77) | 1.49× (1.44–1.54) | 1.34× (1.31–1.36) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.62× (1.60–1.64) | 3.43× (3.36–3.51) | 4.07× (4.01–4.13) | 4.32× (4.21–4.43) | 4.05× (3.95–4.15) | 4.03× (3.94–4.12) | 3.94× (3.86–4.02) | 1.27× (1.22–1.33) | 1.14× (1.11–1.17) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.71× (1.61–1.81) | 3.04× (2.94–3.14) | 3.50× (3.24–3.79) | 3.69× (3.52–3.87) | 3.49× (3.42–3.57) | 2.58× (2.49–2.68) | 2.52× (2.42–2.63) | 0.81× (0.77–0.87) | 0.73× (0.70–0.76) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.28× (0.27–0.29) | 1.05× (1.05–1.05) | 0.84× (0.84–0.84) | 11.90× (11.89–11.91) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 51 MB | 0.37× (0.36–0.38) | 0.43× (0.35–0.53) | 0.77× (0.75–0.78) | 0.64× (0.63–0.65) | 6.08× (5.98–6.19) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.38–0.39) | 0.49× (0.44–0.55) | 0.69× (0.67–0.70) | 0.61× (0.60–0.62) | 4.05× (4.00–4.10) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.43× (0.42–0.45) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.01× (7.99–8.02) | 0.64× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.65× (0.58–0.73) | 1.61× (1.58–1.63) | 0.81× (0.80–0.82) | 5.85× (5.76–5.94) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.60–0.64) | 0.66× (0.60–0.72) | 1.11× (1.08–1.15) | 0.69× (0.67–0.71) | 3.32× (3.21–3.43) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.19–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.39× (0.36–0.44) | 0.93× (0.90–0.96) | 0.72× (0.67–0.76) | 3.21× (3.10–3.32) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.48) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 2.54× (2.48–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.21× (5.20–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.46–0.51) | 0.36× (0.32–0.39) | 0.87× (0.83–0.90) | 0.73× (0.70–0.77) | 2.97× (2.86–3.09) | 0.18× (0.17–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.62) | 0.51× (0.48–0.54) | 0.82× (0.80–0.83) | 0.85× (0.82–0.88) | 2.36× (2.32–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.36× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.77) | 5.31× (5.29–5.33) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.31× (5.29–5.32) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.66–0.67) | 6.44× (6.41–6.46) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 35 MB | 0.48× (0.47–0.48) | 0.42× (0.38–0.46) | 1.12× (1.09–1.15) | 0.72× (0.70–0.74) | 3.53× (3.48–3.58) | 0.21× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.35) | 0.17× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 55 MB | 0.50× (0.49–0.51) | 0.47× (0.44–0.49) | 0.94× (0.90–0.97) | 0.74× (0.72–0.77) | 2.36× (2.32–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 354 | 2.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 631 | 3.02 | 1.07× (1.03–1.11) | 88% | 1.03× (1.01–1.04) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 759 | 5.01 | 1.76× (1.72–1.80) | 54% | 1.47× (1.45–1.50) | 1.67× (1.64–1.70) |

**Best:** mibz2 at 4 threads (759 MB/s); lbzip2 at 4 threads (171 MB/s); pbz2 at 4 threads (305 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.08, lbzip2 1t 0.05, lbzip2 2t 0.17, lbzip2 4t 0.24, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.83 | 0.03 | 0.08 | 0.276 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3680 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 11.0% |
| MTF + RLE2 | 1.73 | 15.4% |
| T-vector build | 1.61 | 14.4% |
| walk (the one each build uses) | 6.29 | 56.2% |
| of which: segment assembly | 0.32 | 2.9% |
| RLE1 | 0.27 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 11.20 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.396 | 0.990 | (in its total) | 2.49× (2.46–2.53) |  |
| walk | 1.587 | 2.945 | (in its total) | 1.84× (1.81–1.87) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.087 | 0.083 | (in its total) | 0.95× (0.93–0.97) |  |
| inverse BWT, total | 2.067 | 4.018 | 2.143 | 1.93× (1.90–1.96) | 1.04× (1.01–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h26m ± 10m19s |
| 4 threads | 40m11s ± 5m40s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m33s ± 4m18s |
| the scanner alone (its serial part) | 14m24s ± 19.7s |
| the workers alone, over 4 | 30m33s ± 4m18s |

At 4 threads the workers bound it. Per byte: the scanner 8.4 ns a byte of bz2, the workers 4.01 ns a byte of output at 4 threads (CPU, summed over them), 2.37 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,559,615 | 12,840,880 | \`c564b1e6883ea19b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6973P-C, 4 cores, reported clock 3680.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
