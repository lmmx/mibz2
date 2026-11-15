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

The latest record: [`86ec176`](<https://github.com/lmmx/mibz2/commit/86ec176411d900fbd985e77495d489b51b7dab27>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43724255391>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.413 s | 508 | 7.6 | 3.26× (3.23–3.30) | 7.46× (7.37–7.55) | 7.99× (7.92–8.06) | 8.85× (8.78–8.93) | 8.03× (7.95–8.12) | 8.38× (8.29–8.47) | 8.15× (8.07–8.25) | 2.58× (2.55–2.60) | 2.29× (2.26–2.32) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.233 s | 901 | 8.3 | 3.24× (3.15–3.33) | 6.99× (6.79–7.19) | 8.41× (8.18–8.65) | 8.33× (8.12–8.55) | 7.58× (7.37–7.79) | 14.74× (14.35–15.14) | 14.35× (13.98–14.73) | 4.53× (4.42–4.65) | 4.03× (3.92–4.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,072 | 12.9 | 3.30× (3.25–3.36) | 6.01× (5.93–6.10) | 7.14× (7.02–7.26) | 6.98× (6.89–7.06) | 6.86× (6.78–6.94) | 17.65× (17.42–17.88) | 17.19× (16.97–17.41) | 5.43× (5.37–5.49) | 4.82× (4.76–4.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,216 | 3.5 | 0.63× (0.62–0.64) | 3.09× (3.05–3.12) | 4.40× (4.35–4.45) | 3.69× (3.65–3.72) | 6.30× (6.21–6.38) | 2.80× (2.77–2.83) | 2.83× (2.81–2.86) | 1.13× (1.11–1.16) | 1.25× (1.24–1.26) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,136 | 3.8 | 0.52× (0.51–0.53) | 2.92× (2.87–2.97) | 5.88× (5.78–5.98) | 3.52× (3.47–3.58) | 5.99× (5.90–6.08) | 2.64× (2.60–2.69) | 2.68× (2.64–2.72) | 1.07× (1.04–1.10) | 1.18× (1.16–1.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,163 | 3.7 | 0.54× (0.52–0.55) | 2.98× (2.91–3.04) | 5.93× (5.78–6.09) | 3.59× (3.50–3.67) | 6.09× (5.98–6.21) | 2.69× (2.63–2.76) | 2.72× (2.66–2.79) | 1.09× (1.05–1.13) | 1.20× (1.18–1.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 628 | 6.5 | 2.58× (2.55–2.62) | 3.27× (3.25–3.30) | 3.97× (3.94–3.99) | 4.25× (4.22–4.29) | 4.69× (4.67–4.72) | 3.16× (3.13–3.19) | 2.91× (2.88–2.93) | 1.98× (1.94–2.02) | 2.06× (1.96–2.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 920 | 7.7 | 2.06× (1.83–2.31) | 2.77× (2.48–3.09) | 3.84× (3.39–4.35) | 3.51× (3.04–4.05) | 4.65× (4.16–5.20) | 4.44× (3.97–4.96) | 4.08× (3.65–4.56) | 2.77× (2.50–3.08) | 2.89× (2.54–3.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 973 | 13.7 | 1.90× (1.85–1.96) | 2.05× (1.98–2.12) | 3.13× (2.96–3.30) | 2.44× (2.35–2.54) | 4.18× (4.02–4.35) | 4.83× (4.66–5.02) | 4.45× (4.28–4.62) | 3.02× (2.92–3.13) | 3.15× (2.92–3.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.115 s | 70 | 56.4 | 1.36× (1.36–1.37) | 2.99× (2.97–3.01) | 2.74× (2.73–2.75) | 2.78× (2.77–2.79) | 3.09× (3.08–3.10) | 3.39× (3.38–3.40) | 3.52× (3.50–3.53) | 1.24× (1.24–1.25) | 1.24× (1.22–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.071 s | 112 | 61.9 | 1.25× (1.23–1.28) | 2.79× (2.74–2.84) | 2.72× (2.68–2.76) | 3.30× (3.24–3.36) | 3.10× (3.05–3.15) | 5.46× (5.38–5.55) | 5.67× (5.58–5.76) | 2.00× (1.97–2.04) | 2.00× (1.96–2.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.057 s | 140 | 83.8 | 1.39× (1.35–1.43) | 2.72× (2.60–2.84) | 2.62× (2.53–2.72) | 3.21× (3.09–3.34) | 3.22× (3.14–3.30) | 6.80× (6.65–6.96) | 7.06× (6.88–7.24) | 2.50× (2.43–2.56) | 2.49× (2.44–2.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.127 s | 76 | 51.9 | 1.41× (1.41–1.42) | 3.37× (3.35–3.39) | 2.88× (2.86–2.91) | 2.98× (2.96–3.00) | 3.36× (3.34–3.39) | 4.06× (4.04–4.08) | 4.59× (4.57–4.60) | 1.46× (1.45–1.46) | 1.52× (1.51–1.52) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 129 | 56.5 | 1.34× (1.31–1.37) | 3.21× (3.07–3.36) | 2.89× (2.83–2.95) | 3.84× (3.77–3.91) | 3.38× (3.32–3.45) | 6.88× (6.76–6.99) | 7.77× (7.64–7.90) | 2.47× (2.43–2.51) | 2.57× (2.53–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 s | 152 | 79.4 | 1.44× (1.40–1.49) | 2.93× (2.84–3.02) | 2.72× (2.64–2.79) | 3.57× (3.46–3.67) | 3.23× (3.15–3.31) | 8.10× (7.89–8.32) | 9.15× (8.92–9.39) | 2.91× (2.84–2.99) | 3.03× (2.94–3.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 909 | 4.9 | 0.67× (0.65–0.69) | 2.77× (2.72–2.82) | 3.90× (3.70–4.12) | 3.12× (3.06–3.18) | 6.76× (6.65–6.88) | 2.40× (2.35–2.45) | 2.44× (2.39–2.49) | 1.02× (1.00–1.04) | 0.95× (0.93–0.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 851 | 5.3 | 0.51× (0.50–0.52) | 2.61× (2.54–2.67) | 5.30× (5.17–5.43) | 2.95× (2.88–3.03) | 6.43× (6.28–6.57) | 2.25× (2.20–2.30) | 2.29× (2.25–2.34) | 0.96× (0.94–0.98) | 0.89× (0.87–0.91) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 855 | 5.2 | 0.52× (0.51–0.53) | 2.64× (2.60–2.68) | 5.35× (5.28–5.43) | 3.02× (2.96–3.08) | 6.44× (6.35–6.53) | 2.28× (2.24–2.32) | 2.32× (2.28–2.36) | 0.97× (0.95–0.99) | 0.90× (0.88–0.91) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.059 s | 214 | 18.6 | 1.81× (1.80–1.83) | 4.12× (4.09–4.15) | 4.65× (4.62–4.68) | 4.70× (4.66–4.75) | 4.87× (4.83–4.90) | 5.07× (5.03–5.11) | 4.98× (4.95–5.02) | 1.48× (1.47–1.49) | 1.53× (1.49–1.57) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.040 s | 319 | 22.8 | 1.50× (1.41–1.60) | 3.38× (3.22–3.55) | 4.22× (4.03–4.42) | 4.52× (4.34–4.71) | 4.37× (4.19–4.55) | 7.44× (7.16–7.72) | 7.31× (7.04–7.59) | 2.17× (2.09–2.26) | 2.24× (2.15–2.34) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.033 s | 380 | 33.9 | 1.58× (1.54–1.62) | 2.93× (2.78–3.08) | 3.82× (3.71–3.93) | 4.22× (4.13–4.30) | 4.30× (4.20–4.42) | 9.10× (8.87–9.34) | 8.95× (8.73–9.18) | 2.66× (2.59–2.73) | 2.75× (2.65–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.25× (3.22–3.29) | 7.41× (7.33–7.50) | 7.94× (7.87–8.02) | 8.82× (8.74–8.90) | 8.00× (7.91–8.09) | 8.34× (8.25–8.43) | 8.12× (8.03–8.21) | 2.57× (2.54–2.59) | 2.27× (2.24–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.09× (3.02–3.15) | 7.12× (6.94–7.30) | 7.83× (7.67–8.00) | 8.51× (8.33–8.69) | 7.40× (7.24–7.57) | 7.57× (7.40–7.73) | 7.37× (7.21–7.53) | 2.33× (2.28–2.38) | 2.06× (2.01–2.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.26× (3.18–3.34) | 6.56× (6.47–6.66) | 7.06× (6.99–7.14) | 7.66× (7.55–7.78) | 6.84× (6.74–6.95) | 4.93× (4.85–5.01) | 4.80× (4.73–4.87) | 1.52× (1.49–1.54) | 1.34× (1.32–1.36) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.65–0.67) | 2.83× (2.80–2.87) | 3.86× (3.82–3.89) | 3.43× (3.40–3.46) | 5.79× (5.71–5.87) | 2.63× (2.59–2.66) | 2.65× (2.63–2.68) | 1.12× (1.09–1.14) | 0.81× (0.80–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.72× (2.68–2.76) | 5.25× (5.17–5.33) | 3.28× (3.23–3.33) | 5.50× (5.42–5.59) | 2.47× (2.43–2.51) | 2.49× (2.46–2.53) | 1.05× (1.02–1.07) | 0.76× (0.75–0.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.59) | 3.11× (3.01–3.22) | 5.35× (5.25–5.46) | 3.37× (3.29–3.45) | 5.63× (5.52–5.75) | 2.54× (2.48–2.60) | 2.57× (2.51–2.63) | 1.08× (1.04–1.12) | 0.78× (0.77–0.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.47–2.54) | 3.05× (3.03–3.07) | 3.59× (3.56–3.61) | 4.09× (4.06–4.11) | 4.51× (4.49–4.53) | 3.05× (3.02–3.08) | 2.81× (2.79–2.83) | 1.93× (1.89–1.97) | 1.90× (1.88–1.91) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.31× (2.26–2.36) | 2.73× (2.68–2.78) | 3.30× (3.21–3.39) | 3.51× (3.44–3.59) | 3.87× (3.81–3.94) | 2.54× (2.49–2.58) | 2.34× (2.29–2.39) | 1.60× (1.56–1.65) | 1.58× (1.55–1.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.08× (2.01–2.14) | 1.86× (1.80–1.93) | 2.34× (2.26–2.42) | 2.30× (2.23–2.37) | 2.39× (2.33–2.45) | 1.43× (1.40–1.47) | 1.32× (1.28–1.36) | 0.91× (0.88–0.94) | 0.89× (0.87–0.92) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.35–1.37) | 2.91× (2.89–2.92) | 2.66× (2.65–2.68) | 2.75× (2.74–2.76) | 3.06× (3.05–3.07) | 3.35× (3.34–3.37) | 3.48× (3.47–3.49) | 1.24× (1.24–1.24) | 1.22× (1.21–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.31) | 2.80× (2.76–2.85) | 2.55× (2.51–2.59) | 3.22× (3.17–3.27) | 2.86× (2.82–2.91) | 3.03× (2.98–3.07) | 3.14× (3.09–3.19) | 1.12× (1.10–1.14) | 1.10× (1.08–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.52–1.58) | 2.88× (2.79–2.97) | 2.51× (2.46–2.57) | 3.15× (3.06–3.24) | 2.90× (2.85–2.94) | 2.26× (2.23–2.29) | 2.34× (2.31–2.38) | 0.83× (0.82–0.85) | 0.82× (0.81–0.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.41× (1.40–1.41) | 3.28× (3.26–3.31) | 2.82× (2.79–2.84) | 2.95× (2.94–2.97) | 3.33× (3.31–3.35) | 4.02× (4.00–4.03) | 4.54× (4.53–4.55) | 1.45× (1.45–1.46) | 1.46× (1.45–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.34–1.36) | 3.19× (3.17–3.21) | 2.74× (2.72–2.76) | 3.67× (3.64–3.70) | 3.14× (3.12–3.15) | 3.68× (3.65–3.70) | 4.15× (4.12–4.18) | 1.33× (1.32–1.34) | 1.33× (1.32–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.54–1.66) | 3.25× (3.18–3.32) | 2.51× (2.46–2.55) | 3.41× (3.32–3.49) | 3.01× (2.96–3.06) | 2.61× (2.57–2.66) | 2.95× (2.90–3.01) | 0.94× (0.93–0.96) | 0.95× (0.93–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.74) | 2.43× (2.37–2.50) | 3.39× (3.31–3.47) | 2.84× (2.78–2.91) | 6.01× (5.87–6.16) | 2.22× (2.17–2.27) | 2.26× (2.21–2.31) | 1.02× (1.00–1.05) | 0.75× (0.73–0.77) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.20× (2.14–2.25) | 4.27× (4.17–4.36) | 2.69× (2.63–2.76) | 5.61× (5.48–5.74) | 2.04× (1.99–2.08) | 2.07× (2.03–2.11) | 0.94× (0.92–0.96) | 0.69× (0.68–0.70) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.64× (2.55–2.73) | 4.36× (4.31–4.42) | 2.83× (2.77–2.89) | 5.81× (5.71–5.90) | 2.13× (2.09–2.17) | 2.16× (2.12–2.20) | 0.98× (0.96–1.00) | 0.72× (0.71–0.73) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.79× (1.78–1.80) | 3.98× (3.95–4.01) | 4.47× (4.45–4.49) | 4.60× (4.55–4.64) | 4.76× (4.73–4.79) | 4.95× (4.92–4.98) | 4.87× (4.84–4.90) | 1.47× (1.46–1.48) | 1.37× (1.36–1.38) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.56× (1.55–1.58) | 3.45× (3.41–3.50) | 4.06× (4.02–4.09) | 4.30× (4.26–4.34) | 4.02× (3.98–4.07) | 4.04× (4.00–4.07) | 3.97× (3.94–4.00) | 1.20× (1.19–1.21) | 1.12× (1.11–1.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.81× (1.77–1.85) | 3.23× (3.20–3.27) | 3.63× (3.54–3.72) | 3.87× (3.81–3.93) | 3.65× (3.59–3.71) | 2.75× (2.70–2.80) | 2.71× (2.66–2.75) | 0.82× (0.80–0.83) | 0.76× (0.75–0.78) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.28× (0.27–0.29) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.04× (11.02–11.06) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 60 MB | 0.32× (0.31–0.33) | 0.49× (0.41–0.57) | 0.66× (0.64–0.68) | 0.55× (0.53–0.57) | 5.23× (5.08–5.38) | 0.12× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.32× (0.30–0.34) | 0.41× (0.38–0.45) | 0.57× (0.53–0.61) | 0.50× (0.47–0.54) | 3.35× (3.15–3.57) | 0.08× (0.07–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.41–0.44) | 1.70× (1.69–1.70) | 0.65× (0.65–0.66) | 8.01× (7.99–8.03) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.70) | 0.63× (0.58–0.69) | 1.61× (1.56–1.66) | 0.81× (0.78–0.83) | 5.86× (5.69–6.05) | 0.47× (0.46–0.48) | 0.50× (0.49–0.52) | 0.55× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.61–0.64) | 0.65× (0.58–0.72) | 1.13× (1.09–1.17) | 0.69× (0.67–0.72) | 3.36× (3.25–3.47) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.54) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.47–0.51) | 0.40× (0.38–0.43) | 0.89× (0.85–0.92) | 0.70× (0.66–0.74) | 3.06× (2.93–3.19) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.61× (0.60–0.62) | 0.44× (0.44–0.45) | 0.84× (0.82–0.86) | 0.79× (0.77–0.80) | 2.44× (2.39–2.49) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.39× (0.36–0.42) | 0.89× (0.86–0.92) | 0.73× (0.70–0.76) | 3.03× (2.92–3.14) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.61× (0.59–0.62) | 0.47× (0.45–0.50) | 0.82× (0.80–0.84) | 0.84× (0.81–0.87) | 2.36× (2.31–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.27–5.30) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.67) | 6.39× (6.36–6.43) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 40 MB | 0.41× (0.40–0.42) | 0.35× (0.33–0.39) | 0.98× (0.95–1.01) | 0.63× (0.62–0.64) | 3.04× (3.00–3.09) | 0.19× (0.18–0.19) | 0.20× (0.20–0.21) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.47× (0.43–0.52) | 0.91× (0.89–0.93) | 0.74× (0.72–0.75) | 2.31× (2.27–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 508 | 1.98 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 901 | 2.15 | 1.10× (1.09–1.12) | 88% | 1.06× (1.05–1.07) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,072 | 3.35 | 1.69× (1.67–1.71) | 53% | 1.50× (1.49–1.51) | 1.70× (1.66–1.73) |

**Best:** mibz2 at 4 threads (1,072 MB/s); lbzip2 at 4 threads (178 MB/s); pbz2 at 4 threads (324 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.19, lbzip2 1t 0.04, lbzip2 2t 0.24, lbzip2 4t 0.27, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.53 | 0.02 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3862 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 10.7% |
| MTF + RLE2 | 1.62 | 15.2% |
| T-vector build | 1.53 | 14.4% |
| walk (the one each build uses) | 5.96 | 56.1% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.26 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.63 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.371 | 0.935 | (in its total) | 2.52× (2.52–2.52) |  |
| walk | 1.462 | 2.777 | (in its total) | 1.90× (1.90–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.91× (0.91–0.92) |  |
| inverse BWT, total | 1.913 | 3.785 | 1.998 | 1.98× (1.98–1.98) | 1.05× (1.04–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m58s |
| 4 threads | 28m30s ± 4m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m12s ± 2m44s |
| the scanner alone (its serial part) | 10m29s ± 26.3s |
| the workers alone, over 4 | 19m12s ± 2m44s |

At 4 threads the workers bound it. Per byte: the scanner 6.1 ns a byte of bz2, the workers 2.52 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,506,264 | 12,614,346 | \`520a957f38391906…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3862.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
