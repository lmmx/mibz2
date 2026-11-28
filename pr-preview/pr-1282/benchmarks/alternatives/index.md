# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`37254a0`](<https://github.com/lmmx/mibz2/commit/37254a0d019ffbf212b74dbc0848e55a45458b1b>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46154430034>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.372 s | 564 | 7.0 | 3.72× (3.70–3.73) | 8.53× (8.49–8.56) | 9.07× (9.02–9.12) | 10.02× (9.98–10.06) | 9.11× (9.06–9.15) | 9.51× (9.46–9.55) | 9.25× (9.21–9.30) | 2.94× (2.93–2.94) | 2.59× (2.58–2.60) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.219 s | 959 | 8.0 | 3.33× (3.17–3.50) | 7.25× (6.89–7.63) | 8.72× (8.30–9.17) | 8.68× (8.25–9.14) | 7.86× (7.47–8.26) | 15.74× (14.94–16.59) | 15.32× (14.53–16.16) | 4.86× (4.62–5.11) | 4.29× (4.07–4.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.178 s | 1,174 | 12.5 | 3.56× (3.51–3.61) | 6.59× (6.51–6.68) | 7.73× (7.64–7.82) | 7.64× (7.58–7.71) | 7.46× (7.39–7.54) | 19.86× (19.66–20.05) | 19.33× (19.12–19.54) | 6.13× (6.08–6.18) | 5.40× (5.36–5.44) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,465 | 2.9 | 0.67× (0.62–0.74) | 3.71× (3.38–4.07) | 5.32× (4.84–5.84) | 4.24× (3.88–4.64) | 7.26× (6.62–7.95) | 3.30× (3.01–3.62) | 3.33× (3.05–3.65) | 1.14× (1.05–1.24) | 1.50× (1.36–1.65) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,333 | 3.3 | 0.59× (0.55–0.62) | 3.38× (3.17–3.61) | 6.53× (6.12–6.97) | 3.91× (3.69–4.15) | 6.58× (6.19–7.00) | 3.01× (2.82–3.22) | 3.04× (2.86–3.23) | 1.04× (0.99–1.10) | 1.37× (1.28–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,212 | 3.5 | 0.56× (0.53–0.60) | 3.22× (3.01–3.45) | 6.26× (5.92–6.63) | 3.74× (3.53–3.97) | 6.34× (5.99–6.72) | 2.86× (2.67–3.07) | 2.89× (2.71–3.08) | 0.99× (0.95–1.04) | 1.30× (1.21–1.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 689 | 5.9 | 2.78× (2.68–2.87) | 3.52× (3.40–3.65) | 4.04× (3.94–4.15) | 4.52× (4.36–4.68) | 4.90× (4.78–5.02) | 3.33× (3.22–3.44) | 3.05× (2.96–3.16) | 2.14× (2.07–2.21) | 2.11× (2.04–2.19) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,160 | 6.7 | 2.63× (2.53–2.73) | 3.66× (3.52–3.80) | 4.80× (4.56–5.06) | 4.57× (4.13–5.06) | 5.86× (5.71–6.02) | 5.74× (5.54–5.95) | 5.27× (5.09–5.45) | 3.69× (3.56–3.83) | 3.65× (3.51–3.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,188 | 11.5 | 2.20× (2.06–2.35) | 2.52× (2.36–2.70) | 3.29× (3.07–3.52) | 2.93× (2.70–3.18) | 4.83× (4.59–5.08) | 5.78× (5.41–6.18) | 5.30× (4.99–5.64) | 3.72× (3.51–3.94) | 3.67× (3.43–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.106 s | 76 | 52.2 | 1.48× (1.47–1.49) | 3.28× (3.25–3.31) | 2.99× (2.97–3.02) | 3.09× (3.06–3.11) | 3.41× (3.38–3.45) | 3.75× (3.72–3.78) | 3.89× (3.87–3.91) | 1.34× (1.33–1.35) | 1.34× (1.33–1.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 130 | 55.3 | 1.37× (1.27–1.49) | 3.12× (2.76–3.53) | 2.96× (2.76–3.19) | 3.82× (3.38–4.31) | 3.34× (3.09–3.61) | 6.15× (5.67–6.67) | 6.38× (5.90–6.91) | 2.20× (2.03–2.38) | 2.20× (2.03–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.048 s | 168 | 74.0 | 1.59× (1.55–1.63) | 3.15× (3.06–3.25) | 3.02× (2.88–3.16) | 3.78× (3.64–3.92) | 3.68× (3.57–3.79) | 8.21× (7.96–8.47) | 8.53× (8.27–8.79) | 2.94× (2.85–3.03) | 2.94× (2.85–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 47.9 | 1.54× (1.52–1.56) | 3.70× (3.65–3.74) | 3.13× (3.11–3.15) | 3.28× (3.26–3.29) | 3.68× (3.66–3.70) | 4.47× (4.44–4.49) | 4.77× (4.73–4.81) | 1.57× (1.56–1.58) | 1.65× (1.64–1.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.066 s | 146 | 51.1 | 1.48× (1.45–1.51) | 3.53× (3.45–3.61) | 3.26× (3.20–3.32) | 4.36× (4.28–4.43) | 3.77× (3.72–3.82) | 7.94× (7.82–8.07) | 8.49× (8.33–8.65) | 2.79× (2.75–2.84) | 2.93× (2.88–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.053 s | 180 | 71.3 | 1.68× (1.62–1.74) | 3.44× (3.32–3.56) | 3.18× (3.07–3.30) | 4.21× (4.10–4.31) | 3.74× (3.67–3.82) | 9.77× (9.58–9.96) | 10.44× (10.23–10.65) | 3.43× (3.37–3.50) | 3.61× (3.53–3.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.008 s | 1,190 | 3.7 | 0.72× (0.68–0.75) | 3.51× (3.24–3.79) | 4.90× (4.52–5.32) | 3.78× (3.54–4.04) | 8.11× (7.64–8.60) | 2.93× (2.72–3.16) | 2.95× (2.74–3.18) | 1.09× (1.04–1.15) | 1.19× (1.10–1.29) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,042 | 4.4 | 0.58× (0.55–0.62) | 3.10× (2.88–3.33) | 5.83× (5.46–6.22) | 3.38× (3.17–3.60) | 7.19× (6.79–7.61) | 2.58× (2.41–2.77) | 2.60× (2.43–2.79) | 0.96× (0.93–1.00) | 1.05× (0.98–1.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 889 | 5.0 | 0.53× (0.51–0.56) | 2.77× (2.66–2.89) | 5.30× (5.10–5.50) | 3.05× (2.93–3.19) | 6.45× (6.28–6.62) | 2.31× (2.22–2.41) | 2.33× (2.24–2.42) | 0.86× (0.82–0.91) | 0.94× (0.90–0.98) | yes |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 1 | 0.055 s | 277 | 14.4 | 2.10× (2.02–2.17) | 4.44× (4.28–4.60) | 5.08× (4.93–5.24) | 5.11× (4.94–5.28) | 5.18× (5.02–5.35) | 5.34× (5.17–5.52) | 5.20× (5.04–5.37) | 1.69× (1.63–1.74) | 1.65× (1.59–1.71) | yes |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 2 | 0.035 s | 433 | 17.4 | 1.86× (1.65–2.11) | 3.90× (3.75–4.06) | 4.87× (4.66–5.09) | 5.27× (5.08–5.47) | 5.00× (4.89–5.10) | 8.54× (8.28–8.80) | 8.32× (8.07–8.57) | 2.70× (2.62–2.78) | 2.64× (2.56–2.73) | yes |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 4 | 0.030 s | 507 | 26.8 | 1.82× (1.75–1.90) | 3.29× (3.09–3.51) | 3.99× (3.87–4.11) | 4.78× (4.57–5.01) | 4.76× (4.61–4.92) | 9.82× (9.47–10.20) | 9.57× (9.23–9.93) | 3.10× (2.99–3.22) | 3.04× (2.92–3.16) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.71× (3.69–3.72) | 8.48× (8.45–8.52) | 9.03× (8.98–9.08) | 9.98× (9.94–10.03) | 9.07× (9.03–9.12) | 9.47× (9.43–9.52) | 9.22× (9.18–9.27) | 2.93× (2.92–2.94) | 2.57× (2.56–2.58) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.26× (3.25–3.27) | 7.55× (7.51–7.59) | 8.31× (8.27–8.34) | 9.08× (9.05–9.11) | 7.88× (7.85–7.90) | 8.27× (8.23–8.32) | 8.05× (8.01–8.10) | 2.56× (2.55–2.57) | 2.24× (2.24–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.34× (3.30–3.37) | 6.88× (6.76–7.00) | 7.32× (7.21–7.43) | 8.00× (7.89–8.12) | 7.16× (7.06–7.27) | 5.29× (5.22–5.37) | 5.15× (5.07–5.23) | 1.64× (1.61–1.66) | 1.44× (1.41–1.46) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.71× (0.65–0.77) | 3.36× (3.07–3.67) | 4.47× (4.09–4.89) | 3.98× (3.66–4.33) | 6.75× (6.19–7.35) | 3.12× (2.86–3.41) | 3.15× (2.89–3.42) | 1.13× (1.05–1.22) | 0.92× (0.86–0.99) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.58–0.64) | 3.04× (2.86–3.24) | 5.72× (5.40–6.06) | 3.58× (3.38–3.80) | 5.96× (5.61–6.33) | 2.77× (2.60–2.95) | 2.79× (2.63–2.96) | 1.00× (0.95–1.06) | 0.82× (0.77–0.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.56–0.62) | 3.26× (3.05–3.47) | 5.54× (5.22–5.87) | 3.50× (3.30–3.70) | 5.84× (5.51–6.19) | 2.68× (2.50–2.87) | 2.70× (2.53–2.87) | 0.97× (0.93–1.02) | 0.79× (0.75–0.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.61–2.80) | 3.26× (3.14–3.38) | 3.78× (3.66–3.91) | 4.37× (4.20–4.54) | 4.73× (4.61–4.86) | 3.23× (3.12–3.35) | 2.97× (2.87–3.08) | 2.09× (2.02–2.17) | 2.02× (1.96–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.63× (2.52–2.75) | 3.11× (2.98–3.25) | 3.72× (3.59–3.86) | 4.06× (3.89–4.24) | 4.39× (4.24–4.55) | 2.92× (2.80–3.05) | 2.68× (2.58–2.79) | 1.89× (1.82–1.96) | 1.82× (1.76–1.89) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.39× (2.23–2.55) | 2.18× (2.05–2.32) | 2.73× (2.56–2.91) | 2.68× (2.51–2.86) | 2.79× (2.63–2.96) | 1.70× (1.60–1.82) | 1.57× (1.47–1.67) | 1.10× (1.04–1.18) | 1.06× (1.00–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.47× (1.46–1.48) | 3.22× (3.20–3.25) | 2.92× (2.90–2.94) | 3.06× (3.03–3.08) | 3.38× (3.34–3.42) | 3.71× (3.68–3.75) | 3.85× (3.83–3.88) | 1.34× (1.33–1.34) | 1.32× (1.31–1.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.43× (1.41–1.44) | 3.15× (3.11–3.18) | 2.87× (2.84–2.91) | 3.66× (3.62–3.69) | 3.20× (3.17–3.23) | 3.51× (3.47–3.54) | 3.64× (3.60–3.68) | 1.26× (1.25–1.27) | 1.24× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.70× (1.66–1.74) | 3.26× (3.18–3.35) | 2.79× (2.70–2.89) | 3.61× (3.53–3.70) | 3.23× (3.15–3.31) | 2.61× (2.54–2.68) | 2.71× (2.64–2.78) | 0.94× (0.92–0.96) | 0.93× (0.90–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.51–1.55) | 3.62× (3.60–3.65) | 3.07× (3.05–3.09) | 3.25× (3.24–3.27) | 3.65× (3.63–3.67) | 4.43× (4.40–4.46) | 4.73× (4.69–4.78) | 1.56× (1.56–1.57) | 1.57× (1.56–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.46–1.50) | 3.52× (3.47–3.56) | 3.01× (2.97–3.04) | 4.06× (4.02–4.11) | 3.46× (3.43–3.49) | 4.16× (4.10–4.21) | 4.44× (4.37–4.51) | 1.47× (1.45–1.48) | 1.47× (1.46–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.80× (1.74–1.86) | 3.58× (3.54–3.62) | 2.78× (2.71–2.85) | 3.84× (3.74–3.95) | 3.35× (3.28–3.43) | 2.99× (2.93–3.05) | 3.19× (3.13–3.26) | 1.06× (1.03–1.08) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.72–0.79) | 2.99× (2.76–3.23) | 4.00× (3.75–4.26) | 3.45× (3.24–3.68) | 7.26× (6.84–7.70) | 2.71× (2.52–2.92) | 2.72× (2.53–2.92) | 1.08× (1.03–1.14) | 0.90× (0.84–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.58–0.65) | 2.52× (2.35–2.69) | 4.74× (4.49–5.00) | 3.05× (2.87–3.24) | 6.27× (5.93–6.64) | 2.32× (2.18–2.47) | 2.33× (2.18–2.49) | 0.93× (0.90–0.96) | 0.77× (0.72–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.59) | 2.67× (2.59–2.76) | 4.38× (4.24–4.54) | 2.85× (2.73–2.96) | 5.80× (5.66–5.93) | 2.14× (2.05–2.22) | 2.14× (2.06–2.22) | 0.85× (0.81–0.89) | 0.71× (0.68–0.73) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 1 | 2.07× (2.00–2.14) | 4.31× (4.17–4.46) | 4.90× (4.74–5.06) | 5.02× (4.85–5.19) | 5.09× (4.93–5.26) | 5.24× (5.07–5.41) | 5.11× (4.95–5.27) | 1.67× (1.62–1.73) | 1.55× (1.50–1.60) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 2 | 1.80× (1.76–1.85) | 3.72× (3.62–3.82) | 4.44× (4.33–4.55) | 4.69× (4.58–4.80) | 4.35× (4.26–4.44) | 4.42× (4.31–4.53) | 4.30× (4.20–4.41) | 1.41× (1.37–1.44) | 1.30× (1.27–1.34) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 4 | 1.89× (1.84–1.94) | 3.25× (3.17–3.34) | 3.72× (3.63–3.80) | 3.95× (3.85–4.05) | 3.71× (3.61–3.80) | 2.85× (2.77–2.93) | 2.78× (2.70–2.85) | 0.91× (0.88–0.93) | 0.84× (0.82–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.29× (0.28–0.29) | 0.98× (0.97–0.98) | 0.78× (0.78–0.79) | 11.01× (10.99–11.03) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.36× (0.34–0.37) | 0.55× (0.47–0.64) | 0.73× (0.69–0.76) | 0.62× (0.60–0.65) | 5.85× (5.61–6.11) | 0.14× (0.13–0.15) | 0.15× (0.14–0.16) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.37× (0.35–0.39) | 0.60× (0.49–0.73) | 0.65× (0.63–0.68) | 0.58× (0.55–0.61) | 3.85× (3.66–4.03) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.26× (4.25–4.28) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.93× (0.93–0.94) | 4.84× (4.83–4.85) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.95× (1.94–1.95) | 0.93× (0.93–0.93) | 4.82× (4.81–4.83) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.68× (1.67–1.68) | 0.66× (0.65–0.66) | 7.92× (7.90–7.93) | 0.65× (0.64–0.65) | 0.67× (0.66–0.68) | 0.73× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.61× (0.54–0.68) | 1.60× (1.54–1.67) | 0.81× (0.78–0.84) | 5.83× (5.61–6.05) | 0.47× (0.46–0.49) | 0.49× (0.47–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.67× (0.64–0.70) | 0.70× (0.65–0.76) | 1.20× (1.16–1.24) | 0.74× (0.71–0.77) | 3.57× (3.46–3.69) | 0.28× (0.27–0.29) | 0.29× (0.28–0.30) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.14× (5.13–5.14) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.43) | 0.90× (0.87–0.92) | 0.69× (0.66–0.73) | 3.09× (3.00–3.17) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.65) | 0.47× (0.46–0.48) | 0.88× (0.86–0.90) | 0.83× (0.80–0.85) | 2.56× (2.51–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.26) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.83–0.83) | 5.20× (5.19–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.49) | 0.38× (0.36–0.40) | 0.84× (0.81–0.88) | 0.71× (0.68–0.74) | 2.90× (2.80–3.00) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.62) | 0.48× (0.45–0.50) | 0.80× (0.78–0.82) | 0.83× (0.81–0.85) | 2.31× (2.25–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.28–5.31) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.86× (1.86–1.87) | 0.90× (0.90–0.91) | 6.18× (6.16–6.21) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.86× (1.85–1.86) | 0.90× (0.89–0.91) | 6.16× (6.13–6.19) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.37–0.38) | 1.38× (1.38–1.39) | 0.66× (0.66–0.66) | 6.38× (6.35–6.40) | 0.39× (0.38–0.39) | 0.41× (0.40–0.41) | 0.63× (0.63–0.63) | 0.30× (0.30–0.30) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.35–0.42) | 1.00× (0.98–1.02) | 0.67× (0.65–0.68) | 3.29× (3.22–3.36) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.27 MB → 15.2 MB, 6.7×) | 4 | 57 MB | 0.49× (0.48–0.50) | 0.42× (0.38–0.46) | 0.89× (0.88–0.91) | 0.72× (0.69–0.76) | 2.28× (2.24–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.22× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 564 | 1.78 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 959 | 2.04 | 1.15× (1.14–1.15) | 83% | 1.02× (1.01–1.03) | 1.01× (1.01–1.01) |
| 4: all 4 CPUs, sharing SMT siblings | 1,174 | 3.21 | 1.79× (1.76–1.82) | 52% | 1.45× (1.44–1.46) | 1.61× (1.59–1.64) |

