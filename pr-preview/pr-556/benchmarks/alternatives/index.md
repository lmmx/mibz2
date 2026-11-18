# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-556/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`f71de52`](<https://github.com/lmmx/mibz2/commit/f71de528d33a6c8288353176f70a9708bdca4283>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44272447110>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.404 s | 519 | 7.4 | 3.36× (3.35–3.38) | 7.70× (7.65–7.75) | 8.21× (8.17–8.24) | 9.11× (9.06–9.16) | 8.24× (8.20–8.27) | 8.61× (8.57–8.65) | 8.42× (8.38–8.47) | 2.62× (2.61–2.63) | 2.33× (2.32–2.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.226 s | 926 | 8.1 | 3.27× (3.20–3.34) | 7.08× (6.95–7.21) | 8.52× (8.33–8.72) | 8.52× (8.35–8.69) | 7.71× (7.57–7.85) | 15.20× (14.93–15.49) | 14.87× (14.58–15.16) | 4.62× (4.54–4.71) | 4.12× (4.04–4.20) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.191 s | 1,100 | 12.4 | 3.30× (3.24–3.36) | 6.10× (5.99–6.22) | 7.16× (7.02–7.30) | 7.09× (6.94–7.24) | 6.93× (6.78–7.08) | 18.19× (17.80–18.58) | 17.79× (17.40–18.18) | 5.53× (5.41–5.66) | 4.93× (4.83–5.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,250 | 3.4 | 0.65× (0.64–0.67) | 3.21× (3.13–3.29) | 4.58× (4.46–4.70) | 3.90× (3.83–3.98) | 6.50× (6.38–6.63) | 2.89× (2.83–2.95) | 2.92× (2.85–2.99) | 1.14× (1.12–1.16) | 1.30× (1.27–1.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,227 | 3.5 | 0.57× (0.55–0.60) | 3.11× (2.97–3.27) | 6.29× (5.99–6.60) | 3.81× (3.66–3.97) | 6.26× (6.00–6.52) | 2.80× (2.68–2.92) | 2.83× (2.70–2.96) | 1.11× (1.06–1.16) | 1.26× (1.20–1.32) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,212 | 3.4 | 0.56× (0.54–0.58) | 3.05× (2.96–3.14) | 5.84× (5.62–6.06) | 3.74× (3.63–3.85) | 6.20× (6.02–6.38) | 2.74× (2.67–2.81) | 2.77× (2.70–2.84) | 1.08× (1.05–1.11) | 1.23× (1.20–1.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 638 | 6.3 | 2.89× (2.84–2.94) | 3.30× (3.24–3.36) | 4.00× (3.93–4.07) | 4.33× (4.26–4.41) | 4.71× (4.63–4.79) | 3.16× (3.11–3.21) | 2.92× (2.87–2.96) | 2.01× (1.98–2.04) | 2.07× (1.97–2.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,026 | 7.5 | 2.52× (2.36–2.69) | 3.16× (3.01–3.31) | 4.39× (4.25–4.53) | 3.89× (3.68–4.11) | 5.23× (4.98–5.49) | 5.02× (4.79–5.26) | 4.63× (4.42–4.85) | 3.19× (3.03–3.36) | 3.29× (3.05–3.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,029 | 13.6 | 2.01× (1.96–2.08) | 2.20× (2.14–2.25) | 3.17× (2.99–3.35) | 2.64× (2.47–2.82) | 4.36× (4.27–4.45) | 5.07× (4.97–5.18) | 4.68× (4.59–4.77) | 3.23× (3.14–3.31) | 3.33× (3.17–3.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 s | 70 | 55.4 | 1.37× (1.37–1.38) | 3.04× (3.02–3.05) | 2.78× (2.76–2.79) | 2.82× (2.81–2.83) | 3.11× (3.09–3.13) | 3.45× (3.43–3.46) | 3.55× (3.52–3.57) | 1.25× (1.24–1.25) | 1.25× (1.24–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 120 | 60.1 | 1.32× (1.28–1.35) | 2.88× (2.79–2.97) | 2.86× (2.79–2.93) | 3.49× (3.40–3.58) | 3.23× (3.15–3.32) | 5.79× (5.65–5.94) | 5.96× (5.84–6.09) | 2.09× (2.04–2.15) | 2.10× (2.05–2.15) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 139 | 82.5 | 1.36× (1.31–1.40) | 2.71× (2.62–2.79) | 2.55× (2.47–2.63) | 3.19× (3.11–3.27) | 3.16× (3.06–3.27) | 6.87× (6.67–7.08) | 7.07× (6.85–7.30) | 2.48× (2.41–2.56) | 2.49× (2.42–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.127 s | 76 | 51.3 | 1.42× (1.41–1.42) | 3.39× (3.36–3.42) | 2.89× (2.86–2.93) | 2.97× (2.95–2.99) | 3.30× (3.28–3.32) | 4.08× (4.06–4.09) | 4.36× (4.34–4.37) | 1.45× (1.44–1.45) | 1.52× (1.51–1.53) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.075 s | 128 | 56.7 | 1.29× (1.23–1.35) | 3.05× (2.92–3.19) | 2.82× (2.73–2.91) | 3.75× (3.61–3.90) | 3.23× (3.11–3.36) | 6.73× (6.46–7.00) | 7.19× (6.92–7.48) | 2.39× (2.29–2.48) | 2.51× (2.41–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 154 | 78.4 | 1.42× (1.39–1.45) | 2.93× (2.85–3.01) | 2.72× (2.63–2.81) | 3.64× (3.53–3.75) | 3.18× (3.09–3.28) | 8.26× (8.04–8.48) | 8.83× (8.60–9.07) | 2.93× (2.86–3.00) | 3.08× (2.99–3.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 935 | 4.8 | 0.69× (0.68–0.71) | 2.90× (2.79–3.01) | 4.09× (3.89–4.30) | 3.31× (3.21–3.41) | 6.98× (6.72–7.25) | 2.48× (2.41–2.56) | 2.51× (2.43–2.59) | 1.05× (1.01–1.09) | 0.99× (0.95–1.02) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 893 | 5.1 | 0.55× (0.54–0.56) | 2.75× (2.69–2.82) | 5.61× (5.47–5.75) | 3.19× (3.12–3.26) | 6.63× (6.51–6.75) | 2.36× (2.31–2.40) | 2.38× (2.34–2.42) | 1.00× (0.97–1.03) | 0.94× (0.92–0.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 873 | 5.0 | 0.54× (0.53–0.55) | 2.70× (2.65–2.75) | 5.38× (5.14–5.64) | 3.15× (3.09–3.21) | 6.58× (6.46–6.69) | 2.31× (2.26–2.35) | 2.33× (2.30–2.37) | 0.98× (0.94–1.02) | 0.92× (0.90–0.93) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 0.060 s | 215 | 18.4 | 1.83× (1.82–1.83) | 4.13× (4.07–4.19) | 4.63× (4.60–4.66) | 4.71× (4.69–4.72) | 4.86× (4.83–4.88) | 5.09× (5.07–5.10) | 5.05× (5.02–5.07) | 1.48× (1.47–1.48) | 1.53× (1.53–1.54) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 0.037 s | 342 | 21.3 | 1.60× (1.52–1.70) | 3.67× (3.38–4.00) | 4.53× (4.30–4.77) | 4.77× (4.53–5.01) | 4.66× (4.44–4.91) | 7.99× (7.59–8.41) | 7.93× (7.52–8.35) | 2.32× (2.20–2.44) | 2.40× (2.28–2.54) | yes |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 0.032 s | 398 | 31.8 | 1.63× (1.57–1.68) | 3.11× (3.00–3.23) | 3.90× (3.78–4.02) | 4.27× (4.17–4.38) | 4.48× (4.37–4.59) | 9.38× (9.14–9.63) | 9.31× (9.07–9.54) | 2.72× (2.66–2.79) | 2.82× (2.76–2.89) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.35× (3.34–3.37) | 7.66× (7.62–7.71) | 8.17× (8.13–8.20) | 9.08× (9.03–9.12) | 8.21× (8.17–8.24) | 8.58× (8.54–8.62) | 8.39× (8.35–8.43) | 2.61× (2.60–2.63) | 2.32× (2.30–2.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.20× (3.17–3.22) | 7.32× (7.25–7.40) | 8.08× (8.02–8.14) | 8.85× (8.79–8.91) | 7.66× (7.59–7.72) | 7.94× (7.88–8.00) | 7.77× (7.70–7.83) | 2.42× (2.40–2.44) | 2.15× (2.13–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.27× (3.20–3.35) | 6.75× (6.66–6.84) | 7.19× (7.10–7.29) | 7.85× (7.75–7.94) | 7.01× (6.92–7.11) | 5.13× (5.07–5.19) | 5.02× (4.95–5.08) | 1.56× (1.54–1.59) | 1.39× (1.37–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.69× (0.67–0.70) | 2.95× (2.86–3.03) | 3.97× (3.88–4.06) | 3.64× (3.56–3.71) | 6.00× (5.86–6.14) | 2.72× (2.65–2.80) | 2.74× (2.67–2.82) | 1.13× (1.10–1.15) | 0.83× (0.82–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.59–0.64) | 2.84× (2.72–2.96) | 5.45× (5.25–5.67) | 3.51× (3.37–3.65) | 5.67× (5.46–5.89) | 2.59× (2.49–2.69) | 2.61× (2.50–2.72) | 1.07× (1.03–1.12) | 0.79× (0.76–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.58–0.61) | 3.15× (3.08–3.21) | 5.41× (5.28–5.53) | 3.53× (3.44–3.63) | 5.76× (5.60–5.93) | 2.60× (2.54–2.67) | 2.62× (2.56–2.69) | 1.08× (1.05–1.11) | 0.79× (0.77–0.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.75–2.84) | 3.06× (3.01–3.12) | 3.59× (3.54–3.64) | 4.16× (4.09–4.23) | 4.52× (4.44–4.60) | 3.05× (3.00–3.11) | 2.82× (2.77–2.87) | 1.96× (1.93–1.99) | 1.89× (1.85–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.49× (2.45–2.53) | 2.76× (2.71–2.81) | 3.35× (3.30–3.40) | 3.60× (3.54–3.66) | 3.90× (3.83–3.98) | 2.56× (2.52–2.61) | 2.37× (2.33–2.41) | 1.65× (1.61–1.69) | 1.59× (1.55–1.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.06× (2.01–2.11) | 1.86× (1.81–1.91) | 2.31× (2.26–2.37) | 2.25× (2.20–2.29) | 2.35× (2.30–2.41) | 1.42× (1.38–1.45) | 1.31× (1.28–1.34) | 0.91× (0.88–0.94) | 0.88× (0.85–0.90) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.37) | 2.95× (2.93–2.97) | 2.69× (2.68–2.70) | 2.79× (2.78–2.80) | 3.08× (3.06–3.10) | 3.41× (3.39–3.42) | 3.51× (3.48–3.53) | 1.24× (1.24–1.25) | 1.23× (1.22–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.30–1.32) | 2.85× (2.82–2.89) | 2.60× (2.57–2.63) | 3.31× (3.27–3.35) | 2.88× (2.85–2.92) | 3.12× (3.09–3.16) | 3.22× (3.18–3.25) | 1.14× (1.13–1.15) | 1.13× (1.11–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.48–1.57) | 2.93× (2.87–2.98) | 2.47× (2.42–2.52) | 3.23× (3.17–3.29) | 2.85× (2.79–2.91) | 2.29× (2.25–2.33) | 2.36× (2.31–2.40) | 0.83× (0.82–0.85) | 0.82× (0.81–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.41–1.41) | 3.31× (3.28–3.35) | 2.82× (2.79–2.84) | 2.94× (2.93–2.96) | 3.27× (3.25–3.29) | 4.03× (4.02–4.05) | 4.31× (4.30–4.32) | 1.44× (1.43–1.45) | 1.45× (1.44–1.45) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.31× (1.27–1.35) | 3.08× (2.98–3.18) | 2.63× (2.55–2.72) | 3.54× (3.42–3.66) | 2.98× (2.88–3.08) | 3.59× (3.48–3.72) | 3.84× (3.72–3.97) | 1.28× (1.24–1.33) | 1.29× (1.25–1.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.62) | 3.19× (3.15–3.25) | 2.47× (2.43–2.50) | 3.37× (3.29–3.45) | 2.91× (2.85–2.97) | 2.62× (2.57–2.67) | 2.80× (2.75–2.85) | 0.93× (0.92–0.95) | 0.94× (0.92–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.75) | 2.52× (2.46–2.59) | 3.50× (3.42–3.59) | 3.01× (2.92–3.11) | 6.20× (5.94–6.47) | 2.29× (2.21–2.37) | 2.31× (2.24–2.39) | 1.04× (1.01–1.08) | 0.77× (0.74–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.62) | 2.31× (2.26–2.35) | 4.48× (4.42–4.53) | 2.90× (2.85–2.95) | 5.82× (5.72–5.92) | 2.14× (2.10–2.18) | 2.16× (2.13–2.19) | 0.97× (0.95–0.99) | 0.72× (0.70–0.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.64× (2.54–2.73) | 4.48× (4.40–4.57) | 2.96× (2.91–3.02) | 5.95× (5.84–6.05) | 2.16× (2.12–2.20) | 2.18× (2.15–2.21) | 0.98× (0.95–1.01) | 0.73× (0.71–0.74) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 1.80× (1.80–1.81) | 3.98× (3.96–4.01) | 4.48× (4.45–4.51) | 4.61× (4.59–4.62) | 4.75× (4.73–4.78) | 4.97× (4.96–4.99) | 4.94× (4.92–4.96) | 1.46× (1.46–1.47) | 1.37× (1.37–1.37) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 1.64× (1.62–1.66) | 3.62× (3.55–3.68) | 4.24× (4.19–4.30) | 4.51× (4.46–4.57) | 4.20× (4.13–4.26) | 4.30× (4.23–4.37) | 4.27× (4.20–4.34) | 1.27× (1.24–1.29) | 1.18× (1.17–1.20) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 1.86× (1.80–1.93) | 3.34× (3.26–3.42) | 3.74× (3.63–3.85) | 4.05× (3.92–4.18) | 3.78× (3.68–3.88) | 2.90× (2.83–2.98) | 2.88× (2.81–2.95) | 0.85× (0.83–0.87) | 0.80× (0.78–0.82) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.28× (0.27–0.30) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.97× (10.96–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.47× (0.41–0.55) | 0.70× (0.68–0.72) | 0.59× (0.57–0.61) | 5.61× (5.45–5.77) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.33× (0.32–0.35) | 0.55× (0.47–0.65) | 0.59× (0.56–0.62) | 0.52× (0.49–0.55) | 3.47× (3.33–3.62) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.42× (0.41–0.44) | 1.70× (1.69–1.71) | 0.66× (0.65–0.66) | 8.02× (7.98–8.05) | 0.66× (0.65–0.66) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.70) | 0.64× (0.56–0.72) | 1.60× (1.55–1.64) | 0.81× (0.78–0.83) | 5.82× (5.65–5.99) | 0.47× (0.46–0.49) | 0.50× (0.48–0.51) | 0.54× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.65–0.66) | 0.74× (0.69–0.78) | 1.16× (1.14–1.19) | 0.73× (0.72–0.74) | 3.49× (3.44–3.54) | 0.27× (0.27–0.28) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.17× (5.14–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.44) | 0.91× (0.87–0.95) | 0.71× (0.67–0.75) | 3.13× (3.00–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.80× (0.78–0.83) | 2.54× (2.48–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.46–0.51) | 0.40× (0.38–0.42) | 0.87× (0.83–0.91) | 0.72× (0.68–0.75) | 2.98× (2.85–3.13) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.50) | 0.81× (0.80–0.82) | 0.81× (0.78–0.85) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.36× (5.35–5.38) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.28–5.30) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.30–5.34) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.66) | 6.39× (6.38–6.39) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 2 | 34 MB | 0.48× (0.46–0.50) | 0.41× (0.37–0.46) | 1.13× (1.07–1.19) | 0.73× (0.71–0.76) | 3.54× (3.41–3.67) | 0.22× (0.21–0.23) | 0.23× (0.22–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.52 MB → 12.8 MB, 5.1×) | 4 | 54 MB | 0.51× (0.49–0.52) | 0.49× (0.47–0.52) | 0.93× (0.90–0.95) | 0.75× (0.72–0.79) | 2.37× (2.31–2.44) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 519 | 1.93 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 926 | 2.10 | 1.08× (1.07–1.09) | 88% | 1.03× (1.03–1.04) | 1.03× (1.03–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,100 | 3.22 | 1.67× (1.65–1.70) | 53% | 1.47× (1.46–1.48) | 1.63× (1.60–1.66) |

**Best:** mibz2 at 4 threads (1,100 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (332 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.23, lbzip2 1t 0.04, lbzip2 2t 0.20, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.50 | 0.02 | 0.07 | 0.191 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3837 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 10.8% |
| MTF + RLE2 | 1.62 | 15.2% |
| T-vector build | 1.54 | 14.5% |
| walk (the one each build uses) | 5.95 | 55.9% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.24 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.65 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.370 | 0.936 | (in its total) | 2.52× (2.52–2.53) |  |
| walk | 1.468 | 2.759 | (in its total) | 1.88× (1.88–1.88) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.91× (0.91–0.91) |  |
| inverse BWT, total | 1.918 | 3.767 | 2.006 | 1.96× (1.96–1.97) | 1.05× (1.04–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 58m47s ± 6m44s |
| 4 threads | 27m51s ± 3m57s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m27s ± 2m38s |
| the scanner alone (its serial part) | 10m46s ± 32.2s |
| the workers alone, over 4 | 18m26s ± 2m38s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.42 ns a byte of output at 4 threads (CPU, summed over them), 1.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,519,058 | 12,810,756 | \`6ae766e8f83bdb21…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3837.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
