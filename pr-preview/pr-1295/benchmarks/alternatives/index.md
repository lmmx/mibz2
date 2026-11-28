# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`540fb14`](<https://github.com/lmmx/mibz2/commit/540fb14282fe53017eaded5ecb870d0e6cabbb3c>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46120353115>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.373 s | 562 | 5.7 | 3.70× (3.64–3.77) | 8.45× (8.29–8.61) | 8.97× (8.80–9.15) | 9.93× (9.77–10.09) | 9.03× (8.85–9.21) | 9.41× (9.24–9.60) | 9.25× (9.08–9.43) | 2.91× (2.86–2.97) | 2.57× (2.52–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.228 s | 919 | 6.7 | 3.37× (3.32–3.43) | 7.29× (7.14–7.45) | 8.80× (8.61–9.00) | 8.70× (8.55–8.84) | 7.90× (7.74–8.06) | 15.66× (15.38–15.94) | 15.39× (15.12–15.66) | 4.85× (4.76–4.93) | 4.27× (4.18–4.37) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.181 s | 1,157 | 10.6 | 3.52× (3.46–3.57) | 6.57× (6.44–6.70) | 7.70× (7.58–7.83) | 7.58× (7.47–7.70) | 7.39× (7.28–7.50) | 19.48× (19.22–19.74) | 19.14× (18.84–19.45) | 6.03× (5.94–6.12) | 5.31× (5.22–5.41) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,227 | 2.9 | 0.64× (0.62–0.66) | 3.13× (3.07–3.19) | 4.46× (4.38–4.55) | 3.74× (3.66–3.81) | 6.35× (6.23–6.48) | 2.84× (2.79–2.90) | 2.87× (2.82–2.91) | 1.12× (1.10–1.15) | 1.26× (1.24–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,153 | 3.2 | 0.54× (0.53–0.56) | 2.97× (2.86–3.09) | 5.94× (5.80–6.07) | 3.56× (3.45–3.68) | 6.00× (5.83–6.18) | 2.70× (2.60–2.80) | 2.72× (2.62–2.81) | 1.06× (1.03–1.10) | 1.20× (1.15–1.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,140 | 3.1 | 0.53× (0.52–0.55) | 2.91× (2.82–3.00) | 5.74× (5.56–5.93) | 3.61× (3.35–3.90) | 5.98× (5.69–6.30) | 2.64× (2.56–2.71) | 2.66× (2.59–2.73) | 1.04× (1.01–1.07) | 1.17× (1.14–1.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 632 | 5.3 | 2.89× (2.85–2.93) | 3.29× (3.26–3.32) | 3.99× (3.95–4.02) | 4.28× (4.24–4.31) | 4.68× (4.64–4.73) | 3.16× (3.14–3.19) | 2.93× (2.89–2.96) | 2.06× (2.03–2.10) | 2.16× (2.04–2.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,034 | 6.2 | 2.55× (2.44–2.67) | 3.49× (3.01–4.06) | 4.59× (4.29–4.91) | 3.93× (3.75–4.13) | 5.32× (5.11–5.54) | 5.09× (4.92–5.27) | 4.71× (4.55–4.88) | 3.32× (3.22–3.42) | 3.48× (3.26–3.72) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,071 | 11.0 | 2.14× (2.02–2.26) | 2.33× (2.20–2.47) | 3.50× (3.34–3.67) | 2.71× (2.51–2.92) | 4.61× (4.40–4.82) | 5.33× (5.08–5.60) | 4.93× (4.70–5.18) | 3.48× (3.30–3.66) | 3.65× (3.38–3.93) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.104 s | 77 | 42.3 | 1.52× (1.52–1.53) | 3.38× (3.35–3.41) | 3.09× (3.06–3.12) | 3.15× (3.14–3.17) | 3.46× (3.44–3.47) | 3.83× (3.81–3.85) | 4.03× (4.01–4.06) | 1.40× (1.39–1.40) | 1.36× (1.36–1.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.061 s | 130 | 45.4 | 1.44× (1.32–1.57) | 3.35× (3.17–3.54) | 3.08× (2.81–3.36) | 3.78× (3.48–4.11) | 3.48× (3.17–3.82) | 6.31× (5.79–6.88) | 6.64× (6.06–7.26) | 2.30× (2.10–2.51) | 2.25× (2.05–2.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.048 s | 167 | 61.6 | 1.62× (1.57–1.67) | 3.15× (3.03–3.27) | 3.04× (2.93–3.16) | 3.73× (3.62–3.85) | 3.71× (3.60–3.81) | 8.09× (7.82–8.38) | 8.51× (8.23–8.81) | 2.95× (2.86–3.05) | 2.88× (2.79–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.117 s | 82 | 39.7 | 1.54× (1.54–1.55) | 3.71× (3.70–3.73) | 3.18× (3.14–3.23) | 3.29× (3.27–3.31) | 3.62× (3.61–3.64) | 4.46× (4.44–4.48) | 4.82× (4.81–4.83) | 1.59× (1.59–1.60) | 1.64× (1.63–1.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 143 | 43.0 | 1.47× (1.44–1.50) | 3.64× (3.38–3.92) | 3.27× (3.20–3.33) | 4.30× (4.24–4.37) | 3.70× (3.64–3.75) | 7.79× (7.68–7.89) | 8.42× (8.30–8.53) | 2.78× (2.75–2.82) | 2.86× (2.83–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 175 | 59.4 | 1.64× (1.57–1.71) | 3.43× (3.30–3.56) | 3.12× (3.03–3.21) | 4.17× (4.07–4.27) | 3.66× (3.55–3.77) | 9.47× (9.19–9.76) | 10.24× (9.94–10.55) | 3.39× (3.29–3.49) | 3.48× (3.37–3.59) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 926 | 4.0 | 0.71× (0.70–0.72) | 2.86× (2.81–2.91) | 3.93× (3.86–4.00) | 3.21× (3.14–3.27) | 6.90× (6.77–7.02) | 2.46× (2.42–2.50) | 2.50× (2.46–2.55) | 1.03× (1.00–1.06) | 0.98× (0.96–0.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 840 | 4.5 | 0.51× (0.50–0.52) | 2.57× (2.51–2.64) | 5.24× (5.11–5.37) | 2.91× (2.84–2.98) | 6.20× (6.08–6.32) | 2.21× (2.16–2.27) | 2.25× (2.19–2.31) | 0.93× (0.90–0.96) | 0.88× (0.86–0.90) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 848 | 4.4 | 0.53× (0.51–0.54) | 2.60× (2.52–2.69) | 5.20× (5.08–5.31) | 2.97× (2.87–3.07) | 6.28× (6.10–6.46) | 2.24× (2.17–2.31) | 2.28× (2.21–2.35) | 0.94× (0.90–0.98) | 0.89× (0.86–0.92) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.056 s | 270 | 12.2 | 2.10× (2.09–2.10) | 4.38× (4.36–4.40) | 5.10× (5.06–5.14) | 5.09× (5.06–5.12) | 5.19× (5.15–5.22) | 5.28× (5.26–5.30) | 5.28× (5.25–5.30) | 1.69× (1.69–1.70) | 1.64× (1.63–1.65) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.035 s | 424 | 14.7 | 1.78× (1.75–1.81) | 3.84× (3.72–3.97) | 5.06× (4.94–5.17) | 5.22× (5.05–5.40) | 4.93× (4.84–5.02) | 8.29× (8.16–8.42) | 8.28× (8.16–8.41) | 2.66× (2.62–2.69) | 2.58× (2.53–2.62) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.031 s | 488 | 22.9 | 1.83× (1.75–1.92) | 3.28× (3.15–3.42) | 4.51× (4.42–4.60) | 4.64× (4.55–4.74) | 4.78× (4.68–4.88) | 9.59× (9.38–9.80) | 9.58× (9.38–9.79) | 3.07× (3.01–3.14) | 2.98× (2.92–3.04) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.69× (3.63–3.75) | 8.40× (8.25–8.56) | 8.92× (8.75–9.09) | 9.88× (9.73–10.04) | 8.99× (8.81–9.17) | 9.37× (9.20–9.55) | 9.21× (9.04–9.39) | 2.90× (2.85–2.96) | 2.55× (2.50–2.59) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.25× (3.21–3.29) | 7.45× (7.36–7.55) | 8.21× (8.13–8.30) | 8.92× (8.81–9.03) | 7.77× (7.65–7.88) | 8.08× (8.00–8.17) | 7.94× (7.84–8.05) | 2.51× (2.48–2.54) | 2.20× (2.17–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.29× (3.22–3.36) | 6.73× (6.61–6.85) | 7.20× (7.11–7.30) | 7.84× (7.73–7.95) | 6.99× (6.89–7.10) | 5.13× (5.06–5.20) | 5.04× (4.97–5.11) | 1.59× (1.57–1.61) | 1.39× (1.37–1.41) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.67× (0.65–0.69) | 2.88× (2.84–2.93) | 3.90× (3.85–3.95) | 3.48× (3.42–3.55) | 5.86× (5.76–5.96) | 2.68× (2.63–2.73) | 2.69× (2.66–2.73) | 1.11× (1.08–1.14) | 0.82× (0.81–0.83) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.69× (2.60–2.78) | 5.17× (5.04–5.30) | 3.23× (3.15–3.33) | 5.38× (5.24–5.52) | 2.46× (2.38–2.54) | 2.47× (2.39–2.56) | 1.02× (0.99–1.05) | 0.75× (0.72–0.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 2.97× (2.90–3.06) | 5.13× (5.01–5.26) | 3.38× (3.08–3.72) | 5.47× (5.20–5.75) | 2.46× (2.40–2.52) | 2.47× (2.42–2.53) | 1.02× (0.99–1.05) | 0.75× (0.73–0.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.76–2.83) | 3.06× (3.04–3.08) | 3.59× (3.56–3.63) | 4.11× (4.08–4.13) | 4.49× (4.46–4.53) | 3.05× (3.03–3.07) | 2.83× (2.80–2.85) | 2.01× (1.98–2.04) | 1.93× (1.89–1.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.56× (2.52–2.60) | 2.84× (2.79–2.90) | 3.48× (3.41–3.55) | 3.66× (3.60–3.71) | 4.01× (3.94–4.07) | 2.63× (2.60–2.67) | 2.44× (2.40–2.47) | 1.73× (1.70–1.76) | 1.66× (1.64–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.00–2.20) | 1.92× (1.88–1.97) | 2.39× (2.32–2.47) | 2.34× (2.27–2.40) | 2.45× (2.38–2.52) | 1.47× (1.43–1.52) | 1.36× (1.33–1.40) | 0.97× (0.94–1.00) | 0.93× (0.90–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.51× (1.51–1.52) | 3.29× (3.27–3.31) | 3.02× (2.99–3.05) | 3.12× (3.10–3.13) | 3.42× (3.40–3.44) | 3.79× (3.77–3.81) | 3.98× (3.96–4.01) | 1.39× (1.39–1.40) | 1.35× (1.34–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.47× (1.46–1.47) | 3.21× (3.19–3.23) | 2.92× (2.90–2.94) | 3.69× (3.67–3.71) | 3.22× (3.19–3.25) | 3.52× (3.49–3.55) | 3.70× (3.67–3.73) | 1.29× (1.28–1.30) | 1.25× (1.24–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.74× (1.70–1.78) | 3.23× (3.12–3.34) | 2.78× (2.72–2.85) | 3.57× (3.47–3.66) | 3.20× (3.13–3.28) | 2.57× (2.50–2.64) | 2.70× (2.63–2.77) | 0.94× (0.92–0.97) | 0.92× (0.89–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.53–1.54) | 3.63× (3.61–3.64) | 3.12× (3.09–3.15) | 3.26× (3.24–3.28) | 3.59× (3.57–3.60) | 4.41× (4.39–4.43) | 4.76× (4.75–4.78) | 1.58× (1.58–1.59) | 1.58× (1.57–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.47–1.49) | 3.51× (3.44–3.58) | 3.00× (2.99–3.02) | 4.01× (3.97–4.05) | 3.37× (3.35–3.39) | 4.08× (4.05–4.12) | 4.41× (4.38–4.45) | 1.47× (1.46–1.48) | 1.46× (1.45–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.79× (1.75–1.84) | 3.59× (3.47–3.72) | 2.79× (2.73–2.86) | 3.86× (3.76–3.96) | 3.31× (3.25–3.38) | 2.96× (2.91–3.01) | 3.20× (3.15–3.26) | 1.07× (1.05–1.08) | 1.06× (1.04–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.75× (0.74–0.76) | 2.51× (2.47–2.55) | 3.47× (3.41–3.54) | 2.90× (2.85–2.95) | 6.08× (5.99–6.17) | 2.25× (2.23–2.28) | 2.29× (2.26–2.33) | 1.03× (1.00–1.06) | 0.76× (0.75–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.57) | 2.12× (2.07–2.18) | 4.10× (4.01–4.19) | 2.61× (2.56–2.66) | 5.34× (5.24–5.43) | 1.97× (1.93–2.01) | 2.01× (1.96–2.05) | 0.90× (0.87–0.93) | 0.67× (0.65–0.69) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.58) | 2.54× (2.40–2.69) | 4.21× (4.12–4.30) | 2.73× (2.65–2.81) | 5.55× (5.40–5.70) | 2.05× (1.99–2.11) | 2.08× (2.03–2.14) | 0.94× (0.90–0.97) | 0.69× (0.67–0.72) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.07× (2.06–2.07) | 4.25× (4.23–4.27) | 4.88× (4.85–4.91) | 4.98× (4.95–5.00) | 5.07× (5.04–5.11) | 5.16× (5.14–5.18) | 5.16× (5.13–5.18) | 1.67× (1.67–1.68) | 1.54× (1.54–1.55) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.79× (1.76–1.81) | 3.67× (3.59–3.74) | 4.36× (4.29–4.44) | 4.63× (4.52–4.74) | 4.26× (4.18–4.33) | 4.26× (4.19–4.33) | 4.26× (4.18–4.33) | 1.38× (1.36–1.40) | 1.27× (1.25–1.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.84× (1.79–1.89) | 3.19× (3.13–3.24) | 3.63× (3.54–3.71) | 3.91× (3.85–3.98) | 3.67× (3.61–3.73) | 2.77× (2.71–2.82) | 2.76× (2.71–2.82) | 0.90× (0.88–0.91) | 0.83× (0.81–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.26–0.29) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 10.98× (10.96–11.01) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.36× (0.34–0.37) | 0.53× (0.45–0.63) | 0.74× (0.72–0.76) | 0.63× (0.60–0.66) | 5.85× (5.67–6.04) | 0.14× (0.13–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 84 MB | 0.36× (0.34–0.38) | 0.55× (0.47–0.65) | 0.63× (0.60–0.67) | 0.56× (0.53–0.58) | 3.73× (3.57–3.91) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.26–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.22) | 0.17× (0.17–0.17) | 1.95× (1.95–1.96) | 0.92× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.94× (1.94–1.95) | 0.92× (0.92–0.92) | 4.82× (4.80–4.83) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.69–1.70) | 0.66× (0.66–0.67) | 7.99× (7.94–8.05) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.70–0.72) | 0.73× (0.69–0.78) | 1.71× (1.69–1.73) | 0.85× (0.84–0.87) | 6.18× (6.09–6.27) | 0.50× (0.49–0.51) | 0.53× (0.52–0.54) | 0.58× (0.57–0.58) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.71) | 0.72× (0.65–0.80) | 1.23× (1.18–1.28) | 0.76× (0.73–0.79) | 3.68× (3.55–3.82) | 0.29× (0.28–0.30) | 0.30× (0.29–0.32) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.15× (5.14–5.15) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.40× (0.37–0.44) | 0.91× (0.88–0.93) | 0.72× (0.70–0.74) | 3.13× (3.04–3.22) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.66× (0.64–0.67) | 0.47× (0.46–0.48) | 0.89× (0.88–0.91) | 0.82× (0.81–0.84) | 2.60× (2.56–2.63) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) | 0.25× (0.25–0.26) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.84) | 5.21× (5.19–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.40) | 0.84× (0.80–0.87) | 0.67× (0.64–0.71) | 2.87× (2.75–2.99) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.45× (0.42–0.48) | 0.80× (0.78–0.81) | 0.80× (0.78–0.83) | 2.29× (2.24–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.20–0.23) | 1.06× (1.06–1.06) | 0.76× (0.76–0.76) | 5.31× (5.29–5.33) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.23) | 1.86× (1.85–1.86) | 0.89× (0.89–0.89) | 6.17× (6.16–6.18) | 0.40× (0.40–0.41) | 0.42× (0.41–0.43) | 1.05× (1.05–1.05) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.23) | 1.85× (1.85–1.86) | 0.89× (0.88–0.89) | 6.14× (6.12–6.16) | 0.40× (0.40–0.40) | 0.42× (0.41–0.43) | 1.05× (1.04–1.05) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.38× (1.38–1.39) | 0.66× (0.66–0.67) | 6.37× (6.34–6.40) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 37 MB | 0.45× (0.44–0.47) | 0.42× (0.38–0.47) | 1.03× (1.01–1.05) | 0.69× (0.67–0.70) | 3.40× (3.32–3.48) | 0.21× (0.20–0.21) | 0.22× (0.22–0.22) | 0.34× (0.33–0.34) | 0.16× (0.16–0.16) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 57 MB | 0.49× (0.49–0.50) | 0.44× (0.40–0.49) | 0.89× (0.87–0.91) | 0.73× (0.72–0.75) | 2.29× (2.26–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 562 | 1.79 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 919 | 2.09 | 1.16× (1.13–1.19) | 83% | 1.03× (1.02–1.03) | 1.02× (1.02–1.02) |
| 4: all 4 CPUs, sharing SMT siblings | 1,157 | 3.31 | 1.83× (1.79–1.87) | 52% | 1.46× (1.44–1.48) | 1.63× (1.61–1.65) |