**Best:** mibz2 at 4 threads (1,174 MB/s); lbzip2 at 4 threads (179 MB/s); pbz2 at 4 threads (331 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 0.98, lbzip2 1t 0.04, lbzip2 2t 0.20, lbzip2 4t 0.33, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.52 | 0.02 | 0.09 | 0.178 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3900 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 7.5% |
| MTF + RLE2 | 1.63 | 15.3% |
| T-vector build | 1.54 | 14.5% |
| walk (the one each build uses) | 6.30 | 59.1% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.66 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.937 | (in its total) | 2.51× (2.51–2.52) |  |
| walk | 1.531 | 2.925 | (in its total) | 1.91× (1.91–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.077 | (in its total) | 0.96× (0.95–0.96) |  |
| inverse BWT, total | 1.984 | 3.939 | 2.022 | 1.99× (1.98–1.99) | 1.02× (1.02–1.02) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m05s ± 6m37s |
| 4 threads | 25m54s ± 3m38s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m57s ± 2m41s |
| the scanner alone (its serial part) | 6m50s ± 30.2s |
| the workers alone, over 4 | 18m57s ± 2m41s |

At 4 threads the workers bound it. Per byte: the scanner 4.0 ns a byte of bz2, the workers 2.48 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,273,028 | 15,151,880 | \`bc0075d9b450862b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3899.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1018-aws; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
