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

The latest record: [`c79ff7d`](<https://github.com/lmmx/mibz2/commit/c79ff7d384072e5e95eda7e513227f2aa06ff3d5>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42867983470>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.704 s | 298 | 11.7 | 2.47× (2.42–2.52) | 5.62× (5.38–5.87) | 6.46× (6.18–6.74) | 6.67× (6.43–6.91) | 6.19× (5.94–6.45) | 6.79× (6.48–7.11) | 6.11× (5.86–6.36) | 1.78× (1.74–1.82) | 1.73× (1.69–1.78) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.397 s | 528 | 12.8 | 2.59× (2.44–2.74) | 5.46× (5.04–5.90) | 7.09× (6.56–7.67) | 6.39× (6.05–6.76) | 5.84× (5.41–6.30) | 12.35× (11.59–13.16) | 11.10× (10.54–11.69) | 3.23× (3.10–3.37) | 3.15× (3.04–3.26) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.351 s | 597 | 22.2 | 2.73× (2.65–2.81) | 4.45× (3.96–4.98) | 5.70× (5.21–6.24) | 5.10× (4.64–5.61) | 4.90× (4.49–5.34) | 13.92× (13.18–14.70) | 12.51× (11.88–13.19) | 3.64× (3.50–3.79) | 3.55× (3.47–3.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,005 | 3.8 | 0.65× (0.64–0.66) | 3.04× (2.97–3.11) | 5.47× (5.34–5.60) | 3.53× (3.46–3.60) | 6.49× (6.38–6.61) | 2.75× (2.69–2.81) | 2.72× (2.66–2.78) | 1.24× (1.20–1.28) | 1.08× (0.98–1.18) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 957 | 4.0 | 0.63× (0.61–0.65) | 2.95× (2.86–3.05) | 6.88× (6.62–7.15) | 3.51× (3.40–3.62) | 6.19× (6.01–6.38) | 2.66× (2.57–2.75) | 2.63× (2.55–2.72) | 1.20× (1.16–1.24) | 1.04× (0.95–1.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 962 | 4.0 | 0.62× (0.61–0.63) | 2.94× (2.86–3.01) | 6.78× (6.61–6.95) | 3.42× (3.34–3.50) | 6.22× (6.08–6.36) | 2.63× (2.57–2.70) | 2.61× (2.56–2.66) | 1.19× (1.16–1.22) | 1.04× (0.96–1.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 528 | 6.9 | 3.17× (3.10–3.24) | 3.22× (3.04–3.40) | 4.40× (4.19–4.62) | 4.10× (3.91–4.29) | 4.62× (4.39–4.87) | 3.69× (3.50–3.90) | 2.90× (2.75–3.06) | 2.33× (2.15–2.52) | 2.22× (2.16–2.29) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 838 | 8.4 | 2.58× (2.41–2.76) | 2.84× (2.52–3.20) | 4.67× (4.08–5.33) | 3.50× (3.06–3.99) | 5.04× (4.50–5.64) | 5.53× (4.91–6.23) | 4.35× (3.87–4.88) | 3.48× (3.26–3.72) | 3.32× (3.03–3.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.028 s | 706 | 17.6 | 2.41× (2.32–2.51) | 1.89× (1.78–2.01) | 3.16× (2.96–3.37) | 2.22× (2.09–2.36) | 4.08× (3.86–4.32) | 5.06× (4.75–5.38) | 3.98× (3.74–4.23) | 3.19× (2.96–3.43) | 3.04× (2.93–3.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.167 s | 48 | 73.6 | 1.26× (1.25–1.27) | 2.57× (2.24–2.95) | 2.46× (2.11–2.87) | 2.43× (2.15–2.74) | 2.60× (2.38–2.83) | 2.99× (2.76–3.24) | 2.92× (2.64–3.24) | 1.20× (1.18–1.22) | 1.21× (1.19–1.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.096 s | 83 | 78.5 | 1.23× (1.20–1.27) | 2.55× (2.23–2.92) | 2.60× (2.26–2.99) | 3.12× (2.80–3.48) | 2.92× (2.57–3.32) | 5.17× (4.81–5.56) | 5.06× (4.61–5.55) | 2.08× (2.00–2.16) | 2.10× (2.03–2.18) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.080 s | 100 | 108.7 | 1.48× (1.42–1.55) | 2.55× (2.19–2.98) | 2.62× (2.26–3.03) | 3.13× (2.81–3.49) | 3.26× (2.93–3.63) | 6.32× (5.79–6.90) | 6.18× (5.54–6.89) | 2.54× (2.46–2.62) | 2.57× (2.49–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.191 s | 50 | 70.3 | 1.32× (1.29–1.34) | 2.95× (2.59–3.37) | 2.69× (2.42–3.00) | 2.61× (2.35–2.89) | 2.92× (2.67–3.18) | 3.59× (3.40–3.78) | 3.57× (3.39–3.76) | 1.37× (1.36–1.38) | 1.43× (1.40–1.45) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.110 s | 87 | 76.0 | 1.18× (1.06–1.30) | 2.53× (2.27–2.82) | 2.53× (2.29–2.81) | 3.16× (2.86–3.49) | 2.84× (2.51–3.21) | 5.80× (5.36–6.29) | 5.78× (5.28–6.33) | 2.22× (2.03–2.42) | 2.31× (2.11–2.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.090 s | 106 | 106.9 | 1.59× (1.53–1.64) | 2.68× (2.38–3.03) | 2.64× (2.27–3.09) | 3.46× (3.04–3.92) | 3.44× (3.02–3.92) | 7.41× (6.98–7.86) | 7.38× (6.89–7.91) | 2.83× (2.71–2.95) | 2.94× (2.86–3.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 5.4 | 0.70× (0.69–0.72) | 3.15× (2.97–3.34) | 5.05× (4.88–5.22) | 3.09× (3.03–3.16) | 7.27× (7.14–7.39) | 2.46× (2.41–2.52) | 2.46× (2.41–2.51) | 1.19× (0.95–1.48) | 1.52× (1.49–1.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 662 | 6.0 | 0.62× (0.58–0.65) | 2.68× (2.53–2.83) | 5.68× (5.38–6.00) | 2.80× (2.65–2.96) | 6.59× (6.15–7.07) | 2.19× (2.06–2.32) | 2.19× (2.07–2.31) | 1.05× (0.85–1.31) | 1.35× (1.28–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 660 | 6.0 | 0.63× (0.61–0.64) | 2.74× (2.68–2.81) | 5.80× (5.66–5.95) | 2.88× (2.80–2.97) | 6.76× (6.51–7.02) | 2.23× (2.19–2.28) | 2.23× (2.20–2.27) | 1.08× (0.88–1.32) | 1.38× (1.34–1.41) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.081 s | 159 | 22.4 | 1.88× (1.80–1.97) | 4.16× (3.84–4.50) | 4.99× (4.52–5.52) | 4.91× (4.48–5.39) | 5.05× (4.58–5.58) | 5.29× (4.96–5.65) | 4.91× (4.59–5.26) | 1.52× (1.45–1.60) | 1.46× (1.43–1.50) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.052 s | 249 | 26.0 | 1.64× (1.49–1.82) | 3.68× (3.27–4.14) | 4.55× (3.99–5.19) | 4.72× (4.27–5.21) | 4.76× (4.39–5.16) | 8.01× (7.35–8.72) | 7.43× (6.82–8.11) | 2.30× (2.12–2.50) | 2.21× (2.08–2.35) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.048 s | 269 | 41.0 | 1.67× (1.59–1.76) | 3.22× (2.92–3.56) | 4.00× (3.54–4.52) | 4.02× (3.63–4.45) | 4.46× (4.10–4.86) | 8.82× (8.16–9.54) | 8.19× (7.53–8.91) | 2.54× (2.38–2.71) | 2.44× (2.38–2.50) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.47× (2.42–2.52) | 5.60× (5.36–5.85) | 6.44× (6.17–6.72) | 6.65× (6.42–6.90) | 6.18× (5.93–6.43) | 6.78× (6.47–7.10) | 6.09× (5.85–6.35) | 1.78× (1.74–1.82) | 1.72× (1.68–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.44× (2.35–2.54) | 5.49× (5.15–5.85) | 6.69× (6.17–7.24) | 6.55× (6.17–6.96) | 5.76× (5.32–6.24) | 6.38× (6.04–6.75) | 5.74× (5.48–6.01) | 1.67× (1.63–1.72) | 1.62× (1.58–1.67) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.57× (2.49–2.65) | 4.53× (4.05–5.06) | 5.36× (4.86–5.92) | 5.20× (4.72–5.73) | 4.59× (4.16–5.06) | 3.64× (3.43–3.86) | 3.27× (3.10–3.45) | 0.95× (0.92–0.99) | 0.92× (0.90–0.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.67–0.69) | 2.79× (2.73–2.86) | 4.99× (4.90–5.09) | 3.32× (3.25–3.39) | 6.04× (5.93–6.15) | 2.60× (2.55–2.66) | 2.59× (2.52–2.65) | 1.22× (1.18–1.26) | 1.00× (0.95–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.67× (0.65–0.69) | 2.68× (2.60–2.75) | 6.16× (5.97–6.36) | 3.28× (3.18–3.38) | 5.70× (5.54–5.87) | 2.49× (2.41–2.56) | 2.47× (2.39–2.55) | 1.17× (1.13–1.21) | 0.96× (0.91–1.01) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.65× (0.64–0.67) | 3.06× (3.00–3.12) | 6.19× (6.07–6.30) | 3.25× (3.17–3.33) | 5.80× (5.67–5.93) | 2.51× (2.45–2.57) | 2.49× (2.44–2.54) | 1.18× (1.15–1.21) | 0.97× (0.92–1.01) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.07× (3.00–3.14) | 2.94× (2.80–3.08) | 4.11× (3.93–4.29) | 3.96× (3.79–4.13) | 4.46× (4.24–4.68) | 3.57× (3.39–3.76) | 2.82× (2.68–2.96) | 2.27× (2.10–2.45) | 2.09× (2.02–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.62× (2.51–2.73) | 2.51× (2.29–2.76) | 3.66× (3.30–4.05) | 3.23× (2.94–3.55) | 3.66× (3.34–4.02) | 2.84× (2.57–3.14) | 2.24× (2.03–2.47) | 1.80× (1.73–1.88) | 1.66× (1.55–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.51× (2.38–2.65) | 1.69× (1.58–1.80) | 2.49× (2.33–2.66) | 1.99× (1.87–2.12) | 2.22× (2.09–2.35) | 1.44× (1.34–1.54) | 1.14× (1.06–1.21) | 0.91× (0.85–0.98) | 0.84× (0.80–0.88) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.25× (1.24–1.27) | 2.53× (2.21–2.90) | 2.42× (2.07–2.82) | 2.41× (2.14–2.72) | 2.58× (2.36–2.81) | 2.97× (2.74–3.22) | 2.90× (2.62–3.21) | 1.20× (1.18–1.22) | 1.18× (1.16–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.21–1.25) | 2.50× (2.18–2.88) | 2.40× (2.06–2.79) | 2.97× (2.66–3.32) | 2.62× (2.27–3.03) | 2.80× (2.60–3.01) | 2.74× (2.49–3.01) | 1.13× (1.10–1.16) | 1.12× (1.09–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.57–1.69) | 2.79× (2.42–3.22) | 2.43× (2.09–2.84) | 3.08× (2.77–3.43) | 2.96× (2.61–3.35) | 2.05× (1.90–2.21) | 2.00× (1.81–2.21) | 0.83× (0.80–0.85) | 0.82× (0.79–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.31× (1.29–1.34) | 2.91× (2.56–3.31) | 2.65× (2.36–2.96) | 2.59× (2.34–2.87) | 2.90× (2.66–3.16) | 3.56× (3.38–3.76) | 3.55× (3.37–3.74) | 1.37× (1.35–1.38) | 1.38× (1.36–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.24× (1.19–1.29) | 2.68× (2.50–2.86) | 2.49× (2.24–2.76) | 3.09× (2.91–3.29) | 2.74× (2.47–3.03) | 3.21× (3.10–3.32) | 3.20× (3.02–3.38) | 1.23× (1.18–1.28) | 1.24× (1.19–1.30) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.66–1.77) | 2.88× (2.58–3.22) | 2.42× (2.06–2.84) | 3.24× (2.93–3.59) | 3.17× (2.79–3.61) | 2.31× (2.20–2.43) | 2.30× (2.19–2.42) | 0.89× (0.87–0.91) | 0.90× (0.88–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.74× (0.72–0.75) | 2.59× (2.48–2.71) | 4.26× (4.18–4.34) | 2.86× (2.80–2.92) | 6.55× (6.44–6.66) | 2.30× (2.25–2.35) | 2.29× (2.24–2.35) | 1.18× (0.95–1.48) | 0.89× (0.87–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.67× (0.63–0.70) | 2.24× (2.11–2.38) | 4.72× (4.46–5.01) | 2.62× (2.49–2.75) | 5.90× (5.53–6.29) | 2.02× (1.91–2.13) | 2.01× (1.91–2.12) | 1.04× (0.84–1.29) | 0.78× (0.74–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.66× (0.65–0.68) | 2.68× (2.61–2.75) | 4.91× (4.84–4.99) | 2.75× (2.68–2.83) | 6.17× (5.96–6.39) | 2.10× (2.07–2.14) | 2.10× (2.07–2.13) | 1.08× (0.88–1.33) | 0.82× (0.79–0.84) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.86× (1.78–1.94) | 4.03× (3.70–4.38) | 4.84× (4.38–5.35) | 4.82× (4.40–5.29) | 4.96× (4.50–5.48) | 5.19× (4.87–5.54) | 4.82× (4.51–5.16) | 1.51× (1.44–1.59) | 1.40× (1.38–1.43) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.67× (1.62–1.73) | 3.81× (3.37–4.31) | 4.45× (3.94–5.03) | 4.59× (4.21–5.00) | 4.32× (3.98–4.69) | 4.39× (4.08–4.72) | 4.07× (3.77–4.40) | 1.28× (1.20–1.36) | 1.19× (1.14–1.23) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.95× (1.87–2.04) | 3.56× (3.13–4.05) | 4.03× (3.44–4.72) | 3.97× (3.57–4.42) | 3.99× (3.56–4.47) | 2.82× (2.61–3.05) | 2.62× (2.42–2.84) | 0.82× (0.77–0.88) | 0.76× (0.74–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.28× (0.28–0.28) | 1.05× (1.05–1.06) | 0.84× (0.84–0.84) | 11.90× (11.88–11.92) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.37× (0.36–0.38) | 0.51× (0.43–0.60) | 0.76× (0.74–0.78) | 0.63× (0.61–0.65) | 6.02× (5.88–6.17) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.57× (0.48–0.68) | 0.70× (0.69–0.72) | 0.63× (0.62–0.63) | 4.10× (4.08–4.11) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.43× (0.42–0.45) | 1.70× (1.70–1.71) | 0.65× (0.65–0.66) | 8.03× (8.01–8.05) | 0.65× (0.65–0.65) | 0.68× (0.67–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.66× (0.63–0.69) | 1.62× (1.59–1.65) | 0.81× (0.80–0.83) | 5.88× (5.77–5.98) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.63–0.66) | 0.73× (0.68–0.78) | 1.15× (1.13–1.18) | 0.71× (0.69–0.73) | 3.46× (3.39–3.54) | 0.27× (0.26–0.28) | 0.28× (0.28–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.38× (0.35–0.41) | 0.88× (0.85–0.92) | 0.70× (0.67–0.73) | 3.05× (2.96–3.15) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.86× (0.84–0.88) | 0.80× (0.78–0.83) | 2.51× (2.46–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.20–5.24) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.48) | 0.34× (0.31–0.38) | 0.84× (0.81–0.86) | 0.71× (0.69–0.73) | 2.86× (2.78–2.94) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.45–0.50) | 0.81× (0.80–0.82) | 0.82× (0.80–0.85) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.20× (0.20–0.21) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.61) | 0.77× (0.76–0.78) | 5.31× (5.29–5.33) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.47) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.67× (0.66–0.67) | 6.44× (6.42–6.46) | 0.38× (0.38–0.39) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.41× (0.37–0.45) | 1.15× (1.12–1.18) | 0.75× (0.73–0.76) | 3.59× (3.53–3.66) | 0.21× (0.21–0.22) | 0.24× (0.23–0.24) | 0.36× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.52) | 0.47× (0.44–0.51) | 0.93× (0.90–0.95) | 0.75× (0.74–0.77) | 2.37× (2.32–2.43) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 298 | 3.36 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 528 | 3.67 | 1.06× (1.04–1.08) | 91% | 1.04× (1.00–1.08) | 1.05× (1.00–1.10) |
| 4: all 4 CPUs, sharing SMT siblings | 597 | 6.35 | 1.86× (1.81–1.92) | 51% | 1.51× (1.37–1.65) | 1.94× (1.85–2.03) |

**Best:** mibz2 at 4 threads (597 MB/s); lbzip2 at 4 threads (135 MB/s); pbz2 at 4 threads (221 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.02, lbzip2 1t 0.05, lbzip2 2t 0.24, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.30. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 6%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.09 | 0.03 | 0.09 | 0.351 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3492 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.27 | 10.3% |
| MTF + RLE2 | 1.75 | 14.3% |
| T-vector build | 1.78 | 14.6% |
| walk (the one each build uses) | 6.98 | 57.0% |
| of which: segment assembly | 0.38 | 3.1% |
| RLE1 | 0.32 | 2.6% |
| CRC | 0.16 | 1.3% |
| **total** | 12.25 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.490 | 1.272 | (in its total) | 2.60× (2.59–2.61) |  |
| walk | 1.835 | 2.811 | (in its total) | 1.54× (1.52–1.57) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.108 | 0.117 | (in its total) | 1.12× (1.07–1.17) |  |
| inverse BWT, total | 2.432 | 4.199 | 2.524 | 1.74× (1.72–1.76) | 1.04× (1.03–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h44m ± 13m46s |
| 4 threads | 50m56s ± 7m28s |
| 4 threads, the sample's overlap gap as a fixed cost | 40m12s ± 6m02s |
| the scanner alone (its serial part) | 17m10s ± 26.3s |
| the workers alone, over 4 | 40m12s ± 6m02s |

At 4 threads the workers bound it. Per byte: the scanner 10.0 ns a byte of bz2, the workers 5.27 ns a byte of output at 4 threads (CPU, summed over them), 2.86 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,559,615 | 12,840,880 | \`c564b1e6883ea19b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 3491.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