**Best:** mibz2 at 4 threads (1,157 MB/s); lbzip2 at 4 threads (177 MB/s); pbz2 at 4 threads (327 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 0.97, lbzip2 1t 0.04, lbzip2 2t 0.24, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.55 | 0.02 | 0.08 | 0.181 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3201 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.65 | 7.5% |
| MTF + RLE2 | 1.33 | 15.2% |
| T-vector build | 1.26 | 14.4% |
| walk (the one each build uses) | 5.19 | 59.3% |
| of which: segment assembly | 0.26 | 2.9% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.11 | 1.3% |
| **total** | 8.77 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.937 | (in its total) | 2.51× (2.51–2.51) |  |
| walk | 1.524 | 2.925 | (in its total) | 1.92× (1.92–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.92× (0.92–0.93) |  |
| inverse BWT, total | 1.977 | 3.935 | 2.022 | 1.99× (1.99–2.00) | 1.02× (1.02–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m44s ± 6m43s |
| 4 threads | 26m27s ± 3m44s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m47s ± 2m49s |
| the scanner alone (its serial part) | 7m16s ± 0.00s |
| the workers alone, over 4 | 19m47s ± 2m49s |

At 4 threads the workers bound it. Per byte: the scanner 4.2 ns a byte of bz2, the workers 2.59 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,263,783 | 15,038,430 | \`7639815fbf8abe4e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3200.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1018-aws; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
